import type {
  PendingAction,
  PreviewProjectDirectorTaskPlanInput,
  RecordGlobalBoundaryReviewInput,
  RecordProjectConsultationProposalDecisionInput,
} from "../../src/lib/types";

export const projectConsultationProposalDecisionSummary =
  "用户确认项目咨询方案范围；仍需全局主管复核后才可自动推进，本轮不会启动真实工作者。";

export const globalBoundaryReviewSummary = "全局主管复核通过方案边界；授权有效，仍未派发工作者。";

export function projectConsultationProposalDecisionPayloadFixture(
  projectRoot: string,
): RecordProjectConsultationProposalDecisionInput {
  return {
    project_root: projectRoot,
    proposal_id: "proposal:offline:c2:pending",
    actor_id: "user",
    decision: "confirm",
    summary: projectConsultationProposalDecisionSummary,
    expected_proposal_store_revision: 1,
    expected_plan_authorization_store_revision: 4,
  };
}

export function globalBoundaryReviewPayloadFixture(projectRoot: string): RecordGlobalBoundaryReviewInput {
  return {
    project_root: projectRoot,
    project_id: "project:offline-fixture-projects-codex-workbench",
    workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
    proposal_id: "proposal:offline:c3:confirmed",
    authorization_id: "plan-auth:offline:pending-global",
    actor_id: "global_director",
    review_status: "approved",
    summary: globalBoundaryReviewSummary,
    checklist: {
      architecture_boundary_checked: true,
      cross_project_impact_checked: true,
      permission_scope_checked: true,
      read_write_scope_checked: true,
      tool_and_check_scope_checked: true,
      memory_boundary_checked: true,
      stop_conditions_checked: true,
      acceptance_criteria_checked: true,
    },
    findings: [],
    expected_authorization_revision: 5,
  };
}

export function projectDirectorTaskPlanRequestFixture(
  projectRoot: string,
  expectedAuthorizationRevision: number,
): PreviewProjectDirectorTaskPlanInput {
  return {
    project_root: projectRoot,
    project_id: "project:offline-fixture-projects-codex-workbench",
    workflow_id: "workflow:offline-fixture-projects-codex-workbench:default",
    proposal_id: "proposal:offline:c4:confirmed",
    authorization_id: "plan-auth:offline:active",
    actor_id: "project_director",
    expected_authorization_revision: expectedAuthorizationRevision,
  };
}

export function directorReviewActionFixture(projectRoot: string): PendingAction {
  return {
    kind: "record-director-review",
    label: "记录总指导回收：接受",
    path: projectRoot,
    source: "索引内项目路径",
    boundary:
      "只写真实 workflow-state.v0.json 的复核记录和审计事件；不启动 Codex、不恢复会话、不发送消息、不写 /Users/yoyi/.codex、不读取完整会话记录。",
    directorReview: {
      project_root: projectRoot,
      work_item_id: "work-item:offline:001",
      dispatch_id: "dispatch:offline:001",
      decision: "accepted",
      summary: "总指导回收：接受；派发结果：WORKFLOW_NODE_DISPATCH_OK_2026_05_29",
    },
  };
}
