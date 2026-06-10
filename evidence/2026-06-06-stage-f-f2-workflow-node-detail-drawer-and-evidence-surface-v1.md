# Evidence：Stage F / F2 Workflow Node Detail Drawer And Evidence Surface v1

日期：2026-06-06

## 1. 结论

F2 已完成，结论为：

```text
accepted_as_workflow_node_detail_drawer_and_evidence_surface
```

接受为：

- 项目工作流节点详情 / evidence surface 产品化完成。
- 节点详情按用户摘要、项目主管信息、技术详情三层展示。
- 任务包、任务记忆包、权限、readback、失败、audit、evidence、handoff 和 director review 以摘要 / 引用方式进入既有项目页节点详情面板。
- 用户摘要回答当前节点、当前状态、为什么停下、谁能处理和下一步。
- 项目主管信息显示任务包、记忆包、会话绑定、派发、权限、readback、失败、黑板候选、director review、evidence / handoff 引用。
- 技术详情显示 completion gate / run check、audit refs 和 source refs。
- 高风险动作仍只显示动作边界和“需确认弹层”，不在详情里直接批准。

不接受为：

- F3 画布编辑边界完成。
- 画布编辑器完成。
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
  - `ProjectCanvasDetailSection` 新增 `layer` 和 `default_open`。
  - 新增 `ProjectCanvasDetailLayer`：`user_summary`、`project_director`、`technical_details`。
  - `ProjectCanvasDetailSectionKind` 新增 `source_refs`。
  - `buildDetail()` 将节点详情拆成用户摘要、项目主管信息和技术详情三层。
  - 新增用户摘要 helper，用来派生“当前节点 / 当前状态 / 为什么停下 / 谁能处理 / 下一步 / workflow node / warning”。
  - 保留 F1 `ProjectWorkflowCanvasReadModel` 为事实读模型，不新增持久 store。
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
  - `ProjectCanvasNodeDetailView` 改为按 layer 分组渲染三组 `<details>`。
  - 动作区只显示动作边界；需要确认的动作显示“需确认弹层”。
  - 未新增一级入口、右侧顶级入口、项目页 tab、真实执行按钮或高风险直接批准入口。
- `prototypes/productized-desktop-shell/src/styles.css`
  - 新增 `.project-canvas-detail-layers` 和 `.project-canvas-detail-layer*` 样式。
  - 折叠标题保持不小于 44px 点击区域。
  - 长路径 / source refs 使用可换行展示，避免撑破详情面板。
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
  - 扩展画布读模型断言，覆盖三层详情、source refs、任务记忆包、readback、权限和禁止全文边界。
  - 扩展项目页静态渲染断言，确认用户摘要、项目主管信息、技术详情、evidence / handoff、需确认弹层等文案可见。

文档：

- `tasks/2026-06-06-stage-f-f2-workflow-node-detail-drawer-and-evidence-surface-v1.md`
- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `docs/plans/2026-06-06-stage-e-f-g-refinement-plan-v1.md`
- `evidence/2026-06-06-stage-f-f2-workflow-node-detail-drawer-and-evidence-surface-v1.md`
- `handoffs/2026-06-06-stage-f-f2-workflow-node-detail-drawer-and-evidence-surface-v1-result.md`

Rust / 后端：

- 未修改 Rust。
- 未扩展 `WorkbenchSnapshot`。
- 未新增 Tauri command。
- 未新增 store、sidecar 或数据库迁移。
- 未改 workflow state 顶层结构或状态枚举。

## 3. 三层详情实现

用户摘要：

- section layer：`user_summary`
- 默认展开。
- 内容包括当前节点、当前状态、为什么停下、谁能处理、下一步、workflow node 和 warning。
- readback unavailable 会显示为不可用 / 不能显示成真实 0 条结果。
- pending permission 会解释为用户或控制核心确认事项。

项目主管信息：

- section layer：`project_director`
- 默认展开。
- 内容包括当前任务、任务包字段、任务记忆包摘要、会话绑定、派发摘要、readback 摘要、权限请求、黑板候选、失败 / 超时、总指导回收摘要、evidence refs 和 handoff refs。
- 任务包只显示字段摘要、artifact path / refs、状态和缺失字段 warning。
- 任务记忆包只显示 included / excluded / review materials 数量和理由摘要，不展示 memory store 原始记录全文。

技术详情：

- section layer：`technical_details`
- 默认折叠。
- 内容包括运行与完成闸门、最近审计和 source refs。
- 只显示 audit id / source ref / workflow ref 等引用，不展示 audit 全文、raw state 或大段 JSON。

## 4. 显示边界

事实源仍来自 F1 已收敛的前端读模型：

- `ProjectWorkflowCanvasReadModel`
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

本轮没有：

- 新增一级入口。
- 新增右侧顶级入口。
- 新增项目页 tab。
- 新增全局 evidence 中心。
- 新增任务包管理器。
- 新增真实执行、resume、retry、stop、restart 按钮。
- 在详情中直接批准高风险权限。
- 把详情面板做成治理后台。

## 5. 验证

通过：

```text
npm run typecheck
```

结果：通过。

```text
npm run test:offline-interaction
```

结果：通过，输出：

```text
offline interaction tests passed: 11
```

```text
npm run build
```

结果：通过。Vite 构建成功；保留既有 chunk size warning。

## 6. 禁止文案和边界扫描

禁止完成态文案扫描：

```text
rg -n "worker 已执行|Codex 已收到任务|自动派发已开始|自动重试已完成|runtime log 已完成|阶段 G 已验收|通用 send/resume 已完成|节点详情已完成" prototypes/productized-desktop-shell/src
```

结果：无命中。

详情边界扫描：

```text
rg -n "任务包全文|audit 全文|transcript 全文|raw workflow state|raw sidecar|raw log|0 条结果|0 条读回" prototypes/productized-desktop-shell/src prototypes/productized-desktop-shell/tests
```

结果有命中，均属于边界说明、否定语境、既有真实 0 计数分支或测试禁止词：

- `AgentView.tsx` 的“raw log”命中是既有否定文案：不显示 raw log、不自动重试、不执行 stop 或 resume。
- `projectCanvas.ts` 的 “0 条结果 / 0 条读回”命中是 F2 / F1 readback unavailable 边界说明：不能显示成真实 0。
- `sessionContinuation.ts` 的命中是 E4 / E5 readback expectation 边界说明。
- `ProjectsView.tsx` 的 `真实 0 条结果` 是既有 dispatch stats 在事件数或命中数确认为 0 时的分支，不用于 unavailable。
- `offline-permission-dialog.test.tsx` 的命中是禁止词断言。

未发现 F2 把 readback unavailable / failed 伪装成真实 0 条结果。

## 7. 浏览器 / Tauri 验收

尝试启动 Vite dev server：

```text
npm run dev -- --host 127.0.0.1
```

结果：失败。

```text
Error: listen EPERM: operation not permitted 127.0.0.1:5173
```

随后按权限规则申请 escalated 本地端口启动，用于项目页真实窗口 smoke。申请被安全审查拒绝，原因是 unsandboxed localhost dev server 对本轮属于可选 UI smoke，用户未单独授权该提升。

因此：

- 浏览器 / 真实窗口 smoke 未完成。
- 真实 Tauri / 截图验收未完成。
- 本轮不接受为真实窗口验收完成；该项仍交给 G3。

## 8. 执行边界

本轮未做：

- 未执行真实 `codex exec`。
- 未执行真实 `codex exec resume`。
- 未发送真实 prompt。
- 未读取完整 transcript / rollout。
- 未读取 auth、token、`.env`、secret、keychain、OAuth、provider credential。
- 未调用 Claude Code / OpenClaw / OpenCode / OpenCode-like。
- 未调用外部模型 provider。
- 未改 workflow state 顶层结构或状态枚举。
- 未迁移数据库。
- 未新增持久 sidecar。
- 未新增真实 worker dispatch。
- 未启动 MCP canvas run。

路径边界说明：

- 本次收尾阶段未读取或写入 `/Users/yoyi/.codex`。
- 产品实现未读取真实 Codex state、session、transcript、rollout 或 credential。
- 交接摘要显示前半段曾为 UI 技能使用读取过技能说明；该读取不涉及真实 Codex 数据，也没有写入 `/Users/yoyi/.codex`。因此本证据不把 F2 包装成“全过程从未触碰技能文件路径”，只确认产品实现和收尾未读写真实 Codex 数据。

## 9. 下一步

F3 可以进入任务包编写 / 执行前确认：

```text
Controlled Workflow Edit Proposal And Layout Boundary
```

F3 仍必须保持：

- React Flow 不直接成为事实源。
- 不直接保存拖拽布局或连线，除非任务包单独授权。
- 不新增真实 Codex 执行。
- 不读写 `/Users/yoyi/.codex`，除非用户对具体任务重新明确授权。
