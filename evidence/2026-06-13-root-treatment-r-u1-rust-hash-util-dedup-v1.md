# Root Treatment / R-U1 Rust Hash Util Dedup Evidence v1

日期：2026-06-13

状态：已完成。

## 1. 实现摘要

本包只做 Rust 后端 hash helper 去重：

- 新增 `prototypes/productized-desktop-shell/src-tauri/src/utils/mod.rs`
- 新增 `prototypes/productized-desktop-shell/src-tauri/src/utils/hash.rs`
- `lib.rs` 增加 `mod utils;`
- 23 个重复 `sha256_hex` 本地定义迁到公共 helper。
- 14 个重复 `short_hash` 本地定义迁到公共 helper。

公共 helper 保留既有行为差异：

- `sha256_hex(value: &str) -> String`
- `sha256_hex_bytes(bytes: &[u8]) -> String`
- `short_hash(value: &str) -> String`，保留 16 位短 hash。
- `short_hash12(value: &str) -> String`，保留 12 位短 hash。
- `short_hash_len(value, len)` 作为长度明确的内部实现。

## 2. 形状结果

U1 前：

- `sha256_hex` 重复定义：`23`
- `short_hash` 重复定义：`14`
- `lib.rs`: `5567` 行
- `real_execution_command.rs`: `8763` 行
- `session_continuation_store.rs`: `5237` 行
- `project_workflow_automation.rs`: `5059` 行

U1 后：

- `sha256_hex` / `short_hash` 定义只剩 `utils/hash.rs` 公共 helper。
- `lib.rs`: `5567` 行，保持不增长。
- `real_execution_command.rs`: `8754` 行，下降 `9` 行。
- `session_continuation_store.rs`: `5228` 行，下降 `9` 行。
- `project_workflow_automation.rs`: `5054` 行，下降 `5` 行。
- `utils/hash.rs`: `42` 行。
- `utils/mod.rs`: `1` 行。

`lib.rs` 增加 `mod utils;` 后，通过移除 `pub use mcp::run_mcp_server_cli;` 后方一个空行抵消新增 module 声明，保持 `lib.rs` waterline 不增长；该变更不改变行为。

## 3. 行为边界

本包没有修改：

- store 业务逻辑。
- `load_store` / `empty_store` / `validate_store` 模式。
- JSON / sidecar schema。
- workflow state schema。
- 状态机语义。
- SQLite schema。
- SQLite 迁移 / read-cut / stop-write 决策。
- 真实 Codex runner / command 参数。

`project_workflow_automation.rs`、`real_execution_command.rs`、`session_continuation_store.rs` 仍保留测试用 `sha256_file` 私有 helper 和 `sha2::{Digest, Sha256}` import；这些不是 U1 的 `sha256_hex` / `short_hash` 重复目标，未顺手扩大范围。

## 4. 验证记录

在 `prototypes/productized-desktop-shell/src-tauri` 执行：

- `cargo fmt -- --check`：通过。
- `cargo test --lib memory_candidate`：通过，`9 passed`。
- `cargo test --lib formal_memory`：通过，`29 passed`。
- `cargo test --lib session_continuation`：通过，`17 passed / 4 ignored`。
- `cargo test --lib real_execution_command`：通过，`36 passed / 7 ignored`。
- `cargo test --lib workbench_sqlite`：通过，`132 passed`。
- `cargo test --lib project_workflow_automation`：通过，`15 passed / 4 ignored`。
- `cargo test --lib codex_local_runner`：通过，`12 passed`。
- `cargo test --lib`：通过，`476 passed / 16 ignored`。

保留既有 warning：

- `src/mcp/protocol.rs` 的 `JsonRpcError::invalid_params` dead_code warning。

在 `product-line` 执行：

- `node scripts/harness/workbench-shape-gate.js --mode check`：通过，`Errors: 0`，`Warnings: 0`。
- `git diff --check`：通过，无输出。

## 5. 扫描记录

重复定义扫描：

- `rg -n "fn sha256_hex|fn short_hash" prototypes/productized-desktop-shell/src-tauri/src`
- 结果只命中 `prototypes/productized-desktop-shell/src-tauri/src/utils/hash.rs`。

禁止路径 / schema 扫描：

- `git diff -- prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_schema.rs prototypes/productized-desktop-shell/src-tauri/src/workflow_state_store.rs prototypes/productized-desktop-shell/src-tauri/src/workflow_state_json_helpers.rs`
- 无输出。

shape gate 关键结果：

- `lib.rs: 5567/5567 (same)`
- `real_execution_command.rs: 8754/8763 (decreased)`
- `session_continuation_store.rs: 5228/5237 (decreased)`
- `project_workflow_automation.rs: 5054/5059 (decreased)`

## 6. 边界确认

本轮未执行真实 `codex exec` / `codex exec resume`，未发送 prompt，未读写 `/Users/yoyi/.codex`，未启动 Tauri / Browser / Chrome / Vite dev / screenshot。

本轮未迁 SQLite，未切生产 DB，未改 workflow state JSON、sidecar schema、store 业务逻辑或状态机语义。

## 7. 独立复核结果

独立复核线 `Wegener`（agent `019ec175-e945-7401-b634-7673af5ef255`）已回交 `STATUS: CLEAR`，P0/P1/P2 均无；记录见 `evidence/2026-06-13-root-treatment-r-u1-rust-hash-util-dedup-v1-review-wegener-v1.md`。

复核确认：

- 公共 helper 是否保持 16 位 / 12 位短 hash 差异。
- 字符串 hash 与字节 hash 是否保持原行为。
- 重复定义是否已归零。
- 是否未改 store 业务 / JSON / 状态机 / SQLite schema。
- 验证记录是否可信。
