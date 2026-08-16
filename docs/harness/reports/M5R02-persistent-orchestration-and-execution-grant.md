# M5R02 持久编排核心与 ExecutionGrant 报告

- 日期：2026-08-16
- 阶段：stage-14 / leaf M5R02
- 结果：`M5R02_PERSISTENT_ORCHESTRATION_AND_EXECUTION_GRANT=PASS`
- 合同：`docs/contracts/m5-persistent-orchestration-and-execution-grant-v1.md`（FROZEN，补充 M1 / M5R01；未改 M1–M4 正文与 hash）

## 1. 持久链

正式 SQLite catalog `syn.m5.orchestration-schema/v1` 落地 AuthorizationDecision、PlanAuthorization、WorkflowRun、WorkItem、worker RoleSession binding、PreparedAttempt、ExecutionGrant、Dispatch，以及 M2 形 command receipt / event / audit / outbox。

`M5SqliteUnitOfWork` 实现 `m2_ports::UnitOfWork`；`M5OutboxRepository` 实现 `m2_ports::OutboxRepository`。签发顺序：

AuthorizationDecision → PlanAuthorization → 原子 Run/WorkItem/binding/PREPARED_NON_RUNNABLE → GRANT_PENDING → mint Grant → persist/readback ACTIVE → GRANT_READY_NON_RUNNABLE → Dispatch + outbox → DISPATCHED。

GRANT_READY 仍不可运行。任意字符串 Grant 不能进入 DISPATCHED。

## 2. 失败与拒绝

- persist / readback 失败：Grant REVOKED，Attempt 回到 PREPARED_NON_RUNNABLE，无 Dispatch。
- dispatch 失败：Grant REVOKED，Attempt CANCELLED。
- 错项目、错误 authorization revision、调用方扩权（command / cwd / write root / object ref）全部拒绝。
- 进程关闭后重开同一 SQLite 文件可读回同一条链。

## 3. Gateway 与入口

- 生产 `PersistentExecutionGrantGateway` 只按 GrantId 加载完整 immutable Grant。
- 非测试副作用入口 `admit_granted_side_effect` 只接 GrantId。
- `ConversationCapabilityGateway` 只管只读/提案，不能替代 Grant。
- `m5_runner_entry_registry` 覆盖 inventory 全部 RUN-/BG- 入口 + `M5-SE-001`：new-grant / guarded-legacy / blocked，无未知旁路。`RUN-006` 为 blocked。

## 4. 验证

- `cargo check --lib --offline`：PASS（无 error；既有 warning debt 未清零）
- `cargo test --lib --offline -- m5_`：57 passed / 0 failed（含状态机、Grant、store/UoW、重启读回、失败恢复、gateway 拒绝、副作用入口、入口登记）
- `git diff --check`：PASS（本包写域）

## 5. 边界与下一步

- 本包不接普通项目 production caller（M5R04）、不实现 Review/事实提升（M5R03）、不实现 stop/retry/resume 持久恢复（M5R05）、不实现 ProjectSummary（M5R06）。
- m6_*.rs 与 M5R04–R06 候选文件保持 untracked / 只读保全，不进入本包载体。
- 路由：M5R03 WorkerReport、独立审查与事实提升。
