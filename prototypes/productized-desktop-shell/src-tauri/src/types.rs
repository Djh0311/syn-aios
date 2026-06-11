// Backend data contracts split out during Task B conservative module split.
// This file is included at crate root to preserve visibility and behavior.

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct FileCandidate {
    kind: Option<String>,
    name: Option<String>,
    path: String,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct HarnessCandidate {
    entry_type: Option<String>,
    name: Option<String>,
    path: String,
    source: Option<String>,
    size_bytes: Option<i64>,
    updated_at_ms: Option<i64>,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct HarnessEntrypoint {
    entry_type: Option<String>,
    name: Option<String>,
    path: String,
    source_kind: Option<String>,
    size_bytes: Option<i64>,
    updated_at_ms: Option<i64>,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct HarnessResource {
    root_path: String,
    display_name: Option<String>,
    harness_kind: Option<String>,
    agent_type: Option<String>,
    adapter_id: Option<String>,
    source_kind: Option<String>,
    capabilities: Vec<String>,
    manifest_path: Option<String>,
    readme_path: Option<String>,
    version: Option<String>,
    entrypoints: Vec<HarnessEntrypoint>,
    permission_level: Option<String>,
    size_bytes: Option<i64>,
    updated_at_ms: Option<i64>,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct ProjectRecord {
    project_root: String,
    name: String,
    active_hint: bool,
    thread_count: usize,
    active_thread_count: usize,
    archived_thread_count: usize,
    latest_updated_at_ms: Option<i64>,
    authority_files: Vec<FileCandidate>,
    handoff_files: Vec<FileCandidate>,
    evidence_files: Vec<FileCandidate>,
    harness_candidates: Vec<HarnessCandidate>,
    harness_resources: Vec<HarnessResource>,
    context_warnings: Vec<String>,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct SessionRecord {
    thread_id: String,
    title: String,
    project_root: Option<String>,
    updated_at_ms: Option<i64>,
    archived: bool,
    rollout_exists: bool,
    rollout_path: Option<String>,
    model: Option<String>,
    reasoning_effort: Option<String>,
    thread_source: Option<String>,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
struct CodexTranscript {
    thread_id: String,
    rollout_path: String,
    project_path: Option<String>,
    title: Option<String>,
    created_at_ms: Option<i64>,
    updated_at_ms: Option<i64>,
    viewer_boundary: CodexTranscriptViewerBoundary,
    events: Vec<CodexTranscriptEvent>,
    summary: CodexTranscriptSummary,
    warnings: Vec<String>,
    source_stats: Value,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct CodexTranscriptViewerBoundary {
    view_kind: String,
    reads_session_history: bool,
    is_execution_readback: bool,
    real_execution_readback_performed: bool,
    execution_readback_scope: String,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
struct CodexTranscriptEvent {
    event_id: String,
    timestamp: Option<String>,
    event_type: Option<String>,
    actor: Option<String>,
    role: Option<String>,
    turn_id: Option<String>,
    call_id: Option<String>,
    tool_name: Option<String>,
    text: Option<String>,
    arguments: Value,
    output: Value,
    stdout: Option<String>,
    stderr: Option<String>,
    exit_code: Value,
    metadata: Value,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct CodexTranscriptSummary {
    total_events: usize,
    event_type_counts: BTreeMap<String, usize>,
    unknown_event_count: usize,
    warning_count: usize,
    encrypted_content_event_count: usize,
    sensitive_like_event_count: usize,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct SkillRecord {
    skill_id: String,
    title: String,
    description: Option<String>,
    path: String,
    source_type: String,
    plugin_name: Option<String>,
    plugin_version: Option<String>,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct PluginRecord {
    plugin_name: String,
    plugin_version: String,
    homepage: Option<String>,
    skill_count: usize,
    has_apps: bool,
    has_mcp_servers: bool,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct TaskEntry {
    status: String,
    title: String,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct Diagnostics {
    index_path: String,
    tasks_path: String,
    generated_at: Option<String>,
    top_level_warning_count: usize,
    context_warning_count: usize,
    allowed_project_path_count: usize,
    allowed_rollout_path_count: usize,
    release_bundle_enabled: bool,
    notes: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct IndexSummary {
    generated_at: Option<String>,
    project_count: usize,
    session_count: usize,
    skill_count: usize,
    plugin_count: usize,
    task_count: usize,
    warning_count: usize,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct AdapterCapability {
    capability_id: String,
    kind: String,
    label: String,
    status: String,
    description: String,
    boundary: String,
    evidence_refs: Vec<String>,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct AgentAdapterDescriptor {
    adapter_id: String,
    agent_type: String,
    agent_id: String,
    display_name: String,
    provider: String,
    status: String,
    permission_level: String,
    source_kind: String,
    capabilities: Vec<AdapterCapability>,
    implemented_action_kinds: Vec<String>,
    hidden_unimplemented_adapters: Vec<String>,
    warnings: Vec<String>,
    execution_status: String,
    credential_status: String,
    model_access_status: String,
    permission_boundary: String,
    unavailable_reason: Option<String>,
    requires_user_setup: bool,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct SessionOperationDescriptor {
    operation_id: String,
    label: String,
    category: String,
    current_status: String,
    risk_level: String,
    adapter_id: String,
    agent_type: String,
    applies_to_session_state: String,
    requires_user_confirmation: bool,
    writes_codex_home: bool,
    writes_workbench_state: bool,
    writes_project_files: bool,
    reads_full_transcript: bool,
    requires_credential: bool,
    requires_model_access: bool,
    requires_runtime_handle: bool,
    audit_requirement: String,
    unavailable_reason: String,
    future_task_hint: String,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct ProviderAvailabilitySummary {
    adapter_id: String,
    provider_id: String,
    provider_label: String,
    provider_kind: String,
    adapter_status: String,
    availability_status: String,
    credential_status: String,
    model_status: String,
    external_call_status: String,
    cost_risk_status: String,
    user_visible_reason: String,
    safe_to_display: bool,
    requires_user_configuration: bool,
    requires_future_task: bool,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct WorkerProtocolSourceRef {
    source_kind: String,
    source_id: String,
    label: String,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct WorkerCapabilityDescriptor {
    capability_id: String,
    capability_kind: String,
    label: String,
    status: String,
    risk_level: String,
    execution_boundary: String,
    provider_id: Option<String>,
    credential_requirement_id: Option<String>,
    risk_envelope_id: Option<String>,
    project_policy_status: String,
    source_refs: Vec<WorkerProtocolSourceRef>,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct WorkerAdapterProtocolDescriptor {
    worker_adapter_id: String,
    adapter_id: String,
    worker_kind: String,
    display_name: String,
    provider_id: String,
    lifecycle_status: String,
    execution_status: String,
    credential_status: String,
    model_status: String,
    source_policy: String,
    capability_descriptors: Vec<WorkerCapabilityDescriptor>,
    source_refs: Vec<WorkerProtocolSourceRef>,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct RunPersistenceHandle {
    handle_id: String,
    adapter_id: String,
    native_thread_id: Option<String>,
    project_id: Option<String>,
    workflow_id: Option<String>,
    node_id: Option<String>,
    work_item_id: Option<String>,
    persistence_kind: String,
    read_policy: String,
    write_policy: String,
    source_refs: Vec<WorkerProtocolSourceRef>,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct WorkThread {
    work_thread_id: String,
    adapter_id: String,
    lifecycle_status: String,
    project_id: Option<String>,
    workflow_id: Option<String>,
    node_id: Option<String>,
    work_item_id: Option<String>,
    run_persistence_handle: Option<RunPersistenceHandle>,
    source_refs: Vec<WorkerProtocolSourceRef>,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct RunAttention {
    attention_id: String,
    kind: String,
    severity: String,
    status: String,
    requires_user_action: bool,
    blocks_continuation: bool,
    readback_status: String,
    result_count: Option<i64>,
    source_refs: Vec<WorkerProtocolSourceRef>,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct RunUnit {
    run_unit_id: String,
    adapter_id: String,
    work_thread_id: Option<String>,
    project_id: Option<String>,
    workflow_id: Option<String>,
    node_id: Option<String>,
    work_item_id: Option<String>,
    lifecycle_status: String,
    operation_id: String,
    prompt_sent: bool,
    real_worker_executed: bool,
    writes_adapter_home: bool,
    writes_project_files: bool,
    writes_workbench_state: bool,
    attention: Vec<RunAttention>,
    source_refs: Vec<WorkerProtocolSourceRef>,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct CredentialRequirementDescriptor {
    requirement_id: String,
    adapter_id: String,
    provider_id: String,
    credential_status: String,
    required_for_real_execution: bool,
    read_policy: String,
    verification_status: String,
    user_action_required: bool,
    source_refs: Vec<WorkerProtocolSourceRef>,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct ExternalCallRiskEnvelope {
    envelope_id: String,
    adapter_id: String,
    provider_id: String,
    capability_kind: String,
    external_call_status: String,
    data_egress_risk: String,
    cost_risk: String,
    credential_risk: String,
    model_risk: String,
    project_policy_status: String,
    user_visible_summary: String,
    source_refs: Vec<WorkerProtocolSourceRef>,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct ProjectCapabilityPolicy {
    policy_id: String,
    project_id: Option<String>,
    workflow_id: Option<String>,
    policy_status: String,
    allowed_capability_kinds: Vec<String>,
    blocked_capability_kinds: Vec<String>,
    requires_user_confirmation: Vec<String>,
    source_refs: Vec<WorkerProtocolSourceRef>,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct RunRelation {
    relation_id: String,
    relation_kind: String,
    parent_run_unit_id: Option<String>,
    child_run_unit_id: String,
    project_id: Option<String>,
    workflow_id: Option<String>,
    source_refs: Vec<WorkerProtocolSourceRef>,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct WorkerLane {
    lane_id: String,
    lane_kind: String,
    project_id: Option<String>,
    workflow_id: Option<String>,
    run_unit_ids: Vec<String>,
    work_thread_ids: Vec<String>,
    status: String,
    reviewer_required: bool,
    source_refs: Vec<WorkerProtocolSourceRef>,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct MultiWorkerDispatchPlan {
    plan_id: String,
    project_id: Option<String>,
    workflow_id: Option<String>,
    status: String,
    dispatch_request_ids: Vec<String>,
    run_unit_ids: Vec<String>,
    lane_ids: Vec<String>,
    relation_ids: Vec<String>,
    verifier_lane_required: bool,
    recovery_lane_available: bool,
    source_policy: String,
    source_refs: Vec<WorkerProtocolSourceRef>,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct AdapterContractChecklist {
    checklist_id: String,
    adapter_id: String,
    status: String,
    protocol_surface_ready: bool,
    control_core_required: bool,
    permission_required: bool,
    audit_required: bool,
    runtime_log_required: bool,
    credential_boundary_defined: bool,
    model_boundary_defined: bool,
    data_location_defined: bool,
    missing_items: Vec<String>,
    source_refs: Vec<WorkerProtocolSourceRef>,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct ControlledApiCliSemantics {
    semantics_id: String,
    adapter_id: String,
    cli_surface: String,
    api_surface: String,
    parity_status: String,
    control_core_path: String,
    permission_path: String,
    audit_path: String,
    universal_api_backdoor_blocked: bool,
    supported_operation_ids: Vec<String>,
    source_refs: Vec<WorkerProtocolSourceRef>,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct DiagnosticEventSchemaDescriptor {
    schema_id: String,
    adapter_id: String,
    event_kinds: Vec<String>,
    severity_levels: Vec<String>,
    required_fields: Vec<String>,
    redaction_policy: String,
    export_policy: String,
    source_refs: Vec<WorkerProtocolSourceRef>,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct AdapterHealthSummary {
    health_id: String,
    adapter_id: String,
    status: String,
    severity: String,
    credential_status: String,
    model_status: String,
    runtime_status: String,
    degraded_reason: Option<String>,
    source_refs: Vec<WorkerProtocolSourceRef>,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct AdapterDegradedMode {
    degraded_mode_id: String,
    adapter_id: String,
    mode: String,
    blocks_real_execution: bool,
    user_visible_summary: String,
    allowed_surfaces: Vec<String>,
    blocked_surfaces: Vec<String>,
    recovery_requirement: String,
    source_refs: Vec<WorkerProtocolSourceRef>,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct AdapterDataLocationDescriptor {
    data_location_id: String,
    adapter_id: String,
    persistence_kind: String,
    workbench_store_refs: Vec<String>,
    adapter_home_policy: String,
    project_write_policy: String,
    transcript_policy: String,
    secret_policy: String,
    source_refs: Vec<WorkerProtocolSourceRef>,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct DispatchRequest {
    dispatch_request_id: String,
    adapter_id: String,
    operation_id: String,
    project_id: Option<String>,
    workflow_id: Option<String>,
    node_id: Option<String>,
    work_item_id: Option<String>,
    target_session_id: Option<String>,
    requested_by: String,
    prompt_source_kind: String,
    prompt_summary: String,
    source_refs: Vec<WorkerProtocolSourceRef>,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct DispatchGuardResult {
    dispatch_request_id: String,
    status: String,
    severity: String,
    blocks_execution: bool,
    requires_user_confirmation: bool,
    reasons: Vec<String>,
    required_fixes: Vec<String>,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct PermissionEnvelope {
    envelope_id: String,
    adapter_id: String,
    operation_id: String,
    status: String,
    explicit_approval_required: bool,
    approved_for_real_execution: bool,
    cwd: Option<String>,
    allowed_write_roots: Vec<String>,
    denied_paths: Vec<String>,
    prompt_summary: String,
    risk_summary: String,
    source_refs: Vec<WorkerProtocolSourceRef>,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct TaskMemoryPacketRef {
    ref_id: String,
    snapshot_id: Option<String>,
    fingerprint: Option<String>,
    included_count: usize,
    excluded_count: usize,
    review_material_count: usize,
    stale: bool,
    source_refs: Vec<WorkerProtocolSourceRef>,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct ReadbackResult {
    readback_id: String,
    status: String,
    attempted: bool,
    real_readback_performed: bool,
    result_count: Option<i64>,
    confidence: String,
    source_refs: Vec<WorkerProtocolSourceRef>,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct WorkerReportCandidate {
    candidate_id: String,
    adapter_id: String,
    project_id: Option<String>,
    workflow_id: Option<String>,
    node_id: Option<String>,
    work_item_id: Option<String>,
    status: String,
    summary: String,
    source_policy: String,
    source_refs: Vec<WorkerProtocolSourceRef>,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct WorkerHandoff {
    handoff_id: String,
    adapter_id: String,
    project_id: Option<String>,
    workflow_id: Option<String>,
    node_id: Option<String>,
    work_item_id: Option<String>,
    handoff_status: String,
    summary: String,
    report_candidate: Option<WorkerReportCandidate>,
    readback_result: Option<ReadbackResult>,
    source_refs: Vec<WorkerProtocolSourceRef>,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct WorkerProtocolReadModel {
    schema_version: String,
    generated_at: String,
    source_policy: String,
    worker_adapters: Vec<WorkerAdapterProtocolDescriptor>,
    work_threads: Vec<WorkThread>,
    run_units: Vec<RunUnit>,
    credential_requirements: Vec<CredentialRequirementDescriptor>,
    external_call_risk_envelopes: Vec<ExternalCallRiskEnvelope>,
    project_capability_policies: Vec<ProjectCapabilityPolicy>,
    run_relations: Vec<RunRelation>,
    worker_lanes: Vec<WorkerLane>,
    multi_worker_dispatch_plans: Vec<MultiWorkerDispatchPlan>,
    adapter_contract_checklists: Vec<AdapterContractChecklist>,
    controlled_api_cli_semantics: Vec<ControlledApiCliSemantics>,
    diagnostic_event_schemas: Vec<DiagnosticEventSchemaDescriptor>,
    adapter_health_summaries: Vec<AdapterHealthSummary>,
    adapter_degraded_modes: Vec<AdapterDegradedMode>,
    adapter_data_locations: Vec<AdapterDataLocationDescriptor>,
    dispatch_requests: Vec<DispatchRequest>,
    dispatch_guards: Vec<DispatchGuardResult>,
    permission_envelopes: Vec<PermissionEnvelope>,
    task_memory_packet_refs: Vec<TaskMemoryPacketRef>,
    worker_handoffs: Vec<WorkerHandoff>,
    readback_results: Vec<ReadbackResult>,
    worker_report_candidates: Vec<WorkerReportCandidate>,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct SessionContinuationRequest {
    adapter_id: String,
    operation_id: String,
    project_id: Option<String>,
    project_root: Option<String>,
    workflow_id: Option<String>,
    node_id: Option<String>,
    session_id: Option<String>,
    #[serde(default)]
    work_item_id: Option<String>,
    target_cwd: Option<String>,
    allowed_write_roots: Vec<String>,
    sandbox: String,
    prompt_source_kind: String,
    prompt_summary: String,
    readback_strategy: String,
    requested_by: String,
    user_confirmation_state: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct ReadbackExpectation {
    strategy: String,
    required: bool,
    expected_sources: Vec<String>,
    unavailable_behavior: String,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct ContinuationFailureBoundary {
    timeout_policy: String,
    retry_policy: String,
    failure_record: String,
    user_visible_behavior: String,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct ContinuationAuditImpact {
    impact_kind: String,
    writes_attempt_in_e4: bool,
    writes_dispatch_in_e4: bool,
    writes_readback_in_e4: bool,
    future_audit_requirement: String,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct SessionContinuationGuardResult {
    status: String,
    severity: String,
    blocks_execution: bool,
    allows_preview: bool,
    requires_user_confirmation: bool,
    reasons: Vec<String>,
    required_fixes: Vec<String>,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct SessionContinuationPreview {
    preview_id: String,
    adapter_id: String,
    operation_id: String,
    target_session_id: Option<String>,
    target_session_title: Option<String>,
    project_id: Option<String>,
    project_root: Option<String>,
    workflow_id: Option<String>,
    node_id: Option<String>,
    binding_id: Option<String>,
    work_item_id: Option<String>,
    target_cwd: Option<String>,
    allowed_write_roots_summary: Vec<String>,
    sandbox_summary: String,
    prompt_source_kind: String,
    prompt_summary: String,
    readback_expectation: ReadbackExpectation,
    failure_handling: ContinuationFailureBoundary,
    audit_impact: ContinuationAuditImpact,
    provider_availability_summary: Option<ProviderAvailabilitySummary>,
    guard_result: SessionContinuationGuardResult,
    request: SessionContinuationRequest,
    user_visible_warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct SessionContinuationStoreScope {
    scope_kind: String,
    workflow_state_path: Option<String>,
    sidecar_path: Option<String>,
    project_roots: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct SessionContinuationStoreV1 {
    schema_version: String,
    store_version: i64,
    storage_kind: String,
    scope: SessionContinuationStoreScope,
    revision: i64,
    last_write_id: Option<String>,
    generated_by: String,
    created_at: String,
    updated_at: String,
    continuations: Vec<ControlledSessionContinuation>,
    attempts: Vec<SessionContinuationAttempt>,
    audit_events: Vec<SessionContinuationAuditEvent>,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct ControlledSessionContinuation {
    record_version: i64,
    continuation_id: String,
    preview_id: String,
    adapter_id: String,
    operation_id: String,
    project_id: String,
    project_root: String,
    workflow_id: String,
    node_id: String,
    session_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    work_item_id: Option<String>,
    target_cwd: String,
    allowed_write_roots: Vec<String>,
    sandbox: String,
    prompt_source_kind: String,
    prompt_summary: String,
    command_preview: String,
    readback_strategy: String,
    status: String,
    execution_level: String,
    runner_kind: String,
    user_confirmation_state: String,
    guard_status: String,
    requested_by: String,
    confirmed_by: String,
    confirmation_reason: String,
    created_at: String,
    updated_at: String,
    audit_refs: Vec<String>,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct SessionContinuationReadbackSummary {
    status: String,
    source_kind: String,
    result_count: Option<i64>,
    unavailable_reason: Option<String>,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct SessionContinuationAttempt {
    attempt_version: i64,
    attempt_id: String,
    continuation_id: String,
    runner_kind: String,
    execution_level: String,
    status: String,
    started_at: String,
    finished_at: Option<String>,
    timeout_ms: Option<i64>,
    command_preview: String,
    prompt_sent: bool,
    real_codex_executed: bool,
    writes_codex_home: bool,
    writes_workbench_state: bool,
    readback_summary: SessionContinuationReadbackSummary,
    failure_reason: Option<String>,
    audit_refs: Vec<String>,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct SessionContinuationAuditEvent {
    event_version: i64,
    event_id: String,
    event_type: String,
    continuation_id: String,
    attempt_id: Option<String>,
    preview_id: String,
    actor_role: String,
    before_status: Option<String>,
    after_status: String,
    store_revision: i64,
    reason: String,
    created_at: String,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct ConfirmControlledSessionContinuationInput {
    preview: SessionContinuationPreview,
    confirmed_by: String,
    confirmation_reason: String,
    expected_store_revision: Option<i64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct ConfirmControlledSessionContinuationOutput {
    continuation: ControlledSessionContinuation,
    audit_event: SessionContinuationAuditEvent,
    store_revision: i64,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct RunControlledSessionContinuationStubInput {
    continuation_id: String,
    actor_role: String,
    expected_store_revision: Option<i64>,
    timeout_ms: Option<i64>,
    force_stub_failure: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct RunControlledSessionContinuationStubOutput {
    continuation: ControlledSessionContinuation,
    attempt: SessionContinuationAttempt,
    audit_events: Vec<SessionContinuationAuditEvent>,
    store_revision: i64,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct H2RealResumeAuthorizationMatrix {
    operation_type: String,
    test_project: String,
    project_root: String,
    target_cwd: String,
    target_session: String,
    prompt_summary: String,
    prompt_sha256: String,
    prompt_ref: String,
    allowed_write_roots: Vec<String>,
    codex_home_scope: String,
    sandbox: String,
    timeout_ms: Option<i64>,
    readback_plan: String,
    evidence_path: String,
    rollback_plan: String,
    user_confirmed_real_resume: bool,
    global_supervisor_confirmed: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct InspectControlledSessionContinuationRealResumeInput {
    continuation_id: String,
    actor_role: String,
    expected_store_revision: Option<i64>,
    authorization: H2RealResumeAuthorizationMatrix,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct InspectControlledSessionContinuationRealResumeOutput {
    continuation: ControlledSessionContinuation,
    attempt: SessionContinuationAttempt,
    audit_event: SessionContinuationAuditEvent,
    store_revision: i64,
    authorization_status: String,
    missing_or_invalid_items: Vec<String>,
    codex_local_request: Option<CodexLocalExecutionRequest>,
    codex_local_guard: Option<CodexLocalExecutionGuard>,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct RunControlledSessionContinuationRealResumePhaseAInput {
    continuation_id: String,
    actor_role: String,
    expected_store_revision: Option<i64>,
    authorization: H2RealResumeAuthorizationMatrix,
    execution_decision: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct RunControlledSessionContinuationRealResumePhaseAOutput {
    continuation: ControlledSessionContinuation,
    attempt: SessionContinuationAttempt,
    audit_events: Vec<SessionContinuationAuditEvent>,
    store_revision: i64,
    authorization_status: String,
    missing_or_invalid_items: Vec<String>,
    codex_local_request: Option<CodexLocalExecutionRequest>,
    codex_local_guard: Option<CodexLocalExecutionGuard>,
    codex_local_attempt: Option<CodexLocalExecutionAttempt>,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct RunControlledSessionContinuationRealResumePhaseBInput {
    continuation_id: String,
    actor_role: String,
    expected_store_revision: Option<i64>,
    authorization: H2RealResumeAuthorizationMatrix,
    execution_decision: Option<String>,
    prompt_body: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct RunControlledSessionContinuationRealResumePhaseBOutput {
    continuation: ControlledSessionContinuation,
    attempt: SessionContinuationAttempt,
    audit_events: Vec<SessionContinuationAuditEvent>,
    store_revision: i64,
    authorization_status: String,
    missing_or_invalid_items: Vec<String>,
    codex_local_request: Option<CodexLocalExecutionRequest>,
    codex_local_guard: Option<CodexLocalExecutionGuard>,
    codex_local_attempt: Option<CodexLocalExecutionAttempt>,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct H3RealNewSessionAuthorizationMatrix {
    operation_type: String,
    test_project: String,
    project_root: String,
    target_cwd: String,
    work_item_id: String,
    prompt_summary: String,
    prompt_sha256: String,
    prompt_ref: String,
    allowed_write_roots: Vec<String>,
    codex_home_scope: String,
    sandbox: String,
    timeout_ms: Option<i64>,
    readback_plan: String,
    evidence_path: String,
    rollback_plan: String,
    user_confirmed_real_new_session: bool,
    global_supervisor_confirmed: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct RunControlledSessionContinuationRealNewSessionH3BInput {
    continuation_id: String,
    actor_role: String,
    expected_store_revision: Option<i64>,
    authorization: H3RealNewSessionAuthorizationMatrix,
    execution_decision: Option<String>,
    prompt_body: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct RunControlledSessionContinuationRealNewSessionH3BOutput {
    continuation: ControlledSessionContinuation,
    attempt: SessionContinuationAttempt,
    audit_events: Vec<SessionContinuationAuditEvent>,
    store_revision: i64,
    authorization_status: String,
    missing_or_invalid_items: Vec<String>,
    codex_local_request: Option<CodexLocalExecutionRequest>,
    codex_local_guard: Option<CodexLocalExecutionGuard>,
    codex_local_attempt: Option<CodexLocalExecutionAttempt>,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct CleanupSessionContinuationStaleAttemptInput {
    attempt_id: String,
    actor_role: String,
    expected_store_revision: Option<i64>,
    stale_reason: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct CleanupSessionContinuationStaleAttemptOutput {
    continuation: ControlledSessionContinuation,
    attempt: SessionContinuationAttempt,
    audit_event: SessionContinuationAuditEvent,
    store_revision: i64,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct CodexLocalRuntimeLogRef {
    ref_id: String,
    category: String,
    status: String,
    redaction_status: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct CodexLocalAuditRef {
    ref_id: String,
    event_type: String,
    actor_role: String,
    decision: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct CodexLocalReadbackPlan {
    strategy: String,
    required: bool,
    expected_sources: Vec<String>,
    unavailable_behavior: String,
    trust_policy: String,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct CodexLocalReadbackResult {
    status: String,
    attempted: bool,
    real_readback_performed: bool,
    result_count: Option<i64>,
    confidence: String,
    unavailable_reason: Option<String>,
    source_refs: Vec<String>,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct CodexLocalFailureReason {
    code: String,
    message: String,
    retryable: bool,
    user_action_required: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct CodexLocalCommandPlan {
    program: String,
    argv: Vec<String>,
    stdin_prompt_ref: String,
    stdin_prompt_sha256: String,
    prompt_in_command: bool,
    shell_invocation: bool,
    redacted_preview: String,
    sensitive_omissions: Vec<String>,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct CodexLocalActiveAttempt {
    attempt_id: String,
    status: String,
    continuation_id: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct CodexLocalExecutionRequest {
    request_version: i64,
    adapter_id: String,
    operation_id: String,
    project_id: String,
    project_root: String,
    workflow_id: String,
    node_id: String,
    session_id: Option<String>,
    work_item_id: Option<String>,
    continuation_id: Option<String>,
    target_cwd: String,
    allowed_write_roots: Vec<String>,
    sandbox: String,
    prompt_source_kind: String,
    prompt_summary: String,
    prompt_sha256: String,
    prompt_ref: String,
    readback_plan: CodexLocalReadbackPlan,
    requested_by: String,
    user_confirmation_state: String,
    authorization_scope_id: Option<String>,
    runtime_log_refs: Vec<CodexLocalRuntimeLogRef>,
    audit_refs: Vec<CodexLocalAuditRef>,
    active_attempts: Vec<CodexLocalActiveAttempt>,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct CodexLocalExecutionGuard {
    guard_version: i64,
    status: String,
    severity: String,
    blocks_execution: bool,
    allows_dry_run: bool,
    requires_user_confirmation: bool,
    duplicate_running_attempt: bool,
    command_plan: Option<CodexLocalCommandPlan>,
    reasons: Vec<String>,
    required_fixes: Vec<String>,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct CodexLocalExecutionAttempt {
    attempt_version: i64,
    attempt_id: String,
    request_id: String,
    runner_kind: String,
    execution_level: String,
    status: String,
    started_at: String,
    finished_at: Option<String>,
    request: CodexLocalExecutionRequest,
    guard: CodexLocalExecutionGuard,
    command_plan: Option<CodexLocalCommandPlan>,
    prompt_sent: bool,
    real_codex_executed: bool,
    writes_codex_home: bool,
    writes_project_files: bool,
    writes_workbench_state: bool,
    runtime_log_ref: Option<CodexLocalRuntimeLogRef>,
    audit_ref: Option<CodexLocalAuditRef>,
    readback_result: CodexLocalReadbackResult,
    failure_reason: Option<CodexLocalFailureReason>,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct RuntimeAttentionSourceRef {
    source_kind: String,
    source_id: String,
    label: String,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct ReadbackBoundaryStatus {
    status: String,
    reason: String,
    attempted: bool,
    real_readback_performed: bool,
    result_count: Option<i64>,
    user_message: String,
    technical_summary: String,
    source_refs: Vec<RuntimeAttentionSourceRef>,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct RuntimeSessionAttention {
    attention_id: String,
    project_id: Option<String>,
    workflow_id: Option<String>,
    node_id: Option<String>,
    session_id: Option<String>,
    adapter_id: String,
    source_refs: Vec<RuntimeAttentionSourceRef>,
    kind: String,
    severity: String,
    status: String,
    title: String,
    user_message: String,
    technical_summary: String,
    recommended_next_step: String,
    requires_user_action: bool,
    blocks_continuation: bool,
    readback_boundary: ReadbackBoundaryStatus,
    created_at: String,
    updated_at: String,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct SessionRunStatusSummary {
    session_id: String,
    adapter_id: String,
    project_id: Option<String>,
    workflow_id: Option<String>,
    node_id: Option<String>,
    current_status: String,
    current_status_label: String,
    attention_count: usize,
    blocking_count: usize,
    needs_user_count: usize,
    readback_status: String,
    latest_attention_ids: Vec<String>,
    source_refs: Vec<RuntimeAttentionSourceRef>,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct RuntimeLogStoreScope {
    scope_kind: String,
    workflow_state_path: Option<String>,
    sidecar_path: Option<String>,
    project_roots: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct RuntimeLogBoundary {
    runtime_log_definition: String,
    audit_event_definition: String,
    separation_rule: String,
    redaction_rule: String,
    forbidden_payloads: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct RuntimeLogSourceRef {
    source_kind: String,
    source_id: String,
    label: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct RuntimeLogEntry {
    entry_version: i64,
    entry_id: String,
    category: String,
    status: String,
    severity: String,
    started_at: Option<String>,
    finished_at: Option<String>,
    duration_ms: Option<i64>,
    project_id: Option<String>,
    workflow_id: Option<String>,
    node_id: Option<String>,
    session_id: Option<String>,
    adapter_id: Option<String>,
    summary: String,
    detail: String,
    source_refs: Vec<RuntimeLogSourceRef>,
    audit_refs: Vec<String>,
    redaction_status: String,
    sensitive_omissions: Vec<String>,
    user_visible: bool,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct RuntimeLogSummary {
    category: String,
    status: String,
    severity: String,
    entry_count: usize,
    latest_entry_ids: Vec<String>,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct RuntimeLogStoreV1 {
    schema_version: String,
    store_version: i64,
    storage_kind: String,
    scope: RuntimeLogStoreScope,
    revision: i64,
    last_write_id: Option<String>,
    generated_by: String,
    created_at: String,
    updated_at: String,
    boundary: RuntimeLogBoundary,
    entries: Vec<RuntimeLogEntry>,
    summaries: Vec<RuntimeLogSummary>,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct StoreIntegrityFinding {
    store_id: String,
    label: String,
    status: String,
    severity: String,
    path: Option<String>,
    schema_version: Option<String>,
    revision: Option<i64>,
    item_count: usize,
    warning_count: usize,
    error: Option<String>,
    summary: String,
    boundary: String,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct ServiceDegradedState {
    state_id: String,
    kind: String,
    severity: String,
    title: String,
    summary: String,
    user_action_required: bool,
    blocks_real_execution: bool,
    source_refs: Vec<String>,
    recommended_next_step: String,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct DiagnosticSummary {
    status: String,
    generated_at: String,
    overall_severity: String,
    healthy_count: usize,
    warning_count: usize,
    degraded_count: usize,
    blocked_count: usize,
    store_integrity: Vec<StoreIntegrityFinding>,
    degraded_states: Vec<ServiceDegradedState>,
    recent_error_summaries: Vec<String>,
    boundary_notes: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct RealExecutionProductCommandRequest {
    product_command_id: String,
    command_family: String,
    operation_id: String,
    project_id: Option<String>,
    project_root: Option<String>,
    workflow_id: Option<String>,
    node_id: Option<String>,
    work_item_id: Option<String>,
    task_package_ref: Option<String>,
    memory_packet_ref: Option<String>,
    adapter_id: String,
    session_mode: String,
    target_session_id: Option<String>,
    #[serde(default)]
    sandbox: String,
    prompt_summary: String,
    prompt_ref: String,
    prompt_hash: String,
    allowed_write_roots: Vec<String>,
    denied_paths: Vec<String>,
    readback_plan: String,
    timeout_ms: Option<i64>,
    requested_by: String,
    created_at: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct RealExecutionProductCommandPermissionEnvelope {
    envelope_id: String,
    product_command_id: String,
    status: String,
    explicit_user_confirmation_required: bool,
    approved_for_real_execution: bool,
    confirmed_by: Option<String>,
    allowed_write_roots: Vec<String>,
    denied_paths: Vec<String>,
    risk_summary: String,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct RealExecutionProductCommandReadiness {
    status: String,
    runner_call_allowed: bool,
    level_b_authorization_required: bool,
    blocked_reasons: Vec<String>,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct RealExecutionProductCommandGuardPreview {
    status: String,
    runner_call_allowed: bool,
    blocks_execution: bool,
    reasons: Vec<String>,
    required_fixes: Vec<String>,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct RealExecutionProductCommandDiagnosticsSummary {
    status: String,
    blocks_real_execution: bool,
    degraded_reasons: Vec<String>,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct RealExecutionProductCommandDuplicateScope {
    scope_id: String,
    active_attempt_count: usize,
    duplicate_blocked: bool,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct RealExecutionProductCommandRuntimeLogPreview {
    status: String,
    runtime_log_refs: Vec<String>,
    redaction_status: String,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct RealExecutionProductCommandAuditPreview {
    status: String,
    audit_refs: Vec<String>,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct RealExecutionProductCommandReadbackBoundary {
    status: String,
    attempted: bool,
    real_readback_performed: bool,
    result_count: Option<i64>,
    unavailable_reason: Option<String>,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct RealExecutionProductCommandPreview {
    preview_id: String,
    request: RealExecutionProductCommandRequest,
    permission_envelope: RealExecutionProductCommandPermissionEnvelope,
    readiness: RealExecutionProductCommandReadiness,
    guard_preview: RealExecutionProductCommandGuardPreview,
    diagnostics_summary: RealExecutionProductCommandDiagnosticsSummary,
    duplicate_scope: RealExecutionProductCommandDuplicateScope,
    runtime_log_preview: RealExecutionProductCommandRuntimeLogPreview,
    audit_preview: RealExecutionProductCommandAuditPreview,
    readback_boundary: RealExecutionProductCommandReadbackBoundary,
    warnings: Vec<String>,
    blocked_reasons: Vec<String>,
    prompt_sent: bool,
    real_codex_executed: bool,
    writes_codex_home: bool,
    writes_project_files: bool,
    writes_workbench_state: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct RealExecutionProductCommandDecision {
    decision_id: String,
    product_command_id: String,
    decision: String,
    confirmed_by: String,
    confirmed_at: String,
    store_revision: i64,
    risk_acknowledgement: String,
    allowed_once: bool,
    reason: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct RealExecutionProductCommandAttempt {
    attempt_id: String,
    product_command_id: String,
    continuation_id: Option<String>,
    adapter_id: String,
    operation_id: String,
    status: String,
    started_at: String,
    completed_at: Option<String>,
    runner_call_allowed: bool,
    prompt_sent: bool,
    real_codex_executed: bool,
    writes_codex_home: bool,
    writes_project_files: bool,
    runtime_log_ref: Option<String>,
    audit_refs: Vec<String>,
    readback_summary: RealExecutionProductCommandReadbackBoundary,
    failure_reason: Option<String>,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct RealExecutionProductCommandStore {
    schema_version: String,
    revision: i64,
    created_at: String,
    updated_at: String,
    last_write_id: Option<String>,
    commands: Vec<RealExecutionProductCommandRequest>,
    previews: Vec<RealExecutionProductCommandPreview>,
    decisions: Vec<RealExecutionProductCommandDecision>,
    attempts: Vec<RealExecutionProductCommandAttempt>,
    audit_refs: Vec<String>,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct RealExecutionProductCommandFailureStopRetryItem {
    kind: String,
    title: String,
    summary: String,
    count: usize,
    severity: String,
    requires_new_user_confirmation: bool,
    result_count: Option<i64>,
    source_refs: Vec<String>,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct RealExecutionProductCommandFailureStopRetrySummary {
    schema_version: String,
    item_count: usize,
    failure_count: usize,
    blocked_count: usize,
    readback_issue_count: usize,
    manual_stop_requested_count: usize,
    retry_requires_new_user_confirmation: bool,
    items: Vec<RealExecutionProductCommandFailureStopRetryItem>,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct RealExecutionProductCommandReadModel {
    schema_version: String,
    sidecar_name: String,
    sidecar_path: Option<String>,
    store_available: bool,
    store_revision: i64,
    command_count: usize,
    pending_decision_count: usize,
    running_attempt_count: usize,
    blocked_attempt_count: usize,
    last_attempt_status: Option<String>,
    failure_stop_retry_summary: RealExecutionProductCommandFailureStopRetrySummary,
    ordinary_product_entry_status: String,
    legacy_entry_status: String,
    runner_entry_status: String,
    level_b_authorization_required: bool,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct ProjectWorkflowAutomationInput {
    project_root: String,
    project_id: Option<String>,
    workflow_id: Option<String>,
    workflow_node_id: Option<String>,
    work_item_id: Option<String>,
    user_goal: String,
    task_package_ref: Option<String>,
    memory_packet_ref: Option<String>,
    target_session_id: Option<String>,
    sandbox: Option<String>,
    requested_by: Option<String>,
    confirmed_by: Option<String>,
    risk_acknowledgement: Option<String>,
    reason: Option<String>,
    expected_workflow_revision: Option<i64>,
    expected_product_command_store_revision: Option<i64>,
    expected_session_continuation_store_revision: Option<i64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct ProjectWorkflowAutomationPlan {
    schema_version: String,
    automation_id: String,
    project_id: String,
    project_root: String,
    workflow_id: String,
    user_goal: String,
    current_phase: String,
    next_step: String,
    run_units: Vec<ProjectWorkflowRunUnit>,
    blocked_reasons: Vec<String>,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct ProjectWorkflowRunUnit {
    run_unit_id: String,
    run_unit_kind: String,
    role: String,
    status: String,
    project_id: String,
    project_root: String,
    workflow_id: String,
    workflow_node_id: String,
    work_item_id: String,
    task_package_ref: Option<String>,
    memory_packet_ref: Option<String>,
    product_command_preview_ref: Option<String>,
    product_command_ref: Option<String>,
    runtime_log_refs: Vec<String>,
    audit_refs: Vec<String>,
    readback_ref: Option<String>,
    readback_status: String,
    readback_result_count: Option<i64>,
    worker_report_ref: Option<String>,
    #[serde(default)]
    capture_event_refs: Vec<String>,
    observation_refs: Vec<String>,
    memory_candidate_refs: Vec<String>,
    runner_call_allowed: bool,
    prompt_sent: bool,
    real_codex_executed: bool,
    writes_codex_home: bool,
    writes_project_files: bool,
    summary: String,
    next_step: String,
    blocked_reasons: Vec<String>,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct ProjectWorkflowAutomationResult {
    status: String,
    plan: ProjectWorkflowAutomationPlan,
    phase_a_output: Option<RealExecutionProductCommandPhaseAOutput>,
    worker_report_result: Option<WorkflowStateMutationResult>,
    process_fact_result: Option<ProjectDirectorProcessFactDecisionResult>,
    read_model: ProjectWorkflowAutomationReadModel,
    prompt_sent: bool,
    real_codex_executed: bool,
    writes_codex_home: bool,
    writes_project_files: bool,
    blocked_reasons: Vec<String>,
    warnings: Vec<String>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
struct ProjectWorkflowAutomationJ2BB1Input {
    project_root: Option<String>,
    project_id: Option<String>,
    workflow_id: Option<String>,
    workflow_node_id: Option<String>,
    work_item_id: Option<String>,
    task_package_ref: Option<String>,
    memory_packet_ref: Option<String>,
    target_session_id: Option<String>,
    sandbox: Option<String>,
    prompt_summary: Option<String>,
    prompt_ref: Option<String>,
    prompt_hash: Option<String>,
    requested_by: Option<String>,
    confirmed_by: Option<String>,
    risk_acknowledgement: Option<String>,
    reason: Option<String>,
    expected_workflow_revision: Option<i64>,
    expected_product_command_store_revision: Option<i64>,
    expected_session_continuation_store_revision: Option<i64>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct ProjectWorkflowAutomationJ2BB1Output {
    status: String,
    plan: ProjectWorkflowAutomationPlan,
    product_command_id: String,
    preview: RealExecutionProductCommandPreview,
    prepare_output: RealExecutionProductCommandPrepareOutput,
    decision_output: RealExecutionProductCommandDecisionOutput,
    phase_a_output: RealExecutionProductCommandPhaseAOutput,
    phase_b_output: RealExecutionProductCommandPhaseBOutput,
    prompt_body_persisted: bool,
    allowed_project_write_roots: Vec<String>,
    runtime_log_refs: Vec<String>,
    audit_refs: Vec<String>,
    readback_ref: Option<String>,
    blocked_reasons: Vec<String>,
    warnings: Vec<String>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
struct ProjectWorkflowAutomationJ2BB2Input {
    project_root: Option<String>,
    project_id: Option<String>,
    workflow_id: Option<String>,
    workflow_node_id: Option<String>,
    work_item_id: Option<String>,
    task_package_ref: Option<String>,
    memory_packet_ref: Option<String>,
    sandbox: Option<String>,
    allowed_write_path: Option<String>,
    prompt_summary: Option<String>,
    prompt_ref: Option<String>,
    prompt_hash: Option<String>,
    requested_by: Option<String>,
    confirmed_by: Option<String>,
    risk_acknowledgement: Option<String>,
    reason: Option<String>,
    expected_workflow_revision: Option<i64>,
    expected_product_command_store_revision: Option<i64>,
    expected_session_continuation_store_revision: Option<i64>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct ProjectWorkflowAutomationJ2BB2Output {
    status: String,
    plan: ProjectWorkflowAutomationPlan,
    product_command_id: String,
    preview: RealExecutionProductCommandPreview,
    prepare_output: RealExecutionProductCommandPrepareOutput,
    decision_output: RealExecutionProductCommandDecisionOutput,
    phase_a_output: RealExecutionProductCommandPhaseAOutput,
    phase_b_output: RealExecutionProductCommandPhaseBOutput,
    prompt_body_persisted: bool,
    allowed_project_write_roots: Vec<String>,
    allowed_project_write_path: String,
    runtime_log_refs: Vec<String>,
    audit_refs: Vec<String>,
    readback_ref: Option<String>,
    blocked_reasons: Vec<String>,
    warnings: Vec<String>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
struct ProjectWorkflowAutomationK3BInput {
    execution_point_id: String,
    project_root: Option<String>,
    project_id: Option<String>,
    workflow_id: Option<String>,
    workflow_node_id: Option<String>,
    run_unit_id: Option<String>,
    work_item_id: Option<String>,
    task_package_ref: Option<String>,
    task_memory_packet_ref: Option<String>,
    permission_envelope_ref: Option<String>,
    readback_marker: Option<String>,
    target_session_id: Option<String>,
    sandbox: Option<String>,
    allowed_write_path: Option<String>,
    prompt_summary: Option<String>,
    prompt_ref: Option<String>,
    prompt_hash: Option<String>,
    runtime_prompt_body: Option<String>,
    requested_by: Option<String>,
    confirmed_by: Option<String>,
    risk_acknowledgement: Option<String>,
    reason: Option<String>,
    expected_workflow_revision: Option<i64>,
    expected_product_command_store_revision: Option<i64>,
    expected_session_continuation_store_revision: Option<i64>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct ProjectWorkflowAutomationK3BOutput {
    status: String,
    execution_point_id: String,
    run_unit_id: String,
    workflow_id: String,
    work_item_id: String,
    task_memory_packet_ref: String,
    permission_envelope_ref: String,
    readback_marker: String,
    plan: ProjectWorkflowAutomationPlan,
    product_command_id: String,
    preview: RealExecutionProductCommandPreview,
    prepare_output: RealExecutionProductCommandPrepareOutput,
    decision_output: RealExecutionProductCommandDecisionOutput,
    phase_a_output: RealExecutionProductCommandPhaseAOutput,
    phase_b_output: RealExecutionProductCommandPhaseBOutput,
    prompt_body_persisted: bool,
    allowed_project_write_roots: Vec<String>,
    allowed_project_write_path: Option<String>,
    baseline_refs: Vec<String>,
    manifest_requirements: Vec<String>,
    runtime_log_refs: Vec<String>,
    audit_refs: Vec<String>,
    readback_ref: Option<String>,
    blocked_reasons: Vec<String>,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct ProjectWorkflowAutomationReadModel {
    schema_version: String,
    available: bool,
    generated_at: String,
    latest_automation_id: Option<String>,
    latest_status: Option<String>,
    latest_plan: Option<ProjectWorkflowAutomationPlan>,
    run_unit_count: usize,
    waiting_user_count: usize,
    blocked_count: usize,
    readback_unknown_count: usize,
    worker_report_count: usize,
    capture_event_count: usize,
    observation_count: usize,
    next_step: Option<String>,
    warnings: Vec<String>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
struct CodexControlCommandInput {
    project_id: Option<String>,
    project_root: String,
    workflow_id: Option<String>,
    node_id: Option<String>,
    work_item_id: Option<String>,
    task_package_ref: Option<String>,
    memory_packet_ref: Option<String>,
    adapter_id: String,
    operation_id: String,
    session_mode: String,
    target_session_id: Option<String>,
    sandbox: String,
    prompt_summary: String,
    prompt_ref: String,
    prompt_hash: String,
    allowed_write_roots: Vec<String>,
    denied_paths: Vec<String>,
    readback_plan: String,
    timeout_ms: Option<i64>,
    requested_by: Option<String>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
struct PreviewRealExecutionProductCommandInput {
    source_kind: String,
    h5_dispatch_preview: Option<H5ProjectWorkflowDispatchPreviewInput>,
    codex_control: Option<CodexControlCommandInput>,
    requested_by: Option<String>,
    created_at: Option<String>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
struct PrepareRealExecutionProductCommandInput {
    source_kind: String,
    h5_dispatch_preview: Option<H5ProjectWorkflowDispatchPreviewInput>,
    codex_control: Option<CodexControlCommandInput>,
    expected_store_revision: Option<i64>,
    requested_by: Option<String>,
    created_at: Option<String>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
struct RecordRealExecutionProductCommandDecisionInput {
    product_command_id: String,
    decision: String,
    expected_store_revision: Option<i64>,
    confirmed_by: String,
    risk_acknowledgement: String,
    allowed_once: bool,
    reason: String,
    requested_by: Option<String>,
    confirmed_at: Option<String>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
struct ConfirmRealExecutionProductCommandInput {
    product_command_id: String,
    expected_store_revision: Option<i64>,
    confirmed_by: String,
    risk_acknowledgement: String,
    allowed_once: bool,
    reason: String,
    requested_by: Option<String>,
    confirmed_at: Option<String>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
struct RunRealExecutionProductCommandPhaseAInput {
    product_command_id: String,
    expected_product_command_store_revision: Option<i64>,
    expected_session_continuation_store_revision: Option<i64>,
    actor_role: String,
    execution_decision: Option<String>,
    timeout_ms: Option<i64>,
    requested_at: Option<String>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
struct RunRealExecutionProductCommandPhaseBInput {
    product_command_id: String,
    expected_product_command_store_revision: Option<i64>,
    expected_session_continuation_store_revision: Option<i64>,
    actor_role: String,
    execution_decision: Option<String>,
    authorization: H2RealResumeAuthorizationMatrix,
    prompt_body: String,
    requested_at: Option<String>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
struct RunRealExecutionProductCommandNewSessionPhaseBInput {
    product_command_id: String,
    expected_product_command_store_revision: Option<i64>,
    expected_session_continuation_store_revision: Option<i64>,
    actor_role: String,
    execution_decision: Option<String>,
    authorization: H3RealNewSessionAuthorizationMatrix,
    prompt_body: String,
    requested_at: Option<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct RealExecutionProductCommandPrepareOutput {
    status: String,
    product_command_id: Option<String>,
    preview: RealExecutionProductCommandPreview,
    read_model: RealExecutionProductCommandReadModel,
    store_revision: i64,
    sidecar_path: Option<String>,
    writes_product_command_sidecar: bool,
    blocked_reasons: Vec<String>,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct RealExecutionProductCommandDecisionOutput {
    status: String,
    decision: Option<RealExecutionProductCommandDecision>,
    read_model: RealExecutionProductCommandReadModel,
    store_revision: i64,
    sidecar_path: Option<String>,
    audit_ref: Option<String>,
    runner_call_allowed: bool,
    prompt_sent: bool,
    real_codex_executed: bool,
    writes_codex_home: bool,
    writes_project_files: bool,
    writes_product_command_sidecar: bool,
    blocked_reasons: Vec<String>,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct RealExecutionProductCommandPhaseAOutput {
    status: String,
    product_command_id: String,
    product_command_attempt: Option<RealExecutionProductCommandAttempt>,
    read_model: RealExecutionProductCommandReadModel,
    product_command_store_revision: i64,
    product_command_sidecar_path: Option<String>,
    continuation_id: Option<String>,
    continuation_attempt_id: Option<String>,
    session_continuation_store_revision: Option<i64>,
    runtime_log_ref: Option<String>,
    audit_refs: Vec<String>,
    readback_summary: RealExecutionProductCommandReadbackBoundary,
    runner_call_allowed: bool,
    prompt_sent: bool,
    real_codex_executed: bool,
    writes_codex_home: bool,
    writes_project_files: bool,
    writes_product_command_sidecar: bool,
    writes_continuation_sidecar: bool,
    writes_runtime_log: bool,
    blocked_reasons: Vec<String>,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct RealExecutionProductCommandPhaseBOutput {
    status: String,
    product_command_id: String,
    product_command_attempt: Option<RealExecutionProductCommandAttempt>,
    read_model: RealExecutionProductCommandReadModel,
    product_command_store_revision: i64,
    product_command_sidecar_path: Option<String>,
    continuation_id: Option<String>,
    continuation_attempt_id: Option<String>,
    session_continuation_store_revision: Option<i64>,
    runtime_log_ref: Option<String>,
    audit_refs: Vec<String>,
    readback_summary: RealExecutionProductCommandReadbackBoundary,
    runner_call_allowed: bool,
    prompt_sent: bool,
    real_codex_executed: bool,
    writes_codex_home: bool,
    writes_project_files: bool,
    writes_product_command_sidecar: bool,
    writes_continuation_sidecar: bool,
    writes_runtime_log: bool,
    blocked_reasons: Vec<String>,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct WorkbenchSnapshot {
    summary: IndexSummary,
    projects: Vec<ProjectRecord>,
    sessions: Vec<SessionRecord>,
    skills: Vec<SkillRecord>,
    plugins: Vec<PluginRecord>,
    tasks: Vec<TaskEntry>,
    agent_adapters: Vec<AgentAdapterDescriptor>,
    session_operations: Vec<SessionOperationDescriptor>,
    provider_availability: Vec<ProviderAvailabilitySummary>,
    session_continuation_previews: Vec<SessionContinuationPreview>,
    session_continuation_store: SessionContinuationStoreV1,
    runtime_session_attention: Vec<RuntimeSessionAttention>,
    session_run_status_summaries: Vec<SessionRunStatusSummary>,
    runtime_log_store: RuntimeLogStoreV1,
    worker_protocol: WorkerProtocolReadModel,
    real_execution_product_commands: RealExecutionProductCommandReadModel,
    project_workflow_automation: ProjectWorkflowAutomationReadModel,
    page_read_model_inventory: page_read_model::WorkbenchPageReadModelInventory,
    diagnostic_summary: DiagnosticSummary,
    diagnostics: Diagnostics,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SessionSourceMode {
    RealWithSqliteFallback,
    IndexOnly,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct WorkflowStateCounts {
    projects: usize,
    agent_adapters: usize,
    workflows: usize,
    nodes: usize,
    edges: usize,
    work_items: usize,
    artifacts: usize,
    reviews: usize,
    audit_events: usize,
    capabilities: usize,
    harness_resources: usize,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum PlanAuthorizationStatus {
    Draft,
    PendingUserConfirmation,
    UserConfirmed,
    PendingGlobalBoundaryReview,
    Active,
    Paused,
    Revoked,
    Expired,
    Completed,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct PlanAuthorizationActorScope {
    allowed_role_ids: Vec<String>,
    allowed_agent_ids: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct PlanAuthorizationResourceScope {
    allowed_read_roots: Vec<String>,
    allowed_write_roots: Vec<String>,
    allowed_tools: Vec<String>,
    allowed_checks: Vec<String>,
    allowed_task_package_kinds: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct PlanAuthorizationStopCondition {
    condition_id: String,
    kind: String,
    summary: String,
    requires_user_confirmation: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct AuthorizedExecutionScope {
    project_id: String,
    workflow_id: String,
    allowed_role_ids: Vec<String>,
    allowed_agent_ids: Vec<String>,
    allowed_read_roots: Vec<String>,
    allowed_write_roots: Vec<String>,
    allowed_tools: Vec<String>,
    allowed_checks: Vec<String>,
    allowed_task_package_kinds: Vec<String>,
    max_worker_dispatches: Option<i64>,
    max_runtime_minutes: Option<i64>,
    stop_conditions: Vec<PlanAuthorizationStopCondition>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct PlanAuthorizationUserConfirmation {
    confirmed_by: String,
    confirmed_at_ms: i64,
    confirmation_summary: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct GlobalBoundaryReviewChecklist {
    architecture_boundary_checked: bool,
    cross_project_impact_checked: bool,
    permission_scope_checked: bool,
    read_write_scope_checked: bool,
    tool_and_check_scope_checked: bool,
    memory_boundary_checked: bool,
    stop_conditions_checked: bool,
    acceptance_criteria_checked: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct GlobalBoundaryReviewFinding {
    finding_id: String,
    severity: String,
    summary: String,
    recommendation: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct PlanAuthorizationGlobalBoundaryReview {
    reviewed_by: String,
    reviewed_at_ms: i64,
    status: String,
    summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    source_proposal_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    checklist: Option<GlobalBoundaryReviewChecklist>,
    #[serde(default)]
    findings: Vec<GlobalBoundaryReviewFinding>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    reviewed_scope_fingerprint: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct PlanAuthorization {
    authorization_id: String,
    schema_version: String,
    project_id: String,
    workflow_id: String,
    source_proposal_id: Option<String>,
    title: String,
    goal_summary: String,
    status: PlanAuthorizationStatus,
    scope: AuthorizedExecutionScope,
    user_confirmation: Option<PlanAuthorizationUserConfirmation>,
    global_boundary_review: Option<PlanAuthorizationGlobalBoundaryReview>,
    audit_refs: Vec<String>,
    created_at_ms: i64,
    updated_at_ms: i64,
    expires_at_ms: Option<i64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct AutoDispatchGuardInput {
    project_id: String,
    workflow_id: String,
    work_item_id: String,
    task_package_id: Option<String>,
    task_package_kind: Option<String>,
    target_role_id: String,
    target_agent_id: Option<String>,
    requested_read_roots: Vec<String>,
    requested_write_roots: Vec<String>,
    requested_tools: Vec<String>,
    requested_checks: Vec<String>,
    triggered_stop_conditions: Vec<String>,
    dispatch_kind: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct AutoDispatchGuardResult {
    status: String,
    authorization_id: Option<String>,
    reasons: Vec<String>,
    required_user_confirmation: bool,
    required_global_review: bool,
    checked_at_ms: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct PlanAuthorizationAuditEvent {
    audit_event_id: String,
    event_type: String,
    actor_id: String,
    actor_role: String,
    project_id: String,
    workflow_id: String,
    authorization_id: Option<String>,
    work_item_id: Option<String>,
    before_status: Option<PlanAuthorizationStatus>,
    after_status: Option<PlanAuthorizationStatus>,
    reason: String,
    guard_result: Option<AutoDispatchGuardResult>,
    created_at_ms: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct PlanAuthorizationStoreV1 {
    schema_version: String,
    revision: i64,
    authorizations: Vec<PlanAuthorization>,
    audit_events: Vec<PlanAuthorizationAuditEvent>,
    updated_at_ms: i64,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct PlanAuthorizationReadModel {
    sidecar_name: String,
    revision: i64,
    project_id: String,
    workflow_id: String,
    authorization_count: usize,
    active_authorization_id: Option<String>,
    latest_authorization_id: Option<String>,
    latest_status: Option<PlanAuthorizationStatus>,
    actor_scope: Option<PlanAuthorizationActorScope>,
    resource_scope: Option<PlanAuthorizationResourceScope>,
    stop_condition_count: usize,
    recent_audit_event_id: Option<String>,
    recent_guard_result: Option<AutoDispatchGuardResult>,
    display_text: String,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum ProjectConsultationProposalStatus {
    Draft,
    PendingUserConfirmation,
    UserConfirmed,
    ChangesRequested,
    Rejected,
    Superseded,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum ProjectConsultationProposalDecisionKind {
    Confirm,
    RequestChanges,
    Reject,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum ProjectConsultationProposalCreatorRole {
    ProjectConsultant,
    ProjectDirector,
    User,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct ProjectConsultationProposalScopeDraft {
    allowed_role_ids: Vec<String>,
    allowed_agent_ids: Vec<String>,
    allowed_read_roots: Vec<String>,
    allowed_write_roots: Vec<String>,
    allowed_tools: Vec<String>,
    allowed_checks: Vec<String>,
    allowed_task_package_kinds: Vec<String>,
    stop_conditions: Vec<String>,
    max_worker_dispatches: Option<i64>,
    max_runtime_minutes: Option<i64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct ProjectConsultationProposalRisk {
    risk_id: String,
    severity: String,
    summary: String,
    mitigation: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct ProjectConsultationProposal {
    proposal_id: String,
    schema_version: String,
    project_id: String,
    workflow_id: String,
    title: String,
    user_goal: String,
    goal_summary: String,
    proposed_steps: Vec<String>,
    scope_draft: ProjectConsultationProposalScopeDraft,
    risks: Vec<ProjectConsultationProposalRisk>,
    acceptance_criteria: Vec<String>,
    status: ProjectConsultationProposalStatus,
    plan_authorization_id: Option<String>,
    created_by_role: ProjectConsultationProposalCreatorRole,
    created_at_ms: i64,
    updated_at_ms: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct ProjectConsultationProposalDecision {
    decision_id: String,
    proposal_id: String,
    decided_by: String,
    decision: ProjectConsultationProposalDecisionKind,
    summary: String,
    created_at_ms: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct ProjectConsultationProposalAuditEvent {
    audit_event_id: String,
    event_type: String,
    actor_id: String,
    actor_role: String,
    project_id: String,
    workflow_id: String,
    proposal_id: Option<String>,
    plan_authorization_id: Option<String>,
    before_status: Option<ProjectConsultationProposalStatus>,
    after_status: Option<ProjectConsultationProposalStatus>,
    reason: String,
    created_at_ms: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct ProjectConsultationProposalStoreV1 {
    schema_version: String,
    revision: i64,
    proposals: Vec<ProjectConsultationProposal>,
    decisions: Vec<ProjectConsultationProposalDecision>,
    audit_events: Vec<ProjectConsultationProposalAuditEvent>,
    updated_at_ms: i64,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct ProjectConsultationProposalReadModel {
    sidecar_name: String,
    revision: i64,
    project_id: String,
    workflow_id: String,
    proposal_count: usize,
    latest_proposal_id: Option<String>,
    latest_status: Option<ProjectConsultationProposalStatus>,
    linked_plan_authorization_id: Option<String>,
    decision_count: usize,
    risk_count: usize,
    stop_condition_count: usize,
    display_text: String,
    warnings: Vec<String>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
struct CreateProjectConsultationProposalInput {
    project_root: String,
    project_id: Option<String>,
    workflow_id: Option<String>,
    title: String,
    user_goal: String,
    goal_summary: String,
    proposed_steps: Vec<String>,
    scope_draft: ProjectConsultationProposalScopeDraft,
    risks: Vec<ProjectConsultationProposalRisk>,
    acceptance_criteria: Vec<String>,
    created_by_role: ProjectConsultationProposalCreatorRole,
    actor_id: String,
    expected_store_revision: Option<i64>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct CreateProjectConsultationProposalOutput {
    proposal: ProjectConsultationProposal,
    audit_event: ProjectConsultationProposalAuditEvent,
    read_model: ProjectConsultationProposalReadModel,
    store_revision: i64,
    warnings: Vec<String>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
struct RenderProjectConsultationProposalMarkdownInput {
    project_root: String,
    proposal_id: String,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct ProjectConsultationProposalMarkdown {
    proposal_id: String,
    markdown: String,
    warnings: Vec<String>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
struct RecordProjectConsultationProposalDecisionInput {
    project_root: String,
    proposal_id: String,
    actor_id: String,
    decision: ProjectConsultationProposalDecisionKind,
    summary: String,
    expected_proposal_store_revision: Option<i64>,
    expected_plan_authorization_store_revision: Option<i64>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct RecordProjectConsultationProposalDecisionOutput {
    proposal: ProjectConsultationProposal,
    decision: ProjectConsultationProposalDecision,
    audit_event: ProjectConsultationProposalAuditEvent,
    read_model: ProjectConsultationProposalReadModel,
    plan_authorization: Option<PlanAuthorization>,
    plan_authorization_audit_event: Option<PlanAuthorizationAuditEvent>,
    plan_authorization_store_revision: Option<i64>,
    store_revision: i64,
    warnings: Vec<String>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
struct CreatePlanAuthorizationInput {
    project_root: String,
    project_id: Option<String>,
    workflow_id: Option<String>,
    source_proposal_id: Option<String>,
    title: String,
    goal_summary: String,
    scope: AuthorizedExecutionScope,
    actor_id: String,
    actor_role: String,
    expires_at_ms: Option<i64>,
    expected_store_revision: Option<i64>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct CreatePlanAuthorizationOutput {
    authorization: PlanAuthorization,
    audit_event: PlanAuthorizationAuditEvent,
    read_model: PlanAuthorizationReadModel,
    store_revision: i64,
    warnings: Vec<String>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
struct RecordPlanAuthorizationUserConfirmationInput {
    project_root: String,
    authorization_id: String,
    actor_id: String,
    confirmation_summary: String,
    expected_store_revision: Option<i64>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
struct RecordPlanAuthorizationGlobalBoundaryReviewInput {
    project_root: String,
    authorization_id: String,
    actor_id: String,
    review_status: String,
    summary: String,
    source_proposal_id: Option<String>,
    checklist: Option<GlobalBoundaryReviewChecklist>,
    findings: Vec<GlobalBoundaryReviewFinding>,
    reviewed_scope_fingerprint: Option<String>,
    expected_store_revision: Option<i64>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
struct RecordGlobalBoundaryReviewInput {
    project_root: String,
    project_id: String,
    workflow_id: String,
    proposal_id: String,
    authorization_id: String,
    actor_id: String,
    review_status: String,
    summary: String,
    checklist: GlobalBoundaryReviewChecklist,
    findings: Vec<GlobalBoundaryReviewFinding>,
    expected_authorization_revision: Option<i64>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
struct RevokePlanAuthorizationInput {
    project_root: String,
    authorization_id: String,
    actor_id: String,
    actor_role: String,
    reason: String,
    expected_store_revision: Option<i64>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct RecordPlanAuthorizationOutput {
    authorization: PlanAuthorization,
    audit_event: PlanAuthorizationAuditEvent,
    read_model: PlanAuthorizationReadModel,
    store_revision: i64,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct RecordGlobalBoundaryReviewOutput {
    authorization: PlanAuthorization,
    audit_event: PlanAuthorizationAuditEvent,
    read_model: PlanAuthorizationReadModel,
    guard_result: AutoDispatchGuardResult,
    store_revision: i64,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct ProjectDirectorTaskScope {
    project_id: String,
    workflow_id: String,
    target_role: String,
    task_package_kind: String,
    allowed_read_scope: Vec<String>,
    allowed_write_scope: Vec<String>,
    callable_tool_capabilities: Vec<String>,
    required_checks: Vec<String>,
    stop_conditions: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct ProjectDirectorPlannedTask {
    planned_task_id: String,
    title: String,
    objective: String,
    scope: ProjectDirectorTaskScope,
    depends_on: Vec<String>,
    acceptance_criteria: Vec<String>,
    report_format: Vec<String>,
    status: String,
    guard_result: Option<AutoDispatchGuardResult>,
    work_item_id: Option<String>,
    workflow_node_id: Option<String>,
    task_package_id: Option<String>,
    memory_packet_snapshot_id: Option<String>,
    prepared_dispatch_id: Option<String>,
    blocked_reasons: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct ProjectDirectorTaskPlan {
    project_root: String,
    project_id: String,
    workflow_id: String,
    proposal_id: String,
    authorization_id: String,
    actor_id: String,
    planned_tasks: Vec<ProjectDirectorPlannedTask>,
    planned_task_count: usize,
    authorized_task_count: usize,
    prepared_dispatch_count: usize,
    blocked_count: usize,
    needs_binding_count: usize,
    blocked_reasons: Vec<String>,
    memory_snapshot_summary: TaskPackageMemoryInjectionSummary,
    display_text: String,
    warnings: Vec<String>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
struct PreviewProjectDirectorTaskPlanInput {
    project_root: String,
    project_id: String,
    workflow_id: String,
    proposal_id: String,
    authorization_id: String,
    actor_id: String,
    expected_authorization_revision: Option<i64>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
struct PrepareAuthorizedAutoDispatchInput {
    project_root: String,
    project_id: String,
    workflow_id: String,
    proposal_id: String,
    authorization_id: String,
    actor_id: String,
    #[serde(default)]
    planned_tasks: Vec<ProjectDirectorPlannedTask>,
    expected_workflow_revision: Option<i64>,
    expected_authorization_revision: Option<i64>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct PreparedAutoDispatchReadModel {
    dispatch_id: Option<String>,
    planned_task_id: String,
    work_item_id: Option<String>,
    workflow_node_id: Option<String>,
    task_package_id: Option<String>,
    status: String,
    authorization_check: AutoDispatchGuardResult,
    memory_packet_snapshot_id: Option<String>,
    memory_packet_fingerprint: Option<String>,
    binding_status: String,
    prompt_preview: Option<String>,
    blocked_reasons: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct AuthorizedPreparedDispatchResult {
    message: String,
    path: String,
    backup_path: Option<String>,
    audit_event_id: String,
    plan: ProjectDirectorTaskPlan,
    prepared_dispatches: Vec<PreparedAutoDispatchReadModel>,
    snapshot: WorkflowStateSnapshot,
    warnings: Vec<String>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
struct H5ProjectWorkflowDispatchPreviewInput {
    project_root: String,
    project_id: String,
    workflow_id: String,
    dispatch_id: String,
    actor_id: String,
    operation_id: Option<String>,
    session_id: Option<String>,
    target_cwd: Option<String>,
    sandbox: Option<String>,
    prompt_summary: String,
    prompt_ref: String,
    prompt_sha256: String,
    #[serde(default)]
    h3_b_level_b_authorized: bool,
    expected_workflow_revision: Option<i64>,
    diagnostic_summary: Option<H5DiagnosticSummaryInput>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct H5DiagnosticDegradedStateInput {
    kind: String,
    blocks_real_execution: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct H5DiagnosticSummaryInput {
    overall_severity: String,
    blocked_count: usize,
    degraded_states: Vec<H5DiagnosticDegradedStateInput>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct H5TaskMemoryPacketDispatchSummary {
    snapshot_id: Option<String>,
    fingerprint: Option<String>,
    included_count: usize,
    excluded_count: usize,
    review_material_count: usize,
    stale: bool,
    stale_reasons: Vec<String>,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct H5PermissionEnvelopePreview {
    status: String,
    explicit_approval_required: bool,
    approved_for_real_execution: bool,
    adapter_id: String,
    operation_id: String,
    target_session_id: Option<String>,
    cwd: String,
    project_root: String,
    allowed_write_roots: Vec<String>,
    denied_paths: Vec<String>,
    prompt_summary: String,
    prompt_ref: String,
    prompt_sha256: String,
    memory_packet_fingerprint: Option<String>,
    readback_boundary: String,
    codex_home_boundary: String,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct H5RuntimeAuditPreview {
    runtime_log_refs: Vec<CodexLocalRuntimeLogRef>,
    audit_refs: Vec<CodexLocalAuditRef>,
    diagnostic_status: String,
    diagnostic_blockers: Vec<String>,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct H5ReadbackBoundaryPreview {
    status: String,
    result_count: Option<i64>,
    unavailable_behavior: String,
    worker_report_candidate_allowed: bool,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct H5ProjectWorkflowDispatchPreview {
    preview_version: i64,
    preview_id: String,
    status: String,
    level: String,
    project_id: String,
    workflow_id: String,
    workflow_node_id: String,
    work_item_id: String,
    dispatch_id: String,
    task_package_id: Option<String>,
    operation_id: String,
    target_session_id: Option<String>,
    memory_packet: H5TaskMemoryPacketDispatchSummary,
    permission_envelope: H5PermissionEnvelopePreview,
    codex_local_request: Option<CodexLocalExecutionRequest>,
    codex_local_guard: Option<CodexLocalExecutionGuard>,
    runtime_audit_preview: H5RuntimeAuditPreview,
    readback_boundary: H5ReadbackBoundaryPreview,
    worker_report_candidate: Option<WorkerStructuredReportInput>,
    process_fact_handoff: Option<ProjectDirectorProcessFactDecisionInput>,
    final_review_handoff_status: String,
    prompt_sent: bool,
    real_codex_executed: bool,
    writes_codex_home: bool,
    writes_project_files: bool,
    writes_workbench_state: bool,
    blocked_reasons: Vec<String>,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct WorkerStructuredReportInput {
    project_root: String,
    project_id: String,
    workflow_id: String,
    workflow_node_id: String,
    work_item_id: String,
    dispatch_id: Option<String>,
    actor_role: String,
    executed_what: String,
    changed_what: String,
    summary: String,
    evidence_refs: Vec<String>,
    open_issues: Vec<String>,
    permission_requests: Vec<String>,
    direction_risks: Vec<String>,
    follow_up_suggestions: Vec<String>,
    acceptance_status: String,
    source_refs: Vec<ObservationSourceRef>,
    expected_workflow_revision: Option<i64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct ProcessFactCandidate {
    process_fact_id: String,
    summary: String,
    source_report_id: String,
    source_dispatch_id: Option<String>,
    evidence_refs: Vec<String>,
    source_refs: Vec<ObservationSourceRef>,
    scope: MemoryScope,
    risk_level: String,
    sensitive_level: String,
    proposed_observation_type: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct ProjectDirectorProcessFactDecisionInput {
    project_root: String,
    project_id: String,
    workflow_id: String,
    report_id: String,
    actor_id: String,
    actor_role: String,
    decision: String,
    accepted_facts: Vec<ProcessFactCandidate>,
    rejected_fact_ids: Vec<String>,
    summary: String,
    expected_workflow_revision: Option<i64>,
    expected_observation_store_revision: Option<i64>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct ProjectDirectorProcessFactDecisionResult {
    message: String,
    path: String,
    backup_path: Option<String>,
    audit_event_id: String,
    decision_record_id: String,
    observations: Vec<ObservationRecord>,
    observation_store_revision: Option<i64>,
    snapshot: WorkflowStateSnapshot,
    warnings: Vec<String>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
struct GlobalFinalResultReviewInput {
    project_root: String,
    project_id: String,
    workflow_id: String,
    authorization_id: String,
    proposal_id: String,
    actor_id: String,
    actor_role: String,
    decision: String,
    summary: String,
    evidence_refs: Vec<String>,
    accepted_process_fact_ids: Vec<String>,
    open_issues: Vec<String>,
    deferred_items: Vec<String>,
    expected_workflow_revision: Option<i64>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
struct UserResultDecisionInput {
    project_root: String,
    project_id: String,
    workflow_id: String,
    actor_id: String,
    actor_role: String,
    decision: String,
    summary: String,
    requested_changes: Vec<String>,
    accepted_review_id: Option<String>,
    expected_workflow_revision: Option<i64>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
struct GenerateStageCAcceptanceSummaryInput {
    project_root: String,
    project_id: String,
    workflow_id: String,
    expected_workflow_revision: Option<i64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct StageCAcceptanceGate {
    gate_id: String,
    label: String,
    status: String,
    reason: String,
    evidence_refs: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct StageCAcceptanceSummary {
    project_id: String,
    workflow_id: String,
    gates: Vec<StageCAcceptanceGate>,
    final_review_status: String,
    user_decision_status: String,
    accepted_as_stage_c_complete: bool,
    deferred_items: Vec<String>,
    open_blockers: Vec<String>,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct WorkflowResultSummaryReadModel {
    project_id: String,
    workflow_id: String,
    final_review_status: String,
    final_review_id: Option<String>,
    user_decision_status: String,
    user_decision_id: Option<String>,
    stage_c_acceptance: StageCAcceptanceSummary,
    open_issues: Vec<String>,
    deferred_items: Vec<String>,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct ProjectWorkflowSummary {
    project_id: String,
    project_root: String,
    workflow_id: String,
    title: String,
    state: String,
    node_count: usize,
    edge_count: usize,
    task_draft_count: usize,
    task_drafts: Vec<TaskDraftSummary>,
    node_session_bindings: Vec<WorkflowNodeSessionBinding>,
    node_dispatches: Vec<WorkflowNodeDispatchRecord>,
    director_reviews: Vec<WorkflowDispatchDirectorReviewRecord>,
    execution_controls: Vec<WorkflowExecutionControlRecord>,
    permission_requests: Vec<WorkflowPermissionRequestRecord>,
    execution_attempts: Vec<WorkflowExecutionAttemptRecord>,
    derived_workflow: Option<Workflow>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct ProjectBlackboard {
    project_id: String,
    project_root: String,
    workflow_id: String,
    entries: Vec<BlackboardEntry>,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct BlackboardEntry {
    entry_id: String,
    project_id: String,
    workflow_id: String,
    work_item_id: Option<String>,
    workflow_node_id: Option<String>,
    kind: BlackboardEntryKind,
    title: String,
    summary: String,
    status: String,
    source_status: Option<String>,
    source_refs: Vec<BlackboardSourceRef>,
    created_at: Option<String>,
    promotion_decision: BlackboardPromotionDecision,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum BlackboardEntryKind {
    SubagentReport,
    Risk,
    PermissionRequest,
    ToolSummary,
    MemoryCandidate,
    KnowledgeRef,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct BlackboardSourceRef {
    source_kind: String,
    source_id: String,
    label: String,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct BlackboardPromotionDecision {
    decision_id: String,
    status: String,
    target_kind: Option<String>,
    decided_by_role: Option<String>,
    decided_at: Option<String>,
    reason: String,
    audit_refs: Vec<String>,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum BlackboardCandidateState {
    CandidatePendingControlCore,
    CandidateConfirmedForFollowup,
    CandidateRejected,
    CandidateDeferred,
    CandidateDiscarded,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum BlackboardCandidateTargetKind {
    WorkflowFact,
    WorkflowRisk,
    PermissionDecision,
    AuditEvent,
    FormalMemory,
    KnowledgeReference,
    NoPromotion,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct BlackboardCandidateSourceRef {
    source_kind: String,
    source_id: String,
    label: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct BlackboardCandidateDecision {
    decision_version: i64,
    decision_id: String,
    decided_by_role: String,
    decided_by_session_id: Option<String>,
    decision_reason: String,
    decided_at: String,
    requested_state: BlackboardCandidateState,
    resulting_state: BlackboardCandidateState,
    promotion_target_blocked: bool,
    followup_required: bool,
    followup_task_ref: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct BlackboardCandidateRecord {
    record_version: i64,
    candidate_id: String,
    candidate_key: String,
    candidate_key_version: i64,
    content_fingerprint: String,
    source_entry_id: Option<String>,
    project_id: String,
    project_root: String,
    workflow_id: String,
    work_item_id: Option<String>,
    workflow_node_id: Option<String>,
    entry_kind: BlackboardEntryKind,
    target_kind: BlackboardCandidateTargetKind,
    state: BlackboardCandidateState,
    title_snapshot: String,
    summary_snapshot: String,
    source_status: Option<String>,
    source_refs: Vec<BlackboardCandidateSourceRef>,
    decision: BlackboardCandidateDecision,
    created_at: String,
    updated_at: String,
    last_seen_at: Option<String>,
    appearance_count: i64,
    superseded_by_candidate_id: Option<String>,
    audit_refs: Vec<String>,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct BlackboardCandidateAuditEvent {
    event_version: i64,
    event_id: String,
    event_type: String,
    candidate_id: String,
    candidate_key: String,
    project_id: String,
    workflow_id: String,
    actor_role: String,
    actor_session_id: Option<String>,
    before_state: Option<BlackboardCandidateState>,
    after_state: BlackboardCandidateState,
    store_revision: i64,
    reason: String,
    created_at: String,
    source_refs: Vec<BlackboardCandidateSourceRef>,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct BlackboardCandidateStoreScope {
    scope_kind: String,
    workflow_state_path: Option<String>,
    sidecar_path: Option<String>,
    project_roots: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct BlackboardCandidateStoreV1 {
    schema_version: String,
    store_version: i64,
    storage_kind: String,
    scope: BlackboardCandidateStoreScope,
    revision: i64,
    last_write_id: Option<String>,
    generated_by: String,
    created_at: String,
    updated_at: String,
    records: Vec<BlackboardCandidateRecord>,
    audit_events: Vec<BlackboardCandidateAuditEvent>,
    warnings: Vec<String>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
struct RecordBlackboardCandidateDecisionInput {
    project_id: String,
    project_root: String,
    workflow_id: String,
    candidate_key: Option<String>,
    source_entry_id: Option<String>,
    entry_kind: BlackboardEntryKind,
    target_kind: BlackboardCandidateTargetKind,
    requested_state: BlackboardCandidateState,
    reason: String,
    actor_role: String,
    actor_session_id: Option<String>,
    source_refs: Vec<BlackboardCandidateSourceRef>,
    expected_store_revision: Option<i64>,
    title_snapshot: Option<String>,
    summary_snapshot: Option<String>,
    source_status: Option<String>,
    work_item_id: Option<String>,
    workflow_node_id: Option<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct RecordBlackboardCandidateDecisionOutput {
    record: BlackboardCandidateRecord,
    audit_event: BlackboardCandidateAuditEvent,
    store_revision: i64,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct MemoryScope {
    scope_id: String,
    scope_type: String,
    user_id: Option<String>,
    project_id: Option<String>,
    workflow_id: Option<String>,
    session_id: Option<String>,
    role_ids: Vec<String>,
    document_refs: Vec<String>,
    permission_policy_ref: Option<String>,
    model_export_policy: String,
    valid_from: String,
    valid_until: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct MemorySourceRef {
    source_ref_id: String,
    source_type: String,
    source_id: Option<String>,
    source_path: Option<String>,
    source_title: Option<String>,
    anchor: Option<String>,
    source_created_at: Option<String>,
    captured_at: String,
    authority_level: String,
    sensitive_level: String,
    content_hash: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum MemoryLifecycleStatus {
    CandidateDraft,
    CandidateNeedsReview,
    CandidateConfirmed,
    CandidateRejected,
    CandidateQuarantined,
    CandidateSuperseded,
    CandidateDiscarded,
    MemoryActive,
    MemoryConflicted,
    MemoryDeprecated,
    MemoryFrozen,
    MemoryArchived,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct MemoryConflict {
    conflict_id: String,
    conflict_type: String,
    left_ref: String,
    right_ref: String,
    severity: String,
    status: String,
    summary: String,
    recommended_action: String,
    source_refs: Vec<MemorySourceRef>,
    audit_refs: Vec<MemoryAuditRef>,
    created_at: String,
    updated_at: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct MemoryAuditRef {
    audit_ref_id: String,
    audit_event_id: Option<String>,
    event_type: String,
    actor_id: String,
    actor_role: String,
    target_kind: String,
    target_id: String,
    before_status: Option<MemoryLifecycleStatus>,
    after_status: Option<MemoryLifecycleStatus>,
    reason: String,
    created_at: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct MemoryCandidateAdoptionRef {
    adopted_memory_id: String,
    adopted_version_id: String,
    adopted_audit_event_id: String,
    adopted_at: String,
    adopted_by_role: String,
    adoption_reason: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct MemoryCandidate {
    candidate_id: String,
    candidate_key: String,
    schema_version: String,
    scope: MemoryScope,
    memory_type: String,
    claim: String,
    body: String,
    source_refs: Vec<MemorySourceRef>,
    generated_by_role: String,
    generated_from: String,
    status: MemoryLifecycleStatus,
    risk_level: String,
    sensitive_level: String,
    requires_user_confirmation: bool,
    review_reason: String,
    conflicts: Vec<MemoryConflict>,
    audit_refs: Vec<MemoryAuditRef>,
    adoption: Option<MemoryCandidateAdoptionRef>,
    created_at: String,
    updated_at: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
struct MemoryRecord {
    memory_id: String,
    schema_version: String,
    record_version: i64,
    scope: MemoryScope,
    memory_type: String,
    claim: String,
    body: String,
    source_refs: Vec<MemorySourceRef>,
    status: MemoryLifecycleStatus,
    supersedes_memory_id: Option<String>,
    superseded_by_memory_id: Option<String>,
    conflict_refs: Vec<String>,
    audit_refs: Vec<MemoryAuditRef>,
    created_at: String,
    updated_at: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct MemoryVersion {
    version_id: String,
    memory_id: String,
    version_number: i64,
    change_type: String,
    change_summary: String,
    record_snapshot: MemoryRecord,
    source_refs: Vec<MemorySourceRef>,
    changed_by_role: String,
    reviewed_by: Option<String>,
    created_at: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct MemoryAuditEvent {
    audit_event_id: String,
    event_type: String,
    actor_id: String,
    actor_role: String,
    project_id: Option<String>,
    workflow_id: Option<String>,
    session_id: Option<String>,
    target_kind: String,
    target_id: Option<String>,
    before_state: Option<String>,
    after_state: Option<String>,
    reason: String,
    source_refs: Vec<MemorySourceRef>,
    status: String,
    created_at: String,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum MemoryEntityKind {
    Project,
    Workflow,
    Session,
    Role,
    KnowledgeDoc,
    Tool,
    Model,
    Harness,
    Proposal,
    MemoryRecord,
    MemoryCandidate,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum MemoryRelationKind {
    Entity,
    Temporal,
    Causal,
    Semantic,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum MemoryRelationSourceKind {
    Manual,
    FormalMemory,
    MemoryCandidate,
    Observation,
    KnowledgeDoc,
    TaskPackage,
    LlmInferred,
    SimilarityHit,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum MemoryRelationStatus {
    Candidate,
    Confirmed,
    Rejected,
    Quarantined,
    Conflicted,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum MemoryEntityAliasDecisionKind {
    ConfirmAlias,
    RejectAlias,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum MemoryEntityMergeDecisionKind {
    ConfirmMerge,
    RejectMerge,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum MemoryRelationCandidateDecisionKind {
    ConfirmRelation,
    RejectRelation,
    QuarantineRelation,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct MemoryRelationSource {
    source_kind: MemoryRelationSourceKind,
    source_id: Option<String>,
    source_path: Option<String>,
    source_title: Option<String>,
    authority_level: String,
    sensitive_level: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct MemoryEntityAlias {
    alias_id: String,
    alias: String,
    source_kind: MemoryRelationSourceKind,
    source_id: Option<String>,
    created_at: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct MemoryEntity {
    entity_id: String,
    entity_kind: MemoryEntityKind,
    canonical_key: String,
    display_name: String,
    aliases: Vec<MemoryEntityAlias>,
    source_refs: Vec<MemoryRelationSource>,
    status: String,
    created_at: String,
    updated_at: String,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct MemoryEntityRegistry {
    entities: Vec<MemoryEntity>,
    updated_at: String,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct MemoryEntityCandidate {
    candidate_id: String,
    entity_kind: MemoryEntityKind,
    display_name: String,
    normalized_key: String,
    source_kind: MemoryRelationSourceKind,
    source_id: Option<String>,
    source_path: Option<String>,
    source_title: Option<String>,
    source_refs: Vec<MemoryRelationSource>,
    confidence_kind: String,
    status: MemoryRelationStatus,
    reason: String,
    created_at: String,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct MemoryEntityMergeCandidate {
    merge_candidate_id: String,
    left_entity_candidate_id: String,
    right_entity_candidate_id: String,
    left_label: String,
    right_label: String,
    normalized_key: String,
    source_kind: MemoryRelationSourceKind,
    status: MemoryRelationStatus,
    requires_user_confirmation: bool,
    reason: String,
    created_at: String,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct MemoryRelationCandidate {
    candidate_id: String,
    relation_kind: MemoryRelationKind,
    subject_entity_id: String,
    object_entity_id: String,
    subject_label: String,
    object_label: String,
    predicate: String,
    source_kind: MemoryRelationSourceKind,
    source_refs: Vec<MemoryRelationSource>,
    confidence_kind: String,
    status: MemoryRelationStatus,
    requires_user_confirmation: bool,
    reason: String,
    created_at: String,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct MemoryRelation {
    relation_id: String,
    relation_kind: MemoryRelationKind,
    subject_entity_id: String,
    object_entity_id: String,
    subject_label: String,
    object_label: String,
    predicate: String,
    source_kind: MemoryRelationSourceKind,
    source_refs: Vec<MemoryRelationSource>,
    status: MemoryRelationStatus,
    confirmed_by: String,
    confirmation_role: String,
    confirmation_reason: String,
    created_at: String,
    updated_at: String,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct MemoryRelationAuditEvent {
    audit_event_id: String,
    event_type: String,
    actor_id: String,
    actor_role: String,
    target_kind: String,
    target_id: String,
    before_status: Option<MemoryRelationStatus>,
    after_status: Option<MemoryRelationStatus>,
    reason: String,
    created_at: String,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct MemoryEntityRelationStoreV1 {
    store_version: String,
    project_id: Option<String>,
    workflow_id: Option<String>,
    revision: i64,
    registry: MemoryEntityRegistry,
    entity_candidates: Vec<MemoryEntityCandidate>,
    merge_candidates: Vec<MemoryEntityMergeCandidate>,
    relation_candidates: Vec<MemoryRelationCandidate>,
    relations: Vec<MemoryRelation>,
    audit_events: Vec<MemoryRelationAuditEvent>,
    updated_at: String,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct MemoryEntityRelationStoreSummary {
    sidecar_name: String,
    revision: i64,
    entity_count: usize,
    entity_candidate_count: usize,
    merge_candidate_count: usize,
    relation_candidate_count: usize,
    confirmed_relation_count: usize,
    display_text: String,
    warnings: Vec<String>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
struct PreviewMemoryEntityRelationCandidatesInput {
    project_root: String,
    project_id: Option<String>,
    workflow_id: Option<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct MemoryEntityRelationPreviewOutput {
    store_revision: i64,
    entity_candidates: Vec<MemoryEntityCandidate>,
    merge_candidates: Vec<MemoryEntityMergeCandidate>,
    relation_candidates: Vec<MemoryRelationCandidate>,
    summary: MemoryEntityRelationStoreSummary,
    warnings: Vec<String>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
struct RecordMemoryEntityAliasDecisionInput {
    project_root: String,
    entity_candidate_id: String,
    decision: MemoryEntityAliasDecisionKind,
    actor_id: String,
    actor_role: String,
    reason: String,
    expected_store_revision: Option<i64>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
struct RecordMemoryEntityMergeDecisionInput {
    project_root: String,
    merge_candidate_id: String,
    decision: MemoryEntityMergeDecisionKind,
    actor_id: String,
    actor_role: String,
    confirmed_by: Option<String>,
    reason: String,
    expected_store_revision: Option<i64>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
struct RecordMemoryRelationCandidateDecisionInput {
    project_root: String,
    relation_candidate_id: String,
    decision: MemoryRelationCandidateDecisionKind,
    actor_id: String,
    actor_role: String,
    confirmed_by: Option<String>,
    reason: String,
    expected_store_revision: Option<i64>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct RecordMemoryEntityAliasDecisionOutput {
    store_revision: i64,
    entity: Option<MemoryEntity>,
    candidate: MemoryEntityCandidate,
    audit_event: MemoryRelationAuditEvent,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct RecordMemoryEntityMergeDecisionOutput {
    store_revision: i64,
    entity: Option<MemoryEntity>,
    merge_candidate: MemoryEntityMergeCandidate,
    audit_event: MemoryRelationAuditEvent,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct RecordMemoryRelationCandidateDecisionOutput {
    store_revision: i64,
    relation: Option<MemoryRelation>,
    relation_candidate: MemoryRelationCandidate,
    audit_event: MemoryRelationAuditEvent,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct MemoryRelationTaskExplanation {
    relation_id: String,
    relation_kind: MemoryRelationKind,
    linked_entity_id: String,
    linked_label: String,
    explanation: String,
    source_count: usize,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum MaturePatternCandidateStatus {
    Candidate,
    Confirmed,
    Rejected,
    Quarantined,
    ChangesRequested,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum MaturePatternDecisionKind {
    ConfirmAsFormalMemory,
    Reject,
    Quarantine,
    RequestChanges,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct MemoryClusterMemberRef {
    member_ref_id: String,
    member_kind: String,
    member_id: String,
    project_id: Option<String>,
    title: String,
    source_refs: Vec<MemorySourceRef>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct MaturePatternCandidate {
    candidate_id: String,
    pattern_kind: String,
    scope: MemoryScope,
    title: String,
    claim: String,
    body: String,
    source_refs: Vec<MemorySourceRef>,
    member_refs: Vec<MemoryClusterMemberRef>,
    signal_refs: Vec<String>,
    status: MaturePatternCandidateStatus,
    requires_user_confirmation: bool,
    review_summary: String,
    created_at: String,
    updated_at: String,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct MemoryClusterReport {
    report_id: String,
    report_kind: String,
    scope_type: String,
    title: String,
    project_ids: Vec<String>,
    member_refs: Vec<MemoryClusterMemberRef>,
    source_refs: Vec<MemorySourceRef>,
    status: String,
    staleness: String,
    display_text: String,
    created_at: String,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct MaturePatternAuditEvent {
    audit_event_id: String,
    event_type: String,
    actor_id: String,
    actor_role: String,
    target_kind: String,
    target_id: String,
    before_status: Option<MaturePatternCandidateStatus>,
    after_status: Option<MaturePatternCandidateStatus>,
    formal_memory_id: Option<String>,
    reason: String,
    created_at: String,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct MemoryPatternStoreV1 {
    store_version: String,
    project_id: Option<String>,
    workflow_id: Option<String>,
    revision: i64,
    mature_pattern_candidates: Vec<MaturePatternCandidate>,
    cluster_reports: Vec<MemoryClusterReport>,
    audit_events: Vec<MaturePatternAuditEvent>,
    updated_at: String,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct MemorySystemAcceptanceGate {
    gate_id: String,
    label: String,
    status: String,
    evidence: String,
    blocking_reason: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct MemorySystemAcceptanceSummary {
    summary_id: String,
    scope_label: String,
    gate_count: usize,
    passed_count: usize,
    blocked_count: usize,
    deferred_count: usize,
    gates: Vec<MemorySystemAcceptanceGate>,
    display_text: String,
    warnings: Vec<String>,
    created_at: String,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct MemoryPatternStoreSummary {
    sidecar_name: String,
    revision: i64,
    mature_pattern_candidate_count: usize,
    cluster_report_count: usize,
    confirmed_pattern_count: usize,
    display_text: String,
    warnings: Vec<String>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
struct PreviewMaturePatternsInput {
    project_root: String,
    project_id: Option<String>,
    workflow_id: Option<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct MaturePatternPreviewOutput {
    store_revision: i64,
    mature_pattern_candidates: Vec<MaturePatternCandidate>,
    cluster_reports: Vec<MemoryClusterReport>,
    acceptance_summary: MemorySystemAcceptanceSummary,
    summary: MemoryPatternStoreSummary,
    warnings: Vec<String>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
struct RecordMaturePatternDecisionInput {
    project_root: String,
    candidate_id: String,
    decision: MaturePatternDecisionKind,
    actor_id: String,
    actor_role: String,
    confirmed_by: Option<String>,
    reason: String,
    expected_pattern_store_revision: Option<i64>,
    expected_formal_store_revision: Option<i64>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct RecordMaturePatternDecisionOutput {
    store_revision: i64,
    candidate: MaturePatternCandidate,
    formal_memory_output: Option<CreateFormalMemoryRecordOutput>,
    audit_event: MaturePatternAuditEvent,
    acceptance_summary: MemorySystemAcceptanceSummary,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct FormalMemoryStoreV1 {
    store_version: String,
    project_id: Option<String>,
    workflow_id: Option<String>,
    revision: i64,
    records: Vec<MemoryRecord>,
    versions: Vec<MemoryVersion>,
    audit_events: Vec<MemoryAuditEvent>,
    updated_at: String,
    warnings: Vec<String>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
struct CreateFormalMemoryRecordInput {
    project_root: String,
    project_id: Option<String>,
    workflow_id: Option<String>,
    scope: MemoryScope,
    memory_type: String,
    claim: String,
    body: String,
    source_refs: Vec<MemorySourceRef>,
    actor_id: String,
    actor_role: String,
    reason: String,
    audit_event_type: Option<String>,
    expected_store_revision: Option<i64>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct CreateFormalMemoryRecordOutput {
    record: MemoryRecord,
    version: MemoryVersion,
    audit_event: MemoryAuditEvent,
    store_revision: i64,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum FormalMemoryLifecycleOperationKind {
    Revise,
    Deprecate,
    Freeze,
    Unfreeze,
    Archive,
    Merge,
    Split,
    PromoteToGlobal,
    DemoteToProject,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct FormalMemoryRevisePlan {
    claim: Option<String>,
    body: Option<String>,
    source_refs: Option<Vec<MemorySourceRef>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct FormalMemoryMergePlan {
    source_memory_ids: Vec<String>,
    target_memory_id: Option<String>,
    merged_claim: String,
    merged_body: String,
    memory_type: Option<String>,
    scope: Option<MemoryScope>,
    source_refs: Vec<MemorySourceRef>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct FormalMemorySplitRecordDraft {
    claim: String,
    body: String,
    memory_type: Option<String>,
    scope: Option<MemoryScope>,
    source_refs: Vec<MemorySourceRef>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct FormalMemorySplitPlan {
    source_memory_id: String,
    split_records: Vec<FormalMemorySplitRecordDraft>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct FormalMemoryScopeChangePlan {
    target_scope: MemoryScope,
    applicability: String,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
struct FormalMemoryLifecyclePreviewInput {
    project_root: String,
    project_id: Option<String>,
    workflow_id: Option<String>,
    operation_kind: FormalMemoryLifecycleOperationKind,
    memory_id: Option<String>,
    #[serde(default)]
    memory_ids: Vec<String>,
    revise: Option<FormalMemoryRevisePlan>,
    merge: Option<FormalMemoryMergePlan>,
    split: Option<FormalMemorySplitPlan>,
    scope_change: Option<FormalMemoryScopeChangePlan>,
    actor_id: String,
    actor_role: String,
    reason: String,
    expected_store_revision: Option<i64>,
    #[serde(default)]
    expected_record_versions: BTreeMap<String, i64>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
struct FormalMemoryLifecycleInput {
    project_root: String,
    project_id: Option<String>,
    workflow_id: Option<String>,
    operation_kind: FormalMemoryLifecycleOperationKind,
    memory_id: Option<String>,
    #[serde(default)]
    memory_ids: Vec<String>,
    revise: Option<FormalMemoryRevisePlan>,
    merge: Option<FormalMemoryMergePlan>,
    split: Option<FormalMemorySplitPlan>,
    scope_change: Option<FormalMemoryScopeChangePlan>,
    actor_id: String,
    actor_role: String,
    reason: String,
    confirmed_by: Option<String>,
    confirmation_summary: Option<String>,
    expected_store_revision: Option<i64>,
    #[serde(default)]
    expected_record_versions: BTreeMap<String, i64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct FormalMemoryRequiredApproval {
    required: bool,
    approval_kind: String,
    required_actor_role: String,
    reason: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct FormalMemoryLifecycleStatusChange {
    memory_id: String,
    before_status: MemoryLifecycleStatus,
    after_status: MemoryLifecycleStatus,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct FormalMemoryLifecycleImpactSummary {
    affected_memory_ids: Vec<String>,
    created_memory_ids: Vec<String>,
    status_changes: Vec<FormalMemoryLifecycleStatusChange>,
    created_memory_count: usize,
    new_version_count: usize,
    task_packet_eligibility_change: String,
    source_ref_count: usize,
    display_text: String,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct FormalMemoryLifecyclePreview {
    preview_id: String,
    operation_kind: FormalMemoryLifecycleOperationKind,
    store_revision: i64,
    target_memory_ids: Vec<String>,
    impact: FormalMemoryLifecycleImpactSummary,
    required_approval: FormalMemoryRequiredApproval,
    before_records: Vec<MemoryRecord>,
    proposed_records: Vec<MemoryRecord>,
    display_text: String,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct FormalMemoryLifecycleOutput {
    operation_id: String,
    preview: FormalMemoryLifecyclePreview,
    records: Vec<MemoryRecord>,
    versions: Vec<MemoryVersion>,
    audit_event: MemoryAuditEvent,
    store_revision: i64,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct FormalMemoryStoreSummary {
    sidecar_name: String,
    revision: i64,
    record_count: usize,
    active_count: usize,
    non_active_count: usize,
    version_count: usize,
    audit_event_count: usize,
    recent_audit_event: Option<MemoryAuditEvent>,
    warnings: Vec<String>,
    display_text: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct MemoryCandidateStoreV1 {
    store_version: String,
    project_id: Option<String>,
    workflow_id: Option<String>,
    revision: i64,
    candidates: Vec<MemoryCandidate>,
    events: Vec<MemoryAuditRef>,
    updated_at: String,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
struct CreateMemoryCandidateInput {
    project_root: String,
    project_id: Option<String>,
    workflow_id: Option<String>,
    scope: MemoryScope,
    memory_type: String,
    claim: String,
    body: String,
    source_refs: Vec<MemorySourceRef>,
    generated_by_role: String,
    generated_from: String,
    risk_level: String,
    sensitive_level: String,
    requires_user_confirmation: bool,
    review_reason: String,
    expected_store_revision: Option<i64>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct CreateMemoryCandidateOutput {
    candidate: MemoryCandidate,
    audit_event: MemoryAuditRef,
    store_revision: i64,
    warnings: Vec<String>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
struct RecordMemoryCandidateDecisionInput {
    project_root: String,
    candidate_key: String,
    requested_status: MemoryLifecycleStatus,
    reason: String,
    actor_id: String,
    actor_role: String,
    expected_store_revision: Option<i64>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct RecordMemoryCandidateDecisionOutput {
    candidate: MemoryCandidate,
    audit_event: MemoryAuditRef,
    store_revision: i64,
    warnings: Vec<String>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
struct AdoptMemoryCandidateInput {
    project_root: String,
    candidate_key: String,
    actor_id: String,
    actor_role: String,
    adoption_reason: String,
    expected_candidate_store_revision: Option<i64>,
    expected_formal_store_revision: Option<i64>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct AdoptMemoryCandidateOutput {
    candidate_key: String,
    candidate_status: MemoryLifecycleStatus,
    record: MemoryRecord,
    version: MemoryVersion,
    audit_event: MemoryAuditEvent,
    adoption: MemoryCandidateAdoptionRef,
    candidate_store_revision: i64,
    formal_store_revision: i64,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum ObservationStatus {
    Recorded,
    CandidateCreated,
    Ignored,
    Quarantined,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct ObservationSourceRef {
    source_ref_id: String,
    source_kind: String,
    source_id: String,
    project_id: Option<String>,
    workflow_id: Option<String>,
    session_id: Option<String>,
    file_path: Option<String>,
    evidence_ref: Option<String>,
    summary: String,
    sensitive_level: String,
    created_at: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct ObservationAuditRef {
    audit_ref_id: String,
    event_type: String,
    actor_id: String,
    actor_role: String,
    target_kind: String,
    target_id: String,
    before_status: Option<ObservationStatus>,
    after_status: Option<ObservationStatus>,
    reason: String,
    created_at: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct ObservationRecord {
    observation_id: String,
    observation_key: String,
    schema_version: String,
    project_id: Option<String>,
    workflow_id: Option<String>,
    scope: MemoryScope,
    observation_type: String,
    summary: String,
    source_refs: Vec<ObservationSourceRef>,
    status: ObservationStatus,
    generated_by_role: String,
    actor_id: String,
    risk_level: String,
    sensitive_level: String,
    candidate_key: Option<String>,
    audit_refs: Vec<ObservationAuditRef>,
    created_at: String,
    updated_at: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct ObservationStoreV1 {
    store_version: String,
    project_id: Option<String>,
    workflow_id: Option<String>,
    revision: i64,
    observations: Vec<ObservationRecord>,
    events: Vec<ObservationAuditRef>,
    updated_at: String,
    warnings: Vec<String>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
struct CreateObservationInput {
    project_root: String,
    project_id: Option<String>,
    workflow_id: Option<String>,
    scope: MemoryScope,
    observation_type: String,
    summary: String,
    source_refs: Vec<ObservationSourceRef>,
    generated_by_role: String,
    actor_id: String,
    risk_level: String,
    sensitive_level: String,
    reason: String,
    expected_store_revision: Option<i64>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct CreateObservationOutput {
    observation: ObservationRecord,
    audit_event: ObservationAuditRef,
    store_revision: i64,
    warnings: Vec<String>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
struct CreateMemoryCandidateFromObservationInput {
    project_root: String,
    observation_key: String,
    actor_id: String,
    actor_role: String,
    memory_type: String,
    claim: String,
    body: String,
    review_reason: String,
    requires_user_confirmation: bool,
    expected_observation_store_revision: Option<i64>,
    expected_candidate_store_revision: Option<i64>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct CreateMemoryCandidateFromObservationOutput {
    observation: ObservationRecord,
    candidate: MemoryCandidate,
    observation_audit_event: ObservationAuditRef,
    candidate_audit_event: MemoryAuditRef,
    observation_store_revision: i64,
    candidate_store_revision: i64,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct ObservationStoreSummary {
    sidecar_name: String,
    revision: i64,
    observation_count: usize,
    recorded_count: usize,
    candidate_created_count: usize,
    ignored_count: usize,
    quarantined_count: usize,
    recent_audit_event: Option<ObservationAuditRef>,
    recent_candidate_key: Option<String>,
    warnings: Vec<String>,
    display_text: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct MemoryCaptureSourceRef {
    source_ref_id: String,
    source_type: String,
    source_id: String,
    project_id: Option<String>,
    workflow_id: Option<String>,
    workflow_node_id: Option<String>,
    run_unit_id: Option<String>,
    product_command_id: Option<String>,
    product_attempt_id: Option<String>,
    runtime_log_ref: Option<String>,
    audit_ref_id: Option<String>,
    readback_ref: Option<String>,
    task_package_ref: Option<String>,
    memory_packet_ref: Option<String>,
    evidence_ref: Option<String>,
    summary: String,
    sensitive_level: String,
    created_at: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct MemoryCaptureCandidateDraft {
    memory_type: String,
    claim: String,
    body: String,
    review_reason: String,
    requires_user_confirmation: bool,
    actor_role: String,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
struct CaptureMemoryEventInput {
    project_root: String,
    project_id: Option<String>,
    workflow_id: Option<String>,
    workflow_node_id: Option<String>,
    run_unit_id: Option<String>,
    product_command_id: Option<String>,
    product_attempt_id: Option<String>,
    runtime_log_ref: Option<String>,
    audit_refs: Vec<String>,
    readback_ref: Option<String>,
    task_package_ref: Option<String>,
    memory_packet_ref: Option<String>,
    scope: MemoryScope,
    source_type: String,
    source_refs: Vec<MemoryCaptureSourceRef>,
    summary: String,
    evidence_summary: String,
    sensitivity: String,
    candidate_policy: String,
    generated_by_role: String,
    actor_id: String,
    risk_level: String,
    reason: String,
    candidate: Option<MemoryCaptureCandidateDraft>,
    expected_capture_store_revision: Option<i64>,
    expected_observation_store_revision: Option<i64>,
    expected_candidate_store_revision: Option<i64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct MemoryCaptureEventRecord {
    capture_event_id: String,
    event_key: String,
    schema_version: String,
    source_type: String,
    source_ref_id: String,
    project_id: Option<String>,
    workflow_id: Option<String>,
    workflow_node_id: Option<String>,
    run_unit_id: Option<String>,
    product_command_id: Option<String>,
    product_attempt_id: Option<String>,
    runtime_log_ref: Option<String>,
    audit_refs: Vec<String>,
    readback_ref: Option<String>,
    task_package_ref: Option<String>,
    memory_packet_ref: Option<String>,
    summary: String,
    evidence_summary: String,
    sensitivity: String,
    candidate_policy: String,
    blocked_reason: Option<String>,
    observation_id: Option<String>,
    candidate_key: Option<String>,
    created_by: String,
    created_at: String,
    updated_at: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct MemoryCaptureStoreV1 {
    store_version: String,
    project_id: Option<String>,
    workflow_id: Option<String>,
    revision: i64,
    events: Vec<MemoryCaptureEventRecord>,
    updated_at: String,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct CaptureMemoryEventOutput {
    capture_event: MemoryCaptureEventRecord,
    observation: Option<ObservationRecord>,
    candidate: Option<MemoryCandidate>,
    observation_store_revision: Option<i64>,
    candidate_store_revision: Option<i64>,
    capture_store_revision: i64,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum MemoryLintFindingSeverity {
    Blocking,
    NeedsReview,
    Info,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum MemoryLintFindingStatus {
    Open,
    Acknowledged,
    Resolved,
    Dismissed,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum MemoryLintFindingType {
    DuplicateClaim,
    ClaimConflict,
    SourcePermissionRevoked,
    AuthoritySuperseded,
    StaleMemory,
    MissingSource,
    CandidateConflictsWithActiveMemory,
    EntityDrift,
    RelationSourceRevoked,
    SensitiveExportRisk,
    PrivateSourceRisk,
    DerivedIndexStale,
    MaturePatternSignal,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum MemoryLintRunIntent {
    CandidateAdoptionGuard,
    TaskPacketGuard,
    MaintenancePreview,
    MaintenanceRun,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum MemoryLintRunStatus {
    Succeeded,
    Blocked,
    Failed,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct MemoryLintFinding {
    finding_id: String,
    schema_version: String,
    finding_type: MemoryLintFindingType,
    severity: MemoryLintFindingSeverity,
    status: MemoryLintFindingStatus,
    source_kind: String,
    source_id: String,
    target_memory_id: Option<String>,
    target_candidate_key: Option<String>,
    scope_type: Option<String>,
    memory_type: Option<String>,
    claim: Option<String>,
    summary: String,
    recommended_action: String,
    evidence_refs: Vec<MemorySourceRef>,
    audit_event_id: Option<String>,
    created_at: String,
    updated_at: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct MemoryLintRunRecord {
    run_id: String,
    lint_intent: MemoryLintRunIntent,
    actor_id: String,
    actor_role: String,
    finding_ids: Vec<String>,
    blocking_count: usize,
    status: MemoryLintRunStatus,
    reason: String,
    #[serde(default)]
    report_id: Option<String>,
    created_at: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum MemoryMaintenanceCheckKind {
    ExpiredOrStale,
    SourceIntegrity,
    DuplicateAndConflict,
    EntityRelationDrift,
    PermissionRevocation,
    SensitiveExportRisk,
    IndexStatus,
    MaturePatternSignal,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct MemoryMaintenanceCheckSummary {
    check_kind: MemoryMaintenanceCheckKind,
    checked_count: usize,
    finding_count: usize,
    blocking_count: usize,
    needs_review_count: usize,
    info_count: usize,
    display_text: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct MemoryMaintenanceRecommendation {
    recommendation_id: String,
    severity: MemoryLintFindingSeverity,
    target_kind: String,
    target_id: Option<String>,
    action_label: String,
    display_text: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct MemoryMaintenanceIndexStatus {
    status: String,
    formal_store_revision: i64,
    lint_store_revision: i64,
    entity_relation_store_revision: i64,
    checked_at: String,
    display_text: String,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct MemoryMaintenanceReport {
    report_id: String,
    run_id: String,
    checked_memory_count: usize,
    checked_candidate_count: usize,
    checked_observation_count: usize,
    checked_relation_count: usize,
    open_count: usize,
    blocking_count: usize,
    needs_review_count: usize,
    info_count: usize,
    check_summaries: Vec<MemoryMaintenanceCheckSummary>,
    recommendations: Vec<MemoryMaintenanceRecommendation>,
    index_status: MemoryMaintenanceIndexStatus,
    display_text: String,
    warnings: Vec<String>,
    created_at: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct MemoryLintStoreV1 {
    store_version: String,
    project_id: Option<String>,
    workflow_id: Option<String>,
    revision: i64,
    findings: Vec<MemoryLintFinding>,
    runs: Vec<MemoryLintRunRecord>,
    #[serde(default)]
    maintenance_reports: Vec<MemoryMaintenanceReport>,
    updated_at: String,
    warnings: Vec<String>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
struct MemoryLintRunInput {
    project_root: String,
    project_id: Option<String>,
    workflow_id: Option<String>,
    actor_id: String,
    actor_role: String,
    lint_intent: MemoryLintRunIntent,
    candidate_key: Option<String>,
    task_id: Option<String>,
    revoked_source_ids: Vec<String>,
    expected_formal_store_revision: Option<i64>,
    expected_candidate_store_revision: Option<i64>,
    expected_lint_store_revision: Option<i64>,
    dry_run: Option<bool>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct MemoryLintRunOutput {
    store: MemoryLintStoreV1,
    run: MemoryLintRunRecord,
    report: Option<MemoryMaintenanceReport>,
    new_findings: Vec<MemoryLintFinding>,
    blocking_count: usize,
    open_count: usize,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct MemoryLintStoreSummary {
    sidecar_name: String,
    revision: i64,
    finding_count: usize,
    open_count: usize,
    blocking_count: usize,
    needs_review_count: usize,
    info_count: usize,
    recent_run: Option<MemoryLintRunRecord>,
    recent_maintenance_report: Option<MemoryMaintenanceReport>,
    display_text: String,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum TaskMemoryPacketExclusionReason {
    CandidateUnconfirmed,
    PermissionBlocked,
    Conflicted,
    Stale,
    ModelExportBlocked,
    TokenLimit,
    NotRelevant,
    StatusNotActive,
    ObservationNotFormalMemory,
    KnowledgeHitNotFormalMemory,
    LlmSummaryNotFormalMemory,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
struct TaskMemoryPacketBuildInput {
    project_root: String,
    project_id: Option<String>,
    workflow_id: Option<String>,
    task_id: Option<String>,
    role_id: String,
    task_goal: String,
    retrieval_intent: String,
    target_model_id: Option<String>,
    model_context_policy: String,
    max_memory_items: usize,
    max_estimated_tokens: usize,
    expected_formal_store_revision: Option<i64>,
    expected_candidate_store_revision: Option<i64>,
    expected_observation_store_revision: Option<i64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct TaskMemoryPacketItem {
    memory_id: String,
    memory_type: String,
    scope_type: String,
    claim: String,
    body: String,
    source_refs: Vec<MemorySourceRef>,
    retrieval_reason: String,
    #[serde(default)]
    relation_explanations: Vec<MemoryRelationTaskExplanation>,
    estimated_tokens: usize,
    model_export_policy: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct TaskMemoryPacketExcludedItem {
    source_kind: String,
    source_id: String,
    claim: Option<String>,
    reason: TaskMemoryPacketExclusionReason,
    detail: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct TaskMemoryPacketReviewMaterial {
    source_kind: String,
    source_id: String,
    title: String,
    reason: TaskMemoryPacketExclusionReason,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct TaskMemoryPacketPreview {
    packet_id: String,
    schema_version: String,
    project_id: Option<String>,
    workflow_id: Option<String>,
    task_id: Option<String>,
    role_id: String,
    retrieval_intent: String,
    included_memories: Vec<TaskMemoryPacketItem>,
    excluded_items: Vec<TaskMemoryPacketExcludedItem>,
    review_materials: Vec<TaskMemoryPacketReviewMaterial>,
    estimated_tokens: usize,
    max_estimated_tokens: usize,
    generated_at: String,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct TaskMemoryPacketBuildOutput {
    preview: TaskMemoryPacketPreview,
    formal_store_revision: i64,
    candidate_store_revision: i64,
    observation_store_revision: i64,
    lint_store_revision: i64,
    entity_relation_store_revision: i64,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct TaskPackageMemoryPacketStoreRevisions {
    formal_store_revision: i64,
    candidate_store_revision: i64,
    observation_store_revision: i64,
    lint_store_revision: Option<i64>,
    #[serde(default)]
    entity_relation_store_revision: Option<i64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct TaskPackageMemoryPacketSnapshot {
    snapshot_id: String,
    schema_version: String,
    source_packet_id: String,
    project_id: Option<String>,
    workflow_id: Option<String>,
    work_item_id: String,
    task_package_artifact_id: Option<String>,
    role_id: String,
    retrieval_intent: String,
    included_memories: Vec<TaskMemoryPacketItem>,
    excluded_items: Vec<TaskMemoryPacketExcludedItem>,
    review_materials: Vec<TaskMemoryPacketReviewMaterial>,
    store_revisions: TaskPackageMemoryPacketStoreRevisions,
    estimated_tokens: usize,
    max_estimated_tokens: usize,
    fingerprint: String,
    generated_at: String,
    stale: bool,
    stale_reasons: Vec<String>,
    warnings: Vec<String>,
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct TaskPackageMemoryInjectionAudit {
    event_type: String,
    work_item_id: String,
    snapshot_id: String,
    included_count: usize,
    excluded_count: usize,
    reason: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct TaskPackageMemoryInjectionSummary {
    snapshot_id: Option<String>,
    included_count: usize,
    excluded_count: usize,
    review_material_count: usize,
    stale: bool,
    stale_reasons: Vec<String>,
    display_text: String,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct Workflow {
    workflow_id: String,
    project_id: String,
    title: String,
    source_proposal_id: Option<String>,
    status: String,
    view_mode: Option<String>,
    created_by_role: Option<String>,
    owner_role: Option<String>,
    current_stage: Option<String>,
    run_check_status: String,
    risk_level: Option<String>,
    created_at: Option<String>,
    updated_at: Option<String>,
    nodes: Vec<WorkflowNode>,
    task_packages: Vec<TaskPackage>,
    ledger_entries: Vec<WorkflowLedgerEntry>,
    subagent_reports: Vec<SubagentReport>,
    review_results: Vec<ReviewResult>,
    exceptions: Vec<WorkflowException>,
    result_summary: WorkflowResultSummaryReadModel,
    interface_boundaries: WorkflowInterfaceBoundaries,
    state_machine: WorkflowStateMachineSummary,
    acceptance_scenarios: Vec<WorkflowAcceptanceScenario>,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct WorkflowNode {
    workflow_node_id: String,
    workflow_id: String,
    node_type: String,
    title: String,
    assigned_role: Option<String>,
    assigned_session_id: Option<String>,
    status: String,
    task_package_id: Option<String>,
    depends_on: Vec<String>,
    harness_requirements: Vec<String>,
    review_requirements: Vec<String>,
    acceptance_criteria: Vec<String>,
    created_at: Option<String>,
    updated_at: Option<String>,
    missing_fields: Vec<String>,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct WorkflowRunCheck {
    project_root: String,
    workflow_id: Option<String>,
    status: String,
    checks: Vec<WorkflowRunCheckItem>,
    blocked_reasons: Vec<String>,
    warnings: Vec<String>,
    evidence_completeness: String,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct WorkflowRunCheckItem {
    check_id: String,
    label: String,
    status: String,
    severity: String,
    reason: String,
    source_ref: Option<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct TaskPackage {
    task_package_id: String,
    workflow_id: String,
    workflow_node_id: String,
    project_id: String,
    target_session_id: Option<String>,
    target_role: Option<String>,
    task_goal: Option<String>,
    allowed_read_scope: Vec<String>,
    allowed_write_scope: Vec<String>,
    available_skills: Vec<String>,
    available_knowledge_refs: Vec<String>,
    available_memory_refs: Vec<String>,
    callable_tool_capabilities: Vec<String>,
    model_id: Option<String>,
    harness_requirements: Vec<String>,
    forbidden_actions: Vec<String>,
    acceptance_criteria: Vec<String>,
    report_format: Vec<String>,
    timeout_policy: Option<String>,
    failure_policy: Option<String>,
    version: i64,
    stale: bool,
    stale_reasons: Vec<String>,
    missing_fields: Vec<String>,
    export_includes_internal_audit: bool,
    memory_injection_summary: TaskPackageMemoryInjectionSummary,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct WorkflowLedgerEntry {
    ledger_entry_id: String,
    workflow_id: String,
    workflow_node_id: Option<String>,
    entry_type: String,
    actor_role: Option<String>,
    actor_session_id: Option<String>,
    summary: String,
    source_refs: Vec<String>,
    tool_call_refs: Vec<String>,
    audit_refs: Vec<String>,
    risk_flags: Vec<String>,
    created_at: Option<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct SubagentReport {
    report_id: String,
    workflow_id: String,
    workflow_node_id: Option<String>,
    actor_role: Option<String>,
    executed_what: String,
    changed_what: String,
    summary: String,
    evidence_refs: Vec<String>,
    open_issues: Vec<String>,
    permission_requests: Vec<String>,
    direction_risks: Vec<String>,
    follow_up_suggestions: Vec<String>,
    acceptance_status: String,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct ReviewResult {
    review_id: String,
    workflow_id: String,
    workflow_node_id: Option<String>,
    reviewer_role: Option<String>,
    report_id: Option<String>,
    accepted_fact_ids: Vec<String>,
    observation_ids: Vec<String>,
    result: String,
    summary: String,
    evidence_refs: Vec<String>,
    requires_director_confirmation: bool,
    can_complete_node: bool,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct WorkflowException {
    exception_id: String,
    workflow_id: String,
    workflow_node_id: Option<String>,
    exception_type: String,
    summary: String,
    status: String,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct WorkflowInterfaceBoundaries {
    proposal_interface: InterfaceBoundary,
    memory_candidate_interface: InterfaceBoundary,
    knowledge_refs_interface: InterfaceBoundary,
    tool_capability_registry: InterfaceBoundary,
    model_pool_selector: InterfaceBoundary,
    harness_requirement_provider: InterfaceBoundary,
    audit_refs_interface: InterfaceBoundary,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct InterfaceBoundary {
    interface_id: String,
    status: String,
    allowed: Vec<String>,
    blocked: Vec<String>,
    source_refs: Vec<String>,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct WorkflowStateMachineSummary {
    workflow_allowed_transitions: Vec<String>,
    workflow_rejected_transitions: Vec<String>,
    node_allowed_transitions: Vec<String>,
    node_rejected_transitions: Vec<String>,
    completion_gate: DirectorCompletionGate,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct DirectorCompletionGate {
    can_complete: bool,
    required: Vec<String>,
    missing: Vec<String>,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct WorkflowAcceptanceScenario {
    scenario_id: String,
    title: String,
    status: String,
    expected: Vec<String>,
    evidence_refs: Vec<String>,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct TaskDraftSummary {
    work_item_id: String,
    workflow_id: String,
    title: String,
    state: String,
    assigned_role_id: Option<String>,
    current_node_id: Option<String>,
    next_states: Vec<String>,
    next_action_label: Option<String>,
    artifact_type: Option<String>,
    artifact_path: Option<String>,
    recent_audit_events: Vec<AuditEventSummary>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct AuditEventSummary {
    event_id: String,
    event_type: String,
    before_state: Option<String>,
    after_state: Option<String>,
    created_at: Option<String>,
    reason: Option<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct WorkflowNodeSessionBinding {
    binding_id: String,
    project_id: String,
    workflow_id: String,
    node_id: String,
    work_item_id: Option<String>,
    agent_type: String,
    adapter_id: String,
    native_thread_id: String,
    native_rollout_path: Option<String>,
    session_title: String,
    session_updated_at_ms: Option<i64>,
    rollout_exists: bool,
    project_binding_source: String,
    binding_source: String,
    binding_mode: String,
    lifecycle: String,
    created_at_ms: i64,
    updated_at_ms: i64,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct WorkflowNodeDispatchRecord {
    dispatch_id: String,
    project_id: String,
    workflow_id: String,
    node_id: String,
    work_item_id: String,
    binding_id: String,
    native_thread_id: String,
    prompt_preview: String,
    prompt_kind: String,
    memory_packet_snapshot_id: Option<String>,
    memory_packet_fingerprint: Option<String>,
    plan_authorization_id: Option<String>,
    authorization_check: Option<AutoDispatchGuardResult>,
    offline_role_dispatch: Option<OfflineRoleDispatchRequest>,
    user_reviewed_instruction: Option<WorkflowUserReviewedInstruction>,
    state: String,
    started_at_ms: Option<i64>,
    ended_at_ms: Option<i64>,
    exit_code: Option<i32>,
    last_message_path: Option<String>,
    last_message_summary: Option<String>,
    transcript_event_count: Option<usize>,
    transcript_target_hits: Option<usize>,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct WorkflowDispatchDirectorReviewRecord {
    review_id: String,
    project_id: String,
    workflow_id: String,
    work_item_id: String,
    dispatch_id: String,
    reviewer_role: String,
    decision: String,
    summary: String,
    evidence_refs: Vec<String>,
    handoff_refs: Vec<String>,
    created_at: String,
    updated_at: String,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct WorkflowUserReviewedInstruction {
    instruction_id: String,
    summary: String,
    objective: String,
    execution_cwd: String,
    sandbox_mode: String,
    allowed_write_roots: Vec<String>,
    allowed_reads: Vec<String>,
    allowed_writes: Vec<String>,
    forbidden_actions: Vec<String>,
    required_return: Vec<String>,
    approval_state: String,
    preview_markdown: String,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct WorkflowExecutionControlRecord {
    control_id: String,
    project_id: String,
    workflow_id: String,
    work_item_id: String,
    control_state: String,
    long_task_state: String,
    retry_count: usize,
    max_retries: usize,
    timeout_seconds: Option<i64>,
    cancel_requested_at: Option<String>,
    failure_reason: Option<String>,
    user_reviewed_instruction: Option<WorkflowUserReviewedInstruction>,
    audit_event_types: Vec<String>,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct WorkflowPermissionRequestRecord {
    request_id: String,
    project_id: String,
    workflow_id: String,
    work_item_id: String,
    dispatch_id: Option<String>,
    permission_kind: String,
    reason: String,
    status: String,
    requested_at: String,
    decided_at: Option<String>,
    decision: Option<String>,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct WorkflowExecutionAttemptRecord {
    attempt_id: String,
    project_id: String,
    workflow_id: String,
    work_item_id: String,
    dispatch_id: Option<String>,
    attempt_no: usize,
    state: String,
    started_at: Option<String>,
    ended_at: Option<String>,
    failure_reason: Option<String>,
    retry_scheduled_at: Option<String>,
    timed_out_at: Option<String>,
    cancel_requested_at: Option<String>,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct WorkflowStateSnapshot {
    exists: bool,
    path: String,
    schema_version: Option<String>,
    workflow_version: Option<i64>,
    workspace_id: Option<String>,
    updated_at: Option<String>,
    initialized: bool,
    counts: WorkflowStateCounts,
    project_workflows: Vec<ProjectWorkflowSummary>,
    project_blackboards: Vec<ProjectBlackboard>,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct WorkflowStateMutationResult {
    message: String,
    path: String,
    backup_path: Option<String>,
    audit_event_id: String,
    first_initialize: bool,
    snapshot: WorkflowStateSnapshot,
}

#[derive(Deserialize)]
struct PathRequest {
    path: String,
}

#[derive(Deserialize)]
struct TaskDraftRequest {
    project_root: String,
    title: String,
    objective: String,
    assigned_role: Option<String>,
}

#[derive(Deserialize)]
struct TaskPackagePreviewRequest {
    project_root: String,
    work_item_id: String,
}

#[derive(Deserialize)]
struct TaskPackageFieldsUpdateRequest {
    project_root: String,
    work_item_id: String,
    fields: TaskPackageFieldsInput,
}

#[derive(Deserialize)]
struct TaskPackageDispatchFieldsCorrectionRequest {
    project_root: String,
    work_item_id: String,
    fields: TaskPackageFieldsInput,
}

#[derive(Deserialize)]
struct TaskPackageFileGenerationRequest {
    project_root: String,
    work_item_id: String,
}

#[derive(Deserialize)]
struct TaskPackageDispatchReadinessRequest {
    project_root: String,
    work_item_id: String,
}

#[derive(Deserialize)]
struct WorkflowRunCheckRequest {
    project_root: String,
    workflow_id: Option<String>,
}

#[derive(Deserialize)]
struct WorkItemStateUpdateRequest {
    project_root: String,
    work_item_id: String,
    next_state: String,
}

#[derive(Deserialize)]
struct WorkflowNodeSessionBindRequest {
    project_root: String,
    node_id: String,
    work_item_id: Option<String>,
    thread_id: String,
}

#[derive(Deserialize)]
struct WorkflowNodeSessionUnbindRequest {
    project_root: String,
    binding_id: String,
}

#[derive(Deserialize)]
struct WorkflowNodeDispatchPrepareRequest {
    project_root: String,
    node_id: String,
    work_item_id: String,
    prompt_kind: String,
    user_reviewed_instruction: Option<UserReviewedInstructionInput>,
}

#[derive(Deserialize)]
struct WorkflowNodeDispatchExecuteRequest {
    project_root: String,
    node_id: String,
    work_item_id: String,
    prompt_kind: String,
    user_reviewed_instruction: Option<UserReviewedInstructionInput>,
}

#[derive(Deserialize)]
struct WorkflowNodeDispatchReadbackRequest {
    project_root: String,
    dispatch_id: String,
}

#[derive(Deserialize)]
struct WorkflowDispatchDirectorReviewRequest {
    project_root: String,
    work_item_id: String,
    dispatch_id: String,
    decision: String,
    summary: String,
}

#[derive(Deserialize)]
struct WorkflowPermissionDecisionRequest {
    project_root: String,
    work_item_id: String,
    request_id: String,
    decision: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
struct OfflineRoleDispatchRequest {
    project_root: String,
    work_item_id: String,
    target_role_id: String,
    target_role_label: String,
    task_title: String,
    objective: String,
    execution_cwd: String,
    allowed_reads: Vec<String>,
    allowed_writes: Vec<String>,
    forbidden_actions: Vec<String>,
    acceptance_criteria: Vec<String>,
    timeout_seconds: i64,
    required_return: Vec<String>,
    raw_block: String,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
struct OfflineRoleResultHandoffRequest {
    project_root: String,
    work_item_id: String,
    dispatch_id: String,
    target_role_id: String,
    summary: String,
    markdown: String,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
struct OfflineDirectorReviewRequest {
    project_root: String,
    work_item_id: String,
    dispatch_id: String,
    decision: String,
    summary: String,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
struct UserReviewedInstructionInput {
    instruction_id: String,
    summary: String,
    objective: String,
    execution_cwd: String,
    sandbox_mode: String,
    allowed_write_roots: Vec<String>,
    allowed_reads: Vec<String>,
    allowed_writes: Vec<String>,
    forbidden_actions: Vec<String>,
    timeout_seconds: i64,
    max_retries: i64,
    required_return: Vec<String>,
    prompt_preview: Option<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct WorkflowNodeDispatchResult {
    message: String,
    path: String,
    backup_path: Option<String>,
    audit_event_id: String,
    product_command_boundary: ProductCommandBoundary,
    dispatch: WorkflowNodeDispatchRecord,
    snapshot: WorkflowStateSnapshot,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct ProductCommandBoundary {
    boundary_version: i64,
    command_name: String,
    command_family: String,
    boundary_kind: String,
    h5_unified_product_command: bool,
    deprecated: bool,
    product_routing_allows_real_execution: bool,
    legacy_path_may_have_real_side_effects: bool,
    replacement_command: Option<String>,
    reason: String,
    warnings: Vec<String>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
struct WorkflowMachineRunRequest {
    project_root: String,
    work_item_id: String,
    objective: String,
    execution_root: Option<String>,
    max_rounds: i64,
    timeout_seconds_per_step: i64,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct WorkflowMachineRunStepRecord {
    step_id: String,
    role_id: String,
    role_label: String,
    node_id: String,
    native_thread_id: String,
    state: String,
    exit_code: Option<i32>,
    last_message_summary: Option<String>,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct WorkflowMachineRunResult {
    message: String,
    path: String,
    backup_path: Option<String>,
    audit_event_id: String,
    product_command_boundary: ProductCommandBoundary,
    run_id: String,
    final_state: String,
    rounds_completed: usize,
    steps: Vec<WorkflowMachineRunStepRecord>,
    snapshot: WorkflowStateSnapshot,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct WorkflowNodeDispatchExecutionOptions {
    readback_stats: Option<CodexDispatchReadbackStats>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CodexResumeRequestOptions {
    prompt_kind: String,
    execution_cwd: Option<PathBuf>,
    sandbox_mode: Option<String>,
    allowed_write_roots: Vec<PathBuf>,
    timeout_seconds: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CodexResumeRunResult {
    exit_code: i32,
    timed_out: bool,
    stderr_summary: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CodexDispatchReadbackStats {
    transcript_event_count: usize,
    transcript_target_hits: usize,
}

#[derive(Deserialize, Clone)]
struct TaskPackageFieldsInput {
    task_name: String,
    assigned_line: String,
    background: Vec<String>,
    goals: Vec<String>,
    allowed_read: Vec<String>,
    allowed_write: Vec<String>,
    forbidden_actions: Vec<String>,
    acceptance_criteria: Vec<String>,
    required_return: Vec<String>,
    review_focus: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct TaskPackagePreview {
    project_root: String,
    workflow_id: String,
    work_item_id: String,
    artifact_id: Option<String>,
    markdown: String,
    memory_injection_summary: TaskPackageMemoryInjectionSummary,
    warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct TaskPackageFileGenerationResult {
    message: String,
    file_path: String,
    workflow_state_path: String,
    backup_path: String,
    audit_event_id: String,
    memory_injection_summary: TaskPackageMemoryInjectionSummary,
    snapshot: WorkflowStateSnapshot,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct TaskPackageDispatchReadiness {
    project_root: String,
    workflow_id: String,
    work_item_id: String,
    artifact_id: Option<String>,
    artifact_path: Option<String>,
    status: String,
    blocking_reasons: Vec<String>,
    warnings: Vec<String>,
    can_generate_next_version: bool,
    memory_injection_summary: TaskPackageMemoryInjectionSummary,
    authorization_check: Option<AutoDispatchGuardResult>,
}
