# Agent 协同参考消化 + 任务包正本差量对照 · B2·C0 v1

日期:2026-07-08 · 出自:执行线(咨询/研究角色,纯调研,零产线代码)· 任务包:`tasks/2026-07-08-phase-b2-c0-research-and-gap-schema-v1.md`。

回交对象:主导线。本文只出研究结论与差量清单,**不替主导线写定稿决策**。

---

## 0. 一句话结论

Claude 侧的"上下文隔离/单进单出/编排器中转/对抗式复核"四条模式,工作台已用**设计层面的同构模式**部分落地(worker 回程契约、独立复核线),但**结构化契约的严格度选型(软着陆 vs schema强制+重试)从未被明确拍过**,这是留给 C2/C3 的一个真实待决策点。Codex CLI 实测发现两个此前文档未记录的原生新能力(`multi_agent`/`memories` 均已默认开启)需要专门划清"不吸收"边界。差量表核心结论:任务包正本(§3.4/§3.5/§3.6)的字段名在现状代码里**大量已存在**,但相当一部分是"有名无实"——恒为硬编码空值/固定字符串,或存在**两条互不相通的并行实现**(SubagentReport 即是典型案例,见 §5.3)。M4 遗产`TaskMemoryPacketBuilder`**不能直接**用于 C2 的"前序任务口供摘要"注入(架构性拒绝,非缺配置)。

---

## 1. Claude 多 agent 协同设计(公开资料 + 库内先例)

体例照 `docs/workbench-system-architecture-v1.md` §5.8/§5.9(odysseus/paseo 先例)。

### 1.1 可吸收为设计参考

依据:Claude Agent SDK 官方文档(`https://code.claude.com/docs/en/agent-sdk/subagents`、`https://code.claude.com/docs/en/workflows`,2026-07-08 抓取)+ 库内先例(`evidence/2026-06-08-stage-i-i0-...md` 的 Reference Mapping、`worker_report.rs` d2dba24/368f126 落地)。

- **上下文隔离**:"Each subagent runs in its own fresh conversation. Intermediate tool calls and results stay inside the subagent; only its final message returns to the parent."——子智能体的中间过程(工具调用、推理)不进父级上下文,只有最终结果回传。**库内已落地的同构模式**:worker 回程契约(d2dba24)只读 `last_message_path` 最终一条消息,不读取 worker 会话的中间工具调用记录。
- **单进单出信道**:"The only channel from parent to subagent is the Agent tool's prompt string"——父到子只有一条 prompt 通道,子到父只有一条 final message 通道,没有旁路。**库内已落地**:`build_goals_with_contract` 是唯一的父→子输入通道(objective + report_format + 契约文本拼进 goals);`parse_worker_report` 是唯一的子→父输出通道(只认最后一条消息里的一个 json 块)。
- **编排器中转数据,子智能体互不直连**:"A workflow script holds the loop, the branching, and the intermediate results itself"、"No direct filesystem or shell access from the workflow itself — Agents read, write, and run commands. The script coordinates the agents"——子智能体之间不直接通信,一切数据流经编排器(脚本/父 agent)中转。**库内已落地**:worker 之间没有任何直连通道,`run_director_task_chain` 是唯一的链驱动者,worker 完成后的口供先回 director、再由 director 落库,worker 之间的依赖(`depends_on`)完全由 director 拓扑排序驱动。
- **对抗式复核**:"it can have independent agents adversarially review each other's findings before they're reported"——独立 agent 互相核验对方结论,不是自己批自己。`/deep-research` 内置工作流的做法是"fetches and cross-checks the sources it finds, votes on each claim, and returns a cited report with claims that didn't survive cross-checking filtered out"。**库内已落地的强对应**:product-line 的每个任务包完成后都经由**独立复核线**(如 Aquinas/Maxwell/Parfit 等具名复核线程)只读核验、判定 `CLEAR`/`CLEAR_WITH_P2` 等,复核线与执行线是不同的 agent 实例——这本身就是"对抗式复核"模式在治理层面的成熟实践,且比 SDK 层面的 subagent 复核更重(带书面判定文档)。
- **结构化契约(schema 强制)**:Workflow 工具的 `agent(prompt, {schema})` 选项——"the subagent is forced to call a StructuredOutput tool and agent() returns the validated object...validation happens at the tool-call layer so the model retries on mismatch"(工具规格原文,本次会话内验证)。**库内现状是弱对应,非同构**:见 §1.3 待决策点。
- **嵌套但有界**:"subagents can spawn their own subagents. A subagent five levels below the main agent can't spawn further subagents"——上下文隔离不等于绝对单层,嵌套允许但有深度上限。**库内 stage-h-i 计划 I4 的"parent / child / sibling / detached run 关系"与此同构**,但 I4 仍是"必须来自项目主管任务包和控制核心"派发,不是 agent 自治嵌套。

### 1.2 明确不吸收

- **不能让工作台的 worker 派发权移交给"LM 自主决定何时 spawn"**——Claude 的 Agent tool 本身是由 Claude(LM)在对话轮次里自主判断"要不要调用子智能体、调用哪个",这是 Claude Code 产品自身的运行时决策权。工作台的 worker 派发权在**项目主管任务包 + 控制核心**,不能因为参考了这套协作模式就把派发决策下放给 LM——这与 §5.10"派发必须来自任务包+控制核心,不能来自 agent 自治 spawn"、stage-h-i 计划 §5.2"不能让 Codex 自带协作能力绕过控制核心"是同一条边界的另一半(对 Claude 侧同理)。
- **不能把"Agent tool 曾从 Task 改名"这类 SDK 实现细节当成工作台需要跟随的规范**——这是 Claude Code 自身的 API 演进史,不构成工作台协议设计依据。
- **不能假设 SDK 层的"上下文隔离"已经解决工作台的治理问题**——SDK 的隔离机制解决的是 token/上下文管理(避免子智能体的中间过程污染父级上下文窗口),这和"谁能写正式记忆""谁的汇报可信"是完全不同维度的问题;不能因为 SDK 有上下文隔离就误认为治理链路(候选→正式记忆确认门)也随之具备。
- **不能把"Workflow 工具的 schema 强制契约"直接当成 worker 回程契约的既定技术选型去照搬字面实现**——见下 §1.3,这是一个需要主导线明确拍板的待决策点,不是可以从参考直接推导的结论。

### 1.3 待决策点:结构化契约的严格度选型(留给 C2/C3)

现状 `worker_report.rs` 的契约走的是**"确定性文本约束 + 解析失败软着陆"**哲学:`WORKER_REPORT_CONTRACT_TEXT` 用中文提示词要求 worker 输出唯一 json 块,`parse_worker_report` 抠不到块或解析失败时**不报错、统一返回 `None`**,由 `consume_worker_report_after_completion` 的 `None` 分支生成一条 `report_warning` 文案,**不阻断链**(director_agent.rs:1140-1166 的 completed 分支照常推进)。

这与 Workflow 工具"schema 强制 + 校验失败在 tool-call 层强制重试"的严格度选型是**两种不同的容错哲学**:前者优先保证链不因为 worker 措辞问题卡死(sacrifice 完整性换可用性),后者优先保证契约完整性(sacrifice 部分可用性换数据质量)。C2(任务包 v2 转发)如果要扩展 worker 回程字段(纳入 §3.6 目前恒空的 `open_issues`/`permission_requests`/`direction_risks`/`follow_up_suggestions`),以及 C3(worker 求助通道)如果要让"求助"成为一类**必须被主管看见、不能被静默软着陆掉**的结构化信号,这个严格度选型需要主导线明确拍——继续软着陆,还是对"求助类"字段单独升级为强校验(缺失/格式错即视为需要主管介入,而不是生成一条可能被忽略的 warning)。

### 1.4 与 B2 切片的对应

| 模式 | 已落地的库内先例 | B2 哪个切片要用 |
|---|---|---|
| 上下文隔离 | worker 回程契约只读最终消息 | C1(每任务独立会话,是"上下文隔离"在会话粒度的落地) |
| 单进单出信道 | `build_goals_with_contract` / `parse_worker_report` | C2(任务包 v2 转发的输入通道扩容) |
| 编排器中转数据 | `run_director_task_chain` 唯一驱动;worker 互不直连 | C4/C5(主管总结经由 director 中转,不是 worker 互相通气) |
| 对抗式复核 | 独立复核线(Aquinas/Maxwell/Parfit 等) | C4(主管判过/退回),但现状是**单一判定者**(主管自己读口供自己判),不是"双agent互核"意义上的对抗式——若要真对抗式复核需另设独立复核角色,C4 目前的设计止步于"主管终标",这点需要主导线知悉:**C4 现有设计不构成"对抗式复核",只是完成判定** |
| 结构化契约 | worker 回程契约(软着陆哲学) | C2/C3,见 §1.3 待决策 |

---

## 2. Codex CLI 多线程/会话能力实测

### 2.1 实测边界(按红线执行)

- 只读元数据:`codex --version`、`codex --help`、`codex exec --help`、`codex resume --help`、`codex fork --help`、`codex features list`、`codex debug --help`、`codex mcp --help`。
- 真起会话:仅在固定测试项目 `/Users/yoyi/codex-workflow-mario-test`(轻档),`-s read-only` 只读沙箱,prompt 明确要求"不要读写任何东西",单次 2 轮对话(exec + resume),未涉及产线 store、未手动读写 `~/.codex` 凭据。
- 实测后核对 `git status --short`:测试项目里的改动/未跟踪文件(README.md/index.html/若干 proof.txt)均为 2026-07-05~07 之前遗留(早于本次实测时间戳),非本次实测产生——本次两次 exec 调用均为 `read-only` 沙箱 + 禁止读写指令,未在测试项目内产生新文件。

### 2.2 实测记录

**版本**:`codex-cli 0.134.0`(基线文档 2026-06-18 未记录版本号,一个月内已升级)。

**实测1 — 新起会话**(`codex exec --json -s read-only`,固定测试项目内,prompt="Reply with exactly the single word: ack..."):

```
{"type":"thread.started","thread_id":"019f420c-ef98-7f61-ad9b-e25ab901dd55"}
{"type":"item.completed","item":{"id":"item_0","type":"error","message":"`[features].codex_hooks` is deprecated. Use `[features].hooks` instead. ..."}}
{"type":"turn.started"}
2026-07-08T14:06:14Z ERROR codex_memories_write::phase2::job: failed to claim job: error returned from database: (code: 1) no such table: jobs
{"type":"item.completed","item":{"id":"item_1","type":"agent_message","text":"ack"}}
{"type":"turn.completed","usage":{"input_tokens":18709,"cached_input_tokens":3328,"output_tokens":86,"reasoning_output_tokens":79}}
```

会话文件落在:`~/.codex/sessions/2026/07/08/rollout-2026-07-08T22-06-11-019f420c-ef98-7f61-ad9b-e25ab901dd55.jsonl`(只读 `ls` 确认存在,未打开内容)。

**实测2 — resume 同一 thread_id**(`codex exec resume 019f420c-... --json`,prompt="ack2"):thread_id **原样复用**(`019f420c-...`),`input_tokens` 从 18709 涨到 37447(上下文在同一 thread 内累积)——确认 `resume` 是同一 thread 内续接,不产生新 thread_id。

**实测3 — fork 是否在非交互(exec)自动化面暴露**:`codex exec --help` 的子命令只列 `resume`/`review`/`help`,**没有 `fork`**;`codex fork` 只存在于顶层交互式命令(需要 TUI picker)。即:工作台若走 `codex exec` 自动化路径(`CodexLocalRunner` 走的正是这条路径),**够不到 `fork`**,只有 `resume` 可用。

### 2.3 对照基线 `docs/plans/2026-06-18-codex-native-conversation-behavior-baseline-v1.md` 的变化点

该基线是对 **Codex 桌面 app UX** 的观察记录,不是 CLI 专项基线,但其"会话模型"部分(新会话内联选择器、项目分组)与本次 CLI 实测的会话/thread 概念是同一底层引擎的两个界面,故按"变化点"逐条核对:

| 项 | 基线记录(2026-06-18) | 本次实测(2026-07-08,CLI 0.134.0) | 变化? |
|---|---|---|---|
| thread/会话标识 | 未专项记录 CLI 层 thread_id 格式 | `thread.started` 事件 + UUID;rollout 文件名嵌 thread_id | **新增记录**(非基线矛盾,是基线未覆盖的补充) |
| resume 行为 | 未记录 CLI resume 语义 | resume 复用同一 thread_id,上下文累积 | **新增记录** |
| fork 能力 | 基线撰写时未提及 fork 命令 | `codex fork` 存在(顶层,TUI-only),`codex exec` 自动化面不可达 | **变化点:新增命令,且明确不进自动化面** |
| `codex_hooks` 配置项 | 未涉及 | 运行时报 deprecated,新名 `hooks`(`features list` 确认 `hooks: stable, true`) | **变化点:配置项改名** |
| **`multi_agent` 特性开关** | 未记录 | `features list` 显示 `multi_agent: stable, true`(默认开启);另有 `multi_agent_v2: under development, false`、`child_agents_md: under development, true`、`enable_fanout: under development, false` | ⚠️**重大变化点,见 §2.4 专项说明** |
| **`memories` 特性开关** | 未记录 | `features list` 显示 `memories: experimental, true`(默认开启);实测中触发 `codex_memories_write::phase2::job` 内部错误(本地无 jobs 表,非本次操作导致,是 Codex 自身该 feature 的运行时状态) | ⚠️**重大变化点,见 §2.4 专项说明** |
| 消息渲染(平铺文本+工具行)、运行态药丸、目标条 | 基线 P3/P4 记录详尽 | 本次实测走 `--json`,未观察交互式 TUI 渲染,**未复核此项** | 未核·如需复核需另跑交互式 `codex`(非 exec)会话观察 TUI |
| 撰写区新会话内联选择器(项目/模式/分支) | 基线 P2 记录详尽 | 本次实测用 `-C`(cd)+ 命令行参数,未走桌面 app UI,**未复核此项** | 未核·基线本身就是桌面 app 专项记录,CLI 无对应 UI 概念,不构成矛盾 |

### 2.4 重大变化点专项说明:`multi_agent` 与 `memories` 默认开启

**`multi_agent`(stable, 默认开启)+ `child_agents_md`(under development, 默认开启)**:这意味着 Codex CLI 当前版本原生已具备某种多智能体协作能力(且 `child_agents_md` 暗示它会读取类似 `AGENTS.md` 的子智能体配置文件,与 Claude Code 的 `.claude/agents/*.md` 文件式子智能体定义是同构概念)。**本次实测未进一步验证其运行时行为**(未做真实的多智能体真实会话,因为这需要更高的实测成本、更多 token 消耗,且红线要求"只读为主"——不确定就标"未核"而非猜测其具体行为)。**怎么核**:需要专题任务,在固定测试项目里用明确要求触发子智能体的 prompt 跑一次真实 `codex exec`,观察 `--json` 输出里是否出现子智能体相关的事件类型(如 `subagent.started` 之类),并读取 `child_agents_md` 涉及的配置文件读取路径。**工作台自建层怎么用/不用**:**不用**——这正是 §5.10"不能把 Codex 的 parent/child/subagent 关系直接当成项目主管/worker/验证线/回收线关系"要挡的东西;`multi_agent` 是 Codex 自己的会话内能力,工作台的 worker 派发权仍在项目主管任务包 + 控制核心,不能因为 Codex 自带了这个能力就绕过工作台自己的派发链。**此变化点需要写进未来阶段边界:一旦 C1-C5 落地后有专题任务评估 `multi_agent`,必须明确"仅供参考,不接入生产路径,不能让 Codex 自治 spawn 替代工作台的 worker 派发"。**

**`memories`(experimental, 默认开启)**:Codex CLI 现在原生带一个"memories write"子系统(`codex_memories_write::phase2::job`),即便本地未配置对应表也会尝试写入(报错但不影响正常输出)。**工作台自建层怎么用/不用**:**不用,且需要专门风险提示**——这正是 canon(`memory-layer-design-v1.md`)反复强调的"记忆层是工作台自建的受治理能力,不能让原生工具的记忆写入绕过候选→正式确认门"的现实案例。若未来某个任务不小心让 Codex 用完整权限跑,`memories` 特性可能会在 `~/.codex` 下写入 Codex 自己的记忆数据——这**不是工作台的记忆层**,不能与工作台记忆混淆,也不需要工作台去读取/信任它。**建议阶段边界**:后续如果 H/I 阶段真的要评估 Codex 原生 `memories` 能力,必须先由主导线拍板是否值得研究,当前不因为它存在就纳入设计。

**未变化项(如实记录)**:消息渲染范式(平铺文本+灰色工具行)、运行态信息展示(步数/计时/目标条)——基线文档描述的是桌面 app,本次实测用 `--json` 走的是纯事件流,两者不在同一界面层,**不构成"没变化"的断言,而是"本次实测范围未覆盖、无法核实是否变化"**,如实标注为未核。

---

## 3. 车间模型修订注记(设计前提复述)

`decisions/2026-05-31-editable-canvas-codex-as-director-v1.md`:35"车间是常驻的……会话也是常驻资产,跟着节点走。"

按蓝图 §5("一个会话同一时间只能执行一个任务"、"主管可创建会话并分配角色")与用户 2026-07-08 口述 canon(每任务一定不同对话),`decisions/2026-07-08-phase-b2-execution-loop-final-v1.md`:18 已将此修订为:

> **"会话跟任务走"**:每任务经"先生后绑"新建会话、以任务命名;节点绑定语义保留为兼容层,C1 落地后退役。

车间模型其余部分(主管每次重起、记忆全落工作台文件层、`submit_outbox` 方向)与 B2 同向,不动——`list_team()` 把文件层 join 成视野塞回主管、主管"脑子"是工作台文件层这套机制(`decisions/2026-05-31-...`:101)不受本次修订影响。

---

## 4. §5.10 判据章(原文照抄)

以下为 `docs/workbench-system-architecture-v1.md` §5.10"Codex 多线程协作参考约束"全文逐字照抄,作为本报告及后续 C1-C5 的判据:

> ### 5.10 Codex 多线程协作参考约束
>
> Codex 当前已有类似主管线向开发线派发任务、开发线完成后回交主管线复核的多线程协作能力。它对工作台有参考价值,但只能作为协作架构模式参考,不能替代工作台自己的项目、工作流、权限、审计和记忆模型。
>
> 参考资料:
>
> - `docs/plans/2026-06-07-stage-h-i-real-codex-automation-and-multi-agent-collaboration-plan-v1.md`
>
> 可吸收为蓝图约束:
>
> - 主管线负责拆解、派发、回收和复核;开发线负责单任务执行;验证线/回收线可以作为职责分离参考。
> - 每条执行线必须有明确目标、输入、允许范围、交付物和回交协议。
> - 主管复核必须看 evidence/handoff/测试/边界,而不是只看 worker 自述。
> - 多线程协作的结果必须进入工作台自己的 `WorkerHandoff`、`ReadbackResult`、`RuntimeLog`、`AuditEvent` 和记忆候选链路。
> - 任务派发必须来自项目主管任务包和控制核心授权,不能来自 agent 自治 spawn。
>
> 明确不吸收:
>
> - 不把 Codex thread id 当成工作台永久业务主键。
> - 不把 Codex 的 parent/child/subagent 关系直接当成项目主管/worker/验证线/回收线关系。
> - 不把 Codex 线程状态当成 workflow state。
> - 不让 Codex 多线程协作绕过任务包、权限、运行日志、审计、记忆状态机或用户确认。
> - 不把 Codex-only 的 send/resume UI 设计成所有 agent 的通用 UI。
>
> 后续阶段:
>
> ```text
> H0-H7:先把 codex-local 真实自动化工作流产品化
> -> I0-I6:再抽象 WorkerAdapter / RunUnit / DispatchRequest / PermissionEnvelope / WorkerHandoff / ReadbackResult / RuntimeLog / AuditEvent
> -> planned adapters 后续独立产品化
> ```
>
> 阶段边界:
>
> - 阶段 H 只产品化 `codex-local`,不接 planned adapters 真实执行。
> - 阶段 I 先做中立协作抽象,不把 abstraction 完成说成 Claude Code/OpenClaw/OpenCode 已接入。
> - provider credential/model verification 仍需后续独立任务,不能因为 Codex 多线程参考存在而提前开放。

**已落地对照**(`evidence/2026-06-08-stage-i-i0-...md`,I0 已 `accepted`):Reference Mapping 已把 thread/thread id → opaque persistence handle、delegation prompt → `WorkerHandoff`/`PermissionEnvelope`/`TaskMemoryPacketRef`、supervisor review → `ReviewGate`/`ReviewDecision` 等逐条映射冻结;Adopt/Reject/Defer 三分表已明确"不把 Codex thread/delegation/handoff 硬编码为工作台事实模型"。**本次 C0 的 §2 实测是 I0 之后一个月的复核补丁**——确认 I0 的映射方向仍然成立,同时新发现 `multi_agent`/`memories` 两个 I0 撰写时不存在的原生特性,需要在后续 I 阶段任务(或专门追加一条阶段边界)里明确纳入"不吸收"清单,而不是自动假设 I0 的边界已经覆盖了它们。

---

## 5. 差量对照表:现状 ↔ 任务包正本(`docs/workflow-task-package-design-v1.md`)

三态标注:**已有(在哪)** / **缺(C几补)** / **语义偏(现状怎么偏·迁移建议)**。不确定处标"未核·怎么核",不下"大概"判断。

### 5.1 TaskPackage §3.4(十二项 + 完整字段清单对照)

现状实物有两层:(a)`ProjectDirectorPlannedTask`(types.rs:2450-2467,主管拆任务的输入形态)+ `ProjectDirectorTaskScope`(types.rs:2437-2448);(b)`prepare_authorized_auto_dispatch_for_index_at` 物化产出的 `task_package` artifact(json! 字面量,c4_c6_workflow_governance_entrypoints.rs:2350-2396);(c)`TaskPackage` struct 本体(types.rs:4643 起,与正本字段名几乎逐字对应)。**三者字段名不完全一致,且(c)是否被(a)/(b)实际生产/消费未查证(见迁移建议)。**

| 正本字段(§3.4) | 已有(在哪) | 缺(C几补) | 语义偏 |
|---|---|---|---|
| `task_goal` | **已有,语义偏**:planned_task 侧叫 `objective`(types.rs:2452);物化 artifact 侧叫 `brief`(c4_c6...rs:2356,值=`task.objective`);`TaskPackage` struct 侧字段名才是 `task_goal`(types.rs:4650) | — | 三层三个不同字段名指向同一份值,C2 需统一或建立明确映射表 |
| `allowed_read_scope` / `allowed_write_scope` | **已有,语义偏**:`ProjectDirectorTaskScope.allowed_read_scope/allowed_write_scope`(types.rs:2443-2444)与 `TaskPackage` struct 同名;但物化 artifact 里 key 改叫 `allowed_read`/`allowed_write`(无 `_scope` 后缀,c4_c6...rs:2370-2371) | — | 物化产物字段名与两处 struct 定义都不一致 |
| `available_skills` | **缺**(C2 补) | C2 | planned_task/scope/物化 artifact 均无此字段;无任何赋值来源 |
| `available_knowledge_refs` | **缺**(C2 补) | C2 | 同上,无任何赋值来源 |
| `available_memory_refs` | **已有,语义偏**:`TaskPackage` struct 有此字段(types.rs:4655,`Vec<String>`),实际填充逻辑在 `task_memory_injection.rs:70-74`——`artifact["available_memory_refs"] = snapshot.included_memories.iter().map(\|item\| item.memory_id...)`,只落 memory_id 字符串列表;富结构(claim/body/source_refs)另落在旁路的 `memory_packet_snapshot`(非 TaskPackage 字段本身) | — | 字段存在且有真实生产者,但走的是独立 M4 遗产链路(见 §5.8),非当前 planned_task 派发路径(director_agent.rs 完全不触碰记忆包,memory_packet_snapshot_id 恒 None) |
| `forbidden_actions` | **已有,语义偏**:物化 artifact 有此 key(c4_c6...rs:2372-2377),值是**4 条硬编码固定字符串**(不读写 `.codex`/不越权/不把汇报直接写正式记忆/触发停止条件先回报主管),不因任务而异 | — | 字段名对但内容非按任务定制;C2 若要做到正本"禁止事项"随任务不同,需要从硬编码改为可配置 |
| `acceptance_criteria` | **已有,直接透传**:`planned_task.acceptance_criteria`(types.rs:2457)→ 物化 artifact 同名 key(c4_c6...rs:2378),无转换 | — | 无 |
| `report_format` | **已有,语义偏**:`planned_task.report_format`(types.rs:2458)被两处使用——物化 artifact 里改名叫 `required_return`(c4_c6...rs:2379);同时被 `build_goals_with_contract` 拼进 `goals` 数组(worker_report.rs:35-40,连同固定契约文本) | — | `TaskPackage` struct 字段名叫 `report_format`(types.rs:4661)与物化产物的 `required_return` key 名不一致,是否有转换函数未查证(需要再核) |
| `timeout_policy` | **缺**(C2 补) | C2 | `TaskPackage` struct 定义有此字段(types.rs:4662),但 planned_task→prepare 物化全链路 grep 无任何赋值代码引用它 |
| `failure_policy` | **缺**(C2 补) | C2 | 同上,struct 有定义、无生产者 |
| `callable_tool_capabilities` | **已有,直接透传**:`ProjectDirectorTaskScope.callable_tool_capabilities`(types.rs:2445)→ 物化 artifact 同名 key(c4_c6...rs:2384) | — | 无 |
| `model_id` | **已有,语义偏**:物化 artifact 有此 key(c4_c6...rs:2388),值是**硬编码固定字符串** `"codex-local-prepared"`,非来自 planned_task/scope 任何字段 | C2(若要真按任务/harness 选模型) | 字段名对但值不可配置 |
| `harness_requirements` | **已有,语义偏**:scope 侧字段名叫 `required_checks`(types.rs:2446),物化到 artifact 时改名为 `harness_requirements`(c4_c6...rs:2385,与 `TaskPackage` struct 字段名一致但与源字段名不同) | — | 命名不一致但有真实透传路径 |
| `created_by` / `created_at` | 未核·怎么核:本次未在物化 artifact json! 字面量(2350-2396 行区间)中确认这两个 key 是否存在,`TaskPackage` struct 是否含此二字段也未读到(struct 读取截断在 `version: i64`);需另行 `grep -n "created_by\|created_at" types.rs` 定位 `TaskPackage` struct 结尾附近确认 | — | — |
| `target_session_id` / `target_role` | **已有**:`TaskPackage` struct 含 `target_session_id: Option<String>`/`target_role: Option<String>`(types.rs 附近);`ProjectDirectorTaskScope.target_role`(types.rs:2440)有直接对应;`target_session_id` 与 C1"每任务独立会话"新建会话绑定后应写入,现状 planned_task 无此字段(靠 C1 落地后补) | C1(session-follows-task 落地后回填) | — |

**迁移建议**:C2 首要任务不是"新发明字段"，是**统一(a)/(b)/(c)三层的字段命名**(task_goal/brief/objective 三选一;allowed_read_scope/allowed_read 二选一;report_format/required_return 二选一),否则新补的 `timeout_policy`/`failure_policy`/`available_skills`/`available_knowledge_refs` 会在第三套命名体系里再添一层混乱。`forbidden_actions`/`model_id` 从硬编码改按任务可配置是 C2 范围内的实质工作,不是差量表的字段有无问题。

### 5.2 WorkflowLedgerEntry §3.5(13 种 entry_type)

现状**没有专门的 WorkflowLedgerEntry 持久化表**;`entry_type` 是裸 `String`(types.rs:4674-4687,无 enum 约束),13 种值分散在**两条互不相关的路径**里派生:(a)`ledger_entry_type_from_audit`(workflow_read_model_entrypoints.rs:1280-1296,从 audit `event_type` 字符串 `contains()` 子串匹配派生);(b)`workflow_read_model.rs` 内多处直接硬编码赋值(71/106/132 行)。

| 正本 entry_type | 已有(在哪) | 缺(C几补) | 语义偏 |
|---|---|---|---|
| `task_package_created` | **已有**:`ledger_entry_type_from_audit` — `event_type.contains("task_package")` → 此值(workflow_read_model_entrypoints.rs:1281-1282) | — | 靠子串匹配,非精确枚举 |
| `subagent_started` | **已有,语义偏**:`workflow_read_model.rs:71-77` 依据 `dispatch` 的 `prompt_kind` 字段判定(非 `tool_call_summary` 则落此值) | — | 派生自 dispatch 记录,非 director_agent.rs 直接产出;完整条件表达式未逐字读完(标"未核·需读 60-100 行确认") |
| `permission_requested`/`permission_granted`/`permission_denied` | **已有**:`workflow_read_model.rs:131-136` 按 `status`(approved/rejected/其它)match 派生 | — | 派生逻辑,非精确落库时即赋 entry_type |
| `tool_call_summary` | **已有**:同 `subagent_started` 判定分支(`prompt_kind == "tool_call_summary"`) | — | 同上 |
| `subagent_report` | **已有,严重语义偏(见下 §5.3 专项)** | — | 字段名存在于 `SubagentReport` 强类型结构体(而非 entry_type 字符串枚举项),另有 `entry_type=="subagent_report"` 字符串用法见 `lib.rs:1097 BlackboardEntryKind::SubagentReport` |
| `review_result` | **已有**:`workflow_read_model.rs:106` 硬编码赋值 `entry_type: "review_result".to_string()` | — | 硬编码,所在函数上下文/是否与 director_agent.rs 有调用关系未核 |
| `node_returned`/`node_failed`/`node_passed` | **已有**:`ledger_entry_type_from_audit` 内 `contains("returned")`/`contains("failed")`/`contains("passed")||contains("accepted")` 三支(workflow_read_model_entrypoints.rs:1287-1292) | — | 子串匹配;链驱动实际写入的 `event_type` 字符串(如 `"workflow_chain_node_skipped"`)**不含**这些关键字,会原样透传而非落进这三个词表值——**即链驱动(`workflow_chain_controller.rs`)产生的审计事件目前不会被这条归一化函数正确分类**,是需要 C5 处理的真实语义偏 |
| `director_summary` | **已有**:`contains("director_review")||contains("director_summary")`(workflow_read_model_entrypoints.rs:1285-1286) | — | 链驱动目前没有产生任何 `event_type` 含"director_review"或"director_summary"字样的审计事件(见 §5.6 完成判定小节),此分支目前无实际输入命中 |
| `user_decision` | **已有**:`contains("permission_decision")` 派生(workflow_read_model_entrypoints.rs:1283-1284) | — | 与 `workflow_audit.rs` 的 `workflow_permission_decision_recorded` 函数产出的 `event_type="workflow_permission_decision_recorded"` 恰好 `contains("permission_decision")`,构成一条隐式但确实存在的映射关系 |

**链驱动实际写入的 event_type**(`workflow_chain_controller.rs:239-264 append_chain_audit`,由 director_agent.rs 各分支调用):`workflow_chain_node_skipped`/`workflow_chain_node_started`/`workflow_chain_node_completed`/`workflow_chain_node_failed`/`workflow_chain_run_failed`/`workflow_chain_run_stopped`——**均非正本 13 词表原词**,是链自定义命名空间。除 `_failed` 会被 `ledger_entry_type_from_audit` 归一化成 `node_failed` 外,其余(`skipped`/`started`/`completed`/`run_stopped`)会原样透传成 entry_type,**不在 13 词表内**。

**迁移建议**:C5(闭环上脸+审计)需要做两件事——① 把 `workflow_chain_controller.rs` 的 `event_type` 命名向 13 词表靠拢(或在 `ledger_entry_type_from_audit` 里补上对链自定义命名空间的映射分支);② 把 `entry_type` 从裸 `String` 升级为有限枚举(哪怕只是运行时校验),否则"13 种"这个正本约束目前无法被代码强制。

### 5.3 SubagentReport §3.6(专项——发现两条互不相通的并行实现)

**这是本次差量核查里最重要的一条发现,原文重点标注。**

正本要求 `SubagentReport` 必须包含:执行了什么/改了什么/证据/问题/是否需要更多权限或资料/是否认为方向可能错误/后续建议/是否满足验收标准(§3.6),字段草案含 `direction_risk`、`permission_requests` 等求助类字段。

现状代码里,**名字相同、语义来源完全不同的两条实现并存**:

**实现 A —— `worker_report.rs` 契约链(真正驱动链的那条,d2dba24/368f126 落地)**:`WorkerStructuredReportInput`(types.rs:2658-2677,18 字段,含 `open_issues`/`permission_requests`/`direction_risks`/`follow_up_suggestions`)由 `build_report_input`(worker_report.rs:155-233)从仅 4 字段的 `WorkerReport{did,outputs,status,evidence}` 映射而来——**这四个求助类字段被硬编码为 `Vec::new()`(永远是空数组)**,因为 `WorkerReport` 里根本没有对应的源字段可填。也就是说:**worker 目前的回程契约完全无法表达"我需要更多权限""我认为方向可能错误"这类求助信号**——这正是 C3(worker 求助通道)要补的口子。

**实现 B —— `derive_subagent_reports`(workflow_read_model_entrypoints.rs:903 起,读模型派生函数)**:直接产出正本的 `SubagentReport` struct(types.rs:4690,14 字段,字段名与正本几乎逐字一致),但数据源是 `node_dispatches`/`audit_events`/`permission_requests` 三个数组的**通用字段**,不读取 `WorkerReport`/`WorkerStructuredReportInput` 任何一处:
- `open_issues: warnings.clone()`(直接搬 dispatch 的 `warnings` 数组,warnings 语义是"结构性告警",不是 worker 自述的问题)
- `direction_risks`:对 `warnings` 做 `contains("direction")||contains("risk")` 的字符串子串过滤(worker_read_model_entrypoints.rs,片段核实)——**这是从通用告警文本里猜"看起来像方向风险"的启发式,不是 worker 结构化自报的方向风险**
- `permission_requests`:从独立的 `permission_requests` 数组按 `work_item_id` 关联而来(与专门的权限请求流程相关,不是 worker 口供的一部分)

**结论**:正本 §3.6 的 `direction_risk`/`permission_requests` 这两个"求助"核心字段,在实现 A(真正驱动链的路径)里**恒空**;在实现 B(读模型)里**存在但语义来源与"worker 自述求助"无关**(是从通用告警文本启发式猜测,或从另一套权限流程关联而来)。**两条实现互不调用、互不感知对方存在**,若 C3 不特别注意,很容易误以为"正本字段已经有对应实现了"而漏掉真正要建的东西:**一条让 worker 在回程契约里能真实表达"缺权限/缺资料/方向可能错"的结构化通道,并让这个信号被主管看见、不被链自动软着陆掉**。

| 正本字段 | 已有(在哪) | 缺(C几补) | 语义偏 |
|---|---|---|---|
| `completion_claim` | **缺**,全仓 grep 无命中 | C3(若要) | 语义最接近 `WorkerStructuredReportInput.acceptance_status`(白名单 4 值,归一化后)或 `WorkerReport.status`(done/partial/failed 原值),但字段名/颗粒度均不同 |
| `changes_summary` | **缺**,全仓 grep 无命中 | C3(若要) | 语义最接近 `changed_what`(`WorkerReport.outputs` join 而来的单字符串),非独立 summary 字段 |
| `evidence_refs` | **已有**:字段名一致,`WorkerStructuredReportInput.evidence_refs`,必填非空校验(c4_c6...rs:1514-1516),源自 `WorkerReport.evidence` clone | — | — |
| `open_issues` | **语义偏(见上专项)**:实现A恒空,实现B有值但来自通用 warnings | C3 需打通 | 见上 |
| `permission_requests` | **语义偏(见上专项)**:实现A恒空,实现B有值但来自独立权限流程 | C3 需打通 | 见上 |
| `direction_risk` | **语义偏(见上专项),且注意单复数**:`WorkerStructuredReportInput.direction_risks`(实现A,恒空)vs task_package artifact 上另一个独立的 `unresolved_direction_risk: bool`(见 workflow_read_model_entrypoints.rs:1240,驱动 `WorkflowException` 生成)——**这是第三条独立机制**,其写入侧未查证(未核·需 grep `unresolved_direction_risk` 的写入侧) | C3 | 三条机制(A恒空/B启发式/C独立bool)同时存在,C3 设计前必须先决定"方向风险"最终该走哪一条,不能三条并存 |
| `follow_up_suggestions` | **缺(恒空)**:实现A硬编码 `Vec::new()` | C3(若要) | 无源字段 |
| `acceptance_status` | **已有,有真实映射**:`WorkerReport.status`(done/partial/failed)→ 归一化(done→reported_completed,partial→needs_rework,failed 或其它→reported_not_completed),白名单校验含第四值 `blocked`**但该值在此契约链路里从未被产出**(match 分支只产出前三种) | — | `blocked` 是校验白名单里的"死值"——语义上应对应"worker 遇到阻塞需要求助",但当前无法从 `WorkerReport` 4 字段推导出这个状态,同样是 C3 要补的口子 |

### 5.4 ReviewResult §3.7

现状 struct(types.rs:4708-4722,13 字段:`review_id`/`workflow_id`/`workflow_node_id`/`reviewer_role`/`report_id`/`accepted_fact_ids`/`observation_ids`/`result`/`summary`/`evidence_refs`/`requires_director_confirmation`/`can_complete_node`/`warnings`)**字段名与正本 §3.7(5 字段:`review_result_id`/`workflow_node_id`/`reviewer_session_id`/`status`/`findings`/`evidence_refs`/`return_reason`/`created_at`)命名不完全对应**(如 `result` vs `status`、`reviewer_role` vs `reviewer_session_id`、无 `return_reason` 但多出 `accepted_fact_ids`/`observation_ids`/`requires_director_confirmation`/`can_complete_node` 这类现状自行扩展的治理字段)。**未核·怎么核**:本次未追溯这个 struct 的实际生产者(是否有真实调用点产生 `ReviewResult` 实例,还是与 `entry_type="review_result"` 的硬编码赋值处同源)——需要专门 grep `ReviewResult {` 的构造点。B2 决策已明确"审查智能体"是§4.6 可选位、本阶段后置,故本条差量**不阻塞 C1-C5**,留给审查智能体后置任务时处理。

### 5.5 WorkflowException §3.8

现状 struct(types.rs:4725-4732,7 字段:`exception_id`/`workflow_id`/`workflow_node_id`/`exception_type`/`summary`/`status`/`warnings`)与正本(6 字段:`exception_id`/`workflow_id`/`workflow_node_id`/`exception_type`/`summary`/`severity`/`suggested_action`/`created_at`)**字段名部分不同**(现状有 `status` 正本没有,正本有 `severity`/`suggested_action`/`created_at` 现状没有)。已知触发点:`unresolved_direction_risk` bool 驱动其生成(workflow_read_model_entrypoints.rs:1240,见 §5.3)。**未核·怎么核**:本次未追溯 `severity`/`suggested_action` 语义在现状里是否有替代表达(如塞进 `summary` 文本);未追溯其余触发条件(子智能体超时/审查反复不通过/权限长期等待/harness 阻断)现状是否都能生成对应 `WorkflowException`。这部分与 C3(worker 求助通道)有交集但非直接依赖,建议 C3 落地后单独核一次。

### 5.6 工作流生命周期 §4.3–4.8

| 正本阶段 | 现状 | 缺/偏 |
|---|---|---|
| §4.3 派发任务包(项目主管选节点→系统生成草稿→检查→确认→派发→账本记录→审计记录) | **部分已有**:`prepare_authorized_auto_dispatch_for_index_at` 覆盖"生成草稿→guard 检查→写 work_item/task_package artifact→建 dispatch 记录"这条链(c4_c6...rs:51-338);审计记录见 §5.2 | "项目主管确认"这一步现状是**自动通过 guard 就地物化**,不是一个显式的"项目主管点确认"交互动作(未核·需查前端是否有独立确认 UI 步骤介于 prepare 和真派发之间) |
| §4.4 子智能体执行(只按任务包执行/请求权限找主管/反馈方向错误/不能自标完成) | **部分已有**:worker 通过契约回程,由 director 消费(不能自标完成——链驱动是 director 侧代码判定 completed/failed,worker 本身无权改变链状态);**请求权限/反馈方向错误** = 缺,见 §5.3 | C3 补 |
| §4.5 待决策(触发/流程) | **缺**:现状节点状态机没有 `waiting_decision` 的实际驱动路径(见 §5.7);`unresolved_direction_risk` 只生成 `WorkflowException`,未见其驱动节点进入等待态的代码 | C3 |
| §4.6 审查(可选/规则) | **本阶段圈外**(决策正本 §"不做"已明确后置) | — |
| §4.7 完成判定(子智能体汇报完成→审查通过→**项目主管最终标记完成**;主管必须检查七项) | **现状严重语义偏**:链驱动逻辑是"worker 契约解析成功即视为该任务 `completed`"(director_agent.rs:1159-1166 completed 分支),**没有一个独立于链驱动之外的"主管读七项、点终标"动作**;`report_status` 黄牌(368f126)只是"呈现不驱动"(前端展示提醒,不改变链状态、不阻断)——即现状是**链自动完成 = 事实上的完成判定**,不存在正本要求的"项目主管最终标记完成"这一独立治理动作 | **C4 核心缺口**,这是 C4"主管总结+终标"要补的最大的一块 |
| §4.8 工作流结束(主管总结→账本保留→记忆候选→异常/待办→关键动作审计) | **缺**:现状链跑完后没有生成"主管总结"这个产物,也没有触发记忆候选生成的代码路径(director_agent.rs 全文 grep 记忆相关调用为 0,见 §5.8) | C4 |

### 5.7 节点状态机 §5.2

正本 14 态(`not_started`/`waiting`/`running`/`waiting_permission`/`waiting_decision`/`reviewing`/`passed`/`returned`/`failed`/`skipped`/`paused` 等)+ 硬规则"`failed` 后由项目主管选择重试、退回、换会话或结束"。

现状(`workflow_chain_controller.rs` 的 `set_chain_node_state`):节点内存态只在 `"skipped"`/`"running"`/`"completed"`/`"failed"` 间流转(director_agent.rs 实测行为),**没有 `waiting_permission`/`waiting_decision`/`reviewing`/`returned`/`paused` 这几个正本态在链驱动路径里的实际使用**。

**"`failed` 后主管四选一"专项核查**:director_agent.rs:1168-1224 的失败分支是**"失败即停"**(代码注释原文:"护栏·不自动重试/不跳过,防在老失败任务上打转")——`finalize_chain_run` 直接把整条链标记为 `failed` 并终结,**没有出现"重试/退回/换会话/结束"四选一的分支逻辑或交互调用点**。另发现两个**未接线**的相关声明:`workflow_transition_allowed`/`workflow_node_transition_allowed`(workflow_read_model_entrypoints.rs:1374-1401)声明了 `failed→running` 需要 `explicit_retry_or_reopen` 标志 + 项目主管角色,但**全仓库生产代码路径无任何调用点**(仅测试调用),是尚未被接入实际驱动流程的规则声明;`NODE_ALLOWED_TRANSITIONS` 静态表(workflow_read_model_entrypoints.rs:1357-1372)里**没有任何 `("failed", ...)` 出边**;`WORKFLOW_ALLOWED_TRANSITIONS` 静态表里 `("failed","archived")` 存在(对应"结束/归档"),但"重试""退回""换会话"三支在现状代码(含静态表和函数特判)里均无对应值。

**结论**:§5.2 硬规则"failed 后主管四选一"是 **C1/C4 的核心待建能力**,现状是"失败即停"的单一出边(等价于四选一里的"结束"),重试/退回/换会话三项**目前在生产代码路径完全不存在**,只有名字相关但未接线的规则声明(`workflow_transition_allowed` 等)可以作为设计起点,不能算"已有"。

### 5.8 M4 遗产 `TaskMemoryPacketBuilder` 可用性实答

**结论:不能直接用于 C2 的"前序任务口供摘要"注入,是架构性拒绝,不是缺配置。**

因由(逐条):

1. `TaskMemoryPacketBuilder::build_preview` 只读取记忆五层 store 的其中三个(`formal_memory_store`/`memory_candidate_store`/`observation_store`)+ `memory_lint_store`/`memory_entity_relation_store`(task_memory_packet_builder.rs:18-24),**不读取、也没有任何输入参数入口可以传入 worker 口供文本或 `WorkerReport` 结构**(`TaskMemoryPacketBuildInput`,types.rs:4437-4453,`task_goal` 是自由文本仅用于关键词相关性打分,不是结构化口供输入通道)。
2. `included_memories` 只能来自 `FormalMemoryStoreV1.records`(正式记忆,task_memory_packet_builder.rs:39)——**这是 M4 evidence/handoff 明文划的红线**:evidence 原句"candidate、observation 只进入 excluded/review materials,并带明确 reason";handoff 原句"M5 之前不要把 M4 预览解释成任务包注入,也不要让 candidate、observation、knowledge hit 或 LLM summary 进入正式 included list"。**worker 口供摘要不是正式记忆(不是走过候选→正式确认门的 `MemoryRecord`),若要塞进 included_memories 必须先走候选→正式记忆的采纳流程,不能被 Builder 直接接受**——这正是记忆层 canon(受控采纳门)对 C2 的硬约束,不是 M4 本身功能不够。
3. 调用现状:`build_preview` 目前唯一的生产调用点是 `workflow_state_lifecycle_task_package.rs:734-750` 的 v0/v1 任务包 markdown 文件生成路径(经 `preview_task_memory_packet_at` 桥接),这条路径**不是** B2·C1/C2 要建的"每任务独立会话+任务包 v2 转发"新路径;`director_agent.rs` 对记忆包的唯一接触是把 `memory_packet_snapshot_id` 显式设为 `None`(即当前 director 派发路径完全不带记忆包)。**未核·怎么核**:M4 evidence 原文承诺"预览不写 store、不推进 workflow state",但现状调用点确实会写任务包 markdown 文件+审计事件,这条能力升级发生在哪次任务包之下未查证,需要按日期序翻 `evidence/`/`handoffs/`(2026-06-04 之后)确认,避免误判为"违反 M4 边界的野生调用"。

**给 C2 的建议**:`available_memory_refs` 字段(纯 `Vec<String>` memory_id 列表)可以继续复用 Builder 产出;但"前序任务口供摘要"必须走一条**独立的新通道**(不是记忆包),或者先把口供摘要走候选→正式确认门采纳成正式记忆后,再让 Builder 按常规路径召回——后者会引入"每个任务口供都要过一次记忆确认门"的治理成本,需要主导线判断是否值得,C2 设计时应把这个成本摆出来一起拍。

### 5.9 迁移次序建议(按 C1–C5)

- **C1(每任务独立会话)**:落 `target_session_id`(§3.4)真实赋值;节点状态机补 `waiting`/`running` 之外的真实态迁移(为 C4 的"重试/换会话"打地基,见 §5.7)。
- **C2(任务包 v2 转发)**:优先统一 §5.1 三层字段命名分裂;补 `timeout_policy`/`failure_policy`/`available_skills`/`available_knowledge_refs`;`forbidden_actions`/`model_id` 从硬编码改可配置;"前序任务口供摘要"走独立新通道(不复用 M4 记忆包,见 §5.8)。
- **C3(worker 求助通道)**:核心是打通 §5.3 专项发现的"三条并存但互不相通"的求助信号(`WorkerStructuredReportInput` 恒空四字段 / `derive_subagent_reports` 启发式 / `unresolved_direction_risk` 独立 bool)——建议先由主导线拍板这三条最终收敛成哪一条,而不是三条都保留;`acceptance_status` 白名单里死值 `blocked` 一并激活。
- **C4(主管总结+终标)**:核心是补 §5.6/§5.7 发现的两大缺口——① 一个独立于链自动 `completed` 之外的"主管七查+终标"动作;② `failed` 后"重试/退回/换会话/结束"四选一(现状只有"结束"一条路,且相关规则声明 `workflow_transition_allowed` 等已声明但未接线,可作为起点)。
- **C5(闭环上脸+审计)**:把链驱动自定义的 `event_type` 命名空间(`workflow_chain_node_*`)向正本 13 词表对齐或建立完整映射;`entry_type` 从裸 `String` 升级为有限枚举(至少运行时校验)。

---

## 6. 验收自证

- 报告体例三章齐(可吸收/明确不吸收/阶段边界,见 §1)+ §5.10 判据章原文在(见 §4)+ 基线变化点如实(含"未核"项,见 §2.3)。
- 差量表覆盖正本五对象(§5.1-§5.5)+ 生命周期(§5.6)+ 状态机(§5.7),每行三态标注,不确定处均标"未核·怎么核"(未使用"大概/可能"类措辞)。
- M4 遗产可用性实答:**不能接**+ 三条具体因由(§5.8)。
- 全程零产线 diff:本包只新增本文件(`docs/research/2026-07-08-agent-collab-transfer-reference-for-b2-v1.md`);实测边界内(temp/固定测试项目、只读沙箱);`git status --short`(product-line 仓)在本文件写入前后核对应只显示本文件为新增,无其它改动。

---

## 附:实测原始记录索引

- Codex CLI 版本:`codex-cli 0.134.0`
- Feature flags 全量(`codex features list`,2026-07-08):详见 §2.4,重点 `multi_agent: stable/true`、`memories: experimental/true`、`child_agents_md: under development/true`、`hooks: stable/true`(取代已 deprecated 的 `codex_hooks`)。
- 实测会话 thread_id:`019f420c-ef98-7f61-ad9b-e25ab901dd55`(固定测试项目 `/Users/yoyi/codex-workflow-mario-test` 内,`-s read-only`,两轮:新起 + resume)。
