# Task Package：Memory Layer M5 Conflict And Memory Lint Minimal Blocking v1

状态：已完成。  
用途：实现中间版本记忆层 M5：冲突和记忆 lint 最小阻断。  
执行方式：一个中等批次完成，最终统一验收；开发重点在后端 deterministic lint、采纳前阻断、任务记忆包预览阻断和审计 / finding 留痕，UI 只做必要只读摘要。

完成记录：

- `evidence/2026-06-04-memory-layer-m5-conflict-and-memory-lint-minimal-blocking-v1.md`
- `handoffs/2026-06-04-memory-layer-m5-conflict-and-memory-lint-minimal-blocking-v1-result.md`

## 1. 先说薄弱点

- M1 / M1.1 / M2 / M3 / M4 已完成正式记忆 store、上下文绑定、候选采纳、observation 入口和任务记忆包预览。
- 现在 M4 已能排除 `conflict_refs` 或非 active 状态，但还没有最小 `MemoryLintFinding` / `MemoryConflict` 生成链路。
- 如果没有 M5，M6 任务包注入前会缺一个明确的冲突闸门：候选采纳时可能写入与 active 正式记忆明显冲突的内容，任务包预览也只能依赖已有 `conflict_refs`。
- M5 不是完整维护任务系统，不是自动整理记忆库，也不是正式记忆生命周期操作；M5 只做确定性冲突 / lint finding 和最小 blocking。

## 2. 任务目标

实现最小记忆冲突和 lint 阻断链路：

```text
FormalMemoryStore / MemoryCandidateStore / MemoryLintStore
-> MemoryLintEngine deterministic rules
-> MemoryLintFinding / MemoryConflict
-> candidate adoption 前阻断 blocking finding
-> TaskMemoryPacketBuilder 排除 blocking conflict / lint finding
-> 只读摘要显示阻断数量和 reason
```

M5 完成后可以说：

- 工作台能用确定性规则发现最小冲突 / stale / 权限撤回 finding。
- 候选采纳前会被 blocking conflict 阻断。
- 任务记忆包预览会排除 blocking conflict / lint finding 命中的正式记忆。
- 维护 lint run 只能生成 finding，不会自动改正式记忆状态。

M5 完成后仍不能说：

- 任务包注入完成。
- worker 已收到记忆包。
- 完整正式记忆生命周期完成。
- 维护任务系统完成。
- LLM 能自动判定、合并、废弃或删除正式记忆。
- 中间版本记忆层完成。

## 3. 前置条件

必须已完成：

- `tasks/2026-06-03-memory-layer-m1-formal-memory-store-and-audit-v1.md`
- `tasks/2026-06-03-memory-layer-m1-1-formal-memory-context-binding-guard-v1.md`
- `tasks/2026-06-03-memory-layer-m2-candidate-to-formal-adoption-v1.md`
- `tasks/2026-06-04-memory-layer-m3-observation-store-and-workflow-entry-v1.md`
- `tasks/2026-06-04-memory-layer-m4-task-memory-packet-builder-and-preview-v1.md`

M5 开始前必须复核：

- `MemoryConflict` 类型已有基础占位，但还不是完整冲突 store。
- `FormalMemoryStoreV1` 当前已有 `records[]`、`versions[]`、`audit_events[]` 和 `MemoryRecord.conflict_refs[]`。
- M4 的 `TaskMemoryPacketBuilder` 已经把 `memory_conflicted` 或 `conflict_refs.len() > 0` 排除为 `conflicted`。
- M5 如新增 store 字段或 sidecar，必须兼容既有 JSON；不能让旧 sidecar 因缺新字段而无法读取。

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
- `docs/plans/2026-06-01-memory-governance-schema-v1.md`
- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`

前置记录：

- `evidence/2026-06-03-memory-layer-m1-formal-memory-store-and-audit-v1.md`
- `evidence/2026-06-03-memory-layer-m1-1-formal-memory-context-binding-guard-v1.md`
- `evidence/2026-06-03-memory-layer-m2-candidate-to-formal-adoption-v1.md`
- `evidence/2026-06-04-memory-layer-m3-observation-store-and-workflow-entry-v1.md`
- `evidence/2026-06-04-memory-layer-m4-task-memory-packet-builder-and-preview-v1.md`
- `handoffs/2026-06-04-memory-layer-m4-task-memory-packet-builder-and-preview-v1-result.md`

当前实现：

- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/formal_memory_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_candidate_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/task_memory_packet_builder.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/control_core.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `prototypes/productized-desktop-shell/src/lib/candidateGovernance.ts`
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

## 5. 范围

允许：

- 新增后端 lint / conflict 模块，例如 `memory_lint_store.rs` 和 `memory_lint_engine.rs`。
- 新增独立 `memory-lint.v1.json` sidecar，包含 revision、findings、runs / audit refs、warnings，并使用 lock、备份和原子写。
- 新增后端类型：
  - `MemoryLintStoreV1`
  - `MemoryLintFinding`
  - `MemoryLintRunInput`
  - `MemoryLintRunOutput`
  - `MemoryLintFindingSeverity`
  - `MemoryLintFindingStatus`
  - `MemoryLintFindingType`
- 新增后端命令：
  - `run_memory_lint`
  - `load_memory_lint_store`
- 新增控制核心 helper：
  - `validate_memory_lint_run(...)`
  - `evaluate_memory_lint_finding(...)`
  - `ensure_no_blocking_memory_conflict_before_adoption(...)`
- 在 `adopt_memory_candidate_to_formal_memory` 前执行确定性冲突检查；有 blocking finding 时拒绝采纳，并写 lint finding / audit 记录。
- 在 `TaskMemoryPacketBuilder` 中读取 lint store；存在 open blocking finding 或 `conflict_refs` 命中时排除为 `conflicted`。
- 维护 lint run 只生成 finding，不自动修改 `MemoryRecord.status`、`versions[]` 或 `conflict_refs[]`。
- 前端新增类型、Tauri wrapper 和只读摘要 helper。
- 项目工作流侧栏候选治理 / 记忆摘要区域显示最小只读“记忆 lint 阻断摘要”。
- 新增 Rust 单测和前端离线测试。
- 更新 evidence / handoff / `CURRENT.md` / `tasks/README.md` / 阶段计划。

禁止：

- 不执行真实 worker。
- 不执行真实 Codex。
- 不执行 `codex exec` / `codex exec resume`。
- 不读写 `/Users/yoyi/.codex`。
- 不把 M5 解释成任务包注入；那是 M6。
- 不让 LLM 推断冲突后直接废弃、冻结、归档、合并或删除正式记忆。
- 不自动修改正式记忆状态。
- 不自动新增正式记忆版本；冲突处理只允许写 lint finding / lint audit，且不改变 `MemoryRecord` 内容。
- 不自动写 `MemoryRecord.conflict_refs[]`；M5 默认通过 `MemoryLintFinding` 阻断，正式记忆状态和引用变更留给后续生命周期任务。
- 不把 candidate、observation、knowledge hit、LLM summary 当正式记忆。
- 不改 `workflow-state.v0.json` 结构。
- 不迁移数据库。
- 不接 Obsidian 原生读写。
- 不接向量库或图数据库。
- 不扫描完整 transcript。
- 不用假数据伪装后端 lint 能力已完成。

## 5.1 UI 显示边界确认

本任务会改前端：

- [ ] 不改前端、不改读模型、不改 UI 文案。
- [x] 改前端类型 / Tauri wrapper，但不新增可见 UI。
- [x] 改读模型摘要或状态显示。
- [x] 改已有页面局部 UI。
- [ ] 新增一级入口、右侧顶级入口、项目页 tab、独立面板或确认动作。

已读取：

- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`
- `CURRENT.md`
- `tasks/README.md`

本任务允许显示：

- “记忆 lint 阻断摘要”最小只读摘要。
- open finding 数量。
- blocking finding 数量。
- finding type / severity / status。
- 被阻断的候选采纳 reason。
- 任务记忆包预览中因 conflict / lint 排除的数量。
- 文案明确：`lint 只生成待处理 finding`、`blocking finding 会阻止进入任务包`、`不会自动修改正式记忆`。

本任务禁止显示：

- 不新增一级入口。
- 不新增右侧顶级入口。
- 不把 lint / conflict 铺进项目工作流画布主区域。
- 不新增完整记忆管理页面。
- 不显示 raw event、schema、数据库路径大表、完整审计日志或完整 sidecar JSON。
- 不显示未实现的“一键合并”“自动废弃”“自动修复全部记忆”按钮。
- 不显示“AI 已判断冲突并自动修复”“系统已废弃旧记忆”“冲突已自动解决”“任务包注入已完成”。

显示位置：

- 一级入口：不改。
- 右侧入口：不改。
- 项目页：允许在项目工作流侧栏、节点详情或项目记忆相关区域显示最小只读摘要。
- 画布：不在画布主区域新增 lint / conflict 面板。
- 记忆入口：允许显示最小 finding 摘要，不做完整记忆管理页面。
- 知识库入口：不改。
- 智能体入口：不改。
- 管理入口：不改。

中间版本范围：

- 本轮必须落地：后端 deterministic lint、blocking finding、采纳前阻断、任务包预览阻断和最小读模型。
- 本轮只做读模型 / 摘要：finding 数量、blocking 数量、reason 和 warnings。
- 本轮后置：完整记忆中心、finding 生命周期 UI、冲突合并向导、自动维护任务调度、图关系可视化、Obsidian / 知识库联动。

后端和数据依赖：

- 预览必须来自 `FormalMemoryStore`、`MemoryCandidateStore`、`MemoryLintStore` 和后端 lint engine。
- blocking 决策必须来自确定性规则或已存在的人工 / 控制核心标记，不能由前端 mock。
- `candidate`、`observation`、`knowledge hit`、`LLM summary` 不能绕过正式记忆状态机。

UI 文案边界：

- 禁止说：“AI 已自动解决冲突”“系统已废弃旧记忆”“旧记忆已自动更新”“任务包注入已完成”“worker 已收到记忆包”“正式记忆生命周期完成”。
- 允许说：“记忆 lint 阻断摘要”“blocking finding 会阻止进入任务包”“lint 只生成待处理 finding”“不会自动修改正式记忆”。

验收：

- 必须跑 `npm run typecheck`。
- 必须跑 `npm run test:offline-interaction`。
- 必须跑 `npm run build`。
- 如改变项目页布局，必须做真实窗口或浏览器截图验收；如果当前对话没有可用截图工具，必须在 evidence / handoff 写明“真实窗口 / 截图验收未完成”。

## 6. 建议数据对象

第一版推荐新增独立 sidecar，不直接把所有 finding 塞进正式记忆 store：

```ts
type MemoryLintStoreV1 = {
  store_version: "memory_lint_store.v1";
  project_id?: string;
  workflow_id?: string;
  revision: number;
  findings: MemoryLintFinding[];
  runs: MemoryLintRunRecord[];
  updated_at: string;
  warnings: string[];
};
```

```ts
type MemoryLintRunRecord = {
  run_id: string;
  lint_intent: "candidate_adoption_guard" | "task_packet_guard" | "maintenance_preview";
  actor_id: string;
  actor_role: "project_director" | "global_director" | "system";
  finding_ids: string[];
  blocking_count: number;
  status: "succeeded" | "blocked" | "failed";
  reason: string;
  created_at: string;
};
```

```ts
type MemoryLintFinding = {
  finding_id: string;
  schema_version: "memory_governance.v1";
  finding_type:
    | "duplicate_claim"
    | "claim_conflict"
    | "source_permission_revoked"
    | "authority_superseded"
    | "stale_memory"
    | "missing_source"
    | "candidate_conflicts_with_active_memory";
  severity: "blocking" | "needs_review" | "info";
  status: "open" | "acknowledged" | "resolved" | "dismissed";
  source_kind: "memory_record" | "memory_candidate" | "lint_run";
  source_id: string;
  target_memory_id?: string;
  target_candidate_key?: string;
  scope_type?: string;
  memory_type?: string;
  claim?: string;
  summary: string;
  recommended_action:
    | "block_adoption"
    | "exclude_from_task_packet"
    | "review_and_deprecate"
    | "review_source_permission"
    | "review_staleness"
    | "no_action";
  evidence_refs: MemorySourceRef[];
  audit_event_id?: string;
  created_at: string;
  updated_at: string;
};
```

```ts
type MemoryLintRunInput = {
  project_root: string;
  project_id?: string;
  workflow_id?: string;
  actor_id: string;
  actor_role: "project_director" | "global_director" | "system";
  lint_intent: "candidate_adoption_guard" | "task_packet_guard" | "maintenance_preview";
  candidate_key?: string;
  task_id?: string;
  revoked_source_ids?: string[];
  expected_formal_store_revision?: number;
  expected_candidate_store_revision?: number;
  expected_lint_store_revision?: number;
  dry_run?: boolean;
};
```

## 7. 确定性规则

第一版只做保守确定性规则，允许漏报，不允许靠猜测制造高风险误阻断。

必须实现：

- 同 `scope_type` / `project_id` / `workflow_id` / `memory_type` 下，normalized claim 完全相同，生成 `duplicate_claim`。
- 同 scope + 同 memory_type 下，normalized token Jaccard 相似度达到实现中写死阈值时，生成 `duplicate_claim` 或 `needs_review` finding；阈值必须有单测。
- 候选 claim 与 active 正式记忆 claim 命中明确互斥词对时，生成 `candidate_conflicts_with_active_memory`，severity 为 `blocking`。
- `revoked_source_ids[]` 命中正式记忆 `source_refs[].source_id` 时，生成 `source_permission_revoked`；对任务包召回必须 blocking。
- 用户最新确认或当前权威来源覆盖旧偏好时，生成 `authority_superseded` 或 `stale_memory` finding；不能自动废弃旧记忆。
- 已存在 `MemoryRecord.conflict_refs[]` 或状态 `memory_conflicted` 的正式记忆，任务包仍必须排除。

第一版不做：

- 不做 LLM contradiction 判断。
- 不做跨语言语义相似。
- 不做向量相似。
- 不扫描完整 transcript。
- 不把知识库命中当正式冲突来源。

## 8. 阻断点

候选采纳前：

- `adopt_memory_candidate_to_formal_memory` 必须在写正式记忆前调用 lint guard。
- 若发现 open blocking finding，必须拒绝采纳。
- 拒绝采纳不能创建 `MemoryRecord`、`MemoryVersion` 或正式记忆采纳 audit。
- 拒绝采纳应写 lint finding / lint run audit，或在输出里返回 finding；如果写入失败，不能继续采纳。

任务记忆包预览：

- `TaskMemoryPacketBuilder` 必须读取 `MemoryLintStore`。
- open blocking finding 命中的正式记忆必须进入 excluded list，reason 为 `conflicted`，detail 里说明来自 memory lint finding。
- `candidate` / `observation` 仍只能作为待审查材料，不因 lint finding 进入 included list。
- 本轮仍不注入真实任务包。

维护 lint run：

- `run_memory_lint` 的 `maintenance_preview` 只能生成 finding。
- 不自动修改正式记忆状态。
- 不自动新增正式记忆版本。
- 不自动写 `conflict_refs[]`。

## 9. 验收标准

后端必须验证：

- 冲突候选采纳被拒绝，且没有创建正式记忆 record / version / adoption audit。
- 非冲突候选仍可按 M2 规则采纳。
- 同 scope + 同 type + duplicate claim 生成 finding。
- 用户最新确认覆盖旧偏好时，旧偏好生成 `authority_superseded` 或 `stale_memory` finding，但正式记忆状态不自动变化。
- revoked source 命中时，任务记忆包预览排除相关正式记忆。
- open blocking finding 命中时，任务记忆包预览排除相关正式记忆，reason 为 `conflicted`。
- maintenance lint run 只生成 finding，不改 formal memory record / version / status。
- 损坏 `memory-lint.v1.json` 必须拒绝覆盖。
- revision 冲突必须被拒绝。

前端必须验证：

- lint 摘要显示 open / blocking / needs_review 数量。
- UI 文案包含“不会自动修改正式记忆”或同等边界。
- UI 文案不包含“AI 已自动解决冲突”“系统已废弃旧记忆”“任务包注入已完成”等越界说法。
- 任务记忆包预览仍显示 `预览未注入任务包`。

建议验证命令：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
cargo test --lib memory_lint
cargo test --lib task_memory_packet
cargo test --lib memory_candidate_adoption
cargo test --lib formal_memory
cargo test --lib
rustfmt --check src/memory_lint_store.rs src/memory_lint_engine.rs src/formal_memory_store.rs src/memory_candidate_store.rs src/task_memory_packet_builder.rs src/control_core.rs src/commands.rs src/types.rs
```

## 10. 回收要求

执行完成后必须新增：

- `evidence/2026-06-04-memory-layer-m5-conflict-and-memory-lint-minimal-blocking-v1.md`
- `handoffs/2026-06-04-memory-layer-m5-conflict-and-memory-lint-minimal-blocking-v1-result.md`

并同步：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `docs/plans/memory-layer-implementation-slice-v1.md`

回收结论必须明确：

- 接受为 M5 冲突和记忆 lint 最小阻断完成。
- 不接受为任务包注入完成。
- 不接受为完整维护任务系统完成。
- 不接受为正式记忆生命周期完成。
- 不接受为中间版本记忆层完成。
