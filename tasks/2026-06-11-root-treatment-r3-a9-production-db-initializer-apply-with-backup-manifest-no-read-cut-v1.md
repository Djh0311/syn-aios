# Root Treatment / R3-A9 Production DB Initializer Apply With Backup Manifest No Read Cut v1

日期：2026-06-11

状态：Level A 已完成；Level B 未执行。本文是 Root Treatment / Stage R 的 R3-A9 任务包，用于在 R3-A6 production cutover / rollback operator contract、R3-A7 production preflight scanner / report、R3-A8 copied snapshot temp DB apply / export / rollback boundary 之后，进入首个受控 production DB initializer + apply 任务。R3-A9 Level A 已完成 fixture / temp production DB initializer + apply 合同验证；但仍不得切产品读路径到 DB，不得停写 JSON / sidecar，不得执行真实 Codex，不得读写 `/Users/yoyi/.codex`。

完成记录：

- evidence：`evidence/2026-06-11-root-treatment-r3-a9-production-db-initializer-apply-with-backup-manifest-no-read-cut-v1.md`
- handoff：`handoffs/2026-06-11-root-treatment-r3-a9-production-db-initializer-apply-with-backup-manifest-no-read-cut-v1-result.md`
- planning baseline commit：`b0d9b3b294af4d4883e4bdd16b2cf0c1f3f110d0`
- implementation commit：待主管线提交后由 checkpoint 记录

## 0. 全局主管理解

已知事实：

- R3-A7 只完成 scanner module + temp fixture validation；真实 production root scan 未执行。
- R3-A8 只完成 Level A fixture / temp copied snapshot rehearsal；Level B 真实 workbench state root copied snapshot 未执行。
- R3-A9 是第一个允许写工作台自有 production SQLite DB 的任务，但它不是 read-cut，不是 stop-write，不是 R3 完成。
- Stage L / L1-L6、K3-B1 retry、K3-B2 和 backlog 功能仍冻结为 `deferred_during_root_treatment`。

核心判断：

```text
R3-A9 可以创建并填充 production DB，但产品真实读写路径必须保持 JSON / sidecar 现状；DB 只是受控生产候选事实库，不是立即权威事实源。
```

## 1. Execution Mode

Execution Mode：Supervisor-led task package with reusable Stage R implementation line。

Multi-Agent Policy：

- 任务包由全局主管冻结和提交。
- 可派发给既有 Stage R 开发线，思考程度 high / xhigh。
- 开发线不得提交；主管线负责 fresh verify、复核、入口同步和 commit。
- 复核线可在实现回交后只读复核，不改文件。

Level split：

- Level A：fixture / temp production-apply contract implementation。必须先完成，用 repo fixture / temp root 验证 DB initializer、backup manifest、production apply report 和 rollback boundary，不读取真实 workbench state root。
- Level B：real workbench-owned production DB apply。只有 Level A 通过、任务包 evidence 追加 Level B execution record、主管自审通过后才执行。Level B 可读取工作台自有 state root 中允许的 JSON / sidecar 并创建生产 DB，但不得切读、不得停写、不得写 source JSON / sidecar。

Fallback If Scope Expands：

- 如果实现需要产品 read-cut、stop-write JSON、恢复写 production JSON / sidecar、Tauri command、startup hook、UI、真实 Codex 执行、`.codex`、secret / full transcript，立即停止并拆新任务包。

## 2. 权威依据

必须读取并服从：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `codex-multi-agent-safe-collaboration.md`
- `docs/plans/2026-06-10-root-treatment-official-development-plan-v1.md`
- `docs/plans/2026-06-11-root-treatment-r3-sqlite-schema-importer-rollback-contract-v1.md`
- `docs/plans/2026-06-11-root-treatment-r3-production-cutover-and-rollback-operator-contract-v1.md`
- `tasks/2026-06-11-root-treatment-r3-a7-production-preflight-scanner-and-report-v1.md`
- `tasks/2026-06-11-root-treatment-r3-a8-copied-production-snapshot-temp-db-apply-and-export-verification-v1.md`
- A7 / A8 evidence 和 handoff。

建议读取的代码：

- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_schema.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_importer.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_preflight.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_apply.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_exporter.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_snapshot_apply.rs`
- `scripts/harness/workbench-shape-gate.js`

## 3. 目标

R3-A9 Level A 必须完成：

- 新增 production DB apply helper，建议 `workbench_sqlite_production_apply.rs` 或等价名称。
- `lib.rs` 只允许新增 1 行 module declaration；不得新增 Tauri command、startup hook、UI 或产品路径调用。
- Helper 必须显式接收：
  - `source_state_root`
  - `production_db_path`
  - `backup_root`
  - `report_path`
  - `rollback_manifest_path`
  - `allowed_sidecars`
  - `denied_path_markers`
  - `expected_source_root_hash`
  - `expected_preflight_report_hash`
  - `expected_copied_snapshot_report_hash`
- Helper 必须先 production preflight，再创建 backup manifest，再初始化 DB，再 apply import，再 export dry-run verification，再写 rollback boundary / production apply report。
- Level A 的 source root 只能是 repo fixture 或 temp copied snapshot root；Level B 才允许真实 workbench state root。
- Helper 必须拒绝：
  - `production_db_path` 位于 `source_state_root` 内。
  - `backup_root` 位于 `source_state_root` 内。
  - `report_path` 位于 `source_state_root` 内。
  - `rollback_manifest_path` 位于 `source_state_root` 内。
  - 任何 path 命中 denied marker。
  - source snapshot 包含 `.env`、`.codex`、secret/token/credential/keychain/OAuth/provider credential/full transcript/rollout body。
  - preflight report hash / source root hash / copied snapshot report hash 不匹配。
  - 已存在 production DB 且无 manifest 证明同一 source hash / same schema / same import batch 可 idempotent rerun。
- Helper 必须输出 production apply report：
  - mode：`production_apply`
  - level：`level_a_fixture` 或 `level_b_workbench_owned_state`
  - status：`completed` / `blocked` / `failed_classified`
  - source root ref / hash / path hash
  - production DB path hash
  - backup root ref / hash
  - backup manifest hash
  - rollback manifest hash
  - preflight report hash
  - copied snapshot report hash
  - DB schema version
  - import batch id / hash
  - table counts / source record counts
  - DB -> JSON export verification summary
  - before / after source file hashes
  - safety flags
  - failure injection point if any
- Safety flags 必须为：
  - `production_db_created=true` only on successful DB initialization / apply.
  - `production_root_written=false`
  - `production_apply_performed=true` only after DB transaction commit.
  - `read_cut_enabled=false`
  - `stop_write_json=false`
  - `production_restore_performed=false`
  - `codex_home_touched=false`
  - `product_read_path_changed=false`
  - `source_json_written=false`
- DB -> JSON export verification 必须确认 canonical `runtime-logs.v1.json`，不能输出 legacy singular `runtime-log.v1.json`。
- Rollback boundary 必须是 dry-run plan：would-disable DB read-cut、would-preserve DB for audit、would-use source backup / last export projection、would-require supervisor decision、production_restore_performed=false。
- Failure injection 必须覆盖：
  - preflight blocked。
  - backup manifest write failure before DB create。
  - DB path inside source root rejected。
  - backup root / report / rollback path inside source root rejected。
  - DB initialize failure leaves no completed report。
  - import rejected / corrupt snapshot leaves no completed report。
  - transaction rollback before commit leaves no partial rows。
  - after DB commit before manifest commit writes failed_classified report and preserves DB for audit。
  - export hash mismatch blocks read-cut readiness。
  - rollback manifest missing / incomplete blocks completion。
  - idempotent rerun same source hash is deterministic or explicit already_applied classification。

R3-A9 Level B 如执行，必须完成：

- 先写 Level B execution record 到 A9 evidence。
- 明确 allowed source root，候选为工作台自有 state root；必须先用只读 existence / path classification 确认。
- 明确 production DB path，默认建议为 workbench-owned storage root 下 `workbench-state.v1.sqlite`；不得写 `/Users/yoyi/.codex`，不得写用户项目目录。
- 明确 backup root、backup manifest path、rollback manifest path、report path。
- 记录 before / after source JSON / sidecar file hashes，证明 source root 未被写。
- 记录 DB file hash、schema migration rows、table counts、import batch hash、export verification hash。
- Level B 仍不得切产品读路径、不得停写 JSON / sidecar、不得让 app startup 或 UI 使用 DB。

## 4. 允许读取

Level A 允许读取：

- `product-line` 内源码、任务包、docs、evidence、handoff、fixtures、git metadata。
- R3 fixtures 和测试创建的 temp roots。

Level B 可选允许读取：

- 任务包 evidence 中明确记录的 workbench state root 中工作台自有 `workflow-state.v0.json` 和 sibling allowed sidecars。
- 只读 metadata/hash/schema/revision/top-level counts 和导入 DB 所需的 allowed JSON / sidecar bytes。

禁止读取：

- `/Users/yoyi/.codex`
- secret、token、`.env`、keychain、OAuth、provider credential、完整 transcript、rollout。
- 用户真实项目源码内容。
- 任意未列入任务包 allowed roots 的路径。

## 5. 允许写入

允许写入：

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`：仅新增 module declaration。
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_production_apply.rs` 或等价 A9 helper。
- 可新增 `prototypes/productized-desktop-shell/src-tauri/fixtures/r3-a9/**`。
- 可写测试用 temp DB / temp backup / temp projection / temp report。
- Level B 如执行，可写任务包明确的 workbench-owned production DB path 和 backup / report / rollback manifest。
- `evidence/2026-06-11-root-treatment-r3-a9-production-db-initializer-apply-with-backup-manifest-no-read-cut-v1.md`
- `handoffs/2026-06-11-root-treatment-r3-a9-production-db-initializer-apply-with-backup-manifest-no-read-cut-v1-result.md`
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

R3-A9 禁止：

- 不切产品读路径到 DB。
- 不停写 JSON / sidecar。
- 不修改任何 source JSON / sidecar。
- 不让真实 app read model 读 DB。
- 不在 app startup / Tauri command / UI 中接入 production SQLite。
- 不新增 Tauri command。
- 不改 workflow state 顶层 schema。
- 不新增 sidecar store 或 sidecar JSON 种类。
- 不改真实 Codex runner。
- 不执行真实 `codex exec` / `codex exec resume`。
- 不发送 prompt。
- 不读写 `/Users/yoyi/.codex`。
- 不读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout。
- 不启动 Tauri / Browser / Chrome / Vite / 截图工具。
- 不启动 Stage L / K3-B1 retry / K3-B2。
- 不解冻 backlog 功能。
- 不把 production DB initializer / apply 冒充为 production read-cut、JSON / sidecar stop-write、rollback production workflow、R3 完成或多 agent 并行真实执行解锁。

## 7. 形状影响

- 任务类型：治理任务包 / production DB apply boundary。
- 新增代码落点：`workbench_sqlite_production_apply.rs` 或等价 A9 helper。
- 触碰棘轮文件：`src-tauri/src/lib.rs`，只允许新增 1 行 module declaration。
- 新文件上限：Rust 新文件必须低于 3,000 行。
- 是否新增 Tauri command：否。
- 是否新增 sidecar JSON 种类：否。
- 是否需要 shape gate 豁免：否。
- 本任务规划基线 commit：`b0d9b3b294af4d4883e4bdd16b2cf0c1f3f110d0`
- 本任务完成 commit：待主管线回收后记录。

## 8. 验收标准

R3-A9 可接受为：

- Level A production apply helper、backup manifest、rollback boundary、export verification 和 failure injection 已完成。
- 如执行 Level B，production DB 只写任务包明确 DB path，source JSON / sidecar 前后 hash 不变。
- `lib.rs` 只新增 module declaration。
- Shape gate 通过。
- Focused cargo tests、`cargo test --lib`、`cargo fmt -- --check`、`git diff --check` 通过。
- Evidence / handoff 明确 Level B 是否执行；默认不能冒充真实 production apply。

R3-A9 不接受为：

- production read-cut。
- JSON / sidecar stop-write。
- rollback production workflow。
- app startup / UI / read model 使用 DB。
- R3 完成。
- 多 agent 并行真实执行解锁。

## 9. 建议验证命令

必须跑：

```bash
node scripts/harness/workbench-shape-gate.js --mode check
cargo test --lib sqlite_production
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
rg -n "read_cut_enabled[:=] true|stop_write_json[:=] true|production_restore_performed[:=] true|codex_home_touched[:=] true|product_read_path_changed[:=] true|source_json_written[:=] true|production_root_written[:=] true" prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_production_apply.rs prototypes/productized-desktop-shell/src-tauri/fixtures/r3-a9
rg --hidden -n "/Users/yoyi/.codex|\\.env|token|secret|credential|keychain|oauth|provider credential|full transcript|rollout|prompt_body" prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_production_apply.rs prototypes/productized-desktop-shell/src-tauri/fixtures/r3-a9
```

命中必须分类：guard / denied marker / redaction / fixture label，不得存在真实 secret、真实 `.codex` 读取或 prompt body 持久化。Evidence / handoff 中的边界文案可另行人工分类，但不应纳入“无命中”机械扫描范围，避免扫描命令文本自命中。

## 10. Evidence / Handoff 结构

Evidence 必须包含：

1. STATUS。
2. READ / WRITE SCOPE。
3. LEVEL A SUMMARY。
4. LEVEL B STATUS。
5. PRODUCTION DB PATH AND BACKUP MANIFEST。
6. PRELIGHT / APPLY / EXPORT SUMMARY。
7. ROLLBACK BOUNDARY SUMMARY。
8. FAILURE INJECTION SUMMARY。
9. BEFORE / AFTER SOURCE HASHES。
10. CHECKS RUN。
11. P0 / P1 / P2。
12. BOUNDARY CONFIRMATION。
13. DO NOT CLAIM。

Handoff 必须包含：

- STATUS。
- changed files。
- Level A / Level B 结论。
- exact validation output summary。
- source root / DB path / backup root / report path 分类。
- 是否触碰真实 workbench state root。
- 是否写 production DB。
- 是否执行真实 Codex。
- P0/P1/P2。
- 下一步 R3-A10 前置条件。

## 11. Do Not Claim

完成 R3-A9 后仍不得声明：

- R3 SQLite 迁移完成。
- production read-cut 完成。
- JSON / sidecar 停写。
- rollback production workflow 完成。
- app read model 已使用 DB。
- 多 agent 并行真实执行解锁。
- Stage L / K3-B1 / K3-B2 已恢复。
