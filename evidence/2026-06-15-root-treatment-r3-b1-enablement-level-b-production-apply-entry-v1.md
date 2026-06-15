# Root Treatment / R3 B1 Enablement Level-B Production Apply Entry v1 Evidence

日期：2026-06-15

状态：已完成，独立复核 `STATUS: CLEAR`。

Planning baseline：`1356378 docs: close C1 in work-schedule (research report verified saved)`

收口前 HEAD：`f50848b docs: durably track round-2 research report in docs/research`

Task package：`tasks/2026-06-15-root-treatment-r3-b1-enablement-level-b-production-apply-entry-v1.md`

## 1. 本包目标

本包只做 B1 enablement，不做 B1 apply：新增显式、窄口径的 Level-B production apply 入口，让后续 B1 可以在用户逐点确认路径后受控执行。

本包不读取真实 `WORKBENCH_STATE_ROOT`，不创建真实 production DB，不执行真实数据 apply，不切 read path，不停写 JSON / sidecar。

## 2. 完成内容

### 2.1 保留 Level-A 旧门

旧入口仍保留：

- `initialize_temp_workbench_sqlite_db`
- `apply_fixture_dir_to_temp_db`
- `export_temp_db_to_json_dry_run`
- `rehearse_production_db_apply_level_a`
- `validate_level_a_source_root`

旧入口仍拒绝非 temp / R3 fixture source root 或 DB path。新增测试 `sqlite_production_level_a_still_rejects_non_temp_output_path` 覆盖该事实。

### 2.2 新增显式 Level-B confirmed path 入口

新增：

- `SqliteProductionApplyLevelBConfig`
- `rehearse_production_db_apply_level_b_workbench_owned_state`
- `validate_level_b_source_root`
- `validate_level_b_output_paths`
- `initialize_confirmed_workbench_sqlite_db`
- `apply_confirmed_workbench_state_root_to_confirmed_db`
- `export_confirmed_db_to_json_dry_run`

Level-B 入口要求调用方传入的 source root / DB / backup / report / rollback 路径必须逐项等于 config 中 confirmed path；任何不一致立即拒绝。

### 2.3 Guard 和 invariant

Level-B 入口保留：

- denied marker guard：`.codex`、`.env`、secret、token、credential、keychain、OAuth、provider credential、full transcript、rollout、prompt body 等。
- 输出路径不得位于 source root 内。
- source root 和输出路径要求 canonical / clean path；拒绝 `.` / `..`，拒绝 canonical 后文本不一致的路径别名，拒绝 canonical 后落入 source root 内的输出路径。
- source before / after hashes 必须一致。
- safety flags 保持 `read_cut_enabled=false`、`stop_write_json=false`、`production_restore_performed=false`、`codex_home_touched=false`、`product_read_path_changed=false`、`source_json_written=false`。
- corrupt snapshot failure injection 走传入的 `apply_source_to_db`，不再在 Level-B 故障注入里硬编码 Level-A helper。

### 2.4 Sparse sidecar 语义

Level-B 允许缺失 optional sidecar：缺失 optional sidecar 不失败，符合 B0 真实状态“主状态已有积累、sidecar 很薄”的事实。

保留 legacy runtime log alias 禁止导出；Level-A full fixture 仍要求 canonical `runtime-logs.v1.json` projection，Level-B sparse source 不再要求缺失 optional runtime log 必须投影。

## 3. 修改范围

代码文件：

- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_schema.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_apply.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_exporter.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_production_apply.rs`

任务包：

- `tasks/2026-06-15-root-treatment-r3-b1-enablement-level-b-production-apply-entry-v1.md`

未修改：

- 未修改 `lib.rs`。
- 未新增 Tauri command。
- 未修改 app startup / UI / CSS / 产品全局读写路径。
- 未新增 sidecar JSON kind。
- 未修改真实 workbench state root。
- 未读取或写入 `/Users/yoyi/.codex`。

## 4. 测试覆盖

新增 / 覆盖的关键测试：

- `sqlite_production_level_b_accepts_confirmed_root_and_paths`
- `sqlite_production_level_b_missing_optional_sidecars_warns_not_fails`
- `sqlite_production_level_b_rejects_unconfirmed_source_root`
- `sqlite_production_level_b_rejects_unconfirmed_output_paths`
- `sqlite_production_level_b_rejects_denied_output_path_marker`
- `sqlite_production_level_b_rejects_output_inside_source_root`
- `sqlite_production_level_b_rejects_parent_dir_output_escape_into_source_root`
- `sqlite_production_level_a_still_rejects_non_temp_output_path`

这些测试只使用 fixture / temp copied source，不读取真实 `WORKBENCH_STATE_ROOT`。

## 5. 验证记录

### 5.1 `cargo fmt -- --check`

结果：

```text
exit code 0
```

### 5.2 `cargo test --lib sqlite_production -- --nocapture`

执行目录：`prototypes/productized-desktop-shell/src-tauri`

```text
running 29 tests
...
test workbench_sqlite_production_apply::tests::sqlite_production_level_b_accepts_confirmed_root_and_paths ... ok
test workbench_sqlite_production_apply::tests::sqlite_production_level_b_missing_optional_sidecars_warns_not_fails ... ok
test workbench_sqlite_production_apply::tests::sqlite_production_level_b_rejects_parent_dir_output_escape_into_source_root ... ok
...
test result: ok. 29 passed; 0 failed; 0 ignored; 0 measured; 477 filtered out; finished in 0.93s
```

结果：通过。

### 5.3 `cargo test --lib sqlite_apply -- --nocapture`

```text
running 6 tests
test workbench_sqlite_apply::tests::sqlite_apply_importer_rejects_non_temp_db_path ... ok
test workbench_sqlite_apply::tests::sqlite_apply_importer_rejects_before_db_begin_without_creating_db ... ok
test workbench_sqlite_apply::tests::sqlite_apply_importer_rejects_conflicts_sensitive_and_corrupt_without_partial_rows ... ok
test workbench_sqlite_apply::tests::sqlite_apply_importer_after_commit_injection_keeps_committed_rows ... ok
test workbench_sqlite_apply::tests::sqlite_apply_importer_applies_valid_chain_and_reapply_is_idempotent ... ok
test workbench_sqlite_apply::tests::sqlite_apply_importer_rolls_back_failure_injection_before_commit ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 500 filtered out; finished in 0.09s
```

结果：通过。

### 5.4 `cargo test --lib sqlite_export -- --nocapture`

```text
running 3 tests
test workbench_sqlite_exporter::tests::sqlite_export_dry_run_uses_canonical_runtime_log_alias_policy ... ok
test workbench_sqlite_exporter::tests::sqlite_export_dry_run_projects_workflow_runtime_and_product_command_without_writes ... ok
test workbench_sqlite_exporter::tests::sqlite_export_dry_run_omits_forbidden_sensitive_fields ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 503 filtered out; finished in 0.03s
```

结果：通过。

### 5.5 `cargo test --lib`

```text
running 506 tests
...
test result: ok. 490 passed; 0 failed; 16 ignored; 0 measured; 0 filtered out; finished in 6.92s
```

结果：通过，仅保留既有 ignored real-execution tests 和既有 `invalid_params` dead-code warning。

### 5.6 Shape gate

命令：

```bash
node scripts/harness/workbench-shape-gate.js --mode check
```

输出摘要：

```text
Status: pass
Errors: 0
Warnings: 0
Git HEAD: f50848b3f8fe24fda8cb74e430f97d7e6e3523ac
Tauri commands: 97 total; 0 in lib.rs
Sidecar JSON kinds: 14 detected; 0 unknown
Converged-helper dups outside utils/: 0 (12 deferred-whitelisted)
```

结果：通过。

### 5.7 `git diff --check`

输出为空。

结果：通过。

## 6. 当前 git 实物

### 6.1 `git status --short`

```text
 M prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_apply.rs
 M prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_exporter.rs
 M prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_production_apply.rs
 M prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_schema.rs
?? tasks/2026-06-15-root-treatment-r3-b1-enablement-level-b-production-apply-entry-v1.md
?? evidence/2026-06-15-root-treatment-r3-b1-enablement-level-b-production-apply-entry-v1.md
?? handoffs/2026-06-15-root-treatment-r3-b1-enablement-level-b-production-apply-entry-v1-result.md
```

### 6.2 `git diff --stat`

```text
 .../src-tauri/src/workbench_sqlite_apply.rs        |  54 ++-
 .../src-tauri/src/workbench_sqlite_exporter.rs     |  28 ++
 .../src/workbench_sqlite_production_apply.rs       | 635 ++++++++++++++++++++-
 .../src-tauri/src/workbench_sqlite_schema.rs       |  27 +-
 4 files changed, 720 insertions(+), 24 deletions(-)
```

注：`git diff --stat` 不显示未跟踪任务包文件。

## 7. 独立复核

复核线：Arendt (`019ec9db-8def-73d0-85a8-7cd8ff0ee74b`)

初轮复核：`STATUS: CLEAR_WITH_P2`。P2 为：

- Level-B outside-source guard 使用 lexical `Path::starts_with`，未覆盖 `..` / symlink 语义路径场景。
- Level-B corrupt snapshot failure injection 共享路径里仍调用 Level-A `apply_fixture_dir_to_temp_db`。

主管线已修补：

- Level-B output path 现在拒绝 `.` / `..`，要求 canonical 文本路径，并用 canonical source root 做 inside-source 判断。
- corrupt snapshot failure injection 改为走传入的 `apply_source_to_db`。

补复核：`STATUS: CLEAR`。复核线确认旧 P2 已收敛，无 P0/P1。

## 8. 边界确认

本包没有：

- 执行 B1 apply。
- 读取真实 `WORKBENCH_STATE_ROOT`。
- 创建真实 production DB。
- 切 read path。
- 停写 JSON / sidecar。
- 修改 app startup / Tauri command / UI / 产品全局读写路径。
- 执行真实 `codex exec` / `codex exec resume`。
- 读取或写入 `/Users/yoyi/.codex`。
- 解锁多 agent 并行真实执行。

## 9. 不接受为

本包不接受为 B1 apply 已执行、真实 DB 已建、真实数据已迁、Level-A 被放宽、read-cut、stop-write、R3 完成、多 agent 解锁、真实 Codex 执行或 `.codex` 接触。
