// ⚠️ 暂时无 import 者 —— **这不是死码，别删**(2026-07-15 ⑥ H 包)。
//
// 定稿(`docs/design/2026-07-14-stage-b-hifi-fullapp-v1.html` H 段)：开发者 11 面板从智能体页
// **退场 → 审计账本页**。⑥ H 包只做得了「退场」这一半：本包明令禁止改 `AuditLedgerView`，
// 所以「归位」由后续包做，届时直接从这里搬。现在删掉 = 那次搬迁无处可搬、要重写 ~1300 行。
//
// 现状：AgentView 已不再渲染本组件；这些开发者信息在 App 里**暂时没有落脚点**(见 ⑥ 交付报告 forks)。
// 覆盖也没丢：它们吃的 read model(adapterCapabilities / h2RealResumeAuthorization /
// sessionContinuation / providerAvailability / sessionOperations)的语义断言仍由
// RunningWorkflowsView / RightDetailPanel 的既有断言 + 纯函数断言锁着。
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
