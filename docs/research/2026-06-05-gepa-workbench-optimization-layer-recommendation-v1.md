# GEPA Workbench Optimization Layer Recommendation v1

日期：2026-06-05

状态：外部项目研究建议已审核。本文只作为后置优化层候选参考，不进入当前阶段 E 的 E1 / E2 主线，不进入当前执行 backlog，不拆任务包，不授权实现，不替代 `CURRENT.md`、`AUTHORITY.md`、`docs/plans/middleware-version-stage-plan-v1.md`、`docs/workbench-system-architecture-v1.md`、`docs/memory-layer-design-v1.md` 或最终蓝图。

## 0.1 全局主管审核结论

审核结论：

- 接受 GEPA 作为“后置优化层候选”保留在研究池。
- 不接受 GEPA 进入当前中间版本近期开发主线。
- 不接受 GEPA 进入当前阶段 E / E1 或 E2 任务包。
- 不新增 GEPA 任务包，不把本文放入执行 backlog。
- 阶段 E 可以保留架构预留意识，但 `GEPA-0` 只能在 E1 / E2 / 必要的模型凭据边界任务完成后，由全局主管重新审核并单独批准。
- 真正运行 GEPA 必须等阶段 G 的运行日志、诊断、eval、成本预算、脱敏和回滚底座完成。

当前 E1 已完成 adapter descriptor 执行边界和模型 / 凭据只读状态底座；GEPA 不吸收进 E1。下一步如果继续阶段 E，仍应优先处理会话操作边界、模型 / 凭据只读深化和真实 adapter 接入前置设计，而不是启动优化器。

## 0. 先说薄弱点

- GEPA 是快速发展的外部项目，本轮只核对了公开官方 README、arXiv 链接和本地蓝图 / 阶段文档，没有本地安装、运行或逐源码审计。
- GEPA 官方 README 中的效果数字和生产案例是上游声明，本轮没有复现实验。
- GEPA 官方 docs 页面本轮通过本地网络读取时出现连接重置；本文主要依据官方 README 和论文入口给出的核心机制描述。
- GEPA 优化的是 prompt、文本参数、agent architecture、config 等“行为文本”，不是正式记忆、控制核心、审计系统或 agent 执行器。
- 如果没有稳定运行日志、评估集、prompt 版本、成本边界和回滚机制，过早接入 GEPA 会放大不可控自我修改风险。

## 1. 本轮建议结论

当前建议：

- 接受 GEPA 作为“工作台后置优化层候选”。
- 不把 GEPA 纳入当前中间版本主线实现。
- 不把 GEPA 纳入当前 E1 / E2 主线，不新增当前执行任务包。
- 不把 GEPA 放进左侧一级入口。
- 不让 GEPA 直接写正式记忆、直接改生产 prompt、直接推进工作流或直接执行真实 agent。
- 阶段 E 后段可做数据契约和设计预留；阶段 G 完成运行日志、真实验收、错误码、自动重试、诊断、成本预算、脱敏和回滚之后，才适合做离线优化实现。

大白话：

GEPA 适合帮工作台“复盘过去的执行轨迹，提出更好的提示词、任务包模板和工作流模板”。它不适合现在直接上来改系统行为。它应该像一个优化实验室，先产出候选和报告，再由项目主管 / 全局主管 / 用户按影响范围确认。

## 2. 资料来源

外部资料：

- GEPA GitHub：`https://github.com/gepa-ai/gepa`
- GEPA README：官方描述为“Optimize any text parameter — prompts, code, agent architectures, configurations — using LLM-based reflection and Pareto-efficient evolutionary search.”
- GEPA 论文入口：`https://arxiv.org/abs/2507.19457`
- GEPA 官方文档入口：`https://gepa-ai.github.io/gepa/`
- GEPA adapters guide 入口：`https://gepa-ai.github.io/gepa/guides/adapters/`

本地权威依据：

- 最终蓝图：`/Users/yoyi/Documents/Codex/2026-05-26/gan-xing-codexbridge-https-github-com/docs/architecture/local-ai-workbench-blueprint-v1.md`
- UI 蓝图：`/Users/yoyi/Documents/Codex/2026-05-26/gan-xing-codexbridge-https-github-com/docs/architecture/local-ai-workbench-ui-blueprint-v1.md`
- `docs/workbench-system-architecture-v1.md`
- `docs/memory-layer-design-v1.md`
- `docs/plans/memory-layer-implementation-slice-v1.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `docs/workbench-frontend-display-boundary-v1.md`
- `CURRENT.md`
- `tasks/README.md`

## 3. GEPA 是什么

GEPA，全称 Genetic-Pareto，是一种用 LLM 反思执行轨迹来优化文本参数的框架。

它优化的对象可以是：

- prompt。
- 任务说明。
- agent architecture 文本。
- 工具描述。
- MCP tool description。
- RAG query / answer / rerank prompt。
- 配置策略。
- 代码或其他文本 artifact。

它大致流程是：

```text
候选文本
-> 在评估集上执行
-> 收集完整 trace、错误、工具输出、日志、评分
-> LLM 读取 trace 并反思失败原因
-> 生成 mutated candidate
-> 用 Pareto selection 保留在不同任务子集上表现好的候选
-> 继续迭代
```

GEPA 和 GRPO / RL 的区别：

- GEPA 不训练模型权重。
- GEPA 不需要访问模型参数。
- GEPA 更像“离线提示词和系统文本进化器”。
- GRPO / RL 更像“用奖励信号优化模型行为或策略”。

## 4. 为什么它和工作台相关

我们的工作台未来会有大量可优化的文本协议：

- 项目主管拆任务说明。
- worker 任务包模板。
- 全局主管方案边界复核提示词。
- readback 失败诊断提示词。
- 权限请求解释模板。
- 记忆候选抽取说明。
- 记忆冲突和影响面报告说明。
- adapter wrapper prompt。
- Codex / Claude Code / OpenClaw / OpenCode 的统一协作协议。
- 画布节点执行说明。
- workflow template。
- 成熟模式候选说明。

这些内容不是普通 UI 文案。它们会影响 agent 怎么理解任务、怎么使用记忆、怎么上报失败、怎么申请权限。GEPA 的价值正好在这里：它可以利用真实执行 trace 和评估结果，提出更好的版本候选。

## 5. 为什么现在不能直接加入实现

当前本地阶段事实：

- 阶段 C1-C6 自动化工作流闭环已完成。
- 阶段 D / M1-M13 记忆系统已完成中间版本最终权威验收，结论为 `accepted_with_deferred_items`。
- 下一步建议进入阶段 E：会话、adapter、多 agent 和模型凭据底座。
- 阶段 G 仍需补真实 Tauri 全面验收、运行日志、自动重试和运维诊断。

GEPA 第一版实现至少需要：

- 稳定的运行日志。
- 可复现的 eval case。
- prompt / template 版本管理。
- agent adapter 身份和模型身份。
- 成本统计。
- trace 脱敏。
- 失败原因分类。
- 回滚机制。
- 候选采纳审计。

这些正好落在阶段 E 和阶段 G 的后续范围里。现在直接实现，会出现几个风险：

- 没有稳定 trace，GEPA 优化材料不可靠。
- 没有 eval set，只会优化偶然案例。
- 没有 prompt version registry，无法追踪哪次改动导致行为变化。
- 没有运行日志和成本边界，可能产生隐形模型调用成本。
- 没有采纳 gate，可能自动把实验候选变成生产规则。

## 6. 和当前架构的适配位置

GEPA 不应放进控制核心内部。

推荐定位：

```text
核心外 OptimizationAdapter / PromptOptimizationService
```

它可以读取受控导出的材料：

- 运行日志。
- adapter trace。
- worker report。
- readback failure。
- 任务包 artifact。
- 记忆包 included / excluded 理由。
- eval case。
- 用户 / 项目主管 / 全局主管结果反馈。

它输出的只能是候选：

- `PromptCandidate`
- `TaskPackageTemplateCandidate`
- `WorkflowTemplateCandidate`
- `AdapterInstructionCandidate`
- `SkillCandidate`
- `MaturePatternCandidate`
- `OptimizationReport`

它不能：

- 直接写正式记忆。
- 直接写 workflow state。
- 直接改生产 prompt。
- 直接改 adapter 权限。
- 直接调用真实 worker。
- 直接调用真实工具。
- 直接删除旧版本或审计。

控制核心仍然负责：

- 判断候选是否可采纳。
- 判断影响范围。
- 判断需要项目主管、全局主管还是用户确认。
- 发布新版本。
- 写审计。
- 支持回滚。

## 7. 和记忆层的边界

GEPA 产出的“经验总结”不能自动成为正式记忆。

正确链路：

```text
GEPA optimization run
-> OptimizationReport
-> MaturePatternCandidate / SkillCandidate / PromptCandidate
-> 主管或用户确认
-> 受控写入对应 store
-> version + audit
-> 后续任务包按权限召回
```

错误链路：

```text
GEPA 发现某个做法有效
-> 直接写 FormalMemory
-> 直接进入 worker 任务包
```

记忆层仍然遵守：

- observation 不是正式记忆。
- candidate 不是正式记忆。
- LLM summary 不是正式记忆。
- 派生索引不是正式记忆。
- 成熟模式候选不能自动升级为全局技能、全局记忆或项目规则。

## 8. 和前端显示边界的关系

GEPA 不应成为左侧一级入口。

如果后续加入 UI，建议位置是：

- `管理 > 优化实验`。
- `管理 > 日志 / 健康 / 诊断` 的某个报告入口。
- 记忆中心里的“成熟模式候选”详情。
- 工作流节点详情里的“模板优化建议”。
- 开发者模式或全局主管审核页。

不应显示在普通主界面：

- raw trace。
- optimizer internal population。
- Pareto frontier 细节。
- embedding / score raw dump。
- prompt diff 的全部内部推理。

普通用户应该看到：

- 这次优化想改什么。
- 为什么建议改。
- 基于哪些案例。
- 可能影响哪些项目 / agent / 模板。
- 预计收益和风险。
- 是否需要用户确认。
- 怎么回滚。

## 9. 推荐阶段安排

### 9.1 现在：只保留研究结论

状态：建议保留本文，不进入执行计划。

原因：

- 当前主线应继续阶段 E。
- GEPA 实现依赖阶段 E / G。
- 直接加入会打乱会话、adapter、多 agent、模型凭据和运行日志底座。

### 9.2 阶段 E 后段：只做设计预留

阶段 E 后段可以考虑一个轻量设计任务，但不能早于 E1 / E2 / 必要的模型凭据边界任务，也不能默认进入当前 backlog。是否拆 `GEPA-0` 必须由全局主管重新审核并单独批准。

```text
GEPA-0：优化目标、prompt 版本和 trace/eval 契约设计
```

目标：

- 定义哪些文本可以被优化。
- 定义 prompt / task package template / adapter instruction 的版本对象。
- 定义 trace 脱敏和导出规则。
- 定义 optimization candidate 的最小 schema。
- 定义成本预算和模型外发边界。

不做：

- 不安装 GEPA。
- 不跑优化。
- 不改生产 prompt。
- 不接真实模型调用。
- 不进入当前 E1 / E2。
- 不替代阶段 G 运行日志、成本、脱敏和回滚底座。

### 9.3 阶段 G 后：做离线优化实验

阶段 G 完成运行日志、真实验收和诊断体系后，可以做：

```text
GEPA-1：离线 replay / eval harness
```

目标：

- 选择一个低风险目标，比如 readback 失败解释 prompt。
- 用固定历史案例做离线评估。
- 输出候选和报告。
- 不自动采纳。

### 9.4 阶段 F 稳定后：优化画布和工作流模板

项目工作流画布产品化后，可以做：

```text
GEPA-2：工作流节点模板和任务包模板候选
```

适合目标：

- 项目主管拆任务模板。
- worker 任务包模板。
- 验证线 checklist。
- 回收线总结模板。
- 画布节点说明。

### 9.5 更后期：受控采纳和自动化优化循环

最终可以做：

```text
GEPA-3：受控采纳 prompt / template 新版本
GEPA-4：成熟模式 / 技能候选和跨项目优化报告
```

必须要求：

- 影响全局的候选由全局主管审核。
- 影响用户偏好、全局蓝图、跨项目行为的候选由用户确认。
- 所有生产替换都有版本、审计、回滚。

## 10. 第一批最适合 GEPA 的目标

优先级建议：

1. readback / 失败诊断说明。
2. worker 任务包模板。
3. 项目主管拆任务模板。
4. 记忆候选抽取和候选说明。
5. 任务包记忆入选 / 排除理由模板。
6. 验证线 checklist。
7. adapter wrapper prompt。

不建议第一批做：

- 用户偏好记忆生成。
- 全局蓝图记忆生成。
- 自动技能化。
- 自动工作流规划。
- 真实 Codex / Claude Code 执行策略。
- 高成本多模型优化。

## 11. 最小数据对象建议

如果后续全局主管同意进入设计预留，建议先定义这些对象：

### 11.1 `OptimizationTarget`

表示要优化什么。

字段候选：

- `target_id`
- `target_kind`
- `scope`
- `owner`
- `current_version_id`
- `allowed_inputs`
- `forbidden_outputs`
- `risk_level`
- `approval_required`

### 11.2 `PromptOrTemplateVersion`

表示当前生产文本版本。

字段候选：

- `version_id`
- `artifact_kind`
- `content_hash`
- `created_at`
- `created_by`
- `source_refs`
- `audit_ref`
- `rollback_from`

### 11.3 `OptimizationTraceRef`

表示 GEPA 可以读取的脱敏 trace 引用。

字段候选：

- `trace_ref_id`
- `source_kind`
- `project_id`
- `workflow_id`
- `agent_adapter_id`
- `model_id`
- `redaction_status`
- `permission_scope`
- `cost_summary`

### 11.4 `OptimizationCandidate`

表示 GEPA 产出的候选。

字段候选：

- `candidate_id`
- `target_id`
- `candidate_kind`
- `diff_summary`
- `expected_benefit`
- `known_risks`
- `eval_summary`
- `source_trace_refs`
- `approval_state`
- `adoption_ref`

### 11.5 `OptimizationReport`

给全局主管 / 项目主管 / 用户看的报告。

必须回答：

- 改了什么。
- 为什么改。
- 哪些案例变好。
- 哪些案例变差。
- 影响哪些项目、模板、agent 或任务包。
- 是否涉及记忆、知识库、权限、模型外发。
- 如何回滚。

## 12. 审核和采纳规则建议

项目内低风险候选：

- 项目主管可审核。
- 只能影响本项目 template 或 prompt。
- 必须写版本和审计。

跨项目或成熟模式候选：

- 全局主管审核。
- 用户确认后才能影响全局默认行为。
- 进入 mature pattern 或 skill candidate，不自动成为正式记忆。

用户偏好、权限、模型外发、凭据相关候选：

- 必须用户确认。
- GEPA 只能提出报告，不能代替确认。

生产替换：

- 必须有 A/B 或 replay 结果。
- 必须有 rollback。
- 必须有审计。
- 必须保留旧版本。

## 13. 不允许事项

GEPA 永远不能直接做这些事：

- 自动修改正式记忆。
- 自动批准候选记忆。
- 自动改用户偏好。
- 自动修改全局蓝图。
- 自动放宽权限。
- 自动改变项目主管 / 全局主管 / 用户确认边界。
- 自动调用真实 agent 执行任务。
- 自动调用 shell / MCP / 文件 / 浏览器工具。
- 自动把失败日志解释成事实。
- 自动删除或覆盖旧 prompt 版本。

## 14. 给全局主管的审核问题

建议新的全局主管审核时回答：

1. 是否接受 GEPA 作为“后置优化层候选”，而不是当前主线功能。
2. 是否允许阶段 E 后段在 E1 / E2 / 必要的模型凭据边界任务后，单独审核 `GEPA-0` 设计预留任务。
3. 第一批优化目标是否只选低风险文本，例如 readback 失败诊断和任务包模板。
4. GEPA 是否只能使用脱敏 trace。
5. GEPA 是否必须在阶段 G 运行日志和 eval harness 完成后才能运行。
6. 哪些候选由项目主管审，哪些由全局主管审，哪些必须用户确认。
7. 是否允许 GEPA 使用外部模型作为 reflection model。
8. 每次优化 run 的成本预算、模型外发和日志保留规则是什么。
9. UI 是否只放在管理 / 开发者模式 / 主管审核区，不作为普通用户入口。
10. 是否需要把 GEPA 和成熟模式 / 技能库设计一起研究。

## 15. 最终建议

建议全局主管给出如下结论：

```text
GEPA 暂不进入中间版本执行计划。
GEPA 作为后置优化层候选保留。
当前只接受研究文档。
阶段 E 后段可考虑新增设计预留任务 GEPA-0，但必须在 E1 / E2 / 必要的模型凭据边界任务后重新审核。
真正运行 GEPA 必须等阶段 G 的日志、诊断、eval、成本、脱敏和回滚底座完成。
GEPA 输出只能是候选和报告，不能绕过控制核心、记忆状态机、权限、审计或用户确认。
```

这条路线既保留了 GEPA 对工作台长期自我改进的价值，也不会让它破坏当前已经确认的项目主轴、控制核心、正式记忆治理和 UI 显示边界。
