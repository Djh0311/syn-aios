# Handoff：final-skeleton-13 记忆治理 schema 设计 v1

## 1. 状态

`final-skeleton-13-memory-governance-schema-design-v1` 已完成。

新增计划文档：

- `docs/plans/2026-06-01-memory-governance-schema-v1.md`

新增 evidence：

- `evidence/2026-06-01-final-skeleton-13-memory-governance-schema-design-v1.md`

## 2. 这轮做了什么

把最终记忆层设计收敛成当前骨架可用的最小治理 schema：

- `MemoryCandidate`
- `MemoryRecord`
- `MemoryScope`
- `MemorySourceRef`
- `MemoryLifecycleStatus`
- `MemoryConflict`
- `MemoryAuditRef`

同时写清：

- 用户偏好记忆优先级最高，但必须确认。
- 知识库材料不是记忆。
- 候选确认不是正式记忆写入。
- Skeleton-14 的最小实现草案只允许做候选生命周期。

## 3. 没做什么

- 没有实现记忆候选写入。
- 没有写正式长期记忆。
- 没有改产品代码。
- 没有改 workflow state JSON。
- 没有迁移数据库。
- 没有接向量库。
- 没有接图数据库。
- 没有接 Obsidian 原生读写。
- 没有执行真实 Codex。
- 没有读写 `/Users/yoyi/.codex`。

## 4. 风险

- `MemoryRecord` 已定义，但不能被误解为正式记忆已可写。
- Skeleton-14 里的 `memory-candidates.v1.json` 只是建议，用户确认前不能实现。
- `candidate_confirmed` 这个词仍可能被误读，后续 UI 文案建议写成“候选已确认保留”，不要写“已记住”。
- 用户偏好记忆会影响所有协作角色，后续实现必须把“用户当前指令优先于旧偏好”写进校验。

## 5. 下一步闸门

现在必须停下来等用户确认四件事：

1. 是否接受 `memory_governance.v1`。
2. 是否接受 `candidate_confirmed` 只表示候选确认，不表示正式记忆。
3. 是否接受 Skeleton-14 第一版建议用独立 `memory-candidates.v1.json` sidecar。
4. 是否允许进入 Skeleton-14 的候选生命周期最小实现。

只确认 1-3 不等于允许开始 4。

## 6. 如果用户允许 Skeleton-14

下一轮只能做：

- 记忆候选 sidecar。
- 候选创建。
- 候选状态变更。
- 候选审计引用。
- 只读 UI 或必要确认入口。

仍然不能做：

- 正式 `MemoryRecord` 写入。
- Obsidian 原生读写。
- 向量库或图数据库。
- 普通聊天自动记忆。
- 秘书直接写正式记忆。
