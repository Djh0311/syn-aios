# Root Treatment / R-U3 Rust Fs Ops Util Dedup Handoff v1

日期：2026-06-14

状态：已完成。

## 1. 主管线结论

R-U3 已完成实现侧准备：6 个生产 `remove_file_if_exists` 本地重复定义已收敛到 `src-tauri/src/utils/fs_ops.rs`；8 个同形状测试 fixture path helper 已改为 `fixture_dir(stage, name)`，并通过 `#[cfg(test)]` 门控。

本包只改 helper 定义 / import / 测试 fixture helper 调用方式；未修改 SQLite schema / migration / production apply / read-cut / stop-write 的业务决策。

## 2. 文件变化

新增：

- `prototypes/productized-desktop-shell/src-tauri/src/utils/fs_ops.rs`

修改：

- `prototypes/productized-desktop-shell/src-tauri/src/utils/mod.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_snapshot_apply.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_production_apply.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_observation_period.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_stop_write.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_dual_write.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_read_cut.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_importer.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_exporter.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_apply.rs`

证据：

- `evidence/2026-06-14-root-treatment-r-u3-rust-fs-ops-util-dedup-v1.md`

## 3. 形状结果

- `remove_file_if_exists` 本地定义归零，只剩 `utils/fs_ops.rs` 一份。
- 8 个同形状测试 fixture helper 归零，只剩 `utils/fs_ops.rs` 一份 `fixture_dir(stage, name)`。
- 4 个 deferred fixture helper 保留在 `snapshot_apply` / `production_apply` / `stop_write` / `transaction_acceptance`。
- `lib.rs` 保持 5567 行。
- shape gate 通过，0 errors / 0 warnings。

## 4. 验证

已通过：

- `cargo fmt -- --check`
- `cargo test --lib fs_ops`
- `cargo test --lib workbench_sqlite_importer`
- `cargo test --lib workbench_sqlite_exporter`
- `cargo test --lib workbench_sqlite_apply`
- `cargo test --lib workbench_sqlite_dual_write`
- `cargo test --lib workbench_sqlite_read_cut`
- `cargo test --lib workbench_sqlite_observation_period`
- `cargo test --lib workbench_sqlite_snapshot_apply`
- `cargo test --lib workbench_sqlite_production_apply`
- `cargo test --lib workbench_sqlite_stop_write`
- `cargo test --lib workbench_sqlite_transaction_acceptance`
- `cargo test --lib`，`480 passed / 16 ignored`
- `node scripts/harness/workbench-shape-gate.js --mode check`
- `git diff --check`

## 5. 扫描结论

- `rg -n "fn remove_file_if_exists\\(" prototypes/productized-desktop-shell/src-tauri/src` 只命中 `utils/fs_ops.rs`。
- `rg -n "fn fixture_dir\\(|fn fixture_dir_a10\\(|fn fixture_dir_r3_a11\\(" ...` 只命中 `utils/fs_ops.rs` 和 4 个 deferred helper。
- `workbench_sqlite_schema.rs`、`workflow_state_store.rs`、`workflow_state_json_helpers.rs` 无 diff。
- `manifest_r3_aX_fixture_root` / `r3_aX_fixture_root` 簇保留原地，未迁移。

## 6. 独立复核结果

独立复核 agent `Hilbert`（`019ec1e4-f49c-7d90-9166-ffd8b1bb1a42`）回交 `STATUS: CLEAR_WITH_P2`。

- P0/P1：无。
- P2：evidence 的验证执行目录记录不够精确；shape gate 应从仓库根运行，不是在 `src-tauri` 内运行。该 P2 已修正到 evidence 第 6 节，不影响代码行为。

复核确认：

- `remove_file_if_exists` 只剩 `utils/fs_ops.rs` 一份，语义和错误前缀保持。
- 8 个同形状 fixture helper 已归零，只剩 `fixture_dir(stage, name)`，stage 映射逐项正确。
- importer 从 `PathBuf::from(env!(...))` 改为公共 helper 后最终路径一致。
- Deferred 的 4 个 fixture helper 保留，root 簇未迁移。
- 未发现 schema / store / JSON / 状态机 / 真实 Codex 路径相关 diff。
- 复核线复跑通过 `cargo fmt -- --check`、`git diff --check`、`node scripts/harness/workbench-shape-gate.js --mode check`；未复跑 `cargo test`，但确认 evidence 中 `cargo test --lib 480 passed / 16 ignored` 与新增 4 个 fs_ops 测试后的计数一致。

## 7. 停止线

主管线完成 implementation commit 和 checkpoint commit 后，停在 U3 复核点；不得顺手进入 U4 / U5 / U-Gate、R3 Level B 或 backlog 解冻。
