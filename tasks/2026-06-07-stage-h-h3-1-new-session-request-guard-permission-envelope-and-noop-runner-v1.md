# Stage H / H3.1 New Session Request, Guard, Permission Envelope, And Noop Runner v1

日期：2026-06-07

状态：已完成并已通过全局主管复核。  
用途：在 H3-A 已完成非执行授权冻结、H3-B 真实新会话仍未授权的前提下，把 H3 通用真实 send / 新会话的 request、guard、permission envelope 和 no-op runner 路径落成产品代码。H3.1 只允许非执行产品路径；不创建真实 Codex session，不发送 prompt，不读写 `/Users/yoyi/.codex`。

## 1. 权威依据

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/plans/README.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `docs/plans/2026-06-07-stage-h-i-real-codex-automation-and-multi-agent-collaboration-plan-v1.md`
- `docs/workbench-system-architecture-v1.md`
- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`
- `tasks/2026-06-07-stage-h-h3-a-new-session-authorization-fixture-and-boundary-freeze-v1.md`
- `evidence/2026-06-07-stage-h-h3-a-new-session-authorization-fixture-and-boundary-freeze-v1.md`
- `handoffs/2026-06-07-stage-h-h3-a-new-session-authorization-fixture-and-boundary-freeze-v1-result.md`

## 2. 当前事实

- H3-A 已完成，结论是 `h3_a_readiness = completed_boundary_freeze`。
- H3-B 真实新会话仍未开始，当前 `h3_b_authorization_request = not_ready`。
- H2 Phase B 仍是 `blocked_waiting_target_session`，H3.1 不能用新会话绕过 H2 target session 缺口。
- 现有 H1 `CodexLocalExecutionRequest` / guard / fake runner 已覆盖 `send_message` 和 `resume`，但 `new_session` 尚未作为 H3 独立 operation 进入产品路径。
- 现有 E2/E4/E5 continuation UI 只覆盖 `send_message` / `resume`，真实执行仍未授权。

## 3. 目标

H3.1 目标：

- 把 `new_session` 明确纳入 `codex-local` 受控 operation / request / guard 范围，并保持与 H2 `resume` 区分。
- 为 `new_session` 生成结构化 command plan：`program + argv + stdin_prompt_ref`，禁止 shell 字符串拼接，禁止 prompt 进入 argv。
- 允许 `new_session` 在非执行 no-op runner 中通过 guard，生成 attempt / runtime log ref / audit ref / readback unavailable 边界。
- 在智能体页显示 H3.1 新会话边界摘要：可见 request / guard / permission envelope / no-op 状态，但没有执行按钮。
- 离线测试覆盖 `new_session` guard、command plan、安全阻断、UI 文案和误导文案黑名单。
- 新增 evidence / handoff，并同步权威入口。

H3.1 不目标：

- 不执行真实 `codex exec`。
- 不执行真实 `codex exec resume`。
- 不发送真实 prompt。
- 不创建真实 Codex session。
- 不读写 `/Users/yoyi/.codex`。
- 不读取 auth、token、secret、`.env`、keychain、OAuth、provider credential 或完整 transcript / rollout。
- 不创建 H3-B 真实 fixture。
- 不启动 Tauri / GUI / 截图。
- 不接入 planned adapters 的真实执行。
- 不做 provider credential store、model verification、自动重试、自动恢复或跨 provider 调度。

## 4. UI 显示边界确认

H3.1 涉及智能体页读模型展示，必须遵守 UI 显示方案：

- 页面位置：智能体页 existing session / adapter / continuation 区域附近。
- 展示内容：`new_session` readiness、guard status、permission envelope 摘要、command plan redacted preview、no-op runner 状态、readback unavailable 边界。
- 禁止内容：裸“新建会话”按钮、自由聊天输入框、发送按钮、执行按钮、重试按钮、真实 Codex 已执行文案。
- planned adapters 必须继续显示 planned / unavailable，不能因为 H3.1 变成可执行。
- readback unavailable / failed / timed_out 的 `result_count` 必须保持 `null`，不能显示为 0 条结果。
- 秘书只能解释风险和查看建议，不得生成批准、发送、新建会话或重试 action proposal。

## 5. 实现范围

建议最小实现：

1. 后端 / 类型：扩展 `CodexLocalExecutionRequest.operation_id` 边界，允许 `new_session`。
2. 后端 guard：`new_session` 不要求 existing `session_id`，但必须绑定 project / workflow / node / work item 或等价任务上下文、authorization scope、prompt summary/ref/hash、readback plan、allowed write roots、audit refs。
3. 后端 command plan：`new_session` 使用 `codex exec -C <cwd> --sandbox <sandbox> --add-dir <root> --json --output-last-message <managed-path>`，不带 `resume` 和 session id。
4. 后端 no-op runner：复用 H1 fake dry-run 或新增 H3.1 no-op runner，但必须显式输出 `prompt_sent=false`、`real_codex_executed=false`、`writes_codex_home=false`。
5. 前端类型 / 读模型：补 `new_session` operation label 和 H3.1 panel。
6. 前端测试：覆盖 H3.1 UI 可见和禁止按钮 / 禁止误导文案。

## 6. 验收命令

如改产品代码，至少运行：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
cargo test --lib codex_local_runner
cargo test --lib session_operation
cargo test --lib session_continuation
cargo test --lib
rustfmt --check src/codex_local_runner.rs src/lib.rs src/types.rs
```

并扫描：

```text
rg -n -e '真实新会话已创建' -e '真实 send 已执行' -e 'prompt 已发送' -e 'H3-B 已授权' -e 'H3 已完成' prototypes/productized-desktop-shell/src
```

## 7. 接受范围

H3.1 完成后可接受为：

- H3 通用真实 send / 新会话的 request、guard、permission envelope 和 no-op runner 非执行产品路径完成。
- `new_session` 与 H2 `resume` 在 contract 和 command plan 上已明确区分。
- H3-B final approval 前置条件更清晰。

H3.1 不接受为：

- 真实 `codex exec` 已执行。
- 真实 Codex session 已创建。
- prompt 已发送。
- `/Users/yoyi/.codex` 已读写。
- H2 Phase B 已满足。
- H3-B 已授权或已执行。
- H3 产品化完成。
- H5 项目工作流真实派发完成。
- 阶段 H 完成。

## 8. 回收结果

H3.1 已完成，记录见：

- `evidence/2026-06-07-stage-h-h3-1-new-session-request-guard-permission-envelope-and-noop-runner-v1.md`
- `handoffs/2026-06-07-stage-h-h3-1-new-session-request-guard-permission-envelope-and-noop-runner-v1-result.md`

实际落地：

- Rust / TS operation 边界包含 `new_session`。
- `SessionContinuationRequest.work_item_id` 已补齐并兼容旧数据。
- `new_session` preview 不要求 existing `session_id`，但必须绑定 `work_item_id`。
- CodexLocal guard 允许 `new_session` 非执行 no-op / dry-run 通过；缺 work item 会阻断。
- command plan 保持结构化 argv，不拼 shell，不把 prompt 放入 argv。
- 智能体页只读显示 H3.1 新会话预览、guard、permission envelope、command plan 和 no-op runner 状态。
- 秘书只解释风险，不提供新建会话、发送、resume、批准或重试 action proposal。

验证已通过：

```text
npm run test:offline-interaction
npm run typecheck
npm run build
cargo test --lib codex_local_runner
cargo test --lib session_operation
cargo test --lib session_continuation
cargo test --lib
rustfmt --check src/codex_local_runner.rs src/lib.rs src/types.rs src/runtime_session_attention.rs src/session_continuation_store.rs
```

边界保持：

- 未执行真实 `codex exec`。
- 未执行真实 `codex exec resume`。
- 未发送真实 prompt。
- 未创建真实 Codex session。
- 未读写 `/Users/yoyi/.codex`。
- H2 Phase B 仍是 `blocked_waiting_target_session`。
- H3-B 仍是 `not_ready`，必须另开 final approval / real run 任务。
