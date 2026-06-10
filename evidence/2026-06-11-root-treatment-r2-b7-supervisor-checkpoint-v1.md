# Root Treatment R2-B7 Supervisor Checkpoint v1

日期：2026-06-11

## 结论

R2-B7 已由主管线回收为 `accepted_with_p2`。

接受范围：

- 任务包点名的 memory command bridge、observation bridge、task memory packet preview bridge 和上下文绑定 guard 已从 `src-tauri/src/lib.rs` 抽出到 `src-tauri/src/memory_context_entrypoints.rs`。
- `lib.rs` 通过 `include!("memory_context_entrypoints.rs")` 在 crate root 展开 helper，保持函数可见性和行为不变。
- `lib.rs` 从 19,401 行降到 18,932 行，低于 R0 水位线。
- 新 helper 文件为 475 行，低于 Rust 3,000 行治理阈值。
- command registry 总量保持 96，`lib.rs` 内 `#[tauri::command]` 保持 0。

不接受范围：

- 不接受为 R2 全部完成。
- 不接受为 `lib.rs <= 15,000` 或 `lib.rs <= 3,000` 目标完成。
- 不接受为 memory store 内部实现重构、memory lifecycle 重构、runtime diagnostics、provider adapter、tests 巨石、SQLite 迁移或 R3 完成。
- 不接受为 Stage L / K3-B1 / K3-B2 恢复。
- 不接受为新的真实 Codex 执行授权。

## Commit 记录

- R2-B7 start commit：`174338153663d1d906933c6893165113f7edb376`
- R2-B7 completion commit：`9cd10bb51fe828ae5b2b72501414b5cf025b77a9`
- 本 supervisor checkpoint 提交：待本 checkpoint 提交后回填

## 复核文件

- `tasks/2026-06-11-root-treatment-r2-b7-memory-command-bridge-and-context-guard-extraction-v1.md`
- `evidence/2026-06-11-root-treatment-r2-b7-memory-command-bridge-and-context-guard-extraction-v1.md`
- `handoffs/2026-06-11-root-treatment-r2-b7-memory-command-bridge-and-context-guard-extraction-v1-result.md`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_context_entrypoints.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`

## 主管 Fresh Verify

已重新运行：

```bash
node scripts/harness/workbench-shape-gate.js --mode baseline
node scripts/harness/workbench-shape-gate.js --mode check
cargo test --lib formal_memory
cargo test --lib memory_candidate
cargo test --lib memory_lint
cargo test --lib observation
cargo test --lib task_memory
cargo test --lib memory_entity_relation
cargo test --lib
cargo fmt -- --check
git diff --check
git status --short
```

结果：

- shape gate baseline：通过，0 errors / 0 warnings；`lib.rs` 18,932 lines；Tauri commands 96 total / 0 in `lib.rs`；Sidecar JSON kinds 14 detected / 0 unknown。
- shape gate check：通过，0 errors / 0 warnings；`lib.rs` ratchet `18,932 / 25,925`，status `decreased`。
- `cargo test --lib formal_memory`：通过，29 passed / 0 failed；保留既有 `JsonRpcError::invalid_params` dead_code warning。
- `cargo test --lib memory_candidate`：通过，9 passed / 0 failed；保留同一既有 warning。
- `cargo test --lib memory_lint`：通过，9 passed / 0 failed；保留同一既有 warning。
- `cargo test --lib observation`：通过，15 passed / 0 failed；保留同一既有 warning。
- `cargo test --lib task_memory`：通过，15 passed / 0 failed；保留同一既有 warning。
- `cargo test --lib memory_entity_relation`：通过，5 passed / 0 failed；保留同一既有 warning。
- `cargo test --lib`：通过，336 passed / 0 failed / 16 ignored；保留同一既有 warning。
- `cargo fmt -- --check`：通过。
- `git diff --check`：通过，无输出。
- `git status --short`：通过，无输出。

## 主管边界复核

已核对：

- R2-B7 completion commit 只包含 4 个文件：`lib.rs`、`memory_context_entrypoints.rs`、R2-B7 evidence、R2-B7 handoff。
- 抽出范围从 `create_formal_memory_record_at` 到 `validate_formal_memory_project_registered`，共 16 个函数。
- `option_trimmed_is_empty` 仍留在 `lib.rs`，没有把后续 shared workflow utilities、blackboard、runtime diagnostics、provider / adapter、SQLite、UI 或 tests 巨石带入本批次。
- helper 内没有新增 `#[tauri::command]`、sidecar store、SQLite schema、UI、planned adapter、provider credential 读取、真实 Codex 调用或 `.codex` 访问。

## P0 / P1 / P2

- P0：无。
- P1：无。
- P2：R2-B7 仍使用 `include!` 作为保守过渡，后续 R2 可以继续收敛为正式模块边界。
- P2：R2-B7 只抽出 memory command bridge / observation bridge / task memory packet preview bridge / context binding guard，不代表 memory store 内部、runtime diagnostics、provider adapter、storage migration、UI、tests 巨石拆分或 R2 水位线目标完成。
- P2：相关测试仍主要保留在 `lib.rs` inline tests 中，后续 R2 后段可按领域迁移 tests。

## 边界确认

- 未执行真实 `codex exec` / `codex exec resume`。
- 未发送 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未读取 secret、token、`.env`、keychain、OAuth、provider credential、完整 transcript 或 rollout。
- 未启动 Tauri / Browser / Chrome / Vite / 截图工具。
- 未新增 sidecar store 或 sidecar JSON 种类。
- 未迁移 SQLite。
- 未改 workflow state 顶层 schema。
- 未新增 Tauri command。
- 未改真实 Codex runner。
- 未启动 Stage L / K3-B1 retry / K3-B2。
- 未解冻 backlog 功能。

## 下一步

继续 R2 小批次治理。下一步应先创建 R2-B8 任务包，建议限定为 diagnostics / provider availability / session continuation / adapter descriptors 等剩余 `lib.rs` 边界的行为不变物理抽出；不顺手做 R3 SQLite、R4 UI/按页读模型、Stage L/K3 恢复或真实 Codex 执行。
