# Handoff：Memory Layer M9 Formal Memory Lifecycle Operations v1

日期：2026-06-05

## 本轮结果

M9 已完成。正式记忆现在可以通过后端 lifecycle 状态机做受控版本化操作，并在记忆中心通过 preview + 确认弹层触发写入。

完成内容：

- 新增 `formal_memory_lifecycle.rs`。
- 新增 lifecycle 类型、merge / split / scope change plan、preview / output 类型。
- 新增 Tauri commands：
  - `preview_formal_memory_lifecycle_operation`
  - `record_formal_memory_lifecycle_operation`
- 前端新增 lifecycle Tauri wrapper、pending action、记忆中心操作按钮和确认弹层详情。
- `revise`、`deprecate`、`freeze`、`unfreeze`、`archive`、`merge`、`split`、`promote_to_global`、`demote_to_project` 均有最小受控实现和测试。
- 权威入口已更新到 M9 完成，下一步指向 M10 关系 / 实体治理。

## 接受范围

只接受为：

- 正式记忆生命周期操作完成。
- 正式记忆编辑会创建新 `MemoryVersion`，不覆盖旧版本。
- 废弃 / 冻结 / 归档 / 被替代正式记忆默认不进任务包 included list。
- merge / split 保留来源、旧 memory id、版本和审计，且只处理显式选择。
- 上升 / 下沉 scope 记录确认权、原因、适用范围、版本和审计。
- 高影响 lifecycle 操作必须由 `confirmed_by: "user"` 明确确认，项目主管确认不能替代用户确认。
- 记忆中心已有最小 lifecycle 操作入口、影响面预览和确认弹层。

不接受为：

- M10 关系 / 实体治理完成。
- M11 维护任务完成。
- M12 成熟模式、跨项目记忆或完整验收完成。
- M13 中间版本记忆系统最终验收完成。
- 自动 dedupe、自动语义合并、图谱推断或因果关系确认完成。
- Obsidian / 知识库 / Markdown 可直接写正式记忆。
- 真实 worker / Codex 已执行。

## 验证

通过：

- `npm run typecheck`
- `npm run test:offline-interaction`，9 scenarios passed
- `npm run build`
- `cargo test --lib formal_memory_lifecycle`，7 passed
- `cargo test --lib formal_memory`，27 passed
- `cargo test --lib task_memory_packet`，10 passed
- `cargo test --lib memory_lint`，9 passed
- `cargo test --lib memory_candidate`，9 passed
- `cargo test --lib`，209 passed / 1 ignored
- `rustfmt --check src/formal_memory_lifecycle.rs src/formal_memory_store.rs src/task_memory_packet_builder.rs src/task_memory_injection.rs src/memory_lint_store.rs src/memory_lint_engine.rs src/control_core.rs src/commands.rs src/types.rs src/lib.rs`
- 禁止文案扫描通过：产品前端 `src`、本轮同步过的权威文档、M9 evidence 和 M9 handoff 对 M9 禁用短语无命中。

说明：

- 首次 `rustfmt --check` 仅发现 `formal_memory_lifecycle.rs` 格式差异；已运行 `rustfmt src/formal_memory_lifecycle.rs` 后重跑通过。
- Rust 测试仍有既有 `JsonRpcError::invalid_params` dead_code warning。
- `npm run build` 仍有 Vite chunk size warning。

## 未完成 / 风险

- 真实窗口 / 截图验收未完成；不能声称 M9 UI 已完成真实 Tauri 或真实浏览器截图验收。
- 当前 `MemoryRecord` 仍只有单值 `supersedes_memory_id` / `superseded_by_memory_id`，M9 merge / split 暂用逗号串保存多来源关系；M10 应补关系 / 实体治理模型。
- M9 没有做语义 dedupe、实体合并、关系推断、维护任务或成熟模式跨项目提升。

## 当前权威入口

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `docs/plans/memory-layer-implementation-slice-v1.md`
- `evidence/2026-06-05-memory-layer-m9-formal-memory-lifecycle-operations-v1.md`

## 下一步

进入 M10：关系 / 实体治理。

M10 建议优先明确：

- `MemoryEntityRegistry` / alias / merge / dedupe 的最小模型。
- `MemoryRelation` / `MemoryRelationCandidate` 的正式与候选边界。
- LLM 推断关系默认只作为候选关系。
- 因果关系和跨项目关系的确认权。
- 关系进入任务包前如何经过状态、权限、来源、冲突和审计检查。

继续保持：

- 不执行真实 worker / Codex。
- 不读写 `/Users/yoyi/.codex`。
- 不让知识库、候选、observation、Markdown、Canvas、Graph、Bases 或 LLM 摘要绕过正式记忆状态机。
