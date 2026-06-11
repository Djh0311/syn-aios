import { useMemo } from "react";
import { formatDate } from "../../lib/format";
import { deriveProjectsPageReadModelFromParts } from "../../lib/pageSelectors";
import type { ProjectRecord, SessionRecord, WorkflowStateSnapshot } from "../../lib/types";

export function ProjectGallery({
  projects,
  sessions,
  workflowState,
  onSelectProject,
}: {
  projects: ProjectRecord[];
  sessions: SessionRecord[];
  workflowState: WorkflowStateSnapshot | null;
  onSelectProject: (projectRoot: string) => void;
}) {
  const pageReadModel = useMemo(
    () => deriveProjectsPageReadModelFromParts({ projects, sessions, workflowState }),
    [projects, sessions, workflowState],
  );
  const sortedProjects = useMemo(
    () => [...pageReadModel.projects].sort((a, b) => (b.latest_updated_at_ms ?? 0) - (a.latest_updated_at_ms ?? 0)),
    [pageReadModel.projects],
  );
  const workflowProjectCount = pageReadModel.projects.filter((project) => project.workflow_count > 0).length;
  const totalWarnings = pageReadModel.projects.reduce((sum, project) => sum + project.warning_count, 0);

  return (
    <section className="project-gallery stage-pad">
      <div className="pg-head">
        <div>
          <p className="pg-sub">项目 · 方块入口</p>
          <h1 className="pg-title">项 目 入 口</h1>
        </div>
        <div className="pg-meta">
          <div className="big">{pageReadModel.project_count} 项目 · {pageReadModel.total_session_count} 会话</div>
          <div>{workflowProjectCount} 个项目有工作流草稿 · {totalWarnings} 个警告</div>
        </div>
      </div>

      <div className="project-card-grid" aria-label="项目方块列表">
        {sortedProjects.map((project) => {
          const fileCount = project.authority_count + project.handoff_count + project.evidence_count;
          return (
            <button
              className={`project-tile ${project.active_hint ? "active" : ""}`}
              key={project.project_root}
              type="button"
              onClick={() => onSelectProject(project.project_root)}
              title={project.project_root}
            >
              <span className="project-tile-seal" aria-hidden="true">{projectInitials(project.name)}</span>
              <span className="project-tile-main">
                <strong>{project.name}</strong>
                <span className="project-tile-path">{project.project_root}</span>
              </span>
              <span className="project-tile-meta">
                <span>最近更新</span>
                <em>{formatDate(project.latest_updated_at_ms)}</em>
              </span>
              <span className="project-tile-stats">
                <span><b>{project.session_count}</b> 会话</span>
                <span><b>{project.workflow_count}</b> 工作流</span>
                <span><b>{fileCount}</b> 文件</span>
                <span><b>{project.warning_count}</b> 警告</span>
              </span>
            </button>
          );
        })}
      </div>
    </section>
  );
}

function projectInitials(name: string) {
  const clean = name.trim();
  if (!clean) return "项";
  const asciiParts = clean.split(/[-_\s/]+/).filter(Boolean);
  if (asciiParts.length > 1) return asciiParts.slice(0, 2).map((part) => part[0]).join("").toUpperCase();
  return clean.slice(0, 2).toUpperCase();
}
