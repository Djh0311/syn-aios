# Evidence: Root Treatment / R2-T10 Rust Stage C Governance Test Extraction v1

日期：2026-06-12

状态：已完成并复核通过，checkpoint 待同步。

任务包：`tasks/2026-06-12-root-treatment-r2-t10-rust-stage-c-governance-test-extraction-v1.md`

Planning baseline commit：`bcf17fa72928a4f772022f67194bb67f2d2f08bc`

Task package commit：`a75ceeefb1cd122e1b65232955aec60e6ba675e5`

Implementation commit：`6fd18a5a7c701e7bfc6aaaa9a970241a6cba250e`

Review result：`CLEAR`；复核线程 `019ebb31-ccb7-7072-b105-6b80f37b997f`；P0/P1/P2 无。

## 1. 本轮目标

按 2026-06-12 新策略继续 R2 后段 inline tests 迁移，只抽能降低 `lib.rs` 棘轮指标的低风险测试切片。本轮迁移 Stage C / C4-C6 governance 相关 Rust inline tests。

## 2. 改动范围

修改：

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `scripts/harness/workbench-shape-gate.js`
- `tasks/2026-06-12-root-treatment-r2-t10-rust-stage-c-governance-test-extraction-v1.md`

新增：

- `prototypes/productized-desktop-shell/src-tauri/src/lib_stage_c_governance_tests.rs`
- `evidence/2026-06-12-root-treatment-r2-t10-rust-stage-c-governance-test-extraction-v1.md`
- `handoffs/2026-06-12-root-treatment-r2-t10-rust-stage-c-governance-test-extraction-v1-result.md`

本轮未修改：

- 产品函数签名、可见性或行为。
- Stage C / C4-C6 产品语义或断言口径。
- helper / fixture builder / stub runner / runner fake。
- memory candidate adoption、formal memory adoption、workflow execution runner、workflow machine、K3-B runtime prompt guard、ignored real-state tests、cross-store memory adoption、legacy dispatch execution 或真实 execution tests。
- Tauri command、DB schema、sidecar schema、workflow state JSON schema。
- 前端 UI / CSS / TS。

## 3. 实现说明

- 将 15 个任务包允许的 Stage C governance tests 原样迁入 `lib_stage_c_governance_tests.rs`。
- 在 `lib.rs` 原位置保留 `include!("lib_stage_c_governance_tests.rs");`，测试仍运行在 crate-root `tests` module 内，不扩大生产函数可见性。
- 共享 helper 继续留在 `lib.rs`，包括 `create_active_project_director_authorization_fixture`、`setup_c5_worker_report_fixture`、`setup_c6_complete_fixture`、`fixture_*` helper 等。
- 将 shape gate `lib.rs` waterline 从 `8893` 更新为 `8045`。

## 4. 形状收益

- `lib.rs`：`8893 -> 8045`，下降 `848` 行。
- 新增 `lib_stage_c_governance_tests.rs`：`849` 行，低于 `.rs` 新文件上限 `3000`。
- shape gate 输出：`lib.rs: 8045/8045 (same)`，0 errors，0 warnings。

## 5. 验证

已通过：

- `cargo test --lib project_director_task_plan`：3 passed，0 failed。
- `cargo test --lib authorized_prepared_dispatch`：2 passed，0 failed。
- `cargo test --lib worker_structured_report`：2 passed，0 failed。
- `cargo test --lib process_fact`：3 passed，0 failed。
- `cargo test --lib global_final_result_review`：3 passed，0 failed。
- `cargo test --lib user_result_decision`：1 passed，0 failed。
- `cargo test --lib stage_c_acceptance_summary`：1 passed，0 failed。
- `cargo test --lib`：471 passed，16 ignored。
- `cargo fmt -- --check`：通过。
- `node scripts/harness/workbench-shape-gate.js --mode check`：pass，0 errors，0 warnings。
- `git diff --check`：通过。

保留既有 warning：

- `src/mcp/protocol.rs` 中 `JsonRpcError::invalid_params` dead_code warning。本轮未触碰该文件。

## 6. 范围扫描

已扫描：

- `rg -n "#\\[test\\]|fn [a-zA-Z0-9_]+\\(|workflow_machine|memory_candidate_adoption|formal_memory_adoption|K3|real_state|stub|runner" lib_stage_c_governance_tests.rs`：新 include 只包含任务包允许的 15 个 tests；禁止关键词无命中。
- 新 include 没有迁移 helper / fixture builder；helper 仍留在 `lib.rs`。

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
- C4-C6 产品语义变更或新增能力
- memory candidate adoption 迁移完成
- formal memory adoption 迁移完成
- workflow execution runner / workflow machine / K3-B guard 迁移完成
- UI / 产品行为修改
- backlog 功能解冻

## 9. 复核结论

复核线只读审查已通过：

- 复核线程：`019ebb31-ccb7-7072-b105-6b80f37b997f`
- 最终结论：`STATUS: CLEAR`
- P0/P1/P2：无。
- 复核确认新 include 只包含任务包允许的 15 个 Stage C governance tests；旧 `lib.rs` 被删测试块与新 include 文件内容一致；helper 仍留在 `lib.rs`；禁止迁移的 workflow machine、runner/stub、K3、real-state、memory/formal/cross-store adoption tests 没有迁入新 include。
- 复核确认 shape gate waterline `8045` 与当前 `wc -l lib.rs` 一致，`git diff --check a75ceeef..6fd18a5` 与当前 `git diff --check` 均无输出。
