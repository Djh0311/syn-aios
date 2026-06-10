# Evidence：Stage E / E6 Runtime Session Attention And Readback Failure Boundary v1

日期：2026-06-06

## 1. 结论

E6 已完成，接受为：

- 新增 `WorkbenchSnapshot.runtime_session_attention[]` 最小运行关注读模型。
- 新增 `WorkbenchSnapshot.session_run_status_summaries[]` 会话运行摘要读模型。
- 能区分 `waiting_permission`、`waiting_level_b_authorization`、`running_stub`、`failed_stub`、`timed_out`、`readback_failed`、`readback_unavailable`、`blocked_by_guard`。
- `readback_failed` 与 `readback_unavailable` 都不会显示为真实 0 条读回；`result_count` 保持 `null`。
- 智能体页显示 E6 运行关注摘要。
- 右侧 `运行中`、`通知`、`待办` 在既有入口内显示 E6 摘要，不新增入口。
- 秘书只读模型能解释 E6 风险和查看建议，但不生成批准、发送、resume、重试、stop 或 restart action proposal。

E6 不接受为：

- 真实 `codex exec` 或 `codex exec resume` 已执行。
- 真实 prompt 已发送。
- 真实 readback 已完成。
- 自动重试、stop / restart 或完整 runtime log 完成。
- 阶段 G 真实 Tauri 全面验收完成。

## 2. 边界记录

- 未执行真实 `codex exec`。
- 未执行真实 `codex exec resume`。
- 未发送真实 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未读取 auth、token、`.env`、完整 transcript、keychain、OAuth 或 provider credential。
- 未新增持久 store。
- 未迁移数据库。
- 未改 `workflow-state.v0.json` 顶层结构或状态枚举。
- 未支持 planned adapters 的 send / resume。
- 未新增一级入口、右侧顶级入口、项目页 tab、执行按钮、自动重试按钮、stop / restart 按钮或自由聊天输入框。

## 3. 数据来源

E6 只消费既有读模型和 sidecar：

- E4 `session_continuation_previews[]`：preview guard、waiting permission、blocked by guard。
- E5 `session_continuation_store`：controlled continuation、stub attempt、readback placeholder、audit ref。
- E2 `session_operations[]`：会话操作边界仍不可直接执行。
- E3 `provider_availability[]`：provider / model / credential 只读可用性。
- 既有 workflow read model：右侧运行、通知、待办的项目工作流上下文。

本轮没有读取真实 transcript / rollout 作为开发证据。

## 4. Readback 边界

区分规则：

- `readback_unavailable`：没有真实读取来源，或 Level A stub 没有执行真实 Codex；`result_count=null`。
- `readback_failed`：读回尝试失败或结果不可信；`result_count=null`，不能写成 0。
- `guard_blocked`：guard 阻断时 readback 不发生，只能显示 boundary。
- `level_b_not_authorized` / `not_attempted_stub`：说明仍停在 E5 Level A 或 preview / confirmation 边界。

## 5. UI 和秘书

智能体页：

- 新增 `运行关注 / E6` 面板。
- 显示 attention count、blocking count、needs user、readback unavailable / failed。
- 显示 session summary 和 attention card。
- 不显示 raw log、raw sidecar、raw workflow state、raw audit、完整路径大表或内部 schema。

右侧入口：

- `运行中`：显示 session run status summary 和 readback boundary。
- `通知`：显示阻断 / failed / unavailable 摘要。
- `待办`：显示需要用户查看的 runtime attention。
- 不新增右侧顶级入口，不把通知 / 待办 / 运行中混成一个列表。

秘书：

- 新增 `runtime_session_attention_boundary` risk signal。
- 新增 `inspect_runtime_session_attention` suggestion。
- action proposals 仍不包含批准、发送、resume、重试、stop 或 restart。

## 6. 修改文件

代码：

- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/runtime_session_attention.rs`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/secretaryReadModel.ts`
- `prototypes/productized-desktop-shell/src/views/AgentView.tsx`
- `prototypes/productized-desktop-shell/src/App.tsx`
- `prototypes/productized-desktop-shell/src/styles.css`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

文档：

- `tasks/2026-06-06-stage-e-e6-runtime-session-attention-and-readback-failure-boundary-v1.md`
- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `docs/plans/2026-06-06-stage-e-f-g-refinement-plan-v1.md`
- `evidence/2026-06-06-stage-e-e6-runtime-session-attention-and-readback-failure-boundary-v1.md`
- `handoffs/2026-06-06-stage-e-e6-runtime-session-attention-and-readback-failure-boundary-v1-result.md`

## 7. 验证命令

前端：

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，11 scenarios passed。
- `npm run build`：通过；有既有 Vite chunk-size warning。

Rust：

- `cargo test --lib runtime_session_attention`：通过，2 passed。
- `cargo test --lib session_continuation`：通过，5 passed。
- `cargo test --lib session_operation`：通过，1 passed。
- `cargo test --lib provider_availability`：通过，1 passed。
- `cargo test --lib workflow_authorization`：通过，1 passed。
- `cargo test --lib`：通过，230 passed，1 ignored。
- `rustfmt --check src/types.rs src/lib.rs src/control_core.rs src/commands.rs src/session_continuation_store.rs src/runtime_session_attention.rs`：通过。

Rust 测试有既有 warning：

- `JsonRpcError::invalid_params` dead code warning，非本轮新增失败项。

## 8. 扫描

禁止误导文案扫描：

```text
rg -n '已自动重试|已停止 agent|已重启 agent|真实派发已完成|真实 prompt 已发送|Codex 已收到任务|真实 readback 已完成|readback 0 条|失败已自动恢复|Claude Code 已接管|OpenClaw 已运行|OpenCode 已 resume' prototypes/productized-desktop-shell/src
```

结果：无命中，exit code 1。

真实执行 / 敏感路径扫描：

```text
rg -n 'Command::new\("codex"\)|codex exec resume|\.codex|read_to_string\(.*auth|read_to_string\(.*token|read_to_string\(.*secret|read_to_string\(.*\.env|keychain|oauth|provider credential' prototypes/productized-desktop-shell/src-tauri/src prototypes/productized-desktop-shell/src
```

结果：有历史 / 边界命中。分类：

- 既有真实 workflow runner：`src-tauri/src/lib.rs` 中历史 `Command::new("codex")` workflow dispatch runner；E6 没有新增或调用。
- 既有 MCP runner：`src-tauri/src/mcp/codex_runner.rs`；E6 没有触碰。
- 既有 session center / transcript fixture `.codex` 字符串；本轮没有读取真实 `/Users/yoyi/.codex`。
- 既有 E5 command preview 字符串和敏感路径 guard；E6 没有新增真实 runner。
- E6 新增扫描命中只在 runtime attention 读模型 / UI / 测试标识上，不涉及真实执行或敏感读取。

## 9. 真实窗口验收

本轮未启动真实 Tauri 窗口、未做截图验收。原因：E6 任务目标是读模型 / 摘要 UI / 离线边界验收；真实窗口和截图仍保留给阶段 G。不能接受为阶段 G 真实 Tauri 全面验收完成。
