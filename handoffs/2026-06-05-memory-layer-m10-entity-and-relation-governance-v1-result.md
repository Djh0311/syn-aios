# Handoff：Memory Layer M10 Entity And Relation Governance v1

日期：2026-06-05

## 本轮结果

M10 已完成。记忆层现在有独立的实体 / 关系治理 sidecar、候选预览、受控确认动作、已确认关系存储，以及任务记忆包召回解释。

完成内容：

- 新增 `memory_entity_relation_store.rs`，sidecar 为 `memory-entity-relations.v1.json`。
- 新增 `memory_entity_relation_governance.rs`，提供 preview、alias decision、merge decision、relation decision。
- 新增 entity / relation / audit / preview / record / task explanation 类型。
- 新增 Tauri commands：
  - `load_memory_entity_relation_store`
  - `preview_memory_entity_relation_candidates`
  - `record_memory_entity_alias_decision`
  - `record_memory_entity_merge_decision`
  - `record_memory_relation_candidate_decision`
- 任务记忆包新增 `relation_explanations`，并在 artifact snapshot / markdown / stale check 里记录 M10 sidecar revision。
- 前端新增 M10 类型、Tauri wrapper、记忆中心原位 UI、pending action 处理和确认弹层详情。
- 权威入口已更新到 M10 完成，下一步指向 M11 维护任务和记忆 lint 的任务包拆分。

## 接受范围

只接受为：

- 最小 `MemoryEntityRegistry` 可用。
- alias / dedupe 候选可预览和受控记录决定。
- 关系候选可预览、拒绝、隔离或在允许条件下确认。
- 已确认关系可写入 M10 sidecar，并进入任务包召回解释。
- LLM inferred、similarity hit 和来源不足关系保持候选边界。
- 已确认关系只解释召回原因，不改变任务包 included list。
- 关系来源权限不允许时不会进入任务包解释。
- 记忆中心具备最小实体 / 关系治理入口，不新增顶级导航。

不接受为：

- M11 维护任务完成。
- M12 成熟模式、跨项目记忆或完整验收完成。
- M13 中间版本记忆系统最终验收完成。
- 向量库、图数据库、GraphRAG 或完整图谱编辑完成。
- 实体自行合并、因果关系自行确认或 LLM 自动写正式关系完成。
- 真实 worker / Codex 已执行。

## 验证

通过：

- `npm run typecheck`
- `npm run test:offline-interaction`，9 scenarios passed
- `npm run build`
- `cargo test --lib memory_entity_relation`，5 passed
- `cargo test --lib task_memory_packet`，10 passed
- `cargo test --lib formal_memory`，27 passed
- `cargo test --lib memory_lint`，9 passed
- `cargo test --lib`，214 passed / 1 ignored
- `rustfmt --check src/memory_entity_relation_store.rs src/memory_entity_relation_governance.rs src/task_memory_packet_builder.rs src/task_memory_injection.rs src/formal_memory_store.rs src/memory_lint_store.rs src/control_core.rs src/commands.rs src/types.rs src/lib.rs`
- 禁止文案扫描通过：产品前端 `src`、本轮同步过的权威文档、M10 evidence 和 M10 handoff 对任务包列出的 M10 禁用短语无命中。

说明：

- 首次 `rustfmt --check` 发现新增 M10 相关 Rust 文件格式差异；已运行 `rustfmt` 后重跑通过。
- Rust 测试仍有既有 `JsonRpcError::invalid_params` dead_code warning。
- `npm run build` 仍有 Vite chunk size warning。

## 未完成 / 风险

- 真实窗口 / 截图验收未完成；不能声称 M10 UI 已完成真实 Tauri 或真实浏览器截图验收。
- M10 sidecar 是 JSON 最小实现，后续如果迁移 SQLite / FTS / 索引，需要保持 store wrapper、revision、审计和损坏 JSON 保护语义。
- M10 关系解释只是召回解释层，不是 task memory packet 选择器；后续不要在未拆任务包时让关系候选改变 worker 输入。
- M11 需要在 M5 最小 lint 之上补维护任务：过期、缺来源、重复 / 实体漂移、权限撤回、私密扫描、索引状态和维护报告，但维护任务仍不能直接改正式记忆。

## 当前权威入口

- `CURRENT.md`
- `README.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `docs/plans/memory-layer-implementation-slice-v1.md`
- `tasks/2026-06-05-memory-layer-m10-entity-and-relation-governance-v1.md`
- `evidence/2026-06-05-memory-layer-m10-entity-and-relation-governance-v1.md`

## 下一步

进入 M11：维护任务和记忆 lint。

M11 建议优先明确：

- 维护任务只生成 finding、提醒、候选、隔离建议或审计，不自动改正式记忆。
- 过期、缺来源、重复 / 实体漂移、权限撤回和私密扫描如何与 M5 finding、M10 entity / relation sidecar 协作。
- 维护报告如何显示在记忆中心或管理入口，且不新增无授权顶级入口。
- 哪些维护发现会阻断任务包召回，哪些只进入 review materials。

继续保持：

- 不执行真实 worker / Codex。
- 不读写 `/Users/yoyi/.codex`。
- 不让 knowledge doc、candidate、observation、relation candidate、LLM summary 或 graph/index 报告绕过正式记忆状态机。
