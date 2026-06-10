# Root Treatment R0 / R1 Supervisor Checkpoint v1

日期：2026-06-10

## 结论

R0 / R1 已由主管线回收为 `accepted_with_p2`，允许进入 R2 `lib.rs` 解体任务包准备。

接受范围：

- R0：workbench shape gate、任务包“形状影响”必填节、治理任务包类型、解冻后 `1:3` 治理配额、commit hash 记录要求已建立。
- R1：`workflow-state.v0.json` 最终写入 / rename 已有文件级 StoreLock；lock busy、corrupt JSON、revision conflict 不覆盖原文件；backup retention 策略已在测试夹具中验证。

不接受范围：

- 不接受为 R2 `lib.rs` 解体完成。
- 不接受为 R3 SQLite 迁移完成。
- 不接受为完整 read-modify-write 事务串行化。
- 不接受为真实历史 backups 已清理。
- 不接受为 Stage L / K3-B1 / K3-B2 恢复。
- 不接受为新的真实 Codex 执行授权。

## Commit 记录

- R-Preflight baseline：`ed01c6f281e3fd7a38548da948046e8366cc368d`
- R-Preflight 收口：`b409ab92d36b44f63911a4f12b057e5577f8aeb5`
- R0 / R1 任务包创建：`a40b7b56ab949cd26b145ba0eccf9f3921886ea0`
- R0 completion：`7563e6a9d11a92217e1baf34ed71b70722bbc17c`
- R1 completion：`7a1ac89173306b50868064b64fb852f57c0550af`

## 分线回收

R0 分线：

- 线程：`019eb263-54a9-7081-9725-25057df15d1c`
- 状态：完成实现、证据和交接；因共享工作树出现 R1 dirty 文件，R0 分线按边界未自行提交。
- 主管动作：只 stage R0 文件并分区提交为 `7563e6a9d11a92217e1baf34ed71b70722bbc17c`，未 stage / 未改 R1 文件。

R1 分线：

- 线程：`019eb264-774b-72a2-a90e-3d89ea77e655`
- 状态：完成实现、证据、交接和独立提交。
- 主管动作：执行 fresh verify 和代码 / 证据复核，确认提交 `7a1ac89173306b50868064b64fb852f57c0550af` 可接受。

## R0 验证

主管线重新运行：

```bash
git diff --cached --check
node --check scripts/harness/workbench-shape-gate.js
node scripts/harness/workbench-shape-gate.js --mode baseline
node scripts/harness/workbench-shape-gate.js --mode check
```

结果：

- staged diff check：通过。
- node syntax check：通过。
- baseline：通过，0 errors / 0 warnings。
- check：通过，0 errors / 0 warnings。

关键 R0 指标：

- `lib.rs`：25,925 lines。
- Tauri commands：96 total，0 in `lib.rs`。
- sidecar JSON kinds：14 detected，0 unknown。
- ratchet files：12。
- gate script：407 lines。

## R1 验证

主管线重新运行：

```bash
cargo test --lib workflow_state
cargo fmt -- --check
node scripts/harness/workbench-shape-gate.js --mode check
git diff --check
git status --short
cargo test --lib
```

结果：

- `cargo test --lib workflow_state`：通过，11 passed / 0 failed / 341 filtered out。
- `cargo fmt -- --check`：通过。
- shape gate check：通过，0 errors / 0 warnings。
- `git diff --check`：通过。
- `git status --short`：通过，无输出。
- `cargo test --lib`：通过，336 passed / 0 failed / 16 ignored；保留既有 `JsonRpcError::invalid_params` dead_code warning。

## P0 / P1 / P2

- P0：无。
- P1：无。
- P2：R0 sidecar 扫描基于源码字符串和允许清单；动态拼接 sidecar 名称仍需 R2/R3 前继续收紧或由 R3 SQLite 收口消化。
- P2：R0 command 总数增加当前只作为 warning；R2 命令注册拆分后可进一步收紧 command surface。
- P2：R1 StoreLock 只覆盖文件级最终写入 / rename，不覆盖完整 read-modify-write 事务窗口。
- P2：R1 backup retention 挂在 `backup_file(...)` helper；历史手写 backup 段的 prune 覆盖仍需后续收敛。
- P2：R1 未清理真实历史 backups；如要清理必须另起 dry-run + 用户确认任务。
- P2：StoreLock 目前无 stale lock recovery；如果进程异常退出遗留 lock 文件，后续需要单独设计诊断 / 恢复路径，不能在 R1 冒充已解决。

## 边界确认

- 未执行真实 `codex exec` / `codex exec resume`。
- 未发送 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未读取 secret、token、`.env`、keychain、OAuth、provider credential、完整 transcript 或 rollout。
- 未启动 Tauri / Browser / Chrome / Vite / 截图工具。
- 未迁移 SQLite。
- 未改 workflow state 顶层 schema。
- 未新增 sidecar store。
- 未新增 Tauri command。
- 未清理真实历史 backups。
- 未启动 Stage L / K3-B1 retry / K3-B2。
- 未解冻 backlog 功能。

## 下一步

进入 R2 前必须先创建 R2 批次任务包。建议 R2 第一批只做命令注册 / 分发出 `lib.rs`，且必须满足：

- 使用 R0 shape gate baseline / check。
- 每批形成独立 commit。
- 每批记录 `lib.rs` 前后行数。
- 不新增 command 到 `lib.rs`。
- 不新增 sidecar。
- 若新文件超过 Rust 3,000 行或 TS/TSX 2,000 行，必须先写 decision。
