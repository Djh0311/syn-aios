import { renderToStaticMarkup } from "react-dom/server.browser";
import { SecretaryBrief } from "../src/components/SecretaryBrief";
import {
  HomeView,
  secretaryCoordinationActionsFor,
  type SecretaryCoordinationIntent,
  type SecretaryPersonalObjectIntent,
} from "../src/views/HomeView";
import type {
  SecretaryHomeAttentionItem,
  SecretaryHomeReadModel,
  SecretaryTypedDeepLinkDescriptor,
} from "../src/lib/types/m4Secretary";
import { assert, assertDeepEqual, findButtonByText } from "./helpers/offlineInteractionTestUtils";

const hash = (character: string) => character.repeat(64);

function attention(overrides: Partial<SecretaryHomeAttentionItem> = {}): SecretaryHomeAttentionItem {
  const itemRef = overrides.item_ref ?? "open-loop:fixture-loop";
  const deepLink: SecretaryTypedDeepLinkDescriptor = {
    kind: "M4_SOURCE_ROUTE",
    source_owner_ref: "owner:workflow",
    source_object_ref: "workflow-object:fixture",
    source_object_type: "WORKFLOW_ATTENTION",
    source_route_ref: "route:opaque-fixture",
    executable_payload: null,
  };
  return {
    item_ref: itemRef,
    item_kind_code: itemRef.startsWith("inbox:") ? "INBOX_ITEM" : "OPEN_LOOP",
    source_authority: "M4_COORDINATION",
    source_owner: { availability: "AVAILABLE", source_owner_ref: "owner:workflow" },
    source_object_ref: "workflow-object:fixture",
    source_object_type: "WORKFLOW_ATTENTION",
    deep_link: overrides.deep_link ?? deepLink,
    why_code: "WAITING_FOR_OWNER",
    priority_rank: 1,
    priority_reason_code: "EXTERNAL_COMMITMENT",
    status_code: "OPEN",
    source_status_code: "ACTIVE",
    last_change_at_utc: "2026-08-10T09:00:00Z",
    due_at_utc: "2026-08-10T16:00:00Z",
    change_hash: hash("a"),
    coordination_revision: "7",
    ...overrides,
  };
}

function home(overrides: Partial<SecretaryHomeReadModel> = {}): SecretaryHomeReadModel {
  const attentionItem = attention();
  return {
    schema_version: "syn.m4.secretary.home.v1",
    state: "ready",
    source_authority: "M4_APPLICATION_SERVICE",
    context: {
      context_ref: "secretary-context:fixture",
      role_session_ref: "role-session:fixture",
      scope_ref: "scope:personal:primary",
      scope_source_watermark: hash("w"),
      snapshot_hash: hash("s"),
      reconstruction_code: "DETERMINISTIC_REBUILD",
    },
    deterministic_brief: {
      brief_ref: "secretary-brief:fixture",
      brief_hash: hash("b"),
      context_ref: "secretary-context:fixture",
      scope_source_watermark: hash("w"),
    },
    scope_source_watermark: hash("w"),
    role_session_recovery: {
      status: "RESTORED",
      role_session_ref: "role-session:fixture",
      context_ref: "secretary-context:fixture",
      recovery_code: null,
    },
    attention_items: [attentionItem],
    personal_actions: [
      {
        personal_action_ref: "personal-action:fixture-only",
        explicit_user_command_ref: "user-command:fixture-only",
        status_code: "OPEN",
        due_at_utc: "2026-08-11T09:00:00Z",
        revision_hash: hash("p"),
        coordination_revision: "4",
        source_authority: "M4_COORDINATION",
      },
    ],
    local_objects: {
      personal_actions: [{
        personal_action_id: "personal-action:fixture-only",
        explicit_user_command_ref: "user-command:fixture-only",
        title: "准备季度复盘",
        status: "OPEN",
        due_at_utc: "2026-08-11T09:00:00Z",
        revision: "4",
      }],
      notifications: [{
        notification_id: "notification:fixture-only",
        source_ref: {
          link_kind: "INTERNAL_ROUTE",
          source_owner_ref: "owner:workflow",
          object_type: "workflow_attention",
          canonical_source_object_id: "workflow-object:fixture",
          expected_source_revision: "7",
          opaque_route_ref: `route:sha256:${hash("r")}`,
        },
        subject_ref: `source-event:sha256:${hash("e")}`,
        notification_purpose_code: "SOURCE_ATTENTION_PUBLISHED",
        delivery_channel: "IN_APP",
        status: "DELIVERED",
        created_at_utc: "2026-08-10T09:00:00Z",
        delivered_at_utc: "2026-08-10T09:00:00Z",
        read_at_utc: null,
        dismissed_at_utc: null,
        revision: "2",
      }],
      reminders: [{
        reminder_id: "reminder:fixture-only",
        owner_ref: "personal-action:fixture-only",
        explicit_schedule_command_id: `schedule-command:sha256:${hash("c")}`,
        scheduled_for_utc: "2026-08-11T10:00:00Z",
        iana_timezone: "Asia/Shanghai",
        status: "SCHEDULED",
        last_fired_at_utc: null,
        snoozed_until_utc: null,
        revision: "3",
      }],
      decisions: [{
        decision_projection_id: "decision-projection:fixture-only",
        source_identity_key: `source-identity:sha256:${hash("i")}`,
        source_event_key: `source-event:sha256:${hash("j")}`,
        source_ref: "proposal:fixture-only",
        owner_status: "EXPIRED",
        local_visibility_status: "UNREAD",
        decision_by_utc: null,
        source_revision: "8",
        revision: "1",
      }],
      reminder_owner_refs: ["personal-action:fixture-only", `source-identity:sha256:${hash("i")}`],
    },
    module_entries: [
      {
        entry_ref: "secretary-owner-route:fixture",
        entry_kind: "SOURCE_OWNER_ROUTE",
        source_owner: { availability: "AVAILABLE", source_owner_ref: "owner:workflow" },
        deep_link: attentionItem.deep_link,
        action_payload: null,
      },
    ],
    model_enhancement: {
      status: "NOT_REQUESTED",
      invocation_ref: null,
      enhancement_ref: null,
      enhancement_hash: null,
      invocation_receipt: null,
      recovery_code: null,
    },
    handoff: {
      status: "NOT_LOADED",
      handoff_ref: null,
      request_receipt_ref: null,
      returned_receipt: null,
      recovery_code: null,
    },
    degradation_code: null,
    ...overrides,
  };
}

// 1) The M4 attention spine is the first content region and carries every
// required source-backed field before the separate PersonalAction section.
{
  const model = home();
  const markup = renderToStaticMarkup(<HomeView secretaryHome={model} onOpenDeepLink={() => undefined} />);
  const attentionAt = markup.indexOf("secretary-attention-region");
  const personalAt = markup.indexOf("secretary-personal-actions");
  assert(attentionAt >= 0 && personalAt > attentionAt, "source-backed attention is the visual/content focal region");
  for (const expected of [
    "EXTERNAL_COMMITMENT",
    "WAITING_FOR_OWNER",
    "当前状态",
    "OPEN",
    "owner:workflow",
    "WORKFLOW_ATTENTION",
    "最后变化 2026-08-10 09:00:00 UTC",
    "到期 2026-08-10 16:00:00 UTC",
    "回到来源",
  ]) {
    assert(markup.includes(expected), `attention spine retains ${expected}`);
  }
  const personalMarkup = markup.slice(personalAt, markup.indexOf("secretary-availability", personalAt));
  assert(personalMarkup.includes("personal-action:fixture-only"), "PersonalAction is independently rendered");
  assert(personalMarkup.includes("准备季度复盘"), "PersonalAction title is rendered from the local-only DTO");
  assert(!personalMarkup.includes("open-loop:fixture-loop"), "OpenLoop is not cloned into PersonalAction");
  assert(!markup.includes("完成业务") && !markup.includes("创建 Todo"), "home exposes no owner completion or Todo creation affordance");
}

// 2) UI action matrix mirrors the finite M4 coordinator transitions and
// remains silent for source-terminal/unknown states.
{
  const inboxNew = attention({ item_ref: "inbox:fixture-new", item_kind_code: "INBOX_ITEM", status_code: "NEW" });
  const inboxRead = attention({ item_ref: "inbox:fixture-read", item_kind_code: "INBOX_ITEM", status_code: "READ" });
  const open = attention({ status_code: "OPEN" });
  const acknowledged = attention({ status_code: "ACKNOWLEDGED" });
  const snoozed = attention({ status_code: "SNOOZED" });
  const closed = attention({ status_code: "CLOSED" });
  assertDeepEqual(
    secretaryCoordinationActionsFor(inboxNew).map((entry) => entry.action),
    ["INBOX_MARK_READ", "INBOX_DISMISS"],
    "new Inbox offers only read/dismiss",
  );
  assertDeepEqual(
    secretaryCoordinationActionsFor(inboxRead).map((entry) => entry.action),
    ["INBOX_DISMISS"],
    "read Inbox offers only dismiss",
  );
  assertDeepEqual(
    secretaryCoordinationActionsFor(open).map((entry) => entry.action),
    ["OPEN_LOOP_ACKNOWLEDGE", "OPEN_LOOP_SNOOZE", "OPEN_LOOP_CLOSE", "OPEN_LOOP_DISMISS", "OPEN_LOOP_CARRY_OVER"],
    "open loop exposes its complete finite action set",
  );
  assertDeepEqual(
    secretaryCoordinationActionsFor(acknowledged).map((entry) => entry.action),
    ["OPEN_LOOP_SNOOZE", "OPEN_LOOP_CLOSE", "OPEN_LOOP_DISMISS", "OPEN_LOOP_CARRY_OVER"],
    "acknowledged loop excludes duplicate acknowledgement",
  );
  assertDeepEqual(
    secretaryCoordinationActionsFor(snoozed).map((entry) => entry.action),
    ["OPEN_LOOP_CLOSE", "OPEN_LOOP_DISMISS"],
    "snoozed loop does not offer renderer-side reopen",
  );
  assertDeepEqual(
    secretaryCoordinationActionsFor(closed).map((entry) => entry.action),
    ["OPEN_LOOP_REOPEN"],
    "closed loop only offers reopen",
  );
  assertDeepEqual(
    secretaryCoordinationActionsFor(attention({ status_code: "UNKNOWN" })),
    [],
    "unknown lifecycle state fails closed",
  );
}

// 3) Native buttons emit only typed M4 action/deep-link intents. Pending and
// failed state stay visible and a failed action remains retryable.
{
  const model = home();
  let receivedIntent: SecretaryCoordinationIntent | null = null;
  let opened: SecretaryTypedDeepLinkDescriptor | null = null;
  const tree = (
    <HomeView
      secretaryHome={model}
      onOperateCoordination={(intent) => { receivedIntent = intent; }}
      onOpenDeepLink={(descriptor) => { opened = descriptor; }}
    />
  );
  const markRead = findButtonByText(tree, "确认看见");
  assert(markRead?.props?.["data-secretary-action"] === "OPEN_LOOP_ACKNOWLEDGE", "action button identifies finite action code");
  (markRead?.props?.onClick as (() => void) | undefined)?.();
  const dispatched = receivedIntent as SecretaryCoordinationIntent | null;
  assert(dispatched?.action === "OPEN_LOOP_ACKNOWLEDGE" && dispatched.item.item_ref === "open-loop:fixture-loop", "button emits typed item/action only");
  const source = findButtonByText(tree, "回到来源");
  assert(source?.props?.["aria-label"] === "在来源模块中查看此关注项", "source route uses a native labelled button");
  (source?.props?.onClick as (() => void) | undefined)?.();
  const openedDescriptor = opened as SecretaryTypedDeepLinkDescriptor | null;
  assert(openedDescriptor?.kind === "M4_SOURCE_ROUTE" && openedDescriptor.executable_payload === null, "deep link preserves sealed descriptor and no executable payload");

  const pending = renderToStaticMarkup(
    <HomeView
      secretaryHome={model}
      coordinationStates={{ "open-loop:fixture-loop": { phase: "pending", action: "OPEN_LOOP_ACKNOWLEDGE" } }}
      onOperateCoordination={() => undefined}
    />,
  );
  assert(pending.includes("正在记录协调动作"), "pending coordination state is visible");
  const failed = renderToStaticMarkup(
    <HomeView
      secretaryHome={model}
      coordinationStates={{ "open-loop:fixture-loop": { phase: "failed", action: "OPEN_LOOP_ACKNOWLEDGE", error_code: "M4_CONFLICT" } }}
      onOperateCoordination={() => undefined}
    />,
  );
  assert(failed.includes("M4_CONFLICT") && failed.includes("可在此重试"), "failure is visible and recoverable without a fake success");
}

// 4) Home and the right summary expose loading, degraded/error, restored
// identity and model/Handoff availability. Mechanical explain remains
// independent from model enhancement availability.
{
  assert(renderToStaticMarkup(<HomeView secretaryHome={home()} presentationState="loading" />).includes("正在恢复同一情境"), "loading state is explicit");
  assert(renderToStaticMarkup(<HomeView secretaryHome={home()} presentationState="error" />).includes("秘书情境暂时没读出来"), "error state is explicit");
  const unavailable = home({
    model_enhancement: { status: "UNAVAILABLE", invocation_ref: null, enhancement_ref: null, enhancement_hash: null, invocation_receipt: null, recovery_code: "MODEL_NOT_CONFIGURED" },
    handoff: { status: "UNAVAILABLE", handoff_ref: null, request_receipt_ref: null, returned_receipt: null, recovery_code: "HANDOFF_NOT_CONFIGURED" },
  });
  const brief = renderToStaticMarkup(<SecretaryBrief home={unavailable} />);
  for (const expected of ["role-session:fixture", "secretary-context:fixture", "secretary-brief:fixture", "MODEL_NOT_CONFIGURED", "HANDOFF_NOT_CONFIGURED", "请秘书解释"]) {
    assert(brief.includes(expected), `continuity / unavailable status shows ${expected}`);
  }
  assert(
    !brief.includes("secretary-explain-unavailable"),
    "model unavailability does not disable the independent mechanical explain path",
  );
  assert(!brief.includes("cachedExplanation"), "brief does not serialize a module-level explanation cache");
}

// 5) The HomeView can be statically rendered (the existing harness's zero-hook
// requirement), and emits stable class hooks for focus/narrow-screen styling.
{
  const markup = renderToStaticMarkup(<HomeView secretaryHome={home()} onOperateCoordination={() => undefined} />);
  assert(markup.includes("secretary-home-action") && markup.includes("data-secretary-action"), "native action buttons retain keyboard/focus style hooks");
  assert(markup.includes("secretary-attention-spine") && markup.includes("secretary-home-layout"), "source spine retains the narrow-screen layout hooks");
}

// 6) Local object controls expose only ordinary local transitions. The typed
// Decision keeps owner expiry and local visibility separate, and the Reminder
// surface never exposes the server-owned fire transition.
{
  const model = home();
  let personalIntent: SecretaryPersonalObjectIntent | null = null;
  const tree = <HomeView secretaryHome={model} onOperatePersonalObject={(intent) => { personalIntent = intent; }} />;
  const complete = findButtonByText(tree, "完成");
  (complete?.props?.onClick as (() => void) | undefined)?.();
  const dispatched = personalIntent as SecretaryPersonalObjectIntent | null;
  assert(
    dispatched?.action === "PERSONAL_ACTION_COMPLETE"
      && "item_ref" in dispatched
      && dispatched.item_ref === "personal-action:fixture-only",
    "PersonalAction button emits only item/revision intent",
  );
  const markup = renderToStaticMarkup(tree);
  for (const expected of [
    "准备季度复盘",
    "SOURCE_ATTENTION_PUBLISHED",
    "server clock reminder",
    "来源状态",
    "EXPIRED",
    "本地显示",
    "UNREAD",
    "data-secretary-personal-action=\"REMINDER_SNOOZE\"",
    "data-secretary-personal-action=\"DECISION_DISMISS\"",
  ]) {
    assert(markup.includes(expected), `local object UI retains ${expected}`);
  }
  assert(!markup.includes("REMINDER_FIRE"), "Reminder fire remains server-owned and absent from the renderer");
}

console.log("m4c06-secretary-home-ui: hierarchy, source continuity, action matrix, recovery, focus and narrow-screen assertions passed");
