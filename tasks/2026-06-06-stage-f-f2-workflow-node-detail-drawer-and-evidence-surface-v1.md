# Task Package：Stage F / F2 Workflow Node Detail Drawer And Evidence Surface v1

状态：已完成。  
用途：在 F1 项目工作流画布读模型收敛完成后，把项目工作流画布的节点详情层级产品化：用右侧抽屉 / 既有节点详情面板承载任务包、任务记忆包、权限、readback、失败、audit、evidence、handoff 和 director review 的摘要 / 引用，让项目主管能看懂“为什么停下、谁能处理、下一步是什么”，但不把详情面板变成治理后台或真实执行入口。  
执行方式：允许最小产品代码改动、测试、evidence 和 handoff；不得执行真实 Codex、不得读写 `/Users/yoyi/.codex`、不得写 workflow state 顶层结构、不得启动真实 worker。

完成记录：

- Evidence：`evidence/2026-06-06-stage-f-f2-workflow-node-detail-drawer-and-evidence-surface-v1.md`
- Handoff：`handoffs/2026-06-06-stage-f-f2-workflow-node-detail-drawer-and-evidence-surface-v1-result.md`
- 结论：接受为 F2 节点详情 / evidence surface 产品化完成；不接受为 F3、真实执行、runtime log、diagnostics、真实 Tauri 验收、阶段 F 完成或中间版本最终验收完成。

## 0. 先说薄弱点

- F1 已经把 `ProjectWorkflowCanvasReadModel`、`status_reason` 和 `attention_items[]` 收敛完成；F2 不应重新发明画布读模型，也不应重写主画布。
- 现有项目页右侧详情已经有很多卡片，F2 的风险是继续堆信息，变成治理后台、证据路径大表或任务包管理器。
- F2 名字里有 drawer / evidence surface，容易被误解成要新增全局 evidence 中心；本任务只在项目工作流节点详情上下文里做 evidence / handoff / audit 引用摘要，不新增一级入口或右侧顶级入口。
- 权限和失败信息必须解释“为什么停下、谁能处理、下一步是什么”，但不能在详情里绕过确认弹层和控制核心批准高风险动作。
- F2 不能继承 E5 Level B 的真实 `codex exec resume` 授权；后续任何新的真实 `codex exec resume`、真实 prompt、readback 或 `/Users/yoyi/.codex` 读写都必须另行授权。
- F2 仍不是 G 阶段；真实 Tauri / 截图验收、runtime log、diagnostics 不能在 F2 中冒领完成。

## 1. 已知事实 / 未知 / 假设

已知事实：

- 阶段 C1-C6 已完成，接受为自动化工作流受控闭环完成，但不等于真实 worker 产品化完成。
- 记忆层 M1-M13 已完成，M13 结论为 `accepted_with_deferred_items`。
- 阶段 E / E1-E7 已完成，E7 结论为 `accepted_with_deferred_items`。
- E5 Level B mario test 健康探针已完成，但只接受为指定 session 的最小真实 resume 健康探针。
- F1 已完成：`tasks/2026-06-06-stage-f-f1-project-workflow-canvas-read-model-consolidation-v1.md`。
- F1 evidence / handoff 明确 F2 可以继承 `ProjectWorkflowCanvasReadModel.status_reason`、`attention_items[]`、节点详情中的 task package / memory packet / permission / readback / audit / evidence / handoff 摘要结构，以及 React Flow 只读渲染边界。
- `docs/plans/2026-06-06-stage-e-f-g-refinement-plan-v1.md` 已把 F2 定义为 Node Detail Drawer For Task Package / Memory / Permission / Readback / Audit。

未知：

- 现有 `ProjectCanvasNodeDetail` 的 section / item 结构是否足够支持三层详情：用户摘要、项目主管信息、技术详情。
- 现有侧栏是否需要真正抽屉化，还是在既有右侧节点详情面板内完成层级改造即可。
- 现有 task package / memory packet / permission / readback / audit / evidence / handoff 摘要字段是否足够；如果不足，优先前端纯派生，必要时才扩展后端 read model。
- 当前真实 Tauri / 浏览器截图工具是否可用；如果不可用，必须在 evidence / handoff 写清。

本任务采用的假设：

- F2 默认不新增持久 sidecar，不迁移数据库，不改 workflow state JSON 顶层结构或状态枚举。
- F2 优先复用 F1 的 `ProjectWorkflowCanvasReadModel` 和既有 `ProjectCanvasSidePanel` / `ProjectCanvasNodeDetailView`。
- F2 可以新增前端纯读类型、分层详情 helper、折叠/展开 UI、摘要组件和离线测试。
- 如果执行者发现必须新增后端正式 read model，应把范围收窄为纯派生字段，不新增 store / command；如必须新增持久化或 workflow state 结构，必须停下回传。
- F2 不新增一级入口、右侧顶级入口或项目页 tab；只改既有项目页工作流节点详情上下文。

## 2. 任务目标

完成阶段 F 第二刀：

```text
F1 ProjectWorkflowCanvasReadModel
+ task package summary / artifact refs
+ task memory packet included / excluded / review reasons
+ permission / failure / readback boundary
+ audit / evidence / handoff / director review refs
-> Node Detail Drawer / Side Panel hierarchy
-> user summary / project director / technical details
-> tests + evidence + handoff
```

F2 完成后可以说：

- 项目工作流节点详情层级产品化完成。
- 用户能在节点详情中先看到用户摘要，再看到项目主管信息，最后按需展开技术详情。
- 任务包只显示摘要、目标、状态、artifact / evidence / handoff 引用，不把任务包管理器铺进主界面。
- 任务记忆包能显示 included / excluded / review materials 及理由，但不把记忆系统后台塞进节点详情。
- 权限、失败、readback unavailable / failed 能说明为什么停下、谁能处理、下一步建议是什么。
- audit / evidence / handoff 只显示引用、摘要和状态，不显示全文。

F2 完成后仍不能说：

- 画布编辑器完成。
- 真实 worker / Codex 执行完成。
- 真实 send / resume 产品化完成。
- 自动派发产品化完成。
- 高风险权限可在详情里直接批准。
- runtime log / diagnostics 完成。
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

F1 前置：

- `tasks/2026-06-06-stage-f-f1-project-workflow-canvas-read-model-consolidation-v1.md`
- `evidence/2026-06-06-stage-f-f1-project-workflow-canvas-read-model-consolidation-v1.md`
- `handoffs/2026-06-06-stage-f-f1-project-workflow-canvas-read-model-consolidation-v1-result.md`

相关前置：

- `tasks/2026-06-04-workflow-c4-project-director-task-decomposition-and-authorized-prepared-auto-dispatch-v1.md`
- `tasks/2026-06-04-workflow-c5-worker-structured-report-process-fact-confirmation-and-failure-visibility-v1.md`
- `tasks/2026-06-05-workflow-c6-global-final-result-review-user-result-view-and-stage-c-acceptance-v1.md`
- `tasks/2026-06-04-memory-layer-m4-task-memory-packet-builder-and-preview-v1.md`
- `tasks/2026-06-04-memory-layer-m6-workflow-task-package-injection-and-end-to-end-loop-v1.md`
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
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`

搜索固定文本必须用 `rg -F '...'` 或单引号，禁止用 shell 双引号包住未转义反引号。

## 4. 范围

允许：

- 扩展 `ProjectCanvasNodeDetail`、`ProjectCanvasDetailSection`、`ProjectCanvasDetailItem` 或等价前端纯读模型。
- 新增详情层级：
  - 用户摘要：当前状态、为什么停下、下一步、是否需要用户处理。
  - 项目主管信息：任务包摘要、权限、readback、失败、记忆包、director review。
  - 技术详情：audit / evidence / handoff 引用、source refs、attempt / dispatch 摘要、warning。
- 在既有项目页右侧详情面板中引入局部抽屉 / 折叠 / 分组 UI；若现有侧栏足够，也可以不新增视觉抽屉，只完成详情层级。
- 显示任务包摘要、目标、artifact refs、evidence refs、handoff refs、状态和更新时间。
- 显示任务记忆包 included / excluded / review materials 的理由、数量和来源引用。
- 显示权限和失败信息：
  - 为什么停下
  - 谁能处理
  - 下一步是什么
  - 是否阻塞继续
  - 是否必须走确认弹层 / 控制核心
- 显示 readback failed / readback unavailable 的区别，且不得显示成真实 0 条结果。
- 显示 audit / evidence / handoff 引用和摘要；可显示路径引用，但不显示全文。
- 增加离线测试覆盖详情层级、折叠/展开、禁止全文、readback unavailable 文案、权限说明和 evidence / handoff 引用。
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
- 不迁移数据库。
- 不新增持久 sidecar。
- 不新增真实 worker dispatch。
- 不启动四角色完整工作流。
- 不把 E5 Level B 单 session 健康探针扩大为通用会话控制能力。
- 不做 React Flow 编辑器、拖拽保存、连线保存、节点新增 / 删除、布局保存或复杂编辑器。
- 不把详情面板做成治理后台。
- 不把任务包管理器铺进项目主界面。
- 不在详情里直接批准高风险权限；高风险动作必须走确认弹层和控制核心。
- 不新增一级入口、右侧顶级入口或项目页 tab。
- 不展示任务包全文、audit 全文、transcript / rollout 全文、raw workflow state、raw sidecar、raw log、数据库路径大表或内部 schema。
- 不把 readback unavailable 显示成真实 0 条结果。
- 不把 observation、candidate、knowledge hit、runtime attention 或 LLM 摘要写成正式事实 / 正式记忆。
- 不把 GEPA / Paseo / Odysseus 研究项并入本任务实现。

## 5. 节点详情层级要求

F2 的节点详情必须按层级显示，不允许把所有信息平铺：

### 5.1 用户摘要

必须回答：

- 当前节点是什么。
- 当前状态是什么。
- 为什么停下或为什么需要注意。
- 下一步建议是什么。
- 是否需要用户处理。

允许显示：

- 一句话状态说明。
- 1-3 个关键 badge。
- “下一步”提示。

禁止显示：

- raw audit。
- raw task package。
- raw transcript。
- raw workflow state。

### 5.2 项目主管信息

必须回答：

- 关联的 work item / task package 是什么。
- 任务包摘要和 artifact refs 是什么。
- 任务记忆包 included / excluded / review materials 及理由是什么。
- 权限 / readback / 失败 / director review 当前状态是什么。
- 谁能处理阻断或下一步。

允许显示：

- task package summary。
- memory packet summary。
- permission / readback / failure cards。
- evidence / handoff links。
- director review summary。

禁止显示：

- 任务包全文。
- 记忆 store 原始记录全文。
- audit 事件全文。
- transcript / rollout 全文。

### 5.3 技术详情

必须回答：

- 事实来源和 source refs 是什么。
- audit / evidence / handoff 引用是什么。
- dispatch / attempt / run check 的技术摘要是什么。
- warnings 是什么。

允许显示：

- 可折叠 source refs。
- audit id / evidence path / handoff path 引用。
- dispatch / attempt 摘要。
- warning 列表。

禁止显示：

- token、secret、`.env`、provider credential。
- raw logs。
- raw sidecar。
- 大段 JSON。

## UI 显示边界确认

本任务是否改前端：

- [ ] 不改前端、不改读模型、不改 UI 文案。
- [ ] 改前端类型 / Tauri wrapper，但不新增可见 UI。
- [x] 改读模型摘要或状态显示。
- [x] 改已有页面局部 UI。
- [x] 新增入口、面板、tab、按钮或确认动作（仅允许既有项目页工作流画布上下文内的节点详情抽屉 / 分层面板；不新增一级入口、右侧顶级入口、项目页 tab、高风险操作按钮或确认动作）。

说明：这里的“新增入口 / 面板”只允许是项目页既有工作流画布上下文内的节点详情抽屉 / 分层面板；折叠 / 展开可以是局部展示控件，但不得变成执行、批准、派发、resume、重试、stop 或 restart 按钮。任何需要新增高风险确认动作或执行按钮的需求都必须停下并另拆任务包。

已读取：

- `docs/workbench-frontend-display-boundary-v1.md`
- `CURRENT.md`
- `tasks/README.md`
- `docs/plans/2026-06-01-project-workflow-canvas-node-schema-v1.md`

本任务允许显示：

- 节点详情中的用户摘要、项目主管信息和技术详情。
- 任务包摘要、artifact / evidence / handoff 引用。
- 任务记忆包 included / excluded / review materials 理由。
- 权限、readback、失败、director review、dispatch、attempt 和 audit 摘要。
- “为什么停下、谁能处理、下一步是什么”的用户可理解文案。

本任务禁止显示：

- 任务包全文。
- audit 全文。
- transcript / rollout 全文。
- raw workflow state / raw sidecar / raw log。
- `/Users/yoyi/.codex` 路径内容、token、secret、`.env`、provider credential。
- “worker 已执行”“Codex 已收到任务”“自动派发已开始”“自动重试已完成”“runtime log 已完成”“阶段 G 已验收”“节点详情已完成”等未完成或误导文案。

显示位置：

- 一级入口：不新增；继续使用 `项目`。
- 右侧入口：不新增；不改秘书 / 通知 / 待办 / 运行中 / 管理入口。
- 项目页：只改既有项目工作流画布节点详情上下文。
- 画布：不改主画布事实源；不使用独立实验画布作为事实源。
- 记忆入口：不改。
- 知识库入口：不改。
- 智能体入口：不改。
- 管理入口：不改。

中间版本范围：

- 本轮必须落地：节点详情分层、任务包 / 记忆包 / 权限 / readback / audit / evidence / handoff 摘要。
- 本轮只做读模型 / 摘要：source refs、dispatch、attempt、director review、warnings。
- 本轮后置：画布编辑、权限高风险直接处理、runtime log、diagnostics、真实 Tauri 全面验收。

后端和数据依赖：

- 需要后端正式读模型：优先复用 `ProjectWorkflowCanvasReadModel`、`WorkflowStateSnapshot.project_workflows[]`、task package、memory packet、permission、readback、runtime attention 和 audit 摘要；如不足，新增纯派生字段，不新增 store。
- 需要审计 / 日志 / 权限 / 状态机：只显示已有 audit / permission / control_core / workflow state 摘要；不伪造日志。
- 不能用假数据伪装：不能 hardcode 成功态，不能把 unavailable 写成 0，不能把 planned adapter 写成可执行。

UI 文案边界：

- 禁止说：`worker 已执行`、`Codex 已收到任务`、`自动派发已开始`、`自动重试已完成`、`runtime log 已完成`、`阶段 G 已验收`、`通用 send/resume 已完成`。
- 允许说：`等待权限`、`等待回收`、`readback 不可用`、`仅显示摘要`、`引用 evidence / handoff`、`需走确认弹层`、`React Flow 仅负责渲染`。

验收：

- 类型检查：必须跑 `npm run typecheck`。
- 离线交互测试：必须跑 `npm run test:offline-interaction`，新增或更新 F2 覆盖。
- 构建：必须跑 `npm run build`。
- Rust：如果改 Rust，必须跑相关 `cargo test --lib ...` 和 `cargo test --lib`，并跑对应 `rustfmt --check ...`。
- 真实窗口 / 截图验收：如可用，做项目页节点详情 smoke 和截图；如不可用，必须写入 evidence / handoff。
- 未验收项必须写入 evidence / handoff。

## 6. 建议实现顺序

1. 复核 F1 后的 `ProjectWorkflowCanvasReadModel`、`ProjectCanvasNodeDetail`、`ProjectCanvasDetailSection` 和 `ProjectCanvasDetailItem`。
2. 定义节点详情三层结构：用户摘要、项目主管信息、技术详情。
3. 把 task package / memory packet / permission / readback / failure / audit / evidence / handoff 摘要映射到对应层级。
4. 在 `ProjectsView.tsx` 既有项目画布侧栏内实现分层展示 / 折叠，不新增全局入口。
5. 确认高风险动作仍走 `PermissionDialog` / control_core，不在详情中直接批准。
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
rg -n "worker 已执行|Codex 已收到任务|自动派发已开始|自动重试已完成|runtime log 已完成|阶段 G 已验收|通用 send/resume 已完成" prototypes/productized-desktop-shell/src
```

详情边界扫描：

```text
rg -n "任务包全文|audit 全文|transcript 全文|raw workflow state|raw sidecar|raw log|0 条结果|0 条读回" prototypes/productized-desktop-shell/src prototypes/productized-desktop-shell/tests
```

浏览器 / Tauri：

- 如果 Browser / Playwright / Tauri 工具可用，打开项目页，确认节点详情分层可见、折叠可用、摘要可读、console 无新增 error。
- 如果不可用，至少做 Vite HTTP smoke，并在 evidence / handoff 写清不接受为真实窗口验收。

## 8. Evidence / Handoff 要求

完成后新增：

- `evidence/2026-06-06-stage-f-f2-workflow-node-detail-drawer-and-evidence-surface-v1.md`
- `handoffs/2026-06-06-stage-f-f2-workflow-node-detail-drawer-and-evidence-surface-v1-result.md`

Evidence 必须写清：

- F2 实际改了哪些读模型 / UI / 类型 / 测试。
- 是否改 Rust；如果改了，改动边界和测试结果。
- 节点详情三层结构如何实现。
- task package / memory packet / permission / readback / failure / audit / evidence / handoff 摘要如何展示。
- 是否新增入口 / tab / 全局面板；如果没有，要明确写没有。
- 高风险动作是否仍走确认弹层和控制核心。
- 禁止文案扫描结果。
- 是否做真实窗口 / 截图验收。
- 本轮不接受为什么。

Handoff 必须写清：

- F2 是否可接受为“节点详情 / evidence surface 产品化完成”。
- F3 是否可以开始。
- F2 仍不能接受为哪些能力完成。
- 遗留风险和建议。

## 9. 回收口径

完成后可接受为：

- F2 节点详情 / evidence surface 产品化完成。
- 节点详情按用户摘要、项目主管信息、技术详情分层展示。
- 任务包、任务记忆包、权限、readback、失败、audit、evidence、handoff 和 director review 以摘要 / 引用方式进入节点详情。
- 高风险动作仍走确认弹层和控制核心。

完成后不接受为：

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

## 10. Stop 条件

遇到以下情况必须停下：

- 需要执行真实 `codex exec` / `codex exec resume`。
- 需要发送真实 prompt。
- 需要读写 `/Users/yoyi/.codex`。
- 需要读取完整 transcript / rollout。
- 需要读取 secret、token、`.env`、keychain、OAuth、provider credential。
- 需要改 workflow state 顶层结构或状态枚举。
- 需要新增持久 sidecar 或数据库迁移。
- 需要新增真实派发、自动重试或 runtime log store。
- 需要新增一级入口、右侧顶级入口或项目页 tab。
- 需要把详情面板做成治理后台。
- 需要在详情中直接批准高风险权限。
- 需要把任务包 / audit / transcript / raw state 全文铺到详情面板。
