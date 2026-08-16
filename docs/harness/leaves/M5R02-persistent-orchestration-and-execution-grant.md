# M5R02 持久编排核心与 ExecutionGrant

阶段：stage-14 M5 项目主管与执行闭环（事实重整与产品闭环）

目标：用户确认后的动作可以安全、持久地准备，但 Grant 完整持久化和回读前绝不运行。用正式 store/UoW 落地 Run、WorkItem、worker RoleSession binding、PreparedAttempt、Grant、Dispatch 和 outbox；严格实现 AuthorizationDecision → Authorization → Run/WorkItem + worker RoleSession binding → PreparedAttempt → mint Grant → persist/readback → Dispatch → Runnable → outbox；建立 production ExecutionGrantGateway/ConversationCapabilityGateway，副作用入口只接 GrantId 且由服务端加载完整 immutable Grant；Runner/side-effect 入口登记为 new-grant / guarded-legacy / blocked。

来源收据：用户 2026-08-16 明确“按计划开始 M5”；REC-00 PASS 后 M5R01 已验收（`M5R01_EXECUTION_CONTRACT_CORRECTION=PASS`），合同 `docs/contracts/m5-execution-identity-and-worker-report-v1.md` 冻结。

产品：m5_prepared_attempt.rs、m5_gateway_traits.rs、m5_controlled_execution.rs、m5_orchestration_identity.rs、worker_report.rs（精确 join 消费）、正式 store/UoW adapter、production ExecutionGrantGateway/ConversationCapabilityGateway、dispatch/outbox 接入

证据：docs/harness/reports/M5R02-persistent-orchestration-and-execution-grant.md [新增]

载体：working-copy + 独立内容 commit（opening HEAD=32221be + M5R01 commit）

允许动：

- docs/contracts/（仅新增 M5 持久化与 Grant 合同/增补；不改 M1–M4 冻结合同正文与既有 hash）
- prototypes/productized-desktop-shell/src-tauri/src/m5_orchestration_identity.rs
- prototypes/productized-desktop-shell/src-tauri/src/m5_prepared_attempt.rs
- prototypes/productized-desktop-shell/src-tauri/src/m5_gateway_traits.rs
- prototypes/productized-desktop-shell/src-tauri/src/m5_controlled_execution.rs
- prototypes/productized-desktop-shell/src-tauri/src/m5_project_supervisor.rs（仅所需接线）
- prototypes/productized-desktop-shell/src-tauri/src/worker_report.rs（仅 exact-join 消费接线）
- prototypes/productized-desktop-shell/src-tauri/src/lib.rs（仅本包所需最小声明/接线）
- prototypes/productized-desktop-shell/src-tauri/src/m2_ports.rs、workbench_sqlite_repository.rs、m2_outbox.rs（仅所需 adapter 接线）
- prototypes/productized-desktop-shell/src-tauri/Cargo.toml、Cargo.lock（仅本包所需最小依赖）
- prototypes/productized-desktop-shell/src-tauri/tests/（本包定向测试）
- tasks/2026-08-16-syn-m5r02-persistent-orchestration-and-execution-grant-v1.md [新增]
- docs/harness/authorization.json
- docs/harness/plan.md（仅状态记录）
- docs/current-state.md（仅状态记录）
- docs/harness/audit/2026-08.jsonl
- docs/harness/usage/.turn
- docs/harness/reports/M5R02-persistent-orchestration-and-execution-grant.md [新增]
- docs/harness/leaves/M5R02-persistent-orchestration-and-execution-grant.md（本叶）
- docs/harness/unfinished/M5R02-persistent-orchestration-and-execution-grant.md [退场时新增]
- docs/harness/done/2026-08/M5R02-persistent-orchestration-and-execution-grant.md [退场时新增]

不许动：

- M1–M4 冻结合同正文与既有 hash；m6_*.rs、m6_member_directory.rs.bak（M6 只读保全）
- stage-12、D0C04、D0C05 及 unfinished/D0C04、D0C05
- 真实资料/项目写入、真实模型/provider/message/connector、凭据、外部网络业务写
- push/merge/rebase/deploy/release；reset/stash/clean；git add -A 吞入混合 WIP
- 伪造 Hook receipt、authorization、stage/leaf、测试或 App 证据
- 在 Grant persist/readback 完成前放行 Runnable/dispatch；留未知 Runner 旁路
