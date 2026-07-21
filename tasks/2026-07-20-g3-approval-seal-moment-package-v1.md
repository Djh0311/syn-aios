# 任务包：G3 · 盖章时刻——批准落石绿印章 + 交货卡黄牌朱砂批注 v1

日期：2026-07-20
状态：**已出包，待总指导派工**
档位：**轻档**（纯前端呈现层，不碰高危清单 5 条；不修宪、零布局变更、零逻辑变化）
执行者：执行线；总指导回收核实物 + **用户最后一眼（感受件，本包硬性验收）**
所属开发线：桌面应用线 / 视觉治理线 G1✓→G2✓→**G3**→G4
上位决策：`decisions/2026-07-20-jiaoban-design-direction-three-column-truth-restoration-v1.md`（G3 行 + §随之拍定「盖章=批的签名时刻」+ §补充拍板红章绿动作）
式样正本：`prototypes/design-mockups/jiaoban-redesign-specimen-v1.html` §签名段（`.seal`/`.stamped`/`@keyframes stamp/thud`/reduced-motion 块，行 319-354）
用户追加拍板（2026-07-20 本代会话）：**「黄牌改朱砂页边批注式样」=只改交货卡步条**——⚠步条/闸条/概览「N 项要看一眼」从赭石 pill 改朱砂色页边批注（无边无底小字），其余面不动
勘察依据：总指导 2026-07-20 G3 写前勘察（G2 后行号重核；数字自带分列，禁用「等」字）
本任务基线 commit：`8213678`

## 一句话目标

批准动作落印章签名：方案卡批后右上盖石绿圆章（样张逐值：76px·-12°·multiply·按压动效·批后常驻·reduced-motion 静态退化），全 App 唯一重彩时刻；交货卡 tone-yellow 三处改朱砂页边批注；PillRow 补回 aria-label（G2 挂账清）——**功能零变化、批准逻辑/状态机/后端零碰、视觉变化仅限 §九枚举**。

## 一、勘察实录（全部实测）

1. **方案卡**=`JiaobanAuthorizeStates.tsx`（473 行）渲染 `.jiaoban-authorize`（`project-canvas-detail-card`，css `projectWorkflowSidePanel.css:1531`），经 `ProjectJiaobanPanel.tsx:1597-1602` `proposalViewContent` 进 `buildJiaobanArtifactCanvasViews`，authorize/running/done/history 各相位方案视图常驻（readOnly 区分）。
2. **批准态判据（真事实）**：`proposal.status === "user_confirmed"`（枚举 `workflow.ts:799-805`：draft/pending_user_confirmation/user_confirmed/changes_requested/rejected/superseded）；日期真源=`proposal.updated_at_ms`（number，批准态下=授权落账时刻）。既有时间工具 `jiaobanTime.ts`（`formatProposalTime`=YYYY-MM-DD HH:MM 过长，章面需 MM-DD 紧凑式，本包新增局部格式化）。
3. **交货卡 tone-yellow 现状**（G2 后）：`JiaobanDoneStates.tsx:20` 查表 `stepBadgePillTone={green:ok,yellow:warn,red:bad,gray:plain}`；yellow 三处出口=①闸条行（:71，word=「⚠ 要改」「⚠ 缺证据」）②步条行（:228，flag.tone==="yellow" 时 ⚠ 前缀+flag.badge）③概览行（:427，`<Pill tone="warn">⚠ N 项要看一眼</Pill>`，PillRow 无 aria-label=G2 挂账）。行 tone 类 `jiaoban-step-row tone-yellow`（赭石左条+浅底）是行 chrome，**不动**。交货卡头「未交货」warn pill（:421）非黄牌项，**不动**。
4. **测试基建**：离线断言=tests/*.test.tsx 注册进 `scripts/run-offline-interaction-test.mjs` 列表（现 24 组）；渲染先例 `report-fact-confirm-recall.test.tsx`（renderToStaticMarkup+字符串断言）。
5. **章/批注 CSS 落点**：`projectWorkflowSidePanel.css` `.jiaoban-authorize` 区（:1531 起）旁新增两段；新类名 `.jiaoban-seal`/`.jiaoban-flag-note`（不在 `retired_style_family` 退休族内，闸不误伤）。

## 二、核心拍板口径（不许自由发挥）

1. **章式样=样张逐值照抄**（行 319-354）：76×76 圆、2.5px `var(--accent)` 边+内 4px 1px 环、`rotate(-12deg)`、`mix-blend-mode:multiply`、静态 opacity .92、`stamp .3s cubic-bezier(.23,1,.32,1)`（0% scale1.7→62% .96→100% 1）+卡 `thud .3s` 同曲线、`@media (prefers-reduced-motion: reduce)` 下两动画全禁静态显。色=石绿 accent，**朱砂不许进章**（补充拍板：批准章=石绿）。
2. **章内容**：上行 `<b>已批准</b>`（serif 16px·letter-spacing .18em）、下行 `<s>SYN · MM-DD</s>`（8.5px；MM-DD 取 `proposal.updated_at_ms` 本地格式化补零，零假数——批准态下该字段=授权落账时刻；不许用 created_at_ms 替代）。
3. **出章条件**：仅 `proposal.status === "user_confirmed"`；pending/其它五态无章。位置=方案卡内 absolute top:14px right:16px（样张值）；卡 `position:relative` 补齐；**标题区右侧留章位**（`.jiaoban-plan-title` padding-right ≥92px，防字章相撞——这是本包唯一布局微调，已枚举）。
4. **动效只放一次**：本会话刚批（authorize→binding/running 首现）放 stamp+thud；重进/刷新/历史查看=静态（无 animation 类）。实现机制执行线定（上一相位 ref 或面板传 justApproved），**行为验收**：DOM 断言 fresh 态带动效类、historical 态静态类。
5. **黄牌批注=只改交货卡**：tone-yellow 三处（§一.3①②③）不走 Pill，改 `<span className="jiaoban-flag-note">⚠ …</span>`：无边无底、`color:var(--danger)`（朱砂=vermil 别名 token）、`font-size:var(--text-xs)`、位置不变（行右/概览行内原序）。**分级语义**：危险=有形章（tone-red 仍 `spec-pill bad`）、黄牌=无形批注——green/gray/「未交货」头 pill/行 tone chrome/标题联动文案/非交货卡各面 warn pill 全不动。
6. **PillRow 补可选 `aria-label`**，交货卡概览行带回「这单概览」（G2 挂账清，逐字）。
7. 零文案改（章面「已批准」「SYN · MM-DD」与 ⚠ 既有人话除外，逐字照本文）；离线断言只加不改（24 组逐字不动，新组另册）。

## 三、施工清单（逐文件枚举）

1. `projectWorkflowSidePanel.css`（`.jiaoban-authorize` 区旁）：新增 `.jiaoban-seal` 系（照样张 §签名段逐值，含 `::before` 内环、`.in b/s`、`.is-fresh` 动效类挂 stamp/thud、`prefers-reduced-motion` 退化块）+`.jiaoban-flag-note`；`.jiaoban-authorize` 补 `position:relative`；`.jiaoban-plan-title` 补 `padding-right:92px`。零 hex 裸值（全 token）。
2. `JiaobanAuthorizeStates.tsx`：章节点挂载（§二.2/.3 口径）；fresh 判定按 §二.4。
3. `ProjectJiaobanPanel.tsx`：仅当 fresh 机制需要面板传参时动（最小 diff）。
4. `JiaobanDoneStates.tsx`：tone-yellow 三处改批注（§二.5）；`stepBadgePillTone` 查表 yellow 出表（留 green/red/gray）；概览行 PillRow 加 `aria-label="这单概览"`。
5. `SpecPrimitives.tsx`：PillRow 加可选 `ariaLabel?: string`（透传到底层 div aria-label；不传=无属性，既有 1 处调用零影响）。
6. `tests/jiaoban-approval-seal-and-flag-note.test.tsx`（新建）+`scripts/run-offline-interaction-test.mjs` 注册一行。断言组：
   - 方案卡 pending_user_confirmation→无章；user_confirmed→有章含「已批准」与「SYN · 」+MM-DD 与 updated_at_ms 对平；fresh 态带 is-fresh（或同等动效类）、historical 态不带；
   - 交货卡带黄牌链：步条 ⚠ 行含 `.jiaoban-flag-note` 且不含 `spec-pill-warn`；闸条「⚠ 要改」同行批注式；概览行「⚠ N 项要看一眼」批注式且 PillRow 带 `aria-label="这单概览"`；tone-red 行仍 `spec-pill-bad`；全绿链零 flag-note。
7. `evidence/2026-07-20-g3-approval-seal-moment-verification-v1.md`（新建）+decisions 落档（`decisions/2026-07-20-g3-approval-seal-and-yellow-flag-margin-note-v1.md`：黄牌批注分级语义+章式样拍板固化）。

## 四、允许读取

本包、上位决策、样张、DESIGN.md、G1/G2 包与证据、`prototypes/productized-desktop-shell/`（src/**、tests/**、scripts/**）。

## 五、允许写入

§三列出的 7 处（含 2 新建）+`CURRENT.md`（收口后总指导笔）。**不许碰**：`styles.css`、其它 css、其它 tsx/ts、Rust、run-offline 既有 24 组测试文件本体。

## 六、禁止事项

1. 不修宪、零布局变更（§二.3 章位 padding 除外）、零逻辑/状态机/后端改动、零文案改（§二.7 除外）。
2. 章只出方案卡——全 App 唯一重彩时刻，其它面不许仿章；朱砂只进 `.jiaoban-flag-note` 与既有 danger token 面，不扩。
3. 离线既有 24 组逐字不动；断言挂=修施工不修断言。
4. 不 stage、不 commit。
5. 视觉变化不得超出 §九 4 项；发现第 5 项先停回总指导。
6. reduced-motion 退化不许砍；动画时序/曲线不许调（样张值）。

## 七、变更辐射面

- 方案卡各相位（authorize/running/done/history）→ 章显隐由 status 判，pending 各面零变化（断言锁）。
- 交货卡全谱（全绿/带黄牌/未交货/只读单）→ 只 yellow 三处变形；red/green/gray 断言锁不变。
- PillRow 1 处调用（交货卡概览行）→ aria-label 回；SpecPrimitives API 加可选 prop 不破坏既有。
- 闸面：新类名非退休族；零 hex；shape 13/5/5 零净增。

## 八、五态旅程走查

- 说：零变化。
- 批：**主项**——点批准→章落纸（fresh 动效），方案卡常驻签名；reduced-motion 静态同形。
- 干：running 相位方案视图章常驻静态。
- 交货：done 相位方案视图章常驻；交货卡黄牌批注式（§九.3）。
- 卡住：zero-touch（卡住脸不动）。

## 九、视觉变化枚举（进截图对照，共 4 项）

1. 方案卡批后右上石绿圆章（76px·-12°·multiply·已批准+SYN·日期）；刚批首现 0.3s 按压+卡 thud；reduced-motion 静态。
2. 方案卡标题区右 padding ≥92px 留章位。
3. 交货卡 tone-yellow 三处（步条 ⚠/闸条 ⚠/概览 ⚠N 项）：赭石 pill→朱砂无边批注小字。
4. 交货卡概览行恢复 aria-label（无视觉）。

## 十、形状影响

- 任务类型：**治理任务包**（拍板式样落地+挂账清 1 件）。
- 新增代码落点：css 两段+tsx 3 处小改+1 测试新册；组件零新增。
- 棘轮文件：`styles.css` 零碰；`workbench-shape-gate.js` 零碰。
- 新增 Tauri command：无。新增 sidecar JSON：无。
- 本任务基线 commit：`8213678`。完成 commit：总指导核收+用户最后一眼后另定（执行线不 commit）。

## 十一、验收标准

1. 四闸：`npx tsc --noEmit`=0；`node scripts/run-offline-interaction-test.mjs` 全绿（24 组不回归+新断言组过）；shape gate baseline+check **13/5/5 零净增**；`git diff --check` 过。Rust 零碰，cargo 免跑（理由明写）。
2. 式样对账：章 CSS 逐值对样张 §签名段（evidence 给对照表：尺寸/角度/blend/时序/reduced-motion 逐项）；零 hex 裸值 grep 证明。
3. DOM 断言 §三.6 全绿。
4. **实渲量尺+截图对照（感受件）**：方案卡批前/批后（historical 静态）×1280+×577、交货卡带黄牌×1280+×577 前后各一套；fresh 动效用 3 帧序列或动静各一证明（playwright 二进制直截法）；差异仅限 §九 4 项。
5. **用户最后一眼**：批后方案卡+交货卡黄牌两组图过用户眼才算收。

## 十二、必须回传（按 TASK_TEMPLATE 10 项）

做了什么 / 改了哪些文件 / 新增哪些测试或证据 / 哪些结论有依据 / 哪些仍不确定 / 风险和下一步建议 / shape gate baseline+check 摘要 / start-end commit / 是否新增 command·sidecar·触碰棘轮文件 / **被闸拦过的事**（无也必须写「无」）。

## 十三、总指导回收动作

- 亲跑四闸不信回传；章 CSS 逐值对样张；DOM 断言逐条核；截图逐张看形（重点：章与标题不撞、multiply 落纸感、批注朱砂不抢戏、577 两态）。
- 用户最后一眼后判断 接受 / 需要修改 / 暂停 / 废弃，记 `docs/harness-catch-log.md`；收口 commit 同笔回写 CURRENT（G3 收口+G4 接续）。
