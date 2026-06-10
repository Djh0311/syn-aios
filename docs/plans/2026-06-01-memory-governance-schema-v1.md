# Memory Governance Schema v1

状态：草案，等待用户确认。  
对应任务：`final-skeleton-13-memory-governance-schema-design-v1`。  
本文件只定义记忆治理最小 schema 和下一步实现草案，不实现记忆写入。

## 1. 先说薄弱点

- 这不是完整记忆层实现，只是最终工作台骨架里的治理对象 schema。
- 正式记忆 `MemoryRecord` 这里只定义形状，不允许写入。
- `approved` 这类词容易被误读成“已经写入正式记忆”，本版本不用它表示候选采纳。
- 记忆候选和黑板候选是两套东西：黑板候选接项目协作中间态，记忆候选接长期行为依据。
- 知识库和 Obsidian-like 能力不在本任务内；知识库材料只能作为来源，不能自动变成记忆。
- 向量库、图数据库、相似度检索、理解图、SQLite 表结构都后置。

## 2. 依据

- `docs/memory-layer-design-v1.md`
- `docs/workbench-system-architecture-v1.md`
- `tasks/2026-06-01-final-workbench-skeleton-execution-package-v1.md`
- `docs/plans/2026-06-01-blackboard-candidate-persistence-schema-v1.md`

已定边界：

- 记忆治理可以进入核心，具体检索和存储实现不进入核心。
- 普通聊天不自动进入长期记忆。
- 知识库是材料和思考空间，不是系统行为依据。
- 记忆是会影响 agent 行为的确认事实。
- 候选记忆不能当正式记忆用。
- 正式记忆必须有来源、版本、状态、权限和审计。
- 用户偏好记忆优先级最高，但更需要明确确认。

## 3. 本版本目标

定义最小治理对象：

- `MemoryCandidate`
- `MemoryRecord`
- `MemoryScope`
- `MemorySourceRef`
- `MemoryLifecycleStatus`
- `MemoryConflict`
- `MemoryAuditRef`

并定义：

- 用户偏好记忆的优先级。
- 用户偏好记忆的确认规则。
- Skeleton-14 的实现任务草案。

## 4. 本版本不做

- 不接向量库。
- 不接图数据库。
- 不接 Obsidian 原生读写。
- 不接知识库自动扫描。
- 不把聊天自动写成长期记忆。
- 不写正式 `MemoryRecord`。
- 不把“采纳候选”解释成“写入正式记忆”。
- 不改 `workflow state JSON`。
- 不实现 SQLite 迁移。
- 不让秘书直接写正式记忆。

## 5. Schema 总览

Schema 名：

- `memory_governance.v1`

对象分层：

| 层级 | 对象 | 是否正式记忆 | 本轮是否实现 |
| --- | --- | --- | --- |
| 候选层 | `MemoryCandidate` | 否 | 否，只设计 |
| 正式层 | `MemoryRecord` | 是 | 否，只设计 |
| 范围层 | `MemoryScope` | 否 | 否，只设计 |
| 来源层 | `MemorySourceRef` | 否 | 否，只设计 |
| 生命周期 | `MemoryLifecycleStatus` | 否 | 否，只设计 |
| 冲突层 | `MemoryConflict` | 否 | 否，只设计 |
| 审计引用 | `MemoryAuditRef` | 否 | 否，只设计 |

## 6. MemoryScope

用途：

- 定义一条候选或正式记忆能被谁读取、修改和用于任务包。
- 防止用户偏好、全局蓝图、项目事实、会话摘要和知识库材料混在一起。

字段：

```ts
type MemoryScope = {
  scope_id: string;
  scope_type:
    | "user_preference"
    | "global"
    | "project"
    | "workflow"
    | "session"
    | "role_limited"
    | "document_limited";
  user_id?: string;
  project_id?: string;
  workflow_id?: string;
  session_id?: string;
  role_ids: string[];
  document_refs: string[];
  permission_policy_ref?: string;
  model_export_policy: "local_only" | "allowed_with_redaction" | "blocked";
  valid_from: string;
  valid_until?: string;
};
```

规则：

- `user_preference` 只能由用户确认或用户明确授权的流程确认。
- `global` 影响所有项目，必须用户确认。
- `project` 只能影响对应项目。
- `workflow` 和 `session` 默认不能上升为项目或全局记忆。
- `document_limited` 不能脱离原始文档权限。
- `model_export_policy = blocked` 的记忆不能进入外发模型上下文。

## 7. MemorySourceRef

用途：

- 记录候选或正式记忆来自哪里。
- 防止“没有来源的长期结论”进入系统行为依据。

字段：

```ts
type MemorySourceRef = {
  source_ref_id: string;
  source_type:
    | "user_confirmed_proposal"
    | "workflow_summary"
    | "stage_report"
    | "director_review"
    | "handoff"
    | "evidence"
    | "audit_event"
    | "session_summary"
    | "knowledge_doc"
    | "observation_ref"
    | "manual_note";
  source_id?: string;
  source_path?: string;
  source_title?: string;
  anchor?: string;
  source_created_at?: string;
  captured_at: string;
  authority_level:
    | "user_confirmed"
    | "current_authority_doc"
    | "audit"
    | "evidence"
    | "handoff"
    | "derived_summary"
    | "knowledge_material"
    | "unverified_note";
  sensitive_level: "public" | "project" | "private" | "secret";
  content_hash?: string;
};
```

规则：

- 正式记忆至少要有一个 `source_refs[]`。
- `knowledge_doc` 只是知识库材料来源，不等于记忆已经成立。
- `derived_summary` 只能生成候选，不能直接写正式记忆。
- `secret` 来源不能进入外发模型上下文。
- 来源被撤权后，相关候选和正式记忆必须进入复核或冲突状态。

## 8. MemoryCandidate

用途：

- 保存“可能应该长期记住，但还没有成为正式记忆”的内容。
- 接住工作流总结、用户偏好、秘书整理、知识库摘要、项目主管总结。

字段：

```ts
type MemoryCandidate = {
  candidate_id: string;
  candidate_key: string;
  schema_version: "memory_governance.v1";
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
  generated_by_role:
    | "user"
    | "secretary"
    | "project_director"
    | "global_director"
    | "review_agent"
    | "system";
  generated_from:
    | "explicit_user_confirmation"
    | "workflow_closeout"
    | "stage_handoff"
    | "secretary_suggestion"
    | "knowledge_summary"
    | "manual_entry";
  status: MemoryLifecycleStatus;
  risk_level: "low" | "medium" | "high";
  sensitive_level: "public" | "project" | "private" | "secret";
  requires_user_confirmation: boolean;
  review_reason: string;
  conflicts: MemoryConflict[];
  audit_refs: MemoryAuditRef[];
  created_at: string;
  updated_at: string;
};
```

`candidate_key` 稳定生成建议：

```text
memcand:v1:sha256(scope_type + scope ids + memory_type + normalized claim + source refs)
```

规则：

- `candidate_confirmed` 只表示候选被确认保留，不表示已经写入正式记忆。
- 用户偏好、全局蓝图、跨项目影响的候选必须 `requires_user_confirmation = true`。
- 普通聊天摘要只能进入候选，不能自动进入正式记忆。
- 知识库摘要只能进入候选，不能自动进入正式记忆。
- 黑板候选不能直接升级成记忆候选；必须经过控制核心生成新的 `MemoryCandidate`。

## 9. MemoryRecord

用途：

- 定义正式长期记忆的目标形状。
- 本版本不允许创建、更新或删除正式 `MemoryRecord`。

字段：

```ts
type MemoryRecord = {
  memory_id: string;
  schema_version: "memory_governance.v1";
  record_version: number;
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
  status: MemoryLifecycleStatus;
  supersedes_memory_id?: string;
  superseded_by_memory_id?: string;
  conflict_refs: string[];
  audit_refs: MemoryAuditRef[];
  created_at: string;
  updated_at: string;
};
```

硬规则：

- 本 schema 不授权写 `MemoryRecord`。
- 正式记忆修改不能覆盖旧版本，只能创建新版本。
- 正式记忆必须有来源。
- 正式记忆变更必须有审计。
- `conflicted`、`deprecated`、`frozen`、`archived` 默认不进入任务包。

## 10. MemoryLifecycleStatus

本版本把候选状态和正式记忆状态放在一个类型里，但用前缀区分，避免误读。

```ts
type MemoryLifecycleStatus =
  | "candidate_draft"
  | "candidate_needs_review"
  | "candidate_confirmed"
  | "candidate_rejected"
  | "candidate_quarantined"
  | "candidate_superseded"
  | "candidate_discarded"
  | "memory_active"
  | "memory_conflicted"
  | "memory_deprecated"
  | "memory_frozen"
  | "memory_archived";
```

候选允许流转：

- `candidate_draft -> candidate_needs_review`
- `candidate_needs_review -> candidate_confirmed`
- `candidate_needs_review -> candidate_rejected`
- `candidate_needs_review -> candidate_quarantined`
- `candidate_quarantined -> candidate_needs_review`
- `candidate_confirmed -> candidate_superseded`
- `candidate_confirmed -> candidate_discarded`

正式记忆允许流转：

- `memory_active -> memory_conflicted`
- `memory_active -> memory_deprecated`
- `memory_active -> memory_frozen`
- `memory_active -> memory_archived`
- `memory_conflicted -> memory_active`
- `memory_conflicted -> memory_deprecated`
- `memory_frozen -> memory_active`
- `memory_frozen -> memory_deprecated`
- `memory_deprecated -> memory_archived`

禁止：

- `candidate_confirmed` 自动变 `memory_active`。
- `knowledge_doc` 自动变 `memory_active`。
- `session_summary` 自动变 `memory_active`。
- 秘书输出自动变 `memory_active`。

## 11. MemoryConflict

用途：

- 记录候选和候选、候选和正式记忆、正式记忆和当前代码或文档之间的冲突。
- 防止冲突记忆偷偷进入任务包。

字段：

```ts
type MemoryConflict = {
  conflict_id: string;
  conflict_type:
    | "claim_contradiction"
    | "scope_overlap"
    | "source_stale"
    | "permission_conflict"
    | "user_preference_override"
    | "code_or_doc_mismatch"
    | "duplicate_candidate";
  left_ref: string;
  right_ref: string;
  severity: "low" | "medium" | "high" | "blocking";
  status: "open" | "acknowledged" | "resolved" | "dismissed";
  summary: string;
  recommended_action:
    | "ask_user"
    | "keep_newer_user_confirmation"
    | "keep_project_scope"
    | "quarantine_candidate"
    | "mark_memory_conflicted"
    | "discard_duplicate";
  source_refs: MemorySourceRef[];
  audit_refs: MemoryAuditRef[];
  created_at: string;
  updated_at: string;
};
```

规则：

- `blocking` 冲突必须阻止进入任务包。
- 用户最新明确确认优先于旧偏好。
- 项目记忆不能自动覆盖全局或用户偏好。
- 代码或当前权威文档冲突时，记忆必须进入复核。

## 12. MemoryAuditRef

用途：

- 让记忆候选和正式记忆能链接到审计事件。
- 本对象只是引用，不替代审计账本。

字段：

```ts
type MemoryAuditRef = {
  audit_ref_id: string;
  audit_event_id?: string;
  event_type:
    | "memory_candidate_created"
    | "memory_candidate_status_changed"
    | "memory_candidate_conflict_detected"
    | "memory_record_created"
    | "memory_record_versioned"
    | "memory_record_status_changed"
    | "memory_recall_included"
    | "memory_recall_excluded";
  actor_id: string;
  actor_role: "user" | "secretary" | "project_director" | "system" | "agent";
  target_kind: "memory_candidate" | "memory_record" | "memory_conflict";
  target_id: string;
  before_status?: MemoryLifecycleStatus;
  after_status?: MemoryLifecycleStatus;
  reason: string;
  created_at: string;
};
```

规则：

- 影响未来召回的动作必须有审计引用。
- 审计记录不能被 Markdown 或知识库编辑覆盖。
- 任务包纳入或排除记忆，也要能解释原因。

## 13. 用户偏好记忆规则

用户偏好记忆优先级最高，因为它会影响秘书、主管、建议方案、界面提醒和 agent 上下文选择。

优先级：

1. 用户当前明确指令。
2. 用户已确认的 `user_preference` 正式记忆。
3. 用户已确认的 `user_preference` 候选。
4. 项目内显式偏好。
5. 从行为推断出的偏好候选。

确认规则：

- 用户偏好不能只靠一次行为自动成立。
- 用户偏好影响多个项目时必须用户确认。
- 和用户当前指令冲突时，旧偏好必须让位。
- 从会话里提取的偏好只能先成为 `MemoryCandidate`。
- 秘书可以提出偏好候选，但不能写正式偏好。
- 敏感偏好不能进入外发模型上下文，除非用户单独授权。

## 14. 和知识库的关系

知识库：

- 保存材料、草稿、笔记、图谱、Canvas、Bases、Obsidian-compatible vault。
- 可以被编辑。
- 不自动改变 agent 行为。

记忆层：

- 决定哪些内容能成为系统行为依据。
- 必须有来源、状态、版本、权限和审计。
- 会影响任务包和 agent 上下文。

从知识库到记忆：

1. 知识库材料被用户或 agent 阅读。
2. 生成摘要或结论。
3. 进入 `MemoryCandidate`。
4. 经过确认、冲突处理、权限检查。
5. 未来单独授权后才可能写正式 `MemoryRecord`。

硬规则：

- 知识库内容不自动变成记忆。
- Obsidian CLI 不能直接写正式记忆。
- Canvas、Graph、Bases 是知识组织视图，不是正式记忆。
- 记忆详情可以反向链接知识库来源。

## 15. Skeleton-14 实现任务草案

任务名：

- `final-skeleton-14-memory-governance-minimal-implementation-v1`

前置：

- 用户确认本 schema。
- 用户明确授权“可以按已确认 schema 做记忆候选生命周期最小实现”。
- 如果用户只说“方向可以”，不能开始实现。

目标：

- 实现 `MemoryCandidate` 的最小生命周期。
- 实现候选创建、候选确认、候选冻结/隔离、候选废弃。
- 只写候选状态和候选审计引用，不写正式 `MemoryRecord`。

建议存储：

- 第一版建议独立 sidecar JSON：`<workflow_state_dir>/memory-candidates.v1.json`。
- 不写 `workflow-state.v0.json` 结构。
- 不写 SQLite。
- 不迁移数据库。

建议数据结构：

```ts
type MemoryCandidateStoreV1 = {
  store_version: "memory_candidate_store.v1";
  project_id?: string;
  workflow_id?: string;
  revision: number;
  candidates: MemoryCandidate[];
  events: MemoryAuditRef[];
  updated_at: string;
};
```

执行步骤草案：

1. 增加后端 / 前端类型。
2. 增加只读读模型。
3. 增加候选创建命令。
4. 增加候选状态变更命令。
5. 控制核心校验来源、作用域、权限和冲突。
6. 写候选审计引用。
7. UI 只做只读和必要确认入口，不往项目画布右侧栏堆新主面板。
8. 补离线测试。

禁止：

- 不写正式 `MemoryRecord`。
- 不把 `candidate_confirmed` 解释成正式记忆。
- 不接向量库。
- 不接图数据库。
- 不接 Obsidian 原生读写。
- 不把普通聊天自动写记忆。
- 不读写 `/Users/yoyi/.codex`。
- 不执行真实 Codex。

验收：

- 用户偏好可以作为候选被确认，但不会直接写成正式长期记忆。
- 工作流总结只能先成为候选。
- 知识库材料不是记忆。
- 冲突候选不能进入任务包依据。
- 如果出现正式记忆写入需求，任务必须停止并回传。

## 16. 需要用户确认的问题

1. 是否接受 `memory_governance.v1` 作为当前骨架 schema。
2. 是否接受候选状态使用 `candidate_confirmed`，避免误读为正式记忆。
3. 是否接受 Skeleton-14 第一版建议用独立 `memory-candidates.v1.json` sidecar。
4. 是否允许下一步进入 Skeleton-14 的候选生命周期最小实现。

只确认 1-3 不等于允许开始 4。
