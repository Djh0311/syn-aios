// 交办·干态(五态之「干」)——阶段3拆巨石第四刀:自 ProjectJiaobanPanel.tsx 原样迁出,零逻辑改动。
// 宪法归属:§一 干态(唯一问题=需要我吗·还要多久;逐格亮·不需要我=零打断)。
import { Pill } from "../../../components/SpecPrimitives";
import type {
  ProjectWorkflowChainStatus,
  SupervisorPilotReadModel,
} from "../../../lib/types";
import type { JiaobanPhase } from "../ProjectJiaobanPanel";
import { JiaobanRawSessionLink } from "./jiaobanSessionParts";
// 人话工程①(2026-07-20):humanizeChainProgress(含私有依赖 countDoneNodes)逐字迁 src/lib/humanize.ts;
// import-back + re-export 保导入面(Panel 经本文件 re-export 链不变)。
import { humanizeChainProgress } from "../../../lib/humanize";

export { humanizeChainProgress };

// 3. 干（人话进度）
export function JiaobanRunningState({
  chainStatus,
  directorPlanningElapsedMinutes,
  isNewSession,
  onStop,
  sessionChoice,
  latestSessionThreadId,
  onOpenAgentSession,
}: {
  chainStatus: ProjectWorkflowChainStatus | null;
  directorPlanningElapsedMinutes: number;
  isNewSession: boolean;
  onStop: () => void;
  // 「看原始对话」桥：existing 单跑中→看原始对话（能看实时进度）；哨兵单→latestSession 兜底看最近对话。
  sessionChoice: string | null;
  latestSessionThreadId: string | null;
  onOpenAgentSession?: (threadId: string) => void;
}) {
  const isDirectorPlanning = !chainStatus || chainStatus.nodes.length === 0;
  const progress = humanizeChainProgress(chainStatus, directorPlanningElapsedMinutes);
  return (
    <div className="project-canvas-detail-card jiaoban-running" aria-label="正在干">
      <div className="panel-heading">
        <div>
          <h3>正在干…</h3>
        </div>
        <Pill tone="candidate">进行中</Pill>
      </div>
      <div className="role-loop-plain" aria-label="进度（人话）">
        <p className="role-loop-plain-lead">
          <span className="jiaoban-spinner" aria-hidden="true" /> {progress}
        </p>
        {isDirectorPlanning && directorPlanningElapsedMinutes >= 2 ? (
          <p className="muted small-note">模型在长考;若超时会自动停下重试,不用干等</p>
        ) : null}
        {isNewSession ? (
          <p className="muted small-note">正在为需要新会话的任务逐一新建会话（约 1 分钟）…</p>
        ) : null}
      </div>
      <JiaobanRawSessionLink
        sessionChoice={sessionChoice}
        latestSessionThreadId={latestSessionThreadId}
        onOpenAgentSession={onOpenAgentSession}
      />
      <div className="workflow-state-actions">
        <button className="secondary-button" type="button" onClick={onStop}>
          停下
        </button>
      </div>
      <p className="muted small-note">想看每一步的过程，看右侧画布。</p>
    </div>
  );
}

// Station 2：主管试点只消费已存在的 sidecar 审计投影；不把任何事件写回链态。
export function JiaobanSupervisorPilotRunningState({
  runId,
  readModel,
  ledgerError,
}: {
  runId: string;
  readModel: SupervisorPilotReadModel | null;
  ledgerError: string | null;
}) {
  const status = readModel?.launch_status ?? "starting";
  const isActive = status === "starting" || status === "running";
  const waitingReason = readModel?.termination_reason.trim();
  const statusText =
    status === "running"
      ? "主管正在编排"
      : status === "waiting_user"
        ? waitingReason || "主管等待用户决定"
      : status === "exited"
        ? "主管进程已结束，业务状态以权威回程和验收账本为准"
        : status === "failed"
          ? "主管会话异常结束"
          : "主管正在启动";
  return (
    <div
      className="project-canvas-detail-card jiaoban-running"
      aria-label={isActive ? "主管进行中" : status === "waiting_user" ? "主管等待用户决定" : "主管已结束"}
    >
      <div className="panel-heading">
        <div>
          <h3>{isActive ? "主管进行中…" : status === "waiting_user" ? "主管等待用户决定" : "主管进程已结束"}</h3>
        </div>
        <Pill tone={status === "failed" || status === "waiting_user" ? "warn" : "candidate"}>{statusText}</Pill>
      </div>
      <div className="role-loop-plain" aria-label="主管账本事件流">
        <p className="role-loop-plain-lead">
          {isActive ? <span className="jiaoban-spinner" aria-hidden="true" /> : null} {statusText}
        </p>
        <p className="muted small-note">本单主管运行编号：{runId}</p>
        {ledgerError ? <p className="state-warning">主管账本暂时不可读：{ledgerError}</p> : null}
        {readModel?.audit_events.length ? (
          <ul className="jiaoban-boundary-points" aria-label="主管账本事件">
            {readModel.audit_events.map((event) => (
              <li key={event.event_id}>
                {event.tool}：{event.result_summary || event.result_status}
              </li>
            ))}
          </ul>
        ) : (
          <p className="muted small-note">账本事件正在到达…</p>
        )}
      </div>
    </div>
  );
}

export function isDirectorPlanningPhase(
  phase: JiaobanPhase,
  chainStatus: ProjectWorkflowChainStatus | null,
): boolean {
  return phase === "running" && (!chainStatus || chainStatus.nodes.length === 0);
}
