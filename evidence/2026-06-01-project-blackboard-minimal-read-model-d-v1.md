# Project Blackboard Minimal Read Model Task D v1

日期：2026-06-01

## 本轮目标

执行 Task D：项目黑板最小实现。

本轮完成的是最小模型和读模型，不是写入命令。

原因：

- 架构计划 Task D 提到“新增项目黑板读模型和最小写入命令”。
- 本轮用户明确禁止“不改 workflow state JSON 结构”“不让黑板直接推进工作流状态”“不让黑板直接写正式记忆”。
- 当前没有新的迁移计划或控制核心确认命令设计，所以本轮不新增黑板写入命令，只从现有 workflow state 派生只读黑板。

## 禁止项执行情况

| 禁止项 | 本轮结果 |
|---|---|
| 不执行真实 Codex | 已遵守，未运行 `codex exec` / `codex exec resume`。 |
| 不改 workflow state JSON 结构 | 已遵守，只新增 `WorkflowStateSnapshot.project_blackboards` 读模型字段，不写入状态文件。 |
| 不让黑板直接推进工作流状态 | 已遵守，黑板没有按钮和 mutation 命令。 |
| 不让黑板直接写正式记忆 | 已遵守，记忆相关条目只有 `memory_candidate`，升级状态为 `candidate_pending_control_core`。 |
| 不把知识库引用直接当记忆 | 已遵守，`knowledge_ref` 和 `memory_candidate` 是不同 kind。 |
| 不读 `/Users/yoyi/.codex`、auth、token、`.env`、完整 transcript | 已遵守。 |
| 不迁移数据库 | 已遵守。 |

## 模型

新增 Rust / TypeScript 对应模型：

| 模型 | 位置 | 用途 |
|---|---|---|
| `ProjectBlackboard` | `src-tauri/src/types.rs`、`src/lib/types.ts` | 项目黑板读模型，挂在 `WorkflowStateSnapshot.project_blackboards`。 |
| `BlackboardEntry` | 同上 | 黑板条目，统一承载汇报、风险、权限、工具摘要、记忆候选、知识引用。 |
| `BlackboardEntryKind` | 同上 | `subagent_report`、`risk`、`permission_request`、`tool_summary`、`memory_candidate`、`knowledge_ref`。 |
| `BlackboardSourceRef` | 同上 | 指向来源对象，只做引用，不展开全文。 |
| `BlackboardPromotionDecision` | 同上 | 记录候选升级状态；本轮默认 `candidate_pending_control_core`。 |

## 派生来源

| 黑板 kind | 来源 | 升级目标 | 本轮边界 |
|---|---|---|---|
| `subagent_report` | `derived_workflow.subagent_reports` | `workflow_fact` | 汇报不等于节点完成。 |
| `risk` | `subagent_reports.direction_risks` | `workflow_risk` | 风险不直接推进 workflow state。 |
| `permission_request` | `ProjectWorkflowSummary.permission_requests` | `permission_decision` | 黑板不能批准/拒绝权限。 |
| `tool_summary` | `derived_workflow.ledger_entries` 中 `entry_type=tool_call_summary` | `audit_event` | 只保留摘要和引用，不保留工具全文。 |
| `memory_candidate` | `TaskPackage.available_memory_refs` | `formal_memory` | 只是候选，不写正式记忆。 |
| `knowledge_ref` | `TaskPackage.available_knowledge_refs` | `knowledge_reference` | 只是资料来源，不当作记忆。 |

## 代码改动

| 文件 | 改动 |
|---|---|
| `prototypes/productized-desktop-shell/src-tauri/src/types.rs` | 新增 ProjectBlackboard / BlackboardEntry / BlackboardEntryKind / BlackboardSourceRef / BlackboardPromotionDecision；`WorkflowStateSnapshot` 新增 `project_blackboards`。 |
| `prototypes/productized-desktop-shell/src-tauri/src/lib.rs` | `read_workflow_state_snapshot` 派生 `project_blackboards`；新增黑板派生函数；新增 Rust 回归测试。 |
| `prototypes/productized-desktop-shell/src/lib/types.ts` | 新增前端黑板类型；`WorkflowStateSnapshot` 接受 `project_blackboards`。 |
| `prototypes/productized-desktop-shell/src/views/ProjectsView.tsx` | 项目工作流页新增只读“项目黑板”面板，展示候选条目和升级状态。 |
| `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx` | 离线 UI fixture 新增黑板条目，断言六类候选和 `candidate_pending_control_core`。 |
| `CURRENT.md` | 更新当前状态，标记 Task D 只读切片已完成，并把下一步收敛到 Task E 或 D-followup 控制核心确认边界。 |

## 验证

通过：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `npm run build`
- `cargo test --lib project_blackboard_read_model_derives_candidates_without_state_promotion`
- `cargo test --lib`

结果摘要：

- 前端类型检查通过。
- 离线交互测试通过：`offline interaction tests passed: 2`。
- 前端构建通过；Vite 仍提示 JS chunk 超过 500 kB。
- Rust 全量 lib 测试通过：82 passed、1 ignored。
- Rust 测试仍有既有 warning：`JsonRpcError::invalid_params` 未使用。

额外检查：

- `cargo fmt --check` 失败。
- 失败输出覆盖大量既有 `src/lib.rs` 和 `src/mcp/**` 格式差异；本轮没有全仓库格式化，避免把无关格式 churn 混入 Task D。

## 结论

Task D 的最小只读切片已完成。

当前黑板是读模型，不是事实层：

- 不写 workflow state JSON。
- 不生成正式记忆。
- 不批准权限请求。
- 不推进 workflow / node / work item 状态。
- 所有条目的升级状态默认是 `candidate_pending_control_core`。

剩余风险：

- 计划中“最小写入命令”还没有实现；需要另开控制核心确认命令设计，不能直接补一个写 JSON 的黑板接口。
- `memory_candidate` 当前从任务包的显式 memory refs 派生，只能表示候选展示，不代表正式记忆系统已落地。
- 项目黑板已进入项目页，但 `ProjectsView.tsx` 仍承载较多内部面板，后续仍需要继续收敛 UI 复杂度。
