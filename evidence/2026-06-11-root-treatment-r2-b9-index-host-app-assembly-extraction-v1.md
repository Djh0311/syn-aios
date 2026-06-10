# Root Treatment R2-B9 Index Host App Assembly Extraction v1 Evidence

日期：2026-06-11

## 结论

R2-B9 已完成行为不变的 `lib.rs` 尾段物理抽出：

- 将 `software_key_of_session` 到 Tauri `run()` 的连续尾段搬入 `prototypes/productized-desktop-shell/src-tauri/src/index_host_app_entrypoints.rs`。
- `lib.rs` 原位置保留 `include!("index_host_app_entrypoints.rs")`，继续在 crate root 展开 helper，避免函数可见性改动。
- 未迁移 inline tests；`#[cfg(test)] mod tests` 仍留在 `lib.rs`。
- 未改函数语义、返回值、错误文案、公开 command/type/schema。

R2-B9 可接受为：index parsing / allowed paths / host OS helper / Tauri app assembly 尾段已从 `lib.rs` 抽出，`lib.rs` 行数继续下降，command 总量和 `lib.rs` 内 command 数量保持不变，并通过 shape gate、任务包指定 Rust 测试、全量库测试和格式检查。

## Commit

- Start commit：`d100d73c39ddb014372c48ea5a7eaa643fd15bf7`
- End commit：本文件随 R2-B9 completion commit 一起提交；提交创建后由最终回交记录实际 hash。

说明：completion commit 的实际 hash 无法稳定写入同一 commit 内的文件内容；本 evidence 记录 start commit 和 completion commit 关系，实际 end hash 以开发线最终回交和主管线 checkpoint / backfill 为准。

## 改动文件

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/index_host_app_entrypoints.rs`
- `evidence/2026-06-11-root-treatment-r2-b9-index-host-app-assembly-extraction-v1.md`
- `handoffs/2026-06-11-root-treatment-r2-b9-index-host-app-assembly-extraction-v1-result.md`

## 形状指标

| 指标 | R2-B8 / start | R2-B9 / current |
| --- | ---: | ---: |
| `lib.rs` 行数 | 17,042 | 16,457 |
| `index_host_app_entrypoints.rs` 行数 | 0 | 586 |
| Tauri command 总量 | 96 | 96 |
| `lib.rs` 内 `#[tauri::command]` | 0 | 0 |
| Sidecar JSON kinds | 14 detected / 0 unknown | 14 detected / 0 unknown |

说明：

- 新 helper 文件 586 行，低于 Rust 3,000 行治理阈值。
- 本轮选择 crate-root `include!`。原因是抽出尾段仍依赖 crate-root private helper、`AppState`、`AllowedPaths`、command registry 和 inline tests；正式 `mod` 会要求扩大可见性修改，不符合本任务的行为不变和小风险边界。
- 本轮只搬移 session/index parser、allowed path derivation、host OS helper 和 Tauri app assembly 尾段；未拆前段 transcript loader、C4-C6、task package render、shared workflow utility、snapshot assembly、atomic helper 或 inline tests。

## 抽出范围

本轮抽出 `lib.rs` 中 `diagnostics_provider_session_entrypoints.rs` include 之后、`#[cfg(test)] mod tests` 之前的连续尾段：

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

留在本批次外：

- `read_index`
- `load_codex_session_transcript_for_index`
- `load_codex_session_transcript_with_catalog`
- `load_codex_session_transcript_with_optional_catalog`
- `load_codex_session_transcript_from_sqlite_row`
- `load_codex_session_transcript_from_index_thread`
- `find_index_thread`
- `codex_home_from_index`
- C4-C6 自动化工作流治理
- task package render / finder helper
- shared workflow utility
- workbench snapshot assembly
- atomic path / time helper
- inline tests 巨石

## 验证命令

已运行：

```bash
node scripts/harness/workbench-shape-gate.js --mode baseline
node scripts/harness/workbench-shape-gate.js --mode check
cargo test --lib transcript
cargo test --lib workbench_snapshot
cargo test --lib workflow_state
cargo test --lib
cargo fmt -- --check
git diff --check
git status --short
```

结果：

- `node scripts/harness/workbench-shape-gate.js --mode baseline`：通过，Status `pass`，0 errors / 0 warnings / 12 info。
- baseline key metrics：`lib.rs` 16,457 lines；Tauri commands 96 total / 0 in `lib.rs`；Sidecar JSON kinds 14 detected / 0 unknown；`lib.rs` ratchet 16,457 / 25,925，status `decreased`。
- `node scripts/harness/workbench-shape-gate.js --mode check`：通过，Status `pass`，0 errors / 0 warnings / 12 info。
- check key metrics：`lib.rs` 16,457 lines；Tauri commands 96 total / 0 in `lib.rs`；Sidecar JSON kinds 14 detected / 0 unknown；`lib.rs` ratchet 16,457 / 25,925，status `decreased`。
- `cargo test --lib transcript`：通过，16 passed / 0 failed / 336 filtered out；filter 有匹配，无需 fallback；保留既有 warning：`JsonRpcError::invalid_params` dead_code。
- `cargo test --lib workbench_snapshot`：通过，1 passed / 0 failed / 351 filtered out；保留同一既有 warning。
- `cargo test --lib workflow_state`：通过，11 passed / 0 failed / 341 filtered out；保留同一既有 warning。
- `cargo test --lib`：通过，336 passed / 0 failed / 16 ignored；ignored 均为显式真实执行授权测试；保留同一既有 warning。
- `cargo fmt -- --check`：通过，无输出。
- `git diff --check`：通过，无输出。
- `git status --short`：提交前仅包含 R2-B9 范围文件。

## 边界确认

- 未改产品业务逻辑。
- 未改函数语义、返回值、错误文案或公开 Tauri command 契约。
- 未改 workflow state 顶层 schema。
- 未新增 `#[tauri::command]`。
- 未新增 sidecar store 或 sidecar JSON 种类。
- 未迁移 SQLite。
- 未做 UI / TypeScript / Browser / Tauri window / screenshot。
- 未改真实 Codex runner。
- 未执行真实 `codex exec` / `codex exec resume`。
- 未发送 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未读取 secret、token、`.env`、keychain、OAuth、provider credential、完整 transcript 或 rollout。
- 未启动 Tauri / Browser / Chrome / Vite / 截图工具。
- 未启动 Stage L / K3-B1 retry / K3-B2。
- 未解冻 backlog 功能。
- 未同步 `CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md`。

## P0 / P1 / P2

- P0：无。
- P1：无。
- P2：继续使用 `include!` 作为保守过渡；后续 R2 可再收敛为正式模块边界。
- P2：本轮只抽出 index parsing / allowed paths / host OS helper / Tauri app assembly 尾段，不代表 R2 全部完成、`lib.rs <= 15,000` 完成、R3 SQLite、R4 按页读模型、UI、Stage L 恢复或真实 Codex 执行授权完成。
- P2：inline tests 仍主要留在 `lib.rs`，后续 R2 后段可按领域迁移 tests。
