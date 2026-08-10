import {
  createSecretaryCoordinationActionRequest,
  deriveSecretaryHomeReadModel,
  mintSecretaryCoordinationIdempotencyKey,
  parseSecretaryCoordinationActionReceipt,
  parseSecretaryHomeContextEnvelope,
} from "../src/lib/secretaryReadModel";
import type { SecretaryContext } from "../src/lib/secretaryReadModel";
import { assert, assertDeepEqual } from "./helpers/offlineInteractionTestUtils";

const hash = (character: string) => (/^[a-f0-9]$/.test(character) ? character : "a").repeat(64);

function sourceBackedItem(input: Partial<Record<string, unknown>> = {}) {
  return {
    item_ref: "open-loop:fixture-b",
    item_kind_code: "OPEN_LOOP",
    source_owner_ref: "owner:workflow",
    source_object_ref: "workflow-object:fixture-b",
    source_object_type: "workflow_attention",
    source_route_ref: "route:opaque-b",
    source_summary_ref: "summary:opaque-b",
    why_code: "WAITING_FOR_OWNER",
    priority_rank: 1,
    priority_code: "EXTERNAL_COMMITMENT",
    status_code: "OPEN",
    source_status_code: "ACTIVE",
    coordination_revision: "7",
    due_at_utc: null,
    last_change_at_utc: "2026-08-10T09:00:00Z",
    change_hash: hash("b"),
    ...input,
  };
}

function personalAction(input: Partial<Record<string, unknown>> = {}) {
  return {
    personal_action_ref: "personal-action:fixture-a",
    explicit_user_command_ref: "user-command:opaque-a",
    status_code: "OPEN",
    due_at_utc: "2026-08-11T09:00:00Z",
    coordination_revision: "4",
    revision_hash: hash("p"),
    ...input,
  };
}

function readyEnvelope(input: {
  attention_items?: readonly Record<string, unknown>[];
  personal_actions?: readonly Record<string, unknown>[];
  model_enhancement?: Record<string, unknown> | null;
} = {}) {
  return {
    status: "READY",
    application_outcome: {
      context: {
        context_ref: "secretary-context:opaque-a",
        role_session_ref: "role-session:opaque-a",
        scope_ref: "scope:personal:primary",
        scope_source_watermark: hash("w"),
        snapshot_hash: hash("s"),
        reconstruction_code: "DETERMINISTIC_REBUILD",
      },
      deterministic_brief: {
        brief_ref: "secretary-brief:opaque-a",
        brief_hash: hash("f"),
        context_ref: "secretary-context:opaque-a",
        scope_source_watermark: hash("w"),
        attention_items: input.attention_items ?? [
          sourceBackedItem(),
          sourceBackedItem({
            item_ref: "inbox:fixture-a",
            item_kind_code: "INBOX_ITEM",
            source_object_ref: "workflow-object:fixture-a",
            source_route_ref: "route:opaque-a",
            source_summary_ref: "summary:opaque-a",
            due_at_utc: "2026-08-10T08:00:00Z",
            last_change_at_utc: "2026-08-10T08:00:00Z",
            coordination_revision: "6",
            change_hash: hash("a"),
          }),
        ],
        personal_actions: input.personal_actions ?? [personalAction()],
      },
      model_enhancement: input.model_enhancement ?? null,
    },
  };
}

// 1) Authoritative M4 output drives deterministic sorting and retains the
// typed source route rather than producing a local command payload.
{
  const raw = readyEnvelope();
  const rawBefore = JSON.stringify(raw);
  const model = deriveSecretaryHomeReadModel({ home_context: parseSecretaryHomeContextEnvelope(raw) });

  assert(model.state === "ready" && model.source_authority === "M4_APPLICATION_SERVICE", "M4 envelope is the default home authority");
  assertDeepEqual(
    model.attention_items.map((item) => item.item_ref),
    ["inbox:fixture-a", "open-loop:fixture-b"],
    "priority / due / change order is mechanically stable",
  );
  const [first] = model.attention_items;
  assert(first?.deep_link.kind === "M4_SOURCE_ROUTE", "source-backed item exposes a typed M4 deep link");
  if (!first || first.deep_link.kind !== "M4_SOURCE_ROUTE") throw new Error("fixture missing typed M4 deep link");
  assert(
    first.deep_link.source_owner_ref === "owner:workflow"
      && first.deep_link.source_object_type === "workflow_attention"
      && first.deep_link.source_route_ref === "route:opaque-a"
      && first.deep_link.executable_payload === null,
    "deep link preserves owner / object / route only",
  );
  assert(first.coordination_revision === "6", "brief item retains canonical decimal coordination revision");
  assert(JSON.stringify(raw) === rawBefore, "read-model projection does not mutate backend input");
}

// 2) A PersonalAction stays a distinct explicit aggregate; the OpenLoop above
// must not appear in the personal action collection.
{
  const model = deriveSecretaryHomeReadModel({ home_context: parseSecretaryHomeContextEnvelope(readyEnvelope()) });
  assertDeepEqual(
    model.personal_actions.map((action) => action.personal_action_ref),
    ["personal-action:fixture-a"],
    "OpenLoop is not cloned into PersonalAction",
  );
  assert(
    model.personal_actions[0]?.coordination_revision === "4" && model.personal_actions[0]?.explicit_user_command_ref === "user-command:opaque-a",
    "independent PersonalAction retains its canonical revision and explicit user-command ref",
  );
}

// 3) A ready C05 result is a durable server-resolved RoleSession/context
// recovery state, never a renderer-selected legacy conversation.
{
  const model = deriveSecretaryHomeReadModel({ home_context: parseSecretaryHomeContextEnvelope(readyEnvelope()) });
  assertDeepEqual(
    model.role_session_recovery,
    {
      status: "RESTORED",
      role_session_ref: "role-session:opaque-a",
      context_ref: "secretary-context:opaque-a",
      recovery_code: null,
    },
    "RoleSession and context refs come from the authoritative C05 outcome",
  );
  assert(model.scope_source_watermark === hash("w"), "scope watermark remains attached to the rebuilt context");
}

// 4) Empty and unavailable/error conditions remain visible.  A compatibility
// summary is explicitly degraded and never recovers a RoleSession.
{
  const empty = deriveSecretaryHomeReadModel({
    home_context: parseSecretaryHomeContextEnvelope(readyEnvelope({ attention_items: [], personal_actions: [] })),
  });
  assert(empty.state === "empty" && empty.role_session_recovery.status === "RESTORED", "an authoritative empty brief is not a load failure");

  const unavailable = deriveSecretaryHomeReadModel({
    home_context: parseSecretaryHomeContextEnvelope({
      status: "UNAVAILABLE",
      reason: "M3_BINDING_UNAVAILABLE",
    }),
  });
  assert(
    unavailable.state === "degraded"
      && unavailable.role_session_recovery.status === "UNAVAILABLE"
      && unavailable.degradation_code === "M3_BINDING_UNAVAILABLE",
    "unavailable M4/M3 state remains explicit without a hidden identity fallback",
  );
}

// 5) The old SecretaryContext remains a compatibility input only.  Its raw
// source metadata is replaced by opaque display refs and cannot be presented
// as an M4 repository read or a persistent command payload.
{
  const legacy = {
    risk_signals: [
      {
        kind: "diagnostic_warning",
        severity: "high",
        source_refs: [{ source_kind: "project", source_id: "/private/raw/path?token=token-value&prompt=response", label: "raw" }],
      },
    ],
    suggestions: [],
    action_proposals: [
      {
        proposal_id: "legacy-proposal",
        target_ref: { source_kind: "project", source_id: "https://example.invalid/raw/response", label: "raw" },
      },
    ],
  } as unknown as SecretaryContext;
  const before = JSON.stringify(legacy);
  const model = deriveSecretaryHomeReadModel({ compatibility_context: legacy, phase: "error", error_code: "M4_HOME_UNAVAILABLE" });
  const serialized = JSON.stringify(model);
  assert(
    model.state === "degraded"
      && model.source_authority === "CANONICAL_SNAPSHOT_SUMMARY"
      && model.role_session_recovery.status === "UNAVAILABLE"
      && model.attention_items[0]?.source_owner.availability === "NOT_EMITTED_BY_SUMMARY",
    "compatibility mode is visibly bounded and has no synthetic source owner/session",
  );
  assert(!serialized.includes("/private/raw/path") && !serialized.includes("https://") && !serialized.includes("token-value") && !serialized.includes("prompt=response"), "serialized projection excludes raw path, URL, token, prompt and response text");
  assert(!serialized.includes("action_payload\":{") && model.module_entries.every((entry) => entry.action_payload === null), "specialist routes retain no executable action payload");
  assert(JSON.stringify(legacy) === before, "compatibility projection does not mutate its input");
}

// 6) Model/Handoff availability is explicit and does not remove the
// deterministic brief.
{
  const model = deriveSecretaryHomeReadModel({
    home_context: parseSecretaryHomeContextEnvelope(readyEnvelope({
      model_enhancement: {
        status: "UNAVAILABLE",
        invocation_ref: null,
        enhancement_ref: null,
        enhancement_hash: null,
        invocation_receipt: null,
        recovery_code: "MODEL_NOT_CONFIGURED",
      },
    })),
    handoff: {
      status: "UNAVAILABLE",
      handoff_ref: null,
      request_receipt_ref: null,
      returned_receipt: null,
      recovery_code: "HANDOFF_NOT_CONFIGURED",
    },
  });
  assert(
    model.model_enhancement.status === "UNAVAILABLE"
      && model.handoff.status === "UNAVAILABLE"
      && model.attention_items.length === 2,
    "model/Handoff unavailable remains explicit while deterministic brief stays available",
  );
}

// 7) The coordination bridge accepts only the registered C04 action matrix
// and returns a scrubbed, repository-minted receipt.
{
  const request = createSecretaryCoordinationActionRequest({
    action: "OPEN_LOOP_SNOOZE",
    item_ref: "open-loop:fixture-b",
    expected_revision: "18446744073709551615",
    idempotency_key: `idempotency:sha256:${hash("a")}`,
    snoozed_until_utc: "2026-08-10T13:00:00Z",
  });
  assert(request.action === "OPEN_LOOP_SNOOZE" && request.expected_revision === "18446744073709551615", "coordination request preserves canonical string revision");
  const receipt = parseSecretaryCoordinationActionReceipt({
    command_receipt_ref: "command-receipt:opaque-a",
    coordination_event_ref: "coordination-event:opaque-a",
    aggregate_kind_code: "OPEN_LOOP",
    item_ref: "open-loop:fixture-b",
    coordination_revision: "18446744073709551615",
    outcome_code: "SNOOZED",
    replayed: true,
  });
  assert(
    receipt.replayed && receipt.coordination_event_ref === "coordination-event:opaque-a" && !JSON.stringify(receipt).includes("snoozed_until_utc"),
    "scrubbed receipt retains refs/codes/replay only",
  );
  let rejected = false;
  try {
    createSecretaryCoordinationActionRequest({
      action: "INBOX_MARK_READ",
      item_ref: "inbox:fixture-a",
      expected_revision: "1",
      idempotency_key: `idempotency:sha256:${hash("b")}`,
      snoozed_until_utc: "2026-08-10T13:00:00Z",
    });
  } catch {
    rejected = true;
  }
  assert(rejected, "only OPEN_LOOP_SNOOZE may carry a timestamp");

  const mintedIdempotencyKey = await mintSecretaryCoordinationIdempotencyKey();
  assert(
    /^secretary-ui:sha256:[a-f0-9]{64}$/.test(mintedIdempotencyKey),
    "renderer mints the exact opaque idempotency reference accepted by Rust",
  );
  const mintedRequest = createSecretaryCoordinationActionRequest({
    action: "INBOX_MARK_READ",
    item_ref: "inbox:fixture-a",
    expected_revision: "1",
    idempotency_key: mintedIdempotencyKey,
  });
  assert(mintedRequest.idempotency_key === mintedIdempotencyKey, "minted opaque idempotency reference crosses unchanged");

  let plainUuidRejected = false;
  try {
    createSecretaryCoordinationActionRequest({
      action: "INBOX_MARK_READ",
      item_ref: "inbox:fixture-a",
      expected_revision: "1",
      idempotency_key: "secretary-ui:plain-uuid",
    });
  } catch {
    plainUuidRejected = true;
  }
  assert(plainUuidRejected, "plain UUID idempotency keys fail before invoking Rust");
}

console.log("m4c06-secretary-read-model: authoritative M4 parser/view-model assertions passed");
