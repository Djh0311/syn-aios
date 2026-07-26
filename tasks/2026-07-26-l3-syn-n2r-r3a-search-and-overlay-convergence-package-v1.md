# 任务包：L3 Syn N2R-R3A Search 与 Overlay 收敛 v1

- 日期：2026-07-26
- 状态：**NEEDS_N2R_R3A_REWORK / NOT_ACCEPTED**
- 负责人：独立知识前端执行线
- 指导/验收：当前总指导对话
- 小阶段计划：`docs/plans/2026-07-23-l3-syn-native-knowledge-workspace-small-stage-plan-v2.md`
- 冻结参考：`docs/design/2026-07-25-l3-obsidian-core-desktop-reference-and-migration-r0-v1.md`
- 上游基线：`tasks/2026-07-26-l3-syn-n2r-r2-isolated-browser-visual-baseline-package-v1.md`
- 目标 evidence：`evidence/2026-07-26-l3-syn-n2r-r3a-search-and-overlay-convergence-verification-v1.md`

## 0. Kickoff

- 任务：关闭 R2 的第一组 P1——`900×760` 壳层横向溢出、左栏 Search 结果态、command/quick-open 的结果列表、当前选择、键盘路径和紧凑视觉层级。
- 交付物：
  1. 所有 `900×760` 左右栏折叠组合均无 document/body/shell 横向溢出；
  2. Search 在左侧栏原位输入、加载并显示结果，不再只提供“打开快速搜索”按钮；
  3. quick-open 成为可搜索、可选择、可用键盘打开笔记的临时 overlay；
  4. command 成为可搜索、可选择、可执行的命令列表，并保留既有安全新建能力；
  5. 红绿合同、synthetic-only 浏览器实图/量尺、完整离线回归和 evidence。
- 完成标准：§6、§7 全部满足，并以 §8 的允许枚举回交。执行线不得自行验收。

本包由用户在 2026-07-26 明确批准并由指导线派发。授权仅限本任务包白名单与 synthetic-only 浏览器验证。

## Authority and plan alignment

- authority_chain: `AGENTS.md` -> `CURRENT.md` -> knowledge/conversation parallel decision -> native knowledge route decision -> native knowledge small-stage plan -> frozen Obsidian reference -> accepted R2 baseline -> this R3A task
- plan_anchor: `docs/plans/2026-07-23-l3-syn-native-knowledge-workspace-small-stage-plan-v2.md#r3-既有能力-交互与视觉收敛`
- existing_before_new: 现有 `knowledge_workspace_search`、标签打开、overlay/reducer、synthetic fixture、Syn token 和原生 button/input 已经足够；不新增后端、依赖或第二套搜索状态源。
- capabilities_touched: none — 当前 partial Code Map 尚未登记知识子壳/Search/overlay，本包不修改已登记 capability 的 canonical/entrypoint。
- forbidden_alternatives: 不用隐藏/裁切伪造无溢出，不把 Search 结果继续塞进 overlay 冒充左栏结果，不做静态假命令列表，不嵌入 Obsidian，不启动真实 Syn/Tauri/Codex/MCP/store/vault。

## 2. 设计意图与 R0 对照

### 2.1 使用者、任务与感受

- 使用者：在 macOS 上长时间检索、浏览和编辑知识的单人桌面用户。
- 核心动作：输入少量字符，用键盘在结果/命令间移动并打开目标，不离开当前工作台上下文。
- 感受：紧凑、安静、连续、桌面工具化；中央工作区仍是主焦点。

### 2.2 设计检查点

```text
Intent:     检索和命令都在当前工作台内快速完成，不制造第二页面。
Hierarchy:  左栏 Search 的输入和结果是辅助层；overlay 中输入框与当前选中行是临时焦点；中央工作区始终保持主层级。
Palette:    只使用现有 Syn 纸张/墨色/青绿语义 token，不新增硬编码颜色。
Depth:      继续使用低对比边界和轻微表面层级，不新增阴影系统、渐变或玻璃效果。
Surfaces:   左栏沿用 panel-soft；overlay 沿用 raised surface；输入保持 inset 感，选中行只用单一 accent。
Typography: 沿用现有 Syn 字体；控件保持桌面工具字号，不使用营销式大标题或 eyebrow 堆层级。
Spacing:    以 4px 为基本单位，主要使用 4/8/12/16px；结果列表密度优先，不做大卡片。
```

- Domain：vault、搜索索引、笔记路径、标签、命令、当前选择、键盘导航、上下文恢复。
- Color world：纸张、石墨、墨色、旧木、青玉状态；全部映射到既有 token。
- Signature：Search、命令与快速打开只切换单壳内的左栏/临时层；安全边界和确认式动作仍由 Syn 自己表达。
- Rejecting：
  - 左栏只有跳转按钮 → 左栏原位搜索结果；
  - “新建表单”冒充命令面板 → 可过滤命令列表，选择后再进入对应安全动作；
  - 无选择状态的结果清单 → 明确当前行、键盘提示与可访问语义。

主参考：

- R0 `03-search-reading-graph-backlinks-984x768.jpg`；
- R0 `05-command-palette-overlay-984x768.jpg`；
- R0 `06-quick-switcher-overlay-984x768.jpg`；
- R2 `10-1180-search-results-preview.png` 只作旧实现差距证据，不作为目标视觉。

本包不得声称整个 Syn 已达到 Obsidian 1:1；活动栏图标、标签组/分栏、Canvas、Graph 和右栏层级属于后续包。

## 3. 开工冻结与并发

### 3.1 冻结基线

- HEAD：`e9ad7f3a204a1ebb11ce26c1e8c05b19c04c0991`
- staged：开工时必须为空。

相对路径均以 `prototypes/productized-desktop-shell/` 为根：

| 文件 | 指导冻结 SHA-256 |
| --- | --- |
| `src/views/knowledge/KnowledgeWorkbenchShell.tsx` | `2ab66a5b00941e3cf0a083e3ed3d24b309f8b7fe5037dd41d8743cf9d2d41c21` |
| `src/views/knowledge/NativeKnowledgeWorkspace.tsx` | `456153d284712beadd46053b4871417ef33fa8bcacbf4a86a117b641774b3e0a` |
| `src/lib/knowledgeWorkbenchLayout.ts` | `cc3a9f8811edb3b6ac231a37738a3b6f0bfba0794aa66f81d4b3a14015fd083a` |
| `src/styles.css` | `fe8bfc74e8d946255fd2850d7ec9218928c0c10995ae6d21450eb7a262bdea71` |
| `tests/knowledge-workbench-shell.test.tsx` | `60b7f8d0e8ed618e0bc51ab857542e55d8c5882dc39d01270a127ca5567e5f2d` |
| `tests/native-knowledge-workspace.test.tsx` | `fdcdfe54fb8f6dd022d4b018635d3bca1f2cd75874f2a835859dacfaa406408e` |
| `tests/knowledge-workbench-visual-fixture.tsx` | `1e590ac9d040d4e9486b8afe8171526f0c9dbecb38a419bfb0d1d9ad07523b30` |
| `tests/knowledge-workbench-visual-fixture.html` | `8a58ac0d02d9c33a9ca151f4ec1e653ccd3c6d79096581efd339a62c1fcec23c` |
| `vite.config.ts` | `68f0c2b5a2fa45af381a43eaaff7a7054b83822d358fac186d3bd3ea0824b506` |
| `scripts/run-offline-interaction-test.mjs` | `798daf13722875befa22eb233c476b957d241005a3b08267ed30eb54e0363082` |

开工前必须：

1. 复核 HEAD、staged 与上述 hash；
2. 记录白名单逐文件 provenance；
3. 确认没有其他线程写同一白名单；
4. 确认本包浏览器端口空闲；
5. 不 reset、clean、stash、checkout、覆盖或批量格式化用户 WIP。

hash 漂移或写所有权冲突时立即停止，不自行合并或重做他人改动。

### 3.2 串行边界

- 本包独占 `src/styles.css`、`NativeKnowledgeWorkspace.tsx` 和知识视觉 fixture 写面。
- 对话底座线可做只读准备，但不得与本包并发构建/启动真实 App、三句重验或写共享运行资源。
- 标签组/分栏、Canvas、Graph、活动栏和右栏后续包不得与本包并发施工。

## 4. 精确写入白名单

### 4.1 产品与测试

- `prototypes/productized-desktop-shell/src/views/knowledge/KnowledgeWorkbenchShell.tsx`
- `prototypes/productized-desktop-shell/src/views/knowledge/NativeKnowledgeWorkspace.tsx`
- `prototypes/productized-desktop-shell/src/lib/knowledgeWorkbenchLayout.ts`
- `prototypes/productized-desktop-shell/src/styles.css`
- `prototypes/productized-desktop-shell/tests/knowledge-workbench-shell.test.tsx`
- `prototypes/productized-desktop-shell/tests/native-knowledge-workspace.test.tsx`
- `prototypes/productized-desktop-shell/tests/knowledge-workbench-visual-fixture.tsx`：仅为 synthetic 场景和测量需要
- `prototypes/productized-desktop-shell/tests/knowledge-workbench-visual-fixture.html`：原则上不改；确需改时必须说明原因
- `prototypes/productized-desktop-shell/scripts/run-offline-interaction-test.mjs`：只有新增测试入口时允许机械追加；优先复用现有测试

### 4.2 证据与控制文档

- `evidence/2026-07-26-l3-syn-n2r-r3a-search-and-overlay-convergence-verification-v1.md`
- `evidence/raw/2026-07-26-l3-syn-n2r-r3a-search-overlay/**`
- 本任务包：只回写实际状态、实际写入和执行结果
- `docs/harness-catch-log.md`：仅发现新的真实 catch 时追加

除此之外一律不写。不得修改 `KnowledgeBaseView.tsx`、Graph/Canvas/Maintenance 源码、`App.tsx`、任何 Rust/Cargo、依赖、CURRENT、AUTHORITY、计划、决策、真实 store/vault 或 `.codex`。

## 5. 实现合同

### 5.1 `900×760` 无横向溢出

- 在左/右栏展开与折叠的四种组合中，document、body 和 `.syn-knowledge-shell` 均满足 `scrollWidth <= clientWidth`。
- 修复必须来自正确的盒模型、轨道和 min-width 收敛，不能通过裁掉可操作内容、强制双侧栏归零、缩放页面或隐藏 overflow 伪造。
- 既有 collapsed aside 的 `aria-hidden`、`inert`、不渲染子控件及活动栏恢复路径继续成立。

### 5.2 左栏 Search

- 选择“搜索”后，`.syn-knowledge-search-panel` 原位提供唯一搜索输入、执行动作、loading/empty/error/results 状态；不再只有“打开快速搜索”按钮。
- 输入 Enter 或点击执行都调用既有 `knowledge_workspace_search`；查询 trim、已有输入验证和 fail-closed 边界不得放宽。
- 结果留在左栏，同一时间只存在一个明确当前项；ArrowUp/ArrowDown 移动当前项，Enter 打开到中央工作区。
- 结果行至少显示标题与相对路径；snippet 作为低层级信息，不做卡片阵列。
- 点击/键盘打开结果不得新增第二编辑器、第二结果容器或外部打开路径。

### 5.3 Quick Open

- quick-open overlay 使用紧凑输入 + 结果列表 + 当前选择 + 键盘提示结构；输入是主要焦点，不保留大标题/说明占据主层级。
- 结果来自既有 search command；输入变化后的实际查询策略必须明确，禁止静态假结果。可采用显式 Enter 搜索，若采用 debounce 必须可取消且不引入依赖。
- ArrowUp/ArrowDown 移动当前项，Enter 打开；当前项有可见选中样式与可访问状态。
- Escape 关闭并返回原触发按钮；成功打开结果后焦点进入已打开的中央标签或工作区，不得丢失到 body。

### 5.4 Command

- command overlay 首屏是可搜索命令列表，不再直接以“新建 Markdown/目录”表单冒充命令面板。
- 至少把现有“新建 Markdown”“新建目录”登记为真实可执行命令；允许纳入已有本地视图切换/侧栏动作，但不得新增后端命令或虚假不可执行项。
- 选择“新建 Markdown/目录”后再进入现有受限相对路径输入与创建动作；不得删除或放宽既有固定 vault、安全路径、busy/disabled 与错误处理。
- 支持输入过滤、ArrowUp/ArrowDown、Enter、Escape、当前选择和空结果；可见提示至少说明 `↑↓`、`Enter`、`Esc`。

### 5.5 快捷键与可访问性

- 在组件存活期间支持 R0 已冻结的 `⌘O` quick-open、`⌘P` command、`⌘⇧F` 左栏 Search；卸载时移除监听，不重复注册。
- overlay 输入使用可验证的 combobox/listbox 或等价可访问语义，关联当前项；不使用 `div onClick`。
- 鼠标 hover、键盘当前项、focus-visible、disabled、loading、empty 和 error 不能只靠颜色区分。
- reduced-motion 下状态不依赖动画理解；本包不新增非必要动画。
- overlay/左栏不得产生重复 DOM id。

## 6. 红合同

实现前先让新增合同失败并记录实数。至少覆盖：

1. 旧壳在 `900×760` 右栏折叠场景仍会被真实浏览器量尺抓到横向溢出；
2. 左栏 Search 必须包含原位输入、结果列表和状态，不允许只有打开 overlay 的按钮；
3. quick-open 与 command 都有输入、listbox/option 或等价关联语义、当前选择和键盘提示；
4. ArrowUp/ArrowDown 的边界行为、Enter 激活和 Escape 关闭由纯函数或可重复交互合同覆盖；
5. command 过滤后仍能选择并进入既有两个安全新建动作；
6. `⌘O`、`⌘P`、`⌘⇧F` 路由到正确状态，且不新增第二壳或第二搜索源；
7. Escape 回焦与成功打开后的中央焦点路径区分明确；
8. 旧单壳、折叠可访问性、来源/候选、Graph/Canvas/附件/冲突合同继续通过。

不得通过删除旧断言、降低选择器精度、快照整段覆盖或仅检查源码字符串制造绿色。

## 7. 浏览器验收与必跑验证

### 7.1 synthetic-only 浏览器实图

复用 R2 的真实 React + 真实生产 CSS + pure-synthetic fixture，使用 fresh context，至少采集：

1. `1180×760` 左栏 Search 输入“合成”并显示结果；
2. `1180×760` quick-open 有结果、当前选择和键盘提示；
3. `900×760` command 有过滤结果、当前选择和键盘提示；
4. `900×760` 左右栏四种折叠组合的 document/body/shell overflow 量尺。

每个 context 均记录：

- mount 前 localStorage 为空；
- 精确 read mock allowlist；
- 外部请求、写调用、未识别调用、console error、page error 均为 `0`；
- 视口、截图、选中项、焦点和 overflow；
- 与 R0 03/05/06 的逐图 `PASS / GAP`，不能只写 DOM/几何 PASS。

执行 command 场景时只验证选择和安全表单入口，不实际创建文件，保证 synthetic 浏览器写调用为 `0`。

### 7.2 必跑验证

从 `prototypes/productized-desktop-shell`：

1. 聚焦 `knowledge-workbench-shell` 合同；
2. `native-knowledge-workspace` 定向合同；
3. `npm run typecheck`；
4. `npm run test:offline-interaction`；

从仓库根：

5. `node scripts/harness/workbench-shape-gate.js --mode baseline`；
6. `node scripts/harness/workbench-shape-gate.js --mode check`，既有非零债务原样报告，只接受零净增类别/finding；
7. `node scripts/harness/workbench-shape-gate.hardcoded-hex.selftest.js`；
8. `node scripts/harness/workbench-shape-gate.machine-face.selftest.js`；
9. `git diff --check`；
10. `git diff --cached --name-only` 为空。

完成后只关闭本包启动的 Vite/浏览器，并确认所用端口无 listener。

## 8. 结论枚举

- `PASS_N2R_R3A_SEARCH_OVERLAY_CONVERGENCE / NEEDS_GUIDANCE_REVIEW / NOT_REAL_APP_ACCEPTED`
- `NEEDS_N2R_R3A_REWORK / NOT_ACCEPTED`
- `BLOCKED_N2R_R3A_BASELINE_DRIFT`
- `BLOCKED_N2R_R3A_WRITE_OWNERSHIP_CONFLICT`
- `BLOCKED_N2R_R3A_SCOPE_EXPANSION`
- `BLOCKED_N2R_R3A_BROWSER_OR_PORT`

即使本包 PASS，也不能声称：

- N2R 全部完成；
- Obsidian 1:1 或全界面高保真通过；
- 标签组/分栏、Canvas、Graph、活动栏或右栏已收敛；
- 真实 Syn/Tauri App、Home-only discovery、Gate 0 或十二项通过。

## 9. 停止与禁止范围

立即停止并回交：

- 冻结 hash 漂移、staged 非空或白名单发生并行写冲突；
- 需要改 Rust、Cargo、App、Graph/Canvas/Maintenance、依赖、schema、真实 store/vault 或安全边界；
- 需要访问用户 Chrome、启动 Syn/Tauri/Obsidian、Codex CLI/MCP 或真实对话三句；
- 只能靠隐藏/裁切、静态假结果或删除安全新建能力才能达到视觉目标；
- shape 新增 finding/类别，或旧回归出现无法在白名单内解释的失败。

禁止 stage、commit、push、reset、clean、stash。不得结束或修改非本包进程。

## 10. 必须回传

1. HEAD、staged、冻结 hash、白名单 provenance 和端口 preflight；
2. 新增红合同失败实数、失败原因及转绿实数；
3. 实际修改文件与每个文件的职责；
4. 左栏 Search、quick-open、command 的状态源、ARIA、键盘与焦点路径；
5. 三张规定实图和四种 900px 折叠量尺；
6. R0 逐图 PASS/GAP，不得以几何代替视觉；
7. mock allowlist、search 调用、写/未识别/外部请求/console/page error 实数；
8. typecheck、聚焦测试、完整 runner、shape、自测、diff/staged 实数；
9. 未完成项和是否触发停点；
10. 新 catch；没有则明确写“零新 catch”。

执行线回交状态必须包含 `NEEDS_GUIDANCE_REVIEW / NOT_ACCEPTED`。指导线随后独立看 diff、复跑聚焦门并查看三张实图，再决定接受或窄返工。

## 11. 实际执行结果（2026-07-26）

- 开工 preflight：HEAD、staged、10 个冻结 SHA-256、白名单写所有权与空闲 5173 均通过；未触发停止枚举。
- 新增 red-first：浏览器 `20` 项断言中 `17` 项真实失败；纯函数合同另有 `2` 项实现前失败。green 浏览器证据为 `74 / 74`，使用 `6` 个 fresh synthetic context。
- 实施范围只涉及任务包允许的 `knowledgeWorkbenchLayout.ts`、`NativeKnowledgeWorkspace.tsx`、`styles.css`、`knowledge-workbench-shell.test.tsx` 和 R3A evidence/raw；没有修改 runner、fixture、Rust/App/依赖、Graph/Canvas/Maintenance、CURRENT、AUTHORITY、计划或 catch log。
- 规定三图、四种 `900×760` 组合量尺、精确 mock allowlist、零 write/unknown/external/console/page error，以及实际 search / 键盘 / 回焦 / 安全 command form 证据见：`evidence/2026-07-26-l3-syn-n2r-r3a-search-and-overlay-convergence-verification-v1.md`。
- 聚焦 shell / native 合同、typecheck、37-entry offline runner、两个 shape selftest 均通过。shape baseline 是 `17 / 5 / 5`，shape check 仍为既有 `17 / 5 / 5` 非零，不存在新增类别或 finding；收尾 diff/staged/端口状态同 evidence 最终复核。
- 零新 catch；不自称真实 App、完整 R0 或指导验收通过。交回状态保持 **`NEEDS_GUIDANCE_REVIEW / NOT_ACCEPTED`**。

## 12. 指导复核裁决（2026-07-26）

- 指导线独立核源码、生产 CSS、原始浏览器报告与三张实图；独立复跑 `npm run typecheck` 和正式 37-entry `npm run test:offline-interaction`，均 exit `0`。`git diff --check` 无输出、staged 为空，5173 无 listener。
- 接受为已成立事实：单一搜索状态、Search/quick-open/command 的结果与当前项、Arrow/Enter/Escape、快捷键、焦点回收、`900×760` 四种折叠组合无横向 overflow、synthetic fixture 零写/零外部请求，以及既有 shape `17 / 5 / 5` 未净增。
- 不接受 §11 对 overlay 视觉范围的完成判断。`src/styles.css` 中 `.syn-knowledge-overlay-backdrop` 使用不透明 `background: var(--panel-soft)`；对应 Quick Open 与 Command 实图把底层工作区完全抹掉，视觉上成为第二张空白页面。冻结 R0 05/06 中底层工作区仍可辨认、临时面板在其上浮起，因此这不是范围外的活动栏/右栏差距，而是本包自身“临时 overlay / 紧凑视觉层级”的未完成项。
- 精确裁决：**`NEEDS_N2R_R3A_REWORK / NOT_ACCEPTED`**。现有行为、隔离、量尺和回归证据保留，不要求重做状态模型。
- 候选最窄返工仅限：让 Quick Open/Command 遮罩保留可辨认但降噪的底层工作区，保持前景面板对比度、键盘与 ARIA 行为不变；重采 02/03 两图，改为诚实的 R0 `PASS/GAP`，复跑 green browser、typecheck、两个聚焦合同、完整 runner、shape/selftest 与 diff/staged/端口收尾。不得借机修改 Search、Graph、Canvas、活动栏、右栏、Rust、真实 App 或数据链路。
- 本段只形成返工边界，**没有授权执行返工**；须由用户明确同意后另行派发。
