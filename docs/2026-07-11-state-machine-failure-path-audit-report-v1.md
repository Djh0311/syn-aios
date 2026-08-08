# 状态机失败路径系统审计报告 v1

> 资料状态（2026-08-09）：日期和版本受限的历史审计报告，只证明当时检查到的现象，不定义当前产品、计划或授权。涉及当前代码的结论必须重新核对源码和新鲜验证。

日期：2026-07-11
性质：第一段只读审计；未修改代码、`CURRENT.md` 或 `decisions/`。
审计对象：`prototypes/productized-desktop-shell` 当前工作树。

## 结论先行

主路径的目标是「说需求 → 看方案和工序图 → 允许并开始 → 开干」，且唯一人闸是“允许并开始”；人闸外的锁、复核迟到、进程中断不应变成用户要理解的流程节点。依据：`decisions/2026-07-11-main-flow-simplicity-principle-v1.md:7-11`。

当前最严重的不是“缺一个重试按钮”，而是链的停止/失败被编排层统一包装成 `stage: "ran"`，前端也把 `failed`、`stopped` 当作进入“做好了”脸的条件。结果是用户已经按过唯一人闸，仍可能看到“已交货”而不是可处置的失败；这是主流程语义错误。依据：`src-tauri/src/director_agent.rs:4060-4091`、`src/views/projects/ProjectJiaobanPanel.tsx:1156-1165`、`src/views/projects/ProjectJiaobanPanel.tsx:2892-2904`。

### 颜色和术语

- 🟢 **已自愈**：无需新的人类决定，下一次推进或受限自动重试可恢复。
- 🟡 **部分自愈**：有恢复代码，但必须依赖用户再次点击“接着跑”，或只覆盖一种失败。
- 🔴 **悬挂/死脸**：状态没有自动收口，或界面给出错误的完成语义、没有该状态专属的正确动作。
- “无生产写入”只指本轮检索到的生产赋值路径；枚举仍可能承接旧盘或测试数据，不能把它当作已安全删除的结论。

### 枚举口径校正

任务包称 work item 为“九态”，但控制核心的合法跳转出现了 **12 个状态**：`draft`、`ready_to_dispatch`、`running`、`waiting_for_permission`、`retry_pending`、`failed`、`timed_out`、`cancelled`、`ready_for_review`、`accepted`、`needs_changes`、`paused`。本报告按代码全量列出，而非按“九态”缩写。依据：`src-tauri/src/control_core.rs:17-48`；任务包要求见 `tasks/2026-07-11-state-machine-failure-path-audit-v1.md:12-21`。

另外，`workflow_chain_runs.nodes`（按 `planned_task_id` 编址）和同步的 workflow task-node 不是同一套字符串状态：前者实际写入 `pending/running/skipped/failed/waiting_decision/completed/needs_rework/archived`，后者还有通用节点转移表。两者必须分开审计，不能把画布节点的状态误认为 work item 状态。依据：`src-tauri/src/director_agent.rs:2203-2210`、`src-tauri/src/director_agent.rs:2314-2349`、`src-tauri/src/workflow_read_model_entrypoints.rs:1449-1508`。

## 1. 方案状态机：`project-proposals.v1.json`

完整枚举为六态。依据：`src-tauri/src/types.rs:2153-2162`。交办页相位只按“是否有最新方案”进入 `authorize`，并不按方案状态细分；因此已确认/已拒绝等状态仍可先落到同一张授权卡。依据：`src/views/projects/ProjectJiaobanPanel.tsx:582-587`、`src/views/projects/ProjectJiaobanPanel.tsx:1331-1360`。

| 状态 | 谁进入 / 谁带出（代码入口） | 失败或中断后的自愈 | 用户看到什么 | 能否删/合并 |
|---|---|---|---|---|
| `draft` | 本轮生产创建不写该态；`record_decision` 仍允许从它做确认、要求修改或拒绝：`project_consultation_proposal_store.rs:165-197`、`:248-283`。 | 🔴 没有将 `draft` 自动迁至可交办态的路径。 | 🔴 最新方案存在即进授权脸；但合流命令只接受 `pending_user_confirmation`，`draft` 点击“允许并开始”会被拒：`ProjectJiaobanPanel.tsx:582-587`、`director_agent.rs:4354-4366`。 | **删除候选。** 保留兼容读取/迁移即可；当前创建一律直接写 `pending_user_confirmation`：`project_consultation_proposal_store.rs:89-121`。 |
| `pending_user_confirmation` | 咨询创建方案进入：`project_consultation_proposal_store.rs:53-130`。用户确认/要求修改/拒绝由 `record_decision` 带出：`:165-283`；主路径按钮“允许并开始”调用合流命令：`ProjectJiaobanPanel.tsx:835-876`、`:2047-2067`。 | 🔴 方案 store 锁撞到即直接返回 `project_consultation_proposal_store_locked`，没有重试：`project_consultation_proposal_store.rs:846-869`。用户从“先不做/重新出方案”返回说脸只清前端缓存，不改变旧方案状态：`ProjectJiaobanPanel.tsx:1137-1154`。 | 授权脸有“允许并开始 / 按我说的改 / 先不做”：`ProjectJiaobanPanel.tsx:2045-2067`。但后两个交办页动作不调用 `record_decision`，旧 pending 可留存。 | **保留。** 它就是唯一人闸前的方案。应在重出方案时将旧的当前 pending 显式标为 `superseded`，避免多份“待批”并存；不碰人闸。 |
| `user_confirmed` | `record_decision(Confirm)` 先创建并确认授权，再将方案写为该态：`project_consultation_proposal_store.rs:211-258`。 | 🟡 仅当用户再次点“接着跑”时，自动推进入口会尝试补全 pending global review：`ProjectJiaobanPanel.tsx:1007-1048`、`director_agent.rs:3518-3607`。无后台恢复。 | 🔴 初始仍是授权脸；再次点“允许并开始”被后端拒绝，前端随后才刷新并转“卡住/接着跑”：`ProjectJiaobanPanel.tsx:897-917`、`director_agent.rs:4361-4366`。 | **合并展示候选。** 存储审计可保留“已确认”事实，但主界面不应把它作为独立可感知脸；应直接派生为“正在自动补齐机器步骤/开干”。不改变人闸。 |
| `changes_requested` | 由 `record_decision(RequestChanges)` 写入：`project_consultation_proposal_store.rs:248-283`。 | 🔴 没有由此态生成新方案或标结旧方案的自动路径。 | 🔴 交办页仍按“有方案”显示授权卡，而合流只接 pending：`ProjectJiaobanPanel.tsx:582-587`、`director_agent.rs:4354-4366`。 | **保留为审计终态，但不应再做当前交办脸。** 新方案生成时标结该方案即可；其存在理由是保存用户的“要求修改”决定。 |
| `rejected` | 由 `record_decision(Reject)` 写入：`project_consultation_proposal_store.rs:248-283`。 | 不需要机器重试；但没有自动切回“说”或标结当前卡。 | 🔴 同样会先进入授权脸，而不是明确的“已回绝，重新说目标”脸：`ProjectJiaobanPanel.tsx:582-587`、`:1331-1360`。 | **保留为审计终态。** UI 可与 `changes_requested` 合并为一个“本单已结束 → 重新说目标”展示，不可合并掉审计决定。 |
| `superseded` | 枚举和标签存在：`types.rs:2155-2162`、`project_consultation_proposal_store.rs:799-817`；本轮生产赋值检索未发现写入点。 | 🔴 无进入者也无自动清理者。 | 运行历史能显示“被替代”：`ProjectJiaobanPanel.tsx:1484-1511`，主交办页没有专属分支。 | **实现候选。** 不要删枚举；将“重出方案”原子地标结旧 pending/changes_requested/rejected，才使它有存在理由。 |

## 2. 授权状态机：`plan-authorizations.v1.json`

完整枚举为九态。依据：`src-tauri/src/types.rs:1965-1977`。交办页没有按授权状态切脸；它只用 `proposal.status === "user_confirmed"` 决定能否给“接着跑”。依据：`src/views/projects/ProjectJiaobanPanel.tsx:1233-1236`。因此下面多数授权态是用户不可见的机器态，却会决定能否执行。

| 状态 | 谁进入 / 谁带出（代码入口） | 失败或中断后的自愈 | 用户看到什么 | 能否删/合并 |
|---|---|---|---|---|
| `draft` | 无本轮生产写入；用户确认函数允许从此态进入 pending global review：`plan_authorization_store.rs:145-184`。 | 🔴 无恢复器。 | 🔴 无独立面；guard 只返回“待用户确认”：`control_core.rs:397-415`。 | **删除候选。** 创建直接进入 `pending_user_confirmation`：`plan_authorization_store.rs:90-118`。 |
| `pending_user_confirmation` | `create_authorization` 写入：`plan_authorization_store.rs:90-129`。`record_user_confirmation` 带出至 `pending_global_boundary_review`：`:145-210`。 | 🔴 `record_decision` 先持久化创建授权、再调用确认：`project_consultation_proposal_store.rs:211-241`；两步之间失败会留下待确认授权，现有恢复器不处理此态。 | 🔴 无专属交办脸；guard 提示待确认：`control_core.rs:397-405`。 | **保留机器态，但不应长期可见。** 可做受限的同源授权回收/补确认；这会写授权事实，需确认审计语义，但不新增用户闸。 |
| `user_confirmed` | 枚举/标签存在：`types.rs:1967-1977`、`plan_authorization_store.rs:1003-1027`；本轮生产确认直接写到 `pending_global_boundary_review`：`:174-184`。 | 🔴 无写入者、无恢复器。 | 🔴 guard 与 pending global review 同样提示“待全局边界复核”：`control_core.rs:407-415`；交办页不读授权态。 | **删除候选。** 这是当前写路径跳过的中间态；迁移旧数据到 `pending_global_boundary_review` 后可收敛。 |
| `pending_global_boundary_review` **（实案：授权悬 pending）** | 用户确认把授权写入此态：`plan_authorization_store.rs:145-210`。`record_global_boundary_review` 依 review 结果带出到 `active/paused`：`:225-311`。 | 🟡 仅 `auto_advance` 开始时调用恢复器；它核对“最新、未过期、方案已确认”后补写 approved：`director_agent.rs:3518-3589`。恢复失败仅在已有 active 时放过，否则报错：`:3590-3607`。 | 🔴 恢复入口依赖用户先进入“卡住”并点“接着跑”：`ProjectJiaobanPanel.tsx:1007-1048`；没有该态专属提示或自动续跑。 | **不可删除。** 它承载边界复核事实。**自动把它改 active 会触及审批逻辑，需用户重档授权。** 可先把恢复触发移到同一次合流失败后的受限重试，但不能绕开当前批准语义。 |
| `active` | approved 边界复核写入：`plan_authorization_store.rs:248-274`。可由 `revoke_authorization` 带出：`:417-481`；到期只在读取时失效，并未改 enum 值：`:539-548`。 | 🟢 起链前反复查 active 且未过期：`director_agent.rs:3490-3515`、`:3776-3793`。 | 通过后进入绑定/运行脸；授权自身不单独展示。依据：`ProjectJiaobanPanel.tsx:877-896`、`:1363-1385`。 | **保留。** 它是唯一允许执行的安全边界，`control_core` 还要求用户确认、approved review、未过期同时成立：`control_core.rs:365-378`。 |
| `paused` | non-approved 的边界复核写入：`plan_authorization_store.rs:248-274`。 | 🔴 无复原路径；guard 明确阻断：`control_core.rs:416-423`。 | 🔴 无暂停原因专属脸；交办页不读取授权态。 | **保留。** 它记录边界未通过，不应自动恢复。补“重新说目标”落点即可；改变暂停→active 的规则属于审批逻辑，需用户重档授权。 |
| `revoked` | 任意合格撤销请求写入：`plan_authorization_store.rs:417-481`。 | 不应自愈；这是明确安全撤销。 | 🔴 guard 仅返回“已撤销”：`control_core.rs:424-431`，交办页无专属解释。 | **保留。** 不可自动合并/恢复；UI 只需正确引导新方案。 |
| `expired` | 过期以 `expires_at_ms <= now` 判定：`plan_authorization_store.rs:539-548`、`control_core.rs:384-395`；本轮未见把记录赋值为 `Expired` 的生产路径。 | 🟡 行为层会阻止执行，但存储状态仍可能是 `active`。 | 🔴 用户只会收到“失效或被撤销”，且无法区分过期：`director_agent.rs:3508-3514`。 | **二选一候选：** 要么删除 `Expired` 枚举、统一用时间派生；要么增加一次性状态迁移。不要同时保留“虚拟过期”和“实体过期”。不碰人闸。 |
| `completed` | 枚举/guard/标签存在：`types.rs:1967-1977`、`control_core.rs:440-447`、`plan_authorization_store.rs:1003-1027`；本轮未见生产写入。 | 🔴 无进入/退出者。 | 🔴 无专属面。 | **删除候选。** “一轮工作完成”已由 chain 终态表达，不应再重复为授权终态。 |

## 3. `workflow_chain_runs` 链状态机

通用链转移表覆盖 `draft/ready/running/paused/waiting_decision/completed/failed/archived/stopped`，失败/停止重开还要求 `explicit_retry_or_reopen`。依据：`src-tauri/src/workflow_read_model_entrypoints.rs:1434-1480`。现行链运行器的创建值是 `running`，节点初值是 `pending`：`src-tauri/src/workflow_chain_controller.rs:120-183`；重拆额外写 `superseded`：`src-tauri/src/director_agent.rs:1631-1681`。

| 状态 | 谁进入 / 谁带出（代码入口） | 失败或中断后的自愈 | 用户看到什么 | 能否删/合并 |
|---|---|---|---|---|
| `draft` | 通用表允许 `draft → ready`：`workflow_read_model_entrypoints.rs:1434-1447`；当前 chain runner 不创建该态：`workflow_chain_controller.rs:169-183`。 | 🔴 无恢复器。 | 🔴 运行画布会把未知值归为“等待”：`ProjectJiaobanPanel.tsx:113-141`。 | **删除/迁移候选。** 当前实际链没有生产进入者。 |
| `ready` | 仅通用表的 `ready → running`：`workflow_read_model_entrypoints.rs:1434-1447`；起链实际直接写 `running`：`workflow_chain_controller.rs:169-183`。 | 🔴 无恢复器。 | 🔴 无专属交办脸。 | **删除候选。** 与“尚未起链”重复。 |
| `running` | 新建或续跑写入：`workflow_chain_controller.rs:120-183`；每个任务启动也写入：`director_agent.rs:2419-2435`。可到 stopped/failed/waiting_decision/completed。 | 🟢 进程中断后再次起链会复用 `running/stopped`，已完成节点跳过，未完成节点继续：`workflow_chain_controller.rs:120-167`、`director_agent.rs:2220-2245`、`:2277-2282`。 | 运行脸轮询并提供“停下”：`ProjectJiaobanPanel.tsx:558-580`、`:2544-2589`。 | **保留。** 它是唯一正常机器运行态。 |
| `paused` | 通用表允许 `running ↔ paused`：`workflow_read_model_entrypoints.rs:1434-1447`；当前 chain runner 无生产赋值。 | 🔴 无恢复器。 | 🔴 无专属面，画布会落“等待”：`ProjectJiaobanPanel.tsx:123-141`。 | **删除/迁移候选。** 当前“用户停链”实际写 `stopped`，不要双态表达暂停。 |
| `waiting_decision` | worker 求助、或返工预算耗尽时，节点和链都写入：`director_agent.rs:2614-2651`、`:2867-2907`。通用表允许 `waiting_decision → running`：`workflow_read_model_entrypoints.rs:1439-1440`。 | 🔴 没有交办页处置入口；失败处置命令只接 `failed/needs_rework` 节点：`director_agent.rs:1060-1086`。 | 🔴 运行脸翻转规则不含 `waiting_decision`，而画布也会把它归“等待”：`ProjectJiaobanPanel.tsx:1156-1165`、`:123-141`。 | **保留。** 求助需要人决定；但必须建立一个明确的“待你决定”脸及允许的动作，不能伪装为运行中。改变决定范围不自动化，不碰人闸。 |
| `completed` | 所有节点收口时 `finalize_chain_run` 写入：`workflow_chain_controller.rs:517-536`；任务级完成也写 node：`director_agent.rs:2716-2769`。可归档：`workflow_read_model_entrypoints.rs:1441-1444`。 | 不需要恢复。 | 🟢 正常“做好了/已交货”：`ProjectJiaobanPanel.tsx:2827-2947`。 | **保留。** 正常终态。 |
| `failed` **（实案：post-confirm 死脸的一半）** | 会话创建失败或 worker 失败即写 failed 并停链：`director_agent.rs:2477-2539`；基础 runner 同样失败即停：`workflow_chain_controller.rs:465-513`。 | 🟡 tier-1 偶发早退会合法复位后仅重试一次：`director_agent.rs:2553-2569`；其他失败不自动恢复。 | 🔴 编排层仍返回 `stage: "ran"`：`director_agent.rs:4060-4091`；前端把 `failed` 当作“done”触发条件：`ProjectJiaobanPanel.tsx:1156-1165`，完成脸默认显示“已交货”：`:2892-2904`。 | **不可删除。** 失败必须留下；应把“链未完整完成”映射为 blocked/待处置，而不是交货。该修复不改人闸。 |
| `stopped` **（实案：post-confirm 死脸的一半）** | 用户停链在下一任务边界正式标 `stopped`：`workflow_chain_controller.rs:547-601`、`director_agent.rs:2283-2308`；runaway 上限和返工也会停链：`director_agent.rs:2392-2417`、`:2827-2865`。 | 🟢 再次起链可断点续：`workflow_chain_controller.rs:120-167`。 | 🔴 当前仍被前端翻为“done/做好了”：`ProjectJiaobanPanel.tsx:1156-1165`。 | **保留。** 这是可恢复的中断，展示应为“已停下，可接着跑”，而不是“交货”。 |
| `archived` | 主管对 failed/needs_rework 节点选 archive 时写入链和节点：`director_agent.rs:1276-1323`。 | 不应自愈；这是主动结束。 | 🟡 交办页的处置代码人为构造 `stage: "ran"` 并切 done：`ProjectJiaobanPanel.tsx:1094-1115`，但未给“已结束”专属语义。 | **保留。** 需要终态展示与“已交货”分离。 |
| `superseded` | re-plan 前把旧 `running/stopped` 链正式标结：`director_agent.rs:1631-1681`。 | 不需要恢复；新链已开始。 | 🟡 历史可显示“被替代”：`ProjectJiaobanPanel.tsx:1484-1511`；当前运行不应再选它。 | **保留。** 它是跨轮审计，不是主流程中间态。 |

## 4. `workflow_chain_runs.nodes`：按 `planned_task_id` 的链节点

这是交办运行画布最应忠实呈现的一层。当前节点状态由链驱动写入：启动 `running`，非 prepared/异常写 `skipped`，会话/worker异常写 `failed`，求助写 `waiting_decision`，终标写 `completed/needs_rework`。依据：`src-tauri/src/director_agent.rs:2314-2539`、`:2614-2651`、`:2716-2907`。前端却只识别五态，未知状态全部回退为 `pending`：`src/views/projects/ProjectJiaobanPanel.tsx:113-163`。

| 状态 | 谁进入 / 谁带出 | 自愈 | 用户脸与动作 | 删/合并判断 |
|---|---|---|---|---|
| `pending` | 新/补链节点初值：`workflow_chain_controller.rs:144-183`；任务启动改 `running`：`director_agent.rs:2419-2435`。 | 🟢 链续跑时未完成节点会重新处理：`director_agent.rs:2277-2282`。 | 🟢 画布显示“等待”：`ProjectJiaobanPanel.tsx:113-121`。 | **保留。** 是尚未执行的最小态。 |
| `running` | 链驱动开始任务写入：`director_agent.rs:2419-2435`。完成、失败、求助、返工均由后续分支带出。 | 🟢 续跑只跳过 completed，因此中断中的任务会再处理：`director_agent.rs:2277-2282`。 | 🟢 画布“正在执行”、主脸有停下：`ProjectJiaobanPanel.tsx:115-121`、`:2544-2589`。 | **保留。** |
| `skipped` | task 非 `prepared` 或缺 node/work item 时写：`director_agent.rs:2314-2389`。 | 🟡 下次续跑不会把它当 completed，可能再进入处理；但没有针对 skipped 的显式恢复语义：`director_agent.rs:2277-2282`。 | 🔴 前端没有 `skipped` 映射，会显示“等待”：`ProjectJiaobanPanel.tsx:123-141`。 | **合并候选。** 不要把“未授权/缺绑定/异常”压成 skipped；保留原因在 outcome，节点显示为 blocked/needs_binding 或失败。 |
| `failed` | 会话创建失败或 worker 执行失败时写：`director_agent.rs:2477-2539`。失败处置可 retry/change_session/rework/archive：`:1060-1323`。 | 🟡 仅 tier-1 早退自动重试一次：`:2553-2569`；其余须明确处置。 | 🟡 画布显示“失败”，但链仍可能被包装为“已交货”：`ProjectJiaobanPanel.tsx:115-121`、`:1156-1165`。 | **保留。** 修复上层完成映射。 |
| `waiting_decision` | worker 求助或返工预算耗尽写入：`director_agent.rs:2614-2651`、`:2867-2907`。 | 🔴 通用节点规则可 `waiting_decision → running`，但 UI/failed-action 没有对应入口：`workflow_read_model_entrypoints.rs:1454-1456`、`director_agent.rs:1060-1086`。 | 🔴 被前端回退成“等待”，没有决定动作：`ProjectJiaobanPanel.tsx:123-141`。 | **保留。** 但它是必须可见的人工决定态；不能拿普通“等待”合并。 |
| `completed` | 主管终标通过写入：`director_agent.rs:2716-2769`。 | 不需恢复。 | 🟢 画布“已完成”：`ProjectJiaobanPanel.tsx:115-121`。 | **保留。** |
| `needs_rework` | 主管终标退回或显式 rework 写入：`director_agent.rs:1162-1274`、`:2772-2865`。可由主管显式 retry/reopen 或 archive 带出：`workflow_read_model_entrypoints.rs:1497-1504`。 | 🟡 不自动重跑；这是正确的人类处置边界。 | 🟢 完成脸会提供“接着跑/换会话/退回重拆/结束这单”：`ProjectJiaobanPanel.tsx:2916-3002`。 | **保留。** |
| `archived` | 显式 archive 写入：`director_agent.rs:1276-1323`。 | 不应自愈。 | 🔴 画布未知值回退“等待”，完成脸也没有“已归档”标签：`ProjectJiaobanPanel.tsx:123-141`、`:2892-2904`。 | **保留审计，合并展示。** 应归到“本单已结束”，绝不能显示为等待或交货。 |

### 同步 workflow task-node 的补充枚举

`update_work_item_state_at` 会把 work item 的下一状态同步给对应 workflow node：`src-tauri/src/workflow_run_dispatch_entrypoints.rs:440-524`。通用 node 转移表还列出 `not_started`、`waiting`、`waiting_permission`、`reviewing`、`passed`、`returned`、`paused`、`cancelled` 等十四态：`src-tauri/src/workflow_read_model_entrypoints.rs:1449-1508`。

这些状态的四问结论可压缩为：

| 通用 node 状态 | 进入/退出与自愈 | 用户面 | 简化判断 |
|---|---|---|---|
| `not_started`、`waiting` | 通用表仅允许 `not_started→waiting→running`，当前交办链不用这两个字符串：`workflow_read_model_entrypoints.rs:1449-1471`。 | 🔴 交办画布无映射。 | **候选迁移。** 和链节点 `pending` 语义重叠。 |
| `waiting_permission` | `running↔waiting_permission`：`workflow_read_model_entrypoints.rs:1452-1454`；无交办页恢复器。 | 🔴 只会落“等待”。 | **保留安全态，不可自动越过。** 需要解释与授权入口；任何自动放行触及人闸/审批，需用户重档授权。 |
| `reviewing`、`passed`、`returned` | `running→reviewing→passed/returned→running`：`workflow_read_model_entrypoints.rs:1457-1460`。 | 🔴 交办画布无对应展示。 | **可从交办主路径隐藏/合并展示**，但不可删审计，因为 review 与完成权限有角色限制：`:1488-1504`。 |
| `cancelled` | `waiting_decision→cancelled`：`workflow_read_model_entrypoints.rs:1454-1457`。 | 🔴 无取消脸。 | **保留终态，合并为“已结束”。** |
| `failed`、`needs_rework`、`archived` | 转移与 project_director 权限见：`workflow_read_model_entrypoints.rs:1461-1504`。 | 与链节点同名但前端只可靠链节点映射，仍有失真。 | **保留。** 统一两层展示键而非再造状态。 |
| `paused` | `running↔paused`：`workflow_read_model_entrypoints.rs:1468-1469`。 | 🔴 无面。 | **候选与 chain `stopped` 收敛。** 若保留需说明是谁暂停、谁恢复。 |
| `skipped` | `waiting→skipped`：`workflow_read_model_entrypoints.rs:1470-1471`。 | 🔴 无面。 | **与链节点结论相同：** 不要用于掩盖 blocked/needs_binding 原因。 |

## 5. work item 状态机（实际 12 态）

所有状态写入都经 `update_work_item_state_at` 先验证合法转移、更新对应 node、写审计并原子写回：`src-tauri/src/workflow_run_dispatch_entrypoints.rs:440-524`。因此它是底层真机状态，交办页却没有逐项展示，必须把它与链节点的派生态分开。

| 状态 | 谁进入 / 谁带出 | 自愈 | 用户看到什么 | 删/合并判断 |
|---|---|---|---|---|
| `draft` | 任务包草稿创建写入：`workflow_state_lifecycle_task_package.rs:359-415`；只能到 `ready_to_dispatch`：`control_core.rs:25-48`。 | 🔴 无自动补齐任务包。 | 🔴 交办页不直接显示。 | **保留。** 未完成任务包不能派发。 |
| `ready_to_dispatch` | 从 draft 或 needs_changes/paused 来：`control_core.rs:25-48`；派发准备和启动都要求此态：`:59-76`。 | 🟢 tier-1 早退/重试可合法复位回此态：`director_agent.rs:1359-1393`。 | 🔴 不直接显示，只由链的 pending/running 间接呈现。 | **保留。** |
| `running` | dispatcher 从 ready_to_dispatch 进入；合法出口含权限等待、重试、失败、超时、取消、待回收：`control_core.rs:25-48`。 | 🟡 残留工作项会在 re-plan 前被合法重置，但 accepted/cancelled 不接管：`director_agent.rs:1577-1617`。 | 🔴 只看链画布，work item 自身不可见。 | **保留。** |
| `waiting_for_permission` | running 进入，之后可 running/failed/cancelled：`control_core.rs:28-37`。 | 🔴 无交办 UI 的权限决策入口。 | 🔴 用户只可能看到泛化“卡住/等待”。 | **保留安全态。** 自动通过会碰权限/人闸，需用户重档授权。 |
| `retry_pending` | running/failed/timed_out 进入，之后 running/failed：`control_core.rs:29-42`。 | 🟡 有合法边，但没有统一调度器保证再次执行；只有 tier-1 路显式重试一次：`director_agent.rs:2553-2569`。 | 🔴 无单独显示。 | **合并展示候选。** 用户只需看到“系统正在重试”，不需看到该键。 |
| `failed` | running 或 waiting_for_permission/retry_pending 进入：`control_core.rs:30-39`。可到 retry_pending/needs_changes：`:39-46`。 | 🟡 tier-1 早退自动复位一次，其他需处置：`director_agent.rs:2553-2569`。 | 🔴 上层可能错误地显示“已交货”，见第 3 节。 | **保留。** |
| `timed_out` | running 进入，后续可 retry_pending/needs_changes：`control_core.rs:31-42`。 | 🟢 有一次自动重拆预算，且第二次回到人：`director_agent.rs:4104-4166`。 | 🟡 work item 不显；用户只能看链结果/警告。 | **保留。** 超时与普通失败的自动恢复策略不同。 |
| `cancelled` | running 或 waiting_for_permission 进入；之后可 needs_changes：`control_core.rs:32-43`。 | 不自动恢复正确，但可人工退回。 | 🔴 无“已取消”专属脸。 | **保留终态，合并展示为“已结束”。** |
| `ready_for_review` | running 成功回传进入；离线回传也只允许进入此态：`control_core.rs:33-45`、`:88-103`。可到 accepted/needs_changes。 | 🔴 进程死在此态可在 re-plan 前被接管，但非默认路径可能失败：`director_agent.rs:1577-1617`。 | 🔴 无独立复核脸。 | **保留。** 它隔离执行完成与验收完成。 |
| `accepted` | ready_for_review 经总指导回收进入：`control_core.rs:105-135`、`:44-45`。 | 不自动恢复正确；残料接管明确不碰：`director_agent.rs:1614-1617`。 | 🔴 无 work item 专属面。 | **保留终态。** |
| `needs_changes` | failed/timed_out/cancelled/ready_for_review 进入，之后 ready_to_dispatch：`control_core.rs:39-47`。 | 🟡 reset helper 会走到 ready_to_dispatch：`director_agent.rs:1359-1413`。 | 🟡 仅当它同时投影为链 `needs_rework` 时才有四选一处置。 | **保留。** 它是失败与可再派发之间的合法缓冲。 |
| `paused` | 任意非 accepted 可进入；只可到 ready_to_dispatch：`control_core.rs:21-22`、`:46-48`。 | 🔴 无统一恢复器。 | 🔴 无单独面。 | **候选收敛。** 若只是机器暂停，可并入 `needs_changes`；若承载人工暂停，需先补“谁暂停/谁恢复”的事实。 |

## 6. 三个实案复核

| 实案 | 已有修补 | 未收口事实 | 判定 |
|---|---|---|---|
| `store_locked` 锁竞态一撞即停 | 授权 store 已做 5 次、每次 100ms 的有限重试，并有“并发写者释放”和“耗尽提示”测试：`plan_authorization_store.rs:18-25`、`:1055-1092`、`:1119-1171`。 | 方案 store 仍只做一次 `create_new`，锁已存在立即抛 `project_consultation_proposal_store_locked`：`project_consultation_proposal_store.rs:846-869`。而方案确认正是人闸入口的第一把锁：`:180-184`。 | **P1 自愈缺口。** 先把相同的有限退避复制到 proposal store；不改变确认/审批结论，不触及人闸。 |
| 合流 post-confirm 死脸 | post-confirm 失败会写 stopped 审计：`director_agent.rs:4392-4470`；前端 catch 会刷新方案 store、给“接着跑”线索：`ProjectJiaobanPanel.tsx:897-917`。 | auto-advance 仍把任何 `DirectorChainOutcome` 包为 `stage: "ran"`，包括 `failed/stopped/waiting_decision`：`director_agent.rs:4060-4091`；前端会将 failed/stopped 翻成 done：`ProjectJiaobanPanel.tsx:1156-1165`。 | **P0 主流程阻塞/误导。** 不是补按钮，而是返回结果必须表达“完整完成 / 中断可续 / 等待决定 / 失败”。不触及人闸。 |
| `pending_global_boundary_review` 悬挂 | `restore_pending_global_boundary_review_after_confirm` 已限制最新授权、未过期、同 proposal/workflow 才补记：`director_agent.rs:3518-3589`。 | 该恢复只在用户再点“接着跑”后才运行：`ProjectJiaobanPanel.tsx:1007-1048`、`director_agent.rs:3765-3793`。直接把 pending 自动批准会改变当前“用户演全局主管”的审批事实。 | **P0.5。** 将同一次用户点击内的短暂失败做受限、幂等续跑可讨论；完全后台批准属于审批逻辑变化，**需用户重档授权**。 |

## 7. 简化提案（先删/合并，再谈补按钮）

| 候选 | 现状 | 为什么可删/合并 | 删后/收敛后的流程 | 是否碰人闸或审批 |
|---|---|---|---|---|
| S1：去除授权 `draft`、`user_confirmed`、`completed` 的运行时分支 | 三态被 enum/label/guard支持，但当前生产创建或确认不写其中任一状态：`plan_authorization_store.rs:90-129`、`:145-184`；`completed`没有写入点。 | 它们增加了读模型分支，却不增加可执行行为。 | 旧盘迁移：`draft→pending_user_confirmation`，`user_confirmed→pending_global_boundary_review`，移除 `completed` 或改由链终态表示。 | 不碰人闸；需做数据迁移和回归。 |
| S2：将方案 `user_confirmed` 从主界面相位中移除 | 当前“有方案即授权脸”，已确认方案还需用户再次点击才获得“接着跑”：`ProjectJiaobanPanel.tsx:582-587`、`:897-917`。 | 这是存储审计事实，却被误做成用户可感知中间态。 | 保留存储状态；UI 以关联授权状态派生“系统正在继续/已卡住”，不再渲染第二次“允许并开始”。 | 不碰唯一人闸。 |
| S3：为重出方案落实 `superseded` | 枚举/历史已有 `superseded`，但方案创建未标结旧方案：`project_consultation_proposal_store.rs:53-130`、`:799-817`。 | 当前可无限积累旧 `pending`，使“最新方案”之外的状态没有收口。 | 新方案写入时，将同 project/workflow 的前一当前方案原子标 `superseded`，保留审计。 | 不碰人闸；需定义哪些终态可被替代。 |
| S4：链 `ready/draft/paused` 收敛 | 通用链表有这三态，实际 runner 直接创建 `running`、用户停链写 `stopped`：`workflow_read_model_entrypoints.rs:1434-1447`、`workflow_chain_controller.rs:169-183`、`:547-601`。 | 同一条链有两组“没开始/暂停”词，却没有完整实现者。 | 运行记录只保留 `running/stopped/waiting_decision/completed/failed/archived/superseded`；旧值迁移到明确目标。 | 不碰人闸。 |
| S5：统一“未完成”结果，不再用 `ran` | `stage: "ran"` 当前同时覆盖完整完成、失败、停链、待决定：`director_agent.rs:4060-4091`。 | 这是直接导致“已交货”死脸的语义坍缩。 | 返回 `completed / interrupted / failed / waiting_decision`，前端只将 completed 映射为 done。 | 不碰人闸。 |
| S6：把 node `skipped` 从展示状态改为原因化结果 | 非 prepared/缺实体时把节点写 `skipped`，UI 却显示“等待”：`director_agent.rs:2314-2389`、`ProjectJiaobanPanel.tsx:123-141`。 | “跳过”没有告诉用户是越界、缺会话还是数据损坏。 | 链节点保留审核字段；主页面显示 `blocked` 或 `needs_binding` 与可行动原因。 | 不碰人闸；越界仍不自动放行。 |

## 8. 必须保留状态的自愈补齐清单

| 优先级 | 缺口与最小修法 | 人闸/审批影响 |
|---|---|---|
| P0 | **链结果语义。** `run_auto_advance...` 依据 `DirectorChainOutcome.stopped_reason` 和链 state 返回 completed/interrupted/failed/waiting_decision；前端仅 completed 进入“已交货”，其余进入卡住或待决定脸。证据：`director_agent.rs:4060-4091`、`ProjectJiaobanPanel.tsx:1156-1165`。 | 不碰人闸或审批。 |
| P0 | **`waiting_decision` 的处置面。** 将链/node 的 waiting_decision 显示为“待你决定”，列出受控的继续、重做、结束动作；不能默认重跑。证据：`director_agent.rs:2614-2651`、`workflow_read_model_entrypoints.rs:1491-1504`。 | 不碰“允许并开始”；人决定本身必须保留。 |
| P0.5 | **pending global review 的同击恢复。** 只对已由本次“允许并开始”产生、同 proposal/authorization、未过期的 pending 做幂等恢复；失败再给卡住脸。现有恢复的安全筛选可复用：`director_agent.rs:3527-3607`。 | **会写 approved review，触及审批逻辑。** 完全自动批准或扩大条件必须由用户重档授权。 |
| P1 | **proposal store 锁退避。** 与 authorization store 同样有限重试、保持最终可行动错误；不要把锁等待变成第二人闸。证据：`project_consultation_proposal_store.rs:846-869`、`plan_authorization_store.rs:1055-1092`。 | 不碰人闸或审批。 |
| P1 | **pending_user_confirmation 授权孤儿回收。** 对“同源 proposal 仍 pending、没有用户确认”的新建授权做幂等删除/失效或在同一次确认内补完；不要让它抢占 read model。证据：`project_consultation_proposal_store.rs:211-241`、`plan_authorization_store.rs:145-184`。 | 需要确认撤销/补确认的审计措辞；不新增人闸。 |
| P1 | **运行画布补足状态。** 至少显式处理 `skipped/waiting_decision/archived`，不可把未知值回落为 `pending`。证据：`ProjectJiaobanPanel.tsx:123-141`。 | 不碰人闸或审批。 |
| P2 | **expired/status 实体化取舍。** 选择派生过期或实体迁移，删除另一份语义。证据：`plan_authorization_store.rs:539-548`、`control_core.rs:384-395`。 | 不碰人闸。 |
| P2 | **work item 等待权限可见化。** 在链 blocked 原因中传递 `waiting_for_permission`，但不自动批准。证据：`control_core.rs:28-37`。 | 自动批准会触及权限/审批，**需用户重档授权**。 |

## 9. 按主流程被卡概率的修复次序

1. **P0：修“失败/停止被报成已交货”。** 每一次真实 worker 失败、用户停链、求助、返工预算耗尽都会走这个结果坍缩路径，且当前证据显示其影响结果脸：`director_agent.rs:4060-4091`、`ProjectJiaobanPanel.tsx:1156-1165`。
2. **P0：给 `waiting_decision` 正确的人工决定脸。** 它已经能由 worker 求助和预算耗尽进入，但当前没有对等出口：`director_agent.rs:2614-2651`、`:2867-2907`、`:1060-1086`。
3. **P0.5：限制条件下的 post-confirm / pending global review 同击续跑。** 现有恢复已证明能安全校验关联关系，但需要决定是否允许同一次人闸内补记审批：`director_agent.rs:3518-3607`。
4. **P1：proposal store 锁重试。** 授权锁已修，方案锁仍是一撞即停；它位于用户唯一人闸的前半段：`project_consultation_proposal_store.rs:180-184`、`:846-869`。
5. **P1：状态展示收敛。** 先补 `skipped/archived` 与授权非 active 的人话，再消除 dead enum；否则状态机再正确也会在 UI 被误译：`ProjectJiaobanPanel.tsx:113-163`、`:1233-1236`。
6. **P2：删除或迁移无生产写入的 enum 状态。** 这降低后续修复时的分支数量，但不比主路径语义错误紧急：`types.rs:1967-1977`、`:2155-2162`。

## 10. 第一段交接边界

- 本报告只新增本文件；未改代码、`CURRENT.md`、`decisions/`，符合任务包第一段只读死线：`tasks/2026-07-11-state-machine-failure-path-audit-v1.md:5-8`、`:33-35`。
- 第二段建议先由总指导抽样核对三实案，再由用户拍板：是否授权把 `pending_global_boundary_review` 在同一次人闸内自动补记为 approved；其余 P0/P1 项不改变“允许并开始”人闸本身。任务包交接要求：`tasks/2026-07-11-state-machine-failure-path-audit-v1.md:37-43`。
