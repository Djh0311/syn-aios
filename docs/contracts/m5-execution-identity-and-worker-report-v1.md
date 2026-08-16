# M5 执行身份链与 WorkerReport 分型合同 v1

- 版本：v1（2026-08-16）
- 状态：**FROZEN（M5R01 冻结）**
- 关系：本合同是 M1 `project-orchestration-v1.md`（冻结）的 M5 实施解释与补充合同；**不修改 M1–M4 任何冻结合同正文与 hash**；不另造第二套 owner。
- 真源链：M1 合同冻结身份/owner/join → 本合同冻结 M5 实现读取与 WorkerReport 分型 → M5R02–M5R07 按本合同落地持久化、Grant、恢复、ProjectSummary、UI 与隔离 App。
- 范围：一次项目动作只有一套明确身份和完整链路；不靠几个字符串拼出看似有效的执行记录。

## 1. 身份 owner、分配时点与关系（冻结）

以下身份与 owner 全部继承 M1 `project-orchestration-v1.md` 冻结语义，本表只记录 M5 实现读取，不改变 M1：

| 身份 | owner（M1 冻结） | 分配时点（M1 冻结） | M5 实现读取 |
|---|---|---|---|
| ProjectId | project scope | 项目建立 | 复用现有 `stable_id(project_root)` 确定性生成，不另造 |
| ProposalId | project_orchestration | CreateProposal | 现有 store 真源；M5R01 只引用，不重造 |
| AuthorizationDecisionId | project_authorization | RecordAuthorizationDecision | 与 ResultUserDecisionId 独立 namespace，绝不互换 |
| AuthorizationId | project_orchestration | CreatePlanAuthorization | 绑定 authorization_revision |
| CorrelationId | command_gateway | 命令创建时 | 贯穿全链关联；与 OrchestrationId **不同一** |
| OrchestrationId | project_orchestration | 一次 Proposal→Execution 链路唯一 | 与 CorrelationId 不互相替代（M1 no_alias） |
| WorkflowRunId | execution_aggregate | CreateAuthorizedRunAndPreparedAttempt | 一个 Orchestration 可含多个 Run（如重试） |
| WorkItemId | execution_aggregate | 同一 atomic group | 绑定 Run 与 source object |
| NodeId | execution_aggregate | WorkItem 内节点 | 与 WorkItemId/AttemptId 精确 join |
| DispatchId | execution_aggregate | DispatchGrantedAttempt | 绑定 outbox_item_id 与 effect_id |
| AttemptId | execution_aggregate | 同一 atomic group | 一次 Dispatch 一个 Attempt |
| GrantId | project_orchestration | MintAttemptScopedGrant | attempt-scoped，不可复用/跨项目 |
| worker RoleSessionId | M3 唯一身份真源 | M3 会话建立 | 执行者身份只来自可信 session/binding，不来自 report 自报 |
| RuntimeReceiptId | execution_readback | RecordExecutionAttemptReadback | 只由独立 receipt verifier 产出 |
| ResultUserDecisionId | review_domain | RecordResultUserDecision | 与 AuthorizationDecisionId 不互换 |
| trusted actor | claim_ledger / review | 来自可信 session/binding | 执行者只接受可信绑定中的 actor |
| hash（report/grant/receipt） | 各 owner | 写入时计算 | 完整性核对必需，缺失即 fail closed |

关系裁定（冻结）：

- **CorrelationId ≠ OrchestrationId**：M1 已裁定两者 owner 不同（command_gateway / project_orchestration）且 `no_alias` 永不互相替代；Correlation 通过 Proposal.root_correlation_id 解析。M5 实现不得把两者当同一标识或父子混用。
- **AuthorizationDecisionId ≠ ResultUserDecisionId**：不同 owner、namespace、合法状态与回执。
- 跨项目串接：任何 join 中 project_id 不一致即拒绝，业务写前 fail closed。

## 2. WorkerReport 分型（冻结）

| 分型 | 允许字段 | 禁止字段（必须缺席） | 事实地位 |
|---|---|---|---|
| EXECUTED | project/run/item/node/dispatch/attempt/grant/RoleSession/authoritative receipt/authenticated actor/hash + report_kind | manual/offline 专有 | 必须精确对上真实执行链才能成为事实 |
| MANUAL / OFFLINE | claim_id、project_id、orchestration_id、authenticated_submitter_id、source_refs、evidence_refs、claim_hash | dispatch_id、attempt_id、grant_id、worker_role_session_id、authoritative_execution_receipt_ref、execution_receipt | 永不上升为执行成功 |

- executed report **必须由显式输入建立**：缺省 ReportKind 或缺任一 join 字段的输入不得自动成为执行报告（修复候选 `ReportKind::default()==Execution` 缺口）。
- actor 只来自可信 session/binding；actor 自报或缺少 worker RoleSession 绑定即拒绝。
- 完整 exact-join 必需字段（M1 ExecutedReport）：project_id、orchestration_id、workflow_run_id、work_item_id、node_id、dispatch_id、attempt_id、grant_id、worker_role_session_id、authoritative_execution_receipt_ref、authenticated_actor_id、report_hash、observed_attempt_state。

## 3. 原型裁决（M5R01 逐项）

| 模块 | 裁决 | 理由 |
|---|---|---|
| m5_orchestration_identity.rs | KEEP + 补缺 | 身份方向符合 M1；补齐 WorkItemId/NodeId/DispatchId/RuntimeReceiptId/ReportId 缺型；Correlation/Orchestration 保持独立 |
| worker_report.rs（M5 扩展） | REWRITE | verify_integrity 缺 exact-join；缺省 ReportKind 默认 Execution；需按本合同重写 |
| m5_prepared_attempt.rs | KEEP（状态机） | Prepared 不可运行、Grant 绑定后才 Runnable 方向正确；Grant 缺失放行缺口由 M5R02 用持久 store 修 |
| m5_gateway_traits.rs / m5_project_summary.rs / m5_project_supervisor.rs / m5_controlled_execution.rs | KEEP（作为候选素材） | M5R02–M5R06 施工 |
| m6_*.rs、m6_member_directory.rs.bak | QUARANTINE（只读保全） | M6 未激活；不进入 M5 载体 |

## 4. 旧数据映射与隔离（legacy mapping / quarantine）

- 旧 report 中缺失 join 或无法精确映射到 M1 执行链的，**只隔离不补齐**：进入 claim 状态 `RECORDED_UNVERIFIED` 或 `QUARANTINED`，绝不猜测填充身份。
- 旧 `contains`/slug 归属判定、路径猜测 owner、无 Grant 的“执行报告”一律 quarantine。
- 物理删除留到 M9；M5 只冻结映射与隔离规则。

## 5. Red probes（先建可复现缺口的测试）

- executed report 缺 workflow_run_id / work_item_id / node_id / dispatch_id / grant_id / worker_role_session_id / receipt_ref / report_hash 任一 → fail closed；
- manual/offline report 携带任一执行 join（dispatch/attempt/grant/RoleSession/receipt）→ 拒绝；
- 缺省 ReportKind 不自动成为 Execution；
- actor 自报（无 worker RoleSession 绑定）→ 拒绝；
- 跨项目 join → 拒绝。

## 6. 退出与验证

- 任一 ID 错配、跨项目串接、actor 自报或 Grant 缺失都在业务写前 fail closed；
- 合同（本文件 + M1）与 schema/迁移矩阵一致；现有单测只作回归输入；
- `cargo check --lib --offline` 通过；M5R01 定向测试全绿。
