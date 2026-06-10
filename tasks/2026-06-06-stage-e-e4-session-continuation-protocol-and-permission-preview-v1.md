# Task Package：Stage E / E4 Session Continuation Protocol And Permission Preview v1

状态：已完成。  
用途：在 E1 adapter descriptor、E2 session operation boundary 和 E3 provider availability 只读边界之后，定义会话继续发送 / resume 的安全协议、权限预览、prompt preview、readback expectation、失败边界和 guard；本任务只做预览和阻断，不执行真实发送。  
执行方式：小切片实现；优先复用 `WorkbenchSnapshot.agent_adapters[]`、`session_operations[]`、`provider_availability[]`、项目 / workflow / session binding 和现有 control core 经验；不执行 `codex exec resume`，不写 `/Users/yoyi/.codex`，不发送真实 prompt。

## 0. 先说薄弱点

- E2 已经定义 `send_message` / `resume` 等会话操作边界，但仍是只读 / 禁用态；E4 不能把 E2 的边界描述直接升级成真实操作。
- 过去工作流体系里已有受控 `codex exec resume` 经验，但那属于项目工作流 dispatch 语境，不等于智能体页可以自由发消息。
- “发消息继续会话”是中间版本最终必须补齐的能力，但如果没有 target session、project binding、cwd、allowed write roots、sandbox、prompt preview、readback expectation、failure handling 和 audit impact，直接实现会复现早期真实 resume 的风险。
- UI 如果出现“预览 / 申请确认”，用户容易误解为已经发送；E4 必须把“预览不是执行”写进读模型、文案和测试。
- planned adapters 仍不可执行；E4 不能让 Claude Code / OpenClaw / OpenCode / OpenCode-like 获得继续会话能力。
- E3 provider availability 只能作为 guard 输入之一，不能被解释成项目授权、会话授权或 provider 可执行。
- GEPA / Paseo / Odysseus 研究资料仍只作为蓝图参考；E4 不吸收优化器、daemon、workspace 复刻或外部项目融合点。

## 1. 已知事实 / 未知 / 假设

已知事实：

- 阶段 C / C1-C6 已完成，接受为受控自动化工作流闭环。
- 阶段 D / M1-M13 已完成，M13 结论为 `accepted_with_deferred_items`。
- 阶段 E / E1 已完成，`WorkbenchSnapshot.agent_adapters[]` 可区分 `codex-local` 和 planned adapters。
- 阶段 E / E2 已完成，`WorkbenchSnapshot.session_operations[]` 覆盖 `send_message` / `stop` / `restart` / `resume` / `export` / `delete` / `favorite`，但不执行真实操作。
- 阶段 E / E3 已完成，`WorkbenchSnapshot.provider_availability[]` 只读表达 provider / model / credential / external call / cost risk 边界，但不验证 provider、不调用模型、不读取凭据。
- `codex-local` 仍是唯一可用 adapter descriptor；planned adapters 当前不可执行、未配置凭据、模型未验证。
- 当前仍禁止默认执行 `codex exec` / `codex exec resume`，也禁止默认读写 `/Users/yoyi/.codex`。
- UI 任务包必须落实 `docs/plans/task-package-ui-display-boundary-rule-v1.md` 的“UI 显示边界确认”章节。

未知：

- 真实 `send_message` / `resume` 最终是复用现有 workflow dispatch、单独 session controller，还是 adapter runner。
- E4 的 preview 是否落持久 sidecar，还是只作为派生 read model / command response；如要持久化必须说明原因、schema、备份和回滚。
- target session 的可信来源最终是 project binding、session catalog、workflow node binding 还是用户选择。
- E5 真实执行时如何记录 attempt、readback、runtime log 和审计；E4 只定义预览字段和 guard 输入，不写真实执行记录。

本任务采用的假设：

- E4 只建立 request / preview / guard / permission preview，不执行真实发送。
- E4 可以新增或扩展 `SessionContinuationRequest` / `SessionContinuationPreview` / `SessionContinuationGuard` / `SessionContinuationPreviewResult` 或等价类型。
- E4 可以在 UI 中显示“预览 / 申请确认”局部入口或确认弹层摘要，但不能出现真实发送按钮、自由聊天输入框或执行态。
- E4 不新增长期 store；如实现者认为必须新增 `session-continuations.v1.json` 或 execution store，必须停下拆后续任务。
- 如果实现者发现必须执行 `codex exec resume`、写 `/Users/yoyi/.codex`、发送 prompt、读完整真实 transcript、调用外部 provider 或支持 planned adapter 才能完成，必须停下回传。

## 2. 任务目标

完成阶段 E 第四刀协议：

```text
E1 adapter descriptors
-> E2 session operation boundary
-> E3 provider availability boundary
-> SessionContinuationRequest
-> SessionContinuationPreview
-> SessionContinuationGuard
-> permission preview / prompt summary / readback expectation / failure boundary
-> UI shows preview is not execution
-> tests + evidence + handoff
```

E4 完成后可以说：

- 工作台已经定义会话继续 / resume 的安全协议和权限预览。
- 用户可以看到继续会话前的 target session、project binding、cwd、allowed write roots、sandbox、prompt summary、readback expectation、failure handling 和 audit impact。
- control core 或等价 guard 能拒绝未绑定项目、越界 cwd、缺少用户确认、planned adapter、敏感路径、无 readback 策略、provider unavailable 的请求。
- E5 后续可以在 E4 协议上实现 `codex-local` 受控 send / resume 最小闭环。

E4 完成后仍不能说：

- 会话中心真实发消息完成。
- 通用 `codex exec resume` 完成。
- Codex 已收到 prompt。
- worker / agent 已执行。
- readback 已发生。
- runtime log、attempt、自动重试、取消恢复完成。
- Claude Code / OpenClaw / OpenCode / OpenCode-like 已具备继续会话能力。
- 阶段 G 真实 Tauri 全面验收完成。

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
- `tasks/2026-06-06-stage-e-e3-model-credential-provider-availability-readonly-boundary-v1.md`
- `evidence/2026-06-06-stage-e-e3-model-credential-provider-availability-readonly-boundary-v1.md`
- `handoffs/2026-06-06-stage-e-e3-model-credential-provider-availability-readonly-boundary-v1-result.md`

会话 / 工作流前置：

- `tasks/2026-06-03-session-center-foundation-hardening-v1.md`
- `evidence/2026-06-03-session-center-foundation-hardening-v1.md`
- `handoffs/2026-06-03-session-center-foundation-hardening-v1-result.md`
- `tasks/2026-06-03-agent-adapter-backend-capability-read-model-v1.md`
- `evidence/2026-06-03-agent-adapter-backend-capability-read-model-v1.md`
- `handoffs/2026-06-03-agent-adapter-backend-capability-read-model-v1-result.md`
- `tasks/2026-06-04-workflow-c4-project-director-task-decomposition-and-authorized-prepared-auto-dispatch-v1.md`
- `evidence/2026-06-04-workflow-c4-project-director-task-decomposition-and-authorized-prepared-auto-dispatch-v1.md`
- `handoffs/2026-06-04-workflow-c4-project-director-task-decomposition-and-authorized-prepared-auto-dispatch-v1-result.md`

主要代码入口：

- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/control_core.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `prototypes/productized-desktop-shell/src/lib/adapterCapabilities.ts`
- `prototypes/productized-desktop-shell/src/lib/providerAvailability.ts`
- `prototypes/productized-desktop-shell/src/views/AgentView.tsx`
- `prototypes/productized-desktop-shell/src/components/PermissionDialog.tsx`
- `prototypes/productized-desktop-shell/src/lib/secretaryReadModel.ts`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

搜索固定文本必须用 `rg -F '...'` 或单引号，避免 shell 反引号命令替换。

## 4. 范围

允许：

- 新增或扩展只读 / 预览类型，例如：
  - `SessionContinuationRequest`
  - `SessionContinuationPreview`
  - `SessionContinuationGuard`
  - `SessionContinuationGuardResult`
  - `SessionContinuationPreviewResult`
  - `PromptPreviewSummary`
  - `ReadbackExpectation`
  - `ContinuationFailureBoundary`
  - `ContinuationAuditImpact`
- 定义 `send_message` 与 `workflow dispatch resume` 的关系和差异。
- 从现有项目、workflow、node、session binding、adapter descriptor、operation descriptor、provider availability 和 allowed roots 派生 preview。
- 为 `codex-local` 输出 preview-capable / needs confirmation / blocked reason；planned adapters 必须 blocked。
- 定义 guard 状态：`allowed_preview` / `needs_user_confirmation` / `blocked` / `requires_future_task` 或等价状态。
- UI 可以在智能体页或项目相关会话区域显示“继续会话预览 / 申请确认”局部入口；入口只能生成预览或确认请求，不能执行发送。
- PermissionDialog 可以显示 E4 预览摘要，但确认结果不能触发真实 `codex exec resume`。
- 秘书可以解释 continuation guard 风险，并引导用户查看预览；不能发送、批准、重试或生成真实执行 action proposal。
- 更新 TypeScript 类型、Tauri wrapper、Rust 类型、guard 单测、离线 UI 测试和文档。
- E4 完成后新增 evidence / handoff，并同步权威入口。

禁止：

- 不执行 `codex exec`。
- 不执行 `codex exec resume`。
- 不发送真实 prompt。
- 不写 `/Users/yoyi/.codex`。
- 不读取真实完整 transcript 作为开发证据。
- 不调用 Claude Code / OpenClaw / OpenCode / OpenCode-like 真实命令。
- 不调用外部模型 provider。
- 不读取 token、secret、`.env`、keychain、OAuth session、provider credential、auth 文件内容或环境变量值。
- 不新增真实 credential store。
- 不新增 adapter / provider / continuation execution sidecar。
- 不迁移数据库。
- 不改 `workflow-state.v0.json` 顶层结构。
- 不新增 runtime log 最终形态；运行日志进入 G1。
- 不写真实 attempt、dispatch、readback 或 worker report。
- 不把 preview confirmed 当成 execution started。
- 不支持 planned adapters 的继续会话能力。
- 不支持 stop / restart / delete / export / favorite。
- 不提供自由聊天输入框绕过项目、workflow、node、session binding、任务包和权限。
- 不新增一级入口、右侧顶级入口或项目页 tab。
- 不把 GEPA / Paseo / Odysseus 研究项并入本任务实现。

如果执行者认为必须做任一禁止项才能完成 E4，必须停下回传并拆 E5 或后续任务包。

## 5. Send Message 与 Workflow Dispatch Resume 的关系

E4 必须明确写进读模型或 evidence：

| 项 | `send_message` / 会话继续 | `workflow dispatch resume` / 项目派发 |
| --- | --- | --- |
| 目标 | 继续一个已绑定会话的下一轮用户意图 | 在项目工作流授权范围内派发任务包 |
| 必须绑定 | project / workflow / node / session | project / workflow / node / task package / session |
| E4 状态 | 只做预览和 guard | 只参考既有经验，不启动派发 |
| prompt | 只显示 summary / preview，不发送全文 | 由任务包生成，E4 不触发 |
| 权限 | 用户确认前只能 needs confirmation | C1-C4 guard 负责 prepared dispatch |
| readback | 只定义 expectation | C5 / 后续 runtime 读模型负责结果可见化 |
| 写入 | E4 默认不写执行状态 | 既有 dispatch 才写状态，E4 不触发 |
| 风险 | 自由聊天绕过任务包、cwd 越界、readback 缺失 | 授权范围、任务包、记忆包、readback、audit |

E4 不能把 workflow dispatch 的历史能力包装成会话中心已可发送消息。

## 6. Guard 矩阵

E4 guard 至少覆盖：

| 场景 | 预期结果 | 原因 |
| --- | --- | --- |
| `codex-local` + 已绑定 project/workflow/node/session + cwd 在 project root / allowed roots 内 + 有 readback strategy | `needs_user_confirmation` 或 `allowed_preview` | 可以生成预览，但不能执行 |
| 缺少 project binding | `blocked` | 不能自由会话绕过项目上下文 |
| 缺少 workflow / node binding | `blocked` | 不能绕过工作流和任务包 |
| 缺少 session binding | `blocked` | 无 target session |
| cwd 越出 project root / allowed write roots | `blocked` | 写入范围风险 |
| target path 命中敏感路径，例如 `/Users/yoyi/.codex`、`.ssh`、`.env`、keychain、auth/token 路径 | `blocked` | secret / 原生状态风险 |
| 缺少用户确认 | `needs_user_confirmation` | 预览可以生成，执行不能发生 |
| planned adapter | `blocked` | 未接入、未验证、不可执行 |
| provider availability 为 `external_call_blocked` / `credential_missing` | `blocked` 或 `requires_future_task` | 外发和凭据边界未满足 |
| 缺少 readback strategy | `blocked` | 不能把 readback 失败伪装成 0 条结果 |
| prompt summary 为空或无法解释 | `blocked` | 用户无法理解将要继续什么 |
| destructive / stop / restart / delete / export / favorite | `blocked` | 不属于 E4 |

## 7. 建议数据模型

优先最小扩展，不新增 store：

```text
SessionContinuationRequest {
  adapter_id,
  operation_id,
  project_id,
  workflow_id,
  node_id,
  session_id,
  target_cwd,
  allowed_write_roots,
  sandbox,
  prompt_source_kind,
  prompt_summary,
  readback_strategy,
  requested_by,
  user_confirmation_state
}
```

```text
SessionContinuationPreview {
  preview_id,
  adapter_id,
  operation_id,
  target_session,
  project_binding,
  workflow_binding,
  node_binding,
  cwd_summary,
  allowed_write_roots_summary,
  sandbox_summary,
  prompt_summary,
  readback_expectation,
  failure_handling,
  audit_impact,
  provider_availability_summary,
  guard_result,
  user_visible_warnings
}
```

```text
SessionContinuationGuardResult {
  status,
  severity,
  blocks_execution,
  allows_preview,
  requires_user_confirmation,
  reasons,
  required_fixes,
  warnings
}
```

状态建议：

- `status`: `allowed_preview` / `needs_user_confirmation` / `blocked` / `requires_future_task`
- `operation_id`: `send_message` / `resume`
- `prompt_source_kind`: `user_draft` / `task_package_summary` / `workflow_followup` / `not_allowed`
- `readback_strategy`: `required` / `unavailable_blocked` / `deferred_to_e5` / `not_defined`
- `audit_impact`: `preview_only_no_execution` / `would_require_attempt_audit_in_e5`

warning 建议：

- `session_continuation_preview_only`
- `no_prompt_sent_in_e4`
- `no_codex_home_write_in_e4`
- `user_confirmation_required_before_execution`
- `planned_adapter_blocked`
- `cwd_out_of_scope_blocked`
- `readback_strategy_required`
- `provider_availability_not_execution_authorization`

## 8. UI 显示边界确认

本任务是否改前端：

- [ ] 不改前端、不改读模型、不改 UI 文案。
- [x] 改前端类型 / Tauri wrapper，但不新增可见 UI。
- [x] 改读模型摘要或状态显示。
- [x] 改已有页面局部 UI。
- [x] 新增入口、面板、tab、按钮或确认动作。

说明：允许新增的是智能体页或项目相关会话区域内部的“继续会话预览 / 申请确认”局部入口、预览面板或确认弹层摘要；不允许新增一级入口、右侧顶级入口、项目页 tab、自由聊天输入框或真实发送按钮。

已读取：

- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`
- `CURRENT.md`
- `tasks/README.md`

本任务允许显示：

- target session、项目 / workflow / node binding、cwd、allowed write roots、sandbox。
- prompt summary / prompt preview 摘要，不显示完整 raw prompt 大段。
- readback expectation、失败处理、audit impact。
- guard result：可预览、需要用户确认、阻断、需要后续任务。
- “预览不是执行”“不会发送 prompt”“不会写 `/Users/yoyi/.codex`”边界说明。
- planned adapters 被阻断的原因。

本任务禁止显示：

- 可点击真实发送按钮。
- 自由聊天输入框或 prompt 编辑器。
- `已发送`、`已 resume`、`Codex 已收到任务`、`自动派发已开始`、`worker 执行中`、`readback 已完成` 等无事实文案。
- planned adapter 的继续会话按钮或可执行态。
- raw transcript、raw adapter JSON、raw workflow state、raw audit、完整日志、token、secret、keychain、OAuth、provider key、环境变量值或路径大表。
- 新的 `模型与 Agent` 一级入口。

显示位置：

- 一级入口：不新增；继续使用既有 `智能体`。
- 右侧入口：不新增；秘书只读摘要可解释风险，不新增顶级图标。
- 项目页：不新增 tab，不占用工作流画布主区域；如涉及项目上下文，只能在项目相关会话 / 节点详情附近显示。
- 画布：不改画布主区域；如未来需要节点入口，必须进入 F 阶段。
- 记忆入口：不改。
- 知识库入口：不改。
- 智能体入口：可显示继续会话预览 / guard 摘要。
- 管理入口：不新增；可选只读健康摘要，不显示 raw schema / secret。

中间版本范围：

- 本轮必须落地：会话继续请求、预览、guard、权限预览和 UI “预览不是执行”边界。
- 本轮只做读模型 / 摘要：target session、binding、cwd、sandbox、prompt summary、readback expectation、failure boundary、audit impact、guard reasons。
- 本轮后置：真实 send / resume、attempt 写入、readback 读取、runtime log、自动重试、取消恢复、真实 Tauri 全面验收。

后端和数据依赖：

- 需要后端正式读模型：优先复用 `WorkbenchSnapshot`、adapter descriptors、session operations、provider availability 和 workflow bindings。
- 需要审计 / 日志 / 权限 / 状态机：本轮只做 preview / guard；真实 attempt audit、runtime log 和 readback 进入 E5 / E6 / G1。
- 不能用假数据伪装：不能用前端 mock 显示“已发送 / 已收到 / 已运行 / 已读回”。

UI 文案边界：

- 禁止说：`已发送`、`已 resume`、`Codex 已收到任务`、`自动派发已开始`、`worker 执行中`、`readback 已完成`、`Claude Code 可继续会话`、`OpenClaw 可 resume`、`OpenCode 已支持发送`。
- 允许说：`预览未执行`、`需要用户确认`、`当前阻断`、`缺少 readback 策略`、`cwd 越界`、`planned adapter 不可执行`、`不会写 Codex 原生状态`、`不会发送 prompt`。

验收：

- 类型检查：如改前端必须 `npm run typecheck`。
- 离线交互测试：如改前端必须 `npm run test:offline-interaction`。
- 构建：如改前端必须 `npm run build`。
- 真实窗口 / 截图验收：尽量做浏览器 / Tauri smoke；未完成必须写入 evidence / handoff，且不能接受为阶段 G 验收。
- 未验收项必须写入 evidence / handoff。

## 9. 建议执行段

### 执行段 A：盘点现有 dispatch / session 语义

目标：

- 明确智能体页会话继续和项目工作流 dispatch resume 的差异，避免复用历史能力制造误导。

建议实现：

1. 搜索 `SessionOperationDescriptor`、`execute-node-dispatch`、`codex exec resume`、`RealCodexResumeRunner`、`workflow_machine`、`active_bindings`。
2. 在 evidence 中列出哪些路径是历史 / 项目工作流能力，哪些不能被 E4 触发。
3. 不改变现有 workflow dispatch 行为。

验收：

- evidence 能说明 E4 没有触发既有 dispatch。
- E4 不新增 `Command::new("codex")` 调用。

### 执行段 B：后端 preview / guard 模型

目标：

- 用机器可测结构表达预览和阻断，不执行真实命令。

建议实现：

1. 新增最小 Rust 类型和 helper。
2. 输入 project / workflow / node / session / adapter / cwd / sandbox / prompt summary / readback strategy。
3. 输出 preview 和 guard result。
4. planned adapter 一律 blocked。
5. cwd / allowed roots / sensitive path / missing binding / missing readback strategy 必须可测。

验收：

- guard 单测覆盖 allowed preview / needs confirmation / blocked。
- blocked reason 可被 UI 展示。
- 不写 workflow state 顶层结构。

### 执行段 C：前端 wrapper 和 UI 预览

目标：

- 用户能看到继续会话前会发生什么，同时明确不会执行。

建议实现：

1. 更新 TS 类型和 Tauri wrapper。
2. 在智能体页或项目相关会话区域新增局部 preview trigger。
3. 预览面板展示 target session、binding、cwd、sandbox、prompt summary、readback expectation、audit impact。
4. 如果接 PermissionDialog，只显示“申请确认 / 预览确认”语义，确认后不能执行。
5. 不新增自由聊天输入框。

验收：

- UI 测试覆盖“预览不是执行”。
- UI 不渲染真实发送按钮。
- UI 不显示已发送 / 已执行 / 已读回文案。

### 执行段 D：秘书风险解释

目标：

- 秘书帮助用户理解为什么被阻断或为什么需要确认，但不代替确认或执行。

建议实现：

1. 秘书只读模型读取 preview / guard 摘要。
2. 秘书可以输出查看建议或风险说明。
3. 秘书不能生成 send / resume / approve / retry action proposal。

验收：

- 离线测试断言秘书没有 send / resume / approve / retry action proposal。

### 执行段 E：测试、扫描和文档回收

目标：

- 证明 E4 是协议和预览，不是真实执行。

建议测试覆盖：

- `codex-local` + 完整绑定可生成 preview，但仍需要用户确认或仍是 preview-only。
- 缺 project / workflow / node / session binding 会被 blocked。
- cwd 越界会被 blocked。
- `/Users/yoyi/.codex` / `.env` / keychain / auth / token / secret 路径会被 blocked。
- planned adapter 会被 blocked。
- 缺 readback strategy 会被 blocked。
- provider availability 不满足会被 blocked 或 requires future task。
- UI 显示 preview-only 文案，无真实发送按钮。
- PermissionDialog 不触发 execution。
- 秘书不生成执行 action proposal。

## 10. 验收命令

必须运行或明确说明无法运行原因：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
cargo test --lib session_continuation
cargo test --lib session_operation
cargo test --lib provider_availability
cargo test --lib workflow_authorization
cargo test --lib
rustfmt --check src/types.rs src/lib.rs src/control_core.rs src/commands.rs
```

如果执行者修改了其他 Rust 文件，必须纳入 `rustfmt --check`。

如果新增的 Rust 单测无法用 `session_continuation` filter 覆盖，必须在 evidence 中写清实际 filter。

必须做禁止文案扫描：

```text
rg -n "已发送|已 resume|Codex 已收到任务|自动派发已开始|worker 执行中|readback 已完成|Claude Code 可继续会话|OpenClaw 可 resume|OpenCode 已支持发送|真实 Codex 已执行" prototypes/productized-desktop-shell/src
```

预期：无误导命中。若历史文案合理存在，必须逐条解释为什么不是 E4 新增误导。

必须做真实执行 / 敏感路径扫描：

```text
rg -n "Command::new\\(\"codex\"\\)|codex exec resume|\\.codex|read_to_string\\(.*auth|read_to_string\\(.*token|read_to_string\\(.*secret|read_to_string\\(.*\\.env|keychain|oauth|provider credential" prototypes/productized-desktop-shell/src-tauri/src prototypes/productized-desktop-shell/src
```

预期：E4 不新增真实 Codex runner、secret 读取、provider credential 读取或 `.codex` 写入。历史命中必须分类解释。

## 11. evidence / handoff 要求

E4 完成后必须新增：

- `evidence/2026-06-06-stage-e-e4-session-continuation-protocol-and-permission-preview-v1.md`
- `handoffs/2026-06-06-stage-e-e4-session-continuation-protocol-and-permission-preview-v1-result.md`

evidence 必须记录：

- E4 接受为什么。
- `send_message` 与 `workflow dispatch resume` 的差异。
- request / preview / guard 最终字段或等价结构。
- guard 矩阵最终结果。
- 为什么 E4 没有执行 `codex exec` / `codex exec resume`。
- 为什么 E4 没有发送 prompt、没有写 `/Users/yoyi/.codex`、没有写真实 attempt / dispatch / readback。
- provider availability 如何参与 guard，但不等于授权。
- planned adapters 如何保持 blocked。
- UI 显示位置和“预览不是执行”证据。
- PermissionDialog 如有接入，确认后是否仍未执行。
- 秘书是否生成 action proposal；如没有，写清测试或代码证据。
- 禁止文案扫描结果。
- 真实执行 / 敏感路径扫描结果和合理命中解释。
- 验证命令和结果。
- 是否完成真实窗口 / 截图验收；如未完成，写清不接受为阶段 G 验收。
- 边界：未执行 `codex exec` / `codex exec resume`，未读写 `/Users/yoyi/.codex`，未调用外部 agent / provider，未新增 execution store，未迁移数据库。

handoff 必须写清：

- E4 接受为什么。
- E4 不接受为什么。
- 后续建议：E5 `codex-local` 受控 send / resume 最小闭环；真实执行必须另行取得用户批准。
- 当前权威入口文件。

## 12. Stop 条件

遇到以下情况必须停下回传：

- 需要读写 `/Users/yoyi/.codex`。
- 需要执行 `codex exec` 或 `codex exec resume`。
- 需要发送真实 prompt。
- 需要调用 Claude Code / OpenClaw / OpenCode / OpenCode-like 真实命令。
- 需要调用外部模型 provider。
- 需要读取 token、secret、`.env`、keychain、OAuth session、provider credential、auth 文件内容或环境变量值。
- 需要读取真实完整 transcript 作为开发证据。
- 需要新增 continuation execution store、credential store、adapter sidecar、provider sidecar 或数据库迁移。
- 需要改 `workflow-state.v0.json` 顶层结构。
- 需要启动真实 worker、workflow machine 或自动派发。
- 需要写真实 attempt、dispatch、readback、worker report 或 runtime log 最终形态。
- 需要新增一级入口、右侧顶级入口或项目页 tab。
- 需要显示可点击的真实发送、resume、stop、restart、delete、export、favorite 按钮。
- 需要提供自由聊天输入框绕过项目 / workflow / node / session binding。
- 需要让 planned adapter 支持继续会话。
- 需要把 GEPA / Paseo / Odysseus 研究点合入当前实现。

## 13. 回收口径

完成后接受为：

- 阶段 E / E4 会话继续协议和权限预览完成。
- `send_message` 与 `workflow dispatch resume` 的关系和差异已明确。
- `SessionContinuationRequest` / `SessionContinuationPreview` / `SessionContinuationGuard` 或等价模型已落地。
- 预览能显示 target session、project binding、cwd、allowed write roots、sandbox、prompt summary、readback expectation、failure handling 和 audit impact。
- guard 能阻断未绑定项目、越界 cwd、缺少用户确认、planned adapter、敏感路径、无 readback 策略等请求。
- UI 能表达“预览不是执行”。

完成后不接受为：

- 会话中心真实发消息完成。
- 通用 `codex exec resume` 完成。
- prompt 已发送或 Codex 已收到任务。
- worker / agent 已执行。
- readback 已发生。
- runtime log、attempt、自动重试、取消恢复完成。
- Claude Code / OpenClaw / OpenCode / OpenCode-like 继续会话能力完成。
- 阶段 G 真实 Tauri 全面验收完成。

建议下一步：

- E5：`codex-local` controlled send / resume minimal loop。E5 才能在 E4 协议上实现最小受控发送 / resume；如要真实执行 `codex exec resume` 或写 `/Users/yoyi/.codex`，必须在任务包内单独列权限、读写范围、回滚和用户批准。
