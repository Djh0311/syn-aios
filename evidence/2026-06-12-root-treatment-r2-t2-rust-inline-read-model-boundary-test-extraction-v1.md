# Evidence: Root Treatment / R2-T2 Rust Inline Read Model Boundary Test Extraction v1

日期：2026-06-12

状态：已完成，hash 已回填。

任务包：`tasks/2026-06-12-root-treatment-r2-t2-rust-inline-read-model-boundary-test-extraction-v1.md`

Planning baseline commit：`3e65ee7c528c06a78bd7e28849bc5751f266522e`

Implementation commit：`818b887a46cf1ddac62c276c5788be37a9474647`

Review result：`CLEAR`，P0/P1/P2 无；复核线程 `019eb850-0698-7f70-a9b2-e7d0d668ccf5`

Checkpoint commit：`3123ec510daa5532d7663dd4b99ed66db4003d13`

## 1. 本轮目标

承接 R2-T0 `PARTIALLY_UNLOCKED_WITH_GUARDS` 裁决和 R2-T1 首批 inline tests 迁移结果，迁移第二批低风险 Rust inline tests：diagnostics / adapter / session operation / provider availability / session continuation guard 读模型边界测试。

## 2. 改动范围

修改：

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `scripts/harness/workbench-shape-gate.js`

新增：

- `prototypes/productized-desktop-shell/src-tauri/src/lib_read_model_boundary_tests.rs`
- `tasks/2026-06-12-root-treatment-r2-t2-rust-inline-read-model-boundary-test-extraction-v1.md`

本轮未修改：

- 产品函数签名、可见性或行为。
- K3-B runtime prompt guard 测试。
- `reads_real_static_index_summary`。
- Tauri command、DB schema、sidecar schema、workflow state JSON。
- 前端 UI / CSS / TS。
- 真实执行路径。

## 3. 迁移内容

从 `lib.rs` 的 `#[cfg(test)] mod tests` 中迁出：

- `path_whitelist_accepts_only_index_projects_and_rollouts`
- `snapshot_keeps_metadata_without_session_body`
- `g2_diagnostic_summary_reports_degraded_store_without_repair`
- `workbench_snapshot_includes_backend_agent_adapter_descriptor`
- `backend_agent_adapter_descriptor_is_stable_without_codex_signals`
- `session_operation_descriptors_cover_e2_boundary_matrix`
- `provider_availability_summaries_cover_e3_boundary_matrix`
- `session_continuation_guard_covers_e4_boundary_matrix`

迁移方式：

- 新增 `lib_read_model_boundary_tests.rs`，仍由 `#[cfg(test)] mod tests` 内 `include!("lib_read_model_boundary_tests.rs");` 引入。
- 测试仍处于同一个 crate-root test module 中，不新增 public API，不扩大生产函数可见性。
- 原位置未迁移 K3-B guard 测试，也未迁移真实静态索引读取测试。

## 4. 棘轮结果

行数：

```text
lib.rs: 13,438 -> 12,699
lib_read_model_boundary_tests.rs: 740
```

shape gate 水位线：

```text
prototypes/productized-desktop-shell/src-tauri/src/lib.rs: 12,699
```

本轮将 R2-T2 的收益写入 `workbench-shape-gate.js`，使 `lib.rs` 新历史低点成为后续防回涨基线。

## 5. 验证结果

在 `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri`：

```text
cargo fmt -- --check
```

结果：通过。

```text
cargo test --lib diagnostic
```

结果：通过，4 passed，483 filtered out。

```text
cargo test --lib adapter
```

结果：通过，6 passed，481 filtered out。

```text
cargo test --lib session_operation
```

结果：通过，1 passed，486 filtered out。

```text
cargo test --lib provider_availability
```

结果：通过，1 passed，486 filtered out。

```text
cargo test --lib session_continuation
```

结果：通过，17 passed，4 ignored，466 filtered out。

```text
cargo test --lib
```

结果：通过，471 passed，16 ignored。

在 `/Users/yoyi/workspace/product-line`：

```text
node scripts/harness/workbench-shape-gate.js --mode check
```

结果：通过，0 errors，0 warnings；`Ratchet policy: historical_lowest_closed_value`；`lib.rs: 12699/12699 (same)`。

```text
git diff --check
```

结果：通过，无输出。

既有 warning：

- Rust 保留既有 warning：`JsonRpcError::invalid_params` never used。

说明：

- 本轮未运行 npm / build，因为未改前端产品代码。

## 6. 边界确认

本轮没有执行真实 `codex exec` / `codex exec resume`，没有发送 prompt，没有读写 `/Users/yoyi/.codex`，没有读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript。

本轮没有启动 Tauri / Browser / Chrome / Vite dev / screenshot。

收尾过程偏差：

- hash 回填扫描第一次误把 Markdown 反引号放在 shell 双引号内，zsh 尝试执行 `TBD` 并返回 `command not found: TBD`；未触发真实 Codex、未读写 `/Users/yoyi/.codex`、未改文件。随后已用单引号安全重跑扫描，无 `TBD` / `待 hash 回填` 残留。

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

- P0：无。
- P1：无。
- P2：无。
- 复核确认 tracked diff 只包含 `lib.rs` 和 `workbench-shape-gate.js`；新增文件只包含 R2-T2 测试文件与 task/evidence/handoff 文档。
- `lib_read_model_boundary_tests.rs` 中 `#[test]` 数量为 8，正好对应任务包允许的 read-model boundary 测试清单。
- K3-B runtime prompt guard 测试和 `reads_real_static_index_summary` 均保留在 `lib.rs`。
- 新测试文件中的 `/Users/yoyi/.codex/...` 只作为 JSON/string fixture 和 `can_copy` 断言输入出现；未发现实际读写 `/Users/yoyi/.codex`、`std::process`、网络、真实 Codex 执行或 Tauri command。
- 未发现产品函数签名、可见性、DB/schema、sidecar、workflow state、UI/CSS/TS 或真实执行路径变更。
- `lib.rs` 当前 12,699 行，`workbench-shape-gate.js` waterline 已更新为 12,699，符合 historical-low ratchet。
- `git diff --check` 无输出。

Residual risk：本复核未重跑主管线列出的 cargo/shape gate 验证，只做静态只读审查。
