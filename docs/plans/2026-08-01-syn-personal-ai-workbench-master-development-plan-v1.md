# Syn 全能个人 AI 工作台总开发计划 v1

日期：2026-08-01<br>
状态：**当前总开发计划；M0–M3 已完成各自具名主线范围。M4 为 M4R07 v2 product-chain PASS、`stage-07` closed；M5 为 scoped product-chain PASS、`stage-14` closed。M6–M11 未激活。**<br>
计划性质：定义长期重构和迁移顺序，不维护逐任务进度，不单独授予代码、桌面应用、存储、真实消息、外部连接、凭据、Git（版本控制写入）或发布权限。当前事实看 `../current-state.md`、源码和新鲜验证；具体施工入口看当前用户指令、`AGENTS.md` 与轻量开发护栏的活动阶段、唯一活动叶和 `../harness/authorization.json`。没有活动阶段时，不从本计划推导自动下一包。

## 0. 目标

把当前“能力很多、日常工作线分散、历史路线叠加”的工作台，重构为用户可以长期依赖的 Syn：

- 秘书统筹用户每天需要知道、需要决定和不能遗漏的事情；
- 全局主管独立处理跨项目优先级、边界、风险和复核；
- 每个项目内有常驻项目主管，用长期对话组织项目事实和执行；
- 个人范围与项目范围并存，不强迫所有生活和信息进入项目；
- 日常与开发是明确的执行通道，共用角色、权限、事件、审计和记忆底座；
- 知识、记忆、任务、方案、审批、工作流、Agent、Harness、通知、日报、外部软件和凭据作为受控能力协同，不成为互不相干的孤岛；
- 用户逐步只保留目标、资源、风险和例外等最高层决定；重复、低风险判断在用户签署的政策内下沉给 Syn；
- Syn 自己持有治理核心与默认 Agent Runtime 合同，并能通过候选、隔离验证、canary、签名晋升和回滚持续升级自己的能力；
- 现有后端能力按 `KEEP / EXTRACT / REWRITE / RETIRE / HOLD` 处置，通过绞杀式迁移逐步换掉旧路，不做一次性推倒重来。

## 1. 权威与证据口径

本计划受以下正本约束：

1. 当前用户指令；
2. `../product/syn-product-canon-v1.md` 与 `../product/authority-register-v1.md`；
3. `../product/knowledge-infrastructure-canon-v1.md`；
4. `../../decisions/2026-08-16-syn-native-governance-core-and-governed-self-upgrade-direction-v1.md`；
5. `../../decisions/2026-08-01-whole-workbench-event-driven-operating-model-amendment-v1.md`；
6. `../../decisions/2026-08-01-memory-self-capture-daily-consolidation-and-skill-governance-amendment-v1.md`；
7. 继续有效的角色、项目内自然对话、两轴治理、共享传输和执行人闸决定；
8. `../workbench-system-architecture-v1.md` 的目标模块边界；
9. `../current-state.md`、当前源码与新鲜验证所证明的实现事实。

`../../AGENTS.md` 与轻量开发护栏只管理每次施工怎样进行和是否有权限，不反过来定义产品。计划、阶段文件和开发护栏存在，都不等于当前任务已经激活。

证据分层：

- 文档合同通过，只证明定义一致；
- 静态 / typecheck / 单测，只证明对应代码性质；
- fake adapter / temp store / synthetic UI，只证明隔离链；
- 真实 Tauri App，只证明本机、该 profile、该场景；
- 真实 Codex、真实项目写入、真实连接器或真实凭据，必须逐项另行授权和验收；
- 任何一层都不自动升级成发布或生产结论。

## 2. 目标运行工作线

### 2.1 主循环

```text
事件在自己的作用域出现
  → 该作用域的 owner 处理
  → 秘书持续看住与用户有关的未闭环事项
  → 必要时查询内部角色 / 请求全局主管只读分析
  → 通过简报、克制打断或日终报告反馈给用户
  → 正式业务对象只按用户明确语义、已批准规则或既有授权创建 / 改变
  → 结果回到原真源
  → 记忆另按记忆治理策略进入日报、正式记忆、个人模型或 SkillCandidate
```

### 2.2 用户每天看到什么

- 打开 Syn：秘书给出有来源的当前情境，外部承诺和时间敏感事项优先。
- 日间：没有事件就不空转；有事件才更新 attention、摘要、待决定或异常。
- 明确项目工作：直接进入项目主管，不经过秘书猜项目。
- 跨项目判断：直接进入全局主管，或由秘书发起有记录的只读咨询。
- 需要专业对象：知识、记忆、Agent、方案、工作流、审计仍可直接打开。
- 日终：定时整理完成、未闭环、风险、纠正和明日延续，并生成可回源的日报。

### 2.3 不会发生什么

- 普通聊天不自动创建项目、任务、工作流、日程、想法箱或批准。
- 秘书不越过项目主管改项目，不替用户决定，不因关注而私自派活。
- 全局主管意见不自动改变项目。
- 项目摘要、首页、日报和读模型不复制成第二份事实。
- 外部软件读取授权不自动包含写入授权。
- 密钥和敏感原文不进入聊天、记忆、事件 payload 或普通数据库。
- 没有新事件时 Agent / 模型不持续运行。

## 3. 目标架构

仍采用本地模块化单体，不拆微服务，不做全量事件溯源，不做通用节点自动化平台。

```text
Tauri / React UI
  → Application Use Cases
    → Identity & Scope Kernel
    → Policy Kernel
    → Knowledge & Context Foundation
    → Domain Aggregates
       ├─ Conversation / Handoff
       ├─ Attention / Decision / Daily
       ├─ Project Orchestration / Execution
       ├─ Knowledge / Memory / Personal Model
       ├─ Connector / Credential Reference
       └─ Governed Upgrade / Promotion
    → Transaction + Event + Audit + Outbox
      → SQLite authoritative state
      → Rebuildable read models
    → Adapters
       ├─ Agent Runtime Workcell / Model
       ├─ Tool / MCP / Harness
       ├─ External Knowledge / File
       └─ External Connector
```

### 3.1 Kernel 合同

`identity_scope` 拥有：

- `ProjectId` / `ProjectRoot`；
- `ScopeRef(personal | global | project)`；
- `RoleRef(secretary | global_supervisor | project_supervisor | stable_member | temporary_agent)`；
- `CurrentObjectRef`；
- `ExecutionChannel(daily | development)`；
- `PermissionProfile`；
- 项目归属、默认范围隔离、用户明确跨项目请求和稳定对象引用规则。

`policy` 统一判定：

- command 是否允许；
- 用户确认、风险升级和权限边界；
- 状态迁移；
- 候选能否成为事实 / 正式记忆 / 正式 skill；
- adapter / connector capability 是否可调用。

所有 Tauri command、MCP、runner、connector 和后台 job 都必须经过同一 policy，不允许前端或调用方自报授权后自洽通过。

### 3.2 统一机制，不统一业务真源

`WorkbenchEventEnvelope` 至少包含：

```text
event_id / schema_version / event_type / occurred_at
actor_id / actor_role
scope_type / scope_id / project_id?
object_type / object_id
execution_channel
command_id / correlation_id / causation_id / idempotency_key
source_adapter / sensitivity
payload_summary / payload_ref / payload_hash
```

- 事件账本只保存结构化事实、最小摘要、hash 和引用；不保存 raw transcript、完整 prompt、完整 tool output 或 secret。
- 项目、个人、连接器各自拥有业务事实；公共 event / audit / outbox 是机制，不是全局业务数据池。
- 每条 command 在同一事务中写领域状态、event、audit 和 outbox；外部动作在提交后执行，再以结果 command 回写。

### 3.3 核心对象合同

| 对象 | 作用 | 真源 / 边界 |
|---|---|---|
| `RoleSession` | 固定角色、作用域、当前对象、通道和持久会话 | Syn 持有 binding；provider thread id 只是外部 handle |
| `Handoff` | 显式跨角色 / 跨通道交接 | 保留 from/to、scope、目标、refs、权限请求与回执 |
| `KnowledgeContextPacket` | 按角色、范围、任务和权限装配资料、来源与技能说明 | 记录来源、版本、纳入和排除理由；不是事实、记忆或执行授权 |
| `OpenLoop` | 秘书内部关注和闭环跟踪 | 协调状态，不是 Task；带来源、owner、理由、dismiss/close |
| `DecisionRequest` | 需要用户作有约束力决定 | 回到原 owner / object；简单可聚合回答，复杂回源 |
| `ProjectSummary` | 给秘书 / 全局主管的最小项目摘要 | 从项目真源投影，可重建，不可反向写项目 |
| `DailyBrief/Report` | 开场态势与日终总结 | 每项带 source ref；不是正式事实或记忆 |
| `Proposal/Authorization` | 执行前的目标、范围与批准 | 用户批准后才产生 ExecutionGrant |
| `WorkflowRun/WorkItem` | 项目内持续或多步骤执行 | 项目主管拥有；adapter 无权推进状态 |
| `Observation/MemoryCandidate/FormalMemory` | 受治理长期记忆 | 来源、scope、敏感性、冲突、版本、外发策略 |
| `PersonalFact/ModelAssertion` | 个人明确事实与系统推断 | 两者严格分型；可纠正、可追源、secret 排除 |
| `ConnectorDefinition/Grant` | 外部软件能力和授权 | view/index/sync/action/secret 分开授权 |
| `CredentialRef` | 受保护凭据引用 | 不保存 secret 本文；日志、事件、记忆只存引用 / hash |
| `SkillCandidate/SkillVersion` | 可复用方法和正式技能 | 版本、验证、权限和回滚；不自动扩权 |
| `AgentRuntimeProfile/WorkcellRun/RuntimeReceipt` | 可替换 AI 执行工作单元 | runtime session / trace 只作执行引用；Syn 持有 operation、grant、事实与验收 |
| `UpgradeCandidate/EvaluationReceipt/PromotionDecision` | Skill、Profile、Plugin、Provider 或代码升级 | 生成、验收、晋升职责分离；治理根不可自批 |

## 4. 现有能力处置

这里处置的是能力，不是按文件粗暴删除。

### 4.1 KEEP：保留规则和已经正确的能力

- Tauri + Rust + React + Vite 桌面壳（2026-08-17 起为存续期载体；长期壳载体为 lightcode fork，见 `2026-08-17-syn-lightcode-fork-shell-adoption-plan-v1.md`）；
- 项目作为复杂工作的主要身份、权限和执行边界；
- proposal → 用户确认 → 边界复核 → authorization → execution 的治理语义；
- worker report、review、用户验收和正式事实分层；
- Supervisor MCP 的 trusted binding 与精确 capability allowlist；
- Codex runner 的 cwd、sandbox、write roots、prompt hash、readback 和 process registry 防线；
- Knowledge Vault 的固定根、路径 / CAS / 大小限制、文件真源、恢复和原生知识工作面；
- Memory 的 observation / candidate / formal、来源、版本、scope、冲突、敏感性和审计结构；
- SQLite / JSON reconcile 和兼容投影作为迁移设施；
- 07-14 项目内五态交互 canon 的有效范围；
- 两轴编排 / 风险治理和确定性机器拦截规则。

### 4.2 EXTRACT：提取为稳定端口

- 从 command registry / `commands.rs` 提取 scope resolver 与 authorization gateway；
- 从 conversation transport 提取 `ConversationTransportPort`；
- 从 Codex runner 提取 vendor-neutral `AgentRuntimeAdapter`、`AgentAdapter` 与 `RuntimeReceipt`；
- 从 Supervisor MCP 提取 `CapabilityGateway` / `ToolAdapter`；
- 从 sidecar / workflow 写入提取 Unit of Work、EventWriter、AuditWriter、Outbox；
- 从原生知识工作区提取外部资料来源端口与知识来源适配器；来源登记、检索路由和上下文装配留在核心知识层；
- 从 Harness 索引、readiness、verifier 提取 `HarnessAdapter`；
- 从现有 memory stores 提取 repository 和治理服务；
- 从前端项目聊天、Agent 会话和 transport UI 提取共用的角色会话显示层。

### 4.3 REWRITE / NEW：重写或新增

- 项目 / 个人 / 全局作用域与稳定对象身份；
- `KnowledgeSourceRegistry`（知识来源登记）、`KnowledgeContextAssembler`（知识上下文装配）和技能发现桥梁；
- 持久 `RoleSession`、`Turn`、`CurrentObjectRef`、`PermissionProfile`、`ExecutionChannel` 和 `Handoff`；
- 统一 command / event / audit / snapshot / outbox 事务底座；
- 通用项目主管编排和 execution aggregate；
- Secretary、Global Supervisor 的应用服务与持续会话；
- `OpenLoop`、Inbox、Todo、Notification、Reminder、DecisionRequest、DailyBrief / Report；
- 个人事实、个人模型断言和来源治理；
- 记忆自动治理策略、每日整理和 SkillCandidate / SkillVersion；
- Connector registry、connection、grant、sync cursor、inbound item、action request / result、credential ref；
- 各域读模型，停止页面查询反复构建完整 snapshot；
- 首页秘书情境 + 对话、全局主管入口和成员目录。
- 可销毁 Agent Workcell、append-only runtime trace、工具前 / 中 / 后管线和 runtime conformance suite；
- ImprovementSignal、UpgradeCandidate、能力 / 权限差异、独立评测、canary、签名晋升与回滚控制面。

### 4.4 RETIRE：等替代链验收后退役

- `run_workflow_machine` 正常产品入口；
- 固定 Mario classic workflow chain 对通用主流程的控制；
- resident supervisor one-shot / pilot legacy action loop；
- synthetic Phase A 自动化的产品入口；
- Legacy Canvas MCP 执行工具（Canvas 数据可迁移保留）；
- offline role 人工粘贴链；
- manual relay 正常业务入口（可降级为受限诊断 / 恢复工具）；
- 等价验收后的旧 `knowledge_vault_*` 单层 note API；
- workflow id contains slug 的项目归属兼容；
- 只在前端内存维护的项目 conversation cache；
- 页面全量 snapshot 拼装和多份启发式审计聚合；
- 旧计划、旧按钮和隐藏 action 对当前施工的授权语义。

退役必须走 `shadow → new primary → compatibility read-only → command unregister → archive/export`，一次只退一类，先验收替代链，再由用户单独批准不可逆清理。

### 4.5 HOLD / UNKNOWN：不能在计划里假定

- dirty worktree 上真实 DB-primary / JSON 降级和启动 reconcile 状态；
- 普通项目对话、知识、runner、stop/retry/resume 的当前真实 App 表现；
- 外部连接器首批具体产品和真实数据保存策略；
- 凭据库采用 Keychain、加密文件或其他实现；
- 自动记忆策略矩阵和 skill 启用阈值；
- 多 provider、OpenClaw / Claude Code / OpenCode 的真实接入；
- DeepSeek Harness 或其他外部 runtime 的真实接入价值、维护成本和 conformance 结果；
- 首个自升级样本、低风险自动晋升政策与 updater / 签名实现；
- 生产、发布、真实资金、个人账号动作和无人值守执行。

## 5. 阶段总览与依赖

```text
M0 文档与正本冻结（本轮）
  ↓
M1 合同 + 安全/作用域止血
  ↓
M2 事实、事件、审计、事务底座
  ↓
M3 角色会话 + 显式交接
  ├─────────────┐
  ↓             ↓
M4 秘书/Attention/日报   M5 项目主管/执行闭环
  └──────┬──────┘
         ↓
M6 全局主管 + 内部成员目录
         ↓
M7 知识 + 记忆 + 个人模型 + 技能治理
         ↓
M8 Connector + CredentialRef
         ↓
M9 读模型切换 + 旧路退役
         ↓
M10 全日使用试点 + 发布硬化
         ↓
M11 受治理自升级平台
```

M4 与 M5 只能在写域不重叠、公共合同冻结后并行。M7 的知识 / 记忆合同和隔离存储可在 M2 后先做；角色知识装配要与 M3 合同对齐，真实日常事件接入再等待 M4 / M5。M8 的连接器框架可早做，真实第三方接入必须等凭据和外部动作合同独立通过。

本计划的完整范围是“Syn 公共底座重构 → 首个全日使用试点 → 发布候选 → 受治理自升级底座”，不是 Syn 一生所有业务能力的逐功能排期。M11 以后，游戏开发、智能体开发、企业系统、个人工作和股票市场分别按真实使用需要建立业务路线；个人服务器异地备份、开源成本工具和高级知识检索也在进入真实需求时单独建包。它们没有提前排成固定阶段，不影响 M3–M11 的主线完整性。

### 5.1 独立阶段计划索引

下列文件把 master 的顺序展开为可单独审查的阶段合同。M1–M3 已完成主线收口；M3C08 内容提交为 `fa8e392`。M4C01–M4C10 已进入主线，M4R07 v2 后端/普通产品链为 12/12 PASS，`stage-07` 已关闭。M5 的产品内容锚 `c91d8fc` 已在具名范围通过独立验收，M5C01 closeout 内容 `de98d69` 已完成交接与生命周期收敛，`stage-14` 已关闭；M6–M11 仍为 `PLANNED / NOT_ACTIVE`。当前 Harness 现场另看 `../harness/plan.md`，不在 master 复制动态 leaf 状态。

| 阶段 | 独立计划 | 当前路由状态 |
|---|---|---|
| M1 | `2026-08-01-syn-stage-1-contracts-and-security-foundation-plan-v1.md` | `COMPLETED / MAINLINE` |
| M2 | `2026-08-01-syn-stage-2-fact-event-audit-transaction-foundation-plan-v1.md` | `COMPLETED / MAINLINE / BOUNDED_REFERENCE_SLICE` |
| M3 | `2026-08-01-syn-stage-3-role-session-and-explicit-handoff-plan-v1.md` | `COMPLETED / MAINLINE / STAGE-05 CLOSED`；内容提交 `fa8e392` |
| M4 | `2026-08-01-syn-stage-4-secretary-attention-and-daily-rhythm-plan-v1.md` + `2026-08-11-syn-m4-independent-remediation-and-reacceptance-plan-v1.md` | `M4R07 V2 PRODUCT-CHAIN PASS / STAGE-07 CLOSED`；UI/CU 未执行，不含视觉或发布验收 |
| M5 | `2026-08-01-syn-stage-5-project-supervisor-and-execution-loop-plan-v1.md` | `SCOPED PRODUCT-CHAIN PASS / STAGE-14 CLOSED / NOT RELEASED` |
| M6 | `2026-08-01-syn-stage-6-global-supervisor-and-internal-organization-plan-v1.md` | `PLANNED / NOT_ACTIVE` |
| M7 | `2026-08-01-syn-stage-7-memory-personal-model-and-skill-governance-plan-v1.md` | `PLANNED / NOT_ACTIVE` |
| M8 | `2026-08-01-syn-stage-8-connector-and-credential-reference-plan-v1.md` | `PLANNED / NOT_ACTIVE` |
| M9 | `2026-08-01-syn-stage-9-read-model-migration-and-legacy-retirement-plan-v1.md` | `PLANNED / NOT_ACTIVE` |
| M10 | `2026-08-01-syn-stage-10-full-day-pilot-and-release-hardening-plan-v1.md` | `PLANNED / NOT_ACTIVE` |
| M11 | `2026-08-16-syn-stage-11-governed-self-upgrade-platform-plan-v1.md` | `PLANNED / NOT_ACTIVE` |

独立计划维护本阶段的现状事实、HOLD、owner、薄切片、写域、授权、验证和退出门；master 继续只维护长期方向、依赖和总完成定义，不复制动态任务状态。

## 6. 分阶段开发合同

### M0 — 文档、决定与当前路线冻结

交付：

- 两份 2026-08-01 正式修订和 2026-08-09 产品正本确认决定；
- 架构、前端边界、记忆、长期工作线、backlog、历史计划和入口对齐；
- 本总计划、产品正本、知识基础设施正本、权威登记表、唯一候选登记、当前状态和阶段计划入口；
- 非权威材料的合并、降级、目录状态与历史回链；
- 能力 disposition、依赖和验收矩阵。

退出条件：当前入口不再指向不存在文件或自称 current 的旧路线；散落候选统一进入唯一登记，非权威材料不再因文件位置自行升格；目标模型与现状库存明确分开；产品代码仍未被文档计划自动激活。该文档收口已在 2026-08-09 完成，不因此激活 M3。

### M1 — 合同与安全 / 作用域基础

当前阶段详见 `2026-08-01-syn-stage-1-contracts-and-security-foundation-plan-v1.md`。

必须完成：

1. 冻结 Scope / Role / ObjectRef / Channel / Command / Event / Audit / OpenLoop / Handoff / Decision / Connector capability 合同；
2. 建立 `identity_scope` 与 `policy` 的纯领域核心和表驱动测试；
3. Agent Conversation 后端读取明确 role / scope / station，Station 3b 后端零写 enforcement；
4. Canvas 路径 ID 拒绝 `..`、绝对路径、编码变体和 symlink 越界；
5. `get_project_workflow_nodes` 强制项目归属；移除 slug contains 归属；
6. worker report 全链验证 project / workflow / dispatch；
7. execution 从持久 Authorization 解析 grant；
8. 顶层秘书不以固定项目目录取得原始项目读取面；
9. audit / error / receipt 统一 secret scrubber。

退出门：所有拒绝都发生在 spawn、文件写、业务状态 / projection / outbox mutation 和外部调用之前；仅允许独立、append-only、已脱敏的 denial audit。跨项目、伪造 ID、Station 3b、路径越界、伪造授权表驱动测试全部 fail closed；隔离 profile 的真实 App 允许 / 拒绝场景通过。

### M2 — 事实、事件、审计与事务底座

交付：

- typed event ledger、audit ledger、current snapshot、outbox；
- command transaction / Unit of Work；
- 旧 store repository adapters；
- deterministic projector、shadow write、parity report、reconcile / recovery；
- event payload secret / transcript / tool-output hard limits。

退出门：同一 command 的领域状态、event、audit、outbox 原子提交；崩溃恢复不重复外部动作；旧 / 新对象 count、key、hash 可对账；投影失败有明确 receipt 和可恢复状态。

真实 App：冷启动、写一笔、强制退出、重启、投影恢复、重复 command 幂等、DB / JSON 故障提示。

### M3 — 角色会话与显式交接

交付：

- 持久 `RoleSession` / `Turn` / provider handle / ConversationContext；
- 秘书、全局主管、项目主管和稳定成员共用角色会话应用合同；
- transport 退化为 start / poll / stop / resume adapter；
- App 重启后恢复正确角色、项目、对象、通道和会话；
- 显式 Handoff、接单、回执和结果回源；
- 项目聊天与 Agent Center 退掉两套前端 cache。

角色会话从第一版就携带可追溯的知识上下文引用：会话开始、恢复、交接和任务派发时，按角色、范围、当前对象和权限装配最小资料包；缺资料时可以说明理由请求更多。此处只冻结和打通上下文合同，不把完整知识检索、同步或界面一次塞进 M3。

前端：先增加明确角色入口和固定上下文标签，不先改首页视觉；智能体中心成为成员与历史会话目录。

退出门：每种角色新建 / 续接 / stop；重启恢复；跨项目续接拒绝；会话不静默生成 workflow / formal memory；Station 3b 后端真拒绝。真实 Codex 消息另行授权。2026-08-10，M3C08 已由 `fa8e392` 收口内容并达到 `COMPLETED / MAINLINE / STAGE-05 CLOSED`。M4 的 stage-06 随后程序性关闭；其普通产品修正已进入 M4R07 v2 产品链 PASS，M4R01–M4R07 已归档且 stage-07 已关闭。M5 后续已于 2026-08-18 以 scoped product-chain PASS 关闭 stage-14。

### M4 — 秘书、个人范围、Attention 与日常节奏

交付：

- PersonalScope、InboxItem、OpenLoop、Todo、Notification、Reminder、DecisionRequest；
- source-first 投影、dedupe、read / dismiss / snooze / close / reopen / carry-over；
- Secretary 应用服务：主动查询内部状态、请求只读咨询、维护关注，不改 owner 事实；
- 首页改为“情境简报 + 秘书持续对话”；
- DailyBrief / DailyReport，LLM 不可用时仍能确定性生成；
- 没有事件不调用 Agent / 模型的机械证明。

排序底线：已与别人约定和时间敏感事项优先；所有条目必须显示来源、owner、为何出现、最后变化并能回源。

退出门：App 重启不丢关注状态；“已知晓”不改项目 / 任务 / 记忆；同窗口日报幂等；每个日报项可精确回源；无事件窗口零模型调用；普通产品具备持续 Secretary 对话和内部来源入口。2026-08-11 独立总线复核指出的普通来源、到期唤醒、精确回源、持续对话和实际 legacy parity 缺口已由 M4R01–M4R06 修正；2026-08-13 M4R07 v2 receipt 以固定 12 次、实际 12 次验证当前后端/普通产品链。第 8 次 `recovery_timer` 保留真实 98 秒等待与后端恢复验证；UI / Computer Use / PNG / attestation 为 `NOT_EXECUTED / NOT_APPLICABLE`，因此本结论不是视觉 PASS。M4R01–M4R07 已归档，`stage-07` 已关闭；M5 后续已于 2026-08-18 关闭 stage-14，M6 仍未激活。

### M5 — 项目主管与既有执行能力重组

交付：

- 普通项目的常驻项目主管读取本项目事实、会话、知识、记忆、黑板和 Harness；
- 对话明确提出结构化动作，用户确认后才创建 Proposal / Task / Workflow / Authorization；
- Proposal、Authorization、C4、Dispatch、ExecutionAttempt、WorkerReport、C5 / C6、GlobalReview、UserDecision 用同一 correlation / run identity；
- 项目主管按协调复杂度选择单 Agent 或多 Agent，风险治理独立判断；
- runner 只接受控制核心生成的 ExecutionGrant；
- vendor-neutral `AgentRuntimeAdapter` 接受 Syn 的 Workcell 合同；DeepSeek Harness 只可能成为一个适配器，不是默认治理核心；
- runtime Step / Turn、Tool Pipeline 和模型可见上下文形成可重建 trace，但不成为项目事实账本；
- Syn 持有 durable operation、lease、timer、retry、dead letter、effect ledger；不依赖 runtime 进程内 job / schedule；
- 外部副作用 checkpoint 后结果缺失时记录 `OUTCOME_UNKNOWN`，按 effect id 回读而不是盲重试；
- stop / retry / resume 名称与真实能力一致；
- 项目摘要投影给秘书 / 全局主管，不复制项目事实。

前端：保留现有项目壳和专业 tab，不重新发明已撤回的项目页布局；默认项目主管对话，方案 / 审批 / 运行按明确动作打开。

退出门：隔离 scratch project 覆盖 read-only、单 allowlisted write、用户拒绝、worker blocked、runtime kill / restart / recovery、child grant 不扩权、duplicate effect、trace readback、全局意见和最终用户决定；至少两种 runtime 实现或 fake conformance adapter 证明合同不绑定单一 Harness。固定测试路径不再定义通用业务语义。

2026-08-18 closeout：M5 产品内容锚 `c91d8fc` 已在修订后的普通产品组合范围获得独立 PASS；M5C01 内容 `de98d69` 完成 M6 输入、欠账路由与 WIP/载体分层，`stage-14` 已关闭。结论为 `SCOPED PRODUCT-CHAIN PASS / NOT RELEASED`，不含真实资料/provider、macOS/BSD 实机、真窗口像素、新壳运行、部署或发布；M6/stage-15 仍未激活。

### M6 — 全局主管与内部组织

交付：

- 顶层全局主管入口和持久会话；
- 跨项目摘要、风险、依赖、优先级建议和来源引用；
- Secretary → Global Supervisor 的留痕只读咨询；
- 稳定成员目录、角色档案、能力 / 权限、当前可用性、会话和直接联系；
- 临时 agent 历史搜索、任务、结果和审计；
- 重大问题可按需组织不同模型或不同方法的独立多视角咨询，并排呈现共识、分歧和来源；
- 内部组织默认后台，不要求组织图成为日常入口。

稳定成员身份由 Syn 持有，不等于模型、服务提供方、线程或进程；替换底层能力不会换人、重置记忆或扩大权限。

Runtime Profile 不是角色档案，child / subagent 不是稳定成员，runtime parent / child 关系也不自动成为组织关系。多视角咨询必须证明输入隔离，执行 workcell 的 final answer 只进入报告候选。

退出门：全局主管能从两个项目摘要发现冲突并回源；需要时能取得彼此独立的多视角意见；意见未经用户确认不改任一项目；用户能找到并直接联系指定稳定成员；替换底层服务后成员身份和权限不漂移；临时 agent 不伪装成常驻成员。

### M7 — 知识、记忆、个人模型、每日整理与技能

交付：

- 统一知识来源登记、索引、检索路由和上下文装配；常驻角色有稳定默认范围，临时智能体拿任务最小资料包；
- 资料、决定、代码索引、历史成果和技能说明带来源、版本、新鲜度、作用域和权限过滤；
- 技能可通过知识层被发现和理解，实际启用仍经过技能注册、角色权限和任务授权；
- memory capture / observation / candidate / formal 写入改为事务或可靠 saga；
- 自动策略矩阵：可自动、需确认、禁止；
- 用户明确事实与系统推断分表型 / 分状态；
- PersonalFact / ModelAssertion 的来源、置信度、时效、纠正和历史；
- 建立可纠正的用户知识深度：用户自述进入个人事实，系统判断进入带来源、置信度和时效的模型推断；
- 每日聊天、项目结果、纠正和重复模式整理；
- SkillCandidate / SkillDraft / SkillVersion、验证、启用、权限和回滚；
- 可执行 Skill / Profile / Plugin package 的来源、digest、依赖、capability manifest、sandbox 要求、兼容范围、评测集、签名、canary 和撤回；
- task memory packet 带 revision / fingerprint / stale 判定；
- 用户查看、纠正、冻结、废弃、关闭自动学习和批量撤销入口。

Runtime session、trace、compaction summary 和 transcript 均不是长期记忆；它们只能作为带来源的 Observation 输入。动态 package 只能生成候选，不能从运行内存直接晋级。

退出门：任一稳定角色和临时智能体都能按范围取得带来源的最小资料包，并能看见资料不足、过期、冲突和技能不可用；常驻角色重启后能区分并取回“用户是谁、近期做了什么、长期做过什么、当前有哪些未闭环事项”，且每项带来源、时间和作用域；用户知识深度可纠正，事实与推断不混型；检索命中不自动成为事实，技能发现不自动扩权；可执行能力包经过静态 / 权限差异、隔离回放、独立评测、canary、签名晋升和撤回；故障注入无无解释半状态；敏感信息、权限和外部动作不得自动入记忆或扩权；冲突不静默覆盖；自动写入每条都有策略结果和审计；外部动作技能不因重复成功获得新权限。

### M8 — Connector 与凭据引用

交付：

- ConnectorDefinition、ConnectionAccount、CredentialRef、CapabilityGrant、SyncCursor、InboundItem、ActionRequest / Result；
- `view / index / sync / action / secret` 分开声明、授权、撤销和审计；
- 分开抽取 AgentRuntime、Agent / Model、Tool、Harness、KnowledgeSource 与 Connector 端口；只统一 capability envelope，不合成一个万能 adapter；
- DeepSeek Harness 若进入，只属于 `AgentRuntimeAdapter`；其 Tool / Plugin 只能提交 CapabilityRequest / ActionIntent，不能直接取得 Connector 凭据或外部写权；
- 先把 Codex（代码智能体）、外部知识库与文件源、主管能力协议和开发护栏包装为对应内部适配器；外部系统支持模型上下文协议（MCP）时优先使用，但核心知识与上下文服务本身不放在适配器层；
- 一个低风险只读外部 connector 作为第一真实样本；
- 设置 / 管理面显示来源、授权范围、最近同步、错误、断开和撤权，不显示 secret 正文。

进入真实连接器前，每个 provider 单独冻结数据合同：真源、正文 / 引用、同步方向、冲突、删除、撤权、外发、保留期和写操作确认。凭据存储选型和真实 secret 使用单独重档授权；secret 尽量只在 connector adapter 内解引用，不进入 runtime session、trace、memory 或普通日志。文件系统 sandbox 不等于网络、进程、同 UID 凭据和现实副作用隔离。

退出门：未授权 capability 在 adapter 前拒绝；secret 不进 SQLite event / audit / memory / chat；至少两种内部标识和调用形状不同的伪适配器通过同一角色、会话、权限和结果合同，证明接口不绑定 Codex 或某一家线程编号；mock connector 全合同通过；真实只读 connector 的授权、同步、断开和错误在 App 可见。

### M9 — 读模型切换、数据迁移与旧路退役

交付：

- 首页、项目、角色会话、成员、运行中、待办、日报、审计各自 projector / query；
- UI 不再拼底层 sidecar 或反复构建完整 snapshot；
- 所有 UI 条目有 typed `ObjectRef` 和精确 deep link；
- shadow read → new primary → compatibility read-only；
- 按 §4.4 清单逐项 command unregister、归档和恢复演练。
- runtime / profile / package / session compatibility 纳入 inventory；可逆卸载只证明内部注册清理，现实副作用单独 reconcile / compensate。

退出门：新旧页面逐页 parity；差异有批准说明；投影可重建；raw JSON 默认不出产品响应；旧 store 有只读 manifest / hash / export；回滚演练通过。物理删除另行批准。

### M10 — 全日使用试点与发布硬化

试点剧本至少覆盖：

1. 打开 Syn，秘书给出可回源情境；
2. 收到个人 / 外部事件，形成关注但不自动成任务；
3. 用户明确把复杂事项升级为项目；
4. 项目主管持续对话，明确创建方案并经授权执行；
5. 项目结果回到项目，秘书只收到摘要；
6. 全局主管复核跨项目冲突；
7. 日终日报、记忆治理和 skill 候选分别结算；
8. App 重启后会话、未决、运行和关注恢复；
9. 连接器断开、provider 失败、DB / 投影失败和敏感内容被 fail closed；
10. 用户能找到任一稳定成员和历史临时 agent；
11. 用户在前端明确编辑或纠正后，内容写回唯一真源，重启后的角色读到新版；
12. 常驻角色带来源恢复用户是谁、近期与长期工作和未闭环事项，知识深度可以纠正；
13. 稳定角色与临时智能体按不同权限取得正确资料和技能说明，未获准的技能仍不可执行。

硬化项：性能、分页、索引重建、备份 / 恢复、迁移回滚、可观测性、secret scrub、成本 / 预算、故障注入、长时间运行、权限回归、真实桌面可用性；另验证 runtime 中途崩溃、session resume / fork / compaction、child grant 不扩权、插件供应链、sandbox degraded、重复外部副作用、动态 package 默认关闭，以及执行者与验收者分离。预算同时覆盖钱、token、时间、模型、工具和外部调用，不把 token pressure 当完整成本治理。

发布门：所有目标场景真实 App 通过；高风险外部动作仍未默认开放；旧路退役清单完成或明确 HOLD；没有把 synthetic / build 冒充真实使用。

### M11 — 受治理自升级平台

详见 `2026-08-16-syn-stage-11-governed-self-upgrade-platform-plan-v1.md`。

交付：

- ImprovementSignal、UpgradeCandidate、CandidateArtifact、CapabilityDiff、EvaluationReceipt、CanaryAssignment、PromotionDecision 与 RollbackReceipt；
- Skill / Profile / Plugin、Runtime / Provider 兼容和 Syn 代码三条升级轨道；
- 隔离 worktree / workcell、供应链与权限差异、确定性测试、历史回放、独立 Verifier、canary、签名晋升、监控和回滚；
- 受保护治理根：Identity / Scope / Policy / Grant / Credential / Audit / Budget / Verifier / Updater / Kill Switch 不进入普通自升级。

退出门：三条轨道各有至少一个有界样本走完候选到回滚；低风险自动晋升只来自用户签署的有限政策；核心代码和治理根不能自批；外部 runtime 是否接入由同一 conformance 与成本证据决定。

## 7. 并行与写所有权

允许并行的前提：公共合同已冻结、文件写域不重叠、同一事实对象只有一个 owner。

- Kernel / schema / migration 承重文件单写者；
- Rust domain 与 React consumer 可以在 DTO 冻结后并行；
- M4 Secretary 与 M5 Project Orchestration 可并行，但不得同时改公共 event / scope / App 装配；
- 知识和记忆可以分域并行，但知识来源登记、上下文包所有权以及统一工作单元与发件箱各自只有一个写入者；
- Connector framework 与具体 connector 分开；真实 provider 一次只激活一个；
- 自升级的 candidate writer、security reviewer、verifier、rollout / updater 分写面；候选不能修改 oracle 或晋升策略；
- UI 视觉优化不与读模型切换混在同一任务包，除非验收必须。

每个任务包必须声明：`authority_chain`、`plan_anchor`、`existing_before_new`、`write_surface`、`capabilities_touched`、`forbidden_alternatives`、`verification`、`rollback` 和 `retirement_effect`。

## 8. 任务包与阶段激活规则

- master plan 只定义顺序，不构成执行授权。当前用户清晰自然语言决定目标与边界；current stage / leaf 是工作投影，不能扩大用户原话；`authorization.json` 只用于满足精确绑定条件的短期 Stop 内部续跑。
- 每个任务包只交付一个可独立验收的薄切片，不能用“全面重构”做无界写面。
- 安全 / scope、schema migration、真实 provider、凭据、真实项目写入、自动连环、升级晋升和不可逆退役分别建包。
- 涉及安全闸、scope 或授权判断的实现包按 `AGENTS.md` 高危清单逐包取得用户明确授权；阶段已排入计划不等于这项授权已经发生。
- 一个阶段未通过退出门，下一阶段只能做不依赖它的只读设计或隔离 fixture，不得假设前置已完成。
- 完成任务后更新 `../current-state.md` 的事实、证据和下一入口；不在总计划复制逐任务进度。

## 9. 全程停止条件

遇到以下任一情况立即停止当前切片：

- 目标对象没有唯一真源或无法证明 project / scope owner；
- 新合同需要静默放宽旧权限、Station 3b 或高风险确认；
- migration 无 before / after、幂等、回滚或残留解释；
- 写面撞上未归属的 dirty WIP；
- 真实消息、外部写入、凭据、生产项目、发布或 Git 操作没有单独授权；
- 只能靠前端隐藏来声称后端安全；
- 只能靠 fake / synthetic 来声称真实 App 可用；
- 为赶进度需要一次删除多个旧真源或失去恢复路径。
- runtime / plugin / 自升级候选需要改写自己的权限、预算、审计、oracle、Verifier 或晋升门；
- 外部副作用结果未知却只能靠自动重试继续。

停止后只报告事实、影响和所需新决定，不私自改目标或放宽验收。

## 10. 完成定义

Syn 的这轮重构只有在以下条件同时满足时才算完成：

- 用户每天可以从秘书、全局主管和项目内项目主管明确工作；
- 角色、项目、对象、通道和权限不会被模型或前端猜测；
- 个人事项与项目工作都能有真源、有闭环、不丢失；
- 所有稳定角色、临时智能体、执行者和审查者都能按角色、范围、任务和权限快速取得所需资料、来源与可用技能说明；
- 常驻角色能带来源恢复用户身份、近期与长期工作、未闭环事项和可纠正的知识深度；
- 日常对话不会误建正式对象，明确决定能可靠进入治理链；
- 项目主管能调用保留下来的方案、授权、执行、验证和交付能力；
- 首页、日报、通知和摘要全部可回源且不形成镜像；
- 用户在前端的明确修改进入唯一真源，并被后续角色稳定读到；
- 记忆可以自发捕获和每日整理，同时可纠正、可审计、不泄密、不扩权；
- 外部连接器按能力分权，凭据只以保护引用参与；
- 内部成员可搜索、可直接联系，临时 agent 可追踪；
- App 重启、失败、迁移和回滚不会让会话、未决、项目事实或未闭环事项静默消失；
- 旧业务路径在替代链真实验收后有序退役；
- Runtime、模型和插件可替换且不会改变角色、事实、权限与未完成 operation；
- 系统能在受保护治理根之下生成、验证、灰度、晋升和撤回改进，不能自己降低验收标准；
- 产品正本、当前状态、代码、测试、真实应用和发布结论各自诚实一致。
