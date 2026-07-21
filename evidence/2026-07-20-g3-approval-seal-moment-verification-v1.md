# G3 · 盖章时刻 验收证据 v1

日期：2026-07-20 · 轻档 · 执行线施工，总指导回收核实物 + **用户最后一眼（硬性验收）**
任务包：`tasks/2026-07-20-g3-approval-seal-moment-package-v1.md`
基线 commit：`8213678`（开工核对 HEAD=`821367810f2554deb67342f9c06b11ba25e083e1`，工作树干净=G2 已收口）
拍板留痕：`decisions/2026-07-20-g3-approval-seal-and-yellow-flag-margin-note-v1.md`
截图：`prototypes/productized-desktop-shell/output/playwright/g3-approval-seal/{before,after}/`（before 6 + after 11·SHA-256 见目录 SHA256SUMS.txt）

## 一、结论

章+批注+aria-label+DOM 断言新册全做，四闸全绿，视觉差异全部落在 §九 4 项枚举内：

1. **批准落石绿圆章**：`JiaobanAuthorizeStates.tsx` 章节点挂载——仅 `user_confirmed` 出章（pending/changes_requested 断言锁无章），章面「已批准」+「SYN · MM-DD」（`updated_at_ms` 本地补零·断言与真源对平）；式样照样张 §签名段逐值（§三对照表）。fresh=刚批首现 stamp+thud（组件内探测 status 翻 confirmed 那一下 + 可选 `sealFresh` prop 直控），historical 静态。
2. **交货卡 tone-yellow 三处改朱砂页边批注**：步条 ⚠ 行/闸条 ⚠ 行/概览「⚠ N 项要看一眼」→ `<span class="jiaoban-flag-note">`（无边无底·var(--danger)·text-xs·原位）；`stepBadgePillTone` 出 yellow（留 green/red/gray）；tone-red 仍 spec-pill bad、green/gray/「未交货」头 pill/行 tone chrome 全不动。
3. **PillRow 补可选 `ariaLabel`**，概览行带回「这单概览」逐字（G2 挂账清；不传=无属性，既有调用零影响）。
4. **DOM 断言新册** `tests/jiaoban-approval-seal-and-flag-note.test.tsx`（8 组断言）+ run-offline 注册一行；既有 24 组逐字不动。

## 二、四闸

| 闸 | 结果 |
|---|---|
| `npx tsc --noEmit` | 0 错 |
| `node scripts/run-offline-interaction-test.mjs` | **25 组全绿**（24 不回归 + 新册过，exit 0） |
| shape gate baseline | 13/5/5；retired_style_family 0 violations / 2 deferred（新类名非退休族·闸不误伤实证） |
| shape gate check | 13/5/5 零净增 |
| `git diff --check` | exit 0 |
| cargo | 免跑：零 Rust 改动（git diff 无 .rs），包 §十一.1 允许 |

## 三、章 CSS 逐值对样张 §签名段（行 319-354）

| 值 | 样张 `.seal` | 落地 `.jiaoban-seal` | 判定 |
|---|---|---|---|
| 定位/尺寸 | absolute top:14px right:16px·76×76 | 同 | ✓ |
| 边/环 | 2.5px var(--accent)·radius 50%；`::before` inset:4px 1px 环 | 同 | ✓ |
| 角度 | rotate(-12deg) | 同 | ✓ |
| 混合 | mix-blend-mode:multiply | 同 | ✓ |
| 静态 opacity | 基 0→stamped/reduced-motion .92 | **基 .92**（章只在批后渲染·等价落地，见 §八.1） | ✓（口径注记） |
| stamp | .3s cubic-bezier(.23,1,.32,1)·0% 1.7→62% .96→100% 1（opacity 0→.94→.92） | 同（挂 `.is-fresh`） | ✓ |
| thud | .3s 同曲线·0/40/100 scale(1/.996/1) | 同（挂 `.jiaoban-authorize.is-fresh`） | ✓ |
| reduced-motion | 两动画全禁·静态 .92 | 同 | ✓ |
| 章文 b | serif 16px/600/.18em/block | var(--font-serif)（token 名差·样张 --serif=正典 --font-serif） | ✓ |
| 章文 s | 8.5px/.08em/none/.85/block/2px | 同 | ✓ |
| pointer-events | none | 同 | ✓ |
| 标题留章位 | —（本包拍板） | `.jiaoban-plan-title` padding-right:92px | ✓ |

零 hex 裸值证明：新增两段 CSS（`.jiaoban-seal` 系+`.jiaoban-flag-note`）`grep -E '#[0-9a-fA-F]{3,8}'` = 0 命中（全 token：--accent/--font-serif/--danger/--text-xs）。

## 四、DOM 断言（§三.6·新册 8 组全绿）

pending→无章·无「已批准」；user_confirmed→有章含「已批准」+「SYN · 」+MM-DD 与 updated_at_ms 对平；fresh 态章+卡双 is-fresh、historical 零 is-fresh；changes_requested 抽样无章；步条 yellow→jiaoban-flag-note 且零 spec-pill-warn、red 仍 spec-pill-bad、green 仍 spec-pill-ok；全绿链零 flag-note；闸条「⚠ 要改」批注式+「✗ 卡住」spec-pill-bad；概览行批注式+`aria-label="这单概览"` 逐字+「完成 2 步」「已交货」不变。

## 五、截图对照（差异仅限 §九 4 项）

| 面 | 前后 | 差异（枚举项） |
|---|---|---|
| 方案卡 pending ×1280/×577（生产夹具） | 异 | 仅枚举②：标题右 padding 92px（长标题折行点右移）；无章 ✓（断言锁） |
| 方案卡 confirmed ×1280/×577（/tmp 夹具·historical） | 异 | 枚举①：右上石绿圆章静态常驻（已批准+SYN · 07-20·-12°·multiply 落纸）；章与标题不撞（zoom 图在档） |
| 交货卡带黄牌 ×1280/×577 | 异 | 枚举③：步条 ⚠/概览 ⚠1 项 赭石 pill→朱砂无边批注小字；绿/灰 pill 与行 tone chrome 不变 |
| fresh 动效 | frame1/2/settled + 数值采样 | 枚举①：Web Animations API 定帧 t=60/t=190ms（章大淡→落）+ computed style 采样序列（t=0 opacity 0 scale 1.66 → t=17.8/51/84 插值 → finished）证明 stamp 曲线照样张；reduced-motion 图=静态显零动画（fresh 态 120ms 即全形） |
| 样张参照 | specimen-seal-reference.png | 式样正本同框对照 |

辅助图：`authorize-seal-zoom.png`（章区 2x 放大：章文清晰·底层路径文字 multiply 下仍可读）。

## 六、五态走查实证

说=零变化（pending 两图仅 padding 枚举项）；批=章落纸（fresh 采样+帧序列）；干/交货=静态常驻口径=historical 默认（组件内探测只在 status 当翻面 fresh 一次）；卡住=zero-touch（JiaobanBlockedStates 未碰·tone-red 断言锁）。

## 七、形状影响与红线自查

新增落点：css 两段 + tsx 3 处小改 + 测试新册 1 + 注册 1 行；组件零新增；`styles.css` 零碰；`workbench-shape-gate.js` 零碰；无新 Tauri command、无新 sidecar JSON；未 stage、未 commit；start = end = `821367810f2554deb67342f9c06b11ba25e083e1`。`ProjectJiaobanPanel.tsx` 零改动（fresh 组件内闭环）。

## 八、枚举外事项与被闸拦过的事

1. **静态 opacity 口径**：样张基值 0 + `.stamped` 动画/reduced-motion 补 .92；落地=章仅批后渲染故基值 .92、is-fresh 播 stamp（0→.92）——视觉效果等价，reduced-motion 块同形保留，特注记。
2. **章与首行事实值轻叠**：章位 top:14 right:16 与右对齐首行值同区（样张同构·multiply 设计件）；zoom 图实证章文与底层文字双可读，未扩 padding（§二.3 只枚举标题区）。若用户最后一眼嫌挤，回总指导另拍。
3. **夹具口径**：confirmed 卡夹具 readOnly=false（按钮区可见）——生产各相位按钮区随 readOnly 收放，章显隐只随 status，不受此影响；fresh 帧用 Web Animations API 定帧（0.3s 窗口截图追不上·数值采样为主证）。
4. 被闸拦过的事：**无**（四闸直过；新类名 `.jiaoban-seal`/`.jiaoban-flag-note` 非退休族，retired_style_family 零误报）。
