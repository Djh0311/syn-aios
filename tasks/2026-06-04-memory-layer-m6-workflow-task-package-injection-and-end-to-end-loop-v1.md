# Task Package：Memory Layer M6 Workflow Task Package Injection And End To End Loop v1

状态：已完成。  
用途：实现中间版本记忆层 M6：工作流任务包注入和端到端闭环。  
执行方式：一个中等偏大的批次完成，最终统一验收；开发重点在后端任务包生成流程、记忆包快照保存、派发准备态携带记忆块和审计，UI 只做必要只读状态 / 摘要。

完成记录：

- `evidence/2026-06-04-memory-layer-m6-workflow-task-package-injection-and-end-to-end-loop-v1.md`
- `handoffs/2026-06-04-memory-layer-m6-workflow-task-package-injection-and-end-to-end-loop-v1-result.md`

## 1. 先说薄弱点

- M1 / M1.1 / M2 / M3 / M4 / M5 已完成正式记忆 store、上下文绑定、候选采纳、observation 入口、任务记忆包预览和 lint blocking。
- 当前 `TaskMemoryPacketBuilder` 仍主要是 preview：能解释 included / excluded / review materials，但还没有成为工作流任务包生成流程的一部分。
- 当前 `TaskPackage` / `generate_task_package_file` / `prepare_offline_role_dispatch` 已存在，但任务包文件和派发预览还没有保存一份“冻结的任务记忆包快照”。
- 如果没有 M6，worker B 的任务包仍不能可靠携带“worker A 已确认并采纳的正式记忆”，第一条真实记忆闭环无法证明。
- M6 是 M1-M6 第一条真实记忆闭环的收口，但不是中间版本记忆层完成；M7-M13 仍要补正式记忆生命周期、关系治理、维护任务、成熟模式和完整验收。

## 2. 任务目标

把 M4/M5 产出的 `TaskMemoryPacket` 接入工作流任务包生成链路：

```text
worker A 汇报
-> 项目主管确认过程事实
-> ObservationStore
-> MemoryCandidate
-> 受控采纳为 FormalMemory
-> TaskMemoryPacketBuilder + MemoryLint guard
-> 生成 TaskPackageMemoryPacketSnapshot
-> 写入 task_package artifact / 任务包 markdown / prepared dispatch prompt
-> worker B 任务包能看到正式记忆 claim、来源、入选理由和禁止事项
-> worker B 后续汇报仍进入 workflow ledger / observation，不自动成为记忆
```

M6 完成后可以说：

- 第一条记忆闭环可在工作流任务包生成流程里跑通。
- active 正式记忆能进入工作流任务包的冻结记忆快照。
- candidate / observation / knowledge hit / LLM summary 仍不能进入 included list。
- task package artifact / 任务包文件 / prepared dispatch prompt 可以显示 included / excluded / review materials 和 warnings。
- 任务包生成有审计，并能判断记忆快照是否 stale。

M6 完成后仍不能说：

- 中间版本记忆层完成。
- 完整正式记忆生命周期完成。
- 维护任务系统完成。
- worker 已真实执行任务，除非本轮另有用户明确授权真实 `codex exec`。
- 自动化工作流产品化闭环完成。
- 秘书可以确认事实、派发任务或写正式记忆。

## 3. 前置条件

必须已完成：

- `tasks/2026-06-03-memory-layer-m1-formal-memory-store-and-audit-v1.md`
- `tasks/2026-06-03-memory-layer-m1-1-formal-memory-context-binding-guard-v1.md`
- `tasks/2026-06-03-memory-layer-m2-candidate-to-formal-adoption-v1.md`
- `tasks/2026-06-04-memory-layer-m3-observation-store-and-workflow-entry-v1.md`
- `tasks/2026-06-04-memory-layer-m4-task-memory-packet-builder-and-preview-v1.md`
- `tasks/2026-06-04-memory-layer-m5-conflict-and-memory-lint-minimal-blocking-v1.md`

M6 开始前必须复核：

- `TaskMemoryPacketBuilder` 已排除 non-active、conflicted、stale、permission blocked、model export blocked、token limit 和 not relevant。
- M5 的 open blocking lint finding 会让任务记忆包预览排除对应正式记忆。
- `generate_task_package_file_at` 当前会生成 markdown 文件，并写 `task_package_file_generated` audit。
- `inspect_task_package_dispatch_readiness_at` 当前会检查 task package artifact、stale、missing fields 等 dispatch readiness。
- `prepare_workflow_node_dispatch_at` / `prepare_offline_role_dispatch_at` 当前只是准备 / 记录派发；真实执行路径是 `execute_workflow_node_dispatch_at`，本任务默认不调用。

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
- `docs/workflow-task-package-design-v1.md`
- `docs/plans/2026-06-01-project-workflow-canvas-node-schema-v1.md`
- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`

前置记录：

- `evidence/2026-06-03-memory-layer-m1-formal-memory-store-and-audit-v1.md`
- `evidence/2026-06-03-memory-layer-m1-1-formal-memory-context-binding-guard-v1.md`
- `evidence/2026-06-03-memory-layer-m2-candidate-to-formal-adoption-v1.md`
- `evidence/2026-06-04-memory-layer-m3-observation-store-and-workflow-entry-v1.md`
- `evidence/2026-06-04-memory-layer-m4-task-memory-packet-builder-and-preview-v1.md`
- `evidence/2026-06-04-memory-layer-m5-conflict-and-memory-lint-minimal-blocking-v1.md`
- `handoffs/2026-06-04-memory-layer-m5-conflict-and-memory-lint-minimal-blocking-v1-result.md`

当前实现：

- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/formal_memory_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_candidate_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/observation_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_lint_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_lint_engine.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/task_memory_packet_builder.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/control_core.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `prototypes/productized-desktop-shell/src/lib/candidateGovernance.ts`
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

## 5. 范围

允许：

- 新增后端 task memory injection helper，例如 `task_memory_injection.rs`。
- 新增后端类型：
  - `TaskPackageMemoryPacketSnapshot`
  - `TaskPackageMemoryPacketStoreRevisions`
  - `TaskPackageMemoryInjectionAudit`
  - `TaskPackageMemoryInjectionSummary`
  - `TaskPackageMemoryInjectionInput`
  - `TaskPackageMemoryInjectionOutput`
- 扩展 `TaskPackage` / `TaskPackagePreview` / `TaskPackageFileGenerationResult` / `TaskPackageDispatchReadiness` 的类型，使其能表达记忆快照状态。
- 扩展 `TaskPackage` artifact 内部字段，保存冻结的 `memory_packet_snapshot`、`memory_packet_fingerprint`、store revisions、stale 状态和 warnings。
- 扩展 generated task package markdown，加入“正式记忆上下文”小节。
- 扩展 prepared dispatch prompt，使离线 / prepared 派发携带同一份冻结记忆块。
- 扩展 dispatch readiness：当 task package 需要记忆但没有 fresh snapshot，或 store revision 已变化时，给出 blocking / stale reason。
- 写工作台自己的 workflow audit，例如 `task_memory_packet_injected_into_task_package`。
- 新增 / 调整 Tauri wrapper、前端类型和只读摘要 helper。
- 在项目工作流侧栏 / task package 详情显示最小“任务包记忆注入摘要”。
- 新增 Rust 单测和前端离线测试，覆盖 M6 端到端场景。
- 更新 evidence / handoff / `CURRENT.md` / `tasks/README.md` / 阶段计划。

本任务显式授权的 workflow-state v0 变更：

- 允许给现有 `task_package` artifact 添加嵌套字段：
  - `memory_packet_snapshot`
  - `memory_packet_fingerprint`
  - `memory_packet_generated_at`
  - `memory_packet_store_revisions`
  - `memory_packet_stale`
  - `memory_packet_warnings`
- 允许给 prepared `workflow_node_dispatches[]` 记录添加只读引用或 prompt snapshot 字段，前提是字段只引用本次 task package artifact / memory packet snapshot。
- 不允许新增顶层数组。
- 不允许改现有 workflow / work item / node 状态枚举。
- 不允许迁移数据库。

禁止：

- 不执行真实 worker。
- 不执行真实 Codex。
- 不执行 `codex exec` / `codex exec resume`。
- 不读写 `/Users/yoyi/.codex`。
- 不让 worker 扫完整记忆库。
- 不把任务包内容回灌成正式记忆。
- 不把 worker 汇报自动写正式记忆；任务结束后仍必须走 ledger / observation / candidate / controlled adoption。
- 不把 candidate、observation、knowledge hit、LLM summary 放进 included list。
- 不绕过 M5 lint blocking。
- 不接 Obsidian 原生读写。
- 不接向量库或图数据库。
- 不扫描完整 transcript。
- 不把 M6 说成中间版本记忆层完成。

如果执行者认为必须做真实 `codex exec` / `codex exec resume` 端到端验收，必须先停止并向用户申请明确授权，写清：

- 目标项目路径。
- 目标 Codex thread / session。
- 会写入哪些文件。
- 是否会写 `/Users/yoyi/.codex`。
- 备份和回滚方案。
- 超时、取消和失败处理。

## 5.1 UI 显示边界确认

本任务会改前端：

- [ ] 不改前端、不改读模型、不改 UI 文案。
- [x] 改前端类型 / Tauri wrapper，但不新增可见 UI。
- [x] 改读模型摘要或状态显示。
- [x] 改已有页面局部 UI。
- [x] 新增或调整已有任务包生成 / 预览 / 派发准备的局部按钮或确认动作。

已读取：

- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`
- `CURRENT.md`
- `tasks/README.md`

本任务允许显示：

- “任务包记忆注入摘要”最小只读摘要。
- included 正式记忆数量、excluded 数量、review materials 数量。
- task package memory packet stale / fresh 状态。
- store revision / fingerprint 摘要。
- generated task package 是否包含正式记忆上下文。
- warnings，例如 `task_memory_packet_snapshot_stale`、`candidate_and_observation_review_materials_only`。
- 文案明确：`仅 active 正式记忆可进入任务包`、`候选 / 观察仅作为待审查材料`、`任务包内容不会回灌成正式记忆`。

本任务禁止显示：

- 不新增一级入口。
- 不新增右侧顶级入口。
- 不新增项目页 tab。
- 不把任务包正文、included/excluded 大表或 raw snapshot 铺进项目工作流画布主区域。
- 不把项目页变成任务包管理器。
- 不显示完整 sidecar JSON、raw event、数据库路径大表或完整审计日志。
- 不显示未实现的“一键执行真实 worker”“自动写记忆”“自动完成闭环”按钮。
- 不显示“worker 已收到记忆包”“真实 worker 已执行”“系统已自动长期记住”“中间版本记忆层已完成”，除非确有对应真实授权与 evidence。

显示位置：

- 一级入口：不改。
- 右侧入口：不改。
- 项目页：允许在项目工作流侧栏、task package 详情、节点详情或候选治理 / 记忆摘要区域显示最小只读摘要。
- 画布：不在画布主区域新增任务包记忆注入面板。
- 记忆入口：不做完整记忆管理页面；可显示项目相关摘要但非本轮重点。
- 知识库入口：不改。
- 智能体入口：不改。
- 管理入口：不改。

中间版本范围：

- 本轮必须落地：后端任务包记忆快照、markdown / prepared dispatch 记忆块、readiness stale 检查和审计。
- 本轮只做读模型 / 摘要：included / excluded / review materials 计数、stale 状态、warnings、audit id。
- 本轮后置：完整记忆中心、完整任务包管理页、真实自动化工作流产品化、真实 worker 派发验收、正式记忆生命周期 UI。

后端和数据依赖：

- 任务包记忆快照必须来自 `TaskMemoryPacketBuilder`。
- blocking 排除必须继承 M5 `MemoryLintStore` 结果。
- 任务包 generated markdown / prepared dispatch prompt 必须使用同一份冻结 snapshot。
- 前端不能 mock 记忆注入成功。
- 如果 snapshot stale，UI / readiness 必须显示原因。

UI 文案边界：

- 禁止说：“worker 已收到记忆包”“真实 worker 已执行”“系统已自动记住”“任务包内容已写入正式记忆”“中间版本记忆层完成”。
- 允许说：“任务包记忆注入摘要”“正式记忆上下文已写入任务包快照”“候选 / 观察仅作为待审查材料”“任务包内容不会回灌成正式记忆”“未执行真实 worker”。

验收：

- 必须跑 `npm run typecheck`。
- 必须跑 `npm run test:offline-interaction`。
- 必须跑 `npm run build`。
- 如改变项目页布局、按钮或任务包详情，必须做真实窗口或浏览器截图验收；如果当前对话没有可用截图工具，必须在 evidence / handoff 写明“真实窗口 / 截图验收未完成”。

## 6. 建议数据对象

```ts
type TaskPackageMemoryPacketStoreRevisions = {
  formal_store_revision: number;
  candidate_store_revision: number;
  observation_store_revision: number;
  lint_store_revision?: number;
};
```

```ts
type TaskPackageMemoryPacketSnapshot = {
  snapshot_id: string;
  schema_version: "task_package_memory_packet_snapshot.v1";
  source_packet_id: string;
  project_id?: string;
  workflow_id?: string;
  work_item_id: string;
  task_package_artifact_id?: string;
  role_id: string;
  retrieval_intent: "worker_task";
  included_memories: TaskMemoryPacketItem[];
  excluded_items: TaskMemoryPacketExcludedItem[];
  review_materials: TaskMemoryPacketReviewMaterial[];
  store_revisions: TaskPackageMemoryPacketStoreRevisions;
  estimated_tokens: number;
  max_estimated_tokens: number;
  fingerprint: string;
  generated_at: string;
  stale: boolean;
  stale_reasons: string[];
  warnings: string[];
};
```

```ts
type TaskPackageMemoryInjectionSummary = {
  snapshot_id?: string | null;
  included_count: number;
  excluded_count: number;
  review_material_count: number;
  stale: boolean;
  stale_reasons: string[];
  display_text: string;
  warnings: string[];
};
```

第一版可以不新增独立 sidecar。推荐把 snapshot 作为 task package artifact 的冻结字段保存，并在 generated markdown 中渲染同一份 snapshot。后续如果任务包历史增长过大，再单开 sidecar / SQLite 迁移任务。

## 7. 后端实现要求

任务包生成：

- `generate_task_package_file_at` 生成 markdown 前必须调用 `TaskMemoryPacketBuilder`。
- 构造 `TaskMemoryPacketBuildInput` 时使用当前 work item / assigned role / task goal / project root / workflow id。
- `retrieval_intent` 必须为 `worker_task`。
- `model_context_policy` 必须来自 task package / model policy；第一版没有明确模型时使用 `local_only`，不能默认外发。
- 生成 `TaskPackageMemoryPacketSnapshot` 后写入 task package artifact。
- markdown 必须包含“正式记忆上下文”小节。

markdown 必须包含：

- included 正式记忆 claim。
- 来源摘要，例如 source type / source id / source title。
- 入选理由 `retrieval_reason`。
- 禁止事项：不得把任务包内容回灌成正式记忆；不得把候选 / observation 当正式事实。
- excluded summary：按 reason 汇总，不默认展开完整大表。
- review materials summary：candidate / observation 只作为待审查材料。
- warnings。

派发准备：

- `prepare_offline_role_dispatch_at` 的 prompt / raw block 必须能携带同一份任务包记忆块。
- `prepare_workflow_node_dispatch_at` 的 prompt preview 必须能携带同一份任务包记忆块。
- `execute_workflow_node_dispatch_at` 不在本轮调用；若现有 UI 有执行按钮，本轮不得自动触发。

readiness：

- `inspect_task_package_dispatch_readiness_at` 必须检查是否存在 fresh memory packet snapshot。
- 若 `memory_packet_stale = true`，必须返回 blocking reason 或 warning；具体 blocking 规则由控制核心 helper 决定。
- 若 formal / candidate / observation / lint store revision 变化，必须标记 stale。
- 若 included 为空但任务包声明 `requires_memory_refs = true`，必须 blocking。
- 如果没有正式记忆但任务不要求记忆，可以 warning，不阻断。

审计：

- 写 workflow audit event `task_memory_packet_injected_into_task_package`。
- audit reason 必须包含 work_item_id、snapshot_id、included_count、excluded_count。
- 不写正式记忆 audit，除非正式记忆 store 发生真实变更；M6 默认不变更正式记忆。

## 8. 端到端验收场景

必须用临时 workflow state / sidecar fixture 跑通：

```text
worker A 汇报接口完成
-> create_observation 写 recorded observation
-> create_memory_candidate_from_observation 生成 candidate_needs_review
-> record_memory_candidate_decision 或等价流程确认 candidate_confirmed
-> adopt_memory_candidate_to_formal_memory 受控采纳为 memory_active
-> generate_task_package_file 或 M6 新命令生成 worker B 任务包
-> 任务包 memory snapshot included 包含该正式记忆
-> 任务包 markdown 包含该正式记忆 claim、来源和入选理由
-> dispatch readiness 为 ready 或带明确非阻塞 warning
-> prepare_offline_role_dispatch / prepare_workflow_node_dispatch 的 prompt preview 含同一记忆块
-> worker B 后续汇报仍只能进入 ledger / observation，不自动写正式记忆
```

必须覆盖反例：

- candidate 不进入 included list。
- observation 不进入 included list。
- open blocking lint finding 命中的正式记忆不进入 included list。
- formal store revision 变化后旧 snapshot stale。
- 任务包内容不会生成新的 formal memory。
- 未授权时不会执行真实 `codex exec` / `codex exec resume`。

## 9. 验收标准

后端必须验证：

- active 正式记忆被写入 task package memory snapshot included list。
- excluded / review materials 被保存到 snapshot，并在 markdown 以摘要形式出现。
- `TaskPackageMemoryPacketSnapshot.fingerprint` 对同一输入 deterministic。
- store revision 变化会让 snapshot stale。
- generated markdown 和 prepared dispatch prompt 使用同一 snapshot。
- `inspect_task_package_dispatch_readiness` 能识别 missing / stale memory snapshot。
- M5 blocking lint finding 会阻止对应正式记忆进入任务包。
- worker B 汇报后不会自动写正式记忆。
- 不调用真实 Codex runner。

前端必须验证：

- 任务包记忆注入摘要显示 included / excluded / review materials / stale / warnings。
- UI 文案包含“任务包内容不会回灌成正式记忆”或同等边界。
- UI 文案不包含“worker 已收到记忆包”“真实 worker 已执行”“系统已自动记住”“中间版本记忆层完成”等越界说法。
- 项目工作流画布主区域不显示任务包记忆大表。

建议验证命令：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
cargo test --lib task_memory_injection
cargo test --lib task_memory_packet
cargo test --lib memory_lint
cargo test --lib memory_candidate_adoption
cargo test --lib observation
cargo test --lib formal_memory
cargo test --lib
rustfmt --check src/task_memory_injection.rs src/task_memory_packet_builder.rs src/memory_lint_store.rs src/memory_lint_engine.rs src/formal_memory_store.rs src/memory_candidate_store.rs src/observation_store.rs src/control_core.rs src/commands.rs src/types.rs
```

## 10. 回收要求

执行完成后必须新增：

- `evidence/2026-06-04-memory-layer-m6-workflow-task-package-injection-and-end-to-end-loop-v1.md`
- `handoffs/2026-06-04-memory-layer-m6-workflow-task-package-injection-and-end-to-end-loop-v1-result.md`

并同步：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `docs/plans/memory-layer-implementation-slice-v1.md`

回收结论必须明确：

- 接受为 M6 工作流任务包注入和第一条端到端记忆闭环完成。
- 不接受为中间版本记忆层完成。
- 不接受为完整正式记忆生命周期完成。
- 不接受为完整维护任务系统完成。
- 不接受为真实 worker 已执行，除非本轮另有用户授权和 evidence。
- 不接受为自动化工作流产品化闭环完成。
