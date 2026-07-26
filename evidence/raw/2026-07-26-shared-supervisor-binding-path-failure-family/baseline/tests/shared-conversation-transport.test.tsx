import {
  createAgentConversationTransportContext,
  createConversationTransportController,
  createSupervisorConversationTransportContext,
  conversationEventsForReceipt,
  failedConversationReceiptLayers,
  mergeConversationTransportReceipts,
  type ConversationTransportClient,
  type ConversationTransportReceipt,
} from "../src/lib/conversationTransport";
import { assert, assertDeepEqual } from "./helpers/offlineInteractionTestUtils";

function layer(status: ConversationTransportReceipt["tool_action"]["status"], human_message: string | null = null) {
  return { status, human_message } as const;
}

function receipt({
  attempt_id,
  conversation_id = "conversation:fixture",
  thread_id = "thread:fixture",
  turn_id,
  transport_status = "pending",
  assistant_status = "pending",
  assistant_text = null,
  tool_status = "not_requested",
  projection_status = "not_requested",
  canonical_status = "not_requested",
  binding_stage = null,
}: {
  attempt_id: string | null;
  conversation_id?: string | null;
  thread_id?: string | null;
  turn_id: string;
  transport_status?: ConversationTransportReceipt["transport"]["status"];
  assistant_status?: ConversationTransportReceipt["assistant_reply"]["status"];
  assistant_text?: string | null;
  tool_status?: ConversationTransportReceipt["tool_action"]["status"];
  projection_status?: ConversationTransportReceipt["read_model_projection"]["status"];
  canonical_status?: ConversationTransportReceipt["canonical_mirror"]["status"];
  binding_stage?: ConversationTransportReceipt["transport"]["binding_stage"];
}): ConversationTransportReceipt {
  return {
    conversation_id,
    thread_id,
    turn_id,
    transport: { ...layer(transport_status), attempt_id, binding_stage },
    assistant_reply: {
      ...layer(assistant_status),
      text: assistant_text,
      assistant_item_id: assistant_text ? "assistant-item:fixture" : null,
    },
    tool_action: layer(tool_status, tool_status === "failed" ? "方案动作没有完成。" : null),
    read_model_projection: layer(projection_status, projection_status === "failed" ? "方案读模型还没有刷新。" : null),
    canonical_mirror: layer(canonical_status, canonical_status === "failed" ? "事实镜像还没有刷新。" : null),
  };
}

// 1a) A pre-transport binding failure is a safe turn-scoped receipt.  It does
// not manufacture a session, reply, tool, card, chain, or worker result.
{
  const failed = receipt({
    attempt_id: null,
    conversation_id: null,
    thread_id: null,
    turn_id: "turn:binding-json",
    transport_status: "failed",
    assistant_status: "not_requested",
    binding_stage: "binding_project_json",
  });
  const controller = createConversationTransportController({
    context: createSupervisorConversationTransportContext({
      project_root: "/fixture/supervisor-project",
      project_id: "project:fixture",
      workflow_id: "workflow:fixture",
    }),
    client: {
      async startNew() { return failed; },
      async startExisting() { throw new Error("not used"); },
      async poll() { throw new Error("not used"); },
      async stop() { throw new Error("not used"); },
    },
    create_turn_id: () => "turn:binding-json",
  });
  const result = await controller.start({ mode: "new", user_text: "安全的测试消息" });
  assertDeepEqual(
    result.start_failure,
    { turn_id: "turn:binding-json", stage: "binding_project_json" },
    "binding JSON projection 失败必须保留安全阶段和 turn，不得伪造 Active session",
  );
  assert(result.session.conversation_id === null && result.session.thread_id === null, "失败 receipt 不得伪造会话");
  assert(result.receipt?.tool_action.status === "not_requested", "失败 binding 不得伪造工具结果");
  const serialized = JSON.stringify(result.receipt);
  for (const forbidden of ["/Users/", "argv", "stderr", "environment", "安全的测试消息"]) {
    assert(!serialized.includes(forbidden), `安全 binding receipt 不得包含 ${forbidden}`);
  }
}

// 1b) If the runtime still rejects before it can return a receipt, retain only
// a recognized stable family.  An unknown error remains generic and is never
// copied into UI/controller state.
{
  const controller = createConversationTransportController({
    context: createSupervisorConversationTransportContext({
      project_root: "/fixture/supervisor-project",
      project_id: "project:fixture",
      workflow_id: "workflow:fixture",
    }),
    client: {
      async startNew() { throw new Error("conversation_transport_supervisor_binding_persist_db_failed"); },
      async startExisting() { throw new Error("not used"); },
      async poll() { throw new Error("not used"); },
      async stop() { throw new Error("not used"); },
    },
    create_turn_id: () => "turn:binding-db",
  });
  const result = await controller.start({ mode: "new", user_text: "安全的测试消息" });
  assertDeepEqual(
    result.start_failure,
    { turn_id: "turn:binding-db", stage: "binding_persist_db" },
    "start catch 必须保留稳定 DB-primary family",
  );
  assert(!JSON.stringify(result).includes("conversation_transport_supervisor_binding_persist_db_failed"), "前端状态不得保留后端 raw family");
}

// 1c) Newly separated store/activation/termination stages must survive the
// runtime receipt boundary.  A termination-unconfirmed receipt is still a
// failure receipt: it does not manufacture a session or tool outcome.
{
  for (const binding_stage of ["binding_store_prepare", "binding_activate", "binding_terminate"] as const) {
    const failed = receipt({
      attempt_id: null,
      conversation_id: null,
      thread_id: null,
      turn_id: `turn:${binding_stage}`,
      transport_status: "failed",
      assistant_status: "not_requested",
      binding_stage,
    });
    const controller = createConversationTransportController({
      context: createSupervisorConversationTransportContext({
        project_root: "/fixture/supervisor-project",
        project_id: "project:fixture",
        workflow_id: "workflow:fixture",
      }),
      client: {
        async startNew() { return failed; },
        async startExisting() { throw new Error("not used"); },
        async poll() { throw new Error("not used"); },
        async stop() { throw new Error("not used"); },
      },
      create_turn_id: () => `turn:${binding_stage}`,
    });
    const result = await controller.start({ mode: "new", user_text: "安全的测试消息" });
    assertDeepEqual(
      result.start_failure,
      { turn_id: `turn:${binding_stage}`, stage: binding_stage },
      `${binding_stage} 必须保留为固定安全阶段`,
    );
    assert(result.session.conversation_id === null && result.session.thread_id === null, `${binding_stage} 不得伪造会话`);
    assert(result.receipt?.tool_action.status === "not_requested", `${binding_stage} 不得伪造工具结果`);
    if (binding_stage === "binding_terminate") {
      assert(
        result.operation_error === "绑定终结未确认；工具继续关闭。",
        "终结失败必须使用中立闭锁文案",
      );
      assert(!result.operation_error.includes("运输"), "终结失败不得臆测 transport 是否启动");
    }
    if (binding_stage === "binding_activate") {
      assert(result.operation_error?.includes("工具继续关闭"), "激活失败必须继续关闭工具");
      assert(!result.operation_error?.includes("运输没有启动"), "激活失败不得臆测运输状态");
    }
  }
}

// 1) New → poll exercises all five receipt layers.  A natural reply remains
// visible even when each later layer fails independently.
{
  const requests: unknown[] = [];
  const client: ConversationTransportClient = {
    async startNew(request) {
      requests.push(request);
      return receipt({
        attempt_id: "attempt:new:1",
        turn_id: request.turn_id,
        assistant_status: "succeeded",
        assistant_text: "我先给出自然回复。",
      });
    },
    async startExisting() {
      throw new Error("existing is exercised below");
    },
    async poll(request) {
      assert(request.attempt_id === "attempt:new:1", "poll 只能收到 transport 已登记的 attempt id");
      return receipt({
        attempt_id: "attempt:new:1",
        turn_id: "turn:new:1",
        transport_status: "succeeded",
        assistant_status: "not_requested",
        tool_status: "failed",
        projection_status: "failed",
        canonical_status: "failed",
      });
    },
    async stop() {
      throw new Error("stop is exercised below");
    },
  };
  const controller = createConversationTransportController({
    context: createAgentConversationTransportContext({ project_root: "/fixture/agent-project" }),
    client,
    create_turn_id: () => "turn:new:1",
    now: () => new Date("2026-07-23T00:00:00.000Z"),
  });

  const afterStart = await controller.start({ mode: "new", user_text: "先聊一下。" });
  assert(afterStart.lifecycle === "running" && afterStart.input_locked, "new turn running 时必须锁住输入");
  assert(afterStart.session.thread_id === "thread:fixture", "new receipt 的 thread 必须进入共享 session state");

  const startRequest = requests[0] as Record<string, unknown>;
  assert(startRequest?.mode === "new", "new turn 必须调用独立 startNew client operation");
  const requestJson = JSON.stringify(startRequest);
  for (const forbidden of ["sandbox", "allowed_write_roots", "capabilities", "approval", "add_dir", "role"]) {
    assert(!requestJson.includes(forbidden), `共享 start request 不得携带 ${forbidden}`);
  }

  const afterPoll = await controller.poll();
  assert(afterPoll.lifecycle === "completed" && !afterPoll.input_locked, "terminal transport receipt 必须解锁输入");
  assert(
    afterPoll.receipt?.assistant_reply.status === "succeeded" && afterPoll.receipt.assistant_reply.text === "我先给出自然回复。",
    "tool/projection/canonical 失败不得擦掉已成立的自然回复",
  );
  assert(
    afterPoll.transcript_events.some((event) => event.text === "我先给出自然回复。"),
    "共享 transcript 必须保留已成立自然回复",
  );
  assertDeepEqual(
    failedConversationReceiptLayers(afterPoll.receipt).map((item) => item.layer),
    ["tool_action", "read_model_projection", "canonical_mirror"],
    "三个后置失败层必须独立结算，不能改写 transport/reply",
  );
}

// 2) The same controller can continue the newly-created session through its
// explicit existing-session operation; no second page-local state machine is
// necessary.
{
  const calls: string[] = [];
  const client: ConversationTransportClient = {
    async startNew(request) {
      calls.push("new");
      return receipt({
        attempt_id: null,
        conversation_id: "conversation:new",
        thread_id: "thread:new",
        turn_id: request.turn_id,
        transport_status: "succeeded",
        assistant_status: "succeeded",
        assistant_text: "第一句回复。",
      });
    },
    async startExisting(request) {
      calls.push(`${request.mode}:${request.conversation_id}:${request.thread_id}`);
      return receipt({
        attempt_id: null,
        conversation_id: request.conversation_id,
        thread_id: request.thread_id,
        turn_id: request.turn_id,
        transport_status: "succeeded",
        assistant_status: "succeeded",
        assistant_text: "第二句回复。",
      });
    },
    async poll() {
      throw new Error("completed turns must not poll");
    },
    async stop() {
      throw new Error("completed turns must not stop");
    },
  };
  const controller = createConversationTransportController({
    context: createAgentConversationTransportContext({ project_root: "/fixture/agent-project" }),
    client,
    create_turn_id: () => "turn:next",
  });
  const first = await controller.start({ mode: "new", user_text: "第一句", turn_id: "turn:first" });
  const second = await controller.start({
    mode: "existing",
    user_text: "第二句",
    conversation_id: first.session.conversation_id ?? "",
    thread_id: first.session.thread_id ?? "",
    turn_id: "turn:second",
  });
  assertDeepEqual(calls, ["new", "existing:conversation:new:thread:new"], "new 与 existing 必须走同一 controller 的两条显式 client operation");
  assert(second.transcript_events.some((event) => event.text === "第二句回复。"), "second turn reply 必须接入同一共享 transcript");
}

// 3) A confirmed Stop is terminal and unlocks input.  A failed Stop would keep
// it locked, but this path proves the positive contract without Tauri/live work.
{
  const stopAttempts: string[] = [];
  const client: ConversationTransportClient = {
    async startNew() {
      throw new Error("not used");
    },
    async startExisting(request) {
      return receipt({ attempt_id: "attempt:stop:1", turn_id: request.turn_id });
    },
    async poll() {
      throw new Error("not used");
    },
    async stop(request) {
      stopAttempts.push(request.attempt_id);
      return receipt({
        attempt_id: request.attempt_id,
        turn_id: "turn:stop",
        transport_status: "stopped",
        assistant_status: "not_requested",
      });
    },
  };
  const controller = createConversationTransportController({
    context: createSupervisorConversationTransportContext({
      project_root: "/fixture/supervisor-project",
      project_id: "project:fixture",
      workflow_id: "workflow:fixture",
    }),
    client,
  });
  assert(
    !("role" in controller.getState().context),
    "主管 role 必须由服务端可信 binding 固定，前端 context 不得提交 role",
  );
  await controller.start({
    mode: "existing",
    user_text: "请停止。",
    conversation_id: "conversation:stop",
    thread_id: "thread:stop",
    turn_id: "turn:stop",
  });
  assert(controller.getState().input_locked, "running supervisor turn 必须先锁输入");
  const afterStop = await controller.stop();
  assertDeepEqual(stopAttempts, ["attempt:stop:1"], "stop 只发送 server-owned active attempt id");
  assert(afterStop.lifecycle === "stopped" && !afterStop.input_locked, "confirmed Stop 必须解锁输入");
}

// 4) Shared receipt/state accepts only its normalized five layers.  Even if a
// caller hands the converter an unexpected legacy-shaped field at runtime, it
// produces assistant text exclusively from safe receipt fields.
{
  const safeReceipt = receipt({
    attempt_id: "attempt:safe-receipt",
    conversation_id: "conversation:safe-receipt",
    thread_id: "thread:safe-receipt",
    turn_id: "turn:safe-receipt",
    transport_status: "succeeded",
    assistant_status: "succeeded",
    assistant_text: "只保留这句主管自然回复。",
  });
  const receiptWithUnexpectedPayload = Object.assign({}, safeReceipt, {
    untrusted_payload: { ignored_detail: "untrusted-payload-must-not-render" },
    transport: Object.assign({}, safeReceipt.transport, {
      ignored_detail: "untrusted-payload-must-not-render",
      binding_stage: "untrusted-binding-stage",
    }),
  });
  const runtimeReceipt = receiptWithUnexpectedPayload as unknown as ConversationTransportReceipt;
  const retainedReceiptJson = JSON.stringify(mergeConversationTransportReceipts(null, runtimeReceipt));
  const eventJson = JSON.stringify(conversationEventsForReceipt(runtimeReceipt));
  assert(eventJson.includes("只保留这句主管自然回复。"), "安全 receipt 必须保留 assistant final reply");
  assert(!eventJson.includes("untrusted-payload-must-not-render"), "共享 transcript 不得读取未知 receipt payload");
  assert(!retainedReceiptJson.includes("untrusted-payload-must-not-render"), "共享 receipt/state 不得保留未知 bridge payload");
  assert(!retainedReceiptJson.includes("untrusted-binding-stage"), "共享 receipt/state 只允许固定 binding 阶段枚举");
}

console.log("shared-conversation-transport: profile-safe requests, new/existing session, independent receipts, Stop, and safe receipt filtering passed");
