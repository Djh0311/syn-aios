# 当前状态

截至 2026-08-08，M2 已在 `main` 完成项目级收口；M3 保持 `PLANNED / NOT_ACTIVE`，没有活动产品开发包。

## 当前权威入口

1. 当前用户指令；
2. `AGENTS.md`；
3. `docs/harness/plan.md` → 当前 stage → 唯一 current leaf；
4. `docs/harness/authorization.json`；
5. 对应阶段计划、合同、源码与验收材料。

Stage 3 归档后没有 active stage、current leaf 或持续授权。计划、handoff、报告和历史 Harness 文件只提供事实与导航，不自行授权施工。

## 已完成

- M1 合同、安全与作用域基础已经进入 main。
- M2 的具名 `workflow-state-sidecar` reference slice 已进入 main，覆盖 UoW、denial audit、receipt、snapshot、outbox、projector/checkpoint、parity/recovery，以及隔离 R4 崩溃/重启验收。
- M2 完整 Rust 库测为 1385 passed / 0 failed / 45 ignored；干净主线 R4 为 7/7 PASS。

验收与提交事实见 `docs/harness/reports/M2C02-mainline-integration-and-acceptance.md` 和 `docs/harness/reports/M2C03-lite-closeout-and-guidance-handoff.md`。

## 保留边界

- live Workbench 数据没有迁移或切换；DAT-007 保持 `NOT_MIGRATED / NO_CUTOVER`。
- provider、真实账号、真实消息、部署和发布没有进入 M2。
- review、decision、source-owner apply-result 与 RoleSession/Handoff 属后续阶段，不因 M2 收口自动成立。
- 混合开发工作树 `/Users/yoyi/workspace/product-line-syn-fnd-002` 保持只读，既有 WIP 不作为 main 的当前事实。

## 下一入口

下一工作只允许先做 M3 计划与事实的只读复核。任何 M3 产品实现、Harness stage 或 leaf 激活，都需要用户新的明确指令。
