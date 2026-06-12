# Evidence: Root Treatment / R2-T8 Rust Memory Entity Relation Test Extraction v1

日期：2026-06-12

状态：已完成本地验证，复核待回收。

任务包：`tasks/2026-06-12-root-treatment-r2-t8-rust-memory-entity-relation-test-extraction-v1.md`

Planning baseline commit：`515eca4abae963eeb94cc898375e956be448ef41`

Task package commit：`7d95a4f09e9fa01454bbba87ec579130e6bba33e`

Implementation commit：`677833f43321723a92e919d81bc49e32aa3cd9fc`

## 1. 本轮目标

按 2026-06-12 新策略继续 R2 后段 inline tests 迁移，只抽能降低 `lib.rs` 棘轮指标的低风险测试切片。本轮迁移 memory entity relation 相关 Rust inline tests。

## 2. 改动范围

修改：

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `scripts/harness/workbench-shape-gate.js`
- `tasks/2026-06-12-root-treatment-r2-t8-rust-memory-entity-relation-test-extraction-v1.md`

新增：

- `prototypes/productized-desktop-shell/src-tauri/src/lib_memory_entity_relation_tests.rs`
- `evidence/2026-06-12-root-treatment-r2-t8-rust-memory-entity-relation-test-extraction-v1.md`
- `handoffs/2026-06-12-root-treatment-r2-t8-rust-memory-entity-relation-test-extraction-v1-result.md`

本轮未修改：

- 产品函数签名、可见性或行为。
- memory entity relation 产品语义、task memory packet 召回语义或正式记忆采纳语义。
- formal memory adoption、memory candidate adoption、memory candidate store、formal memory store、task package、dispatch readiness、workflow execution runner、workflow machine、K3-B runtime prompt guard、ignored real-state tests、cross-store memory adoption 或共享 stub runner / factory。
- Tauri command、DB schema、sidecar schema、workflow state JSON schema。
- 前端 UI / CSS / TS。

## 3. 实现说明

- 将 5 个 `memory_entity_relation_*` tests 原样迁入 `lib_memory_entity_relation_tests.rs`。
- 在 `lib.rs` 原位置保留 `include!("lib_memory_entity_relation_tests.rs");`，测试仍运行在 crate-root `tests` module 内，不扩大生产函数可见性。
- 共享 helper 继续留在 `lib.rs`，包括 `fixture_m10_preview_input`、`fixture_m10_memory_source`、`create_formal_memory_for_task`、`mutate_formal_store` 和 `fixture_task_memory_packet_input` 等。
- 将 shape gate `lib.rs` waterline 从 `9610` 更新为 `9232`。

## 4. 形状收益

- `lib.rs`：`9610 -> 9232`，下降 `378` 行。
- 新增 `lib_memory_entity_relation_tests.rs`：`379` 行，低于 `.rs` 新文件上限 `3000`。
- shape gate 输出：`lib.rs: 9232/9232 (same)`，0 errors，0 warnings。

## 5. 验证

已通过：

- `cargo test --lib memory_entity_relation`：5 passed，0 failed。
- `cargo test --lib task_memory_packet`：10 passed，0 failed。
- `cargo test --lib`：471 passed，16 ignored。
- `cargo fmt -- --check`：通过。
- `node scripts/harness/workbench-shape-gate.js --mode check`：pass，0 errors，0 warnings。
- `git diff --check`：通过。

保留既有 warning：

- `src/mcp/protocol.rs` 中 `JsonRpcError::invalid_params` dead_code warning。本轮未触碰该文件。

## 6. 边界确认

本轮没有：

- 执行真实 `codex exec` / `codex exec resume`。
- 发送 prompt。
- 读写 `/Users/yoyi/.codex`。
- 读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript。
- 启动 Tauri / Browser / Chrome / Vite / 截图工具。
- 修改 UI / CSS / TS。
- 修改 Tauri command、DB/schema、sidecar schema、workflow state JSON schema。

关键词扫描待复核线补充只读确认；主管线本轮改动限定在新增 include、`lib.rs` include 位置和 shape gate waterline。

## 7. 复核状态

复核线待回收。

## 8. 不接受为

本轮不接受为：

- `lib.rs <= 3,000`
- R2 全部完成
- R3 Level B 执行
- 生产 SQLite 迁移 / read-cut / stop-write
- 多 agent 并行真实执行解锁
- 真实 Codex 执行
- memory entity relation 产品能力新增或语义变更
- task memory packet 产品能力新增或语义变更
- formal memory adoption 迁移完成
- UI / 产品行为修改
- backlog 功能解冻
