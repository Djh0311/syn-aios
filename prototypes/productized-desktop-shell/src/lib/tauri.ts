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
  CanvasRunState,
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
  WorkflowMachineRunRequest,
  WorkflowMachineRunResult,
  WorkflowNodeSessionBindRequest,
  WorkflowDispatchDirectorReviewRequest,
  WorkflowPermissionDecisionRequest,
  WorkflowNodeDispatchExecuteRequest,
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

export function executeLegacyWorkflowNodeDispatch(request: WorkflowNodeDispatchExecuteRequest): Promise<WorkflowNodeDispatchResult> {
  ensureTauriRuntime();
  return invoke<WorkflowNodeDispatchResult>("execute_workflow_node_dispatch", { request });
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

export function runLegacyWorkflowMachine(request: WorkflowMachineRunRequest): Promise<WorkflowMachineRunResult> {
  ensureTauriRuntime();
  return invoke<WorkflowMachineRunResult>("run_workflow_machine", { request });
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

export type CanvasStartRunRequest = { canvas_id: string; goal: string };
export type CanvasStartRunResult = {
  run_id: string;
  state_path: string;
  run: CanvasRunState;
};
export type CanvasRunStatus = {
  run: CanvasRunState;
  last_decision: unknown | null;
};

/**
 * @deprecated Legacy experiment canvas real run is sealed by the backend.
 * Use the H-stage unified product command boundary before any real Codex execution.
 */
export function canvasStartRun(request: CanvasStartRunRequest): Promise<CanvasStartRunResult> {
  ensureTauriRuntime();
  return invoke<CanvasStartRunResult>("canvas_start_run", { request });
}

export function canvasAbortRun(runId: string, reason: string): Promise<CanvasRunStatus> {
  ensureTauriRuntime();
  return invoke<CanvasRunStatus>("canvas_abort_run", { runId, reason });
}

export function canvasRunStatus(runId: string): Promise<CanvasRunStatus> {
  ensureTauriRuntime();
  return invoke<CanvasRunStatus>("canvas_run_status", { runId });
}

/**
 * @deprecated Legacy experiment canvas tick can spawn Codex and is sealed by the backend.
 */
export function canvasTickRun(runId: string): Promise<unknown> {
  ensureTauriRuntime();
  return invoke<unknown>("canvas_tick_run", { runId });
}

function ensureTauriRuntime() {
  if (!("__TAURI_INTERNALS__" in window)) {
    throw new Error("当前页面不在 Tauri 窗口中运行；请使用 npm run tauri:dev 启动桌面壳。");
  }
}
