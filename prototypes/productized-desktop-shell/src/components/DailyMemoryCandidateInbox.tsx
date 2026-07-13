import { Badge } from "./Badge";
import {
  buildAdoptMemoryCandidateAction,
  buildBatchAdoptMemoryCandidatesAction,
  buildDailyMemoryCandidateDecisionAction,
  type DailyMemoryCandidateInbox as DailyMemoryCandidateInboxModel,
} from "../lib/memoryDailyLoop";
import type { PendingAction } from "../lib/types";

export function DailyMemoryCandidateInbox({
  inbox,
  projectRoot,
  candidateStoreRevision,
  formalStoreRevision,
  onRequestAction,
}: {
  inbox: DailyMemoryCandidateInboxModel;
  projectRoot: string;
  candidateStoreRevision?: number | null;
  formalStoreRevision?: number | null;
  onRequestAction?: (action: PendingAction) => void;
}) {
  const adoptableCandidates = inbox.items.filter((item) => item.can_adopt).map((item) => item.candidate);
  return (
    <section className="panel running-section daily-memory-candidate-inbox" aria-label="日常记忆候选收件箱">
      <div className="panel-h">
        日常记忆候选收件箱
        <Badge tone={inbox.pending_count ? "warning" : "neutral"}>{inbox.pending_count} 条</Badge>
      </div>
      <p className="muted small-note">
        {inbox.pending_count} 条记忆候选待确认；{inbox.boundary_text}
      </p>
      <div className="workflow-state-actions">
        <button
          className="secondary-button"
          type="button"
          disabled={!onRequestAction || !adoptableCandidates.length}
          onClick={() =>
            onRequestAction?.(
              buildBatchAdoptMemoryCandidatesAction({
                candidates: adoptableCandidates,
                projectRoot,
                candidateStoreRevision,
                formalStoreRevision,
              }),
            )
          }
        >
          批量采纳 {adoptableCandidates.length} 条候选
        </button>
        <span className="running-action-note">批量采纳仍逐条走 M2 采纳门。</span>
      </div>
      <div className="running-workflow-list">
        {inbox.items.length ? (
          inbox.items.slice(0, 4).map((item) => (
            <article className="running-attention-card memory-daily-inbox-card" key={item.candidate_key}>
              <strong>{item.claim}</strong>
              <span>{item.status_label} · {item.risk_label}</span>
              <em>来源：{item.source_label}；候选不是正式记忆，采纳前必须确认。</em>
              {item.can_confirm ? (
                <button
                  className="secondary-button"
                  type="button"
                  disabled={!onRequestAction}
                  onClick={() =>
                    onRequestAction?.(
                      buildDailyMemoryCandidateDecisionAction({
                        candidate: item.candidate,
                        projectRoot,
                        requestedStatus: "candidate_confirmed",
                        reason: `日常记忆候选收件箱确认候选属实：${item.claim}；仍不写正式记忆。`,
                        candidateStoreRevision,
                      }),
                    )
                  }
                >
                  确认候选属实
                </button>
              ) : null}
              <button
                className="secondary-button"
                type="button"
                disabled={!onRequestAction || !item.can_adopt}
                onClick={() =>
                  onRequestAction?.(
                    buildAdoptMemoryCandidateAction({
                      candidate: item.candidate,
                      projectRoot,
                      candidateStoreRevision,
                      formalStoreRevision,
                    }),
                  )
                }
              >
                {item.can_adopt ? `采纳候选：${item.claim}` : "先审查候选状态"}
              </button>
              <button
                className="secondary-button"
                type="button"
                disabled={!onRequestAction || !item.can_defer}
                onClick={() =>
                  onRequestAction?.(
                    buildDailyMemoryCandidateDecisionAction({
                      candidate: item.candidate,
                      projectRoot,
                      requestedStatus: "candidate_discarded",
                      reason: `日常记忆候选收件箱暂不处理并移出待办：${item.claim}；不写正式记忆。`,
                      candidateStoreRevision,
                    }),
                  )
                }
              >
                暂不处理
              </button>
              <button
                className="secondary-button"
                type="button"
                disabled={!onRequestAction || !item.can_reject}
                onClick={() =>
                  onRequestAction?.(
                    buildDailyMemoryCandidateDecisionAction({
                      candidate: item.candidate,
                      projectRoot,
                      requestedStatus: "candidate_rejected",
                      reason: `日常记忆候选收件箱拒绝候选：${item.claim}；不写正式记忆。`,
                      candidateStoreRevision,
                    }),
                  )
                }
              >
                拒绝候选
              </button>
            </article>
          ))
        ) : (
          <p className="empty-line">当前没有日常待确认记忆候选。</p>
        )}
      </div>
    </section>
  );
}
