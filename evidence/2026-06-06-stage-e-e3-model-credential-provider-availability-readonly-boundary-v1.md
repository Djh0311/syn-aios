# Evidence：Stage E / E3 Model Credential Provider Availability Readonly Boundary v1

日期：2026-06-06

## 结论

已完成 `tasks/2026-06-06-stage-e-e3-model-credential-provider-availability-readonly-boundary-v1.md`。

接受为：

- 阶段 E / E3 模型、凭据和 provider availability 只读边界完成。
- 后端 `WorkbenchSnapshot.provider_availability[]` 已从 `agent_adapters[]` 和 `session_operations[]` 派生 provider / credential / model / external call / cost risk 摘要。
- `codex-local` 与 Claude Code / OpenClaw / OpenCode / OpenCode-like planned adapters 的 provider、凭据、模型、外发和成本状态可区分。
- 智能体页在既有 adapter 能力 / 会话操作边界附近显示只读 provider availability 摘要；未新增一级入口、右侧顶级入口、项目 tab 或可点击配置 / 验证 / 调用按钮。
- 秘书只读模型能解释 provider / 凭据 / 模型风险，并提供查看边界建议；不生成配置凭据、验证模型或调用 provider 的 action proposal。

不接受为：

- 真实 credential store 完成。
- provider token、OAuth、keychain、`.env`、auth 文件或 provider credential 读取 / 验证完成。
- 外部模型调用、provider probe、model probe、credential probe 完成。
- Claude Code / OpenClaw / OpenCode / OpenCode-like 真实接入完成。
- 会话中心真实发消息 / resume / stop / restart / export / delete / favorite 完成。
- provider availability 等同于项目授权、任务授权、用户确认或会话操作能力。
- 阶段 G 真实 Tauri 全面验收、运行日志、自动重试或运维诊断完成。

## 改动文件

- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
  - 新增 `ProviderAvailabilitySummary`。
  - `WorkbenchSnapshot` 新增 `provider_availability`。
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
  - 新增 `derive_provider_availability_summaries`。
  - `build_snapshot_with_session_source` 从 E1 `agent_adapters[]` 和 E2 `session_operations[]` 派生 E3 摘要。
  - 新增 Rust 单测 `provider_availability_summaries_cover_e3_boundary_matrix`。
- `prototypes/productized-desktop-shell/src/lib/types.ts`
  - 新增 provider availability / credential / model / external call / cost risk 类型。
  - `WorkbenchSnapshot` 新增 `provider_availability`。
- `prototypes/productized-desktop-shell/src/lib/providerAvailability.ts`
  - 新增前端 fallback 派生器 `deriveProviderAvailabilitySummaries`。
- `prototypes/productized-desktop-shell/src/views/AgentView.tsx`
  - 新增只读 `ProviderAvailabilityPanel`。
  - 面板无按钮，只展示状态、原因、边界 warning。
- `prototypes/productized-desktop-shell/src/lib/secretaryReadModel.ts`
  - 新增 `provider_availability_boundary` 风险和 `inspect_provider_availability_boundary` 查看建议。
  - 不新增 provider / credential / model action proposal。
- `prototypes/productized-desktop-shell/src/App.tsx`
  - `emptySnapshot` 和 Agent 页传参同步 `provider_availability`。
- `prototypes/productized-desktop-shell/src/styles.css`
  - 增加 provider availability 局部面板样式。
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
  - fixture 新增 `provider_availability`。
  - 新增 E3 离线场景，覆盖矩阵、UI 禁止文案、无按钮和秘书 action proposal 边界。
- 权威入口同步：
  - `CURRENT.md`
  - `tasks/README.md`
  - `AUTHORITY.md`
  - `STAGE_PLAN.md`
  - `README.md`
  - `docs/plans/middleware-version-stage-plan-v1.md`

## 读模型字段

`ProviderAvailabilitySummary` 最终字段：

- `adapter_id`
- `provider_id`
- `provider_label`
- `provider_kind`
- `adapter_status`
- `availability_status`
- `credential_status`
- `model_status`
- `external_call_status`
- `cost_risk_status`
- `user_visible_reason`
- `safe_to_display`
- `requires_user_configuration`
- `requires_future_task`
- `warnings`

字段来源：

- `adapter_id`、`provider_id`、`provider_label`、`adapter_status` 来自 E1 `AgentAdapterDescriptor`。
- `requires_future_task` 结合 E2 `SessionOperationDescriptor` 的不可执行 / planned / future task 状态派生。
- credential / model / external call / cost 状态全部是静态安全读模型，不读取本机 secret，不调用 provider。

## 状态矩阵

| adapter | provider_id | provider_kind | availability | credential | model | external call | cost risk |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `codex-local` | `local-codex-cli` | `local_cli` | `available_readonly`；无 Codex signal 时可降为 `not_connected` | `not_required_by_workbench` | `local_cli_managed` | `not_needed_for_readonly` | `unknown` |
| `claude-code` | `anthropic-cli-planned` | `external_cli_planned` | `planned` | `credential_missing` | `model_unverified` | `external_call_blocked` | `blocked_until_authorized` |
| `openclaw` | `openclaw-planned` | `external_agent_planned` | `planned` | `credential_missing` | `model_unverified` | `external_call_blocked` | `blocked_until_authorized` |
| `opencode` | `opencode-planned` | `external_cli_planned` | `planned` | `credential_missing` | `model_unverified` | `external_call_blocked` | `blocked_until_authorized` |
| `opencode-like` | `opencode-compatible-planned` | `compatible_adapter_planned` | `planned` | `credential_missing` | `model_unverified` | `external_call_blocked` | `blocked_until_authorized` |

所有 summary 默认包含：

- `provider_availability_read_model_only`
- `credential_secret_not_read`
- `model_not_verified`
- `cost_not_estimated`
- `provider_availability_not_project_authorization`
- `no_external_provider_call_in_e3`

planned adapters 额外包含：

- `planned_adapter_not_connected`
- `external_call_blocked`

## 边界说明

没有读取 secret / `.env` / keychain / OAuth / provider credential / auth：

- E3 新增代码只从 `AgentAdapterDescriptor` 和 `SessionOperationDescriptor` 派生字符串状态。
- 新增窄扫描：
  - `read_to_string(...auth|token|secret|.env|keychain|oauth|credential)`：无 E3 命中。
  - `std::env::var` 命中为既有 `HOME` / `CODEX_WORKBENCH_DATE_PREFIX`，不是凭据读取。
  - `Command::new(...claude|openclaw|opencode)`、provider/model/credential probe：无命中。
- `rg` 对 `token|secret|api_key|oauth|keychain|.env|provider credential|auth` 有大量历史命中，主要来自 `authorization` 字段、记忆敏感级别、任务包 token 估算、既有禁止文案和测试断言。E3 新增命中只包括 warning 名、只读 UI 边界文案和测试断言，不包含真实 secret 值或读取实现。

没有调用外部 provider / 模型：

- E3 未新增任何 runner、adapter command、provider probe 或网络调用。
- `rg` 仍能在 `src-tauri/src/lib.rs` 找到既有 `RealCodexResumeRunner` / `Command::new("codex")`，但 E3 未调用、未改该路径；它仍只属于既有工作流机器 / dispatch 能力，且必须用户确认。
- UI 文案和 warnings 明确 `no_external_provider_call_in_e3`、`external_call_blocked`。

provider availability 不等于授权：

- `provider_availability[]` 不包含 `project_id`、`workflow_id`、authorization id 或 guard result。
- 项目 / 任务授权仍由 `PlanAuthorization`、`AutoDispatchGuardResult` 和工作流控制核心负责。
- 会话操作能力仍由 E2 `session_operations[]` 表达；E3 只说明 provider / model / credential 是否可安全显示和当前边界。

## UI 证据

- 显示位置：智能体页既有 adapter 能力 / 会话操作边界附近。
- 不新增：
  - 一级入口
  - 右侧顶级入口
  - 项目页 tab
  - 消息输入框
  - 配置凭据 / 验证模型 / provider 测试 / 调用模型按钮
- 文案边界：
  - 允许：`只读 provider availability`、`模型未验证`、`外发调用已阻断`、`授权前阻断`、`不等于项目授权`。
  - 禁止扫描无命中：`已配置凭据`、`模型已验证`、`外部模型已可用`、`Claude Code 已接入`、`OpenClaw 已接入`、`OpenCode 已接入`、`provider 已验证`、`测试调用成功`、`真实 Codex 已执行`、`自动派发已开始`。

## 秘书结果

- 新增风险：`provider_availability_boundary`。
- 新增建议：`inspect_provider_availability_boundary`。
- `SecretaryActionProposal` 未新增 provider / credential / model 动作类型。
- 离线测试断言秘书 action proposal 不包含配置凭据、验证模型、调用模型或 provider 动作。

## 验证

在 `prototypes/productized-desktop-shell`：

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，输出 `offline interaction tests passed: 10`。
- `npm run build`：通过；仍有既有 Vite chunk size warning。

在 `prototypes/productized-desktop-shell/src-tauri`：

- `cargo test --lib provider_availability`：通过，1 passed。
- `cargo test --lib adapter_descriptor`：通过，2 passed。
- `cargo test --lib agent_adapter`：通过，2 passed。
- `cargo test --lib session_operation`：通过，1 passed。
- `cargo test --lib`：通过，223 passed，1 ignored。
- `rustfmt --check src/types.rs src/lib.rs src/commands.rs`：通过。

备注：

- Rust 测试仍有既有 `JsonRpcError::invalid_params` dead_code warning，本轮未处理。
- `npm run build` 保留既有 Vite chunk size warning，本轮未处理。

禁止文案扫描：

```text
rg -n '已配置凭据|模型已验证|外部模型已可用|Claude Code 已接入|OpenClaw 已接入|OpenCode 已接入|provider 已验证|测试调用成功|真实 Codex 已执行|自动派发已开始' prototypes/productized-desktop-shell/src
```

结果：无命中；`rg` 以 no-match 退出。

敏感词 / secret 泄露扫描：

```text
rg -n 'token|secret|api[_-]?key|oauth|keychain|\.env|provider credential|auth' prototypes/productized-desktop-shell/src prototypes/productized-desktop-shell/src-tauri/src
```

结果：有历史和边界文案命中。按 `rg --count`，主要命中包括 `src-tauri/src/lib.rs`、`plan_authorization_store.rs`、`types.rs` / `types.ts`、`ProjectsView.tsx` 等既有授权、记忆敏感级别、token 估算和禁止文案。E3 新增命中集中在：

- `src/lib/providerAvailability.ts`：warning 名和状态名。
- `src/views/AgentView.tsx`：只读边界文案“不读取 secret”。
- `src/lib/secretaryReadModel.ts`：秘书只读模型命名。
- `src-tauri/src/lib.rs`：provider availability 派生和测试中的 warning 名。

解释：未发现真实 secret 值、credential store、provider probe、环境变量值输出、keychain/OAuth/auth 文件读取实现。

## 未做验收

- 未启动真实 Tauri 窗口。
- 未做真实窗口 / 截图验收。
- 因此本轮不接受为阶段 G 真实 Tauri 全面验收完成。

## 边界确认

本轮没有：

- 读写 `/Users/yoyi/.codex`。
- 读取 auth、token、`.env`、keychain、OAuth session、provider credential 或完整真实 transcript。
- 执行 `codex exec` 或 `codex exec resume`。
- 调用 Claude Code / OpenClaw / OpenCode / OpenCode-like 真实命令。
- 调用外部模型 provider。
- 启动真实 worker、workflow machine 或 MCP canvas run。
- 新增 credential store、adapter sidecar、provider sidecar 或数据库迁移。
- 修改 `workflow-state.v0.json` 顶层结构。
- 写正式事实、正式记忆或正式审计事件。
- 新增一级入口、右侧顶级入口或项目页 tab。
