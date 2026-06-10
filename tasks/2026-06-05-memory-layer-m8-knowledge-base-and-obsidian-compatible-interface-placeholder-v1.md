# Task Package：Memory Layer M8 Knowledge Base And Obsidian-compatible Interface Placeholder v1

状态：已完成。  
用途：实现中间版本记忆层 M8：知识库 / Obsidian-compatible 接口占位和边界。  
执行方式：一个中等批次完成；开发重点是知识库材料、来源引用、记忆候选入口和正式记忆反向引用的最小闭环，不做 Obsidian 原生同步、不做自动 vault 扫描、不做正式记忆生命周期操作。

完成记录：

- `evidence/2026-06-05-memory-layer-m8-knowledge-base-and-obsidian-compatible-interface-placeholder-v1.md`
- `handoffs/2026-06-05-memory-layer-m8-knowledge-base-and-obsidian-compatible-interface-placeholder-v1-result.md`

## 1. 先说薄弱点

- M7 已把全局 `记忆` 入口升级为只读记忆中心，但 `知识库` 入口仍是 placeholder，只把项目 `authority_files` 粗略铺出来。
- 现有类型里已经出现 `knowledge_doc`、`knowledge_material`、`knowledge_hit`、`knowledge_reference` 和 task package `available_knowledge_refs`，但缺少能让用户理解“知识库资料”和“正式记忆”边界的最小界面。
- 如果 M8 边界不收紧，执行者很容易把 Obsidian vault、Graph、Canvas、Bases、向量命中或 LLM 摘要直接当正式记忆，绕过 M1-M7 已经建立的来源、版本、审计、权限和候选流程。
- M8 是接口占位和边界，不是 Obsidian 原生功能深度内置；不能把“可兼容 Obsidian 风格资料”说成“已接入 Obsidian 原生同步”。
- M8 会触及 `知识库` 一级入口、记忆来源引用和候选生成入口，必须落实 UI 显示边界固定章节。

## 2. 任务目标

建立知识库和记忆层之间的最小受控接口：

```text
Project authority files / knowledge refs
-> KnowledgeDocumentReadModel
-> 知识库入口展示资料 / 项目文档 / 来源锚点
-> 明确知识库材料不是正式记忆
-> 显式从知识库材料提出 MemoryCandidate
-> FormalMemory source_refs 可引用 knowledge_doc
-> 记忆中心 / 知识库详情可反向显示引用关系
```

M8 完成后可以说：

- `知识库` 一级入口不再只是 placeholder，而是能展示最小知识库资料列表、项目归属、来源类型和引用状态。
- 知识库文档详情能显示关联的正式记忆、候选记忆和任务包知识引用摘要。
- 正式记忆详情或记忆中心能识别 `knowledge_doc` 来源，并可显示“来自知识库资料”的反向引用。
- 用户 / 项目主管可以从明确选中的知识库材料提出记忆候选，写入 `memory-candidates.v1.json`，但候选仍不是正式记忆。
- UI 文案能明确区分：知识库资料、知识命中、记忆候选、正式记忆。
- Obsidian-compatible 只接受为边界和占位：可表达 vault / markdown 资料方向，但不执行原生 Obsidian CLI、插件、图谱或自动同步。

M8 完成后仍不能说：

- Obsidian 原生能力已接入。
- vault 自动扫描完成。
- 知识库文档会自动进入长期记忆。
- 知识命中、Markdown 摘要、Canvas / Graph / Bases 结果已成为正式记忆。
- 正式记忆生命周期操作完成。
- 实体关系治理、维护任务、成熟模式、跨项目记忆或 M13 最终验收完成。
- 中间版本完整记忆系统完成。

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

开始前必须复核：

- 当前 `知识库` 一级入口已经存在，不需要新增一级导航。
- 当前 `App.tsx` 的 `knowledge` view 仍是 `SourceStylePlaceholder`，可作为 M8 的最小 UI 落点。
- 当前 `WorkbenchSnapshot.projects[].authority_files` 可作为第一批知识库资料来源，不等于完整知识库索引。
- 当前 `MemorySourceRef` / TS 类型已经允许 `knowledge_doc` 作为来源类型。
- 当前 `MemoryCandidateStore` 已能承载候选状态和受控采纳回链。
- 当前 `TaskPackage` 已有 `available_knowledge_refs` 字段，可用于任务包知识引用摘要。
- 当前 M7 记忆中心是只读入口，不能在 M8 中改造成生命周期后台。

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

- `evidence/2026-06-04-memory-layer-m6-workflow-task-package-injection-and-end-to-end-loop-v1.md`
- `handoffs/2026-06-04-memory-layer-m6-workflow-task-package-injection-and-end-to-end-loop-v1-result.md`
- `evidence/2026-06-05-memory-layer-m7-memory-management-ui-minimal-entry-v1.md`
- `handoffs/2026-06-05-memory-layer-m7-memory-management-ui-minimal-entry-v1-result.md`

当前实现重点：

- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/formal_memory_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_candidate_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/observation_store.rs`
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
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `prototypes/productized-desktop-shell/src/components/PermissionDialog.tsx`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

## 5. 范围

允许：

- 新增前端 read model helper，例如 `knowledgeBase.ts`，从 `WorkbenchSnapshot.projects[].authority_files`、formal memory source refs、memory candidates 和 task package knowledge refs 派生知识库读模型。
- 新增前端 UI，例如 `KnowledgeBaseView.tsx`，替换现有 `knowledge` placeholder。
- 新增或扩展类型：
  - `KnowledgeDocumentReadModel`
  - `KnowledgeSourceAnchor`
  - `KnowledgeMemoryLink`
  - `KnowledgeCandidateDraft`
  - `KnowledgeTaskReferenceSummary`
  - `ObsidianCompatibleBoundarySummary`
- 复用现有 `create_memory_candidate` 或新增受控 wrapper，例如 `create_memory_candidate_from_knowledge_doc`，但只能写 `memory-candidates.v1.json` 的候选记录。
- 允许从明确选择的知识库资料生成候选，必须包含 project / workflow / source kind / source label / source excerpt 或 source anchor / actor / reason。
- 允许正式记忆 `source_refs` 显示 `knowledge_doc` 反向链接。
- 允许知识库详情显示：
  - 文档 / 资料标题。
  - 项目归属。
  - 来源类型。
  - 来源锚点或路径摘要。
  - 关联正式记忆数量。
  - 关联候选数量。
  - 任务包引用数量。
  - “提出记忆候选”显式动作。
- 允许项目页文档 / 记忆相关轻量摘要复用知识库读模型，但不新增项目页 tab。
- 新增离线 UI 测试，覆盖知识库和记忆边界、候选生成文案、禁止文案和 Obsidian-compatible 占位。
- 新增 Rust 单测，覆盖候选来源为 `knowledge_doc` 时仍只能进入候选流程、不能直接写正式记忆；如果只做前端读模型，则说明未新增 Rust 测试的理由。
- 更新 evidence / handoff / `CURRENT.md` / `tasks/README.md` / `AUTHORITY.md` / `STAGE_PLAN.md` / 阶段计划。

本任务显式授权的数据变更：

- 允许读取 `workflow-state.v0.json` 里的项目、authority files、task package 和 existing artifacts。
- 允许读取 `formal-memories.v1.json`、`memory-candidates.v1.json`、`observations.v1.json`、`memory-lint.v1.json`，用于显示引用关系。
- 允许通过受控命令写 `memory-candidates.v1.json`，仅用于从明确知识库资料提出候选。
- 不允许写 `formal-memories.v1.json`。
- 不允许写 `observations.v1.json`，除非执行者另拆任务证明知识库材料必须先成为 observation；默认不这样做。
- 不允许写 `memory-lint.v1.json`。
- 不允许写 `workflow-state.v0.json`。
- 不允许新增 `workflow-state.v0.json` 顶层数组。
- 不允许迁移数据库。
- 不允许扫描或改写 Obsidian vault。

禁止：

- 不执行真实 worker。
- 不执行真实 Codex。
- 不执行 `codex exec` / `codex exec resume`。
- 不读写 `/Users/yoyi/.codex`。
- 不执行 Obsidian CLI。
- 不自动扫描 vault。
- 不自动把 Markdown / Canvas / Graph / Bases / vault 文档转成正式记忆。
- 不把知识命中、LLM summary、向量命中或文档摘要显示成正式记忆。
- 不让知识库入口直接编辑正式记忆。
- 不新增正式记忆编辑、废弃、冻结、归档、合并、拆分、上升全局、下沉项目等生命周期按钮。
- 不新增向量库或图数据库。
- 不做完整 Obsidian 插件兼容、双链、标签、图谱、文档版本和引用漂移检测。
- 不把 M8 说成中间版本记忆系统完成。

如果执行者认为必须读取真实 vault 文件内容或运行 Obsidian CLI，必须先停止并回传，说明：

- vault 路径。
- 会读取哪些文件。
- 是否会写入或修改文件。
- 是否会产生候选、正式记忆或任务包引用。
- 备份和回滚方案。

## 5.1 UI 显示边界确认

本任务会改前端：

- [ ] 不改前端、不改读模型、不改 UI 文案。
- [x] 改前端类型 / Tauri wrapper，但不新增正式记忆写命令。
- [x] 改读模型摘要或状态显示。
- [x] 改已有页面局部 UI。
- [x] 新增页面内面板、详情或显式候选生成动作，但不新增一级入口 / 右侧顶级入口 / 项目页 tab。

已读取：

- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`
- `CURRENT.md`
- `tasks/README.md`
- `docs/memory-layer-design-v1.md`
- `docs/plans/memory-layer-implementation-slice-v1.md`

本任务允许显示：

- 知识库资料 / 项目资料 / authority file 列表。
- 文档详情、项目归属、来源类型、来源锚点和路径摘要。
- 关联正式记忆、关联候选、任务包知识引用的数量和人话摘要。
- “提出记忆候选”动作，文案必须说明“只生成候选，不写正式记忆”。
- Obsidian-compatible 边界摘要，例如“可兼容 markdown / vault 风格来源，但未执行 Obsidian 原生同步”。
- 知识库和记忆的区别说明：知识库是材料和笔记空间；正式记忆是经过确认、来源、版本、审计和权限治理的行为上下文。

本任务禁止显示：

- 不新增一级入口；复用现有 `知识库`。
- 不新增右侧顶级入口。
- 不新增项目页 tab。
- 不把知识库做成第二个记忆中心。
- 不显示“已接入 Obsidian 原生同步”“vault 已自动扫描”“知识库已自动记住”“文档已成为正式记忆”“知识命中已注入任务包”等误导文案。
- 不显示 raw schema、raw event、完整 sidecar JSON、数据库路径大表或完整审计日志。
- 不显示未实现的 Obsidian 插件、双链、标签、图谱、Canvas、Bases、向量库或图数据库能力。
- 不显示正式记忆生命周期按钮。

显示位置：

- 一级入口：复用现有 `知识库`，不新增。
- 右侧入口：不改。
- 项目页：只允许项目相关知识库 / 文档轻量摘要，不新增 tab，不占用工作流画布主区域。
- 画布：不改画布主区域；任务包详情可继续显示知识库引用摘要。
- 记忆入口：只允许显示 `knowledge_doc` 来源反向链接，不把知识库入口并入记忆中心。
- 知识库入口：本轮主要落地位置，显示资料列表、详情、来源锚点、关联记忆 / 候选和候选生成入口。
- 智能体入口：不改。
- 管理入口：不改。

中间版本范围：

- 本轮必须落地：知识库最小入口、知识库 / 记忆边界、`knowledge_doc` 来源引用、候选生成入口、反向引用摘要。
- 本轮只做读模型 / 摘要：Obsidian-compatible 边界、项目相关知识库摘要、任务包知识引用摘要。
- 本轮后置：Obsidian 原生同步、双链、标签、图谱、插件兼容、文档版本、引用漂移检测、向量库、图数据库。

后端和数据依赖：

- 需要后端正式读模型：可以先复用 `WorkbenchSnapshot.projects[].authority_files`；如新增 Tauri 只读命令，必须是 deterministic read model。
- 需要审计 / 日志 / 权限 / 状态机：候选生成必须走 `MemoryCandidateStore` 和现有控制边界；正式记忆仍必须走 M2 采纳链路。
- 不能用假数据伪装：不能伪造 Obsidian 连接状态、vault 扫描结果、正式记忆引用、候选生成、任务包引用或文档内容。

UI 文案边界：

- 禁止说：`已接入 Obsidian 原生同步`、`vault 已自动扫描`、`知识库已自动记住`、`文档已成为正式记忆`、`知识命中已成为正式记忆`、`知识命中已注入任务包`、`中间版本记忆层已完成`。
- 允许说：`知识库资料`、`知识库来源`、`Obsidian-compatible 占位`、`提出记忆候选`、`只生成候选，不写正式记忆`、`正式记忆引用了该知识库来源`、`未执行 Obsidian 原生同步`。

验收：

- 类型检查：`npm run typecheck`
- 离线交互测试：`npm run test:offline-interaction`
- 构建：`npm run build`
- 真实窗口 / 截图验收：涉及知识库入口布局，必须做真实浏览器或 Tauri 截图验收；如果没有可用截图工具，不能声称 UI 验收完成。
- 未验收项必须写入 evidence / handoff。

## 6. 实施建议

建议按以下顺序实现：

1. 先写知识库读模型：从 `snapshot.projects[].authority_files` 派生 `KnowledgeDocumentReadModel`，并关联 formal memory `source_refs`、memory candidates 和 task package `available_knowledge_refs`。
2. 替换 `App.tsx` 里的 `knowledge` placeholder，新增 `KnowledgeBaseView` 或等价组件。
3. 加候选生成入口：只允许从明确文档 / source anchor 生成 `MemoryCandidate`，文案写明不是正式记忆。
4. 在 M7 记忆中心或读模型里补 `knowledge_doc` 来源反向链接摘要，但不要把知识库 UI 合并进记忆中心。
5. 补离线 UI 测试和必要 Rust 测试。
6. 写 evidence / handoff 并同步入口。

如果实现过程中发现缺少稳定文档 id，不要引入复杂索引系统；先用 project id + authority file path / name 派生 deterministic key，并把完整文档索引列为后置。

## 7. 验收

必须通过：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
cargo test --lib memory_candidate
cargo test --lib formal_memory
cargo test --lib task_memory_packet
cargo test --lib
rustfmt --check src/memory_candidate_store.rs src/formal_memory_store.rs src/task_memory_packet_builder.rs src/task_memory_injection.rs src/control_core.rs src/commands.rs src/types.rs src/lib.rs
```

如果新增 Rust `knowledge_*` 模块或 Tauri command，还必须补：

```text
cargo test --lib knowledge
rustfmt --check src/knowledge_base_read_model.rs
```

UI / 文案必须验证：

- `知识库` 入口展示资料和边界，不显示成记忆中心。
- 知识库材料生成候选时，明确“只生成候选，不写正式记忆”。
- `knowledge_doc` 来源能在记忆详情或知识库详情里反向展示。
- 未出现禁止文案：
  - `已接入 Obsidian 原生同步`
  - `vault 已自动扫描`
  - `知识库已自动记住`
  - `文档已成为正式记忆`
  - `知识命中已成为正式记忆`
  - `知识命中已注入任务包`
  - `中间版本记忆层已完成`

## 8. evidence / handoff 要求

M8 完成后必须新增：

- `evidence/2026-06-05-memory-layer-m8-knowledge-base-and-obsidian-compatible-interface-placeholder-v1.md`
- `handoffs/2026-06-05-memory-layer-m8-knowledge-base-and-obsidian-compatible-interface-placeholder-v1-result.md`

evidence 必须记录：

- 实际新增 / 修改的读模型、命令、类型、UI 文件。
- 知识库和记忆边界的 UI 验收结果。
- 知识库材料生成候选的审计 / sidecar 结果；如果未实现写候选，必须说明原因。
- `knowledge_doc` 来源和正式记忆 / 候选的反向链接验收结果。
- 禁止文案搜索结果。
- 是否做了真实浏览器或 Tauri 截图验收；如果没有，必须明确写“真实窗口 / 截图验收未完成”。
- 验证命令和结果。
- 边界：未执行 Obsidian CLI、未自动扫描 vault、未写正式记忆、未执行真实 worker / Codex、未读写 `/Users/yoyi/.codex`。

handoff 必须写清：

- M8 接受为什么。
- M8 不接受为什么。
- 下一步应进入 M9 还是先补截图 / UI 缺口。
- 知识库与记忆边界、候选生成入口、Obsidian-compatible 占位文案是否成为当前权威。

## 9. Stop 条件

遇到以下情况必须停下回传：

- 需要读写 `/Users/yoyi/.codex`。
- 需要执行真实 `codex exec` 或 `codex exec resume`。
- 需要启动真实 worker。
- 需要执行 Obsidian CLI。
- 需要自动扫描 vault。
- 需要改写 Obsidian vault、Markdown 文件或项目外文档。
- 需要把知识库材料、知识命中、LLM summary、Canvas / Graph / Bases 结果直接写成正式记忆。
- 需要新增正式记忆生命周期操作。
- 需要接向量库或图数据库。
- 需要新增一级入口、右侧顶级入口或项目页 tab。
- 发现本任务与 `docs/workbench-frontend-display-boundary-v1.md`、`docs/memory-layer-design-v1.md` 或 `docs/plans/memory-layer-implementation-slice-v1.md` 冲突。

## 10. 回收口径

完成后接受为：

- M8 知识库 / Obsidian-compatible 接口占位和边界完成。
- `知识库` 一级入口具备最小可理解资料展示、来源锚点、关联记忆 / 候选和候选生成入口。
- `knowledge_doc` 可作为正式记忆 / 候选来源被展示和反向链接。

完成后不接受为：

- Obsidian 原生同步完成。
- vault 自动扫描完成。
- 知识库正式索引系统完成。
- 文档版本和引用漂移检测完成。
- 正式记忆生命周期操作完成。
- M9 / M10 / M11 / M12 / M13 完成。
- 中间版本完整记忆系统完成。
- 真实 worker / Codex 已执行。
