# M4 独立修正增补合同 v1

状态：`FROZEN FOR STAGE-07 IMPLEMENTATION`

日期：2026-08-11

适用任务包：`M4R01`–`M4R07`

## 0. 合同定位

本文件只补充 M4 普通产品组合、生产调用者与再验收接缝，不改写下列冻结合同，也不改变产品方向：

| 冻结合同 | SHA-256 |
|---|---|
| `docs/contracts/role-session-v1.md` | `77c82932e728d4982ebb501b167f274cc31d2076957602771904d96dc399b2ca` |
| `docs/contracts/handoff-v1.md` | `3378f02f5dfb06e4db39125b5828eeda9440fc2c25ddbee3fe4e951fa6c386bf` |
| `docs/contracts/identity-scope-v1.md` | `3cb0073c0fffc2423e3450ce9d9e3c683065cdd075bf618e0d406cc1475e3ea4` |
| `docs/contracts/event-audit-outbox-v1.md` | `15a24d8040da054794e340fe7839b273dce0f60a2c1708513d1b998c8e968e99` |
| `docs/contracts/m3-role-session-turn-handoff-resolution-v1.md` | `946c756b30a8e73aaad441e49ba39a5c9cbd7c7d47241ed97fa19d02783bac48` |
| `docs/contracts/m4-secretary-attention-daily-resolution-v1.md` | `4e4d6251d53e1b9b156fb2fd1266d73d6beace38be2086e83e0f05694dec4e51` |

若本增补与上述冻结合同、产品正本或当前用户指令冲突，以后者为准并停止相关施工。本增补不授权 M5–M10，不把历史 C09/C10 receipt 升格为本轮产品闭环证据。

## 1. 普通产品组合的定义

“普通产品链”必须同时满足：

1. 从 `index_host_app_entrypoints.rs::run -> setup -> AppState` 的共享构造接缝开始；隔离 profile 只可替换 app-data root、server clock、fake provider / transcript 等基础端口。
2. 使用普通 `command_registry.rs`、source dispatcher、scheduler、route resolver、conversation transport 与 legacy readers。
3. fixture 只创建隔离输入，调用真实 owner 的普通 command/event 入口；fixture 不直接调用 M4 adapter、repository、transition 或手工完整 candidate。
4. server 解析固定 PersonalScope、owner、revision、permission 与 role identity；renderer 不提供或猜测这些权威字段。
5. App、provider 或进程重启后，持久 outbox、CAS、RoleSession / Turn 与 source ref 能按相同权威恢复，不靠内存回调或 renderer cache。

`m4_acceptance`、改名 command、专用 handler、repository seed、`advance_open_loop_clock`、`fire_reminder`、手工 exact legacy candidate 和源码字符串检查都只可做低层辅助，不能单独证明普通产品闭环。

## 2. 真实内部 source owners

本轮首个、也是 R06 parity（相等性复核）复用的主 owner 固定为 M2 workflow-state WorkItem owner：

- owner UoW：`workflow_run_dispatch_entrypoints.rs` 的 DB-primary（数据库主写）状态推进事务
- 普通入口：`update_work_item_state`
- 普通注册面：`command_registry.rs`
- 普通 fixture 前置入口：`initialize_workflow_state -> bootstrap_project_workflow -> create_task_draft`，随后仍须走 `update_work_item_state`
- canonical object id：owner command 内的 `work_item_id`
- source event id：同一 owner UoW 产生的 `event.event_id`，经 server seal（服务端封装）成为不可猜的 M4 opaque ref
- source revision：`receipt.committed_revision`，必须与同一 event 的 `source_revision` 精确一致
- source owner watermark：同一事务 current snapshot 的 `source_watermark`，必须与 `event.event_id` 精确一致；不得用时间戳、renderer 或 M4 revision 猜测
- source owner 写权限：只属于 workflow-state owner；M4 只消费脱敏事件并维护自己的协调投影

这个 owner 已在同一事务持久化 receipt、event、audit、domain state 与 current snapshot。R02 在该 durable event ledger（持久事件账本）上增加 M4 consumer checkpoint（消费者检查点）和 typed adapter（类型化适配器）；若现有 event 字段不足，只能在同一 M2 UoW 内补 scrubbed typed publication row（脱敏类型化发布行），不能在 wrapper 返回后 best-effort 投递。

native provenance（owner 原生来源凭证）与 M4 opaque refs（M4 不透明引用）分两层保存和校验：

1. owner publication 先保存 `owner_native_event_id`、`owner_native_watermark` 与 `native_scope_seal`。WorkItem 必须在同一事务内验证 `receipt.committed_revision == parse(event.source_revision)` 且 current snapshot 的 `source_watermark == event.event_id`。
2. M4 `source_event_id` 使用 event 专用 domain-separated seal（域隔离封装）绑定 owner ref + native event id；M4 `source_owner_watermark` 使用另一个 watermark 专用 seal 绑定 owner ref + native watermark。两个 seal 不能共用 namespace，也不要求封装后的字符串相等。
3. `source_revision` 保留 owner 的 exact numeric revision（精确数字版本）；proposal 的整数 `store_revision` 进入 M4 前也遵守同一规则，不能拿最新 store revision 回填旧 event。
4. native 值的相等性在 seal 前验证；`native_scope_seal` 同时保存在 owner publication 与 M4 route/provenance index（回源凭证索引），映射到固定 PersonalScope 时不得丢失。renderer 不参与 seal、scope 或 identity 选择。

typed Decision（类型化决定）次级 owner 固定为项目咨询方案 owner：

- owner store：`project_consultation_proposal_store.rs`
- 普通创建入口：`create_project_consultation_proposal`
- 普通决定入口：`record_project_consultation_proposal_decision`
- canonical object id / event id：同次成功输出的 `proposal_id` / `audit_event_id`
- source revision / owner watermark：同次成功输出的精确 `store_revision`

proposal audit 行当前没有保存同次 `store_revision`，所以 R02 必须在 owner 成功持久化边界内同步写入 scrubbed source-event outbox envelope。两个 owner 的 envelope / publication 至少含：schema version、event id、owner ref、object type、canonical object id、source revision、owner watermark、native scope seal、状态码、有限 signal flags、occurred-at、payload hash、dispatch state / attempt metadata。它不得含方案正文、prompt、tool output、secret、任意路径、URL 或可执行 payload。

proposal owner -> typed Decision 的唯一状态映射固定为：`Draft` / `PendingUserConfirmation` -> `OPEN`；`UserConfirmed` / `ChangesRequested` / `Rejected` -> `ANSWERED`；`Superseded` -> `WITHDRAWN`。`EXPIRED` 只能来自 owner 普通生产链明确持久化的 expiry event（到期事件）；当前 owner 没有该事件，R02 必须补 owner-owned、server-clock 驱动且可恢复的明确到期 transition，禁止由 M4 当前时间、测试 fixture 或最新 store 状态猜出历史 `EXPIRED`。

owner 事实与 publication 必须同成同败；dispatcher 可异步、可重试，但不得在 owner 已提交后永久丢事件。相同 event id + payload hash 是幂等 replay；相同 event id + 不同 payload hash 必须 quarantine。adapter 只能把已登记的本地 owner 事件投影到固定 PersonalScope，且须保留 native scope seal 供回源复核；renderer 不能选择 scope。M4 的 read/dismiss/snooze/close/reopen 不反写 owner。

## 3. 五项 P1 的冻结生产调用图

### P1-A 普通来源与个人对象

```text
ordinary UI / Tauri command
  -> WorkItem / consultation-proposal owner command
  -> owner atomic state + durable event/publication
  -> AppState-installed production dispatcher
  -> registered source-owner adapter
  -> M4 ingest/dedupe/quarantine
  -> Inbox + OpenLoop + typed Decision projection
```

单写者：WorkItem / proposal source status 由各自 owner 写；M4 coordination 由 M4 repository 写。显式 PersonalAction / Reminder 只能由用户普通命令创建；Notification 只能由正式内部事件形成。OpenLoop 不自动复制 Todo。

红灯反例：fixture 直接 `ingest_workflow_attention_source`；wrapper 返回后 best-effort 投递；revision 从时间戳或 renderer 猜；M5/M8 adapter 冒充现存来源。

### P1-B 服务端到期时钟

```text
ordinary AppState startup
  -> start_m4_secretary_scheduler
  -> StartupRecovery / TimerTick
  -> run_daily_scheduler_cycle with one captured server-now
  -> due-transition batch
  -> snoozed OpenLoop reopen + due Reminder fire
  -> atomic event / receipt / audit + brief reprojection
```

单写者：M4 repository。due key 必须绑定 aggregate id、当前 revision 与 due marker；到期比较使用解析后的 UTC 时间。重复 tick、并发 tick、CAS 冲突和重启补偿不得双触发。server fire 的审计 reason 是 `SERVER_CLOCK`。

红灯反例：renderer 计时；验收直接调用 `advance_open_loop_clock` / `fire_reminder`；每次 tick 用随机 id 产生重复 effect；server fire 记成用户命令。

### P1-C 注册 owner 精确回源

```text
server-minted sealed source_route_ref
  -> ordinary resolve_secretary_source_route command
  -> AppState-installed owner route registry
  -> exact owner/type/id/revision/current-target verification
  -> finite typed navigation target
  -> owner page consumes typed focus and selects exact record
```

route read capability 与 owner writeback capability 分离。resolver 只返回有限枚举，不返回 raw path、URL、callback 或可执行内容。unknown owner、missing target、stale revision、scope mismatch、route tamper 均 fail closed；失败时 UI 不导航也不显示成功提示。

红灯反例：renderer 根据 object type 猜 `projects`；只切通用大厅而目标页不消费 focus；相同 id 不同 owner 串路。

### P1-D 持续 Secretary 对话

```text
Home / dock ordinary composer
  -> send_secretary_message
  -> fixed Secretary runtime in AppState
  -> existing M3 RoleSession + binding + authority snapshot
  -> M3 Turn / effect / conversation transport
  -> fake-or-real provider-owned transcript read port
  -> load_secretary_conversation joins display history
```

M3 RoleSession / Turn 是 lifecycle 真源，provider transcript port 是 raw content 真源；M4 只提供机械情境和 typed refs，不建立第二份 transcript。首次明确消息前 provider invocation 必须为 0。restart 使用同一 RoleSession、readback-only recovery 与既有 history，不重发旧 turn。失败落明确 FAILED turn / UI error，不伪造回复。

红灯反例：只恢复 role_session_id 而无历史；UI 本地拼 transcript；发送时创建 Todo/项目/工作流/记忆；空 Home load 调 provider；复用 acceptance wrapper 或 manual relay。

### P1-E 五类旧读面 shadow / parity / fallback

```text
ordinary compatibility read command
  -> AppState-installed five server-owned legacy readers
  -> each reader queries its real owner surface
  -> exact scrubbed candidate tuple
  -> canonical M4 reread + comparator
  -> PARITY / EMPTY / UNJOINABLE / QUARANTINED
  -> guarded read-only fallback + exact source route
```

五类固定 inventory 是 allowlist，不是 candidate 数据。adapter 不修复、不写 owner、不创建 coordination effect。至少一个实际可连接旧面必须形成 PARITY；其余按真实结果报告，不得用五个全空 candidate 代替 reader。

首个必须形成真实 PARITY 的组合固定为 WorkItem owner -> right-rail notification/todo projection（右栏通知/待办投影）；它与 P1-A 共用同一个 owner mapper。React 本地 notice、error、warning 与 pendingAction 没有 server-owned revision / watermark，只能形成带 reader receipt 的 `UNJOINABLE` 或真实空态，不能伪造 candidate。

红灯反例：测试手工构造 exact candidate；renderer 临时态升格为 owner；连接失败被写成 EMPTY；fallback 显示无法精确回源的项。

## 4. 失败、恢复与敏感边界

- 所有跨 owner 接缝默认 fail closed。未知 schema、scope/owner/revision 不匹配、hash 冲突或缺必要 join 字段进入 quarantine，不猜测修复。
- transaction 中断后要么 owner 状态与 outbox 都不存在，要么二者都存在；dispatcher 中断后由同 profile 重启继续，M4 dedupe 防止双投。
- timer、source、route、conversation 与 legacy 读面不能因 App restart 改换 identity、scope、owner 或 permission。
- M4 日报、outbox、receipt、审计、evidence 不持久化 raw transcript、prompt、tool output、secret、credential、真实个人资料或任意外部 payload。
- 协调动作不改变 owner 事实；legacy fallback 只读；聊天不隐式产生结构化对象。

## 5. 验收证据层级

| 层级 | 可证明 | 不可代替 |
|---|---|---|
| Contract / static marker | 冻结声明、目标文件/符号索引、显式反例 marker 与 frozen hash | production reachability、调用边、旁路已消失或功能已运行 |
| Repository / unit | transition、CAS、dedupe、quarantine、comparator | production caller 已接入 |
| Ordinary composition | 共享 constructor 与非测试 caller 串通 | UI 可见、强退恢复 |
| Non-test build | production code 可构建 | 用户交互正确 |
| Isolated product App | 普通入口、可见行为、SIGKILL / restart、fake failure | 真实资料/provider/外部能力 |
| Fresh full regression | 本地回归在新临时目录稳定 | 远端、发布、部署 |
| Independent bus review | Git/Harness/代码/证据/边界一致 | 自动激活 M5–M10 |

R01 的 red probe 是 opt-in 静态门，只记录旧断点，并通过 `--expect=red` 在旧基线成功退出；它不进入默认失败套件。R02–R06 先用相应 `--only` 重放 red，再以真实实现和行为测试转成 green。纯字符串 green 仍不算完成，必须同时有该包规定的 ordinary composition 或更高层直接证据。

## 6. R01 机械 marker

`scripts/run-m4-remediation-probes.mjs` 固定检查以下非测试接缝：

1. source owner durable outbox 与 production dispatcher；
2. ordinary scheduler 的 due-transition batch；
3. server route resolver、普通 command registry 与 owner page focus consumer；
4. 普通 send/load conversation command、transport 与可用 composer；
5. server-owned legacy readers 替换 ordinary command 的 inventory-only candidates。

marker 只是可重现调用图索引。任何只加注释、死代码、测试调用者或 acceptance 专用 wrapper 使 marker 变绿但行为证据不成立的实现，均视为未完成。

## 7. R02–R06 行为 receipt 下限

各包 green receipt 至少使用 `syn.m4.remediation.behavior-receipt.v1`，并明确 `ordinary_composition=true`、`acceptance_wrapper_calls=0`、`direct_repository_seed_calls=0`：

| probe | 必须直接记录 |
|---|---|
| source | 普通 owner command、owner revision/event/watermark、publication/dispatch/M4 delta；duplicate 与 restart 的零重复和 checkpoint |
| clock | captured server-now、StartupRecovery/TimerTick、OpenLoop/Reminder transition delta、重复/并发 tick 零新增、`SERVER_CLOCK` audit reason |
| route | source owner/type/id/revision 与 resolved target 逐项相等、目标 focus 已消费；unknown/stale/missing/tampered 的零导航和固定错误 |
| conversation | 两次明确发送绑定同一 RoleSession/scope/channel、Turn/response-or-failure、restart history、zero-event provider calls=0 |
| legacy | 五类 reader receipt、tuple 完整性、canonical reread、真实 outcome、fallback visibility；至少一个 WorkItem-backed PARITY |

source receipt 的 `ingestion_adapter_id` 必须等于 WorkItem legacy receipt 的 `legacy_reader_adapter_id`，证明 R02 与 R06 复用同一个 registered owner mapper，而非两套状态映射。
