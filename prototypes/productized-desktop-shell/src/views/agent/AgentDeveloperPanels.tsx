import type {
  AgentAdapterDescriptor,
  H2RealResumeAuthorizationReadiness,
  H2RealResumeExecutionDecisionSurface,
  ProjectRecord,
  ProjectWorkflowAutomationReadModel,
  ProviderAvailabilitySummary,
  RealExecutionProductCommandReadModel,
  RuntimeSessionAttention,
  SessionContinuationPreview,
  SessionContinuationStoreV1,
  SessionOperationDescriptor,
  SessionRecord,
  SessionRunStatusSummary,
  WorkerProtocolReadModel,
  WorkflowStateSnapshot,
} from "../../lib/types";
import {
  AgentAdapterCapabilityPanel,
  ProviderAvailabilityPanel,
  SessionOperationBoundaryPanel,
} from "./AgentAdapterBoundaryPanels";
import {
  AdapterSdkCliDiagnosticsPanel,
  ControlledSessionContinuationPanel,
  H2RealResumeAuthorizationPanel,
  H2RealResumeExecutionDecisionPanel,
  RuntimeSessionAttentionPanel,
  SessionContinuationPreviewPanel,
} from "./AgentContinuationBoundaryPanels";
import { CodexControlEntryPanel, UnifiedExecutionStatusPanel } from "./AgentExecutionPanels";

export function AgentDeveloperPanels({
  sessions,
  projects,
  selectedSession,
  realExecutionProductCommands,
  workflowState,
  h2RealResumeExecutionDecisionSurface,
  sessionContinuationStore,
  runtimeSessionAttention,
  sessionRunStatusSummaries,
  projectWorkflowAutomation,
  projectDispatchCount,
  projectAttemptCount,
  adapterDescriptors,
  providerAvailabilitySummaries,
  sessionContinuationPreviews,
  h2RealResumeAuthorizationReadiness,
  workerProtocol,
  sessionOperationDescriptors,
}: {
  sessions: SessionRecord[];
  projects: ProjectRecord[];
  selectedSession: SessionRecord | null;
  realExecutionProductCommands: RealExecutionProductCommandReadModel | null;
  workflowState: WorkflowStateSnapshot | null;
  h2RealResumeExecutionDecisionSurface: H2RealResumeExecutionDecisionSurface;
  sessionContinuationStore: SessionContinuationStoreV1 | null;
  runtimeSessionAttention: RuntimeSessionAttention[];
  sessionRunStatusSummaries: SessionRunStatusSummary[];
  projectWorkflowAutomation: ProjectWorkflowAutomationReadModel | null;
  projectDispatchCount: number;
  projectAttemptCount: number;
  adapterDescriptors: AgentAdapterDescriptor[];
  providerAvailabilitySummaries: ProviderAvailabilitySummary[];
  sessionContinuationPreviews: SessionContinuationPreview[];
  h2RealResumeAuthorizationReadiness: H2RealResumeAuthorizationReadiness;
  workerProtocol: WorkerProtocolReadModel | null;
  sessionOperationDescriptors: SessionOperationDescriptor[];
}) {
  return (
    <>
      <CodexControlEntryPanel
        sessions={sessions}
        projects={projects}
        selectedSession={selectedSession}
        realExecutionProductCommands={realExecutionProductCommands}
        workflowState={workflowState}
      />
      <UnifiedExecutionStatusPanel
        surface={h2RealResumeExecutionDecisionSurface}
        store={sessionContinuationStore}
        runtimeSessionAttention={runtimeSessionAttention}
        sessionRunStatusSummaries={sessionRunStatusSummaries}
        realExecutionProductCommands={realExecutionProductCommands}
        projectWorkflowAutomation={projectWorkflowAutomation}
        projectDispatchCount={projectDispatchCount}
        projectAttemptCount={projectAttemptCount}
      />
      <AgentAdapterCapabilityPanel descriptors={adapterDescriptors} />
      <ProviderAvailabilityPanel summaries={providerAvailabilitySummaries} />
      <SessionContinuationPreviewPanel previews={sessionContinuationPreviews} />
      <ControlledSessionContinuationPanel store={sessionContinuationStore} previews={sessionContinuationPreviews} />
      <H2RealResumeAuthorizationPanel readiness={h2RealResumeAuthorizationReadiness} />
      <H2RealResumeExecutionDecisionPanel surface={h2RealResumeExecutionDecisionSurface} />
      <RuntimeSessionAttentionPanel attention={runtimeSessionAttention} summaries={sessionRunStatusSummaries} />
      <AdapterSdkCliDiagnosticsPanel workerProtocol={workerProtocol} />
      <SessionOperationBoundaryPanel operations={sessionOperationDescriptors} />
    </>
  );
}
