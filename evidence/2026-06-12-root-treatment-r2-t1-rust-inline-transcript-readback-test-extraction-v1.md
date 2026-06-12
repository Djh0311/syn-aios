# Evidence: Root Treatment / R2-T1 Rust Inline Transcript / Readback Test Extraction v1

日期：2026-06-12

状态：已完成，hash 已回填。

任务包：`tasks/2026-06-12-root-treatment-r2-t1-rust-inline-transcript-readback-test-extraction-v1.md`

Planning baseline commit：`f49fb4d60210464084b674218eb11ef4e3c99eb9`

Implementation commit：`1a470f4578934d398218a4b8eaed34b307fb329d`

Review result：`CLEAR`，P0/P1/P2 无；复核线程 `019eb850-0698-7f70-a9b2-e7d0d668ccf5`

Checkpoint commit：`521acd55263c6f77490172dff8e93983dcc914be`

## 1. 本轮目标

承接 R2-T0 `PARTIALLY_UNLOCKED_WITH_GUARDS` 裁决，迁移第一批低风险 Rust inline tests：transcript reader / transcript catalog / dispatch readback stats。

## 2. 改动范围

修改：

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `scripts/harness/workbench-shape-gate.js`

新增：

- `prototypes/productized-desktop-shell/src-tauri/src/lib_transcript_readback_tests.rs`
- `tasks/2026-06-12-root-treatment-r2-t1-rust-inline-transcript-readback-test-extraction-v1.md`

没有修改：

- 产品函数签名、可见性或行为。
- Tauri command、DB schema、sidecar schema、workflow state JSON。
- 前端 UI / CSS / TS。
- 真实执行路径。

## 3. 迁移内容

从 `lib.rs` 的 `#[cfg(test)] mod tests` 中迁出：

- `transcript_reader_rejects_thread_outside_index`
- `parses_transcript_reader_output`
- transcript catalog 相关测试
- dispatch readback stats 相关测试
- 局部 helper：`TranscriptCatalogFixture`、`transcript_catalog_fixture`、`dispatch_readback_fixture`、`dispatch_text_event`、`dispatch_stdout_event`、`transcript_index`、`write_test_rollout`、`write_test_rollout_events`、`create_test_threads_db`

迁移方式：

- 新增 `lib_transcript_readback_tests.rs`，仍由 `#[cfg(test)] mod tests` 内 `include!("lib_transcript_readback_tests.rs");` 引入。
- helper 和测试仍处于同一个 crate-root test module 中，后续 readback 测试仍可访问 helper。
- 不新增 public API，不扩大生产函数可见性。

## 4. 棘轮结果

行数：

```text
lib.rs: 13,965 -> 13,438
lib_transcript_readback_tests.rs: 528
```

shape gate 水位线：

```text
prototypes/productized-desktop-shell/src-tauri/src/lib.rs: 13,438
```

本轮将 R2-T1 的收益写入 `workbench-shape-gate.js`，使 `lib.rs` 新历史低点成为后续防回涨基线。

## 5. 验证结果

在 `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri`：

```text
cargo test --lib transcript
```

结果：通过，16 passed，0 failed，471 filtered out。

```text
cargo test --lib dispatch_readback_stats
```

结果：通过，6 passed，0 failed，481 filtered out。

```text
cargo test --lib
```

结果：通过，471 passed，16 ignored。

```text
cargo fmt -- --check
```

结果：通过。

在 `/Users/yoyi/workspace/product-line`：

```text
node scripts/harness/workbench-shape-gate.js --mode check
```

结果：通过，0 errors，0 warnings；`Ratchet policy: historical_lowest_closed_value`；`lib.rs: 13438/13438 (same)`。

```text
git diff --check
```

结果：通过，无输出。

说明：

- Rust 保留既有 warning：`JsonRpcError::invalid_params` never used。
- 本轮未运行 npm / build，因为未改前端产品代码。

## 6. 边界确认

本轮没有执行真实 `codex exec` / `codex exec resume`，没有发送 prompt，没有读写 `/Users/yoyi/.codex`，没有读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript。

本轮没有启动 Tauri / Browser / Chrome / Vite dev / screenshot。

本轮不接受为：

- `lib.rs <= 3,000` 达成。
- R2 全部完成。
- R3 Level B 执行或完成。
- 生产 SQLite 迁移、read-cut、stop-write 或 rollback production workflow。
- 多 agent 并行真实执行解锁。
- 真实 Codex 执行、`.codex` 读写、UI / 产品行为修改或 backlog 功能解冻。

## 7. 复核结果

复核线程：`019eb850-0698-7f70-a9b2-e7d0d668ccf5`

结论：`STATUS: CLEAR`

复核摘要：

- P0：无。
- P1：无。
- P2：无。
- 工作树范围符合 R2-T1，tracked diff 只有 `lib.rs` 和 `workbench-shape-gate.js`，新增 `lib_transcript_readback_tests.rs` 与 task/evidence/handoff 文档。
- `lib.rs` 只留下 `include!("lib_transcript_readback_tests.rs");`。
- 新文件只包含允许的 transcript/readback 测试与局部 helper；测试写 temp rollout/sqlite fixture，属于原测试语义。
- 新文件未发现网络、`std::process`、真实 Codex 执行、`/Users/yoyi/.codex`、secret/provider credential/full transcript 访问。
- `lib.rs` ratchet waterline 更新为 `13438` 符合 historical-low ratchet 收口。

Residual risk：本复核未重跑 cargo/shape gate，只做静态只读复核；后续 R2-T2 仍需继续防止扩大生产可见性或夹带 runner/real-state/跨 store 端到端组迁移。
