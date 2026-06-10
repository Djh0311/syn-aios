# Evidence: Stage F / F3 Controlled Workflow Edit Proposal And Layout Boundary v1

日期：2026-06-07

## 结论

F3 已完成。接受为项目工作流画布的受控工作流编辑提案和布局边界完成：画布 read model 现在带有只读编辑边界，项目页侧栏能解释“仅视图布局”和“workflow 事实变更”的区别，workflow 节点 / 边 / 权限 / 模型 / 执行变更只显示 proposal / preview / blocked boundary，不会直接写 workflow state。

不接受为画布编辑器、布局持久化、workflow edit proposal 持久 store、节点新增 / 删除 / 连线保存 / 拖拽保存、真实 worker / Codex 执行、runtime log、diagnostics、阶段 F 完成或阶段 G 真实 Tauri 验收完成。

## 改动范围

- `prototypes/productized-desktop-shell/src/lib/projectCanvas.ts`
  - 新增 `ProjectCanvasEditCapabilityKind`、`ProjectCanvasEditCapabilityStatus`、`ProjectCanvasEditCapability`、`ProjectCanvasLayoutBoundary`、`WorkflowEditProposalPreview`、`ProjectWorkflowEditBoundary`。
  - `ProjectWorkflowCanvasReadModel` 新增 `edit_boundary` 字段。
  - 新增 `buildProjectWorkflowEditBoundary()` 纯前端派生 helper。
  - 能力矩阵覆盖 `view_only`、`local_layout_preview`、`personal_layout_preference`、`workflow_node_mutation`、`workflow_edge_mutation`、`permission_or_model_mutation`、`execution_mutation`。
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
  - 在既有项目工作流画布侧栏新增 `ProjectCanvasEditBoundaryPanel`。
  - 显示“编辑 / 布局边界”“仅视图布局”“未保存为事实”“React Flow 仅负责渲染”“需要生成提案”“需要确认弹层”“需要控制核心”“需要审计”。
  - 未新增保存、删除、连线、执行或高风险确认按钮。
- `prototypes/productized-desktop-shell/src/styles.css`
  - 新增紧凑能力矩阵样式。
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
  - 增加 read model 断言和项目页静态文案断言。
  - 增加误导完成态文案黑名单断言。

## 边界说明

- 布局：`layout_boundary.scope = "view_only"`，`writes_workflow_state = false`，`persists_layout = false`。
- React Flow：`react_flow_source_of_truth = false`，只负责渲染和视图交互。
- workflow 节点 / 边变更：`preview_only`，必须生成 proposal，必须经过确认、控制核心和审计。
- 权限 / 模型 / 工具 / 范围变更：`blocked`，属于高风险事实变更。
- 执行 / 派发 / 重试：`blocked`，F3 不启动 worker，也不执行真实 Codex 命令。

## 验证

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，`offline interaction tests passed: 11`。
- `npm run build`：通过；Vite 输出 chunk size warning，未阻断构建。
- Vite smoke：`npm run dev -- --host 127.0.0.1` 在沙箱内失败，错误为 `listen EPERM: operation not permitted 127.0.0.1:5173`。按权限规则申请非沙箱启动被自动拒绝；本轮未绕过，也不接受为真实窗口 / 截图验收。

## 扫描

- 禁止误导完成态扫描：无命中。
- 事实源 / layout / edit proposal store 扫描：无命中。
- 敏感 / 真实执行关键词扫描：有既有命中，集中在已有边界文案、`PermissionDialog`、历史执行能力、后端 guard / 测试路径和敏感词过滤；本轮未新增真实执行路径，未读写 `/Users/yoyi/.codex`。

## 未做

- 未改 Rust / 后端。
- 未改 workflow state JSON 顶层结构或状态枚举。
- 未新增持久 sidecar、layout store、edit proposal store 或数据库迁移。
- 未执行真实 Codex，未发送真实 prompt，未读写 `/Users/yoyi/.codex`。
- 未读取完整 transcript / rollout、auth、token、`.env`、secret、keychain、OAuth 或 provider credential。
- 未新增一级入口、右侧顶级入口或项目页 tab。
- 未做 F4 项目画布 / 实验画布边界硬化。
