use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

mod blackboard_candidate_store;
pub mod codex_db;
mod codex_local_runner;
mod codex_transcript;
mod control_core;
mod formal_memory_lifecycle;
mod formal_memory_store;
mod h4_execution_boundary;
mod h5_project_dispatch_bridge;
mod mature_pattern_governance;
mod mature_pattern_store;
pub mod mcp;
mod memory_candidate_store;
mod memory_capture_bus;
mod memory_consistency;
mod memory_entity_relation_governance;
mod memory_entity_relation_store;
mod memory_lint_engine;
mod memory_lint_store;
mod observation_store;
mod page_read_model;
mod plan_authorization_store;
mod project_consultation_proposal_store;
mod project_workflow_automation;
mod real_execution_command;
mod runtime_log_store;
mod runtime_session_attention;
mod session_continuation_store;
mod task_memory_injection;
mod task_memory_packet_builder;
mod workbench_sqlite_apply;
mod workbench_sqlite_dual_write;
mod workbench_sqlite_exporter;
mod workbench_sqlite_importer;
mod workbench_sqlite_observation_period;
mod workbench_sqlite_preflight;
mod workbench_sqlite_production_apply;
mod workbench_sqlite_read_cut;
mod workbench_sqlite_schema;
mod workbench_sqlite_snapshot_apply;
mod workbench_sqlite_stop_write;
mod workbench_sqlite_transaction_acceptance;
mod worker_protocol;
mod workflow_audit;
mod workflow_read_model;
mod workflow_state_store;
pub use mcp::run_mcp_server_cli;

#[derive(Clone)]
struct AppState {
    index_path: PathBuf,
    tasks_path: PathBuf,
    workflow_state_path: PathBuf,
}

// Type definitions live in src/types.rs for the conservative no-behavior split.
include!("types.rs");

trait CodexResumeRunner {
    fn resume_with_options(
        &self,
        thread_id: &str,
        prompt: &str,
        last_message_path: &Path,
        options: &CodexResumeRequestOptions,
    ) -> Result<(CodexResumeRunResult, WorkflowNodeDispatchExecutionOptions), String>;
}

#[derive(Default)]
struct AllowedPaths {
    projects: BTreeSet<String>,
    rollouts: BTreeSet<String>,
}

impl AppState {
    fn new() -> Self {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        Self {
            index_path: manifest_dir.join("../../index-kernel/codex-index.json"),
            tasks_path: manifest_dir.join("../../tasks/README.md"),
            workflow_state_path: default_workflow_state_path(),
        }
    }
}

pub fn run_workflow_machine_cli(args: Vec<String>) -> Result<String, String> {
    if args.len() != 5 && args.len() != 6 {
        return Err(
            "用法：__run_workflow_machine_real <project_root> <work_item_id> <objective> <max_rounds> <timeout_seconds_per_step> [execution_root]"
                .to_string(),
        );
    }
    Err(legacy_product_command_blocked_message(
        "__run_workflow_machine_real",
    ))
}

fn legacy_product_command_boundary(command_name: &str) -> ProductCommandBoundary {
    let spec = real_execution_command::legacy_product_command_boundary_spec(command_name);
    ProductCommandBoundary {
        boundary_version: spec.boundary_version,
        command_name: spec.command_name,
        command_family: spec.command_family,
        boundary_kind: spec.boundary_kind,
        h5_unified_product_command: spec.h5_unified_product_command,
        deprecated: spec.deprecated,
        product_routing_allows_real_execution: spec.product_routing_allows_real_execution,
        legacy_path_may_have_real_side_effects: spec.legacy_path_may_have_real_side_effects,
        replacement_command: spec.replacement_command,
        reason: spec.reason,
        warnings: spec.warnings,
    }
}

fn legacy_product_command_blocked_message(command_name: &str) -> String {
    real_execution_command::legacy_product_command_blocked_message(command_name)
}

// Tauri command wrappers live in src/commands.rs for the conservative no-behavior split.
include!("commands.rs");
include!("command_registry.rs");

fn read_index(state: &AppState) -> Result<Value, String> {
    let text = fs::read_to_string(&state.index_path)
        .map_err(|error| format!("无法读取索引文件 {}：{error}", state.index_path.display()))?;
    serde_json::from_str(&text)
        .map_err(|error| format!("索引 JSON 解析失败 {}：{error}", state.index_path.display()))
}

fn load_codex_session_transcript_for_index(
    state: &AppState,
    thread_id: &str,
) -> Result<CodexTranscript, String> {
    let db_path = codex_db::default_state_db_path();
    match read_index(state) {
        Ok(index) => load_codex_session_transcript_with_catalog(&index, thread_id, &db_path),
        Err(index_error) => load_codex_session_transcript_with_optional_catalog(
            None,
            thread_id,
            &db_path,
            Some(index_error),
        ),
    }
}

fn load_codex_session_transcript_with_catalog(
    index: &Value,
    thread_id: &str,
    db_path: &Path,
) -> Result<CodexTranscript, String> {
    load_codex_session_transcript_with_optional_catalog(Some(index), thread_id, db_path, None)
}

fn load_codex_session_transcript_with_optional_catalog(
    index: Option<&Value>,
    thread_id: &str,
    db_path: &Path,
    index_error: Option<String>,
) -> Result<CodexTranscript, String> {
    match codex_db::read_threads(db_path) {
        Ok(rows) => {
            if let Some(row) = rows.into_iter().find(|row| row.thread_id == thread_id) {
                return load_codex_session_transcript_from_sqlite_row(&row, db_path);
            }
        }
        Err(err) => {
            if let Some(index) = index {
                if let Some(thread) = find_index_thread(index, thread_id) {
                    return load_codex_session_transcript_from_index_thread(
                        index,
                        &thread,
                        "index_fallback_sqlite_unavailable",
                    );
                }
            }
            if let Some(index_error) = index_error {
                return Err(format!(
                    "sqlite_unavailable:{err};index_unavailable:{index_error}"
                ));
            }
            return Err(format!("sqlite_unavailable:{err}"));
        }
    }

    if let Some(index) = index {
        if let Some(thread) = find_index_thread(index, thread_id) {
            return load_codex_session_transcript_from_index_thread(
                index,
                &thread,
                "index_fallback_thread_missing_in_sqlite",
            );
        }
    }

    Err(format!("session_not_found:{thread_id}"))
}

fn load_codex_session_transcript_from_sqlite_row(
    row: &codex_db::CodexThreadRow,
    db_path: &Path,
) -> Result<CodexTranscript, String> {
    let codex_home = db_path
        .parent()
        .ok_or_else(|| "unexpected_internal_error:sqlite_db_path_without_parent".to_string())?;
    let metadata = codex_transcript::TranscriptThreadMetadata {
        thread_id: row.thread_id.clone(),
        rollout_path: row.rollout_path.clone(),
        project_root: row.project_root.clone(),
        title: Some(row.title.clone()),
        created_at_ms: None,
        updated_at_ms: row.updated_at_ms,
        catalog_source: "sqlite".to_string(),
        index_thread_count: None,
    };
    codex_transcript::read_transcript_from_rollout(metadata, codex_home)
}

fn load_codex_session_transcript_from_index_thread(
    index: &Value,
    thread: &SessionRecord,
    catalog_source: &str,
) -> Result<CodexTranscript, String> {
    let codex_home = codex_home_from_index(index)?;
    let metadata = codex_transcript::TranscriptThreadMetadata {
        thread_id: thread.thread_id.clone(),
        rollout_path: thread.rollout_path.clone(),
        project_root: thread.project_root.clone(),
        title: Some(thread.title.clone()),
        created_at_ms: None,
        updated_at_ms: thread.updated_at_ms,
        catalog_source: catalog_source.to_string(),
        index_thread_count: index.get("threads").and_then(Value::as_array).map(Vec::len),
    };
    codex_transcript::read_transcript_from_rollout(metadata, &codex_home)
}

fn codex_home_from_index(index: &Value) -> Result<PathBuf, String> {
    index
        .get("source_stats")
        .and_then(|source_stats| source_stats.get("codex_home"))
        .and_then(|codex_home| codex_home.get("path"))
        .and_then(Value::as_str)
        .filter(|path| !path.trim().is_empty())
        .map(PathBuf::from)
        .ok_or_else(|| "unexpected_internal_error:missing_index_codex_home_path".to_string())
}

include!("workflow_state_lifecycle_task_package.rs");

include!("workflow_run_dispatch_entrypoints.rs");

// C4-C6 automation workflow governance helpers live in a crate-root include for the conservative no-behavior split.
include!("c4_c6_workflow_governance_entrypoints.rs");

// Workflow dispatch execution control, offline role dispatch, and workflow machine helpers live in a crate-root include for the conservative no-behavior split.
include!("workflow_execution_entrypoints.rs");

fn find_work_item<'a>(
    value: &'a Value,
    workflow_id: &str,
    work_item_id: &str,
) -> Option<&'a Value> {
    value
        .get("work_items")
        .and_then(Value::as_array)
        .and_then(|items| {
            items.iter().find(|item| {
                optional_string_from(item, "workflow_id").as_deref() == Some(workflow_id)
                    && optional_string_from(item, "work_item_id").as_deref() == Some(work_item_id)
            })
        })
}

fn find_work_item_index(value: &Value, workflow_id: &str, work_item_id: &str) -> Option<usize> {
    value
        .get("work_items")
        .and_then(Value::as_array)
        .and_then(|items| {
            items.iter().position(|item| {
                optional_string_from(item, "workflow_id").as_deref() == Some(workflow_id)
                    && optional_string_from(item, "work_item_id").as_deref() == Some(work_item_id)
            })
        })
}

fn find_permission_request_index(
    value: &Value,
    workflow_id: &str,
    work_item_id: &str,
    request_id: &str,
) -> Option<usize> {
    value
        .get("permission_requests")
        .and_then(Value::as_array)
        .and_then(|requests| {
            requests.iter().position(|request| {
                optional_string_from(request, "workflow_id").as_deref() == Some(workflow_id)
                    && optional_string_from(request, "work_item_id").as_deref()
                        == Some(work_item_id)
                    && optional_string_from(request, "request_id").as_deref() == Some(request_id)
            })
        })
}

fn find_task_package_artifact_index(
    value: &Value,
    work_item_id: &str,
    work_item_index: usize,
) -> Option<usize> {
    let work_item = value
        .get("work_items")
        .and_then(Value::as_array)
        .and_then(|items| items.get(work_item_index))?;
    let source_artifact_id = optional_string_from(work_item, "source_ref");
    value
        .get("artifacts")
        .and_then(Value::as_array)
        .and_then(|artifacts| {
            artifacts.iter().position(|artifact| {
                optional_string_from(artifact, "artifact_type").as_deref() == Some("task_package")
                    && (optional_string_from(artifact, "source_ref").as_deref()
                        == Some(work_item_id)
                        || source_artifact_id.as_deref().is_some_and(|id| {
                            optional_string_from(artifact, "artifact_id").as_deref() == Some(id)
                        }))
            })
        })
}

fn find_task_package_artifact<'a>(
    value: &'a Value,
    work_item_id: &str,
    work_item: &Value,
) -> Option<&'a Value> {
    let source_artifact_id = optional_string_from(work_item, "source_ref");
    value
        .get("artifacts")
        .and_then(Value::as_array)
        .and_then(|artifacts| {
            artifacts.iter().find(|artifact| {
                optional_string_from(artifact, "artifact_type").as_deref() == Some("task_package")
                    && (optional_string_from(artifact, "source_ref").as_deref()
                        == Some(work_item_id)
                        || source_artifact_id.as_deref().is_some_and(|id| {
                            optional_string_from(artifact, "artifact_id").as_deref() == Some(id)
                        }))
            })
        })
}

struct RenderTaskPackageFields {
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
    project_name: String,
    project_root: String,
    workflow_id: String,
    work_item_id: String,
}

fn task_package_fields_from(
    work_item: &Value,
    artifact: &Value,
    project: &ProjectRecord,
    workflow_id: &str,
    work_item_id: &str,
) -> RenderTaskPackageFields {
    RenderTaskPackageFields {
        task_name: present_or_placeholder(
            optional_string_from(artifact, "task_name")
                .or_else(|| optional_string_from(work_item, "title")),
        ),
        assigned_line: optional_string_from(artifact, "assigned_line")
            .filter(|value| !value.trim().is_empty())
            .or_else(|| {
                optional_string_from(work_item, "assigned_role_id")
                    .map(|role| assigned_role_label(&role).to_string())
            })
            .unwrap_or_else(|| "未登记".to_string()),
        background: string_array_or_placeholder(
            artifact,
            "background",
            vec!["业务背景：待补充。".to_string()],
        ),
        goals: string_array_or_placeholder(
            artifact,
            "goals",
            vec![present_or_placeholder(optional_string_from(
                artifact, "brief",
            ))],
        ),
        allowed_read: string_array_or_placeholder(
            artifact,
            "allowed_read",
            vec![
                format!("`{}`", project.project_root),
                "待补充：草稿未登记更多允许读取清单。".to_string(),
            ],
        ),
        allowed_write: string_array_or_placeholder(
            artifact,
            "allowed_write",
            vec!["待补充：草稿未登记允许写入清单。".to_string()],
        ),
        forbidden_actions: string_array_or_placeholder(
            artifact,
            "forbidden_actions",
            vec![
                "不写 `/Users/yoyi/.codex`。".to_string(),
                "不改真实 Codex 状态库。".to_string(),
                "不生成真实 `product-line/tasks/*.md` 任务包文件，除非后续任务明确要求并再次确认。"
                    .to_string(),
                "不自动派发真实 Codex 会话。".to_string(),
                "不启动 Codex CLI。".to_string(),
                "不自动运行 harness。".to_string(),
                "待补充：草稿未登记更多禁止事项。".to_string(),
            ],
        ),
        acceptance_criteria: string_array_or_placeholder(
            artifact,
            "acceptance_criteria",
            vec!["待补充：草稿未登记验收标准。".to_string()],
        ),
        required_return: string_array_or_placeholder(
            artifact,
            "required_return",
            vec![
                "做了什么".to_string(),
                "改了哪些文件".to_string(),
                "新增了哪些测试或证据".to_string(),
                "哪些结论有依据".to_string(),
                "哪些仍不确定".to_string(),
                "风险和下一步建议".to_string(),
            ],
        ),
        review_focus: string_array_or_placeholder(
            artifact,
            "review_focus",
            vec![
                "判断是否接受、需要修改、暂停或废弃。".to_string(),
                "说明判断依据。".to_string(),
                "待补充：草稿未登记更具体的回收重点。".to_string(),
            ],
        ),
        project_name: project.name.clone(),
        project_root: project.project_root.clone(),
        workflow_id: workflow_id.to_string(),
        work_item_id: work_item_id.to_string(),
    }
}

fn present_or_placeholder(value: Option<String>) -> String {
    value
        .map(|raw| raw.trim().to_string())
        .filter(|raw| !raw.is_empty())
        .unwrap_or_else(|| "待补充".to_string())
}

fn task_memory_packet_input_from_task_package(
    project_root_value: &str,
    workflow_id: &str,
    work_item_id: &str,
    work_item: &Value,
    artifact: &Value,
    fields: &RenderTaskPackageFields,
) -> TaskMemoryPacketBuildInput {
    let joined_goals = fields
        .goals
        .iter()
        .map(|goal| goal.trim())
        .filter(|goal| !goal.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    let task_goal = if joined_goals.trim().is_empty() {
        fields.task_name.clone()
    } else {
        joined_goals
    };
    let role_id = optional_string_from(work_item, "assigned_role_id")
        .or_else(|| optional_string_from(artifact, "target_role"))
        .unwrap_or_else(|| assigned_line_id(&fields.assigned_line).to_string());
    let model_context_policy = optional_string_from(artifact, "model_context_policy")
        .filter(|policy| !policy.trim().is_empty())
        .unwrap_or_else(|| "local_only".to_string());

    TaskMemoryPacketBuildInput {
        project_root: project_root_value.to_string(),
        project_id: Some(project_id(project_root_value)),
        workflow_id: Some(workflow_id.to_string()),
        task_id: Some(work_item_id.to_string()),
        role_id,
        task_goal,
        retrieval_intent: "worker_task".to_string(),
        target_model_id: optional_string_from(artifact, "model_id"),
        model_context_policy,
        max_memory_items: i64_value(artifact, "max_memory_items").unwrap_or(8).max(1) as usize,
        max_estimated_tokens: i64_value(artifact, "max_estimated_tokens")
            .unwrap_or(2000)
            .max(128) as usize,
        expected_formal_store_revision: None,
        expected_candidate_store_revision: None,
        expected_observation_store_revision: None,
    }
}

fn string_array_or_placeholder(value: &Value, key: &str, fallback: Vec<String>) -> Vec<String> {
    let values = string_array(value, key)
        .into_iter()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    if values.is_empty() {
        fallback
    } else {
        values
    }
}

fn string_vec_value(items: &[String]) -> Value {
    Value::Array(
        items
            .iter()
            .map(|item| item.trim())
            .filter(|item| !item.is_empty())
            .map(|item| Value::String(item.to_string()))
            .collect(),
    )
}

fn cleaned_scalar(value: &str) -> String {
    value.trim().to_string()
}

fn join_lines(items: &[String]) -> String {
    items
        .iter()
        .map(|item| item.trim())
        .filter(|item| !item.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn assigned_line_id(value: &str) -> &str {
    match value.trim() {
        "Codex 开发线" | "codex-dev" => "codex-dev",
        "桌面应用线" | "desktop-app" => "desktop-app",
        "" => "未登记",
        other => other,
    }
}

fn field_warning_strings(fields: &TaskPackageFieldsInput) -> Vec<String> {
    let mut warnings = Vec::new();
    if cleaned_scalar(&fields.task_name).is_empty() {
        warnings.push("missing_task_name".to_string());
    }
    if cleaned_scalar(&fields.assigned_line).is_empty() {
        warnings.push("missing_assigned_line".to_string());
    }
    if join_lines(&fields.background).is_empty() {
        warnings.push("missing_background".to_string());
    }
    if join_lines(&fields.goals).is_empty() {
        warnings.push("missing_goals".to_string());
    }
    if join_lines(&fields.allowed_read).is_empty() {
        warnings.push("missing_allowed_read".to_string());
    }
    if join_lines(&fields.allowed_write).is_empty() {
        warnings.push("missing_allowed_write".to_string());
    }
    if join_lines(&fields.forbidden_actions).is_empty() {
        warnings.push("missing_forbidden_actions".to_string());
    }
    if join_lines(&fields.acceptance_criteria).is_empty() {
        warnings.push("missing_acceptance_criteria".to_string());
    }
    if join_lines(&fields.required_return).is_empty() {
        warnings.push("missing_required_return".to_string());
    }
    if join_lines(&fields.review_focus).is_empty() {
        warnings.push("missing_review_focus".to_string());
    }
    warnings
}

fn assigned_role_label(role: &str) -> &str {
    match role {
        "codex-dev" => "Codex 开发线",
        "desktop-app" => "桌面应用线",
        _ => role,
    }
}

fn preview_warnings(work_item: &Value, artifact: &Value) -> Vec<String> {
    let mut warnings = Vec::new();
    if present_or_placeholder(optional_string_from(work_item, "title")) == "待补充" {
        warnings.push("任务名未登记，预览使用待补充。".to_string());
    }
    if optional_string_from(work_item, "assigned_role_id").is_none() {
        warnings.push("所属开发线未登记，预览使用未登记。".to_string());
    }
    if present_or_placeholder(optional_string_from(artifact, "brief")) == "待补充" {
        warnings.push("目标说明未登记，预览使用待补充。".to_string());
    }
    warnings.extend(string_array(artifact, "warnings"));
    warnings
}

fn dispatch_blocking_reasons(
    fields: &RenderTaskPackageFields,
    artifact_path: Option<&str>,
) -> Vec<String> {
    let mut reasons = Vec::new();
    if artifact_path.is_none_or(|path| path.trim().is_empty()) {
        reasons.push("尚未生成真实任务包文件。".to_string());
    }
    if looks_missing(&fields.task_name) || looks_test_draft(&fields.task_name) {
        reasons.push("任务名为空、待补充或仍像测试草稿。".to_string());
    }
    if list_missing_or_dirty(&fields.goals) {
        reasons.push("目标为空、待补充或含输入法污染。".to_string());
    }
    if list_missing_or_dirty(&fields.allowed_read) {
        reasons.push("允许读取为空、待补充或含输入法污染。".to_string());
    }
    if list_missing_or_dirty(&fields.allowed_write) {
        reasons.push("允许写入为空、待补充或含输入法污染。".to_string());
    }
    if list_missing_or_dirty(&fields.acceptance_criteria) {
        reasons.push("验收标准为空、待补充或含输入法污染。".to_string());
    }
    if fields
        .forbidden_actions
        .iter()
        .any(|line| contains_conflicting_generation_ban(line))
    {
        reasons.push("禁止事项仍包含和当前生成行为冲突的历史禁令。".to_string());
    }
    if fields
        .allowed_write
        .iter()
        .any(|line| line.contains("/Users/yoyi/.codex"))
    {
        reasons.push("允许写入包含 /Users/yoyi/.codex，违反本轮边界。".to_string());
    }
    let required_return = fields
        .required_return
        .iter()
        .map(|line| line.trim())
        .collect::<Vec<_>>();
    for required in ["做了什么", "改了哪些文件", "验证命令和结果", "风险"] {
        if !required_return.iter().any(|line| line.contains(required)) {
            reasons.push(format!("必须回传缺少标准字段：{required}。"));
        }
    }
    reasons
}

fn dispatch_warning_reasons(artifact: &Value, artifact_path: Option<&str>) -> Vec<String> {
    let mut warnings = string_array(artifact, "warnings")
        .into_iter()
        .map(|warning| format!("artifact warning: {warning}"))
        .collect::<Vec<_>>();
    if bool_value(artifact, "stale") {
        warnings.push("任务包已标记 stale；需要重新检查并生成新版本。".to_string());
    }
    warnings.extend(
        string_array(artifact, "stale_reasons")
            .into_iter()
            .map(|reason| format!("stale: {reason}")),
    );
    if let Some(path) = artifact_path {
        if !Path::new(path).exists() {
            warnings.push("artifact path 指向的文件当前不存在。".to_string());
        }
    }
    warnings
}

fn list_missing_or_dirty(values: &[String]) -> bool {
    values.is_empty()
        || values
            .iter()
            .any(|value| looks_missing(value) || contains_input_method_noise(value))
}

fn looks_missing(value: &str) -> bool {
    let cleaned = value.trim();
    cleaned.is_empty()
        || cleaned.contains("待补充")
        || cleaned.contains("未登记")
        || cleaned.contains("草稿未登记")
}

fn looks_test_draft(value: &str) -> bool {
    let cleaned = value.trim().to_ascii_lowercase();
    cleaned.contains("task draft")
        || cleaned.contains("smoke")
        || cleaned.contains("test")
        || value.contains("测试草稿")
}

fn contains_input_method_noise(value: &str) -> bool {
    value.contains("他日") || value.contains("输入法污染")
}

fn contains_conflicting_generation_ban(value: &str) -> bool {
    let cleaned = value.trim();
    cleaned.contains("不生成真实 `product-line/tasks/*.md`")
        || cleaned.contains("不生成真实 product-line/tasks/*.md")
        || cleaned.contains("不生成真实任务文件")
        || cleaned.contains("不生成真实任务包文件")
}

fn render_task_package_markdown(
    fields: &RenderTaskPackageFields,
    artifact_id: Option<&str>,
    memory_snapshot: Option<&TaskPackageMemoryPacketSnapshot>,
) -> String {
    let artifact_ref = artifact_id.unwrap_or("未登记");
    let memory_context = memory_snapshot
        .map(task_memory_injection::render_markdown_block)
        .unwrap_or_default();
    format!(
        r#"# 任务包：{task_name}

## 任务名

{task_name}

## 所属开发线

{assigned_line}

## 背景

- 预览依据：工作台状态文件中的 `work_items[]` 和 `artifacts[]` 草稿记录。
- 项目来源：索引内项目 `{project_name}`。
- 项目根目录：`{project_root}`。
- workflow：`{workflow_id}`。
- work item：`{work_item_id}`。
- artifact：`{artifact_ref}`。
{background}

## 目标

{goals}

{memory_context}

## 允许读取

{allowed_read}

## 允许写入

{allowed_write}

## 禁止事项

{forbidden_actions}

## 验收标准

{acceptance_criteria}

## 必须回传

{required_return}

## 总指导回收重点

{review_focus}
"#,
        task_name = fields.task_name,
        assigned_line = fields.assigned_line,
        background = bullet_lines(&fields.background),
        goals = bullet_lines(&fields.goals),
        memory_context = memory_context,
        allowed_read = bullet_lines(&fields.allowed_read),
        allowed_write = bullet_lines(&fields.allowed_write),
        forbidden_actions = bullet_lines(&fields.forbidden_actions),
        acceptance_criteria = bullet_lines(&fields.acceptance_criteria),
        required_return = numbered_lines(&fields.required_return),
        review_focus = bullet_lines(&fields.review_focus),
        project_name = fields.project_name,
        project_root = fields.project_root,
        workflow_id = fields.workflow_id,
        work_item_id = fields.work_item_id,
        artifact_ref = artifact_ref
    )
}

fn task_package_fingerprint(fields: &RenderTaskPackageFields) -> String {
    stable_id(&format!(
        "{}|{}|{}|{}|{}|{}|{}|{}|{}",
        fields.task_name,
        fields.assigned_line,
        fields.background.join("\n"),
        fields.goals.join("\n"),
        fields.allowed_read.join("\n"),
        fields.allowed_write.join("\n"),
        fields.forbidden_actions.join("\n"),
        fields.acceptance_criteria.join("\n"),
        fields.required_return.join("\n")
    ))
}

fn bullet_lines(items: &[String]) -> String {
    items
        .iter()
        .map(|item| format!("- {item}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn numbered_lines(items: &[String]) -> String {
    items
        .iter()
        .enumerate()
        .map(|(index, item)| format!("{}. {item}", index + 1))
        .collect::<Vec<_>>()
        .join("\n")
}

fn markdown_list_or_empty(items: &[String]) -> String {
    if items.is_empty() {
        "- 待补充".to_string()
    } else {
        items
            .iter()
            .map(|item| format!("- {item}"))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

fn render_user_reviewed_instruction_preview(
    summary: &str,
    objective: &str,
    execution_cwd: &str,
    sandbox_mode: &str,
    allowed_write_roots: &[String],
    allowed_reads: &[String],
    allowed_writes: &[String],
    forbidden_actions: &[String],
    required_return: &[String],
) -> String {
    format!(
        "# 用户审核业务指令预览\n\n## 摘要\n{}\n\n## 目标\n{}\n\n## 执行目录\n{}\n\n## 沙箱模式\n{}\n\n## 允许写入根目录\n{}\n\n## 允许读取\n{}\n\n## 允许写入\n{}\n\n## 禁止事项\n{}\n\n## 必须回传\n{}",
        summary,
        objective,
        execution_cwd,
        sandbox_mode,
        markdown_list_or_empty(allowed_write_roots),
        markdown_list_or_empty(allowed_reads),
        markdown_list_or_empty(allowed_writes),
        markdown_list_or_empty(forbidden_actions),
        markdown_list_or_empty(required_return)
    )
}

include!("workflow_state_json_helpers.rs");

// Memory command bridge, observation bridge, task memory preview, and context guard helpers live in a crate-root include for the conservative no-behavior split.
include!("memory_context_entrypoints.rs");

fn option_trimmed_is_empty(value: Option<&str>) -> bool {
    match value {
        Some(value) => value.trim().is_empty(),
        None => true,
    }
}

fn task_draft_exists(value: &Value, workflow_id: &str, title: &str) -> bool {
    value
        .get("work_items")
        .and_then(Value::as_array)
        .is_some_and(|work_items| {
            work_items.iter().any(|item| {
                optional_string_from(item, "workflow_id").as_deref() == Some(workflow_id)
                    && optional_string_from(item, "title")
                        .is_some_and(|item_title| item_title.trim() == title)
            })
        })
}

fn next_work_item_states(state: &str) -> Vec<String> {
    let states: &[&str] = match state {
        "draft" => &["ready_to_dispatch", "paused"],
        "ready_to_dispatch" => &["running", "paused"],
        "running" => &[
            "waiting_for_permission",
            "retry_pending",
            "failed",
            "timed_out",
            "cancelled",
            "ready_for_review",
            "paused",
        ],
        "waiting_for_permission" => &["running", "failed", "cancelled", "paused"],
        "retry_pending" => &["running", "failed", "paused"],
        "failed" => &["retry_pending", "needs_changes", "paused"],
        "timed_out" => &["retry_pending", "needs_changes", "paused"],
        "cancelled" => &["needs_changes", "paused"],
        "ready_for_review" => &["accepted", "needs_changes", "paused"],
        "needs_changes" => &["ready_to_dispatch", "paused"],
        "paused" => &["ready_to_dispatch"],
        _ => &[],
    };
    states.iter().map(|state| (*state).to_string()).collect()
}

fn work_item_state_label(state: &str) -> &'static str {
    match state {
        "draft" => "草稿",
        "ready_to_dispatch" => "待派发",
        "running" => "执行中",
        "waiting_for_permission" => "等待权限",
        "retry_pending" => "待重试",
        "failed" => "失败",
        "timed_out" => "已超时",
        "cancelled" => "已取消",
        "ready_for_review" => "待回收",
        "accepted" => "已接受",
        "needs_changes" => "需修改",
        "paused" => "暂停",
        _ => "未知",
    }
}

fn next_action_label(state: &str) -> Option<String> {
    let label = match state {
        "draft" => "下一步：标记待派发",
        "ready_to_dispatch" => "下一步：标记执行中",
        "running" => "下一步：等待权限、重试、失败、超时、取消或待回收",
        "waiting_for_permission" => "下一步：记录权限结论",
        "retry_pending" => "下一步：重新执行或标记失败",
        "failed" => "下一步：安排重试或要求修改",
        "timed_out" => "下一步：安排重试或要求修改",
        "cancelled" => "下一步：要求修改或暂停",
        "ready_for_review" => "下一步：接受或要求修改",
        "needs_changes" => "下一步：重新标记待派发",
        "paused" => "下一步：恢复到待派发",
        "accepted" => "已结束：当前没有下一步动作",
        _ => "缺少状态规则",
    };
    Some(label.to_string())
}

fn workflow_node_for_work_item_state(workflow_id: &str, state: &str) -> String {
    let suffix = match state {
        "draft" | "ready_to_dispatch" | "needs_changes" | "paused" => "director",
        "running"
        | "waiting_for_permission"
        | "retry_pending"
        | "failed"
        | "timed_out"
        | "cancelled" => "codex-dev",
        "ready_for_review" | "accepted" => "review",
        _ => "director",
    };
    format!("{workflow_id}:node:{suffix}")
}

fn update_node_state_for_id(
    value: &mut Value,
    node_id: &str,
    state: &str,
    timestamp: &str,
) -> Result<(), String> {
    if let Some(node) = array_mut(value, "nodes")?
        .iter_mut()
        .find(|node| optional_string_from(node, "node_id").as_deref() == Some(node_id))
    {
        node["state"] = Value::String(state.to_string());
        node["updated_at"] = Value::String(timestamp.to_string());
    }
    Ok(())
}

fn update_task_node_state(
    value: &mut Value,
    workflow_id: &str,
    timestamp: &str,
) -> Result<Option<String>, String> {
    let task_node_id = format!("{workflow_id}:node:task");
    let nodes = array_mut(value, "nodes")?;
    let task_node = nodes
        .iter_mut()
        .find(|node| {
            optional_string_from(node, "node_id").as_deref() == Some(task_node_id.as_str())
        })
        .ok_or_else(|| "当前 workflow 缺少任务包节点，已拒绝登记任务包草稿".to_string())?;
    let before_state =
        optional_string_from(task_node, "state").unwrap_or_else(|| "unknown".to_string());
    if before_state == "draft" {
        return Ok(None);
    }
    task_node["state"] = Value::String("draft".to_string());
    task_node["updated_at"] = Value::String(timestamp.to_string());
    Ok(Some(before_state))
}

fn find_index_project(index: &Value, project_root: &str) -> Option<ProjectRecord> {
    parse_projects(index)
        .into_iter()
        .find(|project| project.project_root == project_root)
}

fn find_index_thread(index: &Value, thread_id: &str) -> Option<SessionRecord> {
    parse_sessions(index)
        .into_iter()
        .find(|session| session.thread_id == thread_id)
}

fn project_id(project_root: &str) -> String {
    format!("project:{}", stable_id(project_root))
}

fn default_workflow_id(project_root: &str) -> String {
    format!("workflow:{}:default", stable_id(project_root))
}

fn stable_id(value: &str) -> String {
    let mut output = String::new();
    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            output.push(character.to_ascii_lowercase());
        } else if !output.ends_with('-') {
            output.push('-');
        }
    }
    output
        .trim_matches('-')
        .chars()
        .take(96)
        .collect::<String>()
}

fn blackboard_entry_kind_name(kind: BlackboardEntryKind) -> &'static str {
    match kind {
        BlackboardEntryKind::SubagentReport => "subagent_report",
        BlackboardEntryKind::Risk => "risk",
        BlackboardEntryKind::PermissionRequest => "permission_request",
        BlackboardEntryKind::ToolSummary => "tool_summary",
        BlackboardEntryKind::MemoryCandidate => "memory_candidate",
        BlackboardEntryKind::KnowledgeRef => "knowledge_ref",
    }
}

fn blackboard_target_kind_name(kind: BlackboardCandidateTargetKind) -> &'static str {
    match kind {
        BlackboardCandidateTargetKind::WorkflowFact => "workflow_fact",
        BlackboardCandidateTargetKind::WorkflowRisk => "workflow_risk",
        BlackboardCandidateTargetKind::PermissionDecision => "permission_decision",
        BlackboardCandidateTargetKind::AuditEvent => "audit_event",
        BlackboardCandidateTargetKind::FormalMemory => "formal_memory",
        BlackboardCandidateTargetKind::KnowledgeReference => "knowledge_reference",
        BlackboardCandidateTargetKind::NoPromotion => "no_promotion",
    }
}

fn blackboard_state_name(state: BlackboardCandidateState) -> &'static str {
    match state {
        BlackboardCandidateState::CandidatePendingControlCore => "candidate_pending_control_core",
        BlackboardCandidateState::CandidateConfirmedForFollowup => {
            "candidate_confirmed_for_followup"
        }
        BlackboardCandidateState::CandidateRejected => "candidate_rejected",
        BlackboardCandidateState::CandidateDeferred => "candidate_deferred",
        BlackboardCandidateState::CandidateDiscarded => "candidate_discarded",
    }
}

fn task_file_slug(title: &str, work_item_id: &str) -> String {
    let title_slug = stable_id(title);
    let fallback = format!("task-package-{}", stable_id(work_item_id));
    let raw = if title_slug.is_empty() {
        fallback
    } else {
        title_slug
    };
    raw.chars().take(72).collect()
}

#[cfg(test)]
fn next_available_task_package_path(
    tasks_dir: &Path,
    title: &str,
    work_item_id: &str,
) -> Result<PathBuf, String> {
    let date_prefix = current_date_prefix();
    let slug = task_file_slug(title, work_item_id);
    for suffix in 0..100 {
        let file_name = if suffix == 0 {
            format!("{date_prefix}-generated-{slug}.md")
        } else {
            format!("{date_prefix}-generated-{slug}-{}.md", suffix + 1)
        };
        if file_name == "README.md" {
            continue;
        }
        let candidate = tasks_dir.join(file_name);
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    Err("无法生成不冲突的任务包文件名，已拒绝覆盖已有文件".to_string())
}

fn next_task_package_path_or_existing_match(
    tasks_dir: &Path,
    title: &str,
    work_item_id: &str,
    expected_text: &str,
) -> Result<(PathBuf, bool), String> {
    let date_prefix = current_date_prefix();
    let slug = task_file_slug(title, work_item_id);
    for suffix in 0..100 {
        let file_name = if suffix == 0 {
            format!("{date_prefix}-generated-{slug}.md")
        } else {
            format!("{date_prefix}-generated-{slug}-{}.md", suffix + 1)
        };
        if file_name == "README.md" {
            continue;
        }
        let candidate = tasks_dir.join(file_name);
        if !candidate.exists() {
            return Ok((candidate, false));
        }
        let existing = fs::read_to_string(&candidate).map_err(|error| {
            format!("读取已存在任务包文件失败 {}：{error}", candidate.display())
        })?;
        if existing == expected_text {
            return Ok((candidate, true));
        }
    }
    Err("无法生成不冲突的任务包文件名，已拒绝覆盖已有文件".to_string())
}

fn current_date_prefix() -> String {
    if let Ok(value) = std::env::var("CODEX_WORKBENCH_DATE_PREFIX") {
        let cleaned = value.trim();
        if cleaned.len() == 10
            && cleaned.chars().enumerate().all(|(index, character)| {
                matches!(index, 4 | 7) && character == '-'
                    || !matches!(index, 4 | 7) && character.is_ascii_digit()
            })
        {
            return cleaned.to_string();
        }
    }
    "2026-05-29".to_string()
}

fn default_task_package_output_dir() -> PathBuf {
    PathBuf::from("/Users/yoyi/workspace/product-line/tasks")
}

fn validate_generated_task_package_markdown(markdown: &str) -> Result<(), String> {
    for marker in ["# 任务包：", "## 任务名", "## 目标", "## 禁止事项"] {
        if !markdown.contains(marker) {
            return Err(format!("生成任务包内容缺少关键段落：{marker}"));
        }
    }
    Ok(())
}

fn atomic_write_new_text_file(path: &Path, text: &str) -> Result<(), String> {
    if path.exists() {
        return Err(format!(
            "目标任务包文件已存在，已拒绝覆盖：{}",
            path.display()
        ));
    }
    let parent = path
        .parent()
        .ok_or_else(|| format!("任务包文件路径没有父目录：{}", path.display()))?;
    let temp_path = parent.join(format!(".{}.tmp", path_name(&path.display().to_string())));
    if temp_path.exists() {
        return Err(format!(
            "临时任务包文件已存在，已拒绝覆盖：{}",
            temp_path.display()
        ));
    }
    {
        let mut file = fs::File::create_new(&temp_path)
            .map_err(|error| format!("创建临时任务包文件失败 {}：{error}", temp_path.display()))?;
        file.write_all(text.as_bytes())
            .map_err(|error| format!("写入临时任务包文件失败 {}：{error}", temp_path.display()))?;
        file.sync_all()
            .map_err(|error| format!("同步临时任务包文件失败 {}：{error}", temp_path.display()))?;
    }
    match fs::rename(&temp_path, path) {
        Ok(()) => Ok(()),
        Err(error) => {
            let _ = fs::remove_file(&temp_path);
            Err(format!(
                "原子写入任务包文件失败 {}：{error}",
                path.display()
            ))
        }
    }
}

// Workflow read model, dispatch summary, and readback stats helpers live in a crate-root include for the conservative no-behavior split.
include!("workflow_read_model_entrypoints.rs");

fn next_workflow_node_dispatch_id(
    context: &WorkflowNodeDispatchContext,
    timestamp: &str,
) -> String {
    format!(
        "dispatch:{}:{}:{}",
        stable_id(&context.workflow_id),
        stable_id(&context.work_item_id),
        timestamp
    )
}

fn safe_probe_target() -> &'static str {
    "WORKFLOW_NODE_DISPATCH_OK_2026_05_29"
}

fn safe_probe_prompt() -> String {
    format!("请只回复这一句：{}", safe_probe_target())
}

fn compact_last_message_summary(text: &str) -> String {
    let compact = text
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(240)
        .collect::<String>();
    let control_lines = text
        .lines()
        .map(str::trim)
        .filter(|line| {
            line.contains("WORKFLOW_MACHINE_FINAL_ACCEPTED")
                || line.contains("WORKFLOW_MACHINE_CONTINUE")
                || line.contains("WORKFLOW_MACHINE_STEP_STATUS")
        })
        .collect::<Vec<_>>();
    if control_lines.is_empty() {
        compact
    } else {
        format!("{} {}", compact, control_lines.join(" "))
    }
}

fn default_workflow_node_dispatch_output_dir() -> PathBuf {
    PathBuf::from("/tmp/codex-workflow-node-dispatch-v1")
}

fn atomic_write_json(path: &Path, value: &Value) -> Result<(), String> {
    workflow_state_store::atomic_write(path, value, &unix_timestamp_string())
}

fn default_workflow_state_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/Users/yoyi".to_string());
    PathBuf::from(home)
    .join("Library/Application Support/CodexGovernanceWorkbench/workflow-state/workflow-state.v0.json")
}

fn workspace_id() -> String {
    "workspace:yoyi-workspace".to_string()
}

fn unix_timestamp_string() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0);
    millis.to_string()
}

fn unix_timestamp_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(0)
}

fn unix_timestamp_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0)
}

fn build_snapshot(state: &AppState, index: &Value, tasks_text: &str) -> WorkbenchSnapshot {
    build_snapshot_with_session_source(
        state,
        index,
        tasks_text,
        SessionSourceMode::RealWithSqliteFallback,
    )
}

fn build_snapshot_with_session_source(
    state: &AppState,
    index: &Value,
    tasks_text: &str,
    session_source_mode: SessionSourceMode,
) -> WorkbenchSnapshot {
    let mut projects = parse_projects(index);
    let (sessions, sqlite_warnings) = load_sessions(index, session_source_mode);
    overlay_project_thread_counts(&mut projects, &sessions);
    let skills = parse_skills(index);
    let plugins = parse_plugins(index);
    let tasks = parse_tasks(tasks_text);
    let (workflow_state_for_adapters, workflow_state_adapter_warning) =
        match read_workflow_state_snapshot(&state.workflow_state_path) {
            Ok(snapshot) => (Some(snapshot), None),
            Err(error) => (None, Some(error)),
        };
    let agent_adapters = derive_agent_adapter_descriptors(
        &sessions,
        &projects,
        workflow_state_for_adapters.as_ref(),
        workflow_state_adapter_warning,
    );
    let session_operations = derive_session_operation_descriptors(&agent_adapters);
    let provider_availability =
        derive_provider_availability_summaries(&agent_adapters, &session_operations);
    let session_continuation_previews = derive_session_continuation_previews(
        &agent_adapters,
        &session_operations,
        &provider_availability,
        workflow_state_for_adapters.as_ref(),
    );
    let session_continuation_store = match session_continuation_store::load_store(
        &state.workflow_state_path,
        &unix_timestamp_string(),
    ) {
        Ok(store) => store,
        Err(error) => session_continuation_store::empty_store_with_warning(
            &state.workflow_state_path,
            &unix_timestamp_string(),
            error,
        ),
    };
    let runtime_generated_at = optional_string(index, "generated_at")
        .unwrap_or_else(|| session_continuation_store.updated_at.clone());
    let (runtime_session_attention, session_run_status_summaries) =
        runtime_session_attention::derive_runtime_session_attention(
            &session_continuation_previews,
            &session_continuation_store,
            &runtime_generated_at,
        );
    let runtime_log_store = runtime_log_store::load_store_or_derive(
        &state.workflow_state_path,
        &session_continuation_store,
        &runtime_session_attention,
        &runtime_generated_at,
    );
    let worker_protocol = worker_protocol::derive_worker_protocol_read_model(
        &agent_adapters,
        &session_operations,
        &provider_availability,
        &session_continuation_previews,
        &session_continuation_store,
        &runtime_session_attention,
        &runtime_log_store,
        &runtime_generated_at,
    );
    let real_execution_product_commands =
        real_execution_command::load_real_execution_product_command_read_model(
            &state.workflow_state_path,
            &runtime_generated_at,
        );
    let project_workflow_automation =
        project_workflow_automation::load_project_workflow_automation_read_model(
            &state.workflow_state_path,
            &runtime_generated_at,
        );
    let page_read_model_inventory =
        page_read_model::derive_page_read_model_inventory(&runtime_generated_at);
    let allowed = allowed_paths_with_sessions(index, &sessions);
    let top_level_warning_count = array_len(index, "warnings") + sqlite_warnings.len();
    let context_warning_count = projects
        .iter()
        .map(|project| {
            project.context_warnings.len()
                + project.warnings.len()
                + project
                    .harness_resources
                    .iter()
                    .map(|resource| resource.warnings.len())
                    .sum::<usize>()
        })
        .sum();
    let warning_count = top_level_warning_count
        + context_warning_count
        + sessions
            .iter()
            .map(|session| session.warnings.len())
            .sum::<usize>()
        + skills
            .iter()
            .map(|skill| skill.warnings.len())
            .sum::<usize>()
        + plugins
            .iter()
            .map(|plugin| plugin.warnings.len())
            .sum::<usize>()
        + agent_adapters
            .iter()
            .map(|adapter| {
                adapter.warnings.len()
                    + adapter
                        .capabilities
                        .iter()
                        .map(|capability| capability.warnings.len())
                        .sum::<usize>()
            })
            .sum::<usize>()
        + session_operations
            .iter()
            .map(|operation| operation.warnings.len())
            .sum::<usize>()
        + provider_availability
            .iter()
            .map(|summary| summary.warnings.len())
            .sum::<usize>()
        + session_continuation_previews
            .iter()
            .map(|preview| {
                preview.user_visible_warnings.len()
                    + preview.guard_result.warnings.len()
                    + preview.readback_expectation.warnings.len()
                    + preview.failure_handling.warnings.len()
                    + preview.audit_impact.warnings.len()
            })
            .sum::<usize>()
        + session_continuation_store.warnings.len()
        + session_continuation_store
            .continuations
            .iter()
            .map(|record| record.warnings.len())
            .sum::<usize>()
        + session_continuation_store
            .attempts
            .iter()
            .map(|attempt| attempt.warnings.len() + attempt.readback_summary.warnings.len())
            .sum::<usize>()
        + session_continuation_store
            .audit_events
            .iter()
            .map(|event| event.warnings.len())
            .sum::<usize>()
        + runtime_session_attention
            .iter()
            .map(|item| item.warnings.len() + item.readback_boundary.warnings.len())
            .sum::<usize>()
        + session_run_status_summaries
            .iter()
            .map(|summary| summary.warnings.len())
            .sum::<usize>()
        + runtime_log_store.warnings.len()
        + runtime_log_store
            .entries
            .iter()
            .map(|entry| entry.warnings.len())
            .sum::<usize>()
        + runtime_log_store
            .summaries
            .iter()
            .map(|summary| summary.warnings.len())
            .sum::<usize>()
        + worker_protocol.warnings.len()
        + worker_protocol
            .worker_adapters
            .iter()
            .map(|adapter| {
                adapter.warnings.len()
                    + adapter
                        .capability_descriptors
                        .iter()
                        .map(|capability| capability.warnings.len())
                        .sum::<usize>()
            })
            .sum::<usize>()
        + worker_protocol
            .work_threads
            .iter()
            .map(|thread| {
                thread.warnings.len()
                    + thread
                        .run_persistence_handle
                        .as_ref()
                        .map(|handle| handle.warnings.len())
                        .unwrap_or(0)
            })
            .sum::<usize>()
        + worker_protocol
            .run_units
            .iter()
            .map(|run| {
                run.warnings.len()
                    + run
                        .attention
                        .iter()
                        .map(|attention| attention.warnings.len())
                        .sum::<usize>()
            })
            .sum::<usize>()
        + worker_protocol
            .credential_requirements
            .iter()
            .map(|requirement| requirement.warnings.len())
            .sum::<usize>()
        + worker_protocol
            .external_call_risk_envelopes
            .iter()
            .map(|envelope| envelope.warnings.len())
            .sum::<usize>()
        + worker_protocol
            .project_capability_policies
            .iter()
            .map(|policy| policy.warnings.len())
            .sum::<usize>()
        + worker_protocol
            .run_relations
            .iter()
            .map(|relation| relation.warnings.len())
            .sum::<usize>()
        + worker_protocol
            .worker_lanes
            .iter()
            .map(|lane| lane.warnings.len())
            .sum::<usize>()
        + worker_protocol
            .multi_worker_dispatch_plans
            .iter()
            .map(|plan| plan.warnings.len())
            .sum::<usize>()
        + worker_protocol
            .adapter_contract_checklists
            .iter()
            .map(|checklist| checklist.warnings.len())
            .sum::<usize>()
        + worker_protocol
            .controlled_api_cli_semantics
            .iter()
            .map(|semantics| semantics.warnings.len())
            .sum::<usize>()
        + worker_protocol
            .diagnostic_event_schemas
            .iter()
            .map(|schema| schema.warnings.len())
            .sum::<usize>()
        + worker_protocol
            .adapter_health_summaries
            .iter()
            .map(|summary| summary.warnings.len())
            .sum::<usize>()
        + worker_protocol
            .adapter_degraded_modes
            .iter()
            .map(|mode| mode.warnings.len())
            .sum::<usize>()
        + worker_protocol
            .adapter_data_locations
            .iter()
            .map(|location| location.warnings.len())
            .sum::<usize>()
        + worker_protocol
            .dispatch_requests
            .iter()
            .map(|request| request.warnings.len())
            .sum::<usize>()
        + worker_protocol
            .dispatch_guards
            .iter()
            .map(|guard| guard.warnings.len())
            .sum::<usize>()
        + worker_protocol
            .permission_envelopes
            .iter()
            .map(|envelope| envelope.warnings.len())
            .sum::<usize>()
        + worker_protocol
            .readback_results
            .iter()
            .map(|readback| readback.warnings.len())
            .sum::<usize>()
        + worker_protocol
            .task_memory_packet_refs
            .iter()
            .map(|memory_ref| memory_ref.warnings.len())
            .sum::<usize>()
        + worker_protocol
            .worker_report_candidates
            .iter()
            .map(|candidate| candidate.warnings.len())
            .sum::<usize>()
        + worker_protocol
            .worker_handoffs
            .iter()
            .map(|handoff| {
                handoff.warnings.len()
                    + handoff
                        .report_candidate
                        .as_ref()
                        .map(|candidate| candidate.warnings.len())
                        .unwrap_or(0)
                    + handoff
                        .readback_result
                        .as_ref()
                        .map(|readback| readback.warnings.len())
                        .unwrap_or(0)
            })
            .sum::<usize>()
        + real_execution_product_commands.warnings.len()
        + project_workflow_automation.warnings.len()
        + project_workflow_automation
            .latest_plan
            .as_ref()
            .map(|plan| {
                plan.warnings.len()
                    + plan
                        .run_units
                        .iter()
                        .map(|unit| unit.warnings.len())
                        .sum::<usize>()
            })
            .unwrap_or(0);
    let diagnostic_summary = derive_diagnostic_summary(
        state,
        index,
        &projects,
        &sessions,
        &agent_adapters,
        &provider_availability,
        &session_continuation_store,
        &runtime_session_attention,
        &session_run_status_summaries,
        &runtime_log_store,
        top_level_warning_count,
        context_warning_count,
        &runtime_generated_at,
    );

    WorkbenchSnapshot {
        summary: IndexSummary {
            generated_at: optional_string(index, "generated_at"),
            project_count: projects.len(),
            session_count: sessions.len(),
            skill_count: skills.len(),
            plugin_count: plugins.len(),
            task_count: tasks.len(),
            warning_count,
        },
        projects,
        sessions,
        skills,
        plugins,
        tasks,
        agent_adapters,
        session_operations,
        provider_availability,
        session_continuation_previews,
        session_continuation_store,
        runtime_session_attention,
        session_run_status_summaries,
        runtime_log_store,
        worker_protocol,
        real_execution_product_commands,
        project_workflow_automation,
        page_read_model_inventory,
        diagnostic_summary,
        diagnostics: Diagnostics {
            index_path: state.index_path.display().to_string(),
            tasks_path: state.tasks_path.display().to_string(),
            generated_at: optional_string(index, "generated_at"),
            top_level_warning_count,
            context_warning_count,
            allowed_project_path_count: allowed.projects.len(),
            allowed_rollout_path_count: allowed.rollouts.len(),
            release_bundle_enabled: false,
            notes: vec![
                "会话直读 ~/.codex/state_*.sqlite 的最新状态库，不再依赖 build_index.py 的快照"
                    .to_string(),
                "本机动作只允许索引内已有项目路径和 rollout 路径".to_string(),
                "release bundle 当前关闭，未做完整产品化打包".to_string(),
            ],
        },
    }
}

// Diagnostics, provider availability, session continuation, adapter descriptor, and session operation helpers live in a crate-root include for the conservative no-behavior split.
include!("diagnostics_provider_session_entrypoints.rs");

// Index parsing, allowed path derivation, host OS helpers, and Tauri app assembly live in a crate-root include for the conservative no-behavior split.
include!("index_host_app_entrypoints.rs");

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn k3_b_command_guard_test_input(
        runtime_prompt_body: Option<String>,
    ) -> ProjectWorkflowAutomationK3BInput {
        ProjectWorkflowAutomationK3BInput {
            execution_point_id: "stage-k-k3-b1-mario-test-workflow-read-only".to_string(),
            project_root: None,
            project_id: None,
            workflow_id: None,
            workflow_node_id: None,
            run_unit_id: None,
            work_item_id: None,
            task_package_ref: None,
            task_memory_packet_ref: None,
            permission_envelope_ref: None,
            readback_marker: None,
            target_session_id: None,
            sandbox: None,
            allowed_write_path: None,
            prompt_summary: None,
            prompt_ref: None,
            prompt_hash: None,
            runtime_prompt_body,
            requested_by: None,
            confirmed_by: None,
            risk_acknowledgement: None,
            reason: None,
            expected_workflow_revision: None,
            expected_product_command_store_revision: None,
            expected_session_continuation_store_revision: None,
        }
    }

    #[test]
    fn k3_b_tauri_command_guard_rejects_runtime_prompt_body() {
        let err = ensure_k3_b_tauri_no_real_harness_request(&k3_b_command_guard_test_input(Some(
            "runtime prompt body would reach Phase B without this guard".to_string(),
        )))
        .expect_err("K3-B product command wrapper must reject runtime prompt bodies");

        assert_eq!(
            err,
            "k3_b_real_execution_requires_dedicated_level_b_authorization"
        );
    }

    #[test]
    fn k3_b_tauri_command_guard_allows_no_real_harness_request() {
        ensure_k3_b_tauri_no_real_harness_request(&k3_b_command_guard_test_input(None))
            .expect("K3-B no-real harness request should remain callable");
        ensure_k3_b_tauri_no_real_harness_request(&k3_b_command_guard_test_input(Some(
            String::new(),
        )))
        .expect("empty runtime prompt body should still behave as no-real harness input");
    }

    include!("lib_read_model_boundary_tests.rs");

    #[test]
    fn reads_real_static_index_summary() {
        let state = AppState::new();
        let index = read_index(&state).expect("static index should be readable");
        let snapshot =
            build_snapshot_with_session_source(&state, &index, "", SessionSourceMode::IndexOnly);

        assert!(snapshot.summary.project_count > 0);
        assert!(snapshot.summary.session_count > 0);
        assert!(snapshot.diagnostics.allowed_project_path_count > 0);
        assert!(snapshot.diagnostics.allowed_rollout_path_count > 0);
    }

    include!("lib_transcript_readback_tests.rs");

    fn test_temp_dir(prefix: &str) -> PathBuf {
        std::env::temp_dir().join(format!("{prefix}-{}", unix_timestamp_nanos()))
    }

    fn fixture_memory_scope() -> MemoryScope {
        MemoryScope {
            scope_id: "scope:project:offline".to_string(),
            scope_type: "project".to_string(),
            user_id: None,
            project_id: Some("project:offline".to_string()),
            workflow_id: Some("workflow:offline:default".to_string()),
            session_id: None,
            role_ids: vec![],
            document_refs: vec![],
            permission_policy_ref: None,
            model_export_policy: "local_only".to_string(),
            valid_from: "2026-06-03T00:00:00Z".to_string(),
            valid_until: None,
        }
    }

    fn fixture_memory_source_ref() -> MemorySourceRef {
        MemorySourceRef {
            source_ref_id: "source:stage-report:offline:001".to_string(),
            source_type: "stage_report".to_string(),
            source_id: Some("stage:offline:001".to_string()),
            source_path: Some("evidence/offline.md".to_string()),
            source_title: Some("离线阶段报告".to_string()),
            anchor: None,
            source_created_at: None,
            captured_at: "2026-06-03T00:00:00Z".to_string(),
            authority_level: "evidence".to_string(),
            sensitive_level: "project".to_string(),
            content_hash: None,
        }
    }

    fn fixture_memory_candidate_input() -> CreateMemoryCandidateInput {
        CreateMemoryCandidateInput {
            project_root: "/offline-fixture/projects/codex-workbench".to_string(),
            project_id: Some("project:offline".to_string()),
            workflow_id: Some("workflow:offline:default".to_string()),
            scope: fixture_memory_scope(),
            memory_type: "project_memory".to_string(),
            claim: "候选层确认不能创建正式记忆。".to_string(),
            body: "candidate_confirmed 只表示候选被确认保留。".to_string(),
            source_refs: vec![fixture_memory_source_ref()],
            generated_by_role: "project_director".to_string(),
            generated_from: "stage_handoff".to_string(),
            risk_level: "low".to_string(),
            sensitive_level: "project".to_string(),
            requires_user_confirmation: false,
            review_reason: "候选和正式记忆分离测试".to_string(),
            expected_store_revision: None,
        }
    }

    fn fixture_formal_memory_input() -> CreateFormalMemoryRecordInput {
        CreateFormalMemoryRecordInput {
            project_root: "/offline-fixture/projects/codex-workbench".to_string(),
            project_id: Some("project:offline".to_string()),
            workflow_id: Some("workflow:offline:default".to_string()),
            scope: fixture_memory_scope(),
            memory_type: "project_memory".to_string(),
            claim: "正式记忆创建必须同步写版本和审计。".to_string(),
            body: "M1 正式记忆骨架只验证受控创建，不做候选采纳或任务包注入。".to_string(),
            source_refs: vec![fixture_memory_source_ref()],
            actor_id: "project-director-offline".to_string(),
            actor_role: "project_director".to_string(),
            reason: "创建 M1 正式记忆测试记录。".to_string(),
            audit_event_type: None,
            expected_store_revision: None,
        }
    }

    fn fixture_bound_formal_memory_input(project_root: &str) -> CreateFormalMemoryRecordInput {
        let mut input = fixture_formal_memory_input();
        let project_id_value = project_id(project_root);
        let workflow_id_value = default_workflow_id(project_root);
        input.project_root = project_root.to_string();
        input.project_id = Some(project_id_value.clone());
        input.workflow_id = Some(workflow_id_value.clone());
        input.scope.scope_id = format!("scope:workflow:{}", stable_id(project_root));
        input.scope.scope_type = "workflow".to_string();
        input.scope.project_id = Some(project_id_value);
        input.scope.workflow_id = Some(workflow_id_value);
        input
    }

    fn fixture_bound_memory_scope(project_root: &str) -> MemoryScope {
        let project_id_value = project_id(project_root);
        MemoryScope {
            scope_id: format!("scope:project:{}", stable_id(project_root)),
            scope_type: "project".to_string(),
            user_id: None,
            project_id: Some(project_id_value),
            workflow_id: None,
            session_id: None,
            role_ids: vec![],
            document_refs: vec![],
            permission_policy_ref: None,
            model_export_policy: "local_only".to_string(),
            valid_from: "2026-06-03T00:00:00Z".to_string(),
            valid_until: None,
        }
    }

    fn fixture_bound_memory_candidate_input(project_root: &str) -> CreateMemoryCandidateInput {
        let mut input = fixture_memory_candidate_input();
        input.project_root = project_root.to_string();
        input.project_id = Some(project_id(project_root));
        input.workflow_id = Some(default_workflow_id(project_root));
        input.scope = fixture_bound_memory_scope(project_root);
        input.memory_type = "project_memory".to_string();
        input.risk_level = "low".to_string();
        input.sensitive_level = "project".to_string();
        input.requires_user_confirmation = false;
        input
    }

    fn create_active_plan_authorization_for_fixture(path: &Path, project_root: &str) -> String {
        let project_id_value = project_id(project_root);
        let workflow_id_value = default_workflow_id(project_root);
        let timestamp = unix_timestamp_ms();
        let created = plan_authorization_store::create_authorization(
            path,
            &CreatePlanAuthorizationInput {
                project_root: project_root.to_string(),
                project_id: Some(project_id_value.clone()),
                workflow_id: Some(workflow_id_value.clone()),
                source_proposal_id: Some("proposal:fixture".to_string()),
                title: "测试方案授权".to_string(),
                goal_summary: "允许测试 fixture 在授权范围内检查准备态派发。".to_string(),
                scope: AuthorizedExecutionScope {
                    project_id: project_id_value,
                    workflow_id: workflow_id_value,
                    allowed_role_ids: vec![
                        "codex-dev".to_string(),
                        "desktop-app".to_string(),
                        "validation".to_string(),
                        "review".to_string(),
                        "director".to_string(),
                        "project_director".to_string(),
                    ],
                    allowed_agent_ids: vec![],
                    allowed_read_roots: vec![
                        project_root.to_string(),
                        "/tmp".to_string(),
                        "/Users/yoyi".to_string(),
                    ],
                    allowed_write_roots: vec![
                        project_root.to_string(),
                        "/tmp".to_string(),
                        "/Users/yoyi".to_string(),
                    ],
                    allowed_tools: vec![
                        "read_file".to_string(),
                        "apply_patch".to_string(),
                        "codex_exec_resume".to_string(),
                    ],
                    allowed_checks: vec![],
                    allowed_task_package_kinds: vec![
                        "task_package".to_string(),
                        "safe_probe".to_string(),
                        "user_reviewed_instruction".to_string(),
                        "offline_role_dispatch".to_string(),
                    ],
                    max_worker_dispatches: Some(8),
                    max_runtime_minutes: Some(60),
                    stop_conditions: vec![PlanAuthorizationStopCondition {
                        condition_id: "fixture-stop-user-confirmation".to_string(),
                        kind: "requires_user_confirmation".to_string(),
                        summary: "测试 fixture 触发用户确认时必须停下".to_string(),
                        requires_user_confirmation: true,
                    }],
                },
                actor_id: "project-director-fixture".to_string(),
                actor_role: "project_director".to_string(),
                expires_at_ms: None,
                expected_store_revision: None,
            },
            timestamp,
            &format!("write-fixture-plan-auth-{}", unix_timestamp_nanos()),
        )
        .expect("fixture plan authorization should create");
        let authorization_id = created.authorization.authorization_id.clone();
        plan_authorization_store::record_user_confirmation(
            path,
            &RecordPlanAuthorizationUserConfirmationInput {
                project_root: project_root.to_string(),
                authorization_id: authorization_id.clone(),
                actor_id: "user-fixture".to_string(),
                confirmation_summary: "用户确认测试方案授权范围。".to_string(),
                expected_store_revision: Some(created.store_revision),
            },
            timestamp + 1,
            &format!("write-fixture-plan-auth-user-{}", unix_timestamp_nanos()),
        )
        .expect("fixture user confirmation should write");
        plan_authorization_store::record_global_boundary_review(
            path,
            &RecordPlanAuthorizationGlobalBoundaryReviewInput {
                project_root: project_root.to_string(),
                authorization_id: authorization_id.clone(),
                actor_id: "global-director-fixture".to_string(),
                review_status: "approved".to_string(),
                summary: "全局主管复核通过测试方案边界。".to_string(),
                source_proposal_id: Some("proposal:fixture".to_string()),
                checklist: Some(fixture_global_boundary_review_checklist()),
                findings: vec![],
                reviewed_scope_fingerprint: None,
                expected_store_revision: Some(created.store_revision + 1),
            },
            timestamp + 2,
            &format!(
                "write-fixture-plan-auth-boundary-{}",
                unix_timestamp_nanos()
            ),
        )
        .expect("fixture boundary review should write");
        authorization_id
    }

    fn fixture_plan_authorization_scope(project_root: &str) -> AuthorizedExecutionScope {
        AuthorizedExecutionScope {
            project_id: project_id(project_root),
            workflow_id: default_workflow_id(project_root),
            allowed_role_ids: vec!["codex-dev".to_string()],
            allowed_agent_ids: vec!["agent-1".to_string()],
            allowed_read_roots: vec![project_root.to_string()],
            allowed_write_roots: vec![format!("{project_root}/src")],
            allowed_tools: vec!["read_file".to_string(), "apply_patch".to_string()],
            allowed_checks: vec!["cargo test --lib".to_string()],
            allowed_task_package_kinds: vec!["task_package".to_string()],
            max_worker_dispatches: Some(1),
            max_runtime_minutes: Some(30),
            stop_conditions: vec![PlanAuthorizationStopCondition {
                condition_id: "requires-user-confirmation".to_string(),
                kind: "requires_user_confirmation".to_string(),
                summary: "需要用户确认".to_string(),
                requires_user_confirmation: true,
            }],
        }
    }

    fn fixture_plan_authorization_guard_input(project_root: &str) -> AutoDispatchGuardInput {
        AutoDispatchGuardInput {
            project_id: project_id(project_root),
            workflow_id: default_workflow_id(project_root),
            work_item_id: "work-item:c1-plan-authorization".to_string(),
            task_package_id: Some("artifact:c1-plan-authorization".to_string()),
            task_package_kind: Some("task_package".to_string()),
            target_role_id: "codex-dev".to_string(),
            target_agent_id: Some("agent-1".to_string()),
            requested_read_roots: vec![project_root.to_string()],
            requested_write_roots: vec![format!("{project_root}/src")],
            requested_tools: vec!["read_file".to_string()],
            requested_checks: vec!["cargo test --lib".to_string()],
            triggered_stop_conditions: vec![],
            dispatch_kind: "prepare_offline".to_string(),
        }
    }

    fn fixture_global_boundary_review_checklist() -> GlobalBoundaryReviewChecklist {
        GlobalBoundaryReviewChecklist {
            architecture_boundary_checked: true,
            cross_project_impact_checked: true,
            permission_scope_checked: true,
            read_write_scope_checked: true,
            tool_and_check_scope_checked: true,
            memory_boundary_checked: true,
            stop_conditions_checked: true,
            acceptance_criteria_checked: true,
        }
    }

    fn fixture_global_boundary_review_input(
        project_root: &str,
        proposal_id: &str,
        authorization_id: &str,
        expected_revision: i64,
    ) -> RecordGlobalBoundaryReviewInput {
        RecordGlobalBoundaryReviewInput {
            project_root: project_root.to_string(),
            project_id: project_id(project_root),
            workflow_id: default_workflow_id(project_root),
            proposal_id: proposal_id.to_string(),
            authorization_id: authorization_id.to_string(),
            actor_id: "global-director-fixture".to_string(),
            review_status: "approved".to_string(),
            summary: "全局主管复核通过 C3 fixture 方案边界；授权有效，仍未派发 worker。"
                .to_string(),
            checklist: fixture_global_boundary_review_checklist(),
            findings: vec![],
            expected_authorization_revision: Some(expected_revision),
        }
    }

    fn create_confirmed_proposal_for_global_review(
        path: &Path,
        project_root: &str,
        timestamp_ms: i64,
    ) -> (ProjectConsultationProposal, PlanAuthorization, i64) {
        let created = project_consultation_proposal_store::create_proposal(
            path,
            &fixture_project_consultation_proposal_input(project_root),
            timestamp_ms,
            &format!("write-c3-proposal-create-{}", unix_timestamp_nanos()),
        )
        .expect("proposal should create");
        let confirmed = project_consultation_proposal_store::record_decision(
            path,
            &RecordProjectConsultationProposalDecisionInput {
                project_root: project_root.to_string(),
                proposal_id: created.proposal.proposal_id.clone(),
                actor_id: "user-fixture".to_string(),
                decision: ProjectConsultationProposalDecisionKind::Confirm,
                summary: "用户确认 C3 测试方案；等待全局边界复核。".to_string(),
                expected_proposal_store_revision: Some(created.store_revision),
                expected_plan_authorization_store_revision: None,
            },
            timestamp_ms + 1,
            &format!("write-c3-proposal-confirm-{}", unix_timestamp_nanos()),
            &format!("write-c3-plan-auth-{}", unix_timestamp_nanos()),
            &format!("write-c3-plan-auth-user-{}", unix_timestamp_nanos()),
        )
        .expect("proposal confirm should create authorization");
        (
            confirmed.proposal,
            confirmed
                .plan_authorization
                .expect("confirmed proposal should link authorization"),
            confirmed
                .plan_authorization_store_revision
                .expect("confirmed proposal should return authorization revision"),
        )
    }

    fn fixture_plan_authorization_store_with_status(
        project_root: &str,
        status: PlanAuthorizationStatus,
        timestamp_ms: i64,
    ) -> PlanAuthorizationStoreV1 {
        let project_id_value = project_id(project_root);
        let workflow_id_value = default_workflow_id(project_root);
        let has_user_confirmation = matches!(
            status,
            PlanAuthorizationStatus::UserConfirmed
                | PlanAuthorizationStatus::PendingGlobalBoundaryReview
                | PlanAuthorizationStatus::Active
                | PlanAuthorizationStatus::Paused
                | PlanAuthorizationStatus::Revoked
                | PlanAuthorizationStatus::Expired
                | PlanAuthorizationStatus::Completed
        );
        let has_global_review = matches!(
            status,
            PlanAuthorizationStatus::Active
                | PlanAuthorizationStatus::Paused
                | PlanAuthorizationStatus::Revoked
                | PlanAuthorizationStatus::Expired
                | PlanAuthorizationStatus::Completed
        );
        PlanAuthorizationStoreV1 {
            schema_version: "plan_authorization_store.v1".to_string(),
            revision: 1,
            authorizations: vec![PlanAuthorization {
                authorization_id: "plan-auth:c1-fixture".to_string(),
                schema_version: "plan_authorization.v1".to_string(),
                project_id: project_id_value,
                workflow_id: workflow_id_value,
                source_proposal_id: Some("proposal:c1-fixture".to_string()),
                title: "C1 测试方案授权".to_string(),
                goal_summary: "验证方案授权 guard 的受控自动推进前置检查。".to_string(),
                status,
                scope: fixture_plan_authorization_scope(project_root),
                user_confirmation: has_user_confirmation.then_some(
                    PlanAuthorizationUserConfirmation {
                        confirmed_by: "user".to_string(),
                        confirmed_at_ms: timestamp_ms,
                        confirmation_summary: "用户确认 C1 fixture 授权。".to_string(),
                    },
                ),
                global_boundary_review: has_global_review.then_some(
                    PlanAuthorizationGlobalBoundaryReview {
                        reviewed_by: "global_director".to_string(),
                        reviewed_at_ms: timestamp_ms,
                        status: if status == PlanAuthorizationStatus::Paused {
                            "blocked".to_string()
                        } else {
                            "approved".to_string()
                        },
                        summary: "全局主管复核 C1 fixture 边界。".to_string(),
                        source_proposal_id: Some("proposal:c1-fixture".to_string()),
                        checklist: Some(fixture_global_boundary_review_checklist()),
                        findings: vec![],
                        reviewed_scope_fingerprint: Some("fixture-scope".to_string()),
                    },
                ),
                audit_refs: vec![],
                created_at_ms: timestamp_ms,
                updated_at_ms: timestamp_ms,
                expires_at_ms: None,
            }],
            audit_events: vec![],
            updated_at_ms: timestamp_ms,
            warnings: vec![],
        }
    }

    include!("lib_authorization_proposal_boundary_tests.rs");

    include!("lib_stage_c_governance_tests.rs");

    fn create_active_project_director_authorization_fixture(
        path: &Path,
        project_root: &str,
        thread_id: &str,
        timestamp_ms: i64,
    ) -> (ProjectConsultationProposal, PlanAuthorization, i64) {
        let mut input = fixture_project_consultation_proposal_input(project_root);
        input.scope_draft.allowed_agent_ids = vec![thread_id.to_string()];
        let created = project_consultation_proposal_store::create_proposal(
            path,
            &input,
            timestamp_ms,
            &format!("write-c4-proposal-create-{}", unix_timestamp_nanos()),
        )
        .expect("proposal should create");
        let confirmed = project_consultation_proposal_store::record_decision(
            path,
            &RecordProjectConsultationProposalDecisionInput {
                project_root: project_root.to_string(),
                proposal_id: created.proposal.proposal_id.clone(),
                actor_id: "user-fixture".to_string(),
                decision: ProjectConsultationProposalDecisionKind::Confirm,
                summary: "用户确认 C4 测试方案；等待全局主管复核。".to_string(),
                expected_proposal_store_revision: Some(created.store_revision),
                expected_plan_authorization_store_revision: None,
            },
            timestamp_ms + 1,
            &format!("write-c4-proposal-confirm-{}", unix_timestamp_nanos()),
            &format!("write-c4-plan-auth-{}", unix_timestamp_nanos()),
            &format!("write-c4-plan-auth-user-{}", unix_timestamp_nanos()),
        )
        .expect("proposal confirmation should create authorization");
        let authorization = confirmed
            .plan_authorization
            .expect("confirmed proposal should link authorization");
        let revision = confirmed
            .plan_authorization_store_revision
            .expect("confirmed proposal should return authorization revision");
        let output = plan_authorization_store::record_global_boundary_review_with_proposal(
            path,
            &fixture_global_boundary_review_input(
                project_root,
                &confirmed.proposal.proposal_id,
                &authorization.authorization_id,
                revision,
            ),
            timestamp_ms + 2,
            &format!("write-c4-global-boundary-{}", unix_timestamp_nanos()),
        )
        .expect("global boundary review should activate authorization");
        (
            confirmed.proposal,
            output.authorization,
            output.store_revision,
        )
    }

    fn fixture_project_director_preview_input(
        project_root: &str,
        proposal_id: &str,
        authorization_id: &str,
        authorization_revision: i64,
    ) -> PreviewProjectDirectorTaskPlanInput {
        PreviewProjectDirectorTaskPlanInput {
            project_root: project_root.to_string(),
            project_id: project_id(project_root),
            workflow_id: default_workflow_id(project_root),
            proposal_id: proposal_id.to_string(),
            authorization_id: authorization_id.to_string(),
            actor_id: "project_director".to_string(),
            expected_authorization_revision: Some(authorization_revision),
        }
    }

    fn fixture_project_director_prepare_input(
        project_root: &str,
        proposal_id: &str,
        authorization_id: &str,
        authorization_revision: i64,
        planned_tasks: Vec<ProjectDirectorPlannedTask>,
    ) -> PrepareAuthorizedAutoDispatchInput {
        PrepareAuthorizedAutoDispatchInput {
            project_root: project_root.to_string(),
            project_id: project_id(project_root),
            workflow_id: default_workflow_id(project_root),
            proposal_id: proposal_id.to_string(),
            authorization_id: authorization_id.to_string(),
            actor_id: "project_director".to_string(),
            planned_tasks,
            expected_workflow_revision: None,
            expected_authorization_revision: Some(authorization_revision),
        }
    }

    fn setup_c5_worker_report_fixture(
        name: &str,
    ) -> (PathBuf, PathBuf, ProjectRecord, String, String, String) {
        let dir = test_temp_dir(name);
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project(&format!("/tmp/{name}-project"));
        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_task_draft_at(
            &path,
            &fixture_task_draft_request(&project.project_root, "C5 worker 汇报测试任务"),
        )
        .expect("task draft should exist");
        let value = read_json_file(&path);
        let work_item_id = optional_string_from(&value["work_items"][0], "work_item_id")
            .expect("work item id should exist");
        let dispatch_id = append_fixture_dispatch(
            &path,
            &project.project_root,
            &work_item_id,
            "completed",
            "thread-c5",
        );
        let workflow_id = default_workflow_id(&project.project_root);
        let node_id = format!("{workflow_id}:node:director");
        (dir, path, project, work_item_id, dispatch_id, node_id)
    }

    fn setup_c6_complete_fixture(
        name: &str,
    ) -> (
        PathBuf,
        PathBuf,
        ProjectRecord,
        ProjectConsultationProposal,
        PlanAuthorization,
        String,
    ) {
        let timestamp_ms = 1_765_700_000_000;
        let dir = test_temp_dir(name);
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project(&format!("/tmp/{name}-project"));
        let thread_id = format!("thread-{name}");
        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        let (proposal, authorization, authorization_revision) =
            create_active_project_director_authorization_fixture(
                &path,
                &project.project_root,
                &thread_id,
                timestamp_ms,
            );
        let index = fixture_dispatch_index(&project.project_root, &thread_id);
        let workflow_id = default_workflow_id(&project.project_root);
        let node_id = format!("{workflow_id}:node:codex-dev");
        bind_workflow_node_codex_session_for_index_at(
            &path,
            &index,
            &fixture_node_session_bind_request(&project.project_root, &node_id, None, &thread_id),
        )
        .expect("node-level binding should write for C6 fixture");
        let prepare_input = fixture_project_director_prepare_input(
            &project.project_root,
            &proposal.proposal_id,
            &authorization.authorization_id,
            authorization_revision,
            vec![],
        );
        let prepared = prepare_authorized_auto_dispatch_for_index_at(&path, &index, &prepare_input)
            .expect("C4 prepared dispatch should exist for C6 fixture");
        let dispatch = prepared
            .prepared_dispatches
            .iter()
            .find(|dispatch| dispatch.dispatch_id.is_some())
            .expect("C6 fixture should have prepared dispatch");
        let work_item_id = dispatch
            .work_item_id
            .as_ref()
            .expect("prepared dispatch should link work item")
            .clone();
        let dispatch_id = dispatch
            .dispatch_id
            .as_ref()
            .expect("prepared dispatch should have id")
            .clone();
        let worker_node_id = dispatch
            .workflow_node_id
            .as_ref()
            .expect("prepared dispatch should link node")
            .clone();
        let report = record_worker_structured_report_at(
            &path,
            &fixture_c5_worker_report_input(
                &project.project_root,
                &work_item_id,
                &dispatch_id,
                &worker_node_id,
            ),
        )
        .expect("C5 worker report should write for C6 fixture");
        let process_fact_decision = fixture_c5_process_fact_decision_input(
            &project.project_root,
            &report.audit_event_id,
            &dispatch_id,
            "confirm_process_fact",
        );
        let process_fact_id = process_fact_decision.accepted_facts[0]
            .process_fact_id
            .clone();
        record_project_director_process_fact_decision_at(&path, &process_fact_decision)
            .expect("C5 process fact decision should write for C6 fixture");

        (dir, path, project, proposal, authorization, process_fact_id)
    }

    fn fixture_global_final_result_review_input(
        project_root: &str,
        proposal_id: &str,
        authorization_id: &str,
        process_fact_id: &str,
        decision: &str,
    ) -> GlobalFinalResultReviewInput {
        GlobalFinalResultReviewInput {
            project_root: project_root.to_string(),
            project_id: project_id(project_root),
            workflow_id: default_workflow_id(project_root),
            authorization_id: authorization_id.to_string(),
            proposal_id: proposal_id.to_string(),
            actor_id: "global-director-c6-fixture".to_string(),
            actor_role: "global_director".to_string(),
            decision: decision.to_string(),
            summary: format!("C6 全局最终复核 fixture：{decision}。"),
            evidence_refs: vec![
                proposal_id.to_string(),
                authorization_id.to_string(),
                process_fact_id.to_string(),
            ],
            accepted_process_fact_ids: if decision == "accepted" {
                vec![process_fact_id.to_string()]
            } else {
                vec![]
            },
            open_issues: if decision == "accepted" {
                vec![]
            } else {
                vec![format!("C6 fixture open issue for {decision}.")]
            },
            deferred_items: vec!["真实 worker / Codex 执行仍需单独授权。".to_string()],
            expected_workflow_revision: None,
        }
    }

    fn fixture_user_result_decision_input(
        project_root: &str,
        accepted_review_id: &str,
        decision: &str,
    ) -> UserResultDecisionInput {
        UserResultDecisionInput {
            project_root: project_root.to_string(),
            project_id: project_id(project_root),
            workflow_id: default_workflow_id(project_root),
            actor_id: "user-c6-fixture".to_string(),
            actor_role: "user".to_string(),
            decision: decision.to_string(),
            summary: format!("C6 用户结果决定 fixture：{decision}。"),
            requested_changes: if decision == "accept_result" {
                vec![]
            } else {
                vec![format!("C6 fixture requested change for {decision}.")]
            },
            accepted_review_id: Some(accepted_review_id.to_string()),
            expected_workflow_revision: None,
        }
    }

    fn fixture_c5_worker_report_input(
        project_root: &str,
        work_item_id: &str,
        dispatch_id: &str,
        node_id: &str,
    ) -> WorkerStructuredReportInput {
        WorkerStructuredReportInput {
            project_root: project_root.to_string(),
            project_id: project_id(project_root),
            workflow_id: default_workflow_id(project_root),
            workflow_node_id: node_id.to_string(),
            work_item_id: work_item_id.to_string(),
            dispatch_id: Some(dispatch_id.to_string()),
            actor_role: "codex-dev".to_string(),
            executed_what: "执行 C5 离线结构化汇报测试。".to_string(),
            changed_what: "只写工作台 workflow-state audit event，不写正式事实。".to_string(),
            summary: "worker 汇报：C5 过程事实待项目主管确认。".to_string(),
            evidence_refs: vec!["evidence:c5-worker-report:001".to_string()],
            open_issues: vec!["需要项目主管确认过程事实。".to_string()],
            permission_requests: vec![],
            direction_risks: vec![],
            follow_up_suggestions: vec!["确认后只写 observation，不写正式记忆。".to_string()],
            acceptance_status: "reported_completed".to_string(),
            source_refs: vec![fixture_c5_observation_source_ref(
                project_root,
                "workflow_event",
                dispatch_id,
            )],
            expected_workflow_revision: None,
        }
    }

    fn fixture_c5_process_fact_decision_input(
        project_root: &str,
        report_id: &str,
        dispatch_id: &str,
        decision: &str,
    ) -> ProjectDirectorProcessFactDecisionInput {
        let accepted_facts = if decision == "confirm_process_fact" {
            vec![fixture_c5_process_fact_candidate(
                project_root,
                report_id,
                dispatch_id,
            )]
        } else {
            vec![]
        };
        ProjectDirectorProcessFactDecisionInput {
            project_root: project_root.to_string(),
            project_id: project_id(project_root),
            workflow_id: default_workflow_id(project_root),
            report_id: report_id.to_string(),
            actor_id: "project-director-c5-fixture".to_string(),
            actor_role: "project_director".to_string(),
            decision: decision.to_string(),
            accepted_facts,
            rejected_fact_ids: if decision == "confirm_process_fact" {
                vec![]
            } else {
                vec![format!("process-fact:{}", stable_id(report_id))]
            },
            summary: format!("项目主管 C5 决定：{decision}。"),
            expected_workflow_revision: None,
            expected_observation_store_revision: None,
        }
    }

    fn fixture_c5_process_fact_candidate(
        project_root: &str,
        report_id: &str,
        dispatch_id: &str,
    ) -> ProcessFactCandidate {
        let workflow_id = default_workflow_id(project_root);
        ProcessFactCandidate {
            process_fact_id: format!("process-fact:{}", stable_id(report_id)),
            summary: "C5 低风险本项目过程事实：worker 已提交结构化汇报。".to_string(),
            source_report_id: report_id.to_string(),
            source_dispatch_id: Some(dispatch_id.to_string()),
            evidence_refs: vec!["evidence:c5-process-fact:001".to_string()],
            source_refs: vec![fixture_c5_observation_source_ref(
                project_root,
                "worker_report",
                report_id,
            )],
            scope: MemoryScope {
                scope_id: format!("scope:process-fact:{}", stable_id(report_id)),
                scope_type: "workflow".to_string(),
                user_id: None,
                project_id: Some(project_id(project_root)),
                workflow_id: Some(workflow_id),
                session_id: None,
                role_ids: vec!["project_director".to_string(), "codex-dev".to_string()],
                document_refs: vec![],
                permission_policy_ref: None,
                model_export_policy: "local_only".to_string(),
                valid_from: "2026-06-04T00:00:00Z".to_string(),
                valid_until: None,
            },
            risk_level: "low".to_string(),
            sensitive_level: "internal".to_string(),
            proposed_observation_type: "process_fact".to_string(),
        }
    }

    fn fixture_c5_observation_source_ref(
        project_root: &str,
        source_kind: &str,
        source_id: &str,
    ) -> ObservationSourceRef {
        ObservationSourceRef {
            source_ref_id: format!("obs-source:{source_kind}:{}", stable_id(source_id)),
            source_kind: source_kind.to_string(),
            source_id: source_id.to_string(),
            project_id: Some(project_id(project_root)),
            workflow_id: Some(default_workflow_id(project_root)),
            session_id: None,
            file_path: None,
            evidence_ref: Some(format!("evidence:{source_kind}:{}", stable_id(source_id))),
            summary: format!("C5 fixture source: {source_kind} / {source_id}"),
            sensitive_level: "internal".to_string(),
            created_at: "2026-06-04T00:00:00Z".to_string(),
        }
    }

    fn fixture_project_consultation_proposal_input(
        project_root: &str,
    ) -> CreateProjectConsultationProposalInput {
        let project_id_value = project_id(project_root);
        let workflow_id_value = default_workflow_id(project_root);
        CreateProjectConsultationProposalInput {
            project_root: project_root.to_string(),
            project_id: Some(project_id_value),
            workflow_id: Some(workflow_id_value),
            title: "C2 项目咨询方案测试草案".to_string(),
            user_goal: "让用户先确认项目自动推进方案范围。".to_string(),
            goal_summary: "建立项目咨询方案草案和用户确认入口。".to_string(),
            proposed_steps: vec![
                "整理用户目标和项目上下文。".to_string(),
                "用户确认方案后等待全局主管复核。".to_string(),
            ],
            scope_draft: ProjectConsultationProposalScopeDraft {
                allowed_role_ids: vec!["codex-dev".to_string(), "project_director".to_string()],
                allowed_agent_ids: vec!["agent-1".to_string()],
                allowed_read_roots: vec![project_root.to_string()],
                allowed_write_roots: vec![format!("{project_root}/src")],
                allowed_tools: vec!["read_file".to_string()],
                allowed_checks: vec!["cargo test --lib".to_string()],
                allowed_task_package_kinds: vec!["task_package".to_string()],
                stop_conditions: vec!["requires_user_confirmation".to_string()],
                max_worker_dispatches: Some(3),
                max_runtime_minutes: Some(60),
            },
            risks: vec![ProjectConsultationProposalRisk {
                risk_id: "risk:c2-fixture".to_string(),
                severity: "warning".to_string(),
                summary: "确认后仍不能自动派发。".to_string(),
                mitigation: "等待 C3 全局边界复核。".to_string(),
            }],
            acceptance_criteria: vec!["确认后授权仍停在待全局复核。".to_string()],
            created_by_role: ProjectConsultationProposalCreatorRole::ProjectConsultant,
            actor_id: "project-consultation-fixture".to_string(),
            expected_store_revision: None,
        }
    }

    fn fixture_adopt_memory_candidate_input(
        project_root: &str,
        candidate_key: String,
        expected_candidate_store_revision: Option<i64>,
        expected_formal_store_revision: Option<i64>,
    ) -> AdoptMemoryCandidateInput {
        AdoptMemoryCandidateInput {
            project_root: project_root.to_string(),
            candidate_key,
            actor_id: "project-director-offline".to_string(),
            actor_role: "project_director".to_string(),
            adoption_reason: "项目主管采纳低风险本项目记忆候选。".to_string(),
            expected_candidate_store_revision,
            expected_formal_store_revision,
        }
    }

    fn create_confirmed_memory_candidate(
        path: &Path,
        input: CreateMemoryCandidateInput,
    ) -> MemoryCandidate {
        let created = memory_candidate_store::create_candidate(
            path,
            &input,
            "2026-06-03T00:00:00Z",
            "write-memory-candidate-adoption-create",
        )
        .expect("memory candidate should be created");
        memory_candidate_store::record_decision(
            path,
            &RecordMemoryCandidateDecisionInput {
                project_root: input.project_root,
                candidate_key: created.candidate.candidate_key.clone(),
                requested_status: MemoryLifecycleStatus::CandidateConfirmed,
                reason: "确认保留候选；等待受控采纳。".to_string(),
                actor_id: "project_director".to_string(),
                actor_role: "project_director".to_string(),
                expected_store_revision: Some(created.store_revision),
            },
            "2026-06-03T00:00:01Z",
            &format!(
                "write-memory-candidate-adoption-confirm-{}",
                stable_id(&created.candidate.candidate_key)
            ),
        )
        .expect("memory candidate should be confirmed")
        .candidate
    }

    fn fixture_observation_source_ref(project_root: &str) -> ObservationSourceRef {
        ObservationSourceRef {
            source_ref_id: "obs-source:worker-report:001".to_string(),
            source_kind: "worker_report".to_string(),
            source_id: "worker-report:001".to_string(),
            project_id: Some(project_id(project_root)),
            workflow_id: Some(default_workflow_id(project_root)),
            session_id: Some("session:worker:001".to_string()),
            file_path: Some("handoffs/worker-report.md".to_string()),
            evidence_ref: None,
            summary: "开发线汇报：已完成受控观察入口实现。".to_string(),
            sensitive_level: "internal".to_string(),
            created_at: "2026-06-04T00:00:00Z".to_string(),
        }
    }

    fn fixture_observation_input(project_root: &str) -> CreateObservationInput {
        CreateObservationInput {
            project_root: project_root.to_string(),
            project_id: Some(project_id(project_root)),
            workflow_id: Some(default_workflow_id(project_root)),
            scope: fixture_bound_memory_scope(project_root),
            observation_type: "worker_report".to_string(),
            summary: "worker 汇报已被记录为 observation，但还不是正式记忆。".to_string(),
            source_refs: vec![fixture_observation_source_ref(project_root)],
            generated_by_role: "worker".to_string(),
            actor_id: "worker-memory-layer-dev".to_string(),
            risk_level: "low".to_string(),
            sensitive_level: "internal".to_string(),
            reason: "记录明确 worker 汇报，供项目主管确认后生成候选。".to_string(),
            expected_store_revision: None,
        }
    }

    fn create_recorded_observation(path: &Path, project_root: &str) -> CreateObservationOutput {
        create_observation_at(
            path,
            &fixture_observation_input(project_root),
            "2026-06-04T00:00:00Z",
            "write-observation-create",
        )
        .expect("observation should be recorded")
    }

    fn fixture_observation_candidate_input(
        project_root: &str,
        observation_key: String,
        expected_observation_store_revision: Option<i64>,
        expected_candidate_store_revision: Option<i64>,
    ) -> CreateMemoryCandidateFromObservationInput {
        CreateMemoryCandidateFromObservationInput {
            project_root: project_root.to_string(),
            observation_key,
            actor_id: "project-director-offline".to_string(),
            actor_role: "project_director".to_string(),
            memory_type: "project_memory".to_string(),
            claim: "worker 汇报中的过程事实可作为记忆候选。".to_string(),
            body: "该 observation 只生成 candidate_needs_review，后续仍需确认和采纳。".to_string(),
            review_reason: "项目主管确认 observation 可生成候选。".to_string(),
            requires_user_confirmation: false,
            expected_observation_store_revision,
            expected_candidate_store_revision,
        }
    }

    fn overwrite_first_observation_status(
        path: &Path,
        status: ObservationStatus,
        candidate_key: Option<String>,
    ) {
        let mut store = observation_store::load_store(path, "2026-06-04T00:00:01Z")
            .expect("observation store should load");
        let first = store
            .observations
            .first_mut()
            .expect("fixture observation should exist");
        first.status = status;
        first.candidate_key = candidate_key;
        first.updated_at = "2026-06-04T00:00:01Z".to_string();
        let sidecar = observation_store::sidecar_path(path).expect("sidecar path should resolve");
        fs::write(
            sidecar,
            serde_json::to_string_pretty(&store).expect("store should serialize"),
        )
        .expect("test should overwrite observation sidecar");
    }

    fn fixture_task_memory_packet_input(
        project_root: &str,
        task_goal: &str,
    ) -> TaskMemoryPacketBuildInput {
        TaskMemoryPacketBuildInput {
            project_root: project_root.to_string(),
            project_id: Some(project_id(project_root)),
            workflow_id: Some(default_workflow_id(project_root)),
            task_id: Some("work-item:memory-packet:001".to_string()),
            role_id: "codex-dev".to_string(),
            task_goal: task_goal.to_string(),
            retrieval_intent: "worker_task".to_string(),
            target_model_id: Some("offline-model".to_string()),
            model_context_policy: "local_only".to_string(),
            max_memory_items: 5,
            max_estimated_tokens: 2000,
            expected_formal_store_revision: None,
            expected_candidate_store_revision: None,
            expected_observation_store_revision: None,
        }
    }

    fn fixture_m10_preview_input(project_root: &str) -> PreviewMemoryEntityRelationCandidatesInput {
        PreviewMemoryEntityRelationCandidatesInput {
            project_root: project_root.to_string(),
            project_id: Some(project_id(project_root)),
            workflow_id: Some(default_workflow_id(project_root)),
        }
    }

    fn fixture_m10_memory_source(
        source_type: &str,
        source_id: &str,
        title: &str,
        sensitive_level: &str,
    ) -> MemorySourceRef {
        MemorySourceRef {
            source_ref_id: format!("source:m10:{}:{}", source_type, stable_id(source_id)),
            source_type: source_type.to_string(),
            source_id: Some(source_id.to_string()),
            source_path: None,
            source_title: Some(title.to_string()),
            anchor: None,
            source_created_at: None,
            captured_at: "2026-06-05T10:00:00Z".to_string(),
            authority_level: if source_type == "llm_inferred" {
                "derived_summary".to_string()
            } else {
                "evidence".to_string()
            },
            sensitive_level: sensitive_level.to_string(),
            content_hash: None,
        }
    }

    fn create_formal_memory_for_task(
        path: &Path,
        project_root: &str,
        claim: &str,
        body: &str,
        timestamp: &str,
        write_id: &str,
    ) -> MemoryRecord {
        let mut input = fixture_bound_formal_memory_input(project_root);
        input.claim = claim.to_string();
        input.body = body.to_string();
        input.memory_type = "project_memory".to_string();
        create_formal_memory_record_at(path, &input, timestamp, write_id)
            .expect("formal memory should be created")
            .record
    }

    fn mutate_formal_store<F>(path: &Path, mutate: F)
    where
        F: FnOnce(&mut FormalMemoryStoreV1),
    {
        let mut store = formal_memory_store::load_store(path, "2026-06-04T01:00:00Z")
            .expect("formal store should load");
        mutate(&mut store);
        let sidecar = formal_memory_store::sidecar_path(path).expect("formal sidecar path");
        fs::write(
            sidecar,
            serde_json::to_string_pretty(&store).expect("formal store should serialize"),
        )
        .expect("test should overwrite formal sidecar");
    }

    fn excluded_reason_count(
        output: &TaskMemoryPacketBuildOutput,
        reason: TaskMemoryPacketExclusionReason,
    ) -> usize {
        output
            .preview
            .excluded_items
            .iter()
            .filter(|item| item.reason == reason)
            .count()
    }

    fn fixture_memory_lint_run_input(
        project_root: &str,
        lint_intent: MemoryLintRunIntent,
    ) -> MemoryLintRunInput {
        MemoryLintRunInput {
            project_root: project_root.to_string(),
            project_id: Some(project_id(project_root)),
            workflow_id: Some(default_workflow_id(project_root)),
            actor_id: "project-director-offline".to_string(),
            actor_role: "project_director".to_string(),
            lint_intent,
            candidate_key: None,
            task_id: Some("work-item:memory-lint:001".to_string()),
            revoked_source_ids: vec![],
            expected_formal_store_revision: None,
            expected_candidate_store_revision: None,
            expected_lint_store_revision: None,
            dry_run: Some(false),
        }
    }

    fn create_formal_memory_with_source(
        path: &Path,
        project_root: &str,
        claim: &str,
        source_id: &str,
        authority_level: &str,
        timestamp: &str,
        write_id: &str,
    ) -> MemoryRecord {
        let mut input = fixture_bound_formal_memory_input(project_root);
        input.scope = fixture_bound_memory_scope(project_root);
        input.claim = claim.to_string();
        input.body = format!("{claim} 的详细说明。");
        input.memory_type = "project_memory".to_string();
        input.source_refs[0].source_id = Some(source_id.to_string());
        input.source_refs[0].authority_level = authority_level.to_string();
        create_formal_memory_record_at(path, &input, timestamp, write_id)
            .expect("formal memory should be created")
            .record
    }

    fn create_confirmed_candidate_with_claim(
        path: &Path,
        project_root: &str,
        claim: &str,
    ) -> MemoryCandidate {
        let mut input = fixture_bound_memory_candidate_input(project_root);
        input.claim = claim.to_string();
        input.body = format!("{claim} 的候选说明。");
        create_confirmed_memory_candidate(path, input)
    }

    include!("lib_memory_lint_mature_pattern_tests.rs");

    include!("lib_workflow_state_task_draft_blackboard_tests.rs");

    include!("lib_observation_candidate_tests.rs");

    include!("lib_task_memory_packet_tests.rs");

    include!("lib_memory_entity_relation_tests.rs");

    include!("lib_memory_store_context_tests.rs");

    #[test]
    fn memory_candidate_adoption_project_director_low_risk_project_memory() {
        let dir = test_temp_dir("memory-candidate-adoption-project-director");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/memory-candidate-adoption-project";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        let candidate = create_confirmed_memory_candidate(
            &path,
            fixture_bound_memory_candidate_input(project_root),
        );

        let adopted = adopt_memory_candidate_to_formal_memory_at(
            &path,
            &fixture_adopt_memory_candidate_input(
                project_root,
                candidate.candidate_key.clone(),
                Some(2),
                Some(0),
            ),
            "2026-06-03T00:00:02Z",
            "write-memory-candidate-adoption",
            "write-formal-from-candidate",
        )
        .expect("low risk project candidate should be adopted");

        assert_eq!(adopted.candidate_key, candidate.candidate_key);
        assert_eq!(
            adopted.candidate_status,
            MemoryLifecycleStatus::CandidateConfirmed
        );
        assert_eq!(
            adopted.audit_event.event_type,
            "memory_candidate_adopted_to_formal_memory"
        );
        assert_eq!(adopted.formal_store_revision, 1);
        assert_eq!(adopted.candidate_store_revision, 3);
        let candidate_store = memory_candidate_store::load_store(&path, "2026-06-03T00:00:03Z")
            .expect("candidate store should load");
        let linked = candidate_store
            .candidates
            .iter()
            .find(|item| item.candidate_key == candidate.candidate_key)
            .expect("candidate should remain");
        let adoption = linked
            .adoption
            .as_ref()
            .expect("adoption link should exist");
        assert_eq!(adoption.adopted_memory_id, adopted.record.memory_id);
        assert_eq!(adoption.adopted_version_id, adopted.version.version_id);
        assert_eq!(
            adoption.adopted_audit_event_id,
            adopted.audit_event.audit_event_id
        );
        let formal_store = formal_memory_store::load_store(&path, "2026-06-03T00:00:03Z")
            .expect("formal store should load");
        assert_eq!(formal_store.records.len(), 1);
        assert_eq!(formal_store.versions.len(), 1);
        assert_eq!(formal_store.audit_events.len(), 1);

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn memory_candidate_adoption_rejects_user_preference_without_user() {
        let dir = test_temp_dir("memory-candidate-adoption-user-preference");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/memory-candidate-adoption-project";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        let mut input = fixture_bound_memory_candidate_input(project_root);
        input.memory_type = "user_preference".to_string();
        input.scope = MemoryScope {
            scope_id: "scope:user:yoyi".to_string(),
            scope_type: "user_preference".to_string(),
            user_id: Some("yoyi".to_string()),
            project_id: None,
            workflow_id: None,
            session_id: None,
            role_ids: vec![],
            document_refs: vec![],
            permission_policy_ref: None,
            model_export_policy: "local_only".to_string(),
            valid_from: "2026-06-03T00:00:00Z".to_string(),
            valid_until: None,
        };
        input.requires_user_confirmation = true;
        input.generated_by_role = "user".to_string();
        let candidate = create_confirmed_memory_candidate(&path, input);

        let err = adopt_memory_candidate_to_formal_memory_at(
            &path,
            &fixture_adopt_memory_candidate_input(
                project_root,
                candidate.candidate_key,
                Some(2),
                Some(0),
            ),
            "2026-06-03T00:00:02Z",
            "write-memory-candidate-adoption",
            "write-formal-from-candidate",
        )
        .unwrap_err();

        assert!(err.contains("必须由 user 采纳"));
        assert!(!dir.join("formal-memories.v1.json").exists());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn memory_candidate_adoption_rejects_secret_without_blocked_export() {
        let dir = test_temp_dir("memory-candidate-adoption-secret");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/memory-candidate-adoption-project";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        let mut input = fixture_bound_memory_candidate_input(project_root);
        input.source_refs[0].sensitive_level = "secret".to_string();
        let candidate = create_confirmed_memory_candidate(&path, input);

        let err = adopt_memory_candidate_to_formal_memory_at(
            &path,
            &fixture_adopt_memory_candidate_input(
                project_root,
                candidate.candidate_key,
                Some(2),
                Some(0),
            ),
            "2026-06-03T00:00:02Z",
            "write-memory-candidate-adoption",
            "write-formal-from-candidate",
        )
        .unwrap_err();

        assert!(err.contains("secret 记忆候选必须阻止外发模型上下文"));
        assert!(!dir.join("formal-memories.v1.json").exists());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn memory_candidate_adoption_rejects_cross_project_project_director() {
        let dir = test_temp_dir("memory-candidate-adoption-cross-project");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/memory-candidate-adoption-project";
        let other_project_root = "/tmp/memory-candidate-adoption-other-project";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        bootstrap_project_workflow_at(&path, &fixture_project(other_project_root))
            .expect("workflow state should include other project");
        let candidate = create_confirmed_memory_candidate(
            &path,
            fixture_bound_memory_candidate_input(other_project_root),
        );

        let err = adopt_memory_candidate_to_formal_memory_at(
            &path,
            &fixture_adopt_memory_candidate_input(
                project_root,
                candidate.candidate_key,
                Some(2),
                Some(0),
            ),
            "2026-06-03T00:00:02Z",
            "write-memory-candidate-adoption",
            "write-formal-from-candidate",
        )
        .unwrap_err();

        assert!(err.contains("scope.project_id 与 project_root 不匹配"));
        assert!(!dir.join("formal-memories.v1.json").exists());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn memory_candidate_adoption_rejects_rejected_or_discarded_candidate() {
        let dir = test_temp_dir("memory-candidate-adoption-rejected");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/memory-candidate-adoption-project";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        let created = memory_candidate_store::create_candidate(
            &path,
            &fixture_bound_memory_candidate_input(project_root),
            "2026-06-03T00:00:00Z",
            "write-memory-candidate-adoption-create",
        )
        .expect("memory candidate should be created");
        memory_candidate_store::record_decision(
            &path,
            &RecordMemoryCandidateDecisionInput {
                project_root: project_root.to_string(),
                candidate_key: created.candidate.candidate_key.clone(),
                requested_status: MemoryLifecycleStatus::CandidateRejected,
                reason: "拒绝候选。".to_string(),
                actor_id: "project_director".to_string(),
                actor_role: "project_director".to_string(),
                expected_store_revision: Some(1),
            },
            "2026-06-03T00:00:01Z",
            "write-memory-candidate-adoption-reject",
        )
        .expect("candidate should be rejected");

        let err = adopt_memory_candidate_to_formal_memory_at(
            &path,
            &fixture_adopt_memory_candidate_input(
                project_root,
                created.candidate.candidate_key,
                Some(2),
                Some(0),
            ),
            "2026-06-03T00:00:02Z",
            "write-memory-candidate-adoption",
            "write-formal-from-candidate",
        )
        .unwrap_err();

        assert!(err.contains("只能采纳 candidate_confirmed"));
        assert!(!dir.join("formal-memories.v1.json").exists());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn memory_candidate_adoption_rejects_already_adopted_candidate() {
        let dir = test_temp_dir("memory-candidate-adoption-duplicate");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/memory-candidate-adoption-project";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        let candidate = create_confirmed_memory_candidate(
            &path,
            fixture_bound_memory_candidate_input(project_root),
        );
        adopt_memory_candidate_to_formal_memory_at(
            &path,
            &fixture_adopt_memory_candidate_input(
                project_root,
                candidate.candidate_key.clone(),
                Some(2),
                Some(0),
            ),
            "2026-06-03T00:00:02Z",
            "write-memory-candidate-adoption",
            "write-formal-from-candidate",
        )
        .expect("first adoption should succeed");

        let err = adopt_memory_candidate_to_formal_memory_at(
            &path,
            &fixture_adopt_memory_candidate_input(
                project_root,
                candidate.candidate_key,
                Some(3),
                Some(1),
            ),
            "2026-06-03T00:00:03Z",
            "write-memory-candidate-adoption-second",
            "write-formal-from-candidate-second",
        )
        .unwrap_err();

        assert!(err.contains("记忆候选已经采纳为正式记忆"));
        let formal_store = formal_memory_store::load_store(&path, "2026-06-03T00:00:04Z")
            .expect("formal store should load");
        assert_eq!(formal_store.records.len(), 1);

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn memory_candidate_adoption_rejects_context_binding_mismatch() {
        let dir = test_temp_dir("memory-candidate-adoption-context-mismatch");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/memory-candidate-adoption-project";
        let other_project_root = "/tmp/memory-candidate-adoption-other-project";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        bootstrap_project_workflow_at(&path, &fixture_project(other_project_root))
            .expect("workflow state should include other project");
        let candidate = create_confirmed_memory_candidate(
            &path,
            fixture_bound_memory_candidate_input(other_project_root),
        );

        let err = adopt_memory_candidate_to_formal_memory_at(
            &path,
            &fixture_adopt_memory_candidate_input(
                project_root,
                candidate.candidate_key,
                Some(2),
                Some(0),
            ),
            "2026-06-03T00:00:02Z",
            "write-memory-candidate-adoption",
            "write-formal-from-candidate",
        )
        .unwrap_err();

        assert!(err.contains("project_root 不匹配"));
        assert!(!dir.join("formal-memories.v1.json").exists());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn memory_candidate_rejection_does_not_create_formal_memory() {
        let dir = test_temp_dir("memory-candidate-rejection-no-formal");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/memory-candidate-adoption-project";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        let created = memory_candidate_store::create_candidate(
            &path,
            &fixture_bound_memory_candidate_input(project_root),
            "2026-06-03T00:00:00Z",
            "write-memory-candidate-rejection-create",
        )
        .expect("memory candidate should be created");
        memory_candidate_store::record_decision(
            &path,
            &RecordMemoryCandidateDecisionInput {
                project_root: project_root.to_string(),
                candidate_key: created.candidate.candidate_key,
                requested_status: MemoryLifecycleStatus::CandidateRejected,
                reason: "拒绝候选，不生成正式记忆。".to_string(),
                actor_id: "project_director".to_string(),
                actor_role: "project_director".to_string(),
                expected_store_revision: Some(1),
            },
            "2026-06-03T00:00:01Z",
            "write-memory-candidate-rejection",
        )
        .expect("candidate rejection should write only candidate store");

        let formal_store = formal_memory_store::load_store(&path, "2026-06-03T00:00:02Z")
            .expect("formal store should load empty");
        assert_eq!(formal_store.records.len(), 0);
        assert!(!dir.join("formal-memories.v1.json").exists());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn formal_memory_store_rejects_missing_source_refs() {
        let dir = test_temp_dir("formal-memory-missing-source");
        fs::create_dir_all(&dir).expect("temp dir should exist");
        let path = dir.join("workflow-state.v0.json");
        let mut input = fixture_formal_memory_input();
        input.source_refs = vec![];

        let err = formal_memory_store::create_record(
            &path,
            &input,
            "2026-06-03T00:00:00Z",
            "write-formal-missing-source",
        )
        .unwrap_err();

        assert!(err.contains("正式记忆缺少来源"));
        assert!(!dir.join("formal-memories.v1.json").exists());
    }

    #[test]
    fn formal_memory_store_rejects_candidate_status() {
        assert!(
            control_core::validate_formal_memory_status("candidate_confirmed")
                .unwrap_err()
                .contains("正式记忆初始状态只能是 memory_active")
        );
        assert!(control_core::validate_formal_memory_status("memory_active").is_ok());
    }

    #[test]
    fn formal_memory_store_keeps_candidate_store_separate() {
        let dir = test_temp_dir("formal-memory-candidate-separate");
        fs::create_dir_all(&dir).expect("temp dir should exist");
        let path = dir.join("workflow-state.v0.json");
        let candidate_input = fixture_memory_candidate_input();
        let created_candidate = memory_candidate_store::create_candidate(
            &path,
            &candidate_input,
            "2026-06-03T00:00:00Z",
            "write-memory-candidate-001",
        )
        .expect("memory candidate should write candidate sidecar");
        memory_candidate_store::record_decision(
            &path,
            &RecordMemoryCandidateDecisionInput {
                project_root: candidate_input.project_root,
                candidate_key: created_candidate.candidate.candidate_key,
                requested_status: MemoryLifecycleStatus::CandidateConfirmed,
                reason: "确认候选保留；不创建正式记忆。".to_string(),
                actor_id: "project_director".to_string(),
                actor_role: "project_director".to_string(),
                expected_store_revision: Some(1),
            },
            "2026-06-03T00:00:01Z",
            "write-memory-candidate-002",
        )
        .expect("candidate decision should stay in candidate store");

        let formal = formal_memory_store::load_store(&path, "2026-06-03T00:00:02Z")
            .expect("formal store should load empty even when candidate store exists");
        assert_eq!(formal.records.len(), 0);
        assert_eq!(formal.versions.len(), 0);
        assert_eq!(formal.audit_events.len(), 0);
        assert!(dir.join("memory-candidates.v1.json").exists());
        assert!(!dir.join("formal-memories.v1.json").exists());
    }

    #[test]
    fn formal_memory_store_damaged_json_is_not_overwritten() {
        let dir = test_temp_dir("formal-memory-damaged");
        fs::create_dir_all(&dir).expect("temp dir should exist");
        let path = dir.join("workflow-state.v0.json");
        let formal_path = dir.join("formal-memories.v1.json");
        fs::write(&formal_path, "{not valid json").expect("damaged formal sidecar should write");

        let err = formal_memory_store::create_record(
            &path,
            &fixture_formal_memory_input(),
            "2026-06-03T00:00:00Z",
            "write-formal-damaged",
        )
        .unwrap_err();

        assert!(err.contains("正式记忆 sidecar JSON 损坏"));
        assert_eq!(
            fs::read_to_string(&formal_path).expect("damaged formal sidecar should remain"),
            "{not valid json"
        );
    }

    #[test]
    fn formal_memory_store_revision_conflict_is_rejected() {
        let dir = test_temp_dir("formal-memory-revision-conflict");
        fs::create_dir_all(&dir).expect("temp dir should exist");
        let path = dir.join("workflow-state.v0.json");
        formal_memory_store::create_record(
            &path,
            &fixture_formal_memory_input(),
            "2026-06-03T00:00:00Z",
            "write-formal-rev-001",
        )
        .expect("first formal memory should write");

        let mut stale = fixture_formal_memory_input();
        stale.claim = "第二条正式记忆使用过期 revision。".to_string();
        stale.expected_store_revision = Some(0);
        let err = formal_memory_store::create_record(
            &path,
            &stale,
            "2026-06-03T00:00:01Z",
            "write-formal-rev-002",
        )
        .unwrap_err();

        assert!(err.contains("formal_memory_store_conflict"));
        let store = formal_memory_store::load_store(&path, "2026-06-03T00:00:02Z")
            .expect("store should still load after conflict");
        assert_eq!(store.records.len(), 1);
        assert_eq!(store.versions.len(), 1);
        assert_eq!(store.audit_events.len(), 1);
    }

    include!("lib_task_package_preview_binding_read_model_tests.rs");

    include!("lib_task_package_dispatch_preparation_tests.rs");

    #[test]
    fn workflow_node_dispatch_prepare_requires_binding_and_safe_probe_prompt() {
        let dir =
            std::env::temp_dir().join(format!("node-dispatch-prepare-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");
        let index = fixture_dispatch_index(&project.project_root, "thread-001");
        let draft = fixture_task_draft_request(&project.project_root, "节点派发工作项");

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_task_draft_at(&path, &draft).expect("work item should exist");
        let value = read_json_file(&path);
        let work_item_id = optional_string_from(&value["work_items"][0], "work_item_id")
            .expect("work item id should exist");
        update_work_item_state_at(
            &path,
            &fixture_work_item_state_update_request(
                &project.project_root,
                &work_item_id,
                "ready_to_dispatch",
            ),
        )
        .expect("work item should be ready before dispatch prepare");
        let workflow_id = default_workflow_id(&project.project_root);
        let node_id = format!("{workflow_id}:node:codex-dev");
        let prepare_request =
            fixture_dispatch_prepare_request(&project.project_root, &node_id, &work_item_id);

        let missing_binding =
            prepare_workflow_node_dispatch_for_index_at(&path, &index, &prepare_request);
        assert!(missing_binding.is_err());
        assert!(missing_binding
            .unwrap_err()
            .contains("没有 active Codex 会话绑定"));

        let session = fixture_session("thread-001", &project.project_root, true);
        bind_workflow_node_codex_session_at(
            &path,
            &fixture_node_session_bind_request(
                &project.project_root,
                &node_id,
                Some(&work_item_id),
                "thread-001",
            ),
            &session,
        )
        .expect("binding should write");
        create_active_plan_authorization_for_fixture(&path, &project.project_root);
        let prepared = prepare_workflow_node_dispatch_for_index_at(&path, &index, &prepare_request)
            .expect("safe probe prepare should write");

        assert_eq!(prepared.dispatch.state, "prepared");
        assert_eq!(prepared.dispatch.prompt_kind, "safe_probe");
        assert_eq!(prepared.dispatch.prompt_preview, safe_probe_prompt());
        assert_eq!(
            prepared.snapshot.project_workflows[0].node_dispatches.len(),
            1
        );
        let updated = read_json_file(&path);
        assert_eq!(updated["workflow_node_dispatches"][0]["state"], "prepared");
        assert!(updated["audit_events"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| event["event_type"] == "workflow_node_dispatch_prepared"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn workflow_node_dispatch_prepare_rejects_non_ready_work_item() {
        let dir = std::env::temp_dir().join(format!(
            "node-dispatch-prepare-rejects-draft-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");
        let index = fixture_dispatch_index(&project.project_root, "thread-001");
        let draft = fixture_task_draft_request(&project.project_root, "草稿态派发准备");

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_task_draft_at(&path, &draft).expect("work item should exist");
        let value = read_json_file(&path);
        let work_item_id = optional_string_from(&value["work_items"][0], "work_item_id")
            .expect("work item id should exist");
        let workflow_id = default_workflow_id(&project.project_root);
        let node_id = format!("{workflow_id}:node:codex-dev");
        let session = fixture_session("thread-001", &project.project_root, true);
        bind_workflow_node_codex_session_at(
            &path,
            &fixture_node_session_bind_request(
                &project.project_root,
                &node_id,
                Some(&work_item_id),
                "thread-001",
            ),
            &session,
        )
        .expect("binding should write");

        let result = prepare_workflow_node_dispatch_for_index_at(
            &path,
            &index,
            &fixture_dispatch_prepare_request(&project.project_root, &node_id, &work_item_id),
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("控制核心已拒绝准备派发"));
        let updated = read_json_file(&path);
        assert_eq!(
            updated["workflow_node_dispatches"]
                .as_array()
                .expect("dispatches should be array")
                .len(),
            0
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn workflow_node_dispatch_started_marks_actual_dispatch_node_running() {
        let dir =
            std::env::temp_dir().join(format!("node-dispatch-started-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");
        let index = fixture_dispatch_index(&project.project_root, "thread-001");
        let draft = fixture_task_draft_request(&project.project_root, "节点派发开始工作项");

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_task_draft_at(&path, &draft).expect("work item should exist");
        let value = read_json_file(&path);
        let work_item_id = optional_string_from(&value["work_items"][0], "work_item_id")
            .expect("work item id should exist");
        update_work_item_state_at(
            &path,
            &fixture_work_item_state_update_request(
                &project.project_root,
                &work_item_id,
                "ready_to_dispatch",
            ),
        )
        .expect("work item should be ready");
        let workflow_id = default_workflow_id(&project.project_root);
        let node_id = format!("{workflow_id}:node:codex-dev");
        let session = fixture_session("thread-001", &project.project_root, true);
        bind_workflow_node_codex_session_at(
            &path,
            &fixture_node_session_bind_request(
                &project.project_root,
                &node_id,
                Some(&work_item_id),
                "thread-001",
            ),
            &session,
        )
        .expect("binding should write");
        let prepare_request =
            fixture_dispatch_prepare_request(&project.project_root, &node_id, &work_item_id);
        let context = workflow_node_dispatch_context(&path, &index, &prepare_request)
            .expect("dispatch context should resolve");

        let dispatch =
            write_started_dispatch(&path, &context).expect("started dispatch should write");

        assert_eq!(dispatch.state, "running");
        assert_eq!(dispatch.node_id, node_id);
        let updated = read_json_file(&path);
        assert_eq!(updated["work_items"][0]["state"], "running");
        assert_eq!(updated["work_items"][0]["current_node_id"], node_id);
        assert_eq!(
            fixture_node_state(&updated, &node_id).as_deref(),
            Some("running")
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn workflow_node_dispatch_execute_uses_stub_and_advances_to_review() {
        let dir =
            std::env::temp_dir().join(format!("node-dispatch-execute-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        let index_path = dir.join("codex-index.json");
        let project = fixture_project("/tmp/indexed-project");
        let index = fixture_dispatch_index(&project.project_root, "thread-001");
        let draft = fixture_task_draft_request(&project.project_root, "节点派发执行工作项");

        fs::create_dir_all(&dir).expect("fixture dir should exist");
        fs::write(&index_path, "{}").expect("index fixture should write");
        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_task_draft_at(&path, &draft).expect("work item should exist");
        let value = read_json_file(&path);
        let work_item_id = optional_string_from(&value["work_items"][0], "work_item_id")
            .expect("work item id should exist");
        update_work_item_state_at(
            &path,
            &fixture_work_item_state_update_request(
                &project.project_root,
                &work_item_id,
                "ready_to_dispatch",
            ),
        )
        .expect("work item should be ready");
        let workflow_id = default_workflow_id(&project.project_root);
        let node_id = format!("{workflow_id}:node:codex-dev");
        let session = fixture_session("thread-001", &project.project_root, true);
        bind_workflow_node_codex_session_at(
            &path,
            &fixture_node_session_bind_request(
                &project.project_root,
                &node_id,
                Some(&work_item_id),
                "thread-001",
            ),
            &session,
        )
        .expect("binding should write");
        let runner = StubCodexResumeRunner {
            stats: CodexDispatchReadbackStats {
                transcript_event_count: 12,
                transcript_target_hits: 1,
            },
        };
        DISPATCH_READBACK_NATIVE_READ_COUNT.with(|count| count.set(0));

        let result = execute_workflow_node_dispatch_for_index_at(
            &path,
            &index,
            &index_path,
            &runner,
            &fixture_dispatch_execute_request(&project.project_root, &node_id, &work_item_id),
        )
        .expect("safe probe execute should complete");

        assert_eq!(result.dispatch.state, "completed");
        assert!(!result.product_command_boundary.h5_unified_product_command);
        assert!(result.product_command_boundary.deprecated);
        assert!(
            !result
                .product_command_boundary
                .product_routing_allows_real_execution
        );
        assert!(result
            .dispatch
            .warnings
            .contains(&"legacy_workflow_node_dispatch_not_h5_unified_product_command".to_string()));
        assert_eq!(result.dispatch.exit_code, Some(0));
        assert_eq!(result.dispatch.transcript_event_count, Some(12));
        assert_eq!(result.dispatch.transcript_target_hits, Some(1));
        DISPATCH_READBACK_NATIVE_READ_COUNT.with(|count| assert_eq!(count.get(), 0));
        assert_eq!(
            result.snapshot.project_workflows[0].task_drafts[0].state,
            "ready_for_review"
        );
        assert_eq!(
            result.dispatch.last_message_summary.as_deref(),
            Some(safe_probe_target())
        );
        let updated = read_json_file(&path);
        assert_eq!(updated["work_items"][0]["state"], "ready_for_review");
        assert_eq!(
            updated["work_items"][0]["current_node_id"],
            format!("{workflow_id}:node:review")
        );
        assert_eq!(
            fixture_node_state(&updated, &node_id).as_deref(),
            Some("ready_for_review")
        );
        assert_ne!(
            fixture_node_state(&updated, &node_id).as_deref(),
            Some("running")
        );
        assert_eq!(
            fixture_node_state(&updated, &format!("{workflow_id}:node:review")).as_deref(),
            Some("ready_for_review")
        );
        assert!(updated["workflow_node_dispatches"]
            .as_array()
            .unwrap()
            .iter()
            .any(|dispatch| dispatch["state"] == "prepared"));
        assert!(updated["workflow_node_dispatches"]
            .as_array()
            .unwrap()
            .iter()
            .any(|dispatch| dispatch["state"] == "completed"));
        assert!(updated["audit_events"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| event["event_type"] == "workflow_node_dispatch_started"));
        assert!(updated["audit_events"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| event["event_type"] == "workflow_node_dispatch_completed"));
        assert!(updated["audit_events"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| event["event_type"] == "workflow_node_dispatch_readback_completed"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn legacy_real_execution_entrypoints_are_blocked_for_product_routing() {
        let boundary = legacy_product_command_boundary("execute_workflow_node_dispatch");
        assert!(!boundary.h5_unified_product_command);
        assert!(boundary.deprecated);
        assert!(!boundary.product_routing_allows_real_execution);
        assert!(boundary.legacy_path_may_have_real_side_effects);
        assert_eq!(
            boundary.replacement_command.as_deref(),
            Some("preview_h5_project_workflow_dispatch + controlled_session_continuation")
        );
        assert!(boundary
            .warnings
            .contains(&"real_execution_command_gate_v1".to_string()));

        let cli_error = run_workflow_machine_cli(vec![
            "/tmp/project".to_string(),
            "work-item".to_string(),
            "objective".to_string(),
            "1".to_string(),
            "30".to_string(),
        ])
        .expect_err("legacy CLI entry should be blocked before any real runner");
        assert!(cli_error.contains("legacy_product_command_blocked"));
        assert!(cli_error.contains("__run_workflow_machine_real"));
    }

    #[test]
    fn workflow_node_dispatch_execute_without_stub_stats_uses_native_readback() {
        let dir = std::env::temp_dir().join(format!(
            "node-dispatch-native-readback-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");
        let fixture = dispatch_readback_fixture(
            "node-dispatch-native-readback-rollout",
            "thread-001",
            vec![
                dispatch_text_event("native readback noise"),
                dispatch_stdout_event(safe_probe_target()),
            ],
        );
        let index = json!({
            "projects": [{ "project_root": project.project_root }],
            "threads": [
                {
                    "thread_id": "thread-001",
                    "project_root": project.project_root,
                    "title": "Session thread-001",
                    "rollout_exists": true,
                    "rollout_path": fixture.codex_home.join("sessions").join("thread-001.jsonl").display().to_string()
                }
            ],
            "source_stats": {
                "codex_home": {
                    "path": fixture.codex_home.display().to_string(),
                    "role": "data_source_root"
                }
            }
        });
        let draft = fixture_task_draft_request(&project.project_root, "原生读回执行工作项");

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_task_draft_at(&path, &draft).expect("work item should exist");
        let value = read_json_file(&path);
        let work_item_id = optional_string_from(&value["work_items"][0], "work_item_id")
            .expect("work item id should exist");
        update_work_item_state_at(
            &path,
            &fixture_work_item_state_update_request(
                &project.project_root,
                &work_item_id,
                "ready_to_dispatch",
            ),
        )
        .expect("work item should be ready");
        let workflow_id = default_workflow_id(&project.project_root);
        let node_id = format!("{workflow_id}:node:codex-dev");
        let session = fixture_session("thread-001", &project.project_root, true);
        bind_workflow_node_codex_session_at(
            &path,
            &fixture_node_session_bind_request(
                &project.project_root,
                &node_id,
                Some(&work_item_id),
                "thread-001",
            ),
            &session,
        )
        .expect("binding should write");
        let runner = NoReadbackStatsCodexResumeRunner;
        DISPATCH_READBACK_NATIVE_READ_COUNT.with(|count| count.set(0));

        let result = execute_workflow_node_dispatch_for_index_at(
            &path,
            &index,
            &fixture.db_path,
            &runner,
            &fixture_dispatch_execute_request(&project.project_root, &node_id, &work_item_id),
        )
        .expect("execute should complete with native readback stats");

        assert_eq!(result.dispatch.state, "completed");
        assert_eq!(result.dispatch.transcript_event_count, Some(2));
        assert_eq!(result.dispatch.transcript_target_hits, Some(1));
        DISPATCH_READBACK_NATIVE_READ_COUNT.with(|count| assert_eq!(count.get(), 1));
        assert!(!fixture
            .codex_home
            .parent()
            .expect("fixture codex home should have parent")
            .join("transcript_reader.py")
            .exists());

        let _ = fs::remove_dir_all(dir);
        let _ = fs::remove_dir_all(
            fixture
                .codex_home
                .parent()
                .expect("fixture codex home should have parent"),
        );
    }

    #[test]
    fn workflow_node_dispatch_execute_rejects_user_reviewed_instruction_without_payload() {
        let dir = std::env::temp_dir().join(format!(
            "node-dispatch-user-reviewed-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let index_path = dir.join("codex-index.json");
        let project = fixture_project("/tmp/indexed-project");
        let index = fixture_dispatch_index(&project.project_root, "thread-001");
        let draft = fixture_task_draft_request(&project.project_root, "业务派发阻塞工作项");

        fs::create_dir_all(&dir).expect("fixture dir should exist");
        fs::write(&index_path, "{}").expect("index fixture should write");
        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_task_draft_at(&path, &draft).expect("work item should exist");
        let value = read_json_file(&path);
        let work_item_id = optional_string_from(&value["work_items"][0], "work_item_id")
            .expect("work item id should exist");
        update_work_item_state_at(
            &path,
            &fixture_work_item_state_update_request(
                &project.project_root,
                &work_item_id,
                "ready_to_dispatch",
            ),
        )
        .expect("work item should be ready");
        let workflow_id = default_workflow_id(&project.project_root);
        let node_id = format!("{workflow_id}:node:codex-dev");
        let session = fixture_session("thread-001", &project.project_root, true);
        bind_workflow_node_codex_session_at(
            &path,
            &fixture_node_session_bind_request(
                &project.project_root,
                &node_id,
                Some(&work_item_id),
                "thread-001",
            ),
            &session,
        )
        .expect("binding should write");
        let runner = StubCodexResumeRunner {
            stats: CodexDispatchReadbackStats {
                transcript_event_count: 0,
                transcript_target_hits: 0,
            },
        };
        let request = WorkflowNodeDispatchExecuteRequest {
            project_root: project.project_root.clone(),
            node_id,
            work_item_id: work_item_id.clone(),
            prompt_kind: "user_reviewed_instruction".to_string(),
            user_reviewed_instruction: None,
        };

        let result = execute_workflow_node_dispatch_for_index_at(
            &path,
            &index,
            &index_path,
            &runner,
            &request,
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("阻止真实业务派发"));
        let updated = read_json_file(&path);
        assert_eq!(updated["work_items"][0]["state"], "ready_to_dispatch");
        assert_eq!(
            updated["workflow_node_dispatches"]
                .as_array()
                .unwrap()
                .len(),
            0
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn workflow_node_dispatch_execute_user_reviewed_instruction_uses_codex_options() {
        let dir = std::env::temp_dir().join(format!(
            "node-dispatch-user-reviewed-success-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let index_path = dir.join("codex-index.json");
        let project = fixture_project("/tmp/indexed-project");
        let index = fixture_dispatch_index(&project.project_root, "thread-001");
        let draft = fixture_task_draft_request(&project.project_root, "业务派发执行工作项");

        fs::create_dir_all(&dir).expect("fixture dir should exist");
        fs::write(&index_path, "{}").expect("index fixture should write");
        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_task_draft_at(&path, &draft).expect("work item should exist");
        let value = read_json_file(&path);
        let work_item_id = optional_string_from(&value["work_items"][0], "work_item_id")
            .expect("work item id should exist");
        update_work_item_state_at(
            &path,
            &fixture_work_item_state_update_request(
                &project.project_root,
                &work_item_id,
                "ready_to_dispatch",
            ),
        )
        .expect("work item should be ready");
        let workflow_id = default_workflow_id(&project.project_root);
        let node_id = format!("{workflow_id}:node:codex-dev");
        let session = fixture_session("thread-001", &project.project_root, true);
        bind_workflow_node_codex_session_at(
            &path,
            &fixture_node_session_bind_request(
                &project.project_root,
                &node_id,
                Some(&work_item_id),
                "thread-001",
            ),
            &session,
        )
        .expect("binding should write");
        let instruction = fixture_user_reviewed_instruction();
        let runner = StubCodexResumeRunner {
            stats: CodexDispatchReadbackStats {
                transcript_event_count: 21,
                transcript_target_hits: 2,
            },
        };

        let result = execute_workflow_node_dispatch_for_index_at(
            &path,
            &index,
            &index_path,
            &runner,
            &WorkflowNodeDispatchExecuteRequest {
                project_root: project.project_root.clone(),
                node_id: node_id.clone(),
                work_item_id: work_item_id.clone(),
                prompt_kind: "user_reviewed_instruction".to_string(),
                user_reviewed_instruction: Some(instruction.clone()),
            },
        )
        .expect("user reviewed dispatch should complete with stub runner");

        assert_eq!(result.dispatch.state, "completed");
        assert_eq!(result.dispatch.prompt_kind, "user_reviewed_instruction");
        assert_eq!(
            result
                .dispatch
                .user_reviewed_instruction
                .as_ref()
                .map(|instruction| instruction.execution_cwd.as_str()),
            Some("/Users/yoyi")
        );
        assert_eq!(
            result.dispatch.last_message_summary.as_deref(),
            Some("USER_REVIEWED_STUB_OK")
        );
        let updated = read_json_file(&path);
        assert_eq!(updated["work_items"][0]["state"], "ready_for_review");
        assert_eq!(
            updated["work_items"][0]["current_node_id"],
            format!("{workflow_id}:node:review")
        );
        assert_eq!(
            fixture_node_state(&updated, &node_id).as_deref(),
            Some("ready_for_review")
        );
        assert_ne!(
            fixture_node_state(&updated, &node_id).as_deref(),
            Some("running")
        );
        assert!(updated["workflow_execution_controls"]
            .as_array()
            .unwrap()
            .iter()
            .any(|control| control["work_item_id"] == work_item_id
                && control["user_reviewed_instruction"]["execution_cwd"] == "/Users/yoyi"));
        assert!(updated["execution_attempts"]
            .as_array()
            .unwrap()
            .iter()
            .any(|attempt| attempt["work_item_id"] == work_item_id
                && attempt["state"] == "completed"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn workflow_node_dispatch_readback_restores_user_reviewed_instruction_payload() {
        let dir = std::env::temp_dir().join(format!(
            "node-dispatch-readback-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let index_path = dir.join("codex-index.json");
        let project = fixture_project("/tmp/indexed-project");
        let index = fixture_dispatch_index(&project.project_root, "thread-001");
        let draft = fixture_task_draft_request(&project.project_root, "业务派发读回工作项");

        fs::create_dir_all(&dir).expect("fixture dir should exist");
        fs::write(&index_path, "{}").expect("index fixture should write");
        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_task_draft_at(&path, &draft).expect("work item should exist");
        let value = read_json_file(&path);
        let work_item_id = optional_string_from(&value["work_items"][0], "work_item_id")
            .expect("work item id should exist");
        update_work_item_state_at(
            &path,
            &fixture_work_item_state_update_request(
                &project.project_root,
                &work_item_id,
                "ready_to_dispatch",
            ),
        )
        .expect("work item should be ready");
        let workflow_id = default_workflow_id(&project.project_root);
        let node_id = format!("{workflow_id}:node:codex-dev");
        let session = fixture_session("thread-001", &project.project_root, true);
        bind_workflow_node_codex_session_at(
            &path,
            &fixture_node_session_bind_request(
                &project.project_root,
                &node_id,
                Some(&work_item_id),
                "thread-001",
            ),
            &session,
        )
        .expect("binding should write");
        create_active_plan_authorization_for_fixture(&path, &project.project_root);
        let prepared = prepare_workflow_node_dispatch_for_index_at(
            &path,
            &index,
            &WorkflowNodeDispatchPrepareRequest {
                project_root: project.project_root.clone(),
                node_id,
                work_item_id: work_item_id.clone(),
                prompt_kind: "user_reviewed_instruction".to_string(),
                user_reviewed_instruction: Some(fixture_user_reviewed_instruction()),
            },
        )
        .expect("business prepare should write payload");

        let result = read_workflow_node_dispatch_result_at(
            &path,
            &index,
            &index_path,
            &WorkflowNodeDispatchReadbackRequest {
                project_root: project.project_root.clone(),
                dispatch_id: prepared.dispatch.dispatch_id.clone(),
            },
        )
        .expect("readback should restore user reviewed payload from dispatch");

        assert_eq!(result.dispatch.prompt_kind, "user_reviewed_instruction");
        assert_eq!(result.dispatch.transcript_event_count, Some(0));
        assert_eq!(
            result
                .dispatch
                .user_reviewed_instruction
                .as_ref()
                .map(|instruction| instruction.execution_cwd.as_str()),
            Some("/Users/yoyi")
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn workflow_node_dispatch_user_reviewed_failure_writes_control_and_attempt() {
        let dir = std::env::temp_dir().join(format!(
            "node-dispatch-user-reviewed-failed-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let index_path = dir.join("codex-index.json");
        let project = fixture_project("/tmp/indexed-project");
        let index = fixture_dispatch_index(&project.project_root, "thread-001");
        let draft = fixture_task_draft_request(&project.project_root, "业务派发失败工作项");

        fs::create_dir_all(&dir).expect("fixture dir should exist");
        fs::write(&index_path, "{}").expect("index fixture should write");
        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_task_draft_at(&path, &draft).expect("work item should exist");
        let value = read_json_file(&path);
        let work_item_id = optional_string_from(&value["work_items"][0], "work_item_id")
            .expect("work item id should exist");
        update_work_item_state_at(
            &path,
            &fixture_work_item_state_update_request(
                &project.project_root,
                &work_item_id,
                "ready_to_dispatch",
            ),
        )
        .expect("work item should be ready");
        let workflow_id = default_workflow_id(&project.project_root);
        let node_id = format!("{workflow_id}:node:codex-dev");
        let session = fixture_session("thread-001", &project.project_root, true);
        bind_workflow_node_codex_session_at(
            &path,
            &fixture_node_session_bind_request(
                &project.project_root,
                &node_id,
                Some(&work_item_id),
                "thread-001",
            ),
            &session,
        )
        .expect("binding should write");
        let mut instruction = fixture_user_reviewed_instruction();
        instruction.allowed_writes = vec!["/tmp/outside-target/file.md".to_string()];
        let runner = FailingCodexResumeRunner {
            exit_code: 7,
            timed_out: false,
        };

        let result = execute_workflow_node_dispatch_for_index_at(
            &path,
            &index,
            &index_path,
            &runner,
            &WorkflowNodeDispatchExecuteRequest {
                project_root: project.project_root.clone(),
                node_id: node_id.clone(),
                work_item_id: work_item_id.clone(),
                prompt_kind: "user_reviewed_instruction".to_string(),
                user_reviewed_instruction: Some(instruction),
            },
        )
        .expect("failed business dispatch should still write workflow state");

        assert_eq!(result.dispatch.state, "failed");
        let updated = read_json_file(&path);
        assert_eq!(updated["work_items"][0]["state"], "failed");
        assert_eq!(updated["work_items"][0]["current_node_id"], node_id);
        assert_eq!(
            fixture_node_state(&updated, &node_id).as_deref(),
            Some("failed")
        );
        assert_ne!(
            fixture_node_state(&updated, &node_id).as_deref(),
            Some("running")
        );
        let warnings = updated["workflow_node_dispatches"]
            .as_array()
            .unwrap()
            .iter()
            .find(|dispatch| dispatch["state"] == "failed")
            .unwrap()["warnings"]
            .as_array()
            .unwrap()
            .clone();
        assert!(warnings
            .iter()
            .any(|warning| warning == "codex_resume_exit_nonzero"));
        assert!(warnings
            .iter()
            .any(|warning| warning == "target_path_not_writable"));
        assert!(updated["workflow_execution_controls"]
            .as_array()
            .unwrap()
            .iter()
            .any(|control| control["work_item_id"] == work_item_id
                && control["long_task_state"] == "failed"
                && control["failure_reason"]
                    .as_str()
                    .unwrap()
                    .contains("target_path_not_writable")));
        assert!(updated["execution_attempts"]
            .as_array()
            .unwrap()
            .iter()
            .any(|attempt| attempt["work_item_id"] == work_item_id
                && attempt["state"] == "failed"
                && attempt["warnings"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|warning| warning == "codex_resume_exit_nonzero")));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn workflow_node_dispatch_user_reviewed_timeout_writes_timed_out_attempt() {
        let dir = std::env::temp_dir().join(format!(
            "node-dispatch-user-reviewed-timeout-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let index_path = dir.join("codex-index.json");
        let project = fixture_project("/tmp/indexed-project");
        let index = fixture_dispatch_index(&project.project_root, "thread-001");
        let draft = fixture_task_draft_request(&project.project_root, "业务派发超时工作项");

        fs::create_dir_all(&dir).expect("fixture dir should exist");
        fs::write(&index_path, "{}").expect("index fixture should write");
        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_task_draft_at(&path, &draft).expect("work item should exist");
        let value = read_json_file(&path);
        let work_item_id = optional_string_from(&value["work_items"][0], "work_item_id")
            .expect("work item id should exist");
        update_work_item_state_at(
            &path,
            &fixture_work_item_state_update_request(
                &project.project_root,
                &work_item_id,
                "ready_to_dispatch",
            ),
        )
        .expect("work item should be ready");
        let workflow_id = default_workflow_id(&project.project_root);
        let node_id = format!("{workflow_id}:node:codex-dev");
        let session = fixture_session("thread-001", &project.project_root, true);
        bind_workflow_node_codex_session_at(
            &path,
            &fixture_node_session_bind_request(
                &project.project_root,
                &node_id,
                Some(&work_item_id),
                "thread-001",
            ),
            &session,
        )
        .expect("binding should write");
        let runner = FailingCodexResumeRunner {
            exit_code: -1,
            timed_out: true,
        };

        let result = execute_workflow_node_dispatch_for_index_at(
            &path,
            &index,
            &index_path,
            &runner,
            &WorkflowNodeDispatchExecuteRequest {
                project_root: project.project_root.clone(),
                node_id: node_id.clone(),
                work_item_id: work_item_id.clone(),
                prompt_kind: "user_reviewed_instruction".to_string(),
                user_reviewed_instruction: Some(fixture_user_reviewed_instruction()),
            },
        )
        .expect("timed out business dispatch should still write workflow state");

        assert_eq!(result.dispatch.state, "failed");
        let updated = read_json_file(&path);
        assert_eq!(updated["work_items"][0]["state"], "timed_out");
        assert_eq!(updated["work_items"][0]["current_node_id"], node_id);
        assert_eq!(
            fixture_node_state(&updated, &node_id).as_deref(),
            Some("timed_out")
        );
        assert_ne!(
            fixture_node_state(&updated, &node_id).as_deref(),
            Some("running")
        );
        assert!(updated["execution_attempts"]
            .as_array()
            .unwrap()
            .iter()
            .any(|attempt| attempt["work_item_id"] == work_item_id
                && attempt["state"] == "timed_out"
                && attempt["timed_out_at"].is_string()
                && attempt["warnings"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|warning| warning == "timeout")));
        assert!(updated["workflow_execution_controls"]
            .as_array()
            .unwrap()
            .iter()
            .any(|control| control["work_item_id"] == work_item_id
                && control["long_task_state"] == "timed_out"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn workflow_node_dispatch_user_reviewed_instruction_validates_permission_fields() {
        let mut instruction = fixture_user_reviewed_instruction();
        instruction.allowed_write_roots = vec![];
        assert!(validate_user_reviewed_instruction(&instruction)
            .unwrap_err()
            .contains("allowed_write_roots"));

        let mut invalid_sandbox = fixture_user_reviewed_instruction();
        invalid_sandbox.sandbox_mode = "full-access".to_string();
        assert!(validate_user_reviewed_instruction(&invalid_sandbox)
            .unwrap_err()
            .contains("sandbox_mode"));

        let mut retrying = fixture_user_reviewed_instruction();
        retrying.max_retries = 1;
        assert!(validate_user_reviewed_instruction(&retrying)
            .unwrap_err()
            .contains("自动重试还未产品化"));
    }

    #[test]
    fn workflow_machine_runs_four_role_loop_to_acceptance() {
        let dir =
            std::env::temp_dir().join(format!("workflow-machine-loop-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        let index_path = dir.join("codex-index.json");
        let project = fixture_project("/tmp/indexed-project");
        let draft = fixture_task_draft_request(&project.project_root, "马里奥 demo 闭环验收");

        fs::create_dir_all(&dir).expect("fixture dir should exist");
        fs::write(&index_path, "{}").expect("index fixture should write");
        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_task_draft_at(&path, &draft).expect("work item should exist");
        let value = read_json_file(&path);
        let work_item_id = optional_string_from(&value["work_items"][0], "work_item_id")
            .expect("work item id should exist");
        update_work_item_state_at(
            &path,
            &fixture_work_item_state_update_request(
                &project.project_root,
                &work_item_id,
                "ready_to_dispatch",
            ),
        )
        .expect("work item should be ready");
        let workflow_id = default_workflow_id(&project.project_root);
        let roles = [
            ("director", "thread-director"),
            ("codex-dev", "thread-dev"),
            ("validation", "thread-validation"),
            ("review", "thread-review"),
        ];
        for (role_id, thread_id) in roles {
            let node_id = format!("{workflow_id}:node:{role_id}");
            let session = fixture_session(thread_id, &project.project_root, true);
            bind_workflow_node_codex_session_at(
                &path,
                &fixture_node_session_bind_request(
                    &project.project_root,
                    &node_id,
                    Some(&work_item_id),
                    thread_id,
                ),
                &session,
            )
            .expect("role binding should write");
        }
        let index = fixture_multi_thread_index(
            &project.project_root,
            &[
                "thread-director",
                "thread-dev",
                "thread-validation",
                "thread-review",
            ],
        );
        let runner = WorkflowMachineStubRunner {
            stats: CodexDispatchReadbackStats {
                transcript_event_count: 8,
                transcript_target_hits: 1,
            },
        };

        let result = run_workflow_machine_for_index_at(
            &path,
            &index,
            &index_path,
            &runner,
            &WorkflowMachineRunRequest {
                project_root: project.project_root.clone(),
                work_item_id: work_item_id.clone(),
                objective: "完成马里奥 demo".to_string(),
                execution_root: None,
                max_rounds: 2,
                timeout_seconds_per_step: 600,
            },
        )
        .expect("workflow machine should complete");

        assert_eq!(result.final_state, "accepted");
        assert_eq!(result.rounds_completed, 1);
        assert_eq!(result.steps.len(), 5);
        assert_eq!(
            result
                .steps
                .iter()
                .map(|step| step.role_id.as_str())
                .collect::<Vec<_>>(),
            vec!["director", "codex-dev", "validation", "review", "director"]
        );
        let updated = read_json_file(&path);
        assert_eq!(updated["work_items"][0]["state"], "accepted");
        assert_eq!(
            updated["workflow_machine_runs"][0]["state"],
            Value::String("accepted".to_string())
        );
        assert_eq!(
            updated["workflow_node_dispatches"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|dispatch| dispatch["work_item_id"] == work_item_id)
                .count(),
            10
        );
        assert_eq!(
            result
                .steps
                .iter()
                .map(|step| step.native_thread_id.as_str())
                .collect::<Vec<_>>(),
            vec![
                "thread-director",
                "thread-dev",
                "thread-validation",
                "thread-review",
                "thread-director"
            ]
        );
        assert!(result.steps.iter().all(|step| step
            .warnings
            .iter()
            .all(|warning| !warning.starts_with("local_"))));
        assert!(updated["audit_events"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| event["event_type"] == "workflow_machine_run_finished"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn compact_last_message_summary_preserves_workflow_machine_control_marker() {
        let long_intro = "确认事项。".repeat(80);
        let summary = compact_last_message_summary(&format!(
            "{long_intro}\n最终结论：通过\nWORKFLOW_MACHINE_FINAL_ACCEPTED"
        ));

        assert!(summary.len() > 240);
        assert!(summary.contains("WORKFLOW_MACHINE_FINAL_ACCEPTED"));
        assert!(workflow_machine_final_acceptance(&summary));
    }

    include!("lib_workflow_governance_boundary_tests.rs");

    #[test]
    fn workflow_dispatch_director_review_records_completed_dispatch() {
        let dir = std::env::temp_dir().join(format!("director-review-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        let index_path = dir.join("codex-index.json");
        let project = fixture_project("/tmp/indexed-project");
        let index = fixture_dispatch_index(&project.project_root, "thread-001");
        let draft = fixture_task_draft_request(&project.project_root, "总指导回收工作项");

        fs::create_dir_all(&dir).expect("fixture dir should exist");
        fs::write(&index_path, "{}").expect("index fixture should write");
        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_task_draft_at(&path, &draft).expect("work item should exist");
        let value = read_json_file(&path);
        let work_item_id = optional_string_from(&value["work_items"][0], "work_item_id")
            .expect("work item id should exist");
        update_work_item_state_at(
            &path,
            &fixture_work_item_state_update_request(
                &project.project_root,
                &work_item_id,
                "ready_to_dispatch",
            ),
        )
        .expect("work item should be ready");
        let node_id = format!(
            "{}:node:director",
            default_workflow_id(&project.project_root)
        );
        let session = fixture_session("thread-001", &project.project_root, true);
        bind_workflow_node_codex_session_at(
            &path,
            &fixture_node_session_bind_request(
                &project.project_root,
                &node_id,
                Some(&work_item_id),
                "thread-001",
            ),
            &session,
        )
        .expect("binding should write");
        let runner = StubCodexResumeRunner {
            stats: CodexDispatchReadbackStats {
                transcript_event_count: 32,
                transcript_target_hits: 4,
            },
        };
        let dispatch_result = execute_workflow_node_dispatch_for_index_at(
            &path,
            &index,
            &index_path,
            &runner,
            &fixture_dispatch_execute_request(&project.project_root, &node_id, &work_item_id),
        )
        .expect("safe probe execute should complete");

        let result = record_workflow_dispatch_director_review_at(
            &path,
            &WorkflowDispatchDirectorReviewRequest {
                project_root: project.project_root.clone(),
                work_item_id: work_item_id.clone(),
                dispatch_id: dispatch_result.dispatch.dispatch_id.clone(),
                decision: "accepted".to_string(),
                summary: "总指导回收：接受；派发结果：WORKFLOW_NODE_DISPATCH_OK_2026_05_29"
                    .to_string(),
            },
        )
        .expect("director review should write");

        assert!(result.backup_path.is_some());
        assert_eq!(
            result
                .audit_event_id
                .contains("workflow-dispatch-director-review"),
            true
        );
        assert_eq!(result.snapshot.counts.reviews, 1);
        let updated = read_json_file(&path);
        assert_eq!(updated["work_items"][0]["state"], "ready_for_review");
        assert_eq!(updated["reviews"][0]["work_item_id"], work_item_id);
        assert_eq!(
            updated["reviews"][0]["dispatch_id"],
            dispatch_result.dispatch.dispatch_id
        );
        assert_eq!(updated["reviews"][0]["reviewer_role"], "director");
        assert_eq!(updated["reviews"][0]["decision"], "accepted");
        assert!(
            updated["reviews"][0]["evidence_refs"]
                .as_array()
                .unwrap()
                .len()
                > 0
        );
        assert!(
            updated["reviews"][0]["handoff_refs"]
                .as_array()
                .unwrap()
                .len()
                > 0
        );
        assert!(updated["audit_events"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| event["event_type"] == "workflow_dispatch_director_review_recorded"));
        assert!(result.snapshot.project_workflows[0]
            .director_reviews
            .iter()
            .any(|review| review.dispatch_id == dispatch_result.dispatch.dispatch_id));

        let _ = fs::remove_dir_all(dir);
    }

    include!("lib_director_review_rejection_tests.rs");

    #[test]
    fn offline_role_orchestration_records_dispatch_handoff_and_review() {
        let dir = std::env::temp_dir().join(format!(
            "offline-role-orchestration-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");
        let draft = fixture_task_draft_request(&project.project_root, "离线角色编排工作项");

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_active_plan_authorization_for_fixture(&path, &project.project_root);
        create_task_draft_at(&path, &draft).expect("work item should exist");
        let value = read_json_file(&path);
        let work_item_id = optional_string_from(&value["work_items"][0], "work_item_id")
            .expect("work item id should exist");
        update_work_item_state_at(
            &path,
            &fixture_work_item_state_update_request(
                &project.project_root,
                &work_item_id,
                "ready_to_dispatch",
            ),
        )
        .expect("work item should be ready");

        let dispatch_result = prepare_offline_role_dispatch_at(
            &path,
            &fixture_offline_role_dispatch_request(&project.project_root, &work_item_id),
        )
        .expect("offline dispatch should write prepared record");

        assert_eq!(
            dispatch_result.dispatch.prompt_kind,
            "offline_role_dispatch"
        );
        assert_eq!(dispatch_result.dispatch.state, "prepared");
        assert_eq!(
            dispatch_result.dispatch.native_thread_id,
            "offline-role:codex-dev"
        );
        assert_eq!(
            dispatch_result
                .dispatch
                .offline_role_dispatch
                .as_ref()
                .expect("offline dispatch payload should roundtrip")
                .task_title,
            "离线角色派发测试"
        );
        assert_eq!(
            dispatch_result.dispatch.warnings,
            vec!["offline_only_no_codex_resume".to_string()]
        );
        let prepared = read_json_file(&path);
        assert!(prepared["audit_events"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| event["event_type"] == "offline_role_dispatch_prepared"));
        assert_eq!(prepared["work_items"][0]["state"], "ready_to_dispatch");

        let handoff_result = record_offline_role_result_handoff_at(
            &path,
            &OfflineRoleResultHandoffRequest {
                project_root: project.project_root.clone(),
                work_item_id: work_item_id.clone(),
                dispatch_id: dispatch_result.dispatch.dispatch_id.clone(),
                target_role_id: "codex-dev".to_string(),
                summary: "离线桩结果：已接收任务，没有执行真实 Codex 会话。".to_string(),
                markdown: "离线桩结果\n\n请总指导回收。".to_string(),
            },
        )
        .expect("offline handoff should complete dispatch");

        assert_eq!(handoff_result.dispatch.state, "completed");
        let handed_off = read_json_file(&path);
        assert_eq!(handed_off["work_items"][0]["state"], "ready_for_review");
        assert!(handed_off["artifacts"]
            .as_array()
            .unwrap()
            .iter()
            .any(|artifact| artifact["artifact_type"] == "handoff"
                && artifact["dispatch_id"] == dispatch_result.dispatch.dispatch_id));
        assert!(handed_off["audit_events"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| event["event_type"] == "offline_role_result_handoff_recorded"));

        let review = record_offline_director_review_at(
            &path,
            &OfflineDirectorReviewRequest {
                project_root: project.project_root.clone(),
                work_item_id: work_item_id.clone(),
                dispatch_id: dispatch_result.dispatch.dispatch_id.clone(),
                decision: "accepted".to_string(),
                summary: "离线总指导回收：接受。".to_string(),
            },
        )
        .expect("offline director review should write");

        assert_eq!(
            review.snapshot.project_workflows[0].task_drafts[0].state,
            "accepted"
        );
        assert_eq!(review.snapshot.counts.reviews, 1);
        let reviewed = read_json_file(&path);
        assert_eq!(reviewed["reviews"][0]["decision"], "accepted");
        assert!(reviewed["reviews"][0]["handoff_refs"]
            .as_array()
            .unwrap()
            .iter()
            .any(|reference| reference.as_str().unwrap_or("").starts_with("artifact:")));
        assert_eq!(reviewed["work_items"][0]["state"], "accepted");
        assert!(reviewed["audit_events"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| event["event_type"] == "offline_director_review_recorded"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn offline_role_dispatch_rejects_missing_ready_work_item() {
        let dir = std::env::temp_dir().join(format!(
            "offline-role-dispatch-rejects-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");
        let draft = fixture_task_draft_request(&project.project_root, "离线拒绝工作项");

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_task_draft_at(&path, &draft).expect("work item should exist");
        let value = read_json_file(&path);
        let work_item_id = optional_string_from(&value["work_items"][0], "work_item_id")
            .expect("work item id should exist");

        let result = prepare_offline_role_dispatch_at(
            &path,
            &fixture_offline_role_dispatch_request(&project.project_root, &work_item_id),
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("不是待派发"));
        let updated = read_json_file(&path);
        assert_eq!(
            updated["workflow_node_dispatches"]
                .as_array()
                .unwrap()
                .len(),
            0
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn offline_role_dispatch_rejects_duplicate_prepared_dispatch() {
        let dir = std::env::temp_dir().join(format!(
            "offline-role-dispatch-duplicate-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");
        let draft = fixture_task_draft_request(&project.project_root, "离线重复派发工作项");

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_active_plan_authorization_for_fixture(&path, &project.project_root);
        create_task_draft_at(&path, &draft).expect("work item should exist");
        let value = read_json_file(&path);
        let work_item_id = optional_string_from(&value["work_items"][0], "work_item_id")
            .expect("work item id should exist");
        update_work_item_state_at(
            &path,
            &fixture_work_item_state_update_request(
                &project.project_root,
                &work_item_id,
                "ready_to_dispatch",
            ),
        )
        .expect("work item should be ready");

        prepare_offline_role_dispatch_at(
            &path,
            &fixture_offline_role_dispatch_request(&project.project_root, &work_item_id),
        )
        .expect("first offline dispatch should write prepared record");
        let duplicate = prepare_offline_role_dispatch_at(
            &path,
            &fixture_offline_role_dispatch_request(&project.project_root, &work_item_id),
        );

        assert!(duplicate.is_err());
        assert!(duplicate.unwrap_err().contains("已有待回传的离线派发"));
        let updated = read_json_file(&path);
        assert_eq!(
            updated["workflow_node_dispatches"]
                .as_array()
                .unwrap()
                .len(),
            1
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    #[ignore = "confirmation task only: writes one real product-line/tasks/*.md file and real workflow state"]
    fn real_task_package_file_generation_confirmation_v1() {
        let state = AppState::new();
        let index = read_index(&state).expect("static index should be readable");
        let value =
            read_workflow_state_value(&state.workflow_state_path).expect("real state should exist");
        let work_item = value
            .get("work_items")
            .and_then(Value::as_array)
            .and_then(|items| {
                items.iter().find(|item| {
                    let work_item_id = optional_string_from(item, "work_item_id");
                    let source_artifact_id = optional_string_from(item, "source_ref");
                    value
                        .get("artifacts")
                        .and_then(Value::as_array)
                        .is_some_and(|artifacts| {
                            artifacts.iter().any(|artifact| {
                                optional_string_from(artifact, "artifact_type").as_deref()
                                    == Some("task_package")
                                    && optional_string_from(artifact, "path").is_none()
                                    && (work_item_id.as_deref().is_some_and(|id| {
                                        optional_string_from(artifact, "source_ref").as_deref()
                                            == Some(id)
                                    }) || source_artifact_id.as_deref().is_some_and(|id| {
                                        optional_string_from(artifact, "artifact_id").as_deref()
                                            == Some(id)
                                    }))
                            })
                        })
                })
            })
            .expect("real state should contain one ungenerated task package draft");
        let work_item_id =
            optional_string_from(work_item, "work_item_id").expect("work item id should exist");
        let workflow_id =
            optional_string_from(work_item, "workflow_id").expect("workflow id should exist");
        let project_root = value
            .get("workflows")
            .and_then(Value::as_array)
            .and_then(|workflows| {
                workflows
                    .iter()
                    .find(|workflow| {
                        optional_string_from(workflow, "workflow_id").as_deref()
                            == Some(workflow_id.as_str())
                    })
                    .and_then(|workflow| optional_string_from(workflow, "project_id"))
            })
            .and_then(|project_id_value| {
                value
                    .get("projects")
                    .and_then(Value::as_array)
                    .and_then(|projects| {
                        projects
                            .iter()
                            .find(|project| {
                                optional_string_from(project, "project_id").as_deref()
                                    == Some(project_id_value.as_str())
                            })
                            .and_then(|project| optional_string_from(project, "root_path"))
                    })
            })
            .expect("project root should be recoverable from real state");
        let request = TaskPackageFileGenerationRequest {
            project_root,
            work_item_id,
        };

        let result = generate_task_package_file_for_index_project_at(
            &state.workflow_state_path,
            &index,
            &request,
            &default_task_package_output_dir(),
        )
        .expect("real generation should write one task file");

        println!("generated_file_path={}", result.file_path);
        println!("backup_path={}", result.backup_path);
        println!("audit_event_id={}", result.audit_event_id);
    }

    fn fixture_project(project_root: &str) -> ProjectRecord {
        ProjectRecord {
            project_root: project_root.to_string(),
            name: path_name(project_root),
            active_hint: true,
            thread_count: 0,
            active_thread_count: 0,
            archived_thread_count: 0,
            latest_updated_at_ms: None,
            authority_files: vec![],
            handoff_files: vec![],
            evidence_files: vec![],
            harness_candidates: vec![],
            harness_resources: vec![],
            context_warnings: vec![],
            warnings: vec![],
        }
    }

    fn fixture_task_draft_request(project_root: &str, title: &str) -> TaskDraftRequest {
        TaskDraftRequest {
            project_root: project_root.to_string(),
            title: title.to_string(),
            objective: "写入 work_items 和 artifacts".to_string(),
            assigned_role: Some("codex-dev".to_string()),
        }
    }

    fn fixture_task_preview_request(
        project_root: &str,
        work_item_id: &str,
    ) -> TaskPackagePreviewRequest {
        TaskPackagePreviewRequest {
            project_root: project_root.to_string(),
            work_item_id: work_item_id.to_string(),
        }
    }

    fn fixture_fields_update_request(
        project_root: &str,
        work_item_id: &str,
    ) -> TaskPackageFieldsUpdateRequest {
        TaskPackageFieldsUpdateRequest {
            project_root: project_root.to_string(),
            work_item_id: work_item_id.to_string(),
            fields: TaskPackageFieldsInput {
                task_name: "字段编辑任务".to_string(),
                assigned_line: "桌面应用线".to_string(),
                background: vec!["来自结构化字段。".to_string()],
                goals: vec!["完成字段编辑。".to_string()],
                allowed_read: vec!["/tmp/indexed-project".to_string()],
                allowed_write: vec!["工作台状态文件".to_string()],
                forbidden_actions: vec!["不生成真实任务文件。".to_string()],
                acceptance_criteria: vec!["预览使用新字段。".to_string()],
                required_return: vec!["做了什么".to_string()],
                review_focus: vec!["确认结构化字段。".to_string()],
            },
        }
    }

    fn empty_fields_update_request(
        project_root: &str,
        work_item_id: &str,
    ) -> TaskPackageFieldsUpdateRequest {
        TaskPackageFieldsUpdateRequest {
            project_root: project_root.to_string(),
            work_item_id: work_item_id.to_string(),
            fields: TaskPackageFieldsInput {
                task_name: "".to_string(),
                assigned_line: "".to_string(),
                background: vec![],
                goals: vec![],
                allowed_read: vec![],
                allowed_write: vec![],
                forbidden_actions: vec![],
                acceptance_criteria: vec![],
                required_return: vec![],
                review_focus: vec![],
            },
        }
    }

    fn ready_fields_update_request(
        project_root: &str,
        work_item_id: &str,
    ) -> TaskPackageFieldsUpdateRequest {
        ready_fields_update_request_with_forbidden(
            project_root,
            work_item_id,
            vec![
                "不写 `/Users/yoyi/.codex`。".to_string(),
                "不改真实 Codex 状态库。".to_string(),
                "不派发真实 Codex 会话。".to_string(),
                "不运行 harness。".to_string(),
            ],
        )
    }

    fn ready_fields_update_request_with_forbidden(
        project_root: &str,
        work_item_id: &str,
        forbidden_actions: Vec<String>,
    ) -> TaskPackageFieldsUpdateRequest {
        TaskPackageFieldsUpdateRequest {
            project_root: project_root.to_string(),
            work_item_id: work_item_id.to_string(),
            fields: TaskPackageFieldsInput {
                task_name: "派发准备检查任务".to_string(),
                assigned_line: "桌面应用线".to_string(),
                background: vec!["基于已生成任务包检查派发准备状态。".to_string()],
                goals: vec!["实现派发准备检查并展示不合格原因。".to_string()],
                allowed_read: vec![project_root.to_string()],
            allowed_write: vec![
                format!("{project_root}/product-line/prototypes/productized-desktop-shell/src/"),
                format!("{project_root}/product-line/prototypes/productized-desktop-shell/src-tauri/src/"),
            ],
                forbidden_actions,
                acceptance_criteria: vec![
                    "污染草稿不能标记为 ready。".to_string(),
                    "字段修正后 readiness 为 ready。".to_string(),
                ],
                required_return: vec![
                    "做了什么".to_string(),
                    "改了哪些文件".to_string(),
                    "验证命令和结果".to_string(),
                    "风险和下一步建议".to_string(),
                ],
                review_focus: vec!["确认没有派发真实 Codex。".to_string()],
            },
        }
    }

    fn fixture_task_file_generation_request(
        project_root: &str,
        work_item_id: &str,
    ) -> TaskPackageFileGenerationRequest {
        TaskPackageFileGenerationRequest {
            project_root: project_root.to_string(),
            work_item_id: work_item_id.to_string(),
        }
    }

    fn fixture_dispatch_readiness_request(
        project_root: &str,
        work_item_id: &str,
    ) -> TaskPackageDispatchReadinessRequest {
        TaskPackageDispatchReadinessRequest {
            project_root: project_root.to_string(),
            work_item_id: work_item_id.to_string(),
        }
    }

    fn setup_task_memory_injection_fixture(
        name: &str,
    ) -> (PathBuf, PathBuf, PathBuf, ProjectRecord, String) {
        let dir = test_temp_dir(name);
        let path = dir.join("workflow-state.v0.json");
        let tasks_dir = dir.join("tasks");
        let project_root = format!("/tmp/{name}-project");
        let project = fixture_project(&project_root);
        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_active_plan_authorization_for_fixture(&path, &project.project_root);
        create_task_draft_at(
            &path,
            &fixture_task_draft_request(&project.project_root, "派发准备检查任务"),
        )
        .expect("task draft should exist");
        let value = read_json_file(&path);
        let work_item_id = optional_string_from(&value["work_items"][0], "work_item_id")
            .expect("work item id should exist");
        update_task_package_draft_fields_at(
            &path,
            &ready_fields_update_request(&project.project_root, &work_item_id),
        )
        .expect("ready fields should save");
        mark_task_package_fixture_ready(&path, "codex-test-model");

        (dir, path, tasks_dir, project, work_item_id)
    }

    fn mark_task_package_fixture_ready(path: &Path, model_id: &str) {
        let mut value = read_json_file(path);
        let artifacts = value["artifacts"]
            .as_array_mut()
            .expect("artifacts should be array");
        let artifact = artifacts
            .first_mut()
            .expect("task package artifact should exist");
        artifact["model_id"] = Value::String(model_id.to_string());
        artifact["callable_tool_capabilities"] = json!(["read_file", "apply_patch"]);
        artifact["available_knowledge_refs"] = Value::Array(vec![]);
        artifact["available_memory_refs"] = Value::Array(vec![]);
        artifact["harness_requirements"] = Value::Array(vec![]);
        artifact["requires_tools"] = Value::Bool(false);
        artifact["requires_harness"] = Value::Bool(false);
        write_validated_workflow_state(path, &value).expect("fixture ready fields should write");
    }

    fn fixture_dispatch_prepare_request(
        project_root: &str,
        node_id: &str,
        work_item_id: &str,
    ) -> WorkflowNodeDispatchPrepareRequest {
        WorkflowNodeDispatchPrepareRequest {
            project_root: project_root.to_string(),
            node_id: node_id.to_string(),
            work_item_id: work_item_id.to_string(),
            prompt_kind: "safe_probe".to_string(),
            user_reviewed_instruction: None,
        }
    }

    fn fixture_dispatch_execute_request(
        project_root: &str,
        node_id: &str,
        work_item_id: &str,
    ) -> WorkflowNodeDispatchExecuteRequest {
        WorkflowNodeDispatchExecuteRequest {
            project_root: project_root.to_string(),
            node_id: node_id.to_string(),
            work_item_id: work_item_id.to_string(),
            prompt_kind: "safe_probe".to_string(),
            user_reviewed_instruction: None,
        }
    }

    fn fixture_offline_role_dispatch_request(
        project_root: &str,
        work_item_id: &str,
    ) -> OfflineRoleDispatchRequest {
        OfflineRoleDispatchRequest {
            project_root: project_root.to_string(),
            work_item_id: work_item_id.to_string(),
            target_role_id: "codex-dev".to_string(),
            target_role_label: "开发线".to_string(),
            task_title: "离线角色派发测试".to_string(),
            objective: "记录离线派发，不执行真实 Codex。".to_string(),
            execution_cwd: project_root.to_string(),
            allowed_reads: vec![project_root.to_string()],
            allowed_writes: vec![format!("{project_root}/README.md")],
            forbidden_actions: vec![
                "不读取 auth.json、.env、密钥、token 或授权文件".to_string(),
                "不执行 codex exec resume".to_string(),
            ],
            acceptance_criteria: vec!["离线派发记录进入 prepared".to_string()],
            timeout_seconds: 600,
            required_return: vec!["薄弱点".to_string(), "验证结果".to_string()],
            raw_block:
                "派发给：开发线\n任务名：离线角色派发测试\n目标：记录离线派发，不执行真实 Codex。"
                    .to_string(),
        }
    }

    fn fixture_user_reviewed_instruction() -> UserReviewedInstructionInput {
        UserReviewedInstructionInput {
            instruction_id: "instruction:offline:user-reviewed".to_string(),
            summary: "用户审核业务指令夹具".to_string(),
            objective: "验证后端把审核字段传给 codex exec resume 参数。".to_string(),
            execution_cwd: "/Users/yoyi".to_string(),
            sandbox_mode: "workspace-write".to_string(),
            allowed_write_roots: vec!["/Users/yoyi/codex-workflow-mario-test".to_string()],
            allowed_reads: vec!["/Users/yoyi/codex-workflow-mario-test".to_string()],
            allowed_writes: vec!["/Users/yoyi/codex-workflow-mario-test/index.html".to_string()],
            forbidden_actions: vec![
                "不读取 auth.json、.env、密钥、token 或授权文件。".to_string(),
                "不读取完整 transcript。".to_string(),
                "不运行 harness。".to_string(),
            ],
            timeout_seconds: 600,
            max_retries: 0,
            required_return: vec!["薄弱点".to_string(), "验证命令和结果".to_string()],
            prompt_preview: None,
        }
    }

    fn fixture_director_review_request(
        project_root: &str,
        work_item_id: &str,
        dispatch_id: &str,
        decision: &str,
    ) -> WorkflowDispatchDirectorReviewRequest {
        WorkflowDispatchDirectorReviewRequest {
            project_root: project_root.to_string(),
            work_item_id: work_item_id.to_string(),
            dispatch_id: dispatch_id.to_string(),
            decision: decision.to_string(),
            summary: "总指导回收：离线测试。".to_string(),
        }
    }

    fn append_fixture_dispatch(
        path: &Path,
        project_root: &str,
        work_item_id: &str,
        state: &str,
        thread_id: &str,
    ) -> String {
        let mut value = read_json_file(path);
        let workflow_id = default_workflow_id(project_root);
        let dispatch_id = format!(
            "dispatch:{}:{}:{}",
            stable_id(&workflow_id),
            stable_id(work_item_id),
            state
        );
        value["workflow_node_dispatches"]
            .as_array_mut()
            .expect("dispatches should be array")
            .push(json!({
              "dispatch_id": dispatch_id,
              "project_id": project_id(project_root),
              "workflow_id": workflow_id,
              "node_id": format!("{}:node:director", default_workflow_id(project_root)),
              "work_item_id": work_item_id,
              "binding_id": "binding:fixture",
              "native_thread_id": thread_id,
              "prompt_preview": safe_probe_prompt(),
              "prompt_kind": "safe_probe",
              "state": state,
              "started_at_ms": 1_764_000_000_000_i64,
              "ended_at_ms": if state == "completed" { json!(1_764_000_001_000_i64) } else { Value::Null },
              "exit_code": if state == "completed" { json!(0) } else { Value::Null },
              "last_message_path": if state == "completed" { json!("/tmp/fixture-last-message.txt") } else { Value::Null },
              "last_message_summary": if state == "completed" { json!(safe_probe_target()) } else { Value::Null },
              "transcript_event_count": if state == "completed" { json!(32) } else { Value::Null },
              "transcript_target_hits": if state == "completed" { json!(4) } else { Value::Null },
              "warnings": []
            }));
        write_validated_workflow_state(path, &value).expect("fixture dispatch should write");
        dispatch_id
    }

    fn append_fixture_permission_request(
        path: &Path,
        project_root: &str,
        work_item_id: &str,
        status: &str,
    ) {
        let mut value = read_json_file(path);
        let workflow_id = default_workflow_id(project_root);
        value["permission_requests"] = json!([{
            "request_id": "permission:fixture:001",
            "project_id": project_id(project_root),
            "workflow_id": workflow_id,
            "work_item_id": work_item_id,
            "dispatch_id": Value::Null,
            "permission_kind": "write_workflow_state",
            "reason": "测试控制核心权限确认。",
            "status": status,
            "requested_at": "2026-06-01T00:00:00Z",
            "decided_at": Value::Null,
            "decision": Value::Null,
            "warnings": []
        }]);
        write_validated_workflow_state(path, &value)
            .expect("fixture permission request should write");
    }

    fn fixture_node_state(value: &Value, node_id: &str) -> Option<String> {
        value
            .get("nodes")
            .and_then(Value::as_array)
            .and_then(|nodes| {
                nodes
                    .iter()
                    .find(|node| optional_string_from(node, "node_id").as_deref() == Some(node_id))
            })
            .and_then(|node| optional_string_from(node, "state"))
    }

    fn fixture_dispatch_index(project_root: &str, thread_id: &str) -> Value {
        json!({
          "projects": [{ "project_root": project_root }],
          "threads": [
            {
              "thread_id": thread_id,
              "project_root": project_root,
              "title": format!("Session {thread_id}"),
              "rollout_exists": true,
              "rollout_path": format!("/tmp/{thread_id}.jsonl")
            }
          ]
        })
    }

    fn fixture_multi_thread_index(project_root: &str, thread_ids: &[&str]) -> Value {
        json!({
          "projects": [{ "project_root": project_root }],
          "threads": thread_ids
              .iter()
              .map(|thread_id| {
                  json!({
                    "thread_id": thread_id,
                    "project_root": project_root,
                    "title": format!("Session {thread_id}"),
                    "rollout_exists": true,
                    "rollout_path": format!("/tmp/{thread_id}.jsonl")
                  })
              })
              .collect::<Vec<_>>()
        })
    }

    fn fixture_work_item_state_update_request(
        project_root: &str,
        work_item_id: &str,
        next_state: &str,
    ) -> WorkItemStateUpdateRequest {
        WorkItemStateUpdateRequest {
            project_root: project_root.to_string(),
            work_item_id: work_item_id.to_string(),
            next_state: next_state.to_string(),
        }
    }

    fn fixture_node_session_bind_request(
        project_root: &str,
        node_id: &str,
        work_item_id: Option<&str>,
        thread_id: &str,
    ) -> WorkflowNodeSessionBindRequest {
        WorkflowNodeSessionBindRequest {
            project_root: project_root.to_string(),
            node_id: node_id.to_string(),
            work_item_id: work_item_id.map(str::to_string),
            thread_id: thread_id.to_string(),
        }
    }

    fn fixture_node_session_unbind_request(
        project_root: &str,
        binding_id: &str,
    ) -> WorkflowNodeSessionUnbindRequest {
        WorkflowNodeSessionUnbindRequest {
            project_root: project_root.to_string(),
            binding_id: binding_id.to_string(),
        }
    }

    struct StubCodexResumeRunner {
        stats: CodexDispatchReadbackStats,
    }

    struct NoReadbackStatsCodexResumeRunner;

    struct WorkflowMachineStubRunner {
        stats: CodexDispatchReadbackStats,
    }

    struct FailingCodexResumeRunner {
        exit_code: i32,
        timed_out: bool,
    }

    impl CodexResumeRunner for StubCodexResumeRunner {
        fn resume_with_options(
            &self,
            _thread_id: &str,
            prompt: &str,
            last_message_path: &Path,
            options: &CodexResumeRequestOptions,
        ) -> Result<(CodexResumeRunResult, WorkflowNodeDispatchExecutionOptions), String> {
            let parent = last_message_path
                .parent()
                .ok_or_else(|| "last message path should have parent".to_string())?;
            fs::create_dir_all(parent)
                .map_err(|error| format!("fixture output dir create failed: {error}"))?;
            if options.prompt_kind == "safe_probe" {
                if prompt != safe_probe_prompt() {
                    return Err("stub runner expected safe probe prompt".to_string());
                }
                if options.execution_cwd.is_some()
                    || options.sandbox_mode.is_some()
                    || !options.allowed_write_roots.is_empty()
                {
                    return Err("safe probe should not pass business codex options".to_string());
                }
                fs::write(last_message_path, safe_probe_target())
                    .map_err(|error| format!("fixture last message write failed: {error}"))?;
            } else if options.prompt_kind == "user_reviewed_instruction" {
                if !prompt.contains("用户审核过的真实业务指令")
                    || !prompt.contains("允许写入根目录")
                {
                    return Err("stub runner expected rendered user reviewed prompt".to_string());
                }
                if options.execution_cwd.as_deref() != Some(Path::new("/Users/yoyi")) {
                    return Err("user reviewed dispatch should pass execution cwd".to_string());
                }
                if options.sandbox_mode.as_deref() != Some("workspace-write") {
                    return Err("user reviewed dispatch should pass sandbox mode".to_string());
                }
                if options.allowed_write_roots
                    != vec![PathBuf::from("/Users/yoyi/codex-workflow-mario-test")]
                {
                    return Err(
                        "user reviewed dispatch should pass allowed write roots".to_string()
                    );
                }
                if options.timeout_seconds != Some(600) {
                    return Err("user reviewed dispatch should pass timeout seconds".to_string());
                }
                fs::write(last_message_path, "USER_REVIEWED_STUB_OK")
                    .map_err(|error| format!("fixture last message write failed: {error}"))?;
            } else {
                return Err(format!(
                    "stub runner got unknown prompt kind {}",
                    options.prompt_kind
                ));
            }
            Ok((
                CodexResumeRunResult {
                    exit_code: 0,
                    timed_out: false,
                    stderr_summary: None,
                },
                WorkflowNodeDispatchExecutionOptions {
                    readback_stats: Some(self.stats.clone()),
                },
            ))
        }
    }

    impl CodexResumeRunner for NoReadbackStatsCodexResumeRunner {
        fn resume_with_options(
            &self,
            _thread_id: &str,
            prompt: &str,
            last_message_path: &Path,
            options: &CodexResumeRequestOptions,
        ) -> Result<(CodexResumeRunResult, WorkflowNodeDispatchExecutionOptions), String> {
            if options.prompt_kind != "safe_probe" || prompt != safe_probe_prompt() {
                return Err("native readback runner expected safe probe".to_string());
            }
            let parent = last_message_path
                .parent()
                .ok_or_else(|| "last message path should have parent".to_string())?;
            fs::create_dir_all(parent)
                .map_err(|error| format!("fixture output dir create failed: {error}"))?;
            fs::write(last_message_path, safe_probe_target())
                .map_err(|error| format!("fixture last message write failed: {error}"))?;
            Ok((
                CodexResumeRunResult {
                    exit_code: 0,
                    timed_out: false,
                    stderr_summary: None,
                },
                WorkflowNodeDispatchExecutionOptions {
                    readback_stats: None,
                },
            ))
        }
    }

    impl CodexResumeRunner for WorkflowMachineStubRunner {
        fn resume_with_options(
            &self,
            thread_id: &str,
            prompt: &str,
            last_message_path: &Path,
            options: &CodexResumeRequestOptions,
        ) -> Result<(CodexResumeRunResult, WorkflowNodeDispatchExecutionOptions), String> {
            if options.prompt_kind != "user_reviewed_instruction" {
                return Err(
                    "workflow machine stub only accepts user reviewed instruction".to_string(),
                );
            }
            if !prompt.contains("运行 ID") || !prompt.contains("角色：") {
                return Err("workflow machine prompt missing control fields".to_string());
            }
            let expected_write = thread_id == "thread-dev";
            if expected_write {
                if options.sandbox_mode.as_deref() != Some("workspace-write") {
                    return Err("developer step should be workspace-write".to_string());
                }
                if options.allowed_write_roots.is_empty() {
                    return Err("developer step should have write root".to_string());
                }
            } else if options.sandbox_mode.as_deref() != Some("read-only") {
                return Err("non-developer step should be read-only".to_string());
            }
            if options.timeout_seconds != Some(600) {
                return Err("workflow machine step should pass timeout".to_string());
            }
            let parent = last_message_path
                .parent()
                .ok_or_else(|| "last message path should have parent".to_string())?;
            fs::create_dir_all(parent)
                .map_err(|error| format!("fixture output dir create failed: {error}"))?;
            let message = if thread_id == "thread-director" && prompt.contains("验证线") {
                "总指导结论：最终目标完成\nWORKFLOW_MACHINE_FINAL_ACCEPTED"
            } else {
                "本步完成\nWORKFLOW_MACHINE_STEP_STATUS: completed"
            };
            fs::write(last_message_path, message)
                .map_err(|error| format!("fixture last message write failed: {error}"))?;
            Ok((
                CodexResumeRunResult {
                    exit_code: 0,
                    timed_out: false,
                    stderr_summary: None,
                },
                WorkflowNodeDispatchExecutionOptions {
                    readback_stats: Some(self.stats.clone()),
                },
            ))
        }
    }

    impl CodexResumeRunner for FailingCodexResumeRunner {
        fn resume_with_options(
            &self,
            _thread_id: &str,
            _prompt: &str,
            last_message_path: &Path,
            _options: &CodexResumeRequestOptions,
        ) -> Result<(CodexResumeRunResult, WorkflowNodeDispatchExecutionOptions), String> {
            if let Some(parent) = last_message_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|error| format!("fixture output dir create failed: {error}"))?;
            }
            Ok((
                CodexResumeRunResult {
                    exit_code: self.exit_code,
                    timed_out: self.timed_out,
                    stderr_summary: None,
                },
                WorkflowNodeDispatchExecutionOptions {
                    readback_stats: Some(CodexDispatchReadbackStats {
                        transcript_event_count: 0,
                        transcript_target_hits: 0,
                    }),
                },
            ))
        }
    }

    fn fixture_session(thread_id: &str, project_root: &str, rollout_exists: bool) -> SessionRecord {
        SessionRecord {
            thread_id: thread_id.to_string(),
            title: format!("Session {thread_id}"),
            project_root: Some(project_root.to_string()),
            updated_at_ms: Some(1_764_000_000_000),
            archived: false,
            rollout_exists,
            rollout_path: Some(format!("/tmp/{thread_id}.jsonl")),
            model: Some("offline-model".to_string()),
            reasoning_effort: Some("offline".to_string()),
            thread_source: Some("offline-fixture".to_string()),
            warnings: vec![],
        }
    }

    fn fixture_dispatch_correction_request(
        project_root: &str,
        work_item_id: &str,
    ) -> TaskPackageDispatchFieldsCorrectionRequest {
        let update = ready_fields_update_request(project_root, work_item_id);
        TaskPackageDispatchFieldsCorrectionRequest {
            project_root: update.project_root,
            work_item_id: update.work_item_id,
            fields: update.fields,
        }
    }

    fn read_json_file(path: &Path) -> Value {
        let text = fs::read_to_string(path).expect("json fixture should be readable");
        serde_json::from_str(&text).expect("fixture should parse")
    }
}
