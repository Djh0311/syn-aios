import { invoke } from "@tauri-apps/api/core";
import type { PageReadModelQueryInput, PageReadModelQueryResult } from "./pageReadModel";
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
  AutoAdvanceAuthorizedRoleLoopRequest,
  AutoAdvanceRoleLoopOutcome,
  ConfirmAndStartAuthorizedRunRequest,
  PreviewPendingProposalDirectorPlanRequest,
  PreviewPendingProposalDirectorPlanOutcome,
  RunGlobalSupervisorReviewRequest,
  GlobalSupervisorReviewOutcome,
  RunGlobalSupervisorBoundaryReviewRequest,
  GlobalSupervisorBoundaryReviewOutcome,
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
  WorkflowNodeDispatchPrepareRequest,
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

export function prepareWorkflowNodeDispatch(request: WorkflowNodeDispatchPrepareRequest): Promise<WorkflowNodeDispatchResult> {
  ensureTauriRuntime();
  return invoke<WorkflowNodeDispatchResult>("prepare_workflow_node_dispatch", { request });
}

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

// 件 A · 接咨询 LM（出方案自动化）：目标 → 真 codex 只读咨询 → 写一份 PendingUserConfirmation 方案（不自动确认·人闸守住）。
// 异步长耗时（真 codex），前端只造请求 + 发；返回新建的方案（同 ProjectConsultationProposal 形）。
export function runProjectConsultation(
  request: RunProjectConsultationRequest,
): Promise<ProjectConsultationProposal> {
  ensureTauriRuntime();
  return invoke<ProjectConsultationProposal>("run_project_consultation", { request });
}

// 件 B · 授权后自动推进（核心·步骤塌缩）：前提 = 已有 active 授权；一下串 拆任务→prepare→worker 链跑。
// 前端只造请求 + 发，闸在后端（path-lock 圈测试项目·无 active 授权后端拒）；按 outcome.stage 分支展示。
export function autoAdvanceAuthorizedRoleLoop(
  request: AutoAdvanceAuthorizedRoleLoopRequest,
): Promise<AutoAdvanceRoleLoopOutcome> {
  ensureTauriRuntime();
  return invoke<AutoAdvanceRoleLoopOutcome>("auto_advance_authorized_role_loop", { request });
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
