# Handoff：final-skeleton-10 黑板候选持久状态 schema 设计 v1

日期：2026-06-03

## 本轮完成

完成 `final-skeleton-10-blackboard-candidate-schema-design-v1` 的补充版 schema。

先说限制：

- 本轮只写 schema / 迁移计划 / Skeleton-11 草案。
- 没有实现黑板候选写入。
- 没有改 workflow state JSON。
- 没有迁移数据库。
- 没有写正式事实或正式记忆。
- 没有开始 Skeleton-11。

用户已确认：

- 接受 `blackboard_candidate_persistence.v1` 的方向。
- 接受第一版用独立 sidecar JSON，不写 workflow state JSON。
- 接受 `candidate_confirmed_for_followup` 只表示候选层确认，不做正式晋升。
- 接受 sidecar JSON 路径和作用域。
- 接受原子写入、备份、lock、revision 并发冲突处理。
- 接受 `candidate_key` 稳定生成规则。
- 接受版本字段。
- 接受 `rejected` / `discarded` 后再次出现不自动恢复。

用户仍未允许：

- 进入 `final-skeleton-11`。
- 实现黑板候选写入。

## 新增产物

| 文件 | 内容 |
|---|---|
| `docs/plans/2026-06-01-blackboard-candidate-persistence-schema-v1.md` | 黑板候选持久状态 schema、sidecar 路径和作用域、原子写入 / 备份 / 并发冲突、candidate_key 规则、记录版本、rejected / discarded 再次出现规则、迁移计划和 Skeleton-11 草案。 |
| `evidence/2026-06-01-final-skeleton-10-blackboard-candidate-schema-design-v1.md` | 本轮 evidence。 |
| `handoffs/2026-06-01-final-skeleton-10-blackboard-candidate-schema-design-v1-result.md` | 本 handoff。 |

## 更新文件

| 文件 | 内容 |
|---|---|
| `CURRENT.md` | 标记 Skeleton-10 已完成，下一步停在用户确认 schema / 迁移计划。 |
| `tasks/README.md` | 同步当前任务队列：Skeleton-10 已完成；Skeleton-11 不能自动开始。 |

## 关键判断

当前不能直接进入 Skeleton-11。

依据：

- 总执行包写明 Skeleton-10 完成后必须让用户确认 schema。
- Skeleton-11 前置要求用户明确接受 schema / 迁移计划，并明确允许进入实现。
- 用户已明确接受补充版 5 个点，但暂不自动授权进入 `final-skeleton-11`；是否开始实现写入需要另行确认。

## 待用户另行确认

请另行确认：

1. 是否允许进入 `final-skeleton-11-blackboard-candidate-persistence-implementation-v1`。
2. 是否允许按已确认 schema 实现黑板候选 sidecar 写入。

当前确认 schema 不等于允许开始实现。

## 手动复核清单

1. 打开 `docs/plans/2026-06-01-blackboard-candidate-persistence-schema-v1.md`。
2. 确认状态枚举包含 pending、confirmed、rejected、deferred、discarded。
3. 确认 confirmed 只表示候选层确认，不做正式晋升。
4. 确认迁移计划写明本轮不迁移，Skeleton-11 也不写 workflow state JSON。
5. 确认 Skeleton-11 草案仍禁止正式事实、正式记忆、工作流状态推进和真实 Codex。
6. 确认 `CURRENT.md` 和 `tasks/README.md` 都写明等待用户确认，不能自动开始 Skeleton-11。
7. 确认 sidecar 路径、原子写入、备份、revision 冲突和再次出现规则都已写入 schema。

## 验证

本轮没有跑代码测试。

原因：

- 没有改产品代码。
- Skeleton-10 验收是 schema / 迁移计划确认前不能实现。

已做只读复核：

- 复核 Skeleton-10 / Skeleton-11 原文边界。
- 复核现有黑板读模型。
- 复核新文件和权威入口更新。

## 明确未做

- 未实现候选状态 store。
- 未实现控制核心写入命令。
- 未改前端 UI。
- 未改后端类型。
- 未改 workflow state JSON。
- 未迁移数据库。
- 未写正式事实。
- 未写正式记忆。
- 未改工作流状态机。
- 未执行真实 `codex exec` / `codex exec resume`。
- 未读写 `/Users/yoyi/.codex`。
- 未启动 MCP canvas run。
