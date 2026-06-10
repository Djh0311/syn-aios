import type {
  AgentAdapterDescriptor,
  ProviderAvailabilitySummary,
  SessionOperationDescriptor,
} from "./types";

export type {
  CostRiskStatus,
  CredentialBoundaryStatus,
  ExternalCallStatus,
  ModelAvailabilityStatus,
  ProviderAvailabilityStatus,
  ProviderAvailabilitySummary,
} from "./types";

export function deriveProviderAvailabilitySummaries(
  adapters: AgentAdapterDescriptor[],
  sessionOperations: SessionOperationDescriptor[] = [],
): ProviderAvailabilitySummary[] {
  return adapters.map((adapter) => providerAvailabilityForAdapter(adapter, sessionOperations));
}

function providerAvailabilityForAdapter(
  adapter: AgentAdapterDescriptor,
  sessionOperations: SessionOperationDescriptor[],
): ProviderAvailabilitySummary {
  const plannedAdapter = adapter.status === "planned" || adapter.execution_status === "not_implemented";
  const operationsNeedFutureTask = sessionOperations.some(
    (operation) =>
      operation.adapter_id === adapter.adapter_id &&
      ["requires_future_task", "blocked", "blocked_destructive", "planned"].includes(operation.current_status),
  );
  const warnings = [
    "provider_availability_read_model_only",
    "credential_secret_not_read",
    "model_not_verified",
    "cost_not_estimated",
    "provider_availability_not_project_authorization",
    "no_external_provider_call_in_e3",
    ...(operationsNeedFutureTask ? ["session_operation_requires_future_task"] : []),
  ];

  if (adapter.adapter_id === "codex-local") {
    return {
      adapter_id: adapter.adapter_id,
      provider_id: "local-codex-cli",
      provider_label: "Codex 本地 CLI",
      provider_kind: "local_cli",
      adapter_status: adapter.status,
      availability_status: adapter.status === "available" ? "available_readonly" : adapter.status === "not_connected" ? "not_connected" : "unknown",
      credential_status: "not_required_by_workbench",
      model_status: "local_cli_managed",
      external_call_status: "not_needed_for_readonly",
      cost_risk_status: "unknown",
      user_visible_reason: "Codex 由本地 CLI 管理；工作台只读取索引和边界状态，不读取凭据、不验证模型、不发起 provider 调用。",
      safe_to_display: true,
      requires_user_configuration: adapter.requires_user_setup,
      requires_future_task: operationsNeedFutureTask,
      warnings,
    };
  }

  return {
    adapter_id: adapter.adapter_id,
    provider_id: adapter.provider,
    provider_label: adapter.display_name,
    provider_kind: providerKindForAdapter(adapter.adapter_id),
    adapter_status: adapter.status,
    availability_status: plannedAdapter ? "planned" : "unknown",
    credential_status: plannedAdapter ? "credential_missing" : "unknown",
    model_status: plannedAdapter ? "model_unverified" : "unknown",
    external_call_status: plannedAdapter ? "external_call_blocked" : "requires_future_authorization",
    cost_risk_status: plannedAdapter ? "blocked_until_authorized" : "unknown",
    user_visible_reason: `${adapter.display_name} 仍是 planned descriptor；没有真实命令、会话、凭据或模型访问，外发调用已阻断。`,
    safe_to_display: true,
    requires_user_configuration: true,
    requires_future_task: true,
    warnings: [
      ...warnings,
      ...(plannedAdapter ? ["planned_adapter_not_connected", "external_call_blocked"] : []),
    ],
  };
}

function providerKindForAdapter(adapterId: string): string {
  if (adapterId === "claude-code") return "external_cli_planned";
  if (adapterId === "openclaw") return "external_agent_planned";
  if (adapterId === "opencode") return "external_cli_planned";
  if (adapterId === "opencode-like") return "compatible_adapter_planned";
  return "unknown";
}
