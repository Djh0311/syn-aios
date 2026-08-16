# M5R03 WorkerReport、独立审查与事实提升

阶段：stage-14 M5 项目主管与执行闭环（事实重整与产品闭环）

目标：执行者提交的内容只先成为声明；只有精确对上真实执行链并经过独立 Review 和 ResultUserDecision 才能成为项目事实。把 M5R01 合同接入正式消费、持久化、readback 与恢复；建立与 producer 分离的 production RuntimeReceiptVerifier；actor 只来自可信 binding；AuthorizationDecision 与 ResultUserDecision 不得复用；executed/manual/offline 分型；重复与中断保持幂等。

来源收据：用户 2026-08-16 明确按实际完成剩余 M5；M5R02 PASS（`93ba9b0`）。

产品：claim ledger、RuntimeReceiptVerifier、Review、ResultUserDecision、project fact promotion

证据：docs/harness/reports/M5R03-worker-report-independent-review-and-fact-promotion.md [新增]

载体：working-copy + 独立内容 commit（opening HEAD=93ba9b0）

允许动：

- docs/contracts/（仅新增 M5 claim/review 补充合同；不改 M1–M4 冻结合同正文与既有 hash）
- prototypes/productized-desktop-shell/src-tauri/src/worker_report.rs（仅消费接线）
- prototypes/productized-desktop-shell/src-tauri/src/m5_orchestration_identity.rs
- prototypes/productized-desktop-shell/src-tauri/src/m5_orchestration_schema.rs
- prototypes/productized-desktop-shell/src-tauri/src/m5_orchestration_store.rs
- prototypes/productized-desktop-shell/src-tauri/src/m5_claim_ledger.rs [新增]
- prototypes/productized-desktop-shell/src-tauri/src/m5_runtime_receipt.rs [新增]
- prototypes/productized-desktop-shell/src-tauri/src/lib.rs（仅本包最小声明）
- prototypes/productized-desktop-shell/src-tauri/Cargo.toml、Cargo.lock（仅最小依赖）
- tasks/2026-08-16-syn-m5r03-worker-report-independent-review-and-fact-promotion-v1.md [新增]
- docs/harness/authorization.json
- docs/harness/plan.md（仅状态记录）
- docs/current-state.md（仅状态记录）
- docs/harness/audit/2026-08.jsonl
- docs/harness/usage/.turn
- docs/harness/reports/M5R03-worker-report-independent-review-and-fact-promotion.md [新增]
- docs/harness/leaves/M5R03-worker-report-independent-review-and-fact-promotion.md（本叶）
- docs/harness/unfinished/M5R03-worker-report-independent-review-and-fact-promotion.md [退场时新增]
- docs/harness/done/2026-08/M5R03-worker-report-independent-review-and-fact-promotion.md [退场时新增]

不许动：

- M1–M4 冻结合同正文与既有 hash；m6_*.rs
- stage-12、D0C04、D0C05
- 真实资料/provider/connector、push/merge/rebase、reset/stash/clean
- 把未验证 claim 或 manual/offline 提升为执行成功事实
- 伪造 Hook receipt、authorization、stage/leaf、测试或 App 证据
