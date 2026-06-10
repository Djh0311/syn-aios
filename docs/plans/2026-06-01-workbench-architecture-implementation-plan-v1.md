# Workbench Architecture Implementation Plan v1

状态：架构落地执行计划草案。Task A 架构只读审计、Task B 保守拆模块切片、Task C 项目工作流画布权威收敛、Task D 项目黑板最小只读切片、Task E 控制核心命令收敛保守切片、控制核心第二切片边界拆分均已完成；当前建议不要继续扩大黑板写入，可继续做更小的审计/读模型拆分切片，或单开设计黑板候选持久确认状态的 schema / 迁移计划。本文只规划如何把当前 app 收敛到最终蓝图和软件架构，不等同于已完成实现。

当前进度：

- Task A：已完成。依据见 `evidence/2026-06-01-workbench-architecture-readonly-audit-v1.md` 和 `handoffs/2026-06-01-workbench-architecture-readonly-audit-v1-result.md`。
- Task B：保守切片已完成。依据见 `evidence/2026-06-01-workbench-architecture-task-b-conservative-module-split-v1.md` 和 `handoffs/2026-06-01-workbench-architecture-task-b-conservative-module-split-v1-result.md`。本轮只拆后端类型、Tauri command 包装和前端 editable canvas 纯类型；workflow 读模型和 WorkbenchSnapshot 组装未拆。
- Task C：项目工作流画布权威收敛保守切片已完成。依据见 `evidence/2026-06-01-project-workflow-canvas-authority-convergence-c-v1.md` 和 `handoffs/2026-06-01-project-workflow-canvas-authority-convergence-c-v1-result.md`。
- Task D：项目黑板最小只读切片已完成。依据见 `evidence/2026-06-01-project-blackboard-minimal-read-model-d-v1.md` 和 `handoffs/2026-06-01-project-blackboard-minimal-read-model-d-v1-result.md`。本轮只做模型和从现有 workflow state 派生的只读读模型，未新增黑板写入命令。
- Task E：控制核心命令收敛保守切片已完成。依据见 `evidence/2026-06-01-control-core-command-convergence-v1.md` 和 `handoffs/2026-06-01-control-core-command-convergence-v1-result.md`。本轮新增后端 `control_core.rs` helper，接入工作项状态推进、派发准备/启动/完成/失败、总指导回收、离线派发/回传/回收、工作流机器收口和权限确认；黑板候选只做边界校验，不写正式事实或正式记忆。
- 控制核心第二切片边界拆分已完成。依据见 `evidence/2026-06-01-control-core-second-slice-boundary-extraction-v1.md` 和 `handoffs/2026-06-01-control-core-second-slice-boundary-extraction-v1-result.md`。本轮新增 `workflow_state_store.rs`、`workflow_audit.rs`、`workflow_read_model.rs` 三个小模块；状态读写 wrapper、一类 `work_item_state_changed` audit 构造、项目黑板集合派生已接入新 helper；未改 workflow state JSON 结构。
- 画布权威：已补决策，见 `decisions/2026-06-01-project-workflow-canvas-authority-v1.md`。当前剩余风险是独立 `CanvasView` 仍保留实验运行能力，项目页内部任务包/账本/状态机面板仍未收进节点详情或右侧抽屉。

## 1. 先说薄弱点

当前最大风险不是“少一个功能”，而是功能继续堆在一起。

依据：

- `docs/workbench-system-architecture-v1.md` 已经定义了控制核心、项目黑板、适配器、事实层、读模型，但当前代码还没有按这些边界组织。
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs` 同时包含 Tauri 命令、数据结构、状态读写、派发、工作流机器、读模型和测试，边界过宽。
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx` 仍保留大量任务包和工作流细节面板，容易把产品带回“任务包管理器”。
- `prototypes/productized-desktop-shell/src/views/CanvasView.tsx` 和 `ProjectsView.tsx` 里的项目工作流画布是两条路径，当前还没有确定谁是项目工作流画布的权威承载。
- 秘书、记忆治理、知识库、适配器注册、项目黑板目前主要是设计文档能力，不是已落地能力。

结论：

下一步不应继续直接加大功能，也不应继续只做 UI 微调。应该先做架构收敛：把代码边界、事实边界、画布权威、秘书和记忆治理的最小落地点定下来，再继续功能开发。

## 2. 已知、未知和假设

已知：

- 最终蓝图以项目为最高级业务对象。
- 当前主线是 Codex 会话管理和 Codex 工作流编排，不是任务包管理器。
- 工作台最终形态需要秘书、建议方案、项目主管、工作流画布、通知、待办、审计、记忆、知识库、工具、模型、harness 和多智能体适配。
- 软件架构已暂定为：本地模块化单体运行形态 + 项目单元隔离 + 控制核心 + 项目黑板 + 能力适配器 + 事件账本和当前快照 + 核心记忆治理 + 秘书核心协作层。
- 首页 UI 内容已经确定，架构计划不能重新定义首页。
- 秘书是核心协作角色，不是某个页面或侧栏入口。
- 记忆层可以进入核心，但进入核心的是记忆治理，不是向量、图谱、Obsidian 读取或存储实现。

未知：

- 当前 `workflow-state.v0.json` 还能支撑多久。
- 项目工作流画布和独立可编辑画布是否应合一，还是保留不同层级。
- 秘书第一阶段应该接哪些读模型，哪些建议需要用户确认。
- 记忆治理第一阶段存储在 JSON 还是直接进入 SQLite。
- 真实 Tauri 窗口视觉验收线何时纳入每个实现任务的必做项。

保守假设：

- 在没有新的迁移计划前，继续以 JSON workflow state 作为过渡事实层。
- 不做 SQLite 迁移，不做向量库，不做 Obsidian 原生功能落地。
- 不读取 `/Users/yoyi/.codex`，不读取 auth、token、`.env`、完整 transcript。
- 不执行新的真实 `codex exec` 或 `codex exec resume`，除非用户对具体任务明确批准。
- 不改首页内容，只规定首页读取读模型时不能绕过事实层。

## 3. 依据

最终蓝图：

- `/Users/yoyi/Documents/Codex/2026-05-26/gan-xing-codexbridge-https-github-com/docs/architecture/local-ai-workbench-blueprint-v1.md`
- `/Users/yoyi/Documents/Codex/2026-05-26/gan-xing-codexbridge-https-github-com/docs/architecture/local-ai-workbench-ui-blueprint-v1.md`

当前架构和设计：

- `docs/workbench-system-architecture-v1.md`
- `docs/memory-layer-design-v1.md`
- `docs/workflow-task-package-design-v1.md`

当前权威入口：

- `CURRENT.md`
- `AUTHORITY.md`
- `tasks/README.md`
- `STAGE_PLAN.md`

当前约束决策：

- `decisions/2026-05-27-technical-stack-and-expansion-architecture.md`
- `decisions/2026-05-28-extensible-first-development-rule.md`
- `decisions/2026-05-28-codex-workflow-min-model.md`
- `decisions/2026-05-28-workflow-state-storage-v0.md`
- `decisions/2026-05-29-codex-session-workflow-route-correction.md`
- `decisions/2026-05-29-codex-session-plan-retained-workflow-first.md`
- `decisions/2026-05-31-editable-canvas-codex-as-director-v1.md`

当前代码入口：

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/mcp/**`
- `prototypes/productized-desktop-shell/src/App.tsx`
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `prototypes/productized-desktop-shell/src/views/CanvasView.tsx`
- `prototypes/productized-desktop-shell/src/lib/types.ts`

## 4. 本计划目标

目标：

- 让当前 app 的代码结构开始符合最终架构。
- 明确每个功能属于哪一层。
- 先解决边界，再继续加功能。
- 给其他对话可以直接执行的任务包顺序。

完成后应该达到：

- 后端不再由一个大文件承载所有职责。
- 项目工作流画布的权威关系清楚。
- 任务包退回内部协议位置。
- 项目黑板有最小数据模型和读模型。
- 秘书有核心协作模型，但不能直接改事实。
- 记忆治理有最小核心模型，但不把知识库、向量、图谱混进核心。
- Codex 只是第一个适配器，不是系统被写死的唯一形态。

## 5. 不做

本计划不做：

- 不重做首页 UI。
- 不把产品改成任务包管理器。
- 不把工作台改成 ComfyUI、n8n、Langflow 式通用节点执行器。
- 不做插件市场。
- 不做多 agent 真接入。
- 不做 Obsidian 原生功能内置。
- 不做向量库或图数据库。
- 不做 SQLite 迁移。
- 不默认执行真实 Codex 派发。
- 不读 `/Users/yoyi/.codex`。
- 不读取密钥、授权、`.env`、完整 transcript。
- 不写真实业务项目目录。

如果后续要做上面任何一项，必须另开计划或任务包。

## 6. 最终蓝图能力到架构层的映射

| 蓝图能力 | 所属架构层 | 第一落地点 | 边界 |
|---|---|---|---|
| 项目 | 控制核心、事实层、读模型 | 项目身份、项目隔离、项目读模型 | 项目是最高级业务对象 |
| 会话 | 适配器层、项目单元、事实层 | Codex 会话适配器、会话归属 | 会话必须归属于项目 |
| 项目主管 | 控制核心、工作流编排 | 状态机、派发、回收、完成闸门 | 主管不能绕过项目策略 |
| 子智能体 | 适配器层、项目黑板 | 结构化汇报、候选、阻塞 | 子智能体不能直接找用户决策 |
| 秘书 | 核心协作层、读模型 | 状态汇总、建议候选、风险提醒 | 秘书不是某个固定界面 |
| 建议方案 | 应用服务层、事实层、读模型 | 建议对象、用户确认、审计 | 用户默认看建议，不默认看后台执行计划 |
| 工作流画布 | 界面层、读模型、项目黑板 | 项目工作流画布和节点详情 | 不是通用节点执行器 |
| 通知 | 读模型、审计、待办接口 | 右侧状态入口 | 通知不承载完整讨论记录 |
| 待办 | 事实层、读模型 | 待办对象、负责人、状态 | 待办和工作流需明确绑定或分离 |
| 审计 | 事件账本、审计账本 | 关键动作追加记录 | 审计不能物理删除 |
| 记忆 | 控制核心治理、事实层 | 记忆候选、确认、版本、冲突 | 检索命中不是记忆 |
| 知识库 | 适配器层、读模型 | KnowledgeAdapter、引用和权限 | 知识库是材料空间，不是记忆 |
| 工具 | 适配器层、审计 | ToolAdapter、能力声明、调用摘要 | 工具结果全文不进账本 |
| 模型 | 适配器层、策略 | ModelAdapter、模型策略 | 不自动选择高风险模型 |
| harness | 适配器层、项目规则 | HarnessAdapter、运行性检查 | harness 不是普通画布节点 |

## 7. 当前实现到架构层的映射

| 当前文件或能力 | 当前承担内容 | 目标归属 | 风险 |
|---|---|---|---|
| `src-tauri/src/lib.rs` | 命令、模型、存储、派发、读模型、测试 | 应拆到命令、应用服务、控制核心、事实层、适配器、读模型 | 文件过大，新增功能会继续耦合 |
| `src-tauri/src/mcp/**` | 可编辑画布和 MCP 编排骨架 | 适配器层、工作流编排、画布运行层 | 只覆盖局部，不是统一核心 |
| `src/views/ProjectsView.tsx` | 项目页、任务包、工作流、派发、面板 | 界面层 | UI 内业务判断偏多 |
| `src/views/CanvasView.tsx` | 独立可编辑画布 | 界面层、画布读写 | 和项目工作流画布权威关系不清 |
| `src/lib/types.ts` | 前端类型镜像 | 读模型协议 | 类型多但边界未体现 |
| 工作流 state JSON | 当前事实和派生数据 | 过渡事实层 | 事件、快照、审计、黑板还没分清 |
| 右侧栏入口 | 通知、待办、审计、运行中 | 读模型入口 | 需要只显示入口，不承载全部逻辑 |

## 8. 偏离点清单

| 编号 | 偏离点 | 依据 | 处理策略 |
|---|---|---|---|
| A01 | 后端分层没有落地 | `lib.rs` 集中 1 万多行职责 | 先无行为变化拆模块 |
| A02 | 控制核心没有独立边界 | 状态转移、派发和读模型混在一起 | 抽 `core` 和 `application` |
| A03 | 项目黑板缺失 | 设计文档有黑板，代码没有显式对象 | 先建最小黑板读模型 |
| A04 | 事件账本、当前快照、审计账本没有统一模型 | 当前 workflow state 有多个数组，但不是统一事件体系 | 先定义事件追加规则，不迁移存储 |
| A05 | 适配器没有统一能力声明 | Codex runner 是直接路径 | 抽 AgentAdapter 能力接口，Codex 先实现 |
| A06 | 画布权威不清 | `CanvasView` 和项目 `WorkflowCanvas` 并存 | 先写决策，再实现统一入口 |
| A07 | 任务包仍容易占据主界面 | `ProjectsView.tsx` 任务包组件很多 | 降为节点详情和内部协议 |
| A08 | 秘书未建模 | 文档已定秘书为核心协作层 | 先做只读秘书上下文和建议对象 |
| A09 | 记忆治理未落地 | 记忆设计是草案 | 先做候选、确认、作用域、来源、审计 |
| A10 | 知识库和记忆边界易混 | 用户已指出混乱 | KnowledgeAdapter 只提供材料引用，不能写记忆 |
| A11 | UI 读模型和底层状态混杂 | 前端组件直接拼复杂状态 | 后端生成读模型，前端只展示和发命令 |
| A12 | 真窗口验收不足 | 现有截图多为 Chrome headless | 单独建立 Tauri 视觉验收线 |

## 9. 阶段 0：只读架构审计和代码贴标签

目的：

- 不改行为，先确认现有代码应归属哪一层。

任务：

- 列出 `lib.rs` 内所有结构体、命令、读写函数、派发函数、读模型函数、测试。
- 给每一类标目标层：命令、应用服务、控制核心、事实层、适配器、读模型、测试。
- 列出 `ProjectsView.tsx` 里哪些组件属于主界面，哪些应降为节点详情或内部调试。
- 列出 `CanvasView.tsx` 和项目工作流画布的重叠点。
- 输出一份 evidence，说明没有改代码。

验收：

- 生成模块拆分表。
- 生成前端组件归属表。
- 标出不可直接拆的高风险函数。
- 不改代码，不跑真实 Codex。

## 10. 阶段 1：后端模块边界，保持行为不变

目的：

- 先把大文件拆开，但不改变产品行为。

建议模块：

- `commands`：Tauri 命令入口，只做参数接收和错误包装。
- `application`：应用服务，把用户命令转成核心操作和适配器调用。
- `core`：项目、会话、工作流、权限、状态机、记忆治理的规则。
- `facts`：workflow state、事件、审计、快照读写。
- `read_models`：给前端的派生视图。
- `adapters`：Codex、工具、harness、未来知识库和模型。
- `workflow_machine`：四角色机器和运行控制。
- `task_package_protocol`：任务包内部协议。

任务：

- 先移动纯类型和纯函数。
- 再移动读模型函数。
- 再移动 workflow state 读写。
- 再移动 Codex runner 和派发相关函数。
- 最后移动 Tauri command 包装。
- 每次移动都跑聚焦测试。

验收：

- 前端调用的 Tauri 命令名不变。
- `npm run typecheck` 通过。
- `npm run test:offline-interaction` 通过。
- `npm run build` 通过。
- `cargo test --lib` 通过。
- 无真实 Codex 执行。
- 无真实 workflow state 写入。

## 11. 阶段 2：项目工作流画布权威统一

目的：

- 解决“项目页工作流画布”和“全局可编辑画布”两条路径并存的问题。

必须先确认的决策：

- 项目里的工作流画布是否以项目 workflow state 为权威事实。
- 独立 `CanvasView` 是否作为模板编辑器、实验画布，还是并入项目工作流。
- `src-tauri/src/mcp/**` 的 canvas / run 文件层是否继续保留。

保守建议：

- 项目工作流画布以项目 workflow state 和项目黑板为权威。
- `CanvasView` 暂时作为后置可编辑画布能力，不继续扩大。
- MCP 画布骨架保留，但不得绕过项目规则和控制核心。

任务：

- 写一份画布权威决策。
- 项目详情的工作流界面只显示工作流画布和右侧详情入口。
- 任务包、账本、汇报、审查、异常都进入节点详情或右侧展开，不作为项目工作流主界面平铺。
- 画布顶部只保留项目规则状态条，不放大量说明文字。

验收：

- 用户进入项目后能明确看到项目列表 + 工作流画布。
- 工作流画布读的是同一份项目读模型。
- 不出现“任务包管理器”式主界面。
- 不引入通用节点执行器。

## 12. 阶段 3：项目黑板最小落地

目的：

- 给多智能体协作一个中间态容器，避免候选、汇报、工具摘要、记忆候选直接污染正式事实。

最小对象：

- `ProjectBlackboard`
- `BlackboardEntry`
- `BlackboardEntryKind`
- `BlackboardSourceRef`
- `BlackboardPromotionDecision`

最小内容：

- 当前目标。
- 工作流节点状态摘要。
- 子智能体汇报。
- 证据引用。
- 权限请求。
- 工具调用摘要。
- 审查结果。
- 风险和阻塞。
- 记忆候选。
- 知识库引用。

规则：

- 黑板不是正式事实源。
- 子智能体、工具和知识库只能向黑板提交候选。
- 只有控制核心能把候选升级为事实、记忆、审计或状态变化。

验收：

- 子智能体汇报先进入黑板或账本，不直接改节点完成状态。
- 记忆候选进入黑板，不直接写正式记忆。
- 知识库引用进入黑板，不直接影响 agent 行为。
- 项目主管回收后才推进状态。

## 13. 阶段 4：控制核心命令和状态机收敛

目的：

- 把“谁能做什么、什么时候能做”从 UI 和散落函数里收回核心。

核心命令：

- 创建项目。
- 绑定会话。
- 启动工作流。
- 派发给角色。
- 记录子智能体汇报。
- 记录审查结果。
- 记录主管回收。
- 创建建议方案。
- 用户确认建议方案。
- 创建记忆候选。
- 采纳、冻结、废弃记忆。
- 记录权限申请、批准、拒绝。

规则：

- 前端只发命令，不判断最终状态是否合法。
- 控制核心判断状态转移和权限。
- 适配器只执行受控动作，不推进事实状态。
- 所有关键动作写事件和审计。

验收：

- 非法状态转移被后端拒绝。
- 子智能体不能标记任务完成。
- 秘书不能直接改系统事实。
- UI 中的按钮不可用只是提示，真正拦截在后端。

## 14. 阶段 5：秘书核心协作层最小落地

目的：

- 把秘书建成核心协作角色，而不是固定到某个界面。

第一阶段只读能力：

- 读取用户偏好记忆。
- 读取全局状态摘要。
- 读取项目摘要。
- 汇总通知、待办、运行中工作流、异常。
- 汇总记忆冲突和知识库结构风险。
- 生成建议候选。

第一阶段禁止：

- 不能直接改项目。
- 不能直接派发任务。
- 不能直接写正式记忆。
- 不能绕过用户确认。
- 不能替代审计中心。

最小对象：

- `SecretaryContext`
- `SecretarySuggestion`
- `SecretaryRiskSignal`
- `SecretaryMemoryCandidate`
- `SecretaryActionProposal`

验收：

- 秘书可以出现在首页、项目页、画布、通知、待办、记忆管理等入口，但模型不属于任何单一页面。
- 秘书输出的是建议或候选，不是事实变更。
- 用户确认后，由应用服务调用控制核心生效。

## 15. 阶段 6：记忆治理最小核心

目的：

- 先做“什么可以成为记忆、谁确认、作用域是什么”，不做复杂检索。

最小对象：

- `MemoryCandidate`
- `MemoryRecord`
- `MemoryScope`
- `MemorySourceRef`
- `MemoryLifecycleStatus`
- `MemoryConflict`
- `MemoryAuditRef`

第一优先级：

- 用户偏好记忆。
- 全局产品蓝图记忆。
- 项目记忆。
- 会话摘要。
- 成熟模式候选。

规则：

- 普通聊天不自动进入长期记忆。
- 检索命中不是记忆。
- 知识库材料不是记忆。
- 子智能体汇报不是记忆。
- 工作流总结、阶段汇报、用户采纳建议可以成为记忆候选。
- 采纳正式记忆必须有来源、版本、状态、权限和审计链接。

验收：

- 可以创建记忆候选。
- 可以查看来源和作用域。
- 可以采纳、冻结、废弃。
- 可以标记冲突。
- 不接向量库，不接图谱，不接 Obsidian 原生读写。

## 16. 阶段 7：适配器注册和能力声明

目的：

- 让 Codex 是第一个适配器，而不是系统唯一形态。

最小适配器类型：

- AgentAdapter。
- KnowledgeAdapter。
- MemoryStorageAdapter。
- ModelAdapter。
- ToolAdapter。
- HarnessAdapter。
- CodeIndexAdapter。

第一阶段只实现：

- Codex AgentAdapter 的能力声明。
- 当前已有 harness 候选读取作为 HarnessAdapter 雏形。
- 工具能力用声明，不做任意工具执行。

适配器必须返回：

- 能力声明。
- 输入限制。
- 输出摘要。
- 证据引用。
- 错误分类。

适配器禁止：

- 直接写核心事实。
- 直接写正式记忆。
- 直接跨项目读写。
- 直接提升权限。

验收：

- 当前 Codex 派发路径经过 AgentAdapter。
- 后端 schema 不再写死成 Codex-only。
- 未实现的适配器只显示“未接入”，不显示假能力。

## 17. 阶段 8：事件账本、当前快照和读模型收敛

目的：

- 让事实、事件、审计和 UI 读模型分清。

最小事件：

- 用户确认。
- 权限请求。
- 权限批准或拒绝。
- 工作流状态变化。
- 节点派发。
- 智能体汇报。
- 主管回收。
- 工具调用摘要。
- 模型使用。
- harness 变化。
- 记忆候选创建。
- 记忆采纳、冻结、废弃。
- 知识库授权变化。

规则：

- 事件追加，不修改旧记录。
- 当前快照保存当下状态。
- 读模型可重建，不当事实源。
- UI 不直接拼底层复杂状态。

验收：

- 任一关键状态变化都有事件或审计引用。
- 读模型缺字段时返回 warning 或 missing，不补编事实。
- 工具调用全文不进入工作流账本。

## 18. 阶段 9：UI 按架构收敛

目的：

- 在不重新定义首页的前提下，把 UI 和架构对齐。

原则：

- 首页内容按已确定 UI 走，不在本计划重设。
- 项目页主界面只保留项目列表 + 工作流画布。
- 工作流页只显示工作流画布。
- 会话页只显示会话界面。
- 右侧保留通知、待办、审计、运行中工作流入口，选中后展开详情。
- 左侧保持窄图标导航和悬停提示。
- 任务包、账本、审查、异常进入节点详情或右侧展开，不作为主界面平铺。
- 说明性文字尽量移除，信息通过状态、节点、详情和提示承载。

验收：

- UI 看起来不是任务包后台。
- 项目详情像内嵌一个项目工作台，而不是散乱面板堆叠。
- 画布节点详情在右侧展开，不用弹窗打断。
- 不把秘书固定死在某一个页面。

## 19. 阶段 10：Tauri 视觉和交互验收线

目的：

- 补上当前“Chrome headless 可看，但 Tauri 真窗口未完整验收”的缺口。

任务：

- 建立真实 Tauri 窗口截图流程。
- 对首页、项目列表、项目工作流画布、会话页、右侧栏展开做截图。
- 对窗口尺寸变化做最少一轮验证。
- 记录截图和操作路径到 evidence。

验收：

- 能证明真实 Tauri 窗口不是空白。
- 能证明主导航、项目列表、画布、右侧入口不重叠。
- 能证明项目页符合当前 UI 决策。
- 不能只用 Chrome headless 代替。

## 20. 推荐执行顺序

第一批只做架构收敛，不做新产品能力：

1. 阶段 0：只读架构审计和代码贴标签。
2. 阶段 1：后端模块边界，保持行为不变。
3. 阶段 2：项目工作流画布权威统一。

第二批做核心协作骨架：

4. 阶段 3：项目黑板最小落地。
5. 阶段 4：控制核心命令和状态机收敛。
6. 阶段 8：事件账本、当前快照和读模型收敛。

第三批做最终蓝图核心角色：

7. 阶段 5：秘书核心协作层最小落地。
8. 阶段 6：记忆治理最小核心。
9. 阶段 7：适配器注册和能力声明。

第四批做验收线：

10. 阶段 9：UI 按架构收敛。
11. 阶段 10：Tauri 视觉和交互验收线。

## 21. 可直接派给其他对话的任务包

### Task A：架构只读审计

目标：

- 生成当前代码到目标架构层的映射表。

读范围：

- `docs/workbench-system-architecture-v1.md`
- `docs/workflow-task-package-design-v1.md`
- `docs/memory-layer-design-v1.md`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/mcp/**`
- `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `prototypes/productized-desktop-shell/src/views/CanvasView.tsx`

写范围：

- `evidence/**`
- `handoffs/**`
- 必要时 `docs/plans/**`

禁止：

- 不改代码。
- 不读 `/Users/yoyi/.codex`。
- 不执行 Codex。

验收：

- 有模块拆分表。
- 有前端组件归属表。
- 有高风险函数清单。

### Task B：后端无行为变化拆模块

目标：

- 把 `lib.rs` 按架构层拆成模块，保持命令行为不变。

写范围：

- `prototypes/productized-desktop-shell/src-tauri/src/**`
- `evidence/**`
- `handoffs/**`

验收：

- 命令名不变。
- Rust 测试通过。
- 前端构建通过。
- 没有真实 Codex 执行。

### Task C：画布权威决策和项目画布收敛

目标：

- 明确项目 workflow state、项目黑板、独立 CanvasView、MCP canvas / run 文件层的关系。
- 将产品入口和代码边界收窄到“项目工作流画布以项目 workflow state 为权威；独立 CanvasView 暂为实验/模板/后置能力”。

输出：

- 如现有 `decisions/2026-06-01-project-workflow-canvas-authority-v1.md` 不够细，补充 decision 或执行计划。
- 项目页主路径只显示项目列表 + 工作流画布。
- 任务包和账本进入节点详情或右侧展开。

验收：

- 不再出现两个都像权威画布的入口。
- 不把产品变成通用节点执行器。
- 不直接合并独立 `CanvasView` 和项目 workflow state。
- 不执行真实 Codex。
- 不启动 MCP canvas run。
- 不改 workflow state JSON。

### Task D：项目黑板最小实现

目标：

- 新增项目黑板模型和只读读模型。
- 当前状态：最小只读切片已完成。
- 黑板写入命令不属于 Task D 已完成范围；Task E 已完成控制核心边界的保守切片，但如需继续补黑板写入能力，仍必须先单开 schema / 迁移计划，明确黑板候选持久确认状态。

验收：

- 子智能体汇报、风险、权限请求、记忆候选、知识引用可以作为中间态展示。
- 黑板内容不会直接变成正式事实。

### Task E：控制核心命令收敛

目标：

- 把状态机、权限、派发、回收和完成判定收回后端核心。

验收：

- UI 不能绕过后端推进状态。
- 非法转移被拒绝。
- 审计记录完整。

### Task F：秘书核心只读 v1

目标：

- 建立秘书上下文和建议候选对象。

验收：

- 秘书能汇总状态和风险。
- 秘书不能直接改事实。
- 用户确认后才调用控制核心。

### Task G：记忆治理核心 v1

目标：

- 建立记忆候选、正式记忆、来源、作用域、状态、冲突和审计。

验收：

- 用户偏好记忆可以被确认记录。
- 工作流总结只能先成为候选。
- 不接向量库、图谱、Obsidian 原生功能。

### Task H：适配器注册 v1

目标：

- 给 Codex 派发路径套上 AgentAdapter 能力声明。

验收：

- Codex 是一个适配器实现。
- schema 不写死 Codex-only。
- 未实现适配器不显示假能力。

### Task I：事件、快照、读模型收敛

目标：

- 让关键动作都有事件或审计，读模型从事实派生。

验收：

- UI 不直接拼底层复杂状态。
- 读模型缺字段不补编。
- 工具全文不进入账本。

### Task J：真实 Tauri 验收线

目标：

- 建立真实窗口截图和交互验收流程。

验收：

- 首页、项目页、画布、会话页、右侧栏都有真实 Tauri 证据。
- 不再把 Chrome headless 截图当作完整验收。

## 22. 测试要求

每个代码任务至少跑：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- `cargo test --lib`

如果任务只改 Rust：

- 至少跑相关 Rust 聚焦测试和 `cargo test --lib`。

如果任务只改前端：

- 至少跑 `npm run typecheck`、相关前端测试和 `npm run build`。

如果任务改 UI 布局：

- 需要截图证据。
- 如果声明 Tauri 验收，必须是真实 Tauri 窗口，不是普通浏览器。

## 23. 停止条件

出现以下情况必须停下来问用户或另开计划：

- 需要读取 `/Users/yoyi/.codex`。
- 需要读取 auth、token、`.env`、完整 transcript。
- 需要执行真实 `codex exec` 或 `codex exec resume`。
- 需要写真实业务项目目录。
- 需要迁移 SQLite。
- 需要接 Obsidian 原生功能。
- 需要引入向量库或图数据库。
- 需要把独立 CanvasView 和项目 workflow state 合一。
- 需要改变首页内容。
- 需要把秘书固定成某个页面或右侧栏。

## 24. 当前 app 是否偏离蓝图

判断：

- 方向没有完全偏离，因为当前已有项目、会话、工作流、右侧状态入口、Codex 编排和工作流状态。
- 代码架构已经偏离，因为控制核心、项目黑板、适配器、事实层、读模型没有真正分层。
- UI 仍有偏离风险，因为任务包细节和多个工作流面板容易占据主界面。
- 最终蓝图能力大部分还未落地，不能把当前 app 说成最终架构已实现。

依据：

- `CURRENT.md` 和 `tasks/README.md` 已明确当前只完成 Codex 角色编排闭环的一部分。
- `docs/workbench-system-architecture-v1.md` 明确当前还没有完整落地的系统架构实现。
- 当前代码里 `lib.rs` 和 `ProjectsView.tsx` 的职责过多，和目标分层不一致。

## 25. 下一步建议

最保守的下一步：

- 继续做更小的审计或读模型拆分切片，例如迁移 `workflow_permission_decision_recorded` / `workflow_node_dispatch_prepared` 这类低风险审计构造，或迁移纯派生的账本读模型函数并补一致性测试。

如果继续补黑板写入能力：

- 先单开 schema / 迁移计划，设计黑板候选持久确认状态，以及候选如何经控制核心确认后升级为正式事实、正式记忆、审计事件或状态变化。
- 不要直接补一个写 workflow state JSON 的黑板接口。
- 不执行真实 Codex。
- 不改 workflow state JSON 结构，除非另开迁移计划。

不建议马上做：

- 秘书完整 UI。
- 记忆层真实存储。
- Obsidian 内置。
- 多 agent 接入。
- 通用节点画布。
- 真实业务自动编排。

原因：

- 这些能力都依赖控制核心、项目黑板、适配器和事实层边界。边界未稳之前继续做，会让返工变大。
