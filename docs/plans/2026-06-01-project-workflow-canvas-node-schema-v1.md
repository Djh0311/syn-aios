# 项目工作流画布节点 Schema v1

日期：2026-06-02

状态：schema / 计划草案。只定义项目画布读模型，不改 workflow state JSON，不写 UI 实现。

## 结论

先说薄弱点：

- 当前项目页已经有很多工作流信息，但主界面仍平铺内部协议细节，不能直接拿现有组件当最终画布 schema。
- 独立 `CanvasView` 已有 React Flow 节点/边模型，但它属于实验/模板画布，不能直接成为项目 workflow state 的事实源。
- 本 schema 只是读模型契约，不是数据库 schema，也不是迁移计划。

本轮结论：

- 项目画布应定义为从 `WorkflowStateSnapshot` 派生出来的 `ProjectWorkflowCanvasReadModel`。
- 事实源仍是项目 workflow state、项目黑板、控制核心确认后的事件和审计。
- UI 后续可用 React Flow / xyflow 渲染本读模型，但 React Flow 只承载显示和交互，不承载事实。

## 设计边界

接受：

- 项目画布展示项目目标、总指导、开发线、验证线、回收线、权限请求、黑板候选、证据引用、审计引用。
- 节点详情展示任务包、handoff、evidence、review、权限、模型、工具、harness、账本引用和完成闸门。
- 画布主界面只显示节点、边、状态、少量关键徽标和阻塞提示。
- 高风险动作只作为详情面板中的 `allowed_actions` 暴露，并继续走现有确认弹层和控制核心。

不接受：

- 不改 workflow state JSON。
- 不把任务包正文铺成主 UI。
- 不把独立 `CanvasView` / `CanvasDefinition` 当项目工作流事实源。
- 不启动 MCP canvas run。
- 不做通用自动化节点。
- 不把 harness 当普通节点执行器。
- 不让知识引用直接变正式记忆。

## 输入模型

项目画布读模型只从以下已存在对象派生：

| 输入 | 用途 |
|---|---|
| `WorkflowStateSnapshot.project_workflows[]` | 找到当前项目 workflow、节点、工作项、绑定、派发、回收、权限、执行控制。 |
| `ProjectWorkflowSummary.derived_workflow` | 读取派生 workflow、节点、任务包、账本、报告、审查、异常、状态机、验收场景。 |
| `ProjectWorkflowSummary.task_drafts[]` | 工作项状态、当前节点、下一步动作。 |
| `ProjectWorkflowSummary.node_session_bindings[]` | 节点绑定的 Codex 会话摘要和可读状态。 |
| `ProjectWorkflowSummary.node_dispatches[]` | 派发状态、最终摘要、warning。 |
| `ProjectWorkflowSummary.director_reviews[]` | 总指导回收结论。 |
| `ProjectWorkflowSummary.permission_requests[]` | 权限请求节点和阻塞提示。 |
| `ProjectWorkflowSummary.execution_controls[]` | 可控执行协议状态、超时、重试、取消。 |
| `ProjectWorkflowSummary.execution_attempts[]` | 失败、重试、超时、取消记录摘要。 |
| `WorkflowStateSnapshot.project_blackboards[]` | 黑板候选、风险、工具摘要、记忆候选、知识引用。 |

不从以下对象直接派生项目事实：

- 独立 `CanvasDefinition`
- 独立 `CanvasRunState`
- MCP canvas / run / audit 文件层
- Codex transcript 全文
- auth、token、`.env`、密钥

## 顶层读模型

```ts
type ProjectWorkflowCanvasReadModel = {
  schema_version: "project_workflow_canvas.v1";
  project_id: string;
  project_root: string;
  workflow_id: string;
  title: string;
  status: ProjectCanvasStatus;
  source: ProjectCanvasSource;
  viewport_hint: ProjectCanvasViewportHint;
  nodes: ProjectCanvasNode[];
  edges: ProjectCanvasEdge[];
  detail_panels: Record<string, ProjectCanvasNodeDetail>;
  global_badges: ProjectCanvasBadge[];
  warnings: string[];
};
```

字段说明：

| 字段 | 说明 |
|---|---|
| `schema_version` | 固定为 `project_workflow_canvas.v1`。 |
| `project_id` / `project_root` / `workflow_id` | 项目和 workflow 身份。 |
| `title` | workflow 标题。 |
| `status` | 画布级状态，由 workflow、工作项、异常和完成闸门派生。 |
| `source` | 说明事实源和派生时间。 |
| `viewport_hint` | UI 布局建议，不是事实。 |
| `nodes` | React Flow 可渲染的节点读模型。 |
| `edges` | React Flow 可渲染的边读模型。 |
| `detail_panels` | 节点详情数据，key 为 `node_id`。 |
| `global_badges` | 画布顶部状态徽标，例如 blocked、warning、ready_for_review。 |
| `warnings` | 派生 warning，不补编事实。 |

## 画布状态

```ts
type ProjectCanvasStatus =
  | "idle"
  | "ready_to_dispatch"
  | "running"
  | "waiting_for_permission"
  | "ready_for_review"
  | "needs_changes"
  | "accepted"
  | "failed"
  | "timed_out"
  | "blocked"
  | "unknown";
```

派生规则：

| 条件 | 状态 |
|---|---|
| 当前 work item 为 `running` | `running` |
| 存在 pending 权限请求 | `waiting_for_permission` |
| 当前 work item 为 `ready_to_dispatch` | `ready_to_dispatch` |
| 当前 work item 为 `ready_for_review` | `ready_for_review` |
| 当前 work item 为 `accepted` | `accepted` |
| 当前 work item 为 `failed` | `failed` |
| 当前 work item 为 `timed_out` | `timed_out` |
| 运行前检查为 `blocked` | `blocked` |
| 无工作项但有 workflow | `idle` |
| 无法判断 | `unknown` |

## 事实源说明

```ts
type ProjectCanvasSource = {
  source_kind: "workflow_state_read_model";
  workflow_state_path?: string | null;
  workflow_state_updated_at?: string | null;
  derived_from: ProjectCanvasSourceRef[];
  generated_at?: string | null;
};

type ProjectCanvasSourceRef = {
  kind:
    | "workflow"
    | "workflow_node"
    | "work_item"
    | "task_package"
    | "node_binding"
    | "dispatch"
    | "director_review"
    | "permission_request"
    | "execution_control"
    | "execution_attempt"
    | "ledger_entry"
    | "blackboard_entry"
    | "audit_event"
    | "evidence_ref"
    | "handoff_ref";
  id: string;
  label?: string | null;
};
```

规则：

- `source_refs` 只引用 ID 或路径摘要，不带 transcript 正文。
- `evidence_ref` / `handoff_ref` 可引用文件路径，但不在主画布显示全文。
- `audit_event` 只显示摘要和引用。

## 节点 Schema

```ts
type ProjectCanvasNode = {
  node_id: string;
  node_type: ProjectCanvasNodeType;
  title: string;
  subtitle?: string | null;
  status: ProjectCanvasStatus;
  role_id?: string | null;
  work_item_id?: string | null;
  workflow_node_id?: string | null;
  position_hint?: ProjectCanvasPositionHint | null;
  badges: ProjectCanvasBadge[];
  metrics: ProjectCanvasNodeMetric[];
  source_refs: ProjectCanvasSourceRef[];
  detail_panel_id: string;
  warnings: string[];
};

type ProjectCanvasNodeType =
  | "project_goal"
  | "director"
  | "dev_line"
  | "validation_line"
  | "review_line"
  | "permission_request"
  | "blackboard_candidate"
  | "evidence_ref"
  | "audit_ref";
```

节点类型说明：

| 节点类型 | 用途 | 默认来源 |
|---|---|---|
| `project_goal` | 当前项目目标和 workflow 总目标。 | workflow 标题、当前工作项标题、任务包目标。 |
| `director` | 总指导 / 项目主管。 | `WorkflowNode` assigned_role/director，或 workflow 默认 director 节点。 |
| `dev_line` | 开发线。 | `assigned_role_id=codex-dev`、node binding、dispatch。 |
| `validation_line` | 验证线。 | `assigned_role_id=validation`、review / harness / run check。 |
| `review_line` | 回收线 / 审查线。 | director review、review results、ready_for_review。 |
| `permission_request` | 权限请求。 | pending / decided permission requests。 |
| `blackboard_candidate` | 黑板候选、风险、工具摘要、记忆候选、知识引用。 | project blackboard entries。 |
| `evidence_ref` | evidence 引用。 | task package / report / review / ledger evidence refs。 |
| `audit_ref` | audit 引用。 | recent audit events / ledger audit refs。 |

不设为默认主节点：

- `tool_call`
- `knowledge_read`
- `memory_write`
- `harness`
- `codex_transcript`
- `mcp_canvas_run`

这些内容只能进入详情面板、徽标、引用或后置能力。

## 节点位置建议

```ts
type ProjectCanvasPositionHint = {
  lane:
    | "goal"
    | "director"
    | "execution"
    | "validation"
    | "review"
    | "sidecar";
  order: number;
  x?: number | null;
  y?: number | null;
};
```

第一版布局建议：

| lane | 节点 |
|---|---|
| `goal` | `project_goal` |
| `director` | `director` |
| `execution` | `dev_line` |
| `validation` | `validation_line` |
| `review` | `review_line` |
| `sidecar` | `permission_request`、`blackboard_candidate`、`evidence_ref`、`audit_ref` |

说明：

- 位置只是 UI hint，不写回事实。
- React Flow 实现时可以用这个 hint 初始化节点位置。
- 用户拖动项目画布是否保存为个人布局，属于后置问题；本 schema 不定义写入。

## 节点徽标与指标

```ts
type ProjectCanvasBadge = {
  badge_id: string;
  label: string;
  tone: "neutral" | "ready" | "running" | "warning" | "blocked" | "accepted" | "failed";
  source_refs: ProjectCanvasSourceRef[];
};

type ProjectCanvasNodeMetric = {
  metric_id: string;
  label: string;
  value: string | number | boolean;
  tone?: ProjectCanvasBadge["tone"];
};
```

常用徽标：

| badge | 来源 |
|---|---|
| `missing_fields` | task package missing fields。 |
| `blocked` | run check blocked reasons / pending permission。 |
| `warning` | workflow / node / blackboard warnings。 |
| `ready_for_review` | work item state。 |
| `accepted` | director review decision 或 work item state。 |
| `rollout_missing` | node session binding rollout_exists=false。 |

## 边 Schema

```ts
type ProjectCanvasEdge = {
  edge_id: string;
  edge_type: ProjectCanvasEdgeType;
  source_node_id: string;
  target_node_id: string;
  status: ProjectCanvasEdgeStatus;
  label?: string | null;
  source_refs: ProjectCanvasSourceRef[];
  warnings: string[];
};

type ProjectCanvasEdgeType =
  | "responsibility_flow"
  | "handoff_flow"
  | "review_flow"
  | "evidence_reference"
  | "blocking_relation";

type ProjectCanvasEdgeStatus =
  | "idle"
  | "active"
  | "blocked"
  | "completed"
  | "warning"
  | "unknown";
```

边类型说明：

| 边类型 | 用途 |
|---|---|
| `responsibility_flow` | 目标 -> 总指导 -> 开发线 -> 验证线 -> 回收线。 |
| `handoff_flow` | 开发线/验证线/回收线之间的 handoff。 |
| `review_flow` | review result 和 director review 关系。 |
| `evidence_reference` | 任务、报告、审查、账本到 evidence 节点的引用。 |
| `blocking_relation` | 权限请求、风险、缺字段、运行前检查对主线节点的阻塞。 |

## 节点详情面板

```ts
type ProjectCanvasNodeDetail = {
  detail_panel_id: string;
  node_id: string;
  title: string;
  summary?: string | null;
  sections: ProjectCanvasDetailSection[];
  allowed_actions: ProjectCanvasAction[];
  source_refs: ProjectCanvasSourceRef[];
  warnings: string[];
};

type ProjectCanvasDetailSection = {
  section_id: string;
  title: string;
  kind:
    | "summary"
    | "task_package"
    | "session_binding"
    | "dispatch"
    | "handoff_refs"
    | "evidence_refs"
    | "audit_refs"
    | "permission_requests"
    | "blackboard_entries"
    | "run_check"
    | "completion_gate"
    | "execution_attempts"
    | "review_results";
  items: ProjectCanvasDetailItem[];
};

type ProjectCanvasDetailItem = {
  item_id: string;
  label: string;
  value: string;
  value_kind?: "text" | "status" | "path" | "ref" | "count" | "warning" | "blocked";
  source_refs: ProjectCanvasSourceRef[];
};
```

详情面板原则：

- 主画布显示摘要，详情面板显示字段。
- 任务包只作为详情 section，不作为主 UI。
- evidence / handoff / audit 只显示引用和摘要，不铺全文。
- permission request 的批准/拒绝必须走 `allowed_actions` 和确认弹层，不在主画布直接执行。

## 允许动作

```ts
type ProjectCanvasAction = {
  action_id: string;
  label: string;
  action_kind:
    | "open_agent_session"
    | "bind_node_session"
    | "unbind_node_session"
    | "inspect_run_check"
    | "advance_work_item_state"
    | "execute_safe_probe"
    | "execute_user_reviewed_instruction"
    | "record_director_review"
    | "record_permission_decision"
    | "open_evidence"
    | "open_handoff";
  enabled: boolean;
  disabled_reason?: string | null;
  requires_confirmation: boolean;
  boundary: string;
  source_refs: ProjectCanvasSourceRef[];
};
```

动作规则：

| 动作 | 默认位置 | 规则 |
|---|---|---|
| `open_agent_session` | 节点详情 | 可直接打开，只读查看。 |
| `bind_node_session` / `unbind_node_session` | 节点详情 | 需要确认；只写工作台 workflow state。 |
| `inspect_run_check` | 节点详情 | 只读检查，可执行。 |
| `advance_work_item_state` | 节点详情 | 需要确认；走控制核心。 |
| `execute_safe_probe` | 节点详情 | 高风险；必须确认，会写 `/Users/yoyi/.codex`，不在本轮实现。 |
| `execute_user_reviewed_instruction` | 节点详情 | 高风险；必须确认，会写业务路径，不在本轮实现。 |
| `record_director_review` | 节点详情 | 需要确认；写 review 和 audit。 |
| `record_permission_decision` | 权限节点详情 | 需要确认；走控制核心。 |
| `open_evidence` / `open_handoff` | 详情引用 | 只打开文件，不写事实。 |

本 schema 不新增任何 action 实现，只描述 UI 合约。

## 派生规则

### 基础流程节点

1. 从 `ProjectWorkflowSummary.derived_workflow.nodes` 读取 workflow 节点。
2. 若缺少四角色节点，可从当前 workflow / work item / binding 派生展示占位，但必须加 warning：`derived_placeholder_no_source_node`。
3. `project_goal` 从 workflow title、当前 work item title、task package goal 派生。
4. `director`、`dev_line`、`validation_line`、`review_line` 优先匹配 `workflow_node_id` 或 `assigned_role_id`。

### 状态派生

1. 节点关联当前 work item 时，优先使用 work item state。
2. 存在 running dispatch 时，节点为 `running`。
3. 存在 pending permission request 指向该 work item / dispatch 时，节点为 `waiting_for_permission`。
4. 存在 failed / timed_out execution attempt 时，节点为 `failed` / `timed_out`。
5. review result 不直接让节点 completed；仍需 director review 或 work item accepted。

### 权限请求节点

1. 每个 pending permission request 派生一个 `permission_request` sidecar 节点。
2. 已决定的 permission request 默认不显示为主节点，只进入相关节点详情。
3. 权限节点用 `blocking_relation` 指向被阻塞的主线节点。

### 黑板候选节点

1. 高风险或 pending 的 blackboard entry 可派生为 `blackboard_candidate` sidecar 节点。
2. 已拒绝或已确认的 blackboard entry 默认进入详情历史，不作为主节点。
3. `knowledge_ref` 和 `memory_candidate` 必须标注“候选/引用”，不能显示成正式记忆。

### 证据和审计节点

1. evidence / audit 默认进入详情 section。
2. 只有当 evidence 缺失、审计阻塞或用户选中“显示引用节点”时，才派生 sidecar 节点。
3. 不把 evidence 正文、audit 全文或 transcript 全文铺到画布。

## React Flow 映射

本 schema 到 React Flow 的推荐映射：

| 本 schema | React Flow |
|---|---|
| `ProjectCanvasNode.node_id` | `Node.id` |
| `ProjectCanvasNode.position_hint` | `Node.position` 的初始布局输入 |
| `ProjectCanvasNode` | `Node.data.canvasNode` |
| `ProjectCanvasEdge.edge_id` | `Edge.id` |
| `ProjectCanvasEdge.source_node_id` | `Edge.source` |
| `ProjectCanvasEdge.target_node_id` | `Edge.target` |
| `ProjectCanvasEdge` | `Edge.data.canvasEdge` |

规则：

- React Flow 不持久化项目事实。
- 节点拖拽布局是否保存另开设计。
- React Flow 的 custom node 只读 `ProjectCanvasNode`，不直接读 workflow state。

## 验收场景

| 场景 | 期望 |
|---|---|
| 空 workflow | 显示 `project_goal` 和占位 director，状态 `idle`，带 missing workflow/work item warning。 |
| 四角色 ready | 显示目标、总指导、开发线、验证线、回收线，责任流转边完整。 |
| ready_to_dispatch | 开发线或 assigned role 节点为 `ready_to_dispatch`，详情可显示绑定和派发动作。 |
| running | 当前派发节点为 `running`，边为 active。 |
| waiting_for_permission | 权限 sidecar 节点阻塞主线节点。 |
| ready_for_review | 回收线或 director 节点显示待回收。 |
| accepted | 主线显示 accepted，详情显示 director review 摘要和 evidence refs。 |
| failed / timed_out | 主线节点显示失败或超时，详情显示 execution attempt 摘要。 |
| blackboard candidate | 黑板候选只显示中间态/候选，不升级为正式事实。 |

## 后续任务建议

下一步 `final-skeleton-07-canvas-component-state-examples-v1` 应建立组件状态样例：

- 空画布。
- 四角色节点。
- 执行中。
- 等待权限。
- 失败。
- 回收中。
- accepted。
- 右侧详情打开。
- 黑板候选 sidecar。

Skeleton-07 仍不应：

- 改 workflow state JSON。
- 启动 MCP canvas run。
- 执行真实 Codex。
- 接通用自动化节点。
- 实现完整 React Flow 项目画布。

## 是否需要迁移

不需要。

原因：

- 本 schema 只定义从现有 `WorkflowStateSnapshot`、`ProjectWorkflowSummary.derived_workflow` 和 `ProjectBlackboard` 派生出来的 UI 读模型。
- 不新增、删除或改名 workflow state JSON 字段。
- 不要求独立 `CanvasDefinition` 和项目 workflow state 合一。

如果未来要保存用户布局、节点折叠状态或模板，必须另开 schema / 迁移计划。
