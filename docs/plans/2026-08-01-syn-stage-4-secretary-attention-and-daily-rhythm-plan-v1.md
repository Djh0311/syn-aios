# Syn Stage 4：秘书、Attention 与日常节奏计划 v1

日期：2026-08-01<br>
阶段：`M4`<br>
状态：**M4R07 V2 PRODUCT-CHAIN PASS / STAGE-07 CLOSED。**<br>
上位计划：`2026-08-01-syn-personal-ai-workbench-master-development-plan-v1.md` M4。<br>
硬前置：M1 identity / scope / policy 冻结合同、M2 bounded reference slice（有边界参考切片）、M3 已完成的通用 RoleSession 合同与隔离实现；普通产品 Secretary RoleSession 运行时桥接已由 M4C02 完成。项目摘要仍等待 M5 owner 提供。<br>
已关闭阶段：`stage-06`；M4C01–M4C10 已归档。本阶段历史授权记录为 `USER-SYN-M4-AUTONOMOUS-STAGE-06-20260810`，不延续到新的工程任务。真实个人资料、真实模型 / provider、真实消息、外部连接器、远端和发布均未进入。
已完成修正与收口记录：`2026-08-11-syn-m4-independent-remediation-and-reacceptance-plan-v1.md`。M4R01–M4R07 已完成并归档，M4R07 v2 产品链合同为 PASS，`stage-07` 已关闭；当前没有活动 stage、活动 leaf 或持续授权。

三条权威链分开读取，不能把 current-state（当前事实）排在产品正本之前：

- 产品与架构：当前用户指令 → 产品正本 → authority register → 现行架构 → master → 本计划 → `../contracts/m4-secretary-attention-daily-resolution-v1.md` 的 M4 实施解释；
- 施工与授权：当前用户指令 → `../../../AGENTS.md` → `../../AGENTS.md` → `../harness/plan.md` → 新的活动 stage / 唯一 leaf → `../harness/authorization.json`；已归档 stage-06 与历史授权不产生下一阶段权限；
- 已实现事实与证据：当前源码 / 新鲜验证 → `../current-state.md` → M1-M3 合同、退出回执和验收报告。

M5 项目摘要和 M6 咨询未完成时按 unavailable source（来源暂不可用）处理，不能由 M4 补写成当前事实。

## 0. 当前事实与未知

### stage-06 已完成主线能力与 2026-08-11 修正前基线

- M4C01 冻结了普通产品 M3 bridge、Secretary/PersonalScope、M4 单写 store、source/dedupe/priority、时区/日报、OpenLoop/Todo、M4/M7、迁移回切与证据分层合同。
- M4C02 把后端固定的 Secretary RoleSession 与 PersonalScope 接入普通产品 `AppState`；M4C03 建立 M4 自有 schema/repository/UoW 与 source-first Inbox/OpenLoop/Decision projection。
- M4C04 完成 attention、Notification、Reminder、显式 PersonalAction 与 owner command receipt 的状态机、repository 和单元测试；普通产品入口与到期时钟调用仍待修正。
- M4C06 让首页消费 typed read model，展示来源、owner、优先理由、状态与 source descriptor；当前 deep link 只到通用项目面，持续 Secretary 消息输入仍处于 disabled 占位。
- M4C07 完成本地时区 daily window、daily scheduler、catch-up、幂等、版本纠正、失败恢复与空事件零模型机械证明；它尚未驱动 snoozed OpenLoop 和 Reminder 到期唤醒。
- M4C08 完成五类旧读面的 inventory、comparator、compatibility read-only 边界和 quarantine；普通产品尚无 legacy tuple adapter，当前全部 fail closed，实际 shadow/parity/fallback 尚未成立。
- M4C09 使用 synthetic fixture、两个 source owner、fake model 与隔离 profile 完成 debug App 首启、强退、同 profile 重启、生命周期、日报、deep link 和模型故障验收；C10 完成全量离线回归与 launcher 静态契约消歧。

### 2026-08-11 尚未成立项（M4R01–M4R06 的历史修正输入）与下游 HOLD

- 普通产品 composition 尚无 M4 source ingress 的生产调用者；C09 通过验收代码直接注入 synthetic source，不能证明正常产品会自然接收内部事项。
- snoozed OpenLoop 与 Reminder 的到期推进没有接入生产 scheduler；单元测试手工调用 transition 不能替代产品时钟。
- source deep link 尚未通过 owner adapter 精确落到原对象；当前只进入通用 Projects 页面。
- 首页持续 Secretary 消息发送、M3 Turn 写入和跨重启历史恢复尚未接入；固定机械解释不等于持续对话。
- 五类 legacy read path 尚无实际 server-owned tuple adapter；inventory-only quarantine 不等于真实 parity / fallback。
- M5 ProjectSummary 合同/owner 尚未激活，项目摘要 source 明确 unavailable；M4 的完成口径不覆盖“全部用户相关 open loops”。
- M6 Global Supervisor 成功 consult 尚未实现；普通产品 M4 只保留 M3 Handoff 请求/回执边界并显式 unavailable。
- M7 对 `DailyWindowClosed` / `DailyReportVersioned` 的消费、正式记忆、PersonalFact、个人模型与 Skill 未实现；M8 真实 connector/credential/external source 也未进入。
- M9 command unregister 与旧路物理退役、M10 真实全日试点和发布硬化未进入。C09 synthetic isolated App 不等于真实日常使用、发布包或生产验收。

### 2026-08-13 M4R07 当前结论

- 前五项普通产品缺口已分别由 M4R02 来源与个人对象组合、M4R03 服务端到期时钟、M4R04 注册 owner 精确回源、M4R05 持续 Secretary 对话、M4R06 实际 legacy shadow/parity/fallback 修正；M4R01 冻结其生产调用图和旁路禁令。
- `../harness/reports/M4R07-isolated-product-reacceptance-closeout-behavior-receipt.json` 是 v2 portable PASS：固定 12 次、实际 12 次；第 8 次真实等待 98 秒并保留后端 OPEN/FIRED 恢复验证。
- 第 8 次 UI / Computer Use / PNG / attestation 按当前范围为 `NOT_EXECUTED / NOT_APPLICABLE`，Computer Use 次数为 0，未写截图、attestation 或 capture signal。它既不是视觉失败，也不提供视觉、Accessibility 或截图 PASS。
- v2 manifest 只绑定 portable receipt SHA 与 `launch_8_ui_validation` canonical SHA。真实个人数据、真实模型/provider/connector、远端、部署和发布仍未验；M5–M10 继续 HOLD / NOT_ACTIVE。
- M4R07 产品链完成标记已经成立，M4R01–M4R07 已归档且 `stage-07` Harness 生命周期已经关闭；该事实不自动激活 M5–M10。

### 已冻结边界与验收结论

以下内容已经由产品正本、M1/M3 合同、本计划和 `m4-secretary-attention-daily-resolution-v1.md` 裁决，不是待用户重复回答的问题：

- `OpenLoop` 只拥有协调状态；只有用户明确命令才创建独立 `PersonalAction`，关注项不会自动克隆为 Todo。
- personal inbox v1 只接 allowlist 内的结构化、低风险、可回源引用；持久层禁止 raw transcript、prompt、provider body、tool output、凭据和外部正文。开放事项无自动 TTL，终态可见 90 天、日报可见 365 天；M4 不做物理删除。
- scheduler 使用后端解析的 OS IANA timezone，以本地自然日计算半开窗口；夏令时按真实 UTC 边界处理，启动时最多按最旧优先补 7 个窗口，同窗幂等，纠正产生新版本而不覆盖。
- priority 由确定性 tier、due time、source change、owner/object ID 排序且必须展示 reason；模型只能解释，不能改 rank、dedupe、状态或 owner。
- “用户已知晓”=`ACKNOWLEDGED`，“关闭关注”=`CLOSED`，两者都不等于来源业务事项完成；业务完成只能走 source owner command 并回收 receipt。
- M4 拥有 scheduler、`daily_window_id`、DailyBrief / DailyReport version 和相关事件；M7 只消费 ref/event 并创建独立的 M7 artifact，双方都不能原位修改对方对象。
- 真实个人数据、真实模型、真实 provider / connector 与真实日常使用不在 stage-06 授权内；M4C09 只做合成数据、隔离配置、假 provider 的调试 App 验收。
- 普通产品 M3 runtime bridge、M4 自有数据库路径、PersonalScope/Secretary 稳定身份、迁移与回切、证据等级和叶映射均以 M4 合同唯一机器块为准。

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

由 M4C01 冻结对象状态机、source owner、dedupe、priority basis、生命周期、scheduler、retention、source refs、OpenLoop↔Todo、M4↔M7、M2 复用上限、普通产品 M3 bridge 和 migration matrix。原项目 / 任务拥有的行动只进入 M4 ref / projection；只有用户显式创建的 standalone personal todo 才由 `PersonalAction` aggregate 持有。M4 单独拥有 wall-clock scheduler、`daily_window_id` 与 DailyReport version；M7 只消费 `DailyWindowClosed / DailyReportVersioned`，其 annotation 是 M7-owned 独立 artifact/ref，不能原位改 DailyReport / Attention。

### SYN-SEC-002 — Source projector 与持久 Attention

先接已结构化、低风险、只读来源；通过 M4 自有 repository / UoW 建立 Inbox / OpenLoop / Decision projection，只允许经明确 adapter 复用 M2 已证明的低层 immediate transaction / busy retry 机制。read / dismiss / snooze / close 只改 coordination state，任何 owner write 必须走原对象 command。

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
| Isolated product chain | 普通产品入口、持久状态、强退/重启、后端到期、对话、回源和 legacy reader receipt | UI 可见、截图质量、Computer Use、真实模型 / connector 通过 |
| 经授权真实使用 | 指定数据源 / 模型 / profile 的日常场景 | 发布或所有来源通过 |

关键验收：App 重启不丢关注；“已知晓 / 关闭”不改项目 / task / memory；同窗口日报幂等；每项可回源；模型不可用仍有 deterministic report；空事件窗口 invocation count 为 0。

## 8. 授权与停止条件

stage-06 历史授权曾允许各 leaf 写域内的 local schema/store、离线测试和构建、临时/隔离数据库、假模型/provider、精确本地提交，以及 M4C09 专门验收包中的合成调试 App 启动、强退、重启和脱敏证据保存。阶段关闭后该授权不再产生新的施工权限。

本阶段始终排除了真实个人资料、真实用户项目写入、真实模型/provider、真实 Codex 消息、真实账号/凭据/外部 connector、网络外部写入、远端、push、merge、rebase、部署、发布、reset、clean、stash、破坏性删除和 M5-M10 产品实现；最终结论也不包含这些范围。

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
- `../current-state.md` 回写实际完成、证据上限和暂缓项；M4C10 全量回归、文档同步与 stage-06 收口已经完成，当时停止等待总线主管复核。当前结论与下一入口见本节末尾及独立修正计划。

M4C01–M4C10 和 `stage-06` 的程序性归档已经完成，`../harness/reports/M4C10-mainline-integration-and-acceptance.md` 保留为当时的机械与隔离证据。2026-08-11 独立总线复核指出的五项缺口已经进入 M4R01–M4R07 修正链；M4R07 v2 在当前后端/普通产品链合同范围内 PASS，但第 8 次 UI / Computer Use / PNG / attestation 未执行，真实数据/provider/connector/远端/发布未验。M4R01–M4R07 已归档，`stage-07` 已关闭；当前没有活动 stage / leaf，不自动进入 M5–M10。
