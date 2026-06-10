# Evidence：Stage E / E1 Agent Adapter Descriptor Execution Boundary And Model Credential Readonly Foundation v1

日期：2026-06-05

## 结论

已完成 `tasks/2026-06-05-stage-e-e1-agent-adapter-descriptor-execution-boundary-and-model-credential-readonly-foundation-v1.md`。

接受为：

- 阶段 E / E1 adapter descriptor 执行边界和模型 / 凭据只读状态底座完成。
- 后端 `WorkbenchSnapshot.agent_adapters[]` 可区分 `codex-local`、Claude Code、OpenClaw、OpenCode 和 OpenCode-like 的 descriptor 状态。
- `codex-local` 仍是唯一可用 adapter descriptor；高风险执行能力仍只声明为需要用户确认。
- planned adapters 只显示计划中 / 当前不可执行 / 未配置凭据 / 模型未验证，不显示执行按钮或已实现动作。
- 秘书只读模型把 planned adapter 不可用状态合入 adapter 边界风险 / 提醒，不生成可执行 action proposal。

不接受为：

- Claude Code / OpenClaw / OpenCode / OpenCode-like 已真实接入。
- 外部模型、OAuth、keychain、provider credential 或 credential store 已接通。
- 发消息、resume、stop、restart、dispatch、delete、export、favorite 等会话操作完成。
- 真实 worker / Codex 执行完成。
- 阶段 G 真实 Tauri 全面验收完成。
- Odysseus 研究融合完成。

## 改动文件

- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
  - `AgentAdapterDescriptor` 增加 `execution_status`、`credential_status`、`model_access_status`、`permission_boundary`、`unavailable_reason`、`requires_user_setup`。
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
  - 后端 descriptor 派生从 1 个 `codex-local` 扩展为 `codex-local` + 4 个 planned descriptors。
  - 新增 planned adapter helper，不新增 store，不读取凭据，不调用外部命令。
  - Rust 单测覆盖 planned adapters 不可用、空动作、空能力和未配置凭据。
- `prototypes/productized-desktop-shell/src/lib/types.ts`
  - TS descriptor 类型扩展 planned agent type 和只读状态字段。
- `prototypes/productized-desktop-shell/src/lib/adapterCapabilities.ts`
  - 前端 fallback 同步输出 planned descriptors，保持无后端字段时的只读边界一致。
- `prototypes/productized-desktop-shell/src/views/AgentView.tsx`
  - 既有智能体页 adapter 能力面板显示执行 / 凭据 / 模型只读状态、不可用原因和权限边界。
  - planned adapters 显示“当前不可执行”，`已实现动作：无`，没有操作按钮。
- `prototypes/productized-desktop-shell/src/styles.css`
  - 为 adapter 状态行和 planned adapter 空态补稳定布局。
- `prototypes/productized-desktop-shell/src/lib/secretaryReadModel.ts`
  - adapter warning 优先包含 planned / unavailable descriptor。
  - 新增只读 adapter 边界提醒类型；不生成 adapter 执行 action proposal。
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
  - 更新后端 descriptor fixture，覆盖 Codex + planned adapters。
  - 覆盖 planned adapters 不可用、无 available 执行能力、无已实现动作、未配置凭据，以及 Agent 页无未实现按钮。

## Descriptor 边界

`codex-local`：

- `status = available` 或 `not_connected`，取决于已有 Codex signal。
- `execution_status = available_with_user_confirmation` 或 `not_connected`。
- `credential_status = not_read`。
- `model_access_status = local_read_model_only`。
- `permission_boundary` 明确高风险动作仍必须用户确认；E1 未执行 `codex exec` 或 `codex exec resume`。

planned adapters：

| adapter | status | execution_status | credential_status | model_access_status | actions |
| --- | --- | --- | --- | --- | --- |
| `claude-code` | `planned` | `not_implemented` | `not_configured` | `not_verified` | 空 |
| `openclaw` | `planned` | `not_implemented` | `not_configured` | `not_verified` | 空 |
| `opencode` | `planned` | `not_implemented` | `not_configured` | `not_verified` | 空 |
| `opencode-like` | `planned` | `not_implemented` | `not_configured` | `not_verified` | 空 |

planned adapter warnings 包含：

- `adapter_descriptor_is_read_model_only`
- `planned_adapter_not_connected`
- `no_execution_button`
- `credential_not_configured`
- `model_access_not_verified`

## 模型 / 凭据只读状态来源

- 本轮模型 / 凭据状态全部来自静态读模型字段和空态推导。
- 没有读取 token、secret、keychain、OAuth session、provider credential、`.env` 或任何外部 provider。
- 没有显示 credential 路径大表、环境变量值、token 名称或 secret 摘要。
- planned adapters 的 `credential_status = not_configured` 只表示未配置 / 未接入，不代表已经检查过真实凭据。

## 验证

在 `prototypes/productized-desktop-shell`：

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，输出 `offline interaction tests passed: 9`。
- `npm run build`：通过；仍有既有 Vite chunk size warning。

在 `prototypes/productized-desktop-shell/src-tauri`：

- `cargo test --lib adapter_descriptor`：通过，2 passed；仍有既有 `JsonRpcError::invalid_params` dead_code warning。
- `cargo test --lib agent_adapter`：通过，2 passed；同上 warning。
- `cargo test --lib`：通过，221 passed，1 ignored；同上 warning。
- `rustfmt --check src/types.rs src/lib.rs src/commands.rs`：通过。

禁止文案扫描：

```text
rg -n "Claude Code 已接入|OpenClaw 已接入|OpenCode 已接入|外部模型已可用|凭据已配置|真实 worker 已执行|真实 Codex 已执行|自动派发已开始" prototypes/productized-desktop-shell/src
```

结果：无命中；`rg` 以 no-match 退出。

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
- 新增 credential store、adapter sidecar 或数据库迁移。
- 修改 `workflow-state.v0.json` 顶层结构。
- 写正式事实、正式记忆或正式审计事件。
- 新增一级入口、右侧顶级入口或项目页 tab。
