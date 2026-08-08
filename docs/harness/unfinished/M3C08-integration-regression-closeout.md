# M3C08 M3 集成回归、现状回写与阶段收口

阶段：stage-05 阶段5 M3 角色会话与显式交接
目标：以干净 main 对 M3 合同、后端、repository、transport、Handoff、前端和隔离证据做完整回归，回写当前事实、边界、迁移/回切和下游入口，并归档 stage-05。
干完的标准：M3 退出矩阵逐项有直接证据或明确未进入；完整离线测试和构建通过；工作树干净；真实 provider/消息、merge/push/release 未发生；M4/M5 不自动激活。

允许动：

- docs/current-state.md
- docs/plans/2026-08-01-syn-personal-ai-workbench-master-development-plan-v1.md
- docs/plans/2026-08-01-syn-stage-3-role-session-and-explicit-handoff-plan-v1.md
- docs/plans/README.md
- docs/task-queue.md
- docs/harness/reports/M3C08-mainline-integration-and-acceptance.md [新增]
- handoffs/2026-08-09-syn-m3-closeout-to-m4-m5-guidance-v1.md [新增]

## 步骤

1. 核对 stage-05 全部提交、文件 owner 和无范围外漂移。
2. 跑合同、Rust 聚焦/完整库测、schema、fake provider、前端测试/类型和非测试构建。
3. 汇总隔离桌面证据与真实 provider 未进入边界，复核迁移和 rollback。
4. 回写 current-state、master/M3 计划、计划索引、任务入口、验收报告和交接。
5. 独立审查后精确提交，归档 M3C08 和 stage-05，提交终态控制记录并停止。
