# L3 Syn N2R-R1 React-only 单壳收敛：离线执行证据 v1

- 执行日期：2026-07-25
- 执行线：独立知识前端执行线
- 任务包：`tasks/2026-07-25-l3-syn-n2r-r1-single-shell-convergence-package-v1.md`
- 结论：`ACCEPTED_N2R_R1_OFFLINE / NOT_REAL_APP_ACCEPTED`。在指导裁决的窄返工中，断点无条件压零与折叠侧栏留在 Tab/辅助技术路径两项缺口已用新增 4 项 red→green 合同修复；指导线随后独立核 diff 并复跑关键门，接受 React-only 单壳范围。仓库级 shape `check` 仍因现存聚合债务非零退出，且未做真实 App/视觉验收；此前 13 项结构 red-first 的历史限制继续保留。

## 1. 开工前置与工作树保护

- start HEAD：`e9ad7f3a204a1ebb11ce26c1e8c05b19c04c0991`
- staged：开工前为空；本证据生成前复核仍为空。
- 开工时已逐项核对任务包冻结 SHA-256，均一致：

| 文件 | 冻结 SHA-256 |
| --- | --- |
| `src/views/KnowledgeBaseView.tsx` | `0138a0625fb054f6c622cdff903b5bb1163e28274c9f5b4a708c827aca803e85` |
| `src/views/knowledge/NativeKnowledgeWorkspace.tsx` | `a6d078b3538cd1c8387f33e40cfa0e3f7bcf31eb168987babbe61775bc3ed58c` |
| `src/styles.css` | `454ccbf317d5fde1085e49b433cc5edf98269d77d404015423a9195647304c0f` |
| `tests/native-knowledge-workspace.test.tsx` | `fdcdfe54fb8f6dd022d4b018635d3bca1f2cd75874f2a835859dacfaa406408e` |
| `tests/knowledge-vault-notes.test.tsx` | `1acf93a236b853da2975d1607854e5bc5c45b5f370226e1e6d76654c6a4f6227` |

- 已记录原有脏工作树及白名单 provenance；未执行 `reset`、`clean`、`stash`、`checkout`、`git add`、`commit` 或 `push`。
- 白名单文件无其他写入者；进程核查未发现以本仓库为 cwd 的 Syn/Tauri/Vite/Codex/MCP 真实验收进程。发现的 Codex app server 与 Vite 均位于其他工作目录，未触碰本包边界。

## 2. 实际改动与单壳证据

| 文件 | 实际改动 |
| --- | --- |
| `prototypes/productized-desktop-shell/src/views/knowledge/KnowledgeWorkbenchShell.tsx` | 新增唯一五区桌面壳：activity、left、central、right、status；窄返工中使折叠左右栏同步 `aria-hidden`、`inert` 且不渲染其可聚焦子项。 |
| `prototypes/productized-desktop-shell/src/lib/knowledgeWorkbenchLayout.ts` | 新增四种中央模式、左/右栏折叠、左侧视图与 overlay 的纯 reducer 状态模型；实际 browser 分支由该 reducer 驱动。 |
| `prototypes/productized-desktop-shell/src/views/KnowledgeBaseView.tsx` | 移除知识页根级旧 header、统计、三栏、Vault Notes、Graph/Canvas/Maintenance 纵向拼接，改为只编排原生工作台及来源槽位；右侧来源上下文只接受与当前笔记按目录边界映射的资料。 |
| `prototypes/productized-desktop-shell/src/views/knowledge/NativeKnowledgeWorkspace.tsx` | static 与 browser 分支共同使用单壳；Graph/Canvas/维护只作为 central 当前 surface；来源、候选、属性/反链、冲突、附件、refresh 和 knowledge-open 连接保留，并补齐当前项/tab/折叠控件的 ARIA 状态及 overlay Escape/回焦。 |
| `prototypes/productized-desktop-shell/src/styles.css` | `.syn-knowledge-shell` 五区布局、内部滚动、侧栏折叠、overlay、断点规则；窄返工改为由左右折叠状态消费的轨宽变量，1180/900 断点仅收紧非零轨宽。 |
| `prototypes/productized-desktop-shell/tests/knowledge-workbench-shell.test.tsx` | 单壳合同：唯一五区、根级旧容器清零、中央槽位、折叠、overlay、纯状态 reducer、来源候选动作与 static/browser 初始分支；窄返工追加断点 CSS 与折叠 aside 交互/辅助技术路径合同。 |
| `prototypes/productized-desktop-shell/scripts/run-offline-interaction-test.mjs` | R1 初次施工仅机械登记新增聚焦测试；本次窄返工未修改 runner。 |
| `docs/harness-catch-log.md` | 追加本次完整聚合回归捕获的真实静态兼容性 catch。 |
| `tasks/2026-07-25-l3-syn-n2r-r1-single-shell-convergence-package-v1.md` | 仅更新实际执行状态和回交索引。 |
| `evidence/2026-07-25-l3-syn-n2r-r1-single-shell-convergence-offline-verification-v1.md` | 新增本执行证据。 |

未改动 Rust、`App.tsx`、launcher、真实 store/vault、MCP、共享 binding/transport、`CURRENT.md`、`AUTHORITY.md`；没有新增依赖、Tauri command、sidecar JSON 或 shape 豁免。已有的 `tests/native-knowledge-workspace.test.tsx` 与 `tests/knowledge-vault-notes.test.tsx` 脏工作树内容被保留，执行线没有改写它们。

静态新合同最终确认：`KnowledgeBaseView` 只出现 1 个 `data-knowledge-shell="syn-workbench"`，并且 activity、left、central、right、status 五个 `data-knowledge-region` 各恰好 1 个；不再有 `.pg-head`、`.knowledge-base-stats`、`.knowledge-base-grid`、常驻 Vault Notes、根级 Graph/Canvas/Maintenance。壳的中央区是 `section`，没有以嵌套 `main` 形成重复 landmark。

浏览器分支的 activity 包含文件、搜索、关系图、Canvas、Syn 命令、设置/维护和来源；左右栏独立折叠；中央只切换 Markdown、预览、Graph、Canvas（维护为按需 surface）；快速打开/命令为临时 dialog，支持 Escape、首焦点与关闭后回焦。右栏“来源上下文”在无当前笔记映射时明确显示空态，而不是常驻展示无关项目资料；候选动作留在左侧“来源”视图。

## 3. 红转绿与回归

| 验证 | 实数 / 结果 |
| --- | --- |
| 红合同（实现前） | `knowledge-workbench-shell` 首跑累计 **13 项失败**：唯一壳、五个唯一结构区，以及旧 header/统计/三栏/第二编辑器/Graph/Canvas/维护根级追加均不满足。未删除或放宽这 13 项断言。 |
| 新聚焦合同（最终） | 通过：`knowledge workbench shell static convergence contract passed`。最终合同覆盖中央槽位/模式 reducer、左栏折叠、临时 overlay、Escape/回焦、来源候选动作及 static/browser 初始壳一致性。 |
| `knowledge-open-relay` 定向回归 | 通过。首次完整回归抓到缺少精确字符串“Syn 原生知识工作区”，在静态标题补回后转绿。 |
| 既有知识定向回归 | `native-knowledge-workspace`、`knowledge-vault-notes`、`knowledge-graph`、`knowledge-canvas`、`knowledge-attachment-recovery` 全部通过。 |
| 第二个聚合 catch | `knowledge-vault-notes` 仍要求精确字符串“Syn 原生工作区”。静态标题最终同时保留两个兼容名称后转绿；没有改旧测试。 |
| `npm run typecheck` | 通过，`tsc --noEmit` 无错误。 |
| `npm run test:offline-interaction` | 最终通过。runner 当前登记 **37** 个入口；`offline interaction tests passed: 15` 是其中首个测试模块自身的断言数，不能当作 runner 入口总数。 |

两次聚合 catch 已按规则写入 `docs/harness-catch-log.md`：它们证明新增聚焦断言不能替代既有静态兼容合同，而不是真实运行态故障。

红合同时序留口：实现前的 13 项红灯覆盖任务包的“唯一单壳 + 旧根级容器退场”结构部分；中央状态、折叠、overlay 回焦、来源动作和 static/browser 一致性的补充断言是在首轮转绿后根据复核加入。它们现已通过，但没有把全部七项都重走成实现前红灯，不能伪报为完整 red-first 证据。

## 4. 规定门禁与 shape

| 命令 | 结果 |
| --- | --- |
| `node scripts/harness/workbench-shape-gate.js --mode baseline` | `Status: pass`（baseline 模式强制通过）；**17 errors / 5 warnings / 5 infos**。 |
| `node scripts/harness/workbench-shape-gate.js --mode check` | `Status: fail`，同为 **17 errors / 5 warnings / 5 infos**。这些是当前脏工作树的既有仓库级 ratchet、超限文件、unknown sidecar 债务；没有发现本包新增类别。未越权修复或豁免。 |
| hardcoded-hex selftest | 13/13 通过；实际 UI 扫描为 0 violations（68 deferred-whitelisted）。 |
| machine-face selftest | 18/18 通过；实际 UI 扫描为 0 error-form、0 state-form warning（9 deferred-whitelisted）。 |
| retired style family | 实际 UI 扫描为 0 violations（1 deferred-whitelisted）。 |
| `git diff --check` | 通过。 |
| `git diff --cached --name-only` | 空。 |

`check` 的非零退出不能冒充 shape 全绿；它是本次回交的明确未结项。任务包的停止条件是“新增 finding/类别”，而两种规定模式列出的计数与类别相同，执行线未观测到由本包引入的新类别。

## 5. 行数、哈希与边界

| 文件 | 开工前 | 当前 |
| --- | ---: | ---: |
| `KnowledgeBaseView.tsx` | 1214 | 1145 |
| `NativeKnowledgeWorkspace.tsx` | 1067 | 1195 |
| `styles.css` | 11665 | 11812 |
| 新壳 + 状态模型 | 0 | 135 |
| 新聚焦测试 | 0 | 321 |

- 新壳与状态模型合计 135 行，低于任务包约 500 行的控制线。
- end HEAD：`e9ad7f3a204a1ebb11ce26c1e8c05b19c04c0991`（未提交、HEAD 未变）。
- `styles.css` 较开工净增 147 行，且窄返工在现有 R1 结果上净减 9 行；没有以 CSS 隐藏重复根容器通过合同。仓库级 styles ratchet 超水线在开工冻结脏基线中已存在，未在本包内修复或豁免。

## 6. 未覆盖项、剩余问题与交回

- 未启动真实 App/Home、Tauri 或 Vite；未访问真实 vault/store、Codex CLI/MCP、Obsidian；未做 I5、Gate 0、CLI/MCP 或 12 项真实工具测试。
- 未做真实浏览器/像素截图验收；overlay 的实际焦点行为、窄窗口视觉比例、内部滚动与最终视觉质感仍需后续获授权的真实 App/视觉包核验。当前证据仅覆盖静态 DOM、类型和离线交互合同。
- 仓库级 shape `check` 仍是非零，需指导线判断其既有聚合债务与本 R1 的关系；执行线没有为使其变绿而扩到 Rust、sidecar、依赖或任何共享边界。
- 红合同只记录了 13 项结构性 red-first 失败；新增的行为合同后补且现为绿色，是否要求另包补全红→绿过程须由指导线裁决。
- 被闸拦过的事项：完整离线聚合先后捕获两个精确静态兼容名称遗漏，均在白名单内修复；未发生白名单漂移、并行写冲突、staged 非空或需要扩白的情况。

## 7. 指导裁决后的窄返工（2026-07-25）

- 返工前重新冻结收窄白名单 SHA-256：`KnowledgeWorkbenchShell.tsx`=`2bdfee1d62db6bcd7fc448c992b14f0e0cdf32cca687f4225d7aeaeda820849a`、`NativeKnowledgeWorkspace.tsx`=`456153d284712beadd46053b4871417ef33fa8bcacbf4a86a117b641774b3e0a`、`knowledgeWorkbenchLayout.ts`=`cc3a9f8811edb3b6ac231a37738a3b6f0bfba0794aa66f81d4b3a14015fd083a`、`styles.css`=`a7d845d2fe06dc5de4d7caea35ae1552672c9db4f21946cafcd6d215870eb7dd`、聚焦测试=`e0e3ea12f5a8e8af6b012c084254f693e0bbc17c724642b7663039161b4f5eeb`。staged 为空，`lsof` 未发现白名单文件持有者。
- 新红合同首跑累计 **4 项失败**：1180px 断点无条件右栏零轨、900px 断点覆盖全部状态为双零轨、左栏折叠后仍保留可聚焦子项、右栏折叠后仍保留可聚焦子项。
- 修复没有改 reducer：现有“文件/搜索/来源”动作已把 `leftCollapsed` 清为 false，现有“上下文” toggle 已恢复右栏。CSS 改用 `--syn-knowledge-left-track` / `--syn-knowledge-right-track`；collapsed class 才置零，1180/900 仅收紧非零范围，保持横向单壳而不堆到中央下方。
- `KnowledgeWorkbenchShell` 把各 collapsed prop 同步到 aside 的 `aria-hidden` 与 `inert`，并在折叠时省略 sidebar children；活动栏不受影响，reducer 恢复状态后 children 和可访问语义一起恢复。
- 新聚焦合同绿：`knowledge workbench shell static convergence contract passed`。它直接读取 R1 样式片段断言两处媒体块不再无条件改写 grid 轨，并以 SSR 检查折叠 aside 的实际语义和缺失的可聚焦子项；不是只测 reducer/class。
- `npm run typecheck` 通过；完整 runner 的 37 个登记入口全部完成，单壳测试与既有 N0-N5/Graph/Canvas/附件回归均通过。shape baseline=`pass 17/5/5`，check=`fail 17/5/5`（既有聚合债务、无新增类别）；hex selftest=13/13，machine-face selftest=18/18。
- 没有出现新的真实 catch，因此未追加 `docs/harness-catch-log.md`。未启动 Vite/Tauri/Syn/真实 App，未做 1180/900 的真实浏览器截图或键盘走查；这些仍是后续获授权视觉/真实 App 阶段事项。

精确交回结论：**R1 的 React-only 单壳实现、13 项历史结构红转绿、以及本次 4 项响应式/可访问性红转绿均已完成；typecheck、37 入口完整离线 runner、两项 shape 自检和 diff-check 已完成。仍不宣称完整七项 historical red-first、真实 App/视觉验收或仓库级 shape check 全绿。**

## 8. 指导最终裁决（2026-07-25）

- 指导线独立确认 1180/900 断点只收紧非零轨宽，collapsed class 才将对应轨宽归零；左右 aside 折叠时同步 `aria-hidden`、`inert` 且不渲染子控件，活动栏恢复入口仍在。
- 新增四项合同确实能抓住旧实现。独立复跑聚焦合同、typecheck、37 入口完整离线 runner、shape baseline/check、两项 shape selftest、diff-check 和 staged 检查，结果与执行线回交一致。
- 独立复核没有发现新 catch；没有修改生产文件。
- 最终状态：**`ACCEPTED_N2R_R1_OFFLINE / NOT_REAL_APP_ACCEPTED`**。下一验证入口为隔离浏览器视觉基线，不把本证据升级为真实 Syn App 或高保真完成。
