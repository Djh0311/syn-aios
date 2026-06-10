import type {
  AutoDispatchGuardResult,
  GlobalBoundaryReviewStatus,
  PlanAuthorization,
  PlanAuthorizationReadModel,
  PlanAuthorizationStatus,
  PlanAuthorizationStoreV1,
} from "./types";
import type { ProjectConsultationProposalSummary } from "./projectConsultationProposal";

export const planAuthorizationStatusLabels: Record<PlanAuthorizationStatus, string> = {
  draft: "草稿",
  pending_user_confirmation: "待用户确认",
  user_confirmed: "待全局复核",
  pending_global_boundary_review: "待全局复核",
  active: "授权有效",
  paused: "已暂停",
  revoked: "已撤销",
  expired: "已过期",
  completed: "已完成",
};

export const autoDispatchGuardStatusLabels: Record<string, string> = {
  authorized: "authorized",
  blocked: "blocked",
  needs_review: "needs_review",
};

export const globalBoundaryReviewStatusLabels: Record<GlobalBoundaryReviewStatus, string> = {
  approved: "复核通过，授权有效",
  needs_changes: "要求修改",
  blocked: "已阻断",
};

export function summarizePlanAuthorizationStore(
  store: PlanAuthorizationStoreV1 | null,
  projectId?: string | null,
  workflowId?: string | null,
): PlanAuthorizationReadModel {
  const matching = (store?.authorizations ?? []).filter(
    (authorization) =>
      (!projectId || authorization.project_id === projectId) &&
      (!workflowId || authorization.workflow_id === workflowId),
  );
  const latest = matching[matching.length - 1] ?? null;
  const active = [...matching].reverse().find((authorization) => authorization.status === "active") ?? null;
  const source = active ?? latest;
  const recentAudit = [...(store?.audit_events ?? [])]
    .reverse()
    .find((event) => (!projectId || event.project_id === projectId) && (!workflowId || event.workflow_id === workflowId));

  if (!store || !source) {
    return {
      sidecar_name: "plan-authorizations.v1.json",
      revision: store?.revision ?? 0,
      project_id: projectId ?? "unknown",
      workflow_id: workflowId ?? "unknown",
      authorization_count: matching.length,
      active_authorization_id: null,
      latest_authorization_id: latest?.authorization_id ?? null,
      latest_status: latest?.status ?? null,
      actor_scope: null,
      resource_scope: null,
      stop_condition_count: 0,
      recent_audit_event_id: recentAudit?.audit_event_id ?? null,
      recent_guard_result: recentAudit?.guard_result ?? null,
      display_text: "未建立方案授权；不能自动推进",
      warnings: store?.warnings ?? ["plan_authorization_store_not_loaded"],
    };
  }

  return {
    sidecar_name: "plan-authorizations.v1.json",
    revision: store?.revision ?? 0,
    project_id: source.project_id,
    workflow_id: source.workflow_id,
    authorization_count: matching.length,
    active_authorization_id: active?.authorization_id ?? null,
    latest_authorization_id: latest?.authorization_id ?? null,
    latest_status: latest?.status ?? null,
    actor_scope: {
      allowed_role_ids: source.scope.allowed_role_ids,
      allowed_agent_ids: source.scope.allowed_agent_ids,
    },
    resource_scope: {
      allowed_read_roots: source.scope.allowed_read_roots,
      allowed_write_roots: source.scope.allowed_write_roots,
      allowed_tools: source.scope.allowed_tools,
      allowed_checks: source.scope.allowed_checks,
      allowed_task_package_kinds: source.scope.allowed_task_package_kinds,
    },
    stop_condition_count: source.scope.stop_conditions.length,
    recent_audit_event_id: recentAudit?.audit_event_id ?? null,
    recent_guard_result: recentAudit?.guard_result ?? null,
    display_text: planAuthorizationDisplayText(source),
    warnings: store?.warnings ?? [],
  };
}

export function summarizeAutoDispatchGuardResult(result: AutoDispatchGuardResult | null) {
  if (!result) {
    return {
      status: "not_checked",
      display_text: "未检查当前任务包授权范围",
      reason_text: "暂无授权检查结果",
      reasons: [],
    };
  }
  const reasonText = result.reasons.slice(0, 3).join("；") || "当前任务包在授权范围内";
  return {
    status: result.status,
    display_text: `${autoDispatchGuardStatusLabels[result.status] ?? result.status} / ${reasonText}`,
    reason_text: reasonText,
    reasons: result.reasons.slice(0, 3),
  };
}

export function summarizeGlobalBoundaryReview(
  proposalSummary: ProjectConsultationProposalSummary,
  authorizationSummary: PlanAuthorizationReadModel,
  guardResult: AutoDispatchGuardResult | null,
) {
  const proposal = proposalSummary.latest_proposal;
  const authorization = proposalSummary.linked_plan_authorization;
  const review = authorization?.global_boundary_review ?? null;
  const guardSummary = summarizeAutoDispatchGuardResult(guardResult ?? authorizationSummary.recent_guard_result ?? null);
  const blockedReasons: string[] = [];

  if (!proposal) {
    blockedReasons.push("还没有项目咨询方案草案。");
  } else if (proposal.status !== "user_confirmed") {
    blockedReasons.push("项目咨询方案尚未由用户确认。");
  }
  if (proposal?.status === "user_confirmed" && !authorization) {
    blockedReasons.push("已确认方案缺少 C1 授权回链。");
  }
  if (proposal && authorization?.source_proposal_id !== proposal.proposal_id) {
    blockedReasons.push("C1 授权 source_proposal_id 与 C2 proposal 不匹配。");
  }
  if (authorization && !authorization.user_confirmation) {
    blockedReasons.push("C1 授权缺少用户确认。");
  }

  const canReview =
    Boolean(proposal && proposal.status === "user_confirmed" && authorization?.user_confirmation) &&
    authorization?.source_proposal_id === proposal?.proposal_id &&
    authorization?.status !== "active" &&
    review?.status !== "approved";
  const reviewStatus = review?.status as GlobalBoundaryReviewStatus | undefined;
  const statusLabel =
    authorization?.status === "active"
      ? "复核通过，授权有效"
      : reviewStatus && globalBoundaryReviewStatusLabels[reviewStatus]
        ? globalBoundaryReviewStatusLabels[reviewStatus]
        : proposal?.status === "user_confirmed"
          ? "待全局复核"
          : "未就绪";
  const displayText =
    authorization?.status === "active"
      ? "授权有效；仍未派发工作者"
      : reviewStatus === "needs_changes"
        ? "要求修改后不能自动推进"
        : reviewStatus === "blocked"
          ? "阻断方案后不能自动推进"
          : proposal?.status === "user_confirmed"
            ? "方案已由用户确认；等待全局主管复核"
            : "等待用户确认方案后进入全局复核";

  return {
    proposal,
    authorization,
    review,
    canReview,
    status_label: statusLabel,
    display_text: displayText,
    active_authorization_id: authorizationSummary.active_authorization_id ?? null,
    guard_status: guardSummary.status,
    guard_display_text: guardSummary.display_text,
    guard_reasons: guardSummary.reasons.slice(0, 3),
    blocked_reasons: blockedReasons.slice(0, 3),
    findings: (review?.findings ?? []).slice(0, 3),
  };
}

function planAuthorizationDisplayText(authorization: PlanAuthorization) {
  const status = planAuthorizationStatusLabels[authorization.status] ?? authorization.status;
  return `${status}；角色 ${authorization.scope.allowed_role_ids.length} / agent ${authorization.scope.allowed_agent_ids.length} / 读 ${authorization.scope.allowed_read_roots.length} / 写 ${authorization.scope.allowed_write_roots.length} / 工具 ${authorization.scope.allowed_tools.length} / 检查 ${authorization.scope.allowed_checks.length} / 停止条件 ${authorization.scope.stop_conditions.length}`;
}
