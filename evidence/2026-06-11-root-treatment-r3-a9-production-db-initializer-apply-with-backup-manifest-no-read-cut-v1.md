# Root Treatment / R3-A9 Production DB Initializer Apply With Backup Manifest No Read Cut Evidence v1

日期：2026-06-11

STATUS：DONE

## Scope

Start commit：`6a7211adeeb16d21b29330c85e5d117ebc5d209c`

End commit：未提交；开发线按任务包要求不执行 `git add` / `git commit`，由主管线回收。

本轮只执行 R3-A9 Level A：fixture / temp production DB initializer + apply with backup manifest + export verification + rollback boundary。未执行 Level B，未读取真实 workbench state root，未创建真实 workbench-owned production DB。

## Changed Files

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
  - 仅新增 `mod workbench_sqlite_production_apply;`
  - 未新增 Tauri command、startup hook、UI 或产品路径调用。
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_production_apply.rs`
  - 新增 A9 Level A helper，当前 1345 行，低于 3000 行上限。
- `prototypes/productized-desktop-shell/src-tauri/fixtures/r3-a9/**`
  - 新增 Level A fixture matrix。
- `evidence/2026-06-11-root-treatment-r3-a9-production-db-initializer-apply-with-backup-manifest-no-read-cut-v1.md`
- `handoffs/2026-06-11-root-treatment-r3-a9-production-db-initializer-apply-with-backup-manifest-no-read-cut-v1-result.md`

## Level A Summary

新增 `rehearse_production_db_apply_level_a(...)`，显式接收：

- `source_state_root`
- `production_db_path`
- `backup_root`
- `report_path`
- `rollback_manifest_path`
- `SqliteProductionApplyConfig.allowed_sidecars`
- `SqliteProductionApplyConfig.denied_path_markers`
- `SqliteProductionApplyConfig.expected_source_root_hash`
- `SqliteProductionApplyConfig.expected_preflight_report_hash`
- `SqliteProductionApplyConfig.expected_copied_snapshot_report_hash`

执行顺序：

1. 校验 Level A source root 只能来自 repo fixture 或 temp root。
2. 拒绝 DB / backup / report / rollback path 位于 source root 内。
3. 合并默认 denied markers，拒绝 `.codex`、`.env`、secret/token/credential/keychain/OAuth/provider credential/full transcript/rollout/prompt body 等敏感路径或字段。
4. 调用 R3-A7 preflight scanner。
5. 校验 expected source / preflight / copied snapshot hash。
6. 写 backup manifest。
7. 初始化 temp production DB，并复用 apply importer contract 导入 fixture snapshot。
8. 执行 DB -> JSON export dry-run verification。
9. 校验 canonical `runtime-logs.v1.json`，拒绝 legacy singular `runtime-log.v1.json`。
10. 写 rollback dry-run boundary manifest。
11. 写 completed production apply report。

## Level B Status

Level B 未执行。未读取真实 workbench state root，未复制真实 production snapshot，未创建真实 workbench-owned production DB。

## Production DB Path And Backup Manifest Summary

Level A DB path 由调用方传入，测试只使用 temp path。helper 拒绝：

- `production_db_path` 位于 `source_state_root` 内。
- `backup_root` 位于 `source_state_root` 内。
- `report_path` 位于 `source_state_root` 内。
- `rollback_manifest_path` 位于 `source_state_root` 内。
- 任一路径命中 denied marker。

backup manifest 写在 temp `backup_root`，记录 source file hash、source root hash、preflight report hash、copied snapshot report hash、schema version、backup manifest hash。backup manifest write failure injection 发生在 DB create 前，测试证明不会留下 DB。

## Preflight / Apply / Export Summary

Preflight：

- 使用 R3-A7 `scan_workbench_state_root_preflight_with_config`。
- 自定义 config 不会移除默认 denied markers；默认 marker 会与任务传入 marker 合并。
- `.env` fixture 被 preflight blocked，用于验证敏感 marker 拦截。

Apply：

- 使用已有 temp DB apply importer。
- 成功报告 flags：
  - `production_db_created=true`
  - `production_apply_performed=true`
  - `production_root_written=false`
  - `read_cut_enabled=false`
  - `stop_write_json=false`
  - `production_restore_performed=false`
  - `codex_home_touched=false`
  - `product_read_path_changed=false`
  - `source_json_written=false`

Export：

- 使用 DB -> JSON export dry-run。
- 输出 export verification：`db_export_hash`、`projection_hash`、`export_manifest_hash`、projected files、record count、redaction status。
- canonical runtime log alias 策略：必须存在 `runtime-logs.v1.json`，不得输出 `runtime-log.v1.json`。

## Rollback Boundary Summary

Rollback manifest 是 dry-run boundary，不执行生产恢复：

- `would_disable_db_read_cut=true`
- `would_preserve_db_for_audit=true`
- `would_use_source_backup=true`
- `would_use_last_export_projection=true`
- `would_require_supervisor_decision=true`
- `production_restore_performed=false`

missing / incomplete rollback manifest 均阻断 completed report。

## Failure Injection Summary

覆盖任务包要求的 Level A failure matrix：

- `sqlite_production_preflight_blocked_creates_no_db_or_report`
- `sqlite_production_backup_manifest_failure_happens_before_db_create`
- `sqlite_production_rejects_db_backup_report_and_rollback_inside_source_root`
- `sqlite_production_db_initialize_failure_creates_no_completed_report`
- `sqlite_production_import_rejected_corrupt_snapshot_creates_no_completed_report`
- `sqlite_production_transaction_rollback_before_commit_leaves_no_partial_rows`
- `sqlite_production_after_commit_failure_writes_failed_classified_report`
- `sqlite_production_export_hash_mismatch_blocks_completed_report`
- `sqlite_production_missing_or_incomplete_rollback_manifest_blocks_completion`
- `sqlite_production_level_a_idempotent_rerun_is_deterministic`
- `sqlite_production_expected_hash_mismatches_block`

## Before / After Source Hashes

Helper 在成功路径记录 `before_source_hashes` 和 `after_source_hashes`。若前后 source file hashes 不一致，返回 `production_apply_blocked:source_hashes_changed`，不写 completed report。成功测试覆盖 report 中 before / after source hashes 稳定。

## Fixture Coverage Matrix

| Fixture | Purpose |
| --- | --- |
| `production-valid-core-chain` | valid workflow state + sidecars + canonical runtime logs，用于 success / export / rollback boundary |
| `production-idempotent-rerun` | deterministic rerun fixture |
| `production-preflight-blocked-denied-path` | contains `.env` marker；验证 preflight blocked，不读取 body |

## Checks Run

- `node scripts/harness/workbench-shape-gate.js --mode check`：PASS，0 errors / 0 warnings。
- `cargo test --lib sqlite_production`：PASS，12 passed。
- `cargo test --lib sqlite_snapshot`：PASS，13 passed。
- `cargo test --lib sqlite_preflight`：PASS，8 passed。
- `cargo test --lib sqlite_apply`：PASS，6 passed。
- `cargo test --lib sqlite_export`：PASS，3 passed。
- `cargo test --lib sqlite_observation`：PASS，15 passed。
- `cargo test --lib workflow_state`：PASS，11 passed。
- `cargo test --lib`：PASS，424 passed / 0 failed / 16 ignored。
- `cargo fmt -- --check`：PASS。
- `git diff --check`：PASS。
- forbidden true-flag scan before evidence/handoff：PASS，未发现禁止类 safety flag 被置为 true。
- sensitive marker scan before evidence/handoff：expected matches only in helper default denied marker list and `.env` blocked fixture path.

Known existing warning in cargo tests：

- `JsonRpcError::invalid_params` is never used.

## Supervisor Fresh Verify

主管线在开发线回交后重新运行验证：

- `node scripts/harness/workbench-shape-gate.js --mode check`：PASS，0 errors / 0 warnings。
- `cargo test --lib sqlite_production`：PASS，12 passed。
- `cargo test --lib sqlite_snapshot`：PASS，13 passed。
- `cargo test --lib sqlite_preflight`：PASS，8 passed。
- `cargo test --lib sqlite_apply`：PASS，6 passed。
- `cargo test --lib sqlite_export`：PASS，3 passed。
- `cargo test --lib sqlite_observation`：PASS，15 passed。
- `cargo test --lib workflow_state`：PASS，11 passed。
- `cargo test --lib`：PASS，424 passed / 0 failed / 16 ignored。
- `cargo fmt -- --check`：PASS。
- `git diff --check`：PASS。
- `git status --short`：仅包含 R3-A9 task / helper / fixtures / evidence / handoff 范围。
- forbidden true-flag scan：PASS，源码与 fixture 中未发现禁止类 safety flag 被置为 true。
- sensitive marker scan：命中只在 helper denied-marker 常量、`.env` blocked fixture path 和 evidence / handoff 边界说明中；未发现真实 secret / token / credential / full transcript / rollout body 读取或写入。

主管线确认：本轮仍是 R3-A9 Level A，Level B 未执行，不接受为 production read-cut、JSON / sidecar stop-write、真实 production rollback workflow、R3 完成或多 agent 并行真实执行解锁。

## Supervisor P2 Tightening

只读复核线收口过程中指出两个证据硬度问题，主管线已在提交前修补：

- backup manifest write failure injection 原先在 `reset_dir` / manifest write 前直接返回，现改为先准备 `backup_root` 并写入 `backup-manifest-write-failure.injected` marker，再在 DB 创建前返回；测试断言 marker 存在、DB 不存在、completed report 不存在、正式 backup manifest 不存在。
- evidence / task package 中不再写入会导致机械扫描自命中的 forbidden true-flag 字面量；任务包扫描范围改为 A9 源码与 fixture，evidence / handoff 边界文案另行人工分类。

修补后复跑：

- `cargo test --lib sqlite_production`：PASS，12 passed。
- `cargo fmt -- --check`：PASS。
- `git diff --check`：PASS。
- 更新后的 forbidden true-flag scan：PASS，无命中。
- 更新后的 sensitive marker scan：命中仅 helper denied-marker 常量。

## P0 / P1 / P2

P0：无。

P1：无。

P2：无；建议主管线 fresh verify 后提交，并在 checkpoint 中确认 Level B 仍未执行。

## Boundary Confirmation

- 未执行 Level B。
- 未读取真实 workbench state root。
- 未创建真实 workbench-owned production DB。
- 未修改任何 source JSON / sidecar。
- 未切产品读路径到 DB。
- 未停写 JSON / sidecar。
- 未让真实 app read model 读 DB。
- 未新增 Tauri command / UI / app startup hook。
- 未执行真实 `codex exec` / `codex exec resume`。
- 未发送 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout/prompt body。
- 未启动 Tauri / Browser / Chrome / Vite / screenshot。
- 未解冻 Stage L / K3-B1 / K3-B2 / backlog 功能。

## Cannot Claim

- 不能声明 production read-cut 已完成。
- 不能声明 JSON / sidecar stop-write 已完成。
- 不能声明真实 production rollback workflow 已执行。
- 不能声明真实 workbench state root 已扫描或导入。
- 不能声明 R3 完成。
