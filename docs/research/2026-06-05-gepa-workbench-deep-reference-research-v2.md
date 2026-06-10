# GEPA Workbench Deep Reference Research v2

日期：2026-06-05

状态：外部项目深度研究参考，已作为 `docs/workbench-system-architecture-v1.md` 后置优化层拆解依据登记。本文用于给全局主管审核 GEPA 对最终工作台蓝图的参考价值、适配阶段和风险边界。本文不进入中间版本计划，不进入 backlog，不拆任务包，不授权实现，不替代 `CURRENT.md`、`AUTHORITY.md`、`docs/plans/middleware-version-stage-plan-v1.md`、`docs/workbench-system-architecture-v1.md`、`docs/memory-layer-design-v1.md` 或最终蓝图。

## 0. 先说薄弱点

- 本轮没有本地安装运行 GEPA，也没有复现实验结果。
- 本轮没有完整审计 GEPA 全部源码；重点读取了官方 README、GitHub API、adapter / state / result / engine / callback / cost / acceptance / stop condition 等核心实现和文档。
- arXiv API 本轮返回 rate exceeded；论文入口仍以官方 README 链接为准，但本文不把论文实验数字当成本地已复现事实。
- GEPA 官方 README 中的生产案例、提升比例和成本数字是上游声明；本文只用它们判断“可能适配方向”，不当作我们产品一定能获得同样收益的证据。
- GEPA 优化的是文本参数和可评分系统行为，不是正式记忆、项目事实、权限、审计或控制核心。
- 如果没有稳定 trace、评估集、版本、预算、脱敏和回滚，过早接入 GEPA 会制造“自动改系统行为”的风险。

## 1. 本轮和 v1 的区别

v1 主要回答：GEPA 适不适合进入我们的阶段计划。结论是不进入当前阶段 E 主线，只作为后置优化层候选。

v2 继续往下看：

- GEPA 的核心算法和工程接口。
- 它依赖哪些输入：candidate、eval、trace、feedback、score、budget。
- 它输出什么：candidate lineage、Pareto frontier、best candidate、run log。
- 它的 callback、state resume、成本追踪和 stop condition 对我们有什么启发。
- 它和记忆层、技能层、adapter、工作流画布分别是什么关系。
- 如果未来接入，应该从什么最小能力开始，而不是直接运行优化器。

大白话：

v1 是“现在别做”。v2 是“以后如果要做，必须怎么做才不会失控”。

## 2. 资料来源

外部一手资料：

- GitHub：`https://github.com/gepa-ai/gepa`
- GitHub API：`https://api.github.com/repos/gepa-ai/gepa`
- README：`https://raw.githubusercontent.com/gepa-ai/gepa/main/README.md`
- 官方文档：`https://gepa-ai.github.io/gepa/`
- adapter guide：`https://raw.githubusercontent.com/gepa-ai/gepa/main/docs/docs/guides/adapters.md`
- cost tracking guide：`https://raw.githubusercontent.com/gepa-ai/gepa/main/docs/docs/guides/cost-tracking.md`
- acceptance criterion guide：`https://raw.githubusercontent.com/gepa-ai/gepa/main/docs/docs/guides/acceptance-criterion.md`
- 论文入口：`https://arxiv.org/abs/2507.19457`

本轮只读过的关键源码：

- `src/gepa/api.py`
- `src/gepa/core/adapter.py`
- `src/gepa/core/state.py`
- `src/gepa/core/result.py`
- `src/gepa/core/engine.py`
- `src/gepa/core/callbacks.py`
- `src/gepa/strategies/candidate_selector.py`
- `src/gepa/utils/stop_condition.py`

本地对齐依据：

- `CURRENT.md`
- `AUTHORITY.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `docs/workbench-system-architecture-v1.md`
- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/memory-layer-design-v1.md`
- `docs/plans/memory-layer-implementation-slice-v1.md`
- 最终蓝图：`/Users/yoyi/Documents/Codex/2026-05-26/gan-xing-codexbridge-https-github-com/docs/architecture/local-ai-workbench-blueprint-v1.md`
- UI 蓝图：`/Users/yoyi/Documents/Codex/2026-05-26/gan-xing-codexbridge-https-github-com/docs/architecture/local-ai-workbench-ui-blueprint-v1.md`

## 3. 当前仓库事实

2026-06-05 公开 GitHub API 显示：

- 仓库：`gepa-ai/gepa`
- 描述：`Optimize prompts, code, and more with AI-powered Reflective Text Evolution`
- 默认分支：`main`
- 创建时间：`2025-08-05T09:26:27Z`
- 最近 push：`2026-06-04T18:32:45Z`
- License：MIT
- homepage：`https://gepa-ai.github.io/gepa/`
- GitHub API 主语言显示为 Jupyter Notebook；这通常受 notebook / docs / artifact 体量影响，不代表核心库不是 Python。

另有 `CerebrasResearch/gepa`，但其公开元数据更小、更新时间更早；本文以 `gepa-ai/gepa` 为当前主要研究对象。

## 4. GEPA 到底是什么

GEPA 是 Genetic-Pareto，一种用 LLM 反思执行轨迹来优化文本参数的框架。

它不是训练模型权重，也不是普通 prompt 手工调参。它优化的是“系统中的文本部件”，例如：

- prompt。
- code。
- agent architecture。
- configuration。
- RAG query / answer / rerank prompt。
- MCP tool description。
- task instruction。
- skill text。

官方 README 的核心说法是：如果一个系统能被评分，GEPA 就可以尝试优化它的文本参数。

它的基本循环是：

```text
seed candidate
-> run evaluator on train / val examples
-> capture scores, outputs, trajectories, error messages, logs
-> build reflective dataset
-> reflection LM diagnoses why candidate failed
-> mutate one or more text components
-> accept if minibatch score improves
-> full val eval
-> update Pareto frontier
-> repeat until metric / cost / time / file stop condition hits
```

关键不是“让模型想一个更好的 prompt”，而是“让模型读取失败 trace 和反馈，再提出有针对性的文本改动”。

## 5. GEPA 和 GRPO / RL 的区别

GEPA：

- 不训练模型权重。
- 不需要模型参数或梯度。
- 主要优化文本参数。
- 依赖可读 trace 和 evaluation feedback。
- 可以用于 API-only 模型。
- 输出人可读候选、候选谱系和优化日志。

GRPO / RL：

- 更像策略 / 模型行为优化。
- 通常依赖大量 rollout 和奖励。
- 反馈常被压成标量 reward。
- 更适合模型训练或策略微调层，不适合直接作为工作台产品第一步。

对我们的大白话结论：

GEPA 更像“复盘过去任务，把任务包模板、主管提示词、adapter 说明、工作流节点说明改得更好”。它不是“让工作台自己学习并自动变聪明”的完整记忆系统。

## 6. GEPA 的工程接口

### 6.1 Candidate

GEPA 的核心候选是：

```text
dict[str, str]
```

也就是：

```text
component_name -> component_text
```

这对我们很重要。它天然适合优化：

- `project_director_task_decomposition_prompt`
- `worker_task_package_template`
- `global_boundary_review_prompt`
- `memory_candidate_extraction_prompt`
- `memory_impact_report_prompt`
- `adapter_tool_description`
- `workflow_node_instruction`

但不适合直接优化：

- 正式记忆 record。
- 权限策略。
- workflow state。
- 审计事件。
- 用户偏好事实。
- 项目事实。

### 6.2 GEPAAdapter

官方 adapter 协议要求核心方法：

```text
evaluate(batch, candidate, capture_traces)
make_reflective_dataset(candidate, eval_batch, components_to_update)
```

`evaluate` 要返回：

- outputs。
- scores。
- trajectories。
- objective_scores。
- num_metric_calls。

`make_reflective_dataset` 要把 trace、output、score、feedback 整理成可给反思模型看的 JSON-like 数据。

这意味着我们如果未来接 GEPA，第一步不是装包，而是先定义：

- 工作台哪些任务可评分。
- 每个任务的输入是什么。
- 输出如何判分。
- trace 里哪些字段可以给优化器看。
- 哪些敏感字段必须脱敏。
- 哪些组件允许被优化。

### 6.3 optimize_anything

官方还提供 `optimize_anything`，普通场景只写 evaluator 即可。

这对我们有启发：第一版不需要深度嵌入 GEPA SDK，可以先做一个“导出评估包 + 离线运行 + 导入候选报告”的边界。

正确形态：

```text
Workbench
-> export OptimizationDataset
-> external GEPA run
-> import OptimizationReport
-> Candidate review
-> controlled publish
```

错误形态：

```text
Workbench runtime
-> GEPA directly mutates production prompts
-> worker immediately uses changed behavior
```

## 7. GEPA 的状态、预算和可观测性

源码显示 GEPA 有这些运行结构：

- `GEPAState`
- `GEPAResult`
- `program_candidates`
- `parent_program_for_candidate`
- `val_aggregate_scores`
- `val_subscores`
- `per_val_instance_best_candidates`
- `program_at_pareto_front_valset`
- `objective_pareto_front`
- `evaluation_cache`
- `run_dir`
- `run_log.json`
- `candidates.json`
- `candidate_tree_html`

它还支持：

- `max_metric_calls`
- `max_reflection_cost`
- `FileStopper`
- `TimeoutStopCondition`
- `ScoreThresholdStopper`
- `NoImprovementStopper`
- `SignalStopper`
- `MaxCandidateProposalsStopper`

对我们有价值：

- 优化运行必须是一个可停止、可恢复、可审计的 run object。
- 候选必须有 lineage。
- 预算必须硬限制。
- 每个候选的来源、父候选、评分和被接受 / 拒绝原因都要保留。
- 运行日志不等于审计；审计记录“谁批准候选进入生产”。

## 8. Pareto frontier 对我们的意义

GEPA 不只追一个最高平均分候选。它保留在不同验证样本或不同目标上表现好的候选。

源码里有 `frontier_type`：

- `instance`
- `objective`
- `hybrid`
- `cartesian`

这对我们的工作台很有用。因为工作台不是单目标系统。

一个任务包模板可能：

- 对代码任务更好。
- 对文档任务更好。
- 对低风险任务更快。
- 对高风险任务更稳。
- 对 Codex 好，但对 Claude Code 不好。
- 对英文任务好，但对中文任务不好。

所以未来不能只存一个“全局最优 prompt”。更合理的是：

```text
OptimizationCandidate
-> best_for_project_type
-> best_for_agent_adapter
-> best_for_task_kind
-> best_for_risk_class
-> excluded_when
```

这和我们的记忆层 / 成熟模式设计能对上：成熟模式不是一条万能经验，而是带适用范围和反例的受控经验。

## 9. GEPA 对记忆层的启发

GEPA 不是记忆层。

它能产生：

- OptimizationReport。
- PromptCandidate。
- SkillCandidate。
- MaturePatternCandidate。
- TaskPackageTemplateCandidate。
- AdapterInstructionCandidate。

它不能直接产生：

- FormalMemory。
- 用户偏好正式记忆。
- 项目事实。
- 权限规则。
- 方案授权。

正确链路：

```text
GEPA run
-> OptimizationReport
-> Candidate
-> impact report
-> project director / global director / user review
-> controlled publish to prompt/template/skill/pattern store
-> version + audit
```

如果 GEPA 从 trace 中发现“用户喜欢简短汇报”，也不能自动写用户偏好记忆。它只能生成一个 `MemoryCandidate` 或影响面报告，由用户确认。

## 10. GEPA 对技能层的启发

GEPA 官方 README 和 docs 把 skills 作为重要使用方向之一，尤其是自动学习 coding agent skills。

这点很适合我们的最终蓝图，但必须和记忆分开：

- Memory：系统行为依据和事实依据。
- Skill：如何做某类事的方法、模板、步骤、策略。
- GEPA：优化 Skill / Prompt / Template 的实验机制。

正确边界：

```text
Worker traces
-> repeated success / failure patterns
-> SkillCandidate
-> project/global review
-> versioned Skill
-> future task package can include skill by scope/risk
```

错误边界：

```text
GEPA run
-> directly modifies global skill
-> all future workers silently change behavior
```

## 11. 对当前阶段的适配

### 11.1 当前不应做 GEPA 实现

当前阶段事实：

- C1-C6 自动化工作流闭环已完成。
- M1-M13 记忆系统中间版本权威验收已完成，结论是 `accepted_with_deferred_items`。
- E1 已完成 adapter descriptor 执行边界和模型 / 凭据只读状态底座。
- 后续主线仍是阶段 E / 阶段 G：会话、真实 adapter、多 agent、模型凭据、真实 Tauri 验收、运行日志、自动重试、运维诊断。

因此 GEPA 不应进入当前 E1 / E2 任务包，也不应抢在真实 adapter / 日志 / eval 前面实现。

### 11.2 阶段 E 应该预留什么

阶段 E 可以为 GEPA 预留数据契约，但不运行 GEPA：

- `AgentRunTrace`
- `AdapterExecutionTrace`
- `TaskPackageTemplateVersion`
- `PromptComponentVersion`
- `ModelEndpointIdentity`
- `CredentialScope`
- `ToolCapabilityRisk`
- `EvalCase`
- `EvalMetric`
- `RunCost`
- `FailureReason`
- `UserOutcomeFeedback`

这些字段不是为了 GEPA 本身，而是任何可靠优化系统都需要。

### 11.3 阶段 F 可以考虑什么

如果阶段 F 深化画布和工作流，GEPA 可以作为后置研究影响：

- 工作流节点说明优化。
- 项目工作流模板优化。
- Deep Research 节点 prompt 优化。
- 任务拆分模板优化。
- readback / failure diagnosis prompt 优化。

但画布仍不能成为事实源；优化候选也不能自动改工作流。

### 11.4 阶段 G 是真正前置条件

阶段 G 必须先补：

- `WorkbenchRuntimeLog`
- `ToolExecutionLog`
- `AdapterHealth`
- `DiagnosticBundleExport`
- `TraceRedactionPolicy`
- `PromptVersionRegistry`
- `EvalCaseStore`
- `OptimizationRunStore`
- `CostBudgetGuard`
- `RollbackPlan`

没有这些，GEPA 只能停留在研究池。

## 12. 最适合我们的接入形态

推荐未来架构：

```text
Control Core
  owns permissions, memory, workflow state, audit, publish gates

Trace / Eval Exporter
  exports redacted traces and eval cases

Optimization Adapter
  runs GEPA offline or isolated

Optimization Store
  stores runs, candidates, scores, lineage, cost

Candidate Governance
  turns optimization output into PromptCandidate / SkillCandidate / TemplateCandidate

Review / Publish Gate
  project director, global director, or user approves based on impact
```

GEPA 不应在控制核心内部运行。控制核心只接受 GEPA 产出的候选和报告。

## 13. 第一批可优化对象排序

未来如果真的开始做 GEPA，建议从低风险到高风险：

1. readback failure diagnosis prompt。
2. memory impact report prompt。
3. 记忆中心“影响面说明”模板。
4. worker structured report parsing / summarization prompt。
5. 项目主管过程事实候选说明模板。
6. 任务包模板中的非执行性说明部分。
7. adapter wrapper prompt。
8. Deep Research 节点 prompt。
9. workflow template。
10. coding skill / mature pattern。

不要第一批优化：

- 权限 guard。
- 正式记忆写入规则。
- 用户偏好判定规则。
- 全局主管最终授权规则。
- 真实 agent 执行策略。
- shell/file/MCP/tool permission。

## 14. GEPA 和前端显示边界

GEPA 不应成为左侧一级入口。

如果未来加入 UI，推荐位置：

- `管理 > 优化实验`
- `管理 > 日志 / 诊断`
- `开发者模式`
- `记忆中心 > 成熟模式候选`
- `工作流节点详情 > 模板优化建议`

普通用户应看到：

- 想改什么。
- 为什么建议改。
- 基于哪些案例。
- 预计影响哪些项目 / adapter / 模板 / 技能。
- 成本用了多少。
- 风险是什么。
- 如何回滚。
- 谁需要确认。

普通用户不应默认看到：

- raw trace。
- 全部 optimizer population。
- 反思模型原始长输出。
- Pareto frontier 内部细节。
- token / embedding / vector dump。
- 未脱敏错误日志。

## 15. 安全和隐私风险

GEPA 的输入通常是执行 trace。工作台 trace 可能包含：

- 用户需求。
- 项目路径。
- 文件名。
- 代码片段。
- 错误日志。
- 工具输出。
- 账号、token、环境变量痕迹。
- 业务数据。
- 记忆候选。
- 知识库引用。

因此必须先有：

- trace redaction。
- secret scanner。
- project scope check。
- external model egress policy。
- per-run budget。
- per-run allowed data scope。
- candidate publication gate。
- audit event。

另一个风险是优化目标错配。

如果 eval 只奖励“完成得快”，GEPA 可能学会跳过解释、绕过保守确认、少报风险。我们的 eval 必须包含：

- 正确性。
- 安全边界。
- 权限合规。
- 记忆使用合规。
- 用户可理解性。
- 可回滚性。
- 成本。
- 失败可诊断性。

## 16. 为什么不能把 GEPA 当“自动学习记忆”

GEPA 看到的是历史运行和评分，它能推断“什么文本策略可能更有效”。这不是事实确认。

举例：

```text
GEPA 发现 worker 在任务包里看到三条项目背景时成功率更高
```

这只能产生：

```text
TaskPackageTemplateCandidate
```

不能产生：

```text
FormalMemory: "以后所有 worker 都必须看到三条背景"
```

因为这句话涉及：

- 项目范围。
- 任务类型。
- 隐私。
- 上下文预算。
- agent 类型。
- 反例。
- 用户确认。

所以 GEPA 是优化层，不是记忆层，不是秘书，不是主管。

## 17. 最小可落地路径

未来可以按以下研究切片推进，但现在不授权实现：

### GEPA-0：数据契约设计

只写设计，不接 SDK。

输出：

- `PromptComponentVersion`
- `EvalCase`
- `EvalMetric`
- `RedactedTrace`
- `OptimizationCandidate`
- `OptimizationReport`
- `OptimizationAuditEvent`

### GEPA-1：离线导出

只导出脱敏 eval 包，不运行 GEPA。

验收：

- 能从真实运行日志生成 eval cases。
- 能证明敏感信息已脱敏。
- 能按项目 / adapter / task kind 过滤。

### GEPA-2：离线 dry run

在隔离目录运行 GEPA，小预算，只产出报告。

验收：

- 不改生产 prompt。
- 不改正式记忆。
- 不改 workflow state。
- 只产生 OptimizationReport。

### GEPA-3：候选治理

把 GEPA 输出变成候选。

验收：

- 候选有 diff。
- 候选有影响面。
- 候选有基于哪些案例。
- 候选有回滚计划。

### GEPA-4：受控发布

由项目主管 / 全局主管 / 用户按影响范围批准。

验收：

- 发布写版本和审计。
- 可回滚。
- 可只对某项目 / adapter / task kind 生效。

## 18. 值得借鉴的清单

建议后续保留为研究候选：

1. Candidate 作为 `component_name -> component_text`。
2. `evaluate` / `make_reflective_dataset` adapter 边界。
3. Actionable Side Information，把错误、日志、诊断作为反思材料。
4. Pareto frontier，而不是只追平均分。
5. Candidate lineage。
6. run_dir resume。
7. candidates.json / run_log.json。
8. callback event model。
9. max_metric_calls。
10. max_reflection_cost。
11. FileStopper / graceful stop。
12. evaluation cache。
13. acceptance criterion 可替换。
14. multi-objective scoring。
15. candidate tree visualization。

## 19. 不建议吸收的清单

不建议吸收：

1. 直接把 GEPA SDK 放进控制核心。
2. 直接让 GEPA 修改生产 prompt。
3. 直接让 GEPA 写正式记忆。
4. 直接让 GEPA 修改 workflow state。
5. 直接让 GEPA 改 adapter 权限。
6. 把 GEPA 作为一级入口。
7. 用 GEPA 结果替代项目主管 / 全局主管 / 用户确认。
8. 用单一平均分决定全局发布。
9. 在未脱敏 trace 上调用外部 reflection LM。
10. 没有成本预算就运行优化。
11. 没有 eval set 就优化。
12. 没有回滚计划就发布候选。

## 20. 给全局主管的审核问题

建议新的全局主管审核时回答：

1. 是否继续确认 GEPA 只作为后置优化层候选，不进入当前执行计划。
2. 是否接受阶段 E 只做 trace / eval / prompt version 的数据预留，不运行 GEPA。
3. 是否接受阶段 G 是 GEPA 的硬前置条件：日志、诊断、预算、脱敏、回滚。
4. 是否同意未来 GEPA 只能输出候选和报告，不能直接写正式记忆或生产规则。
5. 是否允许后续单开 `OptimizationRunStore` / `PromptVersionRegistry` 专题设计。
6. 是否允许后续单开 `SkillCandidate` 和 `MaturePatternCandidate` 的 GEPA 适配研究。
7. 是否确认 GEPA UI 只能进管理 / 开发者 / 诊断区域，不做左侧一级入口。
8. 是否确认任何 GEPA 发布都必须有版本、审计和回滚。
9. 是否确认 GEPA eval 必须包含安全、权限、记忆合规和用户可理解性，不只看任务成功率。
10. 是否确认 GEPA 不能替代用户偏好确认。

## 21. 最终建议

建议全局主管给出如下口径：

```text
GEPA 值得继续深入研究，但仍不进入当前中间版本执行计划。
它最有价值的是把真实运行 trace、失败原因、评分和候选版本结合起来，形成 Prompt / Skill / Template 的后置优化实验机制。
它最不能照搬的是自动把优化结果写入生产行为、正式记忆、权限或工作流状态。
当前阶段只保留架构预留意识：阶段 E 补 trace / adapter / model / eval 数据契约，阶段 G 补运行日志、诊断、预算、脱敏和回滚。
如果未来融合，必须先做 GEPA-0 数据契约，再做离线导出和 dry run，最后才进入候选治理和受控发布。
```

这份研究应当作为“优化层安全说明书”使用：它提醒我们以后可以用 GEPA 改进工作台里的提示词、模板、技能和成熟模式，但必须先有日志、评估、预算、脱敏、版本和确认闸门。

## 22. 本轮公开来源复核记录

2026-06-05 本轮只读复核公开来源，结论如下：

- GitHub API `https://api.github.com/repos/gepa-ai/gepa` 显示仓库为 `gepa-ai/gepa`，默认分支 `main`，描述为 `Optimize prompts, code, and more with AI-powered Reflective Text Evolution`，最近 push 为 `2026-06-04T18:32:45Z`。
- README 原文确认 GEPA 的目标是优化 prompts、code、agent architectures、configurations 等文本参数，机制是 LLM-based reflection 与 Pareto-efficient evolutionary search。
- README 原文确认 GEPA 支持 `optimize`、`optimize_anything`、DSPy 集成，以及 Default / Confidence / DSPy Full Program / RAG / MCP / TerminalBench / LangChain 等 adapter。
- `src/gepa/core/adapter.py` 确认 adapter 核心职责是 `evaluate` 和 `make_reflective_dataset`，并要求 individual example failure 不应直接抛异常，而应返回失败 score 和 trace。
- `src/gepa/core/state.py` / `src/gepa/core/result.py` 确认 GEPA 保存 candidates、parents、val scores、Pareto frontier、evaluation cache、run metadata 和 schema version。
- `src/gepa/core/engine.py` / `src/gepa/core/callbacks.py` 确认 GEPA 有 optimization lifecycle、candidate accepted / rejected、merge、Pareto updated、state saved、budget updated、error 等 callback event。
- cost tracking guide 确认 reflection LM 可追踪 cost / token，并支持 `max_reflection_cost`。
- acceptance criterion guide 确认默认是 strict improvement，也可替换成自定义接受规则。
- stop condition source 确认有 timeout、file stopper、score threshold、no improvement、signal、metric calls、reflection cost、candidate proposal 等停止条件。
- 以上复核只证明 GEPA 当前公开实现和文档事实，不证明它已经适合进入我们的当前开发计划。
