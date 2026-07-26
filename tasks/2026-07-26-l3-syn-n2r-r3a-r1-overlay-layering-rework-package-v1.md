# 任务包：L3 Syn N2R-R3A-R1 Overlay 层叠关系窄返工 v1

- 日期：2026-07-26
- 状态：**ACCEPTED_N2R_R3A_R1_OVERLAY_LAYERING / ACCEPTED_N2R_R3A_SEARCH_OVERLAY / NOT_REAL_APP_ACCEPTED**
- 负责人：独立知识前端执行线
- 指导/验收：当前总指导对话
- 上游任务：`tasks/2026-07-26-l3-syn-n2r-r3a-search-and-overlay-convergence-package-v1.md`
- 上游 evidence：`evidence/2026-07-26-l3-syn-n2r-r3a-search-and-overlay-convergence-verification-v1.md`
- 冻结参考：`docs/design/2026-07-25-l3-obsidian-core-desktop-reference-and-migration-r0-v1.md` 的 R0 05/06
- 目标 evidence：`evidence/2026-07-26-l3-syn-n2r-r3a-r1-overlay-layering-rework-verification-v1.md`

## 0. Kickoff

只关闭指导复核确认的一项差距：

> Quick Open 与 Command 当前使用不透明全屏 backdrop，完全抹掉底层知识工作区，视觉上成为第二张空白页面；应改为底层工作区仍可辨认但被降噪、前景面板明确浮起的临时 overlay。

本包**不重做 R3A**。上游已经成立的单一搜索状态、结果列表、当前项、键盘、焦点、ARIA、`900×760` overflow、隔离零写和安全 command form 全部冻结。

用户已于 2026-07-26 在指导裁决后明确回复“可以”，授权派发本窄返工。授权仅限本文白名单和 pure-synthetic 本地浏览器验证。

## 1. 设计意图与 R0 对齐

### 1.1 工作意图

- Intent：人在知识工作台中高频快速找笔记或执行受限命令；打开 overlay 时应始终知道自己仍在当前工作区，不产生页面跳转感。
- Hierarchy：overlay 输入与当前结果是唯一焦点；底层壳仍可辨认但必须退到背景。
- Palette：只使用现有 Syn 纸面、墨色、茶色和 hairline 语义 token 或由这些 token 派生的透明色；禁止新增任意裸色。
- Depth：`工作区 base → 半透明降噪 veil → raised overlay` 三层；拒绝不透明页面替换、玻璃拟态、重 blur、重阴影或戏剧化动画。
- Surfaces：前景 overlay 保持稳定、清晰和高对比；背景只降噪，不消失、不竞争。
- Typography / Spacing：沿用 R3A 已通过的紧凑字号、行高、面板宽高和 4px 系列密度，不借本包重排列表。

### 1.2 产品域检查

- Domain：vault、笔记、快速切换、受限命令、当前项、焦点回收、工作区上下文。
- Color world：纸面、浅墨、茶色、细发丝边、低层级灰褐、抬升白面。
- Signature：Syn 茶色细边的紧凑命令面板浮在仍可辨认的原生知识工作台上。
- Rejecting：
  1. 全屏纯色第二页 → 改为透明降噪 veil；
  2. 玻璃拟态和强 blur → 保留朴素工作台质感；
  3. 重阴影、缩放或慢动画 → 以细边、轻层级和即时键盘响应表达深度。

R0 05/06 是层叠关系验收源；不要求复制 Obsidian 商标、图标、颜色值、字体或受限资产。

## 2. Authority 与边界

- authority_chain：`AGENTS.md` → `CURRENT.md` → native knowledge route v2 → native knowledge small-stage plan v2 → R0 → R3A 指导裁决 → 本包。
- 本包独占 `src/styles.css` 的窄写面；其他 UI 包不得并发写该文件。
- 对话底座线可做只读准备；本包 Vite/浏览器采集期间不得构建或启动三句真实 App 重验。
- 不启动 Syn、Tauri、Obsidian、Codex CLI/MCP server；不读取或写入真实 store/vault，不进入 I5、Home-only discovery 或 Gate 0。

## 3. 冻结基线与开工停点

- HEAD：`e9ad7f3a204a1ebb11ce26c1e8c05b19c04c0991`
- 派发时 staged：空
- 派发时 `127.0.0.1:5173`：无 listener

| 冻结文件 | SHA-256 |
| --- | --- |
| `prototypes/productized-desktop-shell/src/styles.css` | `e040da8b5e4fb18cc0f8b5df1e5a78a70094c239a370704267659fc192a06134` |
| `prototypes/productized-desktop-shell/src/views/knowledge/NativeKnowledgeWorkspace.tsx` | `f4f1c5aed802e66ae3418460f3b5ff1a9a2fe33fddd773ab748e9c2f63025fe5` |
| `prototypes/productized-desktop-shell/src/views/knowledge/KnowledgeWorkbenchShell.tsx` | `2ab66a5b00941e3cf0a083e3ed3d24b309f8b7fe5037dd41d8743cf9d2d41c21` |
| `prototypes/productized-desktop-shell/tests/knowledge-workbench-shell.test.tsx` | `30727a05000561f0d9812c385a8a29fb4d199a35abbd1ba2d33ecbe552aebad2` |
| `prototypes/productized-desktop-shell/tests/native-knowledge-workspace.test.tsx` | `fdcdfe54fb8f6dd022d4b018635d3bca1f2cd75874f2a835859dacfaa406408e` |
| `prototypes/productized-desktop-shell/tests/knowledge-workbench-visual-fixture.tsx` | `1e590ac9d040d4e9486b8afe8171526f0c9dbecb38a419bfb0d1d9ad07523b30` |
| `prototypes/productized-desktop-shell/tests/knowledge-workbench-visual-fixture.html` | `8a58ac0d02d9c33a9ca151f4ec1e653ccd3c6d79096581efd339a62c1fcec23c` |
| `prototypes/productized-desktop-shell/scripts/run-offline-interaction-test.mjs` | `798daf13722875befa22eb233c476b957d241005a3b08267ed30eb54e0363082` |
| 上游 R3A task | `a30b701d0b2121aada767638810972884dadbb82e0f956b1dd7ee10ae10ce517` |
| 上游 R3A evidence | `dd8e67c953004786fc1eac08946e7364fe3b9b5d4322cc71febc8bd21ba003a0` |
| R0 设计参考 | `141ff343312fff91346e6e170f1aea2d07cda5fa2abdb3fde8b840b16f9ed4d9` |

开工前逐一复核 HEAD、staged、全部冻结 hash、5173 和 `src/styles.css` 写所有权。任何漂移、staged 非空、端口占用或并行 writer 都立即停止，不自行合并、重跑他人进程或覆盖 WIP。

## 4. 精确写入白名单

### 4.1 产品

- `prototypes/productized-desktop-shell/src/styles.css`
  - 只允许修改 `.syn-knowledge-overlay-backdrop`；
  - 若前景与背景分离仍不足，可最小修改 `.syn-knowledge-overlay` 的既有 border/shadow，但不得改尺寸、位置、padding、字号、列表或交互状态。

### 4.2 证据与任务

- `evidence/raw/2026-07-26-l3-syn-n2r-r3a-search-overlay/r1-overlay-layering-*`
- `evidence/raw/2026-07-26-l3-syn-n2r-r3a-search-overlay/02-r1-1180-quick-open-results.png`
- `evidence/raw/2026-07-26-l3-syn-n2r-r3a-search-overlay/03-r1-900-command-filter-results.png`
- `evidence/2026-07-26-l3-syn-n2r-r3a-r1-overlay-layering-rework-verification-v1.md`
- 本任务包：只回写实际状态、写入和结果
- `docs/harness-catch-log.md`：仅发现新的真实 catch 时追加

除此之外一律不写。特别禁止修改 React/TS/测试/fixture/runner、Graph/Canvas/Maintenance、活动栏、右栏、Rust/Cargo、依赖、CURRENT、AUTHORITY、计划、决策、真实 store/vault 或 `.codex`。

## 5. 红绿合同

### 5.1 Red

先在当前未修改 CSS 上运行新建的 pure-synthetic 浏览器合同，至少证明：

1. Quick Open backdrop 的实际合成 alpha 为 `1`，不满足 `0 < alpha < 1`；
2. Command backdrop 同样失败；
3. 02/03 当前实图中 overlay 外区域为单一不透明面，底层工作区不可辨认；
4. 上游行为合同仍为绿，证明返工目标只在视觉层。

必须记录 red 断言总数、失败数和失败名称。不得通过纯源码字符串 grep 代替浏览器 computed style 和实图。

### 5.2 Green

最小 CSS 修复后：

- backdrop 使用现有 token 派生的半透明降噪色，浏览器实际合成 alpha 严格满足 `0 < alpha < 1`；
- 1180 Quick Open、900 Command 的底层活动栏/侧栏/中央工作区轮廓和内容层级仍可辨认，但不与前景输入、当前项竞争；
- 前景 overlay 保持清晰、不透明、边界可见，不使用 glassmorphism、重 blur、渐变或强阴影；
- R3A 已通过的面板尺寸、列表密度、选中态、键盘提示、focus-visible、ARIA、Escape 回焦和 Enter 行为不变；
- 页面、body、shell 无新增横向 overflow；
- reduced-motion 下无需动画理解。

浏览器合同必须同时记录 computed backdrop color/alpha、前景 surface、viewport、当前项、焦点、overflow 和零错误审计。

## 6. Synthetic 浏览器验收

使用真实 React、真实生产 CSS、现有 pure-synthetic fixture 和 fresh context，重新采集：

1. `1180×760` Quick Open：输入“合成”、真实结果、当前项、键盘提示、底层工作区仍可辨认；
2. `900×760` Command：过滤“目录”、真实当前命令、键盘提示、底层工作区仍可辨认。

每个 context：

- mount 前 localStorage 为空；
- read mock allowlist 与上游 R3A 一致；
- write、unknown、external request、console error、page error 均为 `0`；
- 复核 Escape 回焦；Command Enter 仍只进入既有安全相对路径表单且不创建；
- 与 R0 05/06 逐图写 `PASS / GAP`，不得以 alpha、DOM 或量尺替代视觉判断。

新图使用 `02-r1-*`、`03-r1-*` 文件名，不覆盖上游失败实图。

## 7. 必跑验证

从 `prototypes/productized-desktop-shell`：

1. 聚焦 `knowledge-workbench-shell` 合同；
2. 聚焦 `native-knowledge-workspace` 合同；
3. `npm run typecheck`；
4. `npm run test:offline-interaction`，明确 runner 入口数与首个测试的 `15` 输出不是同一口径。

从仓库根：

5. `node scripts/harness/workbench-shape-gate.js --mode baseline`；
6. `node scripts/harness/workbench-shape-gate.js --mode check`，既有 `17 / 5 / 5` 非零债务如实保留，只接受零新增类别/finding；
7. hardcoded-hex selftest；
8. machine-face selftest；
9. `git diff --check`；
10. `git diff --cached --name-only` 为空；
11. 关闭且只关闭本包启动的 Vite/浏览器，确认 5173 无 listener。

## 8. 结论枚举

- `PASS_N2R_R3A_R1_OVERLAY_LAYERING / NEEDS_GUIDANCE_REVIEW / NOT_REAL_APP_ACCEPTED`
- `NEEDS_N2R_R3A_R1_REWORK / NOT_ACCEPTED`
- `BLOCKED_N2R_R3A_R1_BASELINE_DRIFT`
- `BLOCKED_N2R_R3A_R1_WRITE_OWNERSHIP_CONFLICT`
- `BLOCKED_N2R_R3A_R1_BROWSER_OR_PORT`
- `BLOCKED_N2R_R3A_R1_SCOPE_EXPANSION`

即使 PASS，也不能声称 R3A、N2R、Obsidian 1:1、完整 R0 或真实 App 已获指导接受。执行线必须以 `NEEDS_GUIDANCE_REVIEW / NOT_ACCEPTED` 回交。

## 9. 停止与禁止范围

立即停止并回交：

- 需要修改 React/TS/fixture/runner 或任何非白名单产品文件；
- 需要改变搜索/命令状态、键盘、ARIA、焦点、overlay 尺寸或列表密度；
- 只能用不透明页面、隐藏底层 DOM、截图后处理、glassmorphism、重 blur/阴影或硬编码裸色才能制造 PASS；
- shape 新增 finding/类别，或旧回归出现无法在本包白名单解释的失败；
- 需要启动真实 App、读取真实数据或占用共享 Rust/运行资源。

禁止 stage、commit、push、reset、clean、stash、checkout、删除或覆盖上游 evidence。不得结束或修改非本包进程。

## 10. 必须回传

1. HEAD、staged、冻结 hash、白名单 provenance、端口和写所有权 preflight；
2. red/green 浏览器断言实数及 computed alpha；
3. `styles.css` 的精确 selector/hunk 与为何是最小改动；
4. 两张新图及 R0 05/06 的逐图 `PASS / GAP`；
5. 快捷键、Arrow/Enter/Escape、回焦、ARIA 和 overflow 未回归证据；
6. localStorage、mock allowlist、读调用与五类零值；
7. typecheck、两个聚焦合同、37-entry runner、shape、自测、diff/staged、端口结果；
8. 实际写入文件；
9. 未完成项与停止条件；
10. 新 catch；没有则明确写“零新 catch”。

执行线不得自行验收、不得继续 R3B。指导线收到回交后独立查看 CSS diff、复跑聚焦/完整门并逐张对照 R0 决定是否接受 R3A。

## 11. 实际执行结果（2026-07-26）

- 开工 preflight：HEAD、staged、11 个冻结 SHA-256、5173 与 `styles.css` 写所有权均通过；未见并行 writer。
- red 浏览器合同为 `33` 项断言、`4` 项失败，失败精确落在 Quick Open/Command 的 computed alpha 与实图背景保留；两者 alpha 均为 `1`、外区 edge ratio 均为 `0`，其余 `29` 项行为/隔离/overflow 合同为绿。
- 最终产品修改只有 `.syn-knowledge-overlay-backdrop` 的一个 value：`var(--panel-soft)` → `color-mix(in srgb, var(--panel-soft) 72%, transparent)`；没有修改前景 overlay 或任何 React/TS/fixture/runner。
- green 浏览器合同为 `33 / 33`：两种实际合成 alpha 均为 `0.7215686274509804`，外区工作区 edge ratio 分别为 `0.28396702271772706`、`0.2786039612430793`，前景 alpha 为 `1`，零 blur/gradient/filter。
- 新图 `02-r1-1180-quick-open-results.png` 与 `03-r1-900-command-filter-results.png` 在本包层叠范围内逐图对照 R0 06/05 为 PASS；完整 R0 与已知活动栏/标签组/分栏/右栏等范围外 GAP 不外推关闭。
- 两个聚焦合同、typecheck、37-entry 完整 runner、hardcoded-hex `13/13` 与 machine-face `18/18` 均通过。shape baseline/check 同为既有 `17 / 5 / 5`，零新增类别/finding；diff/staged/端口收尾见目标 evidence。
- 新增 1 个 catch：首次单声明补丁误命中较早同值 selector，被 green 合同继续以 `4/33` 失败拦下；误命中行原样恢复，改用完整 selector 上下文后转绿，已追加 catch log。
- 完整原始报告、两张 red 图、两张 green 图、mock/read/五类零值、overflow、焦点/ARIA 与收尾证据见 `evidence/2026-07-26-l3-syn-n2r-r3a-r1-overlay-layering-rework-verification-v1.md`。
- 未 stage、commit、push、reset、clean、stash 或 checkout；未启动真实 App、触碰真实 store/vault，也未继续 R3B。执行线回交保持 **`NEEDS_GUIDANCE_REVIEW / NOT_ACCEPTED`**。

## 12. 指导最终复核（2026-07-26）

- 指导线独立核目标 CSS、red/green 合同实现、原始 JSON/hash、两张新图和 R0 05/06。把目标一行在只读管道中还原为旧值后，完整 `styles.css` SHA-256 精确恢复派发冻结值 `e040da8b...a06134`；其余冻结文件 hash 逐一不变，确认本包产品改动只有目标 selector 的一个 value。
- 两张新图均通过本包层叠维度：底层工作区仍可辨认但明显退后，前景输入/当前项保持唯一视觉焦点；与 R0 的临时浮层关系一致。该判断只接受 overlay 层叠，不关闭活动栏、标签组/分栏、Canvas、Graph、右栏等后续 GAP。
- 指导线独立复跑 `npm run typecheck` 与正式 37-entry `npm run test:offline-interaction`，exit 均为 `0`；runner 内两个知识聚焦合同同时打印通过。shape baseline 为 `PASS 17 / 5 / 5`、check 为既有债务 `FAIL 17 / 5 / 5`；hardcoded-hex `13 / 13`、machine-face `18 / 18`。
- `33 / 33` green 合同直接检查目标 backdrop computed alpha、实图外区 edge retention、前景不透明度、无 gradient/blur/filter、键盘/回焦、overflow、mock allowlist 与五类零值；与人工看图相互印证。
- 执行线记录的同值 selector 误命中 catch 成立且已恢复、入账；指导最终复核 **零新 catch**。
- 精确裁决：**`ACCEPTED_N2R_R3A_R1_OVERLAY_LAYERING / ACCEPTED_N2R_R3A_SEARCH_OVERLAY / NOT_REAL_APP_ACCEPTED`**。R3B、完整 R0、真实 Syn/Tauri App、Home-only discovery、Gate 0 和发布验收均未授权、未执行、未接受。
