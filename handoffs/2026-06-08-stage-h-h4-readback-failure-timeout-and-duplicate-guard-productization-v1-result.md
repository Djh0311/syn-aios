# Handoff：Stage H / H4 Readback Failure Timeout And Duplicate Guard Productization v1

日期：2026-06-08

结论：H4 Level A 非真实执行产品化已完成。接受为 readback / failure / timeout / duplicate guard / stale cleanup 统一产品边界完成；不接受为真实 Codex 执行、H4-Level-B 探针、H3-B retry、H5 真实派发或阶段 H 完成。

## 完成内容

- 新增 H4 helper，统一 active attempt 和 unknown-result 状态规则。
- runner / store / runtime attention 统一使用 `result_count = null` 规则。
- duplicate guard 扩展到同 adapter + operation + continuation / work item / workflow node / target session 范围。
- duplicate blocked 写 attempt / audit / runtime log，且 `codex_local_attempt = None`。
- 新增工作台自有 stale attempt cleanup：要求 expected revision，写 audit / runtime log，标记 `stale_cancelled`。
- diagnostics / runtime attention 增加 `readback_timed_out`、`duplicate_blocked`、`stale_cancelled` 边界识别。

## 改动文件

- `prototypes/productized-desktop-shell/src-tauri/src/h4_execution_boundary.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/codex_local_runner.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/session_continuation_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/runtime_session_attention.rs`
- `evidence/2026-06-08-stage-h-h4-readback-failure-timeout-and-duplicate-guard-productization-v1.md`
- `handoffs/2026-06-08-stage-h-h4-readback-failure-timeout-and-duplicate-guard-productization-v1-result.md`

## 验证

已通过：

- `cargo test --lib codex_local_runner -- --nocapture`：11 passed
- `cargo test --lib session_continuation -- --nocapture`：16 passed / 2 ignored
- `cargo test --lib runtime_log -- --nocapture`：5 passed
- `cargo test --lib runtime_session_attention -- --nocapture`：2 passed
- `cargo test --lib g2_diagnostic -- --nocapture`：1 passed
- `cargo test --lib`：254 passed / 3 ignored
- `rustfmt --check src/h4_execution_boundary.rs src/codex_local_runner.rs src/session_continuation_store.rs src/runtime_log_store.rs src/runtime_session_attention.rs src/types.rs src/lib.rs`：通过

未运行 npm 验证，因为本轮未改前端 / TS / UI。

## 边界

本轮没有执行真实 `codex exec` / `codex exec resume`，没有发送 prompt，没有创建真实 fixture run，没有读写 `/Users/yoyi/.codex`，没有读取 credential / secret / auth / token / `.env` / keychain / OAuth / full transcript / rollout。

本轮没有改可见 UI；秘书边界未新增任何批准、发送、resume、new_session、retry、stop 或 kill action proposal。

## 下一步建议

- H3-B retry：如要验证 `--skip-git-repo-check` 修补后的真实 new-session，必须重新取得执行点授权。
- H5-Level-A：可以继续做协议 / 产品路径集成设计，但不能执行真实项目派发。
- H4-Level-B：如需真实失败 / 超时探针，必须单独授权 fixture、operation、allowed write roots、`.codex` 最小范围、prompt summary/ref/hash、timeout、readback、runtime log、audit、evidence、handoff、rollback 和停止条件。

