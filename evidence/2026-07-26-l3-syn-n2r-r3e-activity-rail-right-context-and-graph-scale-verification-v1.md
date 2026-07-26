# L3 Syn N2R-R3E 验证证据：活动栏 ribbon + 右栏层级 + Graph 布局规模化（合并包）v1

- 日期：2026-07-26
- 任务包：`tasks/2026-07-26-l3-syn-n2r-r3e-activity-rail-right-context-and-graph-scale-package-v1.md`
- 范围：synthetic-only。**没有进入真实 App / 真实 store / vault / N6 / R4 最终视觉对照**，没有解除"禁随机/力导布局"禁令，没有新增依赖。
- raw：`evidence/raw/2026-07-26-l3-syn-n2r-r3e-activity-rail-right-context-and-graph-scale/`

## 0. 结论

| 维度 | 自评结论 |
| --- | --- |
| D1 活动栏 ribbon | `PASS_R3E_D1_ACTIVITY_RIBBON` |
| D2 右栏层级 | `PASS_R3E_D2_RIGHT_CONTEXT` |
| D3 Graph 布局规模化 | `PASS_R3E_D3_GRAPH_SCALE` |

整包：`PASS_N2R_R3E_MERGED / NEEDS_GUIDANCE_REVIEW / NOT_REAL_APP_ACCEPTED`

执行线不自行验收。三维度全 PASS 也只回交指导线复核。

**一处经用户明确授权、超出 §4.3 白名单的写入**：§5.4 回归锁首轮 R3D 有 1 条断言失败（`06-900-reduced-motion-collapsed: reduced-motion Escape still returns to the actual opener`）。A/B 证明它在**未改动的派发基线**上同样失败、且回焦行为本身完好，属于该 runner 的竞态取样缺陷。执行线首轮如实上交未动；**用户看过归因后明确指示"直接修吧"**，遂修掉该处取样时机（不是产品代码），并做反向红测证明断言未被改软。修后 R3D `75/0`。详见 §6.1 / §6.2。该 runner 的冻结 hash 因此变更，需指导线更新后续包的冻结表。

## 1. 开工基线与写所有权

| 项 | 派发要求 | 开工实测 |
| --- | --- | --- |
| HEAD | `e9ad7f3a204a1ebb11ce26c1e8c05b19c04c0991` | 一致 |
| staged | 空 | 空 |
| `127.0.0.1:5173` | 无 listener | 无 listener |
| §3.2 冻结 hash | 21 项 | **2 项漂移**（见下） |

### 1.1 开工时抓到并向用户上报的写所有权冲突

第一次复核（`18:43`）发现两个窄写目标已偏离派发 hash：

- `prototypes/productized-desktop-shell/src/views/knowledge/KnowledgeGraphView.tsx`：派发 `c9151ff…` → 实测 `4ab2787…`，mtime `18:21:39`
- `prototypes/productized-desktop-shell/tests/knowledge-workbench-visual-fixture.tsx`：派发 `ea27e80…` → 实测 `4e69d0f…`，mtime `18:15:04`

同时 `evidence/raw/2026-07-26-…-r3e-…/` 已存在（指导线派发前实核写明"目标 evidence 不存在"），内含 §3.1 基线副本 + manifest（`18:13`）、red runner/JSON/5 张红图（`18:19`）、纯函数探针（`18:21`）；`~/.codex/sessions` 多个 rollout 当刻仍在写。

执行线按 §3/§10 **停止、零写入**，回交 `BLOCKED_N2R_R3E_WRITE_OWNERSHIP_CONFLICT`，并把处置权交回用户。

**用户随后明确指示"你接着做"**，本线据此接管写所有权，从既有 WIP 继续。接管前复核：仓内最后一次写入停在 `18:21:39`，接管时刻 `18:48`，近 27 分钟零写入，无活跃 writer。全程未 reset / clean / stash / checkout，未覆盖或合并他人 WIP。

### 1.2 继承到的在途工作（不是本线所写，但本线负责其正确性）

- §3.1 基线副本 7 个文件 + `baseline-manifest.txt`：本线逐条 `shasum -c` 校验，**7 个 hash 与 §3.2 派发值完全相符**，即副本确实取自改动前。
- red 浏览器证据：`13 断言 / 7 失败`，`RED_ESTABLISHED`（三维度反例齐全，见 §3）。
- fixture 规模场景（§4.1.5）：已按 `data-fixtureScenario = graph-scale-<N>` 落地，既有 `syntheticGraph` 分支保留。
- `KnowledgeGraphView.tsx` 的 D3 环形布局：已落地但**自身不变量不成立**——纯函数探针当时报 `FAIL_PURE_FUNCTION_INVARIANT`、`465` 处失败。本线接手后修好（§4.3）。

## 2. 实际写入

| 文件 | 性质 | 相对 §3.1 基线的改动 |
| --- | --- | --- |
| `src/views/knowledge/KnowledgeActivityRail.tsx` | **新增** | 144 行（全新） |
| `src/views/knowledge/KnowledgeContextSidebar.tsx` | **新增** | 172 行（全新） |
| `src/views/knowledge/NativeKnowledgeWorkspace.tsx` | 窄写 | `+32 / -78`，5 hunk |
| `src/views/knowledge/KnowledgeGraphView.tsx` | 窄写 | `+103 / -7`，5 hunk（含继承部分） |
| `src/styles.css` | 窄写 | `+79 / -2`，4 hunk |
| `tests/knowledge-workbench-shell.test.tsx` | 合同 | `+139 / -4`，5 hunk |
| `tests/native-knowledge-workspace.test.tsx` | 合同 | `+24 / -0`，1 hunk |
| `tests/knowledge-graph.test.tsx` | 合同 | `+74 / -6`，3 hunk |
| `tests/knowledge-workbench-visual-fixture.tsx` | 仅新增场景 | `+41 / -2`，2 hunk（继承部分） |

逐 hunk 的完整 patch：`raw/baseline-narrow-write-diff.patch`（816 行）。逐 hunk 落点：

- `NativeKnowledgeWorkspace.tsx`：① import 两个新组件；② 静态壳 activityRail 换成 ribbon；③ 实时壳 activityRail 换成 ribbon；④ 实时壳 rightSidebar 换成 `KnowledgeContextSidebar`（含 §1.2 删字）；⑤ 删掉 `WorkspaceMetadata`（迁入右栏组件）。reducer、草稿/保存写路、overlay、搜索、标签组、状态栏零改动。
- `KnowledgeGraphView.tsx`：① `useStore` import；② `titleReadable` state + stage className/data 属性；③ `minZoom` 改为节点数驱动 + 挂 `KnowledgeGraphZoomTier`；④ `KnowledgeGraphZoomTier` 组件；⑤ 布局常量与 `knowledgeGraphRingCapacity` / `knowledgeGraphRingPlan` / `knowledgeGraphLayoutRadius` / `knowledgeGraphMinZoom` / `deterministicKnowledgeGraphPosition`。`loadKnowledgeGraph`、`openKnowledgeGraphNode`、`nextKnowledgeGraphOpenRequest`、accessible label 生成、typed client 调用面、`onOpenMarkdown(relativePath)` 语义均未动。
- `styles.css`：① `.syn-knowledge-shell__activity button`（ribbon 布局）；② 新增 `.native-activity-icon`；③ 新增 `.native-context-*` 族（`stack/section/heading/summary/chevron/badge/body`，含 `[hidden]`）；④ 新增 `.native-graph-flow-stage.is-compact-zoom .native-graph-node-button strong` 与同族 padding。全部落在 §4.2 允许的 selector 族内；未改全局 token、`:root`、左栏、中央标签组、Canvas、maintenance、overlay、状态栏或非本包 media rule；无新增 hardcoded hex / gradient / blur / 装饰 shadow / 第二套 token。

**未写**：注入方 `KnowledgeBaseView.tsx`、Shell、Canvas、maintenance、typed client、四个冻结 runner、fixture 既有场景、Rust/Cargo、依赖、`CURRENT.md`、`AUTHORITY.md`、计划、决策、真实 store/vault、`.codex`——全部 §8.12 回算 hash 逐个 MATCH。

### 2.1 行数（§8.5）

| 文件 | 行数 | 硬限 |
| --- | --- | --- |
| `NativeKnowledgeWorkspace.tsx` | **1932**（派发时 1978，抽出后下降 46） | 2000 |
| `KnowledgeActivityRail.tsx` | 144 | 2000 |
| `KnowledgeContextSidebar.tsx` | 172 | 2000 |

## 3. Red-first

红测在产品改动前建立，反例逐项留档。浏览器红：`raw/red-browser-evidence.{mjs,json}` + 5 张红图，`RED_ESTABLISHED`，`13 断言 / 7 失败`。纯函数红：`raw/red-graph-layout-invariant-probe-baseline.{mjs,json}`——直接 bundle §3.1 基线副本重算，不复刻算法。

| §6 要求 | 实测反例 |
| --- | --- |
| ① 活动栏按钮为中文文字、`900` 下断行 | 栏宽 `42`，8 个按钮可见文字为 `文件/搜索/关系图/Canvas/Syn 命令/设置与维护/来源/上下文`；`900` 下 4 个按钮真断行（`关系图`、`Syn 命令`、`设置与维护`、`切换右侧上下文`），按行盒计数判定而非 clientHeight 推断 |
| ② 活动栏无 inline SVG | `svgCountTotal = 0` |
| ③ 右栏无区块折叠控件、平铺长列表 | 右栏 `aria-expanded` 总数 `1`（只有"折叠右侧栏"自己），`sectionsWithCollapseControl = 0`，`sectionCount = 2`，注入内容 28 条平铺 |
| ④ 标题声明 `大纲` 但无实现 | 标题实测 `属性 / 反链 / 大纲 / 来源上下文`；本线自行复算：全仓 `大纲` 命中 1 处（`NativeKnowledgeWorkspace.tsx:1694` 标题字符串），`outline*` 标识符命中 0 |
| ⑤ 纯函数最小中心距小于节点盒宽 | 基线副本实算（节点盒宽 `136`）：n=6 → `124.2`；n=7 → `96`；n=12 → `58.87`；n=40 → `17.12`；n=100 → `6.32`；n=512 → `1`。布局外接尺寸恒为 `220×320`，与 `total` 无关 |
| ⑥ 浏览器实测节点矩形相交 | n=6 → `0` 对（如实记录：6 节点在浏览器里未相交，虽然中心距已低于盒宽——两轴未同时重叠）；n=40 → **116** 对；n=512 → **22180** 对 |

⑥ 的 n=6 一条是"现状意外通过"，如实记录，未为凑失败数破坏现状。红测未用 force click、未注入 `.focus()`、未隐藏面板、未改 fixture 既有场景。

## 4. 三维度最小实现

### 4.1 D1 活动栏 ribbon

`KnowledgeActivityRail.tsx` 只做呈现：接收既有 view/tab/collapse 状态与回调，不持业务状态。八个入口的可访问名称逐字保留；`aria-pressed` 只在既有有状态入口出现（`Syn 命令` 仍无）；dispatch/overlay 行为零变化——`Syn 命令` 仍把 `event.currentTarget` 交给 `openWorkbenchOverlay`，回焦链未动。

图标是本仓自绘的极简几何 inline SVG（一页纸 / 放大镜 / 三节点两边 / 画布分栏 / 命令提示符 / 推子 / 三层堆叠 / 右侧分栏），`stroke="currentColor"`，`aria-hidden="true"`、`focusable="false"`，不承担可访问名称，也没有用 `title` 提示替代。零新增依赖、零外部资源、无第三方图标包或品牌资产。SSR 静态壳同步换成同一 ribbon（该组件 hook-free，静态壳的 hook-free 约束保持）。

### 4.2 D2 右栏层级

`KnowledgeContextSidebar.tsx` 把右栏分为「属性」「反向引用」「来源上下文」三区，每区一个 `<h3>` 内的折叠按钮，`aria-expanded` + `aria-controls` 明确，内容用 `hidden` 同时退出 Tab 顺序与 AT 树。默认三区全展开——固定、可预测。

**删字实际落点**：`NativeKnowledgeWorkspace.tsx` 旧行 `1694` 的 `属性 / 反链 / 大纲 / 来源上下文`，随右栏整体迁入组件后改为——

> 改后标题逐字文本：**`属性 / 反向引用 / 来源上下文`**

**本阶段不提供大纲**，也没有做任何大纲派生（含只读标题列表）。全仓 `大纲` 命中数复算：**1 处**，即 `KnowledgeContextSidebar.tsx:14` 的说明性注释 `// - 本阶段不提供大纲：右栏标题不再声明「大纲」，也不派生任何标题列表。`——它是负面声明，不是 UI 字符串；渲染结果里 `大纲` 命中 **0**（由 `native-knowledge-workspace` 合同 + 浏览器 `rightTextDeclaresOutline=false` 双向锁）。`outline*` 标识符命中 0。

注入方 `KnowledgeBaseView.tsx` 与夹具 `SyntheticSourceContext` 均未修改：层级由产品侧容器在**未改动**的注入内容外面包一层实现。折叠状态**不持久化**——不写 store/vault/Markdown，也不新增任何 localStorage 键。

### 4.3 D3 Graph 布局规模化

确定性公式（固定输入 → 固定坐标，无随机、无力导、无坐标持久化、无第二图框架）：

```
节点盒          = 136 × 40           （与 .native-graph-node 同源）
最小间距        = 24
合同下界 PITCH  = 136 + 24 = 160     （任意两节点中心距的下界）
布局步距 LAYOUT = PITCH + 2 = 162    （坐标 Math.round 到整数，两点距离最多被磨掉 √2≈1.42px）

环容量(radius)  = ⌊π / asin(LAYOUT / 2radius)⌋      （相邻弦长 ≥ LAYOUT）
环序列(total)   = total ≤ 容量(LAYOUT) ? 单环 radius = LAYOUT / 2sin(π/total)
                                      : 第 k 环 radius = LAYOUT × k，由内向外按容量填满
坐标(i,total)   = 原点 + radius·(cos θ, sin θ)，θ = -π/2 + 2π·环内序号/环内节点数
minZoom(total)  = min(0.35, 360 / (2·最大半径 + 40))
readableZoom    = 28 / 40 = 0.7      （达标缩放：节点盒 ≥ 28×28）
```

容量推导直接来自"节点盒宽 + 间距"：环内靠弦长 ≥ 步距，跨环靠半径差恒为一个步距。

**接手时的缺陷与修复**：继承实现里 `KNOWLEDGE_GRAPH_LAYOUT_PITCH`（162）被定义并注释了理由，但环容量与环序列实际用的是 `KNOWLEDGE_GRAPH_NODE_PITCH`（160），取整后最小中心距掉到 `159.x`；另有一处浮点边界让 `π/asin(0.5)` 算成 `5.999…`、把整齐的 6 掉成 5。本线两处都修了（改用 `LAYOUT_PITCH` + `1e-9` 余量 + 退化分支 `>= 1` 改 `> 1`）。修复前 `FAIL_PURE_FUNCTION_INVARIANT / 465 失败`，修复后 `PASS_PURE_FUNCTION_INVARIANT / 0 失败`。

缩放分层：低于 `readableZoom` 时整体撤掉渲染的标题（`.is-compact-zoom` 只 `display:none` 掉 `strong`），**`aria-label` 是 button 的 DOM 属性，与缩放无关，任何 `n`、任何缩放下都完整**。没有隐藏节点、没有隐藏边、没有裁切舞台、没有改 `MAX_GRAPH_NODES`、没有碰 Rust、没有改 payload、没有把 node id 当路径。

## 5. Green 证据

### 5.1 纯函数不变量（`raw/graph-layout-invariant-probe.{mjs,json}`）

`PASS_PURE_FUNCTION_INVARIANT`：**1…512 全量扫描**（不是抽样），零失败；确定性复算 365 次全等；每环弦长自洽。

| n | 最小中心距 | 环数 | 最大半径 | 外接尺寸 | minZoom |
| --- | --- | --- | --- | --- | --- |
| 1 | — | 0 | 0 | 136×40 | 0.35 |
| 2 | 162 | 1 | 81 | 298×202 | 0.35 |
| 6 | 161.74 | 1 | 162 | 460×364 | 0.35 |
| 7 | 161.74 | 2 | 324 | 784×688 | 0.35 |
| 12 | 161.74 | 2 | 324 | 784×688 | 0.35 |
| 40 | 161.74 | 4 | 648 | 1432×1336 | 0.2695 |
| 100 | 161.74 | 6 | 972 | 2080×1984 | 0.1815 |
| 512 | 161.25 | 13 | 2106 | 4348×4252 | 0.0847 |

全部 ≥ 合同下界 160。

### 5.2 浏览器矩阵（`raw/green-browser-evidence.{mjs,json}`）

**8 个 fresh context / 126 断言 / 0 失败**，`GREEN_ALL_ASSERTIONS_PASSED`。真实 React + 真实生产 CSS + 冻结 fixture（规模场景为唯一新增）。

| context | 视口 | 断言 | 失败 |
| --- | --- | --- | --- |
| 01-1440-activity-ribbon | 1440×900 | 17 | 0 |
| 02-900-activity-ribbon-narrow | 900×760 | 18 | 0 |
| 03-1440-right-context-sections | 1440×900 | 19 | 0 |
| 04-900-right-context-collapsed | 900×760 | 12 | 0 |
| graph-n6 | 1440×900 | 16 | 0 |
| graph-n40 | 1440×900 | 13 | 0 |
| graph-n512 | 1440×900 | 13 | 0 |
| 08-900-reduced-motion | 900×760 | 18 | 0 |

**D1 量尺**：栏宽 `42 px`（1440 / 900 / reduced-motion 三档一致，落在 36–48）；8 按钮，`svg` 总数 8，**文本行盒总数 0**（真断行判据）；每个按钮 `33.98 × 34`，`hitTargetOk` 全 true；`scrollHeight ≤ clientHeight`、`scrollWidth ≤ clientWidth` 全满足；活动栏本身零横/纵向 overflow；`:focus-visible` 实测 `outlineStyle=solid`、宽度 ≥ 1；初始 `aria-pressed="true"` 2 个（`文件`、`切换右侧上下文`）、`false` 5 个、无该属性的仅 `Syn 命令`；点开关系图后 `关系图` 变为 pressed。折叠右栏 → 用 `切换右侧上下文` 恢复；折叠左栏 → 用 `搜索` 恢复且搜索面板真出现；`Canvas` 入口仍打开 Canvas 面板。

**D2 量尺**：三区块稳定（`属性 / 反向引用 / 来源上下文`），三个 `aria-expanded` 折叠控件 + 右栏折叠控件共 4 个，`aria-controls` 全部可解析；默认展开集合固定为三区全开；标题逐字 `属性 / 反向引用 / 来源上下文`，`headerDeclaresOutline=false`、`rightTextDeclaresOutline=false`。注入的 `SyntheticSourceContext` **在未修改前提下**被产品侧容器包住：`sourceContextInsideProductContainer=true`，28 条量尺项逐字保留（首 `上下文量尺 01`、尾 `上下文量尺 28`、标题 `尚未选择合成笔记`）。逐区折叠 → `aria-expanded=false` + `hidden` + `display:none` + 区内可聚焦元素数 `0`；重新展开全部还原。**真实 Tab 走查**：折叠「属性」后从其标题按 Tab，焦点直接落到「反向引用」标题，证明折叠内容真的不在 Tab 路径上。空态文案 `打开笔记后会显示安全属性、标签和反向引用。` 在 1440 与 900 下都仍可见。`900` 双栏展开时右栏 `scrollWidth ≤ clientWidth`（零横向 overflow），纵向滚动限制在右栏容器内。

**D3 量尺（全部是浏览器实测值，不是算术推导）**：

| n | fitView 缩放 | 该缩放下节点盒实测 | 是否达标(≥28×28) | 标题可见 | `readableZoom` 实测 | 该缩放下节点盒 | 相交对数(fitView / readableZoom) | 全在舞台内 | `aria-label` 完整 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 6 | **1.0** | 136 × 40 | **是** | 是（`titleClipped=false`） | 1.0（0 次放大） | 136 × 40 | 0 / 0 | 6/6 | 6/6 |
| 40 | **0.4955** | 67.39 × 19.82 | 否（俯瞰） | 否（整体隐藏） | **0.7135**（2 次放大） | 97.04 × 28.54 | 0 / 0 | 40/40 | 40/40 |
| 512 | **0.1557** | 21.17 × 6.23 | 否（俯瞰） | 否（整体隐藏） | **0.8033**（9 次放大） | 109.25 × 32.13 | 0 / 0 | 512/512 | 512/512 |

按 §0.1 用户拍板的俯瞰口径分层裁决：`n=6` 的 fitView 即达标（可点、标题可见、未裁切、`zoomTier=readable`，6 节点/5 边全可见，节点盒仍是 `136×40`），`n=40/512` 的 fitView 只作俯瞰、**明写不可点不可读**，可点性与标题只在放大到达标缩放后要求且已实测达成。

两点如实说明：
1. 包 §0.1 里指导线给的 `0.027 / 3.6×1.1`（单环）与 `0.16 / 21×6`（同心环）是算术推导。本包实现走同心环，实测 `0.1557 / 21.17×6.23`——与那组推导相当接近，但上表所有数字都是浏览器实测，**没有照抄推导值**。
2. `readableZoom` 是用真实 zoom-in 控件逐级放大测出的**首个达标档**（每级 ×1.2），不是连续下确界，所以 `n=512` 落在 `0.8033` 而非理论 `0.7`。没有用 force click 或程序化设 zoom 造绿。

**规模场景限制（明写）**：`n = 40 / 512` 的生成节点**只量布局，不激活、不断言可读性**——它们没有可读 Markdown，本包不点开它们。

**reduced-motion**：ribbon + 右栏折叠 + Graph 三处扫描到的元素里，带 transition/animation 的数量为 **0**；折叠态零横向 overflow；Graph 零相交。

### 5.3 隔离与零值

`mount 前目标 origin localStorage 为空`：8/8 context 全 true。写入键并集只有既有可丢弃 UI chrome 偏好 `syn-native-knowledge-workspace-ui-v1`，**零新增键**。

command 次数（精确 read allowlist 内）：

| context | 调用 |
| --- | --- |
| 01 / graph-n6 / graph-n40 / graph-n512 / 08 | `knowledge_workspace_snapshot` ×1、`knowledge_workspace_graph` ×1 |
| 02 | `knowledge_workspace_snapshot` ×2 |
| 03 / 04 | `knowledge_workspace_snapshot` ×1 |

合计 `knowledge_workspace_snapshot` 9、`knowledge_workspace_graph` 5。`knowledge_workspace_read_markdown` / `knowledge_workspace_search` / `knowledge_workspace_read_canvas` 本轮**一次都没被调用**（02 打开了 Canvas 面板但未触发读取）。allowlist 外命中 0。

五类零值：**write 0 / unknown 0 / 外部请求 0 / console error 0 / page error 0**，8/8 context。各层（documentElement / body / shell / 中央面 / 活动组面板 / overlay）横向 overflow 全为 0。

### 5.4 截图

`raw/` 下 §7 规定的六张：`01-1440-activity-ribbon.png`、`02-900-activity-ribbon-narrow.png`、`03-1440-right-context-sections.png`、`04-900-right-context-collapsed.png`、`05-1440-graph-n40.png`、`06-1440-graph-n512.png`；红图五张 `red-0*.png` 保留。

两张 Graph 规模图**拍在 fitView 之后、放大之前**——它们要佐证的正是俯瞰口径下的"全在舞台内 + 零相交 + 标题整体隐藏"。首轮曾误在放大之后截图（图上标题可读、只见约 50 个节点），与其要支撑的结论不符，已改在放大前截图并重跑整个矩阵；放大后的状态另存为补图 `05b-1440-graph-n40-readable-zoom.png` 与 `06b-1440-graph-n512-readable-zoom.png`，用来佐证达标缩放下节点可读且仍零相交。两张补图是本包额外提供，不替代 §7 的六张。

逐图只对照 R0 `01/03/04` 的对应维度与 R2 `01/06/07/08` 的对应差距项。**不声称完整 R0 通过、不声称"Obsidian 1:1"、不声称整个知识 UI 或真实 App 通过。** 活动栏与右栏只解掉 R2 P2 记的这两项差距；Graph 只解掉 R3D 记账的规模化债。

## 6. §5.4 回归锁

四个冻结 green runner 逐字节拷到仓外临时目录运行（**未在原目录运行、未覆盖任何上游 evidence**）。四个副本 hash 与 §3.2 冻结值逐个相符。逐 assertion 结果：`raw/regression-per-assertion-summary.json`，原始 JSON：`raw/regression-r3b.json`、`regression-r3c.json`、`regression-r3c-r1.json`、`regression-r3d-with-r3e.json`。

| runner | 结果 | 断言 | 失败 |
| --- | --- | --- | --- |
| R3B tab-groups-split | PASS | 90 | 0 |
| R3C canvas-first | PASS | 131 | 0 |
| R3C-R1 focus-return | PASS | 73 | 0 |
| R3D graph-convergence（首轮，未修 runner） | NEEDS_R3D_REWORK | 75 | **1** |
| R3D graph-convergence（修掉竞态取样后） | **PASS_SYNTHETIC_GRAPH_EVIDENCE** | 75 | **0** |

首轮合计 369 断言 / 1 失败；按用户明确指示修掉那处竞态取样后 **369 / 0**。所有与隔离、command allowlist、写/未知/外部/console/page error 零值、焦点/回焦、overflow、标签组/草稿/Canvas 行为相关的断言**全部仍绿**——包括 R3C-R1 那 73 条专门锁回焦的断言。

### 6.1 唯一失败项的逐条归因

- 断言：`06-900-reduced-motion-collapsed: reduced-motion Escape still returns to the actual opener`，失败值 `returnedFocus: false`。
- 它**不是**"编码了旧文字活动栏或旧平铺右栏呈现"的断言，所以不落在包里允许失败的范围。执行线不含糊带过。
- **A/B 归因**：把 `KnowledgeGraphView.tsx`、`NativeKnowledgeWorkspace.tsx`、`styles.css` 三个文件逐字节换回 §3.1 基线副本（hash 复核为派发值 `c9151ff…` / `eafd9ed…` / `fcb2892…`）后重跑同一 runner，**同一条断言同样失败**（`raw/regression-r3d-on-dispatch-baseline.json`）。随后逐字节还原本线版本并复核 hash。→ **该失败在未改动的派发基线上就已存在，不是 R3E 引入的回归。**
- **行为本身没坏**：`raw/regression-r3d-focus-return-probe.{mjs,txt}` 复现同一操作序列并逐帧采样——Escape 后 `0 ms` 时 activeElement 是 `BODY`，`30 ms` 起就是那枚"筛选"按钮，且 opener DOM 节点自始至终是同一个（`storedIsLive=true`、从未被替换）。焦点确实回到真实 opener，只是落在下一个动画帧。
- **成因**：`KnowledgeGraphView` 的回焦刻意放在 `requestAnimationFrame` 里（R3C-R1 的设计），而冻结 runner 在 `waitFor({state:"detached"})` 返回后**立刻**取样。两者都挂在 rAF 上，谁先跑取决于当帧调度——R3D 验收时赢了这场竞态，现在输了。
- **首轮上交时执行线没有为了让它变绿而动任何东西**：改冻结 runner 是 §10 停止条件；改回焦机制超出 §4.1.3 给 `KnowledgeGraphView.tsx` 的授权。故先如实上交。

### 6.2 用户拍板后的修复（超出 §4.3 白名单，经用户明确授权）

用户看过归因后明确指示"直接修吧"。修的是**测试取样时机，不是产品代码**——产品源码三个文件的 SHA-256 在修复前后完全未变（`821a49d7…` / `f24235ad…` / `bb814766…`）。

先看清再动手：扫描四个 runner 的 `waitFor({state:"detached"})` 之后如何取样，发现这不是设计分歧而是同一文件的一处漏写——

| 位置 | detached 之后做了什么 | 结果 |
| --- | --- | --- |
| R3C-R1 `browser-evidence.mjs:183` | `waitForFunction(activeElement === opener)` 等条件 | 稳定通过 |
| R3D 同文件 `:533` | `waitForTimeout(40)` | 通过 |
| R3D 同文件 `:543` | `waitForTimeout(40)` | 通过 |
| **R3D 同文件 `:821`** | **两样都没有，同一拍直接取 `activeElement`** | **翻面** |

修法：把该处改成**有界轮询**（最多 2000 ms，逐帧检查 `activeElement === opener`），而不是照抄兄弟处的固定 40 ms 等待——固定等待只是把竞态窗口拉宽。用轮询而不用 `waitForFunction`，是为了真回不去时仍然干净地报 `returnedFocus: false`，而不是抛超时异常、丢掉断言语义。

**修完做了反向红测，证明断言没被改软**：临时把产品侧 `restoreFilterOpener` 打断（`if (1) return;`）后重跑，修过的这条**照样失败**，并且连同该文件另外 3 条回焦断言一起被抓（`4` 条失败）；随后逐字节还原产品代码并复核 hash 回到 `821a49d7…`。

- 原件保全：R3D 验收原件已逐字节留档为 `raw/regression-r3d-green-browser-evidence-as-accepted.mjs`，hash 仍是 `1cee08e8…`，未丢失、可复核。
- **`evidence/raw/2026-07-26-l3-syn-n2r-r3d-graph-convergence/green-browser-evidence.mjs` 的 hash 已变更**：`1cee08e83b44f9f6a6e6d4ea41c1ff005591e79528bb4fa64b2440df54bb9ee1` → **`bb24c2c7482149bf1fac6a0e40600fa0b6e9bae185b0639392f7b80b377e4fb6`**。后续任务包的冻结表需按新值更新。
- 修后重跑：`PASS_SYNTHETIC_GRAPH_EVIDENCE`，**75 断言 / 0 失败**（`raw/regression-r3d-with-r3e.json`）。
- **未修的遗留项（留给指导线定夺）**：同文件 `:533`、`:543` 两处仍是固定 `waitForTimeout(40)`。它们当前通过，但属于同一类潜在竞态，只是窗口更宽。本次按用户指示只修失败那条，没有顺手扩到通过的断言上。

## 7. §8 必跑验证

| # | 项 | 结果 |
| --- | --- | --- |
| 1 | 三合同聚焦 | `knowledge workbench shell static convergence contract passed; R3B 16 / 0; R3E 16 / 0`（R3B 既有 16 条零回归，新增 R3E 16 条）；`native knowledge workspace … contract tests passed`（新增 14 条运行时断言：6 个 SSR 入口 ×2 + svg 计数 + `大纲` 零命中）；`native knowledge graph … tests passed: 76`（新增 D3 规模不变量，既有断言全部保留；其中 6 节点坐标断言按新布局更新为 `162/302/22` 一组） |
| 2 | Canvas 合同不回归 | `native knowledge canvas … R3C 14 / 0, and R3C-R1 4 / 0 contracts passed` |
| 3 | `npm run typecheck` | 通过，零输出 |
| 4 | `npm run test:offline-interaction` | 冻结 runner 精确 **37** 入口全过 |
| 5 | `wc -l` | 1932 / 144 / 172，全部 < 2000（见 §2.1） |
| 6 | shape gate `--mode baseline` | `error 17 / warn 5 / info 5` |
| 7 | shape gate `--mode check` | `error 17 / warn 5 / info 5`——与 baseline **逐条 finding 完全相同**（集合差为空），**零新增类别、零新增 finding**；特别是**没有新增 `file_over_limit_not_in_ratchet`** |
| 8 | `workbench-shape-gate.hardcoded-hex.selftest.js` | `13/13 checks passed` |
| 9 | `workbench-shape-gate.machine-face.selftest.js` | `18/18 checks passed` |
| 10 | `git diff --check` | 干净，零输出 |
| 11 | `git diff --cached --name-only` | 空 |
| 12 | 冻结只读 hash 回算 | 18 个 MATCH；**1 个经用户授权变更**：R3D green runner `1cee08e8…` → `bb24c2c7…`（§6.2，原件已留档）。窄写文件相对 §3.1 基线的 diff 摘要见 §2 与 `raw/baseline-narrow-write-diff.patch` |
| 13 | 端口 | 本包启动的 Vite 已关闭，`5173` 无 listener |

shape gate 的 `check` 状态是 `fail`，但那 17 个 error 全部是既有存量（`.rs` 超限、sidecar kind、ratchet 文件等），与 baseline 模式逐条相同。`styles.css` 的 `ratchet_file_increased` 是既有项：waterline `8464`，本包前 `12419`、本包后 `12496`——同一条 finding，只是 delta 从 3955 变成 4032，不是新增 finding。

### 7.1 一处环境说明

`.claude/launch.json` 里没有 `5173` 的 Vite 配置，而该文件不在 §4.3 白名单内，所以本包的 Vite 由 `npm run dev` 直接起、取证后按 §8.13 关闭。除此之外没有起任何服务，没有碰 Syn/Tauri/Obsidian/Codex CLI/MCP，没有读真实 store/vault。

### 7.2 一次白名单外的瞬时写入（主动报账）

为核对基线合同的断言基数，执行线曾把基线副本的 `knowledge-graph.test.tsx` 临时复制成 `prototypes/productized-desktop-shell/tests/.r3e-baseline-graph-probe.tsx` 跑了一次，随即删除。`tests/` 目录不在 §4.3 的新增白名单内，这是一次白名单外的瞬时写入，主动记账。该文件已删除，`git status` 无残留，仓内净改动为零；核对目的改用静态计数达成。

## 8. 新 catch

**一条新 catch**，已追加到 `docs/harness-catch-log.md`：R3D 冻结 green runner 的 `reduced-motion Escape still returns to the actual opener` 是时序竞态断言，在**未改动的派发基线**上同样失败，因此 R3D 原来的 `75/0` 不是可复现的回归锁。经用户授权已修（§6.2），修后 `75/0` 且反向红测证明断言仍然吃紧。

## 9. 未完成 / 明确不做

- 没有实现任何大纲派生（用户 §0.1 拍板删字）。
- 规模场景的生成节点不激活、不断言可读性。
- 没有进入 R4 最终视觉对照、N6、真实 App/store/vault。
- 没有回写 `CURRENT.md`（不在 §4.3 白名单，且本包禁 stage/commit）；请指导线收口时处理。
- R3D 同文件 `:533`、`:543` 两处固定 `waitForTimeout(40)` 未动——同类潜在竞态、当前通过，按用户指示只修了失败那条，留给指导线定夺是否一并收紧。

## 10. 指导线独立复核与收尾（2026-07-26 · guidance）

结论：**`ACCEPTED_R3E_D1_ACTIVITY_RIBBON / ACCEPTED_R3E_D2_RIGHT_CONTEXT / ACCEPTED_R3E_D3_GRAPH_SCALE`（synthetic 范围）/ `NOT_REAL_APP_ACCEPTED`**。零误报：回传的每个数字都被独立复现或独立重算。

### 10.1 指导线亲自重跑 / 重算（不采信回传数字）

| 项 | 指导线独立结果 |
| --- | --- |
| R3E green 矩阵 | 把 runner 拷到仓外自跑：`GREEN_ALL_ASSERTIONS_PASSED`、8 context / `126` 断言 / `0` 失败；与执行线 JSON **深度 diff 零处差异**（逐 bounds / computed style / ARIA 串 / command 计数） |
| D3 纯函数不变量 | **不复刻算法**：自写探针 bundle 产品源码导出的 `deterministicKnowledgeGraphPosition` 等，`n = 1…512` 全量扫描——低于合同下界 `160` 的 `0` 个；另按更严的**节点矩形真重叠**判据（\|dx\|<136 且 \|dy\|<40）算，重叠对数 `0`；确定性复算全等。全场最紧为 `n = 478 → 161.01`（回传未列，仍达标）。抽样与 §5.1 表逐值相符（`512` → 13 环 / 半径 `2106` / minZoom `0.0847`） |
| 基线副本可信度 | `shasum -c baseline-manifest.txt` 7/7 OK，且 7 个值与派发表逐个相符 → 副本确实取自改动前；据此指导线自行 diff 全部 7 个窄写文件，hunk 数与落点与 §2 相符（+/- 行数回传略高，因其把新增空行计入） |
| CSS 范围 | 自读 diff：只动 `.syn-knowledge-shell__activity button` 与新增 `.native-activity-icon` / `.native-context-*` / `.native-graph-flow-stage.is-compact-zoom`——全在 §4.2 白名单内；`:root`/全局 token/左栏/中央标签组/Canvas/overlay/状态栏零改动 |
| 删字 | 全仓 `大纲` 命中 1 处（`KnowledgeContextSidebar.tsx:14` 负面声明注释），`outline*` 标识符 0；渲染面 0 |
| 门禁 | typecheck 0；离线 runner **exit 0**、37 入口（shell `R3B 16/0 + R3E 16/0`、graph `76`、canvas `R3C 14/0 + R3C-R1 4/0`）；shape baseline `pass 17/5/5`、check `fail 17/5/5`，指导线自做 finding 集合差 = **空**；hex 13/13、machine-face 18/18；`git diff --check` 干净；staged 空；行数 `1932 / 144 / 172` |
| 上游 evidence 未被覆盖 | R3B / R3C / R3C-R1 三目录全部文件 mtime 仍为 `05:35 / 14:09 / 15:19`（R3E 之前）；R3D 四张 PNG hash 未变；R3D 的 `green-browser-evidence.json` 与指导线在 R3D 复核时留存的副本**逐字节相同** |

### 10.2 竞态断言：指导线独立复核（回传属实）

- 自跑 as-accepted 原版 runner **两次**，均在同一条断言失败；自跑修后版本 `75/0`。
- 决定性证据（不需 A/B 改树）：`restoreFilterOpener` / `closeFilters` / `openFilters` 在**派发基线与收口版之间逐字节相同**，且 R3E 的 5 个 hunk 无一处含 `requestAnimationFrame` / `restoreFilterOpener` / `opener` → 该断言测的机制未被 R3E 触碰，失败不可能是 R3E 引入的行为回归。
- 指导线自写逐帧探针（900×760 / reduce）：面板 detached 后 `0 ms` 时 activeElement 是 `BODY`，`8 ms` 起即为**与原 opener 同一个 DOM 节点**并稳定保持，`openerStillSameDomNode = true` → 回焦确实发生，原 runner 早取样一帧。
- 修法审查：有界轮询在真回不去时仍 `resolve(false)`，按构造仍能失败；配合执行线的反向红测（打断 `restoreFilterOpener` 后该条连同另 3 条一起失败），判定**尺子未放松**。

### 10.3 指导线的两处收尾写入（用户 2026-07-26 指示"收掉"）

1. **恢复已验收目录的不可变性**：`evidence/raw/2026-07-26-l3-syn-n2r-r3d-graph-convergence/green-browser-evidence.mjs` 已从执行线留档的原件复原，hash 回到验收值 `1cee08e83b44f9f6…`。修好的版本改为 **R3E 自有工具**：
   - `raw/regression-r3d-green-browser-evidence-r3e-fixed.mjs` = `c956dfd8550d0e27ef9b4d341893a4c8a7108ab26ca9b4587bda568446129567`
   - 该版把本文件**全部 3 处**"detached 之后立刻取 activeElement"改成同一套有界轮询，并删掉 `:533`/`:543` 两处固定 `waitForTimeout(40)`（§9 遗留项一并收掉）。
   - 指导线自跑结果：`PASS_SYNTHETIC_GRAPH_EVIDENCE`、`75 / 0`，原始 JSON 存为 `raw/regression-r3d-with-r3e-guidance-fixed.json`。
   - 后续任务包的冻结表：R3D 目录仍登记 `1cee08e8…`（验收原件），回归工具改登记 R3E 目录的 `c956dfd8…`。
2. **活动栏悬停提示：试过，撤回，转为下一段的决定项**。指导线一度给 8 个按钮加上与 `aria-label` 同值的 `title`（附加提示，非替代），但离线合同 `R3E D1 图标必须是纯装饰：aria-hidden + focusable=false，且不得用 title 提示替代可访问名称` 把"不得替代"实现成了"整条禁止 title"，该改动使合同 `16/1`。这属于**规则改动**（要动派发包 §1.1 拒绝项 5 与对应合同），不该由复核方顺手塞进来，故已**逐字节还原**（`KnowledgeActivityRail.tsx` hash 回到 `3f1c20b9…`，离线 runner 复绿 exit 0）。留作下一段的用户决定项。

收尾后实测：五个产品文件 hash 与执行线交付值逐个相同（`3f1c20b9… / 0184c075… / f24235ad… / 821a49d7… / bb814766…`），HEAD `e9ad7f3` 未漂移，staged 空，`5173` 无 listener，仓内无本次复核残留。

### 10.4 指导线记录的遗留观察（非返工项）

- **图标条无可见文字、无悬停提示**：读屏软件走 `aria-label` 无损，但用鼠标的人只看到 8 个抽象图标；R0 参照的活动栏是有悬停提示的。见 §10.3 第 2 条，等用户拍。
- **同心环不表达团块结构**：512 节点呈"洋葱"，不呈现社群/邻域。这是用户已拍板的"确定性优先 + 大图只作俯瞰"的代价，记录一笔留给 R4 视觉对照时权衡，不属本包缺陷。
- **R3D 已验收的"确定性坐标"数值被 D3 取代**（6 节点坐标改为 `162/302/22` 一组）。这是 D3 的必然后果，合同已同步更新；引用旧坐标的文档按本节口径读。
