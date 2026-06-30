import type {
  AgentAdapterDescriptor,
  SessionOperationDescriptor,
  SessionOperationId,
  SessionOperationRiskLevel,
  SessionOperationStatus,
} from "./types";

type SessionOperationSpec = {
  operation_id: SessionOperationId;
  label: string;
  category: string;
  codex_status: SessionOperationStatus;
  risk_level: SessionOperationRiskLevel;
  applies_to_session_state: string;
  requires_user_confirmation: boolean;
  writes_codex_home: boolean;
  writes_workbench_state: boolean;
  writes_project_files: boolean;
  reads_full_transcript: boolean;
  requires_model_access: boolean;
  requires_runtime_handle: boolean;
  audit_requirement: string;
  unavailable_reason: string;
  future_task_hint: string;
  warnings: string[];
};

const operationSpecs: SessionOperationSpec[] = [
  {
    operation_id: "new_session",
    label: "新会话预览",
    category: "interactive_control",
    codex_status: "requires_future_task",
    risk_level: "high",
    applies_to_session_state: "work_item_without_native_session",
    requires_user_confirmation: true,
    writes_codex_home: true,
    writes_workbench_state: true,
    writes_project_files: false,
    reads_full_transcript: false,
    requires_model_access: true,
    requires_runtime_handle: false,
    audit_requirement: "必须绑定 work item、提示词预览、权限信封、结构化 command plan、attempt、readback 和失败审计。",
    unavailable_reason: "H3.1 只实现新会话 request / guard / permission envelope / no-op runner；真实 codex exec 新会话未授权。",
    future_task_hint: "H3-B 需单独冻结 fixture、权限信封、真实执行范围、readback 和 /Users/yoyi/.codex 读写授权。",
    warnings: [
      "h3_1_new_session_noop_only",
      "requires_work_item_binding",
      "requires_future_authorization_task",
      "no_real_new_session_in_h3_1",
      "no_session_operation_execution_in_e2",
    ],
  },
  {
    operation_id: "send_message",
    label: "发消息",
    category: "interactive_control",
    codex_status: "requires_future_task",
    risk_level: "high",
    applies_to_session_state: "existing_readonly_session",
    requires_user_confirmation: true,
    writes_codex_home: true,
    writes_workbench_state: true,
    writes_project_files: false,
    reads_full_transcript: false,
    requires_model_access: true,
    requires_runtime_handle: false,
    audit_requirement: "必须定义提示词预览、用户确认、执行记录、readback 和失败处理审计。",
    unavailable_reason: "会话中心仍是只读历史浏览器；发送路径、权限和 readback 尚未单独定义。",
    future_task_hint: "E3 或后续任务需定义 adapter runner、用户确认、审计、写入范围和失败恢复。",
    warnings: ["requires_future_authorization_task", "no_session_operation_execution_in_e2"],
  },
  {
    operation_id: "stop",
    label: "停止",
    category: "runtime_control",
    codex_status: "blocked",
    risk_level: "high",
    applies_to_session_state: "running_session_only",
    requires_user_confirmation: true,
    writes_codex_home: false,
    writes_workbench_state: true,
    writes_project_files: false,
    reads_full_transcript: false,
    requires_model_access: false,
    requires_runtime_handle: true,
    audit_requirement: "必须有运行句柄、取消协议、幂等记录、超时和失败恢复审计。",
    unavailable_reason: "当前缺少运行进程 registry、运行句柄和取消协议。",
    future_task_hint: "后续任务需先建立运行句柄、取消协议、运行日志和失败恢复模型。",
    warnings: ["runtime_handle_missing", "no_session_operation_execution_in_e2"],
  },
  {
    operation_id: "restart",
    label: "重启",
    category: "runtime_control",
    codex_status: "blocked",
    risk_level: "high",
    applies_to_session_state: "existing_or_running_session",
    requires_user_confirmation: true,
    writes_codex_home: true,
    writes_workbench_state: true,
    writes_project_files: false,
    reads_full_transcript: false,
    requires_model_access: true,
    requires_runtime_handle: true,
    audit_requirement: "必须先定义 restart 语义、上下文来源、成本提示、运行日志和审计。",
    unavailable_reason: "restart 语义未定：新建会话、恢复旧会话或重跑任务尚未决策。",
    future_task_hint: "后续任务需明确 restart 语义、上下文来源、权限、日志和成本提示。",
    warnings: ["restart_semantics_not_defined", "no_session_operation_execution_in_e2"],
  },
  {
    operation_id: "resume",
    label: "resume",
    category: "interactive_control",
    codex_status: "requires_future_task",
    risk_level: "high",
    applies_to_session_state: "bound_or_existing_session",
    requires_user_confirmation: true,
    writes_codex_home: true,
    writes_workbench_state: true,
    writes_project_files: false,
    reads_full_transcript: false,
    requires_model_access: true,
    requires_runtime_handle: false,
    audit_requirement: "必须绑定会话校验、提示词预览、权限、超时、运行日志和 readback 审计。",
    unavailable_reason: "workflow dispatch 的受控 resume 属于项目工作流语境，不等于会话中心通用 resume。",
    future_task_hint: "后续任务需决定是否复用 workflow dispatch 或建立单独 session adapter runner。",
    warnings: [
      "workflow_dispatch_is_not_session_center_resume",
      "requires_future_authorization_task",
      "no_session_operation_execution_in_e2",
    ],
  },
  {
    operation_id: "export",
    label: "导出",
    category: "data_effect",
    codex_status: "planned",
    risk_level: "medium",
    applies_to_session_state: "readable_session",
    requires_user_confirmation: true,
    writes_codex_home: false,
    writes_workbench_state: false,
    writes_project_files: true,
    reads_full_transcript: true,
    requires_model_access: false,
    requires_runtime_handle: false,
    audit_requirement: "必须有导出范围、脱敏策略、目标位置、用户确认和审计。",
    unavailable_reason: "导出格式、脱敏范围和文件写入位置尚未定义。",
    future_task_hint: "后续任务需定义 Markdown/JSON/证据包格式、脱敏和文件写入位置。",
    warnings: ["export_redaction_policy_missing", "no_session_operation_execution_in_e2"],
  },
  {
    operation_id: "delete",
    label: "删除",
    category: "destructive_data_effect",
    codex_status: "blocked_destructive",
    risk_level: "destructive",
    applies_to_session_state: "existing_session_or_native_store",
    requires_user_confirmation: true,
    writes_codex_home: true,
    writes_workbench_state: true,
    writes_project_files: false,
    reads_full_transcript: false,
    requires_model_access: false,
    requires_runtime_handle: false,
    audit_requirement: "必须有备份、回滚、双确认、作用域、原生系统兼容和审计。",
    unavailable_reason: "破坏性操作已阻断；本阶段不删除、不移动、不归档原生会话。",
    future_task_hint: "后续任务需单独设计备份、回滚、双确认、审计和原生系统兼容。",
    warnings: ["destructive_operation_blocked", "no_session_operation_execution_in_e2"],
  },
  {
    operation_id: "favorite",
    label: "收藏",
    category: "metadata_effect",
    codex_status: "planned",
    risk_level: "low",
    applies_to_session_state: "existing_session",
    requires_user_confirmation: false,
    writes_codex_home: false,
    writes_workbench_state: true,
    writes_project_files: false,
    reads_full_transcript: false,
    requires_model_access: false,
    requires_runtime_handle: false,
    audit_requirement: "必须有工作台自有 metadata store、冲突策略和轻量审计。",
    unavailable_reason: "工作台自有 favorite metadata store 尚未实现。",
    future_task_hint: "后续任务需定义 metadata store、冲突策略、导入导出和审计。",
    warnings: ["favorite_metadata_store_missing", "no_session_operation_execution_in_e2"],
  },
];

export function deriveSessionOperationDescriptors(adapters: AgentAdapterDescriptor[]): SessionOperationDescriptor[] {
  return adapters.flatMap((adapter) => operationSpecs.map((spec) => operationForAdapter(adapter, spec)));
}

function operationForAdapter(adapter: AgentAdapterDescriptor, spec: SessionOperationSpec): SessionOperationDescriptor {
  const plannedAdapter = adapter.status === "planned" || adapter.execution_status === "not_implemented";
  const currentStatus = operationStatusForAdapter(adapter, spec);
  const warnings = [
    "session_operation_boundary_read_model_only",
    "no_session_operation_execution_in_e2",
    "no_codex_home_write_in_e2",
    ...spec.warnings,
    ...(plannedAdapter ? ["planned_adapter_operation_not_available"] : []),
  ];
  const unavailableReason = plannedAdapter
    ? `${spec.unavailable_reason}；${adapter.display_name} 仍只是 planned descriptor，没有真实命令、会话、凭据或模型访问。`
    : spec.unavailable_reason;
  const futureTaskHint = plannedAdapter
    ? `${spec.future_task_hint}；必须先完成 ${adapter.display_name} adapter 真实接入设计和凭据 / 模型只读边界确认。`
    : spec.future_task_hint;

  return {
    operation_id: spec.operation_id,
    label: spec.label,
    category: spec.category,
    current_status: currentStatus,
    risk_level: spec.risk_level,
    adapter_id: adapter.adapter_id,
    agent_type: adapter.agent_type,
    applies_to_session_state: plannedAdapter ? "planned_adapter_without_session_source" : spec.applies_to_session_state,
    requires_user_confirmation: spec.requires_user_confirmation,
    writes_codex_home: spec.writes_codex_home,
    writes_workbench_state: spec.writes_workbench_state,
    writes_project_files: spec.writes_project_files,
    reads_full_transcript: spec.reads_full_transcript,
    requires_credential: plannedAdapter && spec.operation_id !== "favorite",
    requires_model_access: spec.requires_model_access || plannedAdapter,
    requires_runtime_handle: spec.requires_runtime_handle,
    audit_requirement: spec.audit_requirement,
    unavailable_reason: unavailableReason,
    future_task_hint: futureTaskHint,
    warnings,
  };
}

function operationStatusForAdapter(adapter: AgentAdapterDescriptor, spec: SessionOperationSpec): SessionOperationStatus {
  if (adapter.adapter_id === "codex-local") return spec.codex_status;
  if (spec.codex_status === "blocked_destructive") return "blocked_destructive";
  if (spec.operation_id === "export" || spec.operation_id === "favorite") return "planned";
  return "blocked";
}
