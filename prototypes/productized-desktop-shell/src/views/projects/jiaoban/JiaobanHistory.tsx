// 交办·工作历史(回顾面)——阶段3拆巨石第一刀:自 ProjectJiaobanPanel.tsx 原样迁出,零逻辑改动。
// 宪法归属:§六 回顾面(唯一问题=我要找的那单多快找到;永不打断)。
import type { RunHistoryEntry } from "../../../lib/types";
import { formatProposalTime, proposalAgeDays } from "./jiaobanTime";

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

// A·错误族 → 人话短标（前端映射·不露 family 机器键；未知族兜底「运行错误」）。
function historyErrorFamilyLabel(family: string): string {
  switch (family) {
    case "provider_unavailable":
      return "供给不可用";
    case "network":
      return "网络抽风";
    case "timeout":
      return "超时";
    case "sandbox_denied":
      return "权限/沙箱";
    case "command_failed":
      return "命令失败";
    case "codex_subsystem":
      return "codex 子系统";
    case "readback_failed":
      return "口供没读回";
    default:
      return "运行错误";
  }
}

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

// 紧凑时间：今天 HH:mm / 昨天 / 更早 MM-DD。复用 proposalAgeDays 的日历日判据。
function formatHistoryTime(createdAtMs: number): string {
  const days = proposalAgeDays(createdAtMs);
  const d = new Date(createdAtMs);
  if (days <= 0) return `${String(d.getHours()).padStart(2, "0")}:${String(d.getMinutes()).padStart(2, "0")}`;
  if (days === 1) return "昨天";
  return `${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
}

// verdict 英文枚举 → 人话（词表死线）。
function humanizeVerdict(v: string): string {
  switch (v) {
    case "pass":
      return "通过";
    case "needs_rework":
      return "要返工";
    case "needs_human_check":
    case "human_verify":
      return "建议你亲验";
    case "looks_ok":
      return "看着没问题";
    case "mismatch":
      return "对不上目标";
    case "caution":
      return "留个心";
    default:
      return v;
  }
}

export function JiaobanHistoryColumn({
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
}) {
  const mineCount = entries.filter((entry) => matchesHistoryFilter(entry, "mine")).length;
  const runningCount = entries.filter((entry) => entry.state === "running").length;
  const visible = entries.filter((entry) => matchesHistoryFilter(entry, filter));
  return (
    <aside className="jiaoban-history" aria-label="工作历史">
      <div className="jiaoban-history-head">
        <strong>工作历史</strong>
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
          <p className="muted small-note">正在读工作历史…</p>
        ) : total === 0 ? (
          <div className="jiaoban-history-empty">
            <p className="muted">这个项目还没交办过活。</p>
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
            // 07-15 走查#2·对齐定稿+宪法:行=单行三元素(状态点+标题+时间);state_note 人话进悬停与详情卡;
            // 行内[接着跑]快捷钮拆除——canon §四.2 明写「[接着跑]在卡住脸不在历史栏」,旧快捷是修宪前遗留。
            return (
              <button
                key={entry.proposal_id}
                className={`jiaoban-run ${isSelected ? "on" : ""}`}
                type="button"
                title={`${entry.state_note} · ${entry.goal_text || "（没写目标）"}`}
                onClick={() => (isCurrent ? onBackToCurrent() : onSelectEntry(entry))}
              >
                <span className={`jiaoban-run-dot ${visual.toneClass}`} aria-hidden="true">
                  {visual.dot}
                </span>
                <span className="jiaoban-run-goal">{entry.goal_text || "（没写目标）"}</span>
                <span className="jiaoban-run-time">{formatHistoryTime(entry.created_at_ms)}</span>
              </button>
            );
          })
        )}
      </div>
    </aside>
  );
}

// 选中「非当前」历史单 → 主区显只读详情卡（不回放完整五态脸·诚实显读模型有的）。
export function JiaobanHistoryDetail({ entry, onBackToCurrent }: { entry: RunHistoryEntry; onBackToCurrent: () => void }) {
  const visual = historyStateVisual(entry);
  const flags = entry.review_flags;
  const opinions = [
    flags.result_verdict ? `结果复核：${humanizeVerdict(flags.result_verdict)}` : null,
    flags.boundary_verdict ? `批前边界：${humanizeVerdict(flags.boundary_verdict)}` : null,
  ].filter(Boolean);
  return (
    <div className="project-canvas-detail-card jiaoban-history-detail" aria-label="历史单详情">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">工作历史 · 这一单</p>
          <h3>
            <span className={`jiaoban-run-dot ${visual.toneClass}`} aria-hidden="true">
              {visual.dot}
            </span>{" "}
            {visual.word}
          </h3>
        </div>
        <button className="secondary-button" type="button" onClick={onBackToCurrent}>
          回到当前
        </button>
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
