# 指导交接：交办页复用 Syn 对话模块与 MCP 能力层收敛 v1

- 日期：2026-07-22
- 性质：架构纠偏与后续实施指导
- 当前状态：只读核对完成，待先更新权威口径，再另包实施
- 本文不授权：改代码、启动真实 App、操作真实 store、发送真实消息、落卡、stage 或 commit

## 1. 一句话裁决

交办页不应继续自建一条 resident / 私有 `CODEX_HOME` 对话运输链。正确方向是：**复用智能体页已经完成的 Syn 对话模块作为自然对话底座；MCP 作为整个 Syn 的结构化能力层，向 Codex 和后续其他 agent 提供按角色授权的工具；交办页只补项目主管语境、只读权限、方案落卡和方案列表联动。**

在这项纠偏完成前，暂停继续执行 R4G 或继续修补 `preflight_home`、generation、rotate、private-home 等旧路线。

## 2. 用户已经确认的产品原则

1. 对话框只负责自然对话，体验应像普通聊天。
2. 用户明确说“出方案”，或者主管在对话中判断已经可以出方案时，才通过 MCP 提交结构化方案。
3. 方案进入方案卡；不能把普通聊天内容自动当成方案，也不能因为没有落卡就判定聊天失败。
4. MCP 是整个 Syn 的能力，不是交办页私有的 `submit_proposal` 通道。
5. Syn 负责让信息在 Codex、其他 agent、项目事实和用户界面之间自然流转；用户不应手工搬运消息。
6. 自然对话和结构化动作必须分层：工具失败不能吞掉已经完成的对话回复。
7. 只有用户在方案卡上明确批准后，才进入任务执行链；聊天或落一张 Pending 卡都不等于批准执行。

这些原则已经能在现有方向文档中找到一致表述：

- `docs/plans/2026-07-16-master-execution-plan-conversation-first-v1.md`
- `docs/plans/2026-07-16-conversation-first-direction-and-execution-plan-v1.md`
- `decisions/2026-07-18-conversation-substrate-correction-freeform-supervisor-plus-tools-v1.md`

## 3. 已核实的当前事实

### 3.1 智能体页已经有可复用的真实对话模块

现有实现已经覆盖以下能力：

- 选择已有 Codex session 后发送消息；
- 收取并显示 assistant 回复；
- 同一 thread 连续发送第二句；
- 新建 session，取得真实 `thread.started` 后继续会话；
- optimistic user message、事件解析、轮询、停止和输入解锁；
- session 列表、transcript 与聊天 composer；
- 将 Codex `ThreadEvent` 映射成前端会话消息。

主要入口：

- `prototypes/productized-desktop-shell/src/views/agent/AgentConversationShell.tsx`
- `prototypes/productized-desktop-shell/src/views/agent/AgentChatComposer.tsx`
- `prototypes/productized-desktop-shell/src/lib/conversationEngine.ts`
- `prototypes/productized-desktop-shell/src-tauri/src/manual_relay.rs`

既有验收和实现证据：

- `docs/sprint-contract.md`
- `docs/evidence/2026-06-18-conversation-module-native-p1-event-reply-loop-v1.md`
- `docs/evidence/2026-06-18-conversation-module-native-p2-new-session-v1.md`
- `docs/evidence/2026-06-18-conversation-module-native-p3-native-rendering-v1.md`
- `docs/evidence/2026-06-18-conversation-module-native-p4-streaming-v1.md`

边界必须说清：现有直接发送实现当前只对 Codex 开放，且 `manual_relay` 的既有校验要求 `workspace-write`。因此它可以作为复用底座，但不能原样把权限配置搬到主管会话，更不能宣称已经支持其他 agent。

### 3.2 交办页当前没有复用该模块

交办页目前走的是另一条专用链：

`ProjectJiaobanPanel` → `useJiaobanConversationState` → `submit_supervisor_resident_answer` → resident one-shot/private home/generation

主要入口：

- `prototypes/productized-desktop-shell/src/views/projects/jiaoban/ProjectJiaobanPanel.tsx`
- `prototypes/productized-desktop-shell/src/views/projects/jiaoban/useJiaobanConversationState.ts`
- `prototypes/productized-desktop-shell/src-tauri/src/supervisor_resident_oneshot_session.rs`

这解释了为什么最近工作长期卡在 private-home、resume、generation、config drift、preflight 和真实进程残留：执行一直在加固一条交办页专用运输链，而不是复用已经存在的对话底座。

### 3.3 MCP 已经是多能力编排面，不只有方案提交

现有 supervisor orchestrator 已包含或暴露过以下结构化动作：

- `dispatch_worker`
- `inspect_worker`
- `follow_up_worker`
- `wait_worker` / `wait_for_worker`
- `finalize`
- `report_user`
- `submit_proposal`

入口：

- `prototypes/productized-desktop-shell/src-tauri/src/mcp/supervisor_orchestrator.rs`

因此后续不应再把 MCP 定义成“交办页方案卡接口”。它应该被定义成 Syn 的统一能力注册、授权、调用和回传层。

## 4. 正确的目标架构

```text
交办页 UI
  ├─ 自然对话 ──> 共享 Conversation Transport / Session Adapter
  │                  ├─ Codex adapter（当前先落地）
  │                  └─ 其他 agent adapter（后续增量接入）
  │
  ├─ 主管语境 ──> project_id / workflow_id / supervisor role / read-only profile
  │
  └─ 方案列表 <── proposal read model / refresh

共享对话线程中的 agent
  └─ 按角色调用 Syn MCP capability plane
       ├─ submit_proposal
       ├─ dispatch / inspect / follow_up / wait
       ├─ report / finalize
       └─ 后续知识、记忆、事实等 Syn 能力

Syn canonical core
  ├─ 镜像已确认的对话事实
  ├─ 保存结构化工具结果、审计和幂等事实
  └─ 不阻断自然对话运输
```

核心边界：

- 自由文本由对话 transport 承载，不由 MCP 充当聊天运输层。
- MCP 承载结构化动作、结构化结果和能力授权。
- 工具结果回到发起调用的同一会话线程，同时更新对应的 Syn read model。
- canonical 记录是事实镜像与审计面，不应成为发送聊天前的阻断式前置条件。
- 交办页是同一对话能力在“项目主管”角色下的产品视图，不是另一套聊天内核。

## 5. 能力复用矩阵

| 能力 | 当前已有 | 交办页所需动作 |
| --- | --- | --- |
| 已有 session 发送与回复 | 有，Codex | 复用 transport，不复制整页组件 |
| 新 session 与后续续接 | 有，Codex | 绑定项目/工作流主管 session |
| transcript / composer / live event | 有 | 抽出可复用接口或共享 hook，交办页保留自己的布局 |
| stop / polling / input unlock | 有 | 直接继承，补交办页状态映射 |
| 主管只读权限 | 没有；现有 direct send 偏 `workspace-write` | 新增明确的 supervisor read-only profile |
| MCP 能力注入 | 已有 orchestrator 工具，但未形成共享对话 profile | 按角色注入最小工具集，首批包含 `submit_proposal` |
| 项目/工作流上下文 | 交办专用链里有部分事实 | 作为 session binding 注入共享 transport |
| canonical 对话事实 | 有现成记录面 | 改成确认后镜像；写失败不吞自然回复 |
| Pending 方案卡刷新 | handler/store 已有 | 工具成功后刷新对应方案列表 |
| 其他 agent 对话 | 目标已明确，当前直接发送未实现 | 先定义 adapter 合同，后续逐类接入，不虚报完成 |

## 6. 应保留、应改造、应暂停的资产

### 6.1 保留

- 交办页现有视觉结构和交互语义；
- 方案卡、Pending 状态、批准后启动 chain 的产品规则；
- `submit_proposal` handler、服务端幂等键和持久化能力；
- M5 DB-primary / JSON 投影和已经完成的存储修复；
- canonical 审计/read model 中不阻断业务的部分；
- 智能体页的 conversation transport、event mapping、session continuation 和 composer 行为。

### 6.2 改造

- 将智能体页的“页面组件”与“对话 transport/session state”分开；优先复用逻辑层，不把整张 `AgentConversationShell` 塞进交办页。
- 让 transport 接受显式 profile：agent adapter、sandbox、MCP capability set、项目/工作流 binding。
- 新增 `supervisor-read-only` profile；不得沿用当前 `workspace-write` 默认值。
- 让交办页使用共享 transport，同时保留交办页自己的历史/方案列表布局。
- 将结构化 MCP 调用结果同时返回原线程并刷新 Syn read model。

### 6.3 暂停，但不要在脏工作树中直接删除

- `supervisor_resident_oneshot_session` 作为交办页主对话运输；
- 交办页私有 `CODEX_HOME`、generation、archive/rotate、invalid-resume 自愈作为主路径；
- R3B、R4E、R4F、R4F-R1 围绕旧 resident 运输新增的继续诊断；
- 在新路径尚未通过替代性验收前，禁止把旧代码直接清除或把历史 evidence 改写成“无效”。

这些工作不是全部毫无价值：其中的错误分层、用户文案、幂等、M5 写路和审计安全仍可复用。需要暂停的是“把专用 resident 链继续当主架构”的结论。

## 7. 权威文档纠偏要求

当前 `AUTHORITY.md`、`CURRENT.md` 和 master plan 的执行指针仍落在 S1B-H2 / R4G resident 路线。它们与本次已确认的模块复用方向冲突。

后续第一包应只做最小语义纠偏，不能直接覆盖现有脏 hunk：

1. 新增一份正式 architecture decision，明确“共享 Conversation Transport + Syn MCP capability plane”。
2. 在 `CURRENT.md` 标记旧 resident live 路线 paused/superseded-for-primary-transport，并保留历史结果。
3. 更新 `AUTHORITY.md` 的当前执行入口，指向新的 capability audit / reuse 包。
4. 更新 master execution plan 的当前切片；保留原阶段历史，不伪造成从未执行。
5. 在 current feature inventory 中区分：智能体页已具备的对话能力、交办页当前接法、待复用缺口。

如果上述文件仍有未归属脏改，接手人必须先冻结 hash、确认所有者，再做最小合并；不允许另起一份并列 CURRENT 规避冲突。

## 8. 建议实施顺序

### 阶段 A：能力核对与接口冻结

只读核对以下真实调用链，形成一张 capability matrix：

1. `AgentConversationShell` 的 existing/new session 入口；
2. `conversationEngine` 的消息与事件转换；
3. `manual_relay` 的发送、续接、停止、轮询和 sandbox 校验；
4. 交办页当前输入、消息列表、错误态、项目/工作流 binding；
5. MCP server 配置如何进入 Codex 会话；
6. 工具结果如何回到同一 thread，以及 proposal read model 如何刷新。

阶段 A 的输出必须先回答“可直接复用、需参数化、确实缺失”三类，不得一开始重写 transport。

### 阶段 B：抽取共享对话 transport

- 抽取或参数化现有发送/session 状态逻辑；
- 智能体页行为保持不变；
- 交办页只接共享逻辑，不复制一份 `manual_relay` 或另建 sidecar；
- 用离线测试证明 existing session、new session、second send、stop 行为未回归。

### 阶段 C：主管 profile 与 MCP 能力绑定

- 新增显式 `supervisor-read-only` profile；
- 绑定 `project_id`、`workflow_id` 和主管角色事实；
- 为该角色授予最小 MCP 工具集，首个落地工具为 `submit_proposal`；
- 不使用 wildcard、default allow-all、full-auto 或 bypass；
- 工具失败保留自然回复，并给出独立、可理解的结构化动作失败状态。

### 阶段 D：交办页接线

- composer 与 transcript 改接共享 transport；
- 对话确认后再镜像 canonical；
- `submit_proposal` 成功后刷新方案列表；
- 方案卡继续保持 Pending，只有用户点击批准才进入 chain；
- 左侧旧历史栏后续按产品计划移到右侧方案卡左边，作为方案列表；本阶段先保证数据与选中态接口可支撑该布局，不顺手扩大视觉改版。

### 阶段 E：离线与真实 App 验收

离线全绿后，只做一次受控真实验收：

1. 首句普通需求：canonical recorded、同 thread 注入并得到自然回复；
2. 第二句明确要求出方案：同 thread 自然回复，并产生一次 `submit_proposal`；
3. 最多新增一张对应 Pending 卡；
4. 刷新后卡片仍在；
5. 未点击卡片前，chain/worker 必须零增量；
6. 第三句普通追问仍续同一 thread；
7. 工具失败夹具下，对话回复仍保留；
8. 退出后无 App/dev/Codex/MCP/store holder 残留。

### 阶段 F：其他 agent adapter

只有 Codex 路径和共享 profile 通过后，再把 transport contract 扩展到其他 agent。验收必须逐 adapter 进行；“MCP 是全局能力”不等于“所有 agent 的自然对话运输已经完成”。

## 9. 验收标准

本次纠偏只有同时满足以下条件才算完成：

- 交办页与智能体页调用同一个对话 transport/session abstraction；
- 交办页不再以 resident/private-home 链作为主运输；
- 主管会话为明确的 read-only profile；
- MCP 被建模为 Syn 全局能力层，并按角色授予最小工具集；
- 普通聊天不触发 proposal；明确出方案时最多落一张 Pending 卡；
- 工具失败不会把自然对话改写成“没送到主管”；
- canonical 记录失败不阻断已经成功的聊天运输；
- 未批准卡片时不启动 chain；
- 真实 App 能完成“首句聊天 → 第二句出方案 → 卡片出现 → 第三句续聊”；
- `CURRENT.md`、`AUTHORITY.md`、master plan 与 feature inventory 对同一当前路线表述一致。

## 10. 禁止事项与止损条件

- 禁止再新增一套交办页聊天 transport、私有 MCP server 或新 sidecar。
- 禁止把 `submit_proposal` 当作 MCP 的全部定义。
- 禁止为了复用而把 supervisor 放宽为 `workspace-write`。
- 禁止把 canonical CAS、审计写或 proposal 写前置成自然对话发送成功的必要条件。
- 禁止默认批准所有工具或使用模糊 wildcard。
- 禁止在没有 adapter 证据时声称“其他 agent 已经能双向通信”。
- 禁止继续用单一用户错误文案掩盖 recorded、runner、conversation、tool、card 五个不同阶段。
- 遇到权威文档 dirty overlap、真实 store holder、未归属进程或 live 权限缺失时立即止损，不得扩权绕过。

## 11. 接手人开工前必读顺序

1. `AGENTS.md`
2. `CURRENT.md`
3. `AUTHORITY.md`
4. 本交接文档
5. `docs/plans/2026-07-16-master-execution-plan-conversation-first-v1.md`
6. `docs/plans/2026-07-16-conversation-first-direction-and-execution-plan-v1.md`
7. `docs/2026-07-08-workbench-current-feature-inventory-for-prototype-v1.md`
8. `docs/plans/2026-06-18-conversation-module-native-execution-plan-v1.md`
9. `docs/evidence/2026-06-18-conversation-module-native-p1-event-reply-loop-v1.md`
10. `docs/evidence/2026-06-18-conversation-module-native-p2-new-session-v1.md`
11. 上述第 3 节列出的现有对话、交办与 MCP 代码入口

## 12. 给下一位执行者的 kickoff

> 先不要改代码、不要启动 App、不要操作真实 store。以本交接为指导，先对智能体页现有 Conversation Module、交办页当前 resident 接法和 Syn MCP 能力注入点做只读 capability audit。输出：①可直接复用能力；②必须参数化的接口；③交办页真正缺失的能力；④Codex-only 与其他 agent 的当前边界；⑤最小权威文档纠偏清单；⑥一个不复制 transport、不放宽 supervisor 权限、不新增私有 MCP/sidecar 的实施任务包。发现 `CURRENT.md`、`AUTHORITY.md` 或 master plan 的未归属 dirty overlap 时止损并报告，不得覆盖。

## 13. 本轮交付边界

本轮只新增这份指导交接文档。没有修改代码、现有权威文档、测试、真实 store 或运行态；没有 stage、commit，也没有清理既有脏工作。本文给出的是下一阶段的正确指导和执行顺序，不代表复用实现或真实 App 验收已经完成。
