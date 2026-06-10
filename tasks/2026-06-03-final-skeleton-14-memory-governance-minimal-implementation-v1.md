# Task Package：final-skeleton-14 记忆候选生命周期最小实现 v1

状态：待执行。  
对应总包：`2026-06-01-final-workbench-skeleton-execution-package-v1.md` 的 `final-skeleton-14-memory-governance-minimal-implementation-v1`。  
前置：用户已允许进入 Skeleton-14，但本任务仍不能写正式长期记忆。

## 1. 先说薄弱点

- 这不是完整记忆层。
- 这不是正式长期记忆写入。
- 这不是 Obsidian / 知识库集成。
- 这不是向量库、图数据库或召回算法。
- 这只做 `MemoryCandidate` 候选生命周期的最小闭环。
- UI 只能做只读和必要确认入口，不能继续往项目画布右侧栏堆新主面板。

## 2. 依据

必须先读：

- `CURRENT.md`
- `tasks/README.md`
- `docs/plans/2026-06-01-memory-governance-schema-v1.md`
- `docs/memory-layer-design-v1.md`
- `docs/workbench-system-architecture-v1.md`
- `tasks/2026-06-01-final-workbench-skeleton-execution-package-v1.md`

实现时参考：

- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/control_core.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_state_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_audit.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_read_model.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

## 3. 目标

实现一个最小候选记忆闭环：

1. 可以从工作台内生成 `MemoryCandidate`。
2. 可以读取候选列表。
3. 可以把候选状态改为：
   - `candidate_confirmed`
   - `candidate_rejected`
   - `candidate_quarantined`
   - `candidate_discarded`
4. 每次状态变化都写候选事件。
5. UI 只显示候选和必要确认入口。
6. 候选确认不生成正式 `MemoryRecord`。

## 4. 禁止

- 不写正式 `MemoryRecord`。
- 不写长期正式记忆。
- 不把 `candidate_confirmed` 解释成“已经记住”。
- 不把黑板候选直接升级为正式事实、正式记忆或 workflow 状态。
- 不改 `workflow-state.v0.json` 结构。
- 不迁移数据库。
- 不接 SQLite。
- 不接向量库。
- 不接图数据库。
- 不接 Obsidian 原生读写。
- 不自动扫描知识库。
- 不把普通聊天自动写记忆。
- 不让秘书直接写正式记忆。
- 不执行真实 Codex。
- 不读写 `/Users/yoyi/.codex`。
- 不接 Claude / OpenClaw / OpenCode。
- 不启动 MCP canvas run。
- 不写真实业务项目目录。
- 不做广泛 UI 重构。
- 不批量格式化 `src/lib.rs` 或 `src/mcp/**`。

## 5. 存储方案

第一版使用独立 sidecar JSON：

```text
<workflow_state_dir>/memory-candidates.v1.json
```

说明：

- `<workflow_state_dir>` 是当前 workflow state 文件所在目录。
- 不写 `workflow-state.v0.json` 结构。
- 不迁移数据库。
- 不写正式记忆。
- 读模型可以同时读取 workflow state 和 sidecar，但不能让 sidecar 改变 workflow state。

建议 store：

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

建议事件：

```ts
type MemoryCandidateEvent = {
  event_id: string;
  event_version: "memory_candidate_event.v1";
  event_type:
    | "memory_candidate_created"
    | "memory_candidate_status_changed"
    | "memory_candidate_conflict_detected";
  candidate_id: string;
  actor_id: string;
  actor_role: "user" | "secretary" | "project_director" | "system" | "agent";
  before_status?: MemoryLifecycleStatus;
  after_status?: MemoryLifecycleStatus;
  reason: string;
  created_at: string;
};
```

## 6. 写入规则

必须：

- 原子写入。
- 写入前读当前 `revision`。
- 写入后 `revision + 1`。
- 状态变化追加 event。
- 失败时不能留下半写文件。
- 可以复用 `workflow_state_store.rs` 里的原子写 / 备份模式。

建议：

- 如果 sidecar 不存在，创建空 store。
- 如果 JSON 损坏，返回错误，不自动覆盖。
- 如果 `revision` 不匹配，返回并发冲突。
- 候选 id 使用稳定 key：

```text
memcand:v1:sha256(scope_type + scope ids + memory_type + normalized claim + source refs)
```

## 7. 后端实现步骤

### 7.1 类型

在 `src-tauri/src/types.rs` 增加：

- `MemoryScope`
- `MemorySourceRef`
- `MemoryCandidate`
- `MemoryRecord` 只定义类型，不写入。
- `MemoryLifecycleStatus`
- `MemoryConflict`
- `MemoryAuditRef`
- `MemoryCandidateStoreV1`
- `MemoryCandidateEvent`
- 命令输入 / 输出类型。

要求：

- 和 `docs/plans/2026-06-01-memory-governance-schema-v1.md` 对齐。
- `MemoryRecord` 只能作为只读目标类型，不提供写入命令。
- `candidate_confirmed` 不能命名成 `approved`，避免误读。

### 7.2 sidecar store helper

建议新增文件：

- `src-tauri/src/memory_candidate_store.rs`

职责：

- 计算 sidecar 路径。
- 读取 store。
- 初始化空 store。
- 校验 store。
- 原子写 store。
- 追加 event。

禁止：

- 不写 workflow state。
- 不碰真实 Codex。
- 不读 `/Users/yoyi/.codex`。

### 7.3 控制核心校验

在 `control_core.rs` 增加小型校验函数：

- `validate_memory_candidate_create`
- `validate_memory_candidate_status_transition`
- `validate_memory_candidate_source_refs`
- `validate_memory_candidate_scope`

必须拒绝：

- 无来源候选。
- `knowledge_doc` 直接写正式记忆。
- `candidate_confirmed -> memory_active`。
- 普通聊天自动写正式记忆。
- `scope_type = global` 或 `user_preference` 且没有用户确认理由。
- `sensitive_level = secret` 且要进入外发模型上下文。

### 7.4 Tauri commands

在 `commands.rs` / `lib.rs` 接入命令：

- `load_memory_candidate_store`
- `create_memory_candidate`
- `record_memory_candidate_decision`

`record_memory_candidate_decision` 只允许：

- `candidate_confirmed`
- `candidate_rejected`
- `candidate_quarantined`
- `candidate_discarded`

禁止：

- 不提供 `create_memory_record`。
- 不提供 `promote_to_memory_record`。
- 不提供正式记忆写入接口。

### 7.5 读模型

可以新增前端纯读模型：

- `src/lib/memoryCandidates.ts`

用途：

- 把候选列表整理成 UI 需要的分组。
- 标出用户偏好候选、项目候选、知识库来源候选。
- 标出冲突和需要用户确认的候选。

## 8. 前端实现步骤

### 8.1 类型

在 `src/lib/types.ts` 增加与后端一致的类型。

要求：

- 字段名 snake_case。
- 不把正式 `MemoryRecord` 写入能力暴露成按钮。

### 8.2 Tauri 调用

在 `src/lib/tauri.ts` 增加：

- `loadMemoryCandidateStore`
- `createMemoryCandidate`
- `recordMemoryCandidateDecision`

### 8.3 UI 入口

只做最小入口，优先放在项目工作流的现有信息区或项目黑板附近。

允许：

- 一个“记忆候选”只读区域。
- 每个候选显示：
  - claim
  - scope
  - status
  - source refs
  - conflict count
  - warnings
- 必要动作：
  - 确认保留候选
  - 拒绝候选
  - 隔离候选
  - 废弃候选

禁止：

- 不新增大页面。
- 不把记忆候选塞进项目画布右侧详情主面板。
- 不显示“已记住”。
- 不显示“写入正式记忆”。
- 不显示“长期记忆已生效”。
- 不做 Obsidian UI。

### 8.4 文案

必须使用：

- “候选已确认保留”
- “候选待审”
- “候选已隔离”
- “候选已废弃”

禁止使用：

- “已记住”
- “正式记忆已写入”
- “长期记忆已生效”
- “已学习”

## 9. 测试要求

必须补离线测试：

- 创建用户偏好候选。
- 用户偏好候选可变为 `candidate_confirmed`。
- `candidate_confirmed` 不生成 `MemoryRecord`。
- 工作流总结只能生成候选。
- 知识库来源只能生成候选。
- 无来源候选被拒绝。
- `candidate_confirmed -> memory_active` 被拒绝。
- 冲突候选不能进入任务包依据。
- UI 文案不出现“已记住”或“正式记忆已写入”。

建议补 Rust 测试：

- sidecar 不存在时创建空 store。
- 状态变化追加 event。
- revision 增加。
- 损坏 JSON 不覆盖。
- 无来源创建被拒绝。
- 正式记忆写入接口不存在或不可调用。

## 10. 验证命令

至少跑：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
cargo test --lib
```

如果新增 Rust 文件：

```text
rustfmt --check src/memory_candidate_store.rs
```

不要强行跑全仓库 `cargo fmt --check` 后顺手格式化历史债；如果失败，只记录原因。

## 11. 必须输出

新增：

- `evidence/2026-06-01-final-skeleton-14-memory-governance-minimal-implementation-v1.md`
- `handoffs/2026-06-01-final-skeleton-14-memory-governance-minimal-implementation-v1-result.md`

更新：

- `CURRENT.md`
- `tasks/README.md`

evidence 必须写清：

- 改了哪些文件。
- 跑了哪些验证。
- 是否生成 sidecar。
- 是否写了 workflow state。
- 是否写了正式记忆。
- 是否读写 `/Users/yoyi/.codex`。
- 是否执行真实 Codex。

handoff 必须写清：

- 本轮接受为什么。
- 不接受为什么。
- 手动测试清单。
- 下一个任务是否可以继续 Skeleton-15。

## 12. 验收标准

接受为：

- 记忆候选生命周期最小闭环完成。
- 用户偏好可以作为候选被确认保留。
- 工作流总结只能先成为候选。
- 知识库材料只能作为来源，不是记忆。
- 候选状态变化有 event。
- `candidate_confirmed` 不生成正式 `MemoryRecord`。

不接受为：

- 正式长期记忆系统完成。
- 记忆召回算法完成。
- 任务包记忆注入完成。
- Obsidian / 知识库集成完成。
- 向量库 / 图数据库完成。
- 秘书记忆协作完成。

## 13. 停止条件

遇到以下情况必须停止并回传：

- 需要写正式 `MemoryRecord`。
- 需要把候选注入任务包作为正式依据。
- 需要迁移数据库。
- 需要接 Obsidian 原生功能。
- 需要执行真实 Codex。
- 需要读写 `/Users/yoyi/.codex`。
- 需要改 workflow state JSON 结构。
- 需要把黑板候选升级为正式事实或正式记忆。
