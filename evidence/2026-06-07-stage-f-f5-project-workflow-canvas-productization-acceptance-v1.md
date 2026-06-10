# Evidence: Stage F / F5 Project Workflow Canvas Productization Acceptance v1

日期：2026-06-07

## 结论

F5 已完成，阶段 F 最终结论为：

```text
accepted_with_deferred_items
```

接受为：

- 阶段 F / F1-F4 项目工作流画布产品化深化完成全局主管验收。
- F1-F4 task / evidence / handoff 均存在且结论自洽。
- 项目工作流画布可以作为中间版本主工作界面进入阶段 G 验收链路。
- G readiness 决策：允许进入 G1 Runtime Log Boundary And Minimal Store。

不接受为：

- 阶段 G 已开始或已完成。
- 真实 Tauri / 截图验收完成。
- runtime log / diagnostics 完成。
- 中间版本最终验收完成。
- 真实 worker / Codex 自动执行产品化完成。
- 通用 send / resume 产品化完成。
- 项目画布和实验画布合一、自由画布编辑器、模板库、节点市场或 MCP canvas run 正式 workflow 完成。

## Acceptance Matrix

| 切片 | 预期接受范围 | 证据 | 复核结论 |
| --- | --- | --- | --- |
| F1 | 项目工作流画布读模型收敛，React Flow 仅渲染 | `tasks/2026-06-06-stage-f-f1-project-workflow-canvas-read-model-consolidation-v1.md`、`evidence/2026-06-06-stage-f-f1-project-workflow-canvas-read-model-consolidation-v1.md`、`handoffs/2026-06-06-stage-f-f1-project-workflow-canvas-read-model-consolidation-v1-result.md`、离线测试 | accepted |
| F2 | 节点详情和 evidence surface 分层，摘要 / 引用展示 | `tasks/2026-06-06-stage-f-f2-workflow-node-detail-drawer-and-evidence-surface-v1.md`、`evidence/2026-06-06-stage-f-f2-workflow-node-detail-drawer-and-evidence-surface-v1.md`、`handoffs/2026-06-06-stage-f-f2-workflow-node-detail-drawer-and-evidence-surface-v1-result.md`、离线测试 | accepted |
| F3 | 受控编辑提案和布局边界，不把 React Flow 拖拽 / 连线写 workflow 事实 | `tasks/2026-06-07-stage-f-f3-controlled-workflow-edit-proposal-and-layout-boundary-v1.md`、`evidence/2026-06-07-stage-f-f3-controlled-workflow-edit-proposal-and-layout-boundary-v1.md`、`handoffs/2026-06-07-stage-f-f3-controlled-workflow-edit-proposal-and-layout-boundary-v1-result.md`、离线测试 | accepted |
| F4 | 项目画布 / 实验画布边界硬化，实验画布不写项目事实 | `tasks/2026-06-07-stage-f-f4-project-canvas-experiment-canvas-boundary-hardening-v1.md`、`evidence/2026-06-07-stage-f-f4-project-canvas-experiment-canvas-boundary-hardening-v1.md`、`handoffs/2026-06-07-stage-f-f4-project-canvas-experiment-canvas-boundary-hardening-v1-result.md`、离线测试 | accepted |
| Stage F | 是否允许进入 G | F1-F4 综合结论、入口同步、验证和扫描 | accepted_with_deferred_items |

## 存在性复核

已确认以下文件存在：

- F1 task / evidence / handoff
- F2 task / evidence / handoff
- F3 task / evidence / handoff
- F4 task / evidence / handoff

F1-F4 均有明确不接受范围，且共同保留真实 Tauri / 截图验收、runtime log、diagnostics 和真实执行为后续阶段项。

## UI 显示边界复核

结论：通过。

复核到的源码挂载点：

- `prototypes/productized-desktop-shell/src/lib/projectCanvas.ts`
  - `ProjectWorkflowCanvasReadModel` 仍以 `workflow_state_read_model` 为事实源。
  - `edit_boundary` 来自前端只读派生，不写 workflow state。
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
  - `nodesDraggable={false}`、`nodesConnectable={false}`、`Controls showInteractive={false}`。
  - 项目页挂载 `ProjectCanvasNodeDetailView`、`ProjectCanvasSurfaceBoundaryPanel` 和 `ProjectCanvasEditBoundaryPanel`。
  - 节点详情按用户摘要、项目主管信息、技术详情分层。
- `prototypes/productized-desktop-shell/src/views/CanvasView.tsx`
  - 一级画布挂载 `ExperimentCanvasBoundaryPanel`。
  - 一级画布表达 experiment / template / canvas library 语境。
- `prototypes/productized-desktop-shell/src/lib/canvasSurfaceBoundaries.ts`
  - `experimentCanvasBoundary` 与 `projectWorkflowCanvasBoundary` 明确区分实验画布和项目工作流画布。

未发现需要在 F5 中停下修补的可见 UI 越界。

## 验证

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

F5 未改 Rust，因此未运行 `cargo test --lib` 或 `rustfmt --check`。

## 扫描

禁止误导文案扫描：

```text
rg -n '阶段 F 已完成并等同中间版本最终验收|阶段 G 已验收|真实 Tauri 验收已完成|runtime log 已完成|diagnostics 已完成|真实 worker 已执行|Codex 已收到任务|自动派发已开始|自动重试已完成|项目画布和实验画布已经合一|MCP canvas run 已成为正式 workflow|拖拽已写 workflow|连线已写 workflow|实验运行已写项目状态|已写正式记忆' prototypes/productized-desktop-shell/src CURRENT.md tasks/README.md AUTHORITY.md STAGE_PLAN.md README.md docs/plans/middleware-version-stage-plan-v1.md docs/plans/2026-06-06-stage-e-f-g-refinement-plan-v1.md
```

结果：有命中，均为允许语境：

- `canvasSurfaceBoundaries.ts` 中的黑名单常量。
- 当前入口中的“不接受为 / 禁止项 / 边界说明”否定语境。

旧口径扫描：

```text
rg -n 'F5 尚未执行|F5 仍需后续任务包|2026-06-06-stage-f-f5-project-workflow-canvas-productization-acceptance-v1|下一步执行 F5|当前可进入 F5|F5 尚未完成|F5 待执行|F5 任务包已写' CURRENT.md tasks/README.md AUTHORITY.md STAGE_PLAN.md README.md docs/plans/middleware-version-stage-plan-v1.md docs/plans/2026-06-06-stage-e-f-g-refinement-plan-v1.md
```

结果：无命中。

真实执行 / 敏感路径扫描：

```text
rg -n 'codex exec|codex exec resume|/Users/yoyi/.codex|auth\.json|\.env|token|secret|keychain|OAuth|provider credential' prototypes/productized-desktop-shell/src prototypes/productized-desktop-shell/src-tauri/src tasks/2026-06-07-stage-f-f5-project-workflow-canvas-productization-acceptance-v1.md
```

结果：有既有命中，集中在既有权限边界、guard、测试 fixture、历史真实执行能力和 F5 禁止项。本轮未新增真实执行路径。

收尾旧口径扫描：

```text
rg -n 'F5 任务包已写|F5 尚未完成|状态为待执行|当前可继续阶段 F|F2-F5|F1-F4 不接受|后续按 F5|继续阶段 F|阶段 F 剩余：F5|当前可进入 F5|下一步执行 F5' CURRENT.md tasks/README.md AUTHORITY.md STAGE_PLAN.md README.md docs/plans/middleware-version-stage-plan-v1.md docs/plans/2026-06-06-stage-e-f-g-refinement-plan-v1.md
```

结果：无命中。

G1 任务包文件存在性检查：

```text
test -f tasks/2026-06-06-stage-g-g1-runtime-log-boundary-and-minimal-store-v1.md
```

结果：exit code 1。该任务包当前不存在；F5 只授权进入 G1 任务包准备，不等同于创建或执行 G1。

## Deferred Items

- G1：runtime log boundary and minimal store。需要建立运行日志和审计边界；F5 不新增 runtime log store。
- G2：diagnostics / health / degraded state。需要建立诊断摘要和降级状态；F5 不新增 diagnostics store。
- G3：real Tauri acceptance harness and screenshot evidence。F1-F4 真实窗口 / 截图验收均未完成；F5 不冒领。
- G4：middle version end-to-end acceptance replay。需在 G1-G3 后用受控 fixture / 回放验证主链路。
- G5：final authoritative acceptance and deferred freeze。最终冻结中间版本结论和 deferred 项。

## 本轮未做

- 未改产品代码。
- 未改前端 UI 文案或读模型。
- 未改 Rust / Tauri command / store / sidecar / database migration。
- 未改 workflow state JSON 顶层结构或状态枚举。
- 未执行真实 `codex exec`。
- 未执行真实 `codex exec resume`。
- 未发送真实 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未读取完整 transcript / rollout。
- 未读取 auth、token、`.env`、secret、keychain、OAuth、provider credential。
- 未启动 MCP canvas run。
- 未新增 runtime log、diagnostics、真实派发或自动重试。
- 未做真实窗口 / 截图验收；该项明确交给 G3。
