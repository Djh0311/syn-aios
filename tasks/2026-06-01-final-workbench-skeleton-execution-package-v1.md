# 任务包：最终工作台骨架执行总包 v1

## 任务名

最终工作台骨架执行总包 v1。

## 所属开发线

桌面应用线 / 后端架构线 / 前端工作台线 / 工作流状态线。

总指导线分阶段回收。

## 当前理解

这不是一个“一次性做完所有功能”的任务包。

这是一份总执行包：目标是让当前 `prototypes/productized-desktop-shell` 原型继续长成最终本地 AI 工作台的架构骨架。

执行粒度采用“批次化小步”：

- 不是每一个微步骤都单独开任务包。
- 能在同一批次里完成、同一组文件内验证、同一套测试能覆盖的步骤，应合并执行。
- 触及硬边界时才拆开停下来问用户。
- 普通批次不用逐个停下来给用户验收，最终统一验收。

大白话：

现在原型已经能跑一些工作流和 Codex 编排，但还不是最终工作台骨架。接下来要把它逐步整理成：项目是主轴，控制核心管事实和权限，黑板接中间态，适配器接外部能力，事件和审计可追踪，读模型服务界面，秘书和记忆治理进入核心但不越权。

## 已知

- 当前工程仍是原型工程：
  - `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell`
- 当前最终架构结论是：
  - 本地模块化单体运行形态。
  - 项目单元隔离。
  - 控制核心。
  - 项目黑板。
  - 能力适配器。
  - 事件账本和当前快照。
  - 核心记忆治理。
  - 秘书核心协作层。
- 已完成：
  - Task A 架构只读审计。
  - Task B 保守拆模块切片。
  - Task C 项目工作流画布权威收敛保守切片。
  - Task D 项目黑板最小只读切片。
  - Task E 控制核心命令收敛保守切片。
  - 控制核心第二切片边界拆分。
- 当前仍未完成：
  - 最终架构拆分。
  - 黑板候选持久确认状态。
  - 正式记忆治理。
  - 秘书核心协作层。
  - 适配器注册。
  - 完整事件账本。
  - 真实 Tauri 验收线的完整自动化覆盖。
  - 真实业务自动编排产品化。

## 未知

- 黑板候选持久确认状态的 schema 还没定。
- 正式记忆记录是否先落 JSON 还是直接进入 SQLite 还没定。
- 秘书第一阶段需要读取哪些读模型，还要在设计任务里确认。
- 适配器注册最小 schema 还没定。
- 真实 Tauri 验收线已完成最小截图证据，完整自动化覆盖和权限确认弹层验收规范还没最终定。
- `cargo fmt --check` 历史格式债是否单独处理，还要用户确认。

## 假设

- 先继续在原型工程内长骨架，不另起新项目。
- v0 仍以 workflow state JSON 作为过渡事实层。
- SQLite、FTS、向量库、图数据库、Obsidian 原生读写都后置，除非单独设计和用户确认。
- 每一阶段只做最小可验证切片，不做大重构。
- 不通过 UI 直接写事实；关键事实必须走后端命令和控制核心。

## 依据

- `CURRENT.md`：当前完成情况和下一步建议。
- `AUTHORITY.md`：当前权威文档索引。
- `docs/workbench-system-architecture-v1.md`：最终蓝图下的软件开发架构设计草案。
- `docs/plans/2026-06-01-workbench-architecture-implementation-plan-v1.md`：当前架构落地执行计划。
- `docs/memory-layer-design-v1.md`：记忆层设计草案。
- `docs/workflow-task-package-design-v1.md`：工作流和任务包设计草案。
- `decisions/2026-06-01-project-workflow-canvas-authority-v1.md`：项目工作流画布权威关系。
- `decisions/2026-06-01-architecture-module-split-guardrail-v1.md`：保守拆分边界。
- 已完成 evidence / handoff：
  - `evidence/2026-06-01-workbench-architecture-readonly-audit-v1.md`
  - `evidence/2026-06-01-workbench-architecture-task-b-conservative-module-split-v1.md`
  - `evidence/2026-06-01-project-workflow-canvas-authority-convergence-c-v1.md`
  - `evidence/2026-06-01-project-blackboard-minimal-read-model-d-v1.md`
  - `evidence/2026-06-01-control-core-command-convergence-v1.md`
  - `evidence/2026-06-01-control-core-second-slice-boundary-extraction-v1.md`

## 薄弱点

- 如果把下面所有阶段当成一个开发任务，会失控。
- 如果把每个微步骤都拆成独立任务，会让 agent 把大量精力花在治理流程、重复 evidence 和重复 handoff 上，反而拖慢实现。
- 如果先做秘书、记忆、Obsidian 或多 agent 接入，底层事实和权限边界还会继续混。
- 如果先做黑板写入而不设计 schema，会把候选、正式事实、记忆和审计混在一起。
- 如果继续只拆文件不补测试，可能看起来有架构，实际行为没被保护。
- 如果不做真实 Tauri 验收线，UI 状态仍可能只在普通浏览器里看起来正常。
- 如果把画布当成普通 UI 收尾任务，会低估风险；画布需要参考源研究、节点 schema、组件状态样例、React Flow 接入、节点详情面板和回归验收，不是一轮样式调整。

## 总目标

让当前原型具备最终工作台骨架：

1. 项目单元成为主要隔离单元。
2. 控制核心统一管状态、权限、候选转事实和审计。
3. 项目黑板承载中间态，但不直接成为事实源。
4. 事实层有当前快照、审计、事件边界。
5. 读模型服务界面，前端不拼底层事实。
6. 适配器以能力声明接入外部能力，Codex 只是第一个 AgentAdapter。
7. 秘书作为核心协作角色产生建议和候选，不直接改事实。
8. 记忆治理先做候选、来源、作用域、状态、冲突、审计，不做复杂检索。
9. UI 保持当前已确认方向，不把任务包管理器重新变成主界面。
10. 项目工作流画布有独立开发链路：先 schema 和组件状态，再实现 React Flow 画布，不直接手写复杂拖拽。
11. 每个关键切片都有 evidence、handoff 和测试；普通切片连续执行，最终统一验收。

## 总非目标

- 不一次性完成最终产品。
- 不做真实业务自动编排产品化。
- 不直接写黑板持久状态。
- 不直接写正式记忆。
- 不接 Obsidian 原生读写。
- 不接向量库。
- 不接图数据库。
- 不做多 agent 完整生态。
- 不把工作台做成通用节点自动化工具。
- 不把任务包管理器做成主界面。
- 不改变首页 UI 内容。
- 不执行真实 `codex exec` / `codex exec resume`，除非单独任务包、用户明确批准。
- 不读写 `/Users/yoyi/.codex`，除非单独任务包、用户明确批准。
- 不读取 auth、token、`.env`、密钥、完整 transcript 或 rollout JSONL 正文。
- 不迁移数据库，除非单独 schema / 迁移计划并经用户确认。

## 阶段拆分

### 阶段 0：当前事实再冻结

目标：

- 在继续开发前，冻结当前原型状态、完成项、未完成项、禁止边界。

任务：

1. 只读复核 `CURRENT.md`、`AUTHORITY.md`、`tasks/README.md`。
2. 只读复核当前代码模块和 `lib.rs` 主要职责。
3. 输出一份 evidence，列出当前骨架完成度。

验收：

- 不改代码。
- 不跑真实 Codex。
- 不写 workflow state。

停下来：

- 阶段 0 完成后不必停，继续执行下一普通小任务。
- 如果发现当前权威文档互相冲突到无法判断下一步，必须停。

### 阶段 1：事实层和审计边界继续小切片

目标：

- 继续把状态写入、审计构造、读模型派生从 `lib.rs` 小步拆出。

优先任务：

1. 审计切片：
   - 迁移低风险审计构造，例如 `workflow_permission_decision_recorded`。
   - 或迁移 `workflow_node_dispatch_prepared`。
2. 读模型切片：
   - 迁移纯派生函数，例如账本读模型派生。
   - 补一致性测试。
3. 状态写入切片：
   - 只包装底层 store 或低风险 mutation，不改状态机。

验收：

- 不改 workflow state JSON 结构。
- 不改状态机语义。
- 不执行真实 Codex。
- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- `cargo test --lib`

停下来：

- 每完成一个审计/读模型/状态写入切片不必停，继续执行下一普通小任务。
- 如果需要改 JSON schema，立即停。

### 阶段 2：黑板候选持久确认状态设计

目标：

- 先设计黑板候选怎样持久记录确认、拒绝、待处理，不直接实现大写入。

任务：

1. 定义候选状态：
   - candidate_pending_control_core
   - candidate_rejected
   - candidate_confirmed_for_fact
   - candidate_confirmed_for_memory
   - candidate_superseded
2. 定义来源：
   - 子智能体汇报。
   - 权限请求。
   - 工具摘要。
   - 记忆候选。
   - 知识引用。
   - 风险。
3. 定义确认结果：
   - 正式事实。
   - 正式记忆候选。
   - 审计事件。
   - 状态变化建议。
4. 定义 schema / 迁移计划。
5. 定义控制核心命令。

验收：

- 只写设计和任务包，先不实现写入。
- 用户确认 schema 后才进入实现。

停下来：

- schema / 迁移计划写完必须停。
- 用户没确认前不能实现。

### 阶段 3：黑板候选持久确认最小实现

目标：

- 在阶段 2 确认后，实现最小黑板候选确认闭环。

任务：

1. 后端命令：
   - 确认候选。
   - 拒绝候选。
   - 标记待处理。
2. 控制核心校验：
   - 禁止直接写正式记忆。
   - 禁止知识引用直接变记忆。
   - 禁止工具摘要直接推进状态。
3. 审计：
   - 所有候选确认/拒绝写 audit。
4. UI：
   - 只展示确认状态和来源。
   - 不把候选伪装成正式事实。

验收：

- 测试覆盖确认、拒绝、非法升级。
- 不写正式记忆。
- 不做秘书。

停下来：

- 实现完成后不必停，普通情况下继续适配器能力声明骨架。
- 如果实现需要超出已确认 schema 或迁移计划，必须停。

### 阶段 4：适配器注册最小骨架

目标：

- 让 Codex 成为第一个 AgentAdapter，而不是系统唯一形态。

任务：

1. 定义 AdapterCapability。
2. 定义 AgentAdapter read model。
3. 给现有 Codex 路径套能力声明。
4. 未实现适配器不显示假能力。
5. 前端展示“能力声明”，不展示虚假可用动作。

验收：

- 不接 Claude、OpenCode、OpenClaw 实现。
- 不改真实 Codex 执行路径。
- 不执行真实 Codex。

停下来：

- Codex 适配器能力声明完成后不必停，继续执行下一普通小任务。

### 阶段 5：记忆治理最小核心

目标：

- 先做治理，不做复杂检索。

任务：

1. 定义 MemoryCandidate。
2. 定义 MemoryRecord。
3. 定义 MemoryScope。
4. 定义 MemorySourceRef。
5. 定义 MemoryLifecycleStatus。
6. 定义 MemoryConflict。
7. 定义 MemoryAuditRef。
8. 用户偏好记忆作为第一优先级候选。

验收：

- 普通聊天不自动进入长期记忆。
- 检索命中不是记忆。
- 知识库材料不是记忆。
- 不接向量库。
- 不接图谱。
- 不接 Obsidian 原生读写。

停下来：

- 记忆 schema 写完先停。
- 用户确认后再做最小实现。

### 阶段 6：秘书核心只读 v1

目标：

- 把秘书建成核心协作角色的只读模型，不固定到某个界面。

任务：

1. 定义 SecretaryContext。
2. 定义 SecretarySuggestion。
3. 定义 SecretaryRiskSignal。
4. 定义 SecretaryMemoryCandidate。
5. 定义 SecretaryActionProposal。
6. 读取：
   - 用户偏好记忆。
   - 全局状态摘要。
   - 项目摘要。
   - 通知。
   - 待办。
   - 运行中工作流。
   - 异常。
   - 记忆冲突。
7. 输出建议或候选，不改事实。

验收：

- 秘书不能直接改项目。
- 秘书不能直接派发任务。
- 秘书不能直接写正式记忆。
- 用户确认后才调用控制核心。

停下来：

- 秘书只读模型完成后不必停，普通情况下继续工作流和项目页收敛。

### 阶段 7：工作流和项目页收敛

目标：

- 项目工作流画布成为项目工作流主入口，其他内容收回节点详情或右侧抽屉。

任务：

1. 项目页只显示项目该有的界面。
2. 工作流页只显示工作流画布和必要节点详情。
3. 任务包、账本、状态机、子汇报等内部细节收进画布节点详情或右侧抽屉。
4. 独立 CanvasView 保持实验/模板地位，不作为项目事实源。

验收：

- 不改首页内容。
- 不做通用节点自动化平台。
- 不启动 MCP canvas run。
- UI 截图证据必须提供。

停下来：

- UI 布局改变前先写任务包。
- 截图证据进入最终统一验收。

### 阶段 8：真实 Tauri 验收线

目标：

- 建立真实 Tauri 窗口验收，不能只靠普通浏览器。

任务：

1. 定义 Tauri 验收命令。
2. 定义截图路径。
3. 覆盖：
   - 首页。
   - 项目页。
   - 工作流画布。
   - 会话页。
   - 右侧栏。
   - 权限确认弹层。
4. 记录普通浏览器验收和 Tauri 真窗口验收的区别。

验收：

- 有真实 Tauri 证据。
- 不把 Chrome headless 当完整验收。

停下来：

- 验收线跑通后不必停，截图证据进入最终统一验收。

### 阶段 9：格式债单独处理

目标：

- 单独处理 `cargo fmt --check` 历史格式债，不和业务改动混在一起。

任务：

1. 只格式化明确范围。
2. 不改业务逻辑。
3. 跑完整测试。
4. evidence 只记录格式变化。

验收：

- 用户确认是否做。
- 未确认前不要做。

停下来：

- 这是独立任务，必须先问用户。

## 推荐执行顺序

推荐顺序：

1. 阶段 0：当前事实再冻结。已完成。
2. 阶段 1：审计/读模型/状态写入继续小切片。已完成当前保守切片。
3. 阶段 8：真实 Tauri 验收线。已完成最小截图线，完整自动化仍后置。
4. 画布专项：参考源复核、节点 schema、组件状态样例、React Flow 最小实现、节点详情面板。当前进行中。
5. 阶段 2：黑板候选持久确认状态设计。
6. 阶段 3：黑板候选持久确认最小实现。
7. 阶段 4：适配器注册最小骨架。
8. 阶段 5：记忆治理最小核心。
9. 阶段 6：秘书核心只读 v1。
10. 阶段 7：工作流和项目页收敛。
11. 阶段 9：格式债单独处理。

原因：

- 先稳事实、审计和读模型，再做会改变事实含义的黑板、记忆、秘书。
- 画布不是普通页面，必须在黑板、记忆、秘书复杂化之前先建立 schema、组件状态和最小可验收画布。
- Tauri 验收线应尽早补，不然 UI 改动无法可靠验收。
- 格式债不和业务混做，避免看不清真实改动。

## 任务粒度规则

默认按“批次”执行，不按微步骤执行。

可以合并为一个批次的条件：

1. 写入文件范围高度重叠。
2. 行为目标属于同一个用户可感知结果。
3. 不改变 workflow state JSON 结构。
4. 不迁移数据库。
5. 不执行真实 `codex exec` / `codex exec resume`。
6. 不读取或写入 `/Users/yoyi/.codex`。
7. 不写正式记忆、正式黑板持久事实或真实业务项目目录。
8. 同一组测试和截图能覆盖主要风险。

必须拆开的情况：

1. 触及“必须停下来问用户”的硬边界。
2. 后端事实结构和前端展示要同时大改。
3. 不同 agent 或不同对话会并行改同一批文件。
4. 需要先做 schema / 迁移计划，再做写入实现。
5. 验收口径差异太大，合在一起会看不清失败原因。

文档要求也跟着收敛：

- 一个批次输出一份 evidence 和一份 handoff。
- 批次内普通微步骤可以在 evidence 里列清，不必每步单独建 evidence / handoff。
- 只有硬边界、失败回滚、真实写入、真实 Codex 执行才单独追加记录。

当前批次化调整：

- `final-skeleton-07`、`final-skeleton-08`、`final-skeleton-09` 合并为“画布基础批次”执行更合适。
- 这个批次可以覆盖组件状态样例、React Flow 项目画布最小实现、节点详情/右侧面板收敛。
- 如果执行中发现需要改 workflow state JSON、启动 MCP canvas run、做通用低代码编辑器或改首页内容，必须拆开并停下来。

## 骨架完成后的最近两项

骨架完成后不要直接铺开所有功能。最近两项先做：

1. Codex 原生感会话中心，同时抽成多智能体软件 / 会话底座。
   - 目标：把当前混乱的 Codex 会话信息整理成可用的原生感体验，并让 Claude Code、OpenClaw 等后续接入同一套软件层 / 会话层模型。
   - 范围：会话列表、项目归属、当前任务、角色、状态、模型、时间线、权限等待、异常和归档。
   - 不做：第一轮不直接接全量 Claude Code / OpenClaw，不写 Codex 内部状态库，不做会话删除或 CDP 注入。
2. 项目工作流画布产品化深化。
   - 目标：把项目工作流画布从“能看”推进到“能作为项目工作主入口”。
   - 范围：节点状态、React Flow 画布、右侧节点详情、运行性检查、建议 / 运行双视图、证据和权限入口。
   - 不做：不做通用自动化平台，不做完整低代码编辑器，不把独立 `CanvasView` 当项目事实源。

模型和凭据基础作为这两项的依赖先进入会话和节点字段；完整全局模型库、项目模型池、凭据权限和成本统计排在后续独立任务。

## 可执行小任务包清单

下面这些是可逐个派发的小任务包，也可以按“任务粒度规则”合并成执行批次。普通微步骤不必逐个停下来验收；一个批次完成后产出一份 evidence / handoff。触及硬边界时必须停。

### Skeleton-00：当前事实冻结

任务名：

- `final-skeleton-00-current-facts-freeze-v1`

目标：

- 冻结当前原型架构事实，给后续所有任务当基线。

允许读取：

- `CURRENT.md`
- `AUTHORITY.md`
- `README.md`
- `tasks/README.md`
- `docs/workbench-system-architecture-v1.md`
- `docs/plans/2026-06-01-workbench-architecture-implementation-plan-v1.md`
- 最近 6 份架构相关 evidence / handoff
- `prototypes/productized-desktop-shell/src-tauri/src/*.rs`
- `prototypes/productized-desktop-shell/src/**/*.tsx`
- `prototypes/productized-desktop-shell/src/**/*.ts`

禁止：

- 不改代码。
- 不改 workflow state。
- 不执行真实 Codex。
- 不读写 `/Users/yoyi/.codex`。
- 不读取密钥、token、`.env`、完整 transcript。

执行步骤：

1. 列出当前已完成骨架能力。
2. 列出当前未完成骨架能力。
3. 列出当前主要代码模块和职责。
4. 列出 `lib.rs`、`ProjectsView.tsx`、工作流状态、黑板、控制核心、读模型的剩余风险。
5. 输出“最终骨架完成度矩阵”。

验收：

- 只新增 evidence / handoff。
- `CURRENT.md` 和 `tasks/README.md` 可更新当前事实。
- 不需要跑代码测试。

必须输出：

- `evidence/2026-06-01-final-skeleton-00-current-facts-freeze-v1.md`
- `handoffs/2026-06-01-final-skeleton-00-current-facts-freeze-v1-result.md`

完成后停：

- 不必停；继续执行下一普通小任务，除非发现基线冲突。

### Skeleton-01：审计构造小切片

任务名：

- `final-skeleton-01-audit-helper-slice-v1`

目标：

- 继续把低风险审计事件构造从 `lib.rs` 拆到 `workflow_audit.rs`。

优先候选：

- `workflow_permission_decision_recorded`
- `workflow_node_dispatch_prepared`
- 二选一即可，不要贪多。

禁止：

- 不改审计字段含义。
- 不改 workflow state JSON 结构。
- 不改状态机。
- 不执行真实 Codex。
- 不做黑板写入。

执行步骤：

1. 只读列出目标 audit 当前字段。
2. 在 `workflow_audit.rs` 新增一个构造 helper。
3. 改一个真实调用点使用 helper。
4. 补字段一致性测试。
5. 更新 evidence / handoff / 当前入口。

验收：

- 原审计字段不丢。
- 至少一条真实写入路径调用新 helper。
- 测试通过。

必跑验证：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- `rustfmt --check src/workflow_audit.rs`
- `cargo test --lib`

必须输出：

- `evidence/2026-06-01-final-skeleton-01-audit-helper-slice-v1.md`
- `handoffs/2026-06-01-final-skeleton-01-audit-helper-slice-v1-result.md`

完成后停：

- 不必停；继续执行下一普通小任务，除非需要改 schema 或状态机。

### Skeleton-02：读模型纯派生小切片

任务名：

- `final-skeleton-02-read-model-derivation-slice-v1`

目标：

- 迁移一个低风险纯读模型派生函数到 `workflow_read_model.rs`。

优先候选：

- `derive_workflow_ledger_entries`
- 或一个更小的纯函数，例如 ledger entry 类型映射。

禁止：

- 不改 UI 展示语义。
- 不改底层事实。
- 不写 workflow state。
- 不把缺字段补编成事实。

执行步骤：

1. 找出一个没有写入副作用的读模型函数。
2. 迁移到 `workflow_read_model.rs`，或先做外层包装。
3. 保持原调用面尽量稳定。
4. 补一致性测试：迁移前后读模型结果等价。
5. 更新 evidence / handoff / 当前入口。

验收：

- 读模型结果保持一致。
- 前端类型不需要大改。
- 没有新增事实写入。

必跑验证：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- `rustfmt --check src/workflow_read_model.rs`
- `cargo test --lib`

必须输出：

- `evidence/2026-06-01-final-skeleton-02-read-model-derivation-slice-v1.md`
- `handoffs/2026-06-01-final-skeleton-02-read-model-derivation-slice-v1-result.md`

完成后停：

- 不必停；继续执行下一普通小任务，除非读模型拆分需要改事实结构。

### Skeleton-03：真实 Tauri 验收线设计

任务名：

- `final-skeleton-03-tauri-verification-line-design-v1`

目标：

- 先设计真实 Tauri 验收线，不直接改 UI。

禁止：

- 不把 Chrome headless 当完整验收。
- 不改业务代码。
- 不执行真实 Codex。
- 不读写 `/Users/yoyi/.codex`。

执行步骤：

1. 复核当前如何启动 Tauri 应用。
2. 复核是否已有截图或 browser 自动化工具。
3. 设计验收对象：
   - 首页。
   - 项目页。
   - 工作流画布。
   - 会话页。
   - 右侧栏。
   - 权限确认弹层。
4. 设计截图路径和命名。
5. 设计 evidence 模板。
6. 写后续实现任务包。

验收：

- 只交设计和下一步实现任务包。
- 如果设计不要求新增高风险权限或外部工具，可以继续实现验收线。
- 如果需要新增工具链、权限或读写敏感目录，必须先问用户。

必须输出：

- `evidence/2026-06-01-final-skeleton-03-tauri-verification-line-design-v1.md`
- `handoffs/2026-06-01-final-skeleton-03-tauri-verification-line-design-v1-result.md`

完成后：

- 普通情况下继续 Skeleton-04。
- 如果实现需要新增工具链、权限或读写敏感目录，必须停。

### Skeleton-04：真实 Tauri 验收线最小实现

任务名：

- `final-skeleton-04-tauri-verification-line-implementation-v1`

前置：

- Skeleton-03 已完成。

目标：

- 跑通一条真实 Tauri 窗口截图和交互验收路径。

禁止：

- 不改产品功能。
- 不执行真实 Codex。
- 不读写 `/Users/yoyi/.codex`。
- 不把普通浏览器截图冒充 Tauri 验收。

执行步骤：

1. 按 Skeleton-03 的设计启动 Tauri。
2. 截图首页、项目页、工作流页、右侧栏。
3. 如能稳定打开权限确认弹层，也截图。
4. 保存截图到 `evidence/`。
5. 写 evidence 说明哪些是真 Tauri，哪些不是。

验收：

- 至少有 3 张真实 Tauri 窗口截图。
- evidence 记录命令、路径、限制。

必跑验证：

- 取决于 Skeleton-03 设计。
- 至少跑 `npm run typecheck` 和 `npm run build`。

必须输出：

- `evidence/2026-06-01-final-skeleton-04-tauri-verification-line-implementation-v1.md`
- `handoffs/2026-06-01-final-skeleton-04-tauri-verification-line-implementation-v1-result.md`

完成后：

- 不必停；截图证据进入最终统一验收。

### Skeleton-05：画布参考源复核和能力清单

任务名：

- `final-skeleton-05-canvas-reference-research-v1`

目标：

- 专门复核画布相关参考源，形成“工作台项目画布应该具备什么、不应该做什么”的能力清单。

依据：

- `archive/decisions/2026-05-29-ui-reference-sources.md`
- React Flow / xyflow：画布底座、节点边模型、节点工具栏、缩放、小地图。
- n8n：节点配置面板。
- Langflow：可视化 builder 和节点逐步测试。
- Storybook：画布节点状态样例和 UI 回归基准。
- shadcn/ui：抽屉、按钮、表单、滚动区、确认弹层等 open code 组件组织方式。

禁止：

- 不接通用自动化平台。
- 不做复杂低代码编辑器。
- 不启动 MCP canvas run。
- 不改真实工作流事实。
- 不执行真实 Codex。

执行步骤：

1. 复核参考源文档中的画布相关条目。
2. 对照当前 `ProjectsView.tsx` 和独立 `CanvasView.tsx`。
3. 输出画布能力分层：
   - 必须有。
   - 后置。
   - 明确不做。
4. 输出画布风险清单。
5. 输出后续节点 schema 任务建议。

验收：

- 只做研究和任务拆分。
- 不改代码。

必须输出：

- `evidence/2026-06-01-final-skeleton-05-canvas-reference-research-v1.md`
- `handoffs/2026-06-01-final-skeleton-05-canvas-reference-research-v1-result.md`

完成后：

- 普通情况下继续 Skeleton-06。
- 如果发现当前画布方向和权威决策冲突，必须停。

### Skeleton-06：项目画布节点 schema 设计

任务名：

- `final-skeleton-06-project-canvas-node-schema-v1`

目标：

- 先定义项目工作流画布的节点、边、状态、详情面板数据 schema，再写画布实现。

禁止：

- 不写 UI 大改。
- 不改 workflow state JSON 结构。
- 不做通用自动化节点。
- 不把任务包暴露成主 UI。

执行步骤：

1. 定义节点类型：
   - 项目目标。
   - 总指导。
   - 开发线。
   - 验证线。
   - 回收线。
   - 权限请求。
   - 黑板候选。
   - 审计/证据引用。
2. 定义节点状态：
   - idle
   - ready_to_dispatch
   - running
   - waiting_for_permission
   - ready_for_review
   - needs_changes
   - accepted
   - failed
   - timed_out
3. 定义边类型：
   - 责任流转。
   - 证据引用。
   - 阻塞关系。
4. 定义节点详情面板数据。
5. 定义从 workflow state / project blackboard 派生到画布读模型的规则。
6. 输出后续实现任务包草案。

验收：

- 只写 schema / 计划。
- 不改状态文件结构。
- 不写实现。

必须输出：

- `docs/plans/2026-06-01-project-workflow-canvas-node-schema-v1.md`
- `evidence/2026-06-01-final-skeleton-06-project-canvas-node-schema-v1.md`
- `handoffs/2026-06-01-final-skeleton-06-project-canvas-node-schema-v1-result.md`

完成后：

- 如果 schema 不改事实结构，可继续 Skeleton-07。
- 如果 schema 需要改 workflow state JSON，必须停。

### Skeleton-07：画布组件状态样例

任务名：

- `final-skeleton-07-canvas-component-state-examples-v1`

目标：

- 在正式实现复杂画布前，建立画布节点、节点详情、权限队列、状态条的组件状态样例。

说明：

- 如果当前项目尚未接 Storybook，不必第一轮完整引入复杂 Storybook 流水线。
- 可以先用本地组件样例页面、测试 fixture 或轻量 stories 形式。

禁止：

- 不做真实工作流写入。
- 不改工作流状态机。
- 不做完整低代码编辑器。
- 不引入复杂视觉测试流水线。

执行步骤：

1. 建立画布节点状态样例：
   - 空画布。
   - 四角色节点。
   - 执行中。
   - 等待权限。
   - 失败。
   - 回收中。
   - accepted。
2. 建立节点详情样例：
   - 任务摘要。
   - handoff。
   - evidence。
   - review。
   - 黑板候选。
3. 建立权限确认队列样例。
4. 补 UI 测试或截图 evidence。

验收：

- AI 后续开发画布时有可读组件状态基准。
- 不要求完整 Storybook 上线。

必须输出：

- `evidence/2026-06-01-final-skeleton-07-canvas-component-state-examples-v1.md`
- `handoffs/2026-06-01-final-skeleton-07-canvas-component-state-examples-v1-result.md`

完成后：

- 普通情况下继续 Skeleton-08。

### Skeleton-08：React Flow 项目画布最小实现

任务名：

- `final-skeleton-08-react-flow-project-canvas-v1`

前置：

- Skeleton-06 节点 schema 已完成。
- Skeleton-07 组件状态样例已完成。

目标：

- 用 React Flow / xyflow 作为底座，实现项目工作流画布最小可用版本。

禁止：

- 不做通用自动化平台。
- 不做任意节点编辑器。
- 不启动 MCP canvas run。
- 不改 workflow state JSON 结构。
- 不把独立 CanvasView 当项目事实源。

执行步骤：

1. 建立项目画布读模型到 React Flow nodes / edges 的映射。
2. 节点展示角色、状态、最近证据和阻塞。
3. 边展示责任流转。
4. 支持缩放、平移、fit view。
5. 节点点击打开工作台自己的右侧详情，不在节点内塞满任务包。
6. 保留项目 workflow state / project blackboard 为权威。
7. 补测试和截图。

验收：

- 画布能表达当前项目四角色工作流。
- 节点详情来自右侧面板或抽屉。
- 不引入通用自动化语义。

必跑验证：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- 如涉及 Rust：`cargo test --lib`

必须输出：

- `evidence/2026-06-01-final-skeleton-08-react-flow-project-canvas-v1.md`
- `handoffs/2026-06-01-final-skeleton-08-react-flow-project-canvas-v1-result.md`

完成后：

- 普通情况下继续 Skeleton-09。

### Skeleton-09：画布节点详情和右侧面板收敛

任务名：

- `final-skeleton-09-canvas-node-detail-panel-v1`

目标：

- 把任务包、账本、状态机、子汇报、黑板候选等内部信息收进节点详情或右侧面板，避免工作流页重新变成任务包管理器。

禁止：

- 不改首页内容。
- 不把任务包放回主界面中心。
- 不做通用节点配置生态。
- 不启动 MCP canvas run。

执行步骤：

1. 整理项目工作流页现有面板。
2. 定义右侧详情分区：
   - 节点状态。
   - 当前任务。
   - handoff / evidence。
   - 权限请求。
   - 黑板候选。
   - 审计摘要。
3. 工作流主区域只保留画布。
4. 详情面板只显示当前选中节点相关信息。
5. 截图验证工作流页不再杂乱。

验收：

- 工作流页主视觉是画布。
- 内部协议不占据主界面。
- 有截图 evidence。

必须输出：

- `evidence/2026-06-01-final-skeleton-09-canvas-node-detail-panel-v1.md`
- `handoffs/2026-06-01-final-skeleton-09-canvas-node-detail-panel-v1-result.md`

完成后：

- 普通情况下继续 Skeleton-10。

### Skeleton-10：黑板候选持久状态 schema 设计

任务名：

- `final-skeleton-10-blackboard-candidate-schema-design-v1`

目标：

- 设计黑板候选持久确认状态，不实现写入。

禁止：

- 不改 workflow state JSON。
- 不写黑板持久状态。
- 不写正式记忆。
- 不做 UI 写入。

执行步骤：

1. 定义候选状态枚举。
2. 定义候选来源。
3. 定义候选目标类型。
4. 定义确认、拒绝、待处理、废弃的审计事件。
5. 定义是否需要迁移，以及迁移策略。
6. 定义控制核心命令签名。
7. 写实现任务包草案。

验收：

- 用户确认 schema / 迁移计划前不能实现。

必须输出：

- `docs/plans/2026-06-01-blackboard-candidate-persistence-schema-v1.md`
- `evidence/2026-06-01-final-skeleton-10-blackboard-candidate-schema-design-v1.md`
- `handoffs/2026-06-01-final-skeleton-10-blackboard-candidate-schema-design-v1-result.md`

完成后停：

- 必须让用户确认 schema。

### Skeleton-11：黑板候选持久确认最小实现

任务名：

- `final-skeleton-11-blackboard-candidate-persistence-implementation-v1`

前置：

- Skeleton-10 已完成并被用户接受。
- 用户接受内容必须明确包括：可以按已确认 schema / 迁移计划进入黑板候选持久状态最小实现。
- 如果用户只确认了 schema 方向，但没有确认实现写入，本任务不能开始。

目标：

- 实现黑板候选确认、拒绝、待处理的最小持久状态闭环。
- 这个闭环只改变候选状态和审计，不把候选升级成正式事实、正式记忆或 workflow 状态。

禁止：

- 不越过用户已确认的 schema / 迁移计划。
- 不新增未确认的 workflow state JSON 字段。
- 不直接写正式记忆。
- 不直接写正式事实。
- 不让知识引用直接成为记忆。
- 不让工具摘要直接推进 workflow 状态。
- 不执行真实 Codex。
- 不继续往项目画布右侧栏堆新主面板；UI 只展示候选状态和必要确认入口。

执行步骤：

1. 按确认后的 schema / 迁移计划改后端类型。
2. 新增控制核心命令。
3. 写 audit。
4. 更新读模型。
5. UI 只展示候选状态和必要确认入口，不伪装成正式事实。
6. 补测试。

验收：

- 合法确认/拒绝可记录。
- 非法升级被拒绝。
- 候选确认只停留在黑板候选层，不写正式事实、不写正式记忆、不推进 workflow 状态。
- 测试通过。

必跑验证：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- `cargo test --lib`

必须输出：

- `evidence/2026-06-01-final-skeleton-11-blackboard-candidate-persistence-implementation-v1.md`
- `handoffs/2026-06-01-final-skeleton-11-blackboard-candidate-persistence-implementation-v1-result.md`

完成后停：

- 不必停；普通情况下继续 Skeleton-12。若实现时需要越过已确认 schema / 迁移计划，必须停。

### Skeleton-12：适配器能力声明骨架

任务名：

- `final-skeleton-12-adapter-capability-registry-v1`

目标：

- 给现有 Codex 路径套上 AgentAdapter 能力声明，不实现其他 agent。

禁止：

- 不接 Claude / OpenCode / OpenClaw。
- 不改真实 Codex 执行语义。
- 不执行真实 Codex。
- 不显示未实现能力。

执行步骤：

1. 定义 AdapterCapability。
2. 定义 AgentAdapterDescriptor。
3. 给 Codex adapter 声明已有能力。
4. 读模型展示能力声明。
5. UI 不展示未实现能力按钮。
6. 补测试。

验收：

- Codex 不再是硬编码唯一形态。
- 未实现适配器不冒充可用。

必须输出：

- `evidence/2026-06-01-final-skeleton-12-adapter-capability-registry-v1.md`
- `handoffs/2026-06-01-final-skeleton-12-adapter-capability-registry-v1-result.md`

完成后停：

- 不必停；普通情况下继续 Skeleton-13。

### Skeleton-13：记忆治理 schema 设计

任务名：

- `final-skeleton-13-memory-governance-schema-design-v1`

目标：

- 设计记忆治理最小核心对象，不实现复杂检索。

禁止：

- 不接向量库。
- 不接图数据库。
- 不接 Obsidian 原生读写。
- 不把聊天自动写成长期记忆。
- 不写正式记忆。

执行步骤：

1. 定义 MemoryCandidate。
2. 定义 MemoryRecord。
3. 定义 MemoryScope。
4. 定义 MemorySourceRef。
5. 定义 MemoryLifecycleStatus。
6. 定义 MemoryConflict。
7. 定义 MemoryAuditRef。
8. 定义用户偏好记忆的优先级和确认规则。
9. 写实现任务包草案。

验收：

- 只交 schema 和计划。
- 用户确认前不实现。

必须输出：

- `docs/plans/2026-06-01-memory-governance-schema-v1.md`
- `evidence/2026-06-01-final-skeleton-13-memory-governance-schema-design-v1.md`
- `handoffs/2026-06-01-final-skeleton-13-memory-governance-schema-design-v1-result.md`

完成后停：

- 必须让用户确认。

### Skeleton-14：记忆治理最小实现

任务名：

- `final-skeleton-14-memory-governance-minimal-implementation-v1`

前置：

- Skeleton-13 已完成并被用户接受。
- 用户接受内容必须明确包括：可以按已确认 schema 做记忆候选生命周期最小实现。
- 如果用户只确认了 schema 方向，但没有确认候选实现，本任务不能开始。

目标：

- 实现记忆候选、确认状态、来源和审计的最小闭环。
- 这个闭环只处理记忆候选生命周期，不写正式长期记忆记录。

禁止：

- 不接向量库。
- 不接图数据库。
- 不接 Obsidian 原生读写。
- 不把普通聊天自动写正式记忆。
- 不写正式 MemoryRecord / 长期记忆。
- 不把“采纳候选”解释成“写入正式记忆”。
- 如果需要正式记忆写入，必须停下来让用户单独确认。
- 不继续往项目画布右侧栏堆新主面板；UI 只做只读和必要确认入口。

执行步骤：

1. 增加后端类型和读模型。
2. 增加候选创建命令。
3. 增加候选采纳 / 冻结 / 废弃命令；采纳只改变候选生命周期，不生成正式记忆记录。
4. 控制核心校验来源、作用域和权限。
5. 写 audit。
6. UI 只做只读和确认入口。
7. 补测试。

验收：

- 用户偏好记忆可以作为候选被确认，但不会直接写成正式长期记忆。
- 工作流总结只能先成为候选。
- 知识库材料不是记忆。
- 如果出现正式记忆写入需求，任务必须停止并回传。

必须输出：

- `evidence/2026-06-01-final-skeleton-14-memory-governance-minimal-implementation-v1.md`
- `handoffs/2026-06-01-final-skeleton-14-memory-governance-minimal-implementation-v1-result.md`

完成后停：

- 不必停；普通情况下继续 Skeleton-15。

### Skeleton-15：秘书核心只读模型

任务名：

- `final-skeleton-15-secretary-core-readonly-model-v1`

执行任务包：

- `2026-06-03-final-skeleton-15-secretary-core-readonly-model-v1.md`

目标：

- 建立秘书核心协作角色的只读上下文和建议模型。

禁止：

- 不把秘书固定成某个页面。
- 不让秘书直接改事实。
- 不让秘书直接派发任务。
- 不让秘书写正式记忆。

执行步骤：

1. 定义 SecretaryContext。
2. 定义 SecretarySuggestion。
3. 定义 SecretaryRiskSignal。
4. 定义 SecretaryMemoryCandidate。
5. 定义 SecretaryActionProposal。
6. 从现有读模型汇总状态、风险、待办、运行中工作流。
7. UI 只显示只读建议或候选；不把秘书固定塞进项目画布右侧栏。
8. 补测试。

验收：

- 秘书输出不是事实变更。
- 用户确认后才调用控制核心。

必须输出：

- `evidence/2026-06-03-final-skeleton-15-secretary-core-readonly-model-v1.md`
- `handoffs/2026-06-03-final-skeleton-15-secretary-core-readonly-model-v1-result.md`

完成后停：

- 不必停；普通情况下继续 Skeleton-16。

### Skeleton-16：项目工作流页最终收敛

任务名：

- `final-skeleton-16-project-workflow-surface-convergence-v1`

目标：

- 先把秘书摘要从通知 / 待办 / 审计 / 项目运行详情里移出，做成独立右侧入口。
- 再把项目工作流页进一步收敛成“画布主入口 + 节点详情/右侧抽屉”，减少任务包中心感。

禁止：

- 不改首页内容。
- 不做通用节点自动化平台。
- 不启动 MCP canvas run。
- 不把独立 CanvasView 合并为事实源。

执行步骤：

1. 先把秘书摘要做成独立右侧入口。
2. 确认通知、待办、审计、项目运行详情不再混入秘书摘要。
3. 只读对比当前 ProjectsView 和权威 UI 方向。
4. 写 UI 改动计划。
5. 如果 UI 改动不改变首页内容、事实源或画布权威，可继续改代码；如果要改变这些硬边界，必须先问用户。
6. 工作流页只保留画布和必要节点详情。
7. 内部账本、子汇报、状态机、任务包信息收进节点详情或右侧抽屉。
8. 截图验收。

验收：

- 项目工作流主入口清晰。
- 任务包不是主界面中心。
- 有截图证据。

必须输出：

- `evidence/2026-06-03-final-skeleton-16-project-workflow-surface-convergence-v1.md`
- `handoffs/2026-06-03-final-skeleton-16-project-workflow-surface-convergence-v1-result.md`

完成后停：

- 不必停；截图证据进入最终统一验收。

### Skeleton-17：格式债独立任务

任务名：

- `final-skeleton-17-rust-format-debt-isolated-v1`

目标：

- 单独处理 `cargo fmt --check` 历史格式债。

禁止：

- 不改业务逻辑。
- 不和任何功能改动混在一起。
- 未经用户确认不能执行。

执行步骤：

1. 先列出 `cargo fmt --check` diff 范围。
2. 停下来问用户是否接受单独格式化。
3. 用户确认后只运行格式化。
4. 跑完整测试。
5. evidence 只记录格式变化。

验收：

- `cargo fmt --check` 通过。
- 业务测试通过。
- 没有功能改动。

必须输出：

- `evidence/2026-06-01-final-skeleton-17-rust-format-debt-isolated-v1.md`
- `handoffs/2026-06-01-final-skeleton-17-rust-format-debt-isolated-v1-result.md`

完成后停：

- 不必停；进入最终统一验收。

## 每个小切片的固定要求

每个执行批次都必须：

1. 先有任务包或总包内明确批次范围。
2. 普通小任务可连续执行；触及硬边界时先问用户。
3. 只改允许范围。
4. 新增 evidence。
5. 新增 handoff。
6. 更新 `CURRENT.md` 和 `tasks/README.md`。
7. 明确“不接受为”。
8. 明确禁止事项是否遵守。
9. 有测试结果。
10. 普通小任务完成后继续执行，最终统一验收。

## 必跑验证

代码切片默认运行：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- `cargo test --lib`

新增 Rust 文件时：

- `rustfmt --check <新增文件>`

UI 切片：

- 需要截图证据。
- 若声明 Tauri 验收，必须是真实 Tauri 窗口。

只改文档：

- 不需要跑代码测试。
- 需要只读检查入口文档是否一致。

## 必须停下来问用户

出现以下情况必须停：

- 需要改变 workflow state JSON 结构。
- 需要迁移数据库。
- 需要黑板持久写入。
- 需要正式记忆。
- 需要秘书自动改事实。
- 需要真实 `codex exec` 或 `codex exec resume`。
- 需要读取或写入 `/Users/yoyi/.codex`。
- 需要读取 auth、token、`.env`、密钥、完整 transcript 或 rollout JSONL 正文。
- 需要接 Obsidian 原生功能。
- 需要接向量库或图数据库。
- 需要启动 MCP canvas run。
- 需要写真实业务项目目录。
- 需要改变首页内容。
- 需要把任务包管理器做成主界面。
- 需要做大范围格式化。

## 当前建议的下一执行批次

建议继续执行：

- `final-skeleton-10-blackboard-candidate-schema-design-v1`

范围：

- 只写黑板候选持久状态 schema。
- 只写迁移计划。
- 只写后续实现任务包草案。
- 不实现黑板持久写入。
- 不改 workflow state JSON。
- 不写正式事实或正式记忆。
- 不做 UI 写入。

完成后必须停下来让用户确认 schema / 迁移计划；确认前不能执行 Skeleton-11。

## 本总包验收方式

用户验收这份总包时，只需要确认：

1. 是否接受“先长最终骨架，不直接做大功能”。
2. 是否接受每个阶段都拆成小切片。
3. 是否接受普通小任务连续执行，最终统一验收。
4. 是否接受推荐顺序。
5. 是否接受真实 Tauri 最小截图线已建立，完整自动化验收继续后置到单独任务。

## 当前状态

本文件是当前总执行任务包。

它不是已完成实现。
它也不是允许一次性执行所有阶段的授权。

`final-skeleton-00` 到 `final-skeleton-14` 已完成；当前下一步任务包是 `2026-06-03-final-skeleton-15-secretary-core-readonly-model-v1.md`。每个批次仍需 evidence、handoff 和验证；普通微步骤不再强制逐个单独记录。触及正式记忆、秘书自动改事实、真实 Codex 或 `/Users/yoyi/.codex` 时必须停下来。
