# Task Package：Memory Layer M3 Observation Store And Workflow Entry v1

状态：已完成。  
用途：实现中间版本记忆层 M3：ObservationStore 和工作流观察入口。  
执行方式：一个中等批次完成，最终统一验收；不要把本任务扩大成任务包记忆注入或完整记忆中心。

完成记录：

- Evidence：`evidence/2026-06-04-memory-layer-m3-observation-store-and-workflow-entry-v1.md`
- Handoff：`handoffs/2026-06-04-memory-layer-m3-observation-store-and-workflow-entry-v1-result.md`
- 验收结论：接受为 ObservationStore 和工作流观察入口完成；不接受为正式记忆生命周期完成、任务包记忆注入完成或中间版本记忆层完成。

## 1. 先说薄弱点

- M1 / M1.1 / M2 已完成正式记忆 store、上下文绑定 guard、候选到正式记忆受控采纳。
- 当前仍缺少 observation 层；worker 汇报、项目主管确认、全局主管复核、方案采纳和结果验收还不能进入受控观察账本。
- 没有 ObservationStore 时，后续记忆候选生成容易直接依赖聊天摘要、handoff 文本或人工记忆，来源边界不够稳。
- M3 不是“自动记住”。observation 只是观察记录，可以生成记忆候选，但不能直接成为正式记忆，也不能直接注入 worker 任务包。

## 2. 任务目标

实现受控观察链路：

```text
明确工作流事件 / worker 汇报 / 主管确认 / 方案采纳 / 结果验收
-> ObservationStore 写入 ObservationRecord
-> observation 保留 source refs、摘要、状态和审计引用
-> 项目主管确认过程事实后，从 observation 生成 MemoryCandidate
-> MemoryCandidateStore 写入 candidate_needs_review
-> observation 状态变为 candidate_created 并记录 candidate link
```

M3 完成后可以说：

- 工作流观察可以受控记录到 observation sidecar。
- observation 可以在确认后生成记忆候选。
- 被隔离或忽略的 observation 不会生成候选。

M3 完成后仍不能说：

- observation 已经是正式记忆。
- 任务包召回完成。
- 任务包注入完成。
- 完整正式记忆生命周期完成。
- 中间版本记忆层完成。

## 3. 前置条件

必须已完成：

- M1：`tasks/2026-06-03-memory-layer-m1-formal-memory-store-and-audit-v1.md`
- M1.1：`tasks/2026-06-03-memory-layer-m1-1-formal-memory-context-binding-guard-v1.md`
- M2：`tasks/2026-06-03-memory-layer-m2-candidate-to-formal-adoption-v1.md`

M3 开始前必须复核：

- M2 没有把 `candidate_confirmed` 自动当正式记忆。
- `MemoryCandidateStore` 和 `FormalMemoryStore` 仍分开记录。
- `project_root` / `project_id` / `workflow_id` 的上下文绑定 guard 仍有效。

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

前置任务：

- `tasks/2026-06-03-memory-layer-m1-formal-memory-store-and-audit-v1.md`
- `tasks/2026-06-03-memory-layer-m1-1-formal-memory-context-binding-guard-v1.md`
- `tasks/2026-06-03-memory-layer-m2-candidate-to-formal-adoption-v1.md`
- `evidence/2026-06-03-memory-layer-m2-candidate-to-formal-adoption-v1.md`
- `handoffs/2026-06-03-memory-layer-m2-candidate-to-formal-adoption-v1-result.md`

当前实现：

- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_candidate_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/formal_memory_store.rs`
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

- 新增 Rust 后端 observation store，例如 `observation_store.rs`。
- 新增独立 sidecar：`observations.v1.json`。
- 新增类型：
  - `ObservationStoreV1`
  - `ObservationRecord`
  - `ObservationSourceRef`
  - `ObservationAuditRef`
  - `CreateObservationInput`
  - `CreateObservationOutput`
  - `CreateMemoryCandidateFromObservationInput`
  - `CreateMemoryCandidateFromObservationOutput`
- 新增后端命令：
  - `load_observation_store`
  - `create_observation`
  - `create_memory_candidate_from_observation`
- 新增控制核心 helper：
  - `validate_observation_create(...)`
  - `validate_observation_candidate_creation(...)`
- 复用现有 `MemoryCandidateStore` 创建候选。
- 生成候选后，把 observation 状态更新为 `candidate_created`，并记录 `candidate_key` / audit ref。
- 在项目页或记忆入口新增最小只读 observation 摘要和候选 link。
- 新增 Rust 单测和前端离线测试。
- 更新 evidence / handoff / `CURRENT.md` / `tasks/README.md` / 阶段计划。

禁止：

- 不把 observation 直接写成正式记忆。
- 不从 observation 直接调用 M2 采纳正式记忆。
- 不把 observation 注入 worker 任务包；那是 M4 / M6 之后的方向。
- 不把普通聊天自动做成 observation 后立即入记忆。
- 不扫描完整 transcript。
- 不复制不必要敏感原文。
- 不让秘书、worker 或 system 采纳正式记忆。
- 不让 worker 单独把 observation 生成候选。
- 不把知识库命中直接变成 observation 再变候选。
- 不接 Obsidian 原生读写。
- 不接向量库或图数据库。
- 不改 `workflow-state.v0.json` 结构。
- 不迁移数据库。
- 不执行真实 Codex。
- 不执行 `codex exec` / `codex exec resume`。
- 不读写 `/Users/yoyi/.codex`。

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

- 在项目页或记忆入口显示最小“工作流观察”只读摘要。
- 显示 observation 数量、状态计数、最近 observation 摘要、candidate link。
- 显示 observation 生成 candidate 后的 `candidate_key`。
- 显示 warnings，提醒 observation 不是正式记忆。

本任务禁止显示：

- 不新增一级入口。
- 不新增右侧顶级入口。
- 不把 observation 铺进项目工作流画布主区域。
- 不把 observation / candidate 写成“已记住”“正式事实”“已注入任务包”。
- 不把审计、日志、schema、raw event 或路径大表放进普通主界面。
- 不把秘书放进项目画布右侧详情。

显示位置：

- 一级入口：不改。
- 右侧入口：不改。
- 项目页：允许在项目页侧栏或项目记忆相关区域显示最小只读 observation 摘要。
- 画布：不在画布主区域新增 observation 面板。
- 记忆入口：允许显示 observation 摘要或 candidate link，但不做完整记忆中心。
- 知识库入口：不改。
- 智能体入口：不改。
- 管理入口：不改。

中间版本范围：

- 本轮必须落地：ObservationStore 只读摘要、状态计数、candidate link。
- 本轮只做读模型 / 摘要：observation 列表和最近 observation。
- 本轮后置：完整记忆中心、关系图、任务包召回、任务包注入、Obsidian / 知识库联动。

后端和数据依赖：

- observation UI 必须来自后端 ObservationStore 读模型或正式 sidecar。
- candidate link 必须来自 MemoryCandidateStore。
- 不允许用假数据伪装 observation 已写入或 candidate 已生成。

UI 文案边界：

- 禁止说：“系统已记住”“自动学习完成”“observation 已成为正式记忆”“已注入任务包”。
- 允许说：“工作流观察”“观察可生成候选”“候选仍需确认 / 采纳”“observation 不是正式记忆”。

验收：

- 必须跑 `npm run typecheck`。
- 必须跑 `npm run test:offline-interaction`。
- 必须跑 `npm run build`。
- 如改变项目页布局，必须做真实窗口或浏览器截图验收；如果当前对话没有可用截图工具，必须在 evidence / handoff 写明“真实窗口 / 截图验收未完成”。

## 6. 建议存储

M3 推荐继续使用独立 sidecar，和 M1 / M2 保持一致：

```text
<workflow_state_dir>/observations.v1.json
```

sidecar 必须：

- 原子写入。
- lock 防并发覆盖。
- revision 防过期写入。
- 损坏 JSON 读取时拒绝覆盖。
- 写入前备份旧文件。
- 不写入 `workflow-state.v0.json`。

建议结构：

```ts
type ObservationStoreV1 = {
  store_version: "observation_store.v1";
  project_id?: string;
  workflow_id?: string;
  revision: number;
  observations: ObservationRecord[];
  events: ObservationAuditRef[];
  updated_at: string;
  warnings: string[];
};
```

## 7. 数据对象

复用已有：

- `MemoryScope`
- `MemorySourceRef`
- `MemoryLifecycleStatus`
- `CreateMemoryCandidateInput`
- `MemoryCandidate`

新增 observation 状态建议使用字符串枚举，不复用 `MemoryLifecycleStatus`，避免把 observation 和 memory lifecycle 混在一起：

```ts
type ObservationStatus =
  | "recorded"
  | "candidate_created"
  | "ignored"
  | "quarantined";
```

```ts
type ObservationType =
  | "worker_report"
  | "project_director_confirmation"
  | "global_director_review"
  | "plan_adopted"
  | "result_acceptance";
```

```ts
type ObservationSourceRef = {
  source_ref_id: string;
  source_kind:
    | "workflow_event"
    | "worker_report"
    | "director_review"
    | "task_package"
    | "evidence"
    | "handoff"
    | "user_confirmation";
  source_id: string;
  project_id?: string;
  workflow_id?: string;
  session_id?: string;
  file_path?: string;
  evidence_ref?: string;
  summary: string;
  sensitive_level: "public" | "internal" | "sensitive" | "secret";
  created_at: string;
};
```

```ts
type ObservationRecord = {
  observation_id: string;
  observation_key: string;
  schema_version: "memory_observation.v1";
  project_id?: string;
  workflow_id?: string;
  scope: MemoryScope;
  observation_type: ObservationType;
  summary: string;
  source_refs: ObservationSourceRef[];
  status: ObservationStatus;
  generated_by_role:
    | "worker"
    | "project_director"
    | "global_director"
    | "user"
    | "system";
  actor_id: string;
  risk_level: "low" | "medium" | "high";
  sensitive_level: "public" | "internal" | "sensitive" | "secret";
  candidate_key?: string;
  audit_refs: ObservationAuditRef[];
  created_at: string;
  updated_at: string;
};
```

```ts
type ObservationAuditRef = {
  audit_ref_id: string;
  event_type:
    | "observation_recorded"
    | "observation_ignored"
    | "observation_quarantined"
    | "observation_candidate_created";
  actor_id: string;
  actor_role: string;
  target_kind: "observation";
  target_id: string;
  before_status?: ObservationStatus;
  after_status?: ObservationStatus;
  reason: string;
  created_at: string;
};
```

```ts
type CreateObservationInput = {
  project_root: string;
  project_id?: string;
  workflow_id?: string;
  scope: MemoryScope;
  observation_type: ObservationType;
  summary: string;
  source_refs: ObservationSourceRef[];
  generated_by_role:
    | "worker"
    | "project_director"
    | "global_director"
    | "user"
    | "system";
  actor_id: string;
  risk_level: "low" | "medium" | "high";
  sensitive_level: "public" | "internal" | "sensitive" | "secret";
  reason: string;
  expected_store_revision?: number;
};
```

```ts
type CreateMemoryCandidateFromObservationInput = {
  project_root: string;
  observation_key: string;
  actor_id: string;
  actor_role: "project_director" | "global_director" | "user";
  memory_type:
    | "project_memory"
    | "workflow_summary"
    | "session_summary"
    | "user_preference"
    | "global_blueprint"
    | "mature_pattern";
  claim: string;
  body: string;
  review_reason: string;
  requires_user_confirmation: boolean;
  expected_observation_store_revision?: number;
  expected_candidate_store_revision?: number;
};
```

## 8. 控制核心规则

Observation 创建必须满足：

- `project_root` 非空。
- `summary` 非空。
- `source_refs.len() > 0`。
- `observation_type` 必须是白名单之一。
- `status` 初始只能是 `recorded`。
- `scope` 必须通过现有 project / workflow / session 绑定校验。
- `source_refs` 只能记录摘要和引用，不复制完整 transcript 或不必要原文。
- `secret` 来源必须保留为 `sensitive_level = "secret"`，且后续生成候选时 `model_export_policy` 必须 blocked。

Observation 生成候选必须满足：

- observation 存在。
- observation 当前状态必须是 `recorded`。
- observation 必须有 source refs。
- observation 不能是 `ignored` / `quarantined` / `candidate_created`。
- 同一个 observation 默认只能生成一个 candidate。
- 第一版默认要求 `actor_role = "project_director"` 才能把本项目 / workflow / session observation 生成候选。
- `global_director` / `user` 只能在明确全局或用户确认场景生成候选；如实现者开放，必须在 evidence 说明边界。
- `worker`、`secretary`、`system` 不能把 observation 生成候选。
- 生成候选必须走现有 `MemoryCandidateStore`，候选初始状态应是 `candidate_needs_review`。
- 生成候选后 observation 状态变为 `candidate_created`，并记录 candidate link。

普通聊天边界：

- 普通聊天不能被后台自动记录成 observation。
- 如果用户明确把一段聊天确认为方案、验收或偏好，才能以 `user_confirmation` source ref 记录。
- 不能因为 UI 当前显示了聊天摘要，就默认产生 observation。

## 9. 候选生成映射

从 observation 生成 `CreateMemoryCandidateInput` 时：

- `project_id` / `workflow_id` 来自 observation 绑定上下文。
- `scope` 来自 observation.scope。
- `claim` 来自生成请求，必须非空。
- `body` 来自生成请求，必须非空。
- `source_refs` 必须包含 observation source refs，并额外包含 observation 自身引用。
- `generated_by_role` 使用候选生成 actor role，不使用 worker 原始汇报角色。
- `generated_from` 建议写成 `observation:<observation_id>`。
- `status` 由候选 store 维持为 `candidate_needs_review`。
- `risk_level` / `sensitive_level` 至少不低于 observation 对应等级。
- 如果 observation 为 `secret` 或高风险，候选必须 `requires_user_confirmation = true`。

M3 不负责候选确认和正式采纳。候选后续仍必须走已有候选治理和 M2 受控采纳。

## 10. UI / 读模型要求

M3 不做完整记忆中心。

最小读模型需要能展示：

- observation sidecar 名称。
- revision。
- observation 数量。
- `recorded` / `candidate_created` / `ignored` / `quarantined` 数量。
- 最近一条 observation audit。
- 最近生成的 candidate link。
- warnings。

最小 UI 允许放在项目页或记忆入口：

- “工作流观察”只读摘要。
- observation 状态计数。
- 最近 observation 摘要。
- 已生成候选时显示 candidate_key。

UI 文案必须避免：

- “系统已记住”
- “自动学习完成”
- “observation 已成为正式记忆”
- “已注入任务包”

允许文案：

- “工作流观察”
- “观察可生成候选”
- “候选仍需确认 / 采纳”
- “observation 不是正式记忆”

## 11. 测试要求

Rust 必须覆盖：

1. `observation_store_records_worker_report`
   - worker report 写入 observation。
   - observation 状态为 `recorded`。
   - source refs 非空。

2. `observation_candidate_creation_project_director`
   - 项目主管从 recorded observation 生成 `MemoryCandidate`。
   - candidate store 增加候选。
   - observation 状态变为 `candidate_created`。
   - observation 保存 candidate_key。

3. `observation_candidate_creation_rejects_quarantined`
   - quarantined observation 不能生成 candidate。

4. `observation_candidate_creation_rejects_ignored`
   - ignored observation 不能生成 candidate。

5. `observation_candidate_creation_rejects_duplicate`
   - 同一 observation 不能重复生成 candidate。

6. `observation_creation_rejects_missing_source_refs`
   - 无 source refs 时拒绝记录 observation。

7. `observation_creation_rejects_ordinary_chat_auto_capture`
   - 未明确确认为工作流事件 / 汇报 / 复核 / 方案 / 验收的普通聊天不能创建 observation。

8. `observation_candidate_does_not_create_formal_memory`
   - observation 生成 candidate 后，formal memory store 不新增 record。

9. `observation_context_binding_mismatch_rejected`
   - `project_root` 与 project / workflow scope 不匹配时拒绝。

前端离线测试必须覆盖：

- observation 只读摘要能显示状态计数。
- observation 生成 candidate 后 UI 能看到 candidate_key。
- 文案不把 observation 或 candidate 说成正式记忆。

## 12. 验证命令

至少运行：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
cargo test --lib observation
cargo test --lib memory_candidate
cargo test --lib formal_memory
rustfmt --check src/observation_store.rs src/memory_candidate_store.rs src/control_core.rs src/commands.rs src/types.rs
```

如果实现没有改前端，可说明为什么没有跑前端构建；但推荐仍跑 typecheck / offline interaction，避免 Tauri 类型漂移。

## 13. 验收标准

接受为完成：

- 新增 ObservationStore，使用独立 sidecar，具备原子写入、lock、revision 和损坏 JSON 拒绝覆盖。
- worker 汇报、项目主管确认、全局主管复核、方案采纳、结果验收中至少有一条受控入口可写入 observation。
- observation 必须带 source refs。
- observation 能标记 `recorded`、`candidate_created`、`ignored`、`quarantined`。
- 项目主管确认过程事实后，recorded observation 可以生成 MemoryCandidate。
- observation 生成 candidate 后状态变为 `candidate_created`，并能追溯 candidate_key。
- ignored / quarantined observation 不生成 candidate。
- 生成 candidate 后 formal memory store 不新增 record。

不接受为完成：

- observation 直接成为正式记忆。
- observation 直接注入任务包。
- 普通聊天自动入 observation 并生成候选。
- 任务包记忆召回完成。
- 任务包记忆注入完成。
- Obsidian / 知识库 / 向量库 / 图数据库完成。
- 中间版本记忆层完成。

## 14. 回收记录要求

完成实现后新增：

- `evidence/2026-06-04-memory-layer-m3-observation-store-and-workflow-entry-v1.md`
- `handoffs/2026-06-04-memory-layer-m3-observation-store-and-workflow-entry-v1-result.md`

并更新：

- `CURRENT.md`
- `tasks/README.md`
- `docs/plans/middleware-version-stage-plan-v1.md`

回收结论必须明确：

- M3 只接受为 ObservationStore 和工作流观察入口完成。
- 不接受为正式记忆生命周期完成。
- 不接受为任务包记忆注入完成。
- 不接受为中间版本记忆层完成。
