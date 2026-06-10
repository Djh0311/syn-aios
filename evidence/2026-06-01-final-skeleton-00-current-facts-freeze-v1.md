# Final Skeleton 00 Current Facts Freeze v1 Evidence

日期：2026-06-01

## 本轮结论

先说薄弱点：当前原型已经有 Codex 工作流编排、控制核心 helper、项目黑板只读模型和初步模块边界，但还不是最终本地 AI 工作台骨架。

依据：

- `src-tauri/src/lib.rs` 仍有 12347 行，同时承担状态 mutation、派发、读模型、工作流机器和测试。
- `src/views/ProjectsView.tsx` 仍有 2763 行，项目页承载大量任务包、账本、状态机、黑板和派发细节。
- 黑板目前是只读候选模型，没有持久确认状态。
- 记忆治理和秘书核心协作层仍主要停在设计层，还没有最小实现。
- 真实 Tauri 窗口验收线还没有固定下来。

本轮只做事实冻结：

- 未改原型代码。
- 未改 workflow state。
- 未执行真实 Codex。
- 未读取或写入 `/Users/yoyi/.codex`。
- 未跑代码测试，因为本切片只读和写记录。

## 读过的依据

| 文件 | 用途 |
|---|---|
| `CURRENT.md` | 当前事实、完成/未完成能力和下一步边界。 |
| `AUTHORITY.md` | 当前权威入口和设计草案索引。 |
| `tasks/README.md` | 当前任务队列、总执行包状态和禁止边界。 |
| `tasks/2026-06-01-final-workbench-skeleton-execution-package-v1.md` | 总执行包和 Skeleton-00 要求。 |
| `docs/workbench-system-architecture-v1.md` | 最终工作台架构层：控制核心、黑板、适配器、事实层、读模型、秘书、记忆治理。 |
| `docs/plans/2026-06-01-workbench-architecture-implementation-plan-v1.md` | 已完成 Task A-E 和控制核心第二切片，剩余风险。 |
| `evidence/2026-06-01-control-core-second-slice-boundary-extraction-v1.md` | 当前后端边界拆分最新事实。 |
| `evidence/2026-06-01-project-blackboard-minimal-read-model-d-v1.md` | 黑板当前是只读模型的依据。 |

## 已完成骨架能力

| 能力 | 当前完成度 | 依据 |
|---|---|---|
| 项目单元 | 已有项目入口、项目 workflow 读模型和项目页主入口。 | `CURRENT.md`、`ProjectsView.tsx`。 |
| Codex 会话管理 | 已有会话中心、项目会话入口、会话绑定。 | `CURRENT.md`。 |
| 工作流最小状态 | 已有 workflow state v0 JSON、工作项状态推进、节点绑定、派发记录。 | `CURRENT.md`、`src-tauri/src/lib.rs`。 |
| 控制核心 helper | 已有 `control_core.rs`，覆盖状态推进、派发、权限、总指导回收和黑板候选边界校验。 | `evidence/2026-06-01-control-core-command-convergence-v1.md`。 |
| 状态文件读写边界 | 已有 `workflow_state_store.rs`，承载 read/validate/write/backup/atomic write。 | `evidence/2026-06-01-control-core-second-slice-boundary-extraction-v1.md`。 |
| 审计构造边界 | 已有 `workflow_audit.rs`，目前只抽出 `work_item_state_changed`。 | 同上。 |
| 读模型边界 | 已有 `workflow_read_model.rs`，目前只抽出项目黑板集合派生。 | 同上。 |
| 项目黑板 | 已有 `ProjectBlackboard` 等读模型和项目页只读展示。 | `evidence/2026-06-01-project-blackboard-minimal-read-model-d-v1.md`。 |
| 项目工作流画布权威 | 已有决策：项目页是项目工作流主入口，独立 CanvasView 是实验/模板/后置能力。 | `decisions/2026-06-01-project-workflow-canvas-authority-v1.md`、`CURRENT.md`。 |
| 工作流机器 | 已有四角色 runner 和 stub 测试，历史上做过真实 demo 闭环。 | `CURRENT.md`。 |

## 未完成骨架能力

| 能力 | 缺口 | 风险 |
|---|---|---|
| 最终架构拆分 | `lib.rs` 仍是大聚合文件。 | 新功能继续堆进一个文件，边界会反复变糊。 |
| 完整事件账本 | audit 仍散落在 mutation 函数里。 | 事件字段约定难查，后续审计迁移风险高。 |
| 黑板候选持久确认 | 当前只有只读候选，没有确认/拒绝/废弃持久状态。 | 候选、事实、记忆、审计容易混。 |
| 正式记忆治理 | 还没有 MemoryCandidate / MemoryRecord 的确认闭环。 | 检索命中、知识引用、用户偏好可能被误当正式记忆。 |
| 秘书核心协作层 | 还没有只读 SecretaryContext / Suggestion / RiskSignal。 | 秘书能力如果直接做 UI，容易越权改事实。 |
| 适配器能力声明 | Codex 路径还没有统一 AgentAdapter 能力声明读模型。 | Codex 容易被写死成系统唯一形态。 |
| 项目工作流页收敛 | 项目页仍有大量任务包和内部面板。 | 产品观感会回到任务包管理器。 |
| 真实 Tauri 验收线 | 目前主要依赖普通浏览器和 headless 验收。 | UI 在真实桌面壳里的状态没有稳定证据。 |
| 格式债 | `cargo fmt --check` 历史失败仍存在。 | 后续改动 diff 噪声大，但不能和业务混做。 |

## 当前主要代码模块和职责

| 文件或模块 | 当前职责 | 目标层 |
|---|---|---|
| `src-tauri/src/lib.rs` | 状态 mutation、派发、读模型、snapshot、工作流机器、测试。 | 应继续拆到应用服务、事实层、读模型、适配器、测试。 |
| `src-tauri/src/types.rs` | 后端请求/响应/读模型类型。 | 类型协议层。 |
| `src-tauri/src/commands.rs` | Tauri command 包装。 | 应用服务入口包装。 |
| `src-tauri/src/control_core.rs` | 状态、派发、权限、黑板候选边界校验。 | 控制核心。 |
| `src-tauri/src/workflow_state_store.rs` | workflow state 文件读写边界。 | 事实层 store。 |
| `src-tauri/src/workflow_audit.rs` | 审计事件构造 helper。 | 审计/事件边界。 |
| `src-tauri/src/workflow_read_model.rs` | 读模型派生 helper。 | 读模型层。 |
| `src-tauri/src/codex_db.rs` | Codex 索引 / 会话读取相关。 | Codex 适配器 / 数据适配。 |
| `src-tauri/src/mcp/**` | 独立可编辑画布和 MCP 运行骨架。 | 适配器层 / 实验画布运行层。 |
| `src/App.tsx` | 前端全局状态、命令调用、路由入口。 | 界面层 + 前端应用协调。 |
| `src/views/ProjectsView.tsx` | 项目页、工作流、黑板、任务包、权限、面板。 | 界面层，但仍承载过多内部细节。 |
| `src/views/CanvasView.tsx` | 独立可编辑画布。 | 实验/模板画布界面，不是项目 workflow state 事实源。 |
| `src/lib/types.ts` | 前端类型镜像。 | 读模型协议。 |

## 剩余风险

| 区域 | 风险 | 依据 |
|---|---|---|
| `lib.rs` | 仍是最大耦合点，写入、读模型、执行路径和测试混在一起。 | `wc -l` 显示 12347 行。 |
| `ProjectsView.tsx` | 项目页仍可能显得像任务包/内部状态中心。 | `wc -l` 显示 2763 行；Task D evidence 已指出仍需收敛。 |
| 工作流状态 | v0 JSON 仍是过渡事实层，事件、审计、当前快照、黑板没有彻底分离。 | 架构计划和第二切片 evidence。 |
| 黑板 | 只有只读候选，没有持久确认。 | Task D evidence。 |
| 控制核心 | helper 已有，但不是最终控制核心模块体系。 | command convergence 和 second slice evidence。 |
| 读模型 | 只抽出很少 helper，`derive_workflow_read_model` 和账本仍在 `lib.rs`。 | second slice evidence。 |
| 真实验收 | 缺少稳定 Tauri 真窗口截图线。 | `CURRENT.md` 未完成项。 |

## 最终骨架完成度矩阵

| 架构能力 | 状态 | 完成度判断 |
|---|---|---|
| 项目单元隔离 | 部分完成 | 有项目和 workflow 读模型，但隔离策略还未系统化。 |
| 控制核心 | 部分完成 | 有 helper 和调用点，未形成完整核心模块。 |
| 项目黑板 | 只读完成 | 可展示候选，不可持久确认。 |
| 事实层 | 部分完成 | 有 JSON store 边界，事件/审计/快照仍混。 |
| 审计 | 起步完成 | 已抽一类 audit，其他仍散落。 |
| 读模型 | 起步完成 | 已抽项目黑板集合派生，主要派生仍在 `lib.rs`。 |
| 适配器 | 不完整 | Codex 可用，但还没有能力声明骨架。 |
| 记忆治理 | 未实现 | 只有设计草案和黑板候选展示。 |
| 秘书核心协作 | 未实现 | 还没有只读模型。 |
| 项目工作流 UI | 部分完成 | 主入口已收敛，但内部面板仍重。 |
| 真实 Tauri 验收 | 未完成 | 还没有固定截图线。 |
| 格式债 | 未处理 | `cargo fmt --check` 历史失败。 |

## 禁止事项执行情况

| 禁止项 | 结果 |
|---|---|
| 不改代码 | 已遵守，未修改原型代码。 |
| 不改 workflow state | 已遵守。 |
| 不执行真实 Codex | 已遵守。 |
| 不读写 `/Users/yoyi/.codex` | 已遵守。 |
| 不读取密钥、token、`.env`、完整 transcript | 已遵守。 |
| 不迁移数据库 | 已遵守。 |
| 不写真实业务项目目录 | 已遵守。 |

## 验证

本切片是只读冻结和记录输出，没有跑代码测试。

原因：

- 任务包明确只读复核和输出 evidence / handoff。
- 没有修改原型代码。
- 没有修改工作流事实文件。
