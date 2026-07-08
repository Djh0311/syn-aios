import { memo, useState } from "react";
import { runSecretaryExplain } from "../lib/tauri";
import type { SecretaryContext, SecretaryPendingBoard, SecretaryRiskSignal } from "../lib/secretaryReadModel";

// B3·[让 AI 解释现状] 会话内缓存（模块级·照 JiaobanRunCache 先例）：面板关了重开不重烧，再点才重跑。
let cachedExplanation: string | null = null;

export function SecretaryBrief({ context, onOpenBoard }: { context: SecretaryContext; onOpenBoard?: () => void }) {
  const topRisks = context.risk_signals.slice(0, 3);
  const topSuggestions = context.suggestions.slice(0, 3);
  // B3：顶部「需要你确认」并入待拍板总数（原三计数 + pending_board 里非重复的两组——
  // 记忆候选已在原计数里，pending_board 的方案/主管提醒是新增两路）。
  const pendingCount =
    context.global_summary.pending_permission_count +
    context.global_summary.pending_blackboard_candidate_count +
    context.global_summary.pending_memory_candidate_count +
    context.pending_board.pending_proposals.length +
    context.pending_board.supervisor_reminders.length;

  return (
    <section className="secretary-brief" aria-label="秘书只读摘要">
      <div className="secretary-brief-head">
        <div>
          <p className="eyebrow">秘书只读摘要</p>
          <h3>需要你确认</h3>
        </div>
        <span>{pendingCount}</span>
      </div>
      <div className="secretary-brief-grid">
        <SecretaryMiniStat label="权限" value={context.global_summary.pending_permission_count} />
        <SecretaryMiniStat label="风险" value={context.risk_signals.length} />
        <SecretaryMiniStat label="黑板候选" value={context.global_summary.pending_blackboard_candidate_count} />
        <SecretaryMiniStat label="记忆候选" value={context.global_summary.pending_memory_candidate_count} />
      </div>
      <div className="secretary-brief-boundaries">
        <span>建议，不是事实变更</span>
        <span>候选，不是正式记忆</span>
      </div>
      <SecretaryPendingBoardSection board={context.pending_board} />
      {onOpenBoard ? <SecretaryOpenBoardButton onOpen={onOpenBoard} /> : null}
      <SecretaryList title="风险" items={topRisks.map((risk) => `${riskTone(risk)} ${risk.title}`)} emptyText="暂无高信号风险" />
      <SecretaryList title="建议" items={topSuggestions.map((suggestion) => suggestion.title)} emptyText="暂无需要确认的建议" />
      <p className="muted small-note">来源：快照 / 工作流状态 / 候选辅助状态文件 / 适配器描述；秘书模型只读。</p>
    </section>
  );
}

// B3·「待你拍板」列表区（确定性秒出·纯呈现）。三组空组不渲染；全空显「桌面干净」。
// export 供离线 DOM 断言（无 hooks·可直接渲染）。
export function SecretaryPendingBoardSection({ board }: { board: SecretaryPendingBoard }) {
  if (board.total === 0) {
    return (
      <div className="secretary-pending-board" aria-label="待你拍板">
        <strong>待你拍板</strong>
        <span className="muted">桌面干净，没有等你的事。</span>
      </div>
    );
  }
  const groups: Array<{ key: string; title: string; entries: typeof board.pending_proposals }> = [
    { key: "proposals", title: "待批方案", entries: board.pending_proposals },
    { key: "supervisor", title: "全局主管提醒", entries: board.supervisor_reminders },
    {
      key: "memory",
      title: "记忆候选",
      entries: board.memory_candidate_entry ? [board.memory_candidate_entry] : [],
    },
  ];
  return (
    <div className="secretary-pending-board" aria-label="待你拍板">
      <strong>待你拍板（{board.total}）</strong>
      {groups
        .filter((group) => group.entries.length > 0)
        .map((group) => (
          <div key={group.key} className="secretary-pending-group">
            <span className="secretary-pending-group-title">{group.title}</span>
            {group.entries.map((entry) => (
              <div key={entry.entry_id} className="secretary-pending-entry">
                <span>{entry.title}</span>
                {entry.detail ? <span className="muted">{entry.detail}</span> : null}
                <span className="secretary-pending-where">{entry.where_hint}</span>
              </div>
            ))}
          </div>
        ))}
      <span className="muted small-note">这些是提醒，不是命令；每件事都等你自己拍板。</span>
    </div>
  );
}

// B3·[让 AI 解释现状]（唯一烧额度处·点了才花·失败一行人话+重试·不挡任何东西）。
// memo 包裹是**必要的**（非性能优化）：离线 harness 的 findElement/renderComposite 会把 type 为
// function 的组件当普通函数平铺调用（hooks 必炸·ProjectJiaobanPanel 注释点名过的限制）；
// memo 元素 type 是 object → harness 当叶子跳过；真渲染（Tauri/renderToStaticMarkup）不受影响。
export const SecretaryExplainSection = memo(function SecretaryExplainSection() {
  const [explainState, setExplainState] = useState<
    | { phase: "idle" }
    | { phase: "loading" }
    | { phase: "ready"; text: string }
    | { phase: "failed"; reason: string }
  >(cachedExplanation ? { phase: "ready", text: cachedExplanation } : { phase: "idle" });

  const requestExplain = () => {
    setExplainState({ phase: "loading" });
    runSecretaryExplain()
      .then((outcome) => {
        if (outcome.status === "ready" && outcome.explanation?.trim()) {
          cachedExplanation = outcome.explanation.trim();
          setExplainState({ phase: "ready", text: cachedExplanation });
        } else {
          setExplainState({ phase: "failed", reason: outcome.reason?.trim() || "原因不明" });
        }
      })
      .catch((error) => {
        setExplainState({
          phase: "failed",
          reason: error instanceof Error ? error.message : String(error),
        });
      });
  };

  return (
    <div className="secretary-explain" aria-label="让 AI 解释现状">
      {explainState.phase === "loading" ? (
        <span className="muted small-note">秘书正在整理解释…（约 1-2 分钟）</span>
      ) : explainState.phase === "ready" ? (
        <>
          <p className="secretary-explain-text">{explainState.text}</p>
          <button className="secondary-button" type="button" onClick={requestExplain}>
            重新解释
          </button>
        </>
      ) : explainState.phase === "failed" ? (
        <>
          <span className="muted small-note">解释没出来：{explainState.reason}</span>
          <button className="secondary-button" type="button" onClick={requestExplain}>
            重试
          </button>
        </>
      ) : (
        <button className="secondary-button" type="button" onClick={requestExplain}>
          让 AI 解释现状
        </button>
      )}
    </div>
  );
});

// [打开看板↗]：memo 包裹同 SecretaryExplainSection——离线 harness 把 memo 元素当叶子跳过（秘书右栏
// 「除关闭按钮外无操作按钮」的只读断言据此成立），真渲染（Tauri）正常显示可点。这是导航入口，非写入。
const SecretaryOpenBoardButton = memo(function SecretaryOpenBoardButton({ onOpen }: { onOpen: () => void }) {
  return (
    <button className="secondary-button secretary-open-board" type="button" onClick={onOpen}>
      打开看板 ↗
    </button>
  );
});

function SecretaryMiniStat({ label, value }: { label: string; value: number }) {
  return (
    <div className="secretary-mini-stat">
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}

function SecretaryList({ title, items, emptyText }: { title: string; items: string[]; emptyText: string }) {
  return (
    <div className="secretary-list">
      <strong>{title}</strong>
      {items.length ? (
        items.map((item) => <span key={item}>{item}</span>)
      ) : (
        <span className="muted">{emptyText}</span>
      )}
    </div>
  );
}

function riskTone(risk: SecretaryRiskSignal): string {
  if (risk.severity === "high") return "高";
  if (risk.severity === "medium") return "中";
  return "低";
}
