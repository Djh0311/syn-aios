# Task Package：Stage F / F3 Controlled Workflow Edit Proposal And Layout Boundary v1

状态：已完成。  
用途：在 F1 项目工作流画布读模型收敛、F2 节点详情 / evidence surface 产品化完成后，建立项目工作流画布的受控编辑提案与布局边界：区分“个人视图布局 / 临时视图偏好”和“workflow 事实变更”，让用户能看懂哪些交互只是看图方便，哪些必须生成 proposal / preview 并经过控制核心、权限、审计和确认；但不把 React Flow 拖拽、连线、删节点或新增节点直接写成正式 workflow 事实。  
执行方式：允许最小产品代码改动、测试、evidence 和 handoff；不得执行真实 Codex、不得读写 `/Users/yoyi/.codex`、不得写 workflow state 新顶层结构、不得迁移数据库、不得启动真实 worker、不得把 F3 做成完整画布编辑器。

## 0. 先说薄弱点

- F1 和 F2 已经把项目工作流画布读模型、节点详情和 evidence surface 收敛完成；F3 不能重写画布事实源，也不能把 F2 详情面板扩成治理后台。
- “可编辑画布”很容易被误读成 React Flow 拖一下就保存事实；这会绕过方案授权、项目主管 / 全局主管分工、控制核心、权限和审计。
- “布局保存”和“workflow 节点 / 边变更”必须分开：布局只是个人视图偏好，workflow mutation 是事实变更。
- F3 可以让用户看到编辑能力边界和 proposal / preview，但不能直接新增 / 删除节点、改边、改角色、改权限、改模型、改工具或执行任务。
- 如果实现时发现必须新增持久 layout store、workflow edit proposal store、workflow state 顶层结构或数据库迁移，必须停下回传，不能在 F3 内顺手做。
- F3 仍不是 G 阶段；真实 Tauri / 截图验收、runtime log、diagnostics、自动重试不能在 F3 中冒领完成。

## 1. 已知事实 / 未知 / 假设

已知事实：

- 阶段 C1-C6 已完成，接受为自动化工作流受控闭环完成，但不等于真实 worker 产品化完成。
- 阶段 D / M1-M13 已完成，M13 结论为 `accepted_with_deferred_items`。
- 阶段 E / E1-E7 已完成，E7 结论为 `accepted_with_deferred_items`。
- E5 Level B mario test 健康探针已完成，但只接受为指定 session 的最小真实 resume 健康探针。
- F1 已完成：`tasks/2026-06-06-stage-f-f1-project-workflow-canvas-read-model-consolidation-v1.md`。
- F2 已完成：`tasks/2026-06-06-stage-f-f2-workflow-node-detail-drawer-and-evidence-surface-v1.md`。
- F2 evidence / handoff 明确 F3 可以继承 `ProjectWorkflowCanvasReadModel`、`ProjectCanvasDetailSection.layer`、三层节点详情、任务包 / 记忆包 / 权限 / readback / audit / evidence / handoff 摘要和 React Flow 只读渲染边界。
- `docs/plans/2026-06-06-stage-e-f-g-refinement-plan-v1.md` 已把 F3 定义为 Controlled Workflow Edit Proposal And Layout Boundary。

未知：

- 当前项目画布是否已经存在可复用的布局状态、节点 position helper 或 React Flow 交互开关。
- 用户布局是否需要在 F3 持久化；如果需要，保存位置、作用域、冲突和回滚方案尚未批准。
- workflow edit proposal 是否需要独立 sidecar；当前没有授权新增 store。
- 后续 F4 项目画布 / 实验画布边界是否会要求更强的 UI 分离。
- 当前真实 Tauri / 浏览器截图工具是否可用；如果不可用，必须写入 evidence / handoff。

本任务采用的假设：

- F3 默认不新增持久 sidecar、不迁移数据库、不改 workflow state JSON 顶层结构或状态枚举。
- F3 默认不持久化布局；如果需要展示布局交互，只允许本地临时视图状态或只读边界说明。
- F3 可以新增前端纯读类型、编辑能力矩阵、proposal preview helper、局部边界面板、禁用态动作和离线测试。
- F3 如需后端字段，必须限制为纯派生 read model；如必须新增写命令或持久化，必须停下回传。
- F3 不新增一级入口、右侧顶级入口或项目页 tab；只改既有项目页工作流画布上下文。

## 2. 任务目标

完成阶段 F 第三刀：

```text
F1 canvas read model
+ F2 node detail layers
+ edit capability matrix
+ layout vs workflow mutation boundary
+ proposal / preview only for workflow changes
-> controlled edit boundary UI
-> no direct workflow mutation from React Flow
-> tests + evidence + handoff
```

F3 完成后可以说：

- 项目工作流画布的受控编辑和布局边界完成。
- 用户能区分个人视图布局、临时视图偏好、workflow 节点变更、workflow 边变更和高风险事实变更。
- React Flow 仍只负责渲染和受控交互，不是 workflow 事实源。
- 任何 workflow 事实变更只能显示 proposal / preview / disabled boundary，不会直接写 workflow state。
- 删除节点、改边、改角色、改权限、改模型、改工具等高风险操作被明确标为需要控制核心、确认和审计。
- F3 的 UI 能解释“为什么不能直接拖拽保存 / 为什么必须走 proposal”。

F3 完成后仍不能说：

- 画布编辑器完成。
- 布局持久化完成。
- workflow edit proposal 持久 store 完成。
- 节点新增 / 删除 / 连线保存 / 拖拽保存完成。
- 真实 worker / Codex 执行完成。
- 真实 send / resume 产品化完成。
- runtime log / diagnostics 完成。
- 项目画布 / 实验画布边界硬化完成。
- 阶段 F 完成。
- 阶段 G 真实 Tauri 验收完成。
- 中间版本最终验收完成。

## 3. 必须先读

当前入口：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `docs/plans/2026-06-06-stage-e-f-g-refinement-plan-v1.md`

UI / 画布边界：

- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`
- `docs/plans/2026-06-01-project-workflow-canvas-node-schema-v1.md`
- `decisions/2026-06-01-project-workflow-canvas-authority-v1.md`
- `decisions/2026-05-31-editable-canvas-codex-as-director-v1.md`

F1 / F2 前置：

- `tasks/2026-06-06-stage-f-f1-project-workflow-canvas-read-model-consolidation-v1.md`
- `evidence/2026-06-06-stage-f-f1-project-workflow-canvas-read-model-consolidation-v1.md`
- `handoffs/2026-06-06-stage-f-f1-project-workflow-canvas-read-model-consolidation-v1-result.md`
- `tasks/2026-06-06-stage-f-f2-workflow-node-detail-drawer-and-evidence-surface-v1.md`
- `evidence/2026-06-06-stage-f-f2-workflow-node-detail-drawer-and-evidence-surface-v1.md`
- `handoffs/2026-06-06-stage-f-f2-workflow-node-detail-drawer-and-evidence-surface-v1-result.md`

相关前置：

- `tasks/2026-06-04-workflow-c1-plan-authorization-and-controlled-auto-dispatch-foundation-v1.md`
- `tasks/2026-06-04-workflow-c3-global-boundary-review-and-authorization-activation-v1.md`
- `tasks/2026-06-04-workflow-c4-project-director-task-decomposition-and-authorized-prepared-auto-dispatch-v1.md`
- `tasks/2026-06-04-workflow-c5-worker-structured-report-process-fact-confirmation-and-failure-visibility-v1.md`
- `tasks/2026-06-06-stage-e-e4-session-continuation-protocol-and-permission-preview-v1.md`
- `tasks/2026-06-06-stage-e-e6-runtime-session-attention-and-readback-failure-boundary-v1.md`

主要代码入口：

- `prototypes/productized-desktop-shell/src/lib/projectCanvas.ts`
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/App.tsx`
- `prototypes/productized-desktop-shell/src/styles.css`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

如果改后端 snapshot / command，还必须读：

- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/control_core.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`

搜索固定文本必须用 `rg -F '...'` 或单引号，禁止用 shell 双引号包住未转义反引号。

## 4. 范围

允许：

- 新增或扩展前端纯读模型：
  - `ProjectWorkflowEditBoundary`
  - `ProjectCanvasEditCapability`
  - `ProjectCanvasLayoutBoundary`
  - `WorkflowEditProposalPreview`
  - 或等价命名。
- 定义编辑能力矩阵，至少区分：
  - `view_only`
  - `local_layout_preview`
  - `personal_layout_preference`
  - `workflow_node_mutation`
  - `workflow_edge_mutation`
  - `permission_or_model_mutation`
  - `execution_mutation`
- 在既有项目页工作流画布上下文中显示“编辑 / 布局边界”局部面板或卡片。
- 在节点详情或画布旁显示受控 proposal / preview：
  - 变更类型。
  - 是否只是布局。
  - 是否会改变 workflow 事实。
  - 需要谁确认。
  - 是否必须走控制核心。
  - 是否需要审计。
  - 为什么当前不能直接执行。
- 对 React Flow 交互做清晰禁用态或说明态：
  - 拖拽不保存事实。
  - 连线不保存事实。
  - 删除不直接写状态。
  - 新增节点不直接写状态。
- 如当前代码已经支持本地临时布局，可显示“本地临时、未保存、不会改 workflow 事实”的状态；默认不得新增持久保存。
- 增加离线测试覆盖：
  - 布局和 workflow mutation 被区分。
  - React Flow 不是事实源。
  - workflow mutation 只生成 preview / disabled boundary。
  - 高风险操作必须确认 / control_core / audit。
  - 禁止误导文案无命中。
- 更新 evidence / handoff 和当前入口。

禁止：

- 不执行真实 `codex exec`。
- 不执行真实 `codex exec resume`。
- 不发送真实 prompt。
- 不读写 `/Users/yoyi/.codex`。
- 不读取完整 transcript / rollout。
- 不读取 auth、token、`.env`、secret、keychain、OAuth、provider credential 或密钥文件内容。
- 不调用 Claude Code / OpenClaw / OpenCode / OpenCode-like。
- 不调用外部模型 provider。
- 不写 workflow state 新顶层结构。
- 不改 workflow state 状态枚举。
- 不迁移数据库。
- 不新增持久 sidecar。
- 不新增真实 worker dispatch。
- 不启动四角色完整工作流。
- 不把 E5 Level B 单 session 健康探针扩大为通用会话控制能力。
- 不让 React Flow 拖拽、连线、新增、删除直接写 workflow 事实。
- 不新增节点 / 删除节点 / 保存连线 / 保存拖拽布局为正式事实。
- 不做模板库、节点市场、复杂编辑器或通用自动化平台。
- 不在画布或详情里直接批准高风险权限。
- 不新增一级入口、右侧顶级入口或项目页 tab。
- 不展示任务包全文、audit 全文、transcript / rollout 全文、raw workflow state、raw sidecar、raw log、数据库路径大表或内部 schema。
- 不把 GEPA / Paseo / Odysseus 研究项并入本任务实现。

## 5. 编辑边界要求

F3 必须把以下概念分开，不能混用。

### 5.1 个人视图布局

定义：

- 只影响用户如何看画布。
- 不改变 workflow 节点、边、状态、角色、权限、任务包、记忆包或审计事实。

F3 允许：

- 显示“仅视图布局 / 不改事实”。
- 如果已有本地临时布局能力，可显示未保存状态。
- 提供 reset view / fit view / collapse groups 这类不写事实的视图行为。

F3 禁止：

- 新增持久 layout store。
- 把拖拽位置写入 workflow state。
- 把布局变化写成 audit 或正式事实。
- 把个人布局当成项目级 layout。

如执行者认为必须持久化布局，必须停下并另拆任务包，写清：

- 保存位置。
- 作用域。
- revision / 冲突处理。
- 回滚方式。
- 是否跨项目 / 跨用户。
- 是否需要迁移或 sidecar。

### 5.2 Workflow 节点变更

定义：

- 会改变 workflow 事实的节点级操作，包括新增、删除、重命名、改角色、改状态、改绑定、改任务包、改记忆包、改工具、改模型。

F3 允许：

- 显示 proposal / preview。
- 显示为什么需要确认。
- 显示可能影响的任务包、权限、审计、记忆召回和 readback。

F3 禁止：

- 直接写 workflow state。
- 直接写 task package。
- 直接改 node binding。
- 直接改权限或模型。
- 直接启动 worker 或 Codex。

### 5.3 Workflow 边变更

定义：

- 会改变 workflow 顺序、依赖、分支、回收路径或完成条件的边级操作。

F3 允许：

- 显示 edge mutation preview。
- 显示会影响哪些节点、状态流和完成闸门。
- 显示需要全局主管 / 项目主管 / 用户确认的原因。

F3 禁止：

- 通过 React Flow 连线直接写边。
- 通过删除边直接改执行路径。
- 把实验画布连线写入项目 workflow。

### 5.4 高风险变更

高风险变更至少包括：

- 删除节点。
- 删除边。
- 改角色。
- 改权限。
- 改模型 / provider / adapter。
- 改工具权限。
- 改工作目录 / writable roots。
- 改任务包目标。
- 触发真实执行、resume、retry、stop 或 restart。

F3 必须显示：

- 当前不可直接执行。
- 必须走确认弹层。
- 必须走控制核心。
- 必须写审计。
- 如涉及真实 Codex 或 `/Users/yoyi/.codex`，必须另行取得用户明确授权。

## UI 显示边界确认

本任务是否改前端：

- [ ] 不改前端、不改读模型、不改 UI 文案。
- [ ] 改前端类型 / Tauri wrapper，但不新增可见 UI。
- [x] 改读模型摘要或状态显示。
- [x] 改已有页面局部 UI。
- [x] 新增入口、面板、tab、按钮或确认动作（仅允许既有项目页工作流画布上下文内的局部编辑边界面板、proposal / preview 展示、禁用态或非事实视图控件；不新增一级入口、右侧顶级入口、项目页 tab、真实执行按钮、高风险批准按钮或持久保存动作）。

说明：这里的“按钮或控件”只允许是局部视图行为或 proposal / preview 入口，例如查看边界、展开说明、重置视图、查看预览；不得变成执行、批准、派发、resume、重试、stop、restart、保存 workflow mutation 或持久化 layout 的入口。

已读取：

- `docs/workbench-frontend-display-boundary-v1.md`
- `CURRENT.md`
- `tasks/README.md`
- `docs/plans/2026-06-01-project-workflow-canvas-node-schema-v1.md`

本任务允许显示：

- 编辑能力矩阵。
- 布局 vs workflow mutation 边界。
- proposal / preview 摘要。
- “React Flow 仅负责渲染 / 交互预览，不是事实源”。
- “需控制核心 / 需确认弹层 / 需审计”的边界文案。
- 不会改事实的局部视图控件状态。

本任务禁止显示：

- “拖拽已保存”。
- “连线已保存”。
- “节点已删除”。
- “已修改 workflow 事实”。
- “worker 已执行”。
- “Codex 已收到任务”。
- “自动派发已开始”。
- “自动重试已完成”。
- “runtime log 已完成”。
- “阶段 G 已验收”。
- “通用 send/resume 已完成”。
- “画布编辑器已完成”。

显示位置：

- 一级入口：不新增；继续使用 `项目`。
- 右侧入口：不新增；不改秘书 / 通知 / 待办 / 运行中 / 管理入口。
- 项目页：只改既有项目工作流画布上下文。
- 画布：只显示受控编辑边界、局部 preview、禁用态或非事实视图控件；不把 React Flow 当事实源。
- 节点详情：可复用 F2 三层详情展示编辑影响摘要。
- 记忆入口：不改。
- 知识库入口：不改。
- 智能体入口：不改。
- 管理入口：不改。

中间版本范围：

- 本轮必须落地：布局 / workflow mutation 边界、编辑能力矩阵、proposal / preview 只读展示和禁用态。
- 本轮只做读模型 / 摘要：影响范围、确认角色、控制核心、审计、后置任务。
- 本轮后置：真实 workflow mutation、layout 持久化、复杂编辑器、runtime log、diagnostics、真实 Tauri 全面验收。

后端和数据依赖：

- 需要后端正式读模型：优先复用 F1 / F2 的 `ProjectWorkflowCanvasReadModel` 和既有 workflow / permission / audit 摘要；如不足，新增纯派生字段，不新增 store。
- 需要审计 / 日志 / 权限 / 状态机：只显示需要审计和控制核心，不在 F3 写审计或状态。
- 不能用假数据伪装：不能 hardcode 编辑成功，不能把 disabled action 写成已完成，不能把 planned adapter 写成可执行。

UI 文案边界：

- 禁止说：`拖拽已保存`、`连线已保存`、`节点已删除`、`已修改 workflow 事实`、`worker 已执行`、`Codex 已收到任务`、`自动派发已开始`、`自动重试已完成`、`runtime log 已完成`、`阶段 G 已验收`、`通用 send/resume 已完成`、`画布编辑器已完成`。
- 允许说：`仅视图布局`、`未保存为事实`、`需要生成提案`、`需要确认弹层`、`需要控制核心`、`需要审计`、`React Flow 仅负责渲染`、`当前不可直接执行`。

验收：

- 类型检查：必须跑 `npm run typecheck`。
- 离线交互测试：必须跑 `npm run test:offline-interaction`，新增或更新 F3 覆盖。
- 构建：必须跑 `npm run build`。
- Rust：如果改 Rust，必须跑相关 `cargo test --lib ...` 和 `cargo test --lib`，并跑对应 `rustfmt --check ...`。
- 真实窗口 / 截图验收：如可用，做项目页编辑边界 smoke 和截图；如不可用，必须写入 evidence / handoff。
- 未验收项必须写入 evidence / handoff。

## 6. 建议实现顺序

1. 复核 F1 / F2 后的 `ProjectWorkflowCanvasReadModel`、`ProjectCanvasNodeDetail`、`ProjectCanvasDetailSection.layer` 和项目页画布渲染。
2. 定义编辑能力矩阵和布局 / workflow mutation 分类。
3. 在前端纯读 helper 中派生每类操作的 `allowed / preview_only / blocked / requires_confirmation / requires_control_core / requires_audit` 状态。
4. 在既有项目页工作流画布上下文中显示局部“编辑 / 布局边界”面板或卡片。
5. 对 React Flow 的拖拽、连线、删除、新增等交互显示禁用态或边界说明，确认不会写事实。
6. 复用 F2 节点详情三层结构展示编辑影响摘要。
7. 补离线测试和禁止文案扫描。
8. 更新 evidence / handoff 和入口文档。

## 7. 验收

必须通过：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- 如果改 Rust：相关 `cargo test --lib ...`
- 如果改 Rust：`cargo test --lib`
- 如果改 Rust：对应 `rustfmt --check ...`
- 禁止误导文案扫描，无新增误导完成态。

建议扫描：

```text
rg -n "拖拽已保存|连线已保存|节点已删除|已修改 workflow 事实|worker 已执行|Codex 已收到任务|自动派发已开始|自动重试已完成|runtime log 已完成|阶段 G 已验收|通用 send/resume 已完成|画布编辑器已完成" prototypes/productized-desktop-shell/src
```

事实源边界扫描：

```text
rg -n "React Flow.*事实源|react flow.*source of truth|workflow state.*drag|workflow state.*position|save layout|layout store|edit proposal store" prototypes/productized-desktop-shell/src prototypes/productized-desktop-shell/tests
```

敏感 / 真实执行扫描：

```text
rg -n "codex exec|codex exec resume|/Users/yoyi/.codex|auth.json|\\.env|token|secret|keychain|OAuth|provider credential" prototypes/productized-desktop-shell/src prototypes/productized-desktop-shell/src-tauri/src
```

浏览器 / Tauri：

- 如果 Browser / Playwright / Tauri 工具可用，打开项目页，确认编辑边界可见、禁用态清晰、console 无新增 error。
- 如果不可用，至少做 Vite HTTP smoke；如启动被沙箱阻止，必须在 evidence / handoff 写清不接受为真实窗口验收。

## 8. Evidence / Handoff 要求

完成后新增：

- `evidence/2026-06-07-stage-f-f3-controlled-workflow-edit-proposal-and-layout-boundary-v1.md`
- `handoffs/2026-06-07-stage-f-f3-controlled-workflow-edit-proposal-and-layout-boundary-v1-result.md`

Evidence 必须写清：

- F3 实际改了哪些读模型 / UI / 类型 / 测试。
- 是否改 Rust；如果改了，改动边界和测试结果。
- 布局、视图偏好、workflow node mutation、workflow edge mutation 和高风险 mutation 如何区分。
- proposal / preview 如何展示。
- React Flow 如何保持不是事实源。
- 是否新增入口 / tab / 全局面板；如果没有，要明确写没有。
- 是否新增 store、sidecar、数据库迁移或 workflow state 结构；如果没有，要明确写没有。
- 高风险动作是否仍走确认弹层和控制核心。
- 禁止文案扫描结果。
- 是否做真实窗口 / 截图验收。
- 本轮不接受为什么。

Handoff 必须写清：

- F3 是否可接受为“受控工作流编辑提案和布局边界完成”。
- F4 是否可以开始。
- F3 仍不能接受为哪些能力完成。
- 遗留风险和建议。

## 9. 回收口径

完成后可接受为：

- F3 受控工作流编辑提案和布局边界完成。
- 项目工作流画布能区分个人视图布局、临时视图偏好和 workflow 事实变更。
- workflow mutation 只能显示 proposal / preview / disabled boundary，不会直接写 workflow state。
- React Flow 仍只负责渲染和受控交互，不是事实源。
- 高风险动作仍走确认弹层、控制核心和审计边界。

完成后不接受为：

- F4 项目画布 / 实验画布边界硬化完成。
- F5 阶段 F 验收完成。
- 画布编辑器完成。
- 布局持久化完成。
- workflow edit proposal 持久 store 完成。
- 节点新增 / 删除 / 连线保存 / 拖拽保存完成。
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

## 10. Stop 条件

遇到以下情况必须停下：

- 需要执行真实 `codex exec` / `codex exec resume`。
- 需要发送真实 prompt。
- 需要读写 `/Users/yoyi/.codex`。
- 需要读取完整 transcript / rollout。
- 需要读取 secret、token、`.env`、keychain、OAuth、provider credential。
- 需要改 workflow state 顶层结构或状态枚举。
- 需要新增持久 sidecar、layout store、edit proposal store 或数据库迁移。
- 需要新增真实派发、自动重试或 runtime log store。
- 需要新增一级入口、右侧顶级入口或项目页 tab。
- 需要让 React Flow 拖拽、连线、新增或删除直接写 workflow 事实。
- 需要在详情或画布中直接批准高风险权限。
- 需要把任务包 / audit / transcript / raw state 全文铺到详情面板。

## 11. 执行结果

执行状态：已完成。

本轮实际完成：

- 新增前端纯读 `ProjectWorkflowEditBoundary` / `ProjectCanvasEditCapability` / `ProjectCanvasLayoutBoundary` / `WorkflowEditProposalPreview`，并挂到 `ProjectWorkflowCanvasReadModel.edit_boundary`。
- 在项目工作流画布侧栏新增“编辑 / 布局边界”局部面板，显示仅视图布局、未保存为事实、React Flow 仅负责渲染、proposal preview、需要确认弹层、需要控制核心和需要审计。
- 补充离线测试，覆盖 React Flow 不是 workflow authority、布局不写 workflow state、不持久化、workflow node / edge mutation 只允许 preview、高风险权限 / 模型和执行变更被阻断。
- 更新 evidence / handoff 和当前权威入口。

验证结果：

- `npm run typecheck` 通过。
- `npm run test:offline-interaction` 通过。
- `npm run build` 通过；Vite 仍有 chunk size warning。
- Vite smoke 在沙箱内启动失败：`listen EPERM: operation not permitted 127.0.0.1:5173`；非沙箱启动申请被自动拒绝，未绕过。
- 禁止误导完成态扫描通过；事实源 / layout store / edit proposal store 扫描通过；敏感 / 真实执行扫描仅命中既有边界文案和后端测试路径，未新增真实执行路径。

本轮未做：

- 未改 Rust / 后端。
- 未改 workflow state JSON 顶层结构或状态枚举。
- 未新增持久 sidecar、layout store、edit proposal store 或数据库迁移。
- 未执行真实 Codex，未读写 `/Users/yoyi/.codex`。
- 未新增一级入口、右侧顶级入口或项目页 tab。
- 未做画布编辑器、布局持久化、真实 workflow mutation、runtime log、diagnostics 或阶段 G 验收。
