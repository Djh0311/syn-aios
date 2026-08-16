# M5 Claim、独立审查与事实提升合同 v1

- 版本：v1（2026-08-16）
- 状态：**FROZEN（M5R03 冻结）**
- 关系：补充 M1 `project-orchestration-v1` 与 M5R01 WorkerReport 分型；**不修改 M1–M4 正文与 hash**。
- RuntimeReceipt 形状供 M5R05 产出、本包独立验证；本包不实现受控 runtime。

## 1. 产品结果

执行者提交的内容只先成为 claim。只有 exact-join 对上真实执行链，且独立 Review=`VERIFIED`、ResultUserDecision=`ACCEPTED_RESULT` 后，才能成为项目事实。

## 2. RuntimeReceiptVerifier

与 worker/report producer 分离的 production verifier。核对：

- Grant ACTIVE 且 attempt/dispatch/effect exact
- actor binding = worker RoleSession + trusted authentication
- trace hash / effect id 与 Dispatch.effect_id exact
- enforcement status：`OK` 才可进入审查提升；`DEGRADED` / `OUTCOME_UNKNOWN` 只保持 claim/待协调

producer 自验、伪造 receipt、trace/effect 错配、重复验证冲突、actor 错绑一律拒绝。

## 3. Claim 分型

- EXECUTED：必须完整 exact-join + 通过 verifier；claim_status=`RECORDED_UNVERIFIED` 或 `QUARANTINED`
- MANUAL / OFFLINE：禁止执行 join；永不提升为执行成功事实
- 同一 report_hash 幂等：重放返回已有 claim，不写第二份

## 4. Review 与 ResultUserDecision

- Reviewer RoleSession ≠ worker RoleSession
- AuthorizationDecisionId 与 ResultUserDecisionId 不同 namespace，禁止复用
- 未验证、冲突、degraded、`OUTCOME_UNKNOWN` 不能进入事实提升
- 错配或持久失败零部分写

## 5. 退出

完整 exact-join 反例；重启/重放不产生第二份事实；manual/offline 永不冒充执行成功。
