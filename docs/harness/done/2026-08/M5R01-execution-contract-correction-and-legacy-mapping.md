# M5R01 执行合同矫正与旧数据映射

阶段：stage-14 M5 项目主管与执行闭环（事实重整与产品闭环）

目标：一次项目动作只有一套明确身份和完整链路；冻结 ProposalId、AuthorizationDecisionId、AuthorizationId、ProjectId、CorrelationId、OrchestrationId、WorkflowRunId、WorkItemId、NodeId、DispatchId、AttemptId、GrantId、worker RoleSessionId、RuntimeReceiptId、ResultUserDecisionId、trusted actor 与 hash 的 owner、分配时点与精确关系；修正 WorkerReport 为完整 exact join 的 executed 分型；executed/manual/offline 彻底分型；冻结 legacy mapping/quarantine；对当前原型逐项裁决 KEEP/REWRITE/QUARANTINE 并先建立可复现缺口的 red probes。不修改 M1–M4 冻结合同正文。

来源收据：用户 2026-08-16 明确“按计划开始 M5”；REC-00 事实恢复门 PASS（`FACT_FREEZE=PASS / M5R00=NOT_NEEDED / ROUTE=M5R01`），前置矩阵 M1/M2/M3 全部 PASS。

产品：worker_report.rs（executed/manual/offline 分型与 exact join 修正）、m5_* 候选模块按 KEEP/REWRITE 裁决施工、M5 执行身份合同、迁移/quarantine 矩阵

证据：docs/harness/reports/M5R01-execution-contract-correction.md [新增]

载体：working-copy + 独立内容 commit（opening HEAD=32221be）

允许动：

- docs/contracts/（仅新增 M5 执行身份与 WorkerReport 合同；不改 M1–M4 冻结合同正文与既有 hash）
- prototypes/productized-desktop-shell/src-tauri/src/worker_report.rs
- prototypes/productized-desktop-shell/src-tauri/src/m5_orchestration_identity.rs
- prototypes/productized-desktop-shell/src-tauri/src/m5_prepared_attempt.rs
- prototypes/productized-desktop-shell/src-tauri/src/m5_gateway_traits.rs
- prototypes/productized-desktop-shell/src-tauri/src/m5_project_summary.rs
- prototypes/productized-desktop-shell/src-tauri/src/m5_project_supervisor.rs
- prototypes/productized-desktop-shell/src-tauri/src/m5_controlled_execution.rs
- prototypes/productized-desktop-shell/src-tauri/src/lib.rs（仅本包所需最小声明/接线）
- prototypes/productized-desktop-shell/src-tauri/Cargo.toml、Cargo.lock（仅本包所需最小变更）
- tasks/2026-08-16-syn-m5r01-execution-contract-correction-and-legacy-mapping-v1.md [新增]
- docs/harness/authorization.json
- docs/harness/plan.md（仅状态记录）
- docs/current-state.md（仅状态记录）
- docs/harness/audit/2026-08.jsonl
- docs/harness/usage/.turn
- docs/harness/reports/M5R01-execution-contract-correction.md [新增]
- docs/harness/leaves/M5R01-execution-contract-correction-and-legacy-mapping.md（本叶）
- docs/harness/unfinished/M5R01-execution-contract-correction-and-legacy-mapping.md [退场时新增]
- docs/harness/done/2026-08/M5R01-execution-contract-correction-and-legacy-mapping.md [退场时新增]

不许动：

- M1–M4 冻结合同正文（docs/contracts/m1-*、m2-*、m3-*、m4-*、attention-decision、command、identity-scope、event-audit-outbox、object-ref-navigation、project-orchestration、role-session、connector-capability、handoff、memory-personal-model 等既有 hash）
- m6_*.rs、m6_member_directory.rs.bak（M6 候选只读保全）
- stage-12、D0C04、D0C05 及 unfinished/D0C04、D0C05
- 真实资料/项目写入、真实模型/provider/message/connector、凭据、外部网络业务写
- push/merge/rebase/deploy/release；reset/stash/clean；git add -A 吞入混合 WIP
- 伪造 Hook receipt、authorization、stage/leaf、测试或 App 证据
- 默认沿用原型完成状态（必须逐项裁决 KEEP/REWRITE/QUARANTINE）
## 执行记录与完成回执（2026-08-16）

- 结果：`M5R01_EXECUTION_CONTRACT_CORRECTION=PASS`；
- 合同 `docs/contracts/m5-execution-identity-and-worker-report-v1.md` 冻结（继承 M1，未改 M1–M4 正文与 hash）；
- m5_orchestration_identity.rs 补齐 WorkItemId/NodeId/DispatchId/RuntimeReceiptId/ReportId；
- worker_report.rs M5 扩展按合同 REWRITE：executed 完整 exact-join、manual/offline 禁止执行 join、缺省 kind 改 Manual、actor 自报拒绝；
- 验证：`cargo check --lib --offline` PASS；`cargo test --lib --offline -- m5_` 69 passed；`worker_report` 54 passed（含 23 个 M5 分型/red-probe 测试）；
- 原型裁决与 legacy 隔离见任务包与报告；路由下一叶 M5R02 持久编排核心与 ExecutionGrant。
