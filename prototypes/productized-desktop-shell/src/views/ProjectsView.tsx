import { useEffect, useMemo, useRef, useState, type RefObject } from "react";
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
import type { SecretarySourceFocus, SecretarySourceFocusOutcome } from "../lib/types/m4Secretary";
import {
  M4_PROPOSAL_SOURCE_OWNER_REF,
  M4_WORK_ITEM_SOURCE_OWNER_REF,
} from "../lib/types/m4Secretary";
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
  secretarySourceFocus?: SecretarySourceFocus | null;
  onSecretarySourceFocusOutcome?: (outcome: SecretarySourceFocusOutcome) => void;
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
  hasRealSnapshot?: boolean;
  workflowStateLoading?: boolean;
  workflowStateError?: string | null;
  onReloadWorkflowState?: () => void;
  onWorkflowStateReadRefresh?: () => Promise<void>;
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
  // fix8：穿到交办面板 → 出方案成功刷店进批脸。{...props} spread 自动传给 Shell。
  onProposalStoreRefresh?: () => Promise<void>;
};

export type SecretarySourceProjectSelection =
  | Readonly<{ status: "PENDING" }>
  | Readonly<{ status: "READY"; project_root: string; tool: "task-packages" | "jiaoban" }>
  | Readonly<{
      status: "FAILED";
      error_code:
        | "SECRETARY_SOURCE_TARGET_PROJECT_MISSING"
        | "SECRETARY_SOURCE_TARGET_AMBIGUOUS"
        | "SECRETARY_SOURCE_TARGET_RECORD_MISSING";
    }>;

export function resolveSecretarySourceProjectSelection({
  focus,
  projects,
  workflowState,
  proposalStore,
  hasRealSnapshot,
  workflowStateLoading,
  workflowStateError,
}: {
  focus: SecretarySourceFocus;
  projects: ProjectRecord[];
  workflowState: WorkflowStateSnapshot | null;
  proposalStore: ProjectConsultationProposalStoreV1 | null;
  hasRealSnapshot: boolean;
  workflowStateLoading: boolean;
  workflowStateError: string | null;
}): SecretarySourceProjectSelection {
  const target = focus.target;
  const ownerBindingValid = target.kind === "WORK_ITEM"
    ? focus.source_owner_ref === M4_WORK_ITEM_SOURCE_OWNER_REF
      && focus.source_object_type === "workflow_attention"
      && focus.canonical_source_object_id === target.work_item_id
      && focus.source_revision === target.source_revision
    : focus.source_owner_ref === M4_PROPOSAL_SOURCE_OWNER_REF
      && focus.source_object_type === "proposal_decision"
      && focus.canonical_source_object_id === target.proposal_id
      && focus.source_revision === target.source_revision;
  if (!ownerBindingValid) {
    return Object.freeze({ status: "FAILED", error_code: "SECRETARY_SOURCE_TARGET_RECORD_MISSING" });
  }

  if (
    !hasRealSnapshot
    || workflowStateLoading
    || (workflowState === null && workflowStateError === null)
  ) {
    return Object.freeze({ status: "PENDING" });
  }

  const workflows = workflowState?.project_workflows ?? [];
  const projectWorkflows = workflows.filter((workflow) => workflow.project_id === target.project_id);
  if (!projectWorkflows.length) {
    return Object.freeze({ status: "FAILED", error_code: "SECRETARY_SOURCE_TARGET_PROJECT_MISSING" });
  }
  const exactWorkflows = projectWorkflows.filter((workflow) => workflow.workflow_id === target.workflow_id);
  if (exactWorkflows.length > 1) {
    return Object.freeze({ status: "FAILED", error_code: "SECRETARY_SOURCE_TARGET_AMBIGUOUS" });
  }
  const workflow = exactWorkflows[0];
  if (!workflow) {
    return Object.freeze({ status: "FAILED", error_code: "SECRETARY_SOURCE_TARGET_RECORD_MISSING" });
  }
  const exactProjects = projects.filter((project) => project.project_root === workflow.project_root);
  if (exactProjects.length > 1) {
    return Object.freeze({ status: "FAILED", error_code: "SECRETARY_SOURCE_TARGET_AMBIGUOUS" });
  }
  if (!exactProjects.length) {
    return Object.freeze({ status: "FAILED", error_code: "SECRETARY_SOURCE_TARGET_PROJECT_MISSING" });
  }

  if (target.kind === "WORK_ITEM") {
    const exactTasks = workflow.task_drafts.filter(
      (task) => task.workflow_id === target.workflow_id && task.work_item_id === target.work_item_id,
    );
    if (exactTasks.length !== 1) {
      return Object.freeze({
        status: "FAILED",
        error_code: exactTasks.length > 1
          ? "SECRETARY_SOURCE_TARGET_AMBIGUOUS"
          : "SECRETARY_SOURCE_TARGET_RECORD_MISSING",
      });
    }
    return Object.freeze({ status: "READY", project_root: workflow.project_root, tool: "task-packages" });
  }

  const exactProposals = (proposalStore?.proposals ?? []).filter(
    (proposal) => proposal.project_id === target.project_id
      && proposal.workflow_id === target.workflow_id
      && proposal.proposal_id === target.proposal_id,
  );
  if (exactProposals.length !== 1) {
    return Object.freeze({
      status: "FAILED",
      error_code: exactProposals.length > 1
        ? "SECRETARY_SOURCE_TARGET_AMBIGUOUS"
        : "SECRETARY_SOURCE_TARGET_RECORD_MISSING",
      });
  }
  return Object.freeze({ status: "READY", project_root: workflow.project_root, tool: "jiaoban" });
}

export function ProjectsView(props: ProjectsViewProps) {
  const {
    projects,
    sessions,
    workflowState = null,
    hasRealSnapshot = false,
    workflowStateLoading = false,
    workflowStateError = null,
    secretarySourceFocus = null,
    onSecretarySourceFocusOutcome,
    projectConsultationProposalStore = null,
  } = props;
  const [selectedRoot, setSelectedRoot] = useState<string | null>(null);
  const [selectedTool, setSelectedTool] = useState<ProjectToolKey>("jiaoban");
  const previousSelectedProjectRoot = useRef<string | null>(null);
  const appliedSecretaryFocusAttempt = useRef<number | null>(null);
  const reportedSecretaryFocusFailure = useRef<number | null>(null);
  const secretaryFocusSelection = useMemo(
    () => secretarySourceFocus
      ? resolveSecretarySourceProjectSelection({
          focus: secretarySourceFocus,
          projects,
          workflowState,
          proposalStore: projectConsultationProposalStore,
          hasRealSnapshot,
          workflowStateLoading,
          workflowStateError,
        })
      : null,
    [
      hasRealSnapshot,
      projectConsultationProposalStore,
      projects,
      secretarySourceFocus,
      workflowState,
      workflowStateError,
      workflowStateLoading,
    ],
  );
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
    const nextRoot = selectedProject?.project_root ?? null;
    const projectChanged = previousSelectedProjectRoot.current !== nextRoot;
    previousSelectedProjectRoot.current = nextRoot;
    if (secretarySourceFocus || !projectChanged) return;
    setSelectedTool("jiaoban");
  }, [secretarySourceFocus, selectedProject?.project_root]);

  useEffect(() => {
    if (!secretarySourceFocus || !secretaryFocusSelection) return;
    if (secretaryFocusSelection.status === "PENDING") return;
    if (secretaryFocusSelection.status === "FAILED") {
      if (reportedSecretaryFocusFailure.current === secretarySourceFocus.attempt_id) return;
      reportedSecretaryFocusFailure.current = secretarySourceFocus.attempt_id;
      onSecretarySourceFocusOutcome?.({
        attempt_id: secretarySourceFocus.attempt_id,
        source_route_ref: secretarySourceFocus.source_route_ref,
        target_kind: secretarySourceFocus.target.kind,
        status: "FAILED",
        error_code: secretaryFocusSelection.error_code,
      });
      return;
    }
    if (appliedSecretaryFocusAttempt.current === secretarySourceFocus.attempt_id) return;
    appliedSecretaryFocusAttempt.current = secretarySourceFocus.attempt_id;
    setSelectedRoot(secretaryFocusSelection.project_root);
    setSelectedTool(secretaryFocusSelection.tool);
  }, [onSecretarySourceFocusOutcome, secretaryFocusSelection, secretarySourceFocus]);

  if (secretarySourceFocus && secretaryFocusSelection?.status === "PENDING") {
    return (
      <section
        className="stage-pad source-placeholder"
        data-secretary-source-focus-status="PENDING"
        data-secretary-source-owner={secretarySourceFocus.source_owner_ref}
        data-secretary-source-object-type={secretarySourceFocus.source_object_type}
        data-secretary-source-object-id={secretarySourceFocus.canonical_source_object_id}
        data-secretary-source-revision={secretarySourceFocus.source_revision}
        data-secretary-source-route-ref={secretarySourceFocus.source_route_ref}
        aria-busy="true"
      >
        <h1>正在读取来源负责模块</h1>
        <p>索引、工作流与方案读面完成后，将继续定位这条精确记录。</p>
      </section>
    );
  }

  if (secretarySourceFocus && secretaryFocusSelection?.status === "FAILED") {
    return (
      <section
        className="stage-pad source-placeholder"
        data-secretary-source-focus-status="FAILED"
        data-secretary-source-focus-error-code={secretaryFocusSelection.error_code}
        data-secretary-source-owner={secretarySourceFocus.source_owner_ref}
        data-secretary-source-object-type={secretarySourceFocus.source_object_type}
        data-secretary-source-object-id={secretarySourceFocus.canonical_source_object_id}
        data-secretary-source-revision={secretarySourceFocus.source_revision}
        data-secretary-source-route-ref={secretarySourceFocus.source_route_ref}
        role="alert"
      >
        <h1>来源记录未定位</h1>
        <p>{secretaryFocusSelection.error_code}</p>
      </section>
    );
  }

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
          secretarySourceFocus={secretarySourceFocus}
          onSecretarySourceFocusOutcome={onSecretarySourceFocusOutcome}
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

type ProjectDetailCanvasEditing = {
  canvasEditing: boolean;
  onCanvasBack: () => void;
  onEditingChange: (editing: boolean) => void;
  exitEditingRef: RefObject<(() => void) | null>;
};

export function ProjectDetail(props: ProjectDetailProps) {
  // 离线/SSR：findElement(findButtonByText) 会把组件当普通函数调用，组件内 hooks 触发 "Invalid hook call"。
  // 无 window 时走不调 hooks 的渲染（不接编辑态提升；顶部按钮恒「返回项目」，离线不测编辑切换）。
  if (typeof window === "undefined") {
    return renderProjectDetailShell(props, null);
  }
  // 画布编辑态提到这里：顶部「返回项目」在编辑态切成「返回」（点它退出编辑回工作流运行界面）。
  const [canvasEditing, setCanvasEditing] = useState(false);
  const exitCanvasEditingRef = useRef<(() => void) | null>(null);
  return renderProjectDetailShell(props, {
    canvasEditing,
    onCanvasBack: () => exitCanvasEditingRef.current?.(),
    onEditingChange: setCanvasEditing,
    exitEditingRef: exitCanvasEditingRef,
  });
}

function renderProjectDetailShell(props: ProjectDetailProps, editing: ProjectDetailCanvasEditing | null) {
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

  const renderWorkflowCanvas = (embedded = false) => (
    <ProjectWorkflowCanvasView
      project={project}
      sessions={sessions}
      workflowState={workflowState}
      onReloadWorkflowState={props.onReloadWorkflowState}
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
      onEditingChange={embedded ? undefined : editing?.onEditingChange}
      exitEditingRef={embedded ? undefined : editing?.exitEditingRef}
      embedded={embedded}
    />
  );

  return (
    <ProjectWorkspaceShell
      {...props}
      canvasEditing={editing?.canvasEditing}
      onCanvasBack={editing?.onCanvasBack}
      workflowPanel={renderWorkflowCanvas()}
      jiaobanWorkflowPanel={renderWorkflowCanvas(true)}
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
