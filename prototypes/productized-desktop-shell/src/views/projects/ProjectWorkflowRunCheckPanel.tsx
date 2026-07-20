import { memo, useEffect, useState } from "react";
import { Pill } from "../../components/SpecPrimitives";
import type { WorkflowRunCheck } from "../../lib/types";
import { DetailLine, runCheckItemStatusLabel, runCheckStatusLabel, runCheckTone } from "./projectWorkflowLabels";

export const WorkflowRunCheckPanel = memo(function WorkflowRunCheckPanel({
  projectRoot,
  workflowId,
  derivedStatus,
  onInspectWorkflowRunCheck,
}: {
  projectRoot: string;
  workflowId: string;
  derivedStatus: WorkflowRunCheck["status"] | null;
  onInspectWorkflowRunCheck?: (projectRoot: string, workflowId?: string | null) => Promise<WorkflowRunCheck>;
}) {
  const [runCheck, setRunCheck] = useState<WorkflowRunCheck | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    setRunCheck(null);
    setError(null);
  }, [projectRoot, workflowId]);

  async function inspect() {
    if (!onInspectWorkflowRunCheck) {
      setError("当前运行环境没有接入运行前检查入口。");
      return;
    }
    setLoading(true);
    setError(null);
    try {
      setRunCheck(await onInspectWorkflowRunCheck(projectRoot, workflowId));
    } catch (inspectError) {
      setRunCheck(null);
      setError(messageOf(inspectError));
    } finally {
      setLoading(false);
    }
  }

  return (
    <section className="workflow-run-check-panel">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">运行前检查</p>
          <h3>{runCheck ? runCheckStatusLabel(runCheck.status) : "只阻止运行，不阻止查看草稿"}</h3>
        </div>
        <Pill tone={runCheckTone(runCheck?.status ?? derivedStatus)}>
          {runCheck?.status ? runCheckStatusLabel(runCheck.status) : derivedStatus ? runCheckStatusLabel(derivedStatus) : "未检查"}
        </Pill>
      </div>
      <div className="workflow-draft-grid">
        <DetailLine label="当前运行器" value="只读展示；不会自动运行运行器" />
        <DetailLine label="派生状态" value={derivedStatus ? runCheckStatusLabel(derivedStatus) : "未返回"} />
        <DetailLine label="阻塞数量" value={String(runCheck?.blocked_reasons.length ?? 0)} />
        <DetailLine label="警告数量" value={String(runCheck?.warnings.length ?? 0)} />
        <DetailLine label="证据完整度" value={runCheck?.evidence_completeness ?? "未检查"} />
      </div>
      <div className="workflow-state-actions">
        <button className="secondary-button" type="button" onClick={() => void inspect()}>
          检查运行前状态
        </button>
      </div>
      {loading ? <p className="muted small-note">正在读取运行前检查。</p> : null}
      {error ? <p className="state-warning">{error}</p> : null}
      {runCheck ? (
        <WorkflowRunCheckDetails runCheck={runCheck} />
      ) : (
        <p className="muted small-note">缺模型、读写范围、验收标准、权限或决策时会保持阻断。</p>
      )}
    </section>
  );
});

export function WorkflowRunCheckDetails({ runCheck }: { runCheck: WorkflowRunCheck }) {
  return (
    <div className="workflow-run-check-details">
      {runCheck.blocked_reasons.length ? (
        <ul className="state-warning-list">
          {runCheck.blocked_reasons.map((reason) => (
            <li key={reason}>{reason}</li>
          ))}
        </ul>
      ) : null}
      {runCheck.warnings.map((warning) => (
        <p className="state-warning" key={warning}>
          {warning}
        </p>
      ))}
      <div className="run-check-list" aria-label="运行前检查项">
        {runCheck.checks.map((check) => (
          <div className={`run-check-item ${check.status}`} key={`${check.check_id}:${check.source_ref ?? "workflow"}`}>
            <strong>{check.label}</strong>
            <span>{runCheckItemStatusLabel(check.status)}</span>
            <em>{check.reason}</em>
          </div>
        ))}
      </div>
    </div>
  );
}

function messageOf(error: unknown): string {
  if (error instanceof Error) return error.message;
  return String(error);
}
