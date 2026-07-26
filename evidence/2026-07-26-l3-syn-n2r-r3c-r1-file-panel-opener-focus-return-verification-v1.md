# L3 Syn N2R-R3C-R1 文件面板实际触发器回焦窄返工验证 evidence v1

- 日期：2026-07-26
- 执行结果：**`PASS_N2R_R3C_R1_FOCUS_RETURN / NEEDS_GUIDANCE_REVIEW / NOT_REAL_APP_ACCEPTED`**
- 指导结论：**`ACCEPTED_N2R_R3C_R1_FOCUS_RETURN / ACCEPTED_N2R_R3C_CANVAS_FIRST / NOT_REAL_APP_ACCEPTED`**
- 证据级别：真实 React、真实生产 CSS、pure-synthetic fixture、fresh localhost Chromium context；**不是**真实 Syn/Tauri App、真实 vault/store、完整 R0、上游 R3C 或发布验收。
- 视觉结论：**`VISUAL_UNCHANGED / FOCUS_PATH_FIXED`**。
- 执行线只回交改动与证据，不自行验收，不进入 R3D。

## 1. 开工 preflight、权限与 provenance

- 工作目录固定为 `/Users/yoyi/workspace/product-line`。
- HEAD：`e9ad7f3a204a1ebb11ce26c1e8c05b19c04c0991`，与任务包冻结值一致。
- `git diff --cached --name-only`：无输出，派发时 staged 为空。
- `lsof -n -P -iTCP:5173 -sTCP:LISTEN`：无输出，派发时 5173 无 listener。
- `KnowledgeCanvasView.tsx`、`knowledge-canvas.test.tsx` 均是上游已有 untracked WIP；派发 SHA-256 分别精确为 `bbf13ecd68cfe48ef2e04eca28d0e1d2546b5c88f17c7bbe0e173a6ea0deaa25`、`693ec0e86759deeae93d5d76c091ea5befd9f62dd515087e2c61dbff8d42a8e3`。本包在该基线上做窄修改，没有覆盖或重建上游文件。
- 写所有权：任务包把上述两个文件的窄写面独占给本执行线；开工与收口 `lsof -- <两个文件>` 均无输出，未见并行文件句柄或 writer。
- `.interface-design/system.md` 不存在；按任务包冻结的 R3C 设计检查点执行，没有新建设计系统文件。
- 未启动 Syn/Tauri/真实 App，未读取真实 store/vault，未执行 create/save，未接触 Rust 或 typed client。

开工前全部 14 个派发 SHA-256 均匹配。收口时两个窄写文件形成预期新 hash，其余 12 个冻结只读文件仍逐项匹配：

| 文件 | 收口 SHA-256 | 结果 |
| --- | --- | --- |
| `prototypes/productized-desktop-shell/src/views/knowledge/KnowledgeCanvasView.tsx` | `42054605e642100752a109830a8f7601a46eaef68f1bc70c7ca48e5353d6405e` | 预期窄写 |
| `prototypes/productized-desktop-shell/tests/knowledge-canvas.test.tsx` | `802dd4a90ceac4f491ddbf390f8e5734f02984af87c920c170b17d0c723f263c` | 预期窄写 |
| `prototypes/productized-desktop-shell/tests/knowledge-workbench-visual-fixture.tsx` | `ea27e80ec9fff83fc630e4b80074915a39ee1ecf2f5217c3e3940bb51eb929ee` | MATCH |
| `prototypes/productized-desktop-shell/tests/knowledge-workbench-visual-fixture.html` | `8a58ac0d02d9c33a9ca151f4ec1e653ccd3c6d79096581efd339a62c1fcec23c` | MATCH |
| `prototypes/productized-desktop-shell/src/styles.css` | `8d74a3245d07477a44dd29d7fabd67a0cd01300a880bf4db6752dede105d03ed` | MATCH |
| `prototypes/productized-desktop-shell/src/views/knowledge/NativeKnowledgeWorkspace.tsx` | `eafd9edccc3a2cd62e99cfa211809d77b206c826907ca07976ef4c419f4b4464` | MATCH |
| `prototypes/productized-desktop-shell/src/lib/knowledgeWorkbenchLayout.ts` | `8fb9faf01ef9dae509a6cffa4e14fd62031bb9744ea8e44eb1d1476e4f9ee220` | MATCH |
| `prototypes/productized-desktop-shell/src/lib/knowledgeWorkspace.ts` | `81eddc02206035a9d1bf1e3954d5ea19b1864a71b27ffea43efafa5a2fd51fb0` | MATCH |
| `prototypes/productized-desktop-shell/scripts/run-offline-interaction-test.mjs` | `798daf13722875befa22eb233c476b957d241005a3b08267ed30eb54e0363082` | MATCH |
| `prototypes/productized-desktop-shell/package.json` | `08a3abc466e2dd51f946380badb6519b4273353d8d279efa43b1cf6086d87e65` | MATCH |
| `prototypes/productized-desktop-shell/package-lock.json` | `781cb1d94eaaeec0d071f156f9aeca65400acce8bd811c9c95cd4d45cf700bc0` | MATCH |
| 上游 R3C task | `3e6096177e1d6c98c2a01002b76a8ed32c5277f393fb4388dc093ccd15b97b8f` | MATCH |
| 上游 R3C evidence | `a189496276363f916237a443e2906ce63bea040404f263a258bc690eb3dd2fbc` | MATCH |
| R0 设计参考 | `141ff343312fff91346e6e170f1aea2d07cda5fa2abdb3fde8b840b16f9ed4d9` | MATCH |

## 2. Red-first

实现修改前先运行 [red-browser-evidence.mjs](raw/2026-07-26-l3-syn-n2r-r3c-r1-focus-return/red-browser-evidence.mjs)，原始结果见 [red-browser-evidence.json](raw/2026-07-26-l3-syn-n2r-r3c-r1-focus-return/red-browser-evidence.json)。

结果为 **`EXPECTED_RED`，6 项断言 / 4 项失败 / 2 项通过**：

| 矩阵项 | red 结果 | 实际 activeElement |
| --- | --- | --- |
| 顶部 opener → Escape | PASS | 与实际点击的顶部 opener 为同一 DOM 节点 |
| 空态 opener → Escape | FAIL | 错回顶部 opener，不是实际点击的空态 opener |
| 顶部 opener → 显式关闭 | PASS | 与实际点击的顶部 opener 为同一 DOM 节点 |
| 空态 opener → 显式关闭 | FAIL | 错回顶部 opener，不是实际点击的空态 opener |
| 顶部 opener → 选择成功 | FAIL | 回顶部 opener，不是连续 stage |
| 空态 opener → 选择成功 | FAIL | 空态 opener 已卸载，但仍回顶部 opener，不是连续 stage |

red 同时记录到空态 opener 缺少与临时面板对应的 `aria-expanded` / `aria-controls`。每个 fresh context 均从空 localStorage 挂载；取消场景只有 `snapshot=2`，选择场景只有 `snapshot=2, read_canvas=1`；write、unknown、external、console error、page error 均为 `0`。

red 图 [red-empty-opener-escape-wrong-focus.png](raw/2026-07-26-l3-syn-n2r-r3c-r1-focus-return/red-empty-opener-escape-wrong-focus.png) 可见：空态“选择画布”仍在中央，但焦点环错误落到顶部“画布”。

## 3. 最小实现

只在 `KnowledgeCanvasView` 实例内补 actual-opener 与两类回焦策略：

1. `showFilePanel(event.currentTarget)` 把本次真实 `HTMLButtonElement` 写入组件本地 ref；后一次打开覆盖前一次，关闭时立即清空，未写 module global、selector、localStorage 或第二份面板状态。
2. 取消类关闭的优先级是：仍 connected 的 actual opener → 顶部 opener → 连续 stage。
3. 选择/创建成功类关闭的优先级是：仍 connected 的连续 stage → 顶部 opener；失败时沿用既有面板与错误状态，不伪装成功。
4. 顶部按钮在面板已打开时再次点击，显式以这次点击的顶部按钮为回焦目标，不误用旧 opener。
5. 顶部和空态两个真实 opener 都保留 `aria-expanded` 与 `aria-controls="native-canvas-file-panel"`；没有改视觉文案、class、CSS、布局或色彩。
6. 增加 4 个纯策略合同：connected actual opener 优先、actual opener 卸载时回顶部、actual 与顶部都卸载时回 stage、选择成功优先回 stage。

## 4. Green synthetic 浏览器矩阵

最终运行 [browser-evidence.mjs](raw/2026-07-26-l3-syn-n2r-r3c-r1-focus-return/browser-evidence.mjs)，原始结果见 [browser-evidence.json](raw/2026-07-26-l3-syn-n2r-r3c-r1-focus-return/browser-evidence.json)。

结果为 **`PASS`，5 个 fresh context / 73 项断言 / 0 失败**：

| fresh context | 关闭/提交路径 | DOM identity 与焦点结果 |
| --- | --- | --- |
| `1180×760` 顶部 opener 取消 | Escape、显式关闭、再次点击顶部按钮 | 三次均 `sameNodeAsActiveElement=true`，回到各自实际点击的 connected 顶部 DOM 节点 |
| `1180×760` 空态 opener 取消 | Escape、显式关闭 | 两次均 `sameNodeAsActiveElement=true`，回到各自实际点击的 connected 空态 DOM 节点 |
| `1180×760` 顶部 opener 选择 | 选择既有 Canvas | 面板卸载；activeElement 为 connected 连续 stage，不是 body 或 opener |
| `1180×760` 空态 opener 选择 | 选择既有 Canvas | 空态 opener 已 disconnected；activeElement 为 connected 连续 stage，不是 body 或顶部 fallback |
| `900×760` reduced motion | 空态 opener → Escape | `matchMedia(reduce)=true`；仍回同一 connected 空态 DOM 节点 |

三张最终图：

- [01-1180-empty-opener-panel-open.png](raw/2026-07-26-l3-syn-n2r-r3c-r1-focus-return/01-1180-empty-opener-panel-open.png)：空态 opener 打开临时文件面板。
- [02-1180-empty-opener-escape-focus-return.png](raw/2026-07-26-l3-syn-n2r-r3c-r1-focus-return/02-1180-empty-opener-escape-focus-return.png)：Escape 后焦点环回到同一“选择画布”按钮。
- [03-1180-empty-opener-selection-stage-focus.png](raw/2026-07-26-l3-syn-n2r-r3c-r1-focus-return/03-1180-empty-opener-selection-stage-focus.png)：选择成功后文件面板消失，Canvas 连续舞台接续；activeElement 身份以 raw JSON 为准，runner 没有额外调用 `.focus()`。

三图逐张人工检查，没有新增层级、颜色、尺寸、文案或装饰变化，结论为 **`VISUAL_UNCHANGED / FOCUS_PATH_FIXED`**。

## 5. ARIA、Tab/AT、布局与 reduced motion

- 每次面板打开后，activeElement 都进入面板首个有效 Canvas 文件按钮。
- actual opener 在打开态为 `aria-expanded=true`，`aria-controls` 精确指向唯一 live `#native-canvas-file-panel`。
- 打开态面板为 Canvas 根内唯一 absolute surface，含 `5` 个可交互子项；`1180` 下实测 `320×231`，`900` 下实测 `420×231`。
- 关闭或选择成功后，面板数量与面板内可交互项均为 `0`，`aria-expanded=false`，controls target 不再存在，因此已离开 DOM、Tab 与 AT 路径。
- 所有最终 activeElement 都 connected 且不是 body。
- `1180×760`：Canvas 根 `656×694`，stage `656×654`，stage/root 高度比 `0.9423631123919308`。
- `900×760`：Canvas 根 `436×694`，stage `436×619`，stage/root 高度比 `0.8919308357348703`。
- 两个 viewport 的 document、body、shell、中央 surface、活动组面板和 Canvas root 均无横向 overflow；Canvas chrome、连续 stage、浮动工具与单组中央结构保持不变。
- 本包没有新增动画；`prefers-reduced-motion: reduce` context 的实际回焦与 overflow 合同通过。

## 6. Fresh context、localStorage 与 mock 边界

每个 context 在 mount 前均为 `localStorageEmptyBeforeMount=true`。最终只出现一个既有 UI 偏好 key：

```text
syn-native-knowledge-workspace-ui-v1
```

每个 context 记录 `writeCount=2`，最终归一化内容只含 `version=2`、一个 Canvas surface 中央组、`activeGroupId` 与 `splitRatio=50`；没有 opener、Canvas 内容或第二份面板状态。

精确 read allowlist 保持：

```text
knowledge_workspace_graph
knowledge_workspace_read_canvas
knowledge_workspace_read_markdown
knowledge_workspace_search
knowledge_workspace_snapshot
```

| 场景 | 实际 command 次数 | write | unknown | external | console error | page error |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| 顶部取消三路径 | `snapshot=2` | 0 | 0 | 0 | 0 | 0 |
| 空态取消两路径 | `snapshot=2` | 0 | 0 | 0 | 0 | 0 |
| 顶部选择成功 | `snapshot=2, read_canvas=1` | 0 | 0 | 0 | 0 | 0 |
| 空态选择成功 | `snapshot=2, read_canvas=1` | 0 | 0 | 0 | 0 | 0 |
| `900` reduced-motion 空态 Escape | `snapshot=2` | 0 | 0 | 0 | 0 | 0 |

## 7. 必跑门禁

| 验证 | 实际结果 |
| --- | --- |
| 聚焦 `knowledge-canvas` 合同 | PASS：既有 R3C `14 / 0`；新增 R3C-R1 `4 / 0`。 |
| 聚焦 `knowledge-workbench-shell` 合同 | PASS：`knowledge workbench shell static convergence contract passed; R3B 16 / 0`。 |
| 聚焦 `native-knowledge-workspace` 合同 | PASS：N0 optional compatibility 与 N1/N2/N3/N4/N5 fixed client contracts。 |
| 聚焦 `knowledge-graph` 合同 | PASS：native knowledge graph static shell and typed handoff。 |
| `npm run typecheck` | PASS，exit `0`。 |
| `npm run test:offline-interaction` | PASS，exit `0`；冻结 runner 静态计数并实际执行 **37** 个 entry。首个测试打印的 `offline interaction tests passed: 15` 不是 runner entry 数。 |
| green browser evidence | PASS：`73 / 73`，5 个 fresh context，3 张最终图。 |
| shape baseline | PASS：`Errors 17 / Warnings 5 / Info 5`。 |
| shape check | 预期 exit `1`：仍为既有 `17 / 5 / 5`，与 baseline 类别和 finding 相同，零新增类别/finding。 |
| hardcoded-hex selftest | PASS：`13 / 13`。 |
| machine-face selftest | PASS：`18 / 18`。 |
| `git diff --check` | PASS，无输出。 |
| `git diff --cached --name-only` | PASS，无输出，staged 为空。 |
| 冻结 hash | PASS：12 个冻结只读文件全部 MATCH。 |
| 进程收尾 | 只向本包启动的 Vite session 发送 `Ctrl-C`；浏览器由 runner 自行关闭；最终 5173 无 listener。 |

Playwright skill 的 `npx` 前置条件存在；本地包装器 `--help` 在 10 秒内未返回并被终止，没有安装依赖或生成 evidence。本包随后使用仓库现有、R3C 同类 synthetic 取证所用的缓存 `playwright-core` 启动 headless Chromium。首次受限沙箱内启动因 `SIGABRT/EPERM` 未产出 evidence；按浏览器执行边界在提升权限后完成 red/green。该事实没有改变产品结果、数据边界或写入范围。

## 8. 实际写入与产物 hash

实际写入：

1. `prototypes/productized-desktop-shell/src/views/knowledge/KnowledgeCanvasView.tsx`。
2. `prototypes/productized-desktop-shell/tests/knowledge-canvas.test.tsx`。
3. `evidence/raw/2026-07-26-l3-syn-n2r-r3c-r1-focus-return/red-browser-evidence.mjs`。
4. `evidence/raw/2026-07-26-l3-syn-n2r-r3c-r1-focus-return/red-browser-evidence.json`。
5. `evidence/raw/2026-07-26-l3-syn-n2r-r3c-r1-focus-return/red-empty-opener-escape-wrong-focus.png`。
6. `evidence/raw/2026-07-26-l3-syn-n2r-r3c-r1-focus-return/browser-evidence.mjs`。
7. `evidence/raw/2026-07-26-l3-syn-n2r-r3c-r1-focus-return/browser-evidence.json`。
8. 三张最终 green PNG。
9. 本 evidence。
10. 本任务包：只回写实际状态、实际写入和执行结果。

未修改 `styles.css`、fixture、正式 runner、R3B 状态/组壳、Graph、Native workspace、layout/workspace helper、Rust/Cargo、typed client、依赖、CURRENT、AUTHORITY、计划、决策、真实 store/vault 或 `.codex`。未 stage、commit、push、reset、clean、stash 或 checkout。

raw 关键产物 SHA-256：

| 产物 | SHA-256 |
| --- | --- |
| red runner | `5f38dbcc3bb84498fdd0c37202188125f041f6492dcfc2dfbd6d739fbb1a20c9` |
| red JSON | `5f4e1670f93ca8714009ea63aa57d8c62d86a2f4e763b3ec77d086702e886571` |
| red PNG | `201f46103730a37f85439accb67d8ecebedc4275e12f861ffe17b238ced52d2f` |
| green runner | `1dead02d262c6ebd682e6d491e264647fd9ec77cd2caef2450e3bf923dc77f5f` |
| green JSON | `4b835bba1449946d1f557598bd6e294279882ef59b65d9fe9b186531c59b36bf` |
| green panel-open PNG | `bf486aafbab760f1a50c231216fe25252b5537e23c9ddc15b5c77ece05a87523` |
| green Escape PNG | `36e752a79290f65a7c3cc2354d8c2242f673871a67d879cddcf9af1b9546a4ec` |
| green selection PNG | `672ba7daa16fe2fd78cc895409173b4769bd862d332d2d251287060c87b85180` |

## 9. 新 catch

**零新 catch。** red 命中的四个失败就是任务包已知缺口；浏览器受限沙箱启动失败是未产出证据的执行环境边界，提升到已授权的本地 headless Chromium 后直接通过，没有引出产品修补、范围扩张或安全边界变化，因此不写 `docs/harness-catch-log.md`。

## 10. 遗留与停止边界

- 本包只形成执行线候选：`PASS_N2R_R3C_R1_FOCUS_RETURN / NEEDS_GUIDANCE_REVIEW / NOT_REAL_APP_ACCEPTED`。
- 尚待指导线独立复核两个产品/测试 hunk、raw activeElement 矩阵和三张最终图；执行线不自行接受本包或上游 R3C。
- shape `17 / 5 / 5` 是冻结的既有全仓债务，本包零新增，不在本包修复。
- 未开始 R3D；未做真实 Syn/Tauri App、真实 store/vault、完整 R0、上游 R3C 或发布验收。

## 11. 指导线独立复核结论

指导线已独立核对两个产品/合同 hunk、raw activeElement/DOM identity 矩阵、三张最终图和规定门禁。actual opener 由组件本地 ref 记录；取消回仍 connected 的本次 opener，选择成功回连续 Canvas stage；普通路径没有被 fallback 偷换成固定顶部按钮。执行线的 red `6 / 4 failed` 与 green `5 contexts / 73 / 0` 口径相互一致。

Canvas `14 / 0 + 4 / 0`、typecheck、37-entry runner、shape baseline/check 同为既有 `17 / 5 / 5`、两项 selftest、staged 空和 5173 释放证据均成立。指导复核零新 catch。

因此接受 R3C-R1，并解除上游 R3C 因该单一回焦缺口形成的拒绝状态。精确结论为：

`ACCEPTED_N2R_R3C_R1_FOCUS_RETURN / ACCEPTED_N2R_R3C_CANVAS_FIRST / NOT_REAL_APP_ACCEPTED`

本结论只接受 pure-synthetic Canvas-first 范围；完整 R0、真实 Syn/Tauri App、真实 store/vault、全仓 shape 全绿和发布仍未通过。
