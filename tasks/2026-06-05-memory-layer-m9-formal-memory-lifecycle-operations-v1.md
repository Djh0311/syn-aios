# Task Package：Memory Layer M9 Formal Memory Lifecycle Operations v1

状态：已完成。  
用途：实现中间版本记忆层 M9：正式记忆生命周期操作。  
执行方式：一个较大但必须收边界的批次完成；开发重点是正式记忆编辑、废弃、冻结、解冻、归档、合并、拆分、上升为全局记忆、下沉为项目记忆的受控版本化操作，不做关系治理、维护任务、成熟模式或自动推断。

完成记录：

- `evidence/2026-06-05-memory-layer-m9-formal-memory-lifecycle-operations-v1.md`
- `handoffs/2026-06-05-memory-layer-m9-formal-memory-lifecycle-operations-v1-result.md`

## 1. 先说薄弱点

- M1-M2 已能创建正式记忆和从候选受控采纳正式记忆，但正式记忆一旦写入后，当前缺少受控编辑、废弃、冻结、归档、合并、拆分和 scope 调整能力。
- 现有 `MemoryLifecycleStatus` 已有 `memory_active`、`memory_deprecated`、`memory_frozen`、`memory_archived` 等状态，但 `FormalMemoryStore` 主要实现了 create / adoption 路径，还没有生命周期 wrapper。
- M7 记忆中心已能只读展示正式记忆、来源、版本、审计、lint 和任务包 eligibility；M9 才能新增生命周期动作，但必须加确认弹层、影响面摘要和审计，不允许 UI 直接改文件。
- M8 知识库边界已完成，但知识库、Markdown、Obsidian CLI、Canvas、Graph、Bases 都不能绕过正式记忆生命周期状态机。
- 合并、拆分、上升 / 下沉 scope 很容易滑进 M10 关系治理或 M12 跨项目成熟模式；M9 只允许对明确选中的正式记忆做人工受控操作，不做语义 dedupe、图谱推断或跨项目自动提升。

## 2. 任务目标

建立正式记忆生命周期的受控写入链路：

```text
FormalMemoryStore active record
-> lifecycle preview / impact summary / required approval
-> user or project director confirmation
-> controlled lifecycle operation
-> new MemoryVersion snapshot
-> MemoryAuditEvent + MemoryAuditRef
-> updated MemoryRecord current state
-> task memory packet eligibility reflects new status
-> MemoryCenter UI shows old / new version and audit summary
```

M9 完成后可以说：

- 正式记忆可以通过受控操作编辑，但编辑不是覆盖旧内容，而是创建新 `MemoryVersion` 并更新当前 record。
- 正式记忆可以被废弃 / 归档；非 active 记忆默认不再进入任务包 included list。
- 冻结后的记忆不能普通编辑；只能解冻后编辑，或创建替代版本 / 替代记忆。
- 合并 / 拆分会保留来源、旧 memory id、版本、审计和影响范围。
- 项目记忆上升为全局记忆、全局记忆下沉为项目记忆必须记录原因、确认人和适用范围。
- 用户偏好、安全边界、全局蓝图、跨项目记忆和成熟模式相关 lifecycle 变化必须用户确认。
- UI 能显示 lifecycle 操作预览、影响面、确认权、旧版 / 新版和审计摘要。

M9 完成后仍不能说：

- M10 关系 / 实体治理完成。
- M11 维护任务完成。
- M12 成熟模式 / 跨项目候选完成。
- M13 中间版本记忆系统最终验收完成。
- 自动语义 dedupe、图谱合并、因果关系确认或相似度合并完成。
- 维护任务可以自动改正式记忆。
- Obsidian / Markdown / 知识库文档可以直接写正式记忆。
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

开始前必须复核：

- `formal-memories.v1.json` 已有 revision、records、versions、audit_events。
- `MemoryRecord` 已有 `record_version`、`status`、`supersedes_memory_id`、`superseded_by_memory_id`、`audit_refs`。
- `MemoryVersion` 已能保存 `record_snapshot`。
- `MemoryAuditEvent` 和 `MemoryAuditRef` 已能表达 before / after 状态。
- `TaskMemoryPacketBuilder` 已按状态排除 non-active 正式记忆。
- M7 `MemoryCenterView` 目前是只读；M9 才允许新增生命周期动作。
- M8 知识库入口只能生成候选，不能直接写正式记忆。

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
- `evidence/2026-06-05-memory-layer-m8-knowledge-base-and-obsidian-compatible-interface-placeholder-v1.md`
- `handoffs/2026-06-05-memory-layer-m8-knowledge-base-and-obsidian-compatible-interface-placeholder-v1-result.md`

当前实现重点：

- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/formal_memory_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_candidate_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_lint_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_lint_engine.rs`
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

- 新增后端 lifecycle helper，例如 `formal_memory_lifecycle.rs`。
- 新增或扩展类型：
  - `FormalMemoryLifecycleOperationKind`
  - `FormalMemoryLifecyclePreviewInput`
  - `FormalMemoryLifecyclePreview`
  - `FormalMemoryLifecycleInput`
  - `FormalMemoryLifecycleOutput`
  - `FormalMemoryLifecycleImpactSummary`
  - `FormalMemoryRequiredApproval`
  - `FormalMemoryLifecycleAudit`
  - `FormalMemoryMergePlan`
  - `FormalMemorySplitPlan`
  - `FormalMemoryScopeChangePlan`
- 新增 Tauri command / wrapper，例如：
  - `preview_formal_memory_lifecycle_operation`
  - `record_formal_memory_lifecycle_operation`
  - 或按现有命名拆成更小命令。
- 支持 lifecycle operation kind：
  - `revise`
  - `deprecate`
  - `freeze`
  - `unfreeze`
  - `archive`
  - `merge`
  - `split`
  - `promote_to_global`
  - `demote_to_project`
- 每次成功操作必须：
  - 校验 context binding。
  - 校验 expected store revision。
  - 校验 actor / confirmation rule。
  - 创建新的 `MemoryVersion`。
  - 写 `MemoryAuditEvent`。
  - 在 affected records 上追加 `MemoryAuditRef`。
  - 更新 `revision` 和 `updated_at`。
  - 使用原子写和损坏 JSON 防覆盖策略。
- `revise` 必须保留旧版本，更新 current record 的 claim / body / source_refs / record_version，并写 `manual_revision` 或等价 change type。
- `deprecate` 必须把 status 置为 `memory_deprecated`，不物理删除，不清空来源。
- `freeze` 必须把 status 置为 `memory_frozen`，并阻止普通 revise。
- `unfreeze` 必须记录原因和确认人，恢复到 `memory_active` 或明确状态。
- `archive` 必须把 status 置为 `memory_archived`，默认不进入任务包。
- `merge` 必须只处理明确选中的正式记忆；保留 old memory ids，更新 supersedes / superseded_by，创建新 record 或明确 target record 新版本。
- `split` 必须只处理明确选中的正式记忆；创建明确的新 records / versions，并保留 source memory id 和审计。
- `promote_to_global` / `demote_to_project` 必须记录新 scope、原因、确认人和适用范围。
- 前端可在 M7 记忆中心新增 lifecycle 操作区、预览区和确认动作。
- 更新 M4/M6 task memory packet eligibility 读模型，使 deprecated / frozen / archived / superseded 记忆不进入 included list。
- 新增 Rust 单测和前端离线测试。
- 更新 evidence / handoff / `CURRENT.md` / `tasks/README.md` / `AUTHORITY.md` / `STAGE_PLAN.md` / 阶段计划。

本任务显式授权的数据变更：

- 允许写 `formal-memories.v1.json`：
  - 更新 records。
  - 新增 versions。
  - 新增 audit_events。
  - 更新 revision / updated_at。
- 允许读取 `memory-candidates.v1.json`、`observations.v1.json`、`memory-lint.v1.json` 和 workflow state，用于影响面和引用摘要。
- 不允许写 `memory-candidates.v1.json`，除非只是保留 M2 已有 adoption 回链逻辑；M9 默认不改候选。
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
- 不物理删除正式记忆。
- 不直接覆盖旧版本。
- 不允许 UI 直接改 JSON 后视为正式变更。
- 不允许 Markdown、Obsidian CLI、知识库、Canvas、Graph、Bases 或向量命中绕过状态机。
- 不允许维护任务自动改正式记忆。
- 不自动合并相似记忆；语义 dedupe、实体合并和关系治理属于 M10。
- 不自动上升为全局记忆；成熟模式和跨项目提升属于 M12。
- 不把 lifecycle 操作说成中间版本完整记忆系统完成。

如果执行者认为必须自动合并、自动拆分、自动提升或运行维护任务，必须停下回传，说明为什么不能留到 M10 / M11 / M12。

## 5.1 UI 显示边界确认

本任务会改前端：

- [ ] 不改前端、不改读模型、不改 UI 文案。
- [x] 改前端类型 / Tauri wrapper。
- [x] 改读模型摘要或状态显示。
- [x] 改已有页面局部 UI。
- [x] 新增页面内 lifecycle 操作按钮、预览面板和确认动作，但不新增一级入口 / 右侧顶级入口 / 项目页 tab。

已读取：

- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`
- `CURRENT.md`
- `tasks/README.md`
- `docs/memory-layer-design-v1.md`
- `docs/plans/memory-layer-implementation-slice-v1.md`

本任务允许显示：

- 正式记忆 lifecycle 操作区。
- 操作按钮：`编辑提案`、`废弃`、`冻结`、`解冻`、`归档`、`合并`、`拆分`、`上升为全局`、`下沉为项目`。
- 操作预览：当前状态、目标状态、影响范围、确认权、来源 / 版本 / 审计摘要、任务包入选影响。
- 旧版本 / 新版本摘要。
- 操作后审计事件和版本号。
- 明确文案：`编辑会创建新版本，不覆盖旧版本`、`废弃不是删除`、`冻结后不能普通编辑`、`非 active 记忆默认不进任务包`。

本任务禁止显示：

- 不新增一级入口；复用现有 `记忆`。
- 不新增右侧顶级入口。
- 不新增项目页 tab。
- 不把记忆中心做成 raw 治理后台。
- 不显示 raw schema、raw event、完整 sidecar JSON、数据库路径大表或完整审计日志。
- 不显示“删除正式记忆”“永久删除”“直接覆盖”“自动合并”“自动上升全局”“维护任务已自动修正”等误导动作或文案。
- 不把秘书显示为最终确认人；秘书只能输出影响面说明，不能批准生命周期变更。
- 不把 M9 显示为完整记忆系统完成。

显示位置：

- 一级入口：复用现有 `记忆`，不新增。
- 右侧入口：不改。
- 项目页：只允许项目相关记忆生命周期摘要，不新增 tab，不占用工作流画布主区域。
- 画布：不改画布主区域。
- 记忆入口：本轮主要落地位置，显示生命周期动作、预览、确认和结果摘要。
- 知识库入口：不直接操作正式记忆；只显示来源反向链接。
- 智能体入口：不改。
- 管理入口：不改。

中间版本范围：

- 本轮必须落地：正式记忆 revise / deprecate / freeze / unfreeze / archive / merge / split / promote / demote 的受控最小实现，版本和审计完整。
- 本轮只做读模型 / 摘要：影响面报告、任务包入选影响、来源 / 权限 / lint 摘要。
- 本轮后置：关系图治理、实体 dedupe、维护任务自动发现、成熟模式、跨项目模式挖掘、完整通知 / 待办系统。

后端和数据依赖：

- 需要后端正式读写模型：必须通过 Rust lifecycle wrapper 写 `formal-memories.v1.json`。
- 需要审计 / 日志 / 权限 / 状态机：每个操作必须写 `MemoryVersion` 和 `MemoryAuditEvent`；确认权必须由后端校验，不只靠 UI。
- 不能用假数据伪装：不能伪造版本、审计、影响面、确认人、scope change 或 task eligibility。

UI 文案边界：

- 禁止说：`删除正式记忆`、`永久删除`、`直接覆盖`、`自动合并`、`自动修复`、`维护任务已修改正式记忆`、`Obsidian 已修改正式记忆`、`中间版本记忆层已完成`。
- 允许说：`废弃`、`归档`、`冻结`、`解冻`、`创建新版本`、`保留旧版本`、`需要用户确认`、`需要项目主管确认`、`非 active 记忆不进入任务包`。

验收：

- 类型检查：`npm run typecheck`
- 离线交互测试：`npm run test:offline-interaction`
- 构建：`npm run build`
- 真实窗口 / 截图验收：涉及记忆入口 lifecycle 操作，必须做真实浏览器或 Tauri 截图验收；如果没有可用截图工具，不能声称 UI 验收完成。
- 未验收项必须写入 evidence / handoff。

## 6. 确认权规则

最低规则：

- `user_preference`：必须用户确认。
- `global_blueprint`：必须用户确认或全局主管复核后用户确认；不能只有项目主管。
- `project_memory` 低风险本项目事实：项目主管可确认。
- `project_memory` 高风险 / private / secret / 跨项目影响：必须用户确认。
- `workflow_summary` / `session_summary`：项目主管可确认，涉及跨项目或用户偏好时必须用户确认。
- `mature_pattern`：M9 只能保留为正式记忆生命周期对象；成熟模式确认和跨项目提升仍属于 M12，默认必须用户确认。
- `promote_to_global`：必须用户确认。
- `demote_to_project`：如果影响全局规则或跨项目行为，必须用户确认。

秘书边界：

- 秘书可以整理影响面报告和解释差异。
- 秘书不能批准、写入、废弃、冻结、归档、合并、拆分、上升或下沉正式记忆。

## 7. 实施建议

建议按以下顺序实现：

1. 后端 lifecycle preview：先生成 deterministic preview，不写 store。
2. 后端 lifecycle record：统一 guard、revision、版本、审计和原子写。
3. 覆盖 revise / deprecate / freeze / unfreeze / archive 的单记录操作。
4. 覆盖 merge / split / promote / demote 的多记录或 scope 操作。
5. 接 Tauri wrapper 和 TS 类型。
6. 在 `MemoryCenterView` 增加 lifecycle 预览和确认动作，复用 `PermissionDialog`。
7. 调整 M7 memory center read model 和 M4/M6 task eligibility 文案。
8. 补 Rust 单测、前端离线测试、禁止文案搜索和回收文档。

如果实现变得过大，可以先拆为 M9a / M9b，但必须回传：哪些 lifecycle 操作已完成，哪些仍未完成，不能把半包说成 M9 完整完成。

## 8. 验收

必须通过：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
cargo test --lib formal_memory_lifecycle
cargo test --lib formal_memory
cargo test --lib task_memory_packet
cargo test --lib memory_lint
cargo test --lib memory_candidate
cargo test --lib
rustfmt --check src/formal_memory_lifecycle.rs src/formal_memory_store.rs src/task_memory_packet_builder.rs src/task_memory_injection.rs src/memory_lint_store.rs src/memory_lint_engine.rs src/control_core.rs src/commands.rs src/types.rs src/lib.rs
```

如果没有新增 `formal_memory_lifecycle.rs`，必须把等价新增 / 修改文件列入 `rustfmt --check`。

必须覆盖的场景：

- 编辑一条正式记忆后能看到旧版和新版，旧版未被覆盖。
- 废弃记忆默认不再进入任务包。
- 冻结记忆不能被普通编辑。
- 解冻必须写原因和审计。
- 归档记忆不物理删除。
- 合并两条记忆后旧记忆保留历史，新记忆或 target 记忆有来源和审计。
- 拆分一条记忆后新记忆保留来源和 source memory id。
- 项目记忆上升为全局记忆需要用户确认。
- `expected_store_revision` 不匹配会拒绝写入。
- 损坏 JSON 不会被覆盖。
- UI 不出现禁止文案。

## 9. evidence / handoff 要求

M9 完成后必须新增：

- `evidence/2026-06-05-memory-layer-m9-formal-memory-lifecycle-operations-v1.md`
- `handoffs/2026-06-05-memory-layer-m9-formal-memory-lifecycle-operations-v1-result.md`

evidence 必须记录：

- 实际新增 / 修改的后端 lifecycle helper、类型、commands、Tauri wrapper、UI 文件。
- 每类 lifecycle 操作的验收结果。
- 版本、审计、revision guard 和原子写验证。
- UI 确认权、影响面和禁止文案验证。
- task memory packet eligibility 对 deprecated / frozen / archived / superseded 的排除验证。
- 是否做了真实浏览器或 Tauri 截图验收；如果没有，必须明确写“真实窗口 / 截图验收未完成”。
- 验证命令和结果。
- 边界：未执行 Obsidian CLI、未自动合并、未自动维护、未执行真实 worker / Codex、未读写 `/Users/yoyi/.codex`。

handoff 必须写清：

- M9 接受为什么。
- M9 不接受为什么。
- 若拆成 M9a / M9b，哪些 lifecycle operation 已完成，哪些未完成。
- 下一步应进入 M10 还是先补 M9 剩余 / 截图缺口。

## 10. Stop 条件

遇到以下情况必须停下回传：

- 需要读写 `/Users/yoyi/.codex`。
- 需要执行真实 `codex exec` 或 `codex exec resume`。
- 需要启动真实 worker。
- 需要物理删除正式记忆。
- 需要直接覆盖旧版本而不创建 `MemoryVersion`。
- 需要绕过用户 / 项目主管确认权。
- 需要让维护任务自动改正式记忆。
- 需要 Obsidian CLI、Markdown、知识库或向量命中直接写正式记忆。
- 需要自动 dedupe、自动实体合并、图谱推断或因果关系确认。
- 需要新增一级入口、右侧顶级入口或项目页 tab。
- 发现本任务与 `docs/workbench-frontend-display-boundary-v1.md`、`docs/memory-layer-design-v1.md` 或 `docs/plans/memory-layer-implementation-slice-v1.md` 冲突。

## 11. 回收口径

完成后接受为：

- M9 正式记忆生命周期操作完成。
- 正式记忆 revise / deprecate / freeze / unfreeze / archive / merge / split / promote / demote 可通过受控版本化操作完成。
- 版本、审计、确认权、revision guard 和 task eligibility 已接入。

完成后不接受为：

- M10 关系和实体治理完成。
- M11 维护任务完成。
- M12 成熟模式、跨项目记忆和完整验收完成。
- M13 中间版本记忆系统最终验收完成。
- 自动合并 / 自动 dedupe / 图谱推断完成。
- Obsidian / 知识库可直接写正式记忆。
- 真实 worker / Codex 已执行。
