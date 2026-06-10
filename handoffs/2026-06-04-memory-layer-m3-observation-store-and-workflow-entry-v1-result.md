# Memory Layer M3 Observation Store And Workflow Entry v1 Handoff

日期：2026-06-04

## 本轮完成

- 新增 ObservationStore 独立 sidecar：`observations.v1.json`。
- 支持创建 `recorded` observation，并保存 source refs、摘要、状态、风险、敏感级别和 audit refs。
- 支持 `project_director` 从 `recorded` observation 生成 `MemoryCandidate`。
- 生成候选后 observation 更新为 `candidate_created`，并保存 `candidate_key` 回链。
- `ignored` / `quarantined` / 已生成候选的 observation 会拒绝再次生成 candidate。
- 普通聊天自动捕获会被拒绝；observation 创建必须带 source refs。
- observation 创建和生成候选均遵守 `project_root` / `project_id` / `workflow_id` / scope 绑定。

## 当前权威入口

- 任务包：`tasks/2026-06-04-memory-layer-m3-observation-store-and-workflow-entry-v1.md`
- Evidence：`evidence/2026-06-04-memory-layer-m3-observation-store-and-workflow-entry-v1.md`
- 后端 store：`prototypes/productized-desktop-shell/src-tauri/src/observation_store.rs`
- 前端读模型：`prototypes/productized-desktop-shell/src/lib/candidateGovernance.ts`
- 项目页入口：`prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`

## 验证结果

全部通过：

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

已知提示：

- Rust 既有 warning：`JsonRpcError::invalid_params` 未使用。
- Vite 既有 chunk > 500 kB warning。
- 本轮涉及项目页局部 UI 展示，已通过离线交互测试覆盖文案和边界；真实窗口 / 截图验收未完成。

## 边界

接受为：

- ObservationStore 和工作流观察入口完成。
- observation 可生成待审核 MemoryCandidate。

不接受为：

- observation 已成为正式记忆。
- 正式记忆生命周期完成。
- 任务包记忆召回完成。
- 任务包记忆注入完成。
- 中间版本记忆层完成。
- Obsidian / 知识库 / 向量库 / 图数据库完成。

## 下一步建议

按记忆层切片继续 M4：任务记忆包生成器和预览。M4 仍必须禁止把 observation、candidate、知识库命中或聊天摘要伪装成正式记忆。
