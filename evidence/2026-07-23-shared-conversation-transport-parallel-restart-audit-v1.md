# 共享 Conversation Transport 并行恢复只读审计 v1

- 日期：2026-07-23
- 执行包：`tasks/2026-07-23-shared-conversation-transport-parallel-restart-audit-package-v1.md`
- 结论：**对话底座的离线实现与失败闭锁已有受限证据；真实 App 替代性验收仍未通过，不能重跑，直到知识 relay 安全返工稳定、经指导线验收且另获真实运行授权。**
- 本轮边界：只读核对文档、源码和测试；未运行 Cargo/npm/shape、未启动 Syn/Codex CLI/MCP/真实 App、未读取或复制真实 store/vault、未修改产品代码/测试/CURRENT/AUTHORITY/既有文档、未 stage/commit。

## 1. 审计依据与快照范围

权威链为 `AGENTS.md → CURRENT.md → AUTHORITY.md → decisions/2026-07-23-knowledge-and-conversation-parallel-workstreams-v1.md → 本任务包`。

- 【文档事实】`CURRENT.md:7-9`：共享 transport 与 binding 的离线实现已收口；07-23 真实 App 验收首句失败，后两句没有发送；七阶段临时 fixture 的离线结果不解释、也不修复真实首句失败。
- 【文档事实】`CURRENT.md:20-28`：当前唯一执行包是知识 `knowledge_open` relay 的安全返工；真实 App、真实 store、Codex CLI/MCP 与非测试真实项目仍需另包、明确授权。
- 【文档事实】并行决策规定：文档/只读审计/不重叠代码可并行；`commands.rs`、`manual_relay.rs`、`manual_relay/conversation_transport.rs`、`exec_process_registry.rs` 同时只能有一线写；同一真实 store 的运行验收不得并发（`decisions/2026-07-23-knowledge-and-conversation-parallel-workstreams-v1.md:14-20`）。

本文件的源码判断仅适用于下列静态快照。快照时刻为 `2026-07-23T18:27:34+0800`；四个共享承重文件均已有其他线脏改，故不能把其内容当作最终可构建物或真实 App 结果。

| 文件 | 审计时 Git 状态 | SHA-256 |
| --- | --- | --- |
| `src-tauri/src/exec_process_registry.rs` | `M` | `ebd31f2ceb048b401672dfc889bf41f20176b0c372f0deb3a7c0babe14aa8630` |
| `src-tauri/src/manual_relay.rs` | `M` | `409abd3f662c5b287c5c913ae7f6b67549d07cc06b4c1d7a43d6f41783c93440` |
| `src-tauri/src/commands.rs` | `M` | `8c493dcbd4fdcc633afa84b2c9fefa3f7e01c1ef15efd7d62dbb99749244e7b5` |
| `src-tauri/src/manual_relay/conversation_transport.rs` | `??` | `577f67e1171994d0126827e9c53af604a70f05f67bc40f35c5045e0f77f294fc` |

`mcp/supervisor_conversation_binding.rs` 与 `mcp/supervisor_conversation_transport_tests.rs` 在该快照也是未跟踪文件；它们只作静态审计输入，未构建或执行。

## 2. 结论分层

| 分类 | 已经成立的结论 | 不能推出的结论 |
| --- | --- | --- |
| 【源码事实，静态快照】 | 主管启动路径先构造并尝试持久化 `Starting` binding，后启动 transport；只有宿主观察到线程才会激活 binding。`commands.rs:351-425,936-958`；`mcp/supervisor_conversation_binding.rs:265-349`。 | 真实 App 已走到这些调用点、真实 DB/JSON 已写入、真实线程已出现。 |
| 【源码事实，静态快照】 | `tools/list` 与 `tools/call` 共享 Active/正确 context/非空 thread 的服务端校验；缺 binding、Starting、失配或未确认终结都会 fail-closed。`mcp/supervisor_orchestrator.rs:798-851,888-987`。 | 真实 client 实际看过工具，或工具调用已成功。 |
| 【离线测试事实，历史证据】 | 七阶段 receipt、临时 DB/JSON fixture 的 store/activate/transport/terminate 失败注入、工具面关闭与前端 allowlist 已有定向离线记录。`evidence/2026-07-23-shared-supervisor-conversation-binding-phase-semantics-and-failure-closure-rework-verification-v1.md:12-47`。 | 真实 App 首句根因已定位或已修复。 |
| 【真实 App 事实】 | 历史 Gate 0 和 build 曾完成；首句只发送一次，只有 canonical `recorded +1`，没有 durable binding、自然回复、工具发现、proposal/Pending/chain/worker；第二、三句未发。`evidence/2026-07-23-shared-conversation-transport-real-app-substitution-acceptance-v1.md:64-81,91-104`。 | 任一内部阶段、`thread.started` 时序或 MCP 服务端子因已被确认。 |
| 【推断／下一次验收要求】 | 下一次必须对同一 message/run 采集安全 `binding_stage`、durable binding、`thread.started`、Active、首个 `tools/list` 和第二句 `tools/call` 的实际顺序。 | 当前静态时序观察就是过去失败的单一根因。 |

## 3. 上次真实 App 首句停点

### 3.1 最早可证事实

【真实 App 事实】首句后，JSON 和 SQLite 仍为 `sessions=25`、`conversation_turn_binding=0`；唯一业务正增量为 canonical `recorded: 14 → 15`。`injected/reply/diagnostic`、proposal/Pending、chain/execution attempt/node dispatch 都是零增量。此事实只把故障裁决到“**本 turn 的 binding 没有形成可持久化事实**”，不是构造、store prepare、DB-primary persist、JSON projection、锁、`thread.started` 或 `tools/list` 中某一个已证实子因（真实验收 evidence: `74-81,114-130`）。

历史 Gate 0 → 最终只读对账如下；这是历史样本，不得复用为下一轮基线：

| 计数 | Gate 0 | 最终 | 增量 |
| --- | ---: | ---: | ---: |
| `recorded` | 14 | 15 | +1 |
| `injected / supervisor reply / diagnostic` | 5 / 5 / 1 | 5 / 5 / 1 | 0 / 0 / 0 |
| `proposal / Pending / decision / proposal audit` | 74 / 17 / 57 / 131 | 相同 | 0 / 0 / 0 / 0 |
| `supervisor session / audit / binding` | 25 / 263 / 0 | 相同 | 0 / 0 / 0 |
| `chain / execution attempt / node dispatch` | 40 / 164 / 404 | 相同 | 0 / 0 / 0 |
| `workflow revision / audit` | 303 / 1800 | 307 / 1804 | +4 / +4 |
| `registry revision / entries` | 1140 / 0 | 1143 / 0 | +3 / 0 |

### 3.2 七阶段返工解决了什么、没有解决什么

【离线测试事实】当前完整的失败 receipt 枚举是：

`binding_construct → binding_store_prepare → binding_persist_db → binding_project_json → binding_activate → transport_start → binding_terminate`

它是互斥失败分类，不是成功路径时序。临时 fixture 中：store prepare 失败不产生 binding；activation/transport 失败在 DB 与 JSON 都确认 `Failed` 后才报告相应阶段；termination 失败保留事实上的 `Active`，回执为 `binding_terminate`，并保持工具关闭（返工包 `:12-43`；返工验证 `:12-29`）。

【源码事实，静态快照】启动后终结必须先尝试写入并读回 `Failed`；若任一步未确认，命令改回 `binding_terminate` 并调用进程内工具闭锁（`commands.rs:553-572`；`mcp/supervisor_orchestrator.rs:35-83,1235-1265`）。

【文档事实】前序私有副本的 `25/0 → 26/1 Starting` 只证明一份获准副本可以写入 Starting；它不证明 Active、真实 transport、终结成功或真实首句修复（建立链包 `:14-18,43-47`）。真实 App、真实 store 和产品验收仍未获得该返工包的授权（返工包 `:4,45-58`）。

## 4. 当前静态时序与必须观察的 message-scoped 事实

### 4.1 源码中的应有链路

1. 【源码事实】新会话请求没有 `thread_id`；existing mode 必须先验证 host index 中的 thread 归属（`commands.rs:724-754,828-855`）。
2. 【源码事实】主管路径解析 project/workflow/context、构造 `supervisor-read-only` 的 `Starting` binding，并通过既有 DB-primary + JSON projection 尝试持久化；成功后才启动 transport（`commands.rs:351-425,490-504`；`mcp/supervisor_orchestrator.rs:1041-1149`）。
3. 【源码事实】new mode 的 `thread.started` 由子进程 stdout 的内存捕获提取，写入 transport receipt；外层 normalizer 看到 receipt 的 `thread_id` 后才尝试激活 binding（`manual_relay.rs:2565-2583`；`manual_relay/conversation_transport.rs:746-755`；`commands.rs:936-958`）。
4. 【源码事实】binding 只有 `Active`、非空 `thread_id`、一致的 project/root/workflow/run 与精确 allowlist 时才可授权 capability（`mcp/supervisor_conversation_binding.rs:309-349`）。
5. 【源码事实】`tools/list` 与 `tools/call` 都调用同一 capability access gate；list 在失败时返回空集合，call 在 handler 前拒绝（`mcp/supervisor_orchestrator.rs:798-851,892-985`）。

### 4.2 未解决的时序问题

【静态推断，未在真实 App 证实】child 在 binding 持久化后启动，而新会话的前端 poll 有固定延后；审计没有看到“child 必须等待 host 已将 `thread.started` 持久化为 Active 才可首个 `tools/list`”的同步闸。故真实 `tools/list` 有可能先于 parent poll/activation 而被 fail-closed。这是下一次应观察的候选时序，不是对历史首句失败的归因。

下一次获授权验收应使用同一脱敏 `turn_id/run_id` 关联下列只读事实；不得记录用户正文、完整 argv、grant、endpoint、stderr 或私有路径：

| 顺序 | 必须采集的事实 | 未成立时的裁决 |
| --- | --- | --- |
| S0 | Gate 0 基线、空进程/holder/registry/lock 与本轮 frozen hashes | 不启动。 |
| S1 | 首句发出一次；收到 start receipt 的安全 `binding_stage` | 若失败，按该阶段停止；不发第二句。 |
| S2 | 同一 run 的 JSON 与 SQLite：新 binding 是否两端都出现、lifecycle 是否为 `Starting` | 未出现或不一致，停止；不把它猜成某个更深子因。 |
| S3 | child 的真实 `thread.started` 是否被宿主观察，及其安全 thread correlation | 未观察到，停止；不发第二句。 |
| S4 | 同一 binding 是否在 JSON 和 SQLite 都成为 `Active`，且绑定同一 thread | 非 Active／不一致，停止；不发第二句。 |
| S5 | 同一 turn 的首次 `tools/list` 到达时间、完整**名称集合**和 S3/S4 的先后关系 | 空、缺项、额外项、身份不可对账或早于所选合同允许的 Active 点，停止。 |
| S6 | 仅第二句的 `tools/call` / `submit_proposal` 服务端 handler 与 outcome | 未到达／失败或计数不符，停止；不发第三句。 |

## 5. 工具面合同存在的当前不一致

历史真实 App 包把首句通过条件写为“首次 `tools/list` 只看到 `submit_proposal`”（`tasks/2026-07-23-shared-conversation-transport-real-app-substitution-acceptance-package-v1.md:51-58`）。但当前静态快照的 `supervisor-read-only` 精确 allowlist 是五项：`submit_proposal` 加 `knowledge_search/read/open/cite`；静态 fixture 也断言这些是唯一可见工具（`mcp/capability_registry.rs:132-180,382-466`；`mcp/supervisor_conversation_transport_tests.rs:535-616`）。

这不是已经发生的真实 App 行为，却是**下一包开工前必须由指导线明确的验收口径冲突**：

- 若沿用当前源码且不另改能力面，首句必须要求 `tools/list` **精确等于这五项**；本三句中只允许第二句调用 `submit_proposal`，不得调用任何 knowledge capability。
- 若产品验收必须保持“只见 `submit_proposal`”，则需要一个单独授权的实现／隔离决定；不得以真实 App 现场试跑替代该决定。

在该选择未写进新包前，不存在可诚实执行的“精确工具面”真实验收合同。

## 6. 与知识 relay 安全返工的重叠

| 重叠面 | 静态／任务事实 | 对对话重验的影响 |
| --- | --- | --- |
| `exec_process_registry.rs` | 知识包授权其修 registry 脱敏身份和 orphan reaper；当前 host-owned supervisor registration 写固定摘要与 raw argv hash（知识包 `:60-80`；源码 `:195-208,300-336,391-450`）。 | 同一进程登记、sidecar 与退出/回收边界；不能并行真实启动。 |
| `manual_relay.rs` | 知识包修 supervisor-only capture、spawn 前 safe-only 标记与失败清理；当前快照的 stdout/stderr 为内存捕获并 fail-closed（知识包 `:46-58`；源码 `:2475-2743,2756-2792`）。 | 直接承载真实 child、`thread.started` 捕获、raw poll/stop 闭锁和清理。 |
| `manual_relay/conversation_transport.rs` | 知识包为 merge-only；该文件构造 host context、read-only/空写根 command profile 与 internal real GUI relay（源码 `:264-343,461-684`）。 | 直接改变首句启动路径、relay grant 和回执形状。 |
| `commands.rs` | 知识包为 merge-only；该文件在 binding 后签发 relay grant，启动 supervisor transport，并负责 receipt/activation/terminal cleanup（源码 `:293-628,897-969`）。 | 直接改变首句的最早可证边界和失败闭锁。 |
| 运行资源 | 同一 workflow JSON/SQLite、supervisor sidecar、exec registry sidecar/lock、DB WAL/SHM holders、relay grant、child process group、Rust build lock/target 与单一真实测试项目。 | 同一真实 store／运行时不得并发；Cargo/Tauri/App 探针也须等待。 |

知识包本身明确禁止改变 binding、DB/JSON、capability allowlist、主管 profile 和真实 App（知识包 `:14-16,25,80`）。这限制了其授权面，但不使共享启动路径自动稳定；必须等待其实际 diff 和离线验证由指导线验收。

## 7. 可立即并行的预检与等待项

| 现在可做（只读） | 必须等待知识线稳定并获新授权 |
| --- | --- |
| 权威路由、历史失败和三句合同审计；本 evidence 的计数/事件时间线模板。 | Cargo test/check、Tauri build、npm、shape gate 或任何会争夺 build 输出的命令。 |
| 静态调用图：binding establish → transport → `thread.started` → Active → `tools/list/call`。 | 重新冻结 HEAD、承重文件 hash、binary hash、进程/holder/真实 store 基线。 |
| 静态核对 knowledge relay 插入点、进程/sidecar/cleanup 共享面。 | Syn/Codex CLI/MCP/真实 App 启动，真实 store/vault 读取、复制或验收。 |
| 记录当前脏树归属与上述 snapshot hash。 | 同一真实 store 上的任何第二条运行线，或对话三句重试。 |

## 8. 知识线稳定后建议派发的最小真实 App 重验合同

此节是建议合同，不授予运行或写入权限。前提必须同时满足：知识 relay 安全返工已由指导线独立验收；四个共享文件的最终 diff/hash 已重新冻结；工具面冲突已按第 5 节写入新的 acceptance predicate；用户明确授权一次新的真实 App Gate 0、build 与三句验收。

### 8.1 精确边界与建议写面

- 运行只允许：一次 fresh Gate 0、当前源码构建的一枚冻结 debug binary、固定测试项目交办页的三句各一次、最多一次 UI refresh、正常 Quit 与零 holder 后副本 integrity 对账。
- 正常产品行为可写入 canonical、主管 binding、工具审计和至多一张 `PendingUserConfirmation`；禁止直接写、恢复、reseed、migrate、reconcile 或 rollback 真实 store。
- 禁止改代码、测试、配置、schema、依赖、fixed test project；禁止放宽 read-only/空写根/allowlist；禁止重发、补卡、点卡、批准卡、chain/worker、stage/commit/push/reset/clean/stash。
- 建议新包白名单只列：新的 task 包、该次新的脱敏 evidence、结果发生后最小 `CURRENT.md` / `AUTHORITY.md` 同步，以及仅在出现新实际 catch 时的 `docs/harness-catch-log.md`。其它文件一律不写。

### 8.2 Gate 0 与 Gate 1

1. 冻结 HEAD、staged、porcelain、共享承重源码 hash、固定测试项目 HEAD/porcelain/manifest；确认 scoped Workbench/Tauri/dev/Vite/Codex/MCP process、registry、lock、workflow state、DB/WAL/SHM holder 全空，registry entries=0。
2. 只读记录 SQLite integrity、DB/JSON 安全投影、storage mode 与以下**新的**基线：workflow revision/audit；recorded/injected/reply/diagnostic；supervisor session/audit/binding（按 lifecycle）；proposal/Pending/decision/proposal audit；chain/execution attempt/node dispatch；registry revision/entries。不得复用本证据第 3 节的历史数字。
3. 只构建和启动一枚当前源码对应的 binary；冻结其 SHA-256、size、mtime，并确认承重源码 build 前后 hash 不变。

### 8.3 三句精确停止合同

1. **首句（只一次）**：`我想给这个游戏里的标题改成小马里奥`。必须有自然回复、S1-S5 的 message-scoped 事实完整可对账、durable binding 从本轮新增并在 JSON/SQLite 都是同一 `Active + thread_id`，且首次 `tools/list` 的名称集合严格等于新包预先选定的工具面。首句后 `recorded` 严格 `+1`；proposal/Pending/chain/worker 必须 `+0/+0/+0/+0`。无回复、无／不一致 binding、非 Active、无／错工具面、时序无法关联或身份无法对账，**立即停止，不发第二句**。
2. **第二句（仅首句全绿，且只一次）**：`按这个出方案`。必须仍是同一 thread；仅此句出现 `tools/call` 的 `submit_proposal` handler 成功事实；自然回复保留；proposal/Pending 严格 `+1/+1`，chain/worker 严格 `+0/+0`，目标卡匹配“小马里奥”。首句未绿、handler 未到达／失败、任一计数不符、或调用任何非本合同工具，**按最早可证边界停止，不发第三句**。一看到唯一 Pending 卡即停止观察，不点、不批。
3. **第三句（仅第二句全绿、卡未触碰，且只一次）**：`先别执行，告诉我这个方案准备改哪些地方。`。必须仍是同一 thread 且有自然回复；proposal/Pending/chain/worker 全部 `+0/+0/+0/+0`；不得再次 `tools/call`、补发 `submit_proposal` 或调用其他能力。任一增长、重复调用、非同 thread、无回复或时序不能对账，**立即停止并按最早事实收口**。

### 8.4 退出与回交

成功候选最多 refresh 一次，refresh 后上述计数不得重复增长。正常 Quit 后必须确认 scoped process、holder、registry、lock 均为 0；仅在零 holder 后复制 SQLite 到临时目录做 integrity/query；固定测试项目 manifest 不变。若只残留本轮冻结、精确身份匹配且零 holder 的 binary，才可按新包现场授权处理该 PID。

回交必须分开报告：Gate 0/1 证据、每句前后计数、S1-S6 的实际时间线和工具集合、所有停止触发点、process/store 清理、实际写入白名单、staged 状态与未决问题。不得把离线绿、静态调用图、历史计数或“没有卡”写成真实替代性验收通过。

## 9. 本轮裁决

本轮已完成的是**不触碰共享承重文件的恢复审计**。最早真实缺口仍是首句 message-scoped durable binding 事实；七阶段返工提供了离线失败分类与闭锁，但没有让真实 App 自动恢复。当前还发现历史“只见 `submit_proposal`”合同与静态五工具 allowlist 的口径不一致，必须在新包中先裁决。知识线安全返工稳定、指导线验收和新的真实运行授权之前，对话线仅可继续只读预检，不能启动或重试。
