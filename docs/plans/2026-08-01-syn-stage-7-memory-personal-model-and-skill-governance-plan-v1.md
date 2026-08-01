# Syn Stage 7：记忆、个人模型与 Skill 治理计划 v1

日期：2026-08-01<br>
阶段：`M7`<br>
状态：**PLANNED / NOT_ACTIVE / NO_EXECUTION_AUTHORITY。**<br>
上位计划：`2026-08-01-syn-personal-ai-workbench-master-development-plan-v1.md` M7。<br>
分层前置：`M7-A` storage / policy / isolated repository 只需 M1/M2 exit；`M7-B` live capture / daily integration 必须再等 M4/M5 exit。任何 M7 实现仍需独立 active package。<br>
当前 active node / package：`NONE`；本计划不授权自动学习、真实聊天采集、Skill 启用、App 或产品代码。

权威顺序：当前用户指令 → `../harness/AUTHORITY.md` / `../harness/CURRENT.md` → memory 与 whole-workbench 修订、当前 inventory → master → 对应分层前置 receipts → 本计划。M7-A 可提前不等于整阶段可标完成。

## 0. 当前事实与未知

### 已有较厚基础

- Memory 已有 `capture → observation → candidate → user confirmation → formal` 治理语义；candidate 不得冒充 formal。
- Formal record / version / audit 在同一 sidecar 内可原子替换；lint、entity / relation、mature pattern 和 task memory packet 已存在。
- TaskMemoryPacket 已有正式记忆过滤、store revision、fingerprint 和 stale 检测，应 KEEP / HARDEN，不按全新能力重做。
- 当前普通聊天明确不直接变 Observation，这是需要保留的安全语义。

### 当前缺口

- 活跃真源仍是多个 JSON sidecar；SQLite 多为历史快照 / bridge，不是统一 live mirror。
- Capture 写 Observation、Candidate、Capture ledger；Observation→Candidate 和 Candidate→Formal 都存在跨 store 顺序写半状态窗口。
- 成熟模式确认也可能先写 Formal、后写 pattern store，失败产生残留。
- 当前 “daily loop” 只处理少数治理事件的 best-effort candidate，失败仅 warning；不是 scheduler、DailyReport 或全日 consolidation。
- 生产源码没有 `PersonalFact`、`ModelAssertion / PersonalModel`、`SkillCandidate / SkillDraft / SkillVersion` 领域实现。
- Skill 页面只是只读索引，没有登记、验证、启用、回滚的治理状态。

### HOLD / 需冻结决定

- 自动 policy matrix：可自动、需确认、禁止；事实、观察、推断、纠正、冲突、敏感内容的阈值；
- 普通聊天何时只产生 Observation，哪些事件永不采集；
- PersonalFact 与 ModelAssertion 的时效、置信度、冲突、纠正和批量撤销；
- daily consolidation 时区 / window / budget / checkpoint / retention；
- Skill 候选阈值、验证环境、启用权限、来源权限上限与废弃策略；
- SQLite / sidecar 最终切换、历史对账和真实 App 数据。

## 1. 阶段目标

1. 把 memory capture / adoption / mature pattern 决策迁到 M2 transaction 或可证明恢复的 saga；
2. 冻结自动、需确认、禁止的 policy matrix，每次自动处理都有 policy result / audit；
3. 严格分型 `PersonalFact` 与 `ModelAssertion`，都可追源、纠正、版本化、撤销；
4. 对话、项目结果、用户纠正和重复模式只经结构化事件进入 Observation，再按 policy 进入 Candidate；
5. 建立真正的 daily `ConsolidationRun`，支持预算、checkpoint、重跑、部分失败恢复；
6. 建立 `SkillCandidate → SkillDraft → SkillVersion`，本阶段首个实现只采用本地验证、人工启用、可回滚；低风险内部 Skill 的策略自动启用保留为独立 HOLD / 后续 package；
7. 用户可查看、纠正、冻结、废弃、关闭自动学习和批量撤销；
8. secret、permission、allowlist、sandbox 和外部 action 授权永不因记忆 / Skill 自动扩张。

## 2. 本阶段不做

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

### SYN-MEM-001 — Memory / PersonalModel / Skill 合同与策略矩阵

冻结 owner、状态机、事实 / 观察 / 推断分类、自动 / 确认 / 禁止矩阵、敏感矩阵、冲突、撤销、retention、Skill capability ceiling、migration / compensation。只写合同。

### SYN-MEM-002 — 跨 store UoW / reliable saga

只修 Capture→Observation→Candidate、Observation→Candidate、Candidate→Formal、MaturePattern→Formal 的事务 / saga、幂等键和恢复队列；不扩大采集来源。必须对每条链逐故障点冻结：deterministic operation / idempotency key、已落对象探测、补链 / 补 ledger / 补 audit、重试返回同一 object / Formal ID、何时 quarantine，不能只以“存在 repair queue”验收。

最低恢复表：Observation 已写但 Candidate / capture ledger 未写；Candidate 已写但 Observation link 未写；Formal 已写但 Candidate adoption link 未写；Formal 已写但 MaturePattern link 未写。每种残留都要有 machine-detectable state、唯一 repair action、幂等回执和故障注入证据。

### SYN-MEM-003 — Sidecar shadow / parity / migration

为 observation、candidate、formal、lint、relation、pattern 建 exact manifest、revision / count / fingerprint / link / audit parity。旧 sidecar 保持只读 / 可回切；unknown / corrupt 显式隔离。

### SYN-MEM-004 — PersonalFact 与 ModelAssertion

新增最小 schema / repository / read model；明确 source、confidence、validity、conflict、correction、freeze、withdraw。推断只进入 ModelAssertion candidate，不自动成为 Fact。

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
M7-A: MEM-001 → MEM-002 → MEM-003 → MEM-004 → MEM-007
M7-B（等待 M4/M5）: MEM-005 → MEM-006
M7-A + M7-B → MEM-008 → MEM-009
```

- memory / personal model / skill domain 分 owner；公共 UoW / event / migration 由 M2 owner；
- M7-A storage / policy / isolated repository 可在 M2 exit 后独立激活；M7-B 真实事件 capture / daily integration 依赖 M4/M5 exit，M7-A 完成不得把整阶段标 `COMPLETE`；
- M4 owns DailyReport / attention，M7 owns ConsolidationRun / memory；通过 source event/ref 连接，不共享表写；
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
| Contract / policy fixtures | 分类、自动矩阵、权限 ceiling、冲突一致 | service 已实现 |
| Unit / property | 幂等、冲突、saga、撤销、secret exclusion | live memory 已迁移 |
| Temp stores / fault injection | 每个半写点可恢复、parity、daily rerun、Skill rollback | 真实 App / 数据通过 |
| Non-test build | production path 可构建 | 桌面行为正确 |
| Isolated Tauri | 用户控制、source/history、故障状态可见 | 真实聊天 / Skill 启用通过 |
| 经授权 live scenarios | 指定 source、数据、policy 的真实证据 | 全自动学习或发布通过 |

机械验收：无不可解释半状态；自动写每条有 policy result / audit；secret / permission 不进入自动提升；冲突不覆盖；同一 source / window 重跑不重复 Formal / Fact / Skill；TaskMemoryPacket stale 仍准确。

## 8. 授权与停止条件

live sidecar migration、每种真实 capture source、普通聊天采集、PersonalFact 自动规则、真实 model call、Skill validation / enable、批量撤销真实数据分别建包。M7 不授权 external connector、credential、外部 action、项目执行、Git 或发布。

立即停止：政策矩阵未冻结；事实 / 推断混型；secret / permission 进入记忆或 Skill；跨 store 半状态无恢复；冲突被覆盖；Skill 自动扩权，或自动启用缺独立 package、未命中冻结 policy matrix、超过 runtime capability ceiling；M4/M7 双写日报；迁移缺 rollback；WIP 冲突；fixture 被表述成 live data 验收。

## 9. 阶段退出与 M8 输入

全部满足才完成 M7：

- capture / adoption / pattern 决策没有不可解释半状态；
- PersonalFact / ModelAssertion 分型、来源、纠正、历史和撤销成立；
- ConsolidationRun 幂等、可恢复、预算可观测；
- SkillCandidate / Draft / Version 可验证、manual-first 启用、运行时重算 ceiling、可回滚且不扩权；低风险自动启用若未单独完成则明确 HOLD；
- sidecar migration 有 parity / compatibility / rollback，未物理删除；
- isolated App 故障场景通过；真实数据结论分项记录；
- 向 M8 提供 connector inbound event 的 memory policy contract；
- CURRENT 回写实际完成 / HOLD / 下一步，M8 未激活不得续跑。
