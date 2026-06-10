# Task Package：Memory Layer M10 Entity And Relation Governance v1

状态：已完成。  
用途：实现中间版本记忆层 M10：实体和关系治理。  
执行方式：一个较大但必须保守的批次完成；开发重点是 `MemoryEntityRegistry`、`MemoryRelation`、`MemoryRelationCandidate` 的最小可用版本，让别名、重复对象、关系候选和已确认关系能被审计和解释，但不做自动图谱推断、向量库、GraphRAG 或自动合并。

完成记录：

- `evidence/2026-06-05-memory-layer-m10-entity-and-relation-governance-v1.md`
- `handoffs/2026-06-05-memory-layer-m10-entity-and-relation-governance-v1-result.md`

## 1. 先说薄弱点

- M1-M9 已完成正式记忆创建、采纳、观察、任务包注入、记忆中心、知识库边界和正式记忆生命周期，但正式记忆之间还缺少可审计的实体 / 关系治理层。
- 当前 M9 的 merge / split 只允许对明确选中的正式记忆做生命周期操作，不等于语义 dedupe、实体合并或图谱关系确认。
- 如果没有 M10，同一工具、同一项目、同一模型、同一文档可能以多个名字出现在记忆、知识库、任务包和工作流里，任务包只能靠文本相关性解释召回，难以说明“为什么这条记忆相关”。
- 关系治理最容易跑偏：LLM 推断、相似度命中、图谱边、因果关系看起来像事实，但默认只能进入候选；不能直接影响 worker 或任务包 included list。
- M10 会触及记忆入口、项目轻量摘要、任务包解释和读模型展示，必须落实 UI 显示边界固定章节。

## 2. 任务目标

建立实体和关系治理的最小闭环：

```text
FormalMemory / MemoryCandidate / Observation / KnowledgeDoc / TaskPackage refs
-> entity extraction / explicit registration
-> alias and dedupe candidates
-> relation candidates
-> project director or user confirmation
-> confirmed MemoryRelation
-> task memory packet relation explanation
-> MemoryCenter human-readable relation summary
```

M10 完成后可以说：

- 系统有最小 `MemoryEntityRegistry`，能表示项目、会话、角色、文档、工具、模型、harness、建议方案等实体。
- 同一对象的多个名字可以形成 alias / dedupe 候选，但不会自动合并。
- 支持 `entity`、`temporal`、`causal`、`semantic` 四类基础关系。
- LLM inferred、similarity hit、ambiguous relation 默认只进入关系候选。
- 因果关系默认需要项目主管或用户确认；跨项目 / 高影响关系必须用户确认。
- 已确认关系可以帮助任务包解释“为什么召回这条记忆”。
- 未确认、冲突、权限不允许或来源不足的关系不能作为正式事实影响 worker。

M10 完成后仍不能说：

- M11 维护任务完成。
- M12 成熟模式 / 跨项目记忆完成。
- M13 中间版本记忆系统最终验收完成。
- GraphRAG、向量库、图数据库或完整图谱编辑完成。
- LLM 能自动写正式关系。
- 相似度命中能自动合并实体。
- 因果关系可自动影响任务包。
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

开始前必须复核：

- FormalMemoryStore 不是关系图事实源；它只保存正式记忆、版本和审计。
- M9 lifecycle merge / split 不是语义实体合并。
- M4/M6 task memory packet builder 已能解释 included / excluded / review materials。
- M5 lint finding 已能阻断问题记忆进入任务包。
- M7 记忆中心默认不显示 raw graph。
- M8 知识库只提供资料来源和候选入口，不是正式记忆或正式关系来源。

## 4. 必须先读

当前入口：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `docs/middleware-version-development-plan-v1.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `docs/plans/memory-layer-implementation-slice-v1.md`
- `docs/memory-layer-design-v1.md`
- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`

前置记录：

- `evidence/2026-06-05-memory-layer-m8-knowledge-base-and-obsidian-compatible-interface-placeholder-v1.md`
- `handoffs/2026-06-05-memory-layer-m8-knowledge-base-and-obsidian-compatible-interface-placeholder-v1-result.md`
- `evidence/2026-06-05-memory-layer-m9-formal-memory-lifecycle-operations-v1.md`
- `handoffs/2026-06-05-memory-layer-m9-formal-memory-lifecycle-operations-v1-result.md`

当前实现重点：

- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/formal_memory_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/formal_memory_lifecycle.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_candidate_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/observation_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_lint_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/task_memory_packet_builder.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/task_memory_injection.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/control_core.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src/App.tsx`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `prototypes/productized-desktop-shell/src/lib/memoryCenter.ts`
- `prototypes/productized-desktop-shell/src/lib/knowledgeBase.ts`
- `prototypes/productized-desktop-shell/src/views/MemoryCenterView.tsx`
- `prototypes/productized-desktop-shell/src/views/KnowledgeBaseView.tsx`
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `prototypes/productized-desktop-shell/src/components/PermissionDialog.tsx`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

## 5. 范围

允许：

- 新增后端 store，例如 `memory_entity_relation_store.rs`，sidecar 可命名为 `memory-entity-relations.v1.json` 或等价清晰名称。
- 新增后端治理 helper，例如 `memory_entity_relation_governance.rs`。
- 新增或扩展类型：
  - `MemoryEntityRegistry`
  - `MemoryEntity`
  - `MemoryEntityAlias`
  - `MemoryEntityCandidate`
  - `MemoryEntityMergeCandidate`
  - `MemoryRelation`
  - `MemoryRelationCandidate`
  - `MemoryRelationKind`
  - `MemoryRelationSource`
  - `MemoryRelationStatus`
  - `MemoryRelationAuditEvent`
  - `MemoryRelationTaskExplanation`
- 支持实体类型：
  - `project`
  - `workflow`
  - `session`
  - `role`
  - `knowledge_doc`
  - `tool`
  - `model`
  - `harness`
  - `proposal`
  - `memory_record`
  - `memory_candidate`
- 支持关系类型：
  - `entity`
  - `temporal`
  - `causal`
  - `semantic`
- 支持来源类型：
  - `manual`
  - `formal_memory`
  - `memory_candidate`
  - `observation`
  - `knowledge_doc`
  - `task_package`
  - `llm_inferred`
  - `similarity_hit`
- 新增只读 / 写入命令，例如：
  - `load_memory_entity_relation_store`
  - `preview_memory_entity_relation_candidates`
  - `record_memory_entity_alias_decision`
  - `record_memory_relation_candidate_decision`
  - `record_memory_entity_merge_decision`
- 允许写新的 entity / relation sidecar，必须带 revision、lock、备份、原子写和损坏 JSON 拒绝覆盖。
- 允许从现有正式记忆、候选、观察、知识库资料和任务包引用派生候选。
- 允许人工确认 alias、entity merge、relation candidate。
- 允许已确认关系进入 task memory packet explanation，但必须经过状态、权限、来源、冲突和模型外发检查。
- 新增或调整 MemoryCenter / KnowledgeBase / ProjectsView 的只读关系摘要。
- 新增 Rust 单测和前端离线测试。
- 更新 evidence / handoff / `CURRENT.md` / `tasks/README.md` / `AUTHORITY.md` / `STAGE_PLAN.md` / 阶段计划。

本任务显式授权的数据变更：

- 允许新增并写入 entity / relation sidecar，例如 `memory-entity-relations.v1.json`。
- 允许读取 `formal-memories.v1.json`、`memory-candidates.v1.json`、`observations.v1.json`、`memory-lint.v1.json`、workflow state 和 task package artifacts，用于候选和关系解释。
- 允许 task memory packet builder 读取已确认关系，用于解释召回原因。
- 不允许写 `formal-memories.v1.json`，除非只是读取 M9 lifecycle 结果；M10 默认不改正式记忆。
- 不允许写 `memory-candidates.v1.json`。
- 不允许写 `observations.v1.json`。
- 不允许写 `memory-lint.v1.json`。
- 不允许写 `workflow-state.v0.json`。
- 不允许新增 `workflow-state.v0.json` 顶层数组。
- 不允许迁移数据库。

禁止：

- 不执行真实 worker。
- 不执行真实 Codex。
- 不执行 `codex exec` / `codex exec resume`。
- 不读写 `/Users/yoyi/.codex`。
- 不接向量库或图数据库。
- 不做 GraphRAG。
- 不把图谱推断直接当正式关系。
- 不把关系索引当权威来源。
- 不让相似度命中自行合并实体。
- 不让 LLM inferred relation 直接进入 confirmed relation。
- 不让 ambiguous relation 影响任务包。
- 不让因果关系未经项目主管或用户确认就影响 worker。
- 不自动修改正式记忆生命周期；M9 已完成，M10 不继续改正式记忆。
- 不把 M10 说成中间版本完整记忆系统完成。

如果执行者认为必须引入向量库、图数据库、GraphRAG、自动实体合并或自动因果确认，必须停下回传，说明为什么不能留到后续独立任务。

## 5.1 UI 显示边界确认

本任务会改前端：

- [ ] 不改前端、不改读模型、不改 UI 文案。
- [x] 改前端类型 / Tauri wrapper。
- [x] 改读模型摘要或状态显示。
- [x] 改已有页面局部 UI。
- [x] 新增页面内实体 / 关系候选确认动作，但不新增一级入口 / 右侧顶级入口 / 项目页 tab。

已读取：

- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`
- `CURRENT.md`
- `tasks/README.md`
- `docs/memory-layer-design-v1.md`
- `docs/plans/memory-layer-implementation-slice-v1.md`

本任务允许显示：

- 实体摘要：项目、文档、工具、模型、harness、角色、建议方案等实体的人话名称。
- alias / dedupe 候选摘要。
- 关系候选摘要：关系类型、来源、置信来源、确认要求。
- 已确认关系摘要。
- 任务包关系解释：为什么某条记忆被召回。
- 明确边界文案：`LLM 推断关系只是候选`、`相似度命中不会自动合并实体`、`因果关系需要确认后才影响任务包`。

本任务禁止显示：

- 不新增一级入口。
- 不新增右侧顶级入口。
- 不新增项目页 tab。
- 不显示 raw graph、raw relation JSON、完整 sidecar、数据库路径大表或完整审计日志。
- 不显示“自动合并实体”“自动确认因果关系”“图谱已证明”“LLM 已确认关系”“GraphRAG 已接入”等误导文案。
- 不把关系候选显示为正式事实。
- 不把实体 registry 显示为正式记忆。

显示位置：

- 一级入口：不新增；主要复用 `记忆`，必要时在 `知识库` 显示来源关联摘要。
- 右侧入口：不改。
- 项目页：只允许项目相关实体 / 关系轻量摘要，不新增 tab，不占用工作流画布主区域。
- 画布：不改画布主区域。
- 记忆入口：显示实体 / 关系摘要、候选和确认动作。
- 知识库入口：只显示知识文档相关实体 / 关系摘要。
- 智能体入口：不改。
- 管理入口：不改。

中间版本范围：

- 本轮必须落地：最小 entity registry、alias / dedupe 候选、relation candidate、confirmed relation、任务包关系解释。
- 本轮只做读模型 / 摘要：实体 / 关系影响面、项目轻量摘要、知识库来源关系。
- 本轮后置：向量库、图数据库、GraphRAG、自动聚类、复杂关系图编辑、全局实体治理控制台。

后端和数据依赖：

- 需要后端正式读写模型：必须通过 Rust sidecar wrapper 写 entity / relation store。
- 需要审计 / 日志 / 权限 / 状态机：每个候选决定、关系确认、实体合并都必须写审计事件或等价 audit record。
- 不能用假数据伪装：不能伪造 confirmed relation、任务包关系解释、实体合并、因果确认或 LLM 推断来源。

UI 文案边界：

- 禁止说：`自动合并实体`、`自动确认关系`、`图谱已证明`、`LLM 已确认关系`、`相似度已合并实体`、`GraphRAG 已接入`、`关系候选已成为事实`、`中间版本记忆层已完成`。
- 允许说：`实体候选`、`关系候选`、`已确认关系`、`待确认因果关系`、`相似度命中仅作候选`、`LLM 推断仅作候选`、`已确认关系用于解释召回原因`。

验收：

- 类型检查：`npm run typecheck`
- 离线交互测试：`npm run test:offline-interaction`
- 构建：`npm run build`
- 真实窗口 / 截图验收：涉及记忆 / 知识库关系摘要和确认动作，必须做真实浏览器或 Tauri 截图验收；如果没有可用截图工具，不能声称 UI 验收完成。
- 未验收项必须写入 evidence / handoff。

## 6. 确认权规则

最低规则：

- alias / dedupe 候选：低风险本项目实体可由项目主管确认；涉及用户偏好、全局蓝图、跨项目实体或安全边界必须用户确认。
- entity merge：默认需要项目主管确认；跨项目或高影响 merge 必须用户确认。
- semantic relation：默认需要项目主管确认。
- temporal relation：明确来源可由项目主管确认；跨项目或高影响需要用户确认。
- causal relation：默认需要项目主管或用户确认；高风险、跨项目、安全边界和全局规则必须用户确认。
- llm_inferred / ambiguous relation：不能直接确认，必须先转为候选并保留来源和理由。

秘书边界：

- 秘书可以解释候选、整理影响面、提醒确认事项。
- 秘书不能确认实体合并、因果关系或正式关系。

## 7. 实施建议

建议按以下顺序实现：

1. 后端 entity / relation sidecar 和类型。
2. 从现有 formal memory、knowledge docs、task package refs 派生 deterministic entity candidates。
3. 支持 alias / dedupe candidate 和确认。
4. 支持 relation candidate 和确认 / 拒绝 / 隔离。
5. 支持 confirmed relation 进入 task memory packet explanation，但不改变正式记忆本体。
6. 接 Tauri wrapper 和 TS 类型。
7. 在 MemoryCenter / KnowledgeBase 增加实体 / 关系摘要和候选确认动作。
8. 补 Rust 单测、前端离线测试、禁止文案搜索和回收文档。

如果实现变得过大，可以先拆为 M10a / M10b，但必须回传：实体 registry、alias/dedupe、relation candidate、confirmed relation、task packet explanation 各自完成到哪里，不能把半包说成 M10 完整完成。

## 8. 验收

必须通过：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
cargo test --lib memory_entity_relation
cargo test --lib task_memory_packet
cargo test --lib formal_memory
cargo test --lib memory_lint
cargo test --lib
rustfmt --check src/memory_entity_relation_store.rs src/memory_entity_relation_governance.rs src/task_memory_packet_builder.rs src/task_memory_injection.rs src/formal_memory_store.rs src/memory_lint_store.rs src/control_core.rs src/commands.rs src/types.rs src/lib.rs
```

如果实际文件名不同，必须把等价新增 / 修改文件列入 `rustfmt --check`。

必须覆盖的场景：

- 同一工具两个别名能被提示为同一实体候选。
- 相似度命中不会自动合并实体。
- LLM 推断的因果关系只进入候选关系。
- 因果关系确认后才能进入 confirmed relation。
- 已确认关系能帮助任务包解释“为什么召回这条记忆”。
- 冲突或未审关系不能作为正式事实影响 worker。
- 关系来源权限不允许时不能进入任务包解释。
- 损坏 JSON 不会被覆盖。
- expected revision 不匹配会拒绝写入。
- UI 不出现禁止文案。

## 9. evidence / handoff 要求

M10 完成后必须新增：

- `evidence/2026-06-05-memory-layer-m10-entity-and-relation-governance-v1.md`
- `handoffs/2026-06-05-memory-layer-m10-entity-and-relation-governance-v1-result.md`

evidence 必须记录：

- 实际新增 / 修改的 entity / relation store、治理 helper、类型、commands、Tauri wrapper、UI 文件。
- alias / dedupe candidate、relation candidate、confirmed relation、task packet explanation 的验收结果。
- LLM inferred / similarity / ambiguous relation 没有直接成为 confirmed relation 的验证。
- UI 确认权、候选展示和禁止文案验证。
- 是否做了真实浏览器或 Tauri 截图验收；如果没有，必须明确写“真实窗口 / 截图验收未完成”。
- 验证命令和结果。
- 边界：未接向量库、未接图数据库、未做 GraphRAG、未自动合并实体、未执行真实 worker / Codex、未读写 `/Users/yoyi/.codex`。

handoff 必须写清：

- M10 接受为什么。
- M10 不接受为什么。
- 若拆成 M10a / M10b，哪些实体 / 关系能力已完成，哪些未完成。
- 下一步应进入 M11 还是先补 M10 剩余 / 截图缺口。

## 10. Stop 条件

遇到以下情况必须停下回传：

- 需要读写 `/Users/yoyi/.codex`。
- 需要执行真实 `codex exec` 或 `codex exec resume`。
- 需要启动真实 worker。
- 需要接向量库、图数据库或 GraphRAG。
- 需要自动合并实体。
- 需要把 LLM 推断、相似度命中或图谱边直接写成 confirmed relation。
- 需要让因果关系未经确认影响任务包或 worker。
- 需要把 relation store 当成权威事实源覆盖 FormalMemoryStore。
- 需要修改正式记忆生命周期状态。
- 需要新增一级入口、右侧顶级入口或项目页 tab。
- 发现本任务与 `docs/workbench-frontend-display-boundary-v1.md`、`docs/memory-layer-design-v1.md` 或 `docs/plans/memory-layer-implementation-slice-v1.md` 冲突。

## 11. 回收口径

完成后接受为：

- M10 实体和关系治理完成。
- 最小 `MemoryEntityRegistry`、`MemoryRelationCandidate` 和 `MemoryRelation` 可用。
- alias / dedupe 候选、关系候选、已确认关系和任务包关系解释形成受控闭环。

完成后不接受为：

- M11 维护任务完成。
- M12 成熟模式、跨项目记忆和完整验收完成。
- M13 中间版本记忆系统最终验收完成。
- 向量库、图数据库、GraphRAG 或完整图谱编辑完成。
- 自动实体合并、自动因果确认或 LLM 自动写关系完成。
- 真实 worker / Codex 已执行。
