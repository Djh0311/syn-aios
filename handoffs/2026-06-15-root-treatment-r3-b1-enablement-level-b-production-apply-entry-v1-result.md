# Root Treatment / R3 B1 Enablement Level-B Production Apply Entry v1 Result

日期：2026-06-15

状态：已完成，独立复核 `STATUS: CLEAR`。

Task package：`tasks/2026-06-15-root-treatment-r3-b1-enablement-level-b-production-apply-entry-v1.md`

Evidence：`evidence/2026-06-15-root-treatment-r3-b1-enablement-level-b-production-apply-entry-v1.md`

复核线：Arendt (`019ec9db-8def-73d0-85a8-7cd8ff0ee74b`)

## 1. 完成内容

本包只完成 B1 enablement，不执行 B1 apply。

新增显式 Level-B production apply 入口：

- `SqliteProductionApplyLevelBConfig`
- `rehearse_production_db_apply_level_b_workbench_owned_state`
- confirmed source root / DB / backup / report / rollback path 校验
- Level-B canonical / clean path 校验：拒绝 `.` / `..`，拒绝 canonical 后文本不一致的路径别名，拒绝 canonical 后落入 source root 内的输出路径
- Level-B confirmed DB init / apply / export dry-run helper

Level-A 旧入口保持原安全语义：

- `initialize_temp_workbench_sqlite_db`
- `apply_fixture_dir_to_temp_db`
- `export_temp_db_to_json_dry_run`
- `rehearse_production_db_apply_level_a`

## 2. 修改文件

- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_schema.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_apply.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_exporter.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_production_apply.rs`
- `tasks/2026-06-15-root-treatment-r3-b1-enablement-level-b-production-apply-entry-v1.md`
- `evidence/2026-06-15-root-treatment-r3-b1-enablement-level-b-production-apply-entry-v1.md`
- `handoffs/2026-06-15-root-treatment-r3-b1-enablement-level-b-production-apply-entry-v1-result.md`

## 3. 验证

主管线通过：

- `cargo fmt -- --check`
- `cargo test --lib sqlite_production -- --nocapture`：`29 passed`
- `cargo test --lib sqlite_apply -- --nocapture`：`6 passed`
- `cargo test --lib sqlite_export -- --nocapture`：`3 passed`
- `cargo test --lib`：`490 passed / 16 ignored`
- `node scripts/harness/workbench-shape-gate.js --mode check`：`Status: pass`，`Errors: 0`，`Warnings: 0`
- `git diff --check`：输出为空

## 4. 独立复核

Arendt 初轮复核：`STATUS: CLEAR_WITH_P2`。

P2：

- Level-B outside-source guard 使用 lexical `Path::starts_with`，未覆盖 `..` / symlink 语义路径场景。
- Level-B corrupt snapshot failure injection 共享路径里仍调用 Level-A helper。

主管线修补后，Arendt 补复核：`STATUS: CLEAR`。旧 P2 已收敛，无 P0/P1。

## 5. 下一窗 B1 apply 前置

B1 apply 仍未执行。下一窗必须重新用户在场确认：

1. `PRODUCTION_DB_PATH`
2. `BACKUP_ROOT` / `ROLLBACK_MANIFEST_PATH` / `REPORT_ROOT`
3. production apply 开始

确认路径必须使用 canonical / clean path：不得含 `.` / `..`，不得使用 symlink / `/var` 这类 canonical 后文本不一致的别名路径；输出路径 canonical 后必须位于 source state root 之外。

建议路径仍应位于 source state root 外，例如用户在 B1 窗口确认且 canonical 后仍是 source sibling 的：

```text
/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/r3-migration-work/
```

B1 apply 必须使用 B0 冻结：

```text
WORKBENCH_STATE_ROOT=/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state
source_root_hash_before=2fbdb7bfdc71b30d5b4d2bec2dfdde50de98ab24942c8ba550d29b6b539d3b53
```

## 6. 边界确认

本包没有：

- 执行 B1 apply。
- 读取真实 `WORKBENCH_STATE_ROOT`。
- 创建真实 production DB。
- 修改真实 JSON / sidecar。
- 切 read path。
- 停写 JSON / sidecar。
- 新增 Tauri command。
- 修改 UI / CSS / app startup / 产品全局读写路径。
- 执行真实 Codex。
- 读取或写入 `/Users/yoyi/.codex`。

## 7. 不接受为

本包不接受为 B1 apply 已执行、真实 DB 已建、真实数据已迁、Level-A 被放宽、read-cut、stop-write、R3 完成、多 agent 解锁、真实 Codex 执行或 `.codex` 接触。
