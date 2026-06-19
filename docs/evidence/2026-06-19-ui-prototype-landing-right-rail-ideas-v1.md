# UI 原型落地 · 批 B 右栏想法箱入口 evidence v1

日期：2026-06-19

关联目标：
- `docs/plans/2026-06-18-ui-prototype-to-frontend-landing-plan-v1.md`
- 当前 goal：在“先拆再翻”后，推进批 B 复用现成机制的低风险入口项。

## 本轮目标

按计划 §10.2 “四个占位页入口都先做好”与 §5.1 “通知 / 待办 / 审计保持右栏小抽屉”，先把想法箱接入右栏常驻抽屉。

本轮只做入口和只读摘要，不做真实想法数据源、不创建任务、不写事实、不写正式记忆、不触发派发。

## 已改代码

- `src/lib/workbenchNavigation.ts`
  - `RightPanelKey` 增加 `ideas`。
  - `workspaceRailItems` 增加右栏 chip：
    - label：`想法`
    - glyph：`想`
- `src/components/RightDetailPanel.tsx`
  - 增加 `ideas` 面板标题：`想法箱`。
  - 增加摘要列表标题：`想法线索`。
  - 增加只读边界文案：只收纳可见线索，不创建任务、不写事实、不替代用户确认。
  - 从现有数据派生想法线索：
    - `snapshot.tasks`
    - `secretaryContext.suggestions`
    - `project.context_warnings` / `project.warnings`
  - 增加“打开想法箱”按钮，跳到现有左侧 `ideas` 占位页。
- `tests/helpers/offlineReadModelContractFixtures.ts`
  - 右栏非秘书面板枚举增加 `ideas`。
- `tests/helpers/offlineRightRailFixtures.ts`
  - 右栏摘要标题 fixture 增加 `ideas: "想法线索"`。

## 验证

在 `prototypes/productized-desktop-shell` 下执行：

- `npm run typecheck`：通过
- `npm run test:offline-interaction`：通过，15 个 offline interaction tests + r4 page read model / selectors tests 通过
- `git diff --check`：通过

## 未做 / 暂停

- 未做真实想法箱数据模型。
- 未做想法新增、归档、转任务等写操作。
- 未做右栏按项目分组。
- 未调整左栏分组 / 顶栏。
- 未碰智能体页、知识库整页方向。

## 风险

- 当前想法箱面板是只读线索聚合，用已有任务、秘书建议和项目 warning 充当入口内容；这符合“入口先立，功能后填”，但不能冒充完整想法箱。
- 右栏 chip 数量增加后，窄屏视觉需要后续浏览器验收；本轮只做本地类型和离线交互验证。
