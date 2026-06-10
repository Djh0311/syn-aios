import type {
  AdapterCapability,
  AgentAdapterDescriptor,
  ProjectRecord,
  SessionRecord,
  WorkflowStateSnapshot,
} from "./types";

export type {
  AdapterCapability,
  AdapterCapabilityKind,
  AdapterCapabilityStatus,
  AgentAdapterDescriptor,
} from "./types";

export function deriveAgentAdapterDescriptors({
  sessions,
  projects = [],
  workflowState = null,
}: {
  sessions: SessionRecord[];
  projects?: ProjectRecord[];
  workflowState?: WorkflowStateSnapshot | null;
}): AgentAdapterDescriptor[] {
  const codexSessions = sessions.filter((session) => softwareKeyOf(session) === "codex");
  const readableSessions = codexSessions.filter((session) => session.rollout_exists && session.rollout_path);
  const bindings = workflowState?.project_workflows.flatMap((workflow) => workflow.node_session_bindings) ?? [];
  const activeBindings = bindings.filter((binding) => binding.adapter_id === "codex-local" && binding.lifecycle === "active");
  const dispatches = workflowState?.project_workflows.flatMap((workflow) => workflow.node_dispatches) ?? [];
  const executionControls = workflowState?.project_workflows.flatMap((workflow) => workflow.execution_controls) ?? [];
  const permissionRequests = workflowState?.project_workflows.flatMap((workflow) => workflow.permission_requests) ?? [];
  const harnessResources = projects.flatMap((project) => project.harness_resources).filter((resource) => resource.adapter_id === "codex-local");
  const hasCodexSignal = codexSessions.length > 0 || activeBindings.length > 0 || harnessResources.length > 0 || (workflowState?.counts.agent_adapters ?? 0) > 0;

  const capabilities: AdapterCapability[] = [
    capability({
      kind: "session_index_read",
      label: "会话索引读取",
      status: codexSessions.length ? "read_only" : "blocked",
      description: `${codexSessions.length} 条 Codex 会话索引`,
      boundary: "只读取已进入工作台索引的会话元数据，不读取完整会话记录。",
      evidence_refs: codexSessions.slice(0, 3).map((session) => session.thread_id),
      warnings: codexSessions.length ? [] : ["codex_session_index_empty"],
    }),
    capability({
      kind: "session_transcript_read",
      label: "会话正文只读",
      status: readableSessions.length ? "read_only" : "blocked",
      description: `${readableSessions.length} 条会话带回放记录，可在用户打开会话时读取`,
      boundary: "只读展示会话正文；不发送消息、不恢复会话、不写 Codex 状态库。",
      evidence_refs: readableSessions.slice(0, 3).map((session) => session.thread_id),
      warnings: readableSessions.length ? [] : ["readable_codex_session_missing"],
    }),
    capability({
      kind: "workflow_node_binding",
      label: "工作流节点绑定",
      status: activeBindings.length ? "requires_confirmation" : "available",
      description: `${activeBindings.length} 个活跃 Codex 节点绑定`,
      boundary: "只通过工作台确认动作写工作台自己的工作流状态；不启动 Codex。",
      evidence_refs: activeBindings.slice(0, 3).map((binding) => binding.binding_id),
      warnings: [],
    }),
    capability({
      kind: "safe_probe_dispatch",
      label: "安全测试派发",
      status: "requires_confirmation",
      description: `${dispatches.filter((dispatch) => dispatch.prompt_kind === "safe_probe").length} 条历史安全探针派发记录`,
      boundary: "高风险动作；必须用户确认；会执行 codex exec resume。本轮只声明能力，不执行。",
      evidence_refs: dispatches.filter((dispatch) => dispatch.prompt_kind === "safe_probe").slice(0, 3).map((dispatch) => dispatch.dispatch_id),
      warnings: ["declared_only_not_executed_in_this_slice"],
    }),
    capability({
      kind: "user_reviewed_dispatch",
      label: "用户审核业务派发",
      status: "requires_confirmation",
      description: `${executionControls.filter((control) => control.user_reviewed_instruction?.approval_state === "reviewed").length} 条已审核指令记录`,
      boundary: "高风险动作；必须用户确认；可能写业务路径。本轮只声明能力，不执行。",
      evidence_refs: executionControls.slice(0, 3).map((control) => control.control_id),
      warnings: ["declared_only_not_executed_in_this_slice"],
    }),
    capability({
      kind: "workflow_machine_run",
      label: "四角色工作流机器",
      status: "requires_confirmation",
      description: "现有路径支持四角色循环，但启动必须用户确认",
      boundary: "高风险动作；会调用绑定 Codex 会话。本轮只声明能力，不执行。",
      evidence_refs: activeBindings.slice(0, 4).map((binding) => binding.binding_id),
      warnings: ["declared_only_not_executed_in_this_slice"],
    }),
    capability({
      kind: "permission_decision_record",
      label: "权限结论记录",
      status: permissionRequests.length ? "requires_confirmation" : "available",
      description: `${permissionRequests.length} 条权限请求记录`,
      boundary: "只通过控制核心记录权限结论并写工作台工作流状态；不启动 Codex。",
      evidence_refs: permissionRequests.slice(0, 3).map((request) => request.request_id),
      warnings: [],
    }),
    capability({
      kind: "harness_resource_index",
      label: "运行器资源索引",
      status: harnessResources.length ? "read_only" : "blocked",
      description: `${harnessResources.length} 个 Codex 运行器资源索引`,
      boundary: "只展示索引字段；不运行运行器，不证明资源可用。",
      evidence_refs: harnessResources.slice(0, 3).map((resource) => resource.root_path),
      warnings: harnessResources.length ? [] : ["codex_harness_resource_missing"],
    }),
  ];

  const codexDescriptor: AgentAdapterDescriptor = {
    adapter_id: "codex-local",
    agent_type: "codex",
    agent_id: "codex-local",
    display_name: "Codex",
    provider: "local-codex-index",
    status: hasCodexSignal ? "available" : "not_connected",
    permission_level: "read_only",
    source_kind: "frontend_read_model",
    capabilities,
    implemented_action_kinds: [
      "reveal-rollout",
      "bind-node-session",
      "unbind-node-session",
      "execute-node-dispatch",
      "record-permission-decision",
      "run-workflow-machine",
    ],
    hidden_unimplemented_adapters: ["claude-code", "openclaw", "opencode", "opencode-like"],
    warnings: [
      "adapter_descriptor_frontend_fallback_used",
      "adapter_descriptor_is_frontend_read_model_only",
      "does_not_change_codex_execution_semantics",
      "unimplemented_adapters_hidden",
    ],
    execution_status: hasCodexSignal ? "available_with_user_confirmation" : "not_connected",
    credential_status: "not_read",
    model_access_status: "local_read_model_only",
    permission_boundary: "Codex 高风险动作仍必须用户确认；E1 未执行 codex exec 或 codex exec resume。",
    unavailable_reason: hasCodexSignal ? null : "codex_signal_missing",
    requires_user_setup: !hasCodexSignal,
  };

  return [
    codexDescriptor,
    ...plannedAgentAdapterDescriptors("frontend_read_model"),
  ];
}

function plannedAgentAdapterDescriptors(sourceKind: AgentAdapterDescriptor["source_kind"]): AgentAdapterDescriptor[] {
  return [
    plannedAgentAdapterDescriptor("claude-code", "Claude Code", "anthropic-cli-planned", sourceKind),
    plannedAgentAdapterDescriptor("openclaw", "OpenClaw", "openclaw-planned", sourceKind),
    plannedAgentAdapterDescriptor("opencode", "OpenCode", "opencode-planned", sourceKind),
    plannedAgentAdapterDescriptor("opencode-like", "OpenCode-like", "opencode-compatible-planned", sourceKind),
  ];
}

function plannedAgentAdapterDescriptor(
  adapterId: Exclude<AgentAdapterDescriptor["agent_type"], "codex">,
  displayName: string,
  provider: string,
  sourceKind: AgentAdapterDescriptor["source_kind"],
): AgentAdapterDescriptor {
  return {
    adapter_id: adapterId,
    agent_type: adapterId,
    agent_id: adapterId,
    display_name: displayName,
    provider,
    status: "planned",
    permission_level: "read_only",
    source_kind: sourceKind,
    capabilities: [],
    implemented_action_kinds: [],
    hidden_unimplemented_adapters: [],
    warnings: [
      ...(sourceKind === "frontend_read_model" ? ["adapter_descriptor_frontend_fallback_used"] : []),
      "adapter_descriptor_is_read_model_only",
      "planned_adapter_not_connected",
      "no_execution_button",
      "credential_not_configured",
      "model_access_not_verified",
    ],
    execution_status: "not_implemented",
    credential_status: "not_configured",
    model_access_status: "not_verified",
    permission_boundary: "计划中的 adapter 只有只读 descriptor；没有真实命令、会话、凭据或模型调用。",
    unavailable_reason: "planned_adapter_descriptor_only_no_runtime_connection",
    requires_user_setup: true,
  };
}

function capability(input: Omit<AdapterCapability, "capability_id">): AdapterCapability {
  return {
    capability_id: `codex-local:${input.kind}`,
    ...input,
  };
}

function softwareKeyOf(session: SessionRecord): string {
  return (session.thread_source ?? "codex").trim().toLowerCase() || "codex";
}
