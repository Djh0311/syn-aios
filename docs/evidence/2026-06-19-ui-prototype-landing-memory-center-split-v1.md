# UI 原型落地 · MemoryCenterView 拆瘦 evidence v1

日期：2026-06-19

关联目标：
- `docs/plans/2026-06-18-ui-prototype-to-frontend-landing-plan-v1.md`
- 当前 goal：继续完成剩余可落地的 UI landing/workbench 改造项；不改智能体页、不改知识库整页方向。

## 本轮目标

继续执行计划 §10 “先拆再翻”。`MemoryCenterView.tsx` 仍是剩余前端巨石之一，本轮先拆展示组件和 action 构建逻辑，降低后续 UI 收纳/打磨风险。

本轮只搬代码，不改变记忆页行为，不新增写操作，不改变确认边界。

## 已改代码

- `prototypes/productized-desktop-shell/src/views/MemoryCenterView.tsx`
  - 从 1151 行降到 676 行。
  - 保留页面状态、派生 read model、用户动作回调和主布局。
  - 删除已搬出的列表项组件和生命周期 request helper。
- `prototypes/productized-desktop-shell/src/views/memory/MemoryListPanels.tsx`
  - 新增正式记忆、候选记忆、实体候选、去重候选、关系候选、已确认关系、成熟模式候选、跨项目主题报告、M1-M12 摘要等纯展示组件。
- `prototypes/productized-desktop-shell/src/views/memory/MemoryActionBuilders.ts`
  - 新增 formal memory lifecycle request builder。
  - 新增项目 / workflow 上下文、确认摘要、成熟模式决定文案、稳定 ID helper。

## 验证

在 `prototypes/productized-desktop-shell` 下执行：

- `npm run typecheck`：通过
- `npm run test:offline-interaction`：通过，15 个 offline interaction tests + r4 page read model / selectors tests 通过
- `git diff --check`：通过

## 未做 / 暂停

- 未改记忆页视觉。
- 未改 lifecycle 操作语义。
- 未改实体 / 关系 / 成熟模式确认路径。
- 未碰智能体页。
- 未碰知识库整页方向。

## 风险

- 这是结构性搬家，风险主要是漏 import 或组件导出；已由 typecheck 和离线交互测试覆盖。
- `styles.css` 仍是巨石，后续如果继续大规模 UI 调整，样式拆分仍需要单独处理。
