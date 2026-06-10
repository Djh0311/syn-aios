# Task Package：Stage E / E2 Session Operation Boundary Contract And Readonly UI v1

状态：已完成。  
用途：在 E1 adapter descriptor 只读底座之后，逐项定义会话操作边界，先把 `发消息 / 停止 / 重启 / resume / 导出 / 删除 / 收藏` 的权限、审计、UI、数据写入和后续真实执行条件说清楚。  
执行方式：小切片实现；优先做 operation boundary read model 和智能体页只读 / 禁用态展示；不实现真实会话操作，不新增凭据系统，不执行外部 agent，不读写 `/Users/yoyi/.codex`。

完成记录：本轮已执行 E2，实现 `WorkbenchSnapshot.session_operations[]` 会话操作边界读模型、前端 fallback、智能体页只读“会话操作边界”面板、秘书只读风险 / 建议和测试；未实现真实会话操作，未执行 `codex exec` / `codex exec resume`，未读写 `/Users/yoyi/.codex`，未新增 store，未迁移数据库。

回收记录：

- `evidence/2026-06-05-stage-e-e2-session-operation-boundary-contract-and-readonly-ui-v1.md`
- `handoffs/2026-06-05-stage-e-e2-session-operation-boundary-contract-and-readonly-ui-v1-result.md`

## 0. 先说薄弱点

- 当前会话中心仍明确是只读历史会话浏览器，但 UI 和能力声明里已经出现 `execute-node-dispatch`、`codex exec resume` 等高风险路径；如果不把会话操作逐项拆清，后续很容易把“已有派发能力”误读成“会话中心可以发消息”。
- `stop` / `restart` 需要真实运行进程、运行句柄、超时、重试和日志体系支撑；当前只有历史会话和 workflow dispatch 记录，不能假装能控制正在运行的 agent。
- `delete` / `archive` / `move` 是破坏性操作，可能改写 Codex 原生状态或用户历史资料；E2 只能定义边界，不能实现。
- `export` 和 `favorite` 看起来低风险，但仍涉及完整 transcript、隐私脱敏、导出范围、工作台自有 metadata store 和审计；不能顺手做成无审计按钮。
- planned adapters 已在 E1 出现，但它们没有真实命令、凭据、模型验证或会话来源；E2 不能把 Claude Code / OpenClaw / OpenCode / OpenCode-like 会话操作显示成可执行。
- GEPA 和 Odysseus 研究资料仍是后置参考；E2 不吸收优化器、workspace 复刻或外部项目融合点。

## 1. 已知事实 / 未知 / 假设

已知事实：

- E1 已完成，`WorkbenchSnapshot.agent_adapters[]` 可区分 `codex-local` 和 planned adapters。
- `codex-local` 是唯一可用 adapter descriptor，但高风险动作仍必须用户确认。
- 会话中心底座硬化已完成：sqlite 是会话目录权威，Rust 原生 transcript parser 已接入，会话中心仍是只读历史浏览器。
- 现有 `AgentView` 只提供重新读取和定位 rollout；默认文案仍是“不发送消息、不 resume、不删除、不移动”。
- 现有 workflow 派发路径里存在受控 `codex exec resume`，但那属于项目工作流 / dispatch 语境，不等于会话中心直接发消息。
- UI 任务包必须落实 `docs/plans/task-package-ui-display-boundary-rule-v1.md` 的“UI 显示边界确认”章节。

未知：

- 后续真实 `send message` 是复用 workflow dispatch、直接 `codex exec resume`，还是单独 adapter runner。
- 真实 `stop` 是否能通过 Codex CLI、进程管理、运行日志或外部 adapter API 实现。
- `restart` 是恢复旧会话、新建派生会话，还是重新执行上一条任务。
- `export` 最终是 Markdown、JSON、脱敏包、项目证据包，还是用户手动保存文件。
- `favorite` / `archive` 是工作台自有 metadata，还是需要同步到 Codex 原生会话系统。
- 多 adapter 的会话目录、transcript 格式和权限模型尚未确定。

本任务采用的假设：

- E2 只建立 operation boundary contract，不做真实会话操作。
- E2 可以增加只读读模型、类型、前端摘要、禁用态 UI 和测试。
- E2 不新增持久 store；如果实现者认为必须新增 `session-operations.v1.json`、favorite store、export store 或 credential store，必须停下拆后续任务。
- E2 不读写 `/Users/yoyi/.codex`，不执行 `codex exec` / `codex exec resume`，不调用外部 agent / provider。
- E2 可以把未来操作条件写成 `blocked` / `planned` / `requires_future_task`，但不能出现可点击执行按钮。

## 2. 任务目标

完成阶段 E 第二刀底座：

```text
E1 adapter descriptors
-> session operation boundary matrix
-> per-operation permission / audit / data-effect contract
-> Agent UI readonly or disabled operation boundary display
-> secretary / management readonly risk summary if there is already a suitable entry
-> tests + evidence + handoff
```

E2 完成后可以说：

- 工作台已逐项定义会话操作边界：发消息、停止、重启、resume、导出、删除、收藏。
- UI 能解释哪些操作当前不可执行、为什么不可执行、未来需要什么前置条件。
- 会话中心和 adapter descriptor 不会把 planned adapters 或高风险 Codex 动作伪装成可执行按钮。
- 后续真实会话操作可以基于 E2 contract 单独拆小任务。

E2 完成后仍不能说：

- 会话中心已经可以发消息、stop、restart、resume、export、delete 或 favorite。
- Claude Code / OpenClaw / OpenCode / OpenCode-like 已真实接入。
- 外部模型、凭据、OAuth、keychain 或 provider 调用已接通。
- 真实 worker / Codex 执行完成。
- 阶段 G 真实 Tauri 全面验收完成。
- GEPA 或 Odysseus 研究内容已进入当前实现。

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

阶段 E 前置：

- `tasks/2026-06-05-stage-e-e1-agent-adapter-descriptor-execution-boundary-and-model-credential-readonly-foundation-v1.md`
- `evidence/2026-06-05-stage-e-e1-agent-adapter-descriptor-execution-boundary-and-model-credential-readonly-foundation-v1.md`
- `handoffs/2026-06-05-stage-e-e1-agent-adapter-descriptor-execution-boundary-and-model-credential-readonly-foundation-v1-result.md`

会话中心前置：

- `tasks/2026-06-03-session-center-foundation-hardening-v1.md`
- `evidence/2026-06-03-session-center-foundation-hardening-v1.md`
- `handoffs/2026-06-03-session-center-foundation-hardening-v1-result.md`
- `tasks/2026-06-03-agent-adapter-backend-capability-read-model-v1.md`
- `evidence/2026-06-03-agent-adapter-backend-capability-read-model-v1.md`
- `handoffs/2026-06-03-agent-adapter-backend-capability-read-model-v1-result.md`

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

- 新增或扩展只读类型，例如 `SessionOperationDescriptor` / `AgentSessionOperationBoundary`。
- 从现有 `SessionRecord`、`AgentAdapterDescriptor`、`WorkflowStateSnapshot` 派生操作边界，不新增持久 store。
- 在 `WorkbenchSnapshot` 或前端 read model 中暴露操作边界摘要，前提是不伪造真实能力。
- 在智能体页已有会话中心或 adapter 能力区域增加只读 operation boundary 面板 / 禁用态说明。
- 更新秘书只读模型，让它把高风险会话操作整理成提醒 / 风险，而不是 action proposal。
- 给每个操作定义：
  - 当前状态：`readonly_only` / `blocked` / `planned` / `requires_future_task`
  - 风险级别：`low` / `medium` / `high` / `destructive`
  - 是否需要用户确认。
  - 是否会写 `/Users/yoyi/.codex`。
  - 是否会写工作台 workflow state。
  - 是否会写业务项目目录。
  - 是否需要 credential / model access。
  - 是否需要运行日志 / 进程句柄。
  - 审计要求。
  - 未来实现任务建议。
- 增加 TypeScript / Rust 单测和离线 UI 测试。
- E2 完成后新增 evidence / handoff，并同步权威入口。

禁止：

- 不实现发送消息。
- 不实现停止会话。
- 不实现重启会话。
- 不实现通用 resume。
- 不实现导出文件写入。
- 不实现删除、移动、归档真实会话。
- 不实现收藏 / favorite 持久 store。
- 不执行 `codex exec`。
- 不执行 `codex exec resume`。
- 不启动真实 worker。
- 不启动 workflow machine。
- 不调用 Claude Code / OpenClaw / OpenCode / OpenCode-like 真实命令。
- 不调用外部模型 provider。
- 不读取 token、secret、keychain、OAuth session、provider credential、`.env` 或 auth 文件。
- 不读写 `/Users/yoyi/.codex`。
- 不读取真实完整 transcript 作为开发证据。
- 不新增 credential store。
- 不新增 adapter sidecar。
- 不新增 session operation / favorite / export sidecar。
- 不迁移数据库。
- 不改 `workflow-state.v0.json` 顶层结构。
- 不把 planned adapter 显示为可执行。
- 不新增一级入口、右侧顶级入口或项目页 tab。
- 不把 GEPA / Odysseus 研究项并入本任务实现。

如果执行者认为必须做任一禁止项才能完成 E2，必须停下回传并拆后续任务包。

## 5. 会话操作边界矩阵

E2 必须至少覆盖以下操作，并给出机器可测的状态：

| 操作 | E2 当前状态 | 当前允许显示 | 当前禁止 | 后续真实实现前置 |
| --- | --- | --- | --- | --- |
| `send_message` / 发消息 | `blocked` / `requires_future_task` | 可显示“当前不可执行，需要后续任务定义发送路径和确认” | 不显示输入框、不发送 prompt、不写 Codex 状态 | 明确 adapter runner、用户确认、审计、readback、失败处理、写入范围 |
| `resume` | `blocked` / `requires_future_task` | 可解释 workflow dispatch 已有受控 resume 语境，但会话中心未开放通用 resume | 不复用 workflow 派发按钮伪装成会话 resume | 绑定会话校验、prompt 预览、权限、超时、运行日志、readback |
| `stop` / 停止 | `blocked` | 可显示“缺少运行句柄 / 进程控制 / 日志体系” | 不 kill 进程、不声称已停止 agent | 运行进程 registry、取消协议、幂等审计、失败恢复 |
| `restart` / 重启 | `blocked` | 可显示“语义未定：新建会话 / 恢复旧会话 / 重跑任务需后续定义” | 不启动新 CLI、不重放 prompt | restart 语义、上下文来源、用户确认、成本提示、日志 |
| `export` / 导出 | `planned` | 可显示未来导出类型和脱敏要求 | 不写导出文件、不复制完整 transcript、不打包 secret | 导出格式、脱敏、范围选择、用户确认、审计、文件写入位置 |
| `delete` / 删除 | `blocked_destructive` | 可显示“破坏性操作，本阶段不可用” | 不删除、不移动、不归档、不改原生会话库 | 备份、回滚、双确认、作用域、原生系统兼容、审计 |
| `favorite` / 收藏 | `planned` | 可显示“需要工作台自有 metadata store，当前未实现” | 不写 favorite store、不伪造已收藏 | metadata store、冲突策略、导入导出、审计 |

E2 不能把 `reveal-rollout` 算作上述会话操作之一；它只是现有的本机辅助动作。

## 6. 建议数据模型

优先最小扩展，不新增 store：

```text
SessionOperationDescriptor {
  operation_id,
  label,
  category,
  current_status,
  risk_level,
  adapter_id,
  agent_type,
  applies_to_session_state,
  requires_user_confirmation,
  writes_codex_home,
  writes_workbench_state,
  writes_project_files,
  reads_full_transcript,
  requires_credential,
  requires_model_access,
  requires_runtime_handle,
  audit_requirement,
  unavailable_reason,
  future_task_hint,
  warnings
}
```

状态建议：

- `readonly_available`：只允许读取 / 解释，不触发写入。
- `blocked`：当前明确不可用。
- `planned`：未来可能做，但当前没有后端能力。
- `blocked_destructive`：破坏性操作，必须后续单独任务和双确认。
- `requires_future_task`：需要单独任务定义执行语义。

warning 建议：

- `session_operation_boundary_read_model_only`
- `no_session_operation_execution_in_e2`
- `no_codex_home_write_in_e2`
- `requires_future_authorization_task`
- `planned_adapter_operation_not_available`
- `destructive_operation_blocked`

## 7. UI 显示边界确认

本任务是否改前端：

- [ ] 不改前端、不改读模型、不改 UI 文案。
- [ ] 改前端类型 / Tauri wrapper，但不新增可见 UI。
- [x] 改读模型摘要或状态显示。
- [x] 改已有页面局部 UI。
- [x] 新增入口、面板、tab、按钮或确认动作。

说明：允许新增的是智能体页内部的“会话操作边界”局部面板 / 禁用态卡片；不允许新增一级入口、右侧顶级入口、项目页 tab 或可点击执行按钮。

已读取：

- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`
- `CURRENT.md`
- `tasks/README.md`

本任务允许显示：

- 会话操作的当前不可用 / planned / 只读状态。
- 每个操作为什么不可用。
- 每个操作未来需要的用户确认、审计、运行日志、凭据或模型前置。
- `codex-local` 与 planned adapters 在操作边界上的差异。
- “会话中心仍是只读历史浏览器”的边界说明。

本任务禁止显示：

- 可点击的发消息、停止、重启、resume、导出、删除、收藏按钮。
- 消息输入框、prompt 编辑框、发送按钮或聊天控制台。
- `Claude Code 可发送消息`、`OpenClaw 可 resume`、`OpenCode 已支持停止` 等未实现文案。
- `已导出`、`已删除`、`已收藏`、`已停止`、`已重启` 等无事实文案。
- raw transcript、raw adapter JSON、raw workflow state、完整日志、token、secret、keychain、OAuth、provider key、环境变量值或路径大表。
- 新的 `模型与 Agent` 一级入口。

显示位置：

- 一级入口：不新增；继续使用既有 `智能体`。
- 右侧入口：不新增；如需摘要，只能在既有秘书只读摘要或管理内部健康摘要中显示，不新增顶级图标。
- 项目页：不新增 tab，不占用工作流画布主区域。
- 画布：不改画布主区域。
- 记忆入口：不改。
- 知识库入口：不改。
- 智能体入口：可在既有会话中心 / adapter 能力面板附近显示“会话操作边界”。
- 管理入口：可选只读健康摘要，不显示 raw schema / secret。

中间版本范围：

- 本轮必须落地：会话操作边界契约和只读 / 禁用态可见化。
- 本轮只做读模型 / 摘要：操作状态、风险、不可用原因、未来前置条件。
- 本轮后置：真实发消息、stop、restart、resume、export、delete、favorite、运行日志、自动重试、真实 Tauri 全面验收。

后端和数据依赖：

- 需要后端正式读模型：优先复用 `WorkbenchSnapshot` / `agent_adapters[]`；如新增 operation descriptors，应是派生读模型。
- 需要审计 / 日志 / 权限 / 状态机：本轮只定义要求，不新增运行日志或真实操作审计体系。
- 不能用假数据伪装：所有操作默认不可执行或 planned，不能用 mock 成功状态。

UI 文案边界：

- 禁止说：`已发送`、`已停止`、`已重启`、`已 resume`、`已导出`、`已删除`、`已收藏`、`Claude Code 已支持发送`、`OpenClaw 已支持会话控制`、`OpenCode 已接入会话操作`。
- 允许说：`当前不可执行`、`需要后续授权任务`、`只读边界`、`缺少运行句柄`、`缺少导出脱敏策略`、`破坏性操作已阻断`、`计划中`。

验收：

- 类型检查：如改前端必须 `npm run typecheck`。
- 离线交互测试：如改前端必须 `npm run test:offline-interaction`。
- 构建：如改前端必须 `npm run build`。
- 真实窗口 / 截图验收：尽量做浏览器 / Tauri smoke；未完成必须写入 evidence / handoff，且不能接受为阶段 G 验收。
- 未验收项必须写入 evidence / handoff。

## 8. 建议执行段

### 执行段 A：盘点现有操作语义

目标：

- 明确当前已有按钮 / action / command 哪些只是只读辅助，哪些属于项目工作流高风险动作。

建议实现：

1. 搜索 `PendingAction`、`PathActionKind`、`execute-node-dispatch`、`run-workflow-machine`、`reveal-rollout`、`codex exec resume`。
2. 在 evidence 中列出当前已有能力和不属于会话中心通用操作的原因。
3. 不改变现有 workflow dispatch 行为。

验收：

- E2 evidence 能说明 `execute-node-dispatch` 不是会话中心 `send_message`。
- `reveal-rollout` 仍只是定位文件，不被升级为会话操作系统。

### 执行段 B：会话操作边界读模型

目标：

- 用机器可测的结构表达操作边界。

建议实现：

1. 新增轻量 operation descriptor 类型，或在现有前端 helper 中派生。
2. 对 `codex-local` 输出七类操作状态。
3. 对 planned adapters 输出同样操作，但全部 blocked / planned / not available。
4. 不新增持久 sidecar。
5. 不调用任何真实命令或 provider。

验收：

- 七类操作都出现。
- 没有任何操作状态是 `available_to_execute` 或同义可执行。
- destructive 操作必须是 blocked。
- planned adapter 的所有操作必须不可执行。

### 执行段 C：智能体页只读 / 禁用态展示

目标：

- 用户能看到边界，而不是看到坏掉或危险的按钮。

建议实现：

1. 在智能体页已有会话中心 / adapter 能力区域附近新增局部“会话操作边界”面板。
2. 使用卡片或矩阵显示操作、当前状态、原因、未来前置。
3. 不显示可点击执行按钮；如果必须表现按钮形态，只能是不可点击禁用态，并带明确 `aria-disabled` / 文案。
4. 保持现有“重新读取”“定位 rollout”辅助动作不变。
5. 不新增消息输入框。

验收：

- UI 文案包含“当前不可执行”或等价边界。
- UI 不包含可点击 `发送` / `停止` / `重启` / `resume` / `导出` / `删除` / `收藏` 操作按钮。
- planned adapters 不出现可执行操作。

### 执行段 D：秘书 / 管理摘要

目标：

- 如已有读模型入口适合展示，给秘书或管理健康摘要提供轻量风险解释；不生成 action proposal。

建议实现：

1. 秘书只读模型可以提醒“会话操作仍未开放”。
2. 管理健康摘要可选显示 blocked operation count。
3. 不新增右侧顶级入口。
4. 不把提醒变成待办执行项。

验收：

- 秘书不提出“发送消息 / 停止 / 删除”等可执行建议。
- 管理摘要不显示 raw schema 或路径大表。

### 执行段 E：测试和文档回收

目标：

- 用测试证明 operation boundary 不等于真实操作能力。

建议测试覆盖：

- 七类操作都有 descriptor。
- 所有操作在 E2 中不可执行或 planned。
- `delete` 是 destructive blocked。
- `send_message` / `resume` 不复用 workflow dispatch 伪装为可执行。
- planned adapters 全部无会话操作能力。
- UI 显示边界，但不渲染可点击操作按钮。
- 秘书不生成会话操作 action proposal。

## 9. 验收命令

必须运行或明确说明无法运行原因：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
cargo test --lib session_operation
cargo test --lib adapter_descriptor
cargo test --lib agent_adapter
cargo test --lib
rustfmt --check src/types.rs src/lib.rs src/commands.rs
```

如果执行者没有新增 Rust 代码，应在 evidence 中说明 Rust 定向测试是否不适用，并至少跑与改动相关的前端测试。

如果新增的 Rust 单测无法用 `session_operation` filter 覆盖，必须在 evidence 中写清实际 filter。

必须做禁止文案扫描：

```text
rg -n "已发送|已停止|已重启|已 resume|已导出|已删除|已收藏|Claude Code 已支持发送|OpenClaw 已支持会话控制|OpenCode 已接入会话操作|真实 Codex 已执行|自动派发已开始" prototypes/productized-desktop-shell/src
```

预期：无误导命中。若历史文案合理存在，必须逐条解释为什么不是 E2 新增误导。

## 10. evidence / handoff 要求

E2 完成后必须新增：

- `evidence/2026-06-05-stage-e-e2-session-operation-boundary-contract-and-readonly-ui-v1.md`
- `handoffs/2026-06-05-stage-e-e2-session-operation-boundary-contract-and-readonly-ui-v1-result.md`

evidence 必须记录：

- E2 接受为什么。
- 七类会话操作的最终状态矩阵。
- 为什么 E2 没有实现真实操作。
- 为什么 `execute-node-dispatch` / workflow resume 不能等同于会话中心发消息。
- planned adapters 如何保持不可执行。
- UI 是否出现可点击执行按钮。
- 禁止文案扫描结果。
- 验证命令和结果。
- 是否完成真实窗口 / 截图验收；如未完成，写清不接受为阶段 G 验收。
- 边界：未执行 `codex exec` / `codex exec resume`，未读写 `/Users/yoyi/.codex`，未调用外部 agent / provider，未新增 store。

handoff 必须写清：

- E2 接受为什么。
- E2 不接受为什么。
- 后续建议：E3 是真实发送 / resume 方案设计，还是模型 / 凭据只读深化。
- 当前权威入口文件。

## 11. Stop 条件

遇到以下情况必须停下回传：

- 需要读写 `/Users/yoyi/.codex`。
- 需要执行 `codex exec` 或 `codex exec resume`。
- 需要调用 Claude Code / OpenClaw / OpenCode / OpenCode-like 真实命令。
- 需要调用外部模型 provider。
- 需要读取 token、secret、keychain、OAuth session、provider 凭据或 `.env`。
- 需要读取真实完整 transcript 作为开发证据。
- 需要新增 credential store、adapter sidecar、session operation sidecar、favorite store 或 export store。
- 需要迁移数据库。
- 需要改 `workflow-state.v0.json` 顶层结构。
- 需要启动真实 worker、workflow machine 或自动派发。
- 需要新增一级入口、右侧顶级入口或项目页 tab。
- 需要显示可点击的会话操作按钮。
- 需要把 GEPA / Odysseus 研究点合入当前实现。

## 12. 回收口径

完成后接受为：

- 阶段 E / E2 会话操作边界契约完成。
- `send_message` / `stop` / `restart` / `resume` / `export` / `delete` / `favorite` 的权限、审计、数据写入、UI 和后续真实执行条件已逐项定义。
- 智能体页可以安全解释这些操作当前不可执行或 planned。
- planned adapters 继续不可执行。

完成后不接受为：

- 会话中心真实发消息完成。
- 通用 `codex exec resume` / stop / restart 完成。
- 会话导出、删除、收藏完成。
- Claude Code / OpenClaw / OpenCode 真实接入完成。
- 外部模型或凭据管理完成。
- 运行日志、自动重试、取消恢复、运维诊断完成。
- 真实 worker / Codex 执行完成。
- 阶段 G 真实 Tauri 全面验收完成。

建议下一步：

- E3：模型 / 凭据只读状态深化，定义设置 / 管理入口、安全摘要、不可见 secret 边界和 provider 不可用状态。
- 或 E3：真实发送 / resume 的设计预研任务，只写方案和风险，不直接执行。
