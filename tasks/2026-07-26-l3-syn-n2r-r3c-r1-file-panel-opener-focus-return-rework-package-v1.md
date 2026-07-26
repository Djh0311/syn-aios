# 任务包：L3 Syn N2R-R3C-R1 文件面板实际触发器回焦窄返工 v1

- 日期：2026-07-26
- 状态：**ACCEPTED_N2R_R3C_R1_FOCUS_RETURN / ACCEPTED_N2R_R3C_CANVAS_FIRST / NOT_REAL_APP_ACCEPTED**
- 负责人：独立知识前端执行线
- 指导/验收：当前总指导对话
- 上游任务：`tasks/2026-07-26-l3-syn-n2r-r3c-canvas-first-convergence-package-v1.md`
- 上游 evidence：`evidence/2026-07-26-l3-syn-n2r-r3c-canvas-first-convergence-verification-v1.md`
- 目标 evidence：`evidence/2026-07-26-l3-syn-n2r-r3c-r1-file-panel-opener-focus-return-verification-v1.md`

## 0. Kickoff

本包只关闭指导复核确认的一个真实缺口：

> Canvas 文件面板有顶部“画布”和空态“选择画布”两个真实入口。当前 `closeFilePanel` 固定只聚焦顶部入口，因此从空态入口打开后按 Escape，焦点跳到另一个按钮，没有回到实际触发器。

R3C 已成立的 Canvas-first 主结构、紧凑 chrome、连续 stage、文件面板/inspector 层级、浮动工具、typed client、草稿/CAS/冲突保护、隔离、量尺、十张截图和其余 `131 / 0` 证据全部冻结。本包不重做 R3C，不进入 R3D。

用户已于 2026-07-26 在指导裁决后明确回复“可以”，授权派发本窄返工。授权仅限本文白名单和 pure-synthetic 本地浏览器验证。

## 1. 交互意图

### 1.1 设计检查点

- Intent：知识工作者从哪个 Canvas 入口打开临时文件面板，取消后就回到哪个入口；选择文件成功后直接回到连续画布继续工作。
- Hierarchy：Canvas stage 继续是唯一主焦点；回焦逻辑只维持操作连续性，不制造新的可见层级。
- Palette / Depth / Surfaces：完全沿用已接受的 R3C token、边界和表面；本包禁止任何视觉修改。
- Typography / Spacing：完全冻结；不得以增加提示、按钮或布局补丁代替正确焦点管理。
- Signature：Syn 的固定 vault 与确认式写入边界不变；面板是临时工具，关闭后用户不会“迷路”。

### 1.2 产品域与拒绝项

- Domain：Canvas、文件面板、实际 opener、取消、选择成功、连续舞台、焦点返回、固定 vault。
- Color world：暖纸、墨色、茶色、青玉状态、细发丝边；本包不新增或调整任何颜色。
- Signature：同一 Canvas 根内，取消回实际入口，选择成功回连续舞台。
- Rejecting：
  1. 固定回顶部按钮 → 记录本次实际 opener；
  2. 用 `document.querySelector` 猜按钮 → 使用组件本地 ref / `event.currentTarget`；
  3. opener 已卸载仍强行 focus → 使用明确的 stage/顶部安全 fallback；
  4. 只补源码字符串断言 → 用真实浏览器 activeElement 证明。

## 2. Authority 与并发边界

- authority_chain：`AGENTS.md` → `CURRENT.md` → native knowledge route v2 → native knowledge small-stage plan v2 → R0 → R3C task/evidence → R3C 指导裁决 → 本包。
- plan_anchor：`docs/plans/2026-07-23-l3-syn-native-knowledge-workspace-small-stage-plan-v2.md#r3-既有能力交互与视觉收敛`
- existing_before_new：两个真实入口、一个文件面板、现有 `showFilePanel/closeFilePanel`、Canvas stage ref、pure-synthetic fixture 和 37-entry runner 均已存在；只补实际 opener 语义。
- capabilities_touched：none；不得修改 typed client、command、payload、vault、CAS、草稿模型或 capability。
- forbidden_alternatives：不新增焦点管理依赖、不用全局 selector/DOM 文案查找、不用延时轮询掩盖 race、不复制第二个面板、不改视觉、不启动真实 App。

本包独占 `KnowledgeCanvasView.tsx` 和 `knowledge-canvas.test.tsx` 的窄写面。其他知识 UI 包不得并发写这两个文件；`styles.css`、fixture、runner、R3B 状态和 Graph 全部冻结。

对话底座线只可做静态只读准备。本包代码写入、Vite/无头浏览器采集期间不得构建、启动或执行三句真实 App 重验，也不得占用同一真实 store 或 Rust 运行资源。

## 3. 冻结基线与开工停点

- HEAD：`e9ad7f3a204a1ebb11ce26c1e8c05b19c04c0991`
- 派发时 staged：空
- 派发时 `127.0.0.1:5173`：无 listener
- `.interface-design/system.md`：不存在；本包沿用 R3C 已冻结设计意图，不新建设计系统文件。

| 文件 | 派发 SHA-256 | 本包权限 |
| --- | --- | --- |
| `prototypes/productized-desktop-shell/src/views/knowledge/KnowledgeCanvasView.tsx` | `bbf13ecd68cfe48ef2e04eca28d0e1d2546b5c88f17c7bbe0e173a6ea0deaa25` | 窄写 |
| `prototypes/productized-desktop-shell/tests/knowledge-canvas.test.tsx` | `693ec0e86759deeae93d5d76c091ea5befd9f62dd515087e2c61dbff8d42a8e3` | 窄写 |
| `prototypes/productized-desktop-shell/tests/knowledge-workbench-visual-fixture.tsx` | `ea27e80ec9fff83fc630e4b80074915a39ee1ecf2f5217c3e3940bb51eb929ee` | 冻结只读 |
| `prototypes/productized-desktop-shell/tests/knowledge-workbench-visual-fixture.html` | `8a58ac0d02d9c33a9ca151f4ec1e653ccd3c6d79096581efd339a62c1fcec23c` | 冻结只读 |
| `prototypes/productized-desktop-shell/src/styles.css` | `8d74a3245d07477a44dd29d7fabd67a0cd01300a880bf4db6752dede105d03ed` | 冻结只读 |
| `prototypes/productized-desktop-shell/src/views/knowledge/NativeKnowledgeWorkspace.tsx` | `eafd9edccc3a2cd62e99cfa211809d77b206c826907ca07976ef4c419f4b4464` | 冻结只读 |
| `prototypes/productized-desktop-shell/src/lib/knowledgeWorkbenchLayout.ts` | `8fb9faf01ef9dae509a6cffa4e14fd62031bb9744ea8e44eb1d1476e4f9ee220` | 冻结只读 |
| `prototypes/productized-desktop-shell/src/lib/knowledgeWorkspace.ts` | `81eddc02206035a9d1bf1e3954d5ea19b1864a71b27ffea43efafa5a2fd51fb0` | 冻结只读 |
| `prototypes/productized-desktop-shell/scripts/run-offline-interaction-test.mjs` | `798daf13722875befa22eb233c476b957d241005a3b08267ed30eb54e0363082` | 冻结只读 |
| `prototypes/productized-desktop-shell/package.json` | `08a3abc466e2dd51f946380badb6519b4273353d8d279efa43b1cf6086d87e65` | 冻结只读 |
| `prototypes/productized-desktop-shell/package-lock.json` | `781cb1d94eaaeec0d071f156f9aeca65400acce8bd811c9c95cd4d45cf700bc0` | 冻结只读 |
| 上游 R3C task | `3e6096177e1d6c98c2a01002b76a8ed32c5277f393fb4388dc093ccd15b97b8f` | 冻结只读 |
| 上游 R3C evidence | `a189496276363f916237a443e2906ce63bea040404f263a258bc690eb3dd2fbc` | 冻结只读 |
| R0 设计参考 | `141ff343312fff91346e6e170f1aea2d07cda5fa2abdb3fde8b840b16f9ed4d9` | 冻结只读 |

开工前逐一复核 HEAD、staged、全部冻结 hash、5173、两个窄写文件的 provenance 与唯一写所有权。任一 hash 漂移、staged 非空、端口占用或并行 writer 存在，立即按 §9 停止；不得 reset、clean、stash、checkout、覆盖或合并他人 WIP。

## 4. 精确写入白名单

### 4.1 产品与正式合同

1. `prototypes/productized-desktop-shell/src/views/knowledge/KnowledgeCanvasView.tsx`
   - 只允许记录本次实际文件面板 opener；
   - 只允许收敛取消/提交后的确定性 focus target；
   - 可增加稳定、非视觉的测试 handle；
   - 不得改变可见文案、DOM 区域数量、布局、样式 class、typed client 或 Canvas 数据行为。
2. `prototypes/productized-desktop-shell/tests/knowledge-canvas.test.tsx`
   - 只允许增加焦点返回策略的精确回归合同；
   - 纯 helper 合同不得替代 §6 的真实浏览器 activeElement 证据；
   - 禁止用整文件快照或单纯源码字符串出现次数冒充焦点行为。

### 4.2 新 evidence

- `evidence/raw/2026-07-26-l3-syn-n2r-r3c-r1-focus-return/**`
- `evidence/2026-07-26-l3-syn-n2r-r3c-r1-file-panel-opener-focus-return-verification-v1.md`
- 本任务包：只回写实际状态、实际写入和执行结果
- `docs/harness-catch-log.md`：仅发现新的真实 catch 时追加

除此之外一律不写。特别禁止修改 `styles.css`、visual fixture、fixture HTML、正式 runner、R3B reducer/组壳、Graph、活动栏、右栏、Rust/Cargo、依赖、CURRENT、AUTHORITY、计划、决策、真实 store/vault 或 `.codex`。

## 5. 精确行为合同

### 5.1 实际 opener

- 顶部“画布”和空态“选择画布”都必须把自己的真实 `event.currentTarget` 交给同一个组件本地 opener 记录。
- opener 只能属于当前 `KnowledgeCanvasView` 实例；不得写到 module global、localStorage、R3B 偏好或跨组共享状态。
- 不通过文案、CSS selector、DOM 顺序或 `document.activeElement` 的偶然值反查 opener。
- 新一次打开覆盖旧 opener；面板关闭完成后不得保留可误用的陈旧 opener。

### 5.2 取消类关闭

下列动作不改变当前 Canvas 内容，必须在面板卸载后回到**本次实际 opener**：

1. Escape；
2. 面板内显式“关闭”按钮；
3. 顶部“画布”再次切换关闭时，焦点保持/回到该顶部按钮。

若 opener 因组件/标签组已卸载而不再 connected，不得 focus 脱离文档的节点；使用当前组件内明确 fallback，并保证焦点不落到 `body`。普通路径不得因为 fallback 存在而把空态入口重新固定回顶部。

### 5.3 内容提交类关闭

- 从文件列表成功选择并读取 Canvas 后，面板卸载，焦点进入当前 Canvas 的连续 stage；不得回到已经随空态消失的“选择画布”，也不得落到 `body`。
- 本包浏览器不得点击“新建 JSON Canvas”或“保存”；不得制造任何 synthetic/真实 create/write 调用。既有 create 成功路径不得被破坏，但不以写调用为本包验收手段。
- 读取失败或动作未成功时，不得伪装成成功关闭；沿用既有错误/面板状态，不扩大本包。

### 5.4 既有语义冻结

- 顶部触发器的 `aria-expanded/aria-controls`、面板首焦点、Escape 阻止传播、面板卸载、inspector 关联/回焦继续成立。
- R3C Canvas root、chrome、stage、浮动工具、文件面板 bounds、双组与 `900×760` overflow 不变。
- read allowlist、typed client、draft/CAS/conflict、localStorage 偏好和零真实写边界不变。
- reduced-motion 下使用同一焦点策略，不新增动画、延时或视觉提示。

## 6. Red-first 与 synthetic 浏览器验收

### 6.1 Red-first

在修改产品前，先建立并运行新的真实浏览器焦点矩阵。至少保留以下旧实现反例：

1. 顶部 opener → Escape：回顶部，当前应 PASS；
2. 空态 opener → Escape：错误回顶部，必须 RED；
3. 顶部 opener → 显式关闭：回顶部，当前应 PASS；
4. 空态 opener → 显式关闭：错误回顶部，必须 RED；
5. 顶部 opener → 选择已有 Canvas：当前固定回顶部，不满足“成功后进 stage”，必须 RED；
6. 空态 opener → 选择已有 Canvas：空态消失后仍固定回顶部，不满足“成功后进 stage”，必须 RED。

记录总断言、失败数、失败名称、每步 opener/activeElement 身份和一张空态回错按钮的 red 图。不得先改实现再补红；不得用 `force` click、手工 `.focus()` 或 runner 注入产品状态造 red/green。

### 6.2 Green fresh contexts

复用真实 React、真实生产 CSS 和冻结 fixture，至少覆盖：

1. `1180×760` 顶部 opener：Escape 与显式关闭两轮都回同一顶部按钮；
2. `1180×760` 空态 opener：Escape 与显式关闭两轮都回同一空态按钮；
3. `1180×760` 顶部 opener：选择现有 synthetic Canvas 后 activeElement 为连续 stage；
4. `1180×760` 空态 opener：选择现有 synthetic Canvas、空态卸载后 activeElement 为连续 stage；
5. `900×760` 或 reduced-motion fresh context：空态 Escape 回实际 opener，页面和 Canvas 无横向 overflow。

每次开面板都必须证明：

- 点击的 opener 与关闭后 activeElement 是同一个 DOM identity，而不只比较按钮文案；
- 首焦点进入文件面板；
- trigger `aria-expanded/controls` 与面板存在性一致；
- 关闭后面板和其可交互后代退出 DOM/Tab/辅助技术路径；
- activeElement 不为 `body`，也不是 disconnected 节点。

成功选择场景必须证明：

- 只发生允许的 read command；
- 面板卸载、stage 聚焦、Canvas 已加载；
- create/write、unknown、external request、console error、page error 均为 `0`。

每个 fresh context 还要记录 mount 前 localStorage 为空、最终只含允许的 R3B 可丢弃偏好、精确 read allowlist、document/body/shell/active group/Canvas overflow。

至少输出一个 red JSON、一个 green JSON和三张最终图：

- 空态 opener 打开面板；
- Escape 后同一空态 opener 的 focus-visible；
- 选择成功后的连续 stage；activeElement 身份以 raw JSON 为准，不得为制造可见焦点环而由 runner 额外调用 `.focus()`。

本包没有视觉改动，截图结论应写 `VISUAL_UNCHANGED / FOCUS_PATH_FIXED`；不得重新宣称完整 R0、Obsidian 1:1 或 R3C 已获指导接受。

## 7. 必跑验证

从 `prototypes/productized-desktop-shell`：

1. 聚焦 `knowledge-canvas` 合同，报告新增焦点策略断言与既有 R3C `14 / 0`；
2. 聚焦 `knowledge-workbench-shell`、`native-knowledge-workspace` 和 `knowledge-graph` 合同；
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

## 8. 结论枚举

- `PASS_N2R_R3C_R1_FOCUS_RETURN / NEEDS_GUIDANCE_REVIEW / NOT_REAL_APP_ACCEPTED`
- `NEEDS_N2R_R3C_R1_REWORK / NOT_ACCEPTED`
- `BLOCKED_N2R_R3C_R1_BASELINE_DRIFT`
- `BLOCKED_N2R_R3C_R1_WRITE_OWNERSHIP_CONFLICT`
- `BLOCKED_N2R_R3C_R1_BROWSER_OR_PORT`
- `BLOCKED_N2R_R3C_R1_SCOPE_EXPANSION`
- `BLOCKED_N2R_R3C_R1_TYPED_CLIENT_OR_WRITE_BOUNDARY`

即使执行线 PASS，也只能回交 `NEEDS_GUIDANCE_REVIEW / NOT_REAL_APP_ACCEPTED`，不得自行接受 R3C、继续 R3D 或启动真实 App。

## 9. 立即停止条件

出现任一情况立即停止并按对应枚举回交：

- 必须修改 CSS、fixture、runner、R3B 状态、Graph、Rust、typed client 或依赖才能完成；
- 必须点击 create/save、放宽 mock write 计数或接触真实 store/vault；
- 需要全局 selector、文案查询、module global、localStorage 或第二份面板状态才能记住 opener；
- 无法在 actual DOM identity 层证明两个入口分别回焦；
- 需要 `force` click、runner 手工 focus、截图后处理或隐藏按钮才能造绿；
- shape 新增 finding/类别，或出现无法在两个白名单产品/测试文件解释的回归；
- 任一冻结 hash 漂移、staged 非空、5173 被其他进程占用或出现并行 writer。

禁止 stage、commit、push、reset、clean、stash、checkout、删除或覆盖上游 evidence。不得终止或修改非本包进程。

## 10. 必须回传

1. HEAD、staged、冻结 hash、两个写文件 provenance、端口和唯一写所有权；
2. red-first 总断言、失败数、六条矩阵的逐项结果和 red 图；
3. opener 记录与 cancel/selection focus policy 的最小实现；
4. 顶部与空态入口的 Escape/显式关闭 DOM identity 证据；
5. 两入口选择成功后 stage 聚焦证据；
6. ARIA、面板卸载、Tab/AT 路径、reduced-motion 和 overflow；
7. localStorage、read allowlist、各 command 次数，以及 write/unknown/external/console/page error 五类零值；
8. 聚焦合同、typecheck、37-entry runner、shape、自测、diff/staged/hash/端口；
9. 实际写入文件与未完成项；
10. 新 catch；没有则明确写“零新 catch”。

执行线不得自行验收。指导线收到回交后会独立核两个产品/测试 hunk、真实浏览器 activeElement 矩阵和三张图，再决定是否接受 R3C-R1 与上游 R3C。

## 11. 实际执行回填

- 结果：`PASS_N2R_R3C_R1_FOCUS_RETURN / NEEDS_GUIDANCE_REVIEW / NOT_REAL_APP_ACCEPTED`；执行线未自行验收，未进入 R3D。
- 最小实现：组件本地 ref 记录本次实际 `event.currentTarget`；取消类关闭按 actual opener → 顶部 opener → stage 回焦，选择成功按 stage → 顶部 opener 回焦；顶部 toggle-close 显式使用这次点击的顶部按钮；两个 opener 的 `aria-expanded` / `aria-controls` 对齐同一临时面板。
- red-first：`EXPECTED_RED`，`6` 项断言 / `4` 项失败；空态 Escape、空态显式关闭、顶部选择成功、空态选择成功精确失败，red 图已保存。
- green 浏览器：`5` 个 fresh context / `73` 项断言 / `0` 失败；顶部与空态的 Escape/显式关闭均以 actual DOM identity 回焦，两入口选择成功均聚焦 connected 连续 stage；reduced-motion、ARIA、面板卸载、Tab/AT、overflow 均通过。
- 数据边界：每个 context mount 前 localStorage 为空，最终只写既有 `syn-native-knowledge-workspace-ui-v1` UI chrome 偏好；read allowlist 精确保持五项；write/unknown/external/console/page error 均为 `0`。
- 门禁：Canvas `R3C 14 / 0`、新增 R3C-R1 `4 / 0`；Workbench shell `R3B 16 / 0`；Native workspace、Graph、typecheck、37-entry offline runner 全过；shape baseline `pass 17 / 5 / 5`，check 保持既有 `fail 17 / 5 / 5` 且零新增；两组 selftest `13 / 13`、`18 / 18`；diff check 通过，staged 为空，12 个冻结只读 hash 全匹配。
- 收尾：只停止本包 Vite；最终 5173 无 listener；两个窄写文件无打开句柄。零新 catch，未改 catch log。
- 实际产品/合同写入：`KnowledgeCanvasView.tsx`、`knowledge-canvas.test.tsx`。另新增本包 red/green raw runner、JSON、1 张 red 图、3 张 green 图、本 evidence，并只回写本段实际结果。
- 证据：`evidence/2026-07-26-l3-syn-n2r-r3c-r1-file-panel-opener-focus-return-verification-v1.md`。
- 未完成：指导线独立复核、真实 Syn/Tauri App、真实 store/vault、完整 R0、上游 R3C 接受与发布验收；这些均不在本执行线结论内。

## 12. 指导线独立复核

- 指导结论：`ACCEPTED_N2R_R3C_R1_FOCUS_RETURN / ACCEPTED_N2R_R3C_CANVAS_FIRST / NOT_REAL_APP_ACCEPTED`。
- 独立核对 actual opener 的组件本地 ref / `event.currentTarget`、取消与选择成功的两类回焦、两个真实入口的 ARIA，以及 opener 清理和 fallback；未发现用全局 selector、localStorage 或延时造绿。
- 独立核对 red `6 / 4 failed`、green `5 contexts / 73 / 0`、原始 activeElement/DOM identity 和三张图；并复核 Canvas `14 / 0 + 4 / 0`、typecheck、37-entry runner、shape/selftest、staged 空与 5173 已释放。
- 上游 R3C 唯一已知回焦缺口已被关闭，因此 Canvas-first synthetic 范围一并接受；该结论不外推到完整 R0、真实 Syn/Tauri App、真实 store/vault 或发布。
- 指导复核零新 catch；R3C 与 R3C-R1 的产品写授权均已消费，不再自动续写。
