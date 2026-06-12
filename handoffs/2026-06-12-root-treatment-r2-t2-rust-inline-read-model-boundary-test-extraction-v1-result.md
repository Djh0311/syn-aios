# Handoff: Root Treatment / R2-T2 Rust Inline Read Model Boundary Test Extraction v1 Result

日期：2026-06-12

状态：已完成，待 hash 回填。

任务包：`tasks/2026-06-12-root-treatment-r2-t2-rust-inline-read-model-boundary-test-extraction-v1.md`

Evidence：`evidence/2026-06-12-root-treatment-r2-t2-rust-inline-read-model-boundary-test-extraction-v1.md`

Planning baseline commit：`3e65ee7c528c06a78bd7e28849bc5751f266522e`

Implementation commit：`TBD`

Review result：`CLEAR`，P0/P1/P2 无；复核线程 `019eb850-0698-7f70-a9b2-e7d0d668ccf5`

Checkpoint commit：`TBD`

## 1. 完成内容

R2-T2 已把第二批低风险 read-model boundary inline tests 从 `lib.rs` 迁到 `lib_read_model_boundary_tests.rs`，并通过 crate-root test include 保持同一测试模块语义。

关键结果：

- `lib.rs` 从 13,438 行降到 12,699 行。
- 新增 `lib_read_model_boundary_tests.rs`，740 行，低于 Rust 3,000 行阈值。
- shape gate 的 `lib.rs` waterline 已更新为 12,699，锁住本次下降收益。
- 没有改产品函数签名、可见性、测试断言含义或真实执行路径。

## 2. 迁移清单

迁出测试：

- `path_whitelist_accepts_only_index_projects_and_rollouts`
- `snapshot_keeps_metadata_without_session_body`
- `g2_diagnostic_summary_reports_degraded_store_without_repair`
- `workbench_snapshot_includes_backend_agent_adapter_descriptor`
- `backend_agent_adapter_descriptor_is_stable_without_codex_signals`
- `session_operation_descriptors_cover_e2_boundary_matrix`
- `provider_availability_summaries_cover_e3_boundary_matrix`
- `session_continuation_guard_covers_e4_boundary_matrix`

保留在 `lib.rs`：

- K3-B runtime prompt guard 测试。
- `reads_real_static_index_summary`。
- R2-T1 的 `include!("lib_transcript_readback_tests.rs");`。

## 3. 验证结果

通过：

- `cargo fmt -- --check`
- `cargo test --lib diagnostic`：4 passed。
- `cargo test --lib adapter`：6 passed。
- `cargo test --lib session_operation`：1 passed。
- `cargo test --lib provider_availability`：1 passed。
- `cargo test --lib session_continuation`：17 passed，4 ignored。
- `cargo test --lib`：471 passed，16 ignored。
- `node scripts/harness/workbench-shape-gate.js --mode check`：0 errors，0 warnings，`lib.rs: 12699/12699 (same)`。
- `git diff --check`

既有 warning：

- `JsonRpcError::invalid_params` never used。

## 4. 边界确认

本轮没有执行真实 `codex exec` / `codex exec resume`，没有发送 prompt，没有读写 `/Users/yoyi/.codex`，没有读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript。

本轮没有启动 Tauri / Browser / Chrome / Vite dev / screenshot。

本轮不接受为 `lib.rs <= 3,000`、R2 全部完成、R3 Level B、生产 SQLite 迁移、read-cut、stop-write、多 agent 并行真实执行解锁、真实 Codex、`.codex` 读写、UI / 产品行为修改或 backlog 功能解冻。

## 5. 复核回收

复核线程：`019eb850-0698-7f70-a9b2-e7d0d668ccf5`

结论：`STATUS: CLEAR`

- P0：无。
- P1：无。
- P2：无。
- 复核确认 R2-T2 只迁移 8 个 read-model boundary tests。
- 复核确认 K3-B guard 和 `reads_real_static_index_summary` 均保留在 `lib.rs`。
- 复核确认新文件没有实际读写 `/Users/yoyi/.codex`、`std::process`、网络、真实 Codex 执行或 Tauri command。
- 复核确认 `lib.rs` waterline 更新到 `12699` 符合 historical-low ratchet。
- 复核残余风险：未重跑主管线 cargo/shape gate 验证，只做静态只读审查。
