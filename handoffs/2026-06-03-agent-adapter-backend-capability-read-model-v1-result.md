# Handoff：Agent adapter 后端能力声明读模型 v1

日期：2026-06-03

## 本轮完成

`2026-06-03-agent-adapter-backend-capability-read-model-v1.md` 已完成。

完成内容：

1. 后端 `WorkbenchSnapshot` 新增 `agent_adapters[]`。
2. 后端新增 typed `AdapterCapability` / `AgentAdapterDescriptor`。
3. `codex-local` descriptor 改为后端读模型输出，`source_kind = backend_read_model`。
4. Agent 页优先展示后端 descriptor，无后端 descriptor 时才用前端 fallback。
5. 秘书只读模型优先读取后端 adapter warnings。
6. 补 Rust 单测和前端离线测试。

## 改动文件

- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/lib/adapterCapabilities.ts`
- `prototypes/productized-desktop-shell/src/views/AgentView.tsx`
- `prototypes/productized-desktop-shell/src/App.tsx`
- `prototypes/productized-desktop-shell/src/lib/secretaryReadModel.ts`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- `evidence/2026-06-03-agent-adapter-backend-capability-read-model-v1.md`
- `handoffs/2026-06-03-agent-adapter-backend-capability-read-model-v1-result.md`
- `CURRENT.md`
- `tasks/README.md`

## 验证结果

通过：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
cargo test --lib
rustfmt --check src/types.rs
```

备注：

- `npm run test:offline-interaction` 输出 `offline interaction tests passed: 9`。
- `cargo test --lib` 输出 116 passed，1 ignored。
- `npm run build` 有 Vite chunk size warning，但构建成功。

## 没有做

- 没有接 Claude Code / OpenClaw / OpenCode。
- 没有改真实 Codex 执行语义。
- 没有执行 `codex exec` 或 `codex exec resume`。
- 没有启动 workflow machine。
- 没有读写 `/Users/yoyi/.codex`。
- 没有读取真实完整 transcript。
- 没有改 `workflow-state.v0.json` 结构。
- 没有迁移数据库。
- 没有写正式事实或正式记忆。
- 没有运行 harness。
- 没有启动 MCP canvas run。
- 没有写真实业务项目目录。

## 后续建议

后续如果继续多 agent 接入，应先单开 Claude Code / OpenClaw / OpenCode adapter descriptor 设计任务。不要直接显示操作按钮，也不要把隐藏未实现 adapter 改成可用。

如果继续清理架构，建议单开 WorkbenchSnapshot 组装拆分任务；本轮只新增后端 read model，没有拆 `src/lib.rs` 大文件。
