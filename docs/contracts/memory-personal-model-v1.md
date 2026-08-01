---
contract_id: memory-personal-model-v1
version: 1
status: FROZEN_V1
evidence_level: STATIC_OPENING_ONLY
schema_authority: memory_personal_model_contract_authority
dependencies: ["identity-scope-v1", "command-v1", "event-audit-outbox-v1", "attention-decision-v1", "object-ref-navigation-v1"]
hold_refs: ["HOLD-MEMORY-PROMOTION-POLICY", "HOLD-DB-JSON-RUNTIME-TRUTH", "HOLD-RAW-TRANSCRIPT-RETENTION"]
---

# Memory and personal-model contract v1

## contract.owner

`memory_personal_model_contract_authority` owns this schema. Capture, governance, formal-memory,
personal-fact, and model-assertion state each have a distinct single writer below.

## contract.schema

```json contract-schema
{
  "schema_authority": "memory_personal_model_contract_authority",
  "imports": ["ActorId","ScopeRef","ObjectRef","CommandReceipt","CorrelationId"],
  "exports": [
    {"name":"Observation","domain_owner":"memory_capture","required_fields":["observation_id","actor_id","scope_ref","source_event_ref","source_object_ref","content_ref","content_hash","classification","sensitivity","policy_input_ref","status","captured_at"],"opening_status":"PARTIAL_LEGACY"},
    {"name":"MemoryCandidate","domain_owner":"memory_governance","required_fields":["candidate_id","candidate_kind","observation_refs","scope_ref","candidate_reason","policy_result_ref","conflict_refs","content_ref","content_hash","status","revision","created_at"],"opening_status":"PARTIAL_LEGACY"},
    {"name":"FormalMemory","domain_owner":"formal_memory_repository","required_fields":["formal_memory_id","scope_ref","memory_kind","current_version_id","status","revision","created_at"],"opening_status":"PARTIAL_LEGACY"},
    {"name":"PersonalFact","domain_owner":"personal_fact_domain","required_fields":["personal_fact_id","version_id","subject_actor_id","scope_ref","statement_ref","statement_hash","provenance_kind","source_refs","valid_from","valid_until","status","correction_of_version_id","created_at"],"opening_status":"ABSENT"},
    {"name":"ModelAssertion","domain_owner":"personal_model_domain","required_fields":["model_assertion_id","version_id","subject_actor_id","scope_ref","inference_ref","inference_hash","evidence_refs","confidence","valid_from","valid_until","contestability","status","supersedes_version_id","created_at"],"opening_status":"ABSENT"},
    {"name":"MemoryPolicyResult","domain_owner":"memory_governance","required_fields":["policy_result_id","candidate_id","policy_version","decision","reason_codes","required_confirmation","sensitivity_result","evaluated_at"],"opening_status":"ABSENT"},
    {"name":"MemoryConflict","domain_owner":"memory_governance","required_fields":["conflict_id","left_ref","right_ref","conflict_kind","status","resolution_ref","detected_at"],"opening_status":"ABSENT"},
    {"name":"MemoryVersion","domain_owner":"formal_memory_repository","required_fields":["version_id","formal_memory_id","candidate_id","source_refs","content_ref","content_hash","sensitivity","status","supersedes_version_id","created_by_actor_id","created_at"],"opening_status":"PARTIAL_LEGACY"}
  ],
  "type_invariants": [
    "Observation is never a fact",
    "a model inference may create only ModelAssertion, never PersonalFact",
    "PersonalFact.provenance_kind is EXPLICIT_USER or RELIABLE_DETERMINISTIC_SOURCE",
    "PersonalFact and ModelAssertion have distinct IDs, owners, versions, correction paths, and withdrawal histories"
  ]
}
```

## contract.truth-source

Versioned accepted records plus source references and policy receipts are truth. Capture, inference,
confidence, and candidate state are not personal facts.

## contract.legal-states

Candidates are `PENDING_POLICY`, `NEEDS_USER_DECISION`, `ACCEPTED`, `REJECTED`, `QUARANTINED`,
or `SUPERSEDED`. Memory versions are `ACTIVE`, `FROZEN`, `SUPERSEDED`, `DEPRECATED`, or `WITHDRAWN`.

## contract.cross-scope

Personal, project, workflow, and session scopes never merge implicitly. Export requires explicit policy.

## contract.formal-actions

```json action-flow
[
  {"id":"capture-observation","command":"CaptureObservation","policy":"memory-capture-policy","state":"NONE->CAPTURED|QUARANTINED","event":"ObservationCaptured","audit":"SCRUBBED_MEMORY_RECORD","outbox":{"mode":"NONE","reason":"capture is internal and not a fact"},"failure":"FAIL_CLOSED"},
  {"id":"evaluate-candidate","command":"EvaluateMemoryCandidate","policy":"memory-promotion-policy","state":"PENDING_POLICY->ACCEPTED|REJECTED|NEEDS_USER_DECISION|QUARANTINED","event":"MemoryPolicyResultRecorded","audit":"SCRUBBED_MEMORY_DECISION","outbox":{"mode":"NONE","reason":"policy evaluation changes governed internal state"},"failure":"FAIL_CLOSED"},
  {"id":"create-formal-memory","command":"CreateFormalMemoryFromAcceptedCandidate","policy":"formal-memory-policy","state":"ACCEPTED->ACTIVE|QUARANTINED","event":"FormalMemoryCreated","audit":"SCRUBBED_MEMORY_RECORD","outbox":{"mode":"NONE","reason":"formal memory is internal governed state"},"failure":"FAIL_CLOSED"},
  {"id":"create-personal-fact-version","command":"CreatePersonalFactVersion","policy":"personal-fact-provenance-policy","state":"ACCEPTED->ACTIVE|QUARANTINED","event":"PersonalFactVersionCreated","audit":"SCRUBBED_PERSONAL_FACT_RECORD","outbox":{"mode":"NONE","reason":"fact creation is internal and requires provenance"},"failure":"FAIL_CLOSED"},
  {"id":"create-model-assertion-version","command":"CreateModelAssertionVersion","policy":"model-assertion-policy","state":"ACCEPTED->ACTIVE|QUARANTINED","event":"ModelAssertionVersionCreated","audit":"SCRUBBED_MODEL_ASSERTION_RECORD","outbox":{"mode":"NONE","reason":"inference remains a separately typed internal assertion"},"failure":"FAIL_CLOSED"},
  {"id":"correct-personal-fact","command":"CorrectPersonalFactVersion","policy":"personal-fact-provenance-policy","state":"ACTIVE->SUPERSEDED","event":"PersonalFactCorrected","audit":"SCRUBBED_PERSONAL_FACT_RECORD","outbox":{"mode":"NONE","reason":"correction creates a new version"},"failure":"FAIL_CLOSED"},
  {"id":"correct-model-assertion","command":"CorrectModelAssertionVersion","policy":"model-assertion-policy","state":"ACTIVE->SUPERSEDED","event":"ModelAssertionCorrected","audit":"SCRUBBED_MODEL_ASSERTION_RECORD","outbox":{"mode":"NONE","reason":"correction preserves the inference history"},"failure":"FAIL_CLOSED"}
]
```

## contract.events

Events contain classifications, source references/hashes, policy-result references, version IDs, and conflicts.

## contract.audit

Capture, policy result, reject, quarantine, promote, correct, conflict, supersede, withdraw, and export are audited.

## contract.outbox

Memory writes have no direct external effect. Skill or export actions use their own commands and outbox items.

## contract.sensitivity

Raw transcripts, prompts, provider responses, stdout, stderr, tool outputs, credentials, and secrets are forbidden.

## contract.idempotency

Source reference plus canonical content hash deduplicates observations, candidates, and versions.

## contract.failure

Unknown classification, absent policy, conflict, cross-scope input, or sensitive material quarantines.

## contract.rollback

Correction creates a new typed version and preserves prior accepted history and audit.

## contract.compatibility

Legacy observation, candidate, and formal-memory sidecars remain adapters until M2/M7 parity.

## contract.fixtures

`CF-MEMORY-POS-001` proves policy-governed promotion with fact/inference separation.
`CF-MEMORY-NEG-001` proves missing policy or inferred-as-fact input is quarantined.

## contract.non-goals

M1 does not set thresholds, retention days, skill activation, storage, or cutover.

## contract.holds

Promotion policy, DB/JSON truth, and content retention remain open.
