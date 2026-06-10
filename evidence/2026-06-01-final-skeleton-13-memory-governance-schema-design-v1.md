# Evidence：final-skeleton-13 记忆治理 schema 设计 v1

## 1. 结论

已完成 `final-skeleton-13-memory-governance-schema-design-v1` 的设计文档。

新增：

- `docs/plans/2026-06-01-memory-governance-schema-v1.md`

本轮只写 schema 和 Skeleton-14 草案，没有改产品代码。

## 2. 薄弱点

- 这不是完整记忆层实现。
- `MemoryRecord` 只是目标形状，本轮没有也不能写正式长期记忆。
- 候选生命周期还没有实现，`memory-candidates.v1.json` 只是 Skeleton-14 的建议。
- 用户偏好记忆优先级已经定义，但没有接入秘书、任务包生成或 agent 上下文。
- 知识库 / Obsidian-like 能力仍然只是边界定义，没有原生接入。
- 向量库、图数据库、SQLite、FTS、召回算法都没有实现。

## 3. 依据

已只读参考：

- `tasks/2026-06-01-final-workbench-skeleton-execution-package-v1.md`
- `docs/memory-layer-design-v1.md`
- `docs/workbench-system-architecture-v1.md`
- `docs/plans/2026-06-01-blackboard-candidate-persistence-schema-v1.md`
- `CURRENT.md`
- `tasks/README.md`

## 4. 完成内容

计划文档定义了：

- `memory_governance.v1`
- `MemoryCandidate`
- `MemoryRecord`
- `MemoryScope`
- `MemorySourceRef`
- `MemoryLifecycleStatus`
- `MemoryConflict`
- `MemoryAuditRef`
- 用户偏好记忆优先级和确认规则
- 知识库与记忆层边界
- Skeleton-14 候选生命周期最小实现草案

## 5. 关键边界

- `candidate_confirmed` 只表示候选被确认保留，不表示写入正式记忆。
- 正式记忆 `MemoryRecord` 不在本轮实现，也不由本轮授权。
- 知识库材料不能自动变成记忆。
- Obsidian CLI 或 Obsidian-like 功能不能绕过记忆治理状态机。
- 秘书可以提出候选，但不能直接写正式记忆。
- 黑板候选不能直接升级成记忆候选，必须经控制核心生成新的 `MemoryCandidate`。

## 6. Skeleton-14 草案边界

Skeleton-14 只有在用户明确确认后才能开始。

建议：

- 使用独立 sidecar：`<workflow_state_dir>/memory-candidates.v1.json`
- 不改 `workflow-state.v0.json` 结构。
- 不迁移数据库。
- 不写正式 `MemoryRecord`。
- 不接向量库、图数据库或 Obsidian 原生读写。

用户只确认 schema 方向，不等于允许进入 Skeleton-14。

## 7. 未做

- 未改前端代码。
- 未改后端代码。
- 未新增测试。
- 未跑代码测试，因为本轮没有改产品代码。
- 未迁移数据库。
- 未写 workflow state。
- 未写黑板候选 sidecar。
- 未写记忆候选 sidecar。
- 未写正式记忆。
- 未执行真实 Codex。
- 未读写 `/Users/yoyi/.codex`。
- 未接 Claude / OpenClaw / OpenCode。
- 未接 Obsidian。

## 8. 只读验收命令

```text
rg -n "MemoryCandidate|MemoryRecord|MemoryScope|MemorySourceRef|MemoryLifecycleStatus|MemoryConflict|MemoryAuditRef|candidate_confirmed|memory-candidates.v1.json|只确认 1-3 不等于允许开始 4" docs/plans/2026-06-01-memory-governance-schema-v1.md
```

预期：

- 能找到七个核心对象。
- 能找到 `candidate_confirmed` 的候选语义。
- 能找到 `memory-candidates.v1.json` 只是 Skeleton-14 建议。
- 能找到用户确认闸门。

## 9. 当前状态建议

下一步应该停在用户确认：

1. 是否接受 `memory_governance.v1`。
2. 是否接受 `candidate_confirmed` 只表示候选确认。
3. 是否接受 Skeleton-14 第一版建议用独立 `memory-candidates.v1.json` sidecar。
4. 是否允许进入 Skeleton-14。

只确认 1-3 不等于允许开始 4。
