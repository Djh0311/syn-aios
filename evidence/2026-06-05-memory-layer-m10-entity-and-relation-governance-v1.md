# Evidence：Memory Layer M10 Entity And Relation Governance v1

日期：2026-06-05

## 结论

M10 已完成。接受范围仅限实体和关系治理的受控最小闭环：

- 新增独立 `memory-entity-relations.v1.json` sidecar，带 revision、lock、备份、原子写和损坏 JSON 拒绝覆盖。
- 新增最小 `MemoryEntityRegistry`、entity candidate、entity merge candidate、relation candidate、confirmed relation 和 audit event 类型。
- `preview_memory_entity_relation_candidates` 是只读派生，不创建 sidecar。
- alias / dedupe / relation candidate 的确认或拒绝只写 M10 sidecar，不写正式记忆、候选、观察、lint 或 workflow state。
- LLM inferred 和 similarity hit 只能保持候选边界；确认逻辑拒绝把这些来源直接写为 confirmed relation。
- 已确认关系只用于解释任务包召回原因，不改变 task memory packet included / excluded 选择。
- 关系解释进入 task memory packet 前会检查状态、来源和模型外发策略；secret 来源不会被导出到任务包解释。
- 记忆中心复用现有 `记忆` 入口展示实体候选、dedupe 候选、关系候选和已确认关系摘要，不新增一级入口、右侧顶级入口或项目页 tab。

不接受为：

- M11 维护任务完成。
- M12 成熟模式、跨项目记忆或完整验收完成。
- M13 中间版本记忆系统最终验收完成。
- 向量库、图数据库、GraphRAG 或完整图谱编辑完成。
- 实体自行合并、因果关系自行确认或 LLM 自动写正式关系完成。
- 真实 worker / Codex 已执行。

## 主要改动

后端：

- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
  - 新增 entity / relation governance 类型、preview / record input/output、task packet relation explanation 类型。
  - `TaskMemoryPacketItem` 新增 `relation_explanations`，旧 snapshot 兼容使用 serde default。
- `prototypes/productized-desktop-shell/src-tauri/src/memory_entity_relation_store.rs`
  - 新增 M10 sidecar store。
  - 支持空 store 只读加载、revision、lock、备份、原子写、损坏 JSON 拒绝覆盖和摘要。
- `prototypes/productized-desktop-shell/src-tauri/src/memory_entity_relation_governance.rs`
  - 新增候选派生、alias decision、merge decision、relation decision。
  - 从 formal memory、candidate、observation、knowledge doc / source refs 等材料派生实体和关系候选。
  - 确认关系前校验 actor、expected revision、确认权、来源边界和关系类型边界。
- `prototypes/productized-desktop-shell/src-tauri/src/task_memory_packet_builder.rs`
  - 读取 M10 sidecar，为 included formal memory 增加已确认关系解释。
  - 关系解释不改变 included / excluded 选择。
- `prototypes/productized-desktop-shell/src-tauri/src/task_memory_injection.rs`
  - task package snapshot / markdown / stale check 增加 entity relation store revision 和 relation explanations。
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
  - 新增 Tauri commands：
    - `load_memory_entity_relation_store`
    - `preview_memory_entity_relation_candidates`
    - `record_memory_entity_alias_decision`
    - `record_memory_entity_merge_decision`
    - `record_memory_relation_candidate_decision`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
  - 注册 M10 模块、Tauri handlers 和 Rust 单测。

前端：

- `prototypes/productized-desktop-shell/src/lib/types.ts`
  - 新增 M10 TS 类型、pending action 字段和 `relation_explanations`。
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`
  - 新增 M10 Tauri wrapper。
- `prototypes/productized-desktop-shell/src/App.tsx`
  - 加载 M10 sidecar、处理 preview 和 record action，成功后刷新 stores / snapshot。
- `prototypes/productized-desktop-shell/src/lib/memoryCenter.ts`
  - 新增实体 / 关系治理摘要。
- `prototypes/productized-desktop-shell/src/views/MemoryCenterView.tsx`
  - 在记忆中心原有页面内新增 M10 摘要、候选预览、确认动作和已确认关系展示。
- `prototypes/productized-desktop-shell/src/components/PermissionDialog.tsx`
  - 新增 alias / merge / relation candidate 决策详情和边界提示。
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
  - 新增 M10 离线 fixture、记忆中心展示和确认弹层断言。

文档：

- `CURRENT.md`
- `README.md`
- `tasks/README.md`
- `tasks/2026-06-05-memory-layer-m10-entity-and-relation-governance-v1.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `docs/plans/memory-layer-implementation-slice-v1.md`

## 场景验收

- alias / dedupe candidate：`memory_entity_relation_preview_suggests_alias_and_similarity_candidates_readonly` 验证同一工具别名可派生 entity candidate，similarity hit 只保持候选，preview 不创建 sidecar。
- LLM inferred causal relation：`memory_entity_relation_llm_inferred_causal_relation_stays_candidate` 验证 LLM inferred 因果关系只进入候选，尝试确认会被拒绝，且不会创建 sidecar。
- confirmed causal relation：`memory_entity_relation_confirmed_causal_relation_explains_task_packet` 验证手动来源因果关系经确认后进入 confirmed relation，并能为 task memory packet 生成召回解释。
- secret relation source：`memory_entity_relation_secret_relation_source_is_not_exported_to_task_packet` 验证来源权限不允许时，已确认关系也不会进入任务包解释。
- damaged JSON / revision：`memory_entity_relation_damaged_json_and_revision_conflict_are_rejected` 验证 expected revision 不匹配拒绝写入，损坏 JSON 不被覆盖。
- task memory packet：`task_memory_packet` 测试组通过，确认 M10 关系解释没有破坏 active formal memory、candidate、observation、lint、权限、模型外发和 token 过滤边界。
- 前端离线交互：`test:offline-interaction` 9 scenarios passed，覆盖记忆中心 M10 摘要、预览按钮、候选区、已确认关系区和关系候选确认弹层。

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
built in 707ms
```

说明：`npm run build` 保留 Vite 既有 chunk size warning，不影响构建通过。

```text
cargo test --lib memory_entity_relation
5 passed; 0 failed
```

```text
cargo test --lib task_memory_packet
10 passed; 0 failed
```

```text
cargo test --lib formal_memory
27 passed; 0 failed
```

```text
cargo test --lib memory_lint
9 passed; 0 failed
```

```text
cargo test --lib
214 passed; 0 failed; 1 ignored
```

说明：Rust 测试保留既有 `JsonRpcError::invalid_params` dead_code warning。

```text
rustfmt --check src/memory_entity_relation_store.rs src/memory_entity_relation_governance.rs src/task_memory_packet_builder.rs src/task_memory_injection.rs src/formal_memory_store.rs src/memory_lint_store.rs src/control_core.rs src/commands.rs src/types.rs src/lib.rs
```

最终结果：通过。

过程说明：第一次 `rustfmt --check` 在 M10 新增和相关文件上报格式差异；已运行 `rustfmt` 修正，随后重跑 `rustfmt --check`、`cargo test --lib task_memory_packet` 和 `cargo test --lib` 均通过。

## UI / 文案边界

- UI 只复用既有 `记忆` 入口，不新增一级入口、右侧顶级入口或项目页 tab。
- 记忆中心显示实体候选、dedupe 候选、关系候选和已确认关系摘要；不显示 raw graph、完整 sidecar 或完整 audit log。
- 离线测试覆盖 M10 摘要文案、预览按钮、候选展示和确认弹层。
- 禁止文案扫描已执行；产品前端 `src`、本轮同步过的权威文档、M10 evidence 和 M10 handoff 对任务包列出的 M10 禁用短语无命中。
- 测试文件内保留部分禁止词作为负向断言，未纳入 UI / docs 正向文案扫描。

真实窗口 / 截图验收：

- 真实窗口 / 截图验收未完成。
- 本轮未启动 Tauri 真机窗口，也未做浏览器截图验收；因此不能声称 M10 UI 已完成真实窗口验收。

## 边界

- 未执行真实 worker。
- 未执行真实 Codex。
- 未执行 `codex exec` 或 `codex exec resume`。
- 未读写 `/Users/yoyi/.codex`。
- 未接向量库。
- 未接图数据库。
- 未做 GraphRAG。
- 未写 `formal-memories.v1.json`。
- 未写 `memory-candidates.v1.json`。
- 未写 `observations.v1.json`。
- 未写 `memory-lint.v1.json`。
- 未写 `workflow-state.v0.json`。
- 未迁移数据库。

## 后续

- 下一步进入 M11：维护任务和记忆 lint 的任务包拆分。
- M10 真实窗口 / 截图验收仍是缺口，可在阶段 G 或专门 UI 验收任务中补。
