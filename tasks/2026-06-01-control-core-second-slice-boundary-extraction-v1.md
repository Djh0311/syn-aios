# 任务包：控制核心第二切片边界拆分 v1

## 任务名

控制核心第二切片边界拆分 v1。

## 所属开发线

桌面应用线 / 后端架构线 / 工作流状态线。

总指导线回收。

## 当前理解

这次不是继续做新功能，也不是做黑板写入。上一轮 `control-core-command-convergence-v1` 已经把关键命令先接入后端控制核心 helper，但 `src-tauri/src/lib.rs` 仍然同时承担状态读写、审计事件构造、读模型派生、派发、工作流机器、测试等职责。

大白话：

上一轮先把“门卫”立起来了。这一轮不换房子结构，只把最容易继续混乱的几类东西分出清楚位置：状态文件读写放一处，审计事件构造放一处，读模型派生放一处。不要改数据格式，不要加新业务，不要让测试大面积重写。

## 已知

- Task A 架构只读审计已完成。
- Task B 保守拆模块切片已完成，已拆出后端类型、Tauri command 包装和前端 editable canvas 纯类型。
- Task C 项目工作流画布权威收敛保守切片已完成。
- Task D 项目黑板最小只读切片已完成。
- Task E 控制核心命令收敛保守切片已完成。
- 当前 `src-tauri/src/lib.rs` 仍约 12353 行。
- 当前已有模块：
  - `src-tauri/src/types.rs`
  - `src-tauri/src/commands.rs`
  - `src-tauri/src/control_core.rs`
  - `src-tauri/src/codex_db.rs`
- 当前 `lib.rs` 中可见的混合职责包括：
  - `read_workflow_state_snapshot`
  - `initialize_workflow_state_at`
  - `write_prepared_dispatch`
  - `write_started_dispatch`
  - `write_completed_dispatch`
  - `write_failed_dispatch`
  - `write_readback_dispatch`
  - 大量 `audit_events` 追加逻辑
  - `project_workflow_summaries`
  - `project_blackboards_from_workflows`
  - `derive_workflow_read_model`
  - `derive_workflow_ledger_entries`
  - `build_snapshot`

## 未知

- 哪些 helper 的可见性可以安全调整，执行者必须读代码后决定，不能机械移动。
- 读模型派生函数之间的私有依赖较多，是否能在本轮安全移动不确定。
- 状态写入 helper 是否能全部移动不确定。若一次移动会牵扯太多测试，必须缩小范围。
- 审计事件构造是否已有隐含字段约定，必须从现有测试和 JSON 写入逻辑确认。

## 假设

- 本任务只做代码边界拆分和少量 helper 提取，不改变行为。
- 本任务不改变 workflow state JSON 结构。
- 本任务不迁移数据库。
- 本任务不执行真实 Codex。
- 如果某类函数移动风险过高，可以先做“新模块 + 小包装 helper + 局部调用接入”，不追求一次拆完。

## 依据

- `CURRENT.md`：当前建议进入控制核心第二切片，把读模型派生、状态写入和审计事件进一步拆出模块。
- `docs/plans/2026-06-01-workbench-architecture-implementation-plan-v1.md`：下一步建议为控制核心第二切片。
- `docs/workbench-system-architecture-v1.md`：
  - 控制核心管事实和权限。
  - 事件和审计保证可追踪。
  - 读模型服务界面，不是事实源。
- `evidence/2026-06-01-control-core-command-convergence-v1.md`：
  - 控制核心 helper 已存在。
  - `lib.rs` 仍然很大。
  - 黑板候选没有持久确认状态。
  - `read_workflow_node_dispatch_result` 仍是状态写入缺口。
- `handoffs/2026-06-01-control-core-command-convergence-v1-result.md`：
  - 下一步建议是控制核心第二切片、黑板 schema 设计或权限流 followup。

## 薄弱点

- 这个任务如果贪多，会变成大重构，风险比收益大。
- 单纯移动文件不等于架构完成；必须保持行为不变并补证据。
- `lib.rs` 里的测试很多，移动私有函数可能导致大量可见性调整。
- 读模型派生依赖底层 JSON 和类型，硬拆可能引入循环依赖。
- `cargo fmt --check` 已知可能因历史格式问题失败，不能借这个任务批量格式化无关文件。

## 目标

完成一个保守的第二切片：

1. 新增或使用小模块承载状态文件读写边界。
2. 新增或使用小模块承载审计事件构造边界。
3. 新增或使用小模块承载读模型派生边界。
4. 至少把一条真实状态写入路径改成通过新边界 helper。
5. 至少把一种审计事件构造改成通过新边界 helper。
6. 至少把一种读模型派生入口或包装改成通过新边界 helper。
7. 保证已有行为和测试不退化。
8. 产出 evidence，说明哪些边界已拆，哪些仍留在 `lib.rs`。

## 非目标

- 不做黑板持久写入。
- 不设计黑板候选持久确认 schema。
- 不写正式记忆。
- 不做秘书能力。
- 不接 Obsidian。
- 不接向量库或图数据库。
- 不做多 agent 适配器注册。
- 不执行真实 `codex exec`。
- 不执行真实 `codex exec resume`。
- 不读写 `/Users/yoyi/.codex`。
- 不读取 auth、token、`.env`、密钥、授权文件。
- 不读取完整 transcript 或 rollout JSONL 正文。
- 不启动 MCP canvas run。
- 不写真实业务项目目录。
- 不迁移 SQLite。
- 不改变 workflow state JSON 结构。
- 不把独立 `CanvasView` 和项目 workflow state 合一。
- 不改首页内容。
- 不把任务包管理器做成产品主界面。

## 允许读取

允许读取：

- `/Users/yoyi/workspace/AGENTS.md`
- `/Users/yoyi/workspace/product-line/CURRENT.md`
- `/Users/yoyi/workspace/product-line/AUTHORITY.md`
- `/Users/yoyi/workspace/product-line/README.md`
- `/Users/yoyi/workspace/product-line/tasks/README.md`
- `/Users/yoyi/workspace/product-line/tasks/2026-06-01-control-core-second-slice-boundary-extraction-v1.md`
- `/Users/yoyi/workspace/product-line/tasks/2026-06-01-control-core-command-convergence-v1.md`
- `/Users/yoyi/workspace/product-line/docs/workbench-system-architecture-v1.md`
- `/Users/yoyi/workspace/product-line/docs/plans/2026-06-01-workbench-architecture-implementation-plan-v1.md`
- `/Users/yoyi/workspace/product-line/decisions/2026-06-01-architecture-module-split-guardrail-v1.md`
- `/Users/yoyi/workspace/product-line/decisions/2026-06-01-project-workflow-canvas-authority-v1.md`
- `/Users/yoyi/workspace/product-line/evidence/2026-06-01-control-core-command-convergence-v1.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-06-01-control-core-command-convergence-v1-result.md`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri/src/types.rs`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri/src/control_core.rs`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri/src/codex_db.rs`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/lib/types.ts`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/tests/`

允许只读真实 workflow state 的必要摘要：

- `/Users/yoyi/Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json`

只读真实 workflow state 时只能读取完成本任务所需的结构和字段摘要，不要复制完整内容进 evidence。

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
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri/src/control_core.rs`
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri/src/*.rs`，仅限新增小模块，例如：
  - `workflow_state_store.rs`
  - `workflow_audit.rs`
  - `workflow_read_model.rs`
  - 实际命名可按代码风格调整
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src/lib/types.ts`，仅当类型确实需要同步
- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/tests/`

允许写构建产物：

- `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/dist/`

允许写记录：

- `/Users/yoyi/workspace/product-line/evidence/2026-06-01-control-core-second-slice-boundary-extraction-v1.md`
- `/Users/yoyi/workspace/product-line/handoffs/2026-06-01-control-core-second-slice-boundary-extraction-v1-result.md`

允许更新当前入口：

- `/Users/yoyi/workspace/product-line/CURRENT.md`
- `/Users/yoyi/workspace/product-line/tasks/README.md`

如确实需要更新：

- `/Users/yoyi/workspace/product-line/AUTHORITY.md`
- `/Users/yoyi/workspace/product-line/README.md`
- `/Users/yoyi/workspace/product-line/docs/plans/2026-06-01-workbench-architecture-implementation-plan-v1.md`

但不要把普通执行结果写成新的架构规则。

## 禁止写入

- 禁止写 `/Users/yoyi/.codex`。
- 禁止写真实业务项目目录。
- 禁止手工改真实 workflow state JSON。
- 禁止迁移 SQLite。
- 禁止新增 Obsidian vault、向量库、图数据库文件。
- 禁止写 auth、token、`.env`、密钥、授权文件。
- 禁止批量格式化 `src/mcp/**`。
- 禁止为了通过 `cargo fmt --check` 大范围格式化既有无关文件。

## 建议实施步骤

### 第 1 步：只读分层表

先在 evidence 里列出 `lib.rs` 内这些函数或函数族的目标归属：

- 状态文件读写。
- 状态 mutation。
- 审计事件构造。
- 读模型派生。
- Tauri 命令包装。
- 控制核心校验。
- 适配器执行。
- 工作流机器。
- 测试。

必须标清：

- 本轮准备移动哪些。
- 本轮不移动哪些。
- 不移动的原因。

### 第 2 步：状态文件读写边界

建议建立小模块，例如 `workflow_state_store.rs`。

优先迁移或包装：

- `read_workflow_state_value`
- `write_validated_workflow_state`
- `backup_workflow_state_file`
- `validate_workflow_state`

要求：

- 不改变 JSON 格式。
- 不改变备份策略。
- 不改变错误文案，除非测试需要同步且 evidence 说明原因。
- 如果迁移导致私有依赖过多，可以只新增包装 helper，并让至少一条状态写入路径使用它。

### 第 3 步：审计事件构造边界

建议建立小模块，例如 `workflow_audit.rs`。

优先迁移或包装一到三类审计：

- `work_item_state_changed`
- `workflow_permission_decision_recorded`
- `workflow_node_dispatch_prepared`
- 或其他执行者认为依赖最少的审计类型

要求：

- 审计字段保持兼容。
- 审计事件 id / timestamp / actor / before / after 字段不丢。
- 不把审计写入变成前端行为。
- 不新增事件账本迁移。

### 第 4 步：读模型派生边界

建议建立小模块，例如 `workflow_read_model.rs`。

优先迁移或包装：

- `project_workflow_summaries`
- `project_blackboards_from_workflows`
- `derive_workflow_read_model`
- 或先建立 `derive_project_workflow_read_models` 这种外层包装。

要求：

- 读模型仍然从事实层派生，不写事实。
- 缺字段继续返回 warning 或空集合，不补编事实。
- 前端类型不应因为本轮拆分出现行为变化。
- 如果完整迁移依赖太多，可以只迁移包装入口和一小段纯函数。

### 第 5 步：最小接入

至少完成：

- 一个状态写入路径调用新状态边界 helper。
- 一个审计事件通过新审计 helper 构造。
- 一个读模型入口通过新读模型 helper 派生。

不要为了完成数量做大面积移动。

### 第 6 步：测试和记录

必须新增或更新测试，证明：

- 状态读写边界不改变现有行为。
- 审计事件字段没有丢。
- 读模型派生结果保持一致。
- 控制核心上一轮已有测试仍通过。

## 验收标准

必须满足：

- 至少新增一个后端边界模块，最好是状态、审计、读模型中的一类。
- `lib.rs` 的职责至少减少一处真实调用，而不是只新增空模块。
- 不改变 workflow state JSON 结构。
- 不改变真实业务行为。
- 不执行真实 Codex。
- 测试通过。
- evidence 说明本轮移动了什么、没移动什么、为什么。
- handoff 明确下一步和仍然存在的风险。

不接受为：

- 最终架构拆分完成。
- 控制核心最终版。
- 事件账本完整迁移。
- 读模型体系最终完成。
- 黑板持久写入完成。
- 正式记忆完成。
- 秘书能力完成。
- 真实业务自动编排完成。

## 必跑验证

在 `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell` 下运行：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- `cargo test --lib`

如果新增 Rust 文件：

- 对新增文件运行 `rustfmt --check`。

可以尝试：

- `cargo fmt --check`

但不要为了通过它去批量格式化既有 `src/lib.rs` 或 `src/mcp/**`。如果 `cargo fmt --check` 因历史格式失败，必须在 handoff 里说明。

## 必须输出

新增 evidence：

- `/Users/yoyi/workspace/product-line/evidence/2026-06-01-control-core-second-slice-boundary-extraction-v1.md`

新增 handoff：

- `/Users/yoyi/workspace/product-line/handoffs/2026-06-01-control-core-second-slice-boundary-extraction-v1-result.md`

evidence 必须包含：

- 读过哪些权威文档。
- `lib.rs` 职责分层表。
- 新增了哪些模块。
- 哪些函数移动或改为调用新 helper。
- 哪些函数保留在 `lib.rs`，原因是什么。
- 状态写入、审计构造、读模型派生分别完成到什么程度。
- 测试命令和结果。
- 明确没有做哪些禁止事项。

handoff 必须包含：

- 本轮完成了什么。
- 不接受为什么。
- 改了哪些文件。
- 测试结果。
- 仍然存在的架构风险。
- 下一步建议。
- 是否需要用户验收。

## 必须停下来让用户验收

以下情况必须停下来，不要继续开发：

- 任务包写完后，交给用户确认范围。
- 执行完成后，交给用户验收 handoff 和测试结果。
- 需要改变 workflow state JSON 结构。
- 需要迁移数据库。
- 需要黑板持久写入。
- 需要正式记忆。
- 需要秘书自动改事实。
- 需要真实 `codex exec` 或 `codex exec resume`。
- 需要读取或写入 `/Users/yoyi/.codex`。
- 需要接 Obsidian、向量库或图数据库。
- 需要启动 MCP canvas run。

## 停止条件

出现以下情况必须停下来问用户，不能自行继续：

- 移动函数导致需要大范围重写测试。
- 状态写入 helper 需要改变 JSON 字段。
- 审计 helper 需要新增事件账本 schema。
- 读模型拆分需要跨模块循环依赖且无法小范围解决。
- 需要写真实业务项目目录。
- 需要读取 auth、token、`.env`、密钥、授权文件。
- 需要读取完整 transcript。
- 需要改变首页内容。
