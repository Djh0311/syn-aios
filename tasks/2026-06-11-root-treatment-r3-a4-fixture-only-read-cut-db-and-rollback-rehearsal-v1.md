# Root Treatment / R3-A4 Fixture Only Read-Cut DB And Rollback Rehearsal v1

日期：2026-06-11

状态：待执行。本文是 Root Treatment / Stage R 的 R3-A4 任务包，用于在 R3-A3 fixture-only dual-write transaction rehearsal 基础上，演练 fixture-only read-cut DB、JSON fallback、export hash verification 和 rollback recovery dry-run。R3-A4 仍不是生产读切，不创建生产 DB，不切真实产品读写路径，不迁移真实 JSON / sidecar。

## 0. 全局主管理解

已知事实：

- R-Preflight、R0、R1、R2-B1 到 R2-B10、R2 closing / R3 preflight review 已完成。
- R3-P0 SQLite schema / importer / rollback contract freeze 已完成，completion commit：`7022f03d20c77c56a84e9cc9bd2b32aca9b786e6`。
- R3-A1 SQLite schema file + temp DB initializer + idempotent dry-run importer + fixtures 已完成，completion commit：`c6cb5634e79edd9ddba1b1b737c1953806649069`。
- R3-A2 temp DB apply importer、schema hardening、transaction failure injection 和 DB -> JSON export dry-run 已完成，completion commit：`ea982932cd3510487187e710991f20fb9d7467db`。
- R3-A3 fixture-only dual-write transaction rehearsal 已完成，completion commit：`d9e5f0fd637daf7cbb6b117d7a8bac15448c9d8f`。
- R3-A3 只接受为临时 DB + 临时 JSON projection root + R3-A3 fixture root 内的演练完成；不接受为生产双写期、生产 DB、读切 DB、JSON / sidecar 停写或多 agent 并行真实执行解锁。
- R3 SQLite 收口是多 agent 并行真实执行的硬门槛；R3-A4 仍不解锁多 agent 并行真实执行。
- Stage L / L1-L6、K3-B1 retry、K3-B2 和 backlog 功能仍冻结为 `deferred_during_root_treatment`。

本任务核心判断：

```text
R3-A4 只在临时 DB + 临时 JSON projection root + fixture root 内演练“读切 DB / JSON fallback / rollback recovery”，不得把它接到生产 store、app startup、Tauri command、UI 或真实产品读写路径。
```

## 1. Execution Mode

Execution Mode：Multi-agent exception, sequential implementation line。

Multi-Agent Justification：

- R3-A4 涉及 read-cut、fallback、rollback、DB integrity 和 export hash verification，属于高风险存储治理任务。
- 本任务复用既有 Stage R 开发线，仍只派发一条开发线，不并行多写 SQLite 文件。
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

- 本任务是生产 read-cut 前的关键 rehearsal；错误可能让后续把 DB read path 误认为已安全。

Escalation Trigger：

- 任何涉及生产 DB、真实数据迁移、产品读写路径接入、`.codex`、secret / transcript、schema 合同变更、写入范围扩张、测试失败无法解释、read fallback / rollback 边界不清的情况。

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
- `tasks/2026-06-11-root-treatment-r3-a3-fixture-only-dual-write-transaction-rehearsal-v1.md`
- `evidence/2026-06-11-root-treatment-r3-a3-supervisor-checkpoint-v1.md`
- `handoffs/2026-06-11-root-treatment-r3-a3-supervisor-checkpoint-v1-result.md`

建议读取的代码：

- `prototypes/productized-desktop-shell/src-tauri/Cargo.toml`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_schema.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_importer.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_apply.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_exporter.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_dual_write.rs`
- `scripts/harness/workbench-shape-gate.js`

## 4. 目标

R3-A4 必须完成：

- 新增 fixture-only read-cut rehearsal 模块或扩展现有 R3 SQLite module，显式传入：
  - fixture source root。
  - temp DB path。
  - temp JSON projection root。
  - rollback manifest path。
  - optional read-cut / fallback failure injection point。
- 所有写入必须限制在 temp path 或 `src-tauri/fixtures/r3-a4/**` 下；不得推导或写入生产数据目录。
- Read-cut rehearsal 必须从 temp DB 读取 projection / read model，并和 DB export dry-run projection hash 对齐；不得把 source fixture 当成 read-cut 成功证据。
- Read-cut rehearsal 必须能模拟：
  - DB authoritative read 成功。
  - DB unavailable 时 fallback 到已验证 JSON projection。
  - DB integrity / schema mismatch 时 fallback，并返回 degraded status。
  - export hash mismatch 时阻断 read-cut，不把 fallback 冒充为 DB success。
  - unresolved projection failure / rollback manifest incomplete 时阻断 read-cut。
- Rollback rehearsal 必须只生成 dry-run recovery plan，不执行真实恢复；必须说明 would-use DB、would-use JSON projection、would-disable DB read-cut、would-preserve DB for audit。
- 成功时必须生成 read-cut rehearsal report，记录 DB path、projection root、manifest path、source root hash、DB read hash、projection hash、fallback decision、counts、redaction policy 和 recovery instructions。
- Failure injection 必须覆盖：
  - before DB read。
  - after DB read before projection verification。
  - projection hash mismatch。
  - missing rollback manifest。
  - incomplete rollback manifest。
  - corrupt DB path / schema mismatch。
  - after fallback selected before report commit。
- 失败不得写 completed read-cut report；不得把 fallback 状态写成 DB authoritative success。
- Rehearsal 重跑必须幂等：相同 fixture / same temp DB / same projection root / same manifest 的 report 文本稳定。
- Projection / read-cut report 不得包含 prompt body、full transcript、secret/token/credential/keychain/OAuth/provider credential/rollout body。
- 新增 R3-A4 fixtures，覆盖 read-cut success、fallback success、hash mismatch block、missing / incomplete manifest block、schema mismatch fallback、recovery dry-run 和 sensitive redaction。
- 新增 focused Rust tests。
- 写 R3-A4 evidence / handoff。

## 5. 允许读取

允许读取：

- `product-line` 内源码、文档、任务包、evidence、handoff、脚本、git 元数据。
- R3-P0 / R3-A1 / R3-A2 / R3-A3 的 evidence / handoff / supervisor checkpoint。
- 仓库内 `src-tauri/fixtures/r3-a1/**`、`src-tauri/fixtures/r3-a2/**`、`src-tauri/fixtures/r3-a3/**` 和新增 `src-tauri/fixtures/r3-a4/**`。

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
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_dual_write.rs`
- 可新增 `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_read_cut.rs`，仅用于 fixture-only rehearsal，必须低于 3,000 行。
- `prototypes/productized-desktop-shell/src-tauri/fixtures/r3-a4/**`
- `evidence/2026-06-11-root-treatment-r3-a4-fixture-only-read-cut-db-and-rollback-rehearsal-v1.md`
- `handoffs/2026-06-11-root-treatment-r3-a4-fixture-only-read-cut-db-and-rollback-rehearsal-v1-result.md`

默认不更新：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- Root Treatment 官方计划

入口同步由主管线 checkpoint 统一处理。

## 7. 禁止事项

R3-A4 禁止：

- 不创建生产 DB。
- 不写用户真实数据目录。
- 不迁移真实 `workflow-state.v0.json` 或 sidecar。
- 不修改任何真实 JSON / sidecar。
- 不切任何产品读写路径到 DB。
- 不让真实 app read model 读 DB。
- 不在 app startup / Tauri command / UI 中接入 read-cut rehearsal。
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
- 不把 fixture-only read-cut rehearsal 冒充为生产读切、生产双写期或 R3 完成。
- 不夹带 R4 前端按页读模型或 UI 瘦身。

## 8. 形状影响

- 任务类型：治理任务包 / SQLite read-cut rehearsal。
- 新增代码落点：建议 `workbench_sqlite_read_cut.rs`。
- 触碰棘轮文件：`src-tauri/src/lib.rs`，只允许新增 1 行 module declaration；不得增长业务逻辑。
- 新文件上限：
  - Rust 新文件必须低于 3,000 行。
  - 任何 fixture helper 必须低于 500 行。
- 是否新增 Tauri command：否。
- 是否新增 sidecar JSON 种类：否。
- 是否需要 shape gate 豁免：否。
- 本任务规划基线 commit：`6bb27ee` 或主管线派发时的最新 clean HEAD。
- 本任务完成 commit：待主管线回收后记录。

## 9. Fixture 矩阵

必须至少新增并测试：

- `read-cut-valid-core-chain`：DB apply + projection + completed manifest + DB authoritative read success。
- `read-cut-idempotent-rerun`：同一 fixture 重跑，read-cut report / hashes 稳定。
- `read-cut-db-unavailable-json-fallback`：DB unavailable，已验证 projection fallback，状态明确为 fallback。
- `read-cut-db-schema-mismatch-fallback`：DB schema mismatch / integrity failure，fallback 并标 degraded。
- `read-cut-projection-hash-mismatch-blocked`：projection hash mismatch，read-cut blocked，不显示 DB success。
- `read-cut-missing-manifest-blocked`：rollback manifest missing，read-cut blocked。
- `read-cut-incomplete-manifest-blocked`：incomplete manifest，read-cut blocked。
- `read-cut-sensitive-redaction`：report / projection 不含 forbidden sensitive body classes。
- `rollback-read-cut-recovery-dry-run`：生成 recovery dry-run plan，不执行真实恢复。

可以复用 R3-A3 fixtures 作为输入素材，但必须新增 R3-A4 fixture directories 或 manifest，明确每个测试覆盖点。

## 10. 验收标准

R3-A4 可接受为：

- Fixture-only read-cut DB rehearsal 已实现。
- 所有写入只发生在 temp / R3-A4 fixture path。
- DB authoritative read success 由 temp DB projection / export hash 验证，不依赖直接复制 source fixture。
- JSON fallback 状态与 DB success 状态清楚区分。
- DB unavailable / schema mismatch / projection hash mismatch / manifest missing / manifest incomplete 都有 focused tests。
- Rollback recovery 只生成 dry-run plan，不执行真实恢复。
- Rehearsal 重跑幂等。
- Report / projection 不输出 prompt body、full transcript、secret/token/credential/rollout body。
- 未创建生产 DB。
- 未迁移真实数据。
- 未新增 Tauri command。
- 未访问 `/Users/yoyi/.codex`。
- shape gate 通过。
- focused cargo tests、`cargo test --lib`、`cargo fmt -- --check`、`git diff --check` 通过，或如有环境失败则完整记录并不得冒充通过。
- evidence / handoff 记录 start commit、end commit 或未提交状态、写入范围、验证结果、P0/P1/P2 和禁止项确认。

R3-A4 不接受为：

- R3 SQLite 迁移开始或完成。
- 生产 DB 创建完成。
- 生产双写期开始。
- 生产读切 DB 完成。
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
cargo test --lib sqlite_read_cut
cargo test --lib workflow_state
cargo test --lib
cargo fmt -- --check
git diff --check
git status --short
```

若 filtered cargo tests no-match，必须记录 exact no-match，并用实际存在的 module/test name 或更广泛 `cargo test --lib` 覆盖，不能把 no-match 冒充通过。

必须额外扫描：

```bash
rg -n 'Command::new\("codex"\)|codex exec|codex exec resume|/Users/yoyi/\.codex|auth|token|provider credential|prompt_body|full transcript|rollout' prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_schema.rs prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_apply.rs prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_exporter.rs prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_dual_write.rs prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_read_cut.rs prototypes/productized-desktop-shell/src-tauri/fixtures/r3-a4
rg -n 'workflow-state|formal-memories|memory-candidates|observations|runtime-log|runtime-logs|plan-authorizations|project-proposals|real-execution-product-commands|session-continuations|read-cut|rollback-manifest' prototypes/productized-desktop-shell/src-tauri/fixtures/r3-a4 prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_read_cut.rs prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_exporter.rs prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_dual_write.rs
```

如果 optional 文件不存在，扫描命令可改为实际存在的 R3-A4 文件清单，但 evidence 必须说明。

## 12. 必须回传

开发线回传必须包含：

1. STATUS：`DONE` / `DONE_WITH_CONCERNS` / `NEEDS_DECISION` / `BLOCKED`。
2. CHANGED_FILES。
3. READ-CUT REHEARSAL SUMMARY。
4. JSON FALLBACK / DEGRADE SUMMARY。
5. ROLLBACK RECOVERY DRY-RUN SUMMARY。
6. FAILURE INJECTION SUMMARY。
7. FIXTURE COVERAGE MATRIX。
8. FORBIDDEN SENSITIVE FIELD HANDLING。
9. EVIDENCE / CHECKS RUN。
10. P0 / P1 / P2。
11. BOUNDARY CONFIRMATION。
12. REQUESTS。

## 13. 主管回收标准

主管线必须独立检查：

- changed files 是否只在 R3-A4 允许范围。
- `lib.rs` 是否只新增 module declaration。
- 新 Rust 文件是否低于 3,000 行。
- 是否新增 Tauri command、startup hook、UI、sidecar kind 或产品读写路径。
- read-cut success 是否来自 temp DB / export hash，而不是 source fixture copy。
- fallback 是否不冒充 DB success。
- rollback recovery 是否 dry-run only。
- sensitive / real-exec scan 是否只有 redaction policy / fixture naming / legal table names。
- fresh verification 是否由主管线重跑或有明确环境阻断记录。

主管线回收后才允许更新：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- Root Treatment 官方计划

## 14. Do Not Claim

完成 R3-A4 后仍不得声明：

- R3 SQLite 迁移开始或完成。
- 生产 DB 创建完成。
- 生产双写期开始。
- 生产读切 DB 完成。
- JSON / sidecar 停写。
- rollback production workflow 完成。
- 多 agent 并行真实执行解锁。
- Stage L / K3-B1 / K3-B2 已恢复。
