import { Badge } from "../components/Badge";
import { formatDate } from "../lib/format";
import type { Diagnostics } from "../lib/types";

type DiagnosticsViewProps = {
  diagnostics: Diagnostics;
};

export function DiagnosticsView({ diagnostics }: DiagnosticsViewProps) {
  return (
    <section className="view-stack">
      <div className="section-heading">
        <div>
          <p className="eyebrow">诊断</p>
          <h2>诊断页</h2>
        </div>
        <p className="muted">展示运行边界和索引状态。</p>
      </div>

      <div className="card-list">
        <article className="item-card">
          <div className="item-head">
            <div>
              <h3 className="item-title">数据源</h3>
              <p className="path-text">{diagnostics.index_path}</p>
            </div>
            <Badge tone="candidate">只读索引</Badge>
          </div>
          <div className="detail-grid">
            <Detail label="任务队列" value={diagnostics.tasks_path} />
            <Detail label="生成时间" value={formatDate(diagnostics.generated_at)} />
            <Detail label="顶层警告" value={diagnostics.top_level_warning_count} />
            <Detail label="上下文警告" value={diagnostics.context_warning_count} />
            <Detail label="项目路径白名单" value={diagnostics.allowed_project_path_count} />
            <Detail label="回放记录路径白名单" value={diagnostics.allowed_rollout_path_count} />
            <Detail label="发布包" value={diagnostics.release_bundle_enabled ? "开启" : "关闭"} />
          </div>
        </article>

        <article className="panel">
          <div className="panel-heading">
            <h3>边界说明</h3>
            <Badge tone="unknown">一期</Badge>
          </div>
          <div className="list-stack">
            {diagnostics.notes.map((note) => (
              <div className="mini-row" key={note}>
                <strong>{note}</strong>
                <span>依据：后端诊断元数据。</span>
              </div>
            ))}
          </div>
        </article>
      </div>
    </section>
  );
}

function Detail({ label, value }: { label: string; value: string | number }) {
  return (
    <div className="detail">
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}
