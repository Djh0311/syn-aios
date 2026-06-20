import type React from "react";
import { useEffect, useMemo, useState } from "react";
import { pathTail, relativeTime } from "../../lib/format";
import type { SessionRecord } from "../../lib/types";

export const NO_PROJECT_KEY = "__no_project__";
export const NO_PROJECT_LABEL = "直接聊天";

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
  if (!raw || raw === "user" || raw === "subagent") return "codex";
  return raw;
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
  if (filter === "all") return !session.archived;
  if (filter === "missing") return !session.archived && (!session.rollout_exists || !session.rollout_path);
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

const SESSION_RENDER_WINDOW_SIZE = 40;

export function AgentSessionList({
  sessions,
  visibleSessions,
  groups,
  effectiveGroupBy,
  selectedThreadId,
  newSessionActive,
  filteredOutCount,
  filterBar,
  searchQuery,
  readFilter,
  selectedCollapsedGroup,
  showHeader = false,
  eyebrow,
  title,
  onNewConversation,
  onSearchQueryChange,
  onReadFilterChange,
  collapsedKeys,
  onToggleGroup,
  onOpenSession,
  sessionPageStatus = "idle",
  sessionHasMore = false,
  loadingMoreSessions = false,
  onLoadMoreSessions,
}: {
  sessions: SessionRecord[];
  visibleSessions: SessionRecord[];
  groups: AgentSessionGroup[];
  effectiveGroupBy: "project" | "software";
  selectedThreadId: string | null;
  newSessionActive?: boolean;
  filteredOutCount: number;
  filterBar?: React.ReactNode;
  searchQuery: string;
  readFilter: SessionReadFilter;
  selectedCollapsedGroup: AgentSessionGroup | null;
  collapsedKeys: Set<string>;
  showHeader?: boolean;
  eyebrow?: string;
  title: string;
  description?: string;
  onNewConversation?: () => void;
  onSearchQueryChange: (value: string) => void;
  onReadFilterChange: (filter: SessionReadFilter) => void;
  onToggleGroup: (key: string) => void;
  onOpenSession: (session: SessionRecord) => void;
  sessionPageStatus?: "idle" | "loading" | "error";
  sessionPageSource?: string | null;
  sessionPageWarnings?: string[];
  sessionHasMore?: boolean;
  loadingMoreSessions?: boolean;
  onLoadMoreSessions?: () => void;
}) {
  const [renderLimit, setRenderLimit] = useState(SESSION_RENDER_WINDOW_SIZE);
  useEffect(() => {
    setRenderLimit(SESSION_RENDER_WINDOW_SIZE);
  }, [effectiveGroupBy, readFilter, searchQuery]);

  const windowedGroups = useMemo(() => {
    let remaining = renderLimit;
    const next: AgentSessionGroup[] = [];
    for (const group of groups) {
      if (collapsedKeys.has(group.key)) {
        next.push(group);
        continue;
      }
      if (remaining <= 0) break;
      const shownSessions = group.sessions.slice(0, remaining);
      remaining -= shownSessions.length;
      if (shownSessions.length) {
        next.push({ ...group, sessions: shownSessions });
      }
    }
    return next;
  }, [collapsedKeys, groups, renderLimit]);
  const renderedSessionCount = windowedGroups.reduce(
    (count, group) => count + (collapsedKeys.has(group.key) ? 0 : group.sessions.length),
    0,
  );
  const hasLocalMore = renderedSessionCount < visibleSessions.length;
  const canLoadMore = hasLocalMore || sessionHasMore;

  function handleLoadMore() {
    if (hasLocalMore) {
      setRenderLimit((current) => current + SESSION_RENDER_WINDOW_SIZE);
      return;
    }
    onLoadMoreSessions?.();
  }

  return (
    <aside className="agent-session-list" aria-label="会话列表">
      <header className="agent-list-head">
        {showHeader ? (
          <div>
            {eyebrow ? <p className="pg-sub">{eyebrow}</p> : null}
            <h1 className="agent-list-title">{title}</h1>
            <span className="agent-list-count">
              {visibleSessions.length} / {sessions.length} 会话 · {groups.length} {effectiveGroupBy === "software" ? "软件" : "项目"}
            </span>
          </div>
        ) : null}
        <button
          className={`agent-new-chat-button ${newSessionActive ? "active" : ""}`}
          type="button"
          aria-pressed={newSessionActive}
          onClick={onNewConversation}
        >
          <span aria-hidden="true">+</span>
          新对话
        </button>
      </header>
      {filterBar}
      <div className="session-list-controls">
        <label className="session-search">
          <span>搜索会话</span>
          <input
            aria-label="搜索会话"
            type="search"
            value={searchQuery}
            placeholder="标题 / ID / 项目 / 智能体 / 状态"
            onChange={(event) => onSearchQueryChange(event.currentTarget.value)}
            onKeyDown={(event) => {
              if (event.key === "Escape") {
                onSearchQueryChange("");
                event.currentTarget.blur();
              }
            }}
          />
        </label>
        {sessions.some((session) => session.archived || !session.rollout_exists) ? (
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
        ) : null}
        {sessionPageStatus === "loading" ? (
          <p className="muted small-note session-list-read-state">
            正在更新会话列表。
          </p>
        ) : null}
        {selectedCollapsedGroup ? (
          <p className="session-collapsed-selection">
            当前选中会话在已收纳分组内：{effectiveGroupBy === "project" && selectedCollapsedGroup.key !== NO_PROJECT_KEY ? pathTail(selectedCollapsedGroup.label) : selectedCollapsedGroup.label}
          </p>
        ) : null}
      </div>
      {groups.length === 0 ? (
        <p className="muted small-note">没有符合搜索或过滤条件的会话。已过滤 {filteredOutCount} 条。</p>
      ) : (
        windowedGroups.map((group) => {
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
                      {status.tone !== "ok" ? (
                        <span className={`sc-status ${status.tone}`}>{status.label}</span>
                      ) : null}
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
      {canLoadMore ? (
        <div className="session-list-controls">
          <button
            className="secondary-button"
            type="button"
            disabled={loadingMoreSessions}
            onClick={handleLoadMore}
          >
            显示更多会话
          </button>
          <span className="muted small-note">
            {hasLocalMore ? "继续显示当前匹配结果。" : "继续读取更多会话。"}
          </span>
        </div>
      ) : null}
    </aside>
  );
}
