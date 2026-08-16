# SYN-M5R03: WorkerReport、独立审查与事实提升

日期: 2026-08-16
阶段: M5 (Stage-14) / leaf M5R03
状态: COMPLETE
优先级: HIGH

## 目标

执行者提交只先成为声明；精确对上真实执行链并经过独立 Review 与 ResultUserDecision 后才能成为项目事实。

## 范围

1. 正式 claim 持久化（executed/manual/offline）。
2. production RuntimeReceiptVerifier（与 producer 分离）。
3. 独立 Review + ResultUserDecision；禁止复用 AuthorizationDecision。
4. 仅 VERIFIED + ACCEPTED_RESULT 提升事实；幂等；失败零部分写。

## 不许动

M1–M4 冻结合同；m6_*.rs；stage-12/D0C04/D0C05；把未验证或 manual/offline 提升为执行成功。
