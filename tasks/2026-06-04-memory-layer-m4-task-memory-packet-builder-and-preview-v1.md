# Task Package：Memory Layer M4 Task Memory Packet Builder And Preview v1

状态：已完成。  
用途：实现中间版本记忆层 M4：任务记忆包生成器和预览。  
执行方式：一个中等批次完成，最终统一验收；开发重点在后端构建器、过滤规则和排除原因，UI 只做必要预览。

## 1. 先说薄弱点

- M1 / M1.1 / M2 / M3 已完成正式记忆 store、上下文绑定 guard、候选采纳和 observation 入口。
- 当前还没有 `TaskMemoryPacketBuilder`，项目主管派 worker 前无法看到“哪些正式记忆会进入任务上下文、哪些被排除、为什么排除”。
- 如果没有 M4，后续 M6 任务包注入容易直接把 candidate、observation、知识库命中或聊天摘要混进 worker 上下文。
- M4 不是任务执行，不执行 worker，不调用真实 Codex，也不把预览结果注入任务包；M4 只生成可审查的任务记忆包预览。

## 2. 任务目标

实现任务记忆包预览链路：

```text
任务目标 / 角色 / 项目 / workflow / 模型上下文策略
-> TaskMemoryPacketBuilder
-> 读取 FormalMemoryStore
-> 按状态 / 权限 / 冲突 / 过期 / 模型外发 / token / 相关性过滤
-> 输出 included_memories 和 excluded_memories，且每条都有 reason
-> candidate / observation / knowledge hit 只能作为待审查材料，不进入正式记忆列表
-> UI / 读模型展示预览摘要，不执行 worker，不注入任务包
```

M4 完成后可以说：

- 项目主管可以在派 worker 前生成任务记忆包预览。
- active 正式记忆可以进入预览。
- 被排除的记忆、候选、observation 和知识库命中都有明确 reason。

M4 完成后仍不能说：

- 任务包注入完成。
- worker 已经收到记忆包。
- 自动化工作流已经使用记忆执行。
- 完整正式记忆生命周期完成。
- 中间版本记忆层完成。

## 3. 前置条件

必须已完成：

- M1：`tasks/2026-06-03-memory-layer-m1-formal-memory-store-and-audit-v1.md`
- M1.1：`tasks/2026-06-03-memory-layer-m1-1-formal-memory-context-binding-guard-v1.md`
- M2：`tasks/2026-06-03-memory-layer-m2-candidate-to-formal-adoption-v1.md`
- M3：`tasks/2026-06-04-memory-layer-m3-observation-store-and-workflow-entry-v1.md`

M4 开始前必须复核：

- `MemoryCandidateStore`、`ObservationStore` 和 `FormalMemoryStore` 仍分开记录。
- `candidate_confirmed` 仍不等于正式记忆。
- observation 仍不等于正式记忆。
- M3 的真实窗口 / 截图验收未完成只是 UI 验收缺口，不阻塞后端 M4，但 M4 若改 UI 必须按 UI 硬规则补验收状态。

## 4. 必须先读

当前入口：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `docs/middleware-version-development-plan-v1.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `docs/plans/memory-layer-implementation-slice-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`
- `docs/memory-layer-design-v1.md`
- `docs/plans/2026-06-01-memory-governance-schema-v1.md`
- `docs/workbench-frontend-display-boundary-v1.md`

前置记录：

- `evidence/2026-06-03-memory-layer-m1-formal-memory-store-and-audit-v1.md`
- `evidence/2026-06-03-memory-layer-m1-1-formal-memory-context-binding-guard-v1.md`
- `evidence/2026-06-03-memory-layer-m2-candidate-to-formal-adoption-v1.md`
- `evidence/2026-06-04-memory-layer-m3-observation-store-and-workflow-entry-v1.md`
- `handoffs/2026-06-04-memory-layer-m3-observation-store-and-workflow-entry-v1-result.md`

当前实现：

- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/formal_memory_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_candidate_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/observation_store.rs`
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

- 新增 Rust 后端构建器，例如 `task_memory_packet_builder.rs`。
- 新增后端类型：
  - `TaskMemoryPacketBuildInput`
  - `TaskMemoryPacketBuildOutput`
  - `TaskMemoryPacketPreview`
  - `TaskMemoryPacketItem`
  - `TaskMemoryPacketExcludedItem`
  - `TaskMemoryPacketReviewMaterial`
  - `TaskMemoryPacketExclusionReason`
- 新增后端命令：
  - `preview_task_memory_packet`
- 新增控制核心 helper：
  - `validate_task_memory_packet_preview(...)`
  - `evaluate_task_memory_packet_item(...)`
- 读取 `FormalMemoryStore`、`MemoryCandidateStore`、`ObservationStore` 生成预览。
- 第一版可以不新增持久 sidecar；推荐按当前 store 即时生成 deterministic preview。
- 在项目页、项目工作流侧栏或记忆入口显示最小预览摘要。
- 前端新增类型、Tauri wrapper 和读模型摘要。
- 新增 Rust 单测和前端离线测试。
- 更新 evidence / handoff / `CURRENT.md` / `tasks/README.md` / 阶段计划。

禁止：

- 不执行真实 worker。
- 不执行真实 Codex。
- 不执行 `codex exec` / `codex exec resume`。
- 不读写 `/Users/yoyi/.codex`。
- 不把预览结果注入任务包；那是 M6。
- 不把 candidate、observation、knowledge hit、LLM summary 当正式记忆进入 included list。
- 不修改正式记忆。
- 不采纳候选为正式记忆。
- 不改变 observation 状态。
- 不改 `workflow-state.v0.json` 结构。
- 不迁移数据库。
- 不接 Obsidian 原生读写。
- 不接向量库或图数据库。
- 不扫描完整 transcript。
- 不用假数据伪装后端能力已完成。

## 5.1 UI 显示边界确认

本任务会改前端：

- [ ] 不改前端、不改读模型、不改 UI 文案。
- [x] 改前端类型 / Tauri wrapper，但不新增可见 UI。
- [x] 改读模型摘要或状态显示。
- [x] 改已有页面局部 UI。
- [ ] 新增一级入口、右侧顶级入口或项目页 tab。

已读取：

- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`
- `CURRENT.md`
- `tasks/README.md`

本任务允许显示：

- “任务记忆包预览”最小只读摘要。
- included 正式记忆数量。
- excluded 记忆数量和排除 reason。
- 待审查材料数量，例如 candidate / observation / knowledge hit，但必须标注“不进入正式记忆列表”。
- token 预算和估算 token 用量。
- warnings，例如 `preview_only_not_injected`。

本任务禁止显示：

- 不新增一级入口。
- 不新增右侧顶级入口。
- 不把任务记忆包预览铺进项目工作流画布主区域。
- 不把 candidate / observation / knowledge hit 写成“已记住”“正式事实”“已注入任务包”。
- 不显示 raw event、schema、数据库路径大表或完整审计日志。
- 不显示未实现的自动执行按钮。
- 不显示“worker 已收到记忆包”或“任务包注入已完成”。

显示位置：

- 一级入口：不改。
- 右侧入口：不改。
- 项目页：允许在项目工作流侧栏、节点详情或项目记忆相关区域显示最小预览摘要。
- 画布：不在画布主区域新增任务记忆包面板。
- 记忆入口：允许显示预览摘要和排除原因，不做完整记忆管理页面。
- 知识库入口：不改。
- 智能体入口：不改。
- 管理入口：不改。

中间版本范围：

- 本轮必须落地：后端 `TaskMemoryPacketBuilder`、过滤规则、排除 reason 和最小预览读模型。
- 本轮只做读模型 / 摘要：预览展示 included / excluded / review materials / warnings。
- 本轮后置：真实任务包注入、worker 执行、完整记忆中心、图关系编辑、Obsidian / 知识库联动。

后端和数据依赖：

- 预览必须来自 `FormalMemoryStore`、`MemoryCandidateStore`、`ObservationStore` 和后端构建器。
- included list 只能来自 active 正式记忆。
- candidate / observation / knowledge hit 只能进入待审查材料或 excluded list。
- 不允许前端用 mock 数据伪装记忆包预览已经由后端生成。

UI 文案边界：

- 禁止说：“系统已记住”“自动学习完成”“候选已进入任务包”“observation 已注入任务包”“worker 已收到记忆包”“任务包注入已完成”。
- 允许说：“任务记忆包预览”“仅 active 正式记忆可入选”“候选 / 观察仅作为待审查材料”“预览未注入任务包”。

验收：

- 必须跑 `npm run typecheck`。
- 必须跑 `npm run test:offline-interaction`。
- 必须跑 `npm run build`。
- 如改变项目页布局，必须做真实窗口或浏览器截图验收；如果当前对话没有可用截图工具，必须在 evidence / handoff 写明“真实窗口 / 截图验收未完成”。

## 6. 建议数据对象

第一版推荐不新增持久 sidecar，按 store 现场生成预览：

```ts
type TaskMemoryPacketBuildInput = {
  project_root: string;
  project_id?: string;
  workflow_id?: string;
  task_id?: string;
  role_id: string;
  task_goal: string;
  retrieval_intent:
    | "worker_task"
    | "project_director_review"
    | "global_director_review"
    | "result_acceptance";
  target_model_id?: string;
  model_context_policy: "local_only" | "external_model_context";
  max_memory_items: number;
  max_estimated_tokens: number;
  expected_formal_store_revision?: number;
  expected_candidate_store_revision?: number;
  expected_observation_store_revision?: number;
};
```

```ts
type TaskMemoryPacketExclusionReason =
  | "candidate_unconfirmed"
  | "permission_blocked"
  | "conflicted"
  | "stale"
  | "model_export_blocked"
  | "token_limit"
  | "not_relevant"
  | "status_not_active"
  | "observation_not_formal_memory"
  | "knowledge_hit_not_formal_memory"
  | "llm_summary_not_formal_memory";
```

```ts
type TaskMemoryPacketItem = {
  memory_id: string;
  memory_type: string;
  scope_type: string;
  claim: string;
  body: string;
  source_refs: MemorySourceRef[];
  retrieval_reason: string;
  estimated_tokens: number;
  model_export_policy: string;
};
```

```ts
type TaskMemoryPacketExcludedItem = {
  source_kind:
    | "memory_record"
    | "memory_candidate"
    | "observation"
    | "knowledge_hit"
    | "llm_summary";
  source_id: string;
  claim?: string;
  reason: TaskMemoryPacketExclusionReason;
  detail: string;
};
```

```ts
type TaskMemoryPacketReviewMaterial = {
  source_kind: "memory_candidate" | "observation" | "knowledge_hit";
  source_id: string;
  title: string;
  reason:
    | "candidate_unconfirmed"
    | "observation_not_formal_memory"
    | "knowledge_hit_not_formal_memory";
};
```

```ts
type TaskMemoryPacketPreview = {
  packet_id: string;
  schema_version: "task_memory_packet.v1";
  project_id?: string;
  workflow_id?: string;
  task_id?: string;
  role_id: string;
  retrieval_intent: string;
  included_memories: TaskMemoryPacketItem[];
  excluded_items: TaskMemoryPacketExcludedItem[];
  review_materials: TaskMemoryPacketReviewMaterial[];
  estimated_tokens: number;
  max_estimated_tokens: number;
  generated_at: string;
  warnings: string[];
};
```

```ts
type TaskMemoryPacketBuildOutput = {
  preview: TaskMemoryPacketPreview;
  formal_store_revision: number;
  candidate_store_revision: number;
  observation_store_revision: number;
  warnings: string[];
};
```

## 7. 过滤规则

正式记忆入选必须满足：

- 来源是 `FormalMemoryStore.records[]`。
- `status == memory_active`。
- scope 与 `project_root` / `project_id` / `workflow_id` / `task_id` / `role_id` 匹配。
- 没有 blocking conflict。
- 未过期或未被 deprecated / frozen / archived。
- 当前 actor / role / model 有权限看到。
- 当前 `model_context_policy` 允许导出；`model_export_policy = blocked` 时不能进入 external model context。
- 未超过 `max_memory_items` 和 `max_estimated_tokens`。
- 与 `task_goal` 或 `retrieval_intent` 有确定性相关性。

必须排除并给 reason：

- 记忆候选：`candidate_unconfirmed`。
- observation：`observation_not_formal_memory`。
- 知识库命中：`knowledge_hit_not_formal_memory`。
- LLM 摘要：`llm_summary_not_formal_memory`。
- `memory_conflicted` 或有 blocking conflict：`conflicted`。
- `memory_deprecated` / `memory_frozen` / `memory_archived`：`stale` 或 `status_not_active`。
- 权限不足：`permission_blocked`。
- 模型外发受阻：`model_export_blocked`。
- token 超限：`token_limit`。
- 与任务目标无关：`not_relevant`。

第一版相关性可以使用确定性规则，不引入 LLM：

- project / workflow / session scope 匹配优先。
- `task_goal` 命中 claim / body 关键词时入选。
- 同 project 但无关键词命中可进入 excluded，reason 为 `not_relevant`。
- 不做向量召回。
- 不做图遍历。

## 8. 控制核心规则

新增 helper 建议放在 `control_core.rs`：

- `validate_task_memory_packet_preview(...)`
- `validate_task_memory_packet_actor_boundary(...)`
- `evaluate_task_memory_packet_export_policy(...)`

必须校验：

- `project_root` 非空。
- `task_goal` 非空。
- `role_id` 非空。
- `retrieval_intent` 白名单。
- `model_context_policy` 白名单。
- `max_memory_items` 必须大于 0 且小于等于合理上限，建议第一版上限 20。
- `max_estimated_tokens` 必须大于 0 且小于等于合理上限，建议第一版上限 8000。
- project / workflow / session scope 与 `project_root` 绑定一致。
- blocked export 不进入 external model context。
- 预览命令不得写入 formal / candidate / observation store。

## 9. 测试要求

Rust 必须覆盖：

1. `task_memory_packet_includes_active_formal_memory`
   - active 正式记忆进入 included list。

2. `task_memory_packet_excludes_candidates_as_unconfirmed`
   - memory candidate 不进入 included list，只进入 review materials 或 excluded，reason 为 `candidate_unconfirmed`。

3. `task_memory_packet_excludes_observation_as_not_formal`
   - observation 不进入 included list，reason 为 `observation_not_formal_memory`。

4. `task_memory_packet_excludes_inactive_formal_memories`
   - conflicted / deprecated / frozen / archived 正式记忆被排除，且有 reason。

5. `task_memory_packet_excludes_model_export_blocked`
   - `model_export_policy = blocked` 的正式记忆不能进入 external model context。

6. `task_memory_packet_excludes_permission_blocked`
   - 跨项目、跨用户或权限不足的记忆被排除。

7. `task_memory_packet_excludes_token_limit`
   - 超出 token 预算的记忆被排除，reason 为 `token_limit`。

8. `task_memory_packet_excludes_not_relevant`
   - 与任务目标不相关的记忆被排除，reason 为 `not_relevant`。

9. `task_memory_packet_preview_is_readonly`
   - 生成预览不修改 formal / candidate / observation store revision。

10. `task_memory_packet_preview_does_not_execute_worker`
   - 预览不创建 dispatch、attempt、workflow state 写入或真实 Codex 调用。

前端离线测试必须覆盖：

- 预览摘要能显示 included / excluded / review materials 计数。
- 每个 excluded item 显示 reason。
- candidate / observation 显示为待审查材料，不显示为正式记忆。
- UI 文案不出现“已注入任务包”“worker 已收到记忆包”“系统已记住”。

## 10. 验证命令

至少运行：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
cargo test --lib task_memory_packet
cargo test --lib formal_memory
cargo test --lib memory_candidate
cargo test --lib observation
cargo test --lib
rustfmt --check src/task_memory_packet_builder.rs src/formal_memory_store.rs src/memory_candidate_store.rs src/observation_store.rs src/control_core.rs src/commands.rs src/types.rs
```

如果实现没有改可见 UI，可说明不需要截图；如果改项目页、记忆入口或右侧面板布局，必须做真实窗口 / 浏览器截图验收，或者在 evidence / handoff 明确写“真实窗口 / 截图验收未完成”。

## 11. 验收标准

接受为完成：

- 新增 `TaskMemoryPacketBuilder` 或等价后端构建器。
- 可以生成任务记忆包预览。
- active 正式记忆能进入 included list。
- candidate、observation、knowledge hit、LLM summary 不能进入正式 included list。
- conflicted / deprecated / frozen / archived 记忆被排除。
- 每条 included memory 有 retrieval reason。
- 每条 excluded item 有 exclusion reason。
- 预览不写 store，不改 workflow state，不执行 worker。
- UI 如果展示预览，必须按 UI 显示边界执行，并记录真实窗口 / 截图验收状态。

不接受为完成：

- 任务包注入完成。
- worker 执行完成。
- 自动化工作流完成。
- 完整记忆生命周期完成。
- Obsidian / 知识库 / 向量库 / 图数据库完成。
- 中间版本记忆层完成。

## 12. 回收记录要求

完成实现后新增：

- `evidence/2026-06-04-memory-layer-m4-task-memory-packet-builder-and-preview-v1.md`
- `handoffs/2026-06-04-memory-layer-m4-task-memory-packet-builder-and-preview-v1-result.md`

并更新：

- `CURRENT.md`
- `tasks/README.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `AUTHORITY.md`

回收结论必须明确：

- M4 只接受为任务记忆包生成器和预览完成。
- 不接受为任务包注入完成。
- 不接受为 worker 已收到记忆包。
- 不接受为中间版本记忆层完成。

## 13. 回收结果

完成记录：

- `evidence/2026-06-04-memory-layer-m4-task-memory-packet-builder-and-preview-v1.md`
- `handoffs/2026-06-04-memory-layer-m4-task-memory-packet-builder-and-preview-v1-result.md`

接受为：

- `TaskMemoryPacketBuilder` / `preview_task_memory_packet` 已完成。
- active 正式记忆可以进入任务记忆包预览 included list。
- candidate / observation 只进入 excluded / review materials，并带 `candidate_unconfirmed` / `observation_not_formal_memory` reason。
- conflicted / stale / permission / model export / token / relevance 等过滤规则已有后端测试覆盖。
- 项目工作流侧栏已有最小只读预览摘要。

仍不接受为：

- 任务包注入完成。
- worker 已收到记忆包。
- 记忆召回已经参与真实 worker 执行。
- 中间版本记忆层完成。
