# Handoff：Stage E / E3 Model Credential Provider Availability Readonly Boundary v1

日期：2026-06-06

## 本轮完成

E3 已完成：工作台现在有 `WorkbenchSnapshot.provider_availability[]`，用于只读表达 provider availability、credential boundary、model verification、external call risk 和 cost risk。

实现方式是保守派生：

- 后端从 E1 `agent_adapters[]` 和 E2 `session_operations[]` 派生 `ProviderAvailabilitySummary`。
- 前端新增同等 fallback `deriveProviderAvailabilitySummaries`。
- 智能体页在既有 adapter 能力 / 会话操作边界附近显示“Provider / 模型 / 凭据边界”只读面板。
- 秘书只读模型新增 provider availability 风险和查看建议，但不生成配置凭据、验证模型或调用 provider 的 action proposal。

## 接受范围

接受为：

- 阶段 E / E3 模型、凭据和 provider availability 只读边界完成。
- `codex-local` 与 planned adapters 的 provider / model / credential / external call / cost risk 状态可区分。
- UI 能安全显示未配置、模型未验证、外发阻断、成本未知或授权前阻断。
- 秘书能解释风险，但不会把风险解释变成可执行 provider / model / credential 动作。

不接受为：

- 真实 credential store 完成。
- provider token / OAuth / keychain / `.env` / auth 文件读取或验证完成。
- 外部模型调用、provider probe、model probe 或 credential probe 完成。
- Claude Code / OpenClaw / OpenCode / OpenCode-like 真实接入完成。
- 会话中心真实发消息 / resume / stop / restart / export / delete / favorite 完成。
- 阶段 G 真实 Tauri 全面验收完成。

## 验证结果

通过：

- `npm run typecheck`
- `npm run test:offline-interaction`：`offline interaction tests passed: 10`
- `npm run build`：通过，保留既有 Vite chunk size warning
- `cargo test --lib provider_availability`：1 passed
- `cargo test --lib adapter_descriptor`：2 passed
- `cargo test --lib agent_adapter`：2 passed
- `cargo test --lib session_operation`：1 passed
- `cargo test --lib`：223 passed，1 ignored
- `rustfmt --check src/types.rs src/lib.rs src/commands.rs`

扫描：

- 禁止误导文案扫描无命中。
- 敏感词扫描有历史和边界文案命中；未发现 E3 新增真实 secret 值、credential store、provider probe、环境变量值输出、keychain/OAuth/auth 文件读取实现。

未完成：

- 真实窗口 / 截图验收未完成，不能作为阶段 G 验收证据。

## 手动测试清单

在应用里测试：

1. 打开桌面壳，进入左侧“智能体”页面。
2. 确认没有新增“模型 / 凭据”一级入口、右侧顶级入口或项目页 tab。
3. 在“适配器能力”和“会话操作边界”附近找到“Provider / 模型 / 凭据边界”面板。
4. 确认 Codex 显示为只读可见 / 本地 CLI 管理 / 工作台不读取凭据 / 只读不需要外发调用 / 成本未估算或未知。
5. 确认 Claude Code、OpenClaw、OpenCode、OpenCode-like 显示为计划中、缺少凭据边界、模型未验证、外发调用已阻断、授权前阻断。
6. 确认面板文案说明 provider availability 不等于项目授权、任务授权或会话操作能力。
7. 确认面板里没有“配置凭据”“验证模型”“测试 provider”“调用模型”“发送消息”“resume”“dispatch”等可点击按钮。
8. 确认页面不出现“已配置凭据”“模型已验证”“外部模型已可用”“Claude Code 已接入”“OpenClaw 已接入”“OpenCode 已接入”“provider 已验证”“测试调用成功”等误导文案。
9. 打开右侧“秘书只读摘要”，确认能看到 provider / 模型 / 凭据边界风险或查看建议。
10. 确认秘书只给查看 / 解释类建议，不出现配置凭据、验证模型或调用 provider 的 action proposal。

## 当前权威入口

- `CURRENT.md`
- `AUTHORITY.md`
- `README.md`
- `STAGE_PLAN.md`
- `tasks/README.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- 本任务包：`tasks/2026-06-06-stage-e-e3-model-credential-provider-availability-readonly-boundary-v1.md`
- Evidence：`evidence/2026-06-06-stage-e-e3-model-credential-provider-availability-readonly-boundary-v1.md`

## 后续建议

下一步建议进入 E4：会话继续协议和权限预览。

E4 应只设计 send / resume 的 target session、project binding、cwd、allowed write roots、sandbox、prompt preview、readback expectation、failure handling、audit impact 和 guard；进入真实发送或 resume 仍需后续单独授权。

## 边界声明

本轮没有读写 `/Users/yoyi/.codex`，没有读取 auth/token/`.env`/keychain/OAuth/provider credential/完整 transcript，没有执行 `codex exec` 或 `codex exec resume`，没有调用外部 agent 或模型 provider，没有新增 store 或迁移数据库，没有修改 workflow state JSON，没有启动真实 worker / workflow machine / MCP canvas run。
