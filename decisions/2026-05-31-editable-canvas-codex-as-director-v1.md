# 决策：可编辑画布 + Codex 会话当主管 v1

## 结论

工作流的下一层表达采用「车间」模型：用户在画布上摆角色节点、给每个节点挂一个 Codex 会话；其中主管节点上的会话由 LLM 担任，目标驱动、自己派活、自己收工。

v1 范围：

- 一种画布模式：A 模式（车间，常驻）。
- 主管：Codex 会话（α 路线）。
- 调度：单线，同一时刻只能有一个子 agent 在跑。
- 主从通信：双向 MCP，主管和子各挂一组工具。
- 主管脑子：每次唤醒重起一次性会话，记忆全部落工作台文件。
- 角色层级：单层主管 + 子 agent，全局/项目分层后置。

## 大白话

让用户能在画布上画一个车间。

车间里有：

- 一个主管节点。挂一个 Codex 会话当主管。它是 LLM。
- 若干子 agent 节点。每个挂一个 Codex 会话。它们也是 LLM。
- 节点之间画线，表示谁向谁汇报。

跑工作时：

- 用户给主管一个目标，按下开工。
- 主管会话起来，看车间状态，决定派给谁干啥。
- 子 agent 干完，把结果交回来。
- 主管再看一眼，决定继续派、打回、还是收工。
- 目标没完成就继续派，目标达成主管自己宣布收工。
- 用户随时能拍总闸。

车间是常驻的。今天关电脑，明天打开还在。会话也是常驻资产，跟着节点走。

不是流水线。流水线是顺序工序、跑完就停。车间是循环工作、目标驱动。流水线模式后置。

## 模式选择

### A 模式：车间（v1 采用）

- 节点 = 角色 / 人。
- 边 = 汇报关系。
- 跑一次 = 这些角色围绕一个目标协作直到收工。
- 主管手里有电闸：派活、回收、叫停、收工。

### B 模式：流水线（后置）

- 节点 = 步骤。
- 边 = 顺序依赖。
- 跑一次 = 走完一遍图。
- 没有电闸，触发即跑。

B 模式作为后置功能接入，不在 v1 范围。

## 主管路线

### α：主管 = Codex 会话（v1 采用）

- 用户给目标，主管自己派活、自己收工。
- 用户可以离开。
- 风险：跑岔了用户不在。靠总闸 + 审计 + 用户后审控住。

### β：主管 = 用户

- 画布是遥控器。
- 安全可控但不算自迭代。

### γ：可配置（α / β 混合）

- 终态目标。v1 不做。

v1 走 α，但所有主管动作必须显式（dispatch / recycle / stop / finish），这样后续切回 β 或 γ 都不需要重写底层。

## 调度模型

v1 单线。

理由：

- 自迭代第一个真实任务大概率串行可验证。
- 单线把 MCP 总机、主管循环、画布展示都跑通后，多线只是把 slot 数从 1 改 N。
- `codex exec resume` 长任务稳定性当前仍在 unfinished 列表，先不上多进程。

多线作为 v2。

## 架构

### 主管的脑子

主管不是常驻会话。每次该主管说话时，工作台干这串事：

1. 起一次新的 `codex exec` 会话当主管。
2. 把当前车间视野喂给它。
3. 它调一次 MCP 工具（dispatch / recycle / finish / stop）。
4. 工作台拦下动作，写入文件。
5. 关掉这个主管会话。
6. 等下次触发再重起一个全新主管会话。

主管的「脑子」实际上是工作台的文件层。每次主管被唤醒时，`list_team()` 把文件层 join 成一份视野塞回去。

理由：

- 长会话 transcript 越来越长，烧 token 且注意力涣散。
- `codex exec resume` 长任务稳定性是已知坑，主管不该踩。
- 一次性会话每次决策完整记录在文件层，可审、可回卷。
- 单线场景下每次决策点都是具体事件（子 agent 交活了），上下文 = 文件层够用。

### 子 agent 的脑子

子 agent 用 `codex exec resume`，连续会话。

理由：

- 子任务有明确终点，resume 风险比主管小。
- 写代码、改文件这类任务必须连续。

### MCP 总机

工作台跑一个 MCP server 进程，按调用方身份发不同工具集。识别靠 `codex exec` 启动时塞身份参数。

#### 主管侧工具

| 工具 | 作用 | 写到 |
|---|---|---|
| `list_team()` | 看车间状态。返回 canvas + state + 最近 audit join 后的视野 | — |
| `dispatch(node_id, task, scope?)` | 给某子派活 | state.busy + state.inbox + audit |
| `read_outbox(node_id)` | 看子的交付。先返回 summary，按需展开 content | — |
| `recycle(node_id, verdict, notes)` | 收回这次派活。verdict ∈ pass / changes / reject | state.busy 清空 + audit |
| `stop(node_id)` | 拍停某个子 | state + audit |
| `finish(summary)` | 宣布目标完成 | state.status=finished + audit |

#### 子侧工具

| 工具 | 作用 | 触发后工作台做啥 |
|---|---|---|
| `submit_outbox(content, summary)` | 交活 | 落地到 outbox 文件 + 唤醒主管 |
| `report_blocked(reason)` | 卡住，要主管定夺 | 写 audit + 唤醒主管 |

子侧工具刻意精简：

- 没有 `read_inbox`：inbox 在启动 prompt 里直接喂。
- 没有 `ask_director`：所有疑问统一走 `report_blocked`。
- 没有 `read_canvas` / `peek_team`：子不该知道车间结构。

`submit_outbox` 拆成 `content` + `summary`：

- `content` 是完整成果，落到 outbox 文件。
- `summary` 是给主管看的一两句，避免主管 recycle 时被全文塞满。

### 文件层

延用既有 `2026-05-28-codex-workflow-min-model.md` 的 schema 字段，落到这三个具体文件：

```
canvas/<工作台名>.json    画布定义。设计时写一次，运行时只读。
                          包含 team[]、edges[]，team 元素带 session id。

runs/<时间戳>/state.json  本次跑的状态。
                          包含 goal、status、busy、inbox、outbox 指针。

runs/<时间戳>/audit.jsonl 事件流。每次 MCP 工具调用追加一行。

runs/<时间戳>/outbox/<node>.md  子的交付文件。
```

### 会话作为资产

画布节点上挂会话 id，会话本体由工作台另外管。会话独立于画布生灭。

理由：

- 车间常驻，会话也常驻。
- 跟蓝图 §6「agent 实例 = 软件 + 会话」对齐。
- 节点删了会话还在，可以挂到别的节点；画布换了会话还能复用。

### 派发链路

主管调 `dispatch("front", "改 canvas 编辑器")` 后：

1. 工作台查 canvas，front 节点挂的是 `codex-B`。
2. 把 task 写进 `state.inbox`。
3. 拼提示词模板（角色 / 本次任务 / 工作目录 / 交付要求 / 硬边界）喂给子。
4. 起 `codex exec resume codex-B --prompt <拼好的>`。
5. 子调 `submit_outbox` 交活 → 工作台落地 → 唤醒主管。
6. 主管被唤醒 → `read_outbox` → `recycle` 或继续 `dispatch`。

提示词模板字段固定，主管只填「本次任务」一行。主管不用懂工程细节，只管派活。

## 不做（v1）

- 不做 B 模式（流水线）。后置。
- 不做多线并发。后置。
- 不做全局主管 / 项目主管分层。先单层。
- 不做 memory 层接入。主管视野只到 audit + state。
- 不做正则解析自然语言指令。直接走 MCP。
- 不做 transcript 回读。撞 CURRENT.md 硬约束。
- 不让子 agent 知道车间结构。
- 不让主管会话常驻。每次重起。
- 不在主管会话里放敏感串。

## 后置

- B 模式（流水线）。
- 多线并发调度。
- 全局主管 ↔ 项目主管分层。
- memory 层接入主管视野。
- β / γ 主管路线（用户当主管，可配置）。
- 主管动作扩展（追问 / 跨子协调 / 改目标 / 请示上级）。

### 画布参考源深入研究

ComfyUI、n8n、Langflow、React Flow / xyflow、Storybook 等节点式工具继续作为画布后置研究对象。当前先不把这些工具作为产品路线，也不把工作台改成通用节点执行器。

后置研究目标：

- 搞清节点图编辑体验：节点类型、输入输出、连线规则、模板、运行队列、历史记录。
- 搞清右侧节点面板：参数、权限边界、执行状态、输入输出、错误、审计怎么展示。
- 搞清工作流模板：四角色车间、单节点派发、验证线、回收线等常用结构如何复用。
- 搞清 AI 协助开发画布的方法：用 Storybook 固定节点状态样例，用 React Flow schema 固定节点和边，用局部 UI 草稿工具辅助面板设计。

当前可借鉴但不直接吸收：

- 借鉴 ComfyUI 的节点图、节点参数面板、运行队列、历史记录和模板复用。
- 借鉴 n8n / Langflow 的节点配置、执行记录和逐步调试。
- 借鉴 React Flow 的节点 / 边底座。
- 借鉴 Storybook 的组件状态样例，帮助 AI 迭代画布时不把状态写散。

当前明确不做：

- 不做 ComfyUI 式插件节点生态。
- 不做任意 Python / shell / API 节点执行器。
- 不做通用自动化平台。
- 不让外部工具的节点模型覆盖工作台自己的 workflow state。
- 不把画布研究提前到当前 v1 的 Codex 车间闭环之前。

触发条件：

- v1 画布能稳定保存、挂会话、开工、拍停、显示 audit / outbox。
- 项目工作流状态和 canvas / run 文件层是否合一已经有明确决策。
- 当前测试能证明画布基础交互没有退化。

## 风险

- 主管一次性会话失去「上回我犹豫过 X」这种连续性。缓解：每次决策点都是具体事件，文件层视野够用。如果实践中主管做出明显前后不一的决定，再考虑给视野加一段「主管笔记」字段。
- MCP 总机要按身份分流工具集，需要 codex exec 启动时塞身份参数 + server 端识别。这是 v1 必须解决的工程点。
- 子的 `report_blocked` 走主管同一条唤醒路径，主管被唤醒频率可能升高。缓解：保持单线 + 子任务模板里硬要求子先自己尝试再 blocked。
- 画布上节点删除时，挂着的会话怎么处理（断开 / 删除 / 留为孤儿）。v1 默认断开，会话保留。
- α 路线下主管跑岔了用户不在。缓解：总闸 + 审计 + 每次 dispatch 写入文件后用户后审。
- 如果过早深入 ComfyUI / n8n / Langflow，容易把 Codex 工作台带偏成通用节点工具。缓解：只作为后置研究；研究结论必须先转译成 Codex 车间节点、权限、审计和 workflow state，再决定是否落地。
- 如果完全不研究这些成熟节点工具，画布可能只剩能拖线的外壳，缺少队列、历史、模板和节点面板这些实际可用能力。缓解：v1 闭环稳定后单独开 spike，不和当前 v1 交付混在一起。

## 与现有决策的关系

补充并延伸：

- `2026-05-28-codex-workflow-min-model.md`：本决策的画布、节点、边、状态机、审计沿用其字段定义。本决策的「车间」对应该 schema 里的 Workflow，「主管节点」对应 `node_type=director`，「子 agent 节点」对应 `node_type=session` + `agent_type=codex`，「会话作为资产」与 ActorRole / AgentAdapter 配合。
- `2026-05-30-workflow-first-before-workbench-iteration.md`：该决策把「项目团队工作区 v1」放在工作流闭环跑通之后。本决策是该后置项的具体设计落地，触发条件（4 角色机器、real-director 自然接受、UI 替换）已达成。
- `2026-05-29-codex-session-plan-retained-workflow-first.md`：本决策延续「会话能力复用于 Agent 页、项目页、工作流节点，不做三套」的原则。画布节点挂的会话与 Agent 页 / 项目页里看到的是同一份。
- `archive/decisions/2026-05-29-ui-reference-sources.md`：该历史决策保留 React Flow、Storybook、shadcn/ui、v0、n8n、Langflow 等画布和 UI 参考源。本决策补充当前策略：参考源可以后置深入研究，但不能覆盖当前 Codex-only 边界；ComfyUI 也归入后置节点画布研究对象。

## 依据

- 用户明确路线：A 模式 → α 路线 → 单线 → MCP 双向。
- `CURRENT.md` 当前主线：单 transcript 读取、codex exec、resume、agent UI、workflow state、绑定、派发、dry-run、safe probe、director review、role 编排离线、4 角色机器 v1、mario 真实闭环、real-director 自然接受、uiwork inkwash 替换均已 accepted。
- `CURRENT.md` 硬约束：不读完整 transcript / 不写 .codex / 不绕过用户确认 / 不跑 harness / safe probe 不当真实业务自动执行。
- `principles.md`：单层先于分层 / 安全边界先于便利 / 任务包是内部协议而非产品中心 / 计划会变。
- `STAGE_PLAN.md`：当前在 Stage 3B「Codex 控制执行协议」，本决策是 Stage 3B 完成后通往「工作台自迭代」的衔接层。
