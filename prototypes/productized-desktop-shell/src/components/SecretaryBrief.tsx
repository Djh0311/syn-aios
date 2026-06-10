import type { SecretaryContext, SecretaryRiskSignal } from "../lib/secretaryReadModel";

export function SecretaryBrief({ context }: { context: SecretaryContext }) {
  const topRisks = context.risk_signals.slice(0, 3);
  const topSuggestions = context.suggestions.slice(0, 3);
  const pendingCount =
    context.global_summary.pending_permission_count +
    context.global_summary.pending_blackboard_candidate_count +
    context.global_summary.pending_memory_candidate_count;

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
      <SecretaryList title="风险" items={topRisks.map((risk) => `${riskTone(risk)} ${risk.title}`)} emptyText="暂无高信号风险" />
      <SecretaryList title="建议" items={topSuggestions.map((suggestion) => suggestion.title)} emptyText="暂无需要确认的建议" />
      <p className="muted small-note">来源：快照 / 工作流状态 / 候选辅助状态文件 / 适配器描述；秘书模型只读。</p>
    </section>
  );
}

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
