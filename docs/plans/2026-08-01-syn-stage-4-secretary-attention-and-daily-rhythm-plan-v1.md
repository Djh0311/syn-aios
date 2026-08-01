# Syn Stage 4：秘书、Attention 与日常节奏计划 v1

日期：2026-08-01<br>
阶段：`M4`<br>
状态：**PLANNED / NOT_ACTIVE / NO_EXECUTION_AUTHORITY。**<br>
上位计划：`2026-08-01-syn-personal-ai-workbench-master-development-plan-v1.md` M4。<br>
硬前置：M1 identity / scope / policy、M2 UoW / event / projector、M3 Secretary RoleSession 可用；项目摘要由 M5 owner 提供时再接项目源。<br>
当前 active node / package：`NONE`；本计划不授权 App、模型调用、个人真实数据或产品代码。

权威顺序：当前用户指令 → `../harness/AUTHORITY.md` / `../harness/CURRENT.md` → 2026-08-01 修订与当前 inventory → master → M1-M3 exit receipts → 本计划。M5 ProjectSummary 和 M6 consult 未完成时只能显式 HOLD，不能由本计划补写成当前事实。

## 0. 当前事实与未知

### 已有局部能力

- `secretaryReadModel.ts` 能从 snapshot、workflow、runtime、proposal、blackboard、memory candidate 等来源确定性派生摘要；Secretary Board 与右 rail 有离线 DOM / fixture 覆盖。
- `secretary_agent.rs` 能做一次性只读解释；runtime attention、右栏通知 / 待办、memory daily inbox 提供了旧来源和产品形态素材。
- 当前首页、右栏、秘书看板已经显示部分“等用户”“风险”“运行中”信息，但它们是多源临时拼装。

### 尚未成立

- Secretary 没有自己的持久 RoleSession / store；一次性解释固定使用历史测试项目 cwd，前端缓存不跨重启，也没有统一审计。
- 没有 `PersonalScope`、`InboxItem`、`OpenLoop`、持久 `Notification / Todo / Reminder / DecisionRequest` 生命周期。
- `pendingAction`、通知、待办和运行关注主要是 React state 或即时派生，缺 read / dismiss / snooze / close / reopen / carry-over。
- 现有 “daily loop” 是记忆候选筛选 / best-effort capture，不是日报实体、scheduler 或全日整理。
- 没有顶层持续 Secretary conversation；“已知晓”与 owner 事实之间尚无正式边界。
- 没有空事件窗口的模型调用计数证据，也没有日报同窗口幂等、跨重启或逐项 deep link 证据。

### HOLD / 需冻结决定

- OpenLoop 与既有 Todo 的语义 / 物理关系，避免两套待办真源；
- personal inbox 的允许来源、正文 / 引用、保留期、删除和敏感级别；
- scheduler 时区、day window、catch-up、错过窗口、夏令时、重跑与版本纠正；
- priority 的确定性基线与模型可否增强；
- “用户已知晓”“关闭关注”“完成业务事项”的不同含义；
- M4 DailyReport 与 M7 memory consolidation 的事务 / source-ref 边界；
- 真实个人数据、真实模型和真实桌面表现。

## 1. 阶段目标

1. 建立 personal scope 下 source-first 的 Inbox / Attention / OpenLoop / Decision / Daily 对象；
2. Secretary 只维护“用户需要看住什么、为什么、谁拥有、怎样回源”，不改项目或外部 owner 事实；
3. 所有 attention 支持 dedupe、read、dismiss、snooze、close、reopen、carry-over 和跨重启；
4. 首页成为“可回源情境简报 + 持续 Secretary 对话”，但不借视觉重做掩盖后端缺口；
5. DailyBrief / DailyReport 在模型不可用时仍能确定性产生，同窗口幂等、可纠正、可重建；
6. 没有新事件的窗口不调用 Agent / 模型，并有机械计数证明；
7. 为 M6 跨项目咨询与 M7 每日记忆整理提供明确事件和 source refs。

## 2. 本阶段不做

- 不让 Secretary 创建 / 修改项目事实、工作流、授权、正式记忆或 Skill；
- 不把 read / dismiss / snooze 当成 owner 事项完成；
- 不把所有聊天自动转成 Inbox、Todo 或 OpenLoop；
- 不复制项目原文、raw transcript、邮件正文或 secret 到日报；
- 不把 priority 模型变成无来源的黑盒排序；
- 不接真实邮件 / 日历 / 文件 provider；它们属于 M8；
- 不把现有 memory candidate inbox 改名冒充 DailyReport；
- 不用 synthetic UI 或一次性 consult 声称秘书日常闭环通过。

## 3. 对象、owner 与不变量

| 对象 | owner / 真源 | 不变量 |
|---|---|---|
| `PersonalScope` | identity kernel | 与 project/global scope 分离，不借固定项目 root 取权限 |
| 原始 inbound / source record | 来源 domain；外部来源由 M8 connector owner | M4、adapter cache 和页面均不得把 projection 升级为来源真值 |
| `InboxItem` | M4 personal inbox projection | 只持 source ref、dedupe key、received_at、scrubbed summary、sensitivity；不自动成 Task，不拥有原文 |
| `OpenLoop` | Secretary coordination domain | 跟踪 closure，不拥有源业务事实；close 不反写 owner |
| `Todo` | `PersonalAction` aggregate 仅拥有用户显式创建的 standalone personal todo；项目 / 任务行动仍归原 source owner | M4 对原 owner 行动只持 ref / projection；与 OpenLoop 关系由 SEC-001 冻结，不可由 attention 自动创建 |
| `Notification` | Notification domain | delivery/read/dismiss 生命周期，不等于业务状态 |
| `Reminder` | Reminder domain | schedule、timezone、fire/dismiss/snooze、owner ref |
| `DecisionRequest` | 原业务 owner | M4 只持 `DecisionRequestRef/Projection`；不得持久化 / 重放 React `pendingAction` 的可执行确认 payload，重启后须从 owner 重解析并重新确认 |
| `DailyBrief/Report` | Daily projector | source-backed、versioned、rebuildable；不是正式事实 / 记忆 |

排序底线：外部承诺、已与别人约定和时间敏感事项优先。每个可见条目必须展示 source、owner、出现原因、最后变化、当前状态和精确回源入口。

## 4. 任务切片

### SYN-SEC-001 — Personal / Attention / Daily 合同冻结

冻结对象状态机、source owner、dedupe、priority basis、生命周期、scheduler、retention、source refs、OpenLoop↔Todo、M4↔M7 边界和 migration matrix。必须先裁决：原项目 / 任务拥有的行动只进入 M4 ref / projection；只有用户显式创建的 standalone personal todo 才由 `PersonalAction` aggregate 持有；该裁决未通过不得进入 SEC-002。M4 单独拥有 wall-clock scheduler、`daily_window_id` 与 DailyReport version；M7 只消费 `DailyWindowClosed / DailyReportVersioned`，其 annotation 是 M7-owned 独立 artifact/ref，不能原位改 DailyReport / Attention。只写合同。

### SYN-SEC-002 — Source projector 与持久 Attention

先接已结构化、低风险、只读来源；在 M2 UoW 上建立 Inbox / OpenLoop / Decision projection。read / dismiss / snooze / close 只改 coordination state，任何 owner write 必须走原对象 command。

### SYN-SEC-003 — Secretary application service

依赖 M3 RoleSession。实现持续 Secretary context、内部只读查询、发起通用 Handoff、处理 unavailable / pending / returned receipt、结果回源和受控解释。Global Supervisor 成功 consult 的实现与证据归 M6，不作为 M4 exit 前置。模型不可用时 deterministic brief 仍工作。

### SYN-SEC-004 — 首页情境与持续对话

在后端 DTO、source link 和状态机稳定后接首页。保留专业模块入口；Secretary 是顶层入口之一，不是唯一入口。视觉调整与读模型切换分包，禁止 UI 本地再造状态。

### SYN-SEC-005 — DailyBrief / DailyReport / Scheduler

冻结本地时区与窗口；以稳定 `daily_window_id` 和显式 `TimerFired / DailyWindowClosed` event 实现 idempotency、checkpoint、catch-up 上限、失败恢复、版本纠正、保留 / 导出。报告只引用 source-backed 项，不自动提升为 memory / task / project fact。

### SYN-SEC-006 — 零事件零模型调用与预算

以 event watermark、scheduler log、adapter spy / invocation ledger 证明空窗口 0 次模型调用；有事件时也先确定性聚合，模型仅在用户主动解释或合同允许的增强路径调用。

### SYN-SEC-007 — 旧通知 / 待办 / daily read path 兼容迁移

对右栏即时派生、runtime attention、pendingAction、memory daily inbox 做逐项 parity；旧 projection 只贡献 source refs / dedupe candidates，迁移时必须重读 canonical source + watermark，过期或 owner 不明项 expire / quarantine，不直接成为 active OpenLoop。旧面先 compatibility read-only，不在本阶段删除。

### SYN-SEC-008 — 隔离 App 日常节奏验收

用 synthetic structured events、两个 source owner、fake model 覆盖跨重启、dedupe、snooze、carry-over、日报重跑、source deep link、模型故障。真实个人数据 / 模型 / provider 另包。

## 5. 依赖、并行与写域

```text
SEC-001 → SEC-002 → SEC-003 → SEC-004
                 ├→ SEC-005 → SEC-006
                 └────────────→ SEC-007 → SEC-008
```

- Attention / Daily domain 由 M4 单写；ProjectSummary 由 M5 生成，M4 只能消费；
- M4 与 M5 可在合同冻结且写面不重叠后并行；event/schema/AppState/App assembly/public frontend shell 必须由指定单写者串行接线；
- M6 Global Supervisor consult 由 M6 owner，M4 只发 Handoff 并消费 receipt；
- M7 拥有 memory / PersonalFact / Skill；M4 只产生来源事件、DailyWindowClosed / DailyReportVersioned 和日报，不直接写正式记忆，也不接受 M7 原位修改报告；
- M8 外部 provider 只生成合同允许的 InboundItem / source ref；M4 不接 credential；
- 目标文件已有 dirty WIP 或 DTO 未冻结即停止并重新分包。

## 6. 迁移与回滚

- 先 shadow 旧右栏 / secretaryReadModel 的确定性输出，逐项对比 source、status、priority reason；
- 旧 notification / todo / runtime attention 没有独立 owner 时，只作为 adapter 输入，不直接迁成已完成事实；
- dedupe 只合并显示 / coordination，不合并不同 owner 的业务对象；
- scheduler checkpoint 与 DailyReport version 可重建；失败不丢原事件；M7 只以 M4 window/report version 为幂等输入；
- rollback 回到旧只读展示，不撤销已提交的 source owner 事实，也不恢复固定项目 cwd 的 Secretary 权限；
- 所有用户纠正产生新 version / audit，不覆盖历史；
- 旧读面直到 M9 parity / unregister 才退役。

## 7. 验证矩阵

| 层级 | 必须证明 | 不能声称 |
|---|---|---|
| Contract / fixtures | owner、状态、dedupe、scheduler、M4/M7 边界 | service 已实现 |
| Unit / property | coordination 不反写 owner、同窗幂等、时区/catch-up、零事件 gating | App 可用 |
| Temp integration | 跨重启、投影重建、模型失败、source deep link | 真实个人数据通过 |
| Non-test build | production path 可构建 | UI 行为正确 |
| Isolated Tauri | attention 生命周期、日报、对话入口和回源可见，并保留真实桌面窗口截图 / 可见交互 / deep-link 点击证据 | 真实模型 / connector 通过 |
| 经授权真实使用 | 指定数据源 / 模型 / profile 的日常场景 | 发布或所有来源通过 |

关键验收：App 重启不丢关注；“已知晓 / 关闭”不改项目 / task / memory；同窗口日报幂等；每项可回源；模型不可用仍有 deterministic report；空事件窗口 invocation count 为 0。

## 8. 授权与停止条件

local schema / store migration、App 启动 / 强制退出、个人真实数据、真实模型 consult、真实项目摘要、首页主入口切换、scheduler 常驻 / OS wake、任何 connector / credential、旧读路关闭分别建包。M4 不授权项目写、真实外部 action、记忆自动提升、Git 或发布。

立即停止：source owner 不唯一；OpenLoop 与 Todo 重叠成双真源；ack 改了 owner 事实；日报需要 raw transcript / secret；priority 无 reason；空事件仍调用模型；M4 直接写 M5/M7 对象；UI 本地状态冒充持久生命周期；dirty WIP 冲突；synthetic 被表述成真实日常验收。

## 9. 阶段退出与下游交接

全部满足才算 M4 完成：

- PersonalScope、Inbox / OpenLoop / Decision / Daily 合同和 owner 冻结；本阶段完成口径只覆盖个人及已接入内部来源，M5 ProjectSummary 未接时项目来源集成必须标 `HOLD`，不得声称“全部用户相关 open loops”闭环；
- attention 全生命周期跨重启且可回源，不反写 owner；
- Secretary 持久会话与 deterministic brief 可用；
- DailyReport 同窗幂等、可纠正、可重建；空事件零模型调用有机器证据；
- 旧派生面有 parity / compatibility / rollback，不物理删除；
- isolated App 场景通过；真实数据结论逐项记录；
- 把 source event / DailyReport ref 合同交 M7，把咨询合同交 M6；
- CURRENT 回写实际完成和 HOLD，未激活不得续跑。
