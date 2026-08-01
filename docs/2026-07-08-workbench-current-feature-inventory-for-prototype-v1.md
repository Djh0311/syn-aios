# 工作台现状功能与运行工作线盘点 v2

日期：2026-08-01

状态：**当前现状底账；不是目标架构、重构方案或执行授权。** 本文件按用户要求直接在 2026-07-08 的旧盘点中原地更新，没有另建一份互相竞争的“当前清单”。原 v1 的六面快照仍可从 Git 历史回查。

## 0. 这次盘点回答什么

这次不重新设计秘书、全局主管、项目主管的角色能力和权限。那些边界已经有决策，本文件只回答四个问题：

1. 工作台现在到底有哪些能力；
2. 用户每天从一个入口开始后，数据和动作实际怎样经过各模块；
3. 哪些地方已经有源码接线，哪些只是后端基座、旧路或产品设想；
4. 哪些断点使这些能力还没有组成一条完整的日常工作线。

证据口径：

- **源码接线**：当前工作树能找到用户入口、前端调用、Tauri 命令以及后端读写或读回；仍不等于真实桌面 App 已验收。
- **局部接线**：链条的一部分存在，但入口、范围、持久化、回写或下一跳缺失。
- **能力基座**：命令、store、类型或测试存在，但没有成为普通日常入口。
- **旧路**：为兼容或历史验收保留，不是当前默认路径。
- **未知**：静态代码和旧证据都不能回答，必须以后用桌面 App、真实 store 或真实子进程核验。

本轮只做静态盘点：没有启动桌面 App，没有发送消息，没有执行工作流，没有写真实 store，也没有把离线测试当成端到端验收。

## 1. 先说结论

工作台不是“功能太少”，而是**功能很多、纵向模块很多，但整台工作台缺少统一的运行工作线**。

当前最接近总线的东西不是某个角色，而是前端 `App.tsx`：它读取页面快照、工作流和一组 sidecar store，维护本地 `pendingAction`，再把确认动作分派到大量 Tauri 命令。首页、秘书、通知、待办、运行中和审计，多数也是把这些来源临时拼成读模型。

因此当前结构更像：

```text
多个入口
  → 各自的前端状态和命令
  → 各自的 Rust 模块 / store
  → App 再把结果重新读取、拼回界面
```

还不是：

```text
一个事项进入工作台
  → 有明确身份、范围和生命周期
  → 对话、知识、任务、执行、审计、记忆、待办和日报围绕同一事项协同
  → 所有结果都能回到来源和下一步
```

最关键的现状事实有八条：

1. 普通项目消息已经走 agent conversation transport，但它只完成“发消息—续接—回复—停止”，不会自动形成方案、工作流、知识、记忆或待办。
2. 秘书现在是只读摘要、看板和一次性“解释现状”，不是持续对话入口，也没有个人数据管理或日报服务。
3. 全局主管有批前边界意见和跑后结果意见，但只嵌在少数项目工作流里；当前前端没有独立的顶层全局主管入口。
4. 普通项目现在显示聊天专用交办页；完整的项目主管—方案—授权—执行闭环仍主要锁在固定测试项目和站 3b/4 的特殊路径。
5. 知识、记忆、工作流、审计都有较厚的后端能力，但它们主要靠手动按钮、前端刷新和多个 store 接起来，不是统一事件驱动闭环。
6. 通知、待办、运行中和“每日记忆”都是即时派生视图，没有独立生命周期、已读/归档、跨重启队列或定时调度。
7. 当前没有“日报”实体、生成规则、持久化、确认或后续动作；代码里的 `daily loop` 实际是记忆候选收件箱和少数治理事件的 best-effort 捕获。
8. 当前源码和离线测试不能证明真实桌面 App、真实 Codex 子进程、真实 store 以及整条日常工作线已经跑通。

## 2. 当前整台工作台的实际运行图

```text
首页 / 右侧栏 / 项目 / Agent / 知识 / 记忆 / 秘书看板
│
├─ 普通项目对话
│   └─ agent transport → 新建或续接 conversation_id/thread_id → reply / poll / stop
│       └─ 到此结束；不自动投影方案、工作流、知识、记忆或待办
│
├─ 特殊项目的主管工作流
│   └─ 方案 → 用户确认 → 边界复核 → 任务/工作流 → 派发/执行
│       → worker 汇报 → 项目事实确认 → 全局结果意见 → 用户结果决定
│       → 审计 + 可选记忆候选
│
├─ 知识工作
│   └─ Markdown / Canvas / 附件 → 搜索/图谱/反链 → 用户确认写入
│       └─ 可手动形成记忆候选；不是自动同步到项目事实
│
├─ 记忆工作
│   └─ capture → observation → candidate → 用户确认 → formal memory
│       └─ 生成任务包时可做 memory packet；不是每轮对话动态统一召回
│
└─ 聚合读面
    └─ page query + workflow snapshot + sidecars + runtime attention
        → 首页 / 秘书 / 通知 / 待办 / 运行中 / 审计
```

这张图说明：已经存在几条各自能走一段的纵向链，但它们没有围绕“同一个事项”形成稳定的端到端生命周期。

## 3. 逐条运行工作线底账

本节证据中的 `src/` 与 `src-tauri/` 均相对于 `prototypes/productized-desktop-shell/`；第 10 节列出完整仓库相对路径。

### 3.1 启动、首页和全局注意力

- **入口**：桌面壳启动、全局刷新、首页、右侧栏。
- **当前路径**：前端并行请求 projects、agents、running_workflows、memory、knowledge、settings 六个页面读模型，再把六份 `snapshot_slice` 合并回全局 `WorkbenchSnapshot`；工作流和候选 stores 另外读取。
- **输出**：首页“等我的事”、最近项目、记忆动态、系统状态；右侧通知、待办、运行中、审计和秘书摘要。
- **事实所有者**：页面本身没有新事实；它消费 Codex 会话索引、工作流状态、运行关注、诊断和各类 sidecar。
- **状态**：源码接线。
- **断点**：后端每次 page query 仍先构建完整 snapshot，前端再把切片拼回完整 snapshot；`page_read_model.rs` 内的旧自描述还写着 `contract_only`，与当前 `page_data_ready` 接线并存。通知和待办没有自己的 store。
- **证据**：`src/App.tsx:211-260`、`src/lib/pageReadModelRuntime.ts:20-81`、`src-tauri/src/page_read_model.rs:116-140,393-428`、`src/components/RightDetailPanel.tsx:388-563`。

### 3.2 项目发现、项目身份和项目入口

- **入口**：项目库和项目详情。
- **当前路径**：项目来自工作台 index；Codex 会话从 Codex SQLite 只读加载，再按会话 cwd / `project_root` 叠加到项目。
- **输出**：项目列表、会话数、最近更新时间、项目详情的交办/总览/工作流/交接/资源五个可见 tab。
- **事实所有者**：项目根路径是主要 join key；项目名常由路径末段派生。
- **状态**：源码接线。
- **断点**：没有完整的“新建/登记/归档项目”日常流程；首页空态会提示去项目页添加项目，但项目页没有对应动作。项目仍更像路径和会话的聚合，不是具有完整生命周期的统一对象。
- **证据**：`src-tauri/src/index_host_app_entrypoints.rs:25-134`、`src-tauri/src/codex_db.rs:289-370,414-450`、`src/views/projects/ProjectGallery.tsx:17-57`、`src/views/projects/ProjectWorkspaceShell.tsx:59-74`。

### 3.3 普通项目持续对话

- **入口**：任意非测试项目或没有 workflow 的项目交办页。
- **当前路径**：固定走 `agent-codex-workspace-write`；首次新建会话，后续按返回的 `conversation_id/thread_id` 续接；支持 poll 和 stop。
- **输出**：用户/助手对话流和运输分层错误。
- **事实所有者**：真实会话由 Codex 持有；交办页只在模块内按 `project_root` 缓存当前 session 和安全 transcript。
- **状态**：源码接线 + 离线测试；当前桌面 E2E 未验收。
- **断点**：缓存不会跨 App 重启；普通消息不建立 supervisor binding，也不触发方案、DB/JSON 工作流投影、知识、记忆或待办。项目交办和 Agent 会话中心共用同一 transport，却各自维护会话选择和前端缓存。
- **权限风险**：站 3b 的零写根约束当前只在经典 workflow-chain 授权路径中成立；通用 Agent Conversation 后端不读取 station/workflow 身份，只按传入的规范化 `project_root` 建立 `workspace-write`。当前前端排除该入口不能替代后端闸门。本结论是静态可达性检查，未做真实调用。
- **证据**：`src/views/projects/jiaoban/useJiaobanConversationState.ts:31-71,80-179`、`src/views/projects/ProjectJiaobanPanel.tsx:1211-1244`、`src-tauri/src/commands.rs:364-425`。

### 3.4 秘书

- **入口**：右侧 rail、秘书摘要和秘书看板。
- **当前路径**：`deriveSecretaryContext` 从 snapshot、workflow state、方案、主管意见、黑板候选和记忆候选派生摘要；用户可按需调用一次只读 Codex consult 解释现状。
- **输出**：待拍板列表、风险、建议、页面级跳转和一次性解释。
- **事实所有者**：秘书无自己的 store；解释只在前端会话内缓存，不写事实和审计。
- **状态**：只读聚合源码接线。
- **断点**：底部看起来像输入框的位置实际是只读按钮；没有持续对话、个人收件箱、个人资料、日历/邮件/文件接入、日报生成和跨事项跟踪。看板部分跳转只到页面，不一定定位到具体对象。
- **证据**：`src/lib/secretaryReadModel.ts:221-388`、`src/components/SecretaryBrief.tsx:5-45,90-145`、`src/components/SecretaryBoardView.tsx:10-78`、`src-tauri/src/secretary_agent.rs:1-22,176-243`。

### 3.5 全局主管

- **入口**：当前只有项目授权卡和交货面中的嵌入式意见。
- **当前路径**：从盘上的方案或执行证据生成批前边界意见、跑后结果意见，并写入全局主管 review store；意见不自动批准或改变项目状态。
- **输出**：`looks_ok / mismatch / caution` 或结果复核建议，以及用户可见摘要。
- **事实所有者**：全局主管 review store 和内嵌审计。
- **状态**：局部接线。
- **断点**：产品已经定义“全局主管是顶层入口”，但当前 `ViewKey`、主导航和 `ActiveWorkbenchView` 没有独立全局主管页面或持续会话；现有能力仍是项目内两个 advisory 钩子，不能做跨项目工作。
- **证据**：`src/lib/workbenchNavigation.ts:1-20,48-117`、`src/components/ActiveWorkbenchView.tsx:100-177`、`src/views/projects/ProjectJiaobanPanel.tsx:653-776`、`src-tauri/src/global_supervisor_agent.rs:576-595,639-668,761-884`。

### 3.6 项目主管、方案和授权

- **入口**：固定测试项目和站 3b/4 特例中的完整交办流程。
- **当前路径**：咨询/主管产生 proposal；用户确认或要求修改；记录 plan authorization；全局主管提供边界意见；通过授权检查后生成主管任务计划。
- **输出**：方案、授权记录、边界意见、任务计划和待执行状态。
- **事实所有者**：proposal store、plan authorization store、global supervisor review store，部分走 DB-primary 后再做 JSON 投影。
- **状态**：能力较厚，但普通项目入口未接入。
- **断点**：普通项目已经被切到聊天专用页，方案、批准和运行全部隐藏；这意味着当前普通项目里的“项目助手”不是已经实现的常驻项目主管。角色决策存在，日常产品闭环尚未接上。
- **证据**：`src/views/projects/ProjectJiaobanPanel.tsx:100-135,319-350,1236-1244`、`src-tauri/src/plan_authorization_store.rs`、`src-tauri/src/project_consultation_proposal_store.rs`、`src-tauri/src/director_agent.rs`。

### 3.7 任务、工作流和执行

- **入口**：项目完整工作流、交办的批准后动作、实验画布和少数隐藏/开发入口。
- **当前路径**：任务草案 → 任务包 → 运行前检查 → session binding → dispatch → product command / runner → worker report → 项目主管过程事实 → 全局最终复核 → 用户结果决定 → Stage C 摘要。
- **输出**：workflow state、任务包、dispatch、执行 attempt、报告、复核、审计和运行材料。
- **事实所有者**：工作流 DB-primary / `workflow-state.v0.json` 投影、runtime logs 和若干运行 store。
- **状态**：后端基座很厚；通用真实项目路径受限。
- **断点**：经典节点执行、工作流草案写回和 automation Phase B 仍有固定测试项目 path-lock；Phase A 有些只是 fixture/no-op；`run_workflow_machine` 已封存；offline role 是人工粘贴式闭环；Harness 面只索引能力，不负责真实运行。部分 stop/retry/restart/resume 只记录“用户确认过”，并不执行真实操作。
- **证据**：`src-tauri/src/commands.rs:5351-7095`、`src-tauri/src/workflow_execution_entrypoints.rs`、`src-tauri/src/workflow_run_dispatch_entrypoints.rs`、`src-tauri/src/real_execution_command.rs`、`src/views/projects/ProjectWorkflowCanvasView.tsx:263-407`。

### 3.8 运行关注、停止、恢复、读回和审计

- **入口**：Agent 页、右侧运行中/待办/管理、审计账本、项目运行页。
- **当前路径**：conversation attempt、session continuation、runtime attention、run queue、runtime log、workflow audit 和六类审计来源被分别读取并聚合。
- **输出**：运行中、阻塞、失败、需要用户动作、停止/继续建议、审计分页和诊断。
- **事实所有者**：conversation attempt 进程内状态、session continuation store、runtime log store、workflow state 和各模块内嵌审计。
- **状态**：局部接线。
- **断点**：没有一个统一运行实体和统一账本。审计页主分页不包含记忆、观察、候选、lint、关系、成熟模式和黑板候选的全部审计；运行日志和诊断只是平行展示。部分审计焦点不在当前页时需要手动翻页。
- **证据**：`src/components/RightDetailPanel.tsx:397-495`、`src-tauri/src/audit_ledger_read_model.rs:8-18,95-220`、`src/views/AuditLedgerView.tsx:50-120,268-330`。

### 3.9 知识工作线

- **入口**：知识库的原生知识工作区。
- **当前路径**：固定 app-data vault 中的 Markdown、JSON Canvas 和附件为真源；工作区支持目录、搜索、阅读、编辑、反链、关系图、Canvas、附件、备份/恢复和 Obsidian 打开。索引和图谱每次从文件重建。
- **输出**：知识文件、搜索/图谱读模型、引用和写入审计；用户可把知识材料提议为记忆候选。
- **事实所有者**：vault 文件；索引、图谱和反链只是可重建读模型。
- **状态**：源码接线 + 离线验证；十二项真实 App 场景未验收。
- **断点**：旧 `knowledge_vault_*` 顶层笔记接口和新的递归原生工作区并存；文件写入和 workflow 审计不是同一事务，可能出现文件已变但审计失败；`knowledge_open` 的 host 最终打开效果仍未真实验收。
- **证据**：`src/views/knowledge/NativeKnowledgeWorkspace.tsx:418-900`、`src-tauri/src/knowledge_vault.rs:1-22,860-920,960-1436`、`src-tauri/src/knowledge_index.rs:1-20,771-820,1164-1194`。

### 3.10 记忆形成、治理和召回

- **入口**：治理事件的 best-effort 捕获、知识转候选、记忆中心的人工处理、任务包生成。
- **当前路径**：capture event → observation → candidate → 用户确认 → formal memory；另有 lint、实体/关系、成熟模式和正式记忆生命周期；生成任务包时从 active formal memory 构建 memory packet。
- **输出**：候选、正式记忆、冲突/维护结果、关系、模式和任务记忆包快照。
- **事实所有者**：当前 live 真源仍是一组 JSON sidecar；候选、观察不能冒充正式记忆。
- **状态**：源码接线，但跨 store 闭环不完整。
- **断点**：多个 sidecar 顺序写不是事务；正式记忆可能先写成功、候选 adoption link 后写失败；自动捕获失败只变 warning，主治理动作仍成功；任务记忆包只在生成任务包时固化，旧包不会随记忆自动更新；普通对话没有统一的记忆捕获和召回协议。
- **证据**：`src-tauri/src/memory_capture_bus.rs:42-170`、`src-tauri/src/memory_candidate_store.rs:207-330`、`src-tauri/src/task_memory_packet_builder.rs:12-40,282-360`、`src-tauri/src/task_memory_injection.rs:13-100`。

### 3.11 通知、待办、“每日”和日报

- **入口**：首页、右侧栏、记忆中心的“每日”收件箱。
- **当前路径**：通知从读取状态、runtime attention、诊断和项目 warning 临时派生；待办从 task、权限、确认队列、运行关注和待复核任务临时拼接；“daily loop”筛选记忆候选，并在少数治理事件后 best-effort 生成候选。
- **输出**：当前界面的列表和数字。
- **事实所有者**：没有 notification/todo/daily-report store；确认弹层的 `pendingAction` 也只是 React 本地状态。
- **状态**：只读视图接线；日常管理服务未实现。
- **断点**：没有已读、归档、延迟、重复合并、跨重启、提醒调度、日报快照、日报确认或“从日报继续明天工作”的对象协议。
- **证据**：`src/components/RightDetailPanel.tsx:496-563`、`src/App.tsx:113-150`、`src/lib/memoryDailyLoop.ts:33-75`、`src-tauri/src/memory_daily_loop.rs:1-4,143-166`。静态搜索只在架构文档找到“日报”，生产源码没有通用日报实体或调度器。

### 3.12 技能、Harness、工具、模型和 adapter

- **入口**：技能、Harness、设置和开发者入口。
- **当前路径**：snapshot 展示 skills、plugins、harness candidates/resources、agent adapters、provider availability 和系统边界；MCP capability registry 为可信 supervisor binding 暴露精确 allowlist。
- **输出**：能力索引、可用性和警告。
- **事实所有者**：当前索引、配置和服务端 registry。
- **状态**：多数是只读能力目录或专用基座。
- **断点**：技能不能登记或直接打开 `SKILL.md`；Harness 页面不能运行、验证或把候选登记为可运行能力；真实 direct conversation adapter 只有 Codex；MCP 工具面只服务受信 supervisor turn，不是整台工作台的通用能力调度层。
- **证据**：`src/views/SkillsBoardView.tsx:21-120`、`src/views/HarnessBoardView.tsx:30-150`、`src-tauri/src/mcp/capability_registry.rs`、`src-tauri/src/command_registry.rs:57-231`。

### 3.13 个人数据、收件和跨项目协同

- **入口**：产品定位已经要求秘书管理个人数据、工作内容、日报和跨项目事项。
- **当前路径**：没有找到通用收件项、个人数据对象、外部数据连接、跨项目依赖或交接的生产闭环。
- **状态**：产品方向存在，当前运行能力缺失。
- **断点**：临时信息何时只是聊天、何时成为收件项；怎样转成项目/待办/知识/记忆；邮件、日历、文件和外部数据怎样进入；原始数据归谁、工作台存正文还是引用；跨项目事项如何隔离和回写——都还没有当前实现协议。

## 4. 后端能力域底账

| 能力域 | 当前真实能力 | 当前成熟度 | 主要事实 / 状态所有者 | 主要问题 |
|---|---|---|---|---|
| Host 与页面读模型 | Tauri command registry、完整 snapshot、六页 query、系统状态和审计读模型 | 源码接线 | AppState、index、workflow snapshot | 仍重复构建完整 snapshot；自描述漂移 |
| 项目与会话索引 | 读 Codex SQLite、rollout、项目 index，按 cwd 归项目 | 源码接线 | Codex SQLite + 工作台 index | 项目身份仍以路径聚合为主，缺生命周期 |
| Conversation transport | agent/supervisor 两种固定 profile，新建/续接/poll/stop、安全分层 receipt | 源码接线；当前 E2E 未验 | Codex conversation + 进程内 attempt | 普通 chat 与结构化工作无正式连接；UI 状态重复 |
| Supervisor MCP plane | trusted turn binding、capability registry、`submit_proposal` 与知识只读工具 | 局部接线 | binding DB/JSON、服务端 registry | 仅专用 supervisor turn；不是通用工作台能力面 |
| 方案、授权、复核 | proposal、plan authorization、边界复核、结果复核、用户决定 | 能力较厚 | DB-primary 部分 + JSON 投影/sidecar | 普通项目不进入这条线；多套 review/audit |
| 任务与工作流 | 草案、任务包、画布、dispatch、报告、复核、Stage C | 能力较厚 | workflow DB/JSON | 通用真实项目受 path-lock；旧路很多 |
| 执行与控制 | product command、Phase A/B、runner、stop/retry/resume 记录、runtime log | 局部 / 特例 | runtime stores、attempt registry | 一些动作只记录不执行；真实范围很窄 |
| Knowledge | Markdown/Canvas/附件、搜索、图谱、反链、恢复、open relay | 源码接线；真实 App 未验 | vault 文件 | 新旧接口并存；文件与审计非事务 |
| Memory | capture、observation、candidate、formal、lint、关系、模式、memory packet | 源码接线 | 多个 JSON sidecar | 非事务、best-effort、普通对话未接 |
| Blackboard | workflow 投影黑板 + 候选决策 store | 局部接线 | workflow snapshot + blackboard candidate sidecar | 决定后不自动生成跟进或提升正式事实 |
| Audit / Attention | workflow audit、模块内嵌 audit、runtime attention/log、分页账本 | 局部聚合 | 多个 store 和读模型 | 不是一份统一账本，覆盖不全 |
| Secretary | 确定性摘要、看板、只读一次性解释 | 只读局部能力 | 无独立 store | 不是持续助手入口，无个人数据/日报 |
| Global Supervisor | 项目内批前和跑后 advisory | 局部能力 | review store | 没有顶层入口和跨项目工作线 |
| Skills / Harness / Adapters | 目录、描述、可用性、候选资源 | 只读基座 | snapshot/config | 不能登记、运行或统一调度；只有 Codex direct adapter |

这个表只记录能力，不在这里维护“保留、提取、重写、退役”的动态裁决。用户已在 2026-08-01 完成目标工作线确认，当前 disposition 与迁移顺序见 `docs/plans/2026-08-01-syn-personal-ai-workbench-master-development-plan-v1.md`；本 inventory 继续只做现状真相源。

## 5. 当前数据和状态到底散在哪里

| 对象 | 当前主要真源 / 状态 | 重要边界 |
|---|---|---|
| Codex 会话 | Codex SQLite、rollout 文件；工作台主要只读 | 工作台不应把自己的投影冒充原始会话 |
| 项目列表 | 工作台 index + Codex 会话 cwd 聚合 | 项目根路径是主要 join key |
| 工作流事实 | SQLite 主写的已桥接部分 + `workflow-state.v0.json` 投影；JSON-only 可降级 | “数据库有表”不等于所有模块已经 DB-primary |
| 方案 / 授权 / supervisor binding | 部分 DB-primary 后做 JSON 投影 | 仍有兼容 JSON 和多套审计 |
| 记忆、观察、候选、lint、关系、模式 | 多个 JSON sidecar live store | 不是同一事务，也未全部进入统一账本 |
| 项目黑板 | 从 workflow snapshot/audit 派生；另有候选 sidecar | 黑板候选不是正式事实 |
| 知识 | 固定 vault 文件 | 搜索/图谱可重建；文件写与审计非事务 |
| Conversation attempt | Rust 进程内 map + Codex 子进程 | App 或进程重启后的 in-flight 恢复待验证 |
| 交办 conversation cache | 前端模块内 Map | App 重启即失效 |
| 确认弹层 `pendingAction` | React 本地 state | 不是持久待办 |
| 通知 / 待办 / 运行中 | 多源即时派生读模型 | 没有独立生命周期和跨重启状态 |
| 日报 | 未发现生产实体或 store | 目前只是架构设想 |

## 6. 已经形成的闭环和没有形成的闭环

### 已经能从源码看见的局部闭环

- 新建/续接一次 Codex 对话并收回 reply；
- 在受限项目里从 proposal、授权走到工作流执行与报告；
- 知识文件的创建、编辑、搜索、图谱、Canvas、附件和恢复；
- 记忆候选经用户确认后转为正式记忆；
- 页面从多个 store 重读并显示当前摘要、风险、待办和审计。

### 还没有形成的整台工作台闭环

- 一条普通对话如何可靠变成事项、项目动作、知识、待办或记忆候选；
- 秘书怎样持续接住个人信息并管理跨天、跨项目的工作；
- 全局主管怎样作为顶层入口做真正的跨项目工作；
- 普通项目里的常驻项目主管怎样调用现有方案、任务、执行和复核能力；
- 一个事项完成后怎样一次性回写项目事实、审计、知识、记忆候选、通知、日报和下一步；
- 失败或半写入时怎样跨 store 补偿并继续；
- App 重启后怎样恢复对话、未决确认、运行和日常收件；
- 所有专业模块怎样携带当前对象引用进入秘书、全局主管或项目主管会话，再把结果送回原对象。

## 7. 明确存在的重复、旧路和范围债

1. 项目交办与 Agent 中心共用 agent transport，却保留两套前端会话状态和交互壳。
2. 新原生知识工作区与旧 `knowledge_vault_*` 笔记接口并存。
3. 普通 agent transport、supervisor transport、resident/private-home 历史路和旧 manual relay 仍同时存在于代码历史层次中。
4. `run_workflow_machine`、旧 node dispatch 和旧 consultation 等命令/确认动作仍可被类型或界面引用，但真实路径已封存或退役。
5. 页面读模型名义上按页，底层仍是完整 snapshot；前端又合并回完整 snapshot。
6. 工作流、proposal、authorization、review、continuation、runtime、memory、blackboard 各有自己的 store 与审计习惯。
7. 当前结构化 Code Map 只有六个局部域，且本轮 query 因陈旧/不可读候选返回 `UNKNOWN`；不能拿它证明能力不存在或当前可用。
8. `get_project_workflow_nodes` 的后端读取忽略传入 `project_root`，主要按 `workflow_id` 取数；`list_project_workflows` 仍含 workflow id slug 的兼容归属；这些是以后重构前要核的范围债，不在本步修复。
9. Product Command Phase B 虽有真实 runner 路径，但当前静态代码没有在这一层强制反查同一事项的 PlanAuthorization 和全局边界复核记录；它与整条治理线是否真实闭合仍未知。

## 8. 当前仍未知、必须以后真机或真实数据验证

- 普通项目对话在桌面 App 中能否稳定首发、续接、停止和回读；
- App 重启后是否能从 Codex 真源恢复正确项目会话，而不是依赖前端缓存；
- 普通消息是否在所有错误和终态下都绝对不写 supervisor binding 或工作流投影；
- 知识十二项真实 App 操作和 `knowledge_open` host dispatch；
- 当前 DB-primary/JSON 降级和启动 reconcile 在这份 dirty tree 上的真实状态；
- 真实 runner、完整项目链、stop/retry/resume 在获批范围内的当前表现；
- 多个 sidecar 发生中途失败后的真实残留与补偿；
- 当前桌面界面是否仍能到达所有源码中的入口，尤其隐藏发令台和历史分支。

## 9. 本步完成边界

本文件完成的是“现在有什么、现在怎么串、现在断在哪里”的底账。

本文件本身没有做：

- 重新定义角色能力或权限；
- 决定只留一个入口或让秘书替所有入口做意图拆分；
- 把后来确认的目标交互模型写成当前实现；
- 在本 inventory 内维护能力处置或任务排期；
- 用计划或决策替代源码 / 真实 App 证据；
- 修改功能代码或做真实运行。

用户核对和目标运行模型确认已完成。当前下一步只看 Harness CURRENT：先激活 Stage 1 的合同切片，再按 master plan 进入安全 / 作用域基础；不得从本 inventory 自行发起实现。

## 10. 主要源码锚点

- 全局装配：`prototypes/productized-desktop-shell/src/App.tsx`
- 导航与入口：`prototypes/productized-desktop-shell/src/lib/workbenchNavigation.ts`、`prototypes/productized-desktop-shell/src/components/WorkbenchShell.tsx`、`prototypes/productized-desktop-shell/src/components/ActiveWorkbenchView.tsx`
- 页面读模型：`prototypes/productized-desktop-shell/src/lib/pageReadModelRuntime.ts`、`prototypes/productized-desktop-shell/src-tauri/src/page_read_model.rs`
- 项目与会话索引：`prototypes/productized-desktop-shell/src-tauri/src/index_host_app_entrypoints.rs`、`prototypes/productized-desktop-shell/src-tauri/src/codex_db.rs`
- 普通交办 transport：`prototypes/productized-desktop-shell/src/views/projects/jiaoban/useJiaobanConversationState.ts`、`prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- 项目主管与工作流：`prototypes/productized-desktop-shell/src/views/projects/ProjectJiaobanPanel.tsx`、`prototypes/productized-desktop-shell/src-tauri/src/director_agent.rs`、`prototypes/productized-desktop-shell/src-tauri/src/workflow_*`
- 全局主管：`prototypes/productized-desktop-shell/src-tauri/src/global_supervisor_agent.rs`
- 秘书：`prototypes/productized-desktop-shell/src/lib/secretaryReadModel.ts`、`prototypes/productized-desktop-shell/src-tauri/src/secretary_agent.rs`
- 知识：`prototypes/productized-desktop-shell/src/views/knowledge/NativeKnowledgeWorkspace.tsx`、`prototypes/productized-desktop-shell/src-tauri/src/knowledge_*`
- 记忆：`prototypes/productized-desktop-shell/src/views/MemoryCenterView.tsx`、`prototypes/productized-desktop-shell/src-tauri/src/memory_*`、`prototypes/productized-desktop-shell/src-tauri/src/formal_memory_*`
- 审计与注意力：`prototypes/productized-desktop-shell/src-tauri/src/audit_ledger_read_model.rs`、`prototypes/productized-desktop-shell/src-tauri/src/runtime_session_attention.rs`、`prototypes/productized-desktop-shell/src/components/RightDetailPanel.tsx`
- 持久化：`prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_*`、`prototypes/productized-desktop-shell/src-tauri/src/workflow_db_primary_wiring.rs`、各 sidecar store。
