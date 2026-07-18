import type { ReactNode } from "react";
import type {
  BlackboardEntry,
  SupervisorResidentAnswerOutcome,
  WorkflowStateSnapshot,
} from "../../../lib/types";

// A3·视口停最新的滚动落点：挂载/切换/新消息落地都滚这个 id 到底（效果本体在 useJiaobanConversationState.ts——
// 这里若挂 useEffect，会被离线测试的裸函数调用 renderComposite 撞坏 hooks dispatcher）。
export const CONVERSATION_STREAM_ANCHOR_ID = "jiaoban-conversation-stream";

// P1-E 诚实关门（2026-07-18 用户拍板 a）：非固定测试项目的对话族入口统一给这句人话——与后端
// consultant_agent.rs::HONEST_SHUTDOWN_NON_TEST_PROJECT_MESSAGE 逐字同句，别各说各话。
export const HONEST_SHUTDOWN_NON_TEST_PROJECT_MESSAGE =
  "这个项目还没接执行——当前版本先伺候固定测试项目，开放真实项目是后面阶段的事。";

export type JiaobanConversationPhaseKind = "composer" | "conversation" | "proposal" | "delivery" | "legacy";

export type JiaobanConversationUserTurn = {
  id: string;
  text: string;
  createdAtMs: number;
};

export type JiaobanConversationArtifactNotice = {
  id: string;
  createdAtMs: number;
  kind: "proposal" | "delivery";
  copy: string;
  placement?: "after_timeline";
  onActivate: () => void;
};

// P3-A：链事件仍是黑板只读派生；用 source ref 的结构化身份区分过程短讯，绝不靠标题或 reason 猜。
// 后端的 source_id 保留 audit event id，中心只消费已确定性人话的 summary。
export const WORKFLOW_CHAIN_EVENT_SOURCE_KIND = "workflow_chain_event";

export function isSupervisorProcess(entry: BlackboardEntry): boolean {
  return (entry.source_refs ?? []).some((source) => source.source_kind === WORKFLOW_CHAIN_EVENT_SOURCE_KIND);
}

// 右区目标同样只看派生时保留的 canonical source event type；不从标题、摘要或机器 reason 反推。
export function supervisorProcessCanvasView(entry: BlackboardEntry): "delivery" | "graph" {
  return entry.source_status === "workflow_chain_run_completed" ? "delivery" : "graph";
}

export function supervisorProcessFocusedNodeId(entry: BlackboardEntry): string | null {
  return supervisorProcessCanvasView(entry) === "graph" ? entry.workflow_node_id?.trim() || null : null;
}

type ArtifactProposalSource = { proposal_id: string; created_at_ms: number };
type ArtifactDeliverySource = {
  proposal_id: string;
  created_at_ms: number;
  state: string;
};

export function artifactNoticesForConversation({
  proposals,
  history,
  currentProposalId,
  includeCurrentDelivery,
  currentProposalCreatedAtMs,
  onActivate,
}: {
  proposals: ArtifactProposalSource[];
  history: ArtifactDeliverySource[];
  currentProposalId: string | null;
  includeCurrentDelivery: boolean;
  currentProposalCreatedAtMs: number | null;
  onActivate: (kind: "proposal" | "delivery", proposalId: string) => void;
}): JiaobanConversationArtifactNotice[] {
  const proposalNotices = proposals.map((proposal) => ({
    id: conversationMessageIdForProposal(proposal.proposal_id),
    createdAtMs: proposal.created_at_ms,
    kind: "proposal" as const,
    copy: "方案好了，放你右手边了——看一眼，能跑就批。",
    onActivate: () => onActivate("proposal", proposal.proposal_id),
  }));
  const deliverySources = history
    .filter((entry) => entry.state === "delivered")
    .map((entry) => ({ proposalId: entry.proposal_id, createdAtMs: entry.created_at_ms }));
  if (
    includeCurrentDelivery &&
    currentProposalId &&
    !deliverySources.some((source) => source.proposalId === currentProposalId)
  ) {
    deliverySources.push({ proposalId: currentProposalId, createdAtMs: currentProposalCreatedAtMs ?? 0 });
  }
  const deliveryNotices = deliverySources.map((source) => ({
    id: conversationMessageIdForDelivery(source.proposalId),
    createdAtMs: source.createdAtMs,
    kind: "delivery" as const,
    copy: "干完了，结果在右边。",
    // 历史读模型没有交货完成时间；明确置于已知对话之后，避免拿方案创建时间冒充交货时刻。
    placement: "after_timeline" as const,
    onActivate: () => onActivate("delivery", source.proposalId),
  }));
  return [...proposalNotices, ...deliveryNotices];
}

type ConversationProposalSource = {
  proposal_id: string;
  user_goal: string;
  created_at_ms: number;
};

type JiaobanConversationStreamProps = {
  entries: BlackboardEntry[];
  userGoal: string | null;
  userTurns?: JiaobanConversationUserTurn[];
  artifactNotices?: JiaobanConversationArtifactNotice[];
  proposals?: ConversationOrderBoundary[];
  phaseKind: JiaobanConversationPhaseKind;
  phaseContent: ReactNode;
  phaseMessageId?: string;
  consultLoading: boolean;
  answerBusyQuestionId: string | null;
  answerReceipts: Readonly<Record<string, string>>;
  answerErrors: Readonly<Record<string, string>>;
  // P3-A：过程短讯只把用户带到右区既有工序图；回调不写事实，也不接通 P3-B 回话。
  onSupervisorProcessActivate?: (entry: BlackboardEntry) => void;
};

function createdAtSortValue(value: string | number | null | undefined): number | null {
  if (typeof value === "number") return Number.isFinite(value) ? value : null;
  if (!value?.trim()) return null;
  const numeric = Number(value);
  if (Number.isFinite(numeric)) return numeric;
  const parsed = Date.parse(value);
  return Number.isNaN(parsed) ? null : parsed;
}

export function supervisorConversationEntriesForProject(
  workflowState: WorkflowStateSnapshot | null,
  projectRoot: string,
  workflowId: string | null | undefined,
): BlackboardEntry[] {
  if (!workflowId) return [];
  const board = workflowState?.project_blackboards?.find(
    (candidate) => candidate.project_root === projectRoot && candidate.workflow_id === workflowId,
  );
  return (board?.entries ?? [])
    .filter((entry) => entry.kind === "supervisor_message")
    .map((entry, sourceIndex) => ({ entry, sourceIndex }))
    .sort((left, right) => {
      const leftTime = createdAtSortValue(left.entry.created_at);
      const rightTime = createdAtSortValue(right.entry.created_at);
      if (leftTime == null && rightTime == null) return left.sourceIndex - right.sourceIndex;
      if (leftTime == null) return -1;
      if (rightTime == null) return 1;
      return leftTime - rightTime || left.sourceIndex - right.sourceIndex;
    })
    .map(({ entry }) => entry);
}

export function conversationMessageIdForProposal(proposalId: string): string {
  return `jiaoban-message-${proposalId}`;
}

export function conversationMessageIdForDelivery(proposalId: string): string {
  return `jiaoban-delivery-${proposalId}`;
}

export function userTurnsFromProposalHistory(
  proposals: ConversationProposalSource[],
): JiaobanConversationUserTurn[] {
  return proposals.slice(1).flatMap((proposal, index) => {
    const previousGoal = proposals[index]?.user_goal.trim() ?? "";
    const nextGoal = proposal.user_goal.trim();
    if (!nextGoal || nextGoal === previousGoal) return [];
    const amendmentPrefix = `${previousGoal}\n\n补充意见：`;
    return [{
      id: `jiaoban-user-turn-${proposal.proposal_id}`,
      text: nextGoal.startsWith(amendmentPrefix)
        ? nextGoal.slice(amendmentPrefix.length).trim()
        : nextGoal,
      createdAtMs: proposal.created_at_ms,
    }];
  });
}

export function mergeConversationUserTurns(
  baseGoal: string | null,
  persisted: JiaobanConversationUserTurn[],
  transient: JiaobanConversationUserTurn[],
): JiaobanConversationUserTurn[] {
  const persistedCounts = new Map<string, number>();
  for (const turn of persisted) {
    persistedCounts.set(turn.text, (persistedCounts.get(turn.text) ?? 0) + 1);
  }
  let skippedBaseTurn = false;
  const unmatchedTransient = transient.filter((turn) => {
    if (!skippedBaseTurn && turn.text === baseGoal) {
      skippedBaseTurn = true;
      return false;
    }
    const persistedCount = persistedCounts.get(turn.text) ?? 0;
    if (persistedCount === 0) return true;
    persistedCounts.set(turn.text, persistedCount - 1);
    return false;
  });
  return [...persisted, ...unmatchedTransient];
}

export function humanizeResidentAnswerOutcome(outcome: SupervisorResidentAnswerOutcome): string {
  if (outcome.status === "already_answered") {
    return "这问已经答过了，主管没有重复处理。";
  }
  if (outcome.status === "question_asked") {
    return "这句主管收到了；它还有一问，接着答就行。";
  }
  if (outcome.status === "proposal_created") {
    return "这句主管收到了，新方案已经接上。";
  }
  return "这句主管收到了。";
}

function isSupervisorQuestion(entry: BlackboardEntry): boolean {
  return entry.title.startsWith("主管问题");
}

export function latestWaitingQuestionIdOf(entries: BlackboardEntry[]): string | null {
  return (
    [...entries]
      .reverse()
      .find(
        (entry) =>
          isSupervisorQuestion(entry) &&
          (entry.source_status ?? entry.status) === "waiting_user" &&
          Boolean(entry.question_id?.trim()),
      )?.question_id ?? null
  );
}

// 常驻输入框的路由：对话里的话按当前状态送进既有通道，零新命令（修单3）。
export type JiaobanComposerRoute =
  | { kind: "answer"; questionId: string; placeholder?: string }
  | { kind: "amendment"; placeholder?: string }
  | { kind: "new_goal"; placeholder?: string }
  | { kind: "disabled"; reason: string };

const COMPOSER_PLACEHOLDERS: Record<Exclude<JiaobanComposerRoute["kind"], "disabled">, string> = {
  answer: "直接回答主管这一问",
  amendment: "想改哪里直接说——主管会按你说的重出方案",
  new_goal: "跟主管说下一件事",
};

// 「只要一个对话框,上下不带字」(07-18 用户拍):无标题无按钮,Enter 发送、Shift+Enter 换行;
// 错误是异常态不是装饰,以 alert 行上脸(fix8 绝不静默死)。
export function JiaobanConversationComposer({
  route,
  draft,
  busy,
  error = null,
  onDraftChange,
  onSubmit,
}: {
  route: JiaobanComposerRoute;
  draft: string;
  busy: boolean;
  error?: string | null;
  onDraftChange: (value: string) => void;
  onSubmit: () => void;
}) {
  const disabled = route.kind === "disabled" || busy;
  const canSend = !disabled && Boolean(draft.trim());
  return (
    <div className="jiaoban-conversation-composer" data-composer-route={route.kind}>
      {error ? (
        <p className="jiaoban-consult-error" role="alert">
          {error}
        </p>
      ) : null}
      <textarea
        aria-label="跟项目主管说话"
        value={draft}
        placeholder={
          route.kind === "disabled"
            ? route.reason
            : route.placeholder ?? COMPOSER_PLACEHOLDERS[route.kind]
        }
        disabled={disabled}
        onChange={(event) => onDraftChange(event.target.value)}
        onKeyDown={(event) => {
          if (event.key === "Enter" && !event.shiftKey) {
            event.preventDefault();
            if (canSend) onSubmit();
          }
        }}
      />
    </div>
  );
}

function messageText(entry: BlackboardEntry): string {
  if (isSupervisorProcess(entry)) return entry.summary.trim();
  if (isSupervisorQuestion(entry)) {
    return entry.summary.replace(/^第\s*\d+\s*轮主管问题[：:]\s*/, "").trim();
  }
  return entry.summary.replace(/^第\s*\d+\s*轮用户答复[：:]\s*/, "").trim();
}

function phaseMessageLabel(kind: JiaobanConversationPhaseKind): string | null {
  if (kind === "proposal") return "项目主管 · 方案";
  if (kind === "delivery") return "项目主管 · 交货";
  return null;
}

// A4·分组判据(修单4)：某单的对话消息 = 该 proposal 创建至下一 proposal 创建之间的条目；黑板 entries
// 按 created_at 归组，undated 条目兜底进当前单（不静默吞消息）。只有 2+ 单才谈得上「更早」——
// 0/1 单时原样不分组，与折单前的排版逐字节一致。纯函数：零 DOM、零副作用，供离线断言直接摆数据验证。
export type ConversationOrderBoundary = { proposal_id: string; created_at_ms: number };
export type ConversationOrderGroup<T> = { proposalId: string; items: T[] };

export function groupConversationItemsByProposal<T extends { createdAtMs: number | null }>(
  items: T[],
  boundaries: ConversationOrderBoundary[],
): { earlierGroups: ConversationOrderGroup<T>[]; currentItems: T[] } {
  if (boundaries.length <= 1) return { earlierGroups: [], currentItems: items };
  const sorted = [...boundaries].sort((left, right) => left.created_at_ms - right.created_at_ms);
  const buckets: T[][] = sorted.map(() => []);
  for (const item of items) {
    let index = 0;
    for (let i = 0; i < sorted.length; i += 1) {
      if (item.createdAtMs == null || item.createdAtMs >= sorted[i].created_at_ms) index = i;
      else break;
    }
    buckets[index].push(item);
  }
  return {
    earlierGroups: sorted
      .slice(0, -1)
      .map((boundary, index) => ({ proposalId: boundary.proposal_id, items: buckets[index] })),
    currentItems: buckets[buckets.length - 1],
  };
}

export type ConversationTimelineItem = {
  createdAtMs: number | null;
  afterTimeline: boolean;
  tieRank: number;
  sourceIndex: number;
  content: ReactNode;
};

function sortConversationTimelineItems(items: ConversationTimelineItem[]): ConversationTimelineItem[] {
  return [...items].sort((left, right) => {
    if (left.afterTimeline !== right.afterTimeline) return left.afterTimeline ? 1 : -1;
    if (left.createdAtMs == null && right.createdAtMs == null) {
      return left.sourceIndex - right.sourceIndex;
    }
    if (left.createdAtMs == null) return -1;
    if (right.createdAtMs == null) return 1;
    return (
      left.createdAtMs - right.createdAtMs ||
      left.tieRank - right.tieRank ||
      left.sourceIndex - right.sourceIndex
    );
  });
}

export function JiaobanConversationStream({
  entries,
  userGoal,
  userTurns = [],
  artifactNotices = [],
  proposals = [],
  phaseKind,
  phaseContent,
  phaseMessageId,
  consultLoading,
  answerBusyQuestionId,
  answerReceipts,
  answerErrors,
  onSupervisorProcessActivate,
}: JiaobanConversationStreamProps) {
  const latestWaitingQuestionId = latestWaitingQuestionIdOf(entries);
  // 待答追问独占当前动作区：不仅收起说态输入，也隔离仍在 store 里的旧方案授权动作。
  // 回答并刷新后，新的相位消息才重新出现，避免绕过主管追问直接执行旧方案。
  const pendingQuestionOwnsActionArea = latestWaitingQuestionId != null;
  const supervisorThinking = consultLoading || answerBusyQuestionId != null;
  const showPhaseContent = phaseContent != null && !supervisorThinking && !pendingQuestionOwnsActionArea;
  const phaseLabel = phaseMessageLabel(phaseKind);
  const timelineItems: ConversationTimelineItem[] = [];

  for (const [sourceIndex, entry] of entries.entries()) {
    const process = isSupervisorProcess(entry);
    const question = isSupervisorQuestion(entry);
    const status = entry.source_status ?? entry.status;
    const answered = question && status === "answered";
    const questionId = entry.question_id?.trim() || null;
    const waitingForThisAnswer = questionId != null && questionId === latestWaitingQuestionId;
    const receipt = question && questionId ? answerReceipts[questionId] : null;
    const error = question && questionId ? answerErrors[questionId] : null;
    const copy = messageText(entry);
    const content = process ? (
      <article
        key={entry.entry_id}
        className="jiaoban-conversation-message is-supervisor jiaoban-conversation-process"
        data-message-kind="supervisor-process"
      >
        <p className="jiaoban-plan-seg">项目主管</p>
        {onSupervisorProcessActivate ? (
          <button
            className="jiaoban-conversation-process-action"
            type="button"
            aria-label={
              supervisorProcessCanvasView(entry) === "delivery"
                ? "打开右侧交货"
                : supervisorProcessFocusedNodeId(entry)
                  ? "打开右侧工序图并定位任务"
                  : "打开右侧工序图"
            }
            onClick={() => onSupervisorProcessActivate(entry)}
          >
            <span className="jiaoban-conversation-copy">{copy}</span>
          </button>
        ) : (
          <p className="jiaoban-conversation-copy">{copy}</p>
        )}
      </article>
    ) : answered ? (
      <article
        key={entry.entry_id}
        className="jiaoban-conversation-message is-supervisor is-answered"
        data-message-kind="supervisor-question"
      >
        <details className="jiaoban-conversation-answered">
          <summary>项目主管 · 已答</summary>
          <p className="jiaoban-conversation-copy">{copy}</p>
        </details>
        {receipt ? (
          <p className="muted small-note" role="status">
            {receipt}
          </p>
        ) : null}
      </article>
    ) : (
      <article
        key={entry.entry_id}
        className={`jiaoban-conversation-message ${question ? "is-supervisor" : "is-user"}`}
        data-message-kind={question ? "supervisor-question" : "user-answer"}
      >
        <p className="jiaoban-plan-seg">
          {question ? `项目主管${waitingForThisAnswer ? " · 待答" : ""}` : "你"}
        </p>
        <p className="jiaoban-conversation-copy">{copy}</p>
        {receipt ? (
          <p className="muted small-note" role="status">
            {receipt}
          </p>
        ) : null}
        {error ? (
          <p className="jiaoban-consult-error" role="alert">
            {error}
          </p>
        ) : null}
      </article>
    );
    timelineItems.push({
      createdAtMs: createdAtSortValue(entry.created_at),
      afterTimeline: false,
      tieRank: 1,
      sourceIndex,
      content,
    });
  }

  for (const [turnIndex, turn] of userTurns.entries()) {
    timelineItems.push({
      createdAtMs: createdAtSortValue(turn.createdAtMs),
      afterTimeline: false,
      tieRank: 0,
      sourceIndex: entries.length + turnIndex,
      content: (
        <article key={turn.id} className="jiaoban-conversation-message is-user" data-message-kind="user">
          <p className="jiaoban-plan-seg">你</p>
          <p className="jiaoban-conversation-copy">{turn.text}</p>
        </article>
      ),
    });
  }

  for (const [noticeIndex, notice] of artifactNotices.entries()) {
    timelineItems.push({
      createdAtMs: createdAtSortValue(notice.createdAtMs),
      afterTimeline: notice.placement === "after_timeline",
      tieRank: notice.kind === "proposal" ? 2 : 3,
      sourceIndex: entries.length + userTurns.length + noticeIndex,
      content: (
        <article
          key={notice.id}
          id={notice.id}
          className="jiaoban-conversation-message is-supervisor jiaoban-conversation-notice"
          data-message-kind={notice.kind}
        >
          <p className="jiaoban-plan-seg">项目主管</p>
          <button
            className="jiaoban-conversation-notice-action"
            type="button"
            aria-label={notice.kind === "proposal" ? "打开右侧方案" : "打开右侧交货"}
            onClick={notice.onActivate}
          >
            <span className="jiaoban-conversation-copy">{notice.copy}</span>
          </button>
        </article>
      ),
    });
  }

  // 用户目标一句也纳入同一套排序/分组管线：0/1 单时 createdAtMs 为 null，比较器把它排在最前，
  // 与折单前「userGoal 恒居首」逐字节一致；2+ 单时它落进首单边界，成为首单折叠段的「用户目标一句」。
  const boundaries = [...proposals].sort((left, right) => left.created_at_ms - right.created_at_ms);
  const userGoalTrimmed = userGoal?.trim() || null;
  if (userGoalTrimmed) {
    timelineItems.push({
      createdAtMs: boundaries[0]?.created_at_ms ?? null,
      afterTimeline: false,
      tieRank: -1,
      sourceIndex: -1,
      content: (
        <article key="jiaoban-user-goal" className="jiaoban-conversation-message is-user" data-message-kind="user">
          <p className="jiaoban-plan-seg">你</p>
          <p className="jiaoban-conversation-copy">{userGoalTrimmed}</p>
        </article>
      ),
    });
  }

  const { earlierGroups, currentItems } = groupConversationItemsByProposal(timelineItems, boundaries);
  const sortedCurrentItems = sortConversationTimelineItems(currentItems);
  const sortedEarlierGroups = earlierGroups.map((group) => ({
    proposalId: group.proposalId,
    items: sortConversationTimelineItems(group.items),
  }));

  return (
    <section
      id={CONVERSATION_STREAM_ANCHOR_ID}
      className="jiaoban-conversation-stream"
      aria-label="项目对话消息流"
    >
      {sortedEarlierGroups.length > 0 ? (
        <details className="jiaoban-conversation-earlier">
          <summary>更早的 {sortedEarlierGroups.length} 单对话</summary>
          {sortedEarlierGroups.map((group) => (
            <div
              key={group.proposalId}
              className="jiaoban-conversation-earlier-order"
              data-proposal-id={group.proposalId}
            >
              {group.items.map((item) => item.content)}
            </div>
          ))}
        </details>
      ) : null}

      {sortedCurrentItems.map((item) => item.content)}

      {supervisorThinking ? (
        <article className="jiaoban-conversation-message is-supervisor" data-message-kind="waiting">
          <div className="jiaoban-conversation-waiting" role="status" aria-label="主管在看">
            <span className="jiaoban-spinner" aria-hidden="true" />
            <span>主管在看</span>
          </div>
        </article>
      ) : null}

      {showPhaseContent ? (
        <article
          id={phaseMessageId}
          className="jiaoban-conversation-message jiaoban-conversation-phase"
          data-message-kind={phaseKind}
        >
          {phaseLabel ? <p className="jiaoban-plan-seg">{phaseLabel}</p> : null}
          {phaseContent}
        </article>
      ) : null}
    </section>
  );
}
