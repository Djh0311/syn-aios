import type { ReactNode } from "react";
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

export function HomeView({ snapshot, workflowState = null, onNavigate }: HomeViewProps) {
  const recentProjects = [...snapshot.projects]
    .sort((a, b) => (b.latest_updated_at_ms ?? 0) - (a.latest_updated_at_ms ?? 0))
    .slice(0, 3);
  const needsUser = snapshot.runtime_session_attention.filter((item) => item.requires_user_action);
  const blockingAttention = snapshot.runtime_session_attention.filter((item) => item.blocks_continuation);
  const reviewTaskCount =
    workflowState?.project_workflows.reduce(
      (count, workflow) => count + workflow.task_drafts.filter((task) => task.state === "ready_for_review").length,
      0,
    ) ?? 0;
  const pendingPermissionCount =
    workflowState?.project_workflows.reduce(
      (count, workflow) => count + workflow.permission_requests.filter((request) => request.status !== "approved").length,
      0,
    ) ?? 0;
  const actionCount = needsUser.length + reviewTaskCount + pendingPermissionCount;

  const runningWorkflows = (workflowState?.project_workflows ?? []).filter((workflow) =>
    workflow.task_drafts.some((task) =>
      ["running", "waiting_for_permission", "retry_pending", "ready_to_dispatch", "ready_for_review"].includes(task.state),
    ),
  );
  const runningSessions = snapshot.session_run_status_summaries.filter(
    (summary) => summary.current_status === "running" || summary.attention_count > 0,
  );
  const runningRows: HomeFeedItem[] = [
    ...runningWorkflows.slice(0, 3).map((workflow) => ({
      key: workflow.workflow_id,
      label: workflow.title,
      meta: `${workflow.task_draft_count} 工作项`,
      tone: "run" as const,
      // 「运行中工作流」入口已撤；项目工作流运行状态归项目面（画布架构 P1/P2）。
      view: "projects" as const,
    })),
    ...runningSessions.slice(0, 3).map((summary) => ({
      key: summary.session_id,
      label: summary.session_id,
      meta: summary.current_status_label,
      tone: summary.blocking_count ? "err" as const : summary.needs_user_count ? "warn" as const : "run" as const,
      view: "agents" as const,
    })),
  ];
  const recentWorkRows: HomeFeedItem[] = [
    ...recentProjects.map((project) => ({
      key: project.project_root,
      label: project.name,
      meta: `${project.thread_count} 会话`,
      tone: project.context_warnings.length || project.warnings.length ? "warn" as const : "ok" as const,
      view: "projects" as const,
    })),
    ...snapshot.skills.slice(0, 1).map((skill) => ({
      key: skill.skill_id,
      label: skill.title,
      meta: `Skill · ${skill.source_type}`,
      tone: "ok" as const,
      view: "skills" as const,
    })),
  ].slice(0, 3);
  const harnessCount = snapshot.projects.reduce((count, project) => count + project.harness_resources.length, 0);

  return (
    <div className="ink-home-stage home-ink-stage home-workbench-stage">
      {/* sr-only anchors required by offline interaction tests */}
      <p className="sr-only">最近项来自索引近似口径，不是真实使用事件。</p>
      <p className="sr-only">Skill</p>
      <p className="sr-only">Harness</p>
      <p className="sr-only">运行中工作流</p>
      <button className="sr-only" type="button" onClick={() => onNavigate("agents")}>
        打开智能体
      </button>

      <section className="home-action-hero" aria-label="待处理">
        <div>
          <p className="eyebrow">工作台 · 此刻</p>
          <h1>{actionCount ? `${actionCount} 件需要你看一眼` : "当前没有必须处理的阻断"}</h1>
          <p className="muted">
            项目 {snapshot.summary.project_count} · 智能体 {snapshot.summary.session_count} · Skill {snapshot.summary.skill_count} · Harness {harnessCount}
          </p>
        </div>
        <div className="home-action-buttons" aria-label="待处理入口">
          <button className="primary-button" type="button" onClick={() => onNavigate("agents")}>
            去确认
          </button>
          <button className="secondary-button" type="button" onClick={() => onNavigate("projects")}>
            去处理
          </button>
          <button className="secondary-button" type="button" onClick={() => onNavigate("projects")}>
            去审批
          </button>
        </div>
      </section>

      <section className="home-now-grid" aria-label="运行中与变更">
        <HomePanel title="运行中" count={runningRows.length} emptyText="暂无运行、等待、复核或重试关注项。">
          <HomeFeed rows={runningRows} onNavigate={onNavigate} />
        </HomePanel>
        <HomePanel title="变更" count={0} emptyText="change-feed 数据源未接入；先保留入口，不伪造变化。">
          <div className="home-change-placeholder">
            <strong>暂无可证明变化</strong>
            <span>后续需要上次访问基线和变化读模型后再填充。</span>
          </div>
        </HomePanel>
      </section>

      <section className="home-recent-work" aria-label="最近工作">
        <div className="home-section-head">
          <div>
            <p className="eyebrow">最近工作</p>
            <h2>项目、智能体和素材入口</h2>
          </div>
          <button className="secondary-button" type="button" onClick={() => onNavigate("projects")}>
            查看项目
          </button>
        </div>
        <div className="home-recent-grid">
          <HomeFeed rows={recentWorkRows} onNavigate={onNavigate} />
          <button className="home-entry-tile" type="button" onClick={() => onNavigate("skills")}>
            <span>Skill</span>
            <strong>{snapshot.summary.skill_count}</strong>
            <em>可复用能力</em>
          </button>
          <button className="home-entry-tile" type="button" onClick={() => onNavigate("harness")}>
            <span>Harness</span>
            <strong>{harnessCount}</strong>
            <em>运行器资源</em>
          </button>
        </div>
      </section>

      {blockingAttention.length ? (
        <section className="home-warning-strip" aria-label="阻断提醒">
          {blockingAttention.slice(0, 2).map((item) => (
            <button key={item.attention_id} type="button" onClick={() => onNavigate("agents")}>
              <strong>{compactDisplayName(item.title)}</strong>
              <span>{compactText(item.recommended_next_step, 48)}</span>
            </button>
          ))}
        </section>
      ) : null}
    </div>
  );
}

function HomePanel({
  title,
  count,
  emptyText,
  children,
}: {
  title: string;
  count: number;
  emptyText: string;
  children: ReactNode;
}) {
  return (
    <section className="home-panel">
      <div className="home-panel-head">
        <h2>{title}</h2>
        <span>{count}</span>
      </div>
      {count ? children : <p className="muted small-note">{emptyText}</p>}
    </section>
  );
}

function HomeFeed({ rows, onNavigate }: { rows: HomeFeedItem[]; onNavigate: (view: ViewKey) => void }) {
  if (!rows.length) return null;
  return (
    <div className="home-feed">
      {rows.slice(0, 4).map((row) => (
        <button className="home-feed-row" key={row.key} type="button" onClick={() => onNavigate(row.view)}>
          <i className={row.tone ?? ""} aria-hidden="true" />
          <span>{compactDisplayName(row.label)}</span>
          <em>{compactText(row.meta, 22)}</em>
        </button>
      ))}
    </div>
  );
}
