# Task Package：Memory Layer M13 Final Authoritative Acceptance And Conclusion Freeze v1

状态：已完成，最终结论 `accepted_with_deferred_items`。  
用途：执行中间版本记忆系统最终权威验收和最终结论冻结。  
执行方式：验收优先任务；默认不新增记忆能力，不重构，不扩大功能范围。M13 的职责是统一复核 M1-M12.1 的证据、测试、边界和 UI 可理解性，产出最终验收结论；如发现必须修补的缺陷，应拆 follow-up 任务包，不能在 M13 中偷偷开发新功能。

写包边界：本任务包最初只产出 / 修订任务包本体；执行阶段已完成 M13，只做验收和结论冻结，未改产品代码。

预期回收记录：

- `evidence/2026-06-05-memory-layer-m13-final-authoritative-acceptance-and-conclusion-freeze-v1.md`
- `handoffs/2026-06-05-memory-layer-m13-final-authoritative-acceptance-and-conclusion-freeze-v1-result.md`

## 0. 角色边界

写包者职责：

- 只定义 M13 的验收目标、范围、Stop 条件、gate 清单、验证命令、UI 边界和回收口径。
- 不替执行者判断最终结论。
- 不把任务包写成新功能实现需求。

M13 执行者职责：

- 以只读复核、验证命令、文档一致性检查、UI 边界检查和最终结论冻结为主。
- 必须产出 M13 evidence / handoff，并在其中写明 `accepted` / `accepted_with_deferred_items` / `blocked`。
- 如发现阻断缺陷，停止扩大实现，拆后续 `M13.x` 修补任务包。

全局主管回收职责：

- M13 handoff 返回后，必须进行只读复核，而不是默认接受。
- 回收时重点检查最终结论是否有证据、deferred items 是否真的非阻断、是否误宣称最终蓝图 / 真实 worker / GraphRAG / 自动技能化完成。
- 如发现验收结论证据不足，结论应降级为 `blocked` 或要求补证据；不得用口头判断覆盖 evidence。

## 1. 先说薄弱点

- M1-M6 已证明第一条真实记忆闭环，但它不是完整记忆系统。
- M7-M12 已补齐记忆中心、知识库边界、生命周期、实体关系、维护任务、成熟模式和跨项目主题报告，但这些能力分别落在不同批次，最终权威验收必须统一检查。
- M12.1 已修补 mature pattern 正式化后 `acceptance_summary` 新鲜度，但这只解决一个窄问题，不等于 M13 完成。
- 多个 UI 批次记录过真实窗口 / 截图验收缺口，尤其 M10-M12 真实 Tauri 数据桥或截图验收未完成；M13 必须明确这些缺口是否阻断最终结论，还是作为阶段 G / 专门 UI 验收后置。
- M13 最容易越界：执行者可能把“最终验收”做成“继续补功能”。本任务禁止新增记忆能力，除非发现验收无法完成且必须拆出后续修补任务。

## 2. 任务目标

统一验证并冻结中间版本记忆系统结论：

```text
M1 formal memory store
M1.1 context binding
M2 candidate adoption
M3 observation
M4 task memory packet preview
M5 lint blocking
M6 workflow task package injection
M7 memory center UI
M8 knowledge base boundary
M9 lifecycle
M10 entity / relation governance
M11 maintenance
M12 mature pattern / cross-project theme
M12.1 acceptance summary freshness
-> final authoritative acceptance summary
-> accepted / accepted_with_deferred_items / blocked conclusion
-> evidence + handoff
```

M13 完成后可以说：

- 中间版本记忆系统已完成最终权威验收，或明确未通过并列出阻断项。
- 第一条真实闭环、正式记忆系统完成度、中间版本记忆系统完成度、最终蓝图后置能力已被清楚区分。
- M1-M12.1 的关键证据、测试、UI 边界、Stop 条件、缺口和后置项已统一复核。
- observation、candidate、formal memory、version、audit、source、permission、lint、relation、maintenance、mature pattern、cluster report、knowledge doc 和 task packet recall 的边界已形成最终验收结论。

M13 完成后仍不能说：

- 最终蓝图完整能力全部完成。
- 真实 worker / Codex 自动化全链路已执行。
- GraphRAG、向量库、图数据库、自动索引重建、完整理解地图、Obsidian 原生同步或自动技能化完成。
- 阶段 G 的真实 Tauri 全面验收、运行日志、自动重试、运维诊断完成。
- 任何候选、observation、knowledge hit、maintenance report、cluster report 或 LLM summary 可以绕过正式记忆状态机。

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
- `tasks/2026-06-05-memory-layer-m12-mature-pattern-cross-project-memory-and-complete-acceptance-v1.md`
- `tasks/2026-06-05-memory-layer-m12-1-acceptance-summary-freshness-after-mature-pattern-formalization-v1.md`

必须先读：

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
- M1-M12.1 全部 evidence / handoff。

## 4. 范围

允许：

- 做只读架构复核、测试复核、文档一致性复核和 UI 边界复核。
- 新增最终验收 evidence / handoff。
- 如已有后端读模型支持，可以在 evidence / handoff 中整理最终验收摘要；必须优先复用既有机制，不新增产品 artifact / store。
- 如发现 M13 验收需要只读 helper、测试 fixture 或任何代码改动，必须停下并拆 `M13.x` follow-up；不得在 M13 中直接实现。
- 运行完整验证命令。
- 做真实窗口 / 浏览器 / Tauri smoke 或截图验收；如果工具需要读取 `/Users/yoyi/.codex` 下插件说明，必须先判断是否与本任务 stop 条件冲突，必要时停下回传。
- 更新 `CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md`、阶段计划和 M13 任务包状态。

本任务默认不允许新增：

- 新 sidecar。
- 新 Tauri command。
- 新前端入口。
- 新记忆生命周期能力。
- 新成熟模式能力。
- 新 relation / entity 推断能力。
- 新知识库 / Obsidian 集成能力。
- 新 worker / Codex 执行能力。

如果执行者认为必须新增功能才能通过最终验收，必须停下并拆 follow-up 任务包，不得在 M13 中直接实现。

## 5. UI 显示边界确认

本任务是否改前端：

- [x] 默认不改前端、不改读模型、不改 UI 文案。
- [ ] 只有在已有验收摘要入口必须最小展示最终结论时，才允许改读模型摘要或既有 `记忆` / `管理` 页面局部 UI。
- [ ] 不新增一级入口、右侧顶级入口、项目页 tab 或画布区域。

已读取：

- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`
- `CURRENT.md`
- `tasks/README.md`

本任务允许显示：

- 最终验收结论：`accepted`、`accepted_with_deferred_items`、`blocked`。
- M1-M12.1 gate 摘要。
- 真实窗口 / 截图验收状态。
- 后置能力清单。
- 阻断项 / 非阻断缺口。

本任务禁止显示：

- 不新增一级入口。
- 不新增右侧顶级入口。
- 不新增项目页 tab。
- 不把 raw sidecar、raw JSON、raw audit、完整日志或数据库路径大表铺进普通 UI。
- 不显示“最终蓝图完整完成”“真实 worker 已执行”“GraphRAG 已完成”“自动技能化完成”等误导文案。
- 不把候选、observation、cluster report、maintenance report 或 knowledge hit 显示为正式事实。

显示位置：

- 一级入口：不新增。
- 右侧入口：不新增；如需要最终验收摘要，优先进入既有管理内部或 evidence / handoff 文档。
- 项目页：不新增 tab，不占用画布主区域。
- 画布：不改画布主区域。
- 记忆入口：如必须展示，只能是最终验收轻量摘要。
- 管理入口：可显示最终验收健康摘要，不显示 raw store / raw audit。

中间版本范围：

- 本轮必须落地：最终权威验收结论和 evidence / handoff。
- 本轮只做读模型 / 摘要：如已有入口需要展示最终状态，只能做轻量摘要。
- 本轮后置：阶段 G 真实 Tauri 全面验收、运行日志、自动重试、运维诊断、GraphRAG、向量库、图数据库、完整理解地图、自动技能化。

后端和数据依赖：

- 需要后端正式读模型：优先使用已存在 store / summary / tests，不新增 store。
- 需要审计 / 日志 / 权限 / 状态机：最终验收必须以已有 evidence / handoff / tests / store 语义为依据，不能用假数据伪装。
- 不能用假数据伪装：不能伪造真实 worker、真实 Codex、截图、Tauri 数据桥或已完成后置能力。

UI 文案边界：

- 禁止说：`最终蓝图完整完成`、`真实 worker 已执行`、`真实 Codex 已执行`、`GraphRAG 已完成`、`自动技能化完成`、`候选已成为正式事实`、`聚类报告就是事实`。
- 允许说：`中间版本记忆系统最终验收`、`第一条闭环完成`、`正式记忆系统完成`、`中间版本记忆系统完成`、`后置能力未完成`、`accepted_with_deferred_items`。

验收：

- 类型检查：如改前端必须 `npm run typecheck`。
- 离线交互测试：如改前端必须 `npm run test:offline-interaction`。
- 构建：如改前端必须 `npm run build`。
- 真实窗口 / 截图验收：M13 应尽量做；如因工具或权限不可用，必须明确写入 evidence / handoff，并判断是否阻断最终验收。

## 6. 必须复核的 Gate

M13 至少要逐项复核并给出 `passed` / `deferred` / `blocked`：

- `observation_gate`：observation 只能从明确工作流事件生成；普通聊天不自动进入长期记忆。
- `candidate_gate`：candidate 不能进入任务包 included list，不能冒充正式记忆。
- `formal_memory_gate`：正式记忆必须有 source refs、version、audit、status、scope、permission。
- `context_binding_gate`：正式记忆和候选采纳必须绑定正确 project / workflow。
- `adoption_gate`：低风险项目记忆可由项目主管采纳；用户偏好 / 全局 / 跨项目 / mature pattern 必须用户确认。
- `task_packet_gate`：任务包只召回 active formal memory，并能解释 included / excluded / review materials。
- `lint_gate`：blocking lint / conflict / revoked permission 能阻止采纳或召回。
- `workflow_injection_gate`：工作流任务包注入使用冻结正式记忆快照，不执行真实 worker。
- `memory_center_gate`：记忆中心能区分正式记忆、候选、observation、lint、relation、maintenance、mature pattern。
- `knowledge_boundary_gate`：知识库材料只作为来源 / 候选，不直接写正式记忆。
- `lifecycle_gate`：正式记忆 revise / deprecate / freeze / archive / merge / split / promote / demote 都有版本、审计和确认权。
- `entity_relation_gate`：LLM inferred / similarity relation 默认候选；已确认关系只解释召回，不改变 included list。
- `maintenance_gate`：维护任务只生成 finding / report / recommendation，不自动修改正式记忆。
- `mature_pattern_gate`：mature pattern candidate / cluster report 未确认不入包；用户确认后才可写正式 mature pattern memory。
- `m12_1_freshness_gate`：用户确认 mature pattern 正式化后，当次 acceptance summary 使用 fresh formal store。
- `ui_gate`：UI 显示边界、禁止文案、真实窗口 / 截图验收状态清楚。
- `security_boundary_gate`：未执行真实 worker / Codex、未读写 `/Users/yoyi/.codex`、未扫描完整 transcript、未接 GraphRAG / 向量库 / 图数据库。

## 7. 验收命令

必须运行或明确说明无法运行原因：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
cargo test --lib formal_memory
cargo test --lib memory_candidate
cargo test --lib observation
cargo test --lib task_memory_packet
cargo test --lib task_memory_injection
cargo test --lib memory_lint
cargo test --lib memory_entity_relation
cargo test --lib mature_pattern
cargo test --lib memory_cluster
cargo test --lib
rustfmt --check src/formal_memory_store.rs src/formal_memory_lifecycle.rs src/memory_candidate_store.rs src/observation_store.rs src/task_memory_packet_builder.rs src/task_memory_injection.rs src/memory_lint_store.rs src/memory_lint_engine.rs src/memory_entity_relation_store.rs src/memory_entity_relation_governance.rs src/mature_pattern_store.rs src/mature_pattern_governance.rs src/control_core.rs src/commands.rs src/types.rs src/lib.rs
```

如果执行者修改了其他 Rust 文件，必须纳入 `rustfmt --check`。

如果 M13 不改前端，也仍建议运行前端三项，因为最终验收要覆盖 UI 可理解性和离线交互边界。

## 8. 真实窗口 / 截图验收规则

M13 应优先尝试真实窗口 / 浏览器 / Tauri 验收，但必须遵守权限边界：

- 不得为截图验收擅自读写 `/Users/yoyi/.codex`。
- 如果 Browser / screenshot 工具要求读取 `.codex` 插件缓存说明，执行者必须判断是否已有明确授权；没有授权时应停下回传或记录无法执行。
- 普通浏览器 smoke 不能等同真实 Tauri 数据桥验收。
- 如果只能做 HTTP smoke / 普通浏览器 smoke，必须在 evidence 里写清不接受为真实 Tauri 验收。
- 如果真实窗口 / 截图验收无法完成，M13 必须判断它是阻断最终验收，还是作为阶段 G / 专门 UI 验收后置项。

## 9. evidence / handoff 要求

M13 完成后必须新增：

- `evidence/2026-06-05-memory-layer-m13-final-authoritative-acceptance-and-conclusion-freeze-v1.md`
- `handoffs/2026-06-05-memory-layer-m13-final-authoritative-acceptance-and-conclusion-freeze-v1-result.md`

evidence 必须记录：

- M1-M12.1 每个 gate 的依据、测试、结论和缺口。
- 哪些结论接受为第一条闭环完成。
- 哪些结论接受为正式记忆系统完成。
- 是否接受为中间版本记忆系统完成。
- 哪些能力仍明确后置到最终蓝图或阶段 G。
- M12 真实窗口 / 截图验收缺口如何处理。
- M12.1 fresh formal store 修补是否已纳入最终结论。
- 禁止文案和 UI 边界复核结果。
- 验证命令和结果。
- 边界：未执行真实 worker / Codex、未读写 `/Users/yoyi/.codex`、未接 GraphRAG / 向量库 / 图数据库、未自动技能化。

handoff 必须写清：

- M13 接受为什么。
- 如果不接受，阻断项是什么，下一步怎么拆。
- `accepted` / `accepted_with_deferred_items` / `blocked` 的最终结论。
- 后续阶段建议：阶段 G、会话 / adapter、多 agent、模型凭据、工作流真实执行、运维日志等如何衔接。
- 当前权威入口文件。

## 10. Stop 条件

遇到以下情况必须停下回传：

- 需要新增记忆功能而不是验收。
- 需要改正式记忆 schema、workflow state 顶层结构或数据库迁移。
- 需要读写 `/Users/yoyi/.codex`。
- 需要执行真实 `codex exec` 或 `codex exec resume`。
- 需要启动真实 worker。
- 需要扫描完整 transcript。
- 需要接 Obsidian 原生写入、GraphRAG、向量库、图数据库或自动索引重建。
- 需要把候选、observation、knowledge hit、maintenance report、cluster report、relation candidate、LLM summary 或 graph/index report 直接当正式事实。
- 需要让秘书、worker 或 UI 直接写正式记忆。
- 需要新增一级入口、右侧顶级入口或项目页 tab。

## 11. 回收口径

完成后接受为：

- 中间版本记忆系统最终权威验收完成，前提是所有阻断 gate 通过或被明确转为非阻断后置项。
- M1-M12.1 记忆层能力形成最终结论。
- 第一条闭环、正式记忆系统、中间版本记忆系统、最终蓝图后置能力被清楚区分。

完成后不接受为：

- 最终蓝图完整工作台完成。
- 阶段 G 真实 Tauri 全面验收完成。
- 真实 worker / Codex 已执行。
- GraphRAG、向量库、图数据库、自动索引重建、完整理解地图或自动技能化完成。
- 自动化工作流真实执行和运维日志体系完成。

最终结论格式必须使用以下之一：

- `accepted`
- `accepted_with_deferred_items`
- `blocked`

如果使用 `accepted_with_deferred_items`，必须列出 deferred items，并说明为什么不阻断中间版本记忆系统最终验收。
