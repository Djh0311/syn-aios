import { useCallback, useEffect, useMemo, useState, type ReactNode } from "react";
import {
  Background,
  Controls,
  Handle,
  MarkerType,
  MiniMap,
  Position,
  ReactFlow,
  ReactFlowProvider,
  type Edge,
  type Node,
  type NodeProps,
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";
import { Badge } from "../../components/Badge";
import { DetailLine } from "../../components/WorkbenchPrimitives";
import {
  buildBlackboardCandidateOverlay,
  summarizeFormalMemoryStore,
  summarizeMemoryCandidateStore,
  summarizeMemoryLintStore,
  summarizeObservationStore,
} from "../../lib/candidateGovernance";
import { summarizePlanAuthorizationStore } from "../../lib/planAuthorization";
import { summarizeProjectConsultationProposalStore } from "../../lib/projectConsultationProposal";
import {
  deriveProjectWorkflowCanvasReadModel,
  type ProjectCanvasEdge,
  type ProjectCanvasNode,
  type ProjectCanvasStatus,
  type ProjectWorkflowCanvasReadModel,
} from "../../lib/projectCanvas";
import type { CanvasSurfaceBoundary } from "../../lib/canvasSurfaceBoundaries";
import { projectCanvasSurfaceConfig } from "../../lib/canvasSurfaceConfig";
import {
  canvasLoad,
  canvasSave,
  executeProjectWorkflowNode,
  getProjectWorkflowNodes,
  listProjectWorkflows,
  submitProjectWorkflowDraft,
} from "../../lib/tauri";
import type { CanvasDefinition, CanvasEdge, CanvasNode, ProjectWorkflowListItem } from "../../lib/types";
import { WorkflowCanvasEngine } from "../WorkflowCanvasEngine";
import type {
  AutoDispatchGuardInput,
  AutoDispatchGuardResult,
  BlackboardCandidateStoreV1,
  FormalMemoryStoreV1,
  K3B1RecoveryReadModel,
  MemoryCandidateStoreV1,
  MemoryLintStoreV1,
  ObservationStoreV1,
  PendingAction,
  PlanAuthorizationStoreV1,
  PreviewProjectDirectorTaskPlanInput,
  ProjectBlackboard,
  ProjectConsultationProposalStoreV1,
  ProjectDirectorTaskPlan,
  ProjectRecord,
  ProjectWorkflowAutomationReadModel,
  RealExecutionProductCommandReadModel,
  RuntimeSessionAttention,
  SessionRecord,
  TaskDraftSummary,
  TaskMemoryPacketBuildInput,
  TaskMemoryPacketBuildOutput,
  TaskPackage,
  WorkflowRunCheck,
  WorkflowStateSnapshot,
} from "../../lib/types";
import { selectedTaskDraftFor } from "./ProjectWorkspaceShell";

export type ProjectWorkflowCanvasSidePanelProps = {
  canvasModel: ProjectWorkflowCanvasReadModel;
  selectedNodeId: string;
  project: ProjectRecord;
  projectId: string;
  sessions: SessionRecord[];
  projectWorkflow: WorkflowStateSnapshot["project_workflows"][number] | null;
  derivedWorkflow: NonNullable<WorkflowStateSnapshot["project_workflows"][number]["derived_workflow"]> | null;
  selectedTask: TaskDraftSummary | null;
  selectedTaskPackage: TaskPackage | null;
  projectBlackboard: ProjectBlackboard | null;
  blackboardOverlay: ReturnType<typeof buildBlackboardCandidateOverlay>;
  observationSummary: ReturnType<typeof summarizeObservationStore>;
  observationStoreRevision: number;
  observations: ObservationStoreV1["observations"];
  memorySummary: ReturnType<typeof summarizeMemoryCandidateStore>;
  formalSummary: ReturnType<typeof summarizeFormalMemoryStore>;
  memoryLintSummary: ReturnType<typeof summarizeMemoryLintStore>;
  memoryLintFindings: MemoryLintStoreV1["findings"];
  projectConsultationProposalSummary: ReturnType<typeof summarizeProjectConsultationProposalStore>;
  planAuthorizationSummary: ReturnType<typeof summarizePlanAuthorizationStore>;
  projectDirectorTaskPlanRequest: PreviewProjectDirectorTaskPlanInput | null;
  projectDirectorTaskPlan: ProjectDirectorTaskPlan | null;
  projectDirectorTaskPlanLoading: boolean;
  projectDirectorTaskPlanError: string | null;
  onPreviewProjectDirectorTaskPlan: () => void;
  autoDispatchGuardResult: AutoDispatchGuardResult | null;
  autoDispatchGuardError: string | null;
  workflowRevision: number | null;
  blackboardStoreRevision: number;
  memoryStoreRevision: number;
  memoryCandidates: MemoryCandidateStoreV1["candidates"];
  runtimeSessionAttention: RuntimeSessionAttention[];
  realExecutionProductCommands: RealExecutionProductCommandReadModel | null;
  projectWorkflowAutomation: ProjectWorkflowAutomationReadModel | null;
  k3B1Recovery: K3B1RecoveryReadModel | null;
  taskMemoryPacketPreview: TaskMemoryPacketBuildOutput | null;
  taskMemoryPacketLoading: boolean;
  taskMemoryPacketError: string | null;
  onRequestAction: (action: PendingAction) => void;
  onOpenAgentSession: (threadId: string) => void;
  onInspectWorkflowRunCheck?: (projectRoot: string, workflowId?: string | null) => Promise<WorkflowRunCheck>;
};

type ProjectWorkflowCanvasViewProps = {
  project: ProjectRecord;
  sessions: SessionRecord[];
  workflowState: WorkflowStateSnapshot | null;
  onReloadWorkflowState?: () => void;
  blackboardCandidateStore: BlackboardCandidateStoreV1 | null;
  planAuthorizationStore: PlanAuthorizationStoreV1 | null;
  projectConsultationProposalStore: ProjectConsultationProposalStoreV1 | null;
  observationStore: ObservationStoreV1 | null;
  memoryCandidateStore: MemoryCandidateStoreV1 | null;
  formalMemoryStore: FormalMemoryStoreV1 | null;
  memoryLintStore: MemoryLintStoreV1 | null;
  runtimeSessionAttention: RuntimeSessionAttention[];
  realExecutionProductCommands?: RealExecutionProductCommandReadModel | null;
  projectWorkflowAutomation?: ProjectWorkflowAutomationReadModel | null;
  k3B1Recovery?: K3B1RecoveryReadModel | null;
  onRequestAction: (action: PendingAction) => void;
  onNotice?: (msg: string) => void;
  onOpenAgentSession: (threadId: string) => void;
  onInspectWorkflowRunCheck?: (projectRoot: string, workflowId?: string | null) => Promise<WorkflowRunCheck>;
  onInspectAutoDispatchAuthorization?: (request: AutoDispatchGuardInput) => Promise<AutoDispatchGuardResult>;
  onPreviewTaskMemoryPacket?: (request: TaskMemoryPacketBuildInput) => Promise<TaskMemoryPacketBuildOutput>;
  onPreviewProjectDirectorTaskPlan?: (request: PreviewProjectDirectorTaskPlanInput) => Promise<ProjectDirectorTaskPlan>;
  initialTaskMemoryPacketPreview?: TaskMemoryPacketBuildOutput | null;
  renderSidePanel: (props: ProjectWorkflowCanvasSidePanelProps) => ReactNode;
};

export function ProjectWorkflowCanvasView({
  project,
  sessions,
  workflowState,
  onReloadWorkflowState,
  blackboardCandidateStore,
  planAuthorizationStore,
  projectConsultationProposalStore,
  observationStore,
  memoryCandidateStore,
  formalMemoryStore,
  memoryLintStore,
  runtimeSessionAttention,
  realExecutionProductCommands,
  projectWorkflowAutomation,
  k3B1Recovery,
  onRequestAction,
  onNotice = () => {},
  onOpenAgentSession,
  onInspectWorkflowRunCheck,
  onInspectAutoDispatchAuthorization,
  onPreviewTaskMemoryPacket,
  onPreviewProjectDirectorTaskPlan,
  initialTaskMemoryPacketPreview,
  renderSidePanel,
}: ProjectWorkflowCanvasViewProps) {
  // P3 E · 多工作流底座（架构 §12）：项目存 N 个工作流，列表/选择器 + 新建/编辑。
  const [workflows, setWorkflows] = useState<ProjectWorkflowListItem[]>([]);
  const [selectedWorkflowId, setSelectedWorkflowId] = useState<string | null>(null);
  // 渲染跟随选择器（§12）：选中哪个工作流就渲染哪个；未选/找不到 → 回退该项目首个（= 旧行为）。
  const projectWorkflowsForProject =
    workflowState?.project_workflows.filter((workflow) => workflow.project_root === project.project_root) ?? [];
  const projectWorkflow =
    (selectedWorkflowId
      ? projectWorkflowsForProject.find((workflow) => workflow.workflow_id === selectedWorkflowId)
      : undefined) ??
    projectWorkflowsForProject[0] ??
    null;
  const selectedTask = selectedTaskDraftFor(projectWorkflow?.task_drafts ?? [], null);
  const derivedWorkflow = projectWorkflow?.derived_workflow ?? null;
  const selectedTaskPackage = selectedTaskPackageFor(derivedWorkflow?.task_packages ?? [], selectedTask);
  const projectBlackboard =
    workflowState?.project_blackboards?.find(
      (blackboard) =>
        blackboard.project_root === project.project_root &&
        (!projectWorkflow || blackboard.workflow_id === projectWorkflow.workflow_id),
    ) ?? null;
  const canvasModel = useMemo(
    () =>
      deriveProjectWorkflowCanvasReadModel({
        project,
        projectWorkflow,
        projectBlackboard,
        selectedTask,
        workflowStatePath: workflowState?.path ?? null,
        workflowStateUpdatedAt: workflowState?.updated_at ?? null,
        runtimeSessionAttention,
      }),
    [project, projectWorkflow, projectBlackboard, selectedTask, workflowState?.path, workflowState?.updated_at, runtimeSessionAttention],
  );
  const [selectedCanvasNodeId, setSelectedCanvasNodeId] = useState<string | null>(canvasModel.viewport_hint.selected_node_id);
  // P1/P2 项目面（两面一引擎，2026-06-21 真机反馈版）：默认是只读运行状态视图（保留既有治理
  // 落地页）。编辑是「动作」不是「视图」——点「编辑工作流 / 新建工作流」才进编辑态、挂引擎改
  // 草案（原工作流继续跑），离线/SSR 不渲染引擎（React Flow 无法 SSR）。统一草案流：不分空闲/
  // 在跑，一律改草案 → 提交 → 通过（运行性 / 控制核心·权限·审计，P3 重档）才生效。
  const [editing, setEditing] = useState(false);
  const [editingMode, setEditingMode] = useState<"edit" | "new">("edit");
  const projectConfig = useMemo(() => projectCanvasSurfaceConfig(project.project_root), [project.project_root]);
  const projectCanvasId = useMemo(
    () =>
      `project-${
        project.project_root.replace(/^\/+/, "").replace(/[^a-zA-Z0-9]+/g, "-").replace(/^-|-$/g, "").toLowerCase() ||
        "unknown"
      }`,
    [project.project_root],
  );
  // 新建工作流改独立草案画布（不动现有草案）；编辑工作流改项目草案画布。
  const draftCanvasId = editingMode === "new" ? `${projectCanvasId}-draft-new` : projectCanvasId;
  const refreshWorkflows = useCallback(async () => {
    try {
      const list = await listProjectWorkflows(project.project_root);
      setWorkflows(list);
      setSelectedWorkflowId((cur) =>
        cur && list.some((w) => w.workflow_id === cur)
          ? cur
          : (list.find((w) => w.is_default) ?? list[0])?.workflow_id ?? null,
      );
    } catch {
      // 列不出（无 workflow-state 等）→ 静默留空，项目页其余仍可用。
    }
  }, [project.project_root]);
  useEffect(() => {
    void refreshWorkflows();
  }, [refreshWorkflows]);

  // 提交草案 → 真写回 workflow-state（§12）：新建=create（workflow_id 空、不覆盖谁）、编辑=update
  // （带 selectedWorkflowId）。经后端运行性检查「通过」+ 控制核心 + 审计；非测试项目仍被后端挡。
  // 读引擎已保存的草案（引擎是手动「保存」，故提交前需先保存）。
  const submitDraft = useCallback(async () => {
    try {
      const draft = await canvasLoad(draftCanvasId);
      if (!draft || draft.nodes.length === 0) {
        onNotice("草案为空或未保存；请先在画布上编辑并点「保存」，再提交。");
        return;
      }
      const result = await submitProjectWorkflowDraft({
        project_root: project.project_root,
        workflow_id: editingMode === "new" ? null : selectedWorkflowId,
        title: draft.display_name?.trim() || (editingMode === "new" ? "新工作流" : "工作流"),
        nodes: draft.nodes,
        edges: draft.edges,
      });
      onNotice(result?.message ?? "已提交为项目工作流。");
      setEditing(false);
      await refreshWorkflows();
      onReloadWorkflowState?.(); // 后置：提交后刷新画布快照（否则新建/改的工作流只进下拉、画布快照仍旧 → 选它回退默认）
    } catch (e) {
      onNotice(`提交失败：${messageOf(e)}`);
    }
  }, [draftCanvasId, editingMode, selectedWorkflowId, project.project_root, onNotice, refreshWorkflows, onReloadWorkflowState]);

  // 新建工作流：先把 draft-new 画布清空（不覆盖谁）→ 编辑 → 提交走 create。
  const openNewWorkflow = useCallback(async () => {
    const now = new Date().toISOString();
    try {
      await canvasSave({
        schema_version: "canvas-v1",
        canvas_id: `${projectCanvasId}-draft-new`,
        display_name: `新工作流-${Date.now().toString(36).slice(-4)}`,
        project_root: project.project_root,
        scope: "project",
        nodes: [],
        edges: [],
        created_at: now,
        updated_at: now,
        warnings: [],
      });
    } catch {
      // 清空失败不致命；引擎仍会加载该 id。
    }
    setEditingMode("new");
    setEditing(true);
  }, [projectCanvasId, project.project_root]);

  // 编辑工作流：把选中工作流的现有 nodes 加载进草案（§12「要补」——否则提交=空白覆盖）→ 编辑 → update。
  const openEditWorkflow = useCallback(async () => {
    const wid = selectedWorkflowId;
    if (!wid) {
      onNotice("先选一个工作流再编辑。");
      return;
    }
    try {
      const seed = await getProjectWorkflowNodes(project.project_root, wid);
      const now = new Date().toISOString();
      const seeded: CanvasDefinition = {
        schema_version: "canvas-v1",
        canvas_id: projectCanvasId,
        display_name: workflows.find((w) => w.workflow_id === wid)?.title ?? "工作流",
        project_root: project.project_root,
        scope: "project",
        nodes: seed.nodes as CanvasNode[],
        edges: seed.edges as CanvasEdge[],
        created_at: now,
        updated_at: now,
        warnings: [],
      };
      await canvasSave(seeded);
      setEditingMode("edit");
      setEditing(true);
    } catch (e) {
      onNotice(`加载工作流到草案失败：${messageOf(e)}`);
    }
  }, [selectedWorkflowId, project.project_root, projectCanvasId, workflows, onNotice]);
  const [runningProjectNode, setRunningProjectNode] = useState(false);
  const selectedProjectNode = useMemo(
    () =>
      canvasModel.nodes.find(
        (node) => node.node_id === (selectedCanvasNodeId ?? canvasModel.viewport_hint.selected_node_id),
      ) ?? null,
    [canvasModel, selectedCanvasNodeId],
  );
  // P3 项目面真跑（架构方案 §9 的 C 映射）：选中只读运行态里一个节点 → 派发它的 work_item。
  // 节点 = workflow-state work_item 本体（无手绑）；后端从任务包构造指令、用节点既有会话绑定
  // resume。前端不判闸——非固定测试项目仍被后端 path-lock 挡下、零执行。
  const runSelectedProjectNode = useCallback(async () => {
    const node = selectedProjectNode;
    if (!node?.workflow_node_id) {
      onNotice("请先选中一个节点再运行");
      return;
    }
    setRunningProjectNode(true);
    try {
      // 后置C#2：work_item_id 可空——画布建的工作流节点没预存 work_item，后端会自动建临时 work_item
      // 并用节点载荷里的 resume 会话现绑再派发。
      const result = await executeProjectWorkflowNode({
        project_root: project.project_root,
        node_id: node.workflow_node_id,
        work_item_id: node.work_item_id ?? "",
        workflow_id: selectedWorkflowId,
      });
      onNotice(`已派发项目节点「${node.title}」。返回：${compactRunResult(result)}`);
      onReloadWorkflowState?.(); // 后置：运行后刷新画布快照（派发/绑定记录变了）
    } catch (e) {
      onNotice(`运行被拦截或失败：${messageOf(e)}`);
    } finally {
      setRunningProjectNode(false);
    }
  }, [selectedProjectNode, project.project_root, onNotice, onReloadWorkflowState]);
  const blackboardOverlay = useMemo(
    () =>
      buildBlackboardCandidateOverlay({
        store: blackboardCandidateStore,
        entries: projectBlackboard?.entries ?? [],
      }),
    [blackboardCandidateStore, projectBlackboard?.entries],
  );
  const memorySummary = useMemo(() => summarizeMemoryCandidateStore(memoryCandidateStore), [memoryCandidateStore]);
  const observationSummary = useMemo(() => summarizeObservationStore(observationStore), [observationStore]);
  const formalSummary = useMemo(() => summarizeFormalMemoryStore(formalMemoryStore), [formalMemoryStore]);
  const memoryLintSummary = useMemo(() => summarizeMemoryLintStore(memoryLintStore), [memoryLintStore]);
  const planAuthorizationSummary = useMemo(
    () => summarizePlanAuthorizationStore(planAuthorizationStore, projectWorkflow?.project_id, projectWorkflow?.workflow_id),
    [planAuthorizationStore, projectWorkflow?.project_id, projectWorkflow?.workflow_id],
  );
  const projectConsultationProposalSummary = useMemo(
    () =>
      summarizeProjectConsultationProposalStore(
        projectConsultationProposalStore,
        planAuthorizationStore,
        projectWorkflow?.project_id,
        projectWorkflow?.workflow_id,
      ),
    [projectConsultationProposalStore, planAuthorizationStore, projectWorkflow?.project_id, projectWorkflow?.workflow_id],
  );
  const [autoDispatchGuardResult, setAutoDispatchGuardResult] = useState<AutoDispatchGuardResult | null>(null);
  const [autoDispatchGuardError, setAutoDispatchGuardError] = useState<string | null>(null);
  const [taskMemoryPacketPreview, setTaskMemoryPacketPreview] = useState<TaskMemoryPacketBuildOutput | null>(
    initialTaskMemoryPacketPreview ?? null,
  );
  const [taskMemoryPacketLoading, setTaskMemoryPacketLoading] = useState(false);
  const [taskMemoryPacketError, setTaskMemoryPacketError] = useState<string | null>(null);
  const projectDirectorTaskPlanRequest = useMemo(
    () =>
      buildProjectDirectorTaskPlanRequest({
        project,
        projectWorkflow,
        proposalSummary: projectConsultationProposalSummary,
        authorizationSummary: planAuthorizationSummary,
      }),
    [project, projectWorkflow, projectConsultationProposalSummary, planAuthorizationSummary],
  );
  const [projectDirectorTaskPlan, setProjectDirectorTaskPlan] = useState<ProjectDirectorTaskPlan | null>(null);
  const [projectDirectorTaskPlanLoading, setProjectDirectorTaskPlanLoading] = useState(false);
  const [projectDirectorTaskPlanError, setProjectDirectorTaskPlanError] = useState<string | null>(null);

  useEffect(() => {
    setSelectedCanvasNodeId((current) =>
      current && canvasModel.nodes.some((node) => node.node_id === current)
        ? current
        : canvasModel.viewport_hint.selected_node_id,
    );
  }, [canvasModel]);

  useEffect(() => {
    if (initialTaskMemoryPacketPreview) {
      setTaskMemoryPacketPreview(initialTaskMemoryPacketPreview);
      setTaskMemoryPacketError(null);
    }
  }, [initialTaskMemoryPacketPreview]);

  async function refreshProjectDirectorTaskPlan() {
    if (!projectDirectorTaskPlanRequest) {
      setProjectDirectorTaskPlan(null);
      setProjectDirectorTaskPlanError("等待用户确认方案和全局边界复核通过后才能生成拆任务草案。");
      return;
    }
    if (!onPreviewProjectDirectorTaskPlan) {
      setProjectDirectorTaskPlan(null);
      setProjectDirectorTaskPlanError("当前运行环境没有接入项目主管拆任务预览入口。");
      return;
    }
    setProjectDirectorTaskPlanLoading(true);
    setProjectDirectorTaskPlanError(null);
    try {
      setProjectDirectorTaskPlan(await onPreviewProjectDirectorTaskPlan(projectDirectorTaskPlanRequest));
    } catch (previewError) {
      setProjectDirectorTaskPlan(null);
      setProjectDirectorTaskPlanError(messageOf(previewError));
    } finally {
      setProjectDirectorTaskPlanLoading(false);
    }
  }

  useEffect(() => {
    setProjectDirectorTaskPlan(null);
    setProjectDirectorTaskPlanError(null);
    if (!projectDirectorTaskPlanRequest || !onPreviewProjectDirectorTaskPlan) {
      setProjectDirectorTaskPlanLoading(false);
      return;
    }

    let cancelled = false;
    setProjectDirectorTaskPlanLoading(true);
    void onPreviewProjectDirectorTaskPlan(projectDirectorTaskPlanRequest)
      .then((plan) => {
        if (!cancelled) {
          setProjectDirectorTaskPlan(plan);
        }
      })
      .catch((previewError) => {
        if (!cancelled) {
          setProjectDirectorTaskPlan(null);
          setProjectDirectorTaskPlanError(messageOf(previewError));
        }
      })
      .finally(() => {
        if (!cancelled) {
          setProjectDirectorTaskPlanLoading(false);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [onPreviewProjectDirectorTaskPlan, projectDirectorTaskPlanRequest]);

  useEffect(() => {
    if (initialTaskMemoryPacketPreview) return;
    if (!onPreviewTaskMemoryPacket || !projectWorkflow || !selectedTask) {
      setTaskMemoryPacketPreview(null);
      setTaskMemoryPacketError(null);
      setTaskMemoryPacketLoading(false);
      return;
    }

    const request = buildTaskMemoryPacketRequest({
      projectRoot: project.project_root,
      projectId: projectWorkflow.project_id,
      workflowId: projectWorkflow.workflow_id,
      selectedTask,
      selectedTaskPackage,
      formalStoreRevision: formalMemoryStore?.revision ?? null,
      candidateStoreRevision: memoryCandidateStore?.revision ?? null,
      observationStoreRevision: observationStore?.revision ?? null,
    });
    let cancelled = false;
    setTaskMemoryPacketLoading(true);
    setTaskMemoryPacketError(null);
    void onPreviewTaskMemoryPacket(request)
      .then((output) => {
        if (!cancelled) {
          setTaskMemoryPacketPreview(output);
        }
      })
      .catch((previewError) => {
        if (!cancelled) {
          setTaskMemoryPacketPreview(null);
          setTaskMemoryPacketError(messageOf(previewError));
        }
      })
      .finally(() => {
        if (!cancelled) {
          setTaskMemoryPacketLoading(false);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [
    initialTaskMemoryPacketPreview,
    onPreviewTaskMemoryPacket,
    project.project_root,
    projectWorkflow,
    selectedTask,
    selectedTaskPackage,
    formalMemoryStore?.revision,
    memoryCandidateStore?.revision,
    observationStore?.revision,
  ]);

  useEffect(() => {
    if (!onInspectAutoDispatchAuthorization || !projectWorkflow || !selectedTask) {
      setAutoDispatchGuardResult(null);
      setAutoDispatchGuardError(null);
      return;
    }
    const request = buildAutoDispatchGuardInput({
      projectWorkflow,
      selectedTask,
      selectedTaskPackage,
    });
    let cancelled = false;
    setAutoDispatchGuardError(null);
    void onInspectAutoDispatchAuthorization(request)
      .then((result) => {
        if (!cancelled) setAutoDispatchGuardResult(result);
      })
      .catch((error) => {
        if (!cancelled) {
          setAutoDispatchGuardResult(null);
          setAutoDispatchGuardError(messageOf(error));
        }
      });
    return () => {
      cancelled = true;
    };
  }, [onInspectAutoDispatchAuthorization, projectWorkflow, selectedTask, selectedTaskPackage]);

  return (
    <section className="workflow-canvas" aria-label="项目级工作流画布">
      {projectConfig.showProjectRuleBar ? (
        <ProjectRuleStatusBar canvasModel={canvasModel} runCheckStatus={derivedWorkflow?.run_check_status ?? null} />
      ) : null}

      {editing ? (
        <>
          <div className="workflow-orchestration-head project-canvas-edit-head">
            <div>
              <p className="eyebrow">{editingMode === "new" ? "新建项目工作流 · 草案" : "编辑项目工作流 · 草案"}</p>
              <h3>
                {editingMode === "new"
                  ? "新建工作流（草案）"
                  : projectWorkflow
                    ? `编辑：${projectWorkflow.title}（草案）`
                    : "编辑工作流（草案）"}
              </h3>
              <p className="path-text">改的是草案，运行中的工作流不动；提交并通过后才生效（经控制核心 / 权限 / 审计）。</p>
            </div>
            <div className="workflow-state-actions">
              <button className="primary-button" type="button" onClick={() => void submitDraft()}>
                {editingMode === "new" ? "提交为新工作流" : "提交更新"}
              </button>
              <button className="secondary-button" type="button" onClick={() => setEditing(false)}>返回运行状态</button>
            </div>
          </div>
          <div className="project-canvas-plan" aria-label="项目工作流草案（可编辑）">
            <ReactFlowProvider>
              <WorkflowCanvasEngine
                config={projectConfig}
                canvasId={draftCanvasId}
                sessions={sessions}
                onNotice={onNotice}
              />
            </ReactFlowProvider>
          </div>
        </>
      ) : (
      <>
      <div className="workflow-orchestration-head">
        <div>
          <p className="eyebrow">项目工作流主入口</p>
          <h3>{projectWorkflow ? projectWorkflow.title : "当前项目还没有默认工作流"}</h3>
          <p className="path-text">{project.project_root}</p>
        </div>
        <div className="workflow-state-actions">
          {workflows.length > 0 ? (
            <select
              aria-label="选择工作流"
              value={selectedWorkflowId ?? ""}
              onChange={(e) => setSelectedWorkflowId(e.target.value || null)}
            >
              {workflows.map((w) => (
                <option key={w.workflow_id} value={w.workflow_id}>
                  {`${w.title || "(未命名)"}${w.is_default ? "（默认）" : ""} · ${w.node_count} 节点`}
                </option>
              ))}
            </select>
          ) : null}
          <button className="secondary-button" type="button" onClick={() => void openNewWorkflow()}>
            新建工作流
          </button>
          <button
            className="secondary-button"
            type="button"
            onClick={() => void openEditWorkflow()}
            disabled={!selectedWorkflowId}
            title={selectedWorkflowId ? "把选中工作流加载进草案编辑" : "先选一个工作流"}
          >
            编辑工作流
          </button>
          <button
            className="primary-button"
            type="button"
            onClick={() => void runSelectedProjectNode()}
            disabled={runningProjectNode || !selectedProjectNode?.workflow_node_id}
            title={
              selectedProjectNode?.workflow_node_id
                ? "派发选中节点（经 path-lock 闸真跑；画布建的节点会自动建临时 work_item + 用节点 resume 会话）"
                : "先选中一个节点"
            }
          >
            {runningProjectNode ? "运行中…" : "▶ 运行选中节点"}
          </button>
          <Badge tone={projectWorkflow ? "candidate" : "warning"}>{projectWorkflow ? projectWorkflow.state : "缺 workflow"}</Badge>
        </div>
      </div>

      <div className="project-canvas-shell">
        <ProjectWorkflowReactFlowCanvas
          canvasModel={canvasModel}
          selectedNodeId={selectedCanvasNodeId ?? canvasModel.viewport_hint.selected_node_id}
          onSelectNode={setSelectedCanvasNodeId}
        />
        {renderSidePanel({
          canvasModel,
          selectedNodeId: selectedCanvasNodeId ?? canvasModel.viewport_hint.selected_node_id,
          project,
          projectId: canvasModel.project_id,
          sessions,
          projectWorkflow,
          derivedWorkflow,
          selectedTask,
          selectedTaskPackage,
          projectBlackboard,
          blackboardOverlay,
          observationSummary,
          observationStoreRevision: observationStore?.revision ?? 0,
          observations: observationStore?.observations ?? [],
          memorySummary,
          formalSummary,
          memoryLintSummary,
          memoryLintFindings: memoryLintStore?.findings ?? [],
          projectConsultationProposalSummary,
          planAuthorizationSummary,
          projectDirectorTaskPlanRequest,
          projectDirectorTaskPlan,
          projectDirectorTaskPlanLoading,
          projectDirectorTaskPlanError,
          onPreviewProjectDirectorTaskPlan: () => void refreshProjectDirectorTaskPlan(),
          autoDispatchGuardResult,
          autoDispatchGuardError,
          workflowRevision: workflowState?.workflow_version ?? null,
          blackboardStoreRevision: blackboardCandidateStore?.revision ?? 0,
          memoryStoreRevision: memoryCandidateStore?.revision ?? 0,
          memoryCandidates: memoryCandidateStore?.candidates ?? [],
          runtimeSessionAttention,
          realExecutionProductCommands: realExecutionProductCommands ?? null,
          projectWorkflowAutomation: projectWorkflowAutomation ?? null,
          k3B1Recovery: k3B1Recovery ?? null,
          taskMemoryPacketPreview,
          taskMemoryPacketLoading,
          taskMemoryPacketError,
          onRequestAction,
          onOpenAgentSession,
          onInspectWorkflowRunCheck,
        })}
      </div>
      </>
      )}
    </section>
  );
}

// D · 项目规则状态条（蓝图 §11.2）：把已派生的运行性 / 状态原因 / 全局徽标
// （关注 / 权限 / 黑板）condense 成顶部一条；纯读派生数据，不补编、不触发执行。
function ProjectRuleStatusBar({
  canvasModel,
  runCheckStatus,
}: {
  canvasModel: ProjectWorkflowCanvasReadModel;
  runCheckStatus: WorkflowRunCheck["status"] | null;
}) {
  return (
    <div className="project-rule-status-bar" aria-label="项目规则状态条">
      <span className="prsb-headline">{canvasModel.status_reason.label}</span>
      <span className={`prsb-pill runcheck ${runCheckStatus ?? "unknown"}`}>
        运行性：{runCheckStatusLabel(runCheckStatus)}
      </span>
      {canvasModel.global_badges.map((badgeItem) => (
        <span className={`prsb-pill ${badgeItem.tone}`} key={badgeItem.badge_id}>
          {badgeItem.label}
        </span>
      ))}
    </div>
  );
}

function runCheckStatusLabel(status: WorkflowRunCheck["status"] | null) {
  if (status === "runnable") return "可运行";
  if (status === "warning") return "有警告";
  if (status === "blocked") return "不可运行";
  return "未知 / 不可用";
}

type ProjectCanvasFlowNodeData = {
  canvasNode: ProjectCanvasNode;
  selected: boolean;
};

type ProjectCanvasFlowNode = Node<ProjectCanvasFlowNodeData, "projectCanvasNode">;
type ProjectCanvasFlowEdge = Edge<{ canvasEdge: ProjectCanvasEdge }>;

const projectCanvasNodeTypes = {
  projectCanvasNode: ProjectCanvasFlowNodeView,
};

export function ProjectWorkflowReactFlowCanvas({
  canvasModel,
  selectedNodeId,
  onSelectNode,
}: {
  canvasModel: ProjectWorkflowCanvasReadModel;
  selectedNodeId: string;
  onSelectNode: (nodeId: string) => void;
}) {
  // window 守卫放在 hooks 之前：服务端 / 离线测试（直接以普通函数调用本组件）走静态舞台，
  // 不调用任何 hook；浏览器侧才进入下面的 React Flow 内层组件。
  if (typeof window === "undefined") {
    return <ProjectCanvasStaticStage canvasModel={canvasModel} selectedNodeId={selectedNodeId} onSelectNode={onSelectNode} />;
  }
  return <ProjectWorkflowReactFlowCanvasBrowser canvasModel={canvasModel} selectedNodeId={selectedNodeId} onSelectNode={onSelectNode} />;
}

function ProjectWorkflowReactFlowCanvasBrowser({
  canvasModel,
  selectedNodeId,
  onSelectNode,
}: {
  canvasModel: ProjectWorkflowCanvasReadModel;
  selectedNodeId: string;
  onSelectNode: (nodeId: string) => void;
}) {
  const flowNodes = useMemo<ProjectCanvasFlowNode[]>(
    () =>
      canvasModel.nodes.map((node) => ({
        id: node.node_id,
        type: "projectCanvasNode",
        position: {
          x: node.position_hint?.x ?? 0,
          y: node.position_hint?.y ?? 0,
        },
        data: {
          canvasNode: node,
          selected: node.node_id === selectedNodeId,
        },
        selectable: true,
        draggable: false,
      })),
    [canvasModel.nodes, selectedNodeId],
  );
  const flowEdges = useMemo<ProjectCanvasFlowEdge[]>(
    () =>
      canvasModel.edges.map((edge) => ({
        id: edge.edge_id,
        source: edge.source_node_id,
        target: edge.target_node_id,
        label: edge.label ?? undefined,
        type: "smoothstep",
        animated: edge.status === "active",
        markerEnd: { type: MarkerType.ArrowClosed },
        data: { canvasEdge: edge },
        className: `project-canvas-edge ${edge.status}`,
      })),
    [canvasModel.edges],
  );

  return (
    <div className="project-flow-stage" aria-label="项目工作流画布">
      <div className="project-canvas-status-bar" aria-label="画布全局状态">
        {canvasModel.global_badges.map((badgeItem) => (
          <span className={`project-canvas-status-pill ${badgeItem.tone}`} key={badgeItem.badge_id}>
            {badgeItem.label}
          </span>
        ))}
      </div>
      <ProjectCanvasAttentionStrip canvasModel={canvasModel} />
      <ReactFlowProvider>
        <ReactFlow
          nodes={flowNodes}
          edges={flowEdges}
          nodeTypes={projectCanvasNodeTypes}
          nodesDraggable={false}
          nodesConnectable={false}
          elementsSelectable
          fitView
          fitViewOptions={{ padding: 0.14 }}
          minZoom={0.35}
          maxZoom={1.5}
          onNodeClick={(_, node) => onSelectNode(node.id)}
          proOptions={{ hideAttribution: true }}
        >
          <Background gap={28} />
          <Controls showInteractive={false} />
          <MiniMap pannable zoomable nodeStrokeWidth={3} />
        </ReactFlow>
      </ReactFlowProvider>
    </div>
  );
}

function ProjectCanvasStaticStage({
  canvasModel,
  selectedNodeId,
  onSelectNode,
}: {
  canvasModel: ProjectWorkflowCanvasReadModel;
  selectedNodeId: string;
  onSelectNode: (nodeId: string) => void;
}) {
  return (
    <div className="project-flow-stage static" aria-label="项目画布静态状态样例">
      <div className="project-canvas-status-bar" aria-label="画布全局状态">
        {canvasModel.global_badges.map((badgeItem) => (
          <span className={`project-canvas-status-pill ${badgeItem.tone}`} key={badgeItem.badge_id}>
            {badgeItem.label}
          </span>
        ))}
      </div>
      <ProjectCanvasAttentionStrip canvasModel={canvasModel} />
      <div className="project-canvas-static-lanes">
        {canvasModel.nodes.map((node) => (
          <button
            className={`project-canvas-static-node ${node.node_type} ${node.status} ${node.node_id === selectedNodeId ? "selected" : ""}`}
            key={node.node_id}
            type="button"
            onClick={() => onSelectNode(node.node_id)}
          >
            <span>{canvasNodeTypeLabel(node.node_type)}</span>
            <strong>{node.title}</strong>
            <em>{node.subtitle ?? node.status}</em>
            <small>{stateLabel(node.status)}</small>
          </button>
        ))}
      </div>
    </div>
  );
}

function ProjectCanvasAttentionStrip({ canvasModel }: { canvasModel: ProjectWorkflowCanvasReadModel }) {
  const visibleItems = canvasModel.attention_items.slice(0, 2);
  return (
    <div className="project-canvas-attention-strip" aria-label="画布关注摘要">
      <strong>{canvasModel.status_reason.label}</strong>
      <span>{canvasModel.status_reason.summary}</span>
      {visibleItems.map((item) => (
        <em className={item.severity} key={item.attention_id}>{item.title}</em>
      ))}
    </div>
  );
}

export function ProjectCanvasAttentionPanel({ canvasModel }: { canvasModel: ProjectWorkflowCanvasReadModel }) {
  return (
    <section className="project-canvas-detail-card project-canvas-attention-panel">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">画布状态原因</p>
          <h3>{canvasModel.status_reason.label}</h3>
          <p className="path-text">{canvasModel.status_reason.summary}</p>
        </div>
        <Badge tone={badgeToneForCanvasStatus(canvasModel.status)}>{canvasModel.attention_items.length} 项</Badge>
      </div>
      {canvasModel.attention_items.length ? (
        <div className="workflow-compact-list">
          {canvasModel.attention_items.slice(0, 6).map((item) => (
            <div className={`workflow-compact-item ${item.severity}`} key={item.attention_id}>
              <strong>{item.title}</strong>
              <span>{stateLabel(item.status)}</span>
              <em>{item.summary}</em>
            </div>
          ))}
        </div>
      ) : (
        <p className="muted small-note">当前画布没有额外关注项；React Flow 只负责渲染，不保存事实。</p>
      )}
    </section>
  );
}

export function ProjectCanvasEditBoundaryPanel({ boundary }: { boundary: ProjectWorkflowCanvasReadModel["edit_boundary"] }) {
  const layout = boundary.layout_boundary;
  return (
    <section className="project-canvas-detail-card project-canvas-edit-boundary-panel" aria-label="编辑 / 布局边界">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">受控编辑边界</p>
          <h3>编辑 / 布局边界</h3>
          <p className="path-text">{layout.summary}</p>
        </div>
        <Badge tone="unknown">只读</Badge>
      </div>
      <div className="workflow-draft-grid">
        <DetailLine label="布局" value="仅视图布局" />
        <DetailLine label="保存" value="未保存为事实" />
        <DetailLine label="事实源" value="React Flow 仅负责渲染" />
        <DetailLine label="工作流状态" value="不会写入" />
      </div>
      <div className="workflow-compact-list" aria-label="工作流编辑提案预览">
        {boundary.proposal_previews.map((preview) => (
          <div className={`workflow-compact-item ${preview.status === "blocked" ? "warning" : ""}`} key={preview.preview_id}>
            <strong>{preview.label}</strong>
            <span>{projectCanvasEditStatusLabel(preview.status)}</span>
            <em>
              {preview.summary}
              {preview.requires_proposal ? " 需要生成提案。" : ""}
              {preview.requires_confirmation ? " 需要确认弹层。" : ""}
              {preview.requires_control_core ? " 需要控制核心。" : ""}
              {preview.requires_audit ? " 需要审计。" : ""}
            </em>
          </div>
        ))}
      </div>
      <div className="project-canvas-edit-capabilities" aria-label="画布编辑能力矩阵">
        {boundary.capabilities.map((capability) => (
          <span className={capability.status} key={capability.capability_id} title={capability.summary}>
            <strong>{capability.label}</strong>
            <em>{projectCanvasEditStatusLabel(capability.status)}</em>
          </span>
        ))}
      </div>
      <p className="muted small-note">
        本面板只解释边界；节点、边、权限、模型、工具或执行变更都不会从画布直接写成 workflow 事实。
      </p>
    </section>
  );
}

export function ProjectCanvasSurfaceBoundaryPanel({ boundary }: { boundary: CanvasSurfaceBoundary }) {
  return (
    <section className="project-canvas-detail-card project-canvas-surface-boundary-panel" aria-label="项目画布 / 实验画布边界">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">{boundary.eyebrow}</p>
          <h3>{boundary.title}</h3>
          <p className="path-text">{boundary.summary}</p>
        </div>
        <Badge tone="candidate">项目边界</Badge>
      </div>
      <div className="workflow-draft-grid">
        {boundary.items.map((item) => (
          <DetailLine key={item.item_id} label={item.label} value={item.value} />
        ))}
      </div>
      <div className="project-canvas-boundary-badges" aria-label="项目画布边界摘要">
        {boundary.badges.map((badge) => (
          <span key={badge}>{badge}</span>
        ))}
      </div>
    </section>
  );
}

function ProjectCanvasFlowNodeView({ data }: NodeProps<ProjectCanvasFlowNode>) {
  const node = data.canvasNode;
  return (
    <div className={`project-flow-node ${node.node_type} ${node.status} ${data.selected ? "selected" : ""}`}>
      <Handle type="target" position={Position.Left} />
      <div className="project-flow-node-head">
        <span>{canvasNodeTypeLabel(node.node_type)}</span>
        <b>{stateLabel(node.status)}</b>
      </div>
      <strong>{node.title}</strong>
      <em>{node.subtitle ?? "无摘要"}</em>
      <div className="project-flow-node-badges">
        {node.badges.slice(0, 3).map((badgeItem) => (
          <span className={badgeItem.tone} key={badgeItem.badge_id}>{badgeItem.label}</span>
        ))}
      </div>
      <Handle type="source" position={Position.Right} />
    </div>
  );
}

function buildTaskMemoryPacketRequest({
  projectRoot,
  projectId,
  workflowId,
  selectedTask,
  selectedTaskPackage,
  formalStoreRevision,
  candidateStoreRevision,
  observationStoreRevision,
}: {
  projectRoot: string;
  projectId?: string | null;
  workflowId?: string | null;
  selectedTask: TaskDraftSummary;
  selectedTaskPackage: TaskPackage | null;
  formalStoreRevision: number | null;
  candidateStoreRevision: number | null;
  observationStoreRevision: number | null;
}): TaskMemoryPacketBuildInput {
  return {
    project_root: projectRoot,
    project_id: projectId ?? null,
    workflow_id: workflowId ?? null,
    task_id: selectedTask.work_item_id,
    role_id: selectedTask.assigned_role_id?.trim() || selectedTaskPackage?.target_role?.trim() || "project_director",
    task_goal: selectedTaskPackage?.task_goal?.trim() || selectedTask.title,
    retrieval_intent: "worker_task",
    target_model_id: selectedTaskPackage?.model_id ?? null,
    model_context_policy: "local_only",
    max_memory_items: 20,
    max_estimated_tokens: 8000,
    expected_formal_store_revision: formalStoreRevision,
    expected_candidate_store_revision: candidateStoreRevision,
    expected_observation_store_revision: observationStoreRevision,
  };
}

function buildAutoDispatchGuardInput({
  projectWorkflow,
  selectedTask,
  selectedTaskPackage,
}: {
  projectWorkflow: WorkflowStateSnapshot["project_workflows"][number];
  selectedTask: TaskDraftSummary;
  selectedTaskPackage: TaskPackage | null;
}): AutoDispatchGuardInput {
  return {
    project_id: projectWorkflow.project_id,
    workflow_id: projectWorkflow.workflow_id,
    work_item_id: selectedTask.work_item_id,
    task_package_id: selectedTaskPackage?.task_package_id ?? selectedTask.artifact_type ?? null,
    task_package_kind: selectedTaskPackage ? "task_package" : selectedTask.artifact_type ?? "task_package",
    target_role_id: selectedTaskPackage?.target_role?.trim() || selectedTask.assigned_role_id?.trim() || "project_director",
    target_agent_id: selectedTaskPackage?.target_session_id ?? null,
    requested_read_roots: selectedTaskPackage?.allowed_read_scope ?? [],
    requested_write_roots: selectedTaskPackage?.allowed_write_scope ?? [],
    requested_tools: selectedTaskPackage?.callable_tool_capabilities ?? [],
    requested_checks: selectedTaskPackage?.harness_requirements ?? [],
    triggered_stop_conditions: [],
    dispatch_kind: "inspect_only",
  };
}

function buildProjectDirectorTaskPlanRequest({
  project,
  projectWorkflow,
  proposalSummary,
  authorizationSummary,
}: {
  project: ProjectRecord;
  projectWorkflow: WorkflowStateSnapshot["project_workflows"][number] | null;
  proposalSummary: ReturnType<typeof summarizeProjectConsultationProposalStore>;
  authorizationSummary: ReturnType<typeof summarizePlanAuthorizationStore>;
}): PreviewProjectDirectorTaskPlanInput | null {
  const proposal = proposalSummary.latest_proposal;
  const authorization = proposalSummary.linked_plan_authorization;
  if (!projectWorkflow || !proposal || proposal.status !== "user_confirmed") return null;
  if (!authorization || authorization.status !== "active") return null;
  if (authorization.authorization_id !== authorizationSummary.active_authorization_id) return null;
  if (authorization.global_boundary_review?.status !== "approved") return null;
  return {
    project_root: project.project_root,
    project_id: projectWorkflow.project_id,
    workflow_id: projectWorkflow.workflow_id,
    proposal_id: proposal.proposal_id,
    authorization_id: authorization.authorization_id,
    actor_id: "project_director",
    expected_authorization_revision: authorizationSummary.revision,
  };
}

function selectedTaskPackageFor(taskPackages: TaskPackage[], selectedTask: TaskDraftSummary | null): TaskPackage | null {
  if (!selectedTask) return taskPackages[0] ?? null;
  return (
    taskPackages.find((taskPackage) => taskPackage.workflow_node_id === selectedTask.current_node_id) ??
    taskPackages.find((taskPackage) => taskPackage.task_goal === selectedTask.title) ??
    taskPackages[0] ??
    null
  );
}

function stateLabel(state: string) {
  if (state === "empty") return "空态";
  if (state === "idle") return "空闲";
  if (state === "draft") return "草稿";
  if (state === "prepared") return "准备派发";
  if (state === "ready_to_dispatch") return "待派发";
  if (state === "running") return "执行中";
  if (state === "waiting_for_permission") return "等待权限";
  if (state === "needs_review") return "待复核";
  if (state === "retry_pending") return "待重试";
  if (state === "failed") return "失败";
  if (state === "timed_out") return "已超时";
  if (state === "readback_unavailable") return "读回不可用";
  if (state === "cancelled") return "已取消";
  if (state === "ready_for_review") return "待回收";
  if (state === "accepted") return "已接受";
  if (state === "needs_changes") return "需修改";
  if (state === "paused") return "暂停";
  return state || "未知";
}

export function badgeToneForCanvasStatus(status: ProjectCanvasStatus): "candidate" | "warning" | "unknown" {
  if (status === "accepted" || status === "ready_to_dispatch" || status === "ready_for_review" || status === "prepared") return "candidate";
  if (status === "running") return "candidate";
  if (status === "waiting_for_permission" || status === "blocked" || status === "failed" || status === "timed_out" || status === "needs_changes" || status === "needs_review" || status === "readback_unavailable") {
    return "warning";
  }
  return "unknown";
}

function projectCanvasEditStatusLabel(status: ProjectWorkflowCanvasReadModel["edit_boundary"]["capabilities"][number]["status"]) {
  if (status === "allowed") return "允许查看";
  if (status === "preview_only") return "仅预览";
  if (status === "requires_future_task") return "后续任务";
  return "已阻断";
}

function canvasNodeTypeLabel(type: ProjectCanvasNode["node_type"]) {
  if (type === "project_goal") return "项目目标";
  if (type === "director") return "总指导";
  if (type === "dev_line") return "开发线";
  if (type === "validation_line") return "验证线";
  if (type === "review_line") return "回收线";
  if (type === "permission_request") return "权限";
  if (type === "blackboard_candidate") return "黑板候选";
  if (type === "evidence_ref") return "证据";
  if (type === "audit_ref") return "审计";
  return type;
}

function messageOf(error: unknown): string {
  if (error instanceof Error) return error.message;
  return String(error);
}

// 项目节点派发回执的一句话摘要（后端返回 WorkflowNodeDispatchResult；这里只取状态/退出码给提示用）。
function compactRunResult(result: unknown): string {
  const dispatch = (result as { dispatch?: { state?: string; exit_code?: number | null } } | null)?.dispatch;
  if (!dispatch) return "已提交";
  const exit = dispatch.exit_code ?? "—";
  return `${dispatch.state ?? "未知"}（exit ${exit}）`;
}
