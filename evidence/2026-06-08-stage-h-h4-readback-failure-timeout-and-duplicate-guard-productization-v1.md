# Evidence：Stage H / H4 Readback Failure Timeout And Duplicate Guard Productization v1

日期：2026-06-08

## 1. 结论

H4 Level A 非真实执行产品化已完成，结论为：

```text
accepted_as_h4_level_a_non_real_productization
```

接受为：

- `codex-local` readback / failure / timeout / duplicate guard / stale cleanup 的统一产品边界完成。
- `result_count = null` 规则已收敛到 H4 helper，并接入 runner、continuation store 和 runtime attention。
- `readback_unavailable`、`readback_failed`、`readback_timed_out`、`timed_out`、`not_attempted`、`blocked_by_guard`、`duplicate_blocked`、`user_rejected`、`cancel_requested`、`stale_cancelled` 均不会被显示或记录为真实 0 条结果。
- duplicate guard 已从同一 continuation 扩展为同一 adapter + operation + continuation / work item / workflow node / target session 范围内的 active attempt 阻断。
- duplicate blocked 会写工作台自有 attempt / audit / runtime log，不调用 runner。
- stale cleanup 只处理工作台自有 active attempt，必须提供 expected revision，写 audit / runtime log，状态为 `stale_cancelled`，不 kill Codex、不改 `/Users/yoyi/.codex`、不自动 retry。
- diagnostics / runtime attention 能识别 `readback_timed_out`、`duplicate_blocked`、`stale_cancelled` 等 H4 状态。

不接受为：

- 真实 `codex exec` 已执行。
- 真实 `codex exec resume` 已执行。
- prompt 已发送。
- 真实 Codex session 已创建。
- `/Users/yoyi/.codex` 已读写。
- H3-B retry 已授权或已执行。
- H5 项目工作流真实派发已完成。
- 自动重试、自动恢复、自动 stop / kill 产品化完成。
- H4-Level-B 真实失败 / 超时探针完成。
- planned adapters 真实接入。
- provider credential / model verification 完成。
- 阶段 H 完成。

## 2. 实现落点

新增：

- `prototypes/productized-desktop-shell/src-tauri/src/h4_execution_boundary.rs`

修改：

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/codex_local_runner.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/session_continuation_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/runtime_session_attention.rs`

未改：

- 前端 / TS / UI。
- 真实 runner 授权路径。
- `/Users/yoyi/.codex`。

## 3. 关键边界

`h4_execution_boundary.rs` 提供统一规则：

- active attempt status：`queued` / `running` / `waiting_permission` / `waiting_for_permission` / `running_stub` / `dry_run_running` / `queued_real` / `running_real` / `running_h2_phase_a` / `running_h2_phase_b` / `running_h3_b`。
- unknown-result status：`readback_unavailable` / `readback_failed` / `readback_timed_out` / `timed_out` / `not_attempted` / `not_attempted_stub` / `blocked_by_guard` / `blocked_waiting_authorization` / `duplicate_blocked` / `user_rejected` / `cancel_requested` / `stale_cancelled` / `dry_run_blocked`。
- `h4_result_count(...)` 对 unknown-result status 强制返回 `None`，防止把未读回 / 失败 / 超时 / guard 阻断显示为 0。

duplicate guard：

- H2 preflight、H2 Phase A、H2 Phase B、H3-B 都改用 H4 scope 检查。
- 同一 continuation active attempt 阻断。
- 同一 resume target session active attempt 阻断。
- 同一 new_session work item active attempt 阻断。
- 同一 workflow node + work item active attempt 阻断。
- 阻断时 `codex_local_attempt = None`，即不调用 runner。

stale cleanup：

- 新增 `cleanup_stale_attempt` 后端 store 函数。
- 必须传 `expected_store_revision`。
- 只允许清理工作台 sidecar 中 active attempt。
- 写 `session_continuation_stale_attempt_cancelled` audit event。
- 写 runtime log dispatch / readback entries。
- 保留 `result_count = null`。
- 不 kill Codex，不改 Codex home，不做 retry。

## 4. 验证记录

已通过：

```text
cargo test --lib codex_local_runner -- --nocapture
cargo test --lib session_continuation -- --nocapture
cargo test --lib runtime_log -- --nocapture
cargo test --lib runtime_session_attention -- --nocapture
cargo test --lib g2_diagnostic -- --nocapture
cargo test --lib
rustfmt --check src/h4_execution_boundary.rs src/codex_local_runner.rs src/session_continuation_store.rs src/runtime_log_store.rs src/runtime_session_attention.rs src/types.rs src/lib.rs
```

结果摘要：

- `codex_local_runner`：11 passed。
- `session_continuation`：16 passed / 2 ignored。
- `runtime_log`：5 passed。
- `runtime_session_attention`：2 passed。
- `g2_diagnostic`：1 passed。
- `cargo test --lib`：254 passed / 3 ignored。
- `rustfmt --check ...`：通过。

未运行：

- `npm run typecheck`、`npm run test:offline-interaction`、`npm run build`。本轮未改前端 / TS / UI。
- ignored 的真实 H2 / H3-B probe 测试。H4 Level A 不授权真实 Codex 执行。

既有提示：

- Rust 保留既有 warning：`JsonRpcError::invalid_params` unused / dead code。

## 5. 边界确认

本轮做了：

- 修改 Rust 类型、store、runner 边界、runtime attention 和 diagnostic 读模型。
- 用 fake / no-op / 单测注入覆盖 success、unknown-result、duplicate blocked 和 stale cleanup。
- 写入本 evidence / handoff。

本轮没有做：

- 没有执行真实 `codex exec`。
- 没有执行真实 `codex exec resume`。
- 没有发送 prompt。
- 没有创建真实 fixture run。
- 没有读取或写入 `/Users/yoyi/.codex`。
- 没有读取 auth、token、secret、`.env`、keychain、OAuth、provider credential、完整 transcript 或 rollout。
- 没有改可见 UI。
- 没有生成秘书执行 / 批准 / retry / stop / kill action proposal。

## 6. H3-B / H5 状态

H3-B 仍保持：

```text
h3_b_real_new_session_fixture_run = failed_classified
h3_b_product_path_patch = ready_for_next_authorized_probe
```

H4 不依赖 H3-B 已成功；H4 Level A 完成也不等于 H3-B retry 已授权。

H5 仍不能跳过 H3-B / H4：

- H5-Level-A 可以继续做协议 / 产品路径集成设计。
- H5-Level-B 真实项目工作流派发必须等待 H3-B retry 授权回收清楚，并按 H4 duplicate / readback / failure 边界执行点授权。

