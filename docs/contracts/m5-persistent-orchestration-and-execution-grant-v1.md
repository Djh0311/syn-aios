# M5 持久编排与 ExecutionGrant 合同 v1

- 版本：v1（2026-08-16）
- 状态：**FROZEN（M5R02 冻结）**
- 关系：本合同是 M1 `project-orchestration-v1.md` 与 M5R01 `m5-execution-identity-and-worker-report-v1.md` 的实施补充；**不修改 M1–M4 任何冻结合同正文与 hash**。
- 真源链：M1 冻结对象/动作 → M5R01 冻结身份与 WorkerReport 分型 → 本合同冻结持久 store/UoW、Grant mint/persist/readback/dispatch 与入口登记 → M5R03–M5R07 继续消费。

## 1. 产品结果

用户确认后的动作可以安全、持久地准备，但 **Grant 完整持久化并 readback 成功之前绝不运行**。

签发顺序（继承 M1 formal-actions，本包落地）：

1. `RecordAuthorizationDecision`（APPROVED）
2. `CreatePlanAuthorization`（ACTIVE）
3. `CreateAuthorizedRunAndPreparedAttempt`（原子：Run CREATED + WorkItem READY + PreparedAttempt PREPARED_NON_RUNNABLE + worker RoleSession binding）
4. `BeginAttemptGrantBinding`（PREPARED_NON_RUNNABLE → GRANT_PENDING_NON_RUNNABLE）
5. `MintAttemptScopedGrant`（NONE → MINT_PENDING）
6. persist + `ConfirmGrantReadback`（MINT_PENDING → ACTIVE；hash/revision exact）
7. `ConfirmAttemptGrantBinding`（GRANT_PENDING_NON_RUNNABLE → GRANT_READY_NON_RUNNABLE）
8. `DispatchGrantedAttempt`（Dispatch PENDING_DELIVERY + M2 形 outbox REQUIRED）
9. `MarkAttemptDispatched`（GRANT_READY_NON_RUNNABLE → DISPATCHED）

PreparedAttempt 在 DISPATCHED 之前一律不可运行。不存在“任意字符串 Grant 即可 Runnable”的路径。

## 2. ExecutionGrant 字段（冻结，继承 M1 required_fields）

`grant_id, project_id, orchestration_id, workflow_run_id, work_item_id, attempt_id, authorization_id, authorization_revision, principal_actor_id, worker_role_session_id, scope_fingerprint, allowed_commands, cwd_ref, write_root_refs, object_refs, policy_decision_ref, issued_at, expires_at, revoked_at, status, revision, idempotency_key, effect_key, grant_hash, created_by_command_receipt_ref`

合法状态：`MINT_PENDING | ACTIVE | REVOKED | EXPIRED | QUARANTINED`。

核对：actor、RoleSession、Attempt、scope、command、cwd/write roots、object refs、authorization_id/revision、policy decision、expiry/revocation、idempotency/effect key、grant hash 必须 exact。调用参数只能收窄，不得扩权。

## 3. 失败与恢复

mint / persist / readback / dispatch 任一点失败：

- 撤销不可用 Grant（REVOKED，不可复用）；
- PreparedAttempt 留在不可运行状态：persist/readback 失败 → `PREPARED_NON_RUNNABLE`（可再 mint）；dispatch 失败 → `CANCELLED`；
- 不得留下 ACTIVE Grant 配不可核对 Attempt，也不得留下可运行 Attempt 配未 ACTIVE Grant。

拒绝：错项目、过期、已撤销、错误 authorization revision、调用方扩权。

## 4. Gateway 与副作用入口

- `ExecutionGrantGateway`：副作用入口只接 `GrantId`，由服务端加载完整 immutable Grant 后再逐字段核对。
- `ConversationCapabilityGateway`：只读 / `submit_proposal` 走 RoleSession + capability，不得产生副作用，也不得用普通 capability 替代 Grant。
- 至少一个非测试副作用入口（`admit_granted_side_effect`）只消费服务端加载的 Grant。

## 5. Runner / side-effect 入口登记

每个已知 Runner / side-effect 入口必须登记为：

| 分类 | 含义 |
|---|---|
| `new-grant` | 只接 GrantId，服务端加载完整 Grant |
| `guarded-legacy` | 旧守卫仍在，尚未迁到 new-grant；不得扩大权限 |
| `blocked` | 静态拒绝或保留阻断面 |

不得留下未登记旁路。登记表实现为 `m5_runner_entry_registry`，覆盖 `entrypoint-inventory-v1` 的 runner/background 清单加 M5 新入口。

## 6. Store / UoW

正式 SQLite catalog（`syn.m5.orchestration-schema/v1`）落地：AuthorizationDecision、PlanAuthorization、WorkflowRun、WorkItem、worker RoleSession binding、PreparedAttempt、ExecutionGrant、Dispatch、M2 形 command receipt / event / audit / outbox。

UoW 实现 `m2_ports::UnitOfWork`；outbox 实现 `m2_ports::OutboxRepository`。进程关闭后重开同一文件必须读回同一条链。

## 7. 退出

- 重启可读回同一链；
- 失败路径撤销 Grant 且 Attempt 不可运行；
- 错项目 / 过期 / 撤销 / 错误 revision / 扩权全部拒绝；
- 非测试副作用入口只消费服务端 Grant；
- 入口登记无未知旁路；
- `cargo check --lib --offline` 与 M5R02 定向测试通过。
