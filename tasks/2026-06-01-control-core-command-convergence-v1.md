# 任务包：控制核心命令收敛 v1

## 任务名

控制核心命令收敛 v1。

## 所属开发线

桌面应用线 / 后端架构线 / 工作流状态线。

总指导线回收。

## 当前理解

这次不是继续加新产品能力，而是把已经存在的工作流状态推进、权限确认、派发、回收、完成判定这些关键动作，收回到后端控制核心里。

大白话：

现在工作台已经有项目、工作流、黑板只读模型和若干状态动作，但边界还不够硬。下一步要先规定并实现：哪些动作只能由后端核心判断，界面只能发请求；黑板里的候选怎么等待确认、怎么拒绝、怎么被允许升级；关键动作至少要留下审计依据。

本任务包允许一次性做三件事：

1. 任务 E：控制核心命令收敛。
2. 黑板候选确认边界的最小实现或接口占位。
3. 最小审计骨架。

不允许把范围扩大到秘书完整能力、记忆真实存储、Obsidian、向量库、多智能体适配器或真实 Codex 执行。

## 已知

- 当前最终架构已经选择：本地模块化单体运行形态 + 项目单元隔离 + 控制核心 + 项目黑板 + 能力适配器 + 事件账本和当前快照 + 核心记忆治理 + 秘书核心协作层。
- 控制核心应该掌握事实、权限、状态机、策略和审计。
- 只有控制核心能把黑板候选、汇报、工具摘要或建议升级为正式事实。
- 任务 A 架构只读审计已完成。
- 任务 B 保守拆模块切片已完成。
- 任务 C 项目工作流画布权威收敛已完成一个保守切片。
- 任务 D 项目黑板最小只读切片已完成，但没有黑板写入命令。
- 当前 `workflow state JSON` 仍是过渡事实层。
- 当前代码仍有明显风险：`src-tauri/src/lib.rs` 承担过多命令、模型、状态读写、派发、工作流机器、读模型和测试职责。

## 未知

- 现有 workflow state 里哪些审计数组和控制记录字段足够复用，执行者必须读代码确认，不能凭空新增结构。
- 现有前端哪些按钮会直接推进状态，执行者必须读 `ProjectsView.tsx` 和相关测试确认。
- 是否需要新增独立 Rust 模块，例如 `control_core.rs`。如果强拆会扩大风险，可以先用小范围 helper，不要为了形式强拆。
- 当前 app 是否已有某些命令天然等价于控制核心命令。执行者必须先列出来，再决定是包一层还是移动逻辑。

## 假设

- 本任务只做本地工作台内部控制边界，不接真实业务项目。
- 本任务不迁移数据库，不改变 workflow state JSON 结构。
- 如果现有审计字段不足，本任务先写最小审计记录或返回缺口 warning，不做新 schema 迁移。
- 如果黑板候选无法在不改 JSON 结构的情况下持久确认，本任务可以先实现后端拒绝和只读状态解释，但不能伪造“已正式升级”。

## 依据

- `docs/workbench-system-architecture-v1.md`：
  - 控制核心掌握事实、权限、状态机、策略和审计。
  - 只有控制核心能把候选升级为正式事实、正式记忆、审计事件或状态变化。
  - 读模型不是事实源。
- `docs/plans/2026-06-01-workbench-architecture-implementation-plan-v1.md`：
  - 任务 E 目标是把状态机、权限、派发、回收和完成判定收回后端核心。
  - 当前建议不要直接补黑板写 JSON 接口。
  - 如继续黑板写入，必须先设计候选如何经控制核心确认后升级。
- `decisions/2026-06-01-architecture-module-split-guardrail-v1.md`：
  - 保守拆分，不第一批动状态机、真实 Codex 执行、工作流机器、MCP canvas run 或任务包产品规则。
- `decisions/2026-06-01-project-workflow-canvas-authority-v1.md`：
  - 项目工作流画布以项目 workflow state 和项目黑板为权威。
  - 独立 `CanvasView` 不作为项目工作流事实源。
- `evidence/2026-06-01-project-blackboard-minimal-read-model-d-v1.md`：
  - 黑板当前只读，未新增写入命令。
- `handoffs/2026-06-01-project-blackboard-task-d-receive-and-next-v1-result.md`：
  - 下一步建议为控制核心命令收敛。

## 薄弱点

- 当前代码边界仍然偏宽，硬拆会扩大风险。
- 黑板只有只读模型，直接做写入命令会越过控制核心确认边界。
- 如果前端仍能绕过后端直接推进状态，控制核心只是名义存在。
- 如果审计只在 UI 里显示，不在后端状态动作里产生，就不能算控制核心收敛完成。
- 如果本任务顺手做秘书、记忆、适配器或 Obsidian，会把边界再次搅乱。

## 目标

完成一个保守但可执行的控制核心切片：

1. 梳理现有会改变工作流事实的命令和前端动作。
2. 建立后端控制核心入口或 helper，用于统一判断关键动作是否允许。
3. 至少覆盖这些动作类别：
   - 工作项状态推进。
   - 权限确认。
   - 派发准备。
   - 结果回收。
   - 完成判定。
   - 黑板候选确认 / 拒绝 / 待处理边界。
4. 后端拒绝非法状态转移，UI 只能展示拒绝结果，不能自行补状态。
5. 关键动作产生或复用审计记录。
6. 前端只调用后端命令，不在组件里拼出“事实已变更”的假状态。
7. 增加离线测试覆盖成功路径、非法路径和黑板候选不能直接升级的路径。

## 非目标

- 不执行真实 `codex exec`。
- 不执行真实 `codex exec resume`。
- 不写 `/Users/yoyi/.codex`。
- 不读取 `/Users/yoyi/.codex`。
- 不读取 auth、token、`.env`、密钥、授权文件。
- 不读取完整 transcript 或 rollout JSONL 正文。
- 不启动 MCP canvas run。
- 不改真实业务项目目录。
- 不迁移 SQLite。
- 不改变 workflow state JSON 结构，除非另开迁移计划并经用户确认。
- 不新增 Obsidian 原生功能。
- 不接向量库。
- 不接图数据库。
- 不做秘书完整 UI 或秘书自动改事实。
- 不做正式记忆存储。
- 不新增多 agent 适配器生态。
- 不把独立 `CanvasView` 和项目 workflow state 合一。
- 不改首页内容。
- 不把任务包管理器重新做成产品主界面。

## 允许读取

允许读取：

- `/Users/yoyi/workspace/AGENTS.md`
- `/Users/yoyi/workspace/product-line/CURRENT.md`
- `/Users/yoyi/workspace/product-line/AUTHORITY.md`
- `/Users/yoyi/workspace/product-line/README.md`
- `/Users/yoyi/workspace/product-line/tasks/README.md`
- `/Users/yoyi/workspace/product-line/tasks/2026-06-01-control-core-command-convergence-v1.md`
- `/Users/yoyi/workspace/product-line/docs/workbench-system-architecture-v1.md`
- `/Users/yoyi/workspace/product-line/docs/plans/2026-06-01-workbench-architecture-implementation-plan-v1.md`
- `/Users/yoyi/workspace/product-line/docs/workflow-task-package-design-v1.md`
- `/Users/yoyi/workspace/product-line/docs/memory-layer-design-v1.md`
- `/Users/yoyi/workspace/product-line/decisions/2026-06-01-architecture-module-split-guardrail-v1.md`
- `/Users/yoyi/workspace/product-line/decisions/2026-06-01-project-workflow-canvas-authority-v1.md`
- `/Users/yoyi/workspace/product-line/evidence/2026-06-01-workbench-architecture-readonly-audit-v1.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-06-01-workbench-architecture-readonly-audit-v1-result.md`
- `/Users/yoyi/workspace/product-line/evidence/2026-06-01-workbench-architecture-task-b-conservative-module-split-v1.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-06-01-workbench-architecture-task-b-conservative-module-split-v1-result.md`
- `/Users/yoyi/workspace/product-line/evidence/2026-06-01-project-workflow-canvas-authority-convergence-c-v1.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-06-01-project-workflow-canvas-authority-convergence-c-v1-result.md`
- `/Users/yoyi/workspace/product-line/evidence/2026-06-01-project-blackboard-minimal-read-model-d-v1.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-06-01-project-blackboard-minimal-read-model-d-v1-result.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-06-01-project-blackboard-task-d-receive-and-next-v1-result.md`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/lib/types.ts`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/lib/types/canvas.ts`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/App.tsx`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/views/CanvasView.tsx`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/tests/`

允许只读真实 workflow state 的必要摘要：

- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`

只读真实 workflow state 时只能读取完成本任务所需的结构和字段摘要，不要把完整内容复制进 evidence。

## 禁止读取

- `/Users/yoyi/.codex`
- auth、token、`.env`、密钥、授权文件
- 完整 transcript 正文
- rollout JSONL 正文
- 真实业务项目源码，除非用户另行指定并批准

## 允许写入

允许修改代码和测试：

- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri/src/*.rs`，仅限新增小模块承载控制核心 helper 或应用服务边界
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/lib/types.ts`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/views/ProjectsView.tsx`，仅限必要的命令调用、错误展示和只读状态展示
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/tests/`

允许写构建产物：

- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/dist/`

允许写记录：

- `/Users/yoyi/workspace/product-line/evidence/2026-06-01-control-core-command-convergence-v1.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-06-01-control-core-command-convergence-v1-result.md`

允许更新当前入口：

- `/Users/yoyi/workspace/product-line/CURRENT.md`
- `/Users/yoyi/workspace/product-line/tasks/README.md`

如确实需要更新：

- `/Users/yoyi/workspace/product-line/AUTHORITY.md`
- `/Users/yoyi/workspace/product-line/README.md`
- `/Users/yoyi/workspace/product-line/docs/plans/2026-06-01-workbench-architecture-implementation-plan-v1.md`

但不要把普通执行结果写成新的权威规则。

## 禁止写入

- 禁止写 `/Users/yoyi/.codex`。
- 禁止写真实业务项目目录。
- 禁止手工改真实 workflow state JSON。
- 禁止迁移 SQLite。
- 禁止新增 Obsidian vault、向量库、图数据库文件。
- 禁止写 auth、token、`.env`、密钥、授权文件。
- 禁止批量格式化 `src/mcp/**`。
- 禁止为了通过 `cargo fmt --check` 大范围格式化既有无关文件。

## 实施步骤

### 第 1 步：只读梳理现有命令

先列出现有会改变事实的入口：

- Tauri command 包装。
- 工作流状态写入函数。
- 派发准备函数。
- 派发完成 / 失败 / 超时回写函数。
- 总指导回收函数。
- 权限确认函数。
- 黑板读模型派生函数。
- 前端触发这些动作的按钮和确认弹层。

输出到 evidence：

- 命令名。
- 当前所在文件。
- 是否改变事实。
- 是否已有后端校验。
- 是否已有审计记录。
- 目标归属：控制核心、应用服务、适配器、读模型、纯 UI。

### 第 2 步：建立控制核心最小边界

实现方式由执行者根据代码决定，但必须满足：

- 控制核心是后端规则入口，不是前端 helper。
- 控制核心负责判断“这个动作在当前状态能不能做”。
- UI 只能调用命令并展示结果。
- 非法动作返回明确错误或 rejected 结果。
- 不改变 workflow state JSON 结构。

建议优先做小 helper，不要强拆大模块：

- `validate_workflow_transition`
- `validate_permission_decision`
- `validate_dispatch_prepare`
- `validate_director_review`
- `validate_blackboard_promotion`

命名可以按现有代码风格调整。

### 第 3 步：收敛已有命令

把已有关键命令接入控制核心校验：

- 离线 prepared dispatch。
- role handoff。
- director review。
- 工作流机器相关状态收口中已有的完成判定。
- 权限确认路径。
- 黑板候选确认 / 拒绝边界。

如果某个命令范围过大，先包最外层校验，不做大重构。

### 第 4 步：黑板候选确认边界

本任务只允许做最小边界：

- 黑板条目默认仍是候选。
- 黑板候选不能直接升级为正式事实。
- 黑板候选不能直接写正式记忆。
- 知识引用不能直接当记忆。
- 工具摘要不能直接推进 workflow 状态。
- 用户确认动作必须调用控制核心。
- 控制核心如果发现缺少迁移计划或缺少目标事实类型，必须拒绝或返回“待设计”，不能伪造成功。

允许实现：

- `confirm_candidate` / `reject_candidate` / `mark_candidate_pending` 这类后端命令或内部函数。
- 如果无法持久化确认状态，不要新增 JSON schema；可以先把行为限制为后端拒绝、测试覆盖和 evidence 说明。

不允许实现：

- 直接把黑板候选写进正式记忆。
- 直接把黑板候选写进 workflow node state。
- 直接新增 `project_blackboards` 持久写入 schema。

### 第 5 步：最小审计骨架

关键动作必须有事件或审计依据。

优先复用现有 workflow state 里已经存在的审计 / control / attempt / review 记录结构。

至少覆盖：

- 合法状态推进。
- 非法状态推进被拒绝。
- 权限确认。
- 派发准备。
- 回收结论。
- 黑板候选确认被接受、拒绝或因为边界不足被拒绝。

如果现有结构无法写某类审计：

- 不新增迁移。
- 在 evidence 里记录“当前无法审计的动作、原因、后续需要的 schema”。

### 第 6 步：前端只接命令结果

前端只做：

- 调用后端命令。
- 展示成功、拒绝、待确认、缺口 warning。
- 展示黑板候选状态。
- 展示审计摘要。

前端不能：

- 自行把节点标成完成。
- 自行把候选标成正式事实。
- 自行把权限标成已通过。
- 自行补编后端没有返回的事实。

### 第 7 步：测试和记录

必须增加或更新测试，覆盖：

- 合法状态转移通过。
- 非法状态转移被拒绝。
- UI 无法绕过后端直接推进状态。
- 黑板候选不能直接升级为正式事实。
- 记忆候选不能直接成为正式记忆。
- 知识引用不能直接当记忆。
- 关键动作有审计或明确缺口记录。

## 验收标准

必须满足：

- 后端存在明确的控制核心校验入口或 helper。
- 至少一个现有状态推进路径已通过控制核心校验。
- 至少一个现有派发 / 回收路径已通过控制核心校验。
- 至少一个权限确认路径已通过控制核心校验。
- 黑板候选升级路径不会直接写正式事实或正式记忆。
- 非法状态转移会被后端拒绝。
- 前端不能绕过后端把关键事实改成成功态。
- 关键动作有审计记录，或 evidence 明确写出因现有结构限制无法审计的缺口。
- `CURRENT.md` 和 `tasks/README.md` 更新当前状态。
- 新增 evidence 和 handoff。

不接受为：

- 控制核心完整最终版。
- 真实业务自动编排完成。
- 秘书能力完成。
- 记忆层真实存储完成。
- 黑板可自由写入完成。
- Obsidian 内置完成。
- 多 agent 适配器完成。
- 事件账本完整迁移完成。

## 必跑验证

在 `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell` 下运行：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- `cargo test --lib`

如果新增 Rust 文件：

- 对新增文件运行 `rustfmt --check`。

不要为了 `cargo fmt --check` 去批量格式化既有 `src/lib.rs` 或 `src/mcp/**`。如果 `cargo fmt --check` 仍因历史格式失败，必须在 handoff 里说明。

## 必须输出

新增 evidence：

- `/Users/yoyi/workspace/product-line/evidence/2026-06-01-control-core-command-convergence-v1.md`

新增 handoff：

- `/Users/yoyi/workspace/product-line/handoffs/2026-06-01-control-core-command-convergence-v1-result.md`

evidence 必须包含：

- 读过哪些权威文档。
- 改了哪些命令和 helper。
- 哪些动作已纳入控制核心。
- 哪些非法路径会被拒绝。
- 黑板候选边界怎么处理。
- 审计记录怎么处理。
- 测试命令和结果。
- 明确没有做哪些禁止事项。

handoff 必须包含：

- 本轮完成了什么。
- 不接受为什么。
- 改了哪些文件。
- 测试结果。
- 仍然存在的架构风险。
- 下一步建议。

## 停止条件

出现以下情况必须停下来问用户，不能自行继续：

- 需要读取 `/Users/yoyi/.codex`。
- 需要读取 auth、token、`.env`、密钥、授权文件。
- 需要读取完整 transcript。
- 需要执行真实 `codex exec` 或 `codex exec resume`。
- 需要写真实业务项目目录。
- 需要迁移数据库。
- 需要改变 workflow state JSON 结构。
- 需要把黑板候选持久写成正式事实。
- 需要把记忆候选写成正式记忆。
- 需要接 Obsidian 原生功能。
- 需要引入向量库或图数据库。
- 需要启动 MCP canvas run。
- 需要把独立 `CanvasView` 和项目 workflow state 合一。
- 需要改变首页内容。
