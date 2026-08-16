# M5R01 执行合同矫正与旧数据映射报告

- 日期：2026-08-16
- 阶段：stage-14 / leaf M5R01
- 结果：`M5R01_EXECUTION_CONTRACT_CORRECTION=PASS`
- 合同：`docs/contracts/m5-execution-identity-and-worker-report-v1.md`（FROZEN，继承 M1 `project-orchestration-v1.md`，未修改 M1–M4 任何正文与 hash）

## 1. 身份链冻结

- 冻结 ProposalId / AuthorizationDecisionId / AuthorizationId / ProjectId / CorrelationId / OrchestrationId / WorkflowRunId / WorkItemId / NodeId / DispatchId / AttemptId / GrantId / worker RoleSessionId / RuntimeReceiptId / ResultUserDecisionId / trusted actor / hash 的 owner 与分配时点，全部继承 M1 合同语义（owner 表见合同 §1）；
- 裁定 CorrelationId ≠ OrchestrationId（command_gateway / project_orchestration，no_alias，经 Proposal.root_correlation_id 解析）；AuthorizationDecisionId ≠ ResultUserDecisionId；
- m5_orchestration_identity.rs 补齐 WorkItemId、NodeId、DispatchId、RuntimeReceiptId、ReportId 五种缺型；既有 Correlation/Orchestration/WorkflowRun/Attempt/Proposal/Authorization/Decision 类型 KEEP。

## 2. WorkerReport 分型与精确 join（REWRITE）

worker_report.rs M5 扩展按合同重写：

- executed 报告要求完整 exact join：project_id、orchestration_id、workflow_run_id、work_item_id、node_id、dispatch_id、attempt_id、grant_id、worker_role_session_id、authoritative_receipt_ref、report_hash、execution_receipt、trusted actor（含非空 authentication_method）；
- manual/offline 报告禁止携带 dispatch/attempt/grant/worker_role_session/authoritative_receipt_ref/workflow_run_id 与 execution_receipt；只需 project + orchestration 上下文；
- 修复缺口：`ReportKind::default()` 由 Execution 改为 Manual —— 缺省 ReportKind 永不自动成为执行报告；
- actor 自报拒绝：执行报告必须有 worker RoleSession 绑定且 actor 有可信认证方式。

## 3. Red probes（全部通过）

- 缺省 kind 不是 Execution，且缺字段输入 fail；
- executed 缺 workflow_run_id / work_item_id / node_id / dispatch_id / grant_id / worker_role_session_id / receipt_ref / report_hash / receipt 任一 → fail closed；
- actor 无 RoleSession 绑定 / 无认证 → 拒绝；
- manual 带 dispatch/grant/attempt/receipt、offline 带 RoleSession → 拒绝；
- 完整精确 join executed 通过；manual/offline 只带项目上下文通过。

## 4. 原型裁决与 legacy 隔离

- m5_orchestration_identity.rs：KEEP + 补缺；worker_report.rs M5 扩展：REWRITE；
- m5_prepared_attempt.rs、m5_gateway_traits.rs、m5_project_summary.rs、m5_project_supervisor.rs、m5_controlled_execution.rs：KEEP（候选素材，M5R02–M5R06 施工）；
- m6_*.rs、m6_member_directory.rs.bak：QUARANTINE 只读保全，不进入 M5 载体；
- 旧数据：无法精确映射 M1 执行链的只隔离不补齐；缺失 join 的旧 report 进 RECORDED_UNVERIFIED / QUARANTINED；物理删除留 M9。

## 5. 验证

- `cargo check --lib --offline`：PASS（无 error；既有 warning debt 未清零）
- `cargo test --lib --offline -- m5_`：69 passed / 0 failed
- `cargo test --lib --offline -- worker_report`：54 passed / 0 failed（含 23 个 M5 分型与 red-probe 测试）
- `git diff --check`：待 commit 前通过

## 6. 边界与下一步

- 跨项目串接的 store 级检测（同 report 内无法自证）由 M5R02 持久 store/UoW 落地；本合同只在报告层面要求全部 join 在场；
- 路由：M5R02 持久编排核心与 ExecutionGrant（mint/persist/readback/dispatch 原子链与 production gateway）。
