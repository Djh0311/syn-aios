# Task Package：final-skeleton-11 + 14 候选治理最小闭环批次 v1

状态：待执行。  
用途：把 `final-skeleton-11` 和 `final-skeleton-14` 合并成一个顺序批次，减少重复治理流程。  
执行顺序：先 11，后 14，最后做交叉边界测试。

## 1. 先说薄弱点

- 这个批次包含两种“候选”，最容易混淆。
- `BlackboardCandidate` 是项目黑板协作中间态。
- `MemoryCandidate` 是长期行为依据的候选。
- 两者都可以确认，但确认含义不同。
- 两者都用 sidecar，但文件必须分开。
- 两者都不能写正式事实、正式记忆或 workflow state JSON 新字段。

一句话边界：

```text
黑板候选确认 = 候选值得后续处理。
记忆候选确认 = 候选被确认保留。
二者都不等于正式事实或正式长期记忆已经生效。
```

## 2. 必须先读

当前入口：

- `CURRENT.md`
- `tasks/README.md`
- `tasks/2026-06-01-final-workbench-skeleton-execution-package-v1.md`

黑板候选依据：

- `docs/plans/2026-06-01-blackboard-candidate-persistence-schema-v1.md`
- `evidence/2026-06-01-final-skeleton-10-blackboard-candidate-schema-design-v1.md`
- `handoffs/2026-06-01-final-skeleton-10-blackboard-candidate-schema-design-v1-result.md`

记忆候选依据：

- `docs/plans/2026-06-01-memory-governance-schema-v1.md`
- `evidence/2026-06-01-final-skeleton-13-memory-governance-schema-design-v1.md`
- `handoffs/2026-06-01-final-skeleton-13-memory-governance-schema-design-v1-result.md`
- `tasks/2026-06-03-final-skeleton-14-memory-governance-minimal-implementation-v1.md`

实现参考：

- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/control_core.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_state_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_audit.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_read_model.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `prototypes/productized-desktop-shell/src/lib/projectCanvas.ts`
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

## 3. 总目标

在不改 workflow state JSON 结构、不写正式记忆、不写正式事实的前提下，实现两个最小闭环：

1. `final-skeleton-11`：黑板候选持久确认状态。
2. `final-skeleton-14`：记忆候选生命周期。
3. 交叉测试：证明黑板候选和记忆候选不会互相串线。

## 4. 全局禁止

- 不写正式事实。
- 不写正式 `MemoryRecord`。
- 不写长期正式记忆。
- 不改 `workflow-state.v0.json` 结构。
- 不新增未确认的 workflow state JSON 字段。
- 不迁移数据库。
- 不接 SQLite。
- 不接向量库。
- 不接图数据库。
- 不接 Obsidian 原生读写。
- 不自动扫描知识库。
- 不把普通聊天自动写记忆。
- 不让秘书直接写正式记忆。
- 不让知识引用直接成为记忆。
- 不让工具摘要直接推进 workflow 状态。
- 不执行真实 Codex。
- 不读写 `/Users/yoyi/.codex`。
- 不接 Claude / OpenClaw / OpenCode。
- 不启动 MCP canvas run。
- 不写真实业务项目目录。
- 不做广泛 UI 重构。
- 不继续往项目画布右侧栏堆新主面板。
- 不批量格式化 `src/lib.rs` 或 `src/mcp/**`。

## 5. 两个 sidecar

必须分开：

```text
<workflow_state_dir>/blackboard-candidates.v1.json
<workflow_state_dir>/memory-candidates.v1.json
```

规则：

- `blackboard-candidates.v1.json` 只保存黑板候选状态。
- `memory-candidates.v1.json` 只保存记忆候选状态。
- 任一 sidecar 的存在都不能改变 `workflow-state.v0.json` 的 schema。
- 任一 sidecar 都不能保存 transcript 全文、auth、token、密钥或 `.env`。
- 任一 sidecar 都不能写入 `/Users/yoyi/.codex`。
- 允许复用原子写、备份、revision、event 模式，但类型和命令必须分开。

## 6. 第一段：final-skeleton-11 黑板候选持久确认

### 6.1 目标

实现黑板候选确认、拒绝、待处理、暂缓、废弃的最小持久状态闭环。

允许状态：

```ts
type BlackboardCandidateState =
  | "candidate_pending_control_core"
  | "candidate_confirmed_for_followup"
  | "candidate_rejected"
  | "candidate_deferred"
  | "candidate_discarded";
```

关键语义：

- `candidate_confirmed_for_followup` 只表示候选值得保留或进入后续流程。
- 不等于正式事实。
- 不等于正式记忆。
- 不等于权限批准。
- 不等于 workflow 状态推进。

### 6.2 Store

建议：

```ts
type BlackboardCandidateStoreV1 = {
  store_version: "blackboard_candidate_persistence.v1";
  revision: number;
  candidates: BlackboardCandidateRecord[];
  events: BlackboardCandidateEvent[];
  updated_at: string;
};
```

候选记录至少包括：

- `candidate_key`
- `project_id`
- `workflow_id`
- `source_entry_id`
- `source_entry_kind`
- `target_kind`
- `state`
- `source_refs`
- `decision_reason`
- `created_at`
- `updated_at`

事件至少包括：

- `event_id`
- `event_version`
- `event_type`
- `candidate_key`
- `before_state`
- `after_state`
- `actor_id`
- `actor_role`
- `reason`
- `created_at`

### 6.3 后端建议

新增或拆出：

- `src-tauri/src/blackboard_candidate_store.rs`

职责：

- 计算 `blackboard-candidates.v1.json` 路径。
- 初始化空 store。
- 读取 store。
- 校验 store。
- 原子写 store。
- 追加 event。
- 处理 revision 冲突。

控制核心：

- 扩展现有 `validate_blackboard_candidate_decision`。
- 支持 `candidate_confirmed_for_followup`、`candidate_rejected`、`candidate_deferred`、`candidate_discarded`。
- 继续拒绝 `candidate_confirmed_for_fact`、`candidate_confirmed_for_memory`、`permission_approved`、`workflow_state_change` 等正式晋升。

命令建议：

- `load_blackboard_candidate_store`
- `record_blackboard_candidate_decision`

禁止命令：

- `promote_blackboard_candidate_to_fact`
- `promote_blackboard_candidate_to_memory`
- `approve_permission_from_blackboard`
- `change_workflow_state_from_blackboard`

### 6.4 前端建议

在现有项目页 / 项目黑板区显示状态即可。

允许文案：

- “黑板候选待处理”
- “黑板候选已确认后续处理”
- “黑板候选已拒绝”
- “黑板候选已暂缓”
- “黑板候选已废弃”

禁止文案：

- “正式事实已写入”
- “正式记忆已写入”
- “权限已批准”
- “工作流已推进”

## 7. 第二段：final-skeleton-14 记忆候选生命周期

### 7.1 目标

实现 `MemoryCandidate` 的最小生命周期：

- 创建候选。
- 读取候选。
- 确认保留候选。
- 拒绝候选。
- 隔离候选。
- 废弃候选。
- 记录候选事件。

允许状态：

```ts
type MemoryLifecycleStatus =
  | "candidate_draft"
  | "candidate_needs_review"
  | "candidate_confirmed"
  | "candidate_rejected"
  | "candidate_quarantined"
  | "candidate_superseded"
  | "candidate_discarded";
```

关键语义：

- `candidate_confirmed` 只表示候选被确认保留。
- 不等于正式 `MemoryRecord`。
- 不等于系统已经长期记住。
- 不等于任务包已经可引用。

### 7.2 Store

使用：

```text
<workflow_state_dir>/memory-candidates.v1.json
```

建议：

```ts
type MemoryCandidateStoreV1 = {
  store_version: "memory_candidate_store.v1";
  project_id?: string;
  workflow_id?: string;
  revision: number;
  candidates: MemoryCandidate[];
  events: MemoryCandidateEvent[];
  updated_at: string;
};
```

### 7.3 后端建议

新增：

- `src-tauri/src/memory_candidate_store.rs`

控制核心新增：

- `validate_memory_candidate_create`
- `validate_memory_candidate_status_transition`
- `validate_memory_candidate_source_refs`
- `validate_memory_candidate_scope`

命令建议：

- `load_memory_candidate_store`
- `create_memory_candidate`
- `record_memory_candidate_decision`

禁止命令：

- `create_memory_record`
- `promote_to_memory_record`
- `write_formal_memory`

### 7.4 前端建议

只做最小候选区，不新增大页面。

允许文案：

- “记忆候选待审”
- “记忆候选已确认保留”
- “记忆候选已隔离”
- “记忆候选已废弃”

禁止文案：

- “已记住”
- “正式记忆已写入”
- “长期记忆已生效”
- “已学习”

## 8. 交叉边界测试

必须证明：

- 黑板候选确认不会生成 `MemoryCandidate`。
- 黑板候选确认不会生成正式 `MemoryRecord`。
- 黑板候选确认不会改 workflow state。
- 记忆候选确认不会写黑板 sidecar。
- 记忆候选确认不会生成正式 `MemoryRecord`。
- 知识引用不能通过黑板候选直接变记忆候选。
- 工具摘要不能通过黑板候选推进 workflow state。
- 权限请求不能通过黑板候选直接批准。
- 两个 sidecar 都能独立读写、独立 revision。
- 损坏一个 sidecar 不覆盖另一个 sidecar。

## 9. UI 边界

允许：

- 在现有项目黑板 / 候选区域增加小型状态显示。
- 在现有项目页增加必要确认入口。
- 用右侧抽屉 / 现有节点详情展示候选状态。

禁止：

- 新增记忆大页面。
- 新增知识库页面。
- 往项目画布右侧栏继续堆新主面板。
- 把候选状态放成工作流事实。
- 把候选状态放成正式记忆。

## 10. 测试要求

必须补离线测试：

黑板候选：

- sidecar 不存在时能初始化。
- 合法确认写入 `candidate_confirmed_for_followup`。
- 合法拒绝写入 `candidate_rejected`。
- 合法暂缓写入 `candidate_deferred`。
- 合法废弃写入 `candidate_discarded`。
- `candidate_confirmed_for_memory` 被拒绝。
- `candidate_confirmed_for_fact` 被拒绝。
- 权限请求不能通过黑板候选直接批准。
- workflow state JSON 没有新增黑板候选字段。

记忆候选：

- 创建用户偏好候选。
- 用户偏好候选可变为 `candidate_confirmed`。
- `candidate_confirmed` 不生成 `MemoryRecord`。
- 工作流总结只能生成候选。
- 知识库来源只能生成候选。
- 无来源候选被拒绝。
- `candidate_confirmed -> memory_active` 被拒绝。
- UI 文案不出现“已记住”或“正式记忆已写入”。

交叉：

- 黑板 sidecar 和记忆 sidecar 文件名不同。
- 黑板命令不能写记忆 sidecar。
- 记忆命令不能写黑板 sidecar。
- 两者都不改 workflow state JSON 结构。

建议补 Rust 测试：

- 原子写。
- revision 增加。
- 损坏 JSON 不覆盖。
- 并发 revision 冲突。
- 缺来源拒绝。
- 正式晋升命令不存在或不可调用。

## 11. 必跑验证

至少跑：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
cargo test --lib
```

如果新增 Rust 文件：

```text
rustfmt --check src/blackboard_candidate_store.rs src/memory_candidate_store.rs
```

不要顺手批量格式化历史债。`cargo fmt --check` 如果因为既有 `src/lib.rs` 或 `src/mcp/**` 失败，只记录原因。

## 12. 必须输出

新增 evidence：

- `evidence/2026-06-03-final-skeleton-11-14-candidate-governance-minimal-closed-loop-v1.md`

新增 handoff：

- `handoffs/2026-06-03-final-skeleton-11-14-candidate-governance-minimal-closed-loop-v1-result.md`

可以同时兼容总包要求输出：

- `evidence/2026-06-01-final-skeleton-11-blackboard-candidate-persistence-implementation-v1.md`
- `handoffs/2026-06-01-final-skeleton-11-blackboard-candidate-persistence-implementation-v1-result.md`
- `evidence/2026-06-01-final-skeleton-14-memory-governance-minimal-implementation-v1.md`
- `handoffs/2026-06-01-final-skeleton-14-memory-governance-minimal-implementation-v1-result.md`

如果选择只写合并 evidence / handoff，也必须在里面分段说明 11 和 14。

更新：

- `CURRENT.md`
- `tasks/README.md`

## 13. 验收标准

接受为：

- 黑板候选持久确认最小闭环完成。
- 记忆候选生命周期最小闭环完成。
- 两个 sidecar 分离。
- 两套命令分离。
- 两套 UI 文案分离。
- 交叉边界测试覆盖。
- 没有写正式事实。
- 没有写正式长期记忆。
- 没有改 workflow state JSON 结构。

不接受为：

- 正式事实系统完成。
- 正式记忆系统完成。
- 任务包记忆注入完成。
- Obsidian / 知识库集成完成。
- 向量库 / 图数据库完成。
- 秘书核心只读模型完成。
- 黑板候选能直接变成记忆。

## 14. 停止条件

遇到以下情况必须停下来回传：

- 需要写正式事实。
- 需要写正式 `MemoryRecord`。
- 需要把候选注入任务包作为正式依据。
- 需要迁移数据库。
- 需要接 Obsidian 原生功能。
- 需要执行真实 Codex。
- 需要读写 `/Users/yoyi/.codex`。
- 需要改 workflow state JSON 结构。
- 需要把黑板候选升级为正式事实或正式记忆。
- 需要让秘书自动改事实或写正式记忆。

## 15. 完成后

如果全部通过：

- 可以继续 `final-skeleton-15-secretary-core-readonly-model-v1`。

但 Skeleton-15 仍然不能：

- 让秘书直接改事实。
- 让秘书直接派发任务。
- 让秘书写正式记忆。
- 把秘书固定成某一个页面。
