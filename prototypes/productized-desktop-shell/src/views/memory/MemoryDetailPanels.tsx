import { DetailLine } from "../../components/WorkbenchPrimitives";
import type { FormalMemoryListItem, MemoryCandidateListItem } from "../../lib/memoryCenter";
import type { FormalMemoryLifecycleOperationKind } from "../../lib/types";

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

  return (
    <div className="memory-detail-section">
      <h4>正式记忆详情</h4>
      <div className="workflow-draft-grid">
        <DetailLine label="来源" value={sourceText(item.source_summaries)} />
        <DetailLine label="版本摘要" value={item.version_summary} />
        <DetailLine label="审计摘要" value={item.audit_summary} />
        <DetailLine label="冲突 / 检查" value={item.conflict_summary} />
        <DetailLine label="权限 / 外发" value={`${item.permission_summary} / ${item.model_export_summary}`} />
        <DetailLine label="任务包入选状态" value={`${item.task_eligibility.label}：${item.task_eligibility.reason}`} />
      </div>
      {item.conflicts.finding_summaries.slice(0, 3).map((finding) => (
        <p className="state-warning" key={finding}>{finding}</p>
      ))}
      <div className="memory-lifecycle-actions" aria-label="正式记忆生命周期操作">
        <div className="memory-lifecycle-copy">
          <strong>生命周期</strong>
          <span>编辑会创建新版本，不覆盖旧版本；废弃不是移除实体；冻结后不能普通编辑。</span>
        </div>
        <div className="memory-lifecycle-button-row">
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
        <p className="muted small-note">非启用态记忆默认不进任务包；合并和拆分只使用明确可见的当前记录草稿。</p>
        {error ? <p className="state-warning">{error}</p> : null}
      </div>
    </div>
  );
}

export function CandidateMemoryDetail({ item }: { item: MemoryCandidateListItem }) {
  return (
    <div className="memory-detail-section">
      <h4>候选详情</h4>
      <div className="workflow-draft-grid">
        <DetailLine label="候选状态" value={item.status_label} />
        <DetailLine label="确认要求" value={item.confirmation_summary} />
        <DetailLine label="采纳回链" value={item.adoption_summary} />
        <DetailLine label="任务包位置" value={`${item.task_position.label}：${item.task_position.reason}`} />
        <DetailLine label="候选边界" value={item.formal_memory_boundary} />
      </div>
    </div>
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
