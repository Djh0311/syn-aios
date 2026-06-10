# Task Package：Stage F / F4 Project Canvas / Experiment Canvas Boundary Hardening v1

状态：已完成。  
用途：在 F1 项目工作流画布读模型收敛、F2 节点详情 / evidence surface 产品化、F3 受控编辑提案和布局边界完成后，彻底区分“项目工作流画布”和“一级实验 / 模板画布”：项目画布服务项目主管、workflow state、授权、任务包、权限、readback、记忆和审计；一级画布只作为 experiment / template / canvas library 语境，不写正式项目事实、正式记忆或正式 workflow。  
执行方式：允许最小产品代码改动、测试、evidence 和 handoff；不得执行真实 Codex、不得读写 `/Users/yoyi/.codex`、不得启动 MCP canvas run、不得写 workflow state 新顶层结构、不得迁移数据库、不得把实验画布并入项目事实源。

## 0. 先说薄弱点

- 项目工作流画布和独立 `CanvasView` 都使用“画布”语义，用户和开发线容易把实验运行误认为正式项目运行。
- `CanvasView` 当前可显示实验 / 模板画布并有运行相关 UI；F4 不能扩大它，也不能把它变成项目工作流默认执行入口。
- F1-F3 已明确项目工作流画布事实源是 workflow state 派生读模型，React Flow 只负责渲染和受控交互；F4 不能反向让独立 CanvasDefinition / CanvasRunState 抢回事实源。
- F4 的目标是边界硬化，不是新画布设计、模板库、节点市场、ComfyUI / n8n / Langflow 复刻。
- 如果实现时发现必须迁移 `CanvasView`、合并项目 workflow state、写 MCP canvas run 或新增 canvas/project 映射 store，必须停下回传并另拆任务包。

## 1. 已知事实 / 未知 / 假设

已知事实：

- 阶段 C1-C6 已完成，接受为自动化工作流受控闭环完成，但不等于真实 worker 产品化完成。
- 阶段 D / M1-M13 已完成，M13 结论为 `accepted_with_deferred_items`。
- 阶段 E / E1-E7 已完成，E7 结论为 `accepted_with_deferred_items`。
- E5 Level B mario test 健康探针已完成，但只接受为指定 session 的最小真实 resume 健康探针。
- F1 已完成：`tasks/2026-06-06-stage-f-f1-project-workflow-canvas-read-model-consolidation-v1.md`。
- F2 已完成：`tasks/2026-06-06-stage-f-f2-workflow-node-detail-drawer-and-evidence-surface-v1.md`。
- F3 已完成：`tasks/2026-06-07-stage-f-f3-controlled-workflow-edit-proposal-and-layout-boundary-v1.md`。
- `decisions/2026-06-01-project-workflow-canvas-authority-v1.md` 明确独立 `CanvasView` 和 MCP canvas/run 文件层不是当前项目工作流权威事实源。
- `docs/plans/2026-06-01-project-workflow-canvas-node-schema-v1.md` 明确项目画布从 `WorkflowStateSnapshot` 派生，独立 `CanvasDefinition` / `CanvasRunState` 不直接派生项目事实。
- `docs/plans/2026-06-06-stage-e-f-g-refinement-plan-v1.md` 已把 F4 定义为 Project Canvas / Experiment Canvas Boundary Hardening。

未知：

- 现有一级 `画布` 导航和页面文案是否已经完全表达 experiment / template / canvas library 语境。
- 项目页工作流 tab / 画布区域是否还有“实验画布”“模板运行”“MCP canvas run”之类易混文案。
- 现有离线测试是否能覆盖一级画布和项目工作流画布两个入口的文案差异。
- 当前真实 Tauri / 浏览器截图工具是否可用；如果不可用，必须写入 evidence / handoff。

本任务采用的假设：

- F4 默认不新增持久 sidecar、不迁移数据库、不改 workflow state JSON 顶层结构或状态枚举。
- F4 默认不启动、测试或验证真实 MCP canvas run。
- F4 可以新增前端纯读边界 helper、局部文案、只读边界面板、禁用态说明和离线测试。
- F4 不新增一级入口、右侧顶级入口或项目页 tab；只硬化现有 `画布` 一级入口和现有项目页工作流画布语境。

## 2. 任务目标

完成阶段 F 第四刀：

```text
F1 project workflow canvas read model
+ F2 node detail layers
+ F3 edit / layout boundary
+ CanvasView experiment / template boundary
+ Project workflow canvas project / workflow / authorization boundary
-> no confusion between experiment canvas and project workflow canvas
-> tests + evidence + handoff
```

F4 完成后可以说：

- 项目工作流画布和一级实验 / 模板画布的边界硬化完成。
- 一级 `画布` 清楚显示 experiment / template / canvas library 语境。
- 项目页工作流画布清楚显示 project / workflow / authorization / control core 语境。
- 实验画布不会被文案或 UI 暗示为正式项目运行、正式 workflow、正式事实或正式记忆入口。
- 项目画布的运行和变更仍必须经过 workflow state、控制核心、权限和审计边界。

F4 完成后仍不能说：

- F5 阶段 F 验收完成。
- 项目画布和实验画布已经合一。
- 独立画布可以写项目 workflow state。
- MCP canvas run 已成为正式 workflow。
- 模板库 / 节点市场完成。
- ComfyUI / n8n / Langflow 复刻完成。
- 真实 worker / Codex 执行完成。
- runtime log / diagnostics 完成。
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

F1-F3 前置：

- `tasks/2026-06-06-stage-f-f1-project-workflow-canvas-read-model-consolidation-v1.md`
- `evidence/2026-06-06-stage-f-f1-project-workflow-canvas-read-model-consolidation-v1.md`
- `handoffs/2026-06-06-stage-f-f1-project-workflow-canvas-read-model-consolidation-v1-result.md`
- `tasks/2026-06-06-stage-f-f2-workflow-node-detail-drawer-and-evidence-surface-v1.md`
- `evidence/2026-06-06-stage-f-f2-workflow-node-detail-drawer-and-evidence-surface-v1.md`
- `handoffs/2026-06-06-stage-f-f2-workflow-node-detail-drawer-and-evidence-surface-v1-result.md`
- `tasks/2026-06-07-stage-f-f3-controlled-workflow-edit-proposal-and-layout-boundary-v1.md`
- `evidence/2026-06-07-stage-f-f3-controlled-workflow-edit-proposal-and-layout-boundary-v1.md`
- `handoffs/2026-06-07-stage-f-f3-controlled-workflow-edit-proposal-and-layout-boundary-v1-result.md`

主要代码入口：

- `prototypes/productized-desktop-shell/src/App.tsx`
- `prototypes/productized-desktop-shell/src/views/CanvasView.tsx`
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `prototypes/productized-desktop-shell/src/lib/projectCanvas.ts`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- `prototypes/productized-desktop-shell/src/styles.css`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

如果改后端 snapshot / command，还必须读：

- `prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/mcp/**`

搜索固定文本必须用 `rg -F '...'` 或单引号，禁止用 shell 双引号包住未转义反引号。

## 4. 范围

允许：

- 对一级 `画布` 页面补充或收敛 experiment / template / canvas library 语境。
- 对项目页工作流画布补充或收敛 project / workflow / authorization / control core 语境。
- 新增或扩展前端纯读边界模型，例如：
  - `CanvasSurfaceBoundary`
  - `CanvasContextKind`
  - `ProjectCanvasAuthorityBoundary`
  - `ExperimentCanvasBoundary`
  - 或等价命名。
- 在 `CanvasView` 中显示：
  - 当前是实验 / 模板画布。
  - 不写正式项目事实。
  - 不写正式记忆。
  - 不写项目 workflow state。
  - MCP canvas run 不是默认项目工作流。
- 在项目工作流画布中显示：
  - 当前是项目 / workflow / authorization 语境。
  - 事实源来自 workflow state 派生读模型。
  - 运行和变更必须走控制核心、权限和审计。
- 增加离线测试覆盖：
  - 一级画布文案是 experiment / template / canvas library。
  - 项目画布文案是 project / workflow / authorization。
  - 没有“实验运行已写项目状态”“MCP canvas run 已成为正式 workflow”等误导文案。
  - 独立 `CanvasView` 不被测试认定为项目 workflow 事实源。
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
- 不启动 MCP canvas run。
- 不把 MCP canvas run 作为默认项目工作流。
- 不把独立 `CanvasView` / `CanvasDefinition` / `CanvasRunState` 写入正式项目事实。
- 不写 workflow state 新顶层结构。
- 不改 workflow state 状态枚举。
- 不迁移数据库。
- 不新增持久 sidecar。
- 不新增真实 worker dispatch。
- 不启动四角色完整工作流。
- 不把 E5 Level B 单 session 健康探针扩大为通用会话控制能力。
- 不做 ComfyUI / n8n / Langflow 复刻。
- 不做模板库、节点市场或通用节点执行器。
- 不新增一级入口、右侧顶级入口或项目页 tab。
- 不展示任务包全文、audit 全文、transcript / rollout 全文、raw workflow state、raw sidecar、raw log、数据库路径大表或内部 schema。
- 不把 GEPA / Paseo / Odysseus 研究项并入本任务实现。

## 5. 边界要求

### 5.1 一级画布：实验 / 模板语境

必须表达：

- 当前画布不是项目 workflow 事实源。
- 当前画布用于实验、模板、草图或后置 canvas library 能力。
- 实验运行不会自动写正式项目事实、正式记忆或正式 workflow。
- 如果存在运行按钮或运行文案，必须明确不是默认项目工作流。

允许显示：

- `实验 / 模板画布`
- `不写项目状态`
- `不写正式记忆`
- `不是项目 workflow 事实源`
- `MCP canvas run 非默认项目工作流`

禁止显示：

- `项目运行已开始`
- `实验运行已写项目状态`
- `MCP canvas run 已成为正式 workflow`
- `已写正式记忆`
- `已派发项目 worker`

### 5.2 项目画布：project / workflow / authorization 语境

必须表达：

- 当前画布属于项目工作流。
- 事实源来自 workflow state 派生读模型。
- 运行、权限、任务包、记忆包、readback、审计和结果回收都属于项目 workflow 边界。
- 任何变更或执行仍必须走控制核心、权限和审计。

允许显示：

- `项目工作流画布`
- `workflow state 派生读模型`
- `方案授权 / 控制核心 / 权限 / 审计`
- `React Flow 仅负责渲染`
- `实验画布不会写入本项目事实`

禁止显示：

- `实验画布已并入项目`
- `独立 CanvasDefinition 是项目事实源`
- `拖拽已写 workflow`
- `连线已写 workflow`
- `MCP canvas run 已推进项目状态`

### 5.3 Cross-surface 跳转和引用

如果 F4 涉及跨页面链接或提示：

- 允许从项目画布提示“可去实验画布探索模板”，但必须明确不会写项目事实。
- 允许从实验画布提示“正式项目运行请回项目工作流”，但不能直接启动项目 worker。
- 不允许一键把实验画布运行结果写入项目 workflow。
- 不允许从实验画布直接创建正式任务包、正式记忆或正式 workflow mutation。

## UI 显示边界确认

本任务是否改前端：

- [ ] 不改前端、不改读模型、不改 UI 文案。
- [ ] 改前端类型 / Tauri wrapper，但不新增可见 UI。
- [x] 改读模型摘要或状态显示。
- [x] 改已有页面局部 UI。
- [x] 新增入口、面板、tab、按钮或确认动作（仅允许既有一级 `画布` 页面和既有项目页工作流画布上下文内的边界说明、禁用态、语境 badge 或只读提示；不新增一级入口、右侧顶级入口、项目页 tab、真实执行按钮、高风险批准按钮或持久保存动作）。

说明：这里的“新增入口 / 面板 / 按钮”只允许是边界说明和只读提示；不得变成实验画布写项目事实、启动 MCP canvas run、派发 worker、resume、重试、stop、restart、保存 workflow mutation 或写正式记忆的入口。

已读取：

- `docs/workbench-frontend-display-boundary-v1.md`
- `CURRENT.md`
- `tasks/README.md`
- `docs/plans/2026-06-01-project-workflow-canvas-node-schema-v1.md`

本任务允许显示：

- 一级画布的 experiment / template / canvas library 语境。
- 项目画布的 project / workflow / authorization 语境。
- `CanvasView` 不是项目 workflow 事实源。
- 项目画布事实源来自 workflow state 派生读模型。
- MCP canvas run 不是默认项目工作流。
- 实验画布不会写正式项目事实、正式记忆或项目任务。

本任务禁止显示：

- `实验运行已写项目状态`。
- `MCP canvas run 已成为正式 workflow`。
- `实验画布已并入项目`。
- `独立 CanvasDefinition 是项目事实源`。
- `已写正式记忆`。
- `已派发项目 worker`。
- `worker 已执行`。
- `Codex 已收到任务`。
- `自动派发已开始`。
- `自动重试已完成`。
- `runtime log 已完成`。
- `阶段 G 已验收`。

显示位置：

- 一级入口：不新增；继续使用既有 `画布`，但明确为实验 / 模板语境。
- 右侧入口：不新增；不改秘书 / 通知 / 待办 / 运行中 / 管理入口。
- 项目页：只改既有项目工作流画布上下文的边界说明。
- 项目画布：显示 project / workflow / authorization 语境。
- 实验画布：显示 experiment / template / library 语境。
- 记忆入口：不改。
- 知识库入口：不改。
- 智能体入口：不改。
- 管理入口：不改。

中间版本范围：

- 本轮必须落地：项目画布和实验画布的语境边界、误导文案清理、只读边界说明和测试。
- 本轮只做读模型 / 摘要：事实源说明、可写入边界、后置能力提示。
- 本轮后置：画布合一、模板库、节点市场、MCP canvas run 正式接入、runtime log、diagnostics、真实 Tauri 全面验收。

后端和数据依赖：

- 需要后端正式读模型：优先复用 F1-F3 的前端读模型和现有 snapshot；如不足，新增纯前端派生说明，不新增 store。
- 需要审计 / 日志 / 权限 / 状态机：只显示边界，不在 F4 写审计或状态。
- 不能用假数据伪装：不能 hardcode 实验运行成功，不能把 disabled action 写成已完成，不能把 CanvasRunState 写成项目 workflow run。

UI 文案边界：

- 禁止说：`实验运行已写项目状态`、`MCP canvas run 已成为正式 workflow`、`实验画布已并入项目`、`独立 CanvasDefinition 是项目事实源`、`已写正式记忆`、`已派发项目 worker`、`worker 已执行`、`Codex 已收到任务`、`自动派发已开始`、`runtime log 已完成`、`阶段 G 已验收`。
- 允许说：`实验 / 模板画布`、`项目工作流画布`、`不会写项目事实`、`不会写正式记忆`、`workflow state 派生读模型`、`控制核心 / 权限 / 审计`、`React Flow 仅负责渲染`、`MCP canvas run 非默认项目工作流`。

验收：

- 类型检查：必须跑 `npm run typecheck`。
- 离线交互测试：必须跑 `npm run test:offline-interaction`，新增或更新 F4 覆盖。
- 构建：必须跑 `npm run build`。
- Rust：如果改 Rust，必须跑相关 `cargo test --lib ...` 和 `cargo test --lib`，并跑对应 `rustfmt --check ...`。
- 真实窗口 / 截图验收：如可用，做一级画布和项目页工作流画布 smoke 和截图；如不可用，必须写入 evidence / handoff。
- 未验收项必须写入 evidence / handoff。

## 6. 建议实现顺序

1. 复核 `CanvasView.tsx`、`App.tsx` 一级画布入口、`ProjectsView.tsx` 项目工作流画布和 F1-F3 读模型。
2. 定义或补充画布 surface boundary helper，区分 `experiment_canvas` 与 `project_workflow_canvas`。
3. 在一级画布中补充 experiment / template / library 语境和“不写项目事实 / 正式记忆”的提示。
4. 在项目工作流画布中补充 project / workflow / authorization / control core 语境和“实验画布不会写本项目事实”的提示。
5. 确认没有新增项目 tab、右侧入口或全局 evidence / runtime 入口。
6. 补离线测试和禁止文案扫描。
7. 更新 evidence / handoff 和入口文档。

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
rg -n "实验运行已写项目状态|MCP canvas run 已成为正式 workflow|实验画布已并入项目|独立 CanvasDefinition 是项目事实源|已写正式记忆|已派发项目 worker|worker 已执行|Codex 已收到任务|自动派发已开始|自动重试已完成|runtime log 已完成|阶段 G 已验收" prototypes/productized-desktop-shell/src
```

事实源边界扫描：

```text
rg -n "CanvasDefinition.*项目事实|CanvasRunState.*workflow|MCP canvas run.*项目|experiment.*workflow state|template.*workflow state|CanvasView.*事实源" prototypes/productized-desktop-shell/src prototypes/productized-desktop-shell/tests
```

敏感 / 真实执行扫描：

```text
rg -n "codex exec|codex exec resume|/Users/yoyi/.codex|auth.json|\\.env|token|secret|keychain|OAuth|provider credential" prototypes/productized-desktop-shell/src prototypes/productized-desktop-shell/src-tauri/src
```

浏览器 / Tauri：

- 如果 Browser / Playwright / Tauri 工具可用，打开一级画布和项目页工作流画布，确认语境边界清晰、console 无新增 error。
- 如果不可用，至少做 Vite HTTP smoke；如启动被沙箱阻止，必须在 evidence / handoff 写清不接受为真实窗口验收。

## 8. Evidence / Handoff 要求

完成后新增：

- `evidence/2026-06-07-stage-f-f4-project-canvas-experiment-canvas-boundary-hardening-v1.md`
- `handoffs/2026-06-07-stage-f-f4-project-canvas-experiment-canvas-boundary-hardening-v1-result.md`

Evidence 必须写清：

- F4 实际改了哪些读模型 / UI / 类型 / 测试。
- 是否改 Rust；如果改了，改动边界和测试结果。
- 一级画布如何表达 experiment / template / canvas library 语境。
- 项目画布如何表达 project / workflow / authorization 语境。
- 是否启动 MCP canvas run；必须明确没有。
- 是否新增入口 / tab / 全局面板；如果没有，要明确写没有。
- 是否新增 store、sidecar、数据库迁移或 workflow state 结构；如果没有，要明确写没有。
- 禁止文案扫描结果。
- 是否做真实窗口 / 截图验收。
- 本轮不接受为什么。

Handoff 必须写清：

- F4 是否可接受为“项目画布 / 实验画布边界硬化完成”。
- F5 是否可以开始。
- F4 仍不能接受为哪些能力完成。
- 遗留风险和建议。

## 9. 回收口径

完成后可接受为：

- F4 项目画布 / 实验画布边界硬化完成。
- 一级画布明确是 experiment / template / canvas library 语境。
- 项目画布明确是 project / workflow / authorization / control core 语境。
- 实验画布不写正式项目事实、正式记忆或项目 workflow。
- 项目画布运行和变更仍必须经过 workflow state、控制核心、权限和审计。

完成后不接受为：

- F5 阶段 F 验收完成。
- 项目画布和实验画布已经合一。
- 独立画布可以写项目 workflow state。
- MCP canvas run 已成为正式 workflow。
- 模板库 / 节点市场完成。
- ComfyUI / n8n / Langflow 复刻完成。
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
- 需要启动 MCP canvas run。
- 需要把实验画布结果写入项目 workflow state。
- 需要改 workflow state 顶层结构或状态枚举。
- 需要新增持久 sidecar、canvas/project 映射 store 或数据库迁移。
- 需要新增真实派发、自动重试或 runtime log store。
- 需要新增一级入口、右侧顶级入口或项目页 tab。
- 需要把独立 `CanvasView` / `CanvasDefinition` / `CanvasRunState` 当项目事实源。
- 需要在实验画布中直接创建正式任务包、正式记忆或正式 workflow mutation。

## 11. 执行结果

F4 已完成，记录见：

- `evidence/2026-06-07-stage-f-f4-project-canvas-experiment-canvas-boundary-hardening-v1.md`
- `handoffs/2026-06-07-stage-f-f4-project-canvas-experiment-canvas-boundary-hardening-v1-result.md`

本轮新增前端纯读 `CanvasSurfaceBoundary`，在一级实验 / 模板画布显示 experiment / template / canvas library 边界，在项目工作流画布侧栏显示 project / workflow / authorization / control core 边界。一级画布运行文案已明确为实验画布运行，不会自动写正式项目事实、正式记忆或项目 workflow；项目画布明确事实源来自 workflow state 派生读模型，运行和变更仍走方案授权、控制核心、权限和审计。

验证：

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过。
- `npm run build`：通过，保留既有 Vite chunk size warning。
- `npm run dev -- --host 127.0.0.1`：沙箱内 `listen EPERM`；非沙箱本地端口启动申请被安全审查拒绝，真实窗口 / 截图验收未完成。

本轮未改 Rust，未执行真实 Codex，未启动 MCP canvas run，未读写 `/Users/yoyi/.codex`，未新增 store / sidecar / DB migration / workflow state 顶层结构，未新增一级入口、右侧顶级入口或项目页 tab。
