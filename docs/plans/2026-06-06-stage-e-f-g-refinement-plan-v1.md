# Stage E/F/G Refinement Plan v1

日期：2026-06-06

状态：当前中间版本 E/F/G 细化计划。本文用于确定阶段 E、F、G 的整体任务、顺序、验收口径和禁止边界；不是具体执行任务包，不直接授权开发，不替代 `CURRENT.md`、`tasks/README.md`、`STAGE_PLAN.md` 或 `docs/plans/middleware-version-stage-plan-v1.md`。

## 0. 先说薄弱点

- 阶段 C 和阶段 D / M1-M13 已经完成核心闭环，但阶段 E/F/G 之前还没有像 M1-M13 那样拆成可派发任务，容易在“模型 / 凭据 / 会话发送 / 画布 / 真实验收”之间互相挤占。
- `docs/workbench-frontend-display-boundary-v1.md` 明确写过中间版本最终要支持“发消息继续会话”，但 E2 只完成会话操作边界和只读 UI；后续必须单独补受控发送 / resume 方案和最小闭环，不能把 E2 误认为已经完成。
- 阶段 G 的真实 Tauri 验收曾长期作为 deferred 项记录；当前 G1 runtime log、G2 diagnostics、G3-A 真实 Tauri 验收计划和 G3-C 缺口矩阵已完成，G3-B 已回交但未完成，只接受为 10 / 13 真实 Tauri 部分截图证据。
- GEPA、Paseo、Odysseus 已进入蓝图参考层，但不能因为研究资料存在就扩大当前中间版本范围。
- 阶段 F 不能变成通用可视自动化平台；项目工作流画布必须服务项目主管、任务包、权限、readback、记忆召回和审计。

## 1. 已知事实 / 未知 / 假设

已知事实：

- 阶段 C / C1-C6 已完成，接受为受控自动化工作流闭环。
- 阶段 D / M1-M13 已完成，M13 结论为 `accepted_with_deferred_items`。
- 阶段 E / E1 已完成 adapter descriptor 和模型 / 凭据只读状态底座。
- 阶段 E / E2 已完成会话操作边界契约和只读 UI。
- 阶段 E / E3 已完成模型、凭据和 provider availability 只读边界。
- 阶段 E / E4 已完成会话继续协议和权限预览。
- 阶段 E / E5 已完成 Level A；E5 Level B mario test 独立健康探针也已在用户明确批准后完成，只接受为指定 session 的最小真实 resume 健康探针。
- 阶段 E / E6 已完成 runtime session attention 和 readback failure boundary。
- 阶段 E / E7 已完成，阶段 E 总结论为 `accepted_with_deferred_items`。
- 阶段 F / F1-F5 已完成，阶段 F 总结论为 `accepted_with_deferred_items`。
- 阶段 G / G1 Runtime Log Boundary And Minimal Store、G2 Diagnostics Health And Degraded State、G3-A Real Tauri Acceptance Plan And Fixture Freeze 和 G3-C Screenshot Evidence Recovery And Gap Matrix 已完成；G3-B Real Tauri Manual Screenshot Acceptance 已回交但未完成，真实 Tauri 已启动、目标窗口区域截图探针成功并采集 10 / 13 张编号截图。
- `codex-local` 仍是唯一可用 adapter descriptor；Claude Code / OpenClaw / OpenCode / OpenCode-like 仍是 planned，不可执行。
- 当前仍禁止默认执行 `codex exec` / `codex exec resume`，也禁止默认读写 `/Users/yoyi/.codex`。
- 当前 UI 显示规则要求涉及 UI 的任务包必须包含“UI 显示边界确认”。

未知：

- 真实 `send_message` / `resume` 最终使用现有 workflow dispatch、单独 session controller，还是新 adapter runner。
- 模型 / 凭据状态最终来自 provider probe、配置文件、keychain、用户手动标记，还是外部 adapter 自报。
- 阶段 F 的项目画布是否需要保存用户布局，保存到哪里，以及是否允许用户直接改 workflow 节点。
- 阶段 G / G3-B 剩余高风险截图项是否能在不读取 full transcript、不触发真实 Codex、不写业务状态的前提下完成。

本计划采用的假设：

- E/F/G 仍属于中间版本，不是最终蓝图完整实现。
- E/F/G 目标是收口中间版本必要能力，不接真实外部 agent，不做 GEPA/Paseo/Odysseus 实现。
- `codex-local` 的受控发送 / resume 是阶段 E 必须补的最小闭环，但真实执行必须由单独任务包和用户明确授权控制。
- 阶段 G 可以把 E/F 的真实窗口、截图、运行日志和诊断作为最终补证据任务集中处理。

## 2. 总体收口标准

E/F/G 完成后可以说：

- 中间版本的自动化工作流、完整记忆系统、会话 / adapter / 模型凭据底座、项目工作流画布产品化入口、真实验收和运行诊断已经形成可回收闭环。
- `codex-local` 至少具备受控的会话继续 / resume 最小链路，且不会被误认为通用会话中心自由发送能力。
- planned adapters 继续被正确展示为 planned / unavailable / no credential / no verified model，不会出现假按钮或假执行状态。
- 项目工作流画布能承载项目主管过程所需的节点、任务包、权限、readback、记忆召回、审计和失败信息。
- 阶段 G 能用真实 Tauri、运行日志、诊断、测试和最终报告支撑中间版本验收。

E/F/G 完成后仍不能说：

- 最终蓝图完整工作台完成。
- Claude Code / OpenClaw / OpenCode / OpenCode-like 真实接入完成。
- 通用 stop / restart / delete / export / favorite 完成。
- GEPA 优化器、Paseo daemon、Odysseus workspace 能力进入当前产品实现。
- GraphRAG、向量库、图数据库、Obsidian 原生同步、移动端 relay 或自动技能化完成。
- 工作台可以绕过用户确认、控制核心、任务包、权限、审计或正式记忆状态机。

## 3. 阶段顺序

推荐顺序：

```text
E3 -> E4 -> E5 -> E6 -> E7
-> F1 -> F2 -> F3 -> F4 -> F5
-> G1 -> G2 -> G3 -> G4 -> G5
```

允许的并行：

- F1 可以在 E4 完成后并行准备，但 F3 不能早于 E5 的安全边界结论。
- G1 / G2 可以在 F2 后开始设计，但 G3-G5 必须等 E/F 主任务完成后执行。

不允许的跳跃：

- 不能跳过 E3/E4 直接实现真实发送或 resume。
- 不能跳过 F1/F2 直接做复杂 React Flow 编辑。
- 不能跳过 G1/G2 直接做最终验收报告。

## 4. 阶段 E 细化任务

阶段 E 目标：把会话中心、adapter、多 agent 和模型 / 凭据底座收成可验收形态。阶段 E 不是外部 agent 接入阶段，不是 GEPA/Paseo 实现阶段。

### E1：Agent Adapter Descriptor Execution Boundary

状态：已完成。

任务包：

- `tasks/2026-06-05-stage-e-e1-agent-adapter-descriptor-execution-boundary-and-model-credential-readonly-foundation-v1.md`

接受为：

- `codex-local` 和 planned adapters 的 descriptor / execution boundary / credential readonly foundation。

不接受为：

- 真实外部 agent 接入。
- 模型 / 凭据管理完成。

### E2：Session Operation Boundary Contract

状态：已完成。

任务包：

- `tasks/2026-06-05-stage-e-e2-session-operation-boundary-contract-and-readonly-ui-v1.md`

接受为：

- `send_message` / `stop` / `restart` / `resume` / `export` / `delete` / `favorite` 七类操作边界和只读 UI。

不接受为：

- 真实会话操作完成。
- 通用 `codex exec resume` 完成。

### E3：Model / Credential / Provider Availability Readonly Boundary

状态：已完成。

任务包：

- `tasks/2026-06-06-stage-e-e3-model-credential-provider-availability-readonly-boundary-v1.md`

目标：

- 建立模型、凭据、provider availability、外发风险、成本风险和不可读 secret 的只读边界。

必须完成：

- `ModelProviderDescriptor` / `CredentialBoundaryDescriptor` / `ProviderAvailabilitySummary` 或等价读模型设计。
- `codex-local` 与 planned adapters 的 provider / model / credential 状态可区分。
- UI 只显示 availability、not_configured、not_verified、credential_missing、model_unverified、external_call_blocked 等状态。
- 秘书只读模型能解释模型 / 凭据风险，但不生成配置凭据或调用模型的 action proposal。
- 明确不读取 token、secret、`.env`、keychain、OAuth、provider credential。

不做：

- 不新增真实 credential store。
- 不读取或验证真实外部 provider token。
- 不调用外部模型。
- 不把 provider availability 等同于项目授权。

验收：

- 类型检查和离线 UI 测试。
- Rust 定向测试覆盖 descriptor 不暴露 secret。
- 文案扫描确认没有“已配置凭据 / 已验证模型 / 已接入外部 agent”误导。

### E4：Session Continuation Protocol And Permission Preview

状态：已完成。

任务包：

- `tasks/2026-06-06-stage-e-e4-session-continuation-protocol-and-permission-preview-v1.md`

目标：

- 设计会话继续发送 / resume 的安全协议、权限预览、prompt preview、readback 和失败边界；先不执行真实发送。

必须完成：

- 明确 `send_message` 与 `workflow dispatch resume` 的关系和差异。
- 定义 `SessionContinuationRequest` / `SessionContinuationPreview` / `SessionContinuationGuard` 或等价模型。
- 预览必须显示 target session、project binding、cwd、allowed write roots、sandbox、prompt summary、readback expectation、failure handling、audit impact。
- 控制核心必须能拒绝未绑定项目、越界 cwd、缺少用户确认、planned adapter、敏感路径、无 readback 策略的请求。
- UI 可以出现“预览 / 申请确认”入口，但不能直接执行。

不做：

- 不执行 `codex exec resume`。
- 不写 `/Users/yoyi/.codex`。
- 不发送真实 prompt。
- 不让 planned adapters 拥有继续会话能力。

验收：

- guard 单测覆盖 allowed / blocked / needs_confirmation。
- UI 显示边界测试覆盖“预览不是执行”。
- evidence / handoff 明确 E4 不接受为真实会话继续完成。

### E5：Codex-local Controlled Send / Resume Minimal Loop

任务包：

- `tasks/2026-06-06-stage-e-e5-codex-local-controlled-send-resume-minimal-loop-v1.md`

当前状态：

- 已完成 Level A：代码路径、guard、stub / dry-run、工作台自有 continuation 记录和离线验收。
- 已完成 Level B mario test 健康探针：`tasks/2026-06-06-stage-e-e5-level-b-mario-test-controlled-real-resume-health-probe-v1.md`。该探针在用户明确批准后，对 `/Users/yoyi/Documents/mario test` 的“总指导” session 执行了一次真实 `codex exec resume`，真实写入 `/Users/yoyi/.codex`，last message 返回固定标记，四个项目文件 hash 前后一致。
- Level B 只接受为指定 session 的最小健康探针，不接受为通用 send / resume 产品化；后续任何新的真实 `codex exec resume`、真实 prompt 发送、真实 readback 或读写 `/Users/yoyi/.codex` 仍必须执行前另行取得用户明确授权。

目标：

- 在 E4 协议之上，实现 `codex-local` 受控会话继续 / resume 的最小闭环。它是中间版本“发消息继续会话”的最小实现，不是通用会话控制器。

必须完成：

- 只允许 `codex-local`。
- 必须绑定 project / workflow / node / session。
- 必须经过用户确认弹层。
- 必须写工作台自己的权限、attempt、readback 和 audit 记录。
- 必须显示运行中、成功、失败、超时、readback unavailable 的用户可理解状态。
- 必须保留 command preview 和 readback summary。
- 必须复用或对齐现有 workflow dispatch 安全经验，不能新开无审计后门。

真实执行边界：

- 实现任务包可以写代码路径和离线 / stub 测试。
- 如要在真实 Codex 会话上验收，必须在任务包内单独列出会写 `/Users/yoyi/.codex`、会执行 `codex exec resume`、会写哪些工作台状态，并取得用户明确批准。
- 本轮没有真实执行批准，E5 已回收为“代码路径 / guard / stub 验收完成，真实执行待另行授权或 G 阶段补证据”；不能回收为“真实会话继续已验收完成”。

不做：

- 不支持 Claude Code / OpenClaw / OpenCode。
- 不支持 stop / restart / delete / export / favorite。
- 不支持自由聊天输入框绕过项目和任务包。
- 不把会话中心变成无限制 Codex 控制器。

验收：

- 相关 Rust 和前端测试。
- 禁止文案扫描。
- 如获得真实执行批准，必须有真实 Tauri / readback / audit 证据；否则写清 deferred。

### E6：Runtime Session Attention And Readback Failure Boundary

任务包：

- `tasks/2026-06-06-stage-e-e6-runtime-session-attention-and-readback-failure-boundary-v1.md`

当前状态：

- 已完成。
- E6 只接受为 runtime session attention、readback failed / unavailable 边界、状态摘要 UI 和秘书解释。
- E6 不接受为真实执行、自动重试、stop / restart、完整 runtime log 或阶段 G 真实 Tauri 验收。

目标：

- 把会话运行中、权限待处理、readback 失败、超时、取消限制和需要用户介入的状态收成 read model，服务智能体页、运行中入口、通知和秘书摘要。

必须完成：

- 定义最小 `RuntimeSessionAttention` / `ReadbackFailureReason` / `SessionRunStatusSummary` 或等价读模型。
- 区分 running、waiting_permission、readback_failed、timed_out、blocked_by_guard、needs_user。
- readback 失败不能显示为真实 0 条读回。
- 通知 / 待办 / 运行中入口只显示摘要和跳转，不铺 raw logs。
- 秘书只能提醒和解释，不批准权限、不重试、不继续发送。

不做：

- 不做完整自动重试系统。
- 不做运行日志 store 的最终形态；日志进入 G1。
- 不做 stop / restart。

验收：

- 离线 UI 测试覆盖失败和 readback unavailable。
- 禁止文案扫描，不能出现“已自动重试 / 已停止 agent / 已完成真实派发”误导。

### E7：Stage E Acceptance And E-to-F Handoff

状态：已完成，阶段 E 总结论为 `accepted_with_deferred_items`。

任务包：

- `tasks/2026-06-06-stage-e-e7-session-adapter-model-boundary-acceptance-v1.md`

记录：

- `evidence/2026-06-06-stage-e-e7-session-adapter-model-boundary-acceptance-v1.md`
- `handoffs/2026-06-06-stage-e-e7-session-adapter-model-boundary-acceptance-v1-result.md`

目标：

- 对 E1-E6 做阶段 E 总复核，冻结 accepted / deferred 项，并明确是否允许进入 F。

必须完成：

- E1-E6 evidence / handoff 全部可追溯。
- planned adapters 没有被误写成真实接入。
- 模型 / 凭据没有泄露或被误标为已配置。
- 发送 / resume 最小闭环的真实执行状态被准确记录。
- 阶段 E deferred 项进入 G 或后置蓝图，不挤进 F。

不做：

- 不新增功能。
- 不把 E7 当成 G 最终验收。

## 5. 阶段 F 细化任务

阶段 F 目标：把项目工作流画布产品化为项目主管的主工作界面。阶段 F 不做通用自动化平台，不让 React Flow 成为事实源。

### F1：Project Workflow Canvas Read Model Consolidation

状态：已完成。

推荐任务包名：

- `tasks/2026-06-06-stage-f-f1-project-workflow-canvas-read-model-consolidation-v1.md`

记录：

- `evidence/2026-06-06-stage-f-f1-project-workflow-canvas-read-model-consolidation-v1.md`
- `handoffs/2026-06-06-stage-f-f1-project-workflow-canvas-read-model-consolidation-v1-result.md`

目标：

- 从 workflow state、authorization、task package、memory packet、permission、readback、audit 派生统一项目工作流画布读模型。

必须完成：

- 画布节点 / 边 / 状态 / badge / attention 全部来自后端或稳定读模型。
- React Flow 只承载渲染和交互，不承载事实。
- 空态、blocked、needs_review、prepared、running、ready_for_review、accepted、failed 等状态有一致文案。
- 项目画布主区域只显示摘要，不铺任务包全文、audit 全文、transcript 全文。

不做：

- 不写 workflow state 新顶层结构。
- 不新增真实执行。
- 不做复杂编辑器。

### F2：Node Detail Drawer For Task Package / Memory / Permission / Readback / Audit

推荐任务包名：

- `tasks/2026-06-06-stage-f-f2-workflow-node-detail-drawer-and-evidence-surface-v1.md`

当前状态：已完成。记录见 `evidence/2026-06-06-stage-f-f2-workflow-node-detail-drawer-and-evidence-surface-v1.md` 与 `handoffs/2026-06-06-stage-f-f2-workflow-node-detail-drawer-and-evidence-surface-v1-result.md`。

目标：

- 把节点详情收进右侧抽屉或节点详情面板，承载任务包、记忆召回、权限、readback、失败、evidence、handoff 和 audit 引用。

必须完成：

- 节点详情按层级显示：用户摘要、项目主管信息、技术详情。
- 任务包只显示摘要和 artifact 引用，不把任务包管理器铺进主界面。
- 记忆包显示 included / excluded / review materials 理由。
- 权限和失败信息给出“为什么停下、谁能处理、下一步是什么”。
- audit / evidence / handoff 只显示引用和摘要。

不做：

- 不把右侧详情变成治理后台。
- 不直接在详情里批准高风险权限，必须走确认弹层和控制核心。

### F3：Controlled Workflow Edit Proposal And Layout Boundary

推荐任务包名：

- `tasks/2026-06-07-stage-f-f3-controlled-workflow-edit-proposal-and-layout-boundary-v1.md`

当前状态：

- 已完成。
- 记录见 `evidence/2026-06-07-stage-f-f3-controlled-workflow-edit-proposal-and-layout-boundary-v1.md` 与 `handoffs/2026-06-07-stage-f-f3-controlled-workflow-edit-proposal-and-layout-boundary-v1-result.md`。
- F3 接受为受控工作流编辑提案和布局边界完成，不接受为画布编辑器、布局持久化、workflow mutation 持久 store、真实执行、runtime log、diagnostics、阶段 F 完成或阶段 G 验收。

目标：

- 明确项目画布的编辑边界：哪些只是用户布局，哪些是 workflow 事实变更，哪些必须走控制核心和确认。

必须完成：

- 区分 personal layout、view preference、workflow node mutation、workflow edge mutation。
- 如果保存布局，必须说明保存位置、作用域和可回滚方式。
- workflow 事实变更只能生成 proposal / preview，不能直接拖拽即写事实。
- 删除节点、改边、改角色、改权限、改模型、改工具都必须进入确认和审计。

不做：

- 不做自由画布编辑器。
- 不把 React Flow 拖拽结果直接写 workflow state。
- 不做模板库和节点市场。

### F4：Project Canvas / Experiment Canvas Boundary Hardening

推荐任务包名：

- `tasks/2026-06-07-stage-f-f4-project-canvas-experiment-canvas-boundary-hardening-v1.md`（已完成）

当前状态：

- 已完成。
- 记录见 `evidence/2026-06-07-stage-f-f4-project-canvas-experiment-canvas-boundary-hardening-v1.md` 与 `handoffs/2026-06-07-stage-f-f4-project-canvas-experiment-canvas-boundary-hardening-v1-result.md`。
- F4 接受为项目画布 / 实验画布边界硬化完成，不接受为项目画布和实验画布合一、MCP canvas run 正式项目工作流、真实执行、runtime log、diagnostics、阶段 F 完成或阶段 G 验收。

目标：

- 彻底区分项目工作流画布和一级实验画布，避免实验运行被误认为正式项目运行。

必须完成：

- 一级画布显示 experiment / template / canvas library 语境。
- 项目工作流画布显示 project / workflow / authorization 语境。
- 实验画布不能写正式项目事实、正式记忆或项目任务。
- 项目画布的运行必须经过控制核心、workflow state 和权限边界。
- 禁止文案扫描：不能出现“实验运行已写项目状态 / MCP canvas run 已成为正式 workflow”之类误导。

不做：

- 不启动 MCP canvas run 作为默认项目工作流。
- 不做 ComfyUI / n8n / Langflow 复刻。

### F5：Stage F Acceptance

推荐任务包名：

- `tasks/2026-06-07-stage-f-f5-project-workflow-canvas-productization-acceptance-v1.md`（已完成）

当前状态：

- 已完成。
- 记录见 `evidence/2026-06-07-stage-f-f5-project-workflow-canvas-productization-acceptance-v1.md` 与 `handoffs/2026-06-07-stage-f-f5-project-workflow-canvas-productization-acceptance-v1-result.md`。
- 阶段 F 最终结论为 `accepted_with_deferred_items`；允许进入 G1 Runtime Log Boundary And Minimal Store。

目标：

- 对 F1-F4 做阶段 F 总复核，确认项目工作流画布已能作为中间版本主工作界面进入 G 验收。

必须完成：

- F1-F4 evidence / handoff 可追溯。
- 主画布、节点详情、权限、readback、记忆、审计信息层级符合 UI 边界。
- 独立实验画布和项目画布没有混淆。
- 真实 Tauri / 截图缺口明确交给 G3。

不做：

- 不新增功能。
- 不把 F5 说成中间版本最终验收完成。

## 6. 阶段 G 细化任务

阶段 G 目标：真实 Tauri 验收、运行日志、诊断和中间版本最终收口。阶段 G 不新增产品大能力，只补验收和运维闭环。

### G1：Runtime Log Boundary And Minimal Store

状态：已完成。记录见 `tasks/2026-06-06-stage-g-g1-runtime-log-boundary-and-minimal-store-v1.md`、`evidence/2026-06-07-stage-g-g1-runtime-log-boundary-and-minimal-store-v1.md` 与 `handoffs/2026-06-07-stage-g-g1-runtime-log-boundary-and-minimal-store-v1-result.md`。G1 只接受为 runtime log boundary and minimal store，不接受为 G2 diagnostics、G3 真实 Tauri / 截图验收、G4 回放、G5 最终冻结或阶段 G 完成。

推荐任务包名：

- `tasks/2026-06-06-stage-g-g1-runtime-log-boundary-and-minimal-store-v1.md`

目标：

- 建立运行日志和审计的边界，最小支持 E/F/G 验收所需运行记录。

必须完成：

- 定义 runtime log 与 audit event 的区别。
- 记录 app session、workflow run、dispatch attempt、readback、permission wait、diagnostic event 的最小结构。
- 日志可脱敏展示，不含 token、secret、完整 transcript、raw provider credential。
- 管理入口能显示运行日志摘要和过滤。

不做：

- 不把运行日志当审计。
- 不把审计当运行日志。
- 不做 GEPA eval export。

### G2：Diagnostics / Health / Degraded State

状态：已完成。记录见 `tasks/2026-06-06-stage-g-g2-diagnostics-health-and-degraded-state-v1.md`、`evidence/2026-06-07-stage-g-g2-diagnostics-health-and-degraded-state-v1.md` 与 `handoffs/2026-06-07-stage-g-g2-diagnostics-health-and-degraded-state-v1-result.md`。G2 只接受为 diagnostics / health / degraded state 只读读模型、store integrity、diagnostic bundle 引用和管理入口摘要完成；不接受为自动修复、自动重试、真实 Codex 执行、G3 真实 Tauri / 截图验收、G4 回放、G5 最终冻结或阶段 G 完成。

推荐任务包名：

- `tasks/2026-06-06-stage-g-g2-diagnostics-health-and-degraded-state-v1.md`

目标：

- 建立最小运维诊断体系，能解释 store 损坏、readback 失败、adapter unavailable、Tauri bridge 失败、测试环境缺失等问题。

必须完成：

- `DiagnosticSummary` / `ServiceDegradedState` / `StoreIntegrityFinding` 或等价读模型。
- 检查 workflow state、formal memory、candidate、observation、lint、runtime log 等关键 sidecar 的可读性和 revision。
- 显示 adapter health / provider availability / model credential boundary 的诊断摘要。
- 能导出或引用 diagnostic bundle，但不包含 secret。

不做：

- 不自动修复正式记忆。
- 不自动重试真实 worker。
- 不读取 `/Users/yoyi/.codex` secret 或完整 transcript。

### G3：Real Tauri Acceptance Harness And Screenshot Evidence

状态：G3-A 已完成，记录见 `tasks/2026-06-07-stage-g-g3-a-real-tauri-acceptance-plan-and-fixture-freeze-v1.md`、`evidence/2026-06-07-stage-g-g3-a-real-tauri-acceptance-plan-and-fixture-freeze-v1.md` 与 `handoffs/2026-06-07-stage-g-g3-a-real-tauri-acceptance-plan-and-fixture-freeze-v1-result.md`。G3-B Real Tauri Manual Screenshot Acceptance 已回交但未完成，记录见 `tasks/2026-06-07-stage-g-g3-b-real-tauri-manual-screenshot-acceptance-v1.md`、`evidence/2026-06-07-stage-g-g3-b-real-tauri-manual-screenshot-acceptance-v1.md` 与 `handoffs/2026-06-07-stage-g-g3-b-real-tauri-manual-screenshot-acceptance-v1-result.md`；真实 Tauri 已启动、目标窗口区域截图探针成功并采集 10 / 13 张编号截图。G3-C Screenshot Evidence Recovery And Gap Matrix 已完成，记录见 `tasks/2026-06-07-stage-g-g3-c-screenshot-evidence-recovery-and-gap-matrix-v1.md`、`evidence/2026-06-07-stage-g-g3-c-screenshot-evidence-recovery-and-gap-matrix-v1.md` 与 `handoffs/2026-06-07-stage-g-g3-c-screenshot-evidence-recovery-and-gap-matrix-v1-result.md`。G3-A 只接受为真实 Tauri 验收计划和 fixture freeze；G3-B 只接受为 10 / 13 真实 Tauri 部分截图证据；G3-C 只接受为截图证据回收和缺口矩阵完成；不接受为 G3 全量真实 Tauri验收、G4 回放、G5 最终冻结或阶段 G 完成。

推荐任务包名：

- `tasks/2026-06-07-stage-g-g3-a-real-tauri-acceptance-plan-and-fixture-freeze-v1.md`（已完成）
- `tasks/2026-06-07-stage-g-g3-b-real-tauri-manual-screenshot-acceptance-v1.md`（已回交但未完成）
- `tasks/2026-06-07-stage-g-g3-c-screenshot-evidence-recovery-and-gap-matrix-v1.md`（已完成）

目标：

- 补齐真实 Tauri 窗口 / 截图 / 手动清单验收，不再只依赖 Vite HTTP smoke 或普通浏览器壳。

必须覆盖：

- 权限确认弹层。
- 项目页和项目工作流画布。
- 智能体会话中心、adapter / session operation / send-resume 状态。
- 记忆中心。
- 知识库。
- 任务记忆包预览。
- 运行中 / 通知 / 待办 / 管理中的日志和诊断摘要。

必须完成：

- 明确截图工具、保存路径、截图命名和 evidence 引用方式。
- 记录真实 Tauri 环境、数据 fixture、操作步骤和结果。
- 不能把普通浏览器 smoke 说成真实 Tauri 验收。

不做：

- 不新增产品能力。
- 不为截图绕过安全边界读取 secret。

### G4：Middle Version End-to-End Acceptance Replay

推荐任务包名：

- `tasks/2026-06-06-stage-g-g4-middle-version-end-to-end-acceptance-replay-v1.md`

目标：

- 用一组受控 fixture / 测试项目回放中间版本主链路，证明阶段 C、D、E、F、G 之间能被用户理解并可回收。

必须覆盖：

- 用户确认方案。
- 全局主管边界复核。
- 项目主管拆任务。
- 任务记忆包注入。
- worker 汇报 / 项目主管过程事实确认。
- 记忆候选 / 正式记忆 / lifecycle / lint / maintenance / mature pattern gate。
- 会话继续 / resume 的状态和权限边界。
- 项目工作流画布和节点详情。
- 运行日志、诊断和最终结果摘要。

真实执行边界：

- 默认优先用受控 fixture / 离线回放。
- 如要跑真实 Codex / `codex exec resume`，必须单独取得用户批准并写清读写范围。

不做：

- 不把单次 demo 当完整验收。
- 不把缺失 readback 伪装成真实 0 条结果。

### G5：Final Authoritative Acceptance And Deferred Freeze

推荐任务包名：

- `tasks/2026-06-06-stage-g-g5-final-authoritative-acceptance-and-deferred-freeze-v1.md`

目标：

- 冻结中间版本最终结论、完成项、deferred 项、真实验收材料和下一阶段建议。

必须完成：

- 汇总 C1-C6、M1-M13、E1-E7、F1-F5、G1-G4 的结论。
- 给出最终状态：`accepted` / `accepted_with_deferred_items` / `needs_changes`。
- 明确 deferred 项是否进入最终蓝图、阶段 H、backlog 或研究层。
- 更新 `CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md` 和必要阶段计划。
- 给下一任全局主管一份简短 handoff。

不做：

- 不新增产品功能。
- 不把最终蓝图能力包装成中间版本已完成。

## 7. 当前下一步

E3、E4、E5 Level A、E5 Level B mario test 健康探针、E6 和 E7 已完成；阶段 E 总结论为 `accepted_with_deferred_items`。F1 项目工作流画布读模型收敛已完成，F2 节点详情 / evidence surface 已完成，F3 Controlled Workflow Edit Proposal And Layout Boundary 已完成，F4 Project Canvas / Experiment Canvas Boundary Hardening 已完成，F5 Stage F Acceptance 已完成；阶段 F 最终结论为 `accepted_with_deferred_items`：

- F1 任务包：`tasks/2026-06-06-stage-f-f1-project-workflow-canvas-read-model-consolidation-v1.md`
- F1 记录：`evidence/2026-06-06-stage-f-f1-project-workflow-canvas-read-model-consolidation-v1.md` 与 `handoffs/2026-06-06-stage-f-f1-project-workflow-canvas-read-model-consolidation-v1-result.md`
- F2 任务包：`tasks/2026-06-06-stage-f-f2-workflow-node-detail-drawer-and-evidence-surface-v1.md`
- F2 记录：`evidence/2026-06-06-stage-f-f2-workflow-node-detail-drawer-and-evidence-surface-v1.md` 与 `handoffs/2026-06-06-stage-f-f2-workflow-node-detail-drawer-and-evidence-surface-v1-result.md`
- F3 任务包：`tasks/2026-06-07-stage-f-f3-controlled-workflow-edit-proposal-and-layout-boundary-v1.md`
- F3 记录：`evidence/2026-06-07-stage-f-f3-controlled-workflow-edit-proposal-and-layout-boundary-v1.md` 与 `handoffs/2026-06-07-stage-f-f3-controlled-workflow-edit-proposal-and-layout-boundary-v1-result.md`
- F4 任务包：`tasks/2026-06-07-stage-f-f4-project-canvas-experiment-canvas-boundary-hardening-v1.md`
- F4 记录：`evidence/2026-06-07-stage-f-f4-project-canvas-experiment-canvas-boundary-hardening-v1.md` 与 `handoffs/2026-06-07-stage-f-f4-project-canvas-experiment-canvas-boundary-hardening-v1-result.md`
- F5 任务包：`tasks/2026-06-07-stage-f-f5-project-workflow-canvas-productization-acceptance-v1.md`
- F5 记录：`evidence/2026-06-07-stage-f-f5-project-workflow-canvas-productization-acceptance-v1.md` 与 `handoffs/2026-06-07-stage-f-f5-project-workflow-canvas-productization-acceptance-v1-result.md`
- F5 只接受为阶段 F 总复核和进入 G 的 readiness 判断任务，不接受为阶段 G、runtime log、diagnostics、真实 Tauri / 截图验收或中间版本最终验收完成。

- `tasks/2026-06-06-stage-e-e5-level-b-mario-test-controlled-real-resume-health-probe-v1.md` 已完成，记录见 `evidence/2026-06-06-stage-e-e5-level-b-mario-test-controlled-real-resume-health-probe-v1.md` 与 `handoffs/2026-06-06-stage-e-e5-level-b-mario-test-controlled-real-resume-health-probe-v1-result.md`。

- `tasks/2026-06-06-stage-e-e6-runtime-session-attention-and-readback-failure-boundary-v1.md` 已完成，记录见 `evidence/2026-06-06-stage-e-e6-runtime-session-attention-and-readback-failure-boundary-v1.md` 与 `handoffs/2026-06-06-stage-e-e6-runtime-session-attention-and-readback-failure-boundary-v1-result.md`

E7 已完成任务包为：

- `tasks/2026-06-06-stage-e-e7-session-adapter-model-boundary-acceptance-v1.md`

E5 已完成任务包为：

- `tasks/2026-06-06-stage-e-e5-codex-local-controlled-send-resume-minimal-loop-v1.md`

E5 已按任务包边界完成 Level A 和 Level B 健康探针：Level A 是代码路径、guard、stub / dry-run、工作台自有 continuation 记录和离线验收；Level B 是指定 mario test 总指导 session 的最小真实 resume 健康探针。如后续涉及新的真实 `codex exec resume`、真实 prompt 发送、真实 readback 或 `/Users/yoyi/.codex` 读写，仍必须先取得用户明确授权。

E6 已严格按任务包边界回收；未进入 E5 Level B，未新增完整 runtime log store，未做自动重试，未把 readback unavailable 显示成 0 条结果。

E7 已冻结 Stage E acceptance matrix 和 E-to-F handoff；F1 已在独立任务包中完成，但不能继承 Level B、planned adapter 真实接入、provider credential 验证、runtime log、diagnostics 或真实 Tauri 验收。当前 Level B 健康探针即使通过，也只能接受为最小真实 resume 健康探针完成，不能扩大为 G1 或通用会话控制能力完成。

G1、G2、G3-A、G3-C、G4 和 G5 已完成；G3-B Real Tauri Manual Screenshot Acceptance 已回交但未完成，只接受为 10 / 13 真实 Tauri 部分截图证据。G3-C 已冻结缺口矩阵并允许 G4 离线回放；G4 已按离线端到端回放收口；G5 已冻结中间版本最终结论为 `accepted_with_deferred_items`。

## 8. 统一禁止项

E/F/G 全阶段都禁止：

- 不默认执行真实 Codex。
- 不默认执行 `codex exec` / `codex exec resume`。
- 不默认读写 `/Users/yoyi/.codex`。
- 不读取 auth、token、`.env`、keychain、OAuth、provider credential。
- 不新增数据库迁移，除非任务包含迁移、双写、备份、回滚。
- 不改 `workflow-state.v0.json` 顶层结构，除非任务包明确授权。
- 不让 planned adapters 显示为可执行。
- 不把 GEPA / Paseo / Odysseus 研究项并入当前实现。
- 不把 LLM 摘要、readback 失败、timeline event、runtime log、knowledge hit 或 tool output 当正式事实 / 正式记忆。

## 9. 任务包固定要求

后续 E/F/G 每个任务包必须包含：

- 已知事实 / 未知 / 假设。
- 接受范围和不接受范围。
- 必读文件。
- UI 显示边界确认，如果涉及前端、读模型展示、文案、导航、右侧入口、项目页、画布、智能体、记忆、知识库、秘书、通知、待办、运行中或管理入口。
- 后端 / 前端 / 文档修改范围。
- 禁止项。
- 验收命令。
- 禁止文案扫描。
- evidence / handoff 要求。
- 如涉及真实 Codex、`/Users/yoyi/.codex`、真实 Tauri、截图或外部 provider，必须有单独权限和边界声明。

## 10. 权威入口同步要求

本文被采纳后，需要同步：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/plans/middleware-version-stage-plan-v1.md`

同步后，当前入口应指向：

```text
E/F/G 细化计划已完成
-> E3 已完成
-> E4 已完成
-> E5 Level A 已完成；真实执行前必须取得用户授权
-> E6 已完成
-> F1 已完成
-> F2 已完成
-> F3 已完成
-> F4 已完成
-> F5 已完成，阶段 F 结论冻结为 accepted_with_deferred_items
-> G1 已完成
-> G2 已完成
-> G3-A 已完成
-> G3-B 已回交但未完成，只接受为 10 / 13 真实 Tauri 部分截图证据
-> G3-C 已完成缺口矩阵
-> G4 已完成离线端到端回放，结论为 accepted_with_deferred_items
-> G5 已完成最终权威验收和 deferred freeze
-> 中间版本最终结论冻结为 accepted_with_deferred_items
```
