# Root Treatment R1 Workflow State Lock And Backup Retention v1

日期：2026-06-10

## 结论

R1 本轮完成 `workflow-state.v0.json` 当前 JSON 事实层的立即止血：

- 在 `workflow_state_store.rs` 内新增文件级 `StoreLock`。
- `atomic_write` 在 temp 写入和 rename 前 acquire 稳定 lock，lock busy 返回明确 `workflow_state_store_locked` 错误，且不会覆盖原文件。
- `backup_file` 在备份和 retention prune 前 acquire 同一 lock。
- `atomic_write` / `write_validated` 在覆盖已有状态前先确认旧 JSON 可解析，corrupt JSON 会拒绝写入并保留原文件。
- 新增 backup retention 策略：默认保留最近 30 份，同时按日保留每日 1 份；测试夹具验证 prune。
- 未迁移 SQLite，未改 workflow state 顶层 schema，未新增 sidecar / Tauri command / UI。
- 未对真实历史 backups 做不可逆清理；retention prune 只在 Rust 测试临时目录夹具中实际执行过。

R1 可接受为：workflow state 最终写入 / rename 已有文件级止血，backup retention 策略已有测试夹具覆盖。

R1 不接受为：完整 read-modify-write 事务串行化、R3 SQLite 迁移、真实历史 backups 清理、workflow state schema 迁移、R2 `lib.rs` 解体、R0 完成、Stage L / K3-B1 / K3-B2 恢复。

## Commit 记录

- R-Preflight baseline commit：`ed01c6f281e3fd7a38548da948046e8366cc368d`
- R0 completion commit / R1 本轮实际 start commit：`7563e6a9d11a92217e1baf34ed71b70722bbc17c`
- R1 completion commit：`7a1ac89173306b50868064b64fb852f57c0550af`。

## 改动文件

- `prototypes/productized-desktop-shell/src-tauri/src/workflow_state_store.rs`
- `evidence/2026-06-10-root-treatment-r1-workflow-state-lock-and-backup-retention-v1.md`
- `handoffs/2026-06-10-root-treatment-r1-workflow-state-lock-and-backup-retention-v1-result.md`

未同步：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`

原因：主管追加边界要求入口文档由主管线统一同步。

## 写路径审计

`workflow_state_store.rs` 当前职责：

- `read_value(path)`：读取并解析 `workflow-state.v0.json`。
- `validate_value(value, ...)`：校验 v0 JSON 基本形状，不改 schema。
- `write_validated(path, value, validate_workflow_state, atomic_write_json)`：写入前校验待写 value；本轮新增已有文件 JSON 可解析检查。
- `backup_file(path, timestamp)`：创建 `backups/workflow-state.v0.<timestamp>.json`；本轮新增 lock 和 retention prune。
- `atomic_write(path, value, timestamp)`：保持 temp + `sync_all` + rename 原子替换语义；本轮新增 lock 和旧 JSON 可解析检查。

上层调用审计：

- `lib.rs` 的 `write_validated_workflow_state(...)` 统一调用 `workflow_state_store::write_validated(...)`。
- `lib.rs` 的 `atomic_write_json(...)` 统一调用 `workflow_state_store::atomic_write(...)`。
- 多数 mutation 路径先读 `workflow-state.v0.json`，按业务规则 / revision guard / schema guard 修改 value，再调用 `write_validated_workflow_state(...)`。
- `commands.rs` 只把 `state.workflow_state_path` 传入后端 helper；`AppState` 仅持有 `PathBuf`，没有发现覆盖全部 workflow state 写路径的上层 `Mutex` / `RwLock`。
- `lib.rs` 仍存在初始化 / bootstrap 等历史手写 backup 段和直接 `atomic_write_json(...)` 调用；本轮通过 `atomic_write` 覆盖最终 rename 锁和 corrupt guard，但不扩大修改 `lib.rs`。

## StoreLock 边界

新增稳定 lock path：

```text
<workflow-state-dir>/.workflow-state.v0.lock
```

实现行为：

- 使用 `OpenOptions::create_new(true)` acquire。
- lock 文件写入 `write_id`，用于区分 `write:<timestamp>` / `backup:<timestamp>`。
- `Drop` 中删除 lock 文件。
- lock busy 返回：

```text
workflow_state_store_locked: <lock-path>
```

覆盖范围：

- `atomic_write`：覆盖最终 JSON 序列化、临时文件写入、`sync_all` 和 rename。
- `backup_file`：覆盖通过该 helper 进入的 backup copy 和 retention prune。

明确 P2：

- 本轮没有改 `lib.rs` 上层 read-modify-write 调用结构，因此不能宣称完整 RMW 窗口全串行化。
- `backup_file` 的 lock 会在 helper 返回时释放；调用方随后再 `write_validated` / `atomic_write` 时重新 acquire，因此 backup 与最终写入之间仍不是单一事务。
- 直接手写 backup 段不经过 `backup_file`，仍属历史边界；最终 rename 仍由 `atomic_write` lock 兜住。
- 完整跨 store / 跨文件事务仍应由 R3 SQLite 收口。

## Backup Retention 策略

策略函数：

- 只识别 `workflow-state.v0.<timestamp>.json`。
- 默认保留最近 30 份。
- 同时保留每日 1 份。
- 支持测试可控 ISO-like timestamp，例如 `2026-06-10T00-00-59`。
- 支持现有生产毫秒 timestamp，以 `millis / 86_400_000` 分桶成日。
- 不匹配命名的文件不参与 prune，避免误删非 workflow state 备份文件。

触发点：

- `backup_file(...)` 创建备份后调用 `prune_workflow_state_backups(...)`。

真实 backups：

- 本轮未启动 Tauri / Vite / Browser / Chrome。
- 本轮未运行任何会触碰用户真实 workflow state 目录的产品命令。
- prune 只在 Rust 单测创建的临时目录夹具中执行。

## Shape 指标

R1 前：

| 指标 | 值 |
| --- | ---: |
| `workflow_state_store.rs` | 91 lines |
| `lib.rs` | 25,925 lines |

R1 后：

| 指标 | 值 |
| --- | ---: |
| `workflow_state_store.rs` | 416 lines |
| `lib.rs` | 25,925 lines |

Shape gate check：

```text
Status: pass
Errors: 0
Warnings: 0
Info: 12
lib.rs: 25925/25925 (same)
Tauri commands: 96 total; 0 in lib.rs
Sidecar JSON kinds: 14 detected; 0 unknown
```

## TDD / 测试覆盖

先写测试后实现。

初次聚焦测试：

```bash
cargo test --lib workflow_state
```

结果：失败，原因符合预期：

- `StoreLock` 未实现。
- `workflow_state_lock_path` 未实现。
- `prune_workflow_state_backups` 未实现。
- 测试 wrapper 需要匹配 `write_validated` 的 `fn(&Path, &Value)` 写入签名。

新增 / 覆盖测试：

- `workflow_state_atomic_write_refuses_lock_busy_without_overwrite`
- `workflow_state_write_validated_refuses_corrupt_existing_without_overwrite`
- `workflow_state_revision_conflict_refuses_write_without_overwrite`
- `workflow_state_backup_retention_keeps_recent_30_and_daily_one`

## 验证记录

已运行：

```bash
cargo test --lib workflow_state
cargo test --lib
cargo fmt -- --check
git diff --check
node scripts/harness/workbench-shape-gate.js --mode check
git status --short
```

结果：

- `cargo test --lib workflow_state`：通过，11 passed / 0 failed / 341 filtered out；有既有 `invalid_params` dead_code warning。
- `cargo test --lib`：通过，336 passed / 0 failed / 16 ignored；有既有 `invalid_params` dead_code warning。
- `cargo fmt -- --check`：通过，无输出。
- `git diff --check`：通过，无输出。
- `node scripts/harness/workbench-shape-gate.js --mode check`：通过，0 errors / 0 warnings。
- `git status --short`：写 evidence 前仅有 `workflow_state_store.rs`；写 evidence/handoff 后应只包含 R1 范围文件。

## 边界确认

- 未迁移 SQLite。
- 未改 workflow state 顶层 schema。
- 未新增 sidecar store。
- 未新增 Tauri command。
- 未改真实 Codex runner。
- 未执行真实 `codex exec` / `codex exec resume`。
- 未发送 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未读取 secret、token、`.env`、keychain、OAuth、provider credential、完整 transcript 或 rollout。
- 未启动 Tauri / Browser / Chrome / Vite / 截图工具。
- 未对真实历史 backups 做不可逆清理。
- 未启动 Stage L / K3-B1 / K3-B2。
- 未做 UI、无限画布、MCP 视觉工具或 backlog 功能。

## P0 / P1 / P2

- P0：无。
- P1：无。
- P2：StoreLock 是文件级最终写入 / rename 锁，不覆盖完整 read-modify-write 事务窗口；R3 SQLite 或后续上层锁治理仍需补齐。
- P2：backup retention 当前挂在 `backup_file(...)` helper 上；历史手写 backup 段不经过该 helper，最终写入受 `atomic_write` lock 保护，但这些手写 backup 的 prune 覆盖需后续收敛。
- P2：retention prune 会删除匹配命名的过期 backups；本轮只在测试夹具验证，没有对真实历史 backups 做 dry-run 或清理。真实历史清理若需要，应另起任务并先 dry-run + 用户确认。
