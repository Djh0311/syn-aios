import { useState } from "react";
import type { ReactNode } from "react";
import { Badge } from "../../components/Badge";
import type { ProjectCanvasNode, ProjectWorkflowCanvasReadModel } from "../../lib/projectCanvas";
import type {
  TaskPackage,
  WorkflowStateSnapshot,
} from "../../lib/types";
import {
  badgeToneForCanvasStatus,
  type ProjectWorkflowCanvasSidePanelProps,
} from "./ProjectWorkflowCanvasView";
import {
  GlobalBoundaryReviewCard,
  ProjectConsultationProposalCard,
  ProjectDirectorTaskPlanCard,
  AutoAdvanceRoleLoopButton,
} from "./ProjectWorkflowGovernancePanels";
import {
  WorkItemOrchestrationCard,
} from "./ProjectWorkflowExecutionPanels";
import { ProjectCanvasDetailLine } from "./ProjectCanvasDetailPrimitives";
import { WorkflowRunCheckDetails } from "./ProjectWorkflowRunCheckPanel";
import {
  stateLabel,
} from "./projectWorkflowLabels";

export { WorkflowRunCheckDetails };

export function ProjectCanvasSidePanel({
  canvasModel,
  selectedNodeId,
  project,
  projectId,
  sessions,
  projectWorkflow,
  derivedWorkflow,
  selectedTask,
  selectedTaskPackage,
  projectBlackboard,
  blackboardOverlay,
  observationSummary,
  observationStoreRevision,
  observations,
  memorySummary,
  formalSummary,
  memoryLintSummary,
  memoryLintFindings,
  projectConsultationProposalSummary,
  planAuthorizationSummary,
  projectDirectorTaskPlanRequest,
  projectDirectorTaskPlan,
  projectDirectorTaskPlanLoading,
  projectDirectorTaskPlanError,
  onPreviewProjectDirectorTaskPlan,
  autoDispatchGuardResult,
  autoDispatchGuardError,
  workflowRevision,
  blackboardStoreRevision,
  memoryStoreRevision,
  memoryCandidates,
  runtimeSessionAttention,
  realExecutionProductCommands,
  projectWorkflowAutomation,
  k3B1Recovery,
  taskMemoryPacketPreview,
  taskMemoryPacketLoading,
  taskMemoryPacketError,
  onRequestAction,
  onOpenAgentSession,
  onInspectWorkflowRunCheck,
}: ProjectWorkflowCanvasSidePanelProps) {
  // 全局「显示开发者细节」开关：默认关（侧栏带 hide-dev-detail 类，靠 CSS display:none 隐藏所有 agent-boundary-details）。
  // 用 CSS 隐藏而非条件渲染——折叠内容仍在 renderToStaticMarkup 输出里，离线断言照数得到。
  const [showDevDetail, setShowDevDetail] = useState(false);

  return (
    <aside
      className={`project-canvas-side-panel${showDevDetail ? "" : " hide-dev-detail"}`}
      aria-label="节点详情和项目工作流控制"
    >
      <label className="project-side-dev-detail-toggle">
        <input
          type="checkbox"
          checked={showDevDetail}
          onChange={(event) => setShowDevDetail(event.currentTarget.checked)}
        />
        显示开发者细节
      </label>
      {/* 节点详情已挪到画布节点旁的小面板（ProjectWorkflowCanvasView 的 NodeToolbar / 静态舞台），抽屉只剩工作流详情。 */}

      {/* 砍一波（细调）：角色循环主线（方案与授权）= 常驻一等、置顶摊开（说目标→出方案→审→批→一键自动推进）。
          节点详情常驻；工作项执行 / 事实与记忆 / 运行检查（含统一执行链路协议 dump）全默认收起、沉到下面。
          一切只动「显隐 + 折叠层级」——内容仍在 markup 里，离线断言照过。 */}
      <ProjectSidePanelSection title="方案与授权" description="说目标 → AI 出方案 → 你审 → 批 → 一键自动推进" defaultOpen={true}>
        <ProjectConsultationProposalCard
          project={project}
          projectWorkflow={projectWorkflow}
          selectedTask={selectedTask}
          selectedTaskPackage={selectedTaskPackage}
          summary={projectConsultationProposalSummary}
          planAuthorizationRevision={planAuthorizationSummary.revision}
          onRequestAction={onRequestAction}
        />
        {/* 件 D 核心：一键自动推进（方案授权生效后才出现）——主路径，放拆任务卡之上。 */}
        <AutoAdvanceRoleLoopButton project={project} request={projectDirectorTaskPlanRequest} />
        <ProjectDirectorTaskPlanCard
          project={project}
          request={projectDirectorTaskPlanRequest}
          plan={projectDirectorTaskPlan}
          loading={projectDirectorTaskPlanLoading}
          error={projectDirectorTaskPlanError}
          workflowRevision={workflowRevision}
          onPreview={onPreviewProjectDirectorTaskPlan}
          onRequestAction={onRequestAction}
        />
        {/* 全局边界复核偏治理深，默认收起折叠（文案被离线断言，折不删）。
            方案授权摘要卡已裁掉（与拆任务计划 / 全局复核信息重叠）——展示层删除，组件定义保留。 */}
        <ProjectSidePanelFold title="全局边界复核" description="授权 / 守卫 / 复核结论（治理深，默认收起）">
          <GlobalBoundaryReviewCard
            project={project}
            projectWorkflow={projectWorkflow}
            proposalSummary={projectConsultationProposalSummary}
            planAuthorizationSummary={planAuthorizationSummary}
            guardResult={autoDispatchGuardResult}
            guardError={autoDispatchGuardError}
            onRequestAction={onRequestAction}
          />
        </ProjectSidePanelFold>
      </ProjectSidePanelSection>

      {/* 砍一波：工作项执行（派发 / 会话绑定 / C5汇报 / C6 / 回收 + 旧封存入口）= 当前最臃肿区，整段默认收起。 */}
      {selectedTask && projectWorkflow ? (
        <ProjectSidePanelSection title="工作项执行" description="派发 / 会话绑定 / 工作者汇报 / 总指导回收 / 下一步（默认收起）" defaultOpen={false}>
          <WorkItemOrchestrationCard
            project={project}
            projectId={projectId}
            sessions={sessions}
            bindings={projectWorkflow.node_session_bindings}
            dispatches={projectWorkflow.node_dispatches}
            directorReviews={projectWorkflow.director_reviews}
            executionControls={projectWorkflow.execution_controls}
            permissionRequests={projectWorkflow.permission_requests}
            executionAttempts={projectWorkflow.execution_attempts}
            derivedWorkflow={derivedWorkflow}
            projectConsultationProposalSummary={projectConsultationProposalSummary}
            planAuthorizationSummary={planAuthorizationSummary}
            workflowRevision={workflowRevision}
            observationStoreRevision={observationStoreRevision}
            workItem={selectedTask}
            onRequestAction={onRequestAction}
            onOpenAgentSession={onOpenAgentSession}
          />
        </ProjectSidePanelSection>
      ) : null}

    </aside>
  );
}

function ProjectSidePanelSection({
  title,
  description,
  defaultOpen,
  children,
}: {
  title: string;
  description: string;
  defaultOpen: boolean;
  children: ReactNode;
}) {
  return (
    <details className="project-side-panel-section" open={defaultOpen}>
      <summary>
        <span>{title}</span>
        <em>{description}</em>
      </summary>
      <div className="project-side-panel-section-body">
        {children}
      </div>
    </details>
  );
}

// A·节点详情精简：日常决策只看 状态/角色/会话/模型（summary/task/package/binding/dispatch/readback），
// 其余（知识库·记忆包 / 工具·权限 / 审查·验收 / harness·审计）折进默认收起的「节点详情·更多」。
// 只动显示位置：折进的 section 仍在 renderToStaticMarkup 里，离线断言照过。
const NODE_DETAIL_PRIMARY_KINDS = new Set([
  "summary",
  "task_package",
  "session_binding",
  "dispatch",
  "readback",
]);

function isNodeDetailPrimarySection(section: ProjectCanvasDetailSectionView) {
  // 「任务记忆包摘要」(memory_packet) 虽也是 task_package 邻区，但属知识库/记忆，折起。
  if (section.kind === "memory_packet") return false;
  return NODE_DETAIL_PRIMARY_KINDS.has(section.kind);
}

// 单卡级默认收起折叠：把整张卡折进 <details>，summary 给标题/描述。
// 折叠内容在 renderToStaticMarkup 里仍计入，离线断言照过；真机默认收起省地方。
function ProjectSidePanelFold({
  title,
  description,
  children,
}: {
  title: string;
  description: string;
  children: ReactNode;
}) {
  return (
    <details className="project-side-panel-fold">
      <summary>
        <span>{title}</span>
        <em>{description}</em>
      </summary>
      <div className="project-side-panel-fold-body">{children}</div>
    </details>
  );
}

// 后端工作流引擎吐的机器警告码（snake_case，如 previous_state_closure_retest_timed_out、
// legacy_workflow_node_dispatch_not_h5_unified_product_command）是英文噪音。用户定（2026-06-30 警告码处理=完全不显示）：
// 节点详情一律不显示这些码。判据：值按分隔符切开后，每个 token 都是「全小写、含下划线」的机器码；
// 中文内容、含中文的安全状态警告不命中，照常保留。
function isMachineCodeValue(value: string): boolean {
  const tokens = String(value).trim().split(/[;；,，\s]+/).filter(Boolean);
  return tokens.length > 0 && tokens.every((token) => /^[a-z][a-z0-9_:]*$/.test(token) && token.includes("_"));
}

// 警告码隐掉后「用户摘要/节点状态」可能空，给一句始终都有的中文状态概况（按节点状态映射成人话）。
function nodeStateHint(state: string): string {
  const HINT: Record<string, string> = {
    empty: "尚未配置",
    idle: "空闲，等待派发",
    draft: "草稿，待完善",
    prepared: "已准备，待派发",
    ready_to_dispatch: "待派发到运行器",
    running: "正在执行",
    waiting_for_permission: "等待授权批准",
    needs_review: "等待复核",
    retry_pending: "等待重试",
    failed: "执行失败，需排查",
    timed_out: "已超时，需重试或排查",
    readback_unavailable: "结果读回不可用",
    cancelled: "已取消",
    ready_for_review: "待回收结果",
    accepted: "已接受完成",
    needs_changes: "需修改后继续",
    paused: "已暂停",
  };
  return HINT[state] ?? `当前${stateLabel(state)}`;
}

export function ProjectCanvasNodeDetailView({ detail, node }: { detail: NonNullable<ProjectWorkflowCanvasReadModel["detail_panels"][string]>; node: ProjectCanvasNode | null }) {
  const layers = projectCanvasDetailLayers(detail);
  const renderSection = (section: ProjectCanvasDetailSectionView) => {
    const items = section.items
      // 砍一波（按删画布详情那次的办法）：原始 ref（如 workflow node 长 ID）是日常噪音，隐掉。
      // 用户定继续砍：当前节点(=头部重复)、当前状态(=徽章重复)、谁能处理(信息量低) 三行。
      .filter((item) => !["当前节点", "当前状态", "谁能处理"].includes(String(item.label).trim()))
      .filter((item) => !/^workflow:.*:node:/.test(String(item.value).trim()))
      // 用户定（警告码处理=完全不显示）：后端 snake_case 机器警告码是英文噪音，节点详情一律隐掉；中文保留。
      .filter((item) => !isMachineCodeValue(String(item.value)))
      // 空 warning（warning: 无）也隐掉。
      .filter(
        (item) =>
          !(
            ["warning", "警告"].includes(String(item.label).trim().toLowerCase()) &&
            ["", "无", "none", "-", "—"].includes(String(item.value).trim().toLowerCase())
          ),
      );
    if (!items.length) return null;
    return (
      <article className={`project-canvas-detail-section ${section.kind}`} key={section.section_id}>
        <strong>{section.title}</strong>
        {items.map((item) => (
          <ProjectCanvasDetailLine item={item} key={item.item_id} />
        ))}
      </article>
    );
  };
  return (
    <section className="project-canvas-detail-card node-detail-panel">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">节点详情</p>
          <h3>{detail.title}</h3>
          {/* 精简：去掉重复的 summary（多是节点 kind，如 director——和标题/徽章重复）。 */}
        </div>
        <Badge tone={badgeToneForCanvasStatus(node?.status ?? "unknown")}>{node ? stateLabel(node.status) : "未知"}</Badge>
      </div>
      {/* 用户定（2026-06-30）：警告码隐掉后「用户摘要/节点状态」可能空，补一句始终都有的中文状态概况。 */}
      {node ? <p className="node-detail-summary">{nodeStateHint(node.status)}</p> : null}
      {/* 砍一波（拍平）：节点详情只剩「用户摘要」一层，去掉层 <details> 包裹 + 层标题 +「节点详情·更多」折叠，
          直接把 user_summary 的 primary section 行（为什么停下 / 下一步 …）渲染出来。节点详情 = 头部 + 直接几行 + warnings。 */}
      <div className="project-canvas-detail-layers">
        {layers.flatMap((layer) => layer.sections.filter(isNodeDetailPrimarySection)).map(renderSection)}
      </div>
      {/* 用户定（警告码处理=完全不显示）：隐掉后端 snake_case 机器警告码；含中文的安全状态警告仍保留。 */}
      {detail.warnings.filter((warning) => !isMachineCodeValue(warning)).map((warning) => (
        <p className="state-warning" key={warning}>{warning}</p>
      ))}
    </section>
  );
}

type ProjectCanvasDetailSectionView = NonNullable<ProjectWorkflowCanvasReadModel["detail_panels"][string]>["sections"][number];

function projectCanvasDetailLayers(detail: NonNullable<ProjectWorkflowCanvasReadModel["detail_panels"][string]>) {
  // 砍一波（用户定）：项目主管信息 / 技术详情两整层砍掉，节点详情只留「用户摘要」。
  const layerOrder: Array<ProjectCanvasDetailSectionView["layer"]> = ["user_summary"];
  return layerOrder
    .map((layer) => {
      const sections = detail.sections.filter((section) => section.layer === layer);
      return {
        layer,
        sections,
        defaultOpen: sections.some((section) => section.default_open),
      };
    })
    .filter((layer) => layer.sections.length);
}
