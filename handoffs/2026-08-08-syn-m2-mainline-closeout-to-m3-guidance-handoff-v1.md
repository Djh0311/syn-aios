# Syn M2 主线收口 → M3 指导交接 v1

日期：2026-08-08

本文件是跨对话临时导航，不是第二份权威，也不授予 M3 实现。当前权威仍是用户指令、`AGENTS.md` 与 Harness Lite 当前链。

## 已完成事实

- M2 实现已进入 `main`：`d6bf4e464e32bd5310dfdfb2e46dfd0a47fd787f`。
- 主线验收记录提交：`c232fc2`；详细报告见 `docs/harness/reports/M2C02-mainline-integration-and-acceptance.md`。
- 完整 Rust 库测：1385 passed / 0 failed / 45 ignored。
- 干净主线 R4：7/7 PASS；receipt SHA-256 `fbd799a347934225f5e2eb652d286b690d8137c69c7baa55b4835fbebfc3ac13`。
- Code Map 的 M2 grant ledger 已标为 `active / verified-partial`，验证边界写明仅为主线 bounded reference slice 与 isolated scratch R4。

## 仍然不是事实

- live Workbench 未迁移，DAT-007 为 `NOT_MIGRATED / NO_CUTOVER`。
- provider、真实账号、真实消息、部署、发布未进入。
- RoleSession、Turn、ProviderHandle、Handoff、review/decision/source-owner apply-result 未因 M2 自动成立。
- M3 没有 active stage、leaf 或授权。

## Git 与 WIP

- 工作主线：`/Users/yoyi/workspace/product-line-syn-integration-main`，branch `main`。
- M2 提取分支：`/Users/yoyi/workspace/product-line-syn-m2-closeout`，branch `codex/syn-m2-closeout`，保留作审查锚点。
- 混合开发工作树：`/Users/yoyi/workspace/product-line-syn-fnd-002`，branch `syn-fnd-002-dev`，只读保留 64 tracked + 14 untracked；13 项战略 WIP 指纹未被 M2 收口改写。
- 本轮没有 push、部署或发布。

## 新指导对话的第一任务

只读复核 `docs/plans/2026-08-01-syn-stage-3-role-session-and-explicit-handoff-plan-v1.md`：

1. 重新核 main 中与 RoleSession、transport、provider handle、handoff、frontend cache 相关的实现事实与 WIP 边界；
2. 用 M1/M2 的真实 exits 校正 M3 的前置、未知、HOLD、任务切片、单写面和验收；
3. 输出 M3 阶段计划复核结论和建议的首个任务包边界；
4. 在用户再次明确同意前，不修改产品代码，不激活 Harness，不运行真实 provider/消息，不 push。

## 建议阅读顺序

1. `AGENTS.md`
2. `docs/current-state.md`
3. `docs/harness/plan.md`
4. 本交接
5. `docs/harness/reports/M2C02-mainline-integration-and-acceptance.md`
6. `docs/plans/2026-08-01-syn-stage-2-fact-event-audit-transaction-foundation-plan-v1.md`
7. `docs/plans/2026-08-01-syn-stage-3-role-session-and-explicit-handoff-plan-v1.md`
8. 当前 Git、源码与直接验证
