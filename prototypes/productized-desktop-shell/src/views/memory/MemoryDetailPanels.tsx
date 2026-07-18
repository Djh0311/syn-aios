import type { FormalMemoryListItem, MemoryCandidateListItem } from "../../lib/memoryCenter";
import type { FormalMemoryLifecycleOperationKind, MemoryCandidate, MemoryLintFinding } from "../../lib/types";

export function FormalMemoryDetail({
  item,
  busyKind,
  error,
  onLifecycleAction,
}: {
  item: FormalMemoryListItem;
  busyKind: FormalMemoryLifecycleOperationKind | null;
  error: string | null;
  onLifecycleAction: (operationKind: FormalMemoryLifecycleOperationKind) => void;
}) {
  const versionOperations: FormalMemoryLifecycleOperationKind[] = ["freeze", "unfreeze"];
  const secretarySuggestionOperations: FormalMemoryLifecycleOperationKind[] = ["promote_to_global", "merge", "archive"];
  const moreOperations: FormalMemoryLifecycleOperationKind[] = ["deprecate", "split", "demote_to_project"];
  const timeline = [
    {
      at: item.record.created_at,
      key: `created:${item.memory_id}:${item.record.created_at}`,
      text: "正式记忆建立",
    },
    ...item.source_summaries.map((source) => ({
      at: source.captured_at,
      key: `source:${source.label}:${source.captured_at}`,
      text: `来源记录：${source.label}`,
    })),
    ...item.versions.map((version) => ({
      at: version.created_at,
      key: `version:${version.version_label}:${version.created_at}`,
      text: `${version.version_label} · ${version.change_summary}（${version.changed_by}）`,
    })),
    ...item.audits.map((audit) => ({
      at: audit.created_at,
      key: `audit:${audit.event_label}:${audit.created_at}`,
      text: `${audit.event_label} · ${audit.status_label}：${audit.reason}`,
    })),
  ].sort((left, right) => left.at.localeCompare(right.at));

  return (
    <article className="memory-detail-card fcard memory-detail-section" data-memory-detail-kind="formal">
      <p className="memory-detail-kicker">正式 · {item.scope_label} · {item.status_label}</p>
      <p className="mem-body">{item.claim}</p>
      {item.body ? <p className="memory-detail-body">{item.body}</p> : null}

      <section className="memory-provenance" aria-label="来龙去脉">
        <h2>来龙去脉</h2>
        {timeline.length ? (
          <ol className="memory-timeline">
            {timeline.map((entry) => (
              <li key={entry.key}>
                <time dateTime={entry.at}>{shortDate(entry.at)}</time>
                <span>{entry.text}</span>
              </li>
            ))}
          </ol>
        ) : (
          <p className="muted small-note">尚无可展示的来源、版本或审计记录。</p>
        )}
      </section>

      {item.conflicts.finding_summaries.map((finding) => (
        <p className="state-warning" key={finding}>{finding}</p>
      ))}

      <div className="memory-lifecycle-actions" aria-label="正式记忆生命周期操作">
        <div className="memory-lifecycle-copy">
          <strong>生命周期</strong>
          <span>编辑会创建新版本，不覆盖旧版本；废弃不是移除实体；冻结后不能普通编辑。</span>
        </div>
        <div className="memory-lifecycle-button-row lifecycle">
          <LifecycleActionButton
            busyKind={busyKind}
            operationKind="revise"
            onLifecycleAction={onLifecycleAction}
          />
          <details className="memory-lifecycle-menu">
            <summary className="secondary-button memory-version-button">版本</summary>
            <div className="memory-lifecycle-menu-body">
              {versionOperations.map((operationKind) => (
                <LifecycleActionButton
                  busyKind={busyKind}
                  key={operationKind}
                  operationKind={operationKind}
                  onLifecycleAction={onLifecycleAction}
                />
              ))}
            </div>
          </details>
          <details className="memory-lifecycle-menu">
            <summary className="memory-secretary-chip">秘书建议</summary>
            <div className="memory-lifecycle-menu-body">
              {secretarySuggestionOperations.map((operationKind) => (
                <LifecycleActionButton
                  busyKind={busyKind}
                  className="secondary-button memory-suggestion-button"
                  key={operationKind}
                  operationKind={operationKind}
                  onLifecycleAction={onLifecycleAction}
                />
              ))}
            </div>
          </details>
          <details className="memory-lifecycle-menu">
            <summary className="secondary-button memory-more-button">更多</summary>
            <div className="memory-lifecycle-menu-body">
              {moreOperations.map((operationKind) => (
                <LifecycleActionButton
                  busyKind={busyKind}
                  key={operationKind}
                  operationKind={operationKind}
                  onLifecycleAction={onLifecycleAction}
                />
              ))}
            </div>
          </details>
        </div>
        <p className="muted small-note">{item.task_eligibility.label}：{item.task_eligibility.reason}</p>
        {error ? <p className="state-warning">{error}</p> : null}
      </div>
    </article>
  );
}

export function CandidateMemoryDetail({
  item,
  canConfirm = false,
  canAdopt = false,
  canDiscard = false,
  canReject = false,
  onConfirm,
  onAdopt,
  onDiscard,
  onReject,
  hasOpenLintFinding = false,
  sourceRefs,
}: {
  item: MemoryCandidateListItem;
  canConfirm?: boolean;
  canAdopt?: boolean;
  canDiscard?: boolean;
  canReject?: boolean;
  onConfirm?: () => void;
  onAdopt?: () => void;
  onDiscard?: () => void;
  onReject?: () => void;
  hasOpenLintFinding?: boolean;
  sourceRefs?: MemoryCandidate["source_refs"];
}) {
  return (
    <article className="memory-detail-card fcard memory-detail-section" data-memory-detail-kind="candidate">
      <p className="memory-detail-kicker">候选 · {item.status_label} · {item.scope_label}</p>
      <p className="mem-body">{item.claim}</p>
      {item.body ? <p className="memory-detail-body">{item.body}</p> : null}
      <p className="memory-kv"><strong>哪来的</strong>{candidateSourceText(sourceRefs, item.source_summaries)}</p>
      {hasOpenLintFinding ? <p className="memory-kv"><strong>和现有记忆</strong>{item.lint_summary}</p> : null}
      <p className="memory-kv"><strong>候选边界</strong>{item.formal_memory_boundary}</p>
      <div className="knowledge-action-row lifecycle" aria-label="候选记忆操作">
        {canConfirm && onConfirm ? (
          <button className="secondary-button" type="button" onClick={onConfirm}>
            属实（确认）
          </button>
        ) : null}
        {canAdopt && onAdopt ? (
          <button className="secondary-button" type="button" onClick={onAdopt}>
            记住（转正式）
          </button>
        ) : null}
        {canDiscard && onDiscard ? (
          <button className="secondary-button" type="button" onClick={onDiscard}>
            {canReject ? "暂不处理" : "不要"}
          </button>
        ) : null}
        {canReject && onReject ? (
          <button className="secondary-button" type="button" onClick={onReject}>
            不要
          </button>
        ) : null}
      </div>
      {!canConfirm && !canAdopt && !canDiscard && !canReject ? (
        <p className="muted small-note">当前候选状态没有可执行的既有决定动作。</p>
      ) : null}
    </article>
  );
}

export function MemoryLintFindingDetail({
  finding,
  targetMemory,
  busyKind,
  error,
  onLifecycleAction,
}: {
  finding: MemoryLintFinding;
  targetMemory: FormalMemoryListItem | null;
  busyKind: FormalMemoryLifecycleOperationKind | null;
  error: string | null;
  onLifecycleAction: (operationKind: FormalMemoryLifecycleOperationKind) => void;
}) {
  const evidence = finding.evidence_refs
    .map((source) => source.source_title || source.source_id || source.source_type)
    .filter(Boolean)
    .join("；");

  return (
    <article className="memory-detail-card fcard memory-detail-section" data-memory-detail-kind="lint">
      <p className="memory-detail-kicker">维护检查发现 · {finding.severity === "blocking" ? "阻断级" : "需要复核"}</p>
      <p className="mem-body">{finding.summary}</p>
      <p className="memory-kv"><strong>证据</strong>{evidence || "检查未提供可展示的证据来源。"}</p>
      {targetMemory ? (
        <div className="knowledge-action-row lifecycle" aria-label="维护发现操作">
          <button
            className="secondary-button"
            type="button"
            disabled={busyKind !== null}
            onClick={() => onLifecycleAction("revise")}
          >
            {busyKind === "revise" ? "预览中" : "改写提案"}
          </button>
          <button
            className="secondary-button"
            type="button"
            disabled={busyKind !== null}
            onClick={() => onLifecycleAction("deprecate")}
          >
            {busyKind === "deprecate" ? "预览中" : "废弃"}
          </button>
        </div>
      ) : null}
      {!targetMemory ? <p className="muted small-note">此发现没有可由 M9 生命周期命令处理的正式记忆目标。</p> : null}
      {error ? <p className="state-warning">{error}</p> : null}
    </article>
  );
}

function LifecycleActionButton({
  busyKind,
  className = "secondary-button",
  label,
  operationKind,
  onLifecycleAction,
}: {
  busyKind: FormalMemoryLifecycleOperationKind | null;
  className?: string;
  label?: string;
  operationKind: FormalMemoryLifecycleOperationKind;
  onLifecycleAction: (operationKind: FormalMemoryLifecycleOperationKind) => void;
}) {
  return (
    <button
      className={className}
      disabled={busyKind !== null}
      onClick={() => onLifecycleAction(operationKind)}
      type="button"
    >
      {busyKind === operationKind ? "预览中" : label ?? operationLabel(operationKind)}
    </button>
  );
}

export function operationLabel(operationKind: FormalMemoryLifecycleOperationKind): string {
  const labels: Record<FormalMemoryLifecycleOperationKind, string> = {
    revise: "编辑提案",
    deprecate: "废弃",
    freeze: "冻结",
    unfreeze: "解冻",
    archive: "归档",
    merge: "合并",
    split: "拆分",
    promote_to_global: "上升为全局",
    demote_to_project: "下沉为项目",
  };
  return labels[operationKind];
}

export function sourceText(sources: FormalMemoryListItem["source_summaries"] | MemoryCandidateListItem["source_summaries"]) {
  return sources.map((source) => `${source.label} / ${source.authority_label} / ${source.sensitive_label}`).join("；") || "来源未记录";
}

function candidateSourceText(
  sourceRefs: MemoryCandidate["source_refs"] | undefined,
  sourceSummaries: MemoryCandidateListItem["source_summaries"],
): string {
  if (!sourceRefs?.length) return sourceText(sourceSummaries);
  return sourceRefs
    .map((source) => {
      const label = source.source_title || source.source_id || source.source_ref_id;
      const anchor = source.anchor ? ` · 锚点 ${source.anchor}` : "";
      return `${label} · ${source.source_type} · 引用 ${source.source_ref_id}${anchor}`;
    })
    .join("；");
}

function shortDate(value: string): string {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;
  return `${date.getUTCFullYear()}-${String(date.getUTCMonth() + 1).padStart(2, "0")}-${String(date.getUTCDate()).padStart(2, "0")}`;
}
