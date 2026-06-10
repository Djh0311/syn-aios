import type {
  AgentAdapterDescriptor,
  ProviderAvailabilitySummary,
  SessionContinuationGuardResult,
  SessionContinuationPreview,
  SessionContinuationRequest,
  SessionOperationDescriptor,
  WorkflowStateSnapshot,
} from "./types";

type DeriveSessionContinuationInput = {
  adapterDescriptors: AgentAdapterDescriptor[];
  sessionOperationDescriptors: SessionOperationDescriptor[];
  providerAvailabilitySummaries: ProviderAvailabilitySummary[];
  workflowState?: WorkflowStateSnapshot | null;
};

type BindingTuple = {
  workflow: WorkflowStateSnapshot["project_workflows"][number];
  binding: WorkflowStateSnapshot["project_workflows"][number]["node_session_bindings"][number];
};

export function deriveSessionContinuationPreviews({
  adapterDescriptors,
  sessionOperationDescriptors,
  providerAvailabilitySummaries,
  workflowState,
}: DeriveSessionContinuationInput): SessionContinuationPreview[] {
  const previews: SessionContinuationPreview[] = [];
  for (const adapter of adapterDescriptors) {
    const operations = sessionOperationDescriptors.filter(
      (operation) =>
        operation.adapter_id === adapter.adapter_id &&
        (operation.operation_id === "new_session" || operation.operation_id === "send_message" || operation.operation_id === "resume"),
    );
    const providerSummary = providerAvailabilitySummaries.find((summary) => summary.adapter_id === adapter.adapter_id) ?? null;
    const activeBindings = activeSessionBindingsForAdapter(workflowState, adapter.adapter_id);
    for (const operation of operations) {
      if (adapter.adapter_id === "codex-local" && activeBindings.length) {
        for (const activeBinding of activeBindings) {
          previews.push(sessionContinuationPreviewForBinding(adapter, operation, providerSummary, activeBinding));
        }
      } else {
        previews.push(sessionContinuationPreviewForBinding(adapter, operation, providerSummary, null));
      }
    }
  }
  return previews;
}

export function inspectSessionContinuationGuard(
  request: SessionContinuationRequest,
  adapter?: AgentAdapterDescriptor | null,
  operation?: SessionOperationDescriptor | null,
  providerSummary?: ProviderAvailabilitySummary | null,
): SessionContinuationGuardResult {
  let blocked = false;
  let requiresFutureTask = false;
  const reasons = ["e4_preview_only_no_execution"];
  const requiredFixes: string[] = [];
  const warnings = ["session_continuation_preview_only", "no_prompt_sent_in_e4", "no_codex_home_write_in_e4"];

  if (!adapter) {
    blocked = true;
    reasons.push("adapter_descriptor_missing");
    requiredFixes.push("必须先选择已登记适配器描述。");
  } else if (adapter.adapter_id !== "codex-local" || adapter.execution_status === "not_implemented" || adapter.status === "planned") {
    blocked = true;
    reasons.push(`planned_adapter_blocked:${adapter.adapter_id}`);
    requiredFixes.push("先完成该适配器的真实接入、凭据边界和模型验证任务。");
    warnings.push("planned_adapter_blocked");
  }

  if (!operation) {
    blocked = true;
    reasons.push("session_operation_descriptor_missing");
    requiredFixes.push("必须先有 E2 会话操作边界描述。");
  } else if (
    operation.adapter_id !== request.adapter_id ||
    (operation.operation_id !== "new_session" && operation.operation_id !== "send_message" && operation.operation_id !== "resume")
  ) {
    blocked = true;
    reasons.push(`operation_not_allowed_in_e4:${operation.operation_id}`);
    requiredFixes.push("E4/H3.1 只允许新会话、发消息、恢复的预览协议。");
  }

  if (!request.project_id?.trim()) {
    blocked = true;
    reasons.push("missing_project_binding");
    requiredFixes.push("必须绑定项目，不能用自由会话绕过项目上下文。");
  }
  if (!request.project_root?.trim()) {
    blocked = true;
    reasons.push("missing_project_root");
    requiredFixes.push("必须提供项目根目录，才能判断工作目录和允许写入根目录。");
  }
  if (!request.workflow_id?.trim()) {
    blocked = true;
    reasons.push("missing_workflow_binding");
    requiredFixes.push("必须绑定工作流。");
  }
  if (!request.node_id?.trim()) {
    blocked = true;
    reasons.push("missing_node_binding");
    requiredFixes.push("必须绑定工作流节点。");
  }
  if (request.operation_id !== "new_session" && !request.session_id?.trim()) {
    blocked = true;
    reasons.push("missing_session_binding");
    requiredFixes.push("发消息 / 恢复必须绑定目标会话。");
  }
  if (request.operation_id === "new_session" && !request.work_item_id?.trim()) {
    blocked = true;
    reasons.push("missing_work_item_binding");
    requiredFixes.push("新会话必须绑定工作项，不能创建自由会话。");
  }
  if (request.operation_id === "new_session" && !request.session_id?.trim()) {
    warnings.push("new_session_does_not_require_existing_session");
  }
  if (!request.prompt_summary.trim()) {
    blocked = true;
    reasons.push("prompt_summary_missing");
    requiredFixes.push("必须提供用户可理解的提示词摘要，不能展示空预览。");
  }
  if (request.readback_strategy !== "required") {
    blocked = true;
    reasons.push("readback_strategy_required");
    requiredFixes.push("必须定义读回策略，不能把读回失败伪装成 0 条结果。");
    warnings.push("readback_strategy_required");
  }

  const targetCwd = request.target_cwd ?? "";
  const projectRoot = request.project_root ?? "";
  if (!targetCwd.trim()) {
    blocked = true;
    reasons.push("target_cwd_missing");
    requiredFixes.push("必须提供 target cwd。");
  } else if (sensitivePathLike(targetCwd)) {
    blocked = true;
    reasons.push("sensitive_path_blocked:target_cwd");
    requiredFixes.push("target cwd 命中敏感路径，必须更换到项目授权范围。");
    warnings.push("sensitive_path_blocked");
  } else if (
    projectRoot &&
    !pathWithinScope(targetCwd, projectRoot) &&
    !request.allowed_write_roots.some((root) => pathWithinScope(targetCwd, root))
  ) {
    blocked = true;
    reasons.push("cwd_out_of_scope_blocked");
    requiredFixes.push("target cwd 必须在 project root 或 allowed write roots 内。");
    warnings.push("cwd_out_of_scope_blocked");
  }

  if (request.allowed_write_roots.some(sensitivePathLike)) {
    blocked = true;
    reasons.push("sensitive_path_blocked:allowed_write_roots");
    requiredFixes.push("allowed write roots 不能包含 .codex、.env、auth/token/secret/keychain 等路径。");
    warnings.push("sensitive_path_blocked");
  }

  if (
    providerSummary &&
    (providerSummary.external_call_status === "external_call_blocked" ||
      providerSummary.credential_status === "credential_missing" ||
      providerSummary.availability_status === "planned")
  ) {
    requiresFutureTask = true;
    reasons.push(`provider_availability_requires_future_task:${providerSummary.adapter_id}`);
    requiredFixes.push("供应方可用性只是守卫输入；计划中、缺凭据、外发阻断都需要后续任务。");
    warnings.push("provider_availability_not_execution_authorization");
  }

  const userConfirmed = request.user_confirmation_state === "confirmed";
  let status: SessionContinuationGuardResult["status"];
  if (blocked) status = "blocked";
  else if (requiresFutureTask) status = "requires_future_task";
  else if (userConfirmed) status = "allowed_preview";
  else {
    status = "needs_user_confirmation";
    reasons.push("user_confirmation_required_before_execution");
    requiredFixes.push("E5 真实执行前必须经过用户确认；E4 只允许预览。");
  }

  return {
    status,
    severity: blocked ? "high" : requiresFutureTask ? "medium" : userConfirmed ? "low" : "medium",
    blocks_execution: true,
    allows_preview: status === "allowed_preview" || status === "needs_user_confirmation",
    requires_user_confirmation: !userConfirmed && status === "needs_user_confirmation",
    reasons: uniqueSorted(reasons),
    required_fixes: uniqueSorted(requiredFixes),
    warnings: uniqueSorted(warnings),
  };
}

function activeSessionBindingsForAdapter(workflowState: WorkflowStateSnapshot | null | undefined, adapterId: string): BindingTuple[] {
  if (!workflowState) return [];
  return workflowState.project_workflows.flatMap((workflow) =>
    workflow.node_session_bindings
      .filter((binding) => binding.adapter_id === adapterId && binding.lifecycle === "active")
      .map((binding) => ({ workflow, binding })),
  );
}

function sessionContinuationPreviewForBinding(
  adapter: AgentAdapterDescriptor,
  operation: SessionOperationDescriptor,
  providerSummary: ProviderAvailabilitySummary | null,
  activeBinding: BindingTuple | null,
): SessionContinuationPreview {
  const projectRoot = activeBinding?.workflow.project_root ?? null;
  const allowedWriteRoots = projectRoot ? [projectRoot] : [];
  const sessionId = operation.operation_id === "new_session" ? null : activeBinding?.binding.native_thread_id ?? null;
  const workItemId = activeBinding?.binding.work_item_id ?? null;
  const request: SessionContinuationRequest = {
    adapter_id: adapter.adapter_id,
    operation_id: operation.operation_id,
    project_id: activeBinding?.binding.project_id ?? activeBinding?.workflow.project_id ?? null,
    project_root: projectRoot,
    workflow_id: activeBinding?.binding.workflow_id ?? activeBinding?.workflow.workflow_id ?? null,
    node_id: activeBinding?.binding.node_id ?? null,
    session_id: sessionId,
    work_item_id: workItemId,
    target_cwd: projectRoot,
    allowed_write_roots: allowedWriteRoots,
    sandbox: "workspace-write-preview-only",
    prompt_source_kind: continuationPromptSourceKind(operation.operation_id),
    prompt_summary: continuationPromptSummary(operation.operation_id, activeBinding?.binding ?? null),
    readback_strategy: activeBinding ? "required" : "not_defined",
    requested_by: operation.operation_id === "new_session" ? "workbench_h3_1_new_session_preview" : "workbench_e4_preview",
    user_confirmation_state: "missing",
  };
  const guardResult = inspectSessionContinuationGuard(request, adapter, operation, providerSummary);
  const previewId = `session-continuation-preview:${adapter.adapter_id}:${operation.operation_id}:${activeBinding?.binding.binding_id ?? "unbound"}`;
  const warnings = uniqueSorted([
    "session_continuation_preview_only",
    "no_prompt_sent_in_e4",
    "no_codex_home_write_in_e4",
    "h3_1_no_real_new_session",
    "user_confirmation_required_before_execution",
    ...guardResult.warnings,
  ]);
  const readbackExpectedSources =
    operation.operation_id === "new_session"
      ? ["future_h3_new_session_last_message", "future_h3_attempt_audit"]
      : ["target_session_rollout_readback", "future_e5_attempt_audit"];
  const readbackUnavailableBehavior =
    operation.operation_id === "new_session"
      ? "H3.1 只定义新会话读回预期；真实读回必须等 H3-B 真实执行后从受控末条消息 / 审计读取，不能伪装成 0 条结果。"
      : "E4 只定义读回预期；真实读回失败必须在 E5 / G1 显示为不可用，不能伪装成 0 条结果。";
  const failureUserVisibleBehavior =
    operation.operation_id === "new_session"
      ? "本轮只展示 H3.1 新会话守卫和权限预览；真实失败、超时、取消和重试边界进入 H3-B / G1。"
      : "本轮只展示守卫和权限预览；真实失败、超时、取消和重试边界进入 E5 / E6 / G1。";
  const futureAuditRequirement =
    operation.operation_id === "new_session"
      ? "H3-B 真实新会话前必须写用户确认、尝试记录、运行日志 / 会话继续记录、读回和失败审计。"
      : "E5 真实发送 / 恢复前必须写用户确认、尝试记录、派发 / 会话继续记录、读回和失败审计。";

  return {
    preview_id: previewId,
    adapter_id: adapter.adapter_id,
    operation_id: operation.operation_id,
    target_session_id: sessionId,
    target_session_title: operation.operation_id === "new_session" ? null : activeBinding?.binding.session_title ?? null,
    project_id: request.project_id,
    project_root: projectRoot,
    workflow_id: request.workflow_id,
    node_id: request.node_id,
    binding_id: activeBinding?.binding.binding_id ?? null,
    work_item_id: workItemId,
    target_cwd: request.target_cwd,
    allowed_write_roots_summary: allowedWriteRoots,
    sandbox_summary: request.sandbox,
    prompt_source_kind: request.prompt_source_kind,
    prompt_summary: request.prompt_summary,
    readback_expectation: {
      strategy: request.readback_strategy,
      required: request.readback_strategy === "required",
      expected_sources: readbackExpectedSources,
      unavailable_behavior: readbackUnavailableBehavior,
      warnings: ["readback_expectation_only_no_readback_in_e4"],
    },
    failure_handling: {
      timeout_policy: "deferred_to_e5_runtime_boundary",
      retry_policy: "no_retry_in_e4",
      failure_record: "no_attempt_or_runtime_log_written_in_e4",
      user_visible_behavior: failureUserVisibleBehavior,
      warnings: ["failure_boundary_preview_only"],
    },
    audit_impact: {
      impact_kind: "preview_only_no_execution",
      writes_attempt_in_e4: false,
      writes_dispatch_in_e4: false,
      writes_readback_in_e4: false,
      future_audit_requirement: futureAuditRequirement,
      warnings: ["would_require_attempt_audit_in_e5"],
    },
    provider_availability_summary: providerSummary,
    guard_result: guardResult,
    request,
    user_visible_warnings: warnings,
  };
}

function continuationPromptSourceKind(operationId: string): string {
  if (operationId === "new_session") return "h3_new_session_task_package";
  if (operationId === "send_message") return "workflow_followup";
  if (operationId === "resume") return "task_package_summary";
  return "not_allowed";
}

function continuationPromptSummary(
  operationId: string,
  binding: BindingTuple["binding"] | null,
): string {
  const target = binding
    ? `${binding.project_id} / ${binding.node_id} / ${binding.work_item_id ?? "missing-work-item"} / ${binding.native_thread_id}`
    : "未绑定 project / workflow / node / work item / session";
  if (operationId === "new_session") {
    return `H3.1 新会话预览：为已绑定工作项准备独立 Codex 新会话请求；目标 ${target}。不创建真实会话，不发送提示词，不写 Codex 原生状态。`;
  }
  if (operationId === "send_message") {
    return `E4 只读预览：继续已绑定会话的下一轮项目意图；目标 ${target}。不显示原始提示词，不发送消息。`;
  }
  if (operationId === "resume") {
    return `E4 只读预览：恢复只作为会话继续协议检查；目标 ${target}。工作流派发经验仅作边界参考，本轮不启动派发。`;
  }
  return "E4 只读预览：该操作不属于会话继续范围。";
}

function sensitivePathLike(path: string): boolean {
  const normalized = path.toLowerCase();
  return (
    normalized.includes("/.codex") ||
    normalized.includes("\\.codex") ||
    normalized.endsWith(".codex") ||
    normalized.includes("/.ssh") ||
    normalized.includes("\\.ssh") ||
    normalized.includes(".env") ||
    normalized.includes("keychain") ||
    normalized.includes("oauth") ||
    normalized.includes("provider credential") ||
    normalized.includes("token") ||
    normalized.includes("secret") ||
    normalized.includes("/auth") ||
    normalized.includes("\\auth")
  );
}

function pathWithinScope(path: string, root: string): boolean {
  const cleanPath = path.replace(/\/+$/, "");
  const cleanRoot = root.replace(/\/+$/, "");
  return !!cleanRoot && (cleanPath === cleanRoot || cleanPath.startsWith(`${cleanRoot}/`));
}

function uniqueSorted(values: string[]): string[] {
  return Array.from(new Set(values)).sort();
}
