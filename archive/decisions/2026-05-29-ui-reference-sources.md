# 决策：工作台 UI 参考源保留

## 结论

以下项目作为后续 UI、协作形态和 agent 工作流参考源保留：

- `BigPizzaV3/CodexPlusPlus`
- `iOfficeAI/AionUi`
- `multica-ai/multica`
- `langflow-ai/langflow`
- `langgenius/dify`
- `n8n-io/n8n`
- `CopilotKit/open-multi-agent-canvas`
- `OpenHands/OpenHands`
- `google/adk-web`
- `xyflow/xyflow` / React Flow
- `storybookjs/storybook`
- `shadcn-ui/ui`
- `vercel/v0`

当前不把这些项目作为产品路线，也不把当前阶段改成多 agent 工作台、通用自动化平台或 agent 云平台。

当前优先级不变：先把 Codex 会话读取、会话管理和 Codex 工作流做完，让 Codex 工作台可以支持快速自迭代开发。

## 依据

- 用户明确说 AionUi “先作为参考源保留”。
- 用户明确要求把 `multica-ai/multica` 也记录为参考源，并把“其他可借鉴项目”一并记录。
- 用户明确提供 `BigPizzaV3/CodexPlusPlus`，并说明 Codex 会话管理可以参考 Codex++。
- 用户明确说“当务之急还是把 codex 尽快做完”。
- 用户明确纠偏：当前要先把 Codex 内容正确读出来，并能编排工作流，不要继续偏到任务包管理器。
- 用户的目标是做好 Codex 工作流后，用工作台进行快速自迭代开发。
- 当前阶段计划已经限定第一版以治理 Codex 为主。
- Codex++ 自身 README 写明它是面向 Codex App 的外部增强启动器和管理工具，不修改 Codex App 原始安装文件，通过外部 launcher 启动 Codex，并使用 Chromium DevTools Protocol 注入增强脚本。
- Codex++ README 的主要功能包含会话删除、Markdown 导出、项目移动、Timeline、Provider 同步等 Codex 增强能力。
- Multica 自身定位是 open-source managed agents platform，强调把 coding agents 当作 teammates，分配任务、跟踪进度、复用 skills。
- Langflow 自身定位是构建和部署 AI agents / workflows 的平台，并提供可视化编排和 API / MCP 输出。
- Dify 自身定位是 agentic workflow development 平台。
- n8n 自身定位是带 AI 能力的 workflow automation 平台，强调可视化构建、代码扩展、自托管和集成。
- Open Multi-Agent Canvas 自身定位是多 agent 动态对话界面，并支持 MCP servers。
- OpenHands 提供本地 GUI、CLI、SDK 等 AI-driven development 入口。
- Google ADK Web 是 ADK 内置开发 UI，用于 agent 开发和调试。
- React Flow / xyflow 是 React 节点式编辑器和交互图的画布底座参考。
- Storybook 用于组件状态样例、视觉状态文档和 UI 回归检查。
- shadcn/ui 的 open code 组件方式适合作为 AI 可读、可改、可组合的 React 组件参考。
- Vercel v0 可作为局部 React UI 面板草稿生成参考，但不作为工作流状态机或画布逻辑来源。

## 参考源清单

### Codex++

参考价值：

- Codex 会话管理。
- Markdown 导出。
- Timeline。
- 项目移动。
- 外部 launcher / 管理工具思路。
- 不修改 Codex 原始安装文件的边界意识。
- Tauri + React + Rust 的本地管理工具形态。

当前转译到 Codex-only：

- 用来指导“会话列表、会话正文、会话时间线、导出、项目归属”的信息架构。
- 用来提醒我们先探针 Codex 会话控制入口，再做工作台内对话。
- 用来对照“外部增强工具”和“直接写 Codex 内部状态”的边界。

当前不吸收：

- 会话删除。
- 写 Codex provider 或供应商切换。
- 用户脚本注入。
- CDP 注入作为唯一控制路线。
- 中转站、模型供应商和 API key 管理。

### AionUi

参考价值：

- 多会话协作入口。
- Team Mode / Leader / Teammate 形态。
- 共享任务板。
- 权限确认队列。
- 工作区文件入口常驻。
- Skills 分层管理。

当前不吸收：

- 多 agent 全量接入。
- 远程访问。
- 自动批准或全自动执行。
- 手机端或聊天软件接入。

### Multica

参考价值：

- 把 coding agents 表达成可分配任务的 teammate。
- Issue / board / status / blocker 这一套任务生命周期。
- Squads / leader agent 的任务路由思想。
- Local daemon / runtime 这种 agent 执行环境抽象。
- Reusable skills 的团队复用视角。
- 多 workspace 隔离。

当前转译到 Codex-only：

- teammate 先转成 Codex 工作线，不做真实多 agent。
- issue / board 先转成任务包、状态、handoff、evidence、review。
- squad / leader 先转成总指导线和桌面应用里的工作流节点。
- runtime 先转成当前本机 Codex 可用状态，不做云 runtime。

当前不吸收：

- 多 agent 统一运行时。
- 云端平台。
- 后台 daemon 常驻执行。
- 自动领取任务和自动执行。
- Go / PostgreSQL / pgvector 技术栈迁移。

### Langflow

参考价值：

- 可视化 builder。
- 节点逐步测试。
- workflow 转 API 或 MCP tool 的出口。
- 组件可自定义。
- observability 入口。

当前转译到 Codex-only：

- 项目工作流画布先表达任务包、会话、handoff、evidence、review。
- 后续再考虑把稳定工作流导出成工具或模板。

当前不吸收：

- 通用 LLM 应用构建平台。
- 向量库和 RAG 主线。
- 多模型编排。

### Dify

参考价值：

- 生产化 workflow 管理。
- 发布前检查。
- 运行记录和观测。
- 应用 / workflow 的版本化思路。

当前转译到 Codex-only：

- 用在任务包生成、派发、回收、验证记录上。

当前不吸收：

- 直接做 LLM 应用平台。
- 知识库和 RAG。
- 模型供应商管理。

### n8n

参考价值：

- 节点式自动化画布。
- 节点配置面板。
- 模板库。
- 自托管和集成生态的组织方式。

当前转译到 Codex-only：

- 只借鉴节点配置和任务模板组织。
- 不做通用自动化平台。

当前不吸收：

- 400+ 集成生态。
- 通用 webhook / cron 自动化。
- 自动执行外部系统动作。

### Open Multi-Agent Canvas

参考价值：

- 多 agent 在一个动态对话 / 画布中的协作表达。
- MCP server 配置入口。
- agent 对话与任务上下文同屏。

当前转译到 Codex-only：

- 先用于理解未来多 Codex 会话协作 UI。
- 当前不接多 agent。

当前不吸收：

- 依赖 Copilot Cloud。
- 通用 MCP agent 运行。
- 多 agent 动态对话作为当前主功能。

### OpenHands

参考价值：

- 本地 GUI + REST API + 单页应用的开发 agent 体验。
- CLI / SDK / Local GUI 多入口。
- 权限、协作、开发任务执行视图。

当前转译到 Codex-only：

- 借鉴本地开发 agent 的任务执行与进度表达。

当前不吸收：

- 替换 Codex。
- 直接接入 OpenHands runtime。
- 企业协作和云部署。

### Google ADK Web

参考价值：

- agent 开发和调试 UI。
- 事件、轨迹、产物、评估页组织。

当前转译到 Codex-only：

- 后续用于验证线、harness 运行记录、任务执行轨迹设计。

当前不吸收：

- ADK 技术栈。
- 通用 agent 开发框架。

### React Flow / xyflow

参考价值：

- React 画布底座。
- 节点、边、背景、小地图、控制器、节点工具栏、节点尺寸调整等基础能力。
- 适合表达项目级工作流节点、责任流转、运行状态和节点详情入口。
- 适合让 AI 在明确 schema 下开发画布，而不是从零手写拖拽、连线和缩放。

当前转译到 Codex-only：

- 工作流画布底层优先参考 React Flow 的节点 / 边模型。
- 节点数据绑定当前 workflow state、work item、role binding、dispatch、review。
- 节点详情仍由工作台自己的右侧面板承载，不把任务包暴露成主 UI。

当前不吸收：

- 不把 React Flow 示例里的通用流程图当成产品流程。
- 不为画布提前做复杂低代码编辑器。
- 不把画布做成通用自动化平台。

### Storybook

参考价值：

- 给 React 组件建立状态样例。
- 为 AI 提供可读的组件使用说明和状态边界。
- 适合做画布节点、节点详情、建议方案卡片、权限队列、运行性检查条的 UI 回归基准。

当前转译到 Codex-only：

- 后续画布组件应有 stories：空画布、四角色节点、执行中、等待权限、失败、回收中、accepted、右侧详情打开。
- 用 stories 辅助 AI 迭代，减少只改 `App.tsx` 导致的回归。

当前不吸收：

- 不把 Storybook 当成产品运行时。
- 不在第一轮引入复杂视觉测试流水线。

### shadcn/ui

参考价值：

- open code 组件方式。
- 组件代码进入项目后，AI 更容易读取、修改和组合。
- 适合按钮、标签、抽屉、表单、滚动区、命令菜单、确认弹层等基础 UI 的一致性参考。

当前转译到 Codex-only：

- 借鉴组件分层和可组合写法，服务工作台右侧节点详情、建议方案面板、权限确认和审计列表。
- 不强制迁移到 Tailwind；当前工作台仍以现有 React + CSS 为主。

当前不吸收：

- 不为了使用 shadcn/ui 重写整个设计系统。
- 不把组件库安装作为当前画布第一步。

### Vercel v0

参考价值：

- 根据文字 prompt 生成 React UI 草稿。
- 适合快速生成局部面板、建议方案卡片、状态栏和空状态。

当前转译到 Codex-only：

- 只作为局部 UI 草稿参考。
- 生成结果必须回到本地代码规范、状态 schema 和工作流边界里复核。

当前不吸收：

- 不让 v0 主导工作流画布状态机。
- 不把 v0 输出直接视为最终工程代码。

## 可借鉴点

这些参考源可作为这些方向的参考：

- Codex 会话管理。
- 会话正文、Timeline 和 Markdown 导出。
- 多会话协作入口。
- Leader / Teammate 这类总指导与执行线关系。
- 共享任务板。
- 权限确认队列。
- 工作区文件入口常驻。
- Skills 分层管理。
- 并行任务状态展示。
- 节点式工作流画布。
- 任务生命周期和 blocker 显示。
- runtime / adapter 抽象。
- 运行记录、轨迹和验证视图。
- React 画布底座和节点 / 边模型。
- 组件状态样例和 UI 回归基准。
- AI 友好的 open code 组件组织。
- 局部 React UI 面板草稿生成。

这些参考应优先转译为 Codex-only 版本：

- Leader 对应当前总指导线。
- Teammate 对应当前 Codex 开发线、验证线、信息架构线等工作线。
- 共享任务板对应 Codex 会话节点、任务状态、handoff、evidence、review。
- 权限确认队列对应生成任务文件、派发 Codex、运行 harness 等用户确认动作。
- runtime / adapter 对应当前 Codex 本机可用状态和后续 Codex CLI 接入。
- workflow 画布对应当前项目级 Codex 编排，不对应通用自动化平台。
- React Flow 对应画布底座，不对应产品业务模型。
- Storybook 对应组件状态样例，不对应产品运行时。
- shadcn/ui 对应组件组织方式，不强制技术栈迁移。
- v0 对应局部 UI 草稿，不对应最终状态机。

## 当前不吸收

当前不吸收：

- 多 agent 接入。
- 远程访问。
- 自动批准或全自动执行。
- 手机端控制。
- 聊天软件接入。
- 非 Codex agent 编排。
- 通用办公助手能力。
- 通用 workflow automation。
- agent 云平台。
- RAG / 知识库主线。
- 向量数据库主线。
- 技术栈迁移到 Next.js / Go / PostgreSQL / pgvector。
- 直接复制 Codex++ 的 CDP 注入和 provider 写入路线。
- 因为参考 React Flow / Storybook / shadcn/ui / v0 就立即重写当前工作台技术栈。

理由：

- 这些会扩大当前阶段范围。
- 当前 Codex 工作流尚未闭环。
- 第一版目标是治理 Codex，不是复刻 AionUi。

## 对当前任务的影响

当前后续顺序已纠偏为：

1. Codex 会话全文读取。
2. Codex 会话控制能力探针。
3. Codex 工作流编排运行模型。
4. 把任务包能力藏进工作流内部，作为交接和审计协议。

## 风险

- 如果过早追 AionUi 的多 agent 功能，会打断 Codex 工作流闭环。
- 如果完全忽略 AionUi 的协作视图优点，后续 UI 可能继续像普通表单，无法支撑用户想要的工作流感。
- 如果照搬 Codex++ 的 CDP 注入，Codex 页面结构变化会影响稳定性。
- 如果继续围绕任务包文件做 UI，会偏离用户想要的“在工作台里管理和编排 Codex 会话”。
- 如果让 AI 在没有组件样例和节点 schema 的情况下直接写复杂 React 画布，后续很容易退化成大组件和不可控状态。

当前折中：

- 保留参考源。
- 当前优先取 Codex++ 的会话管理参考、AionUi / Multica 的协作工作区参考。
- 画布开发时优先取 React Flow 的画布底座、n8n 的节点配置面板、Langflow 的可视化 builder、Storybook 的组件状态样例。
- 不改变当前只做 Codex 的阶段边界。
