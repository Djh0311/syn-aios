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
mod plan_authorization_store;
mod project_consultation_proposal_store;
mod project_workflow_automation;
mod real_execution_command;
mod runtime_log_store;
mod runtime_session_attention;
mod session_continuation_store;
mod task_memory_injection;
mod task_memory_packet_builder;
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

    #[test]
    fn path_whitelist_accepts_only_index_projects_and_rollouts() {
        let index = json!({
          "projects": [
            { "project_root": "/Users/yoyi/workspace" },
            { "project_root": null }
          ],
          "threads": [
            { "rollout_path": "/Users/yoyi/.codex/sessions/sample.jsonl" },
            { "rollout_path": 12 }
          ]
        });

        let allowed = allowed_paths(&index);

        assert!(allowed.projects.contains("/Users/yoyi/workspace"));
        assert!(allowed
            .rollouts
            .contains("/Users/yoyi/.codex/sessions/sample.jsonl"));
        assert!(allowed.can_copy("/Users/yoyi/workspace"));
        assert!(!allowed.can_copy("/Users/yoyi/.codex/auth.json"));
    }

    #[test]
    fn snapshot_keeps_metadata_without_session_body() {
        let dir = test_temp_dir("snapshot-metadata");
        fs::create_dir_all(&dir).expect("create temp dir");
        let state = AppState {
            index_path: dir.join("codex-index.json"),
            tasks_path: dir.join("tasks.md"),
            workflow_state_path: dir.join("workflow-state.v0.json"),
        };
        let index = json!({
          "generated_at": "2026-05-27T10:23:52Z",
          "projects": [
            {
              "project_root": "/Users/yoyi/workspace",
              "thread_count": 2,
              "authority_files": [{ "kind": "readme", "path": "/Users/yoyi/workspace/README.md" }]
            }
          ],
          "threads": [
            {
              "thread_id": "abc",
              "title": "truncated title",
              "rollout_path": "/Users/yoyi/.codex/sessions/sample.jsonl",
              "rollout_exists": true
            }
          ],
          "skills": [{ "skill_id": "one", "title": "One", "path": "/skills/one", "source_type": "user" }],
          "plugins": [{ "plugin_name": "browser", "plugin_version": "1", "skill_paths": ["/a"] }],
          "warnings": []
        });

        let snapshot = build_snapshot_with_session_source(
            &state,
            &index,
            "## 待派发\n\n- `task.md`：说明正文\n",
            SessionSourceMode::IndexOnly,
        );

        assert_eq!(snapshot.summary.project_count, 1);
        assert_eq!(snapshot.summary.session_count, 1);
        assert_eq!(snapshot.projects[0].authority_files.len(), 1);
        assert_eq!(snapshot.tasks[0].title, "task.md");
        assert_eq!(snapshot.diagnostics.allowed_rollout_path_count, 1);
        assert_eq!(snapshot.diagnostic_summary.status, "degraded_readonly");
        assert!(snapshot
            .diagnostic_summary
            .boundary_notes
            .iter()
            .any(|note| note.contains("只读诊断")));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn g2_diagnostic_summary_reports_degraded_store_without_repair() {
        let dir = test_temp_dir("g2-diagnostic-summary");
        fs::create_dir_all(&dir).expect("create temp dir");
        let workflow_state_path = dir.join("workflow-state.v0.json");
        let workflow_state = json!({
          "schema_version": "workflow_state_v0",
          "workflow_version": 1,
          "workspace_id": "workspace:g2",
          "updated_at": "2026-06-07T00:00:00Z",
          "projects": [{ "project_id": "project:g2", "root_path": "/tmp/g2-project" }],
          "agent_adapters": [{ "adapter_id": "codex-local", "agent_type": "codex" }],
          "workflows": [{ "workflow_id": "workflow:g2", "project_id": "project:g2", "title": "G2", "state": "running" }],
          "nodes": [{ "node_id": "node:g2", "workflow_id": "workflow:g2", "node_type": "dev_line", "title": "G2 node", "state": "running" }],
          "edges": [],
          "work_items": [],
          "artifacts": [],
          "reviews": [],
          "audit_events": [],
          "capabilities": [],
          "harness_resources": [],
          "workflow_node_session_bindings": [],
          "workflow_node_dispatches": [],
          "workflow_execution_controls": [],
          "permission_requests": [],
          "execution_attempts": []
        });
        fs::write(
            &workflow_state_path,
            serde_json::to_string_pretty(&workflow_state).expect("serialize workflow state"),
        )
        .expect("write workflow state");
        fs::write(dir.join("formal-memories.v1.json"), "{broken json")
            .expect("write broken formal memory sidecar");
        let state = AppState {
            index_path: dir.join("codex-index.json"),
            tasks_path: dir.join("tasks.md"),
            workflow_state_path,
        };
        fs::write(&state.tasks_path, "- `g2.md`：G2\n").expect("write tasks");
        let index = json!({
          "generated_at": "2026-06-07T00:00:00Z",
          "projects": [{ "project_root": "/tmp/g2-project" }],
          "threads": []
        });

        let snapshot =
            build_snapshot_with_session_source(&state, &index, "", SessionSourceMode::IndexOnly);

        assert_eq!(snapshot.diagnostic_summary.status, "degraded_readonly");
        assert!(snapshot.diagnostic_summary.blocked_count > 0);
        assert!(snapshot
            .diagnostic_summary
            .store_integrity
            .iter()
            .any(|finding| finding.store_id == "formal_memory"
                && finding.status == "degraded"
                && finding.error.as_deref().unwrap_or("").contains("JSON")));
        assert!(snapshot
            .diagnostic_summary
            .degraded_states
            .iter()
            .any(|state| state.kind == "adapter_unavailable" && state.blocks_real_execution));
        assert!(snapshot
            .diagnostic_summary
            .boundary_notes
            .iter()
            .any(|note| note.contains("readback_unavailable")));
        assert_eq!(
            fs::read_to_string(dir.join("formal-memories.v1.json")).unwrap(),
            "{broken json"
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn workbench_snapshot_includes_backend_agent_adapter_descriptor() {
        let dir = test_temp_dir("adapter-backend-read-model");
        fs::create_dir_all(&dir).expect("create temp dir");
        let workflow_state_path = dir.join("workflow-state.v0.json");
        let workflow_state = json!({
          "schema_version": "workflow_state_v0",
          "workflow_version": 1,
          "workspace_id": "workspace:test",
          "updated_at": "2026-06-03T00:00:00Z",
          "projects": [{ "project_id": "project:test", "root_path": "/tmp/adapter-project" }],
          "agent_adapters": [{ "adapter_id": "codex-local", "agent_type": "codex" }],
          "workflows": [{ "workflow_id": "workflow:test", "project_id": "project:test", "title": "Adapter test", "state": "running" }],
          "nodes": [{ "node_id": "node:codex-dev", "workflow_id": "workflow:test", "node_type": "dev_line", "title": "开发线", "state": "running" }],
          "edges": [],
          "work_items": [{ "work_item_id": "work:test", "workflow_id": "workflow:test", "project_id": "project:test", "title": "Adapter task", "state": "ready_to_dispatch", "assigned_role_id": "codex-dev" }],
          "artifacts": [],
          "reviews": [],
          "audit_events": [],
          "capabilities": [],
          "harness_resources": [],
          "workflow_node_session_bindings": [{
            "binding_id": "binding:codex-dev",
            "project_id": "project:test",
            "workflow_id": "workflow:test",
            "node_id": "node:codex-dev",
            "work_item_id": "work:test",
            "agent_type": "codex",
            "adapter_id": "codex-local",
            "native_thread_id": "thread:adapter",
            "native_rollout_path": "/tmp/adapter-thread.jsonl",
            "session_title": "Adapter thread",
            "rollout_exists": true,
            "lifecycle": "active",
            "created_at_ms": 1,
            "updated_at_ms": 2
          }],
          "workflow_node_dispatches": [{
            "dispatch_id": "dispatch:safe-probe",
            "project_id": "project:test",
            "workflow_id": "workflow:test",
            "node_id": "node:codex-dev",
            "work_item_id": "work:test",
            "binding_id": "binding:codex-dev",
            "native_thread_id": "thread:adapter",
            "prompt_kind": "safe_probe",
            "state": "completed"
          }],
          "workflow_execution_controls": [{
            "control_id": "control:reviewed",
            "project_id": "project:test",
            "workflow_id": "workflow:test",
            "work_item_id": "work:test",
            "user_reviewed_instruction": {
              "instruction_id": "instruction:reviewed",
              "summary": "reviewed",
              "objective": "只测试读模型",
              "execution_cwd": "/tmp/adapter-project",
              "sandbox_mode": "workspace-write",
              "approval_state": "reviewed"
            }
          }],
          "permission_requests": [{
            "request_id": "permission:one",
            "project_id": "project:test",
            "workflow_id": "workflow:test",
            "work_item_id": "work:test",
            "permission_kind": "write_workflow_state",
            "reason": "test",
            "status": "pending",
            "requested_at": "2026-06-03T00:00:00Z"
          }],
          "execution_attempts": []
        });
        fs::write(
            &workflow_state_path,
            serde_json::to_string_pretty(&workflow_state).expect("serialize workflow state"),
        )
        .expect("write workflow state");
        let state = AppState {
            index_path: dir.join("codex-index.json"),
            tasks_path: dir.join("tasks.md"),
            workflow_state_path,
        };
        let index = json!({
          "generated_at": "2026-06-03T00:00:00Z",
          "projects": [{
            "project_root": "/tmp/adapter-project",
            "harness_resources": [{
              "root_path": "/tmp/adapter-project/harness",
              "adapter_id": "codex-local"
            }]
          }],
          "threads": [{
            "thread_id": "thread:adapter",
            "title": "Adapter thread",
            "project_root": "/tmp/adapter-project",
            "thread_source": "codex",
            "rollout_path": "/tmp/adapter-thread.jsonl",
            "rollout_exists": true
          }]
        });

        let snapshot =
            build_snapshot_with_session_source(&state, &index, "", SessionSourceMode::IndexOnly);

        assert_eq!(snapshot.agent_adapters.len(), 5);
        let adapter = snapshot
            .agent_adapters
            .iter()
            .find(|descriptor| descriptor.adapter_id == "codex-local")
            .expect("codex-local descriptor");
        assert_eq!(adapter.adapter_id, "codex-local");
        assert_eq!(adapter.source_kind, "backend_read_model");
        assert_eq!(adapter.status, "available");
        assert_eq!(adapter.execution_status, "available_with_user_confirmation");
        assert_eq!(adapter.credential_status, "not_read");
        assert_eq!(adapter.model_access_status, "local_read_model_only");
        assert!(adapter
            .warnings
            .contains(&"adapter_descriptor_is_backend_read_model_only".to_string()));
        assert!(adapter
            .hidden_unimplemented_adapters
            .contains(&"claude-code".to_string()));
        assert!(adapter
            .hidden_unimplemented_adapters
            .contains(&"openclaw".to_string()));
        assert!(adapter
            .hidden_unimplemented_adapters
            .contains(&"opencode-like".to_string()));
        assert!(adapter.capabilities.iter().any(|capability| capability.kind
            == "workflow_node_binding"
            && capability.status == "requires_confirmation"
            && capability
                .evidence_refs
                .contains(&"binding:codex-dev".to_string())));
        assert!(adapter
            .capabilities
            .iter()
            .filter(|capability| [
                "safe_probe_dispatch",
                "user_reviewed_dispatch",
                "workflow_machine_run"
            ]
            .contains(&capability.kind.as_str()))
            .all(|capability| capability.status == "requires_confirmation"));
        let planned = snapshot
            .agent_adapters
            .iter()
            .find(|descriptor| descriptor.adapter_id == "claude-code")
            .expect("planned claude-code descriptor");
        assert_eq!(planned.status, "planned");
        assert_eq!(planned.execution_status, "not_implemented");
        assert_eq!(planned.credential_status, "not_configured");
        assert_eq!(planned.model_access_status, "not_verified");
        assert_eq!(planned.implemented_action_kinds.len(), 0);
        assert_eq!(planned.capabilities.len(), 0);
        assert!(planned.requires_user_setup);
        assert!(planned.unavailable_reason.is_some());
        assert!(planned
            .warnings
            .contains(&"planned_adapter_not_connected".to_string()));
        assert!(planned
            .warnings
            .contains(&"no_execution_button".to_string()));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn backend_agent_adapter_descriptor_is_stable_without_codex_signals() {
        let dir = test_temp_dir("adapter-backend-empty");
        fs::create_dir_all(&dir).expect("create temp dir");
        let state = AppState {
            index_path: dir.join("codex-index.json"),
            tasks_path: dir.join("tasks.md"),
            workflow_state_path: dir.join("missing-workflow-state.v0.json"),
        };
        let snapshot = build_snapshot_with_session_source(
            &state,
            &json!({ "projects": [], "threads": [] }),
            "",
            SessionSourceMode::IndexOnly,
        );

        assert_eq!(snapshot.agent_adapters.len(), 5);
        let adapter = snapshot
            .agent_adapters
            .iter()
            .find(|descriptor| descriptor.adapter_id == "codex-local")
            .expect("codex-local descriptor");
        assert_eq!(adapter.adapter_id, "codex-local");
        assert_eq!(adapter.source_kind, "backend_read_model");
        assert_eq!(adapter.status, "not_connected");
        assert_eq!(adapter.execution_status, "not_connected");
        assert_eq!(
            adapter.unavailable_reason.as_deref(),
            Some("codex_signal_missing")
        );
        assert!(adapter
            .warnings
            .contains(&"workflow_state_snapshot_missing_for_adapter_descriptor".to_string()));
        assert!(adapter.capabilities.iter().any(|capability| {
            capability.kind == "session_index_read"
                && capability.status == "blocked"
                && capability
                    .warnings
                    .contains(&"codex_session_index_empty".to_string())
        }));
        assert!(snapshot.agent_adapters.iter().any(|descriptor| {
            descriptor.adapter_id == "opencode-like"
                && descriptor.status == "planned"
                && descriptor.implemented_action_kinds.is_empty()
                && descriptor.credential_status == "not_configured"
        }));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn session_operation_descriptors_cover_e2_boundary_matrix() {
        let descriptors = derive_agent_adapter_descriptors(&[], &[], None, None);
        let operations = derive_session_operation_descriptors(&descriptors);

        assert_eq!(descriptors.len(), 5);
        assert_eq!(operations.len(), 40);

        let required_operations = [
            "new_session",
            "send_message",
            "stop",
            "restart",
            "resume",
            "export",
            "delete",
            "favorite",
        ];
        for operation_id in required_operations {
            assert_eq!(
                operations
                    .iter()
                    .filter(|operation| operation.operation_id == operation_id)
                    .count(),
                5,
                "{operation_id} should be present for every adapter"
            );
        }

        assert!(operations.iter().all(|operation| ![
            "available",
            "available_to_execute",
            "executable"
        ]
        .contains(&operation.current_status.as_str())));
        assert!(operations.iter().all(|operation| operation
            .warnings
            .contains(&"session_operation_boundary_read_model_only".to_string())));
        assert!(operations.iter().all(|operation| operation
            .warnings
            .contains(&"no_session_operation_execution_in_e2".to_string())));

        let codex_new_session = operations
            .iter()
            .find(|operation| {
                operation.adapter_id == "codex-local" && operation.operation_id == "new_session"
            })
            .expect("codex new_session operation");
        assert_eq!(codex_new_session.current_status, "requires_future_task");
        assert!(codex_new_session.requires_user_confirmation);
        assert!(codex_new_session.writes_codex_home);
        assert!(codex_new_session.writes_workbench_state);
        assert_eq!(
            codex_new_session.applies_to_session_state,
            "work_item_without_native_session"
        );
        assert!(codex_new_session
            .warnings
            .contains(&"h3_1_new_session_noop_only".to_string()));

        let codex_send = operations
            .iter()
            .find(|operation| {
                operation.adapter_id == "codex-local" && operation.operation_id == "send_message"
            })
            .expect("codex send_message operation");
        assert_eq!(codex_send.current_status, "requires_future_task");
        assert!(codex_send.requires_user_confirmation);
        assert!(codex_send.writes_codex_home);
        assert!(codex_send.writes_workbench_state);

        let codex_resume = operations
            .iter()
            .find(|operation| {
                operation.adapter_id == "codex-local" && operation.operation_id == "resume"
            })
            .expect("codex resume operation");
        assert_eq!(codex_resume.current_status, "requires_future_task");
        assert!(codex_resume
            .warnings
            .contains(&"workflow_dispatch_is_not_session_center_resume".to_string()));
        assert!(codex_resume
            .unavailable_reason
            .contains("不等于会话中心通用 resume"));

        let delete_operations = operations
            .iter()
            .filter(|operation| operation.operation_id == "delete")
            .collect::<Vec<_>>();
        assert_eq!(delete_operations.len(), 5);
        assert!(delete_operations.iter().all(|operation| {
            operation.current_status == "blocked_destructive"
                && operation.risk_level == "destructive"
                && operation.writes_codex_home
                && operation
                    .warnings
                    .contains(&"destructive_operation_blocked".to_string())
        }));

        let planned_operations = operations
            .iter()
            .filter(|operation| operation.adapter_id != "codex-local")
            .collect::<Vec<_>>();
        assert_eq!(planned_operations.len(), 32);
        assert!(planned_operations.iter().all(|operation| operation
            .warnings
            .contains(&"planned_adapter_operation_not_available".to_string())));
        assert!(planned_operations
            .iter()
            .all(|operation| operation.applies_to_session_state
                == "planned_adapter_without_session_source"));
    }

    #[test]
    fn provider_availability_summaries_cover_e3_boundary_matrix() {
        let descriptors = derive_agent_adapter_descriptors(&[], &[], None, None);
        let operations = derive_session_operation_descriptors(&descriptors);
        let summaries = derive_provider_availability_summaries(&descriptors, &operations);

        assert_eq!(descriptors.len(), 5);
        assert_eq!(summaries.len(), 5);
        assert!(summaries.iter().all(|summary| summary.safe_to_display));
        assert!(summaries.iter().all(|summary| summary
            .warnings
            .contains(&"provider_availability_read_model_only".to_string())));
        assert!(summaries.iter().all(|summary| summary
            .warnings
            .contains(&"credential_secret_not_read".to_string())));
        assert!(summaries.iter().all(|summary| summary
            .warnings
            .contains(&"provider_availability_not_project_authorization".to_string())));

        let codex = summaries
            .iter()
            .find(|summary| summary.adapter_id == "codex-local")
            .expect("codex provider summary");
        assert_eq!(codex.provider_kind, "local_cli");
        assert_eq!(codex.credential_status, "not_required_by_workbench");
        assert_eq!(codex.model_status, "local_cli_managed");
        assert_eq!(codex.external_call_status, "not_needed_for_readonly");
        assert_eq!(codex.cost_risk_status, "unknown");
        assert!(codex.requires_future_task);
        assert!(codex.user_visible_reason.contains("不读取凭据"));
        assert!(codex.user_visible_reason.contains("不验证模型"));

        let planned = summaries
            .iter()
            .filter(|summary| summary.adapter_id != "codex-local")
            .collect::<Vec<_>>();
        assert_eq!(planned.len(), 4);
        assert!(planned.iter().all(|summary| {
            summary.availability_status == "planned"
                && summary.credential_status == "credential_missing"
                && summary.model_status == "model_unverified"
                && summary.external_call_status == "external_call_blocked"
                && summary.cost_risk_status == "blocked_until_authorized"
                && summary.requires_user_configuration
                && summary.requires_future_task
                && summary
                    .warnings
                    .contains(&"planned_adapter_not_connected".to_string())
                && summary
                    .warnings
                    .contains(&"external_call_blocked".to_string())
        }));
        assert!(summaries.iter().all(|summary| ![
            "model_available",
            "credential_configured",
            "available_to_execute",
            "provider_verified"
        ]
        .contains(&summary.availability_status.as_str())));
    }

    #[test]
    fn session_continuation_guard_covers_e4_boundary_matrix() {
        let descriptors = derive_agent_adapter_descriptors(&[], &[], None, None);
        let operations = derive_session_operation_descriptors(&descriptors);
        let summaries = derive_provider_availability_summaries(&descriptors, &operations);
        let codex = descriptors
            .iter()
            .find(|descriptor| descriptor.adapter_id == "codex-local")
            .expect("codex adapter descriptor");
        let codex_send = operations
            .iter()
            .find(|operation| {
                operation.adapter_id == "codex-local" && operation.operation_id == "send_message"
            })
            .expect("codex send operation");
        let codex_provider = summaries
            .iter()
            .find(|summary| summary.adapter_id == "codex-local");
        let request = SessionContinuationRequest {
            adapter_id: "codex-local".to_string(),
            operation_id: "send_message".to_string(),
            project_id: Some("project:fixture".to_string()),
            project_root: Some("/workspace/project".to_string()),
            workflow_id: Some("workflow:fixture".to_string()),
            node_id: Some("node:dev".to_string()),
            session_id: Some("thread-fixture".to_string()),
            work_item_id: Some("work-item:fixture".to_string()),
            target_cwd: Some("/workspace/project".to_string()),
            allowed_write_roots: vec!["/workspace/project".to_string()],
            sandbox: "workspace-write-preview-only".to_string(),
            prompt_source_kind: "workflow_followup".to_string(),
            prompt_summary: "E4 prompt summary preview only".to_string(),
            readback_strategy: "required".to_string(),
            requested_by: "test".to_string(),
            user_confirmation_state: "missing".to_string(),
        };

        let needs_confirmation = inspect_session_continuation_guard(
            &request,
            Some(codex),
            Some(codex_send),
            codex_provider,
        );
        assert_eq!(needs_confirmation.status, "needs_user_confirmation");
        assert!(needs_confirmation.allows_preview);
        assert!(needs_confirmation.blocks_execution);
        assert!(needs_confirmation.requires_user_confirmation);
        assert!(needs_confirmation
            .reasons
            .contains(&"user_confirmation_required_before_execution".to_string()));

        let confirmed = inspect_session_continuation_guard(
            &SessionContinuationRequest {
                user_confirmation_state: "confirmed".to_string(),
                ..request.clone()
            },
            Some(codex),
            Some(codex_send),
            codex_provider,
        );
        assert_eq!(confirmed.status, "allowed_preview");
        assert!(confirmed.blocks_execution);
        assert!(!confirmed.requires_user_confirmation);

        let missing_project = inspect_session_continuation_guard(
            &SessionContinuationRequest {
                project_id: None,
                ..request.clone()
            },
            Some(codex),
            Some(codex_send),
            codex_provider,
        );
        assert_eq!(missing_project.status, "blocked");
        assert!(missing_project
            .reasons
            .contains(&"missing_project_binding".to_string()));

        let out_of_scope = inspect_session_continuation_guard(
            &SessionContinuationRequest {
                target_cwd: Some("/workspace/other".to_string()),
                ..request.clone()
            },
            Some(codex),
            Some(codex_send),
            codex_provider,
        );
        assert_eq!(out_of_scope.status, "blocked");
        assert!(out_of_scope
            .reasons
            .contains(&"cwd_out_of_scope_blocked".to_string()));

        let sensitive = inspect_session_continuation_guard(
            &SessionContinuationRequest {
                target_cwd: Some("/workspace/project/.env".to_string()),
                ..request.clone()
            },
            Some(codex),
            Some(codex_send),
            codex_provider,
        );
        assert_eq!(sensitive.status, "blocked");
        assert!(sensitive
            .reasons
            .iter()
            .any(|reason| reason.starts_with("sensitive_path_blocked")));

        let no_readback = inspect_session_continuation_guard(
            &SessionContinuationRequest {
                readback_strategy: "not_defined".to_string(),
                ..request.clone()
            },
            Some(codex),
            Some(codex_send),
            codex_provider,
        );
        assert_eq!(no_readback.status, "blocked");
        assert!(no_readback
            .reasons
            .contains(&"readback_strategy_required".to_string()));

        let codex_new_session = operations
            .iter()
            .find(|operation| {
                operation.adapter_id == "codex-local" && operation.operation_id == "new_session"
            })
            .expect("codex new_session operation");
        let new_session = inspect_session_continuation_guard(
            &SessionContinuationRequest {
                operation_id: "new_session".to_string(),
                session_id: None,
                prompt_source_kind: "h3_new_session_task_package".to_string(),
                ..request.clone()
            },
            Some(codex),
            Some(codex_new_session),
            codex_provider,
        );
        assert_eq!(new_session.status, "needs_user_confirmation");
        assert!(new_session.blocks_execution);
        assert!(!new_session
            .reasons
            .contains(&"missing_session_binding".to_string()));
        assert!(new_session
            .warnings
            .contains(&"new_session_does_not_require_existing_session".to_string()));

        let missing_work_item = inspect_session_continuation_guard(
            &SessionContinuationRequest {
                operation_id: "new_session".to_string(),
                session_id: None,
                work_item_id: None,
                prompt_source_kind: "h3_new_session_task_package".to_string(),
                ..request.clone()
            },
            Some(codex),
            Some(codex_new_session),
            codex_provider,
        );
        assert_eq!(missing_work_item.status, "blocked");
        assert!(missing_work_item
            .reasons
            .contains(&"missing_work_item_binding".to_string()));

        let planned_adapter = descriptors
            .iter()
            .find(|descriptor| descriptor.adapter_id == "claude-code")
            .expect("planned adapter descriptor");
        let planned_operation = operations
            .iter()
            .find(|operation| {
                operation.adapter_id == "claude-code" && operation.operation_id == "send_message"
            })
            .expect("planned send operation");
        let planned_provider = summaries
            .iter()
            .find(|summary| summary.adapter_id == "claude-code");
        let planned = inspect_session_continuation_guard(
            &SessionContinuationRequest {
                adapter_id: "claude-code".to_string(),
                ..request
            },
            Some(planned_adapter),
            Some(planned_operation),
            planned_provider,
        );
        assert_eq!(planned.status, "blocked");
        assert!(planned
            .reasons
            .iter()
            .any(|reason| reason.contains("planned_adapter_blocked")));
        assert!(planned
            .warnings
            .contains(&"provider_availability_not_execution_authorization".to_string()));
    }

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

    #[test]
    fn transcript_reader_rejects_thread_outside_index() {
        let index = json!({
          "threads": [
            {
              "thread_id": "indexed-thread",
              "title": "Indexed",
              "rollout_path": "/tmp/indexed.jsonl",
              "rollout_exists": true
            }
          ]
        });

        assert!(find_index_thread(&index, "indexed-thread").is_some());
        assert!(find_index_thread(&index, "missing-thread").is_none());
    }

    #[test]
    fn parses_transcript_reader_output() {
        let transcript = json!({
          "thread_id": "indexed-thread",
          "rollout_path": "/tmp/indexed.jsonl",
          "project_path": "/tmp/project",
          "title": "Indexed",
          "created_at_ms": 1,
          "updated_at_ms": 2,
          "events": [
            {
              "event_id": "line-000001",
              "event_type": "user_message",
              "actor": "user",
              "role": "user",
              "text": "hello",
              "warnings": []
            },
            {
              "event_id": "line-000002",
              "event_type": "command_output",
              "actor": "tool",
              "stdout": "ok",
              "stderr": "",
              "exit_code": 0,
              "warnings": ["sample_warning"]
            }
          ],
          "summary": {
            "total_events": 2,
            "event_type_counts": {
              "user_message": 1,
              "command_output": 1
            },
            "unknown_event_count": 0,
            "warning_count": 1,
            "encrypted_content_event_count": 0,
            "sensitive_like_event_count": 0
          },
          "warnings": [],
          "source_stats": {
            "jsonl": {
              "line_count": 2,
              "parsed_line_count": 2,
              "bad_json_line_count": 0
            }
          }
        });

        let parsed = parse_codex_transcript(&transcript).expect("transcript should parse");

        assert_eq!(parsed.thread_id, "indexed-thread");
        assert_eq!(parsed.summary.total_events, 2);
        assert_eq!(
            parsed.summary.event_type_counts.get("command_output"),
            Some(&1)
        );
        assert_eq!(parsed.events[0].text.as_deref(), Some("hello"));
        assert_eq!(parsed.events[1].stdout.as_deref(), Some("ok"));
        assert_eq!(parsed.events[1].warnings, vec!["sample_warning"]);
    }

    #[test]
    fn transcript_catalog_reads_sqlite_thread_not_in_index() {
        let fixture = transcript_catalog_fixture("transcript-catalog-sqlite-only", "sqlite-thread");
        let index = transcript_index(&fixture.codex_home, Vec::new());

        let transcript =
            load_codex_session_transcript_with_catalog(&index, "sqlite-thread", &fixture.db_path)
                .expect("sqlite-only transcript should read");

        assert_eq!(transcript.thread_id, "sqlite-thread");
        assert_eq!(transcript.title.as_deref(), Some("Sqlite thread"));
        assert_eq!(
            transcript.events[0].text.as_deref(),
            Some("hello from sqlite")
        );
        assert_eq!(transcript.source_stats["catalog_source"], json!("sqlite"));
    }

    #[test]
    fn transcript_catalog_sqlite_overrides_stale_index_rollout_status() {
        let fixture =
            transcript_catalog_fixture("transcript-catalog-sqlite-overrides", "same-thread");
        let stale_index = transcript_index(
            &fixture.codex_home,
            vec![json!({
                "thread_id": "same-thread",
                "title": "Stale index",
                "project_root": "/tmp/stale",
                "rollout_path": fixture.codex_home.join("sessions").join("missing.jsonl").display().to_string(),
                "rollout_exists": false
            })],
        );

        let transcript = load_codex_session_transcript_with_catalog(
            &stale_index,
            "same-thread",
            &fixture.db_path,
        )
        .expect("sqlite authority should override stale index");

        assert_eq!(transcript.title.as_deref(), Some("Sqlite thread"));
        assert_eq!(
            transcript.events[0].text.as_deref(),
            Some("hello from sqlite")
        );
        assert_eq!(transcript.source_stats["catalog_source"], json!("sqlite"));
    }

    #[test]
    fn transcript_catalog_rejects_sqlite_rollout_outside_allowed_dirs() {
        let dir = test_temp_dir("transcript-catalog-outside");
        fs::create_dir_all(&dir).expect("create temp dir");
        let codex_home = dir.join("fake-codex-home");
        fs::create_dir_all(codex_home.join("sessions")).expect("create sessions");
        fs::create_dir_all(codex_home.join("archived_sessions")).expect("create archived");
        let outside = dir.join("outside.jsonl");
        write_test_rollout(&outside, "outside");
        let db_path = codex_home.join("state_5.sqlite");
        create_test_threads_db(&db_path, "outside-thread", &outside);
        let index = transcript_index(&codex_home, Vec::new());

        let error = load_codex_session_transcript_with_catalog(&index, "outside-thread", &db_path)
            .expect_err("outside rollout should be rejected");

        assert!(error.starts_with("rollout_outside_allowed_dirs:"));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn transcript_catalog_classifies_missing_sqlite_rollout() {
        let dir = test_temp_dir("transcript-catalog-missing");
        fs::create_dir_all(&dir).expect("create temp dir");
        let codex_home = dir.join("fake-codex-home");
        let sessions_dir = codex_home.join("sessions");
        fs::create_dir_all(&sessions_dir).expect("create sessions");
        fs::create_dir_all(codex_home.join("archived_sessions")).expect("create archived");
        let missing = sessions_dir.join("missing.jsonl");
        let db_path = codex_home.join("state_5.sqlite");
        create_test_threads_db(&db_path, "missing-thread", &missing);
        let index = transcript_index(&codex_home, Vec::new());

        let error = load_codex_session_transcript_with_catalog(&index, "missing-thread", &db_path)
            .expect_err("missing rollout should be classified");

        assert!(error.starts_with("rollout_missing:"));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn transcript_catalog_falls_back_to_index_when_sqlite_unavailable() {
        let dir = test_temp_dir("transcript-catalog-index-fallback");
        let codex_home = dir.join("fake-codex-home");
        let sessions_dir = codex_home.join("sessions");
        fs::create_dir_all(&sessions_dir).expect("create sessions");
        fs::create_dir_all(codex_home.join("archived_sessions")).expect("create archived");
        let rollout = sessions_dir.join("index-thread.jsonl");
        write_test_rollout(&rollout, "hello from index");
        let index = transcript_index(
            &codex_home,
            vec![json!({
                "thread_id": "index-thread",
                "title": "Index thread",
                "project_root": "/tmp/index-project",
                "rollout_path": rollout.display().to_string(),
                "rollout_exists": true,
                "updated_at_ms": 55
            })],
        );

        let transcript = load_codex_session_transcript_with_catalog(
            &index,
            "index-thread",
            &codex_home.join("missing-state.sqlite"),
        )
        .expect("index fallback should read when sqlite unavailable");

        assert_eq!(
            transcript.events[0].text.as_deref(),
            Some("hello from index")
        );
        assert_eq!(
            transcript.source_stats["catalog_source"],
            json!("index_fallback_sqlite_unavailable")
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn transcript_catalog_main_path_does_not_need_python_reader() {
        let fixture = transcript_catalog_fixture("transcript-catalog-no-python", "sqlite-thread");
        let index = transcript_index(&fixture.codex_home, Vec::new());

        let transcript =
            load_codex_session_transcript_with_catalog(&index, "sqlite-thread", &fixture.db_path)
                .expect("native reader should not need transcript_reader.py");

        assert_eq!(transcript.source_stats["catalog_source"], json!("sqlite"));
        assert_eq!(transcript.summary.total_events, 1);
    }

    #[test]
    fn transcript_catalog_reads_sqlite_thread_without_index_catalog() {
        let fixture = transcript_catalog_fixture("transcript-catalog-no-index", "sqlite-thread");

        let transcript = load_codex_session_transcript_with_optional_catalog(
            None,
            "sqlite-thread",
            &fixture.db_path,
            Some("索引 JSON 解析失败".to_string()),
        )
        .expect("sqlite transcript should read even when index is unavailable");

        assert_eq!(transcript.source_stats["catalog_source"], json!("sqlite"));
        assert_eq!(
            transcript.events[0].text.as_deref(),
            Some("hello from sqlite")
        );
    }

    #[test]
    fn dispatch_readback_stats_reads_sqlite_only_native_rollout() {
        let fixture = transcript_catalog_fixture("dispatch-readback-sqlite-only", "sqlite-thread");
        let index = transcript_index(&fixture.codex_home, Vec::new());

        let stats = dispatch_readback_stats_native(
            Some(&index),
            &fixture.db_path,
            "sqlite-thread",
            "hello from sqlite",
        )
        .expect("sqlite-only native readback should read");

        assert_eq!(stats.transcript_event_count, 1);
        assert_eq!(stats.transcript_target_hits, 1);
        assert!(!fixture
            .codex_home
            .parent()
            .expect("fixture codex home should have parent")
            .join("transcript_reader.py")
            .exists());
    }

    #[test]
    fn dispatch_readback_stats_reads_sqlite_when_index_unavailable() {
        let fixture = transcript_catalog_fixture("dispatch-readback-no-index", "sqlite-thread");

        let stats = dispatch_readback_stats_native(
            None,
            &fixture.db_path,
            "sqlite-thread",
            "hello from sqlite",
        )
        .expect("sqlite readback should not need index");

        assert_eq!(stats.transcript_event_count, 1);
        assert_eq!(stats.transcript_target_hits, 1);
    }

    #[test]
    fn dispatch_readback_stats_falls_back_to_index_when_sqlite_unavailable() {
        let dir = test_temp_dir("dispatch-readback-index-fallback");
        let codex_home = dir.join("fake-codex-home");
        let sessions_dir = codex_home.join("sessions");
        fs::create_dir_all(&sessions_dir).expect("create sessions");
        fs::create_dir_all(codex_home.join("archived_sessions")).expect("create archived");
        let rollout = sessions_dir.join("index-thread.jsonl");
        write_test_rollout(&rollout, safe_probe_target());
        let index = transcript_index(
            &codex_home,
            vec![json!({
                "thread_id": "index-thread",
                "title": "Index thread",
                "project_root": "/tmp/index-project",
                "rollout_path": rollout.display().to_string(),
                "rollout_exists": true,
                "updated_at_ms": 55
            })],
        );

        let stats = dispatch_readback_stats_native(
            Some(&index),
            &codex_home.join("missing-state.sqlite"),
            "index-thread",
            safe_probe_target(),
        )
        .expect("index fallback should read when sqlite unavailable");

        assert_eq!(stats.transcript_event_count, 1);
        assert_eq!(stats.transcript_target_hits, 1);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn dispatch_readback_stats_hits_safe_probe_target_in_text_and_stdout() {
        let fixture = dispatch_readback_fixture(
            "dispatch-readback-target-hit",
            "thread-hit",
            vec![
                dispatch_text_event("noise"),
                dispatch_text_event(safe_probe_target()),
                dispatch_stdout_event(safe_probe_target()),
            ],
        );
        let index = transcript_index(&fixture.codex_home, Vec::new());

        let stats = dispatch_readback_stats_native(
            Some(&index),
            &fixture.db_path,
            "thread-hit",
            safe_probe_target(),
        )
        .expect("native readback should count target hits");

        assert_eq!(stats.transcript_event_count, 3);
        assert_eq!(stats.transcript_target_hits, 2);
    }

    #[test]
    fn dispatch_readback_stats_returns_zero_hits_when_target_missing() {
        let fixture = dispatch_readback_fixture(
            "dispatch-readback-target-missing",
            "thread-missing",
            vec![
                dispatch_text_event("noise"),
                dispatch_stdout_event("more noise"),
            ],
        );
        let index = transcript_index(&fixture.codex_home, Vec::new());

        let stats = dispatch_readback_stats_native(
            Some(&index),
            &fixture.db_path,
            "thread-missing",
            safe_probe_target(),
        )
        .expect("native readback should return stats");

        assert_eq!(stats.transcript_event_count, 2);
        assert_eq!(stats.transcript_target_hits, 0);
    }

    #[test]
    fn dispatch_readback_stats_failure_preserves_zero_zero_downgrade() {
        let index = transcript_index(&test_temp_dir("dispatch-readback-empty-index"), Vec::new());

        let stats = dispatch_readback_stats_native(
            Some(&index),
            &PathBuf::from("/tmp/missing-dispatch-readback-state.sqlite"),
            "missing-thread",
            safe_probe_target(),
        )
        .expect("readback failure should keep compatibility downgrade");

        assert_eq!(stats.transcript_event_count, 0);
        assert_eq!(stats.transcript_target_hits, 0);
    }

    struct TranscriptCatalogFixture {
        codex_home: PathBuf,
        db_path: PathBuf,
    }

    fn transcript_catalog_fixture(prefix: &str, thread_id: &str) -> TranscriptCatalogFixture {
        let dir = test_temp_dir(prefix);
        fs::create_dir_all(&dir).expect("create temp dir");
        let codex_home = dir.join("fake-codex-home");
        let sessions_dir = codex_home.join("sessions");
        fs::create_dir_all(&sessions_dir).expect("create sessions");
        fs::create_dir_all(codex_home.join("archived_sessions")).expect("create archived");
        let rollout = sessions_dir.join(format!("{thread_id}.jsonl"));
        write_test_rollout(&rollout, "hello from sqlite");
        let db_path = codex_home.join("state_5.sqlite");
        create_test_threads_db(&db_path, thread_id, &rollout);
        TranscriptCatalogFixture {
            codex_home,
            db_path,
        }
    }

    fn dispatch_readback_fixture(
        prefix: &str,
        thread_id: &str,
        events: Vec<Value>,
    ) -> TranscriptCatalogFixture {
        let dir = test_temp_dir(prefix);
        fs::create_dir_all(&dir).expect("create temp dir");
        let codex_home = dir.join("fake-codex-home");
        let sessions_dir = codex_home.join("sessions");
        fs::create_dir_all(&sessions_dir).expect("create sessions");
        fs::create_dir_all(codex_home.join("archived_sessions")).expect("create archived");
        let rollout = sessions_dir.join(format!("{thread_id}.jsonl"));
        write_test_rollout_events(&rollout, events);
        let db_path = codex_home.join("state_5.sqlite");
        create_test_threads_db(&db_path, thread_id, &rollout);
        TranscriptCatalogFixture {
            codex_home,
            db_path,
        }
    }

    fn dispatch_text_event(message: &str) -> Value {
        json!({
            "timestamp": "2026-06-03T00:00:00Z",
            "type": "event_msg",
            "payload": {
                "type": "user_message",
                "message": message
            }
        })
    }

    fn dispatch_stdout_event(stdout: &str) -> Value {
        json!({
            "timestamp": "2026-06-03T00:00:01Z",
            "type": "response_item",
            "payload": {
                "type": "function_call_output",
                "call_id": "call-dispatch-readback",
                "output": json!({
                    "stdout": stdout,
                    "stderr": "",
                    "exit_code": 0
                }).to_string()
            }
        })
    }

    fn transcript_index(codex_home: &Path, threads: Vec<Value>) -> Value {
        json!({
            "threads": threads,
            "source_stats": {
                "codex_home": {
                    "path": codex_home.display().to_string(),
                    "role": "data_source_root"
                }
            }
        })
    }

    fn write_test_rollout(path: &Path, message: &str) {
        let row = json!({
            "timestamp": "2026-06-03T00:00:00Z",
            "type": "event_msg",
            "payload": {
                "type": "user_message",
                "message": message
            }
        });
        fs::write(path, format!("{row}\n")).expect("write rollout");
    }

    fn write_test_rollout_events(path: &Path, events: Vec<Value>) {
        let text = events
            .into_iter()
            .map(|event| event.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(path, format!("{text}\n")).expect("write rollout events");
    }

    fn create_test_threads_db(db_path: &Path, thread_id: &str, rollout_path: &Path) {
        let conn = rusqlite::Connection::open(db_path).expect("open sqlite");
        conn.execute_batch(
            r#"
            CREATE TABLE threads (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                cwd TEXT NOT NULL,
                updated_at_ms INTEGER,
                archived INTEGER NOT NULL,
                rollout_path TEXT NOT NULL,
                model TEXT,
                reasoning_effort TEXT,
                thread_source TEXT,
                has_user_event INTEGER NOT NULL
            );
            "#,
        )
        .expect("create threads table");
        conn.execute(
            r#"
            INSERT INTO threads (
                id,
                title,
                cwd,
                updated_at_ms,
                archived,
                rollout_path,
                model,
                reasoning_effort,
                thread_source,
                has_user_event
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            "#,
            rusqlite::params![
                thread_id,
                "Sqlite thread",
                "/tmp/sqlite-project",
                1000_i64,
                0_i64,
                rollout_path.display().to_string(),
                "gpt-test",
                "medium",
                "codex",
                1_i64,
            ],
        )
        .expect("insert thread");
    }

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

    #[test]
    fn workflow_authorization_plan_authorization_guard_blocks_without_authorization() {
        let timestamp_ms = 1_765_000_000_000;
        let project_root = "/tmp/c1-plan-auth-project";
        let store = PlanAuthorizationStoreV1 {
            schema_version: "plan_authorization_store.v1".to_string(),
            revision: 0,
            authorizations: vec![],
            audit_events: vec![],
            updated_at_ms: timestamp_ms,
            warnings: vec![],
        };
        let input = fixture_plan_authorization_guard_input(project_root);

        let result = control_core::inspect_auto_dispatch_scope(&store, &input, timestamp_ms);

        assert_eq!(result.status, "blocked");
        assert!(result
            .reasons
            .iter()
            .any(|reason| reason.contains("缺少有效方案授权")));
        assert!(result.required_user_confirmation);
        assert!(result.required_global_review);
    }

    #[test]
    fn plan_authorization_guard_needs_review_before_user_and_global_approval() {
        let timestamp_ms = 1_765_000_000_000;
        let project_root = "/tmp/c1-plan-auth-project";
        let input = fixture_plan_authorization_guard_input(project_root);
        let pending_user = fixture_plan_authorization_store_with_status(
            project_root,
            PlanAuthorizationStatus::PendingUserConfirmation,
            timestamp_ms,
        );
        let pending_global = fixture_plan_authorization_store_with_status(
            project_root,
            PlanAuthorizationStatus::PendingGlobalBoundaryReview,
            timestamp_ms,
        );

        let user_result =
            control_core::inspect_auto_dispatch_scope(&pending_user, &input, timestamp_ms);
        let global_result =
            control_core::inspect_auto_dispatch_scope(&pending_global, &input, timestamp_ms);

        assert_eq!(user_result.status, "needs_review");
        assert!(user_result.required_user_confirmation);
        assert!(user_result
            .reasons
            .iter()
            .any(|reason| reason.contains("待用户确认")));
        assert_eq!(global_result.status, "needs_review");
        assert!(global_result.required_global_review);
        assert!(global_result
            .reasons
            .iter()
            .any(|reason| reason.contains("待全局边界复核")));
    }

    #[test]
    fn plan_authorization_guard_authorizes_matching_active_scope() {
        let timestamp_ms = 1_765_000_000_000;
        let project_root = "/tmp/c1-plan-auth-project";
        let store = fixture_plan_authorization_store_with_status(
            project_root,
            PlanAuthorizationStatus::Active,
            timestamp_ms,
        );
        let input = fixture_plan_authorization_guard_input(project_root);

        let result = control_core::inspect_auto_dispatch_scope(&store, &input, timestamp_ms);

        assert_eq!(result.status, "authorized", "{:?}", result.reasons);
        assert_eq!(
            result.authorization_id.as_deref(),
            Some("plan-auth:c1-fixture")
        );
        assert!(result.reasons.is_empty());
    }

    #[test]
    fn plan_authorization_guard_blocks_write_scope_escape() {
        let timestamp_ms = 1_765_000_000_000;
        let project_root = "/tmp/c1-plan-auth-project";
        let store = fixture_plan_authorization_store_with_status(
            project_root,
            PlanAuthorizationStatus::Active,
            timestamp_ms,
        );
        let mut input = fixture_plan_authorization_guard_input(project_root);
        input.requested_write_roots = vec!["/tmp/c1-plan-auth-outside".to_string()];

        let result = control_core::inspect_auto_dispatch_scope(&store, &input, timestamp_ms);

        assert_eq!(result.status, "blocked");
        assert!(result
            .reasons
            .iter()
            .any(|reason| reason.contains("写入范围超出方案授权")));
    }

    #[test]
    fn plan_authorization_guard_blocks_role_and_agent_escape() {
        let timestamp_ms = 1_765_000_000_000;
        let project_root = "/tmp/c1-plan-auth-project";
        let store = fixture_plan_authorization_store_with_status(
            project_root,
            PlanAuthorizationStatus::Active,
            timestamp_ms,
        );
        let mut input = fixture_plan_authorization_guard_input(project_root);
        input.target_role_id = "validation".to_string();
        input.target_agent_id = Some("agent-2".to_string());

        let result = control_core::inspect_auto_dispatch_scope(&store, &input, timestamp_ms);

        assert_eq!(result.status, "blocked");
        assert!(result
            .reasons
            .iter()
            .any(|reason| reason.contains("目标角色不在授权范围内")));
        assert!(result
            .reasons
            .iter()
            .any(|reason| reason.contains("目标 agent 不在授权范围内")));
    }

    #[test]
    fn plan_authorization_guard_blocks_revoked_paused_and_expired() {
        let timestamp_ms = 1_765_000_000_000;
        let project_root = "/tmp/c1-plan-auth-project";
        let input = fixture_plan_authorization_guard_input(project_root);

        for (status, expected_reason) in [
            (PlanAuthorizationStatus::Revoked, "方案授权已撤销"),
            (PlanAuthorizationStatus::Paused, "方案授权已暂停"),
            (PlanAuthorizationStatus::Expired, "方案授权已过期"),
        ] {
            let store =
                fixture_plan_authorization_store_with_status(project_root, status, timestamp_ms);
            let result = control_core::inspect_auto_dispatch_scope(&store, &input, timestamp_ms);

            assert_eq!(result.status, "blocked");
            assert!(
                result
                    .reasons
                    .iter()
                    .any(|reason| reason.contains(expected_reason)),
                "{:?}",
                result.reasons
            );
        }
    }

    #[test]
    fn plan_authorization_guard_needs_review_when_stop_condition_requires_user() {
        let timestamp_ms = 1_765_000_000_000;
        let project_root = "/tmp/c1-plan-auth-project";
        let store = fixture_plan_authorization_store_with_status(
            project_root,
            PlanAuthorizationStatus::Active,
            timestamp_ms,
        );
        let mut input = fixture_plan_authorization_guard_input(project_root);
        input.triggered_stop_conditions = vec!["requires_user_confirmation".to_string()];

        let result = control_core::inspect_auto_dispatch_scope(&store, &input, timestamp_ms);

        assert_eq!(result.status, "needs_review");
        assert!(result.required_user_confirmation);
        assert!(result
            .reasons
            .iter()
            .any(|reason| reason.contains("触发必须请用户确认的停止条件")));
    }

    #[test]
    fn plan_authorization_inspect_writes_auto_dispatch_scope_checked_audit() {
        let timestamp_ms = 1_765_000_000_000;
        let dir = test_temp_dir("plan-authorization-inspect-audit");
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/c1-plan-auth-inspect-project");

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_active_plan_authorization_for_fixture(&path, &project.project_root);
        let mut input = fixture_plan_authorization_guard_input(&project.project_root);
        input.target_agent_id = None;
        input.requested_checks = vec![];

        let result = plan_authorization_store::inspect_auto_dispatch_authorization(
            &path,
            &input,
            timestamp_ms,
            "write-c1-plan-authorization-inspect",
        )
        .expect("inspect should write audit");
        let store = plan_authorization_store::load_store(&path, timestamp_ms + 1)
            .expect("store should load");

        assert_eq!(result.status, "authorized", "{:?}", result.reasons);
        assert!(store.audit_events.iter().any(|event| {
            event.event_type == "auto_dispatch_scope_checked"
                && event.work_item_id.as_deref() == Some(input.work_item_id.as_str())
                && event
                    .guard_result
                    .as_ref()
                    .is_some_and(|guard| guard.status == "authorized")
        }));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn project_consultation_proposal_create_writes_revision_and_read_model() {
        let timestamp_ms = 1_765_100_000_000;
        let dir = test_temp_dir("project-consultation-proposal-create");
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/c2-project-consultation-create");
        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");

        let output = project_consultation_proposal_store::create_proposal(
            &path,
            &fixture_project_consultation_proposal_input(&project.project_root),
            timestamp_ms,
            "write-c2-proposal-create",
        )
        .expect("proposal should create");
        let store = project_consultation_proposal_store::load_store(&path, timestamp_ms + 1)
            .expect("proposal store should load");

        assert_eq!(output.store_revision, 1);
        assert_eq!(store.revision, 1);
        assert_eq!(
            output.proposal.status,
            ProjectConsultationProposalStatus::PendingUserConfirmation
        );
        assert_eq!(output.read_model.proposal_count, 1);
        assert_eq!(
            output.read_model.latest_status,
            Some(ProjectConsultationProposalStatus::PendingUserConfirmation)
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn project_consultation_proposal_rejects_missing_required_fields() {
        let timestamp_ms = 1_765_100_000_000;
        let dir = test_temp_dir("project-consultation-proposal-invalid");
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/c2-project-consultation-invalid");
        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        let mut input = fixture_project_consultation_proposal_input(&project.project_root);
        input.user_goal.clear();
        let err = project_consultation_proposal_store::create_proposal(
            &path,
            &input,
            timestamp_ms,
            "write-c2-proposal-invalid-goal",
        )
        .expect_err("missing user_goal should fail");
        assert!(err.contains("user_goal"));

        let mut input = fixture_project_consultation_proposal_input(&project.project_root);
        input.acceptance_criteria.clear();
        let err = project_consultation_proposal_store::create_proposal(
            &path,
            &input,
            timestamp_ms,
            "write-c2-proposal-invalid-acceptance",
        )
        .expect_err("missing acceptance criteria should fail");
        assert!(err.contains("acceptance criterion"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn project_consultation_proposal_confirm_creates_user_confirmed_authorization_not_active() {
        let timestamp_ms = 1_765_100_000_000;
        let dir = test_temp_dir("project-consultation-proposal-confirm");
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/c2-project-consultation-confirm");
        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        let created = project_consultation_proposal_store::create_proposal(
            &path,
            &fixture_project_consultation_proposal_input(&project.project_root),
            timestamp_ms,
            "write-c2-proposal-confirm-create",
        )
        .expect("proposal should create");

        let output = project_consultation_proposal_store::record_decision(
            &path,
            &RecordProjectConsultationProposalDecisionInput {
                project_root: project.project_root.clone(),
                proposal_id: created.proposal.proposal_id.clone(),
                actor_id: "user-fixture".to_string(),
                decision: ProjectConsultationProposalDecisionKind::Confirm,
                summary: "用户确认 C2 测试方案；仍需全局主管复核。".to_string(),
                expected_proposal_store_revision: Some(created.store_revision),
                expected_plan_authorization_store_revision: None,
            },
            timestamp_ms + 1,
            "write-c2-proposal-confirm",
            "write-c2-proposal-confirm-auth",
            "write-c2-proposal-confirm-auth-user",
        )
        .expect("confirm should write proposal and authorization");
        let authorization = output
            .plan_authorization
            .clone()
            .expect("confirm should return linked authorization");

        assert_eq!(
            output.proposal.status,
            ProjectConsultationProposalStatus::UserConfirmed
        );
        assert_eq!(
            authorization.source_proposal_id.as_deref(),
            Some(created.proposal.proposal_id.as_str())
        );
        assert_eq!(
            authorization.status,
            PlanAuthorizationStatus::PendingGlobalBoundaryReview
        );
        assert!(authorization.user_confirmation.is_some());

        let plan_store = plan_authorization_store::load_store(&path, timestamp_ms + 2)
            .expect("plan authorization store should load");
        let guard = control_core::inspect_auto_dispatch_scope(
            &plan_store,
            &fixture_plan_authorization_guard_input(&project.project_root),
            timestamp_ms + 2,
        );
        assert_eq!(guard.status, "needs_review");
        assert!(guard.required_global_review);
        assert!(guard
            .reasons
            .iter()
            .any(|reason| reason.contains("待全局边界复核")));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn project_consultation_proposal_request_changes_and_reject_do_not_create_authorization() {
        for (decision, suffix) in [
            (
                ProjectConsultationProposalDecisionKind::RequestChanges,
                "request-changes",
            ),
            (ProjectConsultationProposalDecisionKind::Reject, "reject"),
        ] {
            let timestamp_ms = 1_765_100_000_000;
            let dir = test_temp_dir(&format!("project-consultation-proposal-{suffix}"));
            let path = dir.join("workflow-state.v0.json");
            let project = fixture_project(&format!("/tmp/c2-project-consultation-{suffix}"));
            bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
            let created = project_consultation_proposal_store::create_proposal(
                &path,
                &fixture_project_consultation_proposal_input(&project.project_root),
                timestamp_ms,
                &format!("write-c2-proposal-{suffix}-create"),
            )
            .expect("proposal should create");

            let output = project_consultation_proposal_store::record_decision(
                &path,
                &RecordProjectConsultationProposalDecisionInput {
                    project_root: project.project_root.clone(),
                    proposal_id: created.proposal.proposal_id.clone(),
                    actor_id: "user-fixture".to_string(),
                    decision,
                    summary: "用户未确认当前项目咨询方案。".to_string(),
                    expected_proposal_store_revision: Some(created.store_revision),
                    expected_plan_authorization_store_revision: None,
                },
                timestamp_ms + 1,
                &format!("write-c2-proposal-{suffix}"),
                &format!("write-c2-proposal-{suffix}-auth"),
                &format!("write-c2-proposal-{suffix}-auth-user"),
            )
            .expect("decision should write");
            let plan_store = plan_authorization_store::load_store(&path, timestamp_ms + 2)
                .expect("empty plan authorization store should load");

            assert!(output.plan_authorization.is_none());
            assert!(plan_store.authorizations.is_empty());
            assert_eq!(
                output.proposal.status,
                if decision == ProjectConsultationProposalDecisionKind::RequestChanges {
                    ProjectConsultationProposalStatus::ChangesRequested
                } else {
                    ProjectConsultationProposalStatus::Rejected
                }
            );

            let _ = fs::remove_dir_all(dir);
        }
    }

    #[test]
    fn project_consultation_proposal_rejects_repeated_confirmation() {
        let timestamp_ms = 1_765_100_000_000;
        let dir = test_temp_dir("project-consultation-proposal-repeat-confirm");
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/c2-project-consultation-repeat-confirm");
        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        let created = project_consultation_proposal_store::create_proposal(
            &path,
            &fixture_project_consultation_proposal_input(&project.project_root),
            timestamp_ms,
            "write-c2-proposal-repeat-create",
        )
        .expect("proposal should create");
        let first = project_consultation_proposal_store::record_decision(
            &path,
            &RecordProjectConsultationProposalDecisionInput {
                project_root: project.project_root.clone(),
                proposal_id: created.proposal.proposal_id.clone(),
                actor_id: "user-fixture".to_string(),
                decision: ProjectConsultationProposalDecisionKind::Confirm,
                summary: "用户确认 C2 测试方案。".to_string(),
                expected_proposal_store_revision: Some(created.store_revision),
                expected_plan_authorization_store_revision: None,
            },
            timestamp_ms + 1,
            "write-c2-proposal-repeat-confirm",
            "write-c2-proposal-repeat-confirm-auth",
            "write-c2-proposal-repeat-confirm-auth-user",
        )
        .expect("first confirm should work");
        let err = project_consultation_proposal_store::record_decision(
            &path,
            &RecordProjectConsultationProposalDecisionInput {
                project_root: project.project_root.clone(),
                proposal_id: created.proposal.proposal_id,
                actor_id: "user-fixture".to_string(),
                decision: ProjectConsultationProposalDecisionKind::Confirm,
                summary: "重复确认应被拒绝。".to_string(),
                expected_proposal_store_revision: Some(first.store_revision),
                expected_plan_authorization_store_revision: Some(
                    first.plan_authorization_store_revision.unwrap_or(0),
                ),
            },
            timestamp_ms + 2,
            "write-c2-proposal-repeat-confirm-2",
            "write-c2-proposal-repeat-confirm-auth-2",
            "write-c2-proposal-repeat-confirm-auth-user-2",
        )
        .expect_err("second confirm should fail");

        assert!(err.contains("不能重复记录用户决定"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn global_boundary_review_approved_activates_authorization_and_guard_still_checks_scope() {
        let timestamp_ms = 1_765_200_000_000;
        let dir = test_temp_dir("global-boundary-review-approved");
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/c3-global-boundary-approved");
        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        let (proposal, authorization, revision) =
            create_confirmed_proposal_for_global_review(&path, &project.project_root, timestamp_ms);

        let output = plan_authorization_store::record_global_boundary_review_with_proposal(
            &path,
            &fixture_global_boundary_review_input(
                &project.project_root,
                &proposal.proposal_id,
                &authorization.authorization_id,
                revision,
            ),
            timestamp_ms + 2,
            "write-c3-global-boundary-approved",
        )
        .expect("approved review should activate authorization");
        let store = plan_authorization_store::load_store(&path, timestamp_ms + 3)
            .expect("plan authorization store should load");
        let mut guard_input = fixture_plan_authorization_guard_input(&project.project_root);

        assert_eq!(output.authorization.status, PlanAuthorizationStatus::Active);
        assert_eq!(output.guard_result.status, "authorized");
        assert_eq!(
            output
                .authorization
                .global_boundary_review
                .as_ref()
                .and_then(|review| review.source_proposal_id.as_deref()),
            Some(proposal.proposal_id.as_str())
        );
        assert_eq!(
            control_core::inspect_auto_dispatch_scope(&store, &guard_input, timestamp_ms + 3)
                .status,
            "authorized"
        );

        guard_input.requested_write_roots = vec!["/tmp/c3-global-boundary-outside".to_string()];
        let blocked =
            control_core::inspect_auto_dispatch_scope(&store, &guard_input, timestamp_ms + 3);
        assert_eq!(blocked.status, "blocked");
        assert!(blocked
            .reasons
            .iter()
            .any(|reason| reason.contains("写入范围超出方案授权")));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn global_boundary_review_rejects_missing_user_confirmation() {
        let timestamp_ms = 1_765_200_000_000;
        let dir = test_temp_dir("global-boundary-review-missing-user");
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/c3-global-boundary-missing-user");
        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        let (proposal, authorization, revision) =
            create_confirmed_proposal_for_global_review(&path, &project.project_root, timestamp_ms);
        let mut store = plan_authorization_store::load_store(&path, timestamp_ms + 2)
            .expect("plan authorization store should load");
        store.authorizations[0].user_confirmation = None;
        fs::write(
            plan_authorization_store::sidecar_path(&path).expect("sidecar path"),
            serde_json::to_string_pretty(&store).expect("store should serialize"),
        )
        .expect("test should write mutated sidecar");

        let err = plan_authorization_store::record_global_boundary_review_with_proposal(
            &path,
            &fixture_global_boundary_review_input(
                &project.project_root,
                &proposal.proposal_id,
                &authorization.authorization_id,
                revision,
            ),
            timestamp_ms + 3,
            "write-c3-global-boundary-missing-user",
        )
        .expect_err("missing user confirmation should fail");

        assert!(err.contains("缺少用户确认"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn global_boundary_review_rejects_proposal_authorization_mismatch() {
        let timestamp_ms = 1_765_200_000_000;
        let dir = test_temp_dir("global-boundary-review-mismatch");
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/c3-global-boundary-mismatch");
        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        let (proposal, _authorization, revision) =
            create_confirmed_proposal_for_global_review(&path, &project.project_root, timestamp_ms);

        let err = plan_authorization_store::record_global_boundary_review_with_proposal(
            &path,
            &fixture_global_boundary_review_input(
                &project.project_root,
                &proposal.proposal_id,
                "plan-auth:wrong",
                revision,
            ),
            timestamp_ms + 2,
            "write-c3-global-boundary-mismatch",
        )
        .expect_err("mismatched authorization should fail");

        assert!(err.contains("回链"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn global_boundary_review_rejects_incomplete_checklist_and_blocking_finding_for_approved() {
        let timestamp_ms = 1_765_200_000_000;
        for (suffix, mutate) in [("checklist", "checklist"), ("blocking-finding", "finding")] {
            let dir = test_temp_dir(&format!("global-boundary-review-{suffix}"));
            let path = dir.join("workflow-state.v0.json");
            let project = fixture_project(&format!("/tmp/c3-global-boundary-{suffix}"));
            bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
            let (proposal, authorization, revision) = create_confirmed_proposal_for_global_review(
                &path,
                &project.project_root,
                timestamp_ms,
            );
            let mut input = fixture_global_boundary_review_input(
                &project.project_root,
                &proposal.proposal_id,
                &authorization.authorization_id,
                revision,
            );
            if mutate == "checklist" {
                input.checklist.read_write_scope_checked = false;
            } else {
                input.findings.push(GlobalBoundaryReviewFinding {
                    finding_id: "finding:blocking".to_string(),
                    severity: "blocking".to_string(),
                    summary: "存在阻断项。".to_string(),
                    recommendation: Some("先修改方案。".to_string()),
                });
            }

            let err = plan_authorization_store::record_global_boundary_review_with_proposal(
                &path,
                &input,
                timestamp_ms + 2,
                &format!("write-c3-global-boundary-{suffix}"),
            )
            .expect_err("approved review should validate checklist and findings");

            assert!(
                err.contains("checklist") || err.contains("blocking"),
                "{err}"
            );

            let _ = fs::remove_dir_all(dir);
        }
    }

    #[test]
    fn global_boundary_review_needs_changes_and_blocked_do_not_activate_authorization() {
        let timestamp_ms = 1_765_200_000_000;
        for (status, suffix) in [("needs_changes", "needs-changes"), ("blocked", "blocked")] {
            let dir = test_temp_dir(&format!("global-boundary-review-{suffix}"));
            let path = dir.join("workflow-state.v0.json");
            let project = fixture_project(&format!("/tmp/c3-global-boundary-{suffix}"));
            bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
            let (proposal, authorization, revision) = create_confirmed_proposal_for_global_review(
                &path,
                &project.project_root,
                timestamp_ms,
            );
            let mut input = fixture_global_boundary_review_input(
                &project.project_root,
                &proposal.proposal_id,
                &authorization.authorization_id,
                revision,
            );
            input.review_status = status.to_string();
            input.summary = format!("全局主管复核结论为 {status}；不能自动推进。");
            input.findings.push(GlobalBoundaryReviewFinding {
                finding_id: format!("finding:{suffix}"),
                severity: if status == "blocked" {
                    "blocking".to_string()
                } else {
                    "warning".to_string()
                },
                summary: input.summary.clone(),
                recommendation: Some("调整方案后再复核。".to_string()),
            });

            let output = plan_authorization_store::record_global_boundary_review_with_proposal(
                &path,
                &input,
                timestamp_ms + 2,
                &format!("write-c3-global-boundary-{suffix}"),
            )
            .expect("non-approved review should write paused authorization");
            let store = plan_authorization_store::load_store(&path, timestamp_ms + 3)
                .expect("plan authorization store should load");
            let guard = control_core::inspect_auto_dispatch_scope(
                &store,
                &fixture_plan_authorization_guard_input(&project.project_root),
                timestamp_ms + 3,
            );

            assert_eq!(output.authorization.status, PlanAuthorizationStatus::Paused);
            assert_eq!(guard.status, "blocked");
            assert!(guard
                .reasons
                .iter()
                .any(|reason| reason.contains("方案授权已暂停")));

            let _ = fs::remove_dir_all(dir);
        }
    }

    #[test]
    fn project_director_task_plan_rejects_without_active_c3_authorization() {
        let timestamp_ms = 1_765_300_000_000;
        let dir = test_temp_dir("project-director-task-plan-no-c3-active");
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/c4-no-c3-active");
        let thread_id = "thread-c4-no-c3";
        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        let mut proposal_input = fixture_project_consultation_proposal_input(&project.project_root);
        proposal_input.scope_draft.allowed_agent_ids = vec![thread_id.to_string()];
        let created = project_consultation_proposal_store::create_proposal(
            &path,
            &proposal_input,
            timestamp_ms,
            "write-c4-no-c3-proposal-create",
        )
        .expect("proposal should create");
        let confirmed = project_consultation_proposal_store::record_decision(
            &path,
            &RecordProjectConsultationProposalDecisionInput {
                project_root: project.project_root.clone(),
                proposal_id: created.proposal.proposal_id.clone(),
                actor_id: "user-fixture".to_string(),
                decision: ProjectConsultationProposalDecisionKind::Confirm,
                summary: "用户确认 C4 fixture 方案；尚未全局复核。".to_string(),
                expected_proposal_store_revision: Some(created.store_revision),
                expected_plan_authorization_store_revision: None,
            },
            timestamp_ms + 1,
            "write-c4-no-c3-proposal-confirm",
            "write-c4-no-c3-plan-auth",
            "write-c4-no-c3-plan-auth-user",
        )
        .expect("proposal confirmation should create pending authorization");
        let authorization = confirmed
            .plan_authorization
            .expect("confirmed proposal should link authorization");
        let revision = confirmed
            .plan_authorization_store_revision
            .expect("confirmed proposal should return revision");
        let index = fixture_dispatch_index(&project.project_root, thread_id);
        let preview_input = fixture_project_director_preview_input(
            &project.project_root,
            &confirmed.proposal.proposal_id,
            &authorization.authorization_id,
            revision,
        );
        let prepare_input = fixture_project_director_prepare_input(
            &project.project_root,
            &confirmed.proposal.proposal_id,
            &authorization.authorization_id,
            revision,
            vec![],
        );

        let preview_error =
            preview_project_director_task_plan_for_index_at(&path, &index, &preview_input)
                .expect_err("preview should reject missing C3 approval");
        let prepare_error =
            prepare_authorized_auto_dispatch_for_index_at(&path, &index, &prepare_input)
                .expect_err("prepare should reject missing C3 approval");

        assert!(preview_error.contains("C3 approved"), "{preview_error}");
        assert!(prepare_error.contains("C3 approved"), "{prepare_error}");

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn project_director_task_plan_rejects_proposal_authorization_mismatch() {
        let timestamp_ms = 1_765_300_000_000;
        let dir = test_temp_dir("project-director-task-plan-mismatch");
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/c4-proposal-authorization-mismatch");
        let thread_id = "thread-c4-mismatch";
        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        let (proposal, authorization, revision) =
            create_active_project_director_authorization_fixture(
                &path,
                &project.project_root,
                thread_id,
                timestamp_ms,
            );
        let mut proposal_store =
            project_consultation_proposal_store::load_store(&path, timestamp_ms + 4)
                .expect("proposal store should load");
        proposal_store.proposals[0].plan_authorization_id = Some("plan-auth:wrong".to_string());
        fs::write(
            project_consultation_proposal_store::sidecar_path(&path)
                .expect("proposal sidecar path"),
            serde_json::to_string_pretty(&proposal_store).expect("proposal store should serialize"),
        )
        .expect("mutated proposal store should write");
        let index = fixture_dispatch_index(&project.project_root, thread_id);
        let preview_input = fixture_project_director_preview_input(
            &project.project_root,
            &proposal.proposal_id,
            &authorization.authorization_id,
            revision,
        );

        let err = preview_project_director_task_plan_for_index_at(&path, &index, &preview_input)
            .expect_err("mismatched back link should reject C4 preview");

        assert!(err.contains("授权回链"), "{err}");

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn project_director_task_plan_blocks_out_of_scope_planned_task() {
        let timestamp_ms = 1_765_300_000_000;
        let dir = test_temp_dir("project-director-task-plan-out-of-scope");
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/c4-out-of-scope");
        let thread_id = "thread-c4-out-of-scope";
        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        let (proposal, authorization, revision) =
            create_active_project_director_authorization_fixture(
                &path,
                &project.project_root,
                thread_id,
                timestamp_ms,
            );
        let index = fixture_dispatch_index(&project.project_root, thread_id);
        let preview_input = fixture_project_director_preview_input(
            &project.project_root,
            &proposal.proposal_id,
            &authorization.authorization_id,
            revision,
        );
        let plan = preview_project_director_task_plan_for_index_at(&path, &index, &preview_input)
            .expect("preview should build deterministic plan");
        let mut planned_task = plan.planned_tasks[0].clone();
        planned_task.scope.allowed_write_scope = vec!["/tmp/c4-outside-write".to_string()];
        planned_task.scope.callable_tool_capabilities = vec!["network_access".to_string()];
        planned_task.scope.required_checks = vec!["npm run deploy".to_string()];
        planned_task.scope.task_package_kind = "unapproved_kind".to_string();
        let prepare_input = fixture_project_director_prepare_input(
            &project.project_root,
            &proposal.proposal_id,
            &authorization.authorization_id,
            revision,
            vec![planned_task],
        );

        let result = prepare_authorized_auto_dispatch_for_index_at(&path, &index, &prepare_input)
            .expect("blocked planned task should record blocked summary");
        let updated = read_json_file(&path);

        assert_eq!(result.plan.blocked_count, 1);
        assert_eq!(result.plan.prepared_dispatch_count, 0);
        assert!(result.plan.blocked_reasons.iter().any(|reason| {
            reason.contains("写入范围超出方案授权")
                || reason.contains("工具超出方案授权")
                || reason.contains("task package kind 超出方案授权")
        }));
        assert!(updated["workflow_node_dispatches"]
            .as_array()
            .map_or(true, Vec::is_empty));
        assert!(updated["audit_events"]
            .as_array()
            .expect("audit events should be array")
            .iter()
            .any(|event| event["event_type"] == "authorized_prepared_dispatch_blocked"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn authorized_prepared_dispatch_needs_binding_without_executable_dispatch() {
        let timestamp_ms = 1_765_300_000_000;
        let dir = test_temp_dir("authorized-prepared-dispatch-needs-binding");
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/c4-needs-binding");
        let thread_id = "thread-c4-needs-binding";
        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        let (proposal, authorization, revision) =
            create_active_project_director_authorization_fixture(
                &path,
                &project.project_root,
                thread_id,
                timestamp_ms,
            );
        let index = fixture_dispatch_index(&project.project_root, thread_id);
        let prepare_input = fixture_project_director_prepare_input(
            &project.project_root,
            &proposal.proposal_id,
            &authorization.authorization_id,
            revision,
            vec![],
        );

        let result = prepare_authorized_auto_dispatch_for_index_at(&path, &index, &prepare_input)
            .expect("missing binding should write setup artifacts but no prepared dispatch");
        let updated = read_json_file(&path);

        assert_eq!(result.plan.needs_binding_count, 1);
        assert_eq!(result.plan.prepared_dispatch_count, 0);
        assert!(result
            .plan
            .blocked_reasons
            .iter()
            .any(|reason| reason.contains("等待绑定会话")));
        assert!(updated["work_items"]
            .as_array()
            .expect("work items should be array")
            .iter()
            .any(|item| item["source_kind"] == "project_director_task_plan"));
        assert!(updated["artifacts"]
            .as_array()
            .expect("artifacts should be array")
            .iter()
            .any(|artifact| artifact["memory_packet_snapshot"].is_object()));
        assert!(updated["workflow_node_dispatches"]
            .as_array()
            .map_or(true, Vec::is_empty));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn authorized_prepared_dispatch_creates_memory_snapshot_and_remains_unexecuted_and_idempotent()
    {
        let timestamp_ms = 1_765_300_000_000;
        let dir = test_temp_dir("authorized-prepared-dispatch-created");
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/c4-prepared-dispatch");
        let thread_id = "thread-c4-prepared";
        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        let (proposal, authorization, revision) =
            create_active_project_director_authorization_fixture(
                &path,
                &project.project_root,
                thread_id,
                timestamp_ms,
            );
        let index = fixture_dispatch_index(&project.project_root, thread_id);
        let workflow_id = default_workflow_id(&project.project_root);
        let node_id = format!("{workflow_id}:node:codex-dev");
        bind_workflow_node_codex_session_for_index_at(
            &path,
            &index,
            &fixture_node_session_bind_request(&project.project_root, &node_id, None, thread_id),
        )
        .expect("node-level binding should write");
        let prepare_input = fixture_project_director_prepare_input(
            &project.project_root,
            &proposal.proposal_id,
            &authorization.authorization_id,
            revision,
            vec![],
        );

        let first = prepare_authorized_auto_dispatch_for_index_at(&path, &index, &prepare_input)
            .expect("active binding should create prepared dispatch");
        let second = prepare_authorized_auto_dispatch_for_index_at(&path, &index, &prepare_input)
            .expect("repeated prepare should be idempotent");
        let updated = read_json_file(&path);
        let dispatches = updated["workflow_node_dispatches"]
            .as_array()
            .expect("dispatches should be array");
        let dispatch = dispatches.first().expect("prepared dispatch should exist");
        let artifact = updated["artifacts"]
            .as_array()
            .expect("artifacts should be array")
            .iter()
            .find(|artifact| artifact["source_kind"] == "project_director_task_plan")
            .expect("task package artifact should exist");

        assert_eq!(first.plan.prepared_dispatch_count, 1);
        assert_eq!(first.plan.needs_binding_count, 0);
        assert_eq!(first.prepared_dispatches.len(), 1);
        assert_eq!(second.plan.prepared_dispatch_count, 1);
        assert_eq!(
            dispatches.len(),
            1,
            "duplicate prepare must not duplicate dispatch"
        );
        assert_eq!(dispatch["state"], "prepared");
        assert_eq!(dispatch["prompt_kind"], "authorized_prepared_auto_dispatch");
        assert_eq!(
            dispatch["plan_authorization_id"],
            authorization.authorization_id
        );
        assert_eq!(dispatch["authorization_check"]["status"], "authorized");
        assert!(dispatch["prompt_preview"]
            .as_str()
            .unwrap_or("")
            .contains("prepared dispatch 只是工作台准备态记录"));
        assert!(dispatch["memory_packet_snapshot_id"].is_string());
        assert!(dispatch["memory_packet_fingerprint"].is_string());
        assert!(dispatch["started_at_ms"].is_null());
        assert!(dispatch["ended_at_ms"].is_null());
        assert!(dispatch["exit_code"].is_null());
        assert!(dispatch["last_message_path"].is_null());
        assert!(dispatch["last_message_summary"].is_null());
        assert!(dispatch["transcript_event_count"].is_null());
        assert!(dispatch["transcript_target_hits"].is_null());
        assert!(artifact["memory_packet_snapshot"].is_object());
        assert_eq!(
            artifact["memory_packet_snapshot"]["schema_version"],
            "task_package_memory_packet_snapshot.v1"
        );
        assert!(updated["audit_events"]
            .as_array()
            .expect("audit events should be array")
            .iter()
            .any(|event| event["event_type"] == "authorized_prepared_dispatch_created"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn worker_structured_report_rejects_missing_evidence_and_ordinary_chat_source() {
        let (dir, path, project, work_item_id, dispatch_id, node_id) =
            setup_c5_worker_report_fixture("c5-worker-report-invalid");
        let mut input = fixture_c5_worker_report_input(
            &project.project_root,
            &work_item_id,
            &dispatch_id,
            &node_id,
        );
        input.evidence_refs.clear();

        let err = record_worker_structured_report_at(&path, &input)
            .expect_err("worker report without evidence should be rejected");
        assert!(err.contains("evidence_refs"), "{err}");

        let mut input = fixture_c5_worker_report_input(
            &project.project_root,
            &work_item_id,
            &dispatch_id,
            &node_id,
        );
        input.source_refs[0].source_kind = "ordinary_chat".to_string();
        let err = record_worker_structured_report_at(&path, &input)
            .expect_err("ordinary chat source should be rejected");
        assert!(err.contains("普通聊天来源"), "{err}");

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn worker_structured_report_records_audit_without_observation_or_formal_memory() {
        let (dir, path, project, work_item_id, dispatch_id, node_id) =
            setup_c5_worker_report_fixture("c5-worker-report-audit-only");
        let input = fixture_c5_worker_report_input(
            &project.project_root,
            &work_item_id,
            &dispatch_id,
            &node_id,
        );

        let output = record_worker_structured_report_at(&path, &input)
            .expect("worker report should write audit event only");
        let updated = read_json_file(&path);
        let snapshot = read_workflow_state_snapshot(&path).expect("snapshot should read");
        let report = snapshot.project_workflows[0]
            .derived_workflow
            .as_ref()
            .expect("derived workflow should exist")
            .subagent_reports
            .iter()
            .find(|report| report.report_id == output.audit_event_id)
            .expect("worker structured report should derive as subagent report");

        assert_eq!(report.acceptance_status, "reported_completed");
        assert!(report
            .warnings
            .contains(&"worker_report_is_not_formal_fact".to_string()));
        assert!(updated["audit_events"]
            .as_array()
            .expect("audit events should be array")
            .iter()
            .any(
                |event| event["event_type"] == "worker_structured_report_recorded"
                    && event["event_id"] == output.audit_event_id
            ));
        assert!(
            !observation_store::sidecar_path(&path)
                .expect("observation sidecar path")
                .exists(),
            "worker report must not automatically create observation store"
        );
        assert!(
            !formal_memory_store::sidecar_path(&path)
                .expect("formal memory sidecar path")
                .exists(),
            "worker report must not create formal memory"
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn project_director_process_fact_confirmation_writes_recorded_observation_only() {
        let (dir, path, project, work_item_id, dispatch_id, node_id) =
            setup_c5_worker_report_fixture("c5-process-fact-confirm");
        let report = record_worker_structured_report_at(
            &path,
            &fixture_c5_worker_report_input(
                &project.project_root,
                &work_item_id,
                &dispatch_id,
                &node_id,
            ),
        )
        .expect("worker report should write");
        let input = fixture_c5_process_fact_decision_input(
            &project.project_root,
            &report.audit_event_id,
            &dispatch_id,
            "confirm_process_fact",
        );

        let output = record_project_director_process_fact_decision_at(&path, &input)
            .expect("project director should confirm low-risk process fact");
        let updated = read_json_file(&path);
        let observation_store = observation_store::load_store(&path, "2026-06-04T00:00:01Z")
            .expect("observation store should load");
        let derived = output.snapshot.project_workflows[0]
            .derived_workflow
            .as_ref()
            .expect("derived workflow should exist");

        assert_eq!(output.observations.len(), 1);
        assert_eq!(output.observations[0].observation_type, "process_fact");
        assert_eq!(output.observations[0].status, ObservationStatus::Recorded);
        assert_eq!(output.observations[0].generated_by_role, "project_director");
        assert!(output.message.contains("仍不是正式记忆"));
        assert_eq!(observation_store.observations.len(), 1);
        assert!(updated["reviews"]
            .as_array()
            .expect("reviews should be array")
            .iter()
            .any(|review| review["decision"] == "confirm_process_fact"
                && review["report_id"] == report.audit_event_id
                && review["warnings"]
                    .as_array()
                    .is_some_and(|warnings| warnings.iter().any(
                        |warning| warning == "process_fact_observation_is_not_formal_memory"
                    ))));
        assert!(derived.review_results.iter().any(|review| {
            review.report_id.as_deref() == Some(report.audit_event_id.as_str())
                && review.result == "process_fact_confirmed"
                && review.observation_ids.len() == 1
        }));
        assert!(
            !formal_memory_store::sidecar_path(&path)
                .expect("formal memory sidecar path")
                .exists(),
            "process fact observation must not create formal memory"
        );
        assert!(
            !memory_candidate_store::sidecar_path(&path)
                .expect("candidate sidecar path")
                .exists(),
            "process fact confirmation must not automatically create memory candidate"
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn project_director_process_fact_decision_rejects_wrong_actor_and_unsafe_facts() {
        let (dir, path, project, work_item_id, dispatch_id, node_id) =
            setup_c5_worker_report_fixture("c5-process-fact-boundaries");
        let report = record_worker_structured_report_at(
            &path,
            &fixture_c5_worker_report_input(
                &project.project_root,
                &work_item_id,
                &dispatch_id,
                &node_id,
            ),
        )
        .expect("worker report should write");

        let mut wrong_actor = fixture_c5_process_fact_decision_input(
            &project.project_root,
            &report.audit_event_id,
            &dispatch_id,
            "confirm_process_fact",
        );
        wrong_actor.actor_role = "secretary".to_string();
        let err = record_project_director_process_fact_decision_at(&path, &wrong_actor)
            .expect_err("secretary must not confirm process fact");
        assert!(err.contains("只有项目主管"), "{err}");

        let mut high_risk = fixture_c5_process_fact_decision_input(
            &project.project_root,
            &report.audit_event_id,
            &dispatch_id,
            "confirm_process_fact",
        );
        high_risk.accepted_facts[0].risk_level = "high".to_string();
        let err = record_project_director_process_fact_decision_at(&path, &high_risk)
            .expect_err("high risk process fact should require higher confirmation");
        assert!(err.contains("high / medium risk"), "{err}");

        let mut secret = fixture_c5_process_fact_decision_input(
            &project.project_root,
            &report.audit_event_id,
            &dispatch_id,
            "confirm_process_fact",
        );
        secret.accepted_facts[0].sensitive_level = "secret".to_string();
        let err = record_project_director_process_fact_decision_at(&path, &secret)
            .expect_err("secret process fact should require user confirmation");
        assert!(err.contains("secret / sensitive"), "{err}");

        let mut cross_project = fixture_c5_process_fact_decision_input(
            &project.project_root,
            &report.audit_event_id,
            &dispatch_id,
            "confirm_process_fact",
        );
        cross_project.accepted_facts[0].scope.project_id = Some("project:other".to_string());
        let err = record_project_director_process_fact_decision_at(&path, &cross_project)
            .expect_err("cross-project process fact should be rejected");
        assert!(err.contains("cross-project"), "{err}");

        assert!(
            !observation_store::sidecar_path(&path)
                .expect("observation sidecar path")
                .exists(),
            "rejected C5 decisions must not create observations"
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn process_fact_duplicate_is_rejected_and_rework_does_not_write_observation() {
        let (dir, path, project, work_item_id, dispatch_id, node_id) =
            setup_c5_worker_report_fixture("c5-process-fact-duplicate");
        let report = record_worker_structured_report_at(
            &path,
            &fixture_c5_worker_report_input(
                &project.project_root,
                &work_item_id,
                &dispatch_id,
                &node_id,
            ),
        )
        .expect("worker report should write");
        let confirm = fixture_c5_process_fact_decision_input(
            &project.project_root,
            &report.audit_event_id,
            &dispatch_id,
            "confirm_process_fact",
        );
        record_project_director_process_fact_decision_at(&path, &confirm)
            .expect("first process fact confirmation should write");

        let duplicate = record_project_director_process_fact_decision_at(&path, &confirm)
            .expect_err("duplicate process fact confirmation should be rejected");
        assert!(duplicate.contains("process_fact_duplicate"), "{duplicate}");

        let (
            rework_dir,
            rework_path,
            rework_project,
            rework_item_id,
            rework_dispatch_id,
            rework_node_id,
        ) = setup_c5_worker_report_fixture("c5-process-fact-rework");
        let rework_report = record_worker_structured_report_at(
            &rework_path,
            &fixture_c5_worker_report_input(
                &rework_project.project_root,
                &rework_item_id,
                &rework_dispatch_id,
                &rework_node_id,
            ),
        )
        .expect("worker report should write for rework");
        let rework = record_project_director_process_fact_decision_at(
            &rework_path,
            &fixture_c5_process_fact_decision_input(
                &rework_project.project_root,
                &rework_report.audit_event_id,
                &rework_dispatch_id,
                "request_rework",
            ),
        )
        .expect("rework decision should write review only");
        let rework_snapshot =
            read_workflow_state_snapshot(&rework_path).expect("rework snapshot should read");
        let rework_derived = rework_snapshot.project_workflows[0]
            .derived_workflow
            .as_ref()
            .expect("derived workflow should exist");

        assert!(rework.observations.is_empty());
        assert!(
            !observation_store::sidecar_path(&rework_path)
                .expect("observation sidecar path")
                .exists(),
            "rework decision must not create process fact observation"
        );
        assert!(rework_derived.review_results.iter().any(|review| {
            review.report_id.as_deref() == Some(rework_report.audit_event_id.as_str())
                && review.result == "rework_requested"
                && review.observation_ids.is_empty()
        }));

        let _ = fs::remove_dir_all(dir);
        let _ = fs::remove_dir_all(rework_dir);
    }

    #[test]
    fn global_final_result_review_rejects_missing_c2_and_c3_prerequisites() {
        let dir = test_temp_dir("c6-final-review-missing-c2");
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/c6-final-review-missing-c2");
        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        let missing_c2 = fixture_global_final_result_review_input(
            &project.project_root,
            "proposal:missing",
            "plan-auth:missing",
            "process-fact:missing",
            "accepted",
        );

        let err = record_global_final_result_review_at(&path, &missing_c2)
            .expect_err("missing C2 proposal should reject final review");
        assert!(err.contains("找不到 C2"), "{err}");

        let timestamp_ms = 1_765_600_000_000;
        let confirmed =
            create_confirmed_proposal_for_global_review(&path, &project.project_root, timestamp_ms);
        let missing_c3 = fixture_global_final_result_review_input(
            &project.project_root,
            &confirmed.0.proposal_id,
            &confirmed.1.authorization_id,
            "process-fact:missing",
            "accepted",
        );
        let err = record_global_final_result_review_at(&path, &missing_c3)
            .expect_err("missing active C3 authorization should reject final review");
        assert!(err.contains("C3") || err.contains("active"), "{err}");

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn global_final_result_review_records_review_without_memory_or_user_acceptance() {
        let (dir, path, project, proposal, authorization, fact_id) =
            setup_c6_complete_fixture("c6-final-review-records");
        let input = fixture_global_final_result_review_input(
            &project.project_root,
            &proposal.proposal_id,
            &authorization.authorization_id,
            &fact_id,
            "accepted",
        );

        let output = record_global_final_result_review_at(&path, &input)
            .expect("global director should record accepted final review");
        let updated = read_json_file(&path);
        let derived = output.snapshot.project_workflows[0]
            .derived_workflow
            .as_ref()
            .expect("derived workflow should exist");

        assert!(updated["reviews"]
            .as_array()
            .expect("reviews should be array")
            .iter()
            .any(|review| review["review_target"] == "global_final_result"
                && review["reviewer_role"] == "global_director"
                && review["decision"] == "accepted"));
        assert!(updated["audit_events"]
            .as_array()
            .expect("audit events should be array")
            .iter()
            .any(|event| event["event_type"] == "global_final_result_review_recorded"));
        assert_eq!(derived.result_summary.final_review_status, "accepted");
        assert_eq!(derived.result_summary.user_decision_status, "pending");
        assert!(
            !formal_memory_store::sidecar_path(&path)
                .expect("formal memory sidecar path")
                .exists(),
            "final review must not write formal memory"
        );
        assert!(
            !memory_candidate_store::sidecar_path(&path)
                .expect("candidate sidecar path")
                .exists(),
            "final review must not generate memory candidate"
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn global_final_result_review_rejects_wrong_actor() {
        let (dir, path, project, proposal, authorization, fact_id) =
            setup_c6_complete_fixture("c6-final-review-wrong-actor");
        let mut input = fixture_global_final_result_review_input(
            &project.project_root,
            &proposal.proposal_id,
            &authorization.authorization_id,
            &fact_id,
            "accepted",
        );
        input.actor_role = "project_director".to_string();

        let err = record_global_final_result_review_at(&path, &input)
            .expect_err("project director must not record global final review");
        assert!(err.contains("只有全局主管"), "{err}");

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn user_result_decision_requires_user_and_does_not_write_memory() {
        let (dir, path, project, proposal, authorization, fact_id) =
            setup_c6_complete_fixture("c6-user-result-decision");
        record_global_final_result_review_at(
            &path,
            &fixture_global_final_result_review_input(
                &project.project_root,
                &proposal.proposal_id,
                &authorization.authorization_id,
                &fact_id,
                "accepted",
            ),
        )
        .expect("global final review should write");
        let actual_review_id = read_json_file(&path)["reviews"]
            .as_array()
            .expect("reviews should be array")
            .iter()
            .rev()
            .find(|review| review["review_target"] == "global_final_result")
            .and_then(|review| optional_string_from(review, "review_id"))
            .expect("global final review id should exist");
        let mut wrong_actor = fixture_user_result_decision_input(
            &project.project_root,
            &actual_review_id,
            "accept_result",
        );
        wrong_actor.actor_role = "secretary".to_string();
        let err = record_user_result_decision_at(&path, &wrong_actor)
            .expect_err("secretary must not accept result for user");
        assert!(err.contains("只有用户"), "{err}");

        let output = record_user_result_decision_at(
            &path,
            &fixture_user_result_decision_input(
                &project.project_root,
                &actual_review_id,
                "accept_result",
            ),
        )
        .expect("user should accept accepted final review");
        let derived = output.snapshot.project_workflows[0]
            .derived_workflow
            .as_ref()
            .expect("derived workflow should exist");

        assert_eq!(derived.result_summary.user_decision_status, "accept_result");
        assert!(
            !formal_memory_store::sidecar_path(&path)
                .expect("formal memory sidecar path")
                .exists(),
            "user decision must not write formal memory"
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn stage_c_acceptance_summary_records_gates_and_deferred_items() {
        let (dir, path, project, proposal, authorization, fact_id) =
            setup_c6_complete_fixture("c6-stage-acceptance-summary");
        record_global_final_result_review_at(
            &path,
            &fixture_global_final_result_review_input(
                &project.project_root,
                &proposal.proposal_id,
                &authorization.authorization_id,
                &fact_id,
                "accepted",
            ),
        )
        .expect("global final review should write");
        let review_id = read_json_file(&path)["reviews"]
            .as_array()
            .expect("reviews should be array")
            .iter()
            .rev()
            .find(|review| review["review_target"] == "global_final_result")
            .and_then(|review| optional_string_from(review, "review_id"))
            .expect("global final review id should exist");
        record_user_result_decision_at(
            &path,
            &fixture_user_result_decision_input(&project.project_root, &review_id, "accept_result"),
        )
        .expect("user decision should write");

        let output = generate_stage_c_acceptance_summary_at(
            &path,
            &GenerateStageCAcceptanceSummaryInput {
                project_root: project.project_root.clone(),
                project_id: project_id(&project.project_root),
                workflow_id: default_workflow_id(&project.project_root),
                expected_workflow_revision: None,
            },
        )
        .expect("stage C summary should write artifact");
        let updated = read_json_file(&path);
        let derived = output.snapshot.project_workflows[0]
            .derived_workflow
            .as_ref()
            .expect("derived workflow should exist");

        assert!(updated["artifacts"]
            .as_array()
            .expect("artifacts should be array")
            .iter()
            .any(
                |artifact| artifact["artifact_type"] == "stage_c_acceptance_summary"
                    && artifact["stage_c_acceptance_summary"]["accepted_as_stage_c_complete"]
                        == true
            ));
        assert!(
            derived
                .result_summary
                .stage_c_acceptance
                .accepted_as_stage_c_complete
        );
        assert!(derived
            .result_summary
            .stage_c_acceptance
            .gates
            .iter()
            .any(|gate| gate.status == "deferred"));
        assert!(derived
            .result_summary
            .deferred_items
            .iter()
            .any(|item| item.contains("真实 worker")));

        let _ = fs::remove_dir_all(dir);
    }

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

    #[test]
    fn memory_lint_blocks_conflicting_candidate_adoption() {
        let dir = test_temp_dir("memory-lint-conflicting-adoption");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/memory-lint-conflicting-adoption-project";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        create_formal_memory_with_source(
            &path,
            project_root,
            "接口缓存必须启用",
            "source:lint:conflict:formal",
            "evidence",
            "2026-06-04T02:00:00Z",
            "write-lint-conflict-formal",
        );
        let candidate =
            create_confirmed_candidate_with_claim(&path, project_root, "接口缓存禁止启用");

        let err = adopt_memory_candidate_to_formal_memory_at(
            &path,
            &fixture_adopt_memory_candidate_input(
                project_root,
                candidate.candidate_key,
                Some(2),
                Some(1),
            ),
            "2026-06-04T02:00:02Z",
            "write-lint-conflict-adoption",
            "write-lint-conflict-formal-adoption",
        )
        .unwrap_err();

        assert!(err.contains("memory_lint_blocking_findings"));
        let lint_store = memory_lint_store::load_store(&path, "2026-06-04T02:00:03Z")
            .expect("lint store should load");
        assert_eq!(lint_store.findings.len(), 1);
        assert_eq!(
            lint_store.findings[0].finding_type,
            MemoryLintFindingType::CandidateConflictsWithActiveMemory
        );
        assert_eq!(
            lint_store.findings[0].severity,
            MemoryLintFindingSeverity::Blocking
        );
        assert_eq!(lint_store.runs[0].status, MemoryLintRunStatus::Blocked);
        let formal_store = formal_memory_store::load_store(&path, "2026-06-04T02:00:03Z")
            .expect("formal store should load");
        assert_eq!(formal_store.records.len(), 1);
        assert_eq!(formal_store.versions.len(), 1);
        assert_eq!(
            formal_store
                .audit_events
                .iter()
                .filter(|event| event.event_type == "memory_candidate_adopted_to_formal_memory")
                .count(),
            0
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn memory_lint_allows_non_conflicting_candidate_adoption() {
        let dir = test_temp_dir("memory-lint-non-conflicting-adoption");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/memory-lint-non-conflicting-adoption-project";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        create_formal_memory_with_source(
            &path,
            project_root,
            "接口缓存必须启用",
            "source:lint:non-conflict:formal",
            "evidence",
            "2026-06-04T02:10:00Z",
            "write-lint-non-conflict-formal",
        );
        let candidate =
            create_confirmed_candidate_with_claim(&path, project_root, "接口文档需要保留验收步骤");

        let adopted = adopt_memory_candidate_to_formal_memory_at(
            &path,
            &fixture_adopt_memory_candidate_input(
                project_root,
                candidate.candidate_key,
                Some(2),
                Some(1),
            ),
            "2026-06-04T02:10:02Z",
            "write-lint-non-conflict-adoption",
            "write-lint-non-conflict-formal-adoption",
        )
        .expect("non-conflicting candidate should adopt");

        assert_eq!(
            adopted.candidate_status,
            MemoryLifecycleStatus::CandidateConfirmed
        );
        let lint_store = memory_lint_store::load_store(&path, "2026-06-04T02:10:03Z")
            .expect("lint store should load");
        assert_eq!(lint_store.runs[0].status, MemoryLintRunStatus::Succeeded);
        assert_eq!(lint_store.findings.len(), 0);
        let formal_store = formal_memory_store::load_store(&path, "2026-06-04T02:10:03Z")
            .expect("formal store should load");
        assert_eq!(formal_store.records.len(), 2);

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn memory_lint_duplicate_claim_generates_finding() {
        let dir = test_temp_dir("memory-lint-duplicate-claim");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/memory-lint-duplicate-claim-project";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        create_formal_memory_with_source(
            &path,
            project_root,
            "cache interface should stay enabled",
            "source:lint:duplicate:001",
            "evidence",
            "2026-06-04T02:20:00Z",
            "write-lint-duplicate-001",
        );
        create_formal_memory_with_source(
            &path,
            project_root,
            "cache interface should stay enabled now",
            "source:lint:duplicate:002",
            "evidence",
            "2026-06-04T02:20:01Z",
            "write-lint-duplicate-002",
        );
        let output = run_memory_lint_at(
            &path,
            &fixture_memory_lint_run_input(project_root, MemoryLintRunIntent::MaintenancePreview),
            "2026-06-04T02:20:02Z",
            "write-lint-duplicate-run",
        )
        .expect("lint run should succeed");

        let duplicate = output
            .new_findings
            .iter()
            .find(|finding| finding.finding_type == MemoryLintFindingType::DuplicateClaim)
            .expect("duplicate claim finding should be present");
        assert_eq!(duplicate.severity, MemoryLintFindingSeverity::NeedsReview);
        assert!(duplicate.summary.contains("0.80"));
        assert!(output
            .new_findings
            .iter()
            .any(|finding| finding.finding_type == MemoryLintFindingType::DerivedIndexStale));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn memory_lint_authority_superseded_does_not_mutate_formal_memory() {
        let dir = test_temp_dir("memory-lint-authority-superseded");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/memory-lint-authority-superseded-project";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        create_formal_memory_with_source(
            &path,
            project_root,
            "接口缓存策略需要保留验收记录",
            "source:lint:authority:old",
            "evidence",
            "2026-06-04T02:30:00Z",
            "write-lint-authority-old",
        );
        create_formal_memory_with_source(
            &path,
            project_root,
            "接口缓存策略需要保留验收记录",
            "source:lint:authority:new",
            "user_confirmed",
            "2026-06-04T02:30:01Z",
            "write-lint-authority-new",
        );
        let before = formal_memory_store::load_store(&path, "2026-06-04T02:30:02Z")
            .expect("formal store should load");

        let output = run_memory_lint_at(
            &path,
            &fixture_memory_lint_run_input(project_root, MemoryLintRunIntent::MaintenancePreview),
            "2026-06-04T02:30:02Z",
            "write-lint-authority-run",
        )
        .expect("lint run should succeed");
        let after = formal_memory_store::load_store(&path, "2026-06-04T02:30:03Z")
            .expect("formal store should load");

        assert!(output
            .new_findings
            .iter()
            .any(|finding| finding.finding_type == MemoryLintFindingType::AuthoritySuperseded));
        assert_eq!(after.records, before.records);
        assert_eq!(after.versions, before.versions);
        assert!(after
            .records
            .iter()
            .all(|record| record.status == MemoryLifecycleStatus::MemoryActive));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn memory_lint_revoked_source_excludes_task_packet_memory() {
        let dir = test_temp_dir("memory-lint-revoked-source");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/memory-lint-revoked-source-project";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        create_formal_memory_with_source(
            &path,
            project_root,
            "接口权限验收需要保留",
            "source:lint:revoked",
            "evidence",
            "2026-06-04T02:40:00Z",
            "write-lint-revoked-formal",
        );
        let mut input =
            fixture_memory_lint_run_input(project_root, MemoryLintRunIntent::TaskPacketGuard);
        input.revoked_source_ids = vec!["source:lint:revoked".to_string()];
        run_memory_lint_at(
            &path,
            &input,
            "2026-06-04T02:40:01Z",
            "write-lint-revoked-run",
        )
        .expect("lint run should succeed");

        let output = preview_task_memory_packet_at(
            &path,
            &fixture_task_memory_packet_input(project_root, "接口 权限 验收"),
            "2026-06-04T02:40:02Z",
        )
        .expect("task memory packet should preview");

        assert_eq!(output.preview.included_memories.len(), 0);
        assert_eq!(
            excluded_reason_count(&output, TaskMemoryPacketExclusionReason::Conflicted),
            1
        );
        assert!(output
            .preview
            .excluded_items
            .iter()
            .any(|item| item.detail.contains("memory lint open blocking finding")));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn memory_lint_open_blocking_finding_excludes_task_packet_memory() {
        let dir = test_temp_dir("memory-lint-blocking-packet");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/memory-lint-blocking-packet-project";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        create_formal_memory_with_source(
            &path,
            project_root,
            "接口缓存必须启用",
            "source:lint:blocking:formal",
            "evidence",
            "2026-06-04T02:50:00Z",
            "write-lint-blocking-formal",
        );
        let candidate =
            create_confirmed_candidate_with_claim(&path, project_root, "接口缓存禁止启用");
        let mut input = fixture_memory_lint_run_input(
            project_root,
            MemoryLintRunIntent::CandidateAdoptionGuard,
        );
        input.candidate_key = Some(candidate.candidate_key);
        run_memory_lint_at(
            &path,
            &input,
            "2026-06-04T02:50:02Z",
            "write-lint-blocking-run",
        )
        .expect("lint run should write blocking finding");

        let output = preview_task_memory_packet_at(
            &path,
            &fixture_task_memory_packet_input(project_root, "接口 缓存"),
            "2026-06-04T02:50:03Z",
        )
        .expect("task memory packet should preview");

        assert_eq!(output.preview.included_memories.len(), 0);
        assert_eq!(
            excluded_reason_count(&output, TaskMemoryPacketExclusionReason::Conflicted),
            1
        );
        assert!(output
            .preview
            .warnings
            .contains(&"memory_lint_blocking_findings_excluded".to_string()));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn memory_lint_maintenance_run_is_readonly_for_formal_memory() {
        let dir = test_temp_dir("memory-lint-maintenance-readonly");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/memory-lint-maintenance-readonly-project";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        create_formal_memory_with_source(
            &path,
            project_root,
            "cache interface should stay enabled",
            "source:lint:readonly:001",
            "evidence",
            "2026-06-04T03:00:00Z",
            "write-lint-readonly-001",
        );
        create_formal_memory_with_source(
            &path,
            project_root,
            "cache interface should stay enabled now",
            "source:lint:readonly:002",
            "evidence",
            "2026-06-04T03:00:01Z",
            "write-lint-readonly-002",
        );
        let before = formal_memory_store::load_store(&path, "2026-06-04T03:00:02Z")
            .expect("formal store should load");

        let output = run_memory_lint_at(
            &path,
            &fixture_memory_lint_run_input(project_root, MemoryLintRunIntent::MaintenancePreview),
            "2026-06-04T03:00:02Z",
            "write-lint-readonly-run",
        )
        .expect("lint run should succeed");
        let after = formal_memory_store::load_store(&path, "2026-06-04T03:00:03Z")
            .expect("formal store should load");
        let summary = memory_lint_store::summarize_store(&output.store);

        assert_eq!(after.records, before.records);
        assert_eq!(after.versions, before.versions);
        assert_eq!(summary.open_count, 2);
        assert_eq!(summary.needs_review_count, 1);
        assert_eq!(summary.info_count, 1);
        assert!(summary.recent_maintenance_report.is_some());
        assert!(summary.display_text.contains("不会自动修改正式记忆"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn memory_lint_damaged_json_is_not_overwritten() {
        let dir = test_temp_dir("memory-lint-damaged-json");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/memory-lint-damaged-json-project";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        let sidecar = memory_lint_store::sidecar_path(&path).expect("lint sidecar path");
        fs::write(&sidecar, "{ damaged json").expect("damaged lint sidecar should write");

        let err = run_memory_lint_at(
            &path,
            &fixture_memory_lint_run_input(project_root, MemoryLintRunIntent::MaintenancePreview),
            "2026-06-04T03:10:00Z",
            "write-lint-damaged-run",
        )
        .unwrap_err();

        assert!(err.contains("memory lint sidecar JSON 损坏"));
        assert_eq!(fs::read_to_string(&sidecar).unwrap(), "{ damaged json");

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn memory_lint_revision_conflict_is_rejected() {
        let dir = test_temp_dir("memory-lint-revision-conflict");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/memory-lint-revision-conflict-project";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        run_memory_lint_at(
            &path,
            &fixture_memory_lint_run_input(project_root, MemoryLintRunIntent::MaintenancePreview),
            "2026-06-04T03:20:00Z",
            "write-lint-revision-first",
        )
        .expect("first lint run should write store");
        let mut stale =
            fixture_memory_lint_run_input(project_root, MemoryLintRunIntent::MaintenancePreview);
        stale.expected_lint_store_revision = Some(0);

        let err = run_memory_lint_at(
            &path,
            &stale,
            "2026-06-04T03:20:01Z",
            "write-lint-revision-stale",
        )
        .unwrap_err();

        assert!(err.contains("memory_lint_store_conflict"));
        let store = memory_lint_store::load_store(&path, "2026-06-04T03:20:02Z")
            .expect("lint store should load");
        assert_eq!(store.revision, 1);
        assert_eq!(store.runs.len(), 1);

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn memory_maintenance_run_reports_source_secret_and_index_findings_readonly() {
        let dir = test_temp_dir("memory-maintenance-source-secret-index");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/memory-maintenance-source-secret-index";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        create_formal_memory_with_source(
            &path,
            project_root,
            "缺来源记忆不能召回",
            "source:m11:missing",
            "evidence",
            "2026-06-05T11:00:00Z",
            "write-m11-missing-formal",
        );
        create_formal_memory_for_task(
            &path,
            project_root,
            "secret token 不能外发",
            "正文包含 password 和 secret token，仅用于维护扫描测试。",
            "2026-06-05T11:00:01Z",
            "write-m11-secret-formal",
        );
        mutate_formal_store(&path, |store| {
            store.records[1].source_refs[0].source_id = Some("source:m11:secret".to_string());
        });
        mutate_formal_store(&path, |store| {
            store.records[0].source_refs = vec![];
            store.records[1].source_refs[0].sensitive_level = "secret".to_string();
            store.records[1].scope.model_export_policy = "local_only".to_string();
        });
        let before = formal_memory_store::load_store(&path, "2026-06-05T11:00:02Z")
            .expect("formal store should load");

        let output = run_memory_lint_at(
            &path,
            &fixture_memory_lint_run_input(project_root, MemoryLintRunIntent::MaintenanceRun),
            "2026-06-05T11:00:02Z",
            "write-m11-maintenance-run",
        )
        .expect("maintenance run should succeed");
        let after = formal_memory_store::load_store(&path, "2026-06-05T11:00:03Z")
            .expect("formal store should load");

        assert_eq!(after.records, before.records);
        assert_eq!(after.versions, before.versions);
        assert!(output.run.report_id.is_some());
        let report = output
            .report
            .as_ref()
            .expect("maintenance report should exist");
        assert_eq!(output.store.maintenance_reports.len(), 1);
        assert!(report.display_text.contains("维护任务只生成 finding"));
        assert!(output
            .new_findings
            .iter()
            .any(
                |finding| finding.finding_type == MemoryLintFindingType::MissingSource
                    && finding.severity == MemoryLintFindingSeverity::Blocking
            ));
        assert!(output
            .new_findings
            .iter()
            .any(
                |finding| finding.finding_type == MemoryLintFindingType::SensitiveExportRisk
                    && finding.severity == MemoryLintFindingSeverity::Blocking
            ));
        assert!(output
            .new_findings
            .iter()
            .any(
                |finding| finding.finding_type == MemoryLintFindingType::PrivateSourceRisk
                    && finding.severity == MemoryLintFindingSeverity::NeedsReview
            ));
        assert!(output
            .new_findings
            .iter()
            .any(
                |finding| finding.finding_type == MemoryLintFindingType::DerivedIndexStale
                    && finding.severity == MemoryLintFindingSeverity::Info
            ));
        assert!(report.check_summaries.iter().any(|check| check.check_kind
            == MemoryMaintenanceCheckKind::SourceIntegrity
            && check.blocking_count > 0));
        assert!(report.check_summaries.iter().any(|check| check.check_kind
            == MemoryMaintenanceCheckKind::SensitiveExportRisk
            && check.finding_count > 0));

        let packet = preview_task_memory_packet_at(
            &path,
            &fixture_task_memory_packet_input(project_root, "secret token 缺来源"),
            "2026-06-05T11:00:04Z",
        )
        .expect("task packet should preview");
        assert_eq!(
            excluded_reason_count(&packet, TaskMemoryPacketExclusionReason::Conflicted),
            2
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn memory_maintenance_run_reports_entity_drift_and_relation_revoked_readonly() {
        let dir = test_temp_dir("memory-maintenance-entity-relation-drift");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/memory-maintenance-entity-relation-drift";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        create_formal_memory_with_source(
            &path,
            project_root,
            "实体漂移维护测试",
            "source:m11:entity",
            "evidence",
            "2026-06-05T11:10:00Z",
            "write-m11-entity-formal",
        );
        memory_entity_relation_store::with_locked_store(
            &path,
            "2026-06-05T11:10:01Z",
            "write-m11-entity-relation-store",
            |store| {
                store.project_id = Some(project_id(project_root));
                store.workflow_id = Some(default_workflow_id(project_root));
                store.merge_candidates.push(MemoryEntityMergeCandidate {
                    merge_candidate_id: "merge-candidate:m11:codex".to_string(),
                    left_entity_candidate_id: "entity-candidate:left".to_string(),
                    right_entity_candidate_id: "entity-candidate:right".to_string(),
                    left_label: "Codex CLI".to_string(),
                    right_label: "codex tool".to_string(),
                    normalized_key: "codex".to_string(),
                    source_kind: MemoryRelationSourceKind::SimilarityHit,
                    status: MemoryRelationStatus::Candidate,
                    requires_user_confirmation: true,
                    reason: "alias / dedupe 候选需要人工复核。".to_string(),
                    created_at: "2026-06-05T11:10:01Z".to_string(),
                    warnings: vec![],
                });
                store.relations.push(MemoryRelation {
                    relation_id: "relation:m11:revoked".to_string(),
                    relation_kind: MemoryRelationKind::Semantic,
                    subject_entity_id: "entity:codex".to_string(),
                    object_entity_id: "entity:task".to_string(),
                    subject_label: "Codex CLI".to_string(),
                    object_label: "任务包".to_string(),
                    predicate: "explains".to_string(),
                    source_kind: MemoryRelationSourceKind::Manual,
                    source_refs: vec![MemoryRelationSource {
                        source_kind: MemoryRelationSourceKind::Manual,
                        source_id: Some("source:relation:revoked".to_string()),
                        source_path: Some("docs/relation.md".to_string()),
                        source_title: Some("关系来源".to_string()),
                        authority_level: "evidence".to_string(),
                        sensitive_level: "project".to_string(),
                    }],
                    status: MemoryRelationStatus::Confirmed,
                    confirmed_by: "project_director".to_string(),
                    confirmation_role: "project_director".to_string(),
                    confirmation_reason: "测试关系来源撤回。".to_string(),
                    created_at: "2026-06-05T11:10:01Z".to_string(),
                    updated_at: "2026-06-05T11:10:01Z".to_string(),
                    warnings: vec![],
                });
                store.revision += 1;
                Ok(())
            },
        )
        .expect("entity relation store should write test fixture");
        let entity_before = memory_entity_relation_store::load_store(&path, "2026-06-05T11:10:02Z")
            .expect("entity relation store should load");

        let mut input =
            fixture_memory_lint_run_input(project_root, MemoryLintRunIntent::MaintenanceRun);
        input.revoked_source_ids = vec!["source:relation:revoked".to_string()];
        let output = run_memory_lint_at(
            &path,
            &input,
            "2026-06-05T11:10:02Z",
            "write-m11-entity-maintenance-run",
        )
        .expect("maintenance run should succeed");
        let entity_after = memory_entity_relation_store::load_store(&path, "2026-06-05T11:10:03Z")
            .expect("entity relation store should load");

        assert_eq!(entity_after, entity_before);
        assert!(output
            .new_findings
            .iter()
            .any(
                |finding| finding.finding_type == MemoryLintFindingType::EntityDrift
                    && finding.severity == MemoryLintFindingSeverity::NeedsReview
            ));
        assert!(output
            .new_findings
            .iter()
            .any(
                |finding| finding.finding_type == MemoryLintFindingType::RelationSourceRevoked
                    && finding.severity == MemoryLintFindingSeverity::NeedsReview
            ));
        let report = output
            .report
            .as_ref()
            .expect("maintenance report should exist");
        assert!(report.check_summaries.iter().any(|check| check.check_kind
            == MemoryMaintenanceCheckKind::EntityRelationDrift
            && check.needs_review_count > 0));
        assert!(report.check_summaries.iter().any(|check| check.check_kind
            == MemoryMaintenanceCheckKind::PermissionRevocation
            && check.needs_review_count > 0));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn memory_maintenance_run_reports_mature_pattern_signal_without_promoting_memory() {
        let dir = test_temp_dir("memory-maintenance-mature-pattern");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/memory-maintenance-mature-pattern";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        for (index, claim) in ["重复验收模式 A", "重复验收模式 B", "重复验收模式 C"]
            .iter()
            .enumerate()
        {
            let mut input = fixture_bound_memory_candidate_input(project_root);
            input.claim = claim.to_string();
            input.body = format!("{claim} 的候选说明。");
            let created = memory_candidate_store::create_candidate(
                &path,
                &input,
                &format!("2026-06-05T11:20:0{index}Z"),
                &format!("write-m11-mature-candidate-{index}"),
            )
            .expect("memory candidate should be created");
            memory_candidate_store::record_decision(
                &path,
                &RecordMemoryCandidateDecisionInput {
                    project_root: project_root.to_string(),
                    candidate_key: created.candidate.candidate_key,
                    requested_status: MemoryLifecycleStatus::CandidateConfirmed,
                    reason: "确认保留候选；等待后续成熟模式人工复核。".to_string(),
                    actor_id: "project_director".to_string(),
                    actor_role: "project_director".to_string(),
                    expected_store_revision: Some(created.store_revision),
                },
                &format!("2026-06-05T11:20:1{index}Z"),
                &format!("write-m11-mature-candidate-confirm-{index}"),
            )
            .expect("memory candidate should be confirmed");
        }
        let formal_before = formal_memory_store::load_store(&path, "2026-06-05T11:20:00Z")
            .expect("formal store should load");

        let output = run_memory_lint_at(
            &path,
            &fixture_memory_lint_run_input(project_root, MemoryLintRunIntent::MaintenanceRun),
            "2026-06-05T11:20:01Z",
            "write-m11-mature-pattern-run",
        )
        .expect("maintenance run should succeed");
        let formal_after = formal_memory_store::load_store(&path, "2026-06-05T11:20:02Z")
            .expect("formal store should load");

        assert_eq!(formal_after.records, formal_before.records);
        assert_eq!(formal_after.versions, formal_before.versions);
        assert_eq!(formal_after.revision, formal_before.revision);
        assert!(output
            .new_findings
            .iter()
            .any(
                |finding| finding.finding_type == MemoryLintFindingType::MaturePatternSignal
                    && finding.severity == MemoryLintFindingSeverity::NeedsReview
                    && finding.summary.contains("不会自动成为规则")
            ));
        assert!(output
            .report
            .as_ref()
            .expect("maintenance report should exist")
            .check_summaries
            .iter()
            .any(
                |check| check.check_kind == MemoryMaintenanceCheckKind::MaturePatternSignal
                    && check.needs_review_count > 0
            ));

        let _ = fs::remove_dir_all(dir);
    }

    fn fixture_m12_preview_input(project_root: &str) -> PreviewMaturePatternsInput {
        PreviewMaturePatternsInput {
            project_root: project_root.to_string(),
            project_id: Some(project_id(project_root)),
            workflow_id: Some(default_workflow_id(project_root)),
        }
    }

    fn prepare_m12_repeated_candidate_fixture(path: &Path, project_root: &str) {
        for claim in [
            "repeat review failure requires checklist step before release alpha",
            "repeat review failure requires checklist step before release beta",
            "repeat review failure requires checklist step before release gamma",
        ] {
            create_confirmed_candidate_with_claim(path, project_root, claim);
        }
        run_memory_lint_at(
            path,
            &fixture_memory_lint_run_input(project_root, MemoryLintRunIntent::MaintenanceRun),
            "2026-06-05T12:00:00Z",
            "write-m12-maintenance-signal",
        )
        .expect("maintenance run should create mature pattern signal");
    }

    #[test]
    fn mature_pattern_preview_derives_candidates_and_memory_cluster_reports_readonly() {
        let dir = test_temp_dir("mature-pattern-preview-readonly");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/mature-pattern-preview-readonly";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        prepare_m12_repeated_candidate_fixture(&path, project_root);

        let preview = mature_pattern_governance::preview_mature_patterns(
            &path,
            &fixture_m12_preview_input(project_root),
            "2026-06-05T12:00:01Z",
        )
        .expect("mature pattern preview should build");

        assert!(preview
            .mature_pattern_candidates
            .iter()
            .any(|candidate| candidate.pattern_kind == "maintenance_signal"
                && candidate.status == MaturePatternCandidateStatus::Candidate
                && candidate.requires_user_confirmation));
        assert!(preview
            .mature_pattern_candidates
            .iter()
            .any(|candidate| candidate.pattern_kind == "repeated_candidate"
                && candidate.member_refs.len() >= 2));
        assert!(preview
            .cluster_reports
            .iter()
            .any(|report| report.member_refs.len() >= 2
                && report
                    .display_text
                    .contains("报告可下钻来源，但不是正式事实")));
        assert!(preview
            .acceptance_summary
            .display_text
            .contains("M13 最终验收仍后置"));
        assert!(
            !mature_pattern_store::sidecar_path(&path)
                .expect("pattern sidecar path")
                .exists(),
            "preview must not write memory-patterns sidecar"
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn memory_cluster_report_and_unconfirmed_mature_pattern_do_not_enter_task_packet() {
        let dir = test_temp_dir("memory-cluster-report-not-formal");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/memory-cluster-report-not-formal";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        prepare_m12_repeated_candidate_fixture(&path, project_root);
        let preview = mature_pattern_governance::preview_mature_patterns(
            &path,
            &fixture_m12_preview_input(project_root),
            "2026-06-05T12:10:00Z",
        )
        .expect("mature pattern preview should build");

        assert!(!preview.mature_pattern_candidates.is_empty());
        assert!(!preview.cluster_reports.is_empty());
        let packet = preview_task_memory_packet_at(
            &path,
            &fixture_task_memory_packet_input(project_root, "repeat review failure checklist"),
            "2026-06-05T12:10:01Z",
        )
        .expect("task memory packet should build");

        assert!(packet.preview.included_memories.is_empty());
        assert!(packet
            .preview
            .excluded_items
            .iter()
            .all(|item| item.source_kind != "memory_cluster_report"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn mature_pattern_user_confirmation_writes_formal_memory_and_task_packet_can_recall() {
        let dir = test_temp_dir("mature-pattern-user-confirmation");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/mature-pattern-user-confirmation";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        prepare_m12_repeated_candidate_fixture(&path, project_root);
        let preview = mature_pattern_governance::preview_mature_patterns(
            &path,
            &fixture_m12_preview_input(project_root),
            "2026-06-05T12:20:00Z",
        )
        .expect("mature pattern preview should build");
        let candidate = preview
            .mature_pattern_candidates
            .iter()
            .find(|candidate| candidate.pattern_kind == "repeated_candidate")
            .expect("repeated candidate should exist")
            .clone();

        let blocked = mature_pattern_governance::record_mature_pattern_decision(
            &path,
            &RecordMaturePatternDecisionInput {
                project_root: project_root.to_string(),
                candidate_id: candidate.candidate_id.clone(),
                decision: MaturePatternDecisionKind::ConfirmAsFormalMemory,
                actor_id: "project-director-m12".to_string(),
                actor_role: "project_director".to_string(),
                confirmed_by: Some("project_director".to_string()),
                reason: "项目主管尝试确认成熟模式，应被拒绝。".to_string(),
                expected_pattern_store_revision: Some(preview.store_revision),
                expected_formal_store_revision: Some(0),
            },
            "2026-06-05T12:20:01Z",
            "write-m12-project-director-blocked",
            "write-m12-formal-blocked",
        )
        .unwrap_err();
        assert!(blocked.contains("必须由用户确认"));

        let output = mature_pattern_governance::record_mature_pattern_decision(
            &path,
            &RecordMaturePatternDecisionInput {
                project_root: project_root.to_string(),
                candidate_id: candidate.candidate_id.clone(),
                decision: MaturePatternDecisionKind::ConfirmAsFormalMemory,
                actor_id: "user-m12".to_string(),
                actor_role: "user".to_string(),
                confirmed_by: Some("user".to_string()),
                reason: "用户确认该重复评审失败模式可作为成熟模式正式记忆。".to_string(),
                expected_pattern_store_revision: Some(preview.store_revision),
                expected_formal_store_revision: Some(0),
            },
            "2026-06-05T12:20:02Z",
            "write-m12-user-confirm",
            "write-m12-formal-confirm",
        )
        .expect("user confirmation should write formal memory");

        assert_eq!(
            output.candidate.status,
            MaturePatternCandidateStatus::Confirmed
        );
        let formal_gate = output
            .acceptance_summary
            .gates
            .iter()
            .find(|gate| gate.gate_id == "formal_memory")
            .expect("formal memory gate should exist");
        assert_eq!(formal_gate.status, "passed");
        assert!(
            formal_gate
                .evidence
                .contains("record 1 / version 1 / audit 1"),
            "formal memory gate should use fresh formal store after mature pattern formalization"
        );
        let task_packet_gate = output
            .acceptance_summary
            .gates
            .iter()
            .find(|gate| gate.gate_id == "task_packet")
            .expect("task packet gate should exist");
        assert_eq!(task_packet_gate.status, "passed");
        assert!(task_packet_gate.blocking_reason.is_none());
        let formal_output = output
            .formal_memory_output
            .expect("formal mature pattern memory should be written");
        assert_eq!(formal_output.record.memory_type, "mature_pattern");
        assert_eq!(formal_output.record.scope.scope_type, "global");
        assert!(!formal_output.record.source_refs.is_empty());
        assert_eq!(
            formal_output.audit_event.event_type,
            "mature_pattern_user_confirmed_to_formal_memory"
        );
        let formal_store = formal_memory_store::load_store(&path, "2026-06-05T12:20:03Z")
            .expect("formal store should load");
        assert_eq!(formal_store.records.len(), 1);
        assert_eq!(formal_store.versions.len(), 1);
        assert_eq!(formal_store.audit_events.len(), 1);

        let packet = preview_task_memory_packet_at(
            &path,
            &fixture_task_memory_packet_input(project_root, "repeat review failure checklist"),
            "2026-06-05T12:20:04Z",
        )
        .expect("task memory packet should build");
        assert_eq!(packet.preview.included_memories.len(), 1);
        assert_eq!(
            packet.preview.included_memories[0].memory_type,
            "mature_pattern"
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn mature_pattern_reject_quarantine_revision_and_damaged_json_do_not_mutate_formal_memory() {
        let dir = test_temp_dir("mature-pattern-reject-conflict-damaged");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/mature-pattern-reject-conflict-damaged";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        prepare_m12_repeated_candidate_fixture(&path, project_root);
        let preview = mature_pattern_governance::preview_mature_patterns(
            &path,
            &fixture_m12_preview_input(project_root),
            "2026-06-05T12:30:00Z",
        )
        .expect("mature pattern preview should build");
        let candidate_id = preview.mature_pattern_candidates[0].candidate_id.clone();

        let conflict = mature_pattern_governance::record_mature_pattern_decision(
            &path,
            &RecordMaturePatternDecisionInput {
                project_root: project_root.to_string(),
                candidate_id: candidate_id.clone(),
                decision: MaturePatternDecisionKind::Reject,
                actor_id: "global-director-m12".to_string(),
                actor_role: "global_director".to_string(),
                confirmed_by: None,
                reason: "expected revision mismatch should fail".to_string(),
                expected_pattern_store_revision: Some(99),
                expected_formal_store_revision: None,
            },
            "2026-06-05T12:30:01Z",
            "write-m12-revision-conflict",
            "write-m12-formal-unused",
        )
        .unwrap_err();
        assert!(conflict.contains("memory_pattern_store_conflict"));

        let reject_output = mature_pattern_governance::record_mature_pattern_decision(
            &path,
            &RecordMaturePatternDecisionInput {
                project_root: project_root.to_string(),
                candidate_id,
                decision: MaturePatternDecisionKind::Reject,
                actor_id: "global-director-m12".to_string(),
                actor_role: "global_director".to_string(),
                confirmed_by: None,
                reason: "全局主管拒绝成熟模式候选，但不删除来源。".to_string(),
                expected_pattern_store_revision: Some(preview.store_revision),
                expected_formal_store_revision: None,
            },
            "2026-06-05T12:30:02Z",
            "write-m12-reject",
            "write-m12-formal-unused-2",
        )
        .expect("reject should write pattern sidecar only");
        assert!(reject_output.formal_memory_output.is_none());
        let reject_formal_gate = reject_output
            .acceptance_summary
            .gates
            .iter()
            .find(|gate| gate.gate_id == "formal_memory")
            .expect("formal memory gate should exist");
        assert_eq!(reject_formal_gate.status, "blocked");
        assert!(
            reject_formal_gate
                .evidence
                .contains("record 0 / version 0 / audit 0"),
            "reject summary should not report fresh formal memory"
        );
        let formal_store = formal_memory_store::load_store(&path, "2026-06-05T12:30:03Z")
            .expect("formal store should load");
        assert!(formal_store.records.is_empty());
        let pattern_store = mature_pattern_store::load_store(&path, "2026-06-05T12:30:03Z")
            .expect("pattern store should load");
        assert_eq!(pattern_store.revision, 1);
        assert_eq!(
            pattern_store.mature_pattern_candidates[0].status,
            MaturePatternCandidateStatus::Rejected
        );
        let quarantine_candidate_id = preview.mature_pattern_candidates[1].candidate_id.clone();
        let quarantine_output = mature_pattern_governance::record_mature_pattern_decision(
            &path,
            &RecordMaturePatternDecisionInput {
                project_root: project_root.to_string(),
                candidate_id: quarantine_candidate_id,
                decision: MaturePatternDecisionKind::Quarantine,
                actor_id: "global-director-m12".to_string(),
                actor_role: "global_director".to_string(),
                confirmed_by: None,
                reason: "全局主管隔离成熟模式候选，但不写正式记忆。".to_string(),
                expected_pattern_store_revision: Some(pattern_store.revision),
                expected_formal_store_revision: None,
            },
            "2026-06-05T12:30:04Z",
            "write-m12-quarantine",
            "write-m12-formal-unused-4",
        )
        .expect("quarantine should write pattern sidecar only");
        assert!(quarantine_output.formal_memory_output.is_none());
        let formal_store_after_quarantine =
            formal_memory_store::load_store(&path, "2026-06-05T12:30:05Z")
                .expect("formal store should load after quarantine");
        assert!(formal_store_after_quarantine.records.is_empty());

        let damaged_dir = test_temp_dir("mature-pattern-damaged-json");
        let damaged_path = damaged_dir.join("workflow-state.v0.json");
        let damaged_project_root = "/tmp/mature-pattern-damaged-json";
        bootstrap_project_workflow_at(&damaged_path, &fixture_project(damaged_project_root))
            .expect("workflow state should include project");
        prepare_m12_repeated_candidate_fixture(&damaged_path, damaged_project_root);
        let damaged_preview = mature_pattern_governance::preview_mature_patterns(
            &damaged_path,
            &fixture_m12_preview_input(damaged_project_root),
            "2026-06-05T12:31:00Z",
        )
        .expect("mature pattern preview should build");
        let sidecar = mature_pattern_store::sidecar_path(&damaged_path).expect("sidecar path");
        fs::write(&sidecar, "{ damaged json").expect("test should write damaged pattern store");
        let damaged = mature_pattern_governance::record_mature_pattern_decision(
            &damaged_path,
            &RecordMaturePatternDecisionInput {
                project_root: damaged_project_root.to_string(),
                candidate_id: damaged_preview.mature_pattern_candidates[0]
                    .candidate_id
                    .clone(),
                decision: MaturePatternDecisionKind::Quarantine,
                actor_id: "global-director-m12".to_string(),
                actor_role: "global_director".to_string(),
                confirmed_by: None,
                reason: "损坏 JSON 不应被覆盖。".to_string(),
                expected_pattern_store_revision: Some(damaged_preview.store_revision),
                expected_formal_store_revision: None,
            },
            "2026-06-05T12:31:01Z",
            "write-m12-damaged-json",
            "write-m12-formal-unused-3",
        )
        .unwrap_err();
        assert!(damaged.contains("JSON 损坏"));
        assert_eq!(
            fs::read_to_string(&sidecar).expect("damaged sidecar should remain"),
            "{ damaged json"
        );

        let _ = fs::remove_dir_all(dir);
        let _ = fs::remove_dir_all(damaged_dir);
    }

    #[test]
    fn missing_workflow_state_returns_empty_without_creating_file() {
        let dir = std::env::temp_dir().join(format!(
            "workflow-state-missing-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");

        let snapshot = read_workflow_state_snapshot(&path)
            .expect("missing state should return empty snapshot");

        assert!(!snapshot.exists);
        assert!(!snapshot.initialized);
        assert_eq!(snapshot.counts.audit_events, 0);
        assert!(!path.exists());
    }

    #[test]
    fn initializes_workflow_state_with_audit_event() {
        let dir =
            std::env::temp_dir().join(format!("workflow-state-init-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");

        let result = initialize_workflow_state_at(&path).expect("initialize should write state");

        assert!(path.exists());
        assert!(result.first_initialize);
        assert!(result.backup_path.is_none());
        assert_eq!(
            result.snapshot.schema_version.as_deref(),
            Some("workflow_state_v0")
        );
        assert_eq!(result.snapshot.workflow_version, Some(1));
        assert_eq!(result.snapshot.counts.audit_events, 1);

        let text = fs::read_to_string(&path).expect("state file should be readable");
        let value: Value = serde_json::from_str(&text).expect("state should be json");
        assert_eq!(
            value["audit_events"][0]["event_type"],
            "workflow_state_initialized"
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn existing_workflow_state_is_backed_up_before_initialize() {
        let dir =
            std::env::temp_dir().join(format!("workflow-state-backup-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        fs::create_dir_all(&dir).expect("fixture dir should be created");
        fs::write(&path, "{\"old\":true}").expect("old state should be written");

        let result =
            initialize_workflow_state_at(&path).expect("initialize should replace old state");
        let backup_path = result
            .backup_path
            .expect("existing state should be backed up");

        assert!(!result.first_initialize);
        assert!(PathBuf::from(&backup_path).exists());
        let backup_text = fs::read_to_string(backup_path).expect("backup should be readable");
        assert_eq!(backup_text, "{\"old\":true}");

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn bootstrap_project_workflow_initializes_missing_state() {
        let dir = std::env::temp_dir().join(format!(
            "workflow-bootstrap-missing-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");

        let result = bootstrap_project_workflow_at(&path, &project)
            .expect("bootstrap should create state and workflow");

        assert!(result.first_initialize);
        assert!(path.exists());
        assert_eq!(result.snapshot.counts.projects, 1);
        assert_eq!(result.snapshot.counts.workflows, 1);
        assert_eq!(result.snapshot.counts.nodes, 7);
        assert_eq!(result.snapshot.counts.edges, 6);
        assert_eq!(result.snapshot.counts.audit_events, 2);

        let value = read_json_file(&path);
        assert_eq!(value["workflows"][0]["state"], "draft");
        assert_eq!(
            value["audit_events"][1]["event_type"],
            "project_default_workflow_bootstrapped"
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn bootstrap_project_workflow_does_not_duplicate_existing_workflow() {
        let dir = std::env::temp_dir().join(format!(
            "workflow-bootstrap-duplicate-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");

        bootstrap_project_workflow_at(&path, &project).expect("first bootstrap should write");
        let second =
            bootstrap_project_workflow_at(&path, &project).expect("second bootstrap should no-op");

        assert_eq!(second.snapshot.counts.workflows, 1);
        assert_eq!(second.snapshot.counts.nodes, 7);
        assert_eq!(second.audit_event_id, "no-op:existing-workflow");

        let value = read_json_file(&path);
        assert_eq!(array_len(&value, "workflows"), 1);
        assert_eq!(array_len(&value, "nodes"), 7);

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn bootstrap_project_workflow_rejects_non_index_project() {
        let index = json!({
          "projects": [{ "project_root": "/tmp/indexed-project" }]
        });

        assert!(find_index_project(&index, "/tmp/indexed-project").is_some());
        assert!(find_index_project(&index, "/tmp/not-indexed").is_none());
    }

    #[test]
    fn bootstrap_project_workflow_backs_up_existing_state() {
        let dir = std::env::temp_dir().join(format!(
            "workflow-bootstrap-backup-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");

        initialize_workflow_state_at(&path).expect("initial state should exist");
        let result = bootstrap_project_workflow_at(&path, &project)
            .expect("bootstrap should back up existing state");

        assert!(!result.first_initialize);
        let backup_path = result
            .backup_path
            .expect("existing state should be backed up");
        assert!(PathBuf::from(backup_path).exists());
        assert_eq!(result.snapshot.counts.workflows, 1);

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_draft_rejects_missing_workflow_state() {
        let dir = std::env::temp_dir().join(format!(
            "task-draft-missing-state-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let request = fixture_task_draft_request("/tmp/indexed-project", "草稿 A");

        let result = create_task_draft_at(&path, &request);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("工作流状态文件不存在"));
        assert!(!path.exists());
    }

    #[test]
    fn task_draft_rejects_project_without_workflow() {
        let dir = std::env::temp_dir().join(format!(
            "task-draft-missing-workflow-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let request = fixture_task_draft_request("/tmp/indexed-project", "草稿 A");

        initialize_workflow_state_at(&path).expect("state should exist");
        let result = create_task_draft_at(&path, &request);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("还没有本地 workflow"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_draft_creates_work_item_artifact_and_audit() {
        let dir =
            std::env::temp_dir().join(format!("task-draft-create-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");
        let request = fixture_task_draft_request(&project.project_root, "登记任务包草稿");

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        let result = create_task_draft_at(&path, &request).expect("task draft should be created");

        assert_eq!(result.snapshot.counts.work_items, 1);
        assert_eq!(result.snapshot.counts.artifacts, 1);
        assert_eq!(result.snapshot.project_workflows[0].task_draft_count, 1);
        assert_eq!(
            result.snapshot.project_workflows[0].task_drafts[0].title,
            "登记任务包草稿"
        );
        assert_eq!(
            result.snapshot.project_workflows[0].task_drafts[0]
                .artifact_type
                .as_deref(),
            Some("task_package")
        );

        let value = read_json_file(&path);
        assert_eq!(value["work_items"][0]["title"], "登记任务包草稿");
        assert_eq!(value["work_items"][0]["assigned_role_id"], "codex-dev");
        assert_eq!(value["work_items"][0]["agent_type"], "codex");
        assert_eq!(value["work_items"][0]["adapter_id"], "codex-local");
        assert_eq!(value["artifacts"][0]["artifact_type"], "task_package");
        assert_eq!(
            value["artifacts"][0]["brief"],
            "写入 work_items 和 artifacts"
        );
        assert!(value["artifacts"][0]["path"].is_null());
        assert!(value["audit_events"]
            .as_array()
            .expect("audit events should be array")
            .iter()
            .any(|event| event["event_type"] == "task_draft_created"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_draft_rejects_non_index_project() {
        let dir =
            std::env::temp_dir().join(format!("task-draft-non-index-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        let index = json!({
          "projects": [{ "project_root": "/tmp/indexed-project" }]
        });
        let request = fixture_task_draft_request("/tmp/not-indexed", "草稿 A");

        initialize_workflow_state_at(&path).expect("state should exist");
        let result = create_task_draft_for_index_project_at(&path, &index, &request);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("项目不在当前索引内"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_draft_backs_up_existing_state_before_write() {
        let dir =
            std::env::temp_dir().join(format!("task-draft-backup-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");
        let request = fixture_task_draft_request(&project.project_root, "草稿 A");

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        let before_text = fs::read_to_string(&path).expect("state should be readable");
        let result = create_task_draft_at(&path, &request).expect("task draft should be created");
        let backup_path = result
            .backup_path
            .expect("task draft write should back up old state");

        assert!(PathBuf::from(&backup_path).exists());
        let backup_text = fs::read_to_string(backup_path).expect("backup should be readable");
        assert_eq!(backup_text, before_text);

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_draft_does_not_duplicate_same_workflow_title() {
        let dir =
            std::env::temp_dir().join(format!("task-draft-duplicate-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");
        let request = fixture_task_draft_request(&project.project_root, "草稿 A");

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_task_draft_at(&path, &request).expect("first draft should be created");
        let second = create_task_draft_at(&path, &request).expect("duplicate draft should no-op");

        assert_eq!(second.audit_event_id, "no-op:existing-task-draft");
        assert_eq!(second.snapshot.counts.work_items, 1);
        assert_eq!(second.snapshot.counts.artifacts, 1);

        let value = read_json_file(&path);
        assert_eq!(array_len(&value, "work_items"), 1);
        assert_eq!(array_len(&value, "artifacts"), 1);

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn work_item_state_update_advances_state_node_and_audit() {
        let dir = std::env::temp_dir().join(format!(
            "work-item-state-advance-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");
        let request = fixture_task_draft_request(&project.project_root, "编排闭环工作项");

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_task_draft_at(&path, &request).expect("work item should exist");
        let value = read_json_file(&path);
        let work_item_id = optional_string_from(&value["work_items"][0], "work_item_id")
            .expect("work item id should exist");
        let update = fixture_work_item_state_update_request(
            &project.project_root,
            &work_item_id,
            "ready_to_dispatch",
        );

        let result =
            update_work_item_state_at(&path, &update).expect("state update should succeed");

        assert_eq!(
            result.snapshot.project_workflows[0].task_drafts[0].state,
            "ready_to_dispatch"
        );
        assert_eq!(
            result.snapshot.project_workflows[0].task_drafts[0].next_states,
            vec!["running".to_string(), "paused".to_string()]
        );
        let updated = read_json_file(&path);
        assert_eq!(updated["work_items"][0]["state"], "ready_to_dispatch");
        assert_eq!(
            updated["work_items"][0]["current_node_id"],
            format!(
                "{}:node:director",
                default_workflow_id(&project.project_root)
            )
        );
        assert!(updated["audit_events"]
            .as_array()
            .expect("audit events should be array")
            .iter()
            .any(|event| event["event_type"] == "work_item_state_changed"
                && event["before_state"] == "draft"
                && event["after_state"] == "ready_to_dispatch"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn work_item_state_update_rejects_illegal_transition() {
        let dir = std::env::temp_dir().join(format!(
            "work-item-state-illegal-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");
        let request = fixture_task_draft_request(&project.project_root, "非法流转工作项");

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_task_draft_at(&path, &request).expect("work item should exist");
        let value = read_json_file(&path);
        let work_item_id = optional_string_from(&value["work_items"][0], "work_item_id")
            .expect("work item id should exist");
        let update =
            fixture_work_item_state_update_request(&project.project_root, &work_item_id, "running");

        let result = update_work_item_state_at(&path, &update);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("非法工作项状态跳转"));
        let updated = read_json_file(&path);
        assert_eq!(updated["work_items"][0]["state"], "draft");

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn workflow_state_store_helpers_preserve_write_and_backup_behavior() {
        let dir = std::env::temp_dir().join(format!(
            "workflow-state-store-boundary-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");

        initialize_workflow_state_at(&path).expect("state should initialize");
        let timestamp = unix_timestamp_string();
        let backup = backup_workflow_state_file(&path, &timestamp).expect("backup should write");
        assert!(backup.exists());

        let value = read_workflow_state_value(&path).expect("state should read");
        assert!(validate_workflow_state(&value).is_empty());
        write_validated_workflow_state(&path, &value).expect("valid state should write");

        let invalid = json!({
            "schema_version": "bad",
            "workflow_version": 1,
        });
        let rejected = write_validated_workflow_state(&path, &invalid);
        assert!(rejected.is_err());
        assert!(rejected.unwrap_err().contains("写入前 schema 校验失败"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn workflow_permission_decision_records_audit_through_control_core() {
        let dir = std::env::temp_dir().join(format!(
            "permission-decision-control-core-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");
        let request = fixture_task_draft_request(&project.project_root, "权限确认工作项");

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_task_draft_at(&path, &request).expect("work item should exist");
        let value = read_json_file(&path);
        let work_item_id = optional_string_from(&value["work_items"][0], "work_item_id")
            .expect("work item id should exist");
        append_fixture_permission_request(&path, &project.project_root, &work_item_id, "pending");

        let result = record_workflow_permission_decision_at(
            &path,
            &WorkflowPermissionDecisionRequest {
                project_root: project.project_root.clone(),
                work_item_id: work_item_id.clone(),
                request_id: "permission:fixture:001".to_string(),
                decision: "approved".to_string(),
            },
        )
        .expect("permission decision should write");

        assert!(result.message.contains("批准"));
        let updated = read_json_file(&path);
        assert_eq!(updated["permission_requests"][0]["status"], "approved");
        assert_eq!(updated["permission_requests"][0]["decision"], "approved");
        assert!(updated["permission_requests"][0]["decided_at"]
            .as_str()
            .is_some());
        assert!(updated["audit_events"]
            .as_array()
            .expect("audit events should be array")
            .iter()
            .any(
                |event| event["event_type"] == "workflow_permission_decision_recorded"
                    && event["target_ref"] == "permission:fixture:001"
                    && event["before_state"] == "pending"
                    && event["after_state"] == "approved"
            ));

        let duplicate = record_workflow_permission_decision_at(
            &path,
            &WorkflowPermissionDecisionRequest {
                project_root: project.project_root.clone(),
                work_item_id,
                request_id: "permission:fixture:001".to_string(),
                decision: "rejected".to_string(),
            },
        );
        assert!(duplicate.is_err());
        assert!(duplicate.unwrap_err().contains("不是 pending"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn workflow_audit_helper_preserves_work_item_state_changed_fields() {
        let event =
            workflow_audit::work_item_state_changed(workflow_audit::WorkItemStateChangedAudit {
                event_id: "audit:fixture".to_string(),
                work_item_id: "work-item:fixture",
                before_state: "draft",
                after_state: "ready_to_dispatch",
                created_at: "2026-06-01T00:00:00Z",
                reason: "fixture reason".to_string(),
            });

        assert_eq!(event["event_id"], "audit:fixture");
        assert_eq!(event["event_type"], "work_item_state_changed");
        assert_eq!(event["target_ref"], "work-item:fixture");
        assert_eq!(event["actor_ref"], "user_confirmed_desktop_shell");
        assert_eq!(event["source_kind"], "workspace_state");
        assert_eq!(event["permission_level"], "user_confirmed_write");
        assert_eq!(event["before_state"], "draft");
        assert_eq!(event["after_state"], "ready_to_dispatch");
        assert_eq!(event["created_at"], "2026-06-01T00:00:00Z");
        assert_eq!(event["reason"], "fixture reason");
    }

    #[test]
    fn workflow_audit_helper_preserves_permission_decision_fields() {
        let event = workflow_audit::workflow_permission_decision_recorded(
            workflow_audit::WorkflowPermissionDecisionRecordedAudit {
                event_id: "audit:permission:fixture".to_string(),
                request_id: "permission:fixture",
                before_state: "pending",
                after_state: "approved",
                created_at: "2026-06-01T00:00:00Z",
            },
        );

        assert_eq!(event["event_id"], "audit:permission:fixture");
        assert_eq!(event["event_type"], "workflow_permission_decision_recorded");
        assert_eq!(event["target_ref"], "permission:fixture");
        assert_eq!(event["actor_ref"], "user_confirmed_desktop_shell");
        assert_eq!(event["source_kind"], "workspace_state_permission_queue");
        assert_eq!(event["permission_level"], "user_confirmed_write");
        assert_eq!(event["before_state"], "pending");
        assert_eq!(event["after_state"], "approved");
        assert_eq!(event["created_at"], "2026-06-01T00:00:00Z");
        assert_eq!(
            event["reason"],
            "用户确认记录权限请求结论；不启动 Codex、不 resume、不发送消息。"
        );
    }

    #[test]
    fn blackboard_candidate_decision_boundary_rejects_direct_promotion() {
        assert_eq!(
            control_core::validate_blackboard_candidate_decision(
                "memory_candidate",
                "formal_memory",
                "mark_pending",
            )
            .expect("pending should be allowed"),
            control_core::BlackboardCandidateDecisionOutcome::Pending
        );
        assert_eq!(
            control_core::validate_blackboard_candidate_decision(
                "risk",
                "workflow_risk",
                "reject_candidate",
            )
            .expect("reject should be allowed"),
            control_core::BlackboardCandidateDecisionOutcome::Rejected
        );
        assert!(control_core::validate_blackboard_candidate_decision(
            "memory_candidate",
            "formal_memory",
            "confirm_candidate",
        )
        .unwrap_err()
        .contains("不能直接写正式记忆"));
        assert!(control_core::validate_blackboard_candidate_decision(
            "knowledge_ref",
            "formal_memory",
            "confirm_candidate",
        )
        .unwrap_err()
        .contains("知识引用不是记忆"));
        assert!(control_core::validate_blackboard_candidate_decision(
            "tool_summary",
            "workflow_state_change",
            "confirm_candidate",
        )
        .unwrap_err()
        .contains("不能直接推进 workflow state"));
        assert_eq!(
            control_core::validate_blackboard_candidate_decision(
                "subagent_report",
                "workflow_fact",
                "candidate_confirmed_for_followup",
            )
            .expect("followup confirmation should be a candidate-only state"),
            control_core::BlackboardCandidateDecisionOutcome::ConfirmedForFollowup
        );
        assert_eq!(
            control_core::validate_blackboard_candidate_decision(
                "risk",
                "workflow_risk",
                "candidate_deferred",
            )
            .expect("defer should be a candidate-only state"),
            control_core::BlackboardCandidateDecisionOutcome::Deferred
        );
        assert_eq!(
            control_core::validate_blackboard_candidate_decision(
                "tool_summary",
                "audit_event",
                "candidate_discarded",
            )
            .expect("discard should be a candidate-only state"),
            control_core::BlackboardCandidateDecisionOutcome::Discarded
        );
        assert!(
            control_core::validate_blackboard_candidate_decision(
                "memory_candidate",
                "formal_memory",
                "candidate_confirmed_for_memory",
            )
            .is_err(),
            "blackboard candidate confirmation must not promote directly to memory"
        );
    }

    #[test]
    fn blackboard_candidate_store_records_candidate_only_decisions() {
        let dir = std::env::temp_dir().join(format!(
            "blackboard-candidate-store-{}",
            unix_timestamp_nanos()
        ));
        fs::create_dir_all(&dir).expect("temp dir should exist");
        let path = dir.join("workflow-state.v0.json");
        let request = RecordBlackboardCandidateDecisionInput {
            project_id: "project:offline".to_string(),
            project_root: "/offline-fixture/projects/codex-workbench".to_string(),
            workflow_id: "workflow:offline:default".to_string(),
            candidate_key: None,
            source_entry_id: Some("blackboard:offline:report:001".to_string()),
            entry_kind: BlackboardEntryKind::SubagentReport,
            target_kind: BlackboardCandidateTargetKind::WorkflowFact,
            requested_state: BlackboardCandidateState::CandidateConfirmedForFollowup,
            reason: "候选值得后续处理；不写正式事实。".to_string(),
            actor_role: "project_director".to_string(),
            actor_session_id: None,
            source_refs: vec![BlackboardCandidateSourceRef {
                source_kind: "subagent_report".to_string(),
                source_id: "report:offline:001".to_string(),
                label: "子智能体汇报".to_string(),
            }],
            expected_store_revision: None,
            title_snapshot: Some("离线子汇报".to_string()),
            summary_snapshot: Some("只确认后续处理。".to_string()),
            source_status: None,
            work_item_id: None,
            workflow_node_id: None,
        };

        let result = blackboard_candidate_store::record_decision(
            &path,
            &request,
            "2026-06-03T00:00:00Z",
            "write-blackboard-001",
        )
        .expect("blackboard candidate decision should write sidecar");
        assert_eq!(result.store_revision, 1);
        assert_eq!(
            result.record.state,
            BlackboardCandidateState::CandidateConfirmedForFollowup
        );
        assert!(path
            .parent()
            .expect("path should have parent")
            .join("blackboard-candidates.v1.json")
            .exists());
        assert!(
            !path.exists(),
            "blackboard sidecar write must not create workflow state JSON"
        );

        let conflict = blackboard_candidate_store::record_decision(
            &path,
            &RecordBlackboardCandidateDecisionInput {
                expected_store_revision: Some(0),
                requested_state: BlackboardCandidateState::CandidateRejected,
                reason: "并发冲突测试".to_string(),
                ..request.clone()
            },
            "2026-06-03T00:00:01Z",
            "write-blackboard-002",
        )
        .unwrap_err();
        assert!(conflict.contains("blackboard_candidate_store_conflict"));
    }

    #[test]
    fn observation_store_records_worker_report() {
        let dir = test_temp_dir("observation-store-records-worker");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/observation-store-records-worker";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");

        let created = create_recorded_observation(&path, project_root);

        assert_eq!(created.store_revision, 1);
        assert_eq!(created.observation.status, ObservationStatus::Recorded);
        assert_eq!(created.observation.observation_type, "worker_report");
        assert!(!created.observation.source_refs.is_empty());
        assert!(dir.join("observations.v1.json").exists());
        let summary = observation_store::summarize_store(
            &observation_store::load_store(&path, "2026-06-04T00:00:01Z")
                .expect("observation store should load"),
        );
        assert_eq!(summary.recorded_count, 1);
        assert!(summary.display_text.contains("observation 不是正式记忆"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn observation_candidate_creation_project_director() {
        let dir = test_temp_dir("observation-candidate-project-director");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/observation-candidate-project-director";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        let observation = create_recorded_observation(&path, project_root);

        let created = create_memory_candidate_from_observation_at(
            &path,
            &fixture_observation_candidate_input(
                project_root,
                observation.observation.observation_key.clone(),
                Some(1),
                Some(0),
            ),
            "2026-06-04T00:00:02Z",
            "write-observation-candidate",
            "write-candidate-from-observation",
        )
        .expect("project director should create memory candidate from observation");

        assert_eq!(created.observation_store_revision, 2);
        assert_eq!(created.candidate_store_revision, 1);
        assert_eq!(
            created.candidate.status,
            MemoryLifecycleStatus::CandidateNeedsReview
        );
        assert_eq!(
            created.observation.status,
            ObservationStatus::CandidateCreated
        );
        assert_eq!(
            created.observation.candidate_key.as_deref(),
            Some(created.candidate.candidate_key.as_str())
        );
        assert_eq!(
            created.observation_audit_event.event_type,
            "observation_candidate_created"
        );
        let candidate_store = memory_candidate_store::load_store(&path, "2026-06-04T00:00:03Z")
            .expect("candidate store should load");
        assert_eq!(candidate_store.candidates.len(), 1);

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn observation_candidate_creation_rejects_quarantined() {
        let dir = test_temp_dir("observation-candidate-quarantined");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/observation-candidate-quarantined";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        let observation = create_recorded_observation(&path, project_root);
        overwrite_first_observation_status(&path, ObservationStatus::Quarantined, None);

        let err = create_memory_candidate_from_observation_at(
            &path,
            &fixture_observation_candidate_input(
                project_root,
                observation.observation.observation_key,
                Some(1),
                Some(0),
            ),
            "2026-06-04T00:00:02Z",
            "write-observation-candidate-quarantined",
            "write-candidate-from-quarantined",
        )
        .unwrap_err();

        assert!(err.contains("当前状态：quarantined"));
        assert!(!dir.join("memory-candidates.v1.json").exists());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn observation_candidate_creation_rejects_ignored() {
        let dir = test_temp_dir("observation-candidate-ignored");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/observation-candidate-ignored";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        let observation = create_recorded_observation(&path, project_root);
        overwrite_first_observation_status(&path, ObservationStatus::Ignored, None);

        let err = create_memory_candidate_from_observation_at(
            &path,
            &fixture_observation_candidate_input(
                project_root,
                observation.observation.observation_key,
                Some(1),
                Some(0),
            ),
            "2026-06-04T00:00:02Z",
            "write-observation-candidate-ignored",
            "write-candidate-from-ignored",
        )
        .unwrap_err();

        assert!(err.contains("当前状态：ignored"));
        assert!(!dir.join("memory-candidates.v1.json").exists());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn observation_candidate_creation_rejects_duplicate() {
        let dir = test_temp_dir("observation-candidate-duplicate");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/observation-candidate-duplicate";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        let observation = create_recorded_observation(&path, project_root);
        let input = fixture_observation_candidate_input(
            project_root,
            observation.observation.observation_key.clone(),
            Some(1),
            Some(0),
        );
        create_memory_candidate_from_observation_at(
            &path,
            &input,
            "2026-06-04T00:00:02Z",
            "write-observation-candidate-first",
            "write-candidate-from-observation-first",
        )
        .expect("first candidate creation should succeed");

        let err = create_memory_candidate_from_observation_at(
            &path,
            &fixture_observation_candidate_input(
                project_root,
                observation.observation.observation_key,
                Some(2),
                Some(1),
            ),
            "2026-06-04T00:00:03Z",
            "write-observation-candidate-second",
            "write-candidate-from-observation-second",
        )
        .unwrap_err();

        assert!(err.contains("已经生成过 candidate"));
        let candidate_store = memory_candidate_store::load_store(&path, "2026-06-04T00:00:04Z")
            .expect("candidate store should load");
        assert_eq!(candidate_store.candidates.len(), 1);

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn observation_creation_rejects_missing_source_refs() {
        let dir = test_temp_dir("observation-missing-source-refs");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/observation-missing-source-refs";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        let mut input = fixture_observation_input(project_root);
        input.source_refs = vec![];

        let err = create_observation_at(
            &path,
            &input,
            "2026-06-04T00:00:00Z",
            "write-observation-missing-source",
        )
        .unwrap_err();

        assert!(err.contains("缺少 source_refs"));
        assert!(!dir.join("observations.v1.json").exists());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn observation_creation_rejects_ordinary_chat_auto_capture() {
        let dir = test_temp_dir("observation-ordinary-chat");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/observation-ordinary-chat";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        let mut input = fixture_observation_input(project_root);
        input.source_refs[0].source_kind = "ordinary_chat".to_string();
        input.source_refs[0].summary = "普通聊天摘要，未被明确确认为工作流事实。".to_string();

        let err = create_observation_at(
            &path,
            &input,
            "2026-06-04T00:00:00Z",
            "write-observation-ordinary-chat",
        )
        .unwrap_err();

        assert!(err.contains("普通聊天不能自动记录为 observation"));
        assert!(!dir.join("observations.v1.json").exists());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn observation_candidate_does_not_create_formal_memory() {
        let dir = test_temp_dir("observation-candidate-no-formal");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/observation-candidate-no-formal";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        let observation = create_recorded_observation(&path, project_root);

        create_memory_candidate_from_observation_at(
            &path,
            &fixture_observation_candidate_input(
                project_root,
                observation.observation.observation_key,
                Some(1),
                Some(0),
            ),
            "2026-06-04T00:00:02Z",
            "write-observation-candidate-no-formal",
            "write-candidate-from-observation-no-formal",
        )
        .expect("candidate should be created");

        let formal_store = formal_memory_store::load_store(&path, "2026-06-04T00:00:03Z")
            .expect("formal store should load empty");
        assert_eq!(formal_store.records.len(), 0);
        assert_eq!(formal_store.versions.len(), 0);
        assert_eq!(formal_store.audit_events.len(), 0);
        assert!(!dir.join("formal-memories.v1.json").exists());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn observation_context_binding_mismatch_rejected() {
        let dir = test_temp_dir("observation-context-mismatch");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/observation-context-mismatch";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        let mut input = fixture_observation_input(project_root);
        input.scope.project_id = Some("project:other".to_string());

        let err = create_observation_at(
            &path,
            &input,
            "2026-06-04T00:00:00Z",
            "write-observation-context-mismatch",
        )
        .unwrap_err();

        assert!(err.contains("observation 上下文绑定失败"));
        assert!(err.contains("scope.project_id"));
        assert!(!dir.join("observations.v1.json").exists());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_memory_packet_includes_active_formal_memory() {
        let dir = test_temp_dir("task-memory-packet-include-active");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/task-memory-packet-include-active";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        let record = create_formal_memory_for_task(
            &path,
            project_root,
            "接口完成事实可供后续任务使用",
            "接口实现已经完成，后续 worker 可以基于该正式记忆继续处理接口验收。",
            "2026-06-04T01:00:00Z",
            "write-task-memory-include-active",
        );

        let output = preview_task_memory_packet_at(
            &path,
            &fixture_task_memory_packet_input(project_root, "接口 验收"),
            "2026-06-04T01:00:01Z",
        )
        .expect("task memory packet preview should build");

        assert_eq!(output.preview.included_memories.len(), 1);
        assert_eq!(
            output.preview.included_memories[0].memory_id,
            record.memory_id
        );
        assert!(output.preview.included_memories[0]
            .retrieval_reason
            .contains("active formal memory"));
        assert!(output
            .warnings
            .contains(&"preview_only_not_injected".to_string()));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_memory_packet_excludes_candidates_as_unconfirmed() {
        let dir = test_temp_dir("task-memory-packet-candidate");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/task-memory-packet-candidate";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        let candidate = memory_candidate_store::create_candidate(
            &path,
            &fixture_bound_memory_candidate_input(project_root),
            "2026-06-04T01:00:00Z",
            "write-task-memory-candidate",
        )
        .expect("candidate should be created");

        let output = preview_task_memory_packet_at(
            &path,
            &fixture_task_memory_packet_input(project_root, "候选"),
            "2026-06-04T01:00:01Z",
        )
        .expect("task memory packet preview should build");

        assert!(output.preview.included_memories.is_empty());
        assert_eq!(
            excluded_reason_count(
                &output,
                TaskMemoryPacketExclusionReason::CandidateUnconfirmed
            ),
            1
        );
        assert!(output.preview.review_materials.iter().any(|material| {
            material.source_kind == "memory_candidate"
                && material.source_id == candidate.candidate.candidate_key
        }));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_memory_packet_excludes_observation_as_not_formal() {
        let dir = test_temp_dir("task-memory-packet-observation");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/task-memory-packet-observation";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        let observation = create_recorded_observation(&path, project_root);

        let output = preview_task_memory_packet_at(
            &path,
            &fixture_task_memory_packet_input(project_root, "worker 汇报"),
            "2026-06-04T01:00:01Z",
        )
        .expect("task memory packet preview should build");

        assert!(output.preview.included_memories.is_empty());
        assert_eq!(
            excluded_reason_count(
                &output,
                TaskMemoryPacketExclusionReason::ObservationNotFormalMemory
            ),
            1
        );
        assert!(output.preview.review_materials.iter().any(|material| {
            material.source_kind == "observation"
                && material.source_id == observation.observation.observation_key
        }));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_memory_packet_excludes_inactive_formal_memories() {
        let dir = test_temp_dir("task-memory-packet-inactive");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/task-memory-packet-inactive";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        for (index, claim) in [
            "接口 冲突正式记忆",
            "接口 废弃正式记忆",
            "接口 冻结正式记忆",
            "接口 归档正式记忆",
        ]
        .iter()
        .enumerate()
        {
            create_formal_memory_for_task(
                &path,
                project_root,
                claim,
                "接口相关但状态不允许进入任务记忆包。",
                &format!("2026-06-04T01:00:0{index}Z"),
                &format!("write-task-memory-inactive-{index}"),
            );
        }
        mutate_formal_store(&path, |store| {
            store.records[0].status = MemoryLifecycleStatus::MemoryConflicted;
            store.records[1].status = MemoryLifecycleStatus::MemoryDeprecated;
            store.records[2].status = MemoryLifecycleStatus::MemoryFrozen;
            store.records[3].status = MemoryLifecycleStatus::MemoryArchived;
        });

        let output = preview_task_memory_packet_at(
            &path,
            &fixture_task_memory_packet_input(project_root, "接口"),
            "2026-06-04T01:00:10Z",
        )
        .expect("task memory packet preview should build");

        assert!(output.preview.included_memories.is_empty());
        assert_eq!(
            excluded_reason_count(&output, TaskMemoryPacketExclusionReason::Conflicted),
            1
        );
        assert_eq!(
            excluded_reason_count(&output, TaskMemoryPacketExclusionReason::Stale),
            3
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_memory_packet_excludes_model_export_blocked() {
        let dir = test_temp_dir("task-memory-packet-export-blocked");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/task-memory-packet-export-blocked";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        create_formal_memory_for_task(
            &path,
            project_root,
            "接口 blocked export 正式记忆",
            "该正式记忆只允许本地上下文，不允许外发模型上下文。",
            "2026-06-04T01:00:00Z",
            "write-task-memory-export-blocked",
        );
        mutate_formal_store(&path, |store| {
            store.records[0].scope.model_export_policy = "blocked".to_string();
        });
        let mut input = fixture_task_memory_packet_input(project_root, "接口");
        input.model_context_policy = "external_model_context".to_string();

        let output = preview_task_memory_packet_at(&path, &input, "2026-06-04T01:00:01Z")
            .expect("task memory packet preview should build");

        assert!(output.preview.included_memories.is_empty());
        assert_eq!(
            excluded_reason_count(&output, TaskMemoryPacketExclusionReason::ModelExportBlocked),
            1
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_memory_packet_excludes_permission_blocked() {
        let dir = test_temp_dir("task-memory-packet-permission");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/task-memory-packet-permission";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        create_formal_memory_for_task(
            &path,
            project_root,
            "接口 跨项目正式记忆",
            "该记录被测试改为其他项目 scope，应被权限规则排除。",
            "2026-06-04T01:00:00Z",
            "write-task-memory-permission",
        );
        mutate_formal_store(&path, |store| {
            store.records[0].scope.project_id = Some("project:other".to_string());
        });

        let output = preview_task_memory_packet_at(
            &path,
            &fixture_task_memory_packet_input(project_root, "接口"),
            "2026-06-04T01:00:01Z",
        )
        .expect("task memory packet preview should build");

        assert!(output.preview.included_memories.is_empty());
        assert_eq!(
            excluded_reason_count(&output, TaskMemoryPacketExclusionReason::PermissionBlocked),
            1
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_memory_packet_excludes_token_limit() {
        let dir = test_temp_dir("task-memory-packet-token");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/task-memory-packet-token";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        create_formal_memory_for_task(
            &path,
            project_root,
            "接口 大段正式记忆",
            "接口 ".repeat(200).as_str(),
            "2026-06-04T01:00:00Z",
            "write-task-memory-token",
        );
        let mut input = fixture_task_memory_packet_input(project_root, "接口");
        input.max_estimated_tokens = 20;

        let output = preview_task_memory_packet_at(&path, &input, "2026-06-04T01:00:01Z")
            .expect("task memory packet preview should build");

        assert!(output.preview.included_memories.is_empty());
        assert_eq!(
            excluded_reason_count(&output, TaskMemoryPacketExclusionReason::TokenLimit),
            1
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_memory_packet_excludes_not_relevant() {
        let dir = test_temp_dir("task-memory-packet-not-relevant");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/task-memory-packet-not-relevant";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        create_formal_memory_for_task(
            &path,
            project_root,
            "缓存策略正式记忆",
            "构建缓存已完成，与支付网关无关。",
            "2026-06-04T01:00:00Z",
            "write-task-memory-not-relevant",
        );

        let output = preview_task_memory_packet_at(
            &path,
            &fixture_task_memory_packet_input(project_root, "payment gateway"),
            "2026-06-04T01:00:01Z",
        )
        .expect("task memory packet preview should build");

        assert!(output.preview.included_memories.is_empty());
        assert_eq!(
            excluded_reason_count(&output, TaskMemoryPacketExclusionReason::NotRelevant),
            1
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_memory_packet_preview_is_readonly() {
        let dir = test_temp_dir("task-memory-packet-readonly");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/task-memory-packet-readonly";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        create_formal_memory_for_task(
            &path,
            project_root,
            "接口 readonly 正式记忆",
            "接口正式记忆用于只读预览测试。",
            "2026-06-04T01:00:00Z",
            "write-task-memory-readonly-formal",
        );
        memory_candidate_store::create_candidate(
            &path,
            &fixture_bound_memory_candidate_input(project_root),
            "2026-06-04T01:00:01Z",
            "write-task-memory-readonly-candidate",
        )
        .expect("candidate should be created");
        create_recorded_observation(&path, project_root);
        let formal_before = formal_memory_store::load_store(&path, "2026-06-04T01:00:02Z")
            .expect("formal store should load")
            .revision;
        let candidate_before = memory_candidate_store::load_store(&path, "2026-06-04T01:00:02Z")
            .expect("candidate store should load")
            .revision;
        let observation_before = observation_store::load_store(&path, "2026-06-04T01:00:02Z")
            .expect("observation store should load")
            .revision;

        preview_task_memory_packet_at(
            &path,
            &fixture_task_memory_packet_input(project_root, "接口"),
            "2026-06-04T01:00:03Z",
        )
        .expect("task memory packet preview should build");

        assert_eq!(
            formal_memory_store::load_store(&path, "2026-06-04T01:00:04Z")
                .expect("formal store should load")
                .revision,
            formal_before
        );
        assert_eq!(
            memory_candidate_store::load_store(&path, "2026-06-04T01:00:04Z")
                .expect("candidate store should load")
                .revision,
            candidate_before
        );
        assert_eq!(
            observation_store::load_store(&path, "2026-06-04T01:00:04Z")
                .expect("observation store should load")
                .revision,
            observation_before
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_memory_packet_preview_does_not_execute_worker() {
        let dir = test_temp_dir("task-memory-packet-no-worker");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/task-memory-packet-no-worker";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        create_formal_memory_for_task(
            &path,
            project_root,
            "接口 no worker 正式记忆",
            "接口正式记忆用于验证预览不会创建派发。",
            "2026-06-04T01:00:00Z",
            "write-task-memory-no-worker",
        );
        let before = read_json_file(&path);

        preview_task_memory_packet_at(
            &path,
            &fixture_task_memory_packet_input(project_root, "接口"),
            "2026-06-04T01:00:01Z",
        )
        .expect("task memory packet preview should build");

        let after = read_json_file(&path);
        assert_eq!(after["node_dispatches"], before["node_dispatches"]);
        assert_eq!(after["execution_attempts"], before["execution_attempts"]);
        assert_eq!(
            after["workflow_execution_controls"],
            before["workflow_execution_controls"]
        );
        assert_eq!(after, before, "preview must not write workflow state");

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn memory_entity_relation_preview_suggests_alias_and_similarity_candidates_readonly() {
        let dir = test_temp_dir("memory-entity-relation-alias-preview");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/memory-entity-relation-alias-preview";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        create_formal_memory_for_task(
            &path,
            project_root,
            "Codex 工具别名治理",
            "同一工具在来源里可能写作 Codex CLI 或 codex tool。",
            "2026-06-05T10:00:00Z",
            "write-m10-alias-formal",
        );
        mutate_formal_store(&path, |store| {
            store.records[0].source_refs = vec![
                fixture_m10_memory_source("tool", "tool:codex-cli", "Codex CLI", "project"),
                fixture_m10_memory_source("tool", "tool:codex-tool", "codex tool", "project"),
                fixture_m10_memory_source(
                    "similarity_hit",
                    "similarity:codex",
                    "Codex CLI",
                    "project",
                ),
                fixture_m10_memory_source(
                    "similarity_hit",
                    "similarity:codex-tool",
                    "codex tool",
                    "project",
                ),
            ];
        });

        let preview = memory_entity_relation_governance::preview_candidates(
            &path,
            &fixture_m10_preview_input(project_root),
            "2026-06-05T10:00:01Z",
        )
        .expect("entity relation preview should build");

        assert!(preview
            .entity_candidates
            .iter()
            .any(|candidate| candidate.entity_kind == MemoryEntityKind::Tool
                && candidate.display_name == "Codex CLI"));
        assert!(preview
            .merge_candidates
            .iter()
            .any(|candidate| candidate.reason.contains("alias / dedupe")));
        assert!(preview
            .merge_candidates
            .iter()
            .any(|candidate| candidate.source_kind == MemoryRelationSourceKind::SimilarityHit));
        assert!(
            !memory_entity_relation_store::sidecar_path(&path)
                .expect("entity relation sidecar path")
                .exists(),
            "preview must not write entity relation sidecar"
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn memory_entity_relation_llm_inferred_causal_relation_stays_candidate() {
        let dir = test_temp_dir("memory-entity-relation-llm-candidate");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/memory-entity-relation-llm-candidate";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        create_formal_memory_for_task(
            &path,
            project_root,
            "LLM 推断导致任务包变更",
            "LLM inferred causal candidate fixture.",
            "2026-06-05T10:10:00Z",
            "write-m10-llm-formal",
        );
        mutate_formal_store(&path, |store| {
            store.records[0].source_refs = vec![fixture_m10_memory_source(
                "llm_inferred",
                "llm:relation:001",
                "LLM 因果推断",
                "project",
            )];
        });
        let preview = memory_entity_relation_governance::preview_candidates(
            &path,
            &fixture_m10_preview_input(project_root),
            "2026-06-05T10:10:01Z",
        )
        .expect("llm inferred preview should build");
        let candidate = preview
            .relation_candidates
            .iter()
            .find(|candidate| candidate.source_kind == MemoryRelationSourceKind::LlmInferred)
            .expect("llm inferred relation candidate should exist");

        assert_eq!(candidate.relation_kind, MemoryRelationKind::Causal);
        assert_eq!(candidate.status, MemoryRelationStatus::Candidate);
        assert!(candidate.requires_user_confirmation);
        let err = memory_entity_relation_governance::record_relation_decision(
            &path,
            &RecordMemoryRelationCandidateDecisionInput {
                project_root: project_root.to_string(),
                relation_candidate_id: candidate.candidate_id.clone(),
                decision: MemoryRelationCandidateDecisionKind::ConfirmRelation,
                actor_id: "project-director-m10".to_string(),
                actor_role: "project_director".to_string(),
                confirmed_by: Some("project_director".to_string()),
                reason: "尝试确认 LLM 推断关系，应被拒绝。".to_string(),
                expected_store_revision: Some(preview.store_revision),
            },
            "2026-06-05T10:10:02Z",
            "write-m10-llm-relation",
        )
        .expect_err("llm inferred relation must not become confirmed relation");

        assert!(err.contains("llm_inferred relation"), "{err}");
        assert!(
            !memory_entity_relation_store::sidecar_path(&path)
                .expect("entity relation sidecar path")
                .exists(),
            "rejected llm relation must not write sidecar"
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn memory_entity_relation_confirmed_causal_relation_explains_task_packet() {
        let dir = test_temp_dir("memory-entity-relation-task-packet");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/memory-entity-relation-task-packet";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        let record = create_formal_memory_for_task(
            &path,
            project_root,
            "接口 因果关系正式记忆",
            "接口契约变化导致任务包需要复核。",
            "2026-06-05T10:20:00Z",
            "write-m10-causal-formal",
        );
        mutate_formal_store(&path, |store| {
            store.records[0].source_refs = vec![fixture_m10_memory_source(
                "manual_note",
                "manual:contract-change",
                "接口契约资料",
                "project",
            )];
        });
        let before_confirm = preview_task_memory_packet_at(
            &path,
            &fixture_task_memory_packet_input(project_root, "接口"),
            "2026-06-05T10:20:01Z",
        )
        .expect("task packet should build before relation confirmation");
        assert_eq!(
            before_confirm.preview.included_memories[0].memory_id,
            record.memory_id
        );
        assert!(before_confirm.preview.included_memories[0]
            .relation_explanations
            .is_empty());

        let relation_preview = memory_entity_relation_governance::preview_candidates(
            &path,
            &fixture_m10_preview_input(project_root),
            "2026-06-05T10:20:02Z",
        )
        .expect("relation preview should build");
        let causal_candidate = relation_preview
            .relation_candidates
            .iter()
            .find(|candidate| candidate.relation_kind == MemoryRelationKind::Causal)
            .expect("causal relation candidate should exist");
        let decision = memory_entity_relation_governance::record_relation_decision(
            &path,
            &RecordMemoryRelationCandidateDecisionInput {
                project_root: project_root.to_string(),
                relation_candidate_id: causal_candidate.candidate_id.clone(),
                decision: MemoryRelationCandidateDecisionKind::ConfirmRelation,
                actor_id: "project-director-m10".to_string(),
                actor_role: "project_director".to_string(),
                confirmed_by: Some("project_director".to_string()),
                reason: "项目主管确认本项目低风险因果关系，用于解释召回原因。".to_string(),
                expected_store_revision: Some(relation_preview.store_revision),
            },
            "2026-06-05T10:20:03Z",
            "write-m10-causal-relation",
        )
        .expect("project director should confirm causal relation");
        assert_eq!(
            decision
                .relation
                .as_ref()
                .expect("confirmed relation should exist")
                .status,
            MemoryRelationStatus::Confirmed
        );

        let output = preview_task_memory_packet_at(
            &path,
            &fixture_task_memory_packet_input(project_root, "接口"),
            "2026-06-05T10:20:04Z",
        )
        .expect("task packet should include relation explanation after confirmation");
        let item = &output.preview.included_memories[0];
        assert!(!item.relation_explanations.is_empty());
        assert!(item.retrieval_reason.contains("已确认关系用于解释召回原因"));
        assert_eq!(output.entity_relation_store_revision, 1);

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn memory_entity_relation_secret_relation_source_is_not_exported_to_task_packet() {
        let dir = test_temp_dir("memory-entity-relation-secret-source");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/memory-entity-relation-secret-source";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        create_formal_memory_for_task(
            &path,
            project_root,
            "接口 secret 关系正式记忆",
            "接口 secret source 导致任务包需复核。",
            "2026-06-05T10:30:00Z",
            "write-m10-secret-formal",
        );
        mutate_formal_store(&path, |store| {
            store.records[0].source_refs = vec![fixture_m10_memory_source(
                "manual_note",
                "manual:secret-contract",
                "secret 接口资料",
                "secret",
            )];
        });
        let relation_preview = memory_entity_relation_governance::preview_candidates(
            &path,
            &fixture_m10_preview_input(project_root),
            "2026-06-05T10:30:01Z",
        )
        .expect("secret relation preview should build");
        let causal_candidate = relation_preview
            .relation_candidates
            .iter()
            .find(|candidate| candidate.relation_kind == MemoryRelationKind::Causal)
            .expect("secret causal relation candidate should exist");
        memory_entity_relation_governance::record_relation_decision(
            &path,
            &RecordMemoryRelationCandidateDecisionInput {
                project_root: project_root.to_string(),
                relation_candidate_id: causal_candidate.candidate_id.clone(),
                decision: MemoryRelationCandidateDecisionKind::ConfirmRelation,
                actor_id: "user-m10".to_string(),
                actor_role: "user".to_string(),
                confirmed_by: Some("user".to_string()),
                reason: "确认 secret source 关系，但任务包解释应被权限过滤。".to_string(),
                expected_store_revision: Some(relation_preview.store_revision),
            },
            "2026-06-05T10:30:02Z",
            "write-m10-secret-relation",
        )
        .expect("secret relation can be recorded but not exported");

        let output = preview_task_memory_packet_at(
            &path,
            &fixture_task_memory_packet_input(project_root, "接口"),
            "2026-06-05T10:30:03Z",
        )
        .expect("task packet should build");
        assert!(output.preview.included_memories[0]
            .relation_explanations
            .is_empty());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn memory_entity_relation_damaged_json_and_revision_conflict_are_rejected() {
        let dir = test_temp_dir("memory-entity-relation-guard");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/memory-entity-relation-guard";
        bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        create_formal_memory_for_task(
            &path,
            project_root,
            "实体 relation guard 记忆",
            "用于 revision 和 damaged JSON 测试。",
            "2026-06-05T10:40:00Z",
            "write-m10-guard-formal",
        );
        mutate_formal_store(&path, |store| {
            store.records[0].source_refs = vec![fixture_m10_memory_source(
                "manual_note",
                "manual:guard",
                "guard 文档",
                "project",
            )];
        });
        let preview = memory_entity_relation_governance::preview_candidates(
            &path,
            &fixture_m10_preview_input(project_root),
            "2026-06-05T10:40:01Z",
        )
        .expect("preview should build");
        let entity_candidate = preview
            .entity_candidates
            .first()
            .expect("entity candidate should exist")
            .clone();
        memory_entity_relation_governance::record_alias_decision(
            &path,
            &RecordMemoryEntityAliasDecisionInput {
                project_root: project_root.to_string(),
                entity_candidate_id: entity_candidate.candidate_id.clone(),
                decision: MemoryEntityAliasDecisionKind::ConfirmAlias,
                actor_id: "project-director-m10".to_string(),
                actor_role: "project_director".to_string(),
                reason: "确认登记实体候选。".to_string(),
                expected_store_revision: Some(0),
            },
            "2026-06-05T10:40:02Z",
            "write-m10-alias-confirm",
        )
        .expect("alias decision should write sidecar");
        let conflict = memory_entity_relation_governance::record_alias_decision(
            &path,
            &RecordMemoryEntityAliasDecisionInput {
                project_root: project_root.to_string(),
                entity_candidate_id: entity_candidate.candidate_id,
                decision: MemoryEntityAliasDecisionKind::RejectAlias,
                actor_id: "project-director-m10".to_string(),
                actor_role: "project_director".to_string(),
                reason: "旧 revision 应拒绝。".to_string(),
                expected_store_revision: Some(0),
            },
            "2026-06-05T10:40:03Z",
            "write-m10-alias-conflict",
        )
        .expect_err("stale revision should reject write");
        assert!(conflict.contains("memory_entity_relation_store_conflict"));

        let damaged_dir = test_temp_dir("memory-entity-relation-damaged");
        let damaged_path = damaged_dir.join("workflow-state.v0.json");
        let damaged_sidecar =
            memory_entity_relation_store::sidecar_path(&damaged_path).expect("sidecar path");
        fs::create_dir_all(damaged_sidecar.parent().expect("sidecar parent"))
            .expect("damaged sidecar parent should exist");
        fs::write(&damaged_sidecar, "{not valid json")
            .expect("damaged entity relation sidecar should write");
        let damaged = memory_entity_relation_governance::record_alias_decision(
            &damaged_path,
            &RecordMemoryEntityAliasDecisionInput {
                project_root: project_root.to_string(),
                entity_candidate_id: "entity-candidate:missing".to_string(),
                decision: MemoryEntityAliasDecisionKind::ConfirmAlias,
                actor_id: "project-director-m10".to_string(),
                actor_role: "project_director".to_string(),
                reason: "损坏 JSON 应拒绝覆盖。".to_string(),
                expected_store_revision: None,
            },
            "2026-06-05T10:40:04Z",
            "write-m10-damaged",
        )
        .expect_err("damaged json should reject write");
        assert!(damaged.contains("JSON 损坏"), "{damaged}");
        assert_eq!(
            fs::read_to_string(&damaged_sidecar).expect("damaged sidecar should remain"),
            "{not valid json"
        );

        let _ = fs::remove_dir_all(dir);
        let _ = fs::remove_dir_all(damaged_dir);
    }

    #[test]
    fn memory_candidate_store_keeps_candidates_out_of_formal_memory() {
        let dir =
            std::env::temp_dir().join(format!("memory-candidate-store-{}", unix_timestamp_nanos()));
        fs::create_dir_all(&dir).expect("temp dir should exist");
        let path = dir.join("workflow-state.v0.json");
        let create = CreateMemoryCandidateInput {
            project_root: "/offline-fixture/projects/codex-workbench".to_string(),
            project_id: Some("project:offline".to_string()),
            workflow_id: Some("workflow:offline:default".to_string()),
            scope: MemoryScope {
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
            },
            memory_type: "user_preference".to_string(),
            claim: "用户要求先指出风险。".to_string(),
            body: "这是候选，不是正式长期记忆。".to_string(),
            source_refs: vec![MemorySourceRef {
                source_ref_id: "source:user-confirmed:001".to_string(),
                source_type: "user_confirmed_proposal".to_string(),
                source_id: Some("task:offline".to_string()),
                source_path: None,
                source_title: Some("离线确认".to_string()),
                anchor: None,
                source_created_at: None,
                captured_at: "2026-06-03T00:00:00Z".to_string(),
                authority_level: "user_confirmed".to_string(),
                sensitive_level: "private".to_string(),
                content_hash: None,
            }],
            generated_by_role: "user".to_string(),
            generated_from: "explicit_user_confirmation".to_string(),
            risk_level: "low".to_string(),
            sensitive_level: "private".to_string(),
            requires_user_confirmation: true,
            review_reason: "离线候选治理测试".to_string(),
            expected_store_revision: None,
        };

        let created = memory_candidate_store::create_candidate(
            &path,
            &create,
            "2026-06-03T00:00:00Z",
            "write-memory-001",
        )
        .expect("memory candidate should be created");
        assert_eq!(created.store_revision, 1);
        assert_eq!(
            created.candidate.status,
            MemoryLifecycleStatus::CandidateNeedsReview
        );
        let decided = memory_candidate_store::record_decision(
            &path,
            &RecordMemoryCandidateDecisionInput {
                project_root: create.project_root.clone(),
                candidate_key: created.candidate.candidate_key.clone(),
                requested_status: MemoryLifecycleStatus::CandidateConfirmed,
                reason: "确认保留候选；不写正式记忆。".to_string(),
                actor_id: "user".to_string(),
                actor_role: "user".to_string(),
                expected_store_revision: Some(1),
            },
            "2026-06-03T00:00:01Z",
            "write-memory-002",
        )
        .expect("memory candidate should be confirmed as candidate");
        assert_eq!(decided.store_revision, 2);
        assert_eq!(
            decided.candidate.status,
            MemoryLifecycleStatus::CandidateConfirmed
        );
        assert!(
            !path.exists(),
            "memory candidate write must not create workflow state JSON"
        );
        assert!(path
            .parent()
            .expect("path should have parent")
            .join("memory-candidates.v1.json")
            .exists());
        assert!(!path
            .parent()
            .expect("path should have parent")
            .join("blackboard-candidates.v1.json")
            .exists());

        let formal = memory_candidate_store::record_decision(
            &path,
            &RecordMemoryCandidateDecisionInput {
                project_root: create.project_root,
                candidate_key: decided.candidate.candidate_key,
                requested_status: MemoryLifecycleStatus::MemoryActive,
                reason: "禁止正式晋升测试".to_string(),
                actor_id: "user".to_string(),
                actor_role: "user".to_string(),
                expected_store_revision: Some(2),
            },
            "2026-06-03T00:00:02Z",
            "write-memory-003",
        )
        .unwrap_err();
        assert!(formal.contains("不能请求正式记忆状态"));
    }

    #[test]
    fn candidate_sidecars_are_isolated_and_damaged_json_is_not_overwritten() {
        let dir = std::env::temp_dir().join(format!(
            "candidate-sidecar-isolation-{}",
            unix_timestamp_nanos()
        ));
        fs::create_dir_all(&dir).expect("temp dir should exist");
        let path = dir.join("workflow-state.v0.json");
        let memory_path = dir.join("memory-candidates.v1.json");
        fs::write(&memory_path, "{not valid json").expect("damaged memory sidecar should write");
        let err = memory_candidate_store::load_store(&path, "2026-06-03T00:00:00Z").unwrap_err();
        assert!(err.contains("JSON 损坏"));
        assert_eq!(
            fs::read_to_string(&memory_path).expect("damaged file should remain"),
            "{not valid json"
        );
        let blackboard = blackboard_candidate_store::load_store(&path, "2026-06-03T00:00:00Z")
            .expect("blackboard store should ignore damaged memory sidecar");
        assert_eq!(blackboard.revision, 0);
        assert!(!dir.join("blackboard-candidates.v1.json").exists());
    }

    #[test]
    fn formal_memory_store_creates_record_version_and_audit() {
        let dir = test_temp_dir("formal-memory-create");
        fs::create_dir_all(&dir).expect("temp dir should exist");
        let path = dir.join("workflow-state.v0.json");
        let input = fixture_formal_memory_input();

        let created = formal_memory_store::create_record(
            &path,
            &input,
            "2026-06-03T00:00:00Z",
            "write-formal-001",
        )
        .expect("formal memory should create record, version, and audit");

        assert_eq!(created.store_revision, 1);
        assert_eq!(created.record.status, MemoryLifecycleStatus::MemoryActive);
        assert_eq!(created.version.memory_id, created.record.memory_id);
        assert_eq!(created.version.version_number, 1);
        assert_eq!(created.version.record_snapshot, created.record);
        assert_eq!(created.audit_event.event_type, "memory_record_created");
        assert_eq!(created.audit_event.status, "succeeded");

        let store = formal_memory_store::load_store(&path, "2026-06-03T00:00:01Z")
            .expect("formal memory store should load");
        assert_eq!(store.store_version, "formal_memory_store.v1");
        assert_eq!(store.revision, 1);
        assert_eq!(store.records.len(), 1);
        assert_eq!(store.versions.len(), 1);
        assert_eq!(store.audit_events.len(), 1);
        assert!(dir.join("formal-memories.v1.json").exists());
        assert!(
            !path.exists(),
            "formal memory sidecar must not create workflow state JSON"
        );

        let read_model = formal_memory_store::summarize_store(&store);
        assert_eq!(read_model.sidecar_name, "formal-memories.v1.json");
        assert_eq!(read_model.record_count, 1);
        assert_eq!(read_model.active_count, 1);
        assert_eq!(read_model.version_count, 1);
        assert_eq!(read_model.audit_event_count, 1);
        assert_eq!(
            read_model
                .recent_audit_event
                .expect("recent audit should exist")
                .event_type,
            "memory_record_created"
        );
    }

    #[test]
    fn formal_memory_context_accepts_matching_project_and_workflow() {
        let dir = test_temp_dir("formal-memory-context-accept");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/formal-memory-context-project";
        let project = fixture_project(project_root);
        bootstrap_project_workflow_at(&path, &project)
            .expect("workflow state should include project");
        let input = fixture_bound_formal_memory_input(project_root);

        let created = create_formal_memory_record_at(
            &path,
            &input,
            "2026-06-03T00:00:00Z",
            "write-formal-context-accept",
        )
        .expect("matching context should create formal memory");

        assert_eq!(created.store_revision, 1);
        assert_eq!(
            created.record.scope.project_id.as_deref(),
            input.project_id.as_deref()
        );
        assert_eq!(
            created.record.scope.workflow_id.as_deref(),
            input.workflow_id.as_deref()
        );
        assert!(dir.join("formal-memories.v1.json").exists());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn formal_memory_context_rejects_mismatched_project_id() {
        let dir = test_temp_dir("formal-memory-context-project-mismatch");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/formal-memory-context-project";
        let project = fixture_project(project_root);
        bootstrap_project_workflow_at(&path, &project)
            .expect("workflow state should include project");
        let mut input = fixture_bound_formal_memory_input(project_root);
        input.project_id = Some(project_id("/tmp/other-project"));

        let err = create_formal_memory_record_at(
            &path,
            &input,
            "2026-06-03T00:00:00Z",
            "write-formal-context-project-mismatch",
        )
        .unwrap_err();

        assert!(err.contains("project_id 与 project_root 不匹配"));
        assert!(!dir.join("formal-memories.v1.json").exists());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn formal_memory_context_rejects_mismatched_workflow_id() {
        let dir = test_temp_dir("formal-memory-context-workflow-mismatch");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/formal-memory-context-project";
        let project = fixture_project(project_root);
        bootstrap_project_workflow_at(&path, &project)
            .expect("workflow state should include project");
        let mut input = fixture_bound_formal_memory_input(project_root);
        input.workflow_id = Some(default_workflow_id("/tmp/other-project"));

        let err = create_formal_memory_record_at(
            &path,
            &input,
            "2026-06-03T00:00:00Z",
            "write-formal-context-workflow-mismatch",
        )
        .unwrap_err();

        assert!(err.contains("workflow_id 与 project_root 不匹配"));
        assert!(!dir.join("formal-memories.v1.json").exists());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn formal_memory_context_rejects_project_director_cross_project() {
        let dir = test_temp_dir("formal-memory-context-cross-project");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/formal-memory-context-project";
        let project = fixture_project(project_root);
        bootstrap_project_workflow_at(&path, &project)
            .expect("workflow state should include project");
        let mut input = fixture_bound_formal_memory_input(project_root);
        input.scope.project_id = Some(project_id("/tmp/other-project"));

        let err = create_formal_memory_record_at(
            &path,
            &input,
            "2026-06-03T00:00:00Z",
            "write-formal-context-cross-project",
        )
        .unwrap_err();

        assert!(err.contains("scope.project_id 与 project_root 不匹配"));
        assert!(!dir.join("formal-memories.v1.json").exists());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn formal_memory_context_rejects_missing_project_in_workflow_state() {
        let dir = test_temp_dir("formal-memory-context-missing-state-project");
        let path = dir.join("workflow-state.v0.json");
        bootstrap_project_workflow_at(&path, &fixture_project("/tmp/other-project"))
            .expect("workflow state should include only another project");
        let input = fixture_bound_formal_memory_input("/tmp/formal-memory-context-project");

        let err = create_formal_memory_record_at(
            &path,
            &input,
            "2026-06-03T00:00:00Z",
            "write-formal-context-missing-state-project",
        )
        .unwrap_err();

        assert!(err.contains("workflow state projects[] 不包含 project_root"));
        assert!(!dir.join("formal-memories.v1.json").exists());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn formal_memory_context_keeps_existing_m1_guards() {
        let dir = test_temp_dir("formal-memory-context-keeps-m1");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/formal-memory-context-project";
        let project = fixture_project(project_root);
        bootstrap_project_workflow_at(&path, &project)
            .expect("workflow state should include project");
        let mut input = fixture_bound_formal_memory_input(project_root);
        input.source_refs = vec![];

        let err = create_formal_memory_record_at(
            &path,
            &input,
            "2026-06-03T00:00:00Z",
            "write-formal-context-keeps-m1",
        )
        .unwrap_err();

        assert!(err.contains("正式记忆缺少来源"));
        assert!(!dir.join("formal-memories.v1.json").exists());

        let _ = fs::remove_dir_all(dir);
    }

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

    #[test]
    fn work_item_state_update_rejects_non_index_project() {
        let dir = std::env::temp_dir().join(format!(
            "work-item-state-non-index-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");
        let index = json!({
          "projects": [{ "project_root": "/tmp/indexed-project" }]
        });
        let request = fixture_task_draft_request(&project.project_root, "索引内工作项");

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_task_draft_at(&path, &request).expect("work item should exist");
        let value = read_json_file(&path);
        let work_item_id = optional_string_from(&value["work_items"][0], "work_item_id")
            .expect("work item id should exist");
        let update = fixture_work_item_state_update_request(
            "/tmp/not-indexed",
            &work_item_id,
            "ready_to_dispatch",
        );

        let result = update_work_item_state_for_index_project_at(&path, &index, &update);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("项目不在当前索引内"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn workflow_node_session_binding_binds_rebinds_and_unbinds() {
        let dir =
            std::env::temp_dir().join(format!("node-session-bind-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");
        let draft = fixture_task_draft_request(&project.project_root, "节点绑定工作项");

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_task_draft_at(&path, &draft).expect("work item should exist");
        let value = read_json_file(&path);
        let work_item_id = optional_string_from(&value["work_items"][0], "work_item_id")
            .expect("work item id should exist");
        let workflow_id = default_workflow_id(&project.project_root);
        let node_id = format!("{workflow_id}:node:codex-dev");
        let first_session = fixture_session("thread-001", &project.project_root, true);
        let first_request = fixture_node_session_bind_request(
            &project.project_root,
            &node_id,
            Some(&work_item_id),
            &first_session.thread_id,
        );

        let first = bind_workflow_node_codex_session_at(&path, &first_request, &first_session)
            .expect("binding should write");

        assert_eq!(
            first.snapshot.project_workflows[0]
                .node_session_bindings
                .len(),
            1
        );
        assert_eq!(
            first.snapshot.project_workflows[0].node_session_bindings[0].native_thread_id,
            "thread-001"
        );
        let updated = read_json_file(&path);
        assert_eq!(
            updated["workflow_node_session_bindings"][0]["binding_source"],
            "workflow_bound"
        );
        assert!(updated["audit_events"]
            .as_array()
            .expect("audit events should be array")
            .iter()
            .any(|event| event["event_type"] == "workflow_node_session_bound"));

        let second_session = fixture_session("thread-002", &project.project_root, false);
        let second_request = fixture_node_session_bind_request(
            &project.project_root,
            &node_id,
            Some(&work_item_id),
            &second_session.thread_id,
        );
        let second = bind_workflow_node_codex_session_at(&path, &second_request, &second_session)
            .expect("rebind should write");

        assert_eq!(
            second.snapshot.project_workflows[0]
                .node_session_bindings
                .len(),
            1
        );
        assert_eq!(
            second.snapshot.project_workflows[0].node_session_bindings[0].native_thread_id,
            "thread-002"
        );
        assert_eq!(
            second.snapshot.project_workflows[0].node_session_bindings[0].warnings,
            vec!["index_session_rollout_missing".to_string()]
        );
        let rebound = read_json_file(&path);
        assert!(rebound["audit_events"]
            .as_array()
            .expect("audit events should be array")
            .iter()
            .any(|event| event["event_type"] == "workflow_node_session_rebound"));
        let binding_id =
            optional_string_from(&rebound["workflow_node_session_bindings"][0], "binding_id")
                .expect("binding id should exist");
        let unbind_request =
            fixture_node_session_unbind_request(&project.project_root, &binding_id);
        let unbound = unbind_workflow_node_codex_session_at(&path, &unbind_request)
            .expect("unbind should write");

        assert!(unbound.snapshot.project_workflows[0]
            .node_session_bindings
            .is_empty());
        let detached = read_json_file(&path);
        assert_eq!(
            detached["workflow_node_session_bindings"][0]["lifecycle"],
            "detached"
        );
        assert!(detached["audit_events"]
            .as_array()
            .expect("audit events should be array")
            .iter()
            .any(|event| event["event_type"] == "workflow_node_session_unbound"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn workflow_node_session_binding_rejects_non_index_session_and_missing_node() {
        let dir =
            std::env::temp_dir().join(format!("node-session-reject-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");
        let index = json!({
          "projects": [{ "project_root": "/tmp/indexed-project" }],
          "threads": [{ "thread_id": "thread-001", "project_root": "/tmp/indexed-project", "title": "Indexed" }]
        });
        let workflow_id = default_workflow_id(&project.project_root);
        let node_id = format!("{workflow_id}:node:codex-dev");

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        let missing_session_request =
            fixture_node_session_bind_request(&project.project_root, &node_id, None, "missing");
        let missing_session =
            bind_workflow_node_codex_session_for_index_at(&path, &index, &missing_session_request);

        assert!(missing_session.is_err());
        assert!(missing_session.unwrap_err().contains("会话不在当前索引内"));

        let session = fixture_session("thread-001", &project.project_root, true);
        let missing_node_request = fixture_node_session_bind_request(
            &project.project_root,
            "workflow:missing:node:nope",
            None,
            &session.thread_id,
        );
        let missing_node =
            bind_workflow_node_codex_session_at(&path, &missing_node_request, &session);

        assert!(missing_node.is_err());
        assert!(missing_node.unwrap_err().contains("找不到该 node"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_package_preview_rejects_non_index_project() {
        let dir = std::env::temp_dir().join(format!(
            "task-preview-non-index-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let index = json!({
          "projects": [{ "project_root": "/tmp/indexed-project" }]
        });
        let request = fixture_task_preview_request("/tmp/not-indexed", "work-item:missing");

        initialize_workflow_state_at(&path).expect("state should exist");
        let result = render_task_package_preview_for_index_project_at(&path, &index, &request);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("项目不在当前索引内"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_package_preview_rejects_missing_state_file() {
        let dir = std::env::temp_dir().join(format!(
            "task-preview-missing-state-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");
        let request = fixture_task_preview_request(&project.project_root, "work-item:missing");

        let result = render_task_package_preview_at(&path, &project, &request);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("工作流状态文件不存在"));
        assert!(!path.exists());
    }

    #[test]
    fn task_package_preview_rejects_missing_workflow() {
        let dir = std::env::temp_dir().join(format!(
            "task-preview-missing-workflow-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");
        let request = fixture_task_preview_request(&project.project_root, "work-item:missing");

        initialize_workflow_state_at(&path).expect("state should exist");
        let result = render_task_package_preview_at(&path, &project, &request);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("还没有本地 workflow"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_package_preview_rejects_missing_work_item() {
        let dir = std::env::temp_dir().join(format!(
            "task-preview-missing-work-item-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");
        let request = fixture_task_preview_request(&project.project_root, "work-item:missing");

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        let result = render_task_package_preview_at(&path, &project, &request);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("找不到该 work item"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_package_preview_renders_markdown_from_draft() {
        let dir =
            std::env::temp_dir().join(format!("task-preview-render-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");
        let draft_request = fixture_task_draft_request(&project.project_root, "登记任务包草稿");

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_task_draft_at(&path, &draft_request).expect("task draft should be created");
        let value = read_json_file(&path);
        let work_item_id = optional_string_from(&value["work_items"][0], "work_item_id")
            .expect("work item id should exist");
        let preview_request = fixture_task_preview_request(&project.project_root, &work_item_id);
        let preview = render_task_package_preview_at(&path, &project, &preview_request)
            .expect("preview should render");

        assert_eq!(preview.project_root, project.project_root);
        assert_eq!(preview.work_item_id, work_item_id);
        assert!(preview.markdown.contains("# 任务包：登记任务包草稿"));
        assert!(preview.markdown.contains("## 所属开发线"));
        assert!(preview.markdown.contains("Codex 开发线"));
        assert!(preview.markdown.contains("## 背景"));
        assert!(preview.markdown.contains("## 目标"));
        assert!(preview.markdown.contains("写入 work_items 和 artifacts"));
        assert!(preview.markdown.contains("## 允许读取"));
        assert!(preview.markdown.contains("## 允许写入"));
        assert!(preview.markdown.contains("## 禁止事项"));
        assert!(preview.markdown.contains("## 验收标准"));
        assert!(preview.markdown.contains("## 必须回传"));
        assert!(preview.markdown.contains("## 总指导回收重点"));
        assert!(preview.markdown.contains("待补充"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_package_preview_uses_placeholders_for_missing_fields() {
        let dir = std::env::temp_dir().join(format!(
            "task-preview-placeholders-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        let mut value = read_json_file(&path);
        let workflow_id = default_workflow_id(&project.project_root);
        let work_item_id = format!("work-item:{workflow_id}:manual");
        let artifact_id = format!("artifact:{workflow_id}:manual");
        array_mut(&mut value, "work_items")
            .expect("work_items should exist")
            .push(json!({
              "work_item_id": work_item_id,
              "project_id": project_id(&project.project_root),
              "workflow_id": workflow_id,
              "state": "draft",
              "source_kind": "workspace_state",
              "source_ref": artifact_id
            }));
        array_mut(&mut value, "artifacts")
            .expect("artifacts should exist")
            .push(json!({
              "artifact_id": artifact_id,
              "artifact_type": "task_package",
              "project_id": project_id(&project.project_root),
              "source_kind": "workspace_state",
              "source_ref": work_item_id
            }));
        atomic_write_json(&path, &value).expect("fixture should write");

        let request = fixture_task_preview_request(&project.project_root, &work_item_id);
        let preview = render_task_package_preview_at(&path, &project, &request)
            .expect("preview should render");

        assert!(preview.markdown.contains("# 任务包：待补充"));
        assert!(preview.markdown.contains("未登记"));
        assert!(preview.markdown.contains("业务背景：待补充"));
        assert!(preview
            .warnings
            .iter()
            .any(|warning| warning.contains("任务名未登记")));
        assert!(preview
            .warnings
            .iter()
            .any(|warning| warning.contains("所属开发线未登记")));
        assert!(preview
            .warnings
            .iter()
            .any(|warning| warning.contains("目标说明未登记")));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn workflow_task_package_read_model_derives_v1_objects_from_v0_state() {
        let dir =
            std::env::temp_dir().join(format!("workflow-read-model-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        let tasks_dir = dir.join("tasks");
        let project = fixture_project("/tmp/indexed-project");
        let draft_request = fixture_task_draft_request(&project.project_root, "派生读模型任务");

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_task_draft_at(&path, &draft_request).expect("task draft should be created");
        let value = read_json_file(&path);
        let work_item_id = optional_string_from(&value["work_items"][0], "work_item_id")
            .expect("work item id should exist");
        let mut fields = ready_fields_update_request(&project.project_root, &work_item_id);
        fields.fields.assigned_line = "Codex 开发线".to_string();
        update_task_package_draft_fields_at(&path, &fields).expect("fields should save");
        mark_task_package_fixture_ready(&path, "codex-test-model");
        generate_task_package_file_at(
            &path,
            &project,
            &fixture_task_file_generation_request(&project.project_root, &work_item_id),
            &tasks_dir,
        )
        .expect("file should generate");
        append_fixture_dispatch(
            &path,
            &project.project_root,
            &work_item_id,
            "completed",
            "thread-001",
        );

        let snapshot = read_workflow_state_snapshot(&path).expect("snapshot should read");
        let derived = snapshot.project_workflows[0]
            .derived_workflow
            .as_ref()
            .expect("derived workflow should exist");

        assert_eq!(
            derived.workflow_id,
            default_workflow_id(&project.project_root)
        );
        assert!(!derived.nodes.is_empty());
        assert_eq!(derived.task_packages.len(), 1);
        assert_eq!(derived.task_packages[0].version, 2);
        assert_eq!(
            derived.task_packages[0].model_id.as_deref(),
            Some("codex-test-model")
        );
        assert!(derived.task_packages[0].available_memory_refs.is_empty());
        assert!(derived.task_packages[0].available_knowledge_refs.is_empty());
        assert!(derived
            .ledger_entries
            .iter()
            .any(|entry| entry.entry_type == "task_package_created"));
        assert!(derived
            .warnings
            .iter()
            .any(|warning| warning.contains("derived_from_workflow_state_v0")));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn project_blackboard_read_model_derives_candidates_without_state_promotion() {
        let dir = std::env::temp_dir().join(format!(
            "project-blackboard-read-model-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let tasks_dir = dir.join("tasks");
        let project = fixture_project("/tmp/indexed-project");
        let draft_request = fixture_task_draft_request(&project.project_root, "黑板候选任务");
        let workflow_id = default_workflow_id(&project.project_root);

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_task_draft_at(&path, &draft_request).expect("task draft should be created");
        let value = read_json_file(&path);
        let work_item_id = optional_string_from(&value["work_items"][0], "work_item_id")
            .expect("work item id should exist");
        let mut fields = ready_fields_update_request(&project.project_root, &work_item_id);
        fields.fields.assigned_line = "Codex 开发线".to_string();
        update_task_package_draft_fields_at(&path, &fields).expect("fields should save");
        mark_task_package_fixture_ready(&path, "codex-test-model");
        generate_task_package_file_at(
            &path,
            &project,
            &fixture_task_file_generation_request(&project.project_root, &work_item_id),
            &tasks_dir,
        )
        .expect("file should generate");
        append_fixture_dispatch(
            &path,
            &project.project_root,
            &work_item_id,
            "completed",
            "thread-001",
        );

        let mut value = read_json_file(&path);
        let artifact = value["artifacts"]
            .as_array_mut()
            .expect("artifacts should be array")
            .first_mut()
            .expect("task package artifact should exist");
        artifact["available_memory_refs"] = json!(["memory:candidate:001"]);
        artifact["available_knowledge_refs"] = json!(["knowledge:ref:001"]);
        let dispatch_id_value = {
            let dispatch = value["workflow_node_dispatches"]
                .as_array_mut()
                .expect("dispatches should be array")
                .first_mut()
                .expect("dispatch should exist");
            dispatch["prompt_kind"] = json!("tool_call_summary");
            dispatch["prompt_preview"] = json!("工具摘要，只保留摘要和引用。");
            dispatch["tool_call_ref"] = json!("tool-call:blackboard:001");
            dispatch["warnings"] = json!(["direction_risk_blackboard"]);
            dispatch["dispatch_id"].clone()
        };
        if !value
            .get("permission_requests")
            .is_some_and(Value::is_array)
        {
            value["permission_requests"] = json!([]);
        }
        value["permission_requests"]
            .as_array_mut()
            .expect("permission requests should be array")
            .push(json!({
                "request_id": "permission:blackboard:001",
                "project_id": project_id(&project.project_root),
                "workflow_id": workflow_id,
                "work_item_id": work_item_id,
                "dispatch_id": dispatch_id_value,
                "permission_kind": "write_workflow_state",
                "reason": "需要用户确认是否允许写协议字段。",
                "status": "pending",
                "requested_at": "2026-06-01T00:00:00Z",
                "decided_at": Value::Null,
                "decision": Value::Null,
                "warnings": []
            }));
        write_validated_workflow_state(&path, &value).expect("blackboard fixture should write");

        let snapshot = read_workflow_state_snapshot(&path).expect("snapshot should read");
        let blackboard = snapshot
            .project_blackboards
            .first()
            .expect("project blackboard should be derived");

        assert_eq!(blackboard.project_root, project.project_root);
        assert!(blackboard
            .warnings
            .contains(&"blackboard_promotion_requires_control_core_confirmation".to_string()));
        for kind in [
            BlackboardEntryKind::SubagentReport,
            BlackboardEntryKind::Risk,
            BlackboardEntryKind::PermissionRequest,
            BlackboardEntryKind::ToolSummary,
            BlackboardEntryKind::MemoryCandidate,
            BlackboardEntryKind::KnowledgeRef,
        ] {
            assert!(
                blackboard.entries.iter().any(|entry| entry.kind == kind),
                "blackboard should include {kind:?}: {:?}",
                blackboard.entries
            );
        }
        assert!(blackboard
            .entries
            .iter()
            .all(|entry| entry.status == "candidate"));
        assert!(blackboard
            .entries
            .iter()
            .all(|entry| entry.promotion_decision.status == "candidate_pending_control_core"));
        assert!(blackboard.entries.iter().any(|entry| {
            entry.kind == BlackboardEntryKind::MemoryCandidate
                && entry.promotion_decision.target_kind.as_deref() == Some("formal_memory")
                && entry
                    .warnings
                    .contains(&"memory_candidate_not_formal_memory".to_string())
        }));
        assert!(blackboard.entries.iter().any(|entry| {
            entry.kind == BlackboardEntryKind::KnowledgeRef
                && entry
                    .warnings
                    .contains(&"knowledge_ref_is_not_memory".to_string())
        }));
        assert_eq!(
            project_blackboards_from_workflows(&snapshot.project_workflows),
            snapshot.project_blackboards
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn workflow_run_check_blocks_missing_workflow_and_missing_required_fields() {
        let dir = std::env::temp_dir().join(format!(
            "workflow-run-check-blocked-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");
        let request = WorkflowRunCheckRequest {
            project_root: project.project_root.clone(),
            workflow_id: None,
        };

        let missing = inspect_workflow_run_check_at(&path, &project, &request)
            .expect("missing workflow check should return blocked");
        assert_eq!(missing.status, "blocked");
        assert!(missing
            .blocked_reasons
            .iter()
            .any(|reason| reason.contains("状态文件不存在")));

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_task_draft_at(
            &path,
            &fixture_task_draft_request(&project.project_root, "缺字段任务"),
        )
        .expect("task draft should exist");
        let blocked = inspect_workflow_run_check_at(&path, &project, &request)
            .expect("blocked check should inspect");

        assert_eq!(blocked.status, "blocked");
        for expected in [
            "缺模型",
            "没有读范围",
            "没有写范围",
            "没有验收标准",
            "没有 active 会话绑定",
        ] {
            assert!(
                blocked
                    .blocked_reasons
                    .iter()
                    .any(|reason| reason.contains(expected)),
                "blocked reasons should contain {expected}: {:?}",
                blocked.blocked_reasons
            );
        }

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn workflow_run_check_allows_runnable_fixture_without_auto_filling_optional_refs() {
        let dir = std::env::temp_dir().join(format!(
            "workflow-run-check-runnable-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let tasks_dir = dir.join("tasks");
        let project = fixture_project("/tmp/indexed-project");
        let workflow_id = default_workflow_id(&project.project_root);
        let node_id = format!("{workflow_id}:node:codex-dev");
        let thread_id = "thread-001";
        let index = fixture_dispatch_index(&project.project_root, thread_id);

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_task_draft_at(
            &path,
            &fixture_task_draft_request(&project.project_root, "可运行任务"),
        )
        .expect("task draft should exist");
        let value = read_json_file(&path);
        let work_item_id = optional_string_from(&value["work_items"][0], "work_item_id")
            .expect("work item id should exist");
        let mut fields = ready_fields_update_request(&project.project_root, &work_item_id);
        fields.fields.assigned_line = "Codex 开发线".to_string();
        update_task_package_draft_fields_at(&path, &fields).expect("fields should save");
        mark_task_package_fixture_ready(&path, "codex-test-model");
        generate_task_package_file_at(
            &path,
            &project,
            &fixture_task_file_generation_request(&project.project_root, &work_item_id),
            &tasks_dir,
        )
        .expect("file should generate");
        bind_workflow_node_codex_session_for_index_at(
            &path,
            &index,
            &fixture_node_session_bind_request(
                &project.project_root,
                &node_id,
                Some(&work_item_id),
                thread_id,
            ),
        )
        .expect("binding should write");

        let check = inspect_workflow_run_check_at(
            &path,
            &project,
            &WorkflowRunCheckRequest {
                project_root: project.project_root.clone(),
                workflow_id: None,
            },
        )
        .expect("run check should inspect");

        assert_eq!(check.status, "warning", "{:?}", check.blocked_reasons);
        assert!(check.blocked_reasons.is_empty());
        assert!(check
            .warnings
            .iter()
            .any(|warning| warning.contains("工具白名单为空")
                || warning.contains("harness 要求为空")));
        let snapshot = read_workflow_state_snapshot(&path).expect("snapshot should read");
        let task_package = &snapshot.project_workflows[0]
            .derived_workflow
            .as_ref()
            .expect("derived workflow should exist")
            .task_packages[0];
        assert!(task_package.available_knowledge_refs.is_empty());
        assert!(task_package.available_memory_refs.is_empty());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_package_blocks_missing_report_model_and_stale_after_edit() {
        let dir =
            std::env::temp_dir().join(format!("task-package-stale-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        let tasks_dir = dir.join("tasks");
        let project = fixture_project("/tmp/indexed-project");

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_active_plan_authorization_for_fixture(&path, &project.project_root);
        create_task_draft_at(
            &path,
            &fixture_task_draft_request(&project.project_root, "stale 任务"),
        )
        .expect("task draft should exist");
        let value = read_json_file(&path);
        let work_item_id = optional_string_from(&value["work_items"][0], "work_item_id")
            .expect("work item id should exist");
        update_task_package_draft_fields_at(
            &path,
            &empty_fields_update_request(&project.project_root, &work_item_id),
        )
        .expect("empty fields should save");
        let missing = inspect_task_package_dispatch_readiness_at(
            &path,
            &project,
            &fixture_dispatch_readiness_request(&project.project_root, &work_item_id),
        )
        .expect("missing readiness should inspect");
        assert_eq!(missing.status, "not_ready");
        assert!(missing
            .blocking_reasons
            .iter()
            .any(|reason| reason.contains("缺模型")));
        assert!(missing
            .blocking_reasons
            .iter()
            .any(|reason| reason.contains("report format")));

        update_task_package_draft_fields_at(
            &path,
            &ready_fields_update_request(&project.project_root, &work_item_id),
        )
        .expect("ready fields should save");
        mark_task_package_fixture_ready(&path, "codex-test-model");
        generate_task_package_file_at(
            &path,
            &project,
            &fixture_task_file_generation_request(&project.project_root, &work_item_id),
            &tasks_dir,
        )
        .expect("file should generate");
        let ready = inspect_task_package_dispatch_readiness_at(
            &path,
            &project,
            &fixture_dispatch_readiness_request(&project.project_root, &work_item_id),
        )
        .expect("ready check should inspect");
        assert_eq!(ready.status, "ready");

        let mut changed = ready_fields_update_request(&project.project_root, &work_item_id);
        changed.fields.goals = vec!["人工编辑后必须重新检查。".to_string()];
        update_task_package_draft_fields_at(&path, &changed).expect("edit should mark stale");
        mark_task_package_fixture_ready(&path, "codex-test-model");
        let stale = inspect_task_package_dispatch_readiness_at(
            &path,
            &project,
            &fixture_dispatch_readiness_request(&project.project_root, &work_item_id),
        )
        .expect("stale check should inspect");
        assert_eq!(stale.status, "not_ready");
        assert!(stale
            .blocking_reasons
            .iter()
            .any(|reason| reason.contains("stale")));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_memory_injection_writes_snapshot_to_task_package_artifact() {
        let (dir, path, tasks_dir, project, work_item_id) =
            setup_task_memory_injection_fixture("task-memory-injection-artifact");
        create_formal_memory_for_task(
            &path,
            &project.project_root,
            "派发准备检查必须保留模型显式配置",
            "该正式记忆用于后续 worker 的任务包上下文。",
            "2026-06-04T04:00:00Z",
            "write-m6-artifact-formal",
        );

        let result = generate_task_package_file_at(
            &path,
            &project,
            &fixture_task_file_generation_request(&project.project_root, &work_item_id),
            &tasks_dir,
        )
        .expect("task package should generate with memory snapshot");

        assert_eq!(result.memory_injection_summary.included_count, 1);
        assert_eq!(result.memory_injection_summary.excluded_count, 0);
        assert!(!result.memory_injection_summary.stale);
        let updated = read_json_file(&path);
        let artifact = updated["artifacts"]
            .as_array()
            .expect("artifacts should be array")
            .first()
            .expect("task package artifact should exist");
        let snapshot = artifact
            .get("memory_packet_snapshot")
            .expect("artifact should store frozen memory snapshot");
        let snapshot_id = optional_string_from(snapshot, "snapshot_id")
            .expect("snapshot should include snapshot_id");

        assert_eq!(
            optional_string_from(snapshot, "schema_version").as_deref(),
            Some("task_package_memory_packet_snapshot.v1")
        );
        assert_eq!(
            optional_string_from(snapshot, "retrieval_intent").as_deref(),
            Some("worker_task")
        );
        assert_eq!(
            artifact["memory_packet_fingerprint"],
            snapshot["fingerprint"]
        );
        assert_eq!(
            artifact["memory_packet_generated_at"],
            snapshot["generated_at"]
        );
        assert_eq!(artifact["memory_packet_stale"], false);
        assert_eq!(
            artifact["memory_packet_store_revisions"]["formal_store_revision"],
            json!(1)
        );
        assert_eq!(
            snapshot["included_memories"]
                .as_array()
                .expect("included memories should be array")
                .len(),
            1
        );
        assert!(snapshot["included_memories"][0]["claim"]
            .as_str()
            .unwrap_or("")
            .contains("派发准备检查"));
        assert!(artifact["memory_packet_warnings"]
            .as_array()
            .expect("warnings should be array")
            .iter()
            .any(|warning| warning == "candidate_and_observation_review_materials_only"));
        assert!(updated["audit_events"]
            .as_array()
            .expect("audit events should be array")
            .iter()
            .any(|event| {
                event["event_type"] == "task_memory_packet_injected_into_task_package"
                    && event["reason"]
                        .as_str()
                        .unwrap_or("")
                        .contains(&work_item_id)
                    && event["reason"]
                        .as_str()
                        .unwrap_or("")
                        .contains(&snapshot_id)
                    && event["reason"]
                        .as_str()
                        .unwrap_or("")
                        .contains("included_count=1")
                    && event["reason"]
                        .as_str()
                        .unwrap_or("")
                        .contains("excluded_count=0")
            }));
        let derived_package = &result.snapshot.project_workflows[0]
            .derived_workflow
            .as_ref()
            .expect("derived workflow should exist")
            .task_packages[0];
        assert_eq!(derived_package.memory_injection_summary.included_count, 1);
        assert_eq!(
            derived_package
                .memory_injection_summary
                .snapshot_id
                .as_deref(),
            Some(snapshot_id.as_str())
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_memory_injection_markdown_and_dispatch_prompt_use_same_snapshot() {
        let (dir, path, tasks_dir, project, work_item_id) =
            setup_task_memory_injection_fixture("task-memory-injection-prompt");
        create_formal_memory_for_task(
            &path,
            &project.project_root,
            "派发准备检查必须保留模型显式配置",
            "该正式记忆需要进入任务包 markdown 和派发 prompt。",
            "2026-06-04T04:10:00Z",
            "write-m6-prompt-formal",
        );
        let generated = generate_task_package_file_at(
            &path,
            &project,
            &fixture_task_file_generation_request(&project.project_root, &work_item_id),
            &tasks_dir,
        )
        .expect("task package should generate");
        let markdown = fs::read_to_string(&generated.file_path).expect("markdown should read");
        let state_after_generate = read_json_file(&path);
        let snapshot_id = optional_string_from(
            &state_after_generate["artifacts"][0]["memory_packet_snapshot"],
            "snapshot_id",
        )
        .expect("snapshot id should exist");

        assert!(markdown.contains("## 正式记忆上下文"));
        assert!(markdown.contains(&snapshot_id));
        assert!(markdown.contains("派发准备检查必须保留模型显式配置"));
        assert!(markdown.contains("任务包内容不会回灌成正式记忆"));
        assert!(markdown.contains("候选 / 观察仅作为待审查材料"));
        assert!(!markdown.contains("worker 已收到记忆包"));

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
        let index = fixture_dispatch_index(&project.project_root, "thread-m6-prompt");
        let session = fixture_session("thread-m6-prompt", &project.project_root, true);
        bind_workflow_node_codex_session_at(
            &path,
            &fixture_node_session_bind_request(
                &project.project_root,
                &node_id,
                Some(&work_item_id),
                "thread-m6-prompt",
            ),
            &session,
        )
        .expect("binding should write");

        let prepared = prepare_workflow_node_dispatch_for_index_at(
            &path,
            &index,
            &fixture_dispatch_prepare_request(&project.project_root, &node_id, &work_item_id),
        )
        .expect("prepared dispatch should include memory block");

        assert_eq!(
            prepared.dispatch.memory_packet_snapshot_id.as_deref(),
            Some(snapshot_id.as_str())
        );
        assert!(prepared
            .dispatch
            .prompt_preview
            .contains("## 正式记忆上下文"));
        assert!(prepared.dispatch.prompt_preview.contains(&snapshot_id));
        assert!(prepared
            .dispatch
            .prompt_preview
            .contains("派发准备检查必须保留模型显式配置"));
        assert!(!prepared
            .dispatch
            .prompt_preview
            .contains("worker 已收到记忆包"));
        let updated = read_json_file(&path);
        assert_eq!(
            updated["workflow_node_dispatches"][0]["memory_packet_snapshot_id"],
            snapshot_id
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_memory_injection_marks_snapshot_stale_on_store_revision_change() {
        let (dir, path, tasks_dir, project, work_item_id) =
            setup_task_memory_injection_fixture("task-memory-injection-stale");
        create_formal_memory_for_task(
            &path,
            &project.project_root,
            "派发准备检查必须保留模型显式配置",
            "第一条正式记忆进入任务包。",
            "2026-06-04T04:20:00Z",
            "write-m6-stale-formal-001",
        );
        generate_task_package_file_at(
            &path,
            &project,
            &fixture_task_file_generation_request(&project.project_root, &work_item_id),
            &tasks_dir,
        )
        .expect("task package should generate");
        create_formal_memory_for_task(
            &path,
            &project.project_root,
            "派发准备检查新增了验证边界",
            "正式记忆 store revision 变化后，旧任务包快照必须 stale。",
            "2026-06-04T04:20:01Z",
            "write-m6-stale-formal-002",
        );

        let readiness = inspect_task_package_dispatch_readiness_at(
            &path,
            &project,
            &fixture_dispatch_readiness_request(&project.project_root, &work_item_id),
        )
        .expect("readiness should inspect stale memory snapshot");

        assert_eq!(readiness.status, "not_ready");
        assert!(readiness.memory_injection_summary.stale);
        assert!(readiness
            .memory_injection_summary
            .stale_reasons
            .iter()
            .any(|reason| reason.contains("formal_store_revision")));
        assert!(readiness
            .blocking_reasons
            .iter()
            .any(|reason| reason.contains("记忆快照已 stale")));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_memory_injection_blocks_readiness_when_required_snapshot_missing() {
        let (dir, path, tasks_dir, project, work_item_id) =
            setup_task_memory_injection_fixture("task-memory-injection-missing");
        create_formal_memory_for_task(
            &path,
            &project.project_root,
            "派发准备检查必须保留模型显式配置",
            "先生成完整任务包，再模拟旧 artifact 缺快照。",
            "2026-06-04T04:30:00Z",
            "write-m6-missing-formal",
        );
        generate_task_package_file_at(
            &path,
            &project,
            &fixture_task_file_generation_request(&project.project_root, &work_item_id),
            &tasks_dir,
        )
        .expect("task package should generate");
        let mut value = read_json_file(&path);
        let artifact = value["artifacts"]
            .as_array_mut()
            .expect("artifacts should be array")
            .first_mut()
            .expect("artifact should exist");
        artifact["requires_memory_refs"] = Value::Bool(true);
        artifact["available_memory_refs"] = json!(["memory:required"]);
        artifact["memory_packet_snapshot"] = Value::Null;
        artifact["memory_packet_fingerprint"] = Value::Null;
        artifact["memory_packet_generated_at"] = Value::Null;
        artifact["memory_packet_store_revisions"] = Value::Null;
        artifact["memory_packet_stale"] = Value::Bool(true);
        artifact["memory_packet_warnings"] = json!(["task_memory_packet_snapshot_missing"]);
        write_validated_workflow_state(&path, &value)
            .expect("fixture missing snapshot should write");

        let readiness = inspect_task_package_dispatch_readiness_at(
            &path,
            &project,
            &fixture_dispatch_readiness_request(&project.project_root, &work_item_id),
        )
        .expect("readiness should inspect missing memory snapshot");

        assert_eq!(readiness.status, "not_ready");
        assert_eq!(readiness.memory_injection_summary.included_count, 0);
        assert!(readiness.memory_injection_summary.stale);
        assert!(readiness
            .blocking_reasons
            .iter()
            .any(|reason| reason.contains("任务包声明需要记忆作为依据")
                && reason.contains("记忆快照缺失")));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_memory_injection_excludes_lint_blocked_formal_memory() {
        let (dir, path, tasks_dir, project, work_item_id) =
            setup_task_memory_injection_fixture("task-memory-injection-lint");
        create_formal_memory_with_source(
            &path,
            &project.project_root,
            "接口缓存必须启用",
            "source:lint:m6:blocking",
            "evidence",
            "2026-06-04T04:40:00Z",
            "write-m6-lint-formal",
        );
        let candidate =
            create_confirmed_candidate_with_claim(&path, &project.project_root, "接口缓存禁止启用");
        let mut lint_input = fixture_memory_lint_run_input(
            &project.project_root,
            MemoryLintRunIntent::CandidateAdoptionGuard,
        );
        lint_input.candidate_key = Some(candidate.candidate_key);
        run_memory_lint_at(
            &path,
            &lint_input,
            "2026-06-04T04:40:02Z",
            "write-m6-lint-run",
        )
        .expect("lint should write blocking finding");
        let mut fields = ready_fields_update_request(&project.project_root, &work_item_id);
        fields.fields.goals = vec!["接口缓存后续处理。".to_string()];
        update_task_package_draft_fields_at(&path, &fields)
            .expect("fields should target lint claim");
        mark_task_package_fixture_ready(&path, "codex-test-model");

        let result = generate_task_package_file_at(
            &path,
            &project,
            &fixture_task_file_generation_request(&project.project_root, &work_item_id),
            &tasks_dir,
        )
        .expect("task package should generate even with lint excluded memory");

        assert_eq!(result.memory_injection_summary.included_count, 0);
        let updated = read_json_file(&path);
        let snapshot = &updated["artifacts"][0]["memory_packet_snapshot"];
        assert_eq!(
            snapshot["included_memories"]
                .as_array()
                .expect("included should be array")
                .len(),
            0
        );
        assert!(snapshot["excluded_items"]
            .as_array()
            .expect("excluded should be array")
            .iter()
            .any(|item| item["reason"] == "conflicted"
                && item["detail"]
                    .as_str()
                    .unwrap_or("")
                    .contains("memory lint open blocking finding")));
        assert!(snapshot["warnings"]
            .as_array()
            .expect("warnings should be array")
            .iter()
            .any(|warning| warning == "memory_lint_blocking_findings_excluded"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_package_fields_update_rejects_non_index_project() {
        let dir =
            std::env::temp_dir().join(format!("task-fields-non-index-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        let index = json!({
          "projects": [{ "project_root": "/tmp/indexed-project" }]
        });
        let request = fixture_fields_update_request("/tmp/not-indexed", "work-item:missing");

        initialize_workflow_state_at(&path).expect("state should exist");
        let result = update_task_package_draft_fields_for_index_project_at(&path, &index, &request);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("项目不在当前索引内"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_package_fields_update_rejects_missing_state_file() {
        let dir = std::env::temp_dir().join(format!(
            "task-fields-missing-state-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let request = fixture_fields_update_request("/tmp/indexed-project", "work-item:missing");

        let result = update_task_package_draft_fields_at(&path, &request);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("工作流状态文件不存在"));
        assert!(!path.exists());
    }

    #[test]
    fn task_package_fields_update_rejects_missing_workflow() {
        let dir = std::env::temp_dir().join(format!(
            "task-fields-missing-workflow-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let request = fixture_fields_update_request("/tmp/indexed-project", "work-item:missing");

        initialize_workflow_state_at(&path).expect("state should exist");
        let result = update_task_package_draft_fields_at(&path, &request);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("还没有本地 workflow"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_package_fields_update_rejects_missing_work_item() {
        let dir = std::env::temp_dir().join(format!(
            "task-fields-missing-work-item-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");
        let request = fixture_fields_update_request(&project.project_root, "work-item:missing");

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        let result = update_task_package_draft_fields_at(&path, &request);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("找不到该 work item"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_package_fields_update_rejects_missing_task_package_artifact() {
        let dir = std::env::temp_dir().join(format!(
            "task-fields-missing-artifact-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        let mut value = read_json_file(&path);
        let workflow_id = default_workflow_id(&project.project_root);
        let work_item_id = format!("work-item:{workflow_id}:manual");
        array_mut(&mut value, "work_items")
            .expect("work_items should exist")
            .push(json!({
              "work_item_id": work_item_id,
              "project_id": project_id(&project.project_root),
              "workflow_id": workflow_id,
              "title": "没有 artifact 的草稿",
              "state": "draft",
              "source_kind": "workspace_state",
              "source_ref": "artifact:missing"
            }));
        atomic_write_json(&path, &value).expect("fixture should write");
        let request = fixture_fields_update_request(&project.project_root, &work_item_id);

        let result = update_task_package_draft_fields_at(&path, &request);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("找不到 task_package artifact"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_package_fields_update_writes_structured_fields_backup_and_audit() {
        let dir =
            std::env::temp_dir().join(format!("task-fields-update-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");
        let draft_request = fixture_task_draft_request(&project.project_root, "旧标题");

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_task_draft_at(&path, &draft_request).expect("task draft should be created");
        let before_text = fs::read_to_string(&path).expect("state should be readable");
        let value = read_json_file(&path);
        let work_item_id = optional_string_from(&value["work_items"][0], "work_item_id")
            .expect("work item id should exist");
        let request = fixture_fields_update_request(&project.project_root, &work_item_id);

        let result = update_task_package_draft_fields_at(&path, &request)
            .expect("fields update should write");
        let backup_path = result
            .backup_path
            .expect("fields update should back up old state");
        assert!(PathBuf::from(&backup_path).exists());
        let backup_text = fs::read_to_string(backup_path).expect("backup should be readable");
        assert_eq!(backup_text, before_text);

        let updated = read_json_file(&path);
        assert_eq!(updated["work_items"][0]["title"], "字段编辑任务");
        assert_eq!(updated["work_items"][0]["assigned_role_id"], "desktop-app");
        assert_eq!(updated["artifacts"][0]["task_name"], "字段编辑任务");
        assert_eq!(updated["artifacts"][0]["assigned_line"], "桌面应用线");
        assert_eq!(
            updated["artifacts"][0]["template_version"],
            "task_package_v1"
        );
        assert_eq!(updated["artifacts"][0]["path"], Value::Null);
        assert_eq!(updated["artifacts"][0]["background"][0], "来自结构化字段。");
        assert!(updated["audit_events"]
            .as_array()
            .expect("audit events should be array")
            .iter()
            .any(|event| event["event_type"] == "task_package_fields_updated"));

        let preview_request = fixture_task_preview_request(&project.project_root, &work_item_id);
        let preview = render_task_package_preview_at(&path, &project, &preview_request)
            .expect("preview should render updated fields");
        assert!(preview.markdown.contains("# 任务包：字段编辑任务"));
        assert!(preview.markdown.contains("桌面应用线"));
        assert!(preview.markdown.contains("- 来自结构化字段。"));
        assert!(preview.markdown.contains("- 完成字段编辑。"));
        assert!(preview.markdown.contains("- /tmp/indexed-project"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_package_fields_update_keeps_empty_fields_as_missing_facts() {
        let dir =
            std::env::temp_dir().join(format!("task-fields-empty-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");
        let draft_request = fixture_task_draft_request(&project.project_root, "旧标题");

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_task_draft_at(&path, &draft_request).expect("task draft should be created");
        let value = read_json_file(&path);
        let work_item_id = optional_string_from(&value["work_items"][0], "work_item_id")
            .expect("work item id should exist");
        let request = empty_fields_update_request(&project.project_root, &work_item_id);

        update_task_package_draft_fields_at(&path, &request)
            .expect("empty fields should still save");
        let updated = read_json_file(&path);
        assert_eq!(updated["artifacts"][0]["task_name"], "");
        assert_eq!(
            updated["artifacts"][0]["background"]
                .as_array()
                .unwrap()
                .len(),
            0
        );
        assert!(updated["artifacts"][0]["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|warning| warning == "missing_task_name"));

        let preview_request = fixture_task_preview_request(&project.project_root, &work_item_id);
        let preview = render_task_package_preview_at(&path, &project, &preview_request)
            .expect("preview should render placeholders");
        assert!(preview.markdown.contains("# 任务包：待补充"));
        assert!(preview.markdown.contains("未登记"));
        assert!(preview.markdown.contains("待补充"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_package_file_generation_rejects_non_index_project() {
        let dir =
            std::env::temp_dir().join(format!("task-file-non-index-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        let tasks_dir = dir.join("tasks");
        let index = json!({
          "projects": [{ "project_root": "/tmp/indexed-project" }]
        });
        let request = fixture_task_file_generation_request("/tmp/not-indexed", "work-item:missing");

        initialize_workflow_state_at(&path).expect("state should exist");
        let result =
            generate_task_package_file_for_index_project_at(&path, &index, &request, &tasks_dir);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("项目不在当前索引内"));
        assert!(!tasks_dir.exists());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_package_file_generation_rejects_missing_state_file() {
        let dir = std::env::temp_dir().join(format!(
            "task-file-missing-state-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let tasks_dir = dir.join("tasks");
        let project = fixture_project("/tmp/indexed-project");
        let request =
            fixture_task_file_generation_request(&project.project_root, "work-item:missing");

        let result = generate_task_package_file_at(&path, &project, &request, &tasks_dir);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("工作流状态文件不存在"));
        assert!(!path.exists());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_package_file_generation_rejects_missing_workflow() {
        let dir = std::env::temp_dir().join(format!(
            "task-file-missing-workflow-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let tasks_dir = dir.join("tasks");
        let project = fixture_project("/tmp/indexed-project");
        let request =
            fixture_task_file_generation_request(&project.project_root, "work-item:missing");

        initialize_workflow_state_at(&path).expect("state should exist");
        let result = generate_task_package_file_at(&path, &project, &request, &tasks_dir);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("还没有本地 workflow"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_package_file_generation_rejects_missing_work_item() {
        let dir = std::env::temp_dir().join(format!(
            "task-file-missing-work-item-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let tasks_dir = dir.join("tasks");
        let project = fixture_project("/tmp/indexed-project");
        let request =
            fixture_task_file_generation_request(&project.project_root, "work-item:missing");

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        let result = generate_task_package_file_at(&path, &project, &request, &tasks_dir);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("找不到该 work item"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_package_file_generation_rejects_missing_task_package_artifact() {
        let dir = std::env::temp_dir().join(format!(
            "task-file-missing-artifact-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let tasks_dir = dir.join("tasks");
        let project = fixture_project("/tmp/indexed-project");

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        let mut value = read_json_file(&path);
        let workflow_id = default_workflow_id(&project.project_root);
        let work_item_id = format!("work-item:{workflow_id}:manual");
        array_mut(&mut value, "work_items")
            .expect("work_items should exist")
            .push(json!({
              "work_item_id": work_item_id,
              "project_id": project_id(&project.project_root),
              "workflow_id": workflow_id,
              "title": "没有 artifact 的草稿",
              "state": "draft",
              "source_kind": "workspace_state",
              "source_ref": "artifact:missing"
            }));
        atomic_write_json(&path, &value).expect("fixture should write");
        let request = fixture_task_file_generation_request(&project.project_root, &work_item_id);

        let result = generate_task_package_file_at(&path, &project, &request, &tasks_dir);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("找不到 task_package artifact"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_package_file_generation_writes_file_updates_artifact_and_audit() {
        let dir =
            std::env::temp_dir().join(format!("task-file-generate-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        let tasks_dir = dir.join("tasks");
        let project = fixture_project("/tmp/indexed-project");
        let draft_request = fixture_task_draft_request(&project.project_root, "旧标题");

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_task_draft_at(&path, &draft_request).expect("task draft should be created");
        let value = read_json_file(&path);
        let work_item_id = optional_string_from(&value["work_items"][0], "work_item_id")
            .expect("work item id should exist");
        let fields_request = fixture_fields_update_request(&project.project_root, &work_item_id);
        update_task_package_draft_fields_at(&path, &fields_request)
            .expect("fields should be saved before generation");
        let before_text = fs::read_to_string(&path).expect("state should be readable");
        let request = fixture_task_file_generation_request(&project.project_root, &work_item_id);

        let result = generate_task_package_file_at(&path, &project, &request, &tasks_dir)
            .expect("file generation should write");

        let file_path = PathBuf::from(&result.file_path);
        assert!(file_path.exists());
        assert!(file_path.starts_with(&tasks_dir));
        assert!(file_path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("2026-05-29-generated-"));
        let markdown = fs::read_to_string(&file_path).expect("generated file should be readable");
        assert!(markdown.contains("# 任务包：字段编辑任务"));
        assert!(markdown.contains("## 目标"));
        assert!(markdown.contains("- 完成字段编辑。"));
        assert!(markdown.contains("## 禁止事项"));
        assert!(markdown.contains("- 不生成真实任务文件。"));
        assert!(markdown.contains("待补充") || !markdown.contains("{{"));

        let updated = read_json_file(&path);
        assert_eq!(updated["artifacts"][0]["path"], result.file_path);
        assert!(updated["artifacts"][0]["updated_at"].as_str().is_some());
        assert!(!updated["artifacts"][0]["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|warning| warning == "draft_only_no_markdown_file"));
        assert!(updated["audit_events"]
            .as_array()
            .expect("audit events should be array")
            .iter()
            .any(|event| event["event_type"] == "task_package_file_generated"
                && event["target_ref"] == work_item_id));
        let backup_text =
            fs::read_to_string(&result.backup_path).expect("backup should be readable");
        assert_eq!(backup_text, before_text);
        assert_eq!(
            result.snapshot.project_workflows[0].task_drafts[0]
                .artifact_path
                .as_deref(),
            Some(result.file_path.as_str())
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_package_file_generation_uses_suffix_without_overwriting_existing_file() {
        let dir =
            std::env::temp_dir().join(format!("task-file-conflict-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        let tasks_dir = dir.join("tasks");
        let project = fixture_project("/tmp/indexed-project");
        let draft_request = fixture_task_draft_request(&project.project_root, "旧标题");

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_task_draft_at(&path, &draft_request).expect("task draft should be created");
        let value = read_json_file(&path);
        let work_item_id = optional_string_from(&value["work_items"][0], "work_item_id")
            .expect("work item id should exist");
        let fields_request = fixture_fields_update_request(&project.project_root, &work_item_id);
        update_task_package_draft_fields_at(&path, &fields_request)
            .expect("fields should be saved before generation");

        fs::create_dir_all(&tasks_dir).expect("tasks fixture dir should exist");
        let conflict = next_available_task_package_path(&tasks_dir, "字段编辑任务", &work_item_id)
            .expect("first generated path should be calculable");
        fs::write(&conflict, "existing file").expect("conflict fixture should write");
        let request = fixture_task_file_generation_request(&project.project_root, &work_item_id);

        let result = generate_task_package_file_at(&path, &project, &request, &tasks_dir)
            .expect("file generation should use suffix");

        assert_eq!(
            fs::read_to_string(&conflict).expect("conflict file should remain"),
            "existing file"
        );
        assert!(result.file_path.ends_with("-2.md"));
        assert_ne!(result.file_path, conflict.display().to_string());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_package_file_generation_keeps_missing_fields_as_placeholders() {
        let dir = std::env::temp_dir().join(format!(
            "task-file-placeholders-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let tasks_dir = dir.join("tasks");
        let project = fixture_project("/tmp/indexed-project");

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        let mut value = read_json_file(&path);
        let workflow_id = default_workflow_id(&project.project_root);
        let work_item_id = format!("work-item:{workflow_id}:manual");
        let artifact_id = format!("artifact:{workflow_id}:manual");
        array_mut(&mut value, "work_items")
            .expect("work_items should exist")
            .push(json!({
              "work_item_id": work_item_id,
              "project_id": project_id(&project.project_root),
              "workflow_id": workflow_id,
              "state": "draft",
              "source_kind": "workspace_state",
              "source_ref": artifact_id
            }));
        array_mut(&mut value, "artifacts")
            .expect("artifacts should exist")
            .push(json!({
              "artifact_id": artifact_id,
              "artifact_type": "task_package",
              "project_id": project_id(&project.project_root),
              "source_kind": "workspace_state",
              "source_ref": work_item_id,
              "warnings": ["missing_task_name"]
            }));
        atomic_write_json(&path, &value).expect("fixture should write");
        let request = fixture_task_file_generation_request(&project.project_root, &work_item_id);

        let result = generate_task_package_file_at(&path, &project, &request, &tasks_dir)
            .expect("file generation should keep placeholders");
        let markdown = fs::read_to_string(result.file_path).expect("generated file should read");

        assert!(markdown.contains("# 任务包：待补充"));
        assert!(markdown.contains("未登记"));
        assert!(markdown.contains("业务背景：待补充"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_package_dispatch_readiness_flags_polluted_generated_draft_as_not_ready() {
        let dir =
            std::env::temp_dir().join(format!("dispatch-polluted-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");
        let tasks_dir = dir.join("tasks");
        let draft_request = TaskDraftRequest {
            project_root: project.project_root.clone(),
            title: "task draft他日smoke".to_string(),
            objective: "待补充：输入法污染他日".to_string(),
            assigned_role: Some("codex-dev".to_string()),
        };

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_active_plan_authorization_for_fixture(&path, &project.project_root);
        create_task_draft_at(&path, &draft_request).expect("task draft should exist");
        let value = read_json_file(&path);
        let work_item_id = optional_string_from(&value["work_items"][0], "work_item_id")
            .expect("work item id should exist");
        generate_task_package_file_at(
            &path,
            &project,
            &fixture_task_file_generation_request(&project.project_root, &work_item_id),
            &tasks_dir,
        )
        .expect("polluted fixture file should generate");
        let readiness = inspect_task_package_dispatch_readiness_at(
            &path,
            &project,
            &fixture_dispatch_readiness_request(&project.project_root, &work_item_id),
        )
        .expect("readiness should inspect");

        assert_eq!(readiness.status, "not_ready");
        assert!(!readiness.can_generate_next_version);
        assert!(readiness
            .blocking_reasons
            .iter()
            .any(|reason| reason.contains("测试草稿")));
        assert!(readiness
            .blocking_reasons
            .iter()
            .any(|reason| reason.contains("目标")));
        assert!(readiness
            .blocking_reasons
            .iter()
            .any(|reason| reason.contains("历史禁令")));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_package_dispatch_readiness_rejects_missing_fields() {
        let dir =
            std::env::temp_dir().join(format!("dispatch-missing-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");
        let draft_request = fixture_task_draft_request(&project.project_root, "待补充");

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_task_draft_at(&path, &draft_request).expect("task draft should exist");
        let value = read_json_file(&path);
        let work_item_id = optional_string_from(&value["work_items"][0], "work_item_id")
            .expect("work item id should exist");
        update_task_package_draft_fields_at(
            &path,
            &empty_fields_update_request(&project.project_root, &work_item_id),
        )
        .expect("empty fields should save");
        let readiness = inspect_task_package_dispatch_readiness_at(
            &path,
            &project,
            &fixture_dispatch_readiness_request(&project.project_root, &work_item_id),
        )
        .expect("readiness should inspect");

        assert_eq!(readiness.status, "not_ready");
        assert!(readiness
            .blocking_reasons
            .iter()
            .any(|reason| reason.contains("尚未生成")));
        assert!(readiness
            .blocking_reasons
            .iter()
            .any(|reason| reason.contains("允许写入")));
        assert!(readiness
            .blocking_reasons
            .iter()
            .any(|reason| reason.contains("验收标准")));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_package_dispatch_readiness_rejects_conflicting_generation_ban() {
        let dir = std::env::temp_dir().join(format!(
            "dispatch-conflicting-ban-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");
        let tasks_dir = dir.join("tasks");
        let draft_request = fixture_task_draft_request(&project.project_root, "旧标题");

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_active_plan_authorization_for_fixture(&path, &project.project_root);
        create_task_draft_at(&path, &draft_request).expect("task draft should exist");
        let value = read_json_file(&path);
        let work_item_id = optional_string_from(&value["work_items"][0], "work_item_id")
            .expect("work item id should exist");
        update_task_package_draft_fields_at(
            &path,
            &ready_fields_update_request_with_forbidden(
                &project.project_root,
                &work_item_id,
                vec!["不生成真实任务包文件。".to_string()],
            ),
        )
        .expect("fields should save");
        generate_task_package_file_at(
            &path,
            &project,
            &fixture_task_file_generation_request(&project.project_root, &work_item_id),
            &tasks_dir,
        )
        .expect("file should generate");
        let readiness = inspect_task_package_dispatch_readiness_at(
            &path,
            &project,
            &fixture_dispatch_readiness_request(&project.project_root, &work_item_id),
        )
        .expect("readiness should inspect");

        assert_eq!(readiness.status, "not_ready");
        assert!(readiness
            .blocking_reasons
            .iter()
            .any(|reason| reason.contains("历史禁令")));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_package_dispatch_readiness_can_be_ready_after_field_fix_and_file_generation() {
        let dir = std::env::temp_dir().join(format!("dispatch-ready-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");
        let tasks_dir = dir.join("tasks");
        let draft_request = fixture_task_draft_request(&project.project_root, "旧标题");

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_active_plan_authorization_for_fixture(&path, &project.project_root);
        create_task_draft_at(&path, &draft_request).expect("task draft should exist");
        let value = read_json_file(&path);
        let work_item_id = optional_string_from(&value["work_items"][0], "work_item_id")
            .expect("work item id should exist");
        update_task_package_draft_fields_at(
            &path,
            &ready_fields_update_request(&project.project_root, &work_item_id),
        )
        .expect("fields should save");
        mark_task_package_fixture_ready(&path, "codex-test-model");
        let first = generate_task_package_file_at(
            &path,
            &project,
            &fixture_task_file_generation_request(&project.project_root, &work_item_id),
            &tasks_dir,
        )
        .expect("file should generate");
        let readiness = inspect_task_package_dispatch_readiness_at(
            &path,
            &project,
            &fixture_dispatch_readiness_request(&project.project_root, &work_item_id),
        )
        .expect("readiness should inspect");

        assert_eq!(
            readiness.status, "ready",
            "{:?}",
            readiness.blocking_reasons
        );
        assert!(readiness.can_generate_next_version);
        assert!(readiness.blocking_reasons.is_empty());
        assert_eq!(
            readiness.artifact_path.as_deref(),
            Some(first.file_path.as_str())
        );

        let mut next_fields = ready_fields_update_request(&project.project_root, &work_item_id);
        next_fields.fields.task_name = "派发准备检查任务新版".to_string();
        next_fields.fields.goals = vec!["生成修正后的可派发版本。".to_string()];
        update_task_package_draft_fields_at(&path, &next_fields).expect("next fields should save");
        mark_task_package_fixture_ready(&path, "codex-test-model");
        let second = generate_task_package_file_at(
            &path,
            &project,
            &fixture_task_file_generation_request(&project.project_root, &work_item_id),
            &tasks_dir,
        )
        .expect("next file should not overwrite old file");
        assert_ne!(first.file_path, second.file_path);
        assert!(PathBuf::from(&first.file_path).exists());
        assert!(PathBuf::from(&second.file_path).exists());
        let updated = read_json_file(&path);
        assert_eq!(updated["artifacts"][0]["path"], second.file_path);
        assert!(updated["audit_events"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| event["event_type"] == "task_package_file_generated"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn dispatch_field_correction_rejects_non_index_project() {
        let dir =
            std::env::temp_dir().join(format!("correction-non-index-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        let index = json!({
          "projects": [{ "project_root": "/tmp/indexed-project" }]
        });
        let request = fixture_dispatch_correction_request("/tmp/not-indexed", "work-item:missing");

        initialize_workflow_state_at(&path).expect("state should exist");
        let result =
            correct_task_package_dispatch_fields_for_index_project_at(&path, &index, &request);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("项目不在当前索引内"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn dispatch_field_correction_rejects_missing_state_file() {
        let dir = std::env::temp_dir().join(format!(
            "correction-missing-state-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let request =
            fixture_dispatch_correction_request("/tmp/indexed-project", "work-item:missing");
        let update_request = TaskPackageFieldsUpdateRequest {
            project_root: request.project_root,
            work_item_id: request.work_item_id,
            fields: request.fields,
        };

        let result = update_task_package_fields_at(
            &path,
            &update_request,
            TaskPackageFieldWriteMode::DispatchCorrection,
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("工作流状态文件不存在"));
        assert!(!path.exists());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn dispatch_field_correction_rejects_missing_workflow_work_item_and_artifact() {
        let dir = std::env::temp_dir().join(format!(
            "correction-missing-parts-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");
        let missing_work_item =
            fixture_dispatch_correction_request(&project.project_root, "work-item:missing");
        let missing_work_item_update = TaskPackageFieldsUpdateRequest {
            project_root: missing_work_item.project_root.clone(),
            work_item_id: missing_work_item.work_item_id.clone(),
            fields: missing_work_item.fields.clone(),
        };

        initialize_workflow_state_at(&path).expect("state should exist");
        let missing_workflow = update_task_package_fields_at(
            &path,
            &missing_work_item_update,
            TaskPackageFieldWriteMode::DispatchCorrection,
        );
        assert!(missing_workflow.is_err());
        assert!(missing_workflow
            .unwrap_err()
            .contains("还没有本地 workflow"));

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        let missing_item = update_task_package_fields_at(
            &path,
            &missing_work_item_update,
            TaskPackageFieldWriteMode::DispatchCorrection,
        );
        assert!(missing_item.is_err());
        assert!(missing_item.unwrap_err().contains("找不到该 work item"));

        let mut value = read_json_file(&path);
        let workflow_id = default_workflow_id(&project.project_root);
        let work_item_id = format!("work-item:{workflow_id}:manual");
        array_mut(&mut value, "work_items")
            .expect("work_items should exist")
            .push(json!({
              "work_item_id": work_item_id,
              "project_id": project_id(&project.project_root),
              "workflow_id": workflow_id,
              "title": "没有 artifact 的草稿",
              "state": "draft",
              "source_kind": "workspace_state",
              "source_ref": "artifact:missing"
            }));
        atomic_write_json(&path, &value).expect("fixture should write");
        let missing_artifact_request =
            fixture_dispatch_correction_request(&project.project_root, &work_item_id);
        let missing_artifact_update = TaskPackageFieldsUpdateRequest {
            project_root: missing_artifact_request.project_root,
            work_item_id: missing_artifact_request.work_item_id,
            fields: missing_artifact_request.fields,
        };
        let missing_artifact = update_task_package_fields_at(
            &path,
            &missing_artifact_update,
            TaskPackageFieldWriteMode::DispatchCorrection,
        );
        assert!(missing_artifact.is_err());
        assert!(missing_artifact
            .unwrap_err()
            .contains("找不到 task_package artifact"));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn dispatch_field_correction_backs_up_writes_audit_keeps_path_and_rechecks_ready() {
        let dir = std::env::temp_dir().join(format!("correction-save-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        let tasks_dir = dir.join("tasks");
        let project = fixture_project("/tmp/indexed-project");
        let draft_request = fixture_task_draft_request(&project.project_root, "旧标题");

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_active_plan_authorization_for_fixture(&path, &project.project_root);
        create_task_draft_at(&path, &draft_request).expect("task draft should exist");
        let value = read_json_file(&path);
        let work_item_id = optional_string_from(&value["work_items"][0], "work_item_id")
            .expect("work item id should exist");
        generate_task_package_file_at(
            &path,
            &project,
            &fixture_task_file_generation_request(&project.project_root, &work_item_id),
            &tasks_dir,
        )
        .expect("existing generated path should exist");
        mark_task_package_fixture_ready(&path, "codex-test-model");
        let before_text = fs::read_to_string(&path).expect("state should be readable");
        let before = read_json_file(&path);
        let old_path =
            optional_string_from(&before["artifacts"][0], "path").expect("path should exist");
        let request = fixture_dispatch_correction_request(&project.project_root, &work_item_id);
        let update_request = TaskPackageFieldsUpdateRequest {
            project_root: request.project_root,
            work_item_id: request.work_item_id,
            fields: request.fields,
        };

        let result = update_task_package_fields_at(
            &path,
            &update_request,
            TaskPackageFieldWriteMode::DispatchCorrection,
        )
        .expect("correction should save");
        let backup_text =
            fs::read_to_string(&result.backup_path.unwrap()).expect("backup should be readable");
        assert_eq!(backup_text, before_text);

        let updated = read_json_file(&path);
        assert_eq!(updated["artifacts"][0]["path"], old_path);
        assert_eq!(updated["artifacts"][0]["task_name"], "派发准备检查任务");
        updated["artifacts"][0]["model_id"]
            .as_str()
            .expect("fixture model should remain explicit");
        assert!(updated["audit_events"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| event["event_type"] == "task_package_fields_corrected_for_dispatch"));

        let readiness = inspect_task_package_dispatch_readiness_at(
            &path,
            &project,
            &fixture_dispatch_readiness_request(&project.project_root, &work_item_id),
        )
        .expect("readiness should inspect after save");
        assert_eq!(readiness.status, "not_ready");
        assert!(readiness
            .blocking_reasons
            .iter()
            .any(|reason| reason.contains("stale")));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn dispatch_field_correction_keeps_empty_fields_missing() {
        let dir =
            std::env::temp_dir().join(format!("correction-empty-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");
        let draft_request = fixture_task_draft_request(&project.project_root, "旧标题");

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_task_draft_at(&path, &draft_request).expect("task draft should exist");
        let value = read_json_file(&path);
        let work_item_id = optional_string_from(&value["work_items"][0], "work_item_id")
            .expect("work item id should exist");
        let empty_update = empty_fields_update_request(&project.project_root, &work_item_id);
        update_task_package_fields_at(
            &path,
            &empty_update,
            TaskPackageFieldWriteMode::DispatchCorrection,
        )
        .expect("empty correction should save without inventing");

        let updated = read_json_file(&path);
        assert_eq!(updated["artifacts"][0]["task_name"], "");
        assert!(updated["artifacts"][0]["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|warning| warning == "missing_task_name"));
        let readiness = inspect_task_package_dispatch_readiness_at(
            &path,
            &project,
            &fixture_dispatch_readiness_request(&project.project_root, &work_item_id),
        )
        .expect("readiness should inspect after empty save");
        assert_eq!(readiness.status, "not_ready");

        let _ = fs::remove_dir_all(dir);
    }

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

    #[test]
    fn workflow_ledger_derives_summary_entries_without_tool_output_fulltext() {
        let workflow_id = default_workflow_id("/tmp/indexed-project");
        let long_tool_output = "工具输出全文".repeat(80);
        let audit_events = vec![json!({
          "event_id": "audit:tool-summary:001",
          "event_type": "tool_call_summary",
          "target_ref": workflow_id,
          "actor_ref": "codex-dev",
          "reason": "工具调用摘要：读取了允许范围内的 README。",
          "tool_output_fulltext": long_tool_output,
          "risk_flags": ["allowed_scope_only"],
          "created_at": "2026-06-01T00:00:00Z"
        })];
        let dispatches = vec![json!({
          "dispatch_id": "dispatch:tool:001",
          "workflow_id": workflow_id,
          "node_id": format!("{}:node:codex-dev", default_workflow_id("/tmp/indexed-project")),
          "work_item_id": "work-item:001",
          "native_thread_id": "thread-001",
          "prompt_kind": "tool_call_summary",
          "prompt_preview": format!("摘要，不是全文。{long_tool_output}"),
          "tool_call_ref": "tool-call:001",
          "warnings": ["tool_output_trimmed"]
        })];
        let entries =
            derive_workflow_ledger_entries(&workflow_id, &audit_events, &dispatches, &[], &[]);

        assert!(entries
            .iter()
            .any(|entry| entry.entry_type == "tool_call_summary"));
        assert!(entries
            .iter()
            .any(|entry| entry.tool_call_refs == vec!["tool-call:001".to_string()]));
        assert!(entries
            .iter()
            .all(|entry| !entry.summary.contains(&long_tool_output)));
        assert!(entries
            .iter()
            .any(|entry| entry.audit_refs == vec!["audit:tool-summary:001".to_string()]));
    }

    #[test]
    fn subagent_report_derives_required_fields_and_direction_risk() {
        let workflow_id = default_workflow_id("/tmp/indexed-project");
        let dispatches = vec![json!({
          "dispatch_id": "dispatch:report:001",
          "workflow_id": workflow_id,
          "node_id": format!("{workflow_id}:node:codex-dev"),
          "work_item_id": "work-item:001",
          "native_thread_id": "thread-001",
          "prompt_preview": "执行：修改 README。",
          "state": "completed",
          "last_message_summary": "改了 README，发现 direction risk。",
          "last_message_path": "/tmp/report.md",
          "warnings": ["direction_risk:需求冲突"],
          "follow_up_suggestions": ["请项目主管裁决方向。"],
          "acceptance_status": "reported_not_completed"
        })];
        let permission_requests = vec![json!({
          "request_id": "permission:001",
          "workflow_id": workflow_id,
          "work_item_id": "work-item:001",
          "status": "pending",
          "reason": "需要写入 README。",
          "requested_at": "2026-06-01T00:00:00Z"
        })];
        let reports = derive_subagent_reports(&workflow_id, &dispatches, &[], &permission_requests);

        assert_eq!(reports.len(), 1);
        let report = &reports[0];
        assert_eq!(report.actor_role.as_deref(), Some("codex-dev"));
        assert!(report.executed_what.contains("修改 README"));
        assert!(report.changed_what.contains("改了 README"));
        assert_eq!(report.evidence_refs, vec!["/tmp/report.md".to_string()]);
        assert_eq!(
            report.permission_requests,
            vec!["permission:001".to_string()]
        );
        assert_eq!(
            report.direction_risks,
            vec!["direction_risk:需求冲突".to_string()]
        );
        assert_eq!(report.acceptance_status, "reported_not_completed");
    }

    #[test]
    fn review_result_cannot_directly_complete_node() {
        let workflow_id = default_workflow_id("/tmp/indexed-project");
        let reviews = vec![json!({
          "review_id": "review:001",
          "workflow_id": workflow_id,
          "workflow_node_id": format!("{workflow_id}:node:review"),
          "decision": "accepted",
          "summary": "审查通过。",
          "evidence_refs": ["/tmp/evidence.md"],
          "warnings": []
        })];
        let results = derive_review_results(&workflow_id, &reviews);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].result, "passed");
        assert!(results[0].requires_director_confirmation);
        assert!(!results[0].can_complete_node);
        assert!(results[0]
            .warnings
            .contains(&"review_passed_but_director_still_confirms_node_completion".to_string()));
    }

    #[test]
    fn workflow_exception_detects_timeout_permission_review_direction_and_harness() {
        let workflow_id = default_workflow_id("/tmp/indexed-project");
        let artifacts = vec![json!({
          "artifact_id": "artifact:001",
          "artifact_type": "task_package",
          "workflow_id": workflow_id,
          "unresolved_direction_risk": true,
          "harness_blocked": true,
          "warnings": ["fixture"]
        })];
        let permission_requests = vec![json!({
          "request_id": "permission:001",
          "workflow_id": workflow_id,
          "work_item_id": "work-item:001",
          "status": "pending",
          "reason": "等待权限。",
          "requested_at": "2026-06-01T00:00:00Z"
        })];
        let attempts = vec![json!({
          "attempt_id": "attempt:001",
          "workflow_id": workflow_id,
          "state": "timed_out",
          "failure_reason": "超时。"
        })];
        let review_results = vec![
            ReviewResult {
                review_id: "review:001".to_string(),
                workflow_id: workflow_id.clone(),
                workflow_node_id: None,
                reviewer_role: Some("director".to_string()),
                report_id: None,
                accepted_fact_ids: vec![],
                observation_ids: vec![],
                result: "returned".to_string(),
                summary: "退回一次".to_string(),
                evidence_refs: vec![],
                requires_director_confirmation: true,
                can_complete_node: false,
                warnings: vec![],
            },
            ReviewResult {
                review_id: "review:002".to_string(),
                workflow_id: workflow_id.clone(),
                workflow_node_id: None,
                reviewer_role: Some("director".to_string()),
                report_id: None,
                accepted_fact_ids: vec![],
                observation_ids: vec![],
                result: "returned".to_string(),
                summary: "退回两次".to_string(),
                evidence_refs: vec![],
                requires_director_confirmation: true,
                can_complete_node: false,
                warnings: vec![],
            },
        ];
        let exceptions = derive_workflow_exceptions(
            &workflow_id,
            &artifacts,
            &permission_requests,
            &attempts,
            &review_results,
        );
        let types = exceptions
            .iter()
            .map(|exception| exception.exception_type.as_str())
            .collect::<Vec<_>>();

        assert!(types.contains(&"subagent_timeout"));
        assert!(types.contains(&"long_permission_wait"));
        assert!(types.contains(&"repeated_review_return"));
        assert!(types.contains(&"unresolved_direction_risk"));
        assert!(types.contains(&"harness_blocked"));
    }

    #[test]
    fn workflow_state_transition_enforces_confirmed_table() {
        assert!(workflow_transition_allowed("draft", "ready", false));
        assert!(workflow_transition_allowed(
            "running",
            "waiting_decision",
            false
        ));
        assert!(!workflow_transition_allowed("draft", "running", false));
        assert!(!workflow_transition_allowed(
            "waiting_decision",
            "completed",
            false
        ));
        assert!(!workflow_transition_allowed("failed", "running", false));
        assert!(workflow_transition_allowed("failed", "running", true));
    }

    #[test]
    fn workflow_node_state_transition_enforces_actor_boundaries() {
        assert!(workflow_node_transition_allowed(
            "waiting",
            "running",
            "project_director",
            false
        ));
        assert!(workflow_node_transition_allowed(
            "reviewing",
            "passed",
            "review",
            false
        ));
        assert!(!workflow_node_transition_allowed(
            "reviewing",
            "passed",
            "subagent",
            false
        ));
        assert!(!workflow_node_transition_allowed(
            "waiting_decision",
            "running",
            "subagent",
            false
        ));
        assert!(workflow_node_transition_allowed(
            "waiting_decision",
            "running",
            "project_director",
            false
        ));
        assert!(!workflow_node_transition_allowed(
            "failed",
            "running",
            "project_director",
            false
        ));
        assert!(workflow_node_transition_allowed(
            "failed",
            "running",
            "project_director",
            true
        ));
    }

    #[test]
    fn director_completion_gate_requires_evidence_review_and_no_risk() {
        let package = TaskPackage {
            task_package_id: "task-package:001".to_string(),
            workflow_id: "workflow:001".to_string(),
            workflow_node_id: "node:001".to_string(),
            project_id: "project:001".to_string(),
            target_session_id: Some("thread-001".to_string()),
            target_role: Some("codex-dev".to_string()),
            task_goal: Some("完成目标".to_string()),
            allowed_read_scope: vec!["/tmp/project".to_string()],
            allowed_write_scope: vec!["/tmp/project/README.md".to_string()],
            available_skills: vec![],
            available_knowledge_refs: vec![],
            available_memory_refs: vec!["memory:confirmed:001".to_string()],
            callable_tool_capabilities: vec![],
            model_id: Some("codex-test-model".to_string()),
            harness_requirements: vec![],
            forbidden_actions: vec!["不写 .codex".to_string()],
            acceptance_criteria: vec!["验收通过".to_string()],
            report_format: vec!["做了什么".to_string()],
            timeout_policy: Some("600s".to_string()),
            failure_policy: Some("return_to_director".to_string()),
            version: 1,
            stale: false,
            stale_reasons: vec![],
            missing_fields: vec![],
            export_includes_internal_audit: false,
            memory_injection_summary: task_memory_injection::missing_summary(),
            warnings: vec![],
        };
        let reviews = vec![ReviewResult {
            review_id: "review:001".to_string(),
            workflow_id: "workflow:001".to_string(),
            workflow_node_id: Some("node:review".to_string()),
            reviewer_role: Some("director".to_string()),
            report_id: None,
            accepted_fact_ids: vec![],
            observation_ids: vec![],
            result: "passed".to_string(),
            summary: "审查通过".to_string(),
            evidence_refs: vec!["evidence:001".to_string()],
            requires_director_confirmation: true,
            can_complete_node: false,
            warnings: vec![],
        }];
        let gate = director_completion_gate(Some(&package), &reviews, &[]);
        assert!(gate.can_complete);

        let blocked = director_completion_gate(
            Some(&package),
            &reviews,
            &[WorkflowException {
                exception_id: "exception:direction".to_string(),
                workflow_id: "workflow:001".to_string(),
                workflow_node_id: None,
                exception_type: "unresolved_direction_risk".to_string(),
                summary: "方向风险".to_string(),
                status: "waiting_decision".to_string(),
                warnings: vec![],
            }],
        );
        assert!(!blocked.can_complete);
        assert!(blocked.missing.contains(&"no_unresolved_risk".to_string()));
    }

    #[test]
    fn workflow_interfaces_keep_conservative_boundaries() {
        let boundaries = workflow_interface_boundaries();
        assert!(boundaries
            .memory_candidate_interface
            .blocked
            .contains(&"auto_write_formal_memory".to_string()));
        assert!(boundaries
            .knowledge_refs_interface
            .blocked
            .contains(&"auto_scan_knowledge_base".to_string()));
        assert!(boundaries
            .model_pool_selector
            .blocked
            .contains(&"silent_auto_model_selection".to_string()));
        assert!(boundaries
            .harness_requirement_provider
            .blocked
            .contains(&"ordinary_workflow_node".to_string()));
        assert!(boundaries
            .tool_capability_registry
            .blocked
            .contains(&"tool_output_fulltext_in_ledger".to_string()));
    }

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

    #[test]
    fn workflow_dispatch_director_review_rejects_invalid_state_and_dispatch() {
        let dir = std::env::temp_dir().join(format!(
            "director-review-rejects-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project("/tmp/indexed-project");
        let draft = fixture_task_draft_request(&project.project_root, "总指导拒绝夹具");

        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_task_draft_at(&path, &draft).expect("work item should exist");
        let value = read_json_file(&path);
        let work_item_id = optional_string_from(&value["work_items"][0], "work_item_id")
            .expect("work item id should exist");

        let missing_dispatch = record_workflow_dispatch_director_review_at(
            &path,
            &fixture_director_review_request(
                &project.project_root,
                &work_item_id,
                "dispatch:missing",
                "accepted",
            ),
        );
        assert!(missing_dispatch.is_err());
        assert!(missing_dispatch
            .unwrap_err()
            .contains("工作项当前状态不是待回收"));

        update_work_item_state_at(
            &path,
            &fixture_work_item_state_update_request(
                &project.project_root,
                &work_item_id,
                "ready_to_dispatch",
            ),
        )
        .expect("work item should be ready");
        let prepared_dispatch_id = append_fixture_dispatch(
            &path,
            &project.project_root,
            &work_item_id,
            "prepared",
            "thread-001",
        );
        update_work_item_state_at(
            &path,
            &fixture_work_item_state_update_request(
                &project.project_root,
                &work_item_id,
                "running",
            ),
        )
        .expect("work item should be running");
        update_work_item_state_at(
            &path,
            &fixture_work_item_state_update_request(
                &project.project_root,
                &work_item_id,
                "ready_for_review",
            ),
        )
        .expect("work item should be ready for review");

        let not_completed = record_workflow_dispatch_director_review_at(
            &path,
            &fixture_director_review_request(
                &project.project_root,
                &work_item_id,
                &prepared_dispatch_id,
                "accepted",
            ),
        );
        assert!(not_completed.is_err());
        assert!(not_completed
            .unwrap_err()
            .contains("派发记录不是 completed"));

        let invalid_decision_dispatch_id = append_fixture_dispatch(
            &path,
            &project.project_root,
            &work_item_id,
            "completed",
            "thread-001",
        );
        let invalid_decision = record_workflow_dispatch_director_review_at(
            &path,
            &fixture_director_review_request(
                &project.project_root,
                &work_item_id,
                &invalid_decision_dispatch_id,
                "approve-ish",
            ),
        );
        assert!(invalid_decision.is_err());
        assert!(invalid_decision.unwrap_err().contains("未知总指导回收结论"));
        let updated = read_json_file(&path);
        assert_eq!(updated["reviews"].as_array().unwrap().len(), 0);

        let _ = fs::remove_dir_all(dir);
    }

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
