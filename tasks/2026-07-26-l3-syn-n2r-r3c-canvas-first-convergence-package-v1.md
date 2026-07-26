# 任务包：L3 Syn N2R-R3C Canvas-first 视觉收敛 v1

- 日期：2026-07-26
- 状态：**NEEDS_N2R_R3C_REWORK / NOT_ACCEPTED**
- 负责人：独立知识前端执行线
- 指导/验收：当前总指导对话
- 小阶段计划：`docs/plans/2026-07-23-l3-syn-native-knowledge-workspace-small-stage-plan-v2.md`
- 冻结参考：`docs/design/2026-07-25-l3-obsidian-core-desktop-reference-and-migration-r0-v1.md`
- 上游接受：`ACCEPTED_N2R_R3B_CENTRAL_TAB_GROUPS_AND_SPLIT / NOT_REAL_APP_ACCEPTED`

## 1. 目标与完成标准

本包只关闭 R2 差距矩阵中的 **Canvas-first P1**：

1. 把当前常驻的 Canvas 大标题、文件 catalog/新建表单、横向工具条和右侧节点 inspector，收为以连续画布为唯一主焦点的中央标签内容；
2. 当前 Canvas 路径、保存/重读状态使用紧凑 chrome；文件切换/新建改为按需面板，节点属性只在选择节点后按需出现；
3. 节点类型/新增、缩放和视口控制留在画布内部的小型浮动工具，不再切走一整行或长期切走两侧宽度；
4. 保留现有 JSON Canvas 1.0、四类节点、连线、未知字段 roundtrip、固定 vault 相对引用、本地草稿和 mtime/hash 冲突保护；
5. 在 R3B 的单组、双组和 `900×760` 紧凑桌面中维持无横向 overflow、键盘/ARIA、焦点返回和零真实写；
6. 完成 red-first 合同、synthetic-only 浏览器实图/量尺、完整离线回归和 evidence。

完成后只能回交
`PASS_N2R_R3C_CANVAS_FIRST_CONVERGENCE / NEEDS_GUIDANCE_REVIEW / NOT_REAL_APP_ACCEPTED`
或 §9 的其他固定枚举。执行线不得自行验收、不得继续 R3D。

本包由用户在 2026-07-26 明确批准并由指导线派发。授权仅限本文白名单和 synthetic-only 浏览器验证。

## Authority and plan alignment

- authority_chain: `AGENTS.md` -> `CURRENT.md` -> knowledge/conversation parallel decision -> native knowledge route decision -> native knowledge small-stage plan -> frozen Obsidian R0 -> accepted R2 gap matrix -> accepted R3A/R3B -> this R3C task
- plan_anchor: `docs/plans/2026-07-23-l3-syn-native-knowledge-workspace-small-stage-plan-v2.md#r3-既有能力交互与视觉收敛`
- existing_before_new: 已有 `KnowledgeCanvasView`、受限四方法 typed client、React Flow、四类节点/边、未知字段保留、草稿/冲突闭锁、R3B 中央标签组和 synthetic fixture；只重组可见层级，不建第二套 Canvas 模型。
- capabilities_touched: none — 不改 backend capability、command、payload、vault 边界或 Code Map canonical/entrypoint。
- forbidden_alternatives: 不用纯 CSS 隐藏继续常驻的三列 DOM 冒充按需面板，不把 Canvas 做成卡片仪表盘，不复制 Obsidian 商标/图标/CSS，不引入第二图框架，不嵌入/控制 Obsidian，不启动真实 Syn/Tauri/Codex/MCP/store/vault。

## 2. 设计意图与 R0 对照

### 2.1 使用者、任务与感受

- 使用者：在 macOS 上整理笔记、线索和关系的单人桌面用户。
- 核心动作：打开一张画布后持续摆放、连接、选择和编辑节点，同时不丢失中央标签组、左右上下文和当前 vault。
- 感受：连续、安静、可探索；画布胜出，工具只在需要时出现。

### 2.2 设计检查点

```text
Intent:     用户在同一中央标签组内操作开放画布，不进入第二张管理页面。
Hierarchy:  画布平面与节点胜出；当前路径/保存状态次之；文件选择、新建和节点属性按需退后。
Palette:    只使用现有 Syn 暖纸、墨色和青绿语义 token，不新增硬编码颜色。
Depth:      低对比边界和轻微表面差；浮动工具最多一层，不新增阴影系统、渐变或玻璃。
Surfaces:   画布为 base；紧凑 chrome 为 soft；按需面板为 raised，且始终约束在 Canvas 根内。
Typography: 沿用现有 Syn 字体和桌面工具字号；长路径省略显示并保留完整可访问名称。
Spacing:    4px 基础单位；工具、路径、状态和面板使用 4/8/12px 的紧凑节奏。
```

- Domain：vault、Canvas 文件、开放画布、节点、边、选择、视口、草稿、保存、冲突。
- Color world：纸张、石墨、墨色、旧木、青玉状态；全部映射既有 token。
- Signature：Syn 的固定 vault、确认式写入和冲突保护仍在，但视觉中心变为开放画布。
- Rejecting：
  - 常驻 catalog + 创建表单 → 一个紧凑“画布”触发器和按需文件面板；
  - 常驻大标题 + 整行工具条 → 当前路径/保存状态紧凑 chrome + 画布内浮动工具；
  - 常驻空 inspector → 选择节点后才出现、可关闭的局部属性面板。

主参考：

- R0 `04-canvas-graph-context-984x768.jpg`：Canvas/Graph 位于中央标签组，画布连续，小型工具浮于画布，右栏仍是工作台上下文；
- R0 `08-left-sidebar-collapsed-ribbon-visible-984x768.jpg`：侧栏折叠后中央工作区接管空间；
- R2 `05-1180-canvas.png`、`06-1180-left-collapsed.png`：当前常驻 catalog/表单/工具条/inspector 的差距证据；
- R3B `evidence/raw/2026-07-26-l3-syn-n2r-r3b-tab-groups-split/`：当前中央真实标签组和响应基线。

本包不要求像素复制 Obsidian，也不改 Graph、活动栏或右栏内容层级；R0 04 只能在“Canvas 内部 + 已接受中央组关系”维度判定。

## 3. 开工冻结与并发

### 3.1 冻结基线

- HEAD：`e9ad7f3a204a1ebb11ce26c1e8c05b19c04c0991`
- staged：指导派发时为空；执行开工时必须再次为空。
- 端口：指导派发时 `5173` 无 listener；执行线开工时必须再次确认。

相对路径均以仓库根为准：

| 文件 | 指导冻结 SHA-256 |
| --- | --- |
| `prototypes/productized-desktop-shell/src/views/knowledge/KnowledgeCanvasView.tsx` | `af336f7c1176a60791e003f3162456bb31ae6ea6c5cbb6672b3fafafaa973c9c` |
| `prototypes/productized-desktop-shell/src/styles.css` | `0cf85dbd37ca02c17143f6b43c1bb7500d30520238680fa0c770993fe07b3bf0` |
| `prototypes/productized-desktop-shell/tests/knowledge-canvas.test.tsx` | `53481206e5aa70f9b21ee677602ac2cbece70e5cfadbcc9ec58e5998c5f5aab6` |
| `prototypes/productized-desktop-shell/tests/knowledge-workbench-visual-fixture.tsx` | `219637ff4493150ea4fd0aac7e64cb6e232f2840d80560208da7e014d8d63fee` |
| `prototypes/productized-desktop-shell/src/views/knowledge/NativeKnowledgeWorkspace.tsx` | `eafd9edccc3a2cd62e99cfa211809d77b206c826907ca07976ef4c419f4b4464` |
| `prototypes/productized-desktop-shell/src/lib/knowledgeWorkbenchLayout.ts` | `8fb9faf01ef9dae509a6cffa4e14fd62031bb9744ea8e44eb1d1476e4f9ee220` |
| `prototypes/productized-desktop-shell/src/lib/knowledgeWorkspace.ts` | `81eddc02206035a9d1bf1e3954d5ea19b1864a71b27ffea43efafa5a2fd51fb0` |
| `prototypes/productized-desktop-shell/tests/knowledge-workbench-shell.test.tsx` | `2cbf2b61a86731096a876636c27119066c0e10d5bfaac81e8c6c7322660541da` |
| `prototypes/productized-desktop-shell/tests/knowledge-workbench-visual-fixture.html` | `8a58ac0d02d9c33a9ca151f4ec1e653ccd3c6d79096581efd339a62c1fcec23c` |
| `prototypes/productized-desktop-shell/scripts/run-offline-interaction-test.mjs` | `798daf13722875befa22eb233c476b957d241005a3b08267ed30eb54e0363082` |
| `docs/design/2026-07-25-l3-obsidian-core-desktop-reference-and-migration-r0-v1.md` | `141ff343312fff91346e6e170f1aea2d07cda5fa2abdb3fde8b840b16f9ed4d9` |
| `evidence/2026-07-26-l3-syn-n2r-r2-isolated-browser-visual-baseline-v1.md` | `8dd432266f406e96c69359196ea138eede85891d041ed8f98a3907eed55b6c7a` |
| `tasks/2026-07-26-l3-syn-n2r-r3b-central-tab-groups-and-split-package-v1.md` | `9ededa2f7dab53f9fe465aba7187336d25012611cadaf024075a119f0173600b` |
| `evidence/2026-07-26-l3-syn-n2r-r3b-central-tab-groups-and-split-verification-v1.md` | `de571a7a85b07d8152989ecb9e7e368834d85c8a6a00f11189c5d2cfaa4de2ef` |

开工前必须：

1. 复核 HEAD、staged、上述 hash、5173 与白名单逐文件 provenance；
2. 确认没有其他任务写本包白名单，尤其是共享 `src/styles.css` 和 visual fixture；
3. 若任一 hash 漂移、staged 非空、端口被占或写所有权不唯一，停止为对应 §9 枚举；
4. 不 reset、clean、stash、checkout、覆盖或批量格式化现有 WIP。

### 3.2 串行边界

- 本包独占 `KnowledgeCanvasView.tsx`、`src/styles.css`、`knowledge-canvas.test.tsx` 和 visual fixture 写面。
- R3B 的中央状态、Markdown 草稿、组标签和分隔器文件全部冻结只读；Graph、活动栏、右栏后续包不得并发施工。
- 对话底座线只可做静态只读准备；不得并发构建/启动三句重验、真实 App、MCP/CLI 或写共享运行资源。

## 4. 精确写入白名单

### 4.1 产品与测试

- `prototypes/productized-desktop-shell/src/views/knowledge/KnowledgeCanvasView.tsx`
- `prototypes/productized-desktop-shell/src/styles.css`：仅 Canvas selector 和必要的 Canvas-in-group 窄响应规则
- `prototypes/productized-desktop-shell/tests/knowledge-canvas.test.tsx`
- `prototypes/productized-desktop-shell/tests/knowledge-workbench-visual-fixture.tsx`：仅增加 Canvas 场景所需量尺/选择器，不改变 mock 安全边界

`NativeKnowledgeWorkspace.tsx`、两个知识 helper、shell 测试、fixture HTML、正式 runner、Graph/Maintenance 产品与测试均为冻结只读面。若必须改其中任何一个才能完成，停为 `BLOCKED_N2R_R3C_SCOPE_EXPANSION`，不得自行扩白。

### 4.2 证据与控制文档

- `evidence/2026-07-26-l3-syn-n2r-r3c-canvas-first-convergence-verification-v1.md`
- `evidence/raw/2026-07-26-l3-syn-n2r-r3c-canvas-first/**`
- 本任务包：只回写实际状态、实际写入和执行结果
- `docs/harness-catch-log.md`：仅发现新的真实 catch 时追加

除此之外一律不写。不得修改 App、Rust/Cargo、依赖、CURRENT、AUTHORITY、计划、决策、真实 store/vault 或 `.codex`。

## 5. 实现合同

### 5.1 单一 Canvas 事实源与能力保留

- 保留当前 `snapshot/readCanvas/createCanvas/writeCanvas` 四方法 typed seam；不得增加 raw invoke、command 名、vault root、URL、shell 或外部打开面。
- 保留 text/file/link/group 节点、边、拖动、局部字段编辑、未知字段 roundtrip、相对文件引用、删除节点同步删边和整数坐标。
- 本地 add/edit/move/connect/delete 继续只改 draft；只有显式“保存 Canvas”才能调用现有 CAS 写路。冲突、迟到请求、刷新和导航继续 fail closed。
- 不建立第二份 Canvas document、第二条保存路径或与 React Flow 平行的图形模型。

### 5.2 Canvas-first 信息架构

- `.native-knowledge-canvas` 在组内占满可用宽高；移除常驻 `.native-canvas-browser-grid` 三列关系和页面级大标题。
- 顶部只保留一条紧凑 chrome：当前 Canvas 的可访问路径、dirty/保存/冲突状态、打开画布、保存、重新读取。没有当前 Canvas 时用同一位置呈现明确空态。
- 文件 catalog 与新建路径表单只能在显式“画布”动作后按需渲染；默认关闭时不得在 DOM 中留下可聚焦控件或占据画布轨道。选择/新建成功或 Escape 后关闭并确定性回焦。
- 节点 inspector 只在选中节点时按需渲染，位于 Canvas 根内的局部面板；可关闭，关闭后画布恢复完整宽度并把焦点返回画布/已选节点的确定性入口。
- 不实现不存在的边属性编辑、无限多浮窗、画布文件拖入或外部 URL 打开；不能用假按钮冒充。

### 5.3 画布、浮动工具与状态

- 加载成功后 `.native-canvas-flow-stage` 是唯一主视觉焦点并横向占满 Canvas 根；无按需面板时，高度至少占 Canvas 根可用高度的 `75%`，不得因旧 header/toolbar 留白产生页面式分段。
- 节点类型 + 新增节点组成一个紧凑浮动工具组，并与 React Flow 现有缩放/适配/交互控件同属画布内部；不得新增第二整行 toolbar。
- 浮动工具必须有可访问名称、focus-visible、hover、disabled；工具不得遮住当前选中节点的主要内容，窄组中允许收为更小布局但不能消失。
- notice/conflict 使用紧凑状态条或局部提示，不把画布推成第二个长页面；loading、empty、read error、conflict 均有明确可见文案和可恢复动作。
- 按需面板不得使用 `position: fixed` 越出中央组，不使用 blur、gradient、新阴影系统、硬编码色值或新依赖。

### 5.4 键盘、响应与 R3B 集成

- 打开画布面板：触发器暴露 `aria-expanded/controls`，焦点进入首个有效控件；Escape 关闭并回到同一触发器。
- 节点选择与 inspector 建立可访问关联；关闭 inspector 后焦点不落到 `body`。折叠/关闭面板后其中控件必须退出 Tab 和辅助技术路径。
- `1180×760` 单组及 Canvas/Graph 双组、`900×760` 双侧栏展开/右栏折叠/双折叠均不得产生 document、body、shell 或 active group panel 横向 overflow。
- 双组窄 Canvas 中允许内部画布平移/缩放，但不得裁掉顶部 chrome、文件触发器、保存/重读、浮动工具或节点 inspector 关闭动作。
- 保留 R3B 每组单排 tablist、最多两组、split ratio、焦点/ARIA 和可丢弃 UI 偏好；不得修改其 reducer 或 storage 结构。
- `prefers-reduced-motion: reduce` 下所有面板开关、焦点返回和 React Flow 基本操作仍可用；本包不新增必要动画。

## 6. Red-first 合同

实现前先用当前生产 UI 运行新增合同并记录总项、失败项和失败类别；至少保留一张能证明旧三列结构的 red 图。新增合同至少覆盖：

1. 旧 `.native-canvas-browser-grid`、常驻 catalog/创建表单、整行 toolbar 和空 inspector 被精确判为失败；
2. 静态壳和浏览器壳均只有一个 Canvas 根，且没有页面级重复大标题或第二画布模型；
3. 默认关闭的文件面板不占轨、不留可聚焦子控件；打开/关闭/Escape 的 `aria-expanded/controls` 和回焦；
4. 未选节点时 inspector 不渲染；选择节点后出现并关联，关闭后回焦且恢复完整画布；
5. 画布主面宽高占比、浮动工具位于 stage 内，以及单组/双组窄宽度无 overflow；
6. 四类节点、边、拖动、局部字段、未知字段、相对引用和 local-only draft 旧合同继续通过；
7. save/create 仍只走固定 typed client，mtime/hash CAS、dirty/conflict、迟到请求和显式重读闭锁不回归；
8. R3A Search/overlay、R3B 单排 tab group/split、折叠侧栏、焦点和 `900×760` 零 overflow 合同继续通过；
9. SSR 不调用 Tauri/vault，synthetic fixture 的 write/unknown/external fail-closed 边界不放宽；
10. 没有 `position:fixed` Canvas 面板、新依赖、硬编码颜色、第二个 React Flow 根或 Obsidian 品牌资产。

不得删除旧断言、降低 selector 精度、用整段快照或源码字符串计数冒充浏览器行为。若旧断言锁定的是本包明确淘汰的常驻三列，只能用等强或更强的新语义/浏览器合同替换，并在 evidence 对照说明。

## 7. Synthetic 浏览器验收

复用真实 React、真实生产 CSS 和现有 pure-synthetic fixture。fresh context 中至少采集：

1. `1180×760`：单组打开合成 Canvas，文件面板与 inspector 均关闭，连续画布和紧凑 chrome；
2. `1180×760`：显式打开文件/新建面板，记录首焦点、ARIA、占用边界；Escape 关闭并回到同一触发器；
3. `1180×760`：选择合成节点后显示局部 inspector，再关闭并验证回焦/画布恢复；
4. `1180×760`：左组 Canvas、右组既有 Graph，逐图对照 R0 04 的中央组/连续空间关系；
5. `900×760`：双侧栏展开的 Canvas，确认 chrome/浮动工具可达且 document/body/shell/panel 无横向 overflow；
6. `900×760`：右栏折叠与双侧栏折叠，确认 Canvas 接管空间、无横向 overflow；
7. reduced-motion fresh context：画布面板开关、Escape 回焦和节点 inspector 路径继续成立。

至少输出 6 张最终截图和一个原始 JSON 报告；red-first 报告和 red 图单列，不能覆盖。每个 context 记录：

- mount 前 localStorage 为空；
- 精确 read mock allowlist 和各 command 次数；
- 真实 Tauri 写调用、未识别调用、外部请求、console error、page error均为 `0`；
- 视口、Canvas root/stage/按需面板/浮动工具 bounds、stage 占比、组数、焦点、ARIA、document/body/shell/panel overflow；
- 与 R0 04/08 的逐图 `PASS / GAP` 及仍存视觉差异，不能用 DOM/量尺 PASS 代替视觉判断。

浏览器验收不得点击“保存 Canvas”或“新建 JSON Canvas”，不得写真实或 synthetic 文件。允许 add/edit/move 等本地 draft 动作，但必须证明 `knowledge_workspace_write_canvas/create_canvas=0`。localStorage 只允许 R3B 已接受的可丢弃 UI 偏好写入，并单列 key/次数。

## 8. 必跑验证与回交

从 `prototypes/productized-desktop-shell`：

1. 聚焦 `knowledge-canvas` 定向合同；
2. `knowledge-workbench-shell`、`native-knowledge-workspace`、`knowledge-graph` 定向合同；
3. `npm run typecheck`；
4. `npm run test:offline-interaction`，必须确认正式 runner 37 个既有入口全部执行；若未新增正式入口，数量不得伪报变化。

从仓库根：

5. `node scripts/harness/workbench-shape-gate.js --mode baseline`；
6. `node scripts/harness/workbench-shape-gate.js --mode check`，既有非零债务原样报告，只接受零净增类别/finding；
7. `node scripts/harness/workbench-shape-gate.hardcoded-hex.selftest.js`；
8. `node scripts/harness/workbench-shape-gate.machine-face.selftest.js`；
9. `git diff --check`；
10. `git diff --cached --name-only` 必须为空。

收尾只关闭本包启动的 Vite/浏览器，并确认 5173 无 listener。

必须回传：

- preflight、冻结 hash 与白名单写所有权；
- red/green 实数、失败类别和 red 图；
- Canvas chrome、按需文件面板、selection inspector、浮动工具与冲突/草稿处理；
- 七组浏览器场景的截图/量尺/焦点/ARIA/隔离原始报告；
- R0 04/08 逐图结论；
- 完整写入文件；
- 所有门禁、shape 既有债务、staged/端口；
- 新 catch；没有则明确“零新 catch”。

## 9. 固定结论与停止条件

- `PASS_N2R_R3C_CANVAS_FIRST_CONVERGENCE / NEEDS_GUIDANCE_REVIEW / NOT_REAL_APP_ACCEPTED`
- `NEEDS_N2R_R3C_REWORK / NOT_ACCEPTED`
- `BLOCKED_N2R_R3C_BASELINE_DRIFT`
- `BLOCKED_N2R_R3C_WRITE_OWNERSHIP_CONFLICT`
- `BLOCKED_N2R_R3C_SCOPE_EXPANSION`
- `BLOCKED_N2R_R3C_BROWSER_OR_PORT`
- `BLOCKED_N2R_R3C_TYPED_CLIENT_OR_DRAFT_REGRESSION`

遇到下列任一情况立即停止，不自行扩白：

- 需要修改 R3B reducer/组壳/Markdown 草稿、Graph、Rust/后端或 typed bridge；
- 需要新增依赖、第二份 Canvas document、第二条保存路径或放宽 fixed-vault/CAS；
- 需要实际保存/新建 Canvas 才能形成视觉证据；
- fixture 必须访问真实 vault、真实 profile 或外部网络；
- 白名单 drift、并行 writer、staged 非空或端口冲突。

即使本包通过，也不能声称：

- N2R、Obsidian 全界面或 1:1 像素复刻完成；
- Graph、活动栏、右栏内容视觉或完整 R0 04 已收敛；
- 边属性编辑、多人协作、任意外部链接打开或插件 Canvas 兼容完成；
- 真实 Syn/Tauri App、真实 store/vault、Home-only discovery、Gate 0、十二项或发布验收通过。

## 10. 执行结果（2026-07-26）

- 执行线回交：**`PASS_N2R_R3C_CANVAS_FIRST_CONVERGENCE / NEEDS_GUIDANCE_REVIEW / NOT_REAL_APP_ACCEPTED`**；不自行验收，不进入 R3D。
- 开工 HEAD、staged、5173、14 个冻结 hash 与逐文件写所有权均符合 §3；收尾冻结只读 hash 与 HEAD 未漂移，staged 仍为空，5173 已释放。
- red-first：静态合同 `14 项 / 9 失败`，浏览器 `14 条 / 9 失败`；旧三列、页面标题、常驻 catalog/toolbar/空 inspector、缺失 compact chrome/连续 stage/ARIA/浮动工具均被直接抓到。
- green：`knowledge-canvas` R3C `14 / 0`；R3B shell `16 / 0`；native workspace 与 Graph 定向合同、typecheck、冻结 37-entry offline runner、两项 shape selftest 和 `git diff --check` 均通过。
- synthetic-only browser：`7 contexts / 131 assertions / 0 failed`，10 张最终实图；每个 fresh context 均 mount 前 localStorage 为空，真实 Tauri 写、未知调用、外部请求、console error、page error均为 `0`。
- shape baseline/check 都是既有 `17 errors / 5 warnings / 5 info`，零净增；check 仍按预期非零退出，不冒充全仓 shape 全绿。
- R0 04 只在左 Canvas/右 Graph/连续中央空间维度 PASS，R0 08 只在活动 rail 保留与折叠宽度归还维度 PASS；其余视觉差异与完整界面继续记 GAP。
- 发现并写入 `docs/harness-catch-log.md` 的新 catch 为文件面板“DOM visible 但真实命中层错误”；未用 force click，修复后真实点击、bounds、containment 与最终浏览器矩阵均通过。
- 完整实现、写入清单、red/green 实数、量尺、焦点/ARIA、隔离审计、逐图判断与原始链接见 `evidence/2026-07-26-l3-syn-n2r-r3c-canvas-first-convergence-verification-v1.md`。

## 11. 指导独立复核（2026-07-26）

指导线确认执行回交的大部分事实成立：白名单与冻结 hash 一致；raw 报告确为 `7 contexts / 131 assertions / 0 failed`；十张图支持 Canvas-first 连续舞台、Canvas/Graph 组关系和窄宽度无横向 overflow，且 R0 差距没有被冒充为完整高保真通过。独立复跑 `npm run typecheck`、37-entry `npm run test:offline-interaction`、shape baseline/check、两项 shape selftest、`git diff --check`、staged、HEAD 与 5173 收尾，结果均与执行线回交一致；shape 仍为既有 `17 / 5 / 5`。

但指导线在任务包 §5.2、§5.4 的“同一触发器”合同上发现一个真实未覆盖边界：

1. 空 Canvas 中央空态存在第二个真实入口“选择画布”，其 `onClick` 同样调用 `showFilePanel`；
2. 在 fresh synthetic fixture 中聚焦并点击该入口，面板打开后按 Escape；
3. 面板正确卸载，但焦点落到顶部 `data-canvas-file-trigger`“画布”，没有回到实际触发面板的空态“选择画布”；
4. 原 `131 / 0` 浏览器报告只覆盖顶部触发器，不能证明所有真实入口满足“Escape 回到同一触发器”。

因此执行线的 Canvas 主结构、隔离、量尺、十张图和其余 green 证据继续有效，但 R3C 整包暂不接受。精确裁决为：

**`NEEDS_N2R_R3C_REWORK / NOT_ACCEPTED`**

下一步只能另行授权最窄 R3C-R1：记录实际 opener，并对顶部触发器与空态触发器分别 red-first 验证 Escape、选择成功和关闭动作的确定性回焦；不得顺带修改 Canvas 视觉、Graph、活动栏、右栏、Rust、真实数据或真实 App。当前未授权施工，也不得进入 R3D。
