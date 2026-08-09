// ⑥ H 定稿(hifi `H · 智能体页(会话中心·回顾面 B1 同构)`)：左会话列表(搜索+项目分组+三元素行)+
// 右 transcript；composer 上一行常显沙箱与写根。**开发者 11 面板全部退场**(定稿原话：「→审计账本页」)。
//
// 退场 = 本页不再渲染，组件本体保留在 `agent/AgentDeveloperPanels.tsx`(见该文件顶注)——
// 归位到审计账本页是**另一个包**的事(本包禁止改 AuditLedgerView)，删了会让那次搬迁无处可搬。
// ⚠️ 因此当前这些开发者信息**在 App 里暂时没有落脚点**，见交付报告 forks。
import { useEffect, useMemo, useState } from "react";
import { deriveAgentAdapterDescriptors } from "../lib/adapterCapabilities";
import { deriveProviderAvailabilitySummaries } from "../lib/providerAvailability";
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
import { AgentSessionCenter, softwareKeyOf, softwareLabelOf } from "./agent/AgentConversationShell";
import { M3AcceptancePanel } from "./agent/M3AcceptancePanel";
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
  const [roleSessionProjectLocator, setRoleSessionProjectLocator] = useState(
    () => projects[0]?.project_root ?? sessions.find((session) => session.project_root)?.project_root ?? "",
  );
  const sessionPage = useAgentSessionPage(
    sessions,
    sessionSearchQuery,
    onLoadSessionPage,
    roleSessionProjectLocator,
  );
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
  useEffect(() => {
    // A SessionRecord may provide only a non-authoritative project routing
    // hint. The fixed Agent command still resolves every role/binding fact on
    // the server; this state never becomes a continuation identity.
    const nextLocator = selectedSession?.project_root?.trim()
      || projects[0]?.project_root?.trim()
      || "";
    if (nextLocator !== roleSessionProjectLocator) setRoleSessionProjectLocator(nextLocator);
  }, [projects, roleSessionProjectLocator, selectedSession?.project_root]);
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
  // 11 面板退场后随之无消费者的 derive，逐个核过才删：
  //   - sessionContinuationPreviews / h2RealResumeAuthorizationReadiness /
  //     h2RealResumeExecutionDecisionSurface / projectDispatchCount / projectAttemptCount
  //     → 全部只喂 AgentDeveloperPanels，已删。
  // 保留的 adapterDescriptors / sessionOperationDescriptors / providerAvailabilitySummaries
  // **不能删**：它们经 AgentSessionCenter → deriveAgentsPageReadModelFromParts 喂 composer 的项目选择器
  // (AgentConversationShell.tsx:289-300)，删了会打断真实发送链路。
  // 它们的 derive 逻辑(lib/adapterCapabilities.ts 等)也照旧保留，只是不再经 11 面板上脸。

  function openSession(session: SessionRecord) {
    if (session.project_root?.trim()) setRoleSessionProjectLocator(session.project_root.trim());
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
      <M3AcceptancePanel host="agent" />
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
        workflowState={workflowState}
        sessionPageStatus={sessionPage.sessionPageStatus}
        sessionPageSource={sessionPage.sessionPageSource}
        sessionPageWarnings={sessionPage.sessionPageWarnings}
        roleSessionRead={sessionPage.roleSessionRead}
        onSelectRoleSession={sessionPage.selectRoleSession}
        onLoadMoreRoleSessions={sessionPage.loadMoreRoleSessions}
        sessionHasMore={sessionPage.sessionPageHasMore}
        loadingMoreSessions={sessionPage.loadingMoreSessions}
        eyebrow=""
        title="智能体"
        description="历史会话仅供阅读；续聊必须由服务端角色会话绑定重新授权。"
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
