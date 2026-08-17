# M5R07 U01c terminal retry 新 lineage 补充合同 v1

- 版本：v1（2026-08-18）
- 状态：**ADDITIVE / 非冻结正本 / 不构成 M5 或 stage-14 完成**
- 关系：补充 `m5-r07-ui-control-recovery-addendum-v1.md`、`m5-controlled-execution-and-runtime-conformance-v1.md`、`m5-persistent-orchestration-and-execution-grant-v1.md`。**不改 M1–M4 冻结合同正文与 hash，不改 shared-isolated authority/profile。**
- 范围：**只覆盖 U01c scoped candidate**。U01a 默认入口与 U01b 安全有限 STOP/RESUME 已独立 scoped PASS，本包不再重做。U02 ordinary positive Tauri runner、M1 ordinary GUI composition、OUTCOME_UNKNOWN 同 effect reconcile、M6、closeout 均未完成。

## 1. 本包不重做的已落地项

- 默认入口保持 `jiaoban` 与正式 `ProjectSupervisorPanel`。
- `load_m5_execution_control` / `apply_m5_execution_control` request 字段封闭；renderer 不传 operation / grant / dispatch / attempt / effect / fault。
- STOP / RESUME 仍只覆盖 U01b 已证明的无 effect CREATED/PAUSED 与 PAUSED+checkpoint。
- 唯一生产 runtime 入口仍是 `run_m5_authorized_runtime_with_state`。

## 2. RETRY 只做控制与新 lineage 准备

`apply_m5_execution_control(RETRY)`：

1. 只做控制与新 lineage 准备，**绝不执行 runtime**。
2. 每次从 binding/project 重新核 fresh M3 supervisor + worker、formal progress 指向的 Plan/authorization、旧 Grant / Dispatch / Attempt / outbox / effect、DurableOperation、authoritative execution readback / carriers。
3. 旧 Attempt / Grant / Dispatch / outbox / readback / DurableOperation **保持 immutable**；禁止原地复活、覆盖或把旧 effect_id 再投入运行。
4. 后续只有用户显式再点现有 `run_m5_authorized_runtime` 才执行。double click / exact replay 不得建立第二 lineage。

## 3. 何时 can_retry=true

只有同时成立才允许 RETRY：

- 权威 execution readback 明确 `FAILED` 或 `TIMED_OUT`；
- Attempt 同步为 terminal `FAILED` / `TIMED_OUT`；
- DurableOperation 与 readback receipt / outcome 精确一致；
- 服务端能用现有 RuntimeReceipt.enforcement_status / outcome 与 DurableOperation 证明 **原 effect 未发生**。

下列情况继续稳定拒绝，零写：

- `OUTCOME_UNKNOWN` / `UNKNOWN_READBACK`
- `SUCCEEDED`
- `CANCELLED`
- 已有外部 effect（包括 failure 但 effect 已发生）
- readback / carrier 不完整或被篡改

不新增通用故障框架。已知无 effect 判断只复用现有 RuntimeReceipt / enforcement / outcome 与 DurableOperation。

## 4. 同一事务内的新 lineage

允许的 RETRY 必须在一个事务内：

1. 在同一 Plan / orchestration / workflow_run / work_item / node 下创建 **新 Attempt**。
2. 基于当前 **active Plan** 与 fresh worker mint **新 Grant**，fresh grant hash / revision，全新 `effect_id`。
3. 创建新 Dispatch / outbox；状态遵守现有 `approval → GRANT_READY / PENDING_DELIVERY → readback → DISPATCHED` 规则。复用现有正式 persist / mint / dispatch 服务方法，不自造第二套 authority。
4. formal progress 原子切到新 grant / dispatch，并清空旧 receipt / claim / review / result downstream refs。
5. control revision + receipt CAS 与 U01b 精确 replay 一致：同请求返回同一 receipt 且 `replayed=true`；stale / divergent 拒绝零写。

RETRY 返回新 control view。新 lineage 在用户显式 runtime 之前停在 `GRANT_READY_NON_RUNNABLE` / `PENDING_DELIVERY`。

## 5. UI / DTO

- renderer 不得传旧/新 attempt / grant / dispatch / effect / fault。
- 现有 retry 按钮只消费 server `can_retry`。
- 本包不为 RETRY 扩前端，除非类型直接需要；响应不增加调用方自选 authority 字段。

## 6. 明确未关闭

本合同 **不** 宣布下列事项完成：

- U02 ordinary disposable positive Tauri runner / server-owned fixture
- M1 ordinary `ProjectRecord` → canonical/exact alias 的可信创建/迁移 / ordinary GUI composition
- shared-isolated 正向 scene / window / restart
- `RUNNING` / `LEASED` 的 authoritative cancel readback
- `OUTCOME_UNKNOWN` 的同 effect reconcile
- M5 / stage-14 closeout / M6 激活

既有候选实现或定向测试不得写成 M5 / stage 完成。
