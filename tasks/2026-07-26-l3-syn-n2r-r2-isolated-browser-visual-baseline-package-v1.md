# 任务包：L3 Syn N2R-R2 隔离浏览器视觉基线与差距审计 v1

- 日期：2026-07-26
- 状态：**ACCEPTED_N2R_R2_BASELINE / NEEDS_N2R_R3_VISUAL_CONVERGENCE / NOT_REAL_APP_ACCEPTED**
- 负责人：独立知识前端执行线
- 指导/验收：当前总指导对话
- 派发任务：Codex task `019f99b2-aa25-77d0-93b2-766088679137`
- 前置：`N2R-R1` 已由指导线接受为 `ACCEPTED_N2R_R1_OFFLINE / NOT_REAL_APP_ACCEPTED`
- 小阶段计划：`docs/plans/2026-07-23-l3-syn-native-knowledge-workspace-small-stage-plan-v2.md`
- 冻结参考：`docs/design/2026-07-25-l3-obsidian-core-desktop-reference-and-migration-r0-v1.md`

## 0. Kickoff

- 任务：用真实 React 组件、真实生产 CSS 和纯合成知识数据，在隔离浏览器上下文中采集当前 R1 单壳的视觉、尺寸、滚动和键盘焦点基线；逐项对照冻结的 Obsidian `1.12.7` 参考，形成可直接驱动下一包的差距矩阵。
- 本包是**只读产品审计 + 测试夹具**，不是视觉修复包。发现缺口后记录，不在本包修改产品代码或样式。
- 用户 2026-07-26 回复“可以”，授权形成并派发本包，也授权仅用于本包的本地 Vite + 隔离浏览器渲染。该授权不包含真实 Syn/Tauri App、Obsidian、真实 store/vault、Codex CLI/MCP 或对话三句重验。
- 完成标准：§7 的真实渲染、量尺和键盘矩阵完成，§8 的验证通过，并形成 `PASS_N2R_R2_VISUAL_BASELINE` 或 `NEEDS_N2R_R3_VISUAL_CONVERGENCE` 的诚实结论。

## Authority and plan alignment

- authority_chain: `AGENTS.md` -> `CURRENT.md` -> knowledge/conversation parallel decision -> native knowledge route decision -> conversation-first master plan -> native knowledge small-stage plan -> frozen Obsidian reference -> accepted R1 task -> this R2 task
- plan_anchor: `docs/plans/2026-07-23-l3-syn-native-knowledge-workspace-small-stage-plan-v2.md#r2-隔离浏览器视觉基线与差距审计`
- existing_before_new: 现有可注入知识客户端、Tauri mock、Vite 和视觉夹具先例已经足够；不新增产品能力。
- capabilities_touched: none — 本包只新增隔离视觉测试夹具与 evidence，不修改 Code Map 中的产品能力。
- forbidden_alternatives: 真实 Syn/Tauri、用户 Chrome profile、真实 store/vault、产品样式修复与对话 live。

## 1. 已知与未知

R1 离线结构已经接受，但以下事实仍未知，本包只回答这些未知：

1. 真实浏览器像素下，中间内容是否仍是唯一焦点；
2. `1440 × 900`、`1180 × 760`、`900 × 760` 下各栏实际尺寸、内部滚动和文本截断情况；
3. Markdown、Preview、Graph、Canvas、命令面板和快速打开是否仍处于同一壳内；
4. 折叠与 overlay 的真实 Tab 路径、焦点进入和回焦是否可用；
5. 当前视觉与冻结 Obsidian 参考的结构、密度、比例、层级和状态差距。

## 2. 设计审计基线

### 2.1 使用者、任务与感受

- 使用者：在 macOS 上长时间整理、搜索、编辑和关联知识的单人桌面用户。
- 核心任务：不离开当前知识上下文，在文件、笔记、关系图和 Canvas 之间持续切换。
- 目标感受：紧凑、安静、连续、工具化；不是卡片仪表盘或多个页面模块的拼盘。
- 唯一焦点：中央内容工作区。

### 2.2 产品域与视觉约束

- Domain：vault、目录树、Markdown、双链、反链、属性、图谱、Canvas、来源、审计。
- Color world：暖纸、米白、石墨、墨色、旧木、青玉状态、朱砂确认；实现只能消费 Syn 既有语义 token。
- Signature：当前笔记驱动右侧属性/反链/来源上下文，确认式 AI 和审计不另造第二知识仪表盘。
- Rejecting：
  - 卡片仪表盘 → 固定桌面单壳；
  - 多容器纵向堆叠 → 中央标签/工作模式；
  - 每块同等强调 → 中央内容胜出、侧栏和状态区降级；
  - 复制 Obsidian 品牌资产 → Syn 文案、token 和通用语义图标。

视觉审计使用以下固定口径：

- 层级：中央内容必须在对比度、面积和空间上胜出；活动栏和侧栏不得与正文争抢。
- 密度：桌面工具密度，主要间距按既有 4 px 系统；不新增营销式留白。
- 深度：沿用既有低对比边界与轻微 surface shift；不混入厚边框、多重阴影或渐变。
- 字体：沿用 Syn 宋黑分工；不为仿 Obsidian 改用其品牌字体。
- 状态：default、hover、active、focus-visible、disabled、loading、empty、error/conflict 至少在本包可达面中如实记录。

## 3. 为什么本包不直接进入真实 Syn

现有 isolated runtime profile 已隔离 App 数据、索引、workflow、Codex DB 和 vault，但其证据明确指出 Knowledge/Agents 路由的 WebView `localStorage` 尚未纳入该隔离合同。直接从 Home 导航到 Knowledge 会把“未隔离的浏览器状态”与 synthetic runtime 混在一起。

因此本包选择成本最低、可回退且证据足够的路径：

1. 用仓库现有 Vite 在 localhost 渲染；
2. 使用新的隔离浏览器 context，不使用用户 Chrome profile；
3. 直接挂载真实 `NativeKnowledgeWorkspace`；
4. 主工作区通过现有 injectable `KnowledgeWorkspaceClient` 注入纯合成数据；
5. Graph/Canvas 等 Tauri 调用使用现有 `@tauri-apps/api/mocks` 的精确 command mock；
6. 未识别 command 和任何写 command 一律拒绝，不回落真实 host。

该证据属于**真实浏览器渲染**，不属于真实 Syn/Tauri App 验收，也不替代后续 I5 Home-only discovery。

## 4. 脏工作树与写入边界

本包起点 HEAD：

`e9ad7f3a204a1ebb11ce26c1e8c05b19c04c0991`

### 4.1 开工冻结 SHA-256

| 文件 | 2026-07-26 指导冻结 SHA-256 |
| --- | --- |
| `src/views/KnowledgeBaseView.tsx` | `3fa18f9fbba0f6c797cc568132389ac2c02a0e12c7dc4861a27ab4f6c2309e58` |
| `src/views/knowledge/NativeKnowledgeWorkspace.tsx` | `456153d284712beadd46053b4871417ef33fa8bcacbf4a86a117b641774b3e0a` |
| `src/views/knowledge/KnowledgeWorkbenchShell.tsx` | `2ab66a5b00941e3cf0a083e3ed3d24b309f8b7fe5037dd41d8743cf9d2d41c21` |
| `src/lib/knowledgeWorkbenchLayout.ts` | `cc3a9f8811edb3b6ac231a37738a3b6f0bfba0794aa66f81d4b3a14015fd083a` |
| `src/styles.css` | `fe8bfc74e8d946255fd2850d7ec9218928c0c10995ae6d21450eb7a262bdea71` |
| `tests/knowledge-workbench-shell.test.tsx` | `60b7f8d0e8ed618e0bc51ab857542e55d8c5882dc39d01270a127ca5567e5f2d` |
| `vite.config.ts` | `68f0c2b5a2fa45af381a43eaaff7a7054b83822d358fac186d3bd3ea0824b506` |

相对路径以 `prototypes/productized-desktop-shell/` 为根。上述生产文件全部只读；任一 hash 漂移或存在无法归属的并行 hunk，停止为 `BLOCKED_N2R_R2_BASELINE_DRIFT`。

### 4.2 精确写入白名单

允许新增：

- `prototypes/productized-desktop-shell/tests/knowledge-workbench-visual-fixture.tsx`
- `prototypes/productized-desktop-shell/tests/knowledge-workbench-visual-fixture.html`
- `evidence/2026-07-26-l3-syn-n2r-r2-isolated-browser-visual-baseline-v1.md`
- `evidence/raw/2026-07-26-l3-syn-n2r-r2-visual-baseline/`：只放 synthetic-only 截图和不含正文的量尺 JSON

允许更新：

- 本任务包：只更新执行状态和实际结果；
- `docs/harness-catch-log.md`：仅出现新的真实 catch 时追加。

禁止修改所有生产 `src/**`、`src-tauri/**`、`styles.css`、既有测试/runner、依赖、Vite 配置、CURRENT、AUTHORITY 和其他任务包/evidence。禁止 `git add`、commit、push、reset、clean、stash。

## 5. 视觉夹具合同

夹具必须：

1. 直接渲染真实 `NativeKnowledgeWorkspace` 和真实 `styles.css`，不得复制生产 DOM/CSS 造假；
2. 使用纯合成目录、Markdown、属性、标签、反链、搜索、Graph 和 Canvas；不得读取仓库外文件、默认 store、真实 vault 或浏览器持久 profile；
3. 用 fresh browser context，开场确认目标 origin 的 localStorage 为空；结束后关闭 context；
4. 对 mock command 使用精确 allowlist。未识别 command、create/write/delete/import/restore 等写类 command 全部 fail closed，并在 evidence 记录写调用计数为 0；
5. 提供可重复选择的场景入口，至少覆盖：
   - Markdown/source；
   - Preview；
   - Graph；
   - Canvas；
   - 左侧 Search；
   - command overlay；
   - quick-open overlay；
   - 左栏折叠；
   - 右栏折叠；
6. 暴露隐藏量尺节点，仅含 viewport、五区 bounds、document/body/shell overflow、当前场景、activeElement 语义标签、写调用计数；不得存合成正文、绝对路径或原始 IPC payload；
7. 真实 hover/focus 截图只能通过浏览器交互进入，不得靠夹具硬编码 `:hover` 或伪造 focus class。

若 Graph/Canvas 需要修改生产组件才能接受 fixture 数据，立即停止为 `BLOCKED_N2R_R2_FIXTURE_REQUIRES_PRODUCTION_CHANGE`，不得在本包扩白。

## 6. 视觉与几何矩阵

### 6.1 必采截图

| 视口 | 必采场景 |
| --- | --- |
| `1440 × 900` | Markdown/Preview 主工作区、Graph、command overlay |
| `1180 × 760` | Search + Preview、Canvas、左栏折叠 |
| `900 × 760` | 双栏展开 Markdown、右栏折叠、quick-open overlay |

截图必须只含 synthetic 数据。若出现任何真实标题、路径、ID、store 警告或用户内容，立即关闭页面，不保存该截图，状态为 `BLOCKED_N2R_R2_NON_SYNTHETIC_SURFACE`。

### 6.2 量尺合同

- activity rail：`40–44 px`；
- status bar：`24–28 px`；
- `1440` 左栏：`220–288 px`；右栏：`185–240 px`；
- `1180` 左栏：`210–260 px`；右栏：`175–220 px`；
- `900` 左栏：`180–220 px`；右栏：`160–200 px`；
- 三个视口的中央工作区宽度都必须大于 0；`900` 双栏展开时至少 `380 px`；
- html、body 和 fixture 外壳不得出现主页面纵向滚动或横向溢出；
- 左栏内容、中央内容、右栏内容在长内容场景下各自能够内部滚动；
- Graph、Canvas、编辑和预览切换时 shell bounds 不得改变为第二个根级容器；
- 折叠侧栏轨宽为 0，活动栏仍保留，中央区横向接管空间。

### 6.3 视觉审计

逐截图给出 `PASS / GAP`，不使用模糊的“看起来差不多”：

1. 中央内容是否是唯一视觉焦点；
2. 是否存在重复标题、卡片仪表盘、大块营销留白或竞争主面板；
3. 活动栏、左右侧栏、中央标签区、状态栏的比例和密度是否接近 R0 参考；
4. 文本层级是否至少分清 primary / secondary / tertiary / muted；
5. 分隔是否低对比且可辨，不以厚边框或阴影堆叠制造层级；
6. active、hover、focus-visible、disabled 和 conflict 是否可辨；
7. overlay 是否浮于工作台之上而不变成第二页面；
8. Syn 品牌、语义 token 和通用图标是否保持；是否存在 Obsidian 商标、logo、原图标包或 CSS 复制迹象。

## 7. 键盘与焦点矩阵

至少真实执行并记录：

1. Tab 能进入活动栏、左栏、中央和右栏的可操作项；
2. 折叠左栏后，其子控件不再进入 Tab 路径，活动栏“文件/搜索/来源”能恢复；
3. 折叠右栏后，其子控件不再进入 Tab 路径，“上下文”能恢复；
4. 打开 command overlay 后首个可操作项获得焦点；
5. Escape 关闭 command overlay，焦点返回原触发按钮；
6. quick-open 同样满足进入、Escape 和回焦；
7. 真实 `:focus-visible` 清晰可辨，无遮挡或被 overflow 截断；
8. `prefers-reduced-motion: reduce` 下不出现依赖运动才能理解的状态。

R1 的静态 ARIA 合同不能替代这组真实浏览器观察。

## 8. 必跑验证

从 `prototypes/productized-desktop-shell`：

1. 新夹具 TypeScript/浏览器加载无 console error；
2. `npm run typecheck`；
3. 聚焦 `knowledge-workbench-shell` 合同；
4. `npm run test:offline-interaction`；

从仓库根：

5. `node scripts/harness/workbench-shape-gate.js --mode baseline`；
6. `node scripts/harness/workbench-shape-gate.js --mode check`，历史非零必须原样报告；
7. `git diff --check`；
8. `git diff --cached --name-only` 为空。

完成后只终止本包启动的 Vite/浏览器资源，并确认本包端口无残留。不得触碰其他工作区的进程或端口。

## 9. 结论枚举

- `PASS_N2R_R2_VISUAL_BASELINE`：九张规定截图、量尺、内部滚动和八项键盘矩阵均完成，未发现阻断后续视觉施工的夹具/安全问题。它只表示基线采集完成，不表示高保真最终通过。
- `NEEDS_N2R_R3_VISUAL_CONVERGENCE`：基线采集完成，但存在真实视觉/交互差距；按 P0/P1/P2 给出精确 selector、场景、视口和建议白名单。本枚举是本包的正常完成结果之一。
- `BLOCKED_N2R_R2_BASELINE_DRIFT`
- `BLOCKED_N2R_R2_FIXTURE_REQUIRES_PRODUCTION_CHANGE`
- `BLOCKED_N2R_R2_NON_SYNTHETIC_SURFACE`
- `BLOCKED_N2R_R2_BROWSER_OR_PORT`

不得使用 `REAL_APP_ACCEPTED`、`OBSIDIAN_1_TO_1_COMPLETE`、`N2R_COMPLETE`、`I5_PASS`、`GATE0_PASS` 或 `TWELVE_ITEMS_PASS`。

## 10. 并发、停止与禁止范围

- 本包运行 Vite/浏览器期间，对话线只能只读；不得启动三句真实 App 重验。
- 不启动 Tauri/Syn、cargo-tauri、Codex CLI/MCP server 或 Obsidian。
- 不读取/写入默认 store、真实 Codex DB、真实 vault、其他 vault 或用户 Chrome profile。
- 不发送消息、不调用 `tools/list` 或 knowledge tools，不创建卡、chain、worker。
- 不安装依赖，不使用网络，不操作登录/付费/Sync/Publish/插件。
- 不因视觉缺口修改产品文件；本包达到“看清现状并形成精确差距”即停止。

## 11. 必须回传

1. preflight HEAD、staged、冻结 hash 和进程/端口结果；
2. 实际新增文件及 synthetic-only 说明；
3. 九张截图路径和逐张 `PASS / GAP`；
4. 三个视口五区 bounds、overflow 和内部滚动实数；
5. 八项键盘/焦点结果；
6. mock 精确 allowlist、未识别调用和写调用计数；
7. typecheck、聚焦合同、完整离线 runner、shape、diff-check 实数；
8. P0/P1/P2 差距矩阵和建议的 N2R-R3 精确写入面；
9. 最终结论枚举；
10. 新 catch；没有则明确写“零新 catch”。

执行线不得自行验收。指导线必须独立看截图、核量尺/焦点证据并裁决是否接受该基线，以及是否派发 N2R-R3。

## 12. 实际执行结果（2026-07-26）

- 预检通过：HEAD `e9ad7f3a204a1ebb11ce26c1e8c05b19c04c0991`、暂存区为空、7 个冻结读取面 SHA-256 全部匹配，启动前本包端口及目标 Syn/Tauri/Vite/Codex CLI/MCP 进程均未占用。
- 只新增允许的浏览器 fixture、9 张 synthetic-only 截图、无正文/绝对路径/原始 IPC payload 的量尺 JSON 与本 evidence；没有修改产品 `src/**`、Rust、既有 test/runner、Vite、CURRENT 或 AUTHORITY。
- 使用 fresh browser context（共 5 个），直接渲染真实 `NativeKnowledgeWorkspace` 和 `styles.css`，以精确只读 mock allowlist 供给合成数据。每个 context mount 前 localStorage 为空；所有 context 的写调用、未识别调用和外部请求均为 0。
- 九图、内部 wheel 滚动与八项键盘行为已完成。几何合同为 `80 PASS / 1 GAP`：仅 `900×760` 的右栏折叠场景中 shell `scrollWidth=899`、`clientWidth=898`；html/body 无横向溢出，中央区为 `636 px`。因此没有修改产品代码，按本包正常完成枚举收敛为 `NEEDS_N2R_R3_VISUAL_CONVERGENCE`。
- 验证：浏览器 console/page error 皆为 0；typecheck、聚焦 `knowledge-workbench-shell` 合同与登记 37 入口的完整离线 runner 均通过；shape baseline 为 pass `17/5/5`，shape check 为既有非零 fail `17/5/5`；`git diff --check` 通过，暂存区仍为空。
- 本包启动的 `127.0.0.1:5173` Vite 已关闭，端口无残留。零新 catch。
- 证据：[N2R-R2 隔离浏览器视觉基线 evidence](../evidence/2026-07-26-l3-syn-n2r-r2-isolated-browser-visual-baseline-v1.md)。本条仅记录执行事实，不构成指导验收。

## 13. 指导独立复核与窄返工（2026-07-26）

指导线直接查看了 R2 九张截图、原始 `measurements.json` 和 R0 的 01/03/04/05/06/08 冻结实图，并核对夹具源码、冻结生产 hash、端口与 staged 状态。

### 13.1 已成立的执行证据

- 夹具确实直挂真实 `NativeKnowledgeWorkspace` 与真实生产 CSS；合成数据、fresh context、零外部请求、零写调用和零未识别调用成立；
- 三视口几何实数、内部滚动、侧栏移出 Tab/辅助技术路径、overlay 进入与 Escape 回焦证据可用；
- `900×760` 右栏折叠的 shell `899 / 898` 是真实可复现 GAP；
- 九张图尺寸正确，冻结生产 hash 未漂移，R2 没有修改生产文件，5173 已关闭，staged 为空。

### 13.2 不能接受的结论

执行 evidence 把除 `1 px` overflow 之外的截图全部判为视觉 PASS，但没有完成 §6.3 要求的 R0 逐图视觉对照：

1. `04-1180-search-preview.png` 只显示“打开快速搜索”按钮；该场景的 mock 计数没有 `knowledge_workspace_search`，所以没有搜索结果视觉证据，不能判定 Search 对照 PASS。
2. `03-1440-command-overlay.png` 是“新建 Markdown/目录”表单，不是 R0 05 的可搜索命令列表；`09-900-quick-open-overlay.png` 也没有 R0 06 的结果列表、当前选择和键盘提示。二者都不能仅凭 DOM overlay 与回焦通过就判定高保真视觉 PASS。
3. 活动栏仍以可换行文字按钮呈现；R0 §4.1 冻结的是紧凑图标 ribbon。900 视口顶部/活动栏文字已经发生断行，属于密度和可扫读性差距。
4. 当前 Graph 使用矩形卡片阵列，Canvas 保留常驻表单工具条和“原始 JSON 局部编辑”分区；它们与 R0 01/04 的轻量节点图、连续空间画布和浮动工具明显不同。
5. R0 的可关闭/新增标签、多标签组与分栏关系没有被 R2 截图和交互矩阵覆盖；右栏当前长列表式上下文也未与 R0 的上下文标签/折叠层级逐项比较。

因此，`1 px` overflow 是成立的 P1，但不是“唯一已证实视觉差距”。当前不能据此形成仅修改 `styles.css` 的窄 R3。

### 13.3 窄返工要求

本轮仍属于 R2，不授权任何产品代码或样式修改，也不派发 R3。执行线只允许：

1. 使用现有 R0 与 R2 实图重写 §3/§7 的逐图判定和 P0/P1/P2 差距矩阵，逐项列出 selector、证据图和建议 R3 写入面；
2. 仅为补齐 Search 覆盖，重新启动同一 synthetic-only fixture 一次，在 `1180×760` 实际输入合成查询并采集结果态截图；该 context 必须重新证明 localStorage 空、`knowledge_workspace_search >= 1`、外部请求/写调用/未识别调用均为 0；
3. 补图后关闭本线 Vite/浏览器，复核 5173、`git diff --check` 和 staged；不得重跑或触碰 Syn/Tauri/Obsidian、真实 store/vault、Codex CLI/MCP、对话三句；
4. 回交状态仍只能是 `NEEDS_N2R_R3_VISUAL_CONVERGENCE / NOT_ACCEPTED`；指导线验收完整差距矩阵后，才决定 R3 的拆包与授权。

## 14. §13 窄返工实际结果（2026-07-26）

- 执行线已重读 §13 与 evidence §9，并在开工前复核 HEAD `e9ad7f3a204a1ebb11ce26c1e8c05b19c04c0991`、暂存区为空、7 个冻结生产读取面 hash 未漂移、`127.0.0.1:5173` 空闲。
- 没有修改 fixture 或任何产品代码/样式，也没有修改 Rust、App、`CURRENT.md`、`AUTHORITY.md`、计划或 catch log。只更新本任务包、同一份 R2 evidence、同一 raw 目录的 `measurements.json`，并新增 synthetic-only `10-1180-search-results-preview.png`。
- 唯一新增的 `1180×760` fresh context 实际输入合成查询 `合成` 并执行搜索：`resultCount=16`、mount 前 localStorage 为空、`knowledge_workspace_search=1`、外部请求/写调用/未识别调用均为 `0`、console/page error 均为 `0`。该补图证明真实结果态执行，但 evidence 已明确其仍不同于 R0 左栏 Search 和 R0 quick-open 的选择/键盘提示，不把它重判为视觉 PASS。
- evidence §3 已以现有 R0/R2 实图重写逐图结论，§7 已列出 overflow、Search、command/quick-open、标签/分栏、Canvas、活动栏、Graph 与右栏层级的 P0/P1/P2 差距及候选 R3 写入面；所有候选都只供指导线后续拆包，未启动 R3。
- 本线 Vite 与浏览器已关闭；最终 `5173` 无 listener，`git diff --check` 通过，暂存区为空。初始 R2 的 typecheck、offline runner、shape、滚动/焦点结果保留为历史执行事实，窄返工未重跑它们。

本包当前仅可回交 **`NEEDS_N2R_R3_VISUAL_CONVERGENCE / NOT_ACCEPTED`**，等待指导线独立复核。

## 15. 指导最终验收（2026-07-26）

- 指导线独立查看了新增 `10-1180-search-results-preview.png` 及 `measurements.json` 的对应记录，确认该 fresh context 在 mount 前 localStorage 为空，查询 `合成` 得到 `16` 项结果，`knowledge_workspace_search=1`，外部请求、写调用、未识别调用、console error 与 page error 均为 `0`。
- 指导线复核了改写后的 evidence §3/§7。九张初始图不再被误判为高保真 PASS；`1 px` overflow、左栏 Search、command/quick-open、标签组/分栏、Canvas、活动栏、Graph 与右栏层级均已按 P1/P2、证据图和候选写入面如实登记。
- 本次接受的是 R2 的 synthetic-only 浏览器基线、隔离事实和差距审计，不是当前 Syn 界面高保真通过，也不是 Syn/Tauri 真实 App、真实 store/vault 或 N6 验收。
- §13 要求的产品零写入边界成立；窄返工没有新增另一项真实 catch。R3 仍未授权、未形成任务包、未派发。

指导最终结论为 **`ACCEPTED_N2R_R2_BASELINE / NEEDS_N2R_R3_VISUAL_CONVERGENCE / NOT_REAL_APP_ACCEPTED`**。上文执行线的 `NOT_ACCEPTED` 是交回指导前的必要停点，现由本节裁决覆盖。
