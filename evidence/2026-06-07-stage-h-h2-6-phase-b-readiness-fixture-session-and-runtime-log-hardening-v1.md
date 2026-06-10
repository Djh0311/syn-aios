# Stage H / H2.6 Phase B Readiness, Fixture Session Binding, And Runtime Log Hardening Evidence v1

日期：2026-06-07

状态：H2.6 已完成；H2 / H2.5 Phase B 仍未完成。  
结论：接受为 H2 Phase B 前置条件复核和 runtime log 显式 sidecar writer 硬化完成；不接受为真实 `codex exec resume` 已执行、prompt 已发送、`/Users/yoyi/.codex` 已读写、fixture 已创建、target session 已确认、H2 通用真实 resume 产品化完成、H3 可开始或阶段 H 完成。

## 1. 本轮范围

本轮执行 `tasks/2026-06-07-stage-h-h2-6-phase-b-readiness-fixture-session-and-runtime-log-hardening-v1.md`。

本轮允许并完成：

- 复核 H2.5 Phase A 后进入 Phase B 前的 fixture、target session、runtime log、permission envelope 和 rollback 前置条件。
- 将 H2 Phase A attempt 的 runtime log 从仅 `runtime_log_ref` / 派生摘要推进到显式 `runtime-logs.v1.json` sidecar 写入。
- 在 runtime log sidecar 损坏时拒绝继续写 H2 Phase A continuation attempt，避免覆盖损坏 JSON。
- 保持 readback unavailable / failed / timed_out 的 `result_count = None`，不写成真实 0 条结果。
- 保留 runtime log 与 audit event 分离：runtime log 只引用 `audit_refs`，不替代 audit。

本轮禁止且未做：

- 未执行真实 `codex exec`。
- 未执行真实 `codex exec resume`。
- 未发送真实 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未读取 auth、token、secret、`.env`、keychain、OAuth、provider credential 或完整 transcript。
- 未创建真实 H2 fixture。
- 未确认 target session。
- 未启动 Tauri / GUI / 截图。
- 未新增 UI、按钮或真实执行入口。

## 2. 代码落点

- `prototypes/productized-desktop-shell/src-tauri/src/runtime_log_store.rs`
  - 新增 `ensure_appendable`，用于 H2 Phase A 写入前拒绝损坏 runtime log sidecar。
  - 新增 `append_session_continuation_attempt`，显式写入 `runtime-logs.v1.json`。
  - runtime log sidecar 写入包含 lock、backup、atomic rename、redacted summary、summary rebuild 和损坏 JSON 拒绝覆盖。
  - 显式写入的 runtime log entry 包含 workflow run、dispatch attempt 和 readback 三类摘要。

- `prototypes/productized-desktop-shell/src-tauri/src/session_continuation_store.rs`
  - H2.5 Phase A 在写 continuation attempt 前检查 runtime log sidecar 是否可追加。
  - H2.5 Phase A 写完 continuation attempt / audit 后显式 append runtime log sidecar。
  - 新增测试覆盖 runtime log sidecar 成功写入和损坏 sidecar 阻断。

## 3. 验证记录

已通过：

```text
cargo test --lib session_continuation
cargo test --lib runtime_log
cargo test --lib codex_local
cargo test --lib
cargo fmt --check -- src/runtime_log_store.rs src/session_continuation_store.rs
```

结果摘要：

- `cargo test --lib session_continuation`：10 passed。
- `cargo test --lib runtime_log`：2 passed。
- `cargo test --lib codex_local`：6 passed。
- `cargo test --lib`：243 passed，1 ignored。
- `cargo fmt --check -- src/runtime_log_store.rs src/session_continuation_store.rs`：通过。
- Rust 仍有既有 `JsonRpcError::invalid_params` unused warning。

## 4. 测试覆盖

新增 / 更新覆盖：

- H2 Phase A no-op runner path 会显式写 `runtime-logs.v1.json`。
- runtime log sidecar 包含 dispatch attempt 和 readback 摘要。
- runtime log entry 全部保持 `redacted_safe_summary`。
- runtime log 不包含完整 prompt summary 或 `codex exec resume` command body。
- runtime log sidecar 损坏时，H2 Phase A 拒绝继续写 continuation attempt。
- 损坏 runtime log sidecar 不会被覆盖。
- continuation store 在损坏 runtime log 阻断时仍停留在 confirm 后 revision，未新增 attempt。

## 5. Readiness 结论

H2.6 最终 readiness：

```text
h2_phase_b_readiness = blocked_waiting_fixture_and_target_session
runtime_log_writer = explicit_sidecar_writer_ready
phase_b_authorization_request = not_ready
```

判断依据：

- runtime log writer 已补齐为显式 sidecar 写入，不再只是派生摘要。
- 推荐 fixture `/Users/yoyi/workspace/product-line/tmp/h2-real-resume-fixture` 仍未创建。
- 未确认可用于 H2 Phase B 的 target session。
- 未产生可作为真实 Phase B 证据的 workbench continuation sidecar 实例。
- Phase B 真实执行仍缺用户 / 全局主管二次授权、fixture hash、rollback、allowed write roots、prompt hash/ref、`.codex` 最小读写范围和 readback plan。

## 6. 接受范围

接受为：

- H2.6 Phase B 前置条件复核完成。
- runtime log 显式 sidecar writer 硬化完成。
- H2 Phase A attempt 可写 explicit runtime log sidecar。
- runtime log 损坏 JSON 拒绝覆盖和 H2 attempt 阻断完成。
- fixture / target session 缺失状态已冻结。

不接受为：

- H2 通用真实 resume 产品化完成。
- H2.5 Phase B 已授权或已执行。
- 真实 `codex exec resume` 已执行。
- prompt 已发送。
- `/Users/yoyi/.codex` 已读写。
- fixture 已创建。
- target session 已确认。
- H3 通用真实 send / 新会话完成或可直接开始。
- H5 项目工作流真实派发完成。
- 阶段 H 完成。

## 7. 下一步

下一步不能直接执行 Phase B。必须先由用户 / 全局主管明确：

- 是否创建或使用隔离 fixture。
- target session 是哪个 existing session，且如何绑定到 fixture / workflow / node。
- `.codex` 最小读写范围。
- allowed write roots 和 denied paths。
- prompt summary / hash / ref。
- readback plan。
- runtime log / audit / evidence / handoff 路径。
- rollback / failure classification。

如果无法提供 target session，应停在 `blocked_waiting_target_session`，或另拆 H3 通用真实 send / 新会话任务包；不能用 H3 绕过 H2 Phase B。
