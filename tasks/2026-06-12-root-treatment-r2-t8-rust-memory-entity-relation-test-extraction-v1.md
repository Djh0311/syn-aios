# Root Treatment / R2-T8 Rust Memory Entity Relation Test Extraction v1

日期：2026-06-12

状态：待执行。

Planning baseline commit：`515eca4abae963eeb94cc898375e956be448ef41`

本文是 Root Treatment / Stage R 的 R2-T8 任务包，承接 R2-T7 和 2026-06-12 新策略，只迁移能实际降低 `lib.rs` 棘轮指标的低风险 Rust inline tests。

## 1. 目标

把 `prototypes/productized-desktop-shell/src-tauri/src/lib.rs` 中 memory entity relation 相关 inline tests 迁出到 crate-root test include 文件，继续降低 `lib.rs` 历史最低水位线。

目标文件：

- 新增：`prototypes/productized-desktop-shell/src-tauri/src/lib_memory_entity_relation_tests.rs`
- 修改：`prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- 修改：`scripts/harness/workbench-shape-gate.js`

预计收益：

- `lib.rs` 预计下降约 350-380 行。
- 新增 `.rs` test include 预计低于 3,000 行。
- 该切片降低 `lib.rs` 棘轮指标，符合新策略“不得立项不降低棘轮指标的拆分包”。

## 2. 允许范围

允许迁移以下 tests，保持测试体和断言语义不变：

- `memory_entity_relation_preview_suggests_alias_and_similarity_candidates_readonly`
- `memory_entity_relation_llm_inferred_causal_relation_stays_candidate`
- `memory_entity_relation_confirmed_causal_relation_explains_task_packet`
- `memory_entity_relation_secret_relation_source_is_not_exported_to_task_packet`
- `memory_entity_relation_damaged_json_and_revision_conflict_are_rejected`

允许：

- 在 `lib.rs` 原位置插入 `include!("lib_memory_entity_relation_tests.rs");`
- 让共享 helper 继续留在 `lib.rs`，包括 `fixture_m10_preview_input`、`fixture_m10_memory_source`、`create_formal_memory_for_task`、`mutate_formal_store`、`fixture_task_memory_packet_input` 等。
- 更新 shape gate 中 `lib.rs` waterline 为本轮完成后的历史最低收口值。
- 新增 evidence / handoff。

## 3. 禁止范围

禁止：

- 修改产品函数签名、可见性、语义或断言口径。
- 迁移 formal memory adoption、memory candidate adoption、memory candidate store、formal memory store、task package、dispatch readiness、workflow execution runner、workflow machine、K3-B runtime prompt guard、ignored real-state tests、cross-store memory adoption 或共享 stub runner / factory。
- 修改 memory entity relation 产品语义、task memory packet 召回语义或正式记忆采纳语义。
- 修改 Tauri command、DB schema、sidecar schema、workflow state JSON schema。
- 修改前端 UI / CSS / TS。
- 执行真实 `codex exec` / `codex exec resume`、发送 prompt、读写 `/Users/yoyi/.codex`。
- 启动 Tauri / Browser / Chrome / Vite / 截图工具。

## 4. 验收

必须通过：

- `cargo test --lib memory_entity_relation`
- `cargo test --lib task_memory_packet`
- `cargo test --lib`
- `cargo fmt -- --check`
- `node scripts/harness/workbench-shape-gate.js --mode check`
- `git diff --check`
- 复核线只读审查，结论不得有 P0/P1。

## 5. 不接受为

本任务不接受为：

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
