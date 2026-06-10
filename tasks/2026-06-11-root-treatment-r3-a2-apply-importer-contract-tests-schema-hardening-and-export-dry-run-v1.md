# Root Treatment / R3-A2 Apply Importer Contract Tests Schema Hardening And Export Dry Run v1

日期：2026-06-11

状态：已完成。本文是 Root Treatment / Stage R 的 R3-A2 任务包，用于在 R3-A1 schema / dry-run importer 基础上，补齐临时 DB apply importer 合同测试、schema constraint hardening、transaction crash fixtures 和 DB -> JSON export dry-run。completion commit：`ea982932cd3510487187e710991f20fb9d7467db`。

本任务仍然只做临时 DB / fixture / dry-run / contract tests；不创建生产 DB，不迁移真实 JSON / sidecar，不双写，不切 DB 读写路径，不改 workflow state 顶层 schema，不执行真实 Codex，不读写 `/Users/yoyi/.codex`。

## 0. 全局主管理解

已知事实：

- R-Preflight、R0、R1、R2-B1 到 R2-B10、R2 closing / R3 preflight review 已完成。
- R3-P0 SQLite schema / importer / rollback contract freeze 已完成，completion commit：`7022f03d20c77c56a84e9cc9bd2b32aca9b786e6`。
- R3-A1 SQLite schema file + temp DB initializer + idempotent dry-run importer + fixtures 已完成，completion commit：`c6cb5634e79edd9ddba1b1b737c1953806649069`。
- R3-A1 只接受为最小 schema module、临时 / fixture DB initializer、离线 dry-run importer 和 fixture 矩阵完成；不接受为 R3 SQLite 迁移开始或完成。
- R3 SQLite 收口是多 agent 并行真实执行的硬门槛；R3-A2 仍不解锁多 agent 并行真实执行。
- Stage L / L1-L6、K3-B1 retry、K3-B2 和 backlog 功能仍冻结为 `deferred_during_root_treatment`。

本任务核心判断：

```text
R3-A2 要证明 apply importer、schema 约束、事务失败和 DB -> JSON export dry-run 的机制可靠，但仍只能作用于临时 DB / fixture，不能开始生产迁移。
```

## 1. Execution Mode

Execution Mode：Multi-agent exception, sequential implementation line。

Multi-Agent Justification：

- R3-A2 是 DB / importer / transaction / export 方向的高风险治理任务，需要复用 Stage R 开发线的上下文。
- 本任务仍只派发一条开发线，不并行多写同一批 SQLite 文件。
- 全局主管保留集成、fresh verify、commit 和入口同步权。

Coordination Cost：

- 只复用现有 Stage R 工作线。
- 实现线不得更新入口文档，不得提交；主管线回收后统一 checkpoint。

Fallback If Coordination Fails：

- 若开发线返回 `NEEDS_DECISION` / `BLOCKED` / 越界读写，主管线停止集成，回到单线审查并重写任务包边界。

## 2. Model Routing

Assigned Model：strongest available if runtime supports it; otherwise runtime default。

Reasoning Effort：high / xhigh。

Reason For Choice：

- 本任务涉及 SQLite 写入、事务失败、schema 约束、export dry-run 和未来迁移硬门槛，错误成本高。

Escalation Trigger：

- 任何涉及生产 DB、真实数据迁移、读写路径切换、`.codex`、secret / transcript、schema 合同变更、写入范围扩张、测试失败无法解释、或 Product Command / continuation / runtime log transaction 边界不清的情况。

## 3. 权威依据

必须读取并服从：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `AGENTS.md`
- `codex-multi-agent-safe-collaboration.md`
- `docs/plans/2026-06-10-root-treatment-official-development-plan-v1.md`
- `docs/plans/2026-06-10-root-treatment-r0-shape-gate-and-governance-task-package-rule-v1.md`
- `docs/plans/2026-06-11-root-treatment-r3-sqlite-schema-importer-rollback-contract-v1.md`
- `tasks/2026-06-11-root-treatment-r3-a1-sqlite-schema-and-idempotent-importer-dry-run-v1.md`
- `evidence/2026-06-11-root-treatment-r3-a1-sqlite-schema-and-idempotent-importer-dry-run-v1.md`
- `handoffs/2026-06-11-root-treatment-r3-a1-sqlite-schema-and-idempotent-importer-dry-run-v1-result.md`
- `evidence/2026-06-11-root-treatment-r3-a1-supervisor-checkpoint-v1.md`
- `handoffs/2026-06-11-root-treatment-r3-a1-supervisor-checkpoint-v1-result.md`

建议读取的代码：

- `prototypes/productized-desktop-shell/src-tauri/Cargo.toml`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_schema.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_importer.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workflow_state_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/formal_memory_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_candidate_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/observation_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/runtime_log_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/session_continuation_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/real_execution_command.rs`
- `scripts/harness/workbench-shape-gate.js`

## 4. 目标

R3-A2 必须完成：

- 加固 R3-A1 schema 约束：补必要 `UNIQUE`、`FOREIGN KEY`、`CHECK` 或 index，至少覆盖 importer/source/record、workflow core、memory、governance、product command / continuation / runtime log 的核心幂等路径。
- 新增临时 DB apply importer：显式传入 fixture root 和 temp DB path，只允许 temp path 或 R3 fixture path；不得推导生产 DB。
- Apply importer 必须使用 SQLite transaction；同一 fixture 重复 apply 必须幂等，不制造重复 row。
- Apply importer 必须能把 R3-A1 dry-run report 中 accepted records 写入 temp DB 的 metadata/source/record 表和最小 domain 表。
- Apply importer 遇到 duplicate-different-hash、revision conflict、corrupt primary、forbidden sensitive field 时必须拒绝 batch，且不得留下 partial domain rows。
- 新增 transaction failure injection 测试点，至少覆盖：
  - before DB begin。
  - after DB begin before first insert。
  - after import batch/source insert before domain insert。
  - after first domain insert before commit。
  - before commit。
  - after commit before export dry-run manifest。
- 新增 DB -> JSON export dry-run：从 temp DB 生成 export manifest、record counts、file hash / projected hash 和 redaction manifest；不得写出真实 JSON / sidecar 文件。
- Export dry-run 必须能重建至少这些 projection 的 JSON value 或 manifest：
  - `workflow-state.v0.json` top-level shape。
  - `formal-memories.v1.json` 或 `memory-candidates.v1.json` 至少一类记忆 sidecar。
  - `runtime-logs.v1.json` canonical runtime log sidecar。
  - `real-execution-product-commands.v1.json` / `session-continuations.v1.json` 之一。
- Export dry-run 不得输出 prompt body、full transcript、secret/token/credential/keychain/OAuth/provider credential/rollout body。
- 新增 R3-A2 fixtures，覆盖 apply success、idempotent reapply、conflict rollback、crash injection、export dry-run 和 sensitive rejection。
- 新增 focused Rust tests。
- 写 R3-A2 evidence / handoff。

## 5. 允许读取

允许读取：

- `product-line` 内源码、文档、任务包、evidence、handoff、脚本、git 元数据。
- R0 / R1 / R2 / R3-P0 / R3-A1 的 evidence / handoff / supervisor checkpoint。
- 仓库内 `src-tauri/fixtures/r3-a1/**` 和新增 `src-tauri/fixtures/r3-a2/**`。

禁止读取：

- `/Users/yoyi/.codex`
- secret、token、`.env`、keychain、OAuth、provider credential、完整 transcript、rollout。
- 用户真实项目数据，除非它已经作为本仓库内测试 fixture 明确存在。

## 6. 允许写入

允许写入：

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`：仅允许增加新模块声明；不得新增 `#[tauri::command]`、app startup hook 或产品路径调用。
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_schema.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_importer.rs`
- 可新增 `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_exporter.rs`，仅用于 export dry-run，必须低于 3,000 行。
- 可新增 `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_apply.rs`，仅用于 temp DB apply importer / transaction tests，必须低于 3,000 行。
- `prototypes/productized-desktop-shell/src-tauri/fixtures/r3-a2/**`
- `evidence/2026-06-11-root-treatment-r3-a2-apply-importer-contract-tests-schema-hardening-and-export-dry-run-v1.md`
- `handoffs/2026-06-11-root-treatment-r3-a2-apply-importer-contract-tests-schema-hardening-and-export-dry-run-v1-result.md`

默认不更新：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- Root Treatment 官方计划

入口同步由主管线 checkpoint 统一处理。

## 7. 禁止事项

R3-A2 禁止：

- 不创建生产 DB。
- 不写用户真实数据目录。
- 不迁移真实 `workflow-state.v0.json` 或 sidecar。
- 不修改任何真实 JSON / sidecar。
- 不双写 DB + JSON。
- 不切任何产品读写路径到 DB。
- 不在 app startup / Tauri command / UI 中接入 DB initializer、apply importer 或 exporter。
- 不改 workflow state 顶层 schema。
- 不新增 sidecar store 或 sidecar JSON 种类。
- 不新增 Tauri command。
- 不改真实 Codex runner。
- 不执行真实 `codex exec` / `codex exec resume`。
- 不发送 prompt。
- 不读写 `/Users/yoyi/.codex`。
- 不读取 secret、token、`.env`、keychain、OAuth、provider credential、完整 transcript 或 rollout。
- 不启动 Tauri / Browser / Chrome / Vite / 截图工具。
- 不启动 Stage L / K3-B1 retry / K3-B2。
- 不解冻 backlog 功能。
- 不把 temp DB apply importer 或 export dry-run 冒充为 R3 迁移开始 / 完成。
- 不夹带 R2 inline tests 巨石迁移。
- 不夹带 R4 前端按页读模型或 UI 瘦身。

## 8. 形状影响

- 任务类型：治理任务包 / SQLite implementation prep。
- 新增代码落点：`workbench_sqlite_apply.rs`、`workbench_sqlite_exporter.rs` 可选；若实现线判断更简单，也可只扩展 `workbench_sqlite_importer.rs`。
- 触碰棘轮文件：`src-tauri/src/lib.rs`，只允许新增 1-3 行 module declaration；不得增长业务逻辑。
- 新文件上限：
  - Rust 新文件必须低于 3,000 行。
  - 任何 fixture helper 必须低于 500 行。
- 是否新增 Tauri command：否。
- 是否新增 sidecar JSON 种类：否。
- 是否需要 shape gate 豁免：否。
- 本任务基线 commit：`055e51d8ebebb82f12b125c1afac40a756a719e9`。
- 本任务完成 commit：待主管线回收后记录。

## 9. Fixture 矩阵

必须至少新增并测试：

- `apply-valid-core-chain`：可从 workflow + memory + product command / runtime fixture apply 到 temp DB。
- `apply-idempotent-reapply`：同一 fixture apply 两次，第二次 skips / unchanged，不新增重复 row。
- `apply-conflict-rollback`：duplicate natural key + different hash，transaction rollback，无 partial domain rows。
- `apply-revision-conflict-rollback`：expected_revision != revision，transaction rollback。
- `apply-corrupt-primary-reject`：primary corrupt，不写 DB domain rows。
- `apply-sensitive-reject`：prompt body / secret / full transcript / rollout body 被拒绝，不写 DB domain rows。
- `crash-after-source-before-domain`：failure injection 后 rollback。
- `crash-after-domain-before-commit`：failure injection 后 rollback。
- `export-dry-run-workflow-runtime`：从 temp DB 生成 workflow / runtime / product command projection manifest。
- `runtime-log-alias-export-policy`：export dry-run 只输出 canonical `runtime-logs.v1.json`，singular alias 只作为 legacy source ref。

可以复用 R3-A1 fixtures 作为输入，但必须新增 R3-A2 fixtures 或 manifest，明确每个测试覆盖点。

## 10. 验收标准

R3-A2 可接受为：

- Temp DB apply importer 已实现，且只能写入显式传入的 temp / fixture DB path。
- Apply importer 使用 SQLite transaction。
- Reapply 幂等、conflict rollback、revision rollback、corrupt reject、sensitive reject 有测试覆盖。
- Schema 约束 / index / FK / CHECK 有 focused tests 或 PRAGMA / sqlite_master 断言。
- DB -> JSON export dry-run 能输出 deterministic manifest 和 projection hashes。
- Export dry-run 不写真实 JSON / sidecar 文件。
- Export dry-run 不输出 prompt body、full transcript、secret/token/credential/rollout body。
- 未创建生产 DB。
- 未迁移真实数据。
- 未新增 Tauri command。
- 未访问 `/Users/yoyi/.codex`。
- shape gate 通过。
- focused cargo tests、`cargo test --lib`、`cargo fmt -- --check`、`git diff --check` 通过，或如有环境失败则完整记录并不得冒充通过。
- evidence / handoff 记录 start commit、end commit 或未提交状态、写入范围、验证结果、P0/P1/P2 和禁止项确认。

R3-A2 不接受为：

- R3 SQLite 迁移开始或完成。
- 生产 DB 创建完成。
- 双写期开始。
- 读切 DB 完成。
- JSON / sidecar 停写。
- DB -> JSON export 写盘完成。
- transaction boundary 全部产品化完成。
- 多 agent 并行真实执行解锁。
- Stage L / K3-B1 / K3-B2 已恢复。

## 11. 建议验证命令

必须跑：

```bash
node scripts/harness/workbench-shape-gate.js --mode check
cargo test --lib sqlite_schema
cargo test --lib sqlite_importer_dry_run
cargo test --lib sqlite_apply_importer
cargo test --lib sqlite_export_dry_run
cargo test --lib workflow_state
cargo test --lib
cargo fmt -- --check
git diff --check
git status --short
```

若 filtered cargo tests no-match，必须记录 exact no-match，并用实际存在的 module/test name 或更广泛 `cargo test --lib` 覆盖，不能把 no-match 冒充通过。

必须额外扫描：

```bash
rg -n 'Command::new\("codex"\)|codex exec|codex exec resume|/Users/yoyi/\.codex|auth|token|provider credential|prompt_body|full transcript|rollout' prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_schema.rs prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_importer.rs prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_apply.rs prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_exporter.rs prototypes/productized-desktop-shell/src-tauri/fixtures/r3-a2
rg -n 'workflow-state|formal-memories|memory-candidates|observations|runtime-log|runtime-logs|plan-authorizations|project-proposals|real-execution-product-commands|session-continuations' prototypes/productized-desktop-shell/src-tauri/fixtures/r3-a2 prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_importer.rs prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_apply.rs prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_exporter.rs
```

如果 optional 文件不存在，扫描命令可改为实际存在的 R3-A2 文件清单，但 evidence 必须说明。

## 12. 交付物

必须新增：

- `evidence/2026-06-11-root-treatment-r3-a2-apply-importer-contract-tests-schema-hardening-and-export-dry-run-v1.md`
- `handoffs/2026-06-11-root-treatment-r3-a2-apply-importer-contract-tests-schema-hardening-and-export-dry-run-v1-result.md`

Evidence / handoff 必须包含：

- STATUS。
- CHANGED_FILES。
- schema hardening summary。
- temp DB apply importer summary。
- transaction / failure injection summary。
- export dry-run summary。
- fixture coverage matrix。
- forbidden sensitive field handling。
- runtime-log alias export handling。
- checks run。
- P0 / P1 / P2。
- boundary confirmation。
- requests / next recommendation。

## 13. P0 / P1 / P2

预期 P0：

- 无。

预期 P1：

- Apply importer 若能写非 temp / fixture DB path，必须阻断。
- Apply importer 若在 conflict / crash / sensitive reject 后留下 partial domain rows，必须阻断。
- Export dry-run 若写真实 JSON / sidecar 文件，必须阻断。
- Export dry-run 若输出 prompt body、full transcript、secret/token/credential/rollout body，必须阻断。
- 如果新增 Tauri command、startup hook、UI 接入或产品读写路径切换，必须阻断。

预期 P2：

- Schema v0 仍可能偏 coarse；更细 FK / index 可继续放到后续 R3-A3。
- Export dry-run projection 可以先覆盖核心 fixture，不要求覆盖所有历史 corner case。
- Product Command / continuation / runtime log 的真实单事务产品写路径仍待后续 R3 task。
- Sidecar backup retention harmonization 仍待后续。

## 14. 回传格式

开发线回传必须包含：

```text
STATUS: DONE / DONE_WITH_CONCERNS / NEEDS_DECISION / BLOCKED
CHANGED_FILES
SUMMARY
SCHEMA HARDENING SUMMARY
TEMP DB APPLY IMPORTER SUMMARY
TRANSACTION / FAILURE INJECTION SUMMARY
EXPORT DRY-RUN SUMMARY
FIXTURE COVERAGE MATRIX
FORBIDDEN SENSITIVE FIELD HANDLING
RUNTIME-LOG ALIAS EXPORT HANDLING
EVIDENCE / CHECKS RUN
P0 / P1 / P2
BOUNDARY CONFIRMATION
REQUESTS
```

若发现任务包与当前代码事实冲突，先返回 `NEEDS_DECISION`，不要自行扩大范围。
