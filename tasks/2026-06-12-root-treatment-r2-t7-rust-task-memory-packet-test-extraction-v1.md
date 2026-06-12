# Root Treatment / R2-T7 Rust Task Memory Packet Test Extraction v1

日期：2026-06-12

状态：待执行。

Planning baseline commit：`d7d9d3520495425f1c0e8ce5ce3b681970b360be`

本文是 Root Treatment / Stage R 的 R2-T7 任务包，承接 R4-A50 新策略和 R2-T6 inline tests 迁移结果，继续迁移能实际降低 `lib.rs` 棘轮指标的低风险 Rust inline tests。

## 1. 目标

把 `prototypes/productized-desktop-shell/src-tauri/src/lib.rs` 中 task memory packet preview 相关 inline tests 迁出到 crate-root test include 文件，继续降低 `lib.rs` 历史最低水位线。

目标文件：

- 新增：`prototypes/productized-desktop-shell/src-tauri/src/lib_task_memory_packet_tests.rs`
- 修改：`prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- 修改：`scripts/harness/workbench-shape-gate.js`

预计收益：

- `lib.rs` 预计下降约 360 行。
- 新增 `.rs` test include 预计低于 3,000 行。
- 该切片降低 `lib.rs` 棘轮指标，符合新策略“不得立项不降低棘轮指标的拆分包”。

## 2. 允许范围

允许迁移以下 tests，保持测试体和断言语义不变：

- `task_memory_packet_includes_active_formal_memory`
- `task_memory_packet_excludes_candidates_as_unconfirmed`
- `task_memory_packet_excludes_observation_as_not_formal`
- `task_memory_packet_excludes_inactive_formal_memories`
- `task_memory_packet_excludes_model_export_blocked`
- `task_memory_packet_excludes_permission_blocked`
- `task_memory_packet_excludes_token_limit`
- `task_memory_packet_excludes_not_relevant`
- `task_memory_packet_preview_is_readonly`
- `task_memory_packet_preview_does_not_execute_worker`

允许：

- 在 `lib.rs` 原位置插入 `include!("lib_task_memory_packet_tests.rs");`
- 让 task memory packet 共享 helper 继续留在 `lib.rs`，因为后续 memory entity / lint / mature pattern 等 tests 仍复用 `create_formal_memory_for_task`、`mutate_formal_store`、`excluded_reason_count` 等 helper。
- 更新 shape gate 中 `lib.rs` waterline 为本轮完成后的历史最低收口值。
- 新增 evidence / handoff。

## 3. 禁止范围

禁止：

- 修改产品函数签名、可见性、语义或断言口径。
- 迁移 formal memory adoption、memory lint、memory entity relation、mature pattern、K3-B runtime prompt guard、workflow execution runner、workflow machine、ignored real-state tests、cross-store memory adoption 或共享 stub runner / factory。
- 修改 task memory packet helper 语义或把共享 helper 移出 `lib.rs`。
- 修改 Tauri command、DB schema、sidecar schema、workflow state JSON schema。
- 修改前端 UI / CSS / TS。
- 执行真实 `codex exec` / `codex exec resume`、发送 prompt、读写 `/Users/yoyi/.codex`。
- 启动 Tauri / Browser / Chrome / Vite / 截图工具。

## 4. 验收

必须通过：

- `cargo test --lib task_memory_packet`
- `cargo test --lib observation`
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
- task memory packet 产品能力新增或语义变更
- formal memory adoption 迁移完成
- UI / 产品行为修改
- backlog 功能解冻

## 6. 执行记录

待填写。
