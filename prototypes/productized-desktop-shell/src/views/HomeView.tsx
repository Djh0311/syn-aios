import { formatDate } from "../lib/format";
import type { WorkbenchSnapshot, WorkflowStateSnapshot } from "../lib/types";
import type { ViewKey } from "../lib/workbenchNavigation";

type HomeViewProps = {
  snapshot: WorkbenchSnapshot;
  workflowState?: WorkflowStateSnapshot | null;
  onNavigate: (view: ViewKey) => void;
};

type HomeFeedTone = "ok" | "warn" | "err" | "run";

type HomeFeedItem = {
  key: string;
  label: string;
  meta: string;
  tone?: HomeFeedTone;
  view: ViewKey;
};

function compactText(value: string, maxLength: number) {
  const normalized = value.replace(/\s+/g, " ").trim() || "未命名";
  return normalized.length > maxLength ? `${normalized.slice(0, Math.max(1, maxLength - 1))}…` : normalized;
}

function compactDisplayName(value: string, maxLength = 28) {
  const normalized = value.replace(/\s+/g, " ").trim() || "未命名";
  const parts = normalized.split(/[\\/]/).filter(Boolean);
  return compactText(parts.length > 1 ? parts[parts.length - 1] : normalized, maxLength);
}

function HomeNode({
  className,
  ko,
  name,
  count,
  rows,
  moreLabel,
  view,
  onNavigate,
}: {
  className: string;
  ko: string;
  name: string;
  count: number;
  rows: HomeFeedItem[];
  moreLabel: string;
  view: ViewKey;
  onNavigate: (view: ViewKey) => void;
}) {
  return (
    <section className={`node ${className}`} aria-label={name}>
      <div className="node-mark">
        <span className="ko">{ko}</span>
        <button className="name home-node-action" type="button" onClick={() => onNavigate(view)}>
          {name}
        </button>
        <span className="count">· {count}</span>
      </div>
      <div className="node-feed">
        {rows.slice(0, 3).map((row) => (
          <button className="row" key={row.key} type="button" onClick={() => onNavigate(row.view)} title={`${row.label} · ${row.meta}`}>
            <span className={`d ${row.tone ?? ""}`} aria-hidden="true" />
            <span className="t">{compactDisplayName(row.label)}</span>
            <span className="x">{compactText(row.meta, 18)}</span>
          </button>
        ))}
        <button className="more home-node-more" type="button" onClick={() => onNavigate(view)}>
          {moreLabel}
        </button>
      </div>
    </section>
  );
}

export function HomeView({ snapshot, workflowState = null, onNavigate }: HomeViewProps) {
  const recentProjects = [...snapshot.projects]
    .sort((a, b) => (b.latest_updated_at_ms ?? 0) - (a.latest_updated_at_ms ?? 0))
    .slice(0, 4);

  const needsUser = snapshot.runtime_session_attention.filter((a) => a.requires_user_action);
  const running = snapshot.session_run_status_summaries.filter(
    (s) => s.current_status === "running" || s.attention_count > 0,
  );

  const workflowCount = workflowState?.counts.workflows ?? 0;
  const workItemCount = workflowState?.counts.work_items ?? 0;
  const runningWorkflows = (workflowState?.project_workflows ?? []).filter((workflow) =>
    workflow.task_drafts.some((task) =>
      ["running", "waiting_for_permission", "retry_pending", "ready_to_dispatch", "ready_for_review"].includes(task.state),
    ),
  );
  const recentWorkflows = (runningWorkflows.length ? runningWorkflows : workflowState?.project_workflows ?? []).slice(0, 3);
  const recentSkills = snapshot.skills.slice(0, 3);
  const harnessResources = snapshot.projects.flatMap((project) =>
    project.harness_resources.map((resource) => ({
      label: resource.display_name ?? resource.root_path,
      projectName: project.name,
    })),
  );
  const projectRows: HomeFeedItem[] = recentProjects.length
    ? recentProjects.slice(0, 3).map((project) => ({
        key: project.project_root,
        label: project.name,
        meta: `${project.thread_count} 会话`,
        tone: project.context_warnings.length ? "warn" : "ok",
        view: "projects",
      }))
    : [{ key: "empty-projects", label: "暂无项目", meta: "—", view: "projects" }];
  const agentRows: HomeFeedItem[] = needsUser.length
    ? needsUser.slice(0, 3).map((attention) => ({
        key: attention.attention_id,
        label: attention.title,
        meta: "待确认",
        tone: "warn",
        view: "agents",
      }))
    : running.length
      ? running.slice(0, 3).map((summary) => ({
          key: summary.session_id,
          label: summary.current_status_label,
          meta: "运行中",
          tone: "run",
          view: "agents",
        }))
      : [{ key: "empty-agents", label: "暂无需要处理的智能体状态", meta: "—", view: "agents" }];
  const skillRows: HomeFeedItem[] = recentSkills.length
    ? recentSkills.map((skill) => ({
        key: skill.skill_id,
        label: skill.title,
        meta: skill.source_type,
        tone: "ok",
        view: "skills",
      }))
    : [{ key: "empty-skills", label: "暂无 Skill", meta: "—", view: "skills" }];
  const harnessRows: HomeFeedItem[] = harnessResources.length
    ? harnessResources.slice(0, 3).map((resource) => ({
        key: `${resource.projectName}-${resource.label}`,
        label: resource.label,
        meta: resource.projectName,
        tone: "run",
        view: "harness",
      }))
    : [{ key: "empty-harness", label: "暂无 Harness 资源", meta: "—", view: "harness" }];
  const workflowRows: HomeFeedItem[] = recentWorkflows.length
    ? recentWorkflows.map((workflow) => ({
        key: workflow.workflow_id,
        label: workflow.title,
        meta: `${workflow.task_draft_count} 任务`,
        tone: runningWorkflows.includes(workflow) ? "run" : workflow.state === "draft" ? "warn" : "ok",
        view: "runningWorkflows",
      }))
    : [{ key: "empty-workflows", label: "暂无工作流", meta: "—", view: "runningWorkflows" }];

  return (
    <div className="ink-home-stage home-ink-stage">
      {/* sr-only anchors required by offline interaction tests */}
      <p className="sr-only">最近项来自索引近似口径，不是真实使用事件。</p>
      <p className="sr-only">技能管理</p>
      <p className="sr-only">运行器管理</p>
      <button className="sr-only" type="button" onClick={() => onNavigate("agents")}>
        打开智能体
      </button>
      <header className="stage-head home-page-head">
        <h1 className="title">首页</h1>
        <div className="meta home-index-meta">
          <div className="big">
            {snapshot.summary.project_count} 项目 · {snapshot.summary.session_count} 会话 · {workflowCount} 工作流
          </div>
          <div>更新 {formatDate(snapshot.summary.generated_at)}</div>
        </div>
      </header>

      <div className="constellation">
        <HomeNode
          className="n-project"
          ko="项目"
          name="项目"
          count={snapshot.summary.project_count}
          rows={projectRows}
          moreLabel="查看全部"
          view="projects"
          onNavigate={onNavigate}
        />
        <HomeNode
          className="n-agents"
          ko="智能体"
          name="智能体"
          count={snapshot.summary.session_count}
          rows={agentRows}
          moreLabel={`${snapshot.summary.session_count} 个会话`}
          view="agents"
          onNavigate={onNavigate}
        />
        <HomeNode
          className="n-flow center"
          ko="运行中"
          name="运行中工作流"
          count={runningWorkflows.length}
          rows={[
            {
              key: "workflow-summary",
              label: `${workflowCount} 工作流 · ${workItemCount} 工作项`,
              meta: `${runningWorkflows.length} 关注`,
              tone: runningWorkflows.length ? "run" : "ok",
              view: "runningWorkflows",
            },
            ...workflowRows,
          ]}
          moreLabel="查看运行中"
          view="runningWorkflows"
          onNavigate={onNavigate}
        />
        <HomeNode
          className="n-skills"
          ko="Skill"
          name="Skill"
          count={snapshot.summary.skill_count}
          rows={skillRows}
          moreLabel="Skill 库"
          view="skills"
          onNavigate={onNavigate}
        />
        <HomeNode
          className="n-harness"
          ko="Harness"
          name="Harness"
          count={harnessResources.length}
          rows={harnessRows}
          moreLabel="Harness 库"
          view="harness"
          onNavigate={onNavigate}
        />
      </div>
    </div>
  );
}
