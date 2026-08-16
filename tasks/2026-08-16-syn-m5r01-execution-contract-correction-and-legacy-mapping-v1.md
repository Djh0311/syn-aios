# SYN-M5R01: 执行合同矫正与旧数据映射

日期: 2026-08-16
阶段: M5 (Stage-14) / leaf M5R01
状态: COMPLETE
优先级: HIGH

## 目标

一次项目动作只有一套明确身份和完整链路，不靠几个字符串拼出看似有效的执行记录。冻结执行身份 owner/时点/关系，修正 WorkerReport 为完整 exact-join 的 executed 分型，executed/manual/offline 彻底分型，冻结 legacy mapping/quarantine，对当前原型逐项裁决 KEEP/REWRITE/QUARANTINE 并建立可复现缺口的 red probes。

## 范围

### 1. 执行身份链冻结

继承 M1 `project-orchestration-v1.md` 冻结语义，M5 实现读取见 `docs/contracts/m5-execution-identity-and-worker-report-v1.md`：

- ProposalId / AuthorizationDecisionId / AuthorizationId / ProjectId / CorrelationId / OrchestrationId / WorkflowRunId / WorkItemId / NodeId / DispatchId / AttemptId / GrantId / worker RoleSessionId / RuntimeReceiptId / ResultUserDecisionId / trusted actor / hash 的 owner 与分配时点；
- 裁定 CorrelationId ≠ OrchestrationId（不同 owner、no_alias），AuthorizationDecisionId ≠ ResultUserDecisionId；
- 补全缺失身份类型：WorkItemId、NodeId、DispatchId、RuntimeReceiptId、ReportId（m5_orchestration_identity.rs）。

### 2. WorkerReport 分型与精确 join

- executed 报告必须完整核对 project + orchestration + run + item + node + dispatch + attempt + grant + RoleSession + authoritative receipt + trusted actor + hash；
- executed/manual/offline 彻底分型；manual/offline 禁止携带任何执行 join；
- 缺省 ReportKind 不得自动成为执行报告（修复 `ReportKind::default()==Execution` 缺口）；
- actor 只来自可信 session/binding，自报拒绝。

### 3. 原型裁决

- m5_orchestration_identity.rs: KEEP + 补缺；
- worker_report.rs M5 扩展: REWRITE（精确 join）；
- m5_prepared_attempt.rs / m5_gateway_traits.rs / m5_project_summary.rs / m5_project_supervisor.rs / m5_controlled_execution.rs: KEEP（候选素材，M5R02+ 施工）；
- m6_*.rs 及 .bak: QUARANTINE 只读保全。

### 4. 旧数据映射与隔离

- 无法精确映射到 M1 执行链的旧记录只隔离不补齐；缺失 join 的旧 report 进入 RECORDED_UNVERIFIED / QUARANTINED；物理删除留到 M9。

## 验证结果

- `cargo check --lib --offline`: PASS（无 error）
- M5 定向测试: 69 passed（m5_ 过滤，含 identity 补缺）
- worker_report 全量: 54 passed（含 23 个 M5 分型/red-probe 测试）
- red probes: 缺省不可执行、缺 join fail closed、Grant 缺失拒绝、actor 自报拒绝、manual/offline 携带执行 join 拒绝 全部通过

## 退出依据

M5R01 合同（docs/contracts/m5-execution-identity-and-worker-report-v1.md）已冻结；任一 ID 错配/跨项目串接/actor 自报/Grant 缺失在业务写前 fail closed；non-test build 通过；现有单测仅作回归输入。路由下一叶：M5R02 持久编排核心与 ExecutionGrant。
