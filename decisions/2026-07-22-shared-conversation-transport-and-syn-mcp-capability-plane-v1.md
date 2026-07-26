# 架构决策：共享 Conversation Transport + Syn MCP Capability Plane v1

日期：2026-07-22  
状态：**已拍**（用户确认复用并允许扩充现有对话模块；交办页为当前主消费面，智能体页降为次要 / 待定消费面）

## 1. 决策背景

交办页已经有自然对话 UI、方案卡、授权和执行闭环；智能体页已经有 Codex 会话发送、续接、轮询、Stop 和 thread event 映射。当前两者没有共用同一条产品 transport：

- 智能体页 direct send 只支持 Codex，并固定使用 `workspace-write + 项目根写根`；
- 交办页仍走 `submit_supervisor_resident_answer`、私有 `CODEX_HOME`、generation / rotate / resume 自愈等 resident 专用链；
- `submit_proposal` 的参数解析、幂等和 Pending 卡持久化已经存在，但授权外层绑定 resident run 与 active resident message；
- 当前公开 MCP `tools/list` 只有三个只读工具，resident run 才追加 `submit_proposal`；其余主管控制动作仍是 host-only，尚未形成整个 Syn 的统一能力注册与服务端角色授权面。

继续加固 resident/private-home 主路线会重复建设对话运输，并把 MCP 缩成交办页私有点卡通道。该方向停止作为主架构。

## 2. 拍板

### 2.1 一个共享 transport，交办页优先使用

Syn 只保留一套可扩充的 Conversation Transport：

- 复用现有 Codex relay 的 existing/new session、JSONL/thread event、poll、Stop、进程组清理和 readback；
- 抽离页面内发送/session 状态，形成 profile-driven 的共享逻辑；
- 交办页是第一主消费面和本阶段产品验收面；
- 智能体页是现有能力来源和次要 / 待定消费面，不再要求逐项保持既有交互，也不作为交办主线验收阻塞项；
- 智能体页暂不删除。退役或清理必须另有明确决定和引用扫描，不能由“大概率用不上”直接外推。

现有模块缺少主管需要的能力时，允许直接扩充共享 transport；禁止为了“复用”把整张 `AgentConversationShell` 塞进交办页，也禁止继续在页面组件内部堆新的主管专用状态。

### 2.2 profile 是权限边界，不是 UI 参数

共享 transport 必须接受由宿主选择、调用方不可放宽的 profile。首批至少包含：

- `agent-codex-workspace-write`：承接智能体页现有行为；
- `supervisor-read-only`：`sandbox=read-only`、`allowed_write_roots=[]`，不得产生 `--add-dir`，不得使用 wildcard、default allow-all、full-auto 或 approval bypass。

项目主管 profile 绑定 `project_id`、`project_root`、`workflow_id`、role、conversation turn identity 和精确 MCP capability set。前端不得自由拼装 sandbox、写根或能力集。

### 2.3 MCP 是整个 Syn 的统一能力层

自然语言仍由 Conversation Transport 承载；MCP 只承载结构化动作与结构化结果。统一能力层必须具备：

- 单一 capability registry；
- role/profile 级精确 allowlist；
- server-side 的 `tools/list` 与 `tools/call` 双重授权；
- schema 校验、项目/工作流/turn 绑定、幂等、审计和人话错误；
- 结果回到发起调用的同一 thread，并更新对应 Syn read model。

首个交办能力仍为 `submit_proposal`。现有 parser、store、幂等与 `PendingUserConfirmation` 语义保留；resident run-prefix / resident sidecar 绑定必须泛化为由 host 建立的可信 conversation-turn binding。不得新增另一套私有 MCP server 或专用 sidecar。

现有内部控制动作不因本决策自动变成公共 MCP 工具。每项能力必须在 registry、角色授权、schema、审计和验收齐全后才可宣称可用。

### 2.4 对话、动作、事实镜像分别结算

一次交互至少分开结算：

1. transport 是否接受用户消息；
2. 主管是否形成自然回复；
3. MCP 结构化动作是否成功；
4. proposal/read model 是否刷新；
5. canonical 审计镜像是否成功。

canonical 是事实镜像与审计面，不得作为发送聊天前的阻断式前置条件。镜像失败必须单独报告，不能吞掉已成立的自然回复或工具结果。工具失败也不能把成功对话改写成“没送到主管”。

### 2.5 卡片与执行闸不变

`submit_proposal` 只生成 `PendingUserConfirmation` 卡；工具成功后刷新 workflow/proposal read model。只有用户在方案卡上明确批准，才能进入既有 chain。共享 transport、MCP 能力注入和主管只读 profile 均不扩大执行授权。

### 2.6 其他 agent 的口径

当前真实 direct transport 只有 Codex。先冻结 adapter 合同，再逐类接入其他 agent；在 adapter、会话续接、事件回执、Stop 和 MCP 连接完成真实验收前，不得宣称 Claude Code、OpenClaw、OpenCode 或其他 agent 已可用。

## 3. 资产处置

### 保留并复用

- `manual_relay` 的进程生命周期、事件解析、poll / Stop 和 receipt；
- `conversationEngine` 的消息、assistant、live/tool event 转换；
- 交办页现有对话流、输入框、历届方案索引和右侧实体卡布局；
- `submit_proposal` parser、幂等、proposal store 与 Pending 人闸；
- M5 DB-primary / JSON 投影、canonical 审计和现有 proposal/workflow 刷新入口。

### 参数化 / 扩充

- transport/session 状态机；
- Codex command profile 与 sandbox 校验；
- project/workflow/turn binding；
- MCP capability registry 与 server-side allowlist；
- 对话、工具、投影、canonical 的分层 receipt；
- 交办 transcript 的权威数据源。

### 暂停作为主路线，但不删除

- `supervisor_resident_oneshot_session` 作为交办主对话运输；
- 继续加固交办专用私有 `CODEX_HOME`、generation、archive/rotate、invalid-resume 自愈并将其作为主路线；
- R3B、R4E、R4F、R4F-R1 围绕旧 resident 主运输的后续诊断与真实 App 续验；
- 继续把 S1B-H2 的 resident 两句→Pending 卡合同作为当前执行入口。

这些资产的错误分层、用户文案、幂等、审计和历史 evidence 仍有效；暂停的是它们作为当前主运输与下一步排期的地位。

## 4. 对旧决策和计划的关系

- 保留 `decisions/2026-07-18-conversation-substrate-correction-freeform-supervisor-plus-tools-v1.md` 的“自由对话 + MCP 结构化动作 + 用户卡片批准”原则；
- 取代其中“泛化 `submit_supervisor_resident_answer` / 继续 resident 主运输”的实现结论；
- `CURRENT.md`、`AUTHORITY.md` 与总执行计划必须停止把 S1B-H2 / R4F-R1 live 线列为当前第一优先；
- 历史 task、handoff、evidence 不回写、不删除、不伪装成从未发生。

## 5. 后续执行顺序

1. 冻结共享 transport / profile / capability / receipt / binding 接口的正式实施任务包；
2. 抽取并扩充共享 transport，先做离线回归；
3. 接入 `supervisor-read-only` 与服务端 MCP capability allowlist；
4. 交办页切换到共享 transport，保留自身布局与 Pending 人闸；
5. 离线全绿后另包、另授权真实 App 验收；
6. 替代验收通过前不删除旧 resident 路径。

## 6. 本决策不授权

本文只冻结架构、资产处置与排期。它不授权修改代码、启动 App、操作真实 store、发送真实消息、创建/批准方案卡、运行 chain、stage 或 commit。
