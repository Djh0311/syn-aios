# L3 Syn N2R-R3D Graph 关系舞台收敛验证 evidence v1

- 日期：2026-07-26
- 执行结果：**`PASS_N2R_R3D_GRAPH_CONVERGENCE / NEEDS_GUIDANCE_REVIEW / NOT_REAL_APP_ACCEPTED`**
- 证据级别：真实 React、真实生产 CSS、冻结 pure-synthetic fixture、fresh localhost Chromium context；**不是**真实 Syn/Tauri App、真实 store/vault、完整 R0、R3D 指导接受或发布验收。
- 执行边界：只完成 R3D Graph；未进入 R3E，未修改活动栏、右栏、Markdown/Canvas 内容组件、Rust、App、typed client、真实数据或治理文件。

## 1. 开工 preflight、provenance 与写所有权

- 工作目录固定为 `/Users/yoyi/workspace/product-line`。
- HEAD 开工与收口均为 `e9ad7f3a204a1ebb11ce26c1e8c05b19c04c0991`。
- 开工与收口 `git diff --cached --name-only` 均无输出，staged 始终为空。
- 开工时 `127.0.0.1:5173` 无 listener；本包随后独占启动 Vite。浏览器取证结束后只关闭该 Vite，收口 `lsof -n -P -iTCP:5173 -sTCP:LISTEN` 无输出。
- `KnowledgeGraphView.tsx` 与 `knowledge-graph.test.tsx` 是上游已有 untracked WIP，`styles.css` 是上游已有 tracked dirty WIP。本包先保存逐文件基线，再在三者上做白名单内窄修改，没有覆盖或重建其他人的工作。
- 三个窄写文件的派发 hash 精确为 `ca8d88c…b6b83`、`d4d87d8…98b16`、`8d74a324…d03ed`；收口新 hash 分别为：

| 窄写文件 | 收口 SHA-256 |
| --- | --- |
| `prototypes/productized-desktop-shell/src/views/knowledge/KnowledgeGraphView.tsx` | `c9151ffaf0ba406956ea2fd3ef40ff8d737d684578ecb263469eacea668dd781` |
| `prototypes/productized-desktop-shell/tests/knowledge-graph.test.tsx` | `a38f4359169ab363aaca0082c908854809bd4b62978b7e1f3e51be7f343e97f9` |
| `prototypes/productized-desktop-shell/src/styles.css` | `fcb28929d693b9f41721a51d201b9744f5db8d439dbfd694b4381c88c59d396a` |

- 开工与收口对三个窄写文件执行 `lsof -- <targets>` 均无输出，未见并行文件句柄或 writer。
- 开工时全部 20 个派发 hash 精确匹配；收口时 3 个窄写目标形成上述预期新 hash，其余 17 个冻结只读文件仍逐项匹配：

| 冻结只读文件 | 收口 SHA-256 | 结果 |
| --- | --- | --- |
| `.interface-design/system.md` | `2a035b99b0e21b33fb4af259aad0f12515406fede89a9395b9fa70ba868bd766` | MATCH |
| `prototypes/productized-desktop-shell/tests/knowledge-workbench-visual-fixture.tsx` | `ea27e80ec9fff83fc630e4b80074915a39ee1ecf2f5217c3e3940bb51eb929ee` | MATCH |
| `prototypes/productized-desktop-shell/tests/knowledge-workbench-visual-fixture.html` | `8a58ac0d02d9c33a9ca151f4ec1e653ccd3c6d79096581efd339a62c1fcec23c` | MATCH |
| `prototypes/productized-desktop-shell/src/views/knowledge/NativeKnowledgeWorkspace.tsx` | `eafd9edccc3a2cd62e99cfa211809d77b206c826907ca07976ef4c419f4b4464` | MATCH |
| `prototypes/productized-desktop-shell/src/views/knowledge/KnowledgeCanvasView.tsx` | `42054605e642100752a109830a8f7601a46eaef68f1bc70c7ca48e5353d6405e` | MATCH |
| `prototypes/productized-desktop-shell/tests/knowledge-canvas.test.tsx` | `802dd4a90ceac4f491ddbf390f8e5734f02984af87c920c170b17d0c723f263c` | MATCH |
| `prototypes/productized-desktop-shell/src/lib/knowledgeWorkbenchLayout.ts` | `8fb9faf01ef9dae509a6cffa4e14fd62031bb9744ea8e44eb1d1476e4f9ee220` | MATCH |
| `prototypes/productized-desktop-shell/src/lib/knowledgeWorkspace.ts` | `81eddc02206035a9d1bf1e3954d5ea19b1864a71b27ffea43efafa5a2fd51fb0` | MATCH |
| `prototypes/productized-desktop-shell/scripts/run-offline-interaction-test.mjs` | `798daf13722875befa22eb233c476b957d241005a3b08267ed30eb54e0363082` | MATCH |
| `prototypes/productized-desktop-shell/package.json` | `08a3abc466e2dd51f946380badb6519b4273353d8d279efa43b1cf6086d87e65` | MATCH |
| `prototypes/productized-desktop-shell/package-lock.json` | `781cb1d94eaaeec0d071f156f9aeca65400acce8bd811c9c95cd4d45cf700bc0` | MATCH |
| 上游 R3C task | `3e6096177e1d6c98c2a01002b76a8ed32c5277f393fb4388dc093ccd15b97b8f` | MATCH |
| 上游 R3C evidence | `a189496276363f916237a443e2906ce63bea040404f263a258bc690eb3dd2fbc` | MATCH |
| R3C-R1 task | `8db62fa056232a3d878d6f74ba6357cbef29d82b2a2743d8322c7f8a821406ee` | MATCH |
| R3C-R1 evidence | `c56753c6a071b288dc647bb09b964ee88fe01ddc90171092be65380e153050c2` | MATCH |
| R0 设计参考 | `141ff343312fff91346e6e170f1aea2d07cda5fa2abdb3fde8b840b16f9ed4d9` | MATCH |
| R2 基线 evidence | `8dd432266f406e96c69359196ea138eede85891d041ed8f98a3907eed55b6c7a` | MATCH |

## 2. Red-first

产品实现修改前先运行 [red-browser-evidence.mjs](raw/2026-07-26-l3-syn-n2r-r3d-graph-convergence/red-browser-evidence.mjs)，原始结果见 [red-browser-evidence.json](raw/2026-07-26-l3-syn-n2r-r3d-graph-convergence/red-browser-evidence.json) 和 [red-1440-card-array.png](raw/2026-07-26-l3-syn-n2r-r3d-graph-convergence/red-1440-card-array.png)。

- 浏览器 red：**`EXPECTED_RED`，35 项断言 / 17 项失败 / 18 项通过**。
- `1440×900` 实测：Graph root `868×601.125`，常态 chrome `868×77.938`，ReactFlow `868×384.859`，舞台占比 `0.6402312331045955`。
- 最大节点 `208.049×94.447`；6 个节点、5 条边。
- 重复 `h2=1`、说明段 `=1`、常驻详细筛选控件 `=3`、disclosure opener `=0`。
- 可见路径 `=6`、可见标签/孤立文字 `=6`、卡片阴影 `=6`、`2px` 粗左边 `=6`。
- 旧节点 active element 是 `div[role=group]`，无可访问名称、无可见 focus indicator，真实 ReactFlow wrapper 未进入 selected。
- click、Enter、Space 均没有形成 typed Markdown read。

17 个失败名称如下：

1. Graph 内仍重复大标题或说明；
2. query/tag/local-focus 仍常驻；
3. 常态 chrome 高于 `44px`；
4. ReactFlow 占比低于 `0.74`；
5. 节点宽度超过 `144px`；
6. 节点高度超过 `48px`；
7. 路径作为常驻可见元数据；
8. 标签/孤立说明作为常驻可见元数据；
9. 节点仍有卡片阴影或粗左强调边；
10. 不存在带 actual opener 的筛选 disclosure；
11. 节点没有 button/link 等价语义；
12. 键盘聚焦没有可见 focus indicator；
13. selected 样式未作用在真实 ReactFlow selected wrapper；
14. Enter 未形成一次 typed Markdown open；
15. Space 未形成一次 typed Markdown open；
16. click 未形成一次 typed Markdown open；
17. click 未按投影 `relative_path` 交接。

正式合同也按 red-first 先改测试、后改产品。构建成功后运行退出 `1`，精确失败为：

```text
[knowledge-graph] 中央标签已表达关系图，静态壳不得重复大标题
```

原始记录见 [red-formal-contract.txt](raw/2026-07-26-l3-syn-n2r-r3d-graph-convergence/red-formal-contract.txt)。

首次在受限 sandbox 内启动 Chromium 遇到 `SIGABRT/EPERM`，属于浏览器进程权限边界；随后仅对同一 synthetic localhost 取证使用获准浏览器执行，未因此修改产品、fixture 或验证口径。

## 3. 最小实现

1. 移除 Graph 内重复大标题和说明；Graph 根直接填满当前中央标签组，常态只保留 `40px` 范围、状态、筛选入口和刷新 chrome。
2. query、tag、local focus 移入 Graph 根内的按需 disclosure；入口保留稳定 `aria-expanded` / `aria-controls`。打开后首焦点自然进入 query；Escape、显式关闭和成功应用都回到本次真实 connected opener，只有 opener 已失效时才回退到 Graph stage。
3. 全局/局部按钮用 `aria-pressed` 表达真实状态；局部缺焦点继续 fail closed。提交仍只调用既有 `loadKnowledgeGraph` lower-camel typed client。
4. 节点改为原生 `button`，可见内容只留标题；路径、标签和孤立语义进入 accessible label。click、Enter、Space 走同一 `onOpenMarkdown(relativePath)`，Space 阻止页面滚动。
5. 键盘 focus 只在 `:focus-visible` 路径更新本组件 selected state，使真实 `.react-flow__node.selected` wrapper 生效；鼠标点击不因先发生 focus rerender 而丢失 click。
6. 节点布局继续使用既有 ReactFlow，并用确定性椭圆位置 helper；不引入随机、坐标持久化或第二图框架。6 节点位置固定为 `(110,0)、(205,80)、(205,240)、(110,320)、(15,240)、(15,80)`。
7. CSS 只改 `.native-knowledge-graph`、`.native-graph-*`、`.react-flow__node` 和 Graph 专属组合 selector；没有改全局 token、Markdown/Canvas/maintenance computed style、活动栏、左右栏或非 Graph media rule。
8. 继续呈现 loading、unavailable、truncated/diagnostic 和 `6 笔记 · 5 关系` 状态，没有吞掉只读投影语义。

## 4. Green pure-synthetic 浏览器矩阵

最终主取证运行 [green-browser-evidence.mjs](raw/2026-07-26-l3-syn-n2r-r3d-graph-convergence/green-browser-evidence.mjs)，原始结果见 [green-browser-evidence.json](raw/2026-07-26-l3-syn-n2r-r3d-graph-convergence/green-browser-evidence.json)。

- 结果：**`PASS_SYNTHETIC_GRAPH_EVIDENCE`**。
- 8 个 fresh browser context，**75 项断言 / 0 失败**。
- 主 green runner 没有 `force` click、runner 注入 `.focus()`、隐藏 Graph 或改写 fixture。
- `green-activation-inspect.mjs` 中的 programmatic click 只用于开发期定位“focus rerender 取消物理 click”的原因，明确排除在 green 结论之外；最终 click/Tab/Enter/Space 证据均来自主 runner 的自然交互。

| 场景 | Graph root / chrome / ReactFlow | 舞台占比 | 节点与边 | 结果 |
| --- | --- | --- | --- | --- |
| `1440×900` 全局常态 | `868×834 / 868×40 / 868×794` | `0.9520383693045563` | `6 / 5`，最大 `136×40` | 全部节点在舞台内，全部边可见 |
| `1180×760` disclosure | `656×694 / 656×40 / 656×654` | `0.9423631123919308` | `6 / 5`，最大 `136×40` | disclosure `520×173.281`，完全位于 Graph root 内 |
| `1180×760` 局部请求 | `656×694 / 656×40 / 656×654` | `0.9423631123919308` | `6 / 5`，最大 `136×40` | 当前焦点节点以 `aria-current=page` 和现有 accent 呈现 |
| `900×760` 双栏展开 | `436×694 / 436×40 / 436×654` | `0.9423631123919308` | `6 / 5`，最大 `136×40` | 标题 `12px`、未裁切，Tab/Shift+Tab 通过 |
| `900×760` 双栏折叠 + reduce | `856×694 / 856×40 / 856×654` | `0.9423631123919308` | `6 / 5`，最大 `136×40` | disclosure `x=371, y=79, 520×173.281`，动画/过渡均 `none/0s` |

所有场景共同记录：

- 重复标题 `0`、重复说明 `0`、常驻详细筛选控件 `0`；
- 可见节点元数据 `0`、节点 shadow `0`、粗左边 `0`；
- 边使用现有色，computed `stroke-width=1.1px`、`opacity=0.72`，5 条均可见；
- document、body、shell、active group 与 Graph root 均无横向 overflow；
- ReactFlow 内部 transformed canvas 的 `scrollWidth` 另行原样记录，但它不是 document/shell/active-group/Graph-root overflow；每个节点的 bounds 均实际位于可见舞台内。

四张最终图已逐张人工检查：

- [01-1440-global-graph.png](raw/2026-07-26-l3-syn-n2r-r3d-graph-convergence/01-1440-global-graph.png)：连续关系舞台、40px chrome、轻节点和细边。
- [02-1180-filter-disclosure.png](raw/2026-07-26-l3-syn-n2r-r3d-graph-convergence/02-1180-filter-disclosure.png)：按需筛选面完整留在 Graph 根内。
- [03-1180-local-graph.png](raw/2026-07-26-l3-syn-n2r-r3d-graph-convergence/03-1180-local-graph.png)：局部范围和当前 synthetic Markdown 焦点的克制 accent。
- [04-900-keyboard-focus.png](raw/2026-07-26-l3-syn-n2r-r3d-graph-convergence/04-900-keyboard-focus.png)：窄舞台中的原生节点 focus-visible 与真实 wrapper selected。

这些图只证明 R3D Graph 维度，不外推为完整 R0、Obsidian 1:1、整个知识 UI 或真实 App 通过。

## 5. ARIA、焦点、typed activation 与数据边界

- disclosure 打开后 active element 是 `input[aria-label="关系图文字筛选"]`；Escape 与显式关闭都回到同一个 actual opener DOM 节点。
- `900×760` 自然 Tab 进入 `button[data-graph-node-action]`；accessible name 包含标题、后端 `relative_path` 和标签。Shift+Tab 能自然离开节点。
- 键盘焦点 computed style：`outline: solid 2px rgb(42, 107, 94)`；真实 ReactFlow wrapper `selected=true`。选中背景/边框为 `rgb(234,244,241) / rgb(42,107,94)`，未选中为白色/既有 hairline。
- click、Enter、Space 分别在独立 fresh context 对 `notes/visual-baseline.md` 形成且只形成一次 `knowledge_workspace_read_markdown`；父层选择的 source path 三次都精确相同。Space 前后 body/document scroll 均为 `0`。
- 三种激活各用 fresh context，避免既有父层当前标签去重抑制第一次 read；正式 sequence helper 合同另证重复同节点请求的 sequence 单调前进。
- 局部更新只发出一次：

```json
{"scope":"local","focusRelativePath":"notes/visual-baseline.md","query":"合成","tag":"synthetic"}
```

- 冻结 fixture 对每个 Graph 请求都返回同一份 global `6 nodes / 5 edges` 投影。runner 没有改写或重标响应；因此本包只证明 exact lower-camel 局部请求、局部 UI 状态和当前焦点呈现，**不把冻结 global 响应冒充真实局部数据结果**。
- 8 个 context 均在 mount 前确认目标 origin localStorage 为空；最终只出现既有 `syn-native-knowledge-workspace-ui-v1` 可丢弃 UI chrome 偏好。没有 Graph 坐标、Graph 选择或查询持久化。
- 精确 read allowlist：

```text
knowledge_workspace_snapshot
knowledge_workspace_graph
knowledge_workspace_read_markdown
```

- 实际 command 总数：

| command | 次数 |
| --- | ---: |
| `knowledge_workspace_snapshot` | 8 |
| `knowledge_workspace_graph` | 9 |
| `knowledge_workspace_read_markdown` | 3 |

- `search=0`、`canvas=0`；create/write/delete/import/restore 总数 `0`。
- 五类总计：`write=0`、`unknown=0`、`external=0`、`console error=0`、`page error=0`。

## 6. 正式合同与门禁

| 验证 | 实际结果 |
| --- | --- |
| 聚焦 `knowledge-graph` | PASS，`19 / 0`；含 compact chrome、ARIA、disclosure、deterministic positions、原生节点键盘激活、lower-camel typed handoff 与 sequence |
| `knowledge-workbench-shell` | PASS，`R3B 16 / 0` |
| `native-knowledge-workspace` | PASS，fixed client contract |
| `knowledge-canvas` | PASS，`R3C 14 / 0 + R3C-R1 4 / 0` |
| `npm run typecheck` | PASS，退出 `0` |
| `npm run test:offline-interaction` | PASS；冻结 runner 精确登记并执行 `37` 个入口；首条打印仍是单项 `offline interaction tests passed: 15` |
| shape `--mode baseline` | PASS，既有 `Errors 17 / Warnings 5 / Info 5` |
| shape `--mode check` | 退出 `1`，既有债务仍为 `17 / 5 / 5`；零新增类别、零新增 finding。该项按现状原样报告，不冒充 PASS |
| hardcoded-hex selftest | PASS，`13 / 13`；新增 hardcoded hex UI finding `0` |
| machine-face selftest | PASS，`18 / 18`；新增 machine-face UI finding `0` |
| retired-style / scope audit | 新增 retired-style finding `0`；三个产品/合同 hunk 均能映射到 R3D 白名单 |
| `git diff --check` | PASS，无输出 |
| `git diff --cached --name-only` | PASS，无输出 |
| 17 个冻结只读 hash | 全部 MATCH |
| 三个窄写目标 writer | 收口 `lsof` 无输出 |
| 5173 | 只停止本包 Vite；收口无 listener |

## 7. 实际写入、未完成项与停止结论

实际产品/合同写入：

1. `prototypes/productized-desktop-shell/src/views/knowledge/KnowledgeGraphView.tsx`
2. `prototypes/productized-desktop-shell/src/styles.css` 中 Graph 专属 selector/组合 selector
3. `prototypes/productized-desktop-shell/tests/knowledge-graph.test.tsx`

实际证据/回填写入：

1. `evidence/raw/2026-07-26-l3-syn-n2r-r3d-graph-convergence/**`
2. 本 verification evidence
3. R3D 任务包的状态与实际执行回填

未改 `docs/harness-catch-log.md`：**零新 catch**。

R3D synthetic-only 执行范围内没有未完成的实现或验证项。仍未完成且不在本执行线权限内的是：指导线独立核三个产品/合同 hunk、raw JSON 和四张图；真实 Syn/Tauri App、真实 store/vault、完整 R0、R3D 接受、R3E 与发布验收。

精确停止结论：

**`PASS_N2R_R3D_GRAPH_CONVERGENCE / NEEDS_GUIDANCE_REVIEW / NOT_REAL_APP_ACCEPTED`**

执行线不自行验收，至此停止。
