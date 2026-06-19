import { pathTail } from "../../lib/format";
import type {
  AdapterCapabilityStatus,
  AgentAdapterDescriptor,
  ProviderAvailabilitySummary,
  RealExecutionProductCommandPreview,
  RealExecutionProductCommandReadModel,
  SessionContinuationPreview,
  SessionContinuationStoreV1,
  SessionOperationDescriptor,
} from "../../lib/types";

export const J1_DEFAULT_DENIED_PATHS = [
  "secret",
  "token",
  ".env",
  "keychain",
  "OAuth",
  "provider credential",
  "full transcript",
  "rollout",
];

export function codexControlPreviewLabel(preview: RealExecutionProductCommandPreview) {
  if (preview.blocked_reasons.length) return "暂缓 / 阻断";
  if (preview.readiness.status === "ready_for_pcr3_decision_preview_only") return "可进入用户确认";
  return preview.readiness.status;
}

export function codexControlReasonLabel(reason: string) {
  const labels: Record<string, string> = {
    codex_control_new_session_deferred_in_j1a: "新会话真实启动留到后续执行点授权",
    codex_control_resume_requires_target_session: "恢复已有会话需要选择目标 session",
    codex_control_prompt_hash_invalid: "任务正文摘要校验未生成",
    codex_control_sensitive_denied_paths_missing: "敏感路径拒绝清单不完整",
    codex_control_allowed_write_roots_boundary_missing: "需要项目根作为执行边界根",
  };
  return labels[reason] ?? reason;
}

export type AgentUserFacingError = {
  title: string;
  nextStep: string;
  raw: string;
};

export function manualRelayReasonLabel(reason: string) {
  const labels: Record<string, string> = {
    manual_relay_denied_material_requested: "这条消息像是在索取凭据、密钥、完整记录或内部提示材料",
    manual_relay_payload_must_be_exact_original: "发送正文被改写过，Manual relay 只允许原文一次发送",
    manual_relay_policy_must_be_manual_once_without_auto_chain: "发送策略不是一次一发",
    manual_relay_new_session_must_not_bind_target_session: "新建对话不能同时绑定旧会话",
    manual_relay_existing_session_requires_target_session: "继续对话需要绑定目标会话",
    manual_relay_command_plan_missing: "没有生成可执行的发送计划",
    manual_relay_duplicate_running_attempt: "同一个目标已有运行中的发送",
    manual_relay_gui_direct_test_process_mode_required: "测试模式缺少进程模式",
    manual_relay_gui_direct_test_must_not_use_real_codex: "测试入口不能启动真实 Codex",
    manual_relay_confirmation_already_consumed: "这次确认已经被使用过",
  };
  if (reason.startsWith("codex_local_guard:")) {
    return `Codex 本地执行边界阻断：${codexControlReasonLabel(reason.slice("codex_local_guard:".length))}`;
  }
  return labels[reason] ?? reason;
}

export function userFacingAgentError(rawError: string): AgentUserFacingError {
  const raw = rawError.trim();
  if (!raw) {
    return {
      title: "没发出去：没有拿到错误详情",
      nextStep: "重新发送一次；如果仍失败，展开“开发者详情”查看原始诊断。",
      raw,
    };
  }
  if (raw.startsWith("manual_relay_guard_blocked:")) {
    const reasons = raw
      .slice("manual_relay_guard_blocked:".length)
      .split(",")
      .map((reason) => reason.trim())
      .filter(Boolean);
    const humanReasons = reasons.map(manualRelayReasonLabel);
    const title =
      reasons.includes("manual_relay_denied_material_requested")
        ? "没发出去：这条消息像是在索取敏感材料。"
        : `没发出去：${humanReasons[0] ?? "安全边界阻断了这次发送"}。`;
    const nextStep =
      reasons.includes("manual_relay_denied_material_requested")
        ? "删掉凭据、token、.codex、完整 transcript 或内部 prompt 相关请求后再发送；原始 reason 在“开发者详情”里。"
        : "按提示修正目标或正文后再发送；原始 reason 在“开发者详情”里。";
    return { title, nextStep, raw };
  }
  if (raw.includes("状态刷新连续失败")) {
    return {
      title: "状态刷新暂停了，发送进程可能还在跑。",
      nextStep: "点“恢复轮询”继续刷新，或点 Stop 停止这次运行；原始错误在“开发者详情”里。",
      raw,
    };
  }
  if (raw.includes("状态刷新失败")) {
    return {
      title: "状态刷新暂时失败，正在重试。",
      nextStep: "可以先等自动重试；如果多次失败，展开“开发者详情”查看原始错误。",
      raw,
    };
  }
  if (raw.includes("Codex 运行超过 10 分钟")) {
    return {
      title: "Codex 运行超过 10 分钟。",
      nextStep: "可以重新发送，或展开“开发者详情”确认停止/回执状态。",
      raw,
    };
  }
  return {
    title: "没发出去：发送流程返回了错误。",
    nextStep: "展开“开发者详情”查看原始原因；确认目标和正文后再试一次。",
    raw,
  };
}

export function productCommandStatusLabel(readModel: RealExecutionProductCommandReadModel | null | undefined) {
  if (!readModel || !readModel.store_available || readModel.command_count === 0) return "无统一执行命令";
  if (readModel.pending_decision_count > 0) return "等待确认";
  if (readModel.blocked_attempt_count > 0) return "已阻断";
  if (readModel.running_attempt_count > 0) return "受控记录可见";
  return attemptStatusLabel(readModel.last_attempt_status) || "准备执行";
}

export function productEntryStatusLabel(value?: string | null) {
  if (!value) return "未知 / 不可用";
  const labels: Record<string, string> = {
    readiness_only_pcr1_no_execute: "只读准备态，不执行",
    legacy_sealed_blocked_not_product_command: "legacy 已封口",
    internal_runner_blocked_until_unified_execute_and_level_b: "内部 runner 等 Level B",
  };
  return labels[value] ?? value;
}

export function automationStatusLabel(status?: string | null) {
  if (!status) return "未记录";
  if (status === "phase_a_closed_loop_recorded") return "Level A 闭环已记录";
  if (status === "blocked") return "已阻断";
  return status;
}

export function automationRunUnitLabel(kind: string) {
  const labels: Record<string, string> = {
    director_plan: "主管计划",
    developer_execution: "开发线",
    verifier_check: "验证线",
    collector_summary: "回收线",
    director_final_review: "主管复核",
  };
  return labels[kind] ?? kind;
}

export function automationUnitStatusLabel(status: string) {
  if (status === "planned") return "已计划";
  if (status === "waiting_user") return "等待确认";
  if (status === "completed") return "已记录";
  if (status === "needs_review") return "待复核";
  if (status === "blocked_by_guard") return "已阻断";
  if (status === "readback_unavailable") return "读回不可用";
  if (status === "codex_state_error") return "Codex 状态不可写";
  return runtimeAttentionLabel(status) || status;
}

export function latestAttemptByContinuation(attempts: SessionContinuationStoreV1["attempts"]) {
  const map = new Map<string, SessionContinuationStoreV1["attempts"][number]>();
  for (const attempt of attempts) {
    map.set(attempt.continuation_id, attempt);
  }
  return map;
}

export function yesNoLabel(value: boolean) {
  return value ? "是" : "否";
}

export function attemptStatusLabel(status?: string | null) {
  if (!status) return "未见尝试";
  if (status === "preview_confirmed") return "预览已确认";
  if (status === "queued") return "已排队";
  if (status === "waiting_permission") return "等待权限";
  if (status === "running_stub") return "桩执行运行中";
  if (status === "succeeded_stub") return "桩验收通过";
  if (status === "failed_stub") return "桩执行失败";
  if (status === "succeeded") return "成功";
  if (status === "failed") return "失败";
  if (status === "timed_out") return "超时";
  if (status === "codex_state_error") return "Codex 状态不可写";
  if (status === "blocked") return "阻断";
  return status;
}

export function readbackStatusLabel(status?: string | null) {
  if (!status) return "未登记";
  if (status === "not_attempted_stub") return "桩执行未读回";
  if (status === "readback_unavailable") return "读回不可用";
  if (status === "readback_failed") return "读回失败";
  if (status === "readback_succeeded") return "读回成功";
  if (status === "not_attempted") return "未读回";
  if (status === "timed_out") return "超时";
  if (status === "codex_state_error") return "Codex 状态不可写";
  if (status === "blocked") return "阻断";
  return status;
}

export function guardSeverityLabel(severity: string) {
  if (severity === "info") return "提示";
  if (severity === "warning") return "警告";
  if (severity === "blocking") return "阻断";
  if (severity === "needs_user") return "需要用户";
  return severity;
}

export function readbackStrategyLabel(strategy: string) {
  if (strategy === "required") return "必需";
  if (strategy === "none") return "不读回";
  if (strategy === "stub") return "桩读回";
  if (strategy === "manual") return "手动读回";
  if (strategy === "structured") return "结构化读回";
  if (strategy === "last_message") return "末条消息";
  if (strategy === "runtime_log") return "运行日志";
  return strategy;
}

export function retryPolicyLabel(policy: string) {
  if (policy === "none") return "不重试";
  if (policy === "manual_only") return "仅手动";
  if (policy === "blocked") return "阻断";
  if (policy === "future_task") return "后续任务";
  return policy;
}

export function auditImpactLabel(impact: string) {
  if (impact === "preview_only_no_execution") return "仅预览不执行";
  if (impact === "none") return "无";
  if (impact === "preview_only") return "仅预览";
  if (impact === "audit_ref") return "审计引用";
  if (impact === "runtime_ref") return "运行引用";
  if (impact === "write_attempt") return "写入尝试记录";
  return impact;
}

export function controlledContinuationTone(status: string): "candidate" | "warning" | "unknown" {
  if (status === "succeeded_stub") return "candidate";
  if (status === "failed_stub" || status === "timed_out" || status === "blocked") return "warning";
  return "unknown";
}

export function controlledContinuationLabel(status: string) {
  if (status === "preview_confirmed") return "预览已确认";
  if (status === "queued") return "已排队";
  if (status === "waiting_permission") return "等待权限";
  if (status === "running_stub") return "桩执行运行中";
  if (status === "succeeded_stub") return "桩验收通过";
  if (status === "failed_stub") return "桩执行失败";
  if (status === "readback_unavailable") return "读回不可用";
  if (status === "timed_out") return "超时";
  if (status === "codex_state_error") return "Codex 状态不可写";
  if (status === "blocked") return "阻断";
  return status;
}

export function adapterContractStatusLabel(status: string) {
  if (status === "ready_for_controlled_adapter_contract") return "契约材料齐备";
  if (status === "blocked_or_reserved_contract") return "阻断或预留";
  return status;
}

export function h2ReadinessStatusLabel(status: string) {
  if (status === "blocked_waiting_authorization") return "等待授权矩阵";
  if (status === "ready_for_explicit_authorization") return "字段齐备但仍需明确确认";
  return status;
}

export function h2ReadinessItemTone(status: string): "candidate" | "warning" | "unknown" {
  if (status === "confirmed") return "candidate";
  if (status === "blocked") return "warning";
  return "unknown";
}

export function h2ReadinessItemStatusLabel(status: string) {
  if (status === "confirmed") return "已确认";
  if (status === "missing") return "待确认";
  if (status === "recommended_default") return "推荐默认";
  if (status === "blocked") return "阻断";
  return status;
}

export function h2DecisionStatusLabel(status: string) {
  if (status === "ready_for_final_approval") return "材料齐备，仍需最终批准";
  if (status === "ready_but_not_authorized") return "字段齐备但未授权";
  if (status === "blocked_waiting_target_session") return "缺目标会话";
  if (status === "blocked_waiting_fixture") return "缺测试样例";
  if (status === "blocked_waiting_permission_envelope") return "缺权限包";
  if (status === "blocked_waiting_allowed_write_roots") return "缺允许写入根目录";
  if (status === "blocked_waiting_prompt_envelope") return "缺提示词包";
  if (status === "blocked_waiting_codex_home_scope") return "缺 .codex 范围";
  if (status === "blocked_waiting_readback_plan") return "缺读回计划";
  if (status === "blocked_waiting_runtime_log") return "缺运行日志";
  if (status === "blocked_waiting_audit") return "缺审计";
  if (status === "blocked_waiting_rollback") return "缺回滚";
  if (status === "blocked_by_guard") return "边界保护阻断";
  if (status === "blocked_by_duplicate_attempt") return "重复尝试阻断";
  if (status === "blocked_by_diagnostics") return "诊断阻断";
  return status;
}

export function h2DecisionCheckTone(status: string, blocksFinalApproval: boolean): "candidate" | "warning" | "unknown" {
  if (blocksFinalApproval || status === "blocked" || status === "missing") return "warning";
  if (status === "ready") return "candidate";
  return "unknown";
}

export function h2DecisionCheckStatusLabel(status: string, blocksFinalApproval: boolean) {
  if (blocksFinalApproval) return "阻断最终批准";
  if (status === "ready") return "已具备";
  if (status === "preview") return "预览";
  if (status === "missing") return "待确认";
  if (status === "blocked") return "阻断";
  return status;
}

export function runtimeAttentionTone(status: string): "candidate" | "warning" | "unknown" {
  if (status === "blocked_by_guard" || status === "failed_stub" || status === "timed_out" || status === "readback_failed" || status === "codex_state_error") {
    return "warning";
  }
  if (status === "readback_unavailable" || status === "waiting_permission" || status === "waiting_level_b_authorization") {
    return "unknown";
  }
  return "candidate";
}

export function runtimeAttentionLabel(status: string) {
  if (status === "waiting_permission") return "等待确认";
  if (status === "waiting_level_b_authorization") return "等待 Level B";
  if (status === "running_stub") return "桩执行运行中";
  if (status === "succeeded_stub") return "桩执行完成";
  if (status === "failed_stub") return "桩执行失败";
  if (status === "timed_out") return "超时";
  if (status === "readback_failed") return "读回失败";
  if (status === "readback_unavailable") return "读回不可用";
  if (status === "codex_state_error") return "Codex 状态不可写";
  if (status === "blocked_by_guard") return "边界保护阻断";
  if (status === "needs_user") return "需要用户";
  return status;
}

export function groupSessionContinuationPreviewsByAdapter(previews: SessionContinuationPreview[]) {
  const groups = new Map<string, SessionContinuationPreview[]>();
  for (const preview of previews) {
    const existing = groups.get(preview.adapter_id) ?? [];
    existing.push(preview);
    groups.set(preview.adapter_id, existing);
  }
  return Array.from(groups.entries())
    .map(([adapterId, groupedPreviews]) => ({
      adapterId,
      previews: groupedPreviews.sort(
        (a, b) => sessionOperationOrder(a.operation_id as SessionOperationDescriptor["operation_id"]) - sessionOperationOrder(b.operation_id as SessionOperationDescriptor["operation_id"]),
      ),
    }))
    .sort((a, b) => (a.adapterId === "codex-local" ? -1 : b.adapterId === "codex-local" ? 1 : a.adapterId.localeCompare(b.adapterId)));
}

export function sessionContinuationOperationLabel(operationId: string) {
  if (operationId === "new_session") return "新会话预览";
  if (operationId === "send_message") return "发消息预览";
  if (operationId === "resume") return "恢复预览";
  if (operationId === "stop") return "停止预览";
  if (operationId === "restart") return "重启预览";
  if (operationId === "export") return "导出预览";
  if (operationId === "delete") return "删除预览";
  if (operationId === "favorite") return "收藏预览";
  return operationId;
}

export function sessionContinuationCommandPreview(preview: SessionContinuationPreview) {
  const root = preview.allowed_write_roots_summary[0] ?? "<authorized-root>";
  const cwd = preview.target_cwd ?? "<authorized-cwd>";
  return `工作目录 ${pathTail(cwd)}；沙箱 ${preview.sandbox_summary}；授权根 ${pathTail(root)}；提示词来源 ${preview.prompt_source_kind}；原始命令仅在开发者诊断中查看`;
}

export function j1ControlSlug(value: string) {
  return value
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "")
    .slice(0, 80) || "unknown";
}

export function sessionContinuationStatusTone(status: SessionContinuationPreview["guard_result"]["status"]): "candidate" | "warning" | "unknown" {
  if (status === "allowed_preview") return "candidate";
  if (status === "needs_user_confirmation") return "unknown";
  return "warning";
}

export function sessionContinuationStatusLabel(status: SessionContinuationPreview["guard_result"]["status"]) {
  if (status === "allowed_preview") return "可预览";
  if (status === "needs_user_confirmation") return "需要用户确认";
  if (status === "blocked") return "当前阻断";
  if (status === "requires_future_task") return "需要后续任务";
  return status;
}

export function groupSessionOperationsByAdapter(operations: SessionOperationDescriptor[]) {
  const groups = new Map<string, SessionOperationDescriptor[]>();
  for (const operation of operations) {
    const existing = groups.get(operation.adapter_id) ?? [];
    existing.push(operation);
    groups.set(operation.adapter_id, existing);
  }
  return Array.from(groups.entries())
    .map(([adapterId, groupedOperations]) => ({
      adapterId,
      operations: groupedOperations.sort((a, b) => sessionOperationOrder(a.operation_id) - sessionOperationOrder(b.operation_id)),
    }))
    .sort((a, b) => (a.adapterId === "codex-local" ? -1 : b.adapterId === "codex-local" ? 1 : a.adapterId.localeCompare(b.adapterId)));
}

export function sessionOperationOrder(operationId: SessionOperationDescriptor["operation_id"]) {
  return ["new_session", "send_message", "stop", "restart", "resume", "export", "delete", "favorite"].indexOf(operationId);
}

export function adapterDisplayName(adapterId: string) {
  if (adapterId === "codex-local") return "Codex";
  if (adapterId === "claude-code") return "Claude Code";
  if (adapterId === "openclaw") return "OpenClaw";
  if (adapterId === "opencode") return "OpenCode";
  if (adapterId === "opencode-like") return "OpenCode-like";
  return adapterId;
}

export function sessionOperationStatusTone(status: SessionOperationDescriptor["current_status"]): "candidate" | "warning" | "unknown" {
  if (status === "readonly_available") return "candidate";
  if (status === "planned") return "unknown";
  return "warning";
}

export function sessionOperationStatusLabel(status: SessionOperationDescriptor["current_status"]) {
  if (status === "readonly_available") return "只读可解释";
  if (status === "blocked") return "当前不可执行";
  if (status === "planned") return "计划中";
  if (status === "blocked_destructive") return "破坏性阻断";
  if (status === "requires_future_task") return "需要后续任务";
  return status;
}

export function sessionOperationRiskLabel(riskLevel: SessionOperationDescriptor["risk_level"]) {
  if (riskLevel === "low") return "低风险";
  if (riskLevel === "medium") return "中风险";
  if (riskLevel === "high") return "高风险";
  if (riskLevel === "destructive") return "破坏性";
  return riskLevel;
}

export function sessionOperationFlags(operation: SessionOperationDescriptor) {
  return [
    operation.requires_user_confirmation ? "需用户确认" : "无需本轮确认",
    operation.writes_codex_home ? "未来会写 Codex 主目录" : "不写 Codex 主目录",
    operation.writes_workbench_state ? "未来会写工作台状态" : "不写工作台状态",
    operation.writes_project_files ? "未来可能写文件" : "不写项目文件",
    operation.reads_full_transcript ? "需要脱敏会话记录" : "不读取完整会话记录",
    operation.requires_runtime_handle ? "需要运行句柄" : "不依赖运行句柄",
    operation.requires_credential ? "需要凭据边界" : "不读取凭据",
    operation.requires_model_access ? "需要模型访问边界" : "不调用模型",
  ];
}

export function providerAvailabilityTone(status: ProviderAvailabilitySummary["availability_status"]): "candidate" | "warning" | "unknown" {
  if (status === "available_readonly") return "candidate";
  if (status === "planned" || status === "not_configured" || status === "not_verified" || status === "blocked") return "warning";
  return "unknown";
}

export function providerAvailabilityStatusLabel(status: ProviderAvailabilitySummary["availability_status"]) {
  if (status === "available_readonly") return "只读可见";
  if (status === "planned") return "计划中";
  if (status === "not_connected") return "未连接";
  if (status === "not_configured") return "未配置";
  if (status === "not_verified") return "未验证";
  if (status === "blocked") return "阻断";
  if (status === "unknown") return "未知";
  return status;
}

export function credentialBoundaryStatusLabel(status: ProviderAvailabilitySummary["credential_status"]) {
  if (status === "not_required_by_workbench") return "工作台不读取";
  if (status === "not_configured") return "未配置";
  if (status === "not_readable_by_design") return "设计上不可读";
  if (status === "credential_missing") return "缺少凭据边界";
  if (status === "unknown") return "未知";
  return status;
}

export function modelAvailabilityStatusLabel(status: ProviderAvailabilitySummary["model_status"]) {
  if (status === "local_cli_managed") return "本地 CLI 管理";
  if (status === "not_verified") return "未验证";
  if (status === "model_unverified") return "模型未验证";
  if (status === "unknown") return "未知";
  if (status === "blocked") return "阻断";
  return status;
}

export function externalCallStatusLabel(status: ProviderAvailabilitySummary["external_call_status"]) {
  if (status === "not_needed_for_readonly") return "只读不需要";
  if (status === "external_call_blocked") return "外发调用已阻断";
  if (status === "requires_future_authorization") return "需要后续授权";
  return status;
}

export function costRiskStatusLabel(status: ProviderAvailabilitySummary["cost_risk_status"]) {
  if (status === "none_known") return "未见风险";
  if (status === "unknown") return "未估算";
  if (status === "external_cost_possible") return "可能产生成本";
  if (status === "blocked_until_authorized") return "授权前阻断";
  return status;
}

export function adapterHealthStatusLabel(status?: string | null) {
  if (!status) return "未登记";
  if (status === "available_with_guard") return "带边界可用";
  if (status === "degraded") return "降级";
  if (status === "blocked") return "阻断";
  if (status === "not_available") return "不可用";
  if (status === "planned") return "计划中";
  return status;
}

export function severityLabel(severity?: string | null) {
  if (!severity) return "未知";
  if (severity === "healthy") return "健康";
  if (severity === "warning") return "警告";
  if (severity === "degraded") return "降级";
  if (severity === "blocked") return "阻断";
  if (severity === "info") return "提示";
  return severity;
}

export function runtimeStatusLabel(status?: string | null) {
  if (!status) return "未知";
  if (status === "available") return "可用";
  if (status === "not_started") return "未启动";
  if (status === "running") return "运行中";
  if (status === "blocked") return "阻断";
  if (status === "degraded") return "降级";
  if (status === "unknown") return "未知";
  return status;
}

export function degradedModeLabel(mode?: string | null) {
  if (!mode) return "未知";
  if (mode === "none") return "无";
  if (mode === "descriptor_only") return "仅描述";
  if (mode === "readonly_only") return "仅只读";
  if (mode === "execution_blocked") return "执行阻断";
  if (mode === "credential_missing") return "缺凭据";
  return mode;
}

export function persistenceKindLabel(kind?: string | null) {
  if (!kind) return "仅描述";
  if (kind === "descriptor_only") return "仅描述";
  if (kind === "sidecar") return "辅助状态文件";
  if (kind === "workbench_store") return "工作台状态";
  if (kind === "external") return "外部位置";
  return kind;
}

export function eventKindLabel(kind: string) {
  if (kind === "adapter_health") return "适配器健康";
  if (kind === "runtime_log") return "运行日志";
  if (kind === "dispatch_attempt") return "派发尝试";
  if (kind === "readback") return "读回";
  if (kind === "permission") return "权限";
  if (kind === "diagnostic") return "诊断";
  return kind;
}

export function adapterStatusTone(status: AgentAdapterDescriptor["status"]): "candidate" | "warning" | "unknown" {
  if (status === "available") return "candidate";
  if (status === "planned" || status === "not_configured" || status === "blocked") return "warning";
  return "unknown";
}

export function adapterStatusLabel(status: AgentAdapterDescriptor["status"]) {
  if (status === "available") return "可用";
  if (status === "degraded") return "降级";
  if (status === "not_connected") return "未连接";
  if (status === "planned") return "计划中";
  if (status === "not_configured") return "未配置";
  if (status === "blocked") return "阻止";
  return status;
}

export function adapterExecutionStatusLabel(status: AgentAdapterDescriptor["execution_status"]) {
  if (status === "available_with_user_confirmation") return "需用户确认";
  if (status === "not_connected") return "未连接";
  if (status === "not_implemented") return "未实现";
  return status;
}

export function adapterCredentialStatusLabel(status: AgentAdapterDescriptor["credential_status"]) {
  if (status === "not_read") return "未读取";
  if (status === "not_configured") return "未配置";
  return status;
}

export function adapterModelStatusLabel(status: AgentAdapterDescriptor["model_access_status"]) {
  if (status === "local_read_model_only") return "本地读模型";
  if (status === "not_verified") return "未验证";
  return status;
}

export function capabilityStatusLabel(status: AdapterCapabilityStatus) {
  if (status === "available") return "可用";
  if (status === "requires_confirmation") return "需确认";
  if (status === "read_only") return "只读";
  if (status === "blocked") return "阻止";
  return status;
}

export function messageOf(error: unknown): string {
  if (error instanceof Error) return error.message;
  return String(error);
}

export async function sha256HexText(value: string): Promise<string> {
  if (!globalThis.crypto?.subtle) {
    throw new Error("当前环境缺少 Web Crypto，无法生成任务正文摘要。");
  }
  const bytes = new TextEncoder().encode(value);
  const digest = await globalThis.crypto.subtle.digest("SHA-256", bytes);
  return Array.from(new Uint8Array(digest))
    .map((byte) => byte.toString(16).padStart(2, "0"))
    .join("");
}
