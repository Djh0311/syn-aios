# Task Package：Memory Layer M1.1 Formal Memory Context Binding Guard v1

状态：已完成，依据见 `../evidence/2026-06-03-memory-layer-m1-1-formal-memory-context-binding-guard-v1.md` 与 `../handoffs/2026-06-03-memory-layer-m1-1-formal-memory-context-binding-guard-v1-result.md`。  
用途：修补 M1 复核发现的上下文绑定校验缺口，确保正式记忆创建不能只靠前端传入字段自证项目归属。  
执行方式：一个小批次完成，代码和测试一起做，最终统一验收。

## 1. 先说薄弱点

- M1 已完成正式记忆 store / version / audit 骨架，但 `create_formal_memory_record` 当前主要校验请求字段内部一致性。
- `project_root` 当前主要是非空校验，没有证明 `project_id`、`workflow_id`、`scope.project_id`、`scope.workflow_id` 真属于这个 `project_root`。
- M2 会做候选采纳为正式记忆；如果 M1.1 不先补齐，候选可能被采纳到错误项目或错误 workflow 范围。
- 本任务只补上下文绑定校验，不做候选采纳。

## 2. 任务目标

为正式记忆创建加上最小但全面的上下文绑定校验：

```text
CreateFormalMemoryRecordInput
-> project_root 非空
-> 后端推导 expected_project_id / expected_workflow_id
-> 校验 request.project_id / request.workflow_id / scope.project_id / scope.workflow_id
-> 可选校验 project_root 是否存在于当前 workflow state projects[]
-> 通过后才允许 FormalMemoryStore 写 record / version / audit
```

核心目标：

- 不能只因为 `project_id == scope.project_id` 就认为合法。
- 必须确认这些 ID 与 `project_root` 的后端推导结果一致。
- 项目主管只能创建本项目 / 本 workflow / 本 session 作用域正式记忆。

## 3. 必须先读

- `CURRENT.md`
- `tasks/README.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `docs/plans/memory-layer-implementation-slice-v1.md`
- `docs/memory-layer-design-v1.md`
- `tasks/2026-06-03-memory-layer-m1-formal-memory-store-and-audit-v1.md`
- `evidence/2026-06-03-memory-layer-m1-formal-memory-store-and-audit-v1.md`
- `handoffs/2026-06-03-memory-layer-m1-formal-memory-store-and-audit-v1-result.md`

当前实现：

- `prototypes/productized-desktop-shell/src-tauri/src/formal_memory_store.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/control_core.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

## 4. 范围

允许：

- 在 Rust 后端新增 helper，例如：
  - `validate_formal_memory_context_binding(...)`
  - `expected_project_id_for_root(...)`
  - `expected_workflow_id_for_root(...)`
- 复用现有 `project_id(project_root)` 和 `default_workflow_id(project_root)`，如因可见性需要可做小范围 wrapper。
- 在 `create_formal_memory_record` 命令入口或 `formal_memory_store::create_record` 前增加上下文绑定校验。
- 对 `workflow_state.v0.json` 做只读检查，确认 `project_root` 是否已经在当前 workflow state 的 `projects[]` 中。
- 新增 Rust 单测。
- 更新 evidence / handoff / `CURRENT.md` / `tasks/README.md`。

禁止：

- 不实现 M2 候选采纳。
- 不新增“采纳候选为正式记忆”命令。
- 不改 `workflow-state.v0.json` 结构。
- 不迁移数据库。
- 不执行真实 Codex。
- 不执行 `codex exec` / `codex exec resume`。
- 不读写 `/Users/yoyi/.codex`。
- 不接 Obsidian、向量库或图数据库。
- 不把秘书、worker 或黑板候选变成正式记忆写入者。

## 5. 必须校验的绑定规则

给定请求：

```text
project_root
project_id
workflow_id
scope.project_id
scope.workflow_id
scope.scope_type
actor_role
```

后端必须推导：

```text
expected_project_id = project_id(project_root)
expected_workflow_id = default_workflow_id(project_root)
```

必须满足：

- `project_root.trim()` 非空。
- 如果 `project_id` 存在，必须等于 `expected_project_id`。
- 如果 `scope.project_id` 存在，必须等于 `expected_project_id`。
- 如果 `scope.scope_type` 是 `project` / `workflow` / `session`，`scope.project_id` 必须存在并等于 `expected_project_id`。
- 如果 `workflow_id` 存在，必须等于 `expected_workflow_id`，除非后端已经有明确多 workflow registry；当前 M1.1 暂不做多 workflow 例外。
- 如果 `scope.workflow_id` 存在，必须等于 `expected_workflow_id`。
- 如果 `scope.scope_type` 是 `workflow` / `session`，`scope.workflow_id` 必须存在并等于 `expected_workflow_id`。
- `project_director` 只能在 `project` / `workflow` / `session` scope 下写本项目正式记忆。
- `user` 可以创建用户确认范围内的正式记忆，但如果写项目 / workflow / session scope，也必须满足对应绑定。
- `system` 仍默认不能创建高风险正式记忆。
- `global_director` 仍沿用 M1 保守策略：暂不允许创建正式全局记忆，除非后续任务单独授权。

建议增加只读项目存在性校验：

- 如果 workflow state 已存在且有 `projects[]`，`project_root` 必须在其中。
- 如果 workflow state 不存在或没有 projects，M1.1 可选择保守拒绝，或保守允许但必须返回 warning；建议保守拒绝，避免正式记忆写入游离项目。

## 6. 测试要求

Rust 必须新增或扩展测试：

1. `formal_memory_context_accepts_matching_project_and_workflow`
   - `project_root`、`project_id`、`workflow_id`、scope 全部匹配，创建成功。

2. `formal_memory_context_rejects_mismatched_project_id`
   - `project_root` 属于 A，但 `project_id` / `scope.project_id` 是 B，拒绝。

3. `formal_memory_context_rejects_mismatched_workflow_id`
   - `workflow_id` 或 `scope.workflow_id` 不等于 `default_workflow_id(project_root)`，拒绝。

4. `formal_memory_context_rejects_project_director_cross_project`
   - `project_director` 试图写其他项目，拒绝。

5. `formal_memory_context_rejects_missing_project_in_workflow_state`
   - 当前 workflow state 有 projects，但没有该 `project_root`，拒绝。

6. `formal_memory_context_keeps_existing_m1_guards`
   - 无来源、候选状态、损坏 JSON、revision 冲突仍然按 M1 规则拒绝。

前端离线测试可选：

- 如果 UI 只读摘要不变，可以不改前端测试。
- 如果错误提示进入 UI，需要补“项目绑定错误不显示成正式记忆写入成功”。

## 7. 验证命令

至少运行：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
cargo test --lib formal_memory
cargo test --lib formal_memory_store
rustfmt --check src/formal_memory_store.rs src/control_core.rs
```

如只改 Rust，可说明为什么没有跑前端构建；但推荐仍跑 typecheck / offline interaction，避免 Tauri 类型漂移。

## 8. 验收标准

接受为完成：

- 正式记忆创建不再只靠请求字段内部一致性判断项目归属。
- `project_root`、`project_id`、`workflow_id`、scope 绑定校验完整。
- 当前 workflow state 项目存在性有明确校验或明确 warning 策略。
- M1 原有 record / version / audit / damaged JSON / revision conflict / candidate separate 测试仍通过。

不接受为完成：

- M2 候选采纳完成。
- 正式记忆生命周期完成。
- 任务包记忆注入完成。
- 中间版本记忆层完成。

## 9. 回收记录要求

完成后新增：

- `evidence/2026-06-03-memory-layer-m1-1-formal-memory-context-binding-guard-v1.md`
- `handoffs/2026-06-03-memory-layer-m1-1-formal-memory-context-binding-guard-v1-result.md`

并更新：

- `CURRENT.md`
- `tasks/README.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
