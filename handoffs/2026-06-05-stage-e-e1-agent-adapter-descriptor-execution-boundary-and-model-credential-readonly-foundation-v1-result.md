# Handoff：Stage E / E1 Agent Adapter Descriptor Execution Boundary And Model Credential Readonly Foundation v1

日期：2026-06-05

## 本轮完成

E1 已完成：工作台现在有统一 adapter descriptor 边界，可在 `WorkbenchSnapshot.agent_adapters[]` 中同时表达：

- `codex-local`：唯一可用 adapter descriptor，真实高风险动作仍需要用户确认。
- `claude-code`、`openclaw`、`opencode`、`opencode-like`：planned / 当前不可执行 / 未配置凭据 / 模型未验证。

Agent 页只在既有“智能体”入口的 adapter 能力面板展示这些只读状态；未新增入口、tab 或右侧顶级面板。planned adapters 没有可点击执行按钮，`implemented_action_kinds` 为空。

## 接受范围

接受为：

- 阶段 E / E1 adapter descriptor 执行边界完成。
- 模型 / 凭据只读状态底座完成。
- descriptor 和真实执行能力已在类型、后端 read model、Agent UI、秘书读模型和测试中分离。

不接受为：

- Claude Code / OpenClaw / OpenCode 真实接入完成。
- 外部模型或凭据管理完成。
- 任何会话操作能力完成。
- 真实 worker / Codex 执行完成。
- 阶段 G 真实 Tauri 全面验收完成。

## 验证结果

通过：

- `npm run typecheck`
- `npm run test:offline-interaction`：`offline interaction tests passed: 9`
- `npm run build`：通过，保留既有 Vite chunk size warning
- `cargo test --lib adapter_descriptor`：2 passed
- `cargo test --lib agent_adapter`：2 passed
- `cargo test --lib`：221 passed，1 ignored
- `rustfmt --check src/types.rs src/lib.rs src/commands.rs`
- 禁止文案扫描无命中

Rust 测试仍有既有 `JsonRpcError::invalid_params` dead_code warning，本轮未处理。

## 当前权威入口

- `CURRENT.md`
- `AUTHORITY.md`
- `README.md`
- `STAGE_PLAN.md`
- `tasks/README.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- 本任务包：`tasks/2026-06-05-stage-e-e1-agent-adapter-descriptor-execution-boundary-and-model-credential-readonly-foundation-v1.md`
- Evidence：`evidence/2026-06-05-stage-e-e1-agent-adapter-descriptor-execution-boundary-and-model-credential-readonly-foundation-v1.md`

## 后续建议

E2 建议二选一拆小任务：

1. 会话操作边界设计：逐项定义发消息、停止、重启、resume、导出、删除、收藏的权限、审计、UI 和真实执行条件。
2. 模型 / 凭据只读状态深化：先定义设置 / 管理入口、安全摘要和不可见 secret 边界，再考虑任何真实凭据接入。

下一步仍不能直接接 Claude Code / OpenClaw / OpenCode，也不能把 planned descriptor 改成可执行。

## 边界声明

本轮没有读写 `/Users/yoyi/.codex`，没有读取 auth/token/`.env`/keychain/OAuth/provider credential/完整 transcript，没有执行 `codex exec` 或 `codex exec resume`，没有调用外部 agent 或模型 provider，没有新增 store 或迁移数据库，没有修改 workflow state JSON，没有启动真实 worker / workflow machine / MCP canvas run。
