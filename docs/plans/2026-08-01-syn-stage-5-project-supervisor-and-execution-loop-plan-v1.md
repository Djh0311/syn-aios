# Syn Stage 5：项目主管与执行闭环计划 v1

日期：2026-08-01<br>
阶段：`M5`<br>
状态：**PLANNED / NOT_ACTIVE / NO_EXECUTION_AUTHORITY。**<br>
上位计划：`2026-08-01-syn-personal-ai-workbench-master-development-plan-v1.md` M5。<br>
硬前置：M1 scope / workflow-owner / report / ExecutionGrant；M2 UoW；M3 Project Supervisor RoleSession。<br>
当前活动阶段 / 叶：无（`NONE`）；M3 尚未激活，本计划也未激活；本计划不授权真实项目写、执行器、Codex、桌面应用或产品代码。

权威顺序：当前用户指令 → `../../../AGENTS.md` → `../../AGENTS.md` → `../harness/plan.md` → 活动阶段（stage）/ 唯一活动叶（leaf）→ `../harness/authorization.json` → `../current-state.md` → 2026-08-01 修订与当前能力盘点 → master → M1-M3 退出回执 → 本计划。现有特殊路径零件只作迁移素材，不等于普通项目闭环。

## 0. 当前事实与未知

### 可保留 / 提取的现有能力

- Proposal、PlanAuthorization、dispatch、attempt、WorkerReport、review、UserDecision 等对象和多段治理链已经存在；
- trusted supervisor MCP binding 能冻结 project / root / workflow / run / thread / profile / role / capability，并在调用前复核；
- Runner 已有 cwd、write roots、deny-list、attempt、readback 与 process registry 守卫；
- 项目内 proposal / authorization / global review / report / final decision 的事实分层和两轴治理原则应保留；
- 部分 store 已 DB-primary 后做 JSON compatibility projection，可作为迁移材料。

### 当前缺口

- 全链没有统一 `CorrelationId / OrchestrationId / WorkflowRunId`；现状靠多个 id 临时拼接；
- `ExecutionGrant` 领域对象尚未成立，部分执行入口仍消费 caller 布尔矩阵、测试 path-lock 或 prepared-dispatch 形状；
- WorkerReport 未完整核 project owner、dispatch node、attempt、authenticated actor；report 仍可能只证明“有人提交了汇报”；
- `list_project_workflows` 仍有 slug `contains`，node query 丢弃 project root；这些必须由 M1 先止血；
- 普通 Jiaoban 是 project-agent conversation，不绑定 workflow；结构化项目流程主要锁在固定测试 / 特例路径，不能称为常驻项目主管；
- shared supervisor-read-only profile 目前只开放 proposal 与知识只读能力，不是完整项目主管 capability 面；
- 目标 `ProjectSummary` 不存在；当前 page query 仍构建完整 snapshot 后切片。

### HOLD / 需冻结决定

- 普通项目如何登记、归档与解析稳定 ProjectId；
- stop / retry / resume 的真实进程结果、receipt、重复命令与崩溃恢复；
- Proposal → Authorization → Dispatch → Report → Review 各断点的 restart 语义；
- ProjectSummary schema、freshness watermark、敏感裁剪、source refs、失效和重建；
- 单 agent / 多 agent 协调策略的确定性输入；
- 真实 Codex、Runner、项目写、DB/JSON live state 与真实 App 表现。

## 1. 阶段目标

1. 普通项目拥有持久 Project Supervisor RoleSession，并默认只读理解本项目；
2. 对话中的明确动作经 Proposal、用户确认、Authorization、ExecutionGrant 后才进入执行；
3. 全链共享稳定 correlation / run identity，所有 report / review / decision 可精确 join；
4. Runner 只消费控制核心服务端加载的 immutable grant；
5. Project Supervisor 按协调复杂度选择单 / 多 agent，风险治理另由 policy 决定；
6. stop / retry / resume 名称与真实进程语义一致，失败可恢复、重放幂等；
7. 生成可重建、最小、只读 `ProjectSummary` 给 Secretary / Global Supervisor；
8. 保留现有知识、记忆、Harness、MCP、Runner 能力，通过稳定 ports 重组，不推倒重来。

## 2. 本阶段不做

- 不把普通聊天自动转 Proposal / Task / Workflow；
- 不让 Project Supervisor、模型、adapter 或 Runner 自授执行权限；
- 不用 test project / fixed Mario chain 定义通用业务语义；
- 不让 report 直接成为 verified project fact；
- 不让 ProjectSummary 复制项目原文、raw transcript 或可反写项目；
- 不让 Global Supervisor 建议自动改变项目；
- 不在 M5 退役旧 command / store，也不做物理删除；
- 不重做已确认的项目页布局或把 UI 调整混进 execution kernel 包；
- 不以 fake runner、path-lock 或 debug build 冒充真实项目执行。

## 3. owner、身份与不变量

| 对象 | 唯一 owner | 关键 join / 不变量 |
|---|---|---|
| Project / ProjectSupervisor | project domain | stable ProjectId；root 只作 resolver 验证后的定位别名 |
| Proposal / Authorization / Grant | project orchestration | 各自稳定 `ProposalId / AuthorizationId / GrantId` + `ProjectId` + canonical orchestration/correlation identity；Grant 精确引用 `AuthorizationId + authorization_revision`。revision 只作 expected-version，不替代对象 ID |
| WorkflowRun / WorkItem / Dispatch / Attempt | execution aggregate | `ProjectId + OrchestrationId + WorkflowRunId`，再加对象自身 ID；Attempt 先以不可执行 `PreparedAttempt` 分配稳定 AttemptId |
| ExecutedReport | claim ledger | exact project/workflow/item/node/dispatch/attempt/grant/worker RoleSession/authoritative execution receipt/actor/hash，执行 join 全部必填 |
| ManualOfflineClaim | claim ledger | authenticated submitter + source/evidence refs；禁止伪造 Grant / Dispatch / Attempt / execution receipt，也不得提升为“已执行事实” |
| AuthorizationDecision | project authorization | 对 Proposal / scope / risk 的执行前决定；不得复用结果决定的 receipt / idempotency key |
| ResultUserDecision | review domain | 执行后对 report / review 的最终事实决定；引用同一 run，与前置授权严格分型 |
| ProjectSummary | project projector | 最小、source-backed、watermark、可重建、只读、不可反写 |
| runner / side-effect command | infrastructure adapter | 只经 `ExecutionGrantGateway` 消费服务端加载的 Grant；不得用普通 capability 替代 |
| project conversation / MCP read & proposal tools | infrastructure adapter | 经 `RoleSession + ConversationCapabilityGateway`；普通只读 / submit_proposal 不要求 ExecutionGrant，也不得产生副作用 |

PRJ-001 必须裁决 `CorrelationId` 与 `OrchestrationId` 是同一 canonical ID 还是明确父子关系，以及 `WorkflowRunId` 的唯一分配时点。签发顺序固定为：Authorization 后建立 Run / WorkItem 与 worker RoleSession binding，在 M2 UoW 中创建不可执行的 `PreparedAttempt` 并分配稳定 AttemptId，再 mint 精确绑定该 AttemptId 的 attempt-scoped ExecutionGrant；grant 持久化并 readback 通过后才创建 Dispatch、把 Attempt 推进到可运行态并经 outbox 启动。ExecutionGrant 至少冻结 actor、worker / RoleSession、AttemptId、scope、允许 command、cwd / write roots、object refs、authorization id + revision、policy decision、expiry / revocation、idempotency / effect key 和 grant hash；调用参数只能收窄，不能补充权限。mint / readback / dispatch 失败必须撤销 Grant 并把 PreparedAttempt 留在不可运行的可恢复 / 已取消状态，不得留下可复用 Grant 或可运行 Attempt。

## 4. 任务切片

### SYN-PRJ-001 — 项目执行合同与现状映射

冻结全链对象自身 ID、canonical orchestration/correlation identity、WorkflowRunId 分配、PreparedAttempt 状态机、owner、restart points、两类 user decision、上述 Grant 签发 / readback / revoke / dispatch 顺序与 principal、完整 grant 字段、分型 report kind / trusted actor、两类 gateway、ProjectSummary query port 和旧对象 mapping。只写合同 / migration matrix。

### SYN-PRJ-002 — Orchestration identity 与 exact joins

在 M1 owner guard 上引入稳定 OrchestrationId / CorrelationId / WorkflowRunId；把 proposal、authorization、dispatch、attempt、report、review、decision 精确串接。无法映射的 legacy record 隔离。

### SYN-PRJ-003 — WorkerReport claim 与事实提升

接 M1 精确绑定并分型实现：`executed_report` 必须引用 exact Grant / Dispatch / Attempt / worker RoleSession / authoritative execution receipt，由可信 binding 建立 actor，不接受 `actor_role` 自报；`manual/offline_claim` 只绑定 authenticated submitter 与 source/evidence refs，禁止伪造执行 join，不能提升为“已执行事实”。实现 acceptable attempt state、readback / review / ResultUserDecision 提升规则和持久化失败恢复；actor、node、attempt、grant、project 任一错配均零业务写。

### SYN-PRJ-004 — ExecutionGrant → Runner / CapabilityGateway

拆分 `ExecutionGrantGateway` 与 `ConversationCapabilityGateway` 及两组拒绝测试。服务端只在 Run / WorkItem / worker RoleSession binding 和不可执行 PreparedAttempt 已持久化后 mint 精确绑定 AttemptId 的 Grant；副作用 command / Runner 只接受 Grant id，并逐字段对照 actor、RoleSession、AttemptId、scope、command、cwd/write roots、object refs、authorization revision、expiry/revocation 和 effect key；调用参数不得补充权限。只读 project conversation / knowledge / submit_proposal 继续走 RoleSession + capability，不得拿普通 capability 替代 Grant。每次只迁一个入口。

### SYN-PRJ-005 — 持久 Project Supervisor 应用服务

接 M3 RoleSession。默认只读访问本项目 facts、conversation、knowledge、memory packet、blackboard、Harness；结构化动作必须显式展示 Proposal / permission request。能力最小化，按需升级。

### SYN-PRJ-006 — 受控执行与恢复

把保留的 runner / workflow 能力收进 aggregate；定义 dispatch、attempt、stop / retry / resume、timeout、worker blocked、provider failure、receipt lost 和 restart recovery。外部副作用遵循 M2 outbox。

### SYN-PRJ-007 — ProjectSummary projector

建立 versioned schema、watermark、source refs、敏感裁剪、失效 / 重建，并交付 `ProjectSummaryQueryPort`：按 consumer RoleSession / scope / policy 每次判权。Secretary / Global Supervisor 只能调用该 port，不直接读项目 store、projection 或 root，也不复制项目真源。

### SYN-PRJ-008 — 项目 UI 收口

保留项目壳和专业 tab；默认 Project Supervisor conversation；方案 / 审批 / 运行只在明确动作后打开。DTO 冻结后独立包接入，不改 execution kernel。

### SYN-PRJ-009 — scratch project 分层验收

隔离项目覆盖 read-only、用户拒绝、单 allowlisted fake write、worker blocked、runner failure/recovery、global advice、user decision、summary 投影。真实项目、真实 Codex、真实 runner 各自另行授权。

## 5. 顺序、并行与写所有权

```text
PRJ-001 → PRJ-002 → PRJ-004 → PRJ-003 → PRJ-005 → PRJ-006
                                             └→ PRJ-007 → PRJ-008
PRJ-006 + PRJ-007 + PRJ-008 ─────────────────────────────→ PRJ-009
```

- project orchestration aggregate 由 M5 单写；M1 policy / owner guard、M2 UoW、M3 RoleSession 均由其 owner 提供 adapter；
- `commands.rs`、AppState、SQLite schema、command registry、workflow assembly、WorkbenchSnapshot、App shell 是公共承重面，任务包必须唯一 writer；
- M4 与 M5 可在公共合同冻结后并行，但不得共同改 event/schema/App assembly；
- M6 依赖 ProjectSummary；M5 不实现顶层 Global Supervisor 或成员目录；
- Rust domain 与 React consumer 在 DTO 冻结后可并行；UI visual 与 read-model 切换不混包；
- 目标文件 opening hash 与任务包不符或存在未归属 WIP 即停。

## 6. 迁移与回滚

- legacy ids 先建立 exact mapping / quarantine，不再使用 `contains` 或路径猜 owner；
- 每迁一个入口，标记 `new-grant / guarded-legacy / blocked`，保留旧读 / export；
- Project Supervisor 对话先 shadow 旧 Jiaoban 显示，不把前端 cache 当 session owner；
- grant 可撤销、过期、按 revision 失效；rollback 不恢复 caller 自报授权；
- execution attempt 与外部 effect id 幂等；receipt 丢失后 readback 决定状态，不盲重跑；
- grant mint / readback / dispatch 任一点失败都保持 Attempt 不可运行并撤销不可用 grant；恢复只能按同一 AttemptId 重新判定，不得复用到另一 attempt；
- ProjectSummary 可从项目事实重建，rollback 只切旧展示，不反写项目；
- 旧 fixed chain / workflow machine / compatibility JSON 到 M9 才逐项 unregister / archive。

## 7. 验证矩阵

| 层级 | 必须证明 | 不能声称 |
|---|---|---|
| Contract / static | owner、join、grant、状态和 capability 一致 | 运行链已接通 |
| Unit / property | 跨项目拒绝、grant revocation、report binding、幂等 transition | Runner 可用 |
| Temp + fake runner | crash points、stop/retry/resume、outbox、summary rebuild | 真实项目执行 |
| Non-test build | production path 可构建 | 桌面行为正确 |
| Isolated Tauri scratch | 用户可见 proposal→decision 闭环、failure/recovery、source links，并保留桌面窗口截图 / 交互 / deep-link 点击证据 | 真实 Codex / 写根通过 |
| 经授权真实 project | 指定项目、grant、runner、readback 的精确场景 | 其他项目、发布或无人值守通过 |

关键验收：前置 AuthorizationDecision 与末端 ResultUserDecision 不可互换；用户拒绝零 spawn / 零业务 mutation；report 不冒充事实；runner 只消费完整服务端 Grant；两个 project 的 id / root / workflow / grant / actor 互串全部 fail closed；ProjectSummary 与源 watermark 一致且不可反写。

## 8. 独立授权与停止条件

scope / owner 修复、local schema / store migration、App 启动 / 强制退出、report 绑定、ExecutionGrant、每个 runner 入口、真实 Codex、真实项目写、真实 App、旧执行入口关闭分别建包。M5 不授权真实 connector、凭据、跨项目写、自动连环、Git 或发布。

立即停止：owner / join 不唯一；真实执行仍依赖 caller bool / path-lock；report 缺 attempt / actor；权限放宽；restart 只能猜；ProjectSummary 可反写；UI-only guard；迁移无 rollback；WIP 写面冲突；fake runner 被表述成真实执行。

## 9. 阶段退出与 M6 输入

全部满足才进入 M6：

- 普通项目 Project Supervisor RoleSession 与项目 facts / tools 的最小只读能力成立；
- Proposal → AuthorizationDecision → Authorization → Run / WorkItem + worker RoleSession binding → PreparedAttempt(稳定 AttemptId、不可执行) → attempt-scoped Grant → Dispatch → runnable Attempt → ExecutedReport → Review → ResultUserDecision 以对象自身 ID + canonical orchestration identity 精确 join、可恢复、幂等；manual/offline claim 保持非执行分型；
- runner 只消费服务端 grant；stop / retry / resume 结论与实际能力一致；
- ProjectSummary version / watermark / source refs / rebuild / ACL 与 `ProjectSummaryQueryPort` 冻结；
- scratch isolated App 全链通过，真实项目证据按授权单独结算；
- 旧 fixed path 有 manifest / compatibility / rollback，未删除；
- 把 ProjectSummary 和 temporary agent attempt/report 合同交 M6；
- `../current-state.md` 回写实际完成与暂缓项；M6 未激活不得续跑。
