import type { ViewKey } from "../lib/workbenchNavigation";
import type { SecretaryContext, SecretaryPendingBoardEntry } from "../lib/secretaryReadModel";
import { SecretaryExplainSection } from "./SecretaryBrief";

// Part②·秘书看板（新视图·吃现成 secretaryContext·纯只读呈现·不新造数据源）。
// 卡片不许拖 = 秘书零写入；深链不做（等 M1 导航方案）——卡上文字指路 + [去处理] 切对应视图即可。
// 长相基准：v1 原型 secFull（三列）升级为四列泳道。SSR/离线：本组件无 hooks（纯呈现）；
// [让 AI 解释现状] 复用 SecretaryBrief 里 memo 包裹的 SecretaryExplainSection（离线 harness 当叶子跳过）。
// 布局：看板撑满 stage 高（stage overflow:hidden 不滚），每列卡片区（column-body）独立滚动——列表不超主界面容器。
export function SecretaryBoardView({
  context,
  onNavigate,
}: {
  context: SecretaryContext;
  onNavigate: (view: ViewKey) => void;
}) {
  const board = context.pending_board;
  const memoryEntries = board.memory_candidate_entry ? [board.memory_candidate_entry] : [];
  const risks = context.risk_signals;
  const suggestions = context.suggestions;
  return (
    <section className="secretary-board" aria-label="秘书看板">
      <div className="secretary-board-head">
        <div>
          <p className="eyebrow">秘书</p>
          <h2>待你拍板的都在这</h2>
          <p className="muted small-note">秘书整理和解释，不判断、不裁决、不派活——每件事都等你自己拍板。</p>
        </div>
        <div className="secretary-board-explain">
          <SecretaryExplainSection />
        </div>
      </div>

      <div className="secretary-board-columns">
        <SecretaryBoardColumn
          title="等你拍板"
          entries={board.pending_proposals}
          actionLabel="去交办批"
          onAction={() => onNavigate("projects")}
        />
        <SecretaryBoardColumn
          title="主管提醒"
          entries={board.supervisor_reminders}
          actionLabel="去交办看"
          onAction={() => onNavigate("projects")}
        />
        <SecretaryBoardColumn
          title="记忆候选"
          entries={memoryEntries}
          actionLabel="去记忆中心"
          onAction={() => onNavigate("memory")}
        />
        <div className="secretary-board-column" aria-label="风险与建议">
          <div className="secretary-board-column-head">
            <strong>风险与建议</strong>
            <span>{risks.length + suggestions.length}</span>
          </div>
          <div className="secretary-board-column-body">
            {risks.length === 0 && suggestions.length === 0 ? (
              <p className="muted small-note secretary-board-empty">这列干净。</p>
            ) : (
              <>
                {risks.map((risk) => (
                  <div className="secretary-board-card" key={risk.risk_id}>
                    <span className="secretary-board-card-title">
                      {riskSeverityLabel(risk.severity)} · {risk.title}
                    </span>
                    {risk.summary ? <span className="muted">{risk.summary}</span> : null}
                  </div>
                ))}
                {suggestions.map((suggestion) => (
                  <div className="secretary-board-card" key={suggestion.suggestion_id}>
                    <span className="secretary-board-card-title">建议 · {suggestion.title}</span>
                    {suggestion.summary ? <span className="muted">{suggestion.summary}</span> : null}
                  </div>
                ))}
              </>
            )}
          </div>
        </div>
      </div>

      <p className="muted small-note secretary-board-foot">秘书零写入；这里是提醒，不是命令，也不直接执行。</p>
    </section>
  );
}

function SecretaryBoardColumn({
  title,
  entries,
  actionLabel,
  onAction,
}: {
  title: string;
  entries: SecretaryPendingBoardEntry[];
  actionLabel: string;
  onAction: () => void;
}) {
  return (
    <div className="secretary-board-column" aria-label={title}>
      <div className="secretary-board-column-head">
        <strong>{title}</strong>
        <span>{entries.length}</span>
      </div>
      <div className="secretary-board-column-body">
        {entries.length === 0 ? (
          <p className="muted small-note secretary-board-empty">这列干净。</p>
        ) : (
          entries.map((entry) => (
            <div className="secretary-board-card" key={entry.entry_id}>
              <span className="secretary-board-card-title">{entry.title}</span>
              {entry.detail ? <span className="muted">{entry.detail}</span> : null}
              <div className="secretary-board-card-foot">
                <span className="secretary-board-where">{entry.where_hint}</span>
                <button className="secondary-button secretary-board-go" type="button" onClick={onAction}>
                  {actionLabel}
                </button>
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
}

function riskSeverityLabel(severity: string): string {
  if (severity === "high") return "高风险";
  if (severity === "medium") return "中风险";
  return "低风险";
}
