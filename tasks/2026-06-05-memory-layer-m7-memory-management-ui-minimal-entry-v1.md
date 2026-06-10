# Task Package：Memory Layer M7 Memory Management UI Minimal Entry v1

状态：已完成。  
用途：实现中间版本记忆层 M7：记忆管理 UI 最小入口。  
执行方式：一个中等批次完成；开发重点是正式记忆 / 候选 / 来源 / 版本 / 审计 / 冲突 / 任务包入选状态的可理解读模型和 UI，不做正式记忆生命周期操作。

完成记录：

- `evidence/2026-06-05-memory-layer-m7-memory-management-ui-minimal-entry-v1.md`
- `handoffs/2026-06-05-memory-layer-m7-memory-management-ui-minimal-entry-v1-result.md`

## 1. 先说薄弱点

- M1-M6 已完成第一条真实记忆闭环：正式记忆 store、上下文绑定、候选采纳、observation 入口、任务记忆包预览、lint blocking 和任务包注入。
- 当前“记忆”入口仍偏工程摘要，容易显示 sidecar、revision、内部 id 和治理细节；这不是用户能看懂的记忆中心。
- 项目工作流侧栏已经能显示候选、lint、任务记忆包和注入摘要，但它服务项目流程，不应变成全局记忆管理中心。
- M7 必然会改 UI 和读模型；如果边界不写清，容易把 `candidate_confirmed` 显示成“已记住”，或把 observation / knowledge hit / LLM summary 包装成正式记忆。
- M7 只是记忆管理 UI 最小入口，不是 M8 知识库接口，不是 M9 生命周期操作，不是 M10 关系治理，也不是 M13 记忆系统总验收。

## 2. 任务目标

新增或重构一个用户可理解的“记忆管理最小入口”：

```text
FormalMemoryStore
MemoryCandidateStore
ObservationStore
MemoryLintStore
TaskMemoryPacket preview / injection summary
-> Memory Management Read Model
-> 全局「记忆」入口
-> 项目相关记忆摘要
```

M7 完成后可以说：

- 用户和项目主管能在“记忆”入口看清正式记忆和候选记忆的区别。
- 正式记忆列表能显示来源、版本、状态、scope、权限 / 外发限制、冲突 / lint 摘要和是否可进入任务包。
- 候选列表能显示候选状态、来源、风险、确认要求和是否已被受控采纳，但不能显示成正式记忆。
- 记忆详情能显示来源面板、版本摘要、审计摘要、冲突提示和任务包入选 / 排除原因。
- 项目页如显示记忆，只显示当前项目相关的轻量摘要，不铺 raw audit、sidecar 路径或完整治理后台。
- UI 文案能明确区分：观察、候选、正式记忆、任务包预览和任务包冻结快照。

M7 完成后仍不能说：

- 中间版本记忆系统完成。
- 正式记忆生命周期操作完成。
- 知识库 / Obsidian 接口完成。
- 实体关系治理完成。
- 维护任务、成熟模式或跨项目记忆完成。
- UI 可以直接写正式记忆。
- `candidate_confirmed`、observation、knowledge hit 或 LLM summary 已成为正式记忆。

## 3. 前置条件

必须已完成：

- `tasks/2026-06-03-memory-layer-m1-formal-memory-store-and-audit-v1.md`
- `tasks/2026-06-03-memory-layer-m1-1-formal-memory-context-binding-guard-v1.md`
- `tasks/2026-06-03-memory-layer-m2-candidate-to-formal-adoption-v1.md`
- `tasks/2026-06-04-memory-layer-m3-observation-store-and-workflow-entry-v1.md`
- `tasks/2026-06-04-memory-layer-m4-task-memory-packet-builder-and-preview-v1.md`
- `tasks/2026-06-04-memory-layer-m5-conflict-and-memory-lint-minimal-blocking-v1.md`
- `tasks/2026-06-04-memory-layer-m6-workflow-task-package-injection-and-end-to-end-loop-v1.md`

开始前必须复核：

- 当前 “记忆” 一级入口已经存在，不需要新增一级导航。
- 当前 `FormalMemoryStore` 已有 `MemoryRecord`、`MemoryVersion`、`MemoryAuditEvent`。
- 当前 `MemoryCandidateStore` 已能记录候选状态和采纳回链。
- 当前 `ObservationStore` 只记录明确工作流事件和来源，不是正式记忆。
- 当前 `MemoryLintStore` / lint finding 能表达 blocking / needs_review / open / resolved 等状态。
- 当前 `TaskMemoryPacketBuilder` / M6 注入摘要能解释 included / excluded / review materials。
- 当前没有授权执行真实 worker / Codex；本任务默认只做读模型和 UI。

## 4. 必须先读

当前入口：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `docs/middleware-version-development-plan-v1.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `docs/plans/memory-layer-implementation-slice-v1.md`
- `docs/memory-layer-design-v1.md`
- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`

前置记录：

- `evidence/2026-06-03-memory-layer-m1-formal-memory-store-and-audit-v1.md`
- `evidence/2026-06-03-memory-layer-m1-1-formal-memory-context-binding-guard-v1.md`
- `evidence/2026-06-03-memory-layer-m2-candidate-to-formal-adoption-v1.md`
- `evidence/2026-06-04-memory-layer-m3-observation-store-and-workflow-entry-v1.md`
- `evidence/2026-06-04-memory-layer-m4-task-memory-packet-builder-and-preview-v1.md`
- `evidence/2026-06-04-memory-layer-m5-conflict-and-memory-lint-minimal-blocking-v1.md`
- `evidence/2026-06-04-memory-layer-m6-workflow-task-package-injection-and-end-to-end-loop-v1.md`
- `handoffs/2026-06-04-memory-layer-m6-workflow-task-package-injection-and-end-to-end-loop-v1-result.md`

当前实现重点：

- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/formal_memory_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_candidate_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/observation_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_lint_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_lint_engine.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/task_memory_packet_builder.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/task_memory_injection.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/control_core.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src/App.tsx`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `prototypes/productized-desktop-shell/src/lib/candidateGovernance.ts`
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `prototypes/productized-desktop-shell/src/components/PermissionDialog.tsx`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

## 5. 范围

允许：

- 新增后端只读 read model helper，例如 `memory_management_read_model.rs`，或在现有 store / command 中新增只读 summary 组装函数。
- 新增后端 / 前端类型：
  - `MemoryManagementSummary`
  - `FormalMemoryListItem`
  - `MemoryCandidateListItem`
  - `MemoryDetailReadModel`
  - `MemorySourceSummary`
  - `MemoryVersionSummary`
  - `MemoryAuditSummary`
  - `MemoryTaskEligibilitySummary`
  - `MemoryConflictSummary`
- 新增 Tauri 只读命令，例如 `load_memory_management_summary`；如果现有 `load_*_store` 足够，也可以只在前端组装读模型。
- 新增前端 read model helper，例如 `memoryCenter.ts`，用于从 formal memory / candidate / observation / lint / task memory packet 读模型派生 UI 文案。
- 重构现有全局 `记忆` 入口，使默认显示正式记忆、候选记忆、最近变化、冲突 / lint 摘要和任务包入选状态。
- 在记忆详情中显示来源、版本、状态、scope、权限 / 外发限制、审计摘要、冲突和任务包入选 / 排除原因。
- 在项目页保留或新增轻量项目相关记忆摘要，前提是不新增项目页 tab、不把项目画布变成记忆治理后台。
- 复用 M4/M6 的任务记忆包预览 / 注入摘要，只显示“可进入任务包 / 被排除 / 待审查材料”的人话摘要。
- 新增或调整离线 UI 测试，覆盖候选和正式记忆视觉区分、禁止文案、任务包入选状态、lint blocking 摘要。
- 新增 Rust 单测，覆盖 read model 对正式记忆 / 候选 / observation / lint / task eligibility 的分类。
- 更新 evidence / handoff / `CURRENT.md` / `tasks/README.md` / `AUTHORITY.md` / `STAGE_PLAN.md` / 阶段计划。

本任务默认不授权写入数据：

- 不写 `formal-memories.v1.json`。
- 不写 `memory-candidates.v1.json`。
- 不写 `observations.v1.json`。
- 不写 `memory-lint.v1.json`。
- 不写 `workflow-state.v0.json`。
- 不新增 `workflow-state.v0.json` 顶层数组。
- 不改 workflow / work item / node / dispatch 状态枚举。
- 不迁移数据库。

禁止：

- 不执行真实 worker。
- 不执行真实 Codex。
- 不执行 `codex exec` / `codex exec resume`。
- 不读写 `/Users/yoyi/.codex`。
- 不新增一级导航入口；复用现有 `记忆`。
- 不新增右侧顶级入口。
- 不把 `schema`、raw event、sidecar 路径、数据库路径、完整 audit JSON 或内部 id 大表放进普通 UI。
- 不显示未实现的正式记忆编辑、删除、废弃、冻结、归档、合并、拆分、上升全局、下沉项目按钮。
- 不新增 UI 直接写正式记忆的动作。
- 不让秘书确认候选、采纳记忆、编辑记忆或批准生命周期操作。
- 不把 `candidate_confirmed` 显示成“已记住”“正式记忆”或“已注入任务包”。
- 不把 observation、knowledge hit、LLM summary、task package content 显示成正式记忆。
- 不接 Obsidian 原生读写。
- 不接向量库或图数据库。
- 不做完整关系图、实体合并、成熟模式或跨项目提升。
- 不把 M7 说成中间版本记忆层完成。

如果执行者认为必须新增写入动作，必须先停止并回传，说明为什么不能留到 M9，并列出：

- 会写入哪个 store / sidecar / workflow state。
- 谁有确认权。
- 来源、版本、审计和权限如何保留。
- 为什么不属于生命周期操作。

## 5.1 UI 显示边界确认

本任务会改前端：

- [ ] 不改前端、不改读模型、不改 UI 文案。
- [x] 改前端类型 / Tauri wrapper，但不一定新增后端写命令。
- [x] 改读模型摘要或状态显示。
- [x] 改已有页面局部 UI。
- [x] 新增页面内面板或详情，但不新增一级入口 / 右侧顶级入口 / 项目页 tab。

已读取：

- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`
- `CURRENT.md`
- `tasks/README.md`
- `docs/memory-layer-design-v1.md`
- `docs/plans/memory-layer-implementation-slice-v1.md`

本任务允许显示：

- 正式记忆列表、候选记忆列表和清晰的视觉区分。
- 正式记忆详情：标题 / 摘要、scope、状态、来源、版本、审计摘要、权限 / 外发限制、冲突 / lint 摘要、任务包入选状态。
- 候选详情：候选状态、来源、风险、确认要求、是否已有采纳回链。
- observation 只作为来源或候选来源显示，文案必须说明“观察不是正式记忆”。
- 任务包入选摘要：`可进入任务包`、`被排除`、`待审查材料`、`blocking finding 阻断`。
- 最近变化摘要：最多展示少量最近版本 / 审计 / lint finding 的人话摘要。
- 项目相关记忆轻量摘要。

本任务禁止显示：

- 不新增一级入口。
- 不新增右侧顶级入口。
- 不新增项目页 tab。
- 不把记忆中心做成治理后台。
- 不显示 raw schema、raw event、完整 audit JSON、sidecar 路径、数据库路径、内部 key / id 大表。
- 不显示正式记忆生命周期按钮：编辑、删除、废弃、冻结、归档、合并、拆分、上升全局、下沉项目。
- 不显示未实现的 Obsidian 同步、向量库、图数据库、关系图编辑、成熟模式确认或跨项目自动提升。
- 不显示“已记住”“系统已长期记住”“候选已成为正式记忆”“worker 已收到记忆包”“中间版本记忆层已完成”等误导文案。

显示位置：

- 一级入口：复用现有 `记忆`，不新增。
- 右侧入口：不改。
- 项目页：只允许项目相关记忆摘要，不新增 tab，不占用工作流画布主区域。
- 画布：不改画布主区域；如涉及节点详情，只显示轻量记忆摘要。
- 记忆入口：本轮主要落地位置，显示正式记忆 / 候选 / 详情 / 来源 / 版本 / 审计摘要 / 冲突提示 / 任务包入选状态。
- 知识库入口：不改。
- 智能体入口：不改。
- 管理入口：不改。

中间版本范围：

- 本轮必须落地：记忆中心最小可理解入口、正式 / 候选视觉区分、来源 / 版本 / 审计摘要、冲突 / lint 摘要、任务包入选状态。
- 本轮只做读模型 / 摘要：项目相关记忆摘要、最近变化、任务包使用提示。
- 本轮后置：生命周期操作、Obsidian 接口、关系图编辑、维护任务、成熟模式、跨项目提升、完整知识库。

后端和数据依赖：

- 需要后端正式读模型：可以新增只读 summary 命令；也可以复用现有 stores 在前端派生，但不得用假数据。
- 需要审计 / 日志 / 权限 / 状态机：审计只能来自 `MemoryAuditEvent` 或已有 workflow audit 摘要；权限和外发状态必须来自正式字段或明确显示为“未记录”。
- 不能用假数据伪装：不能伪造来源、版本、冲突、权限、任务包入选或使用记录。

UI 文案边界：

- 禁止说：`已记住`、`系统已长期记住`、`候选已成为正式记忆`、`观察已成为正式记忆`、`知识库已同步为记忆`、`worker 已收到记忆包`、`中间版本记忆层已完成`。
- 允许说：`正式记忆`、`候选记忆`、`观察来源`、`待审查材料`、`可进入任务包`、`被 lint 阻断`、`候选已被受控采纳`、`不是正式记忆`、`未实现生命周期操作`。

验收：

- 类型检查：`npm run typecheck`
- 离线交互测试：`npm run test:offline-interaction`
- 构建：`npm run build`
- 真实窗口 / 截图验收：涉及记忆入口布局，必须做真实浏览器或 Tauri 截图验收；如果没有可用截图工具，不能声称 UI 验收完成。
- 未验收项必须写入 evidence / handoff。

## 6. 实施建议

建议按以下顺序实现：

1. 后端 / 前端读模型梳理：先确定 `FormalMemoryStore`、`MemoryCandidateStore`、`ObservationStore`、`MemoryLintStore` 和 `TaskMemoryPacket` 当前字段能支撑哪些 UI 文案。
2. 最小 read model：生成正式记忆列表、候选列表、详情、来源、版本、审计、冲突和任务包 eligibility。
3. 全局记忆入口：把现有偏 sidecar 的 `记忆` 页面改成可理解的记忆中心。
4. 项目相关摘要：只在项目页侧栏或详情里显示项目相关轻量摘要，不抢画布主区域。
5. 禁止文案和视觉区分测试：确保候选 / observation 不会显示成正式记忆。
6. 回收文档：写 evidence / handoff，并同步当前入口。

建议优先后端读模型，但如果现有 store 类型已经足够，允许先用前端纯函数派生，避免为了单一 UI 引入过大的后端抽象。

## 7. 验收

必须通过：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
cargo test --lib formal_memory
cargo test --lib memory_candidate
cargo test --lib observation
cargo test --lib memory_lint
cargo test --lib task_memory_packet
cargo test --lib
rustfmt --check src/formal_memory_store.rs src/memory_candidate_store.rs src/observation_store.rs src/memory_lint_store.rs src/memory_lint_engine.rs src/task_memory_packet_builder.rs src/task_memory_injection.rs src/control_core.rs src/commands.rs src/types.rs src/lib.rs
```

如果新增 `memory_management_read_model.rs` 或等价文件，还必须补：

```text
cargo test --lib memory_management
rustfmt --check src/memory_management_read_model.rs
```

UI / 文案必须验证：

- 正式记忆和候选记忆视觉区分。
- `candidate_confirmed` 不显示为“已记住”或“正式记忆”。
- observation 只显示为来源 / 观察，不显示为正式记忆。
- 正式记忆显示来源、版本、状态和任务包 eligibility。
- blocking lint finding 会显示为阻断任务包入选。
- 不出现禁止文案：
  - `系统已长期记住`
  - `候选已成为正式记忆`
  - `观察已成为正式记忆`
  - `worker 已收到记忆包`
  - `中间版本记忆层已完成`

## 8. evidence / handoff 要求

M7 完成后必须新增：

- `evidence/2026-06-05-memory-layer-m7-memory-management-ui-minimal-entry-v1.md`
- `handoffs/2026-06-05-memory-layer-m7-memory-management-ui-minimal-entry-v1-result.md`

evidence 必须记录：

- 实际新增 / 修改的读模型、命令、类型、UI 文件。
- 正式记忆和候选视觉区分的验收结果。
- 禁止文案搜索结果。
- 是否做了真实浏览器或 Tauri 截图验收；如果没有，必须明确写“真实窗口 / 截图验收未完成”。
- 验证命令和结果。
- 边界：未执行真实 worker / Codex，未读写 `/Users/yoyi/.codex`，未做生命周期操作，未写正式记忆。

handoff 必须写清：

- M7 接受为什么。
- M7 不接受为什么。
- 下一步应进入 M8 还是先补截图 / UI 缺口。
- 哪些 UI 文案和读模型是当前权威。

## 9. Stop 条件

遇到以下情况必须停下回传：

- 需要读写 `/Users/yoyi/.codex`。
- 需要执行真实 `codex exec` 或 `codex exec resume`。
- 需要启动真实 worker。
- 需要改 `workflow-state.v0.json` 结构或状态枚举。
- 需要写正式记忆、编辑正式记忆或变更正式记忆生命周期。
- 需要把候选、观察、知识命中或 LLM summary 直接显示 / 写成正式记忆。
- 需要接 Obsidian 原生写入、向量库或图数据库。
- 需要新增一级入口、右侧顶级入口或项目页 tab。
- 发现本任务与 `docs/workbench-frontend-display-boundary-v1.md`、`docs/memory-layer-design-v1.md` 或 `docs/plans/memory-layer-implementation-slice-v1.md` 冲突。

## 10. 回收口径

完成后接受为：

- M7 记忆管理 UI 最小入口完成。
- 正式记忆 / 候选 / observation / lint / 任务包 eligibility 的最小可理解读模型完成。
- 全局 `记忆` 入口从工程 sidecar 摘要升级为用户可理解的记忆中心。

完成后不接受为：

- 中间版本记忆系统完成。
- 正式记忆生命周期操作完成。
- M8 知识库 / Obsidian 接口完成。
- M9 编辑 / 废弃 / 冻结 / 归档 / 合并 / 拆分完成。
- M10 关系治理完成。
- M11 维护任务完成。
- M12 成熟模式 / 跨项目记忆完成。
- M13 最终验收完成。
- 真实 worker / Codex 已执行。
