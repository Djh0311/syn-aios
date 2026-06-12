# Evidence: Root Treatment / R2-T9 Rust Memory Store Context Test Extraction v1

日期：2026-06-12

状态：已完成本地验证，待复核。

任务包：`tasks/2026-06-12-root-treatment-r2-t9-rust-memory-store-context-test-extraction-v1.md`

Planning baseline commit：`83441187fef4f3b6acd1ae67a17174f28d4b3823`

Task package commit：`d564febc857c4a51c97d819b295ee66a29218858`

Implementation commit：`8776e95ef005a3a6e1e8e8ff2a21357818564817`

## 1. 本轮目标

按 2026-06-12 新策略继续 R2 后段 inline tests 迁移，只抽能降低 `lib.rs` 棘轮指标的低风险测试切片。本轮迁移 memory candidate store、formal memory store 和 formal memory context 相关 Rust inline tests。

## 2. 改动范围

修改：

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `scripts/harness/workbench-shape-gate.js`
- `tasks/2026-06-12-root-treatment-r2-t9-rust-memory-store-context-test-extraction-v1.md`

新增：

- `prototypes/productized-desktop-shell/src-tauri/src/lib_memory_store_context_tests.rs`
- `evidence/2026-06-12-root-treatment-r2-t9-rust-memory-store-context-test-extraction-v1.md`
- `handoffs/2026-06-12-root-treatment-r2-t9-rust-memory-store-context-test-extraction-v1-result.md`

本轮未修改：

- 产品函数签名、可见性或行为。
- memory candidate store、formal memory store、formal memory context binding 或 task memory packet 产品语义。
- memory candidate adoption、formal memory adoption、task package、dispatch readiness、workflow execution runner、workflow machine、K3-B runtime prompt guard、ignored real-state tests、cross-store memory adoption 或共享 stub runner / factory。
- Tauri command、DB schema、sidecar schema、workflow state JSON schema。
- 前端 UI / CSS / TS。

## 3. 实现说明

- 将 9 个任务包允许的 store/context tests 原样迁入 `lib_memory_store_context_tests.rs`。
- 在 `lib.rs` 原位置保留 `include!("lib_memory_store_context_tests.rs");`，测试仍运行在 crate-root `tests` module 内，不扩大生产函数可见性。
- 共享 helper 继续留在 `lib.rs`，包括 `fixture_memory_candidate_input`、`fixture_formal_memory_input`、`fixture_bound_formal_memory_input`、`fixture_bound_memory_scope`、`fixture_bound_memory_candidate_input`、`fixture_project` 等。
- 将 shape gate `lib.rs` waterline 从 `9232` 更新为 `8893`。

## 4. 形状收益

- `lib.rs`：`9232 -> 8893`，下降 `339` 行。
- 新增 `lib_memory_store_context_tests.rs`：`340` 行，低于 `.rs` 新文件上限 `3000`。
- shape gate 输出：`lib.rs: 8893/8893 (same)`，0 errors，0 warnings。

## 5. 验证

已通过：

- `cargo test --lib memory_candidate_store`：1 passed，0 failed。
- `cargo test --lib formal_memory_store`：6 passed，0 failed。
- `cargo test --lib formal_memory_context`：6 passed，0 failed。
- `cargo test --lib task_memory_packet`：10 passed，0 failed。
- `cargo test --lib`：471 passed，16 ignored。
- `cargo fmt -- --check`：通过。
- `node scripts/harness/workbench-shape-gate.js --mode check`：pass，0 errors，0 warnings。
- `git diff --check`：通过。

保留既有 warning：

- `src/mcp/protocol.rs` 中 `JsonRpcError::invalid_params` dead_code warning。本轮未触碰该文件。

## 6. 范围扫描

已扫描：

- `rg -n "#\\[test\\]|fn [a-zA-Z0-9_]+\\(" lib_memory_store_context_tests.rs`：新 include 只包含任务包允许的 9 个 tests。
- `rg -n "memory_candidate_adoption_project_director_low_risk_project_memory|memory_candidate_store_keeps_candidates_out_of_formal_memory|formal_memory_context_keeps_existing_m1_guards|include!(...)"`：store/context tests 已迁入 include；adoption test 仍在 `lib.rs`。
- `rg -n "adoption|workflow_node|workflow_machine|dispatch|codex|resume|runner|K3|real_state|cross_store" lib_memory_store_context_tests.rs`：仅命中 fixture project_root 字符串 `/offline-fixture/projects/codex-workbench`，不是真实执行路径。

## 7. 边界确认

本轮没有：

- 执行真实 `codex exec` / `codex exec resume`。
- 发送 prompt。
- 读写 `/Users/yoyi/.codex`。
- 读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript。
- 启动 Tauri / Browser / Chrome / Vite / 截图工具。
- 修改 UI / CSS / TS。
- 修改 Tauri command、DB/schema、sidecar schema、workflow state JSON schema。

## 8. 不接受为

本轮不接受为：

- `lib.rs <= 3,000`
- R2 全部完成
- R3 Level B 执行
- 生产 SQLite 迁移 / read-cut / stop-write
- 多 agent 并行真实执行解锁
- 真实 Codex 执行
- memory candidate adoption 迁移完成
- formal memory adoption 迁移完成
- memory candidate store、formal memory store 或 formal memory context 产品能力新增或语义变更
- task memory packet 产品能力新增或语义变更
- UI / 产品行为修改
- backlog 功能解冻
