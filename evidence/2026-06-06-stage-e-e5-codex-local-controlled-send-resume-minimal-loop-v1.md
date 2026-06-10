# Evidence：Stage E / E5 Codex-local Controlled Send Resume Minimal Loop v1

日期：2026-06-06

## 1. 结论

E5 已完成 Level A，接受为：

- `codex-local` 受控 send / resume 最小代码路径完成。
- E4 preview / guard 可进入 E5 user confirmation record。
- 工作台自有 continuation sidecar `session-continuations.v1.json` 已落地。
- continuation / attempt / audit / readback unavailable placeholder 已形成闭环。
- Level A stub runner 已落地，且记录 `prompt_sent=false`、`real_codex_executed=false`、`writes_codex_home=false`。
- 智能体页在既有 `智能体` 页面内显示 E5 Level A 受控 continuation 状态。
- 秘书只读模型可解释 E5 状态，但不生成发送、resume、批准或重试 action proposal。

E5 不接受为：

- 真实 `codex exec resume` 已执行。
- 真实 prompt 已发送。
- Codex 已收到任务。
- 真实 readback 已完成。
- 真实会话继续已验收。
- 读写 `/Users/yoyi/.codex` 已授权或已执行。
- 阶段 G 真实 Tauri 全面验收完成。

本轮执行级别：Level A。没有收到 Level B 对具体 session、cwd、prompt、`.codex` 范围、回滚和证据的授权。

## 2. 边界记录

- 未执行真实 `codex exec`。
- 未执行真实 `codex exec resume`。
- 未发送真实 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未读取 auth、token、`.env`、完整 transcript、keychain、OAuth 或 provider credential。
- 未迁移数据库。
- 未改 `workflow-state.v0.json` 顶层结构或状态枚举。
- 未支持 planned adapters 的 send / resume。
- 未新增自由聊天输入框、一级入口、右侧顶级入口或项目页 tab。

E4 偏差记录已复核：历史偏差来自 shell 双引号内反引号命令替换。本轮所有反引号相关扫描均使用单引号命令，没有发生命令替换偏差。

## 3. 数据模型和 sidecar

新增后端 store：

- 文件：`prototypes/productized-desktop-shell/src-tauri/src/session_continuation_store.rs`
- sidecar 路径：`<workflow_state_dir>/session-continuations.v1.json`
- lock：`<workflow_state_dir>/.session-continuations.v1.lock`
- backup：`<workflow_state_dir>/backups/session-continuations.v1.<timestamp>.<revision>.json`
- atomic write：先写临时文件、`sync_all`、再 `rename` 替换 sidecar。
- corrupt JSON：`load_store` 解析失败时拒绝覆盖。
- revision conflict：`expected_store_revision` 不匹配时拒绝写入。

`SessionContinuationStoreV1` 字段：

- `schema_version`
- `store_version`
- `storage_kind`
- `scope`
- `revision`
- `last_write_id`
- `generated_by`
- `created_at`
- `updated_at`
- `continuations[]`
- `attempts[]`
- `audit_events[]`
- `warnings[]`

核心记录：

- `ControlledSessionContinuation`
- `SessionContinuationAttempt`
- `SessionContinuationReadbackSummary`
- `SessionContinuationAuditEvent`

Level A attempt 固定保持：

- `execution_level = level_a_stub_only`
- `runner_kind = stub`
- `prompt_sent = false`
- `real_codex_executed = false`
- `writes_codex_home = false`
- `readback_summary.status = readback_unavailable` 或 `not_attempted_stub`
- `readback_summary.result_count = null`

## 4. E4 Preview 到 E5 Attempt

新增 Tauri commands：

- `load_session_continuation_store`
- `confirm_controlled_session_continuation`
- `run_controlled_session_continuation_stub`

流程：

1. E4 `SessionContinuationPreview` 保持只读 preview / guard。
2. `confirm_controlled_session_continuation` 接收 preview 和用户确认信息。
3. store 重新检查 adapter、operation、project / workflow / node / session binding、cwd、allowed roots、readback strategy 和 guard status。
4. planned adapter、blocked guard、缺 binding、cwd 越界、敏感路径或缺 readback strategy 都不能创建 runnable continuation。
5. `run_controlled_session_continuation_stub` 只创建 Level A stub attempt，不调用真实 Codex。
6. readback unavailable 作为边界状态记录，不写正式事实、正式记忆、observation 或真实 readback。

runner 边界：

- E5 新增的是 stub runner / store abstraction。
- 没有新增 E5 `Command::new("codex")` 真实 runner。
- `command_preview` 只是预览字符串，用于未来 Level B 审批前展示；本轮不可执行。

## 5. UI 和秘书

UI 位置：

- 仍使用既有 `智能体` 页面。
- 不新增一级入口。
- 不新增右侧顶级入口。
- 不新增项目页 tab。
- 不改项目工作流画布主区域。

前端落地：

- `WorkbenchSnapshot.session_continuation_store`
- `SessionContinuationStoreV1` / continuation / attempt / audit TS 类型
- `AgentView` 增加 “受控 continuation / E5 Level A” 只读面板
- 面板显示 `stub 验收`、`真实执行未授权`、`readback unavailable`、`prompt_sent:false`、`real_codex_executed:false`、`writes_codex_home:false`
- 面板没有真实发送、resume、执行或重试按钮

秘书落地：

- 新增 `controlled_session_continuation_boundary` risk
- 新增 `inspect_controlled_session_continuation` suggestion
- action proposals 仍不包含发送、发消息、resume、批准或重试。

## 6. 验证命令

前端：

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，11 scenarios passed。
- `npm run build`：通过；有既有 Vite chunk-size warning。

Rust：

- `cargo test --lib session_continuation`：通过，5 passed。
- `cargo test --lib controlled_session_continuation`：通过，1 passed。
- `cargo test --lib session_continuation_store`：通过，4 passed。
- `cargo test --lib session_operation`：通过，1 passed。
- `cargo test --lib provider_availability`：通过，1 passed。
- `cargo test --lib workflow_authorization`：通过，1 passed。
- `cargo test --lib`：通过，228 passed，1 ignored。
- `rustfmt --check src/types.rs src/lib.rs src/control_core.rs src/commands.rs src/session_continuation_store.rs`：通过。

Rust 测试有既有 warning：

- `JsonRpcError::invalid_params` dead code warning，非本轮新增失败项。

## 7. 扫描

禁止误导文案扫描：

```text
rg -n '已发送|已 resume|Codex 已收到任务|真实 Codex 已执行|worker 执行中|readback 已完成|Claude Code 可继续会话|OpenClaw 可 resume|OpenCode 已支持发送|自动派发已开始' prototypes/productized-desktop-shell/src
```

结果：无命中，exit code 1。

真实执行 / 敏感路径扫描：

```text
rg -n 'Command::new\("codex"\)|codex exec resume|\.codex|read_to_string\(.*auth|read_to_string\(.*token|read_to_string\(.*secret|read_to_string\(.*\.env|keychain|oauth|provider credential' prototypes/productized-desktop-shell/src-tauri/src prototypes/productized-desktop-shell/src
```

结果：有历史 / 边界命中。分类：

- 既有真实 workflow runner：`src-tauri/src/lib.rs` 中历史 `Command::new("codex")` workflow dispatch runner；E5 没有新增或调用。
- 既有 MCP runner：`src-tauri/src/mcp/codex_runner.rs`；E5 没有触碰。
- 既有 session center / transcript 路径和 fixture `.codex` 字符串；本轮没有读取真实 `/Users/yoyi/.codex`。
- E5 新增合理命中：`src-tauri/src/session_continuation_store.rs` 的 `command_preview` 字符串，仅作为 Level B 前置审批展示；不是 `Command::new("codex")`。
- E5 新增合理命中：`src-tauri/src/session_continuation_store.rs` 的敏感路径 guard，用于阻断 `.codex`、`.env`、auth/token/secret/keychain/OAuth/provider credential 等路径。

Shell 安全扫描：

```text
rg -n '``|`codex exec resume`|`codex exec`' tasks evidence handoffs CURRENT.md tasks/README.md AUTHORITY.md STAGE_PLAN.md README.md docs/plans
```

结果：有大量历史 Markdown code fence 和 backticked command 命中。命令使用单引号执行，没有发生 shell 反引号命令替换偏差。

## 8. 真实窗口验收

本轮未启动真实 Tauri 窗口、未做截图验收。原因：E5 Level A 的验收重点是 store / command / read model / offline UI 边界；真实窗口和截图不能据此接受为阶段 G。

## 9. 修改文件

代码：

- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/session_continuation_store.rs`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `prototypes/productized-desktop-shell/src/views/AgentView.tsx`
- `prototypes/productized-desktop-shell/src/lib/secretaryReadModel.ts`
- `prototypes/productized-desktop-shell/src/App.tsx`
- `prototypes/productized-desktop-shell/src/styles.css`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

文档：

- `tasks/2026-06-06-stage-e-e5-codex-local-controlled-send-resume-minimal-loop-v1.md`
- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `docs/plans/2026-06-06-stage-e-f-g-refinement-plan-v1.md`
- `evidence/2026-06-06-stage-e-e5-codex-local-controlled-send-resume-minimal-loop-v1.md`
- `handoffs/2026-06-06-stage-e-e5-codex-local-controlled-send-resume-minimal-loop-v1-result.md`
