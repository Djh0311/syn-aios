# Stage H / H2.0 Real Resume Preflight Authorization Guard v1 Evidence

日期：2026-06-07

结论：H2.0 接受为真实 resume 执行前授权预检和阻断底座完成；不接受为 H2 通用真实 resume 产品化完成。

## 1. 范围

本轮只把 H2 的执行前授权矩阵落成产品 guard 和 continuation sidecar 可追踪记录：

- 新增 H2 real resume authorization matrix 类型。
- 新增 `inspect_controlled_session_continuation_real_resume_authorization` Tauri command。
- 新增后端 `inspect_real_resume_authorization`，在未满足授权矩阵时写入 `blocked_waiting_authorization` attempt / audit。
- 授权矩阵完整时只返回 `complete_but_not_executed` 和 `ready_for_real_resume_authorization`，仍不调用真实 runner。
- 前端新增 TS 类型和 Tauri wrapper；没有新增真实执行按钮或自由发送入口。

## 2. 边界

本轮没有：

- 执行真实 `codex exec`。
- 执行真实 `codex exec resume`。
- 发送真实 prompt。
- 读写 `/Users/yoyi/.codex`。
- 读取 auth/token/.env/secret/keychain/OAuth/provider credential/full transcript。
- 启动 Tauri、GUI 或截图。
- 把 H2 标记为完成。
- 进入 H3/H4/H5。

## 3. 代码落点

- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/session_continuation_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`

## 4. 验证

已通过：

- `rustfmt --check src/session_continuation_store.rs src/types.rs src/commands.rs src/lib.rs`
- `cargo test --lib session_continuation`
- `cargo test --lib`
- `npm run typecheck`

`cargo test --lib` 结果：238 passed, 1 ignored。保留既有 `JsonRpcError::invalid_params` unused warning。

## 5. 关键断言

新增 Rust 测试覆盖：

- 缺 H2 授权矩阵时，attempt 状态为 `blocked_waiting_authorization`。
- blocked attempt 中 `prompt_sent=false`、`real_codex_executed=false`、`writes_codex_home=false`。
- readback unavailable 保持 `result_count=None`，不显示为 0 条结果。
- 授权矩阵完整时，只返回 `complete_but_not_executed`，仍不执行真实 runner。

## 6. 下一步

H2 真实 resume 产品化仍待执行前授权。继续 H2 前必须逐项确认：

- 测试项目。
- target session。
- project root / target cwd。
- allowed write roots。
- `/Users/yoyi/.codex` 最小读写范围。
- prompt summary / hash / ref。
- readback plan。
- runtime log / audit / evidence。
- rollback strategy。
- 用户确认和全局主管确认。
