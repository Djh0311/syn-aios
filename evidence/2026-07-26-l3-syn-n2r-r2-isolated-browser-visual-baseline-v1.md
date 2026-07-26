# L3 Syn N2R-R2 隔离浏览器视觉基线与差距审计 evidence v1

- 日期：2026-07-26
- 状态：**ACCEPTED_N2R_R2_BASELINE / NEEDS_N2R_R3_VISUAL_CONVERGENCE / NOT_REAL_APP_ACCEPTED**
- 结论枚举：`NEEDS_N2R_R3_VISUAL_CONVERGENCE`
- 证据级别：真实 React 组件 + 真实生产 CSS 的 localhost 隔离浏览器渲染；**不是**真实 Syn/Tauri App、I5、Gate 0 或发布验收。

## 1. 范围与隔离

本包只新增视觉夹具、九张规定的 synthetic-only 截图、一次窄返工的 Search 结果态补图，以及不含正文/绝对路径/原始 IPC payload 的量尺 JSON；未修改任何产品 `src/**`、Rust、现有测试/runner、Vite 配置、`CURRENT.md` 或 `AUTHORITY.md`。

最终工作树中 `docs/harness-catch-log.md` 仍显示为修改态（30 行既有历史记录 hunk）；本执行线没有写入或变更该文件。指导复核已登记 R2 的“几何绿灯替代 R0 视觉逐图对照”catch；本次窄返工没有新增另一项 catch，故将该文件原样保留在本包产物之外。

- 夹具直接导入真实 `NativeKnowledgeWorkspace` 与真实 `src/styles.css`，没有复制生产 DOM/CSS。
- 目录、Markdown、搜索、Graph、Canvas、来源和右栏内容均为内存中的合成数据。
- 初始九图使用 Chromium 的 `freshContexts: 5`，每个 context mount 前目标 origin 的 `localStorage` 都为空；窄返工的第 10 图另用一个全新 context，并再次得到 mount 前 `localStorage` 为空。
- 未使用用户 Chrome profile、默认 store、真实 vault、真实 Syn/Tauri、Obsidian、Codex CLI/MCP 或网络资源。
- 本包启动的 Vite 只监听 `127.0.0.1:5173`，已关闭；最终 `lsof -n -P -iTCP:5173 -sTCP:LISTEN` 无输出。

预检：HEAD 为 `e9ad7f3a204a1ebb11ce26c1e8c05b19c04c0991`；暂存区为空；目标 Syn/Tauri/Vite/Codex CLI/MCP 进程及 5173 端口在启动前均未占用。冻结生产读取面复核如下，全部匹配任务包：

| 相对 `prototypes/productized-desktop-shell/` 路径 | SHA-256 |
| --- | --- |
| `src/views/KnowledgeBaseView.tsx` | `3fa18f9fbba0f6c797cc568132389ac2c02a0e12c7dc4861a27ab4f6c2309e58` |
| `src/views/knowledge/NativeKnowledgeWorkspace.tsx` | `456153d284712beadd46053b4871417ef33fa8bcacbf4a86a117b641774b3e0a` |
| `src/views/knowledge/KnowledgeWorkbenchShell.tsx` | `2ab66a5b00941e3cf0a083e3ed3d24b309f8b7fe5037dd41d8743cf9d2d41c21` |
| `src/lib/knowledgeWorkbenchLayout.ts` | `cc3a9f8811edb3b6ac231a37738a3b6f0bfba0794aa66f81d4b3a14015fd083a` |
| `src/styles.css` | `fe8bfc74e8d946255fd2850d7ec9218928c0c10995ae6d21450eb7a262bdea71` |
| `tests/knowledge-workbench-shell.test.tsx` | `60b7f8d0e8ed618e0bc51ab857542e55d8c5882dc39d01270a127ca5567e5f2d` |
| `vite.config.ts` | `68f0c2b5a2fa45af381a43eaaff7a7054b83822d358fac186d3bd3ea0824b506` |

## 2. 夹具与 mock 证据

新增夹具：

- `prototypes/productized-desktop-shell/tests/knowledge-workbench-visual-fixture.tsx`
- `prototypes/productized-desktop-shell/tests/knowledge-workbench-visual-fixture.html`

精确只读 mock allowlist：

```text
knowledge_workspace_snapshot
knowledge_workspace_search
knowledge_workspace_graph
knowledge_workspace_read_markdown
knowledge_workspace_read_canvas
```

所有 create/write/delete/import/restore command 与未识别 command 均 fail closed。初始 5 个 context 的汇总为 `everyContextZeroWriteCalls: true`、`everyContextZeroUnrecognizedCalls: true`、`externalNetworkRequests: []`。末次全场景浏览器审计覆盖 Markdown、Preview、Graph、Canvas、command overlay、Search 与 quick-open：console error `0`、page error `0`、外部请求 `0`、写调用 `0`、未识别调用 `0`。窄返工的单独 fresh context 也记录到外部请求 `0`、写调用 `0`、未识别调用 `0`。

夹具装配期间曾发现浏览器默认请求 favicon 的 404；仅在新夹具 HTML 内加入内嵌 data icon 后，以 fresh context 复测为 0 console/page error。该修正不触及产品 DOM/CSS、runner 或任何真实数据，不构成需登记的产品/harness catch。

原始量尺：[measurements.json](raw/2026-07-26-l3-syn-n2r-r2-visual-baseline/measurements.json)。

## 3. 九张规定截图、补充 Search 图与 R0 逐图判定

截图目录：`evidence/raw/2026-07-26-l3-syn-n2r-r2-visual-baseline/`。均只含合成数据。

“单壳仍成立”“overlay/回焦可用”和几何通过均保留为浏览器合同证据，**但不能替代 R0 逐图视觉结论**。直接对照冻结 R0 实图后，以下每图至少有可复现的视觉或覆盖缺口，故均为 `GAP`；没有把任何一图标记为 R0 视觉 `PASS`。

| 截图 | 视口与场景 / R0 对照 | 判定 | 保留的有效事实与 R0 可见差距 |
| --- | --- | --- | --- |
| `01-1440-markdown-preview.png` | 1440 × 900，Markdown/Preview；R0 01 | GAP | 单壳与中央内容仍成立；活动栏是可换行文字按钮，中央呈双层模式/文档标签且只见单组，右栏为长列表。R0 是紧凑图标 ribbon、可关闭/新增的多标签组/分栏关系及更分层的右侧上下文。 |
| `02-1440-graph.png` | 1440 × 900，Graph；R0 01/03/04/08 | GAP | Graph 确在中央根内；当前 `.native-graph-node` 为矩形卡片阵列及元数据块，非 R0 轻量节点/连线网络。 |
| `03-1440-command-overlay.png` | 1440 × 900，command overlay；R0 05 | GAP | dialog、Escape 与回焦合同成立；画面是“新建 Markdown/目录”表单，而非 R0 可搜索的高密度命令列表、当前选择与键盘提示。 |
| `04-1180-search-preview.png` | 1180 × 760，初始 Search + Preview；R0 03 | GAP（覆盖） | 三栏几何成立，但初始图只有“打开快速搜索”按钮；该场景未调用 `knowledge_workspace_search`、未显示结果，不能作为 R0 左栏 Search 结果态的视觉对照。补图 10 仅补足真实查询/结果执行，并不把该差距判为通过。 |
| `05-1180-canvas.png` | 1180 × 760，Canvas；R0 04 | GAP | Canvas 仍在中央且侧栏未下堆；当前是常驻 catalog/创建表单、工具条和“原始 JSON 局部编辑”分区，R0 为连续 canvas-first 空间和小型浮动工具。 |
| `06-1180-left-collapsed.png` | 1180 × 760，左栏折叠；R0 08 的相关壳状态 | GAP | 左栏退出 Tab/AT 路径且中央接管空间；同时保留文字活动栏和 Canvas 的常驻表单/JSON 分区，仍不能构成 R0 视觉通过。 |
| `07-900-double-sidebar-markdown.png` | 900 × 760，双侧栏展开 Markdown；R0 01/03 的相关桌面壳 | GAP | 中央宽 `436 px` 且三段壳存在；文字活动栏在窄宽度出现断行/可扫读性下降，双层标签和右栏长列表亦与 R0 的 ribbon、标签组/层级不同。 |
| `08-900-right-collapsed.png` | 900 × 760，右栏折叠 | GAP | 真实几何缺口：shell `scrollWidth=899`、`clientWidth=898`；另有文字活动栏和标签/右栏层级的 R0 差异，不能把其余视觉面视为通过。 |
| `09-900-quick-open-overlay.png` | 900 × 760，quick-open overlay；R0 06 | GAP | overlay 浮于工作台、Escape 回焦与主页面无 overflow 均成立；画面仅有空快速打开表单，缺 R0 的候选结果、当前选择和键盘提示。 |
| `10-1180-search-results-preview.png` | 1180 × 760，实际输入 `合成` 后 Search 结果；R0 03/06 | GAP（补齐执行覆盖） | 新 fresh context 得到 `knowledge_workspace_search=1`、`resultCount=16`、零写/外部/未识别调用；结果显示在 quick-open overlay，而非 R0 03 的左栏结果态，且没有 R0 06 的当前选择/键盘提示。 |

本轮仍未发现 Obsidian 商标、logo、原图标或 CSS 复制迹象；这只支持品牌/隔离边界，不支持任何 R0 高保真视觉 PASS。

## 4. 几何、overflow 与内部滚动

完整 x/y/width/height 在 JSON。下表为五区宽高的关键实数（活动 / 左 / 中央 / 右 / 状态）：

| 场景 | activity | left | central | right | status | shell overflow |
| --- | --- | --- | --- | --- | --- | --- |
| 1440 主工作区 / Graph / command | `42×898` | `288×872` | `868×872` | `240×872` | `1396×26` | `1438 / 1438` |
| 1180 Search / Canvas | `42×758` | `260×732` | `656×732` | `220×732` | `1136×26` | `1178 / 1178` |
| 1180 左栏折叠 | `42×758` | `1×732`* | `916×732` | `220×732` | `1136×26` | `1178 / 1178` |
| 900 双侧栏 / quick-open | `42×758` | `220×732` | `436×732` | `200×732` | `856×26` | `898 / 898` |
| 900 右栏折叠 | `42×758` | `220×732` | `636×732` | `1×732`* | `856×26` | **`899 / 898` GAP** |

\* 原始 `1 px` 是零 grid track 外的物理 CSS 边界；JSON 保留原始数值，同时以 `aria-hidden`、`inert` 和 `interactiveChildren: 0` 证明折叠状态。它不是把侧栏内容保留在 1 px 宽轨中。

九张截图的 html/body 都没有横向或主页面纵向溢出。81 条几何合同中 `80 PASS / 1 GAP`，唯一失败为第 08 图的 `shell-no-overflow`。内部真实 wheel 量尺：

- 1440 中央内容：`3424 / 850`，wheel 后 `scrollTop 0 → 480`，复位 `0`；
- 1440 右栏：`1395 / 872`，wheel 后 `0 → 480`，复位 `0`；
- 1180 合成来源左栏：`904 / 732`，wheel 后 `0 → 172`，复位 `0`。

## 5. 键盘、焦点与 reduced motion

八项合同由七组实际浏览器检查覆盖（command 的“初始焦点”和 Escape 回焦合为一组），结果 `7 / 7 PASS`：

1. Tab 在 64 次采样内到达 activity、left、central、right 四区；
2. 左栏折叠后 `aria-hidden=true`、`inert=true`、可交互子项 `0`，其后 64 次 Tab 不进入该栏；活动栏“来源”可恢复；
3. 右栏折叠后同样移出交互与辅助技术路径；活动栏“上下文”可恢复；
4. command overlay 首个可操作项在 dialog 内获得焦点；Escape 后回到 `Syn 命令`，`focus-visible` 为 `2 px solid`；
5. quick-open 同样在 dialog 内进入焦点，Escape 后 50 ms 回到**同一个**左栏触发按钮；
6. 键盘到 activity 的焦点可见，`搜索` 按钮为 `2 px solid`；
7. `prefers-reduced-motion: reduce` 下 command overlay 与回焦仍可完成。

## 6. 规定验证

| 验证 | 实际结果 |
| --- | --- |
| 新夹具浏览器加载 | PASS：全场景 console error `0`、page error `0`、外部请求 `0`。 |
| `npm run typecheck` | PASS，exit `0`。 |
| 聚焦 `knowledge-workbench-shell` 合同 | PASS：`knowledge workbench shell static convergence contract passed`。 |
| `npm run test:offline-interaction` | PASS，exit `0`。runner 登记 **37** 个 entry point；`offline interaction tests passed: 15` 是首个测试自身的断言输出，非 runner 数量。 |
| `node scripts/harness/workbench-shape-gate.js --mode baseline` | PASS，`Errors: 17 / Warnings: 5 / Info: 5`，为既有工作树债务。 |
| `node scripts/harness/workbench-shape-gate.js --mode check` | 非零且如预期 FAIL，exit `1`，同为 `17 / 5 / 5`；本包没有引入类别。 |
| `git diff --check` | PASS。 |
| `git diff --cached --name-only` | PASS：空。 |
| 窄返工 Search 结果态（仅一次 fresh context） | PASS（执行/隔离合同）：`1180×760`、查询 `合成`、`resultCount=16`、`localStorageEmptyBeforeMount=true`、`knowledge_workspace_search=1`、外部请求/写调用/未识别调用均为 `0`；截图为 `10-1180-search-results-preview.png`。 |

## 7. 差距矩阵与建议后续面

| 等级 | 已证实差距 | 精确复现场景 / selector / 证据 | 建议的后续写入面（均须另行新包授权；本 R2 不施工） |
| --- | --- | --- | --- |
| P0 | 无。隔离、fresh context、零外部/写/未识别调用、焦点/回焦及量尺事实有效。 | §2、§4、§5 与补图 10。 | — |
| P1 | 右栏折叠时 shell 多出 `1 px` 横向宽度，违反 shell 无 overflow 合同。 | `08-900-right-collapsed.png`，`900×760`；`section[data-knowledge-shell="syn-workbench"].syn-knowledge-shell.is-right-collapsed`；`clientWidth=898`、`scrollWidth=899`，右 aside 原始边界 `x=899,width=1`。 | 首选 `prototypes/productized-desktop-shell/src/styles.css`；若仅样式不能消除 grid/border 边界，再评估 `src/views/knowledge/KnowledgeWorkbenchShell.tsx` 和对应合同测试。 |
| P1 | Search 结果态与冻结 R0 不一致，且初始 04 未执行搜索。补图 10 证明真实合成查询/结果，但当前结果位于 quick-open overlay，不是 R0 03 的左栏结果列表。 | 初始 `04-1180-search-preview.png`、补图 `10-1180-search-results-preview.png`，`1180×760`；`.syn-knowledge-search-panel`、`.native-workspace-quick-open`、`.native-workspace-search-results`。 | `src/views/knowledge/NativeKnowledgeWorkspace.tsx`、`src/styles.css`、搜索/overlay 合同测试。 |
| P1 | command 与 quick-open 缺少 R0 的可搜索结果列表、当前选择和键盘提示；03 是新建表单，09 为空，10 只有结果清单。 | `03-1440-command-overlay.png`、`09-900-quick-open-overlay.png`、`10-1180-search-results-preview.png`；`.syn-knowledge-overlay[aria-label]`、`.native-workspace-command-controls`、`.native-workspace-command-kind`、`.native-workspace-search-results`。 | `src/views/knowledge/NativeKnowledgeWorkspace.tsx`、`src/styles.css`、overlay/键盘合同测试。 |
| P1 | R0 的关闭/新增、多标签组与分栏关系未被现有 R2 截图或交互矩阵覆盖；当前 01/07 只呈现双层且单组标签。 | `01-1440-markdown-preview.png`、`07-900-double-sidebar-markdown.png`；`.syn-knowledge-central-tabs`、`.native-workspace-tabs`。 | 需先在新包内明确产品行为；候选读取面为 `src/lib/knowledgeWorkbenchLayout.ts`、`src/views/knowledge/KnowledgeWorkbenchShell.tsx`、`src/views/knowledge/NativeKnowledgeWorkspace.tsx`、`src/styles.css` 与合同测试。 |
| P1 | Canvas 仍是 catalog/创建表单/工具条/JSON inspector 的组合，非 R0 canvas-first 连续空间和浮动工具。 | `05-1180-canvas.png`、`06-1180-left-collapsed.png`；`.native-knowledge-canvas`、`.native-canvas-browser-grid`、`.native-canvas-catalog`、`.native-canvas-toolbar`、`.native-canvas-inspector`。 | `src/views/knowledge/KnowledgeCanvasView.tsx`、`src/styles.css`，必要时 `NativeKnowledgeWorkspace.tsx` 与 Canvas 合同测试。 |
| P2 | 活动栏为可断行文字按钮，R0 是紧凑 icon ribbon，900 宽度的可扫读性和密度下降。 | `01-1440-markdown-preview.png`、`06-1180-left-collapsed.png`、`07-900-double-sidebar-markdown.png`、`08-900-right-collapsed.png`；`.syn-knowledge-shell__activity > button`。 | `src/views/knowledge/NativeKnowledgeWorkspace.tsx`、`src/styles.css`；需保留可访问名称/焦点合同。 |
| P2 | Graph 为大矩形卡片阵列，不是 R0 轻量节点/连线网络。 | `02-1440-graph.png`；`.native-knowledge-graph`、`.native-graph-flow-stage`、`.native-graph-node`。 | `src/views/knowledge/KnowledgeGraphView.tsx`、`src/styles.css`、Graph 合同测试。 |
| P2 | 右栏采用长列表式上下文，缺少 R0 的标签/折叠层级和信息节奏。 | `01-1440-markdown-preview.png`、`04-1180-search-preview.png`、`05-1180-canvas.png`、`07-900-double-sidebar-markdown.png`；`.syn-knowledge-shell__right` 及其 workspace context 子区。 | `src/views/knowledge/NativeKnowledgeWorkspace.tsx`、`src/styles.css` 与上下文层级合同测试。 |

若指导线另行派发 R3，应优先把每项 P1 拆成可验收的独立视觉/行为合同，且把 `900×760` 的 document/body/shell 无横向溢出保留为独立硬门。上述仅是基于现图的候选写入面，不授权或启动 R3；本次隔离证据不得表述为真实 App 验收。

## 8. 新 catch 与回交边界

初次执行线回交时自报“零新 catch”，因为当时没有新增 `docs/harness-catch-log.md` 条目；该表述随后被 §9 的指导独立复核覆盖。指导线已登记“几何绿灯替代 R0 视觉逐图对照”为一个 R2 catch。本次窄返工没有发现另一项需要追加的真实 catch，且未写入该 catch log。

## 9. 指导复核裁决（2026-07-26）

指导线接受本 evidence 的隔离、量尺、焦点、零写调用、生产 hash 未漂移和 `1 px` overflow 事实；不接受“其余截图均为视觉 PASS”及“`styles.css` 是唯一建议 R3 面”的结论。

直接对照 R0 冻结实图后确认：Search 结果态未实际执行，command/quick-open 不是冻结参考中的命令/结果列表形态，文字活动栏、Graph 卡片阵列、Canvas 常驻表单/JSON 分区、标签组与右栏上下文层级仍存在多项可见差距。精确复核和窄返工要求见任务包 §13。

当前裁决为 **`NEEDS_MODIFICATION / NOT_ACCEPTED`**。R3 尚未形成或派发。

指导线已将“几何绿灯替代 R0 视觉逐图对照”的漏判登记为新的 R2 catch。

## 10. §9 窄返工实际执行（2026-07-26）

- 已先重读任务包 §13 与本 evidence §9；复核时 HEAD 仍为 `e9ad7f3a204a1ebb11ce26c1e8c05b19c04c0991`、暂存区为空、7 个冻结生产读取面的 SHA-256 均仍匹配。启动前 `5173` 空闲。
- 没有修改 fixture、任何产品 `src/**`、Rust、App、`CURRENT.md`、`AUTHORITY.md`、计划或 catch log。本窄返工只写入本 evidence、本任务包、`measurements.json` 和 synthetic-only 的 `10-1180-search-results-preview.png`。
- 在唯一新增的 `1180×760` fresh browser context 中，实际从活动栏进入 Search/quick-open、向 `#native-knowledge-search` 输入合成查询 `合成` 并执行查找。结果为 `resultCount=16`，且记录为 `localStorageEmptyBeforeMount=true`、`knowledge_workspace_search=1`、外部请求 `0`、写调用 `0`、未识别调用 `0`、console/page error 均为 `0`。没有声称 mount 后 localStorage 仍为空。
- 未重跑 initial R2 的 typecheck、offline runner、shape 或焦点矩阵；§13 仅授权这一次补图与关闭资源后的门禁复核。初始 R2 的历史结果保留在 §6，不被本轮重写或扩大。
- 本线启动的 Chromium context 与 Vite 均已关闭；最终 `5173` 无 listener，`git diff --check` exit `0`，`git diff --cached --name-only` 为空。

本执行线只完成 R2 窄返工和证据更正，状态仍为 **`NEEDS_N2R_R3_VISUAL_CONVERGENCE / NOT_ACCEPTED`**；不自行宣称指导接受，也没有启动 R3。

## 11. 指导最终验收（2026-07-26）

指导线独立查看新增 Search 结果态截图和 `measurements.json`，并复核修正后的逐图判定与 P0/P1/P2 差距矩阵。补图中的查询、16 项结果、一次 `knowledge_workspace_search`、mount 前空 localStorage，以及零外部请求、零写调用、零未识别调用、零 console/page error 均相互一致；该截图仍被正确判为与 R0 左栏 Search / quick-open 形态存在差距。

修正后的 §3/§7 已足以支撑 R3 拆包：`1 px` overflow、Search/command/quick-open、标签组/分栏、Canvas、文字活动栏、Graph 与右栏层级均有明确证据和候选写入面，没有继续用几何绿灯替代视觉对照。§9 的 `NEEDS_MODIFICATION / NOT_ACCEPTED` 是窄返工前的历史裁决，现由本节覆盖。

指导最终结论为 **`ACCEPTED_N2R_R2_BASELINE / NEEDS_N2R_R3_VISUAL_CONVERGENCE / NOT_REAL_APP_ACCEPTED`**。这只接受 synthetic-only 基线、隔离事实和差距审计；不接受当前界面为高保真完成，不代表真实 Syn/Tauri App、真实 store/vault、N6 或发布验收。R3 尚未获授权或派发。
