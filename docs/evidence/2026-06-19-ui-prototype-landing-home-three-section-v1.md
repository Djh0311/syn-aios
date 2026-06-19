# UI 原型落地 · 批 C 首页三段式 evidence v1

日期：2026-06-19

关联目标：
- `docs/plans/2026-06-18-ui-prototype-to-frontend-landing-plan-v1.md`
- 当前 goal：推进批 C 中可控的首页结构改造，同时遵守“变更列先占位，不立项 change-feed”。

## 本轮目标

把首页从 5 个星座入口改成三段式：

- 待处理 hero
- 运行中 + 变更
- 最近工作

保留离线测试 / 辅助技术需要的 sr-only 锚点，避免测试和可访问入口断掉。

## 已改代码

- `src/views/HomeView.tsx`
  - 删除 `HomeNode` 星座卡编排。
  - 新增待处理 hero：
    - 聚合 `runtime_session_attention.requires_user_action`
    - 聚合 `ready_for_review` 工作项
    - 聚合未批准权限请求
    - 提供“去确认 / 去处理 / 去审批”三个入口
  - 新增运行中面板：
    - 展示运行/等待/复核/重试相关 workflow
    - 展示运行关注 session
  - 新增变更面板：
    - 明确显示 change-feed 未接入
    - 不伪造变化数据
  - 新增最近工作：
    - 最近项目取当前索引更新时间排序
    - 保留 Skill / Harness 入口
  - 保留 sr-only 文本：
    - `最近项来自索引近似口径，不是真实使用事件。`
    - `Skill`
    - `Harness`
    - `运行中工作流`
    - 打开智能体按钮
- `src/styles.css`
  - 新增三段式首页样式：
    - `.home-workbench-stage`
    - `.home-action-hero`
    - `.home-now-grid`
    - `.home-panel`
    - `.home-feed`
    - `.home-recent-work`
    - `.home-entry-tile`
    - `.home-warning-strip`

## 验证

在 `prototypes/productized-desktop-shell` 下执行：

- `npm run typecheck`：通过
- `npm run test:offline-interaction`：通过，15 个 offline interaction tests + r4 page read model / selectors tests 通过
- `git diff --check`：通过

## 未做 / 暂停

- 未实现 change-feed。
- 未新增上次访问基线。
- 未做首页写操作。
- 未做右栏按项目分组。
- 未做项目页一屏收纳 / 折叠骨架。
- 未碰智能体页、知识库整页方向。

## 风险

- “最近工作”仍是索引近似口径，不是真实使用历史；sr-only 文本继续说明这点。
- 首页视觉从星座改为信息面板，需要真机看密度和空态文案是否舒服；本轮只做本地类型和离线交互验证。
