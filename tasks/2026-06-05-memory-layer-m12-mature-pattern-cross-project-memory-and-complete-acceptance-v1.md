# Task Package：Memory Layer M12 Mature Pattern Cross Project Memory And Complete Acceptance v1

状态：已完成。  
用途：实现中间版本记忆层 M12：成熟模式、跨项目记忆和完整验收。  
执行方式：一个较大但必须保守的批次完成；开发重点是在 M1-M11 已完成的正式记忆、候选、观察、任务包注入、生命周期、实体关系治理和维护 lint 之上，补齐 `MaturePatternCandidate`、`MemoryClusterReport` 或等价跨项目主题报告、用户确认后的成熟模式 / 全局记忆受控写入，以及 M1-M12 记忆系统完整验收摘要。

预期回收记录：

- `evidence/2026-06-05-memory-layer-m12-mature-pattern-cross-project-memory-and-complete-acceptance-v1.md`
- `handoffs/2026-06-05-memory-layer-m12-mature-pattern-cross-project-memory-and-complete-acceptance-v1-result.md`

## 1. 先说薄弱点

- M11 已能发现 mature pattern signal，但它只是 finding / review 信号，不是成熟模式候选、正式记忆、全局规则或技能。
- M10 已有实体 / 关系解释，但 relation / graph / similarity 不能直接变成跨项目事实，也不能改变 worker 输入。
- M9 已有正式记忆 lifecycle，但成熟模式和跨项目记忆涉及更高影响范围，必须由用户确认，项目主管不能代替用户确认。
- `MemoryClusterReport` 是派生理解索引或报告，不是正式记忆；报告摘要不能被 worker 当作事实或规则。
- “完整验收”容易越界成 M13 最终验收；M12 只接受为 M1-M12 记忆系统能力集成验收材料完成，M13 仍要做最终权威验收和最终结论冻结。
- M12 会影响记忆中心、待办 / 确认、任务包召回解释和可能的管理健康摘要，必须落实 UI 显示边界固定章节。

## 2. 任务目标

建立成熟模式和跨项目记忆的受控闭环：

```text
ObservationStore / MemoryCandidateStore / FormalMemoryStore / MemoryLintStore / EntityRelationStore
-> mature pattern signal / cross-project theme derivation
-> MaturePatternCandidate
-> MemoryClusterReport
-> secretary or global director review summary
-> user confirmation
-> formal mature pattern / global memory with source + version + audit
-> TaskMemoryPacketBuilder scope/permission recall
-> M1-M12 complete memory acceptance summary
```

M12 完成后可以说：

- 系统能从重复错误、重复流程、重复审查意见、维护 mature pattern signal 和跨项目主题中生成成熟模式候选。
- 系统有 `MemoryClusterReport` 或等价跨项目主题报告，能下钻来源，但报告不是正式事实。
- 秘书或全局主管可以汇总成熟模式和跨项目异常，但不能直接写正式全局记忆。
- 用户确认后，成熟模式 / 全局记忆才能通过正式记忆状态机写入，并生成 version、audit 和 source refs。
- 未确认成熟模式候选、cluster report、LLM summary、relation candidate 或 maintenance report 不能进入 worker 任务包当规则。
- 已确认且权限允许的成熟模式正式记忆可以按 scope / model export policy 进入合适任务包。
- M1-M12 记忆系统完整验收摘要能区分观察、候选、正式记忆、版本、来源、权限、冲突、审计、关系、维护、成熟模式和任务包召回链路。

M12 完成后仍不能说：

- M13 中间版本记忆系统最终验收完成。
- 最终蓝图完整记忆系统完成。
- 成熟模式能自动变成技能、全局规则或正式记忆。
- 跨项目摘要能直接影响项目 worker。
- GraphRAG、向量库、图数据库、自动索引重建或完整理解地图完成。
- 真实 worker / Codex 已执行。

## 3. 前置条件

必须已完成：

- `tasks/2026-06-03-memory-layer-m1-formal-memory-store-and-audit-v1.md`
- `tasks/2026-06-03-memory-layer-m1-1-formal-memory-context-binding-guard-v1.md`
- `tasks/2026-06-03-memory-layer-m2-candidate-to-formal-adoption-v1.md`
- `tasks/2026-06-04-memory-layer-m3-observation-store-and-workflow-entry-v1.md`
- `tasks/2026-06-04-memory-layer-m4-task-memory-packet-builder-and-preview-v1.md`
- `tasks/2026-06-04-memory-layer-m5-conflict-and-memory-lint-minimal-blocking-v1.md`
- `tasks/2026-06-04-memory-layer-m6-workflow-task-package-injection-and-end-to-end-loop-v1.md`
- `tasks/2026-06-05-memory-layer-m7-memory-management-ui-minimal-entry-v1.md`
- `tasks/2026-06-05-memory-layer-m8-knowledge-base-and-obsidian-compatible-interface-placeholder-v1.md`
- `tasks/2026-06-05-memory-layer-m9-formal-memory-lifecycle-operations-v1.md`
- `tasks/2026-06-05-memory-layer-m10-entity-and-relation-governance-v1.md`
- `tasks/2026-06-05-memory-layer-m11-maintenance-jobs-and-memory-lint-v1.md`

开始前必须复核：

- M11 `mature_pattern_signal` 只是 finding，不是成熟模式候选正式化。
- M2 / M9 是正式记忆写入和生命周期受控路径；M12 不得绕过 source、version、audit、permission 和 confirmation。
- M10 relation explanation 只能解释任务包召回，不是跨项目主题事实源。
- M8 knowledge doc 只能作为来源和候选材料，不能直接写正式记忆。
- M7 记忆中心是可理解入口，不是 raw governance console。

## 4. 必须先读

当前入口：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/middleware-version-development-plan-v1.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `docs/plans/memory-layer-implementation-slice-v1.md`
- `docs/memory-layer-design-v1.md`
- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`

前置记录：

- `evidence/2026-06-05-memory-layer-m9-formal-memory-lifecycle-operations-v1.md`
- `handoffs/2026-06-05-memory-layer-m9-formal-memory-lifecycle-operations-v1-result.md`
- `evidence/2026-06-05-memory-layer-m10-entity-and-relation-governance-v1.md`
- `handoffs/2026-06-05-memory-layer-m10-entity-and-relation-governance-v1-result.md`
- `evidence/2026-06-05-memory-layer-m11-maintenance-jobs-and-memory-lint-v1.md`
- `handoffs/2026-06-05-memory-layer-m11-maintenance-jobs-and-memory-lint-v1-result.md`

当前实现重点：

- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/formal_memory_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/formal_memory_lifecycle.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_candidate_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/observation_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_lint_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_lint_engine.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_entity_relation_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_entity_relation_governance.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/task_memory_packet_builder.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/task_memory_injection.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/control_core.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src/App.tsx`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `prototypes/productized-desktop-shell/src/lib/memoryCenter.ts`
- `prototypes/productized-desktop-shell/src/views/MemoryCenterView.tsx`
- `prototypes/productized-desktop-shell/src/components/PermissionDialog.tsx`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

## 5. 范围

允许：

- 新增或扩展后端类型，例如：
  - `MaturePatternCandidate`
  - `MaturePatternCandidateStatus`
  - `MemoryClusterReport`
  - `MemoryClusterMemberRef`
  - `CrossProjectMemoryTheme`
  - `MaturePatternReviewInput`
  - `MaturePatternDecisionOutput`
  - `MemorySystemAcceptanceSummary`
  - `MemorySystemAcceptanceGate`
- 优先新增独立 sidecar，例如 `memory-patterns.v1.json`，用于保存 mature pattern candidates、cluster reports、用户确认决定和 pattern audit refs。
- 如果执行者认为必须复用既有 `memory-candidates.v1.json` 或 `memory-lint.v1.json`，必须在 evidence 说明为何不新增 sidecar，并证明不会破坏旧 schema 兼容。
- 从以下材料派生成熟模式候选和跨项目主题报告：
  - confirmed / active formal memories
  - confirmed memory candidates
  - recorded observations
  - M11 maintenance reports 和 mature pattern signal
  - M10 confirmed relation 和 entity alias / dedupe signals
  - task memory packet artifacts 的正式 memory refs
  - workflow audit 中已经结构化确认的结果摘要
- 支持 deterministic 的成熟模式候选生成，不依赖 LLM 才能通过测试。
- 支持 `MemoryClusterReport` 或等价跨项目主题报告，必须包含 member refs、source refs、scope、status、staleness 和 display summary。
- 支持用户确认成熟模式候选：
  - 用户确认后才允许写正式 mature pattern / global memory。
  - 写正式记忆必须使用后端受控 FormalMemoryStore / lifecycle 路径，生成 record、version、audit 和 source refs。
  - 项目主管可以 review / request changes，但不能替代用户确认跨项目、全局、用户偏好或 mature pattern 记忆。
- 支持用户拒绝 / 隔离成熟模式候选，拒绝不删除来源材料，不改正式记忆。
- 支持任务包召回：
  - 未确认 mature pattern candidate 不进入 task memory packet included list。
  - `MemoryClusterReport` 不进入 task memory packet included list。
  - 用户确认后的 active formal mature pattern memory 可按 scope、permission、model export policy、lint blocking guard 进入任务包。
  - 任务包必须解释成熟模式记忆为什么入选或为什么被排除。
- 支持 M1-M12 完整验收摘要：
  - observation -> candidate -> formal memory -> version -> audit -> task packet recall。
  - candidate 不冒充 formal memory。
  - source / permission / conflict / lifecycle / relation / maintenance / mature pattern / cluster report 边界。
  - UI 可理解性和真实窗口验收状态。
- 新增 Rust 单测和前端离线测试。
- 更新 evidence / handoff / `CURRENT.md` / `tasks/README.md` / `AUTHORITY.md` / `STAGE_PLAN.md` / 阶段计划。

本任务显式授权的数据变更：

- 允许新增并写 `memory-patterns.v1.json` 或等价 mature pattern / cluster report sidecar。
- 允许读取 `formal-memories.v1.json`、`memory-candidates.v1.json`、`observations.v1.json`、`memory-lint.v1.json`、`memory-entity-relations.v1.json`、workflow state 和 task package artifacts。
- 允许在用户确认 mature pattern / global memory 后，通过正式记忆受控 store 写 `formal-memories.v1.json`，并生成 version / audit / source refs。
- 允许任务记忆包 builder 读取用户确认后的 formal mature pattern memory。
- 允许写 mature pattern / cluster report audit refs 到 M12 sidecar。
- 不允许未确认 mature pattern candidate 写正式记忆。
- 不允许 `MemoryClusterReport` 直接写正式记忆。
- 不允许写 `workflow-state.v0.json` 顶层结构或新增顶层数组。
- 不允许迁移数据库。

禁止：

- 不执行真实 worker。
- 不执行真实 Codex。
- 不执行 `codex exec` / `codex exec resume`。
- 不读写 `/Users/yoyi/.codex`。
- 不扫描完整 transcript。
- 不让 mature pattern signal 自动成为正式记忆、全局记忆、技能或规则。
- 不让跨项目摘要、cluster report、relation candidate、LLM summary、knowledge hit、observation 或 maintenance report 直接影响 worker。
- 不让项目主管确认替代用户确认跨项目 / 全局 / mature pattern 记忆。
- 不把 `MemoryClusterReport` 当正式事实源。
- 不接向量库、图数据库、GraphRAG 或自动索引重建系统。
- 不自动生成技能。
- 不把 M12 说成 M13 最终验收完成。

如果执行者认为必须自动技能化、接 GraphRAG / 向量库 / 图数据库、读取 `/Users/yoyi/.codex`、执行真实 Codex、跳过用户确认或绕过正式记忆状态机，必须停下回传。

## 5.1 UI 显示边界确认

本任务会改前端：

- [ ] 不改前端、不改读模型、不改 UI 文案。
- [x] 改前端类型 / Tauri wrapper。
- [x] 改读模型摘要或状态显示。
- [x] 改已有页面局部 UI。
- [x] 新增按钮或确认动作，但不新增一级入口、右侧顶级入口或项目页 tab。

已读取：

- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`
- `CURRENT.md`
- `tasks/README.md`
- `docs/memory-layer-design-v1.md`
- `docs/plans/memory-layer-implementation-slice-v1.md`

本任务允许显示：

- 成熟模式候选摘要。
- 跨项目主题报告摘要。
- 候选来源数量、member refs 数量和 source refs 摘要。
- 用户确认要求。
- “候选未确认，不会进入任务包”的边界文案。
- 用户确认后的 formal mature pattern memory 入包资格摘要。
- M1-M12 完整验收 gate 摘要。
- 建议动作，例如“建议用户确认”“建议拒绝 / 隔离”“建议补来源”。

本任务禁止显示：

- 不新增一级入口。
- 不新增右侧顶级入口。
- 不新增项目页 tab。
- 不在画布主区域铺 mature pattern、cluster report、raw audit 或完整验收表。
- 不显示完整 sidecar、raw JSON、raw audit、完整索引日志或数据库路径大表。
- 不把 cluster report 显示为正式事实。
- 不把 mature pattern candidate 显示为已生效规则。
- 不显示“自动成为技能”“自动成为全局规则”“自动写入全局记忆”“跨项目摘要已注入任务包”等误导文案。

显示位置：

- 一级入口：不新增；主要复用 `记忆`。
- 右侧入口：不新增；确认事项进入既有待办 / 秘书摘要，健康摘要可进既有管理内部。
- 项目页：只允许轻量提示，不新增 tab，不占用工作流画布主区域。
- 画布：不改画布主区域。
- 记忆入口：显示成熟模式候选、跨项目主题报告、用户确认状态、正式 mature pattern memory 入包资格和 M1-M12 验收摘要。
- 知识库入口：只允许显示来源引用相关摘要，不直接编辑正式记忆。
- 智能体入口：不改。
- 管理入口：可显示验收 gate / 健康摘要，不显示 raw store / raw audit。

中间版本范围：

- 本轮必须落地：`MaturePatternCandidate`、`MemoryClusterReport` 或等价报告、用户确认链路、正式 mature pattern / global memory 受控写入、任务包召回边界、M1-M12 完整验收摘要。
- 本轮只做读模型 / 摘要：跨项目主题解释、聚类报告摘要、秘书 / 全局主管 review summary、验收 gate 可见化。
- 本轮后置：GraphRAG、向量库、图数据库、自动技能生成、完整理解地图、M13 最终验收权威冻结。

后端和数据依赖：

- 需要后端正式读写模型：必须通过 Rust store wrapper 写 M12 sidecar 和 formal memory。
- 需要审计 / 日志 / 权限 / 状态机：用户确认、正式记忆写入、适用范围变化和拒绝 / 隔离都必须写 audit 或 audit ref。
- 不能用假数据伪装：不能伪造成熟模式已确认、全局记忆已写入、任务包已注入、GraphRAG 已完成或 M13 已验收。

UI 文案边界：

- 禁止说：`自动成为技能`、`自动成为全局规则`、`自动写入全局记忆`、`跨项目摘要已注入任务包`、`聚类报告就是事实`、`成熟模式已生效`、`M13 已完成`、`中间版本记忆系统最终验收完成`。
- 允许说：`成熟模式候选`、`跨项目主题报告`、`需要用户确认`、`候选未确认，不会进入任务包`、`用户确认后才可写正式记忆`、`报告可下钻来源，但不是正式事实`、`M1-M12 验收摘要`。

验收：

- 类型检查：`npm run typecheck`
- 离线交互测试：`npm run test:offline-interaction`
- 构建：`npm run build`
- 真实窗口 / 截图验收：涉及 `记忆` 入口和确认弹层，必须做真实浏览器或 Tauri 截图验收；如果没有可用截图工具，不能声称 UI 验收完成。
- 未验收项必须写入 evidence / handoff。

## 6. 维护和确认规则

最低规则：

- mature pattern candidate：只能由重复流程、重复失败、重复审查意见、M11 mature pattern signal 或用户多次采纳同类建议派生；不能由单条普通聊天直接生成。
- MemoryClusterReport：只能作为派生报告；必须保留 member refs / source refs，不能替代来源。
- user confirmation：跨项目、全局、用户偏好、mature pattern、高风险记忆必须用户确认。
- project director：可以 review、补充影响面、请求修改或拒绝低风险项目内候选；不能确认跨项目 / 全局 / mature pattern 正式化。
- secretary：可以解释、整理、提醒和生成 review summary；不能确认或写正式记忆。
- task packet：只允许召回 active formal memory；mature pattern candidate 和 cluster report 默认不进入 included list。
- lifecycle：适用范围变化、上升为全局、下沉为项目、废弃或冻结必须走 M9 lifecycle 或等价正式记忆受控路径。

## 7. 实施建议

建议按以下顺序实现：

1. 新增 M12 类型和 sidecar store，保持 revision、lock、备份、原子写、损坏 JSON 拒绝覆盖和 expected revision guard。
2. 新增 deterministic mature pattern / cluster report 派生 helper，先覆盖重复候选、重复 observation、重复 maintenance signal 和跨项目主题 member refs。
3. 新增 preview command：只读生成 candidates / reports，不写正式记忆。
4. 新增 record decision command：用户确认、拒绝、隔离、request changes；用户确认 mature pattern / global memory 时通过 formal memory 受控路径写 record / version / audit。
5. 扩展 task memory packet builder，只召回已确认 active formal mature pattern memory；candidate / report 只进入 excluded 或 review materials。
6. 扩展 memory center 读模型和确认弹层，显示候选、报告、确认权、入包资格和 M1-M12 验收摘要。
7. 补 Rust 单测、前端离线测试、禁止文案搜索和回收文档。

如果实现过大，可以拆为 M12a / M12b，但必须回传：MaturePatternCandidate、MemoryClusterReport、用户确认正式化、task packet recall、M1-M12 acceptance summary 各自完成到哪里，不能把半包说成 M12 完整完成。

## 8. 验收

必须通过：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
cargo test --lib mature_pattern
cargo test --lib memory_cluster
cargo test --lib task_memory_packet
cargo test --lib formal_memory
cargo test --lib memory_lint
cargo test --lib memory_entity_relation
cargo test --lib
rustfmt --check src/mature_pattern_store.rs src/mature_pattern_governance.rs src/task_memory_packet_builder.rs src/task_memory_injection.rs src/formal_memory_store.rs src/formal_memory_lifecycle.rs src/memory_lint_store.rs src/memory_entity_relation_store.rs src/control_core.rs src/commands.rs src/types.rs src/lib.rs
```

如果实际文件名不同，必须把等价新增 / 修改文件列入 `rustfmt --check`。

必须覆盖的场景：

- 多次相似失败或重复审查意见能生成 mature pattern candidate。
- M11 mature pattern signal 能进入 mature pattern candidate 派生材料，但不会自动正式化。
- MemoryClusterReport 能下钻 member refs / source refs，且不替代来源。
- 未确认 mature pattern candidate 不能进入 task memory packet included list。
- 未确认 MemoryClusterReport 不能进入 task memory packet included list。
- 项目主管不能确认跨项目 / 全局 / mature pattern 正式化。
- 用户确认 mature pattern candidate 后，正式记忆写入必须有 source refs、version、audit、scope 和 model export policy。
- 用户拒绝 / 隔离 mature pattern candidate 不删除来源材料、不改正式记忆。
- 用户确认后的 active formal mature pattern memory 能按权限进入合适 task memory packet。
- 冲突、blocking lint、权限不满足或 model export policy 不允许时，成熟模式正式记忆仍不能进入任务包。
- 适用范围变化必须产生审计或 lifecycle 记录。
- M1-M12 验收摘要能列出每个 gate 的通过、阻断和后置项。
- 损坏 JSON 不会被覆盖。
- expected revision 不匹配会拒绝写入。
- UI 不出现禁止文案。

## 9. evidence / handoff 要求

M12 完成后必须新增：

- `evidence/2026-06-05-memory-layer-m12-mature-pattern-cross-project-memory-and-complete-acceptance-v1.md`
- `handoffs/2026-06-05-memory-layer-m12-mature-pattern-cross-project-memory-and-complete-acceptance-v1-result.md`

evidence 必须记录：

- 实际新增 / 修改的 mature pattern / cluster report store、治理 helper、类型、commands、Tauri wrapper、UI 文件。
- MaturePatternCandidate、MemoryClusterReport、用户确认、正式记忆写入、task packet recall 和 M1-M12 acceptance summary 的验收结果。
- mature pattern signal、cluster report、relation candidate、knowledge hit、LLM summary 或 maintenance report 没有绕过正式记忆状态机的验证。
- 用户确认权、项目主管边界和秘书边界验证。
- UI 摘要、禁止文案和真实窗口 / 截图验收情况。
- 验证命令和结果。
- 边界：未执行真实 worker / Codex，未读写 `/Users/yoyi/.codex`，未接向量库 / 图数据库 / GraphRAG，未把 M12 说成 M13 最终验收完成。

handoff 必须写清：

- M12 接受为什么。
- M12 不接受为什么。
- MaturePatternCandidate / MemoryClusterReport / 用户确认正式化 / task packet recall / M1-M12 acceptance summary 哪些已完成。
- 下一步应进入 M13，还是先补 M12 剩余 / 截图缺口。

## 10. Stop 条件

遇到以下情况必须停下回传：

- 需要读写 `/Users/yoyi/.codex`。
- 需要执行真实 `codex exec` 或 `codex exec resume`。
- 需要启动真实 worker。
- 需要扫描完整 transcript。
- 需要自动技能化。
- 需要自动写全局记忆。
- 需要项目主管替代用户确认跨项目 / 全局 / mature pattern 记忆。
- 需要让 mature pattern candidate、cluster report、maintenance report、relation candidate、observation、knowledge hit、LLM summary 或 graph/index report 直接影响 worker。
- 需要接向量库、图数据库、GraphRAG 或自动索引重建系统。
- 需要新增一级入口、右侧顶级入口或项目页 tab。
- 需要把 M12 说成 M13 最终验收完成。
- 发现本任务与 `docs/workbench-frontend-display-boundary-v1.md`、`docs/memory-layer-design-v1.md` 或 `docs/plans/memory-layer-implementation-slice-v1.md` 冲突。

## 11. 回收口径

完成后接受为：

- M12 成熟模式、跨项目记忆和 M1-M12 完整验收摘要完成。
- MaturePatternCandidate 和 MemoryClusterReport 或等价跨项目主题报告可用。
- 用户确认后，成熟模式 / 全局记忆能通过正式记忆状态机写入，并按权限进入合适任务包。
- M1-M12 记忆系统 gate 摘要可解释观察、候选、正式记忆、版本、来源、权限、冲突、审计、关系、维护、成熟模式和召回链路。

完成后不接受为：

- M13 中间版本记忆系统最终验收完成。
- 最终蓝图完整记忆系统完成。
- 自动技能化完成。
- mature pattern 自动成为正式记忆、技能或全局规则完成。
- 跨项目摘要直接影响项目 worker 完成。
- GraphRAG、向量库、图数据库、自动索引重建系统或完整理解地图完成。
- 真实 worker / Codex 已执行。
