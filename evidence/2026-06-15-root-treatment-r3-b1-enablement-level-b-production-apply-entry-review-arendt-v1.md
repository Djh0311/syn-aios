# R3 B1 Enablement Level-B Production Apply Entry Review - Arendt v1

日期：2026-06-15

复核线：Arendt (`019ec9db-8def-73d0-85a8-7cd8ff0ee74b`)

任务包：`tasks/2026-06-15-root-treatment-r3-b1-enablement-level-b-production-apply-entry-v1.md`

实现证据：`evidence/2026-06-15-root-treatment-r3-b1-enablement-level-b-production-apply-entry-v1.md`

## 1. 初轮复核结论

状态：`STATUS: CLEAR_WITH_P2`

P0/P1：无。

P2：

- Level-B outside-source guard 使用 lexical `Path::starts_with`，未覆盖 `..` / symlink 语义路径场景。当前 confirmed path + 普通绝对路径下安全，但后续真实 B1 路径确认最好要求 canonical / no-symlink，或补规范化校验与测试。
- Level-B corrupt snapshot failure injection 共享路径里仍调用 Level-A `apply_fixture_dir_to_temp_db`，只影响故障注入语义 / 覆盖，不影响正常 confirmed apply 安全门。

复核证据摘要：

- 查看任务文件和 4 个 Rust diff，确认未新增 Tauri command、UI / app startup、read-cut、stop-write、product read path 改动。
- 确认 Level-A 旧门仍在：source root temp / R3-A9 fixture guard、temp DB init / apply / export guard 未放宽。
- 确认 Level-B 为显式 confirmed config 入口，source / DB / backup / report / rollback 路径均要求等于 confirmed path，并保留 denied marker、outside source、source before / after hash invariant。
- 复跑 `cargo fmt -- --check`、`cargo test --lib sqlite_production -- --nocapture`、`cargo test --lib sqlite_apply -- --nocapture`、`cargo test --lib sqlite_export -- --nocapture`、`cargo test --lib -- --nocapture`、shape gate、`git diff --check` 均通过。

## 2. 主管线 P2 修补

主管线修补：

- Level-B output path 增加 clean / canonical guard：拒绝 `.` / `..`，要求 canonical 后文本路径一致，并用 canonical source root 做 inside-source 判断。
- Level-B source root 要求 canonical confirmed source root。
- corrupt snapshot failure injection 改为走传入的 `apply_source_to_db`，不再硬编码 Level-A helper。
- 新增 `sqlite_production_level_b_rejects_parent_dir_output_escape_into_source_root` 测试。

主管线修补后验证：

- `cargo fmt -- --check`：通过。
- `cargo test --lib sqlite_production -- --nocapture`：29 passed。
- `cargo test --lib sqlite_apply -- --nocapture`：6 passed。
- `cargo test --lib sqlite_export -- --nocapture`：3 passed。
- `cargo test --lib`：490 passed / 16 ignored。
- `node scripts/harness/workbench-shape-gate.js --mode check`：pass，0 errors / 0 warnings。
- `git diff --check`：通过。

## 3. 补复核结论

状态：`STATUS: CLEAR`

Findings：

- P0/P1：无。
- 旧 P2 已收敛：Level-B 输出路径现在拒绝 `.` / `..`，要求 canonical 文本路径，并用 canonical source root 做 inside-source 判断。
- 旧 P2 已收敛：corrupt snapshot failure injection 已改为传入的 `apply_source_to_db`，不再硬编码 Level-A helper。

补复核证据摘要：

- 复核 `workbench_sqlite_production_apply.rs` 的 Level-A / Level-B validator、canonical helpers、failure injection、测试。
- `cargo test --lib sqlite_production -- --nocapture`：29 passed。
- `git diff --check -- prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_production_apply.rs`：通过。
- diff grep 未发现新增 `tauri::command`、真实 `WORKBENCH_STATE_ROOT`、`/Users/yoyi/.codex`、read-cut / stop-write / product-read-path true 标记。

## 4. 边界

复核线未修改文件、未提交、未读取真实 `WORKBENCH_STATE_ROOT` 或 `/Users/yoyi/.codex`，未执行真实 Codex，未做真实 production DB apply。
