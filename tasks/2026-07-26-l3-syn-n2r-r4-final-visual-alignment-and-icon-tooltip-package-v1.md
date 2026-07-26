# 任务包：L3 Syn N2R-R4 最终视觉对照 + 活动栏悬停提示（含一处规则修订）v1

- 日期：2026-07-26
- 状态：**待用户授权派发（DRAFT_AWAITING_DISPATCH）**
- 负责人：独立知识前端执行线
- 指导/验收：当前总指导对话
- 上游接受：`ACCEPTED_R3E_D1_ACTIVITY_RIBBON / ACCEPTED_R3E_D2_RIGHT_CONTEXT / ACCEPTED_R3E_D3_GRAPH_SCALE`（synthetic）/ `NOT_REAL_APP_ACCEPTED`
- 目标 evidence：`evidence/2026-07-26-l3-syn-n2r-r4-final-visual-alignment-verification-v1.md`

## 0. Kickoff

R3 六项 R2 差距已全收（Search/overlay、标签组/分栏、Canvas、Graph、活动栏、右栏）。本包是 N2R 的**最后一关**：把整个知识工作台按维度对照冻结的 R0 实测几何，差哪儿改哪儿；同时把 R3E 复核时抓到的活动栏悬停提示缺口连带那条规则一起修掉。

三个维度：

- **D1 骨架对照**：轨道宽度、chrome 高度、状态区、分隔线、侧栏比例逐项对照 R0 §"实测几何"，超出参照带的改回来。
- **D2 正文字号**：用户已拍板——阅读/编辑正文提到 `16 px` 与 R0 一致；侧栏、标签、状态栏等 chrome 文字**保持现有密集档不动**（真 Obsidian 自身也是 chrome 小于正文）。
- **D3 活动栏悬停提示 + 规则修订**：允许附加 `title`，但必须与 `aria-label` 同值；R3E 那条"整条禁止 title"的合同断言按新规则改写。

本包**不**进入真实 App、真实 store/vault、N6、发布验收；**不**新增依赖；**不**解除"禁随机/力导布局"；**不**碰 Rust。

### 0.1 派发前用户已拍板

- **字号范围 = 骨架 + 正文**（三选一里的推荐项）。**不得**顺手上调 chrome 各级字号，也不得改全局 `--text-*` token 的既有取值——正文另立专用尺寸（见 §4.2）。
- **悬停提示 = 允许附加，不得替代**。`aria-label` 仍是唯一可访问名称来源；`title` 只是给鼠标用户的附加提示，两者必须同值。svg 内部 `<title>` 继续禁止。

### 0.2 指导线派发前实核（非照抄）

- HEAD `1f078835e801caae957901edee0e9d51ab3f64cd`（记录层已落档那笔）、staged 空、`5173` 无 listener。
- R0 §实测几何逐条读过：活动栏 `42 px`、左栏展开 `288 px`、右栏 `185 px`、中央底部状态区 `26 px`、集成顶栏 `39 px`、视图工具栏 `35 px`、左栏 vault/footer `41 px`、正文 `16 px`、缩放 `0`。R0 同时写明"**不应按比例盲目拉伸**"，且 Syn 的验收视口是 `1440×900` 基准 / `1180×760` 紧凑——**不是** `984×768`（那只是 Obsidian 实图尺寸）。本包据此只在 `1440/1180` 量尺，`900×760` 作不回归。
- 当前 shell 实读（`styles.css:8389-8394`）：`grid-template-columns: 42px minmax(220px,288px) minmax(0,1fr) minmax(185px,240px)`、`grid-template-rows: minmax(0,1fr) 26px` → 活动栏、状态区已合参照，左右栏是**带**而非定值，中央 chrome 高度需实测后与 R0 的 `39+35` 关系对齐。
- 当前正文实读：`.native-workspace-markdown { font-size: var(--text-sm) }` = `12 px`（`styles.css:8197-8204`），距 R0 的 `16 px` 差一档以上；`--text-*` 全套为 `10/11/12/13/15`，**没有** 16 px 档。
- 悬停提示待改的断言原文在 `tests/knowledge-workbench-shell.test.tsx:741-745`：`!markup.includes("<title") && !markup.includes("title=")`。
- `KnowledgeBaseView.tsx` 已无页面级大标题/仪表盘残留（`pg-head`、`<h1>`、"统计"零命中）→ R0 迁移表那条"页面级大标题占工作区高度"已在 R1 消化，本包不需为此动组件。

## 1. 设计意图与拒绝项

- Intent：这是**收尾对照**，不是重做。只把量出来与 R0 参照不符的地方拉回参照带，以及把正文提到可读档。
- 判据：R0 的实测几何是**参照带**，不是像素级契约；R0 文档本身禁止宣称"1:1""像素级通过"。
- Signature：Syn 自有暖纸/墨色/茶色/青玉与既有字体族全部保留；不引入 Obsidian 品牌层、图标或配色。

拒绝项：

1. 宣称"完整 R0 通过""Obsidian 1:1""像素级对齐" → 只能按维度报"在参照带内 / 超出并已修 / 有意分歧"；
2. 改全局 `--text-*` token 既有取值，或顺手上调 chrome 字号 → 只允许新增正文专用尺寸并只在正文族引用；
3. 用整体缩放（`zoom`/`transform: scale`）冒充字号对齐；
4. 为了把数字凑进参照带而改 R0 文档、R2 evidence 或任何上游冻结件；
5. 用 `title` 替代 `aria-label`，或让 svg 承担名称（svg 内 `<title>` 仍禁）；
6. 新增卡片仪表盘、全宽模块堆叠、重复标题区或第二个竞争主面板；
7. 借"密度对照"改动 Graph 布局函数、Canvas 内容、reducer、typed client 或写路。

项目界面模式正本：`.interface-design/system.md`。

## 2. Authority、能力和并发边界

- authority_chain：`AGENTS.md` → `CURRENT.md` → native knowledge route v2 → small-stage plan v2（§R4 最终视觉对照）→ R0 → R2 gap matrix → R3B/R3C/R3C-R1/R3D/R3E accepted evidence → 本包。
- plan_anchor：`docs/plans/2026-07-23-l3-syn-native-knowledge-workspace-small-stage-plan-v2.md#r4-最终视觉对照`
- existing_before_new：shell 网格轨道、状态栏、标签组 header、右栏三区、活动栏 ribbon、Markdown 渲染族、既有字体/颜色 token 均已存在；本包只调数值与新增一个正文尺寸，不新造组件、不新造 token 族。
- capabilities_touched：none。command 名、payload、vault root、后端、写路、capability registry 一律不动。
- data_truth：只改呈现；不写回 Markdown、vault、store，不新增持久化键。

本包独占：`styles.css` 中 §4.2 列明的 selector 族、`KnowledgeActivityRail.tsx`、`knowledge-workbench-shell.test.tsx`。其他 UI 包在 R4 结束前不得并发写这三处。对话底座线在本包产品写入与浏览器取证期间只可静态只读准备。

## 3. 冻结基线与开工停点

- HEAD：`1f078835e801caae957901edee0e9d51ab3f64cd`
- 派发时 staged：空
- 派发时 `127.0.0.1:5173`：无 listener
- 工作树仍有大量既有 WIP（代码尚未入库，见计划）。不得用 clean tree 假设覆盖现状。

### 3.1 基线副本（沿用 R3E 硬规矩）

改动前把**三个窄写目标**逐字节复制到

`evidence/raw/2026-07-26-l3-syn-n2r-r4-final-visual-alignment/baseline/`

并写 `baseline-manifest.txt`（`<SHA-256>  <仓内相对路径>`，与下表逐个相符）。**没有基线副本不得开始产品写入**；收口必须给出每个窄写文件相对副本的 diff 摘要（改动行数 + 每个 hunk 的 selector/函数名）。

### 3.2 冻结表

| 文件 | 派发 SHA-256 | 本包权限 |
| --- | --- | --- |
| `src/styles.css` | `bb814766866cb4737d5cb67a47dcd7a9903c668b219d5943c09f786531809257` | 只写 §4.2 精确族 |
| `src/views/knowledge/KnowledgeActivityRail.tsx` | `3f1c20b9cdf51dd06750fe4b09a802561985f11b7866b268f66f130357c0a02e` | 窄写（仅 D3 悬停提示） |
| `tests/knowledge-workbench-shell.test.tsx` | `d79263fc8c5a930a1852496f7459e88f77a4e7cb4e9da6a37f1848ff5d9f4dda` | 窄写（合同：改写 title 断言 + 新增 R4 量尺断言） |
| `src/views/knowledge/KnowledgeContextSidebar.tsx` | `0184c075236848285d67b34311287e70876bab833f23e88c3a5691f61d2b8776` | 冻结只读 |
| `src/views/knowledge/NativeKnowledgeWorkspace.tsx` | `f24235ad60bb93ffc710cee96ceb5aabd8d9bf5ef48ec4598e0a098f3bf94ad4` | 冻结只读 |
| `src/views/knowledge/KnowledgeGraphView.tsx` | `821a49d7c442cf1c15a3b4c689e66beee8fba5694f31307bfdcda652830eb768` | 冻结只读 |
| `src/views/knowledge/KnowledgeWorkbenchShell.tsx` | `2ab66a5b00941e3cf0a083e3ed3d24b309f8b7fe5037dd41d8743cf9d2d41c21` | 冻结只读 |
| `src/views/knowledge/KnowledgeCanvasView.tsx` | `42054605e642100752a109830a8f7601a46eaef68f1bc70c7ca48e5353d6405e` | 冻结只读 |
| `src/views/KnowledgeBaseView.tsx` | `3fa18f9fbba0f6c797cc568132389ac2c02a0e12c7dc4861a27ab4f6c2309e58` | 冻结只读 |
| `src/lib/knowledgeWorkspace.ts` | `81eddc02206035a9d1bf1e3954d5ea19b1864a71b27ffea43efafa5a2fd51fb0` | 冻结只读 |
| `src/lib/knowledgeWorkbenchLayout.ts` | `8fb9faf01ef9dae509a6cffa4e14fd62031bb9744ea8e44eb1d1476e4f9ee220` | 冻结只读 |
| `src/lib/tauri.ts` | `95587bdd68c7e207e18d6ecdc2c862a260706c9aa7f5c3085b7dcf95d8dc14ee` | 冻结只读 |
| `tests/native-knowledge-workspace.test.tsx` | `716e9ea5627960d8a764d243bb5220a9307d923a096c40e437b9d8025094c10c` | 冻结只读 |
| `tests/knowledge-graph.test.tsx` | `10e2763572fa7a46feb12852a709579fc10bb716962e9f3e28327a4d5acd2ded` | 冻结只读 |
| `tests/knowledge-canvas.test.tsx` | `802dd4a90ceac4f491ddbf390f8e5734f02984af87c920c170b17d0c723f263c` | 冻结只读 |
| `tests/knowledge-workbench-visual-fixture.tsx` | `4e69d0fa89408643f8e280e32603ca08d735de5dba3ec649d6f2836dfd5eb458` | 冻结只读 |
| `tests/knowledge-workbench-visual-fixture.html` | `8a58ac0d02d9c33a9ca151f4ec1e653ccd3c6d79096581efd339a62c1fcec23c` | 冻结只读 |
| `scripts/run-offline-interaction-test.mjs` | `798daf13722875befa22eb233c476b957d241005a3b08267ed30eb54e0363082` | 冻结只读（仍 37 入口） |
| `package.json` | `08a3abc466e2dd51f946380badb6519b4273353d8d279efa43b1cf6086d87e65` | 冻结只读 |
| `package-lock.json` | `781cb1d94eaaeec0d071f156f9aeca65400acce8bd811c9c95cd4d45cf700bc0` | 冻结只读 |
| R0 参照 `docs/design/2026-07-25-…-r0-v1.md` | `141ff343312fff91346e6e170f1aea2d07cda5fa2abdb3fde8b840b16f9ed4d9` | 冻结只读 |
| R2 基线 evidence | `8dd432266f406e96c69359196ea138eede85891d041ed8f98a3907eed55b6c7a` | 冻结只读 |
| R3E task | `17f70486ada515c98a867cd5caf7186975062038aae37f5a5e1a6702212600cf` | 冻结只读 |
| R3E evidence | `a47069c06a30860c4d68451fae0cf1f1877ee5d4d0ae6d9dc9bb50c70999d934` | 冻结只读 |
| R3E green runner（回归工具） | `bf2f56590bcd9393257281b91b0720a077831b85a96f4139a53937a7f46384eb` | 冻结只读 |
| R3D 回归工具（R3E 自有修正版） | `c956dfd8550d0e27ef9b4d341893a4c8a7108ab26ca9b4587bda568446129567` | 冻结只读 |
| R3D green runner（验收原件） | `1cee08e83b44f9f6a6e6d4ea41c1ff005591e79528bb4fa64b2440df54bb9ee1` | 冻结只读 · **不得再改** |
| R3B green runner | `087653734143ef34edda722b8b097f02cd0f8ca64111f688a21c8a255dc0e736` | 冻结只读 |
| R3C green runner | `c02997a36c6a6cc6bc8b837d7f03c41f8533eb726735f0e0788f17b86b984468` | 冻结只读 |
| R3C-R1 green runner | `1dead02d262c6ebd682e6d491e264647fd9ec77cd2caef2450e3bf923dc77f5f` | 冻结只读 |

任一 hash 漂移、staged 非空、端口被占或存在并行 writer → 按 §10 停止。不得 reset / clean / stash / checkout / 覆盖他人 WIP。**已验收 evidence 目录一律只读**（R3E 教训：修复归新包所有，历史验收件不改）。

## 4. 精确写入白名单

### 4.1 产品与合同

1. `src/views/knowledge/KnowledgeActivityRail.tsx`：只允许给 8 个按钮加 `title={item.label}`（与 `aria-label` 同值），并更新文件头注释说明新口径。其余（图标几何、`aria-hidden`、`focusable`、`aria-pressed`、dispatch）零改动。
2. `tests/knowledge-workbench-shell.test.tsx`：
   - 把 `:741-745` 那条断言从"整条禁止 title"改写为：**8 个按钮各有 `title`，且 `title` 值与同一按钮的 `aria-label` 逐字相同；svg 内 `<title>` 计数为 0；`aria-hidden="true"` / `focusable="false"` 仍各 8**；
   - 新增 R4 骨架与正文断言（静态可判部分）；量尺主证据仍在 §7 浏览器。
3. `src/styles.css`：见 §4.2。

### 4.2 styles.css 允许的族

- **骨架**：`.syn-knowledge-shell`（`grid-template-columns` / `grid-template-rows` 与两个轨道变量）、`.syn-knowledge-shell__activity|__left|__central|__right|__status`、`.knowledge-workbench-group__header`、`.knowledge-workbench-group__tabs`、`.knowledge-workbench-separator` 及其分隔线；
- **正文**：新增**一个**正文专用尺寸（例如 `--text-body: 16px`，只允许新增、不得改既有 `--text-*` 取值），并只在阅读/编辑正文族引用：`.native-workspace-markdown` 及其 `p / li / blockquote / h1-h4` 后代、编辑正文输入面（executor 必须先从源码枚举出确切 selector 并在 evidence 列出）；正文族内的 `line-height` / `padding` / `max-width` 可随字号同调；
- 明确**不许**动：全局 `:root` 既有 token 取值、活动栏图标尺寸、Graph/Canvas 内部、右栏 `.native-context-*` 字号、状态栏字号、overlay、maintenance、非本包 media/container rule；
- 不新增 hardcoded hex、gradient、backdrop blur、装饰性 shadow 或第二套字体族。

### 4.3 新 evidence

- `evidence/raw/2026-07-26-l3-syn-n2r-r4-final-visual-alignment/**`（含 §3.1 baseline）
- `evidence/2026-07-26-l3-syn-n2r-r4-final-visual-alignment-verification-v1.md`
- 本任务包回填
- `docs/harness-catch-log.md`：仅发现新的真实 catch 时追加；没有则在 evidence 明写零新 catch

除此之外一律不写。禁止 stage / commit / push。

## 5. 产品与视觉合同

### 5.1 D1 骨架对照（逐项，`1440×900` 与 `1180×760` 各一遍）

必须实测并逐项对照下表；**每项只能判"在带内 / 超出已修 / 有意分歧（须写理由）"**：

| 维度 | R0 参照 | 判据 |
| --- | --- | --- |
| 活动栏宽 | `42 px` | 定值，两档均 `42`（±0） |
| 左栏展开宽 | `288 px` | `1440` 档应落在 `220–288` 带内且上界贴 `288`；`1180` 档允许收窄但不得低于 `220` |
| 右栏宽 | `185 px` | 落在 `185–240` 带内；不得低于 `185` |
| 中央底部状态区高 | `26 px` | 定值（±0） |
| 中央 chrome（标签头 + 视图工具条）总高 | R0 为 `39 + 35 = 74 px` 量级 | 实测 Syn 对应层总高，超出 `74 ±10` 需说明 |
| 左栏 footer/vault 带 | `41 px` | Syn 若无该带，明写"无此带、不构成差距" |
| 分隔线 | 单像素 hairline | 两档实测 `borderWidth ≤ 1px`，无双线/无粗边 |
| 侧栏比例 | 左 : 中 : 右（R0 单组 `288 : 235+234 : 185`） | 报实测比例；只要求"中央最宽、左次之、右最窄"的秩序成立 |

同时（沿用 R2/R3 判据）：`1440` 与 `1180` 两档 document / body / shell / 中央面 / 活动组面板 / 右栏 均零横向 overflow；无文字截断（`scrollWidth ≤ clientWidth` 且无省略号意外命中）；无卡片仪表盘、无全宽模块堆叠、无重复标题区、无第二个竞争主面板。

### 5.2 D2 正文字号

- 阅读态与编辑态正文 computed `font-size` 实测为 `16 px`；`line-height` 落在 `1.5–1.8`。
- chrome 文字**未变**：活动栏、左栏树、标签、右栏三区、状态栏、Graph 节点标题的 computed `font-size` 在改前改后**逐项相同**（必须给出改前/改后对照表——改前值取自 §3.1 基线副本渲染，不是凭记忆）。
- 提字号后两档均不得新增 overflow / 截断 / 换行崩坏；右栏与左栏在 `1180` 下仍无横向 overflow。
- Markdown 内 `pre` / `code` / 标题层级仍成比例可读，不得因正文变大而与代码块字号倒挂。

### 5.3 D3 悬停提示

- 8 个按钮 `title` 与 `aria-label` 逐字同值；浏览器实测 hover 后 `title` 属性存在（属性存在即可，不要求截原生 tooltip 位图）。
- accessible name 计算结果**仍为 `aria-label`**：实测每个按钮的可访问名称与 `aria-label` 一致（`aria-label` 优先于 `title`），不得出现双读或名称变化。
- svg 仍 `aria-hidden="true"` / `focusable="false"`、内部无 `<title>`。
- 键盘路径与 `aria-pressed` 行为零变化。

### 5.4 回归锁

把 **6 个冻结 runner** 逐字节拷到仓外临时目录运行（严禁在原目录运行、严禁覆盖任何上游 evidence）：R3B、R3C、R3C-R1、R3E green、R3D 验收原件、R3D 的 R3E 修正版。逐 assertion 报告。

- 与隔离、command allowlist、五类零值、焦点/回焦、overflow、标签组/草稿/Canvas/Graph 行为相关的断言**全部仍绿**；
- 唯一允许失败的是**明确编码了改前骨架数值或改前正文字号**的断言（例如某处硬断言正文 `12px`）；每条失败必须列断言名、改前值、改后值，并对应到 §5.1/§5.2 的哪一项。含糊的"若干条因视觉变更失败"不接受；
- R3D 验收原件那条已知的取样竞态断言若再翻面，按 R3E §6 的既有归因处理，**不得再改该文件**。

## 6. Red-first

改产品前先建立红测，逐项留档：

1. 正文 computed `font-size` 实测为 `12 px`（≠ R0 的 `16 px`）；
2. 活动栏 8 个按钮 `title` 计数为 `0`；
3. §5.1 表里当前**超出参照带**的每一项（逐项给实测值，含 `1440` 与 `1180` 两档）；
4. 至少一张改前全景图（`1440`）。

某项现状已在带内 → 如实记录"无需改动"，不得为凑失败数故意破坏。不得先改实现再补红；不得用 `force` click、runner 注入 `.focus()`、隐藏面板或改 fixture 造红/绿。

## 7. Green pure-synthetic 浏览器矩阵

真实 React + 真实生产 CSS + 冻结 fixture，每场景 fresh context，至少：

1. `1440×900` 全景（Markdown 阅读态）：§5.1 全表 + 正文 `16px` + chrome 字号对照；
2. `1180×760` 全景：同上，另证零 overflow / 零截断；
3. `1440×900` 编辑态：编辑正文 `16px`、光标可用、保存/冲突语义未变；
4. `1440×900` 活动栏：8 个 `title` 同值 + accessible name 仍取 `aria-label` + svg 装饰性;
5. `1180×760` 双栏分栏（Markdown + Graph）：提字号后分栏仍无 overflow、Graph 节点字号未变；
6. `900×760` 不回归：折叠组合下仍零 overflow（本档不作 R0 对照）。

每 context 记录 mount 前 localStorage 为空、最终只含既有可丢弃 UI chrome 偏好、关键 bounds/computed style、五类零值（write / unknown / 外部 / console error / page error 全 `0`）、各层 overflow。

至少输出：red runner + JSON + 改前图；green runner + JSON；`01-1440-final-alignment.png`、`02-1180-final-alignment.png`、`03-1440-reading-16px.png`、`04-1440-activity-tooltip.png`、`05-1180-split-after-typography.png`；以及 §5.4 六个回归 runner 的原始输出。

逐图只对照 R0 `01/02/03` 的对应维度。**不得**声称完整 R0 通过、"Obsidian 1:1"、像素级对齐或真实 App 通过。

## 8. 必跑验证

从 `prototypes/productized-desktop-shell`：

1. 聚焦 `knowledge-workbench-shell` 合同（报告改写后的 title 断言与新增 R4 断言，既有断言零回归）；
2. 聚焦 `native-knowledge-workspace`、`knowledge-graph`、`knowledge-canvas` 三合同不回归；
3. `npm run typecheck`；
4. `npm run test:offline-interaction`（冻结 runner 仍精确 37 入口全过）；
5. `wc -l` 报告三个窄写文件行数（`.tsx` 均 `< 2000`）。

从仓库根：

6. shape gate `--mode baseline`；
7. shape gate `--mode check`：既有 `17 / 5 / 5` 原样报告，**零新增类别/finding**（styles.css 棘轮 delta 变化属同一条既有 finding，需明写改前/改后行数）；
8. hardcoded-hex selftest；
9. machine-face selftest；
10. `git diff --check`；
11. `git diff --cached --name-only` 为空；
12. 回算全部冻结只读 hash + 给出三个窄写文件相对基线副本的 diff 摘要；
13. 只关闭本包启动的 Vite，确认 `5173` 无 listener。

## 9. 结论枚举

三段式，各自独立：

- `PASS_R4_D1_SKELETON` / `NEEDS_R4_D1_REWORK`
- `PASS_R4_D2_BODY_TYPOGRAPHY` / `NEEDS_R4_D2_REWORK`
- `PASS_R4_D3_ICON_TOOLTIP` / `NEEDS_R4_D3_REWORK`

整包：

- `PASS_N2R_R4_FINAL_ALIGNMENT / NEEDS_GUIDANCE_REVIEW / NOT_REAL_APP_ACCEPTED`
- `PARTIAL_N2R_R4 / NEEDS_GUIDANCE_REVIEW / NOT_REAL_APP_ACCEPTED`
- `BLOCKED_N2R_R4_BASELINE_DRIFT` / `BLOCKED_N2R_R4_WRITE_OWNERSHIP_CONFLICT` / `BLOCKED_N2R_R4_BROWSER_OR_PORT` / `BLOCKED_N2R_R4_SCOPE_EXPANSION` / `BLOCKED_N2R_R4_UPSTREAM_EVIDENCE_IMMUTABLE`

即使三维度全 PASS，也只回交 `NEEDS_GUIDANCE_REVIEW / NOT_REAL_APP_ACCEPTED`；不得自行接受 R4、不得声称 N2R 完成、不得启动真实 App。

## 10. 立即停止条件

- 必须改冻结组件（Shell / ContextSidebar / GraphView / CanvasView / NativeKnowledgeWorkspace / KnowledgeBaseView）、typed client、fixture、runner、依赖或 Rust 才能完成；
- 必须改全局 `--text-*` 既有取值或上调 chrome 字号才能达标；
- 必须改 R0 文档、R2 evidence 或任何已验收 evidence 目录（含 R3D 验收原件）；
- 必须用整体缩放冒充字号对齐，或用隐藏/裁切消除 overflow；
- staged 非空、目标 hash 漂移、并行 writer、`5173` 非本包占用；
- 需要启动 Syn/Tauri/Obsidian/Codex CLI/MCP、读真实 store/vault 或进入 N6。

禁止 stage、commit、push、reset、clean、stash、checkout、删除或覆盖上游 evidence；不得终止或修改非本包进程。

## 11. 必须回传

1. HEAD、staged、端口、冻结 hash、基线副本 manifest、唯一写所有权；
2. red 逐项反例（含 §5.1 表的改前实测值，两档）；
3. 三维度最小实现（含正文专用尺寸的定义位置与被引用的确切 selector 清单）；
4. §5.1 全表改后实测（两档）+ §5.2 chrome 字号改前/改后逐项对照表；
5. §5.3 三条实测（title 同值、accessible name 仍取 aria-label、svg 装饰性）；
6. 五张 green 图 + 改前图；
7. §5.4 六个 runner 的逐 assertion 结果与每条允许失败的逐条归因；
8. §8 全部门禁 + 三个窄写文件的 diff 摘要；
9. 新 catch；没有则明写"零新 catch"；
10. 三段式 + 整包结论。

执行线不得自行验收。指导线会独立重算量尺、重跑门禁与回归锁，并按维度分别裁决。

## 12. 实际执行回填

- 状态：**已施工并回交指导线复核**（2026-07-26，用户明确授权"开工"）。evidence：`evidence/2026-07-26-l3-syn-n2r-r4-final-visual-alignment-verification-v1.md`。
- 结论：`PASS_R4_D1_SKELETON`（有保留，见下）/ `PASS_R4_D2_BODY_TYPOGRAPHY` / `PASS_R4_D3_ICON_TOOLTIP`；整包 `PASS_N2R_R4_FINAL_ALIGNMENT / NEEDS_GUIDANCE_REVIEW / NOT_REAL_APP_ACCEPTED`。
- 开工基线：HEAD `1f07883` 一致、staged 空、`5173` 无 listener、§3.2 抽核 10 项零漂移；§3.1 基线副本三个 hash 与派发值相符。R0 几何逐条读自 R0 原文 §2.3，未照抄 §0.2 转述。
- Red：`RED_ESTABLISHED`，**18 断言 / 8 失败**。改前实测：正文阅读/编辑均 `12px`；活动栏 title `0/8`；§5.1 八项里**只有中央 chrome 超标**（`132.19` vs R0 `74`），其余在带内（活动栏 42、状态区 26、左栏 288/260、右栏 240/220、零 overflow）；左栏无 vault/footer 带。红测顺带实核两处 DOM：Markdown 默认落源码态、源码/预览按钮在 `.knowledge-workbench-projection-controls`。
- 实际写入三个窄写文件：`styles.css`(+8/-2,3 hunk：新增 `--text-body:16px` + 两个正文面引用它)、`KnowledgeActivityRail.tsx`(+7/-2,2 hunk：`title={item.label}` + 注释)、`knowledge-workbench-shell.test.tsx`(+66/-3,2 hunk：title 断言改写 + R4 D2 静态断言 4 条)。既有 `--text-*` 七档取值一个未改；chrome 字号一项未动。
- Green：**6 context / 73 断言 / 0 失败**，五张规定图 + 改前图齐备。D2 实测阅读 `16/1.7`、编辑 `16/1.65`，chrome 八项改前改后逐项相同（Graph 节点标题红测未覆盖，另以 diff + 合同断言 + 分栏实测 `12px` 三路补证）。D3 以 **CDP 无障碍树**实证：八个按钮计算名称生效来源全部是 `aria-label`，`title` 被标 superseded，不双读、不顶替。
- §5.4 六锁：R3B `90/0`（首跑一次按键投递抖动 `now=55`，复跑 4 次全过）、R3C `131/0`、R3C-R1 `73/0`、R3D 修正版 `75/0`；**R3E 3 条**为 §0.1 口径修订的预期翻面（复合断言四子句里只有 `title===null` 翻，另三子句违反数均为 0）；**R3D 验收原件 1 条**为 R3E 已记账的取样竞态，按既有归因处理、未改该文件。
- §8 门禁：四合同全绿（shell `R3B 16/0; R3E 22/0`）、typecheck 通过、37 入口全过、两个 `.tsx` < 2000、shape gate baseline 与 check 均 `17/5/5` 且逐条 finding 相同（`styles.css` 棘轮 12496 → 12502，同一条既有 finding）、两项 selftest `13/13`+`18/18`、`git diff --check` 干净、staged 空、**24 项冻结只读 hash 全 MATCH**、Vite 已关 `5173` 无 listener。
- 新 catch：**零新 catch**（两条异常一条是已拍板口径修订、一条是既有账；R3B 抖动单次未复现，暂不立账）。
- **未完成并需指导线裁决**：中央 chrome 总高超出参照带 `58.19 px` 未修——压它必须动 `.native-workspace-document-head`，该 selector 不在 §4.2 白名单且 §10 列为立即停止条件。差距来源已定位：组头 `36`（合参照）+ 文档头 `96.19`（R0 对应层 35，Syn 多一条常驻路径/标题/模式带）。请裁决接受为"有意分歧"还是另开窄包。
- 其余未做：`pre` 代码块字号未动（不在允许引用正文档的枚举内）；未回写 `CURRENT.md`（不在白名单）；未进真实 App / N6 / 发布验收。
