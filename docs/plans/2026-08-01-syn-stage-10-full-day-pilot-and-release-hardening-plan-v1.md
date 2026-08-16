# Syn Stage 10：全日试点与发布硬化计划 v1

日期：2026-08-01<br>
阶段：`M10`<br>
状态：**PLANNED / NOT_ACTIVE / NO_EXECUTION_AUTHORITY。**<br>
上位计划：`2026-08-01-syn-personal-ai-workbench-master-development-plan-v1.md` M10。<br>
硬前置：M1-M9 exit receipts、retirement / HOLD manifest、可恢复 read models、隔离 profile 和发布候选配置均冻结。<br>
当前路线状态：M1–M4 已完成各自具名范围，M5–M10 未激活。Harness 动态 stage / leaf 另看 `../harness/plan.md`；本计划不授权启动桌面应用、DSH、真实数据 / 服务提供方 / 项目写、签名、公证、发布、Git 或外部动作。

权威顺序：当前用户指令 → `../../../AGENTS.md` → `../../AGENTS.md` → `../harness/plan.md` → 活动阶段（stage）/ 唯一活动叶（leaf）→ `../harness/authorization.json` → `../product/syn-product-canon-v1.md` 与 `../product/knowledge-infrastructure-canon-v1.md` → `../current-state.md` → 当前源码 / 真实桌面应用证据 → master → M1-M9 退出 / 暂缓回执 → 本计划。任何历史构建、截图或配置回执都按原证据层级使用。

## 0. 当前事实与未知

### 已有局部素材

- 仓库有 isolated profile preflight 脚本，可创建受限 root、synthetic fixture 并输出进程 / UI observation receipt；
- 各历史任务积累了 static、unit、offline、debug build、隔离启动和少量真实 App 观察证据；
- M1-M9 计划定义了 security、transaction、session、attention、project execution、global supervision、memory、connector、read model 的目标验收。

### 当前证据上限

- 2026-07-27 Home-only 实跑停在 UI target discovery，明确不是完整真实 App 验收；
- 2026-07-31 shared conversation 真实首句最终 lifecycle failed，binding / thread id 存在但未证明 Active、工具列表或成功工具调用；
- 当前 Tauri `bundle.active` 为 false，版本仍为 `0.1.0`；静态清单未见可据以宣称已完成的 CI、签名 / notarization、updater、DMG / appcast 或 publish pipeline；
- debug build、isolated process receipt、fixture screenshot 均不等于全日使用、发布候选或发布成功；
- 当前 active-id 仍为 NONE，App / real store / message / workflow / connector / Git 都未获授权。

### HOLD / 需冻结决定

- 目标发布平台 / 架构、最低系统版本、签名身份、公证、分发渠道、updater、rollback / downgrade；
- pilot profile、真实数据范围、备份、保留、红线与销毁；
- 用户个人服务器作为异地备份目标时的地址、协议、认证、加密、频率和恢复演练；个人服务器尚未进入时，不阻挡本地导出、备份与恢复合同验收；
- 成本统计优先接入的开源工具、数据可信度、采样范围和用户可见指标；
- 真实 provider / account / connector、预算 / rate limit、外部写 action 是否保持关闭；
- release SLO / performance budget、错误预算、telemetry / privacy、crash report、support / recovery；
- 哪些 M9 legacy HOLD 可随 release 保留，哪些必须阻断发布；
- 用户可接受的长时间运行窗口和现场验收安排。
- 首个生产候选使用 Syn 原生 runtime 还是外部 adapter、固定版本 / package digest、动态扩展政策和 conformance 基线。
- sandbox 在目标平台的完整 / 部分 / 不可用状态，以及网络、进程、凭据和插件供应链的独立隔离措施。

“全日”不是若干短测的集合。PIL-001 必须冻结：最短连续 wall-clock 时长、开始 / 结束与跨日边界、允许的计划内重启 / 中断次数和单次上限、静默窗口、scenario 最大间隔、最大人工补录比例、无事件观察时长、用户离开 / 返回规则。缺任一字段不得把分散收据合并成全日试点。

## 1. 阶段目标

1. 在真实 Tauri App 中完成一整天可回源、可恢复、权限诚实的核心工作线；
2. 对 cold start、warm resume、App crash、provider failure、DB / projector failure、connector disconnect、敏感输入做故障注入；
3. 验证 performance、pagination、index rebuild、backup / restore、migration rollback、long-running、cost / budget、observability 和 permission regression；成本统计优先复用成熟开源工具，不预设自研复杂账房；
4. 建立 release configuration、versioning、bundle、签名 / notarization、artifact manifest、update / rollback、release notes 和 support runbook；
5. 高风险外部 action 默认关闭，任何试点例外逐动作 grant；
6. 将 synthetic、build、isolated App、真实 App、release candidate、published artifact 分层结算；
7. 只有所有阻断门通过且用户显式批准发布，才产生 release / publish 动作。
8. 验证 Agent Runtime 只是可销毁执行工作单元：中途崩溃、session resume / fork / compaction、child run 和 adapter 替换不改变 RoleSession、正式事实、权限、记忆或未完成 operation。
9. 同时治理钱、token、时间、模型、工具、外部调用和算力，不把上下文 token pressure 当完整成本预算。

## 2. 全日试点剧本

试点必须按统一 correlation / source refs 记录，并至少覆盖：

1. 打开 Syn，秘书给出有来源、owner、reason 和 deep link 的当前情境；
2. 用户在前端明确编辑或纠正一项内容，变更写回唯一真源；重启后原角色和另一获准角色都读到新版，旧缓存不能反向覆盖；
3. 一条个人 / 外部事件进入，只形成收件与关注，不自动成为任务、项目或正式记忆；
4. 用户明确把复杂事项升级为项目；作用域、所有者和交接清楚；
5. 项目主管持续对话，明确提出方案，用户确认后获得执行授权并执行；
6. 结果经过汇报、复核和用户决定回到项目，秘书只收到可回源的项目摘要；
7. 全局主管对两个项目摘要发现冲突；重大问题可组织独立多视角咨询，意见不自动改项目；
8. 日报、每日记忆整理、个人事实 / 模型推断与技能候选分别结算，不越级；
9. 常驻角色带来源恢复“用户是谁、近期做了什么、长期做过什么、当前有哪些未闭环事项”，用户知识深度可以纠正；
10. 稳定角色和临时智能体按不同权限取得正确资料与技能说明，未获准技能仍不可执行；
11. App 重启后角色会话、未决、运行、关注、连接器游标和投影检查点恢复；
12. 连接器断开、服务提供方失败、数据库忙碌 / 损坏、投影降级和敏感内容均诚实关闭且用户可见；
13. 用户找到一名稳定成员和一名历史临时智能体，并可回源和联系；
14. 无事件窗口保持零模型调用；有事件时预算与成本回执可见；
15. 本地导出、备份 / 恢复与当前发布回退在隔离副本实际演练；个人服务器异地备份在该基础上另包接入。
16. runtime 在 tool / connector action 中途被强制终止；Syn 以 effect id 和来源回读恢复，结果未知时保持 `OUTCOME_UNKNOWN`，不得重复外部副作用。
17. runtime session resume / fork / compaction 后，模型上下文 trace 可重建；RoleSession、公司事实和正式记忆不因摘要或 runtime handle 变化而漂移。
18. child agent 无法扩大 parent grant、scope、Tool、Connector 或 CredentialRef；执行者 final answer / goal complete 仍需独立验收。

任一场景缺少真实 truth source、直接观察、receipt 或 recovery，只能记录 `NOT_ACCEPTED / HOLD`，不能用邻近测试补齐。

## 3. 本阶段不做

- 不把试点当作放开所有真实账号、外部写、资金、生产项目或无人值守执行的总授权；
- 不在 release candidate 中顺带开发大功能、重写 UI 或迁移新 domain；
- 不用 synthetic / unit / build / screenshot 代替真实 App 操作与 source readback；
- 不用 debug build 冒充签名 artifact，不用本机 launch 冒充分发 / updater 成功；
- 不在没有 backup / rollback / exact target 时删旧数据、worktree 或 release artifact；
- 不隐藏 degraded / HOLD / legacy path，也不把 disabled feature 宣称完成；
- 不自动上传日志、transcript、project data、memory 或 secret；
- 不在未获用户明确批准时签名、公证、发布、push、tag 或创建 release。
- 不在生产 Profile 默认开启动态 package / runtime 自改；启用候选必须有来源、digest、能力清单、签名、canary 和回滚。
- 不把 filesystem sandbox、Approval 弹窗或 worker 自报完成单独当成生产安全 / 业务验收。

## 4. 试点与发布对象 owner

| 对象 | owner | 必须冻结 |
|---|---|---|
| `PilotProfile` | pilot owner | exact app/data/provider/project/permission/budget/time window |
| `ScenarioReceipt` | acceptance owner | start/end、build hash、profile、steps、truth refs、expected/actual、status |
| `FailureInjectionReceipt` | reliability owner | injection point、blast radius、recovery、residual state |
| `PerformanceBudget` | platform owner | startup、query、interaction、memory、CPU、disk、model cost / rate |
| `ReleaseCandidate` | release owner | version、source / dependency hashes、config、features/HOLD、artifacts、SBOM/manifest |
| `ReleaseArtifact` | release channel | signature/notarization/checksum/install/update/rollback evidence |
| `Privacy/TelemetryPolicy` | user / policy owner | fields、redaction、retention、opt-in、export/delete |
| `GoNoGoDecision` | 用户 | blockers、accepted risks、HOLD、exact publish action |
| `RuntimeConformanceReceipt` | independent acceptance owner | runtime/profile/package digest、kill/recovery、trace、child grant、sandbox enforcement、cost、duplicate effect 与 unresolved |
| `UpgradeBaseline` | M11 input owner | 可复现 RC、测试 / 回放集、预算、权限基线、updater / rollback anchor；不激活自升级 |

验收 owner 与实现 owner 分离：实现线不能只用自己的测试收据宣布真实 App / release 通过。

## 5. 任务切片

### SYN-PIL-001 — Pilot / release contract freeze

冻结 profile、场景、truth source、evidence schema、SLO、privacy、budget、failure matrix、release target、artifact / signing / updater / rollback、go/no-go，以及“全日”的最短持续时长、中断 / 重启上限、跨日边界、静默窗、scenario 间隔和人工补录上限。只写文档与只读 inventory。

### SYN-PIL-002 — Release-candidate reproducibility

固定版本、source / lockfile / toolchain / config / dependency manifest；release build 只能从独立干净 checkout 或等价不可变 source 生成，固定 HEAD 与 lockfile/toolchain/config hashes 必须与 artifact receipt 一对一绑定，并记录 opening/final source hash。建立 clean reproducible build 过程和 artifact checksums。需要 worktree、依赖下载、build 或 Git 时分别授权。

### SYN-PIL-003 — 隔离 smoke 与权限回归

在 isolated profile 运行 M1-M9 核心 allow / deny、secret scan、migration dry-run、projector rebuild、connector mock、zero-event 模型计数，并冻结 runtime / profile / package digest。覆盖 child grant、动态 extension 默认关闭、sandbox enforcement status 与 trace / fact / memory 分层。只证明候选前置。

### SYN-PIL-004 — 真实 App 全日 pilot

按 §2 逐场景执行，使用获批 profile / data / provider / project；每一步直接观察 UI、source、receipt、App restart 和结果 readback。外部写 action 默认关闭。

### SYN-PIL-005 — 故障、恢复与长运行

独立注入 provider / connector / DB / projector / runtime process / network / disk / permission / receipt failures；在 Tool / Action 中途 kill runtime，覆盖 resume / fork / compaction、child grant 不扩权、duplicate effect、`OUTCOME_UNKNOWN`、sandbox degraded、本地导出、backup / restore、migration rollback、stale / retry、long-running 与钱 / token / time / tool / external-call budgets。个人服务器只在用户已搭建并单独批准对应目标、凭据和网络后作为异地备份适配器加入，不成为本地恢复能力的前置。

### SYN-PIL-006 — 打包、安装、更新与回退候选

在 PIL-005 通过后，才可把 artifact 称为 `ReleaseCandidate` 并在目标平台验证签名 / notarization、checksum、fresh install、upgrade、downgrade / rollback、data compatibility、uninstall residuals。PIL-005 前可以生成 draft artifact 做机械预检，但不得作为 RC 或 Go/No-Go 输入。每项真实系统动作单独授权。

### SYN-PIL-007 — 独立验收与 Go / No-Go

由不承担同一实现写面的验收线核对全日 receipts、HOLD、legacy manifest、privacy、recovery 和 artifacts。输出事实化 go / no-go 建议；发布动作仍等待用户明确指令。

### SYN-PIL-008 — 发布与发布后观察候选

默认未激活。只有用户指定 exact version / channel / artifact 后才可执行 publish / tag / push / rollout；定义 observation window、rollback trigger、support / incident receipt 和停止条件。

## 6. 顺序、并行与写所有权

```text
PIL-001 → PIL-002 → PIL-003 → PIL-004 → PIL-005 → PIL-006 → PIL-007 → PIL-008（需用户发布授权）
```

- release freeze 后除 blocker fix 外停止 feature development；blocker fix 回到对应 stage / bounded package，不直接塞进 pilot 包；任何 freeze 后修复都使受影响的 scenario receipts 与 artifact hash 失效，必须从最早受影响切片重新 freeze / build / 验收后才能进入 PIL-007；
- build / signing / distribution、pilot runtime、acceptance 三条线分 owner；
- 真实 App 同一 profile / store 单写者，禁止并发 worker 修改试点事实；
- fault injection 不与真实用户重要数据混用；优先隔离副本，exact blast radius 预先核验；
- Git tag / push / release / deploy 和签名凭据各有独立授权；
- 任一公共文件或 artifact 与 opening hash 不符即停止重新 freeze。

## 7. 证据分层与通过规则

| 层级 | 证明什么 | 不证明什么 |
|---|---|---|
| Contract / static | 场景、边界、配置和引用一致 | binary 可运行 |
| Unit / integration / build | 代码、fixture、artifact 生成性质 | 用户真实桌面流程 |
| Isolated Tauri | 本机隔离 profile 的真实 App 行为 | 真实数据 / provider / 全日使用 |
| Real App pilot | 获批 profile / data / provider 的精确场景 | 签名分发 / 发布成功 |
| Installed RC | 目标系统安装 / 更新 / 回退 | 已发布给用户群 |
| Published release | exact channel / artifact 可获取且观察窗通过 | 未来版本或其他平台 |

每条 scenario receipt 必须含：source revision、app/build hash、profile、时间、用户动作、actual result、readback、failure / recovery、screenshots / logs 的脱敏引用、证据层级和未证明项。

Runtime 相关 receipt 还必须含：adapter/version/package digest、sandbox enforcement status、grant/policy version、runtime session / trace hash、child refs、tool/effect refs、各类预算、unresolved 和独立 verifier。Harness final answer、goal complete 或 worker report 不能替代这些字段。

## 8. Go / No-Go 阻断门

出现任一项即 `NO-GO / HOLD`：

- M1-M9 有未解释 owner / scope / data-loss / secret / duplicate-effect blocker；
- 真实 App 关键场景依赖 synthetic 替代；
- conversation terminal failure、项目执行、restart recovery 或 source deep link 未通过；
- backup / restore、migration rollback、projector rebuild 或 updater rollback 未实际演练；
- high-risk external action 默认开放或 credential 出现在普通日志 / store；
- performance / cost 超预算且无用户接受；
- legacy path 不知是否仍可写，或 retirement HOLD 未明确风险；
- release artifact 不可复现、未签名 / 未公证（若渠道要求）、checksum / install / update 不一致；
- telemetry / privacy 未冻结；
- runtime 中途崩溃会丢公司 operation，或未知外部结果只能靠盲重试；
- session resume / fork / compaction 会改变 RoleSession、正式事实或正式记忆；
- child agent 能扩大 grant / scope / tool / connector / credential；
- 动态插件默认开启，package 来源 / digest / capability / 签名 / 回滚不明；
- sandbox 部分生效或不可用却未进入可见 `DEGRADED / HOLD`；
- 验收者与执行 workcell 未分离，或把 runtime 自报完成当业务通过；
- 只有 token / round 限制，没有钱、时间、工具、外部调用与异常消费熔断；
- dirty WIP、build source 或 artifact provenance 不清；
- freeze 后修复未使受影响 receipts / artifact hash 失效，或未从最早受影响切片重新执行；
- 发布、push、tag、签名或外部系统写入缺用户明确授权。

## 9. 回滚与事故边界

- pilot 前备份 exact data roots、schemas、permissions 与 hashes；恢复只在隔离副本预演后使用；
- App / schema / projector / connector / provider 各自有独立 kill switch / rollback，不通过放宽 policy 恢复；
- upgrade / downgrade 兼容范围写进 release notes；不兼容时先导出 / 停止，不强行启动；
- external effect 一律有 effect id、result receipt 和人工核对；未决动作不因 App restart 自动重发；
- runtime / plugin 卸载只回退内部注册；现实副作用仍按来源回读与补偿处理；
- incident receipt 只存脱敏摘要 / refs / hashes；secret 与 raw user content 不进入普通诊断；
- rollback 后核对会话、attention、项目 facts、memory、cursor、outbox 和 projector，残留不明即继续 HOLD。

## 10. 阶段完成定义

M10 只有同时满足以下条件才可称“阶段完成”：

- §2 全日真实 App 场景逐项 `ACCEPTED`，或非阻断项有用户明确接受的 bounded HOLD；
- 关键故障与 long-running / budget / permission regression 通过；
- 前端明确修改已写回唯一真源，角色重启后读到新版；
- 常驻角色恢复用户身份、近期 / 长期工作和未闭环事项，知识深度可纠正；不同角色按权限获得资料与技能，未授权技能仍关闭；
- backup / restore、migration / updater rollback、projector rebuild 实际演练；
- release candidate 可复现、可追源、可安装、可更新、可回退；
- high-risk external action 默认关闭，secret boundary 通过；
- M9 legacy / compatibility 清单真实一致；
- runtime conformance、崩溃恢复、trace 分层、child grant、供应链、sandbox degraded 和多维预算门通过；
- 形成可供 M11 使用的可复现 UpgradeBaseline、独立验收集与 updater / rollback anchor，但不因此激活自升级；
- 独立验收给出 go / no-go，用户另行明确是否执行发布；
- 产品权威登记、`../current-state.md`、发布说明以及支持与回滚手册只记录实际事实；
- 未获发布授权时安全停在“release candidate accepted / not published”，不得自动 tag / push / publish。
