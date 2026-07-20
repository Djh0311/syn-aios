# 任务包：G1 · token 归真——单 :root 正典 + 桌面皮治平 + hex 归 token + 字体收敛 + gate 禁新硬编码 v1

日期：2026-07-20
状态：**已出包，待总指导派工**
档位：**轻档**（纯 CSS/呈现层治理，不碰高危清单 5 条）
执行者：执行线；总指导回收核实物 + **用户最后一眼（截图对照，感受件纪律）**
所属开发线：桌面应用线 / 视觉治理线 G1→G4
上位决策：`decisions/2026-07-20-jiaoban-design-direction-three-column-truth-restoration-v1.md`
施工参照：`prototypes/design-mockups/jiaoban-redesign-specimen-v1.html`
法源：`prototypes/productized-desktop-shell/DESIGN.md` §一（零换皮拍板值）、交互宪法 §四、七律
勘察依据：总指导 2026-07-20 G1 写前勘察（逐文件枚举；本文表全部自带行号，**禁用「等」字**）
本任务基线 commit：`1037819`

## 一句话目标

把 6 个 `:root`、83 token、四个命名时代坍成**一个正典 `:root`**（`--bg` 定回拍板值 `#f3f0ea`，中性色取 live 值，强调色取样张/拍板值），桌面皮违规部分（白卡/朱红）治平、死壳退役、孤儿规则先迁后删，255 处硬编码 hex 全量归 token 或登记，字体三族收敛，shape gate 加 `hardcoded_hex_on_ui` 机械规则——**功能零变化，视觉变化仅限本包枚举项**。

## 一、核心拍板口径（不许自由发挥）

1. **中性色 = live 值入正典**（拍板「沿用现在 App 风格原样」，文本墨色/纸面层次不动）。
2. **`--bg` 定回 `#f3f0ea`**（DESIGN.md 拍板值；当前 live 被覆盖成 `#f5f1e8`，这是归真主项）。
3. **强调色 = 样张/拍板值**：`--accent:#2a6b5e` 不动；`--danger = --vermil #a14242`（拍板保留，朱砂只做黄牌/危险）；`--warning` 定回 `#8a4010`（样张/一代拍板值，live 被覆盖成 terra `#b87341`——这是有意的归真变化，进截图对照）。
4. **改名零容忍，alias 桥接**：所有引用面（`var(--x)` 共 69 个活 token、最大 125 处）**一行不动**；时代桥接用别名（如 `--ink: var(--ink-deep)`、`--rice: var(--bg)`、`--muted: var(--ink-light)`、`--line: var(--hair-2)`）。
5. **零类名/选择器改名、零 TSX 结构变化**（仅内联 style 的 hex 值替换）；离线测试断言逐字不动。

## 二、正典 token 表（交付合同）

新正典 `:root` 放 `styles.css` 顶部（原块①位置），其余 5 处 `:root`（:3770/:5257/:7525/:9587/:9803）**整段删除**，断点尺寸块只保留本表「断点」行。

### 2.1 保留并入正典（值=live 赢家，除注明）

纸墨中性：`--bg:#f3f0ea`【定回】 `--bg-subtle:#ede9e1` `--panel:#faf8f4` `--panel-soft:#f0ede6` `--panel-raised:#ffffff` `--paper-card:#fbfcf8`【新增，收 24 处裸值】 `--rice:var(--bg)`【alias】 `--rice-2:#efeadd` `--shell:#ebe5d4` `--ink-deep:#1c1f24` `--ink:var(--ink-deep)`【alias】 `--ink-mid:#4a4d54` `--ink-light:#8a8a85` `--muted:var(--ink-light)`【alias】 `--ink-mist:#b6b3a8` `--hair:rgba(28,31,36,0.1)` `--hair-2:rgba(28,31,36,0.18)` `--line:var(--hair-2)`【alias】

强调/语义：`--accent:#2a6b5e` `--accent-strong:#1b4f45` `--accent-bg:#eaf4f1` `--vermil:#a14242` `--danger:var(--vermil)` `--danger-bg:#fdf0f0` `--danger-border:rgba(162,66,66,0.25)`【值随 vermil 微调，进对照】 `--warning:#8a4010`【定回】 `--warning-bg:#fff0e0` `--warning-border:rgba(138,64,16,0.25)` `--run:#1a5c8a` `--run-bg:#e7f2fa` `--candidate:#546845` `--candidate-bg:rgba(110,127,91,0.1)` `--unknown:#5a5068` `--unknown-bg:#f0eef6` `--ok:var(--accent)` `--ok-bg:var(--accent-bg)` `--tea:#6e7f5b` `--terra:#b87341`

画布色族（新增，收 §3.1 b 类裸值，live 值不动）：`--canvas-green:#5a6f4a` `--canvas-green-bg:#eef2e8` `--canvas-line:#c8c2b4` `--canvas-ink:#2e2a25` `--canvas-orange:#c8602b` `--canvas-orange-bg:#fff3e6` `--canvas-red:#b1493f` `--canvas-red-bg:#fbeceb` `--canvas-gold:#c8993f` `--canvas-gold-bg:#faf3e2` `--canvas-brown:#8a5a12`

尺寸/断点：`--space-1:4px` `--space-2:8px` `--space-3:12px` `--space-4:16px` `--space-5:20px` `--space-6:24px` `--r-sm:6px` `--r-md:8px` `--r-lg:12px` `--shadow:0 1px 2px rgba(26,28,26,0.06)`【从 none 改为样张轻落地，进对照】 `--shadow-lg:0 12px 40px rgba(26,28,26,0.10)` `--rail-left:48px` `--rail-right:58px`（≤1180px 280px / ≥1181px 58px，断点块保留） `--rail-right-expanded:320px`（≥1181px 384px，断点块保留） `--topbar-h:46px`（≥1181px 56px） `--dock-h:64px` `--sidebar-w:144px`

字号正典（§四映射的靶）：`--text-2xs:10px` `--text-xs:11px` `--text-sm:12px`【改值 13→12，用量最大者入档】 `--text-md:13px`【改值 15→13】 `--text-lg:15px`【改值 18→15】 `--text-xl:17px`【改值 22→17】 `--text-2xl:22px`【改值 28→22】

字体族正典（三族，幽灵名全删）：`--font-serif:"Songti SC","STSong",serif` `--font-sans:-apple-system,"PingFang SC","Helvetica Neue",sans-serif` `--font-mono:"SF Mono",ui-monospace,Menlo,Consolas,monospace`

### 2.2 删除（死 token，零引用，勘察已核）

`--planned` `--planned-bg` `--right-panel-w` `--right-rail-w` `--shadow-sm` `--shell-2` `--space-8` `--status-err` `--status-ok` `--status-run` `--status-warn` `--text-lg`（旧 18px 档） `--text-md`（旧 15px 档） `--ui-rail-strong`

### 2.3 --ui-* 系处置（11 个）

桌面皮治平后 `--ui-*` 系**全系退役**：`--ui-bg/--ui-rail/--ui-surface/--ui-surface-soft` 的引用处改纸面 token；`--ui-accent` 删除（其 3 处引用已被 B/D 块覆盖，勘察 §2.1）；`--ui-text→--ink-deep`、`--ui-muted→--ink-light`、`--ui-line→--hair`、`--ui-line-strong→--hair-2`、`--ui-shadow→--shadow-lg`。注意暗雷：`styles.css:1395` `.session-resize-handle::before` 在基座层引用 `--ui-line`（<1181px 已失效），改为 `var(--hair)`；`:10435` 的 `var(--ui-muted,#6b6b6b)` 改 `var(--ink-light)`。

### 2.4 悬空引用 7 处（用了没定义，本包一并归真）

`styles.css:2872 --text`（失效声明，改 `var(--ink-deep)`）；`sourceStylePlaceholder.css:29/:106 --ink-soft`（改 `var(--ink-light)`）；`sidePanel.css:626 --hair-1`（改 `var(--hair)`，删回退）；`sidePanel.css:1050 --text-muted`（改 `var(--ink-light)`，删回退）；`ActiveWorkbenchView.tsx:543 --border-subtle`（改 `var(--hair)`）；`ActiveWorkbenchView.tsx:621 --success`（改 `var(--accent)`）；`WorkflowCommandConsoleView.tsx:228 --border`（改 `var(--hair)`）。

## 三、桌面皮（styles.css:9585-10455）处置

**不删层，治平+退役+迁孤儿**，严格按勘察行号：

1. **退役（死选择器，TSX 零引用，勘察 §2.1 已逐一核）**：9601-9604、9616-9620、9622-9625、9634-9636、9644-9648、9650-9657、9659-9663、9665-9687、9689-9693、9695-9702、9704-9730、C 块整段 10157-10243、D 块 home 系 10273-10323。
2. **治平（活但违宪）**：9732-9745 卡皮组（白底 `rgba(255,253,248,0.82)`+`--ui-shadow`+16px→`background:var(--paper-card)`+`box-shadow:none`+`var(--r-lg)`）；9782-9795 dialog 白面（→`var(--panel-raised)`+`var(--shadow-lg)`）；9764-9771 boundary 白框（→纸面）；9913-9914 `.brand-mark` 朱红底（→`var(--accent)`，**石绿印章=归真主项**，进截图对照）；10258-10260 `.stage > .notice-panel` 朱红 dashed（→`var(--warning)` 系中性化）；10383-10384/10395/10399-10400 agent 半透明白（→纸面）；9588-9592 `--ui-*` 定义本体（随 §2.3 退役）。
3. **孤儿规则先迁后删**（样式只在桌面皮，迁基座层相应区，一行不改）：`.secretary-float`（10151-10153）、`.agent-boundary-details/.agent-boundary-summary`（9764-9780 治平后保留+10406-10418）、`.hide-dev-detail`（10423-10425，**功能性默认隐藏，迁移后必须验**）、`.jiaoban-done-pills`（10444-10449）、`.project-side-dev-detail-toggle`（10428-10441）、`.secretary-dock-trigger`（10024-10028）、尾部全局段 10421-10455 全段。

## 四、hex 归 token（255 处全量，口径=勘察可复核数）

1. **a 类（有 token 可映射）**：直接替换（如 `#a14242→var(--vermil)`、`#fff→var(--panel-raised)`、回退位 `#d6d2c8→var(--hair)` 等）。
2. **b 类（无 token）**：§2.1 新增 `--paper-card` 与 `--canvas-*` 族接收；零散单值（勘察 §3.1 表逐值）并入就近 token 或新增单值 token，**值一律取 live，不许顺手调色**。
3. **错误回退修正**（回退值与活 token 不符，替换成 var 引用删回退）：`sidePanel.css` `#9b4a18×5→var(--warning)`、`#236c68×3→var(--accent)`、`#9a8f80×3→var(--ink-light)`；`ActiveWorkbenchView.tsx:589 #e55b5b→var(--danger)`；`WorkflowCommandConsoleView.tsx:175 #b80→var(--warning)`。
4. **c 类（一次性合理，登记白名单不动）**：boot 诊断屏 6 值（styles.css:98/101/129/142 区）、`lib/canvasNodeData.ts` 16 处节点调色板数据（数据非样式）、SVG data-URI 转义 hex 9 处（`%23…`，含死壳 .node 内 1 条）。
5. `index.html:12` 内联 hex：改写为正典值（在 src 扫描面外，单独核验）。

## 五、字体收敛

1. **字号**：21 裸档 → 正典 7 档。映射：9/10.5→`--text-2xs`；11/11.5→`--text-xs`；12/12.5→`--text-sm`；13/13.5→`--text-md`；14→`--text-md`（进对照）；15/16→`--text-lg`；17/18→`--text-lg` 或保留 17 裸值处改 `--text-xl`；20/22/24→`--text-xl`；26/28/30/32→`--text-2xl`。TSX fontSize 10 处同映射。**0.5px 档消灭**。
2. **字重**：650×9→600；800×8→700。
3. **等宽栈 6→1**：全部改 `var(--font-mono)`（定义见 §2.1）。
4. **幽灵 webfont 名全删**：`Noto Serif SC`（42 处声明）、`DM Sans`（5）、`DM Mono`（1）、`Source Han Serif`（3）、`JetBrains Mono`（39）从声明中移除，栈落到正典三族；`index.html` 不加任何字体加载（拍板=本地字体）。
5. 宋/黑分工写成 `--font-serif/sans` 注释规矩：标题与主管正文=宋，chrome/标签/meta=黑，数字/代码/路径=等宽。

## 六、shape gate 新规则 `hardcoded_hex_on_ui`

1. 规则本体放 `scripts/harness/lib/hardcoded-hex-rule.js`（照 `machine-face-rule.js` 形：scanX+attachX；gate 本体已 492 行，**只许加 require/挂载/打印各行一**，破 500 软限=打回）。
2. 扫描面：6 个 CSS + 全部 .ts/.tsx 内联 style；排除：正典定义行（`^\s*--[\w-]+\s*:`）、注释、本包 c 类白名单；var() 回退位**算违规**（错误回退实证）；`%23` 转义形算违规。
3. 级别：error（新增零容忍）。白名单=`hex值|path` 粒度，**86 条预登记**（勘察 §6 全清单：styles.css 49 + sidePanel 25 + ActiveWorkbenchView 3 + canvasNodeData 6 + ProjectWorkflowCanvasView 1 + WorkflowCommandConsoleView 2），本包施工后实际命中数必须 ≤ 预登记（治平一批就核销一批，**白名单只减不增**）。
4. 三件套：selftest（照 machine-face 夹具树，断言：裸 hex→error、白名单→deferred、正典定义行→不误伤、回退位→error）+ `docs/harness-catalog.md` 登记 + decisions 落档（`decisions/2026-07-20-hardcoded-hex-gate-rule-and-whitelist-v1.md`）。

## 七、允许读取

- 本包、上位决策、DESIGN.md、样张、交互宪法、七律
- `prototypes/productized-desktop-shell/`（src/**、index.html）、`scripts/harness/**`、`docs/harness-catalog.md`

## 八、允许写入

- `prototypes/productized-desktop-shell/src/styles.css`、`src/manualRelay.css`、`src/components/sourceStylePlaceholder.css`、`src/views/memory/memoryCenter.css`、`src/views/projects/projectWorkflowSidePanel.css`、`src/views/projects/projectReferencePanels.css`
- TSX 仅 5 处内联 hex 替换（`ActiveWorkbenchView.tsx:543/589/621`、`WorkflowCommandConsoleView.tsx:175/228`、`ProjectWorkflowCanvasView.tsx:994`）+ fontSize 映射 10 处；`index.html:12` 一处
- `scripts/harness/lib/hardcoded-hex-rule.js`（新建）、`scripts/harness/workbench-shape-gate.js`（+3 行内）、`scripts/harness/workbench-shape-gate.hardcoded-hex.selftest.js`（新建）
- `docs/harness-catalog.md`、`decisions/2026-07-20-hardcoded-hex-gate-rule-and-whitelist-v1.md`（新建）、`evidence/2026-07-20-g1-token-truth-restoration-verification-v1.md`（新建）
- `CURRENT.md` 最小回写（收口后）

## 九、禁止事项

1. 零类名/选择器改名、零 TSX 结构变化（除 §八列出的值替换）、零 JS/TS 逻辑变化、零 Rust 改动。
2. 离线测试/渲染断言文件逐字不动；断言挂=你破了视觉语义，修施工不修断言。
3. 中性色不许顺手调色（live 值入正典）；b 类 hex 只收不挑。
4. 白名单只减不增；新增违规零容忍；不许为过关把违规塞进白名单。
5. 孤儿规则未迁先删=打回（`.hide-dev-detail` 功能失效=B 级事故）。
6. 不 stage、不 commit。
7. **视觉变化不得超出本包枚举**（--bg 定回、warning 定回、桌面皮白卡→纸、brand-mark 朱红→石绿、notice-panel 中性化、--shadow 轻落地、字号映射、字重收敛、mono 统一）——发现第 10 项先停，回总指导补枚举。

## 十、变更辐射面

- 改变「token 四个时代并存」假设 → 全部 `var(--x)` 引用面（69 活 token）依赖旧覆盖链终值：alias 桥接后终值逐点不变（除 §九.7 枚举），**对账法=正典前后 computed 值抽样比对**（evidence 给抽样表：--bg/--ink/--ink-mid/--muted/--line/--panel/--warning/--danger/--candidate/--topbar-h 十项 × 1280/577 两断点）。
- 桌面皮死段删除 → 依赖这些死选择器的 TSX：勘察已核零引用；执行线须复 grep 一遍（命令进 evidence）。
- 字号/字重映射 → 全 App 文本：离线渲染断言全绿+截图对照四视图（首页/交办/记忆中心/智能体页 × 1280，交办加 577）。
- gate 新规则 → 之后所有包新增裸 hex=error。

## 十一、五态旅程走查

- 说：方案卡底不变（纸面），brand-mark 变石绿（枚举项）。
- 批：方案卡/授权卡无白卡浮起（桌面皮治平），印章语义不变。
- 干：画布节点色族（--canvas-*）值不变只归 token；工序图不变。
- 交货：交货卡 pill 行（`.jiaoban-done-pills` 孤儿迁移）不破。
- 卡住：notice-panel 朱红→warning 中性（枚举项）；黄牌色调随 --warning 定回（进对照）。

## 十二、形状影响

- 任务类型：**治理任务包**（形状指标改善+枚举化呈现归真）。
- 新增代码落点：`scripts/harness/lib/hardcoded-hex-rule.js` + selftest；CSS 零新增文件。
- 棘轮文件：`styles.css`（只降不升，退役死段+正典合并预计 −2500~-3500 行）、`workbench-shape-gate.js`（+3 行内）；`lib.rs`/Rust 零碰。
- 预计行数：styles.css −2500 起；sidePanel/memoryCenter 等 −100 内（回退删除）；TSX 净变 <30 行。
- 新增 Tauri command：无。新增 sidecar JSON：无。
- shape gate 豁免：hex 白名单 86 条全登记 decisions，不沉默豁免。
- 本任务基线 commit：`1037819`。完成 commit：总指导核收后另定（执行线不 commit）。

## 十三、验收标准

1. 四闸：`npx tsc --noEmit`=0；`node scripts/run-offline-interaction-test.mjs` 全绿；shape gate baseline+check **13/5/5 零净增**（hex 规则自身 findings 全 deferred，零 error）；`git diff --check` 过。（Rust 零碰，cargo 可免跑，须明写理由。）
2. token 对账：六处 `:root` 坍为一处+两断点尺寸块；83→正典表计数；14 死 token grep 清零；§十 computed 值抽样表进 evidence。
3. hex 对账：施工前 255（190+49+22+index.html 1+转义 9 口径分列）→ 施工后裸值清零或全在白名单；白名单 ≤86 且只减不增；a/b 类映射逐值 grep 证明。
4. 桌面皮：死段删除行数对照；孤儿 6+1 件迁移后 grep 仍在且功能在（`.hide-dev-detail` 默认隐藏实测）。
5. 字体：半像素档/650/800 grep 清零；等宽栈归一；幽灵字体名 grep 清零。
6. **截图对照（感受件）**：首页/交办/记忆中心/智能体页 ×1280 + 交办 ×577，施工前后各一套，差异仅限 §九.7 枚举项；先过总指导看形，**再请用户最后一眼**。

## 十四、必须回传（按 TASK_TEMPLATE 10 项）

做了什么 / 改了哪些文件 / 新增哪些测试或证据 / 哪些结论有依据 / 哪些仍不确定 / 风险和下一步建议 / shape gate baseline+check 摘要（含 hex 规则三数）/ start-end commit / 是否新增 command·sidecar·触碰棘轮文件 / **被闸拦过的事**（无也必须写「无」）。

## 十五、总指导回收动作

- 亲跑四闸不信回传；token/hex 对账表逐行对实物；截图对照逐张看，差异出枚举=打回；`.hide-dev-detail` 功能亲验。
- 用户最后一眼确认后判断 接受 / 需要修改 / 暂停 / 废弃，记 `docs/harness-catch-log.md`。
