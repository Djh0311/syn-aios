import { useEffect, useMemo, useState } from "react";
import { Badge } from "../../components/Badge";
import { conversationTurns } from "../../lib/conversationTurns";
import type { CodexTranscript, CodexTranscriptEvent } from "../../lib/types";

export function TranscriptTimeline({ transcript }: { transcript: CodexTranscript }) {
  return <ChatTranscript transcript={transcript} />;
}

export function ChatTranscript({ transcript }: { transcript: CodexTranscript }) {
  const [showInternal, setShowInternal] = useState(false);
  const [showAllMessages, setShowAllMessages] = useState(false);

  const conversation = useMemo(() => conversationTurns(transcript.events), [transcript.events]);
  const visibleConversation = useMemo(
    () => (showAllMessages ? conversation : conversation.slice(-12)),
    [conversation, showAllMessages],
  );
  const hiddenConversationCount = conversation.length - visibleConversation.length;
  const conversationIds = useMemo(() => new Set(conversation.map((event) => event.event_id)), [conversation]);
  const internalEvents = useMemo(
    () => transcript.events.filter((event) => !conversationIds.has(event.event_id)),
    [transcript.events, conversationIds],
  );
  const internalCount = internalEvents.length;

  useEffect(() => {
    setShowAllMessages(false);
  }, [transcript.thread_id]);

  return (
    <section className="transcript-shell">
      {conversation.length === 0 ? (
        <section className="empty-state">
          <strong>这条会话没有可显示的对话</strong>
          <span>如果需要排查工具调用、上下文或系统事件，请打开开发者详情。</span>
        </section>
      ) : (
        <div className="chat-stream">
          {hiddenConversationCount > 0 ? (
            <div className="chat-fold-notice">
              <span>已收纳较早 {hiddenConversationCount} 条消息</span>
              <button className="secondary-button" type="button" onClick={() => setShowAllMessages(true)}>
                展开全部
              </button>
            </div>
          ) : null}
          {visibleConversation.map((event) => (
            <ChatBubble event={event} key={event.event_id} />
          ))}
        </div>
      )}

      <div className="chat-toolbar">
        <span className="counts"><em>{conversation.length}</em> 条对话</span>
      </div>

      {internalCount > 0 ? (
        <details
          className="agent-session-dev-details transcript-dev-details"
          open={showInternal}
          onToggle={(event) => setShowInternal(event.currentTarget.open)}
        >
          <summary>开发者详情：过程事件（{internalCount}）</summary>
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

function ChatBubble({ event }: { event: CodexTranscriptEvent }) {
  const [expanded, setExpanded] = useState(false);
  const isUser = event.event_type === "user_message";
  const role = isUser ? "user" : "assistant";
  const speaker = isUser ? "你" : "Codex";
  const text = event.text || valuePreview(event.output) || "（无正文）";
  const longMessage = text.length > 680 || text.split(/\r?\n/).length > 10;
  return (
    <article className={`chat-bubble ${role}`}>
      <header className="who">
        <strong>{speaker}</strong>
        {event.timestamp ? <time>{event.timestamp}</time> : null}
      </header>
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
