# 任务包：共享 Conversation Transport + Syn MCP Capability Plane 离线实施 v1

- 日期：2026-07-22
- 状态：**已出包，未执行；等待用户对重档 #3 的单独明确授权**
- 类型：共享对话底座、主管只读 profile 与统一 MCP 能力层的离线实现
- 架构正本：`decisions/2026-07-22-shared-conversation-transport-and-syn-mcp-capability-plane-v1.md`
- 能力审计：`handoffs/2026-07-22-jiaoban-conversation-module-reuse-and-syn-mcp-capability-guidance-v1.md`
- 本包不授权：改代码、启动 App、运行 Codex CLI / MCP server、操作真实 store、发送真实消息、stage 或 commit

## 0. 唯一目标

把智能体页现有 Codex relay 的 existing/new、event mapping、poll、Stop 与 readback 抽成一套 **profile-driven Conversation Transport**，让交办页改接同一 transport；同时将现有 `supervisor_orchestrator` 收敛成服务端 registry + profile/role allowlist 驱动的 **Syn MCP Capability Plane**。本包首个结构化能力仍只有 `submit_proposal`，成功最多生成一张 `PendingUserConfirmation` 卡，未获用户点击批准时不得启动 chain。

本包完成线是**代码 + 离线回归**。真实 App 的首句、同 thread 第二句、落卡和第三句续聊必须另包、用户在场授权；离线全绿不能外推为真实 App 已可用。

## 1. 授权级别与唯一 kickoff

本包会改动 sandbox/profile 校验和 MCP 工具授权逻辑，命中 `AGENTS.md` 高危清单 #3“改动安全闸 / 沙箱 / codex 审批逻辑本身”，因此必须走重档：用户在场、单独一步、明确授权、离线完成后由主导线只读核一件事——diff 是否放宽主管 sandbox / MCP allowlist。

只有用户发送下列等价精确授权后，才可进入代码实施：

> 共享 Conversation Transport + Syn MCP Capability Plane 离线实施开工；只按 2026-07-22 v1 任务包白名单改动。主管必须保持 read-only + 空写根，MCP 必须服务端精确 allowlist；不得启动 App、Codex CLI 或 MCP server，不得操作真实 store、发送消息、落卡、起 chain、stage 或 commit。代码与离线闸通过即停，真实 App 另包另授权。

“允许扩充现有对话模块”、本架构决策、本任务包落档，均不等于上述重档授权。

## 2. 开包时冻结事实

- `HEAD=e9ad7f3a204a1ebb11ce26c1e8c05b19c04c0991`。
- staged 为空；工作树已有多组已知归属脏改，不 reset、clean、stash、覆盖或整文件回写。
- `manual_relay.rs` 当前 SHA-256=`076101d5c3fb55250777c96ba5ccb3f5585f2db58e403acb91a6697a1b6eaf88`。
- `codex_local_runner.rs` 当前 SHA-256=`4f83ca3da18206a925025820329180c3c077a242c919800ca9ae1ddbfb1c046c`；已有 read-only + 空写根 command plan 能力，**默认不在本包写入面**。
- `mcp/mod.rs` 当前 SHA-256=`ce3235a9b7f09e67e475a6ee7c195bb08183dca4404c053d4f0eec7399e24ed7`。
- `mcp/supervisor_orchestrator.rs` 当前 SHA-256=`4d44aff380d92a2a3d9b61e134ac15f727df773f1858a897d3929734db1b40a2`，且已有已知脏改；只可基于当前内容最小合并。
- `mcp/supervisor_orchestrator_submit_proposal.rs` 当前 SHA-256=`afae9f9fc1c5efc5d78fac14b7341da49965e23a6b7363253d8a3b5c7ac67f36`，且已有已知脏改；只可基于当前内容最小合并。
- `AgentConversationShell.tsx` 当前 SHA-256=`8b5f471c3563a7f4b77575eef6beb6b0c1ba717bf15a191b6be7f8e47b31fdba`。
- `conversationEngine.ts` 当前 SHA-256=`b984ad9e86b4c65b1d8a4ef8294576334bf1a72ece6ac9ffd2afb6398172e380`。
- `useJiaobanConversationState.ts` 当前 SHA-256=`47ac7053f55403c55d0a467703937b865c01fe001413bec81dc9776e46558bd2`。

实施者开工时必须重新冻结 HEAD、staged、porcelain 和上述承重文件 hash。任一脏项所有者不明或承重文件相对本包基线再次漂移，先报告差异；不能整文件覆盖。

## 3. 冻结合同

### 3.1 Transport / adapter 合同

首版只支持 `codex` adapter，但公共接口不得再叫 resident：

- `profile_id`：宿主从固定枚举选择，前端不能提交 sandbox、写根、approval 或 capability 数组。
- `conversation_id` / `thread_id`：支持 new 与 existing；只有真实 `thread.started` 才能把新会话标成可续接。
- `turn_id`：每次用户发送唯一；重试、poll、Stop、tool receipt 和 canonical 镜像均按同一 turn 对账。
- 生命周期：`idle → starting → running → completed | failed | stopped`；现有 polling、Stop、进程组清理和 readback 语义不退化。
- receipt 分层：`transport`、`assistant_reply`、`tool_action`、`read_model_projection`、`canonical_mirror` 独立结算；后两层失败不能吞掉已成立的对话回复。

前端 `conversationTransport.ts` 是两张页面共用的 transport/session 状态入口；不得把 `AgentConversationShell` 整体复制进交办页，也不得在交办页再建第二套 poll/Stop 状态机。

### 3.2 两个固定 profile

| profile | 固定宿主策略 | MCP 能力 |
| --- | --- | --- |
| `agent-codex-workspace-write` | 保持智能体页当前 `workspace-write`、项目根写根和既有用户配置行为 | 保持现状；本包不顺手注入主管工具 |
| `supervisor-read-only` | `sandbox=read-only`、`allowed_write_roots=[]`、无 `--add-dir`、cwd 锁定项目根、禁 full-auto / bypass / wildcard | 仅宿主冻结的精确集合；本包只授予 `submit_proposal` |

`supervisor-read-only` 的 Codex command plan 必须忽略用户自定义 MCP 配置，只以内联、宿主生成的配置连接现有 `supervisor_orchestrator` endpoint，并固定关闭 multi-agent。不得新建 private `CODEX_HOME`、新 MCP server 或新 sidecar；不得影响 `agent-codex-workspace-write` 的现有命令形状。

### 3.3 Conversation turn 可信绑定

宿主在启动主管 turn 前建立 `ConversationTurnBinding`，至少包含：

- `profile_id=supervisor-read-only`、`role=project_supervisor`；
- `project_id`、规范化 `project_root`、`workflow_id`；
- `turn_id`、transport attempt、run identity；
- host-observed `thread_id`（新会话在 `thread.started` 后补齐）；
- 精确 capability set、lifecycle status、时间戳。

绑定必须复用现有 supervisor orchestrator / Batch 2 DB-primary 写路和兼容投影，不新增独立文件或 schema 真源。MCP 只能读取宿主建立的 active binding；缺字段、项目不一致、turn 不 active、thread 尚未完成可信绑定时，`submit_proposal` fail closed，但自然回复与 transport receipt 继续保留。旧 resident binding 暂时兼容保留，替代验收前不删除。

### 3.4 Capability registry / allowlist 合同

- 单一 registry 描述 capability id、schema、允许 role/profile、handler、审计和人话错误；`submit_proposal` 不再依赖 resident run-id prefix 才成为可授权能力。
- `tools/list` 与 `tools/call` 必须调用同一个服务端授权判定；客户端传入的工具名或列表不能扩大权限。
- 未注册、重复、大小写变体、wildcard、空集合默认放开均拒绝。
- 现有 `read_worker_report`、`wait_for_worker`、`read_key_file` 可以登记为已有能力事实，但**不会因此自动授给主管对话 profile**。
- `dispatch_worker`、`inspect_worker`、`follow_up_worker`、`finalize`、`report_user` 等 host-only 动作本包不得公开。
- `submit_proposal` 继续复用既有 parser、幂等、proposal store 和 `PendingUserConfirmation` 语义；工具返回原 thread，同时刷新对应 proposal/workflow read model。

### 3.5 交办页结算合同

- 普通消息可以只得到自然回复，不要求 proposal。
- 明确请求方案时，工具成功最多生成一张 Pending 卡；重复 tool call 仍按既有幂等键合并。
- 工具失败：保留 assistant reply，另显示结构化动作失败；不能改写成“没送到主管”。
- projection/canonical 失败：保留 transport、reply 与已成立的 tool result，分别显示可理解状态；不得自动重发或补卡。
- 只有用户点击 Pending 卡批准才进入既有 chain；本包所有离线测试的 chain/worker 增量必须为 0。
- 交办 transcript 以共享 transport/session 状态为聊天来源；canonical/blackboard 是事实镜像和 read model，不再是发送与呈现自然对话的唯一前置来源。

## 4. 写入白名单

下列清单是上限，不是“必须全改”；未形成必要 diff 的文件保持不动。

### 4.1 后端

- `prototypes/productized-desktop-shell/src-tauri/src/manual_relay.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/manual_relay/conversation_transport.rs`（新增）
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/command_registry.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/mcp/mod.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/mcp/capability_registry.rs`（新增）
- `prototypes/productized-desktop-shell/src-tauri/src/mcp/supervisor_conversation_binding.rs`（新增）
- `prototypes/productized-desktop-shell/src-tauri/src/mcp/supervisor_orchestrator.rs`（已有脏改；merge-only）
- `prototypes/productized-desktop-shell/src-tauri/src/mcp/supervisor_orchestrator_submit_proposal.rs`（已有脏改；merge-only）
- `prototypes/productized-desktop-shell/src-tauri/src/mcp/supervisor_conversation_transport_tests.rs`（新增）

### 4.2 前端与离线测试

- `prototypes/productized-desktop-shell/src/lib/types/manualRelay.ts`
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `prototypes/productized-desktop-shell/src/lib/conversationEngine.ts`
- `prototypes/productized-desktop-shell/src/lib/conversationTransport.ts`（新增）
- `prototypes/productized-desktop-shell/src/views/agent/AgentConversationShell.tsx`
- `prototypes/productized-desktop-shell/src/views/projects/ProjectJiaobanPanel.tsx`
- `prototypes/productized-desktop-shell/src/views/projects/jiaoban/useJiaobanConversationState.ts`
- `prototypes/productized-desktop-shell/src/views/projects/jiaoban/JiaobanConversation.tsx`
- `prototypes/productized-desktop-shell/scripts/run-offline-interaction-test.mjs`
- `prototypes/productized-desktop-shell/tests/helpers/offlineConversationEngineScenario.tsx`
- `prototypes/productized-desktop-shell/tests/jiaoban-conversation-center.test.tsx`
- `prototypes/productized-desktop-shell/tests/shared-conversation-transport.test.tsx`（新增）

### 4.3 收口文档

- `evidence/2026-07-22-shared-conversation-transport-and-syn-mcp-capability-plane-offline-verification-v1.md`（新增）
- `CURRENT.md`
- `AUTHORITY.md`
- `docs/2026-07-08-workbench-current-feature-inventory-for-prototype-v1.md`
- `docs/plans/2026-07-16-master-execution-plan-conversation-first-v1.md`
- `docs/harness-catch-log.md`（只有实抓 catch 时 merge-only 追加）

若实现证明必须修改 `codex_local_runner.rs`、`lib.rs`、存储 schema、旧 resident/private-home 模块、样式、非测试项目 path-lock 或任何包外文件，立即停止并报告 `BLOCKED_PACKAGE_SCOPE_EXPANSION`，不得自行扩包。

## 5. 实施切片

### A. 先红：profile 与 receipt 合同

先增加失败测试，证明当前代码尚不能：由宿主枚举解析两个 profile、生成主管 read-only/零写根命令计划、隔离用户 MCP 配置、分层结算 receipt，以及让两张页面调用同一个 transport abstraction。红灯必须来自缺失能力，不得故意破坏既有断言。

### B. 抽取共享 transport

将 `AgentConversationShell` 内的 new/existing send、poll、Stop、timeout/readback 状态移入共享逻辑；现有智能体页经该逻辑继续工作。新增通用 Tauri conversation commands 时，旧 manual relay commands 保留兼容并委托同一 core，不删除既有入口。

### C. 落主管 profile、binding 与 capability registry

宿主从 `profile_id` 派生全部安全参数；主管 profile 使用 read-only、空写根、受控 MCP 配置。将 `tools/list` / `tools/call` 改成 registry + allowlist 同源判定；把 `submit_proposal` 从 resident-prefix 绑定泛化为可信 conversation-turn binding，同时保留旧 resident 兼容路径。

### D. 交办页改接共享 transport

用交办页自己的布局渲染共享 transcript/receipt；发送、续聊、Stop、错误态来自同一 transport。工具成功后只刷新对应 proposal/read model；工具、projection、canonical 各自失败分别上脸。删除前端 resident 专用 truth-table 的主路径依赖，但不删除旧后端 resident command。

### E. 回归与收口

跑完离线闸后新增简短 evidence，清楚区分：已实现、离线已证、真实 App 未验、历史 shape 债。只有到这一步才把 CURRENT / AUTHORITY / feature inventory / master plan 更新为“离线完成，待真实 App 替代性验收另包”。

## 6. 先红后绿验收

至少覆盖以下组：

1. **Agent 不回归**：existing send、new thread、第二句续接、poll、Stop、timeout/readback 保持；`agent-codex-workspace-write` 的 sandbox、写根和用户配置行为不变。
2. **主管 profile 死线**：read-only、写根空、无 `--add-dir`、无 full-auto/bypass/wildcard；调用方伪造 sandbox、写根或 capability 不可达后端命令计划。
3. **配置隔离**：主管命令计划忽略用户自定义 MCP，只注入宿主生成的现有 orchestrator endpoint，并关闭 multi-agent；不创建 private home。
4. **binding fail closed**：project/workflow/turn/thread/capability 任一缺失或不匹配，`submit_proposal` 被拒；自然 reply 与 transport receipt 不丢。
5. **allowlist 双闸**：同一 profile 下 `tools/list` 只见 `submit_proposal`，`tools/call` 对未授权/未知/变体/wildcard 均拒；host-only 动作不可见也不可调用。
6. **方案幂等**：同一 turn 重复 submit 最多一张 Pending 卡；普通聊天零 proposal；未批准时 chain/worker 零增量。
7. **分层 receipt**：reply 成功 + tool 失败、tool 成功 + projection 失败、reply 成功 + canonical 失败三组均不互相吞并，不自动重试或补卡。
8. **交办共享接线**：首句、同 thread 第二句、第三句普通追问的离线场景使用同一 transport/session abstraction；Stop 解锁输入；Jiaoban 不再调用 resident command 作为主发送路。
9. **敏感信息扫描**：UI、canonical/read model、测试 snapshot 和 evidence 不含正文之外的 tool arguments、raw stderr、argv、环境、token/auth、完整 private path 或完整 identity。
10. **旧路兼容**：旧 resident 代码仍编译，现有 S1B/H2/R4E 定向离线测试不因泛化 binding 被静默放宽或删除。

## 7. 必跑离线闸

在 `prototypes/productized-desktop-shell` 内运行：

1. 新增 transport/profile/capability 定向 Rust 测试；
2. 相关 `manual_relay`、`supervisor_orchestrator`、`submit_proposal`、S1B/H2/R4E 回归；
3. `cargo check --lib`（production Rust 必跑，不能只报 test）；
4. `npm run typecheck`；
5. `node scripts/run-offline-interaction-test.mjs`；
6. 仓根 `node scripts/harness/workbench-shape-gate.js --mode baseline` 与 `--mode check`；历史 `13/5/5` 单列，只接受本包零净增；
7. 仓根 `git diff --check`；
8. 收口时复核 `git diff --cached --name-only` 仍为空，并列出实际变更文件是否全部落在白名单。

不得用 App、Codex CLI、MCP server 进程、真实 store 或真实消息补离线证据。若本机依赖使全量闸不可达，先跑最小定向 fallback，并把未跑项和 blocker 如实写入 evidence；不能把部分绿写成全绿。

## 8. 立即停止条件

- `BLOCKED_DIRTY_OVERLAP`：承重文件出现无法归属的并行 hunk，或必须整文件覆盖才能继续。
- `BLOCKED_SUPERVISOR_PERMISSION_EXPANSION`：主管命令计划出现 workspace-write、非空写根、`--add-dir`、full-auto、bypass、wildcard/default allow-all。
- `BLOCKED_CAPABILITY_BINDING_UNTRUSTED`：能力集来自前端、MCP 参数或模型文本，不能由宿主可信 binding 证明。
- `BLOCKED_REPLY_TOOL_COUPLING`：工具/projection/canonical 失败仍会吞掉已成立 reply，且最小修复超出白名单。
- `BLOCKED_STORAGE_OR_SIDECAR_EXPANSION`：需要新私有 home、MCP server、sidecar、DB schema 或第二真源。
- `BLOCKED_PACKAGE_SCOPE_EXPANSION`：需要修改包外文件、安全边界或非测试项目才能通过。
- 任一测试出现 chain/worker 自动启动、真实卡批准、真实 store 写或非测试项目执行。

停止时保留当前安全状态，只报告最早 blocker 和最小下一包，不 reset/clean/stash，不用放宽闸来换绿。

## 9. 明确禁止

- 不启动/构建真实 Tauri App，不运行 Codex CLI 或 MCP server，不读写真实 Workbench store。
- 不发送真实交办消息，不创建/批准真实方案卡，不启动 chain/worker。
- 不复制 `AgentConversationShell`、`manual_relay` 或另建交办专用 transport。
- 不新增 private `CODEX_HOME`、MCP server、sidecar、wildcard/default allow-all。
- 不顺手删除智能体页、旧 resident/private-home、历史 task/handoff/evidence。
- 不宣称其他 agent 已接入；首版 adapter 只有 Codex。
- 不 stage、commit、push、reset、clean 或 stash。

## 10. 回交格式与后续唯一下一步

离线完成后的回交必须包含：

1. 实际改了哪些白名单文件；
2. 两个 profile 的实际 command-plan 证据；
3. `tools/list` / `tools/call` 同源 allowlist 与 trusted binding 的定向测试证据；
4. Agent/Jiaoban 共享 transport 与分层 receipt 的离线证据；
5. chain/worker 零增量、staged 为空、shape 零净增；
6. `cargo check --lib`、typecheck、离线交互、`git diff --check` 的真实输出；
7. 历史债、未跑项和 blocker 单列；
8. 主导线只读 yes/no：本 diff 是否放宽主管 sandbox / MCP allowlist；
9. `docs/harness-catch-log.md` 本轮是新增 catch 还是零 catch。

若全部离线闸通过，唯一下一步是另开“共享 Conversation Transport 真实 App 替代性验收包”：用户在场，从全新 Gate 0 验首句自然回复、同 thread 第二句 `submit_proposal`、一张 Pending 卡、第三句续聊、工具失败不吞回复、未点卡时 chain/worker 为 0、退出无残留。该 live 包不得由本任务包自动续跑。
