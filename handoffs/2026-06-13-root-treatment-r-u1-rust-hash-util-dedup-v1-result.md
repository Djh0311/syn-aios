# Root Treatment / R-U1 Rust Hash Util Dedup Handoff v1

日期：2026-06-13

状态：已完成。

## 1. 主管线结论

R-U1 已完成实现侧闭环：`sha256_hex` / `short_hash` 重复 helper 已迁入 `src-tauri/src/utils/hash.rs`，调用点改为公共 helper，行为差异保持。

本包只做 hash util 去重，不改 store 业务 / JSON / 状态机 / SQLite schema / 真实执行参数。

## 2. 文件变化

新增：

- `prototypes/productized-desktop-shell/src-tauri/src/utils/mod.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/utils/hash.rs`

修改：

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- 23 个原含 `sha256_hex` / `short_hash` 重复 helper 的 Rust 文件。

证据：

- `evidence/2026-06-13-root-treatment-r-u1-rust-hash-util-dedup-v1.md`

## 3. 形状结果

- `sha256_hex` / `short_hash` 定义只剩公共 helper。
- `lib.rs`: `5567` 行，保持不增长。
- `real_execution_command.rs`: `8754` 行。
- `session_continuation_store.rs`: `5228` 行。
- `project_workflow_automation.rs`: `5054` 行。
- `utils/hash.rs`: `42` 行。
- `utils/mod.rs`: `1` 行。

## 4. 验证

已通过：

- `cargo fmt -- --check`
- `cargo test --lib memory_candidate`
- `cargo test --lib formal_memory`
- `cargo test --lib session_continuation`
- `cargo test --lib real_execution_command`
- `cargo test --lib workbench_sqlite`
- `cargo test --lib project_workflow_automation`
- `cargo test --lib codex_local_runner`
- `cargo test --lib`，`476 passed / 16 ignored`
- `node scripts/harness/workbench-shape-gate.js --mode check`
- `git diff --check`

## 5. 扫描结论

- `rg -n "fn sha256_hex|fn short_hash" prototypes/productized-desktop-shell/src-tauri/src` 只命中 `utils/hash.rs`。
- `workbench_sqlite_schema.rs`、`workflow_state_store.rs`、`workflow_state_json_helpers.rs` 无 diff。
- shape gate 通过，`lib.rs` 未增长。

## 6. 独立复核结果

独立复核线 `Wegener`（agent `019ec175-e945-7401-b634-7673af5ef255`）已回交 `STATUS: CLEAR`，P0/P1/P2 均无；记录见 `evidence/2026-06-13-root-treatment-r-u1-rust-hash-util-dedup-v1-review-wegener-v1.md`。

复核确认：

- 16 位 / 12 位短 hash 语义是否保持。
- `&str` / `&[u8]` sha256 语义是否保持。
- 是否仅做 util 去重，没有碰 store 业务 / JSON / 状态机 / SQLite schema。
- 验证记录是否可信。

## 7. 停止线

复核已 CLEAR，主管线可提交 implementation commit，并停在 R-U1 复核点。

不得顺手进入 U2 / U3 / U4 / U5 / U-Gate、R3 Level B 或 backlog 解冻。
