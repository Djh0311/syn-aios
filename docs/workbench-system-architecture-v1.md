# Workbench System Architecture v1

状态：**当前目标软件架构正本。** 已确认的产品运行模型来自 2026-08-01 两份修订；本文不代表当前代码已经实现这些模块。

本文只定义软件架构，不重新定义首页 UI 内容，不替代最终蓝图、`docs/harness/CURRENT.md`、`docs/harness/AUTHORITY.md`、`decisions/**` 或阶段计划。

最终蓝图来源：

- `/Users/yoyi/Documents/Codex/2026-05-26/gan-xing-codexbridge-https-github-com/docs/architecture/local-ai-workbench-blueprint-v1.md`
- `/Users/yoyi/Documents/Codex/2026-05-26/gan-xing-codexbridge-https-github-com/docs/architecture/local-ai-workbench-ui-blueprint-v1.md`

当前相关设计：

- `decisions/2026-08-01-whole-workbench-event-driven-operating-model-amendment-v1.md`
- `decisions/2026-08-01-memory-self-capture-daily-consolidation-and-skill-governance-amendment-v1.md`
- `docs/workflow-task-package-design-v1.md`
- `docs/memory-layer-design-v1.md`
- `decisions/2026-05-28-extensible-first-development-rule.md`
- `decisions/2026-05-28-codex-workflow-min-model.md`
- `decisions/2026-05-29-codex-session-workflow-route-correction.md`
- `decisions/2026-05-31-editable-canvas-codex-as-director-v1.md`

2026-08-01 当前交互校正：

- “Jarvis / 贾维斯”只表示秘书，新文档和界面只保留“秘书”一词。
- 顶层角色入口是秘书与全局主管。
- 项目主管不是第三个顶层入口；它常驻在每个项目内部。
- 前端整体暂不推倒，首页可重构；当前首要设计工作是让已有后端能力形成可解释、可审计的日常协作关系。
- 本文承担该协作关系的架构正本；当前源码能力、迁移和执行顺序分别见 inventory、current master plan 与 stage plan。

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

**本地模块化单体运行形态 + 个人 / 项目作用域 + 控制核心 + 项目黑板 + 能力适配器 + 事件账本 / 当前快照 / outbox + 核心记忆治理 + 秘书核心协作层。**

大白话：

- 工作台是一个本地桌面应用，不拆微服务。
- 项目是复杂结构化工作的最高业务对象和主要执行 / 权限隔离单元；个人范围承载不需要项目化的日常事务与数据。
- 控制核心掌握事实、权限、状态机、策略和审计。
- 智能体、工具、知识库、模型、harness、代码图谱都通过适配器接入。
- 多个智能体围绕项目协作时，先把中间结果放到项目黑板。
- 只有控制核心能把候选、汇报、工具摘要或建议升级为正式事实。
- 记忆层是核心能力，但进入核心的是记忆治理，不是具体检索或存储实现。
- 秘书是核心协作角色，持续看住所有与用户有关的未闭环事项；它不是项目执行者，也不被某个页面、侧栏或拟人形象绑死。
- 日常 / 开发是每个会话、交接、工作项和运行的显式通道，不是两个系统或一个全局开关。
- Agent / 模型只由用户、定时、外部、项目或系统事件激活；无事件时不空转。

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

- 首页按 2026-08-01 运行模型重构为秘书的情境简报与持续对话入口；具体视觉和布局仍由前端任务决定。
- 首页读取 `Attention / OpenLoop / Decision / ProjectSummary / DailyBrief` 等可重建读模型，每项携带来源和 typed object ref。
- 首页不能保存第二份项目、个人或外部软件事实，也不能用摘要反向改真源。

### 5.2 应用服务层

职责：

- 把用户操作变成命令。
- 调用控制核心校验权限、策略和状态机。
- 调用适配器执行外部动作。
- 写事件、审计和当前快照。
- 生成前端读模型。

示例命令：

- 创建项目。
- 把个人事项明确升级为项目。
- 绑定会话。
- 创建显式交接并接单。
- 维护 / 关闭内部关注。
- 回答待决定并回写原 owner。
- 启动工作流。
- 派发给智能体。
- 批准权限。
- 记录主管回收。
- 捕获观察、执行记忆政策、采纳或回退记忆版本。
- 生成可回源的每日简报 / 日报。
- 授权、同步、撤销或调用连接器能力。
- 废弃记忆。
- 生成审计补救建议。

### 5.3 控制核心

控制核心是工作台最硬的部分。

职责：

- 项目身份和项目隔离。
- 个人 / 全局 / 项目作用域、角色、当前对象和日常 / 开发通道。
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
- 内部关注、显式交接和待决定回源规则。
- 事件、事务、outbox、幂等和失败回执规则。
- connector capability 与 CredentialRef 规则。

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
- 当前快照：当前个人 / 项目、角色会话、关注、工作流、权限、记忆等状态。
- 审计账本：权限、状态、工具、模型、harness、记忆等关键事实的审计记录。

外部动作另有可靠 outbox：业务状态、事件、审计和待发送请求在本地事务内提交；adapter 结果再以新 command 回写。外部正文、raw transcript、完整 tool output 和 secret 不进入事件账本。

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
- ConnectorAdapter：邮件、日历、文件、消息、数据库和其他外部软件。

连接器能力必须拆开声明：

- `view`：按需读取；
- `index`：建立可重建索引；
- `sync`：受控同步；
- `action`：外部写入、发送或改变状态；
- `secret`：使用受保护凭据引用。

一项能力获准不自动包含下一项。真实凭据由受保护凭据层持有；适配器、事件、聊天、记忆和普通 store 只接收 `CredentialRef`。

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
- 全局主管 advisory 权限边界。
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
- 绕过项目主管、控制核心、适用政策或用户确认；也不能把全局主管意见冒充批准。

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

当前路线边界：

- 旧文档里的 E / G 是历史阶段标签，不再提供当前排期。
- current master 尚未给 GEPA 类优化器排实现阶段；只有 M2 事件 / 审计底座、M7 Skill 治理和 M10 运行证据成熟后，才可另建专题计划与任务包。
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
- 候选不能绕过项目主管、控制核心、适用政策或用户确认；全局主管意见本身不是批准。

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
- Runtime log、AdapterHealth、ServiceDegradedState、ReadbackFailureReason、ToolExecutionLog、DiagnosticBundleExport、IndexDegradedReport 是 current master M2 / M8 / M10 需要逐步冻结的运维对象。
- Skill 层必须和 Memory 层分开：Skill 可以成为候选能力或模板，但不能绕过正式记忆状态机、版本、权限、审计和用户确认。
- 向量库、索引和检索层只能是可重建派生能力；索引降级不能导致正式事实或正式记忆丢失，也不能把召回失败伪装成“没有相关材料”。

明确不吸收：

- 不把 Chat 作为最高级业务对象；项目仍是复杂结构化工作的最高业务对象，个人范围按本文 §6 并存。
- 不把大工具箱、Cookbook、Email、Calendar、MCP、Shell、Gallery、Tasks 全部做成一级入口。
- 不允许 agent 自由调用工具完成整件事；worker 只能读取任务包允许的最小上下文和工具。
- 不允许 MCP 直接 add / edit / delete FormalMemory。
- 不允许 vector memory 或 LLM memory extraction 直接影响 agent 行为。
- 不允许 admin 能力绕过项目、方案授权、任务包、控制核心和用户确认。
- 不让 app API 成为万能后门。
- 不把 Deep Research 报告自动变成项目事实。
- 不把 raw logs、vector id、adapter schema、provider secret 或路径大表默认显示给普通用户。

当前路线边界：

- 旧 E / E2 标签只表示历史现场；Odysseus v2 不属于 current Stage 1，也不授权实现。
- 后续如要吸收具体能力，必须按 current master 落到对应阶段，经 Harness 任务激活；需要时取得全局主管 advisory，最终授权仍来自用户与控制合同。

### 5.9 外部 agent runtime 参考约束

Paseo 这类 coding agent orchestration / daemon 工具可以作为最终蓝图的多 agent 运行层参考，但不能覆盖本工作台的项目主管制、方案授权制、控制核心、任务包和正式记忆治理。

参考资料：

- `docs/research/2026-06-05-paseo-workbench-deep-reference-research-v1.md`

可吸收为蓝图约束：

- Agent runtime control plane 需要有后端 authoritative read model，前端不能自己拼真实 agent 状态。
- Agent lifecycle、runtime session、timeline、permission request、attention、provider snapshot 和 logs / audit 分层应分别进入 current master M3 / M5 / M8 / M10 的合同与实现。
- Provider adapter 只能声明 capability、mode、model、feature、availability、diagnostic 和 persistence handle；技术可用不等于当前项目已授权可用。
- 会话中心未来如果从 transcript viewer 升级为 runtime session viewer，必须区分 live stream、authoritative fetch、sequence dedupe、pagination、tool call folding、permission、attention 和 error UI。
- 多 agent parent / child / detached 关系必须来自项目主管任务包和控制核心派发，不能来自 agent 自治创建。
- Worktree isolation、setup / teardown、script、service、diff review 和 service proxy 可作为 M5 / M10 的专题研究对象；它们不是完整 sandbox，也不能绕过路径、凭据和 destructive command guard。
- Schedule / loop / verifier 可以作为 long-running run object 和运维模型参考，但不能替代用户方案授权、项目主管过程确认或全局主管最终复核。
- Remote / mobile / relay 如果进入最终蓝图，必须优先建模 pairing、E2E encryption、host allowlist、password auth、DNS rebinding protection 和 daemon exposure diagnostics。
- Runtime timeline、logs、tool output 和 loop report 只能作为 Observation 来源，不能直接写正式事实或正式记忆。
- CLI parity 是后续测试和运维参考：app 能做的运行层动作应有受控 API / CLI 语义，但 CLI 也必须经过同一控制核心、权限和审计边界。

明确不吸收：

- 不把 agent / chat / timeline 作为最高级业务对象；项目仍是复杂结构化工作的最高业务对象，个人范围按本文 §6 并存。
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

当前路线边界：

- Paseo 只作为 M3 / M5 / M8 / M10 的专题参考，不属于 current Stage 1，也不授权实现。
- 后续吸收具体能力必须先形成合同并经 Harness 任务激活；全局主管可给 advisory，不能代替用户授权。

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

历史研究曾建议的顺序（H0–H7 / I0–I6 标签已失去当前排期效力）：

```text
H0-H7：先把 codex-local 真实自动化工作流产品化
-> I0-I6：再抽象 WorkerAdapter / RunUnit / DispatchRequest / PermissionEnvelope / WorkerHandoff / ReadbackResult / RuntimeLog / AuditEvent
-> planned adapters 后续独立产品化
```

当前路线边界：

- current master 先在 M3 / M5 收敛中立会话和执行合同，再在 M8 接 adapter / connector；不按旧 H / I 计划续跑。
- 抽象完成不能声称 Claude Code / OpenClaw / OpenCode 已接入；provider credential / model verification 仍需独立任务与授权。

## 6. 个人范围与项目单元

项目是复杂、持续、需要权限与执行治理的工作的最高结构化业务对象。个人范围与项目并存，承载尚不需要项目化的资料、承诺、日程、沟通、关注和轻量事项。

个人范围包含：

- 个人身份和明确事实。
- 系统推断的个人模型断言（与明确事实分开）。
- 个人数据源和连接器引用。
- 收件、关注、承诺、提醒和待决定。
- 秘书会话、每日简报和日报。

个人事项升级为项目必须来自用户明确决定，保留来源和历史；不得复制成个人 / 项目两份真源。

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

- 项目主管、worker、reviewer 和项目执行会话必须归属于一个明确项目。
- 秘书与全局主管的顶层会话可以属于个人 / 全局作用域；它们不能因此取得任一项目的原始上下文或写权限。
- 会话不能随意跨项目迁移。
- 子智能体只能读任务包或权限允许的项目范围。
- 项目知识库权限按任务临时开放。
- 项目记忆不能直接污染全局记忆。
- 项目审计摘要来自全局审计事实的项目视图。
- 项目事件留在项目；只有跨范围冲突、待用户处理、异常、承诺和必要摘要上浮。
- Secretary / Global Supervisor 通过 `ProjectSummary` 和引用按需读取，不拥有项目事实镜像。

### 6.1 角色入口拓扑

```text
工作台顶层
├── 秘书
│   └── 个人 / 全局整理、提醒、待办、日报、解释、交接建议
├── 全局主管
│   └── 跨项目优先级、边界、风险、方案和最终结果复核
└── 项目列表
    └── 某个项目
        └── 该项目常驻项目主管
            └── 项目会话、知识、记忆、黑板、任务、工作流和执行
```

这不是三个平级的全局聊天入口：

- 秘书和全局主管是顶层明确入口。
- 项目主管只在明确项目内出现。
- 用户进入哪个入口，就是在声明本轮角色和默认作用域；后端不得先让模型猜角色或猜项目。
- 需要跨角色时走显式交接，不静默切换会话、作用域或权限。

### 6.2 会话的六个确定字段

每个角色会话至少固定：

1. `role`：秘书、全局主管、项目主管、稳定成员或临时 Agent；前三者是主要角色入口，后两者通过成员目录 / 任务记录寻找，不新增顶层入口；
2. `scope`：个人、全局或明确项目；
3. `current_object`：当前事项、文档、记忆、方案、运行或审计记录；
4. `permission_profile`：只读、建议、候选写入、项目内受控动作或待用户确认。
5. `execution_channel`：`daily` 或 `development`；跨通道必须显式交接或创建新会话。
6. `role_session_id`：Syn 持有的稳定会话身份；provider `conversation_id / thread_id` 只是可替换外部 handle。

角色与项目范围来自入口和当前页面，不由模型推断。续接时必须复核这些字段，不能因复用 provider thread 发生漂移。模型只解释用户在该已知边界内想做什么。

## 7. 秘书核心协作层

秘书是核心协作角色，不是某个界面。

“Jarvis / 贾维斯”是秘书的历史称呼，不产生另一份角色档案、权限、会话类型或后端服务。

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
- 维护带来源、owner、理由和更新时间的内部关注 / 未闭环状态。
- 主动查询内部角色进度，或请求全局主管做留痕的只读分析。
- 生成开场情境、事件驱动更新和日终报告。

秘书不能：

- 绕过用户确认直接改系统事实。
- 绕过项目主管直接操作项目。
- 绕过权限读取项目私密资料。
- 绕过记忆政策把聊天内容直接写成正式记忆。
- 替代审计中心。

秘书维护内部关注和受治理记忆捕获的权限已经由 2026-08-01 修订确认；该权限只覆盖可撤销的内部协调状态，不包含项目事实、正式任务、外部动作或权限升级。

> **本节状态说明（2026-08-01）**：7.1—7.4 已由用户确认为目标运行合同。它们不是当前源码现状，也不单独授权实现；当前接线与断点以 inventory 为准，迁移顺序以 current master plan 为准。

### 7.1 角色协作矩阵

角色权限沿用既有决策；本矩阵只把入口、作用域、能力和交接关系接起来。

| 角色 | 所在层级与默认作用域 | 日常职责 | 可以形成的输出 | 不能静默做 | 主要交接对象 |
|---|---|---|---|---|---|
| 秘书 | 顶层；个人 / 全局，按权限读取项目摘要 | 收集、整理、搜索、提醒、维护未闭环、日报、长期变化汇总、解释状态、发起受治理记忆捕获和工作建议 | 个人收件项、OpenLoop、提醒、摘要、建议、待确认清单、交接建议、Observation / MemoryCandidate | 不能确认项目事实、越过项目主管派活、把内部关注冒充正式任务、绕过政策写正式记忆 | 用户、全局主管、某项目主管、知识 / 记忆工作面 |
| 全局主管 | 顶层；全局 / 项目组合，按复核需要读取项目证据 | 看跨项目优先级、边界、冲突、风险、方案和最终结果，给独立意见 | 自己的复核记录、风险意见、建议动作、待用户决定项 | 当前不能自动打回、批准、改工作流状态或替用户执行建议 | 用户、秘书、相关项目主管 |
| 项目主管 | 项目内部；一个明确项目 | 理解目标、协商方案、建立任务或工作流、决定单 Agent / 主管编排、派发、追问、回收、验证、形成项目内过程事实 | 方案、任务 / 工作流、项目黑板条目、项目内确认事实、交付、记忆候选 | 不能改变用户目标、验收或授权范围；不能跨项目读写；不能绕过控制核心 | 用户、秘书、全局主管、worker / reviewer、专业能力模块 |

入口开放不等于权限升级。秘书当前只获准维护可撤销的内部关注和受治理捕获；若扩大到正式任务、项目事实、外部动作或权限升级，或给全局主管自动闸权、扩大项目主管权限，必须另立决策。

### 7.2 工作台模块协同合同

下表描述希望形成的责任关系，不代表这些模块当前已经按这条线接通。专业模块可以直接进入，但它们是能力或工作对象，不是新的决策角色。

| 模块 | 日常职责 | 从哪里取数 | 结果回到哪里 |
|---|---|---|---|
| 项目 | 承载项目身份、目标、权限、资料、会话和执行 | 项目事实、当前快照、项目读模型 | 项目页、项目主管、全局摘要 |
| 会话 | 保存角色在明确作用域内的连续交流 | 角色档案、作用域、授权上下文、事实引用 | 当前角色入口；聊天本身不是事实层 |
| 知识库 | 保存材料、文档、引用和思考成果 | 文件、索引、搜索、引用权限 | 对话引用、项目资料、知识工作区 |
| 记忆 / 个人模型 | 保存会影响未来行为的受治理长期内容，并分开用户事实与系统推断 | 捕获、候选、来源、版本、权限、冲突、时效和政策结果 | 角色上下文、记忆中心；低风险内容可按政策自动沉淀，高影响内容待用户决定 |
| 项目黑板 | 汇聚项目协作中间态 | 目标、汇报、风险、证据、请求和候选 | 项目主管回收；经控制核心后才升级为正式事实 |
| 任务 / 工作流 | 组织持续、分步或多角色工作 | 已确认目标、方案、权限和项目黑板 | 状态、过程事件、交付、阻塞和记忆候选 |
| Agent / 工具 / Harness | 执行、搜索、验证和提供证据 | 受控命令、能力 allowlist、工作目录、任务上下文 | 结果、摘要、证据或错误；不能直接改正式状态 |
| 控制核心 / 权限 | 判定角色、范围、状态转移和动作是否合法 | 命令、角色、作用域、权限和当前快照 | 允许、拒绝、待确认和审计事件 |
| 事件 / 审计 | 记录真实发生的关键动作 | 确认、派发、工具、状态、复核和记忆变化 | 审计、复盘、补救建议和日报事实来源 |
| Attention / 读模型 / 通知 / 待办 / 日报 | 把复杂事实变成用户当前需要知道、决定和延续的视图 | 事实、事件、快照、OpenLoop、权限和 source refs | 首页、角色入口、项目页和专业模块；不得反向成为第二真源 |
| 连接器 / 凭据引用 | 按能力读取、索引、同步或操作外部软件 | ConnectorGrant、CredentialRef、sync cursor 和源数据 | 候选事件、引用或动作结果；不能直接写核心事实或暴露 secret |

模块之间优先通过受控命令、事件和引用协作，不继续依赖页面状态、复制聊天或互相直接写 store 串联。

### 7.3 显式交接合同

目标模型中，某个角色发现事情需要另一个角色时：

1. 先完成自己权限内的整理或分析；
2. 明确提出交给谁、属于哪个作用域、为什么交、希望完成什么；
3. 按既有权限和风险规则决定是否需要用户确认；
4. 接收角色明确接单；
5. 原会话、来源和事实引用全部保留。

最小交接包包含：

```text
handoff_id
from_role
to_role
scope_type / scope_id
current_object_id
requested_outcome
reason
fact_refs / knowledge_refs / conversation_refs
current_risks
permission_request
execution_channel
user_decision（若需要）
```

交接不是把一段聊天复制给另一个模型重新猜。完成后，结果必须回到原事项、对应事实 / 事件 / 审计、相关专业模块读模型和发起角色的回执中。

### 7.4 已确认的日常协作节奏（目标，不是现状）

下面是目标日常运行合同。当前盘点确认：持续秘书对话、顶层全局主管入口、个人收件 / OpenLoop、日报以及普通项目对现有治理能力的完整调用都还没有形成生产闭环，因此不能把目标写成已经可用。

#### 早上：秘书总览

打开工作台这个事件触发秘书读取首页读模型、项目摘要、外部承诺、待决定、阻塞、提醒、昨日未完和记忆冲突，生成今日简报。已与别人约定和时间敏感事项优先。它不重新运行每个项目，也不把摘要当正式事实。

#### 临时想法：秘书先接住

秘书可以先在个人范围维护有来源的内部关注，不强迫用户立刻选项目，也不自动创建想法箱、任务或工作流。只有用户明确表达“记下来 / 创建 / 安排 / 按这个做”等有约束力语义后，才创建对应正式对象。需要持续推进时，显式升级为项目或交给某项目主管。

#### 明确项目工作：直接进入项目主管

用户进入项目后直接与常驻项目主管对话。项目主管读取本项目事实、相关会话、知识权限、项目记忆、黑板、未完成工作和 Harness 规则，不经过秘书做意图拆分。

需要方案、任务、执行或批准时，先生成相应草案或待确认结构化实物；聊天内容本身不自动成为批准或正式事实。

#### 跨项目判断：进入全局主管

全局主管读取项目摘要、关键方案、证据、风险和最终结果，给出跨项目优先级、边界和风险意见。改变项目目标或状态仍由用户与相关项目主管处理。

#### 专业工作：直接进入知识、记忆、Agent、工作流、连接器或审计

用户可以先看专业对象本身；需要解释或行动时，再选择秘书、全局主管或当前项目主管，并把当前对象引用带入角色会话。专业界面不被聊天吞掉。

#### 后台执行：项目主管组织

控制核心检查项目、方案、权限和风险；编排轴决定单 Agent 还是多角色工作流；Agent / 工具执行，Harness 验证，中间结果进入项目黑板，项目主管负责回收。

工作流界面保留过程细节；用户日常主要在项目主管对话中看到进展、阻塞、待决定和交付摘要。

#### 完成与沉淀：全局复核、记忆治理、秘书收尾

项目主管形成交付；全局主管按既有规则复核；事实层记录事件和审计；记忆系统按政策生成 Observation / Candidate / FormalMemoryVersion 或 DecisionRequest；秘书把完成、变化、风险、待确认和明日延续整理进日报。

正式事实、复核意见、记忆、个人模型断言、SkillCandidate、日报和秘书摘要必须分开保存，互不冒充。

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

- 捕获事件。
- 观察。
- 候选。
- 待确认。
- 已采纳。
- 冻结。
- 废弃。
- 历史。

普通聊天不逐条自动进入长期记忆，但秘书 / 记忆机制可以自发形成 CaptureEvent、Observation 和 Candidate。进入正式记忆必须经过来源、scope、敏感性、冲突、时效、外发和策略治理；低风险内容可按已冻结政策自动沉淀，高影响或冲突内容进入待用户决定。

可以进入记忆候选的来源：

- 用户明确确认。
- 项目主管总结。
- 工作流总结。
- 阶段汇报。
- 用户采纳的建议方案。
- 秘书整理出的候选。
- 用户纠正、每日整理和带来源的外部数据摘要。

### 8.3 记忆和知识库边界

知识库是材料和思考空间。

记忆是会影响 agent 行为的、经过治理的长期内容。

索引是帮助搜索和理解的可重建辅助视图。

知识库内容不能因为被检索命中就变成记忆。

记忆候选必须经过来源、版本、权限、冲突和政策流程。用户明确事实与系统推断分开；需要用户确认的类别不得靠自动策略绕过。

### 8.4 秘书和记忆

秘书可以：

- 读取用户偏好记忆。
- 读取全局状态。
- 读取项目摘要。
- 汇总记忆冲突。
- 提出记忆候选。
- 提出记忆整理建议。
- 提醒知识库结构变化对记忆的影响。
- 自发提交 CaptureEvent、Observation 和 MemoryCandidate。
- 触发每日整理，并解释自动政策结果。

秘书不能：

- 绕过记忆 policy / repository 直接写正式项目记忆。
- 直接覆盖旧记忆。
- 把密钥、权限、外部动作授权或高影响推断自动写成长期记忆。

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

事件机制是统一基础设施，不是全局业务事件中心：

- 项目事件由项目拥有，个人事件由个人范围拥有，连接器事件由数据源拥有。
- 上浮给秘书 / 全局主管的是跨范围问题、待用户处理、异常、承诺和必要摘要引用。
- 用户消息、App 打开、定时器、外部数据变化、项目状态变化、完成、失败和用户纠正都可以触发事件。
- 没有事件时不启动 Agent / 模型空转。
- 所有 handler 必须有幂等键、预算、重试上限、失败回执和审计。

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
- OpenLoop 创建、关闭、驳回或重新打开。
- 角色交接、接单和回执。
- 每日简报 / 日报生成。
- connector grant、sync 和 action result。
- 自动记忆 / Skill 政策结果。

审计规则：

- 审计记录不能物理删除。
- 审计可以追加备注。
- 回滚不修改旧审计。
- 需要回滚时，从审计生成补救建议方案，再产生新审计。
- 事件和审计只存结构化摘要、引用和 hash；raw transcript、完整 prompt、完整 tool output 和 secret 禁止进入 payload。

## 11. 读模型

读模型服务界面。

读模型可以为以下界面提供数据：

- 秘书首页 / 当前情境。
- 项目列表。
- 项目画布。
- 会话中心。
- 通知中心。
- 待办中心。
- 审计中心。
- 记忆管理。
- 知识库。
- 全局主管。
- 成员目录。
- OpenLoop / 待决定。
- 每日简报 / 日报。
- connector 状态。

边界：

- 读模型可以重建。
- 读模型不是事实源。
- UI 不直接拼底层复杂状态。
- 每个条目携带 typed object ref、source refs、owner 和 scope，能够精确回到原对象。
- ProjectSummary、DailyReport、首页和通知都是可重建投影，不得反向覆盖业务真源。

## 12. 当前代码收敛方向

当前 app 已有一些应保留或提取的点：

- Tauri + Rust + React + Vite 桌面壳。
- 工作流事实层 v0。
- Codex 会话读取和绑定。
- 项目内工作流展示。
- 局部 MCP 画布运行骨架。
- 右侧通知、待办、审计、运行中入口。
- Native Knowledge Workspace 和记忆治理对象。
- proposal、authorization、worker report、review、runner 和 readback 的局部厚能力。

当前偏离或需要收敛的点：

- `src-tauri/src/lib.rs` 承担过多结构和命令职责，需要逐步拆出领域模型、应用服务、适配器和存储模块。
- 普通会话、supervisor、resident、manual relay 和 workflow 路径并存，需要收敛为 role-scoped application service + transport adapter。
- 项目页仍有任务包 / 工作流中心遗产，需要让项目主管对话成为日常入口，同时保留专业工作面。
- 项目工作流与全局可编辑画布入口需要统一权威关系。
- 首页、通知、待办、运行中和审计主要靠前端 / 多 store 拼装，需要由 typed read model 替代。
- 记忆仍是多 sidecar 和逐条确认链；目标自动治理、每日整理和 SkillCandidate 尚未实现。
- 秘书当前只是只读摘要 / 一次咨询；目标持续会话、OpenLoop、日报和个人范围尚未实现。
- 全局主管没有顶层入口，项目主管没有把普通对话接进既有治理闭环。
- Connector、CredentialRef 和个人事实模型尚未形成生产合同。

具体 `KEEP / EXTRACT / REWRITE / RETIRE / HOLD` 和阶段顺序以 current master plan 为准；本节不复制逐任务排期。

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
- 幂等、事务、outbox、迁移和回滚。

再写实现。

禁止：

- 前端直接写事实状态。
- 适配器直接改核心事实。
- LLM 直接推进状态。
- 工具结果全文进入工作流账本。
- 普通聊天未经捕获判定和治理就直接进入正式记忆。
- 没有事件时让 Agent / 模型后台空转。
- 用前端隐藏代替 scope / permission 后端拒绝。
- 把首页、摘要、日报或 ProjectSummary 当第二事实源。
- 把 connector read grant 当 write grant，或把 CredentialRef 展开进业务数据。
- 用占位入口冒充真实能力。

## 14. 一句话

工作台不是聊天软件、任务包管理器或通用节点自动化工具。

工作台是以项目为复杂工作主轴、同时承载个人范围的本地个人 AI 协作系统：

- 项目隔离协作。
- 秘书常驻协作。
- 控制核心管事实和权限。
- 项目黑板接住中间态。
- 工作流驱动自动协作。
- 适配器接入外部能力。
- 事件和审计保证可追踪。
- 记忆治理进入核心，记忆实现保持可替换。
- 秘书持续守住未闭环，首页和日报只做可回源聚合。
- 日常 / 开发按工作显式分通道。
- 外部软件按能力接入，凭据只以保护引用参与。
