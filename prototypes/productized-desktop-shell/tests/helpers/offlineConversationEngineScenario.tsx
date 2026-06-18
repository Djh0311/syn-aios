import React from "react";
import { renderToStaticMarkup } from "react-dom/server.browser";
import { assert, findButtonByText, findElement, visibleText } from "./offlineInteractionTestUtils";
import {
  appendPendingUserMessage,
  buildManualRelayPendingUserMessage,
  buildPendingUserMessage,
  mergeOlderTranscriptPage,
} from "../../src/lib/conversationEngine";
import { AgentChatComposer } from "../../src/views/agent/AgentChatComposer";
import { deriveRelayBindingState } from "../../src/views/agent/AgentConversationShell";
import type { CodexTranscript, PendingAction, SessionRecord } from "../../src/lib/types";
import { AgentSessionCenter, ChatTranscript } from "../../src/views/AgentView";

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

  assert(transcriptMarkup.includes("data-conversation-engine=\"virtualized\""), "M1 对话流应声明使用虚拟化引擎");
  assert(transcriptText.includes("虚拟消息窗口"), "M1 对话流应暴露虚拟窗口计数");
  assert(transcriptText.includes("已渲染"), "M1 对话流应显示当前渲染数量");
  assert(transcriptText.includes("Message fixture 179"), "M1 初始窗口应显示最新消息");
  assert(!transcriptMarkup.includes("Message fixture 20"), "M1 大对话不应默认把早期消息全量放进 DOM");

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
  assert(firstLoadText.includes("这不是 0 条结果"), "M1 首次加载态不得暗示读回 0 条");

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
  assert(streamingMarkup.includes("data-streaming-separated=\"true\""), "M2 流式追加应从稳定虚拟窗口分离");
  assert(streamingText.includes("回到底部"), "M2 滚离底部时应有回到底部入口");
  assert(streamingText.includes("Streaming fixture draft"), "M2 流式草稿应作为单条自然流显示");
  assert(!streamingMarkup.includes("streaming-assistant-draft\" style"), "M2 流式草稿不应进入稳定虚拟窗口绝对定位层");

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
  assert(boundedMarkup.includes("加载更早对话"), "M2 有 older cursor 时应显示上滚加载更早入口");

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
  assert(internalOnlyMarkup.includes("这条会话没有可显示的对话"), "M2 内部事件 tail 应显示空态说明");
  assert(internalOnlyMarkup.includes("加载更早对话"), "M2 内部事件 tail 仍应保留加载更早入口");

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
  assert(composerMarkup.includes("data-send-mode=\"manual_relay_direct\""), "B2 绑定会话撰写区应声明 GUI direct relay 模式");
  assert(composerMarkup.includes("↔"), "B2 撰写区必须常驻显示 target 绑定标记");
  assert(composerMarkup.includes(session.project_root ?? ""), "B2 target 常驻区必须显示 canonical project path");
  assert(composerMarkup.includes(session.thread_id), "B2 target 常驻区必须显示指定会话");
  assert(composerMarkup.includes("会话ID"), "B2 target 常驻区必须显式标出 session id 字段");
  assert(composerMarkup.includes("手动一次一发"), "manual relay UI 必须披露 one-shot 边界");
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
  assert(relayBinding.enabled === true, "B2 bind-fix 点开 Codex 会话后应立即启用 direct relay 绑定");
  assert(
    relayBinding.targetProjectRoot !== staleProjectRoot &&
      relayBinding.targetProjectRoot === "/offline-fixture/projects/selected-codex-project",
    "B2 bind-fix relay target 必须跟随选中会话自己的 project_root，不得沿用旧项目选择",
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
      manualRelayPreview={null}
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
  const directTextarea = findElement(
    directComposer,
    (element) => element.type === "textarea" && element.props?.["aria-label"] === "输入给 Codex 的任务",
  );
  assert(directTextarea, "B2 直发撰写区应有 textarea");
  const directKeyDown = directTextarea.props?.onKeyDown;
  assert(typeof directKeyDown === "function", "B2 直发撰写区应接管 Enter 键");
  (directKeyDown as (event: { key: string; shiftKey: boolean; preventDefault: () => void }) => void)({
    key: "Enter",
    shiftKey: false,
    preventDefault() {},
  });
  assert(directSubmitCount === 1, "B2 绑定会话 Enter 应调用 GUI direct relay 发送 handler");

  let unboundSubmitCount = 0;
  const unboundComposer = (
    <AgentChatComposer
      draftPrompt="Should stay local"
      k2PreviewError={null}
      manualRelayBusy={false}
      manualRelayError={null}
      manualRelayPreview={null}
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

  let nonCodexSubmitCount = 0;
  const nonCodexSession: SessionRecord = { ...session, thread_source: "claude-code" };
  const nonCodexComposer = (
    <AgentChatComposer
      draftPrompt="Should stay blocked"
      k2PreviewError={null}
      manualRelayBusy={false}
      manualRelayError={null}
      manualRelayPreview={null}
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

  const relayPreviewMarkup = renderToStaticMarkup(
    <AgentChatComposer
      draftPrompt="Manual relay exact payload fixture"
      k2PreviewError={null}
      manualRelayBusy={false}
      manualRelayError={null}
      manualRelayPreview={manualRelayPreviewFixture(session)}
      manualRelayReceipt={manualRelayRunningReceiptFixture()}
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
  assert(relayPreviewMarkup.includes("Manual relay exact payload fixture"), "manual relay 预演必须显示 exact payload");
  assert(relayPreviewMarkup.includes(session.project_root ?? ""), "manual relay 预演必须显示 target project/cwd");
  assert(relayPreviewMarkup.includes(session.thread_id), "manual relay 预演必须显示指定 target session");
  assert(relayPreviewMarkup.includes("Write roots"), "manual relay 预演必须显示 allowed write roots");
  assert(relayPreviewMarkup.includes("manual_once / auto_chain=false"), "manual relay 必须显示一次一发且不自动连环");
  assert(relayPreviewMarkup.includes("Path verified"), "manual relay 预演必须显示路径校验结果");
  assert(relayPreviewMarkup.includes("Stop 本 attempt"), "manual relay 必须有可点击 stop 控件");
  assert(!relayPreviewMarkup.includes("确认 mock 中转一次"), "B2 直发 UI 不应出现 mock 二次确认按钮");
  assert(relayPreviewMarkup.includes("real_codex_executed=false"), "manual relay fixture 回执不得声明真实 Codex 执行");
  assert(relayPreviewMarkup.includes("process_kind=fixture"), "manual relay 回执必须显示进程类型");
  assert(relayPreviewMarkup.includes("real_process_killed=false"), "manual relay running fixture 不得伪称已 kill 真进程");

  const relayRunningComposer = (
    <AgentChatComposer
      draftPrompt=""
      k2PreviewError={null}
      manualRelayBusy={false}
      manualRelayError={null}
      manualRelayPreview={manualRelayPreviewFixture(session)}
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
    findButtonByText(relayRunningComposer, "Stop 本 attempt")?.props?.disabled !== true,
    "manual relay running 时 stop 按钮应可点击",
  );
  assert(
    findButtonByText(relayRunningComposer, "Stop 本 attempt")?.props?.disabled !== true,
    "manual relay running 时 Stop 本 attempt 必须保持可点击",
  );

  const relayTerminalComposer = (
    <AgentChatComposer
      draftPrompt="Manual relay next prompt"
      k2PreviewError={null}
      manualRelayBusy={false}
      manualRelayError={null}
      manualRelayPreview={manualRelayPreviewFixture(session)}
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
    findButtonByText(relayTerminalComposer, "Stop 本 attempt")?.props?.disabled === true,
    "manual relay terminal 后 Stop 应禁用",
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
