import type { DiagnosticSummary, RuntimeLogStoreV1, RuntimeSessionAttention } from "../../src/lib/types";

export function diagnosticSummaryFixture(): DiagnosticSummary {
  return {
    status: "degraded_readonly",
    generated_at: "2026-06-07T00:00:00Z",
    overall_severity: "degraded",
    healthy_count: 2,
    warning_count: 2,
    degraded_count: 1,
    blocked_count: 1,
    store_integrity: [
      {
        store_id: "workflow_state",
        label: "工作流事实层",
        status: "ok",
        severity: "info",
        path: "/offline-fixture/workflow-state.v0.json",
        schema_version: "workflow_state.v0",
        revision: 3,
        item_count: 8,
        warning_count: 0,
        error: null,
        summary: "工作流事实层可读取。",
        boundary: "只读解析 workflow-state.v0.json；G2 不修改状态枚举或顶层结构。",
      },
      {
        store_id: "runtime_log",
        label: "运行日志 sidecar",
        status: "warning",
        severity: "warning",
        path: "/offline-fixture/runtime-logs.v1.json",
        schema_version: "runtime_log_store.v1",
        revision: 1,
        item_count: 6,
        warning_count: 1,
        error: null,
        summary: "运行日志 sidecar 有 1 条 warning，G2 只解释不修复。",
        boundary: "运行日志只记录脱敏运行摘要；不能替代审计事件。",
      },
    ],
    degraded_states: [
      {
        state_id: "diagnostic:runtime_attention",
        kind: "runtime_attention",
        severity: "warning",
        title: "运行关注存在阻断或读回边界",
        summary: "1 条运行关注需要解释；读回不可用不是 0 条结果。",
        user_action_required: true,
        blocks_real_execution: true,
        source_refs: ["workbench_snapshot.runtime_session_attention"],
        recommended_next_step: "查看运行中入口和管理入口摘要；G2 不自动恢复、重试或修复。",
      },
      {
        state_id: "diagnostic:bundle_reference",
        kind: "diagnostic_bundle_reference",
        severity: "info",
        title: "诊断 bundle 为只读引用",
        summary: "G2 在 WorkbenchSnapshot.diagnostic_summary 中提供可引用诊断 bundle；不导出 secret、不生成新文件。",
        user_action_required: false,
        blocks_real_execution: false,
        source_refs: ["workbench_snapshot.diagnostic_summary"],
        recommended_next_step: "如需落盘导出 bundle，必须另拆任务并定义脱敏规则。",
      },
    ],
    recent_error_summaries: ["readback · readback failed safe summary"],
    boundary_notes: [
      "G2 是只读诊断，不自动修复 store、不自动重试、不调用 provider。",
      "读回不可用表示无法读回，不能显示成 0 条结果。",
      "真实 Tauri 截图验收仍属于 G3，不由 G2 冒领。",
    ],
  };
}

export function runtimeLogStoreFixture(projectRoot: string, sessionId: string): RuntimeLogStoreV1 {
  const categories = [
    "app_session",
    "workflow_run",
    "dispatch_attempt",
    "readback",
    "permission_wait",
    "diagnostic_event",
  ];
  const entries = categories.map((category, index) => ({
    entry_version: 1,
    entry_id: `runtime-log:${category}:offline`,
    category,
    status: index === 3 ? "readback_unavailable" : "observed",
    severity: index === 3 || index === 4 ? "warning" : "info",
    started_at: "2026-06-07T00:00:00Z",
    finished_at: index === 4 ? null : "2026-06-07T00:00:01Z",
    duration_ms: index === 2 ? 1000 : null,
    project_id: "project:offline",
    workflow_id: "workflow:offline",
    node_id: "node:offline",
    session_id: sessionId,
    adapter_id: "codex-local",
    summary: `${category} redacted runtime summary`,
    detail: "只展示脱敏运行摘要；不展示正文、凭据或原始会话记录。",
    source_refs: [
      {
        source_kind: category === "dispatch_attempt" ? "session_continuation_attempt" : "workbench_runtime",
        source_id: `source:${category}:offline`,
        label: category,
      },
    ],
    audit_refs: category === "dispatch_attempt" ? ["audit:runtime-log:offline"] : [],
    redaction_status: "redacted_safe_summary",
    sensitive_omissions: ["conversation_body", "credential_material"],
    user_visible: true,
    warnings: category === "readback" ? ["readback_unavailable_is_not_zero_results"] : [],
  }));

  return {
    schema_version: "runtime_log_store.v1",
    store_version: 1,
    storage_kind: "sidecar_json_v0",
    scope: {
      scope_kind: "workflow_state_sidecar",
      workflow_state_path: "/offline-fixture/workflow-state.v0.json",
      sidecar_path: "/offline-fixture/runtime-logs.v1.json",
      project_roots: [projectRoot],
    },
    revision: 1,
    last_write_id: null,
    generated_by: "offline-test",
    created_at: "2026-06-07T00:00:00Z",
    updated_at: "2026-06-07T00:00:01Z",
    boundary: {
      runtime_log_definition: "运行日志记录运行状态、耗时、分类、来源引用和脱敏摘要。",
      audit_event_definition: "审计事件记录可追责的操作者、权限、状态变化和原因。",
      separation_rule: "运行日志与审计事件不能互相替代；日志只引用审计引用，不内嵌审计事件本体。",
      redaction_rule: "日志展示必须脱敏；授权材料、环境材料、会话正文和供应方原始材料只记录为省略类别。",
      forbidden_payloads: ["credential_material", "conversation_body", "raw_provider_material"],
    },
    entries,
    summaries: categories.map((category) => ({
      category,
      status: category === "readback" ? "readback_unavailable" : "observed",
      severity: category === "readback" || category === "permission_wait" ? "warning" : "info",
      entry_count: 1,
      latest_entry_ids: [`runtime-log:${category}:offline`],
      warnings: [],
    })),
    warnings: ["runtime_log_does_not_replace_audit_event", "audit_event_does_not_replace_runtime_log"],
  };
}

export function runtimeAttentionFixtures(sessionId: string): RuntimeSessionAttention[] {
  return [
    runtimeAttentionFixture(sessionId, "waiting-permission", "waiting_permission", "needs_user", "readback_unavailable", "level_b_not_authorized", true, false),
    runtimeAttentionFixture(sessionId, "guard-blocked", "blocked_by_guard", "blocking", "readback_unavailable", "guard_blocked", false, true),
    runtimeAttentionFixture(sessionId, "readback-unavailable", "readback_unavailable", "warning", "readback_unavailable", "not_attempted_stub", true, false),
    runtimeAttentionFixture(sessionId, "readback-failed", "readback_failed", "needs_user", "readback_failed", "readback_parser_failed", true, true),
  ];
}

export function runtimeAttentionFixture(
  sessionId: string,
  id: string,
  status: RuntimeSessionAttention["status"],
  severity: RuntimeSessionAttention["severity"],
  readbackStatus: RuntimeSessionAttention["readback_boundary"]["status"],
  reason: RuntimeSessionAttention["readback_boundary"]["reason"],
  requiresUserAction: boolean,
  blocksContinuation: boolean,
): RuntimeSessionAttention {
  return {
    attention_id: `runtime-attention:${id}`,
    project_id: "project:offline",
    workflow_id: "workflow:offline",
    node_id: "node:offline",
    session_id: sessionId,
    adapter_id: "codex-local",
    source_refs: [
      {
        source_kind: "session_continuation_attempt",
        source_id: `source:${id}`,
        label: id,
      },
    ],
    kind: status,
    severity,
    status,
    title: `查看 E6 ${status}`,
    user_message:
      readbackStatus === "readback_failed"
        ? "读回失败表示读回失败或不可信，不能显示成空读回。"
        : "读回不可用表示没有真实读回来源，不能显示成空读回。",
    technical_summary: `status=${status} readback=${readbackStatus}`,
    recommended_next_step: "查看运行关注边界；不要自动重试、停止、恢复或批准权限。",
    requires_user_action: requiresUserAction,
    blocks_continuation: blocksContinuation,
    readback_boundary: {
      status: readbackStatus,
      reason,
      attempted: false,
      real_readback_performed: false,
      result_count: null,
      user_message: "unavailable / failed 都不是空读回结果。",
      technical_summary: `reason=${reason}`,
      source_refs: [
        {
          source_kind: "session_continuation_attempt",
          source_id: `source:${id}`,
          label: id,
        },
      ],
      warnings: [],
    },
    created_at: "2026-06-06T00:00:00Z",
    updated_at: "2026-06-06T00:00:00Z",
    warnings: [],
  };
}
