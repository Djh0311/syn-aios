# Task Package：Memory Layer M1 Formal Memory Store And Audit v1

状态：已完成，依据见 `../evidence/2026-06-03-memory-layer-m1-formal-memory-store-and-audit-v1.md` 与 `../handoffs/2026-06-03-memory-layer-m1-formal-memory-store-and-audit-v1-result.md`。  
用途：实现中间版本记忆层 M1：正式记忆受控存储和审计骨架。  
执行方式：一个中等批次完成，不拆成十几个微任务；最终统一验收。

## 1. 先说薄弱点

- 当前 app 只有 `memory-candidates.v1.json` 候选 sidecar，还没有正式记忆存储。
- `MemoryRecord` 当前只是 Rust 类型目标形状，不是可写、可读、可审计的正式记忆。
- M1 只做正式记忆受控存储、第一版 version、审计事件和只读读模型；不做候选采纳、不做任务包注入。
- M1 完成后仍不能宣称中间版本记忆层完成；只能宣称“正式记忆存储和审计骨架完成”。

## 2. 任务目标

实现一个受控的正式记忆存储骨架：

```text
显式正式记忆创建请求
-> 控制核心校验来源 / 状态 / 权限边界
-> FormalMemoryStore 写入 MemoryRecord
-> MemoryVersionStore 创建第一版版本
-> MemoryAuditStore 记录 memory_record_created
-> WorkbenchSnapshot 或独立读模型能只读展示正式记忆摘要
```

必须证明：

- 正式记忆至少有一个来源。
- 正式记忆创建时同步创建 version 和 audit。
- 写入失败不能留下半条 record。
- `candidate_confirmed` 不会自动创建正式记忆。

## 3. 必须先读

当前入口：

- `CURRENT.md`
- `tasks/README.md`
- `docs/middleware-version-development-plan-v1.md`
- `docs/plans/memory-layer-implementation-slice-v1.md`
- `docs/memory-layer-design-v1.md`
- `docs/plans/2026-06-01-memory-governance-schema-v1.md`

当前实现：

- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_candidate_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/control_core.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

前置记录：

- `evidence/2026-06-03-memory-layer-deep-research-and-implementation-slice-v1.md`
- `handoffs/2026-06-03-memory-layer-deep-research-and-implementation-slice-v1-result.md`
- `evidence/2026-06-03-final-skeleton-11-14-candidate-governance-minimal-closed-loop-v1.md`
- `handoffs/2026-06-03-final-skeleton-11-14-candidate-governance-minimal-closed-loop-v1-result.md`

## 4. 范围

允许：

- 新增 Rust 后端正式记忆 store，例如 `formal_memory_store.rs`。
- 新增正式记忆 store 类型，例如 `FormalMemoryStoreV1`、`MemoryVersion`、`MemoryAuditEvent`。
- 新增后端命令：
  - `load_formal_memory_store`
  - `create_formal_memory_record`
- 新增前端类型和 Tauri 包装。
- 新增只读读模型，用于汇总正式记忆数量、状态、最近审计和 warnings。
- 在项目页或记忆入口放一个最小只读摘要，但不要扩大 UI 改造。
- 新增 Rust 单测和前端离线测试。
- 更新 evidence / handoff / `CURRENT.md` / `tasks/README.md`。

禁止：

- 不从 `MemoryCandidate` 自动升级正式记忆。
- 不新增“采纳候选为正式记忆”命令；那是 M2。
- 不接 `TaskMemoryPacketBuilder`；那是 M4。
- 不把正式记忆注入 worker 任务包；那是 M6。
- 不做正式记忆编辑、废弃、冻结、合并、拆分；那是 M9。
- 不接 Obsidian 原生读写。
- 不接向量库或图数据库。
- 不扫描知识库。
- 不改 `workflow-state.v0.json` 结构。
- 不迁移数据库。
- 不执行真实 Codex。
- 不执行 `codex exec` / `codex exec resume`。
- 不读写 `/Users/yoyi/.codex`。
- 不写真实业务项目目录。

## 5. 建议存储

M1 推荐先使用独立 sidecar，避免未确认数据库迁移：

```text
<workflow_state_dir>/formal-memories.v1.json
```

M1 可以选择 SQLite，但只有在任务执行者能同时提供迁移 / 回滚 / 不影响现有 sidecar 的方案时才允许。否则优先 sidecar。

sidecar 必须：

- 原子写入。
- lock 防并发覆盖。
- revision 防过期写入。
- 损坏 JSON 读取时拒绝覆盖。
- 写入前备份旧文件。
- 不写入 `workflow-state.v0.json`。

建议结构：

```ts
type FormalMemoryStoreV1 = {
  store_version: "formal_memory_store.v1";
  project_id?: string;
  workflow_id?: string;
  revision: number;
  records: MemoryRecord[];
  versions: MemoryVersion[];
  audit_events: MemoryAuditEvent[];
  updated_at: string;
  warnings: string[];
};
```

## 6. 数据对象

复用已有：

- `MemoryScope`
- `MemorySourceRef`
- `MemoryLifecycleStatus`
- `MemoryAuditRef`
- `MemoryRecord`

新增：

```ts
type MemoryVersion = {
  version_id: string;
  memory_id: string;
  version_number: number;
  change_type: "created" | "manual_revision";
  change_summary: string;
  record_snapshot: MemoryRecord;
  source_refs: MemorySourceRef[];
  changed_by_role: "user" | "project_director" | "global_director" | "system";
  reviewed_by?: string;
  created_at: string;
};
```

```ts
type MemoryAuditEvent = {
  audit_event_id: string;
  event_type: "memory_record_created" | "memory_record_create_rejected";
  actor_id: string;
  actor_role: "user" | "project_director" | "global_director" | "system";
  project_id?: string;
  workflow_id?: string;
  session_id?: string;
  target_kind: "memory_record";
  target_id?: string;
  before_state?: string;
  after_state?: string;
  reason: string;
  source_refs: MemorySourceRef[];
  status: "succeeded" | "failed";
  created_at: string;
};
```

```ts
type CreateFormalMemoryRecordInput = {
  project_root: string;
  project_id?: string;
  workflow_id?: string;
  scope: MemoryScope;
  memory_type:
    | "user_preference"
    | "global_blueprint"
    | "project_memory"
    | "workflow_summary"
    | "session_summary"
    | "mature_pattern";
  claim: string;
  body: string;
  source_refs: MemorySourceRef[];
  actor_id: string;
  actor_role: "user" | "project_director" | "global_director" | "system";
  reason: string;
  expected_store_revision?: number;
};
```

```ts
type CreateFormalMemoryRecordOutput = {
  record: MemoryRecord;
  version: MemoryVersion;
  audit_event: MemoryAuditEvent;
  store_revision: number;
  warnings: string[];
};
```

## 7. 控制核心校验

新增 helper，建议放在 `control_core.rs`：

- `validate_formal_memory_create(...)`

必须校验：

- `claim` 非空。
- `body` 非空。
- `source_refs.len() > 0`。
- `scope.scope_type` 合法。
- `model_export_policy` 合法。
- `memory_type` 合法。
- `actor_role` 合法。
- 正式记忆初始 status 只能是 `memory_active`。
- `secret` 来源或敏感内容必须阻止外发模型上下文。
- `user_preference` / `global_blueprint` / `mature_pattern` / 跨项目影响不能由普通 system 或 worker 创建。

M1 不做复杂权限表，但必须先有白名单校验：

- `user` 可以创建任何正式记忆。
- `project_director` 只能创建 `project_memory`、`workflow_summary`、`session_summary`，且 scope 必须是本项目 / workflow / session。
- `global_director` 可以创建 `global_blueprint` 候选之外的正式全局内容吗？M1 暂不建议允许；如果执行者要允许，必须在 evidence 里说明边界。
- `system` 只能用于测试或导入，默认不允许创建高风险正式记忆。

## 8. UI / 读模型要求

M1 不做完整记忆管理页面。

最小读模型需要能展示：

- formal memory sidecar 名称。
- revision。
- 正式记忆数量。
- `memory_active` 数量。
- 非 active 数量。
- 最近一条 audit event。
- warnings。

UI 文案必须避免：

- “候选已记住”
- “系统已学习”
- “正式记忆完整完成”

允许文案：

- “正式记忆骨架”
- “受控正式记忆”
- “创建时写入 version 和 audit”
- “M1 不包含候选采纳和任务包注入”

## 9. 测试要求

Rust 测试至少覆盖：

- `formal_memory_store_creates_record_version_and_audit`
- `formal_memory_store_rejects_missing_source_refs`
- `formal_memory_store_rejects_candidate_status`
- `formal_memory_store_keeps_candidate_store_separate`
- `formal_memory_store_damaged_json_is_not_overwritten`
- `formal_memory_store_revision_conflict_is_rejected`

前端离线测试至少覆盖：

- 正式记忆摘要显示 record / version / audit 数量。
- UI 不把候选确认说成正式记忆。
- M1 文案不暗示任务包注入已完成。

验证命令：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
cargo test --lib
rustfmt --check src/formal_memory_store.rs
```

如果 `cargo fmt --check` 因既有 `lib.rs` 或 `src/mcp/**` 格式债失败，不要批量格式化；只记录原因。

## 10. 验收标准

接受为：

- 正式记忆受控 store M1 完成。
- 显式正式记忆创建能生成 record、version、audit。
- 无来源正式记忆创建会被拒绝。
- 候选 store 和正式记忆 store 分离。
- `candidate_confirmed` 不会自动创建正式记忆。
- 读模型能显示正式记忆骨架状态。

不接受为：

- 候选采纳流程完成。
- 任务包召回完成。
- 任务包注入完成。
- 完整记忆管理页面完成。
- 正式记忆生命周期操作完成。
- 中间版本记忆层完成。
- Obsidian / 知识库集成完成。
- 向量库 / 图数据库完成。

## 11. evidence / handoff 要求

执行完成后新增：

- `evidence/2026-06-03-memory-layer-m1-formal-memory-store-and-audit-v1.md`
- `handoffs/2026-06-03-memory-layer-m1-formal-memory-store-and-audit-v1-result.md`

必须记录：

- 改动文件。
- store 路径。
- 创建 record / version / audit 的测试证据。
- 未做 M2 / M4 / M6 / M9 的边界。
- 验证命令结果。
- 是否改 `workflow-state.v0.json`。
- 是否执行真实 Codex。
- 是否读写 `/Users/yoyi/.codex`。

## 12. Stop 条件

遇到以下情况必须停下回传：

- 需要从候选自动升级正式记忆。
- 需要实现候选采纳正式记忆。
- 需要把正式记忆注入 worker 任务包。
- 需要改 workflow state JSON 结构。
- 需要数据库迁移。
- 需要读写 `/Users/yoyi/.codex`。
- 需要执行真实 Codex。
- 需要接 Obsidian / 向量库 / 图数据库。
- 发现 `docs/memory-layer-design-v1.md` 和本任务包冲突。
