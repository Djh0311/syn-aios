# Syn Stage 6：全局主管与内部组织计划 v1

日期：2026-08-01<br>
阶段：`M6`<br>
状态：**PLANNED / NOT_ACTIVE / NO_EXECUTION_AUTHORITY。**<br>
上位计划：`2026-08-01-syn-personal-ai-workbench-master-development-plan-v1.md` M6。<br>
硬前置：M3 Global Supervisor RoleSession / Handoff；M5 ProjectSummary 与 execution identity；M4 Secretary consult 入口。<br>
当前 active node / package：`NONE`；本计划不授权跨项目写、真实消息、App 或产品代码。

权威顺序：当前用户指令 → `../harness/AUTHORITY.md` / `../harness/CURRENT.md` → 2026-08-01 修订与当前 inventory → master → M3-M5 exit receipts → 本计划。M5 未验收的 legacy records 不得被本计划升级成统一组织事实。

## 0. 当前事实与未知

### 已有局部能力

- 当前 global supervisor 能对单个项目 proposal 做批前只读咨询 / review，并把 advisory 写入独立 store；意见不自动批准项目。
- Agent Center 能显示 conversation sessions、projects、adapter / provider 状态，可作为成员目录 UI 素材。
- 当前 legacy attempts、reports、sessions 与 audits 只可作迁移材料；M6 只消费经 M5 验收的 WorkItem / Attempt / Report identity contract，无法精确映射的记录进入 quarantine。

### 尚未成立

- 导航和 Active View 没有顶层 Global Supervisor 页面或持久会话；
- 当前 global supervisor 不是跨项目摘要、依赖、风险、优先级服务；
- 没有稳定成员、角色档案、能力 / 权限、availability、直接联系与生命周期模型；
- 没有 temporary agent registry / history，现有 session / attempt 也没有统一成员身份；
- 目标 ProjectSummary 尚依赖 M5，实现前不能从完整 snapshot 猜跨项目事实；
- 没有两个项目冲突、source deep link、跨重启、意见不反写的真实 App 验收。

### HOLD / 需冻结决定

- `StableMember` 的创建、更新、停用、权限、直接联系与 provider binding；
- `TemporaryAgent` 的身份来源、任务 / attempt 绑定、历史保留与是否允许人工晋升；
- availability 的定义、TTL、busy / offline / unknown 与陈旧提示；
- 全局主管最小可读 ProjectSummary 字段、freshness / sensitivity / ACL；
- 用户采纳跨项目建议后，由谁、按哪些 per-project grants 应用，部分成功如何回滚；
- 真实全局主管模型 / provider、成本和跨项目真实数据。

## 1. 阶段目标

1. 提供顶层 Global Supervisor 入口和持久 RoleSession；
2. 只读消费多个项目的最小 ProjectSummary，给出风险、依赖、冲突和优先级建议，每项可回源；
3. Secretary → Global Supervisor 的咨询用显式 Handoff，留痕、可拒绝、可回执；
4. 任何 advisory 未经用户确认不改变任何项目；即使用户采纳，也由各项目 owner 通过独立 grant 执行；
5. 建立稳定成员目录：角色、能力、权限、availability、会话和直接联系；
6. 建立临时 agent 历史：来源任务、attempt、结果和审计，并与稳定成员严格分型；
7. 内部组织默认是后台能力与目录，不强迫组织图成为日常入口。

## 2. 本阶段不做

- 不建立跨项目共享业务数据库或复制项目原始事实；
- 不让 Global Supervisor 直接写项目、创建 workflow、批准 grant 或执行 action；
- 不把 advice / summary 变成项目正式事实；
- 不从 session 数量、名称或 provider thread 猜成员身份；
- 不让 temporary agent 因重复使用自动升级为 stable member；
- 不在 M6 实现组织薪酬、HR、通用组织图或多租户权限系统；
- 不接真实外部 connector / credential；
- 不以单项目 review 或静态成员卡片声称跨项目能力完成。

## 3. 对象、owner 与边界

| 对象 | owner / 真源 | 不变量 |
|---|---|---|
| `GlobalSupervisorSession` | M3 RoleSession | global scope、只读默认、provider handle 非授权 |
| `ProjectSummary` | 各项目 projector | M6 只经 M5 `ProjectSummaryQueryPort` 按 RoleSession / scope / policy 读取；不可直接读项目 store / projection / root，不可反写 |
| `CrossProjectAdvisory` | Global Supervisor domain | `AdvisoryId`、Global RoleSession、ConsultHandoff、每个 ProjectSummary id/version/watermark、policy decision、generated_at；status 仅 advice lifecycle |
| `AdvisoryApplicationProjection` | M6 read model | 只引用各项目 owner 的 authoritative command / application receipt；不得改变 Advisory lifecycle 或拥有项目执行结果 |
| `StableMember` | Organization directory | stable `MemberId`、membership lifecycle、scope assignments、role assignments、contact binding。capability / permission 仅存 source+revision+observed_at 的只读 ref/projection，授权仍归 policy owner |
| `TemporaryAgent` | Execution history projection | stable `TemporaryAgentId`，绑定 execution identity、WorkItem / Attempt / Report / RoleSession；不会自动稳定化 |
| `Availability` | directory projection | source、observed_at、TTL；陈旧即 unknown，不作授权 |
| `ConsultHandoff` | M3 Handoff | from/to、scope、refs、question、receipt；无项目写权限 |

跨项目读取只允许 ProjectSummary，以及 ref 的 type、title、scrubbed summary、version、deep-link metadata；原始项目文件、transcript、secret 和未裁剪 memory 不进入 global scope。模型侧解引用必须再次经过项目 owner 的 policy gateway；用户点击 deep link 回源不等于把原文加入 global session。

## 4. 任务切片

### SYN-ORG-001 — 跨项目与成员合同冻结

冻结 ProjectSummary ACL / watermark、Advisory 精确 join、用户采纳、partial apply、stable `MemberId` / `TemporaryAgentId`、scope assignment、RoleSession join、同名冲突、人工晋升 `promoted_from`、availability TTL、contact receipt、retention 和 migration matrix。用户采纳只生成 DecisionRequest；每个项目随后以独立 command / grant 引用该决定。只写合同。

### SYN-ORG-002 — 只读跨项目 query 与 advisory

仅通过 M5 `ProjectSummaryQueryPort` 消费两个以上版本化 ProjectSummary；实现 freshness / missing / denied / degraded 状态、source links 与确定性冲突规则。advisory 与用户采纳本身零项目 mutation；各项目 owner 写 authoritative command / application receipt，M6 只投影 `applied / failed / rolled-back / unknown` 并引用 receipt。模型只增强解释，不可绕过 source / ACL。

### SYN-ORG-003 — 顶层 Global Supervisor RoleSession

接 M3 持久会话；默认只读 global scope；上下文只含最小 summaries 和 refs。入口与 Project Supervisor / Secretary 清楚分离。

### SYN-ORG-004 — Secretary consult Handoff

实现 Secretary 发起、Global Supervisor 接受 / 拒绝、返回 advisory、Secretary 展示 / 回源。重复 Handoff 幂等，意见不触发项目 command。

### SYN-ORG-005 — 稳定成员目录

实现 member identity、membership lifecycle、scope / role assignments、capability / permission 只读 refs、availability、session history、直接联系入口与停用。contact 只建立会话 / Handoff，不自动授予项目能力，目录永远不是授权真源。

### SYN-ORG-006 — 临时 agent 历史

从 M5 WorkItem / Attempt / Report / audit 投影临时 agent；可搜索任务、结果、失败和来源；UI 明确“临时”。人工晋升新建 / 绑定 StableMember 并保留 `promoted_from`，不修改原历史类型 / 来源。

### SYN-ORG-007 — 隔离 App 双项目验收

两个 scratch projects + fake roles 覆盖冲突发现、stale summary、ACL denied、consult、稳定 / 临时成员查找和直接联系。真实项目 summaries / 模型 / messages 另包。

## 5. 顺序、并行与写域

```text
ORG-001 → ORG-003 → ORG-002 → ORG-004
   ├────────────────→ ORG-005
   └────────────────→ ORG-006
ORG-002 + ORG-003 + ORG-004 + ORG-005 + ORG-006 → ORG-007
```

- Global Supervisor / organization domain 由 M6 单写；ProjectSummary 由 M5 owner，RoleSession / Handoff 由 M3 owner；
- frontend top-level entry、member directory 和 shared shell 必须单写者；DTO 稳定后可与 Rust consumer 并行；
- M7 可读取已批准的 source refs，但不能把 advisory 自动写成 PersonalFact / Memory；
- M8 adapter availability 可成为目录来源；CapabilityGrant 仍归 Policy / Grant domain，M8 只拥有 connector definition、account metadata 与 CredentialRef 边界，目录仅持只读 refs；
- M6 不改项目 aggregate、runner、grant 或 summary source，也不直读其 store / projection / root；需要应用建议时只能拆回逐项目、逐 owner、逐 Grant 的 M5 项目任务包；
- 公共 App assembly / navigation / snapshot 写面有未归属 WIP 即停。

## 6. 迁移与回滚

- 旧 single-project global review 先作为 legacy adapter / history，不直接升级为 cross-project advisory；
- existing Agent Center sessions 不自动迁成 StableMember；只有满足 explicit identity contract 的记录进入目录；
- TemporaryAgent 从 immutable execution refs 重建，不复制报告正文；
- summary version / watermark 变化会使 advisory 标记 stale，不静默重算并覆盖历史；
- rollback 回到旧项目内 review / Agent Center 显示，但不恢复跨项目 raw read；
- 目录可导出 / 重建，成员停用保留历史 refs；物理删除到 M9 后另批。

## 7. 验证矩阵

| 层级 | 必须证明 | 不能声称 |
|---|---|---|
| Contract / fixtures | ACL、watermark、member lifecycle、采纳边界 | service 已实现 |
| Unit / property | 无反写、stale/denied、temporary≠stable、contact 不扩权 | App 可用 |
| Temp integration | 两项目冲突、partial/missing summary、handoff 幂等 | 真实项目数据通过 |
| Non-test build | production path 可构建 | 桌面交互正确 |
| Isolated Tauri | 顶层入口、source link、成员查找 / 联系可见，并保留桌面窗口截图 / 交互 / deep-link 点击证据 | 真实模型 / 消息通过 |
| 经授权真实场景 | 指定 summary / provider / profile 的只读分析 | 跨项目写或发布通过 |

关键验收：从两个项目摘要发现冲突并回源；对项目 domain store、event/audit/outbox、sidecar / compatibility projection、文件和 spawn 设置 write-spy / hash baseline，M6 允许变化的只有自身 advisory / directory / audit owner；用户能找到并联系 stable member；temporary agent 不伪装成 stable；stale availability 不参与能力判定。

## 8. 授权与停止条件

local schema / store migration、App 启动 / 强制退出、真实项目 summary、真实 Global Supervisor 模型 / 消息、成员联系 provider 分别建包。跨项目统一写不属于 M6；用户采纳后只能拆成逐项目、逐 owner、逐 Grant 的 M5 项目任务包。M6 不授权真实 runner、connector credential、自动执行、Git 或发布。

立即停止：summary 缺 owner / watermark；需要 raw project root / transcript 才能分析；advisory 反写项目；StableMember 由 heuristic 推断；temporary 自动晋升；availability 被当 permission；用户采纳可绕过 per-project grant；UI-only ACL；dirty WIP 冲突；单项目 fixture 被表述成跨项目验收。

## 9. 阶段退出与 M7 输入

全部满足才完成 M6：

- 顶层 Global Supervisor RoleSession 和只读 cross-project query 成立；
- advisory 有 versioned source refs，项目无自动 mutation；
- advisory / ConsultHandoff / summaries / policy decision 精确 join；用户采纳只形成 DecisionRequest，逐项目应用拥有独立 grant 和 authoritative receipt，M6 的 `applied / failed / rolled-back / unknown` 仅是引用这些 receipts 的 projection；
- Secretary consult Handoff 可追踪 / 幂等 / 回源；
- stable / temporary member 类型、生命周期、availability 和 contact receipt 冻结；
- 双项目 isolated App 场景通过；真实数据证据单独结算；
- 旧 review / Agent Center 有 compatibility / export / rollback；
- 把 advisory、member 与 source-ref 事件合同交 M7，不自动成为记忆；
- CURRENT 回写完成 / HOLD / 下一步，M7 未激活不得续跑。
