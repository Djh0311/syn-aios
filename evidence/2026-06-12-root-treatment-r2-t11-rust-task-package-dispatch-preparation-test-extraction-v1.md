# Evidence: Root Treatment / R2-T11 Rust Task Package Dispatch Preparation Test Extraction v1

日期：2026-06-12

状态：已完成并复核通过，checkpoint 已同步。

任务包：`tasks/2026-06-12-root-treatment-r2-t11-rust-task-package-dispatch-preparation-test-extraction-v1.md`

Planning baseline commit：`d45fd0adaf437eecfbbb1258ff0f095b502b45f1`

Task package commit：`9e1b6abbf46c7203cb90b906900226d9e3a592fe`

Task package hash backfill commit：`5ea9fba80b609b49fee110e32363e44f137c9d18`

Implementation commit：`6a0640a9088445f196282f8c0b657c4ec079872b`

P2 fix commit：`e07fe3411b1a18e01ffff43c9b9443c39dcbdb9a`

Authority sync commit：`fe7f40b3a94fce0227bcc28c4380942e9ad2aadc`

Review result：`CLEAR`；复核线程 `019ebb31-ccb7-7072-b105-6b80f37b997f`；P0/P1/P2 无。

## 1. 本轮目标

按 2026-06-12 新策略继续 R2 后段 inline tests 迁移，只抽能降低 `lib.rs` 棘轮指标的低风险测试切片。本轮迁移任务包生成、任务记忆注入、字段更新、文件生成、dispatch readiness 和 field correction 相关 Rust inline tests。

## 2. 改动范围

修改：

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `scripts/harness/workbench-shape-gate.js`

新增：

- `prototypes/productized-desktop-shell/src-tauri/src/lib_task_package_dispatch_preparation_tests.rs`
- `evidence/2026-06-12-root-treatment-r2-t11-rust-task-package-dispatch-preparation-test-extraction-v1.md`
- `handoffs/2026-06-12-root-treatment-r2-t11-rust-task-package-dispatch-preparation-test-extraction-v1-result.md`

本轮未修改：

- 产品函数签名、可见性或行为。
- task package / task memory injection / dispatch readiness 产品语义或断言口径。
- helper / fixture builder / stub runner / runner fake。
- K3-B runtime prompt guard、workflow node dispatch execute/readback、workflow execution runner、workflow machine、offline role dispatch、ignored real-state tests、cross-store memory adoption、memory candidate adoption、formal memory adoption、legacy dispatch execution 或真实 execution tests。
- Tauri command、DB schema、sidecar schema、workflow state JSON schema。
- 前端 UI / CSS / TS。

## 3. 实现说明

- 将任务包允许的 32 个 tests 原样迁入 `lib_task_package_dispatch_preparation_tests.rs`。
- 在 `lib.rs` 原位置保留 `include!("lib_task_package_dispatch_preparation_tests.rs");`，测试仍运行在 crate-root `tests` module 内，不扩大生产函数可见性。
- 共享 helper 继续留在 `lib.rs`，包括 `fixture_project`、`fixture_task_draft_request`、`fixture_task_preview_request`、`fixture_fields_update_request`、`ready_fields_update_request`、`fixture_task_file_generation_request`、`fixture_dispatch_readiness_request`、`setup_task_memory_injection_fixture`、`mark_task_package_fixture_ready` 和 `fixture_dispatch_correction_request` 等。
- 将 shape gate `lib.rs` waterline 从 `8045` 更新为 `6544`。

## 4. 形状收益

- `lib.rs`：`8045 -> 6544`，下降 `1501` 行。
- 新增 `lib_task_package_dispatch_preparation_tests.rs`：`1503` 行，低于 `.rs` 新文件上限 `3000`。
- shape gate 输出：`lib.rs: 6544/6544 (same)`，0 errors，0 warnings。
- 字节级对比：从 implementation 前 HEAD 的旧 `lib.rs` 起止测试块提取后，与新 include 文件 `byte_exact_match: True`；旧块 `1503` 行，新 include `1503` 行。

## 5. 验证

已通过：

- `cargo test --lib workflow_run_check`：2 passed，0 failed。
- `cargo test --lib task_package`：29 passed，1 ignored，0 failed；ignored 为既有 `real_task_package_file_generation_confirmation_v1` confirmation 测试，本轮未执行。
- `cargo test --lib task_memory_injection`：5 passed，0 failed。
- `cargo test --lib dispatch_field_correction`：5 passed，0 failed。
- `cargo test --lib`：471 passed，16 ignored，0 failed。
- `cargo fmt -- --check`：通过。
- `node scripts/harness/workbench-shape-gate.js --mode check`：pass，0 errors，0 warnings。
- `git diff --check`：通过。

保留既有 warning：

- `src/mcp/protocol.rs` 中 `JsonRpcError::invalid_params` dead_code warning。本轮未触碰该文件。

## 6. 范围扫描

已扫描：

- `rg -n "workflow_node_dispatch_execute_|workflow_machine|memory_candidate_adoption_|formal_memory_adoption_|k3_b_|stub|runner|real_state|prepare_offline_role_dispatch|record_offline_role" lib_task_package_dispatch_preparation_tests.rs`：无命中。
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

过程备注：

- 一次旧口径扫描命令曾因双引号内 Markdown 反引号触发 shell 命令替换，输出 `zsh: command not found: lib.rs`；该误扫没有执行 Codex、没有触碰 `/Users/yoyi/.codex`，随后已用单引号固定短语重跑扫描。

## 8. 不接受为

本轮不接受为：

- `lib.rs <= 3,000`
- R2 全部完成
- R3 Level B 执行
- 生产 SQLite 迁移 / read-cut / stop-write
- 多 agent 并行真实执行解锁
- 真实 Codex 执行
- task package / task memory injection / dispatch readiness 产品语义变更或新增能力
- workflow node dispatch execute/readback 迁移完成
- workflow execution runner / workflow machine / K3-B guard 迁移完成
- memory candidate adoption 迁移完成
- formal memory adoption 迁移完成
- UI / 产品行为修改
- backlog 功能解冻

## 9. 复核结论

复核线只读审查已通过：

- 复核线程：`019ebb31-ccb7-7072-b105-6b80f37b997f`
- 最终结论：`STATUS: CLEAR`
- P0/P1/P2：无。
- 初审唯一 P2 为新 include EOF 空白；已用 `e07fe3411b1a18e01ffff43c9b9443c39dcbdb9a` 窄修。
- 复核确认 `git show --check e07fe3411b1a18e01ffff43c9b9443c39dcbdb9a` 无 whitespace 报告，`git diff --check 5ea9fba80b609b49fee110e32363e44f137c9d18..HEAD` 无输出。
- 复核确认新 include 仍为 32 个 tests，禁止关键词扫描无命中，shape gate waterline `6544` 与当前 `wc -l lib.rs` 一致。
