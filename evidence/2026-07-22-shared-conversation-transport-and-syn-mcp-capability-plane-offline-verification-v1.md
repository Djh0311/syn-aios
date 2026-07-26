# Shared Conversation Transport + Syn MCP Capability Plane 离线验证 v1

- 收口日期：2026-07-23
- 授权任务包：`tasks/2026-07-22-shared-conversation-transport-and-syn-mcp-capability-plane-offline-implementation-package-v1.md`
- 结论：**目标代码与定向离线闸完成；aggregate shape full gate 仍非绿，且缺开工前独立 shape 快照，不能据当前 baseline/check 单独证明零净增。** 真实 App 替代性验收未执行，必须另包、用户在场授权。

## 1. 授权与未触碰面

本轮只在任务包 §4 白名单内实施。开始基线为 `HEAD=e9ad7f3a204a1ebb11ce26c1e8c05b19c04c0991`；已知并行脏改保留，`supervisor_orchestrator.rs` 与 `supervisor_orchestrator_submit_proposal.rs` 只作最小 merge。

没有启动或构建 App，没有运行 Codex CLI/MCP server；没有读写真实 Workbench store、发送真实消息、创建/批准真实卡、启动 chain/worker；没有 stage、commit、push、reset、clean 或 stash。所有运行证据均为 Rust/TypeScript 离线测试与静态命令计划检查。

## 2. 已实现范围

### 共享 transport 与两个固定 profile

- 新增共享 `conversation_transport` core 和前端 `conversationTransport.ts` controller；Agent 与交办共用 new/existing、poll、Stop、readback/transcript 与分层 receipt 状态，不复制第二套状态机。
- Agent 固定保留 `agent-codex-workspace-write` 的既有 workspace-write、项目根写根和用户配置行为。
- 主管只能由固定 `start_supervisor_conversation_transport` server command 启动：`sandbox=read-only`、`allowed_write_roots=[]`、cwd 锁定规范化项目根；拒绝 `--add-dir`、full-auto、bypass、wildcard/default 与调用方配置覆写。
- 主管 command plan 使用 `--ignore-user-config` 和宿主内联的既有 `supervisor_orchestrator` endpoint，固定 `features.multi_agent=false`；没有新 private home、MCP server 或 sidecar。

两个 profile 的实际 command-plan 断言如下（测试只检查结构与受控值，不输出真实 executable、workflow-state path 或用户内容）：

| profile | 实际宿主命令计划 |
| --- | --- |
| `agent-codex-workspace-write` | 保持既有 Codex plan：`-C <canonical project root>`、`--sandbox workspace-write`、允许写根仅为该项目根，并保留既有用户配置行为。 |
| `supervisor-read-only` | `-C <canonical project root>`、`--sandbox read-only`、`allowed_write_roots=[]`、唯一 `--ignore-user-config`；恰好三个宿主 `-c`：`features.multi_agent=false`、现有 `supervisor_orchestrator` endpoint 的 command、同 endpoint 的 args。无 `--add-dir`、full-auto/bypass、wildcard、用户 `--config`/`--mcp-config` 或 private home。 |

### 可信 binding 与 MCP capability plane

- `SupervisorConversationBinding` 复用现有 supervisor session/兼容存储，不新建 schema 或第二真源；持久化 profile、role、规范项目根、project/workflow/turn/thread/run identity、精确 capability set、生命周期和宿主固定的一分钟 lease。
- 新会话只有 host poll 观察到 `thread.started` 后才 Active；缺 project/workflow/turn/thread/capability、任一不匹配、终态或 lease 过期均 fail closed。自然 reply/transport receipt 不因此丢失。
- 单一 registry 登记 `read_worker_report`、`wait_for_worker`、`read_key_file`、`submit_proposal`；主管 profile 唯一被授予的是精确字面 `submit_proposal`。`tools/list` 与 `tools/call` 调用同一授权判定，未知、重复、大小写/空白变体、空集合、通配和 host-only 动作均拒绝。
- 共享 binding 路径仍复用现有 parser、幂等和 proposal store；同一 turn 重复 submit 最多产生一张 `PendingUserConfirmation`，离线 fixture 的 chain/worker 增量为零。没有批准动作。

### 交办与安全 receipt

- 交办主发送改接固定 supervisor transport endpoint；旧 resident command 保留兼容但不再是主路。
- receipt 分为 `transport`、`assistant_reply`、`tool_action`、`read_model_projection`、`canonical_mirror`；tool/projection/canonical 失败不会吞掉已成立 reply，前端不自动重发或补卡。
- 07-23 收口复核抓到初版 `canonical_mirror` 只有状态占位、没有真实 settlement；现已在 completed reply 后，把宿主冻结的用户原文与主管回复作为同一 turn 的两个兼容事件，一次性、幂等地走既有 Batch 2 DB-primary/JSON 投影。事件名与 `source_kind` 保留 resident 拼写仅为现有读模型兼容，不会把发送重新路由到已暂停的 resident/private-home transport。
- 同次复核还抓到 proposal handler 已持久化成功后，后续 `append_audit(...)?` 可先返回并抹掉 tool receipt。现已把 handler outcome 与 audit outcome 分开存入同一可信 binding：audit 失败时仍保留真实 Pending 卡、`tool_action`、proposal read-model projection 与自然回复，只把 audit/canonical 层标为失败；不重发、不补卡、不推进 chain。
- UI/controller 只保留安全投影，不把 raw tool arguments、stderr、argv、环境、token/auth、完整 private path 或 identity 放入 transcript、read model、测试快照或本 evidence。
- 值模式敏感扫描已跑：在 `src-tauri/src/commands.rs`、`manual_relay/conversation_transport.rs`、`mcp/supervisor_conversation_binding.rs`、`mcp/supervisor_orchestrator*.rs`（command/receipt/canonical 与 proposal read-model 路径），全部本包 UI/Jiaoban/Agent 文件，`tests/shared-conversation-transport.test.tsx` 与 `tests/jiaoban-conversation-center.test.tsx`（离线 snapshot/assertion 源），以及本 evidence 上执行 `rg -n -i '(sk-)[a-z0-9]{8}|(bearer)[[:space:]]+[a-z0-9._-]{8}|(authorization)[[:space:]]*:[[:space:]]*(bearer|basic)[[:space:]]+[a-z0-9._-]{8}|(api[_-]?key)[[:space:]]*[:=][[:space:]]*[a-z0-9._-]{8}|/(Users)/[^[:space:]]+/(\\.codex|\\.ssh)'`，结果为零命中（`rg` exit 1）。raw 字段过滤本身另由 shared transport 离线断言覆盖；这里不把字段名出现当作敏感值。

## 3. 定向离线证据

在 `prototypes/productized-desktop-shell/src-tauri`：

| 命令 / 覆盖面 | 结果 |
| --- | --- |
| `cargo test --lib 'mcp::supervisor_conversation_binding::tests' -- --nocapture` | 5 passed, 0 failed；binding 字段、固定 lease 与失效 fail-closed。 |
| `cargo test --lib 'mcp::supervisor_orchestrator::tests::shared_supervisor_' -- --nocapture` | 5 passed, 0 failed；共享 binding、同源 list/call、proposal 路径，以及 audit 失败不抹掉已持久化 tool result。 |
| `cargo test --lib 'mcp::supervisor_orchestrator::tests::stale_shared_' -- --nocapture` | 1 passed, 0 failed；过期 binding 即使调用方配置更长也不可用。 |
| `cargo test --lib conversation_transport_command_tests -- --nocapture` | 6 passed, 0 failed；前端伪造 profile/role/capability 不可达 server plan；completed turn 的 Batch 2 canonical 镜像幂等；canonical/audit 失败均不吞 reply/tool/projection。 |
| `cargo test --lib 'manual_relay::conversation_transport::tests' -- --nocapture` | 6 passed, 0 failed；两个 profile command plan、existing/new/poll/Stop 边界。 |
| `cargo test --lib 'mcp::capability_registry::tests' -- --nocapture` | 4 passed, 0 failed；精确 registry/allowlist 及变体拒绝。 |
| `cargo test --lib 'mcp::supervisor_orchestrator::tests::s1b_h2' -- --nocapture` | 5 passed, 0 failed；旧 resident/S1B-H2/R4E 兼容定向回归。 |
| `cargo check --lib` | passed；production Rust 可编译。Cargo 汇总有 598 条既有警告，未将其表述为零警告。 |

在 `prototypes/productized-desktop-shell`：

| 命令 / 覆盖面 | 结果 |
| --- | --- |
| `npm run typecheck` | passed。 |
| `node scripts/run-offline-interaction-test.mjs` | passed；输出 `offline interaction tests passed: 15`，含新增 shared transport 与 Jiaoban transcript/Stop/三种 receipt 不互吞场景。 |

初轮收口格式检查覆盖本包 10 个 Rust 文件；07-23 纠偏后又以 `rustfmt --edition 2021 --check --config skip_children=true` 复核本次触及的 4 个 Rust 文件，均通过；`git diff --check` 无输出。

## 4. 主管安全只读结论

**YES（静态 diff 审计）：本 diff 未放宽主管 sandbox 或 MCP allowlist。**

- 新路线没有为主管设置 workspace-write、非空写根、`--add-dir`、full-auto、bypass、private home 或 wildcard/default allow-all。
- profile、role、capabilities、sandbox 与写根均由 server 固定；前端 request 含未知字段时 serde 拒绝，wrapper 也会剥离这些字段。
- MCP `tools/list` 和 `tools/call` 共享 exact registry 授权；只有 active trusted binding 中的 `submit_proposal` 可达。

本结论是源代码/fixture 层结论，**不替代**真实 Codex client 的运行时验证。

## 5. 工作树、shape 与收口检查

- `git diff --cached --name-only`：**实际输出为空（零行）**；本轮没有 stage。
- task package 的历史 shape 口径是 `13/5/5`，而本次 post-change `baseline` 与 `check` 都观察到 `Errors=16 / Warnings=5 / Info=5`；`check` 因 aggregate historical debt 非零退出。
- `baseline` 模式读取的是当前形状，不是最初整包开工前快照；因此不能单靠它证明整个实施包相对最初开工零净增。07-23 本次纠偏开工前的回交已明确记录 `16/5/5`，纠偏后仍为 `16/5/5`，所以只能确认本次纠偏的 aggregate error/warn/info 计数未变；`13/5/5` 与当前脏基线不一致仍是整包未消除的 shape 比较缺口。不得把任何一组数字说成全仓绝对全绿。
- 直接执行不带 Cargo edition/child 限制的 `rustfmt --check` 会解析包外 `global_supervisor_agent.rs`（Rust 2015 默认 edition）和已有 `mcp/storage.rs` 格式漂移；因此不以该工作区外溢检查替代已通过的本包目标文件格式检查。

实际可归因的白名单变更文件：

```text
prototypes/productized-desktop-shell/src-tauri/src/manual_relay.rs
prototypes/productized-desktop-shell/src-tauri/src/manual_relay/conversation_transport.rs
prototypes/productized-desktop-shell/src-tauri/src/commands.rs
prototypes/productized-desktop-shell/src-tauri/src/command_registry.rs
prototypes/productized-desktop-shell/src-tauri/src/mcp/mod.rs
prototypes/productized-desktop-shell/src-tauri/src/mcp/capability_registry.rs
prototypes/productized-desktop-shell/src-tauri/src/mcp/supervisor_conversation_binding.rs
prototypes/productized-desktop-shell/src-tauri/src/mcp/supervisor_orchestrator.rs
prototypes/productized-desktop-shell/src-tauri/src/mcp/supervisor_orchestrator_submit_proposal.rs
prototypes/productized-desktop-shell/src-tauri/src/mcp/supervisor_conversation_transport_tests.rs
prototypes/productized-desktop-shell/src/lib/tauri.ts
prototypes/productized-desktop-shell/src/lib/conversationTransport.ts
prototypes/productized-desktop-shell/src/views/agent/AgentConversationShell.tsx
prototypes/productized-desktop-shell/src/views/projects/ProjectJiaobanPanel.tsx
prototypes/productized-desktop-shell/src/views/projects/jiaoban/useJiaobanConversationState.ts
prototypes/productized-desktop-shell/src/views/projects/jiaoban/JiaobanConversation.tsx
prototypes/productized-desktop-shell/scripts/run-offline-interaction-test.mjs
prototypes/productized-desktop-shell/tests/jiaoban-conversation-center.test.tsx
prototypes/productized-desktop-shell/tests/shared-conversation-transport.test.tsx
CURRENT.md
AUTHORITY.md
docs/2026-07-08-workbench-current-feature-inventory-for-prototype-v1.md
docs/plans/2026-07-16-master-execution-plan-conversation-first-v1.md
docs/harness-catch-log.md
evidence/2026-07-22-shared-conversation-transport-and-syn-mcp-capability-plane-offline-verification-v1.md
```

其他已存在的脏文件不归入本包。

## 6. 未跑项、真实 App 风险与唯一下一步

未跑：App 启动、真实 Codex CLI/MCP server、真实 store/消息/卡/chain/worker，全部是本包明确禁止项，不是离线门失败。

真实 App 包必须重新 Gate 0，并验证：首句自然回复；同一 thread 第二句 `submit_proposal`；恰一张 Pending 卡；第三句继续聊；工具失败仍保留回复；未批准时 chain/worker 为零；退出无残留。还必须专门验证新会话时序：binding 只有 host poll 观察到 `thread.started` 后才 Active，所以若真实 MCP client 在该观察前只做一次 `tools/list`，当前设计会安全返回空列表/拒绝调用；这是 fail-closed 的真实替代性验收风险，不能用本离线绿外推。

唯一下一步：另开“共享 Conversation Transport 真实 App 替代性验收包”，用户在场授权；本包到此停止。
