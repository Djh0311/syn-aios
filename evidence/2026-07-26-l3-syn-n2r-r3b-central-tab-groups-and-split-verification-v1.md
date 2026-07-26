# L3 Syn N2R-R3B 中央标签组与左右分栏验证 evidence v1

- 日期：2026-07-26
- 最终状态：**`ACCEPTED_N2R_R3B_CENTRAL_TAB_GROUPS_AND_SPLIT / NOT_REAL_APP_ACCEPTED`**
- 证据级别：真实 React、真实生产 CSS、pure-synthetic fixture、fresh localhost browser context；**不是**真实 Syn/Tauri App、真实 store/vault、完整 R0、Gate 0 或发布验收。
- 执行线只回交 R3B 改动与证据，不自行验收，不进入 R3C。

## 1. 开工 preflight、权限与 provenance

- 固定工作目录：`/Users/yoyi/workspace/product-line`。
- HEAD：`e9ad7f3a204a1ebb11ce26c1e8c05b19c04c0991`，与任务包冻结值一致。
- `git diff --cached --name-only`：开工无输出，staged 为空。
- `lsof -n -P -iTCP:5173 -sTCP:LISTEN`：开工无输出，5173 无 listener。
- 写所有权：用户明确派发本独立知识前端执行线；白名单逐文件未见其他 open handle，没有发现并行 writer。
- 已完整读取 `AGENTS.md`、`CURRENT.md`、`AUTHORITY.md`、唯一施工包和冻结设计参考，并按 `interface-design` 检查点保持低矮单排标签、克制当前组指示、既有 token、无新卡片/渐变/重阴影。
- 未启动真实 Syn/Tauri/Codex/MCP/store/vault，未使用真实知识数据。

写入前 18 个冻结 SHA-256 全部匹配：

| 文件 | 冻结 SHA-256 | 开工 |
| --- | --- | --- |
| `src/lib/knowledgeWorkbenchLayout.ts` | `ad2284fc25fee024fc7b8d71677719dd72802f4ae14b0ef3afbeadb497aaef4f` | MATCH |
| `src/lib/knowledgeWorkspace.ts` | `80114e9582410f395abe033ec268144b5917ca2e9f4aa0e5869b413cd75d9c8b` | MATCH |
| `src/views/knowledge/KnowledgeWorkbenchShell.tsx` | `2ab66a5b00941e3cf0a083e3ed3d24b309f8b7fe5037dd41d8743cf9d2d41c21` | MATCH |
| `src/views/knowledge/NativeKnowledgeWorkspace.tsx` | `f4f1c5aed802e66ae3418460f3b5ff1a9a2fe33fddd773ab748e9c2f63025fe5` | MATCH |
| `src/views/knowledge/KnowledgeGraphView.tsx` | `ca8d88c3183913bd946d2bef35139fce7936f8b071809d4a96f4a573cc0b6b83` | MATCH |
| `src/views/knowledge/KnowledgeCanvasView.tsx` | `af336f7c1176a60791e003f3162456bb31ae6ea6c5cbb6672b3fafafaa973c9c` | MATCH |
| `src/styles.css` | `1e846799b94724a26f3175a24dbf6c5751527113d3b22c57add0c520a6762484` | MATCH |
| `tests/knowledge-workbench-shell.test.tsx` | `30727a05000561f0d9812c385a8a29fb4d199a35abbd1ba2d33ecbe552aebad2` | MATCH |
| `tests/native-knowledge-workspace.test.tsx` | `fdcdfe54fb8f6dd022d4b018635d3bca1f2cd75874f2a835859dacfaa406408e` | MATCH |
| `tests/knowledge-graph.test.tsx` | `d4d87d8b914bac2708eea3c86599d6da8373c2aa8b60b75a70af5d3050e98b16` | MATCH |
| `tests/knowledge-canvas.test.tsx` | `53481206e5aa70f9b21ee677602ac2cbece70e5cfadbcc9ec58e5998c5f5aab6` | MATCH |
| `tests/knowledge-workbench-visual-fixture.tsx` | `1e590ac9d040d4e9486b8afe8171526f0c9dbecb38a419bfb0d1d9ad07523b30` | MATCH |
| `tests/knowledge-workbench-visual-fixture.html` | `8a58ac0d02d9c33a9ca151f4ec1e653ccd3c6d79096581efd339a62c1fcec23c` | MATCH |
| `scripts/run-offline-interaction-test.mjs` | `798daf13722875befa22eb233c476b957d241005a3b08267ed30eb54e0363082` | MATCH |
| R0 设计参考 | `141ff343312fff91346e6e170f1aea2d07cda5fa2abdb3fde8b840b16f9ed4d9` | MATCH |
| R2 evidence | `8dd432266f406e96c69359196ea138eede85891d041ed8f98a3907eed55b6c7a` | MATCH |
| R3A-R1 task | `5b90b5a52fc0d6eccce7712f0da073033c43db5c9c711d412f71a4c17d51e893` | MATCH |
| R3A-R1 evidence | `02cb23f7a37d5390a1688fc7a1b9cc221b74c4cd2ac4ce841711a8df352a0a7a` | MATCH |

表内前 14 个路径均相对 `prototypes/productized-desktop-shell`，后 4 个相对仓库根。开工 worktree 已含上游 dirty/untracked WIP；本包没有 reset、clean、stash、checkout、覆盖或批量 staging。

## 2. Red-first

在产品实现前先把 R3B 纯函数与静态 DOM 合同加入既有 `knowledge-workbench-shell` 入口，再 bundle/import：

- 原始报告：[red-first-contracts.json](raw/2026-07-26-l3-syn-n2r-r3b-tab-groups-split/red-first-contracts.json)
- 实数：**16 项 / 16 失败 / 0 通过**，exit `1`，结果为 `EXPECTED_RED`。
- 失败类别：单组/当前标签、同草稿源码预览分栏、比例/合并、第三组与非有限比例 fail-closed、Markdown 去重、关闭邻项与确定焦点、空组工具、Graph/Canvas/Maintenance 单例、`⌘T`/`⌃Tab`、分隔器键盘、v1 迁移、非法 v2 normalization、dirty/conflict 保护、storage fail-soft、旧全局标签/内部分栏 DOM 退场、真实 tablist 与组工具分离。

实现后同一入口输出：`knowledge workbench shell static convergence contract passed; R3B 16 / 0`，即 **16 项 / 0 失败**。

## 3. 最终中央组状态与交互

### 3.1 单一状态模型

- 中央模型只有 `groups / activeGroupId / splitRatio`；组 id 固定为 primary/secondary，最多 `2` 组。
- 每组只有一条有序真实标签数组和一个 `activeTabId`。Markdown 标签携带 `relativePath + source|preview`，Graph/Canvas/Maintenance 为中央 surface 标签。
- Markdown 路径在组内去重；Graph、Canvas、Maintenance 各自在整个中央工作区保持单例，已有实例只被聚焦/迁移，不复制。
- split 默认 `50`，边界 `30–70`；pointer drag 与 separator 的 ArrowLeft/ArrowRight 共用同一比例动作，NaN/Infinity fail closed。
- merge 保留标签顺序和当前 Markdown；超过两组、无效组 id、无效标签和无效比例都由纯 reducer/normalizer 收敛。

### 3.2 同一 Markdown 草稿的双投影

- 两组可同时显示同一路径：左组 `source`、右组 `preview`。
- 两侧共同消费现有唯一 `selected + draft + mtime/hash/conflict`；最终 DOM 最多 `1` 个 textarea 和 `1` 个保存按钮。
- 切换一侧投影不会创建第二草稿。若尝试把同一路径两侧都切成源码，另一侧确定性收为预览。
- 打开/关闭 Markdown 后，可见的同路径投影同步到新的当前草稿；不支持、也未实现两个不同 Markdown 草稿并行编辑。

### 3.3 焦点、ARIA 与 dirty/conflict

- 每组恰有一个 `role=tablist`；`+`、投影、分栏/合并和刷新是组工具，不进入 tablist。
- 标签采用 roving `tabIndex`、稳定 DOM id、`aria-selected` 与 `aria-controls`；每个受控 `tabpanel` 均真实存在，非当前 panel 用 `hidden` 退场。
- separator 为垂直 `role=separator`，带 `aria-valuemin=30`、`aria-valuemax=70`、实时 `aria-valuenow`。
- 打开、切换、关闭、合并后，焦点确定地落到目标 tab 或组工具；`⌘T` 打开 quick open，`⌃Tab` 循环当前组标签。
- dirty 或 conflict 时，切换文档、关闭当前脏标签和不安全合并均 `preserve`，不丢草稿、不绕过既有冲突保护。
- 最后一个标签可关闭为空组；空组仍保留可聚焦的打开/分栏工具。

### 3.4 偏好迁移

- 继续使用既有可丢弃 UI key `syn-native-knowledge-workspace-ui-v1`，未新增 store 或 sidecar。
- 内容升级为 `version: 2`，只写 `selectedRelativePath` 和规范化的中央 chrome 状态，不写 Markdown body/draft。
- legacy v1 的 `tabs + viewMode=split` 确定性迁移为两个组中同一路径的 source/preview；普通 source/preview 迁移为单组。
- v2 normalization 限制两组、最多 12 个安全 Markdown 路径、已知 surface 单例、合法当前组/标签与 `30–70` 有限比例；坏 JSON、异常 storage、非法路径均回落默认，不阻断启动。

## 4. Synthetic-only 浏览器证据

- runner：[browser-evidence.mjs](raw/2026-07-26-l3-syn-n2r-r3b-tab-groups-split/browser-evidence.mjs)
- 原始报告：[browser-evidence.json](raw/2026-07-26-l3-syn-n2r-r3b-tab-groups-split/browser-evidence.json)
- 最终：**6 个 fresh context，90 / 90 PASS，0 失败，9 张实图**。
- 每个 context mount 前均 `localStorageEmptyBeforeMount=true`。
- 精确 read mock allowlist：`knowledge_workspace_graph`、`knowledge_workspace_read_canvas`、`knowledge_workspace_read_markdown`、`knowledge_workspace_search`、`knowledge_workspace_snapshot`。
- 六组 context 合计真实 Tauri 写调用 `0`、未识别调用 `0`、外部请求 `0`、console error `0`、page error `0`。
- 本地浏览器写入只含可丢弃 key `syn-native-knowledge-workspace-ui-v1`；各 context 次数为 `4 / 5 / 4 / 4 / 4 / 16`，最新内容均为规范化 `version: 2` chrome 偏好，无 Markdown body/draft。

| 场景与实图 | 量尺/状态 | 焦点、ARIA 与隔离结论 |
| --- | --- | --- |
| [01 1440 源码/预览](raw/2026-07-26-l3-syn-n2r-r3b-tab-groups-split/01-1440-markdown-source-preview.png) | `1440×900`；中央 `868×872`；2 组，tabs `1/1`，比例 `50`；textarea/save `1/1` | 左源码、右同草稿预览，live dirty marker 可见；焦点在源码 textarea；2 tablist、tab↔tabpanel 全关联；document/body/shell 无横向 overflow。 |
| [02 1180 Markdown/Graph](raw/2026-07-26-l3-syn-n2r-r3b-tab-groups-split/02-1180-markdown-graph.png) | `1180×760`；中央 `656×732`；2 组，tabs `1/2`，比例 `50`；textarea/save `1/1` | Graph 作为右组当前 tabpanel，不是壳顶伪标签或纵向追加页面；焦点在 Graph tab；零写/零错误。 |
| [03 900 双侧栏](raw/2026-07-26-l3-syn-n2r-r3b-tab-groups-split/03-900-double-sidebar-split.png) | `900×760`；中央 `416×732`；两侧栏展开；2 组，比例 `50` | 单排标签和组工具仍可见，长路径省略但保留完整 title/ARIA；document/body/shell 横向 overflow 均 false。 |
| [04 900 右栏折叠](raw/2026-07-26-l3-syn-n2r-r3b-tab-groups-split/04-900-right-collapsed-split.png) | `900×760`；右栏宽 `0`；2 组 | 右栏 `aria-hidden=true`、`inert=true`、可交互后代 `0`；中央接管空间；焦点在活动栏右栏切换按钮。 |
| [05 900 双栏折叠](raw/2026-07-26-l3-syn-n2r-r3b-tab-groups-split/05-900-both-collapsed-split.png) | `900×760`；中央 `856×732`；左右栏宽均 `0`；2 组 | 左右栏均 hidden/inert、可交互后代均 `0`；活动栏保留；document/body/shell 无横向 overflow。 |
| [06 比例 60](raw/2026-07-26-l3-syn-n2r-r3b-tab-groups-split/06-1180-split-ratio-60.png)、[07 合并](raw/2026-07-26-l3-syn-n2r-r3b-tab-groups-split/07-1180-merged-single-group.png)、[08 快开第二标签](raw/2026-07-26-l3-syn-n2r-r3b-tab-groups-split/08-1180-quick-open-second-tab.png)、[09 脏关闭拒绝](raw/2026-07-26-l3-syn-n2r-r3b-tab-groups-split/09-1180-dirty-close-rejected.png) | `1180×760`；separator `50→60`，ARIA `30/70/60`；merge 后 1 组、textarea `1`；`⌘T` 后 2 tabs；`⌃Tab` 循环 | merge 后焦点回 tab；quick open 打开并聚焦第二 Markdown；脏关闭后标签与 draft marker 均保留，提示可见，没有真实保存调用。 |

浏览器工具说明：通用 Playwright wrapper 的 `--help` 在本机未返回，已终止该探测；没有安装依赖，改用仓库现有 `playwright-core` 执行本包 raw runner。文件系统 sandbox 内 Vite 监听与 Chrome sandbox launch 分别被 `EPERM`/sandbox 拦截后，按授权在本机边界启动；只访问 `127.0.0.1:5173`，结束时只关闭本包 Vite/浏览器。

## 5. R0 01/02/04 逐图人工对照

| R0 | 本包结论 | 视觉判断与未关闭 GAP |
| --- | --- | --- |
| 01 workspace / note / graph / backlinks | **PASS（R3B 中央组关系）/ GAP（完整界面）** | 新图 01/02 显示中央只有一条真实标签组带，两个等高内容组由细分隔器连接，当前组用克制顶边区分；Markdown 与 Graph 都占现有组。活动栏图标、右栏内容和全屏像素高保真不在本包。 |
| 02 reading / graph / backlinks | **PASS（R3B 阅读/Graph 同组语义）/ GAP（完整界面）** | 新图 01 的右侧预览与新图 02 的右组 Graph 都是 tabpanel，不再有壳顶模式排或文档内第二层 split。右栏反链等内容视觉仍未收敛。 |
| 04 canvas / graph / context | **GAP** | Graph 进入中央组已由新图 02 直接证明，Canvas 进入组由纯合同和既有 Canvas 定向合同覆盖；本包没有 Canvas 浏览器实图，也明确不重做 Graph/Canvas 内容视觉，因此不能把 R0 04 整图报成视觉 PASS。 |

人工看图与 DOM/量尺结论一致；DOM/量尺没有被拿来替代上述视觉判断。本包只关闭中央标签组/左右分栏 P1，不声称 Obsidian 1:1 或 N2R 全界面高保真。

## 6. 必跑门禁与验证过程

| 验证 | 实际结果 |
| --- | --- |
| red-first `knowledge-workbench-shell` | EXPECTED RED：`16 / 16` 失败。 |
| green `knowledge-workbench-shell` | PASS：`R3B 16 / 0`。 |
| `native-knowledge-workspace` 定向合同 | PASS：既有 N0 compatibility 与 N1–N5 fixed client contracts。 |
| `knowledge-graph` 定向合同 | PASS：static shell 与 typed handoff。 |
| `knowledge-canvas` 定向合同 | PASS：static shell、typed calls 与 local JSON Canvas patch。 |
| `npm run typecheck` | PASS，exit `0`。 |
| `npm run test:offline-interaction` | 最终 PASS，exit `0`；冻结 runner 精确登记 **37** 个 entry，R3B 未新增入口。首轮曾由既有 `knowledge-vault-notes` 静态文案兼容断言拦下；在 `NativeKnowledgeWorkspace.tsx` 白名单内保留准确的“真实标签页/向右分栏阅读/源码/预览”可见说明后，最终 37 项全过。 |
| synthetic browser | PASS：`90 / 90`，6 contexts，9 screenshots。 |
| shape baseline | PASS：`Errors 17 / Warnings 5 / Info 5`。 |
| shape check | 预期非零 exit `1`：仍为 `17 / 5 / 5`，与 baseline 及上游既有类别/finding 一致，零净增。实施中间态曾因 `NativeKnowledgeWorkspace.tsx=2069` 行出现第 18 个 `file_over_limit_not_in_ratchet`；仅压缩本包新增排版、保持语义不变，最终 `1978` 行，该新增 finding 消失。 |
| hardcoded-hex selftest | PASS：`13 / 13`。 |
| machine-face selftest | PASS：`18 / 18`。 |
| `git diff --check` | PASS，无输出。 |
| `git diff --cached --name-only` | PASS，无输出，staged 为空。 |
| 进程收尾 | 只关闭本包启动的 Vite/浏览器；最终 5173 无 listener。 |

最终 shape 的 17 个 error、5 个 warning、5 个 info 是既有全仓债务；其中包括既有 ratchet 文件、超限 Rust/TS 文件和未知 sidecar kind。本包不修、不掩盖，也不把 shape check 的非零退出伪报为全绿。

## 7. 实际写入文件

产品与测试：

1. `prototypes/productized-desktop-shell/src/lib/knowledgeWorkbenchLayout.ts`
2. `prototypes/productized-desktop-shell/src/lib/knowledgeWorkspace.ts`
3. `prototypes/productized-desktop-shell/src/views/knowledge/NativeKnowledgeWorkspace.tsx`
4. `prototypes/productized-desktop-shell/src/styles.css`
5. `prototypes/productized-desktop-shell/tests/knowledge-workbench-shell.test.tsx`
6. `prototypes/productized-desktop-shell/tests/knowledge-workbench-visual-fixture.tsx`

证据与控制：

7. `evidence/raw/2026-07-26-l3-syn-n2r-r3b-tab-groups-split/red-first-contracts.json`
8. `evidence/raw/2026-07-26-l3-syn-n2r-r3b-tab-groups-split/browser-evidence.mjs`
9. `evidence/raw/2026-07-26-l3-syn-n2r-r3b-tab-groups-split/browser-evidence.json`
10. 上述 9 张 PNG。
11. 本 evidence。
12. 本任务包：只回写实际状态与执行结果。

`tests/native-knowledge-workspace.test.tsx` 在白名单内但最终未修改，SHA 仍为冻结值。未修改 Shell、Graph/Canvas/Maintenance 产品组件、Graph/Canvas 测试、fixture HTML、runner、Rust/Cargo、App、CURRENT、AUTHORITY、计划、决策、依赖、真实 store/vault 或 `.codex`。未 stage、commit、push、reset、clean、stash 或 checkout。

最终产品/测试关键 SHA-256：

| 文件 | 最终 SHA-256 |
| --- | --- |
| `src/lib/knowledgeWorkbenchLayout.ts` | `8fb9faf01ef9dae509a6cffa4e14fd62031bb9744ea8e44eb1d1476e4f9ee220` |
| `src/lib/knowledgeWorkspace.ts` | `81eddc02206035a9d1bf1e3954d5ea19b1864a71b27ffea43efafa5a2fd51fb0` |
| `src/views/knowledge/NativeKnowledgeWorkspace.tsx` | `eafd9edccc3a2cd62e99cfa211809d77b206c826907ca07976ef4c419f4b4464` |
| `src/styles.css` | `0cf85dbd37ca02c17143f6b43c1bb7500d30520238680fa0c770993fe07b3bf0` |
| `tests/knowledge-workbench-shell.test.tsx` | `2cbf2b61a86731096a876636c27119066c0e10d5bfaac81e8c6c7322660541da` |
| `tests/native-knowledge-workspace.test.tsx` | `fdcdfe54fb8f6dd022d4b018635d3bca1f2cd75874f2a835859dacfaa406408e` |
| `tests/knowledge-workbench-visual-fixture.tsx` | `219637ff4493150ea4fd0aac7e64cb6e232f2840d80560208da7e014d8d63fee` |

关键 raw SHA-256：

| 产物 | SHA-256 |
| --- | --- |
| red JSON | `7e829862ac0c9806519408df82f8b634d516643fb7e62521456cb43f35c1814b` |
| browser runner | `087653734143ef34edda722b8b097f02cd0f8ca64111f688a21c8a255dc0e736` |
| browser JSON | `fd114fefc9895ca6f9f01d08821ceb2e30dbf27c17c8fef217839265b1bd8519` |
| 01 | `ccddc38edf39ba5f9e531d1502368c4d87db5f62681df4afe1d2138f2fff4c5e` |
| 02 | `10fd390e7df9d6e09d223c74c304a77e3f53c6817334fc562a7cc2f0b1572bf9` |
| 03 | `8af7b48b7797b5f9ad13f457d3a4a4924946da3493fee3a0b90629636c04391c` |
| 04 | `58b76e4a9aa6020f3381ad44d760f36fca39808dae4d958d4ec136a54adcf804` |
| 05 | `ca3ab37023055eba272c3e1d4bb867c392eb17f708c8471cb13d0c6730cd56f8` |
| 06 | `b5840e70d67ab20df4d9b8b004300f6c1d1b38602c0342e22e8df9d0dd77808f` |
| 07 | `8ed64cabf1997c8987b65068f362010dc2b909816658b430e23db39e87306d2e` |
| 08 | `9ca84472217169cc6ab86d54573ed687f3fd1fb41fa222acb1abfd91344a256a` |
| 09 | `b379e31e6582058140f5d7f40b291cb2f2b21785daea087b06a0582baa4054c7` |

## 8. 新 catch、遗留与停止边界

- **零新 catch**。red-first、首轮 aggregate compatibility failure 与 shape 中间态 finding 都被既有门禁按预期拦截，并在本包白名单内收敛；没有发现需要追加全局 catch log 的新机制问题。
- shape `17 / 5 / 5` 是既有债务，本包零净增。
- R0 04 的 Canvas 实图、Graph/Canvas 内容视觉、活动栏、右栏内容、标签拖拽重排、向下分栏、两个不同 Markdown 草稿并行编辑、任意数量分栏均未完成，也不在本包。
- 未执行、未接受：R3C、真实 Syn/Tauri App、真实 store/vault、Home-only discovery、Gate 0、十二项或发布验收。
- 没有触发必须修改冻结 Graph/Canvas/Maintenance/Rust、引入第二草稿、新依赖或真实数据的停止条件。

执行线精确回交：

**`PASS_N2R_R3B_CENTRAL_TAB_GROUPS_AND_SPLIT / NEEDS_GUIDANCE_REVIEW / NOT_REAL_APP_ACCEPTED`**

## 9. 指导最终验收（2026-07-26）

指导线没有采信执行线自报，而是独立复核实际状态模型、React 写路、CSS、合同实现、raw 浏览器报告和九张截图。结果确认：

- 中央旧全局模式排和文档内部伪 split 已退出；每组只有一排真实 tablist，最多两个左右组；
- 两个 Markdown 投影只读取同一个 `selected + draft + mtime/hash + conflict`，页面最多一个 textarea 和一个保存动作；
- 关闭邻居、组工具回焦、separator ARIA/键盘、quick-open/循环切换与 dirty/conflict preserve 路径均有实现和直接证据；
- 六个 fresh context 的 `90 / 90`、零真实 Tauri 写、零未知调用、零外部请求和零浏览器错误与 raw JSON 一致；localStorage 只写一个可丢弃 UI 偏好 key；
- R0 01/02 只在中央组关系维度 PASS；整页密度、活动栏、右栏、Graph 内容和 R0 04 Canvas 仍为 GAP，不能宣称 Obsidian 1:1 或 N2R 完成。

指导线独立复跑 typecheck 与冻结 37-entry offline runner，均 exit `0`；shape baseline 为 `17 errors / 5 warnings / 5 info`，shape check 保持同一既有集合并按预期 exit `1`；两项 shape selftest 为 `13/13`、`18/18`。`git diff --check` 通过、staged 为空、HEAD 未变，5173 无 listener。

指导最终结论为 **`ACCEPTED_N2R_R3B_CENTRAL_TAB_GROUPS_AND_SPLIT / NOT_REAL_APP_ACCEPTED`**。这只接受 R3B 的 synthetic 浏览器与离线范围；不接受完整 R0、真实 Syn/Tauri App、真实 store/vault、N6、十二项或发布验收，也不授权 R3C。
