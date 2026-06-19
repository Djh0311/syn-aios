# UI 原型落地 · 项目资料/资源样式拆分 evidence v1

日期：2026-06-19

关联目标：
- `docs/plans/2026-06-18-ui-prototype-to-frontend-landing-plan-v1.md`
- 当前 goal：继续完成剩余可落地的 UI landing/workbench 改造项；不改智能体页、不改知识库整页方向。

## 本轮目标

继续执行“先拆再翻”的 CSS 拆瘦方向。先把本轮项目资料/资源收纳相关样式从主 `styles.css` 抽到项目模块 CSS，减少主样式文件继续膨胀。

本轮只移动选择器，不改视觉规则。

## 已改代码

- `prototypes/productized-desktop-shell/src/views/projects/projectReferencePanels.css`
  - 新增项目总览、交接证据、资源、资料 disclosure 的样式。
- `prototypes/productized-desktop-shell/src/main.tsx`
  - 在全局样式后导入 `./views/projects/projectReferencePanels.css`。
- `prototypes/productized-desktop-shell/src/styles.css`
  - 删除同一批项目资料/资源选择器，避免重复定义。
  - 行数从 9123 降到 9026。

## 验证

在 `prototypes/productized-desktop-shell` 下执行：

- `npm run typecheck`：通过
- `npm run test:offline-interaction`：通过，15 个 offline interaction tests + r4 page read model / selectors tests 通过
- `git diff --check`：通过
- `rg "project-disclosure|project-file-columns|project-resource-grid|project-overview-grid" src/styles.css src/views/projects/projectReferencePanels.css`：这些 selector 只保留在新 CSS 文件。

## 未做 / 暂停

- 未拆完整项目工作流画布样式。
- 未拆全局 `styles.css` 的其他页面样式。
- 未改智能体页。
- 未改知识库整页方向。

## 风险

- `styles.css` 仍有 9026 行，样式巨石仍存在；本轮只是低风险第一刀。
- 后续继续拆 CSS 时要注意 import 顺序和媒体查询覆盖关系，不能机械移动。
