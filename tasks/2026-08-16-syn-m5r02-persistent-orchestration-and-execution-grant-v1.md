# SYN-M5R02: 持久编排核心与 ExecutionGrant

日期: 2026-08-16
阶段: M5 (Stage-14) / leaf M5R02
状态: COMPLETE
优先级: HIGH

## 目标

用户确认后的动作可以安全、持久地准备，但 Grant 完整持久化和回读前绝不运行。用正式 store/UoW 落地 Run、WorkItem、worker RoleSession binding、PreparedAttempt、Grant、Dispatch 和 outbox；严格实现 AuthorizationDecision → Authorization → Run/WorkItem + binding → PreparedAttempt → mint Grant → persist/readback → Dispatch → DISPATCHED → outbox。

## 范围

1. 冻结 M5 持久化与 Grant 补充合同（不改 M1–M4 正文/hash）。
2. PreparedAttempt 状态机改为 M1 合法状态；禁止任意字符串 Grant 进入可运行态。
3. 正式 SQLite catalog + `UnitOfWork` / `OutboxRepository` adapter。
4. 生产 `ExecutionGrantGateway` / `ConversationCapabilityGateway`；副作用入口只接 GrantId。
5. Runner/side-effect 入口登记为 new-grant / guarded-legacy / blocked。

## 验证

- `cargo check --lib --offline`
- `cargo test --lib --offline -- m5_`
- 定向：重启读回、失败撤销、错项目/过期/撤销/扩权拒绝、入口登记完整。

## 不许动

M1–M4 冻结合同正文；m6_*.rs；stage-12 / D0C04 / D0C05；真实资料/provider/push/reset/stash/clean；Grant persist/readback 完成前放行可运行 Attempt。
