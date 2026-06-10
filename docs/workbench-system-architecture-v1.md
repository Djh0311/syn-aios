# Workbench System Architecture v1

状态：最终蓝图下的软件开发架构设计草案。

本文只定义软件架构，不重新定义首页 UI 内容，不替代最终蓝图、`CURRENT.md`、`AUTHORITY.md`、`decisions/**` 或阶段计划。

最终蓝图来源：

- `/Users/yoyi/Documents/Codex/2026-05-26/gan-xing-codexbridge-https-github-com/docs/architecture/local-ai-workbench-blueprint-v1.md`
- `/Users/yoyi/Documents/Codex/2026-05-26/gan-xing-codexbridge-https-github-com/docs/architecture/local-ai-workbench-ui-blueprint-v1.md`

当前相关设计：

- `docs/workflow-task-package-design-v1.md`
- `docs/memory-layer-design-v1.md`
- `decisions/2026-05-28-extensible-first-development-rule.md`
- `decisions/2026-05-28-codex-workflow-min-model.md`
- `decisions/2026-05-29-codex-session-workflow-route-correction.md`
- `decisions/2026-05-31-editable-canvas-codex-as-director-v1.md`

## 1. 先说薄弱点

当前还没有完整落地的系统架构实现。

已有的是：

- 技术栈方向。
- 最小工作流数据模型。
- 工作流事实层 v0 存储。
- 记忆层设计草案。
- 工作流和任务包设计草案。
- 局部 MCP 画布运行骨架。

缺的是统一约束：

- 核心模块边界。
- 外部能力接入方式。
- 项目隔离方式。
- 秘书在核心里的位置。
- 记忆治理和记忆实现的分工。
- 状态、事件、审计、读模型之间的关系。
- 当前代码应该如何逐步收敛。

本文补这个缺口。

## 2. 架构结论

本地 AI 工作台采用：

**本地模块化单体运行形态 + 项目单元隔离 + 控制核心 + 项目黑板 + 能力适配器 + 事件账本和当前快照 + 核心记忆治理 + 秘书核心协作层。**

大白话：

- 工作台是一个本地桌面应用，不拆微服务。
- 项目是最高级业务对象，也是主要隔离单元。
- 控制核心掌握事实、权限、状态机、策略和审计。
- 智能体、工具、知识库、模型、harness、代码图谱都通过适配器接入。
- 多个智能体围绕项目协作时，先把中间结果放到项目黑板。
- 只有控制核心能把候选、汇报、工具摘要或建议升级为正式事实。
- 记忆层是核心能力，但进入核心的是记忆治理，不是具体检索或存储实现。
- 秘书是核心协作角色，不是某个页面、侧栏或悬浮入口的附属功能。

## 3. 不采用的架构

### 3.1 不做微服务

原因：

- 工作台是本地桌面软件。
- 微服务会增加本地部署、进程管理、通信、日志和故障恢复成本。
- 当前最大问题不是横向扩展，而是事实、权限、状态和多智能体协作边界。

### 3.2 不做通用节点自动化平台

原因：

- 工作台不是 n8n、Dify、Langflow 或 ComfyUI。
- 工作流画布服务项目主管、角色协作、权限、审计和记忆候选。
- 外部工具节点不能绕过项目规则和控制核心。

### 3.3 不把纯微核作为主架构

原因：

- 纯微核容易过早把重点放到插件系统、动态加载和扩展市场。
- 当前第一问题是项目协作和事实治理，不是插件生态。

### 3.4 不做全量事件溯源

原因：

- 关键事实动作必须写事件。
- 但所有状态都靠事件重放会让 v1 过重。
- 本地工作台更适合事件账本 + 当前快照 + 派生读模型。

## 4. 运行形态

运行形态是本地模块化单体。

当前技术栈：

- 桌面壳：Tauri 2
- 本地核心：Rust
- 前端：React + TypeScript + Vite
- 画布方向：React Flow
- v0 事实层：JSON 文件
- 长期事实库方向：SQLite + FTS

约束：

- 前端不能直接写事实状态。
- Rust 后端统一处理状态变更、权限确认、审计写入和适配器调用。
- 外部能力不能直接写核心事实层。
- 当前可以 Codex-only，但模型、接口和状态不能写死为 Codex-only。

## 5. 核心分层

### 5.1 界面层

职责：

- 展示首页、项目、画布、会话、知识库、记忆、通知、待办、审计等界面。
- 发起用户命令。
- 展示确认弹层、右侧详情、读模型和状态。

不负责：

- 判断状态转移是否合法。
- 直接写 workflow state。
- 直接写记忆。
- 直接调用 Codex、工具、模型或文件系统完成业务动作。

首页说明：

- 首页 UI 内容已经由界面蓝图和后续 UI 决策确定。
- 本文不重新定义首页首屏内容、布局或入口优先级。
- 架构只规定首页读取哪个读模型、不能绕过事实层。

### 5.2 应用服务层

职责：

- 把用户操作变成命令。
- 调用控制核心校验权限、策略和状态机。
- 调用适配器执行外部动作。
- 写事件、审计和当前快照。
- 生成前端读模型。

示例命令：

- 创建项目。
- 绑定会话。
- 启动工作流。
- 派发给智能体。
- 批准权限。
- 记录主管回收。
- 采纳记忆候选。
- 废弃记忆。
- 生成审计补救建议。

### 5.3 控制核心

控制核心是工作台最硬的部分。

职责：

- 项目身份和项目隔离。
- 会话归属。
- 角色和责任边界。
- 工作流状态机。
- 权限和策略。
- 审计规则。
- 任务完成判定。
- 候选转事实。
- 记忆治理。
- 适配器能力校验。
- 秘书核心协作规则。

不负责：

- Codex 本地文件格式细节。
- Obsidian vault 解析细节。
- 向量库实现。
- 图谱索引实现。
- 模型供应商 API。
- 工具调用具体参数。
- UI 页面状态。

### 5.4 项目黑板层

项目黑板保存项目内协作中间态。

黑板内容包括：

- 当前目标。
- 工作流节点。
- 子智能体汇报。
- 证据引用。
- 权限请求。
- 工具调用摘要。
- 审查结果。
- 风险和阻塞。
- 记忆候选。
- 知识库引用。
- 待用户确认的建议。

边界：

- 黑板不是正式事实源。
- 子智能体、工具、知识库和模型只能向黑板提交结果、摘要、候选或请求。
- 控制核心确认后，黑板内容才能升级为正式事实、正式记忆、审计事件或状态变化。

### 5.5 事实层

事实层保存系统可追踪事实。

v1 事实层由三类组成：

- 事件账本：关键动作的不可变记录。
- 当前快照：当前项目、工作流、节点、权限、记忆等状态。
- 审计账本：权限、状态、工具、模型、harness、记忆等关键事实的审计记录。

长期可增加：

- SQLite 表。
- FTS 搜索。
- 派生读模型。
- 可重建索引。

边界：

- 聊天上下文不是事实层。
- 检索命中不是事实层。
- LLM 生成的摘要不是事实层。
- 工具返回全文不是事实层。

### 5.6 适配器层

外部能力全部通过适配器接入。

适配器类型：

- AgentAdapter：Codex、Claude Code、OpenClaw、OpenCode、VS Code 等。
- KnowledgeAdapter：Obsidian-compatible 知识库、本地文档库等。
- MemoryStorageAdapter：JSON、SQLite、未来图数据库或向量库。
- ModelAdapter：本地模型和云模型。
- ToolAdapter：文件、命令、浏览器、API、MCP 工具等。
- HarnessAdapter：项目协议、验证入口、检查器。
- CodeIndexAdapter：代码结构索引、影响范围分析。

适配器只能：

- 声明能力。
- 接收受控命令。
- 返回结果、摘要、证据引用或错误。

适配器不能：

- 直接推进工作流状态。
- 直接写正式记忆。
- 直接删除审计。
- 直接提升权限。
- 直接跨项目读写。

### 5.7 后置优化层

工作台可以在最终蓝图后期引入优化层，例如 GEPA 这类 prompt / template / agent instruction 优化框架。

推荐位置：

```text
核心外 OptimizationAdapter / PromptOptimizationService
```

优化层可以读取受控导出的材料：

- 脱敏运行日志。
- adapter trace。
- worker report。
- readback failure。
- 任务包 artifact。
- 记忆包 included / excluded 理由。
- eval case。
- 用户 / 项目主管 / 全局主管结果反馈。

优化层只能输出候选和报告：

- `PromptCandidate`
- `TaskPackageTemplateCandidate`
- `WorkflowTemplateCandidate`
- `AdapterInstructionCandidate`
- `SkillCandidate`
- `MaturePatternCandidate`
- `OptimizationReport`

优化层可以优化的对象：

- `PromptComponentVersion`
- `TaskPackageTemplateVersion`
- `AdapterInstructionVersion`
- `WorkflowNodeInstructionVersion`
- `SkillVersion`
- `MaturePatternCandidate`
- 解释性模板，例如 readback failure diagnosis、memory impact report、worker report summarization。

优化层不能优化的对象：

- 权限 guard。
- 正式记忆写入规则。
- 用户偏好判定规则。
- 全局主管授权规则。
- workflow state 状态机。
- 真实 agent 执行策略。
- shell / file / MCP / tool permission。

优化层不能：

- 直接写正式记忆。
- 直接写 workflow state。
- 直接改生产 prompt。
- 直接改 adapter 权限。
- 直接调用真实 worker。
- 直接调用 shell、MCP、文件、浏览器或真实工具。
- 直接删除旧版本或审计。
- 绕过项目主管、全局主管或用户确认。

接入前置条件：

- 稳定运行日志和诊断体系。
- 可复现 eval set。
- prompt / template 版本对象。
- 成本预算和模型外发边界。
- trace 脱敏规则。
- 回滚机制。
- 候选采纳审计。
- prompt / template / skill 版本 registry。
- eval case store。
- optimization run store。
- cost budget guard。

阶段边界：

- 当前阶段 E 只允许保留架构预留意识，不把优化器作为当前 E1 / E2 主线。
- 真正运行优化器必须等阶段 G 的运行日志、诊断、eval、成本、脱敏和回滚底座完成。
- 优化报告不是事实源；进入正式记忆、成熟模式、技能或生产 prompt 前，必须走候选、版本、权限、审计和确认流程。

GEPA 类优化器的推荐融合路线：

```text
GEPA-0 data contract design
-> GEPA-1 redacted eval export
-> GEPA-2 isolated dry run
-> GEPA-3 candidate governance
-> GEPA-4 controlled publish
```

GEPA-0 只设计数据契约，不接 SDK：

- `PromptComponentVersion`
- `EvalCase`
- `EvalMetric`
- `RedactedTrace`
- `OptimizationCandidate`
- `OptimizationReport`
- `OptimizationAuditEvent`

GEPA-1 只导出脱敏 eval 包，不运行优化器：

- 从真实运行日志生成 eval cases。
- 证明敏感信息已脱敏。
- 按项目、adapter、task kind 和 risk class 过滤。

GEPA-2 只在隔离目录小预算 dry run：

- 不改生产 prompt。
- 不改正式记忆。
- 不改 workflow state。
- 不调用真实 worker。
- 只产生 `OptimizationReport`。

GEPA-3 把输出转成候选：

- 候选必须有 diff。
- 候选必须有影响面。
- 候选必须列出基于哪些 eval case / trace。
- 候选必须有回滚计划。
- 候选不能绕过项目主管、全局主管或用户确认。

GEPA-4 才允许受控发布：

- 发布必须写版本和审计。
- 必须可回滚。
- 必须能限制到某项目、adapter、task kind、risk class 或语言范围。
- 不能用单一平均分做全局发布依据；需要保留 Pareto frontier、适用范围和反例。

### 5.8 外部 workspace 参考约束

Odysseus 这类自托管 AI workspace 可以作为最终蓝图的外部参考，但不能覆盖本工作台的项目主轴。

参考资料：

- `docs/research/2026-06-05-odysseus-workbench-deep-reference-research-v2.md`

可吸收为蓝图约束：

- Agent / Tool capability registry 不能只声明能力，还必须声明风险等级、owner scope、credential / model access、外发风险和是否需要任务包授权。
- MCP、shell、file、model serving、token、webhook、email、calendar 等高风险工具必须经过控制核心、项目边界、任务包允许清单、用户确认和审计；不能因为 adapter 已接入就自动可用。
- Workspace confinement、敏感路径 deny list、路径允许根、网络外发边界和 provider endpoint scope 必须进入后续执行沙箱设计。
- Prompt injection 防线必须把网页、知识库资料、邮件、transcript、tool output、saved memory、skill text 等外部内容标成不可信材料；它们不能被提升为主管指令或正式事实。
- Deep Research 应建模为工作流 run：包含 status、progress、cancel、sources、fallback、result refs 和 failure reason；报告输出先进入知识库材料、Observation 或 MemoryCandidate，不能直接写正式记忆或项目事实。
- Runtime log、AdapterHealth、ServiceDegradedState、ReadbackFailureReason、ToolExecutionLog、DiagnosticBundleExport、IndexDegradedReport 是阶段 G 和最终蓝图必须预留的运维对象。
- Skill 层必须和 Memory 层分开：Skill 可以成为候选能力或模板，但不能绕过正式记忆状态机、版本、权限、审计和用户确认。
- 向量库、索引和检索层只能是可重建派生能力；索引降级不能导致正式事实或正式记忆丢失，也不能把召回失败伪装成“没有相关材料”。

明确不吸收：

- 不把 Chat 作为最高级业务对象；项目仍是最高级业务对象。
- 不把大工具箱、Cookbook、Email、Calendar、MCP、Shell、Gallery、Tasks 全部做成一级入口。
- 不允许 agent 自由调用工具完成整件事；worker 只能读取任务包允许的最小上下文和工具。
- 不允许 MCP 直接 add / edit / delete FormalMemory。
- 不允许 vector memory 或 LLM memory extraction 直接影响 agent 行为。
- 不允许 admin 能力绕过项目、方案授权、任务包、控制核心和用户确认。
- 不让 app API 成为万能后门。
- 不把 Deep Research 报告自动变成项目事实。
- 不把 raw logs、vector id、adapter schema、provider secret 或路径大表默认显示给普通用户。

阶段边界：

- 当前中间版本阶段 E / E2 只执行已拆任务包，不因为 Odysseus v2 存在而扩展范围。
- Odysseus v2 不进入当前 backlog，不拆任务包，不授权实现。
- 后续如要吸收具体能力，必须先从本蓝图约束倒推专题设计，再由全局主管批准阶段计划和任务包。

### 5.9 外部 agent runtime 参考约束

Paseo 这类 coding agent orchestration / daemon 工具可以作为最终蓝图的多 agent 运行层参考，但不能覆盖本工作台的项目主管制、方案授权制、控制核心、任务包和正式记忆治理。

参考资料：

- `docs/research/2026-06-05-paseo-workbench-deep-reference-research-v1.md`

可吸收为蓝图约束：

- Agent runtime control plane 需要有后端 authoritative read model，前端不能自己拼真实 agent 状态。
- Agent lifecycle、runtime session、timeline、permission request、attention、provider snapshot 和 logs / audit 分层需要进入阶段 E / G 的后续专题设计。
- Provider adapter 只能声明 capability、mode、model、feature、availability、diagnostic 和 persistence handle；技术可用不等于当前项目已授权可用。
- 会话中心未来如果从 transcript viewer 升级为 runtime session viewer，必须区分 live stream、authoritative fetch、sequence dedupe、pagination、tool call folding、permission、attention 和 error UI。
- 多 agent parent / child / detached 关系必须来自项目主管任务包和控制核心派发，不能来自 agent 自治创建。
- Worktree isolation、setup / teardown、script、service、diff review 和 service proxy 可作为阶段 F / G 专题研究对象；它们不是完整 sandbox，也不能绕过路径、凭据和 destructive command guard。
- Schedule / loop / verifier 可以作为 long-running run object 和运维模型参考，但不能替代用户方案授权、项目主管过程确认或全局主管最终复核。
- Remote / mobile / relay 如果进入最终蓝图，必须优先建模 pairing、E2E encryption、host allowlist、password auth、DNS rebinding protection 和 daemon exposure diagnostics。
- Runtime timeline、logs、tool output 和 loop report 只能作为 Observation 来源，不能直接写正式事实或正式记忆。
- CLI parity 是后续测试和运维参考：app 能做的运行层动作应有受控 API / CLI 语义，但 CLI 也必须经过同一控制核心、权限和审计边界。

明确不吸收：

- 不把 agent / chat / timeline 作为最高级业务对象；项目仍是最高级业务对象。
- 不允许 agent 通过 MCP / CLI / skill autopilot 直接创建、取消、归档、kill 或批准正式 worker。
- 不允许 provider mode、bypass、full-access 或技术 capability 绕过方案授权、任务包允许清单、用户确认和审计。
- 不允许 timeline event、log row、tool output 或 verifier report 自动成为项目事实、Observation 结论、MemoryCandidate 或 FormalMemory。
- 不允许 schedule / loop 绕过控制核心反复派发真实 worker。
- 不允许 worktree setup 复制 `.env`、token、secret、keychain、OAuth 或 provider credential 而没有权限、脱敏和审计。
- 不允许 relay / direct daemon 暴露细节默认展示给普通用户；安全诊断应进入管理 / 运维入口并做风险分级。
- 不用 file-based optional schemas 支撑正式记忆、权限、版本和审计的长期治理。

推荐后续研究路线：

```text
PASEO-0 Agent Runtime Control Plane 对比设计
-> PASEO-1 Provider Adapter Contract 对齐
-> PASEO-2 Agent Timeline 和会话中心设计
-> PASEO-3 Worktree Isolation 和 Workflow Node
-> PASEO-4 Remote / Mobile / Relay 安全研究
```

阶段边界：

- Paseo 只作为阶段 E / F / G 的后续专题研究依据，不进入当前 E1 / E2 主线。
- Paseo 不进入当前 backlog，不拆任务包，不授权实现。
- 后续如要吸收具体能力，必须先从本蓝图约束倒推专题设计，再由全局主管批准阶段计划和任务包。

### 5.10 Codex 多线程协作参考约束

Codex 当前已有类似主管线向开发线派发任务、开发线完成后回交主管线复核的多线程协作能力。它对工作台有参考价值，但只能作为协作架构模式参考，不能替代工作台自己的项目、工作流、权限、审计和记忆模型。

参考资料：

- `docs/plans/2026-06-07-stage-h-i-real-codex-automation-and-multi-agent-collaboration-plan-v1.md`

可吸收为蓝图约束：

- 主管线负责拆解、派发、回收和复核；开发线负责单任务执行；验证线 / 回收线可以作为职责分离参考。
- 每条执行线必须有明确目标、输入、允许范围、交付物和回交协议。
- 主管复核必须看 evidence / handoff / 测试 / 边界，而不是只看 worker 自述。
- 多线程协作的结果必须进入工作台自己的 `WorkerHandoff`、`ReadbackResult`、`RuntimeLog`、`AuditEvent` 和记忆候选链路。
- 任务派发必须来自项目主管任务包和控制核心授权，不能来自 agent 自治 spawn。

明确不吸收：

- 不把 Codex thread id 当成工作台永久业务主键。
- 不把 Codex 的 parent / child / subagent 关系直接当成项目主管 / worker / 验证线 / 回收线关系。
- 不把 Codex 线程状态当成 workflow state。
- 不让 Codex 多线程协作绕过任务包、权限、运行日志、审计、记忆状态机或用户确认。
- 不把 Codex-only 的 send / resume UI 设计成所有 agent 的通用 UI。

后续阶段：

```text
H0-H7：先把 codex-local 真实自动化工作流产品化
-> I0-I6：再抽象 WorkerAdapter / RunUnit / DispatchRequest / PermissionEnvelope / WorkerHandoff / ReadbackResult / RuntimeLog / AuditEvent
-> planned adapters 后续独立产品化
```

阶段边界：

- 阶段 H 只产品化 `codex-local`，不接 planned adapters 真实执行。
- 阶段 I 先做中立协作抽象，不把 abstraction 完成说成 Claude Code / OpenClaw / OpenCode 已接入。
- provider credential / model verification 仍需后续独立任务，不能因为 Codex 多线程参考存在而提前开放。

## 6. 项目单元

项目是最高级业务对象。

项目单元包含：

- 项目身份。
- 项目主管。
- 项目工作流。
- 项目会话。
- 项目知识权限。
- 项目记忆。
- 项目黑板。
- 项目待办。
- 项目通知。
- 项目审计摘要。
- 项目启用技能。
- 项目启用 harness。
- 项目模型池。

项目单元边界：

- 会话必须归属于一个项目。
- 会话不能随意跨项目迁移。
- 子智能体只能读任务包或权限允许的项目范围。
- 项目知识库权限按任务临时开放。
- 项目记忆不能直接污染全局记忆。
- 项目审计摘要来自全局审计事实的项目视图。

## 7. 秘书核心协作层

秘书是核心协作角色，不是某个界面。

秘书和工作台是共生关系：

- 工作台提供事实、状态、权限、审计、记忆和工具边界。
- 秘书帮助用户理解状态、发现冲突、整理建议、推动确认和汇总长期变化。

秘书可以出现在：

- 首页。
- 项目页。
- 画布。
- 通知。
- 待办。
- 记忆管理。
- 审计回看。
- 全局对话入口。

但架构不能规定秘书只属于某个界面。

秘书在核心里的职责：

- 汇总项目和全局状态。
- 整理待用户确认的建议。
- 提醒权限、记忆、知识库和项目结构变化风险。
- 发现记忆冲突。
- 生成记忆整理建议。
- 帮用户把想法转为建议方案。
- 帮用户理解多个智能体工作状态。

秘书不能：

- 绕过用户确认直接改系统事实。
- 绕过项目主管直接操作项目。
- 绕过权限读取项目私密资料。
- 把聊天内容直接写入长期记忆。
- 替代审计中心。

后续可选扩展：

- 用户可以授予秘书低风险自动整理权限。
- 该权限必须有范围、有效期、可撤回、可审计。

## 8. 核心记忆治理

记忆层可以进入核心，但只能以治理层进入核心。

核心里包含：

- 记忆类型。
- 记忆作用域。
- 记忆生命周期。
- 记忆来源。
- 记忆权限。
- 记忆冲突规则。
- 记忆进入上下文的规则。
- 记忆变更审计规则。

核心外包含：

- 向量检索。
- 图谱索引。
- Obsidian 文件读取。
- 摘要模型。
- 相似度搜索。
- 存储引擎细节。
- 可视化知识图谱。

### 8.1 记忆类型

- 用户偏好记忆。
- 全局产品蓝图记忆。
- 项目记忆。
- 会话摘要。
- 成熟模式。

用户偏好记忆是高优先级核心记忆，因为它影响秘书、主管、建议方案、界面提醒和智能体上下文选择。

### 8.2 记忆生命周期

- 候选。
- 待确认。
- 已采纳。
- 冻结。
- 废弃。
- 历史。

普通聊天内容不自动进入长期记忆。

可以进入记忆候选的来源：

- 用户明确确认。
- 项目主管总结。
- 工作流总结。
- 阶段汇报。
- 用户采纳的建议方案。
- 秘书整理出的候选。

### 8.3 记忆和知识库边界

知识库是材料和思考空间。

记忆是会影响 agent 行为的确认事实。

索引是帮助搜索和理解的可重建辅助视图。

知识库内容不能因为被检索命中就变成记忆。

记忆候选必须经过来源、版本、权限、冲突和确认流程。

### 8.4 秘书和记忆

秘书可以：

- 读取用户偏好记忆。
- 读取全局状态。
- 读取项目摘要。
- 汇总记忆冲突。
- 提出记忆候选。
- 提出记忆整理建议。
- 提醒知识库结构变化对记忆的影响。

秘书不能：

- 直接写正式项目记忆。
- 直接覆盖旧记忆。
- 在没有确认的情况下把普通聊天写成长期记忆。

## 9. 工作流编排

工作流编排层负责项目内自动协作。

职责：

- 读取项目目标和项目黑板。
- 检查权限、策略、模型、工具和知识库引用。
- 派发给角色和会话。
- 等待结果。
- 接收汇报。
- 触发验证和回收。
- 推进状态机。
- 写事件和审计。
- 生成记忆候选。

边界：

- 工作流不是普通任务列表。
- 工作流不是任务包管理器。
- 任务包是内部协议，不是主界面中心。
- 工作流不是通用节点执行器。
- harness 是项目协议能力，不是默认普通节点。
- 工具调用结果不进入工作流账本全文。

## 10. 事件和审计

关键动作必须写事件。

事件包括：

- 用户确认。
- 权限请求。
- 权限批准或拒绝。
- 工作流状态变化。
- 节点派发。
- 智能体汇报。
- 主管回收。
- 工具调用摘要。
- 模型使用。
- harness 变化。
- 记忆候选创建。
- 记忆采纳、冻结、废弃。
- 知识库授权变化。

审计规则：

- 审计记录不能物理删除。
- 审计可以追加备注。
- 回滚不修改旧审计。
- 需要回滚时，从审计生成补救建议方案，再产生新审计。

## 11. 读模型

读模型服务界面。

读模型可以为以下界面提供数据：

- 首页。
- 项目列表。
- 项目画布。
- 会话中心。
- 通知中心。
- 待办中心。
- 审计中心。
- 记忆管理。
- 知识库。

边界：

- 读模型可以重建。
- 读模型不是事实源。
- UI 不直接拼底层复杂状态。

## 12. 当前代码收敛方向

当前 app 已有一些方向正确的点：

- Tauri + Rust + React + Vite 桌面壳。
- 工作流事实层 v0。
- Codex 会话读取和绑定。
- 项目内工作流展示。
- 局部 MCP 画布运行骨架。
- 右侧通知、待办、审计、运行中入口。

当前偏离或需要收敛的点：

- `src-tauri/src/lib.rs` 承担过多结构和命令职责，需要逐步拆出领域模型、应用服务、适配器和存储模块。
- 项目页仍暴露任务包中心感，需要降级为内部协议和节点详情。
- 项目工作流与全局可编辑画布入口需要统一权威关系。
- 记忆和知识库入口目前多为占位，不能暗示已完成。
- 秘书不能作为普通 UI 按钮或固定页面处理，需要作为核心协作角色建模。

## 13. 开发执行规则

后续开发必须先判断改动属于哪一层：

- 界面层。
- 应用服务层。
- 控制核心。
- 项目黑板。
- 事实层。
- 适配器层。
- 读模型。

接缝区先写：

- schema。
- 状态机。
- 权限规则。
- 事件规则。
- 审计规则。
- 端口接口。

再写实现。

禁止：

- 前端直接写事实状态。
- 适配器直接改核心事实。
- LLM 直接推进状态。
- 工具结果全文进入工作流账本。
- 普通聊天自动进入正式记忆。
- 用占位入口冒充真实能力。

## 14. 一句话

工作台不是聊天软件、任务包管理器或通用节点自动化工具。

工作台是以项目为主轴的本地智能体协作系统：

- 项目隔离协作。
- 秘书常驻协作。
- 控制核心管事实和权限。
- 项目黑板接住中间态。
- 工作流驱动自动协作。
- 适配器接入外部能力。
- 事件和审计保证可追踪。
- 记忆治理进入核心，记忆实现保持可替换。
