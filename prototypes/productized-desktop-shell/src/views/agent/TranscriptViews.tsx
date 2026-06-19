import { useEffect, useMemo, useRef, useState } from "react";
import { Badge } from "../../components/Badge";
import { conversationTurns } from "../../lib/conversationTurns";
import type { CodexTranscript, CodexTranscriptEvent } from "../../lib/types";

const VIRTUAL_MESSAGE_WINDOW_SIZE = 12;
const VIRTUAL_MESSAGE_OVERSCAN = 3;
const ESTIMATED_MESSAGE_HEIGHT = 132;

export function TranscriptTimeline({
  olderLoading = false,
  onLoadOlder,
  transcript,
}: {
  olderLoading?: boolean;
  onLoadOlder?: () => void;
  transcript: CodexTranscript;
}) {
  return <ChatTranscript olderLoading={olderLoading} transcript={transcript} onLoadOlder={onLoadOlder} />;
}

export function ChatTranscript({
  olderLoading = false,
  onLoadOlder,
  transcript,
}: {
  olderLoading?: boolean;
  onLoadOlder?: () => void;
  transcript: CodexTranscript;
}) {
  const [showInternal, setShowInternal] = useState(false);
  const scrollRef = useRef<HTMLDivElement | null>(null);
  const previousScrollHeightRef = useRef<number | null>(null);
  const autoRequestedOlderCursorRef = useRef<number | null>(null);
  const [scrollTop, setScrollTop] = useState(Number.MAX_SAFE_INTEGER);
  const [viewportHeight, setViewportHeight] = useState(720);
  const [isNearBottom, setIsNearBottom] = useState(true);

  const conversationEvents = useMemo(() => conversationTurns(transcript.events), [transcript.events]);
  const conversationIds = useMemo(() => new Set(conversationEvents.map((event) => event.event_id)), [conversationEvents]);
  const displayEvents = useMemo(
    () =>
      transcript.events.filter((event) => {
        if (conversationIds.has(event.event_id)) return true;
        return isCodexNativeStatusEvent(event);
      }),
    [transcript.events, conversationIds],
  );
  const { stableConversation, streamingEvent } = useStreamingSeparatedConversation(displayEvents);
  const virtualWindow = useVirtualMessageWindow(stableConversation, scrollTop, viewportHeight);
  const hiddenConversationCount = stableConversation.length - virtualWindow.visible.length;
  const displayIds = useMemo(() => new Set(displayEvents.map((event) => event.event_id)), [displayEvents]);
  const internalEvents = useMemo(
    () => transcript.events.filter((event) => !displayIds.has(event.event_id)),
    [transcript.events, displayIds],
  );
  const internalCount = internalEvents.length;
  const olderCursor = transcript.pagination?.has_older ? transcript.pagination.older_before_line ?? null : null;
  const hasOlderTranscript = !!olderCursor && !!onLoadOlder;
  const boundedLoadMode = transcript.pagination && transcript.pagination.mode !== "full" ? "bounded" : "full";

  useEffect(() => {
    if (autoRequestedOlderCursorRef.current !== olderCursor) {
      autoRequestedOlderCursorRef.current = null;
    }
  }, [olderCursor]);

  useEffect(() => {
    const node = scrollRef.current;
    if (!node) return;
    node.scrollTop = Math.max(0, stableConversation.length * ESTIMATED_MESSAGE_HEIGHT - node.clientHeight);
    setScrollTop(node.scrollTop);
    setViewportHeight(node.clientHeight || 720);
    setIsNearBottom(true);
  }, [transcript.thread_id]);

  useEffect(() => {
    if (!isNearBottom) return;
    const frame = window.requestAnimationFrame(() => scrollToLatest());
    return () => window.cancelAnimationFrame(frame);
  }, [isNearBottom, stableConversation.length, streamingEvent?.text]);

  useEffect(() => {
    const previousHeight = previousScrollHeightRef.current;
    const node = scrollRef.current;
    if (previousHeight === null || !node) return;
    const delta = node.scrollHeight - previousHeight;
    if (delta > 0) {
      node.scrollTop += delta;
      setScrollTop(node.scrollTop);
    }
    previousScrollHeightRef.current = null;
  }, [transcript.events.length]);

  function handleScroll(event: React.UIEvent<HTMLDivElement>) {
    const target = event.currentTarget;
    setScrollTop(target.scrollTop);
    setViewportHeight(target.clientHeight || 720);
    setIsNearBottom(target.scrollHeight - target.scrollTop - target.clientHeight < 100);
    if (target.scrollTop < 24) requestOlderTranscript(false);
  }

  function scrollToLatest() {
    const node = scrollRef.current;
    if (!node) return;
    node.scrollTop = node.scrollHeight;
    setScrollTop(node.scrollTop);
    setIsNearBottom(true);
  }

  function scrollOneWindow(direction: "earlier" | "newer") {
    const node = scrollRef.current;
    if (!node) return;
    const delta = VIRTUAL_MESSAGE_WINDOW_SIZE * ESTIMATED_MESSAGE_HEIGHT;
    node.scrollTop = Math.max(0, node.scrollTop + (direction === "earlier" ? -delta : delta));
    setScrollTop(node.scrollTop);
  }

  function requestOlderTranscript(manual: boolean) {
    if (!hasOlderTranscript || olderLoading || !olderCursor) return;
    if (!manual && autoRequestedOlderCursorRef.current === olderCursor) return;
    const node = scrollRef.current;
    previousScrollHeightRef.current = node ? node.scrollHeight : null;
    autoRequestedOlderCursorRef.current = olderCursor;
    onLoadOlder?.();
  }

  const transcriptPageBoundary = hasOlderTranscript ? (
    <button
      className="secondary-button chat-load-older"
      disabled={olderLoading}
      type="button"
      onClick={() => requestOlderTranscript(true)}
    >
      {olderLoading ? "正在加载更早对话" : "加载更早对话"}
    </button>
  ) : transcript.pagination?.mode && transcript.pagination.mode !== "full" ? (
    <p className="session-reader-boundary">已到达这条对话的最早可读片段。</p>
  ) : null;

  return (
    <section className="transcript-shell">
      {displayEvents.length === 0 ? (
        <>
          <section className="empty-state">
            <strong>这条会话没有可显示的对话</strong>
            <span>如果需要排查工具调用、上下文或系统事件，请打开开发者详情。</span>
          </section>
          {transcriptPageBoundary}
        </>
      ) : (
        <div
          className="chat-stream"
          data-conversation-engine="virtualized"
          data-transcript-load={boundedLoadMode}
          onScroll={handleScroll}
          ref={scrollRef}
        >
          {transcriptPageBoundary}
          {hiddenConversationCount > 0 ? (
            <div className="chat-fold-notice" data-virtualized-window={`${virtualWindow.visible.length}/${displayEvents.length}`}>
              <span>
                已收纳较早 {hiddenConversationCount} 条对话
              </span>
              {virtualWindow.start > 0 ? (
                <button className="secondary-button" type="button" onClick={() => scrollOneWindow("earlier")}>
                  看更早
                </button>
              ) : null}
              {virtualWindow.end < displayEvents.length ? (
                <button className="secondary-button" type="button" onClick={() => scrollOneWindow("newer")}>
                  查看更新
                </button>
              ) : null}
              <button className="secondary-button" type="button" onClick={scrollToLatest}>
                回到最新
              </button>
            </div>
          ) : null}
          <div
            className="chat-virtual-spacer"
            data-stick-to-bottom={isNearBottom ? "true" : "false"}
            data-streaming-separated={streamingEvent ? "true" : "false"}
            style={{ height: virtualWindow.totalHeight, position: "relative" }}
          >
            <div
              className="chat-virtual-window"
              style={{
                display: "flex",
                flexDirection: "column",
                gap: 18,
                transform: `translateY(${virtualWindow.offsetTop}px)`,
              }}
            >
              {virtualWindow.visible.map((event) => (
                <CodexTranscriptItem event={event} key={event.event_id} />
              ))}
            </div>
          </div>
          {streamingEvent ? (
            <div className="chat-streaming-tail" data-streaming-separated="true" style={{ display: "flex", flexDirection: "column", gap: 18 }}>
              <CodexTranscriptItem event={streamingEvent} />
            </div>
          ) : null}
          {!isNearBottom || streamingEvent ? (
            <button
              className="chat-return-bottom"
              style={{ alignSelf: "center", bottom: 12, position: "sticky", zIndex: 2 }}
              type="button"
              onClick={scrollToLatest}
            >
              回到底部
            </button>
          ) : null}
        </div>
      )}

      <div className="chat-toolbar">
        <span className="counts"><em>{displayEvents.length}</em> 条对话项</span>
      </div>

      {internalCount > 0 ? (
        <details
          className="agent-session-dev-details transcript-dev-details"
          open={showInternal}
          onToggle={(event) => setShowInternal(event.currentTarget.open)}
        >
          <summary>过程事件（{internalCount}）</summary>
          {showInternal ? (
            <section className="internal-events">
              <header className="ie-head">过程事件 · {internalCount}</header>
              <div className="timeline-list">
                {internalEvents.map((event) => (
                  <TranscriptEventCard event={event} key={event.event_id} />
                ))}
              </div>
            </section>
          ) : (
            <p className="session-reader-boundary">打开后显示工具调用、上下文和系统事件。</p>
          )}
        </details>
      ) : null}
    </section>
  );
}

function useStreamingSeparatedConversation(events: CodexTranscriptEvent[]) {
  return useMemo(() => {
    const last = events[events.length - 1];
    if (last && metadataFlag(last, "conversation_engine_streaming")) {
      return {
        stableConversation: events.slice(0, -1),
        streamingEvent: last,
      };
    }
    return {
      stableConversation: events,
      streamingEvent: null,
    };
  }, [events]);
}

function metadataFlag(event: CodexTranscriptEvent, key: string): boolean {
  const metadata = event.metadata;
  return !!metadata && typeof metadata === "object" && !Array.isArray(metadata) && metadata[key] === true;
}

function metadataValue(event: CodexTranscriptEvent, key: string): unknown {
  const metadata = event.metadata;
  if (!metadata || typeof metadata !== "object" || Array.isArray(metadata)) return null;
  return metadata[key];
}

function payloadTypeOf(event: CodexTranscriptEvent): string | null {
  const payloadType = metadataValue(event, "payload_type");
  return typeof payloadType === "string" ? payloadType : null;
}

function metadataString(event: CodexTranscriptEvent, key: string): string | null {
  const value = metadataValue(event, key);
  return typeof value === "string" ? value : null;
}

function isCodexNativeStatusEvent(event: CodexTranscriptEvent): boolean {
  if (metadataFlag(event, "conversation_engine_live_status")) return true;
  if (event.event_type === "tool_call") return true;
  if (event.event_type === "tool_result") return true;
  if (event.event_type === "command_output") return true;
  if (event.event_type === "compacted") return true;
  return event.event_type === "system_context" && payloadTypeOf(event) === "reasoning";
}

function useVirtualMessageWindow(events: CodexTranscriptEvent[], scrollTop: number, viewportHeight: number) {
  return useMemo(() => {
    const visibleByHeight = Math.ceil(viewportHeight / ESTIMATED_MESSAGE_HEIGHT) + VIRTUAL_MESSAGE_OVERSCAN * 2;
    const windowSize = Math.max(VIRTUAL_MESSAGE_WINDOW_SIZE, visibleByHeight);
    const firstVisible = Math.max(0, Math.floor(scrollTop / ESTIMATED_MESSAGE_HEIGHT) - VIRTUAL_MESSAGE_OVERSCAN);
    const latestStart = Math.max(0, events.length - windowSize);
    const start = Math.min(firstVisible, latestStart);
    const end = Math.min(events.length, start + windowSize);
    return {
      start,
      end,
      offsetTop: start * ESTIMATED_MESSAGE_HEIGHT,
      totalHeight: events.length * ESTIMATED_MESSAGE_HEIGHT,
      visible: events.slice(start, end),
    };
  }, [events, scrollTop, viewportHeight]);
}

function CodexTranscriptItem({ event }: { event: CodexTranscriptEvent }) {
  const [expanded, setExpanded] = useState(false);
  if (isCodexNativeStatusEvent(event) && event.event_type !== "user_message" && event.event_type !== "assistant_message") {
    return <CodexStatusItem event={event} />;
  }
  const role = event.event_type === "user_message" ? "user" : "assistant";
  const speaker = role === "user" ? "你" : "";
  const text = event.text || valuePreview(event.output) || "（无正文）";
  const longMessage = text.length > 680 || text.split(/\r?\n/).length > 10;
  const liveClass = metadataFlag(event, "conversation_engine_streaming") ? " streaming" : "";
  const timestamp = event.timestamp ?? undefined;
  const displayTime = role === "user" && timestamp ? compactTimestamp(timestamp) : null;
  return (
    <article className={`codex-transcript-item ${role}${liveClass}`}>
      {speaker || displayTime ? (
        <header className="who">
          {speaker ? <strong>{speaker}</strong> : null}
          {displayTime ? <time dateTime={timestamp}>{displayTime}</time> : null}
        </header>
      ) : null}
      <div className={`body ${longMessage && !expanded ? "is-collapsed" : ""}`}>
        <MessageBody text={text} />
      </div>
      {longMessage ? (
        <button className="chat-expand" type="button" onClick={() => setExpanded((value) => !value)}>
          {expanded ? "收起" : "展开"}
        </button>
      ) : null}
    </article>
  );
}

function CodexStatusItem({ event }: { event: CodexTranscriptEvent }) {
  const liveClass = metadataFlag(event, "conversation_engine_live_status") ? " live" : "";
  const timestamp = event.timestamp ?? undefined;
  const displayTime = timestamp ? compactTimestamp(timestamp) : null;
  if (event.event_type === "compacted") {
    return (
      <article className={`codex-status-item compacted${liveClass}`}>
        <div className="codex-status-line">
          <span>上下文已自动压缩</span>
          {displayTime ? <time dateTime={timestamp}>{displayTime}</time> : null}
        </div>
        {event.text ? <p>{event.text}</p> : null}
      </article>
    );
  }
  if (payloadTypeOf(event) === "reasoning") {
    const label = metadataString(event, "live_title") ?? "思考";
    return (
      <details className={`codex-status-item reasoning${liveClass}`}>
        <summary>
          <span>{label}</span>
          {displayTime ? <time dateTime={timestamp}>{displayTime}</time> : null}
        </summary>
        <p>{event.text || (event.warnings.includes("encrypted_content_omitted") ? "思考内容已由 Codex 加密存储，当前仅显示占位。" : "无可展示正文")}</p>
        {event.warnings.length > 0 ? <WarningStrip warnings={event.warnings} compact /> : null}
      </details>
    );
  }
  const label = statusLabelForEvent(event);
  const detail = statusDetailForEvent(event);
  return (
    <details className={`codex-status-item ${event.event_type ?? "event"}${liveClass}`}>
      <summary>
        <span>{label}</span>
        {detail ? <em>{detail}</em> : null}
        {displayTime ? <time dateTime={timestamp}>{displayTime}</time> : null}
      </summary>
      <StatusEventDetails event={event} />
    </details>
  );
}

function StatusEventDetails({ event }: { event: CodexTranscriptEvent }) {
  const body = event.text || valuePreview(event.output) || event.stdout || event.stderr || valuePreview(event.arguments) || "";
  return (
    <div className="codex-status-details">
      {body ? <p>{body}</p> : null}
      {!isEmptyValue(event.arguments) ? <pre>arguments: {valuePreview(event.arguments)}</pre> : null}
      {!isEmptyValue(event.output) ? <pre>output: {valuePreview(event.output)}</pre> : null}
      {event.stdout ? <pre>stdout: {event.stdout}</pre> : null}
      {event.stderr ? <pre>stderr: {event.stderr}</pre> : null}
      {event.exit_code !== null && event.exit_code !== undefined ? <pre>exit_code: {String(event.exit_code)}</pre> : null}
      {event.warnings.length > 0 ? <WarningStrip warnings={event.warnings} compact /> : null}
    </div>
  );
}

function statusLabelForEvent(event: CodexTranscriptEvent): string {
  const liveTitle = metadataString(event, "live_title");
  if (liveTitle) return friendlyLiveTitle(liveTitle);
  if (event.event_type === "tool_call") {
    if (event.tool_name?.includes("exec_command")) return "准备运行命令";
    if (event.tool_name?.includes("apply_patch")) return "准备编辑文件";
    return "准备调用工具";
  }
  if (event.event_type === "command_output") {
    if (event.tool_name === "apply_patch") return "已应用补丁";
    return exitCodeIsSuccess(event.exit_code) ? "已运行命令" : "命令已返回";
  }
  if (event.event_type === "tool_result") return "工具结果";
  return labelForEvent(event.event_type);
}

function statusDetailForEvent(event: CodexTranscriptEvent): string {
  const liveStatus = metadataString(event, "live_status");
  const liveEventType = metadataString(event, "live_event_type");
  if (liveStatus || liveEventType) return friendlyLiveDetail(event, liveStatus, liveEventType);
  if (event.event_type === "tool_call") {
    const command = commandFromArguments(event.arguments);
    if (command) return command;
    return event.tool_name || "";
  }
  if (event.event_type === "command_output") {
    const exit = event.exit_code !== null && event.exit_code !== undefined ? `exit ${String(event.exit_code)}` : "";
    const stdout = event.stdout?.trim();
    if (stdout) return exit ? `${exit} · ${firstLine(stdout)}` : firstLine(stdout);
    return exit;
  }
  if (event.event_type === "tool_result") return event.tool_name || event.call_id || "";
  return event.text ? firstLine(event.text) : "";
}

function friendlyLiveTitle(title: string): string {
  const labels: Record<string, string> = {
    "Codex 开始处理": "开始处理",
    "Codex 正在回复": "正在回复",
    "Codex 回复完成": "回复完成",
    "Codex 完成": "完成",
    "Codex 失败": "失败",
    "对话已创建": "已创建对话",
    "思考中": "正在思考",
    "思考完成": "思考完成",
    "正在运行命令": "正在运行命令",
    "命令完成": "命令完成",
    "正在调用工具": "正在调用工具",
    "工具完成": "工具完成",
    "工具输出": "工具输出",
  };
  return labels[title] ?? title;
}

function friendlyLiveDetail(event: CodexTranscriptEvent, liveStatus: string | null, liveEventType: string | null): string {
  const command = commandFromArguments(event.arguments);
  if (command) return command;
  const stdout = event.stdout?.trim();
  if (stdout) return firstLine(stdout);
  const liveStatusLabel = liveStatus === "running" ? "运行中" : liveStatus === "completed" ? "已完成" : liveStatus === "failed" ? "失败" : "";
  const eventTypeLabel = liveEventType ? liveEventType.replace("item.", "").replace("turn.", "") : "";
  return [liveStatusLabel, eventTypeLabel].filter(Boolean).join(" · ");
}

function commandFromArguments(value: unknown): string {
  if (!value || typeof value !== "object" || Array.isArray(value)) return "";
  const record = value as Record<string, unknown>;
  const cmd = record.cmd;
  if (typeof cmd === "string") return cmd;
  if (Array.isArray(cmd)) return cmd.map(String).join(" ");
  return "";
}

function exitCodeIsSuccess(value: unknown): boolean {
  return value === 0 || value === "0";
}

function firstLine(text: string): string {
  return text.split(/\r?\n/)[0]?.trim() ?? "";
}

function compactTimestamp(timestamp: string): string {
  const date = new Date(timestamp);
  if (Number.isNaN(date.getTime())) return timestamp;
  return date.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
}

type MessagePart =
  | { kind: "text"; text: string }
  | { kind: "code"; text: string; language: string };

function MessageBody({ text }: { text: string }) {
  return (
    <>
      {splitFencedCodeBlocks(text).map((part, index) =>
        part.kind === "code" ? (
          <CodeBlock code={part.text} key={`${part.kind}-${index}`} language={part.language} />
        ) : (
          <span className="message-text-part" key={`${part.kind}-${index}`}>
            {part.text}
          </span>
        ),
      )}
    </>
  );
}

function CodeBlock({ code, language }: { code: string; language: string }) {
  return (
    <figure className="chat-code-block">
      <figcaption>
        <span>{language || "code"}</span>
        <button type="button" onClick={() => void copyText(code)}>
          复制
        </button>
      </figcaption>
      <pre>
        <code>{code}</code>
      </pre>
    </figure>
  );
}

function splitFencedCodeBlocks(text: string): MessagePart[] {
  const parts: MessagePart[] = [];
  const pattern = /```([^\n`]*)\n([\s\S]*?)```/g;
  let cursor = 0;
  let match: RegExpExecArray | null;
  while ((match = pattern.exec(text)) !== null) {
    if (match.index > cursor) {
      parts.push({ kind: "text", text: text.slice(cursor, match.index) });
    }
    parts.push({
      kind: "code",
      language: match[1]?.trim() ?? "",
      text: match[2] ?? "",
    });
    cursor = match.index + match[0].length;
  }
  if (cursor < text.length || parts.length === 0) {
    parts.push({ kind: "text", text: text.slice(cursor) });
  }
  return parts;
}

async function copyText(text: string) {
  if (typeof navigator === "undefined" || !navigator.clipboard?.writeText) return;
  await navigator.clipboard.writeText(text);
}

function TranscriptEventCard({ event }: { event: CodexTranscriptEvent }) {
  const body = event.text || valuePreview(event.output) || event.stdout || event.stderr || valuePreview(event.arguments) || "无可展示正文";
  return (
    <article className={`timeline-event ${event.actor || "system"}`}>
      <div className="timeline-event-head">
        <div>
          <Badge tone={toneForEvent(event.event_type)}>{labelForEvent(event.event_type)}</Badge>
          <strong>{event.tool_name || event.actor || "system"}</strong>
        </div>
        <span>{event.timestamp || event.event_id}</span>
      </div>
      <p>{body}</p>
      {(event.stdout || event.stderr || !isEmptyValue(event.arguments) || !isEmptyValue(event.output)) && (
        <details className="event-details">
          <summary>工具和原始字段</summary>
          {!isEmptyValue(event.arguments) && <pre>arguments: {valuePreview(event.arguments)}</pre>}
          {!isEmptyValue(event.output) && <pre>output: {valuePreview(event.output)}</pre>}
          {event.stdout && <pre>stdout: {event.stdout}</pre>}
          {event.stderr && <pre>stderr: {event.stderr}</pre>}
          {event.exit_code !== null && event.exit_code !== undefined && <pre>exit_code: {String(event.exit_code)}</pre>}
        </details>
      )}
      {event.warnings.length > 0 && <WarningStrip warnings={event.warnings} compact />}
    </article>
  );
}

export function WarningStrip({ warnings, compact = false }: { warnings: string[]; compact?: boolean }) {
  return (
    <div className={`warning-row ${compact ? "compact-warning-row" : ""}`}>
      {warnings.map((warning) => (
        <Badge tone="warning" key={warning}>
          {warning}
        </Badge>
      ))}
    </div>
  );
}

export function readbackCountLabel(value: number | null | undefined) {
  return value === null || value === undefined ? "未知/不可用" : String(value);
}

function labelForEvent(eventType?: string | null) {
  if (eventType === "user_message") return "用户";
  if (eventType === "assistant_message") return "Codex";
  if (eventType === "tool_call") return "工具调用";
  if (eventType === "tool_result") return "工具结果";
  if (eventType === "command_output") return "命令输出";
  if (eventType === "session_meta") return "会话元数据";
  if (eventType === "turn_context") return "轮次上下文";
  if (eventType === "system_context") return "系统上下文";
  if (eventType === "unknown") return "未知事件";
  return eventType || "事件";
}

function toneForEvent(eventType?: string | null): "candidate" | "warning" | "unknown" | "neutral" {
  if (eventType === "unknown") return "warning";
  if (eventType === "tool_call" || eventType === "tool_result" || eventType === "command_output") return "unknown";
  if (eventType === "user_message" || eventType === "assistant_message") return "candidate";
  return "neutral";
}

function valuePreview(value: unknown): string {
  if (isEmptyValue(value)) return "";
  if (typeof value === "string") return value;
  try {
    return JSON.stringify(value, null, 2);
  } catch {
    return String(value);
  }
}

function isEmptyValue(value: unknown) {
  return value === null || value === undefined || value === "" || (Array.isArray(value) && value.length === 0);
}
