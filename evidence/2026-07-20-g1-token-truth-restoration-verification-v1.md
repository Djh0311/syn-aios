# G1 · token 归真 验收证据 v1

日期：2026-07-20 · 轻档 · 执行线施工，总指导回收核实物 + 用户最后一眼
任务包：`tasks/2026-07-20-g1-token-truth-restoration-package-v1.md`
基线 commit：`1037819`（开工核对 HEAD=`1037819aa2695e19afac8e6e6ce0815874906b33`，工作树干净）
规则留痕：`decisions/2026-07-20-hardcoded-hex-gate-rule-and-whitelist-v1.md`
截图：`prototypes/productized-desktop-shell/output/playwright/g1-token-truth/{before,after}/`

## 一、结论

五件事全做、四闸全绿、视觉差异全部落在枚举内（含总指导当场补的第 10 项）：

1. 六个 `:root` 坍为**一个正典**（styles.css 顶部，76 token = §2.1 表计数）+ 两个断点尺寸块（≤1180px 仅 `--rail-right:280px`；≥1181px 五行）。14 死 token 定义/引用 grep 双清零；`--ui-*` 全系 11 个退役（定义本体删、引用按 §2.3 映射改）；§2.4 悬空引用 7 处全修（另发现 `var(--text-base)`×3 为表外活引用，按 §五.1「14→--text-md」映射，14→13px 属字号映射枚举）。
2. 桌面皮：死段 12 区退役（含 C 块整段 87 行、D 块 home 系 51 行）；活违宪 6 组治平（卡皮组→paper-card/无影/r-lg、dialog→panel-raised+shadow-lg、boundary→纸面、brand-mark 朱红→石绿、notice-panel→warning 系、agent 半透明白→纸面）；孤儿 6+1 件先迁后删全部在位（`.hide-dev-detail` 实测默认隐藏仍在）。
3. hex：施工前实测 **262 裸值 + 9 转义**（styles.css 190 / sidePanel 49 / TSX 22 / index.html 1；包文「255」与自带分列相加不符，以实测口径为准）→ 施工后 violations **0**、deferred **75 处出现**（白名单 42 条 ≤ 86 预登记，只减不增：AWV 3 条、WCCV 2 条全部治平核销）。
4. 字体：21 裸档→正典 7 档（半像素档灭）、650×9→600、800×8→700、等宽栈 6→var(--font-mono)、幽灵名 5 族 grep 清零、index.html 零字体加载。TSX fontSize 10 处同映射。
5. gate：`hardcoded_hex_on_ui` 上线（本体 lib/，gate 495 行 ≤500）；selftest 13/13；catalog 登记（71 顶层+15 lib=86 条）；decisions 落档。

## 二、token computed 抽样对账表（playwright 实机取值）

| token | 1280 前 → 后 | 577 前 → 后 | 判定 |
|---|---|---|---|
| --bg | #f5f1e8 → **#f3f0ea** | 同 | 定回·枚举① |
| --panel | rgba(245,241,232,0.72) → **#faf8f4** | 同 | 总指导补枚举⑩（见 §七.1） |
| --warning | #b87341 → **#8a4010** | 同 | 定回·枚举② |
| --ink | #1c1f24 → #1c1f24 | 同 | 零差 ✓ |
| --ink-mid | #4a4d54 → #4a4d54 | 同 | 零差 ✓ |
| --muted | #8a8a85 → #8a8a85 | 同 | 零差 ✓ |
| --line | rgba(28,31,36,0.18) → 同 | 同 | 零差 ✓ |
| --danger | #a14242 → #a14242 | 同 | 零差 ✓ |
| --candidate | #546845 → #546845 | 同 | 零差 ✓ |
| --topbar-h | 56px → 56px | 46px → 46px | 零差 ✓ |

## 三、hex 对账

- 施工前（实测）：styles.css 190（87 值）+ 转义 9（2 值）、sidePanel 49（25 值）、TSX 22（AWV 3 + canvasNodeData 16 + PWCV 1 + WCCV 2）、index.html 1。
- 施工动作：a 类等值替换 styles.css 69 处 + sidePanel `#fff` 1 处；回退位删除 styles.css 19 + sidePanel 36 + AWV 1 + WCCV 1；嵌套回退自愈 7 处；index.html #1a1c1a→正典值 #1c1f24。
- 施工后（gate 实测）：violations **0**；deferred 75 = styles.css 47（boot 6 + 转义 9 + 无等值 live 值 32）+ sidePanel 11 + canvasNodeData 16 + PWCV 1。
- 白名单 42 条 ≤ 预登记 86，只减不增；全部登记 decisions §白名单明细。

## 四、桌面皮与孤儿

- 死段删除对照：退役枚举区 ≈235 行 + 块②35 + 块④6 + 块⑤13 + 尾部 34 + 其他 ≈ **styles.css 10596 → 10294（−302）**。勘察 §十二「预计 −2500~-3500」与逐区枚举实际不符（枚举区总量即 ~300 行级；按枚举执行，未删活层）。
- 孤儿 grep 全在位：.secretary-float 1 / .agent-boundary-details 4 / .agent-boundary-summary 1 / .memory-advanced-details 1 / .memory-advanced-summary 1 / .hide-dev-detail 1 / .jiaoban-done-pills 1 / .project-side-dev-detail-toggle 2 / .secretary-dock-trigger 1 / .workflow-compact-item 10。
- **`.hide-dev-detail` 实测**（headless chromium 挂生产 styles.css）：带开关 computed `display:none`、无开关 `display:block`；施工前（worktree 1037819）对照同值。默认隐藏功能零回归。

## 五、字体对账

- 裸字号档 CSS grep 清零（映射台账：9×2/10.5×3/10×31/11×98/11.5×10/12×152/12.5×17/13×47/13.5×6/14×16/15×15/16×10/17×10/18×8/20×3/22×3/24×3/26×4/28×2/30×2/32×1；另 11px/12px/32px 各 1 处在注释行未动）。TSX fontSize 数字清零（10 处映射）。
- 650/800 grep 清零（9+8）；等宽栈 grep 清零（44 行→var(--font-mono)）；幽灵名五族 grep 清零（serif 44 行→var(--font-serif)、DM Sans 2 行→var(--font-sans)；另 3 处 DM Sans 随退役死段删除，与勘察「5 处」对平）。

## 六、四闸

| 闸 | 结果 |
|---|---|
| `npx tsc --noEmit` | exit 0 |
| `node scripts/run-offline-interaction-test.mjs` | exit 0，24 组全绿、零失败行 |
| shape gate baseline | **13/5/5 exit 0 零净增**；hardcoded hex 0 violations（75 deferred）；machine-face 0/0；gate 495 行 ≤500 |
| shape gate check | 13/5/5 同数，exit 1 = 历史债非零（既有口径如实报告） |
| `git diff --check` | exit 0 |
| cargo | 免跑：零 Rust 改动（git diff 无 .rs），包 §十三.1 允许 |
| selftest 加跑 | hardcoded-hex 13/13、machine-face 18/18、dedup 8/8 |

## 七、截图对照（差异仅限枚举项）

`output/playwright/g1-token-truth/` 前后各 5 张（SHA-256 见目录）：

- 交办×1280/×577（生产夹具+生产 CSS）：前后**目测无差**（bg 微差不可见层级）。
- 记忆中心×1280（生产夹具）：白卡浮起→纸面平卡（枚举③）。
- 首页×1280（浏览器预览壳）：brand-mark 朱红→石绿（枚举④）、卡影消失（③）、bg（①）；PD 徽章放大核对无差。
- 智能体页×1280：brand-mark（④）、会话列表/转录区半透明白→纸面（③）、notice-panel 朱红 dashed→warning 系（⑤）。

## 八、枚举外事项与被闸拦过的事

1. **§九.7 第 10 项枚举（总指导 07-20 当场拍板补入）**：`--panel`/`--panel-soft` 正典表值 ≠ live 级联终值（rgba(245,241,232,0.72)/rgba(28,31,36,0.035)），总指导拍「照表做，面板略变白」，补为枚举⑩，已进截图对照。
2. **PWCV:994 处置冲突披露**：§八列「替换」、§六.3 又给 PWCV 白名单 1 条；#9aa0a6 无等值 token，替换=调色（禁），取白名单侧（零位移），白名单使用 1/1。
3. **包文计数差**：「255 处」与自带分列（190+49+22+1+9=271/裸值 262）不符；「10 函数」式笔误同型。以实测口径对账（§三）。
4. **勘察行数估计差**：死段「预计 −2500~-3500」实为 −302（枚举区总量 ~300 行级），按枚举执行。
5. **var(--text-base)×3**：§2.1/§2.2 未列的活引用，按 §五.1 映射 var(--text-md)（14→13 属字号映射枚举），否则正典落地即悬空。
6. 施工自愈：值映射先于回退删除造成 7 处 `var(--x, var(--y))` 嵌套，当场压平；sidePanel 3 处**既有** `var(--accent-strong, var(--accent))` 误压后按 HEAD 逐字复原（引用面一行不动红线）。

## 九、红线自查

零类名/选择器改名、零 TSX 结构（仅值替换，TSX 净变 <30 行）、零 JS/TS 逻辑、零 Rust；离线断言文件逐字未动（四闸绿）；白名单只减不增（86→42）；孤儿先迁后删；未 stage、未 commit；视觉差异全部落枚举（§七对照）。start = end = `1037819aa2695e19afac8e6e6ce0815874906b33`。

## 总指导补记（2026-07-20）

机器面核收全绿（tsc 0 / 离线 24 组 / shape 13-5-5 零净增 / hex 0 violations / selftest 13-18-8 / 截图 5 对逐张核）后，**用户最后一眼**对枚举项「brand-mark 朱红→石绿」提出异议：印章文化上即朱砂红。用户拍板**红章+绿动作**：`.brand-mark` 保留 `var(--vermil)` 作品牌专属例外——桌面皮两处规则（styles.css:9643/:9811 区）由 accent 回修为 vermil，投影色同步回 `rgba(161,66,66,0.4)`/`rgba(159,63,52,0.16)`；批准章/按钮/选中态仍石绿。总指导亲验：渲染夹具截图红章绿钮并立（/tmp 夹具+headless chromium，1280 触发 ≥1181px 媒体面）。决策已补拍板条（`decisions/2026-07-20-jiaoban-design-direction-three-column-truth-restoration-v1.md` §补充拍板）。本补记后枚举项⑨「brand-mark 朱红→石绿」**作废**，实际视觉变化=9 项（含 --bg/--warning 定回、--panel 微亮⑩）。
