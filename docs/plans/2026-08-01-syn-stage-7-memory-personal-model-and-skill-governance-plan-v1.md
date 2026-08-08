# Syn Stage 7：知识、记忆、个人模型与技能治理计划 v1

日期：2026-08-01<br>
阶段：`M7`<br>
状态：**PLANNED / NOT_ACTIVE / NO_EXECUTION_AUTHORITY。**<br>
上位计划：`2026-08-01-syn-personal-ai-workbench-master-development-plan-v1.md` M7。<br>
分层前置：知识来源登记、上下文包合同以及隔离存储与策略只需 M1/M2 退出证据；接入真实角色会话需再等 M3，接入真实采集与每日整理需再等 M4/M5。任何 M7 实现仍需独立活动任务包。<br>
当前活动阶段 / 叶：无（`NONE`）；M3 尚未激活，本计划也未激活；本计划不授权自动学习、真实聊天采集、技能启用、桌面应用或产品代码。

权威顺序：当前用户指令 → `../../../AGENTS.md` → `../../AGENTS.md` → `../harness/plan.md` → 活动阶段（stage）/ 唯一活动叶（leaf）→ `../harness/authorization.json` → `../product/syn-product-canon-v1.md` 与 `../product/knowledge-infrastructure-canon-v1.md` → `../current-state.md` → 记忆与全工作台修订 → 总计划 → 对应分层前置回执 → 本计划。前置较少的切片可提前，不等于整阶段可标完成。

## 0. 当前事实与未知

### 已有较厚基础

- 原生知识工作区已有固定根、文件真源、路径与大小限制、引用、恢复和只读检索素材；任务包已有“只给允许资料、缺资料可请求更多”的局部原则。
- 当前架构已有 `KnowledgeAdapter`（知识适配器）和知识页面，但主要面向外部资料源与人类工作区，尚未形成所有智能体共用的知识与上下文服务。
- Memory 已有 `capture → observation → candidate → user confirmation → formal` 治理语义；candidate 不得冒充 formal。
- Formal record / version / audit 在同一 sidecar 内可原子替换；lint、entity / relation、mature pattern 和 task memory packet 已存在。
- TaskMemoryPacket 已有正式记忆过滤、store revision、fingerprint 和 stale 检测，应 KEEP / HARDEN，不按全新能力重做。
- 当前普通聊天明确不直接变 Observation，这是需要保留的安全语义。

### 当前缺口

- 没有统一的知识来源登记、权威状态、新鲜度、作用域、敏感性和权限过滤合同。
- 常驻角色、临时智能体、执行者和审查者尚未在会话开始、恢复、任务派发和交接时收到可追溯的最小知识上下文包。
- 技能说明散落在不同来源，知识检索尚未与技能登记、版本和权限治理建立发现桥梁。
- 活跃真源仍是多个 JSON sidecar；SQLite 多为历史快照 / bridge，不是统一 live mirror。
- Capture 写 Observation、Candidate、Capture ledger；Observation→Candidate 和 Candidate→Formal 都存在跨 store 顺序写半状态窗口。
- 成熟模式确认也可能先写 Formal、后写 pattern store，失败产生残留。
- 当前 “daily loop” 只处理少数治理事件的 best-effort candidate，失败仅 warning；不是 scheduler、DailyReport 或全日 consolidation。
- 生产源码没有 `PersonalFact`、`ModelAssertion / PersonalModel`、`SkillCandidate / SkillDraft / SkillVersion` 领域实现。
- Skill 页面只是只读索引，没有登记、验证、启用、回滚的治理状态。

### HOLD / 需冻结决定

- 来源登记字段、权威与新鲜度词表、全文 / 标签 / 语义 / 图关系的最小采用顺序；
- 默认检索范围、资料预算、排序与裁剪、来源冲突、索引降级和缺资料申请语义；
- 知识上下文包与角色会话、任务包、交接、记忆召回和技能发现的精确接口；
- 自动 policy matrix：可自动、需确认、禁止；事实、观察、推断、纠正、冲突、敏感内容的阈值；
- 普通聊天何时只产生 Observation，哪些事件永不采集；
- PersonalFact 与 ModelAssertion 的时效、置信度、冲突、纠正和批量撤销；
- 用户知识深度的领域范围、证据、新鲜度、置信度和过期策略；
- daily consolidation 时区 / window / budget / checkpoint / retention；
- Skill 候选阈值、验证环境、启用权限、来源权限上限与废弃策略；
- SQLite / sidecar 最终切换、历史对账和真实 App 数据。

## 1. 阶段目标

1. 建立所有智能体共用的知识与上下文服务：来源登记、索引、检索路由、上下文装配、引用、新鲜度和缺口反馈；
2. 常驻角色获得稳定默认知识范围，临时智能体获得当前任务的最小资料包，并能说明理由请求更多；
3. 让技能说明可被知识层发现和解释，但实际启用继续经过技能登记、角色权限和任务授权；
4. 把记忆捕获、提升和成熟模式决定迁到 M2 事务或可证明恢复的长事务；
5. 冻结自动、需确认、禁止的策略矩阵，每次自动处理都有策略结果和审计；
6. 严格分型 `PersonalFact`（个人事实）与 `ModelAssertion`（模型推断），都可追源、纠正、版本化、撤销；
7. 在 PersonalFact 与 ModelAssertion 上建立可纠正的用户知识深度，用于调整解释深度和资料选择，不因一次对话固化；
8. 对话、项目结果、用户纠正和重复模式只经结构化事件进入观察，再按策略进入候选；
9. 建立真正的每日整理运行，支持预算、检查点、重跑和部分失败恢复；
10. 建立技能候选、技能草稿和技能版本，本阶段首个实现只采用本地验证、人工启用、可回滚；低风险内部技能的策略自动启用保留为独立暂缓项；
11. 用户可查看、纠正、冻结、废弃、关闭自动学习和批量撤销；敏感信息、权限、允许清单、沙箱和外部动作授权永不因知识、记忆或技能自动扩张。

## 2. 本阶段不做

- 不把全部资料库默认塞给每个角色，不让索引或检索结果冒充事实、用户决定、记忆或授权；
- 不让“发现可用技能”自动变成“启用技能或取得工具权限”；
- 不要求第一片同时完成所有外部资料同步、语义索引、图谱、知识界面和自动写回；
- 不把 raw transcript、prompt、tool output、credential 或 secret 存成 memory；
- 不让普通聊天直接写 FormalMemory / PersonalFact / Skill；
- 不让模型推断冒充用户明确事实；
- 不静默覆盖冲突或历史版本；
- 不让重复成功自动扩大 Skill 的权限 / scope / capability；
- 不把 M4 DailyReport 当 FormalMemory，也不让 M7 反写 M4 attention；
- 不在 parity 前删除 JSON sidecar；
- 不接真实 external connector 数据或凭据；
- 不以已有 candidate UI、task packet 或 synthetic test 声称自动学习闭环完成。

## 3. 对象、owner 与政策底线

| 对象 | owner / 真源 | 不变量 |
|---|---|---|
| `KnowledgeSource` | knowledge source registry | owner、scope、类型、版本、新鲜度、敏感性、权威状态和来源适配器；外部适配器不是核心真源 |
| `KnowledgeContextPacket` | context assembler | role、scope、task/object、权限快照、来源引用、纳入/排除理由、预算与缺口；可重建，不授予执行权 |
| `SkillDescriptorRef` | knowledge-to-skill bridge | 只描述技能、版本、适用条件与依赖；实际启用和调用由 skill governance 决定 |
| `Observation` | memory capture | source event/ref、scope、sensitivity、policy input；不是事实 |
| `MemoryCandidate` | memory governance | candidate reason、conflict、policy result；需明确提升规则 |
| `FormalMemory` | formal memory repository | versioned、source-backed、user-visible、可纠正 / 废弃 |
| `PersonalFact` | personal fact domain | 用户明确 / 可靠确定性来源；不能由模型推断自动生成 |
| `ModelAssertion` | personal model domain | inference、confidence、validity window、evidence、可否认 |
| `ConsolidationRun` | daily memory service | 消费 M4 `daily_window_id / DailyReportVersion`、input watermark、budget、checkpoint、result refs、status；不拥有 wall-clock scheduler |
| `SkillCandidate/Draft/Version` | skill governance | source refs、tests、runtime capability ceiling、manual-first enable、rollback |
| `TaskMemoryPacket` | task packet builder | revision / fingerprint / stale；只消费 active formal memory |

政策硬边界：`secret / credential / auth / permission / allowlist / sandbox / external-action approval` 一律禁止自动提升；冲突、敏感内容和推断默认 Candidate / Quarantine；自动规则只能匹配显式、低风险、版本化的 allowlist。

Skill capability ceiling 每次执行都按“来源权限 ∩ 生成者权限 ∩ 验证者权限 ∩ 启用者权限 ∩ 当前 RoleSession / grant”重新计算；grant 撤销、scope 变化、source 失效或任一项收窄时立即降级 / 禁用，不能只在启用时检查一次。

## 4. 任务切片

### SYN-KNO-001 — 知识来源与上下文合同

冻结来源登记、作用域、权威、新鲜度、敏感性、权限过滤、检索请求、上下文包、引用、缺口和索引可重建合同；先用隔离资料夹和伪技能目录验证，不接真实外部资料源，不启动模型。

### SYN-KNO-002 — 角色、任务与技能发现最小闭环

在 M3 角色会话合同可用后，只接一种稳定角色会话和一种临时任务：开始或恢复时装配最小资料包，显示来源与缺口，允许请求更多；同时能发现技能说明，但不启用技能。其他角色、完整知识界面、更多索引和外部同步后续逐包扩展。

### SYN-MEM-001 — Memory / PersonalModel / Skill 合同与策略矩阵

冻结 owner、状态机、事实 / 观察 / 推断分类、自动 / 确认 / 禁止矩阵、敏感矩阵、冲突、撤销、retention、Skill capability ceiling、migration / compensation。只写合同。

### SYN-MEM-002 — 跨 store UoW / reliable saga

只修 Capture→Observation→Candidate、Observation→Candidate、Candidate→Formal、MaturePattern→Formal 的事务 / saga、幂等键和恢复队列；不扩大采集来源。必须对每条链逐故障点冻结：deterministic operation / idempotency key、已落对象探测、补链 / 补 ledger / 补 audit、重试返回同一 object / Formal ID、何时 quarantine，不能只以“存在 repair queue”验收。

最低恢复表：Observation 已写但 Candidate / capture ledger 未写；Candidate 已写但 Observation link 未写；Formal 已写但 Candidate adoption link 未写；Formal 已写但 MaturePattern link 未写。每种残留都要有 machine-detectable state、唯一 repair action、幂等回执和故障注入证据。

### SYN-MEM-003 — Sidecar shadow / parity / migration

为 observation、candidate、formal、lint、relation、pattern 建 exact manifest、revision / count / fingerprint / link / audit parity。旧 sidecar 保持只读 / 可回切；unknown / corrupt 显式隔离。

### SYN-MEM-004 — PersonalFact 与 ModelAssertion

新增最小 schema / repository / read model；明确 source、confidence、validity、conflict、correction、freeze、withdraw。用户明确表达的领域熟悉度进入 PersonalFact，系统根据长期行为形成的知识深度进入 ModelAssertion，并带来源、置信度和时效；两者共同形成可纠正的解释深度读模型，不新建第三套真源。推断只进入 ModelAssertion candidate，不自动成为 Fact。

### SYN-MEM-005 — 结构化事件 capture

先接 M4 / M5 已结构化低风险事件；普通聊天只能形成受限 Observation。每种新 source 单独 policy / sensitivity / dedupe / retention package。

### SYN-MEM-006 — Daily ConsolidationRun

独立于 M4 DailyReport，但不建立第二个 wall-clock scheduler。只消费 M4 `DailyWindowClosed / DailyReportVersioned`，以 `daily_window_id + report_version` 为幂等键，实现 watermark、budget、checkpoint、partial failure、rerun、result refs。输出可以是 M7-owned report-annotation artifact/ref、memory candidate、model assertion candidate 或 skill candidate；只能发 result event 供 M4 projector 接纳，不原位修改 DailyReport / Attention，也不能自动越级。

### SYN-MEM-007 — Skill 生命周期

实现 Candidate→Draft→Version→validated→enabled / disabled / rolled_back；验证只在隔离本地 fixture。本阶段首个实现 / exit evidence 只采用人工启用；满足已批准修订的低风险自动启用仍是独立 HOLD / package，不永久删除该产品方向。每次执行按完整权限交集重算 capability ceiling。

### SYN-MEM-008 — 用户控制与批量撤销

查看 source/history/policy result，纠正、冻结、废弃、关闭自动学习、按 source / window 批量撤销；所有操作可审计、幂等且不物理抹除必要历史。

### SYN-MEM-009 — 隔离 App 与故障注入验收

覆盖 Observation 后、Candidate 后、Formal 后、ledger 前后、daily checkpoint、Skill validation 的逐点失败与恢复。真实聊天、真实个人数据、真实 Skill 启用分别另包。

## 5. 顺序、并行与写域

```text
M7-A: KNO-001 与 MEM-001 可并行
M7-B（等待 M3）: KNO-002
M7-C: MEM-002 → MEM-003 → MEM-004 → MEM-007
M7-D（等待 M4/M5）: MEM-005 → MEM-006
全部分片 → MEM-008 → MEM-009
```

- 知识来源登记和上下文包各有唯一写入者；全文、标签、语义和图索引都是可重建读模型；
- 知识服务负责发现和装配，事实、记忆、技能、权限仍由各自领域拥有；
- memory / personal model / skill domain 分 owner；公共 UoW / event / migration 由 M2 owner；
- M7-A 的知识与记忆合同可在 M2 退出后独立激活；M7-B 的角色知识装配依赖 M3；M7-D 的真实事件采集与每日整理依赖 M4/M5。任一分片完成都不得把整阶段标为完成；
- M4 owns DailyReport / attention，M7 owns ConsolidationRun / memory；通过 source event/ref 连接，不共享表写；
- 项目实时状态和 `ProjectSummary` 继续由 M5 项目 owner 持有；M7 只登记、索引和引用，只有经治理确认的长期决定与背景才可进入记忆；
- M8 connector source 接入必须等 M8 provider data contract；M7 不读 credential；
- Rust repositories 与 React governance UI 在 DTO 冻结后可并行；
- sidecar、SQLite schema、AppState、command registry、Memory Center shell 均需唯一 writer 与 opening hash。

## 6. 迁移与回滚

- 采用 shadow write / readback，逐条核 revision、count、fingerprint、links、audit；
- 先新 primary read，再旧 sidecar compatibility read-only，最后 M9 才退役写 / command；
- rollback 只切回已验证旧读主，不删除新记录；通过 idempotency / source refs 防重复；
- 半状态由 durable saga state / repair queue 解释，不以 warning 结束；
- 冲突产生新 candidate / resolution record，不覆盖旧 Formal / Fact / Assertion；
- 批量撤销以 tombstone / lifecycle 处理并保留 audit；
- Skill rollback 恢复上一启用 version，不扩大权限，也不自动删除候选历史；运行时权限交集收窄时立即降级。

## 7. 验证矩阵

| 层级 | 必须证明 | 不能声称 |
|---|---|---|
| Knowledge fixtures | 角色、范围、任务与权限过滤；来源、新鲜度、缺口和技能说明可追溯 | 已接全部角色、真实资料源或技能执行 |
| Contract / policy fixtures | 分类、自动矩阵、权限 ceiling、冲突一致 | service 已实现 |
| Unit / property | 幂等、冲突、saga、撤销、secret exclusion | live memory 已迁移 |
| Temp stores / fault injection | 每个半写点可恢复、parity、daily rerun、Skill rollback | 真实 App / 数据通过 |
| Non-test build | production path 可构建 | 桌面行为正确 |
| Isolated Tauri | 用户控制、source/history、故障状态可见 | 真实聊天 / Skill 启用通过 |
| 经授权 live scenarios | 指定 source、数据、policy 的真实证据 | 全自动学习或发布通过 |

机械验收：无不可解释半状态；自动写每条有 policy result / audit；secret / permission 不进入自动提升；冲突不覆盖；同一 source / window 重跑不重复 Formal / Fact / Skill；TaskMemoryPacket stale 仍准确。

## 8. 授权与停止条件

真实知识源读取或写回、外部索引服务、知识跨范围共享、实时旁路存储迁移、每种真实采集来源、普通聊天采集、个人事实自动规则、真实模型调用、技能验证或启用、批量撤销真实数据分别建包。M7 不授权外部连接器、凭据、外部动作、项目执行、Git（版本控制写入）或发布。

立即停止：政策矩阵未冻结；事实 / 推断混型；secret / permission 进入记忆或 Skill；跨 store 半状态无恢复；冲突被覆盖；Skill 自动扩权，或自动启用缺独立 package、未命中冻结 policy matrix、超过 runtime capability ceiling；M4/M7 双写日报；迁移缺 rollback；WIP 冲突；fixture 被表述成 live data 验收。

## 9. 阶段退出与 M8 输入

全部满足才完成 M7：

- 稳定角色和临时智能体都能按角色、范围、任务和权限取得带来源的最小资料包，资料不足、过期、冲突和技能不可用可见；
- 检索命中不自动成为事实或记忆，技能发现不自动启用或扩权，索引可以从来源重建；
- capture / adoption / pattern 决策没有不可解释半状态；
- PersonalFact / ModelAssertion 分型、来源、纠正、历史和撤销成立；
- 一个常驻角色重启后能带来源、时间和作用域恢复“用户是谁、近期做了什么、长期做过什么、当前有哪些未闭环事项”；用户知识深度可纠正、否认和过期；
- ConsolidationRun 幂等、可恢复、预算可观测；
- SkillCandidate / Draft / Version 可验证、manual-first 启用、运行时重算 ceiling、可回滚且不扩权；低风险自动启用若未单独完成则明确 HOLD；
- sidecar migration 有 parity / compatibility / rollback，未物理删除；
- isolated App 故障场景通过；真实数据结论分项记录；
- 向 M8 提供 connector inbound event 的 memory policy contract；
- `../current-state.md` 回写实际完成、暂缓和下一入口；M8 未激活不得续跑。
