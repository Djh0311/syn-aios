# 工作流画布·全屏 HUD 重做 — 进度交接（在飞 · 2026-06-23）

> 在飞交接：部分做完、真机部分验、有 bug 在修、决策点待定。下一程接手先读本文 + `CURRENT.md` + 方案。**未提交 git。**

**方案**：`docs/plans/2026-06-23-workflow-canvas-fullbleed-hud-redesign-v1.md`
**验证模型**：渲染类，主导线看不到 Tauri 渲染（computer-use 抓不到 dev 二进制），**真机由用户逐阶段 Cmd+R 验**，机器绿 ≠ 真机。

## 已做（机器层全绿：typecheck 0 / offline 15+r4 / build 过）
1. **项目界面全窗重做**：原来 `ProjectWorkspaceShell` 纵向堆 头部+状态条+4 入口 tab+内容，画布铺不满。现改成全窗——项目 chrome（返回/项目名/状态 pill/**4 入口**）收进顶边 `.project-hud-top` 悬浮；`.project-layout` 绝对定位填窗；高度链建在定高根 `.project-detail-shell`（`height:calc(100vh-…)`）上，避开"高度塌 0"。
2. **四边 HUD（P2/P3）**：调色板→左、选中节点面板→右（没选空出）、运行/保存→底、4 入口/选择器→顶；画布自己的动作 HUD 从顶挪到底（避开项目顶边 HUD）；过程内容进按需「详情抽屉」。共享引擎 `WorkflowCanvasEngine`（实验+项目都用）只搬位不动数据层。
3. **详情抽屉第一波精简**：侧栏 14 个面板——节点详情拆常驻+「更多」折叠；工作项编排卡只留派发/汇报/回收、其余折；候选治理条只留记忆候选+正式记忆、其余折；统一执行/全局边界复核/方案授权摘要/两条边界声明 默认折。**注：以"折"为主、几乎没真删**——离线断言保护这些治理内容，真删会挂测试（"折"则 visibleText 仍可达、断言照过）。
4. **真跑/安全逻辑一字未动**：`接执行`、不自动执行、不写 `.codex`、旧派发已封存、读回不撒谎、真执行标志——全部只挪显示位置。纯前端，零后端/双闸/沙箱/manual_relay。

## 真机状态（用户 Cmd+R 实测）
- ✅ **画布铺满整窗**：用户报 `.react-flow` = 1174×668（668 = calc 高度，链通了）。
- ⏳ **四边 HUD 分布 + 砍一波精简**：机器绿，**真机未逐项确认**（用户先撞上下面的 bug）。
- 🐛 **详情面板背景透明** → 已修（浮层加纸面底+边框+阴影），真机未复确认。
- 🐛 **详情面板能横滑 + 宽度只剩一半** → 修了两轮：①外层 `overflow-x:hidden`+长内容换行（仍能横滑）②内层 `.project-canvas-side-panel` 原来也是 `overflow:auto+max-height` 双层滚动容器、改 `overflow:visible`。**第②轮用户喊停、真机未确认**。

## 待办 / 给咨询的决策点
1. **横滑 bug 收口**：若第②轮仍能横滑，下一步**别再猜**——按记忆给面板加 scrollWidth 测量读数，量出哪个子元素撑宽（多半某张表/固定列网格），定点改。
2. **详情抽屉"真删"清单**：第一波是"折"。用户要的是筛掉一批。需用户/咨询定**哪些折起的块要彻底删**（彻底删要同步改离线 fixture，在安全门内、不弱化安全断言）。早先用户已倾向：节点详情再精简、工作项编排卡和候选治理条两个胖块继续砍、4 条边界声明/重叠摘要可砍。
3. **四边 HUD + 精简的真机逐项验**还没走完（背景/横滑 bug 插队了），收口前要补。

## 收尾前要做
- 删临时 `CanvasDebugReadout`（验完才删，现在留着校准）。
- **未提交 git**：按 `AGENTS.md`，commit 要带 `CURRENT.md` 回写（把这条从"在做"挪入①、刷新③）+ 问一次。

## 改动文件（均前端，零后端）
`ProjectWorkspaceShell.tsx` / `ProjectWorkflowCanvasView.tsx` / `WorkflowCanvasEngine.tsx`（共享，注意实验画布回归）/ `ProjectWorkflowSidePanel.tsx` / `ProjectWorkflowExecutionPanels.tsx` / `ProjectWorkflowMemoryPanels.tsx` / `styles.css` / `projectWorkflowSidePanel.css` / `tests/helpers/offlineShellScenarioTextFixtures.ts`
