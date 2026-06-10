# Evidence：Memory Layer M11 Maintenance Jobs And Memory Lint v1

日期：2026-06-05

## 结论

M11 已完成。接受范围仅限维护任务和记忆 lint 的受控最小闭环：

- 复用 `memory-lint.v1.json` sidecar，新增 maintenance run、maintenance report、check summary、recommendation、index status 和 `maintenance_reports[]`。
- 新增 `maintenance_run` intent 和维护 finding 类型：`entity_drift`、`relation_source_revoked`、`sensitive_export_risk`、`private_source_risk`、`derived_index_stale`、`mature_pattern_signal`。
- 维护任务覆盖过期 / stale、缺来源、重复 / 冲突、权限撤回、关系来源撤回、私密 / 外发风险、实体漂移、派生索引状态和成熟模式信号。
- open blocking finding 继续阻止相关正式记忆进入 task memory packet；needs_review / info 只进入摘要和人工复核材料。
- 记忆中心复用既有 `记忆` 入口显示维护摘要、最近 run、check summary、recommendation 和运行入口，不新增一级入口、右侧顶级入口或项目页 tab。
- 维护任务只写 lint / maintenance sidecar，不自动修改正式记忆、候选、observation、实体关系或 workflow state。

不接受为：

- M12 成熟模式、跨项目记忆或完整验收完成。
- M13 中间版本记忆系统最终验收完成。
- 自动修复、自动合并、自动废弃、自动冻结、自动归档或自动删除正式记忆完成。
- mature pattern 自动成为正式记忆、技能或全局规则完成。
- 向量库、图数据库、GraphRAG、自动索引重建系统或完整运维后台完成。
- 真实 worker / Codex 已执行。

## 主要改动

后端：

- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
  - 扩展 `MemoryLintFindingType`、`MemoryLintRunIntent`、`MemoryLintRunRecord` 和 `MemoryLintStoreV1`。
  - 新增 `MemoryMaintenanceReport`、`MemoryMaintenanceCheckSummary`、`MemoryMaintenanceRecommendation` 和 `MemoryMaintenanceIndexStatus`。
- `prototypes/productized-desktop-shell/src-tauri/src/memory_lint_engine.rs`
  - 新增 maintenance report builder 和维护检查。
  - 从 formal memory、candidate、observation、M10 entity / relation store 派生维护 finding。
- `prototypes/productized-desktop-shell/src-tauri/src/memory_lint_store.rs`
  - 复用 `memory-lint.v1.json`，写入 run record、report summary、revision、备份和原子写。
  - 继续拒绝损坏 JSON 覆盖和 expected revision 冲突写入。
- `prototypes/productized-desktop-shell/src-tauri/src/control_core.rs`
  - 允许受控 `maintenance_run` intent。
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
  - 注册 M11 测试，验证维护任务只读正式记忆 / 候选 / 实体关系边界。

前端：

- `prototypes/productized-desktop-shell/src/lib/types.ts`
  - 新增 M11 TS 类型和 pending action 字段。
- `prototypes/productized-desktop-shell/src/lib/candidateGovernance.ts`
  - 摘要读模型读取最近 maintenance report。
- `prototypes/productized-desktop-shell/src/lib/memoryCenter.ts`
  - 新增维护摘要读模型。
- `prototypes/productized-desktop-shell/src/views/MemoryCenterView.tsx`
  - 在既有 `记忆` 页面新增维护任务卡片和运行动作。
- `prototypes/productized-desktop-shell/src/components/PermissionDialog.tsx`
  - 新增 `run-memory-maintenance` 确认弹层，明确不会自动修改正式记忆或调用 lifecycle。
- `prototypes/productized-desktop-shell/src/App.tsx`
  - 执行 `runMemoryLint` 并刷新 stores / snapshot。
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
  - 新增 M11 离线 fixture、记忆中心展示和确认弹层断言。

文档：

- `CURRENT.md`
- `README.md`
- `tasks/README.md`
- `tasks/2026-06-05-memory-layer-m11-maintenance-jobs-and-memory-lint-v1.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `docs/plans/memory-layer-implementation-slice-v1.md`

## 场景验收

- maintenance run 写入 run record / report summary：`memory_maintenance_run_reports_source_secret_and_index_findings_readonly` 验证 `report_id`、`maintenance_reports[]` 和维护摘要。
- 不改正式记忆：同一测试对比 maintenance run 前后 formal records / versions；`memory_maintenance_run_reports_mature_pattern_signal_without_promoting_memory` 对比 formal store revision。
- 缺来源 / 私密 / 外发风险 / 索引状态：同一测试覆盖 `missing_source`、`sensitive_export_risk`、`private_source_risk` 和 `derived_index_stale` finding。
- 任务包 blocking guard：同一测试验证被 blocking finding 命中的正式记忆在 task memory packet 中以 `Conflicted` 排除。
- 实体漂移 / 关系来源撤回：`memory_maintenance_run_reports_entity_drift_and_relation_revoked_readonly` 验证 `entity_drift` 和 `relation_source_revoked` finding，且 M10 entity relation store 不被修改。
- mature pattern signal：`memory_maintenance_run_reports_mature_pattern_signal_without_promoting_memory` 验证只生成 `mature_pattern_signal` needs_review finding，不自动提升为正式记忆或规则。
- 损坏 JSON / revision：`memory_lint_damaged_json_is_rejected_without_overwrite` 和 `memory_lint_revision_conflict_is_rejected` 继续覆盖 sidecar 安全边界。
- 前端离线交互：`test:offline-interaction` 覆盖记忆中心维护卡片、维护运行按钮、确认弹层和禁止文案。

## 验证

通过：

```text
npm run typecheck
```

```text
npm run test:offline-interaction
offline interaction tests passed: 9
```

```text
npm run build
```

说明：`npm run build` 保留既有 Vite chunk size warning，不影响构建通过。

```text
cargo test --lib memory_lint
9 passed; 0 failed
```

```text
cargo test --lib memory_maintenance
3 passed; 0 failed
```

```text
cargo test --lib task_memory_packet
10 passed; 0 failed
```

```text
cargo test --lib
217 passed; 0 failed; 1 ignored
```

说明：Rust 测试保留既有 `JsonRpcError::invalid_params` dead_code warning。

```text
rustfmt --check src/memory_lint_engine.rs src/memory_lint_store.rs src/control_core.rs src/types.rs src/lib.rs
```

最终结果：通过。

## UI / 文案边界

- UI 只复用既有 `记忆` 入口，不新增一级入口、右侧顶级入口或项目页 tab。
- 记忆中心显示维护摘要、最近 run、check summary、recommendation 和运行入口；不显示 raw sidecar、完整 raw audit 或完整索引日志。
- 确认弹层明确 `maintenance_run` 只写 `memory-lint.v1.json` 的 maintenance run / findings / report。
- 离线测试覆盖允许文案：`维护任务只生成 finding`、`blocking finding 会阻止召回`、`不会自动修改正式记忆`。
- 离线测试覆盖禁止文案：`自动清理记忆`、`自动修复记忆`、`自动合并重复记忆`、`维护任务已改正式记忆`。

真实窗口 / 截图验收：

- 普通 in-app Browser smoke 已完成：打开 `http://127.0.0.1:5178/`，进入既有 `记忆` 入口，确认维护任务卡片、`维护任务只生成 finding`、`blocking finding 会阻止召回`、`不会自动修改正式记忆` 可见。
- 普通浏览器 smoke 中产品页面 console error / warn 为空；普通浏览器仍提示当前页面不在 Tauri 窗口中运行，这是预期数据桥限制。
- PNG 截图落盘失败：Browser runtime 对写入 `evidence/2026-06-05-memory-layer-m11-memory-center-maintenance-smoke.png` 返回 `EPERM`。
- 真实 Tauri 数据桥窗口验收未完成，因此不能声称 M11 UI 已完成真实 Tauri 验收。

## 边界

- 未执行真实 worker。
- 未执行真实 Codex。
- 未执行 `codex exec` 或 `codex exec resume`。
- 为执行 in-app Browser smoke，按 Browser 插件要求读取了 `/Users/yoyi/.codex/plugins/cache/openai-bundled/browser/26.602.30954/skills/control-in-app-browser/SKILL.md`；未读取用户 Codex 会话数据，未写 `/Users/yoyi/.codex`。
- 未自动修改正式记忆。
- 未自动调用 M9 lifecycle。
- 未自动合并实体、关系或正式记忆。
- 未让 mature pattern signal 自动成为正式记忆、全局记忆、技能或规则。
- 未接向量库。
- 未接图数据库。
- 未做 GraphRAG。
- 未写 `formal-memories.v1.json`。
- 未写 `memory-candidates.v1.json`。
- 未写 `observations.v1.json`。
- 未写 `memory-entity-relations.v1.json`。
- 未写 `workflow-state.v0.json`。
- 未迁移数据库。
- 本地 Vite smoke 使用的 `127.0.0.1:5178` dev server 已关闭。

## 后续

- 下一步进入 M12：成熟模式、跨项目记忆和完整验收。
- M11 真实 Tauri 数据桥验收仍是缺口，可在阶段 G 或专门 UI 验收任务中补。
