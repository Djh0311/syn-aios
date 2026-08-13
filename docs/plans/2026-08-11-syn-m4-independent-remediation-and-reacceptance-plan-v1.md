# Syn M4 独立修正与再验收计划 v1

日期：2026-08-11<br>
里程碑：`M4 corrective closure`<br>
状态：`M4R07 V2 PRODUCT-CHAIN PASS / STAGE-07 CLOSEOUT PENDING`<br>
当前 Harness 生命周期：`stage-07`；任务包前缀：`M4R`<br>
上位计划：`2026-08-01-syn-personal-ai-workbench-master-development-plan-v1.md` 的 M4<br>
原阶段：`stage-06` 已程序性关闭，M4C01–M4C10 已归档；本计划不重开、不改写历史链。

## 0. 权威定位与启动边界

本计划是现行 M4 阶段计划下的修正实施计划，不是产品正本，不改变已经确认的 M4 核心需求，也不是 M5。它只解决独立总线复核发现的实现与验收缺口。

读取顺序：当前用户指令 → 产品正本 / 权威登记 / 现行架构与前端显示边界 → 当前总计划 → M4 阶段计划 → 本计划 → 新的活动 stage / 唯一 leaf / 当前授权。

本文件存在不产生施工权限。`stage-07` 已按独立授权执行 M4R01–M4R07；M4R01–M4R06 已归档，M4R07 已取得 v2 产品链完成标记，当前只剩文档、独立复核与 Harness 生命周期收口。历史 `stage-06` 授权不得复用，也不能从当前收口推导 M5–M10 权限。

独立复核依据见 [`M4-independent-bus-review-2026-08-11.md`](../harness/reports/M4-independent-bus-review-2026-08-11.md)。其 P0=0、P1=5、P2=4 是修正前基线；五项 P1 已映射并由 M4R01–M4R06 修正。2026-08-13 的当前完成标记是 M4R07 v2 portable receipt 与 v2 manifest 的精确交叉绑定；`stage-07` 尚未关闭。

## 1. 这次修什么

一句话：保留已经做好的 M4 底座，把普通用户真正经过的产品链路接完整，并用普通产品入口重新验收，禁止再用测试直接灌 repository 的旁路代替产品闭环。

修正目标：

1. 普通产品能够从当前可用、结构化、低风险的内部 source owner 接收事项，形成 Inbox / OpenLoop / Decision；显式个人 Todo、Reminder、Notification 有正式产品入口。
2. OpenLoop snooze 和 Reminder 到期由后端生产时钟幂等唤醒，强退和重启不漏、不重。
3. 每个 source link 经注册 owner adapter 解析，并精确落到原对象；通用项目大厅不再冒充精确回源。
4. 首页提供真正可输入、可发送、可继续、可跨重启恢复的 Secretary 对话，复用 M3 RoleSession / Turn / conversation transport，不建立第二套会话真源。
5. 五类旧读面接入实际 server-owned shadow reader，形成 exact tuple、canonical reread、parity / quarantine / fallback 证据。
6. 以全新隔离 profile、普通产品 composition 和 fake provider 完成整体验收；最后由总线再次独立复核。

## 2. 明确不做

- 不接真实个人资料、真实用户项目写入、真实模型/provider、真实消息、真实账号、凭据、外部 connector、网络外部写入、远端、部署或发布。
- 不实现 M5 ProjectSummary、M6 Global Supervisor success、M7 memory consumer 或 M8 外部来源；这些继续 `HOLD / UNAVAILABLE`。
- 不把聊天自动转成 Inbox、OpenLoop、Todo、项目、工作流、正式记忆或 Skill；结构化动作仍需用户明确发起并经过原 owner 边界。
- 不让 renderer、路由、固定 cwd、模型或测试 fixture 成为 identity、scope、permission、owner、clock 或持久状态真源。
- 不修改 M1/M3 冻结合同正文，不原地改写 M4 v1 冻结合同、C01–C10 归档、`stage-06`、C09/C10 历史报告或旧 receipt。确需补充实施解释时新建独立增补合同并保持旧 hash exact。
- 不物理删除旧读面，不借修正阶段实施 M5–M10，不把界面视觉重做混入本阶段。

## 3. 任务包

### M4R01 — 修正合同、生产调用图与红灯验收

目标：把五项 P1 逐一映射到生产入口、owner、单写者、失败反例和验收层级；必要时建立不改写 M4 v1 的增补合同。先冻结可重复的 red probe（红灯探针）和证据规范，再进入功能施工。

完成标准：

- 五项缺口各有一个明确的普通产品生产调用图和一个能复现旧缺口的红灯探针；R01 保存旧基线 red receipt，但不把长期失败或永久 ignored 的测试留在默认套件。
- 验收矩阵明确区分 repository/unit、普通产品 composition、isolated App、真实使用；低层通过不得替代高层。
- 普通产品验收明确禁止直接调用 repository seed、`advance_open_loop_clock`、`fire_reminder` 或手工构造 legacy exact candidate 冒充产品链路。
- R01 必须点名至少一个当前真实存在的内部 source owner 及其普通产品 command/event 发布入口；若 owner、revision 或 watermark 无法精确确定，按停止条件回报，不用 M5/M8 或验收专用 adapter 补造。
- R02–R06 在各自 leaf 内先复跑对应 red probe，再实现到 green；leaf 完成时默认测试套件必须全绿。纯源码字符串匹配和永久 ignored 不算生产行为证据。
- 冻结合同 hash 保持 exact；新增解释不得改变身份、scope、owner、secret、零模型、Todo/OpenLoop 和 M4/M5/M7 边界。

### M4R02 — 普通产品来源与个人对象组合

目标：接通至少一个当前可用、结构化、低风险的内部 source owner adapter，并给显式 PersonalAction、Reminder、Notification 与 typed Decision projection 建立普通产品端口。M5/M8 来源继续 unavailable。

完成标准：

- 使用全新隔离 app-data 和普通产品 constructor；fixture 只提供隔离数据，并调用被点名 source owner 的普通产品 command/event 入口，再经 production adapter 进入 M4。fixture 不直接调用 adapter 或 repository。
- 验收路径不直接调用 M4 repository ingestion；必须存在从正常产品上游到 adapter 的非测试生产调用者，只有公开方法或验收专用调用者不算接入。
- 用户只有通过明确动作才创建 PersonalAction / Reminder；OpenLoop 不自动克隆 Todo，协调动作不反写 source owner。
- Notification 必须由正式内部事件产生，并证明 delivery/read/dismiss、可见读面和重启恢复；不要求用户手工创建通知。
- Decision 必须有独立 typed projection，覆盖 owner 的 OPEN / ANSWERED / EXPIRED / WITHDRAWN 映射；M4 本地 read/dismiss 不改变 owner 状态。
- source revision、dedupe、owner 隔离、敏感输入 quarantine、重启恢复和重复投递幂等继续成立。

### M4R03 — 服务端到期时钟与恢复

目标：让普通 scheduler 驱动 snoozed OpenLoop 和 Reminder 到期推进；renderer 只提交用户动作，不拥有时钟。

完成标准：

- “稍后提醒”到期后自动重新出现；Reminder 到时只触发一次。
- 到期前强退、到期后重启、重复 tick、并发 tick 和 CAS 冲突均不漏触发、不双触发。
- 单元测试可直接调用 transition，但产品验收必须证明 production scheduler 的实际调用链。

### M4R04 — 注册 owner 的精确回源

目标：让 server-minted route 通过注册 source-owner adapter 解析成有限、typed、不可执行的导航目标，并由目标页面实际消费 focus。

完成标准：

- source A 精确落到 A，source B 精确落到 B；相同 object id、不同 owner 也不串。
- unknown owner、过期 ref、revision 不匹配和目标不存在时明确失败，不显示虚假的“已到来源”。
- 禁止 raw path、任意 URL、callback、renderer 猜路由和通用 Projects 页面冒充精确回源。

### M4R05 — 持续 Secretary 对话

目标：接通首页消息输入、发送、turn 状态、响应/失败与重启恢复；会话真源继续是 M3 RoleSession / Turn / conversation transport，M4 只提供个人情境与 typed refs。

完成标准：

- 用户能连续发送至少两轮消息；消息和响应/失败绑定同一后端解析的 Secretary RoleSession、PersonalScope、daily channel 和既有 permission。
- 强退重启后恢复同一 RoleSession 和此前对话历史，并能继续下一轮；只恢复 RoleSession ref 或 deterministic brief 不算对话恢复。
- fake provider 失败在界面明确可见，不伪造成功，也不破坏 Attention、brief 和 DailyReport。
- 无用户消息、无实质事件时 agent/model invocation 继续精确为 0；显式用户消息才允许进入对话调用。
- 对话不自动创建 Todo、项目、工作流、Handoff、FormalMemory 或 Skill，也不扩大权限。

### M4R06 — 五类旧读面的实际 shadow / parity / fallback

目标：把五类 inventory-only 候选换成实际 server-owned shadow reader 输出；逐条完成 exact tuple、canonical reread、parity 或可解释 quarantine。

完成标准：

- 每类 adapter 必须读取实际 server-owned 来源，fixture 数据也从原 owner 的产品入口创建；每类均有匹配、空态、无法连接和拒绝反例。确实无旧行时记录 `EMPTY`，无法精确 join 时记录 `UNJOINABLE / QUARANTINED`，不得把 renderer 临时态升格为 owner。
- PARITY 证据来自生产 adapter，而不是测试手工构造完整候选。
- 至少一个实际可连接的旧面产生真实 PARITY 并在 fallback 可见；其余各类按实际结果记录 PARITY、EMPTY 或 UNJOINABLE。五个固定全空 candidate 不算实际 reader。
- 受守卫 compatibility fallback 能显示真正 parity 的旧项并精确回源；不产生 coordination write、owner write 或 effect replay。
- 旧面仍不物理删除，M9 退役边界不被提前执行。

### M4R07 — 普通产品隔离验收、全量回归与收口

目标：只做集成、故障、证据、回归、独立审查和文档收口，不夹带新的功能施工。

完成标准：

- 全新隔离 root；禁止 repository 预灌。逐名覆盖 Inbox、OpenLoop、Decision、PersonalAction、Reminder、Notification，以及到期唤醒、精确回源、两轮对话、legacy fallback、日报和空事件零调用。
- 隔离 profile 只允许替换 app-data root、server clock、fake provider 等基础端口；必须使用与普通启动相同的 AppState constructor、command registry、source dispatcher、scheduler、route resolver、conversation transport 和 legacy readers。禁止 acceptance wrapper、改名命令或专用 handler 替代普通产品链。
- source 从 owner 产品入口触发，到期由 scheduler tick 触发，对话从真实 UI composer 触发，legacy 候选由实际旧面 reader 产生；静态调用图和运行 receipt 同时证明这些入口与普通产品一致。
- 覆盖 transaction 中断、SIGKILL、同 profile 重启、重复事件、重复 tick、重复消息、fake provider 失败和回切；不产生重复业务 effect。
- 当前用户范围明确取消第 8 次 UI / Computer Use / PNG / attestation gate；不得生成或补造视觉证据。v2 receipt 必须把该范围记为 `NOT_EXECUTED / NOT_APPLICABLE`，同时保留第 8 次普通 `recovery_timer` 的真实 98 秒等待和后端恢复验证。
- M4 定向矩阵、完整 Rust、typecheck、全部离线入口、production build、launcher syntax、rustfmt 与冻结合同 exact 全部通过。完整 Rust 使用新建专用 `TMPDIR`，并验证 fixture 清理。
- Git/Harness、代码/测试、文档/下游三条独立只读复核完成；已知 P1 全部关闭，P2 如有必须如实列明并判断是否影响验收。
- v2 产品链完成标记成立后，再单独完成文档、独立复核和 `stage-07` 生命周期归档；在真正 close-stage 前只能写 closeout pending，不自动激活 M5–M10。

当前结果（2026-08-13）：

- `../harness/reports/M4R07-isolated-product-reacceptance-closeout-behavior-receipt.json` 使用 `syn.m4.isolated-product-reacceptance.behavior-receipt.v2`，结果 `PASS`，expected/observed launches 为 12/12。
- launch 8 保留普通 `recovery_timer`、真实 98 秒等待和后端恢复验证；`launch_8_ui_validation` 为 `required_by_current_contract=false`、`NOT_EXECUTED / NOT_APPLICABLE`，Computer Use 0 次，PNG、attestation、capture signal 均未写。
- `../harness/reports/M4R07-isolated-product-reacceptance-evidence/manifest.json` 使用 `syn.m4r07.closeout-evidence-manifest.v2`，只绑定 portable receipt SHA 与 `launch_8_ui_validation` canonical SHA。
- 该 PASS 不是视觉 PASS，也不覆盖真实资料、真实用户项目写入、真实模型/provider/connector、远端、部署或发布；`stage-07` lifecycle 仍 pending。

## 4. 依赖与单写顺序

```text
M4R01 → M4R02 ─┬→ M4R03 ────────────┐
                ├→ M4R04 → M4R06 ───┼→ M4R07
                └→ M4R05 ────────────┘
```

M4R03、M4R04、M4R05 都依赖 M4R02；M4R06 功能上依赖 M4R02 + M4R04；M4R07 必须等待 M4R02–M4R06 全部完成。普通 AppState、command registry、公共前端壳和验收 launcher 都是共享接缝；在同一主线施工时由开发主管保持单写、顺序提交，不让多个 leaf 并发改同一接缝。

## 5. 验证分层

| 层级 | 必须证明 | 不足以声称 |
|---|---|---|
| Contract / static call graph | owner、scope、入口、禁止旁路、冻结 hash | 功能可用 |
| Unit / property | transition、CAS、dedupe、route reject、parity comparator | production caller 已接入 |
| Ordinary composition integration | 普通 constructor、adapter、scheduler、resolver、conversation transport、legacy reader 都有非测试调用者 | UI 可见、强退恢复 |
| Non-test build | production path 可构建 | 产品交互正确 |
| Isolated product App | 普通入口、可见交互、SIGKILL/重启、fake failure、可携带证据 | 真实资料/provider/日常使用 |
| Fresh full regression | 新专用临时目录下全量回归稳定且清理干净 | 远端、部署、发布 |
| Independent bus review | Git、Harness、代码、测试、证据、边界和文档一致 | 自动激活下游 |

## 6. 整阶段退出门

以下全部成立后，才可完成 M4 修正阶段生命周期收口：

1. 五项 P1 均有普通产品生产调用链、正向证据和失败反例；测试旁路不再承担产品闭环证明。
2. M1/M3/M4 冻结边界保持 exact；没有 renderer 双真源、owner 反写、secret/raw payload 扩散或静默副作用。
3. 普通产品隔离后端/产品链和全量回归按当前合同通过；UI / Computer Use / PNG / attestation 的 `NOT_EXECUTED / NOT_APPLICABLE` 边界写入 v2 receipt，不冒充视觉结论。
4. 已知测试临时目录污染得到修正或被稳定隔离并有清理证明；历史 Harness 追溯瑕疵以勘误记录，不篡改归档。
5. current-state、总计划、M4 计划、计划索引、task queue 和下游交接只写新鲜事实与证据上限。
6. 独立复核与 Harness closeout 完成；随后也只允许建议新的窄包，不自动激活 M5–M10。

## 7. 授权、停止与保全

`stage-07` 与 M4R01…M4R07 已经实际启用；当前只执行 M4R07 文档与生命周期收口，不扩回产品功能施工。新的代码、真实资料/服务、Git 生命周期、远端或下游阶段仍需新的明确授权。

始终停止并回报：source owner / revision 无法精确绑定；修正需要 M5/M8 或真实数据/provider 才能成立；对话或日报会持久化 raw transcript、prompt、tool output、secret；协调动作会改 owner 事实；空事件触发模型；验收 launcher 只能靠 direct seed 通过；发现与冻结合同或产品正本冲突；目标文件存在不明 WIP。

继续只读保全：

- `/Users/yoyi/workspace/product-line-syn-fnd-002`
- `/Users/yoyi/workspace/product-line-syn-m2-closeout`
- M1/M3 冻结合同、M4 v1 合同正文、`stage-06`、M4C01–M4C10 归档和原 C09/C10 证据

push、merge、rebase、部署、发布、不可逆清理以及真实资料/服务动作仍需独立用户指令，不能从本计划或未来整阶段开发授权推导。
