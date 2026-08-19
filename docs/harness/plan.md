# 总计划 product-line 唯一基线与 Harness Lite 切换

目标：保护所有既有 WIP，把 product-line 的有效代码和项目事实收敛到一条可复现基线；退出 Adaptive Harness v0.5，改用 Harness Lite 管后续开发生命周期。

阶段：

- [x] 阶段1 代码事实收敛、唯一权威与 Lite 切换
- [x] 阶段2 Lite Agent 规则权威纠正
- [x] 阶段3 M2 主线收口与交接
- [x] 阶段4 M0 产品与文档正本干净基线收口
- [x] 阶段5 M3 角色会话与显式交接
- [x] 阶段6 M4 秘书、Attention 与日常节奏
- [x] 阶段7 M4 独立修正与再验收
- [x] 阶段8 Syn Primary/Edge D0 文档与迁移权威收口
- [x] 阶段9 Syn 5600X/WSL/Tailscale B 只读预检
- [x] 阶段10 Syn 5600X/WSL 原方案 C0 只读配置门
- [x] 阶段11 Syn 5600X/WSL C1 临时链路证明
- [ ] 阶段12 Syn 5600X/WSL C2 长期 SSH 开发通道与 D 源码迁移
- [x] 阶段13 DeepSeek Harness 方法吸收、Syn 原生核心与自升级计划校准
- [x] 阶段14 M5 项目主管与执行闭环（M5 scoped product-chain PASS / stage-14 closed / not released）
- [ ] 阶段15 M6 全局主管与内部组织（active；M6P00、CP1、CP2、CP3 与最终域层候选 `a3d5759` 均获独立 PASS；等待总指导生命周期处置，UI 验收载体为新壳）
- [ ] 阶段16 F2 壳—核心受控桥（首轮候选 `629e4b2` 独立验收 FAIL；`F2C01R01` 已归档；`F2C01R02` 单阻断项返修已归档，待总指导派本阻断项+回归的复核 verdict。不由本轮关闭阶段；壳侧客户端和 SIGKILL/崩溃恢复取证另派）

当前优先级（2026-08-19 F2C01R02 已归档）：stage-15 最终独立 verdict `stage-15-20260819-1123` 已 PASS，但阶段终包明确要求总指导处置关闭，因此 stage-15 仍不勾选完成、不由 F2 修改。`F2C01R02` 已归档；无 current leaf；authorization closed；不关闭 stage-16。其余已过验收项不得重做或推翻。M6P00、CP1、CP2、CP3 与最终域层候选 `a3d575975033f7eb5ec92dab18c24fe97ddb8001` 的既有 PASS 均不被本轮改写。ORG-007 与 21 个 command 的新壳 UI/isolated App 消费仍在 M6S01；F3/F5、M7–M11、Headless Core、Primary 与 authority epoch 均未激活。

上一阶段结论（2026-08-18 closeout）：M5 内容锚 `c91d8fc72bcbf80186736caff841cb7a9b0660d1` / tree `fe2d982267d474631ca4ea7b3f90ed846f72a89d` 与 M5R09 记账 `8e6f59f48d2d90891d3c02396378921e4a2f5d6e` / tree `2043660c9547c6c102ae24414674918ca8215eb0` 已获独立 PASS；M5C01 closeout 内容 `de98d69a363ff82281330fb3b82de82c03a9b484` / tree `b90244a8535c829e96341d42fef39602ef499f6d` 完成 lifecycle、权威状态、载体/WIP 分层与 M6 输入交接。REC-00、M5R00、M5R01–M5R09 与 M5C01 已按各自惯例归档，stage-14 已关闭，当前没有 M5 leaf。结论只到 `M5 SCOPED PRODUCT-CHAIN PASS / NOT_RELEASED`；用户 OSS 门面 `c1025ba` 独立于 M5 候选，OSS-01 保持 unfinished。当时 `stage-15` 尚未建立；其后已由用户明确激活，见上方当前优先级。stage-12 仍开着，D0C04 / D0C05 保持 unfinished；F2/F3/F5、M7–M11、Headless Core、Primary 与 authority epoch 均未激活。

## M5 + M6 当前状态（2026-08-16 修正，非完成声明）

依 `docs/plans/2026-08-16-syn-m5-m6-fact-reconciliation-and-product-closure-plan-v1.md` 独立复核：此前手工勾选的 stage-14/15 完成标记不构成阶段历史，不授予验收地位；以下只记录候选 WIP 事实。

### M5 候选原型（57 定向单测，未绑定阶段退出）
- SYN-PRJ-001..006 方向与产品正本一致：项目主管、执行授权链、WorkerReport 分型、恢复语义与 ProjectSummary 端口。
- 生产调用图只有模块声明；主要 service/gateway/controller 仅 `#[cfg(test)]` 内存实现，无正式 AppState / Tauri command / repository / 普通产品 caller。
- 当前原型允许任意字符串 Grant 使 Attempt 进入 Runnable；stop/retry/resume 不改变持久状态；WorkerReport 完整性检查缺少完整 Grant/Dispatch/Attempt/RoleSession/receipt/actor exact join。

### M6 候选原型（33 定向单测，未绑定阶段退出）
- SYN-ORG-001/002/005 方向与产品正本一致：跨项目 advisory、稳定成员目录与只读咨询。
- 当前 M6 session/Handoff 是平行内存结构，非 M3 RoleSession/Handoff；跨项目 query 不消费正式 M5 ProjectSummaryQueryPort；成员目录与临时 Agent 历史不是受治理持久投影。

### 候选 Rust 实现文件（uncommitted，位于 prototypes/productized-desktop-shell/src-tauri/src/）
- m5_orchestration_identity.rs、m5_prepared_attempt.rs、m5_gateway_traits.rs、m5_project_summary.rs、m5_project_supervisor.rs、m5_controlled_execution.rs
- m6_organization_identity.rs、m6_cross_project_query.rs、m6_global_supervisor_session.rs、m6_member_directory.rs、m6_temporary_agent_history.rs
- worker_report.rs（扩展 ReportKind / ExecutionReceipt / TrustedActor）

### 实施入口
- 按 `docs/plans/2026-08-16-syn-m5-m6-fact-reconciliation-and-product-closure-plan-v1.md` 的 REC-00 → M5R00(仅 GAP) → M5R01–M5R07 → M5 独立验收顺序执行；M5 验收前不激活 M6。
