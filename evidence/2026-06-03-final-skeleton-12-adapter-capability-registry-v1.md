# Evidence：final-skeleton-12 适配器能力声明骨架 v1

日期：2026-06-03

## 结论

本轮已完成 `final-skeleton-12-adapter-capability-registry-v1` 的最小骨架：

- 定义前端只读 `AdapterCapability` / `AgentAdapterDescriptor`。
- 给现有 Codex 路径声明能力。
- 智能体页展示 Codex adapter 能力声明。
- 未实现的 Claude Code / OpenClaw / OpenCode 只作为隐藏未实现 adapter 记录，不显示能力按钮。

本轮没有进入 `final-skeleton-11`，没有实现黑板候选写入。

## 改动文件

- `prototypes/productized-desktop-shell/src/lib/adapterCapabilities.ts`
  - 新增 `AdapterCapabilityKind`、`AdapterCapabilityStatus`、`AdapterCapability`、`AgentAdapterDescriptor`。
  - 新增 `deriveAgentAdapterDescriptors`，从已传入前端的 sessions、projects、workflowState 派生 Codex adapter 能力声明。
  - 能力包括：会话索引读取、会话正文只读、工作流节点绑定、安全测试派发、用户审核业务派发、四角色工作流机器、权限结论记录、Harness 资源索引。
- `prototypes/productized-desktop-shell/src/views/AgentView.tsx`
  - 智能体页新增“适配器能力”只读面板。
  - 面板只显示 Codex adapter descriptor，不提供未实现 adapter 按钮。
- `prototypes/productized-desktop-shell/src/App.tsx`
  - 给 `AgentView` 传入 projects 和 workflowState，以便前端读模型派生能力声明。
- `prototypes/productized-desktop-shell/src/styles.css`
  - 新增适配器能力声明面板样式。
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
  - 新增 `deriveAgentAdapterDescriptors` 测试。
  - 补 Agent 页能力声明 UI 断言。

## 验证

已通过：

```text
npm run typecheck
npm run test:offline-interaction
npm run build
```

结果：

- `npm run typecheck` 通过。
- `npm run test:offline-interaction` 通过，输出 `offline interaction tests passed: 5`。
- `npm run build` 通过。

构建提醒：

- Vite 仍提示主 chunk 超过 500 kB。本轮不处理拆包策略。

## 边界确认

本轮没有：

- 接 Claude / OpenClaw / OpenCode。
- 改真实 Codex 执行语义。
- 执行真实 Codex。
- 改 workflow state JSON。
- 读写 `/Users/yoyi/.codex`。
- 显示未实现能力按钮。
- 实现黑板候选写入。

## 当前薄弱点

- 适配器能力声明目前是前端读模型，后端 snapshot 还没有结构化暴露 `agent_adapters[]` descriptor。
- 高风险能力只做声明，仍依赖既有确认弹层和后端命令；本轮没有重新验证真实执行路径。
- Claude Code / OpenClaw / OpenCode 只是隐藏未实现记录，不能说已经接入。
