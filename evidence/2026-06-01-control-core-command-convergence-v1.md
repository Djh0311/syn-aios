# Control Core Command Convergence v1 Evidence

日期：2026-06-01

## 本轮目标

完成 `tasks/2026-06-01-control-core-command-convergence-v1.md` 的保守切片。

这轮做的是控制边界收敛，不是最终控制核心：

- 会改变工作流事实的关键命令先过后端控制核心 helper。
- 权限确认从前端 pending action 接入后端命令。
- 黑板候选只建立确认 / 拒绝 / 待处理边界，不写正式事实、不写正式记忆。
- 不改变 workflow state JSON 结构。

## 读过的权威和依据

| 文件 | 本轮使用方式 |
|---|---|
| `CURRENT.md` | 确认当前主线、Task A-D 已完成、下一步是控制核心命令收敛。 |
| `tasks/2026-06-01-control-core-command-convergence-v1.md` | 作为本轮执行任务包和禁止边界。 |
| `decisions/2026-06-01-architecture-module-split-guardrail-v1.md` | 确认不能碰真实 Codex、workflow state JSON 结构、MCP canvas run、任务包产品规则。 |
| `evidence/2026-06-01-project-blackboard-minimal-read-model-d-v1.md` | 确认黑板当前是读模型，不能直接补写 JSON 接口。 |
| `tasks/README.md` | 确认当前任务队列和已完成任务上下文。 |

## 命令和动作梳理

| 入口 | 文件 | 是否改变事实 | 既有后端校验 | 既有审计 | 目标归属 | 本轮处理 |
|---|---|---:|---|---|---|---|
| `update_work_item_state` | `src-tauri/src/commands.rs` / `src-tauri/src/lib.rs` | 是 | 有工作流、work item、状态表校验 | `work_item_state_changed` | 控制核心 | 接入 `control_core::validate_work_item_state_transition`。 |
| `prepare_workflow_node_dispatch` | `src-tauri/src/commands.rs` / `src-tauri/src/lib.rs` | 是 | 有项目、workflow、node、binding、prompt 校验；此前未阻止草稿态 prepared | `workflow_node_dispatch_prepared` | 控制核心 + 应用服务 | 接入 `validate_dispatch_prepare`，非 `ready_to_dispatch` 直接拒绝。 |
| `execute_workflow_node_dispatch` | `src-tauri/src/commands.rs` / `src-tauri/src/lib.rs` | 是，并会真实 resume | 有 ready 状态、binding、runner 结果校验 | started / completed / failed / readback audit | 控制核心 + 适配器 | 启动前接入 `validate_dispatch_start`；完成/失败写回接入 `validate_dispatch_completion_transition`。本轮未执行真实 Codex。 |
| `record_workflow_dispatch_director_review` | `src-tauri/src/commands.rs` / `src-tauri/src/lib.rs` | 是 | 有 work item ready、dispatch completed、decision 校验 | `workflow_dispatch_director_review_recorded` | 控制核心 | 接入 `validate_director_review_work_item_state` 和 `validate_director_review`。 |
| `record_workflow_permission_decision` | `src-tauri/src/commands.rs` / `src-tauri/src/lib.rs` | 是 | 本轮新增 pending 状态和 decision 校验 | 本轮新增 `workflow_permission_decision_recorded` | 控制核心 | 新增后端命令，更新 `permission_requests[]` 既有字段，不改 JSON 结构。 |
| `prepare_offline_role_dispatch` | `src-tauri/src/commands.rs` / `src-tauri/src/lib.rs` | 是 | 有 ready 状态和重复 prepared 校验 | `offline_role_dispatch_prepared` | 控制核心 | 接入 `validate_dispatch_prepare`。 |
| `record_offline_role_result_handoff` | `src-tauri/src/commands.rs` / `src-tauri/src/lib.rs` | 是 | 有 dispatch prepared 校验 | `offline_role_result_handoff_recorded` | 控制核心 | 接入 `validate_offline_role_handoff`。 |
| `record_offline_director_review` | `src-tauri/src/commands.rs` / `src-tauri/src/lib.rs` | 是 | 有 ready 状态、dispatch completed、decision 校验 | `offline_director_review_recorded` | 控制核心 | 接入 `validate_director_review_work_item_state` 和 `validate_director_review`。 |
| `run_workflow_machine` | `src-tauri/src/commands.rs` / `src-tauri/src/lib.rs` | 是，并会真实 resume | 有启动状态、参数范围、最终状态写回 | `workflow_machine_run_started` / `workflow_machine_run_finished` | 控制核心 + 工作流机器 | 接入 `validate_workflow_machine_start_state` 和 `validate_workflow_machine_final_state`。本轮未执行真实 workflow machine。 |
| `read_workflow_node_dispatch_result` | `src-tauri/src/commands.rs` / `src-tauri/src/lib.rs` | 是，回填统计 | 有 dispatch 归属校验 | `workflow_node_dispatch_readback_completed` | 应用服务 + 读回适配 | 本轮只梳理，未改。 |
| `record-permission-decision` pending action | `src/views/ProjectsView.tsx` / `src/App.tsx` | 触发事实写入 | 此前只进通用 path action | 无 | 纯 UI -> 后端命令 | 本轮改为调用 `recordWorkflowPermissionDecision`。 |
| 项目黑板候选展示 | `src-tauri/src/lib.rs` / `src/views/ProjectsView.tsx` | 否 | 只读派生 | 无写入审计 | 读模型 | 本轮只新增控制核心边界 helper 和测试，不新增写 JSON 命令。 |

## 控制核心 helper

新增文件：`prototypes/productized-desktop-shell/src-tauri/src/control_core.rs`。

已覆盖：

- 工作项状态表：`work_item_transition_allowed`、`validate_work_item_state_transition`。
- 派发准备和启动：`validate_dispatch_prepare`、`validate_dispatch_start`。
- 派发完成/失败：`validate_dispatch_completion_transition`。
- 离线回传：`validate_offline_role_handoff`。
- 总指导回收：`validate_director_review_work_item_state`、`validate_director_review`。
- 权限确认：`validate_permission_decision`。
- 工作流机器：`validate_workflow_machine_start_state`、`validate_workflow_machine_final_state`。
- 黑板候选：`validate_blackboard_candidate_decision`。

关键依据：

- `src-tauri/src/lib.rs` 的工作项状态推进现在调用控制核心校验。
- `prepare_workflow_node_dispatch_at` 现在会拒绝非 `ready_to_dispatch` 的 prepared 写入。
- `record_workflow_permission_decision_at` 新增了 pending 权限确认落账和 audit。
- `App.tsx` 的 `record-permission-decision` 分支现在调用后端命令，不再落到通用 path action。

## 非法路径

本轮已覆盖或保留的拒绝路径：

| 非法路径 | 结果 |
|---|---|
| 工作项 `draft -> running` | 后端拒绝，已有测试覆盖。 |
| 派发准备在非 `ready_to_dispatch` 工作项上写 prepared | 后端拒绝，新增测试覆盖。 |
| 派发完成/失败时 dispatch 不是 `running` | 后端拒绝。 |
| 派发完成/失败时 work item 不能按状态表进入目标状态 | 后端拒绝。 |
| 总指导回收时 work item 不是 `ready_for_review` | 后端拒绝。 |
| 总指导回收时 dispatch 不是 `completed` | 后端拒绝。 |
| 权限请求不是 `pending` 时重复记录结论 | 后端拒绝，新增测试覆盖。 |
| 黑板 `memory_candidate -> formal_memory` 直接确认 | 后端 helper 拒绝，新增测试覆盖。 |
| 黑板 `knowledge_ref -> formal_memory` 直接确认 | 后端 helper 拒绝，新增测试覆盖。 |
| 黑板 `tool_summary -> workflow_state_change` 直接确认 | 后端 helper 拒绝，新增测试覆盖。 |

## 黑板候选边界

本轮没有新增黑板写入命令。

原因：

- Task D 的 evidence 已确认黑板当前是读模型。
- 本任务包禁止改 workflow state JSON 结构。
- 没有“黑板候选持久确认状态”的 schema/迁移计划。

当前规则：

- `mark_pending` / `pending`：控制核心 helper 允许作为“仍待处理”边界。
- `reject_candidate` / `rejected`：控制核心 helper 允许作为“拒绝候选”边界，但本轮不持久写入。
- `confirm_candidate` / `approved`：控制核心 helper 对正式事实、正式记忆、workflow state 推进直接拒绝。
- 权限请求不能通过黑板直接批准，必须走 `record_workflow_permission_decision`。

## 审计处理

| 动作 | 审计情况 |
|---|---|
| 工作项状态推进 | 复用 `work_item_state_changed`。 |
| 派发准备 | 复用 `workflow_node_dispatch_prepared`；本轮开始拒绝非 ready prepared。 |
| 派发启动/完成/失败 | 复用 started/completed/failed/readback audit。 |
| 离线派发/回传/回收 | 复用 `offline_role_dispatch_prepared`、`offline_role_result_handoff_recorded`、`offline_director_review_recorded`。 |
| 权限确认 | 新增 `workflow_permission_decision_recorded`。 |
| 工作流机器收口 | 复用 `workflow_machine_run_started`、`workflow_machine_run_finished`。 |
| 黑板候选确认/拒绝/待处理 | 本轮没有持久命令，因此没有写 audit；缺口是需要后续 schema/命令设计。 |

## 改动文件

| 文件 | 改动 |
|---|---|
| `prototypes/productized-desktop-shell/src-tauri/src/control_core.rs` | 新增控制核心 helper。 |
| `prototypes/productized-desktop-shell/src-tauri/src/lib.rs` | 接入控制核心校验；新增权限确认落账函数；新增 Rust 测试。 |
| `prototypes/productized-desktop-shell/src-tauri/src/types.rs` | 新增 `WorkflowPermissionDecisionRequest` 后端请求类型。 |
| `prototypes/productized-desktop-shell/src-tauri/src/commands.rs` | 新增 `record_workflow_permission_decision` Tauri command 包装。 |
| `prototypes/productized-desktop-shell/src/lib/tauri.ts` | 新增 `recordWorkflowPermissionDecision` 前端 API。 |
| `prototypes/productized-desktop-shell/src/App.tsx` | `record-permission-decision` pending action 改为调用后端命令。 |
| `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx` | 更新权限确认动作边界文案。 |
| `prototypes/productized-desktop-shell/src/components/PermissionDialog.tsx` | 更新权限确认弹层说明。 |
| `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx` | 更新权限确认弹层预期。 |
| `CURRENT.md` | 更新当前任务完成状态和下一步建议。 |
| `tasks/README.md` | 更新任务队列状态。 |

## 验证

通过：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- `rustfmt --check src/control_core.rs`
- `cargo test --lib`

结果摘要：

- 前端类型检查通过。
- 离线交互测试通过：`offline interaction tests passed: 2`。
- 前端构建通过；Vite 仍提示 JS chunk 超过 500 kB，这是既有构建警告。
- 新增 Rust 文件 `src/control_core.rs` 的 `rustfmt --check` 通过。
- Rust 全量 lib 测试通过：85 passed、1 ignored。
- Rust 仍有既有 warning：`JsonRpcError::invalid_params` 未使用。

中间修正：

- 第一次 `cargo test --lib` 有 1 个测试失败，原因是总指导回收拒绝顺序变化导致旧断言找不到旧文案。
- 已补 `validate_director_review_work_item_state`，保留先拒绝非 `ready_for_review` work item 的行为，再次 `cargo test --lib` 通过。

## 禁止事项执行情况

| 禁止项 | 本轮结果 |
|---|---|
| 不执行真实 `codex exec` / `codex exec resume` | 已遵守。 |
| 不读或写 `/Users/yoyi/.codex` | 已遵守。 |
| 不读 auth/token/`.env`/密钥/授权文件 | 已遵守。 |
| 不读完整 transcript 或 rollout JSONL 正文 | 已遵守。 |
| 不启动 MCP canvas run | 已遵守。 |
| 不改 workflow state JSON 结构 | 已遵守，只复用 `permission_requests[]` 既有字段。 |
| 不手工改真实 workflow state JSON | 已遵守，测试只写临时状态文件。 |
| 不写真实业务项目目录 | 已遵守。 |
| 不迁移数据库 | 已遵守。 |
| 不写正式记忆 | 已遵守。 |
| 不把知识引用当记忆 | 已遵守。 |
| 不接 Obsidian/向量库/图数据库 | 已遵守。 |
| 不改首页 | 已遵守。 |
| 不把任务包管理器重新做成产品主界面 | 已遵守。 |

## 仍然存在的风险

- `src-tauri/src/lib.rs` 仍然很大，控制核心 helper 只是第一层规则入口，还没有把状态读写、审计生成、读模型和适配器完全拆开。
- 黑板候选没有持久确认状态；现在只能证明直接升级会被拒绝，不能证明完整审批流已完成。
- 权限确认现在会写 `permission_requests[]`，但没有自动推进 `waiting_for_permission` 工作项状态；这是刻意保守，后续需要单独设计。
- `read_workflow_node_dispatch_result` 仍会回填统计，但还没有纳入控制核心 helper；本轮只梳理为读回适配缺口。
