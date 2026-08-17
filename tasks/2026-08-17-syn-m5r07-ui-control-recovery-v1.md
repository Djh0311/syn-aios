# SYN-M5R07 U01b 安全有限 server-owned execution control

日期: 2026-08-17
阶段: stage-14 / leaf M5R07
状态: implementation candidate / NOT_ACCEPTED / NOT_M5_COMPLETE / NOT_CLOSEOUT
合同: `docs/contracts/m5-r07-ui-control-recovery-addendum-v1.md`
父提交: `f962038e725ba4e24b2699a46cd1a8d274f13ae6`

默认入口 `jiaoban` 已由 `f962` 独立通过，本包不再重做。本包只关安全有限控制。

## 做完的标准

- 正式 `load_m5_execution_control` / `apply_m5_execution_control` 上线；request 字段封闭；响应服务端派生 revision/state/can_*/blocked_reason/receipt/replayed。
- load/apply 重新核绑定项目、fresh M3 supervisor/worker、formal progress 指向的 Grant/Dispatch/Attempt/outbox/effect/DurableOperation；直接链错或 authority 失效写前拒绝。
- control 有 durable revision / CAS / 精确同请求 replay；control 不调用 runtime。
- 只开放现有结构能真实证明的动作：无 effect 的 CREATED/PAUSED 可 STOP；PAUSED+checkpoint 可 RESUME。OUTCOME_UNKNOWN 与 terminal Attempt `can_retry=false`。复杂 new-lineage retry 留给 U02/后续。
- TS API 与 `ProjectSupervisorPanel` 只消费 load/apply；按钮只按 `can_*`；renderer 不传 operation/fault/Grant。
- 定向测试覆盖 happy load、stop created/paused、paused resume、精确 replay、stale/divergent revision、cross-project 与 authority 写前拒绝、OUTCOME_UNKNOWN/terminal 不得盲 retry、transaction fault rollback、reopen 一致、control 不增加 runtime effect。不新增攻击矩阵，不写 evidence。

## 明确仍未完成

- U02 ordinary positive runner
- 真实 legacy M1 composition
- shared-isolated 正向证据
- RUNNING/LEASED authoritative cancel
- 新 lineage 上的 RETRY
- OUTCOME_UNKNOWN reconcile
- M5 / stage-14 完成或 closeout
