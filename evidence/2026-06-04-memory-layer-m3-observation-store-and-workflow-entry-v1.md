# Memory Layer M3 Observation Store And Workflow Entry v1 Evidence

日期：2026-06-04

任务包：`tasks/2026-06-04-memory-layer-m3-observation-store-and-workflow-entry-v1.md`

## 结论

M3 已完成，接受为 ObservationStore 和工作流观察入口完成。

本轮新增独立 observation sidecar：`observations.v1.json`。工作流观察可以受控记录为 `ObservationRecord`；`recorded` observation 可由 `project_director` 生成 `MemoryCandidate`，候选初始状态仍为 `candidate_needs_review`；生成后 observation 状态变为 `candidate_created` 并记录 `candidate_key`。

不接受为正式记忆生命周期完成、任务包记忆注入完成、任务包召回完成、Obsidian / 向量库 / 图数据库完成或中间版本记忆层完成。

## 实现证据

后端：

- 新增 `prototypes/productized-desktop-shell/src-tauri/src/observation_store.rs`
- 新增 Rust 类型：`ObservationStoreV1`、`ObservationRecord`、`ObservationSourceRef`、`ObservationAuditRef`、`ObservationStatus`、`CreateObservationInput`、`CreateObservationOutput`、`CreateMemoryCandidateFromObservationInput`、`CreateMemoryCandidateFromObservationOutput`
- 新增控制核心 guard：`validate_observation_create`、`validate_observation_candidate_creation`
- 新增 Tauri 命令：`load_observation_store`、`create_observation`、`create_memory_candidate_from_observation`
- 新增 bridge：`create_observation_at`、`create_memory_candidate_from_observation_at`、observation context binding guard

前端：

- 新增 observation TS 类型和 Tauri wrapper
- `candidateGovernance.ts` 新增 `summarizeObservationStore`
- 项目工作流侧栏候选治理卡展示 observation sidecar、revision、状态计数、最近 audit 和最近 `candidate_key`
- UI 文案明确：观察可生成候选、候选仍需确认 / 采纳、observation 不是正式记忆

## 验证

通过：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
cargo test --lib observation
cargo test --lib memory_candidate
cargo test --lib formal_memory
cargo test --lib
rustfmt --check src/observation_store.rs src/memory_candidate_store.rs src/control_core.rs src/commands.rs src/types.rs
```

结果摘要：

- `cargo test --lib observation`：9 passed
- `cargo test --lib memory_candidate`：9 passed
- `cargo test --lib formal_memory`：15 passed
- `cargo test --lib`：145 passed, 1 ignored
- `npm run test:offline-interaction`：passed
- `npm run build`：passed

已知非阻塞提示：

- Rust 仍有既有 warning：`JsonRpcError::invalid_params` 未使用。
- Vite build 仍有既有 chunk > 500 kB warning。
- 本轮涉及项目页局部 UI 展示，已通过离线交互测试覆盖文案和边界；真实窗口 / 截图验收未完成。

## 明确未做

- 未执行真实 Codex。
- 未执行 `codex exec` / `codex exec resume`。
- 未读写 `/Users/yoyi/.codex`。
- 未修改 `workflow-state.v0.json` 结构。
- 未迁移数据库。
- 未把 observation 直接写成正式记忆。
- 未从 observation 直接采纳为正式记忆。
- 未把 observation 注入 worker 任务包。
- 未接 Obsidian、向量库、图数据库或知识库扫描。
