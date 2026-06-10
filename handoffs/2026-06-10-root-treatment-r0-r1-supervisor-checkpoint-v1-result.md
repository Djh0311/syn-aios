# Root Treatment R0 / R1 Supervisor Checkpoint v1 Result

日期：2026-06-10

## 结论

R0 / R1 已完成并通过主管复核，结论为 `accepted_with_p2`。下一步可以进入 R2 `lib.rs` 解体任务包准备。

## 已完成

- R0 已提交：`7563e6a9d11a92217e1baf34ed71b70722bbc17c`
- R1 已提交：`7a1ac89173306b50868064b64fb852f57c0550af`
- 主管线已同步当前入口、任务包状态和 R0/R1 commit hash。
- 新增 checkpoint evidence：`evidence/2026-06-10-root-treatment-r0-r1-supervisor-checkpoint-v1.md`。

## 验证

主管线 fresh verify 通过：

- `cargo test --lib workflow_state`：11 passed。
- `cargo fmt -- --check`：通过。
- `node scripts/harness/workbench-shape-gate.js --mode check`：通过，0 errors / 0 warnings。
- `git diff --check`：通过。
- `git status --short`：通过，无输出。
- `cargo test --lib`：336 passed / 16 ignored；保留既有 `JsonRpcError::invalid_params` dead_code warning。

## P2

- R0 sidecar 扫描仍是源码字符串级，动态拼接需后续收紧。
- R0 command 总量增加仍是 warning，R2 后可收紧。
- R1 StoreLock 不覆盖完整 read-modify-write 事务窗口。
- R1 retention 不覆盖 `lib.rs` 历史手写 backup 段。
- R1 未清理真实历史 backups。
- R1 StoreLock 未实现 stale lock recovery。

## 边界

本 checkpoint 没有执行真实 Codex，没有发送 prompt，没有读写 `/Users/yoyi/.codex`，没有迁移 SQLite，没有改 workflow state 顶层 schema，没有新增 sidecar / command / UI，没有清理真实 backups，也没有启动 Tauri / Browser / Chrome / Vite / 截图工具。

## 下一步

创建并执行 R2 第一批任务包。建议第一批限定为“命令注册 / 分发出 `lib.rs`”，不要同时做 workflow 读模型、记忆领域、runtime diagnostics 或 SQLite。
