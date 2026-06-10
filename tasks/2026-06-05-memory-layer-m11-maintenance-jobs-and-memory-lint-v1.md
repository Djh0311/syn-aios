# Task Package：Memory Layer M11 Maintenance Jobs And Memory Lint v1

状态：已完成。  
用途：实现中间版本记忆层 M11：维护任务和记忆 lint。  
执行方式：一个较大但必须保守的批次完成；开发重点是在 M5 最小 lint 阻断和 M10 实体 / 关系治理之上，补齐维护运行、维护报告、过期 / 缺来源 / 重复 / 实体漂移 / 权限撤回 / 私密扫描 / 索引状态检查，以及任务包阻断和记忆中心可理解摘要。

完成记录：

- `evidence/2026-06-05-memory-layer-m11-maintenance-jobs-and-memory-lint-v1.md`
- `handoffs/2026-06-05-memory-layer-m11-maintenance-jobs-and-memory-lint-v1-result.md`

实际回收摘要：

- 后端复用并扩展 `memory-lint.v1.json`，新增 maintenance run / maintenance report / check summary / recommendation / index status，不新增维护 sidecar。
- 维护任务覆盖过期 / stale、缺来源、重复、权限撤回、关系来源撤回、私密 / 外发风险、实体漂移、派生索引状态和 mature pattern signal。
- 任务记忆包继续以 open blocking finding 作为召回闸门，不让 needs_review / info 自动排除，也不让维护报告成为正式事实。
- 记忆中心复用既有 `记忆` 入口展示维护摘要和运行入口；确认弹层明确只写 lint / maintenance sidecar，不自动修改正式记忆。
- M11 不自动调用 M9 lifecycle，不写正式记忆 / 候选 / observation / 实体关系 / workflow state，不执行真实 worker 或 Codex。

## 1. 先说薄弱点

- M5 已完成 `memory-lint.v1.json`、deterministic finding、采纳前 blocking guard 和任务包 blocking 排除，但它仍是最小阻断，不是完整维护任务系统。
- M10 已完成 entity / relation sidecar、关系候选和任务包关系解释，但 relation / entity 只能解释召回，不是维护任务的事实源或自动修复器。
- 当前系统仍缺维护运行报告：哪些记忆过期、缺来源、权限撤回、重复、实体漂移、私密风险、索引失败或需要成熟模式候选，只能散落在 finding 或人工判断里。
- 维护任务最容易越界：看起来像“清理记忆库”，但 M11 不能自动废弃、冻结、归档、合并、拆分、提升为全局或改写正式记忆。
- M11 会影响记忆中心、任务包 recall、lint 摘要和可能的管理健康摘要，必须落实 UI 显示边界固定章节。

## 2. 任务目标

建立维护任务的最小闭环：

```text
FormalMemoryStore / MemoryCandidateStore / ObservationStore / MemoryLintStore / EntityRelationStore
-> maintenance run
-> deterministic maintenance findings
-> maintenance report summary
-> blocking / needs_review / info classification
-> task packet recall guard
-> MemoryCenter human-readable maintenance summary
```

M11 完成后可以说：

- 系统有维护任务运行入口和维护运行记录。
- 维护任务能检查过期、缺来源、重复 / 实体漂移、权限撤回、私密和安全风险、索引状态、候选成熟度信号。
- blocking finding 能继续阻止任务包召回相关正式记忆。
- needs_review / info finding 能进入记忆中心或管理摘要，但不自动改正式记忆。
- 维护报告能解释“需要处理什么”和“为什么不能自动处理”。
- 维护任务能与 M10 entity / relation sidecar 协作发现实体漂移或关系来源问题，但不能把关系图当正式事实。

M11 完成后仍不能说：

- M12 成熟模式、跨项目记忆或完整验收完成。
- M13 中间版本记忆系统最终验收完成。
- 维护任务能自动修复、合并、废弃、冻结、归档或删除正式记忆。
- 成熟模式候选能自动成为正式全局记忆。
- 向量库、图数据库、GraphRAG、自动索引重建系统或完整运维后台完成。
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

开始前必须复核：

- M5 `memory-lint.v1.json` 已有 `findings[]`、`runs[]`、revision、lock、备份、原子写和损坏 JSON 保护。
- M5 `MaintenancePreview` intent 已存在，但只覆盖最小 duplicate / authority / revoked / candidate conflict 场景。
- M10 relation explanation 不是任务包选择器；不能让 relation candidate 直接改变 worker 输入。
- M9 lifecycle 是唯一可改正式记忆生命周期的受控路径；M11 只能建议，不自动调用 lifecycle。
- M7 记忆中心是用户可理解入口，不是 raw maintenance console。

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

- `evidence/2026-06-04-memory-layer-m5-conflict-and-memory-lint-minimal-blocking-v1.md`
- `handoffs/2026-06-04-memory-layer-m5-conflict-and-memory-lint-minimal-blocking-v1-result.md`
- `evidence/2026-06-05-memory-layer-m10-entity-and-relation-governance-v1.md`
- `handoffs/2026-06-05-memory-layer-m10-entity-and-relation-governance-v1-result.md`

当前实现重点：

- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_lint_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_lint_engine.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_entity_relation_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_entity_relation_governance.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/formal_memory_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/formal_memory_lifecycle.rs`
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
- `prototypes/productized-desktop-shell/src/components/PermissionDialog.tsx`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

## 5. 范围

允许：

- 复用并扩展 `memory-lint.v1.json`，优先保持同一个 lint / maintenance sidecar；如必须新增 sidecar，必须说明为什么不能复用 M5 store。
- 新增或扩展后端类型，例如：
  - `MemoryMaintenanceRunInput`
  - `MemoryMaintenanceRunOutput`
  - `MemoryMaintenanceReport`
  - `MemoryMaintenanceCheck`
  - `MemoryMaintenanceCheckKind`
  - `MemoryMaintenanceIndexStatus`
  - `MemoryMaintenanceRecommendation`
  - `MemoryMaintenanceFindingImpact`
- 或等价扩展现有 `MemoryLintRunInput` / `MemoryLintRunOutput` / `MemoryLintFinding` / `MemoryLintStoreSummary`，但必须兼容旧 sidecar。
- 支持维护检查：
  - expired memory / stale memory
  - missing source / weak source
  - duplicate claim
  - entity drift / alias drift
  - relation source revoked
  - source permission revoked
  - sensitive export risk
  - private / secret source scan
  - index status / derived index stale
  - mature pattern candidate signal
- 支持维护运行入口，例如：
  - `run_memory_maintenance`
  - `preview_memory_maintenance`
  - 或扩展 `run_memory_lint` 的 `MaintenancePreview` / `MaintenanceRun` intent。
- 维护任务每次运行必须写 run record、report summary 或 audit ref。
- blocking finding 继续阻止任务包召回相关正式记忆。
- needs_review / info finding 可进入 review materials 或记忆中心维护摘要。
- 权限撤回后，相关正式记忆或派生索引必须停止未来召回或进入 blocking / needs_review。
- 索引状态失败只能生成 finding / report，不能导致正式记忆丢失。
- 从 M10 entity / relation sidecar 派生实体漂移、alias drift 或 relation source revoked finding。
- 维护任务可以建议 M9 lifecycle 操作，但不能自动执行；建议必须进入用户 / 项目主管确认链路。
- 新增 Rust 单测和前端离线测试。
- 更新 evidence / handoff / `CURRENT.md` / `tasks/README.md` / `AUTHORITY.md` / `STAGE_PLAN.md` / 阶段计划。

本任务显式授权的数据变更：

- 允许写 `memory-lint.v1.json` 的 maintenance run / findings / report summary / warnings / revision。
- 允许读取 `formal-memories.v1.json`、`memory-candidates.v1.json`、`observations.v1.json`、`memory-entity-relations.v1.json`、workflow state 和 task package artifacts。
- 允许任务记忆包 builder 读取 M11 blocking finding，用于阻断召回或 stale 判断。
- 不允许写 `formal-memories.v1.json`。
- 不允许写 `memory-candidates.v1.json`。
- 不允许写 `observations.v1.json`。
- 不允许写 `memory-entity-relations.v1.json`，除非只是读取 M10 结果；M11 默认不改实体 / 关系决定。
- 不允许写 `workflow-state.v0.json`。
- 不允许新增 `workflow-state.v0.json` 顶层数组。
- 不允许迁移数据库。

禁止：

- 不执行真实 worker。
- 不执行真实 Codex。
- 不执行 `codex exec` / `codex exec resume`。
- 不读写 `/Users/yoyi/.codex`。
- 不自动修改正式记忆状态。
- 不自动新增正式记忆版本。
- 不自动废弃、冻结、解冻、归档、合并、拆分、上升或下沉 scope。
- 不自动合并实体或关系。
- 不让 mature pattern signal 自动成为正式记忆、全局记忆、技能或规则。
- 不让 relation candidate、LLM summary、knowledge hit、observation 或 graph/index report 绕过正式记忆状态机。
- 不接向量库、图数据库、GraphRAG 或自动索引重建系统。
- 不扫描完整 transcript。
- 不把维护报告当权威事实源。
- 不把 M11 说成中间版本完整记忆系统完成。

如果执行者认为必须自动修改正式记忆、调用 M9 lifecycle、写 workflow state、接向量 / 图数据库或读取 `/Users/yoyi/.codex`，必须停下回传。

## 5.1 UI 显示边界确认

本任务会改前端：

- [ ] 不改前端、不改读模型、不改 UI 文案。
- [x] 改前端类型 / Tauri wrapper。
- [x] 改读模型摘要或状态显示。
- [x] 改已有页面局部 UI。
- [ ] 新增一级入口、右侧顶级入口、项目页 tab 或独立治理后台。

已读取：

- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`
- `CURRENT.md`
- `tasks/README.md`
- `docs/memory-layer-design-v1.md`
- `docs/plans/memory-layer-implementation-slice-v1.md`

本任务允许显示：

- 维护任务摘要。
- 最近一次维护 run 状态。
- open / blocking / needs_review / info finding 数量。
- 过期、缺来源、重复、实体漂移、权限撤回、私密风险、索引状态等人话摘要。
- 建议动作，例如“建议人工复核”“建议通过生命周期操作冻结 / 废弃”，但必须写明不会自动执行。
- 任务包被 blocking finding 排除的原因摘要。
- 明确边界文案：`维护任务只生成 finding`、`不会自动修改正式记忆`、`blocking finding 会阻止召回`。

本任务禁止显示：

- 不新增一级入口。
- 不新增右侧顶级入口。
- 不新增项目页 tab。
- 不显示完整 sidecar、raw JSON、raw audit、数据库路径大表或完整索引日志。
- 不显示“自动清理记忆”“自动修复记忆”“自动合并重复记忆”“自动废弃过期记忆”“成熟模式已自动成为规则”“索引重建已改变事实”等误导文案。
- 不把维护报告显示为正式事实。
- 不把 mature pattern signal 显示为正式记忆或全局记忆。

显示位置：

- 一级入口：不新增；主要复用 `记忆`。
- 右侧入口：不新增；如需健康摘要，只能进入既有 `管理` 内部，不新增右侧图标。
- 项目页：只允许轻量提示，不新增 tab，不占用工作流画布主区域。
- 画布：不改画布主区域。
- 记忆入口：显示维护摘要、finding 分类和建议动作摘要。
- 知识库入口：只允许显示知识来源相关维护风险摘要，不直接编辑正式记忆。
- 智能体入口：不改。
- 管理入口：可显示维护健康摘要，但不显示 raw store / raw audit。

中间版本范围：

- 本轮必须落地：维护 run、维护 findings、维护 report summary、任务包 blocking 协作、记忆中心摘要。
- 本轮只做读模型 / 摘要：索引状态、实体漂移、mature pattern signal 和建议动作。
- 本轮后置：自动索引重建、完整运维后台、成熟模式正式化、跨项目全局记忆、GraphRAG / 向量库。

后端和数据依赖：

- 需要后端正式读写模型：必须通过 Rust store wrapper 写 lint / maintenance sidecar。
- 需要审计 / 日志 / 权限 / 状态机：每次维护 run 必须有 run record / audit ref / revision。
- 不能用假数据伪装：不能伪造维护完成、索引健康、成熟模式确认、生命周期执行或任务包阻断结果。

UI 文案边界：

- 禁止说：`自动清理记忆`、`自动修复记忆`、`自动合并重复记忆`、`自动废弃过期记忆`、`维护任务已改正式记忆`、`成熟模式已自动成为规则`、`索引重建已改变事实`、`中间版本记忆层已完成`。
- 允许说：`维护任务`、`维护摘要`、`finding`、`blocking finding`、`needs_review finding`、`建议人工复核`、`建议生命周期操作`、`不会自动修改正式记忆`、`blocking finding 会阻止召回`。

验收：

- 类型检查：`npm run typecheck`
- 离线交互测试：`npm run test:offline-interaction`
- 构建：`npm run build`
- 真实窗口 / 截图验收：涉及记忆中心维护摘要和可能的管理摘要，必须做真实浏览器或 Tauri 截图验收；如果没有可用截图工具，不能声称 UI 验收完成。
- 未验收项必须写入 evidence / handoff。

## 6. 维护规则

最低规则：

- expired / stale：只生成 finding，建议人工复核或 M9 lifecycle，不自动改状态。
- missing source：缺来源正式记忆必须 needs_review；严重缺来源或安全边界相关可 blocking。
- duplicate / entity drift：结合 M5 duplicate 和 M10 entity / relation store 生成 finding；不能自动合并。
- permission revoked：默认 blocking，任务包不能召回相关正式记忆。
- sensitive export risk：外发策略和 sensitive level 冲突时 blocking 或 needs_review。
- index status：索引缺失、过期或重建失败只生成 finding / report；不能删除正式记忆。
- mature pattern signal：只生成候选信号或 finding；留给 M12。

确认权：

- 项目内低风险 maintenance finding 可由项目主管确认处理建议。
- 用户偏好、安全边界、跨项目、全局记忆、权限撤回和 mature pattern 相关建议必须用户确认。
- 秘书只能解释 finding、整理影响面、提醒确认事项；不能确认维护处理或 lifecycle 操作。

## 7. 实施建议

建议按以下顺序实现：

1. 复核并扩展 M5 `MemoryLintStoreV1` / `MemoryLintRunInput` / `MemoryLintRunOutput`，保持旧 JSON 兼容。
2. 新增 maintenance report / check / recommendation 类型或等价字段。
3. 扩展 deterministic engine，覆盖 expired、missing source、duplicate、entity drift、permission revoked、sensitive export、index status、mature pattern signal。
4. 让 task memory packet builder 继续以 blocking finding 为召回闸门，不因 needs_review / info 自动排除。
5. 接 Tauri command / TS wrapper / App handler。
6. 在 MemoryCenter 增加维护摘要和最近 run / findings 分类，不新增顶级入口。
7. 补 Rust 单测、前端离线测试、禁止文案搜索和回收文档。

如果实现过大，可以拆为 M11a / M11b，但必须回传：maintenance run、finding 类型、task packet guard、UI 摘要、report summary 各自完成到哪里，不能把半包说成 M11 完整完成。

## 8. 验收

必须通过：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
cargo test --lib memory_lint
cargo test --lib task_memory_packet
cargo test --lib memory_entity_relation
cargo test --lib formal_memory
cargo test --lib
rustfmt --check src/memory_lint_store.rs src/memory_lint_engine.rs src/memory_entity_relation_store.rs src/memory_entity_relation_governance.rs src/task_memory_packet_builder.rs src/task_memory_injection.rs src/formal_memory_store.rs src/control_core.rs src/commands.rs src/types.rs src/lib.rs
```

如果实际文件名不同，必须把等价新增 / 修改文件列入 `rustfmt --check`。

必须覆盖的场景：

- 维护 run 写入 run record / report summary，不改正式记忆。
- 过期或 stale memory 生成 finding。
- 缺来源正式记忆被标风险。
- 权限撤回后相关正式记忆不再进入任务包召回。
- 私密 / secret 来源和外发策略冲突时阻断或进入复核。
- 重复 / entity drift 生成 finding，但不自动合并。
- M10 relation / entity source 问题能进入维护 finding，但不把 relation candidate 当事实。
- 索引失败只生成 finding / report，不删除正式记忆。
- mature pattern signal 只进入候选信号，不自动成为正式记忆。
- 损坏 JSON 不会被覆盖。
- expected revision 不匹配会拒绝写入。
- UI 不出现禁止文案。

## 9. evidence / handoff 要求

M11 完成后必须新增：

- `evidence/2026-06-05-memory-layer-m11-maintenance-jobs-and-memory-lint-v1.md`
- `handoffs/2026-06-05-memory-layer-m11-maintenance-jobs-and-memory-lint-v1-result.md`

evidence 必须记录：

- 实际新增 / 修改的 maintenance / lint store、engine、类型、commands、Tauri wrapper、UI 文件。
- 维护 run、finding 分类、report summary 和 task packet guard 的验收结果。
- M11 没有自动修改正式记忆、没有自动 lifecycle、没有自动合并实体、没有自动 mature pattern 的验证。
- UI 摘要、禁止文案和真实窗口 / 截图验收情况。
- 验证命令和结果。
- 边界：未执行真实 worker / Codex，未读写 `/Users/yoyi/.codex`，未接向量库 / 图数据库 / GraphRAG，未写 workflow state。

handoff 必须写清：

- M11 接受为什么。
- M11 不接受为什么。
- 维护 run / finding / report / task packet guard / UI 摘要哪些已完成。
- 下一步应进入 M12，还是先补 M11 剩余 / 截图缺口。

## 10. Stop 条件

遇到以下情况必须停下回传：

- 需要读写 `/Users/yoyi/.codex`。
- 需要执行真实 `codex exec` 或 `codex exec resume`。
- 需要启动真实 worker。
- 需要自动修改正式记忆。
- 需要自动调用 M9 lifecycle 操作。
- 需要自动合并实体、关系或正式记忆。
- 需要让 mature pattern signal 直接成为正式记忆、全局记忆、技能或规则。
- 需要让 relation candidate、observation、candidate、knowledge hit、LLM summary 或 graph/index report 直接影响 worker。
- 需要接向量库、图数据库、GraphRAG 或自动索引重建系统。
- 需要新增一级入口、右侧顶级入口或项目页 tab。
- 发现本任务与 `docs/workbench-frontend-display-boundary-v1.md`、`docs/memory-layer-design-v1.md` 或 `docs/plans/memory-layer-implementation-slice-v1.md` 冲突。

## 11. 回收口径

完成后接受为：

- M11 维护任务和记忆 lint 完成。
- 维护 run、维护 finding、维护 report summary 和任务包 blocking guard 可用。
- 过期、缺来源、重复 / 实体漂移、权限撤回、私密风险、索引状态和 mature pattern signal 有最小检测和摘要。

完成后不接受为：

- M12 成熟模式、跨项目记忆或完整验收完成。
- M13 中间版本记忆系统最终验收完成。
- 自动修复、自动合并、自动废弃、自动冻结、自动归档或自动删除正式记忆完成。
- mature pattern 自动成为正式记忆、技能或全局规则完成。
- 向量库、图数据库、GraphRAG、自动索引重建系统或完整运维后台完成。
- 真实 worker / Codex 已执行。
