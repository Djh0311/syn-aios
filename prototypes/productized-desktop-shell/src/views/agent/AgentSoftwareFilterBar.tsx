import { softwareLabelOf } from "./AgentSessionList";

export function AgentSoftwareFilterBar({
  activeKey,
  counts,
  total,
  onChange,
}: {
  activeKey: string | null;
  counts: Array<{ key: string; label: string; count: number }>;
  total: number;
  onChange: (key: string | null) => void;
}) {
  if (counts.length <= 1) return null;
  return (
    <div className="session-filter-bar" role="group" aria-label="按软件筛选会话">
      <button
        className={`filter-chip ${activeKey === null ? "active" : ""}`}
        type="button"
        onClick={() => onChange(null)}
      >
        全部 <em>{total}</em>
      </button>
      {counts.map((row) => (
        <button
          className={`filter-chip ${activeKey === row.key ? "active" : ""}`}
          key={row.key}
          type="button"
          onClick={() => onChange(row.key)}
        >
          {row.label || softwareLabelOf(row.key)} <em>{row.count}</em>
        </button>
      ))}
    </div>
  );
}
