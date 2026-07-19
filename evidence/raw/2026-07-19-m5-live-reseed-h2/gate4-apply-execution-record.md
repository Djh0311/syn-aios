# Gate 4 执行事实记录（2026-07-20 00:50 +0800）

## 执行尝试 1 / 2（均未构成 apply）

- 两次在 `~` 下执行、未带 `cd`，cargo `could not find Cargo.toml` 即退出；另有一次 zsh parse error（shell 层，未运行程序）。
- 每次执行线均当场复核：production-db 空、apply-backup/report/rollback 未生成。**零写入，不计 apply 次数。**

## 执行尝试 3（用户亲手，单行命令，apply 本体，仅一次）

- 用户终端贴回：`test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1067 filtered out; finished in 15.76s`。
- report 权威核验（`production-apply-report.json`）：
  - `status=completed`、`level=level_b_workbench_owned_state`、`failure_point=null`
  - `source_root_hash=c3038dc407fa9decf1323fed21909b6a72beb50f13aaf3dd30524c31326540f2`（与 Gate 3 现场值一致）
  - 安全旗九项全绿：`production_apply_performed=true`、`production_db_created=true`、`source_json_written=false`、`production_root_written=false`、`codex_home_touched=false`、`read_cut_enabled=false`、`stop_write_json=false`、`production_restore_performed=false`、`product_read_path_changed=false`
  - `before_source_hashes == after_source_hashes`（11 源文件）
  - `export_verification.status=verified`，db_export_hash=`73e103eb…3a50`
  - rollback boundary = dry_run_only，restore 未执行
  - source_records 4142 vs 源计数和 4148（差 6，与 07-16「3815 vs 3821 差 6」同型 = 导入层口径常态）
- 产物 hash 冻结：
  - report `a3949363fe55248afa3ab84850c5113861e575222fecf1aae682145077f3e9ad`
  - rollback-manifest `aa71640bb6bdc90acf3a23f9fc71e24ee8ecedeb593124f8d025c78bc78b3067`
  - apply-backup 三 manifest：`f7c531fb…`（backup）/ `8b1d0e5a…`（apply）/ `d4dcbdc8…`（export）
- 新生产 DB：`workbench-state.v1.sqlite` size 32157696，mtime 2026-07-20 00:49:58，SHA-256 `e302a0fb3e7822d7411ba5a0817a4e896d32f9ea93750f651b3a954d14cb0b43`（apply 时刻快照）。
- 源 JSON 只读复核（apply 后 hash 与 Gate 0 冻结逐一相等）：workflow-state `7d1153f3…` ✓ / project-proposals `3d7d965e…` ✓ / supervisor-orchestrator `699043d7…` ✓ / storage-mode `b35188a1…` ✓。

## Gate 5 静态对账（App 未启动，sqlite3 -readonly）

DB 表计数 vs Gate 0 JSON 冻结，逐项相等：

| 面 | DB | JSON | 结果 |
|---|---|---|---|
| workflows | 8 | 8 | ✓ |
| workflow_nodes | 66 | 66 | ✓ |
| workflow_edges | 50 | 50 | ✓ |
| work_items | 58 | 58 | ✓ |
| workflow_audit_events | 1771 | 1771 | ✓ |
| workflow_chain_runs | 40 | 40 | ✓ |
| execution_attempts | 164 | 164 | ✓ |
| workflow_node_dispatches | 404 | 404 | ✓ |
| workflow_artifacts | 27 | 27 | ✓ |
| projects | 5 | 5 | ✓ |
| workflow_reviews | 11 | 11 | ✓ |
| permission_requests | 1 | 1 | ✓ |
| workflow_execution_controls | 164 | 164 | ✓ |
| workflow_node_session_bindings | 76 | 76 | ✓ |
| project_proposals | 74（pending 17 / confirmed 56 / rejected 1） | 同 | ✓ |
| plan_authorizations | 56 | 56 | ✓ |
| supervisor_orchestrator_sessions | 25 | 25 | ✓ |

**apply 闸 + 静态对账全绿。等总指导核后，用户打开 Gate 1 新二进制做启动对账。**
