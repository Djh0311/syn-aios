import { Badge } from "../components/Badge";
import type { ProjectRecord, TaskEntry } from "../lib/types";

type TasksEvidenceViewProps = {
  tasks: TaskEntry[];
  projects: ProjectRecord[];
};

export function TasksEvidenceView({ tasks, projects }: TasksEvidenceViewProps) {
  const statuses = ["待派发", "进行中", "已回收", "暂停"];
  const handoffCount = projects.reduce((sum, project) => sum + project.handoff_files.length, 0);
  const evidenceCount = projects.reduce((sum, project) => sum + project.evidence_files.length, 0);

  return (
    <section className="view-stack">
      <div className="section-heading">
        <div>
          <p className="eyebrow">任务线与证据入口</p>
          <h2>任务线 / 证据 / 交接页</h2>
        </div>
        <p className="muted">只展示任务入口和候选路径，不展开正文。</p>
      </div>

      <div className="metric-grid compact-metrics">
        <article className="metric">
          <span>证据候选</span>
          <strong>{evidenceCount}</strong>
          <small>来自项目上下文扫描</small>
        </article>
        <article className="metric">
          <span>交接候选</span>
          <strong>{handoffCount}</strong>
          <small>只显示路径，不显示正文</small>
        </article>
      </div>

      <div className="content-grid four">
        {statuses.map((status) => (
          <article className="panel task-column" key={status}>
            <h3>{status}</h3>
            <div className="list-stack">
              {tasks.filter((task) => task.status === status).length ? (
                tasks
                  .filter((task) => task.status === status)
                  .map((task) => (
                    <div className="mini-row" key={`${status}-${task.title}`}>
                      <strong>{task.title}</strong>
                      <span>任务队列候选入口；不展开说明正文。</span>
                      <Badge tone={status === "已回收" ? "candidate" : status === "暂停" ? "warning" : "unknown"}>{status}</Badge>
                    </div>
                  ))
              ) : (
                <p className="empty-line">暂无条目。</p>
              )}
            </div>
          </article>
        ))}
      </div>
    </section>
  );
}
