# UI 原型落地 · 批 B 顶栏最近项目入口 evidence v1

日期：2026-06-19

关联目标：
- `docs/plans/2026-06-18-ui-prototype-to-frontend-landing-plan-v1.md`
- 当前 goal：推进批 B 外壳入口改动，同时不引入无数据源的伪功能。

## 本轮目标

按计划中的“顶栏加最近项目 chip”先做低风险版本。

本轮不做访问历史，不写 localStorage，不新增数据源；“最近项目”暂取当前索引项目列表中的第一项，只作为快速进入项目页的入口。

## 已改代码

- `src/components/WorkbenchShell.tsx`
  - `WorkbenchTopbar` 增加 `onActiveViewChange`。
  - 顶栏右侧增加 `project-switch` 按钮：
    - 文案：`最近项目`
    - 展示：`displaySnapshot.projects[0].name`
    - title：项目路径
    - 点击：跳转到 `projects` 视图
- `src/styles.css`
  - 给 `.project-switch` 增加按钮态样式、最大宽度和文本省略。
  - 保持搜索框原有受控过滤，不缩成纯图标。

## 验证

在 `prototypes/productized-desktop-shell` 下执行：

- `npm run typecheck`：通过
- `npm run test:offline-interaction`：通过，15 个 offline interaction tests + r4 page read model / selectors tests 通过
- `git diff --check`：通过

## 未做 / 暂停

- 未实现真实最近访问历史。
- 未写 localStorage。
- 未移动品牌印章到顶栏。
- 未把搜索框缩成图标态。
- 未碰智能体页、知识库整页方向。

## 风险

- 当前“最近项目”名称来自索引列表第一项，不代表真实用户最近访问项目；后续若要严格语义，需要建立访问历史或最后打开项目状态。
- 顶栏新增 chip 后仍需真机视觉看一眼长项目名和窄窗口表现；本轮用 `max-width` 和 ellipsis 做了基础保护。
