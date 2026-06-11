// Tauri command wrappers split out during Task B conservative module split.
// This file is included at crate root to preserve command names and behavior.

#[tauri::command]
fn load_workbench_snapshot(state: tauri::State<'_, AppState>) -> Result<WorkbenchSnapshot, String> {
    let index = read_index(&state)?;
    let tasks_text = fs::read_to_string(&state.tasks_path).unwrap_or_default();
    Ok(build_snapshot(&state, &index, &tasks_text))
}

#[tauri::command]
fn query_workbench_page_read_model(
    request: page_read_model::PageReadModelQueryInput,
    state: tauri::State<'_, AppState>,
) -> Result<page_read_model::PageReadModelQueryResult, String> {
    let index = read_index(&state)?;
    let generated_at = optional_string(&index, "generated_at").unwrap_or_else(unix_timestamp_string);
    page_read_model::query_page_read_model(&request, &generated_at)
}

#[tauri::command]
fn load_codex_session_transcript(
    thread_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<CodexTranscript, String> {
    load_codex_session_transcript_for_index(&state, &thread_id)
}

#[tauri::command]
fn load_workflow_state_snapshot(
    state: tauri::State<'_, AppState>,
) -> Result<WorkflowStateSnapshot, String> {
    read_workflow_state_snapshot(&state.workflow_state_path)
}

#[tauri::command]
fn load_plan_authorization_store(
    state: tauri::State<'_, AppState>,
) -> Result<PlanAuthorizationStoreV1, String> {
    plan_authorization_store::load_store(&state.workflow_state_path, unix_timestamp_ms())
}

#[tauri::command]
fn create_plan_authorization(
    request: CreatePlanAuthorizationInput,
    state: tauri::State<'_, AppState>,
) -> Result<CreatePlanAuthorizationOutput, String> {
    plan_authorization_store::create_authorization(
        &state.workflow_state_path,
        &request,
        unix_timestamp_ms(),
        &format!("write-plan-authorization-{}", unix_timestamp_nanos()),
    )
}

#[tauri::command]
fn record_plan_authorization_user_confirmation(
    request: RecordPlanAuthorizationUserConfirmationInput,
    state: tauri::State<'_, AppState>,
) -> Result<RecordPlanAuthorizationOutput, String> {
    plan_authorization_store::record_user_confirmation(
        &state.workflow_state_path,
        &request,
        unix_timestamp_ms(),
        &format!("write-plan-authorization-user-{}", unix_timestamp_nanos()),
    )
}

#[tauri::command]
fn record_plan_authorization_global_boundary_review(
    request: RecordPlanAuthorizationGlobalBoundaryReviewInput,
    state: tauri::State<'_, AppState>,
) -> Result<RecordPlanAuthorizationOutput, String> {
    plan_authorization_store::record_global_boundary_review(
        &state.workflow_state_path,
        &request,
        unix_timestamp_ms(),
        &format!(
            "write-plan-authorization-boundary-{}",
            unix_timestamp_nanos()
        ),
    )
}

#[tauri::command]
fn record_global_boundary_review(
    request: RecordGlobalBoundaryReviewInput,
    state: tauri::State<'_, AppState>,
) -> Result<RecordGlobalBoundaryReviewOutput, String> {
    plan_authorization_store::record_global_boundary_review_with_proposal(
        &state.workflow_state_path,
        &request,
        unix_timestamp_ms(),
        &format!("write-global-boundary-review-{}", unix_timestamp_nanos()),
    )
}

#[tauri::command]
fn revoke_plan_authorization(
    request: RevokePlanAuthorizationInput,
    state: tauri::State<'_, AppState>,
) -> Result<RecordPlanAuthorizationOutput, String> {
    plan_authorization_store::revoke_authorization(
        &state.workflow_state_path,
        &request,
        unix_timestamp_ms(),
        &format!("write-plan-authorization-revoke-{}", unix_timestamp_nanos()),
    )
}

#[tauri::command]
fn inspect_auto_dispatch_authorization(
    request: AutoDispatchGuardInput,
    state: tauri::State<'_, AppState>,
) -> Result<AutoDispatchGuardResult, String> {
    plan_authorization_store::inspect_auto_dispatch_authorization(
        &state.workflow_state_path,
        &request,
        unix_timestamp_ms(),
        &format!("write-auto-dispatch-scope-check-{}", unix_timestamp_nanos()),
    )
}

#[tauri::command]
fn preview_project_director_task_plan(
    request: PreviewProjectDirectorTaskPlanInput,
    state: tauri::State<'_, AppState>,
) -> Result<ProjectDirectorTaskPlan, String> {
    let index = read_index(&state)?;
    preview_project_director_task_plan_for_index_at(&state.workflow_state_path, &index, &request)
}

#[tauri::command]
fn prepare_authorized_auto_dispatch(
    request: PrepareAuthorizedAutoDispatchInput,
    state: tauri::State<'_, AppState>,
) -> Result<AuthorizedPreparedDispatchResult, String> {
    let index = read_index(&state)?;
    prepare_authorized_auto_dispatch_for_index_at(&state.workflow_state_path, &index, &request)
}

#[tauri::command]
fn preview_h5_project_workflow_dispatch(
    request: H5ProjectWorkflowDispatchPreviewInput,
    state: tauri::State<'_, AppState>,
) -> Result<H5ProjectWorkflowDispatchPreview, String> {
    h5_project_dispatch_bridge::preview_h5_project_workflow_dispatch_at(
        &state.workflow_state_path,
        &request,
    )
}

#[tauri::command]
fn preview_real_execution_product_command(
    request: PreviewRealExecutionProductCommandInput,
    state: tauri::State<'_, AppState>,
) -> Result<RealExecutionProductCommandPreview, String> {
    real_execution_command::preview_real_execution_product_command_at(
        &state.workflow_state_path,
        &request,
    )
}

#[tauri::command]
fn prepare_real_execution_product_command(
    request: PrepareRealExecutionProductCommandInput,
    state: tauri::State<'_, AppState>,
) -> Result<RealExecutionProductCommandPrepareOutput, String> {
    real_execution_command::prepare_real_execution_product_command_at(
        &state.workflow_state_path,
        &request,
    )
}

#[tauri::command]
fn record_real_execution_product_command_decision(
    request: RecordRealExecutionProductCommandDecisionInput,
    state: tauri::State<'_, AppState>,
) -> Result<RealExecutionProductCommandDecisionOutput, String> {
    real_execution_command::record_real_execution_product_command_decision_at(
        &state.workflow_state_path,
        &request,
    )
}

#[tauri::command]
fn confirm_real_execution_product_command(
    request: ConfirmRealExecutionProductCommandInput,
    state: tauri::State<'_, AppState>,
) -> Result<RealExecutionProductCommandDecisionOutput, String> {
    real_execution_command::confirm_real_execution_product_command_at(
        &state.workflow_state_path,
        &request,
    )
}

#[tauri::command]
fn run_real_execution_product_command_phase_a(
    request: RunRealExecutionProductCommandPhaseAInput,
    state: tauri::State<'_, AppState>,
) -> Result<RealExecutionProductCommandPhaseAOutput, String> {
    real_execution_command::run_real_execution_product_command_phase_a_at(
        &state.workflow_state_path,
        &request,
        &unix_timestamp_string(),
        &format!("write-real-exec-command-phase-a-{}", unix_timestamp_nanos()),
    )
}

#[tauri::command]
fn run_real_execution_product_command_phase_b(
    request: RunRealExecutionProductCommandPhaseBInput,
    state: tauri::State<'_, AppState>,
) -> Result<RealExecutionProductCommandPhaseBOutput, String> {
    real_execution_command::run_real_execution_product_command_phase_b_at(
        &state.workflow_state_path,
        &request,
        &unix_timestamp_string(),
        &format!("write-real-exec-command-phase-b-{}", unix_timestamp_nanos()),
    )
}

#[tauri::command]
fn run_real_execution_product_command_new_session_phase_b(
    request: RunRealExecutionProductCommandNewSessionPhaseBInput,
    state: tauri::State<'_, AppState>,
) -> Result<RealExecutionProductCommandPhaseBOutput, String> {
    real_execution_command::run_real_execution_product_command_new_session_phase_b_at(
        &state.workflow_state_path,
        &request,
        &unix_timestamp_string(),
        &format!(
            "write-real-exec-command-new-session-phase-b-{}",
            unix_timestamp_nanos()
        ),
    )
}

#[tauri::command]
fn run_project_workflow_automation_phase_a(
    request: ProjectWorkflowAutomationInput,
    state: tauri::State<'_, AppState>,
) -> Result<ProjectWorkflowAutomationResult, String> {
    project_workflow_automation::run_project_workflow_automation_phase_a_at(
        &state.workflow_state_path,
        &request,
        &unix_timestamp_string(),
        &format!("write-j2-project-workflow-automation-{}", unix_timestamp_nanos()),
    )
}

#[tauri::command]
fn run_project_workflow_automation_j2_b_b1(
    request: ProjectWorkflowAutomationJ2BB1Input,
    state: tauri::State<'_, AppState>,
) -> Result<ProjectWorkflowAutomationJ2BB1Output, String> {
    project_workflow_automation::run_project_workflow_automation_j2_b_b1_at(
        &state.workflow_state_path,
        &request,
        &unix_timestamp_string(),
        &format!("write-j2-b-b1-project-workflow-{}", unix_timestamp_nanos()),
    )
}

#[tauri::command]
fn run_project_workflow_automation_j2_b_b2(
    request: ProjectWorkflowAutomationJ2BB2Input,
    state: tauri::State<'_, AppState>,
) -> Result<ProjectWorkflowAutomationJ2BB2Output, String> {
    project_workflow_automation::run_project_workflow_automation_j2_b_b2_at(
        &state.workflow_state_path,
        &request,
        &unix_timestamp_string(),
        &format!("write-j2-b-b2-project-workflow-{}", unix_timestamp_nanos()),
    )
}

#[tauri::command]
fn run_project_workflow_automation_k3_b(
    request: ProjectWorkflowAutomationK3BInput,
    state: tauri::State<'_, AppState>,
) -> Result<ProjectWorkflowAutomationK3BOutput, String> {
    ensure_k3_b_tauri_no_real_harness_request(&request)?;
    project_workflow_automation::run_project_workflow_automation_k3_b_at(
        &state.workflow_state_path,
        &request,
        &unix_timestamp_string(),
        &format!("write-k3-b-project-workflow-{}", unix_timestamp_nanos()),
    )
}

fn ensure_k3_b_tauri_no_real_harness_request(
    request: &ProjectWorkflowAutomationK3BInput,
) -> Result<(), String> {
    if request
        .runtime_prompt_body
        .as_deref()
        .is_some_and(|body| !body.is_empty())
    {
        return Err(
            "k3_b_real_execution_requires_dedicated_level_b_authorization".to_string(),
        );
    }
    Ok(())
}

#[tauri::command]
fn record_worker_structured_report(
    request: WorkerStructuredReportInput,
    state: tauri::State<'_, AppState>,
) -> Result<WorkflowStateMutationResult, String> {
    record_worker_structured_report_at(&state.workflow_state_path, &request)
}

#[tauri::command]
fn record_project_director_process_fact_decision(
    request: ProjectDirectorProcessFactDecisionInput,
    state: tauri::State<'_, AppState>,
) -> Result<ProjectDirectorProcessFactDecisionResult, String> {
    record_project_director_process_fact_decision_at(&state.workflow_state_path, &request)
}

#[tauri::command]
fn record_global_final_result_review(
    request: GlobalFinalResultReviewInput,
    state: tauri::State<'_, AppState>,
) -> Result<WorkflowStateMutationResult, String> {
    record_global_final_result_review_at(&state.workflow_state_path, &request)
}

#[tauri::command]
fn record_user_result_decision(
    request: UserResultDecisionInput,
    state: tauri::State<'_, AppState>,
) -> Result<WorkflowStateMutationResult, String> {
    record_user_result_decision_at(&state.workflow_state_path, &request)
}

#[tauri::command]
fn generate_stage_c_acceptance_summary(
    request: GenerateStageCAcceptanceSummaryInput,
    state: tauri::State<'_, AppState>,
) -> Result<WorkflowStateMutationResult, String> {
    generate_stage_c_acceptance_summary_at(&state.workflow_state_path, &request)
}

#[tauri::command]
fn load_project_consultation_proposal_store(
    state: tauri::State<'_, AppState>,
) -> Result<ProjectConsultationProposalStoreV1, String> {
    project_consultation_proposal_store::load_store(&state.workflow_state_path, unix_timestamp_ms())
}

#[tauri::command]
fn create_project_consultation_proposal(
    request: CreateProjectConsultationProposalInput,
    state: tauri::State<'_, AppState>,
) -> Result<CreateProjectConsultationProposalOutput, String> {
    project_consultation_proposal_store::create_proposal(
        &state.workflow_state_path,
        &request,
        unix_timestamp_ms(),
        &format!(
            "write-project-consultation-proposal-{}",
            unix_timestamp_nanos()
        ),
    )
}

#[tauri::command]
fn render_project_consultation_proposal_markdown(
    request: RenderProjectConsultationProposalMarkdownInput,
    state: tauri::State<'_, AppState>,
) -> Result<ProjectConsultationProposalMarkdown, String> {
    project_consultation_proposal_store::render_markdown(
        &state.workflow_state_path,
        &request,
        unix_timestamp_ms(),
    )
}

#[tauri::command]
fn record_project_consultation_proposal_decision(
    request: RecordProjectConsultationProposalDecisionInput,
    state: tauri::State<'_, AppState>,
) -> Result<RecordProjectConsultationProposalDecisionOutput, String> {
    project_consultation_proposal_store::record_decision(
        &state.workflow_state_path,
        &request,
        unix_timestamp_ms(),
        &format!(
            "write-project-consultation-proposal-decision-{}",
            unix_timestamp_nanos()
        ),
        &format!(
            "write-project-consultation-plan-authorization-{}",
            unix_timestamp_nanos()
        ),
        &format!(
            "write-project-consultation-plan-confirm-{}",
            unix_timestamp_nanos()
        ),
    )
}

#[tauri::command]
fn load_session_continuation_store(
    state: tauri::State<'_, AppState>,
) -> Result<SessionContinuationStoreV1, String> {
    session_continuation_store::load_store(&state.workflow_state_path, &unix_timestamp_string())
}

#[tauri::command]
fn confirm_controlled_session_continuation(
    request: ConfirmControlledSessionContinuationInput,
    state: tauri::State<'_, AppState>,
) -> Result<ConfirmControlledSessionContinuationOutput, String> {
    session_continuation_store::confirm_continuation(
        &state.workflow_state_path,
        &request,
        &unix_timestamp_string(),
        &format!("write-session-continuation-{}", unix_timestamp_nanos()),
    )
}

#[tauri::command]
fn run_controlled_session_continuation_stub(
    request: RunControlledSessionContinuationStubInput,
    state: tauri::State<'_, AppState>,
) -> Result<RunControlledSessionContinuationStubOutput, String> {
    session_continuation_store::run_stub(
        &state.workflow_state_path,
        &request,
        &unix_timestamp_string(),
        &format!("write-session-continuation-stub-{}", unix_timestamp_nanos()),
    )
}

#[tauri::command]
fn inspect_controlled_session_continuation_real_resume_authorization(
    request: InspectControlledSessionContinuationRealResumeInput,
    state: tauri::State<'_, AppState>,
) -> Result<InspectControlledSessionContinuationRealResumeOutput, String> {
    session_continuation_store::inspect_real_resume_authorization(
        &state.workflow_state_path,
        &request,
        &unix_timestamp_string(),
        &format!(
            "write-session-continuation-h2-preflight-{}",
            unix_timestamp_nanos()
        ),
    )
}

#[tauri::command]
fn run_controlled_session_continuation_real_resume_phase_a(
    request: RunControlledSessionContinuationRealResumePhaseAInput,
    state: tauri::State<'_, AppState>,
) -> Result<RunControlledSessionContinuationRealResumePhaseAOutput, String> {
    session_continuation_store::run_real_resume_phase_a(
        &state.workflow_state_path,
        &request,
        &unix_timestamp_string(),
        &format!(
            "write-session-continuation-h2-phase-a-{}",
            unix_timestamp_nanos()
        ),
    )
}

#[tauri::command]
fn run_controlled_session_continuation_real_resume_phase_b(
    request: RunControlledSessionContinuationRealResumePhaseBInput,
    state: tauri::State<'_, AppState>,
) -> Result<RunControlledSessionContinuationRealResumePhaseBOutput, String> {
    session_continuation_store::run_real_resume_phase_b(
        &state.workflow_state_path,
        &request,
        &unix_timestamp_string(),
        &format!(
            "write-session-continuation-h2-phase-b-{}",
            unix_timestamp_nanos()
        ),
    )
}

#[tauri::command]
fn load_blackboard_candidate_store(
    state: tauri::State<'_, AppState>,
) -> Result<BlackboardCandidateStoreV1, String> {
    blackboard_candidate_store::load_store(&state.workflow_state_path, &unix_timestamp_string())
}

#[tauri::command]
fn record_blackboard_candidate_decision(
    request: RecordBlackboardCandidateDecisionInput,
    state: tauri::State<'_, AppState>,
) -> Result<RecordBlackboardCandidateDecisionOutput, String> {
    control_core::validate_blackboard_candidate_decision(
        blackboard_entry_kind_name(request.entry_kind),
        blackboard_target_kind_name(request.target_kind),
        blackboard_state_name(request.requested_state),
    )?;
    blackboard_candidate_store::record_decision(
        &state.workflow_state_path,
        &request,
        &unix_timestamp_string(),
        &format!("write-{}", unix_timestamp_nanos()),
    )
}

#[tauri::command]
fn load_observation_store(state: tauri::State<'_, AppState>) -> Result<ObservationStoreV1, String> {
    observation_store::load_store(&state.workflow_state_path, &unix_timestamp_string())
}

#[tauri::command]
fn load_memory_capture_store(
    state: tauri::State<'_, AppState>,
) -> Result<MemoryCaptureStoreV1, String> {
    memory_capture_bus::load_store(&state.workflow_state_path, &unix_timestamp_string())
}

#[tauri::command]
fn capture_memory_event(
    request: CaptureMemoryEventInput,
    state: tauri::State<'_, AppState>,
) -> Result<CaptureMemoryEventOutput, String> {
    let timestamp = unix_timestamp_string();
    memory_capture_bus::capture_event(
        &state.workflow_state_path,
        &request,
        &timestamp,
        &format!("write-memory-capture-{}", unix_timestamp_nanos()),
        &format!("write-memory-capture-observation-{}", unix_timestamp_nanos()),
        &format!("write-memory-capture-candidate-{}", unix_timestamp_nanos()),
    )
}

#[tauri::command]
fn create_observation(
    request: CreateObservationInput,
    state: tauri::State<'_, AppState>,
) -> Result<CreateObservationOutput, String> {
    create_observation_at(
        &state.workflow_state_path,
        &request,
        &unix_timestamp_string(),
        &format!("write-observation-{}", unix_timestamp_nanos()),
    )
}

#[tauri::command]
fn create_memory_candidate_from_observation(
    request: CreateMemoryCandidateFromObservationInput,
    state: tauri::State<'_, AppState>,
) -> Result<CreateMemoryCandidateFromObservationOutput, String> {
    let timestamp = unix_timestamp_string();
    create_memory_candidate_from_observation_at(
        &state.workflow_state_path,
        &request,
        &timestamp,
        &format!("write-observation-candidate-{}", unix_timestamp_nanos()),
        &format!(
            "write-memory-candidate-from-observation-{}",
            unix_timestamp_nanos()
        ),
    )
}

#[tauri::command]
fn preview_task_memory_packet(
    request: TaskMemoryPacketBuildInput,
    state: tauri::State<'_, AppState>,
) -> Result<TaskMemoryPacketBuildOutput, String> {
    preview_task_memory_packet_at(
        &state.workflow_state_path,
        &request,
        &unix_timestamp_string(),
    )
}

#[tauri::command]
fn load_memory_lint_store(state: tauri::State<'_, AppState>) -> Result<MemoryLintStoreV1, String> {
    memory_lint_store::load_store(&state.workflow_state_path, &unix_timestamp_string())
}

#[tauri::command]
fn load_memory_entity_relation_store(
    state: tauri::State<'_, AppState>,
) -> Result<MemoryEntityRelationStoreV1, String> {
    memory_entity_relation_store::load_store(&state.workflow_state_path, &unix_timestamp_string())
}

#[tauri::command]
fn load_memory_pattern_store(
    state: tauri::State<'_, AppState>,
) -> Result<MemoryPatternStoreV1, String> {
    mature_pattern_store::load_store(&state.workflow_state_path, &unix_timestamp_string())
}

#[tauri::command]
fn preview_mature_patterns(
    request: PreviewMaturePatternsInput,
    state: tauri::State<'_, AppState>,
) -> Result<MaturePatternPreviewOutput, String> {
    mature_pattern_governance::preview_mature_patterns(
        &state.workflow_state_path,
        &request,
        &unix_timestamp_string(),
    )
}

#[tauri::command]
fn record_mature_pattern_decision(
    request: RecordMaturePatternDecisionInput,
    state: tauri::State<'_, AppState>,
) -> Result<RecordMaturePatternDecisionOutput, String> {
    mature_pattern_governance::record_mature_pattern_decision(
        &state.workflow_state_path,
        &request,
        &unix_timestamp_string(),
        &format!("write-memory-pattern-{}", unix_timestamp_nanos()),
        &format!("write-formal-memory-pattern-{}", unix_timestamp_nanos()),
    )
}

#[tauri::command]
fn preview_memory_entity_relation_candidates(
    request: PreviewMemoryEntityRelationCandidatesInput,
    state: tauri::State<'_, AppState>,
) -> Result<MemoryEntityRelationPreviewOutput, String> {
    memory_entity_relation_governance::preview_candidates(
        &state.workflow_state_path,
        &request,
        &unix_timestamp_string(),
    )
}

#[tauri::command]
fn record_memory_entity_alias_decision(
    request: RecordMemoryEntityAliasDecisionInput,
    state: tauri::State<'_, AppState>,
) -> Result<RecordMemoryEntityAliasDecisionOutput, String> {
    memory_entity_relation_governance::record_alias_decision(
        &state.workflow_state_path,
        &request,
        &unix_timestamp_string(),
        &format!("write-memory-entity-alias-{}", unix_timestamp_nanos()),
    )
}

#[tauri::command]
fn record_memory_entity_merge_decision(
    request: RecordMemoryEntityMergeDecisionInput,
    state: tauri::State<'_, AppState>,
) -> Result<RecordMemoryEntityMergeDecisionOutput, String> {
    memory_entity_relation_governance::record_merge_decision(
        &state.workflow_state_path,
        &request,
        &unix_timestamp_string(),
        &format!("write-memory-entity-merge-{}", unix_timestamp_nanos()),
    )
}

#[tauri::command]
fn record_memory_relation_candidate_decision(
    request: RecordMemoryRelationCandidateDecisionInput,
    state: tauri::State<'_, AppState>,
) -> Result<RecordMemoryRelationCandidateDecisionOutput, String> {
    memory_entity_relation_governance::record_relation_decision(
        &state.workflow_state_path,
        &request,
        &unix_timestamp_string(),
        &format!("write-memory-relation-{}", unix_timestamp_nanos()),
    )
}

#[tauri::command]
fn run_memory_lint(
    request: MemoryLintRunInput,
    state: tauri::State<'_, AppState>,
) -> Result<MemoryLintRunOutput, String> {
    run_memory_lint_at(
        &state.workflow_state_path,
        &request,
        &unix_timestamp_string(),
        &format!("write-memory-lint-{}", unix_timestamp_nanos()),
    )
}

#[tauri::command]
fn load_memory_candidate_store(
    state: tauri::State<'_, AppState>,
) -> Result<MemoryCandidateStoreV1, String> {
    memory_candidate_store::load_store(&state.workflow_state_path, &unix_timestamp_string())
}

#[tauri::command]
fn create_memory_candidate(
    request: CreateMemoryCandidateInput,
    state: tauri::State<'_, AppState>,
) -> Result<CreateMemoryCandidateOutput, String> {
    memory_candidate_store::create_candidate(
        &state.workflow_state_path,
        &request,
        &unix_timestamp_string(),
        &format!("write-{}", unix_timestamp_nanos()),
    )
}

#[tauri::command]
fn record_memory_candidate_decision(
    request: RecordMemoryCandidateDecisionInput,
    state: tauri::State<'_, AppState>,
) -> Result<RecordMemoryCandidateDecisionOutput, String> {
    memory_candidate_store::record_decision(
        &state.workflow_state_path,
        &request,
        &unix_timestamp_string(),
        &format!("write-{}", unix_timestamp_nanos()),
    )
}

#[tauri::command]
fn adopt_memory_candidate_to_formal_memory(
    request: AdoptMemoryCandidateInput,
    state: tauri::State<'_, AppState>,
) -> Result<AdoptMemoryCandidateOutput, String> {
    let timestamp = unix_timestamp_string();
    adopt_memory_candidate_to_formal_memory_at(
        &state.workflow_state_path,
        &request,
        &timestamp,
        &format!("write-memory-candidate-adoption-{}", unix_timestamp_nanos()),
        &format!("write-formal-memory-adoption-{}", unix_timestamp_nanos()),
    )
}

#[tauri::command]
fn load_formal_memory_store(
    state: tauri::State<'_, AppState>,
) -> Result<FormalMemoryStoreV1, String> {
    formal_memory_store::load_store(&state.workflow_state_path, &unix_timestamp_string())
}

#[tauri::command]
fn create_formal_memory_record(
    request: CreateFormalMemoryRecordInput,
    state: tauri::State<'_, AppState>,
) -> Result<CreateFormalMemoryRecordOutput, String> {
    create_formal_memory_record_at(
        &state.workflow_state_path,
        &request,
        &unix_timestamp_string(),
        &format!("write-{}", unix_timestamp_nanos()),
    )
}

#[tauri::command]
fn preview_formal_memory_lifecycle_operation(
    request: FormalMemoryLifecyclePreviewInput,
    state: tauri::State<'_, AppState>,
) -> Result<FormalMemoryLifecyclePreview, String> {
    formal_memory_lifecycle::preview_operation(
        &state.workflow_state_path,
        &request,
        &unix_timestamp_string(),
    )
}

#[tauri::command]
fn record_formal_memory_lifecycle_operation(
    request: FormalMemoryLifecycleInput,
    state: tauri::State<'_, AppState>,
) -> Result<FormalMemoryLifecycleOutput, String> {
    formal_memory_lifecycle::record_operation(
        &state.workflow_state_path,
        &request,
        &unix_timestamp_string(),
        &format!("write-formal-memory-lifecycle-{}", unix_timestamp_nanos()),
    )
}

#[tauri::command]
fn initialize_workflow_state(
    state: tauri::State<'_, AppState>,
) -> Result<WorkflowStateMutationResult, String> {
    initialize_workflow_state_at(&state.workflow_state_path)
}

#[tauri::command]
fn bootstrap_project_workflow(
    request: PathRequest,
    state: tauri::State<'_, AppState>,
) -> Result<WorkflowStateMutationResult, String> {
    let index = read_index(&state)?;
    let project = find_index_project(&index, &request.path)
        .ok_or_else(|| "项目不在当前索引内，已拒绝创建本地工作流草稿".to_string())?;
    bootstrap_project_workflow_at(&state.workflow_state_path, &project)
}

#[tauri::command]
fn create_task_draft(
    request: TaskDraftRequest,
    state: tauri::State<'_, AppState>,
) -> Result<WorkflowStateMutationResult, String> {
    let index = read_index(&state)?;
    create_task_draft_for_index_project_at(&state.workflow_state_path, &index, &request)
}

fn create_task_draft_for_index_project_at(
    path: &Path,
    index: &Value,
    request: &TaskDraftRequest,
) -> Result<WorkflowStateMutationResult, String> {
    if find_index_project(&index, &request.project_root).is_none() {
        return Err("项目不在当前索引内，已拒绝登记任务包草稿".to_string());
    }
    create_task_draft_at(path, request)
}

#[tauri::command]
fn render_task_package_preview(
    request: TaskPackagePreviewRequest,
    state: tauri::State<'_, AppState>,
) -> Result<TaskPackagePreview, String> {
    let index = read_index(&state)?;
    render_task_package_preview_for_index_project_at(&state.workflow_state_path, &index, &request)
}

#[tauri::command]
fn copy_task_package_preview(
    request: TaskPackagePreviewRequest,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let index = read_index(&state)?;
    let preview = render_task_package_preview_for_index_project_at(
        &state.workflow_state_path,
        &index,
        &request,
    )?;
    copy_to_clipboard(&preview.markdown)?;
    Ok("已复制任务包 Markdown 预览文本；没有写入真实任务包文件。".to_string())
}

fn render_task_package_preview_for_index_project_at(
    path: &Path,
    index: &Value,
    request: &TaskPackagePreviewRequest,
) -> Result<TaskPackagePreview, String> {
    let project = find_index_project(index, &request.project_root)
        .ok_or_else(|| "项目不在当前索引内，已拒绝渲染任务包预览".to_string())?;
    render_task_package_preview_at(path, &project, request)
}

#[tauri::command]
fn update_task_package_draft_fields(
    request: TaskPackageFieldsUpdateRequest,
    state: tauri::State<'_, AppState>,
) -> Result<WorkflowStateMutationResult, String> {
    let index = read_index(&state)?;
    update_task_package_draft_fields_for_index_project_at(
        &state.workflow_state_path,
        &index,
        &request,
    )
}

fn update_task_package_draft_fields_for_index_project_at(
    path: &Path,
    index: &Value,
    request: &TaskPackageFieldsUpdateRequest,
) -> Result<WorkflowStateMutationResult, String> {
    if find_index_project(index, &request.project_root).is_none() {
        return Err("项目不在当前索引内，已拒绝更新任务包字段".to_string());
    }
    update_task_package_draft_fields_at(path, request)
}

#[tauri::command]
fn correct_task_package_dispatch_fields(
    request: TaskPackageDispatchFieldsCorrectionRequest,
    state: tauri::State<'_, AppState>,
) -> Result<WorkflowStateMutationResult, String> {
    let index = read_index(&state)?;
    correct_task_package_dispatch_fields_for_index_project_at(
        &state.workflow_state_path,
        &index,
        &request,
    )
}

fn correct_task_package_dispatch_fields_for_index_project_at(
    path: &Path,
    index: &Value,
    request: &TaskPackageDispatchFieldsCorrectionRequest,
) -> Result<WorkflowStateMutationResult, String> {
    if find_index_project(index, &request.project_root).is_none() {
        return Err("项目不在当前索引内，已拒绝修正派发字段".to_string());
    }
    let update_request = TaskPackageFieldsUpdateRequest {
        project_root: request.project_root.clone(),
        work_item_id: request.work_item_id.clone(),
        fields: request.fields.clone(),
    };
    update_task_package_fields_at(
        path,
        &update_request,
        TaskPackageFieldWriteMode::DispatchCorrection,
    )
}

#[tauri::command]
fn generate_task_package_file(
    request: TaskPackageFileGenerationRequest,
    state: tauri::State<'_, AppState>,
) -> Result<TaskPackageFileGenerationResult, String> {
    let index = read_index(&state)?;
    generate_task_package_file_for_index_project_at(
        &state.workflow_state_path,
        &index,
        &request,
        &default_task_package_output_dir(),
    )
}

fn generate_task_package_file_for_index_project_at(
    path: &Path,
    index: &Value,
    request: &TaskPackageFileGenerationRequest,
    tasks_dir: &Path,
) -> Result<TaskPackageFileGenerationResult, String> {
    let project = find_index_project(index, &request.project_root)
        .ok_or_else(|| "项目不在当前索引内，已拒绝生成真实任务包文件".to_string())?;
    generate_task_package_file_at(path, &project, request, tasks_dir)
}

#[tauri::command]
fn inspect_task_package_dispatch_readiness(
    request: TaskPackageDispatchReadinessRequest,
    state: tauri::State<'_, AppState>,
) -> Result<TaskPackageDispatchReadiness, String> {
    let index = read_index(&state)?;
    inspect_task_package_dispatch_readiness_for_index_project_at(
        &state.workflow_state_path,
        &index,
        &request,
    )
}

fn inspect_task_package_dispatch_readiness_for_index_project_at(
    path: &Path,
    index: &Value,
    request: &TaskPackageDispatchReadinessRequest,
) -> Result<TaskPackageDispatchReadiness, String> {
    let project = find_index_project(index, &request.project_root)
        .ok_or_else(|| "项目不在当前索引内，已拒绝检查任务包派发准备状态".to_string())?;
    inspect_task_package_dispatch_readiness_at(path, &project, request)
}

#[tauri::command]
fn inspect_workflow_run_check(
    request: WorkflowRunCheckRequest,
    state: tauri::State<'_, AppState>,
) -> Result<WorkflowRunCheck, String> {
    let index = read_index(&state)?;
    inspect_workflow_run_check_for_index_at(&state.workflow_state_path, &index, &request)
}

fn inspect_workflow_run_check_for_index_at(
    path: &Path,
    index: &Value,
    request: &WorkflowRunCheckRequest,
) -> Result<WorkflowRunCheck, String> {
    let project = find_index_project(index, &request.project_root)
        .ok_or_else(|| "项目不在当前索引内，已拒绝检查工作流运行性".to_string())?;
    inspect_workflow_run_check_at(path, &project, request)
}

#[tauri::command]
fn update_work_item_state(
    request: WorkItemStateUpdateRequest,
    state: tauri::State<'_, AppState>,
) -> Result<WorkflowStateMutationResult, String> {
    let index = read_index(&state)?;
    update_work_item_state_for_index_project_at(&state.workflow_state_path, &index, &request)
}

fn update_work_item_state_for_index_project_at(
    path: &Path,
    index: &Value,
    request: &WorkItemStateUpdateRequest,
) -> Result<WorkflowStateMutationResult, String> {
    if find_index_project(index, &request.project_root).is_none() {
        return Err("项目不在当前索引内，已拒绝推进工作项状态".to_string());
    }
    update_work_item_state_at(path, request)
}

#[tauri::command]
fn bind_workflow_node_codex_session(
    request: WorkflowNodeSessionBindRequest,
    state: tauri::State<'_, AppState>,
) -> Result<WorkflowStateMutationResult, String> {
    let index = read_index(&state)?;
    bind_workflow_node_codex_session_for_index_at(&state.workflow_state_path, &index, &request)
}

fn bind_workflow_node_codex_session_for_index_at(
    path: &Path,
    index: &Value,
    request: &WorkflowNodeSessionBindRequest,
) -> Result<WorkflowStateMutationResult, String> {
    if find_index_project(index, &request.project_root).is_none() {
        return Err("项目不在当前索引内，已拒绝绑定节点会话".to_string());
    }
    let session = find_index_thread(index, &request.thread_id)
        .ok_or_else(|| "会话不在当前索引内，已拒绝绑定节点会话".to_string())?;
    bind_workflow_node_codex_session_at(path, request, &session)
}

#[tauri::command]
fn unbind_workflow_node_codex_session(
    request: WorkflowNodeSessionUnbindRequest,
    state: tauri::State<'_, AppState>,
) -> Result<WorkflowStateMutationResult, String> {
    let index = read_index(&state)?;
    unbind_workflow_node_codex_session_for_index_at(&state.workflow_state_path, &index, &request)
}

fn unbind_workflow_node_codex_session_for_index_at(
    path: &Path,
    index: &Value,
    request: &WorkflowNodeSessionUnbindRequest,
) -> Result<WorkflowStateMutationResult, String> {
    if find_index_project(index, &request.project_root).is_none() {
        return Err("项目不在当前索引内，已拒绝解绑节点会话".to_string());
    }
    unbind_workflow_node_codex_session_at(path, request)
}

#[tauri::command]
fn prepare_workflow_node_dispatch(
    request: WorkflowNodeDispatchPrepareRequest,
    state: tauri::State<'_, AppState>,
) -> Result<WorkflowNodeDispatchResult, String> {
    let index = read_index(&state)?;
    prepare_workflow_node_dispatch_for_index_at(&state.workflow_state_path, &index, &request)
}

fn prepare_workflow_node_dispatch_for_index_at(
    path: &Path,
    index: &Value,
    request: &WorkflowNodeDispatchPrepareRequest,
) -> Result<WorkflowNodeDispatchResult, String> {
    if find_index_project(index, &request.project_root).is_none() {
        return Err("项目不在当前索引内，已拒绝准备节点派发".to_string());
    }
    prepare_workflow_node_dispatch_at(path, index, request)
}

#[tauri::command]
fn execute_workflow_node_dispatch(
    _request: WorkflowNodeDispatchExecuteRequest,
    state: tauri::State<'_, AppState>,
) -> Result<WorkflowNodeDispatchResult, String> {
    let _ = state;
    Err(legacy_product_command_blocked_message(
        "execute_workflow_node_dispatch",
    ))
}

fn execute_workflow_node_dispatch_for_index_at(
    path: &Path,
    index: &Value,
    readback_db_path: &Path,
    runner: &dyn CodexResumeRunner,
    request: &WorkflowNodeDispatchExecuteRequest,
) -> Result<WorkflowNodeDispatchResult, String> {
    if find_index_project(index, &request.project_root).is_none() {
        return Err("项目不在当前索引内，已拒绝执行节点派发".to_string());
    }
    execute_workflow_node_dispatch_at(path, index, readback_db_path, runner, request)
}

#[tauri::command]
fn read_workflow_node_dispatch_result(
    _request: WorkflowNodeDispatchReadbackRequest,
    state: tauri::State<'_, AppState>,
) -> Result<WorkflowNodeDispatchResult, String> {
    let _ = state;
    Err(legacy_product_command_blocked_message(
        "read_workflow_node_dispatch_result",
    ))
}

#[tauri::command]
fn record_workflow_dispatch_director_review(
    request: WorkflowDispatchDirectorReviewRequest,
    state: tauri::State<'_, AppState>,
) -> Result<WorkflowStateMutationResult, String> {
    let index = read_index(&state)?;
    record_workflow_dispatch_director_review_for_index_at(
        &state.workflow_state_path,
        &index,
        &request,
    )
}

fn record_workflow_dispatch_director_review_for_index_at(
    path: &Path,
    index: &Value,
    request: &WorkflowDispatchDirectorReviewRequest,
) -> Result<WorkflowStateMutationResult, String> {
    if find_index_project(index, &request.project_root).is_none() {
        return Err("项目不在当前索引内，已拒绝记录总指导回收意见".to_string());
    }
    record_workflow_dispatch_director_review_at(path, request)
}

#[tauri::command]
fn record_workflow_permission_decision(
    request: WorkflowPermissionDecisionRequest,
    state: tauri::State<'_, AppState>,
) -> Result<WorkflowStateMutationResult, String> {
    let index = read_index(&state)?;
    record_workflow_permission_decision_for_index_at(&state.workflow_state_path, &index, &request)
}

fn record_workflow_permission_decision_for_index_at(
    path: &Path,
    index: &Value,
    request: &WorkflowPermissionDecisionRequest,
) -> Result<WorkflowStateMutationResult, String> {
    if find_index_project(index, &request.project_root).is_none() {
        return Err("项目不在当前索引内，已拒绝记录权限结论".to_string());
    }
    record_workflow_permission_decision_at(path, request)
}

#[tauri::command]
fn prepare_offline_role_dispatch(
    request: OfflineRoleDispatchRequest,
    state: tauri::State<'_, AppState>,
) -> Result<WorkflowNodeDispatchResult, String> {
    let index = read_index(&state)?;
    prepare_offline_role_dispatch_for_index_at(&state.workflow_state_path, &index, &request)
}

fn prepare_offline_role_dispatch_for_index_at(
    path: &Path,
    index: &Value,
    request: &OfflineRoleDispatchRequest,
) -> Result<WorkflowNodeDispatchResult, String> {
    if find_index_project(index, &request.project_root).is_none() {
        return Err("项目不在当前索引内，已拒绝记录离线角色派发".to_string());
    }
    prepare_offline_role_dispatch_at(path, request)
}

#[tauri::command]
fn record_offline_role_result_handoff(
    request: OfflineRoleResultHandoffRequest,
    state: tauri::State<'_, AppState>,
) -> Result<WorkflowNodeDispatchResult, String> {
    let index = read_index(&state)?;
    record_offline_role_result_handoff_for_index_at(&state.workflow_state_path, &index, &request)
}

fn record_offline_role_result_handoff_for_index_at(
    path: &Path,
    index: &Value,
    request: &OfflineRoleResultHandoffRequest,
) -> Result<WorkflowNodeDispatchResult, String> {
    if find_index_project(index, &request.project_root).is_none() {
        return Err("项目不在当前索引内，已拒绝记录离线角色回传".to_string());
    }
    record_offline_role_result_handoff_at(path, request)
}

#[tauri::command]
fn record_offline_director_review(
    request: OfflineDirectorReviewRequest,
    state: tauri::State<'_, AppState>,
) -> Result<WorkflowStateMutationResult, String> {
    let index = read_index(&state)?;
    record_offline_director_review_for_index_at(&state.workflow_state_path, &index, &request)
}

fn record_offline_director_review_for_index_at(
    path: &Path,
    index: &Value,
    request: &OfflineDirectorReviewRequest,
) -> Result<WorkflowStateMutationResult, String> {
    if find_index_project(index, &request.project_root).is_none() {
        return Err("项目不在当前索引内，已拒绝记录离线总指导回收".to_string());
    }
    record_offline_director_review_at(path, request)
}

#[tauri::command]
fn run_workflow_machine(
    _request: WorkflowMachineRunRequest,
    state: tauri::State<'_, AppState>,
) -> Result<WorkflowMachineRunResult, String> {
    let _ = state;
    Err(legacy_product_command_blocked_message(
        "run_workflow_machine",
    ))
}

fn run_workflow_machine_for_index_at(
    path: &Path,
    index: &Value,
    readback_db_path: &Path,
    runner: &dyn CodexResumeRunner,
    request: &WorkflowMachineRunRequest,
) -> Result<WorkflowMachineRunResult, String> {
    if find_index_project(index, &request.project_root).is_none() {
        return Err("项目不在当前索引内，已拒绝运行工作流机器".to_string());
    }
    run_workflow_machine_at(path, index, readback_db_path, runner, request)
}

#[tauri::command]
fn copy_indexed_path(
    request: PathRequest,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let index = read_index(&state)?;
    let allowed = allowed_paths(&index);
    if !allowed.can_copy(&request.path) {
        return Err("路径不在索引白名单内，已拒绝复制".to_string());
    }
    copy_to_clipboard(&request.path)?;
    Ok(format!("已复制索引内路径：{}", request.path))
}

#[tauri::command]
fn open_indexed_project(
    request: PathRequest,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let index = read_index(&state)?;
    let allowed = allowed_paths(&index);
    if !allowed.projects.contains(&request.path) {
        return Err("路径不是索引内项目根目录，已拒绝打开".to_string());
    }
    let path = PathBuf::from(&request.path);
    if !path.is_dir() {
        return Err("索引项目路径当前不是可打开目录".to_string());
    }
    run_open(&[request.path.as_str()])?;
    Ok(format!("已请求打开项目目录：{}", request.path))
}

#[tauri::command]
fn reveal_indexed_rollout(
    request: PathRequest,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let index = read_index(&state)?;
    let mut allowed = allowed_paths(&index);
    extend_allowed_rollouts_from_sqlite(&mut allowed);
    if !allowed.rollouts.contains(&request.path) {
        return Err(
            "rollout_outside_allowed_dirs:路径不是允许的 rollout 文件，已拒绝定位".to_string(),
        );
    }
    let path = PathBuf::from(&request.path);
    if !path.is_file() {
        return Err("rollout_missing:允许的 rollout 路径当前不是文件".to_string());
    }
    run_open(&["-R", request.path.as_str()])?;
    Ok(format!("已请求定位 rollout 文件：{}", request.path))
}
