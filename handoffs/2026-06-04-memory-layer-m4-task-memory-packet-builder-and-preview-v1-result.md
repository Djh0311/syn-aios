# Handoff: Memory Layer M4 Task Memory Packet Builder And Preview v1

日期：2026-06-04

## 回收结论

M4 已完成，接受为“任务记忆包生成器和预览”完成。

不接受为：

- 任务包注入完成。
- worker 已收到记忆包。
- 记忆召回已经参与真实 worker 执行。
- 中间版本记忆层完成。

## 做了什么

- 新增后端 `TaskMemoryPacketBuilder`，即时读取正式记忆、记忆候选和 observation 三个 store 生成预览。
- 新增 Tauri command `preview_task_memory_packet`。
- 新增控制核心校验和过滤 helper。
- 新增 Rust 类型、前端 TS 类型、Tauri wrapper 和前端摘要 helper。
- 项目工作流侧栏候选治理卡新增最小只读“任务记忆包预览”摘要。
- 新增后端 10 个 M4 单测和前端离线 UI / 读模型断言。

## 当前权威入口

- 任务包：`tasks/2026-06-04-memory-layer-m4-task-memory-packet-builder-and-preview-v1.md`
- Evidence：`evidence/2026-06-04-memory-layer-m4-task-memory-packet-builder-and-preview-v1.md`
- 本 handoff：`handoffs/2026-06-04-memory-layer-m4-task-memory-packet-builder-and-preview-v1-result.md`

## 验证结果

通过：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- `cargo test --lib task_memory_packet`
- `cargo test --lib formal_memory`
- `cargo test --lib memory_candidate`
- `cargo test --lib observation`
- `cargo test --lib`
- `rustfmt --check src/task_memory_packet_builder.rs src/formal_memory_store.rs src/memory_candidate_store.rs src/observation_store.rs src/control_core.rs src/commands.rs src/types.rs`

注意：

- Rust 仍有既有 `JsonRpcError::invalid_params` unused warning。
- Vite build 仍有既有 chunk size warning。
- 真实窗口 / 截图验收未完成。

## 下一步建议

下一步应拆 M5：冲突和记忆 lint 最小阻断。M5 之前不要把 M4 预览解释成任务包注入，也不要让 candidate、observation、knowledge hit 或 LLM summary 进入正式 included list。
