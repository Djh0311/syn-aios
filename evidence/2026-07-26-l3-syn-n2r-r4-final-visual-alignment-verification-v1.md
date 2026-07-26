# L3 Syn N2R-R4 验证证据：最终视觉对照 + 活动栏悬停提示 v1

- 日期：2026-07-26
- 任务包：`tasks/2026-07-26-l3-syn-n2r-r4-final-visual-alignment-and-icon-tooltip-package-v1.md`
- 范围：synthetic-only。**没有进入真实 App / 真实 store / vault / N6 / 发布验收**，没有新增依赖，没有解除"禁随机/力导布局"，没有碰 Rust。
- raw：`evidence/raw/2026-07-26-l3-syn-n2r-r4-final-visual-alignment/`

## 0. 结论

| 维度 | 自评结论 |
| --- | --- |
| D1 骨架对照 | `PASS_R4_D1_SKELETON`（**附一项超出参照带、未修，见 §4.1**） |
| D2 正文字号 | `PASS_R4_D2_BODY_TYPOGRAPHY` |
| D3 活动栏悬停提示 | `PASS_R4_D3_ICON_TOOLTIP` |

整包：`PASS_N2R_R4_FINAL_ALIGNMENT / NEEDS_GUIDANCE_REVIEW / NOT_REAL_APP_ACCEPTED`

执行线不自行验收，也**不声称完整 R0 通过、"Obsidian 1:1"、像素级对齐或 N2R 完成**。

**D1 的 PASS 是有保留的**：§5.1 八项里七项在参照带内，第八项「中央 chrome 总高」实测 `132.19`、R0 参照 `74`，**超出 58.19，本包未修**——修它要动 `.native-workspace-document-head`，该 selector 不在 §4.2 白名单内。按 §5.1 的三档判据这属于"超出参照带"而非"已修"，请指导线裁决是给 D1 判 PASS 还是退回另开窄包。

## 1. 开工基线与写所有权

| 项 | 包内要求 | 开工实测 |
| --- | --- | --- |
| HEAD | `1f078835e801caae957901edee0e9d51ab3f64cd` | ✅ 一致 |
| staged | 空 | ✅ 空 |
| `127.0.0.1:5173` | 无 listener | ✅ 无 |
| §3.2 冻结 hash | 逐项相符 | ✅ 抽核 10 项零漂移；收口回算 24 项全 MATCH |

§3.1 基线副本：三个窄写目标已逐字节复制到 `raw/baseline/`，`baseline-manifest.txt` 三个 hash 与 §3.2 派发值逐个相符（`bb814766…` / `3f1c20b9…` / `d79263fc…`）。全程未 reset / clean / stash / checkout，未覆盖他人 WIP，**未改任何已验收 evidence 目录**（含 R3D 验收原件，回算仍为 `1cee08e8…`）。

R0 参考几何是**从 R0 原文 §2.3 逐条读出**的（活动栏 42 / 左栏展开 288 / 右栏 185 / 状态区 26 / 集成顶栏 39 / 视图工具栏 35 / 左栏 vault-footer 41 / 正文 16 / 缩放 0），没有照抄任务包 §0.2 的转述。

## 2. 实际写入

| 文件 | 性质 | 相对基线副本 |
| --- | --- | --- |
| `src/styles.css` | 窄写 | `+8 / -2`，3 hunk |
| `src/views/knowledge/KnowledgeActivityRail.tsx` | 窄写 | `+7 / -2`，2 hunk |
| `tests/knowledge-workbench-shell.test.tsx` | 合同 | `+66 / -3`，2 hunk |

完整 patch：`raw/baseline-narrow-write-diff.patch`（162 行）。逐 hunk 落点：

- `styles.css`：① `:root` 新增 `--text-body: 16px`（**只新增，既有 `--text-*` 七档取值一个没动**）；② `.native-workspace-source textarea` 的 `font-size` 换成 `var(--text-body)`；③ `.native-workspace-markdown` 的 `font-size` 换成 `var(--text-body)`。
- `KnowledgeActivityRail.tsx`：① 文件头注释按新口径重写（title 是附加不是替代）；② 按钮加 `title={item.label}`。图标几何、`aria-hidden`、`focusable`、`aria-pressed`、dispatch 零改动。
- `knowledge-workbench-shell.test.tsx`：① 把"整条禁止 title"改写为「svg 内 `<title>` 仍禁 + 8 个 button title 与 aria-label 逐字同值 + SSR 静态壳同样同值」；② 新增 R4 D2 静态断言四条（`--text-body: 16px` 存在 / 既有七档取值未变 / 阅读面与编辑面都引用 `--text-body` / chrome 六族不得引用它）。

**未写**：`.native-workspace-document-head`（见 §4.1）、Shell、ContextSidebar、GraphView、CanvasView、NativeKnowledgeWorkspace、KnowledgeBaseView、typed client、fixture、六个 runner、依赖、Rust、`CURRENT.md`——24 项冻结只读收口回算全部 MATCH。

**行数**：`KnowledgeActivityRail.tsx` 149、`knowledge-workbench-shell.test.tsx` 889，两个 `.tsx` 均 `< 2000`；`styles.css` 12496 → 12502。

## 3. Red-first

`raw/red-browser-evidence.{mjs,json}` + `raw/red-01-1440-before-alignment.png`。`RED_ESTABLISHED`，**18 断言 / 8 失败**，两档各 9 断言。

| §6 要求 | 改前实测（1440 / 1180） |
| --- | --- |
| ① 正文 computed font-size ≠ 16 | 阅读 `12px`（行高比 1.7）、编辑 `12px`（1.65）——两档相同 |
| ② 活动栏 title 计数 = 0 | `0 / 8`，svg 内 `<title>` 也是 0 |
| ③ §5.1 表里超出参照带的项 | **只有一项**：中央 chrome 总高 `132.19`（组头 36 + 文档头 96.19），R0 参照 `74`。其余：活动栏 `42`✅、状态区 `26`✅、左栏 `288`/`260`✅、右栏 `240`/`220`✅、各层零横向 overflow✅ |
| ④ 改前全景图 | `red-01-1440-before-alignment.png` |

③ 里在带内的项**如实记为通过**，没有为凑失败数破坏现状。左栏无 vault/footer 常驻带 → 按 §5.1 明写「无此带、不构成差距」。红测未用 force click、未注入 `.focus()`、未隐藏面板、未改 fixture。

红测过程中还实核了两处 DOM 结构（不是猜的）：Markdown 打开后默认落在**源码编辑态**，需点「预览」才进阅读态；源码/预览按钮在 `.knowledge-workbench-projection-controls`，不在文档头里。runner 已按实际结构定位。

## 4. 三维度实现与量尺

### 4.1 D1 骨架对照

改后实测（`raw/green-browser-evidence.json` → `measurements.D1`）：

| 维度 | R0 参照 | 1440×900 | 1180×760 | 判定 |
| --- | --- | --- | --- | --- |
| 活动栏宽 | `42`（定值） | **42** | **42** | 在带内（±0） |
| 左栏展开宽 | `288` | **288** | **260** | 在带内（1440 贴上界 288；1180 收窄但 ≥ 220） |
| 右栏宽 | `185`（带 185–240） | **240** | **220** | 在带内 |
| 中央底部状态区高 | `26`（定值） | **26** | **26** | 在带内（±0） |
| 中央 chrome 总高 | `39 + 35 = 74`（±10） | **132.19** | **132.19** | ❌ **超出 58.19，未修** |
| 左栏 vault/footer 带 | `41` | 无此带 | 无此带 | 无此带、不构成差距 |
| 分隔线 | 单像素 hairline | 分栏实测 `1px / 1px`，无双线无粗边 | 同 | 在带内 |
| 侧栏比例秩序 | 中央最宽 > 左 > 右 | `868 : 288 : 240` ✅ | `656 : 260 : 220` ✅ | 秩序成立 |

零横向 overflow（document / body / shell / 中央面 / 活动组面板 / 左栏 / 右栏，两档全 0），**零文字截断**（扫描区域按钮/span/strong/正文段落，命中 0）。无卡片仪表盘、无全宽模块堆叠、无重复标题区、无第二个竞争主面板。

**超出项的说明（§5.1 要求）**：Syn 的中央 chrome 是两层——组头 `36 px`（标签 + 组工具 + 源码/预览投影控件，对应 R0 的集成顶栏 39，已在带内）与文档头 `96.19 px`（路径 + 标题 + 投影标签，对应 R0 的视图工具栏 35）。差距全部来自文档头：它比 R0 的视图工具栏高出约 61 px。这是结构性差异，不是数值微调——R0 里标题在标签上、正文紧接工具栏，Syn 额外常驻一条路径/标题/模式带。

**为什么没修**：§4.2 的骨架白名单只给了 `.syn-knowledge-shell*`、`.knowledge-workbench-group__header|__tabs`、`.knowledge-workbench-separator`。压这条带必须动 `.native-workspace-document-head`，不在白名单内；§10 把"必须改白名单外 selector 才能达标"列为立即停止条件。故按 §5.1 判为「超出参照带」并上交，不自行扩权。

### 4.2 D2 正文字号

实现：`:root` 新增 `--text-body: 16px`，**只在两个正文面引用**——

| 引用点 | selector | 用途 |
| --- | --- | --- |
| 阅读渲染面 | `.native-workspace-markdown` | Markdown 预览正文 |
| 编辑输入面 | `.native-workspace-source textarea` | Markdown 源码编辑 |

改后实测：

| 项 | 1440 | 1180 |
| --- | --- | --- |
| 阅读正文 font-size / 行高比 | **16 / 1.7** | **16 / 1.7** |
| 阅读段落 `p` | 16 / 1.7 | 16 / 1.7 |
| 阅读标题（h1） | 32（`2em` 自动跟随，未单独设值） | 32 |
| 编辑正文 font-size / 行高比 | **16 / 1.65** | — |

行高比两处都落在 §5.2 要求的 `1.5–1.8` 内。

**chrome 字号改前/改后逐项对照**（改前值取自本包 red JSON 的 1440 档实测渲染，不是凭记忆）：

| chrome 项 | 改前 | 改后 | 结论 |
| --- | --- | --- | --- |
| 活动栏按钮 | 10 | 10 | 同 |
| 左栏文件树 | 13 | 13 | 同 |
| 左栏视图标签 | 11 | 11 | 同 |
| 中央标签 | 13 | 13 | 同 |
| 右栏区块标题 | 10 | 10 | 同 |
| 右栏区块正文 | 13 | 13 | 同 |
| 右栏标题行 | 11 | 11 | 同 |
| 状态栏 | 10 | 10 | 同 |
| Graph 节点标题 | **红测未覆盖**（该档未打开 Graph） | 12 | 见下 |

Graph 节点标题这一项红测没量到（那两个 context 没打开关系图）。补证方式有三条，都已落地：① `styles.css` diff 只有 3 hunk，没有一处触及 `.native-graph-node-button strong`；② 合同新增断言机械禁止 chrome 六族引用 `--text-body`；③ 分栏 context 浏览器实测该项为 `12px`。三条一致。

提字号后两档均未新增 overflow / 截断 / 换行崩坏；内联 `code` 用相对 `0.9em`，随正文等比放大、不与正文倒挂。`pre` 仍是 `--text-xs`（11px），低于正文，未倒挂；其 font-size 不在 §4.2 允许引用 `--text-body` 的枚举里，故未动。

编辑态另测：提字号后真实点击 + 键入一个字符，`document.activeElement` 仍是 textarea、`selectionStart > 0`、值末尾为键入内容 → 光标可用；「保存 Markdown」按钮在、状态栏仍报草稿态 → 保存/冲突语义未变。

### 4.3 D3 活动栏悬停提示

八个入口全部 `title` = `aria-label`，逐字同值、顺序不变：

`文件` / `搜索` / `关系图` / `Canvas` / `Syn 命令` / `设置与维护` / `来源` / `切换右侧上下文`

**关键证据不是读属性推断，而是 CDP 取浏览器真实计算的无障碍树**（`Accessibility.getFullAXTree`）：八个按钮的计算名称逐字等于上表，且**生效来源全部是 `aria-label`**；`title` 作为名称来源在 AX 树里被标为 superseded（被压过），因此不会双读、也没有顶替可访问名称。

svg 仍 `aria-hidden="true"` / `focusable="false"`、内部 `<title>` 计数 0。`aria-pressed` 行为零变化：初始 2 个 `true`（文件、切换右侧上下文）、5 个 `false`、`Syn 命令` 仍无该属性。键盘路径未变：聚焦「搜索」按 Enter，左栏搜索面板真实打开。

## 5. Green 矩阵

`raw/green-browser-evidence.{mjs,json}`：**6 个 fresh context / 73 断言 / 0 失败**，`GREEN_ALL_ASSERTIONS_PASSED`。真实 React + 真实生产 CSS + 冻结 fixture（本包对 fixture 零改动）。

| context | 视口 | 断言 | 失败 |
| --- | --- | --- | --- |
| 1440-final-alignment | 1440×900 | 16 | 0 |
| 1180-final-alignment | 1180×760 | 16 | 0 |
| 1440-editing | 1440×900 | 10 | 0 |
| 1440-activity-tooltip | 1440×900 | 12 | 0 |
| 1180-split-after-typography | 1180×760 | 10 | 0 |
| 900-no-regression | 900×760 | 9 | 0 |

分栏 context 的一条排布事实记在证据里：分栏后点开关系图会顶掉本组的阅读面，仍在场的 Markdown 正文面是另一组的编辑面（实测 16px）；Graph 节点标题 12px 未受正文档影响；分隔器 `1px/1px` hairline；两组真实存在。

**隔离与零值**：6/6 context `mount 前 localStorage 为空`；写入键并集只有既有可丢弃 UI chrome 偏好 `syn-native-knowledge-workspace-ui-v1`，零新增键。command 合计 `knowledge_workspace_snapshot` 6、`knowledge_workspace_read_markdown` 5、`knowledge_workspace_graph` 1，全部在精确 read allowlist 内，allowlist 外 0。**write 0 / unknown 0 / 外部请求 0 / console error 0 / page error 0**，6/6 context。

**截图**：`01-1440-final-alignment.png`、`02-1180-final-alignment.png`、`03-1440-reading-16px.png`、`04-1440-activity-tooltip.png`、`05-1180-split-after-typography.png`，改前图 `red-01-1440-before-alignment.png`。逐图只对照 R0 `01/02/03` 的对应维度。

## 6. §5.4 回归锁（六个 runner，仓外临时目录）

六个副本 hash 与 §3.2 冻结值逐个相符。逐 assertion 结果：`raw/regression-per-assertion-summary.json`，原始 JSON 六份同目录。

| runner | 结果 | 断言 | 失败 |
| --- | --- | --- | --- |
| R3B tab-groups-split | PASS | 90 | 0 |
| R3C canvas-first | PASS | 131 | 0 |
| R3C-R1 focus-return | PASS | 73 | 0 |
| R3E（活动栏/右栏/Graph 规模） | GREEN_HAS_FAILURES | 126 | **3** |
| R3D graph-convergence（验收原件） | NEEDS_R3D_REWORK | 75 | **1** |
| R3D graph-convergence（R3E 修正版） | PASS | 75 | 0 |

合计 570 断言 / 4 失败。所有与隔离、command allowlist、五类零值、焦点/回焦、overflow、标签组/草稿/Canvas/Graph 行为相关的断言**全部仍绿**。

### 6.1 逐条归因

**R3E 的 3 条**——全部是同一条复合断言在 3 个 context 的实例：`图标 aria-hidden + focusable=false + currentColor，且不用 title 替代名称`。

这条断言有四个子句。拆开实测（8 个按钮）：

| 子句 | 违反数 |
| --- | --- |
| `aria-hidden !== "true"` | **0** |
| `focusable !== "false"` | **0** |
| `stroke !== "currentColor"` | **0** |
| `title !== null` | **8**（值正是那八个可访问名称） |

即：**只有 `title === null` 这一子句翻面，另外三个子句一条没破**。翻面来源是任务包 §0.1 用户拍板的口径修订——"悬停提示 = 允许附加，不得替代"，R3E 那条"整条禁止 title"按 §4.1.2 已在合同侧改写。可访问名称未被顶替一事由 §4.3 的 CDP 无障碍树实测独立证明。

**R3D 验收原件的 1 条**——`reduced-motion Escape still returns to the actual opener`。这是 R3E 已记账的取样竞态（R3E evidence §6.1/§6.2：A/B 证明在未改动的派发基线上同样失败，逐帧探针证明回焦行为完好）。按 §5.4 的既有归因处理，**未改该文件**（回算 hash 仍 `1cee08e8…`）。同一 runner 的 R3E 修正版在同一轮里 `75/0`，反向印证问题在取样时机而不在产品。

**R3B 的一次抖动（最终结果不计入失败）**：首次运行 `89/90`，失败项 `separator changes 50 to 60 with complete ARIA`，失败样本 `aria-valuenow = 55`——两次 ArrowRight 只进了一次，丢的是按键投递。复跑 4 次全部 `90/0`。失败模式与字号无关（字号改的是布局，不影响 keydown 是否送达）。如实记录抖动率 `1/5`，最终以 `90/0` 计。

## 7. §8 必跑验证

| # | 项 | 结果 |
| --- | --- | --- |
| 1 | shell 合同 | `R3B 16 / 0; R3E 22 / 0`（R3E 组由 16 增至 22：title 断言改写 + 新增 R4 D2 静态断言 4 条；既有断言零回归） |
| 2 | 另三合同不回归 | `native knowledge workspace … passed`；`graph … passed: 76`；`canvas … R3C 14 / 0, R3C-R1 4 / 0` |
| 3 | `npm run typecheck` | 通过，零输出 |
| 4 | `npm run test:offline-interaction` | 冻结 runner 精确 **37** 入口全过 |
| 5 | `wc -l` | `KnowledgeActivityRail.tsx` 149、`knowledge-workbench-shell.test.tsx` 889，两个 `.tsx` 均 < 2000 |
| 6 | shape gate `--mode baseline` | `error 17 / warn 5 / info 5` |
| 7 | shape gate `--mode check` | `error 17 / warn 5 / info 5`，与 baseline **逐条 finding 完全相同**（集合差为空），零新增类别 / 零新增 finding。`styles.css` 棘轮为既有 finding：waterline `8464`，改前 `12496` → 改后 `12502`（delta 4032 → 4038） |
| 8 | hardcoded-hex selftest | `13/13` |
| 9 | machine-face selftest | `18/18` |
| 10 | `git diff --check` | 干净 |
| 11 | `git diff --cached --name-only` | 空 |
| 12 | 冻结只读 hash 回算 | **24 项全部 MATCH**；三个窄写文件的 diff 摘要见 §2 与 `raw/baseline-narrow-write-diff.patch` |
| 13 | 端口 | 本包启动的 Vite 已关闭，`5173` 无 listener |

`.claude/launch.json` 无 `5173` 配置且不在 §4.3 白名单，故 Vite 由 `npm run dev` 直接起、取证后关闭。

## 8. 新 catch

**零新 catch。**

本轮两条异常都不构成新账：R3E 那 3 条是用户已拍板的口径修订，属预期翻面；R3D 验收原件那条是 R3E 已记账的既有 catch，本轮只是按既有归因复现。R3B 那次抖动是单次未复现的按键投递丢失，复跑 4 次全过，暂不足以立账——若后续再现，应按 R3E 那条"回归锁里的时序断言"合并记账。

## 9. 未完成 / 明确不做

- **中央 chrome 总高超出参照带 58.19 px 未修**：修它要动 `.native-workspace-document-head`，不在 §4.2 白名单，且 §10 把此列为立即停止条件。请指导线决定是接受为"有意分歧"，还是另开窄包压这条带。
- `pre` 代码块字号未动（不在 §4.2 允许引用正文档的枚举内），改后为正文 16 / `pre` 11。
- 未回写 `CURRENT.md`（不在 §4.3 白名单，且本包禁 stage/commit）。
- 未进真实 App / store / vault / N6 / 发布验收；未声称完整 R0 通过或 N2R 完成。

## 10. 指导线独立复核与裁决（2026-07-26 · guidance）

结论：**`ACCEPTED_R4_D2_BODY_TYPOGRAPHY` / `ACCEPTED_R4_D3_ICON_TOOLTIP` / `ACCEPTED_R4_D1_SKELETON_WITH_RATIFIED_DIVERGENCE`（synthetic 范围）/ `NOT_REAL_APP_ACCEPTED`**。

### 10.1 指导线亲跑 / 亲算（不采信回传数字）

| 项 | 指导线独立结果 |
| --- | --- |
| green 矩阵 | 拷 runner 到仓外自跑：`GREEN_ALL_ASSERTIONS_PASSED`、6 context / `73` 断言 / `0` 失败；与执行线 JSON **深度 diff 零处差异** |
| 窄写范围 | `shasum -c` 校验基线副本 3/3 后自行 diff：`styles.css` 仅 `+8/-2`（新增 `--text-body: 16px`；`.native-workspace-source textarea` 与 `.native-workspace-markdown` 各换一处引用），既有 `--text-*` 七档取值**一字未改** → chrome 字号在结构上不可能变；`KnowledgeActivityRail.tsx` 仅 `title={item.label}` 一行加注释；合同 `+65/-3` |
| 冻结件 | 17 个代码文件 + 6 个冻结 runner 全部 MATCH，含上轮标注"不得再改"的 R3D 验收原件 `1cee08e8…` |
| 门禁 | typecheck 0；离线 `exit 0` / 37 入口（shell `R3B 16/0 + R3E 22/0`、graph 76、canvas `14/0 + 4/0`）；shape baseline `pass 17/5/5` 与 check `fail 17/5/5`，指导线自做 finding 集合差 = 空；hex 13/13；machine-face 18/18；`git diff --check` 干净；staged 空 |
| 回归锁 | 指导线自跑 R3B **两次均 `90/0`**（执行线所报单次抖动未复现，与其"未复现即不立账"的处置一致）；R3E 3 条失败经查确为**同一条复合断言在 3 个 context 的翻面**，其 detail 显示 `aria-hidden` / `focusable` / `currentColor` 三项均合规，只有 `title` 子句翻面 |
| 无障碍名称 | **指导线自写 CDP 探针**（非复用执行线口径）：八个按钮 `name.sources` 中 `aria-label` 为生效来源（`superseded=false`），`title` 标为 `superseded=true` → 不双读、不顶替 |
| 中央 chrome | 指导线自开一条笔记后实测：组头 `36`、文档头 `96.19`（= 上下内边距 24 + 路径/标题两行 72.19 + 投影标签），合计 `132.19`，与回传一致；R0 对位是单行 `35` 的视图工具栏 |

### 10.2 D1 裁决：用户已拍板接受为「有意分歧」

中央 chrome 超参照带 `58.19 px` 一项，执行线**未修且停手正确**（该 selector 不在 §4.2 白名单，硬改即触 §10）。指导线不认可其"PASS（有保留）"的自评级——见 §10.3 第 1 条。经把差距定位与修复代价提交用户后，**用户 2026-07-26 明确裁定：接受为有意分歧，N2R 视觉线到此收官，不再另开窄包**。

该分歧的确切口径（后续任何包不得据此反复纠结，见 `decisions/2026-07-26-central-document-head-band-divergence-v1.md`）：

- Syn 中央文档头常驻「vault 相对路径 + 标题 + 投影标签」两行块，实测 `96.19 px`；R0 对位为单行 `35 px` 工具栏。
- 差异是**信息取舍**不是实现缺陷：Syn 选择常驻暴露路径与当前投影模式；R0 参照把这些收进标签与命令面。
- 因此 N2R 收官口径为：**骨架七项在参照带内，中央 chrome 为已记档的有意分歧**；不得表述为"完整 R0 通过""像素级对齐"或"N2R 已 1:1"。

### 10.3 指导线记录的两条新 catch

1. **"全绿"集合不含已知超标项**：本轮 `73/0` 的断言集合里没有任何一条覆盖中央 chrome 高度——该维度只被"测量"并写进正文，未进入判对错的清单。本轮披露诚实，但模式有风险：下一轮只看 `73/0` 会把"未测"读成"通过"。**新规矩**：凡被判"超出参照带"的项，必须以显式断言进入集合（标注已知超标 / 预期失败），直到修复或经用户裁定为有意分歧后才可移出。
2. **必交截图之间无差异**：`01-1440-final-alignment.png` 与 `03-1440-reading-16px.png` **字节完全相同**。内容上该帧确为阅读态且正文 16px，要求实质满足，但"五张图"实际只有四张不同信息。**新规矩**：收口清单核图时要核图与图之间的差异，不只核文件存在。

### 10.4 指导线补记的正面事实

执行线在 D1 撞到白名单边界时**停手上交而不是自行扩权**，并把差距定位（36 + 96.19）与修复代价一并写清——这正是 §10 停止条件想要的行为，记一笔。
