# UI 原型落地 · 项目资料/资源收纳 evidence v1

日期：2026-06-19

关联目标：
- `docs/plans/2026-06-18-ui-prototype-to-frontend-landing-plan-v1.md`
- 当前 goal：继续完成剩余可落地的 UI landing/workbench 改造项；不改智能体页、不改知识库整页方向。

## 本轮目标

按计划 §3 “整页一屏、超出收纳”的方向，先处理项目页中较安全的次级资料区：交接 / 证据 / 权威、资源。

本轮不重构工作流执行面板，不移动横向 tab，不改变项目事实层。

## 已改代码

- `prototypes/productized-desktop-shell/src/views/projects/ProjectReferencePanels.tsx`
  - `ProjectHandoffEvidencePanel`
    - compact 模式保持直接展示摘要。
    - 完整页增加 `details.project-disclosure`，默认按文件数量决定是否展开。
    - summary 文案：`展开完整资料索引`。
  - `ProjectResourcesPanel`
    - 资源详情进入 `details.project-disclosure`。
    - summary 文案：`展开资源详情`。
- `prototypes/productized-desktop-shell/src/styles.css`
  - 增加 `.project-disclosure` 样式，保留原项目页轻量线框风格。
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
  - 增加断言，防止项目资料/资源 disclosure 被后续重构误删。

## 验证

在 `prototypes/productized-desktop-shell` 下执行：

- `npm run typecheck`：通过
- `npm run test:offline-interaction`：通过，15 个 offline interaction tests + r4 page read model / selectors tests 通过
- `git diff --check`：通过

## 未做 / 暂停

- 未重构项目工作流执行面板。
- 未把项目横向 tab 改左侧栏。
- 未改变任何写入 / 派发 / 真实执行路径。
- 未碰智能体页。
- 未碰知识库整页方向。

## 风险

- 这是“一屏收纳”的第一层骨架，只覆盖项目资料和资源两个次级面板；工作流侧的深层执行/治理区域仍然需要后续更谨慎地拆分和收纳。
