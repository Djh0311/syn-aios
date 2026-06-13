import { Badge } from "../../components/Badge";
import { formatDate } from "../../lib/format";
import type {
  BlackboardCandidateStoreV1,
  MemoryCandidateStoreV1,
  ProjectRecord,
  SessionRecord,
  TaskDraftSummary,
  WorkflowStateSnapshot,
} from "../../lib/types";
import { ProjectHandoffEvidencePanel } from "./ProjectReferencePanels";

type ProjectOverviewSelectTool = "workflow" | "handoff-evidence" | "resources";

export function ProjectOverview({
  project,
  sessions,
  workflowState,
  blackboardCandidateStore,
  memoryCandidateStore,
  onOpenAgentSession,
  onSelectTool,
}: {
  project: ProjectRecord;
  sessions: SessionRecord[];
  workflowState: WorkflowStateSnapshot | null;
  blackboardCandidateStore: BlackboardCandidateStoreV1 | null;
  memoryCandidateStore: MemoryCandidateStoreV1 | null;
  onOpenAgentSession: (threadId: string) => void;
  onSelectTool: (tool: ProjectOverviewSelectTool) => void;
}) {
  const projectWorkflow = workflowState?.project_workflows.find((workflow) => workflow.project_root === project.project_root) ?? null;
  const latestSession =
    sessions
      .filter((session) => !session.archived && session.rollout_exists)
      .sort((a, b) => (b.updated_at_ms ?? 0) - (a.updated_at_ms ?? 0))[0] ?? null;
  const fileCount = project.authority_files.length + project.handoff_files.length + project.evidence_files.length;
  const warningCount = projectWarnings(project);
  const activeTask = overviewSelectedTaskDraftFor(projectWorkflow?.task_drafts ?? [], null);
  const blackboardCount = blackboardCandidateStore?.records.filter((record) => record.project_root === project.project_root).length ?? 0;
  const memoryCount =
    memoryCandidateStore?.candidates.filter((candidate) => candidate.scope.project_id === projectWorkflow?.project_id || !candidate.scope.project_id).length ?? 0;

  return (
    <section className="project-overview-grid">
      <article className="project-overview-card primary">
        <div className="panel-heading">
          <div>
            <p className="eyebrow">项目概览</p>
            <h3>{project.active_hint ? "索引标记为活跃项目" : "当前没有活跃提示"}</h3>
          </div>
          <Badge tone={warningCount ? "warning" : "candidate"}>{warningCount ? `${warningCount} warning` : "无 warning"}</Badge>
        </div>
        <div className="workflow-draft-grid">
          <DetailLine label="会话" value={`${sessions.length || project.thread_count} 个；完整列表在智能体页`} />
          <DetailLine label="工作流" value={projectWorkflow ? projectWorkflow.title : "缺少项目默认 workflow"} />
          <DetailLine label="交接 / 证据 / 权威" value={`${fileCount} 个文件`} />
          <DetailLine label="运行器" value={`${project.harness_resources.length + project.harness_candidates.length} 个资源 / 候选`} />
          <DetailLine label="候选治理" value={`黑板 ${blackboardCount} / 记忆 ${memoryCount}`} />
        </div>
        <div className="workflow-state-actions">
          <button className="secondary-button" type="button" onClick={() => onSelectTool("workflow")}>
            打开工作流
          </button>
          <button className="secondary-button" type="button" onClick={() => onSelectTool("handoff-evidence")}>
            查看交接证据
          </button>
          <button className="secondary-button" type="button" onClick={() => onSelectTool("resources")}>
            查看资源
          </button>
        </div>
      </article>

      <article className="project-overview-card">
        <div className="panel-heading">
          <div>
            <p className="eyebrow">智能体入口</p>
            <h3>会话列表和对话界面已放到智能体页</h3>
          </div>
          <Badge tone={latestSession ? "candidate" : "unknown"}>{latestSession ? "可打开" : "无会话"}</Badge>
        </div>
        <p className="muted small-note">
          项目工作台只保留会话摘要；选中智能体后再看会话列表和正文，避免项目页变回会话中心。
        </p>
        <div className="workflow-draft-grid">
          <DetailLine label="最近会话" value={latestSession?.title ?? "没有可读取会话"} />
          <DetailLine label="更新时间" value={formatDate(latestSession?.updated_at_ms)} />
        </div>
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
      </article>

      <article className="project-overview-card">
        <div className="panel-heading">
          <div>
            <p className="eyebrow">当前工作流</p>
            <h3>{activeTask?.title ?? "还没有当前工作项"}</h3>
          </div>
          <Badge tone={projectWorkflow ? "candidate" : "warning"}>{projectWorkflow?.state ?? "缺 workflow"}</Badge>
        </div>
        <div className="workflow-draft-grid">
          <DetailLine label="工作流" value={projectWorkflow?.title ?? "未创建"} />
          <DetailLine label="工作项状态" value={activeTask ? overviewStateLabel(activeTask.state) : "未登记"} />
          <DetailLine label="当前位置" value={overviewWorkflowNodeLabel(activeTask?.current_node_id)} />
          <DetailLine label="下一步" value={activeTask?.next_action_label ?? "缺少状态规则"} />
        </div>
      </article>

      <ProjectHandoffEvidencePanel project={project} compact />
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

export function DetailLine({ label, value }: { label: string; value: string }) {
  return (
    <div className="detail-line">
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}

function projectWarnings(project: ProjectRecord) {
  return project.context_warnings.length + project.warnings.length;
}

function overviewSelectedTaskDraftFor(taskDrafts: TaskDraftSummary[], selectedWorkItemId: string | null): TaskDraftSummary | null {
  if (!selectedWorkItemId) return taskDrafts[0] ?? null;
  return taskDrafts.find((taskDraft) => taskDraft.work_item_id === selectedWorkItemId) ?? null;
}

function overviewStateLabel(state: string) {
  if (state === "empty") return "空态";
  if (state === "idle") return "空闲";
  if (state === "draft") return "草稿";
  if (state === "prepared") return "准备派发";
  if (state === "ready_to_dispatch") return "待派发";
  if (state === "running") return "执行中";
  if (state === "waiting_for_permission") return "等待权限";
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

function overviewWorkflowNodeLabel(nodeId?: string | null) {
  if (!nodeId) return "未绑定节点";
  if (nodeId.includes("analysis")) return "分析";
  if (nodeId.includes("implement")) return "实现";
  if (nodeId.includes("review")) return "复核";
  if (nodeId.includes("accept")) return "验收";
  return nodeId;
}
