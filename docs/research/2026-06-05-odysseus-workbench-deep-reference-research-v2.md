# Odysseus Workbench Deep Reference Research v2

日期：2026-06-05

状态：外部项目深度研究参考，已作为 `docs/workbench-system-architecture-v1.md` 的最终蓝图参考约束登记。本文用于给全局主管审核 Odysseus 对最终工作台蓝图的参考价值、冲突点和后续研究方向。本文不进入中间版本计划，不进入 backlog，不拆任务包，不授权实现，不替代 `CURRENT.md`、`AUTHORITY.md`、`docs/plans/middleware-version-stage-plan-v1.md`、`docs/workbench-system-architecture-v1.md`、`docs/memory-layer-design-v1.md` 或最终蓝图。

## 0. 先说薄弱点

- 本轮仍然没有本地安装运行 Odysseus，也没有完整逐文件审计全部源码。
- Odysseus 更新非常快。v1 研究记录的仓库状态是 2026-06-04；本轮重新读取时，仓库已经更新到 2026-06-05。
- GitHub 星标、fork、issue 数只能说明关注度和活跃度，不能证明产品成熟、架构稳健或安全完备。
- 本轮读取的是公开 GitHub `dev` 分支的 README、ROADMAP、SECURITY、THREAT_MODEL、目录树和若干关键源码 / 测试文件；没有执行测试，也没有验证 demo 页面。
- Odysseus 是自托管 AI workspace，不是我们的最终架构蓝图。它可以作为参考和反面警示，不能覆盖我们已经确认的项目主轴、控制核心、正式记忆治理、方案授权制和 UI 显示边界。

## 1. 本轮和 v1 的区别

v1 已经回答了“Odysseus 有哪些功能、哪些方向值得看”。

v2 继续往下看：

- 当前仓库状态变化。
- 代码目录和模块边界。
- 权限和安全实现。
- Threat Model 承认的已知缺口。
- Memory / Skills 的实际实现倾向。
- Deep Research 的任务生命周期。
- Agent 工具执行和 workspace confinement。
- 测试文件暴露的真实工程风险。
- 这些内容如何映射到我们阶段 E / F / G / 最终蓝图。

大白话：

v1 是“这个项目有什么”。v2 是“这个项目的做法、风险和工程经验，哪些能被我们吸收，哪些必须挡在外面”。

## 2. 资料来源

外部项目：

- GitHub：`https://github.com/pewdiepie-archdaemon/odysseus`
- 默认分支：`dev`
- README：`https://raw.githubusercontent.com/pewdiepie-archdaemon/odysseus/dev/README.md`
- ROADMAP：`https://raw.githubusercontent.com/pewdiepie-archdaemon/odysseus/dev/ROADMAP.md`
- SECURITY：`https://raw.githubusercontent.com/pewdiepie-archdaemon/odysseus/dev/SECURITY.md`
- THREAT_MODEL：`https://raw.githubusercontent.com/pewdiepie-archdaemon/odysseus/dev/THREAT_MODEL.md`

本轮只读过的关键源码 / 测试：

- `core/auth.py`
- `src/tool_security.py`
- `src/prompt_security.py`
- `src/tool_execution.py`
- `src/agent_tools.py`
- `services/memory/service.py`
- `routes/memory_routes.py`
- `mcp_servers/memory_server.py`
- `services/research/research_handler.py`
- `tests/test_memory_extractor_vector_degraded.py`
- `tests/test_research_endpoint_owner_scope.py`
- `tests/test_webhook_ssrf_resilience.py`
- `tests/test_reserved_username_admin_escalation.py`
- `tests/test_workspace_confine.py`

本地对齐依据：

- `CURRENT.md`
- `tasks/README.md`
- `docs/workbench-system-architecture-v1.md`
- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/memory-layer-design-v1.md`
- `docs/plans/memory-layer-implementation-slice-v1.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- 最终蓝图：`/Users/yoyi/Documents/Codex/2026-05-26/gan-xing-codexbridge-https-github-com/docs/architecture/local-ai-workbench-blueprint-v1.md`
- UI 蓝图：`/Users/yoyi/Documents/Codex/2026-05-26/gan-xing-codexbridge-https-github-com/docs/architecture/local-ai-workbench-ui-blueprint-v1.md`

## 3. 当前仓库状态

本轮重新读取 GitHub repo metadata 时显示：

- 仓库：`pewdiepie-archdaemon/odysseus`
- 描述：`Self-hosted AI workspace.`
- 默认分支：`dev`
- 创建时间：`2026-05-31T14:05:51Z`
- 最近 push：`2026-06-05T13:22:08Z`
- 主语言：Python
- License：MIT
- GitHub Pages：`https://pewdiepie-archdaemon.github.io/odysseus/`
- 本轮读取时 GitHub API 显示星标和 fork 很高，但这不是成熟度证据。

语言统计本轮显示：

- Python 约 5.54M bytes。
- JavaScript 约 5.34M bytes。
- CSS 约 1.14M bytes。
- HTML 约 225K bytes。
- 另有 Shell、PowerShell、TypeScript、Dockerfile、Batchfile。

和 v1 的差异：

- v1 记录主语言为 JavaScript；本轮 GitHub API 返回主语言为 Python。
- v1 记录最近 push 为 2026-06-04；本轮已到 2026-06-05。
- 这说明 Odysseus 变化极快，后续任何融合前都必须重新固定 commit 或读取日期，不能拿旧研究当代码事实。

## 4. Odysseus 的真实定位

Odysseus 不是单纯聊天壳。

README 把它定义为 self-hosted AI workspace，目标接近 ChatGPT / Claude UI 体验，但运行在自己的硬件和数据上。

它的功能集合包括：

- Chat。
- Agent。
- Cookbook。
- Deep Research。
- Compare。
- Documents。
- Memory / Skills。
- Email。
- Notes & Tasks。
- Calendar。
- Mobile / PWA。
- Image editor、theme editor、file uploads、web search、presets、sessions、2FA。
- MCP、shell、file、model serving。

大白话：

Odysseus 是“个人自托管 AI 工作空间 + 本地模型管理 + agent 工具箱 + 轻办公套件”。它的中心更像 workspace / app 功能集合，不是项目主管制。

我们的最终蓝图中心是：

```text
项目
-> 方案授权
-> 项目主管
-> worker 任务包
-> 工作流画布
-> 过程事实确认
-> 正式记忆
-> 后续任务包召回
```

因此 Odysseus 不能被照搬成我们的主架构。

## 5. 工程结构观察

README 自述架构：

```text
app.py                   # FastAPI entry point
core/      auth, database, middleware, constants
src/       llm_core, agent_loop, agent_tools, chat_processor, search/
routes/    chat, session, document, memory, model ... endpoints
services/  docs, memory, search, hwfit (Cookbook) ...
static/    index.html + app.js + style.css + js/
docs/      landing page + clips
```

目录树本轮确认还包含：

- `mcp_servers/`
- `integrations/`
- `scripts/`
- `tests/`
- `companion/`
- Docker / GPU compose overlays。

对我们的启发：

- 它选择的是“自托管 Web app + FastAPI + 大量功能路由 + 服务模块”。
- 我们选择的是“本地 Tauri 桌面 + Rust 控制核心 + React 读模型 + 能力适配器”。
- 两者都不是微服务，但 Odysseus 更偏功能路由堆叠；我们更需要项目单元、控制核心、事实层、审计和读模型分层。

不建议照搬：

- 不用单个 web app route collection 直接承载所有核心事实变更。
- 不让 routes / UI 直接成为事实和权限中心。
- 不把工具执行、记忆写入、模型管理、邮件日历、工作流全部放成同一层能力。

## 6. 安全边界研究

### 6.1 它把自己当 admin console

SECURITY 和 THREAT_MODEL 都明确：Odysseus 是带高权限本地能力的 self-hosted workspace，不应公开无认证部署。

Threat Model 直接说明：

- 管理员可以执行 shell。
- 管理员可以读写文件。
- 管理员可以发邮件。
- 管理员可以控制模型服务。
- 非管理员默认不能用 shell、Python、文件读写、邮件、MCP、日历、token、webhook、模型服务、vault、settings 等能力。

这对我们的启发很重要：

- 本地 AI 工作台一旦能执行工具，就不能再当普通聊天 app 设计。
- 它应该像“本地总控台”一样处理权限、日志、审计、回滚、敏感数据和运行状态。
- 我们的方案授权制、控制核心、adapter capability、管理入口方向是正确的。

### 6.2 默认非管理员权限

`core/auth.py` 中 `DEFAULT_PRIVILEGES` 显示非管理员默认：

- 可以 use agent。
- 可以 use browser。
- 可以 use documents。
- 可以 use research。
- 可以 generate images。
- 可以 manage memory。
- 不能 use bash。

这套默认值对我们不是可直接照搬的答案。

原因：

- 我们不是多用户 Web workspace 优先。
- 我们的权限中心不是 admin / non-admin 二分，而是项目、任务包、工具能力、知识库、模型外发、记忆写入、方案授权和用户确认。
- “manage memory” 在 Odysseus 是普通 memory store 管理；在我们这里，正式记忆写入影响后续 agent 行为，不能默认给普通角色。

可借鉴：

- 区分高风险工具和普通功能。
- 非管理员默认禁用 shell / file / MCP / token / model serving。
- 账号、token、session、2FA、reserved username 都需要系统性测试。

不可照搬：

- 不把 admin 等同于“可以绕过项目和任务包边界”。
- 不把 can_manage_memory 等同于“可以写正式记忆”。

### 6.3 工具黑名单和执行链阻断

`src/tool_security.py` 定义了 `NON_ADMIN_BLOCKED_TOOLS`，包括：

- `bash`
- `python`
- `read_file`
- `write_file`
- `edit_file`
- `grep`
- `glob`
- `ls`
- `manage_memory`
- `manage_skills`
- `manage_tasks`
- `manage_endpoints`
- `manage_mcp`
- `manage_webhooks`
- `manage_tokens`
- `send_email`
- `read_email`
- `manage_calendar`
- `vault_*`
- `download_model`
- `serve_model`
- 以及任何 `mcp__` 开头的工具。

`src/tool_execution.py` 又在执行分发层阻断非管理员使用 admin 工具和 public-blocked tools。

对我们的启发：

- UI 禁用按钮不够，执行层必须 fail closed。
- adapter descriptor 不能只是展示能力，还要进入执行 guard。
- 工具权限必须在后端 / 控制核心校验，不能由前端决定。

对我们的改造方向：

```text
AdapterCapability
-> ToolRiskClass
-> ProjectEnabledCapability
-> TaskPackageAllowedTool
-> ControlCore guard
-> ExecutionAdapter
-> RuntimeLog + Audit
```

### 6.4 Prompt injection 防线

`src/prompt_security.py` 把外部内容、检索文档、web result、email、transcript、tool output、saved memory、skill text 都视为 untrusted data。

它提供：

- `UNTRUSTED_CONTEXT_POLICY`
- `UNTRUSTED_CONTEXT_HEADER`
- `untrusted_context_message(label, content)`

对我们的启发：

- 知识库、网页、邮件、日志、worker 输出、候选记忆、技能文本都应该按不可信材料处理。
- 它们不能被塞进 system prompt 当指令。
- 任务包里也要区分“用户指令 / 主管指令 / 正式记忆 / 资料引用 / 工具输出”。

映射到我们的记忆层：

- 正式记忆是受控事实，但仍然要带来源、版本、权限和状态。
- 知识库材料、候选、观察、LLM 摘要、检索命中都必须标成材料或候选，不能变成 agent 指令。

### 6.5 已知安全缺口

Threat Model 明确列出已知缺口：

- 没有 shell/filesystem sandbox。
- `/api/v1/chat` `base_url` 参数存在 SSRF 风险，PR #1039 修复中。
- `src/search/` 部分 consolidation 还可能 drift。
- token scopes 粗，不能给 session 精细 capability subset。

对我们的警示：

- 我们后续不能只说“本地运行所以安全”。
- 真正的本地 agent 工作台必须处理 sandbox、SSRF、token scope、路径 confinement、网络外发、内部服务暴露。
- 阶段 G 的运行日志、自动重试、运维诊断还不够，后续还需要安全威胁模型和执行沙箱设计。

## 7. Workspace confinement 和文件工具

`src/tool_execution.py` 的文件路径策略值得单独看。

它做了几层事：

- 默认工具工作目录指向项目 `data/`。
- read/write/edit file 有敏感路径 deny list。
- 默认允许 roots 是 data、tmp、TMPDIR 和设置里额外 roots。
- 支持 workspace 模式时，路径必须落在 workspace 内。
- 阻断 `.ssh`、`.gnupg`、shell rc、`.env`、`.netrc`、private key、authorized_keys 等敏感路径。
- `tests/test_workspace_confine.py` 覆盖了相对路径、绝对路径、越界路径、父级逃逸、敏感路径、bash/python cwd 等。

对我们有价值：

- 未来 Codex / Claude Code / OpenClaw / OpenCode adapter 的文件读写必须有项目 workspace root。
- 所有 agent 文件操作要能解释“为什么这个路径允许 / 拒绝”。
- 敏感路径 deny list 应是执行层默认防线。

但还不够：

- 它自己承认没有真正 shell/filesystem sandbox。
- workspace confinement 不能阻止 shell 自己访问网络或系统资源，除非 shell 被进一步沙箱化。
- 我们需要把“路径限制”和“进程沙箱 / 网络 egress / 工具权限”分开设计。

## 8. Memory / Skills 深度研究

### 8.1 Odysseus 的 memory 更像普通持久记忆

`services/memory/service.py` 暴露的是：

- `remember(text, session_id)`
- `recall(query, top_k)`
- `get_all(limit)`
- `delete(memory_id)`

它使用：

- `MemoryManager`
- `MemoryVectorStore`
- `NativeMemoryProvider`

`routes/memory_routes.py` 提供：

- add
- get
- search
- timeline
- by-session
- extract from chat
- audit / dedupe
- import from file
- pin
- update
- delete

它有 owner scope 和 session ownership check，这比纯单用户 memory store 更强。

但它仍然不是我们的正式记忆层。

缺少我们必须要的：

- observation -> candidate -> formal memory 的状态链。
- 正式记忆 version。
- 正式记忆 audit event。
- 作用域上升 / 下沉。
- 冲突 finding 阻断。
- 进入任务包的 included / excluded 理由。
- 用户偏好 / 全局蓝图 / 跨项目 / 高风险记忆确认边界。

### 8.2 它的 MCP memory server 对我们是反面提醒

`mcp_servers/memory_server.py` 暴露：

- list
- add
- edit
- delete
- search

并且 add 时 `source="ai_agent"`。

这对 Odysseus 的产品定位是合理的：agent workspace 里 agent 可以管理 memory。

但对我们来说，这是高风险模式：

```text
agent -> MCP memory server -> add/edit/delete memory
```

如果映射到我们的正式记忆层，会直接绕过：

- 项目主管确认。
- 用户确认。
- 来源审计。
- 版本。
- 冲突检查。
- 任务包影响面。

因此我们只能借鉴 MCP server 的“工具接口形态”，不能让 MCP 直接写 FormalMemory。

正确映射应该是：

```text
agent / MCP / knowledge
-> observation 或 MemoryCandidate
-> control core
-> project director / global director / user review
-> FormalMemory + version + audit
```

### 8.3 vector degraded 测试很有价值

`tests/test_memory_extractor_vector_degraded.py` 证明 Odysseus 曾经遇到一个重要问题：vector store 运行时失败会导致抽取出的事实全部丢失。修复后的预期是 vector backend 失败时，仍然 fallback 到 text/fuzzy dedup，事实仍写入 JSON store。

对我们的启发：

- 索引失败不能导致正式记忆丢失。
- 向量库只能是可重建索引，不是事实源。
- `index degraded` 应进入管理 / 运维状态。
- 任务包召回时，如果索引降级，应显示降级原因，而不是伪装成“没有相关记忆”。

这与我们的记忆层设计一致：

- 删除派生索引不能导致正式记忆丢失。
- 向量召回进入任务包前必须经过状态、权限、冲突、过期检查。

## 9. Deep Research 深度研究

`services/research/research_handler.py` 显示 Deep Research 不是一次普通聊天调用，而是一个可运行、可取消、可持久化结果的后台任务：

- `start_research`
- `get_status`
- `cancel_research`
- `get_result`
- `get_sources`
- `_save_result`
- fallback 到 legacy engine，再 fallback 到 basic web search。
- 结果保存到 `data/deep_research/{session_id}.json`。
- 报告包含 summary、sources、raw collected findings。

测试文件显示它还关注：

- owner scope。
- endpoint owner scope。
- path confinement。
- raw payload 非 dict。
- source link XSS。
- query fallback。
- report read。
- session id validation。

对我们的启发：

- Deep Research 很适合成为工作流节点 / 画布节点，不适合作为普通聊天插件。
- 它应该有自己的 run object、status、progress、source refs、result refs。
- 输出首先进入知识库材料、Observation 或 MemoryCandidate，不能直接写正式记忆。
- source list 和 raw findings 要可回看，但普通 UI 不应堆 raw findings。
- 研究失败要 fallback 和错误分类，不能显示成空报告。

映射到我们的蓝图：

```text
ProjectWorkflowNode(kind=research)
-> ResearchRun
-> ResearchReport
-> KnowledgeSourceRef
-> Observation / Candidate
-> ProjectDirector review
```

## 10. Agent / Tools 深度研究

Odysseus 的 agent 工具集合很大。

`src/agent_tools.py` 的 `TOOL_TAGS` 包括：

- bash / python。
- web_search / web_fetch。
- read_file / write_file / edit_file。
- grep / glob / ls。
- documents。
- search_chats。
- chat_with_model / create_session / send_to_session。
- manage_memory。
- manage_tasks。
- api_call。
- manage_skills。
- endpoints / MCP / webhooks / tokens / settings。
- notes / calendar / contact / email。
- Cookbook 模型下载 / serve / stop / list。
- image edit。
- trigger_research / manage_research。
- app_api。

这说明 Odysseus 的 agent 是“拿很多工具做完整任务”的 agent。

这和我们的差异很大：

- 我们的 worker 只能读任务包允许的最小上下文。
- 工具必须项目启用且任务包允许。
- 工具结果进入审计和节点详情，不直接成为事实。
- 项目主管确认过程事实。
- 全局主管复核方案和最终结果。

可借鉴：

- 工具 registry 要完整。
- 工具执行要有 progress。
- 长任务要可取消。
- 输出要截断和格式化。
- 文件写入要有 diff。
- 后台 job 要有结果回调或运行状态。

不可照搬：

- 不让 agent “拿到工具就跑完整任务”。
- 不让 app_api 成为万能后门。
- 不让 agent 自己决定是否读写记忆、模型服务、邮件、日历和文件。

## 11. Model / Cookbook / 运维

README 和 ROADMAP 对 Cookbook 的描述非常具体：

- 扫描硬件。
- 推荐模型。
- 下载模型。
- 启动模型服务。
- 支持 VRAM-aware、GGUF、FP8、AWQ、vLLM、llama.cpp。
- Docker GPU、NVIDIA、AMD、Mac Metal、Ollama、remote servers 都有说明。
- ROADMAP 高优先级里明确写了 Cookbook reliability、SGLang support、model scan/download ranking、error feedback and logging。

对我们的价值：

- 阶段 E 的模型 / 凭据底座不应只做“能填 API key”。
- 最终蓝图需要模型运行环境和健康状态。
- 本地模型支持需要硬件扫描、服务启动、失败日志、模型适配、成本 / 速度 / 外发边界。

但它不应成为当前一级入口：

- 我们已确认模型和 agent 相关底层内容不作为左侧一级入口。
- Cookbook 应归入 `管理 / 设置 / 运行环境`，不是普通工作区主界面。

## 12. Email / Calendar / Notes / Tasks

Odysseus 把这些都内置进 workspace。

价值：

- 说明 AI workspace 迟早会接收外部输入源。
- Email / Calendar / Notes / Tasks 可以成为秘书整理、通知、待办、知识库来源和工作流触发来源。

风险：

- 它们涉及敏感数据、账号权限、外部副作用和隐私。
- 直接内置会扩大产品边界。
- 对我们当前中间版本来说，容易冲散自动化工作流和记忆层主线。

建议：

- Notes / Tasks 可作为待办中心和想法箱的长期参考。
- Documents 可作为知识库和建议方案编辑体验参考。
- Email / Calendar 只作为远期 adapter 研究，不进入当前主线。
- 任何外部 inbox 都必须经过权限、来源、审计和秘书整理边界。

## 13. UI 和信息架构启发

Odysseus README 追求 ChatGPT / Claude-like UI。

这点对我们有价值，但不能照搬。

可借鉴：

- Chat / Agent 会话体验要接近原生产品，不要像治理后台。
- Settings / 管理中心承载 provider、model、MCP、logs、tokens、health。
- Deep Research 报告应该有 sources 和 summary。
- Documents / Notes / Tasks 需要独立工作区体验。
- 移动端和 PWA 思路可作为远期参考。

不能照搬：

- 不把 Chat 变成最高级对象。
- 不把 Email、Calendar、Cookbook、MCP、Shell、Gallery、Tasks 全部放成一级入口。
- 不让 raw logs、adapter schema、vector id、MCP details 进入普通主界面。
- 不把项目工作流画布变成普通工具箱。

我们的 UI 边界仍然是：

- 左侧一级入口：项目、智能体、画布、记忆、知识库、设置。
- 右侧入口：秘书、通知、待办、运行中、管理。
- 审计和日志进入管理。
- 项目页以项目工作流画布为主。
- 记忆中心要让用户看懂正式记忆、候选、来源、版本、审计、冲突、影响面。

## 14. 对当前阶段的适配建议

### 14.1 阶段 E：会话、adapter、多 agent 和模型凭据底座

Odysseus 最适合影响阶段 E。

建议吸收的设计点：

- Agent / Tool capability registry 不能只声明能做什么，还要声明风险等级。
- adapter health、provider probe、model endpoint owner scope 要进入设计。
- 会话中心要支持 agent 运行状态、权限需要、卡住原因、搜索和过滤。
- 模型 / 凭据要区分 provider、endpoint、owner、scope、外发风险、成本。
- 非管理员 / 非授权任务不能使用 shell、file write、MCP、tokens、model serving。

不建议阶段 E 做：

- 完整 Cookbook。
- Email / Calendar。
- Deep Research 完整产品化。
- Skills 自动化。
- GEPA / 自动优化。

### 14.2 阶段 F：项目工作流画布产品化深化

Odysseus 可影响：

- Deep Research 节点。
- Agent run 节点状态。
- 工具执行摘要。
- sources / raw findings 的详情抽屉。
- 长任务 progress / cancel。
- 文件 diff 和工具结果截断。

不建议：

- 不把画布变成“任意工具节点自动化平台”。
- 不让 React Flow 或 UI 节点成为事实源。

### 14.3 阶段 G：真实验收、运维日志和中间版本收口

Odysseus 对阶段 G 价值最大的一点是运维意识。

建议阶段 G 后续补：

- `WorkbenchRuntimeLog`。
- `AdapterHealth`。
- `ServiceDegradedState`。
- `ReadbackFailureReason`。
- `ModelEndpointProbe`。
- `ToolExecutionLog`。
- `DiagnosticBundleExport`。
- `IndexDegradedReport`。

不建议：

- 不把日志当审计。
- 不把审计当日志。
- 不把失败显示成空结果。
- 不让 demo 代替真实验收。

### 14.4 最终蓝图后续：技能层和外部输入源

Odysseus 让我们更明确：

- Skill 层要和 Memory 层分开。
- Deep Research 是工作流能力，不是正式记忆能力。
- Email / Calendar / Notes / Tasks 是外部输入 / 待办 /秘书材料源，不是工作流核心。
- Cookbook 是运行环境管理，不是普通用户主入口。

## 15. 值得借鉴的清单

建议保留为后续研究候选：

1. 管理中心。
2. adapter health。
3. model endpoint owner scope。
4. provider probe。
5. local model cookbook 的硬件扫描和服务日志。
6. workspace confinement。
7. sensitive path deny list。
8. prompt injection untrusted wrapper。
9. Deep Research 的 run / status / cancel / sources / fallback。
10. vector degraded 不丢事实。
11. memory import/export。
12. tool progress。
13. file diff。
14. Diagnostic bundle。
15. Skill 层单独设计。

## 16. 不建议吸收的清单

不建议吸收：

1. 大工具箱式一级入口。
2. Chat 作为最高级对象。
3. agent 自由调用工具完成整件事。
4. MCP 直接写正式记忆。
5. vector memory 直接影响 agent 行为。
6. LLM memory extraction 直接写正式记忆。
7. admin 能力绕过项目 / 任务包边界。
8. app_api 万能后门。
9. Email / Calendar 进入当前中间版本主线。
10. Cookbook 进入左侧一级入口。
11. Deep Research 报告自动成为项目事实。
12. raw logs / vector id / adapter schema 默认显示给普通用户。

## 17. 给全局主管的审核问题

建议新的全局主管审核时回答：

1. 是否接受 Odysseus v2 只作为外部研究参考，不进入当前计划。
2. 是否确认 Odysseus 的核心定位和我们不同：它是 workspace 工具箱，我们是项目总控台。
3. 是否把 Odysseus 的安全经验纳入阶段 E / G 的设计预留。
4. 是否允许后续单开“管理中心 / 运维日志 / adapter health”专题设计。
5. 是否允许后续单开“Skill 层 vs Memory 层”专题设计。
6. 是否允许后续单开“Deep Research 工作流节点”专题研究。
7. 是否确认 Email / Calendar 不进入中间版本主线。
8. 是否确认 Cookbook 不作为一级入口，只作为管理 / 设置 / 运行环境能力。
9. 是否确认 MCP / shell / file / model serving 都必须经过控制核心和任务包授权。
10. 是否确认 memory/skills/import/export 只能生成候选或受控材料，不能绕过正式记忆状态机。

## 18. 最终建议

建议全局主管给出如下口径：

```text
Odysseus 值得继续研究，但不进入当前中间版本执行计划。
它最有价值的是安全边界、运维日志、adapter health、Deep Research run 模型、workspace confinement、本地模型 cookbook 和 Skill / Memory 分离提醒。
它最不能照搬的是大工具箱信息架构、chat 中心、agent 自由工具执行、MCP 直接写 memory、vector memory 直接影响 agent 行为。
后续如果融合，必须先按专题拆研究，再进入设计文档，再拆任务包。
当前主线仍按 CURRENT.md 和 middleware-version-stage-plan-v1.md：阶段 E / 阶段 G 后续。
```

这份研究应当作为“镜子”使用：用来发现我们最终蓝图哪里需要补定义、哪里要加强安全和运维、哪里必须保持边界，而不是用来把 Odysseus 的功能清单搬进我们的工作台。

## 19. 本轮公开来源复核记录

2026-06-05 本轮再次只读复核公开来源，结论如下：

- GitHub API `https://api.github.com/repos/pewdiepie-archdaemon/odysseus` 显示默认分支为 `dev`，描述为 `Self-hosted AI workspace.`，主语言为 Python，最近 push 为 `2026-06-05T13:22:08Z`。
- GitHub API `https://api.github.com/repos/pewdiepie-archdaemon/odysseus/languages` 显示语言体量约为 Python 5.54M、JavaScript 5.34M、CSS 1.14M、HTML 225K。
- README 原文确认 Odysseus 的定位是 self-hosted AI workspace，并列出 Chat、Agent、Cookbook、Deep Research、Compare、Documents、Memory / Skills、Email、Notes & Tasks、Calendar、PWA 等能力。
- THREAT_MODEL 原文确认 Odysseus 自认是有高权限本地能力的 self-hosted workspace，建议按 admin console 看待；同时列出 shell/filesystem sandbox 缺失、`base_url` SSRF、token scope 粗等已知缺口。
- 以上复核只证明 Odysseus 当前公开仓库事实，不证明其成熟度、安全性或适合直接进入我们的开发计划。
