import type { WorkflowResultSummaryReadModel, WorkflowStateSnapshot } from "../../src/lib/types";

interface C6ResultSummaryFixtureInput {
  workflowProjectId: string;
  workflowId: string;
  pendingWorkflowResultSummary: WorkflowResultSummaryReadModel;
  workflowStateWithDerivedWorkflow: WorkflowStateSnapshot;
}

export function c6ResultSummaryFixtures(input: C6ResultSummaryFixtureInput) {
  const { workflowProjectId, workflowId, pendingWorkflowResultSummary, workflowStateWithDerivedWorkflow } = input;

  const c6WorkflowResultSummary: WorkflowResultSummaryReadModel = {
    ...pendingWorkflowResultSummary,
    final_review_status: "accepted",
    final_review_id: "global-final-review:offline:001",
    user_decision_status: "accept_result",
    user_decision_id: "user-result-decision:offline:001",
    stage_c_acceptance: {
      project_id: workflowProjectId,
      workflow_id: workflowId,
      gates: [
        {
          gate_id: "c1-plan-authorization",
          label: "C1 方案授权",
          status: "passed",
          reason: "authorization plan-auth:offline:active / status active",
          evidence_refs: ["plan-auth:offline:active"],
        },
        {
          gate_id: "c2-user-confirmed-proposal",
          label: "C2 用户确认方案",
          status: "passed",
          reason: "proposal proposal:offline:001 / status user_confirmed",
          evidence_refs: ["proposal:offline:001"],
        },
        {
          gate_id: "c3-global-boundary-review",
          label: "C3 全局边界复核",
          status: "passed",
          reason: "authorization active 且 global boundary review 为 approved。",
          evidence_refs: ["plan-auth:offline:active"],
        },
        {
          gate_id: "c4-prepared-dispatch",
          label: "C4 项目主管拆任务 / prepared dispatch",
          status: "passed",
          reason: "task package artifact 和 prepared dispatch 记录存在。",
          evidence_refs: ["dispatch:offline:001", "artifact:offline:task-package:001"],
        },
        {
          gate_id: "c5-worker-report-process-fact",
          label: "C5 工作者汇报 / 过程事实确认",
          status: "passed",
          reason: "工作者汇报已由项目主管确认过程事实；观察仍不是正式记忆。",
          evidence_refs: ["report:dispatch:offline:001", "review:process-fact:offline:001"],
        },
        {
          gate_id: "c6-global-final-review",
          label: "C6 全局最终复核",
          status: "passed",
          reason: "全局主管最终复核不能代表用户已接受。",
          evidence_refs: ["global-final-review:offline:001"],
        },
        {
          gate_id: "c6-user-result-decision",
          label: "C6 用户结果决定",
          status: "passed",
          reason: "用户决定只适用于本次结果，不代表未来任务默认接受。",
          evidence_refs: ["user-result-decision:offline:001"],
        },
        {
          gate_id: "stage-c-deferred-real-worker",
          label: "后置：真实工作者 / Codex 执行",
          status: "deferred",
          reason: "C6 默认不执行真实工作者、codex exec 或 codex exec resume。",
          evidence_refs: [],
        },
      ],
      final_review_status: "accepted",
      user_decision_status: "accept_result",
      accepted_as_stage_c_complete: true,
      deferred_items: ["真实工作者 / Codex 执行仍需单独授权任务包。", "真实 Tauri 全面截图验收仍是后置项。"],
      open_blockers: [],
      warnings: ["stage_c_acceptance_does_not_complete_middle_version", "process_fact_observation_is_not_formal_memory"],
    },
    open_issues: [],
    deferred_items: ["真实工作者 / Codex 执行仍需单独授权任务包。", "真实 Tauri 全面截图验收仍是后置项。"],
    warnings: ["stage_c_acceptance_does_not_complete_middle_version", "process_fact_observation_is_not_formal_memory"],
  };

  const workflowStateWithC6ResultSummary: WorkflowStateSnapshot = {
    ...workflowStateWithDerivedWorkflow,
    project_workflows: [
      {
        ...workflowStateWithDerivedWorkflow.project_workflows[0],
        derived_workflow: {
          ...workflowStateWithDerivedWorkflow.project_workflows[0].derived_workflow!,
          review_results: [
            ...workflowStateWithDerivedWorkflow.project_workflows[0].derived_workflow!.review_results,
            {
              review_id: "review:process-fact:offline:001",
              workflow_id: workflowId,
              workflow_node_id: "workflow:offline-fixture-projects-codex-workbench:default:node:director",
              reviewer_role: "project_director",
              report_id: "report:dispatch:offline:001",
              accepted_fact_ids: ["process-fact:offline:001"],
              observation_ids: ["observation:process-fact:offline:001"],
              result: "process_fact_confirmed",
              summary: "项目主管确认 C5 过程事实；observation 仍不是正式记忆。",
              evidence_refs: ["evidence:offline:process-fact:001"],
              requires_director_confirmation: false,
              can_complete_node: false,
              warnings: ["observation_is_not_formal_memory"],
            },
          ],
          result_summary: c6WorkflowResultSummary,
        },
      },
    ],
  };

  return {
    c6WorkflowResultSummary,
    workflowStateWithC6ResultSummary,
  };
}
