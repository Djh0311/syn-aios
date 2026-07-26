# L3 Syn N2R-R3A Search 与 Overlay 收敛验证 evidence v1

- 日期：2026-07-26
- 执行结果：**`PASS_N2R_R3A_SEARCH_OVERLAY_CONVERGENCE / NEEDS_GUIDANCE_REVIEW / NOT_ACCEPTED`**
- 指导复核：**`NEEDS_N2R_R3A_REWORK / NOT_ACCEPTED`**
- 证据级别：真实 React 组件 + 真实生产 CSS + pure-synthetic fixture 的 fresh localhost browser context；**不是**真实 Syn/Tauri App、真实 vault/store、I5、Gate 0 或发布验收。
- 本线只交回证据与建议结论；不自行宣称指导验收通过。

## 1. 预检、权限与写入面

- HEAD：`e9ad7f3a204a1ebb11ce26c1e8c05b19c04c0991`；与任务包冻结值一致。
- 开工和收尾 staged 均为空。
- 任务包 §3.1 的 10 个冻结 SHA-256 在写前逐一匹配。
- 开工前 `127.0.0.1:5173` 无 listener，白名单没有并行写者。只启动了本包 synthetic fixture 的 Vite 与 fresh headless Chrome context；未启动 Syn、Tauri、Obsidian、真实 App、Codex CLI/MCP，也未接触真实 store/vault 或用户 Chrome profile。
- 保留既有脏工作树：`styles.css`、知识壳/fixture/tests 的先前 R1/R2 基线与 `docs/harness-catch-log.md` 均未被 reset、stash、覆盖或扩面。

本包实际写入：

| 文件 | 本次职责 |
| --- | --- |
| `prototypes/productized-desktop-shell/src/lib/knowledgeWorkbenchLayout.ts` | 共用 Arrow/Enter/Escape 映射、钳制式列表边界、冻结快捷键路由，以及可回焦 HTMLElement 类型。 |
| `prototypes/productized-desktop-shell/src/views/knowledge/NativeKnowledgeWorkspace.tsx` | 一个真实 search 状态源驱动左栏 Search / quick-open；紧凑 command 列表与既有安全表单的两阶段转换；键盘、ARIA 与焦点路径。 |
| `prototypes/productized-desktop-shell/src/styles.css` | 左栏/overlay 的紧凑列表层级、状态与提示；折叠 aside 的物理边框归零，消除 1px overflow；reduced-motion 不依赖动画。 |
| `prototypes/productized-desktop-shell/tests/knowledge-workbench-shell.test.tsx` | R3A 的共享列表键盘与 `⌘O` / `⌘P` / `⌘⇧F` 纯合同。 |
| `evidence/raw/2026-07-26-l3-syn-n2r-r3a-search-overlay/**` | red/green 原始浏览器报告、三张 screenshot 与可复现离线检查脚本。 |
| 本 evidence 与 R3A task package | 实际结果回写。 |

未修改 Rust、App、Graph、Canvas、Maintenance、活动栏结构/视觉、右栏、依赖、runner、fixture、`CURRENT.md`、`AUTHORITY.md`、计划或 catch log。

## 2. red-first 与转绿

实现前的 browser red 合同保留在 [red-contracts.json](raw/2026-07-26-l3-syn-n2r-r3a-search-overlay/red-contracts.json)：**20 项断言，17 项失败**，为 `EXPECTED_RED`。

- 两种 `900×760` 折叠组合仍有 shell `899 / 898` 的 1px 横向溢出；
- 左栏没有原位 combobox/listbox；
- quick-open 与 command 都没有 combobox/listbox/option、唯一当前项或 `↑↓ / Enter / Esc` 提示；
- `⌘O`、`⌘P`、`⌘⇧F` 都没有路由。

同一轮新增的纯函数合同先以 **2 项失败**落地：共享列表键盘 helper 与快捷键 helper 在旧实现中不存在。实现后一次聚焦运行还抓到 `-1` 未选中项按 ArrowDown 错进第二项的边界缺口；已改为首次导航落到第 0 项，再重跑为绿。

最终 browser green 报告为 [green-browser-evidence.json](raw/2026-07-26-l3-syn-n2r-r3a-search-overlay/green-browser-evidence.json)：**74 项断言 / 0 失败 / 6 个 fresh context**。

## 3. 实现事实

### 单一搜索源、ARIA、键盘与焦点

- 左栏和 quick-open 共享既有 `client.search(query)`、`searchQuery`、结果、状态和当前项；没有第二个数据源或静态结果。
- 输入变化清空旧结果与当前项；显式 Enter / 按钮才发出实际查询。响应以 request id + 当前 trim query 防止迟到结果回填。
- 左栏、quick-open、command 分别使用唯一 input/listbox/option id，combobox 用 `aria-controls` 与 `aria-activedescendant` 关联唯一当前项。
- Arrow 的冻结策略为钳制：空列表 `-1`，首次 Arrow 到 `0`，首/尾继续 Arrow 留在首/尾；Enter 激活；overlay Escape 由壳关闭并回焦。
- `⌘O` 打开 quick-open，`⌘P` 打开 command，`⌘⇧F` 展开左栏 Search 并聚焦其输入；组件卸载时移除 window listener。
- quick-open 成功打开只关闭 overlay、不会把焦点还给触发器；成功后的焦点进入中央已打开 tab。Escape 则回到原触发按钮。
- command 首屏只登记真实既有“新建 Markdown”“新建目录”；Enter 只进入原有受限相对路径表单，浏览器验证没有点“创建”。

### `900×760` 量尺与折叠可访问性

| fresh 场景 | document | body | shell | 结果 |
| --- | --- | --- | --- | --- |
| 双栏展开（command context） | `900 / 900` | `900 / 900` | `898 / 898` | PASS |
| 仅左栏折叠 | `900 / 900` | `900 / 900` | `898 / 898` | PASS |
| 仅右栏折叠 | `900 / 900` | `900 / 900` | `898 / 898` | PASS |
| 双栏折叠 | `900 / 900` | `900 / 900` | `898 / 898` | PASS |

折叠事实来自真实盒模型修复：被折叠的 aside 保持 0 track，并将物理 border width 归零；没有把两栏强制压成 0、缩放页面或裁掉内容。浏览器中左/右折叠区域各自为 `aria-hidden=true`、`inert=true`、可交互子项 `0`；双折叠时两者均为 `0`，活动栏仍有 8 个恢复控件。

## 4. 三张规定实图与 R0 对照

三图均只含 synthetic 数据。下表保留执行线回交判断，并追加指导复核；指导判断优先。

| 实图 | R0 对照 | 观察与判定 |
| --- | --- | --- |
| [01-1180-left-search-results.png](raw/2026-07-26-l3-syn-n2r-r3a-search-overlay/01-1180-left-search-results.png) | R0 03 Search | **PASS（R3A 范围）**：左栏原位单输入、16 条紧凑结果、路径和低层级 snippet、一个当前行；不新增中央编辑器或 overlay 结果替身。R0 的活动栏/多标签/右栏细节仍不在本包结论内。 |
| [02-1180-quick-open-results.png](raw/2026-07-26-l3-syn-n2r-r3a-search-overlay/02-1180-quick-open-results.png) | R0 06 Quick switcher | 执行线：**PASS（R3A 范围）**。指导：**GAP**。输入、真实 16 项结果、当前行和键盘提示成立；但全屏不透明 backdrop 把底层工作区完全抹掉，临时 overlay 视觉上变成第二张空白页面，不符合 R0 的层叠关系。 |
| [03-900-command-filter-results.png](raw/2026-07-26-l3-syn-n2r-r3a-search-overlay/03-900-command-filter-results.png) | R0 05 Command palette | 执行线：**PASS（R3A 范围）**。指导：**GAP**。真实命令过滤、当前项和键盘提示成立；同样因不透明 backdrop 丢失底层工作区上下文，未形成冻结参考中的临时浮层。 |

## 5. fresh-context mock、零写与焦点事实

每个 context mount 前 `localStorageEmptyBeforeMount=true`。精确 read allowlist 均为：

```text
knowledge_workspace_snapshot
knowledge_workspace_search
knowledge_workspace_graph
knowledge_workspace_read_markdown
knowledge_workspace_read_canvas
```

| context | 实际 read 调用 | write / unknown / external / console / page error |
| --- | --- | --- |
| 左栏 Search | `snapshot=1, search=1, read_markdown=1` | `0 / 0 / 0 / 0 / 0` |
| quick-open | `snapshot=1, search=1` | `0 / 0 / 0 / 0 / 0` |
| command | `snapshot=1` | `0 / 0 / 0 / 0 / 0` |
| 左折叠 / 右折叠 / 双折叠 | 各 `snapshot=1` | 每个均 `0 / 0 / 0 / 0 / 0` |

交互实数：左栏 `合成` 得到 `16` 项、ArrowDown 到 option `1`、Enter 后焦点在中央 `role=tab`；quick-open 也是 `16` 项、Escape 后焦点回到 `搜索`；command 初始为两个安全命令，过滤后一个当前项，Enter 后只有一个受限路径 input、创建按钮 disabled，Escape 后回到 `Syn 命令`。

## 6. 必跑门禁

| 验证 | 实际结果 |
| --- | --- |
| 聚焦 `knowledge-workbench-shell` 合同 | PASS：先发现并修正一次 `-1` Arrow 边界后，最终输出 `knowledge workbench shell static convergence contract passed`。 |
| 聚焦 `native-knowledge-workspace` 合同 | PASS：`native knowledge workspace ... fixed client contract tests passed`。 |
| `npm run typecheck` | PASS，exit `0`。 |
| `npm run test:offline-interaction` | PASS，exit `0`；runner 登记 **37** 个 entry。`offline interaction tests passed: 15` 是首个测试自身的断言输出，绝非 runner 数量。 |
| green browser evidence | PASS：`74 / 74`、6 fresh context、三图与四组 900px 量尺。 |
| shape baseline | PASS：`Errors: 17 / Warnings: 5 / Info: 5`，既有全仓债务。 |
| shape check | 非零 FAIL：仍为 `17 / 5 / 5`，无新增类别或 finding。 |
| hardcoded-hex selftest | PASS：`13 / 13`。 |
| machine-face selftest | PASS：`18 / 18`。 |
| `git diff --check` | PASS，exit `0`。 |
| `git diff --cached --name-only` | PASS：无输出，staged 为空。 |

## 7. 遗留、catch 与交回边界

- 执行线回交时报告零新 catch；指导线复核新增一项真实 catch：overlay 的行为/DOM 绿灯和列表内容被误外推为视觉层级 PASS，而生产 CSS 与实图均显示底层上下文被不透明遮罩完全删除。已追加 `docs/harness-catch-log.md`。
- 未做、也不声称完成：真实 Syn/Tauri App、真实 store/vault、I5、Gate 0、发布验收；活动栏图标化、标签组/分栏、Canvas、Graph、右栏层级和其余 R0 高保真差距。
- shape check 的非零仍是上游既有问题，不能被本次前端局部绿灯掩盖。
- 本 evidence 支持 R3A 的代码/隔离浏览器门禁通过；指导复核已完成，最终状态改为 **`NEEDS_N2R_R3A_REWORK / NOT_ACCEPTED`**。

## 8. 收尾复核

- 本包启动的 Vite 已用其自身 session 关闭；最终 `lsof -n -P -iTCP:5173 -sTCP:LISTEN` 无输出。
- `git diff --check` exit `0`；`git diff --cached --name-only` 无输出。
- 工作树仍为用户既有的大量脏改动加本包白名单改动；没有执行 stage、commit、push、reset、clean、stash 或 checkout。

## 9. 指导线独立复核

- 源码事实：`.syn-knowledge-overlay-backdrop` 当前为 fixed 全屏层，且 `background: var(--panel-soft)` 不含透明度；这与 02/03 图的纯色空白背景一致，不是截图工具或量尺误差。
- 独立复跑：`npm run typecheck` exit `0`；正式 `npm run test:offline-interaction` 完整退出 `0`，其中包含两个知识聚焦合同。指导线临时逐文件加载器也打印两项聚焦通过，但因测试保留定时器未自然退场，主动中止该临时加载器，其退出码不计作绿灯。shape baseline 为 `PASS 17 / 5 / 5`、check 为既有债务 `FAIL 17 / 5 / 5`；hardcoded-hex `13 / 13`、machine-face `18 / 18`。
- 独立收尾：`git diff --check` 无输出；`git diff --cached --name-only` 无输出；5173 无 listener。工作树的大量既有脏项保持原状。
- 行为/隔离/量尺证据继续成立；视觉结论不成立。精确裁决为 **`NEEDS_N2R_R3A_REWORK / NOT_ACCEPTED`**，且不外推真实 Syn/Tauri App、完整 R0 或发布验收。
