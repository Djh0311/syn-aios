# L3 Syn N2R-R3C Canvas-first 视觉收敛验证 evidence v1

- 日期：2026-07-26
- 执行线回交：**`PASS_N2R_R3C_CANVAS_FIRST_CONVERGENCE / NEEDS_GUIDANCE_REVIEW / NOT_REAL_APP_ACCEPTED`**
- 指导裁决：**`NEEDS_N2R_R3C_REWORK / NOT_ACCEPTED`**
- 证据级别：真实 React、真实生产 CSS、pure-synthetic fixture、fresh localhost browser context；**不是**真实 Syn/Tauri App、真实 store/vault、完整 R0、Gate 0、十二项或发布验收。
- 本执行线只完成并回交 R3C，不自行验收，不进入 R3D。

## 1. 开工 preflight、冻结 hash 与写所有权

- 固定工作目录：`/Users/yoyi/workspace/product-line`。
- HEAD：`e9ad7f3a204a1ebb11ce26c1e8c05b19c04c0991`，与任务包冻结值一致。
- `git diff --cached --name-only`：开工无输出，staged 为空。
- `lsof -n -P -iTCP:5173 -sTCP:LISTEN`：开工无输出，5173 无 listener。
- 白名单四个产品/测试文件均无 open handle；没有发现并行 writer。共享 worktree 已有大量上游 dirty/untracked WIP，本包没有 reset、clean、stash、checkout、覆盖、批量格式化或批量 staging。
- 已完整读取 `AGENTS.md`、`CURRENT.md`、`AUTHORITY.md` 和唯一施工包；按 `interface-design` 冻结检查点实施：Canvas 为唯一主焦点，沿用暖纸/墨色/青绿 token，低对比边界，最多一层局部 raised 面，4/8/12px 节奏，不引入常驻 catalog、页面大标题、整行 toolbar 或空 inspector。
- 未启动真实 Syn/Tauri/Codex/MCP/store/vault，未使用真实知识数据。

写入前 14 个冻结 SHA-256 全部匹配：

| 文件 | 冻结 SHA-256 | 开工 |
| --- | --- | --- |
| `prototypes/productized-desktop-shell/src/views/knowledge/KnowledgeCanvasView.tsx` | `af336f7c1176a60791e003f3162456bb31ae6ea6c5cbb6672b3fafafaa973c9c` | MATCH |
| `prototypes/productized-desktop-shell/src/styles.css` | `0cf85dbd37ca02c17143f6b43c1bb7500d30520238680fa0c770993fe07b3bf0` | MATCH |
| `prototypes/productized-desktop-shell/tests/knowledge-canvas.test.tsx` | `53481206e5aa70f9b21ee677602ac2cbece70e5cfadbcc9ec58e5998c5f5aab6` | MATCH |
| `prototypes/productized-desktop-shell/tests/knowledge-workbench-visual-fixture.tsx` | `219637ff4493150ea4fd0aac7e64cb6e232f2840d80560208da7e014d8d63fee` | MATCH |
| `prototypes/productized-desktop-shell/src/views/knowledge/NativeKnowledgeWorkspace.tsx` | `eafd9edccc3a2cd62e99cfa211809d77b206c826907ca07976ef4c419f4b4464` | MATCH |
| `prototypes/productized-desktop-shell/src/lib/knowledgeWorkbenchLayout.ts` | `8fb9faf01ef9dae509a6cffa4e14fd62031bb9744ea8e44eb1d1476e4f9ee220` | MATCH |
| `prototypes/productized-desktop-shell/src/lib/knowledgeWorkspace.ts` | `81eddc02206035a9d1bf1e3954d5ea19b1864a71b27ffea43efafa5a2fd51fb0` | MATCH |
| `prototypes/productized-desktop-shell/tests/knowledge-workbench-shell.test.tsx` | `2cbf2b61a86731096a876636c27119066c0e10d5bfaac81e8c6c7322660541da` | MATCH |
| `prototypes/productized-desktop-shell/tests/knowledge-workbench-visual-fixture.html` | `8a58ac0d02d9c33a9ca151f4ec1e653ccd3c6d79096581efd339a62c1fcec23c` | MATCH |
| `prototypes/productized-desktop-shell/scripts/run-offline-interaction-test.mjs` | `798daf13722875befa22eb233c476b957d241005a3b08267ed30eb54e0363082` | MATCH |
| `docs/design/2026-07-25-l3-obsidian-core-desktop-reference-and-migration-r0-v1.md` | `141ff343312fff91346e6e170f1aea2d07cda5fa2abdb3fde8b840b16f9ed4d9` | MATCH |
| `evidence/2026-07-26-l3-syn-n2r-r2-isolated-browser-visual-baseline-v1.md` | `8dd432266f406e96c69359196ea138eede85891d041ed8f98a3907eed55b6c7a` | MATCH |
| `tasks/2026-07-26-l3-syn-n2r-r3b-central-tab-groups-and-split-package-v1.md` | `9ededa2f7dab53f9fe465aba7187336d25012611cadaf024075a119f0173600b` | MATCH |
| `evidence/2026-07-26-l3-syn-n2r-r3b-central-tab-groups-and-split-verification-v1.md` | `de571a7a85b07d8152989ecb9e7e368834d85c8a6a00f11189c5d2cfaa4de2ef` | MATCH |

收尾前再次回算 10 个冻结只读面：`NativeKnowledgeWorkspace`、两个 helper、workbench test、fixture HTML、正式 runner、R0、R2 evidence、R3B task/evidence 均仍为上述冻结值，没有 drift。

## 2. Red-first

- 聚合报告：[red-first-contracts.json](raw/2026-07-26-l3-syn-n2r-r3c-canvas-first/red-first-contracts.json)
- 静态合同：**14 项 / 9 失败**，结果 `EXPECTED_RED`。失败类别为：compact chrome、页面级标题退场、旧静态三列退场、常驻 catalog 退场、连续 stage、stage 内浮动工具、完整可访问路径、紧凑 saved 状态、旧静态 toolbar 退场。
- 浏览器合同：**14 条断言 / 9 失败**，结果 `EXPECTED_RED`。旧产品实测 Canvas root `656×573`、stage `440×415`、stage 高度占比 `0.7240213717`；旧三列、页面标题、常驻 catalog、整行 toolbar、空 inspector 均存在，compact chrome/触发器 ARIA/连续 stage/内部浮动工具均缺失。
- red 图：[旧三列 Canvas](raw/2026-07-26-l3-syn-n2r-r3c-canvas-first/red-1180-canvas-three-column.png)
- red 浏览器原始报告：[red-browser-evidence.json](raw/2026-07-26-l3-syn-n2r-r3c-canvas-first/red-browser-evidence.json)
- red 阶段真实 Tauri 写调用 `0`、未识别调用 `0`。

实现后同一 `knowledge-canvas` 聚焦入口输出 R3C **14 项 / 0 失败**；既有 static shell、四方法 typed client、JSON Canvas patch、draft/conflict/late-request 合同继续通过。

## 3. 实际实现

### 3.1 Canvas-first 层级

- static 与浏览器壳都只保留一个 `.native-knowledge-canvas` 和一个 React Flow 根；旧 `.native-canvas-browser-grid`、页面级大标题、常驻 catalog/新建表单、整行工具条和默认空 inspector 退场。
- 顶部为一条 `data-canvas-chrome="compact"` 紧凑 chrome，保留完整可访问 Canvas 路径、saved/dirty/conflict 状态、按需“画布”触发器、显式保存与重新读取。
- `data-canvas-stage="continuous"` 为唯一主视觉面；节点类型和新增动作成为 stage 内有可访问名称的浮动工具，与既有 React Flow 控件处于同一画布层。
- 仅使用既有 Syn token、边界与 surface 语义；没有 fixed Canvas 面板、硬编码颜色、渐变、新阴影系统、品牌资产或新依赖。

### 3.2 按需面板、焦点与 ARIA

- 文件 catalog/新建表单只有显式触发后才渲染在 Canvas 根内；默认 DOM 中无面板和可聚焦后代。触发器具备 `aria-expanded`、`aria-controls`，打开后首个有效 Canvas 文件动作获焦；Escape 后面板卸载并回到同一触发器。
- inspector 只在节点选中后渲染，与选中节点通过 `aria-controls=native-canvas-node-inspector` 和 `aria-expanded=true` 关联；关闭或 Escape 后 inspector 卸载，焦点落到稳定 Canvas stage，不落 `body`。
- 文件面板与 inspector 互斥；面板关闭后可交互后代为 `0`。
- 双组窄 Canvas 通过 Canvas-in-group 条件规则占满组内第二行和剩余高度，没有修改 R3B reducer、组壳、tab、split ratio 或 storage 结构。

### 3.3 单一事实源与写边界

- 保留 `snapshot/readCanvas/createCanvas/writeCanvas` 四方法 typed seam，没有 raw invoke、第二文档模型、第二保存路径或平行图框架。
- text/file/link/group、边、拖动、局部字段、未知字段 roundtrip、相对引用、删节点同步删边、整数坐标继续由既有合同覆盖。
- add/edit/move/connect/delete 仍只改本地 draft；显式保存才可进入既有 mtime/hash CAS 写路。冲突、刷新、迟到请求和导航继续 fail closed。
- 浏览器取证没有点击“保存 Canvas”或“新建 JSON Canvas”，没有写真实或 synthetic 文件。

## 4. Synthetic-only 浏览器取证

- runner：[browser-evidence.mjs](raw/2026-07-26-l3-syn-n2r-r3c-canvas-first/browser-evidence.mjs)
- 原始报告：[browser-evidence.json](raw/2026-07-26-l3-syn-n2r-r3c-canvas-first/browser-evidence.json)
- 最终：**7 个 fresh context / 131 条断言 / 0 失败 / 10 张最终实图**。
- 每个 context mount 前均 `localStorageEmptyBeforeMount=true`。
- 精确 read mock allowlist 只有 `knowledge_workspace_graph`、`knowledge_workspace_read_canvas`、`knowledge_workspace_read_markdown`、`knowledge_workspace_search`、`knowledge_workspace_snapshot`；各场景实际命令次数逐项保留在 raw JSON。
- 每个 context 均为真实 Tauri 写调用 `0`、未识别调用 `0`、外部请求 `0`、console error `0`、page error `0`。
- localStorage 只写既有可丢弃 key `syn-native-knowledge-workspace-ui-v1`：普通场景 `2` 次、Canvas/Graph split 场景 `10` 次；最新值均为规范化 `version: 2` chrome/tab 偏好，不含 Canvas/Markdown body 或 draft。
- 所有场景 document、body、shell、central surface/active group panel 横向 overflow 均为 `false`。

| 场景与实图 | 量尺/状态 | 焦点、ARIA、隔离与视觉结论 |
| --- | --- | --- |
| [01 1180 连续 Canvas](raw/2026-07-26-l3-syn-n2r-r3c-canvas-first/01-1180-canvas-continuous.png) | root `656×694`；stage `656×654`，占比 `0.94236`；chrome `656×40`；面板均关闭；浮动工具 `160×38` 且在 stage 内 | 一个 Canvas 根、一个 React Flow；完整路径与 saved 状态存在；R0 04/08 因本图是双侧栏展开的单组场景均记 `GAP`。 |
| [02 文件面板打开](raw/2026-07-26-l3-syn-n2r-r3c-canvas-first/02-1180-file-panel-open.png)、[03 Escape 后](raw/2026-07-26-l3-syn-n2r-r3c-canvas-first/03-1180-file-panel-escaped.png) | 面板 `320×231`，`position:absolute`，完全在 Canvas root 内，5 个可交互后代 | 打开时 `aria-expanded=true`、controls 真实存在，首焦点为 `canvas/visual-baseline.canvas`；Escape 后面板卸载、交互后代 `0`、`aria-expanded=false`，焦点回到同一触发器。 |
| [04 inspector 打开](raw/2026-07-26-l3-syn-n2r-r3c-canvas-first/04-1180-node-inspector-open.png)、[05 关闭后](raw/2026-07-26-l3-syn-n2r-r3c-canvas-first/05-1180-node-inspector-closed.png) | inspector `300×490`，`position:absolute`，在 root 内，9 个可交互后代 | 选中节点 `baseline` 与 inspector 的 controls/expanded 关联完整；关闭后 inspector 卸载，焦点回稳定 stage，`isBody=false`，画布恢复全宽。 |
| [06 1180 Canvas/Graph 双组](raw/2026-07-26-l3-syn-n2r-r3c-canvas-first/06-1180-canvas-graph-split.png) | 2 个真实组；Canvas 在左、Graph 在右；Canvas root `325×694`、stage `325×619`，占比 `0.89193` | R0 04 在“左 Canvas、右 Graph、外侧目录/上下文、连续中央空间”维度为 `PASS`；Syn 组 chrome、节点尺度和冻结 Graph 工具仍不同，完整 R0 04 仍是 `GAP`。 |
| [07 900 双侧栏](raw/2026-07-26-l3-syn-n2r-r3c-canvas-first/07-900-double-sidebar-canvas.png) | root `436×694`、stage 占比 `0.89193` | 紧凑 chrome、文件触发器、保存/重读和浮动工具均可达；无横向 overflow。 |
| [08 900 右栏折叠](raw/2026-07-26-l3-syn-n2r-r3c-canvas-first/08-900-right-collapsed-canvas.png)、[09 双栏折叠](raw/2026-07-26-l3-syn-n2r-r3c-canvas-first/09-900-both-collapsed-canvas.png) | 右折叠 root 宽 `636`；双折叠 root 宽 `856`；42px 活动 rail 保留；无横向 overflow | 右折叠后焦点在活动栏右侧上下文切换按钮。双折叠最后一步因当前聚焦的左栏控件随折叠退场，raw 最终焦点记录为 `body`；这是冻结 R3B 壳的既有行为，不作为 Canvas 面板回焦通过的证据，本包未越界修改，R3B 聚焦合同仍通过。R0 08 在“活动 rail 保留、折叠宽度归还中央舞台”维度为 `PASS`；Syn 顶栏、节点卡、状态栏和场景内容仍不同。 |
| [10 reduced motion inspector](raw/2026-07-26-l3-syn-n2r-r3c-canvas-first/10-1180-reduced-motion-inspector.png) | `matchMedia(reduce)=true`；inspector 仍为 root 内 `300×490` | 文件面板 Escape 回焦、节点关联和 inspector 关闭后 stage 回焦继续成立；无写、未知调用、外部请求或浏览器错误。 |

除 R0 04 对应的双组图和 R0 08 对应的折叠图外，其余 context 都诚实记录为 `GAP`，没有用 DOM、bounds 或断言通过替代人工视觉判断。本包只关闭 Canvas-first P1，不声称 Obsidian 1:1、完整 R0 或 N2R 完成。

浏览器工具说明：通用 Playwright CLI wrapper 的 `--help` 在受限环境中等待 30 秒仍无返回，已终止该探测；没有安装依赖，改用仓库已有 `playwright-core` 执行 raw runner。只访问 `127.0.0.1:5173`，收尾只终止本包启动的 Vite；最终 5173 无 listener。

## 5. 新 catch

本包发现并记录 **1 个新真实 catch**：

- 首轮 green 浏览器中，文件面板 locator 判为 visible，但真实点击被中央工作区/顶栏拦截。诊断证明单组 Canvas root 只有 `656×40`、workspace 高度为 `0`；没有使用 force click 造绿。
- 以必要的 Canvas-in-group grid-row/高度规则修复后，root 为 `656×694`、workspace 为 `656×654`、面板为 `320×231`，入口中心的 `elementFromPoint` 与真实点击都命中目标按钮。
- 原始诊断：[panel-layering-diagnostic.json](raw/2026-07-26-l3-syn-n2r-r3c-canvas-first/panel-layering-diagnostic.json)、[runner](raw/2026-07-26-l3-syn-n2r-r3c-canvas-first/panel-layering-diagnostic.mjs)、[诊断图](raw/2026-07-26-l3-syn-n2r-r3c-canvas-first/diagnostic-1180-file-panel-layering.png)。
- 已在 `docs/harness-catch-log.md` 追加 `N2R-R3C 文件面板真实命中层`；经验是浮层不能只核 DOM/locator visible，还必须核 bounds、containment 和真实命中层。

## 6. 必跑门禁

| 验证 | 实际结果 |
| --- | --- |
| red-first `knowledge-canvas` 静态合同 | EXPECTED RED：`14 项 / 9 失败`。 |
| red-first browser | EXPECTED RED：`14 条 / 9 失败`，red 图与 raw JSON 已保留。 |
| green `knowledge-canvas` 定向合同 | PASS：既有 static/typed/local JSON Canvas 合同继续通过；R3C `14 / 0`。 |
| `knowledge-workbench-shell` 定向合同 | PASS：R3B `16 / 0`。 |
| `native-knowledge-workspace` 定向合同 | PASS。 |
| `knowledge-graph` 定向合同 | PASS。 |
| `npm run typecheck` | PASS，exit `0`。 |
| `npm run test:offline-interaction` | PASS，exit `0`；正式 runner 精确登记并执行 **37** 个既有入口，本包未新增或伪报入口。 |
| synthetic browser | PASS：`7 contexts / 131 assertions / 0 failed`，10 张最终实图。 |
| shape baseline | PASS：`17 errors / 5 warnings / 5 info`。 |
| shape check | 预期非零 exit `1`：仍为 `17 / 5 / 5`。冻结 R3B evidence 的包前基线同为 `17 / 5 / 5`，因此类别与 finding 零净增；这不是全仓 shape 全绿。 |
| hardcoded-hex selftest | PASS：`13 / 13`；当前 Canvas 相关硬编码 hex violation 为 `0`。 |
| machine-face selftest | PASS：`18 / 18`；当前 Canvas machine-face error 为 `0`。 |
| retired family 检查 | 当前 Canvas 相关 retired family 为 `0`。 |
| `git diff --check` | PASS，无输出。 |
| `git diff --cached --name-only` | PASS，无输出，staged 为空。 |
| 进程收尾 | 只终止本包启动的 Vite；最终 5173 无 listener。 |

一次聚焦合同命令曾因临时输出文件名中的 backtick 被 shell 解释，在测试启动前以 `command not found`/esbuild 临时路径错误退出；没有产品写入。改用普通字符串拼接后同一聚焦套件通过。这是命令构造噪声，不是产品失败，也没有被计入 green 证据。

shape 的 `17 / 5 / 5` 是既有全仓债务；本包不修、不掩盖，也不把 check 的非零退出写成 PASS。

## 7. 实际写入文件

产品与测试：

1. `prototypes/productized-desktop-shell/src/views/knowledge/KnowledgeCanvasView.tsx`
2. `prototypes/productized-desktop-shell/src/styles.css`，仅 Canvas selectors 与必要 Canvas-in-group 窄规则
3. `prototypes/productized-desktop-shell/tests/knowledge-canvas.test.tsx`
4. `prototypes/productized-desktop-shell/tests/knowledge-workbench-visual-fixture.tsx`，仅 Canvas metrics/selectors

证据、控制与 catch：

5. `evidence/raw/2026-07-26-l3-syn-n2r-r3c-canvas-first/red-first-contracts.json`
6. `evidence/raw/2026-07-26-l3-syn-n2r-r3c-canvas-first/red-browser-evidence.mjs`
7. `evidence/raw/2026-07-26-l3-syn-n2r-r3c-canvas-first/red-browser-evidence.json`
8. `evidence/raw/2026-07-26-l3-syn-n2r-r3c-canvas-first/red-1180-canvas-three-column.png`
9. `evidence/raw/2026-07-26-l3-syn-n2r-r3c-canvas-first/panel-layering-diagnostic.mjs`
10. `evidence/raw/2026-07-26-l3-syn-n2r-r3c-canvas-first/panel-layering-diagnostic.json`
11. `evidence/raw/2026-07-26-l3-syn-n2r-r3c-canvas-first/diagnostic-1180-file-panel-layering.png`
12. `evidence/raw/2026-07-26-l3-syn-n2r-r3c-canvas-first/browser-evidence.mjs`
13. `evidence/raw/2026-07-26-l3-syn-n2r-r3c-canvas-first/browser-evidence.json`
14. `evidence/raw/2026-07-26-l3-syn-n2r-r3c-canvas-first/01-1180-canvas-continuous.png`
15. `evidence/raw/2026-07-26-l3-syn-n2r-r3c-canvas-first/02-1180-file-panel-open.png`
16. `evidence/raw/2026-07-26-l3-syn-n2r-r3c-canvas-first/03-1180-file-panel-escaped.png`
17. `evidence/raw/2026-07-26-l3-syn-n2r-r3c-canvas-first/04-1180-node-inspector-open.png`
18. `evidence/raw/2026-07-26-l3-syn-n2r-r3c-canvas-first/05-1180-node-inspector-closed.png`
19. `evidence/raw/2026-07-26-l3-syn-n2r-r3c-canvas-first/06-1180-canvas-graph-split.png`
20. `evidence/raw/2026-07-26-l3-syn-n2r-r3c-canvas-first/07-900-double-sidebar-canvas.png`
21. `evidence/raw/2026-07-26-l3-syn-n2r-r3c-canvas-first/08-900-right-collapsed-canvas.png`
22. `evidence/raw/2026-07-26-l3-syn-n2r-r3c-canvas-first/09-900-both-collapsed-canvas.png`
23. `evidence/raw/2026-07-26-l3-syn-n2r-r3c-canvas-first/10-1180-reduced-motion-inspector.png`
24. 本 evidence
25. 本任务包，只回写实际状态与执行结果
26. `docs/harness-catch-log.md`，只追加上述一条真实 catch

最终关键 SHA-256：

| 文件 | 最终 SHA-256 |
| --- | --- |
| `prototypes/productized-desktop-shell/src/views/knowledge/KnowledgeCanvasView.tsx` | `bbf13ecd68cfe48ef2e04eca28d0e1d2546b5c88f17c7bbe0e173a6ea0deaa25` |
| `prototypes/productized-desktop-shell/src/styles.css` | `8d74a3245d07477a44dd29d7fabd67a0cd01300a880bf4db6752dede105d03ed` |
| `prototypes/productized-desktop-shell/tests/knowledge-canvas.test.tsx` | `693ec0e86759deeae93d5d76c091ea5befd9f62dd515087e2c61dbff8d42a8e3` |
| `prototypes/productized-desktop-shell/tests/knowledge-workbench-visual-fixture.tsx` | `ea27e80ec9fff83fc630e4b80074915a39ee1ecf2f5217c3e3940bb51eb929ee` |
| `evidence/raw/2026-07-26-l3-syn-n2r-r3c-canvas-first/browser-evidence.json` | `3ad095fa26ae81b0eeaea62aa2e5acaa0a3b9178ba22767d09d055644e8c87e1` |

未修改 `NativeKnowledgeWorkspace.tsx`、两个 helper、workbench test、fixture HTML、正式 runner、Graph/Maintenance 产品与测试、Rust/Cargo、App、CURRENT、AUTHORITY、计划、决策、依赖、真实 store/vault 或 `.codex`。未 stage、commit、push、reset、clean、stash 或 checkout。

## 8. 遗留与停止边界

- 本包没有进入 R3D，没有修改 Graph/Canvas 之外的内容组件，也没有引入第二份 Markdown/Canvas 草稿或真实数据需求。
- R0 04/08 只在任务包允许的中央组/连续空间与折叠宽度归还维度 PASS；Graph 内容、活动栏图标、右栏内容、全屏像素高保真和完整 Obsidian 视觉仍是 GAP。
- 双侧栏折叠最终焦点为 `body` 的既有壳行为已如实保留，不被拿来证明 Canvas 面板回焦；Canvas 文件面板、inspector 和 reduced-motion 回焦均有独立直接证据。
- 未执行、未接受：真实 Syn/Tauri App、真实 store/vault、Home-only discovery、Gate 0、十二项、发布验收或 N2R 整体验收。

执行线精确回交：

**`PASS_N2R_R3C_CANVAS_FIRST_CONVERGENCE / NEEDS_GUIDANCE_REVIEW / NOT_REAL_APP_ACCEPTED`**

## 9. 指导独立复核（2026-07-26）

### 9.1 已确认成立

- 四个授权产品/测试文件的最终 SHA-256 与本 evidence 一致；十个冻结只读面未漂移。
- `browser-evidence.json` 确为 `PASS / 7 contexts / 131 assertions / 0 failed`；每个 context 的 localStorage、read allowlist、写/未知/外部调用和 console/page error 记录自洽。
- 指导逐张查看十张最终图并对照 R0 04/08：Canvas 已从常驻三列管理页收为连续舞台；Canvas/Graph 并排和侧栏折叠宽度归还只在本包允许维度成立，Graph、活动栏、右栏和完整 R0 仍是可见 GAP。
- 指导独立复跑 `npm run typecheck` 与 37-entry `npm run test:offline-interaction`，均 exit `0`；runner 中 Canvas 仍输出 R3C `14 / 0`。
- shape baseline 为 PASS、check 为预期非零，均为既有 `17 errors / 5 warnings / 5 info`；hardcoded-hex `13 / 13`、machine-face `18 / 18`。
- `git diff --check` 无输出；staged 为空；HEAD 仍为 `e9ad7f3a204a1ebb11ce26c1e8c05b19c04c0991`；指导复现所用 Vite 已关闭，5173 无 listener。

### 9.2 新发现的未覆盖反例

指导使用同一 synthetic fixture 做了一次 fresh read-only 浏览器复现：

1. 打开 Canvas 空态并将焦点置于中央“选择画布”按钮；
2. 点击该按钮打开文件面板，首焦点正常进入面板；
3. 按 Escape，面板正常卸载；
4. 关闭后的 `document.activeElement` 是顶部 `data-canvas-file-trigger`“画布”，不是实际 opener“选择画布”。

源码原因可直接定位到 `KnowledgeCanvasView.tsx`：顶部按钮和空态按钮都可调用 `showFilePanel`，但 `closeFilePanel` 固定只把焦点送到 `filePanelTriggerRef`。原 browser runner 只从顶部按钮打开面板，因此 `131 / 0` 没覆盖这个反例。

这违反任务包 §5.2、§5.4 的“选择/新建成功或 Escape 后确定性回焦”“回到同一触发器”。它不推翻已成立的 Canvas-first 视觉、隔离和几何证据，但阻止 R3C 整包接受。

### 9.3 精确裁决

**`NEEDS_N2R_R3C_REWORK / NOT_ACCEPTED`**

只建议另行授权最窄 R3C-R1：保存实际 opener，覆盖顶部“画布”和空态“选择画布”两个入口的 red-first 浏览器合同；其余 R3C 实现与已成立证据冻结。当前不得进入 R3D、真实 Syn/Tauri App、真实 store/vault、Gate 0 或对话三句重验。
