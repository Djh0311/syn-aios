# Workbench Architecture Readonly Audit v1

日期：2026-06-01

## 1. 本轮边界

已执行范围：

- 只做架构只读审计。
- 只新增本 evidence 和对应 handoff。
- 没有改代码。
- 没有运行真实 `codex exec` / `codex exec resume`。
- 没有读取 `/Users/yoyi/.codex`。
- 没有读取 auth、token、`.env`、完整 transcript。
- 没有写真实业务项目目录。
- 没有迁移数据库。

依据：

- `docs/plans/2026-06-01-workbench-architecture-implementation-plan-v1.md:566-599` 要求 Task A 只读审计，写范围是 `evidence/**`、`handoffs/**`，禁止改代码、读 `/Users/yoyi/.codex`、执行 Codex。
- 本轮用户明确要求只执行“任务 A：架构只读审计”，不改代码、不跑真实 Codex、不读 `/Users/yoyi/.codex`、不迁移数据库。

已知：

- 当前主线是 Codex 会话管理 + Codex 工作流编排，不是任务包管理器。依据：`CURRENT.md:3-11`。
- 当前架构目标已经定义界面层、应用服务层、控制核心、项目黑板、事实层、适配器层、读模型。依据：`docs/workbench-system-architecture-v1.md:116-177`、`docs/workbench-system-architecture-v1.md:189-263`、`docs/workbench-system-architecture-v1.md:476-496`。
- 当前架构计划已经点名 `lib.rs` 过宽、`ProjectsView.tsx` 任务包倾向、`CanvasView.tsx` 和项目画布权威不清。依据：`docs/plans/2026-06-01-workbench-architecture-implementation-plan-v1.md:5-19`。

未知：

- 项目工作流画布和独立可编辑画布最终是否合并，当前文档只把它列为未知和后续决策点。依据：`docs/plans/2026-06-01-workbench-architecture-implementation-plan-v1.md:33-39`。
- 记忆第一阶段存储方式未定。依据：`docs/plans/2026-06-01-workbench-architecture-implementation-plan-v1.md:33-39`。
- 当前审计没有运行应用，所以不判断真实窗口视觉表现。

假设：

- 新增 `evidence/**` 和 `handoffs/**` 是本轮允许写入，不属于“写真实业务项目目录”。
- 在没有新迁移计划前，继续把 JSON workflow state 看作过渡事实层。依据：`docs/plans/2026-06-01-workbench-architecture-implementation-plan-v1.md:41-47`。

## 2. 已读范围

当前入口：

- `CURRENT.md`
- `AUTHORITY.md`
- `docs/plans/2026-06-01-workbench-architecture-implementation-plan-v1.md`
- `docs/workbench-system-architecture-v1.md`
- `docs/workflow-task-package-design-v1.md`
- `docs/memory-layer-design-v1.md`

只读代码：

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/mcp/**`
- `prototypes/productized-desktop-shell/src/App.tsx`
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `prototypes/productized-desktop-shell/src/views/CanvasView.tsx`
- `prototypes/productized-desktop-shell/src/lib/types.ts`
- 额外只读 `prototypes/productized-desktop-shell/src/lib/tauri.ts`，因为前端通过它调用 Tauri 命令。

代码规模：

| 文件 | 行数 | 判断 |
|---|---:|---|
| `src-tauri/src/lib.rs` | 13076 | 单文件承担职责过多，是后续拆分主风险区。 |
| `src/views/ProjectsView.tsx` | 2712 | 项目页仍把大量工作流、任务包、回收、状态机面板放在一个文件里。 |
| `src/views/CanvasView.tsx` | 518 | 独立可编辑画布较集中，但和项目工作流画布权威关系未定。 |
| `src/App.tsx` | 809 | 应用入口和确认动作集中区。 |
| `src/lib/types.ts` | 992 | 前端类型聚合，含 workflow 和 editable canvas 两套模型。 |
| `src/lib/tauri.ts` | 220 | 前端到 Tauri 的应用服务调用封装。 |
| `src-tauri/src/mcp/storage.rs` | 330 | 独立画布存储模型和文件读写。 |
| `src-tauri/src/mcp/orchestrator.rs` | 311 | 独立画布运行循环，会启动 Codex。 |
| `src-tauri/src/mcp/tools.rs` | 492 | MCP 工具直接读写独立画布 run state 和 audit。 |

依据：本轮只读命令 `wc -l` 输出。

## 3. 当前代码到架构层总表

| 架构层 | 当前代码位置 | 当前状态 | 风险判断 | 依据 |
|---|---|---|---|---|
| 界面层 | `App.tsx`、`ProjectsView.tsx`、`CanvasView.tsx` | 已有主导航、项目页、项目工作流画布、独立可编辑画布、确认弹层和右侧运行入口。 | 界面层已经存在，但 `ProjectsView.tsx` 里仍直接拼复杂 workflow state，容易越过读模型边界。 | 界面层职责见 `docs/workbench-system-architecture-v1.md:118-137`；`ProjectsView.tsx:1502-2020` 直接渲染 derived workflow、账本、报告、审查、异常、状态机、接口边界、验收场景。 |
| 应用服务层 | `lib.rs` 的 `#[tauri::command]` 区、`src/lib/tauri.ts` | 已有命令入口和前端调用封装。 | 入口存在，但命令、领域规则、状态读写、执行器混在同一文件里，层边界不清。 | 应用服务层职责见 `docs/workbench-system-architecture-v1.md:139-159`；`lib.rs:1082-1591` 集中 Tauri 命令；`src/lib/tauri.ts:60-196` 封装 workflow 和 canvas 命令。 |
| 控制核心 | `lib.rs` 中状态机、派发校验、完成闸门、角色流转、工作流机器 | 已有部分真实规则。 | 高风险区。无行为变化拆分可以搬位置，但不能改规则或顺手抽象产品语义。 | 控制核心职责见 `docs/workbench-system-architecture-v1.md:161-177`；`lib.rs:4889` 起有 `run_workflow_machine_at`；`lib.rs:6829` 起有 `derive_workflow_read_model`；`lib.rs:3019` 起有 work item 状态更新。 |
| 项目黑板 | 当前没有独立黑板模块；workflow state 中散落 work items、reports、reviews、exceptions、audit 等中间态 | 未明确落地。 | 缺口明显。现在很多中间态直接进 workflow state，未体现“候选/请求先到黑板，经控制核心升级”的边界。 | 黑板定义见 `docs/workbench-system-architecture-v1.md:189-212`；当前 `lib.rs:254-450` 定义 Workflow、TaskPackage、Ledger、Exception 等结构，但未见独立 blackboard 模型。 |
| 事实层 | `workflow-state.v0.json` 读写逻辑、MCP canvas JSON/JSONL 文件 | 已有 v0 JSON 事实层和独立画布文件层。 | 事实层存在但没有拆成事件账本、当前快照、审计账本三个模块；独立画布文件层也未归并到项目事实规则。 | 事实层定义见 `docs/workbench-system-architecture-v1.md:213-235`；`lib.rs:6076` 读 workflow state；`mcp/storage.rs:121-235` 管 canvas、run state、audit 文件。 |
| 适配器层 | `lib.rs` 中 Codex resume runner 相关逻辑；`src-tauri/src/mcp/codex_runner.rs`；MCP tools | 已有 Codex-only 路径和 MCP 画布工具路径。 | 不统一。长期架构要求外部能力都经适配器声明能力、接收受控命令、返回结果；当前 Codex、MCP、workflow dispatch 分散在不同路径。 | 适配器规则见 `docs/workbench-system-architecture-v1.md:237-263`；`lib.rs:897` 起直接构造 `codex` 命令；`mcp/codex_runner.rs:28`、`mcp/codex_runner.rs:60` 会启动 director/subagent。 |
| 读模型 | `derive_workflow_read_model`、`WorkbenchSnapshot`、前端 `WorkflowStateSnapshot` 类型 | 已有项目工作流读模型雏形。 | 方向对，但 UI 仍自己组装很多底层状态，读模型还不够稳定。 | 读模型规则见 `docs/workbench-system-architecture-v1.md:476-496`；`lib.rs:6829` 派生 workflow read model；`types.ts:173-665` 定义 WorkbenchSnapshot 和 WorkflowStateSnapshot。 |

## 4. 后端模块拆分表

这张表只判断后续能否“无行为变化拆模块”，不建议在 Task A 中改代码。

| 当前位置 | 现有职责 | 建议目标模块 | 能否无行为变化拆出 | 风险 | 依据 |
|---|---|---|---|---|---|
| `lib.rs:202-450` | Workbench、Workflow、TaskPackage、Ledger、Exception、InterfaceBoundary、StateMachine、AcceptanceScenario 等结构体 | `domain/workflow.rs`、`domain/task_package.rs`、`read_model/types.rs` | 可以先搬类型 | 中 | 类型搬迁低风险，但要保持 serde 字段不变。 |
| `lib.rs:654-846` | 请求/响应结构体 | `commands/types.rs` 或 `api/types.rs` | 可以先搬类型 | 中 | 前端 Tauri invoke 依赖字段名，字段不能改。 |
| `lib.rs:1082-1591` | Tauri command 入口 | `commands/workflow.rs`、`commands/snapshot.rs`、`commands/dispatch.rs` | 可以拆命令包装层 | 中 | 入口名字必须保持不变，`tauri::generate_handler!` 要保留同名命令。 |
| `lib.rs:1679-1788` | workflow state 初始化、备份、写当前快照 | `storage/workflow_state.rs` | 可拆，但要加回归测试 | 高 | 直接影响事实层写入和备份。 |
| `lib.rs:1853-1976` | 默认项目工作流结构写入 | `application/bootstrap_project_workflow.rs` | 可拆实现位置，不改 JSON 结构 | 高 | 这是项目工作流默认事实结构，改错会影响后续读模型。 |
| `lib.rs:1978-2240` | 任务包草稿、预览、字段更新 | `application/task_package_service.rs` | 可拆，但不要改产品规则 | 高 | 当前主线不是任务包管理器，拆分时容易顺手强化任务包中心。依据：`CURRENT.md:3-11`、`docs/workbench-system-architecture-v1.md:440-447`。 |
| `lib.rs:3019-3288` | work item 状态更新、节点绑定/解绑 | `core/workflow_state_machine.rs`、`application/session_binding.rs` | 只能搬位置，不改规则 | 高 | 属控制核心和项目身份/会话归属边界。 |
| `lib.rs:3391-4272` | 节点派发准备、执行、读回、Codex resume 失败分类 | `application/dispatch_service.rs`、`adapters/codex_adapter.rs` | 拆分需很保守 | 高 | 会调用 Codex resume，涉及权限、审计、外部写入。 |
| `lib.rs:4455-4735` | 离线角色派发、handoff、总指导 review | `application/offline_orchestration.rs` | 可拆，但不能改状态推进语义 | 高 | 影响当前角色编排闭环。 |
| `lib.rs:4889-5310` | 四角色工作流机器 | `core/workflow_machine.rs`、`application/workflow_machine_service.rs` | 不建议第一批拆 | 很高 | 会串行调用绑定 Codex 会话并推进状态，属于真实执行路径。 |
| `lib.rs:6076` 附近 | workflow state 读写基础函数 | `storage/json_store.rs` | 可拆纯存储，但必须保持原子写/备份行为 | 高 | 事实层底座，不能改变路径、备份、写入时机。 |
| `lib.rs:6265-6414` 附近 | 状态流转和节点映射 | `core/state_transition.rs` | 不建议第一批拆 | 很高 | 属控制核心，任何细微行为变化都会改业务状态。 |
| `lib.rs:6584-7565`、`lib.rs:6829` | workflow read model、完成闸门、边界、验收展示派生 | `read_model/workflow.rs` | 可以作为第一批拆出候选 | 中 | 读模型可重建，风险低于事实写入。依据：`docs/workbench-system-architecture-v1.md:492-496`。 |
| `lib.rs:8133-8745` | WorkbenchSnapshot、会话、项目、skills/plugins、harness 解析和路径白名单 | `read_model/workbench_snapshot.rs`、`adapters/codex_index.rs`、`security/path_policy.rs` | 可分批拆 | 中到高 | 快照读模型可拆；路径白名单和敏感路径规则不能改。 |
| `src-tauri/src/mcp/storage.rs:50-235` | 独立 canvas/run/audit 文件模型和读写 | `mcp/storage.rs` 已拆 | 暂不再拆，先等画布权威 decision | 高 | 独立画布事实源与项目 workflow state 权威关系未定。 |
| `src-tauri/src/mcp/orchestrator.rs:71-245` | 独立画布运行循环、后台线程、调用 Codex runner | 已独立但需收敛到适配器/控制核心 | 暂不拆行为 | 很高 | 会启动 Codex，且像通用节点执行器。 |
| `src-tauri/src/mcp/tools.rs:168-450` | director/subagent MCP 工具读写 run state 和 audit | 已独立但绕开项目控制核心 | 暂不扩展 | 很高 | 工具直接操作独立画布 run state，后续需纳入适配器边界。 |

## 5. 前端组件归属表

| 位置 | 当前职责 | 归属层 | 风险判断 | 依据 |
|---|---|---|---|---|
| `App.tsx:110-273` | 全局 pending action 确认和调用后端 mutation | 界面层 + 应用服务入口调用 | 中 | 适合保留为 UI 确认入口，但业务规则不应继续堆在这里。 |
| `App.tsx:671-706` | 在 projects 和 workflow 两个页面间切换，分别渲染 `ProjectsView` 和 `CanvasViewWithProvider` | 界面层 | 高 | 两个画布入口并存，权威关系不清。 |
| `src/lib/tauri.ts:60-196` | 前端 Tauri invoke 封装 | 应用服务调用封装 | 中 | 它是前端调用边界，不应承载业务判断。 |
| `ProjectsView.tsx:37-50` | 项目页 tool key，类型仍包含 `task-packages` | 界面层 | 中 | 可见 tabs 已不列 `task-packages`，但类型和分支仍在。 |
| `ProjectsView.tsx:233-340` | ProjectDetail 分支含 workflow 和 task-packages | 界面层 | 中到高 | `task-packages` 分支当前未直接显示，但代码还保留入口。 |
| `ProjectsView.tsx:686-1038` | 任务包草稿、选择、预览、生成文件 | 界面层，但内容偏应用服务操作台 | 高 | 任务包管理器倾向最强。 |
| `ProjectsView.tsx:1502-1609` | 项目级 WorkflowCanvas，读取 `workflowState` 和 `derived_workflow` | 项目工作流界面层/读模型消费者 | 中 | 方向符合项目工作流，但 UI 仍直接拼较多状态。 |
| `ProjectsView.tsx:1708-2020` | DerivedWorkflowSummary、蓝图画布、账本、报告、审查、异常、状态机、接口边界、验收场景 | 读模型展示层 | 中到高 | 展示信息完整，但平铺过多，会让项目页像内部协议调试台。 |
| `ProjectsView.tsx:2023-2335` | WorkItemOrchestrationCard，绑定会话、派发、运行机器、总指导回收 | 界面层 + 应用服务入口调用 | 高 | UI 内承载大量业务可用性判断，容易越过读模型边界。 |
| `CanvasView.tsx:52-103` | CanvasDefinition 和 React Flow 的互转 | 独立可编辑画布界面层 | 中 | 可作为纯 UI 转换，但不应成为项目 workflow state 的第二事实源。 |
| `CanvasView.tsx:105-275` | 加载、保存、开工、拍停、轮询 run 状态 | 界面层 + 独立 canvas 应用服务入口 | 高 | 这里操作独立 canvas run，不读项目 workflow state。 |
| `CanvasView.tsx:323-391` | 独立可编辑画布主 UI | 界面层 | 高 | 和项目工作流画布功能相似，需明确权威。 |
| `CanvasView.tsx:406-470` | 节点详情编辑，可挂 Codex 会话 | 界面层 | 高 | 节点身份和会话归属未来应受项目控制核心约束。 |
| `types.ts:183-865` | workflow/read model/commands 类型 | 前端类型层 | 中 | 可按领域和命令分文件，无行为变化风险较低。 |
| `types.ts:865-988` | editable canvas 类型 | 前端类型层 | 中 | 和 workflow 类型并存，说明两个画布模型未统一。 |

## 6. `lib.rs` 可后续无行为变化拆出的内容

可以作为第一批低风险拆分候选：

| 内容 | 建议拆法 | 注意事项 | 依据 |
|---|---|---|---|
| 领域和读模型结构体 | 先拆到 `domain/*`、`read_model/*` | 不改字段名、不改 serde 行为、不改默认值。 | `lib.rs:202-846` 聚合大量结构体。 |
| Tauri command 包装函数 | 拆到 `commands/*`，保留命令名和请求响应类型 | `generate_handler!` 中命令名不能变。 | `lib.rs:1082-1591` 是命令包装集中区；`lib.rs:8742-8765` 注册命令。 |
| 前端 `types.ts` | 按 snapshot/workflow/canvas/commands 分文件后再 barrel export | 不改导出类型名，避免前端大面积改 import。 | `types.ts:173-989` 同时含 workflow 和 canvas。 |
| workflow read model 派生函数 | 拆到 `read_model/workflow.rs` | 读模型可重建，优先于事实写入拆分。 | 读模型边界见 `docs/workbench-system-architecture-v1.md:492-496`；`lib.rs:6829` 是派生入口。 |
| WorkbenchSnapshot 组装 | 拆到 `read_model/workbench_snapshot.rs` | 注意敏感路径和 transcript 边界不能放松。 | `lib.rs:1083` 命令入口；`lib.rs:8133-8745` 有快照和会话读取相关逻辑。 |
| MCP storage 类型 | 已在 `mcp/storage.rs` 中，可保持现状 | 不应在画布权威未定前继续扩大行为。 | `mcp/storage.rs:50-235`。 |

结论：

- 可以进入“无行为变化拆模块”，但第一批只建议拆类型、命令包装、读模型和快照组装。
- 不能把“拆模块”顺手变成“改产品规则”或“扩展可编辑画布执行能力”。

依据：

- 架构计划 Task B 目标是“把 `lib.rs` 按架构层拆成模块，保持命令行为不变”。依据：`docs/plans/2026-06-01-workbench-architecture-implementation-plan-v1.md:600-609`。
- 读模型不是事实源，可以重建。依据：`docs/workbench-system-architecture-v1.md:492-496`。

## 7. 高风险不可直接拆清单

| 高风险点 | 为什么不能直接拆 | 后续建议 | 依据 |
|---|---|---|---|
| workflow state 写入、备份、初始化 | 这是 v0 事实层底座，改错会损坏当前项目状态。 | 先补测试，再只搬函数位置。 | `lib.rs:1679-1788`、`lib.rs:6076`。 |
| 默认项目工作流 bootstrap | 它决定项目 workflow state 初始结构。 | 拆前先固定样例 JSON 快照。 | `lib.rs:1853-1976`。 |
| work item 状态流转 | 属控制核心，影响任务完成判定和 UI 状态。 | 先保留行为，后续单独做状态机测试。 | 控制核心职责见 `docs/workbench-system-architecture-v1.md:161-177`；代码在 `lib.rs:3019` 起。 |
| 节点派发执行和 Codex resume | 涉及真实外部执行、权限、审计、失败分类。 | 拆适配器边界前不改语义。 | `lib.rs:3391-4272`；外部能力不能直接写核心事实层，见 `docs/workbench-system-architecture-v1.md:111-114`。 |
| 四角色工作流机器 | 会串行调用绑定 Codex 会话并推进状态。 | 第一批拆分不要碰；后续单开任务。 | `lib.rs:4889-5310`。 |
| 任务包草稿和生成文件 | 产品方向要求任务包是内部协议，不是主界面中心。 | 可降级 UI，但不能顺手改真实任务包规则。 | `docs/workbench-system-architecture-v1.md:440-447`；`ProjectsView.tsx:686-1038`。 |
| 独立可编辑画布 run | 它有单独 canvas/run/audit 文件层，会启动 Codex，可能变成通用节点执行器。 | 先冻结扩展，等画布权威 decision。 | `mcp/orchestrator.rs:71-245`；工作流不是通用节点执行器见 `docs/workbench-system-architecture-v1.md:440-447`。 |
| MCP tools 直接写 run state/audit | 绕开项目控制核心和项目黑板规则。 | 后续要纳入适配器和控制核心边界。 | `mcp/tools.rs:168-450`；适配器不能直接推进工作流状态，见 `docs/workbench-system-architecture-v1.md:257-263`。 |

## 8. 画布权威风险判断

判断：当前有权威冲突风险，需要新 decision。

依据：

- 架构计划已经把“项目工作流画布和独立可编辑画布是否应合一，还是保留不同层级”列为未知。依据：`docs/plans/2026-06-01-workbench-architecture-implementation-plan-v1.md:33-39`。
- 架构计划明确当前 `CanvasView.tsx` 和 `ProjectsView.tsx` 里的项目工作流画布是两条路径，还没有确定谁是项目工作流画布的权威承载。依据：`docs/plans/2026-06-01-workbench-architecture-implementation-plan-v1.md:11-15`。
- 项目工作流画布读取 `workflowState` / `derived_workflow`。依据：`ProjectsView.tsx:1502-1519`。
- 独立可编辑画布读取和保存 `CanvasDefinition`，并能开工、拍停、轮询 run。依据：`CanvasView.tsx:52-103`、`CanvasView.tsx:105-275`。
- MCP 独立画布有自己的 canvas、run state、audit 文件层。依据：`mcp/storage.rs:50-235`。
- MCP orchestrator 会启动 director/subagent Codex。依据：`mcp/orchestrator.rs:209-245`。

建议 decision 内容：

- 项目工作流画布的权威状态应是项目 workflow state 和未来项目黑板，不应是独立 canvas 文件。
- 独立 `CanvasView` 可以暂定为实验性可编辑画布、模板编辑器或后置能力，不能默认成为项目工作流事实源。
- MCP canvas/run 文件层可以保留，但在接入项目工作流前必须经过控制核心、项目权限、审计和适配器边界。

不确定：

- 最终是“一套画布两种模式”还是“项目画布和实验画布长期分离”，当前文档没有定论。

## 9. `ProjectsView.tsx` 的任务包管理器倾向

判断：`ProjectsView.tsx` 仍有明显任务包管理器倾向，虽然可见 tabs 已把 `task-packages` 降级。

依据和表现：

| 位置 | 表现 | 风险 |
|---|---|---|
| `ProjectsView.tsx:37-50` | `ProjectToolKey` 仍包含 `task-packages`，但 `projectTools` 只列 overview/workflow/handoff/resources。 | 代码里保留隐藏入口，后续容易恢复成主入口。 |
| `ProjectsView.tsx:233-340` | `ProjectDetail` 仍保留 `selectedTool === "task-packages"` 分支。 | 任务包面板虽未直接暴露，但仍是产品结构的一部分。 |
| `ProjectsView.tsx:686-1038` | `ProjectWorkflowDraftPanel`、`TaskDraftSelectionController`、`TaskPreviewController`、`TaskFileGenerationController` 集中处理草稿、预览、生成文件。 | 这是任务包管理器倾向最强区域。 |
| `ProjectsView.tsx:1708-2020` | DerivedWorkflowSummary 下平铺任务包、账本、报告、审查、异常、状态机、接口边界、验收场景。 | 信息过多，像内部协议仪表盘，不像面向项目工作的主画布。 |
| `ProjectsView.tsx:2023-2335` | WorkItemOrchestrationCard 处理绑定会话、派发、业务可派发判断、运行机器、主管回收。 | UI 承担过多业务判断，读模型边界不够清楚。 |

产品规则依据：

- 当前主线不是任务包管理器。依据：`CURRENT.md:3-11`。
- 架构文档要求工作流不是任务包管理器，任务包是内部协议，不是主界面中心。依据：`docs/workbench-system-architecture-v1.md:440-447`。

## 10. 秘书和记忆治理第一阶段判断

判断：第一阶段应该只建模型、候选、读模型和确认边界；不应直接做“秘书自动改事实”或“真实写长期记忆”的闭环。

依据：

- 秘书是核心协作角色，不是某个界面。依据：`docs/workbench-system-architecture-v1.md:294-315`。
- 秘书不能绕过用户确认直接改系统事实，不能把聊天内容直接写入长期记忆。依据：`docs/workbench-system-architecture-v1.md:326-333`。
- 记忆治理可以进核心，但核心外包含向量检索、图谱索引、Obsidian 文件读取、摘要模型、相似度搜索、存储引擎细节。依据：`docs/workbench-system-architecture-v1.md:339-363`。
- 阶段 0 是对象和边界，必须完成 `MemorySourceRef`、`MemoryCandidate`、`MemoryPage`、`MemoryVersion`、`MemoryRelation`、`MemoryLintFinding`、`TaskMemoryPacket`、边界规则和权限字段。依据：`docs/memory-layer-design-v1.md:1496-1519`。
- 阶段 1 才支撑“工作流结束到项目记忆沉淀，再到下一次任务包召回”。依据：`docs/memory-layer-design-v1.md:1520-1535`。
- 记忆设计距离工程落地还缺落地切片、数据表、权限工程化、任务包算法、UI 详细规格、后台任务、验收测试用例。依据：`docs/memory-layer-design-v1.md:3094-3219`。

实现风险：

- 如果第一阶段就写正式记忆，会越过“候选、来源、版本、权限、冲突、确认”的治理边界。
- 如果秘书第一阶段直接改事实，会违反秘书不能绕过用户确认和项目主管的规则。
- 如果把 Obsidian、向量、图谱或摘要模型直接塞进核心，会违反“进入核心的是治理，不是存储/检索实现”的规则。

建议第一阶段只做：

- 记忆对象模型和状态字段。
- 记忆候选读模型。
- 项目/全局状态摘要读模型。
- 秘书建议候选列表。
- 用户确认/拒绝/冻结/废弃的命令边界草案。
- 审计事件模型草案。

不建议第一阶段做：

- 自动写正式记忆。
- 自动修改项目事实。
- 真实 Obsidian 文件写入。
- 向量库、图谱、相似度搜索。
- 让秘书直接调工具改项目。

## 11. 当前 app 偏离最终蓝图清单

| 偏离点 | 判断 | 依据 |
|---|---|---|
| 后端没有分层 | `lib.rs` 同时包含命令、结构体、状态读写、派发、工作流机器、读模型和测试。 | 架构计划已点名 `lib.rs` 边界过宽：`docs/plans/2026-06-01-workbench-architecture-implementation-plan-v1.md:11-15`；代码规模 13076 行。 |
| 项目黑板未显式落地 | 当前有 workflow state、task package、ledger/report/review/exception，但不是独立黑板对象。 | 黑板边界见 `docs/workbench-system-architecture-v1.md:189-212`；当前结构体集中在 `lib.rs:254-450`。 |
| 事实层未拆成事件账本、当前快照、审计账本 | JSON 文件中混放多类状态，`lib.rs` 直接读写。 | 事实层定义见 `docs/workbench-system-architecture-v1.md:213-221`；`lib.rs:6076` 读 workflow state。 |
| 适配器层不统一 | Codex resume、MCP canvas、workflow dispatch 分散，未统一为 AgentAdapter/ToolAdapter。 | 适配器定义见 `docs/workbench-system-architecture-v1.md:237-263`；`lib.rs:897`、`mcp/codex_runner.rs:28`。 |
| 画布权威不清 | 项目 workflow canvas 和独立 editable canvas 两套模型并存。 | `ProjectsView.tsx:1502-1519`、`CanvasView.tsx:52-103`、`types.ts:216-665`、`types.ts:865-988`。 |
| 项目页仍有任务包中心感 | 任务包草稿、预览、文件生成、派发 readiness、内部协议面板仍集中在项目页。 | `ProjectsView.tsx:686-1038`、`ProjectsView.tsx:1708-2020`；架构要求任务包不是主界面中心，见 `docs/workbench-system-architecture-v1.md:440-447`。 |
| 秘书未作为核心协作模型落地 | 当前审计范围内未见秘书核心模型、秘书建议候选、秘书权限边界实现。 | 秘书职责见 `docs/workbench-system-architecture-v1.md:294-333`；架构计划说秘书主要是设计文档能力，见 `docs/plans/2026-06-01-workbench-architecture-implementation-plan-v1.md:11-15`。 |
| 记忆治理未落地 | 目前未见 MemoryCandidate/Page/Version/Scope 等核心对象实现。 | 记忆阶段 0 对象见 `docs/memory-layer-design-v1.md:1500-1519`；当前 `types.ts:183-865` 主要是 workflow 类型。 |
| UI 仍拼复杂底层状态 | `ProjectsView.tsx` 直接组合 workflow state、derived workflow、director reviews、execution controls 等。 | 读模型要求 UI 不直接拼底层复杂状态，见 `docs/workbench-system-architecture-v1.md:492-496`；代码见 `ProjectsView.tsx:1502-2335`。 |
| 独立 canvas 有通用节点执行器风险 | 可编辑画布有 director/subagent 和 run loop，能启动 Codex。 | 工作流不是通用节点执行器，见 `docs/workbench-system-architecture-v1.md:440-447`；MCP run loop 见 `mcp/orchestrator.rs:209-245`。 |

## 12. 下一步是否可以进入“无行为变化拆模块”

判断：可以进入，但只建议进入保守的 Task B 第一段。

可以做：

- 拆类型文件。
- 拆 Tauri command 包装。
- 拆 workflow read model。
- 拆 WorkbenchSnapshot 组装。
- 拆纯路径/JSON 工具函数，但不改变行为。

暂时不要做：

- 改 workflow state JSON 结构。
- 改 work item 状态机。
- 改四角色工作流机器。
- 改 Codex runner 或 resume 参数策略。
- 改 MCP canvas run 行为。
- 扩大 `CanvasView` 的项目工作流权威。
- 把任务包管理器重新推到主界面中心。
- 写正式记忆或秘书自动改事实。

进入 Task B 前建议先补两个决策：

- 画布权威 decision：项目 workflow canvas 与独立 editable canvas 的层级关系。
- 拆分保护 decision：Task B 只允许无行为变化移动代码，不允许改状态机、Codex 执行、任务包产品规则。

## 13. 本轮未做事项声明

- 没有改任何 `src` 代码。
- 没有运行构建、测试、dev server 或 Tauri 窗口。
- 没有执行真实 Codex。
- 没有读取 `/Users/yoyi/.codex`。
- 没有读取 auth、token、`.env`、完整 transcript。
- 没有写真实业务项目目录。
- 没有迁移数据库。
