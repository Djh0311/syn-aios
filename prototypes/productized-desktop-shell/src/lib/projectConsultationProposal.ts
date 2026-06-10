import type {
  PlanAuthorization,
  PlanAuthorizationStoreV1,
  ProjectConsultationProposal,
  ProjectConsultationProposalDecision,
  ProjectConsultationProposalStatus,
  ProjectConsultationProposalStoreV1,
} from "./types";

export const projectConsultationProposalStatusLabels: Record<ProjectConsultationProposalStatus, string> = {
  draft: "草案",
  pending_user_confirmation: "待用户确认",
  user_confirmed: "用户已确认，待全局复核",
  changes_requested: "用户要求修改",
  rejected: "用户已拒绝",
  superseded: "已被新方案取代",
};

export type ProjectConsultationProposalSummary = {
  sidecar_name: "project-proposals.v1.json";
  revision: number;
  project_id: string;
  workflow_id: string;
  proposal_count: number;
  latest_proposal: ProjectConsultationProposal | null;
  latest_decision: ProjectConsultationProposalDecision | null;
  linked_plan_authorization: PlanAuthorization | null;
  authorization_missing_after_confirmation: boolean;
  status_label: string;
  display_text: string;
  warnings: string[];
};

export function summarizeProjectConsultationProposalStore(
  proposalStore: ProjectConsultationProposalStoreV1 | null,
  authorizationStore: PlanAuthorizationStoreV1 | null,
  projectId?: string | null,
  workflowId?: string | null,
): ProjectConsultationProposalSummary {
  const safeProjectId = projectId ?? "";
  const safeWorkflowId = workflowId ?? "";
  if (!proposalStore || !safeProjectId || !safeWorkflowId) {
    return {
      sidecar_name: "project-proposals.v1.json",
      revision: proposalStore?.revision ?? 0,
      project_id: safeProjectId,
      workflow_id: safeWorkflowId,
      proposal_count: 0,
      latest_proposal: null,
      latest_decision: null,
      linked_plan_authorization: null,
      authorization_missing_after_confirmation: false,
      status_label: "未建立",
      display_text: "还没有项目咨询方案草案",
      warnings: proposalStore?.warnings ?? ["project_consultation_proposal_store_not_loaded"],
    };
  }

  const proposals = proposalStore.proposals.filter(
    (proposal) => proposal.project_id === safeProjectId && proposal.workflow_id === safeWorkflowId,
  );
  const latestProposal = proposals.at(-1) ?? null;
  const latestDecision = latestProposal
    ? proposalStore.decisions.filter((decision) => decision.proposal_id === latestProposal.proposal_id).at(-1) ?? null
    : null;
  const linkedAuthorization = latestProposal?.plan_authorization_id
    ? authorizationStore?.authorizations.find(
        (authorization) => authorization.authorization_id === latestProposal.plan_authorization_id,
      ) ?? null
    : null;
  const authorizationMissingAfterConfirmation =
    latestProposal?.status === "user_confirmed" && !linkedAuthorization;
  const statusLabel = latestProposal
    ? projectConsultationProposalStatusLabels[latestProposal.status] ?? latestProposal.status
    : "未建立";
  const displayText = latestProposal
    ? `${statusLabel}；步骤 ${latestProposal.proposed_steps.length} / 风险 ${latestProposal.risks.length} / 停止条件 ${latestProposal.scope_draft.stop_conditions.length}`
    : "还没有项目咨询方案草案";
  const warnings = [
    ...(proposalStore.warnings ?? []),
    ...(authorizationMissingAfterConfirmation ? ["proposal_confirmed_but_authorization_missing"] : []),
  ];

  return {
    sidecar_name: "project-proposals.v1.json",
    revision: proposalStore.revision,
    project_id: safeProjectId,
    workflow_id: safeWorkflowId,
    proposal_count: proposals.length,
    latest_proposal: latestProposal,
    latest_decision: latestDecision,
    linked_plan_authorization: linkedAuthorization,
    authorization_missing_after_confirmation: authorizationMissingAfterConfirmation,
    status_label: statusLabel,
    display_text: displayText,
    warnings,
  };
}
