# Syn Stage 11：受治理自升级平台计划 v1

日期：2026-08-16<br>
阶段：`M11`<br>
状态：**PLANNED / NOT_ACTIVE / NO_EXECUTION_AUTHORITY。**<br>
上位计划：`2026-08-01-syn-personal-ai-workbench-master-development-plan-v1.md` M11。<br>
硬前置：M7 候选 / Skill 治理、M9 版本退役与恢复、M10 可复现候选 / 更新 / 回滚和独立验收门。M5 的 `AgentRuntimeAdapter`、执行轨迹与外部副作用语义必须先冻结。<br>
当前活动阶段 / 叶：本文件只是长期阶段合同；不激活产品实现，不授权模型、插件、worktree、构建、Git、真实 Provider、外部动作、发布或自动升级。

本阶段落实用户早于 DeepSeek Harness 已提出的自升级方向。DeepSeek Harness / Cordis 只提供插件接缝、运行事件、动态候选与可逆注册的工程参考；Syn 不依赖它们才能完成本阶段，也不把它们放在治理根。

## 0. 事实、判断与未知

### 已确认方向

- Syn 的长期目标是用户只做最高层目标、资源、风险与例外决定，平台持续管理、执行、检查、沉淀并改进；
- 自升级要覆盖 Skill、Prompt、Profile、工具 / 插件包、Provider 适配和 Syn 代码，但不同对象使用不同晋升门；
- M1–M4 的具名范围不因新参考重开；冻结 v1 合同不倒改，后继合同版本化新增；
- 生成候选、验收候选和批准晋升必须职责分离；
- 身份、权限、审计、预算、Verifier、Updater 信任链与 Kill Switch 不能成为普通自升级对象。

### 仍未知

- 首个真实自升级样本选 Provider 兼容修复、确定性性能优化、Skill / Profile 还是有复现测试的 Bug；
- 首版签名、包格式、隔离 backend、canary 范围和 updater 的实际实现；
- 哪些低风险对象可在用户预先签署的政策内自动晋升；
- 对主观体验、新功能和开放式目标使用什么人类验收界面；
- DSH 或其他外部 runtime 是否在实测后比 Syn 原生 workcell 更值得接入。

## 1. 阶段目标

1. 建立统一但不抹平对象差异的自升级候选模型；
2. 从真实反馈、故障、成本、版本变化和重复模式生成有来源候选；
3. 在隔离 worktree / workcell 中构建候选，不改当前运行实例；
4. 对来源、依赖、能力、权限、数据、网络和成本做静态差异分析；
5. 通过确定性测试、历史回放、反例集与独立 Verifier 比较旧版和候选；
6. 使用签名、版本 pin、canary、监控、撤回和回滚控制晋升；
7. 让用户把反复决策编译成可撤销政策，而不是永久逐次批准低风险动作；
8. 确保任何失败都不会让候选自行降低验收标准、扩大权限或成为新的治理根。

## 2. 本阶段不做

- 不在线覆盖当前进程、当前二进制、当前数据库或当前正式配置；
- 不允许候选生成者同时签署 Verifier 通过和 PromotionDecision；
- 不让模型改测试、删断言、降低覆盖或改隐藏 oracle 后再声称通过；
- 不自动修改 Identity / Scope / Policy / ExecutionGrant / Credential / Audit / Budget / Verifier / Updater / Kill Switch；
- 不把一次成功、模型自评、benchmark 单分数或测试全绿直接当“变得更好”；
- 不把插件卸载、Git revert 或代码回滚冒充邮件、付款、发布、合同等现实副作用回滚；
- 不做模型权重递归自训，不让代码进化循环跨代积累不可审计的隐藏状态；
- 不默认自动 merge、commit、tag、push、签名、发布或连接真实账号；
- 不因 DSH 存在而安装 DSH、启用动态 Cordis package 或引入 npm 插件供应链。

## 3. 对象、owner 与状态机

| 对象 | owner | 关键边界 |
|---|---|---|
| `ImprovementSignal` | 来源 domain | 事实、失败、成本或版本变化；不是改动指令 |
| `UpgradeCandidate` | upgrade registry | kind、目标版本、来源、目标、适用范围、风险、旧版基线 |
| `CandidateArtifact` | isolated builder | 内容 hash、依赖 lock、toolchain、构建环境、SBOM / manifest |
| `CapabilityDiff` | policy / security reviewer | 新增 / 删除 capability、scope、secret、network、process、data effect |
| `EvaluationPlan` | acceptance owner | 隐藏 oracle、回放集、反例、预算、比较规则和停止条件 |
| `EvaluationReceipt` | independent verifier | 旧版 / 候选结果、失败分布、成本、未覆盖项；候选不能写 |
| `CanaryAssignment` | rollout owner | exact scope、时间、用户、项目、runtime profile、kill switch |
| `PromotionDecision` | 用户或用户签署的政策 | exact artifact hash、范围、policy version、有效期、回滚锚点 |
| `UpgradeRelease` | updater / registry | 已签名版本、激活范围、观察状态、撤回状态 |
| `RollbackReceipt` | recovery owner | 触发原因、恢复版本、事实核对、残留和外部补偿 |

候选状态机：

```text
observed
  -> drafted
  -> static_checked
  -> isolated_built
  -> evaluated
  -> canary_ready
  -> canary_running
  -> promotion_pending
  -> promoted
  -> monitored
  -> retired
```

任一阶段可进入 `rejected / quarantined / rolled_back / outcome_unknown`。状态推进只由对象 owner 根据独立 receipt 执行，不能由 Agent Runtime final answer 自报。

## 4. 三条自升级轨道

### A. 行为与能力包

对象：Prompt、Skill、Profile、确定性脚本、工具 / 插件包。

- 必须有来源、版本、package digest、依赖、capability manifest、sandbox 要求、适用范围和撤回状态；
- 低风险、无外部副作用、权限不增加的候选可在用户签署政策内自动 canary / 晋升；
- 动态 runtime package 只落成 `UpgradeCandidate`，进程内可用不等于已安装。

### B. Runtime 与 Provider 兼容

对象：Codex / 模型版本适配、`AgentRuntimeAdapter`、Tool Pipeline、Session / Trace 兼容。

- 固定旧版 / 新版、Profile digest、模型与工具矩阵；
- 必须通过 conformance suite、重启恢复、权限拒绝、预算和不重复 effect；
- 外部 DSH 只作为一个 adapter 候选，与 Syn 原生 runtime 使用相同合同和证据门。

### C. Syn 代码

对象：有失败复现的 Bug、保行为重构、性能 / 成本优化和用户提出的新功能。

- 有确定性 verifier 的 Bug、兼容和性能优化可自动生成候选；
- 主观体验、新功能和产品取舍只能停在方案与可体验候选，由用户决定是否符合意图；
- 核心代码候选默认需要用户签署 PromotionDecision；
- 治理根相关代码永不进入普通自动晋升政策。

## 5. 任务切片

### SYN-EVO-001 — 自升级宪法与对象合同

冻结三轨对象、owner、状态机、L0–L4 决策权、不可委托清单、职责分离、证据等级和 PromotionDecision。只写版本化合同，不运行候选。

### SYN-EVO-002 — Improvement Signal 与候选登记

接收具名来源的 failure、cost、version change、用户纠正和重复模式；去重、归因、标敏感与 scope，只生成候选，不自动建 worktree。

### SYN-EVO-003 — 隔离构建工作单元

建立 exact source / lock / toolchain / config 的可复现 worktree / workcell；候选默认无用户凭据、无生产数据、无真实 connector write。输入输出带 hash 和 provenance。

### SYN-EVO-004 — 供应链、权限与能力差异

比较 package、install / prepare script、文件、网络、进程、CredentialRef、Tool、Connector 与外发范围。任何治理根或未知能力变化直接升级 L4 / quarantine。

### SYN-EVO-005 — 独立评测与防钻空子

建立旧版对照、隐藏 oracle、历史回放、ImpossibleBench 式金丝雀、断言 / 覆盖缩水检测、成本和性能预算。Verifier 与候选生成 workcell 分进程、分写面、分权限。

### SYN-EVO-006 — Canary、签名晋升与监控

按 exact artifact hash、scope、比例、时限和 kill switch 灰度；用户或已签署政策产生 PromotionDecision；运行异常自动停用候选，但不能自行放宽门槛。

### SYN-EVO-007 — Skill / Profile 首个闭环

选择无外部副作用的低风险候选，完成 signal → candidate → evaluation → canary → promotion → rollback 全链。只证明该对象和政策。

### SYN-EVO-008 — Runtime / Provider 兼容闭环

选择一个版本适配样本，让 Syn 原生 runtime 与一个可选外部 adapter 运行同一 conformance suite；不以“接入成功”替代权限、恢复、成本和结果验收。

### SYN-EVO-009 — 有确定性 Verifier 的代码候选

选择有失败复现或机器可判指标的窄改动，生成隔离候选和完整证据；默认停在用户签署晋升门，不自动 merge / publish。

### SYN-EVO-010 — 回滚、灾难恢复与独立收口

演练候选 build 失败、Verifier 异常、canary 回归、updater 中断、启动失败和 `OUTCOME_UNKNOWN`；核对事实、审计、配置、包、运行会话和未决外部动作。

## 6. 顺序、并行与写所有权

```text
EVO-001 -> EVO-002 -> EVO-003 -> EVO-004 -> EVO-005 -> EVO-006
                                                     |-> EVO-007
                                                     |-> EVO-008
                                                     `-> EVO-009
EVO-007 + EVO-008 + EVO-009 -> EVO-010
```

- upgrade registry、policy/security review、isolated builder、verifier、rollout/updater 分 owner；
- 同一候选同一时刻只有一个构建 writer、一个事实状态 owner；
- oracle、测试基线和晋升策略不能由候选 writer 修改；
- release / updater、生产配置和真实数据根保持单写；
- 发现未归属 dirty WIP、source hash 漂移、现有同名候选或证据冲突立即停止。

## 7. 验证矩阵

| 层级 | 必须证明 | 不能声称 |
|---|---|---|
| Contract / static | 状态、owner、权限差异和职责分离 | 候选有效 |
| Isolated build | 可复现 artifact、依赖与来源 | 比旧版更好 |
| Replay / verifier | 具名场景、反例、预算内不回归 | 主观体验正确 |
| Canary | exact scope / 时间内真实行为 | 可全局发布 |
| Signed promotion | exact artifact / policy / rollback 获准 | 永久安全 |
| Monitored rollout | 观察窗和回滚门成立 | 未来版本自动通过 |

验收至少核对：候选无法读写隐藏 oracle；子 agent 不能扩大 grant；runtime 崩溃不重复外部动作；compaction / resume 不改正式事实；动态插件默认关闭；成本、权限与供应链差异可见；旧版可恢复。

## 8. 回滚与现实副作用

- 代码、Profile、Skill 和插件回滚使用版本 pin、签名 manifest 与保留旧 artifact；
- schema 变化使用 additive migration、before / after、export、restore 和兼容窗；
- 外部动作使用 `ActionIntent -> durable outbox -> execute -> source readback -> reconcile / compensate -> receipt`；
- 已发送邮件、已提交付款或已作承诺只能补偿、撤回或人工处理，不能声称字节回滚恢复现实；
- 结果 receipt 丢失时进入 `OUTCOME_UNKNOWN`，按 provider 幂等键和来源回读判定，不盲目重试；
- 回滚后核对正式事实、RoleSession、runtime session refs、outbox、connector cursor、memory / skill version 和审计。

## 9. 激活、授权与停止条件

M11 只有在用户明确激活对应薄切片、创建匹配阶段 / leaf 后施工。以下分别需要独立授权：创建 worktree、运行模型、安装 package、下载依赖、构建、真实 Provider、真实账号、connector action、签名、Git 写入、canary、发布和删除。

立即停止：

- 候选触碰治理根而没有 L4 用户决定；
- 生成者与 verifier / promoter 无法分离；
- oracle 可被候选读取、修改或推断答案；
- 只能靠模型自评、单 benchmark 或测试全绿判“变好”；
- 权限 / 依赖 / 数据 / 网络差异不完整；
- artifact 不可复现或来源 hash 漂移；
- 外部 effect 无幂等、回读、补偿或人工接管；
- updater / rollback 未演练；
- 需要覆盖 dirty WIP、降低验收门或扩大授权。

## 10. 阶段完成定义

M11 只有同时满足以下条件才可称完成：

- 三轨对象、状态机、职责分离和不可委托治理根在产品 App 与后端一致；
- 至少一个低风险 Skill / Profile、一个 Runtime / Provider 兼容样本和一个有确定性 verifier 的代码候选走完全链；
- 所有候选都有 provenance、artifact hash、capability diff、EvaluationReceipt、PromotionDecision 和 rollback anchor；
- 动态扩展默认关闭，启用范围、签名、版本和撤回可见；
- 低风险自动晋升只来自用户签署、可撤销、有限范围的政策；
- 核心代码与治理根仍不能自行批准；
- runtime 崩溃、重复 effect、receipt 丢失、canary 回归和 updater 中断恢复通过；
- 用户可以看见为什么建议升级、改变了什么、依据是什么、风险和权限是否变化，以及如何撤回；
- DSH 是否接入由同一 conformance 与成本证据决定，不成为架构前提；
- `docs/current-state.md` 只回写真实完成项；未实现、未 canary、未发布部分保持明确 HOLD。
