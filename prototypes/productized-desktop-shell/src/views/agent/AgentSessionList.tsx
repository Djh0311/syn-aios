import type React from "react";
import { pathTail, relativeTime } from "../../lib/format";
import type { SessionRecord } from "../../lib/types";

export const NO_PROJECT_KEY = "__no_project__";
export const NO_PROJECT_LABEL = "未关联项目";

const SOFTWARE_LABELS: Record<string, string> = {
  codex: "Codex",
  "claude-code": "Claude Code",
  claude_code: "Claude Code",
  openclaw: "OpenClaw",
};

export type SessionReadFilter = "readable" | "all" | "missing" | "archived";

export const SESSION_READ_FILTERS: Array<{ key: SessionReadFilter; label: string }> = [
  { key: "readable", label: "可读取" },
  { key: "all", label: "全部" },
  { key: "missing", label: "缺回放记录" },
  { key: "archived", label: "已归档" },
];

export function softwareKeyOf(session: SessionRecord): string {
  const raw = (session.thread_source ?? "codex").trim().toLowerCase();
  return raw || "codex";
}

export function softwareLabelOf(key: string): string {
  return SOFTWARE_LABELS[key] ?? key.replace(/[-_]/g, " ").replace(/\b\w/g, (c) => c.toUpperCase());
}

export function statusOf(session: SessionRecord): { tone: "ok" | "warn" | "err" | "run" | "muted"; label: string } {
  if (session.archived) return { tone: "muted", label: "已归档" };
  if (!session.rollout_exists) return { tone: "warn", label: "缺回放记录" };
  if (session.warnings.length) return { tone: "warn", label: session.warnings[0] };
  return { tone: "ok", label: "可读取" };
}

export function sessionMatchesReadFilter(session: SessionRecord, filter: SessionReadFilter): boolean {
  if (filter === "all") return true;
  if (filter === "missing") return !session.rollout_exists || !session.rollout_path;
  if (filter === "archived") return session.archived;
  return !session.archived && !!session.rollout_exists && !!session.rollout_path;
}

export function sessionMatchesSearch(session: SessionRecord, query: string): boolean {
  const normalized = query.trim().toLowerCase();
  if (!normalized) return true;
  const status = statusOf(session);
  const values = [
    session.title,
    session.thread_id,
    session.project_root,
    session.project_root ? pathTail(session.project_root) : "",
    session.rollout_path,
    session.rollout_path ? pathTail(session.rollout_path) : "",
    session.model,
    session.reasoning_effort,
    softwareLabelOf(softwareKeyOf(session)),
    status.label,
    ...session.warnings,
  ];
  return values.some((value) => (value ?? "").toLowerCase().includes(normalized));
}

export function filterAgentSessions(
  sessions: SessionRecord[],
  readFilter: SessionReadFilter,
  searchQuery: string,
): SessionRecord[] {
  return sessions.filter(
    (session) => sessionMatchesReadFilter(session, readFilter) && sessionMatchesSearch(session, searchQuery),
  );
}

export function softwareGroupsForSessions(sessions: SessionRecord[]) {
  const map = new Map<string, { label: string; sessions: SessionRecord[] }>();
  for (const s of sessions) {
    const key = softwareKeyOf(s);
    const label = softwareLabelOf(key);
    const bucket = map.get(key) ?? { label, sessions: [] };
    bucket.sessions.push(s);
    map.set(key, bucket);
  }
  const known = ["codex", "claude-code", "claude_code", "openclaw"];
  const arr = Array.from(map.entries()).map(([key, value]) => ({
    key,
    label: value.label,
    sessions: value.sessions,
  }));
  arr.sort((a, b) => {
    const ai = known.indexOf(a.key);
    const bi = known.indexOf(b.key);
    const an = ai === -1 ? known.length : ai;
    const bn = bi === -1 ? known.length : bi;
    if (an !== bn) return an - bn;
    return a.label.localeCompare(b.label);
  });
  return arr;
}


export type AgentSessionGroup = {
  key: string;
  label: string;
  sessions: SessionRecord[];
};

export function AgentSessionList({
  sessions,
  visibleSessions,
  groups,
  effectiveGroupBy,
  selectedThreadId,
  filteredOutCount,
  filterBar,
  searchQuery,
  readFilter,
  selectedCollapsedGroup,
  showSoftware,
  eyebrow,
  title,
  description,
  onSearchQueryChange,
  onReadFilterChange,
  collapsedKeys,
  onToggleGroup,
  onOpenSession,
}: {
  // 当前仍由适配层传入数组；后续可替换成分页、虚拟滚动或直读数据库数据源。
  sessions: SessionRecord[];
  visibleSessions: SessionRecord[];
  groups: AgentSessionGroup[];
  effectiveGroupBy: "project" | "software";
  selectedThreadId: string | null;
  filteredOutCount: number;
  filterBar?: React.ReactNode;
  searchQuery: string;
  readFilter: SessionReadFilter;
  selectedCollapsedGroup: AgentSessionGroup | null;
  collapsedKeys: Set<string>;
  showSoftware: boolean;
  eyebrow?: string;
  title: string;
  description?: string;
  onSearchQueryChange: (value: string) => void;
  onReadFilterChange: (filter: SessionReadFilter) => void;
  onToggleGroup: (key: string) => void;
  onOpenSession: (session: SessionRecord) => void;
}) {
  return (
    <aside className="agent-session-list" aria-label="会话列表">
      {!showSoftware ? (
        <header className="agent-list-head">
          {eyebrow ? <p className="pg-sub">{eyebrow}</p> : null}
          <h1 className="agent-list-title">{title}</h1>
          <span className="agent-list-count">
            {visibleSessions.length} / {sessions.length} 会话 · {groups.length} {effectiveGroupBy === "software" ? "软件" : "项目"}
          </span>
          {description ? <p className="agent-list-desc">{description}</p> : null}
        </header>
      ) : null}
      {filterBar}
      <div className="session-list-controls">
        <label className="session-search">
          <span>搜索会话</span>
          <input
            aria-label="搜索会话"
            type="search"
            value={searchQuery}
            placeholder="标题 / 项目 / 智能体 / 状态"
            onChange={(event) => onSearchQueryChange(event.currentTarget.value)}
            onKeyDown={(event) => {
              if (event.key === "Escape") {
                onSearchQueryChange("");
                event.currentTarget.blur();
              }
            }}
          />
        </label>
        <div className="session-state-filter" role="group" aria-label="按读取状态筛选会话">
          {SESSION_READ_FILTERS.map((filter) => (
            <button
              className={`filter-chip ${readFilter === filter.key ? "active" : ""}`}
              key={filter.key}
              type="button"
              onClick={() => onReadFilterChange(filter.key)}
            >
              {filter.label} <em>{sessions.filter((session) => sessionMatchesReadFilter(session, filter.key)).length}</em>
            </button>
          ))}
        </div>
        {selectedCollapsedGroup ? (
          <p className="session-collapsed-selection">
            当前选中会话在已收纳分组内：{effectiveGroupBy === "project" && selectedCollapsedGroup.key !== NO_PROJECT_KEY ? pathTail(selectedCollapsedGroup.label) : selectedCollapsedGroup.label}
          </p>
        ) : null}
      </div>
      {groups.length === 0 ? (
        <p className="muted small-note">没有符合搜索或过滤条件的会话。已过滤 {filteredOutCount} 条。</p>
      ) : (
        groups.map((group) => {
          const collapsed = collapsedKeys.has(group.key);
          return (
          <div className={`session-group ${collapsed ? "collapsed" : ""}`} key={group.key}>
            <button
              type="button"
              className="session-group-head"
              aria-expanded={!collapsed}
              onClick={() => onToggleGroup(group.key)}
            >
              <span className="sg-caret" aria-hidden="true">{collapsed ? "▸" : "▾"}</span>
              <span className="sg-name" title={group.label}>
                {effectiveGroupBy === "project" && group.key !== NO_PROJECT_KEY ? pathTail(group.label) : group.label}
              </span>
              <span className="sg-sub">{group.sessions.length}</span>
            </button>
            {collapsed ? null : (
            <div className="session-card-list">
              {group.sessions.map((session) => {
                const status = statusOf(session);
                const isActive = session.thread_id === selectedThreadId;
                const disabled = !session.rollout_exists || !session.rollout_path;
                return (
                  <button
                    key={session.thread_id}
                    type="button"
                    className={`session-card ${isActive ? "active" : ""} ${disabled ? "disabled" : ""}`}
                    disabled={disabled}
                    onClick={() => !disabled && onOpenSession(session)}
                    title={session.title || "未命名会话"}
                  >
                    <span className="sc-line-1">
                      <span className={`sc-dot ${status.tone}`} aria-hidden="true" />
                      <span className="sc-title">{session.title || "未命名会话"}</span>
                    </span>
                    <span className="sc-line-2">
                      <span className="sc-time">{relativeTime(session.updated_at_ms)}</span>
                      <span className={`sc-status ${status.tone}`}>{status.label}</span>
                    </span>
                  </button>
                );
              })}
            </div>
            )}
          </div>
          );
        })
      )}
    </aside>
  );
}
