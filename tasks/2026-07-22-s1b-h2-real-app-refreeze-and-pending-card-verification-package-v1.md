# 任务包：S1B-H2-R2 修后 binary 重冻结与真实 App Pending 卡验收 v1

- 日期：2026-07-22
- 状态：**已出包，未执行；待用户另行明确现场开工令**
- 类型：真实 App 高风险验收；不含代码修复
- 权威 kickoff：`handoffs/2026-07-22-s1b-h2-real-app-refreeze-and-pending-card-verification-kickoff-v1.md`
- 上游合同：`tasks/2026-07-19-s1b-h2-supervisor-syn-natural-information-flow-package-v1.md`
- 历史现场证据：`evidence/2026-07-19-m5-f1-r1-live-reseed-and-s1b-h2-real-app-verification-v1.md`

## 0. 本包结论

S1B-H2 的代码与离线面已有通过证据，但真实 App 验收尚未通过。知识库生产写路修复后，相关源码与构建输入已经变化，因此 2026-07-20 冻结的 debug binary 及其 hash 不能继续作为本轮验收对象。

下一次获授权后只做一件事：重建并重冻结当前源码对应的裸 debug binary，在真实 App 中完成一组新鲜的两回合对话，确认主管通过唯一 MCP 工具生成恰好一张 Pending 卡，然后立即停止。不得批准卡片、不得启动 chain、不得修改测试项目。

本包本身不构成现场执行授权。2026-07-22 此前的“可以做”已被用户后续的“不执行，只写任务包和 kickoff”覆盖，后续不得沿用。

## 1. 产品合同

1. 对话框只负责自然对话。用户可以像普通聊天一样先讨论需求。
2. 只有用户明确要求“出方案”，或主管在对话中明确判断已经可以出方案时，才可调用 MCP 生成结构化方案卡。
3. Syn 负责在用户、主管 Codex/其他 agent 与 MCP 之间自然传递信息，并负责权限、校验、幂等、审计和权威事实投影；它不应把工具错误原文投影给用户。
4. 产品运行时只允许预批准唯一工具 `supervisor_orchestrator.submit_proposal`。不得开放 wildcard、默认工具集、第二个工具或全自动执行权限。
5. 一张 Pending 卡只表示“待用户确认方案”，不表示执行批准。产生卡片后必须停止。

## 2. 已知事实

1. S1B-H2 已具备单工具配置、会话/工具/卡片分层、错误人话、服务端幂等及 invalid-resume 单次轮转的源码和离线证据。
2. M5-F1-R1 已恢复 blocked fallback 的单 CAS 语义，并已有离线覆盖；历史真实现场曾完成 DB-primary 恢复。
3. 知识库 audit 已改走固定 Batch 2 生产写路。已有证据记录：`cargo check --lib`、Tauri debug build、完整外层 Rust、M5-B/M5-C/M5-F1、TypeScript 与离线交互均通过；shape 仍是历史 `13 errors / 5 warnings / 5 infos`，本修复零新增。
4. 历史 H2 现场只成功写入第一句 canonical；主管因当时现场条件未完成回复，第二句未发送，也没有新增方案卡或 chain。
5. 历史基线 `proposal=74 / Pending=17 / chain=40` 只可作旧证据引用，不得当作本轮当前基线。
6. 当前工作树有既有未提交改动。本包禁止 reset、clean、stash、覆盖或顺手整理这些改动。

## 3. 当前冻结的相关源码 hash

下列 hash 是出包时的只读快照，只用于下一轮 Gate 0 检测相关代码是否漂移：

```text
279a728ee6487f7e8afecf5e81ad4df1dccb06b2b68179fafc2444a5bce3cb92  prototypes/productized-desktop-shell/src-tauri/src/knowledge_vault.rs
d15bbdb16dee75dc415d1e4e050b275eb89e75a940f1a93c281e7845693519f0  prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_storage_mode_m5b_tests.rs
86bae55ccc9cd9e1499eae9396b987ea9ef18a31c43f872ad97c0e5e79db2da3  prototypes/productized-desktop-shell/src-tauri/src/supervisor_resident_oneshot_session.rs
d13a9ac9b5b4d0ed9e8fb9d55e713495be48ddc8073bc0b742e946a2aaa56845  prototypes/productized-desktop-shell/src-tauri/src/mcp/supervisor_orchestrator_resident_session.rs
6130ee77e3b6ce4a3730fd049adc2b9bc18718ae49d2401af8d2c035d351962b  prototypes/productized-desktop-shell/src-tauri/src/mcp/supervisor_orchestrator_submit_proposal.rs
47ac7053f55403c55d0a467703937b865c01fe001413bec81dc9776e46558bd2  prototypes/productized-desktop-shell/src/views/projects/jiaoban/useJiaobanConversationState.ts
82b15432fa35e47b4b6bcc26cab1a20906f8f307b491b8d326602b1bb7ea9c58  prototypes/productized-desktop-shell/src-tauri/src/supervisor_resident_oneshot_tests.rs
```

任一相关文件在开工前发生语义漂移，都必须停止并报告 `BLOCKED_DIRTY_OVERLAP`；不得拿旧验证替新源码背书。

## 4. 未知项

以下事实只能在下一次明确授权后的 Gate 0 只读检查中确认：

- Workbench、dev server、probe、构建或测试进程是否全部关闭；
- workflow-state、DB、WAL、SHM、JSON、registry 是否仍有持有者；
- 真实 store 当前 revision、proposal/Pending/chain 数、canonical 消息数、generation 与 thread；
- 当前 registry 和相关进程基线；
- 当前裸 debug binary 的 hash、大小和 mtime；
- 固定测试项目的当前 git 状态与业务文件 hash；
- 主管额度、真实 Codex/MCP 可用性和本轮是否发生一次合法 invalid-resume 轮转。

不得把这些未知项写成已通过。

## 5. 唯一现场开工令

只有用户在后续新消息中明确给出以下授权，才可进入 Gate 0：

> S1B-H2-R2 开工；授权重建并重冻结当前 debug binary，在真实 App 中发送两句验收话，只允许 supervisor_orchestrator.submit_proposal 生成一张 PendingUserConfirmation 卡；不得批准卡、不得启动 chain、不得改测试项目。

任何较早的授权、含糊的“继续”“可以”或本任务包的存在，都不能替代这条现场开工令。

## 6. 允许范围

获开工令后允许：

- 只读检查仓库、任务/证据、相关源码、进程、文件持有者、真实 store 与固定测试项目；
- 在 `prototypes/productized-desktop-shell` 运行既定 debug build，产生正常 `dist/`、`target/` 构建产物；
- 启动本轮新冻结的裸 debug binary；
- 在固定测试项目中发送本包规定的两句验收话；
- 允许产品正常写入 canonical 对话、审计记录及恰好一张 Pending 方案卡；
- 保存原始验收证据，并在验收结束后更新新的 evidence、`CURRENT.md` 与 catch log。

## 7. 禁止范围

- 不修改 Rust、TypeScript、配置、schema、依赖、command、sidecar 或 MCP server；
- 不 reset、clean、stash、stage、commit 或 push；
- 不执行 apply、reseed、迁移、生产恢复或备份删除；
- 不批准方案卡，不启动 chain、worker 或任何项目执行；
- 不修改固定测试项目的业务文件；
- 不添加第二个 MCP 工具，不扩大 approval、sandbox、path-lock、写根或清理权限；
- 不为证明幂等而重发第二句；只允许刷新/重新进入一次后观察；
- 不自行 kill 残留进程。持有者或残留未清零时，止损并请用户关闭；
- 不把 stderr、工具原始错误、内部审计 detail 投影给用户；
- 构建或真实 App 暴露代码缺陷时不得现场修码，另出修复包。

## 8. 执行闸门

### Gate -1：新授权

- 收到第 5 节的唯一现场开工令。
- 未收到则维持“已出包，未执行”。

### Gate 0：只读现场与脏基线

1. 用户先关闭所有 Workbench/dev 实例。
2. 只读确认以下目标无持有者：workflow-state、DB、WAL、SHM、JSON、registry。
3. 只读确认无本轮相关 Workbench、Tauri dev/probe、Vite、cargo-tauri 构建残留；registry 为空。
4. 记录 HEAD、`git status --porcelain`、staged 集与本包第 3 节全部 hash。
5. 保留既有脏项，不清理、不改写。相关 hash 漂移则 `BLOCKED_DIRTY_OVERLAP`。
6. 记录真实 store 当前 revision，以及 `proposal=B0`、`Pending=P0`、`chain=C0`、canonical user/assistant 数、generation、thread 与审计基线。
7. 记录固定测试项目的 git 状态和验收涉及文件 hash。
8. 任一 App/dev/holder 未清零即停止；不创建“已安全关闭”的虚假结论。

### Gate 1：重建与重冻结当前 binary

在 `prototypes/productized-desktop-shell` 运行既定命令：

```bash
../tauri-capability-probe/.tauri-cli/bin/cargo-tauri build --debug
```

必须同时满足：

- build exit 0；
- 第 3 节相关源码 hash 在 build 前后不变；
- 冻结 `src-tauri/target/debug/codex-governance-workbench` 的绝对路径、SHA-256、大小与 mtime；
- 启动对象是这个裸 binary，不是历史 `.app` 或旧 hash；
- build 后仍无未解释的持有者或残留进程。

build 失败即停止，只记录证据，不修代码。

### Gate 2：第一回合自然对话

1. 启动 Gate 1 冻结的裸 binary，进入固定测试项目交办页。
2. 以一个新的用户 turn 发送精确文本：

   `我想给这个游戏里的标题改成小马里奥`

3. 这不是旧 transport 的重试，不复用旧 `client_request_id`。
4. 等待 canonical 已记录并收到主管自然回复，记录 message/event/thread/generation 与审计链。
5. 在主管回复前绝对不发送第二句。
6. 如果第一回合没有 canonical、没有主管回复、出现 quota/transport/tool 原始错误或需要人工批准，立即停止。
7. 第一回合允许 invalid-resume 最多发生一次 rotate→initial；若轮转，冻结新 thread。不得递归轮转。

### Gate 3：明确出方案与单卡落地

只有 Gate 2 已收到主管自然回复，才发送精确文本：

`按这个出方案`

随后必须证明：

- 第二句 canonical 已记录；
- 主管通过唯一工具 `supervisor_orchestrator.submit_proposal` 到达真实 handler；
- 主管有自然 final，不将工具原始错误投影给用户；
- 相对 Gate 0，proposal 恰好 `B0+1`，Pending 恰好 `P0+1`；
- chain 始终等于 `C0`；
- 第一回合成功后的 thread 与第二回合保持同一新 thread；
- 没有出现需要点击的工具批准提示；若出现，停止且不点击。

看到一张匹配的 Pending 卡后立即停止，不打开批准动作，不继续执行。

### Gate 4：刷新幂等

1. 只允许刷新或离开再进入交办页一次。
2. 刷新后仍只能有这一张新增 Pending 卡。
3. 不重发“按这个出方案”，不使用重复用户动作伪造幂等证据。

### Gate 5：收尾对账

1. 正常关闭 App；若出现孤儿进程，只记录并报告，不自行 kill。
2. 最终只读对账：
   - 本轮 user canonical 增量应为 `+2`；
   - supervisor 回复与实际观察一致；
   - proposal `B0+1`、Pending `P0+1`、chain `C0`；
   - refresh 后无重复卡；
   - 固定测试项目 git 状态和业务文件 hash 不变；
   - DB/JSON 投影若适用，应 lag 0 且无新增 degradation；
   - registry 与相关进程无本轮孤儿残留。
3. 任一对账不符，H2-R2 判失败或阻断，不得宣称真实验收通过。

### Gate 6：证据与权威状态

建议证据落点：

- 原始证据目录：`evidence/raw/2026-07-22-s1b-h2-r2-real-app/`
- 验收证据：`evidence/2026-07-22-s1b-h2-r2-real-app-pending-card-verification-v1.md`

验收结束后才允许最小更新：

- `CURRENT.md`：写真实通过、失败或阻断结论；
- `docs/harness-catch-log.md`：只在发现新拦截时 EOF 追加；
- 新 evidence：记录命令、exit code、binary hash、基线/末态 delta、thread/generation、卡片键、项目 hash、进程与持有者对账。

仍不得 stage 或 commit。

## 9. 失败真话与停机矩阵

| 现场事实 | 用户面/结论 | 后续动作 |
|---|---|---|
| 第一条 canonical 未记录 | “这句没送到主管——稍后再试一次。”为事实真话 | 不发第二句，停止 |
| 第一条已记录但主管未回复 | 明确“已送到，但主管还没完成回复” | 不发第二句，停止 |
| quota 或 transport 阻断 | 记录已完成到哪一层，不冒充方案完成 | 不发后续消息，停止 |
| 主管回复成功但第二回合未落卡 | “主管已回复，但方案卡未生成” | 不重发，不补卡，停止 |
| 多于一张新增卡 | 幂等失败 | 不删卡、不批准，停止 |
| chain 增长、项目文件变化或出现 worker | 越过 Pending 边界 | 立即停止并保全证据 |
| 出现工具批准提示 | 单工具预批准合同未满足 | 不点击，停止 |
| App/dev/store holder 未清零 | `BLOCKED_LIVE_HOLDER` | 请用户关闭，不自行 kill |
| 相关源码 hash 漂移 | `BLOCKED_DIRTY_OVERLAP` | 不构建、不覆盖，等待归属处理 |
| 原始 stderr/审计 detail 泄露到用户面 | 用户真话分层失败 | 停止并另出修复包 |

## 10. 验收标准

只有以下条件全部成立，才可写“S1B-H2 真实 App 验收通过”：

1. 当前源码对应的裸 debug binary 已重新构建、冻结并实际启动；
2. 两句精确验收话形成新鲜的自然两回合，第一句回复完成后才发送第二句；
3. 信息在用户、Syn、主管与唯一 MCP handler 间自然流转；
4. 只新增一张匹配的 PendingUserConfirmation 卡；
5. refresh 后无重复卡；
6. chain、worker、测试项目业务文件均未变化；
7. 没有人工工具批准、第二工具、原始错误泄露或假成功文案；
8. thread/generation、store delta、项目 hash、registry/进程均有可核对证据；
9. 结果已写入新的 evidence 与 `CURRENT.md`；
10. 未 stage、未 commit。

若只能证明其中一部分，必须按事实写“部分通过”“失败”或具体 blocker，不能借用离线通过补齐真实 App 结论。

## 11. 回传格式

执行者完成或止损后回传 10 项：

1. Gate -1 至 Gate 6 各自结果；
2. HEAD、脏基线与相关源码 hash 是否匹配；
3. 新 binary 路径、SHA-256、大小、mtime 与 build exit；
4. 真实 store 的 `B0/P0/C0` 与最终 delta；
5. 两条 canonical、主管回复、thread/generation 与 invalid-resume 情况；
6. 唯一 MCP handler 到达证据与 proposal 幂等键；
7. Pending 卡唯一性及 refresh 结果；
8. chain、worker、固定测试项目 hash/git 状态对账；
9. App 关闭后 holder、registry、进程对账；
10. 新 evidence/CURRENT/catch log 路径，以及 stage/commit 状态。

