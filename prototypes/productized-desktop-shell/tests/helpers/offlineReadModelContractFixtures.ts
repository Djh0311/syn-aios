const executionForbiddenSuggestionKinds: readonly string[] = ["approve", "dispatch", "retry", "stop", "restart", "resume", "send"];

const runQueueForbiddenActionProposalKinds: readonly string[] = ["retry", "stop", "restart", "resume", "send"];

const runQueueReadbackNullStatuses: readonly string[] = ["readback_unavailable", "readback_failed"];

const runQueueFailureNullStatuses: readonly string[] = ["timed_out"];

const runQueueFailureClassifications: readonly string[] = ["duplicate_blocked"];

const runQueueConfirmationKinds: readonly string[] = [
  "execute_confirmation",
  "retry_confirmation",
  "stop_cancel_confirmation",
  "result_confirmation",
  "process_fact_confirmation",
  "memory_candidate_confirmation",
  "memory_formalization_confirmation",
  "capture_compensation_confirmation",
];

const projectCanvasNodeTypes: readonly string[] = [
  "project_goal",
  "director",
  "dev_line",
  "validation_line",
  "review_line",
  "permission_request",
  "blackboard_candidate",
];

const projectCanvasEdgeTypes: readonly string[] = ["responsibility_flow", "blocking_relation"];

const projectCanvasPreviewMutationKinds: readonly string[] = ["workflow_node_mutation", "workflow_edge_mutation"];

const adapterHiddenUnimplementedIds = ["openclaw", "claude-code", "opencode-like"] as const;

const adapterImplementedActionKinds: readonly string[] = ["bind-node-session", "execute-node-dispatch"];

const adapterExpectedCapabilityKinds: readonly string[] = [
  "session_index_read",
  "session_transcript_read",
  "workflow_node_binding",
  "safe_probe_dispatch",
  "user_reviewed_dispatch",
  "workflow_machine_run",
  "permission_decision_record",
  "harness_resource_index",
];

const adapterConfirmationBoundaryFragments: readonly string[] = ["本轮只声明能力", "控制核心", "工作流状态"];

const sessionOperationIds: readonly string[] = [
  "new_session",
  "send_message",
  "stop",
  "restart",
  "resume",
  "export",
  "delete",
  "favorite",
];

const blockedSessionOperationStatuses: readonly string[] = ["available", "available_to_execute", "executable"];

const sessionOperationRequiredWarnings: readonly string[] = [
  "session_operation_boundary_read_model_only",
  "no_session_operation_execution_in_e2",
];

const providerSummaryRequiredWarnings: readonly string[] = [
  "provider_availability_read_model_only",
  "credential_secret_not_read",
  "provider_availability_not_project_authorization",
];

const plannedProviderBoundary = {
  availabilityStatus: "planned",
  credentialStatus: "credential_missing",
  modelStatus: "model_unverified",
  externalCallStatus: "external_call_blocked",
  costRiskStatus: "blocked_until_authorized",
};

const adapterContractMissingItems: readonly string[] = ["runtime_connection_not_implemented"];

const adapterDiagnosticRedactionPolicy = "no_secret_no_raw_transcript_no_provider_payload";

const adapterDataLocationSecretPolicy = "never_read_auth_token_env_keychain_oauth_provider_credentials";

const continuationOperationIds: readonly string[] = ["new_session", "send_message", "resume"];

const plannedContinuationGuardStatuses: readonly string[] = ["blocked", "requires_future_task"];

const plannedContinuationReasonFragments: readonly string[] = ["planned_adapter_blocked"];

const h2DecisionBlockingCheckIds: readonly string[] = ["codex_home_scope", "rollback"];

const h2MissingReadinessItemIds: readonly string[] = [
  "prompt_hash_ref",
  "codex_home_scope",
  "user_confirmation",
  "global_supervisor_confirmation",
];

const rightRailNonSecretaryPanelIds = ["notifications", "todos", "ideas", "audit", "running"] as const;

const transcriptCleaningExpectedIds = {
  eventMessages: ["e2", "e3"] as readonly string[],
  mixedTextParts: ["m1", "m2"] as readonly string[],
  responseItems: ["r1", "r2"] as readonly string[],
  normalizedTurns: ["n4", "n5"] as readonly string[],
};

export const readModelContractFixtures = {
  executionForbiddenSuggestionKinds,
  runQueueForbiddenActionProposalKinds,
  runQueueReadbackNullStatuses,
  runQueueFailureNullStatuses,
  runQueueFailureClassifications,
  runQueueConfirmationKinds,
  projectCanvasNodeTypes,
  projectCanvasEdgeTypes,
  projectCanvasPreviewMutationKinds,
  adapterHiddenUnimplementedIds,
  adapterImplementedActionKinds,
  adapterExpectedCapabilityKinds,
  adapterConfirmationBoundaryFragments,
  sessionOperationIds,
  blockedSessionOperationStatuses,
  sessionOperationRequiredWarnings,
  providerSummaryRequiredWarnings,
  plannedProviderBoundary,
  adapterContractMissingItems,
  adapterDiagnosticRedactionPolicy,
  adapterDataLocationSecretPolicy,
  continuationOperationIds,
  plannedContinuationGuardStatuses,
  plannedContinuationReasonFragments,
  h2DecisionBlockingCheckIds,
  h2MissingReadinessItemIds,
  rightRailNonSecretaryPanelIds,
  transcriptCleaningExpectedIds,
};
