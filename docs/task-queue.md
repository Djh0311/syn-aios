# 当前任务入口

截至 2026-08-13，M3 / stage-05 已完成并关闭，M4 / stage-06 已程序性关闭。M4R01–M4R06 已在 `stage-07` 归档；M4R07 的 v2 portable receipt 为 `PASS`，固定 12 次、实际 12 次，第 8 次普通 `recovery_timer` 完成真实 98 秒等待及后端恢复验证。

M4C10 退出矩阵见 `docs/harness/reports/M4C10-mainline-integration-and-acceptance.md`：M1/M3/M4 合同 exact；M4 聚焦 98/98；最终主机权限完整 Rust `--lib` 为 1639 passed / 0 failed / 45 ignored；typecheck、44-entrypoint offline interaction、production build、launcher syntax、定向 rustfmt 与 C09 evidence 复核均通过。受限 sandbox 首跑的 5 个 launcher 静态碰撞和 1 个 PID `lstart` EPERM、等价源码消歧与最终绿灯均透明保留。

当前唯一入口仍是 `docs/plans/2026-08-11-syn-m4-independent-remediation-and-reacceptance-plan-v1.md` 与当前 M4R07 leaf，但只用于文档、独立复核和 Harness 生命周期收口。产品链完成标记是 `docs/harness/reports/M4R07-isolated-product-reacceptance-closeout-behavior-receipt.json` 的 v2 PASS 加 v2 manifest 交叉绑定；`stage-07` 尚未关闭，不得写成 lifecycle 已完成。

第 8 次 UI / Computer Use / PNG / attestation 按当前合同明确为 `NOT_EXECUTED / NOT_APPLICABLE`：这既不是 FAIL，也不是视觉、Accessibility、截图或 Computer Use 的 PASS。真实个人资料、真实项目写入、真实模型/provider/connector、远端、部署和发布均未验。

M5 ProjectSummary、M6 Global Supervisor 成功 consult、M7 memory consumer、M8 connector、M9 退役和 M10 真实全日试点都未激活。本文件只导航，不授权。

用户提出过 5600X / WSL 迁移方向；当前尚未拿到该机器上的改稿、未建立对应 stage / leaf，也没有迁移验收证据，因此它只是待澄清方向，不是当前实现、完成事实或执行入口。
