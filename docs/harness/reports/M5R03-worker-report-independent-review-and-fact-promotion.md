# M5R03 WorkerReport、独立审查与事实提升报告

- 日期：2026-08-16
- 阶段：stage-14 / leaf M5R03
- 结果：`M5R03_WORKER_REPORT_INDEPENDENT_REVIEW_AND_FACT_PROMOTION=PASS`
- 合同：`docs/contracts/m5-claim-review-and-fact-promotion-v1.md`（FROZEN，未改 M1–M4 正文与 hash）

## 1. 行为

- executed claim 必须通过 M5R01 exact-join 与独立 `IndependentRuntimeReceiptVerifier`（Grant/Attempt/Dispatch/effect/actor binding）；未验证只记 `RECORDED_UNVERIFIED`。
- `DEGRADED` / `OUTCOME_UNKNOWN` 进 `QUARANTINED`，不能审查提升。
- Reviewer RoleSession 必须独立于 worker；`AuthorizationDecisionId` 不得复用为 ResultUserDecision。
- 仅 `VERIFIED` + `ACCEPTED_RESULT` 写入 `m5_project_facts`。
- 同一 report_hash / decision key 重放返回已有记录，不产生第二份事实。
- manual/offline 永不作为执行成功提升。
- 伪造 receipt / effect 错配在 insert 前失败，零部分写。

## 2. 验证

- `cargo test --lib --offline -- m5_`：66 passed / 0 failed
- 定向：独立审查、重放幂等、worker 自审拒绝、未知 receipt 隔离、authorization id 复用拒绝、manual 不提升、伪造 receipt 零 claim。

## 3. 边界与下一步

RuntimeReceipt 形状供 M5R05 产出；本包不实现受控 runtime。普通项目 Supervisor / AppState 属于 M5R04。
