# Root Treatment / R2-T2 Rust Inline Read Model Boundary Test Extraction v1

日期：2026-06-12

状态：已完成，待 hash 回填。

Planning baseline commit：`3e65ee7c528c06a78bd7e28849bc5751f266522e`

Implementation commit：`TBD`

Review result：`CLEAR`，P0/P1/P2 无；复核线程继续复用 `019eb850-0698-7f70-a9b2-e7d0d668ccf5`。

Checkpoint commit：`TBD`

本文是 Root Treatment / Stage R 的 R2-T2 任务包，承接 R2-T0 的 `PARTIALLY_UNLOCKED_WITH_GUARDS` 裁决和 R2-T1 的首批 inline tests 迁移结果，继续迁移低风险 Rust inline tests，并把 `lib.rs` 历史最低水位线继续下压。

## 1. 目标

把 `prototypes/productized-desktop-shell/src-tauri/src/lib.rs` 中 diagnostics / adapter / session operation / provider availability / session continuation guard 相关 inline tests 迁出到 crate-root test include 文件，降低 `lib.rs` 棘轮指标。

目标文件：

- 新增：`prototypes/productized-desktop-shell/src-tauri/src/lib_read_model_boundary_tests.rs`
- 修改：`prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- 修改：`scripts/harness/workbench-shape-gate.js`

## 2. 允许范围

允许：

- 迁移以下 `lib.rs` inline tests：
  - `path_whitelist_accepts_only_index_projects_and_rollouts`
  - `snapshot_keeps_metadata_without_session_body`
  - `g2_diagnostic_summary_reports_degraded_store_without_repair`
  - `workbench_snapshot_includes_backend_agent_adapter_descriptor`
  - `backend_agent_adapter_descriptor_is_stable_without_codex_signals`
  - `session_operation_descriptors_cover_e2_boundary_matrix`
  - `provider_availability_summaries_cover_e3_boundary_matrix`
  - `session_continuation_guard_covers_e4_boundary_matrix`
- 在 `#[cfg(test)] mod tests` 内新增 `include!("lib_read_model_boundary_tests.rs");`。
- 保持测试仍在同一个 crate-root `tests` module 中，避免扩大生产函数可见性。
- 更新 shape gate 中 `lib.rs` 的 historical-low ratchet waterline。

禁止：

- 迁移或修改 K3-B runtime prompt guard 测试。
- 迁移 `reads_real_static_index_summary`。
- 迁移 workflow execution runner、workflow machine、ignored real-state tests、cross-store memory adoption 或共享 stub runner / factory。
- 修改产品函数签名、可见性或行为。
- 新增 public API。
- 修改测试断言含义。
- 修改 DB schema、sidecar schema、workflow state JSON、Tauri command、UI、CSS、前端代码或真实执行路径。
- 执行真实 `codex exec` / `codex exec resume`，发送 prompt，读写 `/Users/yoyi/.codex`。
- 读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript。
- 启动 Tauri / Browser / Chrome / Vite dev / screenshot。

## 3. 形状影响

预计影响：

- `lib.rs` 预计下降约 600-750 行。
- 新增测试 include 文件预计 600-750 行，低于 Rust 新文件 3,000 行上限。
- 不新增 Tauri command。
- 不新增 sidecar JSON 种类。
- 不新增生产模块，只新增测试 include 文件。

棘轮目标：

```text
prototypes/productized-desktop-shell/src-tauri/src/lib.rs: 13,438 -> 新历史低点
```

## 4. Implementation Plan

1. 新增 `lib_read_model_boundary_tests.rs`，把目标测试原样移入。
2. 在 `lib.rs` 原位置保留一个 crate-root test include。
3. 更新 `workbench-shape-gate.js` 的 `lib.rs` waterline 到迁移后的实际行数。
4. 运行 focused cargo tests、全量 cargo lib、format check、shape gate 和 diff check。
5. 写 evidence / handoff。
6. 交给既有复核线只读审查。
7. 复核 `CLEAR` 或仅 P2 后提交 implementation commit，再做 checkpoint 和 hash backfill。

## 5. 验收标准

必须满足：

- `lib.rs` 行数下降，且 shape gate 通过。
- 新测试文件低于 Rust 3,000 行阈值。
- focused tests 通过：
  - `cargo test --lib diagnostic`
  - `cargo test --lib adapter`
  - `cargo test --lib session_operation`
  - `cargo test --lib provider_availability`
  - `cargo test --lib session_continuation`
- `cargo test --lib` 通过。
- `cargo fmt -- --check` 通过。
- `node scripts/harness/workbench-shape-gate.js --mode check` 通过。
- `git diff --check` 通过。
- 复核线 `STATUS: CLEAR` 或仅 P2。

## 6. 不接受为

本任务不接受为：

- `lib.rs <= 3,000` 达成。
- R2 全部完成。
- R3 Level B 执行或完成。
- 生产 SQLite 迁移、read-cut、stop-write 或 rollback production workflow。
- 多 agent 并行真实执行解锁。
- 真实 Codex 执行、`.codex` 读写、UI / 产品行为修改或 backlog 功能解冻。

## 7. Review Result

只读复核线 `019eb850-0698-7f70-a9b2-e7d0d668ccf5` 结论：`STATUS: CLEAR`。

复核结论：

- P0：无。
- P1：无。
- P2：无。
- 复核确认 tracked diff 只包含 `lib.rs` 和 `workbench-shape-gate.js`；新增文件只包含 R2-T2 测试文件与 task/evidence/handoff 文档。
- 复核确认 `lib_read_model_boundary_tests.rs` 中 `#[test]` 数量为 8，正好对应任务包允许的 read-model boundary 测试清单。
- 复核确认 K3-B runtime prompt guard 测试和 `reads_real_static_index_summary` 均保留在 `lib.rs`。
- 复核确认新测试文件中的 `/Users/yoyi/.codex/...` 只作为 JSON/string fixture 和 `can_copy` 断言输入出现，未发现实际读写 `/Users/yoyi/.codex`、`std::process`、网络、真实 Codex 执行或 Tauri command。
- 复核确认 `lib.rs` waterline 更新到 `12699` 符合 historical-low ratchet 收口。
