---
contract_id: attention-decision-v1
version: 1
status: FROZEN_V1
evidence_level: STATIC_OPENING_ONLY
schema_authority: attention_decision_contract_authority
dependencies: ["identity-scope-v1", "command-v1", "event-audit-outbox-v1", "handoff-v1", "object-ref-navigation-v1"]
hold_refs: ["HOLD-ATTENTION-NOTIFICATION-POLICY", "HOLD-RAW-TRANSCRIPT-RETENTION"]
---

# Attention and decision contract v1

## contract.owner

`attention_decision_contract_authority` owns this schema. Source domains retain business decisions;
attention owns coordination projections, and `personal_action_aggregate` alone owns standalone todos.

## contract.schema

```json contract-schema
{
  "schema_authority": "attention_decision_contract_authority",
  "imports": ["ActorId","ScopeRef","ObjectRef","CommandReceipt","CorrelationId"],
  "exports": [
    {"name":"InboxItem","domain_owner":"personal_inbox_projector","required_fields":["inbox_item_id","personal_scope_ref","source_owner_ref","source_ref","source_revision","dedupe_key","received_at","scrubbed_summary_ref","sensitivity","projection_status","source_watermark"],"opening_status":"ABSENT"},
    {"name":"OpenLoop","domain_owner":"secretary_coordination_domain","required_fields":["open_loop_id","personal_scope_ref","source_owner_ref","source_object_ref","source_revision","reason_code","priority_basis","coordination_state","dedupe_key","todo_ref","decision_request_ref","created_at","last_changed_at","closed_at"],"opening_status":"ABSENT"},
    {"name":"OpenLoopTodoRelation","domain_owner":"attention_relation_projector","required_fields":["relation_id","open_loop_id","todo_id","relation_kind","source_owner_ref","source_revision","projection_revision"],"opening_status":"ABSENT"},
    {"name":"Todo","domain_owner":"personal_action_aggregate","required_fields":["todo_id","personal_scope_ref","created_by_actor_id","source_kind","summary_ref","status","revision","idempotency_key","created_at","completed_at"],"opening_status":"ABSENT","constants":{"source_kind":"STANDALONE_USER_CREATED"}},
    {"name":"Notification","domain_owner":"notification_domain","required_fields":["notification_id","personal_scope_ref","source_ref","delivery_channel_kind","delivery_status","read_status","dismiss_status","outbox_item_ref","idempotency_key","created_at"],"opening_status":"ABSENT"},
    {"name":"Reminder","domain_owner":"reminder_domain","required_fields":["reminder_id","personal_scope_ref","source_owner_ref","source_ref","schedule_at","timezone","status","revision","created_at","last_fired_at","snoozed_until"],"opening_status":"ABSENT"},
    {"name":"DecisionRequest","domain_owner":"SOURCE_OWNER_REF","required_fields":["decision_request_id","source_owner_ref","source_object_ref","source_revision","requesting_actor_id","required_actor_ref","required_scope_ref","question_schema_ref","allowed_answer_schema_ref","decision_command_type","status","idempotency_key","created_at","expires_at"],"opening_status":"ABSENT"},
    {"name":"DecisionRequestProjection","domain_owner":"attention_decision_projector","required_fields":["decision_request_id","source_owner_ref","source_object_ref","source_revision","scrubbed_summary_ref","priority_basis","projected_status","source_watermark","last_projected_at"],"opening_status":"ABSENT"}
  ],
  "open_loop_todo_relation": {
    "cardinality": "OpenLoop.todo_ref is zero-or-one; relation records are projections, not ownership",
    "todo_creation_rule": "Todo.source_kind=STANDALONE_USER_CREATED and only explicit CreateStandaloneTodo creates it",
    "source_action_rule": "source-owned actions remain ObjectRef projections and never clone into Todo",
    "no_auto_creation": true,
    "independent_lifecycle": true,
    "no_second_truth": "legacy pending and todo lists remain read-only projections until parity"
  },
  "legal_states": {
    "OpenLoop.coordination_state": ["OPEN","SNOOZED","DISMISSED","CLOSED","QUARANTINED"],
    "Todo.status": ["OPEN","COMPLETED","CANCELLED","ARCHIVED"],
    "DecisionRequestProjection.projected_status": ["PENDING","ROUTED","STALE","DENIED","ANSWERED","CANCELLED","EXPIRED","SUPERSEDED"]
  },
  "decision_owner_rule": "Only DecisionRequest.source_owner_ref may change business decision status; attention routes an expected-revision command and projects the result."
}
```

## contract.truth-source

The source domain owns business state. Inbox, open loops, relations, notifications, reminders, and
decision-request views are correlated, rebuildable coordination state.

## contract.legal-states

The schema `legal_states` map is authoritative. Reopen is a transition from `CLOSED` or `DISMISSED`
back to `OPEN`, not a persistent `REOPENED` state. Todo and OpenLoop lifecycles remain independent.

## contract.cross-scope

Every record preserves personal scope, source owner, source object, and source revision. Answering
routes only to the source-domain decision port with the expected source revision.

## contract.formal-actions

```json action-flow
[
  {"id":"open-attention-loop","command":"OpenAttentionLoop","policy":"attention-policy","state_owner":"secretary_coordination_domain","state_target":"OpenLoop.coordination_state","preconditions":["source_owner_ref is present","source_object_ref is present","source_revision is exact","no Todo is auto-created"],"state":"NONE->OPEN|QUARANTINED","event":"OpenLoopCreated","audit":"SCRUBBED_ATTENTION_RECORD","outbox":{"mode":"OPTIONAL","reason":"notification may be an external effect"},"failure":"FAIL_CLOSED"},
  {"id":"snooze-open-loop","command":"SnoozeOpenLoop","policy":"attention-policy","state_owner":"secretary_coordination_domain","state_target":"OpenLoop.coordination_state","preconditions":["current coordination_state is OPEN","expected OpenLoop revision matches"],"state":"OPEN->SNOOZED","event":"OpenLoopSnoozed","audit":"SCRUBBED_COORDINATION_RECORD","outbox":{"mode":"OPTIONAL","reason":"notification schedule may change"},"failure":"FAIL_CLOSED"},
  {"id":"dismiss-open-loop","command":"DismissOpenLoop","policy":"attention-policy","state_owner":"secretary_coordination_domain","state_target":"OpenLoop.coordination_state","preconditions":["current coordination_state is OPEN or SNOOZED","expected OpenLoop revision matches"],"state":"OPEN|SNOOZED->DISMISSED","event":"OpenLoopDismissed","audit":"SCRUBBED_COORDINATION_RECORD","outbox":{"mode":"OPTIONAL","reason":"notification schedule may change"},"failure":"FAIL_CLOSED"},
  {"id":"close-open-loop","command":"CloseOpenLoop","policy":"attention-policy","state_owner":"secretary_coordination_domain","state_target":"OpenLoop.coordination_state","preconditions":["current coordination_state is OPEN or SNOOZED","expected OpenLoop revision matches"],"state":"OPEN|SNOOZED->CLOSED","event":"OpenLoopClosed","audit":"SCRUBBED_COORDINATION_RECORD","outbox":{"mode":"OPTIONAL","reason":"notification schedule may change"},"failure":"FAIL_CLOSED"},
  {"id":"reopen-open-loop","command":"ReopenOpenLoop","policy":"attention-policy","state_owner":"secretary_coordination_domain","state_target":"OpenLoop.coordination_state","preconditions":["current coordination_state is CLOSED or DISMISSED","expected OpenLoop revision matches"],"state":"CLOSED|DISMISSED->OPEN","event":"OpenLoopReopened","audit":"SCRUBBED_COORDINATION_RECORD","outbox":{"mode":"OPTIONAL","reason":"notification schedule may change"},"failure":"FAIL_CLOSED"},
  {"id":"create-standalone-todo","command":"CreateStandaloneTodo","policy":"personal-action-policy","state_owner":"personal_action_aggregate","state_target":"Todo.status","preconditions":["explicit user command is present","Todo.source_kind is STANDALONE_USER_CREATED","no OpenLoop or source-domain state is mutated"],"state":"NONE->OPEN","event":"StandaloneTodoCreated","audit":"SCRUBBED_TODO_RECORD","outbox":{"mode":"NONE","reason":"todo creation is internal personal state"},"failure":"FAIL_CLOSED"},
  {"id":"complete-standalone-todo","command":"CompleteStandaloneTodo","policy":"personal-action-policy","state_owner":"personal_action_aggregate","state_target":"Todo.status","preconditions":["current Todo.status is OPEN","expected Todo revision matches","no OpenLoop or source-domain state is mutated"],"state":"OPEN->COMPLETED","event":"StandaloneTodoCompleted","audit":"SCRUBBED_TODO_RECORD","outbox":{"mode":"NONE","reason":"todo lifecycle is internal personal state"},"failure":"FAIL_CLOSED"},
  {"id":"cancel-standalone-todo","command":"CancelStandaloneTodo","policy":"personal-action-policy","state_owner":"personal_action_aggregate","state_target":"Todo.status","preconditions":["current Todo.status is OPEN","expected Todo revision matches","no OpenLoop or source-domain state is mutated"],"state":"OPEN->CANCELLED","event":"StandaloneTodoCancelled","audit":"SCRUBBED_TODO_RECORD","outbox":{"mode":"NONE","reason":"todo lifecycle is internal personal state"},"failure":"FAIL_CLOSED"},
  {"id":"archive-standalone-todo","command":"ArchiveStandaloneTodo","policy":"personal-action-policy","state_owner":"personal_action_aggregate","state_target":"Todo.status","preconditions":["current Todo.status is COMPLETED or CANCELLED","expected Todo revision matches","no OpenLoop or source-domain state is mutated"],"state":"COMPLETED|CANCELLED->ARCHIVED","event":"StandaloneTodoArchived","audit":"SCRUBBED_TODO_RECORD","outbox":{"mode":"NONE","reason":"todo lifecycle is internal personal state"},"failure":"FAIL_CLOSED"},
  {"id":"route-decision-answer","command":"RouteDecisionAnswer","policy":"decision-source-routing-policy","state_owner":"attention_decision_projector","state_target":"DecisionRequestProjection.projected_status","preconditions":["source_owner_ref is present","expected source revision matches","required actor and scope match","attention does not mutate DecisionRequest.status"],"state":"PENDING->ROUTED|STALE|DENIED","event":"DecisionAnswerRouted","audit":"SCRUBBED_DECISION_ROUTING_RECORD","outbox":{"mode":"NONE","reason":"the source owner receives a new internal command"},"failure":"FAIL_CLOSED"}
]
```

## contract.events

Attention events carry source references, revision, reason, priority basis, and coordination state only.

## contract.audit

Open, dedupe, snooze, dismiss, close, reopen, explicit todo creation, route, stale, and denial are audited.

## contract.outbox

Only accessible notification/reminder delivery uses outbox; decisions return to source-domain ports.

## contract.sensitivity

Inbox and notification content excludes secrets, transcripts, prompts, provider responses, stdout,
stderr, tool outputs, and raw source payloads.

## contract.idempotency

Source owner, source object, and revision deduplicate open loops and decision requests. Standalone todo
creation uses a separate user-command idempotency namespace.

## contract.failure

Missing source owner, stale revision, invalid actor, or unknown source quarantines or denies without
changing source truth or creating a todo.

## contract.rollback

Open-loop coordination may reopen independently. Todo completion and source decisions roll back only
through their respective owners.

## contract.compatibility

Legacy notification, pending-action, and todo lists remain read-only projections until M4 parity.

## contract.fixtures

`CF-ATTENTION-POS-001`, `CF-ATTENTION-POS-002`, `CF-ATTENTION-POS-003`, and
`CF-ATTENTION-POS-004` prove explicit Todo lifecycle and reachable
OpenLoop close/reopen transitions mutate only their declared owner. `CF-ATTENTION-NEG-001` proves
auto-created Todo or missing source ownership is denied.

## contract.non-goals

M1 does not select notification providers, escalation timing, or implement daily delivery.

## contract.holds

Notification policy and redacted-content retention remain with later owners.
