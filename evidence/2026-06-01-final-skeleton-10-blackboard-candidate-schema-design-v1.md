# Evidence：final-skeleton-10 黑板候选持久状态 schema 设计 v1

日期：2026-06-03

## 本轮结论

先说薄弱点：

- 本轮没有实现黑板候选写入。
- 本轮没有创建候选状态文件。
- 本轮没有迁移数据库。
- 用户已确认 schema 方向和 5 个补充点；仍未授权进入 Skeleton-11。

结论：

- 已完成 `final-skeleton-10-blackboard-candidate-schema-design-v1` 的 schema / 迁移计划 / Skeleton-11 草案。
- 建议候选持久状态使用独立逻辑 schema `blackboard_candidate_persistence.v1`。
- 建议第一版采用独立 sidecar JSON 作为最小存储，不写入 workflow state JSON。
- 候选确认状态只停留在黑板候选层，不写正式事实、不写正式记忆、不推进 workflow 状态。
- 已补充 sidecar JSON 文件路径和作用域、原子写入 / 备份 / 并发冲突、`candidate_key` 稳定生成规则、记录版本字段、rejected / discarded 后再次出现规则。

## 用户已确认和仍未允许

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

## 任务边界

执行：

- `final-skeleton-10-blackboard-candidate-schema-design-v1`

允许：

- 写黑板候选持久状态 schema。
- 写迁移计划。
- 写后续 Skeleton-11 实现任务草案。
- 更新 `CURRENT.md` 和 `tasks/README.md`。
- 新增 evidence / handoff。

禁止：

- 不实现黑板候选写入。
- 不改 workflow state JSON。
- 不迁移数据库。
- 不写正式事实。
- 不写正式记忆。
- 不改工作流状态机。
- 不执行真实 Codex。
- 不读写 `/Users/yoyi/.codex`。
- 不往项目画布右侧栏继续堆新主面板。

## 读过的依据

| 文件 | 用途 |
|---|---|
| `tasks/2026-06-01-final-workbench-skeleton-execution-package-v1.md:1040-1138` | Skeleton-10 / Skeleton-11 的目标、禁止项、输出和停止条件。 |
| `tasks/2026-06-01-final-workbench-skeleton-execution-package-v1.md:1458-1482` | 当前建议下一批次和确认前不能执行 Skeleton-11 的边界。 |
| `CURRENT.md` | 当前权威入口，确认 00-09 已完成，下一步是 Skeleton-10。 |
| `tasks/README.md` | 当前任务队列，确认只做 Skeleton-10。 |
| `evidence/2026-06-01-project-blackboard-minimal-read-model-d-v1.md` | 当前黑板是读模型，不是写入层。 |
| `handoffs/2026-06-02-final-skeleton-07-09-canvas-foundation-batch-v1-result.md` | 画布基础批次已完成，下一步是黑板候选 schema。 |
| `prototypes/productized-desktop-shell/src/lib/types.ts` | 现有前端黑板类型。 |
| `prototypes/productized-desktop-shell/src-tauri/src/types.rs` | 现有后端黑板类型。 |
| `prototypes/productized-desktop-shell/src-tauri/src/control_core.rs` | 当前控制核心已有候选直接晋升拒绝边界。 |
| `prototypes/productized-desktop-shell/src-tauri/src/workflow_read_model.rs` | 当前黑板从 workflow 读模型派生。 |

没有读取：

- `/Users/yoyi/.codex`
- auth、token、`.env`、密钥、完整 transcript

## 新增产物

| 文件 | 内容 |
|---|---|
| `docs/plans/2026-06-01-blackboard-candidate-persistence-schema-v1.md` | 黑板候选持久状态 schema、状态枚举、来源、目标类型、审计事件、读模型 overlay 规则、控制核心命令签名草案、迁移计划和 Skeleton-11 实现任务草案。 |

## Schema 摘要

顶层 schema：

- `BlackboardCandidatePersistenceStore`

核心对象：

- `BlackboardCandidateRecord`
- `BlackboardCandidateSourceRef`
- `BlackboardCandidateDecision`
- `BlackboardCandidateAuditEvent`

补充字段：

- `store_version`
- `record_version`
- `event_version`
- `candidate_key_version`
- `revision`
- `last_write_id`
- `content_fingerprint`

候选状态：

- `candidate_pending_control_core`
- `candidate_confirmed_for_followup`
- `candidate_rejected`
- `candidate_deferred`
- `candidate_discarded`

候选目标：

- `workflow_fact`
- `workflow_risk`
- `permission_decision`
- `audit_event`
- `formal_memory`
- `knowledge_reference`
- `no_promotion`

审计事件：

- `blackboard_candidate_pending_recorded`
- `blackboard_candidate_confirmed`
- `blackboard_candidate_rejected`
- `blackboard_candidate_deferred`
- `blackboard_candidate_discarded`

## 关键设计判断

| 判断 | 依据 |
|---|---|
| 不把候选状态写入 workflow state JSON | 用户禁止项和 Skeleton-10 禁止项都明确不改 workflow state JSON。 |
| 第一版建议独立 sidecar JSON | 当前项目 v0 事实层仍大量使用 JSON 文件，但 Skeleton-10 禁止数据库迁移；独立存储能避免污染 workflow state JSON。 |
| `candidate_confirmed_for_followup` 不等于正式晋升 | 用户禁止写正式事实 / 正式记忆；Skeleton-11 目标也只允许改变候选状态和审计。 |
| `candidate_key` 不能只用 `entry_id` | 当前 `BlackboardEntry` 是派生读模型，派生规则变化可能导致 entry_id 变化。 |
| 读模型应使用 overlay | 现有 `ProjectBlackboard` 仍由 workflow 派生，持久状态只能叠加显示，不能反向写来源。 |
| sidecar 路径应由 workflow state 路径推导 | 这样能保持一个 workflow state 对应一个候选状态文件，同时避免写入 `/Users/yoyi/.codex`。 |
| 写入必须有 revision 和 lock | 候选状态是可编辑 sidecar；没有并发控制会导致多窗口覆盖。 |
| rejected / discarded 不能自动恢复 | 来源再次出现不代表用户撤销了拒绝或废弃决定。 |

## 迁移计划摘要

本轮：

- 不迁移。
- 不创建持久文件。
- 不改数据库。
- 不改 workflow state JSON。

Skeleton-11 如获确认后：

- 创建独立空候选状态 store。
- 路径为 `<workflow_state_dir>/blackboard-candidates.v1.json`。
- 不批量回填现有候选。
- 用户首次处理候选时才写 `BlackboardCandidateRecord` 和候选审计事件。
- 读模型通过 `candidate_key` overlay 候选状态。
- 测试必须证明 workflow state JSON 未新增持久字段。
- 测试必须覆盖原子写入、备份、revision 冲突和 rejected / discarded 再次出现规则。

SQLite：

- 后置。
- 需要另开迁移计划。

## 验证

本轮没有改产品代码，所以没有跑代码测试。

依据：

- Skeleton-10 验收要求是 schema / 迁移计划确认前不能实现。
- 本轮没有 TypeScript / Rust / CSS 代码改动。

做了只读复核：

- 确认 Skeleton-10 原文输出路径。
- 确认现有黑板类型仍是读模型。
- 确认控制核心已有“黑板候选不能直接晋升”的边界。
- 确认新增计划文件已落盘。

## 禁止事项执行情况

| 禁止项 | 结果 |
|---|---|
| 不实现黑板候选写入 | 已遵守。 |
| 不改 workflow state JSON | 已遵守。 |
| 不迁移数据库 | 已遵守。 |
| 不写正式事实 | 已遵守。 |
| 不写正式记忆 | 已遵守。 |
| 不改工作流状态机 | 已遵守。 |
| 不执行真实 Codex | 已遵守。 |
| 不读写 `/Users/yoyi/.codex` | 已遵守。 |
| 不往项目画布右侧栏继续堆新主面板 | 已遵守。 |

## 不接受为

不接受为：

- 黑板候选持久确认已实现。
- 黑板候选写入命令已实现。
- workflow state JSON 已迁移。
- 数据库已迁移。
- 正式事实写入已实现。
- 正式记忆写入已实现。
- Skeleton-11 已开始。
