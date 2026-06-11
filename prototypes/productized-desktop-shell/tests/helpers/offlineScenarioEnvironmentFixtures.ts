import { summarizePlanAuthorizationStore } from "../../src/lib/planAuthorization";
import { summarizeProjectConsultationProposalStore } from "../../src/lib/projectConsultationProposal";
import type { WorkflowStateSnapshot } from "../../src/lib/types";
import { authorizationWorkflowFixtures } from "./offlineAuthorizationWorkflowFixtures";
import { c6ResultSummaryFixtures } from "./offlineC6ResultSummaryFixtures";
import { derivedWorkflowStateFixtures } from "./offlineDerivedWorkflowFixtures";
import { buildNotReadyDispatchReadiness } from "./offlineTaskFieldTestUtils";
import { workbenchBaseFixtures } from "./offlineWorkbenchBaseFixtures";
import { projectWorkflowStateFixtures } from "./offlineProjectWorkflowStateFixtures";
import {
  workflowStateReadyForReviewFixture,
  workflowStateWithCompletedOfflineDispatchFixture,
  workflowStateWithGeneratedTaskFileFixture,
  workflowStateWithPreparedOfflineDispatchFixture,
} from "./offlineWorkflowStateVariantFixtures";

type ProjectWorkflowFixture = WorkflowStateSnapshot["project_workflows"][number];
type TaskDraftFixture = ProjectWorkflowFixture["task_drafts"][number];

export function offlineScenarioEnvironmentFixtures() {
  const base = workbenchBaseFixtures();
  const { project, session, workflowId, workflowProjectId } = base;

  const { workflowState, workflowStateWithProjectWorkflow } = projectWorkflowStateFixtures(project.project_root, session);
  const authorization = authorizationWorkflowFixtures(project.project_root, session.thread_id, workflowProjectId, workflowId);

  const planAuthorizationSummary = summarizePlanAuthorizationStore(
    authorization.planAuthorizationStore,
    workflowProjectId,
    workflowId,
  );
  const projectConsultationProposalSummary = summarizeProjectConsultationProposalStore(
    authorization.projectConsultationProposalStoreActive,
    authorization.planAuthorizationStore,
    workflowProjectId,
    workflowId,
  );

  const { pendingWorkflowResultSummary, workflowStateWithDerivedWorkflow } = derivedWorkflowStateFixtures({
    projectRoot: project.project_root,
    sessionThreadId: session.thread_id,
    workflowProjectId,
    workflowId,
    workflowStateWithProjectWorkflow,
  });
  const { workflowStateWithC6ResultSummary } = c6ResultSummaryFixtures({
    workflowProjectId,
    workflowId,
    pendingWorkflowResultSummary,
    workflowStateWithDerivedWorkflow,
  });

  const workflowStateReadyForReview: WorkflowStateSnapshot =
    workflowStateReadyForReviewFixture(workflowStateWithProjectWorkflow);
  const workflowStateWithPreparedOfflineDispatch: WorkflowStateSnapshot = workflowStateWithPreparedOfflineDispatchFixture(
    workflowStateWithProjectWorkflow,
    project.project_root,
  );
  const workflowStateWithCompletedOfflineDispatch: WorkflowStateSnapshot = workflowStateWithCompletedOfflineDispatchFixture(
    workflowStateWithProjectWorkflow,
    project.project_root,
  );
  const workflowStateWithGeneratedTaskFile: WorkflowStateSnapshot =
    workflowStateWithGeneratedTaskFileFixture(workflowStateWithProjectWorkflow);
  const notReadyDispatchReadiness = buildNotReadyDispatchReadiness(project.project_root);

  return {
    ...base,
    ...authorization,
    workflowState,
    workflowStateWithProjectWorkflow,
    planAuthorizationSummary,
    projectConsultationProposalSummary,
    pendingWorkflowResultSummary,
    workflowStateWithDerivedWorkflow,
    workflowStateWithC6ResultSummary,
    workflowStateReadyForReview,
    workflowStateWithPreparedOfflineDispatch,
    workflowStateWithCompletedOfflineDispatch,
    workflowStateWithGeneratedTaskFile,
    notReadyDispatchReadiness,
  };
}

export function preparedProjectWorkflowFixture(
  projectWorkflow: ProjectWorkflowFixture,
  selectedTask: TaskDraftFixture,
): ProjectWorkflowFixture {
  return {
    ...projectWorkflow,
    permission_requests: [],
    execution_attempts: [],
    task_drafts: [{ ...selectedTask, state: "prepared" }],
    node_dispatches: [
      {
        ...projectWorkflow.node_dispatches[0],
        state: "prepared",
        last_message_summary: null,
        transcript_event_count: null,
        transcript_target_hits: null,
        warnings: ["prepared_dispatch_is_not_worker_execution"],
      },
    ],
  };
}
