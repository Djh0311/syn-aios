import { renderToStaticMarkup } from "react-dom/server.browser";
import type {
  BlackboardEntry,
  ProjectWorkflowSummary,
  RunHistoryEntry,
  WorkflowStateSnapshot,
} from "../src/lib/types";
import {
  JiaobanConversationComposer,
  JiaobanConversationStream,
  SUPERVISOR_RESIDENT_SUPERVISOR_MESSAGE_SOURCE_KIND,
  SUPERVISOR_RESIDENT_USER_MESSAGE_SOURCE_KIND,
  WORKFLOW_CHAIN_EVENT_SOURCE_KIND,
  artifactNoticesForConversation,
  groupConversationItemsByProposal,
  isSupervisorProcess,
  isSupervisorResidentSupervisorMessage,
  isSupervisorResidentUserMessage,
  supervisorProcessCanvasView,
  supervisorProcessFocusedNodeId,
  userTurnsFromProposalHistory,
  supervisorConversationEntriesForProject,
} from "../src/views/projects/jiaoban/JiaobanConversation";
import type { JiaobanPhase } from "../src/views/projects/jiaoban/JiaobanArtifactViews";
import { JiaobanHistoryColumn } from "../src/views/projects/jiaoban/JiaobanHistory";
import { useJiaobanConversationState } from "../src/views/projects/jiaoban/useJiaobanConversationState";
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
const noop = () => {};

function supervisorEntry(overrides: Partial<BlackboardEntry>): BlackboardEntry {
  return {
    entry_id: "blackboard:supervisor:fixture",
    project_id: projectId,
    workflow_id: workflowId,
    work_item_id: null,
    workflow_node_id: null,
    question_id: null,
    kind: "supervisor_message",
    title: "主管消息",
    summary: "我先梳理一下。",
    status: "reported",
    source_status: "reported",
    source_refs: [
      {
        source_kind: SUPERVISOR_RESIDENT_SUPERVISOR_MESSAGE_SOURCE_KIND,
        source_id: "message:resident:supervisor:fixture",
        label: "主管自由消息",
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

function supervisorProcessEntry({
  entryId,
  sourceEventType,
  summary,
  createdAt,
  workflowNodeId = null,
}: {
  entryId: string;
  sourceEventType: string;
  summary: string;
  createdAt: string;
  workflowNodeId?: string | null;
}): BlackboardEntry {
  return supervisorEntry({
    entry_id: entryId,
    question_id: null,
    workflow_node_id: workflowNodeId,
    title: `主管过程 · ${sourceEventType}`,
    summary,
    status: "reported",
    source_status: sourceEventType,
    source_refs: [
      {
        source_kind: WORKFLOW_CHAIN_EVENT_SOURCE_KIND,
        source_id: `audit:${sourceEventType}:${entryId}`,
        label: "raw_reason: worker_retry_exhausted: confidential-machine-detail",
      },
    ],
    created_at: createdAt,
  });
}

function stream(overrides: Partial<Parameters<typeof JiaobanConversationStream>[0]> = {}) {
  return (
    <JiaobanConversationStream
      entries={[]}
      userGoal={null}
      phaseKind="legacy"
      // P1-D 后「binding」相位已不可达(批准默认自动新会话直进 prepare)；phaseContent 通道本体是
      // P3-C 占位件(blocked/legacy 渲法)，随 P3-C 退场，本包零碰——这里改名去掉过期的「旧七态」框架。
      phaseContent={<p>blocked 相位占位内容照常在</p>}
      consultLoading={false}
      messageBusyKey={null}
      messageErrors={{}}
      {...overrides}
    />
  );
}

const firstSupervisorMessage = supervisorEntry({
  entry_id: "blackboard:supervisor:first-message",
  // 标题故意像旧用户答复，断言角色绝不能从标题猜。
  title: "用户答复 · 伪装标题",
  summary: "先确认验收是否只看离线结果？",
  created_at: "2026-07-17T01:00:00.000Z",
});
const secondSupervisorMessage = supervisorEntry({
  entry_id: "blackboard:supervisor:second-message",
  summary: "这一单我会按只读边界推进。",
});
const userMessage = supervisorEntry({
  entry_id: "blackboard:supervisor:user-message",
  // 标题故意像旧主管问题，仍必须显示为“你”。
  title: "主管问题 · 伪装标题",
  summary: "只看离线结果，真机由用户重启后验。",
  source_refs: [
    {
      source_kind: SUPERVISOR_RESIDENT_USER_MESSAGE_SOURCE_KIND,
      source_id: "message:resident:user:fixture",
      label: "用户自由消息",
    },
  ],
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
      entries: [userMessage, secondSupervisorMessage, unrelatedRisk, firstSupervisorMessage],
      warnings: [],
    },
  ],
} as unknown as WorkflowStateSnapshot;

const chronologicalEntries = supervisorConversationEntriesForProject(workflowState, projectRoot, workflowId);
assertDeepEqual(
  chronologicalEntries.map((entry) => entry.entry_id),
  [firstSupervisorMessage.entry_id, secondSupervisorMessage.entry_id, userMessage.entry_id],
  "主管消息应按 created_at 正序，且只取当前项目工作流的 supervisor_message",
);
assert(isSupervisorResidentSupervisorMessage(firstSupervisorMessage), "主管身份必须来自 structured source_kind");
assert(isSupervisorResidentUserMessage(userMessage), "用户身份必须来自 structured source_kind");
assert(
  !isSupervisorResidentUserMessage(firstSupervisorMessage) && !isSupervisorResidentSupervisorMessage(userMessage),
  "source identity 不得由标题前缀倒推或混淆",
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
    entries: [firstSupervisorMessage, userMessage],
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
    (proposalMarkup.match(/data-message-kind="supervisor"/g) ?? []).length === 1,
    "主管自由消息应使用 supervisor 消息包装",
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
      element.props?.["data-message-kind"] === "supervisor" &&
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
  assert(userTurn && supervisorTurn, "用户/主管自由消息应保留可独立排版的角色挂点");
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
  const firstSupervisorMessageIndex = proposalMarkup.indexOf("先确认验收是否只看离线结果");
  const userMessageIndex = proposalMarkup.indexOf("只看离线结果，真机由用户重启后验");
  const proposalIndex = proposalMarkup.indexOf("方案好了，放你右手边了");
  assert(
    userGoalIndex < firstSupervisorMessageIndex &&
      firstSupervisorMessageIndex < userMessageIndex &&
      userMessageIndex < proposalIndex,
    "用户原话、主管自由往返与方案短讯应按消息时间序落位",
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
      entries: [secondSupervisorMessage],
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
      amendmentMarkup.indexOf("右区零改。") < amendmentMarkup.indexOf("这一单我会按只读边界推进。"),
    "改要求后的真实口供应落在旧方案之后、主管自由回文之前",
  );
}

const testProjectWorkflow: ProjectWorkflowSummary = {
  project_id: projectId,
  project_root: projectRoot,
  workflow_id: workflowId,
  title: "conversation center fixture",
  state: "running",
  node_count: 0,
  edge_count: 0,
  task_draft_count: 0,
  task_drafts: [],
  node_session_bindings: [],
  node_dispatches: [],
  director_reviews: [],
  execution_controls: [],
  permission_requests: [],
  execution_attempts: [],
};

function ComposerRouteProbe({ phase, isTestProject = true }: { phase: JiaobanPhase; isTestProject?: boolean }) {
  void phase;
  const conversation = useJiaobanConversationState({
    projectWorkflow: testProjectWorkflow,
    workflowState,
    projectRoot,
    onProposalStoreRefresh: noop,
    humanizeAnswerError: () => "这句没送到主管——稍后再试一次。",
  });
  const composer = conversation.makeConversationComposer({ isTestProject });
  return <JiaobanConversationComposer {...composer} />;
}

// 2) 底1常驻框：唯一 message 路由、受控草稿、Enter 语义不变；所有 test-project 相位均可发。
{
  const draftChanges: string[] = [];
  const submission = { count: 0 };
  const composer = (
    <JiaobanConversationComposer
      route={{ kind: "message" }}
      draft="保持只读。"
      busy={false}
      onDraftChange={(value) => draftChanges.push(value)}
      onSubmit={() => {
        submission.count += 1;
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

  // 「只要一个对话框,上下不带字」：无按钮，Enter 发送、Shift+Enter 换行。
  assert(!findButtonByText(composer, "说给主管"), "常驻输入框不再挂发送按钮");
  const onKeyDown = textarea.props?.onKeyDown as
    | ((event: { key: string; shiftKey: boolean; preventDefault: () => void }) => void)
    | undefined;
  assert(onKeyDown, "常驻输入框应接 Enter 发送");
  onKeyDown({ key: "Enter", shiftKey: true, preventDefault: noop });
  assert(Number(submission.count) === 0, "Shift+Enter 只换行不发送");
  onKeyDown({ key: "Enter", shiftKey: false, preventDefault: noop });
  assertDeepEqual(draftChanges, ["只读，不写项目根。"], "草稿变更应回传输入值");
  assert(Number(submission.count) === 1, "Enter 应触发一次提交");

  for (const phase of ["say", "authorize", "running", "done", "blocked"] as JiaobanPhase[]) {
    const markup = renderToStaticMarkup(<ComposerRouteProbe phase={phase} />);
    assert(markup.includes('data-composer-route="message"'), `${phase} 相位必须走唯一 user message 路由`);
    assert(!markup.includes("disabled"), `${phase} 相位不得因旧状态机锁住常驻框`);
  }

  const nonTestMarkup = renderToStaticMarkup(<ComposerRouteProbe phase="running" isTestProject={false} />);
  assert(
    nonTestMarkup.includes("这个项目还没接执行") && nonTestMarkup.includes("disabled"),
    "P1-E 非测试项目的诚实关门语义仍须保留",
  );

  const streamMarkup = renderToStaticMarkup(stream({ entries: chronologicalEntries }));
  assert(!streamMarkup.includes("<textarea"), "消息流内不再内嵌输入框——输入统一走常驻框");
}

// 3) 自由消息没有 question/answer 折叠或 receipt；右区方案批准动作不被一条普通消息锁走。
{
  const freeMessageMarkup = renderToStaticMarkup(stream({ entries: [firstSupervisorMessage, userMessage] }));
  assert(
    freeMessageMarkup.includes('data-message-kind="supervisor"') && freeMessageMarkup.includes('data-message-kind="user"'),
    "普通主管/用户消息必须按 source identity 显示",
  );
  for (const retiredMarkup of ["supervisor-question", "user-answer", "waiting_user", "question_id", "已答"]) {
    assert(!freeMessageMarkup.includes(retiredMarkup), `自由消息流不得再暴露 P1-B 回合语义：${retiredMarkup}`);
  }
  const proposalActionMarkup = renderToStaticMarkup(
    stream({
      entries: [secondSupervisorMessage],
      phaseKind: "proposal",
      phaseContent: <button type="button">允许并开始旧方案</button>,
    }),
  );
  assert(
    proposalActionMarkup.includes("允许并开始旧方案"),
    "自由消息不应替右侧既有方案批准动作抢占或推进工作流",
  );
}

// 4) 发送失败与刷新失败都以统一 alert 行上脸；不泄露已退役 question 路由机器词。
{
  const errorMarkup = renderToStaticMarkup(
    stream({
      entries: [userMessage],
      messageErrors: { "resident-message": "这句没送到主管——稍后再试一次。" },
    }),
  );
  assert(
    errorMarkup.includes('data-message-kind="message-error"') && errorMarkup.includes('role="alert"'),
    "统一用户消息失败必须以 alert 行上脸，不许静默",
  );
  assert(!errorMarkup.includes("question_id") && !errorMarkup.includes("waiting_user"), "失败行不得复活旧问答路由词");

  const composerErrorMarkup = renderToStaticMarkup(
    <JiaobanConversationComposer
      route={{ kind: "message" }}
      draft=""
      busy={false}
      error="这句没送到主管——稍后再试一次。"
      onDraftChange={noop}
      onSubmit={noop}
    />,
  );
  assert(
    composerErrorMarkup.includes('role="alert"') && composerErrorMarkup.includes("这句没送到主管"),
    "发送失败的常驻框也必须保留可访问 alert",
  );
}

// 5) 提交期间的「主管在看」仍是人话等待态，但不会吞掉已落 canonical 的过程/对话消息。
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
  assert(waitingStatus, "发送期间应有可访问的等待状态");
  assert(visibleText(waitingStatus).includes("主管在看"), "等待态必须带主管在看人话，不能只有 spinner");
  assert(markup.includes('class="jiaoban-spinner" aria-hidden="true"'), "呼吸点只作装饰并应对读屏隐藏");
  assert(!markup.includes("旧说态输入框"), "主管在看时不应同时露出旧相位内容");

  const messageRoundMarkup = renderToStaticMarkup(
    stream({
      entries: [secondSupervisorMessage],
      messageBusyKey: "resident-message",
    }),
  );
  assert(messageRoundMarkup.includes("主管在看"), "任意用户消息送达后的 resident 回合都必须进入主管在看等待态");
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
  const order1SupervisorMessage = supervisorEntry({
    entry_id: "blackboard:supervisor:order1-message",
    summary: "登录页要不要支持手机号？",
    created_at: "2026-07-17T01:30:00.000Z",
  });
  const order3SupervisorMessage = supervisorEntry({
    entry_id: "blackboard:supervisor:order3-message",
    summary: "找回密码走邮箱还是短信？",
    created_at: "2026-07-17T03:30:00.000Z",
  });

  const multiOrderTree = stream({
    entries: [order1SupervisorMessage, order3SupervisorMessage],
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
    "折叠段应含首单目标一句、首单主管消息、第二单补充目标一句",
  );
  assert(
    !earlierText.includes("找回密码走邮箱还是短信") && !earlierText.includes("再加个找回密码"),
    "当前单(第三单)的主管消息与补充目标不得混进折叠段",
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

// 8) P3-A：链审计派生的过程短讯走结构化 source_kind/source_status，不碰用户答复、机器 reason 或 P3-B 回话。
{
  const processEntries = [
    supervisorProcessEntry({
      entryId: "blackboard:process:run-started",
      sourceEventType: "workflow_chain_run_started",
      summary: "开跑了，3 个任务。",
      createdAt: "2026-07-18T01:00:00.000Z",
    }),
    supervisorProcessEntry({
      entryId: "blackboard:process:node-started",
      sourceEventType: "workflow_chain_node_started",
      summary: "第 1 件开始做了。",
      createdAt: "2026-07-18T01:00:01.000Z",
      workflowNodeId: "node:build-shell",
    }),
    supervisorProcessEntry({
      entryId: "blackboard:process:node-completed",
      sourceEventType: "workflow_chain_node_completed",
      summary: "第 1 件干完了。",
      createdAt: "2026-07-18T01:00:02.000Z",
      workflowNodeId: "node:build-shell",
    }),
    supervisorProcessEntry({
      entryId: "blackboard:process:waiting",
      sourceEventType: "workflow_chain_node_waiting_decision",
      summary: "停下来了——worker 有话问你。",
      createdAt: "2026-07-18T01:00:03.000Z",
      workflowNodeId: "node:review",
    }),
    supervisorProcessEntry({
      entryId: "blackboard:process:rework",
      sourceEventType: "workflow_chain_node_needs_rework",
      summary: "这一步得回去重做。",
      createdAt: "2026-07-18T01:00:04.000Z",
      workflowNodeId: "node:review",
    }),
    supervisorProcessEntry({
      entryId: "blackboard:process:run-completed",
      sourceEventType: "workflow_chain_run_completed",
      summary: "都干完了，结果放你右手边。",
      createdAt: "2026-07-18T01:00:05.000Z",
    }),
    supervisorProcessEntry({
      entryId: "blackboard:process:run-stopped",
      sourceEventType: "workflow_chain_run_stopped",
      summary: "这单停下来了，右边能看到进度。",
      createdAt: "2026-07-18T01:00:06.000Z",
    }),
  ];
  assert(processEntries.every(isSupervisorProcess), "链过程消息必须由 workflow_chain_event source ref 结构化识别");
  assert(
    processEntries.every((entry) => entry.status === "reported" && entry.question_id == null),
    "过程消息是只读汇报，不得变成待答问题或回话入口",
  );
  assert(
    processEntries.filter((entry) => supervisorProcessCanvasView(entry) === "delivery").length === 1,
    "只有链完成过程消息可切交货；其余事件一律回到工序图",
  );

  const activatedProcessEntries: BlackboardEntry[] = [];
  const processTree = stream({
    entries: processEntries,
    phaseKind: "conversation",
    phaseContent: null,
    onSupervisorProcessActivate: (entry) => activatedProcessEntries.push(entry),
  });
  const processMarkup = renderToStaticMarkup(processTree);
  assert(
    (processMarkup.match(/data-message-kind="supervisor-process"/g) ?? []).length === processEntries.length,
    "七类链事件应各渲一条主管过程短讯，不得折成用户答复",
  );
  assert(!processMarkup.includes('data-message-kind="user-answer"'), "过程短讯不得误渲为“你”的答复");
  assert(
    !processMarkup.includes("worker_retry_exhausted") && !processMarkup.includes("confidential-machine-detail"),
    "机器 reason 只能留在右区/details，过程消息正文不得泄露",
  );
  const sequence = processEntries.map((entry) => entry.summary);
  for (let index = 1; index < sequence.length; index += 1) {
    assert(
      processMarkup.indexOf(sequence[index - 1]!) < processMarkup.indexOf(sequence[index]!),
      "过程短讯应按审计派生时间顺序落入中心消息流",
    );
  }

  const nodeProcessButton = findElement(
    processTree,
    (element) => element.type === "button" && element.props?.["aria-label"] === "打开右侧工序图并定位任务",
  );
  assert(nodeProcessButton, "节点过程短讯应可切右侧工序图并定位节点");
  const activateNodeProcess = nodeProcessButton.props?.onClick as (() => void) | undefined;
  assert(activateNodeProcess, "节点过程短讯应保留只读右区聚焦回调");
  activateNodeProcess();
  assert(
    activatedProcessEntries[0]?.workflow_node_id === "node:build-shell" &&
      supervisorProcessCanvasView(activatedProcessEntries[0]!) === "graph" &&
      supervisorProcessFocusedNodeId(activatedProcessEntries[0]!) === "node:build-shell",
    "节点过程消息应只切 graph 并携带原 workflow_node_id",
  );

  const graphOnlyProcessButton = findElement(
    processTree,
    (element) => element.type === "button" && element.props?.["aria-label"] === "打开右侧工序图",
  );
  assert(graphOnlyProcessButton, "无节点链过程短讯应只可打开右侧工序图");
  const activateGraphOnlyProcess = graphOnlyProcessButton.props?.onClick as (() => void) | undefined;
  assert(activateGraphOnlyProcess, "无节点过程短讯也应保留只读右区聚焦回调");
  activateGraphOnlyProcess();
  assert(
    activatedProcessEntries[1]?.source_status === "workflow_chain_run_started" &&
      supervisorProcessCanvasView(activatedProcessEntries[1]!) === "graph" &&
      supervisorProcessFocusedNodeId(activatedProcessEntries[1]!) === null,
    "无 workflow_node_id 的链过程消息只聚焦 graph，绝不伪造节点定位",
  );

  const deliveryProcessButton = findElement(
    processTree,
    (element) => element.type === "button" && element.props?.["aria-label"] === "打开右侧交货",
  );
  assert(deliveryProcessButton, "链完成短讯应可结构化聚焦右侧交货");
  const activateDeliveryProcess = deliveryProcessButton.props?.onClick as (() => void) | undefined;
  assert(activateDeliveryProcess, "链完成短讯应保留右侧交货回调");
  activateDeliveryProcess();
  assert(
    activatedProcessEntries[2]?.source_status === "workflow_chain_run_completed" &&
      supervisorProcessCanvasView(activatedProcessEntries[2]!) === "delivery" &&
      supervisorProcessFocusedNodeId(activatedProcessEntries[2]!) === null,
    "只有 canonical workflow_chain_run_completed 可切 delivery，不得看 title/reason 猜",
  );

  const processAndThinkingMarkup = renderToStaticMarkup(
    stream({
      entries: [processEntries[1]!],
      consultLoading: true,
      phaseKind: "conversation",
      phaseContent: null,
      onSupervisorProcessActivate: noop,
    }),
  );
  assert(
    processAndThinkingMarkup.includes("第 1 件开始做了。") && processAndThinkingMarkup.includes("主管在看"),
    "跑期过程短讯应与“主管在看”等待态共存，不能被空转等待吞掉",
  );
}

console.log(
  "jiaoban-conversation-center: 5 组消息流/追问回答/幂等/等待态 + 2 组修单4分组折叠 + P3-A 过程短讯 离线 DOM 断言全过",
);
