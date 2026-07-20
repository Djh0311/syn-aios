import { Pill } from "../../components/SpecPrimitives";
import type { FileCandidate, ProjectRecord } from "../../lib/types";

export function ProjectHandoffEvidencePanel({
  project,
  compact = false,
}: {
  project: ProjectRecord;
  compact?: boolean;
}) {
  const fileCount = project.handoff_files.length + project.evidence_files.length + project.authority_files.length;
  const fileColumns = (
    <div className="project-file-columns">
      <ProjectFileList title="当前权威" files={project.authority_files} emptyText="没有 authority 文件索引" />
      <ProjectFileList title="交接" files={project.handoff_files} emptyText="没有交接文件索引" />
      <ProjectFileList title="证据" files={project.evidence_files} emptyText="没有证据文件索引" />
    </div>
  );

  return (
    <section className={`project-evidence-panel ${compact ? "compact" : ""}`}>
      <div className="panel-heading">
        <div>
          <p className="eyebrow">交接 / 证据 / 权威</p>
          <h3>{compact ? "最近资料摘要" : "项目资料索引"}</h3>
        </div>
        <Pill tone="unknown">{fileCount} 文件</Pill>
      </div>
      {compact ? (
        fileColumns
      ) : (
        <details className="project-disclosure" open={fileCount <= 3}>
          <summary>展开完整资料索引</summary>
          {fileColumns}
        </details>
      )}
    </section>
  );
}

export function ProjectResourcesPanel({ project }: { project: ProjectRecord }) {
  const resourceCount = project.harness_resources.length + project.harness_candidates.length;
  const resourceGrid = (
    <div className="project-resource-grid">
      <article>
        <strong>运行器资源</strong>
        {project.harness_resources.length ? (
          project.harness_resources.slice(0, 4).map((resource) => (
            <span key={resource.root_path}>{resource.display_name ?? resource.root_path}</span>
          ))
        ) : (
          <span>没有运行器资源索引</span>
        )}
      </article>
      <article>
        <strong>运行器候选</strong>
        {project.harness_candidates.length ? (
          project.harness_candidates.slice(0, 4).map((candidate) => (
            <span key={candidate.path}>{candidate.name ?? candidate.path}</span>
          ))
        ) : (
          <span>没有运行器候选索引</span>
        )}
      </article>
      <article>
        <strong>项目设置</strong>
        <span>路径：{project.project_root}</span>
        <span>上下文警告：{project.context_warnings.length}</span>
        <span>项目警告：{project.warnings.length}</span>
      </article>
    </div>
  );

  return (
    <section className="project-resources-panel">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">资源</p>
          <h3>技能、运行器和项目级设置分散在对应资源里</h3>
        </div>
        <Pill tone="unknown">{resourceCount} 项</Pill>
      </div>
      <details className="project-disclosure" open={resourceCount <= 2}>
        <summary>展开资源详情</summary>
        {resourceGrid}
      </details>
    </section>
  );
}

function ProjectFileList({
  title,
  files,
  emptyText,
}: {
  title: string;
  files: FileCandidate[];
  emptyText: string;
}) {
  return (
    <article className="project-file-list">
      <strong>{title}</strong>
      {files.length ? (
        files.slice(0, 6).map((file) => (
          <span key={file.path} title={file.path}>
            {file.name ?? file.path}
            {file.warnings.length ? <em>{file.warnings.join(", ")}</em> : null}
          </span>
        ))
      ) : (
        <span>{emptyText}</span>
      )}
    </article>
  );
}
