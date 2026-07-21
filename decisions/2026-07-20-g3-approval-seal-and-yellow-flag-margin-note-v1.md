# 决策：G3 盖章时刻——批准石绿印章 + 交货卡黄牌朱砂页边批注 v1

日期：2026-07-20
任务包：`tasks/2026-07-20-g3-approval-seal-moment-package-v1.md`（G3 盖章时刻·轻档）
上位：`decisions/2026-07-20-jiaoban-design-direction-three-column-truth-restoration-v1.md`（G3 行 + §随之拍定「盖章=批的签名时刻」+ §补充拍板红章绿动作）
式样正本：`prototypes/design-mockups/jiaoban-redesign-specimen-v1.html` §签名段（行 319-354）

## 拍板固化

1. **章=批的签名时刻（全 App 唯一重彩）**：仅 `proposal.status === "user_confirmed"` 出章，位置方案卡右上（absolute top:14px right:16px），式样照样张逐值——76×76 圆、2.5px `var(--accent)` 边 + 内 4px 1px 环（`::before` inset:4px）、`rotate(-12deg)`、`mix-blend-mode:multiply`、静态 opacity .92、`<b>已批准</b>`（var(--font-serif) 16px/600/letter-spacing .18em）+ `<s>SYN · MM-DD</s>`（8.5px·text-decoration:none）。**批准章=石绿 accent，朱砂不进章**。日期真源=`proposal.updated_at_ms`（批准态下=授权落账时刻，本地时补零 MM-DD；不许 created_at_ms 替代）。
2. **动效只放一次**：刚批首现 `.is-fresh` 挂 `stamp .3s cubic-bezier(.23,1,.32,1)`（0% scale 1.7→62% .96→100% 1）+ 卡 `thud .3s` 同曲线；重进/刷新/历史=静态。`@media (prefers-reduced-motion: reduce)` 两动画全禁静态显（不许砍）。fresh 判定=组件内探测（本会话 status 翻 user_confirmed 那一下，0.5s 后落静态）+ 可选 `sealFresh` prop 直控（离线断言/调用方）。
3. **黄牌分级语义**：**危险=有形章**（tone-red 仍 `spec-pill bad`）、**黄牌=无形批注**——交货卡 tone-yellow 三处（步条 ⚠ 行/闸条 ⚠ 行/概览「⚠ N 项要看一眼」）不走 Pill，改 `<span class="jiaoban-flag-note">`：无边无底、`color:var(--danger)`（朱砂=vermil 别名 token）、`font-size:var(--text-xs)`、位置不变。green/gray/「未交货」头 pill/行 tone chrome（`jiaoban-step-row tone-yellow` 赭石左条）/非交货卡各面 warn pill 全不动；朱砂不扩面。
4. **PillRow 补可选 `ariaLabel?: string`**，交货卡概览行带回「这单概览」逐字（G2 挂账清）。
5. 标题区右 padding 92px 留章位（本包唯一布局微调）。

## 落点

- `projectWorkflowSidePanel.css`：`.jiaoban-authorize` 补 `position:relative`；`.jiaoban-plan-title` 补 `padding-right:92px`；新增 `.jiaoban-seal` 系（含 `::before`/`.in b/s`/`.is-fresh`/`@keyframes stamp/thud`/reduced-motion 块）与 `.jiaoban-flag-note`。零 hex 裸值。
- `JiaobanAuthorizeStates.tsx`：章节点挂载 + fresh 探测 + `sealDateText`（MM-DD 局部格式化）。
- `JiaobanDoneStates.tsx`：yellow 三处改批注；`stepBadgePillTone` 出 yellow（留 green/red/gray）。
- `SpecPrimitives.tsx`：PillRow 加可选 ariaLabel（透传 aria-label；不传=无属性）。
- `ProjectJiaobanPanel.tsx`：**零改动**（fresh 机制组件内闭环，不需面板传参）。
- 测试新册 `tests/jiaoban-approval-seal-and-flag-note.test.tsx` + run-offline 注册一行（既有 24 组逐字不动）。

## 纪律

不修宪、零状态机/后端/逻辑改、零文案改（章面「已批准」「SYN · MM-DD」与既有 ⚠ 人话除外）；章只出方案卡（其它面不许仿章）；动画时序/曲线不许调（样张值）；离线断言只加不改。
