# SYN-M5R07 U01c terminal retry 新 lineage

日期: 2026-08-18
阶段: stage-14 / leaf M5R07
状态: implementation candidate / NOT_ACCEPTED / NOT_M5_COMPLETE / NOT_CLOSEOUT
合同: `docs/contracts/m5-r07-terminal-retry-lineage-addendum-v1.md`
父提交: `6694b1e7c40571111e057bee5b770faca72c162d`

U01a 默认入口与 U01b 安全有限 STOP/RESUME 已独立 scoped PASS，本包不再重做。本包只关 terminal known-no-effect 的新 Attempt / Grant / Dispatch / effect lineage。

## 做完的标准

- `apply_m5_execution_control(RETRY)` 仍只做控制与新 lineage 准备，绝不执行 runtime。
- load/apply 重新核 binding/project、fresh M3 supervisor/worker、formal progress 指向的 Plan/authorization、旧 Grant/Dispatch/Attempt/outbox/effect、DurableOperation、authoritative execution readback/carriers。
- 只有权威 readback 明确 `FAILED`/`TIMED_OUT` 且服务端能证明原 effect 未发生时 `can_retry=true`。
- RETRY 在一个事务内：旧链 immutable；同一 Plan/orchestration/workflow_run/work_item/node 下创建新 Attempt/Grant/Dispatch/新 effect；formal progress 原子切换并清空下游 refs；control revision/receipt CAS 与精确 replay 一致。
- 后续只有用户显式调用现有 `run_m5_authorized_runtime` 才执行；exact replay / double click 不建第二 lineage。
- UI 现有 retry 按钮只吃 server `can_retry`；renderer 不传旧/新 attempt/grant/dispatch/effect/fault。
- 定向测试覆盖 A–E。不跑全 m5 / npm / evidence。

## 明确仍未完成

- U02 ordinary positive Tauri runner / server-owned fixture
- 真实 legacy M1 composition
- shared-isolated 正向证据
- RUNNING/LEASED authoritative cancel
- OUTCOME_UNKNOWN 同 effect reconcile
- M5 / stage-14 完成或 closeout
