# Paseo Workbench Deep Reference Research v1

日期：2026-06-05

状态：已审核并登记为最终蓝图多 agent 运行层参考。本文用于记录 Paseo 对最终工作台蓝图的参考价值、适配阶段和冲突边界；已作为 `docs/workbench-system-architecture-v1.md` 的外部运行层参考约束登记。本文不进入中间版本计划，不进入 backlog，不拆任务包，不授权实现，不替代 `CURRENT.md`、`AUTHORITY.md`、`docs/plans/middleware-version-stage-plan-v1.md`、`docs/workbench-system-architecture-v1.md`、`docs/memory-layer-design-v1.md` 或最终蓝图。

## 0. 先说薄弱点

- 本地 `product-line` 和 `/Users/yoyi/workspace` 内未检索到既有 Paseo 研究文档或正文引用，所以本文按“第一次研究 Paseo”处理。
- 本轮没有本地安装运行 Paseo，也没有复现 daemon、desktop、mobile、CLI 或 relay。
- 本轮没有完整逐文件审计全部源码；重点读取了公开 GitHub metadata、README、SECURITY、architecture、data-model、agent lifecycle、providers、custom providers、CLI、MCP、worktrees、schedules、skills、configuration、best practices 等文档和部分源码入口。
- GitHub API / raw 读取中途出现 DNS / partial transfer 问题；已通过 paseo.sh docs 和已取得的 GitHub 文档交叉复核主要事实。
- Paseo 是快速发展的项目。本文只代表 2026-06-05 公开资料状态。
- Paseo 是 coding agent orchestration / daemon 工具，不是我们的项目主管制工作台，不能照搬为最终架构。

## 1. 本轮结论

Paseo 值得作为阶段 E / 阶段 G 的重要参考，但不应进入当前执行计划。

它最有价值的是：

- daemon 作为 agent runtime control plane。
- 多客户端连接同一个本地 agent daemon。
- agent lifecycle 和 timeline 的源头统一。
- provider adapter 抽象。
- provider snapshot / availability / model / mode 发现。
- CLI、desktop、mobile 共用同一协议。
- MCP / skills 让 agent 编排其他 agent 的接口设计。
- worktree 隔离并行 agent 的工程经验。
- schedule / loop / verifier 的长任务模型。
- relay E2E 加密和本地 daemon 暴露安全边界。
- daemon log、timeline、e2e、diagnostics 的运维意识。

它最不能照搬的是：

- 让 agent 自己通过 MCP 创建、取消、归档、等待、批准其他 agent。
- 把 agent / chat / timeline 当最高级对象。
- 把多 agent 编排权限交给 agent skill，而不是控制核心和授权方案。
- 把 permission request 只当 provider 操作确认，不纳入项目方案、任务包、记忆和审计边界。
- 用 file-based JSON 和 optional fields 替代我们正式记忆、审计、权限和版本治理。

大白话：

Paseo 很适合作为“怎么管很多 coding agent 进程、会话、权限、日志、手机远程操作”的参考；它不适合作为“谁能决定项目事实、谁能写记忆、谁能授权自动化工作流”的参考。

## 2. 资料来源

外部资料：

- GitHub：`https://github.com/getpaseo/paseo`
- GitHub API：`https://api.github.com/repos/getpaseo/paseo`
- README：`https://raw.githubusercontent.com/getpaseo/paseo/main/README.md`
- SECURITY：`https://raw.githubusercontent.com/getpaseo/paseo/main/SECURITY.md`
- Docs：`https://paseo.sh/docs`
- Security docs：`https://paseo.sh/docs/security`
- Providers docs：`https://paseo.sh/docs/providers`
- Custom providers docs：`https://paseo.sh/docs/custom-providers`
- CLI docs：`https://paseo.sh/docs/cli`
- MCP docs：`https://paseo.sh/docs/mcp`
- Worktrees docs：`https://paseo.sh/docs/worktrees`
- Schedules docs：`https://paseo.sh/docs/schedules`
- Skills docs：`https://paseo.sh/docs/skills`
- Configuration docs：`https://paseo.sh/docs/configuration`
- Best practices docs：`https://paseo.sh/docs/best-practices`

本轮只读过的关键仓库文档：

- `README.md`
- `SECURITY.md`
- `docs/architecture.md`
- `docs/data-model.md`
- `docs/agent-lifecycle.md`
- `docs/providers.md`
- `docs/custom-providers.md`

本地对齐依据：

- `CURRENT.md`
- `AUTHORITY.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `docs/workbench-system-architecture-v1.md`
- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/memory-layer-design-v1.md`
- `docs/plans/memory-layer-implementation-slice-v1.md`
- 最终蓝图：`/Users/yoyi/Documents/Codex/2026-05-26/gan-xing-codexbridge-https-github-com/docs/architecture/local-ai-workbench-blueprint-v1.md`
- UI 蓝图：`/Users/yoyi/Documents/Codex/2026-05-26/gan-xing-codexbridge-https-github-com/docs/architecture/local-ai-workbench-ui-blueprint-v1.md`

## 3. 当前仓库事实

2026-06-05 公开 GitHub API 显示：

- 仓库：`getpaseo/paseo`
- 描述：`Coding agents from your phone, desktop and CLI`
- 默认分支：`main`
- 创建时间：`2025-10-13T12:27:02Z`
- 最近 push：`2026-06-05T14:31:39Z`
- homepage：`https://paseo.sh`
- 主语言：TypeScript
- license：README 写 AGPL-3.0；GitHub API license 显示为 Other / NOASSERTION。
- topics 包括 `agents`、`claude-code`、`codex`、`copilot`、`opencode`、`orchestration`、`mobile`。

语言统计：

- TypeScript 约 16.38M bytes。
- JavaScript 约 124K bytes。
- 另有 Shell、Swift、Nix、Kotlin、CSS、PowerShell、Ruby、HTML、Batchfile。

## 4. Paseo 是什么

Paseo 官方 README 把它定义为一个统一界面，用来运行 Claude Code、Codex、Copilot、OpenCode、Pi 等 coding agents。

它的核心不是自己造一个新 agent，而是：

```text
已有 agent CLI / SDK
-> Paseo daemon 统一启动和管理
-> WebSocket protocol
-> desktop / mobile / web / CLI 多端连接
-> timeline / permissions / provider / worktree / schedule / loop / relay
```

大白话：

Paseo 是“本地 agent 进程编排器 + 多端遥控器”。它像 Docker daemon 管容器一样管 coding agents。

我们的工作台是：

```text
项目
-> 方案授权
-> 项目主管
-> worker 任务包
-> 过程事实确认
-> 正式记忆
-> 后续任务包召回
```

所以 Paseo 可以参考 agent runtime，但不能覆盖我们的项目治理核心。

## 5. 工程结构观察

Paseo README 列出的 monorepo 包：

- `packages/server`：Paseo daemon，负责 agent process orchestration、WebSocket API、MCP server。
- `packages/app`：Expo client，覆盖 iOS、Android、web。
- `packages/cli`：`paseo` CLI。
- `packages/desktop`：Electron desktop app。
- `packages/relay`：remote connectivity relay。
- `packages/website`：官网和 docs。

`docs/architecture.md` 进一步说明：

- daemon 是核心。
- client 通过 WebSocket 连接 daemon。
- desktop app 可以把 daemon 当 managed subprocess。
- relay 用于远程访问。
- provider 负责包装 Claude、Codex、Copilot、OpenCode、Pi。
- timeline 是 append-only，客户端通过 live stream 和 authoritative fetch 保证一致性。

对我们的启发：

- 阶段 E 需要一个清晰的 `AgentRuntimeControlPlane` 或等价边界。
- 前端不应该自己拼真实 agent 状态，应该读后端 authoritative read model。
- 会话中心需要分清 live stream、paged fetch、catch-up、dedupe 和 state snapshot。
- 多 agent 接入必须有 shared protocol / type 层，而不是每个页面各写各的 provider 逻辑。

## 6. 安全边界研究

### 6.1 本地 daemon 是高权限控制面

Paseo SECURITY 明确说 daemon 管理本机 coding agents，客户端能监控和控制 agents。

默认 daemon 绑定 `127.0.0.1`。如果没有密码，能访问 daemon socket 的进程就能控制 daemon。这类似 Docker daemon 的安全模型。

对我们有价值：

- 工作台只要能停止、继续、发消息、派发 agent，就不是普通 UI。
- 本地 loopback 也不是绝对安全边界。
- daemon / control core 必须有明确“谁能控制 agent”的规则。

对我们的风险提醒：

- 不能因为“本地运行”就省掉权限、日志和审计。
- 前端按钮隐藏不是安全边界。
- agent 自己调用 CLI / MCP 管其他 agent 更不是安全边界。

### 6.2 Relay E2E 加密

Paseo relay 设计成不可信 relay：

- daemon 首次运行生成持久 Curve25519 keypair。
- pairing URL / QR code 带 daemon public key。
- phone 用 ephemeral key handshake。
- 双方 ECDH 得到 shared key。
- 后续消息用 XSalsa20-Poly1305 / NaCl box。
- relay 只能看到 IP、时间、消息大小、session id 和公开握手帧。

对我们有价值：

- 如果最终工作台要手机/远程查看 agent 运行状态，relay 不能读用户代码和 agent 内容。
- QR / pairing link 是信任锚，必须像密码一样处理。
- 远程连接应该优先 E2E，而不是直接把 daemon 暴露到公网。

### 6.3 Direct connection 风险

Paseo docs 明确警告：绑定 `0.0.0.0` 会让 daemon 可被网络访问，必须配密码、host allowlist、防火墙。

它还做：

- Host header allowlist，防 DNS rebinding。
- CORS origin 检查。
- optional password auth，bcrypt hash 存储。
- WebSocket 通过 subprotocol 传 bearer password。

对我们的阶段 G 价值很高：

- 后续 `管理 > 日志 / 诊断` 不能只做日志，还要能显示 daemon 暴露面。
- 需要检查 listen address、host allowlist、password、relay 状态。
- 如果有手机/远程入口，必须把连接安全设计成一等能力。

## 7. Agent lifecycle 和 timeline

Paseo 的 agent 状态：

```text
initializing -> idle <-> running
              -> error
              -> closed
```

重要设计：

- `AgentManager` 是 agent state 的 source of truth。
- 状态变更持久化到磁盘，并通过 WebSocket 推送。
- timeline append-only。
- 每次 run 有 epoch。
- storage 用 sequence number 支持客户端 dedupe。
- live stream 只为即时性；authoritative timeline fetch 才保证正确性。
- 默认 fetch page 是 200 rows。
- timestamp 是 daemon-owned canonical timestamp。

这正好对应我们会话中心之前暴露的问题：

- 会话列表状态滞后。
- transcript / sqlite / index 双轨。
- 对话流没有正确收纳。
- 真实运行状态看不到。

可借鉴口径：

```text
AgentRuntimeSnapshot
AgentTimelineEvent
AgentTimelineFetch
AgentLiveStreamEvent
AgentPermissionRequest
AgentStateTransition
AgentRuntimeAttention
```

不应照搬：

- 不把 agent 状态等同于项目状态。
- 不让 parent agent 的子 agent 自动影响项目事实。
- 不让 timeline event 自动进入正式记忆。

## 8. Agent 关系和子代理

Paseo 支持 agent 通过 MCP 创建其他 agents，并用 `paseo.parent-agent-id` label 标记父子关系。

关系分两类：

- subagent：默认，属于创建它的 parent agent。
- detached agent：独立根 agent，不出现在 parent track 中。

archive 也有 cascade：

- root agent archive 会递归 archive children。
- subagent tab close 只是 layout，不等于 archive。

对我们有价值：

- 多 agent 关系需要显式 parent / child / detached 模型。
- UI 上 tab / view 关闭和 agent 生命周期必须分开。
- 子 agent 积累需要清理策略。
- workspace aggregate activity 可以把子 agent running 归到 root 所在 workspace。

和我们冲突：

- 我们的 worker 不应该由普通 agent 自由创建。
- 子 agent 关系应该从项目主管 / 任务包派发而来，而不是 agent MCP 自治。
- cascade archive 不能替代工作流取消、回滚和审计。

## 9. Provider adapter 研究

Paseo provider mental model：

```text
provider = how to launch external agent CLI, stream output, send input, expose modes/models/features
```

内置 provider：

- Claude Code。
- Codex。
- Copilot。
- OpenCode。
- Pi。
- 另有 Cursor / Generic ACP / Mock 等 adapter。

两种接入方式：

1. ACP：推荐。继承 `ACPAgentClient`，由基类处理 process spawn、stdio transport、session lifecycle、streaming、permissions、model discovery。
2. Direct：直接实现 `AgentClient` / `AgentSession`。

对我们阶段 E 非常有价值：

- 我们后续要接 Claude Code、OpenClaw / OpenCode、Codex，本质也需要 provider adapter。
- provider 应暴露 capabilities、modes、models、features、availability、diagnostic、persistence handle。
- provider snapshot 应按 cwd scope 缓存，不应该每次 UI 打开都重新 probe。
- setting refresh 是显式动作，不应隐式乱刷 provider。

但要加我们的边界：

- provider capability 只是“技术能力”，不能等同“当前项目允许使用”。
- provider modes 里的 bypass / full-access 不能绕过方案授权。
- provider auth 不由工作台直接偷读 token；只能报告 availability / diagnostic。

## 10. CLI 和 agent-to-agent 编排

Paseo CLI 暴露：

- `paseo run`
- `paseo ls`
- `paseo attach`
- `paseo send`
- `paseo logs`
- `paseo stop`
- `paseo wait`
- `paseo permit`
- `paseo agent mode`
- `paseo daemon`
- `paseo schedule`
- `paseo worktree`

CLI 明确支持让 agents 自己使用 Paseo CLI 来 spawn / manage other agents。

对我们有价值：

- CLI/API 是好边界：app 能做的，CLI 也能做，方便测试和运维。
- `run / attach / send / logs / stop / wait` 是阶段 E 会话操作的基本能力集。
- `--output-schema` 很适合结构化 worker report / verifier report。

对我们高风险：

- 让 agent 自己“paseo run”另一个 agent，等同把派发权交给 agent。
- 我们的派发必须由控制核心、方案授权、项目主管任务包驱动。
- agent 可以建议创建 worker，但不能直接创建正式 worker。

## 11. MCP 和 Skills

Paseo MCP 可注入到新 agents，工具包括：

- create / wait / send / status / list / cancel / archive / kill / update agent。
- list / create / kill / capture / send terminal。
- create / list / inspect / pause / resume / delete schedules。
- list providers / models / provider capabilities。
- create / list / archive worktrees。
- list / respond permissions。

Paseo skills 包括：

- `/paseo-handoff`
- `/paseo-loop`
- `/paseo-advisor`
- `/paseo-committee`
- `/paseo-epic`

对我们有价值：

- Skill 层可以教 agent 使用工作台能力。
- handoff / advisor / committee / loop 都是很好的工作模式分类。
- advisor / committee 的“analysis-only, no edits”边界值得吸收。
- epic 的 plan file 作为 source of truth 对长任务有启发。

必须拒绝的做法：

- MCP 直接给 agent 创建 worker、批准权限、kill agent 的能力。
- skill 自动变成项目授权。
- autopilot 跳过用户方案确认和全局主管结果复核。

正确映射应该是：

```text
Agent/Skill suggests action
-> Control Core validates authorization
-> Project Director creates task package
-> Worker run
-> RuntimeLog + Timeline + Audit
```

## 12. Worktrees 和并行隔离

Paseo Git worktree 设计：

- 每个 agent 可运行在独立 git worktree。
- worktrees 默认在 `$PASEO_HOME/worktrees/`。
- `paseo.json` 可声明 setup、teardown、scripts、services。
- 服务可由 daemon reverse proxy 暴露到 deterministic localhost hostname。
- 每个 worktree 有独立 port / URL 环境变量。

对我们非常有价值：

- 并行 worker 不应直接挤在同一个 checkout。
- 工作流节点可以使用 isolated workspace。
- 自动化任务必须知道 setup / teardown / service / test entrypoint。
- UI 可以显示 worktree diff、service URL、script status、teardown 状态。

但风险也明显：

- setup 里复制 `.env` 这种动作必须有权限和审计。
- teardown / rm 操作必须控制风险。
- service proxy 不能绕过安全边界。
- worktree 隔离不是完整 sandbox。

## 13. Schedules / loops / verifier

Paseo schedules 让 agent 定时回来：

- new agent each time。
- existing agent。
- self heartbeat。
- interval / cron。
- run history。
- pause / resume / run once / update / delete。

Paseo loop 数据模型包含：

- worker agent。
- verifier agent。
- shell checks。
- max iterations。
- max time。
- archive worker。
- logs。

对我们有价值：

- 自动化工作流需要 recurring / long-running run object。
- verifier 和 worker 分离是正确方向。
- shell check 和 LLM verifier 都可以成为验收信号。
- run history、log seq、active worker / verifier 是运维基础。

和我们差异：

- 我们的全局主管不是每轮 verifier。
- 项目主管可以看 worker 汇报；全局主管看方案和最终结果。
- schedule 不能绕过用户方案授权。

## 14. 存储和日志

Paseo 用 file-based JSON persistence，不用传统数据库。

重要路径：

```text
$PASEO_HOME/
agents/{cwd}/{agent-id}.json
projects/projects.json
projects/workspaces.json
chat/rooms.json
schedules/{id}.json
loops/loops.json
config.json
daemon-keypair.json
push-tokens.json
daemon.log
```

它的 data-model 文档明确：

- 多数 store 原子写。
- 少数仍 direct write。
- 没有完整 schema-versioning / migration framework。
- 用 Zod runtime validation。
- optional fields + defaults 做 forward compatibility。

对我们有价值：

- sidecar JSON 适合原型和早期本地状态。
- daemon.log、rotate、trace level 是阶段 G 必要能力。
- timeline rows 和 agent record 分离值得参考。

对我们不够：

- 正式记忆、审计、权限、版本、关系治理不能靠 optional fields 长期撑。
- 我们中间版本后续如果进入真实产品，需要明确 migration / schema version / store interface。
- 日志不是审计；agent timeline 也不是正式事实。

## 15. UI / 产品启发

Paseo 的 UI 方向：

- mobile / desktop / web 多端。
- workspace / agent pane。
- subagent track。
- agent timeline。
- permission request。
- terminal / diff / scripts / services。
- settings 里管理 providers、integrations、relay、host。

对我们有价值：

- 智能体中心要像原生 agent 客户端，而不是治理后台。
- 会话列表需要实时状态、attention、permission、running、error。
- agent timeline 应折叠工具调用、流式输出、permission 和 user message。
- 运行中入口和通知中心可以借鉴 agent attention。
- 管理入口应放 provider、daemon health、logs、relay/security。

不能照搬：

- 不把 agent pane 变成工作台最高级页面。
- 不让 subagent track 代替项目工作流画布。
- 不把 raw timeline、provider schema、terminal internals 默认展示给普通用户。

## 16. 和记忆层 / 知识库的关系

Paseo 不是记忆层项目。

它有：

- agent timeline。
- logs。
- chat rooms。
- schedules。
- loop logs。
- skills。
- provider persistence handles。

但这些都不是我们的正式记忆。

正确映射：

```text
Agent timeline / logs / tool output
-> Observation
-> MemoryCandidate
-> FormalMemory
-> version + audit
```

错误映射：

```text
Paseo timeline event
-> FormalMemory
```

Paseo 对记忆层的价值主要是“提供高质量观察来源”，不是替代记忆治理。

## 17. 对当前阶段的适配

### 17.1 阶段 E：高度相关

Paseo 最适合影响阶段 E：

- agent adapter。
- 会话中心发消息。
- stop / resume / wait / attach / logs。
- provider availability。
- modes / models / features。
- permission request。
- agent timeline。
- multi-agent relationship。
- provider snapshot。
- custom provider / ACP。

建议阶段 E 预留的数据结构：

```text
AgentProviderDescriptor
AgentRuntimeSession
AgentPersistenceHandle
AgentTimelineEvent
AgentPermissionRequest
AgentRuntimeAttention
ProviderSnapshot
ProviderMode
ProviderModel
ProviderFeature
AgentParentChildRelation
```

### 17.2 阶段 F：画布 / 工作流深化相关

Paseo worktrees、scripts、services、loops 对阶段 F 有价值：

- worker run 节点。
- verifier run 节点。
- setup / teardown 节点。
- service node。
- diff review node。
- schedule / recurring node。

但画布仍必须由项目工作流和控制核心驱动，不由 agent skill 自由造任务。

### 17.3 阶段 G：运维和安全相关

Paseo 对阶段 G 价值很高：

- daemon.log。
- log rotation。
- health endpoint。
- relay status。
- host allowlist。
- password auth。
- DNS rebinding protection。
- provider diagnostics。
- agent timeline catch-up。
- e2e real provider tests。

我们后续的 `管理` 入口应该吸收这些能力，但仍区分：

- runtime log。
- audit。
- formal memory audit。
- workflow ledger。
- user-visible status。

### 17.4 不适合直接进入记忆层阶段

Paseo 不应影响已完成的 M1-M13 记忆层结论。它可以作为 Observation source 研究，但不能改写正式记忆设计。

## 18. 推荐未来研究切片

当前不授权实现，但后续可拆研究：

### PASEO-0：Agent Runtime Control Plane 对比设计

目标：定义我们的 agent runtime 是否需要 daemon-like control plane。

输出：

- runtime session 模型。
- timeline 模型。
- permission request 模型。
- provider snapshot 模型。
- logs / audit 分层。

### PASEO-1：Provider Adapter Contract 对齐

目标：对比 Paseo provider 和我们的 adapter descriptor。

输出：

- Codex / Claude Code / OpenCode / OpenClaw adapter 合约。
- capability / mode / model / feature / auth / diagnostic 字段。
- provider snapshot refresh 规则。

### PASEO-2：Agent Timeline 和会话中心设计

目标：把会话中心从 transcript viewer 升级为 runtime session viewer。

输出：

- live stream vs authoritative fetch。
- sequence dedupe。
- pagination。
- tool call folding。
- permission / attention / error UI。

### PASEO-3：Worktree Isolation 和 Workflow Node

目标：评估 worker 是否默认用 worktree / isolated checkout。

输出：

- setup / teardown。
- service proxy。
- diff review。
- destructive command guard。

### PASEO-4：Remote / Mobile / Relay 安全研究

目标：如果最终工作台要手机远程查看 / 控制，先定 relay 和安全边界。

输出：

- pairing。
- E2E。
- password auth。
- host allowlist。
- daemon exposure diagnostics。

## 19. 值得借鉴的清单

建议保留为后续研究候选：

1. daemon-like local control plane。
2. shared WebSocket protocol。
3. desktop / mobile / CLI 共用 daemon API。
4. AgentManager 作为 state source of truth。
5. append-only timeline。
6. live stream + authoritative fetch。
7. sequence dedupe。
8. permission request model。
9. provider adapter contract。
10. provider snapshot per cwd。
11. explicit settings refresh。
12. ACP custom providers。
13. worktree isolation。
14. setup / teardown / scripts / services。
15. loop worker + verifier。
16. schedules with run history。
17. relay E2E encryption。
18. host allowlist / DNS rebinding protection。
19. daemon log + rotation。
20. CLI parity for app operations。

## 20. 不建议吸收的清单

不建议吸收：

1. agent 自己通过 MCP 直接创建正式 worker。
2. agent 自己批准权限。
3. agent 自己 kill / archive 其他正式 worker。
4. skill autopilot 跳过用户方案确认。
5. chat / agent 作为最高级对象。
6. timeline event 自动成为正式事实或正式记忆。
7. provider mode 直接等同工作台权限。
8. file-based optional schemas 支撑正式记忆长期治理。
9. relay / direct daemon 暴露细节默认显示给普通用户。
10. worktree setup 直接复制敏感文件而无审计。
11. schedule 绕过方案授权。
12. loop verifier 替代项目主管 / 全局主管分工。

## 21. 给全局主管的审核问题

建议新的全局主管审核时回答：

1. 是否确认 Paseo 之前没有本地研究沉淀，本文作为 v1 外部研究。
2. 是否确认 Paseo 只作为阶段 E / G 参考，不进入当前计划。
3. 是否接受 daemon-like agent runtime control plane 是后续重点研究方向。
4. 是否接受 provider adapter / provider snapshot / timeline 进入阶段 E 设计预留。
5. 是否确认 Paseo MCP / skills 不能绕过我们的控制核心和方案授权。
6. 是否允许后续单开 `AgentTimelineEvent` / `AgentRuntimeSession` 专题设计。
7. 是否允许后续单开 worktree isolation 专题设计。
8. 是否允许后续单开 remote/mobile/relay 安全专题设计。
9. 是否确认 agent timeline 只能作为 observation source，不能直接写正式记忆。
10. 是否确认 logs / runtime timeline / audit / formal memory audit 必须分层。

## 22. 最终建议

建议全局主管给出如下口径：

```text
Paseo 值得继续研究，但不进入当前中间版本执行计划。
它最有价值的是 agent runtime control plane、provider adapter、timeline、permission、worktree isolation、CLI parity、relay security 和 daemon operations。
它最不能照搬的是 agent 自治编排、MCP 直接管理其他 agent、skill autopilot 跳过授权、以及把 agent timeline 当项目事实或正式记忆。
后续如果融合，必须先按阶段 E / G 专题拆研究，再进入设计文档，再拆任务包。
当前主线仍按 CURRENT.md 和 middleware-version-stage-plan-v1.md。
```

这份研究应当作为“多 agent 运行层参考”使用：帮助我们把会话中心、adapter、运行中工作流、日志、远程访问和 worktree 隔离想清楚，而不是把 Paseo 的 agent-first 产品结构搬进我们的项目总控工作台。

## 23. 本轮公开来源复核记录

2026-06-05 本轮只读复核公开来源，结论如下：

- 本地 `rg -i paseo` 没有找到既有 Paseo 研究文档或正文引用。
- GitHub API `https://api.github.com/repos/getpaseo/paseo` 显示仓库为 `getpaseo/paseo`，描述为 `Coding agents from your phone, desktop and CLI`，默认分支为 `main`，最近 push 为 `2026-06-05T14:31:39Z`。
- GitHub language API 显示 TypeScript 为主要语言，约 16.38M bytes。
- README 原文确认 Paseo 是 Claude Code、Codex、Copilot、OpenCode、Pi 等 agents 的统一界面，并以 daemon 管理 agents。
- SECURITY 原文确认 daemon / client 架构、relay E2E、direct connection、password auth、host allowlist、provider auth 不由 Paseo 管理等安全边界。
- `docs/architecture.md` 确认 daemon、WebSocket protocol、AgentManager、timeline、provider adapter、desktop managed subprocess、relay、CLI 的架构关系。
- `docs/data-model.md` 确认 `$PASEO_HOME` 下 file-based JSON persistence、Zod validation、agent / schedule / chat / loop / project / workspace / daemon meta 文件结构。
- `docs/agent-lifecycle.md` 确认 initializing / idle / running / error / closed 状态、parent-agent label、subagent / detached agent、archive cascade 和 tab/archive 区分。
- `docs/providers.md` 和 `docs/custom-providers.md` 确认 provider contract、ACP / Direct 两种 provider 接入方式、custom provider、models / modes / env / command override。
- docs CLI / MCP / worktrees / schedules / skills / configuration / best practices 确认 CLI parity、MCP tools、worktree setup / services、schedule runs、orchestration skills 和 daemon logging。
- 以上复核只证明 Paseo 当前公开实现和文档事实，不证明它适合直接进入我们的当前开发计划。
