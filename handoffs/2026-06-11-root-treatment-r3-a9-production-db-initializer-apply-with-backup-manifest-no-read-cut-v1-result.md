# Root Treatment / R3-A9 Production DB Initializer Apply With Backup Manifest No Read Cut Handoff v1

日期：2026-06-11

STATUS：DONE

## Changed Files

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_production_apply.rs`
- `prototypes/productized-desktop-shell/src-tauri/fixtures/r3-a9/**`
- `evidence/2026-06-11-root-treatment-r3-a9-production-db-initializer-apply-with-backup-manifest-no-read-cut-v1.md`
- `handoffs/2026-06-11-root-treatment-r3-a9-production-db-initializer-apply-with-backup-manifest-no-read-cut-v1-result.md`

## Summary

完成 R3-A9 Level A fixture / temp production DB initializer apply rehearsal。新增 `workbench_sqlite_production_apply.rs`，实现 preflight -> backup manifest -> temp production DB initialize/apply -> export dry-run verification -> rollback boundary manifest -> completed report 的合同链路。`lib.rs` 只新增一行 module declaration。

Level B 未执行；未读取真实 workbench state root，未创建真实 workbench-owned production DB。

## Production DB Path And Backup Manifest Summary

Level A 只接受 repo fixture 或 temp source root，且拒绝 DB / backup / report / rollback path 落在 source root 内。backup manifest 在 DB create 前写入，记录 source file hashes、preflight hash、source root hash、copied snapshot report hash 和 schema metadata。backup manifest write failure injection 验证 DB 不会创建。

## Preflight / Apply / Export Summary

Preflight 复用 R3-A7 scanner，并强制合并默认 denied markers。Apply 复用 temp DB apply importer。Export verification 复用 DB -> JSON dry-run exporter，要求 canonical `runtime-logs.v1.json`，拒绝 legacy `runtime-log.v1.json`。

成功 safety flags：

- `production_db_created=true`
- `production_apply_performed=true`
- `production_root_written=false`
- `read_cut_enabled=false`
- `stop_write_json=false`
- `production_restore_performed=false`
- `codex_home_touched=false`
- `product_read_path_changed=false`
- `source_json_written=false`

## Rollback Boundary Summary

rollback manifest 只表达 dry-run plan：would-disable DB read-cut、would-preserve DB for audit、would-use source backup / last export projection、would-require supervisor decision，并固定 `production_restore_performed=false`。missing / incomplete rollback manifest 会阻断 completed report。

## Failure Injection Summary

已覆盖 preflight blocked、backup manifest failure before DB create、DB / backup / report / rollback inside source root、DB initialize failure、corrupt snapshot apply failure、transaction rollback before commit、after DB commit before manifest commit、export hash mismatch、rollback manifest missing / incomplete、idempotent rerun、expected hash mismatch。

## Before / After Source Hashes

helper 记录 before / after source file hashes；若 source root 文件发生变化会阻断 completed report。Level A success 路径证明 source hashes 稳定。

## Checks Run

- `node scripts/harness/workbench-shape-gate.js --mode check`：PASS。
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
- forbidden true-flag scan：PASS，no forbidden true flag matches。
- sensitive scan：expected matches only in denied marker list / blocked `.env` fixture path.

主管线 fresh verify 复跑结果一致：

- shape gate：PASS，0 errors / 0 warnings。
- `sqlite_production` / `sqlite_snapshot` / `sqlite_preflight` / `sqlite_apply` / `sqlite_export` / `sqlite_observation` / `workflow_state` filtered tests：全部通过。
- `cargo test --lib`：424 passed / 16 ignored。
- `cargo fmt -- --check`、`git diff --check`：通过。
- 禁止 true-flag scan：无命中。
- 敏感 marker scan：仅 helper deny markers、`.env` blocked fixture path 和边界文案命中。

提交前 P2 tightening 已完成：

- backup manifest write failure injection 已补强为 backup root 准备后、DB 创建前失败，并断言正式 backup manifest 不存在。
- 任务包 / evidence 的 forbidden true-flag 扫描说明已改为避免机械扫描自命中。
- 修补后复跑 `cargo test --lib sqlite_production`、`cargo fmt -- --check`、`git diff --check` 和更新后的 forbidden true-flag / sensitive marker scans，均通过或仅命中 helper deny markers。

## P0 / P1 / P2

P0：无。

P1：无。

P2：无；建议主管线 fresh verify 后提交，并在 checkpoint 里明确 Level B 未执行。

## Boundary Confirmation

未提交 git；未同步入口文档；未执行 Level B；未读取真实 workbench state root；未创建真实 workbench-owned production DB；未修改 source JSON / sidecar；未切产品读路径到 DB；未停写 JSON / sidecar；未新增 Tauri command / UI / app startup hook；未执行真实 Codex；未读写 `/Users/yoyi/.codex`；未读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout/prompt body；未启动 Tauri / Browser / Chrome / Vite / screenshot。

## Requests

请求主管线执行 fresh verify、只读复核、checkpoint 记录，并决定是否进入单独授权的 R3-A9 Level B 或后续任务包。当前结果不能声明 production read-cut、JSON stop-write、真实 production rollback workflow 或 R3 完成。
