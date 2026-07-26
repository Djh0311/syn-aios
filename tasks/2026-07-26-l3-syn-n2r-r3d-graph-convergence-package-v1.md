# 任务包：L3 Syn N2R-R3D Graph 关系舞台收敛 v1

- 日期：2026-07-26
- 状态：**PASS_N2R_R3D_GRAPH_CONVERGENCE / NEEDS_GUIDANCE_REVIEW / NOT_REAL_APP_ACCEPTED**
- 负责人：独立知识前端执行线
- 指导/验收：当前总指导对话
- 上游接受：`ACCEPTED_N2R_R3C_R1_FOCUS_RETURN / ACCEPTED_N2R_R3C_CANVAS_FIRST / NOT_REAL_APP_ACCEPTED`
- 目标 evidence：`evidence/2026-07-26-l3-syn-n2r-r3d-graph-convergence-verification-v1.md`

## 0. Kickoff

本包只关闭 R2 差距矩阵中的 Graph P2：

> 当前 Graph 虽已在中央标签组内，也已有全局/局部、文字/标签筛选和从节点打开笔记的能力，但可见形态仍是带路径、标签和阴影的大矩形卡片阵列；上方重复大标题与常驻整行表单把它做成管理页。冻结 R0 是轻量节点、细连线和连续关系舞台。

用户已在 R3C-R1 回交并获指导接受后明确回复“可以”，授权派发本包。授权仅限本文白名单和 pure-synthetic 本地浏览器验证。

本包不重做 Graph 后端，不修改 fixture 来制造更好看的图，不进入活动栏、右栏、真实 App 或 N6。

## 1. 设计意图

### 1.1 Intent、Hierarchy、Density

- Intent：用户先看懂“哪些笔记互相关联”，再沿节点打开笔记；不是在图上阅读路径、标签和统计卡片。
- Hierarchy：中央关系舞台是唯一主焦点；标签组已表达“关系图”，Graph 内不再重复大标题。
- Density：常态只保留紧凑范围/筛选入口和必要状态；文字、标签、局部焦点等详细控件按需展开。
- Signature：轻节点、清楚标签、细连线、克制的当前/选中 accent；保留 Syn 暖纸、墨色、茶色、青玉状态和既有 token。
- Depth：以连续舞台、边线、微弱表面差和选中态表达层级；禁止渐变、大卡片阴影和新增装饰性光效。

项目界面模式正本：`.interface-design/system.md`。实现必须使用其中已接受的实际 opener、overlay 和折叠语义，不得另造冲突模式。

### 1.2 拒绝项

1. 大矩形笔记卡片 → 轻量节点与标题标签；
2. 节点内常驻路径/标签/“独立笔记”元数据 → 视觉隐藏，必要语义进入 accessible name/description 或按需状态；
3. 重复大标题 + 常驻五列表单 → 紧凑 chrome + 按需筛选面；
4. 只支持鼠标点击 → 节点可键盘聚焦，并以 Enter/Space 走同一 typed open；
5. 为“自然”而引入随机布局、持久化坐标或第二图框架 → 继续复用确定性 `@xyflow/react`；
6. 通过缩小整图、模糊文字或隐藏边来制造“轻” → 标签仍可读，边仍可辨，fitView 不裁切；
7. 复制 Obsidian 图标、颜色或品牌资产 → 只借结构和交互，继续使用 Syn 自有 token 与可访问名称。

## 2. Authority、能力和并发边界

- authority_chain：`AGENTS.md` → `CURRENT.md` → native knowledge route v2 → native knowledge small-stage plan v2 → R0 → R2 gap matrix → R3C/R3C-R1 accepted evidence → 本包。
- plan_anchor：`docs/plans/2026-07-23-l3-syn-native-knowledge-workspace-small-stage-plan-v2.md#r3-既有能力交互与视觉收敛`
- existing_before_new：`@xyflow/react`、Graph typed client、全局/局部、文字/标签筛选、孤立节点、fitView、zoom controls、重复节点打开 sequence 和父层原生笔记 handoff 均已存在；本包优先收敛呈现和键盘路径。
- capabilities_touched：none；不得修改 command 名、payload、vault root、Graph backend、Markdown/Canvas 写路或 capability registry。
- data_truth：Graph 继续只读已验证 wikilink/反链投影；布局和选择不得写回 Markdown、vault、store 或 localStorage。

本包独占：

- `KnowledgeGraphView.tsx`；
- `knowledge-graph.test.tsx`；
- `styles.css` 中本文列明的 Graph selector hunk。

任何其他 UI 包在 R3D 结束前不得并发写这些文件/selector。对话底座线只可静态只读准备；R3D 产品写入、Vite/浏览器取证期间不得构建、启动或执行三句真实 App 重验，不得占用同一真实 store 或 Rust 运行资源。

## 3. 冻结基线与开工停点

- HEAD：`e9ad7f3a204a1ebb11ce26c1e8c05b19c04c0991`
- 派发时 staged：空
- 派发时 `127.0.0.1:5173`：无 listener
- 工作树存在大量既有 WIP；目标文件均按下表逐文件冻结。不得用 clean tree 假设覆盖现状。

| 文件 | 派发 SHA-256 | 本包权限 |
| --- | --- | --- |
| `prototypes/productized-desktop-shell/src/views/knowledge/KnowledgeGraphView.tsx` | `ca8d88c3183913bd946d2bef35139fce7936f8b071809d4a96f4a573cc0b6b83` | 窄写 |
| `prototypes/productized-desktop-shell/tests/knowledge-graph.test.tsx` | `d4d87d8b914bac2708eea3c86599d6da8373c2aa8b60b75a70af5d3050e98b16` | 窄写 |
| `prototypes/productized-desktop-shell/src/styles.css` | `8d74a3245d07477a44dd29d7fabd67a0cd01300a880bf4db6752dede105d03ed` | 只写 §4.1 精确 Graph selector |
| `.interface-design/system.md` | `2a035b99b0e21b33fb4af259aad0f12515406fede89a9395b9fa70ba868bd766` | 冻结只读 |
| `prototypes/productized-desktop-shell/tests/knowledge-workbench-visual-fixture.tsx` | `ea27e80ec9fff83fc630e4b80074915a39ee1ecf2f5217c3e3940bb51eb929ee` | 冻结只读 |
| `prototypes/productized-desktop-shell/tests/knowledge-workbench-visual-fixture.html` | `8a58ac0d02d9c33a9ca151f4ec1e653ccd3c6d79096581efd339a62c1fcec23c` | 冻结只读 |
| `prototypes/productized-desktop-shell/src/views/knowledge/NativeKnowledgeWorkspace.tsx` | `eafd9edccc3a2cd62e99cfa211809d77b206c826907ca07976ef4c419f4b4464` | 冻结只读 |
| `prototypes/productized-desktop-shell/src/views/knowledge/KnowledgeCanvasView.tsx` | `42054605e642100752a109830a8f7601a46eaef68f1bc70c7ca48e5353d6405e` | 冻结只读 |
| `prototypes/productized-desktop-shell/tests/knowledge-canvas.test.tsx` | `802dd4a90ceac4f491ddbf390f8e5734f02984af87c920c170b17d0c723f263c` | 冻结只读 |
| `prototypes/productized-desktop-shell/src/lib/knowledgeWorkbenchLayout.ts` | `8fb9faf01ef9dae509a6cffa4e14fd62031bb9744ea8e44eb1d1476e4f9ee220` | 冻结只读 |
| `prototypes/productized-desktop-shell/src/lib/knowledgeWorkspace.ts` | `81eddc02206035a9d1bf1e3954d5ea19b1864a71b27ffea43efafa5a2fd51fb0` | 冻结只读 |
| `prototypes/productized-desktop-shell/scripts/run-offline-interaction-test.mjs` | `798daf13722875befa22eb233c476b957d241005a3b08267ed30eb54e0363082` | 冻结只读 |
| `prototypes/productized-desktop-shell/package.json` | `08a3abc466e2dd51f946380badb6519b4273353d8d279efa43b1cf6086d87e65` | 冻结只读 |
| `prototypes/productized-desktop-shell/package-lock.json` | `781cb1d94eaaeec0d071f156f9aeca65400acce8bd811c9c95cd4d45cf700bc0` | 冻结只读 |
| 上游 R3C task | `3e6096177e1d6c98c2a01002b76a8ed32c5277f393fb4388dc093ccd15b97b8f` | 冻结只读 |
| 上游 R3C evidence | `a189496276363f916237a443e2906ce63bea040404f263a258bc690eb3dd2fbc` | 冻结只读 |
| R3C-R1 task | `8db62fa056232a3d878d6f74ba6357cbef29d82b2a2743d8322c7f8a821406ee` | 冻结只读 |
| R3C-R1 evidence | `c56753c6a071b288dc647bb09b964ee88fe01ddc90171092be65380e153050c2` | 冻结只读 |
| R0 设计参考 | `141ff343312fff91346e6e170f1aea2d07cda5fa2abdb3fde8b840b16f9ed4d9` | 冻结只读 |
| R2 基线 evidence | `8dd432266f406e96c69359196ea138eede85891d041ed8f98a3907eed55b6c7a` | 冻结只读 |

开工前逐一复核 HEAD、staged、全部冻结 hash、5173、三个窄写文件的 provenance 与唯一写所有权。任一目标/冻结 hash 漂移、staged 非空、端口占用或并行 writer 存在，立即按 §10 停止；不得 reset、clean、stash、checkout、覆盖或合并他人 WIP。

## 4. 精确写入白名单

### 4.1 产品与正式合同

1. `prototypes/productized-desktop-shell/src/views/knowledge/KnowledgeGraphView.tsx`
   - 只允许收敛 Graph 内部 chrome、筛选 disclosure、节点呈现数据和节点鼠标/键盘打开；
   - 可增加组件内实际 opener/focus ref、稳定 ARIA 和非视觉测试 handle；
   - 必须继续使用 `knowledgeWorkspace.graph` 和 `onOpenMarkdown(relativePath)`，不得暴露 raw invoke、vault root 或任意路由。
2. `prototypes/productized-desktop-shell/src/styles.css`
   - 只允许修改/新增 `.native-knowledge-graph`、`.native-graph-*`、`.react-flow__node` 与 Graph 组合 selector；
   - 如需处理 `.knowledge-workbench-group__panel > .native-knowledge-graph`，必须拆成 Graph 专属 rule，不得改变同组 Markdown/Canvas/maintenance 的 computed style；
   - 禁止改全局 token、活动栏、左右栏、标签组、Canvas、overlay 或非 Graph media rule。
3. `prototypes/productized-desktop-shell/tests/knowledge-graph.test.tsx`
   - 增加 compact chrome、只读 typed handoff、节点键盘激活、ARIA 和 deterministic presentation/helper 合同；
   - 静态/源码合同不能替代 §7 真实浏览器视觉、activeElement 和 command 证据。

### 4.2 新 evidence

- `evidence/raw/2026-07-26-l3-syn-n2r-r3d-graph-convergence/**`
- `evidence/2026-07-26-l3-syn-n2r-r3d-graph-convergence-verification-v1.md`
- 本任务包：只回写实际状态、实际写入和执行结果
- `docs/harness-catch-log.md`：仅发现新的真实 catch 时追加；没有则在 evidence 明写零新 catch

除此之外一律不写。尤其禁止修改 fixture/HTML、正式 runner、Native workspace、R3B 组壳/reducer、Canvas、活动栏、右栏、Rust/Cargo、typed client、依赖、CURRENT、AUTHORITY、计划、决策、真实 store/vault 或 `.codex`。

## 5. 产品与视觉合同

### 5.1 连续关系舞台

- Graph 根直接填满当前中央标签组，外层不出现卡片边框、圆角、阴影或额外页边距。
- 常态不显示 Graph 内第二个大 `h2`/说明段；“关系图”已经由中央标签表达，Syn 原生/只读投影语义保留给可访问名称或紧凑状态。
- 常态 chrome 不高于 44 px，只保留范围、按需筛选入口、刷新/状态中的最小充分集合。
- 文字、标签和局部焦点控件必须进入一个按需 disclosure；打开后留在 Graph 根内，不挤成第二主面板，也不越出当前组。
- `1440×900`、`1180×760` 常态 ReactFlow 可见高度 / Graph 根高度至少 `0.74`；`900×760` 至少 `0.68`。量尺写入 raw JSON。

### 5.2 轻量节点和连线

- 每个可见 Graph node 只常驻一条可读标题；路径、标签和“独立笔记”文字不得继续作为节点内卡片元数据常驻。
- `1440/1180` 下每个节点外框宽不超过 `144 px`、高不超过 `48 px`；不得使用卡片阴影，左侧粗强调边必须移除。
- 节点仍有可辨 hit target 和 focus-visible；不能只剩不可操作的装饰点。
- 连线数量与投影响应一致，边在默认、hover/selected 和浅色背景下可辨；不得用隐藏边来制造轻量感。
- 当前/选中节点用现有 accent 克制强调，其他节点退后；孤立节点仍可识别，但不以大段常驻徽标抢层级。
- fitView 后所有节点在关系舞台内，不因缩小节点而裁切标签或把标签缩至不可读。

### 5.3 范围与筛选

- 全局/局部范围控件必须有真实选中语义（`aria-pressed`、radio 或等价语义），不能只靠颜色。
- 筛选 disclosure 具有 `aria-expanded`、`aria-controls`，打开后首焦点进入第一个有效筛选控件。
- Escape/显式关闭卸载或关闭 disclosure，并按 `.interface-design/system.md` 回到本次真实触发器；不得固定猜测按钮或落到 `body`。
- query/tag/local focus 提交继续生成既有 lower-camel payload；局部范围缺少焦点时保留 fail-closed notice，不伪造结果。
- loading、unavailable、truncated/diagnostic 语义保留，呈现可以压低层级但不得吞掉。

### 5.4 节点打开与键盘

- 鼠标点击、Enter 和 Space 激活同一节点时，均只把响应中的 `relative_path` 交给既有 `onOpenMarkdown`。
- 节点具有清晰的 button/link 等价语义、可访问名称和 focus-visible；Tab 能进入关系舞台节点，Shift+Tab 能离开。
- Space 激活必须阻止页面/组壳意外滚动；一次按键只形成一次 open request。
- 重复激活同一节点仍使用既有 sequence 语义触发受限读取；不得把 node id 当路径。
- 节点打开后由既有父层切到 Syn 原生 Markdown；不得外部打开、创建第二标签系统或直接读写文件。

### 5.5 响应和动效

- `900×760` 四种左右栏折叠组合不得产生 document/body/shell/active group/Graph 横向 overflow。
- 筛选面打开时也必须完全位于当前 Graph 根内，控件不重叠、不截断主操作。
- reduced-motion 下不新增布局动画、漂浮、脉冲或强制平移；缩放/fitView 保持可用。
- 不新增 hardcoded hex、gradient、backdrop blur、装饰性 shadow 或第二套字体/间距 token。

## 6. Red-first

修改产品前，先建立并运行 Graph 浏览器合同和必要的正式合同红测，保留当前实现的精确反例。至少逐项记录：

1. 重复大标题/说明仍可见；
2. query/tag/local focus 四类表单常驻；
3. ReactFlow 舞台占比不足；
4. 节点宽度超过 `144 px`；
5. 节点高度超过 `48 px`；
6. 路径常驻；
7. 标签/孤立文字常驻；
8. 卡片阴影或粗左边存在；
9. 选中态没有在真实 ReactFlow selected wrapper 上生效；
10. Enter/Space 不能各形成一次与 click 等价的 typed open；
11. 节点缺可访问 button/link 等价语义或 focus-visible；
12. 筛选面没有 actual-opener Escape 回焦合同。

报告总断言、失败数、失败名称、关键 computed style/量尺与至少一张当前卡片阵列 red 图。不得先改实现再补红；不得用 `force` click、runner 注入 `.focus()`、隐藏整个 Graph 或修改 fixture 造 red/green。

若某条现状已意外通过，如实记录；不得为了凑失败数故意破坏。关键要求是当前卡片形态和键盘缺口必须有实现前证据。

## 7. Green pure-synthetic 浏览器矩阵

复用真实 React、真实生产 CSS和冻结 fixture，每个场景使用 fresh browser context，至少覆盖：

1. `1440×900`：全局 Graph 常态；连续舞台、轻节点/边、fitView、常态 compact chrome；
2. `1180×760`：打开筛选面，证明首焦点、ARIA、bounds、query/tag 控件和 Escape 回实际触发器；
3. `1180×760`：选择局部范围、指定 synthetic Markdown 焦点并更新，证明 lower-camel graph payload、局部结果和关闭后的 compact 状态；
4. `1180×760`：对一个节点分别以 click、Enter、Space 激活，证明每次只打开响应中的 `relative_path`，重复节点 sequence 仍前进；
5. `900×760`：左右栏展开的 Graph，Tab/Shift+Tab、focus-visible、全部层级无横向 overflow；
6. `900×760` reduced-motion：至少一种侧栏折叠组合，筛选面与关系舞台均无 overflow/动画回归。

每个 context 记录：

- mount 前目标 origin localStorage 为空；
- 最终只允许既有 R3B 可丢弃 UI chrome 偏好，不得新增 Graph 坐标/选择持久化；
- Graph root、compact chrome、disclosure、ReactFlow viewport、全部节点和 active group 的 bounds/computed style；
- node width/height、可见文本、shadow/border、focus-visible、selected style；
- response nodes/edges 与 DOM nodes/edges 数；
- activeElement、ARIA、Tab 路径、Escape 回焦和 node activation；
- document/body/shell/active group/Graph overflow。

精确 read allowlist：

```text
knowledge_workspace_snapshot
knowledge_workspace_graph
knowledge_workspace_read_markdown
```

所有 create/write/delete/import/restore、未知 command、外部请求、console error 和 page error 必须为 `0`。不得调用 search/canvas 证明本包。

至少输出：

- red runner、red JSON、red PNG；
- green runner、green JSON；
- `01-1440-global-graph.png`；
- `02-1180-filter-disclosure.png`；
- `03-1180-local-graph.png`；
- `04-900-keyboard-focus.png`。

逐图只对照 R0 01/03 的 Graph 维度和 R2 `02-1440-graph.png`。不得把 Graph 单项通过写成完整 R0、“Obsidian 1:1”、整个知识 UI 或真实 App 通过。

## 8. 必跑验证

从 `prototypes/productized-desktop-shell`：

1. 聚焦 `knowledge-graph` 合同，报告新增断言和既有 typed handoff；
2. 聚焦 `knowledge-workbench-shell`、`native-knowledge-workspace` 与 `knowledge-canvas` 合同；
3. `npm run typecheck`；
4. `npm run test:offline-interaction`，确认冻结 runner 仍精确登记并执行 37 个入口；首个测试打印的 `offline interaction tests passed: 15` 不是入口数量。

从仓库根：

5. `node scripts/harness/workbench-shape-gate.js --mode baseline`；
6. `node scripts/harness/workbench-shape-gate.js --mode check`，既有 `17 / 5 / 5` 原样报告，只接受零新增类别/finding；
7. `node scripts/harness/workbench-shape-gate.hardcoded-hex.selftest.js`；
8. `node scripts/harness/workbench-shape-gate.machine-face.selftest.js`；
9. `git diff --check`；
10. `git diff --cached --name-only` 必须为空；
11. 回算全部冻结只读 hash；
12. 只关闭本包启动的 Vite/浏览器，确认 5173 无 listener。

## 9. 结论枚举

- `PASS_N2R_R3D_GRAPH_CONVERGENCE / NEEDS_GUIDANCE_REVIEW / NOT_REAL_APP_ACCEPTED`
- `NEEDS_N2R_R3D_REWORK / NOT_ACCEPTED`
- `BLOCKED_N2R_R3D_BASELINE_DRIFT`
- `BLOCKED_N2R_R3D_WRITE_OWNERSHIP_CONFLICT`
- `BLOCKED_N2R_R3D_BROWSER_OR_PORT`
- `BLOCKED_N2R_R3D_SCOPE_EXPANSION`
- `BLOCKED_N2R_R3D_TYPED_CLIENT_OR_WRITE_BOUNDARY`

即使执行线 PASS，也只能回交 `NEEDS_GUIDANCE_REVIEW / NOT_REAL_APP_ACCEPTED`，不得自行接受 R3D、继续活动栏/右栏或启动真实 App。

## 10. 立即停止条件

出现任一情况立即停止并按对应枚举回交：

- 必须修改 fixture、runner、Native workspace、组壳/reducer、Canvas、活动栏、右栏、Rust、typed client 或依赖才能完成；
- 必须增加第二图框架、随机/持久化布局或真实文件写入；
- Graph 请求出现未授权 command、write、unknown 或外部网络；
- 红测没有真实覆盖卡片形态/键盘缺口，或 green 只能靠 force click/runner focus/隐藏 Graph 通过；
- staged 非空、目标 hash 漂移、并行 writer、5173 非本包占用；
- 需要启动 Syn/Tauri/Obsidian/Codex CLI/MCP、读取真实 store/vault 或进入 N6；
- 任一冻结文件需要变化。

禁止 stage、commit、push、reset、clean、stash、checkout、删除或覆盖上游 evidence。不得终止或修改非本包进程。

## 11. 必须回传

1. HEAD、staged、冻结 hash、三个窄写文件 provenance、端口和唯一写所有权；
2. red-first 总断言、失败数、逐项反例、量尺/computed style 和 red 图；
3. compact chrome/disclosure、轻节点/边、selected/focus 样式和 typed node activation 的最小实现；
4. `1440/1180/900` 舞台占比、节点尺寸、overflow 与四张 green 图；
5. click/Enter/Space、Tab/Shift+Tab、actual-opener Escape 回焦、ARIA 与 reduced-motion；
6. localStorage、read allowlist、各 command 次数，以及 write/unknown/external/console/page error 五类零值；
7. 聚焦合同、typecheck、37-entry runner、shape、自测、diff/staged/hash/端口；
8. 实际写入文件与未完成项；
9. 新 catch；没有则明确写“零新 catch”；
10. 精确停止结论。

执行线不得自行验收。指导线收到回交后会独立核三个产品/合同 hunk、raw 浏览器报告和四张图，再决定是否接受 R3D。

## 12. 实际执行回填

- 结果：`PASS_N2R_R3D_GRAPH_CONVERGENCE / NEEDS_GUIDANCE_REVIEW / NOT_REAL_APP_ACCEPTED`；执行线未自行验收，未进入 R3E。
- red-first：浏览器 `EXPECTED_RED`，`35` 项断言 / `17` 项失败；正式 Graph 合同也先红，精确失败为“中央标签已表达关系图，静态壳不得重复大标题”。卡片阵列实测舞台占比 `0.6402312331045955`、最大节点 `208.049×94.447`，red JSON 和 PNG 已保存。
- 最小实现：移除重复标题/说明，常态 chrome 收到 `40px`；详细筛选进入 actual-opener disclosure；全局/局部使用 `aria-pressed`；节点改为只显标题的原生 button，click/Enter/Space 只交接响应 `relative_path`；键盘 focus 驱动真实 ReactFlow wrapper selected；继续使用确定性 ReactFlow 布局、既有 lower-camel typed client 和 fail-closed 局部语义。
- green 浏览器：`8` 个 fresh context / `75` 项断言 / `0` 失败；`1440/1180/900` 舞台占比分别为 `0.9520383693045563 / 0.9423631123919308 / 0.9423631123919308`，节点最大 `136×40`，6 节点/5 边均可见且在舞台内；四张最终图已逐图检查。
- 交互与边界：Tab/Shift+Tab、focus-visible、真实 selected wrapper、click/Enter/Space、Space 零滚动、disclosure 首焦点、Escape/显式关闭 actual-opener 回焦、ARIA、reduced-motion 和所有外层 overflow 均通过。主 green 证据没有 force click 或 runner 注入 `.focus()`。
- fixture 限制：局部 Graph 精确发送 `{\"scope\":\"local\",\"focusRelativePath\":\"notes/visual-baseline.md\",\"query\":\"合成\",\"tag\":\"synthetic\"}`，但冻结 fixture 对所有 Graph 请求仍返回 global `6 / 5` 投影；未改写响应，未把它冒充真实局部数据。
- 数据边界：每个 context mount 前 localStorage 为空，最终只含既有 `syn-native-knowledge-workspace-ui-v1` UI chrome 偏好；command 总数为 snapshot `8`、graph `9`、read_markdown `3`；search/canvas/create/write/unknown/external/console/page error 均为 `0`。
- 门禁：Graph `19 / 0`、Workbench shell `R3B 16 / 0`、Native fixed-client、Canvas `R3C 14 / 0 + R3C-R1 4 / 0`、typecheck 和 37-entry offline runner 全过；shape baseline `pass 17 / 5 / 5`，check 维持既有 `exit 1, 17 / 5 / 5` 且零新增；selftest `13 / 13`、`18 / 18`；diff check 通过，staged 为空，17 个冻结只读 hash 全匹配。
- 收尾：HEAD 未漂移；三个窄写目标无打开句柄；只停止本包 Vite，最终 5173 无 listener；零新 catch，未改 catch log。
- 实际产品/合同写入：`KnowledgeGraphView.tsx`、`styles.css` 的 Graph 专属 selector、`knowledge-graph.test.tsx`。另新增本包 red/green raw runner、JSON、截图和诊断记录、本 evidence，并只回写本段实际结果。
- 证据：`evidence/2026-07-26-l3-syn-n2r-r3d-graph-convergence-verification-v1.md`。
- 未完成：指导线独立复核、真实 Syn/Tauri App、真实 store/vault、完整 R0、R3D 接受、R3E 与发布验收；这些均不在本执行线结论内。
