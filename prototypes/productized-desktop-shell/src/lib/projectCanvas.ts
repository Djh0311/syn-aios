import type {
  BlackboardEntry,
  ProjectBlackboard,
  ProjectRecord,
  ProjectWorkflowSummary,
  RuntimeSessionAttention,
  TaskDraftSummary,
  TaskPackage,
  Workflow,
  WorkflowDispatchDirectorReviewRecord,
  WorkflowExecutionAttemptRecord,
  WorkflowNode,
  WorkflowNodeDispatchRecord,
  WorkflowNodeSessionBinding,
  WorkflowPermissionRequestRecord,
} from "./types";

export type ProjectCanvasStatus =
  | "empty"
  | "idle"
  | "prepared"
  | "ready_to_dispatch"
  | "running"
  | "waiting_for_permission"
  | "needs_review"
  | "ready_for_review"
  | "needs_changes"
  | "accepted"
  | "failed"
  | "timed_out"
  | "readback_unavailable"
  | "blocked"
  | "unknown";

export type ProjectCanvasNodeType =
  | "project_goal"
  | "director"
  | "dev_line"
  | "validation_line"
  | "review_line"
  | "permission_request"
  | "blackboard_candidate"
  | "evidence_ref"
  | "audit_ref";

export type ProjectCanvasBadgeTone = "neutral" | "ready" | "running" | "warning" | "blocked" | "accepted" | "failed";

export type ProjectCanvasSourceRef = {
  kind:
    | "workflow"
    | "workflow_node"
    | "project"
    | "project_blackboard"
    | "work_item"
    | "task_package"
    | "memory_packet"
    | "node_binding"
    | "dispatch"
    | "director_review"
    | "permission_request"
    | "execution_control"
    | "execution_attempt"
    | "ledger_entry"
    | "blackboard_entry"
    | "audit_event"
    | "authorization"
    | "authorization_check"
    | "proposal"
    | "readback"
    | "runtime_attention"
    | "run_check"
    | "evidence_ref"
    | "handoff_ref";
  id: string;
  label?: string | null;
};

export type ProjectCanvasBadge = {
  badge_id: string;
  label: string;
  tone: ProjectCanvasBadgeTone;
  source_refs: ProjectCanvasSourceRef[];
};

export type ProjectCanvasNodeMetric = {
  metric_id: string;
  label: string;
  value: string | number | boolean;
  tone?: ProjectCanvasBadgeTone;
};

export type ProjectCanvasPositionHint = {
  lane: "goal" | "director" | "execution" | "validation" | "review" | "sidecar";
  order: number;
  x?: number | null;
  y?: number | null;
};

export type ProjectCanvasNode = {
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

export type ProjectCanvasEdgeType =
  | "responsibility_flow"
  | "handoff_flow"
  | "review_flow"
  | "evidence_reference"
  | "blocking_relation";

export type ProjectCanvasEdgeStatus = "idle" | "active" | "blocked" | "completed" | "warning" | "unknown";

export type ProjectCanvasEdge = {
  edge_id: string;
  edge_type: ProjectCanvasEdgeType;
  source_node_id: string;
  target_node_id: string;
  status: ProjectCanvasEdgeStatus;
  label?: string | null;
  source_refs: ProjectCanvasSourceRef[];
  warnings: string[];
};

export type ProjectCanvasDetailSectionKind =
  | "summary"
  | "attention"
  | "task_package"
  | "memory_packet"
  | "session_binding"
  | "dispatch"
  | "readback"
  | "handoff_refs"
  | "evidence_refs"
  | "audit_refs"
  | "permission_requests"
  | "blackboard_entries"
  | "run_check"
  | "completion_gate"
  | "execution_attempts"
  | "review_results"
  | "source_refs";

export type ProjectCanvasDetailLayer = "user_summary" | "project_director" | "technical_details";

export type ProjectCanvasDetailItem = {
  item_id: string;
  label: string;
  value: string;
  value_kind?: "text" | "status" | "path" | "ref" | "count" | "warning" | "blocked";
  source_refs: ProjectCanvasSourceRef[];
};

export type ProjectCanvasDetailSection = {
  section_id: string;
  title: string;
  kind: ProjectCanvasDetailSectionKind;
  layer: ProjectCanvasDetailLayer;
  default_open: boolean;
  items: ProjectCanvasDetailItem[];
};

export type ProjectCanvasAction = {
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

export type ProjectCanvasNodeDetail = {
  detail_panel_id: string;
  node_id: string;
  title: string;
  summary?: string | null;
  sections: ProjectCanvasDetailSection[];
  allowed_actions: ProjectCanvasAction[];
  source_refs: ProjectCanvasSourceRef[];
  warnings: string[];
};

export type ProjectCanvasAttentionKind =
  | "empty"
  | "blocked"
  | "needs_review"
  | "prepared"
  | "running"
  | "ready_for_review"
  | "failed"
  | "timed_out"
  | "waiting_for_permission"
  | "readback_unavailable"
  | "task_package"
  | "memory_packet"
  | "audit_ref"
  | "evidence_ref"
  | "handoff_ref"
  | "unknown";

export type ProjectCanvasAttentionSeverity = "info" | "warning" | "needs_user" | "blocking";

export type ProjectCanvasAttention = {
  attention_id: string;
  kind: ProjectCanvasAttentionKind;
  severity: ProjectCanvasAttentionSeverity;
  status: ProjectCanvasStatus;
  title: string;
  summary: string;
  requires_user_action: boolean;
  source_refs: ProjectCanvasSourceRef[];
};

export type ProjectCanvasStatusReason = {
  reason_id: string;
  status: ProjectCanvasStatus;
  label: string;
  summary: string;
  source_refs: ProjectCanvasSourceRef[];
};

export type ProjectCanvasEditCapabilityKind =
  | "view_only"
  | "local_layout_preview"
  | "personal_layout_preference"
  | "workflow_node_mutation"
  | "workflow_edge_mutation"
  | "permission_or_model_mutation"
  | "execution_mutation";

export type ProjectCanvasEditCapabilityStatus = "allowed" | "preview_only" | "blocked" | "requires_future_task";

export type ProjectCanvasEditCapability = {
  capability_id: string;
  kind: ProjectCanvasEditCapabilityKind;
  label: string;
  status: ProjectCanvasEditCapabilityStatus;
  summary: string;
  changes_workflow_facts: boolean;
  requires_proposal: boolean;
  requires_confirmation: boolean;
  requires_control_core: boolean;
  requires_audit: boolean;
  source_refs: ProjectCanvasSourceRef[];
};

export type ProjectCanvasLayoutBoundary = {
  layout_kind: "role_lanes_v1";
  scope: "view_only";
  summary: string;
  react_flow_source_of_truth: false;
  writes_workflow_state: false;
  persists_layout: false;
  reset_view_allowed: true;
  source_refs: ProjectCanvasSourceRef[];
  warnings: string[];
};

export type WorkflowEditProposalPreview = {
  preview_id: string;
  change_kind: Extract<
    ProjectCanvasEditCapabilityKind,
    "workflow_node_mutation" | "workflow_edge_mutation" | "permission_or_model_mutation" | "execution_mutation"
  >;
  label: string;
  status: Extract<ProjectCanvasEditCapabilityStatus, "preview_only" | "blocked">;
  summary: string;
  changes_workflow_facts: true;
  requires_proposal: true;
  requires_confirmation: boolean;
  requires_control_core: true;
  requires_audit: true;
  disabled_reason: string;
  source_refs: ProjectCanvasSourceRef[];
};

export type ProjectWorkflowEditBoundary = {
  boundary_id: string;
  source_kind: "frontend_read_model";
  layout_boundary: ProjectCanvasLayoutBoundary;
  capabilities: ProjectCanvasEditCapability[];
  proposal_previews: WorkflowEditProposalPreview[];
  warnings: string[];
};

export type ProjectWorkflowCanvasReadModel = {
  schema_version: "project_workflow_canvas.v1";
  project_id: string;
  project_root: string;
  workflow_id: string;
  title: string;
  status: ProjectCanvasStatus;
  source: {
    source_kind: "workflow_state_read_model";
    workflow_state_path?: string | null;
    workflow_state_updated_at?: string | null;
    derived_from: ProjectCanvasSourceRef[];
    generated_at?: string | null;
  };
  viewport_hint: {
    layout: "role_lanes_v1";
    selected_node_id: string;
  };
  status_reason: ProjectCanvasStatusReason;
  nodes: ProjectCanvasNode[];
  edges: ProjectCanvasEdge[];
  detail_panels: Record<string, ProjectCanvasNodeDetail>;
  global_badges: ProjectCanvasBadge[];
  attention_items: ProjectCanvasAttention[];
  edit_boundary: ProjectWorkflowEditBoundary;
  warnings: string[];
};

export type ProjectCanvasComponentStateExample = {
  example_id: string;
  label: string;
  status: ProjectCanvasStatus;
  node_count: number;
  detail_sections: string[];
  permission_queue: "none" | "pending" | "decided";
  description: string;
};

type RoleLane = {
  role_id: "director" | "codex-dev" | "validation" | "review";
  node_type: Extract<ProjectCanvasNodeType, "director" | "dev_line" | "validation_line" | "review_line">;
  title: string;
  subtitle: string;
  lane: ProjectCanvasPositionHint["lane"];
  x: number;
  y: number;
};

const roleLanes: RoleLane[] = [
  { role_id: "director", node_type: "director", title: "总指导", subtitle: "规划 / 回收", lane: "director", x: 260, y: 150 },
  { role_id: "codex-dev", node_type: "dev_line", title: "开发线", subtitle: "执行 / 回传", lane: "execution", x: 520, y: 150 },
  { role_id: "validation", node_type: "validation_line", title: "验证线", subtitle: "检查 / 证据", lane: "validation", x: 780, y: 150 },
  { role_id: "review", node_type: "review_line", title: "回收线", subtitle: "审查 / 结论", lane: "review", x: 1040, y: 150 },
];

export function projectCanvasStateExamples(): ProjectCanvasComponentStateExample[] {
  return [
    {
      example_id: "empty",
      label: "空画布",
      status: "empty",
      node_count: 2,
      detail_sections: ["summary", "run_check"],
      permission_queue: "none",
      description: "只有项目目标和总指导占位，不补编工作项。",
    },
    {
      example_id: "four_roles",
      label: "四角色",
      status: "ready_to_dispatch",
      node_count: 5,
      detail_sections: ["summary", "task_package", "session_binding"],
      permission_queue: "none",
      description: "目标、总指导、开发线、验证线、回收线责任流转完整。",
    },
    {
      example_id: "prepared",
      label: "准备派发",
      status: "prepared",
      node_count: 5,
      detail_sections: ["summary", "task_package", "memory_packet", "dispatch"],
      permission_queue: "none",
      description: "准备派发只代表准备记录，不代表已有工作者产出。",
    },
    {
      example_id: "running",
      label: "执行中",
      status: "running",
      node_count: 5,
      detail_sections: ["dispatch", "execution_attempts", "audit_refs"],
      permission_queue: "none",
      description: "派发节点和责任边进入 active 态。",
    },
    {
      example_id: "needs_review",
      label: "待复核",
      status: "needs_review",
      node_count: 5,
      detail_sections: ["summary", "run_check", "audit_refs"],
      permission_queue: "none",
      description: "授权、guard 或边界需要复核；不能自动扩大范围。",
    },
    {
      example_id: "waiting_permission",
      label: "等待权限",
      status: "waiting_for_permission",
      node_count: 6,
      detail_sections: ["permission_requests", "blackboard_entries", "run_check"],
      permission_queue: "pending",
      description: "权限 sidecar 和黑板候选阻塞主线节点，结论仍走控制核心。",
    },
    {
      example_id: "blocked",
      label: "阻断",
      status: "blocked",
      node_count: 5,
      detail_sections: ["summary", "run_check", "audit_refs"],
      permission_queue: "none",
      description: "控制核心或运行检查阻断，不启动真实执行。",
    },
    {
      example_id: "failed",
      label: "失败",
      status: "failed",
      node_count: 5,
      detail_sections: ["execution_attempts", "audit_refs"],
      permission_queue: "decided",
      description: "失败和超时只展示摘要，不展开完整会话记录。",
    },
    {
      example_id: "timed_out",
      label: "超时",
      status: "timed_out",
      node_count: 5,
      detail_sections: ["execution_attempts", "readback", "audit_refs"],
      permission_queue: "decided",
      description: "超时只显示摘要和来源，不自动重试。",
    },
    {
      example_id: "readback_unavailable",
      label: "读回不可用",
      status: "readback_unavailable",
      node_count: 5,
      detail_sections: ["readback", "execution_attempts", "audit_refs"],
      permission_queue: "none",
      description: "读回不可用不显示成真实 0 条结果。",
    },
    {
      example_id: "reviewing",
      label: "回收中",
      status: "ready_for_review",
      node_count: 5,
      detail_sections: ["review_results", "evidence_refs", "completion_gate"],
      permission_queue: "none",
      description: "审查通过仍不等于完成，等待总指导回收。",
    },
    {
      example_id: "accepted",
      label: "accepted",
      status: "accepted",
      node_count: 5,
      detail_sections: ["review_results", "evidence_refs", "handoff_refs"],
      permission_queue: "none",
      description: "展示 accepted 结论和证据引用，不写新的事实。",
    },
  ];
}

export function deriveProjectWorkflowCanvasReadModel({
  project,
  projectWorkflow,
  projectBlackboard,
  selectedTask,
  workflowStatePath = null,
  workflowStateUpdatedAt = null,
  generatedAt = null,
  runtimeSessionAttention = [],
}: {
  project: ProjectRecord;
  projectWorkflow: ProjectWorkflowSummary | null;
  projectBlackboard: ProjectBlackboard | null;
  selectedTask: TaskDraftSummary | null;
  workflowStatePath?: string | null;
  workflowStateUpdatedAt?: string | null;
  generatedAt?: string | null;
  runtimeSessionAttention?: RuntimeSessionAttention[];
}): ProjectWorkflowCanvasReadModel {
  const workflowId = projectWorkflow?.workflow_id ?? `missing-workflow:${slug(project.project_root)}`;
  const projectId = projectWorkflow?.project_id ?? `project:${slug(project.project_root)}`;
  const derivedWorkflow = projectWorkflow?.derived_workflow ?? null;
  const selectedTaskPackage = selectTaskPackage(derivedWorkflow?.task_packages ?? [], selectedTask);
  const runtimeAttention = matchingRuntimeAttention(runtimeSessionAttention, projectWorkflow, selectedTask);
  const status = canvasStatus(projectWorkflow, selectedTask, runtimeAttention);
  const sourceRefs = compactRefs([
    ref("project", project.project_root, project.name),
    projectWorkflow ? ref("workflow", projectWorkflow.workflow_id, projectWorkflow.title) : null,
    selectedTask ? ref("work_item", selectedTask.work_item_id, selectedTask.title) : null,
    selectedTaskPackage ? ref("task_package", selectedTaskPackage.task_package_id, selectedTaskPackage.task_goal) : null,
    selectedTaskPackage?.memory_injection_summary?.snapshot_id
      ? ref("memory_packet", selectedTaskPackage.memory_injection_summary.snapshot_id, "任务包记忆快照")
      : null,
    ...runtimeAttention.slice(0, 3).map((attention) => ref("runtime_attention", attention.attention_id, attention.title)),
    ...(selectedTask?.recent_audit_events ?? []).slice(0, 3).map((event) => ref("audit_event", event.event_id, event.event_type)),
  ]);
  const attentionItems = buildAttentionItems({
    projectWorkflow,
    derivedWorkflow,
    selectedTask,
    selectedTaskPackage,
    projectBlackboard,
    runtimeAttention,
    status,
  });
  const statusReason = buildStatusReason(status, projectWorkflow, selectedTask, attentionItems, sourceRefs);
  const nodes: ProjectCanvasNode[] = [];
  const detailPanels: Record<string, ProjectCanvasNodeDetail> = {};
  const goalNode = buildGoalNode(project, workflowId, projectWorkflow, selectedTask, status);
  nodes.push(goalNode);
  detailPanels[goalNode.detail_panel_id] = buildDetail(goalNode, {
    projectWorkflow,
    derivedWorkflow,
    selectedTask,
    selectedTaskPackage,
    binding: null,
    dispatch: null,
    permissionRequests: [],
    executionAttempts: [],
    blackboardEntries: [],
    directorReviews: [],
  });

  const roleNodes = roleLanes.map((role) => {
    const workflowNode = findWorkflowNode(derivedWorkflow?.nodes ?? [], workflowId, role.role_id);
    const workflowNodeId = workflowNode?.workflow_node_id ?? `${workflowId}:node:${role.role_id}`;
    const binding = findRoleBinding(projectWorkflow?.node_session_bindings ?? [], workflowNodeId, selectedTask);
    const dispatch = findRoleDispatch(projectWorkflow?.node_dispatches ?? [], workflowNodeId, selectedTask);
    const permissions = relatedPermissions(projectWorkflow?.permission_requests ?? [], workflowNodeId, selectedTask);
    const attempts = relatedAttempts(projectWorkflow?.execution_attempts ?? [], dispatch, selectedTask);
    const blackboardEntries = relatedBlackboardEntries(projectBlackboard?.entries ?? [], workflowNodeId, selectedTask);
    const directorReviews = relatedDirectorReviews(projectWorkflow?.director_reviews ?? [], dispatch, selectedTask);
    const node = buildRoleNode({
      role,
      workflowId,
      workflowNode,
      workflowNodeId,
      selectedTask,
      selectedTaskPackage,
      binding,
      dispatch,
      permissionRequests: permissions,
      executionAttempts: attempts,
      blackboardEntries,
    });
    detailPanels[node.detail_panel_id] = buildDetail(node, {
      projectWorkflow,
      derivedWorkflow,
      selectedTask,
      selectedTaskPackage,
      binding,
      dispatch,
      permissionRequests: permissions,
      executionAttempts: attempts,
      blackboardEntries,
      directorReviews,
    });
    return node;
  });
  nodes.push(...roleNodes);

  const sidecarNodes = buildSidecarNodes({
    workflowId,
    selectedTask,
    permissionRequests: projectWorkflow?.permission_requests ?? [],
    blackboardEntries: projectBlackboard?.entries ?? [],
  });
  for (const node of sidecarNodes) {
    nodes.push(node);
    const targetWorkflowNodeId = node.workflow_node_id ?? dispatchNodeIdForTask(selectedTask);
    detailPanels[node.detail_panel_id] = buildDetail(node, {
      projectWorkflow,
      derivedWorkflow,
      selectedTask,
      selectedTaskPackage,
      binding: null,
      dispatch: null,
      permissionRequests: node.node_type === "permission_request" ? relatedPermissions(projectWorkflow?.permission_requests ?? [], targetWorkflowNodeId, selectedTask) : [],
      executionAttempts: [],
      blackboardEntries:
        node.node_type === "blackboard_candidate"
          ? (projectBlackboard?.entries ?? []).filter((entry) => node.source_refs.some((source) => source.id === entry.entry_id))
          : [],
      directorReviews: [],
    });
  }

  const edges = buildEdges(workflowId, nodes, selectedTask);
  const selectedNodeId = preferredSelectedNodeId(nodes, selectedTask);
  const editBoundary = buildProjectWorkflowEditBoundary({
    workflowId,
    sourceRefs,
    nodes,
    edges,
    selectedTask,
  });
  const warnings = [
    ...(projectWorkflow ? [] : ["missing_project_workflow"]),
    ...(projectWorkflow?.derived_workflow?.warnings ?? []),
    ...(projectBlackboard?.warnings ?? []),
    ...attentionItems.filter((attention) => attention.severity === "blocking").map((attention) => attention.summary),
  ];

  return {
    schema_version: "project_workflow_canvas.v1",
    project_id: projectId,
    project_root: project.project_root,
    workflow_id: workflowId,
    title: projectWorkflow?.title ?? `${project.name} 项目工作流`,
    status,
    source: {
      source_kind: "workflow_state_read_model",
      workflow_state_path: workflowStatePath,
      workflow_state_updated_at: workflowStateUpdatedAt,
      derived_from: sourceRefs,
      generated_at: generatedAt,
    },
    viewport_hint: {
      layout: "role_lanes_v1",
      selected_node_id: selectedNodeId,
    },
    status_reason: statusReason,
    nodes,
    edges,
    detail_panels: detailPanels,
    global_badges: [
      badge("canvas-status", status, toneForStatus(status), sourceRefs),
      badge("nodes", `${nodes.length} nodes`, "neutral", sourceRefs),
      badge(
        "attention",
        `${attentionItems.length} attention`,
        attentionItems.some((attention) => attention.severity === "blocking" || attention.severity === "needs_user") ? "warning" : "neutral",
        attentionItems.flatMap((attention) => attention.source_refs).slice(0, 3),
      ),
      badge(
        "permissions",
        `${(projectWorkflow?.permission_requests ?? []).filter((request) => request.status === "pending").length} pending`,
        (projectWorkflow?.permission_requests ?? []).some((request) => request.status === "pending") ? "warning" : "neutral",
        [],
      ),
      badge("blackboard", `${projectBlackboard?.entries.length ?? 0} refs`, projectBlackboard?.entries.length ? "warning" : "neutral", []),
    ],
    attention_items: attentionItems,
    edit_boundary: editBoundary,
    warnings,
  };
}

function buildProjectWorkflowEditBoundary({
  workflowId,
  sourceRefs,
  nodes,
  edges,
  selectedTask,
}: {
  workflowId: string;
  sourceRefs: ProjectCanvasSourceRef[];
  nodes: ProjectCanvasNode[];
  edges: ProjectCanvasEdge[];
  selectedTask: TaskDraftSummary | null;
}): ProjectWorkflowEditBoundary {
  const boundaryRefs = compactRefs([
    ref("workflow", workflowId, "项目工作流画布"),
    selectedTask ? ref("work_item", selectedTask.work_item_id, selectedTask.title) : null,
    ...sourceRefs.slice(0, 3),
  ]);
  const layoutBoundary: ProjectCanvasLayoutBoundary = {
    layout_kind: "role_lanes_v1",
    scope: "view_only",
    summary: "仅视图布局；未保存为事实；React Flow 仅负责渲染。",
    react_flow_source_of_truth: false,
    writes_workflow_state: false,
    persists_layout: false,
    reset_view_allowed: true,
    source_refs: boundaryRefs,
    warnings: ["layout_preview_not_persisted", "react_flow_not_workflow_source_of_truth"],
  };
  const capabilities: ProjectCanvasEditCapability[] = [
    {
      capability_id: `${workflowId}:edit-capability:view-only`,
      kind: "view_only",
      label: "查看项目工作流",
      status: "allowed",
      summary: `可查看 ${nodes.length} 个节点和 ${edges.length} 条边；不改 workflow 事实。`,
      changes_workflow_facts: false,
      requires_proposal: false,
      requires_confirmation: false,
      requires_control_core: false,
      requires_audit: false,
      source_refs: boundaryRefs,
    },
    {
      capability_id: `${workflowId}:edit-capability:local-layout-preview`,
      kind: "local_layout_preview",
      label: "本地布局预览",
      status: "allowed",
      summary: "允许适配视图、缩放、平移和临时查看；布局不会写入工作流状态。",
      changes_workflow_facts: false,
      requires_proposal: false,
      requires_confirmation: false,
      requires_control_core: false,
      requires_audit: false,
      source_refs: boundaryRefs,
    },
    {
      capability_id: `${workflowId}:edit-capability:personal-layout-preference`,
      kind: "personal_layout_preference",
      label: "个人布局偏好",
      status: "requires_future_task",
      summary: "F3 不新增持久布局存储；如需保存，必须另拆任务说明作用域、冲突和回滚。",
      changes_workflow_facts: false,
      requires_proposal: false,
      requires_confirmation: false,
      requires_control_core: false,
      requires_audit: false,
      source_refs: boundaryRefs,
    },
    {
      capability_id: `${workflowId}:edit-capability:workflow-node-mutation`,
      kind: "workflow_node_mutation",
      label: "工作流节点变更",
      status: "preview_only",
      summary: "新增、删除或改角色只能进入提案预览；当前不可直接执行。",
      changes_workflow_facts: true,
      requires_proposal: true,
      requires_confirmation: true,
      requires_control_core: true,
      requires_audit: true,
      source_refs: boundaryRefs,
    },
    {
      capability_id: `${workflowId}:edit-capability:workflow-edge-mutation`,
      kind: "workflow_edge_mutation",
      label: "工作流连线变更",
      status: "preview_only",
      summary: "新增、删除或改连线只能进入提案预览；React Flow 连线不会保存。",
      changes_workflow_facts: true,
      requires_proposal: true,
      requires_confirmation: true,
      requires_control_core: true,
      requires_audit: true,
      source_refs: boundaryRefs,
    },
    {
      capability_id: `${workflowId}:edit-capability:permission-model-mutation`,
      kind: "permission_or_model_mutation",
      label: "权限 / 模型变更",
      status: "blocked",
      summary: "权限、模型、工具或读写范围属于高风险事实变更；需要确认弹层、控制核心和审计。",
      changes_workflow_facts: true,
      requires_proposal: true,
      requires_confirmation: true,
      requires_control_core: true,
      requires_audit: true,
      source_refs: boundaryRefs,
    },
    {
      capability_id: `${workflowId}:edit-capability:execution-mutation`,
      kind: "execution_mutation",
      label: "执行变更",
      status: "blocked",
      summary: "启动工作者、派发或重试不属于 F3；需要另行授权真实执行。",
      changes_workflow_facts: true,
      requires_proposal: true,
      requires_confirmation: true,
      requires_control_core: true,
      requires_audit: true,
      source_refs: boundaryRefs,
    },
  ];
  const proposalPreviews: WorkflowEditProposalPreview[] = [
    {
      preview_id: `${workflowId}:edit-preview:node-mutation`,
      change_kind: "workflow_node_mutation",
      label: "节点变更提案预览",
      status: "preview_only",
      summary: "需要生成提案后才能讨论新增、删除或变更节点。",
      changes_workflow_facts: true,
      requires_proposal: true,
      requires_confirmation: true,
      requires_control_core: true,
      requires_audit: true,
      disabled_reason: "当前只显示预览，不写工作流状态。",
      source_refs: boundaryRefs,
    },
    {
      preview_id: `${workflowId}:edit-preview:edge-mutation`,
      change_kind: "workflow_edge_mutation",
      label: "边变更提案预览",
      status: "preview_only",
      summary: "需要生成提案后才能讨论新增、删除或变更连线。",
      changes_workflow_facts: true,
      requires_proposal: true,
      requires_confirmation: true,
      requires_control_core: true,
      requires_audit: true,
      disabled_reason: "React Flow 连线不保存为 workflow 边事实。",
      source_refs: boundaryRefs,
    },
    {
      preview_id: `${workflowId}:edit-preview:permission-model`,
      change_kind: "permission_or_model_mutation",
      label: "高风险变更预览",
      status: "blocked",
      summary: "权限、模型、工具和范围变更需要确认弹层、控制核心和审计。",
      changes_workflow_facts: true,
      requires_proposal: true,
      requires_confirmation: true,
      requires_control_core: true,
      requires_audit: true,
      disabled_reason: "F3 不在画布侧栏直接批准高风险权限或模型变更。",
      source_refs: boundaryRefs,
    },
    {
      preview_id: `${workflowId}:edit-preview:execution`,
      change_kind: "execution_mutation",
      label: "执行变更预览",
      status: "blocked",
      summary: "执行、派发和重试需要另行授权；当前不执行真实 Codex。",
      changes_workflow_facts: true,
      requires_proposal: true,
      requires_confirmation: true,
      requires_control_core: true,
      requires_audit: true,
      disabled_reason: "F3 不启动工作者，也不执行真实 Codex 命令。",
      source_refs: boundaryRefs,
    },
  ];
  return {
    boundary_id: `${workflowId}:edit-boundary:f3`,
    source_kind: "frontend_read_model",
    layout_boundary: layoutBoundary,
    capabilities,
    proposal_previews: proposalPreviews,
    warnings: [
      "react_flow_not_source_of_truth",
      "layout_not_persisted",
      "workflow_mutation_requires_proposal_control_core_audit",
      "execution_requires_separate_authorization",
    ],
  };
}

function buildGoalNode(
  project: ProjectRecord,
  workflowId: string,
  projectWorkflow: ProjectWorkflowSummary | null,
  selectedTask: TaskDraftSummary | null,
  status: ProjectCanvasStatus,
): ProjectCanvasNode {
  const nodeId = `${workflowId}:canvas:goal`;
  const refs = compactRefs([
    projectWorkflow ? ref("workflow", projectWorkflow.workflow_id, projectWorkflow.title) : null,
    selectedTask ? ref("work_item", selectedTask.work_item_id, selectedTask.title) : null,
  ]);
  return {
    node_id: nodeId,
    node_type: "project_goal",
    title: projectWorkflow?.title ?? project.name,
    subtitle: selectedTask?.title ?? "未登记当前工作项",
    status,
    role_id: "project",
    work_item_id: selectedTask?.work_item_id ?? null,
    workflow_node_id: null,
    position_hint: { lane: "goal", order: 0, x: 20, y: 150 },
    badges: [
      badge("source", projectWorkflow ? "项目事实" : "缺 workflow", projectWorkflow ? "ready" : "warning", refs),
      badge("task", selectedTask ? selectedTask.state : "无工作项", selectedTask ? toneForStatus(status) : "neutral", refs),
    ],
    metrics: [
      { metric_id: "sessions", label: "会话", value: project.thread_count },
      { metric_id: "files", label: "资料", value: project.authority_files.length + project.handoff_files.length + project.evidence_files.length },
    ],
    source_refs: refs,
    detail_panel_id: `${nodeId}:detail`,
    warnings: projectWorkflow ? [] : ["当前项目缺少默认 workflow，只显示占位读模型。"],
  };
}

function buildRoleNode({
  role,
  workflowId,
  workflowNode,
  workflowNodeId,
  selectedTask,
  selectedTaskPackage,
  binding,
  dispatch,
  permissionRequests,
  executionAttempts,
  blackboardEntries,
}: {
  role: RoleLane;
  workflowId: string;
  workflowNode?: WorkflowNode | null;
  workflowNodeId: string;
  selectedTask: TaskDraftSummary | null;
  selectedTaskPackage: TaskPackage | null;
  binding: WorkflowNodeSessionBinding | null;
  dispatch: WorkflowNodeDispatchRecord | null;
  permissionRequests: WorkflowPermissionRequestRecord[];
  executionAttempts: WorkflowExecutionAttemptRecord[];
  blackboardEntries: BlackboardEntry[];
}): ProjectCanvasNode {
  const nodeId = `${workflowId}:canvas:${role.role_id}`;
  const status = roleStatus(role.role_id, workflowNodeId, selectedTask, dispatch, permissionRequests, executionAttempts);
  const refs = compactRefs([
    ref("workflow_node", workflowNodeId, workflowNode?.title ?? role.title),
    selectedTask ? ref("work_item", selectedTask.work_item_id, selectedTask.title) : null,
    selectedTaskPackage ? ref("task_package", selectedTaskPackage.task_package_id, selectedTaskPackage.task_goal) : null,
    binding ? ref("node_binding", binding.binding_id, binding.session_title) : null,
    dispatch ? ref("dispatch", dispatch.dispatch_id, dispatch.state) : null,
  ]);
  const warnings = [
    ...(workflowNode?.warnings ?? []),
    ...(workflowNode?.missing_fields ?? []).map((field) => `missing:${field}`),
    ...(binding && !binding.rollout_exists ? ["rollout_missing"] : []),
    ...blackboardEntries.flatMap((entry) => entry.warnings.slice(0, 1)),
  ];
  return {
    node_id: nodeId,
    node_type: role.node_type,
    title: workflowNode?.title ?? role.title,
    subtitle: binding?.session_title ?? workflowNode?.assigned_role ?? role.subtitle,
    status,
    role_id: role.role_id,
    work_item_id: selectedTask?.work_item_id ?? null,
    workflow_node_id: workflowNodeId,
    position_hint: { lane: role.lane, order: roleLanes.indexOf(role) + 1, x: role.x, y: role.y },
    badges: [
      badge("role", role.subtitle, "neutral", refs),
      badge("status", status, toneForStatus(status), refs),
      ...(binding ? [badge("binding", binding.rollout_exists ? "已绑定" : "缺回放记录", binding.rollout_exists ? "ready" : "warning", refs)] : []),
      ...(permissionRequests.some((request) => request.status === "pending") ? [badge("permission", "待权限", "blocked", refs)] : []),
      ...(warnings.length ? [badge("warning", `${warnings.length} 条警告`, "warning", refs)] : []),
    ],
    metrics: [
      { metric_id: "task", label: "任务", value: selectedTask?.state ?? "无" },
      { metric_id: "dispatch", label: "派发", value: dispatch?.state ?? "无" },
      { metric_id: "blackboard", label: "黑板", value: blackboardEntries.length, tone: blackboardEntries.length ? "warning" : "neutral" },
    ],
    source_refs: refs,
    detail_panel_id: `${nodeId}:detail`,
    warnings,
  };
}

function buildSidecarNodes({
  workflowId,
  selectedTask,
  permissionRequests,
  blackboardEntries,
}: {
  workflowId: string;
  selectedTask: TaskDraftSummary | null;
  permissionRequests: WorkflowPermissionRequestRecord[];
  blackboardEntries: BlackboardEntry[];
}): ProjectCanvasNode[] {
  const pendingPermissions = permissionRequests
    .filter((request) => request.status === "pending")
    .filter((request) => !selectedTask || request.work_item_id === selectedTask.work_item_id)
    .slice(0, 3);
  const pendingBlackboard = blackboardEntries
    .filter((entry) => ["candidate", "pending", "open"].includes(entry.status) || entry.promotion_decision.status.includes("pending"))
    .slice(0, 4);
  const permissionNodes = pendingPermissions.map((request, index): ProjectCanvasNode => {
    const nodeId = `${workflowId}:canvas:permission:${slug(request.request_id)}`;
    const refs = [ref("permission_request", request.request_id, request.permission_kind)];
    return {
      node_id: nodeId,
      node_type: "permission_request",
      title: "权限请求",
      subtitle: request.permission_kind,
      status: "waiting_for_permission",
      role_id: null,
      work_item_id: request.work_item_id,
      workflow_node_id: dispatchNodeIdForTask(selectedTask),
      position_hint: { lane: "sidecar", order: index, x: 620 + index * 140, y: 360 },
      badges: [badge("pending", "待确认", "blocked", refs)],
      metrics: [{ metric_id: "status", label: "状态", value: request.status, tone: "blocked" }],
      source_refs: refs,
      detail_panel_id: `${nodeId}:detail`,
      warnings: request.warnings,
    };
  });
  const blackboardNodes = pendingBlackboard.map((entry, index): ProjectCanvasNode => {
    const nodeId = `${workflowId}:canvas:blackboard:${slug(entry.entry_id)}`;
    const refs = [ref("blackboard_entry", entry.entry_id, entry.title)];
    const risk = entry.kind === "risk" || entry.kind === "permission_request";
    return {
      node_id: nodeId,
      node_type: "blackboard_candidate",
      title: entry.title,
      subtitle: entry.summary,
      status: risk ? "blocked" : "idle",
      role_id: null,
      work_item_id: entry.work_item_id ?? selectedTask?.work_item_id ?? null,
      workflow_node_id: entry.workflow_node_id ?? dispatchNodeIdForTask(selectedTask),
      position_hint: { lane: "sidecar", order: permissionNodes.length + index, x: 420 + index * 190, y: 510 },
      badges: [
        badge("kind", blackboardKindLabel(entry.kind), risk ? "warning" : "neutral", refs),
        badge("promotion", entry.promotion_decision.status, "warning", refs),
      ],
      metrics: [
        { metric_id: "source", label: "来源", value: entry.source_refs.map((source) => source.label).join(" / ") || "无" },
      ],
      source_refs: refs,
      detail_panel_id: `${nodeId}:detail`,
      warnings: [...entry.warnings, ...entry.promotion_decision.warnings],
    };
  });
  return [...permissionNodes, ...blackboardNodes];
}

function buildEdges(workflowId: string, nodes: ProjectCanvasNode[], selectedTask: TaskDraftSummary | null): ProjectCanvasEdge[] {
  const node = (suffix: string) => `${workflowId}:canvas:${suffix}`;
  const roleEdges: ProjectCanvasEdge[] = [
    edge("goal-director", "responsibility_flow", node("goal"), node("director"), "目标"),
    edge("director-dev", "responsibility_flow", node("director"), node("codex-dev"), "派发"),
    edge("dev-validation", "handoff_flow", node("codex-dev"), node("validation"), "验证"),
    edge("validation-review", "review_flow", node("validation"), node("review"), "回收"),
    edge("review-director", "review_flow", node("review"), node("director"), "结论"),
  ];
  const sidecarEdges = nodes
    .filter((item) => item.node_type === "permission_request" || item.node_type === "blackboard_candidate")
    .map((item, index) => {
      const targetRole = roleSuffixForWorkflowNode(item.workflow_node_id ?? dispatchNodeIdForTask(selectedTask));
      return edge(`sidecar-${index}`, "blocking_relation", item.node_id, node(targetRole), item.node_type === "permission_request" ? "阻塞" : "候选");
    });
  return [...roleEdges, ...sidecarEdges].filter((item) =>
    nodes.some((nodeItem) => nodeItem.node_id === item.source_node_id) && nodes.some((nodeItem) => nodeItem.node_id === item.target_node_id),
  );

  function edge(
    suffix: string,
    edgeType: ProjectCanvasEdgeType,
    sourceNodeId: string,
    targetNodeId: string,
    label: string,
  ): ProjectCanvasEdge {
    const sourceNode = nodes.find((nodeItem) => nodeItem.node_id === sourceNodeId);
    const targetNode = nodes.find((nodeItem) => nodeItem.node_id === targetNodeId);
    const status = edgeStatus(sourceNode?.status, targetNode?.status);
    return {
      edge_id: `${workflowId}:edge:${suffix}`,
      edge_type: edgeType,
      source_node_id: sourceNodeId,
      target_node_id: targetNodeId,
      status,
      label,
      source_refs: compactRefs([sourceNode?.source_refs[0] ?? null, targetNode?.source_refs[0] ?? null]),
      warnings: [],
    };
  }
}

function buildDetail(
  node: ProjectCanvasNode,
  {
    projectWorkflow,
    derivedWorkflow,
    selectedTask,
    selectedTaskPackage,
    binding,
    dispatch,
    permissionRequests,
    executionAttempts,
    blackboardEntries,
    directorReviews,
  }: {
    projectWorkflow: ProjectWorkflowSummary | null;
    derivedWorkflow: Workflow | null;
    selectedTask: TaskDraftSummary | null;
    selectedTaskPackage: TaskPackage | null;
    binding: WorkflowNodeSessionBinding | null;
    dispatch: WorkflowNodeDispatchRecord | null;
    permissionRequests: WorkflowPermissionRequestRecord[];
    executionAttempts: WorkflowExecutionAttemptRecord[];
    blackboardEntries: BlackboardEntry[];
    directorReviews: WorkflowDispatchDirectorReviewRecord[];
  },
): ProjectCanvasNodeDetail {
  const memorySummary = selectedTaskPackage?.memory_injection_summary ?? null;
  const sections = compactSections([
    section("summary", "用户摘要 / 节点状态", "summary", userSummaryItems(node, selectedTask, dispatch, permissionRequests, executionAttempts), "user_summary"),
    selectedTask
      ? section("task", "当前任务", "task_package", [
          item("work-item", "工作项", selectedTask.work_item_id, "ref", [ref("work_item", selectedTask.work_item_id, selectedTask.title)]),
          item("task-title", "标题", selectedTask.title, "text", [ref("work_item", selectedTask.work_item_id, selectedTask.title)]),
          item("task-state", "状态", selectedTask.state, "status", [ref("work_item", selectedTask.work_item_id, selectedTask.title)]),
          item("next", "下一步", selectedTask.next_action_label ?? "未登记", "text", [ref("work_item", selectedTask.work_item_id, selectedTask.title)]),
          item("artifact", "任务包", selectedTask.artifact_path ?? selectedTaskPackage?.task_package_id ?? "未生成", selectedTask.artifact_path ? "path" : "ref", node.source_refs),
        ])
      : null,
    selectedTaskPackage
      ? section("package", "任务包字段", "task_package", [
          item("model", "模型", selectedTaskPackage.model_id ?? "missing: model_id", selectedTaskPackage.model_id ? "text" : "warning", [ref("task_package", selectedTaskPackage.task_package_id)]),
          item("read-scope", "允许读取", listValue(selectedTaskPackage.allowed_read_scope, "missing: allowed_read_scope"), "text", [ref("task_package", selectedTaskPackage.task_package_id)]),
          item("write-scope", "允许写入", listValue(selectedTaskPackage.allowed_write_scope, "missing: allowed_write_scope"), selectedTaskPackage.allowed_write_scope.length ? "text" : "warning", [ref("task_package", selectedTaskPackage.task_package_id)]),
          item("acceptance", "验收标准", listValue(selectedTaskPackage.acceptance_criteria, "missing: acceptance_criteria"), selectedTaskPackage.acceptance_criteria.length ? "text" : "warning", [ref("task_package", selectedTaskPackage.task_package_id)]),
          item("stale", "stale", selectedTaskPackage.stale ? selectedTaskPackage.stale_reasons.join("；") || "stale" : "fresh", selectedTaskPackage.stale ? "warning" : "status", [ref("task_package", selectedTaskPackage.task_package_id)]),
        ])
      : null,
    selectedTaskPackage
      ? section("memory-packet", "任务记忆包摘要", "memory_packet", [
          item(
            "memory-snapshot",
            "快照",
            memorySummary?.snapshot_id ?? "未生成；候选和观察不会当作正式记忆注入",
            memorySummary?.snapshot_id ? "ref" : "warning",
            memorySummary?.snapshot_id
              ? [ref("memory_packet", memorySummary.snapshot_id, "任务包记忆快照")]
              : [ref("task_package", selectedTaskPackage.task_package_id)],
          ),
          item("memory-included", "入选正式记忆", String(memorySummary?.included_count ?? 0), "count", [ref("task_package", selectedTaskPackage.task_package_id)]),
          item("memory-excluded", "排除材料", String(memorySummary?.excluded_count ?? 0), "count", [ref("task_package", selectedTaskPackage.task_package_id)]),
          item("memory-review", "待审材料", String(memorySummary?.review_material_count ?? 0), "count", [ref("task_package", selectedTaskPackage.task_package_id)]),
          item(
            "memory-reason",
            "理由摘要",
            memorySummary?.display_text ?? "仅活跃正式记忆可入选；候选 / 观察只作为待审材料，不写正式记忆。",
            "text",
            [ref("task_package", selectedTaskPackage.task_package_id)],
          ),
          item(
            "memory-stale",
            "状态",
            memorySummary ? (memorySummary.stale ? memorySummary.stale_reasons.join("；") || "stale" : "fresh") : "未生成",
            memorySummary?.stale || !memorySummary ? "warning" : "status",
            [ref("task_package", selectedTaskPackage.task_package_id)],
          ),
        ])
      : null,
    binding
      ? section("binding", "会话绑定", "session_binding", [
          item("session", "会话", binding.session_title, "text", [ref("node_binding", binding.binding_id, binding.session_title)]),
          item("thread", "会话", binding.native_thread_id, "ref", [ref("node_binding", binding.binding_id, binding.session_title)]),
          item("rollout", "读取状态", binding.rollout_exists ? "可读取" : "缺回放记录", binding.rollout_exists ? "status" : "warning", [ref("node_binding", binding.binding_id, binding.session_title)]),
        ])
      : null,
    dispatch
      ? section("dispatch", "派发摘要", "dispatch", [
          item("dispatch-id", "dispatch", dispatch.dispatch_id, "ref", [ref("dispatch", dispatch.dispatch_id, dispatch.state)]),
          item("dispatch-state", "状态", dispatch.state, "status", [ref("dispatch", dispatch.dispatch_id, dispatch.state)]),
          item("prompt-kind", "模式", dispatch.prompt_kind, "text", [ref("dispatch", dispatch.dispatch_id, dispatch.state)]),
          item("last-message", "最后摘要", dispatch.last_message_summary ?? "未回读", "text", [ref("dispatch", dispatch.dispatch_id, dispatch.state)]),
          item("authorization", "授权检查", dispatch.authorization_check?.status ?? dispatch.plan_authorization_id ?? "未登记", dispatch.authorization_check?.status === "blocked" ? "blocked" : "status", [ref("authorization_check", dispatch.dispatch_id, dispatch.authorization_check?.status ?? "未登记")]),
          item("warnings", "warning", dispatch.warnings.join("；") || "无", dispatch.warnings.length ? "warning" : "text", [ref("dispatch", dispatch.dispatch_id, dispatch.state)]),
        ])
      : null,
    dispatch
      ? section("readback", "读回摘要", "readback", [
          item("readback-status", "状态", readbackStatusForDispatch(dispatch), readbackUnavailableForDispatch(dispatch) ? "warning" : "status", [ref("readback", dispatch.dispatch_id, "readback")]),
          item("readback-events", "event count", dispatch.transcript_event_count == null ? "不可用" : String(dispatch.transcript_event_count), dispatch.transcript_event_count == null ? "warning" : "count", [ref("readback", dispatch.dispatch_id, "readback")]),
          item("readback-hits", "target hits", dispatch.transcript_target_hits == null ? "不可用" : String(dispatch.transcript_target_hits), dispatch.transcript_target_hits == null ? "warning" : "count", [ref("readback", dispatch.dispatch_id, "readback")]),
        ])
      : null,
    permissionRequests.length
      ? section(
          "permissions",
          "权限请求",
          "permission_requests",
          permissionRequests.map((request) =>
            item(request.request_id, request.permission_kind, `${request.status} / ${request.reason}`, request.status === "pending" ? "blocked" : "status", [
              ref("permission_request", request.request_id, request.permission_kind),
            ]),
          ),
        )
      : null,
    blackboardEntries.length
      ? section(
          "blackboard",
          "黑板候选",
          "blackboard_entries",
          blackboardEntries.map((entry) =>
            item(entry.entry_id, blackboardKindLabel(entry.kind), `${entry.title} / ${entry.promotion_decision.status}`, "warning", [
              ref("blackboard_entry", entry.entry_id, entry.title),
            ]),
          ),
        )
      : null,
    executionAttempts.length
      ? section(
          "attempts",
          "失败 / 超时",
          "execution_attempts",
          executionAttempts.map((attempt) =>
            item(attempt.attempt_id, `第 ${attempt.attempt_no} 次`, `${attempt.state} / ${attempt.failure_reason ?? attempt.timed_out_at ?? "无异常摘要"}`, attempt.state === "failed" || attempt.state === "timed_out" ? "warning" : "status", [
              ref("execution_attempt", attempt.attempt_id, attempt.state),
            ]),
          ),
        )
      : null,
    directorReviews.length
      ? section(
          "director-reviews",
          "总指导回收摘要",
          "review_results",
          directorReviews.map((review) =>
            item(review.review_id, review.decision, review.summary, review.decision === "accepted" ? "status" : "warning", [
              ref("director_review", review.review_id, review.decision),
            ]),
          ),
        )
      : null,
    directorReviews.some((review) => review.evidence_refs.length)
      ? section(
          "evidence-refs",
          "Evidence 引用",
          "evidence_refs",
          directorReviews.flatMap((review) =>
            review.evidence_refs.slice(0, 4).map((evidenceRef, index) =>
              item(`${review.review_id}:evidence:${index}`, "evidence", evidenceRef, "ref", [
                ref("evidence_ref", evidenceRef, "evidence"),
                ref("director_review", review.review_id, review.decision),
              ]),
            ),
          ),
        )
      : null,
    directorReviews.some((review) => review.handoff_refs.length)
      ? section(
          "handoff-refs",
          "Handoff 引用",
          "handoff_refs",
          directorReviews.flatMap((review) =>
            review.handoff_refs.slice(0, 4).map((handoffRef, index) =>
              item(`${review.review_id}:handoff:${index}`, "handoff", handoffRef, "ref", [
                ref("handoff_ref", handoffRef, "handoff"),
                ref("director_review", review.review_id, review.decision),
              ]),
            ),
          ),
        )
      : null,
    derivedWorkflow
      ? section("run-check", "技术详情：运行与完成闸门", "completion_gate", [
          item("run-check", "运行检查", derivedWorkflow.run_check_status, derivedWorkflow.run_check_status === "blocked" ? "blocked" : "status", [ref("workflow", derivedWorkflow.workflow_id)]),
          item("gate", "完成闸门", derivedWorkflow.state_machine.completion_gate.can_complete ? "can_complete" : listValue(derivedWorkflow.state_machine.completion_gate.missing, "missing"), derivedWorkflow.state_machine.completion_gate.can_complete ? "status" : "blocked", [ref("workflow", derivedWorkflow.workflow_id)]),
          item("reviews", "审查结果", `${derivedWorkflow.review_results.length} 条`, "count", [ref("workflow", derivedWorkflow.workflow_id)]),
          item("ledger", "账本摘要", `${derivedWorkflow.ledger_entries.length} 条`, "count", [ref("workflow", derivedWorkflow.workflow_id)]),
        ], "technical_details", false)
      : null,
    selectedTask?.recent_audit_events.length
      ? section(
          "audit",
          "技术详情：最近审计",
          "audit_refs",
          selectedTask.recent_audit_events.map((event) =>
            item(event.event_id, event.event_type, `${event.before_state ?? "unknown"} -> ${event.after_state ?? "unknown"} / ${event.reason ?? event.created_at ?? ""}`, "ref", [
              ref("audit_event", event.event_id, event.event_type),
            ]),
          ),
          "technical_details",
          false,
        )
      : null,
    node.source_refs.length
      ? section(
          "source-refs",
          "技术详情：事实来源",
          "source_refs",
          node.source_refs.slice(0, 8).map((sourceRef, index) =>
            item(`source:${index}`, sourceRef.kind, sourceRef.label ? `${sourceRef.label} / ${sourceRef.id}` : sourceRef.id, "ref", [sourceRef]),
          ),
          "technical_details",
          false,
        )
      : null,
  ]);
  return {
    detail_panel_id: node.detail_panel_id,
    node_id: node.node_id,
    title: node.title,
    summary: node.subtitle ?? null,
    sections,
    allowed_actions: allowedActionsForNode(node, selectedTask, binding, dispatch, permissionRequests),
    source_refs: node.source_refs,
    warnings: node.warnings,
  };
}

function userSummaryItems(
  node: ProjectCanvasNode,
  selectedTask: TaskDraftSummary | null,
  dispatch: WorkflowNodeDispatchRecord | null,
  permissionRequests: WorkflowPermissionRequestRecord[],
  executionAttempts: WorkflowExecutionAttemptRecord[],
): ProjectCanvasDetailItem[] {
  return [
    item("current-node", "当前节点", `${canvasNodeTypeLabel(node.node_type)} / ${node.title}`, "text", node.source_refs),
    item("current-status", "当前状态", statusLabel(node.status), "status", node.source_refs),
    item("stop-reason", "为什么停下", nodePauseReason(node, selectedTask, dispatch, permissionRequests, executionAttempts), reasonKindForNode(node, permissionRequests, executionAttempts), node.source_refs),
    item("owner", "谁能处理", nodeOwnerHint(node, dispatch, permissionRequests, executionAttempts), permissionRequests.some((request) => request.status === "pending") ? "blocked" : "text", node.source_refs),
    item("next-step", "下一步", nodeNextStep(node, selectedTask, dispatch, permissionRequests, executionAttempts), "text", node.source_refs),
    item("workflow-node", "workflow node", node.workflow_node_id ?? "无", "ref", node.source_refs),
    item("warnings", "warning", node.warnings.join("；") || "无", node.warnings.length ? "warning" : "text", node.source_refs),
  ];
}

function reasonKindForNode(
  node: ProjectCanvasNode,
  permissionRequests: WorkflowPermissionRequestRecord[],
  executionAttempts: WorkflowExecutionAttemptRecord[],
): ProjectCanvasDetailItem["value_kind"] {
  if (permissionRequests.some((request) => request.status === "pending")) return "blocked";
  if (executionAttempts.some((attempt) => attempt.state === "failed" || attempt.state === "timed_out")) return "warning";
  if (node.status === "blocked" || node.status === "waiting_for_permission") return "blocked";
  if (node.status === "failed" || node.status === "timed_out" || node.status === "readback_unavailable" || node.status === "needs_review") return "warning";
  return "text";
}

function nodePauseReason(
  node: ProjectCanvasNode,
  selectedTask: TaskDraftSummary | null,
  dispatch: WorkflowNodeDispatchRecord | null,
  permissionRequests: WorkflowPermissionRequestRecord[],
  executionAttempts: WorkflowExecutionAttemptRecord[],
) {
  const pendingPermission = permissionRequests.find((request) => request.status === "pending");
  if (pendingPermission) return `权限待处理：${pendingPermission.permission_kind}；需走确认弹层和控制核心。`;
  if (dispatch?.authorization_check?.status === "blocked") {
    return dispatch.authorization_check.reasons.join("；") || "授权检查阻断；不能继续推进。";
  }
  if (dispatch?.authorization_check?.status === "needs_review") {
    return dispatch.authorization_check.reasons.join("；") || "授权检查需要复核。";
  }
  const failedAttempt = executionAttempts.find((attempt) => attempt.state === "failed" || attempt.state === "timed_out");
  if (failedAttempt) return failedAttempt.failure_reason ?? failedAttempt.timed_out_at ?? "只有失败 / 超时摘要，没有完整会话记录。";
  if (dispatch && readbackUnavailableForDispatch(dispatch)) return "读回不可用；不能显示成真实 0 条结果。";
  if (dispatch?.state === "prepared") return "准备派发只表示准备记录；仍未启动工作者或真实 Codex。";
  if (dispatch?.state === "running") return "运行状态来自工作流状态摘要；本详情不执行真实工作者。";
  if (node.warnings.length) return node.warnings.join("；");
  if (!selectedTask) return "没有选中工作项；当前只显示项目级摘要。";
  return `${selectedTask.title} 当前处于 ${statusLabel(node.status)}。`;
}

function nodeOwnerHint(
  node: ProjectCanvasNode,
  dispatch: WorkflowNodeDispatchRecord | null,
  permissionRequests: WorkflowPermissionRequestRecord[],
  executionAttempts: WorkflowExecutionAttemptRecord[],
) {
  if (permissionRequests.some((request) => request.status === "pending")) return "用户 / 项目主管通过确认弹层处理；详情面板不直接批准。";
  if (dispatch?.authorization_check?.status === "blocked" || dispatch?.authorization_check?.status === "needs_review") return "全局主管 / 项目主管复核授权边界。";
  if (executionAttempts.some((attempt) => attempt.state === "failed" || attempt.state === "timed_out") || node.status === "readback_unavailable") {
    return "项目主管复核失败或读回摘要；日志和诊断留给 G 阶段。";
  }
  if (node.node_type === "project_goal" || node.node_type === "director") return "项目主管查看和回收；用户只处理明确待办。";
  return "对应角色和项目主管查看；高风险动作必须另走确认。";
}

function nodeNextStep(
  node: ProjectCanvasNode,
  selectedTask: TaskDraftSummary | null,
  dispatch: WorkflowNodeDispatchRecord | null,
  permissionRequests: WorkflowPermissionRequestRecord[],
  executionAttempts: WorkflowExecutionAttemptRecord[],
) {
  if (permissionRequests.some((request) => request.status === "pending")) return "打开确认弹层后记录权限结论；本侧栏只显示摘要。";
  if (dispatch?.authorization_check?.status === "blocked" || dispatch?.authorization_check?.status === "needs_review") return "先复核授权范围，再决定是否继续。";
  if (executionAttempts.some((attempt) => attempt.state === "failed" || attempt.state === "timed_out")) return "查看失败摘要并另开重试 / 诊断任务；本轮不自动重试。";
  if (dispatch && readbackUnavailableForDispatch(dispatch)) return "查看读回摘要和证据 / 交接引用；不要当成空读回。";
  if (dispatch?.state === "prepared") return "等待后续受控派发任务包；当前只是准备态记录。";
  if (selectedTask?.next_action_label) return selectedTask.next_action_label;
  if (!selectedTask) return "先登记或选择工作项。";
  return "继续查看项目主管信息和技术详情。";
}

function allowedActionsForNode(
  node: ProjectCanvasNode,
  selectedTask: TaskDraftSummary | null,
  binding: WorkflowNodeSessionBinding | null,
  dispatch: WorkflowNodeDispatchRecord | null,
  permissionRequests: WorkflowPermissionRequestRecord[],
): ProjectCanvasAction[] {
  const sourceRefs = node.source_refs;
  return [
    {
      action_id: `${node.node_id}:inspect-run-check`,
      label: "检查运行前状态",
      action_kind: "inspect_run_check",
      enabled: Boolean(selectedTask),
      disabled_reason: selectedTask ? null : "缺少当前工作项",
      requires_confirmation: false,
      boundary: "只读运行前检查，不写工作流状态，不启动 Codex。",
      source_refs: sourceRefs,
    },
    {
      action_id: `${node.node_id}:open-session`,
      label: "打开绑定会话",
      action_kind: "open_agent_session",
      enabled: Boolean(binding),
      disabled_reason: binding ? null : "该节点未绑定会话",
      requires_confirmation: false,
      boundary: "只打开智能体会话视图，不读取完整会话记录。",
      source_refs: sourceRefs,
    },
    {
      action_id: `${node.node_id}:permission-decision`,
      label: "记录权限结论",
      action_kind: "record_permission_decision",
      enabled: permissionRequests.some((request) => request.status === "pending"),
      disabled_reason: permissionRequests.some((request) => request.status === "pending") ? null : "没有待确认权限",
      requires_confirmation: true,
      boundary: "通过控制核心记录权限结论，只写工作台工作流状态，不启动 Codex。",
      source_refs: sourceRefs,
    },
    {
      action_id: `${node.node_id}:director-review`,
      label: "记录总指导回收",
      action_kind: "record_director_review",
      enabled: Boolean(dispatch && selectedTask?.state === "ready_for_review"),
      disabled_reason: dispatch && selectedTask?.state === "ready_for_review" ? null : "需要 completed 派发和 ready_for_review 工作项",
      requires_confirmation: true,
      boundary: "只写 reviews[] 和审计事件，不启动 Codex、不读取完整会话记录。",
      source_refs: sourceRefs,
    },
  ];
}

function buildStatusReason(
  status: ProjectCanvasStatus,
  projectWorkflow: ProjectWorkflowSummary | null,
  selectedTask: TaskDraftSummary | null,
  attentionItems: ProjectCanvasAttention[],
  fallbackRefs: ProjectCanvasSourceRef[],
): ProjectCanvasStatusReason {
  const leadingAttention =
    attentionItems.find((attention) => attention.severity === "blocking") ??
    attentionItems.find((attention) => attention.severity === "needs_user") ??
    attentionItems[0] ??
    null;
  if (leadingAttention) {
    return {
      reason_id: `status-reason:${status}:${slug(leadingAttention.attention_id)}`,
      status,
      label: statusLabel(status),
      summary: leadingAttention.summary,
      source_refs: leadingAttention.source_refs,
    };
  }
  if (!projectWorkflow) {
    return {
      reason_id: "status-reason:empty:no-workflow",
      status,
      label: statusLabel(status),
      summary: "当前项目没有 workflow 读模型；画布只显示空态占位，不补编工作项。",
      source_refs: fallbackRefs,
    };
  }
  if (!selectedTask) {
    return {
      reason_id: "status-reason:empty:no-work-item",
      status,
      label: statusLabel(status),
      summary: "当前 workflow 没有选中工作项；画布不补编任务，只展示项目级摘要。",
      source_refs: fallbackRefs,
    };
  }
  return {
    reason_id: `status-reason:${status}`,
    status,
    label: statusLabel(status),
    summary: `${selectedTask.title} 当前处于 ${statusLabel(status)}。`,
    source_refs: fallbackRefs,
  };
}

function buildAttentionItems({
  projectWorkflow,
  derivedWorkflow,
  selectedTask,
  selectedTaskPackage,
  projectBlackboard,
  runtimeAttention,
  status,
}: {
  projectWorkflow: ProjectWorkflowSummary | null;
  derivedWorkflow: Workflow | null;
  selectedTask: TaskDraftSummary | null;
  selectedTaskPackage: TaskPackage | null;
  projectBlackboard: ProjectBlackboard | null;
  runtimeAttention: RuntimeSessionAttention[];
  status: ProjectCanvasStatus;
}): ProjectCanvasAttention[] {
  const attention: ProjectCanvasAttention[] = [];
  if (!projectWorkflow) {
    attention.push(canvasAttention("empty:no-workflow", "empty", "warning", "empty", "缺少项目 workflow", "没有 workflow 读模型；画布不补编任务。", false, []));
  } else if (!selectedTask) {
    attention.push(canvasAttention("empty:no-work-item", "empty", "info", "empty", "没有当前工作项", "workflow 存在，但没有选中 work item；当前只显示项目摘要。", false, [
      ref("workflow", projectWorkflow.workflow_id, projectWorkflow.title),
    ]));
  }

  for (const request of (projectWorkflow?.permission_requests ?? []).filter(
    (permission) => permission.status === "pending" && (!selectedTask || permission.work_item_id === selectedTask.work_item_id),
  )) {
    attention.push(
      canvasAttention(
        `permission:${request.request_id}`,
        "waiting_for_permission",
        "needs_user",
        "waiting_for_permission",
        "权限待处理",
        `${request.permission_kind}：${request.reason}`,
        true,
        [ref("permission_request", request.request_id, request.permission_kind)],
      ),
    );
  }

  for (const dispatch of (projectWorkflow?.node_dispatches ?? []).filter((item) => !selectedTask || item.work_item_id === selectedTask.work_item_id)) {
    if (dispatch.state === "prepared") {
      attention.push(
        canvasAttention(
          `prepared:${dispatch.dispatch_id}`,
          "prepared",
          "info",
          "prepared",
          "准备派发",
          "准备派发只表示准备记录；仍未启动工作者或真实 Codex。",
          false,
          [ref("dispatch", dispatch.dispatch_id, dispatch.state)],
        ),
      );
    }
    if (dispatch.authorization_check?.status === "needs_review") {
      attention.push(
        canvasAttention(
          `authorization:${dispatch.dispatch_id}`,
          "needs_review",
          "needs_user",
          "needs_review",
          "授权需要复核",
          dispatch.authorization_check.reasons.join("；") || "authorization check 需要人工复核。",
          true,
          [ref("authorization_check", dispatch.dispatch_id, dispatch.authorization_check.status)],
        ),
      );
    }
    if (dispatch.authorization_check?.status === "blocked") {
      attention.push(
        canvasAttention(
          `authorization-blocked:${dispatch.dispatch_id}`,
          "blocked",
          "blocking",
          "blocked",
          "授权阻断",
          dispatch.authorization_check.reasons.join("；") || "authorization check 阻断派发。",
          true,
          [ref("authorization_check", dispatch.dispatch_id, dispatch.authorization_check.status)],
        ),
      );
    }
    if (readbackUnavailableForDispatch(dispatch)) {
      attention.push(
        canvasAttention(
          `readback:${dispatch.dispatch_id}`,
          "readback_unavailable",
          "warning",
          "readback_unavailable",
          "读回不可用",
          "本节点没有可信读回摘要；不能显示为 0 条结果。",
          false,
          [ref("readback", dispatch.dispatch_id, "readback")],
        ),
      );
    }
  }

  for (const attempt of (projectWorkflow?.execution_attempts ?? []).filter((item) => !selectedTask || item.work_item_id === selectedTask.work_item_id)) {
    if (attempt.state === "failed" || attempt.state === "timed_out") {
      attention.push(
        canvasAttention(
          `attempt:${attempt.attempt_id}`,
          attempt.state === "timed_out" ? "timed_out" : "failed",
          attempt.state === "timed_out" ? "blocking" : "warning",
          attempt.state === "timed_out" ? "timed_out" : "failed",
          attempt.state === "timed_out" ? "执行超时" : "执行失败",
          attempt.failure_reason ?? attempt.timed_out_at ?? "只有失败 / 超时摘要，没有完整会话记录。",
          attempt.state === "timed_out",
          [ref("execution_attempt", attempt.attempt_id, attempt.state)],
        ),
      );
    }
  }

  if (derivedWorkflow?.run_check_status === "blocked") {
    attention.push(
      canvasAttention(
        `run-check:${derivedWorkflow.workflow_id}`,
        "blocked",
        "blocking",
        "blocked",
        "运行检查阻断",
        "run_check_status=blocked；画布只显示摘要，不启动真实执行。",
        true,
        [ref("run_check", derivedWorkflow.workflow_id, derivedWorkflow.run_check_status)],
      ),
    );
  }

  if (selectedTaskPackage?.missing_fields.length) {
    attention.push(
      canvasAttention(
        `task-package:${selectedTaskPackage.task_package_id}`,
        "task_package",
        "warning",
        status,
        "任务包字段缺失",
        selectedTaskPackage.missing_fields.join("；"),
        false,
        [ref("task_package", selectedTaskPackage.task_package_id, selectedTaskPackage.task_goal)],
      ),
    );
  }

  const memorySummary = selectedTaskPackage?.memory_injection_summary ?? null;
  if (selectedTaskPackage && (!memorySummary || memorySummary.stale)) {
    attention.push(
      canvasAttention(
        `memory-packet:${selectedTaskPackage.task_package_id}`,
        "memory_packet",
        "warning",
        status,
        "任务记忆包需要关注",
        memorySummary?.stale_reasons.join("；") || "任务记忆包尚未生成；候选 / 观察不能当作正式记忆注入。",
        false,
        [memorySummary?.snapshot_id ? ref("memory_packet", memorySummary.snapshot_id, "任务包记忆快照") : ref("task_package", selectedTaskPackage.task_package_id)],
      ),
    );
  }

  for (const item of runtimeAttention) {
    attention.push(
      canvasAttention(
        `runtime:${item.attention_id}`,
        item.kind === "readback_unavailable" ? "readback_unavailable" : item.kind === "blocked_by_guard" ? "blocked" : "running",
        item.severity === "blocking" ? "blocking" : item.severity === "needs_user" ? "needs_user" : item.severity === "warning" ? "warning" : "info",
        item.kind === "readback_unavailable" ? "readback_unavailable" : item.kind === "blocked_by_guard" ? "blocked" : status,
        item.title,
        item.user_message || item.technical_summary,
        item.requires_user_action,
        [ref("runtime_attention", item.attention_id, item.title)],
      ),
    );
  }

  for (const entry of (projectBlackboard?.entries ?? []).filter(
    (item) => (item.kind === "risk" || item.kind === "permission_request") && (!selectedTask || !item.work_item_id || item.work_item_id === selectedTask.work_item_id),
  ).slice(0, 4)) {
    attention.push(
      canvasAttention(
        `blackboard:${entry.entry_id}`,
        entry.kind === "permission_request" ? "waiting_for_permission" : "blocked",
        entry.kind === "permission_request" ? "needs_user" : "warning",
        entry.kind === "permission_request" ? "waiting_for_permission" : status,
        entry.kind === "permission_request" ? "黑板权限候选" : "黑板风险",
        `${entry.title}：${entry.promotion_decision.status}`,
        entry.kind === "permission_request",
        [ref("blackboard_entry", entry.entry_id, entry.title)],
      ),
    );
  }

  return uniqueAttention(attention);
}

function canvasAttention(
  attentionId: string,
  kind: ProjectCanvasAttentionKind,
  severity: ProjectCanvasAttentionSeverity,
  status: ProjectCanvasStatus,
  title: string,
  summary: string,
  requiresUserAction: boolean,
  sourceRefs: ProjectCanvasSourceRef[],
): ProjectCanvasAttention {
  return {
    attention_id: attentionId,
    kind,
    severity,
    status,
    title,
    summary,
    requires_user_action: requiresUserAction,
    source_refs: sourceRefs,
  };
}

function uniqueAttention(items: ProjectCanvasAttention[]) {
  const seen = new Set<string>();
  return items.filter((item) => {
    if (seen.has(item.attention_id)) return false;
    seen.add(item.attention_id);
    return true;
  });
}

function statusLabel(status: ProjectCanvasStatus) {
  if (status === "empty") return "空态";
  if (status === "idle") return "空闲";
  if (status === "prepared") return "准备派发";
  if (status === "ready_to_dispatch") return "待派发";
  if (status === "running") return "执行中";
  if (status === "waiting_for_permission") return "等待权限";
  if (status === "needs_review") return "待复核";
  if (status === "ready_for_review") return "待回收";
  if (status === "needs_changes") return "需修改";
  if (status === "accepted") return "已接受";
  if (status === "failed") return "失败";
  if (status === "timed_out") return "已超时";
  if (status === "readback_unavailable") return "读回不可用";
  if (status === "blocked") return "阻断";
  return "未知";
}

function matchingRuntimeAttention(
  runtimeAttention: RuntimeSessionAttention[],
  projectWorkflow: ProjectWorkflowSummary | null,
  selectedTask: TaskDraftSummary | null,
) {
  if (!projectWorkflow) return [];
  return runtimeAttention.filter((attention) => {
    const projectMatches = !attention.project_id || attention.project_id === projectWorkflow.project_id;
    const workflowMatches = !attention.workflow_id || attention.workflow_id === projectWorkflow.workflow_id;
    const nodeMatches = !selectedTask || !attention.node_id || attention.node_id === selectedTask.current_node_id || attention.node_id === dispatchNodeIdForTask(selectedTask);
    return projectMatches && workflowMatches && nodeMatches;
  });
}

function canvasStatus(
  projectWorkflow: ProjectWorkflowSummary | null,
  selectedTask: TaskDraftSummary | null,
  runtimeAttention: RuntimeSessionAttention[],
): ProjectCanvasStatus {
  if (!projectWorkflow || !selectedTask) return "empty";
  const pendingPermission = projectWorkflow.permission_requests.some(
    (request) => request.status === "pending" && (!selectedTask || request.work_item_id === selectedTask.work_item_id),
  );
  if (pendingPermission) return "waiting_for_permission";
  const dispatchRunning = projectWorkflow.node_dispatches.some(
    (dispatch) => dispatch.state === "running" && (!selectedTask || dispatch.work_item_id === selectedTask.work_item_id),
  );
  if (dispatchRunning) return "running";
  if (projectWorkflow.execution_attempts.some((attempt) => attempt.work_item_id === selectedTask.work_item_id && attempt.state === "timed_out")) {
    return "timed_out";
  }
  if (projectWorkflow.execution_attempts.some((attempt) => attempt.work_item_id === selectedTask.work_item_id && attempt.state === "failed")) {
    return "failed";
  }
  if (runtimeAttention.some((attention) => attention.kind === "readback_unavailable" || attention.readback_boundary?.status === "readback_unavailable")) {
    return "readback_unavailable";
  }
  const selectedDispatches = projectWorkflow.node_dispatches.filter((dispatch) => dispatch.work_item_id === selectedTask.work_item_id);
  if (selectedDispatches.some((dispatch) => dispatch.state === "prepared")) return "prepared";
  if (selectedDispatches.some((dispatch) => dispatch.authorization_check?.status === "needs_review")) return "needs_review";
  if (projectWorkflow.derived_workflow?.run_check_status === "blocked") return "blocked";
  return normalizeStatus(selectedTask?.state ?? projectWorkflow.state);
}

function roleStatus(
  roleId: RoleLane["role_id"],
  workflowNodeId: string,
  selectedTask: TaskDraftSummary | null,
  dispatch: WorkflowNodeDispatchRecord | null,
  permissionRequests: WorkflowPermissionRequestRecord[],
  attempts: WorkflowExecutionAttemptRecord[],
): ProjectCanvasStatus {
  if (permissionRequests.some((request) => request.status === "pending")) return "waiting_for_permission";
  if (dispatch?.state === "running") return "running";
  if (dispatch?.state === "prepared") return "prepared";
  if (dispatch?.state === "failed") return "failed";
  if (dispatch?.authorization_check?.status === "needs_review") return "needs_review";
  if (dispatch && readbackUnavailableForDispatch(dispatch)) return "readback_unavailable";
  if (attempts.some((attempt) => attempt.state === "timed_out")) return "timed_out";
  if (attempts.some((attempt) => attempt.state === "failed") && selectedTask?.state === "failed") return "failed";
  if (!selectedTask) return "idle";
  const dispatchNodeId = dispatchNodeIdForTask(selectedTask);
  const activeByCurrentNode = selectedTask.current_node_id === workflowNodeId;
  const activeByDispatchRole = dispatchNodeId === workflowNodeId;
  if (activeByCurrentNode || activeByDispatchRole) {
    return normalizeStatus(selectedTask.state);
  }
  if (roleId === "review" && selectedTask.state === "ready_for_review") return "ready_for_review";
  if (roleId === "director" && ["accepted", "needs_changes"].includes(selectedTask.state)) return normalizeStatus(selectedTask.state);
  return "idle";
}

function normalizeStatus(state?: string | null): ProjectCanvasStatus {
  if (!state || state === "draft") return "idle";
  if (state === "prepared") return "prepared";
  if (state === "ready_to_dispatch") return "ready_to_dispatch";
  if (state === "running") return "running";
  if (state === "waiting_for_permission") return "waiting_for_permission";
  if (state === "needs_review") return "needs_review";
  if (state === "ready_for_review") return "ready_for_review";
  if (state === "needs_changes") return "needs_changes";
  if (state === "accepted" || state === "completed") return "accepted";
  if (state === "failed") return "failed";
  if (state === "timed_out") return "timed_out";
  if (state === "readback_unavailable") return "readback_unavailable";
  if (state === "blocked" || state === "retry_pending" || state === "paused" || state === "cancelled") return "blocked";
  return "unknown";
}

function toneForStatus(status: ProjectCanvasStatus): ProjectCanvasBadgeTone {
  if (status === "accepted") return "accepted";
  if (status === "running") return "running";
  if (status === "ready_to_dispatch" || status === "ready_for_review" || status === "prepared") return "ready";
  if (status === "waiting_for_permission" || status === "blocked" || status === "needs_changes" || status === "needs_review") return "blocked";
  if (status === "failed" || status === "timed_out") return "failed";
  if (status === "readback_unavailable" || status === "unknown") return "warning";
  return "neutral";
}

function edgeStatus(source?: ProjectCanvasStatus, target?: ProjectCanvasStatus): ProjectCanvasEdgeStatus {
  if (source === "waiting_for_permission" || target === "waiting_for_permission" || source === "blocked" || target === "blocked") return "blocked";
  if (source === "running" || target === "running") return "active";
  if (source === "accepted" || target === "accepted") return "completed";
  if (source === "failed" || target === "failed" || source === "timed_out" || target === "timed_out" || source === "readback_unavailable" || target === "readback_unavailable" || source === "needs_review" || target === "needs_review") return "warning";
  if (!source || !target) return "unknown";
  return "idle";
}

function findWorkflowNode(nodes: WorkflowNode[], workflowId: string, roleId: RoleLane["role_id"]) {
  const nodeId = `${workflowId}:node:${roleId}`;
  return (
    nodes.find((node) => node.workflow_node_id === nodeId) ??
    nodes.find((node) => node.assigned_role === roleId) ??
    null
  );
}

function findRoleBinding(
  bindings: WorkflowNodeSessionBinding[],
  workflowNodeId: string,
  selectedTask: TaskDraftSummary | null,
): WorkflowNodeSessionBinding | null {
  return (
    bindings.find((binding) => binding.node_id === workflowNodeId && selectedTask && binding.work_item_id === selectedTask.work_item_id && binding.lifecycle === "active") ??
    bindings.find((binding) => binding.node_id === workflowNodeId && !binding.work_item_id && binding.lifecycle === "active") ??
    null
  );
}

function findRoleDispatch(
  dispatches: WorkflowNodeDispatchRecord[],
  workflowNodeId: string,
  selectedTask: TaskDraftSummary | null,
): WorkflowNodeDispatchRecord | null {
  return (
    dispatches.find((dispatch) => dispatch.node_id === workflowNodeId && selectedTask && dispatch.work_item_id === selectedTask.work_item_id) ??
    null
  );
}

function relatedPermissions(
  requests: WorkflowPermissionRequestRecord[],
  workflowNodeId: string,
  selectedTask: TaskDraftSummary | null,
) {
  const dispatchNodeId = dispatchNodeIdForTask(selectedTask);
  const currentNodeId = selectedTask?.current_node_id ?? "";
  return requests.filter(
    (request) =>
      (!selectedTask || request.work_item_id === selectedTask.work_item_id) &&
      (workflowNodeId === dispatchNodeId || workflowNodeId === currentNodeId),
  );
}

function relatedAttempts(
  attempts: WorkflowExecutionAttemptRecord[],
  dispatch: WorkflowNodeDispatchRecord | null,
  selectedTask: TaskDraftSummary | null,
) {
  return attempts.filter(
    (attempt) =>
      (!selectedTask || attempt.work_item_id === selectedTask.work_item_id) &&
      (!dispatch || !attempt.dispatch_id || attempt.dispatch_id === dispatch.dispatch_id),
  );
}

function relatedDirectorReviews(
  reviews: WorkflowDispatchDirectorReviewRecord[],
  dispatch: WorkflowNodeDispatchRecord | null,
  selectedTask: TaskDraftSummary | null,
) {
  return reviews.filter(
    (review) =>
      (!selectedTask || review.work_item_id === selectedTask.work_item_id) &&
      (!dispatch || review.dispatch_id === dispatch.dispatch_id),
  );
}

function relatedBlackboardEntries(entries: BlackboardEntry[], workflowNodeId: string, selectedTask: TaskDraftSummary | null) {
  return entries.filter(
    (entry) =>
      (entry.workflow_node_id === workflowNodeId || !entry.workflow_node_id) &&
      (!selectedTask || !entry.work_item_id || entry.work_item_id === selectedTask.work_item_id),
  );
}

function readbackUnavailableForDispatch(dispatch: WorkflowNodeDispatchRecord) {
  if (dispatch.state === "prepared") return false;
  const warningText = dispatch.warnings.join(" ").toLowerCase();
  if (warningText.includes("readback_unavailable") || warningText.includes("readback unavailable")) return true;
  if (dispatch.state === "completed" && !dispatch.last_message_summary && dispatch.transcript_event_count == null) return true;
  return false;
}

function readbackStatusForDispatch(dispatch: WorkflowNodeDispatchRecord) {
  if (readbackUnavailableForDispatch(dispatch)) return "读回不可用；不能显示成 0 条结果";
  if (dispatch.last_message_summary) return "有摘要";
  if (dispatch.transcript_event_count != null) return `${dispatch.transcript_event_count} 个事件`;
  if (dispatch.state === "prepared") return "尚未执行；无读回";
  return "未登记";
}

function selectTaskPackage(taskPackages: TaskPackage[], selectedTask: TaskDraftSummary | null): TaskPackage | null {
  if (!taskPackages.length) return null;
  if (!selectedTask) return taskPackages[0] ?? null;
  const dispatchNodeId = dispatchNodeIdForTask(selectedTask);
  return (
    taskPackages.find((taskPackage) => taskPackage.workflow_node_id === selectedTask.current_node_id) ??
    taskPackages.find((taskPackage) => taskPackage.workflow_node_id === dispatchNodeId) ??
    taskPackages.find((taskPackage) => taskPackage.task_goal === selectedTask.title) ??
    taskPackages[0] ??
    null
  );
}

function dispatchNodeIdForTask(task: TaskDraftSummary | null): string {
  if (!task) return "";
  const assignedRole = task.assigned_role_id?.trim();
  if (assignedRole) return `${task.workflow_id}:node:${assignedRole}`;
  return task.current_node_id ?? "";
}

function roleSuffixForWorkflowNode(workflowNodeId: string) {
  if (workflowNodeId.endsWith(":node:director")) return "director";
  if (workflowNodeId.endsWith(":node:validation")) return "validation";
  if (workflowNodeId.endsWith(":node:review")) return "review";
  return "codex-dev";
}

function preferredSelectedNodeId(nodes: ProjectCanvasNode[], selectedTask: TaskDraftSummary | null) {
  const dispatchRole = roleSuffixForWorkflowNode(dispatchNodeIdForTask(selectedTask));
  const activeNode =
    nodes.find((node) => node.node_id.endsWith(`:canvas:${dispatchRole}`) && node.status !== "idle") ??
    nodes.find((node) => node.status === "waiting_for_permission") ??
    nodes.find((node) => node.status === "running") ??
    nodes.find((node) => node.node_type === "project_goal");
  return activeNode?.node_id ?? nodes[0]?.node_id ?? "";
}

function blackboardKindLabel(kind: BlackboardEntry["kind"]) {
  if (kind === "subagent_report") return "子智能体汇报";
  if (kind === "risk") return "风险";
  if (kind === "permission_request") return "权限请求";
  if (kind === "tool_summary") return "工具摘要";
  if (kind === "memory_candidate") return "记忆候选";
  if (kind === "knowledge_ref") return "知识引用";
  return kind;
}

function canvasNodeTypeLabel(type: ProjectCanvasNode["node_type"]) {
  if (type === "project_goal") return "项目目标";
  if (type === "director") return "总指导";
  if (type === "dev_line") return "开发线";
  if (type === "validation_line") return "验证线";
  if (type === "review_line") return "回收线";
  if (type === "permission_request") return "权限请求";
  if (type === "blackboard_candidate") return "黑板候选";
  if (type === "evidence_ref") return "Evidence";
  if (type === "audit_ref") return "Audit";
  return type;
}

function badge(
  badgeId: string,
  label: string,
  tone: ProjectCanvasBadgeTone,
  sourceRefs: ProjectCanvasSourceRef[],
): ProjectCanvasBadge {
  return { badge_id: badgeId, label, tone, source_refs: sourceRefs };
}

function ref(kind: ProjectCanvasSourceRef["kind"], id: string, label?: string | null): ProjectCanvasSourceRef {
  return { kind, id, label };
}

function item(
  itemId: string,
  label: string,
  value: string,
  valueKind: ProjectCanvasDetailItem["value_kind"],
  sourceRefs: ProjectCanvasSourceRef[],
): ProjectCanvasDetailItem {
  return { item_id: itemId, label, value, value_kind: valueKind, source_refs: sourceRefs };
}

function section(
  sectionId: string,
  title: string,
  kind: ProjectCanvasDetailSectionKind,
  items: ProjectCanvasDetailItem[],
  layer: ProjectCanvasDetailLayer = "project_director",
  defaultOpen = true,
): ProjectCanvasDetailSection {
  return { section_id: sectionId, title, kind, layer, default_open: defaultOpen, items };
}

function compactRefs(refs: Array<ProjectCanvasSourceRef | null | undefined>) {
  return refs.filter((sourceRef): sourceRef is ProjectCanvasSourceRef => Boolean(sourceRef?.id));
}

function compactSections(sections: Array<ProjectCanvasDetailSection | null | undefined>) {
  return sections.filter((section): section is ProjectCanvasDetailSection => Boolean(section && section.items.length));
}

function listValue(values: string[], fallback: string) {
  return values.length ? values.join("；") : fallback;
}

function slug(value: string) {
  return value
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "")
    .slice(0, 80) || "unknown";
}
