# Root Treatment / R2-T1 Rust Inline Transcript / Readback Test Extraction v1

日期：2026-06-12

状态：已完成，implementation / checkpoint hash 待回填。

Planning baseline commit：`f49fb4d60210464084b674218eb11ef4e3c99eb9`

Implementation commit：待回填。

Review result：`CLEAR`，P0/P1/P2 无；复核线程继续复用 `019eb850-0698-7f70-a9b2-e7d0d668ccf5`。

Checkpoint commit：待回填。

本文是 Root Treatment / Stage R 的 R2-T1 任务包，承接 R2-T0 的 `PARTIALLY_UNLOCKED_WITH_GUARDS` 裁决，开始第一批低风险 inline tests 迁移。

## 1. 目标

把 `prototypes/productized-desktop-shell/src-tauri/src/lib.rs` 中 transcript catalog / dispatch readback stats 相关 inline tests 迁出到 crate-root test include 文件，降低 `lib.rs` 棘轮指标。

目标文件：

- 新增：`prototypes/productized-desktop-shell/src-tauri/src/lib_transcript_readback_tests.rs`
- 修改：`prototypes/productized-desktop-shell/src-tauri/src/lib.rs`

## 2. 允许范围

允许：

- 迁移 `lib.rs` 中 transcript reader / transcript catalog / dispatch readback stats 测试。
- 迁移这些测试依赖的局部 helper：`TranscriptCatalogFixture`、`transcript_catalog_fixture`、`dispatch_readback_fixture`、`dispatch_text_event`、`dispatch_stdout_event`、`transcript_index`、`write_test_rollout`、`write_test_rollout_events`、`create_test_threads_db`。
- 在 `#[cfg(test)] mod tests` 内新增 `include!("lib_transcript_readback_tests.rs");`。
- 保持 helper 和测试仍在同一个 crate-root `tests` module 中，避免扩大生产函数可见性。

禁止：

- 修改产品函数签名、可见性或行为。
- 新增 public API。
- 修改测试断言含义。
- 迁移 cross-store memory adoption、workflow execution runner、workflow machine、ignored real-state test 或共享 stub runner / factory。
- 修改 DB schema、sidecar schema、workflow state JSON、Tauri command、UI、CSS、前端代码或真实执行路径。
- 执行真实 `codex exec` / `codex exec resume`，发送 prompt，读写 `/Users/yoyi/.codex`。
- 读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript。
- 启动 Tauri / Browser / Chrome / Vite dev / screenshot。

## 3. Implementation Plan

1. 新增 `lib_transcript_readback_tests.rs`，把目标测试和局部 helper 原样移入。
2. 在 `lib.rs` 原位置保留一个 crate-root test include。
3. 确认后续测试仍可访问迁移 helper，尤其后段 dispatch readback 场景。
4. 运行 focused cargo tests、全量 cargo lib、shape gate 和 diff check。
5. 复核线只读审查 diff，再写 evidence / handoff / checkpoint。

## 4. 验收标准

必须满足：

- `lib.rs` 行数下降，且 shape gate 通过。
- 新测试文件低于 Rust 3,000 行阈值。
- `cargo test --lib transcript` 通过。
- `cargo test --lib dispatch_readback_stats` 通过。
- `cargo test --lib` 通过。
- `cargo fmt -- --check` 通过。
- `node scripts/harness/workbench-shape-gate.js --mode check` 通过。
- `git diff --check` 通过。
- 复核线 `STATUS: CLEAR` 或仅 P2。

## 5. 不接受为

本任务不接受为：

- `lib.rs <= 3,000` 达成。
- R2 全部完成。
- R3 Level B 执行或完成。
- 生产 SQLite 迁移、read-cut、stop-write 或 rollback production workflow。
- 多 agent 并行真实执行解锁。
- 真实 Codex 执行、`.codex` 读写、UI / 产品行为修改或 backlog 功能解冻。

## 6. Review Result

只读复核线 `019eb850-0698-7f70-a9b2-e7d0d668ccf5` 结论：`STATUS: CLEAR`。

复核结论：

- P0：无。
- P1：无。
- P2：无。
- 复核确认工作树范围符合 R2-T1，`lib.rs` 只留下 crate-root test include。
- 复核确认 `lib_transcript_readback_tests.rs` 只包含 transcript reader / catalog 与 dispatch readback stats 测试，以及清单内局部 helper。
- 复核确认新增文件没有网络、`std::process`、真实 Codex 执行、`/Users/yoyi/.codex`、secret/provider credential/full transcript 访问；`codex_home` 命中只是测试 fixture 字段名。
- 复核确认 `lib.rs` ratchet waterline 更新为 `13438` 符合 historical-low ratchet 收口。
