# Stage H / H2.3 Real Resume Request Builder And Codex Local Guard Bridge v1 Result

日期：2026-06-07

H2.3 已完成：H2 授权矩阵完整时，现在可以构建 H1 `CodexLocalExecutionRequest`，并通过 H1 CodexLocal guard 做执行前复核。

## 已完成

- `InspectControlledSessionContinuationRealResumeOutput` 新增 `codex_local_request` 和 `codex_local_guard`。
- `inspect_real_resume_authorization` 在授权矩阵完整时构建 H1 request。
- `inspect_real_resume_authorization` 接入 `inspect_codex_local_execution_guard`。
- guard 阻断时返回 `blocked_by_codex_local_guard` 并保留 reason。
- 授权矩阵不完整时不构建 request / guard。
- 授权矩阵完整且 guard 允许时只返回 `complete_but_not_executed`，不调用真实 runner。
- 单测覆盖 request builder、guard bridge、结构化 argv 和非执行边界。
- 同步权威入口。

## 验证

已通过：

- `rustfmt --check src/session_continuation_store.rs src/types.rs`
- `cargo test --lib session_continuation`
- `cargo test --lib codex_local`
- `npm run typecheck`

复核扫描已完成：H2.3 已同步到 `CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md` 和两份阶段计划。误导口径扫描未发现 H2.3 被写成真实 resume 已执行、prompt 已发送、`.codex` 已读写、H2 已完成、H3 可开始、planned adapters 已接入或 provider credential 已验证；相关命中均为“不接受为 / 禁止说 / 未获授权前不得执行”等安全语境或历史记录。

## 边界

本轮没有执行真实 `codex exec` / `codex exec resume`，没有发送 prompt，没有读写 `/Users/yoyi/.codex`，没有读取 secret / credential / full transcript，没有创建 fixture 项目，没有启动 Tauri / GUI / 截图，也没有新增执行按钮。

## 接受范围

接受为 H2 非执行 request builder 和 CodexLocal guard bridge 完成。

不接受为 H2 通用真实 resume 产品化完成，不接受为真实 resume 已执行，不接受为 prompt 已发送，不接受为 `.codex` 已读写，不接受为 H3 send / 新会话可开始。

## 下一步

H2 真实执行前仍必须由用户和全局主管重新确认：

- 测试项目 / fixture。
- target session。
- 是否允许真实 `codex exec resume`。
- 是否允许触碰 `/Users/yoyi/.codex` 的 resume 必需最小范围。
- prompt summary/hash/ref。
- allowed write roots。
- readback plan。
- runtime log / audit / evidence。
- rollback / 降级策略。

未确认前不得执行真实 resume，也不得进入 H3。
