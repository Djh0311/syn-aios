import { SecretaryBrief } from "./SecretaryBrief";
import type { SecretaryContext, SecretaryPendingBoardEntry } from "../lib/secretaryReadModel";
import type {
  SecretaryHomeReadModel,
  SecretaryTypedDeepLinkDescriptor,
} from "../lib/types/m4Secretary";
import type { ViewKey } from "../lib/workbenchNavigation";

// Secondary full-page surface for the same M4 context shown on Home.  It does
// not recreate a conversation, source fact or coordination state; actions
// remain limited to the Home continuity spine.
export function SecretaryBoardView({
  home,
  context,
  presentationState = null,
  onOpenDeepLink,
  onReloadSecretaryHome,
}: {
  home?: SecretaryHomeReadModel;
  context?: SecretaryContext;
  presentationState?: "loading" | "error" | null;
  onOpenDeepLink?: (descriptor: SecretaryTypedDeepLinkDescriptor) => void;
  onReloadSecretaryHome?: () => void;
  onNavigate?: (view: ViewKey) => void;
}) {
  if (!home && context) return <LegacySecretaryBoardView context={context} />;
  if (!home) return <section className="secretary-board" aria-label="持续 Secretary 情境">正在恢复 Secretary 情境。</section>;
  const openDeepLink = onOpenDeepLink ?? (() => undefined);
  const reload = onReloadSecretaryHome ?? (() => undefined);
  const state = presentationState ?? home.state;
  return (
    <section className="secretary-board" data-secretary-board-state={state} aria-label="持续 Secretary 情境">
      <header className="secretary-board-head">
        <div>
          <p className="eyebrow">持续 Secretary</p>
          <h2>回到同一情境</h2>
          <p className="muted small-note">这里延续后端恢复的 RoleSession/context；协调动作仍只在首页的来源关注项上执行。</p>
        </div>
      </header>

      <div className="secretary-board-layout">
        <section className="secretary-board-attention" aria-labelledby="secretary-board-attention-title">
          <div className="secretary-section-head">
            <div>
              <p className="eyebrow">source-backed attention</p>
              <h3 id="secretary-board-attention-title">关注回源</h3>
            </div>
            <span className="secretary-section-count">{home.attention_items.length}</span>
          </div>
          {home.attention_items.length ? (
            <div className="secretary-board-attention-list">
              {home.attention_items.map((item) => (
                <article className="secretary-board-attention-row" key={item.item_ref}>
                  <div>
                    <strong>{item.priority_reason_code} · {item.why_code}</strong>
                    <span>当前 <code>{item.status_code}</code> · 来源 <code>{item.source_status_code}</code></span>
                    <span>Owner <code>{item.source_owner.source_owner_ref ?? "UNAVAILABLE"}</code></span>
                    <span>最后变化 {displayUtc(item.last_change_at_utc)} · 到期 {displayUtc(item.due_at_utc)}</span>
                  </div>
                  <button
                    className="secondary-button secretary-board-go"
                    type="button"
                    onClick={() => openDeepLink(item.deep_link)}
                    aria-label="在来源模块中查看此关注项"
                  >
                    回到来源
                  </button>
                </article>
              ))}
            </div>
          ) : (
            <p className="secretary-inline-empty">当前没有来源型关注项。</p>
          )}
          <p className="muted small-note secretary-board-foot">不在本面完成 owner 的业务，也不创建或复制 Todo。</p>
        </section>

        <aside className="secretary-board-side" aria-label="情境摘要与专业入口">
          <SecretaryBrief
            home={home}
            presentationState={presentationState}
            onOpenDeepLink={openDeepLink}
            onReload={reload}
          />
          <section className="secretary-board-modules" aria-label="专业模块入口">
            <p className="eyebrow">专业模块入口</p>
            {home.module_entries.length ? (
              <ul>
                {home.module_entries.map((entry) => (
                  <li key={entry.entry_ref}>
                    <code>{entry.source_owner.source_owner_ref ?? "UNAVAILABLE"}</code>
                    <button className="secondary-button" type="button" onClick={() => openDeepLink(entry.deep_link)}>
                      打开模块
                    </button>
                  </li>
                ))}
              </ul>
            ) : (
              <p className="muted small-note">没有可打开的来源模块入口。</p>
            )}
          </section>
        </aside>
      </div>
    </section>
  );
}

// Compatibility-only pre-M4 board for old static callers. The active App
// intercepts secretary_board and always renders the typed M4 branch above.
function LegacySecretaryBoardView({ context }: { context: SecretaryContext }) {
  const board = context.pending_board;
  const memoryEntries = board.memory_candidate_entry ? [board.memory_candidate_entry] : [];
  return (
    <section className="secretary-board" aria-label="秘书看板">
      <div className="secretary-board-head"><div><p className="eyebrow">秘书</p><h2>待你拍板的都在这</h2></div></div>
      <div className="secretary-board-columns">
        <LegacyBoardColumn title="等你拍板" entries={board.pending_proposals} actionLabel="去交办批" />
        <LegacyBoardColumn title="主管提醒" entries={board.supervisor_reminders} actionLabel="去交办看" />
        <LegacyBoardColumn title="记忆候选" entries={memoryEntries} actionLabel="去记忆中心" />
        <div className="secretary-board-column" aria-label="风险与建议">
          <div className="secretary-board-column-head"><strong>风险与建议</strong><span>{context.risk_signals.length + context.suggestions.length}</span></div>
          <div className="secretary-board-column-body">
            {context.risk_signals.length === 0 && context.suggestions.length === 0 ? <p className="muted small-note secretary-board-empty">这列干净。</p> : (
              <>
                {context.risk_signals.map((risk) => <div className="secretary-board-card" key={risk.risk_id}><span className="secretary-board-card-title">{risk.title}</span>{risk.summary ? <span className="muted">{risk.summary}</span> : null}</div>)}
                {context.suggestions.map((suggestion) => <div className="secretary-board-card" key={suggestion.suggestion_id}><span className="secretary-board-card-title">建议 · {suggestion.title}</span>{suggestion.summary ? <span className="muted">{suggestion.summary}</span> : null}</div>)}
              </>
            )}
          </div>
        </div>
      </div>
      <p className="muted small-note secretary-board-foot">秘书零写入；这里是提醒，不是命令，也不直接执行。</p>
    </section>
  );
}

function LegacyBoardColumn({ title, entries, actionLabel }: { title: string; entries: SecretaryPendingBoardEntry[]; actionLabel: string }) {
  return (
    <div className="secretary-board-column" aria-label={title}>
      <div className="secretary-board-column-head"><strong>{title}</strong><span>{entries.length}</span></div>
      <div className="secretary-board-column-body">
        {entries.length === 0 ? <p className="muted small-note secretary-board-empty">这列干净。</p> : entries.map((entry) => (
          <div className="secretary-board-card" key={entry.entry_id}>
            <span className="secretary-board-card-title">{entry.title}</span>
            {entry.detail ? <span className="muted">{entry.detail}</span> : null}
            <div className="secretary-board-card-foot"><span className="secretary-board-where">{entry.where_hint}</span><button className="secondary-button secretary-board-go" type="button">{actionLabel}</button></div>
          </div>
        ))}
      </div>
    </div>
  );
}

function displayUtc(value: string | null): string {
  if (!value) return "未设置";
  return value.replace("T", " ").replace("Z", " UTC");
}
