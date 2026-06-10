# Task Package：Stage F / F5 Project Workflow Canvas Productization Acceptance v1

状态：已完成。  
用途：对阶段 F / F1-F4 做全局主管总复核，冻结阶段 F accepted / deferred / blocked 项，判断项目工作流画布是否可以进入阶段 G 的真实 Tauri、运行日志、诊断和最终中间版本验收链路。F5 是阶段验收任务包，不是新功能开发任务。  
执行方式：默认只做证据复核、文档同步、禁止口径扫描、必要验证命令和 evidence / handoff；不得新增产品功能，不得执行真实 Codex，不得读写 `/Users/yoyi/.codex`，不得启动 MCP canvas run，不得新增 runtime log / diagnostics / Tauri 截图验收能力。

## 0. 先说薄弱点

- F1-F4 已经把项目工作流画布从读模型、节点详情、编辑边界、实验画布边界四层收起来，但它们都没有完成真实 Tauri / 截图验收。
- F5 容易被误写成“阶段 F 已验收，所以中间版本可交付”。这是错的：F5 只能决定是否允许进入 G，不能替代 G1-G5。
- F5 也容易被误写成“发现问题顺手修掉”。作为全局主管验收，发现产品缺陷时应记录、分类、必要时拆 F5.1 / F4.1 修补包；不要在 F5 中偷偷开发。
- 阶段 F 的关键边界是：React Flow 不是事实源，项目画布不等于自由画布编辑器，实验画布不写项目事实，真实执行、运行日志、诊断和截图验收仍在 G。
- 如果 F5 发现 F1-F4 任一 evidence / handoff 不存在、结论冲突、入口口径冲突或可见 UI 文案越界，必须停下并给出 `needs_changes`，不能硬收 `accepted`。

## 1. 已知事实 / 未知 / 假设

已知事实：

- 阶段 C1-C6 已完成，接受为自动化工作流受控闭环完成，但不等于真实 worker / Codex 产品化完成。
- 阶段 D / M1-M13 已完成，M13 最终结论为 `accepted_with_deferred_items`。
- 阶段 E / E1-E7 已完成，E7 最终结论为 `accepted_with_deferred_items`。
- E5 Level B mario test 健康探针已完成，但只接受为指定 session 的最小真实 `codex exec resume` 健康探针，不是通用 send / resume 产品化。
- F1 已完成：`tasks/2026-06-06-stage-f-f1-project-workflow-canvas-read-model-consolidation-v1.md`。
- F2 已完成：`tasks/2026-06-06-stage-f-f2-workflow-node-detail-drawer-and-evidence-surface-v1.md`。
- F3 已完成：`tasks/2026-06-07-stage-f-f3-controlled-workflow-edit-proposal-and-layout-boundary-v1.md`。
- F4 已完成：`tasks/2026-06-07-stage-f-f4-project-canvas-experiment-canvas-boundary-hardening-v1.md`。
- F1-F4 都不接受为真实执行、runtime log、diagnostics、阶段 G 真实 Tauri 验收或中间版本最终验收完成。
- `docs/plans/2026-06-06-stage-e-f-g-refinement-plan-v1.md` 已把 F5 定义为 Stage F Acceptance。

未知：

- F1-F4 evidence / handoff 是否存在互相冲突或入口旧口径残留。
- 当前源码里是否仍有把阶段 F 夸大为阶段 G / 中间版本最终验收完成的可见文案。
- 当前真实 Tauri / 浏览器截图工具是否可用；如果不可用，必须记录为 deferred 到 G3。
- 是否存在 F1-F4 之外的历史画布文案误导后续执行线。

本任务采用的假设：

- F5 默认不改产品代码、不改 Rust、不改 Tauri command、不新增 store、不迁移数据库。
- F5 默认不运行真实 worker / Codex，不执行 `codex exec` / `codex exec resume`，不读写 `/Users/yoyi/.codex`。
- F5 可以更新任务包状态、CURRENT、tasks/README、AUTHORITY、STAGE_PLAN、README、阶段计划、evidence 和 handoff。
- F5 可以运行静态扫描和前端验证命令；如果命令因环境不可用失败，必须记录失败原因和是否阻断。
- F5 最终结论只能是 `accepted`、`accepted_with_deferred_items` 或 `needs_changes` 之一。

## 2. 任务目标

完成阶段 F 第五刀：

```text
F1 project workflow canvas read model
+ F2 node detail / evidence surface
+ F3 controlled edit proposal / layout boundary
+ F4 project canvas / experiment canvas boundary
+ F5 authoritative acceptance
-> Stage F acceptance matrix
-> G readiness decision
-> evidence + handoff + authority sync
```

F5 完成后可以说：

- 阶段 F / F1-F4 已完成全局主管验收。
- 项目工作流画布产品化深化可接受为进入阶段 G 真实验收链路。
- F1-F4 的 evidence / handoff、入口文档和禁止边界已复核。
- F5 给出了阶段 F 的最终结论：`accepted` / `accepted_with_deferred_items` / `needs_changes`。

F5 完成后仍不能说：

- 阶段 G 已开始或已完成。
- 真实 Tauri / 截图验收已完成。
- runtime log / diagnostics 已完成。
- 真实 worker / Codex 自动执行已产品化。
- 通用 send / resume 已产品化。
- 项目画布和实验画布已经合一。
- 自由画布编辑器、模板库、节点市场、ComfyUI / n8n / Langflow 复刻完成。
- 中间版本最终验收完成。
- 最终蓝图完整工作台完成。

## 3. 必须先读

当前入口：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `docs/plans/2026-06-06-stage-e-f-g-refinement-plan-v1.md`

UI / 画布边界：

- `docs/workbench-frontend-display-boundary-v1.md`
- `docs/plans/task-package-ui-display-boundary-rule-v1.md`
- `docs/plans/2026-06-01-project-workflow-canvas-node-schema-v1.md`
- `decisions/2026-06-01-project-workflow-canvas-authority-v1.md`
- `decisions/2026-05-31-editable-canvas-codex-as-director-v1.md`

F1-F4 任务包和回收材料：

- `tasks/2026-06-06-stage-f-f1-project-workflow-canvas-read-model-consolidation-v1.md`
- `evidence/2026-06-06-stage-f-f1-project-workflow-canvas-read-model-consolidation-v1.md`
- `handoffs/2026-06-06-stage-f-f1-project-workflow-canvas-read-model-consolidation-v1-result.md`
- `tasks/2026-06-06-stage-f-f2-workflow-node-detail-drawer-and-evidence-surface-v1.md`
- `evidence/2026-06-06-stage-f-f2-workflow-node-detail-drawer-and-evidence-surface-v1.md`
- `handoffs/2026-06-06-stage-f-f2-workflow-node-detail-drawer-and-evidence-surface-v1-result.md`
- `tasks/2026-06-07-stage-f-f3-controlled-workflow-edit-proposal-and-layout-boundary-v1.md`
- `evidence/2026-06-07-stage-f-f3-controlled-workflow-edit-proposal-and-layout-boundary-v1.md`
- `handoffs/2026-06-07-stage-f-f3-controlled-workflow-edit-proposal-and-layout-boundary-v1-result.md`
- `tasks/2026-06-07-stage-f-f4-project-canvas-experiment-canvas-boundary-hardening-v1.md`
- `evidence/2026-06-07-stage-f-f4-project-canvas-experiment-canvas-boundary-hardening-v1.md`
- `handoffs/2026-06-07-stage-f-f4-project-canvas-experiment-canvas-boundary-hardening-v1-result.md`

主要代码入口，只读复核：

- `prototypes/productized-desktop-shell/src/lib/projectCanvas.ts`
- `prototypes/productized-desktop-shell/src/lib/canvasSurfaceBoundaries.ts`
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `prototypes/productized-desktop-shell/src/views/CanvasView.tsx`
- `prototypes/productized-desktop-shell/src/styles.css`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`

搜索固定文本必须用 `rg -F '...'` 或单引号，禁止用 shell 双引号包住未转义反引号。

## 4. 范围

允许：

- 复核 F1-F4 task / evidence / handoff 是否存在、是否自洽、是否与当前权威入口一致。
- 复核 F1-F4 是否都明确“不接受为阶段 F 完成 / 阶段 G 验收 / 中间版本最终验收”。
- 复核项目工作流画布 UI 边界：
  - F1 read model 来自 workflow state 派生读模型。
  - F2 node detail 分层清楚，摘要 / 引用 / evidence / handoff 不展开原始全文。
  - F3 edit / layout boundary 不把拖拽 / 连线 / 删除写成 workflow 事实。
  - F4 project canvas / experiment canvas boundary 不混淆项目画布和实验画布。
- 复核当前入口文档和阶段计划，必要时同步状态为 F5 任务包已写 / 待执行，执行完成后再改为 F5 已完成。
- 新增 F5 evidence / handoff。
- 给出阶段 F acceptance matrix。
- 给出 G readiness decision。
- 给出 deferred items，明确进入 G1 / G2 / G3 / G4 / G5 或后续 backlog。
- 运行验证命令和扫描命令。

禁止：

- 不新增产品功能。
- 不改项目画布、实验画布、会话中心、记忆中心、知识库、秘书、通知、待办、运行中或管理入口的可见 UI。
- 不新增一级入口、右侧入口、项目 tab、按钮或确认动作。
- 不新增 runtime log store。
- 不新增 diagnostics store。
- 不新增 sidecar。
- 不迁移数据库。
- 不改 workflow state JSON 顶层结构或状态枚举。
- 不启动 MCP canvas run。
- 不把 MCP canvas run 作为默认项目工作流。
- 不把独立 `CanvasView` / `CanvasDefinition` / `CanvasRunState` 写入正式项目事实。
- 不执行真实 `codex exec`。
- 不执行真实 `codex exec resume`。
- 不发送真实 prompt。
- 不读写 `/Users/yoyi/.codex`。
- 不读取完整 transcript / rollout。
- 不读取 auth、token、`.env`、secret、keychain、OAuth、provider credential 或密钥文件内容。
- 不调用 Claude Code / OpenClaw / OpenCode / OpenCode-like。
- 不调用外部模型 provider。
- 不启动四角色完整工作流。
- 不把 E5 Level B 单 session 健康探针扩大为通用会话控制能力。
- 不把 GEPA / Paseo / Odysseus 研究项并入本任务实现。

## 5. UI 显示边界确认

本任务是否改前端：

- [x] 不改前端、不改读模型、不改 UI 文案。
- [ ] 改前端类型 / Tauri wrapper，但不新增可见 UI。
- [ ] 改读模型摘要或状态显示。
- [ ] 改已有页面局部 UI。
- [ ] 新增入口、面板、tab、按钮或确认动作。

说明：F5 是 UI 边界复核任务，不是 UI 实现任务。若复核发现 UI 文案越界，应在 F5 evidence 中记录为 `needs_changes` 或另拆修补任务包，不在 F5 中直接开发。

必须复核的 UI 口径：

- 项目工作流画布显示的是 project / workflow / authorization / control core 语境。
- 项目画布事实源来自 workflow state 派生读模型。
- React Flow 仅负责渲染。
- 节点详情按用户摘要、项目主管信息、技术详情分层。
- evidence / handoff / audit / source refs 只显示摘要和引用，不展示全文。
- 编辑 / 布局边界只解释 proposal / preview / 禁用态，不把拖拽写成事实。
- 一级画布显示 experiment / template / canvas library 语境。
- 实验画布不写正式项目事实、正式记忆或项目 workflow。

本任务禁止显示或写入权威入口：

- `阶段 F 已完成并等同中间版本最终验收`
- `阶段 G 已验收`
- `真实 Tauri 验收已完成`
- `runtime log 已完成`
- `diagnostics 已完成`
- `真实 worker 已执行`
- `Codex 已收到任务`
- `自动派发已开始`
- `自动重试已完成`
- `项目画布和实验画布已经合一`
- `MCP canvas run 已成为正式 workflow`
- `拖拽已写 workflow`
- `连线已写 workflow`
- `实验运行已写项目状态`
- `已写正式记忆`

完成 F5 后允许显示：

- `阶段 F 已完成全局主管验收`
- `F1-F4 已可追溯`
- `项目工作流画布产品化深化可进入阶段 G`
- `真实 Tauri / 截图验收仍在 G3`
- `runtime log / diagnostics 仍在 G1 / G2`

## 6. Acceptance Matrix

F5 必须输出表格或等价结构，至少包含：

| 切片 | 预期接受范围 | 证据 | 复核结论 |
| --- | --- | --- | --- |
| F1 | 项目工作流画布读模型收敛，React Flow 仅渲染 | task / evidence / handoff / test | pending |
| F2 | 节点详情和 evidence surface 分层 | task / evidence / handoff / test | pending |
| F3 | 受控编辑提案和布局边界 | task / evidence / handoff / test | pending |
| F4 | 项目画布 / 实验画布边界硬化 | task / evidence / handoff / test | pending |
| Stage F | 是否允许进入 G | F1-F4 综合结论 | pending |

每项结论只能是：

- `accepted`
- `accepted_with_deferred_items`
- `needs_changes`
- `blocked`

建议默认 Stage F 结论：

- 如果 F1-F4 均自洽且仅剩真实 Tauri / runtime log / diagnostics 等 G 阶段项，结论应为 `accepted_with_deferred_items`。
- 如果发现 F1-F4 某一项证据缺失、入口口径冲突或源码 UI 明显越界，结论应为 `needs_changes`。
- 不建议使用 `accepted`，除非真实窗口 / 截图缺口也已经被单独证据补齐；当前根据 F1-F4 记录，G3 缺口仍存在。

## 7. 建议执行顺序

1. 核对 F1-F4 task / evidence / handoff 文件存在。
2. 核对 F1-F4 各自结论、接受范围、不接受范围和未完成项。
3. 核对 F1-F4 在 `CURRENT.md`、`tasks/README.md`、`AUTHORITY.md`、`STAGE_PLAN.md`、`README.md`、阶段计划中的入口状态。
4. 只读复核项目画布、节点详情、编辑边界、实验画布边界相关源码，确认不是只写了文案常量而没有挂 UI。
5. 运行必要验证命令。
6. 跑禁止文案扫描和旧口径扫描。
7. 输出 Stage F acceptance matrix。
8. 新增 F5 evidence / handoff。
9. 更新 F5 任务包状态和权威入口。

## 8. 验收

必须通过或记录失败原因：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`

如果 F5 没改 Rust：

- 不要求 `cargo test --lib`。
- 不要求 `rustfmt --check`。

如果执行者违反默认边界改了 Rust：

- 必须跑相关 `cargo test --lib ...`
- 必须跑 `cargo test --lib`
- 必须跑对应 `rustfmt --check ...`
- 必须在 evidence 中解释为什么 F5 需要改 Rust；通常应视为越界并另拆修补任务。

建议存在性扫描：

```text
test -f tasks/2026-06-06-stage-f-f1-project-workflow-canvas-read-model-consolidation-v1.md
test -f evidence/2026-06-06-stage-f-f1-project-workflow-canvas-read-model-consolidation-v1.md
test -f handoffs/2026-06-06-stage-f-f1-project-workflow-canvas-read-model-consolidation-v1-result.md
test -f tasks/2026-06-06-stage-f-f2-workflow-node-detail-drawer-and-evidence-surface-v1.md
test -f evidence/2026-06-06-stage-f-f2-workflow-node-detail-drawer-and-evidence-surface-v1.md
test -f handoffs/2026-06-06-stage-f-f2-workflow-node-detail-drawer-and-evidence-surface-v1-result.md
test -f tasks/2026-06-07-stage-f-f3-controlled-workflow-edit-proposal-and-layout-boundary-v1.md
test -f evidence/2026-06-07-stage-f-f3-controlled-workflow-edit-proposal-and-layout-boundary-v1.md
test -f handoffs/2026-06-07-stage-f-f3-controlled-workflow-edit-proposal-and-layout-boundary-v1-result.md
test -f tasks/2026-06-07-stage-f-f4-project-canvas-experiment-canvas-boundary-hardening-v1.md
test -f evidence/2026-06-07-stage-f-f4-project-canvas-experiment-canvas-boundary-hardening-v1.md
test -f handoffs/2026-06-07-stage-f-f4-project-canvas-experiment-canvas-boundary-hardening-v1-result.md
```

禁止误导文案扫描：

```text
rg -n '阶段 F 已完成并等同中间版本最终验收|阶段 G 已验收|真实 Tauri 验收已完成|runtime log 已完成|diagnostics 已完成|真实 worker 已执行|Codex 已收到任务|自动派发已开始|自动重试已完成|项目画布和实验画布已经合一|MCP canvas run 已成为正式 workflow|拖拽已写 workflow|连线已写 workflow|实验运行已写项目状态|已写正式记忆' prototypes/productized-desktop-shell/src CURRENT.md tasks/README.md AUTHORITY.md STAGE_PLAN.md README.md docs/plans/middleware-version-stage-plan-v1.md docs/plans/2026-06-06-stage-e-f-g-refinement-plan-v1.md
```

允许命中：

- evidence / handoff / task 中的“不接受为”“禁止项”“扫描命令”“黑名单常量”。

旧口径扫描：

```text
rg -n 'F5 尚未执行|F5 仍需后续任务包|2026-06-06-stage-f-f5-project-workflow-canvas-productization-acceptance-v1|下一步执行 F5|当前可进入 F5' CURRENT.md tasks/README.md AUTHORITY.md STAGE_PLAN.md README.md docs/plans/middleware-version-stage-plan-v1.md docs/plans/2026-06-06-stage-e-f-g-refinement-plan-v1.md
```

F5 执行完成后，上述旧口径应无当前入口残留；但本任务包刚写完、尚未执行时，允许入口显示“F5 任务包已写，状态待执行，F5 尚未完成”。

真实执行 / 敏感路径扫描：

```text
rg -n 'codex exec|codex exec resume|/Users/yoyi/.codex|auth\.json|\.env|token|secret|keychain|OAuth|provider credential' prototypes/productized-desktop-shell/src prototypes/productized-desktop-shell/src-tauri/src tasks/2026-06-07-stage-f-f5-project-workflow-canvas-productization-acceptance-v1.md
```

允许命中：

- F5 禁止项。
- 既有 guard / 边界文案 / 测试 fixture。

浏览器 / Tauri：

- F5 不负责完成真实 Tauri / 截图验收。
- 如果 Browser / Playwright / Tauri 工具可用，可做只读 smoke 并记录，但不得把它声明为 G3 完成。
- 如果不可用，明确写入 evidence / handoff：真实窗口 / 截图验收仍交给 G3。

## 9. Evidence / Handoff 要求

完成后新增：

- `evidence/2026-06-07-stage-f-f5-project-workflow-canvas-productization-acceptance-v1.md`
- `handoffs/2026-06-07-stage-f-f5-project-workflow-canvas-productization-acceptance-v1-result.md`

Evidence 必须写清：

- F5 最终结论：`accepted` / `accepted_with_deferred_items` / `needs_changes`。
- F1-F4 acceptance matrix。
- F1-F4 task / evidence / handoff 是否都存在。
- F1-F4 各自是否有不接受范围遗漏。
- UI 显示边界复核结果。
- 禁止误导文案扫描结果。
- 旧口径扫描结果。
- 验证命令结果。
- 是否改产品代码；默认应为否。
- 是否执行真实 Codex；必须为否。
- 是否读写 `/Users/yoyi/.codex`；必须为否。
- 是否做真实窗口 / 截图验收；若未做，明确交给 G3。
- 阶段 G readiness：是否允许进入 G1。

Handoff 必须写清：

- 阶段 F 是否可以接受为完成。
- F5 是否允许进入 G1。
- F5 仍不能接受为哪些能力完成。
- Deferred items 分配到 G1 / G2 / G3 / G4 / G5 / backlog。
- 给下一任全局主管的最短下一步。

## 10. 回收口径

完成后可接受为：

- 阶段 F / 项目工作流画布产品化深化完成全局主管验收。
- F1-F4 evidence / handoff 和入口口径已复核。
- 项目工作流画布可作为中间版本主工作界面进入阶段 G 验收。
- 阶段 F 结论已冻结为 `accepted` / `accepted_with_deferred_items` / `needs_changes`。

完成后不接受为：

- 阶段 G 已开始。
- 阶段 G 真实 Tauri 验收完成。
- runtime log / diagnostics 完成。
- 中间版本最终验收完成。
- 最终蓝图完整工作台完成。
- 真实 worker / Codex 自动执行产品化完成。
- 通用 send / resume 产品化完成。
- 自动重试完成。
- planned adapters 真实接入完成。
- provider credential / model verification 完成。
- 项目画布和实验画布已经合一。
- 自由画布编辑器完成。
- MCP canvas run 已成为正式 workflow。
- 模板库 / 节点市场完成。
- ComfyUI / n8n / Langflow 复刻完成。

## 11. Stop 条件

遇到以下情况必须停下：

- 需要新增或修改产品功能。
- 需要改前端 UI 文案或读模型才能通过验收。
- 需要改 Rust / Tauri command / store / sidecar / database migration。
- 需要改 workflow state 顶层结构或状态枚举。
- 需要执行真实 `codex exec` / `codex exec resume`。
- 需要发送真实 prompt。
- 需要读写 `/Users/yoyi/.codex`。
- 需要读取完整 transcript / rollout。
- 需要读取 secret、token、`.env`、keychain、OAuth、provider credential。
- 需要启动 MCP canvas run。
- 需要新增 runtime log store。
- 需要新增 diagnostics store。
- 需要新增真实派发、自动重试或外部 provider 调用。
- F1-F4 任一 evidence / handoff 缺失或结论冲突。
- 发现可见 UI 文案把实验画布、项目画布、真实执行、阶段 G 或最终验收混淆。

## 12. 完成后入口同步

F5 执行完成后必须同步：

- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `docs/plans/2026-06-06-stage-e-f-g-refinement-plan-v1.md`
- 本任务包状态

同步后的方向应是：

```text
F1 已完成
-> F2 已完成
-> F3 已完成
-> F4 已完成
-> F5 已完成，阶段 F 结论冻结
-> 下一步 G1 Runtime Log Boundary And Minimal Store
```

如果 F5 结论为 `needs_changes`，不得推进 G1；必须先拆修补任务包。

## 13. 执行结果

F5 已完成，记录见：

- `evidence/2026-06-07-stage-f-f5-project-workflow-canvas-productization-acceptance-v1.md`
- `handoffs/2026-06-07-stage-f-f5-project-workflow-canvas-productization-acceptance-v1-result.md`

阶段 F 最终结论为：

```text
accepted_with_deferred_items
```

F1-F4 task / evidence / handoff 已复核，入口口径已同步；项目工作流画布可作为中间版本主工作界面进入阶段 G 验收链路。G readiness 决策：允许进入 G1 Runtime Log Boundary And Minimal Store。

验证：

- `npm run typecheck`：通过。
- `npm run test:offline-interaction`：通过，输出 `offline interaction tests passed: 11`。
- `npm run build`：通过，保留既有 Vite chunk size warning。

本轮未改产品代码，未改 Rust，未改 workflow state JSON，未新增 store / sidecar / DB migration，未执行真实 Codex，未启动 MCP canvas run，未读写 `/Users/yoyi/.codex`，未做真实窗口 / 截图验收。真实 Tauri / 截图验收仍交给 G3，runtime log / diagnostics 仍交给 G1 / G2。
