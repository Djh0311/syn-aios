# UI Prototype Landing Batch A / Shell Split Evidence v1

日期：2026-06-19
范围：`prototypes/productized-desktop-shell`
对应计划：`docs/plans/2026-06-18-ui-prototype-to-frontend-landing-plan-v1.md`

## 本轮目标

- 按 2026-06-19 审后修订执行：智能体页不纳入本方案；结构翻新前先拆瘦。
- 先做不改变功能的外壳拆分，为后续左栏分组、顶栏、右栏抽屉和占位页转真页建立稳定落点。
- 先落批 A 低风险叶子：dock 去占位、首页去可见大标题栏、记忆页生命周期入口收纳。

## 已落改动

1. App 外壳拆瘦
   - 新增 `src/components/WorkbenchShell.tsx`，承载 topbar / sidebar / main / right rail / dock / permission dialog / secretary float。
   - `src/App.tsx` 保留状态、数据加载、action 确认和 `renderActiveView`，外壳 JSX 下沉到 `WorkbenchShell`。
   - `App.tsx` 当前约 903 行；拆分前本轮读取时约 1104 行。

2. 占位页组件拆出
   - 新增 `src/components/SourceStylePlaceholder.tsx`。
   - `ideas / proposal / tools / models` 继续使用同一只读占位结构；这是后续“四个入口先立、功能后填”的落点。

3. 批 A 叶子
   - `HomeView` 删除可见 `stage-head home-page-head` 标题栏；保留 sr-only 测试锚点。
   - dock 输入框去掉 placeholder；保留 `aria-label` 和快捷 chip。
   - `MemoryCenterView` 的正式记忆生命周期操作从 9 个平铺按钮收纳为：
     - 常驻：`编辑提案`
     - `版本` 菜单：`冻结 / 解冻`
     - 朱砂 `秘书建议`：`上升为全局 / 合并 / 归档`
     - `更多`：`废弃 / 拆分 / 下沉为项目`
   - 生命周期底层 action、preview、permission dialog 逻辑未改。

## 验证

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过。
- `git diff --check`：通过。
- 浏览器预览只读检查：
  - app shell 区域正常挂载。
  - 首页可见标题栏已移除。
  - dock input placeholder 为 `null`，`aria-label` 保留。
  - 记忆页浏览器预览样例无正式记忆数据，生命周期区由离线夹具覆盖。

## 未做 / 仍受阻

- 未做首页三段式结构重构、右栏按项目分组、项目页 3 格状态条、项目页一屏收纳：这些属于批 C/D，仍应等拆瘦继续推进后再动。
- 未碰智能体页：该页由 `2026-06-19-conversation-shell-codex-layout-refactor-plan-v1.md` 单独负责。
- 未碰知识库整页方向。
- `styles.css` 仍是巨石，后续需要继续拆样式落点。

## 下一步建议

1. 继续拆 `styles.css` 中外壳 / 首页 / 记忆页 / 智能体页样式段，至少先为外壳和批 A 页建立可维护 CSS 分区。
2. 再拆 `MemoryCenterView` 的详情区和生命周期组件，使记忆页低于巨石阈值。
3. 完成拆瘦后再进入批 B/C：左栏分组、顶栏改造、右栏 ideas chip、首页三段式和项目页 3 格状态条。
