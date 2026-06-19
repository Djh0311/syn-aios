import { useEffect, useMemo, useState } from "react";
import { deriveAgentAdapterDescriptors } from "../lib/adapterCapabilities";
import {
  deriveH2RealResumeAuthorizationReadiness,
  deriveH2RealResumeExecutionDecisionSurface,
} from "../lib/h2RealResumeAuthorization";
import { deriveProviderAvailabilitySummaries } from "../lib/providerAvailability";
import { deriveSessionContinuationPreviews } from "../lib/sessionContinuation";
import { deriveSessionOperationDescriptors } from "../lib/sessionOperations";
import type {
  AgentAdapterDescriptor,
  CodexTranscript,
  CodexTranscriptPageRequest,
  CodexSessionPage,
  CodexSessionPageRequest,
  PendingAction,
  ProviderAvailabilitySummary,
  ProjectRecord,
  RuntimeSessionAttention,
  RealExecutionProductCommandReadModel,
  ProjectWorkflowAutomationReadModel,
  SessionRecord,
  SessionContinuationPreview,
  SessionContinuationStoreV1,
  SessionRunStatusSummary,
  SessionOperationDescriptor,
  WorkerProtocolReadModel,
  WorkflowStateSnapshot,
} from "../lib/types";
import { AgentDeveloperPanels } from "./agent/AgentDeveloperPanels";
import { AgentSessionCenter, softwareKeyOf, softwareLabelOf } from "./agent/AgentConversationShell";
import { AgentSoftwareFilterBar } from "./agent/AgentSoftwareFilterBar";
import { useAgentSessionPage } from "./agent/useAgentSessionPage";
import { useAgentTranscriptLoader } from "./agent/useAgentTranscriptLoader";
export { ChatTranscript, TranscriptTimeline } from "./agent/TranscriptViews";
export { AgentSessionCenter, filterAgentSessions, softwareGroupsForSessions } from "./agent/AgentConversationShell";

type AgentViewProps = {
  sessions: SessionRecord[];
  projects?: ProjectRecord[];
  adapterDescriptors?: AgentAdapterDescriptor[];
  sessionOperationDescriptors?: SessionOperationDescriptor[];
  providerAvailabilitySummaries?: ProviderAvailabilitySummary[];
  sessionContinuationPreviews?: SessionContinuationPreview[];
  sessionContinuationStore?: SessionContinuationStoreV1 | null;
  runtimeSessionAttention?: RuntimeSessionAttention[];
  sessionRunStatusSummaries?: SessionRunStatusSummary[];
  realExecutionProductCommands?: RealExecutionProductCommandReadModel | null;
  projectWorkflowAutomation?: ProjectWorkflowAutomationReadModel | null;
  workerProtocol?: WorkerProtocolReadModel | null;
  workflowState?: WorkflowStateSnapshot | null;
  focusedThreadId?: string | null;
  onLoadSessionPage?: (request: CodexSessionPageRequest) => Promise<CodexSessionPage>;
  onLoadTranscript?: (threadId: string) => Promise<CodexTranscript>;
  onLoadTranscriptPage?: (request: CodexTranscriptPageRequest) => Promise<CodexTranscript>;
  onRequestAction?: (action: PendingAction) => void;
};

export function AgentView({
  sessions,
  projects = [],
  adapterDescriptors: backendAdapterDescriptors = [],
  sessionOperationDescriptors: backendSessionOperationDescriptors = [],
  providerAvailabilitySummaries: backendProviderAvailabilitySummaries = [],
  sessionContinuationPreviews: backendSessionContinuationPreviews = [],
  sessionContinuationStore = null,
  runtimeSessionAttention = [],
  sessionRunStatusSummaries = [],
  realExecutionProductCommands = null,
  projectWorkflowAutomation = null,
  workerProtocol = null,
  workflowState = null,
  focusedThreadId = null,
  onLoadSessionPage,
  onLoadTranscript,
  onLoadTranscriptPage,
  onRequestAction = () => {},
}: AgentViewProps) {
  const [sessionSearchQuery, setSessionSearchQuery] = useState("");
  const sessionPage = useAgentSessionPage(sessions, sessionSearchQuery, onLoadSessionPage);
  const { shellSessions } = sessionPage;
  const softwareCounts = useMemo(() => {
    const map = new Map<string, number>();
    for (const s of shellSessions) {
      const key = softwareKeyOf(s);
      map.set(key, (map.get(key) ?? 0) + 1);
    }
    return Array.from(map.entries()).map(([key, count]) => ({ key, label: softwareLabelOf(key), count }));
  }, [shellSessions]);
  const [softwareFilter, setSoftwareFilter] = useState<string | null>(null);

  const filteredSessions = useMemo(
    () => (softwareFilter ? shellSessions.filter((session) => softwareKeyOf(session) === softwareFilter) : shellSessions),
    [shellSessions, softwareFilter],
  );
  const readableSessions = useMemo(
    () => filteredSessions.filter((session) => session.rollout_exists && session.rollout_path),
    [filteredSessions],
  );
  const [selectedThreadId, setSelectedThreadId] = useState<string | null>(readableSessions[0]?.thread_id ?? null);

  useEffect(() => {
    if (!focusedThreadId) return;
    const focusedSession = shellSessions.find((session) => session.thread_id === focusedThreadId);
    if (!focusedSession) return;
    setSoftwareFilter(null);
    setSelectedThreadId(focusedThreadId);
  }, [focusedThreadId, shellSessions]);

  useEffect(() => {
    if (selectedThreadId && filteredSessions.some((session) => session.thread_id === selectedThreadId)) return;
    setSelectedThreadId(readableSessions[0]?.thread_id ?? null);
  }, [filteredSessions, readableSessions, selectedThreadId]);

  const selectedSession = filteredSessions.find((session) => session.thread_id === selectedThreadId) ?? null;
  const { loadingOlderThreadId, loadingThreadId, loadOlderTranscript, loadTranscript, selectedTranscript, transcriptError } = useAgentTranscriptLoader({
    onLoadTranscript,
    onLoadTranscriptPage,
    selectedSession,
  });
  const adapterDescriptors = useMemo(
    () =>
      backendAdapterDescriptors.length
        ? backendAdapterDescriptors
        : deriveAgentAdapterDescriptors({ sessions: shellSessions, projects, workflowState }),
    [backendAdapterDescriptors, shellSessions, projects, workflowState],
  );
  const sessionOperationDescriptors = useMemo(
    () =>
      backendSessionOperationDescriptors.length
        ? backendSessionOperationDescriptors
        : deriveSessionOperationDescriptors(adapterDescriptors),
    [backendSessionOperationDescriptors, adapterDescriptors],
  );
  const providerAvailabilitySummaries = useMemo(
    () =>
      backendProviderAvailabilitySummaries.length
        ? backendProviderAvailabilitySummaries
        : deriveProviderAvailabilitySummaries(adapterDescriptors, sessionOperationDescriptors),
    [backendProviderAvailabilitySummaries, adapterDescriptors, sessionOperationDescriptors],
  );
  const sessionContinuationPreviews = useMemo(
    () =>
      backendSessionContinuationPreviews.length
        ? backendSessionContinuationPreviews
        : deriveSessionContinuationPreviews({
            adapterDescriptors,
            sessionOperationDescriptors,
            providerAvailabilitySummaries,
            workflowState,
          }),
    [
      backendSessionContinuationPreviews,
      adapterDescriptors,
      sessionOperationDescriptors,
      providerAvailabilitySummaries,
      workflowState,
    ],
  );
  const h2RealResumeAuthorizationReadiness = useMemo(
    () =>
      deriveH2RealResumeAuthorizationReadiness({
        previews: sessionContinuationPreviews,
        store: sessionContinuationStore,
      }),
    [sessionContinuationPreviews, sessionContinuationStore],
  );
  const h2RealResumeExecutionDecisionSurface = useMemo(
    () =>
      deriveH2RealResumeExecutionDecisionSurface({
        previews: sessionContinuationPreviews,
        store: sessionContinuationStore,
      }),
    [sessionContinuationPreviews, sessionContinuationStore],
  );
  const projectDispatchCount =
    workflowState?.project_workflows.reduce((count, workflow) => count + workflow.node_dispatches.length, 0) ?? 0;
  const projectAttemptCount =
    workflowState?.project_workflows.reduce((count, workflow) => count + workflow.execution_attempts.length, 0) ?? 0;

  function openSession(session: SessionRecord) {
    if (session.thread_id === selectedThreadId) {
      // Already selected — re-read on demand (used by the reader's reload button).
      if (session.rollout_exists && session.rollout_path) void loadTranscript(session.thread_id);
      return;
    }
    setSelectedThreadId(session.thread_id);
  }

  async function handleNewSessionThreadStarted(threadId: string) {
    setSoftwareFilter(null);
    setSessionSearchQuery(threadId);
    const page = await sessionPage.loadSessionPage(0, "replace", threadId);
    if (page?.sessions.some((session) => session.thread_id === threadId)) {
      setSelectedThreadId(threadId);
    }
  }

  return (
    <section className="view-stack agent-view-root">
      <AgentSessionCenter
        sessions={filteredSessions}
        selectedThreadId={selectedThreadId}
        selectedSession={selectedSession}
        transcript={selectedTranscript}
        loadingThreadId={loadingThreadId}
        loadingOlderThreadId={loadingOlderThreadId}
        transcriptError={transcriptError}
        projectSessionCount={0}
        projects={projects}
        scope="global"
        groupBy="project"
        embedded
        showSoftwareLayer={false}
        filterBar={
          <AgentSoftwareFilterBar
            activeKey={softwareFilter}
            counts={softwareCounts}
            total={shellSessions.length}
            onChange={setSoftwareFilter}
          />
        }
        adapterDescriptors={adapterDescriptors}
        sessionOperationDescriptors={sessionOperationDescriptors}
        providerAvailabilitySummaries={providerAvailabilitySummaries}
        sessionContinuationPreviews={sessionContinuationPreviews}
        sessionContinuationStore={sessionContinuationStore}
        runtimeSessionAttention={runtimeSessionAttention}
        sessionRunStatusSummaries={sessionRunStatusSummaries}
        realExecutionProductCommands={realExecutionProductCommands}
        projectWorkflowAutomation={projectWorkflowAutomation}
        workerProtocol={workerProtocol}
        workflowState={workflowState}
        sessionPageStatus={sessionPage.sessionPageStatus}
        sessionPageSource={sessionPage.sessionPageSource}
        sessionPageWarnings={sessionPage.sessionPageWarnings}
        sessionHasMore={sessionPage.sessionPageHasMore}
        loadingMoreSessions={sessionPage.loadingMoreSessions}
        developerDetails={
          <AgentDeveloperPanels
            sessions={shellSessions}
            projects={projects}
            selectedSession={selectedSession}
            realExecutionProductCommands={realExecutionProductCommands}
            workflowState={workflowState}
            h2RealResumeExecutionDecisionSurface={h2RealResumeExecutionDecisionSurface}
            sessionContinuationStore={sessionContinuationStore}
            runtimeSessionAttention={runtimeSessionAttention}
            sessionRunStatusSummaries={sessionRunStatusSummaries}
            projectWorkflowAutomation={projectWorkflowAutomation}
            projectDispatchCount={projectDispatchCount}
            projectAttemptCount={projectAttemptCount}
            adapterDescriptors={adapterDescriptors}
            providerAvailabilitySummaries={providerAvailabilitySummaries}
            sessionContinuationPreviews={sessionContinuationPreviews}
            h2RealResumeAuthorizationReadiness={h2RealResumeAuthorizationReadiness}
            workerProtocol={workerProtocol}
            sessionOperationDescriptors={sessionOperationDescriptors}
          />
        }
        eyebrow=""
        title="智能体"
        description="新对话、搜索和会话分组在左栏；当前对话在中间继续。"
        emptyTitle="选择左侧会话开始阅读"
        emptyMessage="点任意会话即可查看你与 Agent 的对话。"
        onOpenSession={(session) => void openSession(session)}
        onLoadOlderTranscript={loadOlderTranscript}
        onLoadMoreSessions={() => void sessionPage.loadSessionPage(sessionPage.sessionPageOffset, "append")}
        onNewSessionThreadStarted={(threadId) => void handleNewSessionThreadStarted(threadId)}
        onReadFilterChange={sessionPage.setSessionPageReadFilter}
        onSearchQueryChange={setSessionSearchQuery}
        onRequestAction={onRequestAction}
      />
    </section>
  );
}
