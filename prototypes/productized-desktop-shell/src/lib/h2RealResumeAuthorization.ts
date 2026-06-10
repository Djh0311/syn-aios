import type {
  H2RealResumeDecisionCheck,
  H2RealResumeAuthorizationReadiness,
  H2RealResumeAuthorizationReadinessItem,
  H2RealResumeExecutionDecisionStatus,
  H2RealResumeExecutionDecisionSurface,
  H2RealResumeReadbackDecisionBoundary,
  SessionContinuationPreview,
  SessionContinuationStoreV1,
} from "./types";

const RECOMMENDED_FIXTURE_PATH = "/Users/yoyi/workspace/product-line/tmp/h2-real-resume-fixture";

type DeriveH2ReadinessInput = {
  previews: SessionContinuationPreview[];
  store: SessionContinuationStoreV1 | null;
};

export function deriveH2RealResumeAuthorizationReadiness({
  previews,
  store,
}: DeriveH2ReadinessInput): H2RealResumeAuthorizationReadiness {
  const resumePreview = previews.find(
    (preview) => preview.adapter_id === "codex-local" && preview.operation_id === "resume",
  );
  const latestContinuation = latestCodexResumeContinuation(store);
  const targetSessionId = latestContinuation?.session_id ?? resumePreview?.target_session_id ?? null;
  const targetProjectRoot = latestContinuation?.project_root ?? resumePreview?.project_root ?? null;
  const targetCwd = latestContinuation?.target_cwd ?? resumePreview?.target_cwd ?? null;
  const allowedWriteRoots = latestContinuation?.allowed_write_roots ?? resumePreview?.allowed_write_roots_summary ?? [];
  const promptSummary = latestContinuation?.prompt_summary ?? resumePreview?.prompt_summary ?? "";
  const readbackStrategy = latestContinuation?.readback_strategy ?? resumePreview?.readback_expectation.strategy ?? "";
  const sandbox = latestContinuation?.sandbox ?? resumePreview?.sandbox_summary ?? "";

  const items: H2RealResumeAuthorizationReadinessItem[] = [
    confirmedItem("operation_type", "操作类型", "resume", "H2 只允许 codex-local resume，不包含 H3 send / new session。"),
    readinessItem("test_project", "测试项目", null, "missing", "必须由用户确认隔离测试项目；不能默认复用 mario test。"),
    readinessItem("recommended_fixture", "推荐测试夹具", RECOMMENDED_FIXTURE_PATH, "recommended_default", "推荐路径仅是低风险默认建议，创建前仍需用户确认。"),
    readinessItem("project_root", "项目根目录", targetProjectRoot, targetProjectRoot ? "confirmed" : "missing", "必须是用户确认的绝对路径。"),
    readinessItem("target_cwd", "执行目录", targetCwd, targetCwd ? "confirmed" : "missing", "必须在项目根目录或允许写入根目录内。"),
    readinessItem("target_session", "目标会话", targetSessionId, targetSessionId ? "confirmed" : "missing", "必须由用户指定或工作台绑定；不能读取 .codex 猜测。"),
    readinessItem("prompt_summary", "提示词摘要", promptSummary, promptSummary ? "confirmed" : "missing", "必须有人可读摘要；完整提示词不进入命令参数。"),
    readinessItem("prompt_hash_ref", "提示词哈希 / 引用", null, "missing", "真实执行前必须固定哈希 / 引用，H2.2 不伪造。"),
    readinessItem(
      "allowed_write_roots",
      "允许写入根目录",
      allowedWriteRoots.length ? allowedWriteRoots.join(" / ") : null,
      allowedWriteRoots.length ? "confirmed" : "missing",
      "写入范围必须足够窄，建议只包含 fixture 项目目录。",
    ),
    readinessItem("codex_home_scope", ".codex 最小范围", null, "missing", "必须由用户确认，只能覆盖 resume 必需最小触碰范围。"),
    readinessItem("sandbox", "沙箱模式", sandbox, sandbox ? "confirmed" : "missing", "必须明确沙箱，禁止危险绕过。"),
    readinessItem("timeout", "超时时间", "120000 ms", "recommended_default", "推荐默认值；真实执行前仍需确认。"),
    readinessItem(
      "readback_plan",
      "读回计划",
      readbackStrategy,
      readbackStrategy && readbackStrategy !== "not_defined" ? "confirmed" : "missing",
      "必须说明预期来源、不可用时行为和信任策略。",
    ),
    readinessItem(
      "runtime_audit_evidence",
      "运行日志 / 审计 / 证据",
      "运行日志 + 审计 + H2 证据 / 交接",
      "recommended_default",
      "真实执行后必须分别记录运行态、权限审计和读回，不互相替代。",
    ),
    readinessItem("rollback_plan", "回滚 / 降级", null, "missing", "必须包含执行前后哈希、失败分类和停止策略。"),
    readinessItem("user_confirmation", "用户确认", null, "missing", "必须由用户明确允许真实 codex exec resume。"),
    readinessItem("global_supervisor_confirmation", "全局主管确认", null, "missing", "必须由全局主管复核授权矩阵后才可执行。"),
  ];

  const missingCount = items.filter((item) => item.status === "missing").length;
  const blockedCount = items.filter((item) => item.status === "blocked").length;
  const confirmedCount = items.filter((item) => item.status === "confirmed").length;
  const ready = missingCount === 0 && blockedCount === 0;

  return {
    schema_version: "h2_real_resume_authorization_readiness.v1",
    status: ready ? "ready_for_explicit_authorization" : "blocked_waiting_authorization",
    summary: ready
      ? "授权矩阵字段已齐，但仍必须由用户和全局主管明确确认后才可执行真实恢复。"
      : "H2 真实恢复仍缺执行前授权项；当前只能展示矩阵，不执行、不发送提示词、不读写 .codex。",
    target_continuation_id: latestContinuation?.continuation_id ?? null,
    target_session_id: targetSessionId,
    target_project_root: targetProjectRoot,
    recommended_fixture_path: RECOMMENDED_FIXTURE_PATH,
    missing_count: missingCount,
    confirmed_count: confirmedCount,
    blocked_count: blockedCount,
    readiness_items: items,
    warnings: [
      "h2_readiness_is_not_execution_authorization",
      "no_prompt_sent_in_h2_readiness",
      "no_codex_home_read_write_in_h2_readiness",
      "readback_unavailable_is_not_zero_results",
    ],
  };
}

export function deriveH2RealResumeExecutionDecisionSurface({
  previews,
  store,
}: DeriveH2ReadinessInput): H2RealResumeExecutionDecisionSurface {
  const readiness = deriveH2RealResumeAuthorizationReadiness({ previews, store });
  const resumePreview = previews.find(
    (preview) => preview.adapter_id === "codex-local" && preview.operation_id === "resume",
  );
  const latestContinuation = latestCodexResumeContinuation(store);
  const latestAttempt = latestContinuation
    ? latestAttemptForContinuation(store, latestContinuation.continuation_id)
    : null;
  const duplicateAttempts = (store?.attempts ?? []).filter(
    (attempt) =>
      attempt.execution_level === "level_b_real_user_approved" &&
      (attempt.status === "queued" || attempt.status === "running" || attempt.status === "running_real"),
  );
  const targetSessionId = latestContinuation?.session_id ?? resumePreview?.target_session_id ?? null;
  const targetProjectRoot = latestContinuation?.project_root ?? resumePreview?.project_root ?? null;
  const targetCwd = latestContinuation?.target_cwd ?? resumePreview?.target_cwd ?? null;
  const allowedWriteRoots = latestContinuation?.allowed_write_roots ?? resumePreview?.allowed_write_roots_summary ?? [];
  const promptSummary = latestContinuation?.prompt_summary ?? resumePreview?.prompt_summary ?? "";
  const readbackStrategy = latestContinuation?.readback_strategy ?? resumePreview?.readback_expectation.strategy ?? "";
  const guardBlocked = Boolean(resumePreview?.guard_result.blocks_execution || resumePreview?.guard_result.status === "blocked");
  const readbackBoundary = deriveReadbackDecisionBoundary(latestAttempt?.readback_summary.status, latestAttempt?.readback_summary.result_count);

  const checks: H2RealResumeDecisionCheck[] = [
    decisionCheck("operation", "operation", "resume", "ready", false, "H2.8 仅覆盖 codex-local resume；send / new session 属于 H3。"),
    decisionCheck(
      "target_session",
      "target session",
      targetSessionId,
      targetSessionId ? "ready" : "missing",
      !targetSessionId,
      "必须由用户或工作台绑定目标会话；不能读取 .codex 猜测。",
    ),
    decisionCheck(
      "fixture",
      "测试夹具",
      targetProjectRoot ? "需要最终批准绑定测试夹具" : null,
      "missing",
      true,
      "H2 Phase B 仍缺确认测试夹具；不能默认拿当前真实项目执行。",
    ),
    decisionCheck(
      "permission_envelope",
      "permission envelope",
      null,
      "missing",
      true,
      "必须冻结谁批准、允许写哪里、拒绝后如何记录、失败后如何降级。",
    ),
    decisionCheck(
      "allowed_write_roots",
      "allowed write roots",
      allowedWriteRoots.length ? allowedWriteRoots.join(" / ") : null,
      allowedWriteRoots.length ? "ready" : "missing",
      allowedWriteRoots.length === 0,
      "真实执行前必须限制到明确测试项目或授权根目录。",
    ),
    decisionCheck(
      "prompt_envelope",
      "提示词信封",
      promptSummary,
      promptSummary ? "preview" : "missing",
      !promptSummary,
      "只展示提示词摘要；真实执行前还必须固定提示词引用和哈希。",
    ),
    decisionCheck(
      "codex_home_scope",
      ".codex 副作用范围",
      null,
      "missing",
      true,
      "真实 resume 会触碰 Codex 原生状态；必须先声明最小范围和回收证据。",
    ),
    decisionCheck(
      "readback_plan",
      "读回计划",
      readbackStrategy,
      readbackStrategy && readbackStrategy !== "not_defined" ? "preview" : "missing",
      !readbackStrategy || readbackStrategy === "not_defined",
      "必须说明读回不可用 / 失败 / 超时时如何展示，不能推断为空结果。",
    ),
    decisionCheck(
      "runtime_log",
      "运行日志预览",
      "runtime_log_event:start/end/failure/readback_boundary",
      "preview",
      false,
      "这里只预览将写入的脱敏运行日志；H2.8 不写真实执行日志。",
    ),
    decisionCheck(
      "audit",
      "审计预览",
      "audit:user_decision/guard_blocked/attempt_boundary",
      "preview",
      false,
      "审计记录决策事实，不替代运行日志或读回。",
    ),
    decisionCheck(
      "rollback",
      "rollback / cleanup",
      null,
      "missing",
      true,
      "真实执行前必须有失败分类、停止策略、diff/hash 检查和清理说明。",
    ),
    decisionCheck(
      "duplicate_attempt",
      "重复排队 / 运行尝试",
      duplicateAttempts.length ? `${duplicateAttempts.length} 条活跃尝试` : "无",
      duplicateAttempts.length ? "blocked" : "ready",
      duplicateAttempts.length > 0,
      "同一类真实恢复已排队 / 运行时必须阻断最终批准。",
    ),
    decisionCheck(
      "guard",
      "H1 guard",
      resumePreview?.guard_result.status ?? "missing preview",
      guardBlocked ? "blocked" : resumePreview ? "preview" : "missing",
      guardBlocked || !resumePreview,
      "守卫阻断或预览缺失时不能进入最终批准。",
    ),
    decisionCheck(
      "diagnostics",
      "诊断",
      "H2.8 派生界面没有阻断性诊断",
      "preview",
      false,
      "H2.8 只显示诊断入口；若 G2 诊断阻断，后续最终批准必须停下。",
    ),
    decisionCheck(
      "user_final_approval",
      "用户最终批准",
      null,
      "missing",
      true,
      "用户最终批准不能由就绪面板替代。",
    ),
    decisionCheck(
      "global_supervisor_final_review",
      "全局主管最终复核",
      null,
      "missing",
      true,
      "全局主管必须在 evidence/handoff 和授权矩阵齐备后复核。",
    ),
  ];

  const status = decisionSurfaceStatus(checks);
  const finalApprovalAllowed = status === "ready_for_final_approval";

  return {
    schema_version: "h2_real_resume_execution_decision_surface.v1",
    adapter_id: "codex-local",
    operation_id: "resume",
    status,
    authorization_status: readiness.status,
    summary: finalApprovalAllowed
      ? "H2.8 决策材料已齐备；仍需用户和全局主管最终批准后才可真实恢复。"
      : "H2.8 仍是执行前决策材料；当前存在阻断或缺项，不能真实恢复，不能发送提示词，不能读写 .codex。",
    final_approval_allowed: finalApprovalAllowed,
    target_continuation_id: latestContinuation?.continuation_id ?? null,
    target_session_id: targetSessionId,
    duplicate_attempt_blocked: duplicateAttempts.length > 0,
    duplicate_attempt_count: duplicateAttempts.length,
    decision_checks: checks,
    permission_preview: {
      operation_label: "codex-local resume",
      target_project: targetProjectRoot ?? "待确认测试项目",
      workflow_label: resumePreview?.workflow_id ?? latestContinuation?.workflow_id ?? "待确认 workflow",
      node_label: resumePreview?.node_id ?? latestContinuation?.node_id ?? "待确认 node",
      work_item_label: resumePreview?.work_item_id ?? "待确认 work item",
      target_session_summary: targetSessionId ? `session:${targetSessionId}` : "待确认；不能读取 .codex 猜测",
      project_root: targetProjectRoot ?? "待确认",
      target_cwd: targetCwd ?? "待确认",
      allowed_write_roots: allowedWriteRoots,
      denied_paths: [
        "/Users/yoyi/.codex/auth.json",
        "/Users/yoyi/.codex/state_*.sqlite direct edit",
        "auth/token/.env/keychain/OAuth/provider credential",
        "原始会话记录 / 完整回放记录",
      ],
      prompt_summary: promptSummary || "待确认；完整提示词不在 UI 展示",
      prompt_ref: "待确认提示词引用",
      prompt_hash: "待确认提示词 sha256",
      task_memory_packet_summary: "只展示入选 / 排除 / 待审查材料 / 检查阻断摘要；候选和知识命中不等于正式记忆。",
      codex_home_scope_summary: "待最终批准声明最小 .codex 副作用；H2.8 不读写。",
      sandbox_summary: latestContinuation?.sandbox ?? resumePreview?.sandbox_summary ?? "待确认",
      timeout_summary: "推荐 120000 ms；真实执行前仍需确认。",
      duplicate_guard_summary: duplicateAttempts.length
        ? `阻断：已有 ${duplicateAttempts.length} 条排队 / 运行中的真实尝试。`
        : "未发现排队 / 运行中的 Level B 尝试；仍需执行前复核。",
      approval_effect: "批准后才允许后续任务进入真实 codex exec resume；H2.8 本身不执行。",
      rejection_effect: "拒绝只写用户决定和审计摘要，不发送提示词，不写 Codex 原生状态。",
      blocked_effect: "阻断时只展示必须修复项和证据 / 交接缺口，不自动重试。",
      warnings: [
        "permission_preview_is_not_approval",
        "full_prompt_hidden_by_default",
        "secret_and_raw_transcript_forbidden",
      ],
    },
    audit_runtime_preview: {
      audit_preview: [
        "user_final_approval_or_rejection",
        "global_supervisor_boundary_review",
        "guard_blocked_or_duplicate_blocked",
        "attempt_started_finished_failed_after_future_execution",
      ],
      runtime_log_preview: [
        "runtime_start_redacted_command",
        "runtime_finish_duration_exit_code",
        "timeout_or_failure_category",
        "source_refs_without_secret_or_raw_transcript",
      ],
      readback_preview: [
        "readback_unavailable_is_boundary_state",
        "readback_failed_keeps_result_count_null",
        "readback_timed_out_keeps_result_count_null",
      ],
      evidence_preview: [
        "H2 Phase B evidence path must be created by future task",
        "handoff must state whether .codex was touched",
      ],
      rollback_preview: [
        "pre/post project hash plan required",
        "cleanup plan required for fixture writes",
        "no rollback promise for Codex internal state without explicit scope",
      ],
    },
    readback_boundary: readbackBoundary,
    planned_adapter_boundary: "planned adapters remain planned / unavailable / blocked; H2.8 only covers codex-local resume.",
    warnings: [
      "h2_8_decision_surface_is_not_execution",
      "h2_phase_b_final_approval_not_granted",
      "no_prompt_sent_in_h2_8",
      "no_codex_home_read_write_in_h2_8",
      "readback_unavailable_failed_timed_out_are_not_zero_results",
    ],
  };
}

function latestCodexResumeContinuation(store: SessionContinuationStoreV1 | null) {
  return [...(store?.continuations ?? [])]
    .filter((continuation) => continuation.adapter_id === "codex-local" && continuation.operation_id === "resume")
    .sort((a, b) => b.updated_at.localeCompare(a.updated_at))[0] ?? null;
}

function latestAttemptForContinuation(store: SessionContinuationStoreV1 | null, continuationId: string) {
  return [...(store?.attempts ?? [])]
    .filter((attempt) => attempt.continuation_id === continuationId)
    .sort((a, b) => b.started_at.localeCompare(a.started_at))[0] ?? null;
}

function decisionSurfaceStatus(checks: H2RealResumeDecisionCheck[]): H2RealResumeExecutionDecisionStatus {
  if (checkBlocks(checks, "duplicate_attempt")) return "blocked_by_duplicate_attempt";
  if (checkBlocks(checks, "target_session")) return "blocked_waiting_target_session";
  if (checkBlocks(checks, "fixture")) return "blocked_waiting_fixture";
  if (checkBlocks(checks, "permission_envelope")) return "blocked_waiting_permission_envelope";
  if (checkBlocks(checks, "allowed_write_roots")) return "blocked_waiting_allowed_write_roots";
  if (checkBlocks(checks, "prompt_envelope")) return "blocked_waiting_prompt_envelope";
  if (checkBlocks(checks, "codex_home_scope")) return "blocked_waiting_codex_home_scope";
  if (checkBlocks(checks, "readback_plan")) return "blocked_waiting_readback_plan";
  if (checkBlocks(checks, "runtime_log")) return "blocked_waiting_runtime_log";
  if (checkBlocks(checks, "audit")) return "blocked_waiting_audit";
  if (checkBlocks(checks, "rollback")) return "blocked_waiting_rollback";
  if (checkBlocks(checks, "guard")) return "blocked_by_guard";
  if (checkBlocks(checks, "diagnostics")) return "blocked_by_diagnostics";
  if (checkBlocks(checks, "user_final_approval") || checkBlocks(checks, "global_supervisor_final_review")) {
    return "ready_but_not_authorized";
  }
  return "ready_for_final_approval";
}

function checkBlocks(checks: H2RealResumeDecisionCheck[], checkId: string) {
  return checks.some((check) => check.check_id === checkId && check.blocks_final_approval);
}

function decisionCheck(
  checkId: string,
  label: string,
  value: string | null,
  status: H2RealResumeDecisionCheck["status"],
  blocksFinalApproval: boolean,
  userVisibleReason: string,
): H2RealResumeDecisionCheck {
  return {
    check_id: checkId,
    label,
    status,
    value,
    blocks_final_approval: blocksFinalApproval,
    user_visible_reason: userVisibleReason,
  };
}

function deriveReadbackDecisionBoundary(
  status: string | undefined,
  resultCount: number | null | undefined,
): H2RealResumeReadbackDecisionBoundary {
  if (status === "readback_failed") {
    return {
      status: "readback_failed",
      attempted: true,
      real_readback_performed: false,
      result_count: null,
      display_label: "读回失败，结果数未知",
      user_message: "读回失败表示结果数未知；必须显示为失败边界并保留 required fixes。",
      warnings: ["readback_failed_is_not_zero_results"],
    };
  }
  if (status === "timed_out" || status === "readback_timed_out") {
    return {
      status: "readback_timed_out",
      attempted: true,
      real_readback_performed: false,
      result_count: null,
      display_label: "读回超时，结果数未知",
      user_message: "读回超时表示结果数未知；不能把超时展示成空结果。",
      warnings: ["readback_timed_out_is_not_zero_results"],
    };
  }
  if (status === "readback_unavailable" || status === "not_attempted_stub") {
    return {
      status: "readback_unavailable",
      attempted: status === "readback_unavailable",
      real_readback_performed: false,
      result_count: null,
      display_label: "读回不可用，结果数未知",
      user_message: "读回不可用是边界状态；H2.8 只能显示结果数未知。",
      warnings: ["readback_unavailable_is_not_zero_results"],
    };
  }
  if (typeof resultCount === "number") {
    return {
      status: "ready_for_plan",
      attempted: true,
      real_readback_performed: false,
      result_count: resultCount,
      display_label: "读回计划预览",
      user_message: "H2.8 只显示读回计划或历史摘要，不执行真实读回。",
      warnings: ["h2_8_readback_preview_only"],
    };
  }
  return {
    status: "not_attempted",
    attempted: false,
    real_readback_performed: false,
    result_count: null,
    display_label: "未尝试读回",
    user_message: "真实执行前必须先有读回计划；未尝试表示结果数未知。",
    warnings: ["readback_not_attempted_is_not_zero_results"],
  };
}

function confirmedItem(
  itemId: string,
  label: string,
  value: string,
  userVisibleReason: string,
): H2RealResumeAuthorizationReadinessItem {
  return readinessItem(itemId, label, value, "confirmed", userVisibleReason);
}

function readinessItem(
  itemId: string,
  label: string,
  value: string | null,
  status: H2RealResumeAuthorizationReadinessItem["status"],
  userVisibleReason: string,
): H2RealResumeAuthorizationReadinessItem {
  return {
    item_id: itemId,
    label,
    status,
    value,
    user_visible_reason: userVisibleReason,
  };
}
