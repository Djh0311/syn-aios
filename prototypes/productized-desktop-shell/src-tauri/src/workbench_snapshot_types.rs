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
    // 工作台绑过工作流节点的会话标记（智能体页据此打「工作台任务」徽标）。默认 false·serde default 旧路径零改；
    // 只有 load_codex_session_page 合并 store 绑定会话那步才置 true（判据=store 绑定硬信号·不靠标题猜）。
    #[serde(default)]
    workbench_bound: bool,
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
    k3_b1_recovery: k3_b1_recovery::K3B1RecoveryReadModel,
    operation_control: operation_control::OperationControlReadModel,
    page_read_model_inventory: page_read_model::WorkbenchPageReadModelInventory,
    diagnostic_summary: DiagnosticSummary,
    diagnostics: Diagnostics,
}
