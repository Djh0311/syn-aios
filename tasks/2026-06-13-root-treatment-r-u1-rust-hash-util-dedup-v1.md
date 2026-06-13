# Root Treatment / R-U1 Rust Hash Util Dedup v1

日期：2026-06-13

状态：已完成。

性质：R-U 后端 util 去重第 1 包。本包只把 Rust 后端重复的 `sha256_hex` / `short_hash` 私有 helper 收敛到 `src-tauri/src/utils/hash.rs`，并将调用点改为公共 helper；严格无行为变化。

Planning baseline：`c8b3a1e`。

Task package commit：`5a295e0`。

## 0. 主管线理解

用户要求按合并正本进入 R-U，先开 U1：

- 读取 `docs/plans/2026-06-13-stage-r-remaining-execution-plan-v1.md` §3。
- 读取 `decisions/2026-06-01-architecture-module-split-guardrail-v1.md`。
- 将 `sha256_hex` / `short_hash` 抽到 `src-tauri/src/utils/hash.rs`。
- 23+14 处调用点改调公共函数。
- 无行为变化，以 `cargo test --lib` 全绿作为铁证。
- 不碰 store 业务 / JSON / 状态机。
- 不迁 SQLite。
- 不合并 store 模式。
- 完成后交独立复核线 CLEAR，再 commit，再停复核点。
- 完成 / 提交报告必须附 `git log` 实际输出。

## 1. 当前扫描事实

当前重复定义：

- `sha256_hex`: `23` 个定义。
- `short_hash`: `14` 个定义。

当前存在三类行为，必须保留：

- 字符串 sha256：`fn sha256_hex(value: &str) -> String`。
- 字节 sha256：`fn sha256_hex(bytes: &[u8]) -> String`，主要用于 SQLite / 文件 hash。
- 短 hash：
  - 多数模块截取 `16` 位。
  - `real_execution_command.rs` 和 `memory_capture_bus.rs` 截取 `12` 位。

因此本包不得把 12 位短 hash 误改为 16 位，也不得把字节 hash 改为字符串 hash。

## 2. 目标

完成后：

- 新增 `prototypes/productized-desktop-shell/src-tauri/src/utils/mod.rs`。
- 新增 `prototypes/productized-desktop-shell/src-tauri/src/utils/hash.rs`。
- `lib.rs` 增加 `mod utils;`。
- 重复私有 helper 删除或改为公共 helper import。
- 调用点行为保持不变：
  - 原 `sha256_hex(&str)` 仍输出相同 sha256 hex。
  - 原 `sha256_hex(&[u8])` 仍输出相同 sha256 hex。
  - 原 16 位 `short_hash` 仍是 sha256 hex 前 16 位。
  - 原 12 位 `short_hash` 仍是 sha256 hex 前 12 位。

## 3. 允许范围

允许修改：

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/utils/mod.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/utils/hash.rs`
- 当前含重复 `sha256_hex` / `short_hash` helper 的 Rust 文件。
- `tasks/2026-06-13-root-treatment-r-u1-rust-hash-util-dedup-v1.md`
- 对应 evidence / handoff / review evidence。
- 必要 checkpoint 入口。

允许的代码变化仅限：

- 增加 `crate::utils::hash::*` import。
- 将本地 helper 删除。
- 对 SQLite 字节 hash 调用改为 `sha256_hex_bytes(...)`。
- 对 12 位短 hash 调用改为 `short_hash12(...)` 或等价公共 helper。
- 对 16 位短 hash 调用改为 `short_hash(...)`。

## 4. 禁止范围

禁止：

- 修改 store 业务逻辑。
- 修改 JSON / sidecar / workflow state schema。
- 修改状态机语义。
- 迁移 SQLite。
- 合并 `load_store` / `empty_store` / `validate_store` 等 store 模式。
- 修改真实 Codex runner / command 参数。
- 执行真实 `codex exec` / `codex exec resume`。
- 发送 prompt。
- 读写 `/Users/yoyi/.codex`。
- 启动 Tauri / Browser / Chrome / Vite dev / screenshot。
- 解冻 backlog。

若抽取任一 helper 需要改变业务语义或牵连 store schema / 状态机，本包必须停止，并把该 helper 留原地记 deferred。

## 5. 实现步骤

1. 新增 `utils/mod.rs` 和 `utils/hash.rs`。
2. 在 `lib.rs` 声明 `mod utils;`。
3. 在重复 helper 文件中引入公共 hash helper。
4. 删除本地 `sha256_hex` / `short_hash` 定义。
5. 保留 16 位 / 12 位短 hash 差异。
6. 保留 `&str` / `&[u8]` 输入差异。

## 6. 验证

必须通过：

- `cargo fmt -- --check`
- 聚焦测试：
  - `cargo test --lib memory_candidate`
  - `cargo test --lib formal_memory`
  - `cargo test --lib session_continuation`
  - `cargo test --lib real_execution_command`
  - `cargo test --lib workbench_sqlite`
  - `cargo test --lib project_workflow_automation`
  - `cargo test --lib codex_local_runner`
- `cargo test --lib`
- `node scripts/harness/workbench-shape-gate.js --mode check`
- `git diff --check`

建议扫描：

- `rg -n "fn sha256_hex|fn short_hash" prototypes/productized-desktop-shell/src-tauri/src`
- `rg -n "short_hash12|short_hash\\(" prototypes/productized-desktop-shell/src-tauri/src`
- `git diff -- prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_schema.rs prototypes/productized-desktop-shell/src-tauri/src/workflow_state_store.rs prototypes/productized-desktop-shell/src-tauri/src/workflow_state_json_helpers.rs`

## 7. 复核结果

独立复核线 `Wegener`（agent `019ec175-e945-7401-b634-7673af5ef255`）已于 2026-06-13 回交 `STATUS: CLEAR`，P0/P1/P2 均无；记录见 `evidence/2026-06-13-root-treatment-r-u1-rust-hash-util-dedup-v1-review-wegener-v1.md`。

复核确认：

- 23 个 `sha256_hex` 重复定义是否归零或只剩公共 helper。
- 14 个 `short_hash` 重复定义是否归零或只剩公共 helper。
- 16 位与 12 位短 hash 行为是否保持。
- 字节 hash 行为是否保持。
- 是否未改 store 业务 / JSON / 状态机 / SQLite 迁移路径。
- 是否未改真实执行 guard / command 参数。
- 验证记录是否可信。

## 8. 不接受为

本包不接受为：

- R-U 全部完成。
- U2 / U3 / U4 / U5 / U-Gate 完成。
- store 模式合并完成。
- R3 Level B 执行。
- SQLite 真实切换。
- 真实 Codex 执行。
- backlog 解冻。

## 9. 停止线

U1 完成、独立复核线 CLEAR、implementation commit 后，停在 U1 复核点；不得顺手进入 U2。
