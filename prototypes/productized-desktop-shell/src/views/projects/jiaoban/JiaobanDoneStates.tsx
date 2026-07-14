// 交办·交货态(五态之「交货」)——阶段3拆巨石第五刀:自 ProjectJiaobanPanel.tsx 原样迁出,零逻辑改动。
// 宪法归属:§一 交货态(唯一问题=能信吗·证据呢;口供上脸+黄牌+[属实,沉淀])。
import { useState } from "react";
import { Badge } from "../../../components/Badge";
import type {
  AutoAdvanceRoleLoopOutcome,
  CreateMemoryCandidateInput,
  DirectorChainStep,
  GlobalSupervisorReviewOutcome,
  PendingAction,
  ProjectWorkflowChainStatus,
} from "../../../lib/types";
import { JiaobanNeedsReworkDisposal } from "./JiaobanBlockedStates";
import { JiaobanRawSessionLink } from "./jiaobanSessionParts";

// 刀A·口供上脸：单步徽章判定。执行态(failed/skipped)优先于自述；completed 才看 worker 口供。
export type StepReportFlag = {
  kind: "ok" | "yellow" | "fail" | "skip";
  badge: string;
  tone: "green" | "yellow" | "red" | "gray";
};

export function stepReportFlag(step: DirectorChainStep): StepReportFlag {
  if (step.state === "failed") {
    return { kind: "fail", badge: "失败", tone: "red" };
  }
  if (step.state === "skipped") {
    return { kind: "skip", badge: "跳过", tone: "gray" };
  }
  // 到这里视为已完成——看 worker 自述，自报没干完的不许装全绿。
  if (step.report_warning) {
    return { kind: "yellow", badge: step.report_warning, tone: "yellow" };
  }
  if (!step.report_summary) {
    return { kind: "yellow", badge: "没交汇报", tone: "yellow" };
  }
  const status = step.report_status ?? "";
  if (status === "done") {
    return { kind: "ok", badge: "自述：做好了", tone: "green" };
  }
  if (status === "partial") {
    return { kind: "yellow", badge: "自述：没干完", tone: "yellow" };
  }
  if (status === "failed") {
    return { kind: "yellow", badge: "自述：失败", tone: "yellow" };
  }
  return { kind: "yellow", badge: "自述：状态不明", tone: "yellow" };
}

// 黄牌数：只数「完成但自述有问题」的 yellow（failed/skipped 是执行态红/灰，不计入黄牌 N）。
export function countYellowFlags(steps: DirectorChainStep[]): number {
  return steps.filter((step) => stepReportFlag(step).kind === "yellow").length;
}

// 标题联动：有黄牌 → 「✓ 做好了（有 N 项要看一眼）」；全绿 → 「✓ 做好了」。自报没干完的不许装全绿。
export function jiaobanDoneTitle(steps: DirectorChainStep[]): string {
  const yellow = countYellowFlags(steps);
  return yellow > 0 ? `✓ 做好了（有 ${yellow} 项要看一眼）` : "✓ 做好了";
}

// 交货脸/失败脸每任务一行：任务标题 + 自述一句 + 人话徽章。无 steps → 不渲染（零回退）。
// 刀B·事实确认上下文（构造记忆候选需要的项目锚）。
export type FactMemoryContext = {
  projectRoot: string;
  projectId?: string | null;
  workflowId?: string | null;
};

// 刀B·事实确认：把「绿✓且有自述」的任务行构造成记忆**候选**入参（候选≠正式·待治理转正·治理一字不动）。
// claim=自述、body=标题+确认语；memory_type/risk/sensitive 取核查后合法最保守档（workflow_summary/low/project）。
export function buildFactMemoryCandidate(
  step: DirectorChainStep,
  ctx: FactMemoryContext,
): CreateMemoryCandidateInput {
  const nowIso = new Date().toISOString();
  const claim = (step.report_summary ?? step.title).trim();
  const scopeId = ctx.projectId ? `scope:${ctx.projectId}` : `scope:project:${ctx.projectRoot}`;
  return {
    project_root: ctx.projectRoot,
    project_id: ctx.projectId ?? null,
    workflow_id: ctx.workflowId ?? null,
    scope: {
      scope_id: scopeId,
      scope_type: "project",
      user_id: null,
      project_id: ctx.projectId ?? null,
      workflow_id: null,
      session_id: null,
      role_ids: [],
      document_refs: [],
      permission_policy_ref: null,
      model_export_policy: "local_only",
      valid_from: nowIso,
      valid_until: null,
    },
    memory_type: "workflow_summary",
    claim,
    body: `任务「${step.title}」经用户在交货脸确认属实。自述：${step.report_summary ?? "（无）"}`,
    source_refs: [
      {
        source_ref_id: `worker-report:${step.planned_task_id}`,
        source_type: "workflow_summary",
        source_id: ctx.workflowId ?? null,
        source_title: step.title,
        anchor: step.planned_task_id,
        captured_at: nowIso,
        authority_level: "user_confirmed",
        sensitive_level: "project",
      },
    ],
    generated_by_role: "user",
    generated_from: "explicit_user_confirmation",
    risk_level: "low",
    sensitive_level: "project",
    requires_user_confirmation: true,
    review_reason: "用户在交货脸确认任务属实，沉淀为项目记忆候选（候选≠正式，待治理转正）。",
    expected_store_revision: null,
  };
}

export function JiaobanStepReportList({
  steps,
  onConfirmFact,
  confirmedTaskIds,
}: {
  steps: DirectorChainStep[];
  onConfirmFact?: (step: DirectorChainStep) => void;
  confirmedTaskIds?: ReadonlySet<string>;
}) {
  if (!steps || steps.length === 0) {
    return null;
  }
  return (
    <ul className="jiaoban-step-report" aria-label="每一步的自述">
      {steps.map((step) => {
        const flag = stepReportFlag(step);
        return (
          <li key={step.planned_task_id} className={`jiaoban-step-row tone-${flag.tone}`}>
            <span className="jiaoban-step-title">{step.title}</span>
            {step.report_summary ? (
              <span className="jiaoban-step-say">{step.report_summary}</span>
            ) : null}
            <span className={`jiaoban-step-badge tone-${flag.tone}`}>
              {flag.tone === "yellow" ? "⚠ " : ""}
              {flag.badge}
            </span>
            {flag.kind === "ok" && step.report_summary && onConfirmFact ? (
              confirmedTaskIds?.has(step.planned_task_id) ? (
                <span className="jiaoban-fact-done">已沉淀 ✓</span>
              ) : (
                <button
                  type="button"
                  className="jiaoban-fact-btn"
                  onClick={() => onConfirmFact(step)}
                >
                  属实，沉淀
                </button>
              )
            ) : null}
          </li>
        );
      })}
    </ul>
  );
}

// B1·全局主管复核区（纯展示·无 hooks·export 供离线 DOM 断言直接调）。
// 词表死线：「全局主管意见/复核意见」——**不是审批**（意见不是闸，按钮全走现成用户动作）。
// 四态：loading / 意见到（总判 + 每任务点评 + 建议动作按钮）/ 不可用（人话 + [重试]·绝不零出路）/
// 没起（outcome null 且不 loading → 零渲染，如无本轮链、旧数据）。
export function JiaobanSupervisorReviewSection({
  loading,
  outcome,
  onRetry,
  onReplan,
}: {
  loading: boolean;
  outcome: GlobalSupervisorReviewOutcome | null;
  onRetry: () => void;
  onReplan: () => void;
}) {
  if (loading) {
    return (
      <div className="jiaoban-supervisor" aria-label="全局主管意见">
        <p className="jiaoban-field-label jiaoban-supervisor-title">全局主管意见</p>
        <p className="muted small-note">
          <span className="jiaoban-spinner" aria-hidden="true" /> 全局主管复核中…（约 2-7 分钟，不影响交货）
        </p>
      </div>
    );
  }
  if (!outcome) return null;
  const review = outcome.status === "ready" ? (outcome.review ?? null) : null;
  if (!review) {
    // 不可用：人话原因 + [重试]（force）——复核缺席不挡任何事，但绝不零出路。
    const reason = outcome.reason?.trim() || outcome.review?.unavailable_reason?.trim() || "原因不明";
    return (
      <div className="jiaoban-supervisor" aria-label="全局主管意见">
        <p className="jiaoban-field-label jiaoban-supervisor-title">全局主管意见</p>
        <p className="muted small-note">复核不可用：{reason}</p>
        <button className="secondary-button" type="button" onClick={onRetry}>
          重试复核
        </button>
      </div>
    );
  }
  // 总判一行（词表：意见，不是审批）。
  const overallLine =
    review.overall === "pass"
      ? "✓ 全局主管看过：这轮没发现问题"
      : review.overall === "needs_rework"
        ? "⚠ 全局主管意见：建议打回重拆"
        : "⚠ 全局主管意见：建议你亲自核验";
  const overallTone = review.overall === "pass" ? "jiaoban-supervisor-pass" : "jiaoban-supervisor-flag";
  return (
    <div className="jiaoban-supervisor" aria-label="全局主管意见">
      <p className="jiaoban-field-label jiaoban-supervisor-title">全局主管意见</p>
      <p className={`jiaoban-supervisor-overall ${overallTone}`}>{overallLine}</p>
      {review.summary.trim() ? <p className="jiaoban-supervisor-summary">{review.summary}</p> : null}
      {review.tasks.length > 0 ? (
        <ul className="jiaoban-supervisor-tasks" aria-label="每任务点评">
          {review.tasks.map((task, index) => (
            <li key={index} className={task.verdict === "issue" ? "jiaoban-supervisor-issue" : undefined}>
              {task.verdict === "issue" ? "⚠ " : ""}
              {task.title ? `${task.title}：` : ""}
              {task.comment}
            </li>
          ))}
        </ul>
      ) : null}
      {review.suggested_action === "replan" ? (
        <div className="workflow-state-actions">
          <button className="secondary-button" type="button" onClick={onReplan}>
            按建议打回重拆
          </button>
        </div>
      ) : null}
      {review.suggested_action === "human_verify" ? (
        <p className="jiaoban-supervisor-note">
          建议你亲验：{review.human_note.trim() || "亲自核验这轮结果。"}
        </p>
      ) : null}
      <button className="jiaoban-linklike jiaoban-supervisor-rerun" type="button" onClick={onRetry}>
        重新复核
      </button>
    </div>
  );
}

// 4. 交货
export function JiaobanDoneState({
  outcome,
  chainStatus,
  onContinue,
  needsRework,
  needsReworkActionError,
  needsReworkActionStarting,
  onNeedsReworkContinue,
  onNeedsReworkAction,
  onRequestAction,
  factCtx,
  sessionChoice,
  latestSessionThreadId,
  onOpenAgentSession,
  supervisorLoading,
  supervisorOutcome,
  onSupervisorRetry,
  onSupervisorReplan,
}: {
  outcome: AutoAdvanceRoleLoopOutcome | null;
  chainStatus: ProjectWorkflowChainStatus | null;
  onContinue: () => void;
  needsRework: { reason: string; actionsReady: boolean } | null;
  needsReworkActionError: string | null;
  needsReworkActionStarting: boolean;
  onNeedsReworkContinue: () => void;
  onNeedsReworkAction: (action: "change_session" | "rework" | "archive") => void;
  onRequestAction: (action: PendingAction) => void;
  factCtx: FactMemoryContext | null;
  // 「看原始对话」桥：existing 单→看原始对话（就是干这单的那条）；哨兵单→latestSession 兜底看最近对话。
  sessionChoice: string | null;
  latestSessionThreadId: string | null;
  onOpenAgentSession?: (threadId: string) => void;
  // B1·全局主管复核区（advisory）：意见 + 建议动作按钮（按钮走现成用户动作·意见不是闸）。
  supervisorLoading: boolean;
  supervisorOutcome: GlobalSupervisorReviewOutcome | null;
  onSupervisorRetry: () => void;
  onSupervisorReplan: () => void;
}) {
  const chain = outcome?.chain_outcome ?? null;
  const isCompleted =
    outcome?.stage === "completed" ||
    ["finished", "completed", "done", "succeeded"].includes(chainStatus?.state.trim().toLowerCase() ?? "");
  // 刀B·事实确认本地态（防重复点·经现成 create-memory-candidate PendingAction 走确认弹层）。
  const [confirmedTaskIds, setConfirmedTaskIds] = useState<ReadonlySet<string>>(() => new Set());
  const onConfirmFact = factCtx
    ? (step: DirectorChainStep) => {
        onRequestAction({
          kind: "create-memory-candidate",
          label: `沉淀记忆候选：${step.title}`,
          path: factCtx.projectRoot,
          source: "Tauri 应用数据目录",
          boundary: "只产候选、不是正式记忆；候选待治理转正才进正式记忆库。",
          memoryCandidateCreation: buildFactMemoryCandidate(step, factCtx),
        });
        setConfirmedTaskIds((prev) => new Set(prev).add(step.planned_task_id));
      }
    : undefined;
  const resultLine = chain
    ? `完成 ${chain.completed} 步${chain.stopped_reason ? `；中途停了：${chain.stopped_reason}` : ""}。`
    : outcome?.message || (isCompleted ? "做完了。" : "这单没有完整交货。");
  const isReadOnlyRun =
    (outcome?.planned_tasks ?? []).length > 0 &&
    (outcome?.planned_tasks ?? []).every((task) => task.scope.allowed_write_scope.length === 0);
  // fix3 后端新 warnings（如「角色已按 codex-dev 执行」「已接续上次中断的运行」）→ 小字列出，不挡主路径。
  const warnings = chain?.warnings ?? [];

  return (
    <div className="project-canvas-detail-card jiaoban-done" aria-label={isCompleted ? "做好了" : "未完整交货"}>
      <div className="panel-heading">
        <div>
          <h3 className="jiaoban-done-title">
            {isCompleted
              ? jiaobanDoneTitle(chain?.steps ?? [])
              : needsRework
                ? "这一步需要重做"
                : "这单没有完整交货"}
          </h3>
        </div>
        <Badge tone={isCompleted ? "candidate" : "warning"}>{isCompleted ? "已交货" : "未交货"}</Badge>
      </div>
      {/* 批1·骨架化(DESIGN.md §二):状态 pill 行=现有事实上脸,不造数据;只读单注升为 pill。 */}
      {isCompleted ? (
        <div className="jiaoban-done-pills" aria-label="这单概览">
          {chain ? <span className="jiaoban-step-badge tone-green">完成 {chain.completed} 步</span> : null}
          {countYellowFlags(chain?.steps ?? []) > 0 ? (
            <span className="jiaoban-step-badge tone-yellow">⚠ {countYellowFlags(chain?.steps ?? [])} 项要看一眼</span>
          ) : null}
          {isReadOnlyRun ? <span className="jiaoban-step-badge tone-gray">只读单·未改文件</span> : null}
        </div>
      ) : null}
      <div className="role-loop-plain" aria-label="结果（人话）">
        <p className="role-loop-plain-lead">{resultLine}</p>
        {!isCompleted && isReadOnlyRun ? <p className="role-loop-plain-note">只读单·未改文件</p> : null}
      </div>
      {(chain?.steps ?? []).length > 0 ? <p className="jiaoban-field-label">干了什么</p> : null}
      <JiaobanStepReportList
        steps={chain?.steps ?? []}
        onConfirmFact={onConfirmFact}
        confirmedTaskIds={confirmedTaskIds}
      />
      {needsRework ? (
        <JiaobanNeedsReworkDisposal
          reason={needsRework.reason}
          actionsReady={needsRework.actionsReady}
          starting={needsReworkActionStarting}
          error={needsReworkActionError}
          onContinue={onNeedsReworkContinue}
          onAction={onNeedsReworkAction}
        />
      ) : null}
      {/* B1：全局主管复核区——交货后 async 后填（loading/意见/不可用+重试），不挡上面交货内容。 */}
      <JiaobanSupervisorReviewSection
        loading={supervisorLoading}
        outcome={supervisorOutcome}
        onRetry={onSupervisorRetry}
        onReplan={onSupervisorReplan}
      />
      {warnings.length > 0 ? (
        <ul className="jiaoban-warnings muted small-note" aria-label="附带说明">
          {warnings.map((w, i) => (
            <li key={i}>{w}</li>
          ))}
        </ul>
      ) : null}
      {!needsRework ? (
        <div className="workflow-state-actions">
          <button className="primary-button" type="button" onClick={onContinue}>
            继续弄别的
          </button>
          {/* 批1·「查看原始口供」下钻挪进动作行(骨架动作位;原独立行删,防双入口)。 */}
          <JiaobanRawSessionLink
            sessionChoice={sessionChoice}
            latestSessionThreadId={latestSessionThreadId}
            onOpenAgentSession={onOpenAgentSession}
          />
        </div>
      ) : (
        <JiaobanRawSessionLink
          sessionChoice={sessionChoice}
          latestSessionThreadId={latestSessionThreadId}
          onOpenAgentSession={onOpenAgentSession}
        />
      )}
    </div>
  );
}
