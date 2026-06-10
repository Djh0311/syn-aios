# Stage H-I Real Codex Automation And Multi-Agent Collaboration Plan v1

日期：2026-06-07

状态：G5 后续开发计划草案；H0 阶段 H 安全边界和任务包冻结已完成文档冻结并已通过全局主管复核；H1 CodexLocalRunner 架构和数据契约已完成并已通过全局主管复核；H2 通用真实 resume 产品化任务包已创建并已完成 Phase B `mario test` 真实 resume 产品化探针；H2.0 real resume preflight authorization guard 已完成；H2.1 real resume authorization matrix and execution decision freeze 已完成；H2.2 real resume authorization readiness read model and readonly UI 已完成；H2.3 real resume request builder and CodexLocal guard bridge 已完成；H2.4 real resume execution authorization and fixture freeze 已完成；H2.5 real resume runner execution path and authorized fixture run Phase A 已完成；H2.6 Phase B readiness / fixture session binding / runtime log hardening 已完成，runtime log 显式 sidecar writer 已补齐；H2.7 Phase B authorization / fixture / target session confirmation 已完成为当时的授权准备复核和阻断状态冻结；H2.8 real execution permission dialog / audit summary / readiness decision surface 已完成并回收；后续 H2 Phase B 已在 2026-06-08 对 `mario test` 授权并完成一次真实探针；H3-A new session authorization / fixture / boundary freeze 已完成；H3.1 new session request / guard / permission envelope / no-op runner 已完成并已通过全局主管复核；H3-B final approval / real new session fixture run 已执行一次隔离 fixture 真实 probe 但失败分类完成，产品路径已补 `--skip-git-repo-check`，等待新的 retry 授权；H4 readback / failure / timeout / duplicate guard Level A 非真实产品化已完成并通过全局主管复核；H5 project workflow real dispatch integration 已完成 Level A 非真实产品路径集成并通过全局主管复核；H5-Level-B project workflow real dispatch authorization and fixture freeze 已完成；H5-Level-B1 mario test project workflow real dispatch run 已完成一次 read-only 真实 `resume` probe并已通过全局主管复核；H5-Level-B2 mario test project workflow write probe 已完成一次 workspace-write 真实 `resume` probe；合并型 H5 product command formalization / H5 acceptance checkpoint 已完成并通过全局主管复核：`tasks/2026-06-08-stage-h-h5-product-command-formalization-and-acceptance-checkpoint-v1.md`。本文用于冻结阶段 H / I 的开发方向、顺序、安全边界和验收口径；不是单个执行任务包，不授权直接开发，不授权直接执行新的真实 `codex exec` / `codex exec resume`。

## 0. 先说薄弱点

- 中间版本已经在 G5 冻结为 `accepted_with_deferred_items`，但这不是最终工作台可用态。
- 当前最关键缺口不是 UI 展示，而是 `codex-local` 真实 send / resume 还没有完成通用项目工作流产品化。E5 Level B、H2 Phase B、H5-Level-B1 和 H5-Level-B2 分别证明指定 mario test session 的真实 resume、项目工作流 read-only probe 和受控写入 probe 可行，但仍不等于 H3、H5 通用产品化或阶段 H 完成。
- 如果 H 阶段只继续做 preview、stub 或只读面板，工作台会继续停留在“能说明边界但不能真正干活”的状态。
- 如果 H 阶段直接开放自由发消息 / resume，又会绕过项目、方案授权、任务包、记忆包、权限、运行日志和审计，变成危险的裸 Codex 控制器。
- Codex 当前已有主管线向开发线派任务、开发线回交给主管线的多线程协作能力，有参考价值；但不能照搬。我们的工作台未来要接多种模型和 agent 产品，不能把产品模型硬编码成 Codex 线程模型。
- I 阶段如果太早接 Claude Code / OpenClaw / OpenCode 的真实执行，会把 provider 凭据、模型验证、外发风险、权限确认和运行日志问题一次性放大。I 必须先抽象协作协议，再逐个 adapter 产品化。

## 1. 权威依据

必须服从：

- `CURRENT.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `tasks/README.md`
- `docs/workbench-system-architecture-v1.md`
- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/workflow-task-package-design-v1.md`
- `docs/memory-layer-design-v1.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `docs/plans/2026-06-06-stage-e-f-g-refinement-plan-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`
- `evidence/2026-06-07-stage-g-g5-final-authoritative-acceptance-and-deferred-freeze-v1.md`
- `handoffs/2026-06-07-stage-g-g5-final-authoritative-acceptance-and-deferred-freeze-v1-result.md`

外部 / 内部参考只能作为设计输入：

- Codex 当前多线程协作能力：可参考“主管线派发、开发线执行、回交复核”的架构模式；不能硬编码为工作台事实模型。
- `docs/research/2026-06-05-paseo-workbench-deep-reference-research-v1.md`：只作为多 agent runtime、timeline、provider adapter、worktree、schedule / loop、CLI parity 的参考。
- `docs/research/2026-06-05-odysseus-workbench-deep-reference-research-v2.md`：只作为 capability registry、workspace confinement、tool risk、diagnostics 和运行边界参考。
- `docs/research/2026-06-05-gepa-workbench-deep-reference-research-v2.md`：只作为后置优化层参考；不进入 H / I 默认实现。

## 2. 当前事实

已完成：

- C1-C6：受控自动化工作流闭环。已具备方案授权、用户确认、全局边界复核、项目主管拆任务、prepared dispatch、worker 汇报、过程事实确认和最终复核链路。
- M1-M13：记忆系统权威验收已完成，结论为 `accepted_with_deferred_items`。正式记忆、候选、观察、任务记忆包、lint、生命周期、实体关系、维护任务和成熟模式已经能作为 H 阶段真实执行的依赖。
- E1-E7：adapter descriptor、会话操作边界、provider availability、continuation preview、Level A stub、Level B 单 session 健康探针、runtime attention 和阶段 E acceptance freeze 已完成。
- F1-F5：项目工作流画布产品化验收已完成。
- G1-G5：runtime log 最小底座、diagnostics 最小底座、部分真实 Tauri 截图证据、离线端到端回放和中间版本最终冻结已完成。
- H0-H1：阶段 H 安全边界冻结和 CodexLocalRunner 架构 / 数据契约已完成并已通过全局主管复核。
- H2：通用真实 resume 产品化任务包已创建并已完成 Phase B `mario test` 真实 resume 产品化探针：`tasks/2026-06-07-stage-h-h2-general-real-resume-productization-v1.md`，记录见 `evidence/2026-06-08-stage-h-h2-phase-b-mario-test-real-resume-productization-probe-v1.md` 与 `handoffs/2026-06-08-stage-h-h2-phase-b-mario-test-real-resume-productization-probe-v1-result.md`。H2.0 real resume preflight authorization guard 已完成，记录见 `evidence/2026-06-07-stage-h-h2-0-real-resume-preflight-authorization-guard-v1.md` 与 `handoffs/2026-06-07-stage-h-h2-0-real-resume-preflight-authorization-guard-v1-result.md`。H2.1 real resume authorization matrix and execution decision freeze 已完成，记录见 `evidence/2026-06-07-stage-h-h2-1-real-resume-authorization-matrix-and-execution-decision-freeze-v1.md` 与 `handoffs/2026-06-07-stage-h-h2-1-real-resume-authorization-matrix-and-execution-decision-freeze-v1-result.md`。H2.2 real resume authorization readiness read model and readonly UI 已完成，记录见 `evidence/2026-06-07-stage-h-h2-2-real-resume-authorization-readiness-read-model-and-readonly-ui-v1.md` 与 `handoffs/2026-06-07-stage-h-h2-2-real-resume-authorization-readiness-read-model-and-readonly-ui-v1-result.md`。H2.3 real resume request builder and CodexLocal guard bridge 已完成，记录见 `evidence/2026-06-07-stage-h-h2-3-real-resume-request-builder-and-codex-local-guard-bridge-v1.md` 与 `handoffs/2026-06-07-stage-h-h2-3-real-resume-request-builder-and-codex-local-guard-bridge-v1-result.md`。H2.4 real resume execution authorization and fixture freeze 已完成，记录见 `evidence/2026-06-07-stage-h-h2-4-real-resume-execution-authorization-and-fixture-freeze-v1.md` 与 `handoffs/2026-06-07-stage-h-h2-4-real-resume-execution-authorization-and-fixture-freeze-v1-result.md`。H2.5 real resume runner execution path and authorized fixture run Phase A 已完成，记录见 `evidence/2026-06-07-stage-h-h2-general-real-resume-productization-v1.md` 与 `handoffs/2026-06-07-stage-h-h2-general-real-resume-productization-v1-result.md`。H2.6 Phase B readiness / fixture session binding / runtime log hardening 已完成，记录见 `evidence/2026-06-07-stage-h-h2-6-phase-b-readiness-fixture-session-and-runtime-log-hardening-v1.md` 与 `handoffs/2026-06-07-stage-h-h2-6-phase-b-readiness-fixture-session-and-runtime-log-hardening-v1-result.md`。H2.7 Phase B authorization / fixture / target session confirmation 已完成为当时的授权准备复核和阻断状态冻结，记录见 `evidence/2026-06-07-stage-h-h2-7-phase-b-authorization-fixture-and-target-session-confirmation-v1.md` 与 `handoffs/2026-06-07-stage-h-h2-7-phase-b-authorization-fixture-and-target-session-confirmation-v1-result.md`。H2 Phase B 探针确实执行真实 `codex exec resume`、发送固定安全 probe prompt、写入 `/Users/yoyi/.codex`，并写入 continuation / runtime log / audit / readback；但不接受为 H3、H5、阶段 H、任意项目无限制执行、planned adapters 或 provider/model verification 完成。
- H2.8：real execution permission dialog / audit summary / readiness decision surface 已完成：`tasks/2026-06-07-stage-h-h2-8-real-execution-permission-dialog-audit-summary-and-readiness-decision-surface-v1.md`，记录见 `evidence/2026-06-07-stage-h-h2-8-real-execution-permission-dialog-audit-summary-and-readiness-decision-surface-v1.md` 与 `handoffs/2026-06-07-stage-h-h2-8-real-execution-permission-dialog-audit-summary-and-readiness-decision-surface-v1-result.md`。它用于补齐 H2 Phase B 真实 resume 前的权限弹层预览、审计摘要、runtime log preview、readback 边界、duplicate guard 和 readiness 决策面；H2.8 自身不授权真实 `codex exec resume`，不发送 prompt，不创建 fixture，后续 H2 Phase B 真实探针已单独授权并完成。
- H3-A：new session authorization / fixture / boundary freeze 已完成：`tasks/2026-06-07-stage-h-h3-a-new-session-authorization-fixture-and-boundary-freeze-v1.md`；记录见 `evidence/2026-06-07-stage-h-h3-a-new-session-authorization-fixture-and-boundary-freeze-v1.md` 与 `handoffs/2026-06-07-stage-h-h3-a-new-session-authorization-fixture-and-boundary-freeze-v1-result.md`。只接受为 H3 通用真实 send / 新会话的非执行授权冻结和边界设计；不授权真实 `codex exec`，不发送 prompt，不创建真实 Codex session，不读写 `/Users/yoyi/.codex`。
- H3.1：new session request / guard / permission envelope / no-op runner 已完成并已通过全局主管复核：`tasks/2026-06-07-stage-h-h3-1-new-session-request-guard-permission-envelope-and-noop-runner-v1.md`；记录见 `evidence/2026-06-07-stage-h-h3-1-new-session-request-guard-permission-envelope-and-noop-runner-v1.md` 与 `handoffs/2026-06-07-stage-h-h3-1-new-session-request-guard-permission-envelope-and-noop-runner-v1-result.md`。只接受为 `new_session` 非执行产品路径完成；不授权真实 `codex exec`，不发送 prompt，不创建真实 Codex session，不读写 `/Users/yoyi/.codex`。
- H3-B：final approval / real new session fixture run 已执行一次隔离 fixture 真实 probe：`tasks/2026-06-07-stage-h-h3-b-real-new-session-final-approval-and-fixture-run-v1.md`，记录见 `evidence/2026-06-07-stage-h-h3-b-real-new-session-final-approval-and-fixture-run-v1.md` 与 `handoffs/2026-06-07-stage-h-h3-b-real-new-session-final-approval-and-fixture-run-v1-result.md`。本次确实执行真实 `codex exec`、发送 prompt、写入 `/Users/yoyi/.codex`，但 attempt failed，readback failed / `result_count=null`，未成功创建可接受的新会话；产品路径已补 `--skip-git-repo-check`，但未二次真实执行。H3-B 当前不接受为成功、真实新会话创建成功、H3 产品化完成或阶段 H 完成；任何 retry 必须再次取得执行点授权。
- H4：readback / failure / timeout / duplicate guard 产品化任务包已完成 Level A 非真实产品化：`tasks/2026-06-08-stage-h-h4-readback-failure-timeout-and-duplicate-guard-productization-v1.md`，记录见 `evidence/2026-06-08-stage-h-h4-readback-failure-timeout-and-duplicate-guard-productization-v1.md` 与 `handoffs/2026-06-08-stage-h-h4-readback-failure-timeout-and-duplicate-guard-productization-v1-result.md`。接受为 readback / failure / timeout / duplicate guard / stale cleanup 统一产品边界完成；不执行真实 Codex、不发送 prompt、不创建 fixture run、不读写 `/Users/yoyi/.codex`。真实失败 / 超时探针如需执行，必须作为 H4-Level-B 单独取得执行点授权。
- H5：project workflow real dispatch integration 已完成 Level A 非真实产品路径集成并通过全局主管复核：`tasks/2026-06-08-stage-h-h5-project-workflow-real-dispatch-integration-v1.md`，开发记录见 `evidence/2026-06-08-stage-h-h5-project-workflow-real-dispatch-integration-level-a-v1.md` 与 `handoffs/2026-06-08-stage-h-h5-project-workflow-real-dispatch-integration-level-a-v1-result.md`，主管复核见 `evidence/2026-06-08-stage-h-h5-supervisor-acceptance-review-v1.md` 与 `handoffs/2026-06-08-stage-h-h5-supervisor-acceptance-review-v1-result.md`。当前只接受为 H5-Level-A 后端预览 / 校验链路完成；H5-Level-B 授权与 fixture freeze 已完成：`tasks/2026-06-08-stage-h-h5-level-b-project-workflow-real-dispatch-authorization-and-fixture-freeze-v1.md`；H5-Level-B1 执行任务包已完成一次 `/Users/yoyi/Documents/mario test` 开发线 worker session `019e798a-ac37-7771-b982-e38084fcd22e` 的 `resume` read-only 真实 probe：`tasks/2026-06-08-stage-h-h5-level-b1-mario-test-project-workflow-real-dispatch-run-v1.md`，记录见 `evidence/2026-06-08-stage-h-h5-level-b1-mario-test-project-workflow-real-dispatch-run-v1.md` 与 `handoffs/2026-06-08-stage-h-h5-level-b1-mario-test-project-workflow-real-dispatch-run-v1-result.md`，主管复核见 `evidence/2026-06-08-stage-h-h5-level-b1-supervisor-acceptance-review-v1.md` 与 `handoffs/2026-06-08-stage-h-h5-level-b1-supervisor-acceptance-review-v1-result.md`。H5-Level-B2 写入型 probe 已完成一次 `/Users/yoyi/Documents/mario test` 开发线 worker session 的 `resume` workspace-write 真实 probe：`tasks/2026-06-08-stage-h-h5-level-b2-mario-test-project-workflow-write-probe-v1.md`，记录见 `evidence/2026-06-08-stage-h-h5-level-b2-mario-test-project-workflow-write-probe-v1.md` 与 `handoffs/2026-06-08-stage-h-h5-level-b2-mario-test-project-workflow-write-probe-v1-result.md`。B1 和 B2 均不接受为 H5 通用产品化、H3-B 成功、planned adapters 真实接入、provider/model 验证或阶段 H 完成。

未完成 / deferred：

- 通用真实 send / resume 产品化。
- 从项目工作流任务包到真实 `codex-local` 执行再到 readback / worker report 的可重复闭环。
- 自动重试、取消、恢复、重复派发保护的完整产品能力。
- G3 全量真实 Tauri 截图验收。
- planned adapters 真实接入。
- provider credential store / model verification。
- 多 agent / 多模型中立协作抽象的产品化。

当前假设：

- H 阶段先做 `codex-local`，不接 planned adapters 的真实执行。
- H 阶段可以在明确任务包、权限弹层、审计、运行日志和用户确认下读写 `/Users/yoyi/.codex`，但不能使用临时脚本或人工口头约定绕过产品路径。
- I 阶段先做 adapter-neutral 协议和协作模型，再考虑某个非 Codex adapter 的真实接入。
- 记忆层已足以支撑 H 阶段任务记忆包注入，但 H 必须在真实执行链路里重新验证“使用了哪些记忆、排除了哪些记忆、是否被 lint 阻断”。

## 3. H / I 和中间版本的关系

中间版本 G5 的结论不改：`accepted_with_deferred_items`。

H / I 是 G5 后续阶段，不是补写 G5，也不是推翻中间版本结论。

阶段 H 的目标是把 deferred 中最影响可用性的能力先产品化：

```text
项目方案 / 任务包 / 记忆包
-> 权限预览
-> 用户确认
-> codex-local 真实 send / resume
-> 运行日志
-> readback
-> worker 结构化汇报
-> 项目主管过程确认
-> observation / candidate / formal memory
-> 全局主管复核
```

阶段 I 的目标是把 H 中验证过的 `codex-local` 执行链路抽象成多 agent / 多模型协作协议：

```text
WorkerAdapter
-> WorkThread / RunUnit
-> DispatchRequest
-> PermissionEnvelope
-> WorkerHandoff
-> ReadbackResult
-> RuntimeLog
-> AuditEvent
```

## 4. 阶段 H：Codex-local 真实自动化工作流产品化

### 4.1 阶段目标

把 `codex-local` 从“descriptor + preview + stub + 单 session 健康探针”推进到可在工作台内受控使用的真实工作流执行能力。

H 阶段完成后，应当能够：

- 用户在项目方案和任务包范围内批准真实执行。
- 项目主管在授权范围内派发真实 `codex-local` worker。
- 工作台能真实发送 prompt 或 resume 目标 session。
- 每次真实执行都有 continuation record、runtime log、audit event、permission envelope、readback status 和失败分类。
- readback 成功时能进入 worker structured report 或等价结果摘要。
- readback 失败 / 超时 / 被 guard 阻断 / 用户拒绝时，UI 能清楚解释状态和下一步，不伪装成 0 条结果。
- 任务记忆包真实参与执行链路，并能解释 included / excluded / review materials。

### 4.2 非目标

H 不做：

- Claude Code / OpenClaw / OpenCode / OpenCode-like 真实接入。
- provider credential store 或模型验证。
- 跨 provider 调度。
- agent 自治创建线程、自治批准权限或自治写正式记忆。
- 自由聊天式任意 send/resume。
- 绕过项目、方案授权、任务包、记忆包、权限、运行日志和审计的裸 Codex 控制器。
- 自动技能化、GraphRAG、向量库、图数据库或 Obsidian 原生同步。

### 4.3 H 阶段任务拆分

#### H0：阶段 H 安全边界和任务包冻结

目的：

- 冻结 H 阶段真实执行权限、路径、fixture、测试项目、任务包顺序和失败降级规则。

必须产出：

- H0 任务包。
- H 阶段安全边界文档或任务包固定章节。
- 明确哪些 H 任务允许真实 `codex exec` / `codex exec resume`。
- 明确哪些 H 任务允许读写 `/Users/yoyi/.codex`。
- 明确测试项目、允许写入路径、禁止路径、备份 / 回滚策略。
- 明确 UI 显示边界和真实 Tauri 验收要求。

验收：

- `CURRENT.md` / `tasks/README.md` 能指向 H0。
- H0 不改产品代码。
- H0 不执行真实 Codex。

#### H1：CodexLocalRunner 架构和数据契约

目的：

- 把 E4/E5 的 continuation preview / stub / attempt 结构收敛成正式 `CodexLocalRunner` 或等价应用服务契约。

必须产出：

- `CodexLocalExecutionRequest`
- `CodexLocalExecutionGuard`
- `CodexLocalExecutionAttempt`
- `CodexLocalReadbackPlan`
- `CodexLocalReadbackResult`
- `CodexLocalFailureReason`
- `CodexLocalRuntimeLogRef`
- `CodexLocalAuditRef`

约束：

- 前端不能直接拼 CLI 命令。
- 后端应用服务负责 guard、命令构造、路径校验、审计和运行日志。
- CLI 参数必须结构化，不能用字符串拼接绕过 shell 安全边界。
- 不能读取 secret、auth token、`.env`、完整 transcript 或 provider credential。

验收：

- 有 Rust / TS 类型和后端 guard 单测。
- 没有真实执行。
- 没有读写 `/Users/yoyi/.codex`。

当前记录：

- H1 已完成并已通过全局主管复核：`tasks/2026-06-07-stage-h-h1-codex-local-runner-architecture-and-data-contract-v1.md`。
- 证据：`evidence/2026-06-07-stage-h-h1-codex-local-runner-architecture-and-data-contract-v1.md`。
- 交接：`handoffs/2026-06-07-stage-h-h1-codex-local-runner-architecture-and-data-contract-v1-result.md`。
- H1 只接受为类型、guard、runner 契约、fake dry-run、结构化 argv 计划、prompt stdin ref/hash、runtime log / audit / readback 分离边界和单测完成；不接受为 H2/H3、真实执行、prompt 发送、真实 readback 或 planned adapters 接入。

#### H2：通用真实 resume 产品化

目的：

- 把 E5 Level B 的单 session 健康探针升级为工作台受控的通用真实 resume 能力。

必须产出：

- 从项目 / 工作流 / 任务包 / 目标 session 派生 resume request。
- 用户确认弹层展示目标 session、project root、cwd、允许写入根、prompt summary、任务记忆包摘要、风险和 readback 预期。
- 后端执行真实 `codex exec resume`。
- 写 continuation record、runtime log、audit event 和 readback result。
- 支持成功、失败、超时、被 guard 阻断、用户拒绝、readback unavailable。
- 支持重复派发保护。

执行权限：

- H2 任务包必须在执行前明确列出测试项目、目标 session、允许写入路径和 `.codex` 读写范围。
- 任何真实 resume 都必须由用户通过工作台确认或任务包级明确授权触发。

验收：

- 至少一个隔离测试项目完成真实 resume。
- 目标项目文件变化符合任务要求或明确无变化。
- runtime log 与 audit 能追溯。
- readback 不可用时不能显示为 0 条结果。

当前记录：

- H2 任务包已完成到 Phase B `mario test` 真实 resume 产品化探针：`tasks/2026-06-07-stage-h-h2-general-real-resume-productization-v1.md`。
- 本次 H2 Phase B 探针已在 2026-06-08 对 `/Users/yoyi/Documents/mario test` session `019e798a-6ce5-76c3-b8ee-33bd0fda841f` 执行真实 `codex exec resume`，readback 标记为 `H2_PHASE_B_MARIO_TEST_REAL_RESUME_OK_2026_06_08`。
- H2.0 已完成执行前授权矩阵 guard / blocked attempt / audit 底座；授权矩阵完整时也只返回 `complete_but_not_executed`，不调用真实 runner。
- H2.1 已完成执行前授权矩阵和决策工作表，推荐隔离 fixture 路径、待用户确认项、禁止假设项和停止条件已冻结。
- H2.2 已完成执行前授权准备读模型和智能体页只读 UI，把授权矩阵缺口可视化，但不调用 H2.0 Tauri command，不写 sidecar，不新增执行按钮。
- H2.3 已完成 request builder 和 CodexLocal guard bridge：授权矩阵完整时会构建 H1 `CodexLocalExecutionRequest` 并执行 guard inspection；guard 允许时仍只返回 `complete_but_not_executed`，不调用真实 runner。
- H2.4 已完成真实执行授权包和 fixture freeze：推荐 fixture、target session、`.codex` 最小范围、prompt summary/ref/hash、allowed write roots、readback plan、runtime log、audit、evidence 和 rollback 的审批项已冻结。
- H2.5 Phase A 已完成：已实现并测试非执行产品路径；后续 Phase B 已在 `mario test` 上单独授权并完成一次真实探针。
- H2.6 已完成：runtime log writer 已补齐为显式 sidecar 写入；接受为当时前置条件复核和阻断状态冻结。
- H2.7 已完成：接受为当时 Phase B authorization preparation review、缺 existing target session 阻断冻结、fixture / permission envelope secondary blockers 冻结。
- H2.8 已完成并回收：接受为权限弹层预览、审计摘要、runtime log preview、readback 边界、duplicate guard 和 readiness 决策面加固；后续 H2 Phase B 已单独授权并完成一次真实探针。
- H3.1 已完成：只接受为 `new_session` request、guard、permission envelope、command plan preview、no-op runner 和只读 UI 完成；H3.1 不授权真实执行。
- H3-B 已执行一次隔离 fixture 真实 new-session probe，但因当时 command plan 缺少 `--skip-git-repo-check` 失败并完成分类；产品路径已修补但未二次真实执行。H3-B 不接受为成功，任何 retry 仍不能继承 H2 Phase B 授权。
- H2 Phase B 成功不接受为真实新会话已创建、H3-B 真实新会话可直接开始、H5 项目工作流真实派发或阶段 H 完成。

#### H3：通用真实 send / 新会话产品化

目的：

- 建立受控新建 Codex 会话 / send message 能力，解决只有 resume 不足以创建新 worker 的问题。

必须产出：

- 新会话创建 request。
- 项目绑定、角色绑定、cwd、sandbox、allowed write roots、prompt summary 和 readback plan。
- 后端执行真实 `codex exec` 或等价新会话命令。
- 新 session 进入工作台会话目录和项目绑定读模型。
- 写 continuation record、runtime log、audit event。

约束：

- 不能开放自由聊天输入框绕过任务包。
- 新会话必须绑定项目、角色、任务包和授权范围。
- 不能默认继承 secret、`.env`、token 或 provider credential。

验收：

- 至少一个隔离测试项目完成真实新会话创建。
- 会话能在智能体页和项目工作流节点中被追踪。
- readback、失败和审计可见。

#### H4：readback、失败、超时、取消和重复派发保护

目的：

- 把真实执行的不确定性产品化，不再靠人工解释。

必须产出：

- `ReadbackFailureReason` 分类。
- timeout policy。
- cancel / stop 边界。
- duplicate dispatch guard。
- stale run cleanup 规则。
- runtime attention 与 G1/G2 diagnostics 联动。

约束：

- cancel / stop 不能默认为 kill。
- 自动重试必须先做 preview 和用户确认，不能静默重试。
- readback unavailable / failed / timed out 必须保持和真实 0 条结果区分。

验收：

- 覆盖成功、超时、失败、重复派发、readback unavailable。
- UI 的运行中 / 通知 / 待办分开显示。
- 管理入口可看到脱敏 runtime log 和 diagnostic summary。

当前记录：

- H4 Level A 非真实产品化已完成：`tasks/2026-06-08-stage-h-h4-readback-failure-timeout-and-duplicate-guard-productization-v1.md`。
- H4 已收敛现有 H2/H3.1/G1/G2 类型、store、runtime log、diagnostics、read model 和测试；本轮未改 UI。
- H4 不依赖 H3-B 已完成，也不代表阶段 H 已完成。
- unknown-result 状态保持 `result_count=null`；duplicate blocked 写 attempt / audit / runtime log 且不调用 runner；stale cleanup 要求 expected revision，只处理工作台自有 active attempt，不 kill Codex、不自动 retry。
- H4-Level-B 真实失败 / 超时探针必须另行授权，不得继承 H2 Phase B 或 H3-B 授权。

#### H5：项目工作流真实派发集成

目的：

- 把 C4 prepared dispatch、M6 task memory injection、E5 continuation、G1 runtime log 和 F1-F5 项目画布串成真实执行链路。

必须产出：

- prepared dispatch -> user confirmation -> real dispatch -> readback -> worker report -> project director process fact decision。
- 项目画布节点状态、节点详情、运行中、通知、待办、管理入口同步更新。
- 任务记忆包 included / excluded / lint / review materials 进入真实 dispatch evidence。

约束：

- React Flow 仍只是渲染，不是事实源。
- worker report 不能直接写正式事实。
- observation / candidate / formal memory 仍按记忆状态机推进。

验收：

- 至少一个隔离项目跑通真实项目工作流最小闭环。
- evidence / handoff 能追溯每一步。
- 失败路径不阻断后续复核和用户理解。

#### H6：真实执行 UI 产品化和 Tauri 验收

当前记录：

- H6 已完成 checkpoint：`tasks/2026-06-08-stage-h-h6-real-execution-ui-productization-and-tauri-acceptance-v1.md`。
- 记录见 `evidence/2026-06-08-stage-h-h6-real-execution-ui-productization-and-tauri-acceptance-v1.md` 与 `handoffs/2026-06-08-stage-h-h6-real-execution-ui-productization-and-tauri-acceptance-v1-result.md`。
- 结论为 `accepted_with_deferred_items`：接受真实执行状态 UI 产品化、权限弹层边界、unknown readback 文案修补、前端 / Rust 边界验证和真实 Tauri 窗口探针；不接受真实 Tauri H6 关键截图清单完整完成或阶段 H 完成。

目的：

- 让用户在工作台里看得懂真实执行，而不是看日志猜状态。

必须产出：

- 智能体页：真实 send / resume 状态、目标 session、readback、失败、runtime log 引用。
- 项目页：工作流节点真实执行状态、任务包摘要、记忆包摘要、权限和结果。
- 右侧入口：运行中 / 通知 / 待办保持分离。
- 管理入口：运行日志、诊断、审计和数据位置脱敏展示。
- 权限弹层：说明做什么、为什么、影响哪里、谁提出、批准后发生什么、失败如何处理。

验收：

- 必须按 `docs/workbench-frontend-display-boundary-v1.md` 和 `docs/plans/task-package-ui-display-boundary-rule-v1.md` 执行。
- 涉及 UI 的任务必须有真实 Tauri 或明确降级的截图证据。
- 普通浏览器 smoke 不能冒充真实 Tauri 验收。

#### H7：H 阶段最终验收和冻结

目的：

- 对 H1-H6 做全局主管总复核，冻结可接受项、deferred 项和 I 阶段前置条件。

必须产出：

- H acceptance matrix。
- H evidence / handoff。
- H-to-I handoff。
- 权威入口同步。

验收：

- 通用真实 resume 产品化完成与否有证据。
- 通用真实 send / 新会话产品化完成与否有证据。
- 项目工作流真实派发闭环完成与否有证据。
- 不把单次 demo 冒充产品化。

## 5. 阶段 I：多 agent / 多模型协作中立抽象

### 5.1 阶段目标

在 H 阶段验证过 `codex-local` 真实执行链路之后，把协作模型抽象成不依赖 Codex 的工作台自有协议，为后续 Claude Code、OpenClaw、OpenCode、OpenCode-like、模型 provider 和其他 agent runtime 接入打底。

I 阶段完成后，应当能够：

- 工作台用同一套 `WorkerAdapter` / `RunUnit` / `DispatchRequest` / `WorkerHandoff` / `ReadbackResult` 描述不同 agent 产品。
- 项目主管通过控制核心派发 worker，而不是由某个 agent 自治创建子线程。
- 权限、任务包、记忆包、runtime log、diagnostics、audit 和 readback 不依赖 Codex 内部格式。
- planned adapters 仍可保持不可执行，直到后续独立任务完成凭据、模型、运行沙箱和真实验收。

### 5.2 Codex 多线程协作参考规则

可学习：

- 主管线负责拆解、派发、回收、复核。
- 开发线负责单任务执行。
- 任务完成后回交给主管线。
- 主管线统一判断是否接受、要求修补或继续派发。
- 每条线有明确职责、上下文和结果边界。

不能照搬：

- 不能把 Codex thread id 当成工作台永久业务主键。
- 不能把 Codex 的线程生命周期当成工作台 workflow state。
- 不能让 Codex 线程关系直接决定项目主管 / worker / 验证线 / 回收线关系。
- 不能让 Codex 自带协作能力绕过控制核心、权限、任务包、记忆包、审计和用户确认。
- 不能把 Codex-only 的 send / resume UI 设计成所有 agent 的通用 UI。

工作台必须拥有：

- 项目事实。
- 工作流状态。
- 任务包。
- 角色和职责。
- 权限 envelope。
- 运行日志。
- 审计事件。
- 记忆和知识引用。
- 回交 / readback 协议。

### 5.3 I 阶段任务拆分

#### I0：Codex 多线程协作参考复核和抽象映射

目的：

- 从 Codex 多线程协作能力中提取可借鉴的模式，不把实现细节写进产品模型。

必须产出：

- Reference mapping：Codex thread / subagent / handoff / director review 对应到工作台概念。
- Adopt / reject / defer 表。
- I 阶段协议对象草案。

验收：

- 没有产品代码。
- 没有执行真实 Codex。
- 没有读写 `/Users/yoyi/.codex`。

#### I1：WorkerAdapter 和 RunUnit 中立模型

目的：

- 建立 adapter-neutral 的 worker / run 抽象。

必须产出：

- `WorkerAdapterDescriptor`
- `WorkerCapabilityDescriptor`
- `WorkThread`
- `RunUnit`
- `RunLifecycleStatus`
- `RunAttention`
- `RunPersistenceHandle`

约束：

- `codex-local` 是第一个实现映射，不是模型中心。
- planned adapters 可以声明能力和不可用原因，但不能显示为可执行。

验收：

- 类型、读模型和单测完成。
- 不接真实外部 provider。

#### I2：DispatchRequest / PermissionEnvelope / WorkerHandoff 协议

目的：

- 把任务派发、权限、回交和 readback 变成中立协议。

必须产出：

- `DispatchRequest`
- `DispatchGuardResult`
- `PermissionEnvelope`
- `TaskMemoryPacketRef`
- `WorkerHandoff`
- `ReadbackResult`
- `WorkerReportCandidate`

约束：

- dispatch 必须来自项目主管任务包和控制核心。
- handoff 只能成为 process fact / observation / candidate 的来源，不能直接写正式事实或正式记忆。

验收：

- H 阶段 `codex-local` 执行链路能映射到新协议。
- E/F/G 既有读模型不被破坏。

#### I3：Capability、provider、credential 和风险 envelope 对齐

目的：

- 为后续多 provider / 多模型真实接入建立边界，但不在 I3 里直接接入。

必须产出：

- capability risk class。
- provider availability extension。
- credential requirement descriptor。
- external call / cost / data egress risk。
- project-level allowed capability policy。

约束：

- 技术可用不等于项目授权可用。
- 不能读取真实 token、auth、`.env`、keychain、OAuth 或 provider credential。

验收：

- planned adapters 仍是 planned / unavailable / credential missing / model unverified。
- UI 不出现未实现执行按钮。

#### I4：多 worker 编排和项目工作流集成

目的：

- 让项目主管能够用中立协议组织多个 worker，而不是硬编码多个 Codex 线程。

必须产出：

- parent / child / sibling / detached run 关系。
- multi-worker dispatch plan。
- verifier / reviewer / recovery lane 抽象。
- project workflow canvas 显示多个 run 的状态。

约束：

- 多 worker 关系必须来自项目主管任务包和控制核心。
- 不能让 agent 自治 spawn、kill、archive 或 approve worker。
- verifier 结果不能自动变正式事实。

验收：

- 至少用 `codex-local` 或 stub adapter 回放多 worker 协作。
- 不接 planned adapters 真实执行。

#### I5：Adapter SDK / CLI parity 和运维诊断预留

目的：

- 让未来 adapter 接入有一致的后端接口、测试方式和诊断方式。

必须产出：

- adapter contract checklist。
- controlled API / CLI semantics。
- diagnostic event schema。
- adapter health。
- degraded mode。
- data location / persistence handle。

约束：

- CLI parity 也必须经过控制核心、权限和审计。
- 不开放万能 app API 后门。

验收：

- 新 adapter 接入前能用 checklist 判断是否满足安全边界。
- 管理入口能解释 adapter unavailable / credential missing / model unverified / sandbox blocked。

#### I6：I 阶段最终验收和后续 adapter 路线冻结

目的：

- 冻结中立协作抽象是否足以支持后续 planned adapters 产品化。

必须产出：

- I acceptance matrix。
- Adapter readiness matrix。
- planned adapter 后续任务建议。
- I-to-next-stage handoff。

验收：

- 不把抽象完成说成 Claude Code / OpenClaw / OpenCode 已接入。
- 不把 capability descriptor 说成真实执行能力。
- 不把 provider availability 说成 credential / model 已验证。

## 6. UI 显示边界

H / I 涉及 UI 时必须执行：

- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`

固定规则：

- 一级入口仍是项目 / 智能体 / 画布 / 记忆 / 知识库 / 设置。
- 右侧入口仍是秘书 / 通知 / 待办 / 运行中 / 管理。
- 运行中显示正在做什么、是否卡住、是否需要权限。
- 通知显示发生了什么。
- 待办显示用户需要处理什么。
- 管理显示运行日志、诊断、审计、权限、健康状态和数据位置。
- 智能体页可以显示会话、adapter、operation boundary、send / resume 状态和 readback，但不能变成绕过项目工作流的自由控制台。
- 项目页必须保持项目主管工作流为主，不把任务包管理器、日志管理器或 schema 管理器推到普通用户主界面。

截图规则：

- 涉及真实执行 UI、权限弹层、项目工作流状态、智能体页 send / resume、运行中 / 通知 / 待办 / 管理入口的任务，必须做真实 Tauri 截图验收或明确记录未完成。
- 普通浏览器 smoke 只能作为辅助证据，不能替代真实 Tauri。

## 7. 安全边界

默认禁止：

- 未经任务包和用户确认执行真实 `codex exec` / `codex exec resume`。
- 未经授权读写 `/Users/yoyi/.codex`。
- 读取 `auth.json`、token、`.env`、keychain、OAuth、provider credential 或完整 transcript。
- 使用 shell 字符串拼接构造高风险命令。
- 使用 `--dangerously-bypass-approvals-and-sandbox`。
- 让 UI、Markdown、日志、readback、worker report 或 LLM summary 绕过控制核心。
- 把 worker report、tool output、timeline、readback 或 verifier report 直接写正式事实或正式记忆。
- 把 planned adapter、provider availability、capability descriptor 说成真实可执行。

H 阶段允许在任务包明确授权后做：

- 真实 `codex exec resume`。
- 真实 `codex exec`。
- 工作台受控读写必要的 Codex session / index / transcript metadata。
- 将执行结果写入工作台自有 continuation record、runtime log、audit 和 workflow state。

但必须满足：

- 有项目、任务包、目标 session 或新 session、cwd、sandbox、allowed write roots、prompt summary、readback plan。
- 有用户确认或任务包级明确授权。
- 有运行日志和审计。
- 有失败降级和回滚说明。

## 8. 记忆层依赖和验收要求

H 阶段不能绕过记忆层：

- 真实 dispatch 前必须能看到任务记忆包 included / excluded / review materials。
- candidate / knowledge hit 不能说成正式记忆。
- lint blocking 必须阻断或进入明确用户确认。
- worker report 只能先进入 process fact confirmation / observation / candidate 链路。
- 正式记忆写入仍必须有来源、版本、权限、审计和用户确认。

I 阶段不能让 adapter 写记忆：

- adapter 只能返回结果、摘要、证据引用、错误或候选。
- 是否进入 observation / candidate / formal memory 由控制核心和记忆状态机决定。

## 9. 禁止过度声明

不能声明：

- H0 / H1 完成后就“通用真实 send / resume 已完成”。
- H2 单次成功后就“真实 Codex 自动工作流产品化完成”。
- H3 新会话成功后就“多 agent 工作流完成”。
- H5 单项目跑通后就“复杂业务自动编排完成”。
- I1 / I2 抽象完成后就“Claude Code / OpenClaw / OpenCode 已接入”。
- provider availability 完成后就“凭据已配置 / 模型已验证”。
- runtime log 存在后就“自动重试 / 自动恢复完成”。
- 普通浏览器 smoke 后就“真实 Tauri 验收完成”。

## 10. 推荐执行顺序

第一批：

1. H0：阶段 H 安全边界和任务包冻结。
2. H1：CodexLocalRunner 架构和数据契约。
3. H2：通用真实 resume 产品化。

第二批：

1. H3：通用真实 send / 新会话产品化。
2. H4：readback、失败、超时、取消和重复派发保护。
3. H5：项目工作流真实派发集成。

第三批：

1. H6：真实执行 UI 产品化和 Tauri 验收。
2. H7：H 阶段最终验收和冻结。

第四批：

1. I0：Codex 多线程协作参考复核和抽象映射。已完成。
2. I1-I2 合并 checkpoint：WorkerAdapter / WorkThread / RunUnit 中立模型 + DispatchRequest / PermissionEnvelope / WorkerHandoff 协议。已完成。
3. I3-I4 合并 checkpoint：capability / provider / credential 风险 envelope 对齐 + 多 worker 编排和项目工作流集成。已完成。
4. I5：Adapter SDK / CLI parity 和运维诊断预留。已完成。

第五批：

1. I3：Capability、provider、credential 和风险 envelope 对齐。
2. I4：多 worker 编排和项目工作流集成。
3. I5：Adapter SDK / CLI parity 和运维诊断预留。已完成。
4. I6：I 阶段最终验收和后续 adapter 路线冻结。已完成。

## 11. 当前下一步

当前 H-I 阶段已收口：

- I6：I5 已完成后已完成阶段 I 最终验收和后续 adapter 路线冻结，阶段 I 结论为 `accepted_with_deferred_items`。记录见 `tasks/2026-06-08-stage-i-i6-stage-i-final-acceptance-and-adapter-roadmap-freeze-v1.md`、`evidence/2026-06-08-stage-i-i6-stage-i-final-acceptance-and-adapter-roadmap-freeze-v1.md` 与 `handoffs/2026-06-08-stage-i-i6-stage-i-final-acceptance-and-adapter-roadmap-freeze-v1-result.md`。H-I 阶段整体收口为 `accepted_with_deferred_items`；后续如继续开发，应进入新的 adapter productization / multi-provider runtime 阶段。默认不授权真实 Codex 执行、`.codex` 读写、planned adapters 真实接入或 provider credential / model verification。
- H7 已完成 checkpoint：`tasks/2026-06-08-stage-h-h7-stage-h-final-acceptance-and-freeze-v1.md`；记录见 `evidence/2026-06-08-stage-h-h7-stage-h-final-acceptance-and-freeze-v1.md` 与 `handoffs/2026-06-08-stage-h-h7-stage-h-final-acceptance-and-freeze-v1-result.md`。H7 接受为 H0-H6 总复核、H acceptance matrix、deferred 项冻结和 H-to-I handoff；不接受 H3-B 成功、H4-Level-B、H6 全量 Tauri 截图、通用自由 Codex 控制台、planned adapters 真实接入或 provider/model verification。
- H3-B retry final approval / real new session fixture run：H3-A new session authorization / fixture / boundary freeze 已完成；H3.1 new session request / guard / permission envelope / no-op runner 已完成；H3-B 已执行一次隔离 fixture 真实 probe 并失败分类，产品路径已补 `--skip-git-repo-check`。任何 retry 必须等待用户 / 全局主管在执行点明确授权，且不能继承 H2 Phase B 或本次失败 probe 的授权。
- H4-Level-B 真实失败 / 超时探针：H4 Level A 非真实产品化已完成；如需真实失败 / 超时探针，必须另行授权 fixture、operation、allowed write roots、`.codex` 最小范围、prompt summary/ref/hash、timeout、readback、runtime log、audit、evidence、handoff、rollback 和停止条件。
- 后续 H2.x 修补任务：如全局主管复核发现 H2 真实 resume 产品路径仍需修补，可单独拆包；不得用修补任务冒充新的真实执行授权。

H0 / H1 已通过复核；H2.0-H2.8 已完成；H2 Phase B `mario test` 真实 resume 探针已完成。但仍不能在没有新的执行点授权时直接执行：

- 新的 H2 真实 resume。
- H3 真实 send / 新会话。
- 新的 H5 写入型 probe 或新的项目工作流真实派发执行任务；H5 product command formalization checkpoint 已完成，后续新的真实执行仍必须重新取得执行点授权。
- planned adapters 真实接入。
- provider credential / model verification。

H2 任务包必须首先确认：

- 测试项目使用哪个。
- 是否允许读写 `/Users/yoyi/.codex`。
- 是否允许执行真实 `codex exec` / `codex exec resume`。
- 真实执行截图和 evidence 保存路径。
- 失败时是否停止、回滚或降级记录。
- 多线程协作是否只作为任务执行组织方式，不作为产品模型事实源。
