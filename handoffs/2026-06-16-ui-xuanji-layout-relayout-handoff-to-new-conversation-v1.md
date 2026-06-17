# 交接：工作台 UI 布局重整（采用 Xuanji 布局结构、保留水墨风格）· 给新对话 v1

日期：2026-06-16
出自：咨询线（Claude，另一会话）。性质：**把这件 UI 布局重整交给你（新对话）接手**。你冷启动、没有上一会话的记忆，本文自洽,照它接上即可。

---

## 0. 接手须知（先读这条）

- 你的任务：把工作台的 **UI 布局结构** 重整成 **Xuanji 的布局**，**保留现有水墨（inkwash）视觉风格**。
- **先跟用户细聊、再动手。** 这是产品/设计决策，不是机械任务。先和用户对齐"到底改哪、改到什么程度、和我们已定的 IA 怎么调和"，**别上来就大改**。
- **看不见 UI 的硬限制**：上一会话所在环境跑不了浏览器/Tauri（系统 Chrome headless `SIGABRT`），**盲改 UI 很危险**。你要么让用户**截图**现状给你看，要么用**预览类 MCP**（如 `Claude Preview` / `visualize`）把前端渲染出来对着改。**UI 看图工具口径（2026-06-17 用户拍板）：用 `Claude Preview` MCP 当 agent 的"眼睛"（截图 / 页面结构快照），暂不自建 MCP 工具**（backlog「UI 视觉反馈 MCP 工具」已标暂缓）。注：之前"用户做过一个打磨 UI 的 MCP 工具、去问用户要"是误记——该工具并不存在，别去找；直接用 Claude Preview。
- 走治理流程：任务包 + 独立复核 + **提交前问用户一次**（见 `AGENTS.md`）；子线不 `git add/commit`。

## 1. 问题（用户原话口径）

- 用户觉得当前 UI **看着难受**，但澄清过：**难受的是"布局结构"，不是视觉**。
- **水墨（inkwash）视觉风格用户觉得"现在挺不错的"，保留不动。**

## 2. 方向（已确认）

**用 Xuanji 的 UI 布局结构 + 保留水墨视觉风格。** 即：换骨架、不换皮。

## 3. 参照实物（路径）

- **Xuanji 布局正本**：`/Users/yoyi/Documents/Codex/2026-05-26/gan-xing-codexbridge-https-github-com/docs/research/xuanji-ui-design-extraction-report.md`（1564 行，UI 布局/IA 提炼）；源码快照 `…/docs/research/xuanji-ui-source-snapshot-2026-06-10/`。
- **水墨风格/现状原型**：`/Users/yoyi/Documents/Codex/2026-05-26/gan-xing-codexbridge-https-github-com/docs/architecture/ui-prototype/inkwash-full.html`（3593 行）。**风格好、保留**；它的布局是痛点来源。同目录有 `styles.css` / `app.js` / `HANDOFF.md`。
- **已决定的 IA**：`product-line/decisions/2026-05-28-codex-workbench-ui-ia-direction.md`（首页四入口 = Agent/项目/Skill/Harness；项目进去是**可视化工作流画布**；看板做辅助；左窄功能列 + 中画布 + 右详情）。配套 `product-line/docs/plans/2026-06-08-workbench-ui-information-architecture-and-developer-settings-plan-v1.md`。**这是已定 IA，别推翻。**
- **当前前端实现**：`product-line/prototypes/productized-desktop-shell/src/`
  - `App.tsx`：整体壳 + CSS grid（`topbar / rail-left / stage / rail-right / dock`）。
  - `lib/workbenchNavigation.ts`：导航模型（目前是平铺 primary/dev 项）。
  - `views/projects/ProjectWorkspaceShell.tsx`：项目内布局（目前是横向 tab）。
  - `views/RunningWorkflowsView.tsx`、`views/ProjectsView.tsx`、`components/RightDetailPanel.tsx`、`components/WorkbenchNavigation*`。
  - 全局样式 `styles.css`（含水墨 token + 布局）。

## 4. ⚠️ 关键张力（务必先和用户对齐，别盲抄）

**Xuanji 本质是"聊天工具"的布局**（会话侧栏 + 聊天区 + ~50/50 监控区）；**我们工作台是"项目/工作流优先"**（四入口首页 → 项目画布）。所以 **"用 Xuanji 布局" ≠ "把工作台变成聊天 app"**，只能是**借它的结构纪律，不是借它的聊天形态**。

- **可借（结构纪律）**：分组+带文字标签的侧栏、清晰的导航层级、项目内左侧导航。
- **不要盲目照搬（聊天形态，和项目优先冲突）**：50/50 聊天/监控分屏、会话优先、把秘书从底部 dock 挪走。**除非用户明确要往聊天形态走，否则别做这些。**

## 5. 现状 vs Xuanji 差距 + 候选改动（咨询线已过滤；现状为读代码所得，需用户/截图确认）

- **头号痛点（最可能就是用户难受的点）：左侧导航。**
  - 现状：`rail-left` 约 72px **纯图标竖条**，~17 项**平铺、不分组**，**标签只在悬停时出现**。
  - Xuanji：**分组、带文字标签的侧栏**（会话 / 智能体 / 配置 / 运维 / 文件 分块，标签常显）。
  - 改法：把图标条换成**分组+带标签的侧栏**。**纯结构改动，水墨风格一点不动。** 这一条大概率就解决"布局难受"。
- **次要、合理**：项目内现在是**横向 tab**（总览|工作流|会话|任务包|handoff|资源）→ 改成**左侧栏导航**（像 Xuanji 项目内），切换不丢上下文。
- **风险高、且属"聊天形态"**：50/50 分屏、会话优先、秘书挪位——**先放着**，除非用户要。

（上一会话有一份更细的"4 阶段重整方案"草稿,可按需重做;但**别照单全收**——其中含上面"不要盲抄"的那几样。建议你和用户细聊后,自己重新出方案。）

## 6. 风格 vs 布局的分离（改布局别动风格）

- **保留（风格，别碰）**：`styles.css` 里的 `--ink-*` / `--tea` / `--terra` / `--vermil` 等色 token、`.ink-shell` 纸纹/径向渐变、毛笔曲线 SVG、远山装饰、点色/标签状态色、`WorkbenchPrimitives.tsx` 的视觉原语。
- **可改（布局，目标）**：`App.tsx` 的 grid 模板 + rail 宽高结构、`workbenchNavigation` 的导航模型、`ProjectWorkspaceShell` 的 tab→侧栏、各 view 的容器结构。
- 一句话:**动 JSX 结构 + 布局 CSS,别动颜色 token / 装饰 / 风格类。**

## 7. 先和用户细聊要确认的问题

1. **难受的主要就是左侧导航**(图标条/无常显标签/不分组)吗?还是还有别处?
2. **范围**:只做"导航纪律"这一档(低风险、最可能见效),还是要更大幅地向 Xuanji 靠?
3. **聊天形态那几样**(50/50、会话优先、秘书挪位)要不要?——这会动产品形态,需用户拍板。
4. 请用户**截图**现状(导航/首页/项目页)给你看;并问用户那个"打磨 UI 的 MCP 工具"在哪/叫什么,好让你**看着改、不盲改**。
5. **Xuanji 布局 × 我们项目优先 IA 怎么调和**:确认最终骨架(建议:Xuanji 式分组侧栏 + 保留四入口首页 + 项目画布优先)。

## 8. 约束 / 注意

- **保留水墨风格**(§6);**遵守已定 IA**(§3 的 2026-05-28 决定,四入口 + 项目画布优先,别推翻)。
- **看不见 UI → 必须靠截图或预览 MCP 迭代**(§0),别盲改大改。
- **App.tsx 的 grid 是核心、高风险**:建议**小步先行**(先做分组侧栏,看是否见效),别一次大爆改。
- **治理**:任务包 + 独立复核 + 提交前问用户;尺寸闸(新文件 .tsx ≤ 2000 / .rs ≤ 3000),新布局逻辑落**新组件**,别撑爆 `App.tsx` 等大文件。
- **范围归属**:这是一条**独立的 UI 布局小 track**(挨着 Stage L 的 L4 深层 Tauri 验收 + 一批 deferred UI 项),**不影响** Stage L 已收口的 L1/L3/L5,也不动 L5 待兑现的"真写一条真实记忆"。

## 9. 项目背景一句话（给冷启动的你）

产品是本地 AI 工作台(代号 Syn,Tauri 桌面壳,编排本地 Codex)。事实入口 `product-line/CURRENT.md`,权威索引 `product-line/AUTHORITY.md`,协作规则 `product-line/AGENTS.md`(**全程中文、术语标中文注释**是硬规)。当前 Stage L 进行中;记忆层刚拍板"当前形态对单人过度但不返工、改走真用观察"的决策。**UI 布局这条是用户单独拎出来、交给你这个对话细聊+实施的。**

---

*本文为交接上下文,非冻结任务包。请先与用户细聊(§7)再出实施方案、再走治理流程。*
