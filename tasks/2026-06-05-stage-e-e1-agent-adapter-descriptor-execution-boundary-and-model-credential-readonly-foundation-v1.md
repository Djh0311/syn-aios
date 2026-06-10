# Task Package：Stage E / E1 Agent Adapter Descriptor Execution Boundary And Model Credential Readonly Foundation v1

状态：已完成。  
用途：进入阶段 E 的第一刀，把现有 `codex-local` adapter 能力声明扩展为可承载 Claude Code / OpenClaw / OpenCode / OpenCode-like agent 的只读 descriptor 边界，同时建立模型 / 凭据 / 权限 / 真实执行能力的最小只读边界。  
执行方式：小批次实现，优先复用既有 `WorkbenchSnapshot.agent_adapters[]`；不新增持久 store，不接真实外部 agent，不调用外部模型，不读写 `/Users/yoyi/.codex`。

完成记录：本轮已执行 E1，实现只读 descriptor / 执行边界 / 模型凭据只读状态底座；未接真实外部 agent，未调用外部模型，未读写 `/Users/yoyi/.codex`，未新增 store，未迁移数据库。

回收记录：

- `evidence/2026-06-05-stage-e-e1-agent-adapter-descriptor-execution-boundary-and-model-credential-readonly-foundation-v1.md`
- `handoffs/2026-06-05-stage-e-e1-agent-adapter-descriptor-execution-boundary-and-model-credential-readonly-foundation-v1-result.md`

## 0. 先说薄弱点

- 现有 `agent_adapters[]` 已有 `codex-local` 后端 descriptor，但 Claude Code / OpenClaw / OpenCode 仍只是后续方向，不能显示成已接入。
- UI 很容易把“descriptor 存在”误读成“adapter 可运行”；E1 必须把能力声明和真实执行能力分开。
- 模型 / 凭据状态如果没有严格边界，容易滑向读取 secret、调用外部 provider 或显示敏感信息；E1 只允许只读状态和不可用原因，不允许凭据接入。
- 阶段 G 的真实 Tauri 全面验收仍后置，不能让 E1 顺手补截图或真机验收。
- Odysseus 研究资料暂不并入中间版本计划；E1 不吸收 Odysseus 改动点。

## 1. 已知事实 / 未知 / 假设

已知事实：

- M13 已完成，最终结论为 `accepted_with_deferred_items`；下一步进入阶段 E。
- `tasks/2026-06-03-agent-adapter-backend-capability-read-model-v1.md` 已完成，后端 `WorkbenchSnapshot.agent_adapters[]` 输出 `codex-local` descriptor。
- `codex-local` descriptor 是后端读模型，不是持久事实源。
- 现有 UI 已有智能体入口和 adapter 能力面板，未实现 adapter 不应出现可点击执行按钮。
- `docs/plans/task-package-ui-display-boundary-rule-v1.md` 要求涉及 UI / 读模型 / 文案的任务包必须写清 UI 显示边界。

未知：

- Claude Code / OpenClaw / OpenCode 未来真实接入方式、CLI 参数、会话目录、权限模型和运行日志格式。
- 模型 / 凭据最终是否由设置页、管理页、系统 keychain 或独立 credential store 承载。
- 多 agent 真实执行是否复用现有 workflow machine，还是进入新的 adapter runner。

本任务采用的假设：

- E1 只做只读 descriptor 和边界，不做真实接入。
- E1 可以扩展 `AgentAdapterDescriptor` 的字段或派生语义，但不新增持久 sidecar。
- 模型 / 凭据只读状态第一版可以是 `not_configured`、`not_supported`、`unknown`、`local_only` 等非敏感摘要，不读取 token、不显示路径大表。
- 如果实现者发现必须新增 store、读写真实凭据或调用外部 agent，必须停下并拆 E2 / E1.x，不得在 E1 中直接实现。

## 2. 任务目标

完成阶段 E 第一刀底座：

```text
existing codex-local backend descriptor
-> adapter descriptor taxonomy for planned agents
-> execution capability boundary
-> model / credential readonly status boundary
-> Agent UI / optional management summary only shows safe unavailable reasons
-> evidence + handoff
```

E1 完成后可以说：

- 工作台已有统一 adapter descriptor 边界，可区分 `codex-local`、Claude Code、OpenClaw、OpenCode / OpenCode-like agent 的计划状态。
- UI 可以只读展示 adapter 状态、不可用原因、能力声明边界、模型 / 凭据只读状态。
- descriptor 存在不等于真实 agent 已接入，真实执行能力仍由后续任务单独实现。

E1 完成后仍不能说：

- Claude Code / OpenClaw / OpenCode 已真实接入。
- 外部模型、凭据、OAuth、keychain 或 provider 调用已接通。
- 可以向未实现 adapter 发消息、resume、stop、restart、delete、export 或 dispatch。
- 阶段 G 真实 Tauri 全面验收、运行日志、自动重试或运维诊断完成。
- Odysseus 研究内容已融入产品计划。

## 3. 必须先读

当前入口：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `docs/plans/middleware-version-stage-plan-v1.md`

UI 边界：

- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`

前置 adapter / 会话记录：

- `tasks/2026-06-03-agent-adapter-backend-capability-read-model-v1.md`
- `evidence/2026-06-03-agent-adapter-backend-capability-read-model-v1.md`
- `handoffs/2026-06-03-agent-adapter-backend-capability-read-model-v1-result.md`
- `tasks/2026-06-03-session-center-foundation-hardening-v1.md`
- `evidence/2026-06-03-session-center-foundation-hardening-v1.md`
- `handoffs/2026-06-03-session-center-foundation-hardening-v1-result.md`
- `evidence/2026-06-03-final-skeleton-12-adapter-capability-registry-v1.md`
- `handoffs/2026-06-03-final-skeleton-12-adapter-capability-registry-v1-result.md`

主要代码入口：

- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/adapterCapabilities.ts`
- `prototypes/productized-desktop-shell/src/views/AgentView.tsx`
- `prototypes/productized-desktop-shell/src/lib/secretaryReadModel.ts`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

搜索固定文本必须用 `rg -F '...'` 或单引号，避免 shell 反引号命令替换。

## 4. 范围

允许：

- 复用并扩展现有 `AgentAdapterDescriptor` / `AdapterCapability` / `WorkbenchSnapshot.agent_adapters[]`。
- 为 planned adapters 增加只读 descriptor 语义，例如 `claude-code`、`openclaw`、`opencode`、`opencode-like`。
- planned adapter 的状态只能是 `not_connected`、`not_configured`、`planned`、`blocked` 或同义不可用状态；不得显示为 `available`。
- 增加只读字段表达：
  - `execution_status`
  - `credential_status`
  - `model_access_status`
  - `permission_boundary`
  - `unavailable_reason`
  - `requires_user_setup`
  - `source_kind`
- 让 `codex-local` 继续作为唯一可用 adapter descriptor，但不改变真实执行语义。
- Agent 页在既有 adapter 能力面板中显示只读状态和不可用原因。
- 可选：管理入口健康摘要显示 adapter / model / credential 只读健康状态，但不得新增右侧顶级入口。
- 更新秘书只读模型，让它能把 planned adapter 不可用状态整理成提醒 / 风险，而不是执行建议。
- 更新 TypeScript 类型、离线测试和 Rust 单元测试。
- 新增 E1 evidence / handoff。
- E1 完成后同步 `CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md` 和阶段计划。

禁止：

- 不接 Claude Code / OpenClaw / OpenCode 真实实现。
- 不执行任何外部 agent 命令。
- 不执行 `codex exec`。
- 不执行 `codex exec resume`.
- 不启动真实 worker。
- 不启动 workflow machine。
- 不调用外部模型 provider。
- 不读取 token、secret、keychain、OAuth session 或 provider 凭据。
- 不读写 `/Users/yoyi/.codex`。
- 不读取真实完整 transcript。
- 不新增 credential store。
- 不新增 adapter sidecar。
- 不迁移数据库。
- 不改 `workflow-state.v0.json` 顶层结构。
- 不改真实 Codex 执行语义。
- 不显示未实现 adapter 的可点击执行按钮。
- 不新增一级入口、右侧顶级入口或项目页 tab。
- 不把 descriptor、模型状态或凭据状态写成正式事实或正式记忆。
- 不把 Odysseus 研究项并入本任务实现。

如果执行者认为必须做任一禁止项才能完成 E1，必须停下回传并拆后续任务包。

## 5. 建议数据模型

优先复用现有类型；如需扩展字段，建议保持最小：

```text
AgentAdapterDescriptor {
  adapter_id,
  agent_type,
  agent_id,
  display_name,
  provider,
  status,
  permission_level,
  source_kind,
  capabilities,
  implemented_action_kinds,
  hidden_unimplemented_adapters,
  warnings,
  execution_status,
  credential_status,
  model_access_status,
  unavailable_reason,
  requires_user_setup
}
```

状态建议：

- `codex-local.status = available` 或现有等价状态。
- `claude-code.status = planned` / `not_configured` / `blocked`，不能是 `available`。
- `openclaw.status = planned` / `not_configured` / `blocked`，不能是 `available`。
- `opencode.status = planned` / `not_configured` / `blocked`，不能是 `available`。
- `opencode-like.status = planned` / `not_configured` / `blocked`，不能是 `available`。

能力建议：

- 对 planned adapters，可以没有 `capabilities`，或全部能力为 `blocked` / `not_implemented`。
- `implemented_action_kinds` 必须为空。
- `warnings` 必须包含：
  - `adapter_descriptor_is_read_model_only`
  - `planned_adapter_not_connected`
  - `no_execution_button`
  - `credential_not_configured`

模型 / 凭据边界建议：

- `credential_status` 只能来自静态配置、空态或后端安全摘要，不能读取真实 token。
- `model_access_status` 只能表达是否已知可用 / 未配置 / 不支持 / 未验证，不能调用 provider。
- 不显示 credential 文件路径大表，不显示 token 名称，不显示环境变量值。

## 6. UI 显示边界确认

本任务是否改前端：

- [ ] 不改前端、不改读模型、不改 UI 文案。
- [x] 改前端类型 / Tauri wrapper，但不新增可见 UI。
- [x] 改读模型摘要或状态显示。
- [x] 改已有页面局部 UI。
- [ ] 新增入口、面板、tab、按钮或确认动作。

已读取：

- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`
- `CURRENT.md`
- `tasks/README.md`

本任务允许显示：

- `codex-local` 当前只读能力声明。
- Claude Code / OpenClaw / OpenCode / OpenCode-like 的 planned / unavailable / not configured 只读状态。
- adapter 不可用原因。
- 模型 / 凭据只读状态：未配置、未验证、不支持、需要后续任务。
- “descriptor 只是能力声明，不代表真实接入”的边界说明。

本任务禁止显示：

- 未实现 adapter 的可点击发消息、启动、resume、stop、restart、dispatch、delete、export、favorite 按钮。
- `Claude Code 已接入`、`OpenClaw 已接入`、`OpenCode 已接入`、`外部模型已可用`、`凭据已配置` 等无证据文案。
- token、secret、keychain、OAuth、provider key、环境变量值或凭据文件路径大表。
- raw adapter JSON、raw workflow state、raw audit、完整日志或数据库路径大表。
- “模型与 Agent”一级入口。

显示位置：

- 一级入口：不新增；继续使用既有 `智能体`。
- 右侧入口：不新增；如要显示健康摘要，只能在既有 `管理` 内部。
- 项目页：不新增 tab，不占用工作流画布主区域。
- 画布：不改画布主区域。
- 记忆入口：不改。
- 知识库入口：不改。
- 智能体入口：可在既有 adapter 能力面板中显示只读状态。
- 管理入口：可选显示只读健康摘要，不显示 raw schema / secret。

中间版本范围：

- 本轮必须落地：adapter descriptor 和执行边界的只读模型。
- 本轮只做读模型 / 摘要：模型 / 凭据状态只能是只读安全摘要。
- 本轮后置：真实 adapter 接入、真实凭据管理、外部模型调用、运行日志、真实 Tauri 全面验收。

后端和数据依赖：

- 需要后端正式读模型：优先复用 `WorkbenchSnapshot.agent_adapters[]`。
- 需要审计 / 日志 / 权限 / 状态机：本轮不新增审计或运行日志体系；真实执行权限后置。
- 不能用假数据伪装：planned adapter 不能显示为可用，模型 / 凭据不能伪装已配置。

UI 文案边界：

- 禁止说：`Claude Code 已接入`、`OpenClaw 已接入`、`OpenCode 已接入`、`外部模型已可用`、`凭据已配置`、`真实 worker 已执行`、`真实 Codex 已执行`、`自动派发已开始`。
- 允许说：`计划中的 adapter`、`尚未接入`、`未配置凭据`、`只读能力声明`、`需要后续授权任务`、`当前不可执行`。

验收：

- 类型检查：如改前端必须 `npm run typecheck`。
- 离线交互测试：如改前端必须 `npm run test:offline-interaction`。
- 构建：如改前端必须 `npm run build`。
- 真实窗口 / 截图验收：本任务不要求完成阶段 G 真机验收；如改可见 UI，尽量做浏览器 / Tauri smoke。未完成必须写入 evidence / handoff。
- 未验收项必须写入 evidence / handoff。

## 7. 建议执行段

### 执行段 A：后端 descriptor 边界

目标：

- 后端 descriptor 能表达 planned adapters，但不暗示可执行。

建议实现：

1. 检查现有 Rust `AgentAdapterDescriptor` / `AdapterCapability` 类型是否足够表达 planned adapter 状态。
2. 如字段不足，最小扩展类型；不要新增 store。
3. 在现有 descriptor 派生 helper 中返回 `codex-local` 和 planned adapter 只读状态。
4. planned adapter 必须带不可用原因和 warnings。
5. 保持 workflow state 结构不变。

验收：

- `WorkbenchSnapshot.agent_adapters[]` 至少能区分 `codex-local` 和 planned adapters。
- planned adapters 不能有可执行 action。
- 无 workflow state / 无 session signal 时仍不崩溃。

### 执行段 B：模型 / 凭据只读状态

目标：

- 建立安全摘要，不读取 secret。

建议实现：

1. 如字段不足，在 descriptor 中加入最小只读状态字段。
2. `codex-local` 可以显示 `local_only` / `not_required` / `unknown` 等安全状态。
3. planned adapters 默认 `credential_status = not_configured` 或 `unknown`。
4. 不访问 keychain、环境变量值、OAuth session 或 provider API。

验收：

- UI 和读模型不泄露任何 secret。
- planned adapters 不因凭据未知而显示为可执行。

### 执行段 C：前端只读展示

目标：

- 用户能理解“规划中 / 未接入 / 不可执行”，而不是看到坏掉的按钮。

建议实现：

1. 更新 TS 类型。
2. Agent 页既有 adapter 能力面板显示只读 planned 状态。
3. 如果已有面板无法容纳，局部调整，不新增入口。
4. 秘书只读模型可以把 planned adapter 不可用整理为提醒，但不能提出执行动作。
5. 可选：管理健康摘要只显示 adapter / model / credential 概览。

验收：

- 未实现 adapter 没有可点击执行按钮。
- 文案明确“未接入 / 计划中 / 需后续授权任务”。
- 不显示敏感凭据。

### 执行段 D：测试和文档回收

目标：

- 用测试证明 descriptor 不等于真实执行。

建议测试覆盖：

- `codex-local` 仍可显示现有能力声明。
- Claude Code / OpenClaw / OpenCode / OpenCode-like 显示为不可用或计划中。
- planned adapters 的 `implemented_action_kinds` 为空。
- planned adapters 的 capabilities 不包含 available execution action。
- UI 不渲染未实现 adapter 的执行按钮。
- 模型 / 凭据只读状态不显示 secret。
- 秘书只读模型不会把 planned adapter 转成可执行 action proposal。

## 8. 验收命令

必须运行或明确说明无法运行原因：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
cargo test --lib adapter_descriptor
cargo test --lib agent_adapter
cargo test --lib
rustfmt --check src/types.rs src/lib.rs src/commands.rs
```

如果执行者修改了其他 Rust 文件，必须纳入 `rustfmt --check`。

如果新增的 Rust 单测无法用 `adapter_descriptor` 或 `agent_adapter` filter 覆盖，必须在 evidence 中写清实际 filter。

必须做禁止文案扫描：

```text
rg -n "Claude Code 已接入|OpenClaw 已接入|OpenCode 已接入|外部模型已可用|凭据已配置|真实 worker 已执行|真实 Codex 已执行|自动派发已开始" prototypes/productized-desktop-shell/src
```

预期：无命中。

## 9. evidence / handoff 要求

E1 完成后必须新增：

- `evidence/2026-06-05-stage-e-e1-agent-adapter-descriptor-execution-boundary-and-model-credential-readonly-foundation-v1.md`
- `handoffs/2026-06-05-stage-e-e1-agent-adapter-descriptor-execution-boundary-and-model-credential-readonly-foundation-v1-result.md`

evidence 必须记录：

- E1 接受为什么。
- descriptor 和真实执行能力如何区分。
- planned adapters 的状态、不可用原因和 UI 表达。
- 模型 / 凭据只读状态来源，以及为什么不泄露 secret。
- 禁止文案扫描结果。
- 验证命令和结果。
- 是否完成真实窗口 / 截图验收；如未完成，写清不接受为阶段 G 验收。
- 边界：未接真实 Claude Code / OpenClaw / OpenCode，未调用外部模型，未读写 `/Users/yoyi/.codex`，未新增 store，未执行 worker / Codex。

handoff 必须写清：

- E1 接受为什么。
- E1 不接受为什么。
- 后续建议：E2 是会话操作边界、模型凭据只读深化，还是具体 adapter 接入设计。
- 当前权威入口文件。

## 10. Stop 条件

遇到以下情况必须停下回传：

- 需要读写 `/Users/yoyi/.codex`。
- 需要执行 `codex exec` 或 `codex exec resume`。
- 需要调用 Claude Code / OpenClaw / OpenCode / OpenCode-like 真实命令。
- 需要调用外部模型 provider。
- 需要读取 token、secret、keychain、OAuth session 或 provider 凭据。
- 需要新增 credential store、adapter sidecar 或数据库迁移。
- 需要改 `workflow-state.v0.json` 顶层结构。
- 需要启动真实 worker、workflow machine 或自动派发。
- 需要新增一级入口、右侧顶级入口或项目页 tab。
- 需要把未实现 adapter 显示为可执行。
- 需要把 Odysseus 研究点合入当前实现。

## 11. 回收口径

完成后接受为：

- 阶段 E / E1 adapter descriptor 执行边界和模型 / 凭据只读状态底座完成。
- `codex-local`、Claude Code、OpenClaw、OpenCode / OpenCode-like 的 descriptor 状态可区分。
- UI 能安全表达 planned adapters 不可用，不显示未实现执行按钮。
- 模型 / 凭据状态具备最小只读边界。

完成后不接受为：

- Claude Code / OpenClaw / OpenCode 真实接入完成。
- 外部模型或凭据管理完成。
- 发消息、停止、重启、resume、导出、删除、收藏等会话操作完成。
- 真实 worker / Codex 执行完成。
- 阶段 G 真实 Tauri 全面验收完成。
- Odysseus 研究融合完成。

建议下一步：

- E2：会话操作边界设计，逐项拆 `发消息 / 停止 / 重启 / resume / 导出 / 删除 / 收藏` 的权限、审计、UI 和真实执行条件。
- 或 E2：模型 / 凭据只读状态深化，先定义设置 / 管理入口和安全摘要，再考虑任何真实凭据接入。
