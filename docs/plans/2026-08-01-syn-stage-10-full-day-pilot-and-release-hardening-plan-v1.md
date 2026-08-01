# Syn Stage 10：全日试点与发布硬化计划 v1

日期：2026-08-01<br>
阶段：`M10`<br>
状态：**PLANNED / NOT_ACTIVE / NO_EXECUTION_AUTHORITY。**<br>
上位计划：`2026-08-01-syn-personal-ai-workbench-master-development-plan-v1.md` M10。<br>
硬前置：M1-M9 exit receipts、retirement / HOLD manifest、可恢复 read models、隔离 profile 和发布候选配置均冻结。<br>
当前 active node / package：`NONE`；本计划不授权启动 App、真实数据 / provider / 项目写、签名、公证、发布、Git 或外部动作。

权威顺序：当前用户指令 → `../harness/AUTHORITY.md` / `../harness/CURRENT.md` → 当前 source / real-App evidence → master → M1-M9 exit / HOLD receipts → 本计划。任何历史 build、截图或 profile receipt 都按原证据层级使用。

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
- 真实 provider / account / connector、预算 / rate limit、外部写 action 是否保持关闭；
- release SLO / performance budget、错误预算、telemetry / privacy、crash report、support / recovery；
- 哪些 M9 legacy HOLD 可随 release 保留，哪些必须阻断发布；
- 用户可接受的长时间运行窗口和现场验收安排。

“全日”不是若干短测的集合。PIL-001 必须冻结：最短连续 wall-clock 时长、开始 / 结束与跨日边界、允许的计划内重启 / 中断次数和单次上限、静默窗口、scenario 最大间隔、最大人工补录比例、无事件观察时长、用户离开 / 返回规则。缺任一字段不得把分散收据合并成全日试点。

## 1. 阶段目标

1. 在真实 Tauri App 中完成一整天可回源、可恢复、权限诚实的核心工作线；
2. 对 cold start、warm resume、App crash、provider failure、DB / projector failure、connector disconnect、敏感输入做故障注入；
3. 验证 performance、pagination、index rebuild、backup / restore、migration rollback、long-running、cost / budget、observability 和 permission regression；
4. 建立 release configuration、versioning、bundle、签名 / notarization、artifact manifest、update / rollback、release notes 和 support runbook；
5. 高风险外部 action 默认关闭，任何试点例外逐动作 grant；
6. 将 synthetic、build、isolated App、真实 App、release candidate、published artifact 分层结算；
7. 只有所有阻断门通过且用户显式批准发布，才产生 release / publish 动作。

## 2. 全日试点剧本

试点必须按统一 correlation / source refs 记录，并至少覆盖：

1. 打开 Syn，Secretary 给出有来源、owner、reason 和 deep link 的当前情境；
2. 一条个人 / 外部事件进入，只形成 Inbox / Attention，不自动成 Task / Project / Memory；
3. 用户明确把复杂事项升级为项目；scope / owner / handoff 清楚；
4. Project Supervisor 持续对话，明确提出 Proposal，用户确认后获得 ExecutionGrant 并执行；
5. 结果经过 Report / Review / UserDecision 回到项目，Secretary 只收到 ProjectSummary；
6. Global Supervisor 对两个项目摘要发现冲突，意见不自动改项目；
7. DailyReport、Memory consolidation、PersonalFact / Assertion 与 SkillCandidate 分别结算，不越级；
8. App 重启后角色会话、未决、运行、attention、connector cursor 和 projector checkpoint 恢复；
9. connector disconnect、provider failure、DB busy/corrupt、projection degraded、敏感内容均 fail closed 且用户可见；
10. 用户找到一名 stable member 和一名 historical temporary agent，并可回源 / 联系；
11. 无事件窗口保持零模型调用；有事件时预算 / cost receipt 可见；
12. backup / restore 与当前 release rollback 在隔离副本实际演练。

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

验收 owner 与实现 owner 分离：实现线不能只用自己的测试收据宣布真实 App / release 通过。

## 5. 任务切片

### SYN-PIL-001 — Pilot / release contract freeze

冻结 profile、场景、truth source、evidence schema、SLO、privacy、budget、failure matrix、release target、artifact / signing / updater / rollback、go/no-go，以及“全日”的最短持续时长、中断 / 重启上限、跨日边界、静默窗、scenario 间隔和人工补录上限。只写文档与只读 inventory。

### SYN-PIL-002 — Release-candidate reproducibility

固定版本、source / lockfile / toolchain / config / dependency manifest；release build 只能从独立干净 checkout 或等价不可变 source 生成，固定 HEAD 与 lockfile/toolchain/config hashes 必须与 artifact receipt 一对一绑定，并记录 opening/final source hash。建立 clean reproducible build 过程和 artifact checksums。需要 worktree、依赖下载、build 或 Git 时分别授权。

### SYN-PIL-003 — 隔离 smoke 与权限回归

在 isolated profile 运行 M1-M9 核心 allow / deny、secret scan、migration dry-run、projector rebuild、connector mock、zero-event 模型计数。只证明候选前置。

### SYN-PIL-004 — 真实 App 全日 pilot

按 §2 逐场景执行，使用获批 profile / data / provider / project；每一步直接观察 UI、source、receipt、App restart 和结果 readback。外部写 action 默认关闭。

### SYN-PIL-005 — 故障、恢复与长运行

独立注入 provider / connector / DB / projector / process / network / disk / permission / receipt failures；覆盖 backup / restore、migration rollback、stale / retry、long-running 与 resource budgets。

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
- dirty WIP、build source 或 artifact provenance 不清；
- freeze 后修复未使受影响 receipts / artifact hash 失效，或未从最早受影响切片重新执行；
- 发布、push、tag、签名或外部系统写入缺用户明确授权。

## 9. 回滚与事故边界

- pilot 前备份 exact data roots、schemas、permissions 与 hashes；恢复只在隔离副本预演后使用；
- App / schema / projector / connector / provider 各自有独立 kill switch / rollback，不通过放宽 policy 恢复；
- upgrade / downgrade 兼容范围写进 release notes；不兼容时先导出 / 停止，不强行启动；
- external effect 一律有 effect id、result receipt 和人工核对；未决动作不因 App restart 自动重发；
- incident receipt 只存脱敏摘要 / refs / hashes；secret 与 raw user content 不进入普通诊断；
- rollback 后核对会话、attention、项目 facts、memory、cursor、outbox 和 projector，残留不明即继续 HOLD。

## 10. 阶段完成定义

M10 只有同时满足以下条件才可称“阶段完成”：

- §2 全日真实 App 场景逐项 `ACCEPTED`，或非阻断项有用户明确接受的 bounded HOLD；
- 关键故障与 long-running / budget / permission regression 通过；
- backup / restore、migration / updater rollback、projector rebuild 实际演练；
- release candidate 可复现、可追源、可安装、可更新、可回退；
- high-risk external action 默认关闭，secret boundary 通过；
- M9 legacy / compatibility 清单真实一致；
- 独立验收给出 go / no-go，用户另行明确是否执行发布；
- CURRENT、AUTHORITY、release notes、support / rollback runbook 只记录实际事实；
- 未获发布授权时安全停在“release candidate accepted / not published”，不得自动 tag / push / publish。
