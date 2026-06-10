# Task Package：Stage E / E3 Model Credential Provider Availability Readonly Boundary v1

状态：已完成。  
用途：在 E1 adapter descriptor 和 E2 session operation boundary 之后，把模型、凭据、provider availability、外发风险、成本风险和不可读 secret 的只读边界定成机器可测的读模型和 UI 表达。  
执行方式：小切片实现；优先复用 `WorkbenchSnapshot.agent_adapters[]` 和 E1 / E2 已有只读模型；不新增真实 credential store，不读取或验证真实 provider token，不调用外部模型，不读写 `/Users/yoyi/.codex`。

## 0. 先说薄弱点

- E1 已经有模型 / 凭据只读状态底座，但那只是 adapter descriptor 的基础字段；E3 必须把 provider availability、credential boundary、model verification、external call 和 cost risk 拆成更清楚的只读合同。
- UI 很容易把“provider 已登记”误解成“模型可用 / 凭据已配置”；E3 必须把 `known_provider`、`not_configured`、`not_verified`、`external_call_blocked` 这类状态分开。
- 凭据状态最危险：不能读取 token、secret、`.env`、keychain、OAuth、provider credential，也不能通过 provider probe 变相验证真实凭据。
- planned adapters 仍不可执行，E3 不能为了显示 provider availability 而把 Claude Code / OpenClaw / OpenCode / OpenCode-like 写成可运行。
- 秘书只能解释风险，不能提出“去配置凭据 / 调用模型 / 发送测试请求”的 action proposal。
- GEPA / Paseo / Odysseus 研究资料仍只作为蓝图参考；E3 不吸收优化器、daemon、workspace 复刻或外部项目融合点。

## 1. 已知事实 / 未知 / 假设

已知事实：

- 阶段 C / C1-C6 已完成，接受为受控自动化工作流闭环。
- 阶段 D / M1-M13 已完成，M13 结论为 `accepted_with_deferred_items`。
- 阶段 E / E1 已完成，`WorkbenchSnapshot.agent_adapters[]` 可区分 `codex-local` 和 planned adapters。
- 阶段 E / E2 已完成，会话操作 `send_message` / `stop` / `restart` / `resume` / `export` / `delete` / `favorite` 仍只是 operation boundary，不是真实执行。
- `codex-local` 仍是唯一可用 adapter descriptor；Claude Code / OpenClaw / OpenCode / OpenCode-like 当前仍是 planned / unavailable。
- 当前仍禁止默认执行 `codex exec` / `codex exec resume`，也禁止默认读写 `/Users/yoyi/.codex`。
- `docs/plans/task-package-ui-display-boundary-rule-v1.md` 要求涉及 UI / 读模型 / 文案的任务包必须写清 UI 显示边界。

未知：

- 模型 / 凭据最终是否由设置页、管理页、系统 keychain、独立 credential store、adapter 自报或用户手动标记承载。
- provider availability 第一版是否放进后端 `WorkbenchSnapshot`、前端 helper 派生，还是单独 Rust read model。
- `codex-local` 的模型状态是否能安全表达真实模型名；如果模型名来源不稳定或可能来自敏感配置，必须降级成 `unknown` / `local_cli_managed`。
- 成本风险第一版是否只做静态 risk label，还是需要后续接真实用量和预算模型。

本任务采用的假设：

- E3 只做只读 availability / boundary / risk 摘要，不做真实凭据管理。
- E3 可以新增或扩展 `ModelProviderDescriptor` / `CredentialBoundaryDescriptor` / `ProviderAvailabilitySummary` 或等价类型，但必须保持读模型性质。
- provider / credential / model 状态可以来自静态 descriptor、planned adapter 状态、已知本地 CLI 管理状态和安全默认值；不能来自读取 secret 或调用 provider。
- 如果实现者发现必须新增 credential store、读取真实凭据、调用外部 provider、跑真实 model probe 或写 `/Users/yoyi/.codex` 才能完成，必须停下并拆后续任务。

## 2. 任务目标

完成阶段 E 第三刀底座：

```text
E1 adapter descriptors
-> E2 session operation boundary
-> provider / model / credential readonly descriptors
-> external call and cost risk summary
-> Agent UI and optional Management health readonly display
-> secretary risk explanation without action proposal
-> tests + evidence + handoff
```

E3 完成后可以说：

- 工作台已有模型、凭据、provider availability 的只读边界。
- `codex-local` 与 planned adapters 的 provider / model / credential 状态可区分。
- UI 能安全显示 `available` / `not_configured` / `not_verified` / `credential_missing` / `model_unverified` / `external_call_blocked` 等状态。
- 秘书能解释模型 / 凭据风险，但不会生成配置凭据或调用模型的 action proposal。
- E4 / E5 后续做会话继续协议和受控 send / resume 时，可以复用 E3 的 provider availability guard 输入。

E3 完成后仍不能说：

- 真实 credential store 已完成。
- 外部 provider token 已读取、验证或接入。
- 外部模型调用已完成。
- Claude Code / OpenClaw / OpenCode / OpenCode-like 已真实接入。
- 会话中心真实发消息、resume、stop、restart、export、delete 或 favorite 已完成。
- provider availability 等同于项目授权、任务授权或用户确认。
- 阶段 G 真实 Tauri 全面验收、运行日志、自动重试或运维诊断完成。

## 3. 必须先读

当前入口：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `docs/plans/2026-06-06-stage-e-f-g-refinement-plan-v1.md`

UI 边界：

- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`
- `decisions/2026-05-29-codex-agent-session-center-project-binding-v1.md`

阶段 E 前置：

- `tasks/2026-06-05-stage-e-e1-agent-adapter-descriptor-execution-boundary-and-model-credential-readonly-foundation-v1.md`
- `evidence/2026-06-05-stage-e-e1-agent-adapter-descriptor-execution-boundary-and-model-credential-readonly-foundation-v1.md`
- `handoffs/2026-06-05-stage-e-e1-agent-adapter-descriptor-execution-boundary-and-model-credential-readonly-foundation-v1-result.md`
- `tasks/2026-06-05-stage-e-e2-session-operation-boundary-contract-and-readonly-ui-v1.md`
- `evidence/2026-06-05-stage-e-e2-session-operation-boundary-contract-and-readonly-ui-v1.md`
- `handoffs/2026-06-05-stage-e-e2-session-operation-boundary-contract-and-readonly-ui-v1-result.md`

adapter / 会话前置：

- `tasks/2026-06-03-agent-adapter-backend-capability-read-model-v1.md`
- `evidence/2026-06-03-agent-adapter-backend-capability-read-model-v1.md`
- `handoffs/2026-06-03-agent-adapter-backend-capability-read-model-v1-result.md`
- `tasks/2026-06-03-session-center-foundation-hardening-v1.md`
- `evidence/2026-06-03-session-center-foundation-hardening-v1.md`
- `handoffs/2026-06-03-session-center-foundation-hardening-v1-result.md`

主要代码入口：

- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/adapterCapabilities.ts`
- `prototypes/productized-desktop-shell/src/views/AgentView.tsx`
- `prototypes/productized-desktop-shell/src/components/PermissionDialog.tsx`
- `prototypes/productized-desktop-shell/src/lib/secretaryReadModel.ts`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

搜索固定文本必须用 `rg -F '...'` 或单引号，避免 shell 反引号命令替换。

## 4. 范围

允许：

- 新增或扩展只读类型，例如：
  - `ModelProviderDescriptor`
  - `CredentialBoundaryDescriptor`
  - `ProviderAvailabilitySummary`
  - `ModelAvailabilityDescriptor`
  - `ExternalCallRiskDescriptor`
  - `CostRiskDescriptor`
- 优先从 `AgentAdapterDescriptor`、`SessionOperationDescriptor`、`WorkbenchSnapshot` 派生 provider / model / credential 状态。
- 对 `codex-local` 输出安全状态，例如 `local_cli_managed`、`credential_not_required_by_workbench`、`model_unknown`、`external_call_controlled_by_codex_cli`。
- 对 planned adapters 输出 `planned`、`not_configured`、`not_verified`、`credential_missing`、`model_unverified`、`external_call_blocked` 等状态。
- 显示 provider availability、credential boundary、model verification、external call risk、cost risk、用户需要知道的不可用原因。
- 在智能体页既有 adapter 能力 / 会话操作边界附近增加只读 provider availability 摘要。
- 可选：在管理入口内部健康摘要显示 provider / credential / model 只读健康概览；不得新增右侧顶级入口。
- 更新秘书只读模型，让它能整理模型 / 凭据风险，但不能生成配置凭据、调用模型或 provider probe 的 action proposal。
- 更新 TypeScript 类型、Rust 类型、离线 UI 测试和 Rust 单元测试。
- E3 完成后新增 evidence / handoff，并同步权威入口。

禁止：

- 不新增真实 credential store。
- 不读取、列举或验证真实 token、secret、`.env`、keychain、OAuth session、provider credential、auth 文件内容。
- 不调用外部模型 provider。
- 不做 provider probe、model probe、credential probe 或测试调用。
- 不执行 `codex exec`。
- 不执行 `codex exec resume`。
- 不读写 `/Users/yoyi/.codex`。
- 不启动真实 worker。
- 不启动 workflow machine。
- 不调用 Claude Code / OpenClaw / OpenCode / OpenCode-like 真实命令。
- 不新增 adapter sidecar。
- 不新增 credential sidecar。
- 不新增 provider sidecar。
- 不迁移数据库。
- 不改 `workflow-state.v0.json` 顶层结构。
- 不把 planned adapters 显示为可执行。
- 不显示未实现 adapter 的可点击执行按钮。
- 不新增一级入口、右侧顶级入口或项目页 tab。
- 不把 provider availability 写成项目授权、任务授权、正式事实或正式记忆。
- 不把 GEPA / Paseo / Odysseus 研究项并入本任务实现。

如果执行者认为必须做任一禁止项才能完成 E3，必须停下回传并拆后续任务包。

## 5. Provider / Credential / Model 边界矩阵

E3 必须至少覆盖以下状态，并给出机器可测的结构：

| 对象 | E3 当前允许表达 | 当前禁止表达 | 后续真实实现前置 |
| --- | --- | --- | --- |
| `codex-local` provider | `local_cli_managed` / `known_local_adapter` / `availability_unknown` | 不说“外部模型已验证”，不读取 Codex 凭据 | E5 受控 send / resume、G1 runtime log、G2 diagnostics |
| Claude Code provider | `planned` / `not_connected` / `credential_missing` / `model_unverified` | 不说“Claude Code 已接入”或“凭据已配置” | 独立 adapter 接入设计、credential boundary、用户授权 |
| OpenClaw provider | `planned` / `not_connected` / `external_call_blocked` | 不调用真实命令，不探测模型 | 独立 adapter 接入设计、运行日志、health check |
| OpenCode provider | `planned` / `not_connected` / `model_unverified` | 不显示为可执行 | 独立 adapter 接入设计、session source、权限模型 |
| OpenCode-like provider | `planned` / `not_supported_yet` / `requires_future_adapter` | 不伪造 provider availability | provider taxonomy、adapter contract |
| credential boundary | `not_required_by_workbench` / `not_configured` / `not_readable_by_design` | 不读取或显示 token / secret / env value / keychain item | credential store 设计、备份、脱敏、用户确认 |
| model availability | `unknown` / `not_verified` / `model_unverified` / `external_call_blocked` | 不调用 provider 验证模型 | provider probe 设计、成本预算、失败处理 |
| cost risk | `none_known` / `unknown` / `external_cost_possible` / `blocked_until_authorized` | 不显示真实账单、余额或用量 | 成本统计设计、provider API、用户授权 |

E3 不能把 `provider_known` 算作 `model_available`；也不能把 `credential_missing` 算作系统错误。

## 6. 建议数据模型

优先最小扩展，不新增 store：

```text
ProviderAvailabilitySummary {
  adapter_id,
  provider_id,
  provider_label,
  provider_kind,
  adapter_status,
  availability_status,
  credential_status,
  model_status,
  external_call_status,
  cost_risk_status,
  user_visible_reason,
  safe_to_display,
  requires_user_configuration,
  requires_future_task,
  warnings
}
```

可选拆分：

```text
CredentialBoundaryDescriptor {
  adapter_id,
  provider_id,
  status,
  readable_by_workbench,
  source_kind,
  secret_material_present,
  secret_material_visible,
  validation_status,
  user_visible_reason,
  warnings
}

ModelProviderDescriptor {
  adapter_id,
  provider_id,
  model_status,
  model_label,
  model_label_source,
  verified_by_workbench,
  external_call_required_to_verify,
  warnings
}
```

状态建议：

- `availability_status`: `available_readonly` / `planned` / `not_connected` / `not_configured` / `not_verified` / `blocked` / `unknown`
- `credential_status`: `not_required_by_workbench` / `not_configured` / `not_readable_by_design` / `credential_missing` / `unknown`
- `model_status`: `local_cli_managed` / `not_verified` / `model_unverified` / `unknown` / `blocked`
- `external_call_status`: `not_needed_for_readonly` / `external_call_blocked` / `requires_future_authorization`
- `cost_risk_status`: `none_known` / `unknown` / `external_cost_possible` / `blocked_until_authorized`

warning 建议：

- `provider_availability_read_model_only`
- `credential_secret_not_read`
- `model_not_verified`
- `external_call_blocked`
- `planned_adapter_not_connected`
- `cost_not_estimated`
- `provider_availability_not_project_authorization`

显示要求：

- `secret_material_present` 只能是 `unknown` 或安全默认值，不能通过扫描本机 secret 得出。
- `model_label` 如果可能来自敏感配置或不稳定来源，必须为空或显示 `由本地 CLI 管理` / `未验证`。
- `safe_to_display` 为 false 的记录不能进入普通 UI，只能在 evidence 中解释为什么被过滤。

## 7. UI 显示边界确认

本任务是否改前端：

- [ ] 不改前端、不改读模型、不改 UI 文案。
- [x] 改前端类型 / Tauri wrapper，但不新增可见 UI。
- [x] 改读模型摘要或状态显示。
- [x] 改已有页面局部 UI。
- [ ] 新增入口、面板、tab、按钮或确认动作。

说明：允许新增的是智能体页既有 adapter / operation 区域内部的只读 provider availability 卡片或摘要；可选进入管理内部健康摘要。不允许新增一级入口、右侧顶级入口、项目页 tab、消息输入框或任何可点击执行按钮。

已读取：

- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`
- `CURRENT.md`
- `tasks/README.md`

本任务允许显示：

- `codex-local` 的安全 provider 摘要，例如本地 CLI 管理、凭据不由工作台读取、模型未由工作台验证。
- planned adapters 的 `planned` / `not_connected` / `not_configured` / `credential_missing` / `model_unverified` / `external_call_blocked` 状态。
- provider availability 与项目授权不同的边界说明。
- 外发风险和成本风险的只读摘要。
- “未读取 secret / 未验证模型 / 不调用 provider”的边界说明。

本任务禁止显示：

- `Claude Code 已接入`、`OpenClaw 已接入`、`OpenCode 已接入`、`外部模型已可用`、`凭据已配置`、`模型已验证`、`provider 已验证`。
- token、secret、keychain、OAuth、provider key、环境变量值、auth 文件内容、凭据文件路径大表。
- 可点击的配置凭据、验证模型、测试 provider、发送消息、resume、启动 agent、dispatch、delete、export、favorite 按钮。
- raw adapter JSON、raw workflow state、raw audit、完整日志、数据库路径大表。
- 新的 `模型与 Agent` 一级入口。

显示位置：

- 一级入口：不新增；继续使用既有 `智能体`。
- 右侧入口：不新增；如需摘要，只能在既有 `管理` 内部健康摘要或秘书只读摘要中显示。
- 项目页：不新增 tab，不占用工作流画布主区域。
- 画布：不改画布主区域。
- 记忆入口：不改。
- 知识库入口：不改。
- 智能体入口：可在既有 adapter 能力 / 会话操作边界附近显示只读 provider availability 摘要。
- 管理入口：可选显示只读健康摘要，不显示 raw schema / secret / 路径大表。

中间版本范围：

- 本轮必须落地：模型 / 凭据 / provider availability 只读边界读模型，以及智能体页安全状态展示。
- 本轮只做读模型 / 摘要：availability、credential、model verification、external call、cost risk、不可用原因。
- 本轮后置：真实 credential store、provider probe、外部模型调用、成本统计、真实 adapter 接入、运行日志、真实 Tauri 全面验收。

后端和数据依赖：

- 需要后端正式读模型：优先复用 `WorkbenchSnapshot.agent_adapters[]`；如新增 provider descriptors，应是安全派生读模型。
- 需要审计 / 日志 / 权限 / 状态机：本轮只定义 provider availability 边界，不新增运行日志、真实权限操作或执行审计体系。
- 不能用假数据伪装：不能把静态 planned descriptor 显示成已配置凭据或已验证模型；不能用前端 mock 表现 provider 可用。

UI 文案边界：

- 禁止说：`已配置凭据`、`模型已验证`、`外部模型已可用`、`Claude Code 已接入`、`OpenClaw 已接入`、`OpenCode 已接入`、`provider 已验证`、`测试调用成功`。
- 允许说：`只读状态`、`未配置凭据`、`模型未验证`、`外发调用已阻断`、`计划中`、`需要后续授权任务`、`未读取 secret`、`不等于项目授权`。

验收：

- 类型检查：如改前端必须 `npm run typecheck`。
- 离线交互测试：如改前端必须 `npm run test:offline-interaction`。
- 构建：如改前端必须 `npm run build`。
- 真实窗口 / 截图验收：尽量做浏览器 / Tauri smoke；未完成必须写入 evidence / handoff，且不能接受为阶段 G 验收。
- 未验收项必须写入 evidence / handoff。

## 8. 建议执行段

### 执行段 A：盘点 E1 / E2 现有字段

目标：

- 确认现有 descriptor 是否已经能表达 provider / credential / model 状态，避免重复造 store。

建议实现：

1. 搜索 `AgentAdapterDescriptor`、`SessionOperationDescriptor`、`credential_status`、`model_access_status`、`provider`、`adapterCapabilities`。
2. 列出 E1 / E2 已有状态字段和 E3 缺口。
3. 如果字段足够，优先派生 `ProviderAvailabilitySummary`；如果字段不足，最小扩展类型。
4. 不改变 E2 的会话操作边界，不新增执行按钮。

验收：

- evidence 能说明哪些字段复用 E1 / E2，哪些字段是 E3 新增。
- 没有新增 credential / provider / adapter sidecar。

### 执行段 B：后端只读 availability 读模型

目标：

- 用机器可测结构表达 provider availability 和 secret 不可读边界。

建议实现：

1. 新增或扩展 Rust 类型。
2. 从现有 adapter descriptors 派生 provider availability。
3. `codex-local` 表达为本地 CLI 管理 / 工作台不读取凭据 / 模型未由工作台验证。
4. planned adapters 表达为 planned / credential missing / model unverified / external call blocked。
5. 不读取任何本机 secret，不调用任何 provider。

验收：

- `codex-local` 与 planned adapters 状态可区分。
- planned adapters 不出现 `available_to_execute` 或同义可执行状态。
- 所有 credential descriptors 都不包含 token、secret、env value、keychain item、OAuth 或 provider key。
- provider availability 不影响 C1 / C3 / C4 工作流授权 guard。

### 执行段 C：前端类型和智能体页只读展示

目标：

- 用户能理解“未配置 / 未验证 / 不会外发”，而不是看到已接入错觉。

建议实现：

1. 更新 TypeScript 类型和前端 fallback。
2. 在智能体页既有 adapter 能力或会话操作边界区域附近显示只读 provider availability 摘要。
3. 用短文案说明 `provider availability` 不等于 `project authorization`。
4. 不新增消息输入框、配置按钮、验证按钮或 provider 测试按钮。
5. 不显示 raw schema 或路径大表。

验收：

- UI 显示 planned / unavailable / not configured / not verified。
- UI 不显示任何“已配置 / 已验证 / 已接入”误导。
- UI 没有可点击执行、配置或测试按钮。

### 执行段 D：秘书和管理摘要

目标：

- 秘书能解释风险，管理能可选显示健康摘要，但都不触发动作。

建议实现：

1. 秘书只读模型可增加 provider / credential / model 风险解释。
2. 秘书不能生成配置凭据、验证模型、调用 provider 或发送消息的 action proposal。
3. 管理健康摘要如已有合适入口，可显示 count / severity / brief reason。
4. 不新增右侧顶级入口。

验收：

- 秘书输出的是 `read` / `explain` / `inspect` 级建议，不是 `configure` / `call_model` / `send_message`。
- 管理摘要不显示 secret、raw schema 或路径大表。

### 执行段 E：测试、文案扫描和文档回收

目标：

- 用测试证明 E3 没有接真实凭据或外部模型。

建议测试覆盖：

- provider availability descriptors 至少覆盖 `codex-local` 和 planned adapters。
- `codex-local` 不要求工作台读取凭据。
- planned adapters 显示 `credential_missing` / `model_unverified` / `external_call_blocked` 或等价状态。
- descriptors 不包含 token、secret、`.env` value、keychain、OAuth、provider key。
- provider availability 不等于 project authorization。
- UI 不显示未实现 adapter 的执行 / 配置 / 验证按钮。
- 秘书不生成 credential setup 或 model call action proposal。

## 9. 验收命令

必须运行或明确说明无法运行原因：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
cargo test --lib provider_availability
cargo test --lib adapter_descriptor
cargo test --lib agent_adapter
cargo test --lib session_operation
cargo test --lib
rustfmt --check src/types.rs src/lib.rs src/commands.rs
```

如果执行者修改了其他 Rust 文件，必须纳入 `rustfmt --check`。

如果新增的 Rust 单测无法用 `provider_availability`、`adapter_descriptor` 或 `agent_adapter` filter 覆盖，必须在 evidence 中写清实际 filter。

必须做禁止文案扫描：

```text
rg -n "已配置凭据|模型已验证|外部模型已可用|Claude Code 已接入|OpenClaw 已接入|OpenCode 已接入|provider 已验证|测试调用成功|真实 Codex 已执行|自动派发已开始" prototypes/productized-desktop-shell/src
```

预期：无误导命中。若历史文案合理存在，必须逐条解释为什么不是 E3 新增误导。

必须做敏感词 / secret 泄露扫描：

```text
rg -n "token|secret|api[_-]?key|oauth|keychain|\\.env|provider credential|auth" prototypes/productized-desktop-shell/src prototypes/productized-desktop-shell/src-tauri/src
```

预期：只允许类型名、边界文案、禁止项或测试断言中的安全命中；不得出现真实 secret 值、读取 secret 的实现、provider probe 或环境变量值输出。

## 10. evidence / handoff 要求

E3 完成后必须新增：

- `evidence/2026-06-06-stage-e-e3-model-credential-provider-availability-readonly-boundary-v1.md`
- `handoffs/2026-06-06-stage-e-e3-model-credential-provider-availability-readonly-boundary-v1-result.md`

evidence 必须记录：

- E3 接受为什么。
- provider / model / credential 读模型最终字段或等价结构。
- `codex-local` 和 planned adapters 的最终状态矩阵。
- 为什么没有读取 secret、`.env`、keychain、OAuth、provider credential 或 auth 文件内容。
- 为什么没有调用外部 provider / 模型。
- provider availability 与项目授权 / 任务授权 / 会话操作能力如何区分。
- UI 显示位置和不显示内容。
- 秘书是否生成 action proposal；如没有，写清测试或代码证据。
- 禁止文案扫描结果。
- 敏感词 / secret 泄露扫描结果和合理命中解释。
- 验证命令和结果。
- 是否完成真实窗口 / 截图验收；如未完成，写清不接受为阶段 G 验收。
- 边界：未执行 `codex exec` / `codex exec resume`，未读写 `/Users/yoyi/.codex`，未调用外部 agent / provider，未新增 credential store，未迁移数据库。

handoff 必须写清：

- E3 接受为什么。
- E3 不接受为什么。
- 后续建议：E4 会话继续协议和权限预览。
- 当前权威入口文件。

## 11. Stop 条件

遇到以下情况必须停下回传：

- 需要读写 `/Users/yoyi/.codex`。
- 需要执行 `codex exec` 或 `codex exec resume`。
- 需要调用 Claude Code / OpenClaw / OpenCode / OpenCode-like 真实命令。
- 需要调用外部模型 provider。
- 需要做 provider probe、model probe、credential probe 或测试调用。
- 需要读取 token、secret、`.env`、keychain、OAuth session、provider credential、auth 文件内容或环境变量值。
- 需要读取真实完整 transcript 作为开发证据。
- 需要新增 credential store、adapter sidecar、provider sidecar 或数据库迁移。
- 需要改 `workflow-state.v0.json` 顶层结构。
- 需要启动真实 worker、workflow machine 或自动派发。
- 需要新增一级入口、右侧顶级入口或项目页 tab。
- 需要显示可点击的配置凭据、验证模型、provider 测试、发送消息或会话控制按钮。
- 需要把 provider availability 当作项目授权、任务授权、正式事实或正式记忆。
- 需要把 GEPA / Paseo / Odysseus 研究点合入当前实现。

## 12. 回收口径

完成后接受为：

- 阶段 E / E3 模型、凭据和 provider availability 只读边界完成。
- `codex-local` 与 planned adapters 的 provider / model / credential 状态可区分。
- UI 能安全显示未配置、未验证、外发阻断、成本风险和不可用原因。
- 秘书能解释模型 / 凭据风险，但不生成配置凭据或调用模型的 action proposal。
- 读模型和测试能证明没有读取 secret 或调用外部 provider。

完成后不接受为：

- 真实 credential store 完成。
- 外部 provider token 读取、验证或接入完成。
- 外部模型调用完成。
- Claude Code / OpenClaw / OpenCode / OpenCode-like 真实接入完成。
- 会话中心真实发消息 / resume / stop / restart / export / delete / favorite 完成。
- provider availability 等同于项目授权、任务授权或用户确认。
- 运行日志、自动重试、取消恢复、运维诊断完成。
- 真实 worker / Codex 执行完成。
- 阶段 G 真实 Tauri 全面验收完成。

建议下一步：

- E4：会话继续协议和权限预览，设计 `send_message` / `resume` 的 target session、project binding、cwd、allowed write roots、sandbox、prompt preview、readback expectation、failure handling、audit impact 和 guard；E4 仍不执行真实发送。
