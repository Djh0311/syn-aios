import { invoke } from "@tauri-apps/api/core";
import {
  knowledgeOpenRelayAckRequest,
  type KnowledgeOpenRelayIntent,
  type KnowledgeOpenRelayOutcome,
} from "./knowledgeOpenRelay";
import {
  AGENT_CODEX_WORKSPACE_WRITE_PROFILE,
  SUPERVISOR_READ_ONLY_PROFILE,
  type AgentConversationTransportContext,
  type ConversationTransportAttemptRequest,
  type ConversationTransportLegacyExistingStartRequest,
  type ConversationTransportNewStartRequest,
  type ConversationTransportReceipt,
  type SupervisorConversationTransportContext,
} from "./conversationTransport";
import {
  createRoleSessionContinuationStartRequest,
  createRoleSessionDetailRequest,
  createRoleSessionDirectoryRequest,
  parseRoleSessionDetail,
  parseRoleSessionDirectory,
  roleSessionDetailMatchesRequest,
  roleSessionDirectoryMatchesRequest,
  type RoleSessionContinuationStartRequest,
  type RoleSessionDetail,
  type RoleSessionDetailRequest,
  type RoleSessionDirectory,
  type RoleSessionDirectoryRequest,
} from "./roleSessionReadModel";
import {
  createSecretarySourceRouteRequest,
  createSecretaryCoordinationActionRequest,
  createSecretaryPersonalObjectRequest,
  parseSecretaryLegacyReadCompatibilityReportEnvelope,
  parseSecretaryCoordinationActionReceipt,
  parseSecretaryDailyReportEnvelope,
  parseSecretaryHomeContextEnvelope,
  parseSecretarySourceRouteResolution,
  type M4SecretaryDailyReportEnvelopeDto,
  type M4LegacyReadCompatibilityReportEnvelopeDto,
} from "./secretaryReadModel";
import type { PageReadModelQueryInput, PageReadModelQueryResult } from "./pageReadModel";
import type {
  M4SecretaryCoordinationActionReceiptDto,
  M4SecretaryCoordinationActionRequestDto,
  M4SecretaryHomeContextEnvelopeDto,
  M4SecretaryPersonalObjectRequestDto,
  M4SecretarySourceRouteRequestDto,
  M4SecretarySourceRouteResolutionDto,
} from "./types/m4Secretary";
import type {
  AdoptMemoryCandidateInput,
  AdoptMemoryCandidateOutput,
  AutoDispatchGuardInput,
  AutoDispatchGuardResult,
  AuthorizedPreparedDispatchResult,
  BlackboardCandidateStoreV1,
  CanvasDefinition,
  CanvasNodeDispatchRequest,
  ExperimentNodeDispatchRequest,
  ProjectWorkflowNodeRunRequest,
  ProjectWorkflowChainRunRequest,
  ProjectWorkflowChainStopRequest,
  ProjectWorkflowChainRunResult,
  ProjectWorkflowChainStatus,
  StartProjectDirectorChainRequest,
  DirectorChainOutcome,
  RunProjectConsultationRequest,
  SubmitSupervisorResidentAnswerRequest,
  SupervisorResidentAnswerOutcome,
  AutoAdvanceAuthorizedRoleLoopRequest,
  AutoAdvanceRoleLoopOutcome,
  ProjectDirectorFailedActionRequest,
  ProjectDirectorFailedActionOutcome,
  ConfirmAndStartAuthorizedRunRequest,
  PreviewPendingProposalDirectorPlanRequest,
  PreviewPendingProposalDirectorPlanOutcome,
  RunGlobalSupervisorReviewRequest,
  GlobalSupervisorReviewOutcome,
  RunGlobalSupervisorBoundaryReviewRequest,
  GlobalSupervisorBoundaryReviewOutcome,
  SupervisorPilotLaunchReceipt,
  SupervisorPilotLaunchRequest,
  SupervisorPilotReadModel,
  SupervisorPilotReadModelRequest,
  ListProjectRunHistoryRequest,
  RunHistoryList,
  GlobalSupervisorReviewStoreV1,
  SecretaryExplainOutcome,
  ProjectConsultationProposal,
  ProjectWorkflowListItem,
  SubmitProjectWorkflowDraftRequest,
  WorkflowTemplate,
  WorkflowTemplateSummary,
  CodexSessionPage,
  CodexSessionPageRequest,
  CodexTranscript,
  CodexTranscriptPageRequest,
  CreateFormalMemoryRecordInput,
  CreateFormalMemoryRecordOutput,
  CreateMemoryCandidateInput,
  CreateMemoryCandidateFromObservationInput,
  CreateMemoryCandidateFromObservationOutput,
  CreateMemoryCandidateOutput,
  CreateObservationInput,
  CreateObservationOutput,
  CaptureMemoryEventInput,
  CaptureMemoryEventOutput,
  ConfirmRealExecutionProductCommandInput,
  ConfirmControlledSessionContinuationInput,
  ConfirmControlledSessionContinuationOutput,
  FormalMemoryLifecycleInput,
  FormalMemoryLifecycleOutput,
  FormalMemoryLifecyclePreview,
  FormalMemoryLifecyclePreviewInput,
  CreatePlanAuthorizationInput,
  CreatePlanAuthorizationOutput,
  CreateProjectConsultationProposalInput,
  CreateProjectConsultationProposalOutput,
  FormalMemoryStoreV1,
  GenerateStageCAcceptanceSummaryInput,
  GlobalFinalResultReviewInput,
  H5ProjectWorkflowDispatchPreview,
  H5ProjectWorkflowDispatchPreviewInput,
  InspectControlledSessionContinuationRealResumeInput,
  InspectControlledSessionContinuationRealResumeOutput,
  MemoryCandidateStoreV1,
  MemoryCaptureStoreV1,
  MemoryEntityRelationPreviewOutput,
  MemoryEntityRelationStoreV1,
  MemoryLintRunInput,
  MemoryLintRunOutput,
  MemoryLintStoreV1,
  MemoryPatternStoreV1,
  MaturePatternPreviewOutput,
  ObservationStoreV1,
  OfflineDirectorReviewRequest,
  OperationControlDecisionRequest,
  ManualRelayConfirmInput,
  ManualRelayConfirmation,
  ManualRelayPreview,
  ManualRelayPreviewInput,
  ManualRelayPollInput,
  ManualRelayReceipt,
  ManualRelayGuiDirectNewSessionInput,
  ManualRelayGuiDirectRunInput,
  ManualRelayRunInput,
  ManualRelayStopInput,
  OfflineRoleDispatchRequest,
  OfflineRoleResultHandoffRequest,
  PendingAction,
  PlanAuthorizationStoreV1,
  PrepareRealExecutionProductCommandInput,
  PrepareAuthorizedAutoDispatchInput,
  PreviewRealExecutionProductCommandInput,
  ProjectDirectorProcessFactDecisionInput,
  ProjectDirectorProcessFactDecisionResult,
  PreviewProjectDirectorTaskPlanInput,
  ProjectDirectorTaskPlan,
  ProjectConsultationProposalMarkdown,
  ProjectConsultationProposalStoreV1,
  ProjectWorkflowAutomationInput,
  ProjectWorkflowAutomationResult,
  RecordBlackboardCandidateDecisionInput,
  RecordBlackboardCandidateDecisionOutput,
  RecordGlobalBoundaryReviewInput,
  RecordGlobalBoundaryReviewOutput,
  RecordMemoryCandidateDecisionInput,
  RecordMemoryCandidateDecisionOutput,
  PreviewMemoryEntityRelationCandidatesInput,
  RecordRealExecutionProductCommandDecisionInput,
  RecordMemoryEntityAliasDecisionInput,
  RecordMemoryEntityAliasDecisionOutput,
  RecordMemoryEntityMergeDecisionInput,
  RecordMemoryEntityMergeDecisionOutput,
  PreviewMaturePatternsInput,
  RecordMemoryRelationCandidateDecisionInput,
  RecordMemoryRelationCandidateDecisionOutput,
  RecordMaturePatternDecisionInput,
  RecordMaturePatternDecisionOutput,
  RecordPlanAuthorizationGlobalBoundaryReviewInput,
  RecordPlanAuthorizationOutput,
  RecordPlanAuthorizationUserConfirmationInput,
  RecordProjectConsultationProposalDecisionInput,
  RecordProjectConsultationProposalDecisionOutput,
  RenderProjectConsultationProposalMarkdownInput,
  RevokePlanAuthorizationInput,
  RealExecutionProductCommandDecisionOutput,
  RealExecutionProductCommandPhaseAOutput,
  RealExecutionProductCommandPhaseBOutput,
  RealExecutionProductCommandPrepareOutput,
  RealExecutionProductCommandPreview,
  RunControlledSessionContinuationRealResumePhaseAInput,
  RunControlledSessionContinuationRealResumePhaseAOutput,
  RunControlledSessionContinuationRealResumePhaseBInput,
  RunControlledSessionContinuationRealResumePhaseBOutput,
  RunControlledSessionContinuationStubInput,
  RunControlledSessionContinuationStubOutput,
  RunRealExecutionProductCommandPhaseAInput,
  RunRealExecutionProductCommandPhaseBInput,
  RunRealExecutionProductCommandNewSessionPhaseBInput,
  SessionContinuationStoreV1,
  TaskMemoryPacketBuildInput,
  TaskMemoryPacketBuildOutput,
  TaskDraftRequest,
  TaskPackageDispatchFieldsCorrectionRequest,
  TaskPackageDispatchReadiness,
  TaskPackageDispatchReadinessRequest,
  TaskPackageFileGenerationRequest,
  TaskPackageFileGenerationResult,
  TaskPackageFieldsUpdateRequest,
  TaskPackagePreview,
  TaskPackagePreviewRequest,
  UserResultDecisionInput,
  WorkbenchSnapshot,
  WorkItemStateUpdateRequest,
  WorkflowRunCheck,
  WorkflowRunCheckRequest,
  WorkflowNodeSessionBindRequest,
  WorkflowDispatchDirectorReviewRequest,
  WorkflowPermissionDecisionRequest,
  WorkflowNodeDispatchResult,
  WorkflowNodeSessionUnbindRequest,
  WorkflowStateMutationResult,
  WorkflowStateSnapshot,
  WorkerStructuredReportInput,
} from "./types";

export function loadWorkbenchSnapshot(): Promise<WorkbenchSnapshot> {
  ensureTauriRuntime();
  return invoke<WorkbenchSnapshot>("load_workbench_snapshot");
}

export type SystemStatusReadModel = {
  storage_mode: "db_primary" | "json_only";
  storage_healthy: boolean;
  observation_day: number;
  last_degradation: { at_ms: number; reason_human: string } | null;
  recent_catches: Array<{ at_ms: number; summary: string }>;
  gate_summary: string | null;
  warnings: string[];
};

export function loadSystemStatusReadModel(): Promise<SystemStatusReadModel> {
  ensureTauriRuntime();
  return invoke<SystemStatusReadModel>("load_system_status_read_model");
}

export type AuditLedgerReadModelRequest = {
  page: number;
  page_size?: number;
  kind_filter?: string;
};

export type AuditLedgerReadModelItem = {
  at_ms: number;
  source: string;
  event_type: string;
  human_summary: string;
  target_ref: string | null;
  raw_json: unknown;
};

export type AuditLedgerReadModel = {
  total: number;
  items: AuditLedgerReadModelItem[];
  page: number;
  page_size: number;
  storage_mode: "db_primary" | "json_only";
  kinds: string[];
  warnings: string[];
};

export function queryAuditLedgerReadModel(
  request: AuditLedgerReadModelRequest,
): Promise<AuditLedgerReadModel> {
  ensureTauriRuntime();
  return invoke<AuditLedgerReadModel>("query_audit_ledger_read_model", { request });
}

export function queryWorkbenchPageReadModel(
  request: PageReadModelQueryInput,
): Promise<PageReadModelQueryResult> {
  ensureTauriRuntime();
  return invoke<PageReadModelQueryResult>("query_workbench_page_read_model", { request });
}

export function loadSessionContinuationStore(): Promise<SessionContinuationStoreV1> {
  ensureTauriRuntime();
  return invoke<SessionContinuationStoreV1>("load_session_continuation_store");
}

export function confirmControlledSessionContinuation(
  request: ConfirmControlledSessionContinuationInput,
): Promise<ConfirmControlledSessionContinuationOutput> {
  ensureTauriRuntime();
  return invoke<ConfirmControlledSessionContinuationOutput>("confirm_controlled_session_continuation", { request });
}

export function runControlledSessionContinuationStub(
  request: RunControlledSessionContinuationStubInput,
): Promise<RunControlledSessionContinuationStubOutput> {
  ensureTauriRuntime();
  return invoke<RunControlledSessionContinuationStubOutput>("run_controlled_session_continuation_stub", { request });
}

export function inspectControlledSessionContinuationRealResumeAuthorization(
  request: InspectControlledSessionContinuationRealResumeInput,
): Promise<InspectControlledSessionContinuationRealResumeOutput> {
  ensureTauriRuntime();
  return invoke<InspectControlledSessionContinuationRealResumeOutput>(
    "inspect_controlled_session_continuation_real_resume_authorization",
    { request },
  );
}

export function runControlledSessionContinuationRealResumePhaseA(
  request: RunControlledSessionContinuationRealResumePhaseAInput,
): Promise<RunControlledSessionContinuationRealResumePhaseAOutput> {
  ensureTauriRuntime();
  return invoke<RunControlledSessionContinuationRealResumePhaseAOutput>(
    "run_controlled_session_continuation_real_resume_phase_a",
    { request },
  );
}

export function runControlledSessionContinuationRealResumePhaseB(
  request: RunControlledSessionContinuationRealResumePhaseBInput,
): Promise<RunControlledSessionContinuationRealResumePhaseBOutput> {
  ensureTauriRuntime();
  return invoke<RunControlledSessionContinuationRealResumePhaseBOutput>(
    "run_controlled_session_continuation_real_resume_phase_b",
    { request },
  );
}

export function loadCodexSessionTranscript(threadId: string): Promise<CodexTranscript> {
  ensureTauriRuntime();
  return invoke<CodexTranscript>("load_codex_session_transcript", { threadId });
}

export function loadCodexSessionTranscriptPage(request: CodexTranscriptPageRequest): Promise<CodexTranscript> {
  ensureTauriRuntime();
  return invoke<CodexTranscript>("load_codex_session_transcript_page", { request });
}

export function loadCodexSessionPage(request: CodexSessionPageRequest): Promise<CodexSessionPage> {
  ensureTauriRuntime();
  return invoke<CodexSessionPage>("load_codex_session_page", { request });
}

export function runPathAction(action: PendingAction): Promise<string> {
  ensureTauriRuntime();
  const payload = { request: { path: action.path } };

  if (action.kind === "copy") {
    return invoke<string>("copy_indexed_path", payload);
  }

  if (action.kind === "open-project") {
    return invoke<string>("open_indexed_project", payload);
  }

  return invoke<string>("reveal_indexed_rollout", payload);
}

export function loadWorkflowStateSnapshot(): Promise<WorkflowStateSnapshot> {
  ensureTauriRuntime();
  return invoke<WorkflowStateSnapshot>("load_workflow_state_snapshot");
}

export function loadPlanAuthorizationStore(): Promise<PlanAuthorizationStoreV1> {
  ensureTauriRuntime();
  return invoke<PlanAuthorizationStoreV1>("load_plan_authorization_store");
}

export function createPlanAuthorization(request: CreatePlanAuthorizationInput): Promise<CreatePlanAuthorizationOutput> {
  ensureTauriRuntime();
  return invoke<CreatePlanAuthorizationOutput>("create_plan_authorization", { request });
}

export function recordPlanAuthorizationUserConfirmation(
  request: RecordPlanAuthorizationUserConfirmationInput,
): Promise<RecordPlanAuthorizationOutput> {
  ensureTauriRuntime();
  return invoke<RecordPlanAuthorizationOutput>("record_plan_authorization_user_confirmation", { request });
}

export function recordPlanAuthorizationGlobalBoundaryReview(
  request: RecordPlanAuthorizationGlobalBoundaryReviewInput,
): Promise<RecordPlanAuthorizationOutput> {
  ensureTauriRuntime();
  return invoke<RecordPlanAuthorizationOutput>("record_plan_authorization_global_boundary_review", { request });
}

export function recordGlobalBoundaryReview(request: RecordGlobalBoundaryReviewInput): Promise<RecordGlobalBoundaryReviewOutput> {
  ensureTauriRuntime();
  return invoke<RecordGlobalBoundaryReviewOutput>("record_global_boundary_review", { request });
}

export function revokePlanAuthorization(request: RevokePlanAuthorizationInput): Promise<RecordPlanAuthorizationOutput> {
  ensureTauriRuntime();
  return invoke<RecordPlanAuthorizationOutput>("revoke_plan_authorization", { request });
}

export function inspectAutoDispatchAuthorization(request: AutoDispatchGuardInput): Promise<AutoDispatchGuardResult> {
  ensureTauriRuntime();
  return invoke<AutoDispatchGuardResult>("inspect_auto_dispatch_authorization", { request });
}

export function previewProjectDirectorTaskPlan(
  request: PreviewProjectDirectorTaskPlanInput,
): Promise<ProjectDirectorTaskPlan> {
  ensureTauriRuntime();
  return invoke<ProjectDirectorTaskPlan>("preview_project_director_task_plan", { request });
}

export function prepareAuthorizedAutoDispatch(
  request: PrepareAuthorizedAutoDispatchInput,
): Promise<AuthorizedPreparedDispatchResult> {
  ensureTauriRuntime();
  return invoke<AuthorizedPreparedDispatchResult>("prepare_authorized_auto_dispatch", { request });
}

export function previewH5ProjectWorkflowDispatch(
  request: H5ProjectWorkflowDispatchPreviewInput,
): Promise<H5ProjectWorkflowDispatchPreview> {
  ensureTauriRuntime();
  return invoke<H5ProjectWorkflowDispatchPreview>("preview_h5_project_workflow_dispatch", { request });
}

export function previewRealExecutionProductCommand(
  request: PreviewRealExecutionProductCommandInput,
): Promise<RealExecutionProductCommandPreview> {
  ensureTauriRuntime();
  return invoke<RealExecutionProductCommandPreview>("preview_real_execution_product_command", { request });
}

export function prepareRealExecutionProductCommand(
  request: PrepareRealExecutionProductCommandInput,
): Promise<RealExecutionProductCommandPrepareOutput> {
  ensureTauriRuntime();
  return invoke<RealExecutionProductCommandPrepareOutput>("prepare_real_execution_product_command", { request });
}

export function recordRealExecutionProductCommandDecision(
  request: RecordRealExecutionProductCommandDecisionInput,
): Promise<RealExecutionProductCommandDecisionOutput> {
  ensureTauriRuntime();
  return invoke<RealExecutionProductCommandDecisionOutput>("record_real_execution_product_command_decision", { request });
}

export function confirmRealExecutionProductCommand(
  request: ConfirmRealExecutionProductCommandInput,
): Promise<RealExecutionProductCommandDecisionOutput> {
  ensureTauriRuntime();
  return invoke<RealExecutionProductCommandDecisionOutput>("confirm_real_execution_product_command", { request });
}

export function runRealExecutionProductCommandPhaseA(
  request: RunRealExecutionProductCommandPhaseAInput,
): Promise<RealExecutionProductCommandPhaseAOutput> {
  ensureTauriRuntime();
  return invoke<RealExecutionProductCommandPhaseAOutput>("run_real_execution_product_command_phase_a", { request });
}

export function runRealExecutionProductCommandPhaseB(
  request: RunRealExecutionProductCommandPhaseBInput,
): Promise<RealExecutionProductCommandPhaseBOutput> {
  ensureTauriRuntime();
  return invoke<RealExecutionProductCommandPhaseBOutput>("run_real_execution_product_command_phase_b", { request });
}

export function runRealExecutionProductCommandNewSessionPhaseB(
  request: RunRealExecutionProductCommandNewSessionPhaseBInput,
): Promise<RealExecutionProductCommandPhaseBOutput> {
  ensureTauriRuntime();
  return invoke<RealExecutionProductCommandPhaseBOutput>("run_real_execution_product_command_new_session_phase_b", { request });
}

export function runProjectWorkflowAutomationPhaseA(
  request: ProjectWorkflowAutomationInput,
): Promise<ProjectWorkflowAutomationResult> {
  ensureTauriRuntime();
  return invoke<ProjectWorkflowAutomationResult>("run_project_workflow_automation_phase_a", { request });
}

export function recordWorkerStructuredReport(request: WorkerStructuredReportInput): Promise<WorkflowStateMutationResult> {
  ensureTauriRuntime();
  return invoke<WorkflowStateMutationResult>("record_worker_structured_report", { request });
}

export function recordProjectDirectorProcessFactDecision(
  request: ProjectDirectorProcessFactDecisionInput,
): Promise<ProjectDirectorProcessFactDecisionResult> {
  ensureTauriRuntime();
  return invoke<ProjectDirectorProcessFactDecisionResult>("record_project_director_process_fact_decision", { request });
}

export function recordGlobalFinalResultReview(request: GlobalFinalResultReviewInput): Promise<WorkflowStateMutationResult> {
  ensureTauriRuntime();
  return invoke<WorkflowStateMutationResult>("record_global_final_result_review", { request });
}

export function recordUserResultDecision(request: UserResultDecisionInput): Promise<WorkflowStateMutationResult> {
  ensureTauriRuntime();
  return invoke<WorkflowStateMutationResult>("record_user_result_decision", { request });
}

export function generateStageCAcceptanceSummary(
  request: GenerateStageCAcceptanceSummaryInput,
): Promise<WorkflowStateMutationResult> {
  ensureTauriRuntime();
  return invoke<WorkflowStateMutationResult>("generate_stage_c_acceptance_summary", { request });
}

export function loadProjectConsultationProposalStore(): Promise<ProjectConsultationProposalStoreV1> {
  ensureTauriRuntime();
  return invoke<ProjectConsultationProposalStoreV1>("load_project_consultation_proposal_store");
}

// B3·主管复核整店只读 load（秘书「待你拍板」面数据源之一·soft 语义损坏空店不炸）。纯封装无逻辑。
export function loadGlobalSupervisorReviewStore(): Promise<GlobalSupervisorReviewStoreV1> {
  ensureTauriRuntime();
  return invoke<GlobalSupervisorReviewStoreV1>("load_global_supervisor_review_store");
}

// B3·秘书按需解释（唯一烧额度处·用户点才花·后端盘读事实·零持久）。纯封装无逻辑。
export function runSecretaryExplain(): Promise<SecretaryExplainOutcome> {
  ensureTauriRuntime();
  return invoke<SecretaryExplainOutcome>("run_secretary_explain");
}

export function createProjectConsultationProposal(
  request: CreateProjectConsultationProposalInput,
): Promise<CreateProjectConsultationProposalOutput> {
  ensureTauriRuntime();
  return invoke<CreateProjectConsultationProposalOutput>("create_project_consultation_proposal", { request });
}

export function renderProjectConsultationProposalMarkdown(
  request: RenderProjectConsultationProposalMarkdownInput,
): Promise<ProjectConsultationProposalMarkdown> {
  ensureTauriRuntime();
  return invoke<ProjectConsultationProposalMarkdown>("render_project_consultation_proposal_markdown", { request });
}

export function recordProjectConsultationProposalDecision(
  request: RecordProjectConsultationProposalDecisionInput,
): Promise<RecordProjectConsultationProposalDecisionOutput> {
  ensureTauriRuntime();
  return invoke<RecordProjectConsultationProposalDecisionOutput>("record_project_consultation_proposal_decision", { request });
}

export function loadBlackboardCandidateStore(): Promise<BlackboardCandidateStoreV1> {
  ensureTauriRuntime();
  return invoke<BlackboardCandidateStoreV1>("load_blackboard_candidate_store");
}

export function recordBlackboardCandidateDecision(
  request: RecordBlackboardCandidateDecisionInput,
): Promise<RecordBlackboardCandidateDecisionOutput> {
  ensureTauriRuntime();
  return invoke<RecordBlackboardCandidateDecisionOutput>("record_blackboard_candidate_decision", { request });
}

export function loadObservationStore(): Promise<ObservationStoreV1> {
  ensureTauriRuntime();
  return invoke<ObservationStoreV1>("load_observation_store");
}

export function loadMemoryCaptureStore(): Promise<MemoryCaptureStoreV1> {
  ensureTauriRuntime();
  return invoke<MemoryCaptureStoreV1>("load_memory_capture_store");
}

export function captureMemoryEvent(request: CaptureMemoryEventInput): Promise<CaptureMemoryEventOutput> {
  ensureTauriRuntime();
  return invoke<CaptureMemoryEventOutput>("capture_memory_event", { request });
}

export function createObservation(request: CreateObservationInput): Promise<CreateObservationOutput> {
  ensureTauriRuntime();
  return invoke<CreateObservationOutput>("create_observation", { request });
}

export function createMemoryCandidateFromObservation(
  request: CreateMemoryCandidateFromObservationInput,
): Promise<CreateMemoryCandidateFromObservationOutput> {
  ensureTauriRuntime();
  return invoke<CreateMemoryCandidateFromObservationOutput>("create_memory_candidate_from_observation", { request });
}

export function loadMemoryCandidateStore(): Promise<MemoryCandidateStoreV1> {
  ensureTauriRuntime();
  return invoke<MemoryCandidateStoreV1>("load_memory_candidate_store");
}

export function createMemoryCandidate(request: CreateMemoryCandidateInput): Promise<CreateMemoryCandidateOutput> {
  ensureTauriRuntime();
  return invoke<CreateMemoryCandidateOutput>("create_memory_candidate", { request });
}

export function recordMemoryCandidateDecision(
  request: RecordMemoryCandidateDecisionInput,
): Promise<RecordMemoryCandidateDecisionOutput> {
  ensureTauriRuntime();
  return invoke<RecordMemoryCandidateDecisionOutput>("record_memory_candidate_decision", { request });
}

export function adoptMemoryCandidateToFormalMemory(
  request: AdoptMemoryCandidateInput,
): Promise<AdoptMemoryCandidateOutput> {
  ensureTauriRuntime();
  return invoke<AdoptMemoryCandidateOutput>("adopt_memory_candidate_to_formal_memory", { request });
}

export function loadFormalMemoryStore(): Promise<FormalMemoryStoreV1> {
  ensureTauriRuntime();
  return invoke<FormalMemoryStoreV1>("load_formal_memory_store");
}

export function previewTaskMemoryPacket(request: TaskMemoryPacketBuildInput): Promise<TaskMemoryPacketBuildOutput> {
  ensureTauriRuntime();
  return invoke<TaskMemoryPacketBuildOutput>("preview_task_memory_packet", { request });
}

export function loadMemoryLintStore(): Promise<MemoryLintStoreV1> {
  ensureTauriRuntime();
  return invoke<MemoryLintStoreV1>("load_memory_lint_store");
}

export function loadMemoryEntityRelationStore(): Promise<MemoryEntityRelationStoreV1> {
  ensureTauriRuntime();
  return invoke<MemoryEntityRelationStoreV1>("load_memory_entity_relation_store");
}

export function loadMemoryPatternStore(): Promise<MemoryPatternStoreV1> {
  ensureTauriRuntime();
  return invoke<MemoryPatternStoreV1>("load_memory_pattern_store");
}

export function previewMaturePatterns(request: PreviewMaturePatternsInput): Promise<MaturePatternPreviewOutput> {
  ensureTauriRuntime();
  return invoke<MaturePatternPreviewOutput>("preview_mature_patterns", { request });
}

export function recordMaturePatternDecision(
  request: RecordMaturePatternDecisionInput,
): Promise<RecordMaturePatternDecisionOutput> {
  ensureTauriRuntime();
  return invoke<RecordMaturePatternDecisionOutput>("record_mature_pattern_decision", { request });
}

export function previewMemoryEntityRelationCandidates(
  request: PreviewMemoryEntityRelationCandidatesInput,
): Promise<MemoryEntityRelationPreviewOutput> {
  ensureTauriRuntime();
  return invoke<MemoryEntityRelationPreviewOutput>("preview_memory_entity_relation_candidates", { request });
}

export function recordMemoryEntityAliasDecision(
  request: RecordMemoryEntityAliasDecisionInput,
): Promise<RecordMemoryEntityAliasDecisionOutput> {
  ensureTauriRuntime();
  return invoke<RecordMemoryEntityAliasDecisionOutput>("record_memory_entity_alias_decision", { request });
}

export function recordMemoryEntityMergeDecision(
  request: RecordMemoryEntityMergeDecisionInput,
): Promise<RecordMemoryEntityMergeDecisionOutput> {
  ensureTauriRuntime();
  return invoke<RecordMemoryEntityMergeDecisionOutput>("record_memory_entity_merge_decision", { request });
}

export function recordMemoryRelationCandidateDecision(
  request: RecordMemoryRelationCandidateDecisionInput,
): Promise<RecordMemoryRelationCandidateDecisionOutput> {
  ensureTauriRuntime();
  return invoke<RecordMemoryRelationCandidateDecisionOutput>("record_memory_relation_candidate_decision", { request });
}

export function runMemoryLint(request: MemoryLintRunInput): Promise<MemoryLintRunOutput> {
  ensureTauriRuntime();
  return invoke<MemoryLintRunOutput>("run_memory_lint", { request });
}

export function createFormalMemoryRecord(
  request: CreateFormalMemoryRecordInput,
): Promise<CreateFormalMemoryRecordOutput> {
  ensureTauriRuntime();
  return invoke<CreateFormalMemoryRecordOutput>("create_formal_memory_record", { request });
}

export function previewFormalMemoryLifecycleOperation(
  request: FormalMemoryLifecyclePreviewInput,
): Promise<FormalMemoryLifecyclePreview> {
  ensureTauriRuntime();
  return invoke<FormalMemoryLifecyclePreview>("preview_formal_memory_lifecycle_operation", { request });
}

export function recordFormalMemoryLifecycleOperation(
  request: FormalMemoryLifecycleInput,
): Promise<FormalMemoryLifecycleOutput> {
  ensureTauriRuntime();
  return invoke<FormalMemoryLifecycleOutput>("record_formal_memory_lifecycle_operation", { request });
}

export function initializeWorkflowState(): Promise<WorkflowStateMutationResult> {
  ensureTauriRuntime();
  return invoke<WorkflowStateMutationResult>("initialize_workflow_state");
}

export function bootstrapProjectWorkflow(projectRoot: string): Promise<WorkflowStateMutationResult> {
  ensureTauriRuntime();
  return invoke<WorkflowStateMutationResult>("bootstrap_project_workflow", { request: { path: projectRoot } });
}

export function createTaskDraft(request: TaskDraftRequest): Promise<WorkflowStateMutationResult> {
  ensureTauriRuntime();
  return invoke<WorkflowStateMutationResult>("create_task_draft", { request });
}

export function renderTaskPackagePreview(request: TaskPackagePreviewRequest): Promise<TaskPackagePreview> {
  ensureTauriRuntime();
  return invoke<TaskPackagePreview>("render_task_package_preview", { request });
}

export function copyTaskPackagePreview(request: TaskPackagePreviewRequest): Promise<string> {
  ensureTauriRuntime();
  return invoke<string>("copy_task_package_preview", { request });
}

export function updateTaskPackageDraftFields(request: TaskPackageFieldsUpdateRequest): Promise<WorkflowStateMutationResult> {
  ensureTauriRuntime();
  return invoke<WorkflowStateMutationResult>("update_task_package_draft_fields", { request });
}

export function correctTaskPackageDispatchFields(
  request: TaskPackageDispatchFieldsCorrectionRequest,
): Promise<WorkflowStateMutationResult> {
  ensureTauriRuntime();
  return invoke<WorkflowStateMutationResult>("correct_task_package_dispatch_fields", { request });
}

export function generateTaskPackageFile(request: TaskPackageFileGenerationRequest): Promise<TaskPackageFileGenerationResult> {
  ensureTauriRuntime();
  return invoke<TaskPackageFileGenerationResult>("generate_task_package_file", { request });
}

export function inspectTaskPackageDispatchReadiness(
  request: TaskPackageDispatchReadinessRequest,
): Promise<TaskPackageDispatchReadiness> {
  ensureTauriRuntime();
  return invoke<TaskPackageDispatchReadiness>("inspect_task_package_dispatch_readiness", { request });
}

export function inspectWorkflowRunCheck(request: WorkflowRunCheckRequest): Promise<WorkflowRunCheck> {
  ensureTauriRuntime();
  return invoke<WorkflowRunCheck>("inspect_workflow_run_check", { request });
}

export function updateWorkItemState(request: WorkItemStateUpdateRequest): Promise<WorkflowStateMutationResult> {
  ensureTauriRuntime();
  return invoke<WorkflowStateMutationResult>("update_work_item_state", { request });
}

export function bindWorkflowNodeCodexSession(request: WorkflowNodeSessionBindRequest): Promise<WorkflowStateMutationResult> {
  ensureTauriRuntime();
  return invoke<WorkflowStateMutationResult>("bind_workflow_node_codex_session", { request });
}

export function unbindWorkflowNodeCodexSession(request: WorkflowNodeSessionUnbindRequest): Promise<WorkflowStateMutationResult> {
  ensureTauriRuntime();
  return invoke<WorkflowStateMutationResult>("unbind_workflow_node_codex_session", { request });
}

// P1-E 死码清扫（2026-07-18）：prepareWorkflowNodeDispatch 全仓零调用者（P1-D 勘察已实锤·本轮复核仍零）——
// 随其删除的后端命令包装层见 commands.rs（内层 prepare_workflow_node_dispatch_for_index_at 仍被测试直调，未删）。

export function prepareOfflineRoleDispatch(request: OfflineRoleDispatchRequest): Promise<WorkflowNodeDispatchResult> {
  ensureTauriRuntime();
  return invoke<WorkflowNodeDispatchResult>("prepare_offline_role_dispatch", { request });
}

export function recordOfflineRoleResultHandoff(request: OfflineRoleResultHandoffRequest): Promise<WorkflowNodeDispatchResult> {
  ensureTauriRuntime();
  return invoke<WorkflowNodeDispatchResult>("record_offline_role_result_handoff", { request });
}

export function recordOfflineDirectorReview(request: OfflineDirectorReviewRequest): Promise<WorkflowStateMutationResult> {
  ensureTauriRuntime();
  return invoke<WorkflowStateMutationResult>("record_offline_director_review", { request });
}

export function recordWorkflowDispatchDirectorReview(
  request: WorkflowDispatchDirectorReviewRequest,
): Promise<WorkflowStateMutationResult> {
  ensureTauriRuntime();
  return invoke<WorkflowStateMutationResult>("record_workflow_dispatch_director_review", { request });
}

export function recordWorkflowPermissionDecision(
  request: WorkflowPermissionDecisionRequest,
): Promise<WorkflowStateMutationResult> {
  ensureTauriRuntime();
  return invoke<WorkflowStateMutationResult>("record_workflow_permission_decision", { request });
}

export function recordOperationControlDecision(
  request: OperationControlDecisionRequest,
): Promise<WorkflowStateMutationResult> {
  ensureTauriRuntime();
  return invoke<WorkflowStateMutationResult>("record_operation_control_decision", { request });
}

// L3 知识库第一片：工作台自管 vault 五命令（AI 写入仅经 PermissionDialog 用户允许那一下调用）。
export type KnowledgeVaultNoteSummary = {
  slug: string;
  title: string;
  mtime_ms: number;
  outlinks: string[];
};
export type KnowledgeVaultNote = {
  slug: string;
  title: string;
  body: string;
  mtime_ms: number;
  content_hash: string;
};
export type KnowledgeVaultWriteResult = {
  slug: string;
  title: string;
  audit_event_id: string;
  created: boolean;
};

export function knowledgeVaultListNotes(): Promise<KnowledgeVaultNoteSummary[]> {
  ensureTauriRuntime();
  return invoke<KnowledgeVaultNoteSummary[]>("knowledge_vault_list_notes");
}

export function knowledgeVaultReadNote(slug: string): Promise<KnowledgeVaultNote> {
  ensureTauriRuntime();
  return invoke<KnowledgeVaultNote>("knowledge_vault_read_note", { slug });
}

export function knowledgeVaultCreateNote(title: string): Promise<KnowledgeVaultWriteResult> {
  ensureTauriRuntime();
  return invoke<KnowledgeVaultWriteResult>("knowledge_vault_create_note", { title });
}

export function knowledgeVaultWriteNote(
  slug: string,
  body: string,
  expectedMtimeMs: number,
  expectedContentHash: string,
): Promise<KnowledgeVaultWriteResult> {
  ensureTauriRuntime();
  return invoke<KnowledgeVaultWriteResult>("knowledge_vault_write_note", {
    slug,
    body,
    expectedMtimeMs,
    expectedContentHash,
  });
}

export function knowledgeVaultAiWrite(request: {
  note_title: string;
  body: string;
  source_summary: string;
}): Promise<KnowledgeVaultWriteResult> {
  ensureTauriRuntime();
  return invoke<KnowledgeVaultWriteResult>("knowledge_vault_ai_write", {
    noteTitle: request.note_title,
    body: request.body,
    sourceSummary: request.source_summary,
  });
}

// R2 host-owned relay acknowledgement: this is deliberately a single fixed
// command and exact intent payload.  It does not expose a route, command,
// vault root, or a generic invoke surface to the knowledge UI.
export const KNOWLEDGE_OPEN_RELAY_TAURI_COMMANDS = {
  acknowledge: "acknowledge_knowledge_open_relay_intent",
} as const;

export function acknowledgeKnowledgeOpenRelayIntent(
  intent: KnowledgeOpenRelayIntent,
  outcome: KnowledgeOpenRelayOutcome,
): Promise<void> {
  ensureTauriRuntime();
  return invoke<void>(KNOWLEDGE_OPEN_RELAY_TAURI_COMMANDS.acknowledge, {
    request: knowledgeOpenRelayAckRequest(intent, outcome),
  });
}

// N1-N5 Syn 原生知识工作区：前端只能调用这组固定 host command。固定 vault 根、
// 路径解析、CAS、审计和文件系统操作全部留在 Rust；这里不暴露泛型 command/root/shell
// 入口。测试可以注入 recorder，但生产调用仍只经过同一份固定命令表。
export const KNOWLEDGE_WORKSPACE_TAURI_COMMANDS = {
  snapshot: "knowledge_workspace_snapshot",
  vault_manifest: "knowledge_workspace_vault_manifest",
  search: "knowledge_workspace_search",
  graph: "knowledge_workspace_graph",
  read_markdown: "knowledge_workspace_read_markdown",
  read_canvas: "knowledge_workspace_read_canvas",
  create_directory: "knowledge_workspace_create_directory",
  create_markdown: "knowledge_workspace_create_markdown",
  write_markdown: "knowledge_workspace_write_markdown",
  create_canvas: "knowledge_workspace_create_canvas",
  write_canvas: "knowledge_workspace_write_canvas",
  import_attachment: "knowledge_workspace_import_attachment",
  read_attachment: "knowledge_workspace_read_attachment",
  create_recovery_backup: "knowledge_workspace_create_recovery_backup",
  list_recovery_backups: "knowledge_workspace_list_recovery_backups",
  restore_recovery_backup: "knowledge_workspace_restore_recovery_backup",
  move_entry: "knowledge_workspace_move_entry",
  rename_entry: "knowledge_workspace_rename_entry",
  delete_entry: "knowledge_workspace_delete_entry",
} as const;

export type KnowledgeWorkspaceInvokeName =
  (typeof KNOWLEDGE_WORKSPACE_TAURI_COMMANDS)[keyof typeof KNOWLEDGE_WORKSPACE_TAURI_COMMANDS];

export type KnowledgeWorkspaceEntryKind = "directory" | "markdown" | "canvas" | "attachment";

export type KnowledgeWorkspaceEntry = Readonly<{
  relative_path: string;
  parent_path: string | null;
  kind: KnowledgeWorkspaceEntryKind;
  title: string | null;
  tags: ReadonlyArray<string>;
  aliases: ReadonlyArray<string>;
  properties: Readonly<Record<string, string>>;
  mtime_ms: number;
  size_bytes: number;
  outlinks: ReadonlyArray<string>;
  backlinks: ReadonlyArray<string>;
}>;

export type KnowledgeWorkspaceTag = Readonly<{
  tag: string;
  note_count: number;
}>;

export type KnowledgeWorkspaceDiagnostic = Readonly<{
  code: string;
  relative_path: string | null;
  message: string;
}>;

export type KnowledgeWorkspaceSnapshot = Readonly<{
  entries: ReadonlyArray<KnowledgeWorkspaceEntry>;
  tags: ReadonlyArray<KnowledgeWorkspaceTag>;
  diagnostics: ReadonlyArray<KnowledgeWorkspaceDiagnostic>;
}>;

// N5 manifest 仍是从固定 vault 即时重建的只读投影；它不携带正文、附件 bytes、根路径
// 或任意文件句柄。具体内容只由下方的固定 read/backup 命令读取。
export type KnowledgeWorkspaceVaultManifestEntry = Readonly<{
  relative_path: string;
  kind: KnowledgeWorkspaceEntryKind;
  mtime_ms: number;
  size_bytes: number;
}>;

export type KnowledgeWorkspaceVaultManifest = Readonly<{
  entries: ReadonlyArray<KnowledgeWorkspaceVaultManifestEntry>;
  diagnostics: ReadonlyArray<KnowledgeWorkspaceDiagnostic>;
}>;

export type KnowledgeWorkspaceSearchResult = Readonly<{
  relative_path: string;
  title: string;
  snippet: string;
  tags: ReadonlyArray<string>;
  mtime_ms: number;
}>;

export type KnowledgeWorkspaceSearchResponse = Readonly<{
  query: string;
  results: ReadonlyArray<KnowledgeWorkspaceSearchResult>;
  diagnostics: ReadonlyArray<KnowledgeWorkspaceDiagnostic>;
}>;

// N3 图谱只描述可重建的 Markdown 关系投影。`id` 与 `relative_path` 都是同一个已
// 验证的 vault 相对路径，可直接作为 React Flow 节点 id；绝不接受 URI、root 或布局状态。
export type KnowledgeWorkspaceGraphScope = "global" | "local";

export type KnowledgeWorkspaceGraphOptions = Readonly<{
  scope: KnowledgeWorkspaceGraphScope;
  focusRelativePath?: string;
  query?: string;
  tag?: string;
}>;

export type KnowledgeWorkspaceGraphNode = Readonly<{
  id: string;
  relative_path: string;
  title: string;
  tags: ReadonlyArray<string>;
}>;

export type KnowledgeWorkspaceGraphEdge = Readonly<{
  id: string;
  source: string;
  target: string;
}>;

export type KnowledgeWorkspaceGraphResponse = Readonly<{
  scope: KnowledgeWorkspaceGraphScope;
  focus_relative_path: string | null;
  query: string | null;
  tag: string | null;
  nodes: ReadonlyArray<KnowledgeWorkspaceGraphNode>;
  edges: ReadonlyArray<KnowledgeWorkspaceGraphEdge>;
  diagnostics: ReadonlyArray<KnowledgeWorkspaceDiagnostic>;
  truncated: boolean;
}>;

export type KnowledgeWorkspaceMarkdownDocument = Readonly<{
  relative_path: string;
  title: string;
  body: string;
  tags: ReadonlyArray<string>;
  aliases: ReadonlyArray<string>;
  properties: Readonly<Record<string, string>>;
  outlinks: ReadonlyArray<string>;
  backlinks: ReadonlyArray<string>;
  mtime_ms: number;
  content_hash: string;
}>;

// N4 JSON Canvas 使用自己的 JSON 值合同，不复用工作流 CanvasDefinition。这个值仅能
// 经过固定 `read/create/write_canvas` 命令进入 Syn 自管 vault，不能携带文件根或动作。
export type JsonCanvasPrimitive = string | number | boolean | null;
export type JsonCanvasValue = JsonCanvasPrimitive | ReadonlyArray<JsonCanvasValue> | JsonCanvasObject;
export interface JsonCanvasObject {
  readonly [key: string]: JsonCanvasValue;
}

export type KnowledgeWorkspaceCanvasDiagnostic = Readonly<{
  code: string;
  node_id: string | null;
  reference: string | null;
  message: string;
}>;

export type KnowledgeWorkspaceCanvasDocument = Readonly<{
  relative_path: string;
  document: JsonCanvasObject;
  mtime_ms: number;
  content_hash: string;
  diagnostics: ReadonlyArray<KnowledgeWorkspaceCanvasDiagnostic>;
}>;

export type KnowledgeWorkspaceAttachmentMimeType =
  | "image/png"
  | "image/jpeg"
  | "image/gif"
  | "image/webp"
  | "application/pdf"
  | "text/plain"
  | "text/csv";

// Attachment bytes only cross the typed host boundary as browser File bytes. No source path,
// URL, vault root or arbitrary filesystem handle is accepted or returned.
export type KnowledgeWorkspaceAttachment = Readonly<{
  relative_path: string;
  mime_type: KnowledgeWorkspaceAttachmentMimeType;
  bytes: ReadonlyArray<number>;
  mtime_ms: number;
  content_hash: string;
  size_bytes: number;
}>;

export type KnowledgeWorkspaceAttachmentImportResult = Readonly<{
  relative_path: string;
  mime_type: KnowledgeWorkspaceAttachmentMimeType;
  mtime_ms: number;
  content_hash: string;
  size_bytes: number;
  audit_event_id: string;
}>;

export type KnowledgeWorkspaceRecoveryKind = "markdown" | "canvas" | "attachment";

export type KnowledgeWorkspaceRecoveryBackup = Readonly<{
  backup_id: string;
  relative_path: string;
  kind: KnowledgeWorkspaceRecoveryKind;
  size_bytes: number;
  content_hash: string;
  created_at_ms: number;
  audit_event_id: string;
}>;

export type KnowledgeWorkspaceRecoveryBackupSummary = Readonly<{
  backup_id: string;
  relative_path: string;
  kind: KnowledgeWorkspaceRecoveryKind;
  size_bytes: number;
  content_hash: string;
  created_at_ms: number;
}>;

export type KnowledgeWorkspaceRecoveryRestoreResult = Readonly<{
  backup_id: string;
  relative_path: string;
  mtime_ms: number;
  content_hash: string;
  audit_event_id: string;
}>;

export type KnowledgeWorkspaceMutationOperation =
  | "directory_created"
  | "markdown_created"
  | "markdown_updated"
  | "canvas_created"
  | "canvas_updated"
  | "markdown_moved"
  | "markdown_renamed"
  | "markdown_deleted";

export type KnowledgeWorkspaceMutationResult = Readonly<{
  operation: KnowledgeWorkspaceMutationOperation;
  relative_path: string;
  source_relative_path: string | null;
  mtime_ms: number | null;
  content_hash: string | null;
  audit_event_id: string;
}>;

type KnowledgeWorkspaceSearchArgs = Readonly<{ query: string }>;
type KnowledgeWorkspaceGraphArgs = Readonly<{
  scope: KnowledgeWorkspaceGraphScope;
  focusRelativePath?: string;
  query?: string;
  tag?: string;
}>;
type KnowledgeWorkspaceReadMarkdownArgs = Readonly<{ relativePath: string }>;
type KnowledgeWorkspaceReadCanvasArgs = Readonly<{ relativePath: string }>;
type KnowledgeWorkspaceImportAttachmentArgs = Readonly<{
  bytes: ReadonlyArray<number>;
  displayName: string;
  mimeType: KnowledgeWorkspaceAttachmentMimeType;
}>;
type KnowledgeWorkspaceReadAttachmentArgs = Readonly<{ relativePath: string }>;
type KnowledgeWorkspaceCreateDirectoryArgs = Readonly<{ relativePath: string }>;
type KnowledgeWorkspaceCreateMarkdownArgs = Readonly<{ relativePath: string; body: string }>;
type KnowledgeWorkspaceWriteMarkdownArgs = Readonly<{
  relativePath: string;
  body: string;
  expectedMtimeMs: number;
  expectedContentHash: string;
}>;
type KnowledgeWorkspaceCreateCanvasArgs = Readonly<{
  relativePath: string;
  document: JsonCanvasObject;
}>;
type KnowledgeWorkspaceWriteCanvasArgs = Readonly<{
  relativePath: string;
  document: JsonCanvasObject;
  expectedMtimeMs: number;
  expectedContentHash: string;
}>;
type KnowledgeWorkspaceMoveEntryArgs = Readonly<{
  from: string;
  to: string;
  expectedMtimeMs: number;
  expectedContentHash: string;
}>;
type KnowledgeWorkspaceRenameEntryArgs = KnowledgeWorkspaceMoveEntryArgs;
type KnowledgeWorkspaceDeleteEntryArgs = Readonly<{
  relativePath: string;
  expectedMtimeMs: number;
  expectedContentHash: string;
}>;
type KnowledgeWorkspaceCreateRecoveryBackupArgs = Readonly<{
  relativePath: string;
  expectedMtimeMs: number;
  expectedContentHash: string;
}>;
type KnowledgeWorkspaceRestoreRecoveryBackupArgs = Readonly<{
  backupId: string;
  expectedMtimeMs: number;
  expectedContentHash: string;
}>;

export type KnowledgeWorkspaceInvokeArgs =
  | KnowledgeWorkspaceSearchArgs
  | KnowledgeWorkspaceGraphArgs
  | KnowledgeWorkspaceReadMarkdownArgs
  | KnowledgeWorkspaceReadCanvasArgs
  | KnowledgeWorkspaceImportAttachmentArgs
  | KnowledgeWorkspaceReadAttachmentArgs
  | KnowledgeWorkspaceCreateDirectoryArgs
  | KnowledgeWorkspaceCreateMarkdownArgs
  | KnowledgeWorkspaceWriteMarkdownArgs
  | KnowledgeWorkspaceCreateCanvasArgs
  | KnowledgeWorkspaceWriteCanvasArgs
  | KnowledgeWorkspaceMoveEntryArgs
  | KnowledgeWorkspaceRenameEntryArgs
  | KnowledgeWorkspaceDeleteEntryArgs
  | KnowledgeWorkspaceCreateRecoveryBackupArgs
  | KnowledgeWorkspaceRestoreRecoveryBackupArgs;

export type KnowledgeWorkspaceInvoke = <T>(
  command: KnowledgeWorkspaceInvokeName,
  args?: KnowledgeWorkspaceInvokeArgs,
) => Promise<T>;

export type KnowledgeWorkspaceClient = Readonly<{
  snapshot: () => Promise<KnowledgeWorkspaceSnapshot>;
  vaultManifest: () => Promise<KnowledgeWorkspaceVaultManifest>;
  search: (query: string) => Promise<KnowledgeWorkspaceSearchResponse>;
  graph: (options: KnowledgeWorkspaceGraphOptions) => Promise<KnowledgeWorkspaceGraphResponse>;
  readMarkdown: (relativePath: string) => Promise<KnowledgeWorkspaceMarkdownDocument>;
  readCanvas: (relativePath: string) => Promise<KnowledgeWorkspaceCanvasDocument>;
  importAttachment: (
    bytes: Uint8Array,
    displayName: string,
    mimeType: KnowledgeWorkspaceAttachmentMimeType,
  ) => Promise<KnowledgeWorkspaceAttachmentImportResult>;
  readAttachment: (relativePath: string) => Promise<KnowledgeWorkspaceAttachment>;
  createRecoveryBackup: (
    relativePath: string,
    expectedMtimeMs: number,
    expectedContentHash: string,
  ) => Promise<KnowledgeWorkspaceRecoveryBackup>;
  listRecoveryBackups: () => Promise<ReadonlyArray<KnowledgeWorkspaceRecoveryBackupSummary>>;
  restoreRecoveryBackup: (
    backupId: string,
    expectedMtimeMs: number,
    expectedContentHash: string,
  ) => Promise<KnowledgeWorkspaceRecoveryRestoreResult>;
  createDirectory: (relativePath: string) => Promise<KnowledgeWorkspaceMutationResult>;
  createMarkdown: (relativePath: string, body: string) => Promise<KnowledgeWorkspaceMutationResult>;
  writeMarkdown: (
    relativePath: string,
    body: string,
    expectedMtimeMs: number,
    expectedContentHash: string,
  ) => Promise<KnowledgeWorkspaceMutationResult>;
  createCanvas: (
    relativePath: string,
    document: JsonCanvasObject,
  ) => Promise<KnowledgeWorkspaceMutationResult>;
  writeCanvas: (
    relativePath: string,
    document: JsonCanvasObject,
    expectedMtimeMs: number,
    expectedContentHash: string,
  ) => Promise<KnowledgeWorkspaceMutationResult>;
  moveEntry: (
    from: string,
    to: string,
    expectedMtimeMs: number,
    expectedContentHash: string,
  ) => Promise<KnowledgeWorkspaceMutationResult>;
  renameEntry: (
    from: string,
    to: string,
    expectedMtimeMs: number,
    expectedContentHash: string,
  ) => Promise<KnowledgeWorkspaceMutationResult>;
  deleteEntry: (
    relativePath: string,
    expectedMtimeMs: number,
    expectedContentHash: string,
  ) => Promise<KnowledgeWorkspaceMutationResult>;
}>;

const MAX_KNOWLEDGE_WORKSPACE_PATH_BYTES = 512;
const MAX_KNOWLEDGE_WORKSPACE_PATH_SEGMENTS = 32;
const MAX_KNOWLEDGE_WORKSPACE_SEGMENT_CHARS = 128;
const MAX_KNOWLEDGE_WORKSPACE_MARKDOWN_BYTES = 64 * 1024;
const MAX_KNOWLEDGE_WORKSPACE_CANVAS_BYTES = 256 * 1024;
const MAX_KNOWLEDGE_WORKSPACE_ATTACHMENT_BYTES = 10 * 1024 * 1024;
const MAX_KNOWLEDGE_WORKSPACE_SEARCH_BYTES = 256;
const MAX_KNOWLEDGE_WORKSPACE_GRAPH_TAG_CHARS = 256;
const SHA256_HEX_PATTERN = /^[a-f0-9]{64}$/;
const UNSAFE_WORKSPACE_SEGMENT_PATTERN = /[\\/:*?\[\]{}'"=|<>]/u;
const UNSAFE_WORKSPACE_GRAPH_TAG_PATTERN = /[\[\]{}|>]/u;
const CONTROL_CHARACTER_PATTERN = /\p{Cc}/u;
const KNOWLEDGE_WORKSPACE_ATTACHMENT_MIME_BY_EXTENSION = {
  png: "image/png",
  jpg: "image/jpeg",
  jpeg: "image/jpeg",
  gif: "image/gif",
  webp: "image/webp",
  pdf: "application/pdf",
  txt: "text/plain",
  csv: "text/csv",
} as const satisfies Readonly<Record<string, KnowledgeWorkspaceAttachmentMimeType>>;
const RECOVERY_BACKUP_ID_PATTERN = /^[a-f0-9]{32}$/;

export function createKnowledgeWorkspaceClient(
  invokeCommand: KnowledgeWorkspaceInvoke = invokeKnowledgeWorkspace,
): KnowledgeWorkspaceClient {
  return Object.freeze({
    snapshot: () => invokeCommand<KnowledgeWorkspaceSnapshot>(KNOWLEDGE_WORKSPACE_TAURI_COMMANDS.snapshot),
    vaultManifest: () =>
      invokeCommand<KnowledgeWorkspaceVaultManifest>(KNOWLEDGE_WORKSPACE_TAURI_COMMANDS.vault_manifest),
    search: async (query) =>
      invokeCommand<KnowledgeWorkspaceSearchResponse>(KNOWLEDGE_WORKSPACE_TAURI_COMMANDS.search, {
        query: safeWorkspaceSearchQuery(query),
      }),
    graph: async (options) =>
      invokeCommand<KnowledgeWorkspaceGraphResponse>(
        KNOWLEDGE_WORKSPACE_TAURI_COMMANDS.graph,
        safeWorkspaceGraphOptions(options),
      ),
    readMarkdown: async (relativePath) =>
      invokeCommand<KnowledgeWorkspaceMarkdownDocument>(KNOWLEDGE_WORKSPACE_TAURI_COMMANDS.read_markdown, {
        relativePath: safeWorkspaceMarkdownPath(relativePath),
      }),
    readCanvas: async (relativePath) =>
      invokeCommand<KnowledgeWorkspaceCanvasDocument>(KNOWLEDGE_WORKSPACE_TAURI_COMMANDS.read_canvas, {
        relativePath: safeWorkspaceCanvasPath(relativePath),
      }),
    importAttachment: async (bytes, displayName, mimeType) =>
      invokeCommand<KnowledgeWorkspaceAttachmentImportResult>(
        KNOWLEDGE_WORKSPACE_TAURI_COMMANDS.import_attachment,
        safeWorkspaceAttachmentImport(bytes, displayName, mimeType),
      ),
    readAttachment: async (relativePath) =>
      invokeCommand<KnowledgeWorkspaceAttachment>(KNOWLEDGE_WORKSPACE_TAURI_COMMANDS.read_attachment, {
        relativePath: safeWorkspaceAttachmentPath(relativePath),
      }),
    createRecoveryBackup: async (relativePath, expectedMtimeMs, expectedContentHash) =>
      invokeCommand<KnowledgeWorkspaceRecoveryBackup>(
        KNOWLEDGE_WORKSPACE_TAURI_COMMANDS.create_recovery_backup,
        {
          relativePath: safeWorkspaceRecoverablePath(relativePath),
          expectedMtimeMs: safeWorkspaceMtime(expectedMtimeMs),
          expectedContentHash: safeWorkspaceContentHash(expectedContentHash),
        },
      ),
    listRecoveryBackups: () =>
      invokeCommand<ReadonlyArray<KnowledgeWorkspaceRecoveryBackupSummary>>(
        KNOWLEDGE_WORKSPACE_TAURI_COMMANDS.list_recovery_backups,
      ),
    restoreRecoveryBackup: async (backupId, expectedMtimeMs, expectedContentHash) =>
      invokeCommand<KnowledgeWorkspaceRecoveryRestoreResult>(
        KNOWLEDGE_WORKSPACE_TAURI_COMMANDS.restore_recovery_backup,
        {
          backupId: safeWorkspaceRecoveryBackupId(backupId),
          expectedMtimeMs: safeWorkspaceMtime(expectedMtimeMs),
          expectedContentHash: safeWorkspaceContentHash(expectedContentHash),
        },
      ),
    createDirectory: async (relativePath) =>
      invokeCommand<KnowledgeWorkspaceMutationResult>(KNOWLEDGE_WORKSPACE_TAURI_COMMANDS.create_directory, {
        relativePath: safeWorkspaceRelativePath(relativePath),
      }),
    createMarkdown: async (relativePath, body) =>
      invokeCommand<KnowledgeWorkspaceMutationResult>(KNOWLEDGE_WORKSPACE_TAURI_COMMANDS.create_markdown, {
        relativePath: safeWorkspaceMarkdownPath(relativePath),
        body: safeWorkspaceMarkdownBody(body),
      }),
    writeMarkdown: async (relativePath, body, expectedMtimeMs, expectedContentHash) =>
      invokeCommand<KnowledgeWorkspaceMutationResult>(KNOWLEDGE_WORKSPACE_TAURI_COMMANDS.write_markdown, {
        relativePath: safeWorkspaceMarkdownPath(relativePath),
        body: safeWorkspaceMarkdownBody(body),
        expectedMtimeMs: safeWorkspaceMtime(expectedMtimeMs),
        expectedContentHash: safeWorkspaceContentHash(expectedContentHash),
      }),
    createCanvas: async (relativePath, document) =>
      invokeCommand<KnowledgeWorkspaceMutationResult>(KNOWLEDGE_WORKSPACE_TAURI_COMMANDS.create_canvas, {
        relativePath: safeWorkspaceCanvasPath(relativePath),
        document: safeWorkspaceCanvasDocument(document),
      }),
    writeCanvas: async (relativePath, document, expectedMtimeMs, expectedContentHash) =>
      invokeCommand<KnowledgeWorkspaceMutationResult>(KNOWLEDGE_WORKSPACE_TAURI_COMMANDS.write_canvas, {
        relativePath: safeWorkspaceCanvasPath(relativePath),
        document: safeWorkspaceCanvasDocument(document),
        expectedMtimeMs: safeWorkspaceMtime(expectedMtimeMs),
        expectedContentHash: safeWorkspaceContentHash(expectedContentHash),
      }),
    moveEntry: async (from, to, expectedMtimeMs, expectedContentHash) =>
      invokeCommand<KnowledgeWorkspaceMutationResult>(KNOWLEDGE_WORKSPACE_TAURI_COMMANDS.move_entry, {
        from: safeWorkspaceMarkdownPath(from),
        to: safeWorkspaceMarkdownPath(to),
        expectedMtimeMs: safeWorkspaceMtime(expectedMtimeMs),
        expectedContentHash: safeWorkspaceContentHash(expectedContentHash),
      }),
    renameEntry: async (from, to, expectedMtimeMs, expectedContentHash) =>
      invokeCommand<KnowledgeWorkspaceMutationResult>(KNOWLEDGE_WORKSPACE_TAURI_COMMANDS.rename_entry, {
        from: safeWorkspaceMarkdownPath(from),
        to: safeWorkspaceMarkdownPath(to),
        expectedMtimeMs: safeWorkspaceMtime(expectedMtimeMs),
        expectedContentHash: safeWorkspaceContentHash(expectedContentHash),
      }),
    deleteEntry: async (relativePath, expectedMtimeMs, expectedContentHash) =>
      invokeCommand<KnowledgeWorkspaceMutationResult>(KNOWLEDGE_WORKSPACE_TAURI_COMMANDS.delete_entry, {
        relativePath: safeWorkspaceMarkdownPath(relativePath),
        expectedMtimeMs: safeWorkspaceMtime(expectedMtimeMs),
        expectedContentHash: safeWorkspaceContentHash(expectedContentHash),
      }),
  });
}

// Production singleton. The injectable factory above is the only testing seam;
// it cannot accept a different command name, vault root, arbitrary filesystem target or shell action.
export const knowledgeWorkspace = createKnowledgeWorkspaceClient();

function invokeKnowledgeWorkspace<T>(
  command: KnowledgeWorkspaceInvokeName,
  args?: KnowledgeWorkspaceInvokeArgs,
): Promise<T> {
  ensureTauriRuntime();
  return args === undefined ? invoke<T>(command) : invoke<T>(command, args);
}

function safeWorkspaceRelativePath(relativePath: string): string {
  if (
    typeof relativePath !== "string"
    || relativePath.length === 0
    || utf8ByteLength(relativePath) > MAX_KNOWLEDGE_WORKSPACE_PATH_BYTES
    || relativePath.includes("\\")
    || relativePath.startsWith("/")
  ) {
    throw new Error("knowledge_workspace_invalid_path");
  }
  const segments = relativePath.split("/");
  if (segments.length === 0 || segments.length > MAX_KNOWLEDGE_WORKSPACE_PATH_SEGMENTS) {
    throw new Error("knowledge_workspace_invalid_path");
  }
  for (const segment of segments) {
    if (
      segment.length === 0
      || segment === "."
      || segment === ".."
      || segment.startsWith(".")
      || segment.startsWith("-")
      || segment.includes("--")
      || Array.from(segment).length > MAX_KNOWLEDGE_WORKSPACE_SEGMENT_CHARS
      || CONTROL_CHARACTER_PATTERN.test(segment)
      || UNSAFE_WORKSPACE_SEGMENT_PATTERN.test(segment)
    ) {
      throw new Error("knowledge_workspace_invalid_path");
    }
  }
  return relativePath;
}

function safeWorkspaceMarkdownPath(relativePath: string): string {
  const safePath = safeWorkspaceRelativePath(relativePath);
  const fileName = safePath.split("/").at(-1) ?? "";
  if (!fileName.endsWith(".md") || fileName.length <= ".md".length) {
    throw new Error("knowledge_workspace_markdown_only");
  }
  return safePath;
}

function safeWorkspaceCanvasPath(relativePath: string): string {
  const safePath = safeWorkspaceRelativePath(relativePath);
  const fileName = safePath.split("/").at(-1) ?? "";
  if (!fileName.endsWith(".canvas") || fileName.length <= ".canvas".length) {
    throw new Error("knowledge_workspace_canvas_only");
  }
  return safePath;
}

function safeWorkspaceAttachmentImport(
  bytes: Uint8Array,
  displayName: string,
  mimeType: KnowledgeWorkspaceAttachmentMimeType,
): KnowledgeWorkspaceImportAttachmentArgs {
  if (!(bytes instanceof Uint8Array) || bytes.byteLength > MAX_KNOWLEDGE_WORKSPACE_ATTACHMENT_BYTES) {
    throw new Error("knowledge_workspace_attachment_too_large");
  }
  const safeDisplayName = safeWorkspaceAttachmentDisplayName(displayName);
  const expectedMimeType = attachmentMimeTypeForPath(`attachments/${safeDisplayName}`);
  if (mimeType !== expectedMimeType) {
    throw new Error("knowledge_workspace_attachment_invalid_mime_type");
  }
  return {
    bytes: Array.from(bytes),
    displayName: safeDisplayName,
    mimeType,
  };
}

function safeWorkspaceAttachmentPath(relativePath: string): string {
  const safePath = safeWorkspaceRelativePath(relativePath);
  if (!safePath.startsWith("attachments/")) {
    throw new Error("knowledge_workspace_attachment_only");
  }
  attachmentMimeTypeForPath(safePath);
  return safePath;
}

function safeWorkspaceAttachmentDisplayName(displayName: string): string {
  if (typeof displayName !== "string" || displayName.includes("/") || displayName.includes("\\")) {
    throw new Error("knowledge_workspace_attachment_invalid_display_name");
  }
  const safePath = safeWorkspaceRelativePath(`attachments/${displayName}`);
  if (safePath !== `attachments/${displayName}` || safePath.split("/").length !== 2) {
    throw new Error("knowledge_workspace_attachment_invalid_display_name");
  }
  attachmentMimeTypeForPath(safePath);
  return displayName;
}

function attachmentMimeTypeForPath(relativePath: string): KnowledgeWorkspaceAttachmentMimeType {
  const fileName = relativePath.split("/").at(-1) ?? "";
  const separator = fileName.lastIndexOf(".");
  const extension = separator > 0 ? fileName.slice(separator + 1) : "";
  const mimeType = KNOWLEDGE_WORKSPACE_ATTACHMENT_MIME_BY_EXTENSION[
    extension as keyof typeof KNOWLEDGE_WORKSPACE_ATTACHMENT_MIME_BY_EXTENSION
  ];
  if (mimeType === undefined) {
    throw new Error("knowledge_workspace_attachment_type_not_allowed");
  }
  return mimeType;
}

function safeWorkspaceRecoverablePath(relativePath: string): string {
  const safePath = safeWorkspaceRelativePath(relativePath);
  if (safePath.startsWith("attachments/")) {
    return safeWorkspaceAttachmentPath(safePath);
  }
  if (safePath.endsWith(".md")) {
    return safeWorkspaceMarkdownPath(safePath);
  }
  if (safePath.endsWith(".canvas")) {
    return safeWorkspaceCanvasPath(safePath);
  }
  throw new Error("knowledge_workspace_recovery_unsupported_entry");
}

function safeWorkspaceRecoveryBackupId(backupId: string): string {
  if (typeof backupId !== "string" || !RECOVERY_BACKUP_ID_PATTERN.test(backupId)) {
    throw new Error("knowledge_workspace_backup_invalid_id");
  }
  return backupId;
}

function safeWorkspaceCanvasDocument(document: JsonCanvasObject): JsonCanvasObject {
  if (!isJsonCanvasObject(document) || !isJsonCanvasValue(document, new Set<object>())) {
    throw new Error("knowledge_workspace_canvas_invalid_json");
  }
  let serialized: string;
  try {
    serialized = JSON.stringify(document);
  } catch {
    throw new Error("knowledge_workspace_canvas_invalid_json");
  }
  if (utf8ByteLength(serialized) > MAX_KNOWLEDGE_WORKSPACE_CANVAS_BYTES) {
    throw new Error("knowledge_workspace_canvas_too_large");
  }
  try {
    const parsed: unknown = JSON.parse(serialized);
    if (!isJsonCanvasObject(parsed)) {
      throw new Error("knowledge_workspace_canvas_invalid_json");
    }
    return parsed;
  } catch {
    throw new Error("knowledge_workspace_canvas_invalid_json");
  }
}

function isJsonCanvasObject(value: unknown): value is JsonCanvasObject {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    return false;
  }
  const prototype = Object.getPrototypeOf(value);
  return prototype === Object.prototype || prototype === null;
}

function isJsonCanvasValue(value: unknown, ancestors: Set<object>): value is JsonCanvasValue {
  if (value === null || typeof value === "string" || typeof value === "boolean") {
    return true;
  }
  if (typeof value === "number") {
    return Number.isFinite(value);
  }
  if (typeof value !== "object" || value === null || ancestors.has(value)) {
    return false;
  }
  if (!Array.isArray(value) && !isJsonCanvasObject(value)) {
    return false;
  }
  ancestors.add(value);
  const children = Array.isArray(value) ? value : Object.values(value);
  const valid = children.every((child) => isJsonCanvasValue(child, ancestors));
  ancestors.delete(value);
  return valid;
}

function safeWorkspaceSearchQuery(query: string): string {
  if (typeof query !== "string") {
    throw new Error("knowledge_workspace_invalid_search_query");
  }
  const normalized = query.trim();
  if (
    normalized.length === 0
    || utf8ByteLength(normalized) > MAX_KNOWLEDGE_WORKSPACE_SEARCH_BYTES
    || CONTROL_CHARACTER_PATTERN.test(normalized)
  ) {
    throw new Error("knowledge_workspace_invalid_search_query");
  }
  return normalized;
}

function safeWorkspaceGraphOptions(
  options: KnowledgeWorkspaceGraphOptions,
): KnowledgeWorkspaceGraphArgs {
  if (typeof options !== "object" || options === null || Array.isArray(options)) {
    throw new Error("knowledge_workspace_invalid_graph_request");
  }
  const rawOptions = options as Readonly<Record<string, unknown>>;
  for (const key of Object.keys(rawOptions)) {
    if (key !== "scope" && key !== "focusRelativePath" && key !== "query" && key !== "tag") {
      throw new Error("knowledge_workspace_invalid_graph_request");
    }
  }
  if (rawOptions.scope !== "global" && rawOptions.scope !== "local") {
    throw new Error("knowledge_workspace_invalid_graph_scope");
  }
  const scope = rawOptions.scope;
  const focusRelativePath = rawOptions.focusRelativePath;
  if (scope === "global" && focusRelativePath !== undefined) {
    throw new Error("knowledge_workspace_invalid_graph_focus");
  }
  let safeFocusRelativePath: string | undefined;
  if (scope === "local") {
    if (typeof focusRelativePath !== "string") {
      throw new Error("knowledge_workspace_invalid_graph_focus");
    }
    safeFocusRelativePath = safeWorkspaceMarkdownPath(focusRelativePath);
  }
  const query = rawOptions.query;
  if (query !== undefined && typeof query !== "string") {
    throw new Error("knowledge_workspace_invalid_search_query");
  }
  const tag = rawOptions.tag;
  if (tag !== undefined && typeof tag !== "string") {
    throw new Error("knowledge_workspace_invalid_graph_tag");
  }
  return {
    scope,
    ...(safeFocusRelativePath === undefined ? {} : { focusRelativePath: safeFocusRelativePath }),
    ...(query === undefined ? {} : { query: safeWorkspaceSearchQuery(query) }),
    ...(tag === undefined ? {} : { tag: safeWorkspaceGraphTag(tag) }),
  };
}

function safeWorkspaceGraphTag(tag: string): string {
  const normalized = tag.trim();
  if (
    normalized.length === 0
    || utf8ByteLength(normalized) > MAX_KNOWLEDGE_WORKSPACE_SEARCH_BYTES
    || Array.from(normalized).length > MAX_KNOWLEDGE_WORKSPACE_GRAPH_TAG_CHARS
    || CONTROL_CHARACTER_PATTERN.test(normalized)
    || UNSAFE_WORKSPACE_GRAPH_TAG_PATTERN.test(normalized)
  ) {
    throw new Error("knowledge_workspace_invalid_graph_tag");
  }
  return normalized;
}

function safeWorkspaceMarkdownBody(body: string): string {
  if (typeof body !== "string" || utf8ByteLength(body) > MAX_KNOWLEDGE_WORKSPACE_MARKDOWN_BYTES) {
    throw new Error("knowledge_workspace_markdown_too_large");
  }
  return body;
}

function safeWorkspaceMtime(expectedMtimeMs: number): number {
  if (!Number.isSafeInteger(expectedMtimeMs) || expectedMtimeMs < 0) {
    throw new Error("knowledge_workspace_invalid_mtime");
  }
  return expectedMtimeMs;
}

function safeWorkspaceContentHash(expectedContentHash: string): string {
  if (typeof expectedContentHash !== "string" || !SHA256_HEX_PATTERN.test(expectedContentHash)) {
    throw new Error("knowledge_workspace_invalid_content_hash");
  }
  return expectedContentHash;
}

function utf8ByteLength(value: string): number {
  return new TextEncoder().encode(value).byteLength;
}

export function previewManualCodexRelay(
  request: ManualRelayPreviewInput,
): Promise<ManualRelayPreview> {
  ensureTauriRuntime();
  return invoke<ManualRelayPreview>("preview_manual_codex_relay", { request });
}

export function confirmManualCodexRelayOnce(
  request: ManualRelayConfirmInput,
): Promise<ManualRelayConfirmation> {
  ensureTauriRuntime();
  return invoke<ManualRelayConfirmation>("confirm_manual_codex_relay_once", { request });
}

export function runManualCodexRelayOnce(
  request: ManualRelayRunInput,
): Promise<ManualRelayReceipt> {
  ensureTauriRuntime();
  return invoke<ManualRelayReceipt>("run_manual_codex_relay_once", { request });
}

// M3 RoleSession read model ---------------------------------------------------
//
// These wrappers intentionally have separate fixed-host command names.  They
// accept only a canonical project locator hint plus opaque selectors/cursors;
// no renderer path can send actor/role/scope/object/channel/permission,
// owner, provider, profile, or legacy thread truth.
export async function loadAgentRoleSessionDirectory(
  request: RoleSessionDirectoryRequest,
): Promise<RoleSessionDirectory> {
  const safeRequest = createRoleSessionDirectoryRequest(request);
  ensureTauriRuntime();
  const directory = parseRoleSessionDirectory(
    await invoke<unknown>("load_agent_role_session_directory", { request: safeRequest }),
  );
  if (!roleSessionDirectoryMatchesRequest(directory, safeRequest)) {
    throw new Error("m3_read_model_stale_directory_response");
  }
  return directory;
}

export async function loadAgentRoleSessionDetail(
  request: RoleSessionDetailRequest,
): Promise<RoleSessionDetail> {
  const safeRequest = createRoleSessionDetailRequest(request);
  ensureTauriRuntime();
  const detail = parseRoleSessionDetail(
    await invoke<unknown>("load_agent_role_session_detail", { request: safeRequest }),
  );
  if (!roleSessionDetailMatchesRequest(detail, safeRequest)) {
    throw new Error("m3_read_model_stale_detail_response");
  }
  return detail;
}

export async function loadJiaobanRoleSessionDirectory(
  request: RoleSessionDirectoryRequest,
): Promise<RoleSessionDirectory> {
  const safeRequest = createRoleSessionDirectoryRequest(request);
  ensureTauriRuntime();
  const directory = parseRoleSessionDirectory(
    await invoke<unknown>("load_jiaoban_role_session_directory", { request: safeRequest }),
  );
  if (!roleSessionDirectoryMatchesRequest(directory, safeRequest)) {
    throw new Error("m3_read_model_stale_directory_response");
  }
  return directory;
}

export async function loadJiaobanRoleSessionDetail(
  request: RoleSessionDetailRequest,
): Promise<RoleSessionDetail> {
  const safeRequest = createRoleSessionDetailRequest(request);
  ensureTauriRuntime();
  const detail = parseRoleSessionDetail(
    await invoke<unknown>("load_jiaoban_role_session_detail", { request: safeRequest }),
  );
  if (!roleSessionDetailMatchesRequest(detail, safeRequest)) {
    throw new Error("m3_read_model_stale_detail_response");
  }
  return detail;
}

export function startAgentRoleSessionContinuation(
  request: RoleSessionContinuationStartRequest,
): Promise<void> {
  const safeRequest = createRoleSessionContinuationStartRequest(request);
  ensureTauriRuntime();
  return invoke<void>("start_agent_role_session_continuation", { request: safeRequest });
}

export function startJiaobanRoleSessionContinuation(
  request: RoleSessionContinuationStartRequest,
): Promise<void> {
  const safeRequest = createRoleSessionContinuationStartRequest(request);
  ensureTauriRuntime();
  return invoke<void>("start_jiaoban_role_session_continuation", { request: safeRequest });
}

// M4C06 Secretary home context ------------------------------------------------
//
// The command has no renderer-supplied identity, scope, cwd, project locator,
// provider, credential, prompt or callback.  The parser also keeps the M4
// envelope as the authoritative homepage route rather than falling back to a
// legacy Workbench snapshot.

// M4C08 sends no renderer data. The backend fixes the inventory and scope,
// re-reads canonical sources in a read-only transaction and returns a guarded
// report; no legacy summary, tuple, label, route guess or action payload
// crosses this command boundary.
export async function loadSecretaryLegacyReadCompatibilityReport(): Promise<M4LegacyReadCompatibilityReportEnvelopeDto> {
  ensureTauriRuntime();
  return parseSecretaryLegacyReadCompatibilityReportEnvelope(
    await invoke<unknown>("load_secretary_legacy_read_compatibility_report"),
  );
}

export async function loadSecretaryHomeContext(): Promise<M4SecretaryHomeContextEnvelopeDto> {
  ensureTauriRuntime();
  return parseSecretaryHomeContextEnvelope(await invoke<unknown>("load_secretary_home_context"));
}

export async function resolveSecretarySourceRoute(
  request: M4SecretarySourceRouteRequestDto,
): Promise<M4SecretarySourceRouteResolutionDto> {
  const safeRequest = createSecretarySourceRouteRequest(request);
  ensureTauriRuntime();
  const resolution = parseSecretarySourceRouteResolution(
    await invoke<unknown>("resolve_secretary_source_route", { request: safeRequest }),
  );
  if (resolution.source_route_ref !== safeRequest.source_route_ref) {
    throw new Error("m4_secretary_source_route_response_request_mismatch");
  }
  return resolution;
}

// M4C07 has no renderer-supplied selector, scope, timezone, window, or
// recovery flag.  The server resolves all of those inputs and the parser
// rejects a malformed result instead of exposing a stale/local fallback.
export async function loadSecretaryDailyReport(): Promise<M4SecretaryDailyReportEnvelopeDto> {
  ensureTauriRuntime();
  return parseSecretaryDailyReportEnvelope(await invoke<unknown>("load_secretary_daily_report"));
}

export async function recoverSecretaryDailyCatchUp(
  catchUpTruncationId: string,
): Promise<M4SecretaryDailyReportEnvelopeDto> {
  if (!/^catch-up-truncation:[a-f0-9]{64}$/.test(catchUpTruncationId)) {
    throw new Error("m4_secretary_daily_catch_up_reference_invalid");
  }
  ensureTauriRuntime();
  return parseSecretaryDailyReportEnvelope(await invoke<unknown>(
    "recover_secretary_daily_catch_up",
    { catchUpTruncationId },
  ));
}

export async function operateSecretaryCoordination(
  request: M4SecretaryCoordinationActionRequestDto,
): Promise<M4SecretaryCoordinationActionReceiptDto> {
  const safeRequest = createSecretaryCoordinationActionRequest(request);
  ensureTauriRuntime();
  return parseSecretaryCoordinationActionReceipt(
    await invoke<unknown>("operate_secretary_coordination", { request: safeRequest }),
  );
}

export async function operateSecretaryPersonalObject(
  request: M4SecretaryPersonalObjectRequestDto,
): Promise<M4SecretaryCoordinationActionReceiptDto> {
  const safeRequest = createSecretaryPersonalObjectRequest(request);
  ensureTauriRuntime();
  return parseSecretaryCoordinationActionReceipt(
    await invoke<unknown>("operate_secretary_personal_object", { request: safeRequest }),
  );
}

// M3C07 isolated acceptance surface -----------------------------------------
//
// This surface has no legacy thread/user-message transport.  The host is fixed
// by each wrapper and the only renderer-controlled values are an enum action
// and a bounded nonce.
export type M3C07AcceptanceAction =
  | "observe"
  | "new"
  | "continue"
  | "stop"
  | "stage_create_pending"
  | "stage_start_pending"
  | "stage_stop_pending"
  | "restart_readback"
  | "failure_injection_rollback"
  | "handoff_exact_replay"
  | "object_navigation";

export type M3C07AcceptanceStatus = Readonly<{
  runtimeVersion: string;
  host: "agent" | "jiaoban";
  lifecycleState: string;
  sessionState: string;
  turnState: string;
  labels: Readonly<{
    role: string;
    project: string;
    object: string;
    channel: string;
    permission: string;
  }>;
  ledger: Readonly<{
    fakeDispatches: number;
    fakeReadbacks: number;
    realProviderAttempts: number;
    persistentLedger: boolean;
  }>;
  receipt: Readonly<{
    schemaVersion: string;
    receiptId: string;
    host: "agent" | "jiaoban";
    action: string;
    outcome: string;
    replayed: boolean;
    rollbackApplied: boolean;
    realProviderAttempts: number;
    redaction: string;
  }>;
  recovery: Readonly<{
    state: string;
    restartReadbacks: number;
    dispatchesAfterRestart: number;
  }>;
  objectNavigation: Readonly<{
    available: boolean;
    state: string;
  }>;
}>;

function createM3C07AcceptanceActionRequest(
  action: M3C07AcceptanceAction,
  requestNonce: string,
): Readonly<{ action: M3C07AcceptanceAction; requestNonce: string }> {
  if (!requestNonce.trim() || utf8ByteLength(requestNonce) > 160 || /[\u0000-\u001f\u007f]/.test(requestNonce)) {
    throw new Error("m3c07_acceptance_nonce_invalid");
  }
  return Object.freeze({ action, requestNonce });
}

function assertM3C07AcceptanceHost(
  status: M3C07AcceptanceStatus,
  host: "agent" | "jiaoban",
): M3C07AcceptanceStatus {
  if (status.host !== host || status.receipt.host !== host || status.ledger.realProviderAttempts !== 0) {
    throw new Error("m3c07_acceptance_response_invalid");
  }
  return status;
}

export async function loadAgentM3C07AcceptanceStatus(): Promise<M3C07AcceptanceStatus> {
  ensureTauriRuntime();
  return assertM3C07AcceptanceHost(
    await invoke<M3C07AcceptanceStatus>("load_agent_m3c07_acceptance_status"),
    "agent",
  );
}

export async function operateAgentM3C07Acceptance(
  action: M3C07AcceptanceAction,
  requestNonce: string,
): Promise<M3C07AcceptanceStatus> {
  ensureTauriRuntime();
  return assertM3C07AcceptanceHost(
    await invoke<M3C07AcceptanceStatus>("operate_agent_m3c07_acceptance", {
      request: createM3C07AcceptanceActionRequest(action, requestNonce),
    }),
    "agent",
  );
}

export async function loadJiaobanM3C07AcceptanceStatus(): Promise<M3C07AcceptanceStatus> {
  ensureTauriRuntime();
  return assertM3C07AcceptanceHost(
    await invoke<M3C07AcceptanceStatus>("load_jiaoban_m3c07_acceptance_status"),
    "jiaoban",
  );
}

export async function operateJiaobanM3C07Acceptance(
  action: M3C07AcceptanceAction,
  requestNonce: string,
): Promise<M3C07AcceptanceStatus> {
  ensureTauriRuntime();
  return assertM3C07AcceptanceHost(
    await invoke<M3C07AcceptanceStatus>("operate_jiaoban_m3c07_acceptance", {
      request: createM3C07AcceptanceActionRequest(action, requestNonce),
    }),
    "jiaoban",
  );
}

// Shared Conversation Transport -------------------------------------------------
//
// The profile remains a local controller contract so each page can select only
// its own fixed endpoint. It is deliberately removed before invoke: sandbox,
// write roots, approval, MCP endpoint, and capabilities are never Tauri input.
type ConversationTransportStartRequestForContext<TContext> =
  | (Omit<ConversationTransportNewStartRequest, "context"> & Readonly<{ context: TContext }>)
  | (Omit<ConversationTransportLegacyExistingStartRequest, "context"> & Readonly<{ context: TContext }>);

export type AgentCodexConversationTransportStartRequest =
  ConversationTransportStartRequestForContext<AgentConversationTransportContext>;

export type SupervisorCodexConversationTransportStartRequest =
  ConversationTransportStartRequestForContext<SupervisorConversationTransportContext>;

type ConversationTransportInvokeContext = Readonly<{
  project_root: string;
  project_id?: string;
  workflow_id?: string;
}>;

type ConversationTransportInvokeStartRequest = Readonly<{
  context: ConversationTransportInvokeContext;
  mode: "new" | "existing";
  conversation_id: string | null;
  thread_id: string | null;
  turn_id: string;
  user_text: string;
}>;

function conversationTransportInvokeRequest(
  request: AgentCodexConversationTransportStartRequest | SupervisorCodexConversationTransportStartRequest,
  context: ConversationTransportInvokeContext,
): ConversationTransportInvokeStartRequest & Readonly<{ context: ConversationTransportInvokeContext }> {
  return {
    context,
    mode: request.mode,
    conversation_id: request.conversation_id,
    thread_id: request.thread_id,
    turn_id: request.turn_id,
    user_text: request.user_text,
  };
}

export function startAgentCodexConversationTransport(
  request: AgentCodexConversationTransportStartRequest,
): Promise<ConversationTransportReceipt> {
  if (request.context.profile_id !== AGENT_CODEX_WORKSPACE_WRITE_PROFILE) {
    return Promise.reject(new Error("conversation_transport_agent_profile_required"));
  }
  ensureTauriRuntime();
  return invoke<ConversationTransportReceipt>("start_agent_conversation_transport", {
    request: conversationTransportInvokeRequest(request, {
      project_root: request.context.project_root,
    }),
  });
}

export function startSupervisorCodexConversationTransport(
  request: SupervisorCodexConversationTransportStartRequest,
): Promise<ConversationTransportReceipt> {
  if (request.context.profile_id !== SUPERVISOR_READ_ONLY_PROFILE) {
    return Promise.reject(new Error("conversation_transport_supervisor_profile_required"));
  }
  ensureTauriRuntime();
  return invoke<ConversationTransportReceipt>("start_supervisor_conversation_transport", {
    request: conversationTransportInvokeRequest(request, {
      project_root: request.context.project_root,
      project_id: request.context.project_id,
      workflow_id: request.context.workflow_id,
    }),
  });
}

export function pollCodexConversationTransportAttempt(
  request: ConversationTransportAttemptRequest,
): Promise<ConversationTransportReceipt> {
  ensureTauriRuntime();
  return invoke<ConversationTransportReceipt>("poll_conversation_transport_attempt", { request });
}

export function stopCodexConversationTransportAttempt(
  request: ConversationTransportAttemptRequest,
): Promise<ConversationTransportReceipt> {
  ensureTauriRuntime();
  return invoke<ConversationTransportReceipt>("stop_conversation_transport_attempt", { request });
}

export function runManualCodexRelayGuiDirect(
  request: ManualRelayGuiDirectRunInput,
): Promise<ManualRelayReceipt> {
  ensureTauriRuntime();
  return invoke<ManualRelayReceipt>("run_manual_codex_relay_gui_direct", { request });
}

export function runManualCodexRelayGuiDirectNewSession(
  request: ManualRelayGuiDirectNewSessionInput,
): Promise<ManualRelayReceipt> {
  ensureTauriRuntime();
  return invoke<ManualRelayReceipt>("run_manual_codex_relay_gui_direct_new_session", { request });
}

export function stopManualCodexRelayAttempt(
  request: ManualRelayStopInput,
): Promise<ManualRelayReceipt> {
  ensureTauriRuntime();
  return invoke<ManualRelayReceipt>("stop_manual_codex_relay_attempt", { request });
}

export function pollManualCodexRelayAttempt(
  request: ManualRelayPollInput,
): Promise<ManualRelayReceipt> {
  ensureTauriRuntime();
  return invoke<ManualRelayReceipt>("poll_manual_codex_relay_attempt", { request });
}

// =============================================================
// Editable Canvas v1
// =============================================================

export function canvasLoad(canvasId: string): Promise<CanvasDefinition> {
  ensureTauriRuntime();
  return invoke<CanvasDefinition>("canvas_load", { canvasId });
}

export function canvasSave(canvas: CanvasDefinition): Promise<void> {
  ensureTauriRuntime();
  return invoke<void>("canvas_save", { canvas });
}

// ---- Workflow templates (plan B · 成熟模式保留) — data store only ----

export function saveWorkflowTemplate(template: WorkflowTemplate): Promise<void> {
  ensureTauriRuntime();
  return invoke<void>("save_workflow_template", { template });
}

export function listWorkflowTemplates(): Promise<WorkflowTemplateSummary[]> {
  ensureTauriRuntime();
  return invoke<WorkflowTemplateSummary[]>("list_workflow_templates", {});
}

export function loadWorkflowTemplate(templateId: string): Promise<WorkflowTemplate> {
  ensureTauriRuntime();
  return invoke<WorkflowTemplate>("load_workflow_template", { templateId });
}

export function deleteWorkflowTemplate(templateId: string): Promise<void> {
  ensureTauriRuntime();
  return invoke<void>("delete_workflow_template", { templateId });
}

// ---- Canvas node real run (plan C1) — wires to the EXISTING double-gated
// command. Zero new/relaxed gate: the backend returns a blocked message unless
// the fixed test project + WORKFLOW_ENGINE_TEST_CONFIRM env key are both set.
// The frontend only sends the request; it never decides execution.
export function executeWorkflowNodeDispatch(request: CanvasNodeDispatchRequest): Promise<unknown> {
  ensureTauriRuntime();
  return invoke<unknown>("execute_workflow_node_dispatch", { request });
}

// P3 实验面真跑（架构方案 §9 的 A 映射）。目标恒为固定测试项目（后端硬锁 project_root，前端传
// 不进），临时 work_item 后端自动建。同样零新闸：非测试项目仍被 path-lock 挡（这条本就只打测试项目）。
export function executeExperimentNodeDispatch(request: ExperimentNodeDispatchRequest): Promise<unknown> {
  ensureTauriRuntime();
  return invoke<unknown>("execute_experiment_node_dispatch", { request });
}

// P3 项目面真跑（C 映射）。目标项目由前端传入（项目面绑定的项目），非固定测试项目仍被后端
// path-lock 挡下。派发指令后端从 work_item 任务包构造，会话用节点既有绑定。
export function executeProjectWorkflowNode(request: ProjectWorkflowNodeRunRequest): Promise<unknown> {
  ensureTauriRuntime();
  return invoke<unknown>("execute_project_workflow_node", { request });
}

// P1 工作流自动连环（决策 2026-06-23 · 圈固定测试项目）：起链 = 按拓扑序逐节点自动真跑到底。
// 前端只造请求 + 发——闸在后端 path-lock，非测试项目造不了钥匙、按钮单独开不了闸。
export function startProjectWorkflowChain(
  request: ProjectWorkflowChainRunRequest,
): Promise<ProjectWorkflowChainRunResult> {
  ensureTauriRuntime();
  return invoke<ProjectWorkflowChainRunResult>("start_project_workflow_chain", { request });
}
export function stopProjectWorkflowChain(
  request: ProjectWorkflowChainStopRequest,
): Promise<ProjectWorkflowChainRunResult> {
  ensureTauriRuntime();
  return invoke<ProjectWorkflowChainRunResult>("stop_project_workflow_chain", { request });
}
// #19 实时进度：轮询该工作流最新一条链运行记录（state + 每节点状态）。只读、无副作用。
export function getProjectWorkflowChainStatus(
  projectRoot: string,
  workflowId: string,
): Promise<ProjectWorkflowChainStatus | null> {
  ensureTauriRuntime();
  return invoke<ProjectWorkflowChainStatus | null>("get_project_workflow_chain_status", {
    projectRoot,
    workflowId,
  });
}
// C1 主管链：起整条主管链（收前端回传的已审 planned_tasks·后端 spawn_blocking 调 run_director_task_chain）。
// 停/进度复用上面的 stopProjectWorkflowChain / getProjectWorkflowChainStatus（主管链建同种链记录）。
export function startProjectDirectorChain(
  request: StartProjectDirectorChainRequest,
): Promise<DirectorChainOutcome> {
  ensureTauriRuntime();
  return invoke<DirectorChainOutcome>("start_project_director_chain", { request });
}

// 历史兼容命令：S1 已退役这条“直接让 AI 出方案”路线，调用会得到明确错误；
// 请改用项目主管对话，只有主管私有 submit_proposal 工具可以创建方案卡。
export function runProjectConsultation(
  request: RunProjectConsultationRequest,
): Promise<ProjectConsultationProposal> {
  ensureTauriRuntime();
  return invoke<ProjectConsultationProposal>("run_project_consultation", { request });
}

// S1：用户消息统一注入常驻主管 thread。前端只能以 user identity 调用；方案等实物仍必须
// 由主管 MCP 工具落库，批准仍在右侧方案卡。
export function submitSupervisorResidentAnswer(
  request: SubmitSupervisorResidentAnswerRequest,
): Promise<SupervisorResidentAnswerOutcome> {
  ensureTauriRuntime();
  return invoke<SupervisorResidentAnswerOutcome>("submit_supervisor_resident_answer", { request });
}

// 件 B · 授权后自动推进（核心·步骤塌缩）：前提 = 已有 active 授权；一下串 拆任务→prepare→worker 链跑。
// 前端只造请求 + 发，闸在后端（path-lock 圈测试项目·无 active 授权后端拒）；按 outcome.stage 分支展示。
export function autoAdvanceAuthorizedRoleLoop(
  request: AutoAdvanceAuthorizedRoleLoopRequest,
): Promise<AutoAdvanceRoleLoopOutcome> {
  ensureTauriRuntime();
  return invoke<AutoAdvanceRoleLoopOutcome>("auto_advance_authorized_role_loop", { request });
}

// 主管四选一处置：前端只把用户已选择的动作和同一份任务带回，转移/授权/C1 均仍在后端校验。
export function applyProjectDirectorFailedAction(
  request: ProjectDirectorFailedActionRequest,
): Promise<ProjectDirectorFailedActionOutcome> {
  ensureTauriRuntime();
  return invoke<ProjectDirectorFailedActionOutcome>("apply_project_director_failed_action", { request });
}

// 交办地基 2.2 合流命令（刀1 已注册）：用户点[允许并开始]那一下 → 后端一原子命令做完
// 确认方案 + 边界复核 + 授权生效 + 绑现有会话 + 自动推进；返回同形 outcome（组件按 stage 分支不变）。
// 纯封装、不带逻辑：前端只造请求 + 发；人闸仍是用户点击那一下，闸在后端 path-lock（圈固定测试项目）。
export function confirmAndStartAuthorizedRun(
  request: ConfirmAndStartAuthorizedRunRequest,
): Promise<AutoAdvanceRoleLoopOutcome> {
  ensureTauriRuntime();
  return invoke<AutoAdvanceRoleLoopOutcome>("confirm_and_start_authorized_run", { request });
}

// P1-E 死码清扫（2026-07-18）：confirmProjectDirectorTaskSessionBindings 前端 wrapper 零调用者
// （P1-D 摘了绑定停点面板后没人再调）——后端命令 confirm_project_director_task_session_bindings
// 仍注册在案（director_agent.rs），本包不动它：留给 P2-B「挑会话降为可选项」判是否复用，明示不静默漂。

// Station 2 · 主管编排试点：只发给后端新发射器；授权确认仍复用现有人闸命令。
export function launchSupervisorPilot(
  request: SupervisorPilotLaunchRequest,
): Promise<SupervisorPilotLaunchReceipt> {
  ensureTauriRuntime();
  return invoke<SupervisorPilotLaunchReceipt>("launch_supervisor_pilot", { request });
}

// 主管账本只读投影：不读取、更不写工作流链状态。
export function loadSupervisorPilotReadModel(
  request: SupervisorPilotReadModelRequest,
): Promise<SupervisorPilotReadModel> {
  ensureTauriRuntime();
  return invoke<SupervisorPilotReadModel>("load_supervisor_pilot_read_model", { request });
}

// B1·全局主管复核（advisory·意见不是闸）：交货后自动触发，读盘复核本轮口供出意见；幂等防重烧
// （同轮已有记录直接回、[重新复核]/[重试] 才 force）。纯封装无逻辑；输入只传定位键（内容后端盘读）。
export function runGlobalSupervisorReview(
  request: RunGlobalSupervisorReviewRequest,
): Promise<GlobalSupervisorReviewOutcome> {
  ensureTauriRuntime();
  return invoke<GlobalSupervisorReviewOutcome>("run_global_supervisor_review", { request });
}

// B2·全局主管批前边界意见（advisory·意见不是闸）：批脸自动触发，读盘上 pending 方案出「目标 vs 方案」意见；
// 幂等 by proposal_id（同方案已有记录直接回、[重试] 才 force）。纯封装无逻辑；输入只传定位键（内容后端盘读）。
export function runGlobalSupervisorBoundaryReview(
  request: RunGlobalSupervisorBoundaryReviewRequest,
): Promise<GlobalSupervisorBoundaryReviewOutcome> {
  ensureTauriRuntime();
  return invoke<GlobalSupervisorBoundaryReviewOutcome>("run_global_supervisor_boundary_review", { request });
}

// 工作历史·后端读模型（纯只读·跨店按 workflow+时间窗拼单列表）。薄封装无逻辑；零 UI（UI 半包等 M1）。
export function listProjectRunHistory(request: ListProjectRunHistoryRequest): Promise<RunHistoryList> {
  ensureTauriRuntime();
  return invoke<RunHistoryList>("list_project_run_history", { request });
}

// 刀2「批前看图」封装：对 pending 方案只读预拆工序图（零写盘·真 LM 1-7 分钟·偶发 flaky）。纯封装无逻辑；
// 前端拿 planned_tasks 画迷你图 + 原样回传给 confirm 的 approved_planned_tasks（所见即所跑）。
export function previewPendingProposalDirectorPlan(
  request: PreviewPendingProposalDirectorPlanRequest,
): Promise<PreviewPendingProposalDirectorPlanOutcome> {
  ensureTauriRuntime();
  return invoke<PreviewPendingProposalDirectorPlanOutcome>("preview_pending_proposal_director_plan", { request });
}

// P3 E · 多工作流底座（架构 §12）。
export function listProjectWorkflows(projectRoot: string): Promise<ProjectWorkflowListItem[]> {
  ensureTauriRuntime();
  return invoke<ProjectWorkflowListItem[]>("list_project_workflows", { projectRoot });
}
export function submitProjectWorkflowDraft(request: SubmitProjectWorkflowDraftRequest): Promise<{ message?: string }> {
  ensureTauriRuntime();
  return invoke<{ message?: string }>("submit_project_workflow_draft", { request });
}
// 取某工作流的画布节点/边，供「编辑工作流」把现有 nodes 加载进草案（避免空白覆盖）。
export function getProjectWorkflowNodes(
  projectRoot: string,
  workflowId: string,
): Promise<{ nodes: unknown[]; edges: unknown[] }> {
  ensureTauriRuntime();
  return invoke<{ nodes: unknown[]; edges: unknown[] }>("get_project_workflow_nodes", { projectRoot, workflowId });
}

function ensureTauriRuntime() {
  if (!("__TAURI_INTERNALS__" in window)) {
    throw new Error("当前页面不在 Tauri 窗口中运行；请使用 npm run tauri:dev 启动桌面壳。");
  }
}
