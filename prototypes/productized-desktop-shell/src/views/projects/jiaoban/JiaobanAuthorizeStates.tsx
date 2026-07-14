// 交办·说态+批态(五态之「说/批」)——阶段3拆巨石第六刀:自 ProjectJiaobanPanel.tsx 原样迁出,零逻辑改动。
// 宪法归属:§一 说态(唯一问题=它理解对了吗)/批态(唯一问题=它要动什么·我敢不敢)。
import type {
  GlobalSupervisorBoundaryReviewOutcome,
  ProjectConsultationProposal,
  ProjectDirectorPlannedTask,
  SessionRecord,
} from "../../../lib/types";
import type { JiaobanOrchestrationMode } from "../ProjectJiaobanPanel";
import { JiaobanSessionPicker } from "./jiaobanSessionParts";

// 1. 说
export function JiaobanSayState({
  goal,
  onGoalChange,
  onSubmit,
  lastStopHint,
  loading,
  error,
  onEditAgain,
}: {
  goal: string;
  onGoalChange: (value: string) => void;
  onSubmit: () => void;
  lastStopHint: string | null;
  loading: boolean;
  error: string | null;
  onEditAgain: () => void;
}) {
  return (
    <div className="project-canvas-detail-card jiaoban-say" aria-label="想让 AI 干点啥">
      <div className="panel-heading">
        <div>
          <h3>想让 AI 干点啥？</h3>
        </div>
      </div>
      {lastStopHint ? (
        <div className="jiaoban-say-hint" role="note" aria-label="上次停在哪">
          上次停在：{lastStopHint}——目标已带回来，改一改再出一版新方案。
        </div>
      ) : null}
      {/* fix8：出方案失败上脸——人话（供给类专句/后端原话）+ 目标不清空，绝不静默死。 */}
      {error ? (
        <div className="jiaoban-consult-error" role="alert" aria-label="出方案没成">
          <span aria-hidden="true">⚠</span> {error}
        </div>
      ) : null}
      <label className="proposal-decision-field">
        <span>说一句话，AI 会读你的项目、想个方案给你审。</span>
        <textarea
          value={goal}
          onChange={(event) => onGoalChange(event.target.value)}
          placeholder="例：给这小游戏加个计分板——吃到东西 +1、显示在右上角。"
          rows={4}
          disabled={loading}
        />
      </label>
      <div className="workflow-state-actions">
        {error && !loading ? (
          // 失败态：绝不零按钮——[重试]（重发原目标·目标还在框里）+ [改要求]（回编辑态改了再出）。
          <>
            <button className="primary-button" type="button" disabled={!goal.trim()} onClick={onSubmit}>
              重试
            </button>
            <button className="secondary-button" type="button" onClick={onEditAgain}>
              改要求
            </button>
          </>
        ) : (
          <button className="primary-button" type="button" disabled={loading || !goal.trim()} onClick={onSubmit}>
            {loading ? "AI 正在读项目、想方案…（约 1–2 分钟）" : "出方案"}
          </button>
        )}
      </div>
    </div>
  );
}

// 2. 批（授权卡·定稿字段）
// export 供离线 DOM 断言（fix9 诚实脸两态；renderToStaticMarkup 渲染·不平铺调用）。
export function JiaobanAuthorizeState({
  proposal,
  proposalIsStale,
  proposalAgeDays,
  sessions,
  sessionChoice,
  onSessionChoiceChange,
  orchestrationMode = "classic",
  onOrchestrationModeChange = () => {},
  supervisorPilotDisabledReason = null,
  classicDisabledReason = null,
  amendment,
  onAmendmentChange,
  onAmend,
  onAuthorizeAndStart,
  onRePlan,
  onDecline,
  starting,
  consultLoading,
  consultError,
  worksmapSwitchOn,
  onToggleWorksmapSwitch,
  worksmapTasks,
  worksmapLoading,
  worksmapError,
  boundaryLoading,
  boundaryOutcome,
  onBoundaryRetry,
  onOpenAgentSession,
}: {
  proposal: ProjectConsultationProposal;
  proposalIsStale: boolean;
  proposalAgeDays: number;
  sessions: SessionRecord[];
  sessionChoice: string | null;
  onSessionChoiceChange: (value: string | null) => void;
  orchestrationMode?: JiaobanOrchestrationMode;
  onOrchestrationModeChange?: (mode: JiaobanOrchestrationMode) => void;
  supervisorPilotDisabledReason?: string | null;
  classicDisabledReason?: string | null;
  amendment: string;
  onAmendmentChange: (value: string) => void;
  onAmend: () => void;
  onAuthorizeAndStart: () => void;
  onRePlan: () => void;
  onDecline: () => void;
  starting: boolean;
  consultLoading: boolean;
  consultError: string | null;
  worksmapSwitchOn: boolean;
  onToggleWorksmapSwitch: (value: boolean) => void;
  worksmapTasks: ProjectDirectorPlannedTask[] | null;
  worksmapLoading: boolean;
  worksmapError: string | null;
  // B2·全局主管批前边界意见（advisory·意见不是闸·async·缺席不挡批）。
  boundaryLoading: boolean;
  boundaryOutcome: GlobalSupervisorBoundaryReviewOutcome | null;
  onBoundaryRetry: () => void;
  // 「看原始对话」桥：批卡收纳行入口（**必填**·透传给 picker）。批卡是任务点名的主入口，
  // 设必填让上游漏传直接 tsc 报错——防「组件接了、上游忘喂、入口静默不显」的假绿（审查线逮到过）。
  onOpenAgentSession: (threadId: string) => void;
}) {
  const targetFiles = extractTargetFiles(proposal.proposed_steps);
  const willWrite = proposal.scope_draft.allowed_write_roots.length > 0;
  const workerAcceptance = proposal.worker_acceptance_criteria ?? [];
  const controlCoreAcceptance = proposal.control_core_acceptance_criteria ?? [];
  const supervisorAcceptance = proposal.supervisor_acceptance_criteria ?? [];
  const hasRoleAcceptance =
    workerAcceptance.length > 0 || controlCoreAcceptance.length > 0 || supervisorAcceptance.length > 0;
  // 按钮旁的状态话只说明右侧预演画布，不再暗示卡内另有一张图。
  const worksmapReady = !worksmapLoading && !worksmapError && !!(worksmapTasks && worksmapTasks.length);
  const worksmapNote =
    !worksmapSwitchOn || worksmapError
      ? null
      : worksmapLoading
        ? "右侧预演画布正在绘制工序图…（可先批，先批就按现场拆）"
        : worksmapReady
          ? "✓ 工序图已在右侧预演画布显示"
          : null;

  return (
    <div className="project-canvas-detail-card jiaoban-authorize" aria-label="方案">

      {/* 旧方案不冒充当前：不是今天生成 → 顶部黄条 + 主按钮换「重新说目标」，防再批库存。 */}
      {proposalIsStale ? (
        <div className="jiaoban-stale-banner" role="note" aria-label="旧方案提醒">
          <span aria-hidden="true">⚠</span>
          {/* 同 advice-only 警条：正文包单 span 防 flex 拆柱（此条现在恰好没内联元素才幸免，统一防）。 */}
          <span className="jiaoban-banner-body">
            这是 {proposalAgeDays} 天前的旧方案，项目可能已变——建议重新说一遍。
          </span>
        </div>
      ) : null}

      {/* 写根空是可执行的只读单：仍走同一人闸，只是不授予写入。 */}
      {!willWrite ? (
        <div className="jiaoban-advice-only-banner" role="note" aria-label="只读单提醒">
          <span aria-hidden="true">⚠</span>
          <span className="jiaoban-banner-body">
            这单是只读的——AI 只看不改，交货是结论不是改动
          </span>
        </div>
      ) : null}

      <div className="role-loop-plain jiaoban-plan-body" aria-label="方案要点（人话）">
        <p className="jiaoban-field">
          <span className="jiaoban-field-label">我来做：</span>
          {proposal.goal_summary || proposal.user_goal}
        </p>
        {targetFiles ? (
          <p className="jiaoban-field">
            <span className="jiaoban-field-label">会改的文件：</span>
            {targetFiles}
          </p>
        ) : null}
        {hasRoleAcceptance ? (
          <>
            <p className="jiaoban-field">
              <span className="jiaoban-field-label">执行 Agent 要做到：</span>
              {workerAcceptance.join("；") || "未提供"}
            </p>
            <p className="jiaoban-field">
              <span className="jiaoban-field-label">Syn 要保证：</span>
              {controlCoreAcceptance.join("；") || "未提供"}
            </p>
            <p className="jiaoban-field">
              <span className="jiaoban-field-label">主管要判断：</span>
              {supervisorAcceptance.join("；") || "未提供"}
            </p>
          </>
        ) : proposal.acceptance_criteria.length ? (
          <p className="jiaoban-field">
            <span className="jiaoban-field-label">改完怎么验：</span>
            {proposal.acceptance_criteria.join("；")}
          </p>
        ) : null}
      </div>

      {/* B2·全局主管批前边界意见：方案要点之后、按钮区之前。async 后填·意见没到也可以先批（不拦事）。 */}
      <JiaobanBoundaryReviewSection
        loading={boundaryLoading}
        outcome={boundaryOutcome}
        onRetry={onBoundaryRetry}
      />

      <JiaobanWorksmap
        suggestWorkflow={proposal.suggest_workflow === true}
        switchOn={worksmapSwitchOn}
        onToggleSwitch={onToggleWorksmapSwitch}
      />

      <JiaobanOrchestrationModePicker
        mode={orchestrationMode}
        disabledReason={supervisorPilotDisabledReason}
        classicDisabledReason={classicDisabledReason}
        disabled={starting || consultLoading}
        onChange={onOrchestrationModeChange}
      />

      <JiaobanSessionPicker
        sessions={sessions}
        sessionChoice={sessionChoice}
        onSessionChoiceChange={onSessionChoiceChange}
        onOpenAgentSession={onOpenAgentSession}
        label="给第一个预演节点预填对话"
      />

      {willWrite ? (
        <div className="jiaoban-grant" role="note">
          <span aria-hidden="true">🔓</span> 需要你允许：改这个测试项目
        </div>
      ) : null}

      <label className="proposal-decision-field jiaoban-amend">
        <input
          type="text"
          aria-label="修改方案"
          value={amendment}
          onChange={(event) => onAmendmentChange(event.target.value)}
          placeholder="例：改成暗色、分数存下来…"
          disabled={consultLoading}
        />
      </label>

      {/* fix8：改要求出新方案期间/失败也上脸——loading 提示 + 失败人话，绝不静默。 */}
      {consultLoading ? (
        <p className="muted small-note">正在出新方案…（约 1–2 分钟）</p>
      ) : consultError ? (
        <div className="jiaoban-consult-error" role="alert" aria-label="出方案没成">
          <span aria-hidden="true">⚠</span> {consultError}
        </div>
      ) : null}

      {worksmapNote ? (
        <p className={`jiaoban-worksmap-cta ${worksmapReady ? "ready" : ""}`}>{worksmapNote}</p>
      ) : null}

      <div className="workflow-state-actions">
        {!willWrite ? (
          // 只读单的唯一开工门仍是 [允许并开始]；重新出方案保留为次操作。
          <>
            <button
              className="primary-button"
              type="button"
              disabled={starting || consultLoading}
              onClick={onAuthorizeAndStart}
            >
              {starting ? "正在开始…" : "允许并开始（只读）"}
            </button>
            <button
              className="secondary-button"
              type="button"
              disabled={starting || consultLoading}
              onClick={onRePlan}
            >
              {consultLoading ? "正在出新方案…" : "重新出方案（要动手）"}
            </button>
          </>
        ) : proposalIsStale ? (
          // 旧方案：主按钮 = 重新说目标；[允许并开始] 降为次按钮（防再批库存），但仍可手动点。
          <>
            <button
              className="primary-button"
              type="button"
              disabled={starting || consultLoading}
              onClick={onRePlan}
            >
              {consultLoading ? "正在出新方案…" : "重新说目标出新方案"}
            </button>
            <button
              className="secondary-button"
              type="button"
              disabled={starting || consultLoading}
              onClick={onAuthorizeAndStart}
            >
              {starting ? "正在开始…" : "仍要允许并开始（旧方案）"}
            </button>
          </>
        ) : (
          <>
            <button
              className="primary-button"
              type="button"
              disabled={starting || consultLoading}
              onClick={onAuthorizeAndStart}
            >
              {starting ? "正在开始…" : "允许并开始"}
            </button>
            <button
              className="secondary-button"
              type="button"
              disabled={starting || consultLoading || !amendment.trim()}
              onClick={onAmend}
            >
              {consultLoading ? "正在出新方案…" : "按我说的改"}
            </button>
          </>
        )}
        <button className="secondary-button" type="button" disabled={starting} onClick={onDecline}>
          先不做
        </button>
      </div>
    </div>
  );
}

// Station 2：按单选择入口。默认经典；试点不可用时仍展示原因，不能靠前端状态偷开。
export function JiaobanOrchestrationModePicker({
  mode,
  disabledReason,
  classicDisabledReason = null,
  disabled,
  onChange,
}: {
  mode: JiaobanOrchestrationMode;
  disabledReason: string | null;
  classicDisabledReason?: string | null;
  disabled: boolean;
  onChange: (mode: JiaobanOrchestrationMode) => void;
}) {
  const pilotDisabled = disabled || disabledReason !== null;
  const classicDisabled = disabled || classicDisabledReason !== null;
  return (
    <fieldset className="proposal-decision-field" aria-label="执行模式">
      <legend className="jiaoban-field-label">执行模式</legend>
      <label className={classicDisabled ? "muted" : undefined}>
        <input
          type="radio"
          name="jiaoban-orchestration-mode"
          checked={mode === "classic"}
          disabled={classicDisabled}
          onChange={() => onChange("classic")}
        />
        经典状态机（默认）
      </label>
      <label className={pilotDisabled ? "muted" : undefined}>
        <input
          type="radio"
          name="jiaoban-orchestration-mode"
          checked={mode === "supervisor_pilot"}
          disabled={pilotDisabled}
          onChange={() => onChange("supervisor_pilot")}
        />
        主管编排（试点）
      </label>
      {disabledReason ? <p className="muted small-note">{disabledReason}</p> : null}
      {classicDisabledReason ? <p className="muted small-note">{classicDisabledReason}</p> : null}
    </fieldset>
  );
}

// B2·全局主管批前边界意见区（纯展示·无 hooks·export 供离线 DOM 断言直接调）。
// 词表死线：「全局主管意见/边界意见」——**不是审批**（意见不是闸·不拦批·按钮区行为一概不变）。
// 四态：loading / 意见到（verdict 人话行 + points 列表·mismatch 告警调）/ 不可用（人话 + [重试]）/
// 没触发（outcome null 且不 loading → 零渲染，如 stale 方案/无方案/意见缺席）。
export function JiaobanBoundaryReviewSection({
  loading,
  outcome,
  onRetry,
}: {
  loading: boolean;
  outcome: GlobalSupervisorBoundaryReviewOutcome | null;
  onRetry: () => void;
}) {
  if (loading) {
    return (
      <div className="jiaoban-boundary" aria-label="全局主管意见">
        <p className="jiaoban-field-label jiaoban-boundary-title">全局主管意见（批前边界）</p>
        <p className="muted small-note">
          <span className="jiaoban-spinner" aria-hidden="true" /> 全局主管正在看边界…（意见没到也可以先批——它不拦事）
        </p>
      </div>
    );
  }
  if (!outcome) return null;
  const review = outcome.status === "ready" ? (outcome.review ?? null) : null;
  if (!review) {
    // 不可用：人话原因 + [重试]（force）——意见缺席不挡批，但绝不零出路。
    const reason = outcome.reason?.trim() || outcome.review?.unavailable_reason?.trim() || "原因不明";
    return (
      <div className="jiaoban-boundary" aria-label="全局主管意见">
        <p className="jiaoban-field-label jiaoban-boundary-title">全局主管意见（批前边界）</p>
        <p className="muted small-note">边界意见暂时不可用：{reason}（不影响你批）</p>
        <button className="secondary-button" type="button" onClick={onRetry}>
          重试
        </button>
      </div>
    );
  }
  // verdict 人话行（词表：意见，不是审批）。mismatch/caution 告警调、looks_ok 一行绿。
  const verdictLine =
    review.verdict === "looks_ok"
      ? "✓ 全局主管看过：范围和你的目标对得上"
      : review.verdict === "mismatch"
        ? "⚠ 全局主管意见：这方案好像对不上你的目标"
        : "⚠ 全局主管提醒：有几处要留意一下";
  const verdictTone = review.verdict === "looks_ok" ? "jiaoban-boundary-ok" : "jiaoban-boundary-flag";
  return (
    <div className="jiaoban-boundary" aria-label="全局主管意见">
      <p className="jiaoban-field-label jiaoban-boundary-title">全局主管意见（批前边界）</p>
      <p className={`jiaoban-boundary-verdict ${verdictTone}`}>{verdictLine}</p>
      {review.summary.trim() ? <p className="jiaoban-boundary-summary">{review.summary}</p> : null}
      {review.points.length > 0 ? (
        <ul className="jiaoban-boundary-points" aria-label="边界意见要点">
          {review.points.map((point, index) => (
            <li key={index}>{point}</li>
          ))}
        </ul>
      ) : null}
      <p className="muted small-note jiaoban-boundary-foot">这只是提醒，批不批还是你说了算。</p>
      <button className="jiaoban-linklike jiaoban-boundary-rerun" type="button" onClick={onRetry}>
        重新看一遍
      </button>
    </div>
  );
}

// 刀2「批前看图」的开关仍留在授权卡：它控制预拆，不再在卡内重复画图。
function JiaobanWorksmap({
  suggestWorkflow,
  switchOn,
  onToggleSwitch,
}: {
  suggestWorkflow: boolean;
  switchOn: boolean;
  onToggleSwitch: (value: boolean) => void;
}) {
  return (
    <div className="jiaoban-worksmap" aria-label="工作流预演开关">
      <label className="jiaoban-worksmap-toggle">
        <input type="checkbox" checked={switchOn} onChange={(event) => onToggleSwitch(event.target.checked)} />
        <span>按工作流来（在右侧预演画布看工序图）</span>
        {suggestWorkflow ? <span className="jiaoban-worksmap-suggest">AI 建议：这活值得先预演</span> : null}
      </label>
    </div>
  );
}

function extractTargetFiles(proposedSteps: string[]): string | null {
  const line = proposedSteps.find((step) => step.startsWith("目标文件："));
  if (!line) return null;
  const files = line.replace(/^目标文件：/, "").trim();
  return files || null;
}
