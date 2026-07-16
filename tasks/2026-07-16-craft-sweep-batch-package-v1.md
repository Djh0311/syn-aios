# 任务包:手艺扫除批——全页按七律扫 v1

日期:2026-07-16 · 档位:**轻档**(纯前端样式/结构收敛·后端零碰) · 执行者:前端施工对话 · 背景:手艺七律已入宪(`prototypes/productized-desktop-shell/DESIGN.md` §五),交办页已按律重造并冻结(commit `b0ea8a5`)=**唯一样板**。

**必读**:DESIGN.md §五(七律)、对照稿 `docs/design/2026-07-15-craft-rules-and-jiaoban-target-v1.html`、交办页样板实现(`src/views/projects/projectWorkflowSidePanel.css` 的 jiaoban-merged 段 + `jiaoban/JiaobanAuthorizeStates.tsx`)、`docs/agent-mistake-ledger.md` M-2026-07-16(实渲自查铁律)。

## 范围(按用户走查频度排序,每页一波)

S1 首页(HomeView) → S2 记忆中心(MemoryCenterView·B1) → S3 智能体页(AgentView/AgentSessionList/AgentConversationShell) → S4 技能+harness 双板(SkillsBoardView/HarnessBoardView) → S5 审计账本页(AuditLedgerView) → S6 设置+想法箱(SettingsView/想法箱) → S7 项目总览(ProjectOverviewPanels)。

**交办页零碰(07-16 用户拍冻结)**;工作流页/画布 monolith 不在本包。

## 每页七律逐条(样板=交办页的做法)

1. **纸上放卡**:区域级白底与框中框排查——承载区/列容器一律透明,白面退场,内容卡=纸底勾线(hairline+圆角+透明底,样板:`.jiaoban-merged-region` 透明 + `.project-jiaoban-col` 勾线);一栏至多一个框。
2. **虚线只做占位**:容器虚线边框清除;虚线只留给空态/将来有物(样板:`.jiaoban-preview-ghosts`)。
3. **统一网格**:卡内边距 20px;组间距 14px;行内节拍 4/8/12px;密排列表区参照批卡终态 8px;左缘一条线拉到底。
4. **小签唯一定式**:pill/徽标收敛为 11px·2px 9px·全圆角·语义 token 色(样板:`.jiaoban-chip`);各页自造的 pill 形态清点后统一。
5. **事实行定式**:标签灰左·值右·细虚线分隔(样板:`.jiaoban-fact`);项目总览事实卡等照此。
6. **长单分层**:人读条目直排;机器/治理细则收「N 条 ▸」展开(展开≠截断)。
7. **一代化残留清扫**:全仓 grep 朱红/vermil 的选中·hover·active 残留(`styles.css` Inkwash 段的 `.entry-card:hover/.session-card.active` 等 rgba(161,66,66,…) 家族)收敛到石绿系(--accent/--accent-bg 既有 token);`border-radius: 0` 方钮残留同扫。**只动交互态色,米纸底/宋体/朱红 danger 语义保留**(DESIGN §五⑦)。

## 红线与纪律

- token 不引新色值;交办页与 `src-tauri/**` 零碰;发现信息级问题(该删的字段/缺的动作)**报回不自拍**(信息规范的板在总指导/用户)。
- 每页收口:`npm run typecheck` + `npm run test:offline-interaction` 全过;断言只因旧语义锁死而更新、不许删;无 hooks 约定。
- **实渲自查铁律(ledger M-2026-07-16)**:自查必须从真实组件链最外层渲入(整页壳/面板层),真机栏宽,禁止手拼替身;回传注明渲染链从哪层入。
- 回传每页一段:改了什么/七律逐条对照/改动落在哪些文件(**标明 css 文件名**——WKWebView 对部分 css 热更不可靠,用户真机验收需脚本重启,回传要提示);不 commit;全部完成总回传,shape gate 三数仓根跑(基线 13/5/5)。
