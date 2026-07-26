# 任务包：L3 Syn N2R-R3B 中央标签组与左右分栏收敛 v1

- 日期：2026-07-26
- 状态：**ACCEPTED_N2R_R3B_CENTRAL_TAB_GROUPS_AND_SPLIT / NOT_REAL_APP_ACCEPTED**
- 负责人：独立知识前端执行线
- 指导/验收：当前总指导对话
- 小阶段计划：`docs/plans/2026-07-23-l3-syn-native-knowledge-workspace-small-stage-plan-v2.md`
- 冻结参考：`docs/design/2026-07-25-l3-obsidian-core-desktop-reference-and-migration-r0-v1.md`
- 上游接受：`ACCEPTED_N2R_R3A_SEARCH_OVERLAY / NOT_REAL_APP_ACCEPTED`

## 1. 目标与完成标准

本包只关闭 R2 差距矩阵中的 **中央标签组/分栏 P1**：

1. 把当前“中央模式标签 + Markdown 文档标签 + 文档内部 split”三层叠加，收为一个真实中央标签组系统；
2. 保留既有多 Markdown 标签的打开、关闭、切换与未保存草稿保护；
3. 支持最多两个左右标签组、当前组、组内当前标签和可调整分隔比例；
4. Markdown 源码与预览可以在两个组中同步呈现同一草稿；编辑侧更新时，预览侧同步更新；
5. Graph、Canvas 和维护能力只能作为中央标签内容进入现有组，不再由壳顶另设一排伪标签；本包不重做它们的内容视觉；
6. 完成 red-first 合同、synthetic-only 浏览器实图/量尺、完整离线回归和 evidence。

完成后只能回交
`PASS_N2R_R3B_CENTRAL_TAB_GROUPS_AND_SPLIT / NEEDS_GUIDANCE_REVIEW / NOT_REAL_APP_ACCEPTED`
或 §9 的其他固定枚举。执行线不得自行验收、不得继续 R3C。

本包由用户在 2026-07-26 明确批准并由指导线派发。授权仅限本文白名单和 synthetic-only 浏览器验证。

## Authority and plan alignment

- authority_chain: `AGENTS.md` -> `CURRENT.md` -> knowledge/conversation parallel decision -> native knowledge route decision -> native knowledge small-stage plan -> frozen Obsidian R0 -> accepted R2 gap matrix -> accepted R3A -> this R3B task
- plan_anchor: `docs/plans/2026-07-23-l3-syn-native-knowledge-workspace-small-stage-plan-v2.md#r3-既有能力-交互与视觉收敛`
- existing_before_new: 已有 `tabs`、`selectedRelativePath`、`viewMode`、草稿冲突保护、单壳 reducer、Graph/Canvas 现成中央内容、quick-open 和 synthetic fixture；必须迁移这些状态，不再创建第二套标签数据源或第二编辑器根。
- capabilities_touched: none — 当前 partial Code Map 未登记知识子壳的中央标签组；本包不修改已登记 capability 的 canonical/entrypoint。
- forbidden_alternatives: 不用第二排模式按钮冒充标签组，不把两个全宽页面并排，不复制第二份 Markdown 草稿，不用 `overflow:hidden` 裁掉控件，不改 Graph/Canvas 内容视觉，不嵌入 Obsidian，不启动真实 Syn/Tauri/Codex/MCP/store/vault。

## 2. 设计意图与 R0 对照

### 2.1 使用者、任务与感受

- 使用者：在 macOS 上长时间浏览、编辑和关联知识的单人桌面用户。
- 核心动作：保留当前笔记上下文，在标签间切换，并把源码/预览或既有 Graph/Canvas 放到右侧组对照查看。
- 感受：紧凑、安静、连续；中央内容是唯一主焦点，标签和分隔器只服务于内容。

### 2.2 设计检查点

```text
Intent:     用户在同一工作台内切换或并排查看知识，不进入第二页面。
Hierarchy:  当前组的当前标签和正文胜出；非当前标签、组工具和分隔器退后。
Palette:    只使用现有 Syn 暖纸、墨色和青绿语义 token，不新增硬编码颜色。
Depth:      延续低对比边界与轻微表面差，不新增阴影系统、渐变、玻璃或大卡片。
Surfaces:   壳为 base，组内正文为 raised；当前标签与正文连成一个表面，非当前标签留在 soft 层。
Typography: 沿用现有 Syn 字体和桌面工具字号，长路径用省略号并保留可访问全名。
Spacing:    4px 基础单位；标签、组工具和分隔器保持 4/8/12px 的紧凑节奏。
```

- Domain：vault、笔记、标签、标签组、源码、预览、分隔器、草稿、焦点、上下文连续性。
- Color world：纸张、石墨、墨色、旧木、青玉状态；全部映射既有 token。
- Signature：同一份 Syn Markdown 草稿可在左右组中以源码和预览同步呈现，确认式写入与冲突保护不因视觉复刻而降级。
- Rejecting：
  - 顶部模式导航 + 文档标签双层堆叠 → 每个中央组只有一排真实标签；
  - 文档内部两列伪装分栏 → 两个具备独立 header、焦点与可调比例的真实标签组；
  - 两个独立 Markdown 草稿副本 → 单一草稿事实源、双组不同投影。

主参考：

- R0 `01-workspace-note-graph-backlinks-984x768.jpg`：两个中央标签组、单一壳和右栏关系；
- R0 `02-reading-graph-backlinks-984x768.jpg`：阅读态与 Graph 并排；
- R0 `04-canvas-graph-context-984x768.jpg`：Canvas/Graph 作为组内标签内容；
- R0 `08-left-sidebar-collapsed-ribbon-visible-984x768.jpg`：侧栏折叠后中央接管空间；
- R2 `01-1440-markdown-preview.png`、`07-900-double-sidebar-markdown.png`：当前双层模式/文档标签和文档内部 split 的差距证据。

本包不宣称完整 Obsidian 1:1；Graph、Canvas 内容视觉、活动栏、右栏和最终像素密度仍属后续包。

## 3. 开工冻结与并发

### 3.1 冻结基线

- HEAD：`e9ad7f3a204a1ebb11ce26c1e8c05b19c04c0991`
- staged：指导派发时为空；执行开工时必须再次为空。
- 端口：指导派发时 `5173` 无 listener；执行线开工时必须再次确认。

相对路径均以仓库根为准：

| 文件 | 指导冻结 SHA-256 |
| --- | --- |
| `prototypes/productized-desktop-shell/src/lib/knowledgeWorkbenchLayout.ts` | `ad2284fc25fee024fc7b8d71677719dd72802f4ae14b0ef3afbeadb497aaef4f` |
| `prototypes/productized-desktop-shell/src/lib/knowledgeWorkspace.ts` | `80114e9582410f395abe033ec268144b5917ca2e9f4aa0e5869b413cd75d9c8b` |
| `prototypes/productized-desktop-shell/src/views/knowledge/KnowledgeWorkbenchShell.tsx` | `2ab66a5b00941e3cf0a083e3ed3d24b309f8b7fe5037dd41d8743cf9d2d41c21` |
| `prototypes/productized-desktop-shell/src/views/knowledge/NativeKnowledgeWorkspace.tsx` | `f4f1c5aed802e66ae3418460f3b5ff1a9a2fe33fddd773ab748e9c2f63025fe5` |
| `prototypes/productized-desktop-shell/src/views/knowledge/KnowledgeGraphView.tsx` | `ca8d88c3183913bd946d2bef35139fce7936f8b071809d4a96f4a573cc0b6b83` |
| `prototypes/productized-desktop-shell/src/views/knowledge/KnowledgeCanvasView.tsx` | `af336f7c1176a60791e003f3162456bb31ae6ea6c5cbb6672b3fafafaa973c9c` |
| `prototypes/productized-desktop-shell/src/styles.css` | `1e846799b94724a26f3175a24dbf6c5751527113d3b22c57add0c520a6762484` |
| `prototypes/productized-desktop-shell/tests/knowledge-workbench-shell.test.tsx` | `30727a05000561f0d9812c385a8a29fb4d199a35abbd1ba2d33ecbe552aebad2` |
| `prototypes/productized-desktop-shell/tests/native-knowledge-workspace.test.tsx` | `fdcdfe54fb8f6dd022d4b018635d3bca1f2cd75874f2a835859dacfaa406408e` |
| `prototypes/productized-desktop-shell/tests/knowledge-graph.test.tsx` | `d4d87d8b914bac2708eea3c86599d6da8373c2aa8b60b75a70af5d3050e98b16` |
| `prototypes/productized-desktop-shell/tests/knowledge-canvas.test.tsx` | `53481206e5aa70f9b21ee677602ac2cbece70e5cfadbcc9ec58e5998c5f5aab6` |
| `prototypes/productized-desktop-shell/tests/knowledge-workbench-visual-fixture.tsx` | `1e590ac9d040d4e9486b8afe8171526f0c9dbecb38a419bfb0d1d9ad07523b30` |
| `prototypes/productized-desktop-shell/tests/knowledge-workbench-visual-fixture.html` | `8a58ac0d02d9c33a9ca151f4ec1e653ccd3c6d79096581efd339a62c1fcec23c` |
| `prototypes/productized-desktop-shell/scripts/run-offline-interaction-test.mjs` | `798daf13722875befa22eb233c476b957d241005a3b08267ed30eb54e0363082` |
| `docs/design/2026-07-25-l3-obsidian-core-desktop-reference-and-migration-r0-v1.md` | `141ff343312fff91346e6e170f1aea2d07cda5fa2abdb3fde8b840b16f9ed4d9` |
| `evidence/2026-07-26-l3-syn-n2r-r2-isolated-browser-visual-baseline-v1.md` | `8dd432266f406e96c69359196ea138eede85891d041ed8f98a3907eed55b6c7a` |
| `tasks/2026-07-26-l3-syn-n2r-r3a-r1-overlay-layering-rework-package-v1.md` | `5b90b5a52fc0d6eccce7712f0da073033c43db5c9c711d412f71a4c17d51e893` |
| `evidence/2026-07-26-l3-syn-n2r-r3a-r1-overlay-layering-rework-verification-v1.md` | `02cb23f7a37d5390a1688fc7a1b9cc221b74c4cd2ac4ce841711a8df352a0a7a` |

开工前必须：

1. 复核 HEAD、staged、上述 hash、5173 与白名单逐文件 provenance；
2. 确认没有其他任务写本包白名单，尤其是 `src/styles.css`、`NativeKnowledgeWorkspace.tsx`；
3. 若任一 hash 漂移、staged 非空、端口被占或写所有权不唯一，停止为对应 §9 枚举；
4. 不 reset、clean、stash、checkout、覆盖或批量格式化现有 WIP。

### 3.2 串行边界

- 本包独占 `src/styles.css`、`NativeKnowledgeWorkspace.tsx`、两个布局/偏好 helper 和视觉 fixture 写面。
- 对话底座线只可做静态只读准备；不得并发构建/启动真实 App、三句重验、MCP/CLI 或写共享运行资源。
- Canvas、Graph、活动栏、右栏后续视觉包不得与本包并发施工。

## 4. 精确写入白名单

### 4.1 产品与测试

- `prototypes/productized-desktop-shell/src/lib/knowledgeWorkbenchLayout.ts`
- `prototypes/productized-desktop-shell/src/lib/knowledgeWorkspace.ts`
- `prototypes/productized-desktop-shell/src/views/knowledge/NativeKnowledgeWorkspace.tsx`
- `prototypes/productized-desktop-shell/src/styles.css`
- `prototypes/productized-desktop-shell/tests/knowledge-workbench-shell.test.tsx`
- `prototypes/productized-desktop-shell/tests/native-knowledge-workspace.test.tsx`
- `prototypes/productized-desktop-shell/tests/knowledge-workbench-visual-fixture.tsx`：仅增加本包 synthetic 场景和量尺状态

`KnowledgeWorkbenchShell.tsx`、Graph/Canvas 产品组件及其测试、fixture HTML、runner 均为冻结只读面。本包现有两个测试入口已在 37-entry runner 内，不得为方便扩大 runner 或新增测试框架。

### 4.2 证据与控制文档

- `evidence/2026-07-26-l3-syn-n2r-r3b-central-tab-groups-and-split-verification-v1.md`
- `evidence/raw/2026-07-26-l3-syn-n2r-r3b-tab-groups-split/**`
- 本任务包：只回写实际状态、实际写入和执行结果
- `docs/harness-catch-log.md`：仅发现新的真实 catch 时追加

除此之外一律不写。不得修改 `App.tsx`、`KnowledgeBaseView.tsx`、Graph/Canvas/Maintenance 产品组件、任何 Rust/Cargo、依赖、CURRENT、AUTHORITY、计划、决策、真实 store/vault 或 `.codex`。

## 5. 实现合同

### 5.1 单一中央标签组状态

- 在现有 helper 中建立一个可纯函数验证的中央组模型；最多 `2` 个组，每组有稳定 id、顺序标签和当前标签，并有唯一 `activeGroupId`。
- Markdown 标签继续以安全相对路径为身份；Graph、Canvas、Maintenance 只能作为既有 singleton surface tab 进入组，不能复制业务状态或新增后端命令。
- 不允许同时保留 `.syn-knowledge-central-tabs` 的全局模式排和 `.native-workspace-tabs` 的文档排。最终每个组只有一排真实 tablist；刷新等组工具不能伪装成 tab。
- 打开已存在的 Markdown 路径时聚焦既有标签，不复制；打开新路径加入当前 Markdown 组。Graph/Canvas 入口聚焦或打开其 singleton tab，不改变对应内容实现。
- R3B 不实现标签拖拽重排，也不实现两个不同 Markdown 草稿同时编辑；这两项必须留作明确 GAP，不能用不可保存的假交互冒充。

### 5.2 标签行为

- 每个 tab 有 `role="tab"`、稳定 `id`、关联 `tabpanel` 和准确 `aria-selected`；当前组有可见但克制的 active-group 状态。
- `+` 触发既有 quick-open，不创建空白假笔记；选择结果后进入当前组并落到新/既有 Markdown 标签。
- `⌘T` 与 `+` 走同一路径；`⌃Tab` 在当前组内循环切换标签。普通输入、浏览器保留组合键和 overlay 内快捷键不得被误拦截。
- 点击或键盘切换标签后焦点落在对应 tab 或其正文入口；关闭标签后焦点按“右邻居 -> 左邻居 -> 组工具”确定性恢复，不得掉到 body。
- 关闭当前 Markdown 标签、切换到另一 Markdown 或关闭最后承载该草稿的组之前，必须复用既有 dirty/conflict fail-closed 规则；未保存草稿不得被静默丢弃。

### 5.3 左右分栏与单一草稿事实源

- “向右分栏”只在当前有 Markdown 时启用；从单组进入双组后，左组为源码、右组为同一 Markdown 草稿的预览，默认比例 `50/50`。
- 两组共享同一个 `selectedRelativePath`、draft、mtime/hash 和冲突状态；不得再建第二个 textarea、第二份 draft state 或第二条保存路径。
- 源码编辑后右侧预览即时读取同一 draft；保存仍只有既有一个“保存 Markdown”动作。
- 分隔器使用语义 separator，支持 pointer 拖动和键盘 `ArrowLeft/ArrowRight` 调整；比例限制为 `30–70%`，并暴露 `aria-valuemin/max/now`。不引入依赖。
- 关闭右组或执行“合并分栏”回到单组时保留当前标签、草稿和焦点；不能通过重新读取 vault 重建。
- 双组可以把右组切换为既有 Graph/Canvas 内容以复现 R0 01/04 的结构关系，但本包只验它们位于组内且边界正确，不验 Graph/Canvas 内容高保真。

### 5.4 偏好迁移

- 现有 `syn-native-knowledge-workspace-ui-v1` 是可丢弃 UI 偏好，不是知识真相源；若扩展结构，必须提供确定性的向后兼容 normalization，不得让旧偏好阻断启动。
- 只持久化可重建 chrome：Markdown tabs、selected path、单/双组、右组 surface、split ratio 和 view projection；不得持久化 Markdown/Canvas 内容、草稿正文、真实绝对路径或后端返回对象。
- 非法组数、重复/不安全路径、未知 surface、非有限 ratio 一律归一到安全默认；最大 Markdown tabs 保持 `12`。
- storage 读取/写入失败继续 fail-soft；fresh synthetic context 的 mount 前 localStorage 必须为空。

### 5.5 视觉、响应与可访问性

- 当前标签和其正文表面连续；非当前标签退后。中央正文是唯一视觉焦点，不增加大标题、卡片容器、强阴影、gradient 或新色相。
- 标签可见高度以 R0 约 `35px` 工具/标签带为参考；路径过长省略显示，但 `title`/可访问名称保留完整路径。关闭按钮不得靠 hover 才能被键盘发现。
- `1440×900` 与 `1180×760` 双组应稳定并排；`900×760` 在双侧栏展开、单侧折叠和双侧折叠状态下都不得造成 document/body/shell 横向 overflow。
- 组和内容内部允许自己的横向/纵向滚动；不得裁掉 tab、关闭按钮、分隔器或正文操作。
- focus-visible、hover、active、disabled、empty、dirty/conflict 和 reduced-motion 状态必须继续可辨认，不能只靠颜色。

## 6. Red-first 合同

实现前先让新增合同失败并记录“总项/失败项”和失败类别。至少覆盖：

1. 旧实现同时存在全局模式 tablist 与 Markdown tablist，无法形成每组单排；
2. 旧 `viewMode="split"` 只是 `.native-workspace-document--split` 内部两列，不是两个具备 tablist/panel/separator 的组；
3. 纯 reducer 覆盖单组 -> 双组 -> 调比例 -> 合并，非法 ratio 与第三组 fail closed；
4. tab 打开去重、切换、关闭后的邻居选择和最后标签空态；
5. dirty/conflict 时关闭/切换/合并不会丢草稿；
6. `⌘T`、`⌃Tab`、`+`、关闭按钮和分隔器键盘路径；
7. tab/tablist/tabpanel/separator 的 ARIA 关联和 focus 恢复；
8. 旧 UI 偏好到新结构的迁移、非法输入归一、最大 12 tabs 和 storage fail-soft；
9. Graph/Canvas 只能落在中央组 panel，不能成为根级兄弟或第二壳；
10. R3A Search/quick-open/command、overlay 回焦、折叠侧栏、来源上下文和零 overflow 合同继续通过。

不得删除旧断言、降低 selector 精度、整段快照覆盖或仅检查源码字符串制造绿色。若必须改变旧断言，需证明它锁定的是已被本包明确替换的旧双层结构，并把替代合同写清。

## 7. Synthetic 浏览器验收

复用真实 React、真实生产 CSS和现有 pure-synthetic fixture，fresh context 中至少采集：

1. `1440×900`：左组 Markdown 源码、右组同一草稿预览，两个组各有单排真实标签；
2. `1180×760`：左组阅读或源码、右组既有 Graph，证明 Graph 位于标签组 panel 内；
3. `900×760`：双组 Markdown/Preview，左右侧栏至少覆盖“全展开、右折叠、双折叠”三种量尺；
4. 分隔器从 `50` 调到 `60` 后的量尺和 ARIA；再合并回单组并验证草稿/焦点未丢；
5. `⌘T` 打开 quick-open、选择第二条合成 Markdown 后形成/聚焦标签，再用 `⌃Tab` 切换；
6. dirty draft 下关闭当前 Markdown 被拒绝，页面与草稿仍在。

每个 context 记录：

- mount 前 localStorage 为空；
- 精确 read mock allowlist；
- 外部请求、真实 Tauri 写调用、未识别调用、console error、page error均为 `0`；
- 视口、组数、标签数、当前组/标签、焦点、split ratio、document/body/shell overflow；
- 与 R0 01/02/04 的逐图 `PASS / GAP`，不得用 DOM/量尺 PASS 代替视觉判断。

synthetic 场景不实际保存 Markdown，不创建目录/文件，不写 Canvas。允许浏览器本地写可丢弃 UI 偏好，但必须单列 key、次数和规范化内容，不能混写成“全部写调用为零”。

## 8. 必跑验证与回交

从 `prototypes/productized-desktop-shell`：

1. 聚焦 `knowledge-workbench-shell` 合同；
2. `native-knowledge-workspace` 定向合同；
3. `knowledge-graph`、`knowledge-canvas` 定向合同；
4. `npm run typecheck`；
5. `npm run test:offline-interaction`，必须确认正式 runner 37 个既有入口全部执行；若本包未新增入口，数量不得伪报变化。

从仓库根：

6. `node scripts/harness/workbench-shape-gate.js --mode baseline`；
7. `node scripts/harness/workbench-shape-gate.js --mode check`，既有非零债务原样报告，只接受零净增类别/finding；
8. `node scripts/harness/workbench-shape-gate.hardcoded-hex.selftest.js`；
9. `node scripts/harness/workbench-shape-gate.machine-face.selftest.js`；
10. `git diff --check`；
11. `git diff --cached --name-only` 必须为空。

收尾只关闭本包启动的 Vite/浏览器，并确认 5173 无 listener。

必须回传：

- preflight 与冻结 hash；
- red/green 实数和类别；
- 最终组状态模型、偏好迁移、dirty/conflict 处理；
- 六组浏览器场景的截图/量尺/焦点/ARIA/隔离报告；
- R0 逐图结论；
- 完整写入文件；
- 所有门禁、shape 既有债务、staged/端口；
- 新 catch；没有则明确“零新 catch”。

## 9. 固定结论与停止条件

- `PASS_N2R_R3B_CENTRAL_TAB_GROUPS_AND_SPLIT / NEEDS_GUIDANCE_REVIEW / NOT_REAL_APP_ACCEPTED`
- `NEEDS_N2R_R3B_REWORK / NOT_ACCEPTED`
- `BLOCKED_N2R_R3B_BASELINE_DRIFT`
- `BLOCKED_N2R_R3B_WRITE_OWNERSHIP_CONFLICT`
- `BLOCKED_N2R_R3B_SCOPE_EXPANSION`
- `BLOCKED_N2R_R3B_BROWSER_OR_PORT`
- `BLOCKED_N2R_R3B_DIRTY_DRAFT_MODEL`

遇到下列任一情况立即停止，不自行扩白：

- 需要修改 Graph/Canvas/Maintenance 产品组件或 Rust/后端才能形成组；
- 需要新增依赖、第二份 Markdown 草稿或绕过既有冲突保护；
- 两组不同 Markdown 同时编辑才能继续；
- fixture 必须访问真实 vault、真实 profile 或外部网络；
- 白名单 drift、并行 writer、staged 非空或端口冲突。

即使本包通过，也不能声称：

- N2R 或 Obsidian 全界面高保真完成；
- Graph、Canvas、活动栏、右栏内容视觉已收敛；
- 标签拖拽重排、两个不同 Markdown 草稿并行编辑或任意数量分栏完成；
- 真实 Syn/Tauri App、真实 store/vault、Home-only discovery、Gate 0、十二项或发布验收通过。

## 10. 实际执行结果（2026-07-26）

- 开工 preflight：HEAD `e9ad7f3a204a1ebb11ce26c1e8c05b19c04c0991`、staged 空、5173 空闲、18 个冻结 SHA-256 和白名单写所有权全部通过；未见并行 writer。
- red-first `knowledge-workbench-shell` 为 `16 / 16` 失败，覆盖单/双组、同草稿 source/preview、比例/合并、上限/非法值、去重/关闭/焦点、surface 单例、快捷键、迁移、dirty/conflict、storage fail-soft、旧 DOM 退场与真实 tablist；最终同入口为 `R3B 16 / 0`。
- 最终中央模型只有 `groups / activeGroupId / splitRatio`，最多两个左右组、每组一条真实标签序列；Graph/Canvas/Maintenance 作为组内单例 surface tab；分隔比例 `30–70`，支持 pointer 与键盘。
- 两组 Markdown 只投影现有唯一 `selected + draft + mtime/hash/conflict`；最终最多一个 textarea/保存按钮。dirty/conflict 下切换、关闭和不安全合并均 preserve，不绕过冲突保护，不支持第二草稿。
- 继续使用 `syn-native-knowledge-workspace-ui-v1`，内容升级为规范化 `version: 2` chrome 偏好；legacy v1 split 确定迁移为同路径 source/preview 两组，坏 JSON/storage/路径/比例 fail soft。
- synthetic browser 最终 `6` 个 fresh context、`90 / 90`、`9` 张实图；覆盖 `1440×900`、`1180×760`、`900×760`、双侧栏与折叠、Graph 组内 tab、比例 `50→60`、合并、`⌘T`、`⌃Tab`、焦点/ARIA、dirty close 拒绝和 overflow。真实 Tauri 写、未知调用、外部请求、console error、page error 均为 `0`；localStorage 只写可丢弃 UI 偏好 key，次数按 context 为 `4 / 5 / 4 / 4 / 4 / 16`。
- R0 01/02 在本包中央组关系上 PASS，但完整界面仍 GAP；R0 04 为 GAP：Graph 入组有实图，Canvas 只有合同覆盖且本包不重做二者内容视觉。
- 四个聚焦入口、typecheck、正式 37-entry offline runner、hardcoded-hex `13/13`、machine-face `18/18` 全过。shape baseline/check 最终均为既有 `17 errors / 5 warnings / 5 info`，check 按既有债务非零退出，零新增类别/finding；`git diff --check` 通过、staged 为空、5173 收尾无 listener。
- 实际写入 6 个产品/测试文件、R3B raw runner/JSON/9 图、本 evidence 和本任务包实际结果；`tests/native-knowledge-workspace.test.tsx` 保持冻结 hash。未改 Shell、Graph/Canvas/Maintenance 产品组件、Graph/Canvas 测试、fixture HTML、runner、Rust/Cargo、App、CURRENT、AUTHORITY、计划、决策、依赖、真实 store/vault 或 `.codex`。
- 零新 catch；未 stage、commit、push、reset、clean、stash 或 checkout；未进入 R3C。
- 完整证据：`evidence/2026-07-26-l3-syn-n2r-r3b-central-tab-groups-and-split-verification-v1.md`。
- 执行线只回交 **`PASS_N2R_R3B_CENTRAL_TAB_GROUPS_AND_SPLIT / NEEDS_GUIDANCE_REVIEW / NOT_REAL_APP_ACCEPTED`**，不自行验收。

## 11. 指导最终验收（2026-07-26）

指导线独立核对了实际 reducer、唯一 `selected + draft` 编辑事实源、v1→v2 可丢弃偏好迁移、dirty/conflict 闭锁、生产 CSS、red/green 合同、浏览器 runner/JSON 和 9 张截图，并逐图对照 R0 01/02/04。确认每组只剩一排真实 tablist，双组最多两个，源码/预览共享同一草稿和保存入口，Graph 位于组内 panel；`900×760` 三种侧栏状态没有 document/body/shell 横向 overflow。

指导线独立复跑 `npm run typecheck`、冻结 37-entry `npm run test:offline-interaction`、shape baseline/check、hardcoded-hex `13/13`、machine-face `18/18`、`git diff --check`、staged 与 5173 收尾检查，结果与执行线回交一致。shape check 仍因既有 `17 / 5 / 5` 非零债务退出 `1`，未发现本包新增类别或 finding。

R0 01/02 只在本包“中央组关系”维度通过；R0 04 的 Canvas 实图和内容视觉、Graph 内容视觉、活动栏与右栏仍是明确 GAP。未做真实 Syn/Tauri App、真实 store/vault、Home-only discovery、Gate 0、十二项或发布验收。

指导最终裁决：**`ACCEPTED_N2R_R3B_CENTRAL_TAB_GROUPS_AND_SPLIT / NOT_REAL_APP_ACCEPTED`**。本裁决不授权 R3C、其他后续 R3、真实 App 或任何共享运行资源写入。
