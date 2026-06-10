# 黑板候选持久状态 Schema v1

日期：2026-06-03

状态：schema / 迁移计划补充版已确认。用户暂不允许进入 `final-skeleton-11-blackboard-candidate-persistence-implementation-v1`。

## 先说风险

- 当前 `ProjectBlackboard` 是从 workflow 读模型派生出来的只读黑板，不是持久事实层。
- 如果把候选确认直接写成正式事实、正式记忆、权限决定或 workflow 状态变化，会越过控制核心边界。
- 现有 `BlackboardEntry.entry_id` 是派生读模型 ID，未来派生规则变动时可能变化；持久确认状态不能只依赖这个字段。

依据：

- `tasks/2026-06-01-final-workbench-skeleton-execution-package-v1.md` 的 Skeleton-10 要求只设计 schema，不实现写入。
- `evidence/2026-06-01-project-blackboard-minimal-read-model-d-v1.md` 明确当前黑板是读模型，不写 workflow state JSON，不写正式记忆。
- `prototypes/productized-desktop-shell/src/lib/types.ts` 现有 `ProjectBlackboard` / `BlackboardEntry` / `BlackboardPromotionDecision` 只表达候选和升级状态。

## 本轮结论

本 schema 只定义黑板候选的持久确认状态。

它不改变：

- workflow state JSON。
- 工作流状态机。
- 正式事实层。
- 正式记忆层。
- 权限确认命令。
- 项目画布右侧栏布局。

建议后续最小实现采用独立候选状态存储，逻辑 schema 固定为 `blackboard_candidate_persistence.v1`。第一版可以是 sidecar JSON，未来再迁入 SQLite；但 Skeleton-10 本轮不创建文件、不迁移数据库。

用户已确认：

- 接受 `blackboard_candidate_persistence.v1` 的方向。
- 接受第一版用独立 sidecar JSON，不写 workflow state JSON。
- 接受 `candidate_confirmed_for_followup` 只表示候选层确认，不做正式晋升。
- 接受 sidecar JSON 路径和作用域。
- 接受原子写入、备份、lock、revision 并发冲突处理。
- 接受 `candidate_key` 稳定生成规则。
- 接受版本字段。
- 接受 `rejected` / `discarded` 后再次出现不自动恢复。

用户暂未允许：

- 进入 `final-skeleton-11`。
- 实现黑板候选写入。

## Sidecar 文件路径和作用域

第一版 sidecar JSON 建议与当前 `workflow-state.v0.json` 同目录，文件名固定为：

```text
<workflow_state_dir>/blackboard-candidates.v1.json
```

示例：

```text
.../workflow-state.v0.json
.../blackboard-candidates.v1.json
.../backups/blackboard-candidates.v1.<timestamp>.<revision>.json
.../.blackboard-candidates.v1.<timestamp>.<write_id>.tmp
.../.blackboard-candidates.v1.lock
```

作用域：

- 一个 `workflow-state.v0.json` 对应一个 `blackboard-candidates.v1.json`。
- sidecar 内可保存多个 project / workflow 的候选状态。
- sidecar 不保存 transcript 正文。
- sidecar 不保存 auth、token、密钥或 `.env` 内容。
- sidecar 不写入 `/Users/yoyi/.codex`。

路径推导规则：

```text
sidecar_path = workflow_state_path.parent().join("blackboard-candidates.v1.json")
backup_dir = workflow_state_path.parent().join("backups")
lock_path = workflow_state_path.parent().join(".blackboard-candidates.v1.lock")
```

边界：

- sidecar 是候选状态文件，不是 workflow state 文件。
- sidecar 的存在不能改变 `workflow-state.v0.json` 的 schema。
- 如果未来迁入 SQLite，必须另开迁移计划。

## 现有输入

只允许从现有黑板读模型读取候选来源：

| 来源 | 现有 kind | 说明 |
|---|---|---|
| 子智能体汇报 | `subagent_report` | 可作为 workflow fact 候选，但不能直接写正式事实。 |
| 风险 | `risk` | 可作为 workflow risk 候选，但不能直接推进 workflow state。 |
| 权限请求 | `permission_request` | 只能引用权限请求，不能替代权限批准 / 拒绝命令。 |
| 工具摘要 | `tool_summary` | 只能作为工具摘要候选，不能直接写 workflow state。 |
| 记忆候选 | `memory_candidate` | 只能作为正式记忆候选，不能直接写正式记忆。 |
| 知识引用 | `knowledge_ref` | 只能作为知识引用候选，不能直接当成记忆。 |

不读取：

- Codex transcript 全文。
- `/Users/yoyi/.codex`。
- auth、token、`.env`、密钥。
- 独立 `CanvasView` 文件层。
- MCP canvas run 状态。

## 候选状态枚举

```ts
type BlackboardCandidateState =
  | "candidate_pending_control_core"
  | "candidate_confirmed_for_followup"
  | "candidate_rejected"
  | "candidate_deferred"
  | "candidate_discarded";
```

状态含义：

| 状态 | 含义 | 不等于 |
|---|---|---|
| `candidate_pending_control_core` | 候选已出现，但还没被控制核心处理。 | 不等于已接受。 |
| `candidate_confirmed_for_followup` | 控制核心确认该候选值得保留或进入后续专门流程。 | 不等于写正式事实、正式记忆、权限批准或 workflow 状态推进。 |
| `candidate_rejected` | 控制核心拒绝该候选。 | 不删除来源读模型。 |
| `candidate_deferred` | 控制核心决定暂缓处理。 | 不等于接受或拒绝。 |
| `candidate_discarded` | 候选因来源过期、重复、被替代或不再适用而废弃。 | 不等于正式删除审计。 |

## 候选目标类型

```ts
type BlackboardCandidateTargetKind =
  | "workflow_fact"
  | "workflow_risk"
  | "permission_decision"
  | "audit_event"
  | "formal_memory"
  | "knowledge_reference"
  | "no_promotion";
```

目标类型说明：

| target_kind | 允许作为候选吗 | Skeleton-11 是否能直接写目标 |
|---|---:|---:|
| `workflow_fact` | 是 | 否 |
| `workflow_risk` | 是 | 否 |
| `permission_decision` | 是 | 否，必须走权限确认命令。 |
| `audit_event` | 是 | 只能写黑板候选状态审计；不能伪装成全局正式 audit。 |
| `formal_memory` | 是 | 否，必须另走记忆治理计划。 |
| `knowledge_reference` | 是 | 否，只能保留引用。 |
| `no_promotion` | 是 | 否，只表示不晋升。 |

## 逻辑 Schema

```ts
type BlackboardCandidatePersistenceStore = {
  schema_version: "blackboard_candidate_persistence.v1";
  store_version: 1;
  storage_kind: "sidecar_json_v0" | "sqlite_future";
  scope: BlackboardCandidateStoreScope;
  revision: number;
  last_write_id?: string | null;
  generated_by: "control_core";
  created_at: string;
  updated_at: string;
  records: BlackboardCandidateRecord[];
  audit_events: BlackboardCandidateAuditEvent[];
  warnings: string[];
};
```

```ts
type BlackboardCandidateStoreScope = {
  scope_kind: "workflow_state_sidecar";
  workflow_state_path?: string | null;
  sidecar_path?: string | null;
  project_roots: string[];
};
```

```ts
type BlackboardCandidateRecord = {
  record_version: 1;
  candidate_id: string;
  candidate_key: string;
  candidate_key_version: 1;
  content_fingerprint: string;
  source_entry_id?: string | null;

  project_id: string;
  project_root: string;
  workflow_id: string;
  work_item_id?: string | null;
  workflow_node_id?: string | null;

  entry_kind: BlackboardEntryKind;
  target_kind: BlackboardCandidateTargetKind;
  state: BlackboardCandidateState;

  title_snapshot: string;
  summary_snapshot: string;
  source_status?: string | null;
  source_refs: BlackboardCandidateSourceRef[];

  decision: BlackboardCandidateDecision;
  created_at: string;
  updated_at: string;
  last_seen_at?: string | null;
  appearance_count: number;
  superseded_by_candidate_id?: string | null;
  audit_refs: string[];
  warnings: string[];
};
```

```ts
type BlackboardEntryKind =
  | "subagent_report"
  | "risk"
  | "permission_request"
  | "tool_summary"
  | "memory_candidate"
  | "knowledge_ref";
```

```ts
type BlackboardCandidateSourceRef = {
  source_kind:
    | "subagent_report"
    | "risk"
    | "permission_request"
    | "tool_summary"
    | "memory_candidate"
    | "knowledge_ref"
    | "workflow"
    | "workflow_node"
    | "work_item"
    | "dispatch"
    | "director_review"
    | "ledger_entry"
    | "task_package"
    | "evidence_ref"
    | "handoff_ref";
  source_id: string;
  label: string;
};
```

```ts
type BlackboardCandidateDecision = {
  decision_version: 1;
  decision_id: string;
  decided_by_role: "project_director" | "control_core" | "user" | "system";
  decided_by_session_id?: string | null;
  decision_reason: string;
  decided_at: string;
  requested_state: BlackboardCandidateState;
  resulting_state: BlackboardCandidateState;
  promotion_target_blocked: boolean;
  followup_required: boolean;
  followup_task_ref?: string | null;
};
```

## 候选身份规则

`candidate_id` 是持久记录 ID。

`candidate_key` 是用于把派生黑板条目和持久状态对齐的稳定键。

生成公式：

```text
candidate_key = "bbcand:v1:" + sha256(
  normalize(project_id) + "\0" +
  normalize(workflow_id) + "\0" +
  normalize(entry_kind) + "\0" +
  normalize(target_kind) + "\0" +
  normalize_source_refs(source_refs)
)
```

`normalize_source_refs(source_refs)` 规则：

- 每个 ref 只使用 `source_kind` 和 `source_id`。
- 不使用 `label`，因为 label 是展示文本，可能变化。
- 不使用 `title_snapshot`、`summary_snapshot`、`status`、`warnings`。
- `source_kind` 和 `source_id` 做首尾空白裁剪。
- 文件路径类 `source_id` 统一使用 `/` 分隔。
- 去重后按 `source_kind + "\0" + source_id` 字典序排序。
- 使用 `"\n"` 拼接排序后的 ref。

`content_fingerprint` 生成公式：

```text
content_fingerprint = "bbcand-content:v1:" + sha256(
  normalize(title_snapshot) + "\0" +
  normalize(summary_snapshot) + "\0" +
  normalize(source_status ?? "") + "\0" +
  normalize_source_refs(source_refs)
)
```

用途：

- `candidate_key` 用来判断是不是同一个逻辑候选。
- `content_fingerprint` 用来判断同一个候选来源是否出现内容变化。

规则：

- `source_entry_id` 可以保存现有 `BlackboardEntry.entry_id`，但不能作为唯一身份。
- 派生标题和摘要只保存 snapshot，用于人工复核；后续来源变化时不能反向改写原来源。
- 如果同一个 `candidate_key` 已存在，只更新状态，不新增重复记录。
- 如果来源语义变更但 `source_refs` 不变，保留旧记录并追加 warning，不能静默覆盖人工决定。
- 如果 `source_refs` 缺失，控制核心命令必须拒绝写入。
- 如果 `entry_kind` / `target_kind` 不在枚举中，必须拒绝。
- 如果生成 `candidate_key` 时发现规范化后为空，必须拒绝。

## rejected / discarded 后再次出现

再次出现定义：

- 派生黑板读模型中再次出现同一个 `candidate_key`。

处理规则：

| 已有状态 | 再次出现条件 | 默认处理 |
|---|---|---|
| `candidate_rejected` | `candidate_key` 相同，`content_fingerprint` 相同 | 保持 rejected，不自动回到 pending；读模型展示“已拒绝但来源仍出现”。 |
| `candidate_rejected` | `candidate_key` 相同，`content_fingerprint` 不同 | 保持 rejected，追加 transient warning：`source_content_changed_after_rejection`；需要人工重新标记 pending。 |
| `candidate_discarded` | `candidate_key` 相同，`content_fingerprint` 相同 | 保持 discarded，不自动恢复。 |
| `candidate_discarded` | `candidate_key` 相同，`content_fingerprint` 不同 | 保持 discarded，追加 transient warning：`source_content_changed_after_discard`；需要人工重新标记 pending。 |
| 任意终态 | `source_refs` 改变导致 `candidate_key` 变化 | 视为新候选。 |

重新打开规则：

- 只有用户或控制核心显式执行 `candidate_pending_control_core`，并提供 reason，才能把 rejected / discarded 候选重新置为 pending。
- 重新打开必须写审计事件，`before_state` 记录原终态。
- 重新打开不写正式事实、正式记忆、权限决定或 workflow state。
- 读模型发现再次出现时不能为了更新 `last_seen_at` 自动写 sidecar；`last_seen_at` / `appearance_count` 只能在显式候选状态写入时更新。

## 审计事件

```ts
type BlackboardCandidateAuditEvent = {
  event_version: 1;
  event_id: string;
  event_type:
    | "blackboard_candidate_pending_recorded"
    | "blackboard_candidate_confirmed"
    | "blackboard_candidate_rejected"
    | "blackboard_candidate_deferred"
    | "blackboard_candidate_discarded";
  candidate_id: string;
  candidate_key: string;
  project_id: string;
  workflow_id: string;
  actor_role: "project_director" | "control_core" | "user" | "system";
  actor_session_id?: string | null;
  before_state?: BlackboardCandidateState | null;
  after_state: BlackboardCandidateState;
  store_revision: number;
  reason: string;
  created_at: string;
  source_refs: BlackboardCandidateSourceRef[];
  warnings: string[];
};
```

事件含义：

| event_type | 写入时机 |
|---|---|
| `blackboard_candidate_pending_recorded` | 第一次为派生候选建立持久状态。 |
| `blackboard_candidate_confirmed` | 控制核心确认候选可保留或进入后续专门流程。 |
| `blackboard_candidate_rejected` | 控制核心拒绝候选。 |
| `blackboard_candidate_deferred` | 控制核心暂缓候选。 |
| `blackboard_candidate_discarded` | 控制核心废弃候选。 |

## 读模型叠加规则

后续实现时，项目黑板读模型仍从 workflow state 派生。

持久候选状态只作为 overlay：

1. 从 workflow state 派生 `ProjectBlackboard.entries[]`。
2. 为每个 `BlackboardEntry` 计算 `candidate_key`。
3. 从候选持久 store 查找同 key 的 `BlackboardCandidateRecord`。
4. 如果找到，把 `record.state` 映射到 `promotion_decision.status`。
5. 如果找不到，继续显示现有默认状态 `candidate_pending_control_core`。
6. 如果找到终态记录，并且来源再次出现，只添加 transient warning，不自动写 sidecar。

禁止：

- overlay 不能反向写 workflow state JSON。
- overlay 不能改变 work item / workflow node 状态。
- overlay 不能写正式记忆。
- overlay 不能把权限请求变成已批准。

## 控制核心命令签名草案

后续 Skeleton-11 只允许实现候选状态闭环，命令签名建议如下：

```ts
type RecordBlackboardCandidateDecisionInput = {
  project_id: string;
  project_root: string;
  workflow_id: string;
  candidate_key: string;
  source_entry_id?: string | null;
  entry_kind: BlackboardEntryKind;
  target_kind: BlackboardCandidateTargetKind;
  requested_state: BlackboardCandidateState;
  reason: string;
  actor_role: "project_director" | "control_core" | "user";
  source_refs: BlackboardCandidateSourceRef[];
  expected_store_revision?: number | null;
};

type RecordBlackboardCandidateDecisionOutput = {
  record: BlackboardCandidateRecord;
  audit_event: BlackboardCandidateAuditEvent;
  store_revision: number;
  warnings: string[];
};
```

建议命令名：

```text
record_blackboard_candidate_decision(input)
```

后端校验：

- `requested_state` 只能是本 schema 的状态。
- `candidate_confirmed_for_followup` 只改变候选状态，不写目标层。
- `target_kind=permission_decision` 时必须提示“后续权限决定走权限确认命令”。
- `target_kind=formal_memory` 时必须提示“后续进入记忆治理，不写正式记忆”。
- `target_kind=workflow_fact` / `workflow_risk` 时必须提示“后续需要事实晋升计划，不改 workflow state”。
- 缺少 `reason` 时拒绝。
- 缺少 `source_refs` 时拒绝。
- 如果传入 `expected_store_revision` 且和当前 store revision 不一致，拒绝并返回并发冲突。

## 原子写入、备份和并发冲突

后续 Skeleton-11 如果获准实现，sidecar 写入必须按以下顺序：

1. 推导 `sidecar_path`、`backup_dir`、`lock_path`。
2. 用独占创建方式创建 `lock_path`；如果 lock 已存在且未过期，拒绝本次写入。
3. 读取 sidecar；如果不存在，按空 store 初始化。
4. 校验 `schema_version`、`store_version`、`revision`。
5. 如果请求带 `expected_store_revision`，必须和当前 `revision` 相同。
6. 写入前如 sidecar 已存在，先复制到：

```text
<workflow_state_dir>/backups/blackboard-candidates.v1.<timestamp>.<revision>.json
```

7. 在内存里生成新 store，`revision = old_revision + 1`，`updated_at` 更新，`last_write_id` 更新。
8. 写临时文件：

```text
<workflow_state_dir>/.blackboard-candidates.v1.<timestamp>.<write_id>.tmp
```

9. 对临时文件执行写入和 sync。
10. 用 rename 原子替换 `sidecar_path`。
11. 尽力 sync 父目录。
12. 删除 lock。

冲突处理：

| 情况 | 处理 |
|---|---|
| lock 已存在且未过期 | 返回 `blackboard_candidate_store_locked`，不写入。 |
| lock 已过期 | 返回 warning，允许控制核心清理后重试；第一版不自动吞掉冲突。 |
| `expected_store_revision` 不匹配 | 返回 `blackboard_candidate_store_conflict`，UI 需要重读再提交。 |
| 备份失败 | 拒绝写入。 |
| 临时文件写入失败 | 删除临时文件，保留原 sidecar。 |
| rename 失败 | 保留原 sidecar，返回错误。 |
| 进程崩溃留下临时文件 | 下次读取时忽略临时文件，可在持锁后清理过期 tmp。 |

备份保留策略：

- 第一版至少保留最近 20 个 `blackboard-candidates.v1.*.json` 备份。
- 备份清理只能清理候选 sidecar 备份，不能清理 workflow state 备份。

## 迁移计划

### Skeleton-10 本轮

不迁移。

本轮只写 schema 和计划，不创建持久文件，不改数据库，不改 workflow state JSON。

### Skeleton-11 允许的最小迁移

前提：用户明确确认本 schema / 迁移计划，并明确允许进入 Skeleton-11。

建议迁移步骤：

1. 新增独立候选状态存储，不写入 workflow state JSON。
2. 如果候选状态存储不存在，创建空 store：

```json
{
  "schema_version": "blackboard_candidate_persistence.v1",
  "store_version": 1,
  "storage_kind": "sidecar_json_v0",
  "scope": {
    "scope_kind": "workflow_state_sidecar",
    "workflow_state_path": null,
    "sidecar_path": null,
    "project_roots": []
  },
  "revision": 0,
  "last_write_id": null,
  "generated_by": "control_core",
  "created_at": "<iso8601>",
  "updated_at": "<iso8601>",
  "records": [],
  "audit_events": [],
  "warnings": []
}
```

3. 不批量回填所有现有黑板候选。
4. 用户第一次对某个候选执行 pending / confirm / reject / defer / discard 时，才写入对应 `BlackboardCandidateRecord`。
5. 读模型只用 overlay 合并候选状态。
6. 测试必须证明 workflow state JSON 未被新增黑板候选持久字段。
7. 测试必须覆盖原子写入、备份、revision 冲突和 rejected / discarded 再次出现规则。

### 后续 SQLite 迁移

SQLite 迁移不属于 Skeleton-11 默认范围。

如果后续要迁入 SQLite，必须另开计划：

- 定义表结构。
- 写从 sidecar JSON 到 SQLite 的一次性迁移。
- 保留回滚方案。
- 补迁移测试。
- 仍不能把候选状态写进 workflow state JSON。

## Skeleton-11 实现任务草案

任务名：

- `final-skeleton-11-blackboard-candidate-persistence-implementation-v1`

前置：

- Skeleton-10 schema / 迁移计划已被用户明确接受。
- 用户明确允许“按已确认 schema / 迁移计划实现黑板候选持久状态最小闭环”。

目标：

- 实现 `record_blackboard_candidate_decision`。
- 建立独立候选状态 store。
- 让读模型用候选状态 overlay 更新 `promotion_decision.status`。
- UI 只展示候选状态和必要确认入口，不新增右侧栏主面板。
- 补测试证明候选状态变化不会写正式事实、正式记忆或 workflow state。

禁止：

- 不改 workflow state JSON 结构。
- 不直接写正式事实。
- 不直接写正式记忆。
- 不推进 workflow / work item / node 状态。
- 不把权限请求直接标为批准或拒绝。
- 不执行真实 Codex。
- 不读写 `/Users/yoyi/.codex`。
- 不启动 MCP canvas run。
- 不往项目画布右侧栏继续堆新主面板。

建议改动点：

- 后端：新增候选状态类型、store 读写 helper、控制核心命令和审计事件构造。
- 后端：sidecar 路径固定由 workflow state 路径推导，不允许落到 `/Users/yoyi/.codex`。
- 后端：写入必须实现 lock、revision、backup、tmp + rename 原子替换和冲突返回。
- 读模型：把候选持久状态 overlay 到 `ProjectBlackboard.entries[]`。
- 前端：在既有右侧节点详情 / 黑板候选区展示候选状态和必要确认入口。
- 测试：新增 Rust store / command 测试和前端离线交互测试。

必跑验证：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
cargo test --lib
```

验收：

- pending / confirm / reject / defer / discard 都能写候选状态和候选审计。
- confirm 只表示候选层确认，不写正式事实或正式记忆。
- workflow state JSON 没有新增黑板候选持久字段。
- UI 没有新增右侧栏主面板，只在既有候选详情里展示状态。
- sidecar 路径、原子写入、备份、revision 冲突、rejected / discarded 再次出现规则都有测试。

## 用户确认清单

请确认：

1. 是否接受 `blackboard_candidate_persistence.v1` 作为黑板候选持久状态 schema。
2. 是否接受第一版使用独立 sidecar JSON，不写 workflow state JSON。
3. 是否接受 `candidate_confirmed_for_followup` 只表示候选层确认，不做正式晋升。
4. 是否允许下一步进入 Skeleton-11 的最小实现。

在 1-4 都确认前，不能执行 Skeleton-11。
