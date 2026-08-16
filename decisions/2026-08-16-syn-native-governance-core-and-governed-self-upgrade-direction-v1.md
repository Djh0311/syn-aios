# 决定：Syn 原生治理核心与受治理自升级方向 v1

日期：2026-08-16
状态：**当前有效。**

## 1. 用户本轮确认

用户在讨论 DeepSeek Harness（DSH）与个人 AI 平台后明确确认：

1. 长期目标是一个 AI 原生个人平台 / 一人公司操作系统：用户只做最高层目标、资源、风险和例外决策，其余管理与执行尽量交给 AI；
2. DSH 的插件化、事件日志、工具管线和动态扩展理念与目标接近，但 Syn 不必把 DSH 当核心依赖；
3. Syn 更适合借鉴 DSH 的实现方法，自己实现核心，从一开始就保留平台自升级能力；
4. 既有自进化研究早于 DSH 出现，DSH 是新的工程参照和交叉验证，不是 Syn 自升级方向的来源。

## 2. 本次决定

### 2.1 Syn 自己拥有不可外包的治理根

下列能力由 Syn 原生控制核心持有，不成为普通插件，也不交给任一 Agent Runtime：

- 用户与角色身份、作用域和对象所有权；
- Policy、ExecutionGrant、权限扩大、撤销与职责分离；
- 正式事实、事件、审计、长期记忆晋级与冲突治理；
- 预算上限、外部动作边界、CredentialRef 与 ConnectorGateway；
- 完成判定、独立 Verifier、发布信任链、停机与回滚权。

模型、插件、工具、工作流或 runtime 的自述不能改变上述权威。

### 2.2 默认核心原生实现，外部 Runtime 可替换

Syn 原生实现 vendor-neutral 的 `AgentRuntime` / `AgentWorkcell` 合同。DeepSeek Harness、Codex 或其他 runtime 可以在合同下作为适配器接入，但不成为 Syn 的身份、事实、权限、记忆或运营真源。

这不是拒绝使用 DSH。判断标准是：若某个版本在隔离一致性测试中证明能降低实现与维护成本，且不要求侵入治理根，就可以成为可替换运行时；否则 Syn 仍可使用自己的默认实现。

### 2.3 吸收 DSH 的方法，而不是照搬产品边界

Syn 当前吸收以下方法：

- Service / Provider / Consumer 式能力接缝；
- Profile / Bundle / Patch 式运行配置组合；
- 模型可见上下文必须能从 append-only runtime event 重建；
- Step / Turn 分离和 pre-execute / execute / post-execute 工具管线；
- 外部副作用前 checkpoint，以及崩溃后诚实记录 `OUTCOME_UNKNOWN`；
- Plugin / Tool / Workflow / Child Run 的版本化、可观察和可替换；
- 动态扩展只能产生候选，不能自己批准并晋升。

不吸收以下越界：

- “Everything is a Plugin”扩大到治理根；
- 把 session log 当公司事实或长期记忆；
- 把插件卸载当现实副作用回滚；
- 把 worker / goal / final answer 自报当独立验收；
- 把进程内 job / schedule 当 24×7 持久运营；
- 把文件系统 sandbox 当网络、进程、凭据和恶意代码的完整安全边界。

### 2.4 自升级成为明确的长期平台阶段

Syn 自升级不是在线修改当前进程并立即生效，而是受治理的候选晋升链：

```text
真实反馈 / 失败 / 成本信号
  -> 生成 Skill / Profile / Plugin / Code 候选
  -> 来源、依赖、能力与权限差异分析
  -> 隔离 worktree / workcell
  -> 确定性测试、历史回放与反例集
  -> 独立 Reviewer / Verifier
  -> 小范围 canary
  -> 签名晋升
  -> 观察、撤回与回滚
```

首版边界：

- 低风险 Skill、Prompt、Profile 和确定性脚本可以在用户已签署的政策内逐步自动化；
- 核心代码候选默认停在用户批准的晋升门；
- 修改身份、Policy、ExecutionGrant、Sandbox、Credential、Audit、Budget、Verifier、Updater 信任链或 Kill Switch 永远属于治理根变更，生成者不能批准；
- 没有独立、便宜、精确且难以被候选钻空子的验证器时，只能生成建议或草案；
- 邮件、付款、发布、合同等现实动作必须使用幂等、回读、补偿和人工接管，不能靠“插件可逆”声明回滚。

## 3. 对当前路线的影响

- M1–M4 已完成的具名范围不重开，也不改写已冻结 v1 合同；
- M5 增加 vendor-neutral `AgentRuntimeAdapter`、运行工作单元、轨迹、Checkpoint、不确定结果和独立验收合同；
- M6 区分组织身份与 runtime child run；
- M7 把可执行 Skill / Plugin / Profile 纳入来源、权限差异、评测、签名、灰度和撤回治理；
- M8 区分 AgentRuntime、Tool、Harness、KnowledgeSource 与 Connector，不合成万能 adapter；
- M9 把 runtime / profile / package 纳入可逆注册与退役清单，但现实副作用另行结算；
- M10 增加 runtime 崩溃、恢复、重复副作用、供应链、sandbox 降级、成本和独立验收门；
- M11 新增“受治理自升级平台”阶段，依赖 M7 的候选治理、M9 的退役能力与 M10 的发布 / 回滚基础。

## 4. 不代表什么

本决定不表示：

- 当前已经实现 AgentRuntime、DSH 适配或自升级；
- 已经批准安装 DSH、动态插件或第三方 package；
- 已经授权运行真实模型、外部连接器、真实账号、远端服务或无人值守任务；
- 已经授权修改产品代码、Git 提交、合并、推送或发布；
- DSH 当前版本的可靠性、安全性、成本或长期自治已经通过 Syn 验收。

实现仍需按当前用户指令、对应阶段、唯一任务包、验证和退场规则逐步进入。
