# 任务包：L3 Syn N2R-R3E 活动栏 ribbon + 右栏层级 + Graph 布局规模化（合并包）v1

- 日期：2026-07-26
- 状态：**已授权派发（DISPATCHED · 2026-07-26 用户明确授权）**
- 负责人：独立知识前端执行线
- 指导/验收：当前总指导对话
- 上游接受：`ACCEPTED_N2R_R3D_GRAPH_CONVERGENCE / NOT_REAL_APP_ACCEPTED`（见计划 §5 R3D 段、`docs/harness-catch-log.md` 07-26 两行）
- 目标 evidence：`evidence/2026-07-26-l3-syn-n2r-r3e-activity-rail-right-context-and-graph-scale-verification-v1.md`

## 0. Kickoff 与合并风险

用户明确要求把剩余三项合并为一个包。本包一次关闭三个维度：

- **D1**：R2 P2 差距——活动栏是可断行中文文字按钮，R0 是紧凑 icon ribbon；`900` 宽度下断行、可扫读性下降。
- **D2**：R2 P2 差距——右栏是长列表式上下文，缺 R0 的分区/折叠层级与信息节奏。
- **D3**：R3D 记账债——确定性环形布局的半轴恒为 `110/160`、与节点数无关，节点盒固定 `136×40`，椭圆周长约 `855 px`；6 节点间距约 `143 px` 刚好排满，12 节点起互压，后端 `MAX_GRAPH_NODES = 512` 时必然堆叠。冻结夹具正好 6 节点，R3D 的 `75/0` 恰停在临界点。

**合并的已知代价（指导线已向用户明示）**：R3C 曾因单一回焦缺口整包退回。三维度同包时，任一维度不过就会牵连另两维度的复核结论。为把返工范围压到最小，本包强制：三个维度各自独立的 red/green evidence 段、独立量尺、独立结论枚举（§9），指导线按维度分别裁决；被拒维度单独窄返工，已通过维度的证据保留。

本包不进入真实 App、真实 store/vault、N6、R4 最终视觉对照，不解除"禁随机/力导布局"的既有禁令，不新增依赖。

### 0.1 派发前用户拍板的两处收紧（2026-07-26 · 覆盖 v1 草稿）

派发前指导线复核发现草稿两处未定死，均已由用户拍板，执行线**不得再自选**：

- **D3「看得见」口径 = 俯瞰图**。草稿 §5.3 同时要求"fitView 后全部节点 bounds 在舞台内"与"节点仍有可辨 hit target"，在 `n = 512` 上不可能同时为真。指导线按冻结常数（节点盒 `136×40`）算术推导：单环放大半轴需 `R ≈ 13,000 px`，`1440×900`（舞台约 `1000×700`）下 fitView 缩放 ≈ `0.027`，节点渲染 ≈ `3.6 × 1.1 px`；同心环（环距 `200`、每节点弧长 `160`）需 11 环、外径 ≈ `4,540 × 4,440`，fitView 缩放 ≈ `0.16`，节点渲染 ≈ `21 × 6 px`——均低于本包 §5.1 自定的 `28×28` hit target。**该两组数字是算术推导，不是浏览器实测，执行线必须以实测值覆盖，不得直接引用。** 用户裁定：大 `n` 的 fitView 只作俯瞰，明写不可点、不可读；可点性与标题只在放大后要求。§5.3 已按此重写。
- **D2「大纲」= 删字**。草稿把"真做/删字"交给执行线自选，属于用户可见功能取舍，不该由执行线拍。用户裁定：**删字**——把 `大纲` 从右栏标题去掉，evidence 明写"本阶段不提供大纲"，不做任何大纲派生实现。§1.2、§5.2、§11 已按此收紧。

指导线派发前实核（非照抄）：HEAD `e9ad7f3` 一致、staged 空、`5173` 无 listener、近 30 分钟仓内零写入且无并行 writer；§3.2 中 12 个关键文件 hash 逐个重算全部相符；两个待新增组件文件与目标 evidence 均不存在；`NativeKnowledgeWorkspace.tsx` 实测 `1978` 行；`大纲` 全仓命中仅 [`NativeKnowledgeWorkspace.tsx:1694`] 一处标题字符串、`outline*` 标识符零命中（空承诺属实）；`deterministicKnowledgeGraphPosition`（`KnowledgeGraphView.tsx:521`）半轴确为写死 `110/160`、与 `total` 无关。

## 1. 设计意图与拒绝项

### 1.1 D1 活动栏 ribbon

- Intent：一列窄图标，一眼扫过就知道去哪；不是一列会折行的中文按钮。
- Density：`42 px` 量级窄栏，图标 + 稳定 hit target；当前视图/面板状态靠既有 accent 与 `aria-pressed` 表达。
- Signature：图标由本仓自绘的极简几何 inline SVG，`currentColor` 取既有 token；保留 Syn 暖纸/墨色/青玉。

拒绝项：

1. 复制或改写 Obsidian（或任何第三方）图标、字体图标包、品牌资产 → 只允许本仓自绘几何路径；
2. 新增图标库依赖或外部资源请求 → `package.json`/`package-lock.json` 冻结，零外部请求；
3. 用图标换掉可访问名称 → 八个入口的 accessible name 必须逐字保留：`文件`、`搜索`、`关系图`、`Canvas`、`Syn 命令`、`设置与维护`、`来源`、`切换右侧上下文`；
4. 为省事去掉 `aria-pressed`/`is-active` 语义或改动 dispatch/overlay 行为 → 行为零变化，只改呈现；
5. 用 `title` 提示替代可访问名称，或让图标成为不可聚焦装饰。

### 1.2 D2 右栏层级

- Intent：右栏是"当前选中内容的上下文"，按区块分层、可折叠、有节奏；不是一路平铺到底的长列表。
- Hierarchy：每区一个稳定标题 + 可折叠内容；默认展开集合最小充分，不靠颜色表达开合。
- 诚实性：当前右栏标题声明 `属性 / 反链 / 大纲 / 来源上下文`，但全仓 `大纲` 只作为该字符串出现一次，功能不存在。

**用户已拍板（§0.1）：删字，不得再二选一**：

- 把 `大纲` 从右栏标题字符串中去掉（当前唯一命中 `NativeKnowledgeWorkspace.tsx:1694` 的 `属性 / 反链 / 大纲 / 来源上下文`），标题只声明本阶段真实存在的区块；
- evidence 明写"本阶段不提供大纲"；
- **不得**顺手实现任何大纲派生（含只读标题列表）——那属于本包范围外的新增能力，出现即按 §10 报 `BLOCKED_N2R_R3E_SCOPE_EXPANSION`。

拒绝项：

1. 改注入方 → `src/views/KnowledgeBaseView.tsx` 与夹具 `SyntheticSourceContext` 均冻结只读；来源上下文的层级必须由**产品侧包一层可折叠容器**实现，并在未修改的注入内容上生效；
2. 折叠状态写进真实 store/vault/Markdown → 只允许既有可丢弃 UI chrome 偏好键 `syn-native-knowledge-workspace-ui-v1`，或完全不持久化；不得新增持久化键；
3. 吞掉 loading/空态/冲突/`未选择文件` 语义 → 可压低层级，不得消失；
4. 借层级重做把右栏做成第二主面板，或改动左栏、活动栏（D1 除外）、中央标签组、overlay。

### 1.3 D3 Graph 布局规模化

- Intent：节点数增长时关系舞台仍可读；不出现节点互相压叠。
- 既有禁令**不解除**：继续使用 `@xyflow/react`，布局必须是"固定输入 → 固定坐标"的确定性函数；禁止随机、力导、坐标持久化和第二图框架。
- 允许的方向：确定性且随节点数增长的布局（例如按节点数放大半轴、或容量分层的同心环），容量由 `节点盒宽 + 间距` 推出。
- 缩放层级：允许标题在低缩放下整体隐藏（与 R0 参照一致），但**可访问名称（`aria-label`）在任何缩放下都必须完整**——它是 DOM 属性，与缩放无关；不得出现被缩到不可读的标题糊块。
- **俯瞰口径（用户 §0.1 拍板）**：大 `n` 的 fitView 只保证"布局正确、零相交、全在舞台内"，**不保证可点、可读**。可点与标题可见性只在放大到达标缩放后要求。执行线不得为了"让 512 也能点"而改布局密度、隐藏节点或裁切舞台。

拒绝项：

1. 改 fixture 既有场景/数据来制造好看的图 → 只允许**新增**一个规模场景（§4.1.5）；
2. 用隐藏节点、隐藏边、裁切舞台或强行缩到不可读来"解决"堆叠；
3. 把 node id 当路径、改 payload、改 `MAX_GRAPH_NODES` 或任何 Rust；
4. 只交纯函数断言而不给浏览器规模实测，或只给浏览器图而不给纯函数不变量。

项目界面模式正本：`.interface-design/system.md`。实现必须使用其中已接受的实际 opener、overlay 和折叠语义，不得另造冲突模式。

## 2. Authority、能力和并发边界

- authority_chain：`AGENTS.md` → `CURRENT.md` → native knowledge route v2 → native knowledge small-stage plan v2 → R0 → R2 gap matrix → R3B/R3C/R3C-R1/R3D accepted evidence → 本包。
- plan_anchor：`docs/plans/2026-07-23-l3-syn-native-knowledge-workspace-small-stage-plan-v2.md#r3-既有能力交互与视觉收敛`
- existing_before_new：活动栏八入口、左右栏折叠 reducer、`aria-pressed`/`is-active`、右栏 `WorkspaceMetadata`/反链、`sourceContext` render prop、Graph typed client、`@xyflow/react`、`deterministicKnowledgeGraphPosition`、fitView/zoom controls、可丢弃 UI 偏好键均已存在；本包优先收敛呈现与布局函数，不新造能力。
- capabilities_touched：none；不得修改 command 名、payload、vault root、Graph/Markdown/Canvas 后端、写路或 capability registry。
- data_truth：右栏与活动栏只呈现既有只读投影；Graph 继续只读已验证 wikilink/反链投影，布局与选择不得写回 Markdown、vault、store。

本包独占（其他 UI 包在 R3E 结束前不得并发写）：

- `NativeKnowledgeWorkspace.tsx`、两个新增组件文件、`KnowledgeGraphView.tsx`；
- `styles.css` 中 §4.2 列明的 selector 族；
- `knowledge-workbench-shell.test.tsx`、`native-knowledge-workspace.test.tsx`、`knowledge-graph.test.tsx`；
- `knowledge-workbench-visual-fixture.tsx` 的**新增**规模场景。

对话底座线在本包产品写入、Vite/浏览器取证期间只可静态只读准备，不得构建、启动或执行三句真实 App 重验，不得占用同一真实 store 或 Rust 运行资源。

## 3. 冻结基线与开工停点

- HEAD：`e9ad7f3a204a1ebb11ce26c1e8c05b19c04c0991`
- 派发时 staged：空
- 派发时 `127.0.0.1:5173`：无 listener
- 工作树存在大量既有 WIP；目标文件均按下表逐文件冻结。不得用 clean tree 假设覆盖现状。

### 3.1 新增强制项：基线副本（源自 07-26 catch ②）

开工第一步，把下列**全部窄写目标**在改动前逐字节复制到

`evidence/raw/2026-07-26-l3-syn-n2r-r3e-activity-rail-right-context-and-graph-scale/baseline/`

并在同目录写 `baseline-manifest.txt`（每行 `<SHA-256>  <仓内相对路径>`，与 §3.2 派发 hash 逐个相符）。**没有基线副本不得开始产品写入**：R3A–R3D 只冻 hash，导致 R3D 收口时指导线在物理上无法核"只改了白名单 hunk"（派发 hash 对应内容在磁盘、git 对象与 stash 中均不存在）。收口时必须给出每个窄写文件相对该副本的 `diff` 摘要（改动行数 + 每个 hunk 的 selector/函数名）。

### 3.2 冻结表

| 文件 | 派发 SHA-256 | 本包权限 |
| --- | --- | --- |
| `src/views/knowledge/NativeKnowledgeWorkspace.tsx` | `eafd9edccc3a2cd62e99cfa211809d77b206c826907ca07976ef4c419f4b4464` | 窄写（含强制抽出，见 §4.1） |
| `src/views/knowledge/KnowledgeGraphView.tsx` | `c9151ffaf0ba406956ea2fd3ef40ff8d737d684578ecb263469eacea668dd781` | 窄写（仅 D3） |
| `src/styles.css` | `fcb28929d693b9f41721a51d201b9744f5db8d439dbfd694b4381c88c59d396a` | 只写 §4.2 精确 selector 族 |
| `tests/knowledge-workbench-shell.test.tsx` | `2cbf2b61a86731096a876636c27119066c0e10d5bfaac81e8c6c7322660541da` | 窄写（合同） |
| `tests/native-knowledge-workspace.test.tsx` | `fdcdfe54fb8f6dd022d4b018635d3bca1f2cd75874f2a835859dacfaa406408e` | 窄写（合同） |
| `tests/knowledge-graph.test.tsx` | `a38f4359169ab363aaca0082c908854809bd4b62978b7e1f3e51be7f343e97f9` | 窄写（合同） |
| `tests/knowledge-workbench-visual-fixture.tsx` | `ea27e80ec9fff83fc630e4b80074915a39ee1ecf2f5217c3e3940bb51eb929ee` | **仅新增**规模场景（§4.1.5） |
| `src/views/knowledge/KnowledgeWorkbenchShell.tsx` | `2ab66a5b00941e3cf0a083e3ed3d24b309f8b7fe5037dd41d8743cf9d2d41c21` | 冻结只读 |
| `src/views/KnowledgeBaseView.tsx` | `3fa18f9fbba0f6c797cc568132389ac2c02a0e12c7dc4861a27ab4f6c2309e58` | 冻结只读（注入方） |
| `src/views/knowledge/KnowledgeCanvasView.tsx` | `42054605e642100752a109830a8f7601a46eaef68f1bc70c7ca48e5353d6405e` | 冻结只读 |
| `src/views/knowledge/KnowledgeWorkspaceMaintenancePanel.tsx` | `d7cc6f5b56b1b7731566050556e469035a57d1048bc72deca3f7ef396b184c94` | 冻结只读 |
| `src/lib/knowledgeWorkbenchLayout.ts` | `8fb9faf01ef9dae509a6cffa4e14fd62031bb9744ea8e44eb1d1476e4f9ee220` | 冻结只读 |
| `src/lib/knowledgeWorkspace.ts` | `81eddc02206035a9d1bf1e3954d5ea19b1864a71b27ffea43efafa5a2fd51fb0` | 冻结只读 |
| `src/lib/tauri.ts` | `95587bdd68c7e207e18d6ecdc2c862a260706c9aa7f5c3085b7dcf95d8dc14ee` | 冻结只读 |
| `tests/knowledge-canvas.test.tsx` | `802dd4a90ceac4f491ddbf390f8e5734f02984af87c920c170b17d0c723f263c` | 冻结只读 |
| `tests/knowledge-workbench-visual-fixture.html` | `8a58ac0d02d9c33a9ca151f4ec1e653ccd3c6d79096581efd339a62c1fcec23c` | 冻结只读 |
| `scripts/run-offline-interaction-test.mjs` | `798daf13722875befa22eb233c476b957d241005a3b08267ed30eb54e0363082` | 冻结只读（仍 37 入口） |
| `package.json` | `08a3abc466e2dd51f946380badb6519b4273353d8d279efa43b1cf6086d87e65` | 冻结只读 |
| `package-lock.json` | `781cb1d94eaaeec0d071f156f9aeca65400acce8bd811c9c95cd4d45cf700bc0` | 冻结只读 |
| `.interface-design/system.md` | `2a035b99b0e21b33fb4af259aad0f12515406fede89a9395b9fa70ba868bd766` | 冻结只读 |
| R0 设计参考 `docs/design/2026-07-25-l3-obsidian-core-desktop-reference-and-migration-r0-v1.md` | `141ff343312fff91346e6e170f1aea2d07cda5fa2abdb3fde8b840b16f9ed4d9` | 冻结只读 |
| R2 基线 evidence | `8dd432266f406e96c69359196ea138eede85891d041ed8f98a3907eed55b6c7a` | 冻结只读 |
| R3D task | `baf5f5b3725a473203c983ce810ddd3e31fa06b61391cdf7ef8387e52c636602` | 冻结只读 |
| R3D evidence | `5d9e17d6703c8ed7a2d25f97f1ad671aad9f4ac16a0eed0eae266ed1a904aebe` | 冻结只读 |
| R3B green runner `evidence/raw/…-r3b-tab-groups-split/browser-evidence.mjs` | `087653734143ef34edda722b8b097f02cd0f8ca64111f688a21c8a255dc0e736` | 冻结只读（回归工具） |
| R3C green runner `evidence/raw/…-r3c-canvas-first/browser-evidence.mjs` | `c02997a36c6a6cc6bc8b837d7f03c41f8533eb726735f0e0788f17b86b984468` | 冻结只读（回归工具） |
| R3C-R1 green runner `evidence/raw/…-r3c-r1-focus-return/browser-evidence.mjs` | `1dead02d262c6ebd682e6d491e264647fd9ec77cd2caef2450e3bf923dc77f5f` | 冻结只读（回归工具） |
| R3D green runner `evidence/raw/…-r3d-graph-convergence/green-browser-evidence.mjs` | `1cee08e83b44f9f6a6e6d4ea41c1ff005591e79528bb4fa64b2440df54bb9ee1` | 冻结只读（回归工具） |

开工前逐一复核 HEAD、staged、全部冻结 hash、5173、窄写文件的 provenance 与唯一写所有权，并完成 §3.1 基线副本。任一目标/冻结 hash 漂移、staged 非空、端口占用或并行 writer 存在，立即按 §10 停止；不得 reset、clean、stash、checkout、覆盖或合并他人 WIP。

## 4. 精确写入白名单

### 4.1 产品与合同

1. **强制抽出（不是可选）**：`NativeKnowledgeWorkspace.tsx` 当前 `1978` 行，`.tsx` 硬限 `2000` 且该文件不在 shape 棘轮名单里，只剩 `22` 行余量。D1/D2 必须抽成两个新文件，抽出后该文件行数应**下降**：
   - `src/views/knowledge/KnowledgeActivityRail.tsx`（新增，D1；接收既有 view/tab/collapse 状态与 dispatch，不自持业务状态）
   - `src/views/knowledge/KnowledgeContextSidebar.tsx`（新增，D2；含可折叠区块容器与包裹注入 `sourceContext` 的产品侧容器）
   - 两个新文件各自 `< 2000` 行；不得新增第三个视图容器或第二套壳。
2. `NativeKnowledgeWorkspace.tsx`：只允许把活动栏与右栏的呈现替换为对上述组件的调用、传入既有状态/回调，以及 §1.2 的标题删字。**不得新增任何大纲派生逻辑**。reducer、草稿/保存写路、overlay、搜索、标签组、状态栏行为零变化。
3. `KnowledgeGraphView.tsx`：只允许改布局函数与节点标题的缩放可见性；`loadKnowledgeGraph`、`openKnowledgeGraphNode`、`nextKnowledgeGraphOpenRequest`、accessible label 生成、typed client 调用面与 `onOpenMarkdown(relativePath)` 语义不得变。
4. 合同测试三份：新增 D1 accessible-name/语义、D2 区块/折叠/诚实性、D3 布局不变量断言；静态合同不能替代 §7 浏览器证据。
5. `knowledge-workbench-visual-fixture.tsx`：**只允许新增**一个受控规模场景——按既有 `document.documentElement.dataset.fixtureScenario` 钩子，在规模场景下让 `knowledge_workspace_graph` 返回生成的 `N` 节点投影（`N ∈ {40, 512}`）。既有 `syntheticGraph`、既有场景、其余 mock 分支、允许 command 列表与拒绝语义逐字不变；生成节点不需要可读 Markdown（规模场景只量布局，不激活节点，必须在 evidence 明写这一限制）。

### 4.2 styles.css 允许的 selector 族

- D1：`.syn-knowledge-shell__activity` 及其子选择器、新增 `.native-activity-*`；
- D2：`.syn-knowledge-sidebar-tabs--right`、`.native-workspace-margin`、新增 `.native-context-*`；如需处理 `.syn-knowledge-shell__right` 的内部排布，必须拆成右栏专属 rule；
- D3：`.native-graph-node*`、`.native-graph-flow-stage` 内 Graph 专属组合 selector；
- 禁止改全局 token、`:root`、左栏、中央标签组、Canvas、maintenance、overlay、状态栏与非本包 media/container rule；禁止新增 hardcoded hex、gradient、backdrop blur、装饰性 shadow 或第二套字体/间距 token。

### 4.3 新 evidence

- `evidence/raw/2026-07-26-l3-syn-n2r-r3e-activity-rail-right-context-and-graph-scale/**`（含 §3.1 baseline 子目录）
- `evidence/2026-07-26-l3-syn-n2r-r3e-activity-rail-right-context-and-graph-scale-verification-v1.md`
- 本任务包：只回写实际状态、实际写入和执行结果
- `docs/harness-catch-log.md`：仅发现新的真实 catch 时追加；没有则在 evidence 明写零新 catch

除此之外一律不写。尤其禁止修改注入方、Shell、Canvas、maintenance、typed client、runner、fixture 既有场景、Rust/Cargo、依赖、CURRENT、AUTHORITY、计划、决策、真实 store/vault 或 `.codex`。

## 5. 产品与视觉合同

### 5.1 D1 活动栏

- `1440×900`、`1180×760`、`900×760` 三档下，活动栏宽度稳定在 `36–48 px`，八个入口**零断行**（每个按钮 `clientHeight` 不超过单行图标行高 + padding，且 `scrollHeight <= clientHeight`）。
- 每个入口：hit target 至少 `28×28`；accessible name 与 §1.1 列表逐字相符；`aria-pressed` 只在既有有状态入口出现且与状态一致；`:focus-visible` 有可见 outline。
- 图标为 inline SVG，`fill`/`stroke` 取 `currentColor` 或既有 token；`aria-hidden="true"`、`focusable="false"`，不承担可访问名称。
- 活动栏整体无横向/纵向 overflow；折叠左右栏后入口仍可恢复对应面板（沿用 R2 §101–102 已验证行为）。

### 5.2 D2 右栏

- 右栏分为「属性」「反向引用」「来源上下文」**三个**稳定区块（大纲按 §1.2 删字，不设第四区）；每区有可及标题与 `aria-expanded` 明确的折叠控件，折叠后内容退出 Tab/AT 路径。
- 右栏标题字符串不得再声明 `大纲`；green evidence 必须给出改后标题的逐字文本与"全仓 `大纲` 命中数"（应为 `0`，或仅剩与右栏无关的命中并逐条列出）。
- 注入的 `sourceContext` 内容在**未修改**的前提下被产品侧容器包成同一形态区块；证据必须显示夹具 `SyntheticSourceContext` 内容仍逐字呈现。
- 默认展开集合固定且可预测；`900×760` 双栏展开时右栏无横向 overflow，内容纵向滚动限制在区块容器内。
- 空态（`未选择文件`）、loading、冲突提示继续可见，可压低层级；不得因分区而丢字。
- 折叠状态若持久化，只允许复用 `syn-native-knowledge-workspace-ui-v1`；证据必须列出 mount 前/后 localStorage 键集合，且不得出现新键。

### 5.3 D3 Graph 布局

- 纯函数不变量（合同测试）：对 `n = 1…512` 的每个 `n`（可分档抽样但必须含 `1, 2, 6, 7, 12, 40, 100, 512`），任意两节点中心距 ≥ `节点盒宽 + 最小间距`，且同一 `(index, total)` 恒定产出同一坐标。
- 浏览器实测：`1440×900` 下 `n = 6 / 40 / 512` 三个 fresh context，fitView 后全部节点 bounds 在舞台内、无两节点矩形相交、document/body/shell/active group/Graph root 均无横向 overflow。
- **达标缩放（`readableZoom`）定义**：渲染后节点盒实测 `≥ 28 × 28 CSS px` 的最小缩放。按此分层裁决，不得混谈：
  - `n = 6`：fitView 后即须达标——节点可点、标题可见、`titleClipped=false`（沿用 R3D 已验行为，不得回归）；
  - `n = 40 / 512`：fitView **只作俯瞰**，允许不达标、允许标题整体隐藏、允许不可点；但必须另外实测"放大到 `readableZoom` 后"节点盒达标、`aria-label` 完整、仍零相交。
- 每个 `n` 必须逐项记录：fitView 实际缩放、该缩放下节点盒实测宽高、是否达标、`readableZoom` 实测值、标题可见性、相交对数。**§0.1 里指导线给的 `0.027 / 3.6×1.1`、`0.16 / 21×6` 是算术推导，不是观测值——必须被实测覆盖，禁止照抄进 evidence 的"实测"段。**
- `aria-label` 在任何缩放下完整（DOM 属性，与缩放无关），任何 `n` 下都不得因隐藏标题而丢失可访问名称。
- R3D 既有 Graph 不变量在新活动栏宽度下继续成立：常态 chrome 高 `40 px`、节点盒 `136×40`（或按 §1.3 放大后的确定值，需在 evidence 明写并同步合同）、6 节点/5 边全可见、click/Enter/Space 各形成且只形成一次 typed read、`aria-current`/selected/focus 样式不回归。
- 规模场景只量布局：不激活生成节点，不断言其可读性，evidence 明写该限制。

### 5.4 回归锁（跨维度）

把四个冻结 green runner **拷到仓外临时目录**再运行（严禁在原目录运行、严禁覆盖上游 evidence），逐 assertion 报告结果。要求：

- 所有与隔离、command allowlist、写/未知/外部/console/page error 零值、焦点/回焦、overflow、标签组/草稿/Canvas 行为相关的断言**全部仍绿**；
- 唯一允许失败的是**明确编码了旧文字活动栏或旧平铺右栏呈现**的断言；每一条失败必须逐条列出断言名、失败值与"由本包 D1/D2 有意变更"的对应关系。含糊的"若干条因视觉变更失败"不接受。

## 6. Red-first

改产品前先建立并运行三个维度的红测，保留当前实现的精确反例，逐项记录：

1. 活动栏按钮可见文本为中文文字、`900` 下断行（记录每个按钮 bounds、`scrollHeight/clientHeight`、字号）；
2. 活动栏无 inline SVG 图标（记录 `svg` 计数 `0`）；
3. 右栏无区块折叠控件（`aria-expanded` 计数 `0`）、注入内容为平铺长列表（记录条目数与容器 bounds）；
4. 右栏标题声明 `大纲` 但全仓无实现（记录字符串命中与实现命中差；指导线派发前实核为 `大纲` 命中 1 处、`outline*` 标识符命中 0 处，执行线须自行复算而非引用）；
5. `deterministicKnowledgeGraphPosition` 在 `n = 12 / 40 / 512` 下最小中心距小于节点盒宽（给出实算值）；
6. 规模场景浏览器实测存在节点矩形相交（给出相交对数与示例 bounds）。

不得先改实现再补红；不得用 `force` click、runner 注入 `.focus()`、隐藏面板或改 fixture 既有场景造红/绿。某条现状意外通过就如实记录，不得为凑失败数故意破坏。

## 7. Green pure-synthetic 浏览器矩阵

复用真实 React、真实生产 CSS 和冻结 fixture（规模场景为唯一新增），每个场景 fresh browser context，至少覆盖：

1. `1440×900` D1：ribbon 常态、八入口无断行、accessible name/`aria-pressed`/focus-visible；
2. `900×760` D1：窄宽仍无断行、左右栏折叠后入口恢复面板；
3. `1440×900` D2：右栏三/四区块、逐区折叠展开、注入内容逐字保留、Tab 路径；
4. `900×760` D2：双栏展开无横向 overflow、区块内滚动、空态与冲突文案仍在；
5. `1440×900` D3：`n = 6`（fitView 即须达 §5.3 达标缩放：可点 + 标题可见）；6. `n = 40`；7. `n = 512`——后两者按俯瞰口径：fitView bounds 全在舞台内、相交对数 `0`、并**另测 `readableZoom` 下**的节点盒尺寸与零相交；三者均记录 fitView 缩放、节点盒实测宽高、标题可见性；
8. `900×760` reduced-motion：ribbon + 右栏折叠 + Graph 均无新增动画/漂移/overflow。

每个 context 记录：mount 前目标 origin localStorage 为空；最终只允许既有可丢弃 UI chrome 偏好；关键 bounds/computed style；activeElement/ARIA/Tab 路径；response 与 DOM 的节点/边数；各层 overflow。

精确 read allowlist：

```text
knowledge_workspace_snapshot
knowledge_workspace_graph
knowledge_workspace_read_markdown
knowledge_workspace_search
knowledge_workspace_read_canvas
```

（后两项仅在 D1 入口恢复面板场景确有调用时允许，并须逐场景列出次数。）所有 create/write/delete/import/restore、未知 command、外部请求、console error、page error 必须为 `0`。

至少输出：三维度 red runner + JSON + 红图；green runner + JSON；`01-1440-activity-ribbon.png`、`02-900-activity-ribbon-narrow.png`、`03-1440-right-context-sections.png`、`04-900-right-context-collapsed.png`、`05-1440-graph-n40.png`、`06-1440-graph-n512.png`；以及 §5.4 四个回归 runner 的原始输出。

逐图只对照 R0 01/03/04 的对应维度与 R2 `01/06/07/08`；不得把单维度通过写成完整 R0、"Obsidian 1:1"、整个知识 UI 或真实 App 通过。

## 8. 必跑验证

从 `prototypes/productized-desktop-shell`：

1. 聚焦 `knowledge-workbench-shell`、`native-knowledge-workspace`、`knowledge-graph` 三个合同，报告新增断言数与既有断言不回归；
2. 聚焦 `knowledge-canvas` 合同不回归；
3. `npm run typecheck`；
4. `npm run test:offline-interaction`，冻结 runner 仍精确 `37` 入口全过；
5. `wc -l` 报告 `NativeKnowledgeWorkspace.tsx` 与两个新文件行数，全部 `< 2000`。

从仓库根：

6. `node scripts/harness/workbench-shape-gate.js --mode baseline`；
7. `--mode check`：既有 `17 / 5 / 5` 原样报告，**只接受零新增类别/零新增 finding**（尤其不得出现新的 `file_over_limit_not_in_ratchet`）；
8. `node scripts/harness/workbench-shape-gate.hardcoded-hex.selftest.js`；
9. `node scripts/harness/workbench-shape-gate.machine-face.selftest.js`；
10. `git diff --check`；
11. `git diff --cached --name-only` 必须为空；
12. 回算全部冻结只读 hash；给出每个窄写文件相对 §3.1 基线副本的 diff 摘要；
13. 只关闭本包启动的 Vite/浏览器，确认 5173 无 listener。

## 9. 结论枚举（按维度独立裁决）

主结论必须是三段式，每段独立：

- `PASS_R3E_D1_ACTIVITY_RIBBON` / `NEEDS_R3E_D1_REWORK`
- `PASS_R3E_D2_RIGHT_CONTEXT` / `NEEDS_R3E_D2_REWORK`
- `PASS_R3E_D3_GRAPH_SCALE` / `NEEDS_R3E_D3_REWORK`

整包状态另取其一：

- `PASS_N2R_R3E_MERGED / NEEDS_GUIDANCE_REVIEW / NOT_REAL_APP_ACCEPTED`
- `PARTIAL_N2R_R3E / NEEDS_GUIDANCE_REVIEW / NOT_REAL_APP_ACCEPTED`（至少一维度自评未过）
- `BLOCKED_N2R_R3E_BASELINE_DRIFT` / `BLOCKED_N2R_R3E_WRITE_OWNERSHIP_CONFLICT` / `BLOCKED_N2R_R3E_BROWSER_OR_PORT` / `BLOCKED_N2R_R3E_SCOPE_EXPANSION` / `BLOCKED_N2R_R3E_FILE_LIMIT` / `BLOCKED_N2R_R3E_TYPED_CLIENT_OR_WRITE_BOUNDARY`

即使三维度全 PASS，也只能回交 `NEEDS_GUIDANCE_REVIEW / NOT_REAL_APP_ACCEPTED`，不得自行接受 R3E、进入 R4 或启动真实 App。

## 10. 立即停止条件

- 必须修改注入方 `KnowledgeBaseView.tsx`、Shell、Canvas、maintenance、typed client、runner、fixture 既有场景或依赖才能完成；
- 必须解除"禁随机/力导布局"禁令、引入第二图框架、持久化坐标或真实文件写入；
- 抽出后仍会让任一 `.tsx` 触到 `2000` 行限（报 `BLOCKED_N2R_R3E_FILE_LIMIT`，不得靠删测试或压行绕过）；
- 出现未授权 command、write、unknown 或外部网络；
- 红测没有真实覆盖三维度反例，或 green 只能靠 force click/runner focus/隐藏面板通过；
- 需要修改任一冻结 green runner 或覆盖上游 evidence 才能让回归锁通过；
- staged 非空、目标 hash 漂移、并行 writer、5173 非本包占用；
- 需要启动 Syn/Tauri/Obsidian/Codex CLI/MCP、读取真实 store/vault 或进入 N6。

禁止 stage、commit、push、reset、clean、stash、checkout、删除或覆盖上游 evidence。不得终止或修改非本包进程。

## 11. 必须回传

1. HEAD、staged、冻结 hash、基线副本 manifest、窄写 provenance、端口与唯一写所有权；
2. 三维度 red 总断言/失败数/逐项反例/量尺；
3. 三维度最小实现说明（含 D2 删字的实际落点与改后标题逐字文本、D3 布局函数的确定性公式与容量推导）；
4. 三维度 green 量尺与六张图；D3 须含各 `n` 的 fitView 缩放、节点盒实测宽高、`readableZoom` 实测值、标题可见性、相交对数；
5. §5.4 四个回归 runner 的逐 assertion 结果，以及每条允许失败的逐条归因；
6. localStorage 键集合、read allowlist 与各 command 次数、五类零值；
7. 三合同 + Canvas 合同 + typecheck + 37 入口 runner + 行数 + shape + 两项 selftest + diff/staged/hash/基线 diff 摘要/端口；
8. 实际写入文件（含两个新文件）与未完成项；
9. 新 catch；没有则明确写"零新 catch"；
10. 三段式维度结论 + 整包结论。

执行线不得自行验收。指导线收到回交后会独立核基线 diff、三维度 raw 报告、六张图与回归 runner 输出，并按维度分别裁决。

## 12. 实际执行回填

- 状态：**已施工并回交指导线复核**（2026-07-26）。evidence：`evidence/2026-07-26-l3-syn-n2r-r3e-activity-rail-right-context-and-graph-scale-verification-v1.md`。
- 结论：`PASS_R3E_D1_ACTIVITY_RIBBON` / `PASS_R3E_D2_RIGHT_CONTEXT` / `PASS_R3E_D3_GRAPH_SCALE`；整包 `PASS_N2R_R3E_MERGED / NEEDS_GUIDANCE_REVIEW / NOT_REAL_APP_ACCEPTED`。
- 开工先撞写所有权冲突：`18:43` 复核发现 `KnowledgeGraphView.tsx`、`knowledge-workbench-visual-fixture.tsx` 两个窄写目标已偏离派发 hash，且目标 evidence 目录已存在（基线副本 `18:13`、red `18:19`、探针 `18:21`），另一条线在途。执行线按 §3/§10 零写入停止并回交 `BLOCKED_N2R_R3E_WRITE_OWNERSHIP_CONFLICT`；**用户明确指示"你接着做"**后接管，接管前实测仓内已 27 分钟零写入、无活跃 writer。全程未 reset/clean/stash/checkout，未覆盖他人 WIP。
- 继承工作已复核：§3.1 基线副本 7 个 hash 与 §3.2 派发值逐个相符；red `13/7 RED_ESTABLISHED`；D3 环形布局已落地但**自身不变量不成立**（探针 `465` 处失败，`LAYOUT_PITCH` 定义了却没用 + 一处浮点边界），由本线修好至 `PASS_PURE_FUNCTION_INVARIANT / 0 失败`（1…512 全量）。
- 实际写入：新增 `KnowledgeActivityRail.tsx`(144)、`KnowledgeContextSidebar.tsx`(172)；窄写 `NativeKnowledgeWorkspace.tsx`(+32/-78，1978→**1932**)、`KnowledgeGraphView.tsx`(+103/-7)、`styles.css`(+79/-2)、三份合同(+139/-4、+24/-0、+74/-6)、fixture 仅新增规模场景(+41/-2)。逐 hunk patch 见 `raw/baseline-narrow-write-diff.patch`。
- D2 删字落点：右栏标题改为 **`属性 / 反向引用 / 来源上下文`**；渲染结果 `大纲` 零命中，未做任何大纲派生。
- Green：浏览器 **8 context / 126 断言 / 0 失败**；六张规定截图齐备。D3 实测（非推导）：n=6 fitView `1.0`/`136×40`/达标/标题可见；n=40 fitView `0.4955`/`67.39×19.82`/俯瞰，`readableZoom` 实测 `0.7135`/`97.04×28.54`；n=512 fitView `0.1557`/`21.17×6.23`/俯瞰，`readableZoom` 实测 `0.8033`/`109.25×32.13`；三档 fitView 与 readableZoom 下相交对数均 `0`、节点全在舞台内、`aria-label` 全完整。
- §5.4 回归锁：R3B `90/0`、R3C `131/0`、R3C-R1 `73/0` 全绿；R3D 首轮 `74/75`，唯一失败 `reduced-motion Escape still returns to the actual opener` **不属于**允许失败类别，但 A/B 证明它在**未改动的派发基线**上同样失败（`raw/regression-r3d-on-dispatch-baseline.json`），逐帧探针证明回焦行为本身完好（`30 ms` 即回到真实 opener，opener 节点全程未换）。首轮如实上交未动。
- **用户明确授权后修掉该处（超出 §4.3 白名单，仅此一处）**：修的是**测试取样时机不是产品代码**——产品源码三文件 hash 修前修后完全未变。四 runner 扫描确认这是同一文件的漏写（R3C-R1 `:183` 等条件、R3D `:533`/`:543` 各留 40 ms、只有 `:821` 两样都没有），改为有界轮询（≤2000 ms 逐帧）。**反向红测证明断言未被改软**：临时打断产品侧 `restoreFilterOpener` 后重跑，该条照样失败并连带抓出同文件另外 3 条回焦断言；随后逐字节还原并复核 hash。修后 R3D **`75/0`**，四锁合计 **369/0**。R3D 验收原件已留档 `raw/regression-r3d-green-browser-evidence-as-accepted.mjs`（hash 仍 `1cee08e8…`）；**`evidence/raw/…-r3d-graph-convergence/green-browser-evidence.mjs` 冻结 hash 变更 `1cee08e8…` → `bb24c2c7…`，后续包冻结表需同步**。遗留：同文件 `:533`/`:543` 的固定 40 ms 属同类潜在竞态、当前通过，按指示未顺手改。
- §8 门禁：三合同 + Canvas 合同全绿；typecheck 通过；37 入口 runner 全过；行数 1932/144/172 全 < 2000；shape gate baseline 与 check 均 `17/5/5` 且逐条 finding 完全相同（零新增类别、零新增 finding，无新 `file_over_limit_not_in_ratchet`）；两项 selftest `13/13`、`18/18`；`git diff --check` 干净；staged 空；冻结只读 hash 18 个 MATCH + 1 个经用户授权变更（R3D runner）；本包 Vite 已关闭、`5173` 无 listener。
- 新 catch：**两条**，已记入 `docs/harness-catch-log.md`：① R3D 冻结 runner 的回焦断言是时序竞态，原 `75/0` 不是可复现回归锁；② 修 flaky 时用反向红测证明断言未被调松，并保留原件与新旧 hash。
- 未做：无大纲派生；规模场景节点不激活；未进 R4/N6/真实 App；未回写 `CURRENT.md`（不在 §4.3 白名单）。另主动报账一次白名单外瞬时写入（`tests/.r3e-baseline-graph-probe.tsx`，已删除、无残留），见 evidence §7.2。
