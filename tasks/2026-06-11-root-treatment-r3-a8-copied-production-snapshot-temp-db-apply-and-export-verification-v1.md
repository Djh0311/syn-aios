# Root Treatment / R3-A8 Copied Production Snapshot Temp DB Apply And Export Verification v1

日期：2026-06-11

状态：已完成。本文是 Root Treatment / Stage R 的 R3-A8 任务包，用于在 R3-A6 production cutover / rollback operator contract freeze 和 R3-A7 production preflight scanner / report 基础上，实现 copied production snapshot temp DB apply / export / rollback boundary 验证。R3-A8 已完成 Level A fixture / temp snapshot 能力；Level B 未执行。如要执行真实工作台 state root 的复制快照演练，必须另行记录 allowed source root、copy destination、report path、hash manifest、cleanup / rollback boundary 和 denied paths。

完成记录：

- evidence：`evidence/2026-06-11-root-treatment-r3-a8-copied-production-snapshot-temp-db-apply-and-export-verification-v1.md`
- handoff：`handoffs/2026-06-11-root-treatment-r3-a8-copied-production-snapshot-temp-db-apply-and-export-verification-v1-result.md`
- planning baseline commit：`7ef100767b63369bfb62dd87c08b59db6f58f7ca`
- implementation commit：`ce631c1cd23dadb367288885d61a331b88b83511`

## 0. 全局主管理解

已知事实：

- R3-A6 已冻结 R3 production mode contract：`production_preflight` 不建 DB，`copied_snapshot_apply` 只能读复制快照、写临时 DB，不得写 production root。
- R3-A7 已实现 scanner module + temp fixture validation，并经复核线修补 2 个 P1；真实 production root scan 未执行。
- R3-A8 是进入生产前置演练的下一步，但仍不是生产迁移，不解锁 read-cut / stop-write / 多 agent 并行真实执行。
- Stage L / L1-L6、K3-B1 retry、K3-B2 和 backlog 功能仍冻结为 `deferred_during_root_treatment`。

本任务核心判断：

```text
R3-A8 只能证明“复制快照 + 临时 DB + export / rollback boundary”可用；不能写真实 production root，不能创建生产 DB，不能切产品读写路径。
```

## 1. Execution Mode

Execution Mode：Supervisor-led task package with reusable Stage R implementation line。

Multi-Agent Policy：

- 任务包由全局主管冻结和提交。
- 实现可派发给既有 Stage R 开发线，思考程度 high / xhigh。
- 开发线不得提交；主管线负责 fresh verify、复核、入口同步和 commit。
- 复核线可在实现回交后只读复核，不改文件。

Level split：

- Level A：fixture / temp snapshot implementation and tests。必须完成，默认不读取真实 production root。
- Level B：optional copied real workbench state root rehearsal。只有主管线在 evidence 中写清 allowed source root、copy destination、report path、hash manifest 和 cleanup boundary 后才能执行；不得写 source root。

Fallback If Scope Expands：

- 如果实现需要 production DB path、production root write、read-cut、stop-write、Tauri command、startup hook、UI、真实 Codex 执行、`.codex`、secret / transcript，立即停止并拆新任务包。

## 2. 权威依据

必须读取并服从：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `codex-multi-agent-safe-collaboration.md`
- `docs/plans/2026-06-10-root-treatment-official-development-plan-v1.md`
- `docs/plans/2026-06-11-root-treatment-r3-production-cutover-and-rollback-operator-contract-v1.md`
- `tasks/2026-06-11-root-treatment-r3-a7-production-preflight-scanner-and-report-v1.md`
- `evidence/2026-06-11-root-treatment-r3-a7-production-preflight-scanner-and-report-v1.md`
- `evidence/2026-06-11-root-treatment-r3-a7-supervisor-checkpoint-v1.md`

建议读取的代码：

- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_preflight.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_importer.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_schema.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_apply.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_exporter.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_observation_period.rs`
- `scripts/harness/workbench-shape-gate.js`

## 3. 目标

R3-A8 Level A 必须完成：

- 新增 copied snapshot rehearsal helper，建议 `workbench_sqlite_snapshot_apply.rs` 或等价名称。
- `lib.rs` 只允许新增 1 行 module declaration；不得新增 Tauri command、startup hook、UI 或产品路径调用。
- Helper 必须显式接收：
  - `source_snapshot_root`
  - `snapshot_copy_root`
  - `temp_db_path`
  - `temp_export_root`
  - `report_path`
  - `allowed_sidecars`
  - `denied_path_markers`
  - `expected_source_root_hash` / optional preflight report hash
- Helper 必须先复制 source snapshot 到 temp copy root，再只对 temp copy root 运行 apply / export / rollback boundary。
- Helper 必须拒绝：
  - `snapshot_copy_root` 位于 `source_snapshot_root` 内。
  - `report_path` 位于 `source_snapshot_root` 内。
  - `temp_db_path` 位于 `source_snapshot_root` 内。
  - `temp_export_root` 位于 `source_snapshot_root` 内。
  - source / copy / report / DB / export 路径命中 denied markers。
  - source snapshot 包含 `.env`、`.codex`、secret/token/credential/keychain/OAuth/provider credential/full transcript/rollout body。
- Helper 必须输出 copied snapshot apply report：
  - mode：`copied_snapshot_apply`
  - level：`level_a_fixture` 或 `level_b_copied_real_state`
  - source root ref / hash
  - snapshot copy root ref / hash
  - temp DB path hash
  - temp export root hash
  - report path hash
  - copied file manifest：path ref、path hash、file hash、size、schema/revision metadata
  - preflight status consumed / reproduced
  - apply importer report summary
  - DB -> JSON export verification summary
  - rollback dry-run boundary summary
  - production_db_created=false
  - production_root_written=false
  - production_apply_performed=false
  - read_cut_enabled=false
  - stop_write_json=false
  - production_restore_performed=false
  - codex_home_touched=false
- Export verification 必须确认 canonical `runtime-logs.v1.json`，不能输出 legacy singular `runtime-log.v1.json`。
- Rollback boundary 必须是 dry-run plan：would-disable DB read-cut、would-use snapshot / export projection、would-preserve temp DB for audit、would-require supervisor decision、production_restore_performed=false。
- Failure injection 必须覆盖：
  - source preflight blocked。
  - copy destination inside source root。
  - report path inside source root。
  - temp DB path inside source root。
  - copy interrupted before manifest。
  - apply importer rejected / corrupt snapshot。
  - export hash mismatch。
  - rollback manifest missing / incomplete。
  - cleanup failure leaves only temp artifacts and never source changes。
- Idempotent rerun：same snapshot + same temp roots should produce deterministic report hash or explicit duplicate / already completed classification;不得覆盖损坏 report。

R3-A8 Level B 如执行，必须完成：

- 先写 Level B execution record 到 A8 evidence。
- 明确 allowed source root，默认候选为工作台自有 state root：`/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state`，但必须在执行前用存在性检查确认。
- source root 只允许读取工作台自有 JSON / sidecar 文件用于复制、metadata、hash、schema/revision；不得读取 secret、`.codex`、full transcript 或 provider credential。
- copy destination 必须在 `/private/tmp`、`/private/var/.../T` 或 `product-line/tmp/root-treatment-r3-a8/**` 等临时 / 工作区路径内。
- report 必须写入 `evidence/r3-a8-copied-snapshot/**` 或 A8 evidence 指定路径；不得写 source root。
- Level B 仍不创建 production DB、不写 production root、不生产 read-cut、不停写 JSON / sidecar。

## 4. 允许读取

Level A 允许读取：

- `product-line` 内源码、任务包、docs、evidence、handoff、fixtures、git metadata。
- R3 fixtures 和测试创建的 temp roots。

Level B 可选允许读取：

- 显式记录的 workbench state root 中工作台自有 `workflow-state.v0.json` 和 sibling allowed sidecars。
- 只读 metadata/hash/schema/revision/top-level counts 和复制到 temp snapshot 所需的文件 bytes；不得输出 forbidden body。

禁止读取：

- `/Users/yoyi/.codex`
- secret、token、`.env`、keychain、OAuth、provider credential、完整 transcript、rollout。
- 用户真实项目源码内容。
- 任意未列入任务包 allowed roots 的路径。

## 5. 允许写入

允许写入：

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`：仅新增 module declaration。
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_snapshot_apply.rs` 或等价 A8 helper。
- 可新增 `prototypes/productized-desktop-shell/src-tauri/fixtures/r3-a8/**`。
- 可写测试用 temp DB / temp projection / temp report。
- 可写 `product-line/tmp/root-treatment-r3-a8/**`，仅用于 Level B copied snapshot / temp DB / temp export / report staging。
- `evidence/2026-06-11-root-treatment-r3-a8-copied-production-snapshot-temp-db-apply-and-export-verification-v1.md`
- `handoffs/2026-06-11-root-treatment-r3-a8-copied-production-snapshot-temp-db-apply-and-export-verification-v1-result.md`
- 本任务包状态。

默认不更新：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- Root Treatment 官方计划

入口同步由主管线 checkpoint 统一处理。

## 6. 禁止事项

R3-A8 禁止：

- 不创建 production DB。
- 不写 production root。
- 不把 temp DB 放入 production root。
- 不把 report / export / copy 目标放入 production root。
- 不迁移真实 `workflow-state.v0.json` 或 sidecar。
- 不修改任何真实 JSON / sidecar。
- 不切任何产品读写路径到 DB。
- 不让真实 app read model 读 DB。
- 不停止 JSON / sidecar 写入。
- 不把 JSON 降为 production fallback。
- 不在 app startup / Tauri command / UI 中接入 SQLite apply。
- 不改 workflow state 顶层 schema。
- 不新增 sidecar store 或 sidecar JSON 种类。
- 不新增 Tauri command。
- 不改真实 Codex runner。
- 不执行真实 `codex exec` / `codex exec resume`。
- 不发送 prompt。
- 不读写 `/Users/yoyi/.codex`。
- 不读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout。
- 不启动 Tauri / Browser / Chrome / Vite / 截图工具。
- 不启动 Stage L / K3-B1 retry / K3-B2。
- 不解冻 backlog 功能。
- 不把 copied snapshot apply 冒充为 production apply、production DB 创建、read-cut、stop-write、R3 完成或多 agent 并行真实执行解锁。

## 7. 形状影响

- 任务类型：治理任务包 / copied production snapshot apply rehearsal。
- 新增代码落点：`workbench_sqlite_snapshot_apply.rs` 或等价 A8 helper。
- 触碰棘轮文件：`src-tauri/src/lib.rs`，只允许新增 1 行 module declaration。
- 新文件上限：Rust 新文件必须低于 3,000 行。
- 是否新增 Tauri command：否。
- 是否新增 sidecar JSON 种类：否。
- 是否需要 shape gate 豁免：否。
- 本任务规划基线 commit：`7ef100767b63369bfb62dd87c08b59db6f58f7ca`
- 本任务完成 commit：`ce631c1cd23dadb367288885d61a331b88b83511`

## 8. 验收标准

R3-A8 可接受为：

- copied snapshot helper 已实现。
- Level A fixture / temp snapshot apply、export、rollback boundary 验证完成。
- Helper 只写 temp copy / temp DB / temp export / report，不写 source root。
- `lib.rs` 只新增 module declaration。
- Shape gate 通过。
- Focused cargo tests、`cargo test --lib`、`cargo fmt -- --check`、`git diff --check` 通过。
- Evidence / handoff 明确 Level B 是否执行；默认应记录未执行真实 workbench state root copied snapshot。
- 若 Level B 执行，必须记录 exact source root、copy root、temp DB path、export root、report path、manifest hash、source hash、before / after source file hashes，并证明 source root 未写。

R3-A8 不接受为：

- production DB 创建。
- production apply。
- production read-cut。
- JSON / sidecar stop-write。
- rollback production workflow。
- R3 完成。
- 多 agent 并行真实执行解锁。

## 9. 建议验证命令

必须跑：

```bash
node scripts/harness/workbench-shape-gate.js --mode check
cargo test --lib sqlite_snapshot
cargo test --lib sqlite_preflight
cargo test --lib sqlite_apply
cargo test --lib sqlite_export
cargo test --lib sqlite_observation
cargo test --lib workflow_state
cargo test --lib
cargo fmt -- --check
git diff --check
git status --short
```

如果某个 filtered test no-match，必须记录 exact no-match，并用实际测试名或 `cargo test --lib` 覆盖，不能冒充通过。

必须跑扫描：

```bash
rg -n "production_db_created=true|production_root_written=true|production_apply_performed=true|read_cut_enabled=true|stop_write_json=true|production_restore_performed=true|codex_home_touched=true" prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_snapshot_apply.rs evidence/2026-06-11-root-treatment-r3-a8-copied-production-snapshot-temp-db-apply-and-export-verification-v1.md handoffs/2026-06-11-root-treatment-r3-a8-copied-production-snapshot-temp-db-apply-and-export-verification-v1-result.md
rg -n "/Users/yoyi/.codex|\\.env|token|secret|credential|keychain|oauth|provider credential|full transcript|rollout" prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_snapshot_apply.rs evidence/2026-06-11-root-treatment-r3-a8-copied-production-snapshot-temp-db-apply-and-export-verification-v1.md handoffs/2026-06-11-root-treatment-r3-a8-copied-production-snapshot-temp-db-apply-and-export-verification-v1-result.md
```

命中必须分类：guard / denied marker / redaction / forbidden scan text / fixture label，不得存在真实 secret 或真实 `.codex` 读取。

## 10. Evidence / Handoff 结构

Evidence 必须包含：

1. STATUS。
2. READ / WRITE SCOPE。
3. LEVEL A SUMMARY。
4. LEVEL B STATUS。
5. COPIED SNAPSHOT MANIFEST。
6. TEMP DB APPLY SUMMARY。
7. EXPORT VERIFICATION SUMMARY。
8. ROLLBACK BOUNDARY SUMMARY。
9. FAILURE INJECTION SUMMARY。
10. CHECKS RUN。
11. P0 / P1 / P2。
12. BOUNDARY CONFIRMATION。
13. DO NOT CLAIM。

Handoff 必须包含：

- STATUS。
- changed files。
- Level A / Level B 结论。
- exact validation output summary。
- source root / copy root / temp DB / export root / report path 分类。
- 是否触碰真实 workbench state root。
- 是否执行真实 Codex。
- P0/P1/P2。
- 下一步 R3-A9 前置条件。

## 11. Do Not Claim

完成 R3-A8 后仍不得声明：

- R3 SQLite 迁移开始或完成。
- 生产 DB 创建完成。
- production apply 已完成。
- 生产双写期开始。
- 生产读切 DB 完成。
- JSON / sidecar 停写。
- rollback production workflow 完成。
- 多 agent 并行真实执行解锁。
- Stage L / K3-B1 / K3-B2 已恢复。
