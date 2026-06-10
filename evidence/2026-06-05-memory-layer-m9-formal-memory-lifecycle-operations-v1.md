# Evidence：Memory Layer M9 Formal Memory Lifecycle Operations v1

日期：2026-06-05

## 结论

M9 已完成。接受范围仅限正式记忆生命周期操作的受控最小链路：

- 后端新增 lifecycle preview / record wrapper，所有写入只通过 `formal-memories.v1.json` 的正式记忆 store 原子写入路径完成。
- 支持 `revise`、`deprecate`、`freeze`、`unfreeze`、`archive`、`merge`、`split`、`promote_to_global`、`demote_to_project`。
- 每次成功操作都会生成新的 `MemoryVersion`、`MemoryAuditEvent` 和 affected record 的 `MemoryAuditRef`，并更新 store revision / updated_at。
- 校验 `project_root` 上下文绑定、expected store revision、expected record version、actor role、reason、required approval 和 confirmation summary；高影响操作必须由 `confirmed_by: "user"` 明确确认，项目主管确认不能替代用户确认。
- 冻结后的正式记忆不能普通编辑；解冻后才能恢复 active 评估。
- 废弃 / 冻结 / 归档 / 被替代的非 active 正式记忆默认不进任务包 included list。
- merge / split 只处理明确选择的正式记忆和草稿，不做语义推断、实体 dedupe 或关系治理。
- 记忆中心新增 lifecycle 操作区和确认弹层预览，展示影响面、确认权、任务包影响、old/new version 和版本 / 审计提示。

不接受为：

- M10 关系 / 实体治理完成。
- M11 维护任务完成。
- M12 成熟模式、跨项目记忆和完整验收完成。
- M13 中间版本记忆系统最终验收完成。
- 自动 dedupe、图谱推断、因果关系确认或语义合并完成。
- Obsidian / 知识库可直接写正式记忆。
- 真实 worker / Codex 已执行。

## 主要改动

后端：

- `prototypes/productized-desktop-shell/src-tauri/src/formal_memory_lifecycle.rs`
  - 新增 lifecycle preview / record 逻辑。
  - 覆盖 revise / deprecate / freeze / unfreeze / archive / merge / split / promote / demote。
  - 写入前校验上下文、版本冲突、确认权和损坏 JSON。
  - 写入时生成 version / audit / audit ref，并通过 formal store 原子写入。
- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
  - 新增 lifecycle operation、preview input、record input、impact summary、required approval、merge / split / scope change plan、output 类型。
- `prototypes/productized-desktop-shell/src-tauri/src/formal_memory_store.rs`
  - 暴露 sidecar path、lock path、atomic write 和 store lock helper 给 lifecycle wrapper 复用。
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
  - 新增 `preview_formal_memory_lifecycle_operation`。
  - 新增 `record_formal_memory_lifecycle_operation`。
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
  - 注册 `formal_memory_lifecycle` 模块和 Tauri commands。

前端：

- `prototypes/productized-desktop-shell/src/lib/types.ts`
  - 新增 lifecycle TS 类型和 `record-formal-memory-lifecycle-operation` pending action。
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`
  - 新增 `previewFormalMemoryLifecycleOperation` 和 `recordFormalMemoryLifecycleOperation` wrapper。
- `prototypes/productized-desktop-shell/src/views/MemoryCenterView.tsx`
  - 在正式记忆详情区新增 lifecycle 操作按钮。
  - 每个动作先调用后端 preview，再打开确认弹层。
- `prototypes/productized-desktop-shell/src/components/PermissionDialog.tsx`
  - 展示 lifecycle 操作、写入位置、目标记忆、确认权、影响面、任务包影响、old/new version 和边界。
- `prototypes/productized-desktop-shell/src/App.tsx`
  - 确认后调用 `recordFormalMemoryLifecycleOperation` 并重新加载 stores / snapshot。
- `prototypes/productized-desktop-shell/src/lib/memoryCenter.ts`
  - 任务包 eligibility 继续基于 active / lint / scope 判断，非 active 正式记忆排除。
- `prototypes/productized-desktop-shell/src/styles.css`
  - 新增 lifecycle 操作区样式。
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
  - 新增记忆中心 lifecycle 操作按钮、确认弹层和禁止文案断言。

文档：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `docs/plans/memory-layer-implementation-slice-v1.md`

## lifecycle 场景验收

- revise：`formal_memory_lifecycle_revise_creates_new_version_and_audit` 验证 record version 从 v1 到 v2、生成 `manual_revision` version、追加 audit event，旧版本未覆盖。
- freeze / unfreeze：`formal_memory_lifecycle_freeze_blocks_revise_until_unfreeze` 验证冻结后普通 revise 被拒绝，解冻后恢复 `memory_active`。
- deprecate / archive：`formal_memory_lifecycle_deprecate_and_archive_exclude_from_task_packet` 验证状态变为 `memory_deprecated` / `memory_archived`，预览显示非 active 默认不进任务包。
- merge：`formal_memory_lifecycle_merge_is_explicit_and_versions_sources` 验证必须显式选择来源，创建新 active 记录并把来源转为 deprecated，生成多条 version，warning 标记 no semantic dedupe。
- split：`formal_memory_lifecycle_split_keeps_source_and_creates_children` 验证显式拆分草稿创建两条新 active 记录，并把来源记录转为 deprecated。
- promote / demote：`formal_memory_lifecycle_promote_and_demote_require_confirmation` 验证上升 global 缺确认会拒绝，只有项目主管确认也会拒绝，用户确认后才可上升 global / 下沉 project。
- revision / damaged JSON：`formal_memory_lifecycle_rejects_revision_conflict_and_damaged_json` 验证 expected revision 不匹配拒绝写入，损坏 JSON 不被覆盖。
- task memory packet：`task_memory_packet_excludes_inactive_formal_memories` 验证 inactive 正式记忆不会进入 included list。

## 验证

通过：

```text
npm run typecheck
tsc --noEmit
```

```text
npm run test:offline-interaction
offline interaction tests passed: 9
```

```text
npm run build
tsc --noEmit && vite build
215 modules transformed
built in 611ms
```

说明：`npm run build` 保留 Vite 既有 chunk size warning，不影响构建通过。

```text
cargo test --lib formal_memory_lifecycle
7 passed; 0 failed
```

```text
cargo test --lib formal_memory
27 passed; 0 failed
```

```text
cargo test --lib task_memory_packet
10 passed; 0 failed
```

```text
cargo test --lib memory_lint
9 passed; 0 failed
```

```text
cargo test --lib memory_candidate
9 passed; 0 failed
```

```text
cargo test --lib
209 passed; 0 failed; 1 ignored
```

说明：Rust 测试保留既有 `JsonRpcError::invalid_params` dead_code warning。

```text
rustfmt --check src/formal_memory_lifecycle.rs src/formal_memory_store.rs src/task_memory_packet_builder.rs src/task_memory_injection.rs src/memory_lint_store.rs src/memory_lint_engine.rs src/control_core.rs src/commands.rs src/types.rs src/lib.rs
```

最终结果：通过。

过程说明：第一次 `rustfmt --check` 只在新增 `formal_memory_lifecycle.rs` 上报格式差异；已运行 `rustfmt src/formal_memory_lifecycle.rs`，随后重跑 `rustfmt --check` 和 `cargo test --lib formal_memory_lifecycle` / `cargo test --lib` 均通过。

## UI / 文案边界

- 离线测试覆盖记忆中心 lifecycle 操作按钮：`编辑提案`、`废弃`、`冻结`、`解冻`、`归档`、`合并`、`拆分`、`上升为全局`、`下沉为项目`。
- 离线测试覆盖确认弹层：`formal-memories.v1.json`、确认权、影响面、任务包影响、`old v1 / new v2`、版本和审计提示。
- 禁止文案扫描已执行；产品前端 `src`、本轮同步过的权威文档、M9 evidence 和 M9 handoff 对禁用短语无命中。
- 测试文件内保留部分禁止词作为负向断言，未纳入 UI / docs 正向文案扫描。

```text
rg -n -F <M9 禁用短语列表> <frontend src + authority docs + M9 evidence/handoff>
```

结果：无命中。

真实窗口 / 截图验收：

- 未完成。
- 本轮没有暴露可直接使用的 in-app Browser 控制工具；未读取 `/Users/yoyi/.codex` 下插件技能文件来绕开任务边界。
- 未启动 Tauri 真机窗口截图验收；因此不能声称 M9 UI 已完成真实窗口验收。

## 边界

- 未执行真实 worker。
- 未执行真实 Codex。
- 未执行 `codex exec` 或 `codex exec resume`。
- 未读写 `/Users/yoyi/.codex`。
- M9 默认只写 `formal-memories.v1.json`；不写 `memory-candidates.v1.json`、`observations.v1.json`、`memory-lint.v1.json` 或 `workflow-state.v0.json`。
- 未执行 Obsidian CLI。
- 未自动扫描 vault。
- 未做语义 dedupe、实体合并、关系治理、维护任务自动修改或成熟模式跨项目提升。

## 后续

- 下一步进入 M10：关系 / 实体治理。
- M9 真实窗口 / 截图验收仍是缺口，可在阶段 G 或专门 UI 验收任务中补。
