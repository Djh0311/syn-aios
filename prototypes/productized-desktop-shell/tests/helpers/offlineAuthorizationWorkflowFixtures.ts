import { authorizationWorkflowClusterFixtures } from "./offlineAuthorizationWorkflowClusterFixtures";

export function authorizationWorkflowFixtures(
  projectRoot: string,
  sessionThreadId: string,
  workflowProjectId: string,
  workflowId: string,
) {
  return authorizationWorkflowClusterFixtures({
    projectRoot,
    sessionThreadId,
    workflowProjectId,
    workflowId,
  });
}
