import type { WorkflowStateSnapshot } from "../../src/lib/types";
import { projectBlackboardFixture } from "./offlineProjectBlackboardFixtures";
import { derivedProjectWorkflowFixture } from "./offlineDerivedProjectWorkflowFixtures";

interface DerivedWorkflowStateFixtureInput {
  projectRoot: string;
  sessionThreadId: string;
  workflowProjectId: string;
  workflowId: string;
  workflowStateWithProjectWorkflow: WorkflowStateSnapshot;
}

export function derivedWorkflowStateFixtures(input: DerivedWorkflowStateFixtureInput) {
  const { projectRoot, sessionThreadId, workflowProjectId, workflowId, workflowStateWithProjectWorkflow } = input;
  const { pendingWorkflowResultSummary, derivedWorkflow } = derivedProjectWorkflowFixture({
    projectRoot,
    sessionThreadId,
    workflowProjectId,
    workflowId,
  });

  const workflowStateWithDerivedWorkflow: WorkflowStateSnapshot = {
    ...workflowStateWithProjectWorkflow,
    project_blackboards: [projectBlackboardFixture(projectRoot)],
    project_workflows: [
      {
        ...workflowStateWithProjectWorkflow.project_workflows[0],
        derived_workflow: derivedWorkflow,
      },
    ],
  };

  return {
    pendingWorkflowResultSummary,
    workflowStateWithDerivedWorkflow,
  };
}
