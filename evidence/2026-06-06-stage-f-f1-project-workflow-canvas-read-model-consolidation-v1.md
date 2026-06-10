# Evidence：Stage F / F1 Project Workflow Canvas Read Model Consolidation v1

日期：2026-06-06

## 1. 结论

F1 已完成，结论为：

```text
accepted_as_project_workflow_canvas_read_model_consolidation
```

接受为：

- 项目工作流画布统一使用 `ProjectWorkflowCanvasReadModel` 作为节点、边、状态、badge、attention、状态原因和节点详情摘要的来源。
- `ProjectWorkflowCanvasReadModel` 已扩展 `status_reason` 和 `attention_items[]`，并覆盖 `empty`、`blocked`、`needs_review`、`prepared`、`running`、`ready_for_review`、`accepted`、`failed`、`timed_out`、`waiting_for_permission`、`readback_unavailable`、`unknown` 等状态表达。
- 项目画布详情侧栏以摘要 / 引用方式显示任务包、任务记忆包、权限、readback、audit、evidence、handoff、dispatch、director review 和黑板候选。
- React Flow 仍只负责渲染、选择和查看；节点不可拖拽、不可连线，不保存布局，不写 workflow state。
- 项目页只改既有项目工作流画布区域和既有节点详情侧栏，没有新增一级入口、右侧顶级入口、项目页 tab、真实执行按钮或确认动作。

不接受为：

- F2 节点详情完整抽屉完成。
- 画布编辑能力完成。
- 真实 worker / Codex 执行完成。
- 真实 send / resume 产品化完成。
- 自动派发产品化完成。
- 自动重试完成。
- runtime log / diagnostics 完成。
- planned adapters 真实接入完成。
- provider credential / model verification 完成。
- 阶段 F 完成。
- 阶段 G 真实 Tauri 验收完成。
- 中间版本最终验收完成。

## 2. 修改范围

产品代码：

- `prototypes/productized-desktop-shell/src/lib/projectCanvas.ts`
  - 扩展 `ProjectCanvasStatus`、`ProjectCanvasSourceRef`、`ProjectCanvasDetailSectionKind`。
  - 新增 `ProjectCanvasAttention`、`ProjectCanvasStatusReason`。
  - `deriveProjectWorkflowCanvasReadModel` 新增 attention / status reason / runtime attention 可选输入。
  - 状态派生收敛到读模型：权限、running、failed、timed_out、prepared、needs_review、readback_unavailable、run_check blocked 和 empty。
  - 节点详情新增任务记忆包、readback、director review、evidence、handoff 摘要。
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
  - 项目画布接收 `runtimeSessionAttention` 并传入读模型。
  - React Flow / 静态 fallback 使用 `attention_items[]` 和 `status_reason` 显示少量画布摘要。
  - 右侧节点详情侧栏新增“画布状态原因”只读卡片。
  - 补齐 `empty`、`prepared`、`needs_review`、`readback_unavailable` 文案。
- `prototypes/productized-desktop-shell/src/App.tsx`
  - 向项目页传入 `snapshot.runtime_session_attention`。
- `prototypes/productized-desktop-shell/src/styles.css`
  - 新增画布 attention 摘要条样式。
  - 补齐 prepared / needs_review / readback_unavailable 节点状态样式。
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
  - 扩展项目画布读模型测试，覆盖 status reason、attention、memory packet、readback、prepared、empty 和 readback unavailable。
  - 扩展项目页静态渲染断言，确认画布状态原因、任务记忆包摘要和 readback 摘要进入侧栏。

文档：

- `tasks/2026-06-06-stage-f-f1-project-workflow-canvas-read-model-consolidation-v1.md`
- `CURRENT.md`
- `tasks/README.md`
- `evidence/2026-06-06-stage-f-f1-project-workflow-canvas-read-model-consolidation-v1.md`
- `handoffs/2026-06-06-stage-f-f1-project-workflow-canvas-read-model-consolidation-v1-result.md`

Rust / 后端：

- 未修改 Rust。
- 未扩展 `WorkbenchSnapshot`。
- 未新增 command、store、sidecar 或数据库迁移。

## 3. 事实源和显示边界

事实源来自：

- `WorkflowStateSnapshot.project_workflows[]`
- `ProjectWorkflowSummary.derived_workflow`
- `ProjectWorkflowSummary.task_drafts[]`
- `ProjectWorkflowSummary.node_session_bindings[]`
- `ProjectWorkflowSummary.node_dispatches[]`
- `ProjectWorkflowSummary.permission_requests[]`
- `ProjectWorkflowSummary.execution_attempts[]`
- `ProjectWorkflowSummary.director_reviews[]`
- `TaskPackage.memory_injection_summary`
- `TaskDraftSummary.recent_audit_events[]`
- `WorkflowStateSnapshot.project_blackboards[]`
- `WorkbenchSnapshot.runtime_session_attention[]` 的前端可选输入

显示边界：

- 主画布只显示节点、边、badge、少量 attention 摘要和状态原因。
- 任务包、记忆包、权限、readback、audit、evidence、handoff 只在侧栏详情显示摘要 / 引用。
- 不展示任务包全文、audit 全文、transcript / rollout 全文、raw workflow state、raw sidecar、raw log、路径大表或内部 schema。
- readback unavailable 显示为不可用 / 无可信来源，不显示为真实 0 条结果。

## 4. React Flow 边界

本轮确认：

- `nodesDraggable={false}`
- `nodesConnectable={false}`
- `Controls showInteractive={false}`
- React Flow 节点只读取 `ProjectCanvasNode`。
- React Flow edge 只读取 `ProjectCanvasEdge`。
- React Flow 不保存布局，不写 workflow state，不新增节点 / 删除节点 / 保存连线。

静态 fallback：

- `ProjectCanvasStaticStage` 同样读取 `ProjectWorkflowCanvasReadModel`。
- 静态 fallback 也显示 global badges、status reason 和 attention 摘要。

## 5. 状态覆盖

本轮覆盖：

- `empty`：缺 workflow 或缺当前 work item，只显示空态占位，不补编任务。
- `blocked`：run check / authorization 阻断。
- `needs_review`：authorization check 需要复核。
- `prepared`：prepared dispatch 只代表准备记录，不代表已有 worker 产出。
- `running`：running dispatch。
- `ready_for_review`：等待回收。
- `accepted`：工作项或回收结论已接受。
- `failed`：失败 attempt。
- `timed_out`：超时 attempt。
- `waiting_for_permission`：pending permission request。
- `readback_unavailable`：runtime attention 或 dispatch readback 摘要不可用，不能显示成真实 0 条结果。
- `unknown`：无法判断的状态。

## 6. 验证

通过：

```text
npm run typecheck
```

结果：通过。

```text
npm run test:offline-interaction
```

结果：通过，输出 `offline interaction tests passed: 11`。

```text
npm run build
```

结果：通过。Vite 构建成功；保留既有 chunk size warning。

禁止完成态文案扫描：

```text
rg -n 'worker 已执行|Codex 已收到任务|自动派发已开始|自动重试已完成|runtime log 已完成|阶段 G 已验收|通用 send/resume 已完成' src
```

结果：无命中。

readback 边界扫描：

```text
rg -n 'readback unavailable.*0|readback_unavailable.*0|0 条结果|0 条读回' src tests
```

结果：有命中，均为边界说明、guard、测试断言或“不能显示成 0 条结果”的否定语境；未发现把 readback unavailable 声明为真实 0 条结果。

`codex exec resume` 扫描：

```text
rg -n -F 'codex exec resume' src tests
```

结果：有命中，均为既有权限边界、测试 fixture、能力声明或禁止 / 需确认文案；F1 未新增真实执行路径。

## 7. 浏览器 / Tauri 验收

尝试启动 Vite dev server：

```text
npm run dev
```

结果：沙箱内绑定 `127.0.0.1:5173` 失败，`listen EPERM`。

随后按权限规则申请 escalated 本地端口启动，用于浏览器 smoke。该申请被安全审查拒绝，原因是本轮任务不需要打开非沙箱监听端口完成核心验收。

因此：

- 普通浏览器 / 截图 smoke 未完成。
- 真实 Tauri / 截图验收未完成。
- 本轮不接受为真实窗口验收完成；该项仍交给 G3。

## 8. 边界自检

本轮没有：

- 执行真实 `codex exec`。
- 执行真实 `codex exec resume`。
- 发送真实 prompt。
- 读写 `/Users/yoyi/.codex`。
- 读取完整 transcript / rollout。
- 读取 auth、token、`.env`、secret、keychain、OAuth、provider credential。
- 调用 Claude Code / OpenClaw / OpenCode / OpenCode-like。
- 调用外部模型 provider。
- 改 workflow state 顶层结构或状态枚举。
- 迁移数据库。
- 新增持久 sidecar。
- 新增真实 worker dispatch。
- 启动 MCP canvas run。
- 把独立实验 `CanvasView` / `CanvasDefinition` 当项目 workflow 事实源。
- 新增一级入口、右侧顶级入口或项目页 tab。
