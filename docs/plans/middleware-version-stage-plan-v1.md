# Middleware Version Stage Plan v1

> **状态校正（2026-08-09）：HISTORICAL / SUPERSEDED。** 本文只保存中间版本的阶段现场，正文中的“当前中间版本”和后续入口均已失效。当前产品方向看 `../product/syn-product-canon-v1.md`，当前计划看 `2026-08-01-syn-personal-ai-workbench-master-development-plan-v1.md`。

日期：2026-06-06

状态：当前中间版本整体阶段计划。本文承接 `docs/middleware-version-development-plan-v1.md` 和 `docs/plans/memory-layer-implementation-slice-v1.md`，用于指导后续任务包拆分和执行顺序。

## 0. 先说薄弱点

- 这不是单个实现任务包，不能直接让 agent 一次性全部开发。
- 自动化工作流和记忆层都很大，必须分阶段、分任务包推进。
- M1、M1.1、M2、M3、M4、M5、M6、M7、M8、M9、M10、M11、M12、M12.1 和 M13 已完成；C1、C2、C3、C4、C5 和 C6 已完成；阶段 C 可接受为受控自动化工作流闭环完成；阶段 D / 记忆系统最终权威验收结论为 `accepted_with_deferred_items`。
- 阶段 E / E1 已完成 adapter descriptor 执行边界和模型 / 凭据只读状态底座；E2 已完成会话操作边界契约；E3 已完成模型、凭据和 provider availability 只读边界；E4、E5 Level A、E5 Level B mario test 健康探针、E6 和 E7 已完成。阶段 E 总结论为 `accepted_with_deferred_items`。E5 Level B 只接受为指定 `/Users/yoyi/Documents/mario test` “总指导” session 的最小真实 resume 健康探针；这不代表 Claude Code / OpenClaw / OpenCode 已真实接入，也不代表真实 provider / 模型 / 凭据已验证或通用真实 send / resume 已验收。
- 画布、UI、模型、凭据、知识库和多 agent 接入都重要，但不能抢在控制核心、正式记忆和任务包召回之前制造新入口。
- M1 到 M6 只是第一条真实记忆闭环；中间版本记忆系统最终权威验收已在 M13 完成，但真实 Tauri 全面验收、运维日志和最终蓝图能力仍后置。
- G5 后续 H-I 计划已新增：`docs/plans/2026-06-07-stage-h-i-real-codex-automation-and-multi-agent-collaboration-plan-v1.md`。H-I 是中间版本冻结后的后续开发计划，不改变 G5 `accepted_with_deferred_items` 结论。H0 已完成文档冻结并已通过全局主管复核；H1 CodexLocalRunner 架构和数据契约已完成并已通过全局主管复核；H2 通用真实 resume 产品化任务包已创建并已完成 Phase B `mario test` 真实 resume 产品化探针；H2.0 real resume preflight authorization guard 已完成；H2.1 real resume authorization matrix and execution decision freeze 已完成；H2.2 real resume authorization readiness read model and readonly UI 已完成；H2.3 real resume request builder and CodexLocal guard bridge 已完成；H2.4 real resume execution authorization and fixture freeze 已完成；H2.5 real resume runner execution path and authorized fixture run Phase A 已完成；H2.6 Phase B readiness / fixture session binding / runtime log hardening 已完成，runtime log 显式 sidecar writer 已补齐；H2.7 Phase B authorization / fixture / target session confirmation 已完成为当时的授权准备复核和阻断状态冻结；H2.8 real execution permission dialog / audit summary / readiness decision surface 已完成并回收；后续 H2 Phase B 已在 2026-06-08 对 `mario test` 授权并完成一次真实探针；H3-A new session authorization / fixture / boundary freeze 已完成；H3.1 new session request / guard / permission envelope / no-op runner 已完成并已通过全局主管复核；H3-B final approval / real new session fixture run 已执行一次隔离 fixture 真实 probe 但失败分类完成，产品路径已补 `--skip-git-repo-check`，等待新的 retry 授权。H0/H1、H2.0-H2.8、H3-A、H3.1 和 H3-B 失败分类不代表 H3 真实新会话创建成功、H5 项目工作流真实派发、planned adapters 真实接入或阶段 H 完成。

## 1. 权威依据

必须服从：

- `/Users/yoyi/Documents/Codex/2026-05-26/gan-xing-codexbridge-https-github-com/docs/architecture/local-ai-workbench-blueprint-v1.md`
- `/Users/yoyi/Documents/Codex/2026-05-26/gan-xing-codexbridge-https-github-com/docs/architecture/local-ai-workbench-ui-blueprint-v1.md`
- `docs/middleware-version-development-plan-v1.md`
- `docs/memory-layer-design-v1.md`
- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/memory-layer-implementation-slice-v1.md`
- `docs/workbench-system-architecture-v1.md`
- `CURRENT.md`
- `tasks/README.md`

当前任务入口：

- M1 已完成：`tasks/2026-06-03-memory-layer-m1-formal-memory-store-and-audit-v1.md`
- M1.1 已完成：`tasks/2026-06-03-memory-layer-m1-1-formal-memory-context-binding-guard-v1.md`
- M2 已完成：`tasks/2026-06-03-memory-layer-m2-candidate-to-formal-adoption-v1.md`
- M3 已完成：`tasks/2026-06-04-memory-layer-m3-observation-store-and-workflow-entry-v1.md`
- M4 已完成：`tasks/2026-06-04-memory-layer-m4-task-memory-packet-builder-and-preview-v1.md`
- M5 已完成：`tasks/2026-06-04-memory-layer-m5-conflict-and-memory-lint-minimal-blocking-v1.md`
- M6 已完成：`tasks/2026-06-04-memory-layer-m6-workflow-task-package-injection-and-end-to-end-loop-v1.md`
- C1 已完成：`tasks/2026-06-04-workflow-c1-plan-authorization-and-controlled-auto-dispatch-foundation-v1.md`
- C2 已完成：`tasks/2026-06-04-workflow-c2-project-consultation-proposal-and-user-confirmation-entry-v1.md`
- C3 已完成：`tasks/2026-06-04-workflow-c3-global-boundary-review-and-authorization-activation-v1.md`
- C4 已完成：`tasks/2026-06-04-workflow-c4-project-director-task-decomposition-and-authorized-prepared-auto-dispatch-v1.md`
- C5 已完成：`tasks/2026-06-04-workflow-c5-worker-structured-report-process-fact-confirmation-and-failure-visibility-v1.md`
- C6 已完成：`tasks/2026-06-05-workflow-c6-global-final-result-review-user-result-view-and-stage-c-acceptance-v1.md`
- M7 已完成：`tasks/2026-06-05-memory-layer-m7-memory-management-ui-minimal-entry-v1.md`
- M8 已完成：`tasks/2026-06-05-memory-layer-m8-knowledge-base-and-obsidian-compatible-interface-placeholder-v1.md`
- M9 已完成：`tasks/2026-06-05-memory-layer-m9-formal-memory-lifecycle-operations-v1.md`
- M10 已完成：`tasks/2026-06-05-memory-layer-m10-entity-and-relation-governance-v1.md`
- M11 已完成：`tasks/2026-06-05-memory-layer-m11-maintenance-jobs-and-memory-lint-v1.md`
- M12 已完成：`tasks/2026-06-05-memory-layer-m12-mature-pattern-cross-project-memory-and-complete-acceptance-v1.md`
- M12.1 已完成：`tasks/2026-06-05-memory-layer-m12-1-acceptance-summary-freshness-after-mature-pattern-formalization-v1.md`
- M13 已完成：`tasks/2026-06-05-memory-layer-m13-final-authoritative-acceptance-and-conclusion-freeze-v1.md`
- E1 已完成：`tasks/2026-06-05-stage-e-e1-agent-adapter-descriptor-execution-boundary-and-model-credential-readonly-foundation-v1.md`
- E2 已完成：`tasks/2026-06-05-stage-e-e2-session-operation-boundary-contract-and-readonly-ui-v1.md`
- E3 已完成：`tasks/2026-06-06-stage-e-e3-model-credential-provider-availability-readonly-boundary-v1.md`
- E4 已完成：`tasks/2026-06-06-stage-e-e4-session-continuation-protocol-and-permission-preview-v1.md`
- E5 已完成 Level A：`tasks/2026-06-06-stage-e-e5-codex-local-controlled-send-resume-minimal-loop-v1.md`
- E6 已完成：`tasks/2026-06-06-stage-e-e6-runtime-session-attention-and-readback-failure-boundary-v1.md`
- E7 已完成：`tasks/2026-06-06-stage-e-e7-session-adapter-model-boundary-acceptance-v1.md`
- E5 Level B mario test 健康探针已完成：`tasks/2026-06-06-stage-e-e5-level-b-mario-test-controlled-real-resume-health-probe-v1.md`
- E/F/G 细化计划已完成：`docs/plans/2026-06-06-stage-e-f-g-refinement-plan-v1.md`
- F1-F5 已完成，阶段 F 最终结论为 `accepted_with_deferred_items`；G1、G2、G3-A、G3-C、G4 和 G5 已完成，G3-B 已回交但未完成，只接受为 10 / 13 真实 Tauri 部分截图证据
- H0 已完成并已通过全局主管复核：`tasks/2026-06-07-stage-h-h0-safety-boundary-and-task-package-freeze-v1.md`
- H1 已完成并已通过全局主管复核：`tasks/2026-06-07-stage-h-h1-codex-local-runner-architecture-and-data-contract-v1.md`
- H2 任务包已创建并已完成 Phase B `mario test` 真实 resume 产品化探针：`tasks/2026-06-07-stage-h-h2-general-real-resume-productization-v1.md`
- H2.0 real resume preflight authorization guard 已完成：`evidence/2026-06-07-stage-h-h2-0-real-resume-preflight-authorization-guard-v1.md`
- H2.1 real resume authorization matrix and execution decision freeze 已完成：`evidence/2026-06-07-stage-h-h2-1-real-resume-authorization-matrix-and-execution-decision-freeze-v1.md`
- H2.2 real resume authorization readiness read model and readonly UI 已完成：`evidence/2026-06-07-stage-h-h2-2-real-resume-authorization-readiness-read-model-and-readonly-ui-v1.md`
- H2.3 real resume request builder and CodexLocal guard bridge 已完成：`evidence/2026-06-07-stage-h-h2-3-real-resume-request-builder-and-codex-local-guard-bridge-v1.md`
- H2.4 real resume execution authorization and fixture freeze 已完成：`evidence/2026-06-07-stage-h-h2-4-real-resume-execution-authorization-and-fixture-freeze-v1.md`
- H2.6 Phase B readiness / fixture session binding / runtime log hardening 已完成：`tasks/2026-06-07-stage-h-h2-6-phase-b-readiness-fixture-session-and-runtime-log-hardening-v1.md`
- H2.7 Phase B authorization / fixture / target session confirmation 已完成为授权准备复核和阻断状态冻结：`tasks/2026-06-07-stage-h-h2-7-phase-b-authorization-fixture-and-target-session-confirmation-v1.md`
- H2.8 real execution permission dialog / audit summary / readiness decision surface 已完成：`tasks/2026-06-07-stage-h-h2-8-real-execution-permission-dialog-audit-summary-and-readiness-decision-surface-v1.md`
- H3-A new session authorization / fixture / boundary freeze 已完成：`tasks/2026-06-07-stage-h-h3-a-new-session-authorization-fixture-and-boundary-freeze-v1.md`
- H3.1 new session request / guard / permission envelope / no-op runner 已完成：`tasks/2026-06-07-stage-h-h3-1-new-session-request-guard-permission-envelope-and-noop-runner-v1.md`
- H3-B final approval / real new session fixture run 已执行一次真实 fixture run 并失败分类，产品路径已补 `--skip-git-repo-check`，下一次 retry 必须重新授权：`tasks/2026-06-07-stage-h-h3-b-real-new-session-final-approval-and-fixture-run-v1.md`

## 2. 中间版本最终目标

中间版本要完成两个核心闭环：

1. 自动化工作流闭环。
2. 完整记忆系统闭环。

大白话：

用户确认方案后，工作台能在授权范围内自动推进项目工作；项目主管管理 worker 和过程事实；全局主管只复核方案边界和最终结果；用户主要看方案、结果和必须拍板的事项；记忆层能把已确认事实沉淀成正式记忆，并在后续任务包里按权限召回。

中间版本完成不能只看：

- 有没有候选 sidecar。
- 有没有 SQLite 表。
- 有没有记忆页面。
- 有没有只读秘书摘要。
- 有没有单次 demo。

必须看：

- 工作流能自动推进。
- worker 汇报能被项目主管确认成过程事实。
- 过程事实能进入 observation / candidate / formal memory。
- 正式记忆有来源、版本、权限、冲突和审计。
- 后续 worker 的任务包能召回正式记忆，并解释入选和排除原因。
- 正式记忆生命周期、关系治理、维护任务、成熟模式和知识库边界可运行。

## 3. 当前基线

已完成：

- 最终工作台骨架 `final-skeleton-00` 到 `final-skeleton-16`。
- 会话中心底座硬化。
- 工作流派发 readback native parser 迁移。
- adapter 后端能力声明读模型。
- 黑板候选和记忆候选最小治理闭环。
- 秘书只读模型和独立入口。
- 记忆层设计、实施切片、M1 正式记忆受控存储任务包、M1.1 正式记忆上下文绑定 guard、M2 候选到正式记忆受控采纳、M3 ObservationStore / 工作流观察入口、M4 任务记忆包生成器 / 预览、M5 冲突与记忆 lint 最小阻断、M6 工作流任务包注入、M7 记忆管理 UI 最小入口、M8 知识库 / Obsidian-compatible 接口占位和边界、M9 正式记忆生命周期操作、M10 实体和关系治理、M11 维护任务和记忆 lint、M12 成熟模式 / 跨项目记忆 / M1-M12 gate 摘要、M12.1 freshness 修补、M13 最终权威验收。
- 工作流 C1 方案授权对象、授权范围 guard 和受控自动推进基础。
- 工作流 C2 项目咨询方案草案、用户确认入口和 C1 `PlanAuthorization` 受控联动。
- 工作流 C3 全局主管方案边界复核、授权 active 受控生效和 guard 验证摘要。
- 工作流 C4 项目主管拆任务、任务包 / 记忆包准备和授权范围内 prepared dispatch 落账。
- 工作流 C5 worker 结构化汇报、项目主管过程事实确认、`process_fact` observation 写入，以及失败 / readback / 权限最小可见化。
- 阶段 E / E1 adapter descriptor 执行边界和模型 / 凭据只读状态底座。
- 阶段 E / E2 会话操作边界契约和只读 UI。
- 阶段 E / E3 模型、凭据和 provider availability 只读边界。
- 阶段 E / E4 会话继续协议和权限预览、E5 Level A、E5 Level B mario test 健康探针、E6 runtime session attention / readback failure boundary、E7 阶段 E 总复核。
- 阶段 F / F1 项目工作流画布读模型收敛。
- 阶段 F / F2 节点详情 / evidence surface 产品化。
- 阶段 F / F3 受控工作流编辑提案和布局边界。
- 前端显示边界已完成问答确认并形成 `docs/workbench-frontend-display-boundary-v1.md`；该文档包含最终产品方向、中间版本显示范围、后端依赖和后置能力。

- 仍未完成 / deferred：

- 真实 Tauri 全面验收和自动重试；G1 runtime log 最小底座、G2 diagnostics / health / degraded state、G4 离线回放和 G5 最终冻结已完成。
- Obsidian 原生同步、vault 扫描和知识库深度能力。
- 多 agent 真实接入；E1 仅完成 planned descriptors 和只读状态边界。
- 真实 Tauri 全面验收和更完整的运维恢复体系；G1 runtime log 最小底座和 G2 diagnostics / health / degraded state 已完成。
- 按前端显示边界继续补智能体发消息、秘书悬浮标、管理入口、开发者模式和更完整真实 Tauri 验收。

## 4. 阶段总览

### 阶段 A：权威入口和底座对齐

状态：已完成；已补阶段 C1-C6 和 M7-M13 完成状态，并将下一步指向阶段 E / 阶段 G 后续范围。

目标：

- 让所有入口都能识别中间版本方案、记忆层实施切片、C1-C6 和 M7-M13 完成状态，以及阶段 E / 阶段 G 后续任务方向。
- 防止后续对话继续按旧骨架阶段或旧 harness 记忆治理推进。

必须完成：

- `docs/plans/middleware-version-stage-plan-v1.md`。
- `STAGE_PLAN.md` 指向中间版本阶段计划。
- `CURRENT.md` 指向中间版本方案、记忆层实施切片、M1 / M1.1 / M2 / M3 / M4 / M5 / M6 / M7 / M8 完成状态和 C1-C6 完成状态。
- `AUTHORITY.md` 收录中间版本方案和实施切片。
- `tasks/README.md` 标记 M1 / M1.1 / M2 / M3 / M4 / M5 / M6 / M7 / M8 / M9 / M10 / M11 / M12 / M12.1 / M13 和 C1-C6 已完成，并指向阶段 E / 阶段 G 为后续。

验收：

- 从 `CURRENT.md` 能找到中间版本方案、记忆层实施切片、M1 / M1.1 / M2 / M3 / M4 / M5 / M6 / M7 / M8 / M9 / M10 / M11 / M12 / M12.1 / M13 完成状态、C1-C6 完成状态以及阶段 E / 阶段 G 后续范围。
- 从 `AUTHORITY.md` 能找到当前阶段计划和设计依据。
- `STAGE_PLAN.md` 不再停留在旧 final-skeleton 进度。
- 从 `CURRENT.md`、`AUTHORITY.md` 和 `README.md` 能找到前端显示边界文档。

### 阶段 B：记忆层第一条真实闭环

状态：已完成；M1、M1.1、M2、M3、M4、M5 和 M6 均已完成；M7 已完成记忆管理 UI 最小入口。

对应记忆层 M1 到 M6。

目标：

```text
worker 汇报
-> 项目主管确认过程事实
-> ObservationStore
-> MemoryCandidate
-> Formal Memory
-> MemoryVersion + MemoryAuditEvent
-> TaskMemoryPacketBuilder
-> 下一个 worker 按权限召回正式记忆
```

任务顺序：

1. M1：正式记忆受控存储和审计骨架。
2. M1.1：正式记忆创建上下文绑定 guard。
3. M2：候选到正式记忆的受控采纳。
4. M3：ObservationStore 和工作流观察入口。
5. M4：任务记忆包生成器和预览。
6. M5：冲突和记忆 lint 最小阻断。
7. M6：工作流任务包注入和端到端闭环。

验收：

- 候选不能当正式记忆进入任务包。
- 正式记忆创建时必须有来源、版本和审计。
- 冲突、过期、权限不满足时不能进入任务包。
- 下一个 worker 能拿到最小必要正式记忆。
- M6 完成后只能说“第一条真实闭环完成”，不能说中间版本记忆层完成。

### 阶段 C：自动化工作流产品化闭环

状态：已完成；C1、C2、C3、C4、C5 和 C6 已完成。

目标：

- 把当前已有工作流机器、派发、readback、项目主管、全局主管、权限确认收敛成中间版本产品闭环。
- 用户确认方案后，项目主管可以在授权范围内自动派 worker 和推进过程。

必须完成：

- 方案授权对象和 UI。
- 项目咨询生成方案。
- 用户确认方案。
- 全局主管复核方案边界。
- 项目主管拆任务并派 worker。
- worker 结构化汇报。
- 项目主管确认过程事实。
- 失败、readback、权限、超时、取消和重试可见化。
- 全局主管复核最终结果。
- 用户查看结果和必须拍板事项。

依赖：

- Agent adapter 后端读模型已完成。
- M1 到 M4 后可以开始做方案授权和任务包预览。
- M6 后才能宣称记忆召回参与了自动化闭环。

任务顺序建议：

1. C1：方案授权对象、授权范围 guard 和受控自动推进基础。
2. C2：项目咨询方案生成和用户确认入口。
3. C3：全局主管方案边界复核和授权生效。
4. C4：项目主管拆任务和授权范围内 prepared auto dispatch。
5. C5：worker 结构化汇报、项目主管过程事实确认和失败 / readback / 权限可见化。
6. C6：全局主管最终结果复核、用户结果查看和阶段 C 验收。

禁止：

- 不绕过方案授权执行真实 Codex。
- 不把 worker 汇报直接写正式事实。
- 不把 readback 失败伪装成真实 0 条读回。
- 不让秘书当成果裁判。

### 阶段 D：中间版本完整记忆系统

状态：M7 已完成；M8 已完成；M9 已完成；M10 已完成；M11 已完成；M12 已完成；M12.1 已完成；M13 已完成，最终结论 `accepted_with_deferred_items`。

对应记忆层 M7 到 M13。

目标：

- 把记忆层从“第一条闭环可运行”补齐到“中间版本完整记忆系统”。

任务顺序：

1. M7：记忆管理 UI 最小入口。
2. M8：知识库和 Obsidian 接口占位。
3. M9：正式记忆生命周期操作。
4. M10：实体和关系治理。
5. M11：维护任务和记忆 lint。
6. M12：成熟模式、跨项目记忆和完整验收。
7. M13：中间版本记忆系统最终验收。

验收：

- 正式记忆可以新增、编辑、废弃、冻结、归档、合并、拆分、上升和下沉。
- 每次变化都有版本和审计。
- 关系和实体治理可用，LLM 推断关系默认只是候选。
- 维护任务能检查过期、冲突、缺来源、重复、私密、权限撤回和索引状态。
- 成熟模式和跨项目记忆必须用户确认；M12 已实现候选 / 报告 / 用户确认写入链路，M12.1 已修补 acceptance summary freshness，M13 已完成最终权威验收。
- Obsidian-compatible 知识库不能绕过记忆状态机。

### 阶段 E：会话、adapter、多 agent 和模型凭据底座

状态：E1 已完成；E2 已完成；E3 已完成；E4 已完成；E5 Level A 已完成；E5 Level B mario test 健康探针已完成；E6 已完成；E7 已完成，阶段 E 总结论为 `accepted_with_deferred_items`；E/F/G 细化计划已完成。

目标：

- 把当前 Codex 会话中心和 adapter 读模型扩展为可支持 Claude Code / OpenClaw / OpenCode / OpenCode-like agent 的底座。
- 建立模型、凭据、权限、成本和运行边界。
- E1 已完成 descriptor 和执行边界底座；planned adapters 只读展示为不可执行，不能宣称已接入。
- E2 已完成：`tasks/2026-06-05-stage-e-e2-session-operation-boundary-contract-and-readonly-ui-v1.md`。它只接受为 operation boundary read model 和智能体页只读 / 禁用态展示完成，不接受为真实发消息、停止、重启、resume、导出、删除或收藏。
- E3 已完成：`tasks/2026-06-06-stage-e-e3-model-credential-provider-availability-readonly-boundary-v1.md`。它只接受为 provider / model / credential availability、外发风险和成本风险只读边界完成，不接受为真实 credential store、provider token 验证、外部模型调用或 planned adapter 真实接入。
- E4 已完成：`tasks/2026-06-06-stage-e-e4-session-continuation-protocol-and-permission-preview-v1.md`。E4 只接受为会话继续协议和权限预览，不接受为真实 send / resume、prompt 已发送、attempt / dispatch / readback 写入或阶段 G 真实 Tauri 验收。
- E5 已完成 Level A：`tasks/2026-06-06-stage-e-e5-codex-local-controlled-send-resume-minimal-loop-v1.md`。接受为代码路径、guard、stub / dry-run、工作台自有 continuation 记录和离线验收；不接受为真实 prompt 已发送、真实 readback 或通用会话继续已验收。
- E6 已完成：`tasks/2026-06-06-stage-e-e6-runtime-session-attention-and-readback-failure-boundary-v1.md`。E6 只接受为 runtime session attention、readback failed / unavailable 边界、状态摘要 UI 和秘书解释完成，不授权真实执行、自动重试、完整 runtime log 或阶段 G 验收。
- E5 Level B mario test 健康探针已完成：`tasks/2026-06-06-stage-e-e5-level-b-mario-test-controlled-real-resume-health-probe-v1.md`。本轮只接受为指定 session 的最小真实 resume 健康探针；不接受为通用 send / resume 产品化、自动重试、runtime log、planned adapters 或阶段 G 验收完成。
- E/F/G 细化计划已完成：`docs/plans/2026-06-06-stage-e-f-g-refinement-plan-v1.md`。E7 已完成：`tasks/2026-06-06-stage-e-e7-session-adapter-model-boundary-acceptance-v1.md`，结论为 `accepted_with_deferred_items`；F1 已完成：`tasks/2026-06-06-stage-f-f1-project-workflow-canvas-read-model-consolidation-v1.md`；F2 已完成：`tasks/2026-06-06-stage-f-f2-workflow-node-detail-drawer-and-evidence-surface-v1.md`；F3 已完成：`tasks/2026-06-07-stage-f-f3-controlled-workflow-edit-proposal-and-layout-boundary-v1.md`；F4 已完成：`tasks/2026-06-07-stage-f-f4-project-canvas-experiment-canvas-boundary-hardening-v1.md`；F5 已完成：`tasks/2026-06-07-stage-f-f5-project-workflow-canvas-productization-acceptance-v1.md`，阶段 F 最终结论为 `accepted_with_deferred_items`。F1-F5 不接受为画布编辑器、布局持久化、项目画布和实验画布合一、真实执行、runtime log、diagnostics、真实 Tauri 验收、阶段 G 开始或中间版本最终验收完成。

必须完成：

- Claude Code / OpenClaw / OpenCode descriptor 设计任务。
- 不显示未实现 adapter 的可点击能力按钮。
- 模型 / 凭据只读状态和权限边界。
- agent 能力声明和真实执行能力分离。
- 会话操作边界：E2 已完成，覆盖发消息、停止、重启、resume、导出、删除、收藏等逐项设计和验收。
- E5-E7：`codex-local` 受控 send / resume 最小闭环；runtime session attention 和 readback failure 边界；阶段 E 验收。E3 模型 / 凭据 / provider availability 只读边界、E4 会话继续协议 / 权限预览、E5 Level A stub 闭环、E6 runtime attention 和 E7 阶段 E 总复核已完成；阶段 E 结论为 `accepted_with_deferred_items`。
- GEPA 只作为后置优化层候选保留研究结论；不进入当前 E1 / E2 主线，不进入当前执行 backlog。`GEPA-0` 只能在 E1 / E2 / 必要的模型凭据边界任务后重新审核并单独批准，真正运行 GEPA 必须等阶段 G 的运行日志、诊断、eval、成本、脱敏和回滚底座完成。
- Paseo 只作为多 agent 运行层外部参考保留研究结论；不进入当前 E1 / E2 主线，不进入当前执行 backlog。`PASEO-0` 到 `PASEO-4` 只能在阶段 E / F / G 后续专题中重新审核并单独拆任务包。

禁止：

- 不因为 descriptor 存在就宣称 agent 已接入。
- 不绕过凭据权限调用外部模型。
- 不读写 `/Users/yoyi/.codex`，除非用户对具体任务明确授权。
- 不因为 GEPA 研究存在就启动优化器、安装 GEPA、调用外部模型或改生产 prompt。
- 不因为 Paseo 研究存在就启动 daemon、接外部 agent、开放 MCP agent 自治编排、创建 worktree / schedule / relay 或把 timeline 当正式事实 / 正式记忆。

### 阶段 F：项目工作流画布产品化深化

状态：已完成整体细化；E5 Level B 健康探针已回收，F1 项目工作流画布读模型收敛已完成，F2 节点详情 / evidence surface 已完成，F3 受控工作流编辑提案和布局边界已完成，F4 项目画布 / 实验画布边界硬化已完成，F5 项目工作流画布产品化验收已完成；阶段 F 最终结论为 `accepted_with_deferred_items`。

阶段 F 剩余：无。阶段 G / G1 Runtime Log Boundary And Minimal Store、G2 Diagnostics Health And Degraded State、G3-A Real Tauri Acceptance Plan And Fixture Freeze、G3-C Screenshot Evidence Recovery And Gap Matrix、G4 Middle Version End-to-End Acceptance Replay 和 G5 Final Authoritative Acceptance And Deferred Freeze 已完成；G3-B Real Tauri Manual Screenshot Acceptance 已回交但未完成，只接受为真实 Tauri 10 / 13 部分截图证据。中间版本最终结论冻结为 `accepted_with_deferred_items`。

目标：

- 把项目工作流画布从只读骨架推进到可服务真实项目主管和自动化工作流的工作界面。
- F1-F5 细化见 `docs/plans/2026-06-06-stage-e-f-g-refinement-plan-v1.md`。

必须完成：

- 项目工作流画布读模型、节点详情、编辑 / 布局边界、项目画布 / 实验画布边界和阶段 F 验收已完成。
- 节点详情承载任务包、权限、readback、记忆召回、审计和失败信息。
- 画布编辑能力必须受控制核心和 workflow state 约束。
- 独立实验画布继续降权，不和项目工作流主画布混淆。
- 阶段 F 总验收确认项目工作流画布能进入阶段 G 真实验收。

依赖：

- 阶段 B 的记忆召回。
- 阶段 C 的方案授权和自动化工作流。

禁止：

- 不让 React Flow 成为事实源。
- 不启动 MCP canvas run 作为默认工作流。
- 不把画布变成通用节点自动化平台。

### 阶段 G：真实验收、运维日志和中间版本收口

状态：已完成整体细化；G1 Runtime Log Boundary And Minimal Store、G2 Diagnostics Health And Degraded State、G3-A Real Tauri Acceptance Plan And Fixture Freeze、G3-C Screenshot Evidence Recovery And Gap Matrix、G4 Middle Version End-to-End Acceptance Replay 和 G5 Final Authoritative Acceptance And Deferred Freeze 已完成；G3-B Real Tauri Manual Screenshot Acceptance 已回交但未完成。中间版本最终结论冻结为 `accepted_with_deferred_items`。

目标：

- 用真实 Tauri、手动验收、离线测试、Rust 测试和运行日志把中间版本收口。
- G1-G5 细化见 `docs/plans/2026-06-06-stage-e-f-g-refinement-plan-v1.md`。

必须完成：

- 真实 Tauri 窗口验收。
- 权限弹层、项目页、会话中心、工作流画布、记忆管理、任务包预览的手动测试清单。
- 运行日志和运维诊断体系已完成 G1 / G2 最小底座；G4 离线端到端回放和 G5 最终冻结已完成；真实 Tauri 全量验收仍未完成，只接受 G3-B 10 / 13 部分截图证据和 G3-C 缺口矩阵。
- 审计和运行日志边界。
- 真实 Tauri 截图证据和中间版本端到端验收回放。
- 中间版本最终回收报告。

禁止：

- 不用单次 demo 代替中间版本验收。
- 不把运行日志当审计。
- 不把审计当运行日志。

## 5. 当前可执行下一步

当前下一步是：

- E/F/G 细化计划已完成：`docs/plans/2026-06-06-stage-e-f-g-refinement-plan-v1.md`。
- E3 已完成：`tasks/2026-06-06-stage-e-e3-model-credential-provider-availability-readonly-boundary-v1.md`。
- E4 已完成：`tasks/2026-06-06-stage-e-e4-session-continuation-protocol-and-permission-preview-v1.md`；E5 Level A 已完成：`tasks/2026-06-06-stage-e-e5-codex-local-controlled-send-resume-minimal-loop-v1.md`；E6 已完成：`tasks/2026-06-06-stage-e-e6-runtime-session-attention-and-readback-failure-boundary-v1.md`；E7 已完成：`tasks/2026-06-06-stage-e-e7-session-adapter-model-boundary-acceptance-v1.md`。
- 阶段 G 后续小切片已收口：F1 已完成，F2 节点详情 / evidence surface 已完成，F3 受控工作流编辑提案和布局边界已完成，F4 项目画布 / 实验画布边界硬化已完成，F5 阶段 F 验收已完成；G1、G2、G3-A、G3-C、G4 和 G5 已完成；G3-B 已回交但未完成，只接受为 10 / 13 真实 Tauri 部分截图证据；中间版本最终结论冻结为 `accepted_with_deferred_items`。
- 保留后置项：真实 Tauri 全面验收、自动重试、通用真实 send / resume 产品化、planned adapters 真实接入、provider credential / model verification 和最终蓝图能力。
- H-I 后续计划已新增：`docs/plans/2026-06-07-stage-h-i-real-codex-automation-and-multi-agent-collaboration-plan-v1.md`。阶段 H 已由 H7 冻结为 `accepted_with_deferred_items`；阶段 I 已完成中立多 agent / 多模型协作协议抽象，I6 记录见 `evidence/2026-06-08-stage-i-i6-stage-i-final-acceptance-and-adapter-roadmap-freeze-v1.md` 与 `handoffs/2026-06-08-stage-i-i6-stage-i-final-acceptance-and-adapter-roadmap-freeze-v1-result.md`。Codex 多线程协作只作为架构参考，不能照搬为工作台事实模型。H-I 阶段整体收口为 `accepted_with_deferred_items`；默认不授权新的真实 Codex 执行、`.codex` 读写、planned adapters 真实接入或 provider credential / model verification。
- GEPA 暂不作为当前下一步，只作为后置优化层候选研究资料保留。
- Paseo 暂不作为当前下一步，只作为多 agent 运行层外部研究资料保留。
- M7 已完成：`tasks/2026-06-05-memory-layer-m7-memory-management-ui-minimal-entry-v1.md`。
- M7 记录见 `evidence/2026-06-05-memory-layer-m7-memory-management-ui-minimal-entry-v1.md` 与 `handoffs/2026-06-05-memory-layer-m7-memory-management-ui-minimal-entry-v1-result.md`。
- M8 已完成：`tasks/2026-06-05-memory-layer-m8-knowledge-base-and-obsidian-compatible-interface-placeholder-v1.md`。
- M8 记录见 `evidence/2026-06-05-memory-layer-m8-knowledge-base-and-obsidian-compatible-interface-placeholder-v1.md` 与 `handoffs/2026-06-05-memory-layer-m8-knowledge-base-and-obsidian-compatible-interface-placeholder-v1-result.md`。
- M9 已完成：`tasks/2026-06-05-memory-layer-m9-formal-memory-lifecycle-operations-v1.md`。
- M9 记录见 `evidence/2026-06-05-memory-layer-m9-formal-memory-lifecycle-operations-v1.md` 与 `handoffs/2026-06-05-memory-layer-m9-formal-memory-lifecycle-operations-v1-result.md`。
- C1 已完成：`tasks/2026-06-04-workflow-c1-plan-authorization-and-controlled-auto-dispatch-foundation-v1.md`。
- C1 记录见 `evidence/2026-06-04-workflow-c1-plan-authorization-and-controlled-auto-dispatch-foundation-v1.md` 与 `handoffs/2026-06-04-workflow-c1-plan-authorization-and-controlled-auto-dispatch-foundation-v1-result.md`。
- C2 已完成：`tasks/2026-06-04-workflow-c2-project-consultation-proposal-and-user-confirmation-entry-v1.md`。
- C2 记录见 `evidence/2026-06-04-workflow-c2-project-consultation-proposal-and-user-confirmation-entry-v1.md` 与 `handoffs/2026-06-04-workflow-c2-project-consultation-proposal-and-user-confirmation-entry-v1-result.md`。
- C3 已完成：`tasks/2026-06-04-workflow-c3-global-boundary-review-and-authorization-activation-v1.md`。
- C3 记录见 `evidence/2026-06-04-workflow-c3-global-boundary-review-and-authorization-activation-v1.md` 与 `handoffs/2026-06-04-workflow-c3-global-boundary-review-and-authorization-activation-v1-result.md`。
- C4 已完成：`tasks/2026-06-04-workflow-c4-project-director-task-decomposition-and-authorized-prepared-auto-dispatch-v1.md`。
- C4 记录见 `evidence/2026-06-04-workflow-c4-project-director-task-decomposition-and-authorized-prepared-auto-dispatch-v1.md` 与 `handoffs/2026-06-04-workflow-c4-project-director-task-decomposition-and-authorized-prepared-auto-dispatch-v1-result.md`。
- C5 已完成：`tasks/2026-06-04-workflow-c5-worker-structured-report-process-fact-confirmation-and-failure-visibility-v1.md`。
- C5 记录见 `evidence/2026-06-04-workflow-c5-worker-structured-report-process-fact-confirmation-and-failure-visibility-v1.md` 与 `handoffs/2026-06-04-workflow-c5-worker-structured-report-process-fact-confirmation-and-failure-visibility-v1-result.md`。
- C6 已完成：`tasks/2026-06-05-workflow-c6-global-final-result-review-user-result-view-and-stage-c-acceptance-v1.md`。
- C6 记录见 `evidence/2026-06-05-workflow-c6-global-final-result-review-user-result-view-and-stage-c-acceptance-v1.md` 与 `handoffs/2026-06-05-workflow-c6-global-final-result-review-user-result-view-and-stage-c-acceptance-v1-result.md`。
- 阶段 D 已完成 M7-M13；M13 最终结论为 `accepted_with_deferred_items`。
- E1 已完成：`tasks/2026-06-05-stage-e-e1-agent-adapter-descriptor-execution-boundary-and-model-credential-readonly-foundation-v1.md`。
- E1 记录见 `evidence/2026-06-05-stage-e-e1-agent-adapter-descriptor-execution-boundary-and-model-credential-readonly-foundation-v1.md` 与 `handoffs/2026-06-05-stage-e-e1-agent-adapter-descriptor-execution-boundary-and-model-credential-readonly-foundation-v1-result.md`。

M1 已完成，记录为：

- `evidence/2026-06-03-memory-layer-m1-formal-memory-store-and-audit-v1.md`
- `handoffs/2026-06-03-memory-layer-m1-formal-memory-store-and-audit-v1-result.md`

M1.1 已完成，记录为：

- `evidence/2026-06-03-memory-layer-m1-1-formal-memory-context-binding-guard-v1.md`
- `handoffs/2026-06-03-memory-layer-m1-1-formal-memory-context-binding-guard-v1-result.md`

M2 已完成，记录为：

- `evidence/2026-06-03-memory-layer-m2-candidate-to-formal-adoption-v1.md`
- `handoffs/2026-06-03-memory-layer-m2-candidate-to-formal-adoption-v1-result.md`

M3 已完成，记录为：

- `evidence/2026-06-04-memory-layer-m3-observation-store-and-workflow-entry-v1.md`
- `handoffs/2026-06-04-memory-layer-m3-observation-store-and-workflow-entry-v1-result.md`

M4 已完成，记录为：

- `evidence/2026-06-04-memory-layer-m4-task-memory-packet-builder-and-preview-v1.md`
- `handoffs/2026-06-04-memory-layer-m4-task-memory-packet-builder-and-preview-v1-result.md`

M5 已完成，记录为：

- `evidence/2026-06-04-memory-layer-m5-conflict-and-memory-lint-minimal-blocking-v1.md`
- `handoffs/2026-06-04-memory-layer-m5-conflict-and-memory-lint-minimal-blocking-v1-result.md`

M6 已完成，记录为：

- `evidence/2026-06-04-memory-layer-m6-workflow-task-package-injection-and-end-to-end-loop-v1.md`
- `handoffs/2026-06-04-memory-layer-m6-workflow-task-package-injection-and-end-to-end-loop-v1-result.md`

M7 已完成，记录为：

- `evidence/2026-06-05-memory-layer-m7-memory-management-ui-minimal-entry-v1.md`
- `handoffs/2026-06-05-memory-layer-m7-memory-management-ui-minimal-entry-v1-result.md`

M8 已完成，记录为：

- `evidence/2026-06-05-memory-layer-m8-knowledge-base-and-obsidian-compatible-interface-placeholder-v1.md`
- `handoffs/2026-06-05-memory-layer-m8-knowledge-base-and-obsidian-compatible-interface-placeholder-v1-result.md`

M9 / C1 / C2 / C3 / C4 / C5 / C6 / M13 / E1 / E2 / E3 完成后下一步建议：

1. E/F/G 细化计划已完成；E5 Level B mario test 健康探针已回收，F1 项目工作流画布读模型收敛已完成，F2 节点详情 / evidence surface 已完成，F3 受控工作流编辑提案和布局边界已完成，F4 项目画布 / 实验画布边界硬化已完成，F5 阶段 F 验收已完成，G1 Runtime Log Boundary And Minimal Store、G2 Diagnostics Health And Degraded State、G3-A Real Tauri Acceptance Plan And Fixture Freeze、G3-C Screenshot Evidence Recovery And Gap Matrix、G4 Middle Version End-to-End Acceptance Replay 和 G5 Final Authoritative Acceptance And Deferred Freeze 已完成；G3-B Real Tauri Manual Screenshot Acceptance 已回交但未完成，真实 Tauri 已启动、目标窗口区域截图探针成功并采集 10 / 13 张编号截图。E4 已完成，只接受为会话继续协议和权限预览；E5 Level A 已完成，只接受为代码路径、guard、stub、工作台自有 continuation 记录和离线验收；E5 Level B 只接受为指定 session 的最小真实 resume 健康探针；E6 已完成，只接受为 runtime attention、readback failed / unavailable 边界、摘要 UI 和秘书解释；E7 已完成，阶段 E 总结论为 `accepted_with_deferred_items`。F1/F2/F3/F4/F5 任务包不接受为画布编辑器、布局持久化、项目画布和实验画布合一、真实执行、diagnostics、真实 Tauri 全量验收或最终蓝图完整完成；G1、G2、G3-A、G3-B 和 G3-C 当前证据不替代 G4/G5 的独立回放与冻结结论，也不接受为 G3 全量真实 Tauri 验收。
2. 中间版本最终结论冻结为 `accepted_with_deferred_items`；后置仍需另拆真实 Tauri 全面验收、自动重试、通用真实 send / resume 产品化、planned adapters 真实接入、provider credential / model verification 和最终蓝图能力。
3. GEPA 研究建议只作为后置优化层候选保留，不进入当前 E2，不授权安装 / 运行 GEPA 或修改生产 prompt。
4. Paseo 研究建议只作为多 agent 运行层外部参考保留，不进入当前 E2，不授权启动 daemon、接外部 agent、开放 MCP agent 自治编排、创建 worktree / schedule / relay 或把 timeline 当正式事实 / 正式记忆。
5. 继续避免把 observation sidecar、候选 sidecar、M1 store、M2 采纳链路、M3 观察入口、M4 预览、M5 finding、M6 第一条闭环、M7 记忆中心 UI、M8 知识库占位、M9 生命周期操作、M10 实体 / 关系治理、C4 prepared dispatch、C5 process fact observation、C6 gate 摘要或 SQLite 建表误认为最终蓝图完整完成。

## 6. 粒度判断

可以支撑开发的粒度：

- 单个任务包：M1 这种粒度可以直接交给其他对话执行。
- 记忆层：M1 到 M13 可以支撑连续推进。
- 自动化工作流：C1、C2、C3、C4、C5 和 C6 已完成。
- 多 agent / 模型 / 凭据：E1 已完成 descriptor 执行边界和模型 / 凭据只读底座；E2 已完成会话操作边界和只读 UI；E3 已完成 provider / model / credential availability 只读边界；E4、E5 Level A、E5 Level B 健康探针、E6 和 E7 已完成，真实发送 / resume 产品化仍需后续任务包；runtime log G1 和 diagnostics G2 已完成。
- 画布深化：F1 画布读模型收敛、F2 节点详情 / evidence surface、F3 受控工作流编辑提案 / 布局边界、F4 项目画布 / 实验画布边界硬化和 F5 阶段 F 验收已完成；阶段 F 结论为 `accepted_with_deferred_items`。
- E/F/G：`docs/plans/2026-06-06-stage-e-f-g-refinement-plan-v1.md` 已把后续拆成 E3-E7、F1-F5、G1-G5；E3、E4、E5 Level A、E5 Level B 健康探针、E6、E7、F1、F2、F3、F4、F5、G1、G2、G3-A、G3-C、G4 和 G5 已完成；G3-B 已回交但未完成，只接受为 10 / 13 真实 Tauri 部分截图证据。

不足：

- F1、F2、F3、F4、F5、G1、G2、G3-A、G3-C、G4 和 G5 任务包已完成；G3-B 已回交但未完成，只接受为 10 / 13 真实 Tauri 部分截图证据。E5 Level B 健康探针已完成但不能替代通用 send / resume 产品化；G1 runtime log、G2 diagnostics 和 G3-C 缺口矩阵不能替代 G3 全量真实 Tauri 验收。中间版本最终结论冻结为 `accepted_with_deferred_items`。
- UI 真机验收 G3-B 已回交但未完成，当前已有 10 / 13 部分截图并由 G3-C 冻结缺口矩阵；运行日志 G1 和诊断 G2 已完成。

## 7. 总边界

- 不默认执行真实 Codex。
- 不默认执行 `codex exec` / `codex exec resume`.
- 不读写 `/Users/yoyi/.codex`。
- 不改 `workflow-state.v0.json` 结构，除非单独任务包明确授权。
- 不迁移数据库，除非有迁移 / 双写 / 回滚方案。
- 不把 worker 汇报当正式事实。
- 不把候选、知识库命中、LLM 摘要或图谱推断当正式记忆。
- 不让秘书确认事实、派活或写正式记忆。
- 不让画布、UI 或 Markdown 绕过控制核心。
