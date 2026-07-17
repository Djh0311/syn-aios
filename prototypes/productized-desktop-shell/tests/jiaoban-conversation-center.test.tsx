import { renderToStaticMarkup } from "react-dom/server.browser";
import type {
  BlackboardEntry,
  RunHistoryEntry,
  SupervisorResidentAnswerOutcome,
  WorkflowStateSnapshot,
} from "../src/lib/types";
import { humanizeConsultError } from "../src/views/projects/ProjectJiaobanPanel";
import {
  JiaobanConversationComposer,
  JiaobanConversationStream,
  artifactNoticesForConversation,
  groupConversationItemsByProposal,
  humanizeResidentAnswerOutcome,
  latestWaitingQuestionIdOf,
  userTurnsFromProposalHistory,
  supervisorConversationEntriesForProject,
} from "../src/views/projects/jiaoban/JiaobanConversation";
import { JiaobanHistoryColumn } from "../src/views/projects/jiaoban/JiaobanHistory";
import {
  assert,
  assertDeepEqual,
  findButtonByText,
  findElement,
  visibleText,
} from "./helpers/offlineInteractionTestUtils";

const projectRoot = "/Users/yoyi/codex-workflow-mario-test";
const projectId = "project:conversation-center";
const workflowId = "workflow:conversation-center";
const answeredQuestionId = "question:resident:001";
const waitingQuestionId = "question:resident:002";
const noop = () => {};

function supervisorEntry(overrides: Partial<BlackboardEntry>): BlackboardEntry {
  return {
    entry_id: "blackboard:supervisor:fixture",
    project_id: projectId,
    workflow_id: workflowId,
    work_item_id: null,
    workflow_node_id: null,
    question_id: waitingQuestionId,
    kind: "supervisor_message",
    title: "主管问题 · 第 2 轮",
    summary: "第 2 轮主管问题：这单只读吗？",
    status: "waiting_user",
    source_status: "waiting_user",
    source_refs: [
      {
        source_kind: "supervisor_resident_question",
        source_id: waitingQuestionId,
        label: "主管追问",
      },
    ],
    created_at: "2026-07-17T02:00:00.000Z",
    promotion_decision: {
      decision_id: "promotion:supervisor:fixture",
      status: "canonical",
      target_kind: "supervisor_message",
      decided_by_role: "supervisor",
      decided_at: "2026-07-17T02:00:00.000Z",
      reason: "resident 主管消息读模型夹具。",
      audit_refs: [],
      warnings: [],
    },
    warnings: [],
    ...overrides,
  };
}

function stream(overrides: Partial<Parameters<typeof JiaobanConversationStream>[0]> = {}) {
  return (
    <JiaobanConversationStream
      entries={[]}
      userGoal={null}
      phaseKind="legacy"
      phaseContent={<p>旧七态内容照常在</p>}
      consultLoading={false}
      answerBusyQuestionId={null}
      answerReceipts={{}}
      answerErrors={{}}
      {...overrides}
    />
  );
}

const answeredQuestion = supervisorEntry({
  entry_id: "blackboard:supervisor:answered",
  question_id: answeredQuestionId,
  title: "主管问题 · 第 1 轮",
  summary: "第 1 轮主管问题：先确认验收是否只看离线结果？",
  status: "answered",
  source_status: "answered",
  created_at: "2026-07-17T01:00:00.000Z",
});
const waitingQuestion = supervisorEntry({
  entry_id: "blackboard:supervisor:waiting",
});
const userAnswer = supervisorEntry({
  entry_id: "blackboard:supervisor:user-answer",
  question_id: answeredQuestionId,
  title: "用户答复 · 第 1 轮",
  summary: "第 1 轮用户答复：只看离线结果，真机由用户重启后验。",
  status: "answered",
  source_status: "answered",
  created_at: "2026-07-17T03:00:00.000Z",
});
const unrelatedRisk = supervisorEntry({
  entry_id: "blackboard:risk:unrelated",
  question_id: null,
  kind: "risk",
  title: "无关风险",
  summary: "不应进入主管对话消息流。",
  status: "candidate",
  source_status: "pending",
  created_at: "2026-07-17T00:00:00.000Z",
});

const workflowState = {
  project_blackboards: [
    {
      project_id: projectId,
      project_root: projectRoot,
      workflow_id: workflowId,
      entries: [userAnswer, waitingQuestion, unrelatedRisk, answeredQuestion],
      warnings: [],
    },
  ],
} as unknown as WorkflowStateSnapshot;

const chronologicalEntries = supervisorConversationEntriesForProject(workflowState, projectRoot, workflowId);
assertDeepEqual(
  chronologicalEntries.map((entry) => entry.entry_id),
  [answeredQuestion.entry_id, waitingQuestion.entry_id, userAnswer.entry_id],
  "主管消息应按 created_at 正序，且只取当前项目工作流的 supervisor_message",
);

assertDeepEqual(
  userTurnsFromProposalHistory([
    { proposal_id: "proposal:old", user_goal: "旧目标", created_at_ms: 1 },
    {
      proposal_id: "proposal:amended",
      user_goal: "旧目标\n\n补充意见：右区零改。",
      created_at_ms: 2,
    },
    {
      proposal_id: "proposal:answer-only",
      user_goal: "旧目标\n\n补充意见：右区零改。",
      created_at_ms: 3,
    },
  ]).map((turn) => turn.text),
  ["右区零改。"],
  "重挂载后应从方案历史恢复用户真实补充原话，且同目标的主管回答续跑不应伪造新口供",
);

// 1) 中栏只留对话；方案 / 交货以主管短讯挂右区焦点，实体卡不得回流进消息流。
{
  const focusedViews: string[] = [];
  const proposalArtifactNotices = artifactNoticesForConversation({
    proposals: [{ proposal_id: "proposal-fixture", created_at_ms: Date.parse("2026-07-17T04:00:00.000Z") }],
    history: [],
    currentProposalId: "proposal-fixture",
    includeCurrentDelivery: false,
    currentProposalCreatedAtMs: Date.parse("2026-07-17T04:00:00.000Z"),
    onActivate: (kind) => focusedViews.push(kind),
  });
  const proposalTree = stream({
    entries: [answeredQuestion, userAnswer],
    userGoal: "把中栏改成项目对话消息流。",
    phaseKind: "proposal",
    phaseContent: null,
    artifactNotices: proposalArtifactNotices,
  });
  const proposalMarkup = renderToStaticMarkup(proposalTree);

  assert(proposalMarkup.includes('data-message-kind="proposal"'), "方案生成后应留下 proposal 主管短讯锚点");
  assert(
    proposalMarkup.includes("方案好了，放你右手边了——看一眼，能跑就批。"),
    "方案短讯应把实体卡去向说成人话",
  );
  assert(
    !proposalMarkup.includes("方案卡正文") && !proposalMarkup.includes("项目主管 · 方案"),
    "方案实体卡与旧卡头不得回流进中栏",
  );
  assert(
    (proposalMarkup.match(/data-message-kind="supervisor-question"/g) ?? []).length === 1,
    "主管追问应使用 supervisor-question 消息包装",
  );
  const userTurn = findElement(
    proposalTree,
    (element) =>
      element.props?.["data-message-kind"] === "user" &&
      typeof element.props?.className === "string" &&
      element.props.className.includes("is-user"),
  );
  const supervisorTurn = findElement(
    proposalTree,
    (element) =>
      element.props?.["data-message-kind"] === "supervisor-question" &&
      typeof element.props?.className === "string" &&
      element.props.className.includes("is-supervisor"),
  );
  const proposalNotice = findElement(
    proposalTree,
    (element) =>
      element.props?.["data-message-kind"] === "proposal" &&
      typeof element.props?.className === "string" &&
      element.props.className.includes("is-supervisor") &&
      element.props.className.includes("jiaoban-conversation-notice"),
  );
  assert(userTurn && supervisorTurn, "用户回合与主管回合应保留可独立排版的角色挂点");
  assert(proposalNotice, "方案短讯应使用主管角色形态并保留锚点");
  const proposalFocusButton = findElement(
    proposalTree,
    (element) => element.type === "button" && element.props?.["aria-label"] === "打开右侧方案",
  );
  assert(proposalFocusButton, "方案短讯应可聚焦右侧方案视图");
  const focusProposal = proposalFocusButton.props?.onClick as (() => void) | undefined;
  assert(focusProposal, "方案短讯应保留可执行的右区聚焦回调");
  focusProposal();
  assertDeepEqual(focusedViews, ["proposal"], "方案短讯点击应只触发右区方案聚焦回调");

  const userGoalIndex = proposalMarkup.indexOf("把中栏改成项目对话消息流");
  const firstQuestionIndex = proposalMarkup.indexOf("先确认验收是否只看离线结果");
  const userAnswerIndex = proposalMarkup.indexOf("只看离线结果，真机由用户重启后验");
  const proposalIndex = proposalMarkup.indexOf("方案好了，放你右手边了");
  assert(
    userGoalIndex < firstQuestionIndex &&
      firstQuestionIndex < userAnswerIndex &&
      userAnswerIndex < proposalIndex,
    "用户原话、主管往返与方案短讯应按消息时间序落位",
  );

  const deliveryTree = stream({
    phaseKind: "delivery",
    phaseContent: null,
    artifactNotices: artifactNoticesForConversation({
      proposals: [{ proposal_id: "proposal-before-delivery", created_at_ms: Date.parse("2026-07-17T04:00:00.000Z") }],
      // 历史条目的时间是方案创建时间，刻意早于方案夹具；交货短讯仍必须置于已知对话之后。
      history: [{ proposal_id: "proposal-before-delivery", created_at_ms: Date.parse("2026-07-17T00:30:00.000Z"), state: "delivered" }],
      currentProposalId: "proposal-before-delivery",
      includeCurrentDelivery: false,
      currentProposalCreatedAtMs: Date.parse("2026-07-17T04:00:00.000Z"),
      onActivate: (kind) => focusedViews.push(kind),
    }),
  });
  const deliveryMarkup = renderToStaticMarkup(deliveryTree);
  assert(deliveryMarkup.includes('data-message-kind="delivery"'), "交货完成后应留下 delivery 主管短讯锚点");
  assert(
    deliveryMarkup.includes("干完了，结果在右边。") && !deliveryMarkup.includes("交货卡正文"),
    "交货实体卡不得回流进中栏，只保留人话短讯",
  );
  const deliveryNotice = findElement(
    deliveryTree,
    (element) =>
      element.props?.["data-message-kind"] === "delivery" &&
      typeof element.props?.className === "string" &&
      element.props.className.includes("is-supervisor") &&
      element.props.className.includes("jiaoban-conversation-notice"),
  );
  assert(deliveryNotice, "交货短讯应使用主管角色形态并保留锚点");
  const deliveryFocusButton = findElement(
    deliveryTree,
    (element) => element.type === "button" && element.props?.["aria-label"] === "打开右侧交货",
  );
  assert(deliveryFocusButton, "交货短讯应可聚焦右侧交货视图");
  const focusDelivery = deliveryFocusButton.props?.onClick as (() => void) | undefined;
  assert(focusDelivery, "交货短讯应保留可执行的右区聚焦回调");
  focusDelivery();
  assertDeepEqual(
    focusedViews,
    ["proposal", "delivery"],
    "交货短讯点击应只触发右区交货聚焦回调",
  );
  assert(
    deliveryMarkup.indexOf("方案好了，放你右手边了") < deliveryMarkup.indexOf("干完了，结果在右边"),
    "相位到交货后，方案与交货主管短讯应按时间序保留",
  );

  const deliveredHistoryEntry: RunHistoryEntry = {
    proposal_id: "proposal-before-delivery",
    workflow_id: workflowId,
    goal_text: "交付 P1-C",
    created_at_ms: Date.parse("2026-07-17T05:00:00.000Z"),
    state: "delivered",
    state_note: "已交货",
    advice_only: false,
    review_flags: {},
    correlation: "exact",
  };
  let selectedDeliveredCount = 0;
  let backToCurrentCount = 0;
  const deliveredHistoryRow = findElement(
    <JiaobanHistoryColumn
      entries={[deliveredHistoryEntry]}
      total={1}
      loading={false}
      filter="all"
      onFilterChange={noop}
      selectedId={null}
      currentProposalId={deliveredHistoryEntry.proposal_id}
      latestBlockedId={null}
      onSelectEntry={() => { selectedDeliveredCount += 1; }}
      onBackToCurrent={() => { backToCurrentCount += 1; }}
      onNewJiaoban={noop}
      onContinueRun={noop}
    />,
    (element) => element.props?.["aria-controls"] === `jiaoban-delivery-${deliveredHistoryEntry.proposal_id}`,
  );
  assert(deliveredHistoryRow, "已交货单行应声明交货短讯锚点");
  const clickDeliveredHistory = deliveredHistoryRow.props?.onClick;
  assert(typeof clickDeliveredHistory === "function", "已交货单行应保留可执行的锚点点击");
  clickDeliveredHistory();
  assert(
    selectedDeliveredCount === 1 && backToCurrentCount === 0,
    "即使是当前单，已交货行也必须走交货消息选择回调，不能错跳方案卡",
  );
  const continuedMarkup = renderToStaticMarkup(
    stream({
      phaseKind: "composer",
      phaseContent: <section>继续弄别的输入区</section>,
      artifactNotices: [
        {
          id: "jiaoban-message-proposal-before-delivery",
          createdAtMs: Date.parse("2026-07-17T04:00:00.000Z"),
          kind: "proposal",
          copy: "方案好了，放你右手边了——看一眼，能跑就批。",
          onActivate: noop,
        },
        {
          id: "jiaoban-delivery-proposal-before-delivery",
          createdAtMs: Date.parse("2026-07-17T05:00:00.000Z"),
          kind: "delivery",
          copy: "干完了，结果在右边。",
          placement: "after_timeline",
          onActivate: noop,
        },
      ],
    }),
  );
  assert(
    continuedMarkup.indexOf("方案好了，放你右手边了") < continuedMarkup.indexOf("干完了，结果在右边") &&
      continuedMarkup.indexOf("干完了，结果在右边") < continuedMarkup.indexOf("继续弄别的输入区"),
    "done→继续弄别的后，方案与交货短讯仍应按序留在说态输入区之前",
  );

  const amendmentMarkup = renderToStaticMarkup(
    stream({
      entries: [waitingQuestion],
      userGoal: "旧目标",
      userTurns: [
        {
          id: "user-turn-amendment",
          text: "右区零改。",
          createdAtMs: Date.parse("2026-07-17T01:30:00.000Z"),
        },
      ],
      artifactNotices: [
        {
          id: "jiaoban-message-old-proposal",
          createdAtMs: Date.parse("2026-07-17T01:00:00.000Z"),
          kind: "proposal",
          copy: "方案好了，放你右手边了——看一眼，能跑就批。",
          onActivate: noop,
        },
      ],
      phaseContent: null,
    }),
  );
  assert(
    amendmentMarkup.indexOf("方案好了，放你右手边了") < amendmentMarkup.indexOf("右区零改。") &&
      amendmentMarkup.indexOf("右区零改。") < amendmentMarkup.indexOf("这单只读吗"),
    "改要求后的真实口供应落在旧方案之后、主管新追问之前",
  );
}

// 2) 常驻输入框(修单3)：answer 路由受控输入+提交；路由判据只认结构化 question_id；跑态禁发人话；
//    消息流内不再内嵌输入框——输入统一走常驻框。
{
  assert(
    latestWaitingQuestionIdOf(chronologicalEntries) === waitingQuestionId,
    "常驻框 answer 路由判据必须来自结构化 entry.question_id",
  );
  assert(latestWaitingQuestionIdOf([answeredQuestion]) === null, "没有 waiting_user 时不应给出 answer 路由");
  assert(waitingQuestionId !== waitingQuestion.entry_id, "question_id 不得退化为 blackboard entry_id");

  const draftChanges: string[] = [];
  let submitted: number = 0;
  const composer = (
    <JiaobanConversationComposer
      route={{ kind: "answer", questionId: waitingQuestionId }}
      draft="保持只读。"
      busy={false}
      onDraftChange={(value) => draftChanges.push(value)}
      onSubmit={() => {
        submitted += 1;
      }}
    />
  );
  const textarea = findElement(
    composer,
    (element) => element.type === "textarea" && element.props?.["aria-label"] === "跟项目主管说话",
  );
  assert(textarea, "常驻输入框应挂在对话底部");
  assert(textarea.props?.value === "保持只读。", "常驻输入框应为受控草稿");
  const onChange = textarea.props?.onChange as ((event: { target: { value: string } }) => void) | undefined;
  assert(onChange, "常驻输入框应接受受控变更回调");
  onChange({ target: { value: "只读，不写项目根。" } });

  // 「只要一个对话框,上下不带字」:无按钮,Enter 发送、Shift+Enter 换行。
  assert(!findButtonByText(composer, "说给主管"), "常驻输入框不再挂发送按钮");
  const onKeyDown = textarea.props?.onKeyDown as
    | ((event: { key: string; shiftKey: boolean; preventDefault: () => void }) => void)
    | undefined;
  assert(onKeyDown, "常驻输入框应接 Enter 发送");
  onKeyDown({ key: "Enter", shiftKey: true, preventDefault: noop });
  const submittedAfterShiftEnter = submitted;
  assert(submittedAfterShiftEnter === 0, "Shift+Enter 只换行不发送");
  onKeyDown({ key: "Enter", shiftKey: false, preventDefault: noop });
  const submittedAfterEnter = submitted;

  assertDeepEqual(draftChanges, ["只读，不写项目根。"], "草稿变更应回传输入值");
  assert(submittedAfterEnter === 1, "Enter 应触发一次提交");

  const composerErrorMarkup = renderToStaticMarkup(
    <JiaobanConversationComposer
      route={{ kind: "new_goal" }}
      draft=""
      busy={false}
      error="这句没送到主管——稍后再试一次。"
      onDraftChange={noop}
      onSubmit={noop}
    />,
  );
  assert(
    composerErrorMarkup.includes('role="alert"') && composerErrorMarkup.includes("这句没送到主管"),
    "出方案失败必须以 alert 行上脸,不许静默",
  );

  const disabledMarkup = renderToStaticMarkup(
    <JiaobanConversationComposer
      route={{ kind: "disabled", reason: "正在干活——有话等交货或卡住时说" }}
      draft=""
      busy={false}
      onDraftChange={noop}
      onSubmit={noop}
    />,
  );
  assert(disabledMarkup.includes("正在干活——有话等交货或卡住时说"), "跑态应以人话说明禁发");
  assert(disabledMarkup.includes("disabled"), "跑态输入框应禁用");

  const streamMarkup = renderToStaticMarkup(stream({ entries: chronologicalEntries }));
  assert(!streamMarkup.includes("<textarea"), "消息流内不再内嵌输入框——输入统一走常驻框");

  const pendingWithOldProposal = renderToStaticMarkup(
    stream({
      entries: [waitingQuestion],
      phaseKind: "proposal",
      phaseContent: <button type="button">允许并开始旧方案</button>,
    }),
  );
  assert(
    !pendingWithOldProposal.includes("允许并开始旧方案"),
    "待答追问必须独占当前动作区，不能露出仍在 store 里的旧方案授权动作",
  );
}

// 3) answered 追问收成默认闭合摘要，旧七态内容仍并存，且不再挂回答框。
{
  const tree = stream({ entries: [answeredQuestion] });
  const markup = renderToStaticMarkup(tree);
  const details = findElement(
    tree,
    (element) => element.type === "details" && element.props?.className === "jiaoban-conversation-answered",
  );
  assert(details, "answered 追问应使用折叠详情");
  assert(details.props?.open !== true, "answered 追问默认应闭合");
  const detailsText = visibleText(details);
  assert(
    detailsText.includes("项目主管 · 已答") && detailsText.includes("先确认验收是否只看离线结果？"),
    "折叠摘要应说明已答并保留问题",
  );
  assert(!markup.includes('aria-label="回答主管"') && !markup.includes(">答主管<"), "answered 追问不应再挂输入框");
  assert(markup.includes("旧七态内容照常在"), "消息流过渡期不得拆掉旧七态内容");
}

// 4) already_answered 只显示人话回执，不把机器状态或后端注入细节带上脸。
{
  const outcome: SupervisorResidentAnswerOutcome = {
    status: "already_answered",
    question_id: answeredQuestionId,
    reply_injected: true,
    thread_id: "thread:resident:fixture",
    supervisor_reply: null,
    proposal: null,
    question: null,
    message: "该问题已有答复且已注入；拒绝重复提交，未再次调用模型。原答复长度=8。",
  };
  const receipt = humanizeResidentAnswerOutcome(outcome);
  const markup = renderToStaticMarkup(
    stream({
      entries: [answeredQuestion],
      answerReceipts: { [answeredQuestionId]: receipt },
    }),
  );
  assert(markup.includes("这问已经答过了，主管没有重复处理。"), "幂等命中应显示可理解的人话回执");
  for (const forbidden of ["already_answered", "已注入", "调用模型", "答复长度"]) {
    assert(!markup.includes(forbidden), `幂等回执不得上脸机器词：${forbidden}`);
  }
}

// 5) consulting 是带人话的「主管在看」等待消息，不允许裸 spinner；新 waiting_user 前缀也必须翻成人话。
{
  const tree = stream({
    consultLoading: true,
    phaseKind: "composer",
    phaseContent: <p>旧说态输入框</p>,
  });
  const markup = renderToStaticMarkup(tree);
  const waitingStatus = findElement(
    tree,
    (element) => element.props?.role === "status" && element.props?.["aria-label"] === "主管在看",
  );
  assert(waitingStatus, "consulting 应有可访问的等待状态");
  assert(visibleText(waitingStatus).includes("主管在看"), "等待态必须带主管在看人话，不能只有 spinner");
  assert(markup.includes('class="jiaoban-spinner" aria-hidden="true"'), "呼吸点只作装饰并应对读屏隐藏");
  assert(!markup.includes("旧说态输入框"), "主管在看时不应同时露出旧说态输入框");

  const answerRoundMarkup = renderToStaticMarkup(
    stream({
      entries: [waitingQuestion],
      answerBusyQuestionId: waitingQuestionId,
    }),
  );
  assert(answerRoundMarkup.includes("主管在看"), "回答送达后的 resident 新回合也必须进入主管在看等待态");

  const humanized = humanizeConsultError(
    new Error(`supervisor_resident_question_waiting_user:${waitingQuestionId}`),
  );
  assert(humanized === "主管想先问清一件事，请在下面直接回答。", "waiting_user 新前缀应翻成既定人话");
  assert(
    !humanized.includes("supervisor_resident_question_waiting_user") && !humanized.includes(waitingQuestionId),
    "humanizeConsultError 不得泄露机器前缀或 question_id",
  );
}

// 6) 修单4·A4 分组判据：groupConversationItemsByProposal 是纯函数——纯数据摆样例验证，零 DOM。
{
  type Fixture = { id: string; createdAtMs: number | null };
  const items: Fixture[] = [
    { id: "before-first", createdAtMs: 50 },
    { id: "first-order", createdAtMs: 120 },
    { id: "second-order", createdAtMs: 220 },
    { id: "undated", createdAtMs: null },
    { id: "current-order", createdAtMs: 320 },
  ];
  const boundaries = [
    { proposal_id: "p1", created_at_ms: 100 },
    { proposal_id: "p2", created_at_ms: 200 },
    { proposal_id: "p3", created_at_ms: 300 },
  ];
  const grouped = groupConversationItemsByProposal(items, boundaries);
  assertDeepEqual(
    grouped.earlierGroups.map((group) => group.proposalId),
    ["p1", "p2"],
    "2+ 单时最后一个边界是当前单，其余按序收进更早组",
  );
  assertDeepEqual(
    grouped.earlierGroups[0]?.items.map((item) => item.id),
    ["before-first", "first-order"],
    "早于首单边界的条目应归入首单，而不是被丢弃",
  );
  assertDeepEqual(
    grouped.earlierGroups[1]?.items.map((item) => item.id),
    ["second-order"],
    "某单的条目 = 该 proposal 创建至下一 proposal 创建之间",
  );
  assertDeepEqual(
    grouped.currentItems.map((item) => item.id),
    ["undated", "current-order"],
    "undated 条目与当前单边界及以后的条目都应落当前单，不静默吞消息",
  );

  const single = groupConversationItemsByProposal(items, boundaries.slice(0, 1));
  assertDeepEqual(single.earlierGroups, [], "只有 1 单时没有「更早」可分");
  assertDeepEqual(
    single.currentItems.map((item) => item.id),
    items.map((item) => item.id),
    "0/1 单原样返回、顺序不变——与折单前逐字节一致",
  );

  const none = groupConversationItemsByProposal(items, []);
  assertDeepEqual(none.earlierGroups, [], "零方案时不分组");
  assertDeepEqual(
    none.currentItems.map((item) => item.id),
    items.map((item) => item.id),
    "零方案原样返回全部条目",
  );
}

// 7) 修单4·A1/A2：2+ 单时中栏应只留一条「更早的 N 单对话」折叠行(默认闭合)，按单分组紧凑收纳，
//    当前单完整留在折叠段之外；1 单时不出现折叠段，行为对折单前 0-diff。
{
  const order1Goal = "先把登录页做好。";
  const order2Goal = `${order1Goal}\n\n补充意见：加个记住我。`;
  const order3Goal = `${order2Goal}\n\n补充意见：再加个找回密码。`;
  const multiOrderProposals = [
    { proposal_id: "proposal:order-1", user_goal: order1Goal, created_at_ms: Date.parse("2026-07-17T01:00:00.000Z") },
    { proposal_id: "proposal:order-2", user_goal: order2Goal, created_at_ms: Date.parse("2026-07-17T02:00:00.000Z") },
    { proposal_id: "proposal:order-3", user_goal: order3Goal, created_at_ms: Date.parse("2026-07-17T03:00:00.000Z") },
  ];
  const multiOrderTurns = userTurnsFromProposalHistory(multiOrderProposals);
  const multiOrderBoundaries = multiOrderProposals.map(({ proposal_id, created_at_ms }) => ({
    proposal_id,
    created_at_ms,
  }));
  const multiOrderNotices = artifactNoticesForConversation({
    proposals: multiOrderBoundaries,
    history: [],
    currentProposalId: "proposal:order-3",
    includeCurrentDelivery: false,
    currentProposalCreatedAtMs: multiOrderProposals[2].created_at_ms,
    onActivate: noop,
  });
  const order1AnsweredQuestion = supervisorEntry({
    entry_id: "blackboard:supervisor:order1-answered",
    question_id: "question:resident:order1",
    title: "主管问题 · 第 1 轮",
    summary: "第 1 轮主管问题：登录页要不要支持手机号？",
    status: "answered",
    source_status: "answered",
    created_at: "2026-07-17T01:30:00.000Z",
  });
  const order3WaitingQuestion = supervisorEntry({
    entry_id: "blackboard:supervisor:order3-waiting",
    question_id: "question:resident:order3",
    title: "主管问题 · 第 2 轮",
    summary: "第 2 轮主管问题：找回密码走邮箱还是短信？",
    status: "waiting_user",
    source_status: "waiting_user",
    created_at: "2026-07-17T03:30:00.000Z",
  });

  const multiOrderTree = stream({
    entries: [order1AnsweredQuestion, order3WaitingQuestion],
    userGoal: order1Goal,
    userTurns: multiOrderTurns,
    artifactNotices: multiOrderNotices,
    proposals: multiOrderBoundaries,
    phaseKind: "proposal",
    phaseContent: null,
  });
  const multiOrderMarkup = renderToStaticMarkup(multiOrderTree);

  const earlierDetails = findElement(
    multiOrderTree,
    (element) => element.type === "details" && element.props?.className === "jiaoban-conversation-earlier",
  );
  assert(earlierDetails, "2+ 单场景应出现「更早的 N 单对话」折叠段");
  assert(earlierDetails.props?.open !== true, "更早的折叠段默认应闭合");
  const earlierText = visibleText(earlierDetails);
  assert(earlierText.includes("更早的 2 单对话"), "折叠摘要应报出更早单数(3 单=2 更早+1 当前)");
  assert(
    earlierText.includes("先把登录页做好") &&
      earlierText.includes("登录页要不要支持手机号") &&
      earlierText.includes("加个记住我"),
    "折叠段应含首单目标一句、首单问答(沿用已答折叠)、第二单补充目标一句",
  );
  assert(
    !earlierText.includes("找回密码走邮箱还是短信") && !earlierText.includes("再加个找回密码"),
    "当前单(第三单)的问答与补充目标不得混进折叠段",
  );
  assert(
    multiOrderMarkup.includes('data-proposal-id="proposal:order-1"') &&
      multiOrderMarkup.includes('data-proposal-id="proposal:order-2"'),
    "折叠段内应按单分组，各段落带自己的 proposal 锚点",
  );
  assert(
    multiOrderMarkup.indexOf('data-proposal-id="proposal:order-1"') <
      multiOrderMarkup.indexOf('data-proposal-id="proposal:order-2"'),
    "更早的单在折叠段内应按时间序排列",
  );
  assert(
    multiOrderMarkup.indexOf("更早的 2 单对话") < multiOrderMarkup.indexOf("再加个找回密码") &&
      multiOrderMarkup.indexOf("更早的 2 单对话") < multiOrderMarkup.indexOf("找回密码走邮箱还是短信"),
    "当前单内容应落在折叠行之后，不被折叠段抢到前面",
  );

  const singleOrderMarkup = renderToStaticMarkup(
    stream({
      entries: chronologicalEntries,
      proposals: [{ proposal_id: "proposal:solo", created_at_ms: Date.parse("2026-07-17T01:00:00.000Z") }],
    }),
  );
  assert(
    !singleOrderMarkup.includes("jiaoban-conversation-earlier"),
    "只有 1 单时不应出现折叠段，行为需与折单前一致",
  );
}

console.log(
  "jiaoban-conversation-center: 5 组消息流/追问回答/幂等/等待态 + 2 组修单4分组折叠 离线 DOM 断言全过",
);
