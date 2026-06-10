# Root Treatment R2-B2 Lib Map And Workflow State Helper Extraction v1 Result

日期：2026-06-11

## 结论

R2-B2 本轮可接受为：R2 `lib.rs` 代码地图已补齐，任务包点名的 workflow state JSON helper 已从 `lib.rs` 物理抽出到 `workflow_state_json_helpers.rs`，并通过必需验证。行为保持不变，command 总量仍为 96，`lib.rs` 内 `#[tauri::command]` 仍为 0。

不接受为：R2 全部完成、`lib.rs <= 15,000` 水位线完成、workflow read model / memory / runtime diagnostics 拆分完成、SQLite 迁移完成、workflow state schema 迁移或真实 Codex 执行恢复。

## 已完成

- 新增 `docs/plans/2026-06-11-root-treatment-r2-lib-rs-code-map-v1.md`。
- 新增 `prototypes/productized-desktop-shell/src-tauri/src/workflow_state_json_helpers.rs`。
- 将 15 个 workflow state JSON helper 从 `lib.rs` 搬到新 helper 文件。
- `lib.rs` 原位置改为 `include!("workflow_state_json_helpers.rs")`。
- 新增 R2-B2 evidence：`evidence/2026-06-11-root-treatment-r2-b2-lib-map-and-workflow-state-helper-extraction-v1.md`。
- 未同步入口文档。

## Shape 指标

- `lib.rs`：25,829 lines -> 25,643 lines。
- `workflow_state_json_helpers.rs`：新增 190 lines。
- Tauri command registry：96 total -> 96 total。
- `lib.rs` 内 `#[tauri::command]`：0 -> 0。
- Sidecar JSON kinds：14 detected / 0 unknown。

## 代码地图摘要

代码地图记录了 R2-B2 后 `lib.rs` 的主要领域块：crate 装配、index / transcript、workflow state 生命周期、task package、workflow run / dispatch、C4-C6、offline role、workflow machine、memory guard、workflow read model / dispatch summary、snapshot assembly、diagnostics、provider / continuation / adapter、session/index parsing、allowed paths、host OS helper、Tauri app assembly 和 inline tests。

后续批次建议：先拆 workflow state 生命周期和 task package，再拆 workflow read model、dispatch / workflow machine、memory context guard、diagnostics / continuation / adapter、index parsing / app assembly，最后迁移 tests。

## 验证

已通过：

- `node scripts/harness/workbench-shape-gate.js --mode baseline`：pass，0 errors / 0 warnings；`lib.rs` 25,643 / 25,925，status `decreased`；Tauri commands 96 total / 0 in `lib.rs`。
- `node scripts/harness/workbench-shape-gate.js --mode check`：pass，0 errors / 0 warnings；`lib.rs` 25,643 / 25,925，status `decreased`；Tauri commands 96 total / 0 in `lib.rs`。
- `cargo test --lib workflow_state`：11 passed / 0 failed / 341 filtered out。
- `cargo test --lib`：336 passed / 0 failed / 16 ignored。
- `cargo fmt -- --check`：通过。
- `git diff --check`：通过。
- `git status --short`：提交前仅包含 R2-B2 范围文件。

已知 warning：

- Rust 测试仍有既有 dead_code warning：`JsonRpcError::invalid_params` 未使用。

## Commit 记录

- Start commit：`d737c78eb9e9ce1e1f8e620390d595c498a70e0f`
- End commit：本 R2-B2 提交承载；提交后真实 hash 由回交记录。

## 边界

- 未同步 `CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md`。
- 未改产品业务逻辑。
- 未改函数语义、返回值、错误文案、公开 Tauri command 契约或 workflow state schema。
- 未新增 Tauri command。
- 未新增 sidecar / sidecar JSON kind。
- 未迁移 SQLite。
- 未做 UI。
- 未做 workflow read model / memory / runtime diagnostics 其他拆分。
- 未执行真实 Codex。
- 未发送 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未读取 secret、token、`.env`、keychain、OAuth、provider credential、完整 transcript 或 rollout。
- 未启动 Tauri / Browser / Chrome / Vite / 截图工具。

## P0 / P1 / P2

- P0：无。
- P1：无。
- P2：`include!` 是保守过渡，后续 R2 可再收敛为正式模块边界。
- P2：代码地图是人工静态地图，后续行号需随治理批次更新或脚本化。
- P2：本轮只抽出 workflow state JSON helper，不代表 R2 水位线、workflow read model、storage migration 或 diagnostics / memory 拆分完成。
