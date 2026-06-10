# Root Treatment / R3-A3 Fixture Only Dual Write Transaction Rehearsal v1

日期：2026-06-11

状态：待执行。本文是 Root Treatment / Stage R 的 R3-A3 任务包，用于在 R3-A2 temp DB apply importer / export dry-run 基础上，做 fixture-only dual-write transaction rehearsal：证明 DB 写入与 JSON projection / rollback manifest 可以在临时目录中保持一致。R3-A3 仍不是生产双写期，不切真实产品读写路径，不迁移真实 JSON / sidecar。

## 0. 全局主管理解

已知事实：

- R-Preflight、R0、R1、R2-B1 到 R2-B10、R2 closing / R3 preflight review 已完成。
- R3-P0 SQLite schema / importer / rollback contract freeze 已完成，completion commit：`7022f03d20c77c56a84e9cc9bd2b32aca9b786e6`。
- R3-A1 SQLite schema file + temp DB initializer + idempotent dry-run importer + fixtures 已完成，completion commit：`c6cb5634e79edd9ddba1b1b737c1953806649069`。
- R3-A2 temp DB apply importer、schema hardening、transaction failure injection 和 DB -> JSON export dry-run 已完成，completion commit：`ea982932cd3510487187e710991f20fb9d7467db`。
- R3 SQLite 收口是多 agent 并行真实执行的硬门槛；R3-A3 仍不解锁多 agent 并行真实执行。
- Stage L / L1-L6、K3-B1 retry、K3-B2 和 backlog 功能仍冻结为 `deferred_during_root_treatment`。

本任务核心判断：

```text
R3-A3 只在临时 DB + 临时 JSON projection root 内演练“DB 与 JSON projection 一致写入 / 失败回滚 / 可导出恢复”，不得把它接到生产 store 或产品命令路径。
```

## 1. Execution Mode

Execution Mode：Multi-agent exception, sequential implementation line。

Multi-Agent Justification：

- R3-A3 涉及 storage transaction、dual-write 语义、rollback manifest 和一致性校验，风险高，适合复用现有 Stage R 开发线。
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

- 本任务是生产双写前的关键 rehearsal；错误可能污染后续迁移判断。

Escalation Trigger：

- 任何涉及生产 DB、真实数据迁移、产品读写路径接入、`.codex`、secret / transcript、schema 合同变更、写入范围扩张、测试失败无法解释、或 rollback/export 边界不清的情况。

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
- `docs/plans/2026-06-11-root-treatment-r3-sqlite-schema-importer-rollback-contract-v1.md`
- `tasks/2026-06-11-root-treatment-r3-a1-sqlite-schema-and-idempotent-importer-dry-run-v1.md`
- `tasks/2026-06-11-root-treatment-r3-a2-apply-importer-contract-tests-schema-hardening-and-export-dry-run-v1.md`
- `evidence/2026-06-11-root-treatment-r3-a2-supervisor-checkpoint-v1.md`
- `handoffs/2026-06-11-root-treatment-r3-a2-supervisor-checkpoint-v1-result.md`

建议读取的代码：

- `prototypes/productized-desktop-shell/src-tauri/Cargo.toml`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_schema.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_importer.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_apply.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_exporter.rs`
- `scripts/harness/workbench-shape-gate.js`

## 4. 目标

R3-A3 必须完成：

- 新增 fixture-only dual-write rehearsal 模块或扩展现有 R3 SQLite module，显式传入：
  - fixture source root。
  - temp DB path。
  - temp JSON projection root。
  - rollback manifest path。
- 所有写入必须限制在 temp path 或 `src-tauri/fixtures/r3-a3/**` 下；不得推导或写入生产数据目录。
- Dual-write rehearsal 必须先写 temp DB，再生成 JSON projection 到 temp projection root；projection 必须来自 DB export dry-run，而不是直接复制 source fixture。
- 成功时必须生成 rollback manifest，记录 DB path、projection root、source root hash、projected files、hashes、counts、redaction policy 和 recovery instructions。
- 失败注入必须覆盖：
  - before DB apply。
  - after DB apply before projection write。
  - after first projection file before manifest。
  - before manifest commit。
  - after manifest commit。
- 失败在 manifest commit 前必须能清理 projection partial files，且不得把 partial 写成 completed。
- 失败在 DB apply 后但 projection 前，必须保留 DB committed rows 并返回 `projection_failed_after_db_commit` 或等价状态，不冒充 transaction rollback。
- Rehearsal 重跑必须幂等：相同 fixture / same hash 不重复写 DB row，不重复生成冲突 manifest。
- Projection root 中不得包含 prompt body、full transcript、secret/token/credential/keychain/OAuth/provider credential/rollout body。
- 新增 R3-A3 fixtures，覆盖 dual-write success、idempotent rerun、projection failure cleanup、manifest failure、sensitive redaction 和 rollback manifest。
- 新增 focused Rust tests。
- 写 R3-A3 evidence / handoff。

## 5. 允许读取

允许读取：

- `product-line` 内源码、文档、任务包、evidence、handoff、脚本、git 元数据。
- R3-P0 / R3-A1 / R3-A2 的 evidence / handoff / supervisor checkpoint。
- 仓库内 `src-tauri/fixtures/r3-a1/**`、`src-tauri/fixtures/r3-a2/**` 和新增 `src-tauri/fixtures/r3-a3/**`。

禁止读取：

- `/Users/yoyi/.codex`
- secret、token、`.env`、keychain、OAuth、provider credential、完整 transcript、rollout。
- 用户真实项目数据，除非它已经作为本仓库内测试 fixture 明确存在。

## 6. 允许写入

允许写入：

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`：仅允许增加新模块声明；不得新增 `#[tauri::command]`、app startup hook 或产品路径调用。
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_schema.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_apply.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_exporter.rs`
- 可新增 `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_dual_write.rs`，仅用于 fixture-only rehearsal，必须低于 3,000 行。
- `prototypes/productized-desktop-shell/src-tauri/fixtures/r3-a3/**`
- `evidence/2026-06-11-root-treatment-r3-a3-fixture-only-dual-write-transaction-rehearsal-v1.md`
- `handoffs/2026-06-11-root-treatment-r3-a3-fixture-only-dual-write-transaction-rehearsal-v1-result.md`

默认不更新：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- Root Treatment 官方计划

入口同步由主管线 checkpoint 统一处理。

## 7. 禁止事项

R3-A3 禁止：

- 不创建生产 DB。
- 不写用户真实数据目录。
- 不迁移真实 `workflow-state.v0.json` 或 sidecar。
- 不修改任何真实 JSON / sidecar。
- 不切任何产品读写路径到 DB。
- 不在 app startup / Tauri command / UI 中接入 dual-write rehearsal。
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
- 不把 fixture-only rehearsal 冒充为生产双写期开始。
- 不夹带 R4 前端按页读模型或 UI 瘦身。

## 8. 形状影响

- 任务类型：治理任务包 / SQLite migration rehearsal。
- 新增代码落点：建议 `workbench_sqlite_dual_write.rs`。
- 触碰棘轮文件：`src-tauri/src/lib.rs`，只允许新增 1 行 module declaration；不得增长业务逻辑。
- 新文件上限：
  - Rust 新文件必须低于 3,000 行。
  - 任何 fixture helper 必须低于 500 行。
- 是否新增 Tauri command：否。
- 是否新增 sidecar JSON 种类：否。
- 是否需要 shape gate 豁免：否。
- 本任务规划基线 commit：`ea982932cd3510487187e710991f20fb9d7467db`。
- 本任务完成 commit：待主管线回收后记录。

## 9. Fixture 矩阵

必须至少新增并测试：

- `dual-write-valid-core-chain`：DB apply + projection + rollback manifest success。
- `dual-write-idempotent-rerun`：同一 fixture 重跑，DB rows / projected hashes 稳定。
- `dual-write-after-db-before-projection-failure`：DB committed，projection 未写，状态明确为 projection failure。
- `dual-write-after-first-projection-before-manifest-failure`：projection partial cleanup，manifest 不标 completed。
- `dual-write-before-manifest-commit-failure`：projection 已可清理或标 incomplete，manifest 不完成。
- `dual-write-after-manifest-commit`：completed manifest 存在，projection hashes 可校验。
- `dual-write-sensitive-redaction`：projection 不含 forbidden sensitive keys。
- `rollback-manifest-recovery-dry-run`：从 manifest 生成 recovery plan，不执行真实恢复。

可以复用 R3-A2 fixtures 作为输入，但必须新增 R3-A3 fixtures 或 manifest，明确每个测试覆盖点。

## 10. 验收标准

R3-A3 可接受为：

- Fixture-only dual-write rehearsal 已实现。
- 所有写入只发生在 temp / R3-A3 fixture path。
- DB projection JSON 来自 DB export dry-run，不直接复制 source fixture。
- Projection partial failure 不冒充完成。
- Manifest commit 前失败有清理 / incomplete 证据。
- Rehearsal 重跑幂等。
- Rollback manifest / recovery dry-run 有 focused tests。
- Projection 不输出 prompt body、full transcript、secret/token/credential/rollout body。
- 未创建生产 DB。
- 未迁移真实数据。
- 未新增 Tauri command。
- 未访问 `/Users/yoyi/.codex`。
- shape gate 通过。
- focused cargo tests、`cargo test --lib`、`cargo fmt -- --check`、`git diff --check` 通过，或如有环境失败则完整记录并不得冒充通过。
- evidence / handoff 记录 start commit、end commit 或未提交状态、写入范围、验证结果、P0/P1/P2 和禁止项确认。

R3-A3 不接受为：

- R3 SQLite 迁移开始或完成。
- 生产 DB 创建完成。
- 生产双写期开始。
- 读切 DB 完成。
- JSON / sidecar 停写。
- rollback production workflow 完成。
- 多 agent 并行真实执行解锁。
- Stage L / K3-B1 / K3-B2 已恢复。

## 11. 建议验证命令

必须跑：

```bash
node scripts/harness/workbench-shape-gate.js --mode check
cargo test --lib sqlite_schema
cargo test --lib sqlite_apply_importer
cargo test --lib sqlite_export_dry_run
cargo test --lib sqlite_dual_write
cargo test --lib workflow_state
cargo test --lib
cargo fmt -- --check
git diff --check
git status --short
```

若 filtered cargo tests no-match，必须记录 exact no-match，并用实际存在的 module/test name 或更广泛 `cargo test --lib` 覆盖，不能把 no-match 冒充通过。

必须额外扫描：

```bash
rg -n 'Command::new\("codex"\)|codex exec|codex exec resume|/Users/yoyi/\.codex|auth|token|provider credential|prompt_body|full transcript|rollout' prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_schema.rs prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_apply.rs prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_exporter.rs prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_dual_write.rs prototypes/productized-desktop-shell/src-tauri/fixtures/r3-a3
rg -n 'workflow-state|formal-memories|memory-candidates|observations|runtime-log|runtime-logs|plan-authorizations|project-proposals|real-execution-product-commands|session-continuations' prototypes/productized-desktop-shell/src-tauri/fixtures/r3-a3 prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_dual_write.rs prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_exporter.rs
```

如果 optional 文件不存在，扫描命令可改为实际存在的 R3-A3 文件清单，但 evidence 必须说明。

## 12. 交付物

必须新增：

- `evidence/2026-06-11-root-treatment-r3-a3-fixture-only-dual-write-transaction-rehearsal-v1.md`
- `handoffs/2026-06-11-root-treatment-r3-a3-fixture-only-dual-write-transaction-rehearsal-v1-result.md`

Evidence / handoff 必须包含：

- STATUS。
- CHANGED_FILES。
- dual-write rehearsal summary。
- projection / export dry-run summary。
- rollback manifest / recovery dry-run summary。
- failure injection summary。
- fixture coverage matrix。
- forbidden sensitive field handling。
- checks run。
- P0 / P1 / P2。
- boundary confirmation。
- requests / next recommendation。

## 13. P0 / P1 / P2

预期 P0：

- 无。

预期 P1：

- Dual-write rehearsal 若能写非 temp / fixture path，必须阻断。
- Projection 若直接复制 source fixture 而非 DB export projection，必须阻断。
- Projection partial failure 若被标记为 completed，必须阻断。
- Projection 若输出 prompt body、full transcript、secret/token/credential/rollout body，必须阻断。
- 如果新增 Tauri command、startup hook、UI 接入或产品读写路径切换，必须阻断。

预期 P2：

- 仍不是生产双写期。
- 仍不是 read-cut DB。
- Rollback manifest 先做 dry-run recovery plan，不要求真实恢复生产数据。
- Product Command / continuation / runtime log 的真实单事务产品写路径仍待后续 R3 task。

## 14. 回传格式

开发线回传必须包含：

```text
STATUS: DONE / DONE_WITH_CONCERNS / NEEDS_DECISION / BLOCKED
CHANGED_FILES
SUMMARY
DUAL-WRITE REHEARSAL SUMMARY
PROJECTION / EXPORT DRY-RUN SUMMARY
ROLLBACK MANIFEST / RECOVERY DRY-RUN SUMMARY
FAILURE INJECTION SUMMARY
FIXTURE COVERAGE MATRIX
FORBIDDEN SENSITIVE FIELD HANDLING
EVIDENCE / CHECKS RUN
P0 / P1 / P2
BOUNDARY CONFIRMATION
REQUESTS
```

若发现任务包与当前代码事实冲突，先返回 `NEEDS_DECISION`，不要自行扩大范围。
