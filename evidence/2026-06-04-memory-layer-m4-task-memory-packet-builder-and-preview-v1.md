# Evidence: Memory Layer M4 Task Memory Packet Builder And Preview v1

日期：2026-06-04

## 结论

M4 已完成为“任务记忆包生成器和预览”：

- 后端新增 `TaskMemoryPacketBuilder`，即时读取 `FormalMemoryStore`、`MemoryCandidateStore` 和 `ObservationStore` 生成 deterministic preview。
- included list 只允许来自 active 正式记忆。
- candidate / observation 只进入 excluded / review materials，并带明确 reason。
- 预览不写 store、不推进 workflow state、不执行 worker、不调用真实 Codex、不读写 `/Users/yoyi/.codex`。
- 项目工作流侧栏新增最小只读“任务记忆包预览”摘要。

不接受为：

- 任务包注入完成。
- worker 已收到记忆包。
- 自动化工作流已经使用记忆执行。
- 中间版本记忆层完成。

## 关键实现

后端：

- `prototypes/productized-desktop-shell/src-tauri/src/task_memory_packet_builder.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/control_core.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`

前端：

- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `prototypes/productized-desktop-shell/src/lib/candidateGovernance.ts`
- `prototypes/productized-desktop-shell/src/App.tsx`
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`

测试：

- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

## 覆盖规则

- `memory_active` 且 scope / 权限 / 相关性 / token 通过的正式记忆进入 included。
- `memory_conflicted` 或有 conflict refs 排除为 `conflicted`。
- `memory_deprecated` / `memory_frozen` / `memory_archived` / 过期排除为 `stale`。
- 非 active 状态排除为 `status_not_active`。
- 跨项目、跨 workflow 或 scope 不匹配排除为 `permission_blocked`。
- external model context 中 `model_export_policy = blocked` 排除为 `model_export_blocked`。
- 超过 `max_memory_items` 或 `max_estimated_tokens` 排除为 `token_limit`。
- 与任务目标无确定性关键词命中排除为 `not_relevant`。
- candidate 排除为 `candidate_unconfirmed`，只作为待审查材料。
- observation 排除为 `observation_not_formal_memory`，只作为待审查材料。

## 验证

已通过：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
cargo test --lib task_memory_packet
cargo test --lib formal_memory
cargo test --lib memory_candidate
cargo test --lib observation
cargo test --lib
rustfmt --check src/task_memory_packet_builder.rs src/formal_memory_store.rs src/memory_candidate_store.rs src/observation_store.rs src/control_core.rs src/commands.rs src/types.rs
```

结果摘要：

- `npm run test:offline-interaction`：offline interaction tests passed: 9。
- `cargo test --lib task_memory_packet`：10 passed。
- `cargo test --lib`：155 passed, 1 ignored。
- Rust 测试仍有既有 warning：`JsonRpcError::invalid_params` unused。
- `npm run build` 仍有既有 Vite chunk size warning。

## UI 边界

- 显示位置：项目工作流侧栏候选治理详情卡内。
- 显示内容：included / excluded / review materials / estimated tokens / warnings / excluded reason。
- 文案明确：`预览未注入任务包`、`仅 active 正式记忆可入选`、`候选 / 观察仅作为待审查材料`。
- 未新增一级入口、右侧入口或画布主区域面板。
- 未显示“系统已记住”“自动学习完成”“候选已进入任务包”“observation 已注入任务包”“worker 已收到记忆包”“任务包注入已完成”。

真实窗口 / 截图验收未完成。本轮完成了前端离线渲染测试和构建验证，但没有启动真实 Tauri 窗口采集截图。
