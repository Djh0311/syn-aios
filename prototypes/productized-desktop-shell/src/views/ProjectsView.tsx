import { useEffect, useMemo, useState } from "react";
import type {
  AutoDispatchGuardInput,
  AutoDispatchGuardResult,
  BlackboardCandidateStoreV1,
  CodexTranscript,
  FormalMemoryStoreV1,
  MemoryCandidateStoreV1,
  MemoryLintStoreV1,
  ObservationStoreV1,
  PendingAction,
  PlanAuthorizationStoreV1,
  PreviewProjectDirectorTaskPlanInput,
  ProjectConsultationProposalStoreV1,
  ProjectDirectorTaskPlan,
  ProjectRecord,
  ProjectWorkflowAutomationReadModel,
  RealExecutionProductCommandReadModel,
  RuntimeSessionAttention,
  SessionRecord,
  TaskPackageDispatchReadiness,
  TaskMemoryPacketBuildInput,
  TaskMemoryPacketBuildOutput,
  TaskPackagePreview,
  WorkflowRunCheck,
  WorkflowStateSnapshot,
} from "../lib/types";
import { AgentSessionCenter } from "./AgentView";
import { ProjectGallery } from "./projects/ProjectGallery";
import { ProjectWorkflowCanvasView } from "./projects/ProjectWorkflowCanvasView";
import { ProjectCanvasSidePanel, WorkflowRunCheckDetails } from "./projects/ProjectWorkflowSidePanel";
import {
  buildGlobalBoundaryReviewAction,
  buildPrepareAuthorizedAutoDispatchAction,
  buildProjectConsultationProposalCreationAction,
  buildProjectConsultationProposalDecisionAction,
  ProjectDirectorTaskPlanCard,
} from "./projects/ProjectWorkflowGovernancePanels";
import { WorkItemOrchestrationCard } from "./projects/ProjectWorkflowExecutionPanels";
import {
  ProjectWorkspaceShell,
  selectedTaskDraftFor,
  type ProjectDetailProps,
  type ProjectToolKey,
} from "./projects/ProjectWorkspaceShell";

export {
  TaskDispatchFieldCorrectionEditor,
  TaskDispatchFieldCorrectionShell,
  TaskDispatchReadinessController,
  TaskDispatchReadinessDetails,
  TaskDispatchReadinessShell,
  TaskFieldCorrectionPreview,
  TaskFileGenerationController,
  missingCorrectionFields,
  nextSelectedWorkItemId,
  selectedTaskDraftFor,
} from "./projects/ProjectWorkspaceShell";

export {
  buildGlobalBoundaryReviewAction,
  buildPrepareAuthorizedAutoDispatchAction,
  buildProjectConsultationProposalCreationAction,
  buildProjectConsultationProposalDecisionAction,
  ProjectDirectorTaskPlanCard,
  WorkflowRunCheckDetails,
  WorkItemOrchestrationCard,
};

type ProjectsViewProps = {
  projects: ProjectRecord[];
  sessions: SessionRecord[];
  workflowState?: WorkflowStateSnapshot | null;
  blackboardCandidateStore?: BlackboardCandidateStoreV1 | null;
  planAuthorizationStore?: PlanAuthorizationStoreV1 | null;
  projectConsultationProposalStore?: ProjectConsultationProposalStoreV1 | null;
  observationStore?: ObservationStoreV1 | null;
  memoryCandidateStore?: MemoryCandidateStoreV1 | null;
  formalMemoryStore?: FormalMemoryStoreV1 | null;
  memoryLintStore?: MemoryLintStoreV1 | null;
  runtimeSessionAttention?: RuntimeSessionAttention[];
  realExecutionProductCommands?: RealExecutionProductCommandReadModel | null;
  projectWorkflowAutomation?: ProjectWorkflowAutomationReadModel | null;
  k3B1Recovery?: ProjectDetailProps["k3B1Recovery"];
  workflowStateLoading?: boolean;
  workflowStateError?: string | null;
  onReloadWorkflowState?: () => void;
  onRequestAction: (action: PendingAction) => void;
  onNotice?: (msg: string) => void;
  onLoadTranscript: (threadId: string) => Promise<CodexTranscript>;
  onRenderTaskPreview?: (projectRoot: string, workItemId: string) => Promise<TaskPackagePreview>;
  onInspectDispatchReadiness?: (projectRoot: string, workItemId: string) => Promise<TaskPackageDispatchReadiness>;
  onInspectWorkflowRunCheck?: (projectRoot: string, workflowId?: string | null) => Promise<WorkflowRunCheck>;
  onInspectAutoDispatchAuthorization?: (request: AutoDispatchGuardInput) => Promise<AutoDispatchGuardResult>;
  onPreviewTaskMemoryPacket?: (request: TaskMemoryPacketBuildInput) => Promise<TaskMemoryPacketBuildOutput>;
  onPreviewProjectDirectorTaskPlan?: (request: PreviewProjectDirectorTaskPlanInput) => Promise<ProjectDirectorTaskPlan>;
  taskMemoryPacketPreview?: TaskMemoryPacketBuildOutput | null;
  onOpenAgentSession?: (threadId: string) => void;
};

export function ProjectsView(props: ProjectsViewProps) {
  const { projects, sessions, workflowState = null } = props;
  const [selectedRoot, setSelectedRoot] = useState<string | null>(null);
  const [selectedTool, setSelectedTool] = useState<ProjectToolKey>("workflow");
  const selectedProject = selectedRoot ? projects.find((project) => project.project_root === selectedRoot) ?? null : null;
  const projectSessions = useMemo(
    () => (selectedProject ? filterProjectSessionsForProject(sessions, selectedProject) : []),
    [sessions, selectedProject],
  );

  useEffect(() => {
    if (selectedRoot && !projects.some((project) => project.project_root === selectedRoot)) {
      setSelectedRoot(null);
    }
  }, [projects, selectedRoot]);

  useEffect(() => {
    setSelectedTool("workflow");
  }, [selectedProject?.project_root]);

  if (!projects.length) {
    return (
      <section className="stage-pad source-placeholder">
        <div className="sr-only">
          <h1>项 目 入 口</h1>
          <p>0 项目 · 0 会话；普通浏览器没有 Tauri 数据桥；这里不能假装有项目</p>
        </div>
        <div className="stat-strip">
          <div className="stat-cell">
            <div className="lbl">项目</div>
            <div className="val mono">0</div>
          </div>
          <div className="stat-cell">
            <div className="lbl">工作流</div>
            <div className="val mono">0</div>
          </div>
          <div className="stat-cell">
            <div className="lbl">状态</div>
            <div className="val warn">未接真实数据</div>
          </div>
        </div>
        <section className="panel">
          <div className="panel-h">
            项目列表
            <span className="count">空</span>
          </div>
          <div className="card lit">
            <div className="c-head">
              <span className="c-title">没有项目索引</span>
              <span className="c-meta">边界保护</span>
            </div>
            <div className="c-body">当前静态索引没有提供项目，真实项目页需要 Tauri 窗口读取本地索引后展示。</div>
          </div>
        </section>
      </section>
    );
  }

  if (!selectedProject) {
    return (
      <ProjectGallery
        projects={projects}
        sessions={sessions}
        workflowState={workflowState}
        onSelectProject={(projectRoot) => setSelectedRoot(projectRoot)}
      />
    );
  }

  return (
    <section className="project-workbench">
      <article className="project-detail-shell">
        <ProjectDetail
          {...props}
          project={selectedProject}
          sessions={projectSessions}
          selectedTool={selectedTool}
          onSelectTool={setSelectedTool}
          onBackToGallery={() => setSelectedRoot(null)}
        />
      </article>
    </section>
  );
}

export function filterProjectSessionsForProject(sessions: SessionRecord[], project: ProjectRecord) {
  return sessions.filter((session) => session.project_root === project.project_root);
}

export function ProjectDetail(props: ProjectDetailProps) {
  const {
    project,
    sessions,
    workflowState = null,
    blackboardCandidateStore = null,
    planAuthorizationStore = null,
    projectConsultationProposalStore = null,
    observationStore = null,
    memoryCandidateStore = null,
    formalMemoryStore = null,
    memoryLintStore = null,
    runtimeSessionAttention = [],
    realExecutionProductCommands = null,
    projectWorkflowAutomation = null,
    onRequestAction,
    onNotice = () => {},
    onOpenAgentSession = () => {},
    onInspectWorkflowRunCheck,
    onInspectAutoDispatchAuthorization,
    onPreviewTaskMemoryPacket,
    onPreviewProjectDirectorTaskPlan,
    taskMemoryPacketPreview,
  } = props;

  return (
    <ProjectWorkspaceShell
      {...props}
      workflowPanel={
        <ProjectWorkflowCanvasView
          project={project}
          sessions={sessions}
          workflowState={workflowState}
          blackboardCandidateStore={blackboardCandidateStore}
          planAuthorizationStore={planAuthorizationStore}
          projectConsultationProposalStore={projectConsultationProposalStore}
          observationStore={observationStore}
          memoryCandidateStore={memoryCandidateStore}
          formalMemoryStore={formalMemoryStore}
          memoryLintStore={memoryLintStore}
          runtimeSessionAttention={runtimeSessionAttention}
          realExecutionProductCommands={realExecutionProductCommands}
          projectWorkflowAutomation={projectWorkflowAutomation}
          k3B1Recovery={props.k3B1Recovery ?? null}
          onRequestAction={onRequestAction}
          onNotice={onNotice}
          onOpenAgentSession={onOpenAgentSession}
          onInspectWorkflowRunCheck={onInspectWorkflowRunCheck}
          onInspectAutoDispatchAuthorization={onInspectAutoDispatchAuthorization}
          onPreviewTaskMemoryPacket={onPreviewTaskMemoryPacket}
          onPreviewProjectDirectorTaskPlan={onPreviewProjectDirectorTaskPlan}
          initialTaskMemoryPacketPreview={taskMemoryPacketPreview}
          renderSidePanel={(sidePanelProps) => <ProjectCanvasSidePanel {...sidePanelProps} />}
        />
      }
    />
  );
}

function ProjectAgentSessionsPanel({
  project,
  sessions,
  onLoadTranscript,
  focusedThreadId,
  onRequestAction,
}: {
  project: ProjectRecord;
  sessions: SessionRecord[];
  onLoadTranscript?: (threadId: string) => Promise<CodexTranscript>;
  focusedThreadId?: string | null;
  onRequestAction: (action: PendingAction) => void;
}) {
  const readableSessions = useMemo(
    () => sessions.filter((session) => session.rollout_exists && session.rollout_path),
    [sessions],
  );
  const [selectedThreadId, setSelectedThreadId] = useState<string | null>(readableSessions[0]?.thread_id ?? null);
  const [transcript, setTranscript] = useState<CodexTranscript | null>(null);
  const [loadingThreadId, setLoadingThreadId] = useState<string | null>(null);
  const [transcriptError, setTranscriptError] = useState<string | null>(null);

  useEffect(() => {
    if (selectedThreadId && readableSessions.some((session) => session.thread_id === selectedThreadId)) return;
    setSelectedThreadId(readableSessions[0]?.thread_id ?? null);
    setTranscript(null);
    setTranscriptError(null);
  }, [readableSessions, selectedThreadId]);

  useEffect(() => {
    if (!focusedThreadId || !sessions.some((session) => session.thread_id === focusedThreadId)) return;
    setSelectedThreadId(focusedThreadId);
    setTranscript(null);
    setTranscriptError(null);
  }, [focusedThreadId, sessions]);

  const selectedSession = sessions.find((session) => session.thread_id === selectedThreadId) ?? null;
  const projectSessionCount = new Set(sessions.map((session) => session.project_root).filter(Boolean)).size;

  async function openSession(session: SessionRecord) {
    setSelectedThreadId(session.thread_id);
    setTranscript(null);
    setTranscriptError(null);
    if (!onLoadTranscript) {
      setTranscriptError("当前运行环境没有接入会话记录读取入口。");
      return;
    }
    setLoadingThreadId(session.thread_id);
    try {
      const nextTranscript = await onLoadTranscript(session.thread_id);
      setTranscript(nextTranscript);
    } catch (error) {
      setTranscriptError(messageOf(error));
    } finally {
      setLoadingThreadId(null);
    }
  }

  return (
    <div className="project-agent-session-panel">
      <AgentSessionCenter
        scope="project"
        groupBy="software"
        showSoftwareLayer={false}
        eyebrow={`项目 · ${project.name}`}
        title="软 件 与 会 话"
        description={`项目内会话按软件层分组。当前项目：${project.name}`}
        emptyTitle="没有索引推断关联的会话"
        emptyMessage="当前项目还没有任何 Codex / Claude Code / OpenClaw 会话。"
        loadingThreadId={loadingThreadId}
        onOpenSession={(session) => void openSession(session)}
        onRequestAction={onRequestAction}
        projectSessionCount={projectSessionCount}
        selectedSession={selectedSession}
        selectedThreadId={selectedThreadId}
        sessions={sessions}
        transcript={transcript}
        transcriptError={transcriptError}
      />
    </div>
  );
}

function messageOf(error: unknown): string {
  if (error instanceof Error) return error.message;
  return String(error);
}
