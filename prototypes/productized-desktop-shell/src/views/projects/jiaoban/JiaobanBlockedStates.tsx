// 交办·卡住/待决定态(五态之「卡住」)——阶段3拆巨石第三刀:自 ProjectJiaobanPanel.tsx 原样迁出,零逻辑改动。
// 宪法归属:§一 卡住态(唯一问题=为什么停·我选哪个;人话停因+死配对按钮·永不冻)。
import { Badge } from "../../../components/Badge";
import type { AutoAdvanceRoleLoopOutcome, SessionRecord } from "../../../lib/types";
import { JiaobanRawSessionLink, JiaobanSessionPicker } from "./jiaobanSessionParts";

export function JiaobanWaitingDecisionState({
  reason,
  actionsReady,
  starting,
  error,
  onContinue,
  onChangeSession,
  onRework,
  onArchive,
}: {
  reason: string;
  actionsReady: boolean;
  starting: boolean;
  error: string | null;
  onContinue: () => void;
  onChangeSession: () => void;
  onRework: () => void;
  onArchive: () => void;
}) {
  return (
    <div className="project-canvas-detail-card jiaoban-blocked" aria-label="待你决定">
      <div className="panel-heading">
        <div>
          <h3 className="jiaoban-blocked-title">待你决定</h3>
        </div>
        <Badge tone="warning">未自动重跑</Badge>
      </div>
      <div className="role-loop-plain" aria-label="worker 求助原文">
        <p className="role-loop-plain-lead">{reason}</p>
        {error ? <p className="jiaoban-consult-error" role="alert">{error}</p> : null}
        {!actionsReady ? <p className="muted small-note">求助已停住；任务信息载入后可继续或换会话。</p> : null}
      </div>
      <div className="workflow-state-actions">
        <button className="primary-button" type="button" disabled={starting || !actionsReady} onClick={onContinue}>
          让它继续（按现状态）
        </button>
        <button className="secondary-button" type="button" disabled={starting || !actionsReady} onClick={onChangeSession}>
          换个新会话重做
        </button>
        <button className="secondary-button" type="button" disabled={starting} onClick={onRework}>
          退回主管重拆
        </button>
        <button className="secondary-button" type="button" disabled={starting || !actionsReady} onClick={onArchive}>
          结束这单
        </button>
      </div>
    </div>
  );
}

// 主管已明确退回时，这里是主界面的用户处置入口；四个动作都要可见，绝不自动重跑。
export function JiaobanNeedsReworkDisposal({
  reason,
  actionsReady,
  starting,
  error,
  onContinue,
  onAction,
}: {
  reason: string;
  actionsReady: boolean;
  starting: boolean;
  error: string | null;
  onContinue: () => void;
  onAction: (action: "change_session" | "rework" | "archive") => void;
}) {
  return (
    <div className="role-loop-plain" aria-label="主管退回处置">
      <p className="role-loop-plain-lead">主管退回理由：{reason}</p>
      {error ? <p className="jiaoban-consult-error" role="alert">{error}</p> : null}
      {!actionsReady ? <p className="muted small-note">重做信息还在载入，三个处置按钮会稍后可用。</p> : null}
      <div className="workflow-state-actions">
        <button className="primary-button" type="button" disabled={starting} onClick={onContinue}>
          接着跑（按原样重做）
        </button>
        <button
          className="secondary-button"
          type="button"
          disabled={starting || !actionsReady}
          onClick={() => onAction("change_session")}
        >
          换个新会话重做
        </button>
        <button
          className="secondary-button"
          type="button"
          disabled={starting || !actionsReady}
          onClick={() => onAction("rework")}
        >
          退回主管重拆
        </button>
        <button
          className="secondary-button"
          type="button"
          disabled={starting || !actionsReady}
          onClick={() => onAction("archive")}
        >
          结束这单
        </button>
      </div>
    </div>
  );
}

// 停因→动作 死配对（§2.2）。给定 outcome / error / 方案是否已确认，判这张卡住脸该主打哪个按钮。
// 铁律：绝不返回零按钮（fallback 双给[接着跑]+[重新说目标]）。
type BlockedPlan = {
  // 主按钮语义：continue=[接着跑,不用重批]，replan=[重新说目标出新方案]，session=先选一条会话（选完再[接着跑]）。
  primary: "continue" | "replan" | "session";
  // 是否同时给次按钮（另一条出路）。
  showReplanSecondary: boolean;
  showContinueSecondary: boolean;
  // 主按钮上方的一句提示（如「上一步失败了，接着跑会从拆任务重来」）。
  note: string | null;
};

// 关键词特征判定（都在前端 message/stop_reason 里找人话词，不碰后端）。
// export 供离线 DOM 断言测试直接喂各类停因验按钮（行为中性，不改运行时）。
export function classifyBlocked(
  outcome: AutoAdvanceRoleLoopOutcome | null,
  error: string | null,
  planIsConfirmed: boolean,
): BlockedPlan {
  const stage = outcome?.stage ?? "";
  const text = `${outcome?.stop_reason ?? ""} ${outcome?.message ?? ""} ${error ?? ""}`;

  // 主动结束是终态，不再给「接着跑」造成可恢复错觉；只允许重新说目标另起一单。
  if (outcome?.stop_reason?.startsWith("archived:")) {
    return {
      primary: "replan",
      showReplanSecondary: false,
      showContinueSecondary: false,
      note: "这单已按你的选择结束；如需继续，请重新说目标另起一单。",
    };
  }

  // 1) needs_binding / 会话类 → 先选一条会话，选完回[接着跑]。
  const needsBinding =
    stage === "needs_binding" ||
    (outcome?.needs_binding_count ?? 0) > 0 ||
    /没.{0,3}会话|选.{0,3}会话|绑.{0,4}会话|哪个对话|接现有|会话.{0,3}(缺|没|未)/.test(text);
  if (needsBinding) {
    // 会话类：选会话是正路；但方案已确认时，选完就能[接着跑]，故次按钮给 continue。
    return {
      primary: "session",
      showReplanSecondary: true,
      showContinueSecondary: planIsConfirmed,
      note: null,
    };
  }

  // 2) blocked·写范围 / 方案内容类（message 含「方案缺 / 写范围 / 重新让 AI 出方案」）→ 重新说目标。
  //    这类是方案本身不够（少写范围、内容缺），接着跑也过不去，得回去出新方案。
  const planContentIssue = /方案.{0,3}(缺|不全|不够|有问题|没.{0,2}写)|写范围|可写|允许.{0,2}改|重新.{0,4}方案|重新让.{0,3}出/.test(
    text,
  );
  if (planContentIssue) {
    return {
      primary: "replan",
      showReplanSecondary: false,
      // 方案已确认时仍给个[接着跑]次口（万一只是复核噪音，接着跑能过）。
      showContinueSecondary: planIsConfirmed,
      note: null,
    };
  }

  // 2b) 主管拆不出任务(空任务列表·07-16 真单实案):方案带着没答的问题或说得太笼统——
  //     [接着跑]=原样重拆必然再空的死循环,正路=补充说明出新方案。必须先于「拆任务|失败」的临时类判定。
  if (/空任务列表|拆不出.{0,4}任务/.test(text)) {
    return {
      primary: "replan",
      showReplanSecondary: false,
      showContinueSecondary: planIsConfirmed, // 万一重拆能出,留个次口
      note: "主管拆不出可执行的任务——方案里还有没答的问题，或者活说得太笼统。把问题答上、活说得更具体，再出一版方案。",
    };
  }

  // 3) startError / 拆任务失败 / 超时 / flaky → [接着跑]（注明「上一步失败了，接着跑会从拆任务重来」）。
  const transientFailure =
    /拆任务|超时|timeout|timed out|失败|重试|中断|flaky|临时|偶发|网络|连接/i.test(text) ||
    (!!error && !planContentIssue); // startError 走到这（非方案内容类）多为合流/推进途中失败
  if (transientFailure && planIsConfirmed) {
    return {
      primary: "continue",
      showReplanSecondary: true,
      showContinueSecondary: false,
      note: "上一步失败了，接着跑会从拆任务重来。",
    };
  }

  // 4) blocked·其它（含「角色」类·fix3 后端钳位后应基本消失）→ [接着跑]主、重新说目标次。
  if (planIsConfirmed && (stage === "blocked" || stage === "no_dispatchable" || !!outcome)) {
    return {
      primary: "continue",
      showReplanSecondary: true,
      showContinueSecondary: false,
      note: null,
    };
  }

  // 5) 兜底（识别不了 / 方案未确认无法接着跑）→ 至少给[重新说目标]；方案已确认再补[接着跑]。
  //    绝不零按钮。
  return {
    primary: "replan",
    showReplanSecondary: false,
    showContinueSecondary: planIsConfirmed,
    note: null,
  };
}

// 5. 卡住（永不冻）：按停因死配对给「能点的正确按钮」。绝不零按钮终态。
// export 供离线 DOM 断言测试直接挂载验各分支按钮（行为中性，不改运行时）。
//
// 乙型「出问题了」（定稿 F·2026-07-14）：停因 + 「直接回它一句」回话框 + [发送并继续]。
// ⚠️ follow-up 回话通道**后端未就绪**（核实物 2026-07-15：`follow_up_suggestions` 是 worker 回程里给主管的
// 建议字段，不是「用户回话给 worker」的通道；tauri.ts 零 follow 命令；后端包
// `tasks/2026-07-15-backend-ui-support-readmodels-package-v1.md` §C 正在勘察补缺）。
// 故按包规：UI 先立形态 + disabled + 人话「通道接线中」，零假按钮（宪法 §四.3 禁死按钮）。
// 通道就绪后：把 followUpReady 改真、接 onSendFollowUp 即可，形态不动。
export function JiaobanBlockedState({
  outcome,
  error,
  planIsConfirmed,
  sessions,
  sessionChoice,
  onSessionChoiceChange,
  onContinueRun,
  onRePlan,
  starting,
  onOpenWorkflow,
  latestSessionThreadId,
  onOpenAgentSession,
  replyDraft = "",
  onReplyDraftChange,
  followUpReady = false,
  onSendFollowUp,
}: {
  outcome: AutoAdvanceRoleLoopOutcome | null;
  error: string | null;
  planIsConfirmed: boolean;
  sessions: SessionRecord[];
  sessionChoice: string | null;
  onSessionChoiceChange: (value: string | null) => void;
  onContinueRun: () => void;
  onRePlan: () => void;
  starting: boolean;
  onOpenWorkflow: (() => void) | null;
  // 「看原始对话」桥：面级入口（卡住脸不放 picker 行内入口·防一脸双入口）。existing 单→看原始对话；
  // 哨兵单→latestSession 兜底看最近对话；皆无→不显。
  latestSessionThreadId: string | null;
  onOpenAgentSession?: (threadId: string) => void;
  // 乙型回话框：状态提升到 ProjectJiaobanPanel（本组件被离线测试平铺裸调，不能有 hooks）。
  replyDraft?: string;
  onReplyDraftChange?: (value: string) => void;
  followUpReady?: boolean;
  onSendFollowUp?: () => void;
}) {
  // 停因人话：直接用后端 message / stop_reason（已带具体原因，不包糊话盖住）；再兜底一句 error。
  const reason =
    outcome?.stop_reason?.trim() ||
    outcome?.message?.trim() ||
    error?.trim() ||
    "碰到拿不准的地方，先停下了。";

  const plan = classifyBlocked(outcome, error, planIsConfirmed);
  const warnings = outcome?.chain_outcome?.warnings ?? [];
  const archived = outcome?.stop_reason?.startsWith("archived:") ?? false;
  const interrupted = outcome?.stage === "interrupted" && !archived;
  const faceTitle = archived ? "这单已结束" : interrupted ? "已停下·可接着跑" : "⚠ 卡住了";
  const faceLabel = archived ? "本单已结束" : interrupted ? "已停下可接着跑" : "卡住了";
  const faceBadge = archived ? "已结束" : interrupted ? "可接着跑" : "停下了";
  // 乙型 =「真出问题」那一支（已结束/已停下两支不是出问题，不给回话框）。
  const isTypeB = !archived && !interrupted;
  // 通道没通时不抢主按钮：死配对的[接着跑]/[重新说目标]仍是能点的主路径（宪法「永不冻」）。
  // 通道通了→[发送并继续]升主、死配对降次（定稿 F 乙型的按钮次序）。
  const typeBPrimary = isTypeB && followUpReady;

  // 主/次按钮拼装。continue 主按钮统一文案「接着跑（方案已批过，不用重批）」。
  const continueBtn = (isPrimary: boolean) => (
    <button
      key="continue"
      className={isPrimary ? "primary-button" : "secondary-button"}
      type="button"
      disabled={starting}
      onClick={onContinueRun}
    >
      {starting ? "正在开始…" : "接着跑（方案已批过，不用重批）"}
    </button>
  );
  const replanBtn = (isPrimary: boolean) => (
    <button
      key="replan"
      className={isPrimary ? "primary-button" : "secondary-button"}
      type="button"
      disabled={starting}
      onClick={onRePlan}
    >
      重新说目标出新方案
    </button>
  );

  return (
    <div className="project-canvas-detail-card jiaoban-blocked" aria-label={faceLabel}>
      <div className="panel-heading">
        <div>
          <h3 className="jiaoban-blocked-title">{faceTitle}</h3>
        </div>
        <Badge tone="warning">{faceBadge}</Badge>
      </div>
      <div className="role-loop-plain" aria-label="停下的原因（人话）">
        <p className="role-loop-plain-lead">{reason}</p>
        {plan.note ? <p className="role-loop-plain-note">{plan.note}</p> : null}
      </div>

      {/* 会话类：把选会话入口直接嵌进卡住脸——选完就能点下面[接着跑]。 */}
      {plan.primary === "session" ? (
        <JiaobanSessionPicker
          sessions={sessions}
          sessionChoice={sessionChoice}
          onSessionChoiceChange={onSessionChoiceChange}
        />
      ) : null}

      {/* 乙型（真出问题·非「已结束」/「已停下」）：直接回它一句。通道未接通时整块 disabled + 人话原因。 */}
      {isTypeB ? (
        <div className="jiaoban-blocked-reply" aria-label="直接回它一句">
          <textarea
            className="jiaoban-blocked-reply-input"
            rows={2}
            value={replyDraft}
            placeholder="直接回它：例「放在右上角 .hud 容器里，新建一个 span」——发送后按你说的继续"
            disabled={!followUpReady}
            onChange={(event) => onReplyDraftChange?.(event.target.value)}
            aria-label="回话内容"
          />
          {!followUpReady ? (
            <p className="muted small-note">回话通道还在接线，先用下面的按钮；接通后这里就能直接回它一句。</p>
          ) : null}
        </div>
      ) : null}

      <div className="workflow-state-actions">
        {isTypeB ? (
          <button
            className={typeBPrimary ? "primary-button" : "secondary-button"}
            type="button"
            disabled={!followUpReady || starting || !replyDraft.trim()}
            onClick={onSendFollowUp}
          >
            发送并继续
          </button>
        ) : null}
        {plan.primary === "continue" ? continueBtn(!typeBPrimary) : null}
        {plan.primary === "replan" ? replanBtn(!typeBPrimary) : null}
        {plan.primary === "session" ? (
          // 会话类主路径 = 上面选一条；这里的主按钮是选完[接着跑]（方案已确认时可点，否则引导重新说目标）。
          planIsConfirmed ? (
            continueBtn(true)
          ) : (
            replanBtn(true)
          )
        ) : null}
        {plan.showContinueSecondary && plan.primary !== "continue" ? continueBtn(false) : null}
        {plan.showReplanSecondary && plan.primary !== "replan" ? replanBtn(false) : null}
        {onOpenWorkflow ? (
          <button className="secondary-button" type="button" onClick={onOpenWorkflow}>
            看右侧画布
          </button>
        ) : null}
      </div>

      {/* 面级「看原始对话」：卡了想看它到底干了啥的天然下钻路（picker 行内入口不放·防一脸双入口）。 */}
      <JiaobanRawSessionLink
        sessionChoice={sessionChoice}
        latestSessionThreadId={latestSessionThreadId}
        onOpenAgentSession={onOpenAgentSession}
      />

      {/* fix3 后端新 warnings（如「角色已按 codex-dev 执行」）→ 小字列出，不挡主路径。 */}
      {warnings.length > 0 ? (
        <ul className="jiaoban-warnings muted small-note" aria-label="附带说明">
          {warnings.map((w, i) => (
            <li key={i}>{w}</li>
          ))}
        </ul>
      ) : null}

      <p className="muted small-note">卡了总给下一步，不会停在死路。</p>
    </div>
  );
}
