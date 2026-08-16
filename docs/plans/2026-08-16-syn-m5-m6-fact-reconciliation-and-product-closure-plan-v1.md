# Syn M5/M6 事实重整与产品闭环计划 v1

日期：2026-08-16<br>
里程碑：`M5 / M6 corrective closure`<br>
状态：**PLANNED / NOT_ACTIVE / NO_EXECUTION_AUTHORITY**<br>
当前实现定级：**M5/M6 CANDIDATE WIP / UNIT-LEVEL PROTOTYPE / NOT_ACCEPTED / NOT_MAINLINE**<br>
当前 Git 锚点：WSL `main@9103c3b26b060e854be119a8cedaa856a2a900ce`；迁移、Harness Lite 0.8、架构文档与 M5/M6 候选仍混在未提交工作树中。<br>
上位计划：[`Syn 总开发计划 v1`](2026-08-01-syn-personal-ai-workbench-master-development-plan-v1.md)、[`M5 阶段计划 v1`](2026-08-01-syn-stage-5-project-supervisor-and-execution-loop-plan-v1.md)、[`M6 阶段计划 v1`](2026-08-01-syn-stage-6-global-supervisor-and-internal-organization-plan-v1.md)。<br>
拟用 Harness 生命周期：真实 `stage-14` 完成 M5；M5 独立验收后才建立真实 `stage-15` 完成 M6。过去只在 `docs/harness/plan.md` 手工勾选的 14/15 不视为已发生的 stage 历史。<br>

## 0. 权威定位与启动边界

本计划是现行 M5、M6 阶段计划下的事实恢复和补齐实施计划，不是产品正本，不降低原 M5/M6 退出条件，也不把现有原型改名成完成。

读取顺序：当前用户指令 → 有效 workspace/repo `AGENTS.md` → 产品正本、权威登记、现行架构与前端显示边界 → Harness plan → 当前活动 stage、唯一 current leaf → authorization → `current-state.md` 与当前源码/新鲜验证 → 总计划 → M5/M6 阶段计划 → 本计划。stage/leaf 只投影当前施工写域与生命周期，不能改写产品正本；但当前实施计划也不能越过它们扩大写域或续跑权限。

本文件存在不产生施工权限。当前 `authorization.json` 是 malformed（格式不合法）的 fail-closed 状态；仓内没有真实 stage-14/15、M5/M6 current/done leaf 或阶段验收报告。历史任务包、57/21 测试摘要和手工完成勾选都不能补造授权、生命周期或产品完成事实。

本计划不重新询问已经由产品正本和阶段计划回答的问题。schema、字段、阈值、fixture、迁移脚本、精确写域和测试选择由开发主管在任务包内决定；只有出现产品正本冲突、需要扩大真实数据/服务/设备/发布范围或无法保全既有 WIP 时才回到用户。

M5 与 M6 分别由独立开发主管任务施工。总线只负责计划、阶段间门禁和独立验收；M5 完成不会自动激活 M6，M6 完成不会自动激活 M7。

## 1. 独立复核后的事实基线

### 1.1 可以保留的成果

- M5/M6 新模块可以编译，身份类型、PreparedAttempt 状态机、trait 形状和现有单元测试可作为候选素材。
- 独立定向复跑中，M5 过滤测试 85 项通过，其中当前新增 M5 测试可对应 57 项；M6 定向测试 33 项通过。
- 新对象方向与产品正本一致：项目主管、全局主管、稳定成员和临时 Agent 分型，以及角色不绑定模型/provider/thread/process 的原则没有冲突。
- Harness Lite 0.8 runtime 本体仍为 current，Codex 四事件仍指向 WSL 单 dispatcher；问题在项目控制状态，不在 Lite runtime 漂移。

### 1.2 不能继承为完成事实的部分

- M5/M6 在生产调用图中只有模块声明；主要 service/gateway/controller 只有 `#[cfg(test)]` 内存实现，没有 AppState、Tauri command、正式 repository、普通产品 caller 或前端入口。
- 当前 M5 原型允许任意字符串 Grant 使 Attempt 进入 Runnable；Project Supervisor 测试服务不持久保存状态；stop/retry/resume 不改变持久状态；WorkerReport 完整性检查缺少完整 Grant/Dispatch/Attempt/RoleSession/receipt/actor exact join。
- 当前 M6 session/Handoff 是平行内存结构，不是 M3 RoleSession/Handoff；跨项目 query 不消费正式 M5 ProjectSummaryQueryPort；成员目录与临时 Agent 历史不是受治理持久投影。
- 没有 M5 Runtime conformance、ProjectSummary、项目 UI、scratch isolated App；没有 M6 顶层入口、Secretary consult、临时 Agent 完整历史、独立多视角、成员 UI 或双项目 isolated App。
- `main` 仍停在 M4；M5/M6 没有独立 commit、原始测试 receipt、stage/leaf 归档或可携带验收报告。
- `docs/harness/plan.md` 宣称 M5/M6 完成，但 `current-state.md`、总计划、M5/M6 阶段计划和计划索引仍为 `PLANNED / NOT_ACTIVE`。

因此，所有当前 M5/M6 字节统一视为“进入本计划前的未验收候选 WIP”：可以复用、修正或淘汰，但不得默认继承包完成状态。

## 2. 计划目标与明确不做

### 2.1 最终目标

1. 在不 reset、stash、clean、覆盖或丢失现有 WIP 的前提下，建立可恢复、可归责、可逐层提交的 WSL 事实基线。
2. 恢复 Harness Lite 0.8 的真实 fail-closed 控制状态，从当前时刻建立合法的 stage/leaf 生命周期，不倒填历史授权或回执。
3. 按现行 M5 计划完成普通项目的持久 Project Supervisor、完整执行授权链、恢复语义、ProjectSummary、项目 UI 和隔离产品 App 验收。
4. M5 独立验收通过后，按现行 M6 计划完成持久 Global Supervisor、跨项目咨询、稳定/临时成员、多视角、产品 UI 和双项目隔离 App 验收。
5. 每个阶段形成独立 Git 载体、新鲜验证、Harness 退场、事实文档和下游交接；任何局部测试不得再冒充整阶段完成。

### 2.2 本计划不做

- 不接真实个人资料、真实用户项目写入、真实模型/provider、真实消息、账号、凭据、connector 或外部网络业务动作。
- 不运行真实用户日常环境；M5/M6 的产品层证据只使用隔离 app-data、scratch projects、fake roles/provider/runtime 和明确白名单的合成动作。
- 不修改 M1–M4 冻结合同正文；若实现解释确需补充，只能新建不改旧 hash 的增补合同。
- 不推翻项目壳、前端显示边界或重画整个工作台；不做移动端、HR、薪酬、多租户组织图。
- 不物理删除旧执行入口、旧 review、Agent Center 或 compatibility 数据；退役仍留给 M9。
- 不自动生成 M7 正式记忆、PersonalFact 或 Skill；M6 只交付带来源的 advisory/member/source-ref 事件合同。
- 不 push、merge、rebase、部署、发布；不 reset、stash、clean 或覆盖既有 WIP。必要本地 commit 只在未来明确施工授权内进行。

### 2.3 DSH 方法吸收后的当前架构硬边界

本计划以当前有效的 [`Syn 原生治理核心与受治理自升级方向决定`](../../decisions/2026-08-16-syn-native-governance-core-and-governed-self-upgrade-direction-v1.md)和[`系统架构正本`](../workbench-system-architecture-v1.md)为准，不沿用 DSH 进入前的旧执行架构口径：

- Identity、Scope、Policy、ExecutionGrant、正式事实/事件/审计、长期记忆晋级、Budget、CredentialRef/ConnectorGateway、独立 Verifier、Updater 信任链、Kill Switch 和最终完成判定属于 Syn 不可外包的治理根。
- Syn 原生实现 vendor-neutral `AgentRuntime` / `AgentWorkcell` 合同和默认执行路径；DeepSeek Harness、Codex 或其他 runtime 只可能作为可替换 adapter，不能成为身份、事实、权限、记忆、持久 operation 或验收真源。
- M5 必须吸收 Service/Provider/Consumer 接缝、Profile/Bundle/Patch 组合、append-only runtime event、可重建模型上下文、Step/Turn、pre/execute/post Tool Pipeline、checkpoint、`OUTCOME_UNKNOWN` 和版本化 Child Run/Tool/Profile 候选方法。
- runtime event/trace 只记录模型实际看见和做过什么，不自动成为 WorkbenchEvent、AuditEvent、项目事实、长期记忆或完成结论。
- runtime Approval/Sandbox 只是第二道防线；DSH Goal/Job/Schedule/Workflow 不能替代 Syn durable queue、lease、timer、retry、dead letter、effect ledger 和恢复 owner。
- Dynamic Cordis/Plugin/Package 默认关闭；即使生成成功，也只形成受治理候选，不能自批、自签、自装或进入生产。
- M5 的完成前提是通用合同与 Syn 原生/fake conformance 闭合，不要求安装或接入真实 DSH。外部 DSH adapter 只有在后续具名任务中按相同 conformance、安全、维护和成本证据决定；M11 自升级链不因本计划激活。
- M6 必须继续把 runtime session/child run 与 RoleSession、StableMember、TemporaryAgent 和组织关系分开。

## 3. 共用恢复门：REC-00 — 事实、Git 与 Harness 重建

REC-00 是 M5 的第一个 leaf，也是任何产品代码继续施工的硬前置。

### 3.1 写前 R0 恢复载体

先做 R0a 元数据预检，再做 R0b 受控归档，不能在安全分类前打包整个工作树。

R0a 只读取元数据：

- 写前必须精确为 branch=`main`、HEAD=`9103c3b26b060e854be119a8cedaa856a2a900ce`、index 相对 HEAD 无 staged delta；本地 `origin/main` 只记录为未联网 tracking fact，不 fetch；
- HEAD、tree、branch、本地 upstream ref、index SHA；
- Git-visible tracked/untracked 路径；ignored 路径只记录名称、数量、`lstat` 类型和 size，不读取内容；
- `.git`、`target`、`node_modules`、`.env`/密钥候选、活动 DB/运行数据、旧 rollback carrier、symlink、special file、未知 ignored path 和未知大文件一律先 HOLD；
- 发现 secret/credential 候选时只记录存在与元数据，不查看、复制或输出内容。

R0b 只归档 tracked 与逐项批准的 Git-visible untracked，至少冻结并校验：

- `status --porcelain=v2 -z`、tracked/cached binary diff 与 index SHA；REC-00 只验证恢复载体，不实际恢复 index；
- 经批准的 untracked 路径、普通文件 mode、size、SHA-256 和可恢复 tar；
- 批准范围内的 regular-file manifest；
- ownership、AGENTS/CLAUDE managed block、两套 Hook、authorization、Harness plan/stage/leaf、current-state；
- 现场已接受的 D0 `SOURCE_BYTES_MATCH` receipt、其实际 attempt ID 与最终目标 manifest；
- Lite 0.8 升级后的候选 digest、managed runtime identity 和路径基线；
- 当前 `.bak`、临时文件和其他不应进入候选的普通字节，先保存后裁决。

归档必须使用相对路径、no-dereference，并拒绝绝对路径、`..`、重复成员、symlink 和 special file。被排除对象只留元数据；读取秘密或复制受保护内容需要另行授权。

R0 必须在独立临时目录中完成清单复核和只读可恢复性检查。写前 HEAD/branch/index 不满足上述锚点，或冻结前后 HEAD、index、路径集、任一字节并发变化，立即停止并由总线重建计划基线，不猜归属。工作树状态行数不作为固定门。

### 3.2 分层归责

建立 provenance manifest，只分类、不搬动原文件：

| 层 | 内容 |
|---|---|
| L0 | `main@9103c3b` 的 M4 主线 |
| L1 | D0 receipt 冻结的原始迁移 WIP |
| L2 | Harness Lite 0.8、Hook、D0D closeout |
| L3 | D1A 与服务器/Primary-Edge 架构文档 |
| L4 | M5 candidate WIP |
| L5 | M6 candidate WIP |
| LX | 共享、重叠或无法证明来源的字节 |
| LR | 从当前时刻重新审阅并由新授权诚实采纳的治理/事实字节 |

不得用 mtime、文件名或口头声明猜归属。共享文件按 hunk 形成精确投影。对每个 LX hunk 记录 R0 hash、语义 owner、来源证据、当前裁决和新 hash；历史作者无法证明但当前内容可被重新验证时，可由新授权从现在开始采纳为 LR，必须明确“当前重新采纳，不恢复历史归属”。未审阅 LX 继续阻断。

### 3.3 恢复真实 Harness 状态

- R0a/R0b 完成后仍不写仓库。随后用 R0 preimage 执行一次具名、可回滚的 bootstrap transaction：把 authorization 恢复 closed，纠正 Harness plan/current-state，创建 stage-14 与 REC-00 leaf；任一步失败整体恢复 preimage，不留下半套 current chain。
- 保存当前 malformed authorization 原字节与 SHA，只把它记录为“无效控制状态”，不冒充历史授权。
- 将仓内 authorization 恢复为 Lite 0.8 精确 closed 两字段；不得手工编造 executionReceipt、session、turn 或 expiresAt。
- 撤销 `docs/harness/plan.md` 中 stage-14/15 的假完成勾选，把现有 57/33 测试准确写为“候选原型单测基线，未绑定阶段退出”。
- 把当前状态统一回写为 `M5/M6 CANDIDATE WIP / NOT_ACCEPTED / NOT_MAINLINE`。
- 从当前时刻创建真实 `stage-14` 和唯一 REC-00 current leaf。`stage-12`、D0C04、D0C05 原位保全，不恢复、不关闭、不归入 M5/M6。
- 先以 closed auth 创建 stage-14/REC-00 leaf，并只读证明 `currentCount=1`、唯一 leaf 的 `stageId=stage-14`、selected stage 为 stage-14；只有随后由受支持宿主在同一 WSL project/session/turn 产生真实 UserPromptSubmit receipt，才允许 Lite 0.8 签 active 六字段。直接运行 dispatcher、synthetic 输入、手填 observed/receipt 都无效。
- active authorization 必须经 runtime `read/validateStop` 验证，不能只看 JSON 外形；每个 leaf 退场前先回 closed，新 leaf 从 closed 开始并重新签发与当前 leaf 精确绑定的 active 状态，旧 active JSON 禁止跨 leaf 续用。
- 若 WSL Hook 尚未 trusted/observed，始终保持 closed，只执行当前用户明确回合，不允许 Stop 自动续跑。
- 使用只读 runtime API 验证：`stages` 同时保留 stage-12/stage-14；`currentCount=1`；唯一 current leaf 指向 stage-14；selected stage 为 stage-14；D0C04/D0C05 仍只在 unfinished 且 `stageId=stage-12`；stage-14 progress 不计入 D0C04/D0C05。任何移动、改写或恢复这两个 D0 leaf 都硬停。
- 不使用会附带写 `.turn` 的快照命令冒充只读检查。
- chain 健康后，才在 REC-00 写域内建立 `docs/harness/reports/M5M6-REC00-fact-freeze.md` 与 `M5M6-REC00-provenance.json`。初版只绑定 R0 路径、manifest SHA、现场事实和 unresolved LX/LR；L1–L3 commit/tree 在 3.4 形成后，以明确 finalized section/revision 补入，不预填未来值、不倒填时间。

用户对 stage-14 的一次明确授权决定整个 M5 的目标和范围；leaf 是工作投影，不是新的产品授权。每次 leaf 切换仍必须先 closed，再用同一真实 stage-start receipt 重新签发，且仅限同一受支持 project/session/turn、有效 lease 和 runtime 校验通过的情况。若宿主不能合法重绑、receipt/lease 已失效或已跨 turn，则保持 closed 并停止自动续跑，只请求用户发送“继续”恢复宿主控制链；不得把这个运行时动作包装成重新拍板产品需求，也不得伪造 receipt。

### 3.4 建立前置本地载体

在 M5 产品提交前，已验收的 L1/L2 必须按来源形成 reviewed local commits 并进入 M5 candidate ancestry；仓外 tar 只能恢复，不能替代 Git 父链。L3 若已独立验收则单独提交；若未验收，先验收或留在隔离分支，不能漂在主线 working tree 后继续 M5。LR 使用独立 truthful-adoption 治理提交，未解决 LX 不得进入任何阶段载体。

只精确 staging 已分类 path/hunk；禁止 `git add -A` 吞入混合 WIP。每层 commit tree 都在任务具名的 disposable worktree/clone 与该层 manifest 精确比对。只删除本任务创建且已验 hash 的临时对象，不动权威 worktree、旧 recovery carrier 或用户其他临时目录。

M5/M6 candidate 在这一门内不因被保存或编译而获得验收地位。每个后续候选 commit 必须在 disposable checkout 中复跑验证，禁止让工作树中未提交的另一阶段代码掩盖依赖缺口。

REC-00 还必须形成 fresh prerequisite matrix：

- M1：Project/scope/workflow owner 精确；`list_project_workflows` 不再用 slug `contains` 猜归属，node query 不丢 project root；Proposal/Authorization/ExecutionGrant/Report owner 唯一。
- M2：UoW、outbox、effect/readback 和失败原子性可供 M5 production adapter 使用。
- M3：Project Supervisor RoleSession/repository/Handoff 是唯一身份真源。

REC-00 只冻结和判定，不修产品代码。矩阵逐项记录 `PASS / GAP / HOLD`；出现 GAP 时，REC-00 以 `FACT_FREEZE_COMPLETE / PREREQUISITE_GAPS_IDENTIFIED` 退场并路由到 M5R00，禁止在恢复门写域内顺手施工。出现产品正本冲突、合同必须改版或来源无法证明则记 HOLD 并回总线。

REC-00 退出时必须具备：可验证 R0、完整 provenance manifest、truthful closed/active 控制状态、真实 stage-14/leaf、前置载体、前置矩阵、明确下一路由和逐包 opening hash。

## 4. M5 修正包：stage-14

### M5R00 — 前置实现与 adapter 修正（按矩阵条件进入）

只有 REC-00 前置矩阵出现 GAP 时建立本 leaf；全部 PASS 时记为 `NOT_NEEDED`，不得制造空施工。

完成范围：在 M1/M2/M3 既有冻结合同下修正 owner query、UoW/outbox/effect readback 或 RoleSession/Handoff production adapter，使 M5 能复用唯一真源。禁止修改冻结合同正文、另造第二套 owner 或提前实现 M5R01 以后能力。

退出证据：REC-00 矩阵全部转为 PASS；每个修正有直接反例、non-test caller 和独立内容提交。若需要改变产品需求或冻结合同版本，立即停止交总线，不在本 leaf 自行决定。

### M5R01 — 执行合同矫正与旧数据映射

产品结果：一次项目动作只有一套明确身份和完整链路，不能靠几个字符串拼出看似有效的执行记录。

完成范围：

- 冻结 ProposalId、AuthorizationDecisionId、AuthorizationId、ProjectId、CorrelationId、OrchestrationId、WorkflowRunId、WorkItemId、NodeId、DispatchId、AttemptId、GrantId、worker RoleSessionId、RuntimeReceiptId、ResultUserDecisionId、trusted actor 和 hash 的 owner、分配时点与精确关系；明确 CorrelationId 与 OrchestrationId 是同一标识还是父子关系。
- 修正 WorkerReport：executed report 必须完整核对 `ProjectId + CorrelationId/OrchestrationId + WorkflowRunId + WorkItemId + NodeId + DispatchId + AttemptId + GrantId + worker RoleSessionId + authoritative receipt + trusted actor + hash`。
- executed、manual、offline report 彻底分型；后两类不能填写或伪造执行 join。
- executed report 必须由显式输入建立；禁止缺省 ReportKind 或缺字段输入自动成为执行报告。
- 冻结 legacy mapping/quarantine；无法精确映射的旧记录只隔离，不猜测补齐。
- 把当前原型逐项裁决为 `KEEP / REWRITE / QUARANTINE`，先建立能复现现有缺口的 red probes。

退出证据：任一 ID 错配、跨项目串接、actor 自报或 Grant 缺失都在业务写前 fail closed；合同/schema/迁移矩阵一致；non-test build 通过。现有单测只是回归输入，不是本包完成理由。

### M5R02 — 持久编排核心与 ExecutionGrant

产品结果：用户确认后的动作可以安全、持久地准备，但 Grant 完整持久化和回读前绝不运行。

完成范围：

- 用正式 store/UoW 落地 Run、WorkItem、worker RoleSession binding、PreparedAttempt、Grant、Dispatch 和 outbox。
- 严格实现 AuthorizationDecision → Authorization → Run/WorkItem + worker RoleSession binding → PreparedAttempt → mint Grant → persist/readback → Dispatch → Runnable → outbox。
- 建立 production ExecutionGrantGateway/ConversationCapabilityGateway；副作用入口只接 GrantId，由服务端加载完整 immutable Grant。
- 完整核对 actor、RoleSession、Attempt、scope、command、cwd/write roots、object refs、authorization_id/revision、policy decision、expiry/revocation、idempotency/effect key 和 grant hash。
- 所有 Runner/side-effect 入口登记为 `new-grant / guarded-legacy / blocked`；不得留下未知旁路。

退出证据：进程重启可读回同一链；mint/persist/readback/dispatch 任一点失败时撤销不可用 Grant，PreparedAttempt 留在不可运行的可恢复/已取消状态；错项目、过期、撤销、错误 revision 和调用方扩权全部拒绝；至少一个非测试副作用入口只消费服务端加载的 Grant。普通项目 production caller 由 M5R04 证明。

### M5R03 — WorkerReport、独立审查与事实提升

产品结果：执行者提交的内容只先成为声明；只有精确对上真实执行链并经过独立 Review 和 ResultUserDecision 才能成为项目事实。

完成范围：

- 把 R01 合同接入正式 WorkerReport 消费、持久化、readback 和恢复路径。
- 建立与 runtime/worker/report producer 分离的 production `RuntimeReceiptVerifier`：按 Grant、Attempt、effect readback、trace hash、actor binding 和 enforcement status 验证 M5R05 产出的原始 RuntimeReceipt。
- actor 只来自可信 session/binding；前置 AuthorizationDecision 与末端 ResultUserDecision 不得复用。
- executed/manual/offline 分型处理；重复 report、重复 decision 和持久化中断保持幂等。

退出证据：完整 exact-join 反例；producer 自验、伪造 receipt、trace/effect 错配、重复验证和 actor 错绑均拒绝；未验证、冲突、degraded 或 `OUTCOME_UNKNOWN` receipt 只保持 claim/待协调，不能进入事实提升；错配或持久失败零部分写；重启和重放不产生第二份事实；manual/offline 永不冒充执行成功；receipt/source/evidence refs 可追溯。

### M5R04 — 普通项目的持久 Project Supervisor

产品结果：用户进入普通项目后面对可恢复的项目主管会话；默认理解和只读，只有明确要求动作时才进入 Proposal 和授权步骤。

完成范围：

- 复用 M3 正式 RoleSession/repository，不自建字符串 session 真源。
- 接项目 facts、conversation、knowledge/memory packet、blackboard 与 Harness 的最小只读能力。
- 普通对话与 Proposal 分开；Proposal/Authorization 继续由既有 M1 owner 持有，经 M5R02 编排，执行结果再进入 M5R03；Project Supervisor 不得直接调用 start/dispatch 绕过 Grant。
- 建立正式 AppState、command、service caller；UI 在 M5R07 接入。

退出证据：强退/重启恢复同一 RoleSession；两个项目不串会话/事实/capability；普通聊天和只读查询零 Proposal、零 Grant、零 spawn、零业务副作用；生产调用图不再只有 trait 和 test-only 实现。

### M5R05 — 受控执行、恢复与 runtime conformance

产品结果：在隔离 scratch 项目中，一个经授权的白名单动作可以真实进入受控执行状态；停止、重试、恢复和未知结果与进程事实一致。

完成范围：

- 落地 durable operation、lease、timer、checkpoint、attempt、outbox、effect id、dead letter 和 restart recovery。
- 修正当前 stop/retry/resume 只返回文字、不改变状态的问题。
- 接 vendor-neutral AgentRuntimeAdapter、Workcell、Profile digest、SessionRef、TraceRef、ChildRunRef、Budget、stop conditions 和 RuntimeReceipt。
- 明确 Step/Turn、pre-execute/execute/post-execute Tool Pipeline、append-only runtime event、trace/context rebuild、child run 深度与 grant 不扩权；dynamic package 默认关闭。
- 运行状态、预算和外部 effect 继续由 Syn/M2 outbox 与 effect readback 持有，不交给 runtime 进程内 job/schedule。
- 至少一个非 `#[cfg(test)]`、进入普通产品 composition 的 Syn-native 默认 adapter，加一个状态语义独立的 fake/第二实现，共同运行同一 conformance suite；禁止复制一份 fake 冒充第二实现。隔离 App 使用 Syn-native adapter 的确定性配置，不要求真实模型、provider 或 DSH；本地证据只证明 vendor-neutral 合同，不冒充外部 runtime 成熟。

退出证据：kill/restart、resume/compaction、child grant 不扩权、duplicate effect、timeout、provider failure、receipt lost、degraded sandbox 和 `OUTCOME_UNKNOWN` 都有故障注入；产出供 M5R03 独立验证的原始 RuntimeReceipt、trace/effect refs；未知外部结果先按 effect id readback/reconcile，不盲重跑；stop/retry/resume 改变持久状态并与进程结果一致。

### M5R06 — ProjectSummary 正式投影

产品结果：Secretary/M6 可以通过受控端口读取项目必要状态，但看不到原文，也不能反写项目。

完成范围：

- 建立正式 projector/repository、version、watermark、source refs、敏感裁剪、失效和重建。
- ProjectSummaryQueryPort 每次按 consumer RoleSession、scope 和 policy 判权。
- consumer 不得直接读项目 store、兼容 projection、文件 root 或完整 snapshot。

退出证据：同一事实集确定性重建；落后 watermark 明确 stale；跨项目、无权限、过期 consumer 拒绝；查询/重建零项目反写；source refs 可回权威事实而摘要不复制原文。

### M5R07 — 项目 UI、隔离 App 与阶段候选

产品结果：用户能在隔离产品 App 中看见并控制完整项目闭环，而不是只验证后台对象。

完成范围：

- 项目默认入口接持久 Project Supervisor conversation。
- Proposal、授权、运行、恢复、报告、独立审查和结果决定只在用户明确动作后展开。
- 使用冻结 DTO 接现有项目壳，不在本包重写 execution kernel 或页面布局。
- 两个隔离 scratch 场景覆盖：只读、用户拒绝、一个白名单 fake write、worker blocked、执行失败/恢复、重复操作、强退重启、summary、只读 global advice、最终用户决定和 source deep link；global advice 只使用冻结的 fixture/port boundary，不要求提前实现 M6。
- 旧 fixed path/legacy execution 具有 manifest、compatibility 和 rollback 证据；不在 M5 物理删除。

退出证据：隔离 Tauri 交互、窗口截图、deep-link 点击、强退/重启和持久状态；用户拒绝时零 spawn/零业务 mutation；完整 `Proposal → AuthorizationDecision → Authorization → Run/WorkItem + worker RoleSession binding → PreparedAttempt → attempt-scoped Grant → Dispatch → AgentWorkcellRun → RuntimeReceipt/ExecutedReport → independent Review → ResultUserDecision` 精确闭环，manual/offline claim 始终保持非执行分型；Git、Harness、测试、产品路径 delta 和文档候选交总线独立复核。

### M5 顺序与退出

```text
REC-00 → M5R00（仅有 GAP 时）→ M5R01 → M5R02 → M5R03 → M5R04
                                      ├→ M5R05 ─┐
                                      └→ M5R06 ─┴→ M5R07 → M5 独立验收
```

M5 最终退出仍以原 M5 计划第 9 节为准。M5R00（如进入）与 M5R01–M5R07 全部完成前只能写“原型/局部链路完成”。M5R00（如进入）及 M5R01–M5R06 的完成事实成立后可原子归档到 done，但 leaf done 不等于 stage 验收。M5R07 必须先形成只含 M5 投影的 candidate commit series 与不可变 candidate tip，在 disposable checkout 生成绑定该 SHA 的原始 receipts 和候选报告；随后保持 `AWAITING_INDEPENDENT_ACCEPTANCE`，authorization 回 closed，不继续施工。总线只读复核 exact candidate SHA。总线确认后，原主管只执行 closeout-only：

- 确认 M5R00 已明确 `NOT_NEEDED` 或唯一存在于 done，且 M5R01–M5R06 已唯一存在于 done；只原子归档仍 current 的 M5R07，然后归档真实 stage-14。done 目标冲突、同 leaf 同时出现在 current/unfinished/done，或缺任一前序 leaf 都硬停；
- authorization 回到精确 closed；
- 由有授权的主管形成单独 closeout commit；candidate series/tip 与证据继续保持精确绑定；
- current-state、总计划、M5 计划、计划索引与 Harness plan 一致写为 M5 完成；
- 向 M6 交付 versioned ProjectSummary/QueryPort、TemporaryAgent 所需的完整 execution identity envelope，以及旧执行入口 compatibility/rollback manifest；这些都绑定 M5 candidate SHA；
- M6 仍保持未激活，另建独立 M6 开发主管任务。

closeout-only 形成单独 lifecycle commit；总线再核一次该 commit 的 stage/leaf/auth、文档与候选 SHA 绑定，不能先审 mutable working tree、后补内容 commit。

## 5. M6 修正包：stage-15

### M6 写前硬门

除合同裁决外，M6 生产施工必须等待 M5 独立验收以下输入：

1. 正式 ProjectSummary projector/QueryPort，具备 version、watermark、source refs、敏感裁剪、deep link、每次按 RoleSession/scope/policy 判权、重建和失效。
2. WorkItem、Attempt、Grant、worker RoleSession、RuntimeReceipt、ExecutedReport、actor、audit 的精确身份链。
3. M5 已提供并验收逐项目 DecisionRequest、command、Grant 与 authoritative application receipt 的通用能力；M6R04 再建立 Advisory 采纳与这些项目回执的引用关系。
4. M3 通用 RoleSession/Handoff 生产基础已经验收，M4 Secretary consult 扩展点与合同稳定；具体 Global Supervisor 绑定和正式接收者由 M6R02/M6R04 落地。
5. AppState、command registry、SQLite、导航等公共承重面已有唯一写者和 opening hash。

M5 未通过时，M6 candidate 只读保全；不得用现有 M6 单测反向证明前置成立。

### M6R01 — 合同修正与原型裁决

产品结果：Global Supervisor、Advisory、StableMember、TemporaryAgent 和多视角只有一套 owner 与精确 join。

完成范围：

- 冻结 GlobalSupervisorSession、CrossProjectAdvisory、ConsultHandoff、StableMember、TemporaryAgent、Availability、ContactReceipt、MultiView 的 owner/lifecycle/join。
- TemporaryAgent 必须绑定完整 M5 执行链；Grant 不得可选。
- Advisory 必须引用 Global RoleSession、Handoff、每份 ProjectSummary version/watermark、policy decision 和 generated_at。
- 用户采纳只产生 DecisionRequest；应用状态只投影各项目 authoritative receipt。
- 对当前 M6 原型逐项裁决 `KEEP / REWRITE / QUARANTINE`；`.bak` 先在 R0 保存，确认不含唯一内容且删除路径已进入当前 leaf 写域后才删除，否则只从 candidate 排除并原位保全。

退出证据：合同、迁移矩阵、fixture 与反例闭合；本包不声称服务可用。

### M6R02 — 持久 Global Supervisor RoleSession

产品结果：顶层全局主管是可恢复的稳定角色，不因模型/provider/thread/process 更换而换人或扩权。

完成范围：复用 M3 RoleSession 真源，固定 global scope、只读默认、current object、channel 和 permission；接正式 repository/application service；不新造平行 session/Handoff 状态机。

退出证据：重启恢复同一角色会话；底层 fake provider/runtime 更换不改变身份、职责或权限；本包尚不查询项目、不生成 advisory。

### M6R03 — 跨项目查询与 Advisory 真源

产品结果：全局主管只通过 M5 受控摘要发现项目间风险、依赖、冲突和优先级，并能精确回源。

完成范围：

- 只经已验收 ProjectSummaryQueryPort 读取两个以上项目。
- 实现 `ok/stale/missing/denied/degraded`、确定性冲突、版本化来源和 deep link。
- 建立 Advisory 持久真源与 application projection。
- 对项目 store、event/audit/outbox、文件和进程面设置 write-spy/hash baseline。

退出证据：任何跨项目读取都经 M5 owner policy；项目业务写面零变化；不得读取 root、原始材料、transcript、secret 或完整 snapshot。

### M6R04 — Secretary 显式咨询与用户决定边界

产品结果：Secretary 可显式把问题交给 Global Supervisor，接受、拒绝、完成和回执全程留痕；咨询不会自动改变项目。

完成范围：把 M4 普通产品 consult port 接到 M3 Handoff 与 M6；覆盖幂等、回源和失败；用户采纳只建立 DecisionRequest，各项目后续分别走 M5 Grant。

退出证据：Handoff/Advisory/policy/DecisionRequest/项目 receipt 精确 join；M6 的 applied/failed/rolled-back/unknown 只引用项目回执，不自行猜结果。

### M6R05 — 稳定成员目录

产品结果：用户可以找到和联系稳定内部成员，但目录不拥有授权。

完成范围：生产持久化、成员生命周期、role/scope assignments、capability/permission 只读 refs、availability 来源/TTL、会话历史、搜索与 ContactReceipt。

退出证据：陈旧 availability 变 unknown 且不参与授权；直接联系只创建 RoleSession/Handoff，不扩项目能力；不从 session 名称/数量猜成员；更换 fake provider/runtime 后成员身份不漂移。

### M6R06 — 临时 Agent 历史与人工晋升

产品结果：用户能查到临时 Agent 做过什么、结果和来源；临时身份永不因重复使用自动变成稳定成员。

完成范围：从 M5 已验收的完整 execution identity envelope 精确投影，至少覆盖 Project、Orchestration、WorkflowRun、WorkItem/Node、Dispatch、Attempt、Grant、worker RoleSession、RuntimeReceipt、ExecutedReport、trusted actor 和 audit；支持搜索、失败、回源；人工晋升新建/绑定 StableMember 并保留 `promoted_from`。

退出证据：任一执行 join 缺失即拒绝投影；不复制报告正文；runtime child 不因名称或 parent/child 自动成为组织成员；原临时历史类型和来源不被改写。

### M6R07 — 独立多视角会诊

产品结果：重大问题可以取得至少两个彼此独立的意见，并排显示共识、分歧和来源，仍由用户决定。

完成范围：至少两个独立 RoleSession/Workcell 接收同一最小来源包；提交前互不读取对方结论，封存后才汇总；普通问题仍走单角色；本阶段使用 fake roles/provider。

退出证据：输入隔离可机械证明；结果不生成项目命令、Grant、正式事实或记忆；超时/失败/部分完成诚实显示。

### M6R08 — 普通产品接线与桌面 UI

产品结果：用户有明确的顶层 Global Supervisor 入口，并能在成员工作面区分稳定成员、临时 Agent、会话和历史。

完成范围：

- 新增顶层 Global Supervisor，不复用项目页旧 review 冒充。
- `智能体/成员` 支持稳定/临时分型、搜索和联系。
- Secretary consult、Advisory 来源、stale/denied/degraded、多视角共识/分歧用人话展示并精确回源。
- 显示 role/scope/current object/channel/permission 的人话版本；内部 ID/schema 只进开发者模式。
- 不显示未实现按钮，不用前端假数据，不把组织图设为日常入口；仅桌面 Tauri。

### M6R09 — 双项目隔离 App、阶段候选与收口

使用两个 scratch projects 和 fake roles/provider，至少证明：

- 顶层 Global Supervisor 会话跨强退/重启恢复；
- 只经 M5 port 读取两个版本化摘要，发现冲突并点击 deep link 回源；
- stale、missing、ACL denied、degraded 诚实显示；
- Secretary Handoff 接受/拒绝/完成/幂等/回源；
- Advisory 和用户采纳本身不改变项目；模拟逐项目回执后 M6 只正确投影结果；
- stable member 可查找/联系且不扩权；temporary agent 精确回源且不冒充 stable；
- 多视角输入在汇总前保持独立；
- SQLite、普通产品 composition、兼容展示、强退重启和回滚通过；
- 保留 Tauri 桌面截图、交互和 deep-link 点击证据，浏览器 smoke 不替代桌面验收。

### M6 顺序与退出

```text
M5 独立验收 → M6R01 → M6R02 → M6R03 → M6R04
                         ├──────────→ M6R05
                         └─M5执行链→ M6R06
M6R02 + M6R03 + M6R05 + M6R06 → M6R07
M6R04 + M6R05 + M6R06 + M6R07 → M6R08 → M6R09 → M6 独立验收
```

M6 最终退出仍以原 M6 计划第 9 节为准。M6R01–M6R08 完成后可逐叶归档；M6R09 先形成只含 M6 投影的 candidate commit series 与不可变 candidate tip，在 disposable checkout 生成绑定该 SHA 的原始 receipts 和候选报告，然后保持 `AWAITING_INDEPENDENT_ACCEPTANCE`、authorization closed。总线只读复核 exact candidate SHA；PASS 后原主管只执行 closeout-only：归档 M6R09/stage-15、同步事实文档、形成 M7 输入交接和单独 lifecycle commit。总线再核该 lifecycle commit。M7 保持未激活。

## 6. 验证与证据分层

| 层级 | 必须证明 | 不足以声称 |
|---|---|---|
| Contract/static | owner、identity、join、scope、状态、禁止旁路 | production service 已接入 |
| Unit/property | 跨项目拒绝、revocation、幂等、状态反例 | 普通产品可用 |
| Temp integration/fake runtime | repository/UoW、重启、故障注入、summary/advisory 重建 | 真实项目/provider 可用 |
| Non-test build/call graph | production path 编译并有普通 caller | UI/强退恢复正确 |
| Isolated Tauri | 普通 composition、桌面交互、SIGKILL/重启、deep link、fake failure | 真实资料、模型或日常使用 |
| Fresh stage regression | 候选 commit 的定向、全量 Rust、typecheck、离线交互、production build 和格式检查 | push、部署、发布 |
| Independent bus review | Git、Harness、代码、测试、证据、边界、文档与下游一致 | 自动激活下一阶段 |

每个 leaf 保存原始命令、环境、退出码、测试选择、结果、opening/final hash 和 commit 绑定。单元测试数量、working-copy 运行、浏览器 smoke、fake provider 或开发主管自评都不能替代更高层证据。

整阶段全量回归只在 M5R07、M6R09 各执行一次；各中间包只跑直接对应的定向矩阵，避免重复审核和复测膨胀。独立复核优先复用可携带证据，只对高风险缺口做新鲜抽查。

## 7. Git、Harness、文档与交接

- 每个 leaf 唯一写域、opening hash 和独立内容提交；控制归档与 stage close 单独留下可追溯载体。
- M5/M6 staged projection 必须排除另一阶段及 L1–L3/LX；共享文件按 hunk 审核，cached diff 与白名单精确一致。
- 候选 commit 在 disposable checkout 复测；权威工作树中的另一阶段 WIP不得参与证明。
- 禁止倒填 Hook 事件、authorization、stage/leaf 时间线或测试回执；历史错误只写勘误。
- M5 关闭时同步 current-state、总计划、M5 计划、计划索引、Harness plan 和 M5→M6 handoff；M6 关闭时同步 M6 计划、架构/前端当前事实、M7 输入 handoff。
- `stage-12` 与 D0C04/D0C05 在全过程中只读保全；任何变化都单独归责，不能夹带进 M5/M6。
- 阶段候选应做到 M5/M6 写面零未知 delta；若仍有其他 WIP，必须与 R0 provenance 精确一致。`git diff --check` 必须通过，无 `.bak`、临时 target、截图缓存或未知大文件进入候选。
- push、merge、rebase、部署和发布始终不从阶段完成自动推导。

## 8. 开发主管自主范围与硬停止

### 8.1 在一次阶段授权内自主处理

- Rust/TypeScript 编译问题、fixture、Linux path/mode、测试隔离和普通迁移 bug；
- 本计划点名合同内的 schema、repository、UoW、adapter、DTO、command 和 UI 接线；
- 现有原型的 KEEP/REWRITE/QUARANTINE 裁决；
- 直接相关的格式、warning、测试、证据和文档同步；
- 同一明确故障在有新证据时继续修复，不因普通工程问题反复询问用户。

### 8.2 必须停止并回到总线/用户

- R0 前后出现并发变化，或任一共享字节无法归入 L1–L5/LX；
- 需要 reset、stash、clean、覆盖或丢弃既有 WIP 才能继续；
- 发现 secret、credential、真实运行数据、未知 symlink/special file 或受保护大文件；
- 要求伪造 Hook receipt、authorization、stage/leaf、测试或 App 证据；
- stage-12、D0C04/D0C05 或 M1–M4 冻结合同将被意外修改、提升或关闭；
- M5 硬退出门未成立却要启动 M6，或 M6 未验收却要激活 M7；
- 需要真实资料/项目写、真实模型/provider/message/connector、外部网络业务写、新系统权限、push、merge、release、deploy；
- 实现与产品正本、角色身份边界或前端显示边界发生真实冲突；
- 候选 commit 与新鲜证据 SHA 不一致，或独立验收仍存在无法解释的产品代码 delta。

## 9. 派发与停止点

本计划写完后的正确停止点是：计划文件存在，但 Harness、Git 和产品代码均不因计划自动变化。

后续派发顺序：

1. 建立独立“Syn M5 事实重整与产品闭环开发主管”任务；用户在该任务明确授权 stage-14 整阶段目标与边界。
2. 主管先完成 REC-00；前置矩阵有 GAP 时先完成 M5R00，否则明确 `NOT_NEEDED`，再顺序执行 M5R01–M5R07；提交待验收候选后停止。
3. 总线并行做 Git/Harness、代码/测试、文档/下游三路独立只读复核；证据支持时才验收 M5。
4. M5 验收后另建“Syn M6 全局主管与内部组织开发主管”任务；不得复用 M5 task 上下文或授权。
5. M6R01–M6R09 完成后再次独立验收；只给出 M7 首个窄包建议，不自动激活。

本计划不要求用户再次拍板 M5/M6 已确定的核心产品需求。未来施工任务只需用户确认“按本计划开始 M5”或“在 M5 验收后按本计划开始 M6”；其余实现细节由对应开发主管自主收敛。
