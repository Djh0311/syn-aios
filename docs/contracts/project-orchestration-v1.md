---
contract_id: project-orchestration-v1
version: 1
status: FROZEN_V1
evidence_level: STATIC_OPENING_ONLY
schema_authority: project_orchestration_contract_authority
dependencies: ["identity-scope-v1", "command-v1", "event-audit-outbox-v1", "role-session-v1", "handoff-v1", "attention-decision-v1", "object-ref-navigation-v1"]
hold_refs: ["HOLD-EXECUTION-GRANT-PERSISTENCE", "HOLD-ORCHESTRATION-RETRY-LEASE", "HOLD-DB-JSON-RUNTIME-TRUTH"]
---

# Project orchestration contract v1

## contract.owner

`project_orchestration_contract_authority` owns this schema. Proposal/authorization, execution,
claim, review, and projection state are deliberately split across their declared domain owners.

## contract.schema

```json contract-schema
{
  "schema_authority": "project_orchestration_contract_authority",
  "imports": ["ActorId","ProjectId","ScopeRef","ObjectRef","RoleSessionId","CommandReceipt","OutboxItem","CorrelationId"],
  "exports": [
    {"name":"OrchestrationId","domain_owner":"project_orchestration","required_fields":["value"],"opening_status":"ABSENT"},
    {"name":"Proposal","domain_owner":"project_orchestration","required_fields":["proposal_id","project_id","orchestration_id","proposer_actor_id","goal_ref","goal_hash","scope_ref","object_refs","requested_commands","risk_class","status","revision","idempotency_key","root_correlation_id","created_by_command_receipt_ref","created_at"],"opening_status":"PARTIAL_LEGACY"},
    {"name":"PlanAuthorization","domain_owner":"project_orchestration","required_fields":["authorization_id","authorization_revision","authorization_decision_id","proposal_id","proposal_revision","project_id","orchestration_id","authorized_scope_ref","allowed_commands","allowed_object_refs","cwd_ref","write_root_refs","risk_constraints","status","expires_at","revoked_at","authorization_hash","created_by_command_receipt_ref"],"opening_status":"PARTIAL_LEGACY"},
    {"name":"WorkflowRun","domain_owner":"execution_aggregate","required_fields":["workflow_run_id","project_id","orchestration_id","authorization_id","authorization_revision","workflow_ref","status","revision","created_by_command_receipt_ref","created_at"],"opening_status":"PARTIAL_LEGACY"},
    {"name":"WorkItem","domain_owner":"execution_aggregate","required_fields":["work_item_id","project_id","orchestration_id","workflow_run_id","source_object_ref","status","revision","created_by_command_receipt_ref"],"opening_status":"PARTIAL_LEGACY"},
    {"name":"PreparedAttempt","domain_owner":"execution_aggregate","required_fields":["attempt_id","project_id","orchestration_id","workflow_run_id","work_item_id","node_id","worker_role_session_id","authorization_id","authorization_revision","scope_ref","object_refs","command_type","state","revision","created_by_command_receipt_ref","created_at"],"opening_status":"PARTIAL_LEGACY"},
    {"name":"ExecutionGrant","domain_owner":"project_orchestration","required_fields":["grant_id","project_id","orchestration_id","workflow_run_id","work_item_id","attempt_id","authorization_id","authorization_revision","principal_actor_id","worker_role_session_id","scope_fingerprint","allowed_commands","cwd_ref","write_root_refs","object_refs","policy_decision_ref","issued_at","expires_at","revoked_at","status","revision","idempotency_key","effect_key","grant_hash","created_by_command_receipt_ref"],"opening_status":"ABSENT"},
    {"name":"Dispatch","domain_owner":"execution_aggregate","required_fields":["dispatch_id","project_id","orchestration_id","workflow_run_id","work_item_id","node_id","attempt_id","grant_id","grant_revision","worker_role_session_id","outbox_item_id","effect_id","state","revision","created_by_command_receipt_ref","created_at"],"opening_status":"PARTIAL_LEGACY"},
    {"name":"ExecutedReport","domain_owner":"claim_ledger","required_fields":["report_id","report_kind","project_id","orchestration_id","workflow_run_id","work_item_id","node_id","dispatch_id","attempt_id","grant_id","worker_role_session_id","authoritative_execution_receipt_ref","authenticated_actor_id","report_hash","observed_attempt_state","claim_status","recorded_by_command_receipt_ref","created_at"],"opening_status":"PARTIAL_LEGACY","constants":{"report_kind":"EXECUTED","acceptable_attempt_states":["SUCCEEDED","FAILED","CANCELLED","TIMED_OUT","UNKNOWN_READBACK"]}},
    {"name":"ManualOfflineClaim","domain_owner":"claim_ledger","required_fields":["claim_id","report_kind","project_id","orchestration_id","authenticated_submitter_id","source_refs","evidence_refs","claim_hash","claim_status","recorded_by_command_receipt_ref","created_at"],"opening_status":"ABSENT","constants":{"report_kind":"MANUAL_OFFLINE"},"forbidden_fields":["dispatch_id","attempt_id","grant_id","worker_role_session_id","authoritative_execution_receipt_ref"]},
    {"name":"Review","domain_owner":"review_domain","required_fields":["review_id","project_id","orchestration_id","workflow_run_id","report_or_claim_ref","reviewer_actor_id","reviewer_role_session_id","readback_refs","review_outcome","reason_code","revision","recorded_by_command_receipt_ref","created_at"],"opening_status":"PARTIAL_LEGACY"},
    {"name":"AuthorizationDecision","domain_owner":"project_authorization","required_fields":["authorization_decision_id","proposal_id","proposal_revision","project_id","orchestration_id","deciding_actor_id","decision","constraint_ref","reason_code","idempotency_key","recorded_by_command_receipt_ref","decided_at"],"opening_status":"PARTIAL_LEGACY","constants":{"idempotency_namespace":"PLAN_AUTHORIZATION"}},
    {"name":"ResultUserDecision","domain_owner":"review_domain","required_fields":["result_decision_id","project_id","orchestration_id","workflow_run_id","review_id","report_ref","deciding_actor_id","decision","result_revision","reason_code","idempotency_key","recorded_by_command_receipt_ref","decided_at"],"opening_status":"PARTIAL_LEGACY","constants":{"idempotency_namespace":"RESULT_ACCEPTANCE"}},
    {"name":"ProjectSummary","domain_owner":"project_projector","required_fields":["project_id","orchestration_id","source_watermark","schema_version","summary_ref","summary_hash","rebuilt_at"],"opening_status":"PARTIAL_LEGACY"}
  ],
  "decision_type_invariant": "AuthorizationDecision and ResultUserDecision have different IDs, owners, namespaces, legal transitions, and receipts; they are never interchangeable.",
  "report_type_invariant": "ExecutedReport and ManualOfflineClaim are claims. ManualOfflineClaim is structurally barred from execution joins; neither type becomes a verified fact without readback and review.",
  "orchestration_identity": {
    "relationship": "PARENT_CHILD",
    "orchestration_id_owner": "project_orchestration",
    "correlation_id_owner": "command_gateway",
    "allocation": "OrchestrationId is allocated at proposal submission; WorkflowRunId is allocated only after approved authorization when CreateAuthorizedRunAndPreparedAttempt commits",
    "join_rule": "Every project record stores orchestration_id and its creating or recording command receipt; receipt correlation resolves to Proposal.root_correlation_id",
    "no_alias": "CorrelationId, object revisions, and OrchestrationId never substitute for one another"
  },
  "atomic_transition_groups": {
    "create-authorized-run-and-prepared-attempt": {
      "command": "CreateAuthorizedRunAndPreparedAttempt",
      "facets": ["WorkflowRun.status","WorkItem.status","PreparedAttempt.state"],
      "commit_semantics": "ALL_OR_NONE",
      "receipt_semantics": "ONE_SHARED_COMMAND_RECEIPT",
      "event_semantics": "ONE_SHARED_EVENT"
    }
  },
  "legal_states": {
    "Proposal.status": ["DRAFT","SUBMITTED","WITHDRAWN","SUPERSEDED"],
    "AuthorizationDecision.decision": ["APPROVED","REJECTED"],
    "PlanAuthorization.status": ["ACTIVE","REVOKED","EXPIRED","SUPERSEDED","QUARANTINED"],
    "WorkflowRun.status": ["CREATED","ACTIVE","SUCCEEDED","FAILED","CANCELLED","TIMED_OUT","UNKNOWN_READBACK"],
    "WorkItem.status": ["READY","ACTIVE","SUCCEEDED","FAILED","CANCELLED","BLOCKED"],
    "PreparedAttempt.state": ["PREPARED_NON_RUNNABLE","GRANT_PENDING_NON_RUNNABLE","GRANT_READY_NON_RUNNABLE","DISPATCHED","RUNNING","SUCCEEDED","FAILED","CANCELLED","TIMED_OUT","UNKNOWN_READBACK"],
    "ExecutionGrant.status": ["MINT_PENDING","ACTIVE","REVOKED","EXPIRED","QUARANTINED"],
    "Dispatch.state": ["PENDING_DELIVERY","DISPATCHED","FAILED","CANCELLED","UNKNOWN_READBACK"],
    "ExecutedReport.claim_status": ["RECORDED_UNVERIFIED","QUARANTINED","SUPERSEDED"],
    "ManualOfflineClaim.claim_status": ["RECORDED_UNVERIFIED","QUARANTINED","SUPERSEDED"],
    "Review.review_outcome": ["VERIFIED","REJECTED","NEEDS_READBACK","UNKNOWN"],
    "ResultUserDecision.decision": ["ACCEPTED_RESULT","REJECTED_RESULT","NEEDS_FOLLOWUP"]
  }
}
```

## contract.truth-source

Server-derived proposal, authorization, grant, dispatch, attempt readback, claim, review, and user-decision
receipts with exact joins are truth. A worker report is a claim, not execution truth.

## contract.legal-states

The schema `legal_states` map is the sole enum authority for every state-bearing export. Each action
transition stays within its `state_target`; conditions owned by another aggregate appear only in
`preconditions`. Claims, reviews, and user decisions therefore never rewrite attempt state or one another.

## contract.cross-scope

Project, orchestration, workflow run, work item, node, attempt, authenticated actor, worker role session,
dispatch, authorization revision, and grant revision must match exactly.

## contract.formal-actions

```json action-flow
[
  {"id":"create-proposal","command":"CreateProposal","policy":"proposal-policy","state_owner":"project_orchestration","state_target":"Proposal.status","preconditions":["proposal_id absent","actor/scope/object refs exact"],"state":"NONE->DRAFT","event":"ProposalCreated","audit":"SCRUBBED_PROPOSAL_RECORD","outbox":{"mode":"NONE","reason":"proposal creation is internal"},"failure":"FAIL_CLOSED"},
  {"id":"submit-proposal","command":"SubmitProposal","policy":"proposal-policy","state_owner":"project_orchestration","state_target":"Proposal.status","preconditions":["proposal revision exact"],"state":"DRAFT->SUBMITTED","event":"ProposalSubmitted","audit":"SCRUBBED_PROPOSAL_RECORD","outbox":{"mode":"NONE","reason":"proposal submission is internal"},"failure":"FAIL_CLOSED"},
  {"id":"record-authorization-decision","command":"RecordAuthorizationDecision","policy":"project-authorization-policy","state_owner":"project_authorization","state_target":"AuthorizationDecision.decision","preconditions":["Proposal.status=SUBMITTED","proposal revision exact","deciding actor authenticated"],"state":"NONE->APPROVED|REJECTED","event":"AuthorizationDecisionRecorded","audit":"SCRUBBED_AUTHORIZATION_RECORD","outbox":{"mode":"NONE","reason":"decision recording is internal and rejection creates no run, attempt, grant, or dispatch"},"failure":"FAIL_CLOSED"},
  {"id":"create-plan-authorization","command":"CreatePlanAuthorization","policy":"project-authorization-policy","state_owner":"project_orchestration","state_target":"PlanAuthorization.status","preconditions":["AuthorizationDecision.decision=APPROVED","proposal and decision revisions exact"],"state":"NONE->ACTIVE|QUARANTINED","event":"PlanAuthorizationCreated","audit":"SCRUBBED_AUTHORIZATION_RECORD","outbox":{"mode":"NONE","reason":"authorization is an internal immutable scope record"},"failure":"FAIL_CLOSED"},
  {"id":"create-authorized-workflow-run","command":"CreateAuthorizedRunAndPreparedAttempt","atomic_group":"create-authorized-run-and-prepared-attempt","policy":"orchestration-policy","state_owner":"execution_aggregate","state_target":"WorkflowRun.status","preconditions":["PlanAuthorization.status=ACTIVE and authorization revision exact","workflow_run_id, work_item_id, and attempt_id are absent","worker RoleSession and all project/orchestration/object joins exact"],"state":"NONE->CREATED","event":"AuthorizedRunAndPreparedAttemptCreated","audit":"SCRUBBED_AUTHORIZED_RUN_RECORD","outbox":{"mode":"NONE","reason":"the atomic group creates non-runnable internal state only"},"failure":"FAIL_CLOSED"},
  {"id":"create-authorized-work-item","command":"CreateAuthorizedRunAndPreparedAttempt","atomic_group":"create-authorized-run-and-prepared-attempt","policy":"orchestration-policy","state_owner":"execution_aggregate","state_target":"WorkItem.status","preconditions":["PlanAuthorization.status=ACTIVE and authorization revision exact","workflow_run_id, work_item_id, and attempt_id are absent","worker RoleSession and all project/orchestration/object joins exact"],"state":"NONE->READY","event":"AuthorizedRunAndPreparedAttemptCreated","audit":"SCRUBBED_AUTHORIZED_RUN_RECORD","outbox":{"mode":"NONE","reason":"the atomic group creates non-runnable internal state only"},"failure":"FAIL_CLOSED"},
  {"id":"create-prepared-attempt","command":"CreateAuthorizedRunAndPreparedAttempt","atomic_group":"create-authorized-run-and-prepared-attempt","policy":"orchestration-policy","state_owner":"execution_aggregate","state_target":"PreparedAttempt.state","preconditions":["PlanAuthorization.status=ACTIVE and authorization revision exact","workflow_run_id, work_item_id, and attempt_id are absent","worker RoleSession and all project/orchestration/object joins exact"],"state":"NONE->PREPARED_NON_RUNNABLE","event":"AuthorizedRunAndPreparedAttemptCreated","audit":"SCRUBBED_AUTHORIZED_RUN_RECORD","outbox":{"mode":"NONE","reason":"the atomic group creates non-runnable internal state only"},"failure":"FAIL_CLOSED"},
  {"id":"begin-grant-binding","command":"BeginAttemptGrantBinding","policy":"execution-grant-policy","state_owner":"execution_aggregate","state_target":"PreparedAttempt.state","preconditions":["PlanAuthorization.status=ACTIVE","attempt revision exact","no active grant bound"],"state":"PREPARED_NON_RUNNABLE->GRANT_PENDING_NON_RUNNABLE","event":"AttemptGrantBindingStarted","audit":"SCRUBBED_GRANT_RECORD","outbox":{"mode":"NONE","reason":"attempt remains non-runnable"},"failure":"FAIL_CLOSED"},
  {"id":"mint-attempt-grant","command":"MintAttemptScopedGrant","policy":"execution-grant-policy","state_owner":"project_orchestration","state_target":"ExecutionGrant.status","preconditions":["PreparedAttempt.state=GRANT_PENDING_NON_RUNNABLE","authorization revision and all exact joins match"],"state":"NONE->MINT_PENDING|QUARANTINED","event":"ExecutionGrantMinted","audit":"SCRUBBED_GRANT_RECORD","outbox":{"mode":"NONE","reason":"grant persistence is internal authorization work"},"failure":"FAIL_CLOSED"},
  {"id":"confirm-grant-readback","command":"ConfirmGrantReadback","policy":"execution-grant-readback-policy","state_owner":"project_orchestration","state_target":"ExecutionGrant.status","preconditions":["grant persistence readback exact","grant hash/revision exact"],"state":"MINT_PENDING->ACTIVE|REVOKED|QUARANTINED","event":"ExecutionGrantReadbackConfirmed","audit":"SCRUBBED_GRANT_RECORD","outbox":{"mode":"NONE","reason":"readback does not dispatch work"},"failure":"FAIL_CLOSED"},
  {"id":"confirm-attempt-grant-binding","command":"ConfirmAttemptGrantBinding","policy":"execution-grant-readback-policy","state_owner":"execution_aggregate","state_target":"PreparedAttempt.state","preconditions":["ExecutionGrant.status=ACTIVE for success; REVOKED or QUARANTINED for recovery","attempt/grant/authorization joins exact"],"state":"GRANT_PENDING_NON_RUNNABLE->GRANT_READY_NON_RUNNABLE|PREPARED_NON_RUNNABLE","event":"AttemptGrantBindingConfirmed","audit":"SCRUBBED_GRANT_RECORD","outbox":{"mode":"NONE","reason":"attempt remains non-runnable until dispatch"},"failure":"FAIL_CLOSED"},
  {"id":"dispatch-attempt","command":"DispatchGrantedAttempt","policy":"execution-grant-policy","state_owner":"execution_aggregate","state_target":"Dispatch.state","preconditions":["PreparedAttempt.state=GRANT_READY_NON_RUNNABLE","ExecutionGrant.status=ACTIVE","grant not expired/revoked","all joins exact"],"state":"NONE->PENDING_DELIVERY|FAILED","event":"ExecutionAttemptDispatchRequested","audit":"SCRUBBED_DISPATCH_RECORD","outbox":{"mode":"REQUIRED","reason":"worker execution is an external effect"},"failure":"FAIL_CLOSED"},
  {"id":"record-dispatch-readback","command":"RecordDispatchReadback","policy":"execution-readback-policy","state_owner":"execution_aggregate","state_target":"Dispatch.state","preconditions":["effect_id and outbox result exact"],"state":"PENDING_DELIVERY->DISPATCHED|FAILED|UNKNOWN_READBACK","event":"DispatchReadbackRecorded","audit":"SCRUBBED_DISPATCH_RECORD","outbox":{"mode":"NONE","reason":"readback records the external delivery result"},"failure":"FAIL_CLOSED"},
  {"id":"mark-attempt-dispatched","command":"MarkAttemptDispatched","policy":"execution-readback-policy","state_owner":"execution_aggregate","state_target":"PreparedAttempt.state","preconditions":["Dispatch.state=DISPATCHED","dispatch/attempt/grant joins exact"],"state":"GRANT_READY_NON_RUNNABLE->DISPATCHED","event":"ExecutionAttemptDispatched","audit":"SCRUBBED_ATTEMPT_RECORD","outbox":{"mode":"NONE","reason":"dispatch effect is already recorded"},"failure":"FAIL_CLOSED"},
  {"id":"record-attempt-readback","command":"RecordExecutionAttemptReadback","policy":"execution-readback-policy","state_owner":"execution_aggregate","state_target":"PreparedAttempt.state","preconditions":["authoritative execution receipt exact","attempt revision exact"],"state":"DISPATCHED|RUNNING->RUNNING|SUCCEEDED|FAILED|CANCELLED|TIMED_OUT|UNKNOWN_READBACK","event":"ExecutionAttemptReadbackRecorded","audit":"SCRUBBED_ATTEMPT_RECORD","outbox":{"mode":"NONE","reason":"authoritative readback alone owns attempt terminal state"},"failure":"FAIL_CLOSED"},
  {"id":"record-executed-report-claim","command":"RecordExecutedReportClaim","policy":"worker-report-binding-policy","state_owner":"claim_ledger","state_target":"ExecutedReport.claim_status","preconditions":["PreparedAttempt.state in ExecutedReport.acceptable_attempt_states","exact project/run/item/node/dispatch/attempt/grant/RoleSession/actor/receipt joins"],"state":"NONE->RECORDED_UNVERIFIED|QUARANTINED","event":"ExecutedReportClaimRecorded","audit":"SCRUBBED_REPORT_RECORD","outbox":{"mode":"NONE","reason":"claim recording never mutates attempt or review state"},"failure":"FAIL_CLOSED"},
  {"id":"record-manual-offline-claim","command":"RecordManualOfflineClaim","policy":"manual-claim-policy","state_owner":"claim_ledger","state_target":"ManualOfflineClaim.claim_status","preconditions":["authenticated submitter exact","execution join fields absent"],"state":"NONE->RECORDED_UNVERIFIED|QUARANTINED","event":"ManualOfflineClaimRecorded","audit":"SCRUBBED_REPORT_RECORD","outbox":{"mode":"NONE","reason":"manual claim has no execution effect"},"failure":"FAIL_CLOSED"},
  {"id":"review-execution-claim","command":"ReviewExecutionClaim","policy":"execution-review-policy","state_owner":"review_domain","state_target":"Review.review_outcome","preconditions":["claim_status=RECORDED_UNVERIFIED","readback refs and reviewer identity exact"],"state":"NONE->VERIFIED|REJECTED|NEEDS_READBACK|UNKNOWN","event":"ExecutionClaimReviewed","audit":"SCRUBBED_REVIEW_RECORD","outbox":{"mode":"NONE","reason":"review never rewrites claim or attempt"},"failure":"FAIL_CLOSED"},
  {"id":"record-result-decision","command":"RecordResultUserDecision","policy":"result-decision-policy","state_owner":"review_domain","state_target":"ResultUserDecision.decision","preconditions":["Review.review_outcome=VERIFIED","review/report/run revisions exact","deciding actor authenticated"],"state":"NONE->ACCEPTED_RESULT|REJECTED_RESULT|NEEDS_FOLLOWUP","event":"ResultUserDecisionRecorded","audit":"SCRUBBED_RESULT_DECISION","outbox":{"mode":"NONE","reason":"source owner applies an accepted result only through a new command"},"failure":"FAIL_CLOSED"}
]
```

## contract.events

Events expose immutable project/orchestration/run/work-item/attempt/grant/dispatch/claim/review references
and their command correlation chain.

## contract.audit

Proposal, authorization, preparation, grant issuance/readback/revoke, dispatch, attempt readback, claim,
review, and both decision types are audited separately.

## contract.outbox

Only a read-back grant in `GRANT_READY_NON_RUNNABLE` may create a dispatch outbox item. The runner
receives a grant ID and the server revalidates all fields.

## contract.sensitivity

Commands, events, claims, and audit omit credentials, secrets, transcripts, prompts, provider responses,
stdout, stderr, and tool outputs.

## contract.idempotency

Proposal, authorization, attempt, grant, dispatch, claim, review, and each decision namespace use stable
independent keys. Divergent duplicate claims quarantine.

## contract.failure

Grant mismatch, failed persistence/readback, revoke/expiry, join mismatch, non-terminal report state,
unknown readback, or manual evidence never becomes success. The path fails closed before dispatch or acceptance.

## contract.rollback

Stop new dispatch, revoke outstanding grants, preserve attempts and receipts, and resume from authoritative
readback without replay by default.

## contract.compatibility

Legacy chains, path locks, phase runners, and report parsers remain labelled adapters until their later packages.

## contract.fixtures

`CF-ORCHESTRATION-POS-001` proves exact grant readback before dispatch. `CF-ORCHESTRATION-POS-002`
and `CF-ORCHESTRATION-POS-003` prove claim and review mutate only their own targets.
`CF-ORCHESTRATION-POS-004` proves the three create facets commit as one atomic group.
`CF-ORCHESTRATION-NEG-001`, `CF-ORCHESTRATION-NEG-002`, `CF-ORCHESTRATION-NEG-003`, and
`CF-ORCHESTRATION-NEG-004` prove manual execution joins,
non-terminal reports, and unverified result decisions fail with no foreign-owner mutation.
`CF-ORCHESTRATION-NEG-005` proves a failed create facet rolls back all three targets.

## contract.non-goals

M1 does not persist grants, run workers, choose leases/retries, execute readback, or accept a runtime result.

## contract.holds

Grant persistence, retry/lease mechanics, and DB/JSON runtime truth remain open.
