# Evidence：Agent adapter 后端能力声明读模型 v1

日期：2026-06-03

## 结论

本轮已完成 `tasks/2026-06-03-agent-adapter-backend-capability-read-model-v1.md`。

接受为：

- 后端 `WorkbenchSnapshot` 已输出结构化 `agent_adapters[]`。
- `codex-local` 现在由后端 typed read model 声明已有能力，`source_kind = backend_read_model`。
- Agent 页优先展示后端 descriptor；没有后端 descriptor 时保留前端 fallback。
- 秘书只读模型优先读取后端 adapter descriptor warnings。

不接受为：

- Claude Code / OpenClaw / OpenCode 已接入。
- adapter 能力已经重新真实验证。
- 真实 Codex 执行语义被修改。
- workflow state JSON 结构已迁移。

## 改动文件

- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
  - 新增 `AdapterCapability` / `AgentAdapterDescriptor`。
  - `WorkbenchSnapshot` 新增 `agent_adapters: Vec<AgentAdapterDescriptor>`。
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
  - 新增 `derive_agent_adapter_descriptors` 后端读模型派生。
  - `build_snapshot_with_session_source` 读取 workflow state snapshot 用于 adapter 能力声明；读取失败只进入 descriptor warning，不让 workbench snapshot 失败。
  - 新增 Rust 单测覆盖有 Codex signal / 无 Codex signal 两类 descriptor。
- `prototypes/productized-desktop-shell/src/lib/types.ts`
  - 新增前端 `AdapterCapability` / `AgentAdapterDescriptor` 类型。
  - `WorkbenchSnapshot` 新增 `agent_adapters`。
- `prototypes/productized-desktop-shell/src/lib/adapterCapabilities.ts`
  - 改为复用 `types.ts` 的 adapter 类型。
  - 前端派生 helper 保留为 fallback，并新增 `adapter_descriptor_frontend_fallback_used` warning。
- `prototypes/productized-desktop-shell/src/views/AgentView.tsx`
  - Agent 页优先使用后端 `adapterDescriptors`。
  - 能力面板显示 `source_kind`。
- `prototypes/productized-desktop-shell/src/App.tsx`
  - `emptySnapshot` 补 `agent_adapters: []`。
  - 向 Agent 页传入 `snapshot.agent_adapters`。
- `prototypes/productized-desktop-shell/src/lib/secretaryReadModel.ts`
  - 秘书读模型优先读取 `snapshot.agent_adapters`，无后端 descriptor 时才 fallback。
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
  - 补后端 descriptor fixture。
  - Agent 页覆盖后端主路径和 fallback。
  - 前端 helper 覆盖 fallback warning。

## 能力声明范围

后端 `codex-local` descriptor 第一版覆盖：

- `session_index_read`
- `session_transcript_read`
- `workflow_node_binding`
- `safe_probe_dispatch`
- `user_reviewed_dispatch`
- `workflow_machine_run`
- `permission_decision_record`
- `harness_resource_index`

高风险能力仍是 `requires_confirmation`，boundary 说明“本轮只声明能力，不执行”。

## 验证

已通过：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
cargo test --lib
rustfmt --check src/types.rs
```

结果：

- `npm run typecheck` 通过。
- `npm run test:offline-interaction` 通过，输出 `offline interaction tests passed: 9`。
- `npm run build` 通过；Vite 仍有 chunk size warning。
- `cargo test --lib` 通过：116 passed，1 ignored。
- `rustfmt --check src/types.rs` 通过。

## 边界确认

本轮没有：

- 接 Claude Code / OpenClaw / OpenCode。
- 显示未实现 adapter 的可点击能力按钮。
- 执行 `codex exec`。
- 执行 `codex exec resume`。
- 启动真实 workflow machine。
- 改真实 Codex 执行语义。
- 读写 `/Users/yoyi/.codex`。
- 读取真实完整 transcript。
- 改 `workflow-state.v0.json` 结构。
- 迁移数据库。
- 写正式事实。
- 写正式记忆。
- 运行 harness。
- 启动 MCP canvas run。
- 写真实业务项目目录。

## 当前薄弱点

- 后端 descriptor 仍是 `WorkbenchSnapshot` 读模型，不是持久事实源。
- 本轮为了派生 adapter 能力，`load_workbench_snapshot` 会只读 workflow state snapshot；读取失败降级为 adapter warning。
- 前端 fallback 仍保留，目的是兼容测试或旧 snapshot；UI 主路径已经优先后端 descriptor。
- `src/lib.rs` 仍是大文件；本轮没有拆 WorkbenchSnapshot 组装模块。
