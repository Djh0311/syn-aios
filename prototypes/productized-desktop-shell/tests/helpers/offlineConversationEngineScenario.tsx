import React from "react";
import { renderToStaticMarkup } from "react-dom/server.browser";
import { assert, buttonTextsInMarkup, findButtonByText, findElement, visibleText } from "./offlineInteractionTestUtils";
import {
  appendPendingUserMessage,
  buildManualRelayAssistantMessage,
  buildManualRelayLiveTranscriptEvents,
  buildManualRelayPendingUserMessage,
  buildPendingUserMessage,
  mergeOlderTranscriptPage,
} from "../../src/lib/conversationEngine";
import { AgentChatComposer } from "../../src/views/agent/AgentChatComposer";
import {
  AgentManualRelayDeveloperDetails,
  agentRoleSessionContinuationBlockedReason,
  deriveRelayBindingState,
  manualRelayAttemptTimedOut,
  nextManualRelayPollFailureDecision,
} from "../../src/views/agent/AgentConversationShell";
import type { CodexTranscript, PendingAction, SessionRecord } from "../../src/lib/types";
import { AgentSessionCenter, ChatTranscript } from "../../src/views/AgentView";

// Remove the per-turn process folds (depth-aware, so nested status <details>
// inside them are handled) to get "what the main stream shows when folds are
// collapsed" — the faithful basis for asserting 剥折叠后正常态不含过程事件.
function stripProcessFolds(markup: string): string {
  const OPEN = '<details class="chat-turn-process"';
  const CLOSE = "</details>";
  let result = "";
  let cursor = 0;
  while (cursor < markup.length) {
    const open = markup.indexOf(OPEN, cursor);
    if (open === -1) {
      result += markup.slice(cursor);
      break;
    }
    result += markup.slice(cursor, open);
    let depth = 0;
    let scan = open + OPEN.length;
    while (scan < markup.length) {
      const nextOpen = markup.indexOf("<details", scan);
      const nextClose = markup.indexOf(CLOSE, scan);
      if (nextClose === -1) {
        scan = markup.length;
        break;
      }
      if (nextOpen !== -1 && nextOpen < nextClose) {
        depth += 1;
        scan = nextOpen + "<details".length;
        continue;
      }
      if (depth === 0) {
        scan = nextClose + CLOSE.length;
        break;
      }
      depth -= 1;
      scan = nextClose + CLOSE.length;
    }
    cursor = scan;
  }
  return result;
}

export function runConversationEngineScenario({
  captureAction,
  session,
}: {
  captureAction: (action: PendingAction) => void;
  session: SessionRecord;
}) {
  const transcript = buildLargeTranscript(session.thread_id, session.rollout_path ?? "fixture-rollout.jsonl", 180);
  const transcriptMarkup = renderToStaticMarkup(<ChatTranscript transcript={transcript} />);
  const transcriptText = visibleText(<ChatTranscript transcript={transcript} />);
  const firstPollFailure = nextManualRelayPollFailureDecision(0);
  const terminalPollFailure = nextManualRelayPollFailureDecision(4);
  assert(firstPollFailure.shouldRetry && firstPollFailure.nextDelayMs === 1000, "P4 轮询首次失败应退避重试，不应冻结");
  assert(!terminalPollFailure.shouldRetry, "P4 轮询连续失败到上限应进入可恢复状态");
  assert(
    manualRelayAttemptTimedOut({ started_at: "2026-06-17T00:00:00Z" }, Date.parse("2026-06-17T00:10:00Z")),
    "P4 relay 前端墙钟超时应能独立判定",
  );

  assert(transcriptMarkup.includes("data-conversation-engine=\"turns\""), "M1 对话流应声明使用按轮渲染引擎");
  assert(transcriptMarkup.includes("codex-transcript-item"), "P3 对话流应使用 Codex 平铺消息项");
  assert(!transcriptMarkup.includes("chat-bubble"), "P3 对话流不应再使用聊天气泡类名");
  assert(
    !transcriptMarkup.includes("data-virtualized-window=") && !transcriptMarkup.includes("translateY("),
    "M1 抖动修复：不应再使用固定估算高度的虚拟窗口/绝对偏移",
  );
  assert(transcriptText.includes("Message fixture 179"), "M1 应显示最新消息");
  assert(
    transcriptText.includes("Message fixture 0") && transcriptText.includes("Message fixture 20"),
    "M1 整段渲染：更早消息也应进入 DOM 可达，不再被写死的 132 估算裁掉",
  );

  const nativeTranscript: CodexTranscript = {
    ...transcript,
    events: [
      {
        event_id: "native-user",
        timestamp: "2026-06-17T00:00:00Z",
        event_type: "user_message",
        actor: "user",
        text: "P3 native render user fixture",
        warnings: [],
      },
      {
        event_id: "native-assistant",
        timestamp: "2026-06-17T00:00:01Z",
        event_type: "assistant_message",
        actor: "assistant",
        text: "P3 native render assistant fixture",
        warnings: [],
      },
      {
        event_id: "native-tool-call",
        timestamp: "2026-06-17T00:00:02Z",
        event_type: "tool_call",
        actor: "assistant",
        tool_name: "functions.exec_command",
        arguments: { cmd: "pwd" },
        warnings: [],
      },
      {
        event_id: "native-command-output",
        timestamp: "2026-06-17T00:00:03Z",
        event_type: "command_output",
        actor: "tool",
        stdout: "/tmp/offline\n",
        stderr: "",
        exit_code: 0,
        output: { stdout: "/tmp/offline\n", stderr: "", exit_code: 0 },
        warnings: [],
      },
      {
        event_id: "native-reasoning",
        timestamp: "2026-06-17T00:00:04Z",
        event_type: "system_context",
        actor: "assistant",
        text: "Reasoning fixture summary",
        metadata: { payload_type: "reasoning" },
        warnings: [],
      },
      {
        event_id: "native-compacted",
        timestamp: "2026-06-17T00:00:05Z",
        event_type: "compacted",
        actor: "system",
        text: "Compacted fixture summary.",
        warnings: [],
      },
    ],
  };
  const nativeMarkup = renderToStaticMarkup(<ChatTranscript transcript={nativeTranscript} />);
  const nativeText = visibleText(<ChatTranscript transcript={nativeTranscript} />);
  assert(nativeMarkup.includes("codex-status-item"), "P3 工具/系统事件应作为 Codex 状态行渲染");
  assert(
    nativeMarkup.includes("codex-transcript-item assistant") && nativeText.includes("P3 native render assistant fixture"),
    "P3 助手回复应一眼可辨",
  );
  assert(nativeText.includes("准备运行命令"), "P3 工具调用应显示为命令状态行");
  assert(nativeText.includes("已运行命令"), "P3 命令输出应显示为完成状态行");
  assert(nativeText.includes("思考"), "P3 reasoning 应显示为思考块");
  assert(nativeText.includes("上下文已自动压缩"), "P3 compacted 应显示为压缩分隔");

  // Part 3 · 过程事件按轮收纳：每一轮 = 过程[默认折叠] + 最终输出（主流只显最终 agent_message）。
  // 死线（接 ui-internal-field-disclosure-sweep）：过程事件是收纳进折叠、不是删——折叠内必须可达。
  const turnDisclosureTranscript: CodexTranscript = {
    ...transcript,
    events: [
      {
        event_id: "turn-user",
        timestamp: "2026-06-17T00:10:00Z",
        event_type: "user_message",
        actor: "user",
        text: "P3 turn user fixture",
        warnings: [],
      },
      {
        event_id: "turn-tool",
        timestamp: "2026-06-17T00:10:01Z",
        event_type: "tool_call",
        actor: "assistant",
        tool_name: "functions.exec_command",
        arguments: { cmd: "pwd" },
        warnings: [],
      },
      {
        event_id: "turn-reasoning",
        timestamp: "2026-06-17T00:10:02Z",
        event_type: "system_context",
        actor: "assistant",
        text: "P3 turn reasoning fixture",
        metadata: { payload_type: "reasoning" },
        warnings: [],
      },
      {
        event_id: "turn-final",
        timestamp: "2026-06-17T00:10:03Z",
        event_type: "assistant_message",
        actor: "assistant",
        text: "P3 turn final fixture",
        warnings: [],
      },
    ],
  };
  const turnMarkup = renderToStaticMarkup(<ChatTranscript transcript={turnDisclosureTranscript} />);
  const turnText = visibleText(<ChatTranscript transcript={turnDisclosureTranscript} />);
  assert(turnMarkup.includes("chat-turn-process") && turnMarkup.includes("chat-turn-process-list"), "Part3 每轮过程事件应收纳进按轮折叠");
  assert(
    /<details class="chat-turn-process"[^>]*>/.test(turnMarkup) && !/<details class="chat-turn-process"[^>]*\sopen/.test(turnMarkup),
    "Part3 过程折叠应默认收起，主流默认只显最终输出",
  );
  assert(
    turnMarkup.includes("</details><article class=\"codex-transcript-item assistant"),
    "Part3 最终 agent 输出应作为主流条目位于过程折叠之外",
  );
  assert(turnText.includes("P3 turn final fixture"), "Part3 最终 agent 输出必须显示在主流");
  assert(
    turnMarkup.includes("准备运行命令") && turnMarkup.includes("思考"),
    "Part3 死线：过程事件（工具调用 / 思考）必须在折叠内可达，不能被删而非收纳",
  );
  assert(turnMarkup.includes("data-turn-process=\"2\""), "Part3 折叠应标注该轮过程步数");
  assert(
    stripProcessFolds(turnMarkup).includes("P3 turn final fixture") && !stripProcessFolds(turnMarkup).includes("准备运行命令"),
    "Part3 剥折叠后主流应只剩最终输出，不含过程事件",
  );

  // R2（真机回归）· 一轮多条 assistant_message：轮按 user_message 划，末条 assistant = 最终输出，
  // 之前的前导消息（codex 常见“我看下这个文件”）连同工具一起折进 process，不漏进主流、也不被删。
  const multiAssistantTurnTranscript: CodexTranscript = {
    ...transcript,
    events: [
      {
        event_id: "ma-user",
        timestamp: "2026-06-17T00:20:00Z",
        event_type: "user_message",
        actor: "user",
        text: "R2 multi-assistant user fixture",
        warnings: [],
      },
      {
        event_id: "ma-preamble",
        timestamp: "2026-06-17T00:20:01Z",
        event_type: "assistant_message",
        actor: "assistant",
        text: "R2 preamble 我看下这个文件",
        warnings: [],
      },
      {
        event_id: "ma-tool",
        timestamp: "2026-06-17T00:20:02Z",
        event_type: "tool_call",
        actor: "assistant",
        tool_name: "functions.exec_command",
        arguments: { cmd: "cat file" },
        warnings: [],
      },
      {
        event_id: "ma-final",
        timestamp: "2026-06-17T00:20:03Z",
        event_type: "assistant_message",
        actor: "assistant",
        text: "R2 final answer fixture",
        warnings: [],
      },
    ],
  };
  const multiAssistantMarkup = renderToStaticMarkup(<ChatTranscript transcript={multiAssistantTurnTranscript} />);
  const multiAssistantMainStream = stripProcessFolds(multiAssistantMarkup);
  assert(
    multiAssistantMainStream.includes("R2 final answer fixture"),
    "R2 一轮多条 assistant 时主流必须显示末条最终输出",
  );
  assert(
    !multiAssistantMainStream.includes("R2 preamble 我看下这个文件"),
    "R2 前导 assistant 消息不应漏进主流最终输出",
  );
  assert(
    multiAssistantMarkup.includes("R2 preamble 我看下这个文件"),
    "R2 死线：前导消息应折进 per-turn 过程折叠内可达，不能被删",
  );
  assert(
    multiAssistantMarkup.includes("data-turn-process=\"2\""),
    "R2 前导消息 + 工具应合计为该轮 2 步过程",
  );
  assert(
    multiAssistantMainStream.split("codex-transcript-item assistant").length - 1 === 1,
    "R2 该轮主流应只剩唯一一条 agent 最终输出，不再每条 assistant 各成一条",
  );

  // §2 标题溢出：会话卡标题挂 sc-title 截断类（CSS ellipsis 钩子），超长标题完整原文进 title tooltip，
  // 不靠撑破容器来显示。真实 DOM 文本截短由后端 truncate_display_title 负责（codex_db Rust 测试覆盖）。
  const overflowTitle = "超长标题".repeat(2000);
  const longTitleSession: SessionRecord = { ...session, thread_id: "long-title-thread", title: overflowTitle };
  const longTitleCenterMarkup = renderToStaticMarkup(
    <AgentSessionCenter
      sessions={[longTitleSession]}
      selectedThreadId={longTitleSession.thread_id}
      selectedSession={longTitleSession}
      transcript={null}
      loadingThreadId={null}
      transcriptError={null}
      projectSessionCount={1}
      onOpenSession={() => {}}
      onRequestAction={captureAction}
    />,
  );
  assert(longTitleCenterMarkup.includes("sc-title"), "§2 会话卡标题应使用 sc-title 截断类");
  assert(
    longTitleCenterMarkup.includes(`title="${overflowTitle}"`),
    "§2 会话卡应把完整标题留在 tooltip，截断只发生在显示层",
  );

  const centerText = visibleText(
    <AgentSessionCenter
      sessions={[session]}
      selectedThreadId={session.thread_id}
      selectedSession={session}
      transcript={transcript}
      loadingThreadId={session.thread_id}
      transcriptError={null}
      projectSessionCount={1}
      onOpenSession={() => {}}
      onRequestAction={captureAction}
    />,
  );
  assert(centerText.includes("Message fixture 179"), "M1 选中会话加载中也应保留已读历史，不出现空窗");
  assert(centerText.includes("正在刷新这条对话"), "M1 背景刷新应是状态提示，不应清空历史");
  const firstLoadText = visibleText(
    <AgentSessionCenter
      sessions={[session]}
      selectedThreadId={session.thread_id}
      selectedSession={session}
      transcript={null}
      loadingThreadId={session.thread_id}
      transcriptError={null}
      projectSessionCount={1}
      onOpenSession={() => {}}
      onRequestAction={captureAction}
    />,
  );
  assert(firstLoadText.includes("正在读取这条对话"), "M1 首次加载新会话应显示读取态，不应静默空窗");
  assert(!firstLoadText.includes("0 条结果"), "M1 首次加载态不得暗示读回 0 条");

  const streamingTranscript = {
    ...transcript,
    events: [
      ...transcript.events,
      {
        event_id: "streaming-assistant-draft",
        timestamp: "2026-06-17T00:59:59Z",
        event_type: "assistant_message",
        actor: "assistant",
        text: "Streaming fixture draft",
        metadata: { conversation_engine_streaming: true },
        warnings: [],
      },
    ],
  };
  const streamingMarkup = renderToStaticMarkup(<ChatTranscript transcript={streamingTranscript} />);
  const streamingText = visibleText(<ChatTranscript transcript={streamingTranscript} />);
  assert(streamingMarkup.includes("data-stick-to-bottom=\"true\""), "M2 对话流应声明默认黏底");
  assert(streamingMarkup.includes("data-streaming=\"true\""), "M2 末轮流式输出应被标记为流式");
  assert(streamingText.includes("回到底部"), "M2 流式时应有回到底部入口");
  assert(streamingText.includes("Streaming fixture draft"), "M2 流式草稿应作为末轮最终输出自然显示");
  assert(!streamingMarkup.includes("translateY("), "M2 不应再使用固定偏移的绝对定位虚拟层");

  const boundedTailTranscript = {
    ...transcript,
    events: transcript.events.slice(-12),
    pagination: {
      mode: "tail",
      page_size: 12,
      returned_events: 12,
      total_line_count: 180,
      selected_line_count: 12,
      has_older: true,
      older_before_line: 169,
    },
  };
  const boundedMarkup = renderToStaticMarkup(
    <ChatTranscript
      olderLoading={false}
      transcript={boundedTailTranscript}
      onLoadOlder={() => {
        throw new Error("offline render should not invoke older loader");
      }}
    />,
  );
  assert(boundedMarkup.includes("data-transcript-load=\"bounded\""), "M2 点开对话应声明 transcript 加载已界定");
  assert(
    !boundedMarkup.includes("加载更早对话") &&
      !boundedMarkup.includes("chat-load-older") &&
      !buttonTextsInMarkup(boundedMarkup).some((text) => text.includes("加载更早")),
    "M2 去栏：顶部不应再有「加载更早」按钮",
  );
  assert(boundedMarkup.includes("data-older-preload=\"pending\""), "M2 有 older cursor 时应改为距顶预加载的静默提示");

  const internalOnlyTailTranscript: CodexTranscript = {
    ...boundedTailTranscript,
    events: boundedTailTranscript.events.map((event, index) => ({
      ...event,
      event_id: `internal-only-tail-${index}`,
      event_type: "tool_call",
      actor: "tool",
      text: `Tool fixture ${index}`,
    })),
  };
  const internalOnlyMarkup = renderToStaticMarkup(
    <ChatTranscript
      olderLoading={false}
      transcript={internalOnlyTailTranscript}
      onLoadOlder={() => {
        throw new Error("offline render should not invoke older loader");
      }}
    />,
  );
  assert(internalOnlyMarkup.includes("codex-status-item"), "P3 工具事件应作为 Codex 状态行渲染");
  assert(internalOnlyMarkup.includes("chat-turn-process"), "Part3 纯工具事件 tail 应收纳进按轮过程折叠（可达）");
  assert(internalOnlyMarkup.includes("准备调用工具"), "P3 无具体工具名的工具事件应显示通用工具状态");
  assert(!internalOnlyMarkup.includes("这条会话没有可显示的对话"), "P3 工具事件 tail 不应再显示对话空态");
  assert(!internalOnlyMarkup.includes("加载更早对话"), "M2 去栏后工具事件 tail 不应再有加载更早按钮");

  const olderPage = {
    ...transcript,
    events: transcript.events.slice(-24, -12),
    pagination: {
      mode: "older",
      page_size: 12,
      returned_events: 12,
      total_line_count: 180,
      selected_line_count: 12,
      has_older: true,
      older_before_line: 157,
    },
  };
  const mergedTranscript = mergeOlderTranscriptPage(boundedTailTranscript, olderPage);
  assert(mergedTranscript.events[0].text === "Message fixture 156", "M2 更早页应前插到当前尾页之前");
  assert(mergedTranscript.events.at(-1)?.text === "Message fixture 179", "M2 前插更早页后必须保留最新尾部消息");
  assert(mergedTranscript.pagination?.older_before_line === 157, "M2 前插后应延续 older cursor");

  const composerMarkup = renderToStaticMarkup(
    <AgentSessionCenter
      sessions={[session]}
      selectedThreadId={session.thread_id}
      selectedSession={session}
      transcript={transcript}
      loadingThreadId={null}
      transcriptError={null}
      projectSessionCount={1}
      showSoftwareLayer={false}
      onOpenSession={() => {}}
      onRequestAction={captureAction}
    />,
  );
  assert(
    composerMarkup.includes("data-send-mode=\"decision-only\""),
    "M3C06 未收到服务端角色会话 selector 时，历史会话撰写区必须保持禁用",
  );
  assert(composerMarkup.includes("历史会话仅供阅读"), "M3C06 不得把 SessionRecord 变成 existing send 目标");
  const selectedProjectTail = (session.project_root ?? "").split("/").filter(Boolean).at(-1) ?? "";
  assert(!composerMarkup.includes("继续对话"), "信息收口后撰写区不应常驻显示普通对话目标");
  assert(composerMarkup.includes("发送"), "M3 撰写区主按钮应是发送");
  assert(!composerMarkup.includes("生成发送预览"), "M3 普通撰写区不应保留 6 步预览入口");
  assert(!composerMarkup.includes("确认执行 Codex"), "M3 普通撰写区不应出现真实执行按钮");
  assert(!composerMarkup.includes("确认 mock 中转一次"), "B2 主路径不应保留二次 mock 中转确认按钮");

  const otherProjectSession: SessionRecord = {
    ...session,
    thread_id: "codex-other-project-thread",
    title: "Other project Codex",
    project_root: "/offline-fixture/projects/other-codex-project",
    thread_source: "codex",
  };
  const crossProjectMarkup = renderToStaticMarkup(
    <AgentSessionCenter
      sessions={[session, otherProjectSession]}
      selectedThreadId={session.thread_id}
      selectedSession={session}
      transcript={transcript}
      loadingThreadId={null}
      transcriptError={null}
      projectSessionCount={2}
      showSoftwareLayer={false}
      onOpenSession={() => {}}
      onRequestAction={captureAction}
    />,
  );
  assert(
    crossProjectMarkup.includes("Other project Codex"),
    "B2 bind-fix 对话下拉不得被当前项目过滤到看不见其它项目的 Codex 会话",
  );

  const staleProjectRoot = "/offline-fixture/projects/stale-project";
  const relayBinding = deriveRelayBindingState({
    ...session,
    project_root: "/offline-fixture/projects/selected-codex-project",
    thread_source: "codex",
  });
  assert(relayBinding.enabled === true, "B2 bind-fix 可从历史会话取得非权威项目定位提示");
  assert(
    relayBinding.targetProjectRoot !== staleProjectRoot &&
      relayBinding.targetProjectRoot === "/offline-fixture/projects/selected-codex-project",
    "B2 bind-fix 项目定位提示必须跟随选中会话自己的 project_root，不得沿用旧项目选择",
  );
  assert(
    agentRoleSessionContinuationBlockedReason(undefined, relayBinding.targetProjectRoot)?.includes("仅供阅读"),
    "M3C06 历史项目定位提示不得自行授权 existing continuation",
  );
  const missingProjectBinding = deriveRelayBindingState({
    ...session,
    project_root: null,
    thread_source: "codex",
  });
  assert(missingProjectBinding.enabled === false, "B2 bind-fix 缺 project_root 的 Codex 会话不得猜测目标项目");
  assert(
    missingProjectBinding.blockedReason === "当前会话未记录项目路径",
    "B2 bind-fix 缺 project_root 时 UI/读模型必须写清绑定失败原因",
  );

  let directSubmitCount = 0;
  const directComposer = (
    <AgentChatComposer
      draftPrompt="Send this exact GUI prompt"
      k2PreviewError={null}
      manualRelayBusy={false}
      manualRelayError={null}
      manualRelayReceipt={null}
      relayDirectSendEnabled={true}
      relayDirectSendBlockedReason={null}
      selectedProjectRoot={session.project_root ?? ""}
      selectedSession={session}
      onChangeDraft={() => {}}
      onOpenDeveloperDetails={() => {}}
      onStopManualRelayAttempt={() => {}}
      onSubmitDraft={() => {
        directSubmitCount += 1;
      }}
    />
  );
  const directComposerMarkup = renderToStaticMarkup(directComposer);
  assert(!directComposerMarkup.includes("继续对话"), "信息收口后独立 composer 不应常驻显示普通对话目标");
  // ⑥ H 定稿(2026-07-14 hifi `H · 智能体页`)**定向反转**了这一条：
  //   原断言 = `!includes(selectedProjectTail)`「信息收口后 composer 不应常驻显示项目名」。
  //   定稿 H 段原话：「manual relay 发送框在底部——发送前写根/沙箱一行可见(治体检 P1「批态可见性缺席」)」。
  //   即：当初的「信息收口」把**写根**一起收掉了，而写根恰恰是宪法 §一 批态 D5 点名必须可见的东西
  //   (「用户能在批面看到写根/工具/边界」的可见性承诺不变)——用户正要往一个真实项目里发写指令时，
  //   看不见会写到哪 = 违宪。故项目名(=写根)从「禁止常驻」改为「必须常驻」。
  // ⚠️ 只反转「项目名」这一项。同组其余收口断言(完整路径 / session id / 会话ID / relay 原始字段 / 边界折叠)
  //   定稿没动，**原样留着**，下面逐条仍在。
  assert(
    directComposerMarkup.includes(selectedProjectTail),
    "⑥ H：composer 必须常驻显示写根项目名(批态可见性·治体检 P1)",
  );
  assert(directComposerMarkup.includes("workspace-write"), "⑥ H：composer 必须常驻显示本次发送的沙箱模式");
  assert(!directComposerMarkup.includes(session.project_root ?? ""), "信息收口后 composer 不应常驻显示完整项目路径");
  assert(!directComposerMarkup.includes(session.thread_id), "信息收口后 composer 不应常驻显示 session id");
  assert(!directComposerMarkup.includes("会话ID"), "信息收口后 composer 不应常驻显示 session id 字段");
  assert(!directComposerMarkup.includes("manual-relay-boundary-details"), "composer 不应再自带 manual relay 边界折叠");
  assert(!directComposerMarkup.includes("target_cwd_canonical"), "composer 不应显示 relay envelope 原始字段");
  assert(!directComposerMarkup.includes("real_codex_executed"), "composer 不应显示 relay receipt 原始字段");
  const directTextarea = findElement(
    directComposer,
    (element) => element.type === "textarea" && element.props?.["aria-label"] === "输入给 Codex 的任务",
  );
  assert(directTextarea, "M3C06 selector 已获确认的撰写区应有 textarea");
  const directKeyDown = directTextarea.props?.onKeyDown;
  assert(typeof directKeyDown === "function", "M3C06 selector 已获确认的撰写区应接管 Enter 键");
  (directKeyDown as (event: { key: string; shiftKey: boolean; preventDefault: () => void }) => void)({
    key: "Enter",
    shiftKey: false,
    preventDefault() {},
  });
  assert(directSubmitCount === 1, "M3C06 仅在宿主已确认 continuation 后才调用受守卫发送 handler");

  let newSessionSubmitCount = 0;
  const newSessionComposer = (
    <AgentChatComposer
      draftPrompt="Start a new Codex conversation"
      k2PreviewError={null}
      manualRelayBusy={false}
      manualRelayError={null}
      manualRelayReceipt={null}
      relayDirectSendEnabled={false}
      relayDirectSendBlockedReason="新建会话需要 M3C07 的已验证运行时；当前不会使用旧 transport 创建会话。"
      selectedProjectRoot="/offline-fixture/projects/new-codex-project"
      selectedSession={null}
      sendMode="new_session"
      onChangeDraft={() => {}}
      onOpenDeveloperDetails={() => {}}
      onStopManualRelayAttempt={() => {}}
      onSubmitDraft={() => {
        newSessionSubmitCount += 1;
      }}
    />
  );
  const newSessionMarkup = renderToStaticMarkup(newSessionComposer);
  assert(
    newSessionMarkup.includes('data-send-mode="decision-only"'),
    "M3C06 新建对话在 M3C07 运行时注入前必须保持禁用",
  );
  assert(newSessionMarkup.includes("M3C07"), "M3C06 新建对话禁用原因必须指向后续已验证运行时");
  assert(newSessionMarkup.includes("选择新对话项目"), "P2 新建对话撰写区应提供项目选择器");
  assert(newSessionMarkup.includes("new-codex-project"), "P2 新建对话撰写区应显示项目名");
  assert(!newSessionMarkup.includes("新建对话"), "信息收口后新建对话 composer 不应显示目标说明条");
  assert(!newSessionMarkup.includes("new session"), "信息收口后新建对话 composer 不应显示 raw session 占位");
  const newSessionTextarea = findElement(
    newSessionComposer,
    (element) => element.type === "textarea" && element.props?.["aria-label"] === "输入给 Codex 的任务",
  );
  assert(newSessionTextarea, "P2 新建对话撰写区应有 textarea");
  const newSessionKeyDown = newSessionTextarea.props?.onKeyDown;
  assert(typeof newSessionKeyDown === "function", "P2 新建对话撰写区应接管 Enter 键");
  (newSessionKeyDown as (event: { key: string; shiftKey: boolean; preventDefault: () => void }) => void)({
    key: "Enter",
    shiftKey: false,
    preventDefault() {},
  });
  assert(newSessionSubmitCount === 0, "M3C06 新建对话不得回退到旧 transport 发送 handler");

  let unboundSubmitCount = 0;
  const unboundComposer = (
    <AgentChatComposer
      draftPrompt="Should stay local"
      k2PreviewError={null}
      manualRelayBusy={false}
      manualRelayError={null}
      manualRelayReceipt={null}
      relayDirectSendEnabled={false}
      relayDirectSendBlockedReason="未绑定会话"
      selectedProjectRoot={session.project_root ?? ""}
      selectedSession={null}
      onChangeDraft={() => {}}
      onOpenDeveloperDetails={() => {}}
      onStopManualRelayAttempt={() => {}}
      onSubmitDraft={() => {
        unboundSubmitCount += 1;
      }}
    />
  );
  const unboundTextarea = findElement(
    unboundComposer,
    (element) => element.type === "textarea" && element.props?.["aria-label"] === "输入给 Codex 的任务",
  );
  assert(unboundTextarea, "B2 非绑定撰写区仍应显示 textarea");
  const unboundKeyDown = unboundTextarea.props?.onKeyDown;
  assert(typeof unboundKeyDown === "function", "B2 非绑定撰写区应接管 Enter 键");
  (unboundKeyDown as (event: { key: string; shiftKey: boolean; preventDefault: () => void }) => void)({
    key: "Enter",
    shiftKey: false,
    preventDefault() {},
  });
  assert(unboundSubmitCount === 0, "B2 非绑定会话 Enter 不得触发 direct relay");

  const codexUserSourceSession: SessionRecord = { ...session, thread_source: "user" };
  const codexUserRelayBinding = deriveRelayBindingState(codexUserSourceSession);
  assert(
    codexUserRelayBinding.targetProjectRoot === session.project_root,
    "P1 Codex sqlite thread_source=user 会话只可保留项目根目录作为服务端读取提示",
  );
  assert(
    agentRoleSessionContinuationBlockedReason(undefined, codexUserRelayBinding.targetProjectRoot)?.includes("仅供阅读"),
    "M3C06 thread_source=user 不得充当 selector、角色或权限真源",
  );

  let nonCodexSubmitCount = 0;
  const nonCodexSession: SessionRecord = { ...session, thread_source: "claude-code" };
  const nonCodexComposer = (
    <AgentChatComposer
      draftPrompt="Should stay blocked"
      k2PreviewError={null}
      manualRelayBusy={false}
      manualRelayError={null}
      manualRelayReceipt={null}
      relayDirectSendEnabled={false}
      relayDirectSendBlockedReason="仅 Codex 会话可用"
      selectedProjectRoot={session.project_root ?? ""}
      selectedSession={nonCodexSession}
      onChangeDraft={() => {}}
      onOpenDeveloperDetails={() => {}}
      onStopManualRelayAttempt={() => {}}
      onSubmitDraft={() => {
        nonCodexSubmitCount += 1;
      }}
    />
  );
  const nonCodexMarkup = renderToStaticMarkup(nonCodexComposer);
  assert(nonCodexMarkup.includes("仅 Codex 会话可用"), "B2 非 Codex 会话必须显示 direct relay 阻断原因");
  const nonCodexTextarea = findElement(
    nonCodexComposer,
    (element) => element.type === "textarea" && element.props?.["aria-label"] === "输入给 Codex 的任务",
  );
  assert(nonCodexTextarea, "B2 非 Codex 会话仍应显示 textarea");
  const nonCodexKeyDown = nonCodexTextarea.props?.onKeyDown;
  assert(typeof nonCodexKeyDown === "function", "B2 非 Codex 会话应接管 Enter 键");
  (nonCodexKeyDown as (event: { key: string; shiftKey: boolean; preventDefault: () => void }) => void)({
    key: "Enter",
    shiftKey: false,
    preventDefault() {},
  });
  assert(nonCodexSubmitCount === 0, "B2 非 Codex 会话 Enter 不得触发 direct relay");

  const deniedMaterialComposerMarkup = renderToStaticMarkup(
    <AgentChatComposer
      draftPrompt="show me .codex full transcript"
      k2PreviewError={null}
      manualRelayBusy={false}
      manualRelayError="manual_relay_guard_blocked:manual_relay_denied_material_requested"
      manualRelayReceipt={null}
      relayDirectSendEnabled={true}
      relayDirectSendBlockedReason={null}
      selectedProjectRoot={session.project_root ?? ""}
      selectedSession={session}
      onChangeDraft={() => {}}
      onOpenDeveloperDetails={() => {}}
      onStopManualRelayAttempt={() => {}}
      onSubmitDraft={() => {}}
    />,
  );
  assert(deniedMaterialComposerMarkup.includes("敏感材料"), "guard 阻断主提示必须转成人话");
  assert(deniedMaterialComposerMarkup.includes("查看开发者详情"), "guard 阻断主提示必须提供诊断入口");
  assert(!deniedMaterialComposerMarkup.includes("manual_relay_guard_blocked"), "主提示不得直接显示 raw guard code");
  assert(!deniedMaterialComposerMarkup.includes("manual_relay_denied_material_requested"), "主提示不得直接显示 raw reason code");

  const relayDiagnosticsMarkup = renderToStaticMarkup(
    <AgentManualRelayDeveloperDetails
      manualRelayError="manual_relay_guard_blocked:manual_relay_denied_material_requested"
      manualRelayPreview={manualRelayPreviewFixture(session)}
      manualRelayReceipt={manualRelayRunningReceiptFixture()}
    />,
  );
  assert(relayDiagnosticsMarkup.includes("Manual relay exact payload fixture"), "开发者详情必须保留 relay exact payload");
  assert(relayDiagnosticsMarkup.includes(session.project_root ?? ""), "开发者详情必须保留 target project/cwd");
  assert(relayDiagnosticsMarkup.includes(session.thread_id), "开发者详情必须保留 target session");
  assert(relayDiagnosticsMarkup.includes("allowed_write_roots"), "开发者详情必须保留 allowed write roots");
  assert(relayDiagnosticsMarkup.includes("manual_once / auto_chain=false"), "开发者详情必须保留一次一发策略");
  assert(relayDiagnosticsMarkup.includes("path_verified"), "开发者详情必须保留路径校验结果");
  assert(relayDiagnosticsMarkup.includes("real_codex_executed=false"), "开发者详情必须保留真实 Codex 执行状态");
  assert(relayDiagnosticsMarkup.includes("process_kind=fixture"), "开发者详情必须保留进程类型");
  assert(relayDiagnosticsMarkup.includes("real_process_killed=false"), "开发者详情必须保留 kill 状态");
  assert(
    relayDiagnosticsMarkup.includes("manual_relay_denied_material_requested"),
    "开发者详情必须保留原始 guard reason",
  );
  assert(relayDiagnosticsMarkup.includes("索取凭据"), "开发者详情必须同时显示 guard reason 人话");

  const relayRunningComposer = (
    <AgentChatComposer
      draftPrompt=""
      k2PreviewError={null}
      manualRelayBusy={false}
      manualRelayError={null}
      manualRelayReceipt={manualRelayRunningReceiptFixture()}
      relayDirectSendEnabled={true}
      relayDirectSendBlockedReason={null}
      selectedProjectRoot={session.project_root ?? ""}
      selectedSession={session}
      onChangeDraft={() => {}}
      onOpenDeveloperDetails={() => {}}
      onStopManualRelayAttempt={() => {}}
      onSubmitDraft={() => {}}
    />
  );
  const relayRunningTextarea = findElement(
    relayRunningComposer,
    (element) => element.type === "textarea" && element.props?.["aria-label"] === "输入给 Codex 的任务",
  );
  assert(relayRunningTextarea?.props?.value === "", "manual relay 触发 run 后输入框应立即清空");
  assert(relayRunningTextarea?.props?.readOnly === true, "manual relay running 时 textarea 应锁定键盘输入");
  assert(findButtonByText(relayRunningComposer, "发送")?.props?.disabled === true, "manual relay running 时普通发送应禁用");
  assert(
    findButtonByText(relayRunningComposer, "Stop")?.props?.disabled !== true,
    "manual relay running 时 stop 按钮应可点击",
  );
  assert(
    findButtonByText(relayRunningComposer, "Stop")?.props?.disabled !== true,
    "manual relay running 时 Stop 必须保持可点击",
  );
  const relayPausedComposer = (
    <AgentChatComposer
      draftPrompt=""
      k2PreviewError={null}
      manualRelayBusy={false}
      manualRelayError="状态刷新连续失败，已暂停轮询。"
      manualRelayPollingPaused
      manualRelayReceipt={manualRelayRunningReceiptFixture()}
      relayDirectSendEnabled={true}
      relayDirectSendBlockedReason={null}
      selectedProjectRoot={session.project_root ?? ""}
      selectedSession={session}
      onChangeDraft={() => {}}
      onOpenDeveloperDetails={() => {}}
      onResumeManualRelayPolling={() => {}}
      onStopManualRelayAttempt={() => {}}
      onSubmitDraft={() => {}}
    />
  );
  assert(findButtonByText(relayPausedComposer, "恢复轮询"), "P4 轮询失败暂停后必须提供 Stop 以外恢复路");

  const relayTerminalComposer = (
    <AgentChatComposer
      draftPrompt="Manual relay next prompt"
      k2PreviewError={null}
      manualRelayBusy={false}
      manualRelayError={null}
      manualRelayReceipt={manualRelayCompletedReceiptFixture()}
      relayDirectSendEnabled={true}
      relayDirectSendBlockedReason={null}
      selectedProjectRoot={session.project_root ?? ""}
      selectedSession={session}
      onChangeDraft={() => {}}
      onOpenDeveloperDetails={() => {}}
      onStopManualRelayAttempt={() => {}}
      onSubmitDraft={() => {}}
    />
  );
  assert(
    findButtonByText(relayTerminalComposer, "发送")?.props?.disabled !== true,
    "manual relay terminal 后普通发送应恢复",
  );
  const relayTerminalTextarea = findElement(
    relayTerminalComposer,
    (element) => element.type === "textarea" && element.props?.["aria-label"] === "输入给 Codex 的任务",
  );
  assert(relayTerminalTextarea?.props?.readOnly !== true, "manual relay terminal 后 textarea 应恢复输入");
  assert(
    !findButtonByText(relayTerminalComposer, "Stop"),
    "manual relay terminal 后 Stop 不应占用主路径",
  );

  const pendingMessage = buildPendingUserMessage({
    prompt: "M3 optimistic send fixture",
    threadId: session.thread_id,
  });
  const optimisticTranscript = appendPendingUserMessage(transcript, pendingMessage);
  const optimisticText = visibleText(<ChatTranscript transcript={optimisticTranscript} />);
  assert(optimisticText.includes("M3 optimistic send fixture"), "M3 发送后应立即冒泡用户消息");
  assert(pendingMessage.metadata?.conversation_engine_send_mode === "decision_only", "M3 pending 消息必须标记为 decision-only");
  assert(pendingMessage.metadata?.real_codex_executed === false, "M3 pending 消息不得声明真实 Codex 执行");
  const repeatedPendingMessage = buildPendingUserMessage({
    createdAt: "2026-06-17T01:00:00Z",
    prompt: "M3 optimistic send fixture",
    threadId: session.thread_id,
  });
  const nextRepeatedPendingMessage = buildPendingUserMessage({
    createdAt: "2026-06-17T01:00:01Z",
    prompt: "M3 optimistic send fixture",
    threadId: session.thread_id,
  });
  assert(
    repeatedPendingMessage.event_id !== nextRepeatedPendingMessage.event_id,
    "M3 相同 prompt 连续发送也不应生成重复 pending event_id",
  );

  const relayPendingMessage = buildManualRelayPendingUserMessage({
    confirmationId: "manual-relay-confirmation:fixture",
    prompt: "Manual relay exact payload fixture",
    promptSha256: "a".repeat(64),
    relayAttemptId: "manual-relay-attempt:fixture",
    targetProjectRoot: session.project_root ?? "",
    targetSessionId: session.thread_id,
    threadId: session.thread_id,
  });
  assert(
    relayPendingMessage.metadata?.conversation_engine_send_mode === "manual_relay_confirmed_once",
    "manual relay pending 消息必须使用 relay 专属模式",
  );
  assert(relayPendingMessage.metadata?.auto_chain === false, "manual relay pending 消息必须钉死 auto_chain=false");
  assert(relayPendingMessage.metadata?.real_codex_executed === false, "manual relay fixture pending 不得声明真实执行");
  const relayAssistantMessage = buildManualRelayAssistantMessage({
    assistantItemId: "item-json-fixture",
    promptSha256: "a".repeat(64),
    relayAttemptId: "manual-relay-attempt:fixture",
    text: "JSON_EVENT_REPLY_OK",
    threadId: session.thread_id,
    usage: { input_tokens: 10, cached_input_tokens: 2, output_tokens: 4, reasoning_output_tokens: 1 },
  });
  assert(relayAssistantMessage.event_type === "assistant_message", "manual relay assistant reply 必须成为助手消息");
  assert(relayAssistantMessage.text === "JSON_EVENT_REPLY_OK", "manual relay assistant reply 必须保留事件流文本");
  assert(
    relayAssistantMessage.metadata?.conversation_engine_send_mode === "manual_relay_thread_event_reply",
    "manual relay assistant reply 必须标明来自 ThreadEvent",
  );
  assert(relayAssistantMessage.metadata?.real_codex_executed === true, "manual relay assistant reply 必须标明真实 codex 已执行");

  const relayLiveEvents = buildManualRelayLiveTranscriptEvents({
    liveEvents: [
      {
        sequence: 1,
        event_type: "turn.started",
        thread_id: session.thread_id,
        item_id: null,
        item_type: null,
        title: "Codex 开始处理",
        text: null,
        delta: null,
        tool_name: null,
        arguments_preview: null,
        output_preview: null,
        stdout: null,
        stderr: null,
        exit_code: null,
        status: "running",
      },
      {
        sequence: 2,
        event_type: "item.updated",
        thread_id: session.thread_id,
        item_id: "item-live-fixture",
        item_type: "agent_message",
        title: "Codex 正在回复",
        text: "P4 live partial",
        delta: null,
        tool_name: null,
        arguments_preview: null,
        output_preview: null,
        stdout: null,
        stderr: null,
        exit_code: null,
        status: "running",
      },
    ],
    relayAttemptId: "manual-relay-attempt:live",
    threadId: session.thread_id,
  });
  const relayLiveTranscript = {
    ...transcript,
    events: [...transcript.events, ...relayLiveEvents],
  };
  const relayLiveMarkup = renderToStaticMarkup(<ChatTranscript transcript={relayLiveTranscript} />);
  const relayLiveText = visibleText(<ChatTranscript transcript={relayLiveTranscript} />);
  assert(relayLiveMarkup.includes("codex-status-item"), "P4 live turn 状态应作为 Codex 状态行渲染");
  assert(relayLiveMarkup.includes("data-streaming=\"true\""), "P4 live assistant 应标记为末轮流式输出");
  assert(relayLiveText.includes("开始处理"), "P4 live turn 状态应显示友好的运行标题");
  assert(relayLiveText.includes("P4 live partial"), "P4 live assistant partial 应显示在对话尾部");
  const canonicalHistoryWithLive = {
    ...transcript,
    events: [
      {
        event_id: "canonical-user",
        timestamp: "2026-06-17T01:01:00Z",
        event_type: "user_message",
        actor: "user",
        text: "Canonical user",
        metadata: { raw_type: "event_msg" },
        warnings: [],
      },
      {
        event_id: "canonical-assistant",
        timestamp: "2026-06-17T01:01:01Z",
        event_type: "assistant_message",
        actor: "assistant",
        text: "Canonical assistant",
        metadata: { raw_type: "event_msg" },
        warnings: [],
      },
      ...relayLiveEvents,
    ],
  };
  const canonicalLiveText = visibleText(<ChatTranscript transcript={canonicalHistoryWithLive} />);
  assert(canonicalLiveText.includes("P4 live partial"), "P4 live assistant 不应被 event_msg 规范历史过滤掉");
}

function manualRelayPreviewFixture(session: SessionRecord) {
  const projectRoot = session.project_root ?? "/tmp/offline";
  return {
    envelope: {
      relay_id: "manual-relay:fixture",
      target_binding: {
        project_root_canonical: projectRoot,
        target_cwd_canonical: projectRoot,
        target_session_id: session.thread_id,
        new_session: false,
        sandbox: "workspace-write",
        allowed_write_roots: [projectRoot],
        target_hash: "b".repeat(64),
        path_verified: true,
      },
      payload: {
        original_user_text: "Manual relay exact payload fixture",
        effective_prompt: "Manual relay exact payload fixture",
        payload_layers: [],
        prompt_sha256: "a".repeat(64),
        prompt_length_bytes: 34,
        exact_original: true,
      },
      policy: {
        manual_once: true,
        auto_chain: false,
        duplicate_scope: "manual-relay:fixture",
        denied_material_policy: "deny_secret_token_env_keychain_oauth_credential_full_transcript_rollout_codex_home",
      },
      future_hooks: {
        role_id: null,
        task_package_ref: null,
        memory_packet_ref: null,
        supervisor_review_ref: null,
        post_run_memory_capture_policy: null,
      },
      audit_refs: ["audit:manual-relay-fixture"],
      receipt_refs: [],
    },
    guard: {
      status: "ready_fixture_only",
      blocks_execution: false,
      reasons: [],
      warnings: ["manual_relay_fixture_only_no_real_codex"],
      command_plan: {
        program: "codex",
        argv: ["exec", "resume", session.thread_id, "--output-last-message", "<workbench-managed-last-message>"],
        stdin_prompt_ref: "manual-relay-prompt",
        stdin_prompt_sha256: "a".repeat(64),
        prompt_in_command: false,
        shell_invocation: false,
        redacted_preview: "codex exec resume <session> <stdin prompt>",
        last_message_path: "/tmp/codex-governance-workbench/manual-relay-runs/fixture/last-message.txt",
      },
    },
  };
}

function manualRelayRunningReceiptFixture() {
  return {
    relay_attempt_id: "manual-relay-attempt:fixture",
    confirmation_id: "manual-relay-confirmation:fixture",
    target: {
      project_root_canonical: "/tmp/offline",
      target_cwd_canonical: "/tmp/offline",
      target_session_id: "offline-thread",
      new_session: false,
      sandbox: "workspace-write",
      allowed_write_roots: ["/tmp/offline"],
      target_hash: "b".repeat(64),
      path_verified: true,
    },
    effective_prompt_sha256: "a".repeat(64),
    prompt_length_bytes: 34,
    prompt_exact_original: true,
    command_plan: {
      program: "codex",
      argv: ["exec", "resume", "offline-thread"],
      stdin_prompt_ref: "manual-relay-prompt",
      stdin_prompt_sha256: "a".repeat(64),
      prompt_in_command: false,
      shell_invocation: false,
      redacted_preview: "codex exec resume <session> <stdin prompt>",
      last_message_path: "/tmp/codex-governance-workbench/manual-relay-runs/fixture/last-message.txt",
    },
    started_at: "2026-06-17T01:00:00Z",
    ended_at: null,
    exit_code: null,
    process_id: null,
    process_kind: "fixture",
    real_process_killed: false,
    status: "running",
    prompt_sent: false,
    real_codex_executed: false,
    syn_read_codex_home: false,
    syn_wrote_codex_home: false,
    killed_by_user: false,
    timed_out: false,
    readback_status: "not_attempted_running_fixture",
    assistant_message_text: null,
    thread_event_summary: {
      thread_id: null,
      assistant_item_id: null,
      assistant_message_text: null,
      turn_completed: false,
      turn_failed: false,
      usage: {},
      event_types: [],
      json_line_count: 0,
      malformed_json_line_count: 0,
      stderr_summary: null,
    },
    live_events: [],
    last_message_hash: null,
    last_message_size_bytes: null,
    changed_files: [],
    git_head_before: "fixture-head-before",
    git_head_after: null,
    git_status_before: "clean_fixture",
    git_status_after: "clean_fixture",
    rollback: {
      git_available: true,
      dirty_before: false,
      auto_rollback_performed: false,
      rollback_suggestion_available: true,
      summary: "fixture only",
    },
    warnings: ["manual_relay_fixture_runner_only"],
  };
}

function manualRelayCompletedReceiptFixture() {
  return {
    ...manualRelayRunningReceiptFixture(),
    ended_at: "2026-06-17T01:00:03Z",
    exit_code: 0,
    status: "completed_fixture",
    readback_status: "fixture_last_message_available",
    assistant_message_text: null,
    last_message_hash: "c".repeat(64),
    last_message_size_bytes: 33,
    git_head_after: "fixture-head-after",
  };
}

function buildLargeTranscript(threadId: string, rolloutPath: string, count: number): CodexTranscript {
  return {
    thread_id: threadId,
    rollout_path: rolloutPath,
    project_path: "/tmp/offline",
    title: "Large conversation fixture",
    created_at_ms: 1,
    updated_at_ms: count,
    viewer_boundary: {
      view_kind: "session_history_viewer",
      reads_session_history: true,
      is_execution_readback: false,
      real_execution_readback_performed: false,
      execution_readback_scope: "not_execution_readback",
      warnings: [],
    },
    events: Array.from({ length: count }, (_, index) => ({
      event_id: `message-fixture-${index}`,
      timestamp: `2026-06-17T00:${String(index % 60).padStart(2, "0")}:00Z`,
      event_type: index % 2 === 0 ? "user_message" : "assistant_message",
      actor: index % 2 === 0 ? "user" : "assistant",
      text: `Message fixture ${index}`,
      warnings: [],
    })),
    summary: {
      total_events: count,
      event_type_counts: {
        user_message: Math.ceil(count / 2),
        assistant_message: Math.floor(count / 2),
      },
      unknown_event_count: 0,
      warning_count: 0,
      encrypted_content_event_count: 0,
      sensitive_like_event_count: 0,
    },
    warnings: [],
    source_stats: {},
  };
}
