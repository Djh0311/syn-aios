import { Badge } from "../../components/Badge";
import { DetailLine } from "../../components/WorkbenchPrimitives";
import { FactRow, SegTitle } from "../../components/SpecPrimitives";
import type {
  PlanAuthorization,
  PlanAuthorizationStoreV1,
  ProjectRecord,
  SessionRecord,
  TaskDraftSummary,
  WorkflowStateSnapshot,
} from "../../lib/types";

export { DetailLine };

type ProjectOverviewSelectTool = "jiaoban" | "workflow" | "handoff-evidence" | "resources";

// ⑥ G 定稿(hifi `G · 项目页·总览`)：总览 = 项目事实卡**单卡** + 第二卡位留白(用户 07-15 拍：
// 不硬塞，等真实需求出现再定)。旧的四块(项目概览 / 智能体入口 / 工作流 / 交接证据 compact)退场——
// 会话入口归智能体页(H)、工作流详情归「完整工作流」tab、交接证据归「交接」tab，都不是本面的唯一问题。
//
// ⚠️ 零 hooks 硬约束：`tests/helpers/offlineInteractionTestUtils.tsx` 的 renderComposite 裸调
// `Component(element.props)`，`ProjectOverview` 及其子组件用任何 hook 都会当场炸。本文件保持纯函数。
export function ProjectOverview({
  project,
  workflowState,
  planAuthorizationStore,
  onSelectTool,
}: {
  project: ProjectRecord;
  workflowState: WorkflowStateSnapshot | null;
  planAuthorizationStore: PlanAuthorizationStoreV1 | null;
  onSelectTool: (tool: ProjectOverviewSelectTool) => void;
}) {
  const projectWorkflow = workflowState?.project_workflows.find((workflow) => workflow.project_root === project.project_root) ?? null;
  const taskDrafts = projectWorkflow?.task_drafts ?? [];
  const warningCount = projectWarnings(project);
  const writeAuthorization = overviewWriteAuthorization(planAuthorizationStore, projectWorkflow?.project_id ?? null, project.project_root);

  return (
    <section className="project-overview-grid">
      <article className="project-overview-card primary">
        <div className="panel-heading">
          <div>
            <SegTitle>项目事实</SegTitle>
          </div>
          <Badge tone={warningCount ? "warning" : "candidate"}>{warningCount ? `${warningCount} warning` : "无 warning"}</Badge>
        </div>
        <div>
          <FactRow k="路径">{project.project_root}</FactRow>
          {/* 最近交货：读模型 `list_project_run_history`(RunHistoryEntry.state="delivered")真实存在、
              且已被 ProjectJiaobanPanel 消费，但它是异步 Tauri 命令 —— 本组件零 hooks，取不到。
              接线要由能持 hooks 的上层把 prop 喂下来 = 超本包范围 → 留位「接线中」，不拿 latestSession
              (那是会话不是交货，口径不同)冒充。 */}
          <FactRow k="最近交货">接线中——交货读模型还没接到本页</FactRow>
          <FactRow k="工单">{overviewTaskDraftValue(taskDrafts, projectWorkflow !== null)}</FactRow>
          <FactRow k="写授权" bad={writeAuthorization.bad}>
            {writeAuthorization.text}
          </FactRow>
          {/* 文件：定稿列的是 README·game.js·index.html·styles.css = 项目源文件；索引只有
              authority_files/handoff_files/evidence_files = 治理类文件候选，口径不同。
              照图渲染会照出假事实 → 留位「接线中」，治理类文件的真实入口在「交接」tab。 */}
          <FactRow k="文件">接线中——索引只有治理类文件，没有项目源文件清单</FactRow>
        </div>
        <div className="workflow-state-actions">
          <button className="primary-button" type="button" onClick={() => onSelectTool("jiaoban")}>
            去交办
          </button>
          <button className="secondary-button" type="button" onClick={() => onSelectTool("workflow")}>
            看工作流
          </button>
        </div>
      </article>

      {/* 第二卡位：定稿明确留白(不是占位待做)。虚线框 + 人话注明为什么空着。 */}
      <div className="project-overview-reserved">
        第二卡位留白
        <span>用户 07-15 拍：不硬塞，等真实需求出现再定</span>
      </div>
    </section>
  );
}

export function ProjectAgentMovedPanel({
  project,
  sessions,
  onOpenAgentSession,
}: {
  project: ProjectRecord;
  sessions: SessionRecord[];
  onOpenAgentSession: (threadId: string) => void;
}) {
  const latestSession =
    sessions
      .filter((session) => !session.archived && session.rollout_exists)
      .sort((a, b) => (b.updated_at_ms ?? 0) - (a.updated_at_ms ?? 0))[0] ?? null;

  return (
    <section className="project-tool-placeholder">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">智能体承接</p>
          <h3>会话列表和对话界面不再放在项目工作台</h3>
        </div>
        <Badge tone="unknown">{sessions.length} 会话</Badge>
      </div>
      <p className="muted small-note">
        {project.name} 的会话仍按 project_root 过滤，但入口在智能体页：先选智能体，再看会话列表和正文。
      </p>
      <div className="workflow-state-actions">
        <button
          className="secondary-button"
          type="button"
          disabled={!latestSession}
          onClick={() => latestSession && onOpenAgentSession(latestSession.thread_id)}
        >
          在智能体中打开
        </button>
      </div>
    </section>
  );
}

export function ProjectToolPlaceholder({ project, label }: { project: ProjectRecord; label: string }) {
  return (
    <section className="project-tool-placeholder">
      <div className="panel-heading">
        <div>
          <h3>{label}</h3>
        </div>
        <Badge tone="unknown">占位</Badge>
      </div>
      <p className="muted small-note">{project.name}</p>
    </section>
  );
}

function projectWarnings(project: ProjectRecord) {
  return project.context_warnings.length + project.warnings.length;
}

// 定稿「工单 7 单(1 等你)」。总数 = task_drafts.length(实数)。
// ⚠️「等你」的状态集合定稿没写死 → 取两个**标签本身就是「在等人」**的状态：
// waiting_for_permission(等待权限，要用户批)/ ready_for_review(待回收，要用户复核)。
// 其余状态(running/failed/…)是机器侧或系统侧，不算欠用户动作。此选择列进 forks 待主导线确认。
const OVERVIEW_WAITING_ON_USER_STATES = new Set(["waiting_for_permission", "ready_for_review"]);

function overviewTaskDraftValue(taskDrafts: TaskDraftSummary[], hasWorkflow: boolean): string {
  // D7：空态必答下一步，不许只说「这里没有」。
  if (!hasWorkflow) return "缺少项目默认 workflow；先去交办建一个";
  if (!taskDrafts.length) return "0 单；去交办派第一单";
  const waiting = taskDrafts.filter((taskDraft) => OVERVIEW_WAITING_ON_USER_STATES.has(taskDraft.state)).length;
  return waiting ? `${taskDrafts.length} 单(${waiting} 等你)` : `${taskDrafts.length} 单`;
}

// 定稿「写授权 已开(仅此项目)」。源 = PlanAuthorizationStoreV1(active 授权的 scope.allowed_write_roots)。
// 授权按 project_id 挂在项目 workflow 上；没有 workflow 就不可能有本项目的授权。
function overviewWriteAuthorization(
  store: PlanAuthorizationStoreV1 | null,
  projectId: string | null,
  projectRoot: string,
): { text: string; bad: boolean } {
  if (!store) return { text: "接线中——授权状态还没读取", bad: false };
  if (!projectId) return { text: "未开——项目还没有工作流，先去交办建一个", bad: false };
  const active = store.authorizations.filter(
    (authorization: PlanAuthorization) => authorization.status === "active" && authorization.project_id === projectId,
  );
  if (!active.length) return { text: "未开——没有生效的写授权", bad: false };
  const writeRoots = Array.from(new Set(active.flatMap((authorization) => authorization.scope.allowed_write_roots)));
  if (!writeRoots.length) return { text: "已开·但没有写根(只能读，不能写)", bad: false };
  const outside = writeRoots.filter((root) => !overviewPathWithin(root, projectRoot));
  if (!outside.length) return { text: "已开(仅此项目)", bad: false };
  return { text: `已开·含 ${outside.length} 个项目外写根`, bad: true };
}

function overviewPathWithin(path: string, root: string): boolean {
  const normalizedPath = path.replace(/\/+$/, "");
  const normalizedRoot = root.replace(/\/+$/, "");
  return normalizedPath === normalizedRoot || normalizedPath.startsWith(`${normalizedRoot}/`);
}
