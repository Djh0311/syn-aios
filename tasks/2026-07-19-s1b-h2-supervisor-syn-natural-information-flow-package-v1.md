# 任务包：S1B-H2 主管 ↔ Syn 自然信息流转第一片（真实 App 出方案落卡 + 对话不吞）v1

日期：2026-07-19
状态：**已出包，待用户高危开工令**
所属开发线：桌面应用线 / 主管对话底座
上位计划：`docs/plans/2026-07-16-master-execution-plan-conversation-first-v1.md`
方向正本：`decisions/2026-07-18-conversation-substrate-correction-freeform-supervisor-plus-tools-v1.md`
案发证据：`evidence/2026-07-19-s1b-h2-real-app-message-to-proposal-failure-preregistration-v1.md`
本任务基线 commit：`97fca19bc8d3effd4959dec8cc4827e27cac31e6`

## 一、用户拍板与授权边界

用户已拍板的产品原则：

> 对话框只负责自然对话；用户明确说“出方案”，或主管判断已经可以出方案时，主管通过 MCP 把方案落到方案卡。信息应在 Syn、Codex 和其他 agent 之间自然流转，用户不做人工中转。

本轮“出包、给 kickoff”只授权写任务包、计划与证据，**不等于授权实施**。本包会改变真实产品中的 Codex 单工具批准逻辑，命中 `AGENTS.md` 高危清单第 3 条；实施前必须由用户在场，单独明确：

> S1B-H2 开工；授权只把 `supervisor_orchestrator.submit_proposal` 在真实主管会话中设为可用并单工具预批准。不得批准其他工具，不得放宽沙箱、全局审批或执行闸。

该预批准只允许主管把结构化方案写成 `PendingUserConfirmation` 卡，**不等于用户批准方案，更不得启动 workflow chain**。用户在卡上点“允许并开始”仍是唯一执行授权。

## 二、已知事实、未知项与实施假设

### 已知事实

1. 2026-07-19 11:06，真实 Tauri App 已先写入用户原话的 canonical `supervisor_resident_user_message_recorded`，所以“没送到主管”不属实。
2. invalid-resume 自愈已在真实 App 生效：旧 generation 5 thread 轮转到 generation 6 thread `019f7857-0630-7d50-910d-855fa3e0d87a`。
3. 主管完成只读勘察，生成了字段完整的 `submit_proposal` 调用；11:09:05 Codex 客户端在 MCP handler 前返回 `user cancelled MCP tool call`。
4. rollout 之后仍有主管 final answer，`step-0.last-message.txt` 也完整存在；但产品后端把本回合判失败，没有追加主管 canonical 消息，前端显示“这句没送到主管——稍后再试一次。”
5. 真实 App 私有家 `active/config.toml` 只有 MCP `command` / `args`，没有 H1 测试 wrapper 中的 `enabled_tools=["submit_proposal"]` 与该工具 `approval_mode="approve"`。
6. 案发后 proposal store 仍是 revision 131 / 74 张，workflow chain 仍是 40 条；没有 Pending 新卡、没有起链、没有修改测试项目文件。
7. 当前 one-shot 解析器把 stdout `"error"` 与 `"turn.failed"` 都写入同一个粘性 `terminal_error`；后续 `"turn.completed"` 不会清除它。案发 stderr 同时存在早期 websocket reset，而 rollout 最终产出了 final answer。

### 未知项

1. 案发时 Codex CLI 原始 stdout 事件的完整先后序列未被独立冻结；rollout、stderr 和 last-message 能证明回合最终产出了答复，但不能直接证明究竟是哪一条 stdout `error` 令 runner 失败。
2. `supervisor_mcp_config_toml` 是共享函数；给它加字段会辐射哪些非 resident 调用者，必须先逐调用点读清，不能按函数名猜。
3. 现有 proposal store 是否已经有可复用的“同一 resident turn 至多一张卡”幂等键，需要源码审计。

### 实施假设

- 单工具 `submit_proposal` 是 Syn 内部结构化落物动作；它只创建待确认卡，因此可以和“用户点卡才执行”的人闸分层。
- 本包复用现有 Syn 控制核心、私有 MCP server、proposal store 与主管 thread，不新增“Syn agent 层”，也不新造第五条消息运输路。
- 自然流转不等于所有 MCP 工具自动批准；能力必须按角色、server、tool 精确收口，并继续由 Syn 做参数校验、权限、审计和权威事实写入。

## 三、目标

1. 真实 Tauri App 的主管会话可以稳定调用**唯一** `supervisor_orchestrator.submit_proposal`，并落一张 `PendingUserConfirmation` 方案卡。
2. 对话、工具动作、卡片物化、用户执行授权成为四个可分别成功或失败的结果；工具或卡片失败不得吞掉已经完成的对话。
3. 只有 canonical 事实证明消息未记录时，UI 才能说“没送到主管”；已经记录或主管已经回答时必须给出对应真话。
4. MCP 工具结果回到发起调用的同一主管 thread；主管可以继续用自然语言解释成功或失败，用户不负责在 Syn 与 agent 之间搬运结果。
5. transport / UI 刷新重试不得让同一 resident turn 重复创建方案卡。

## 四、架构合同（本包必须守住）

| 层 | 负责什么 | 不负责什么 |
|---|---|---|
| 对话面 | 用户与主管的自然语言理解、追问、协商、解释 | 不直接写方案卡、不启动链 |
| MCP 动作面 | 方案、任务、状态、证据等结构化动作与结果回传 | 不替代用户执行授权 |
| Syn 控制核心 | 路由、角色能力、参数校验、幂等、审计、权威事实 | 不要求用户充当中转 |
| 卡片 / 索引 | 结构化实物和权威状态的投影 | 不是第二条聊天通道 |
| 用户人闸 | 在方案卡上批准执行 | 不为主管内部落 Pending 卡重复点一次 |

普通聊天、MCP 工具动作、用户批准执行三层不得合成一把闸。工具失败不能抹去聊天；聊天成功也不能伪报卡片成功。

## 五、允许读取

- `/Users/yoyi/workspace/product-line/AGENTS.md`
- `/Users/yoyi/workspace/product-line/CURRENT.md`
- 本包、上位计划、方向正本与案发 evidence
- `prototypes/productized-desktop-shell/src-tauri/src/supervisor_resident_oneshot_session.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/supervisor_session_launcher.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/supervisor_resident_oneshot_tests.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/mcp/supervisor_orchestrator_resident_session.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/project_consultation_proposal_store.rs`
- `prototypes/productized-desktop-shell/src/views/projects/jiaoban/useJiaobanConversationState.ts`
- `prototypes/productized-desktop-shell/src/views/projects/ProjectJiaobanPanel.tsx`
- `prototypes/productized-desktop-shell/tests/jiaoban-conversation-center.test.tsx`
- 真实 App 案发 store、rollout、last-message、stderr 与进程登记表（只读）

## 六、允许写入

默认最小写面：

- `prototypes/productized-desktop-shell/src-tauri/src/supervisor_resident_oneshot_session.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/supervisor_session_launcher.rs`（仅当调用点审计证明这里是正确的真实产品配置层）
- `prototypes/productized-desktop-shell/src-tauri/src/supervisor_resident_oneshot_tests.rs`
- `prototypes/productized-desktop-shell/src/views/projects/jiaoban/useJiaobanConversationState.ts`
- `prototypes/productized-desktop-shell/src/views/projects/ProjectJiaobanPanel.tsx`
- `prototypes/productized-desktop-shell/tests/jiaoban-conversation-center.test.tsx`

条件写面：

- `prototypes/productized-desktop-shell/src-tauri/src/mcp/supervisor_orchestrator_resident_session.rs`：只允许补本包所需的内部结果 / 审计语义。
- `prototypes/productized-desktop-shell/src-tauri/src/project_consultation_proposal_store.rs`：只有现有幂等能力不足且能使用**服务端派生**的 resident turn 身份时才允许；不得新增模型可控幂等参数。
- `CURRENT.md`、`docs/harness-catch-log.md` 与新完成 evidence：只在实现与验证后回写。

任何新增写面必须先说明必要性；碰到来源不明的并行改动，按 `BLOCKED_DIRTY_OVERLAP` 停止。

## 七、禁止事项

- 不得新增 Tauri command、sidecar JSON 种类、MCP server 或消息运输路。
- 不得改变 `--sandbox read-only`、`approval_policy=never`、`approvals_reviewer`、path-lock、写根、进程组清理或看门狗边界。
- 不得使用 server-wide / wildcard 批准、`default_tools_approval_mode`、`--full-auto`、`--dangerously-bypass-approvals-and-sandbox`，不得批准 `submit_proposal` 之外的工具。
- 不得把 H1 测试 wrapper 直接塞进产品 `PATH`；产品能力必须落在真实 App 私有家 / one-shot 的正确配置消费层。
- 不得批准方案卡、启动 chain、派发 worker、修改 `/Users/yoyi/codex-workflow-mario-test` 业务文件。
- 不得借本包实现底2、worker 求助、reviewer、终标或 Syn 自有 agent 层。
- 不得把原始 stderr、rollout 内部错误或 MCP 参数全文投影给用户。
- 不得递归重跑主管回合；invalid-resume 仍只允许同回合一次 rotate → initial。

## 八、实施要求

### A. 真实产品单工具可达

1. 先画清 `supervisor_mcp_config_toml` 的全部生产调用者与 initial / resume 实际消费位置。
2. 真实 App 生成的主管私有配置必须精确包含：
   - `mcp_servers.supervisor_orchestrator.enabled_tools=["submit_proposal"]`；
   - `mcp_servers.supervisor_orchestrator.tools.submit_proposal.approval_mode="approve"`。
3. initial 与 resume 都必须实际收到配置；测试不能只断言“外层 argv 看起来带了”。
4. 如果共享函数会让其他角色或非 resident 路径获得同样能力，必须缩小 / 拆分配置函数，不能接受辐射式放宽。
5. 配置字段漂移或真实 Codex 不认识字段时 fail closed，不降级到广域批准。

### B. one-shot 完成语义

按以下优先级收口状态机：

1. `turn.failed`、非零退出、缺 `turn.completed`、缺 thread、缺 final message 仍失败关闭；invalid-resume 现有一次轮转语义不变。
2. 若先出现 `error`，但之后同一进程满足：退出 0、`turn.completed`、合法 thread、非空 final message，则该 `error` 只能作为诊断留审计，不能覆盖最终完成事实。
3. 工具调用返回 Err 后，若主管仍在同一 turn 产出 final answer 并完成，conversation outcome 必须成功落 canonical supervisor message；proposal outcome 可以单独失败。
4. 不能简单“见到任意 final message 就算成功”；缺完成事件或异常退出仍拒绝。

至少补四组案发测试：

- transient `error` → final message → `turn.completed` → exit 0：对话成功；
- `turn.failed`，即使已有 message：失败；
- `error` + 无 completion：失败；
- resume 非零且无真实 `thread.started`：现有单次换代仍成功，不能回归。

### C. canonical 与 UI 真话

1. 用户消息只由既有命令写 canonical，不做前端乐观副本。
2. 命令失败路径也应尽力重读 canonical workflow / proposal，避免已记录消息要等下次刷新才出现。
3. 用户面至少区分：
   - canonical 未记录：`这句没送到主管——稍后再试一次。`
   - 已记录但主管未完成：`消息已送到主管，但主管这次没回上来——可以再发一次。`
   - 主管已完成但方案卡未生成：显示主管 canonical 答复，并给出 `主管收到了，但方案卡没有生成——请再说一次“出方案”。`
   - canonical / proposal 仅刷新失败：沿用“已经送到，但对话还没刷新”的真话。
4. 成功路径只重读 canonical workflow / proposal；不在前端伪造主管消息或 Pending 卡。

### D. 同 thread 结果回路与幂等

1. MCP 成功 / 失败结果必须回到原主管 thread，允许主管自然解释，不新开旁路会话。
2. 同一 resident turn 因 transport、回读或 UI 重试，proposal store 最多新增一张卡。
3. 幂等身份必须复用既有 proposal 机制或服务端派生的 message / turn 身份；不得让模型提供或覆盖 idempotency key。
4. 用户后来明确再说一次“出方案”属于新意图，可以产生新 turn；本包只禁止**同一 turn 的技术性重复落卡**。
5. 若源码中没有可安全派生的内部身份，停止并回传设计缺口，不得用标题 / 文案 hash 猜同一方案。

### E. 审计与隐私

- 至少能分辨 conversation completed / proposal materialized / proposal tool failed 三种内部结果。
- 原始 `user cancelled MCP tool call`、stderr、工具参数只留既有私有 runtime / audit detail，不进入普通对话正文或 UI read model。
- 不新增“为了提示而提示”的常驻牌；失败文案只在对应回合出现。

## 九、变更辐射面

改变的假设：过去 `submit_supervisor_resident_answer` 被当成“主管回合 + 落卡”一个原子成功值；H2 改为“用户消息、主管答复、工具结果、卡片投影”四个独立但可对账的结果。

必须逐项核：

1. `supervisor_resident_oneshot_session.rs`：stdout 事件优先级、invalid-resume、自愈、final message、进程清理。
2. `supervisor_session_launcher.rs`：共享 MCP 配置调用者，防能力扩散。
3. resident orchestrator：`submit_proposal` 严格 schema、server-owned 字段、Pending-only、chain 不动。
4. `useJiaobanConversationState.ts`：catch 后 canonical refresh、草稿清理时机、错误文案缓存。
5. `ProjectJiaobanPanel.tsx`：generic fallback 只能用于真未送达。
6. 方案索引与右侧卡：新增卡只从 proposal store 重读，不在对话流复制。
7. M5 DB / JSON 对账：本包不改存储模式；新增审计若进入既有 sidecar，必须继续走已有桥。

## 十、五态旅程走查

- **说**：用户和主管自然多轮；消息已记录就立即可见。主管工具失败时，对话仍能继续。
- **批**：主管或用户明确“出方案”后，右区正好出现一张 Pending 卡；未点卡前 chain 不变。
- **干**：不涉及执行链改造；运行中输入框继续可说，底2另包。
- **交货**：不涉及；既有交货卡和对话投影零回归。
- **卡住**：不涉及 worker 卡住语义；本包只保证主管 / 落卡失败有诚实出口，不伪装成“消息未送达”。

## 十一、形状影响

- 任务类型：高危功能缺陷 / 对话底座修复。
- 新增代码落点：原则上不新增模块；在现有 runner、配置、hook 与测试内最小修。
- 棘轮文件：预计会碰 `ProjectJiaobanPanel.tsx` 与离线交互测试；不得碰 `src-tauri/src/lib.rs`、`real_execution_command.rs`、`types.ts`、`styles.css`。
- 预计变化：Rust 实现 +40～120 行、Rust 测试 +80～180 行、TS/TSX 实现 +20～80 行、前端测试 +40～100 行；明显超出即先简化 / 回报。
- 新增 Tauri command：否。
- 新增 sidecar JSON：否。
- shape gate 豁免：否。
- 本任务基线 commit：`97fca19bc8d3effd4959dec8cc4827e27cac31e6`。
- 本任务完成 commit：不 commit；执行线回传 end commit。

## 十二、脏基线冻结

本包生成时以下目标文件已是当前主线未提交改动，不得清理、reset、stash 或据此宣称是 H2 新增：

| 文件 | 状态 | SHA-256 |
|---|---|---|
| `src-tauri/src/mcp/supervisor_orchestrator_resident_session.rs` | M | `a4f4d025843922a7ba58d0a4245b89cf4cb1c16a65f7433098083277156141fe` |
| `src-tauri/src/supervisor_session_launcher.rs` | M | `03c8954cc2958a80c4a2f189272aaef3cc65344ddc70bd566fa690319b067e0c` |
| `src/views/projects/ProjectJiaobanPanel.tsx` | M | `fc93a8669c003f22b4b9082cf91c6c4b078801ee64df73e8ac3a2ce16ae813fe` |
| `src/views/projects/jiaoban/useJiaobanConversationState.ts` | M | `1f71671735ffc2a296f12cbe6bd36c6b801b993e21d3666caec746a3661265b4` |
| `tests/jiaoban-conversation-center.test.tsx` | M | `1996bd39e4c21f0aaaae6bd57fe3cd0685ba22cfc83bdc91a0853fd20653b656` |
| `src-tauri/src/supervisor_resident_oneshot_session.rs` | ?? | `baf7aa7950a4e339ecc6be155ac56de22f3ecc29f4d879abe6098a32910cd267` |
| `src-tauri/src/supervisor_resident_oneshot_tests.rs` | ?? | `e0313aa2b682468325289bcdd1c10c1543ca502cdf8b5b4260aaf3c986afb685` |

执行前逐个复核；若 hash 已变且无法由用户 / 当前主线归属，立即 `BLOCKED_DIRTY_OVERLAP`。

## 十三、验收标准

### 离线 / 定向

1. 产品私有 MCP config 测试证明真实 one-shot initial 与 resume 都只放行 / 预批准 `submit_proposal`。
2. 配置反例断言：无 wildcard、无 server-wide default、无 `approval_policy` / reviewer / sandbox 放宽、无 full-auto / bypass。
3. runner 四组事件顺序案发测试全过，invalid-resume 既有测试不回归。
4. 前端测试覆盖四种真话、catch 后 canonical refresh、成功只读 canonical、无乐观消息 / 卡片。
5. 幂等测试证明同一 turn 重放两次只有一张 Pending 卡；新 turn 不被误吞。

### 真实 Tauri App（用户在场；到卡为止）

1. 记录复跑前 proposal 数、chain 数、generation、thread、进程登记表。
2. 用户先说：`我想给这个游戏里的标题改成小马里奥`；再明确说：`按这个出方案`。
3. 中栏能看到用户消息与主管自然答复；不得出现“没送到主管”的假话。
4. 右区 / 方案索引正好新增一张目标匹配的 `PendingUserConfirmation` 卡。
5. MCP handler 有到达证据；工具结果回到同一主管 thread，后续一句仍续同 thread。
6. 未点批准卡；chain 数不变；`/Users/yoyi/codex-workflow-mario-test` 文件 hash / git 状态不因本包 live 改变。
7. 重复刷新不增卡；不通过重复发送“出方案”伪造幂等验证。
8. 测试结束相关 one-shot / MCP 进程登记表清空；只读 `ps` 对账无本轮孤儿。

### 聚合四闸

- S1B 定向、S1 聚合、M5B、M5C 全绿。
- `cargo test --lib` 不低于当前 **1009 passed / 0 failed / 44 ignored**（新增测试只增不减；ignored 口径单列）。
- `cargo check --offline`、`pnpm typecheck`、离线 interaction 测试通过。
- shape baseline / check 保持历史 **13 errors / 5 warnings / 5 infos**，零净增；check 的历史非零退出如实报告。
- `git diff --check` 通过；只对本包改动 Rust 文件做 `rustfmt --check`，不得全仓格式化制造无关 diff。

## 十四、停止条件

- 未拿到本包开头的高危精确授权：只读勘察后停，不改产品审批逻辑。
- 正确实现需要批准第二个工具、放宽全局审批 / 沙箱或改用户执行闸：停。
- 共享配置无法隔离 resident supervisor：停，回传调用图。
- 无安全的服务端幂等身份且只能让模型传 key / 用文案猜：停。
- 真实 App 复跑要求用户点方案卡或起链：H2 到卡即停，另等底1授权。
- 发现来源不明的 dirty overlap：停，不 reset / clean / stash。

## 十五、必须回传（10 项）

1. 修复范围与真实用户结果。
2. 改动文件。
3. 产品配置 initial / resume 如何精确到单工具，未辐射哪些路径。
4. runner 成功 / 失败优先级与案发测试。
5. 对话 / 工具 / 卡片 / 用户授权如何分层，UI 四种真话证据。
6. 幂等键来源与重复落卡反例。
7. 真实 App：消息、同 thread、卡片数、chain 数、项目文件、进程对账。
8. 定向与聚合闸数字、shape baseline / check。
9. 是否新增 command / sidecar、是否触碰棘轮文件、start / end commit、未 commit / 未 stage 声明。
10. 被哪道闸拦过什么；没有也写“无”，并回写 `docs/harness-catch-log.md`。

## 十六、总指导回收

只有“真实 App 对话可见 + 单工具 handler 到达 + 一张 Pending 卡 + chain / 项目文件不动 + 无假错误 + 四闸”同时成立，才可接受并把下一施工项恢复为底1真机首单（从**用户点卡**继续）。fixture-only、复制店或只证明模型“想调用”均不得收口。
