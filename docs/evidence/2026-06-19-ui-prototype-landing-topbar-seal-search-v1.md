# UI 原型落地 · 顶栏印章与搜索收口 evidence v1

日期：2026-06-19

关联目标：
- `docs/plans/2026-06-18-ui-prototype-to-frontend-landing-plan-v1.md`
- 补充 `docs/evidence/2026-06-19-ui-prototype-landing-topbar-recent-project-v1.md` 中当时未完成的顶栏印章 / 搜索项。

## 本轮目标

收口计划 §1.2 顶栏剩余项：

- 顶栏增加可回首页的印章按钮。
- 搜索从顶栏中心大输入收成右侧小搜索控件。
- 移除搜索可见占位文案。
- 保留搜索受控输入和过滤能力。
- 保留此前已落地的最近项目 chip。

## 已改代码

- `prototypes/productized-desktop-shell/src/components/WorkbenchShell.tsx`
  - `WorkbenchTopbar` 左侧增加 `topbar-seal-button`，点击回到 `home`。
  - 移除左栏原品牌按钮，避免左栏和顶栏重复出现印章。
  - 搜索框移入 `topbar-actions`，删除 `placeholder` 和 `kbd`。
  - 搜索输入仍使用 `query` / `onQueryChange`，不改变过滤语义。
- `prototypes/productized-desktop-shell/src/styles.css`
  - 增加 `.topbar-seal-button` 点击尺寸、hover、focus 样式。
  - 桌面水墨 shell 下将 `.shell-topbar .search-box` 收为 112px 小控件。
  - 让顶栏印章占据左侧，最近项目 / 搜索 / 待审 / 刷新 / 健康状态保留在右侧操作区。

## 验证

在 `prototypes/productized-desktop-shell` 下执行：

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，15 个 offline interaction tests + r4 page read model / selectors tests 通过。
- `git diff --check`：通过。

## 边界

- 未新增真实最近访问历史；最近项目仍沿用当前索引项目列表第一项。
- 未改智能体页；智能体页属于 `2026-06-19-conversation-shell-codex-layout-refactor-plan-v1.md`。
- 未改知识库整页方向。
- 未做真机 / browser 视觉验收；按当前约定由用户验收。
