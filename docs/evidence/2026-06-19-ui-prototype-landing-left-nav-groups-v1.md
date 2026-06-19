# UI 原型落地 · 批 B 左栏分组 evidence v1

日期：2026-06-19

关联目标：
- `docs/plans/2026-06-18-ui-prototype-to-frontend-landing-plan-v1.md`
- 当前 goal：在“先拆再翻”后推进批 B 复用现成机制的外壳入口改动。

## 本轮目标

按计划的左栏分组方向，把主导航从纯平铺改成分组结构。

当前真实前端左栏是窄 rail，桌面态只显示图标，文字通过 hover / focus 标签显示。因此本轮不强行改成宽菜单，只做结构分组和组间分隔，避免把 56/72px 左栏塞乱。

## 已改代码

- `src/lib/workbenchNavigation.ts`
  - 新增 `WorkbenchNavGroup`。
  - 新增 `primaryNavGroups`：
    - `主入口`：项目、智能体
    - `流转`：想法箱、运行中工作流
    - `积累`：知识库、记忆层
    - `中枢`：Skill、Harness
  - 保留原 `primaryNavItems`，兼容现有测试和其他调用方。
- `src/components/WorkbenchShell.tsx`
  - 左栏从 `primaryNavItems.map` 改为 `primaryNavGroups.map`。
  - 每组用 `section.nav-group` + `aria-label` 表达分组语义。
  - 每个 nav item 的 label / glyph / active / onClick 保持原行为。
- `src/styles.css`
  - 增加 `.nav-group` 和 `.nav-group + .nav-group`。
  - 只加组间 dashed 分隔，不改变左栏宽度。
  - 桌面覆盖段同步 `.nav-group` 分隔。

## 验证

在 `prototypes/productized-desktop-shell` 下执行：

- `npm run typecheck`：通过
- `npm run test:offline-interaction`：通过，15 个 offline interaction tests + r4 page read model / selectors tests 通过
- `git diff --check`：通过

## 未做 / 暂停

- 未把左栏改成宽菜单。
- 未把开发入口（建议方案 / 实验画布 / 工具 / 模型）提升到左栏主导航；仍保持在设置页开发者区。
- 未做顶栏“最近项目 chip / 搜索缩小 / 印章迁移”。
- 未碰智能体页、知识库整页方向。

## 风险

- 这是窄 rail 语义分组，不是视觉上完整的文字分组菜单；用户真机看起来会更有分隔，但不会像宽侧栏那样直读所有组名。
- 如果后续决定左栏加宽，需要另做视觉验收，尤其要重看 project / agent stage 的可用宽度。
