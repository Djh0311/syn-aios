import { useCallback, useEffect, useMemo, useRef, useState } from "react";
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
import { messageOf } from "./agent/agentLabels";
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
  onLoadTranscript?: (threadId: string) => Promise<CodexTranscript>;
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
  onLoadTranscript,
  onRequestAction = () => {},
}: AgentViewProps) {
  const softwareCounts = useMemo(() => {
    const map = new Map<string, number>();
    for (const s of sessions) {
      const key = softwareKeyOf(s);
      map.set(key, (map.get(key) ?? 0) + 1);
    }
    return Array.from(map.entries()).map(([key, count]) => ({ key, label: softwareLabelOf(key), count }));
  }, [sessions]);
  const multiSoftware = softwareCounts.length > 1;
  const [softwareFilter, setSoftwareFilter] = useState<string | null>(null);

  const filteredSessions = useMemo(
    () => (softwareFilter ? sessions.filter((session) => softwareKeyOf(session) === softwareFilter) : sessions),
    [sessions, softwareFilter],
  );
  const readableSessions = useMemo(
    () => filteredSessions.filter((session) => session.rollout_exists && session.rollout_path),
    [filteredSessions],
  );
  const [selectedThreadId, setSelectedThreadId] = useState<string | null>(readableSessions[0]?.thread_id ?? null);
  const [transcript, setTranscript] = useState<CodexTranscript | null>(null);
  const [loadingThreadId, setLoadingThreadId] = useState<string | null>(null);
  const [transcriptError, setTranscriptError] = useState<string | null>(null);

  useEffect(() => {
    if (!focusedThreadId) return;
    const focusedSession = sessions.find((session) => session.thread_id === focusedThreadId);
    if (!focusedSession) return;
    setSoftwareFilter(null);
    setSelectedThreadId(focusedThreadId);
  }, [focusedThreadId, sessions]);

  useEffect(() => {
    if (selectedThreadId && filteredSessions.some((session) => session.thread_id === selectedThreadId)) return;
    setSelectedThreadId(readableSessions[0]?.thread_id ?? null);
  }, [filteredSessions, readableSessions, selectedThreadId]);

  const selectedSession = filteredSessions.find((session) => session.thread_id === selectedThreadId) ?? null;
  const adapterDescriptors = useMemo(
    () =>
      backendAdapterDescriptors.length
        ? backendAdapterDescriptors
        : deriveAgentAdapterDescriptors({ sessions, projects, workflowState }),
    [backendAdapterDescriptors, sessions, projects, workflowState],
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

  const loadTranscript = useCallback(
    async (threadId: string) => {
      setTranscript(null);
      setTranscriptError(null);
      if (!onLoadTranscript) {
        setTranscriptError("当前运行环境没有接入会话记录读取入口。");
        return;
      }
      setLoadingThreadId(threadId);
      try {
        const nextTranscript = await onLoadTranscript(threadId);
        setTranscript((current) => (threadId === selectedThreadIdRef.current ? nextTranscript : current));
      } catch (error) {
        setTranscriptError((current) => (threadId === selectedThreadIdRef.current ? messageOf(error) : current));
      } finally {
        setLoadingThreadId((current) => (current === threadId ? null : current));
      }
    },
    [onLoadTranscript],
  );

  // Keep a ref so an in-flight load can tell if its thread is still the selected one.
  const selectedThreadIdRef = useRef<string | null>(selectedThreadId);
  useEffect(() => {
    selectedThreadIdRef.current = selectedThreadId;
  }, [selectedThreadId]);

  function openSession(session: SessionRecord) {
    if (session.thread_id === selectedThreadId) {
      // Already selected — re-read on demand (used by the reader's reload button).
      if (session.rollout_exists && session.rollout_path) void loadTranscript(session.thread_id);
      return;
    }
    setSelectedThreadId(session.thread_id);
  }

  const filterBar = multiSoftware ? (
    <div className="session-filter-bar" role="group" aria-label="按软件筛选会话">
      <button
        className={`filter-chip ${softwareFilter === null ? "active" : ""}`}
        type="button"
        onClick={() => setSoftwareFilter(null)}
      >
        全部 <em>{sessions.length}</em>
      </button>
      {softwareCounts.map((row) => (
        <button
          className={`filter-chip ${softwareFilter === row.key ? "active" : ""}`}
          key={row.key}
          type="button"
          onClick={() => setSoftwareFilter(row.key)}
        >
          {row.label} <em>{row.count}</em>
        </button>
      ))}
    </div>
  ) : null;

  return (
    <section className="view-stack agent-view-root">
      <AgentSessionCenter
        sessions={filteredSessions}
        selectedThreadId={selectedThreadId}
        selectedSession={selectedSession}
        transcript={transcript}
        loadingThreadId={loadingThreadId}
        transcriptError={transcriptError}
        projectSessionCount={0}
        projects={projects}
        scope="global"
        groupBy="project"
        embedded
        showSoftwareLayer={false}
        filterBar={filterBar}
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
        developerDetails={
          <AgentDeveloperPanels
            sessions={sessions}
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
        description="选择项目和对话，继续处理任务。"
        emptyTitle="选择左侧会话开始阅读"
        emptyMessage="点任意会话即可查看你与 Agent 的对话。"
        onOpenSession={(session) => void openSession(session)}
        onRequestAction={onRequestAction}
      />
    </section>
  );
}
