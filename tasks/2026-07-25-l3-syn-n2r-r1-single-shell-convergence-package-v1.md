# 任务包：L3 Syn N2R-R1 React-only 单壳结构收口 v1

- 日期：2026-07-25
- 状态：**ACCEPTED_N2R_R1_OFFLINE / NOT_REAL_APP_ACCEPTED**
- 负责人：独立知识前端执行线
- 指导/验收：当前总指导对话
- 总计划：`docs/plans/2026-07-16-master-execution-plan-conversation-first-v1.md`
- 小阶段计划：`docs/plans/2026-07-23-l3-syn-native-knowledge-workspace-small-stage-plan-v2.md`
- 决策：`decisions/2026-07-23-l3-syn-native-knowledge-workspace-route-v2.md`
- 冻结参考：`docs/design/2026-07-25-l3-obsidian-core-desktop-reference-and-migration-r0-v1.md`

## 0. Kickoff

- 任务：把当前纵向叠放的新旧知识界面收拢为一个 React-only 桌面单壳；复用现有 Markdown、Graph、Canvas、附件/恢复和来源能力，不重做 Rust/backend。
- 负责人：获得用户 kickoff 后，由独立知识前端执行线施工；总指导只派发、核实物和验收，不直接施工。
- 交付物：
  1. 唯一知识工作台根；
  2. 固定活动栏、左侧栏、中央标签工作区、右侧上下文栏和底部状态区；
  3. Graph/Canvas/编辑器只在中央工作区出现；
  4. 旧统计、旧三栏、第二编辑器和维护区退出主页面常驻层；
  5. 聚焦测试、完整前端离线回归和离线 evidence。
- 完成标准：§7 全部通过，指导线独立核 diff 和测试后接受。

用户已明确“派”后完成本包离线施工；实际结果与未验收状态见 §11。用户说“开工/派发本包”前不得写业务代码。

## 1. 权威与设计意图

```yaml
authority_chain:
  - AGENTS.md
  - CURRENT.md
  - decisions/2026-07-23-knowledge-and-conversation-parallel-workstreams-v1.md
  - decisions/2026-07-23-l3-syn-native-knowledge-workspace-route-v2.md
  - docs/plans/2026-07-16-master-execution-plan-conversation-first-v1.md
  - docs/plans/2026-07-23-l3-syn-native-knowledge-workspace-small-stage-plan-v2.md
  - docs/design/2026-07-25-l3-obsidian-core-desktop-reference-and-migration-r0-v1.md
  - tasks/2026-07-25-l3-syn-n2r-r1-single-shell-convergence-package-v1.md
```

### 使用者、任务与感受

- 使用者：在 macOS 上长时间浏览、编辑、搜索和关联知识的单人桌面用户。
- 核心任务：不离开当前上下文，在文件、笔记、链接关系、Graph 和 Canvas 之间切换。
- 感受：紧凑、安静、连续，像一个真正的桌面知识工具；不是卡片仪表盘或多个页面模块的拼盘。
- 唯一视觉焦点：中央内容工作区。

### 产品域探索

- Domain：vault、目录树、笔记、双链、属性、反链、图谱、Canvas、来源、审计。
- Color world：纸张、石墨、墨色、旧木、青玉状态、朱砂确认；实际颜色仍只取 Syn 既有语义 token。
- Signature：AI 确认、来源和审计只在与当前笔记相关时进入右侧上下文或低层状态，不另造 AI 仪表盘。
- Rejecting：
  - 卡片统计首页 → 固定桌面单壳；
  - Graph/Canvas 全宽纵向追加 → 中央标签/分栏；
  - 复制 Obsidian 品牌和资产 → Syn 自有文案、token 与通用语义图标。

## 2. 当前已确认代码事实

`KnowledgeBaseView` 当前主页面按顺序渲染：

1. 页面级标题；
2. `NativeKnowledgeWorkspace`；
3. `KnowledgeGraphView`；
4. `KnowledgeCanvasView`；
5. `KnowledgeWorkspaceMaintenancePanel`；
6. 统计条和旧知识三栏；
7. 第二套 `KnowledgeVaultNotesPanel`；
8. 页面级警告。

当前“双显示容器”不是单一 CSS 问题。必须改 React 渲染归属，不能只用 `display: none`、绝对定位或负 margin 遮住重复结构。

必须保住的现有连接：

- `knowledgeOpenIntent` 与 outcome 回传；
- Graph 节点打开 Markdown；
- 附件引用插入当前草稿；
- vault save/focus refresh；
- stale revision/mtime/hash 冲突拒绝；
- 来源、候选动作和 warning 的真实数据；
- Obsidian 只作为可选外部兼容入口。

## 3. 脏工作树与并发前置

本包起点 HEAD：

`e9ad7f3a204a1ebb11ce26c1e8c05b19c04c0991`

当前白名单文件含既有未提交或未跟踪工作，全部按 **merge-only** 处理。开工前必须重新核对以下 SHA-256：

| 文件 | 2026-07-25 指导冻结 SHA-256 |
| --- | --- |
| `src/views/KnowledgeBaseView.tsx` | `0138a0625fb054f6c622cdff903b5bb1163e28274c9f5b4a708c827aca803e85` |
| `src/views/knowledge/NativeKnowledgeWorkspace.tsx` | `a6d078b3538cd1c8387f33e40cfa0e3f7bcf31eb168987babbe61775bc3ed58c` |
| `src/styles.css` | `454ccbf317d5fde1085e49b433cc5edf98269d77d404015423a9195647304c0f` |
| `tests/native-knowledge-workspace.test.tsx` | `fdcdfe54fb8f6dd022d4b018635d3bca1f2cd75874f2a835859dacfaa406408e` |
| `tests/knowledge-vault-notes.test.tsx` | `1acf93a236b853da2975d1607854e5bc5c45b5f370226e1e6d76654c6a4f6227` |

相对路径均以 `prototypes/productized-desktop-shell/` 为根。

开工硬前置：

1. staged 为空；
2. 上述 hash 全部一致；
3. 没有其他线程正在写本包白名单；
4. 没有 Syn/Tauri/Vite/Codex/MCP 真实验收进程；
5. 记录 `git status --short` 和白名单逐文件 provenance；
6. 不 reset、clean、stash、checkout 或覆盖用户 WIP。

任一不满足，停止为 `BLOCKED_N2R_R1_DIRTY_BASELINE_DRIFT` 或 `BLOCKED_N2R_R1_WRITE_OWNERSHIP_CONFLICT`，不得自行修现场。

## 4. 精确写入白名单

### 现有文件

- `prototypes/productized-desktop-shell/src/views/KnowledgeBaseView.tsx`
- `prototypes/productized-desktop-shell/src/views/knowledge/NativeKnowledgeWorkspace.tsx`
- `prototypes/productized-desktop-shell/src/styles.css`
- `prototypes/productized-desktop-shell/tests/native-knowledge-workspace.test.tsx`
- `prototypes/productized-desktop-shell/tests/knowledge-vault-notes.test.tsx`
- `prototypes/productized-desktop-shell/scripts/run-offline-interaction-test.mjs`：仅在新增测试文件时机械追加一次测试路径

### 允许新增

- `prototypes/productized-desktop-shell/src/views/knowledge/KnowledgeWorkbenchShell.tsx`
- `prototypes/productized-desktop-shell/src/lib/knowledgeWorkbenchLayout.ts`
- `prototypes/productized-desktop-shell/tests/knowledge-workbench-shell.test.tsx`
- `evidence/2026-07-25-l3-syn-n2r-r1-single-shell-convergence-offline-verification-v1.md`

### 文档

- 本任务包：只更新实际状态、实际白名单和回交结果。
- `docs/harness-catch-log.md`：只有出现新的真实 catch 才追加。

除此之外一律不写。若必须修改 `App.tsx`、Graph/Canvas/Maintenance 组件、任何 `src-tauri`、Cargo、package 依赖、共享 transport/binding/MCP、CURRENT 或 AUTHORITY，立即停止申请扩白。

## 5. 实现合同

### 5.1 唯一 DOM 结构

主知识页面必须只保留：

```text
KnowledgeBaseView
└─ KnowledgeWorkbenchShell
   ├─ activity rail
   ├─ left sidebar
   ├─ central tab group workspace
   ├─ right context sidebar
   └─ status bar
```

要求：

- `KnowledgeWorkbenchShell` 根严格 1 个；
- 活动栏、左栏、中央工作区、右栏、状态区各严格 1 个；
- 页面外层不再渲染 `.pg-head`、`.knowledge-base-stats`、`.knowledge-base-grid` 或常驻 `.knowledge-vault-notes`；
- Graph、Canvas、Markdown/Preview 只能作为中央当前标签或分栏内容出现，不能作为根级兄弟纵向追加；
- 维护能力只能通过显式 activity/command 打开的按需抽屉或中央临时标签进入；
- 旧来源/候选独有信息迁入左侧“来源”视图和右侧“来源上下文”，不得丢失或伪造。

### 5.2 壳状态

- 活动栏至少提供文件、搜索、Graph、Canvas、命令和设置/维护入口。
- 左侧栏首包覆盖文件、搜索、来源；标签视图若现有数据不足，显示明确 disabled/empty，不新增后端。
- 中央状态模型至少区分 `markdown`、`preview`、`graph`、`canvas`，并保留当前标签和当前分组。
- 首包允许不做拖拽重排，但状态模型不能把 Graph/Canvas 写死成页面下方区域。
- 右侧栏至少覆盖属性、反链、大纲/上下文、来源；只显示当前内容相关数据。
- 左右栏可独立折叠；左栏折叠后活动栏仍存在。
- 快速打开和命令面板从常驻工具行改为临时 overlay；关闭后焦点返回触发控件。
- 底部只放当前文件、模式、保存/冲突、字数或引用等低层级短状态。

### 5.3 参考比例和视觉边界

R1 不是最终视觉验收，但结构必须按冻结参照建立：

- `984 × 768` 参考下活动栏约 `42 px`；
- 左栏展开约 `288 px`；
- 右栏约 `185 px`；
- 顶部集成标签区约 `39 px`，视图工具栏约 `35 px`；
- 中央状态区约 `26 px`；
- `1180 × 760` 与 `1440 × 900` 下维持桌面单壳，优先折叠侧栏，不堆到中央下方；
- 使用 Syn 既有 token、字体和通用语义图标；不复制 Obsidian logo、原图标包、CSS 或品牌资产；
- 新样式必须收敛在 `.syn-knowledge-shell` 命名空间，不新增随机 hex、渐变、营销卡片或多重阴影系统。

### 5.4 状态与可访问性

- 文件/视图/侧栏切换使用真实 `button`、tab 或等价可访问 primitive，不使用 `div onClick`。
- 当前活动项、折叠状态和 tab 选中具有明确 ARIA 状态。
- overlay 支持 Escape、焦点进入和焦点返回。
- loading、empty、unavailable、error、conflict、disabled、hover、active、focus-visible 和 reduced-motion 均有明确表现；首包未覆盖的状态必须作为阻塞，不得静默删掉。
- 高频 tab/命令切换不增加拖沓动画；任何偶发 overlay 动画只动 `transform/opacity` 且尊重 reduced motion。

## 6. 变更辐射面与五态走查

### 变更辐射面

| 改变的假设 | 既有依赖 | 本包必须核对 |
| --- | --- | --- |
| 页面从纵向多模块变为固定单壳 | `stage-pad`、外层滚动、紧凑窗口 CSS | 主页面不滚动；左右中区域内部滚动 |
| Graph/Canvas 改为中央标签 | graph open request、Canvas refresh/mutation | 节点打开、保存和刷新事件不丢 |
| Native workspace 拆入壳槽位 | 文件树、草稿、预览、反链、属性 | 选择、编辑、保存、冲突、反链仍有唯一状态源 |
| 旧三栏与第二编辑器退场 | 来源、候选动作、Obsidian 兼容入口 | 独有信息有明确新归属 |
| 快速打开改 overlay | 搜索状态、焦点、键盘路径 | 打开/关闭/选择/焦点返回 |
| 维护区改按需入口 | 附件引用、备份、恢复 | 能进入但不常驻；不改能力实现 |

### 五态旅程

- 说：不改变交办对话面；知识壳只提供只读引用上下文。
- 批：不改变 Pending/批准动作；若存在关联知识，只进入右侧上下文，不新建主卡片。
- 干：不改变 chain/worker；知识壳保持可读，不能因运行态退回旧多容器。
- 交货：不改变交货卡；来源和引用仍可在右侧上下文查看。
- 卡住：知识加载错误、保存冲突和维护失败必须在当前上下文显式呈现，不用旧页面警告堆兜底。

## 7. 红合同与验收

### 7.1 先写红合同

至少新增下列预期失败断言，再实施：

1. `KnowledgeBaseView` 静态渲染只有一个 shell 根和五个唯一结构区域；
2. 根级渲染不存在旧统计、旧三栏、第二编辑器、常驻 Graph/Canvas/Maintenance；
3. 切换中央状态时同一位置呈现 Markdown、Graph 或 Canvas，不增加根级兄弟；
4. 左栏折叠后活动栏仍存在；
5. 快速打开/命令 overlay 关闭后焦点回到触发按钮；
6. 来源和候选动作没有因旧三栏退场而消失；
7. static/server 分支与 browser 分支使用同一单壳结构，不再由 static 分支一次渲染四个主组件。

记录红灯实数和失败原因，不得通过删除旧断言或放宽选择器制造绿色。

### 7.2 必跑验证

从 `prototypes/productized-desktop-shell` 执行：

1. 聚焦 `knowledge-workbench-shell` 测试；
2. `native-knowledge-workspace`、`knowledge-vault-notes`、`knowledge-graph`、`knowledge-canvas`、`knowledge-attachment-recovery` 定向回归；
3. `npm run typecheck`；
4. `npm run test:offline-interaction`；

从仓库根执行：

5. `node scripts/harness/workbench-shape-gate.js --mode baseline`；
6. `node scripts/harness/workbench-shape-gate.js --mode check`；
7. `node scripts/harness/workbench-shape-gate.hardcoded-hex.selftest.js`；
8. `node scripts/harness/workbench-shape-gate.machine-face.selftest.js`；
9. `git diff --check`；
10. `git diff --cached --name-only` 为空。

本包禁止 Rust 修改，不以 `cargo check` 作为成功门。R1 只结算结构和离线行为；真实截图与最终像素对照留给后续获授权的视觉/真实 App 阶段。

## 8. 形状影响

- 任务类型：功能任务包，React-only 结构重构。
- 新增代码落点：最多两个知识壳/状态模型文件和一个聚焦测试文件。
- 棘轮文件：`src/styles.css`、离线 runner 和既有大型 `KnowledgeBaseView.tsx`；必须报告前后行数和 shape 三数。
- 预计变化：
  - `KnowledgeBaseView.tsx`：删除根级旧渲染并改为壳编排，目标净减；
  - `NativeKnowledgeWorkspace.tsx`：拆出可嵌入内容/上下文，目标不新增第二状态源；
  - 新壳与状态模型：合计目标不超过约 `500` 行；
  - `styles.css`：新增壳命名空间的同时删除确认无消费者的旧主布局规则，禁止只增不退。
- 不新增 Tauri command、sidecar JSON、依赖、schema 或 shape 豁免。
- 本任务基线 commit：`e9ad7f3a204a1ebb11ce26c1e8c05b19c04c0991` + §3 脏基线 hash。
- 本任务完成 commit：不 commit；回交时报告 end working-tree hash。

## 9. 并发与停止条件

本包施工期间允许对话线做只读准备，但禁止：

- 启动对话三句真实 App 重验；
- 构建或启动 Syn/Tauri/Vite；
- 读写真实 store/vault、Codex CLI/MCP；
- 修改共享 Rust、binding、registry、allowlist、runtime profile；
- 安装/操作 Obsidian、访问其他 vault；
- stage、commit、push、reset、clean、stash。

立即停止：

- 白名单 hash 漂移或出现无法归属的并行 hunk；
- 必须扩到 `App.tsx`、Graph/Canvas/Maintenance 源码、Rust、依赖或 schema；
- 需要 CSS 隐藏重复容器才能过测试；
- 旧来源、候选、附件、恢复、冲突或 knowledge-open 行为无法迁移；
- shape 新增 finding/类别，或 staged 非空。

## 10. 必须回传

1. 红合同失败与转绿实数；
2. 实际修改文件及白名单核对；
3. 单壳 DOM 结构与旧根级容器清零证据；
4. 文件/搜索/来源、中央四模式、右侧上下文、状态栏和 overlay 的行为；
5. knowledge-open、Graph open、附件引用、refresh、冲突和来源动作回归；
6. typecheck、聚焦/完整离线测试、shape、自测和 diff-check 实数；
7. 前后行数、start/end hash、staged 状态；
8. 未覆盖的视觉/交互项和下一包建议；
9. 被闸拦过的事；没有也写“无”。

执行线自报完成不等于验收通过。总指导必须独立核 diff、复跑聚焦测试并裁决“接受 / 需要修改 / 暂停 / 废弃”。

## 11. 执行回交（2026-07-25）

- 开工 preflight 通过：冻结 HEAD 与五个 SHA-256 一致、staged 为空、白名单无并行写入者，且未发现以本仓库为 cwd 的 Syn/Tauri/Vite/Codex/MCP 真实验收进程。
- 已完成 React-only 单壳收敛；实际写入仍在 §4 白名单内，未改 Rust、`App.tsx`、launcher、真实 store/vault、MCP、`CURRENT.md` 或 `AUTHORITY.md`，未 stage/commit/push。
- 结构红合同实现前累计 13 项失败，最终聚焦合同、五项定向知识回归、`npm run typecheck` 与完整离线 runner 均通过；完整 runner 的两个静态兼容 catch 已在白名单内修复并记入 catch-log。
- 最终 shape baseline 为 `17 errors / 5 warnings / 5 infos`，check 同计数非零退出；未观测到本包新增 finding/类别。hardcoded-hex 与 machine-face selftest 分别为 13/13、18/18；diff-check 及 staged 复核见 evidence。
- 红合同时序留口：13 项结构断言为 red-first；后续复核补进的中央状态、折叠、overlay、来源和 static/browser 行为断言现为绿色，但没有完整实现前红灯记录。
- 证据：`evidence/2026-07-25-l3-syn-n2r-r1-single-shell-convergence-offline-verification-v1.md`。本段是当时的执行回交记录：状态为 **NEEDS_GUIDANCE_REVIEW / NOT ACCEPTED**；随后已由 §12 的独立裁决覆盖。

### 11.1 指导验收后的窄返工（2026-07-25）

- 指导裁决为 `NEEDS_MODIFICATION / NOT ACCEPTED` 后，用户明确批准在原 R1 范围内只修响应式侧栏状态收敛与折叠侧栏可访问性两个缺口；返工前已重新核对收窄白名单 hash、staged 和写所有权，无漂移或并发冲突。
- 新增聚焦合同先以 **4 项失败** 记录红灯：1180px 无条件右栏零轨、900px 全状态双零轨、左栏折叠后仍有可聚焦内容、右栏折叠后仍有可聚焦内容。13 项初次结构 red-first 的历史范围不变，未将其改写成完整行为 red-first。
- 未新增状态源或改 reducer：既有活动栏动作恢复左右栏状态。`KnowledgeWorkbenchShell` 令视觉折叠的 aside 同步 `aria-hidden`/`inert` 且不渲染 sidebar children；样式以左右轨宽变量让 collapsed class 才归零，1180/900 断点只收紧非零宽度，保持横向桌面单壳而不堆叠到中央下方。
- 新合同转绿；`npm run typecheck` 与 `npm run test:offline-interaction` 通过。完整离线 runner 当前登记 **37** 个入口；`offline interaction tests passed: 15` 仅是首个测试模块自身断言输出。shape baseline 为 `pass 17/5/5`、check 为 `fail 17/5/5`（既有聚合债务，未见新增类别），两项 shape 自测为 13/13、18/18。
- 本次实际写入：`KnowledgeWorkbenchShell.tsx`、`styles.css`、`knowledge-workbench-shell.test.tsx`、本任务包与原 R1 evidence。未改 `NativeKnowledgeWorkspace.tsx`、`knowledgeWorkbenchLayout.ts`、runner 或 catch-log；没有新真实 catch。
- 未启动 Vite/Tauri/Syn/真实 App，未做 1180/900 真实浏览器截图或键盘视觉走查；这些不是本次离线返工验收的替代物。最终 diff/staged 复核与全部命令实数见 evidence。本段保留当时的 **NEEDS_GUIDANCE_REVIEW / NOT ACCEPTED** 回交状态；随后已由 §12 的独立裁决覆盖。

## 12. 指导最终验收（2026-07-25）

- 指导线独立核对断点 CSS、collapsed aside 的 DOM/焦点路径和新增四项合同，确认旧实现会被合同抓住，返工实现没有新增第二状态源。
- 独立复跑聚焦合同、`npm run typecheck`、37 入口完整离线 runner、shape baseline/check、两项 shape selftest、`git diff --check` 与 staged 检查；结果与回交一致。
- shape check 仍为既有 `17/5/5` 非零债务；13 项初始结构 red-first 的历史范围不改写。
- 精确结论：**`ACCEPTED_N2R_R1_OFFLINE / NOT_REAL_APP_ACCEPTED`**。R1 只关闭 React 单壳、响应式状态和折叠可访问性；真实浏览器像素、键盘视觉和真实 Syn App 均留给后续包。
- 指导复核零新 catch，没有修改 R1 生产代码。
