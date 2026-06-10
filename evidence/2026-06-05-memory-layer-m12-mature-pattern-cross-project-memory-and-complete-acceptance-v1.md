# Evidence：Memory Layer M12 Mature Pattern Cross Project Memory And Complete Acceptance v1

日期：2026-06-05

## 结论

M12 已完成。接受范围仅限成熟模式候选、跨项目主题报告、用户确认后受控写正式 mature pattern 记忆、任务包召回边界和 M1-M12 gate 摘要：

- 新增 `memory-patterns.v1.json` sidecar，对 mature pattern candidates、cluster reports 和 audit events 使用 revision、lock、备份、原子写和 damaged JSON 拒绝覆盖。
- 新增 deterministic `MaturePatternCandidate` 派生，来源包括 M11 mature pattern signal、重复 confirmed memory candidates、重复 observations、M10 confirmed relation 和 formal memories 主题线索。
- 新增 `MemoryClusterReport` 作为跨项目主题报告，可下钻 member refs 和 source refs；报告不是正式事实，也不进入 task memory packet included list。
- 新增 `record_mature_pattern_decision`：confirm / reject / quarantine / request changes 都写 M12 audit；只有 `actor_role: "user"` 且 `confirmed_by: "user"` 的 confirm 才能通过正式记忆 store 写 formal mature pattern memory，并生成 record / version / audit / source refs。
- `control_core` 的 task memory packet guard 允许已确认且权限允许的 global formal mature pattern memory 被评估召回；未确认 candidate、cluster report、maintenance report、relation candidate、observation、knowledge hit 或 LLM summary 仍不能进入 included list。
- 记忆中心复用既有 `记忆` 入口，显示成熟模式候选、跨项目主题报告、用户确认状态和 M1-M12 gate 摘要；不新增一级入口、右侧顶级入口或项目页 tab。

不接受为：

- M13 最终权威验收完成。
- 最终蓝图完整记忆系统完成。
- 自动技能化或自动全局规则完成。
- 跨项目摘要直接影响 worker。
- 向量库、图数据库、GraphRAG、自动索引重建系统或完整理解地图完成。
- 真实 worker / Codex 已执行。

## 主要改动

后端：

- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
  - 新增 M12 类型：`MaturePatternCandidate`、`MemoryClusterReport`、`MemoryPatternStoreV1`、`MemorySystemAcceptanceSummary`、preview / decision input-output 等。
- `prototypes/productized-desktop-shell/src-tauri/src/mature_pattern_store.rs`
  - 新增 `memory-patterns.v1.json` store wrapper，支持 revision、lock、backup、atomic write 和 damaged JSON 拒绝覆盖。
- `prototypes/productized-desktop-shell/src-tauri/src/mature_pattern_governance.rs`
  - 新增 preview / decision / acceptance summary 逻辑。
  - 用户确认正式化时调用 formal memory store 创建正式 mature pattern memory。
- `prototypes/productized-desktop-shell/src-tauri/src/control_core.rs`
  - 允许 global scope 正式 mature pattern memory 进入 task packet scope 评估，同时保留 status / lint / model export / token guard。
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
  - 新增 `load_memory_pattern_store`、`preview_mature_patterns`、`record_mature_pattern_decision`。
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
  - 注册新模块、Tauri commands 和 M12 测试。

前端：

- `prototypes/productized-desktop-shell/src/lib/types.ts`
  - 新增 M12 TS 类型和 `record-mature-pattern-decision` pending action。
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`
  - 新增 M12 Tauri wrapper。
- `prototypes/productized-desktop-shell/src/lib/memoryCenter.ts`
  - 新增 mature pattern summary、cluster report summary、acceptance summary 汇总和 warning 合并。
- `prototypes/productized-desktop-shell/src/App.tsx`
  - 加载 M12 store，执行 mature pattern decision 后刷新 stores。
- `prototypes/productized-desktop-shell/src/views/MemoryCenterView.tsx`
  - 在既有 `记忆` 页面新增成熟模式 / 跨项目主题面板、preview 按钮、用户确认 / 拒绝 / 隔离 / 要求补来源动作。
- `prototypes/productized-desktop-shell/src/components/PermissionDialog.tsx`
  - 新增成熟模式决定确认弹层，明确 confirm 写 `memory-patterns.v1.json / formal-memories.v1.json`，其他决定只写 `memory-patterns.v1.json`。
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
  - 新增 M12 fixture、记忆中心展示断言、M12 summary 断言和确认 / 隔离弹层断言。

文档：

- `CURRENT.md`
- `README.md`
- `tasks/README.md`
- `tasks/2026-06-05-memory-layer-m12-mature-pattern-cross-project-memory-and-complete-acceptance-v1.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `docs/plans/memory-layer-implementation-slice-v1.md`

## 场景验收

- readonly preview：`mature_pattern_preview_derives_candidates_and_memory_cluster_reports_readonly` 验证 preview 能派生成熟模式候选和 cluster report，且不创建 sidecar。
- 未确认边界：`memory_cluster_report_and_unconfirmed_mature_pattern_do_not_enter_task_packet` 验证未确认 candidate 和 cluster report 不进入 task packet included list。
- 用户确认正式化：`mature_pattern_user_confirmation_writes_formal_memory_and_task_packet_can_recall` 验证只有用户确认才能写正式 mature pattern memory，并且权限允许时可被 task packet 召回。
- 拒绝 / 隔离 / revision / damaged JSON：`mature_pattern_reject_quarantine_revision_and_damaged_json_do_not_mutate_formal_memory` 验证非确认决定不改 formal store，revision 冲突和 damaged JSON 不被覆盖。
- M11 输入信号：`memory_maintenance_run_reports_mature_pattern_signal_without_promoting_memory` 验证 mature pattern signal 仍只是 finding，不会自动正式化。
- 前端离线交互：`test:offline-interaction` 覆盖记忆中心 M12 面板、按钮文案、确认弹层和隔离弹层。

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
cargo test --lib mature_pattern
5 passed; 0 failed
```

```text
cargo test --lib memory_cluster
2 passed; 0 failed
```

```text
cargo test --lib task_memory_packet
10 passed; 0 failed
```

```text
cargo test --lib formal_memory
29 passed; 0 failed
```

```text
cargo test --lib memory_lint
9 passed; 0 failed
```

```text
cargo test --lib memory_entity_relation
5 passed; 0 failed
```

```text
cargo test --lib
221 passed; 0 failed; 1 ignored
```

说明：Rust 测试保留既有 `JsonRpcError::invalid_params` dead_code warning。

```text
rustfmt --check src/mature_pattern_store.rs src/mature_pattern_governance.rs src/task_memory_packet_builder.rs src/task_memory_injection.rs src/formal_memory_store.rs src/formal_memory_lifecycle.rs src/memory_lint_store.rs src/memory_entity_relation_store.rs src/control_core.rs src/commands.rs src/types.rs src/lib.rs
```

最终结果：通过。

禁用文案扫描：

- 范围：`prototypes/productized-desktop-shell/src`、`README.md`、`CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`docs/plans`、`docs`、`evidence`、`handoffs`。
- 应用源码和当前权威文档未命中 M12 禁用文案。
- 命中项仅来自 M9-M11 既有 evidence / handoff 的负向边界记录；M12 本轮未新增这些禁用文案到 UI 或当前入口。

## UI / 文案边界

- 只复用既有 `记忆` 入口。
- 不新增一级入口、右侧顶级入口或项目页 tab。
- 记忆中心显示成熟模式候选、跨项目主题报告、用户确认状态、候选未确认边界和 M1-M12 gate 摘要。
- 不显示 raw sidecar、raw JSON、完整 audit、完整索引日志或数据库路径大表。
- Permission dialog 明确候选和跨项目主题报告未确认不进入任务包；只有用户确认正式化时才会通过正式记忆路径写 version、audit 和 source refs。

真实窗口 / 截图验收：

- 真实窗口 / 截图验收未完成。
- 原因：M12 任务包禁止读写 `/Users/yoyi/.codex`；本环境的 in-app Browser 技能说明位于 `/Users/yoyi/.codex/plugins/...`，本轮未为截图验收读取该路径。
- 不能声称 M12 UI 已完成真实 Tauri 数据桥或真实窗口截图验收。

## 边界

- 未执行真实 worker。
- 未执行真实 Codex。
- 未执行 `codex exec` 或 `codex exec resume`。
- 未读写 `/Users/yoyi/.codex`。
- 未扫描完整 transcript。
- 未自动技能化。
- 未让项目主管替代用户确认跨项目 / global / mature pattern 记忆。
- 未让 mature pattern candidate、cluster report、maintenance report、relation candidate、observation、knowledge hit、LLM summary 或 graph/index report 直接影响 worker。
- 未接向量库。
- 未接图数据库。
- 未做 GraphRAG。
- 未自动重建索引。
- 未写 `workflow-state.v0.json` 顶层结构。
- 未迁移数据库。

## 后续

- 下一步进入 M13：中间版本记忆系统最终权威验收和最终结论冻结。
- M12 真实窗口 / 截图验收仍是缺口，可在阶段 G 或专门 UI 验收任务中补。
