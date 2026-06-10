# Stage H / H3.1 New Session Request, Guard, Permission Envelope, And Noop Runner Handoff v1

日期：2026-06-07

结论：H3.1 已完成并通过全局主管复核。  
接受范围：`new_session` request、guard、permission envelope、command plan preview、no-op runner、智能体页只读展示和秘书只读解释完成。  
不接受范围：真实 `codex exec`、真实 `codex exec resume`、prompt 发送、真实 Codex session 创建、`.codex` 读写、H2 Phase B 满足、H3-B 授权 / 执行、H3 产品化完成或阶段 H 完成。

## 本轮完成

- 执行并收口 `tasks/2026-06-07-stage-h-h3-1-new-session-request-guard-permission-envelope-and-noop-runner-v1.md`。
- `new_session` 已进入 Rust / TS session operation、continuation preview 和 CodexLocal guard 范围。
- `new_session` 不要求 existing `session_id`，但必须绑定 `work_item_id`。
- no-op / dry-run attempt 显式保持 `prompt_sent=false`、`real_codex_executed=false`、`writes_codex_home=false`。
- 智能体页只读展示 H3.1 新会话预览、command plan 摘要、guard 状态和 no-op runner 状态。
- 秘书只解释风险和查看建议，不生成新建会话、发送、resume、批准或重试 action proposal。
- 新增 evidence，并同步当前入口、任务队列、权威索引、阶段计划和 H-I 阶段计划。

## 验证

已通过：

- `npm run test:offline-interaction`：12 passed
- `npm run typecheck`
- `npm run build`：通过，仅保留既有 Vite chunk-size warning
- `cargo test --lib codex_local_runner`：8 passed
- `cargo test --lib session_operation`：1 passed
- `cargo test --lib session_continuation`：10 passed
- `cargo test --lib`：245 passed / 1 ignored
- `rustfmt --check src/codex_local_runner.rs src/lib.rs src/types.rs src/runtime_session_attention.rs src/session_continuation_store.rs`
- 旧状态和误导文案扫描已完成；文档里的敏感完成态词只出现在禁止项 / 不接受范围 / 扫描要求语境。

已知非阻塞提示：

- Rust 仍有既有 `JsonRpcError::invalid_params` unused warning。
- Vite build 仍有既有 chunk size warning。

## 边界确认

本轮未执行真实 `codex exec`。  
本轮未执行真实 `codex exec resume`。  
本轮未发送真实 prompt。  
本轮未创建真实 Codex session。  
本轮未读写 `/Users/yoyi/.codex`。  
本轮未读取 auth、token、secret、`.env`、keychain、OAuth、provider credential 或完整 transcript / rollout。  
本轮未创建 H3-B 真实 fixture。  
本轮未启动 Tauri / GUI / 截图。  

## 当前阻断

```text
h2_phase_b_readiness = blocked_waiting_target_session
h3_b_authorization_request = not_ready
real_codex_execution = not_authorized
codex_home_access = not_authorized
```

H3.1 不解除 H2 Phase B 的 target session 阻断，也不能用 `new_session` 绕过 H2 final approval。

## 下一步建议

下一步仍需全局主管明确选择：

- 若要做真实新会话，单独拆 H3-B final approval / real new session fixture run，先冻结 fixture、allowed write roots、prompt ref/hash、`.codex` 最小范围、readback、runtime log、audit、evidence 和 rollback，再请求用户明确授权。
- 若要补 H2 Phase B，必须先提供 existing target session，并再次确认 fixture、permission envelope 和 `.codex` 最小读写范围。
- 未获授权前，不进入 H4 / H5 真实执行链路，也不把 H3.1 冒认为 H3 产品化完成。
