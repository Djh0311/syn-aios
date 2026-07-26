import type { CodexTranscriptEvent } from "./types";

// This module deliberately owns no React state and no Tauri invocation.  Pages
// supply a small client adapter, which keeps the transport state machine usable
// by both the agent page and Jiaoban without either page choosing permissions.

export const AGENT_CODEX_WORKSPACE_WRITE_PROFILE = "agent-codex-workspace-write" as const;
export const SUPERVISOR_READ_ONLY_PROFILE = "supervisor-read-only" as const;

export type ConversationProfileId =
  | typeof AGENT_CODEX_WORKSPACE_WRITE_PROFILE
  | typeof SUPERVISOR_READ_ONLY_PROFILE;

export type AgentConversationTransportContext = Readonly<{
  profile_id: typeof AGENT_CODEX_WORKSPACE_WRITE_PROFILE;
  project_root: string;
}>;

export type SupervisorConversationTransportContext = Readonly<{
  profile_id: typeof SUPERVISOR_READ_ONLY_PROFILE;
  project_root: string;
  project_id: string;
  workflow_id: string;
}>;

export type ConversationTransportContext =
  | AgentConversationTransportContext
  | SupervisorConversationTransportContext;

export function createAgentConversationTransportContext({
  project_root,
}: {
  project_root: string;
}): AgentConversationTransportContext {
  return Object.freeze({
    profile_id: AGENT_CODEX_WORKSPACE_WRITE_PROFILE,
    project_root: requireIdentifier(project_root, "project_root"),
  });
}

export function createSupervisorConversationTransportContext({
  project_root,
  project_id,
  workflow_id,
}: {
  project_root: string;
  project_id: string;
  workflow_id: string;
}): SupervisorConversationTransportContext {
  return Object.freeze({
    profile_id: SUPERVISOR_READ_ONLY_PROFILE,
    project_root: requireIdentifier(project_root, "project_root"),
    project_id: requireIdentifier(project_id, "project_id"),
    workflow_id: requireIdentifier(workflow_id, "workflow_id"),
  });
}

export type ConversationReceiptLayerStatus =
  | "not_requested"
  | "pending"
  | "succeeded"
  | "failed"
  | "stopped";

export type SupervisorConversationBindingStage =
  | "binding_construct"
  | "binding_store_prepare"
  | "binding_persist_db"
  | "binding_project_json"
  | "binding_activate"
  | "transport_start"
  | "binding_terminate";

export type ConversationReceiptLayer = Readonly<{
  status: ConversationReceiptLayerStatus;
  // Server-provided, user-safe copy only.  Raw tool arguments, stderr, argv,
  // environment, and identity material never belong in this receipt contract.
  human_message: string | null;
}>;

export type ConversationTransportReceipt = Readonly<{
  conversation_id: string | null;
  thread_id: string | null;
  turn_id: string;
  transport: ConversationReceiptLayer & Readonly<{
    attempt_id: string | null;
    binding_stage: SupervisorConversationBindingStage | null;
  }>;
  assistant_reply: ConversationReceiptLayer & Readonly<{
    text: string | null;
    assistant_item_id: string | null;
  }>;
  tool_action: ConversationReceiptLayer;
  read_model_projection: ConversationReceiptLayer;
  canonical_mirror: ConversationReceiptLayer;
}>;

export type ConversationTransportMode = "new" | "existing";

export type ConversationTransportNewTurn = Readonly<{
  mode: "new";
  user_text: string;
  turn_id?: string;
}>;

export type ConversationTransportExistingTurn = Readonly<{
  mode: "existing";
  user_text: string;
  conversation_id: string;
  thread_id: string;
  turn_id?: string;
}>;

export type ConversationTransportTurn = ConversationTransportNewTurn | ConversationTransportExistingTurn;

// The request shape intentionally has no sandbox, write-root, approval, or
// capability fields.  The injected Tauri adapter must route the fixed profile
// to a server-owned command, where those settings are derived again.
export type ConversationTransportNewStartRequest = Readonly<{
  context: ConversationTransportContext;
  mode: "new";
  conversation_id: null;
  thread_id: null;
  turn_id: string;
  user_text: string;
}>;

export type ConversationTransportExistingStartRequest = Readonly<{
  context: ConversationTransportContext;
  mode: "existing";
  conversation_id: string;
  thread_id: string;
  turn_id: string;
  user_text: string;
}>;

export type ConversationTransportAttemptRequest = Readonly<{
  attempt_id: string;
}>;

export type ConversationTransportClient = Readonly<{
  startNew: (request: ConversationTransportNewStartRequest) => Promise<ConversationTransportReceipt>;
  startExisting: (request: ConversationTransportExistingStartRequest) => Promise<ConversationTransportReceipt>;
  poll: (request: ConversationTransportAttemptRequest) => Promise<ConversationTransportReceipt>;
  stop: (request: ConversationTransportAttemptRequest) => Promise<ConversationTransportReceipt>;
}>;

export type ConversationTransportLifecycle = "idle" | "starting" | "running" | "completed" | "failed" | "stopped";

export type ConversationTransportSession = Readonly<{
  conversation_id: string | null;
  thread_id: string | null;
}>;

export type ConversationTransportState = Readonly<{
  context: ConversationTransportContext;
  lifecycle: ConversationTransportLifecycle;
  input_locked: boolean;
  active_attempt_id: string | null;
  session: ConversationTransportSession;
  receipt: ConversationTransportReceipt | null;
  transcript_events: readonly CodexTranscriptEvent[];
  operation_error: string | null;
  start_failure: Readonly<{ turn_id: string; stage: SupervisorConversationBindingStage }> | null;
}>;

export type ConversationTransportController = Readonly<{
  getState: () => ConversationTransportState;
  subscribe: (listener: (state: ConversationTransportState) => void) => () => void;
  start: (turn: ConversationTransportTurn) => Promise<ConversationTransportState>;
  poll: () => Promise<ConversationTransportState>;
  stop: () => Promise<ConversationTransportState>;
  reset: () => ConversationTransportState;
}>;

export function createConversationTransportController({
  context,
  client,
  initial_session,
  initial_transcript_events = [],
  create_turn_id = defaultTurnId,
  now = () => new Date(),
}: {
  context: ConversationTransportContext;
  client: ConversationTransportClient;
  initial_session?: ConversationTransportSession;
  initial_transcript_events?: readonly CodexTranscriptEvent[];
  create_turn_id?: () => string;
  now?: () => Date;
}): ConversationTransportController {
  let state: ConversationTransportState = {
    context,
    lifecycle: "idle",
    input_locked: false,
    active_attempt_id: null,
    session: initial_session ?? { conversation_id: null, thread_id: null },
    receipt: null,
    transcript_events: [...initial_transcript_events],
    operation_error: null,
    start_failure: null,
  };
  const listeners = new Set<(next: ConversationTransportState) => void>();

  function publish(next: ConversationTransportState): ConversationTransportState {
    state = next;
    const snapshot = snapshotOf(state);
    for (const listener of listeners) listener(snapshot);
    return snapshot;
  }

  function finishWithoutRequest(message: string): ConversationTransportState {
    return publish({
      ...state,
      lifecycle: "failed",
      input_locked: false,
      active_attempt_id: null,
      operation_error: message,
      start_failure: null,
    });
  }

  function adoptReceipt(incoming: ConversationTransportReceipt): ConversationTransportState {
    const receipt = mergeConversationTransportReceipts(state.receipt, incoming);
    const lifecycle = lifecycleForTransportStatus(receipt.transport.status);
    const session = {
      conversation_id: receipt.conversation_id ?? state.session.conversation_id,
      thread_id: receipt.thread_id ?? state.session.thread_id,
    };
    const active = lifecycle === "starting" || lifecycle === "running";
    return publish({
      ...state,
      lifecycle,
      input_locked: active,
      active_attempt_id: active ? receipt.transport.attempt_id : null,
      session,
      receipt,
      transcript_events: mergeTranscriptEvents(
        state.transcript_events,
        conversationEventsForReceipt(receipt),
      ),
      operation_error:
        lifecycle === "failed"
          ? receipt.transport.human_message
            ?? (receipt.transport.binding_stage
              ? supervisorStartFailureMessage(receipt.transport.binding_stage)
              : "对话运输未完成。")
          : null,
      start_failure:
        lifecycle === "failed" && receipt.transport.binding_stage
          ? { turn_id: receipt.turn_id, stage: receipt.transport.binding_stage }
          : null,
    });
  }

  async function start(turn: ConversationTransportTurn): Promise<ConversationTransportState> {
    if (state.input_locked) {
      return publish({ ...state, operation_error: "当前对话仍在进行中。" });
    }
    const userText = turn.user_text.trim();
    if (!userText) return finishWithoutRequest("不能发送空白消息。");
    if (turn.mode === "existing" && (!turn.conversation_id.trim() || !turn.thread_id.trim())) {
      return finishWithoutRequest("续聊需要已绑定的会话与 thread。");
    }

    const turnId = turn.turn_id?.trim() || create_turn_id();
    const optimisticUserEvent = buildConversationTransportUserMessage({
      mode: turn.mode,
      turn_id: turnId,
      user_text: userText,
      created_at: now().toISOString(),
    });
    publish({
      ...state,
      lifecycle: "starting",
      input_locked: true,
      active_attempt_id: null,
      receipt: null,
      transcript_events: mergeTranscriptEvents(state.transcript_events, [optimisticUserEvent]),
      operation_error: null,
      start_failure: null,
    });

    try {
      if (turn.mode === "new") {
        const receipt = await client.startNew(
          Object.freeze({
            context,
            mode: "new" as const,
            conversation_id: null,
            thread_id: null,
            turn_id: turnId,
            user_text: userText,
          }),
        );
        return adoptReceipt(receipt);
      }
      const receipt = await client.startExisting(
        Object.freeze({
          context,
          mode: "existing" as const,
          conversation_id: turn.conversation_id.trim(),
          thread_id: turn.thread_id.trim(),
          turn_id: turnId,
          user_text: userText,
        }),
      );
      return adoptReceipt(receipt);
    } catch (error) {
      const startFailure = safeSupervisorStartFailure(error, turnId);
      return publish({
        ...state,
        lifecycle: "failed",
        input_locked: false,
        active_attempt_id: null,
        operation_error: startFailure ? supervisorStartFailureMessage(startFailure.stage) : "对话运输未能启动。",
        start_failure: startFailure,
      });
    }
  }

  async function poll(): Promise<ConversationTransportState> {
    const attemptId = state.active_attempt_id;
    if (!attemptId) return snapshotOf(state);
    try {
      return adoptReceipt(await client.poll(Object.freeze({ attempt_id: attemptId })));
    } catch {
      // A failed poll does not prove that the running process stopped.  Keep
      // the input locked until a terminal receipt or a confirmed Stop arrives.
      return publish({ ...state, operation_error: "状态刷新失败，输入仍保持锁定。" });
    }
  }

  async function stop(): Promise<ConversationTransportState> {
    const attemptId = state.active_attempt_id;
    if (!attemptId) return snapshotOf(state);
    try {
      return adoptReceipt(await client.stop(Object.freeze({ attempt_id: attemptId })));
    } catch {
      // Stop errors are likewise not permission to unlock a possibly-live turn.
      return publish({ ...state, operation_error: "停止请求未确认，输入仍保持锁定。" });
    }
  }

  function reset(): ConversationTransportState {
    return publish({
      ...state,
      lifecycle: "idle",
      input_locked: false,
      active_attempt_id: null,
      receipt: null,
      operation_error: null,
      start_failure: null,
    });
  }

  return Object.freeze({
    getState: () => snapshotOf(state),
    subscribe: (listener) => {
      listeners.add(listener);
      return () => listeners.delete(listener);
    },
    start,
    poll,
    stop,
    reset,
  });
}

export function mergeConversationTransportReceipts(
  previous: ConversationTransportReceipt | null,
  incoming: ConversationTransportReceipt,
): ConversationTransportReceipt {
  const next = safeConversationTransportReceipt(incoming);
  if (!previous) return next;
  const prior = safeConversationTransportReceipt(previous);
  return {
    conversation_id: next.conversation_id ?? prior.conversation_id,
    thread_id: next.thread_id ?? prior.thread_id,
    turn_id: next.turn_id,
    transport: next.transport,
    assistant_reply: mergeAssistantReply(prior.assistant_reply, next.assistant_reply),
    tool_action: next.tool_action,
    read_model_projection: next.read_model_projection,
    canonical_mirror: next.canonical_mirror,
  };
}

// Tauri receipts cross a runtime boundary.  Project their public contract
// before retaining them in controller state so extra bridge/debug payloads can
// never propagate into a shared transcript or a page's receipt state.
function safeConversationTransportReceipt(receipt: ConversationTransportReceipt): ConversationTransportReceipt {
  return {
    conversation_id: receipt.conversation_id,
    thread_id: receipt.thread_id,
    turn_id: receipt.turn_id,
    transport: {
      status: receipt.transport.status,
      human_message: receipt.transport.human_message,
      attempt_id: receipt.transport.attempt_id,
      binding_stage: safeSupervisorBindingStage(receipt.transport.binding_stage),
    },
    assistant_reply: {
      status: receipt.assistant_reply.status,
      text: receipt.assistant_reply.text,
      assistant_item_id: receipt.assistant_reply.assistant_item_id,
      human_message: receipt.assistant_reply.human_message,
    },
    tool_action: {
      status: receipt.tool_action.status,
      human_message: receipt.tool_action.human_message,
    },
    read_model_projection: {
      status: receipt.read_model_projection.status,
      human_message: receipt.read_model_projection.human_message,
    },
    canonical_mirror: {
      status: receipt.canonical_mirror.status,
      human_message: receipt.canonical_mirror.human_message,
    },
  };
}

function safeSupervisorBindingStage(value: unknown): SupervisorConversationBindingStage | null {
  switch (value) {
    case "binding_construct":
    case "binding_store_prepare":
    case "binding_persist_db":
    case "binding_project_json":
    case "binding_activate":
    case "transport_start":
    case "binding_terminate":
      return value;
    default:
      return null;
  }
}

function safeSupervisorStartFailure(
  error: unknown,
  turnId: string,
): Readonly<{ turn_id: string; stage: SupervisorConversationBindingStage }> | null {
  const code =
    error instanceof Error
      ? error.message
      : typeof error === "object" && error !== null && "message" in error && typeof error.message === "string"
        ? error.message
        : null;
  const stage = code === "conversation_transport_supervisor_binding_construct_failed"
    ? "binding_construct"
    : code === "conversation_transport_supervisor_binding_store_prepare_failed"
      ? "binding_store_prepare"
    : code === "conversation_transport_supervisor_binding_persist_db_failed"
      ? "binding_persist_db"
    : code === "conversation_transport_supervisor_binding_project_json_failed"
      ? "binding_project_json"
      : code === "conversation_transport_supervisor_binding_activate_failed"
        ? "binding_activate"
      : code === "conversation_transport_start_failed"
        ? "transport_start"
        : code === "conversation_transport_supervisor_binding_terminate_unconfirmed"
          ? "binding_terminate"
          : null;
  return stage ? { turn_id: turnId, stage } : null;
}

function supervisorStartFailureMessage(stage: SupervisorConversationBindingStage): string {
  switch (stage) {
    case "binding_construct":
      return "主管对话绑定准备未完成；运输没有启动。";
    case "binding_store_prepare":
      return "主管对话绑定存储未准备完成；运输没有启动。";
    case "binding_persist_db":
      return "主管对话绑定没有写入主存储；运输没有启动。";
    case "binding_project_json":
      return "主管对话绑定兼容投影未完成；运输没有启动。";
    case "binding_activate":
      return "主管对话绑定未能激活；工具继续关闭。";
    case "transport_start":
      return "主管对话运输没有启动。";
    case "binding_terminate":
      return "绑定终结未确认；工具继续关闭。";
  }
}

// A later tool/projection/canonical failure must never erase an already proven
// natural-language reply.  Other receipt layers intentionally take the newest
// server value independently.
function mergeAssistantReply(
  previous: ConversationTransportReceipt["assistant_reply"],
  incoming: ConversationTransportReceipt["assistant_reply"],
): ConversationTransportReceipt["assistant_reply"] {
  const priorText = previous.text?.trim() ?? "";
  const nextText = incoming.text?.trim() ?? "";
  if (previous.status === "succeeded" && priorText && incoming.status !== "succeeded") return previous;
  if (previous.status === "succeeded" && priorText && incoming.status === "succeeded" && !nextText) {
    return { ...incoming, text: previous.text, assistant_item_id: incoming.assistant_item_id ?? previous.assistant_item_id };
  }
  return incoming;
}

export function failedConversationReceiptLayers(
  receipt: ConversationTransportReceipt | null,
): ReadonlyArray<Readonly<{ layer: Exclude<keyof ConversationTransportReceipt, "conversation_id" | "thread_id" | "turn_id">; human_message: string }>> {
  if (!receipt) return [];
  const layers = [
    ["transport", receipt.transport],
    ["assistant_reply", receipt.assistant_reply],
    ["tool_action", receipt.tool_action],
    ["read_model_projection", receipt.read_model_projection],
    ["canonical_mirror", receipt.canonical_mirror],
  ] as const;
  return layers
    .filter(([, layer]) => layer.status === "failed")
    .map(([layer, value]) => ({
      layer,
      human_message: value.human_message ?? defaultLayerFailureMessage(layer),
    }));
}

// The shared transcript deliberately contains only normalized user/assistant
// text.  The receipt contract has no legacy relay payload, so raw tool args,
// stdout/stderr, argv, paths, and environment data cannot enter controller
// state through this conversion.
export function conversationEventsForReceipt(receipt: ConversationTransportReceipt): CodexTranscriptEvent[] {
  const threadId = receipt.thread_id ?? receipt.conversation_id;
  const replyText = receipt.assistant_reply.text?.trim() ?? "";
  if (!threadId || receipt.assistant_reply.status !== "succeeded" || !replyText) return [];
  return [{
    event_id: `conversation-transport-assistant:${threadId}:${receipt.turn_id}:${receipt.transport.attempt_id ?? "final"}`,
    timestamp: new Date().toISOString(),
    event_type: "assistant_message",
    actor: "assistant",
    role: "assistant",
    text: replyText,
    metadata: {
      conversation_transport: true,
      turn_id: receipt.turn_id,
      attempt_id: receipt.transport.attempt_id,
    },
    warnings: [],
  }];
}

function buildConversationTransportUserMessage({
  mode,
  turn_id,
  user_text,
  created_at,
}: {
  mode: ConversationTransportMode;
  turn_id: string;
  user_text: string;
  created_at: string;
}): CodexTranscriptEvent {
  return {
    event_id: `conversation-transport-user:${turn_id}`,
    timestamp: created_at,
    event_type: "user_message",
    actor: "user",
    role: "user",
    text: user_text,
    metadata: {
      conversation_transport: true,
      transport_mode: mode,
      turn_id,
    },
    warnings: [],
  };
}

function lifecycleForTransportStatus(status: ConversationReceiptLayerStatus): ConversationTransportLifecycle {
  if (status === "pending") return "running";
  if (status === "succeeded") return "completed";
  if (status === "failed") return "failed";
  if (status === "stopped") return "stopped";
  return "idle";
}

function mergeTranscriptEvents(
  existing: readonly CodexTranscriptEvent[],
  incoming: readonly CodexTranscriptEvent[],
): CodexTranscriptEvent[] {
  const byId = new Map(existing.map((event) => [event.event_id, event]));
  for (const event of incoming) byId.set(event.event_id, event);
  return [...byId.values()];
}

function snapshotOf(state: ConversationTransportState): ConversationTransportState {
  return {
    ...state,
    session: { ...state.session },
    transcript_events: [...state.transcript_events],
  };
}

function defaultLayerFailureMessage(layer: string): string {
  if (layer === "transport") return "对话运输未完成。";
  if (layer === "assistant_reply") return "主管没有形成自然回复。";
  if (layer === "tool_action") return "结构化动作未完成。";
  if (layer === "read_model_projection") return "对话已完成，但读模型还未刷新。";
  return "对话已完成，但事实镜像还未刷新。";
}

function requireIdentifier(value: string, label: string): string {
  const normalized = value.trim();
  if (!normalized) throw new Error(`conversation_transport_${label}_required`);
  return normalized;
}

let fallbackTurnCounter = 0;

function defaultTurnId(): string {
  const uuid = globalThis.crypto?.randomUUID?.();
  if (uuid) return `turn:${uuid}`;
  fallbackTurnCounter += 1;
  return `turn:${Date.now()}:${fallbackTurnCounter}`;
}
