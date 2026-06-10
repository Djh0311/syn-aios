# Evidence: reference workflow and session-node research v1

## 薄弱点先说

- 这不是九个参考项目的逐行源码审计。Dify、n8n、Langflow、OpenHands 都是大型平台，逐行读完整仓库不是本轮能完成的工作。
- AionUi 的桌面端前端、共享类型、数据库 schema、迁移、测试用例已经读到；但 `/api/teams` 的服务端实现和 `TeamMcpServer` 主体没有在已解出的 `packages/desktop/src/process` 中找到。结论不能说“已经读透后端调度实现”。
- AionUi tarball 解包时在 `resources/AionUi_team.gif` 报 `truncated gzip input`。源码主体已解出，但这份 tarball 不是完整无损证据。
- 参考源 README 有宣传成分，只能作为项目定位证据，不能单独当架构证据。AionUi 的结论主要以源码和测试为准。

## 用户问题

用户问：

- “我们不能直接把会话当作工作流节点来编排吗”
- “先把工作流能做出来再说”
- 要先深度研究 `iOfficeAI/AionUi`，并尽量研究参考源里的项目。

本轮研究目标：

- 判断“会话作为工作流节点”的路线是否可行。
- 判断参考源对当前阶段下一步的影响。

## AionUi 证据

### 项目定位

依据：`iOfficeAI/AionUi` README 和 `docs/readme/readme_ch.md`。

AionUi 的 Team Mode 描述是：

- Leader 接收用户指令。
- Leader 分解子任务。
- 通过内置 Team MCP Server 委派给 Teammate。
- Teammate 并行执行。
- 通过异步 mailbox 共享结果。
- 写入共享 task board。
- 每个 agent 有自己的权限确认。

判断：

- AionUi 不是简单聊天壳。
- AionUi 也不是纯节点画布优先。
- 它更像“团队/会话工作区”：会话是可见主体，团队、任务、邮箱、权限、状态是底层结构。

### 类型模型

依据：`packages/desktop/src/common/types/team/teamTypes.ts`。

关键类型：

- `TeamAgent`
  - `slot_id`
  - `conversation_id`
  - `role`
  - `agent_type`
  - `agent_name`
  - `conversation_type`
  - `status`
  - `model`
  - `pending_confirmations`
- `TTeam`
  - `id`
  - `name`
  - `workspace`
  - `workspace_mode`
  - `leader_agent_id`
  - `agents`
  - `session_mode`

判断：

- AionUi 把“团队成员”和“会话”绑定在一起，但没有把原始会话当成唯一事实。
- `conversation_id` 是 agent slot 的执行入口之一，不是完整工作流状态本身。
- `pending_confirmations` 和 `session_mode` 说明权限和运行模式是团队级/成员级状态，不是聊天正文能表达清楚的。

### 后端桥接

依据：`packages/desktop/src/common/adapter/ipcBridge.ts` 的 Team Mode API。

读到的接口：

- `team.create`
- `team.list`
- `team.get`
- `team.remove`
- `team.addAgent`
- `team.removeAgent`
- `team.stop`
- `team.ensureSession`
- `team.renameAgent`
- `team.renameTeam`
- `team.setSessionMode`
- `team.agentStatusChanged`
- `team.agentSpawned`
- `team.agentRemoved`
- `team.agentRenamed`
- `team.listChanged`
- `team.created`
- `team.teammateMessage`

判断：

- AionUi 的 Team UI 不是直接操作聊天窗口数组，而是通过显式 team API 操作团队。
- `ensureSession` 说明团队会话有运行时准备动作。
- `agentStatusChanged` / `agentSpawned` / `agentRemoved` 说明 UI 靠状态事件驱动，而不是靠轮询聊天文本猜测。

### 数据库模型

依据：`packages/desktop/src/process/services/database/schema.ts` 和 `migrations.ts`。

读到的表：

- `teams`
  - `id`
  - `user_id`
  - `name`
  - `workspace`
  - `workspace_mode`
  - `lead_agent_id`
  - `agents`
  - `created_at`
  - `updated_at`
- `mailbox`
  - `team_id`
  - `to_agent_id`
  - `from_agent_id`
  - `type`
  - `content`
  - `summary`
  - `read`
  - `files`
- `team_tasks`
  - `team_id`
  - `subject`
  - `description`
  - `status`
  - `owner`
  - `blocked_by`
  - `blocks`
  - `metadata`

判断：

- AionUi 的“团队协作”有持久化状态，不只是多条会话。
- `mailbox` 是 agent 间通信事实。
- `team_tasks` 是任务板事实。
- 这支持“会话作为可见节点”，但反对“只靠会话编排，不建工作流账本”。

### 前端 Team Page

依据：`packages/desktop/src/renderer/pages/team/TeamPage.tsx`、`TeamTabs.tsx`、`TeamChatView.tsx`。

读到的 UI 形态：

- TeamPage 同时渲染多个 agent slot。
- 每个 slot 绑定一个 conversation。
- Leader 有视觉强调。
- 多成员横向排列，可全屏单个 agent。
- 顶部有 TeamTabs。
- tab 上显示 agent 状态和 pending permission 提醒。
- 右侧/侧边保留 workspace 文件入口。
- `TeamChatView` 根据 conversation type 分发到 ACP / Aionrs / OpenClaw / Nanobot / Remote 等聊天实现。

判断：

- AionUi 的核心体验不是流程图节点，而是“多个会话并排工作”。
- 对我们有价值的是这个信息架构：把 Codex 会话变成用户看得见、能点开、能观察状态的工作单元。

### 权限模式

依据：`TeamPermissionContext.tsx`、`useTeamPendingPermissions.ts`、`ApprovalStore.ts`、AionUi E2E。

读到的机制：

- team page 统计每个 conversation 的 pending confirmations。
- sidebar / tabs 可显示待确认数量。
- `setSessionMode` 会把模式写到 team record。
- session mode 会影响新 spawn agent。
- E2E 中 MCP tool confirmation 需要点击 “Yes, allow always” 才能继续。

判断：

- AionUi 把权限确认做成工作流运行的一等状态。
- 我们当前缺的“权限确认队列”不是锦上添花，而是工作流能真实跑起来的必要部分。
- 但 AionUi 的 YOLO / full-auto 不适合当前阶段吸收。当前边界仍应保留用户明确确认。

### 测试证据

依据：GitHub API 列出的 `tests/e2e/cases/teams`，以及读到的测试文件。

读到的测试覆盖：

- `team-create.e2e.ts`
- `team-whitelist.e2e.ts`
- `team-communication.e2e.ts`
- `team-member-messaging.e2e.ts`
- `team-agent-lifecycle.e2e.ts`
- `team-session-mode.e2e.ts`
- 还有 delete、rename、view mode、workspace migration 等 team 测试。

关键行为：

- create team 需要选择 leader。
- leader 可通过自然语言添加 member。
- member tab 会出现。
- 可直接给 member 发消息。
- 可通过 leader 移除 member。
- session mode 会写入 team record。
- whitelist 限制 team-capable backend。

判断：

- AionUi 的 Team Mode 是经过端到端测试的产品面，不只是 README 设想。
- 它的可借鉴点优先是“团队会话工作区 + 状态/权限/任务底层账本”，不是“节点画布”。

## 其他参考源横向判断

### Codex++

依据：`BigPizzaV3/CodexPlusPlus` README 和 GitHub contents。

定位：

- Codex App 外部增强启动器和管理工具。
- Tauri + React + Rust。
- 通过外部 launcher 启动 Codex。
- 通过 Chromium DevTools Protocol 注入增强脚本。
- 提供会话删除、Markdown 导出、项目移动、Timeline、provider 同步等。

对我们可吸收：

- 会话列表、会话正文、时间线、导出、项目归属的体验。
- 外部增强工具边界意识。

不建议当前吸收：

- CDP 注入作为主控制路径。
- 会话删除、provider 写入、中转注入。

判断：

- Codex++ 证明 Codex 会话管理有产品价值。
- 但它不是工作流编排参考主轴。

### Multica

依据：`multica-ai/multica` README 和 GitHub contents。

定位：

- open-source managed agents platform。
- 把 coding agents 当 teammates。
- issue / board / status / blocker。
- local daemon / cloud runtime。
- agents 自动领取、执行、汇报进度。
- squads 由 leader agent 路由任务。

对我们可吸收：

- “agent 是队友，不是按钮”的表达。
- issue / board / status / blocker 的生命周期。
- leader / squad 的路由思想。
- runtime 能力检测。

不建议当前吸收：

- 云平台。
- 常驻 daemon。
- 多 agent 统一运行时。
- 自动领取任务。

判断：

- Multica 支持“会话/agent 作为工作单元”的产品表达。
- 但它比当前 Codex-only 范围大得多。

### Langflow

依据：`langflow-ai/langflow` README 和 GitHub contents。

定位：

- 构建和部署 AI agents / workflows 的平台。
- 可视化 builder。
- 逐步测试 playground。
- workflow 可作为 API 或 MCP server。
- 支持多 agent orchestration 和 observability。

对我们可吸收：

- 节点逐步测试。
- workflow 导出为工具/模板的远期思路。
- 运行可观测性。

不建议当前吸收：

- 通用 LLM 应用平台。
- 多模型、多向量库、多 RAG。

判断：

- Langflow 是“画布/组件工作流”参考，不是“会话工作区”参考。
- 对当前阶段，它提示我们需要可测试节点和运行轨迹，但不要求马上做通用画布。

### Dify

依据：`langgenius/dify` README 和 GitHub contents。

定位：

- LLM app development platform。
- AI workflow、RAG、agent capabilities、model management、observability。
- visual canvas 构建和测试 workflow。
- prototype 到 production。

对我们可吸收：

- workflow 发布前检查。
- 运行日志和观测。
- 应用/工作流版本化。

不建议当前吸收：

- LLM 应用平台。
- RAG 知识库。
- 模型供应商管理。

判断：

- Dify 的价值在生产化工作流治理，不在 Codex 会话调度。
- 当前只应吸收“运行记录、检查、版本”这些观念。

### n8n

依据：`n8n-io/n8n` README 和 GitHub contents。

定位：

- workflow automation platform。
- 400+ integrations。
- visual interface + code。
- AI agent workflows。
- 自托管、模板、企业权限。

对我们可吸收：

- 节点配置面板。
- 模板组织。
- 运行历史。
- 节点和连接的清晰语义。

不建议当前吸收：

- 通用自动化平台。
- webhook / cron / 400+ 外部集成。

判断：

- n8n 是“自动化节点系统”参考。
- 当前若过早照 n8n 做，会偏离 Codex 会话主线。

### Open Multi-Agent Canvas

依据：`CopilotKit/open-multi-agent-canvas` README。

定位：

- 多 agent chat interface。
- 管理多个 agent 在动态对话里协作。
- Next.js + LangGraph + CopilotKit。
- 可配置 MCP servers。
- 依赖 Copilot Cloud。

对我们可吸收：

- agent 对话和任务上下文同屏。
- MCP 配置入口形态。

不建议当前吸收：

- Copilot Cloud 依赖。
- LangGraph 多 agent 架构。

判断：

- 它支持“多会话/多 agent 同屏”的 UI 方向。
- 但不是当前 Codex-only 的执行层参考。

### OpenHands

依据：`OpenHands/OpenHands` README 和 GitHub contents。

定位：

- AI-driven development。
- SDK、CLI、Local GUI、Cloud、Enterprise 多入口。
- Local GUI 有 REST API 和 React SPA。
- 用于本机运行开发 agent。

对我们可吸收：

- 本地开发 agent 执行视图。
- REST API + GUI 分层。
- 任务执行过程表达。

不建议当前吸收：

- 替换 Codex runtime。
- 接 OpenHands runtime。

判断：

- OpenHands 是“开发 agent 执行产品”的参考。
- 当前最多借鉴执行日志、文件变更、运行过程可视化。

### Google ADK Web

依据：`google/adk-web` README。

定位：

- ADK 内置开发调试 UI。
- 用于 agent development 和 debug。
- 重点页面包括 events、tracing、artifacts、evaluations、agent builder。

对我们可吸收：

- 事件视图。
- trace 视图。
- artifacts 视图。
- evaluation / verification 视图。

不建议当前吸收：

- ADK 技术栈。
- 通用 agent framework。

判断：

- ADK Web 对“总指导回收判断”和“执行可观测性”有价值。
- 不是工作流主 UI 的第一参考。

## 对“会话能不能作为工作流节点”的判断

可以，但要限定：

- UI 上可以把 Codex 会话当成工作流节点/队友/工作单元。
- 数据上不能只保存会话。还需要保存 work item、dispatch、permission、artifact、review、audit、status。

依据：

- AionUi 的 `TeamAgent` 明确绑定 `conversation_id`，说明会话可作为 agent slot 的执行入口。
- AionUi 的 `teams` / `mailbox` / `team_tasks` 说明只靠会话不够。
- AionUi 的 `pending_confirmations` / `session_mode` 说明权限和运行模式必须外置成状态。
- Multica 的 issue / board / status / blocker 说明 agent 工作需要任务生命周期。
- Langflow / Dify / n8n 说明 workflow 需要运行记录、配置和可测试性。

更准确的说法：

- “会话作为可见节点。”
- “工作流状态作为底层账本。”

不建议的说法：

- “直接把会话当唯一工作流模型。”

风险：

- 会话正文是叙事，不是可靠状态机。
- 仅靠 transcript 很难判断当前任务是否 blocked、是否待权限、是否超时、是否可重试。
- 多轮 resume 长任务会产生中间状态，不能靠最后一句回复恢复完整过程。
- 权限确认不能藏在聊天文本里，否则总指导无法稳定回收。

## 对当前产品路线的影响

当前 `CURRENT.md` 写的下一步是“工作流节点 safe probe 真实确认派发 v1”。

研究后的判断：

- 这个任务技术上仍有价值，因为它验证 `codex exec resume` 可从工作台派发到真实绑定会话。
- 但从产品信心角度，它不是最该先做的下一步。用户现在卡的是“工作流形态是否对”，不是 safe probe 这一步是否能跑。

建议下一步改成：

- 先做“项目团队工作区 v1 / 会话即可见节点 v1”的信息架构和最小 UI。

最小形态：

- 左侧：项目内 Codex 会话/角色列表
  - 总指导
  - 开发线
  - 验证线
  - 回收线
- 中间：当前会话正文、派发记录、执行结果
- 右侧或底部：工作项账本
  - 状态
  - 最近 dispatch
  - evidence / handoff / review
  - 权限队列
  - 失败/超时/重试入口

底层状态仍保留：

- workflow state
- node/session binding
- work item
- dispatch
- audit event
- transcript stats

不建议现在做：

- 复杂画布编辑器。
- 多 agent 平台。
- 自动批准。
- 真实业务自动编排。
- 把任务包重新放回主界面中心。

## 当前结论

- AionUi 支持“会话作为可见工作节点”的方向。
- AionUi 也证明“只用会话做工作流模型”不够。
- 参考源整体支持一个折中路线：先把 Codex 会话做成项目团队工作区，隐藏的工作流账本继续存在。
- 真实 safe probe 仍要做，但建议排在“项目团队工作区 v1”之后，或者作为这个工作区里的第一个受控动作。

## 未做

- 没有执行真实 Codex 派发。
- 没有写 `/Users/yoyi/.codex`。
- 没有读取授权、密钥、`.env`。
- 没有改产品代码。
- 没有改当前权威路线文档。
- 没有运行 harness。
