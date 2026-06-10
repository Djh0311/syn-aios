# Stage H / H3.1 New Session Request, Guard, Permission Envelope, And Noop Runner Evidence v1

日期：2026-06-07

状态：H3.1 已完成并已通过全局主管复核。  
结论：接受为 H3 通用真实 send / 新会话的 `new_session` request、guard、permission envelope、command plan preview 和 no-op runner 非执行产品路径完成；不接受为 H3-B 已授权、真实 Codex session 已创建、真实 `codex exec` 已执行、prompt 已发送、`/Users/yoyi/.codex` 已读写、H2 Phase B 已满足、H3 产品化完成或阶段 H 完成。

## 1. 本轮范围

本轮执行：

- `tasks/2026-06-07-stage-h-h3-1-new-session-request-guard-permission-envelope-and-noop-runner-v1.md`

本轮允许并完成：

- 将 `new_session` 纳入 `codex-local` operation / preview / guard 范围。
- 为 `new_session` 补齐结构化 request 和 command plan preview。
- 明确 `new_session` 不要求 existing `session_id`，但必须绑定 `work_item_id`。
- 在 no-op runner / fake dry-run 中记录 `prompt_sent=false`、`real_codex_executed=false`、`writes_codex_home=false`。
- 在智能体页只读展示 H3.1 新会话预览、guard、command plan 摘要和 no-op runner 状态。
- 让秘书只解释风险和查看建议，不生成新建会话、发送、resume、批准或重试 action proposal。

本轮禁止且未做：

- 未执行真实 `codex exec`。
- 未执行真实 `codex exec resume`。
- 未发送真实 prompt。
- 未创建真实 Codex session。
- 未读写 `/Users/yoyi/.codex`。
- 未读取 auth、token、secret、`.env`、keychain、OAuth、provider credential 或完整 transcript / rollout。
- 未创建 H3-B 真实 fixture。
- 未启动 Tauri / GUI / 截图。
- 未接入 planned adapters 的真实执行。

## 2. 主管复核事实

已复核的关键实现：

- `src-tauri/src/types.rs`：`SessionContinuationRequest.work_item_id` 增加 `#[serde(default)]`，兼容旧数据；`CodexLocalExecutionRequest` 保持 `work_item_id` 绑定。
- `src-tauri/src/codex_local_runner.rs`：`operation_id` 允许 `new_session / send_message / resume`；`new_session` 缺 `work_item_id` 阻断；`new_session` command plan 不带 `resume` 或 session id，prompt 只通过 stdin ref/hash；dry-run attempt 显式记录三项 false。
- `src-tauri/src/session_continuation_store.rs`：continuation no-op / Phase A 路径保持不发送 prompt、不执行真实 Codex、不写 Codex home。
- `src-tauri/src/runtime_session_attention.rs`：运行关注摘要继续区分 no-op / readback unavailable，`result_count` 保持 `null`，不伪装成 0 条结果。
- `src/lib/sessionOperations.ts`：会话操作边界包含 `new_session`，真实实现仍标记为 future task / no-op only。
- `src/lib/sessionContinuation.ts`：codex-local 派生 `new_session / send_message / resume` 三类 preview；`new_session` 不要求 target session，但要求 work item。
- `src/views/AgentView.tsx`：智能体页只读显示 H3.1 no-op、command plan、`prompt_sent=false`、`real_codex_executed=false`、`writes_codex_home=false`。
- `src/lib/secretaryReadModel.ts`：秘书只解释会话继续 / 新会话预览风险，不生成执行或批准 action proposal。
- `tests/offline-permission-dialog.test.tsx`：覆盖 H3.1 operation、preview、guard、UI 文案和误导文案黑名单。

## 3. UI 显示边界复核

H3.1 涉及智能体页读模型展示，已按 `docs/workbench-frontend-display-boundary-v1.md` 和 `docs/plans/task-package-ui-display-boundary-rule-v1.md` 控制：

- 位置：智能体页现有会话继续 / adapter / operation 区域附近。
- 展示：`new_session` preview、guard status、permission envelope 摘要、command plan redacted preview、no-op runner 状态和 readback unavailable 边界。
- 禁止：没有裸“新建会话”执行按钮，没有自由聊天输入框，没有发送 / resume / 重试执行入口，没有真实 Codex 已执行文案。
- planned adapters：继续显示 planned / unavailable / blocked，不因 H3.1 变成可执行。
- readback：`readback_unavailable` 仍是 unavailable，`result_count = null`。
- 秘书：只说明风险和查看建议，不批准、不派发、不重试。

真实 Tauri / 截图验收本轮未做，因为 H3.1 任务包明确禁止 Tauri / GUI / 截图；本轮 UI 验收只接受为离线交互覆盖和 build 通过。

## 4. H2 / H3 边界

H2 Phase B 仍保持：

```text
h2_phase_b_readiness = blocked_waiting_target_session
```

H3-B 仍保持：

```text
h3_b_authorization_request = not_ready
real_codex_execution = not_authorized
codex_home_access = not_authorized
```

H3.1 不能解释为：

- H2 Phase B 已满足。
- H3-B 已授权。
- H3 通用真实 send / 新会话已产品化。
- 新 worker 已真实创建。
- Codex 已收到任务。

## 5. 验证记录

前端验证：

```text
npm run test:offline-interaction
offline interaction tests passed: 12

npm run typecheck
passed

npm run build
passed; only existing Vite chunk-size warning
```

Rust 验证：

```text
cargo test --lib codex_local_runner
8 passed

cargo test --lib session_operation
1 passed

cargo test --lib session_continuation
10 passed

cargo test --lib
245 passed; 1 ignored

rustfmt --check src/codex_local_runner.rs src/lib.rs src/types.rs src/runtime_session_attention.rs src/session_continuation_store.rs
passed
```

已知非阻塞 warning：

- Rust 仍有既有 `JsonRpcError::invalid_params` unused warning。
- Vite build 仍有既有 chunk size warning。

## 6. 扫描记录

最终权威入口同步后补跑扫描，结论写入本 evidence / handoff：

- 旧状态扫描：未发现旧状态完成态冲突。
- 产品源码误导文案扫描：没有把 H3.1 写成真实新会话已创建、真实 send 已执行、prompt 已发送、H3-B 已授权或 H3 已完成。既有 `canvasSurfaceBoundaries.ts` 中的 `Codex 已收到任务` 是 forbidden phrase 常量，不是产品完成态。
- 文档范围内的“真实新会话已创建 / prompt 已发送 / H3-B 已授权 / H3 已完成”等命中仅出现在禁止项、不接受范围或扫描要求语境。

## 7. 接受范围

接受为：

- H3.1 `new_session` request / guard / permission envelope / command plan preview 完成。
- H3.1 no-op runner / fake dry-run 路径完成，且显式记录不发送 prompt、不执行真实 Codex、不写 Codex home。
- 智能体页只读展示 H3.1 新会话边界完成。
- 秘书只读解释边界完成。

不接受为：

- 真实 `codex exec` 已执行。
- 真实 `codex exec resume` 已执行。
- 真实 Codex session 已创建。
- prompt 已发送。
- `/Users/yoyi/.codex` 已读写。
- H2 Phase B 已满足。
- H3-B 已授权或已执行。
- H3 产品化完成。
- H5 项目工作流真实派发完成。
- 阶段 H 完成。
