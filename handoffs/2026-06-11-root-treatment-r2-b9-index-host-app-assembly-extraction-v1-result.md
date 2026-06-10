# Root Treatment R2-B9 Index Host App Assembly Extraction v1 Result

日期：2026-06-11

## 结论

R2-B9 已完成。接受范围是 `src-tauri/src/lib.rs` 中从 `software_key_of_session` 到 Tauri `run()` 的连续尾段，已抽出到 `src-tauri/src/index_host_app_entrypoints.rs`，并通过 `include!("index_host_app_entrypoints.rs")` 在 crate root 展开，保持行为和可见性不变。

不接受为 R2 全部完成、`lib.rs <= 15,000` 目标完成、R3 SQLite、R4 UI / 按页读模型、Stage L / K3-B1 / K3-B2 恢复或新的真实 Codex 执行授权。

## 做了什么

- 新增 `index_host_app_entrypoints.rs`。
- `lib.rs` 原尾段替换为 crate-root `include!`。
- 未移动 inline tests；测试巨石仍留在 `lib.rs`。
- 未同步入口文档，入口同步留给主管线 checkpoint。

## 改动文件

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/index_host_app_entrypoints.rs`
- `evidence/2026-06-11-root-treatment-r2-b9-index-host-app-assembly-extraction-v1.md`
- `handoffs/2026-06-11-root-treatment-r2-b9-index-host-app-assembly-extraction-v1-result.md`

## 指标

- `lib.rs`：17,042 lines -> 16,457 lines。
- `index_host_app_entrypoints.rs`：新增 586 lines。
- Tauri command registry：96 total，`lib.rs` 内 `#[tauri::command]` 为 0。
- Sidecar JSON kinds：14 detected / 0 unknown。

## 抽出函数 / 类型

- `software_key_of_session`
- `load_sessions`
- `load_sessions_from_sqlite_or_index`
- `overlay_project_thread_counts`
- `parse_projects`
- `parse_sessions`
- `parse_codex_transcript`
- `parse_codex_transcript_event`
- `parse_skills`
- `parse_plugins`
- `parse_file_candidates`
- `parse_harness_candidates`
- `parse_harness_resources`
- `parse_harness_entrypoints`
- `parse_tasks`
- `allowed_paths`
- `allowed_paths_with_sessions`
- `extend_allowed_rollouts_from_sqlite`
- `impl AllowedPaths`
- `array_len`
- `optional_string`
- `optional_string_from`
- `optional_i64_from`
- `string_array`
- `usize_value`
- `i64_value`
- `usize_map`
- `bool_value`
- `path_name`
- `copy_to_clipboard`（macOS / non-macOS variants）
- `run_open`（macOS / non-macOS variants）
- `run`

## 验证

已通过：

- `node scripts/harness/workbench-shape-gate.js --mode baseline`
- `node scripts/harness/workbench-shape-gate.js --mode check`
- `cargo test --lib transcript`，16 passed / 0 failed；filter 有匹配，无需 fallback。
- `cargo test --lib workbench_snapshot`，1 passed / 0 failed。
- `cargo test --lib workflow_state`，11 passed / 0 failed。
- `cargo test --lib`，336 passed / 0 failed / 16 ignored。
- `cargo fmt -- --check`
- `git diff --check`
- `git status --short`，提交前仅包含 R2-B9 范围文件。

已知 warning：

- Rust 测试仍有既有 dead_code warning：`JsonRpcError::invalid_params` 未使用。

## Commit

- Start commit：`d100d73c39ddb014372c48ea5a7eaa643fd15bf7`
- End commit：本文件随 R2-B9 completion commit 一起提交；提交创建后由最终回交记录实际 hash。

说明：completion commit 的实际 hash 无法稳定写入同一 commit 内的文件内容；本 handoff 记录 start commit 和 completion commit 关系，实际 end hash 以开发线最终回交和主管线 checkpoint / backfill 为准。

## 边界

- 未执行真实 `codex exec` / `codex exec resume`。
- 未发送 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未读取 secret、token、`.env`、keychain、OAuth、provider credential、完整 transcript 或 rollout。
- 未启动 Tauri / Browser / Chrome / Vite / 截图工具。
- 未迁移 SQLite，未改 workflow state schema，未新增 sidecar JSON 种类，未新增 Tauri command。
- 未改 UI / TypeScript。
- 未同步入口文档。

## P0 / P1 / P2

- P0：无。
- P1：无。
- P2：`include!` 仍是保守过渡，后续 R2 可再收敛正式模块边界。
- P2：R2-B9 不是 R2 全部完成，也不是 `lib.rs <= 15,000` 水位线完成。
- P2：inline tests 仍主要留在 `lib.rs`，后续 R2 后段再迁移。

## 不能声明完成

- 不能声明 R2 全部完成。
- 不能声明 R3 SQLite 或 workflow state schema 迁移完成。
- 不能声明 UI / Tauri 截图验收完成。
- 不能声明真实 Codex send / resume / exec、新真实执行授权、K3-B1 retry、K3-B2 或 Stage L 恢复完成。
- 不能声明前段 transcript loader、C4-C6、task package render、shared workflow utility、snapshot assembly、atomic helper 或 inline tests 巨石已经拆完。
