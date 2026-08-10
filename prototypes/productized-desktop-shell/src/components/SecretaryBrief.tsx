import { memo, useState } from "react";
import { runSecretaryExplain } from "../lib/tauri";
import type { SecretaryContext, SecretaryPendingBoard, SecretaryPendingBoardEntry, SecretaryRiskSignal } from "../lib/secretaryReadModel";
import type {
  SecretaryHomeReadModel,
  SecretaryTypedDeepLinkDescriptor,
} from "../lib/types/m4Secretary";

type SecretaryBriefProps = {
  home?: SecretaryHomeReadModel;
  // Old workbench surfaces still compile against this compatibility input.
  // App never supplies it to the M4 Secretary path.
  context?: SecretaryContext;
  presentationState?: "loading" | "error" | null;
  onOpenBoard?: () => void;
  onOpenDeepLink?: (descriptor: SecretaryTypedDeepLinkDescriptor) => void;
  onReload?: () => void;
};

// The right rail is a compact continuity readout. It intentionally receives
// the same server-owned M4 projection as Home rather than the old derived
// `SecretaryContext` summary.
export function SecretaryBrief({
  home,
  context,
  presentationState = null,
  onOpenBoard,
  onOpenDeepLink,
  onReload,
}: SecretaryBriefProps) {
  if (!home && context) return <LegacySecretaryBrief context={context} onOpenBoard={onOpenBoard} />;
  if (!home) return <BriefState title="正在恢复 Secretary 情境" detail="等待后端持续情境读模型。" />;
  const state = presentationState ?? home.state;
  if (state === "loading") {
    return <BriefState title="正在恢复 Secretary 情境" detail="正在读取后端 RoleSession、context 与确定性 brief。" />;
  }
  if (state === "error") {
    return (
      <BriefState
        title="秘书情境读取失败"
        detail="没有用旧的前端摘要补造身份或上下文。"
        recoveryCode={home.degradation_code ?? home.role_session_recovery.recovery_code}
        onReload={onReload}
      />
    );
  }
  if (home.state === "degraded") {
    return (
      <BriefState
        title="秘书情境降级"
        detail="当前没有可确认的持续情境；来源事实仍留在其负责模块。"
        recoveryCode={home.degradation_code ?? home.role_session_recovery.recovery_code}
        onReload={onReload}
      />
    );
  }

  return (
    <section className="secretary-brief" aria-label="持续 Secretary 摘要">
      <div className="secretary-brief-head">
        <div>
          <p className="eyebrow">持续 Secretary</p>
          <h3>{home.state === "empty" ? "情境已恢复，当前干净" : "继续同一情境"}</h3>
        </div>
        <span aria-label={`${home.attention_items.length} 条持续关注`}>{home.attention_items.length}</span>
      </div>

      <SecretaryContinuityRefs home={home} />
      <div className="secretary-brief-boundaries">
        <span>确定性 brief</span>
        <span>来源事实留在 owner</span>
        <span>无前端身份缓存</span>
      </div>

      {home.attention_items.length ? (
        <div className="secretary-brief-attention" aria-label="关注来源摘要">
          {home.attention_items.slice(0, 3).map((item) => (
            <div key={item.item_ref} className="secretary-brief-attention-row">
              <span>{item.priority_reason_code} · {item.why_code}</span>
              <code>{item.status_code}</code>
              <code>{item.source_owner.source_owner_ref ?? "owner unavailable"}</code>
              <button
                className="secondary-button secretary-brief-source-link"
                type="button"
                onClick={() => onOpenDeepLink?.(item.deep_link)}
                disabled={!onOpenDeepLink}
                aria-label="在来源模块中查看关注项"
              >
                来源
              </button>
            </div>
          ))}
        </div>
      ) : (
        <p className="muted small-note">没有待持续看住的来源关注项。</p>
      )}

      <SecretaryAvailability home={home} />
      <SecretaryExplainSection home={home} />
      {onOpenBoard ? <SecretaryOpenBoardButton onOpen={onOpenBoard} /> : null}
    </section>
  );
}

// Kept solely for the pre-M4 static surfaces which still pass SecretaryContext.
// It is visibly separate from the server-owned M4 path above and cannot create
// a role/session/context or exercise the new coordination boundary.
function LegacySecretaryBrief({ context, onOpenBoard }: { context: SecretaryContext; onOpenBoard?: () => void }) {
  const topRisks = context.risk_signals.slice(0, 3);
  const topSuggestions = context.suggestions.slice(0, 3);
  const pendingCount =
    context.global_summary.pending_permission_count
    + context.global_summary.pending_blackboard_candidate_count
    + context.global_summary.pending_memory_candidate_count
    + context.pending_board.pending_proposals.length
    + context.pending_board.supervisor_reminders.length;
  return (
    <section className="secretary-brief" aria-label="秘书只读摘要">
      <div className="secretary-brief-head">
        <div><p className="eyebrow">秘书只读摘要</p><h3>需要你确认</h3></div>
        <span>{pendingCount}</span>
      </div>
      <div className="secretary-brief-grid">
        <LegacySecretaryMiniStat label="权限" value={context.global_summary.pending_permission_count} />
        <LegacySecretaryMiniStat label="风险" value={context.risk_signals.length} />
        <LegacySecretaryMiniStat label="黑板候选" value={context.global_summary.pending_blackboard_candidate_count} />
        <LegacySecretaryMiniStat label="记忆候选" value={context.global_summary.pending_memory_candidate_count} />
      </div>
      <div className="secretary-brief-boundaries"><span>建议，不是事实变更</span><span>候选，不是正式记忆</span></div>
      <SecretaryPendingBoardSection board={context.pending_board} />
      {onOpenBoard ? <LegacyOpenBoardButton onOpen={onOpenBoard} /> : null}
      <LegacySecretaryList title="风险" items={topRisks.map((risk) => `${riskTone(risk)} ${risk.title}`)} emptyText="暂无高信号风险" />
      <LegacySecretaryList title="建议" items={topSuggestions.map((suggestion) => suggestion.title)} emptyText="暂无需要确认的建议" />
      <p className="muted small-note">来源：快照 / 工作流状态 / 候选辅助状态文件 / 适配器描述；秘书模型只读。</p>
    </section>
  );
}

// Export remains for the existing offline presentation fixture. The M4C06
// actual path uses the source-backed attention projection above instead.
export function SecretaryPendingBoardSection({ board }: { board: SecretaryPendingBoard }) {
  if (board.total === 0) {
    return <div className="secretary-pending-board" aria-label="待你拍板"><strong>待你拍板</strong><span className="muted">桌面干净，没有等你的事。</span></div>;
  }
  const groups: Array<{ key: string; title: string; entries: SecretaryPendingBoardEntry[] }> = [
    { key: "proposals", title: "待批方案", entries: board.pending_proposals },
    { key: "supervisor", title: "全局主管提醒", entries: board.supervisor_reminders },
    { key: "memory", title: "记忆候选", entries: board.memory_candidate_entry ? [board.memory_candidate_entry] : [] },
  ];
  return (
    <div className="secretary-pending-board" aria-label="待你拍板">
      <strong>待你拍板（{board.total}）</strong>
      {groups.filter((group) => group.entries.length > 0).map((group) => (
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

const LegacyOpenBoardButton = memo(function LegacyOpenBoardButton({ onOpen }: { onOpen: () => void }) {
  return <button className="secondary-button secretary-open-board" type="button" onClick={onOpen}>打开看板 ↗</button>;
});

function LegacySecretaryMiniStat({ label, value }: { label: string; value: number }) {
  return <div className="secretary-mini-stat"><span>{label}</span><strong>{value}</strong></div>;
}

function LegacySecretaryList({ title, items, emptyText }: { title: string; items: string[]; emptyText: string }) {
  return (
    <div className="secretary-list">
      <strong>{title}</strong>
      {items.length ? items.map((item) => <span key={item}>{item}</span>) : <span className="muted">{emptyText}</span>}
    </div>
  );
}

function riskTone(risk: SecretaryRiskSignal): string {
  if (risk.severity === "high") return "高";
  if (risk.severity === "medium") return "中";
  return "低";
}

function BriefState({
  title,
  detail,
  recoveryCode = null,
  onReload,
}: {
  title: string;
  detail: string;
  recoveryCode?: string | null;
  onReload?: () => void;
}) {
  return (
    <section className="secretary-brief secretary-brief-state" aria-live="polite">
      <p className="eyebrow">持续 Secretary</p>
      <h3>{title}</h3>
      <p className="muted small-note">{detail}</p>
      {recoveryCode ? <p className="secretary-recovery-code">恢复码：<code>{recoveryCode}</code></p> : null}
      {onReload ? <button className="secondary-button" type="button" onClick={onReload}>重新读取</button> : null}
    </section>
  );
}

function SecretaryContinuityRefs({ home }: { home: SecretaryHomeReadModel }) {
  const recovery = home.role_session_recovery;
  const brief = home.deterministic_brief;
  return (
    <dl className="secretary-continuity-refs" aria-label="后端持续情境引用">
      <div>
        <dt>RoleSession</dt>
        <dd><code>{recovery.role_session_ref ?? "UNAVAILABLE"}</code></dd>
      </div>
      <div>
        <dt>Context</dt>
        <dd><code>{recovery.context_ref ?? "UNAVAILABLE"}</code></dd>
      </div>
      <div>
        <dt>Brief</dt>
        <dd><code>{brief?.brief_ref ?? "UNAVAILABLE"}</code></dd>
      </div>
    </dl>
  );
}

function SecretaryAvailability({ home }: { home: SecretaryHomeReadModel }) {
  const modelCode = home.model_enhancement.recovery_code;
  const handoffCode = home.handoff.recovery_code;
  return (
    <section className="secretary-brief-availability" aria-label="模型与 Handoff 边界">
      <div><span>模型增强</span><code>{home.model_enhancement.status}</code></div>
      <div><span>Handoff</span><code>{home.handoff.status}</code></div>
      {modelCode ? <p className="muted small-note">模型恢复码：<code>{modelCode}</code></p> : null}
      {handoffCode ? <p className="muted small-note">Handoff 恢复码：<code>{handoffCode}</code></p> : null}
    </section>
  );
}

// This component has local display state only.  It is never a source of
// persistent identity, scope, source facts or conversation history.  The
// invocation occurs solely from the explicit user click below.
export const SecretaryExplainSection = memo(function SecretaryExplainSection({ home }: { home: SecretaryHomeReadModel }) {
  const [explainState, setExplainState] = useState<
    | { phase: "idle" }
    | { phase: "loading" }
    | { phase: "ready"; text: string }
    | { phase: "failed"; reason: string }
  >({ phase: "idle" });
  const unavailable = !canRequestMechanicalExplain(home);
  const unavailableReason = explainUnavailableReason(home);

  const requestExplain = () => {
    if (unavailable) return;
    setExplainState({ phase: "loading" });
    runSecretaryExplain()
      .then((outcome) => {
        if (outcome.status === "ready" && outcome.explanation?.trim()) {
          setExplainState({ phase: "ready", text: outcome.explanation.trim() });
        } else {
          setExplainState({ phase: "failed", reason: safeExplainReason(outcome.reason) });
        }
      })
      .catch((error) => setExplainState({ phase: "failed", reason: safeExplainReason(error) }));
  };

  return (
    <section className="secretary-explain" aria-label="请求秘书解释">
      <p className="eyebrow">机械解释</p>
      {unavailable ? (
        <>
          <p className="muted small-note">当前无法请求解释：<code>{unavailableReason}</code>。</p>
          <button className="secondary-button" type="button" disabled aria-describedby="secretary-explain-unavailable">
            请秘书解释
          </button>
          <span id="secretary-explain-unavailable" className="sr-only">模型或持续情境当前不可用</span>
        </>
      ) : explainState.phase === "loading" ? (
        <p className="muted small-note" aria-live="polite">正在请求机械解释…</p>
      ) : explainState.phase === "ready" ? (
        <>
          <p className="secretary-explain-text">{explainState.text}</p>
          <p className="muted small-note">这是解释，不是来源事实；需要事实请回到来源模块。</p>
          <button className="secondary-button" type="button" onClick={requestExplain}>再次请求解释</button>
        </>
      ) : explainState.phase === "failed" ? (
        <>
          <p className="muted small-note" aria-live="assertive">解释未返回：<code>{explainState.reason}</code>。</p>
          <button className="secondary-button" type="button" onClick={requestExplain}>重试解释</button>
        </>
      ) : (
        <button className="secondary-button" type="button" onClick={requestExplain}>
          请秘书解释
        </button>
      )}
    </section>
  );
});

// Kept memoized because the established offline composite renderer treats
// memo elements as leaves while normal React rendering still supports hooks.
const SecretaryOpenBoardButton = memo(function SecretaryOpenBoardButton({ onOpen }: { onOpen: () => void }) {
  return (
    <button className="secondary-button secretary-open-board" type="button" onClick={onOpen}>
      打开持续 Secretary ↗
    </button>
  );
});

function canRequestMechanicalExplain(home: SecretaryHomeReadModel): boolean {
  if (home.state !== "ready" && home.state !== "empty") return false;
  return home.role_session_recovery.status === "RESTORED";
}

function explainUnavailableReason(home: SecretaryHomeReadModel): string {
  if (home.role_session_recovery.status !== "RESTORED") {
    return home.role_session_recovery.recovery_code ?? "SECRETARY_CONTEXT_UNAVAILABLE";
  }
  return "SECRETARY_HOME_UNAVAILABLE";
}

function safeExplainReason(reason: unknown): string {
  const value = reason instanceof Error ? reason.message : String(reason ?? "SECRETARY_EXPLAIN_FAILED");
  return /^[A-Za-z0-9_:-]{1,128}$/.test(value) ? value : "SECRETARY_EXPLAIN_FAILED";
}
