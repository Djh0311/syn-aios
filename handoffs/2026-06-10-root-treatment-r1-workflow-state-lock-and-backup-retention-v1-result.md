# Root Treatment R1 Workflow State Lock And Backup Retention v1 Result

日期：2026-06-10

## 结论

R1 本轮可接受为：`workflow-state.v0.json` 最终写入 / rename 已有文件级 StoreLock，lock busy / corrupt JSON / revision conflict 均有“不覆盖原文件”测试，backup retention 策略已在测试夹具中验证。

不接受为：完整 read-modify-write 串行化、真实 backups 已清理、R3 SQLite 迁移、workflow state schema 迁移、R2 `lib.rs` 解体、Stage L / K3-B1 / K3-B2 恢复。

## 已完成

- 修改 `prototypes/productized-desktop-shell/src-tauri/src/workflow_state_store.rs`。
- 新增 `.workflow-state.v0.lock` 文件级 lock。
- `atomic_write` 在最终 temp 写入 / `sync_all` / rename 前 acquire lock。
- `backup_file` 在 copy / prune 前 acquire lock。
- `atomic_write` / `write_validated` 覆盖旧文件前先确认已有 JSON 可解析。
- 新增 backup retention：最近 30 份 + 每日 1 份。
- 新增 Rust 单测覆盖 lock busy、corrupt JSON、revision conflict、retention prune。
- 新增 R1 evidence：`evidence/2026-06-10-root-treatment-r1-workflow-state-lock-and-backup-retention-v1.md`。

## 写路径审计

- 未发现覆盖全部 workflow state 写路径的上层 `Mutex` / `RwLock`。
- `commands.rs` 只传 `state.workflow_state_path`。
- 主写路径经 `lib.rs` 的 `write_validated_workflow_state(...)` 进入 `workflow_state_store::write_validated(...)`。
- 最终原子替换经 `atomic_write_json(...)` 进入 `workflow_state_store::atomic_write(...)`。
- 初始化 / bootstrap 等少数历史路径仍有 `lib.rs` 内手写 backup；本轮不改 `lib.rs`，只通过底层 `atomic_write` 覆盖最终写入 lock 和 corrupt guard。

## StoreLock 边界

lock path：

```text
<workflow-state-dir>/.workflow-state.v0.lock
```

覆盖：

- 最终 JSON 写入临时文件、`sync_all`、rename。
- 通过 `backup_file(...)` 的 backup copy 和 retention prune。

不覆盖 / P2：

- 不覆盖完整 read-modify-write 窗口。
- `backup_file` 与后续 write 之间不是单一事务。
- 不覆盖 `lib.rs` 内手写 backup 段的 prune。
- 跨 store / 跨文件事务仍待 R3 SQLite。

## Backup Retention

策略：

- 识别 `workflow-state.v0.<timestamp>.json`。
- 保留最近 30 份。
- 每日额外保留 1 份。
- ISO-like timestamp 和毫秒 timestamp 都可分日。
- 非匹配文件不参与 prune。

真实 backups：

- 未触碰真实历史 backups。
- 只在 Rust 测试临时目录中执行 prune。

## Shape 指标

- `workflow_state_store.rs`：91 lines -> 416 lines。
- `lib.rs`：25,925 lines -> 25,925 lines。
- 新增 Tauri command：0。
- 新增 sidecar JSON kind：0。
- UI / TS / TSX：未改。

## 验证

已通过：

- `cargo test --lib workflow_state`：11 passed / 0 failed / 341 filtered out。
- `cargo test --lib`：336 passed / 0 failed / 16 ignored。
- `cargo fmt -- --check`：通过。
- `git diff --check`：通过。
- `node scripts/harness/workbench-shape-gate.js --mode check`：通过，0 errors / 0 warnings。
- `git status --short`：提交前仅应包含 R1 范围文件。

已知 warning：

- Rust 测试存在既有 dead_code warning：`JsonRpcError::invalid_params` 未使用。

## Commit 记录

- R-Preflight baseline：`ed01c6f281e3fd7a38548da948046e8366cc368d`
- R0 completion / R1 start：`7563e6a9d11a92217e1baf34ed71b70722bbc17c`
- R1 completion commit：`7a1ac89173306b50868064b64fb852f57c0550af`。

## 边界

- 未同步入口文档。
- 未修改 R0 文件。
- 未迁移 SQLite。
- 未改 workflow state 顶层 schema。
- 未新增 sidecar / command / UI。
- 未执行真实 Codex。
- 未发送 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未启动 Tauri / Browser / Chrome / Vite / 截图工具。
- 未清理真实历史 backups。

## P0 / P1 / P2

- P0：无。
- P1：无。
- P2：StoreLock 只覆盖文件级最终写入 / rename，不覆盖完整 RMW 串行化。
- P2：retention 挂在 `backup_file(...)` helper；历史手写 backup prune 仍需后续收敛。
- P2：真实历史 backup 清理未执行，若需要必须另起 dry-run + 用户确认任务。
