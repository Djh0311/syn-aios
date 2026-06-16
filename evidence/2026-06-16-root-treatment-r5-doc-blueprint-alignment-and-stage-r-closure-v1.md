# Root Treatment / R5 Doc Blueprint Alignment And Stage R Closure

日期：2026-06-16

状态：review_clear_gates_passed_pending_user_ratification

## 拍板摘要

建议批准：R5 将 product-line 的权威文档口径对齐到外部蓝图正本和已落地实现，并在独立复核 CLEAR 后把 Stage R 收口提交用户拍板。

代价：无运行时代价。本包纯文档，不改代码、不跑 runner、不触碰真实数据、不触碰外部蓝图源；不把 `/Users/yoyi/.codex` 下的运行状态、会话、凭据或 transcript 作为业务输入。

不批的后果：蓝图、记忆层、吸收建议和 Stage L deferred 口径继续漂移，Stage R 无法形成可交接的收口结论。

一句话判据：R5 收没收口，看四块是否都“对齐到蓝图 + 实现、查重无残漂、Stage L 纯文档项已并、复核 CLEAR”；是，则 Stage R 可收口；产品切换类与 Stage L 产品代码类仍另窗另批。

## 1. 输入与边界

### 1.1 只读参考

- 外部蓝图正本：`/Users/yoyi/Documents/Codex/2026-05-26/gan-xing-codexbridge-https-github-com/docs/architecture/local-ai-workbench-blueprint-v1.md`
- 外部 UI 蓝图正本：`/Users/yoyi/Documents/Codex/2026-05-26/gan-xing-codexbridge-https-github-com/docs/architecture/local-ai-workbench-ui-blueprint-v1.md`
- 吸收建议文档 A：`/Users/yoyi/Documents/Codex/2026-05-26/gan-xing-codexbridge-https-github-com/docs/architecture/blueprint-absorption-notes.md`
- 吸收建议文档 B：`/Users/yoyi/Documents/Codex/2026-05-26/gan-xing-codexbridge-https-github-com/docs/architecture/xuanji-blueprint-absorption-notes.md`

### 1.2 product-line 正本 / 证据

- `docs/memory-layer-design-v1.md`
- `docs/plans/memory-layer-implementation-slice-v1.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `docs/plans/2026-06-10-stage-l-post-k-deferred-closure-and-daily-use-hardening-plan-v1.md`
- `docs/plans/2026-06-13-stage-r-remaining-execution-plan-v1.md`
- `evidence/r3-level-b/2026-06-16-b5-final-matrix-and-r3-closure-v1.md`
- `AUTHORITY.md`
- `CURRENT.md`

### 1.3 不做

- 不改外部蓝图源。
- 不改 backlog；旧 R5-4 的 backlog 分类在本包只保留为历史计划项，本轮按用户 2026-06-16 四块范围执行。
- 不启动 Stage L 产品代码、K3-B1 retry、K3-B2、真实操作控制或真实 Tauri 深层验收。
- 不触碰 R3 deferred 产品切换：真实停写 JSON / sidecar、产品全局 read path 切 DB、完整存储迁移都另窗另批。
- 不改代码、runner、真实数据目录；不读写 `/Users/yoyi/.codex` 下的运行状态、会话、凭据或 transcript。

## 2. 蓝图经验沉淀口径对齐

### 2.1 蓝图原口径

外部蓝图 §17 定义记忆类型和治理原则：用户偏好、全局产品蓝图、项目记忆、会话摘要、成熟模式记录；普通聊天不自动进入长期记忆；工作流总结、阶段汇报和用户确认采纳的建议方案可以进入候选链路；子智能体汇报先进入工作流账本；审查或项目主管总结后再进入项目记忆；旧记忆保留版本；结构调整需用户确认。

外部蓝图 §22 定义策略系统和成熟模式沉淀：规则不应全写死，工作台需要在真实开发中逐渐沉淀成熟模式；成熟模式可由用户手动保存或系统建议保存，适用范围由人决定，可全局也可项目私有，策略改动进入审计。

外部蓝图 §26.4 定义 Successful Run Pattern：一次任务完成得好后生成“成功运行模式候选”，记录任务类型、适用项目、上下文包、角色、模型、工具、技能、步骤、权限边界、禁止事项、验收、审查、失败处理、成本耗时、证据和审计链接；用户确认后才可沉淀为稳定工作流、harness 规则更新、任务包模板、项目记忆或全局成熟模式。

### 2.2 R5 统一解释

product-line 采用以下统一口径：

```text
工作流 / 阶段 / 成功运行证据
-> observation / candidate / cross-project report
-> MaturePatternCandidate
-> 用户确认
-> mature pattern formal memory / 项目记忆 / 全局成熟模式
-> 后续任务包按权限召回，并保留来源、版本、审计和反例
```

对应已落地路径：

- M3：`ObservationStore` 和工作流观察入口，worker 汇报先进入 observation，不直接成为正式记忆。
- M4 / M6：任务记忆包预览和注入，正式记忆才可按权限进入任务包，candidate / observation 只能作为待审材料。
- M9：正式记忆生命周期，编辑、废弃、冻结、解冻、归档、合并、拆分、上升 / 下沉 scope 都保留版本和审计。
- M11：维护任务和 memory lint，成熟模式检查只生成候选，不自动修改正式记忆。
- M12：成熟模式候选、跨项目主题报告、用户确认后 mature pattern 正式记忆受控写入、任务包召回边界和 M1-M12 gate 摘要。
- M12.1：用户确认 mature pattern 正式化后，当次 acceptance summary 使用 fresh formal store。
- M13：记忆系统最终权威验收，结论为 `accepted_with_deferred_items`。

R5 不把 Successful Run Pattern 解释为自动技能化、自动 harness 规则改写、自动全局策略升级或绕过用户确认的经验写入。

## 3. 两份吸收建议文档身份确认

本轮确认的“两份吸收建议文档”是外部蓝图仓库 `docs/architecture/` 下两份标题和用途均明确为吸收建议的文件：

| 文档 | 为什么是本轮对象 | 本轮处理 |
| --- | --- | --- |
| `blueprint-absorption-notes.md` | 标题为“蓝图吸收建议文档”，用途写明“综合两份研究记录，给出对蓝图 v1 的具体吸收建议” | 对照 M1-M13 / C1-C6 查重，区分 accepted / partial / deferred / not_absorbed |
| `xuanji-blueprint-absorption-notes.md` | 标题为“Xuanji 蓝图吸收建议文档”，用途写明“综合 Xuanji 研究报告，给出对蓝图 v1 的具体吸收建议” | 对照 M1-M13 / C1-C6 查重，区分 accepted / partial / deferred / not_absorbed |

product-line 内 `docs/research/2026-06-05-odysseus-*`、`gepa-*`、`paseo-*` 是研究参考或后置优化参考，不是本轮“两份吸收建议文档”的正本对象。

## 4. 吸收建议 × M1-M13 / C1-C6 查重矩阵

| 来源 | 吸收建议 | 对应已实现 / 正本 | R5 分类 | 口径 |
| --- | --- | --- | --- | --- |
| blueprint §1.1 | 任务包字段补充：filesToCreate / filesToModify / requiredSources / estimatedHours / actualHours 等 | C1-C6 已形成方案授权、任务包、prepared dispatch、worker 汇报、最终复核链；R0/R 系列任务包已实践 allowed roots、forbid、验证和 evidence | partial | 治理任务包实践已吸收“范围、来源、验证”思想；未形成统一产品 schema 字段，不声称全量实现 |
| blueprint §1.2 | 成熟模式操作类型 insert/update/merge/deprecate/promote/demote/freeze/fork | M9 正式记忆生命周期覆盖编辑、废弃、冻结、解冻、归档、合并、拆分、上升 / 下沉；M12 覆盖成熟模式候选和用户确认写入 | partial | 生命周期机制已实现；成熟模式专属操作 enum 和 fork 语义未单独产品化 |
| blueprint §1.3 | 成熟模式元数据字段 | M12 `MaturePatternCandidate`、evidence refs、counterexamples、scope、review status；M13 总验收 | partial | 已有核心候选和证据字段；hitCount / successRate / usedModels 等完整字段仍是后续 schema 扩展 |
| blueprint §1.4 | 审查 agent 四类一致性检查 | C3 全局边界复核、C6 最终复核、R 阶段独立复核、checkpoint-audit 工具 | accepted_with_tooling_gap | 人工 / 复核流程已吸收；自动化一致性 checker 只部分存在 |
| blueprint §2.1 | 成熟模式沉淀闭环 | M3 -> M12 -> M12.1 -> M13 | accepted_with_template_gap | 已落地为 observation / candidate / MaturePatternCandidate / 用户确认 / formal memory / 召回边界；写入 Skill / Harness / Workflow Template / task package template 仍未产品化，不声称已实现 |
| blueprint §2.2 | Scenario 实体化 | 现有 stage / task package / Product Command 场景口径分散在计划与任务包 | deferred | 不是 M1-M13 / C1-C6 已完成项；后续如做需独立 schema / 任务包 |
| blueprint §2.3 | Harness 防漂移验证 | R-U Gate 草稿、harness catalog、checkpoint-audit、shape gate | partial | 治理期已有检查工具和草稿；未实现 harness 启用前/定期防漂移产品门 |
| blueprint §2.4 | CurrentProjectSpecs / PendingChangeProposals | C1 PlanAuthorization、C2 proposal、C3 boundary review、C4 task decomposition、formal memory/decision docs | partial | 概念已由授权/建议/decision/记忆链路覆盖；独立规格账本对象未实现 |
| blueprint §3 | SkillOS RL、gstack browser daemon、OpenAI connector、Praxis runtime | 无 | not_absorbed | 明确不进入 Stage R；只保留概念参考 |
| xuanji §2.1 | 记忆对象字段补全 | M1 FormalMemoryStore、MemoryVersion、Audit、source refs；M9 生命周期；M10 实体关系 | accepted_with_gaps | 核心正式记忆字段已落地；confidence/expiresAt 等字段不作为已实现声称 |
| xuanji §2.2 | 记忆写入流程 | M1-M6 第一条真实记忆闭环 | accepted | 候选生成、来源检查、权限/确认、写入 formal memory、版本审计、任务包召回已完成 |
| xuanji §2.3 | workflow template executionStrategy | C4 task decomposition / prepared dispatch，Stage I/H 参考多 agent 协作 | deferred | 未实现 executionStrategy enum；R3 收口前也不解锁多 agent 并行真实执行 |
| xuanji §2.4 | 权限系统四层拆分 | C1/C3 authorization guard、H/J/R permission envelope、审计和复核制度 | partial | 治理链路已有策略/guard/approval/audit；完整产品化权限 runtime 仍在后续阶段 |
| xuanji §3.1 | 记忆召回机制 | M4 TaskMemoryPacketBuilder、M6 任务包注入 | accepted_with_gaps | 正式记忆按权限召回已落地；复杂排序、压缩、时效提示仍可后续扩展 |
| xuanji §3.2 | 记忆层与成熟模式分层 | 本 R5 §2；M12 / M13 | accepted_with_template_gap | 统一为 observation/candidate -> MaturePatternCandidate -> 用户确认 -> mature pattern formal memory；harness rule / workflow template / task package template 固化仍是后续能力 |
| xuanji §3.3 | 工作区隔离策略 | R-Preflight git baseline、Stage R 每批 commit；B5 deferred 清单仍未解锁多 agent 并行 | deferred | 不在 R5 实现；未来多 agent 并行需另窗设计和用户批准 |
| xuanji §3.4 | Provider Registry | E1 adapter descriptor、E3 provider availability 只读边界 | partial | 只读边界已吸收；真实 provider/model/credential registry 未完成 |
| xuanji §5.1 | Workbench Runtime 层描述 | `docs/workbench-system-architecture-v1.md`、C/E/F/G/H/J/R 阶段计划 | partial | 架构叙述已有分散落点；不在 R5 新增运行时 |
| xuanji §5.2 | Harness 定义为项目工作协议包 | `docs/harness-catalog.md`、任务包 / review / shape gate 纪律 | partial | 治理制度已吸收一部分；产品级 harness package schema 后续再做 |
| xuanji §6 | 不吸收：AI 管家中心化、自动 commit/merge、许可证不清源码、自动实体抽取、项目事实自动衰减 | 当前边界文档和任务包禁止事项 | not_absorbed | R5 明确不复制源码、不自动 commit/merge、不让 LLM 自动事实化 |

## 5. 产品蓝图正本路径锚定

`AUTHORITY.md` 的“产品蓝图”已写死两份唯一正本路径：

- `/Users/yoyi/Documents/Codex/2026-05-26/gan-xing-codexbridge-https-github-com/docs/architecture/local-ai-workbench-blueprint-v1.md`
- `/Users/yoyi/Documents/Codex/2026-05-26/gan-xing-codexbridge-https-github-com/docs/architecture/local-ai-workbench-ui-blueprint-v1.md`

R5 判定：§3 已满足。选择 `AUTHORITY.md` 锚定口径，不迁移外部源，不复制外部蓝图到 product-line，也不修改外部蓝图源。

## 6. Stage L 纯文档 / 口径项并入

Stage L plan 仍是治理收口后恢复 Stage L / Stage K 的后续计划，不是当前执行入口。R5 只并入纯文档 / 口径项：

| Stage L 项 | R5 并入口径 | R5 结论 |
| --- | --- | --- |
| L0 范围、权限、验收矩阵冻结 | 已作为 Stage L plan 和 R5 边界引用 | 已并入，不重复执行 |
| L1 K3-B1 blocked recovery product path | 只保留“治理后恢复，retry/替代路径需另包另批”口径 | deferred_during_root_treatment |
| L2 K3-B2 isolated workspace-write execution closure | 依赖 L1 成功或等价替代，通过前不得启动 | deferred_during_root_treatment |
| L3 Operation control hardening | 产品代码 / 状态机 / 真实操作控制，不属于 R5 | deferred_during_root_treatment |
| L4 Deep Tauri subview acceptance | 真实 Tauri 深层截图验收，不属于 R5 | deferred_during_root_treatment |
| L5 Memory capture to candidate daily loop | 记忆层产品体验增强，不属于 R5；R5 只保留候选/正式化边界 | deferred_during_root_treatment |
| L6 Stage L final acceptance freeze | 只能在 L1-L5 或其 deferred/blocked 判定后执行 | deferred_during_root_treatment |

R5 不启动 Stage L 产品代码，不取消 Stage L，不取消 K3-B1 / K3-B2。Stage R 收口后，Stage L 是否恢复以及如何排期仍由用户另行拍板。

## 7. Stage R 收口判定输入

| Stage R 子项 | 证据 | R5 判定 |
| --- | --- | --- |
| R4-H | `docs/plans/2026-06-13-stage-r-remaining-execution-plan-v1.md` §2 | ✅ 已完成：H1 / H2 / H3 均收口 |
| R-U | `docs/plans/2026-06-13-stage-r-remaining-execution-plan-v1.md` §3 | ✅ 已完成：U1-U5 完成，U-Gate 草稿完成；查重门未接入 runtime/CI，按草稿边界处理 |
| R3 Level B | `evidence/r3-level-b/2026-06-16-b5-final-matrix-and-r3-closure-v1.md` | ✅ 受控迁移验证阶段 B0-B4 已由用户 ratified 收口为 `accepted_with_deferred_product_cutover` |
| R5 | 本文 + `evidence/2026-06-16-root-treatment-r5-doc-blueprint-alignment-and-stage-r-closure-review-sagan-v1.md` | 独立复核线 Sagan `STATUS: CLEAR`；shape gate / `git diff --check` 已通过；待用户拍板 |

R5 建议的 Stage R 收口口径：

- 独立复核线 Sagan 已确认本文无 P0/P1/P2 overclaim，`STATUS: CLEAR`。
- shape gate / `git diff --check` 已通过，Stage R 可提交用户拍板收口。
- Stage R 收口不等于 Stage L 完成，不等于 R3 产品切换完成，不等于 backlog 解冻。

## 8. Deferred / 另窗清单

- Stage L L1-L6 产品代码和真实执行项。
- K3-B1 retry / 替代恢复路径。
- K3-B2 isolated workspace-write execution closure。
- 真实停写 JSON / sidecar。
- 产品全局 read path 切 DB。
- 完整存储迁移，包括全部 sidecar。
- 解锁多 agent 并行真实执行。
- 真实 Codex 执行。
- Scenario 实体化、executionStrategy enum、完整 provider registry、完整 harness package schema、harness 防漂移产品门。
- 记忆召回时效提示等 backlog 功能项：本包只记录边界，不实现、不回填 backlog。

## 9. 不可声称

- 不得声称 Stage L 已完成或取消。
- 不得声称 K3-B1 retry / K3-B2 已启动或已完成。
- 不得声称产品全局读写路径已切 DB。
- 不得声称 stop-write 已执行。
- 不得声称完整存储迁移完成。
- 不得声称多 agent 并行真实执行已解锁。
- 不得声称真实 Codex 已执行。
- 不得声称本包读取或写入 `/Users/yoyi/.codex` 下的运行状态、会话、凭据或 transcript。

## 10. 验证

已运行：

- `node scripts/harness/workbench-shape-gate.js --mode check`

```text
Workbench shape gate: /Users/yoyi/workspace/product-line
Mode: check
Status: pass
Errors: 0
Warnings: 0
Git HEAD: 57da3d0b10ea1a98113750d760dd9c83db4800f1
Tauri commands: 97 total; 0 in lib.rs
Converged-helper dups outside utils/: 0 (12 deferred-whitelisted)
```

- `git diff --check`

```text
<no output; exit 0>
```

本包不创建 JSON execution record，因此不需要 `checkpoint-audit --record` hash 字段自检。
