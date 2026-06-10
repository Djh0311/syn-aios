# Evidence：Stage E / E7 Session Adapter Model Boundary Acceptance v1

日期：2026-06-06

## 1. 结论

E7 已完成，阶段 E 总结论冻结为：

```text
accepted_with_deferred_items
```

接受为：

- E1-E6 的 evidence / handoff 全部存在且可追溯。
- 阶段 E 的 adapter descriptor、session operation、provider availability、session continuation preview、E5 Level A controlled continuation、runtime attention / readback failure boundary 已形成可进入阶段 F 的只读 / 边界底座。
- `codex-local` 仍是唯一可用 adapter descriptor；Claude Code / OpenClaw / OpenCode / OpenCode-like 仍是 planned / not implemented / credential not configured / model not verified。
- E5 只接受为 Level A：代码路径、guard、stub / dry-run、工作台自有 continuation sidecar、audit ref 和 readback unavailable 边界。
- E6 只接受为 runtime attention 和 readback failed / unavailable 的最小读模型、摘要 UI 和秘书只读解释。
- F1 可以开始，但只能继承阶段 E 的只读边界和 deferred 约束，不能继承真实 send / resume、真实 readback、planned adapter 真实接入、provider credential 验证、runtime log、diagnostics 或真实 Tauri 验收。

不接受为：

- E5 Level B 真实 `codex exec resume` 完成。
- 真实 prompt 已发送、Codex 已收到任务或真实 readback 已完成。
- Claude Code / OpenClaw / OpenCode / OpenCode-like 真实接入完成。
- provider credential store、provider token / OAuth / keychain / `.env` / auth 文件读取或模型验证完成。
- stop / restart / delete / export / favorite 真实操作完成。
- 自动重试、完整 runtime log、诊断中心或阶段 G 真实 Tauri 全面验收完成。
- 中间版本整体最终验收完成；最终验收仍归 G5。

本轮没有改产品代码，没有新增后端类型 / command / store / read model，没有改前端 UI / TS / wrapper / 样式 / 测试，没有执行真实 `codex exec` 或 `codex exec resume`，没有发送真实 prompt，没有读写 `/Users/yoyi/.codex`，没有读取 auth、token、`.env`、keychain、OAuth、provider credential 或完整 transcript，没有迁移数据库，没有改 `workflow-state.v0.json`。

## 2. 证据完整性

| item | task | evidence | handoff | 复核结果 |
| --- | --- | --- | --- | --- |
| E1 | `tasks/2026-06-05-stage-e-e1-agent-adapter-descriptor-execution-boundary-and-model-credential-readonly-foundation-v1.md` | `evidence/2026-06-05-stage-e-e1-agent-adapter-descriptor-execution-boundary-and-model-credential-readonly-foundation-v1.md` | `handoffs/2026-06-05-stage-e-e1-agent-adapter-descriptor-execution-boundary-and-model-credential-readonly-foundation-v1-result.md` | 齐全；接受为 adapter descriptor / model credential readonly foundation，不接受为 planned adapters 真实接入。 |
| E2 | `tasks/2026-06-05-stage-e-e2-session-operation-boundary-contract-and-readonly-ui-v1.md` | `evidence/2026-06-05-stage-e-e2-session-operation-boundary-contract-and-readonly-ui-v1.md` | `handoffs/2026-06-05-stage-e-e2-session-operation-boundary-contract-and-readonly-ui-v1-result.md` | 齐全；接受为七类 session operation 边界和只读 UI，不接受为真实操作。 |
| E3 | `tasks/2026-06-06-stage-e-e3-model-credential-provider-availability-readonly-boundary-v1.md` | `evidence/2026-06-06-stage-e-e3-model-credential-provider-availability-readonly-boundary-v1.md` | `handoffs/2026-06-06-stage-e-e3-model-credential-provider-availability-readonly-boundary-v1-result.md` | 齐全；接受为 provider / model / credential availability 只读边界，不接受为真实 credential / provider 验证。 |
| E4 | `tasks/2026-06-06-stage-e-e4-session-continuation-protocol-and-permission-preview-v1.md` | `evidence/2026-06-06-stage-e-e4-session-continuation-protocol-and-permission-preview-v1.md` | `handoffs/2026-06-06-stage-e-e4-session-continuation-protocol-and-permission-preview-v1-result.md` | 齐全；接受为 preview / guard。E4 记录过一次 shell 反引号误触发 `codex exec resume` 的过程偏差，没有 prompt，未完成 resume；该偏差不改变产品代码结论，但不能被抹掉。 |
| E5 | `tasks/2026-06-06-stage-e-e5-codex-local-controlled-send-resume-minimal-loop-v1.md` | `evidence/2026-06-06-stage-e-e5-codex-local-controlled-send-resume-minimal-loop-v1.md` | `handoffs/2026-06-06-stage-e-e5-codex-local-controlled-send-resume-minimal-loop-v1-result.md` | 齐全；接受为 Level A，Level B deferred。 |
| E6 | `tasks/2026-06-06-stage-e-e6-runtime-session-attention-and-readback-failure-boundary-v1.md` | `evidence/2026-06-06-stage-e-e6-runtime-session-attention-and-readback-failure-boundary-v1.md` | `handoffs/2026-06-06-stage-e-e6-runtime-session-attention-and-readback-failure-boundary-v1-result.md` | 齐全；接受为 runtime attention / readback failure boundary，不接受为真实执行、自动重试或 runtime log。 |

## 3. Stage E Acceptance Matrix

| item_id | stage_item | title | status | accepted_as | not_accepted_as | evidence_path | handoff_path | deferred_to | notes |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| E1 | E1 | Adapter descriptor execution boundary and model credential readonly foundation | accepted | `codex-local` 与 planned adapters 的 descriptor / execution / credential / model 边界完成 | 外部 agent 真实接入、外部模型或凭据管理、真实执行 | `evidence/2026-06-05-stage-e-e1-agent-adapter-descriptor-execution-boundary-and-model-credential-readonly-foundation-v1.md` | `handoffs/2026-06-05-stage-e-e1-agent-adapter-descriptor-execution-boundary-and-model-credential-readonly-foundation-v1-result.md` | planned adapter real integration | planned adapters 仍无执行按钮、无 implemented actions。 |
| E2 | E2 | Session operation boundary contract and readonly UI | accepted | `send_message` / `stop` / `restart` / `resume` / `export` / `delete` / `favorite` 七类操作边界和只读 UI | 真实发消息、通用 resume、stop / restart、导出、删除、收藏持久化 | `evidence/2026-06-05-stage-e-e2-session-operation-boundary-contract-and-readonly-ui-v1.md` | `handoffs/2026-06-05-stage-e-e2-session-operation-boundary-contract-and-readonly-ui-v1-result.md` | independent operation tasks / G | E2 是契约和禁用态，不是执行能力。 |
| E3 | E3 | Model credential provider availability readonly boundary | accepted | provider / model / credential / external call / cost risk 只读边界 | credential store、token / OAuth / keychain / `.env` 读取验证、provider / model probe、外部模型调用 | `evidence/2026-06-06-stage-e-e3-model-credential-provider-availability-readonly-boundary-v1.md` | `handoffs/2026-06-06-stage-e-e3-model-credential-provider-availability-readonly-boundary-v1-result.md` | provider credential / model verification future task | provider availability 不等于项目授权或会话操作能力。 |
| E4 | E4 | Session continuation protocol and permission preview | accepted_with_deferred_items | session continuation preview、permission preview、guard、prompt summary、readback expectation、audit impact | 真实 send / resume、prompt 已发送、attempt / dispatch / readback / runtime log 已写入 | `evidence/2026-06-06-stage-e-e4-session-continuation-protocol-and-permission-preview-v1.md` | `handoffs/2026-06-06-stage-e-e4-session-continuation-protocol-and-permission-preview-v1-result.md` | E5 Level A / Level B authorization | 产品路径未执行真实 Codex；E4 过程偏差已记录并作为后续搜索 guard。 |
| E5 | E5 | Codex-local controlled send / resume minimal loop Level A | accepted_with_deferred_items | Level A code path、guard、stub、continuation sidecar、audit ref、readback unavailable boundary | 真实 `codex exec resume`、真实 prompt、真实 readback、真实会话继续验收、`.codex` 读写 | `evidence/2026-06-06-stage-e-e5-codex-local-controlled-send-resume-minimal-loop-v1.md` | `handoffs/2026-06-06-stage-e-e5-codex-local-controlled-send-resume-minimal-loop-v1-result.md` | Level B real send / resume explicit authorization, G4 if used in replay | E5 Level B 仍需用户对具体 session、cwd、prompt、读写范围、回滚和证据授权。 |
| E6 | E6 | Runtime session attention and readback failure boundary | accepted | runtime attention、session run status summary、readback failed / unavailable、秘书只读解释 | 真实执行、真实 readback、自动重试、stop / restart、完整 runtime log、诊断中心 | `evidence/2026-06-06-stage-e-e6-runtime-session-attention-and-readback-failure-boundary-v1.md` | `handoffs/2026-06-06-stage-e-e6-runtime-session-attention-and-readback-failure-boundary-v1-result.md` | G1 runtime log, G2 diagnostics, G3 real Tauri | readback unavailable / failed 不能显示成 0 条真实读回。 |
| E-D1 | Stage E deferred | Planned adapters real integration | deferred | 保留 planned / unavailable / no credential / no verified model 边界 | 真实 Claude Code / OpenClaw / OpenCode / OpenCode-like 接入 | E1-E3 evidence | E1-E3 handoff | 后置独立 adapter 任务 / 蓝图层 | 不进入 F1。 |
| E-D2 | Stage E deferred | Provider credential store and model verification | deferred | 只读状态和风险解释 | credential store、provider probe、model verification | E3 evidence | E3 handoff | 后置 provider / credential 专题 | 不读取 secret。 |
| E-D3 | Stage E deferred | Session operations real implementation | deferred | 操作边界和禁用态 | stop / restart / export / delete / favorite 真实操作 | E2 evidence | E2 handoff | independent operation tasks / G | F1 不能把这些显示为可执行。 |
| E-D4 | Stage E deferred | Runtime log / diagnostics / real Tauri | deferred | E6 最小 attention 和 readback boundary | 完整 runtime log、诊断中心、真实 Tauri 全面验收 | E6 evidence | E6 handoff | G1 / G2 / G3 | G 阶段集中补验收和运维闭环。 |
| F-H1 | E-to-F handoff | Project workflow canvas read model consolidation | allowed_next | F1 可读取 E1-E6 边界并做项目画布读模型收敛 | F1 不能实现 Level B、planned adapter、runtime log、diagnostics 或真实 Tauri | 本文件 | `handoffs/2026-06-06-stage-e-e7-session-adapter-model-boundary-acceptance-v1-result.md` | F1 | F1 需要单独任务包。 |

## 4. Blocking / Needs Changes

本轮未发现阻止进入 F1 的 blocking / needs_changes 项。

理由：

- E1-E6 的 evidence / handoff 文件齐全。
- 每个任务的“不接受范围”都保留了真实执行、凭据、planned adapter、runtime log、真实 Tauri 的边界。
- 当前入口文档已同步为 E7 完成和阶段 E `accepted_with_deferred_items`，不再把 E7 标为待执行。
- F1 的工作范围可以只做项目工作流画布读模型收敛，不需要继承 E5 Level B 或 G 阶段能力。

风险：

- E4 曾有一次过程偏差，E7 只能接受为“已记录且不阻断 F1”，不能改写为“阶段 E 全程零触碰 `.codex`”。
- E5 Level A 有 command preview 字符串，但它不是 runner；后续任何 Level B 都必须重新授权。
- 旧历史 evidence / handoff 中存在真实 Codex 历史任务记录，不能拿来证明 E7 或 E5 Level B 已完成。

## 5. E-to-F 准入判断

F1 可以开始，条件是：

- 先写或执行独立 F1 任务包：`tasks/2026-06-06-stage-f-f1-project-workflow-canvas-read-model-consolidation-v1.md` 或等价任务包。
- F1 只收敛项目工作流画布读模型，不改 `workflow-state.v0.json` 顶层结构。
- F1 不新增真实执行、不启动 MCP canvas run、不做复杂编辑器、不把 React Flow 当事实源。
- F1 UI 如果涉及项目页、画布、节点详情、运行中、通知、待办或秘书，必须按 `docs/plans/task-package-ui-display-boundary-rule-v1.md` 写 UI 显示边界。

F1 可继承：

- `agent_adapters[]` / `session_operations[]` / `provider_availability[]` / `session_continuation_previews[]` / `session_continuation_store` / `runtime_session_attention[]` / `session_run_status_summaries[]` 的只读边界。
- planned adapters 不可执行状态。
- E5 Level A continuation sidecar 和 readback unavailable 表达。
- E6 readback failed / unavailable 不能伪装成 0 条结果的规则。

F1 不能继承：

- Level B 真实 send / resume。
- planned adapters 真实接入。
- provider credential / model verification。
- stop / restart / delete / export / favorite 真实操作。
- 自动重试。
- runtime log store。
- diagnostics center。
- 真实 Tauri 全面验收。

## 6. 文档同步

本轮新增：

- `evidence/2026-06-06-stage-e-e7-session-adapter-model-boundary-acceptance-v1.md`
- `handoffs/2026-06-06-stage-e-e7-session-adapter-model-boundary-acceptance-v1-result.md`

本轮同步：

- `tasks/2026-06-06-stage-e-e7-session-adapter-model-boundary-acceptance-v1.md`
- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `docs/plans/2026-06-06-stage-e-f-g-refinement-plan-v1.md`

本轮未修改 `prototypes/**` 产品代码。

## 7. 扫描命令和结果

E7 按任务包要求执行了文档 / 文案扫描。最终扫描结果：

```text
rg -n -F 'E7 仍待写任务包' CURRENT.md tasks/README.md AUTHORITY.md STAGE_PLAN.md README.md docs/plans
```

结果：无命中。

```text
rg -n -F 'E6-E7 仍待写任务包' CURRENT.md tasks/README.md AUTHORITY.md STAGE_PLAN.md README.md docs/plans
```

结果：无命中。

```text
rg -n -F '阶段 E 已完成无 deferred' CURRENT.md tasks/README.md AUTHORITY.md STAGE_PLAN.md README.md docs/plans
```

结果：无命中。

误导文案扫描：

```text
rg -n '真实 prompt 已发送|Codex 已收到任务|真实 readback 已完成|真实会话继续已验收|已自动重试|已停止 agent|Claude Code 已接入|OpenClaw 已接入|OpenCode 已接入|planned adapter 已可执行' CURRENT.md tasks/README.md AUTHORITY.md STAGE_PLAN.md README.md docs/plans tasks evidence handoffs
```

结果：有命中，均为任务包、evidence、handoff 和入口文档中的禁止项、`不接受为`、手动测试清单或历史边界说明；未发现 E7 新增文档把上述能力声明为已完成。

真实执行 / 敏感路径扫描：

```text
rg -n 'Command::new\("codex"\)|codex exec resume|\.codex|read_to_string\(.*auth|read_to_string\(.*token|read_to_string\(.*secret|read_to_string\(.*\.env|keychain|oauth|provider credential' CURRENT.md tasks/README.md AUTHORITY.md STAGE_PLAN.md README.md docs/plans tasks evidence handoffs prototypes/productized-desktop-shell/src-tauri/src prototypes/productized-desktop-shell/src
```

结果：有命中，分类如下：

- 历史真实 workflow runner / MCP runner：既有产品代码中 `Command::new("codex")` 路径，非 E7 新增。
- E4 / E5 / E6 guard 和 fixture：用于阻断 `.codex`、`.env`、auth/token/secret/keychain/OAuth/provider credential 等敏感路径，非读取实现。
- 任务包、evidence、handoff、入口文档中的禁止项、偏差记录、扫描命令和不接受范围。
- E5 command preview 字符串：Level B 前置审批展示，不是 runner。

E7 没有新增产品代码命中。

## 8. 测试说明

E7 没有改产品代码，因此未运行 `npm` / `cargo` 产品测试。

本轮执行的是任务包要求的文档 / 文案扫描。E1-E6 的产品测试结果已经分别记录在各自 evidence / handoff 中；E7 只复核这些证据，不重新解释为新的真实执行验收。
