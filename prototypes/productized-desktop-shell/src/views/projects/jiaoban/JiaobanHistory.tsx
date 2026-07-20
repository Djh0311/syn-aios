// 交办·历届方案索引——由原工作历史栏演化而来，数据源与九态语义不变。
// 宪法归属:修宪3号「话在左·物右侧双列」；本组件只控制右侧方案/交货实体，不再控制对话锚点。
import type { RunHistoryEntry } from "../../../lib/types";
import { historyErrorFamilyLabel, humanizeVerdict } from "../../../lib/humanize";
import { formatProposalTime } from "./jiaobanTime";

type HistoryVisual = { dot: string; toneClass: string; word: string };
export type HistoryFilter = "all" | "mine" | "running";

function historyHasAttention(entry: RunHistoryEntry): boolean {
  const r = entry.review_flags.result_verdict;
  const b = entry.review_flags.boundary_verdict;
  return r === "needs_human_check" || r === "needs_rework" || r === "human_verify" || b === "mismatch";
}

// 九态英文键 → 状态点 + 配色 class + 短词（词表死线：主路径人话，不露英文枚举）。
function historyStateVisual(entry: RunHistoryEntry): HistoryVisual {
  switch (entry.state) {
    case "running":
      return { dot: "●", toneClass: "run-dot-running", word: "跑着" };
    case "blocked":
      return { dot: "⚠", toneClass: "run-dot-blocked", word: "卡住" };
    case "delivered":
      return {
        dot: "✓",
        toneClass: historyHasAttention(entry) ? "run-dot-attention" : "run-dot-done",
        word: "交货",
      };
    case "pending":
      return { dot: "○", toneClass: "run-dot-pending", word: "等你批" };
    case "advice_only":
      return { dot: "◐", toneClass: "run-dot-advice", word: "纯建议" };
    case "confirmed_not_run":
      return { dot: "◌", toneClass: "run-dot-confirmed", word: "批了没跑" };
    case "declined":
      return { dot: "·", toneClass: "run-dot-terminal", word: "已回绝" };
    case "superseded":
      return { dot: "·", toneClass: "run-dot-terminal", word: "被替代" };
    case "changes_requested":
      return { dot: "·", toneClass: "run-dot-terminal", word: "要改" };
    default:
      return { dot: "·", toneClass: "run-dot-terminal", word: entry.state_note || "—" };
  }
}

// 人话工程①(2026-07-20):historyErrorFamilyLabel 逐字迁 src/lib/humanize.ts,顶部 import-back。

function matchesHistoryFilter(entry: RunHistoryEntry, filter: HistoryFilter): boolean {
  if (filter === "running") return entry.state === "running";
  if (filter === "mine")
    return (
      entry.state === "pending" ||
      entry.state === "blocked" ||
      (entry.state === "delivered" && historyHasAttention(entry))
    );
  return true;
}

// 方案索引只显示日期：当年 MM-DD，跨年 YYYY-MM-DD。
function formatProposalIndexDate(createdAtMs: number): string {
  const d = new Date(createdAtMs);
  const monthDay = `${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
  return d.getFullYear() === new Date().getFullYear() ? monthDay : `${d.getFullYear()}-${monthDay}`;
}

// 人话工程①(2026-07-20):humanizeVerdict 逐字迁 src/lib/humanize.ts,顶部 import-back。

export function JiaobanProposalIndex({
  entries,
  total,
  loading,
  filter,
  onFilterChange,
  selectedId,
  currentProposalId,
  latestBlockedId,
  onSelectEntry,
  onBackToCurrent,
  onNewJiaoban,
  onContinueRun,
  knownProposalIds,
}: {
  entries: RunHistoryEntry[];
  total: number;
  loading: boolean;
  filter: HistoryFilter;
  onFilterChange: (filter: HistoryFilter) => void;
  selectedId: string | null;
  currentProposalId: string | null;
  latestBlockedId: string | null;
  onSelectEntry: (entry: RunHistoryEntry) => void;
  onBackToCurrent: () => void;
  onNewJiaoban: () => void;
  onContinueRun: () => void;
  // null=方案店尚不可用，不能武断标成旧单；Set=可据实判断右侧是否有方案实体。
  knownProposalIds: ReadonlySet<string> | null;
}) {
  const mineCount = entries.filter((entry) => matchesHistoryFilter(entry, "mine")).length;
  const runningCount = entries.filter((entry) => entry.state === "running").length;
  const visible = entries.filter((entry) => matchesHistoryFilter(entry, filter));
  return (
    <aside className="jiaoban-history" aria-label="历届方案">
      <div className="jiaoban-history-head">
        <strong>历届方案</strong>
        {/* 07-15 走查:头部入口降为次级小钮——空态里的 [+新交办] 才是主动作,俩黑色主钮叠着打架。 */}
        <button
          className="secondary-button jiaoban-history-new"
          type="button"
          onClick={onNewJiaoban}
          aria-label="新交办"
          title="新交办"
        >
          +
        </button>
      </div>
      <div className="jiaoban-history-filters">
        <button className={`jiaoban-chip ${filter === "all" ? "on" : ""}`} type="button" onClick={() => onFilterChange("all")}>
          全部
        </button>
        <button className={`jiaoban-chip ${filter === "mine" ? "on" : ""}`} type="button" onClick={() => onFilterChange("mine")}>
          等我的{mineCount ? ` ${mineCount}` : ""}
        </button>
        <button className={`jiaoban-chip ${filter === "running" ? "on" : ""}`} type="button" onClick={() => onFilterChange("running")}>
          跑着{runningCount ? ` ${runningCount}` : ""}
        </button>
      </div>
      <div className="jiaoban-history-list">
        {loading && entries.length === 0 ? (
          <p className="muted small-note">正在读历届方案…</p>
        ) : total === 0 ? (
          <div className="jiaoban-history-empty">
            <p className="muted">这个项目还没有方案记录。</p>
            <button className="primary-button" type="button" onClick={onNewJiaoban}>
              + 新交办
            </button>
          </div>
        ) : visible.length === 0 ? (
          <p className="muted small-note">这个筛选下没有单子。</p>
        ) : (
          visible.map((entry) => {
            const visual = historyStateVisual(entry);
            const isCurrent = currentProposalId != null && entry.proposal_id === currentProposalId;
            const isSelected = selectedId === entry.proposal_id || (selectedId === null && isCurrent);
            const hasProposalRecord = knownProposalIds == null || knownProposalIds.has(entry.proposal_id);
            const canvasView = entry.state === "delivered" ? "delivery" : "proposal";
            const dateLabel = formatProposalIndexDate(entry.created_at_ms);
            // 修宪3号:目标一句 + 九态人话状态 + 日期；缺实体只如实标旧单，不补造方案。
            // 行内[接着跑]仍不回流索引——它只属于卡住脸。
            return (
              <button
                key={entry.proposal_id}
                className={`jiaoban-run ${isSelected ? "on" : ""}`}
                type="button"
                title={`${entry.state_note} · ${entry.goal_text || "（没写目标）"}${hasProposalRecord ? "" : " · 旧单·无方案记录"}`}
                aria-controls={`jiaoban-canvas-view-${canvasView}`}
                onClick={() => (isCurrent && entry.state !== "delivered" ? onBackToCurrent() : onSelectEntry(entry))}
              >
                <span className={`jiaoban-run-dot ${visual.toneClass}`} aria-hidden="true">
                  {visual.dot}
                </span>
                <span className="jiaoban-run-goal">{entry.goal_text || "（没写目标）"}</span>
                <span className="jiaoban-run-meta">
                  <span className="jiaoban-run-state">{visual.word}</span>
                  {!hasProposalRecord ? <span className="jiaoban-run-legacy">旧单·无方案记录</span> : null}
                  <time className="jiaoban-run-time" dateTime={new Date(entry.created_at_ms).toISOString()}>
                    {dateLabel}
                  </time>
                </span>
              </button>
            );
          })
        )}
      </div>
    </aside>
  );
}

// 选中「非当前」历史单 → 主区显只读详情卡（不回放完整五态脸·诚实显读模型有的）。
export function JiaobanHistoryDetail({
  entry,
  onBackToCurrent,
  showBackAction = true,
}: {
  entry: RunHistoryEntry;
  onBackToCurrent: () => void;
  showBackAction?: boolean;
}) {
  const visual = historyStateVisual(entry);
  const flags = entry.review_flags;
  const opinions = [
    flags.result_verdict ? `结果复核：${humanizeVerdict(flags.result_verdict)}` : null,
    flags.boundary_verdict ? `批前边界：${humanizeVerdict(flags.boundary_verdict)}` : null,
  ].filter(Boolean);
  return (
    <div
      className="project-canvas-detail-card jiaoban-history-detail"
      aria-label="历史单详情"
    >
      <div className="panel-heading">
        <div>
          <p className="eyebrow">历届方案 · 这一单</p>
          <h3>
            <span className={`jiaoban-run-dot ${visual.toneClass}`} aria-hidden="true">
              {visual.dot}
            </span>{" "}
            {visual.word}
          </h3>
        </div>
        {showBackAction ? (
          <button className="secondary-button" type="button" onClick={onBackToCurrent}>
            回到当前
          </button>
        ) : null}
      </div>
      <p className="jiaoban-field">
        <span className="jiaoban-field-label">目标：</span>
        {entry.goal_text || "（没写目标）"}
      </p>
      <p className="jiaoban-field">
        <span className="jiaoban-field-label">状态：</span>
        {entry.state_note}
      </p>
      {/* A·运行错误两层脸：默认显人话摘要+族标；「查看原文」下钻原始 stderr。呈现不阻断（同黄牌哲学）。 */}
      {entry.error ? (
        <div className="jiaoban-run-error" aria-label="运行错误诊断">
          <p className="jiaoban-field">
            <span className="jiaoban-field-label">出错：</span>
            <span className="jiaoban-run-error-family">
              {historyErrorFamilyLabel(entry.error.family)}
            </span>
            <span className="jiaoban-run-error-human">{entry.error.human}</span>
          </p>
          {entry.error.raw_snippet.trim() ? (
            <details className="jiaoban-run-error-raw">
              <summary>查看原文</summary>
              <pre>{entry.error.raw_snippet}</pre>
            </details>
          ) : null}
        </div>
      ) : null}
      <p className="jiaoban-field">
        <span className="jiaoban-field-label">时间：</span>
        {formatProposalTime(entry.created_at_ms)}
      </p>
      {entry.chain ? (
        <p className="jiaoban-field">
          <span className="jiaoban-field-label">进度：</span>
          做到第 {entry.chain.done_count}/{entry.chain.total_count} 步
        </p>
      ) : null}
      {opinions.length ? (
        <p className="jiaoban-field">
          <span className="jiaoban-field-label">意见：</span>
          {opinions.join("；")}
        </p>
      ) : null}
      {entry.correlation === "time_window" ? (
        <p className="muted small-note">意见归属按时间近似匹配（不是精确挂到这一单）。</p>
      ) : null}
      <p className="muted small-note">具体每一步的过程，看右侧画布。</p>
    </div>
  );
}
