use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{fs, io::Write};
mod blackboard_candidate_store;
pub mod codex_db;
mod codex_local_runner;
mod codex_transcript;
mod control_core;
mod formal_memory_lifecycle;
mod formal_memory_store;
mod h4_execution_boundary;
mod h5_project_dispatch_bridge;
mod manual_relay;
mod mature_pattern_governance;
mod mature_pattern_store;
pub mod mcp;
mod memory_candidate_store;
mod memory_capture_bus;
mod memory_consistency;
mod memory_daily_loop;
mod memory_entity_relation_governance;
mod memory_entity_relation_store;
mod memory_lint_engine;
mod memory_lint_store;
mod observation_store;
mod operation_control;
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
mod utils;
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

    if let Some(codex_home) = db_path.parent() {
        if let Some(transcript) =
            load_codex_session_transcript_from_rollout_fallback(codex_home, thread_id)?
        {
            return Ok(transcript);
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

// P1 工作流自动连环 controller（链驱动逐节点真跑，圈固定测试项目；决策 2026-06-23）。
include!("workflow_chain_controller.rs");

// S3 agent 智能层·咨询第一刀（契约 trait + ProjectContext 装配 + v0 静态档案 + CliConsultantAgent 只读 + 喂 C1）。
include!("consultant_agent.rs");

// S3 agent 智能层·项目主管第一刀（复用咨询 harness：读已授权方案 → LM 只读拆解 → planned_tasks → 喂 prepare）。
include!("director_agent.rs");

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
                artifact,
                "task_goal",
            ))],
        ),
        allowed_read: string_array_or_placeholder(
            artifact,
            "allowed_read_scope",
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
            "report_format",
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
    if present_or_placeholder(optional_string_from(artifact, "task_goal")) == "待补充" {
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

// 路A（拦路石①）· 会话查找：先查静态快照 codex-index.json，找不到就回退实时 sqlite。
// 沿用 transcript reader 2026-06-02 的同款回退（记忆 codex-workbench-session-data-sources）：
// 静态索引是快照、会过期；bind/派发若只认它，5/31 后新建/新 mint 的会话全被判「不在索引」而拒。
// 实时 sqlite 是 codex 自己的库（会话列表也走它），新会话会立刻出现在这。读不到库 → 当作没有。
// 给 bind / 派发上下文 / 实验真跑用；普通会话列表/快照消费方不动（最小爆炸半径）。
fn find_index_thread_or_sqlite(index: &Value, thread_id: &str) -> Option<SessionRecord> {
    if let Some(session) = find_index_thread(index, thread_id) {
        return Some(session);
    }
    let db_path = codex_db::default_state_db_path();
    let rows = codex_db::read_threads(&db_path).ok()?;
    if let Some(row) = rows.into_iter().find(|row| row.thread_id == thread_id) {
        return Some(session_record_from_codex_thread(row));
    }
    // 方案a 真跑逮到（2026-07-05）：`codex exec` 产的会话 has_user_event=0，被 read_threads 的
    // **列表显示过滤**（has_user_event=1/非 subagent）滤掉 → 按 id 也永远找不到（交办新会话
    // 出生后绑定被拒的根因）。这里最后按主键精确查一次（存在性语义，不是列表语义）。
    // 找到 ≠ 能执行：执行闸（S1/path-lock/沙箱）全在下游、一字未动。
    codex_db::find_thread_by_id(&db_path, thread_id)
        .ok()
        .flatten()
        .map(session_record_from_codex_thread)
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
            .unwrap_or(0)
        + k3_b1_recovery::WARNING_COUNT;
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
    let operation_control =
        operation_control::read(&runtime_generated_at, &state.workflow_state_path);
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
        k3_b1_recovery: k3_b1_recovery::derive_k3_b1_recovery_read_model(),
        operation_control,
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
    use std::cell::{Cell, RefCell};

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
            chain_binds_per_task: false,
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
            suggest_workflow: false,
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

    // 引擎解封·走通整条派发路径(高危#1)。默认 #[ignore];显式
    // `cargo test --lib real_run_full_dispatch_resume -- --ignored --nocapture` 才起真 codex。
    // 端到端:bootstrap 工作流 → 备节点 → 绑真实 codex 会话 → execute_workflow_node_dispatch_for_index_at
    // (= 双闸命令过闸后调的真实现) → RealWorkflowNodeCodexRunner resume 真会话 → 真 codex。
    #[test]
    #[ignore = "spawns real codex through the full resume-based dispatch path"]
    fn real_run_full_dispatch_resume() {
        let test_root = "/Users/yoyi/codex-workflow-mario-test";
        let real_session = "019ed9f7-c0c2-7213-b871-6d18959b7c24";
        let dir =
            std::env::temp_dir().join(format!("full-dispatch-real-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project(test_root);
        let index = fixture_dispatch_index(test_root, real_session);
        let draft = fixture_task_draft_request(test_root, "走通真派发节点");
        fs::create_dir_all(&dir).expect("fixture dir should exist");
        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_task_draft_at(&path, &draft).expect("work item should exist");
        let value = read_json_file(&path);
        let work_item_id = optional_string_from(&value["work_items"][0], "work_item_id")
            .expect("work item id should exist");
        update_work_item_state_at(
            &path,
            &fixture_work_item_state_update_request(test_root, &work_item_id, "ready_to_dispatch"),
        )
        .expect("work item should be ready");
        let workflow_id = default_workflow_id(test_root);
        let node_id = format!("{workflow_id}:node:codex-dev");
        let session = fixture_session(real_session, test_root, true);
        bind_workflow_node_codex_session_at(
            &path,
            &fixture_node_session_bind_request(
                test_root,
                &node_id,
                Some(&work_item_id),
                real_session,
            ),
            &session,
        )
        .expect("binding real session should write");
        let runner = codex_local_runner::RealWorkflowNodeCodexRunner;
        let request = WorkflowNodeDispatchExecuteRequest {
            project_root: test_root.to_string(),
            node_id: node_id.clone(),
            work_item_id: work_item_id.clone(),
            prompt_kind: "user_reviewed_instruction".to_string(),
            user_reviewed_instruction: Some(UserReviewedInstructionInput {
                instruction_id: "instruction:realrun:full-dispatch".to_string(),
                summary: "走通真派发：建证明文件".to_string(),
                objective: "在测试项目创建 full dispatch 证明文件".to_string(),
                execution_cwd: test_root.to_string(),
                sandbox_mode: "workspace-write".to_string(),
                allowed_write_roots: vec![test_root.to_string()],
                allowed_reads: vec![test_root.to_string()],
                allowed_writes: vec![format!("{test_root}/workflow-fulldispatch-proof.txt")],
                forbidden_actions: vec![
                    "不读取 auth.json、.env、密钥、token 或授权文件。".to_string(),
                    "不读取完整 transcript。".to_string(),
                    "不运行 harness。".to_string(),
                ],
                timeout_seconds: 180,
                max_retries: 0,
                required_return: vec!["本步做了什么".to_string(), "改了哪些文件".to_string()],
                prompt_preview: Some(
                    "在当前目录创建文件 workflow-fulldispatch-proof.txt，写入一行：full dispatch path ok。完成后用一句话说明你做了什么。"
                        .to_string(),
                ),
            }),
        };
        let readback_db_path = codex_db::default_state_db_path();
        let result = execute_workflow_node_dispatch_for_index_at(
            &path,
            &index,
            &readback_db_path,
            &runner,
            &request,
        )
        .expect("full dispatch path should complete");
        println!(
            "[FULL_DISPATCH] state={} exit={:?} summary={:?}",
            result.dispatch.state, result.dispatch.exit_code, result.dispatch.last_message_summary
        );
        assert_eq!(result.dispatch.exit_code, Some(0), "codex exit should be 0");
        assert_eq!(result.dispatch.state, "completed");
    }

    // S1-③·正向真跑（高危#1 轻档·固定测试项目）：真 codex 经 execute_project_workflow_node_at
    // （= S1 合并强闸所在层）真跑一个画布工作流节点，第一次用真 codex 验 S1 闸在真实执行里成立。
    // 与 real_run_full_dispatch_resume 的关键区别：那个直调 execute_workflow_node_dispatch_for_index_at
    // （在闸之后那层，绕闸）；本测试走 execute_project_workflow_node_at（过闸）——闸不放行就会在起
    // runner 前返回 real_execution_gate_blocked，所以「completed」本身即证明真 codex 过了
    // decide_real_execution_command（authorized·path-lock 命中）。默认 #[ignore]；显式
    // `cargo test --lib s1_step3_real_run_through_gate -- --ignored --nocapture` 才起真 codex。
    #[test]
    #[ignore = "S1-③: spawns real codex THROUGH the S1 gate (execute_project_workflow_node_at) in the test project"]
    fn s1_step3_real_run_through_gate() {
        let test_root = "/Users/yoyi/codex-workflow-mario-test";
        let real_session = "019ed9f7-c0c2-7213-b871-6d18959b7c24";
        let proof_token = unix_timestamp_string();
        // 跑前清掉旧 proof，确保断言看到的是本次产物。
        let proof_path = format!("{test_root}/s1-step3-proof.txt");
        let _ = fs::remove_file(&proof_path);
        let dir = std::env::temp_dir().join(format!("s1-step3-gate-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project(test_root);
        let index = fixture_dispatch_index(test_root, real_session);
        fs::create_dir_all(&dir).expect("fixture dir should exist");
        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        // 任务包 task_goal → artifact.task_goal → fields.goals → objective → prompt_preview → codex 真 prompt。
        let objective = format!(
            "在当前项目根目录创建文件 s1-step3-proof.txt，只写入一行内容：S1-step3 gate real-run ok {proof_token}。完成后用一句话说明你创建了该文件，不要修改任何其它文件。"
        );
        create_task_draft_at(
            &path,
            &TaskDraftRequest {
                project_root: test_root.to_string(),
                title: format!("S1-③ 过闸真跑 {proof_token}"),
                objective: objective.clone(),
                assigned_role: Some("codex-dev".to_string()),
            },
        )
        .expect("work item should exist");
        let work_item_id =
            optional_string_from(&read_json_file(&path)["work_items"][0], "work_item_id")
                .expect("work item id should exist");
        update_work_item_state_at(
            &path,
            &fixture_work_item_state_update_request(test_root, &work_item_id, "ready_to_dispatch"),
        )
        .expect("work item should be ready");
        let workflow_id = default_workflow_id(test_root);
        let node_id = format!("{workflow_id}:node:codex-dev");
        let session = fixture_session(real_session, test_root, true);
        bind_workflow_node_codex_session_at(
            &path,
            &fixture_node_session_bind_request(
                test_root,
                &node_id,
                Some(&work_item_id),
                real_session,
            ),
            &session,
        )
        .expect("binding real session should write");
        let runner = codex_local_runner::RealWorkflowNodeCodexRunner;
        let request = ProjectWorkflowNodeRunRequest {
            project_root: test_root.to_string(),
            node_id,
            work_item_id,
            workflow_id: None,
        };
        let readback_db_path = codex_db::default_state_db_path();
        let result =
            execute_project_workflow_node_at(&path, &index, &readback_db_path, &runner, &request)
                .expect("S1 闸应 authorized 放行并走通真派发到 completed");
        println!(
            "[S1_STEP3] state={} exit={:?} summary={:?}",
            result.dispatch.state, result.dispatch.exit_code, result.dispatch.last_message_summary
        );
        assert_eq!(result.dispatch.state, "completed", "派发应 completed");
        assert_eq!(result.dispatch.exit_code, Some(0), "codex exit 应为 0");
        let proof = fs::read_to_string(&proof_path)
            .unwrap_or_else(|e| panic!("proof 文件应在测试项目内生成 {proof_path}：{e}"));
        assert!(
            proof.contains(&proof_token),
            "proof 文件应含本次 token {proof_token}，实际：{proof}"
        );
        println!("[S1_STEP3] proof_path={proof_path} content={proof:?}");
    }

    // S1-③·非测试拦截（关键·只验"被拦"、绝不真跑进非测试）：把 project_root 换成非测试路径走
    // execute_project_workflow_node_at，期望 S1 闸在起 runner 前返回 real_execution_gate_blocked。
    // runner 用 panic-stub——一旦被调即 panic（= 即便闸有 bug 也绝不真起 codex 进非测试，守 §3 铁律）。
    // 常驻回归（去 #[ignore]）：铁律在产品路径的护栏——安全（panic-stub 保证不起 codex）、快。
    #[test]
    fn s1_step3_nontest_root_blocked_before_runner() {
        struct MustNotRunRunner;
        impl CodexResumeRunner for MustNotRunRunner {
            fn resume_with_options(
                &self,
                _thread_id: &str,
                _prompt: &str,
                _last_message_path: &Path,
                _options: &CodexResumeRequestOptions,
            ) -> Result<(CodexResumeRunResult, WorkflowNodeDispatchExecutionOptions), String>
            {
                panic!(
                    "S1-③ 铁律违规：非测试 project_root 到达了 runner（本应被 S1 闸拦截，绝不真起 codex 进非测试）"
                );
            }
        }
        let nontest_root = std::env::temp_dir()
            .join(format!("s1-step3-nontest-{}", unix_timestamp_string()))
            .display()
            .to_string();
        let fake_session = "thread-s1-step3-nontest";
        let dir = std::env::temp_dir().join(format!(
            "s1-step3-nontest-state-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project(&nontest_root);
        let index = fixture_dispatch_index(&nontest_root, fake_session);
        fs::create_dir_all(&dir).expect("fixture dir should exist");
        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_task_draft_at(
            &path,
            &fixture_task_draft_request(&nontest_root, "S1-③ 非测试拦截用例"),
        )
        .expect("work item should exist");
        let work_item_id =
            optional_string_from(&read_json_file(&path)["work_items"][0], "work_item_id")
                .expect("work item id should exist");
        update_work_item_state_at(
            &path,
            &fixture_work_item_state_update_request(
                &nontest_root,
                &work_item_id,
                "ready_to_dispatch",
            ),
        )
        .expect("work item should be ready");
        let workflow_id = default_workflow_id(&nontest_root);
        let node_id = format!("{workflow_id}:node:codex-dev");
        let session = fixture_session(fake_session, &nontest_root, true);
        bind_workflow_node_codex_session_at(
            &path,
            &fixture_node_session_bind_request(
                &nontest_root,
                &node_id,
                Some(&work_item_id),
                fake_session,
            ),
            &session,
        )
        .expect("binding should write");
        let runner = MustNotRunRunner;
        let request = ProjectWorkflowNodeRunRequest {
            project_root: nontest_root.clone(),
            node_id,
            work_item_id,
            workflow_id: None,
        };
        let readback_db_path = codex_db::default_state_db_path();
        let error =
            execute_project_workflow_node_at(&path, &index, &readback_db_path, &runner, &request)
                .expect_err("非测试 project_root 应被 S1 闸拦截、不起 runner");
        assert!(
            error.contains("real_execution_gate_blocked"),
            "应是 S1 闸拦截错误（real_execution_gate_blocked），实际：{error}"
        );
        // MustNotRunRunner 未 panic（没被调）+ 非测试路径未被创建 = 没起 codex 进非测试。
        assert!(
            !std::path::Path::new(&nontest_root).exists(),
            "非测试路径不应有任何写入（codex 不该被起）"
        );
        println!("[S1_STEP3_NONTEST] blocked_error={error}");
    }

    // 宽松 stub：不像 StubCodexResumeRunner 那样硬编码 execution_cwd=/Users/yoyi（那是某旧
    // fixture 绑死的）；本 runner 只验证「走到了真跑这步」，回 exit 0 + readback。实验命令的
    // execution_cwd 应是固定测试项目（codex_resume_options_for_context 取 instruction 的值）。
    struct PermissiveExperimentRunner {
        stats: CodexDispatchReadbackStats,
    }
    impl CodexResumeRunner for PermissiveExperimentRunner {
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
            fs::write(last_message_path, "EXPERIMENT_STUB_OK")
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

    #[derive(Default)]
    struct RecordingOptionsRunner {
        options: RefCell<Option<CodexResumeRequestOptions>>,
    }
    impl CodexResumeRunner for RecordingOptionsRunner {
        fn resume_with_options(
            &self,
            _thread_id: &str,
            _prompt: &str,
            last_message_path: &Path,
            options: &CodexResumeRequestOptions,
        ) -> Result<(CodexResumeRunResult, WorkflowNodeDispatchExecutionOptions), String> {
            self.options.replace(Some(options.clone()));
            if let Some(parent) = last_message_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|error| format!("fixture output dir create failed: {error}"))?;
            }
            fs::write(last_message_path, "READONLY_DISPATCH_STUB_OK")
                .map_err(|error| format!("fixture last message write failed: {error}"))?;
            Ok((
                CodexResumeRunResult {
                    exit_code: 0,
                    timed_out: false,
                    stderr_summary: None,
                },
                WorkflowNodeDispatchExecutionOptions {
                    readback_stats: Some(CodexDispatchReadbackStats {
                        transcript_event_count: 1,
                        transcript_target_hits: 1,
                    }),
                },
            ))
        }
    }

    // P3 实验面真跑（A 映射）·机器闸：用 stub runner（不起真 codex）验证
    // execute_experiment_node_dispatch_at 在固定测试项目里自动建临时 work_item + 绑会话 +
    // 走通派发到 completed。真 codex 真跑由用户真机做（#[ignore] 见 real_run_full_dispatch_resume 同款）。
    #[test]
    fn experiment_node_dispatch_creates_temp_work_item_and_dispatches() {
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let dir =
            std::env::temp_dir().join(format!("experiment-dispatch-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        let index_path = dir.join("codex-index.json");
        let project = fixture_project(test_root);
        let index = fixture_dispatch_index(test_root, "thread-exp-1");
        fs::create_dir_all(&dir).expect("fixture dir should exist");
        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        let before = read_json_file(&path)["work_items"]
            .as_array()
            .map(|items| items.len())
            .unwrap_or(0);
        let runner = PermissiveExperimentRunner {
            stats: CodexDispatchReadbackStats {
                transcript_event_count: 5,
                transcript_target_hits: 1,
            },
        };
        DISPATCH_READBACK_NATIVE_READ_COUNT.with(|count| count.set(0));
        let request = ExperimentNodeDispatchExecuteRequest {
            session_mode: "resume".to_string(),
            thread_id: Some("thread-exp-1".to_string()),
            summary: "实验节点".to_string(),
            objective: "建实验证明文件".to_string(),
            sandbox_mode: "workspace-write".to_string(),
            timeout_seconds: Some(120),
        };
        let result =
            execute_experiment_node_dispatch_at(&path, &index, &index_path, &runner, &request)
                .expect("experiment dispatch should complete");
        assert_eq!(result.dispatch.state, "completed");
        assert_eq!(result.dispatch.exit_code, Some(0));
        // A · 自动建了一个临时 work_item（数量增长 + 标题前缀），无需手填 work_item_id。
        let after_value = read_json_file(&path);
        let items = after_value["work_items"]
            .as_array()
            .expect("work_items array");
        assert!(items.len() > before, "应自动建一个临时 work_item");
        let temp = items
            .iter()
            .find(|item| {
                optional_string_from(item, "title")
                    .map(|title| title.starts_with("experiment-temp-"))
                    .unwrap_or(false)
            })
            .expect("临时 work_item 标题应为 experiment-temp-*");
        // 目标锁死固定测试项目：临时票挂在测试项目的 default workflow 上。
        assert_eq!(
            optional_string_from(temp, "workflow_id").as_deref(),
            Some(default_workflow_id(test_root).as_str())
        );
    }

    // C · new 不启用（resume-only，用户拍板 C）：报清楚的错、不假跑。
    #[test]
    fn experiment_node_dispatch_new_session_returns_clear_error() {
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let dir = std::env::temp_dir().join(format!(
            "experiment-dispatch-new-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let index_path = dir.join("codex-index.json");
        let index = fixture_dispatch_index(test_root, "thread-exp-1");
        let runner = StubCodexResumeRunner {
            stats: CodexDispatchReadbackStats {
                transcript_event_count: 0,
                transcript_target_hits: 0,
            },
        };
        let request = ExperimentNodeDispatchExecuteRequest {
            session_mode: "new".to_string(),
            thread_id: None,
            summary: "实验节点".to_string(),
            objective: "建实验证明文件".to_string(),
            sandbox_mode: "workspace-write".to_string(),
            timeout_seconds: Some(120),
        };
        let error =
            execute_experiment_node_dispatch_at(&path, &index, &index_path, &runner, &request)
                .expect_err("resume-only：new 应报错不假跑");
        assert!(
            error.contains("resume-only"),
            "错误应点明 resume-only / 开新会话未启用：{error}"
        );
    }

    // 会话不在 5/31 静态名册 → 拒绝（resume 近期/新会话的现实障碍，拦路石①）。
    #[test]
    fn experiment_node_dispatch_session_not_in_index_refused() {
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let dir = std::env::temp_dir().join(format!(
            "experiment-dispatch-noidx-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let index_path = dir.join("codex-index.json");
        let index = fixture_dispatch_index(test_root, "thread-in-index");
        let runner = StubCodexResumeRunner {
            stats: CodexDispatchReadbackStats {
                transcript_event_count: 0,
                transcript_target_hits: 0,
            },
        };
        let request = ExperimentNodeDispatchExecuteRequest {
            session_mode: "resume".to_string(),
            thread_id: Some("thread-NOT-in-index".to_string()),
            summary: "实验节点".to_string(),
            objective: "建实验证明文件".to_string(),
            sandbox_mode: "workspace-write".to_string(),
            timeout_seconds: Some(120),
        };
        let error =
            execute_experiment_node_dispatch_at(&path, &index, &index_path, &runner, &request)
                .expect_err("会话不在名册应拒绝");
        assert!(
            error.contains("不在当前索引内"),
            "错误应点明会话不在名册：{error}"
        );
    }

    // S2-3·全链 stub 集成：把 C 阶段角色循环 8 步串成一条真任务端到端（stub worker）——证明编排在
    // 真实命令上跑得通、worker **经 S1 闸**（execute_project_workflow_node_at）被派发。stub runner 不起
    // 真 codex（自动测试守死线）；真跑 codex 是 §6 单独步（#[ignore]，见 s2_3_real_run_role_loop_through_gate）。
    // 死线：worker 步用固定测试项目 root → path-lock 命中 → S1 闸授权放行；命令本体/闸/沙箱 0-diff。
    #[test]
    fn s2_3_role_loop_full_chain_through_gate_with_stub() {
        let timestamp_ms = 1_765_300_000_000;
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT; // path-lock 命中 → ⑤ 经 S1 闸授权放行
        let thread_id = "thread-s2-3-stub";
        let dir = test_temp_dir("s2-3-full-chain-stub");
        let path = dir.join("workflow-state.v0.json");
        let index_path = dir.join("codex-index.json");
        let project = fixture_project(test_root);
        let index = fixture_dispatch_index(test_root, thread_id);
        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");

        // ①②③ 方案 → 授权 → 边界复核（真命令链，推到 active 授权）
        let (proposal, authorization, revision) =
            create_active_project_director_authorization_fixture(
                &path,
                test_root,
                thread_id,
                timestamp_ms,
            );

        // ④ 主管拆 + 准备：先给 worker 节点(codex-dev)绑会话，再 prepare（产 prepared dispatch、不执行）
        let workflow_id = default_workflow_id(test_root);
        let node_id = format!("{workflow_id}:node:codex-dev");
        bind_workflow_node_codex_session_for_index_at(
            &path,
            &index,
            &fixture_node_session_bind_request(test_root, &node_id, None, thread_id),
        )
        .expect("node binding should write");
        let prepared = prepare_authorized_auto_dispatch_for_index_at(
            &path,
            &index,
            &fixture_project_director_prepare_input(
                test_root,
                &proposal.proposal_id,
                &authorization.authorization_id,
                revision,
                vec![],
            ),
        )
        .expect("授权范围内应准备出 worker 派发");
        assert_eq!(
            prepared.prepared_dispatches.len(),
            1,
            "主管拆任务应准备 1 个 worker 派发"
        );
        let prep = &prepared.prepared_dispatches[0];
        let prep_work_item = prep
            .work_item_id
            .clone()
            .expect("prepared dispatch 应有 work_item_id（带任务包目标）");
        let prep_node = prep
            .workflow_node_id
            .clone()
            .expect("prepared dispatch 应有 workflow_node_id");

        // ④→⑤ glue：从 prepared dispatch 提 node/work_item 组请求喂⑤（样板 chain_controller:427）。worker
        // 真派发经 S1 闸（stub runner）；用 prepared 的 work_item（带任务包 task_goal）→ execute 从任务包构指令。
        // ⑤ 不放行会在起 runner 前 Err（real_execution_gate_blocked），故 completed 即证明过了 S1 闸（path-lock 命中）。
        let runner = PermissiveExperimentRunner {
            stats: CodexDispatchReadbackStats {
                transcript_event_count: 3,
                transcript_target_hits: 1,
            },
        };
        let run_request = ProjectWorkflowNodeRunRequest {
            project_root: test_root.to_string(),
            node_id: prep_node.clone(),
            work_item_id: prep_work_item.clone(),
            workflow_id: Some(workflow_id.clone()),
        };
        let run =
            execute_project_workflow_node_at(&path, &index, &index_path, &runner, &run_request)
                .expect("worker 经 S1 闸应授权放行并走通派发到 completed");
        assert_eq!(run.dispatch.state, "completed", "worker 派发应 completed");
        let worker_node_id = run.dispatch.node_id.clone();
        let worker_work_item_id = run.dispatch.work_item_id.clone();
        let worker_dispatch_id = run.dispatch.dispatch_id.clone();

        // ⑥ worker 汇报（引用⑤的真 dispatch/work_item/node）
        let report = record_worker_structured_report_at(
            &path,
            &fixture_c5_worker_report_input(
                test_root,
                &worker_work_item_id,
                &worker_dispatch_id,
                &worker_node_id,
            ),
        )
        .expect("worker 结构化汇报应写 audit");

        // ⑦ 主管确认过程事实
        record_project_director_process_fact_decision_at(
            &path,
            &fixture_c5_process_fact_decision_input(
                test_root,
                &report.audit_event_id,
                &worker_dispatch_id,
                "confirm_process_fact",
            ),
        )
        .expect("主管应确认过程事实");

        // ⑧ 全局复核 + 用户决定（看真结果）
        record_global_final_result_review_at(
            &path,
            &GlobalFinalResultReviewInput {
                project_root: test_root.to_string(),
                project_id: project_id(test_root),
                workflow_id: workflow_id.clone(),
                authorization_id: authorization.authorization_id.clone(),
                proposal_id: proposal.proposal_id.clone(),
                actor_id: "global_director".to_string(),
                actor_role: "global_director".to_string(),
                decision: "accepted".to_string(),
                summary: "全局复核：角色循环端到端跑通、过程事实已确认。".to_string(),
                evidence_refs: vec!["evidence:s2-3-final-review:001".to_string()],
                accepted_process_fact_ids: vec![format!(
                    "process-fact:{}",
                    stable_id(&report.audit_event_id)
                )],
                open_issues: vec![],
                deferred_items: vec![],
                expected_workflow_revision: None,
            },
        )
        .expect("全局最终复核应通过");
        record_user_result_decision_at(
            &path,
            &UserResultDecisionInput {
                project_root: test_root.to_string(),
                project_id: project_id(test_root),
                workflow_id: workflow_id.clone(),
                actor_id: "user-fixture".to_string(),
                actor_role: "user".to_string(),
                decision: "accept_result".to_string(),
                summary: "用户验收：接受角色循环结果。".to_string(),
                requested_changes: vec![],
                accepted_review_id: None,
                expected_workflow_revision: None,
            },
        )
        .expect("用户结果决定应记录");

        let _ = fs::remove_dir_all(dir);
    }

    // S2-3·path-lock 负向（铁律）：角色循环即便授权了方案、准备了派发，worker 步(⑤)在**非测试 root**
    // 仍被 S1 闸拦——role-loop 授权 ≠ path-lock 旁路（④ prepare 不自带 path-lock，全靠⑤的 S1 闸兜底）。
    // panic-stub：被调即 panic（即便闸有 bug 也绝不真起 codex 进非测试，守 §3 死线）。
    #[test]
    fn s2_3_role_loop_worker_blocked_at_nontest_root() {
        struct MustNotRunRunner;
        impl CodexResumeRunner for MustNotRunRunner {
            fn resume_with_options(
                &self,
                _thread_id: &str,
                _prompt: &str,
                _last_message_path: &Path,
                _options: &CodexResumeRequestOptions,
            ) -> Result<(CodexResumeRunResult, WorkflowNodeDispatchExecutionOptions), String>
            {
                panic!("S2-3 铁律违规：worker 在非测试 project_root 到达了 runner（应被 S1 闸拦）");
            }
        }
        let timestamp_ms = 1_765_300_000_000;
        let nontest_root = "/tmp/s2-3-nontest-roleloop";
        let thread_id = "thread-s2-3-nontest";
        let dir = test_temp_dir("s2-3-nontest-roleloop");
        let path = dir.join("workflow-state.v0.json");
        let index_path = dir.join("codex-index.json");
        let project = fixture_project(nontest_root);
        let index = fixture_dispatch_index(nontest_root, thread_id);
        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        // ①②③④：非测试 root 也能授权+准备（path-lock 不在这几步）
        let (proposal, authorization, revision) =
            create_active_project_director_authorization_fixture(
                &path,
                nontest_root,
                thread_id,
                timestamp_ms,
            );
        let workflow_id = default_workflow_id(nontest_root);
        let node_id = format!("{workflow_id}:node:codex-dev");
        bind_workflow_node_codex_session_for_index_at(
            &path,
            &index,
            &fixture_node_session_bind_request(nontest_root, &node_id, None, thread_id),
        )
        .expect("node binding should write");
        let prepared = prepare_authorized_auto_dispatch_for_index_at(
            &path,
            &index,
            &fixture_project_director_prepare_input(
                nontest_root,
                &proposal.proposal_id,
                &authorization.authorization_id,
                revision,
                vec![],
            ),
        )
        .expect("非测试 root 的 prepare 仍可准备（path-lock 在 worker 步⑤兜底，不在④）");
        let prep = &prepared.prepared_dispatches[0];
        // ⑤ worker 步：非测试 root → S1 闸拦、panic-stub 绝不被调
        let runner = MustNotRunRunner;
        let run_request = ProjectWorkflowNodeRunRequest {
            project_root: nontest_root.to_string(),
            node_id: prep.workflow_node_id.clone().expect("prepared node id"),
            work_item_id: prep.work_item_id.clone().expect("prepared work_item_id"),
            workflow_id: Some(workflow_id.clone()),
        };
        let err =
            execute_project_workflow_node_at(&path, &index, &index_path, &runner, &run_request)
                .expect_err("worker 步在非测试 root 应被 S1 闸拦、不起 runner");
        assert!(
            err.contains("real_execution_gate_blocked"),
            "应是 S1 闸拦截（real_execution_gate_blocked），实际：{err}"
        );

        let _ = fs::remove_dir_all(dir);
    }

    // S2-3·§6 真跑（高危#1·固定测试项目轻档·默认 #[ignore]）：worker 真 codex 经 C 阶段角色循环 + S1 闸
    // 建真文件。自定义 proposal 的 goal_summary/proposed_steps（→ planned_task.task_goal → 任务包 goals →
    // codex prompt）让 worker 写 s2-3-loop-proof.txt（含本次 token）。worker 步走 execute_project_workflow_node_at
    // （S1 闸）——非 authorized 会在起 runner 前 Err，故 completed + proof 即证明真 codex 过了闸（path-lock 命中）。
    // 显式 `cargo test --lib s2_3_real_run_role_loop_builds_proof_through_gate -- --ignored --nocapture` 才起真 codex。
    #[test]
    #[ignore = "S2-3 §6: spawns real codex through the C-stage role loop + S1 gate in the test project"]
    fn s2_3_real_run_role_loop_builds_proof_through_gate() {
        let timestamp_ms = 1_765_300_000_000;
        let test_root = "/Users/yoyi/codex-workflow-mario-test";
        let real_session = "019ed9f7-c0c2-7213-b871-6d18959b7c24";
        let thread_id = real_session;
        let proof_token = unix_timestamp_string();
        let proof_path = format!("{test_root}/s2-3-loop-proof.txt");
        let _ = fs::remove_file(&proof_path);
        let dir = test_temp_dir("s2-3-real-run-roleloop");
        let path = dir.join("workflow-state.v0.json");
        let project = fixture_project(test_root);
        let index = fixture_dispatch_index(test_root, thread_id);
        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");

        // ①②③ 自定义 proposal（goal_summary/proposed_steps 注入 proof 指令）→ 确认建授权 → 边界复核激活
        let mut proposal_input = fixture_project_consultation_proposal_input(test_root);
        proposal_input.scope_draft.allowed_agent_ids = vec![thread_id.to_string()];
        proposal_input.goal_summary = format!(
            "在当前项目根目录创建文件 s2-3-loop-proof.txt，只写入一行：s2-3 role-loop real-run ok {proof_token}"
        );
        proposal_input.proposed_steps = vec![
            format!(
                "在当前项目根目录创建文件 s2-3-loop-proof.txt，写入一行内容：s2-3 role-loop real-run ok {proof_token}"
            ),
            "完成后用一句话说明你创建了该文件，不要修改任何其它文件。".to_string(),
        ];
        let created = project_consultation_proposal_store::create_proposal(
            &path,
            &proposal_input,
            timestamp_ms,
            "write-s2-3-realrun-proposal",
        )
        .expect("proposal should create");
        let confirmed = project_consultation_proposal_store::record_decision(
            &path,
            &RecordProjectConsultationProposalDecisionInput {
                project_root: test_root.to_string(),
                proposal_id: created.proposal.proposal_id.clone(),
                actor_id: "user-fixture".to_string(),
                decision: ProjectConsultationProposalDecisionKind::Confirm,
                summary: "用户确认 S2-3 §6 真跑方案。".to_string(),
                expected_proposal_store_revision: Some(created.store_revision),
                expected_plan_authorization_store_revision: None,
            },
            timestamp_ms + 1,
            "write-s2-3-realrun-confirm",
            "write-s2-3-realrun-auth",
            "write-s2-3-realrun-auth-user",
        )
        .expect("confirm should create authorization");
        let authorization = confirmed
            .plan_authorization
            .expect("confirmed proposal should link authorization");
        let revision = confirmed
            .plan_authorization_store_revision
            .expect("confirmed proposal should return revision");
        let activated = plan_authorization_store::record_global_boundary_review_with_proposal(
            &path,
            &fixture_global_boundary_review_input(
                test_root,
                &confirmed.proposal.proposal_id,
                &authorization.authorization_id,
                revision,
            ),
            timestamp_ms + 2,
            "write-s2-3-realrun-boundary",
        )
        .expect("global boundary review should activate authorization");

        // ④ 主管拆 + 准备（先绑 worker 节点会话）
        let workflow_id = default_workflow_id(test_root);
        let node_id = format!("{workflow_id}:node:codex-dev");
        bind_workflow_node_codex_session_for_index_at(
            &path,
            &index,
            &fixture_node_session_bind_request(test_root, &node_id, None, thread_id),
        )
        .expect("node binding should write");
        let prepared = prepare_authorized_auto_dispatch_for_index_at(
            &path,
            &index,
            &fixture_project_director_prepare_input(
                test_root,
                &confirmed.proposal.proposal_id,
                &activated.authorization.authorization_id,
                activated.store_revision,
                vec![],
            ),
        )
        .expect("授权范围内应准备出 worker 派发");
        let prep = &prepared.prepared_dispatches[0];

        // ⑤ worker 真派发：真 codex 经 S1 闸（path-lock 命中测试项目 → authorized）→ 建 proof 文件
        let runner = codex_local_runner::RealWorkflowNodeCodexRunner;
        let readback_db_path = codex_db::default_state_db_path();
        let run_request = ProjectWorkflowNodeRunRequest {
            project_root: test_root.to_string(),
            node_id: prep.workflow_node_id.clone().expect("prepared node id"),
            work_item_id: prep.work_item_id.clone().expect("prepared work_item_id"),
            workflow_id: Some(workflow_id.clone()),
        };
        let run = execute_project_workflow_node_at(
            &path,
            &index,
            &readback_db_path,
            &runner,
            &run_request,
        )
        .expect("worker 经 S1 闸应授权放行并走通真派发到 completed");
        println!(
            "[S2_3_REALRUN] state={} exit={:?} summary={:?}",
            run.dispatch.state, run.dispatch.exit_code, run.dispatch.last_message_summary
        );
        assert_eq!(run.dispatch.state, "completed", "worker 真派发应 completed");
        assert_eq!(run.dispatch.exit_code, Some(0), "codex exit 应为 0");
        let proof = fs::read_to_string(&proof_path)
            .unwrap_or_else(|e| panic!("worker 应在测试项目内建 proof 文件 {proof_path}：{e}"));
        assert!(
            proof.contains(&proof_token),
            "proof 应含本次 token {proof_token}，实际：{proof}"
        );
        println!("[S2_3_REALRUN] proof_path={proof_path} content={proof:?}");

        // ⑥⑦⑧ 汇报 → 主管确认 → 全局复核 + 用户决定（看真结果）
        let report = record_worker_structured_report_at(
            &path,
            &fixture_c5_worker_report_input(
                test_root,
                &run.dispatch.work_item_id,
                &run.dispatch.dispatch_id,
                &run.dispatch.node_id,
            ),
        )
        .expect("worker 汇报应写 audit");
        record_project_director_process_fact_decision_at(
            &path,
            &fixture_c5_process_fact_decision_input(
                test_root,
                &report.audit_event_id,
                &run.dispatch.dispatch_id,
                "confirm_process_fact",
            ),
        )
        .expect("主管应确认过程事实");
        record_global_final_result_review_at(
            &path,
            &GlobalFinalResultReviewInput {
                project_root: test_root.to_string(),
                project_id: project_id(test_root),
                workflow_id: workflow_id.clone(),
                authorization_id: activated.authorization.authorization_id.clone(),
                proposal_id: confirmed.proposal.proposal_id.clone(),
                actor_id: "global_director".to_string(),
                actor_role: "global_director".to_string(),
                decision: "accepted".to_string(),
                summary: "全局复核：worker 真跑出真结果、过程事实已确认。".to_string(),
                evidence_refs: vec!["evidence:s2-3-realrun-final:001".to_string()],
                accepted_process_fact_ids: vec![format!(
                    "process-fact:{}",
                    stable_id(&report.audit_event_id)
                )],
                open_issues: vec![],
                deferred_items: vec![],
                expected_workflow_revision: None,
            },
        )
        .expect("全局最终复核应通过");
        record_user_result_decision_at(
            &path,
            &UserResultDecisionInput {
                project_root: test_root.to_string(),
                project_id: project_id(test_root),
                workflow_id: workflow_id.clone(),
                actor_id: "user-fixture".to_string(),
                actor_role: "user".to_string(),
                decision: "accept_result".to_string(),
                summary: "用户验收：接受 S2-3 真跑结果。".to_string(),
                requested_changes: vec![],
                accepted_review_id: None,
                expected_workflow_revision: None,
            },
        )
        .expect("用户结果决定应记录");

        let _ = fs::remove_dir_all(dir);
    }

    // S2-3·旁路封堵：旧桩/H5 真 runner 产品入口的 path-lock 守卫——非测试 root（含 j2_b_b1 写死的
    // Documents/mario test、空 root）一律拦，仅固定测试项目放行。守卫只在 #[tauri::command] 包装层用
    // （不被单测调），内层 _at/_with_runner 零影响。
    #[test]
    fn s2_3_bypass_wrappers_path_lock_blocks_nontest_allows_test() {
        assert!(require_test_project_path_lock("/tmp/not-test", "x").is_err());
        assert!(
            require_test_project_path_lock(project_workflow_automation::J2_B_B1_PROJECT_ROOT, "x")
                .is_err(),
            "j2_b_b1 写死的 Documents/mario test（非测试）应被拦"
        );
        assert!(
            require_test_project_path_lock(project_workflow_automation::J2_B_B2_PROJECT_ROOT, "x")
                .is_err(),
            "j2_b_b2 写死的 product-line/tmp 隔离项目（非测试·workspace-write）应被拦"
        );
        assert!(require_test_project_path_lock("", "x").is_err());
        assert!(
            require_test_project_path_lock(WORKFLOW_ENGINE_TEST_PROJECT_ROOT, "x").is_ok(),
            "固定测试项目应放行"
        );
    }

    // ===== S3 咨询第一刀·stub TDD（自动测试绝不真起 codex）=====

    // stub ConsultantAgent：不起 codex，按 ctx/question 回固定结构化方案——证编排，不证脑。
    struct StubConsultant;
    impl ConsultantAgent for StubConsultant {
        fn consult(
            &self,
            ctx: &ProjectContext,
            question: &str,
        ) -> Result<ConsultationProposal, String> {
            Ok(ConsultationProposal {
                user_goal: format!("就「{question}」给方向"),
                goal_summary: format!("基于 {} 的入口文档定方向", ctx.project_name),
                scope_note: "只读咨询、不执行".to_string(),
                reasoning: vec![format!("入口文档存在={}", ctx.entry_document.is_some())],
                risks: vec![ConsultationRisk {
                    severity: "warning".to_string(),
                    summary: "文档可能过期".to_string(),
                    mitigation: "交叉核对".to_string(),
                }],
                must_stop_points: vec!["需用户确认范围".to_string()],
                next_steps: vec!["进角色循环授权".to_string()],
                execution_scope: None, // 只读咨询 stub·不需要下游改东西
                suggest_workflow: false,
            })
        }
    }

    fn s3_make_fixture_project(name: &str) -> PathBuf {
        let dir = test_temp_dir(name);
        let docs = dir.join("docs");
        fs::create_dir_all(&docs).expect("docs dir should exist");
        fs::write(docs.join("README.md"), "# 测试项目入口\n\n进度：开发中。\n")
            .expect("readme write");
        fs::write(docs.join("01-需求.md"), "需求内容").expect("doc1 write");
        fs::write(dir.join("note.md"), "根级文档").expect("doc2 write");
        dir
    }

    #[test]
    fn s3_project_context_assembles_entry_doc_map_and_signal() {
        let dir = s3_make_fixture_project("s3-ctx-assemble");
        let ctx = load_project_context(&dir.to_string_lossy()).expect("装配应成功");
        assert!(
            ctx.entry_document
                .as_deref()
                .unwrap_or("")
                .contains("测试项目入口"),
            "入口文档全文应注入"
        );
        assert_eq!(
            ctx.project_name, "测试项目入口",
            "项目名取入口文档首个 # 标题"
        );
        assert!(
            ctx.document_map.iter().any(|p| p.ends_with("README.md")),
            "地图含 README"
        );
        assert!(
            ctx.document_map.iter().any(|p| p.contains("01-需求")),
            "地图含子目录文档"
        );
        assert!(
            ctx.document_map.iter().any(|p| p.ends_with("note.md")),
            "地图含根级文档"
        );
        assert!(
            ctx.version_signal.starts_with("mtime:") || ctx.version_signal.starts_with("git:"),
            "信号应是 git 或 mtime 降级：{}",
            ctx.version_signal
        );
        assert!(
            ctx.blackboard_summary.is_none() && ctx.memory_summary.is_none(),
            "无工作台黑板/记忆 → 空（防御式降级）"
        );
        // tier-1 策展核心：文档**正文**被注入（不只路径）——codex exec 不 on-demand 读、靠这个喂全文。
        assert!(
            ctx.injected_documents
                .iter()
                .any(|(path, content)| path.ends_with("README.md")
                    && content.contains("测试项目入口")),
            "入口文档正文应被注入（codex 靠注入读、不 on-demand）"
        );
        assert!(
            ctx.injected_documents.len() >= 2,
            "策展核心应注入多篇文档正文（README + 子目录文档），实际 {}",
            ctx.injected_documents.len()
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn s3_project_context_degrades_when_no_entry_doc() {
        let dir = test_temp_dir("s3-ctx-no-entry");
        fs::create_dir_all(&dir).expect("dir should exist");
        fs::write(dir.join("data.txt"), "无 md").expect("write");
        let ctx = load_project_context(&dir.to_string_lossy()).expect("无入口也应优雅装配");
        assert!(ctx.entry_document.is_none(), "无入口文档 → None");
        assert!(!ctx.project_name.is_empty(), "项目名退回目录名");
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn s3_consultant_full_chain_stub_feeds_c1() {
        let dir = s3_make_fixture_project("s3-fullchain");
        let project_root = dir.to_string_lossy().to_string();
        // 装配 ProjectContext → stub 咨询 → 产出
        let ctx = load_project_context(&project_root).expect("ctx");
        let agent = StubConsultant;
        let proposal = agent
            .consult(&ctx, "这个项目下一步该做什么?")
            .expect("stub 咨询");
        assert!(!proposal.goal_summary.is_empty());
        // 映射进 C1 输入
        let input = map_consultation_to_c1_input(&proposal, &project_root, "consultant-fixture")
            .expect("map 应成功（纯咨询 stub·无执行范围）");
        assert!(
            matches!(
                input.created_by_role,
                ProjectConsultationProposalCreatorRole::ProjectConsultant
            ),
            "创建者=项目咨询"
        );
        assert!(
            input.scope_draft.allowed_write_roots.is_empty(),
            "咨询只读 → 写盘根空"
        );
        assert!(!input.risks.is_empty() && !input.proposed_steps.is_empty());
        // 喂 C1：bootstrap workflow + create_proposal
        let state_dir = test_temp_dir("s3-fullchain-state");
        let path = state_dir.join("workflow-state.v0.json");
        bootstrap_project_workflow_at(&path, &fixture_project(&project_root)).expect("workflow");
        let output = project_consultation_proposal_store::create_proposal(
            &path,
            &input,
            1_765_300_000_000,
            "write-s3-consult",
        )
        .expect("ConsultationProposal 应喂得进 C1");
        assert!(!output.proposal.proposal_id.is_empty(), "C1 应建出方案");
        let _ = fs::remove_dir_all(dir);
        let _ = fs::remove_dir_all(state_dir);
    }

    // ===== P2·件 A：run_project_consultation 编排（目标 → 咨询 LM 出方案）·stub =====

    // 全链 stub：注入假咨询（不起 codex）→ 命令内层 → 方案写进 store·status=PendingUserConfirmation·没自动确认
    // （plan_authorization_id None + plan-auth store 无 active）+ map 无损（goal/risks/steps）。
    #[test]
    fn run_project_consultation_writes_pending_proposal_no_autoconfirm() {
        let proj = s3_make_fixture_project("p2-consult-stub");
        let project_root = proj.to_string_lossy().to_string();
        let state_dir = test_temp_dir("p2-consult-state");
        let path = state_dir.join("workflow-state.v0.json");
        bootstrap_project_workflow_at(&path, &fixture_project(&project_root)).expect("workflow");
        let proposal = run_project_consultation_inner(
            &path,
            &StubConsultant,
            &project_root,
            "这个项目下一步该做什么?",
            "tester",
        )
        .expect("咨询出方案应写进 store");
        assert!(!proposal.proposal_id.is_empty(), "应建出方案");
        assert!(
            matches!(
                proposal.status,
                ProjectConsultationProposalStatus::PendingUserConfirmation
            ),
            "出的方案应 PendingUserConfirmation（人闸不省·不自动确认）：{:?}",
            proposal.status
        );
        assert!(proposal.plan_authorization_id.is_none(), "不应自动建授权");
        assert!(!proposal.goal_summary.is_empty(), "方案应有目标");
        assert!(
            !proposal.risks.is_empty() && !proposal.proposed_steps.is_empty(),
            "map 无损：风险/步骤应进方案"
        );
        // 没自动授权：plan-auth store 无 active 授权。
        let auth_store =
            plan_authorization_store::load_store(&path, unix_timestamp_ms()).expect("auth store");
        assert!(
            !auth_store
                .authorizations
                .iter()
                .any(|a| matches!(a.status, PlanAuthorizationStatus::Active)),
            "咨询出方案不应自动建/激活授权"
        );
        let _ = fs::remove_dir_all(proj);
        let _ = fs::remove_dir_all(state_dir);
    }

    // 只读不碰闸：咨询命令只出方案——不起 worker、不建派发/链记录（结构性只读）。
    #[test]
    fn run_project_consultation_is_readonly_no_dispatch() {
        let proj = s3_make_fixture_project("p2-consult-readonly");
        let project_root = proj.to_string_lossy().to_string();
        let state_dir = test_temp_dir("p2-consult-readonly-state");
        let path = state_dir.join("workflow-state.v0.json");
        bootstrap_project_workflow_at(&path, &fixture_project(&project_root)).expect("workflow");
        run_project_consultation_inner(&path, &StubConsultant, &project_root, "下一步?", "tester")
            .expect("咨询出方案");
        let value = read_workflow_state_value(&path).expect("state readable");
        let empty = |key: &str| {
            value
                .get(key)
                .and_then(|v| v.as_array())
                .map(|a| a.is_empty())
                .unwrap_or(true)
        };
        assert!(empty("workflow_node_dispatches"), "咨询只读·不应建派发");
        assert!(empty("workflow_chain_runs"), "咨询只读·不应建链记录");
        let _ = fs::remove_dir_all(proj);
        let _ = fs::remove_dir_all(state_dir);
    }

    #[test]
    fn s3_readonly_consult_request_is_structurally_readonly() {
        let project = "/Users/yoyi/project/some-consult-target";
        let req = codex_local_runner::build_readonly_consult_request(project, "读项目答问题");
        // 结构性只读：read-only 沙箱、写盘根空、cwd=被咨询项目、不带授权产物
        assert_eq!(req.sandbox, "read-only", "必须只读沙箱");
        assert!(
            req.allowed_write_roots.is_empty(),
            "写盘根必须空 → 无 --add-dir、不能写"
        );
        assert_eq!(req.target_cwd, project, "cwd=被咨询项目");
        assert_eq!(req.project_root, project);
        assert!(
            req.authorization_scope_id.is_none(),
            "不带授权 scope（不走授权路）"
        );
    }

    #[test]
    fn s3_parse_consultation_proposal_extracts_json_block() {
        let raw = "我读了 docs/README.md 和红队评审。\n\n```json\n{\"user_goal\":\"核对收口\",\"goal_summary\":\"抽查 M0 红队覆盖\",\"scope_note\":\"只读核对\",\"reasoning\":[\"红队 19 条 vs 开发计划 M0 交叉\"],\"risks\":[{\"severity\":\"blocker\",\"summary\":\"有一条没接\",\"mitigation\":\"补进 M0\"}],\"must_stop_points\":[\"补全前不开工\"],\"next_steps\":[\"列出缺口\"]}\n```\n";
        let p = parse_consultation_proposal(raw).expect("应抠出 json 块");
        assert_eq!(p.goal_summary, "抽查 M0 红队覆盖");
        assert_eq!(p.risks.len(), 1);
        assert_eq!(p.risks[0].severity, "blocker");
        assert_eq!(p.reasoning.len(), 1);
    }

    #[test]
    fn s3_parse_consultation_proposal_rejects_empty() {
        assert!(
            parse_consultation_proposal("没有 json 块的纯文本").is_err(),
            "无结构化产出应报错"
        );
    }

    // S3·探针（§2C·诊断 tier-1 codex exec 到底 on-demand 读不读项目文件）：经真 consult 路 readonly_codex_consult，
    // 硬命令 codex 读「红队正文」(内容**不在**注入的 README) 并逐字引红队专属串。引到=能读(没读·修 prompt)；引不出=不读(修注入)。
    #[test]
    #[ignore = "S3 §2C diag: does tier-1 codex exec read project files on-demand (real codex)"]
    fn s3_diag_codex_reads_redteam_file() {
        let project = "/Users/yoyi/project/猫猫点菜小程序";
        let prompt = "你在只读沙箱里(只能读、不能写、不能跑命令)。请用你的只读文件读取能力，读取这个文件的正文：\n\
docs/03-评审/恋点_红队对抗评审_V1.0.md\n\
读到后，逐字原文引用它「红队结论(BLUF)」里关于微信对个人主体三条红线的那句原话(含『禁 UGC』那串)，以及严重度数量表里 P0/P1/P2 的数量。\n\
只输出你从该文件里逐字引到的原文。如果你无法读取该文件，就只回一句：无法读取文件。";
        let raw = codex_local_runner::readonly_codex_consult(project, prompt, Some(180_000))
            .unwrap_or_else(|e| format!("<consult Err: {e}>"));
        println!("[S3_PROBE] raw=\n{raw}");
        let markers = [
            "禁 UGC",
            "限本人使用",
            "暂缓全量开发",
            "验证 spike",
            "P0 阻断",
        ];
        let hit: Vec<&str> = markers
            .iter()
            .filter(|m| raw.contains(**m))
            .copied()
            .collect();
        println!(
            "[S3_PROBE] codex_read_file={} 命中红队专属标记={:?}",
            !hit.is_empty(),
            hit
        );
    }

    // S3·§6 真咨询（高危·只读上真项目·用户在场·默认 #[ignore]）：CliConsultantAgent 上猫猫点菜 + 防幻觉真题。
    // 真跑=spec §6 单独步；显式 `cargo test --lib s3_real_consult_mao_mao_dian_cai -- --ignored --nocapture` 才起真 codex。
    #[test]
    #[ignore = "S3 §6: real read-only codex consultation on a real non-test project (user present)"]
    fn s3_real_consult_mao_mao_dian_cai() {
        let project = "/Users/yoyi/project/猫猫点菜小程序";
        let ctx = load_project_context(project).expect("装配猫猫点菜 ProjectContext");
        assert!(
            ctx.entry_document.is_some(),
            "猫猫点菜应有入口文档(docs/README.md)"
        );
        let agent = CliConsultantAgent::default();
        let question = "红队 19 条说全收口，抽查开发计划 M0，有没有红队点了、开发计划没接的?";
        let proposal = agent
            .consult(&ctx, question)
            .expect("真咨询应产出结构化方案");
        println!("[S3_CONSULT] goal_summary={}", proposal.goal_summary);
        println!("[S3_CONSULT] reasoning={:?}", proposal.reasoning);
        println!("[S3_CONSULT] risks={:?}", proposal.risks);
        assert!(
            !proposal.goal_summary.trim().is_empty() && !proposal.reasoning.is_empty(),
            "答案应落地非空"
        );
        let input = map_consultation_to_c1_input(&proposal, project, "consultant").expect("map");
        assert!(!input.proposed_steps.is_empty(), "产出应喂得进 C1");
    }

    // §5 真跑·咨询判定要改东西→给执行范围块（单独步·#[ignore]·真 codex 只读·用户在场）：对"要改代码"目标 → 真咨询
    // 应给 execution_scope 块（判定改东西·非纯咨询）；map 后写范围非空(=档位) + 带 codex-dev 角色（修用户踩的
    // "写入范围为空"/"角色不在授权范围"两卡点）。flake→retry（核方案实物·记忆 real-codex-run-flaky）。
    #[test]
    #[ignore = "consultant execution-scope: real read-only codex consult on a change-goal yields an execution_scope block; map assembles profile write range (user present)"]
    fn consultant_real_consult_yields_execution_scope_for_change_goal() {
        let project = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let ctx = load_project_context(project).expect("ctx");
        let proposal = CliConsultantAgent::default()
            .consult(
                &ctx,
                "我要在这个项目里新增一个小功能、需要真改代码文件——给我落地方案，圈清要改哪些文件、怎么验收。",
            )
            .expect("真咨询应产出方案");
        println!("[CONSULT_SCOPE] goal={}", proposal.goal_summary);
        println!(
            "[CONSULT_SCOPE] execution_scope={:?}",
            proposal.execution_scope
        );
        assert!(
            proposal.execution_scope.is_some(),
            "要改代码的目标·咨询应给 execution_scope 块（判定改东西·非纯咨询/只读）"
        );
        // map 按档位装配 → 写范围非空(=固定测试项目根) + 接上 codex-dev（两卡点都不再触发）。
        let input = map_consultation_to_c1_input(&proposal, project, "consultant").expect("map");
        assert!(
            !input.scope_draft.allowed_write_roots.is_empty(),
            "map 后写范围非空"
        );
        assert!(
            input
                .scope_draft
                .allowed_role_ids
                .contains(&"codex-dev".to_string()),
            "map 后带 codex-dev 执行角色"
        );
        println!(
            "[CONSULT_SCOPE] mapped write_roots={:?} roles={:?}",
            input.scope_draft.allowed_write_roots, input.scope_draft.allowed_role_ids
        );
    }

    // ===== 咨询方案自带执行范围（execution_scope）·map 透传 / 不默认 / 护栏 / 解析向后兼容 ·stub =====

    fn consult_proposal_fixture(
        execution_scope: Option<ConsultationExecutionScope>,
    ) -> ConsultationProposal {
        ConsultationProposal {
            user_goal: "改点东西".to_string(),
            goal_summary: "在 src 下改文件".to_string(),
            scope_note: "范围".to_string(),
            reasoning: vec!["因为".to_string()],
            risks: vec![],
            must_stop_points: vec![],
            next_steps: vec!["下一步".to_string()],
            execution_scope,
            suggest_workflow: false,
        }
    }

    // 交办地基 2.1：有执行范围 → map 按**档位**装配 write/tools/roles（写死·固定测试项目·忽略咨询报的 write/tools）
    // + 仍用咨询提的 checks + target_files 进 steps。
    #[test]
    fn consult_map_assembles_profile_and_keeps_consult_checks() {
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let proposal = consult_proposal_fixture(Some(ConsultationExecutionScope {
            write_roots: vec!["咨询乱报的/路径".to_string()], // 应被忽略（用档位）
            target_files: vec!["src/main.rs".to_string()],
            tools: vec!["咨询乱报的工具".to_string()], // 应被忽略（用档位）
            checks: vec!["cargo test".to_string()],
        }));
        let input = map_consultation_to_c1_input(&proposal, test_root, "tester").expect("map");
        assert_eq!(
            input.scope_draft.allowed_write_roots,
            vec![test_root.to_string()],
            "写范围=档位（固定测试项目根·非咨询报的）"
        );
        assert!(
            input
                .scope_draft
                .allowed_tools
                .contains(&"write_file".to_string()),
            "工具=档位（含写能力·非咨询报的）"
        );
        assert_eq!(
            input.scope_draft.allowed_checks,
            vec!["cargo test".to_string()],
            "checks 仍用咨询提的"
        );
        assert!(
            input
                .scope_draft
                .allowed_role_ids
                .contains(&"codex-dev".to_string()),
            "档位含 codex-dev 执行角色（否则会卡在「目标角色不在授权范围内」）"
        );
        assert!(
            input
                .proposed_steps
                .iter()
                .any(|s| s.contains("src/main.rs")),
            "target_files 不丢·进 proposed_steps"
        );
    }

    // 纯咨询（execution_scope=None）：map 保持只读·空写范围；codex-dev 只获准交付结论，不获写能力。
    #[test]
    fn consult_map_readonly_when_no_execution_scope() {
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let input = map_consultation_to_c1_input(&consult_proposal_fixture(None), test_root, "t")
            .expect("map");
        assert!(
            input.scope_draft.allowed_write_roots.is_empty(),
            "纯咨询·写范围空（不默认兜底）"
        );
        assert_eq!(
            input.scope_draft.allowed_tools,
            vec!["read_file".to_string()],
            "纯咨询·只读工具"
        );
        assert_eq!(
            input.scope_draft.allowed_role_ids,
            vec!["project_consultant".to_string(), "codex-dev".to_string()],
            "纯咨询·允许只读 worker 交付结论"
        );
    }

    // 安全不变量（2.1 档位写死）：写范围来源是**档位·不可参数化**——咨询乱报/恶意 write_roots 一律被忽略，
    // scope 里的写范围恒 = 固定测试项目根（防"能预览任意项目"滑成"能改任意项目"）。map 不因此报错（不再有越界护栏）。
    #[test]
    fn consult_map_write_scope_hardcoded_ignores_consult_paths() {
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let proposal = consult_proposal_fixture(Some(ConsultationExecutionScope {
            write_roots: vec!["/etc".to_string(), "../../escape".to_string()],
            ..Default::default()
        }));
        let input = map_consultation_to_c1_input(&proposal, test_root, "t").expect("map");
        assert_eq!(
            input.scope_draft.allowed_write_roots,
            vec![test_root.to_string()],
            "写范围恒=档位（固定测试项目根）·咨询报的 /etc、../../escape 被忽略"
        );
    }

    // 解析·Some/None = 给没给 execution_scope 块（新口径）+ 向后兼容旧样本（带 write_roots 的老块）。
    #[test]
    fn parse_consultation_proposal_execution_scope_backward_compat() {
        // 旧样本（老 prompt·带 write_roots）→ Some（块在），write_roots 保留（下游 map 忽略·用档位）。
        let old_style = "```json\n{\"user_goal\":\"g\",\"goal_summary\":\"s\",\"scope_note\":\"n\",\"reasoning\":[\"r\"],\"risks\":[],\"must_stop_points\":[],\"next_steps\":[],\"execution_scope\":{\"write_roots\":[\"src\"],\"target_files\":[\"src/a.rs\"],\"tools\":[\"write_file\"],\"checks\":[\"cargo test\"]}}\n```";
        let es = parse_consultation_proposal(old_style)
            .expect("解析老块")
            .execution_scope
            .expect("给了块 → Some");
        assert_eq!(es.target_files, vec!["src/a.rs".to_string()]);
        // 新 prompt 样本（只报 target_files/checks·不报 write_roots）→ 仍 Some（块在=判定改东西）。
        let new_style = "```json\n{\"user_goal\":\"g\",\"goal_summary\":\"s\",\"scope_note\":\"n\",\"reasoning\":[\"r\"],\"risks\":[],\"must_stop_points\":[],\"next_steps\":[],\"execution_scope\":{\"target_files\":[\"a.rs\"],\"checks\":[\"cargo test\"]}}\n```";
        let es2 = parse_consultation_proposal(new_style)
            .expect("解析新块")
            .execution_scope
            .expect("给了块（不报 write_roots）→ 仍 Some");
        assert!(
            es2.write_roots.is_empty(),
            "新 prompt 不报 write_roots（下游用档位）"
        );
        assert_eq!(es2.checks, vec!["cargo test".to_string()]);
        // 纯咨询（无 execution_scope 块 / null）→ None。
        let pure = "```json\n{\"user_goal\":\"g\",\"goal_summary\":\"s\",\"scope_note\":\"n\",\"reasoning\":[\"r\"],\"risks\":[],\"must_stop_points\":[],\"next_steps\":[]}\n```";
        assert!(
            parse_consultation_proposal(pure)
                .expect("纯咨询仍解析")
                .execution_scope
                .is_none(),
            "无 execution_scope 块 → None（纯咨询/只读）"
        );
    }

    // P2·§5 真跑（单独步·用户在场·真咨询只读·默认 #[ignore]）：真项目（猫猫点菜·只读）+ 目标 → **经命令内层** →
    // CliConsultantAgent 真 codex 只读咨询 → 出 grounded 方案 → 写进 store（PendingUserConfirmation·没自动确认）。
    // 证「一个命令把目标→AI 出方案跑通」。显式 `cargo test --lib run_project_consultation_real_run -- --ignored --nocapture`。
    // 只读·读真实非测试项目不碰高危#1（只读豁免决策 2026-06-25）；flake → retry（咨询偶发·核方案实物）。
    #[test]
    #[ignore = "P2 consultant-LM: one command turns a goal into an AI-consulted proposal (real read-only codex) on a real project (user present)"]
    fn run_project_consultation_real_run() {
        let project = "/Users/yoyi/project/猫猫点菜小程序";
        let state_dir = test_temp_dir("p2-consult-real-state");
        let path = state_dir.join("workflow-state.v0.json");
        bootstrap_project_workflow_at(&path, &fixture_project(project)).expect("workflow");
        let consultant = CliConsultantAgent::default();
        let proposal = run_project_consultation_inner(
            &path,
            &consultant,
            project,
            "红队 19 条说全收口，抽查开发计划 M0，有没有红队点了、开发计划没接的?",
            "user-fixture",
        )
        .expect("一个命令应把目标→AI 出方案跑通");
        println!(
            "[P2_CONSULT] proposal_id={} status={:?} goal={}",
            proposal.proposal_id, proposal.status, proposal.goal_summary
        );
        println!("[P2_CONSULT] steps={:?}", proposal.proposed_steps);
        assert!(!proposal.proposal_id.is_empty(), "应建出方案");
        assert!(
            !proposal.goal_summary.trim().is_empty() && !proposal.proposed_steps.is_empty(),
            "AI 方案应落地非空（grounded）"
        );
        assert!(
            matches!(
                proposal.status,
                ProjectConsultationProposalStatus::PendingUserConfirmation
            ),
            "出方案就停·等用户确认（人闸不省）：{:?}",
            proposal.status
        );
        assert!(proposal.plan_authorization_id.is_none(), "不自动建授权");
        let _ = fs::remove_dir_all(state_dir);
    }

    // ===== S3·项目主管 agent·stub TDD（自动测试不真起 codex）=====

    fn s3_director_fixture_proposal(name: &str) -> (ProjectConsultationProposal, PathBuf) {
        let dir = test_temp_dir(name);
        let path = dir.join("workflow-state.v0.json");
        bootstrap_project_workflow_at(&path, &fixture_project(WORKFLOW_ENGINE_TEST_PROJECT_ROOT))
            .expect("workflow should exist");
        let (proposal, _auth, _rev) = create_active_project_director_authorization_fixture(
            &path,
            WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
            "thread-s3-dir-fx",
            1_765_300_000_000,
        );
        (proposal, dir)
    }

    // stub DirectorAgent：不起 codex，按 proposal 产 2 个有依赖的 task（scope 取自授权 scope_draft）。
    struct StubDirector;
    impl DirectorAgent for StubDirector {
        fn plan(
            &self,
            _ctx: &ProjectContext,
            proposal: &ProjectConsultationProposal,
        ) -> Result<Vec<ProjectDirectorPlannedTask>, String> {
            let scope = director_task_scope_from_proposal(proposal, "codex-dev");
            let mk = |id: usize, title: &str, objective: &str, deps: Vec<String>| {
                ProjectDirectorPlannedTask {
                    planned_task_id: format!("planned-task:{}:{}", proposal.workflow_id, id),
                    title: title.to_string(),
                    task_goal: objective.to_string(),
                    scope: scope.clone(),
                    depends_on: deps,
                    acceptance_criteria: vec!["可验收".to_string()],
                    report_format: vec!["做了什么".to_string()],
                    status: "planned".to_string(),
                    guard_result: None,
                    work_item_id: None,
                    workflow_node_id: None,
                    task_package_id: None,
                    memory_packet_snapshot_id: None,
                    prepared_dispatch_id: None,
                    blocked_reasons: vec![],
                }
            };
            Ok(vec![
                mk(
                    1,
                    "搭骨架",
                    &format!("就 {} 搭基础结构", proposal.goal_summary),
                    vec![],
                ),
                mk(2, "接业务", "在骨架上接业务", vec!["搭骨架".to_string()]),
            ])
        }
    }

    // ===== 方案a·新会话出生口 stubs（合流 session_choice=new 单测用·不碰真 relay）=====
    // 成功桩：记录收到的初始化文案、返回固定 thread_id。
    struct StubJiaobanSessionCreator {
        thread_id: &'static str,
        received_texts: std::cell::RefCell<Vec<String>>,
    }
    impl JiaobanNewSessionCreator for StubJiaobanSessionCreator {
        fn create_initialized_session(
            &self,
            initialization_text: &str,
            _requested_by: &str,
        ) -> Result<String, String> {
            self.received_texts
                .borrow_mut()
                .push(initialization_text.to_string());
            Ok(self.thread_id.to_string())
        }
    }
    // 失败桩：模拟 relay 建会话失败（人话原因）。
    struct FailingJiaobanSessionCreator {
        called: std::cell::RefCell<bool>,
    }
    impl JiaobanNewSessionCreator for FailingJiaobanSessionCreator {
        fn create_initialized_session(
            &self,
            _initialization_text: &str,
            _requested_by: &str,
        ) -> Result<String, String> {
            *self.called.borrow_mut() = true;
            Err("codex 起不来（stub）".to_string())
        }
    }
    // 炸桩：existing 分支/被拒路径**绝不该**碰新会话出生口——碰到即 panic（回归护栏）。
    struct PanicJiaobanSessionCreator;
    impl JiaobanNewSessionCreator for PanicJiaobanSessionCreator {
        fn create_initialized_session(
            &self,
            _initialization_text: &str,
            _requested_by: &str,
        ) -> Result<String, String> {
            panic!("此路径不该触发新会话出生口（existing/被拒路径必须与 relay 无关）");
        }
    }

    #[test]
    fn s3_director_stub_plans_valid_tasks_feed_prepare() {
        let (proposal, dir) = s3_director_fixture_proposal("s3-director-stub");
        let ctx = load_project_context(WORKFLOW_ENGINE_TEST_PROJECT_ROOT).expect("ctx");
        let plan = StubDirector.plan(&ctx, &proposal).expect("director plan");
        assert_eq!(plan.len(), 2);
        assert!(!plan[0].title.is_empty() && !plan[0].task_goal.is_empty());
        // scope 取自**已授权 scope_draft**（LM 不扩范围）
        assert_eq!(
            plan[0].scope.allowed_write_scope, proposal.scope_draft.allowed_write_roots,
            "写盘 scope 取自授权 scope_draft"
        );
        assert_eq!(plan[0].scope.project_id, proposal.project_id);
        // depends_on 自洽：task2 依赖 task1.title
        assert!(plan[1].depends_on.contains(&plan[0].title), "依赖自洽");
        // 下游字段留空（主管不填，由 prepare/派发机器填）
        assert!(plan[0].work_item_id.is_none() && plan[0].prepared_dispatch_id.is_none());
        // 喂得进 prepare_authorized_auto_dispatch 入参（映射对）
        let prepare_input = fixture_project_director_prepare_input(
            WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
            &proposal.proposal_id,
            &proposal
                .plan_authorization_id
                .clone()
                .unwrap_or_else(|| "plan-auth:x".to_string()),
            1,
            plan.clone(),
        );
        assert_eq!(
            prepare_input.planned_tasks.len(),
            2,
            "planned_tasks 喂进 prepare 入参"
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn s3_director_parse_plan_extracts_tasks() {
        let (proposal, dir) = s3_director_fixture_proposal("s3-director-parse");
        let raw = "拆解如下:\n```json\n[{\"title\":\"建表\",\"objective\":\"建 7 集合\",\"target_role\":\"codex-dev\",\"depends_on\":[],\"acceptance_criteria\":[\"表建好\"],\"report_format\":[\"建了哪些表\"]},{\"title\":\"云函数公共层\",\"objective\":\"取 OPENID 中间件\",\"target_role\":\"codex-dev\",\"depends_on\":[\"建表\"],\"acceptance_criteria\":[\"中间件通\"],\"report_format\":[\"接口清单\"]}]\n```";
        let plan = parse_director_plan(raw, &proposal).expect("parse director plan");
        assert_eq!(plan.len(), 2);
        assert_eq!(plan[0].title, "建表");
        assert_eq!(plan[1].depends_on, vec!["建表".to_string()]);
        // scope 取自授权 scope_draft；下游字段留空
        assert_eq!(
            plan[0].scope.allowed_write_scope,
            proposal.scope_draft.allowed_write_roots
        );
        assert!(plan[1].work_item_id.is_none());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn s3_director_parse_plan_rejects_empty_or_bad() {
        let (proposal, dir) = s3_director_fixture_proposal("s3-director-parse-bad");
        assert!(parse_director_plan("没有 json 块", &proposal).is_err());
        assert!(
            parse_director_plan("```json\n[]\n```", &proposal).is_err(),
            "空任务列表应报错"
        );
        let _ = fs::remove_dir_all(dir);
    }

    // S3·项目主管真跑（§5 单独步·用户在场·只读·默认 #[ignore]）：真 LM 拆解已授权方案 → planned_tasks。
    // 显式 `cargo test --lib s3_director_real_plan -- --ignored --nocapture` 才起真 codex（只读 confinement·项目不可写）。
    #[test]
    #[ignore = "S3 director: real read-only LM decomposition of an authorized proposal (user present)"]
    fn s3_director_real_plan() {
        let (proposal, dir) = s3_director_fixture_proposal("s3-director-real");
        let ctx = load_project_context(WORKFLOW_ENGINE_TEST_PROJECT_ROOT).expect("ctx");
        let plan = CliDirectorAgent::default()
            .plan(&ctx, &proposal)
            .expect("real director plan should return planned_tasks");
        println!("[S3_DIRECTOR] task_count={}", plan.len());
        for t in &plan {
            println!(
                "[S3_DIRECTOR] - {} | task_goal={} | depends_on={:?}",
                t.title, t.task_goal, t.depends_on
            );
        }
        assert!(!plan.is_empty(), "应拆出至少 1 个任务");
        assert!(
            plan.iter()
                .all(|t| !t.title.trim().is_empty() && !t.task_goal.trim().is_empty()),
            "每个任务 title/task_goal 非空"
        );
        // scope 仍取自授权 scope_draft（LM 不扩范围）
        assert!(plan
            .iter()
            .all(|t| t.scope.allowed_write_scope == proposal.scope_draft.allowed_write_roots));
        let _ = fs::remove_dir_all(dir);
    }

    // S3·主管档案钉死「自包含任务」（修真跑根因：worker 隔离上下文只看 task_goal、拿不到方案）。
    #[test]
    fn s3_director_prompt_requires_self_contained_tasks() {
        let (proposal, dir) = s3_director_fixture_proposal("s3-director-selfcontained");
        let ctx = load_project_context(WORKFLOW_ENGINE_TEST_PROJECT_ROOT).expect("ctx");
        let prompt = director_build_prompt(&ctx, &proposal);
        assert!(prompt.contains("自包含"), "档案应要求自包含任务");
        assert!(
            prompt.contains("绝不") && prompt.contains("参见"),
            "应明令禁「参见方案/上文」引用"
        );
        assert!(
            prompt.contains("完整路径"),
            "应要求把目标文件完整路径写进 task_goal"
        );
        assert!(prompt.contains("结构化返回"), "应要求 worker 结构化返回");
        let _ = fs::remove_dir_all(dir);
    }

    // ===== S3·主管→派发联调（LM planned_tasks → prepare → S1 闸 → worker）·stub 集成 =====
    // 复用 S2-3 全链结构，把 prepare 的 vec![]兜底 换成 **director 的显式 planned_tasks**；闸/派发/沙箱 0-diff。
    #[test]
    fn s3_director_dispatch_integration_stub() {
        let timestamp_ms = 1_765_300_000_000;
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let thread_id = "thread-s3-dir-dispatch";
        let dir = test_temp_dir("s3-director-dispatch");
        let path = dir.join("workflow-state.v0.json");
        let index_path = dir.join("codex-index.json");
        let index = fixture_dispatch_index(test_root, thread_id);
        bootstrap_project_workflow_at(&path, &fixture_project(test_root)).expect("workflow");
        let (proposal, authorization, revision) =
            create_active_project_director_authorization_fixture(
                &path,
                test_root,
                thread_id,
                timestamp_ms,
            );
        let workflow_id = default_workflow_id(test_root);
        let node_id = format!("{workflow_id}:node:codex-dev");
        bind_workflow_node_codex_session_for_index_at(
            &path,
            &index,
            &fixture_node_session_bind_request(test_root, &node_id, None, thread_id),
        )
        .expect("bind codex-dev");
        let ctx = load_project_context(test_root).expect("ctx");
        // 主管 LM(stub) 拆 planned_tasks（target_role=codex-dev → c4_node_id 映射到已绑节点）
        let planned = StubDirector.plan(&ctx, &proposal).expect("director plan");
        // prepare 用 director **显式** planned_tasks（非 vec![] 兜底）
        let prepared = prepare_authorized_auto_dispatch_for_index_at(
            &path,
            &index,
            &fixture_project_director_prepare_input(
                test_root,
                &proposal.proposal_id,
                &authorization.authorization_id,
                revision,
                planned.clone(),
            ),
        )
        .expect("prepare with director planned_tasks");
        assert!(
            !prepared.prepared_dispatches.is_empty(),
            "director planned_tasks 应过授权 guard 产出 prepared dispatch"
        );
        let prep = &prepared.prepared_dispatches[0];
        let prep_node = prep.workflow_node_id.clone().expect("prepared node");
        let prep_work_item = prep.work_item_id.clone().expect("prepared work_item");
        assert_eq!(prep_node, node_id, "director 任务(codex-dev)映射到已绑节点");
        // execute 过 S1 闸（stub runner）→ worker 跑 LM 计划的任务
        let runner = PermissiveExperimentRunner {
            stats: CodexDispatchReadbackStats {
                transcript_event_count: 3,
                transcript_target_hits: 1,
            },
        };
        let run = execute_project_workflow_node_at(
            &path,
            &index,
            &index_path,
            &runner,
            &ProjectWorkflowNodeRunRequest {
                project_root: test_root.to_string(),
                node_id: prep_node,
                work_item_id: prep_work_item,
                workflow_id: Some(workflow_id),
            },
        )
        .expect("worker 经 S1 闸应授权放行并 completed");
        assert_eq!(
            run.dispatch.state, "completed",
            "LM 主管计划真驱动的 worker 应 completed"
        );
        let _ = fs::remove_dir_all(dir);
    }

    // S3·主管→派发真跑（§5 单独步·用户在场·固定测试项目·默认 #[ignore]）：真 director LM 拆已授权方案 →
    // prepare → 派第一个任务 → worker 真 codex 跑出真结果。自定义 proof-goal 方案，让 director 拆出写文件任务。
    // 显式 `cargo test --lib s3_director_dispatch_real_run -- --ignored --nocapture` 才起真 codex。
    #[test]
    #[ignore = "S3 director-dispatch: real LM plan drives a real worker codex run in the test project (user present)"]
    fn s3_director_dispatch_real_run() {
        let timestamp_ms = 1_765_300_000_000;
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let real_session = "019ed9f7-c0c2-7213-b871-6d18959b7c24";
        let proof_token = unix_timestamp_string();
        let proof_path = format!("{test_root}/s3-director-dispatch-proof.txt");
        let _ = fs::remove_file(&proof_path);
        let dir = test_temp_dir("s3-director-dispatch-real");
        let path = dir.join("workflow-state.v0.json");
        let index = fixture_dispatch_index(test_root, real_session);
        bootstrap_project_workflow_at(&path, &fixture_project(test_root)).expect("workflow");
        // 自定义 proof-goal 方案 → 确认建授权 → 边界复核激活
        let mut proposal_input = fixture_project_consultation_proposal_input(test_root);
        proposal_input.scope_draft.allowed_agent_ids = vec![real_session.to_string()];
        proposal_input.goal_summary = format!(
            "在当前项目根目录创建文件 s3-director-dispatch-proof.txt，只写入一行：s3 director dispatch ok {proof_token}"
        );
        proposal_input.proposed_steps = vec![format!(
            "创建文件 s3-director-dispatch-proof.txt，写入一行内容：s3 director dispatch ok {proof_token}"
        )];
        let created = project_consultation_proposal_store::create_proposal(
            &path,
            &proposal_input,
            timestamp_ms,
            "write-s3-dispatch-proposal",
        )
        .expect("proposal");
        let confirmed = project_consultation_proposal_store::record_decision(
            &path,
            &RecordProjectConsultationProposalDecisionInput {
                project_root: test_root.to_string(),
                proposal_id: created.proposal.proposal_id.clone(),
                actor_id: "user-fixture".to_string(),
                decision: ProjectConsultationProposalDecisionKind::Confirm,
                summary: "用户确认 S3 派发真跑方案。".to_string(),
                expected_proposal_store_revision: Some(created.store_revision),
                expected_plan_authorization_store_revision: None,
            },
            timestamp_ms + 1,
            "write-s3-dispatch-confirm",
            "write-s3-dispatch-auth",
            "write-s3-dispatch-auth-user",
        )
        .expect("confirm");
        let authorization = confirmed.plan_authorization.expect("authorization");
        let revision = confirmed
            .plan_authorization_store_revision
            .expect("revision");
        let activated = plan_authorization_store::record_global_boundary_review_with_proposal(
            &path,
            &fixture_global_boundary_review_input(
                test_root,
                &confirmed.proposal.proposal_id,
                &authorization.authorization_id,
                revision,
            ),
            timestamp_ms + 2,
            "write-s3-dispatch-boundary",
        )
        .expect("boundary review activate");
        // 真 director LM 拆解
        let ctx = load_project_context(test_root).expect("ctx");
        let planned = CliDirectorAgent::default()
            .plan(&ctx, &confirmed.proposal)
            .expect("real director plan");
        println!("[S3_DISPATCH] director planned {} task(s)", planned.len());
        for t in &planned {
            println!("[S3_DISPATCH]   task: {} | {}", t.title, t.task_goal);
        }
        assert!(!planned.is_empty(), "director 应拆出任务");
        // bind + prepare(director planned_tasks)
        let workflow_id = default_workflow_id(test_root);
        let node_id = format!("{workflow_id}:node:codex-dev");
        bind_workflow_node_codex_session_for_index_at(
            &path,
            &index,
            &fixture_node_session_bind_request(test_root, &node_id, None, real_session),
        )
        .expect("bind");
        let prepared = prepare_authorized_auto_dispatch_for_index_at(
            &path,
            &index,
            &fixture_project_director_prepare_input(
                test_root,
                &confirmed.proposal.proposal_id,
                &activated.authorization.authorization_id,
                activated.store_revision,
                planned.clone(),
            ),
        )
        .expect("prepare");
        let prep = &prepared.prepared_dispatches[0];
        // worker 真 codex 跑第一个任务（过 S1 闸）
        let runner = codex_local_runner::RealWorkflowNodeCodexRunner;
        let readback_db_path = codex_db::default_state_db_path();
        let run = execute_project_workflow_node_at(
            &path,
            &index,
            &readback_db_path,
            &runner,
            &ProjectWorkflowNodeRunRequest {
                project_root: test_root.to_string(),
                node_id: prep.workflow_node_id.clone().expect("node"),
                work_item_id: prep.work_item_id.clone().expect("work_item"),
                workflow_id: Some(workflow_id),
            },
        )
        .expect("worker 经 S1 闸真跑应 completed");
        println!(
            "[S3_DISPATCH] worker state={} exit={:?} summary={:?}",
            run.dispatch.state, run.dispatch.exit_code, run.dispatch.last_message_summary
        );
        assert_eq!(run.dispatch.state, "completed");
        let proof = fs::read_to_string(&proof_path)
            .unwrap_or_else(|e| panic!("LM 计划应驱动 worker 建 proof {proof_path}：{e}"));
        assert!(
            proof.contains(&proof_token),
            "proof 应含本次 token {proof_token}，实际：{proof}"
        );
        println!("[S3_DISPATCH] proof={proof:?}");
        let _ = fs::remove_dir_all(dir);
    }

    // S3·§5 真链跑（单独步·用户在场·固定测试项目·默认 #[ignore]）：真 director LM 把多步 proof-goal 方案
    // 拆成**自包含任务链**（建文件→回读核验→建第二文件）→ **经 C1 起链请求（StartProjectDirectorChainRequest·
    // 前端回传已审计划）走命令路径** → 薄驱动按 depends_on 序逐任务过 S1 闸真跑 → **多 worker 真 codex 接连跑**
    // → 最终真建出 proof + 回读核验。证「LM 多步计划经 app 命令路径真驱动 worker 链干成事」（C1·§5）。
    // 显式 `cargo test --lib s3_director_chain_real_run -- --ignored --nocapture` 才起真 codex。
    #[test]
    #[ignore = "S3 director-chain C1: real LM multi-step plan drives a real codex chain through the start-chain command request path (user present)"]
    fn s3_director_chain_real_run() {
        let timestamp_ms = 1_765_300_000_000;
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let real_session = "019ed9f7-c0c2-7213-b871-6d18959b7c24";
        let proof_token = unix_timestamp_string();
        let proof_a = format!("{test_root}/s3-chain-proof-a.txt");
        let proof_b = format!("{test_root}/s3-chain-proof-b.txt");
        let _ = fs::remove_file(&proof_a);
        let _ = fs::remove_file(&proof_b);
        let dir = test_temp_dir("s3-director-chain-real");
        let path = dir.join("workflow-state.v0.json");
        let index = fixture_dispatch_index(test_root, real_session);
        bootstrap_project_workflow_at(&path, &fixture_project(test_root)).expect("workflow");
        // 多步 proof-goal 方案（强制拆 ≥2 步、第二步回读依赖第一步）→ 确认建授权 → 边界复核激活。
        let mut proposal_input = fixture_project_consultation_proposal_input(test_root);
        proposal_input.scope_draft.allowed_agent_ids = vec![real_session.to_string()];
        // 授权 scope 与 proof 写入目标对齐（authorize 测试项目根）——证 §3「worker 写范围 = 授权 scope 派生·不扩」，
        // 不让授权(/src)与目标(根)自相矛盾。注：真实沙箱写界本就是 project_root（commands.rs·path-lock 限测试项目），
        // 此处只把授权记录摆正、与实际一致。
        proposal_input.scope_draft.allowed_write_roots = vec![test_root.to_string()];
        proposal_input.scope_draft.allowed_read_roots = vec![test_root.to_string()];
        proposal_input.goal_summary = format!(
            "分两步在当前项目根目录建证据，第二步依赖第一步：① 创建 s3-chain-proof-a.txt，只写入一行：a {proof_token}；② 先读回 s3-chain-proof-a.txt 确认其内容含 {proof_token}，确认后再创建 s3-chain-proof-b.txt，只写入一行：verified {proof_token}。"
        );
        proposal_input.proposed_steps = vec![
            format!("创建 s3-chain-proof-a.txt，写入一行：a {proof_token}"),
            format!(
                "读回 s3-chain-proof-a.txt 核验含 {proof_token}，再创建 s3-chain-proof-b.txt 写入一行：verified {proof_token}（本步依赖上一步的产出）"
            ),
        ];
        let created = project_consultation_proposal_store::create_proposal(
            &path,
            &proposal_input,
            timestamp_ms,
            "write-s3-chain-proposal",
        )
        .expect("proposal");
        let confirmed = project_consultation_proposal_store::record_decision(
            &path,
            &RecordProjectConsultationProposalDecisionInput {
                project_root: test_root.to_string(),
                proposal_id: created.proposal.proposal_id.clone(),
                actor_id: "user-fixture".to_string(),
                decision: ProjectConsultationProposalDecisionKind::Confirm,
                summary: "用户确认 S3 真链跑方案。".to_string(),
                expected_proposal_store_revision: Some(created.store_revision),
                expected_plan_authorization_store_revision: None,
            },
            timestamp_ms + 1,
            "write-s3-chain-confirm",
            "write-s3-chain-auth",
            "write-s3-chain-auth-user",
        )
        .expect("confirm");
        let authorization = confirmed.plan_authorization.expect("authorization");
        let revision = confirmed
            .plan_authorization_store_revision
            .expect("revision");
        let activated = plan_authorization_store::record_global_boundary_review_with_proposal(
            &path,
            &fixture_global_boundary_review_input(
                test_root,
                &confirmed.proposal.proposal_id,
                &authorization.authorization_id,
                revision,
            ),
            timestamp_ms + 2,
            "write-s3-chain-boundary",
        )
        .expect("boundary review activate");
        // 真 director LM 拆解成自包含任务链（应 ≥2 任务、带 depends_on）。
        let ctx = load_project_context(test_root).expect("ctx");
        let planned = CliDirectorAgent::default()
            .plan(&ctx, &confirmed.proposal)
            .expect("real director plan");
        println!("[S3_CHAIN] director planned {} task(s)", planned.len());
        for t in &planned {
            println!(
                "[S3_CHAIN]   task: {} | deps={:?} | {}",
                t.title, t.depends_on, t.task_goal
            );
        }
        assert!(
            planned.len() >= 2,
            "director 应把多步方案拆成 ≥2 任务的链（实际 {}）",
            planned.len()
        );
        // 自包含核验：task_goal 不含「参见方案/上文」引用词（worker 在隔离上下文拿不到方案）。
        for t in &planned {
            for bad in ["参见", "上文", "上一步", "如方案", "见方案"] {
                assert!(
                    !t.task_goal.contains(bad),
                    "任务「{}」task_goal 含引用词「{bad}」，非自包含：{}",
                    t.title,
                    t.task_goal
                );
            }
        }
        // 真实依赖链核验（证「核对→创建→回读」非两个独立写）：须有带 depends_on 的任务，且其 task_goal
        // 真的把「回读 proof_a + 核验」写进去——否则只是两个无关写文件、谈不上链。
        let dependent = planned
            .iter()
            .find(|t| !t.depends_on.is_empty())
            .expect("应有带 depends_on 的任务（链有真实依赖，非平行写）");
        assert!(
            ["读回", "读取", "proof_a", "核验", "读 proof", "读 s3-chain-proof-a"]
                .iter()
                .any(|kw| dependent.task_goal.contains(kw)),
            "依赖任务 task_goal 应含回读/核验 proof_a 的步骤（证 LM 拆的是 核对→创建→回读 真依赖链），实际：{}",
            dependent.task_goal
        );
        // bind + prepare(director planned_tasks)。
        let workflow_id = default_workflow_id(test_root);
        let node_id = format!("{workflow_id}:node:codex-dev");
        bind_workflow_node_codex_session_for_index_at(
            &path,
            &index,
            &fixture_node_session_bind_request(test_root, &node_id, None, real_session),
        )
        .expect("bind");
        let prepared = prepare_authorized_auto_dispatch_for_index_at(
            &path,
            &index,
            &fixture_project_director_prepare_input(
                test_root,
                &confirmed.proposal.proposal_id,
                &activated.authorization.authorization_id,
                activated.store_revision,
                planned.clone(),
            ),
        )
        .expect("prepare");
        let prepared_count = prepared
            .plan
            .planned_tasks
            .iter()
            .filter(|t| t.status == "prepared")
            .count();
        println!("[S3_CHAIN] prepared {prepared_count} task(s)");
        assert!(
            prepared_count >= 2,
            "应有 ≥2 个 prepared 任务真跑成链（实际 {prepared_count}）"
        );
        // C1·走命令路径：模拟前端把 prepare 返回的「已审 planned_tasks」经 JSON 回传给 start_project_director_chain
        // 请求（**不重跑 LM**），下面按命令内层（spawn_blocking 闭包）的同款调用真跑这份回传计划。
        let request: StartProjectDirectorChainRequest = serde_json::from_value(serde_json::json!({
            "project_root": test_root,
            "workflow_id": workflow_id,
            "planned_tasks": prepared.plan.planned_tasks,
            "max_nodes": 50,
        }))
        .expect("起链请求应能反序列化前端回传的已审计划");
        // 薄驱动按 depends_on 序真跑多 worker（每步过 S1 闸·真 codex）——经命令请求字段（= start_project_director_chain 内层）。
        let runner = codex_local_runner::RealWorkflowNodeCodexRunner;
        let readback_db_path = codex_db::default_state_db_path();
        let outcome = run_director_task_chain(
            &path,
            &index,
            &readback_db_path,
            &runner,
            &request.project_root,
            &request.workflow_id,
            &request.planned_tasks,
            request.max_nodes.unwrap_or(50),
        )
        .expect("真链跑应返回结果");
        println!(
            "[S3_CHAIN] outcome total={} dispatched={} completed={} skipped={} stop={:?}",
            outcome.total,
            outcome.dispatched,
            outcome.completed,
            outcome.skipped,
            outcome.stopped_reason
        );
        for s in &outcome.steps {
            println!("[S3_CHAIN]   step: {} -> {}", s.title, s.state);
        }
        assert!(
            outcome.stopped_reason.is_none(),
            "链应按序全跑完无失败：{:?}",
            outcome.stopped_reason
        );
        assert!(
            outcome.dispatched >= 2,
            "应真派发 ≥2 个任务（防驱动循环提前早退），实际 {}",
            outcome.dispatched
        );
        assert!(
            outcome.completed >= 2,
            "应 ≥2 个 worker 接连真跑完成（实际 {}）",
            outcome.completed
        );
        // 最终 proof：两个文件都真建出、内容含本次 token（回读核验在第二步真发生）。
        let a = fs::read_to_string(&proof_a)
            .unwrap_or_else(|e| panic!("第一步应建 proof_a {proof_a}：{e}"));
        let b = fs::read_to_string(&proof_b)
            .unwrap_or_else(|e| panic!("第二步（依赖回读）应建 proof_b {proof_b}：{e}"));
        assert!(
            a.contains(&proof_token),
            "proof_a 应含 token {proof_token}，实际：{a}"
        );
        assert!(
            b.contains(&proof_token),
            "proof_b 应含 token {proof_token}，实际：{b}"
        );
        println!("[S3_CHAIN] proof_a={a:?} proof_b={b:?}");
        let _ = fs::remove_dir_all(dir);
    }

    // C1·§5 补：真·中途停（单独步·用户在场·固定测试项目·默认 #[ignore]）：真 director LM 拆 2-任务依赖链 →
    // 经命令路径起链真跑 → **另起线程在 task1 链节点变 running 时调现成 stop_project_workflow_chain_at（=用户点停）**
    // → task2 边界抓到 flag 被跳。证「真 codex 链 + 现成停命令」app 层中途停端到端：proof_a 在、proof_b 不在。
    // 显式 `cargo test --lib s3_director_chain_real_run_interrupts_mid_chain -- --ignored --nocapture` 才起真 codex。
    // flake（真 codex）：task1 偶发早退→completed==0；极端时序 task2 抢跑→completed==2。皆 flake → retry（核 proof 实物分辨）。
    #[test]
    #[ignore = "S3 director-chain C1: real codex chain stopped mid-way via the real stop command (user present, timing-based, retry on flake)"]
    fn s3_director_chain_real_run_interrupts_mid_chain() {
        let timestamp_ms = 1_765_300_000_000;
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let real_session = "019ed9f7-c0c2-7213-b871-6d18959b7c24";
        let proof_token = unix_timestamp_string();
        let proof_a = format!("{test_root}/s3-chain-midstop-proof-a.txt");
        let proof_b = format!("{test_root}/s3-chain-midstop-proof-b.txt");
        let _ = fs::remove_file(&proof_a);
        let _ = fs::remove_file(&proof_b);
        let dir = test_temp_dir("s3-director-chain-midstop");
        let path = dir.join("workflow-state.v0.json");
        let index = fixture_dispatch_index(test_root, real_session);
        bootstrap_project_workflow_at(&path, &fixture_project(test_root)).expect("workflow");
        // 多步 proof-goal 方案（同 §5·独立 proof 文件名）→ 确认 → 边界复核激活。
        let mut proposal_input = fixture_project_consultation_proposal_input(test_root);
        proposal_input.scope_draft.allowed_agent_ids = vec![real_session.to_string()];
        proposal_input.scope_draft.allowed_write_roots = vec![test_root.to_string()];
        proposal_input.scope_draft.allowed_read_roots = vec![test_root.to_string()];
        proposal_input.goal_summary = format!(
            "分两步在当前项目根目录建证据，第二步依赖第一步：① 创建 s3-chain-midstop-proof-a.txt，只写入一行：a {proof_token}；② 先读回 s3-chain-midstop-proof-a.txt 确认其内容含 {proof_token}，确认后再创建 s3-chain-midstop-proof-b.txt，只写入一行：verified {proof_token}。"
        );
        proposal_input.proposed_steps = vec![
            format!("创建 s3-chain-midstop-proof-a.txt，写入一行：a {proof_token}"),
            format!(
                "读回 s3-chain-midstop-proof-a.txt 核验含 {proof_token}，再创建 s3-chain-midstop-proof-b.txt 写入一行：verified {proof_token}（本步依赖上一步的产出）"
            ),
        ];
        let created = project_consultation_proposal_store::create_proposal(
            &path,
            &proposal_input,
            timestamp_ms,
            "write-s3-midstop-proposal",
        )
        .expect("proposal");
        let confirmed = project_consultation_proposal_store::record_decision(
            &path,
            &RecordProjectConsultationProposalDecisionInput {
                project_root: test_root.to_string(),
                proposal_id: created.proposal.proposal_id.clone(),
                actor_id: "user-fixture".to_string(),
                decision: ProjectConsultationProposalDecisionKind::Confirm,
                summary: "用户确认 S3 中途停真跑方案。".to_string(),
                expected_proposal_store_revision: Some(created.store_revision),
                expected_plan_authorization_store_revision: None,
            },
            timestamp_ms + 1,
            "write-s3-midstop-confirm",
            "write-s3-midstop-auth",
            "write-s3-midstop-auth-user",
        )
        .expect("confirm");
        let authorization = confirmed.plan_authorization.expect("authorization");
        let revision = confirmed
            .plan_authorization_store_revision
            .expect("revision");
        let activated = plan_authorization_store::record_global_boundary_review_with_proposal(
            &path,
            &fixture_global_boundary_review_input(
                test_root,
                &confirmed.proposal.proposal_id,
                &authorization.authorization_id,
                revision,
            ),
            timestamp_ms + 2,
            "write-s3-midstop-boundary",
        )
        .expect("boundary review activate");
        let ctx = load_project_context(test_root).expect("ctx");
        let planned = CliDirectorAgent::default()
            .plan(&ctx, &confirmed.proposal)
            .expect("real director plan");
        assert!(
            planned.len() >= 2,
            "应拆出 ≥2 任务的链（实际 {}）",
            planned.len()
        );
        let workflow_id = default_workflow_id(test_root);
        let node_id = format!("{workflow_id}:node:codex-dev");
        bind_workflow_node_codex_session_for_index_at(
            &path,
            &index,
            &fixture_node_session_bind_request(test_root, &node_id, None, real_session),
        )
        .expect("bind");
        let prepared = prepare_authorized_auto_dispatch_for_index_at(
            &path,
            &index,
            &fixture_project_director_prepare_input(
                test_root,
                &confirmed.proposal.proposal_id,
                &activated.authorization.authorization_id,
                activated.store_revision,
                planned.clone(),
            ),
        )
        .expect("prepare");
        let prepared_count = prepared
            .plan
            .planned_tasks
            .iter()
            .filter(|t| t.status == "prepared")
            .count();
        assert!(
            prepared_count >= 2,
            "应有 ≥2 个 prepared 任务（实际 {prepared_count}）"
        );
        // task1 = 无依赖那个（链记录节点按 planned_task_id 编址，停链线程据此查 task1 边界）。
        let task1_id = prepared
            .plan
            .planned_tasks
            .iter()
            .find(|t| t.depends_on.is_empty() && t.status == "prepared")
            .map(|t| t.planned_task_id.clone())
            .expect("应有无依赖的 task1");
        // 经 C1 起链请求（前端回传已审计划·走命令路径）。
        let request: StartProjectDirectorChainRequest = serde_json::from_value(serde_json::json!({
            "project_root": test_root,
            "workflow_id": workflow_id,
            "planned_tasks": prepared.plan.planned_tasks,
            "max_nodes": 50,
        }))
        .expect("起链请求反序列化");
        // 「用户点停」模拟线程：本链 task1 节点变 running/completed（= task1 边界已过、worker 在跑/已完）时，调
        // **现成** stop_project_workflow_chain_at 置 flag。设在 task1 的 ~30s running 窗口内 → task2 边界稳抓到。
        // 轮询上限防卡死（600×100ms=60s）。
        let stop_path = path.clone();
        let stop_workflow_id = workflow_id.clone();
        let stopper = std::thread::spawn(move || {
            for _ in 0..600 {
                if let Ok(value) = read_workflow_state_value(&stop_path) {
                    if let Some(run) = latest_chain_run_for(
                        &value,
                        WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
                        &stop_workflow_id,
                    ) {
                        if let Some(chain_run_id) = optional_string_from(&run, "chain_run_id") {
                            let node_state = chain_node_state(&value, &chain_run_id, &task1_id);
                            if matches!(node_state.as_deref(), Some("running") | Some("completed"))
                            {
                                // stop_issued 反映停链命令是否真成功（非「调过」），好分辨 flake 与真 bug。
                                return stop_project_workflow_chain_at(
                                    &stop_path,
                                    &ProjectWorkflowChainStopRequest {
                                        project_root: WORKFLOW_ENGINE_TEST_PROJECT_ROOT.to_string(),
                                        workflow_id: stop_workflow_id.clone(),
                                    },
                                )
                                .is_ok();
                            }
                        }
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            false
        });
        // 主线程：经命令内层真跑链（真 codex）。
        let runner = codex_local_runner::RealWorkflowNodeCodexRunner;
        let readback_db_path = codex_db::default_state_db_path();
        let outcome = run_director_task_chain(
            &path,
            &index,
            &readback_db_path,
            &runner,
            &request.project_root,
            &request.workflow_id,
            &request.planned_tasks,
            request.max_nodes.unwrap_or(50),
        )
        .expect("真链跑应返回结果");
        let stop_issued = stopper.join().unwrap_or(false);
        println!(
            "[S3_MIDSTOP] stop_issued={stop_issued} completed={} dispatched={} stop={:?}",
            outcome.completed, outcome.dispatched, outcome.stopped_reason
        );
        for s in &outcome.steps {
            println!("[S3_MIDSTOP]   step: {} -> {}", s.title, s.state);
        }
        assert!(stop_issued, "停链线程应在 task1 边界后真发出停链命令");
        // 中途停核验：task1 完、task2 停在边界。
        assert_eq!(
            outcome.stopped_reason.as_deref(),
            Some("user_stop_requested"),
            "应因用户停链在边界停：{:?}（flake：早退→completed0/时序→completed2，retry）",
            outcome.stopped_reason
        );
        assert_eq!(
            outcome.completed, 1,
            "task1 完成、task2 被停（实际 {}·flake retry）",
            outcome.completed
        );
        assert_eq!(
            outcome.dispatched, 1,
            "只派发 task1（实际 {}）",
            outcome.dispatched
        );
        // 最关键实物：proof_a 真建（task1 跑了·含本次 token）、proof_b 不存在（task2 从没跑）= 真停在两任务之间。
        let a = fs::read_to_string(&proof_a)
            .unwrap_or_else(|e| panic!("task1 应真建 proof_a {proof_a}：{e}"));
        assert!(
            a.contains(&proof_token),
            "proof_a 应含本次 token {proof_token}，实际：{a}"
        );
        assert!(
            !std::path::Path::new(&proof_b).exists(),
            "task2 应被中途停、proof_b 不该存在 {proof_b}"
        );
        // 链记录 stopped + 停链审计。
        let value = read_workflow_state_value(&path).expect("state readable");
        let run = latest_chain_run_for(&value, test_root, &workflow_id).expect("应能读到本链记录");
        assert_eq!(
            optional_string_from(&run, "state").as_deref(),
            Some("stopped"),
            "链记录应 state=stopped"
        );
        assert!(
            audit_has(&path, "workflow_chain_run_stopped"),
            "应有停链审计事件"
        );
        let _ = fs::remove_dir_all(dir);
    }

    // ===== S3·主管→worker 多任务依赖链（薄驱动·拓扑序+失败即停+runaway）·stub =====

    // 共享 setup：bind codex-dev + create_active 方案 + StubDirector 多任务 + prepare → 返回链驱动所需件。
    fn s3_director_prepared_chain(
        name: &str,
    ) -> (
        PathBuf,
        PathBuf,
        Value,
        PathBuf,
        String,
        AuthorizedPreparedDispatchResult,
    ) {
        let timestamp_ms = 1_765_300_000_000;
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let thread_id = "thread-s3-chain";
        let dir = test_temp_dir(name);
        let path = dir.join("workflow-state.v0.json");
        let index_path = dir.join("codex-index.json");
        let index = fixture_dispatch_index(test_root, thread_id);
        bootstrap_project_workflow_at(&path, &fixture_project(test_root)).expect("workflow");
        let (proposal, authorization, revision) =
            create_active_project_director_authorization_fixture(
                &path,
                test_root,
                thread_id,
                timestamp_ms,
            );
        let workflow_id = default_workflow_id(test_root);
        bind_workflow_node_codex_session_for_index_at(
            &path,
            &index,
            &fixture_node_session_bind_request(
                test_root,
                &format!("{workflow_id}:node:codex-dev"),
                None,
                thread_id,
            ),
        )
        .expect("bind");
        let ctx = load_project_context(test_root).expect("ctx");
        let planned = StubDirector.plan(&ctx, &proposal).expect("plan");
        let prepared = prepare_authorized_auto_dispatch_for_index_at(
            &path,
            &index,
            &fixture_project_director_prepare_input(
                test_root,
                &proposal.proposal_id,
                &authorization.authorization_id,
                revision,
                planned,
            ),
        )
        .expect("prepare");
        (dir, path, index, index_path, workflow_id, prepared)
    }

    #[test]
    fn s3_director_chain_runs_all_prepared_tasks_topo() {
        let (dir, path, index, index_path, workflow_id, prepared) =
            s3_director_prepared_chain("s3-director-chain-ok");
        // F4·按**授权状态**数（status=="prepared"），不是「有 work_item」——annotate 给所有任务都设了
        // work_item，旧口径恒为全量、量不出授权。
        let prepared_count = prepared
            .plan
            .planned_tasks
            .iter()
            .filter(|t| t.status == "prepared")
            .count();
        assert!(
            prepared_count >= 2,
            "多任务链：至少 2 个任务被 prepare 授权"
        );
        let runner = PermissiveExperimentRunner {
            stats: CodexDispatchReadbackStats {
                transcript_event_count: 3,
                transcript_target_hits: 1,
            },
        };
        let outcome = run_director_task_chain(
            &path,
            &index,
            &index_path,
            &runner,
            WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
            &workflow_id,
            &prepared.plan.planned_tasks,
            10,
        )
        .expect("chain");
        assert!(
            outcome.stopped_reason.is_none(),
            "应按拓扑序全跑完无失败：{:?}",
            outcome.stopped_reason
        );
        assert_eq!(
            outcome.total,
            prepared.plan.planned_tasks.len(),
            "total = 计划任务总数（含非 prepared）"
        );
        assert!(!outcome.chain_run_id.is_empty(), "应建出链运行记录 id");
        assert_eq!(
            outcome.completed, prepared_count,
            "所有 prepared 任务按序跑完"
        );
        assert_eq!(outcome.dispatched, prepared_count, "真派发数 = prepared 数");
        assert_eq!(outcome.skipped, 0, "全 prepared → 0 跳过");
        assert!(outcome.warnings.is_empty(), "依赖自洽 → 无 dangling 警告");
        // F4·依赖序断言：「搭骨架」必须先于「接业务」真跑（depends_on 拓扑序生效）。
        let pos = |title: &str| {
            outcome
                .steps
                .iter()
                .position(|s| s.title == title)
                .unwrap_or_else(|| panic!("steps 应含「{title}」：{:?}", outcome.steps))
        };
        assert!(
            pos("搭骨架") < pos("接业务"),
            "依赖序：搭骨架 应先于 接业务，实际 steps={:?}",
            outcome.steps
        );
        assert!(
            outcome.steps.iter().all(|s| !s.planned_task_id.is_empty()),
            "每步应带 planned_task_id（链记录按它编址）"
        );
        // 链记录收尾为 completed + 审计在。
        assert!(
            audit_has(&path, "workflow_chain_run_completed"),
            "应有链完成审计"
        );
        let _ = fs::remove_dir_all(dir);
    }

    struct C4aReportRunner {
        message: String,
        stats: CodexDispatchReadbackStats,
    }

    impl CodexResumeRunner for C4aReportRunner {
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
            fs::write(last_message_path, &self.message)
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

    fn c4a_worker_report(status: &str, evidence: &[&str], direction_risks: &[&str]) -> String {
        serde_json::json!({
            "did": format!("C4a fixture worker status {status}"),
            "outputs": ["/tmp/c4a-output.txt"],
            "status": status,
            "evidence": evidence,
            "direction_risks": direction_risks,
        })
        .to_string()
    }

    fn c4a_report_runner(
        status: &str,
        evidence: &[&str],
        direction_risks: &[&str],
    ) -> C4aReportRunner {
        C4aReportRunner {
            message: format!(
                "```json\n{}\n```",
                c4a_worker_report(status, evidence, direction_risks)
            ),
            stats: CodexDispatchReadbackStats {
                transcript_event_count: 3,
                transcript_target_hits: 1,
            },
        }
    }

    fn c4a_single_prepared_task(
        prepared: &AuthorizedPreparedDispatchResult,
    ) -> Vec<ProjectDirectorPlannedTask> {
        let mut task = prepared
            .plan
            .planned_tasks
            .iter()
            .find(|task| task.status == "prepared")
            .expect("fixture should have a prepared task")
            .clone();
        task.depends_on.clear();
        task.scope.required_checks.clear();
        vec![task]
    }

    fn c4b_prepared_tasks_without_required_checks(
        prepared: &AuthorizedPreparedDispatchResult,
    ) -> Vec<ProjectDirectorPlannedTask> {
        let mut tasks = prepared
            .plan
            .planned_tasks
            .iter()
            .filter(|task| task.status == "prepared")
            .cloned()
            .collect::<Vec<_>>();
        assert!(
            tasks.len() >= 2,
            "C4b once-per-chain test needs multiple tasks"
        );
        for task in tasks.iter_mut() {
            task.scope.required_checks.clear();
        }
        tasks
    }

    fn c4c_failed_task_fixture(
        name: &str,
        index: Option<Value>,
    ) -> (
        PathBuf,
        PathBuf,
        Value,
        PathBuf,
        String,
        ProjectDirectorPlannedTask,
        String,
    ) {
        let (dir, path, default_index, index_path, workflow_id, prepared) =
            s3_director_prepared_chain(name);
        let index = index.unwrap_or(default_index);
        let tasks = c4a_single_prepared_task(&prepared);
        let failing = FailingCodexResumeRunner {
            exit_code: 1,
            timed_out: false,
        };
        let failed = run_director_task_chain(
            &path,
            &index,
            &index_path,
            &failing,
            WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
            &workflow_id,
            &tasks,
            1,
        )
        .expect("fixture chain should fail-stop");
        assert_eq!(failed.steps[0].state, "failed");
        (
            dir,
            path,
            index,
            index_path,
            workflow_id,
            tasks[0].clone(),
            failed.chain_run_id,
        )
    }

    fn c4c_failed_action_request(
        action: &str,
        workflow_id: &str,
        chain_run_id: &str,
        task: &ProjectDirectorPlannedTask,
    ) -> ProjectDirectorFailedActionRequest {
        ProjectDirectorFailedActionRequest {
            project_root: WORKFLOW_ENGINE_TEST_PROJECT_ROOT.to_string(),
            workflow_id: workflow_id.to_string(),
            chain_run_id: chain_run_id.to_string(),
            planned_task_id: task.planned_task_id.clone(),
            action: action.to_string(),
            actor_role: "project_director".to_string(),
            actor_id: Some("project_director_fixture".to_string()),
            explicit_retry_or_reopen: action == "retry" || action == "change_session",
            planned_task: Some(task.clone()),
            max_nodes: Some(1),
        }
    }

    fn c4c_needs_rework_task_fixture(
        name: &str,
        index: Option<Value>,
    ) -> (
        PathBuf,
        PathBuf,
        Value,
        PathBuf,
        String,
        ProjectDirectorPlannedTask,
        String,
    ) {
        let (dir, path, index, index_path, workflow_id, task, chain_run_id) =
            c4c_failed_task_fixture(name, index);
        let runner = c4a_report_runner("done", &["unused"], &[]);
        let request = c4c_failed_action_request("rework", &workflow_id, &chain_run_id, &task);
        let outcome =
            run_project_director_failed_action(&path, &index, &index_path, &runner, &request)
                .expect("fixture should enter needs_rework");
        assert_eq!(outcome.node_state, "needs_rework");
        assert_eq!(
            chain_node_state(&read_json_file(&path), &chain_run_id, &task.planned_task_id)
                .as_deref(),
            Some("needs_rework")
        );
        (
            dir,
            path,
            index,
            index_path,
            workflow_id,
            task,
            chain_run_id,
        )
    }

    #[derive(Clone)]
    struct StubDirectorFinalMarker {
        calls: std::cell::Cell<usize>,
        result: Result<DirectorFinalMark, String>,
    }

    impl StubDirectorFinalMarker {
        fn completed(reason: &str) -> Self {
            Self {
                calls: std::cell::Cell::new(0),
                result: Ok(DirectorFinalMark {
                    decision: DirectorFinalMarkDecision::Completed,
                    reason: reason.to_string(),
                }),
            }
        }

        fn needs_rework(reason: &str) -> Self {
            Self {
                calls: std::cell::Cell::new(0),
                result: Ok(DirectorFinalMark {
                    decision: DirectorFinalMarkDecision::NeedsRework,
                    reason: reason.to_string(),
                }),
            }
        }

        fn unavailable(reason: &str) -> Self {
            Self {
                calls: std::cell::Cell::new(0),
                result: Err(reason.to_string()),
            }
        }
    }

    impl DirectorFinalMarker for StubDirectorFinalMarker {
        fn final_mark(&self, _ctx: &DirectorFinalMarkContext) -> Result<DirectorFinalMark, String> {
            self.calls.set(self.calls.get() + 1);
            self.result.clone()
        }
    }

    #[derive(Clone)]
    struct StubDirectorSummaryGenerator {
        calls: std::cell::Cell<usize>,
        result: Result<DirectorWorkflowSummary, String>,
    }

    impl StubDirectorSummaryGenerator {
        fn summarized(summary: &str) -> Self {
            Self {
                calls: std::cell::Cell::new(0),
                result: Ok(DirectorWorkflowSummary {
                    summary: summary.to_string(),
                    key_facts: vec!["关键事实：链路已按任务完成".to_string()],
                    open_items: vec!["未决项：无".to_string()],
                    next_suggestions: vec!["后续建议：进入人工确认记忆候选".to_string()],
                }),
            }
        }

        fn unavailable(reason: &str) -> Self {
            Self {
                calls: std::cell::Cell::new(0),
                result: Err(reason.to_string()),
            }
        }
    }

    impl DirectorSummaryGenerator for StubDirectorSummaryGenerator {
        fn summarize_chain(
            &self,
            _ctx: &DirectorWorkflowSummaryContext,
        ) -> Result<DirectorWorkflowSummary, String> {
            self.calls.set(self.calls.get() + 1);
            self.result.clone()
        }
    }

    #[test]
    fn c4a_director_final_mark_green_report_completes_without_lm() {
        let (dir, path, index, index_path, workflow_id, prepared) =
            s3_director_prepared_chain("c4a-final-green-zero-lm");
        let tasks = c4a_single_prepared_task(&prepared);
        let runner = c4a_report_runner("done", &["cargo test c4a 通过"], &[]);
        let marker = StubDirectorFinalMarker::unavailable("full green must not call LM");
        let outcome = run_director_task_chain_with_final_marker(
            &path,
            &index,
            &index_path,
            &runner,
            WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
            &workflow_id,
            &tasks,
            10,
            &marker,
        )
        .expect("full green should complete");
        assert_eq!(marker.calls.get(), 0, "全绿终标不得调用 LM");
        assert_eq!(outcome.completed, 1);
        assert!(outcome.stopped_reason.is_none());
        assert_eq!(outcome.steps[0].state, "completed");
        assert!(
            audit_has(
                &path,
                "workflow_chain_node_director_deterministic_completed"
            ),
            "应记录主管终标·确定性直过审计"
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn c4a_director_final_mark_yellow_report_uses_lm_pass() {
        let (dir, path, index, index_path, workflow_id, prepared) =
            s3_director_prepared_chain("c4a-final-yellow-lm-pass");
        let tasks = c4a_single_prepared_task(&prepared);
        let runner = c4a_report_runner("partial", &["证据有但 status 非 done"], &[]);
        let marker = StubDirectorFinalMarker::completed("主管判定黄牌仍可接受");
        let outcome = run_director_task_chain_with_final_marker(
            &path,
            &index,
            &index_path,
            &runner,
            WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
            &workflow_id,
            &tasks,
            10,
            &marker,
        )
        .expect("LM pass should complete");
        assert_eq!(marker.calls.get(), 1, "黄牌必须调用主管 LM");
        assert_eq!(outcome.completed, 1);
        assert!(outcome.stopped_reason.is_none());
        assert_eq!(outcome.steps[0].state, "completed");
        assert!(
            audit_has(&path, "workflow_chain_node_director_lm_completed"),
            "应记录主管 LM 终标通过审计"
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn c4a_director_final_mark_yellow_report_rework_spends_budget_and_stops() {
        let (dir, path, index, index_path, workflow_id, prepared) =
            s3_director_prepared_chain("c4a-final-yellow-rework");
        let tasks = c4a_single_prepared_task(&prepared);
        let runner = c4a_report_runner("partial", &["证据不足以直过"], &[]);
        let marker = StubDirectorFinalMarker::needs_rework("需要补证据");
        let outcome = run_director_task_chain_with_final_marker(
            &path,
            &index,
            &index_path,
            &runner,
            WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
            &workflow_id,
            &tasks,
            10,
            &marker,
        )
        .expect("LM rework should stop for redo");
        assert_eq!(marker.calls.get(), 1);
        assert_eq!(outcome.completed, 0);
        assert_eq!(outcome.steps[0].state, "needs_rework");
        assert!(
            outcome
                .stopped_reason
                .as_deref()
                .unwrap_or("")
                .contains("needs_rework"),
            "退回应停链待重做：{:?}",
            outcome.stopped_reason
        );
        assert!(audit_has(&path, "workflow_chain_node_needs_rework"));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn c4a_director_final_mark_rework_budget_exhausted_waits_for_human() {
        let (dir, path, index, index_path, workflow_id, prepared) =
            s3_director_prepared_chain("c4a-final-rework-budget");
        let tasks = c4a_single_prepared_task(&prepared);
        let runner = c4a_report_runner("partial", &["仍然证据不足"], &[]);
        let marker = StubDirectorFinalMarker::needs_rework("仍需返工");
        let first = run_director_task_chain_with_final_marker(
            &path,
            &index,
            &index_path,
            &runner,
            WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
            &workflow_id,
            &tasks,
            10,
            &marker,
        )
        .expect("first rework should spend budget");
        assert_eq!(first.steps[0].state, "needs_rework");
        let second = run_director_task_chain_with_final_marker(
            &path,
            &index,
            &index_path,
            &runner,
            WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
            &workflow_id,
            &tasks,
            10,
            &marker,
        )
        .expect("second rework should stop for human");
        assert_eq!(marker.calls.get(), 2);
        assert_eq!(second.completed, 0);
        assert_eq!(second.steps[0].state, "waiting_decision");
        assert!(
            second
                .stopped_reason
                .as_deref()
                .unwrap_or("")
                .contains("budget_exhausted"),
            "预算耗尽应待人决策：{:?}",
            second.stopped_reason
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn c4a_director_final_mark_lm_unavailable_does_not_complete() {
        let (dir, path, index, index_path, workflow_id, prepared) =
            s3_director_prepared_chain("c4a-final-lm-unavailable");
        let tasks = c4a_single_prepared_task(&prepared);
        let runner = c4a_report_runner("partial", &["证据不足"], &[]);
        let marker = StubDirectorFinalMarker::unavailable("codex_provider_unavailable:quota");
        let outcome = run_director_task_chain_with_final_marker(
            &path,
            &index,
            &index_path,
            &runner,
            WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
            &workflow_id,
            &tasks,
            10,
            &marker,
        )
        .expect("LM unavailable should soft stop");
        assert_eq!(marker.calls.get(), 1);
        assert_eq!(outcome.completed, 0);
        assert_eq!(outcome.steps[0].state, "waiting_decision");
        assert!(
            outcome
                .stopped_reason
                .as_deref()
                .unwrap_or("")
                .contains("final_mark_unavailable"),
            "LM 断供应待人、不 completed：{:?}",
            outcome.stopped_reason
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn c4a_director_final_mark_preserves_worker_help_waiting_decision() {
        let (dir, path, index, index_path, workflow_id, prepared) =
            s3_director_prepared_chain("c4a-final-help-route");
        let tasks = c4a_single_prepared_task(&prepared);
        let runner = c4a_report_runner("blocked", &["缺权限"], &["方向可能错"]);
        let marker = StubDirectorFinalMarker::completed("help route must not call final marker");
        let outcome = run_director_task_chain_with_final_marker(
            &path,
            &index,
            &index_path,
            &runner,
            WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
            &workflow_id,
            &tasks,
            10,
            &marker,
        )
        .expect("help route should stop");
        assert_eq!(marker.calls.get(), 0, "求助路不应进入 C4a 终标");
        assert_eq!(outcome.completed, 0);
        assert_eq!(outcome.steps[0].state, "waiting_decision");
        assert!(
            outcome
                .stopped_reason
                .as_deref()
                .unwrap_or("")
                .contains("worker_help"),
            "求助路停因应保持 C3 waiting_decision：{:?}",
            outcome.stopped_reason
        );
        let retry_runner = c4a_report_runner("done", &["补齐权限后完成"], &[]);
        let request =
            c4c_failed_action_request("retry", &workflow_id, &outcome.chain_run_id, &tasks[0]);
        let retried =
            run_project_director_failed_action(&path, &index, &index_path, &retry_runner, &request)
                .expect("waiting_decision 应接受主管显式 retry 并走合法 running 转移");
        assert_eq!(retried.transition_to, "running");
        assert_eq!(
            retried.chain_outcome.expect("retry should run").completed,
            1
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn waiting_decision_archive_cancels_node_and_archives_chain_without_delivery() {
        let (dir, path, index, index_path, workflow_id, prepared) =
            s3_director_prepared_chain("waiting-decision-archive");
        let tasks = c4a_single_prepared_task(&prepared);
        let help_runner = c4a_report_runner("blocked", &["缺权限"], &["方向可能错"]);
        let waiting = run_director_task_chain_with_final_marker(
            &path,
            &index,
            &index_path,
            &help_runner,
            WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
            &workflow_id,
            &tasks,
            10,
            &StubDirectorFinalMarker::completed("help must not final mark"),
        )
        .expect("worker help should wait");
        assert_eq!(waiting.steps[0].state, "waiting_decision");

        let request =
            c4c_failed_action_request("archive", &workflow_id, &waiting.chain_run_id, &tasks[0]);
        let archived = run_project_director_failed_action(
            &path,
            &index,
            &index_path,
            &c4a_report_runner("done", &["unused"], &[]),
            &request,
        )
        .expect("waiting_decision 结束应走合法 cancelled / archived 转移");
        assert_eq!(archived.transition_to, "cancelled");
        assert_eq!(archived.node_state, "cancelled");
        assert_eq!(archived.chain_state, "archived");
        assert_eq!(
            archived.stopped_reason.as_deref(),
            Some("archived:waiting_decision_action")
        );
        assert!(
            archived.chain_outcome.is_none(),
            "结束这单不能伪造交货链结果"
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn c4c_failed_action_retry_requires_director_and_explicit_reopen() {
        let (dir, path, index, index_path, workflow_id, task, chain_run_id) =
            c4c_failed_task_fixture("c4c-failed-retry", None);
        let runner = c4a_report_runner("done", &["retry evidence"], &[]);
        let mut request = c4c_failed_action_request("retry", &workflow_id, &chain_run_id, &task);
        request.actor_role = "subagent".to_string();
        let err = run_project_director_failed_action(&path, &index, &index_path, &runner, &request)
            .expect_err("非 project_director 不得 reopen failed 节点");
        assert!(err.contains("project_director"), "错误应点名主管门：{err}");

        request.actor_role = "project_director".to_string();
        request.explicit_retry_or_reopen = false;
        let err = run_project_director_failed_action(&path, &index, &index_path, &runner, &request)
            .expect_err("failed->running 必须 explicit retry/reopen");
        assert!(err.contains("explicit"), "错误应点名显式重试：{err}");

        request.explicit_retry_or_reopen = true;
        let outcome =
            run_project_director_failed_action(&path, &index, &index_path, &runner, &request)
                .expect("主管显式 retry 应复用现有链驱动重跑单任务");
        assert_eq!(outcome.transition_to, "running");
        assert_eq!(outcome.action, "retry");
        let chain = outcome.chain_outcome.expect("retry 应触发既有链驱动");
        assert_eq!(chain.completed, 1);
        assert!(audit_has(&path, "workflow_chain_node_failed_action_retry"));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn needs_rework_retry_keeps_already_reset_work_item_idempotent_then_runs() {
        let (dir, path, index, index_path, workflow_id, task, chain_run_id) =
            c4c_needs_rework_task_fixture("needs-rework-retry-idempotent", None);
        let work_item_id = task.work_item_id.as_deref().expect("prepared work item");
        let before = find_work_item(&read_json_file(&path), &workflow_id, work_item_id)
            .cloned()
            .expect("work item before retry");
        assert_eq!(
            optional_string_from(&before, "state").as_deref(),
            Some("ready_to_dispatch"),
            "主管退回夹具应已复位"
        );
        assert!(
            reset_work_item_for_retry(&path, WORKFLOW_ENGINE_TEST_PROJECT_ROOT, work_item_id),
            "已复位的任务应幂等通过"
        );
        assert_eq!(
            find_work_item(&read_json_file(&path), &workflow_id, work_item_id),
            Some(&before),
            "幂等复位不应重复写 work item"
        );

        let runner = c4a_report_runner("done", &["retry evidence"], &[]);
        let request = c4c_failed_action_request("retry", &workflow_id, &chain_run_id, &task);
        let outcome =
            run_project_director_failed_action(&path, &index, &index_path, &runner, &request)
                .expect("needs_rework retry should reuse the original session path");
        assert_eq!(outcome.action, "retry");
        assert_eq!(
            outcome.chain_outcome.expect("retry should run").completed,
            1,
            "retry should still hand off to the existing chain driver"
        );
        assert!(audit_has(&path, "workflow_chain_node_failed_action_retry"));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn needs_rework_change_session_reuses_c1_then_runs() {
        struct OneShotCreator {
            calls: Cell<usize>,
        }
        impl JiaobanNewSessionCreator for OneShotCreator {
            fn create_initialized_session(&self, text: &str, _by: &str) -> Result<String, String> {
                self.calls.set(self.calls.get() + 1);
                assert!(text.contains("交办任务专用会话"));
                Ok("thread-needs-rework-new".to_string())
            }
        }

        let index = fixture_multi_thread_index(
            WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
            &["thread-s3-chain", "thread-needs-rework-new"],
        );
        let (dir, path, index, index_path, workflow_id, task, chain_run_id) =
            c4c_needs_rework_task_fixture("needs-rework-change-session", Some(index));
        let creator = OneShotCreator {
            calls: Cell::new(0),
        };
        let runner = c4a_report_runner("done", &["new session evidence"], &[]);
        let request =
            c4c_failed_action_request("change_session", &workflow_id, &chain_run_id, &task);
        let outcome = run_project_director_failed_action_with_session_creator(
            &path,
            &index,
            &index_path,
            &runner,
            &request,
            &creator,
        )
        .expect("needs_rework change_session should use C1 then run");
        assert_eq!(creator.calls.get(), 1, "只给退回任务新建 1 条会话");
        assert_eq!(
            outcome.new_session_id.as_deref(),
            Some("thread-needs-rework-new")
        );
        assert_eq!(outcome.chain_outcome.expect("should run").completed, 1);
        assert!(audit_has(
            &path,
            "workflow_chain_node_failed_action_change_session"
        ));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn needs_rework_rework_keeps_the_existing_rework_decision_without_spending_again() {
        let (dir, path, index, index_path, workflow_id, task, chain_run_id) =
            c4c_needs_rework_task_fixture("needs-rework-rework", None);
        let runner = c4a_report_runner("done", &["unused"], &[]);
        let request = c4c_failed_action_request("rework", &workflow_id, &chain_run_id, &task);
        let outcome =
            run_project_director_failed_action(&path, &index, &index_path, &runner, &request)
                .expect("needs_rework rework should remain a user-directed no-op");
        assert_eq!(outcome.node_state, "needs_rework");
        let value = read_json_file(&path);
        assert_eq!(
            chain_node_usize_field(
                &value,
                &chain_run_id,
                &task.planned_task_id,
                "director_rework_attempts"
            ),
            DIRECTOR_FINAL_REWORK_BUDGET,
            "already spent final-mark budget must not be charged again"
        );
        assert!(audit_has(&path, "workflow_chain_node_failed_action_rework"));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn needs_rework_archive_uses_existing_archived_transition() {
        let (dir, path, index, index_path, workflow_id, task, chain_run_id) =
            c4c_needs_rework_task_fixture("needs-rework-archive", None);
        let runner = c4a_report_runner("done", &["unused"], &[]);
        let request = c4c_failed_action_request("archive", &workflow_id, &chain_run_id, &task);
        let outcome =
            run_project_director_failed_action(&path, &index, &index_path, &runner, &request)
                .expect("needs_rework archive should reuse the archived transition");
        assert_eq!(outcome.transition_to, "archived");
        let value = read_json_file(&path);
        assert_eq!(
            chain_node_state(&value, &chain_run_id, &task.planned_task_id).as_deref(),
            Some("archived")
        );
        assert!(audit_has(
            &path,
            "workflow_chain_node_failed_action_archive"
        ));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn failed_action_rejects_states_other_than_failed_needs_rework_or_waiting_decision() {
        let (dir, path, index, index_path, workflow_id, task, chain_run_id) =
            c4c_failed_task_fixture("failed-action-other-state", None);
        let mut value = read_json_file(&path);
        set_chain_node_state(
            &mut value,
            &chain_run_id,
            &task.planned_task_id,
            "completed",
            None,
            None,
        );
        write_validated_workflow_state(&path, &value).expect("fixture state write");
        let runner = c4a_report_runner("done", &["unused"], &[]);
        let request = c4c_failed_action_request("archive", &workflow_id, &chain_run_id, &task);
        let err = run_project_director_failed_action(&path, &index, &index_path, &runner, &request)
            .expect_err("completed node must remain outside the four-action surface");
        assert!(
            err.contains("failed / needs_rework / waiting_decision"),
            "error should name the exact accepted states: {err}"
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn c4c_failed_action_rework_reuses_c4a_budget_and_reset() {
        let (dir, path, index, index_path, workflow_id, task, chain_run_id) =
            c4c_failed_task_fixture("c4c-failed-rework", None);
        let runner = c4a_report_runner("done", &["unused"], &[]);
        let request = c4c_failed_action_request("rework", &workflow_id, &chain_run_id, &task);
        let outcome =
            run_project_director_failed_action(&path, &index, &index_path, &runner, &request)
                .expect("failed rework should reuse C4a reset + budget");
        assert_eq!(outcome.transition_to, "needs_rework");
        assert!(outcome.chain_outcome.is_none(), "rework 不应真跑 codex");
        let value = read_json_file(&path);
        assert_eq!(
            chain_node_state(&value, &chain_run_id, &task.planned_task_id).as_deref(),
            Some("needs_rework")
        );
        assert_eq!(
            chain_node_usize_field(
                &value,
                &chain_run_id,
                &task.planned_task_id,
                "director_rework_attempts"
            ),
            1
        );
        let work_item_id = task.work_item_id.as_deref().expect("prepared work item");
        let work_item_state = value["work_items"]
            .as_array()
            .and_then(|items| {
                items.iter().find(|item| {
                    optional_string_from(item, "work_item_id").as_deref() == Some(work_item_id)
                })
            })
            .and_then(|item| optional_string_from(item, "state"));
        assert_eq!(
            work_item_state.as_deref(),
            Some("ready_to_dispatch"),
            "rework 应复用 C4a reset，把 work_item 复位到可重做"
        );
        assert!(audit_has(&path, "workflow_chain_node_failed_action_rework"));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn c4c_failed_action_rework_budget_exhausted_does_not_loop() {
        let (dir, path, index, index_path, workflow_id, task, chain_run_id) =
            c4c_failed_task_fixture("c4c-failed-rework-budget", None);
        let mut value = read_json_file(&path);
        set_chain_node_usize_field(
            &mut value,
            &chain_run_id,
            &task.planned_task_id,
            "director_rework_attempts",
            DIRECTOR_FINAL_REWORK_BUDGET,
        );
        write_validated_workflow_state(&path, &value).expect("fixture budget write");
        let runner = c4a_report_runner("done", &["unused"], &[]);
        let request = c4c_failed_action_request("rework", &workflow_id, &chain_run_id, &task);
        let err = run_project_director_failed_action(&path, &index, &index_path, &runner, &request)
            .expect_err("预算耗尽不得继续退回/重跑");
        assert!(
            err.contains("director_rework_budget_exhausted"),
            "错误应点明预算耗尽：{err}"
        );
        let value = read_json_file(&path);
        assert_eq!(
            chain_node_state(&value, &chain_run_id, &task.planned_task_id).as_deref(),
            Some("failed"),
            "预算耗尽不应把 failed 静默改成 needs_rework"
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn c4c_failed_action_change_session_reuses_c1_create_bind_then_runs_single_task() {
        struct OneShotCreator {
            calls: Cell<usize>,
        }
        impl JiaobanNewSessionCreator for OneShotCreator {
            fn create_initialized_session(&self, text: &str, _by: &str) -> Result<String, String> {
                self.calls.set(self.calls.get() + 1);
                assert!(text.contains("交办任务专用会话"));
                Ok("thread-c4c-new".to_string())
            }
        }
        struct RecordingRunner {
            threads: RefCell<Vec<String>>,
        }
        impl CodexResumeRunner for RecordingRunner {
            fn resume_with_options(
                &self,
                thread_id: &str,
                _prompt: &str,
                last_message_path: &Path,
                _options: &CodexResumeRequestOptions,
            ) -> Result<(CodexResumeRunResult, WorkflowNodeDispatchExecutionOptions), String>
            {
                self.threads.borrow_mut().push(thread_id.to_string());
                if let Some(parent) = last_message_path.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|error| format!("fixture output dir create failed: {error}"))?;
                }
                fs::write(
                    last_message_path,
                    format!(
                        "```json\n{}\n```",
                        c4a_worker_report("done", &["change session evidence"], &[])
                    ),
                )
                .map_err(|error| format!("fixture last message write failed: {error}"))?;
                Ok((
                    CodexResumeRunResult {
                        exit_code: 0,
                        timed_out: false,
                        stderr_summary: None,
                    },
                    WorkflowNodeDispatchExecutionOptions {
                        readback_stats: Some(CodexDispatchReadbackStats {
                            transcript_event_count: 3,
                            transcript_target_hits: 1,
                        }),
                    },
                ))
            }
        }

        let index = fixture_multi_thread_index(
            WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
            &["thread-s3-chain", "thread-c4c-new"],
        );
        let (dir, path, index, index_path, workflow_id, task, chain_run_id) =
            c4c_failed_task_fixture("c4c-failed-change-session", Some(index));
        let creator = OneShotCreator {
            calls: Cell::new(0),
        };
        let runner = RecordingRunner {
            threads: RefCell::new(vec![]),
        };
        let request =
            c4c_failed_action_request("change_session", &workflow_id, &chain_run_id, &task);
        let outcome = run_project_director_failed_action_with_session_creator(
            &path,
            &index,
            &index_path,
            &runner,
            &request,
            &creator,
        )
        .expect("change_session 应复用 C1 create_and_bind 后重跑单任务");
        assert_eq!(creator.calls.get(), 1, "只给失败任务建 1 条新会话");
        assert_eq!(outcome.transition_to, "running");
        assert_eq!(outcome.new_session_id.as_deref(), Some("thread-c4c-new"));
        assert_eq!(
            runner.threads.borrow().as_slice(),
            &["thread-c4c-new".to_string()],
            "重跑应使用刚绑定的新会话"
        );
        assert_eq!(
            outcome
                .chain_outcome
                .expect("change_session 应真跑")
                .completed,
            1
        );
        assert!(audit_has(
            &path,
            "workflow_chain_node_failed_action_change_session"
        ));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn c4c_failed_action_archive_uses_existing_failed_to_archived_transition() {
        let (dir, path, index, index_path, workflow_id, task, chain_run_id) =
            c4c_failed_task_fixture("c4c-failed-archive", None);
        let runner = c4a_report_runner("done", &["unused"], &[]);
        let request = c4c_failed_action_request("archive", &workflow_id, &chain_run_id, &task);
        let outcome =
            run_project_director_failed_action(&path, &index, &index_path, &runner, &request)
                .expect("archive 应复用 failed->archived 转移");
        assert_eq!(outcome.transition_to, "archived");
        assert!(outcome.chain_outcome.is_none(), "archive 不应真跑 codex");
        let value = read_json_file(&path);
        let run = chain_run_record(&value, &chain_run_id).expect("chain run");
        assert_eq!(
            optional_string_from(run, "state").as_deref(),
            Some("archived")
        );
        assert_eq!(
            chain_node_state(&value, &chain_run_id, &task.planned_task_id).as_deref(),
            Some("archived")
        );
        assert!(audit_has(
            &path,
            "workflow_chain_node_failed_action_archive"
        ));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn c4b_director_summary_creates_candidate_once_after_chain_completed() {
        let (dir, path, index, index_path, workflow_id, prepared) =
            s3_director_prepared_chain("c4b-summary-candidate");
        let tasks = c4b_prepared_tasks_without_required_checks(&prepared);
        let runner = c4a_report_runner("done", &["cargo test c4b 通过"], &[]);
        let final_marker = StubDirectorFinalMarker::unavailable("green should not call final LM");
        let summary_generator =
            StubDirectorSummaryGenerator::summarized("主管总结：本链完成了 C4b 验证任务。");
        let outcome = run_director_task_chain_with_markers(
            &path,
            &index,
            &index_path,
            &runner,
            WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
            &workflow_id,
            &tasks,
            10,
            &final_marker,
            &summary_generator,
        )
        .expect("chain should complete with director summary");
        assert_eq!(final_marker.calls.get(), 0, "全绿终标仍不得调用 LM");
        assert_eq!(summary_generator.calls.get(), 1, "链末总结只允许每链一次");
        assert_eq!(outcome.completed, tasks.len());
        assert!(outcome.stopped_reason.is_none());
        assert_eq!(
            outcome
                .director_summary
                .as_ref()
                .map(|summary| summary.summary.as_str()),
            Some("主管总结：本链完成了 C4b 验证任务。")
        );
        assert!(audit_has(&path, "workflow_chain_director_summary"));
        let candidate_store = memory_candidate_store::load_store(&path, "2026-07-09T00:00:00Z")
            .expect("candidate store should load");
        assert_eq!(candidate_store.candidates.len(), 1);
        let candidate = &candidate_store.candidates[0];
        assert_eq!(
            memory_candidate_store::memory_status_name(candidate.status),
            "candidate_needs_review",
            "capture_event 只能生成候选态，不能自动转正"
        );
        assert!(
            candidate.generated_from.starts_with("observation:"),
            "候选应从 observation 派生：{}",
            candidate.generated_from
        );
        assert_eq!(candidate.generated_by_role, "project_director");
        assert!(
            candidate
                .source_refs
                .iter()
                .any(|source| source.source_type == "director_review"),
            "候选来源应经 final_review 映射到 director_review"
        );
        assert!(
            formal_memory_store::load_store(&path, "2026-07-09T00:00:00Z")
                .expect("formal memory store should load")
                .records
                .is_empty()
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn c4b_director_summary_unavailable_keeps_chain_completed_with_warning() {
        let (dir, path, index, index_path, workflow_id, prepared) =
            s3_director_prepared_chain("c4b-summary-unavailable");
        let tasks = c4a_single_prepared_task(&prepared);
        let runner = c4a_report_runner("done", &["cargo test c4b 通过"], &[]);
        let final_marker = StubDirectorFinalMarker::unavailable("green should not call final LM");
        let summary_generator =
            StubDirectorSummaryGenerator::unavailable("codex_provider_unavailable:summary");
        let outcome = run_director_task_chain_with_markers(
            &path,
            &index,
            &index_path,
            &runner,
            WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
            &workflow_id,
            &tasks,
            10,
            &final_marker,
            &summary_generator,
        )
        .expect("summary failure should soft land");
        assert_eq!(summary_generator.calls.get(), 1);
        assert_eq!(outcome.completed, 1);
        assert!(outcome.stopped_reason.is_none());
        assert!(outcome.director_summary.is_none());
        assert!(
            outcome
                .warnings
                .iter()
                .any(|warning| warning.contains("director_summary_unavailable")),
            "总结失败应进 warning，不阻断链：{:?}",
            outcome.warnings
        );
        assert!(
            memory_candidate_store::load_store(&path, "2026-07-09T00:00:00Z")
                .expect("candidate store should load")
                .candidates
                .is_empty()
        );
        let _ = fs::remove_dir_all(dir);
    }

    // B1·只派 status=="prepared"：把一个任务手动置 blocked → 断言它被跳过（不进 execute）、其余照跑。
    #[test]
    fn s3_director_chain_skips_non_prepared_tasks() {
        let (dir, path, index, index_path, workflow_id, prepared) =
            s3_director_prepared_chain("s3-director-chain-skip");
        let mut tasks = prepared.plan.planned_tasks.clone();
        assert!(tasks.len() >= 2, "需要多任务来验证跳过");
        // 把依赖任务「接业务」置 blocked（它带 work_item+node，但未授权派发）。
        let blocked_title = "接业务".to_string();
        for task in tasks.iter_mut() {
            if task.title == blocked_title {
                task.status = "blocked".to_string();
                task.blocked_reasons = vec!["测试：人为置 blocked".to_string()];
            }
        }
        let runner = PermissiveExperimentRunner {
            stats: CodexDispatchReadbackStats {
                transcript_event_count: 3,
                transcript_target_hits: 1,
            },
        };
        let outcome = run_director_task_chain(
            &path,
            &index,
            &index_path,
            &runner,
            WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
            &workflow_id,
            &tasks,
            10,
        )
        .expect("chain");
        assert!(outcome.stopped_reason.is_none(), "跳过 blocked 不致停链");
        assert_eq!(outcome.skipped, 1, "1 个 blocked 任务被跳过");
        assert_eq!(outcome.completed, 1, "只剩 1 个 prepared 任务真跑");
        let blocked_step = outcome
            .steps
            .iter()
            .find(|s| s.title == blocked_title)
            .expect("steps 应记录被跳过的任务");
        assert_eq!(
            blocked_step.state, "skipped",
            "blocked 任务记为 skipped（未 execute）"
        );
        let _ = fs::remove_dir_all(dir);
    }

    // B2·可中断：runner 跑完第一个任务后，**调用现成 stop_project_workflow_chain_at**（证停命令 0-diff
    // 就能找到本驱动建的 running 链记录）→ 下个任务边界停。
    #[test]
    fn s3_director_chain_interrupts_at_task_boundary_on_stop() {
        let (dir, path, index, index_path, workflow_id, prepared) =
            s3_director_prepared_chain("s3-director-chain-stop");
        struct StopViaCommandRunner {
            state_path: PathBuf,
            project_root: String,
            workflow_id: String,
            stats: CodexDispatchReadbackStats,
        }
        impl CodexResumeRunner for StopViaCommandRunner {
            fn resume_with_options(
                &self,
                _thread_id: &str,
                _prompt: &str,
                last_message_path: &Path,
                _options: &CodexResumeRequestOptions,
            ) -> Result<(CodexResumeRunResult, WorkflowNodeDispatchExecutionOptions), String>
            {
                if let Some(parent) = last_message_path.parent() {
                    fs::create_dir_all(parent).ok();
                }
                fs::write(last_message_path, "STOP_VIA_CMD_OK").ok();
                // 模拟用户点「停链」：走现成停链命令（按 workflow_id+running 找记录）。能找到 → 证明
                // 本薄驱动建的链记录被现成 stop 命令认得（0-diff 复用）。
                stop_project_workflow_chain_at(
                    &self.state_path,
                    &ProjectWorkflowChainStopRequest {
                        project_root: self.project_root.clone(),
                        workflow_id: self.workflow_id.clone(),
                    },
                )
                .expect("现成 stop_project_workflow_chain 应能找到并停本驱动的 running 链记录");
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
        let runner = StopViaCommandRunner {
            state_path: path.clone(),
            project_root: WORKFLOW_ENGINE_TEST_PROJECT_ROOT.to_string(),
            workflow_id: workflow_id.clone(),
            stats: CodexDispatchReadbackStats {
                transcript_event_count: 3,
                transcript_target_hits: 1,
            },
        };
        let outcome = run_director_task_chain(
            &path,
            &index,
            &index_path,
            &runner,
            WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
            &workflow_id,
            &prepared.plan.planned_tasks,
            10,
        )
        .expect("chain");
        assert_eq!(
            outcome.stopped_reason.as_deref(),
            Some("user_stop_requested"),
            "收到停 → 任务边界停：{:?}",
            outcome.stopped_reason
        );
        assert_eq!(outcome.completed, 1, "只完成第一个，停在边界");
        assert_eq!(outcome.dispatched, 1, "只派发了第一个");
        assert!(
            audit_has(&path, "workflow_chain_run_stopped"),
            "应有停链审计"
        );
        let _ = fs::remove_dir_all(dir);
    }

    // F5·健壮：重复 title → 报错（防后一个永不跑）；dangling 依赖 → 记 warning 不静默丢。
    #[test]
    fn s3_director_chain_rejects_duplicate_titles_and_warns_dangling_dep() {
        let (dir, path, index, index_path, workflow_id, prepared) =
            s3_director_prepared_chain("s3-director-chain-robust");
        let runner = PermissiveExperimentRunner {
            stats: CodexDispatchReadbackStats {
                transcript_event_count: 3,
                transcript_target_hits: 1,
            },
        };
        // 重复 title → Err（不真起链）。
        let mut dup = prepared.plan.planned_tasks.clone();
        if let Some(first_title) = dup.first().map(|t| t.title.clone()) {
            if let Some(last) = dup.last_mut() {
                last.title = first_title;
            }
        }
        let dup_err = run_director_task_chain(
            &path,
            &index,
            &index_path,
            &runner,
            WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
            &workflow_id,
            &dup,
            10,
        );
        assert!(dup_err.is_err(), "重复 title 应拒绝起链");
        assert!(
            dup_err.unwrap_err().contains("重复 title"),
            "错误信息应点名重复 title"
        );
        // dangling 依赖 → 记 warning（不静默丢），链照跑。
        let mut dangling = prepared.plan.planned_tasks.clone();
        if let Some(first) = dangling.first_mut() {
            first.depends_on = vec!["不存在的前置任务X".to_string()];
        }
        let outcome = run_director_task_chain(
            &path,
            &index,
            &index_path,
            &runner,
            WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
            &workflow_id,
            &dangling,
            10,
        )
        .expect("dangling 依赖不致起链失败");
        assert!(
            outcome.warnings.iter().any(|w| w.contains("不存在的前置")),
            "dangling 依赖应记 warning：{:?}",
            outcome.warnings
        );
        let _ = fs::remove_dir_all(dir);
    }

    // path-lock·反：非测试 root → 驱动入口直接拒（零副作用：不建链记录、不起 codex）。
    // 「正」side（测试项目 + 授权 → 真跑）由 s3_director_chain_runs_all_prepared_tasks_topo 覆盖。
    #[test]
    fn s3_director_chain_blocks_non_test_project() {
        let (dir, path, index, index_path, workflow_id, prepared) =
            s3_director_prepared_chain("s3-director-chain-nontest");
        let runner = PermissiveExperimentRunner {
            stats: CodexDispatchReadbackStats {
                transcript_event_count: 3,
                transcript_target_hits: 1,
            },
        };
        let result = run_director_task_chain(
            &path,
            &index,
            &index_path,
            &runner,
            "/tmp/some-non-test-project",
            &workflow_id,
            &prepared.plan.planned_tasks,
            10,
        );
        assert!(result.is_err(), "非测试 root 应被 path-lock 闸拦");
        // 零副作用：没建任何链记录（连环根本没起）。
        let value = read_workflow_state_value(&path).expect("state readable");
        let chain_runs_empty = value
            .get("workflow_chain_runs")
            .and_then(|runs| runs.as_array())
            .map(|runs| runs.is_empty())
            .unwrap_or(true);
        assert!(chain_runs_empty, "非测试 root 不应建任何链记录");
        let _ = fs::remove_dir_all(dir);
    }

    // ===== (b) 放开「角色循环只认默认工作流」→ 可跑在项目内任意合法工作流上 ·stub =====

    // 放开核心证据：建第二条（非默认）工作流 → 在它上面 proposal→确认→边界复核激活 → C4 主管 preview 在那条
    // 非默认工作流上跑通（不再被"必须默认"拦），plan.workflow_id = 那条。
    #[test]
    fn role_loop_runs_on_non_default_project_workflow() {
        let timestamp_ms = 1_765_300_000_000;
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let thread_id = "thread-nondefault-wf";
        let dir = test_temp_dir("role-loop-nondefault");
        let path = dir.join("workflow-state.v0.json");
        let index = fixture_dispatch_index(test_root, thread_id);
        bootstrap_project_workflow_at(&path, &fixture_project(test_root)).expect("workflow");
        // 1. 建第二条（非默认）工作流（带 director 节点）。
        submit_project_workflow_draft_at(
            &path,
            &SubmitProjectWorkflowDraftRequest {
                project_root: test_root.to_string(),
                workflow_id: None,
                title: "非默认角色循环工作流".to_string(),
                nodes: vec![
                    json!({"id":"d1","kind":"director","label":"主管","prompt":"统筹","position":{"x":1,"y":2}}),
                    json!({"id":"a1","kind":"subagent","label":"开发","prompt":"写","position":{"x":3,"y":4}}),
                ],
                edges: vec![json!({"id":"e1","from":"d1","to":"a1"})],
            },
        )
        .expect("submit 2nd workflow");
        let wid = read_json_file(&path)["workflows"]
            .as_array()
            .unwrap()
            .iter()
            .find(|w| optional_string_from(w, "title").as_deref() == Some("非默认角色循环工作流"))
            .and_then(|w| optional_string_from(w, "workflow_id"))
            .expect("new workflow id");
        assert_ne!(wid, default_workflow_id(test_root), "确认是非默认工作流");
        // 2. 在非默认工作流上 proposal → 确认 → 边界复核激活（workflow_id 全程 = wid·不用 default 夹具）。
        let mut input = fixture_project_consultation_proposal_input(test_root);
        input.workflow_id = Some(wid.clone());
        input.scope_draft.allowed_agent_ids = vec![thread_id.to_string()];
        let created = project_consultation_proposal_store::create_proposal(
            &path,
            &input,
            timestamp_ms,
            "write-nondefault-proposal",
        )
        .expect("非默认工作流已存在 → 方案应能创建");
        let confirmed = project_consultation_proposal_store::record_decision(
            &path,
            &RecordProjectConsultationProposalDecisionInput {
                project_root: test_root.to_string(),
                proposal_id: created.proposal.proposal_id.clone(),
                actor_id: "user-fixture".to_string(),
                decision: ProjectConsultationProposalDecisionKind::Confirm,
                summary: "确认非默认工作流方案。".to_string(),
                expected_proposal_store_revision: Some(created.store_revision),
                expected_plan_authorization_store_revision: None,
            },
            timestamp_ms + 1,
            "write-nondefault-confirm",
            "write-nondefault-auth",
            "write-nondefault-auth-user",
        )
        .expect("confirm");
        let authorization = confirmed.plan_authorization.expect("auth");
        let revision = confirmed
            .plan_authorization_store_revision
            .expect("revision");
        // 边界复核输入手搭（workflow_id=wid·default 夹具硬编默认不能用）。
        let activated = plan_authorization_store::record_global_boundary_review_with_proposal(
            &path,
            &RecordGlobalBoundaryReviewInput {
                project_root: test_root.to_string(),
                project_id: project_id(test_root),
                workflow_id: wid.clone(),
                proposal_id: confirmed.proposal.proposal_id.clone(),
                authorization_id: authorization.authorization_id.clone(),
                actor_id: "global-director-fixture".to_string(),
                review_status: "approved".to_string(),
                summary: "全局主管复核通过非默认工作流方案边界。".to_string(),
                checklist: fixture_global_boundary_review_checklist(),
                findings: vec![],
                expected_authorization_revision: Some(revision),
            },
            timestamp_ms + 2,
            "write-nondefault-boundary",
        )
        .expect("activate");
        // 3. C4 主管 preview 在非默认工作流上跑通（放开生效）。
        let preview_input = PreviewProjectDirectorTaskPlanInput {
            project_root: test_root.to_string(),
            project_id: project_id(test_root),
            workflow_id: wid.clone(),
            proposal_id: confirmed.proposal.proposal_id.clone(),
            authorization_id: activated.authorization.authorization_id.clone(),
            actor_id: "project_director".to_string(),
            expected_authorization_revision: Some(activated.store_revision),
        };
        let plan = preview_project_director_task_plan_for_index_at(&path, &index, &preview_input)
            .expect("preview 应在非默认工作流上跑通（放开 C4「只认默认」）");
        assert_eq!(plan.workflow_id, wid, "计划应落在那条非默认工作流上");
        assert!(!plan.planned_tasks.is_empty(), "应拆出任务");
        let _ = fs::remove_dir_all(dir);
    }

    // 合法性闸：放开 ≠ 任意串都过——传不存在的 workflow_id → 拒（防注入不存在/跨项目工作流）。
    #[test]
    fn role_loop_rejects_unknown_workflow() {
        let timestamp_ms = 1_765_300_000_000;
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let thread_id = "thread-unknown-wf";
        let dir = test_temp_dir("role-loop-unknown");
        let path = dir.join("workflow-state.v0.json");
        let index = fixture_dispatch_index(test_root, thread_id);
        bootstrap_project_workflow_at(&path, &fixture_project(test_root)).expect("workflow");
        let (proposal, authorization, revision) =
            create_active_project_director_authorization_fixture(
                &path,
                test_root,
                thread_id,
                timestamp_ms,
            );
        // 传一个不存在的 workflow_id（默认方案/授权在·但 workflow 不存在）→ 拒。
        let preview_input = PreviewProjectDirectorTaskPlanInput {
            project_root: test_root.to_string(),
            project_id: project_id(test_root),
            workflow_id: "workflow:does-not-exist".to_string(),
            proposal_id: proposal.proposal_id.clone(),
            authorization_id: authorization.authorization_id.clone(),
            actor_id: "project_director".to_string(),
            expected_authorization_revision: Some(revision),
        };
        let result = preview_project_director_task_plan_for_index_at(&path, &index, &preview_input);
        assert!(result.is_err(), "不存在的 workflow_id 应被拒（合法性闸）");
        let _ = fs::remove_dir_all(dir);
    }

    // 失败即停：worker 第一步就失败 → 链停、不继续。
    struct FailingChainRunner;
    impl CodexResumeRunner for FailingChainRunner {
        fn resume_with_options(
            &self,
            _thread_id: &str,
            _prompt: &str,
            _last_message_path: &Path,
            _options: &CodexResumeRequestOptions,
        ) -> Result<(CodexResumeRunResult, WorkflowNodeDispatchExecutionOptions), String> {
            Err("boom: worker failed".to_string())
        }
    }

    #[test]
    fn s3_director_chain_fail_stop_halts() {
        let (dir, path, index, index_path, workflow_id, prepared) =
            s3_director_prepared_chain("s3-director-chain-fail");
        let runner = FailingChainRunner;
        let outcome = run_director_task_chain(
            &path,
            &index,
            &index_path,
            &runner,
            WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
            &workflow_id,
            &prepared.plan.planned_tasks,
            10,
        )
        .expect("chain returns outcome");
        assert!(
            outcome
                .stopped_reason
                .as_deref()
                .unwrap_or("")
                .contains("fail_stop"),
            "worker 失败应即停：{:?}",
            outcome.stopped_reason
        );
        assert_eq!(outcome.completed, 0, "第一步就失败 → 0 完成");
        let _ = fs::remove_dir_all(dir);
    }

    // ===== C1·生产起链命令（start_project_director_chain）·stub =====

    // C1 命令线契约（真新增面）：prepare 返回的「已审 planned_tasks」经 JSON 往返回传给起链请求 → 计划无损
    // （depends_on/status 保住）→ 驱动跑这份（= 用户审过的、**不重跑 LM**）→ outcome 能 Serialize 回前端。
    #[test]
    fn s3_director_chain_command_carries_approved_plan_and_serializes_outcome() {
        let (dir, path, index, index_path, workflow_id, prepared) =
            s3_director_prepared_chain("s3-director-chain-cmd");
        // 模拟前端：把 prepare 返回的 planned_tasks 经 JSON 往返回传给起链命令请求。
        let request_json = serde_json::json!({
            "project_root": WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
            "workflow_id": workflow_id,
            "planned_tasks": prepared.plan.planned_tasks,
            "max_nodes": 10,
        });
        let request: StartProjectDirectorChainRequest =
            serde_json::from_value(request_json).expect("起链请求应能反序列化前端回传的已审计划");
        // 计划无损：任务数 / depends_on / status 与 prepare 产出一致（跑的就是用户审过的那份）。
        assert_eq!(
            request.planned_tasks.len(),
            prepared.plan.planned_tasks.len(),
            "回传计划任务数应无损"
        );
        for (got, want) in request
            .planned_tasks
            .iter()
            .zip(prepared.plan.planned_tasks.iter())
        {
            assert_eq!(got.planned_task_id, want.planned_task_id);
            assert_eq!(got.depends_on, want.depends_on, "depends_on 应无损往返");
            assert_eq!(got.status, want.status, "status 应无损往返");
        }
        // 命令内层（= async 壳里 spawn_blocking 调的那一下）：跑回传的计划，不构造任何 director（不重跑 LM）。
        let runner = PermissiveExperimentRunner {
            stats: CodexDispatchReadbackStats {
                transcript_event_count: 3,
                transcript_target_hits: 1,
            },
        };
        let outcome = run_director_task_chain(
            &path,
            &index,
            &index_path,
            &runner,
            &request.project_root,
            &request.workflow_id,
            &request.planned_tasks,
            request.max_nodes.unwrap_or(50),
        )
        .expect("命令内层应起链跑通");
        assert!(outcome.completed >= 2, "回传的多任务计划应按序跑完");
        // outcome 能 Serialize 回前端（命令返回类型）。
        let serialized = serde_json::to_value(&outcome).expect("outcome 应能 Serialize 给前端");
        assert_eq!(
            serialized["completed"],
            serde_json::json!(outcome.completed)
        );
        assert!(
            serialized["chain_run_id"].is_string(),
            "序列化 outcome 应含 chain_run_id"
        );
        assert!(serialized["steps"].is_array(), "序列化 outcome 应含 steps");
        let _ = fs::remove_dir_all(dir);
    }

    // C1 进度复用（0 新命令）：主管链跑完后，现成 get_project_workflow_chain_status 的内层
    // latest_chain_run_for 能按 project_root+workflow_id 读到本驱动建的链记录（state=completed）。
    #[test]
    fn s3_director_chain_command_status_reuse_reads_director_record() {
        let (dir, path, index, index_path, workflow_id, prepared) =
            s3_director_prepared_chain("s3-director-chain-status");
        let runner = PermissiveExperimentRunner {
            stats: CodexDispatchReadbackStats {
                transcript_event_count: 3,
                transcript_target_hits: 1,
            },
        };
        let outcome = run_director_task_chain(
            &path,
            &index,
            &index_path,
            &runner,
            WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
            &workflow_id,
            &prepared.plan.planned_tasks,
            10,
        )
        .expect("chain");
        let value = read_workflow_state_value(&path).expect("state readable");
        let record = latest_chain_run_for(&value, WORKFLOW_ENGINE_TEST_PROJECT_ROOT, &workflow_id)
            .expect("现成进度命令应能按 project+workflow 读到主管链记录");
        assert_eq!(
            optional_string_from(&record, "chain_run_id").as_deref(),
            Some(outcome.chain_run_id.as_str()),
            "进度命令读到的应是本次主管链记录"
        );
        assert_eq!(
            optional_string_from(&record, "state").as_deref(),
            Some("completed"),
            "跑完后链记录 state=completed"
        );
        let _ = fs::remove_dir_all(dir);
    }

    // ===== P1·角色循环「授权后自动推进」编排（auto_advance_authorized_role_loop）·stub =====

    // setup：bootstrap + 造 active 方案授权（proposal + 激活）+ 可选绑 codex-dev 会话 → 返回编排所需件。
    fn auto_advance_fixture(name: &str, bind_session: bool) -> (PathBuf, PathBuf, Value, String) {
        let timestamp_ms = 1_765_300_000_000;
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let thread_id = "thread-auto-advance";
        let dir = test_temp_dir(name);
        let path = dir.join("workflow-state.v0.json");
        let index_path = dir.join("codex-index.json");
        let index = fixture_dispatch_index(test_root, thread_id);
        bootstrap_project_workflow_at(&path, &fixture_project(test_root)).expect("workflow");
        create_active_project_director_authorization_fixture(
            &path,
            test_root,
            thread_id,
            timestamp_ms,
        );
        let workflow_id = default_workflow_id(test_root);
        if bind_session {
            bind_workflow_node_codex_session_for_index_at(
                &path,
                &index,
                &fixture_node_session_bind_request(
                    test_root,
                    &format!("{workflow_id}:node:codex-dev"),
                    None,
                    thread_id,
                ),
            )
            .expect("bind");
        }
        (dir, index_path, index, workflow_id)
    }

    // 全链 stub：active 授权 + 绑会话 → 编排 → StubDirector 拆 → prepare → 链跑 → stage=completed + 审计在。
    #[test]
    fn auto_advance_runs_chain_when_authorized_and_bound() {
        let (dir, index_path, index, workflow_id) = auto_advance_fixture("auto-advance-ran", true);
        let path = dir.join("workflow-state.v0.json");
        let runner = PermissiveExperimentRunner {
            stats: CodexDispatchReadbackStats {
                transcript_event_count: 3,
                transcript_target_hits: 1,
            },
        };
        let outcome = run_auto_advance_authorized_role_loop(
            &path,
            &index,
            &index_path,
            &runner,
            &StubDirector,
            WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
            &workflow_id,
            "tester",
            10,
            None,
        )
        .expect("授权+绑会话应自动推进跑通");
        assert_eq!(outcome.stage, "completed", "应完整跑完链：{outcome:?}");
        let chain = outcome.chain_outcome.expect("completed 应带 chain_outcome");
        assert!(
            chain.completed >= 2,
            "应跑完 ≥2 worker：{}",
            chain.completed
        );
        assert!(
            audit_has(&path, "role_loop_auto_advance_started"),
            "应有编排起审计"
        );
        assert!(
            audit_has(&path, "role_loop_auto_advance_completed"),
            "应有完整完成审计"
        );
        let _ = fs::remove_dir_all(dir);
    }

    // 件 C-1：没绑会话 → prepared==0/needs_binding → 停在 needs_binding、不跑链（链记录都没建）。
    #[test]
    fn auto_advance_stops_at_needs_binding_when_unbound() {
        let (dir, index_path, index, workflow_id) =
            auto_advance_fixture("auto-advance-needsbind", false);
        let path = dir.join("workflow-state.v0.json");
        let runner = PermissiveExperimentRunner {
            stats: CodexDispatchReadbackStats {
                transcript_event_count: 0,
                transcript_target_hits: 0,
            },
        };
        let outcome = run_auto_advance_authorized_role_loop(
            &path,
            &index,
            &index_path,
            &runner,
            &StubDirector,
            WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
            &workflow_id,
            "tester",
            10,
            None,
        )
        .expect("没绑会话也应返回 outcome（停在 needs_binding）");
        assert_eq!(
            outcome.stage, "needs_binding",
            "没绑应停在 needs_binding：{outcome:?}"
        );
        assert_eq!(outcome.prepared_count, 0, "没绑 → 0 prepared");
        assert!(outcome.needs_binding_count > 0, "应有 needs_binding 任务");
        assert!(outcome.chain_outcome.is_none(), "停在 needs_binding 不跑链");
        // 链根本没起：没有任何链运行记录。
        let value = read_workflow_state_value(&path).expect("state readable");
        assert!(
            latest_chain_run_for(&value, WORKFLOW_ENGINE_TEST_PROJECT_ROOT, &workflow_id).is_none(),
            "停在 needs_binding 不应建链记录"
        );
        assert!(
            audit_has(&path, "role_loop_auto_advance_stopped"),
            "应有停因审计"
        );
        let _ = fs::remove_dir_all(dir);
    }

    // 人闸不省：无 active 授权 → 直接拒（不创建、不跳过授权）。
    #[test]
    fn auto_advance_rejects_without_active_authorization() {
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let dir = test_temp_dir("auto-advance-noauth");
        let path = dir.join("workflow-state.v0.json");
        let index_path = dir.join("codex-index.json");
        let index = fixture_dispatch_index(test_root, "thread-noauth");
        bootstrap_project_workflow_at(&path, &fixture_project(test_root)).expect("workflow");
        // 故意不造授权。
        let workflow_id = default_workflow_id(test_root);
        let runner = PermissiveExperimentRunner {
            stats: CodexDispatchReadbackStats {
                transcript_event_count: 0,
                transcript_target_hits: 0,
            },
        };
        let result = run_auto_advance_authorized_role_loop(
            &path,
            &index,
            &index_path,
            &runner,
            &StubDirector,
            test_root,
            &workflow_id,
            "tester",
            10,
            None,
        );
        assert!(result.is_err(), "无 active 授权应被拒");
        assert!(
            result.unwrap_err().contains("active"),
            "错误应点名缺 active 授权"
        );
        let _ = fs::remove_dir_all(dir);
    }

    // 入口 path-lock（决策 2026-06-27 纵深防御）：非测试 root 在 LM 拆 / prepare 之前直接被拒。
    #[test]
    fn auto_advance_blocks_non_test_project() {
        let non_test = "/tmp/some-non-test-project";
        let dir = test_temp_dir("auto-advance-nontest");
        let path = dir.join("workflow-state.v0.json");
        let index_path = dir.join("codex-index.json");
        let index = fixture_dispatch_index(non_test, "thread-nontest");
        let runner = PermissiveExperimentRunner {
            stats: CodexDispatchReadbackStats {
                transcript_event_count: 0,
                transcript_target_hits: 0,
            },
        };
        let result = run_auto_advance_authorized_role_loop(
            &path,
            &index,
            &index_path,
            &runner,
            &StubDirector,
            non_test,
            "wf",
            "tester",
            10,
            None,
        );
        assert!(result.is_err(), "非测试 root 应被入口 path-lock 拒");
        // 有意义：错误须是 path-lock 那条（不是"缺授权"），证 path-lock 在 LM / prepare 之前真拦。
        assert!(
            result
                .unwrap_err()
                .contains("legacy_product_command_blocked"),
            "应是 path-lock 拒绝（legacy_product_command_blocked），证入口提前拦"
        );
        let _ = fs::remove_dir_all(dir);
    }

    // ===== 交办地基·刀1：2.4 flaky retry / 2.5 授权复查 / 2.2 合流命令 / 2.3-existing ·stub =====

    // 早退一次后成功的 runner（call0 → exit1 无输出=偶发早退；此后 → exit0 完成）。
    struct RetryFlakyThenOkRunner {
        calls: std::sync::atomic::AtomicUsize,
        stats: CodexDispatchReadbackStats,
    }
    impl CodexResumeRunner for RetryFlakyThenOkRunner {
        fn resume_with_options(
            &self,
            _thread_id: &str,
            _prompt: &str,
            last_message_path: &Path,
            _options: &CodexResumeRequestOptions,
        ) -> Result<(CodexResumeRunResult, WorkflowNodeDispatchExecutionOptions), String> {
            let n = self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            if n == 0 {
                return Ok((
                    CodexResumeRunResult {
                        exit_code: 1,
                        timed_out: false,
                        stderr_summary: None,
                    },
                    WorkflowNodeDispatchExecutionOptions {
                        readback_stats: None,
                    },
                ));
            }
            if let Some(parent) = last_message_path.parent() {
                fs::create_dir_all(parent).ok();
            }
            fs::write(last_message_path, "RETRY_OK").ok();
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

    // 始终早退（exit1 无输出）——证 retry 只一次·不循环，仍会 fail-stop。
    struct AlwaysEarlyExitRunner;
    impl CodexResumeRunner for AlwaysEarlyExitRunner {
        fn resume_with_options(
            &self,
            _t: &str,
            _p: &str,
            _l: &Path,
            _o: &CodexResumeRequestOptions,
        ) -> Result<(CodexResumeRunResult, WorkflowNodeDispatchExecutionOptions), String> {
            Ok((
                CodexResumeRunResult {
                    exit_code: 1,
                    timed_out: false,
                    stderr_summary: None,
                },
                WorkflowNodeDispatchExecutionOptions {
                    readback_stats: None,
                },
            ))
        }
    }

    // 超时（exit≠0 但 timed_out）——证 timeout **不** retry（只早退才 retry）。
    struct TimeoutRunner;
    impl CodexResumeRunner for TimeoutRunner {
        fn resume_with_options(
            &self,
            _t: &str,
            _p: &str,
            _l: &Path,
            _o: &CodexResumeRequestOptions,
        ) -> Result<(CodexResumeRunResult, WorkflowNodeDispatchExecutionOptions), String> {
            Ok((
                CodexResumeRunResult {
                    exit_code: 1,
                    timed_out: true,
                    stderr_summary: Some("timeout".to_string()),
                },
                WorkflowNodeDispatchExecutionOptions {
                    readback_stats: None,
                },
            ))
        }
    }

    // 2.4：偶发早退 → 自动重试一次 → 成功；warnings 记「已自动重试」。
    #[test]
    fn chain_retries_once_on_early_exit_then_succeeds() {
        let (dir, path, index, index_path, workflow_id, prepared) =
            s3_director_prepared_chain("chain-retry-ok");
        let runner = RetryFlakyThenOkRunner {
            calls: std::sync::atomic::AtomicUsize::new(0),
            stats: CodexDispatchReadbackStats {
                transcript_event_count: 3,
                transcript_target_hits: 1,
            },
        };
        let outcome = run_director_task_chain(
            &path,
            &index,
            &index_path,
            &runner,
            WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
            &workflow_id,
            &prepared.plan.planned_tasks,
            10,
        )
        .expect("chain");
        assert!(
            outcome.stopped_reason.is_none() && outcome.completed >= 1,
            "偶发早退重试一次后应跑通：{outcome:?}"
        );
        assert!(
            outcome.warnings.iter().any(|w| w.contains("自动重试")),
            "warnings 应记已自动重试：{:?}",
            outcome.warnings
        );
        let _ = fs::remove_dir_all(dir);
    }

    // 2.4：始终早退 → 重试一次仍败 → fail-stop（证只一次·不循环）。
    #[test]
    fn chain_fail_stops_after_single_retry_on_persistent_early_exit() {
        let (dir, path, index, index_path, workflow_id, prepared) =
            s3_director_prepared_chain("chain-retry-persist");
        let outcome = run_director_task_chain(
            &path,
            &index,
            &index_path,
            &AlwaysEarlyExitRunner,
            WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
            &workflow_id,
            &prepared.plan.planned_tasks,
            10,
        )
        .expect("chain returns outcome");
        assert!(
            outcome
                .stopped_reason
                .as_deref()
                .unwrap_or("")
                .contains("fail_stop"),
            "持续早退·重试一次仍败 → fail-stop（不循环）：{:?}",
            outcome.stopped_reason
        );
        assert!(
            outcome.warnings.iter().any(|w| w.contains("自动重试")),
            "应记重试过一次"
        );
        assert_eq!(outcome.completed, 0, "都没完成");
        let _ = fs::remove_dir_all(dir);
    }

    // 2.4：timeout **不** retry（只早退才 retry）——无「自动重试」warning、直接 fail-stop。
    #[test]
    fn chain_does_not_retry_timeout() {
        let (dir, path, index, index_path, workflow_id, prepared) =
            s3_director_prepared_chain("chain-timeout-noretry");
        let outcome = run_director_task_chain(
            &path,
            &index,
            &index_path,
            &TimeoutRunner,
            WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
            &workflow_id,
            &prepared.plan.planned_tasks,
            10,
        )
        .expect("chain returns outcome");
        assert!(
            outcome
                .stopped_reason
                .as_deref()
                .unwrap_or("")
                .contains("fail_stop"),
            "timeout 直接 fail-stop：{:?}",
            outcome.stopped_reason
        );
        assert!(
            !outcome.warnings.iter().any(|w| w.contains("自动重试")),
            "timeout 不应触发重试：{:?}",
            outcome.warnings
        );
        let _ = fs::remove_dir_all(dir);
    }

    // 2.5：授权在 → require_active_authorization Ok；撤销后 → 拒（起链前复查）。
    #[test]
    fn require_active_authorization_ok_then_rejects_revoked() {
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let dir = test_temp_dir("require-active-auth");
        let path = dir.join("workflow-state.v0.json");
        bootstrap_project_workflow_at(&path, &fixture_project(test_root)).expect("workflow");
        let (_proposal, authorization, revision) =
            create_active_project_director_authorization_fixture(
                &path,
                test_root,
                "thread-recheck",
                1_765_300_000_000,
            );
        let workflow_id = default_workflow_id(test_root);
        require_active_authorization(&path, test_root, &workflow_id).expect("active → Ok");
        // 撤销授权 → 复查应拒。
        plan_authorization_store::revoke_authorization(
            &path,
            &RevokePlanAuthorizationInput {
                project_root: test_root.to_string(),
                authorization_id: authorization.authorization_id.clone(),
                actor_id: "user-fixture".to_string(),
                actor_role: "global_director".to_string(),
                reason: "测试撤销".to_string(),
                expected_store_revision: Some(revision),
            },
            1_765_300_000_100,
            "write-revoke",
        )
        .expect("revoke");
        assert!(
            require_active_authorization(&path, test_root, &workflow_id).is_err(),
            "授权撤销后·起链前复查应拒"
        );
        let _ = fs::remove_dir_all(dir);
    }

    // 开工前绑定面板：顶层旧会话只作第一项预填。确认后必须停在 needs_binding，绝不把它直接绑成整链共用。
    #[test]
    fn confirm_and_start_runs_from_pending_with_existing_session() {
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let thread_id = "thread-jiaoban";
        let dir = test_temp_dir("confirm-and-start");
        let path = dir.join("workflow-state.v0.json");
        let index_path = dir.join("codex-index.json");
        let index = fixture_dispatch_index(test_root, thread_id);
        bootstrap_project_workflow_at(&path, &fixture_project(test_root)).expect("workflow");
        // 造 Pending 方案（要改东西·execution_scope Some → map → 档位 写范围 + codex-dev）。
        let proposal = consult_proposal_fixture(Some(ConsultationExecutionScope {
            write_roots: vec![],
            target_files: vec!["a.rs".to_string()],
            tools: vec![],
            checks: vec!["cargo test".to_string()],
        }));
        let c1_input =
            map_consultation_to_c1_input(&proposal, test_root, "consultant").expect("map");
        let created = project_consultation_proposal_store::create_proposal(
            &path,
            &c1_input,
            1_765_300_000_000,
            "write-jiaoban-proposal",
        )
        .expect("proposal");
        assert!(
            matches!(
                created.proposal.status,
                ProjectConsultationProposalStatus::PendingUserConfirmation
            ),
            "方案应 Pending"
        );
        let runner = PermissiveExperimentRunner {
            stats: CodexDispatchReadbackStats {
                transcript_event_count: 3,
                transcript_target_hits: 1,
            },
        };
        let request = ConfirmAndStartAuthorizedRunRequest {
            project_root: test_root.to_string(),
            proposal_id: created.proposal.proposal_id.clone(),
            session_choice: "existing".to_string(),
            session_id: Some(thread_id.to_string()),
            actor_id: Some("user-fixture".to_string()),
            max_nodes: Some(10),
            approved_planned_tasks: None,
            preview_session_bindings: vec![],
        };
        let paused = run_confirm_and_start_authorized_run_inner(
            &path,
            &index,
            &index_path,
            &runner,
            &StubDirector,
            // 回归护栏：existing 分支绝不碰新会话出生口（碰到即 panic）。
            &PanicJiaobanSessionCreator,
            &request,
        )
        .expect("确认后应停在逐任务绑定面板");
        assert_eq!(
            paused.stage, "needs_binding",
            "确认→复核→拆任务后应停在绑定面板：{paused:?}"
        );
        assert!(paused.task_session_binding_required, "需要逐任务映射标记");
        assert_eq!(paused.prepared_count, 0, "停点前不得 prepare/派发");
        assert!(
            read_json_file(&path)["workflow_node_session_bindings"]
                .as_array()
                .map(|bindings| bindings.is_empty())
                .unwrap_or(true),
            "顶层 existing 不得写全局 codex-dev 绑定"
        );
        let _ = fs::remove_dir_all(dir);
    }

    // M1：批前画布已逐项选好时，仍只点一次人闸；后端按稳定步骤 id 映射并复用原绑定确认路径。
    #[test]
    fn confirm_and_start_auto_confirms_preview_node_bindings() {
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let thread_id = "thread-jiaoban-preview";
        let dir = test_temp_dir("confirm-preview-bindings");
        let path = dir.join("workflow-state.v0.json");
        let index_path = dir.join("codex-index.json");
        let index = fixture_dispatch_index(test_root, thread_id);
        bootstrap_project_workflow_at(&path, &fixture_project(test_root)).expect("workflow");
        let proposal = consult_proposal_fixture(Some(ConsultationExecutionScope {
            target_files: vec!["a.rs".to_string()],
            ..Default::default()
        }));
        let c1 = map_consultation_to_c1_input(&proposal, test_root, "consultant").expect("map");
        let created = project_consultation_proposal_store::create_proposal(
            &path,
            &c1,
            1_765_300_000_000,
            "confirm-preview-bindings",
        )
        .expect("proposal");
        let executor = PermissiveExperimentRunner {
            stats: CodexDispatchReadbackStats {
                transcript_event_count: 3,
                transcript_target_hits: 1,
            },
        };
        let preview_session_bindings = (1..=2)
            .map(|step| ProjectDirectorPreviewNodeSessionBinding {
                preview_node_id: format!("planned-task:{}:{step}", created.proposal.workflow_id),
                session_choice: "existing".to_string(),
                session_id: Some(thread_id.to_string()),
            })
            .collect();
        let outcome = run_confirm_and_start_authorized_run_inner(
            &path,
            &index,
            &index_path,
            &executor,
            &StubDirector,
            &PanicJiaobanSessionCreator,
            &ConfirmAndStartAuthorizedRunRequest {
                project_root: test_root.to_string(),
                proposal_id: created.proposal.proposal_id.clone(),
                session_choice: "new".to_string(),
                session_id: None,
                actor_id: Some("user-fixture".to_string()),
                max_nodes: Some(10),
                approved_planned_tasks: None,
                preview_session_bindings,
            },
        )
        .expect("预演节点映射齐全时应自动继续");
        assert_eq!(
            outcome.stage, "completed",
            "不应再停在绑定面板：{outcome:?}"
        );
        assert!(
            !outcome.task_session_binding_required,
            "自动确认后不应残留绑定停点"
        );
        let state = read_json_file(&path);
        let bindings = state["workflow_node_session_bindings"]
            .as_array()
            .expect("每项任务应落自己的绑定");
        assert_eq!(bindings.len(), 2, "两项预演选择应分别写入任务绑定");
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_session_binding_rejects_missing_extra_and_unavailable_existing_session() {
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let dir = test_temp_dir("task-session-binding-validation");
        let path = dir.join("workflow-state.v0.json");
        let index_path = dir.join("codex-index.json");
        let index = fixture_dispatch_index(test_root, "thread-existing");
        bootstrap_project_workflow_at(&path, &fixture_project(test_root)).expect("workflow");
        let proposal = consult_proposal_fixture(Some(ConsultationExecutionScope {
            target_files: vec!["a.rs".to_string()],
            ..Default::default()
        }));
        let c1 = map_consultation_to_c1_input(&proposal, test_root, "consultant").expect("map");
        let created = project_consultation_proposal_store::create_proposal(
            &path,
            &c1,
            1_765_300_000_000,
            "task-session-binding-validation",
        )
        .expect("proposal");
        let runner = PermissiveExperimentRunner {
            stats: CodexDispatchReadbackStats {
                transcript_event_count: 0,
                transcript_target_hits: 0,
            },
        };
        let paused = run_confirm_and_start_authorized_run_inner(
            &path,
            &index,
            &index_path,
            &runner,
            &StubDirector,
            &PanicJiaobanSessionCreator,
            &ConfirmAndStartAuthorizedRunRequest {
                project_root: test_root.to_string(),
                proposal_id: created.proposal.proposal_id.clone(),
                session_choice: "new".to_string(),
                session_id: None,
                actor_id: Some("user-fixture".to_string()),
                max_nodes: Some(10),
                approved_planned_tasks: None,
                preview_session_bindings: vec![],
            },
        )
        .expect("确认应给绑定面板");
        let make_request =
            |task_session_bindings| ConfirmProjectDirectorTaskSessionBindingsRequest {
                project_root: test_root.to_string(),
                workflow_id: created.proposal.workflow_id.clone(),
                planned_tasks: paused.planned_tasks.clone(),
                task_session_bindings,
                actor_id: Some("user-fixture".to_string()),
                max_nodes: Some(10),
            };

        let missing = vec![ProjectDirectorTaskSessionBinding {
            planned_task_id: paused.planned_tasks[0].planned_task_id.clone(),
            session_choice: "new".to_string(),
            session_id: None,
        }];
        let missing_error = run_confirm_project_director_task_session_bindings_inner(
            &path,
            &index,
            &index_path,
            &runner,
            &StubDirector,
            &PanicJiaobanSessionCreator,
            &make_request(missing),
        )
        .expect_err("漏一项任务映射必须拒绝");
        assert!(missing_error.contains("缺项或多项"), "{missing_error}");

        let mut extra: Vec<ProjectDirectorTaskSessionBinding> = paused
            .planned_tasks
            .iter()
            .map(|task| ProjectDirectorTaskSessionBinding {
                planned_task_id: task.planned_task_id.clone(),
                session_choice: "new".to_string(),
                session_id: None,
            })
            .collect();
        extra.push(ProjectDirectorTaskSessionBinding {
            planned_task_id: "not-a-planned-task".to_string(),
            session_choice: "new".to_string(),
            session_id: None,
        });
        let extra_error = run_confirm_project_director_task_session_bindings_inner(
            &path,
            &index,
            &index_path,
            &runner,
            &StubDirector,
            &PanicJiaobanSessionCreator,
            &make_request(extra),
        )
        .expect_err("多一项任务映射必须拒绝");
        assert!(extra_error.contains("缺项或多项"), "{extra_error}");

        let unavailable: Vec<ProjectDirectorTaskSessionBinding> = paused
            .planned_tasks
            .iter()
            .enumerate()
            .map(|(position, task)| ProjectDirectorTaskSessionBinding {
                planned_task_id: task.planned_task_id.clone(),
                session_choice: if position == 0 {
                    "existing".to_string()
                } else {
                    "new".to_string()
                },
                session_id: if position == 0 {
                    Some("thread-no-longer-available".to_string())
                } else {
                    None
                },
            })
            .collect();
        let unavailable_error = run_confirm_project_director_task_session_bindings_inner(
            &path,
            &index,
            &index_path,
            &runner,
            &StubDirector,
            &PanicJiaobanSessionCreator,
            &make_request(unavailable),
        )
        .expect_err("已有会话不存在必须拒绝");
        assert!(
            unavailable_error.contains("已有对话已不可用"),
            "{unavailable_error}"
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn task_session_binding_binds_existing_and_new_sessions_to_their_own_tasks() {
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let dir = test_temp_dir("task-session-binding-per-task");
        let path = dir.join("workflow-state.v0.json");
        let index_path = dir.join("codex-index.json");
        let index = fixture_multi_thread_index(test_root, &["thread-existing", "thread-new"]);
        bootstrap_project_workflow_at(&path, &fixture_project(test_root)).expect("workflow");
        let proposal = consult_proposal_fixture(Some(ConsultationExecutionScope {
            target_files: vec!["a.rs".to_string()],
            ..Default::default()
        }));
        let c1 = map_consultation_to_c1_input(&proposal, test_root, "consultant").expect("map");
        let created = project_consultation_proposal_store::create_proposal(
            &path,
            &c1,
            1_765_300_000_000,
            "task-session-binding-per-task",
        )
        .expect("proposal");
        let runner = PermissiveExperimentRunner {
            stats: CodexDispatchReadbackStats {
                transcript_event_count: 3,
                transcript_target_hits: 1,
            },
        };
        let creator = StubJiaobanSessionCreator {
            thread_id: "thread-new",
            received_texts: std::cell::RefCell::new(vec![]),
        };
        let paused = run_confirm_and_start_authorized_run_inner(
            &path,
            &index,
            &index_path,
            &runner,
            &StubDirector,
            &creator,
            &ConfirmAndStartAuthorizedRunRequest {
                project_root: test_root.to_string(),
                proposal_id: created.proposal.proposal_id.clone(),
                session_choice: "existing".to_string(),
                session_id: Some("thread-existing".to_string()),
                actor_id: Some("user-fixture".to_string()),
                max_nodes: Some(10),
                approved_planned_tasks: None,
                preview_session_bindings: vec![],
            },
        )
        .expect("确认应给绑定面板");
        assert_eq!(paused.planned_tasks.len(), 2, "fixture 需要两个任务");
        let binding_request = ConfirmProjectDirectorTaskSessionBindingsRequest {
            project_root: test_root.to_string(),
            workflow_id: created.proposal.workflow_id.clone(),
            task_session_bindings: paused
                .planned_tasks
                .iter()
                .enumerate()
                .map(|(position, task)| ProjectDirectorTaskSessionBinding {
                    planned_task_id: task.planned_task_id.clone(),
                    session_choice: if position == 0 {
                        "existing".to_string()
                    } else {
                        "new".to_string()
                    },
                    session_id: if position == 0 {
                        Some("thread-existing".to_string())
                    } else {
                        None
                    },
                })
                .collect(),
            planned_tasks: paused.planned_tasks,
            actor_id: Some("user-fixture".to_string()),
            max_nodes: Some(10),
        };
        let outcome = run_confirm_project_director_task_session_bindings_inner(
            &path,
            &index,
            &index_path,
            &runner,
            &StubDirector,
            &creator,
            &binding_request,
        )
        .expect("混合映射应跑通");
        assert_eq!(outcome.stage, "completed", "{outcome:?}");
        assert_eq!(
            creator.received_texts.borrow().len(),
            1,
            "只有选 new 的任务应触发一次 C1"
        );
        let state = read_json_file(&path);
        let bindings = state["workflow_node_session_bindings"]
            .as_array()
            .expect("绑定记录");
        for (position, task) in outcome.planned_tasks.iter().enumerate() {
            let work_item_id = task.work_item_id.as_deref().expect("prepared work item");
            let node_id = task.workflow_node_id.as_deref().expect("prepared node");
            let binding = bindings
                .iter()
                .find(|binding| {
                    optional_string_from(binding, "work_item_id").as_deref() == Some(work_item_id)
                })
                .expect("每项任务都应有自己的绑定");
            assert_eq!(
                optional_string_from(binding, "node_id").as_deref(),
                Some(node_id),
                "绑定必须落到该任务自己的节点"
            );
            assert_eq!(
                optional_string_from(binding, "native_thread_id").as_deref(),
                Some(if position == 0 {
                    "thread-existing"
                } else {
                    "thread-new"
                }),
                "每项任务应使用用户选择的会话"
            );
        }
        let _ = fs::remove_dir_all(dir);
    }

    // 2.2 人闸：非 Pending 方案 → 拒；未知 session_choice → 清错。（原「new 清错拒」断言已被方案a 取代——
    // new 现在真接了，见 confirm_and_start_new_session_* 三测。）
    #[test]
    fn confirm_and_start_rejects_non_pending_and_new_session() {
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let dir = test_temp_dir("confirm-and-start-guard");
        let path = dir.join("workflow-state.v0.json");
        let index_path = dir.join("codex-index.json");
        let index = fixture_dispatch_index(test_root, "thread-guard");
        bootstrap_project_workflow_at(&path, &fixture_project(test_root)).expect("workflow");
        // 已 active 的方案（非 Pending）——人闸应拒。
        let (proposal, _auth, _rev) = create_active_project_director_authorization_fixture(
            &path,
            test_root,
            "thread-guard",
            1_765_300_000_000,
        );
        let runner = PermissiveExperimentRunner {
            stats: CodexDispatchReadbackStats {
                transcript_event_count: 0,
                transcript_target_hits: 0,
            },
        };
        let non_pending = ConfirmAndStartAuthorizedRunRequest {
            project_root: test_root.to_string(),
            proposal_id: proposal.proposal_id.clone(),
            session_choice: "existing".to_string(),
            session_id: Some("thread-guard".to_string()),
            actor_id: Some("user-fixture".to_string()),
            max_nodes: Some(10),
            approved_planned_tasks: None,
            preview_session_bindings: vec![],
        };
        assert!(
            run_confirm_and_start_authorized_run_inner(
                &path,
                &index,
                &index_path,
                &runner,
                &StubDirector,
                &PanicJiaobanSessionCreator,
                &non_pending,
            )
            .is_err(),
            "非 Pending 方案·人闸应拒（本命令只表达用户刚点允许）"
        );
        // 未知 session_choice → 清错（非静默）。需一个 Pending 方案触到 session 分流前。
        let proposal2 = consult_proposal_fixture(Some(ConsultationExecutionScope {
            target_files: vec!["a.rs".to_string()],
            ..Default::default()
        }));
        let c1 = map_consultation_to_c1_input(&proposal2, test_root, "consultant").expect("map");
        let created2 = project_consultation_proposal_store::create_proposal(
            &path,
            &c1,
            1_765_300_001_000,
            "write-guard-proposal2",
        )
        .expect("proposal2");
        let weird_choice = ConfirmAndStartAuthorizedRunRequest {
            project_root: test_root.to_string(),
            proposal_id: created2.proposal.proposal_id.clone(),
            session_choice: "both".to_string(),
            session_id: None,
            actor_id: Some("user-fixture".to_string()),
            max_nodes: Some(10),
            approved_planned_tasks: None,
            preview_session_bindings: vec![],
        };
        let err = run_confirm_and_start_authorized_run_inner(
            &path,
            &index,
            &index_path,
            &runner,
            &StubDirector,
            &PanicJiaobanSessionCreator,
            &weird_choice,
        )
        .expect_err("未知 session_choice 应拒");
        assert!(
            err.contains("未知 session_choice"),
            "错误应点名未知选项：{err}"
        );
        let _ = fs::remove_dir_all(dir);
    }

    // 方案a·先生后绑①（stub 出生口）：session_choice=new → 建会话 → existing 同款绑定 → 链照旧推进全通；
    // outcome 带「已为这单活新建会话」人话说明；初始化文案人话点名方案（出生口收到的就是这份）。
    // C1 收官（canon 2026-07-09·prepare C1-aware 后合流-new 退 S0）：合流 session_choice=new →
    // **每任务**先生后绑（非 S0 一次性）——prepare 走 chain_binds_per_task=true 产 prepared·链每任务各建会话。
    // 断言 C1 每任务信号：出生口被调=任务数（S0 时代恒 1·C1=N）· 各任务 target_session_id 互异物化 ·
    // 初始化文案是「任务专用会话」逐条人话（非 S0 的「新会话初始化」总纲）。原 S0 版断言 texts.len()==1 /
    // 单条 codex-dev 绑定 / 新会话 notice = 拐杖，退 S0 后失效，按 C1 每任务改写（守卫零改·失败无回落见下一测）。
    #[test]
    fn confirm_and_start_new_session_births_binds_and_advances() {
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let dir = test_temp_dir("confirm-and-start-new");
        let path = dir.join("workflow-state.v0.json");
        let index_path = dir.join("codex-index.json");
        // C1 每任务各开新会话 → 索引须含每任务的新 thread（StubDirector 拆 2 任务 → 2 条·对齐出生口回执）。
        let threads_json: Vec<Value> = ["thread-c1-new-1", "thread-c1-new-2"]
            .iter()
            .map(|t| {
                json!({
                    "thread_id": t, "project_root": test_root, "title": format!("Session {t}"),
                    "rollout_exists": true, "rollout_path": format!("/tmp/{t}.jsonl")
                })
            })
            .collect();
        let index = json!({
            "projects": [{ "project_root": test_root }],
            "threads": threads_json
        });
        bootstrap_project_workflow_at(&path, &fixture_project(test_root)).expect("workflow");
        let proposal = consult_proposal_fixture(Some(ConsultationExecutionScope {
            write_roots: vec![],
            target_files: vec!["a.rs".to_string()],
            tools: vec![],
            checks: vec!["cargo test".to_string()],
        }));
        let c1_input =
            map_consultation_to_c1_input(&proposal, test_root, "consultant").expect("map");
        let created = project_consultation_proposal_store::create_proposal(
            &path,
            &c1_input,
            1_765_300_000_000,
            "write-plan-a-proposal",
        )
        .expect("proposal");
        // C1·每任务先生后绑：出生口每次返回互异 thread（对齐索引里那 2 条）+ 记初始化文案。
        struct DistinctCreator {
            calls: std::cell::Cell<usize>,
            texts: std::cell::RefCell<Vec<String>>,
        }
        impl JiaobanNewSessionCreator for DistinctCreator {
            fn create_initialized_session(&self, text: &str, _by: &str) -> Result<String, String> {
                self.texts.borrow_mut().push(text.to_string());
                let n = self.calls.get() + 1;
                self.calls.set(n);
                Ok(format!("thread-c1-new-{n}"))
            }
        }
        let creator = DistinctCreator {
            calls: std::cell::Cell::new(0),
            texts: std::cell::RefCell::new(vec![]),
        };
        let runner = PermissiveExperimentRunner {
            stats: CodexDispatchReadbackStats {
                transcript_event_count: 3,
                transcript_target_hits: 1,
            },
        };
        let request = ConfirmAndStartAuthorizedRunRequest {
            project_root: test_root.to_string(),
            proposal_id: created.proposal.proposal_id.clone(),
            session_choice: "new".to_string(),
            session_id: None,
            actor_id: Some("user-fixture".to_string()),
            max_nodes: Some(10),
            approved_planned_tasks: None,
            preview_session_bindings: vec![],
        };
        let paused = run_confirm_and_start_authorized_run_inner(
            &path,
            &index,
            &index_path,
            &runner,
            &StubDirector,
            &creator,
            &request,
        )
        .expect("确认应先停在逐任务绑定面板");
        assert_eq!(paused.stage, "needs_binding", "{paused:?}");
        assert!(paused.task_session_binding_required, "{paused:?}");
        let task_session_bindings = paused
            .planned_tasks
            .iter()
            .map(|task| ProjectDirectorTaskSessionBinding {
                planned_task_id: task.planned_task_id.clone(),
                session_choice: "new".to_string(),
                session_id: None,
            })
            .collect();
        let binding_request = ConfirmProjectDirectorTaskSessionBindingsRequest {
            project_root: test_root.to_string(),
            workflow_id: created.proposal.workflow_id.clone(),
            planned_tasks: paused.planned_tasks,
            task_session_bindings,
            actor_id: Some("user-fixture".to_string()),
            max_nodes: Some(10),
        };
        let outcome = run_confirm_project_director_task_session_bindings_inner(
            &path,
            &index,
            &index_path,
            &runner,
            &StubDirector,
            &creator,
            &binding_request,
        )
        .expect("全新映射应逐任务建→绑→推进");
        assert_eq!(outcome.stage, "completed", "建→绑→链应全通：{outcome:?}");
        assert!(
            outcome
                .chain_outcome
                .as_ref()
                .map(|chain| chain.completed >= 1)
                .unwrap_or(false),
            "worker 链应跑出结果"
        );
        // C1-aware 核证：全新未绑 + chain_binds_per_task=true → prepare **跳过 needs_binding**、产 prepared·
        // thread 延迟，留**透明审计新变体**（Edit C/D）——退 S0 前这里恒 needs_binding·0 prepared·空转。
        assert!(
            audit_has(&path, "authorized_prepared_dispatch_thread_deferred"),
            "C1 deferred 应留透明审计变体 authorized_prepared_dispatch_thread_deferred（证 prepare 走了延迟路·非 needs_binding）"
        );
        // C1 每任务信号①：出生口被调=任务数（StubDirector 拆 2 → 2 次；S0 时代恒 1）。
        assert_eq!(
            creator.calls.get(),
            2,
            "C1 每任务各建会话·出生口应被调 2 次（非 S0 一次性）"
        );
        // C1 每任务信号②：初始化文案逐条是「任务专用会话」人话（非 S0 的总纲式「新会话初始化」）。
        let texts = creator.texts.borrow();
        assert!(
            !texts.is_empty() && texts.iter().all(|t| t.contains("交办任务专用会话")),
            "每条初始化文案应人话点名任务专用：{texts:?}"
        );
        // C1 每任务信号③：各任务 target_session_id 互异物化（先生后绑把新 thread 回填任务包 artifact）。
        let state = read_json_file(&path);
        let session_ids: std::collections::BTreeSet<String> = state["artifacts"]
            .as_array()
            .cloned()
            .unwrap_or_default()
            .iter()
            .filter_map(|artifact| optional_string_from(artifact, "target_session_id"))
            .collect();
        assert_eq!(
            session_ids.len(),
            2,
            "2 任务各物化互异 target_session_id：{session_ids:?}"
        );
        let _ = fs::remove_dir_all(dir);
    }

    // C1 收官（canon 2026-07-09·合流-new 退 S0 后）：建会话失败发生在**链内每任务**（create_and_bind·
    // director_agent:1128），不再是 S0 一次性预建。**守卫语义保留**：失败即停·**不回落共用会话**·**不派空会话**
    //（execute 在 create_and_bind 成功后才走·失败在 execute 前 return → 派不到空会话）。形态变：S0 时代外层返
    // Err；C1 链自身 Ok+stopped_reason=fail_stop:session_create（auto_advance 内层已留档·非外抛 Err）。
    // 这是**守卫测**（失败无回落·§4 对抗式自检的测试面证据），不是拐杖——按 C1 形态微调、语义一字不松。
    #[test]
    fn confirm_and_start_new_session_failure_audits_no_fallback() {
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let dir = test_temp_dir("confirm-and-start-new-fail");
        let path = dir.join("workflow-state.v0.json");
        let index_path = dir.join("codex-index.json");
        let index = fixture_dispatch_index(test_root, "thread-unused");
        bootstrap_project_workflow_at(&path, &fixture_project(test_root)).expect("workflow");
        let proposal = consult_proposal_fixture(Some(ConsultationExecutionScope {
            target_files: vec!["a.rs".to_string()],
            ..Default::default()
        }));
        let c1_input =
            map_consultation_to_c1_input(&proposal, test_root, "consultant").expect("map");
        let created = project_consultation_proposal_store::create_proposal(
            &path,
            &c1_input,
            1_765_300_000_000,
            "write-plan-a-fail-proposal",
        )
        .expect("proposal");
        let creator = FailingJiaobanSessionCreator {
            called: std::cell::RefCell::new(false),
        };
        let runner = PermissiveExperimentRunner {
            stats: CodexDispatchReadbackStats {
                transcript_event_count: 0,
                transcript_target_hits: 0,
            },
        };
        let request = ConfirmAndStartAuthorizedRunRequest {
            project_root: test_root.to_string(),
            proposal_id: created.proposal.proposal_id.clone(),
            session_choice: "new".to_string(),
            session_id: None,
            actor_id: Some("user-fixture".to_string()),
            max_nodes: Some(10),
            approved_planned_tasks: None,
            preview_session_bindings: vec![],
        };
        let paused = run_confirm_and_start_authorized_run_inner(
            &path,
            &index,
            &index_path,
            &runner,
            &StubDirector,
            &creator,
            &request,
        )
        .expect("确认应先停在逐任务绑定面板");
        assert_eq!(paused.stage, "needs_binding", "{paused:?}");
        let binding_request = ConfirmProjectDirectorTaskSessionBindingsRequest {
            project_root: test_root.to_string(),
            workflow_id: created.proposal.workflow_id.clone(),
            task_session_bindings: paused
                .planned_tasks
                .iter()
                .map(|task| ProjectDirectorTaskSessionBinding {
                    planned_task_id: task.planned_task_id.clone(),
                    session_choice: "new".to_string(),
                    session_id: None,
                })
                .collect(),
            planned_tasks: paused.planned_tasks,
            actor_id: Some("user-fixture".to_string()),
            max_nodes: Some(10),
        };
        let outcome = run_confirm_project_director_task_session_bindings_inner(
            &path,
            &index,
            &index_path,
            &runner,
            &StubDirector,
            &creator,
            &binding_request,
        )
        .expect("C1 链内建会话失败即停走 Ok（链自报 stopped）·不外抛 Err");
        // ① 确实走了出生口（不是悄悄改走别的路/回落）。
        assert!(
            *creator.called.borrow(),
            "确实走的是出生口（不是悄悄改走别的路）"
        );
        // ② 失败即停信号：链因建会话失败停（stopped_reason=fail_stop:session_create:…）·非跑完。
        let chain = outcome.chain_outcome.as_ref().expect("应有链结果");
        assert!(
            chain
                .stopped_reason
                .as_deref()
                .map(|reason| reason.starts_with("fail_stop:session_create:"))
                .unwrap_or(false),
            "应因建会话失败即停：{:?}",
            chain.stopped_reason
        );
        // ③ **不派空会话**（§4 测试面）：execute 在 create_and_bind 成功后才走，建失败即停在 execute 前 →
        //    一个任务都没 completed。
        assert_eq!(
            chain.completed, 0,
            "失败即停不该有任务派发完成（不派空会话）"
        );
        // ④ **不回落**：create 失败在 bind 之前 → 没绑任何会话（不静默回落 existing/共用）。
        let state = read_json_file(&path);
        let binding_count = state["workflow_node_session_bindings"]
            .as_array()
            .map(|bindings| bindings.len())
            .unwrap_or(0);
        assert_eq!(binding_count, 0, "失败路径不该绑任何会话（不回落）");
        // ⑤ 留档：状态里带人话停因（建会话失败与原因·透明不吞）。
        let state_text = fs::read_to_string(&path).expect("state");
        assert!(
            state_text.contains("新建会话失败") && state_text.contains("codex 起不来"),
            "留档应带人话停因（建会话失败与原因）"
        );
        let _ = fs::remove_dir_all(dir);
    }

    // 方案a·先生后绑③：非测试项目 root → path-lock 在建会话之前就拒（出生口一次没被碰=Panic 桩没炸）。
    #[test]
    fn confirm_and_start_new_session_rejected_outside_test_project() {
        let dir = test_temp_dir("confirm-and-start-new-lock");
        let path = dir.join("workflow-state.v0.json");
        let index = fixture_dispatch_index("/Users/yoyi/some-real-project", "thread-x");
        let runner = PermissiveExperimentRunner {
            stats: CodexDispatchReadbackStats {
                transcript_event_count: 0,
                transcript_target_hits: 0,
            },
        };
        let request = ConfirmAndStartAuthorizedRunRequest {
            project_root: "/Users/yoyi/some-real-project".to_string(),
            proposal_id: "proposal-x".to_string(),
            session_choice: "new".to_string(),
            session_id: None,
            actor_id: Some("user-fixture".to_string()),
            max_nodes: Some(10),
            approved_planned_tasks: None,
            preview_session_bindings: vec![],
        };
        let err = run_confirm_and_start_authorized_run_inner(
            &path,
            &index,
            &path,
            &runner,
            &StubDirector,
            &PanicJiaobanSessionCreator,
            &request,
        )
        .expect_err("非测试 root 应拒（path-lock 在最前）");
        assert!(
            err.contains("legacy_product_command_blocked"),
            "应是 path-lock 拒：{err}"
        );
        let _ = fs::remove_dir_all(dir);
    }

    // C1 收官·§4 真跑（单独步·#[ignore]·固定测试项目·用户点击直接效果语义）：**全新未绑**工作流走合流
    // session_choice=new 一路到 proof——退 S0 后，prepare 走 chain_binds_per_task=true **不空转 needs_binding**、产
    // prepared·thread 延迟；链**每任务**经现成 relay 建一条任务命名新会话（真跑 codex 初始化）→ 绑任务节点 →
    // resume 它跑出 proof（**无 S0 孤儿**·无预绑）。核实物：绑定记录=真新 thread、该 thread 在 codex 侧真存在
    //（实时 sqlite 查得到·不在本测试静态索引里）、proof 含本次 token、`.codex` 凭据没碰（auth mtime）。多任务
    // N 条互异会话由离线测 births_binds_and_advances(creator.calls==2·2 互异 target_session_id) 已证·此处聚焦真
    // codex 端到端单任务。显式 `cargo test --lib confirm_and_start_new_session_real_run -- --ignored --nocapture`。
    // flake：真 codex 偶发早退 → retry（记忆 real-codex-run-flaky·核实物）。
    #[test]
    #[ignore = "C1: confluence session_choice=new (unbound workflow, no needs_binding stall) births a real per-task codex session in-chain, binds it, resumes to proof — no S0 orphan (user present, test project)"]
    fn confirm_and_start_new_session_real_run() {
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let proof_token = unix_timestamp_string();
        let proof = format!("{test_root}/jiaoban-plan-a-proof.txt");
        let _ = fs::remove_file(&proof);
        // `.codex` 凭据死线：记录 auth.json mtime，跑完必须没变。
        let auth_path = std::path::PathBuf::from(std::env::var("HOME").expect("HOME"))
            .join(".codex")
            .join("auth.json");
        let auth_mtime_before = fs::metadata(&auth_path)
            .and_then(|meta| meta.modified())
            .expect("auth.json mtime before");
        let dir = test_temp_dir("plan-a-new-real");
        let path = dir.join("workflow-state.v0.json");
        // 静态索引只带占位 thread——真新会话必须靠「实时 sqlite 回退」被绑定/被找到（bind 路A）。
        let index = fixture_dispatch_index(test_root, "thread-placeholder-not-used");
        let readback_db_path = codex_db::default_state_db_path();
        bootstrap_project_workflow_at(&path, &fixture_project(test_root)).expect("workflow");
        let mut cproposal = consult_proposal_fixture(Some(ConsultationExecutionScope {
            target_files: vec!["jiaoban-plan-a-proof.txt".to_string()],
            ..Default::default()
        }));
        cproposal.user_goal =
            format!("在项目根建 jiaoban-plan-a-proof.txt，写一行：plan-a ok {proof_token}");
        cproposal.goal_summary = format!(
            "在当前项目根目录创建文件 jiaoban-plan-a-proof.txt，只写入一行：plan-a ok {proof_token}"
        );
        cproposal.next_steps = vec![format!(
            "创建 jiaoban-plan-a-proof.txt，写入一行内容：plan-a ok {proof_token}"
        )];
        let c1_input =
            map_consultation_to_c1_input(&cproposal, test_root, "consultant").expect("map");
        let created = project_consultation_proposal_store::create_proposal(
            &path,
            &c1_input,
            1_765_300_000_000,
            "write-plan-a-real-proposal",
        )
        .expect("proposal");
        // 所批即所跑单任务（自包含 task_goal）——跳过真 LM 拆，聚焦本包新链路：出生→绑→resume。
        let scope = director_task_scope_from_proposal(&created.proposal, "codex-dev");
        let task = ProjectDirectorPlannedTask {
            planned_task_id: format!("planned-task:{}:1", created.proposal.workflow_id),
            title: "建 proof".to_string(),
            task_goal: format!(
                "在当前项目根目录创建文件 jiaoban-plan-a-proof.txt，只写入一行：plan-a ok {proof_token}。不改其它任何文件。"
            ),
            scope,
            depends_on: vec![],
            acceptance_criteria: vec![format!("jiaoban-plan-a-proof.txt 存在且含 {proof_token}")],
            report_format: vec!["做了什么".to_string()],
            status: "planned".to_string(),
            guard_result: None,
            work_item_id: None,
            workflow_node_id: None,
            task_package_id: None,
            memory_packet_snapshot_id: None,
            prepared_dispatch_id: None,
            blocked_reasons: vec![],
        };
        let runner = codex_local_runner::RealWorkflowNodeCodexRunner;
        let request = ConfirmAndStartAuthorizedRunRequest {
            project_root: test_root.to_string(),
            proposal_id: created.proposal.proposal_id.clone(),
            session_choice: "new".to_string(),
            session_id: None,
            actor_id: Some("user-fixture".to_string()),
            max_nodes: Some(10),
            approved_planned_tasks: Some(vec![task]),
            preview_session_bindings: vec![],
        };
        let outcome = run_confirm_and_start_authorized_run_inner(
            &path,
            &index,
            &readback_db_path,
            &runner,
            &CliDirectorAgent::default(),
            &ManualRelayJiaobanNewSessionCreator,
            &request,
        )
        .expect("方案a 真跑应一气跑完（真建会话→绑→真 worker 链）");
        println!(
            "[PLAN_A] stage={} completed={:?} warnings={:?}",
            outcome.stage,
            outcome.chain_outcome.as_ref().map(|chain| chain.completed),
            outcome.warnings
        );
        assert_eq!(outcome.stage, "completed", "建→绑→链应全通：{outcome:?}");
        assert!(
            outcome
                .chain_outcome
                .as_ref()
                .map(|chain| chain.completed >= 1)
                .unwrap_or(false),
            "worker 应真跑完成"
        );
        // 绑定实物：C1 下链每任务把新会话绑到**任务节点**（先生后绑）→ state 里存在一条真新 thread 的绑定。
        let state = read_json_file(&path);
        let bound_thread = state["workflow_node_session_bindings"]
            .as_array()
            .and_then(|bindings| {
                bindings
                    .iter()
                    .find_map(|binding| optional_string_from(binding, "native_thread_id"))
            })
            .expect("应有节点会话绑定");
        assert_ne!(
            bound_thread, "thread-placeholder-not-used",
            "绑的必须是真新会话，不是静态索引占位"
        );
        // C1 每任务物化：新会话 thread 回填任务包 artifact 的 target_session_id（退 S0 后无「新建会话」总纲 notice·
        // 每任务会话由链侧建·此处核 target_session_id 与绑定一致即证 resume 的就是这条新会话）。
        let materialized_session = state["artifacts"]
            .as_array()
            .and_then(|artifacts| {
                artifacts
                    .iter()
                    .find_map(|artifact| optional_string_from(artifact, "target_session_id"))
            })
            .expect("应有任务包 target_session_id 物化");
        assert_eq!(
            materialized_session, bound_thread,
            "物化的 target_session_id 应与绑定的新会话一致：{materialized_session} vs {bound_thread}"
        );
        // 会话实物：真新 thread 在 codex 侧存在（静态索引没有它 → 只能是实时 sqlite 查到 = 真建出来的）。
        let session = find_index_thread_or_sqlite(&index, &bound_thread)
            .expect("新会话应真存在于 codex 侧（实时 sqlite）");
        println!(
            "[PLAN_A] birth thread={bound_thread} rollout_exists={}",
            session.rollout_exists
        );
        // proof 实物：worker 链 resume 的就是这条新会话、真跑出 proof。
        let proof_text = fs::read_to_string(&proof)
            .unwrap_or_else(|error| panic!("worker 应真建 proof {proof}：{error}"));
        assert!(
            proof_text.contains(&proof_token),
            "proof 应含本次 token {proof_token}，实际：{proof_text}"
        );
        // `.codex` 凭据死线：auth.json 没被碰。
        let auth_mtime_after = fs::metadata(&auth_path)
            .and_then(|meta| meta.modified())
            .expect("auth.json mtime after");
        assert_eq!(auth_mtime_before, auth_mtime_after, ".codex 凭据不许被碰");
        let _ = fs::remove_dir_all(dir);
    }

    // P1·§5 真跑（单独步·用户在场·固定测试项目·默认 #[ignore]）：测试项目造 active 授权（proof-goal）+ 绑真
    // Codex 会话 → **经编排一下** → 真主管 LM 拆 → prepare → worker 链真 codex 接连跑 → proof 建出。证「一个
    // 命令把授权后自动推进跑通」。显式 `cargo test --lib auto_advance_authorized_role_loop_real_run -- --ignored --nocapture`。
    // flake：真 codex 偶发早退 → retry（核 proof 实物）。无授权拦由 stub auto_advance_rejects_without_active_authorization 证。
    #[test]
    #[ignore = "P1 role-loop auto-advance: one command drives authorized auto-advance (real director LM + real worker codex chain) in the test project (user present)"]
    fn auto_advance_authorized_role_loop_real_run() {
        let timestamp_ms = 1_765_300_000_000;
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let real_session = "019ed9f7-c0c2-7213-b871-6d18959b7c24";
        let proof_token = unix_timestamp_string();
        let proof_a = format!("{test_root}/auto-advance-proof-a.txt");
        let proof_b = format!("{test_root}/auto-advance-proof-b.txt");
        let _ = fs::remove_file(&proof_a);
        let _ = fs::remove_file(&proof_b);
        let dir = test_temp_dir("auto-advance-real");
        let path = dir.join("workflow-state.v0.json");
        let index = fixture_dispatch_index(test_root, real_session);
        bootstrap_project_workflow_at(&path, &fixture_project(test_root)).expect("workflow");
        // active 授权 proof-goal 方案（scope 圈测试项目根·绑 real_session）→ 确认 → 边界复核激活。
        let mut proposal_input = fixture_project_consultation_proposal_input(test_root);
        proposal_input.scope_draft.allowed_agent_ids = vec![real_session.to_string()];
        proposal_input.scope_draft.allowed_write_roots = vec![test_root.to_string()];
        proposal_input.scope_draft.allowed_read_roots = vec![test_root.to_string()];
        proposal_input.goal_summary = format!(
            "分两步在当前项目根目录建证据，第二步依赖第一步：① 创建 auto-advance-proof-a.txt，只写入一行：a {proof_token}；② 先读回 auto-advance-proof-a.txt 确认其内容含 {proof_token}，确认后再创建 auto-advance-proof-b.txt，只写入一行：verified {proof_token}。"
        );
        proposal_input.proposed_steps = vec![
            format!("创建 auto-advance-proof-a.txt，写入一行：a {proof_token}"),
            format!(
                "读回 auto-advance-proof-a.txt 核验含 {proof_token}，再创建 auto-advance-proof-b.txt 写入一行：verified {proof_token}（本步依赖上一步）"
            ),
        ];
        let created = project_consultation_proposal_store::create_proposal(
            &path,
            &proposal_input,
            timestamp_ms,
            "write-auto-advance-proposal",
        )
        .expect("proposal");
        let confirmed = project_consultation_proposal_store::record_decision(
            &path,
            &RecordProjectConsultationProposalDecisionInput {
                project_root: test_root.to_string(),
                proposal_id: created.proposal.proposal_id.clone(),
                actor_id: "user-fixture".to_string(),
                decision: ProjectConsultationProposalDecisionKind::Confirm,
                summary: "用户确认 P1 自动推进真跑方案。".to_string(),
                expected_proposal_store_revision: Some(created.store_revision),
                expected_plan_authorization_store_revision: None,
            },
            timestamp_ms + 1,
            "write-auto-advance-confirm",
            "write-auto-advance-auth",
            "write-auto-advance-auth-user",
        )
        .expect("confirm");
        let authorization = confirmed.plan_authorization.expect("authorization");
        let revision = confirmed
            .plan_authorization_store_revision
            .expect("revision");
        plan_authorization_store::record_global_boundary_review_with_proposal(
            &path,
            &fixture_global_boundary_review_input(
                test_root,
                &confirmed.proposal.proposal_id,
                &authorization.authorization_id,
                revision,
            ),
            timestamp_ms + 2,
            "write-auto-advance-boundary",
        )
        .expect("boundary review activate");
        // 绑真 Codex 会话到 codex-dev 节点（自动推进要派 worker 真跑）。
        let workflow_id = default_workflow_id(test_root);
        let node_id = format!("{workflow_id}:node:codex-dev");
        bind_workflow_node_codex_session_for_index_at(
            &path,
            &index,
            &fixture_node_session_bind_request(test_root, &node_id, None, real_session),
        )
        .expect("bind");
        // 一个命令：授权后自动推进（真主管 LM 拆 + 真 worker codex 链）。
        let runner = codex_local_runner::RealWorkflowNodeCodexRunner;
        let readback_db_path = codex_db::default_state_db_path();
        let director = CliDirectorAgent::default();
        let outcome = run_auto_advance_authorized_role_loop(
            &path,
            &index,
            &readback_db_path,
            &runner,
            &director,
            test_root,
            &workflow_id,
            "user-fixture",
            50,
            None,
        )
        .expect("授权后自动推进应跑通");
        println!(
            "[P1_AUTO] stage={} planned={} prepared={} completed={:?} stop={:?}",
            outcome.stage,
            outcome.planned_task_count,
            outcome.prepared_count,
            outcome.chain_outcome.as_ref().map(|c| c.completed),
            outcome.stop_reason
        );
        assert_eq!(
            outcome.stage, "completed",
            "应一路推进并完整跑完：{outcome:?}"
        );
        let chain = outcome.chain_outcome.expect("completed 应带 chain_outcome");
        assert!(
            chain.completed >= 1,
            "应 ≥1 worker 真跑完成：{}",
            chain.completed
        );
        // proof 实物：proof_a 建出含本次 token（worker 真跑了·内容对）。
        let a = fs::read_to_string(&proof_a)
            .unwrap_or_else(|e| panic!("worker 应真建 proof_a {proof_a}：{e}"));
        assert!(
            a.contains(&proof_token),
            "proof_a 应含本次 token {proof_token}，实际：{a}"
        );
        println!("[P1_AUTO] proof_a={a:?}");
        let _ = fs::remove_dir_all(dir);
    }

    // 交办地基·刀1 §4 真跑（单独步·#[ignore]·测试项目·真 codex）：Pending proof-goal 方案 + 点[允许并开始] + 绑
    // 现有会话 → **合流一个命令**（确认→复核→授权→绑→自动推进：真主管 LM 拆 + 真 worker 链）→ proof 落测试项目。
    // 证「点一下允许 → 测试项目里自动跑完出 proof」。flake→retry（记忆 real-codex-run-flaky·核 proof 实物）。
    #[test]
    #[ignore = "交办地基·刀1: one [允许并开始] command runs confirm→boundary→auth→bind→auto-advance (real LM + real worker) to proof (user present)"]
    fn confirm_and_start_authorized_run_real_run() {
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let real_session = "019ed9f7-c0c2-7213-b871-6d18959b7c24";
        let proof_token = unix_timestamp_string();
        let proof = format!("{test_root}/jiaoban-proof.txt");
        let _ = fs::remove_file(&proof);
        let dir = test_temp_dir("jiaoban-real");
        let path = dir.join("workflow-state.v0.json");
        let index = fixture_dispatch_index(test_root, real_session);
        let readback_db_path = codex_db::default_state_db_path();
        bootstrap_project_workflow_at(&path, &fixture_project(test_root)).expect("workflow");
        // 造 Pending proof-goal 方案（execution_scope Some·要改→档位 写范围；goal 让主管 LM 拆出建 proof 的任务）。
        let mut cproposal = consult_proposal_fixture(Some(ConsultationExecutionScope {
            target_files: vec!["jiaoban-proof.txt".to_string()],
            ..Default::default()
        }));
        cproposal.user_goal =
            format!("在项目根建 jiaoban-proof.txt，写一行：jiaoban ok {proof_token}");
        cproposal.goal_summary = format!(
            "在当前项目根目录创建文件 jiaoban-proof.txt，只写入一行：jiaoban ok {proof_token}"
        );
        cproposal.next_steps = vec![format!(
            "创建 jiaoban-proof.txt，写入一行内容：jiaoban ok {proof_token}"
        )];
        let c1_input =
            map_consultation_to_c1_input(&cproposal, test_root, "consultant").expect("map");
        let created = project_consultation_proposal_store::create_proposal(
            &path,
            &c1_input,
            1_765_300_000_000,
            "write-jiaoban-real-proposal",
        )
        .expect("proposal");
        // 合流：用户点[允许并开始] + 绑现有真会话。
        let runner = codex_local_runner::RealWorkflowNodeCodexRunner;
        let director = CliDirectorAgent::default();
        let request = ConfirmAndStartAuthorizedRunRequest {
            project_root: test_root.to_string(),
            proposal_id: created.proposal.proposal_id.clone(),
            session_choice: "existing".to_string(),
            session_id: Some(real_session.to_string()),
            actor_id: Some("user-fixture".to_string()),
            max_nodes: Some(50),
            approved_planned_tasks: None,
            preview_session_bindings: vec![],
        };
        let outcome = run_confirm_and_start_authorized_run_inner(
            &path,
            &index,
            &readback_db_path,
            &runner,
            &director,
            &ManualRelayJiaobanNewSessionCreator,
            &request,
        )
        .expect("合流一个命令应一气跑完");
        println!(
            "[JIAOBAN] stage={} prepared={} completed={:?} stop={:?}",
            outcome.stage,
            outcome.prepared_count,
            outcome.chain_outcome.as_ref().map(|c| c.completed),
            outcome.stop_reason
        );
        assert_eq!(
            outcome.stage, "completed",
            "点一下[允许并开始]→一气跑完：{outcome:?}"
        );
        let content = fs::read_to_string(&proof)
            .unwrap_or_else(|e| panic!("worker 应真建 proof {proof}：{e}"));
        assert!(
            content.contains(&proof_token),
            "proof 应含本次 token {proof_token}，实际：{content}"
        );
        println!("[JIAOBAN] proof={content:?}");
        let _ = fs::remove_dir_all(dir);
    }

    // ===== 交办·刀2 后端（预拆 / 所批即所跑 / 任务级节点边 / 拆步 retry）·stub =====

    // 造一个最小合规 planned task（scope 落在 fixture 授权范围内·codex-dev·{root}/src 可写）。
    fn jiaoban_test_planned_task(
        workflow_id: &str,
        id: usize,
        title: &str,
        deps: Vec<String>,
    ) -> ProjectDirectorPlannedTask {
        let root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        ProjectDirectorPlannedTask {
            planned_task_id: format!("planned-task:{workflow_id}:{id}"),
            title: title.to_string(),
            task_goal: format!("自包含指令：{title}"),
            scope: ProjectDirectorTaskScope {
                project_id: project_id(root),
                workflow_id: workflow_id.to_string(),
                target_role: "codex-dev".to_string(),
                task_package_kind: "task_package".to_string(),
                allowed_read_scope: vec![root.to_string()],
                allowed_write_scope: vec![format!("{root}/src")],
                available_skills: vec![],
                available_knowledge_refs: vec![],
                callable_tool_capabilities: vec!["read_file".to_string()],
                required_checks: vec![],
                stop_conditions: vec![],
                timeout_policy: None,
                failure_policy: None,
                forbidden_actions: vec![],
                model_id: None,
            },
            depends_on: deps,
            acceptance_criteria: vec!["ok".to_string()],
            report_format: vec!["done".to_string()],
            status: "planned".to_string(),
            guard_result: None,
            work_item_id: None,
            workflow_node_id: None,
            task_package_id: None,
            memory_packet_snapshot_id: None,
            prepared_dispatch_id: None,
            blocked_reasons: vec![],
        }
    }

    // 目录内所有文件的 (名, 字节) 快照（排序）——比对前后证「零写盘」。
    fn jiaoban_dir_snapshot(dir: &Path) -> Vec<(String, Vec<u8>)> {
        let mut out: Vec<(String, Vec<u8>)> = fs::read_dir(dir)
            .expect("read dir")
            .filter_map(|entry| {
                let p = entry.ok()?.path();
                if p.is_file() {
                    Some((
                        p.file_name()?.to_string_lossy().to_string(),
                        fs::read(&p).ok()?,
                    ))
                } else {
                    None
                }
            })
            .collect();
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out
    }

    // 计数 director：前 err_until 次返回 err_msg，之后返回 tasks。验 2.4 retry 判据/次数。
    struct CountingDirector {
        err_until: usize,
        err_msg: String,
        tasks: Vec<ProjectDirectorPlannedTask>,
        calls: std::cell::RefCell<usize>,
    }
    impl DirectorAgent for CountingDirector {
        fn plan(
            &self,
            _ctx: &ProjectContext,
            _proposal: &ProjectConsultationProposal,
        ) -> Result<Vec<ProjectDirectorPlannedTask>, String> {
            let mut calls = self.calls.borrow_mut();
            *calls += 1;
            if *calls <= self.err_until {
                Err(self.err_msg.clone())
            } else {
                Ok(self.tasks.clone())
            }
        }
    }

    // 炸弹 director：plan 被调即 panic——验「带 approved 图时 director.plan 绝不被调」。
    struct BombDirector;
    impl DirectorAgent for BombDirector {
        fn plan(
            &self,
            _ctx: &ProjectContext,
            _proposal: &ProjectConsultationProposal,
        ) -> Result<Vec<ProjectDirectorPlannedTask>, String> {
            panic!("所批即所跑：带 approved 图时不该调用 director.plan（应跳过重拆）");
        }
    }

    // 2.1·预拆：PendingUserConfirmation 方案 → 预拆出含依赖的任务图 + **零写盘**。
    #[test]
    fn jiaoban_preview_pending_proposal_plans_with_deps_zero_writes() {
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let dir = test_temp_dir("jiaoban-preview");
        let path = dir.join("workflow-state.v0.json");
        bootstrap_project_workflow_at(&path, &fixture_project(test_root)).expect("workflow");
        let input = fixture_project_consultation_proposal_input(test_root);
        let created = project_consultation_proposal_store::create_proposal(
            &path,
            &input,
            1_765_300_000_000,
            "write-preview-proposal",
        )
        .expect("proposal");
        assert_eq!(
            created.proposal.status,
            ProjectConsultationProposalStatus::PendingUserConfirmation,
            "预拆针对的是待确认方案"
        );
        let before = jiaoban_dir_snapshot(&dir);
        let request = PreviewPendingProposalDirectorPlanRequest {
            project_root: test_root.to_string(),
            proposal_id: created.proposal.proposal_id.clone(),
        };
        let outcome =
            run_preview_pending_proposal_director_plan_inner(&path, &StubDirector, &request)
                .expect("pending 方案应能预拆");
        assert!(
            !outcome.planned_tasks.is_empty(),
            "预拆应产出任务：{outcome:?}"
        );
        assert!(
            outcome
                .planned_tasks
                .iter()
                .any(|task| !task.depends_on.is_empty()),
            "预拆图应含依赖关系（StubDirector task2 依赖 task1）"
        );
        let after = jiaoban_dir_snapshot(&dir);
        assert_eq!(before, after, "预拆必须零写盘（目录文件字节前后一致）");
        let _ = fs::remove_dir_all(dir);
    }

    // 2.1·预拆幂等无状态：已确认方案也能预览（不校 Pending-only·预览无副作用）。
    #[test]
    fn jiaoban_preview_works_on_confirmed_proposal() {
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let dir = test_temp_dir("jiaoban-preview-confirmed");
        let path = dir.join("workflow-state.v0.json");
        bootstrap_project_workflow_at(&path, &fixture_project(test_root)).expect("workflow");
        let (proposal, _auth, _rev) = create_active_project_director_authorization_fixture(
            &path,
            test_root,
            "thread-preview-confirmed",
            1_765_300_000_000,
        );
        let request = PreviewPendingProposalDirectorPlanRequest {
            project_root: test_root.to_string(),
            proposal_id: proposal.proposal_id.clone(),
        };
        let outcome =
            run_preview_pending_proposal_director_plan_inner(&path, &StubDirector, &request)
                .expect("已确认方案也应可预览（幂等·无状态）");
        assert!(!outcome.planned_tasks.is_empty(), "已确认方案预览也出图");
        let _ = fs::remove_dir_all(dir);
    }

    // 2.4·拆步 retry：consult 偶发早退（consult_last_message_read_failed）→ 原地重试一次成功 + 标记 retried。
    #[test]
    fn jiaoban_director_plan_retries_once_on_consult_early_exit() {
        let director = CountingDirector {
            err_until: 1,
            err_msg: "consult_last_message_read_failed:No such file (os error 2)".to_string(),
            tasks: vec![jiaoban_test_planned_task("wf", 1, "t", vec![])],
            calls: std::cell::RefCell::new(0),
        };
        let ctx = load_project_context(WORKFLOW_ENGINE_TEST_PROJECT_ROOT).expect("ctx");
        let (proposal, pdir) = s3_director_fixture_proposal("jiaoban-retry-ok");
        let (tasks, retried) = director_plan_with_retry(&director, &ctx, &proposal, false)
            .expect("偶发早退应重试一次后成功");
        assert!(retried, "应标记发生过重试");
        assert_eq!(tasks.len(), 1, "重试后拿到任务");
        assert_eq!(
            *director.calls.borrow(),
            2,
            "恰好调 2 次（首次 + 重试一次）"
        );
        let _ = fs::remove_dir_all(pdir);
    }

    // 2.4·不循环：两次都早退 → 报错停（只重试一次·不无限重试）。
    #[test]
    fn jiaoban_director_plan_stops_after_second_failure() {
        let director = CountingDirector {
            err_until: 2,
            err_msg: "consult_last_message_read_failed:still gone".to_string(),
            tasks: vec![jiaoban_test_planned_task("wf", 1, "t", vec![])],
            calls: std::cell::RefCell::new(0),
        };
        let ctx = load_project_context(WORKFLOW_ENGINE_TEST_PROJECT_ROOT).expect("ctx");
        let (proposal, pdir) = s3_director_fixture_proposal("jiaoban-retry-stop");
        let result = director_plan_with_retry(&director, &ctx, &proposal, false);
        assert!(result.is_err(), "连续两次早退应报错：{result:?}");
        assert_eq!(*director.calls.borrow(), 2, "只重试一次·不循环（共 2 次）");
        let _ = fs::remove_dir_all(pdir);
    }

    // 2.4·解析类不 retry：json 解析失败不是偶发早退 → 立即报错·不重试。
    #[test]
    fn jiaoban_director_plan_no_retry_on_parse_error() {
        let director = CountingDirector {
            err_until: 1,
            err_msg: "主管 plan json 解析失败:expected value".to_string(),
            tasks: vec![jiaoban_test_planned_task("wf", 1, "t", vec![])],
            calls: std::cell::RefCell::new(0),
        };
        let ctx = load_project_context(WORKFLOW_ENGINE_TEST_PROJECT_ROOT).expect("ctx");
        let (proposal, pdir) = s3_director_fixture_proposal("jiaoban-retry-parse");
        let result = director_plan_with_retry(&director, &ctx, &proposal, false);
        assert!(result.is_err(), "解析错应直接报错");
        assert_eq!(*director.calls.borrow(), 1, "解析类不 retry（只调 1 次）");
        let _ = fs::remove_dir_all(pdir);
    }

    // fix8·供给类不 retry（consult 侧）：codex_provider_unavailable（额度/订阅/登录）不是抽风 →
    // 立即报错·不重试（否则白等一分钟）。
    #[test]
    fn jiaoban_director_plan_no_retry_on_provider_unavailable() {
        let director = CountingDirector {
            err_until: 1,
            err_msg: "codex_provider_unavailable:codex 供给不可用（403 订阅/额度/登录类）"
                .to_string(),
            tasks: vec![jiaoban_test_planned_task("wf", 1, "t", vec![])],
            calls: std::cell::RefCell::new(0),
        };
        let ctx = load_project_context(WORKFLOW_ENGINE_TEST_PROJECT_ROOT).expect("ctx");
        let (proposal, pdir) = s3_director_fixture_proposal("jiaoban-retry-provider");
        let result = director_plan_with_retry(&director, &ctx, &proposal, false);
        assert!(result.is_err(), "供给类应直接报错（不重试）");
        assert_eq!(
            *director.calls.borrow(),
            1,
            "供给类不 retry（只调 1 次·不白等）"
        );
        let _ = fs::remove_dir_all(pdir);
    }

    // fix8·worker resume 侧：stderr 命中供给特征 → warnings 带 codex_provider_unavailable
    // （供 is_tier1_early_exit 排除重试 + UI 上脸）；普通错不误标、成败判定不变。
    #[test]
    fn fix8_classify_codex_resume_failure_flags_provider() {
        let no_instruction: Option<UserReviewedInstructionInput> = None;
        let hit = classify_codex_resume_failure(
            1,
            false,
            &no_instruction,
            "Reconnecting... 5/5 ERROR: unexpected status 403 Forbidden: SUBSCRIPTION_NOT_FOUND",
        );
        assert!(
            hit.iter().any(|w| w.contains("codex_provider_unavailable")),
            "供给类 stderr 应上 provider 标签：{hit:?}"
        );
        let miss =
            classify_codex_resume_failure(1, false, &no_instruction, "some random compile error");
        assert!(
            !miss
                .iter()
                .any(|w| w.contains("codex_provider_unavailable")),
            "普通错不该误标 provider：{miss:?}"
        );
        assert!(
            miss.iter().any(|w| w == "codex_resume_exit_nonzero"),
            "普通非 0 退出仍标 exit_nonzero（成败判定不变）：{miss:?}"
        );
    }

    // 2.3·任务级节点 + 依赖边落画布（纯加法）：一任务一节点、id 带后缀不撞保留节点、position 错开、depends_on 建边、
    // 老 role/保留节点原样、重跑幂等。
    #[test]
    fn jiaoban_prepare_writes_task_level_nodes_and_dep_edges() {
        let timestamp_ms = 1_765_300_000_000;
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let thread_id = "thread-jiaoban-nodes";
        let dir = test_temp_dir("jiaoban-task-nodes");
        let path = dir.join("workflow-state.v0.json");
        let index = fixture_dispatch_index(test_root, thread_id);
        bootstrap_project_workflow_at(&path, &fixture_project(test_root)).expect("workflow");
        let (proposal, authorization, revision) =
            create_active_project_director_authorization_fixture(
                &path,
                test_root,
                thread_id,
                timestamp_ms,
            );
        let workflow_id = default_workflow_id(test_root);
        bind_workflow_node_codex_session_for_index_at(
            &path,
            &index,
            &fixture_node_session_bind_request(
                test_root,
                &format!("{workflow_id}:node:codex-dev"),
                None,
                thread_id,
            ),
        )
        .expect("bind");
        let ctx = load_project_context(test_root).expect("ctx");
        let planned = StubDirector.plan(&ctx, &proposal).expect("plan"); // 搭骨架 / 接业务(dep 搭骨架)
        prepare_authorized_auto_dispatch_for_index_at(
            &path,
            &index,
            &fixture_project_director_prepare_input(
                test_root,
                &proposal.proposal_id,
                &authorization.authorization_id,
                revision,
                planned.clone(),
            ),
        )
        .expect("prepare");
        let state = read_workflow_state_value(&path).expect("state");
        let task_nodes: Vec<&Value> = state["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .filter(|node| {
                optional_string_from(node, "node_type").as_deref() == Some("project_director_task")
            })
            .collect();
        assert_eq!(task_nodes.len(), 2, "两个任务 → 两个任务级节点");
        let reserved = format!("{workflow_id}:node:task");
        for node in &task_nodes {
            let node_id = optional_string_from(node, "node_id").expect("node_id");
            assert!(
                node_id.starts_with(&format!("{workflow_id}:node:task:")),
                "任务级 id 带 :task: 后缀：{node_id}"
            );
            assert_ne!(node_id, reserved, "绝不等于 bootstrap 保留节点");
        }
        assert_ne!(
            task_nodes[0]["position"], task_nodes[1]["position"],
            "position 应按 index 错开、不叠"
        );
        let dep_edges: Vec<&Value> = state["edges"]
            .as_array()
            .expect("edges")
            .iter()
            .filter(|edge| optional_string_from(edge, "edge_type").as_deref() == Some("depends_on"))
            .collect();
        assert_eq!(dep_edges.len(), 1, "接业务→搭骨架 一条 depends_on 边");
        assert!(
            node_exists(&state, &workflow_id, &reserved),
            "bootstrap 保留节点 {reserved} 不受扰"
        );
        assert!(
            node_exists(
                &state,
                &workflow_id,
                &format!("{workflow_id}:node:codex-dev")
            ),
            "老 role 节点原样"
        );
        // 幂等：重跑 prepare 不翻倍
        prepare_authorized_auto_dispatch_for_index_at(
            &path,
            &index,
            &fixture_project_director_prepare_input(
                test_root,
                &proposal.proposal_id,
                &authorization.authorization_id,
                revision,
                planned,
            ),
        )
        .expect("prepare rerun");
        let state2 = read_workflow_state_value(&path).expect("state2");
        let count2 = state2["nodes"]
            .as_array()
            .expect("nodes2")
            .iter()
            .filter(|node| {
                optional_string_from(node, "node_type").as_deref() == Some("project_director_task")
            })
            .count();
        assert_eq!(count2, 2, "重跑 prepare 幂等·任务级节点不重复建");
        let _ = fs::remove_dir_all(dir);
    }

    // 2.3·悬空依赖记 warning 不建边（依赖指向未物化的 title）。
    #[test]
    fn jiaoban_prepare_dangling_dep_warns_no_edge() {
        let timestamp_ms = 1_765_300_000_000;
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let thread_id = "thread-jiaoban-dangling";
        let dir = test_temp_dir("jiaoban-dangling");
        let path = dir.join("workflow-state.v0.json");
        let index = fixture_dispatch_index(test_root, thread_id);
        bootstrap_project_workflow_at(&path, &fixture_project(test_root)).expect("workflow");
        let (proposal, authorization, revision) =
            create_active_project_director_authorization_fixture(
                &path,
                test_root,
                thread_id,
                timestamp_ms,
            );
        let workflow_id = default_workflow_id(test_root);
        bind_workflow_node_codex_session_for_index_at(
            &path,
            &index,
            &fixture_node_session_bind_request(
                test_root,
                &format!("{workflow_id}:node:codex-dev"),
                None,
                thread_id,
            ),
        )
        .expect("bind");
        let ctx = load_project_context(test_root).expect("ctx");
        let mut planned = StubDirector.plan(&ctx, &proposal).expect("plan");
        planned[0].depends_on = vec!["幽灵前置".to_string()]; // 搭骨架 依赖一个不存在的 title
        let prepared = prepare_authorized_auto_dispatch_for_index_at(
            &path,
            &index,
            &fixture_project_director_prepare_input(
                test_root,
                &proposal.proposal_id,
                &authorization.authorization_id,
                revision,
                planned,
            ),
        )
        .expect("prepare");
        assert!(
            prepared.warnings.iter().any(|w| w.contains("幽灵前置")),
            "悬空依赖应记 warning：{:?}",
            prepared.warnings
        );
        let state = read_workflow_state_value(&path).expect("state");
        assert!(
            !state["edges"]
                .as_array()
                .expect("edges")
                .iter()
                .any(|edge| optional_string_from(edge, "source_ref")
                    .as_deref()
                    .is_some_and(|r| r.contains("幽灵"))),
            "悬空依赖不建边"
        );
        let _ = fs::remove_dir_all(dir);
    }

    // 2.2·所批即所跑：带 approved 图 → 跳过 director.plan（BombDirector 不炸）+ 跑的就是那份图。
    #[test]
    fn jiaoban_auto_advance_uses_approved_graph_skips_plan() {
        let timestamp_ms = 1_765_300_000_000;
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let thread_id = "thread-jiaoban-approved";
        let dir = test_temp_dir("jiaoban-approved");
        let path = dir.join("workflow-state.v0.json");
        let index_path = dir.join("codex-index.json");
        let index = fixture_dispatch_index(test_root, thread_id);
        bootstrap_project_workflow_at(&path, &fixture_project(test_root)).expect("workflow");
        let (proposal, _auth, _rev) = create_active_project_director_authorization_fixture(
            &path,
            test_root,
            thread_id,
            timestamp_ms,
        );
        let workflow_id = default_workflow_id(test_root);
        bind_workflow_node_codex_session_for_index_at(
            &path,
            &index,
            &fixture_node_session_bind_request(
                test_root,
                &format!("{workflow_id}:node:codex-dev"),
                None,
                thread_id,
            ),
        )
        .expect("bind");
        let ctx = load_project_context(test_root).expect("ctx");
        let approved = StubDirector.plan(&ctx, &proposal).expect("approved graph"); // 用户批过的图
        let runner = PermissiveExperimentRunner {
            stats: CodexDispatchReadbackStats {
                transcript_event_count: 3,
                transcript_target_hits: 1,
            },
        };
        // BombDirector：若跳过逻辑失效而调 plan → panic 炸测试。
        let outcome = run_auto_advance_authorized_role_loop(
            &path,
            &index,
            &index_path,
            &runner,
            &BombDirector,
            test_root,
            &workflow_id,
            "user",
            10,
            Some(&approved),
        )
        .expect("所批即所跑应一路跑到链");
        assert_eq!(
            outcome.stage, "completed",
            "带 approved 图应完整跑完链：{outcome:?}"
        );
        let chain = outcome.chain_outcome.expect("completed 带 chain");
        let ran: std::collections::BTreeSet<String> =
            chain.steps.iter().map(|step| step.title.clone()).collect();
        let want: std::collections::BTreeSet<String> =
            approved.iter().map(|task| task.title.clone()).collect();
        assert_eq!(
            ran, want,
            "跑的就是 approved 那几个任务（task_goal 原样·不重拆）"
        );
        let _ = fs::remove_dir_all(dir);
    }

    // 2.2·一致性校验：approved 图的 workflow_id 与本次授权不一致 → 拒（防串工作流）。
    #[test]
    fn jiaoban_auto_advance_rejects_approved_graph_workflow_mismatch() {
        let timestamp_ms = 1_765_300_000_000;
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let thread_id = "thread-jiaoban-mismatch";
        let dir = test_temp_dir("jiaoban-mismatch");
        let path = dir.join("workflow-state.v0.json");
        let index_path = dir.join("codex-index.json");
        let index = fixture_dispatch_index(test_root, thread_id);
        bootstrap_project_workflow_at(&path, &fixture_project(test_root)).expect("workflow");
        create_active_project_director_authorization_fixture(
            &path,
            test_root,
            thread_id,
            timestamp_ms,
        );
        let workflow_id = default_workflow_id(test_root);
        bind_workflow_node_codex_session_for_index_at(
            &path,
            &index,
            &fixture_node_session_bind_request(
                test_root,
                &format!("{workflow_id}:node:codex-dev"),
                None,
                thread_id,
            ),
        )
        .expect("bind");
        // 串了工作流的 approved 图（scope.workflow_id ≠ 本授权）。
        let bad = vec![jiaoban_test_planned_task(
            "some-other-workflow",
            1,
            "越权",
            vec![],
        )];
        let runner = PermissiveExperimentRunner {
            stats: CodexDispatchReadbackStats {
                transcript_event_count: 3,
                transcript_target_hits: 1,
            },
        };
        let result = run_auto_advance_authorized_role_loop(
            &path,
            &index,
            &index_path,
            &runner,
            &BombDirector,
            test_root,
            &workflow_id,
            "user",
            10,
            Some(&bad),
        );
        assert!(
            result.is_err_and(|error| error.contains("不一致")),
            "workflow 不匹配的已批图应被拒"
        );
        let _ = fs::remove_dir_all(dir);
    }

    // 2.5·咨询标记向后兼容：带 suggest_workflow 解析出真值；老样本缺字段 → false。
    #[test]
    fn jiaoban_parse_consultation_suggest_workflow_backward_compat() {
        let with = "```json\n{\"goal_summary\":\"g\",\"reasoning\":[\"r\"],\"suggest_workflow\":true}\n```";
        let parsed = parse_consultation_proposal(with).expect("parse with field");
        assert!(parsed.suggest_workflow, "显式 true 应解析出 true");
        let without = "```json\n{\"goal_summary\":\"g\",\"reasoning\":[\"r\"]}\n```";
        let parsed_old = parse_consultation_proposal(without).expect("parse old sample");
        assert!(
            !parsed_old.suggest_workflow,
            "老样本缺字段 → false（向后兼容）"
        );
    }

    // 交办·刀2 §4 真跑（单独步·#[ignore]·测试项目·真 codex）：批前**预拆**(真主管 LM 出图·零写盘) →
    // **合流带图**(所批即所跑·approved_planned_tasks·跳过重拆·真 worker 链) → proof 落测试项目 + 画布读模型里
    // 任务级节点在、链完成后 state 刷 completed。证「预拆看的那份 = 真跑那份 + 图真落画布」。
    // flake→retry（记忆 real-codex-run-flaky·核 proof/state 实物·预拆步已内建 2.4 自动重试一次）。
    #[test]
    #[ignore = "交办·刀2 §4: preview (real LM graph, zero-write) → confirm_and_start with that approved graph (real worker) → proof + task-level nodes on canvas (user present)"]
    fn jiaoban_preview_then_confirm_with_graph_real_run() {
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let real_session = "019ed9f7-c0c2-7213-b871-6d18959b7c24";
        let proof_token = unix_timestamp_string();
        let proof = format!("{test_root}/jiaoban-slice2-proof.txt");
        let _ = fs::remove_file(&proof);
        let dir = test_temp_dir("jiaoban-slice2-real");
        let path = dir.join("workflow-state.v0.json");
        let index = fixture_dispatch_index(test_root, real_session);
        let readback_db_path = codex_db::default_state_db_path();
        bootstrap_project_workflow_at(&path, &fixture_project(test_root)).expect("workflow");
        let mut cproposal = consult_proposal_fixture(Some(ConsultationExecutionScope {
            target_files: vec!["jiaoban-slice2-proof.txt".to_string()],
            ..Default::default()
        }));
        cproposal.user_goal =
            format!("在项目根建 jiaoban-slice2-proof.txt，写一行：slice2 ok {proof_token}");
        cproposal.goal_summary = format!(
            "在当前项目根目录创建文件 jiaoban-slice2-proof.txt，只写入一行：slice2 ok {proof_token}"
        );
        cproposal.next_steps = vec![format!(
            "创建 jiaoban-slice2-proof.txt，写入一行内容：slice2 ok {proof_token}"
        )];
        let c1_input =
            map_consultation_to_c1_input(&cproposal, test_root, "consultant").expect("map");
        let created = project_consultation_proposal_store::create_proposal(
            &path,
            &c1_input,
            1_765_300_000_000,
            "write-slice2-real-proposal",
        )
        .expect("proposal");
        let director = CliDirectorAgent::default();
        // 1) 批前预拆（真主管 LM 出图·零写盘·2.4 偶发早退自动重试一次）
        let preview = run_preview_pending_proposal_director_plan_inner(
            &path,
            &director,
            &PreviewPendingProposalDirectorPlanRequest {
                project_root: test_root.to_string(),
                proposal_id: created.proposal.proposal_id.clone(),
            },
        )
        .expect("批前预拆真 LM 出图");
        println!(
            "[JIAOBAN2] preview tasks={} warnings={:?}",
            preview.planned_tasks.len(),
            preview.warnings
        );
        assert!(!preview.planned_tasks.is_empty(), "预拆应出 ≥1 任务");
        // 2) 合流带图：所批即所跑（用预拆那份·跳过重拆·真 worker 链）
        let runner = codex_local_runner::RealWorkflowNodeCodexRunner;
        let request = ConfirmAndStartAuthorizedRunRequest {
            project_root: test_root.to_string(),
            proposal_id: created.proposal.proposal_id.clone(),
            session_choice: "existing".to_string(),
            session_id: Some(real_session.to_string()),
            actor_id: Some("user-fixture".to_string()),
            max_nodes: Some(50),
            approved_planned_tasks: Some(preview.planned_tasks.clone()),
            preview_session_bindings: vec![],
        };
        let outcome = run_confirm_and_start_authorized_run_inner(
            &path,
            &index,
            &readback_db_path,
            &runner,
            &director,
            &ManualRelayJiaobanNewSessionCreator,
            &request,
        )
        .expect("合流带图应一气跑完");
        println!(
            "[JIAOBAN2] stage={} prepared={} completed={:?}",
            outcome.stage,
            outcome.prepared_count,
            outcome.chain_outcome.as_ref().map(|c| c.completed)
        );
        assert_eq!(
            outcome.stage, "completed",
            "合流带图应完整跑完链：{outcome:?}"
        );
        // 3) proof 实物
        let content = fs::read_to_string(&proof)
            .unwrap_or_else(|e| panic!("worker 应真建 proof {proof}：{e}"));
        assert!(
            content.contains(&proof_token),
            "proof 应含 token {proof_token}，实际：{content}"
        );
        println!("[JIAOBAN2] proof={content:?}");
        // 4) 画布读模型：任务级节点一任务一个 + 链完成后至少一个 state 刷成 completed（2.3 尾）
        let state = read_workflow_state_value(&path).expect("state");
        let task_nodes: Vec<&Value> = state["nodes"]
            .as_array()
            .expect("nodes")
            .iter()
            .filter(|node| {
                optional_string_from(node, "node_type").as_deref() == Some("project_director_task")
            })
            .collect();
        assert_eq!(
            task_nodes.len(),
            preview.planned_tasks.len(),
            "任务级节点一任务一个（预拆图原样落画布）"
        );
        assert!(
            task_nodes
                .iter()
                .any(|node| optional_string_from(node, "state").as_deref() == Some("completed")),
            "链完成后至少一个任务级节点 state 刷成 completed（2.3 尾）"
        );
        println!(
            "[JIAOBAN2] task_nodes={} states={:?}",
            task_nodes.len(),
            task_nodes
                .iter()
                .map(|node| optional_string_from(node, "state"))
                .collect::<Vec<_>>()
        );
        let _ = fs::remove_dir_all(dir);
    }

    // ===== 交办 fix3 后端（角色钳位 / 失败留档 / 接续告知）·stub =====

    // 界外角色 director：吐 target_role="reviewer"（不在档位授权集）——验钳位。plan_preview 默认回落到 plan。
    struct ReviewerDirector;
    impl DirectorAgent for ReviewerDirector {
        fn plan(
            &self,
            _ctx: &ProjectContext,
            proposal: &ProjectConsultationProposal,
        ) -> Result<Vec<ProjectDirectorPlannedTask>, String> {
            let scope = director_task_scope_from_proposal(proposal, "reviewer");
            Ok(vec![ProjectDirectorPlannedTask {
                planned_task_id: format!("planned-task:{}:1", proposal.workflow_id),
                title: "审查任务".to_string(),
                task_goal: "自包含：审查一下".to_string(),
                scope,
                depends_on: vec![],
                acceptance_criteria: vec!["ok".to_string()],
                report_format: vec!["done".to_string()],
                status: "planned".to_string(),
                guard_result: None,
                work_item_id: None,
                workflow_node_id: None,
                task_package_id: None,
                memory_packet_snapshot_id: None,
                prepared_dispatch_id: None,
                blocked_reasons: vec![],
            }])
        }
    }

    fn jiaoban_task_with_role(
        workflow_id: &str,
        id: usize,
        title: &str,
        role: &str,
    ) -> ProjectDirectorPlannedTask {
        let mut task = jiaoban_test_planned_task(workflow_id, id, title, vec![]);
        task.scope.target_role = role.to_string();
        task
    }

    // 2.1·钳位核心：界外角色归一 codex-dev + 出警告；授权集内（codex-dev/project_director）原样不动。
    #[test]
    fn jiaoban_fix3_clamp_only_out_of_set_roles() {
        let allowed = vec!["codex-dev".to_string(), "project_director".to_string()];
        let mut tasks = vec![
            jiaoban_task_with_role("wf", 1, "a", "reviewer"),
            jiaoban_task_with_role("wf", 2, "b", "codex-dev"),
            jiaoban_task_with_role("wf", 3, "c", "project_director"),
        ];
        let warnings = clamp_planned_task_roles(&mut tasks, &allowed);
        assert_eq!(
            tasks[0].scope.target_role, "codex-dev",
            "界外 reviewer→codex-dev"
        );
        assert_eq!(tasks[1].scope.target_role, "codex-dev", "codex-dev 原样");
        assert_eq!(
            tasks[2].scope.target_role, "project_director",
            "project_director 在授权集·原样"
        );
        assert_eq!(warnings.len(), 1, "只 reviewer 一条警告");
        assert!(warnings[0].contains("reviewer") && warnings[0].contains("codex-dev"));
    }

    // 2.1·plan_preview 路：预拆给用户看的图必须已钳后（所见即所跑）+ 警告在。
    #[test]
    fn jiaoban_fix3_clamp_in_preview_path() {
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let dir = test_temp_dir("fix3-clamp-preview");
        let path = dir.join("workflow-state.v0.json");
        bootstrap_project_workflow_at(&path, &fixture_project(test_root)).expect("workflow");
        let input = fixture_project_consultation_proposal_input(test_root);
        let created = project_consultation_proposal_store::create_proposal(
            &path,
            &input,
            1_765_300_000_000,
            "write-fix3-preview",
        )
        .expect("proposal");
        let outcome = run_preview_pending_proposal_director_plan_inner(
            &path,
            &ReviewerDirector,
            &PreviewPendingProposalDirectorPlanRequest {
                project_root: test_root.to_string(),
                proposal_id: created.proposal.proposal_id.clone(),
            },
        )
        .expect("preview");
        assert!(
            outcome
                .planned_tasks
                .iter()
                .all(|task| task.scope.target_role == "codex-dev"),
            "预拆图角色应已钳成 codex-dev"
        );
        assert!(
            outcome.warnings.iter().any(|w| w.contains("reviewer")),
            "预拆应带钳位警告：{:?}",
            outcome.warnings
        );
        let _ = fs::remove_dir_all(dir);
    }

    // 2.1·plan 路（auto_advance）：界外角色钳成 codex-dev 后**能跑通**（而非撞 guard 变 blocked）+ outcome.warnings 带钳位。
    #[test]
    fn jiaoban_fix3_clamp_unblocks_in_auto_advance() {
        let (dir, index_path, index, workflow_id) = auto_advance_fixture("fix3-clamp-ran", true);
        let path = dir.join("workflow-state.v0.json");
        let runner = PermissiveExperimentRunner {
            stats: CodexDispatchReadbackStats {
                transcript_event_count: 3,
                transcript_target_hits: 1,
            },
        };
        let outcome = run_auto_advance_authorized_role_loop(
            &path,
            &index,
            &index_path,
            &runner,
            &ReviewerDirector,
            WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
            &workflow_id,
            "tester",
            10,
            None,
        )
        .expect("钳后应跑通");
        assert_eq!(
            outcome.stage, "completed",
            "界外角色钳成 codex-dev 后应 completed（而非 blocked）：{outcome:?}"
        );
        assert!(
            outcome
                .warnings
                .iter()
                .any(|w| w.contains("reviewer") && w.contains("codex-dev")),
            "outcome.warnings 应带钳位：{:?}",
            outcome.warnings
        );
        let _ = fs::remove_dir_all(dir);
    }

    // 2.1·辨析：所批即所跑（approved）路**不钳**——界外角色照 guard 兜底拦成 blocked（回传数据不静默改·安全不降）。
    #[test]
    fn jiaoban_fix3_approved_out_of_set_role_not_clamped_blocked() {
        let (dir, index_path, index, workflow_id) =
            auto_advance_fixture("fix3-approved-noclamp", true);
        let path = dir.join("workflow-state.v0.json");
        let approved = vec![jiaoban_task_with_role(&workflow_id, 1, "审查", "reviewer")];
        let runner = PermissiveExperimentRunner {
            stats: CodexDispatchReadbackStats {
                transcript_event_count: 3,
                transcript_target_hits: 1,
            },
        };
        let outcome = run_auto_advance_authorized_role_loop(
            &path,
            &index,
            &index_path,
            &runner,
            &BombDirector,
            WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
            &workflow_id,
            "tester",
            10,
            Some(&approved),
        )
        .expect("approved 路不炸 director");
        assert_eq!(
            outcome.stage, "blocked",
            "approved 图界外角色不钳、guard 兜底拦→blocked：{outcome:?}"
        );
        assert!(
            outcome.warnings.is_empty(),
            "approved 路不钳→无钳位警告：{:?}",
            outcome.warnings
        );
        let _ = fs::remove_dir_all(dir);
    }

    // 2.2·留档：拆步两次早退（retry 后仍败）→ 返回 Err（不吞错）+ 审计里有带人话的 stopped 事件。
    #[test]
    fn jiaoban_fix3_plan_failure_after_started_is_audited() {
        let (dir, index_path, index, workflow_id) = auto_advance_fixture("fix3-audit-err", true);
        let path = dir.join("workflow-state.v0.json");
        let director = CountingDirector {
            err_until: 2,
            err_msg: "consult_last_message_read_failed:gone".to_string(),
            tasks: vec![],
            calls: std::cell::RefCell::new(0),
        };
        let runner = PermissiveExperimentRunner {
            stats: CodexDispatchReadbackStats {
                transcript_event_count: 3,
                transcript_target_hits: 1,
            },
        };
        let result = run_auto_advance_authorized_role_loop(
            &path,
            &index,
            &index_path,
            &runner,
            &director,
            WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
            &workflow_id,
            "tester",
            10,
            None,
        );
        assert!(result.is_err(), "拆步连败（retry 后仍败）应返回 Err·不吞错");
        assert_eq!(*director.calls.borrow(), 2, "retry 一次·共 2 次");
        let state = read_json_file(&path);
        let has_err_stopped = state["audit_events"]
            .as_array()
            .expect("audit_events")
            .iter()
            .any(|event| {
                optional_string_from(event, "event_type").as_deref()
                    == Some("role_loop_auto_advance_stopped")
                    && optional_string_from(event, "reason")
                        .is_some_and(|reason| reason.contains("失败（已留档）"))
            });
        assert!(has_err_stopped, "确认后失败应写带人话的 stopped 审计");
        let _ = fs::remove_dir_all(dir);
    }

    // 2.2·确认前拒不多写：非 Pending 方案 → 合流 step1 拒（record_decision 之前）→ 不写 stopped 审计。
    #[test]
    fn jiaoban_fix3_confirm_pre_decision_reject_no_stopped_audit() {
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let dir = test_temp_dir("fix3-pre-decision");
        let path = dir.join("workflow-state.v0.json");
        let index = fixture_dispatch_index(test_root, "thread-fix3-pre");
        let readback_db_path = codex_db::default_state_db_path();
        bootstrap_project_workflow_at(&path, &fixture_project(test_root)).expect("workflow");
        // create_active_...fixture 把方案推到 UserConfirmed（非 PendingUserConfirmation）。
        let (proposal, _auth, _rev) = create_active_project_director_authorization_fixture(
            &path,
            test_root,
            "thread-fix3-pre",
            1_765_300_000_000,
        );
        let runner = PermissiveExperimentRunner {
            stats: CodexDispatchReadbackStats {
                transcript_event_count: 0,
                transcript_target_hits: 0,
            },
        };
        let request = ConfirmAndStartAuthorizedRunRequest {
            project_root: test_root.to_string(),
            proposal_id: proposal.proposal_id.clone(),
            session_choice: "existing".to_string(),
            session_id: Some("thread-fix3-pre".to_string()),
            actor_id: Some("user".to_string()),
            max_nodes: Some(10),
            approved_planned_tasks: None,
            preview_session_bindings: vec![],
        };
        let result = run_confirm_and_start_authorized_run_inner(
            &path,
            &index,
            &readback_db_path,
            &runner,
            &BombDirector,
            &PanicJiaobanSessionCreator,
            &request,
        );
        assert!(result.is_err(), "非 Pending 方案应拒");
        assert!(
            !audit_has(&path, "role_loop_auto_advance_stopped"),
            "确认前（record_decision 之前）的拒不写 stopped 审计"
        );
        let _ = fs::remove_dir_all(dir);
    }

    // 读某 work_item 当前 state（fix4 残料测试用）。
    fn fix4_work_item_state(path: &Path, workflow_id: &str, work_item_id: &str) -> Option<String> {
        find_work_item(&read_json_file(path), workflow_id, work_item_id)
            .and_then(|item| optional_string_from(item, "state"))
    }

    // 注入一条 running 旧链记录（同 workflow/project·模拟上次进程没了没收尾）。
    fn fix4_inject_running_chain(path: &Path, workflow_id: &str, test_root: &str) {
        let mut value = read_workflow_state_value(path).expect("state");
        ensure_array_mut(&mut value, "workflow_chain_runs")
            .expect("arr")
            .push(json!({
              "chain_run_id": "prior-interrupted",
              "project_id": project_id(test_root),
              "workflow_id": workflow_id,
              "state": "running",
              "stop_requested": false,
              "max_nodes": 10,
              "started_at": "t0",
              "ended_at": Value::Null,
              "nodes": []
            }));
        write_validated_workflow_state(path, &value).expect("inject running chain");
    }

    fn fix4_permissive_runner() -> PermissiveExperimentRunner {
        PermissiveExperimentRunner {
            stats: CodexDispatchReadbackStats {
                transcript_event_count: 3,
                transcript_target_hits: 1,
            },
        }
    }

    // fix4 2.2·重拆即新链（re-plan）：挂 running 旧链 → re-plan 跑 → 旧记录标结 superseded + outcome.warnings 含
    // 「标结/重跑」+ 全程无「已接续」+ 新链另起 + 任务完成。
    #[test]
    fn jiaoban_fix4_replan_supersedes_stale_chain() {
        let (dir, index_path, index, workflow_id) =
            auto_advance_fixture("fix4-replan-supersede", true);
        let path = dir.join("workflow-state.v0.json");
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        fix4_inject_running_chain(&path, &workflow_id, test_root);
        let runner = fix4_permissive_runner();
        let outcome = run_auto_advance_authorized_role_loop(
            &path,
            &index,
            &index_path,
            &runner,
            &StubDirector,
            test_root,
            &workflow_id,
            "tester",
            10,
            None,
        )
        .expect("re-plan 应跑通");
        let runs = read_json_file(&path)["workflow_chain_runs"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        let prior = runs
            .iter()
            .find(|r| {
                optional_string_from(r, "chain_run_id").as_deref() == Some("prior-interrupted")
            })
            .expect("prior 记录还在");
        assert_eq!(
            optional_string_from(prior, "state").as_deref(),
            Some("superseded"),
            "旧链应被标结 superseded（离开 running/stopped）"
        );
        assert!(
            runs.iter().any(|r| {
                optional_string_from(r, "chain_run_id").as_deref() != Some("prior-interrupted")
            }),
            "应另起新链记录（≠prior-interrupted）"
        );
        let chain = outcome.chain_outcome.expect("ran 带 chain");
        assert!(
            outcome
                .warnings
                .iter()
                .any(|w| w.contains("标结") || w.contains("重跑")),
            "re-plan 应告知重来：{:?}",
            outcome.warnings
        );
        assert!(
            !outcome
                .warnings
                .iter()
                .chain(chain.warnings.iter())
                .any(|w| w.contains("已接续")),
            "re-plan 不该出现「已接续」：outcome={:?} chain={:?}",
            outcome.warnings,
            chain.warnings
        );
        assert!(chain.completed >= 1, "本轮任务应完成");
        let _ = fs::remove_dir_all(dir);
    }

    // fix4 2.2·approved 续跑不动：挂 running 旧链 → approved（所批即所跑）→ 仍接续（不标结）·chain.warnings 含「已接续」。
    #[test]
    fn jiaoban_fix4_approved_still_resumes_stale_chain() {
        let timestamp_ms = 1_765_300_000_000;
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let thread_id = "thread-fix4-approved-resume";
        let dir = test_temp_dir("fix4-approved-resume");
        let path = dir.join("workflow-state.v0.json");
        let index_path = dir.join("codex-index.json");
        let index = fixture_dispatch_index(test_root, thread_id);
        bootstrap_project_workflow_at(&path, &fixture_project(test_root)).expect("workflow");
        let (proposal, _auth, _rev) = create_active_project_director_authorization_fixture(
            &path,
            test_root,
            thread_id,
            timestamp_ms,
        );
        let workflow_id = default_workflow_id(test_root);
        bind_workflow_node_codex_session_for_index_at(
            &path,
            &index,
            &fixture_node_session_bind_request(
                test_root,
                &format!("{workflow_id}:node:codex-dev"),
                None,
                thread_id,
            ),
        )
        .expect("bind");
        let ctx = load_project_context(test_root).expect("ctx");
        let approved = StubDirector.plan(&ctx, &proposal).expect("approved graph");
        fix4_inject_running_chain(&path, &workflow_id, test_root);
        let runner = fix4_permissive_runner();
        let outcome = run_auto_advance_authorized_role_loop(
            &path,
            &index,
            &index_path,
            &runner,
            &BombDirector,
            test_root,
            &workflow_id,
            "tester",
            10,
            Some(&approved),
        )
        .expect("approved 应跑通");
        let prior_state = read_json_file(&path)["workflow_chain_runs"]
            .as_array()
            .and_then(|runs| {
                runs.iter()
                    .find(|r| {
                        optional_string_from(r, "chain_run_id").as_deref()
                            == Some("prior-interrupted")
                    })
                    .and_then(|r| optional_string_from(r, "state"))
            });
        assert_ne!(
            prior_state.as_deref(),
            Some("superseded"),
            "approved 路不标结旧链（续跑语义不动）"
        );
        let chain = outcome.chain_outcome.expect("ran 带 chain");
        assert!(
            chain.warnings.iter().any(|w| w.contains("已接续")),
            "approved 续跑应仍告知「已接续」：{:?}",
            chain.warnings
        );
        let _ = fs::remove_dir_all(dir);
    }

    // fix4 2.1·残料接管：task1 遗留 ready_for_review + task2 遗留 running（撞 C4 保护）→ re-plan → 合法复位 →
    // prepare 过、ran、warnings 含「已接管」。
    #[test]
    fn jiaoban_fix4_reconcile_stale_residue_unblocks_replan() {
        let timestamp_ms = 1_765_300_000_000;
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let thread_id = "thread-fix4-reconcile";
        let dir = test_temp_dir("fix4-reconcile");
        let path = dir.join("workflow-state.v0.json");
        let index_path = dir.join("codex-index.json");
        let index = fixture_dispatch_index(test_root, thread_id);
        bootstrap_project_workflow_at(&path, &fixture_project(test_root)).expect("workflow");
        let (proposal, authorization, revision) =
            create_active_project_director_authorization_fixture(
                &path,
                test_root,
                thread_id,
                timestamp_ms,
            );
        let workflow_id = default_workflow_id(test_root);
        bind_workflow_node_codex_session_for_index_at(
            &path,
            &index,
            &fixture_node_session_bind_request(
                test_root,
                &format!("{workflow_id}:node:codex-dev"),
                None,
                thread_id,
            ),
        )
        .expect("bind");
        let ctx = load_project_context(test_root).expect("ctx");
        let planned = StubDirector.plan(&ctx, &proposal).expect("plan"); // 2 任务
                                                                         // 第一轮 prepare：建工作项（ready_to_dispatch）。
        let prepared = prepare_authorized_auto_dispatch_for_index_at(
            &path,
            &index,
            &fixture_project_director_prepare_input(
                test_root,
                &proposal.proposal_id,
                &authorization.authorization_id,
                revision,
                planned.clone(),
            ),
        )
        .expect("prepare1");
        let wid1 = prepared.plan.planned_tasks[0]
            .work_item_id
            .clone()
            .expect("wid1");
        let wid2 = prepared.plan.planned_tasks[1]
            .work_item_id
            .clone()
            .expect("wid2");
        // 造线上同款残料：task1 → ready_for_review（活完成审查没记）、task2 → running（派发后进程死）。
        let step = |wid: &str, next: &str| {
            update_work_item_state_at(
                &path,
                &WorkItemStateUpdateRequest {
                    project_root: test_root.to_string(),
                    work_item_id: wid.to_string(),
                    next_state: next.to_string(),
                },
            )
            .expect("set residue state");
        };
        step(&wid1, "running");
        step(&wid1, "ready_for_review");
        step(&wid2, "running");
        assert_eq!(
            fix4_work_item_state(&path, &workflow_id, &wid1).as_deref(),
            Some("ready_for_review"),
            "task1 残料就位"
        );
        // re-plan 再跑 → reconcile 接管 → prepare 过 → ran + 已接管。
        let runner = fix4_permissive_runner();
        let outcome = run_auto_advance_authorized_role_loop(
            &path,
            &index,
            &index_path,
            &runner,
            &StubDirector,
            test_root,
            &workflow_id,
            "tester",
            10,
            None,
        )
        .expect("残料接管后 re-plan 应跑通");
        assert_eq!(
            outcome.stage, "completed",
            "残料被合法接管后 re-plan 应 completed（而非被 C4 卡死）：{outcome:?}"
        );
        assert!(
            outcome.warnings.iter().any(|w| w.contains("已接管")),
            "应告知接管：{:?}",
            outcome.warnings
        );
        let _ = fs::remove_dir_all(dir);
    }

    // fix4 2.1·accepted 不接管：accepted 残料 → C4 照拒（Err）+ 状态不被改（终态不碰）。
    #[test]
    fn jiaoban_fix4_reconcile_skips_accepted_residue() {
        let timestamp_ms = 1_765_300_000_000;
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let thread_id = "thread-fix4-accepted";
        let dir = test_temp_dir("fix4-accepted");
        let path = dir.join("workflow-state.v0.json");
        let index_path = dir.join("codex-index.json");
        let index = fixture_dispatch_index(test_root, thread_id);
        bootstrap_project_workflow_at(&path, &fixture_project(test_root)).expect("workflow");
        let (proposal, authorization, revision) =
            create_active_project_director_authorization_fixture(
                &path,
                test_root,
                thread_id,
                timestamp_ms,
            );
        let workflow_id = default_workflow_id(test_root);
        bind_workflow_node_codex_session_for_index_at(
            &path,
            &index,
            &fixture_node_session_bind_request(
                test_root,
                &format!("{workflow_id}:node:codex-dev"),
                None,
                thread_id,
            ),
        )
        .expect("bind");
        let ctx = load_project_context(test_root).expect("ctx");
        let planned = StubDirector.plan(&ctx, &proposal).expect("plan");
        let prepared = prepare_authorized_auto_dispatch_for_index_at(
            &path,
            &index,
            &fixture_project_director_prepare_input(
                test_root,
                &proposal.proposal_id,
                &authorization.authorization_id,
                revision,
                planned.clone(),
            ),
        )
        .expect("prepare1");
        let wid1 = prepared.plan.planned_tasks[0]
            .work_item_id
            .clone()
            .expect("wid1");
        // task1 走到 accepted（终态）：ready_to_dispatch→running→ready_for_review→accepted。
        for next in ["running", "ready_for_review", "accepted"] {
            update_work_item_state_at(
                &path,
                &WorkItemStateUpdateRequest {
                    project_root: test_root.to_string(),
                    work_item_id: wid1.clone(),
                    next_state: next.to_string(),
                },
            )
            .expect("to accepted");
        }
        let runner = fix4_permissive_runner();
        let result = run_auto_advance_authorized_role_loop(
            &path,
            &index,
            &index_path,
            &runner,
            &StubDirector,
            test_root,
            &workflow_id,
            "tester",
            10,
            None,
        );
        assert!(
            result.is_err(),
            "accepted 残料不接管 → C4 照拒 → Err（不吞错）"
        );
        assert_eq!(
            fix4_work_item_state(&path, &workflow_id, &wid1).as_deref(),
            Some("accepted"),
            "accepted 终态未被接管·状态不变"
        );
        let _ = fs::remove_dir_all(dir);
    }

    // fix4 2.1·只碰本轮 ids：外来（canvas-run 形状）id 的 ready_for_review 残料 → reconcile 不碰（防全库扫荡）。
    #[test]
    fn jiaoban_fix4_reconcile_only_touches_planned_ids() {
        let timestamp_ms = 1_765_300_000_000;
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let thread_id = "thread-fix4-foreign";
        let dir = test_temp_dir("fix4-foreign");
        let path = dir.join("workflow-state.v0.json");
        let index = fixture_dispatch_index(test_root, thread_id);
        bootstrap_project_workflow_at(&path, &fixture_project(test_root)).expect("workflow");
        let (proposal, authorization, revision) =
            create_active_project_director_authorization_fixture(
                &path,
                test_root,
                thread_id,
                timestamp_ms,
            );
        let workflow_id = default_workflow_id(test_root);
        let ctx = load_project_context(test_root).expect("ctx");
        let planned = StubDirector.plan(&ctx, &proposal).expect("plan");
        // 先 prepare 建出正规工作项（拿一个真 work_item 的完整形状去克隆）。
        prepare_authorized_auto_dispatch_for_index_at(
            &path,
            &index,
            &fixture_project_director_prepare_input(
                test_root,
                &proposal.proposal_id,
                &authorization.authorization_id,
                revision,
                planned.clone(),
            ),
        )
        .expect("prepare1");
        // 注入一条**外来 id**（canvas-run 形状·不在本轮 planned 派生集）的 ready_for_review 残料（克隆真形状改 id/state）。
        let foreign_id = "canvas-run:work-item:foreign-legacy-1";
        let mut value = read_workflow_state_value(&path).expect("state");
        let mut foreign = value["work_items"]
            .as_array()
            .and_then(|items| items.first())
            .cloned()
            .expect("有一个真工作项可克隆");
        foreign["work_item_id"] = json!(foreign_id);
        foreign["state"] = json!("ready_for_review");
        ensure_array_mut(&mut value, "work_items")
            .expect("arr")
            .push(foreign);
        write_validated_workflow_state(&path, &value).expect("inject foreign");
        // 直接调 reconcile（本轮 planned）——外来 id 不在派生集 → 不该被碰。
        let warnings = reconcile_stale_work_items_for_plan(&path, test_root, &planned);
        assert_eq!(
            fix4_work_item_state(&path, &workflow_id, foreign_id).as_deref(),
            Some("ready_for_review"),
            "外来（canvas-run）残料不该被 reconcile 碰（只扫本轮 ids）"
        );
        assert!(
            !warnings.iter().any(|w| w.contains(foreign_id)),
            "reconcile 不该提及外来 id：{warnings:?}"
        );
        let _ = fs::remove_dir_all(dir);
    }

    // ===== 交办 fix5（中断遗留的 running 派发墓碑标结）·stub =====

    fn fix5_inject_dispatch(
        path: &Path,
        dispatch_id: &str,
        workflow_id: &str,
        node_id: &str,
        state: &str,
        started_at_ms: i64,
    ) {
        let mut v = read_workflow_state_value(path).expect("state");
        ensure_array_mut(&mut v, "workflow_node_dispatches")
            .expect("arr")
            .push(json!({
              "dispatch_id": dispatch_id,
              "workflow_id": workflow_id,
              "node_id": node_id,
              "state": state,
              "started_at_ms": started_at_ms,
              "warnings": []
            }));
        write_validated_workflow_state(path, &v).expect("inject dispatch");
    }

    fn fix5_dispatch_state(path: &Path, dispatch_id: &str) -> Option<String> {
        read_json_file(path)["workflow_node_dispatches"]
            .as_array()?
            .iter()
            .find(|d| optional_string_from(d, "dispatch_id").as_deref() == Some(dispatch_id))
            .and_then(|d| optional_string_from(d, "state"))
    }

    // fix5·复刻撞死：本轮 codex-dev 节点上超龄（40 分钟前·>30min 物理上限）running 墓碑 → 标结 failed + 审计 + warning。
    #[test]
    fn jiaoban_fix5_supersedes_stale_running_dispatch() {
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let dir = test_temp_dir("fix5-supersede");
        let path = dir.join("workflow-state.v0.json");
        bootstrap_project_workflow_at(&path, &fixture_project(test_root)).expect("workflow");
        let workflow_id = default_workflow_id(test_root);
        let node_id = format!("{workflow_id}:node:codex-dev");
        let now_ms = 2_000_000_000_000i64;
        fix5_inject_dispatch(
            &path,
            "d-stale-1",
            &workflow_id,
            &node_id,
            "running",
            now_ms - 40 * 60_000,
        );
        let planned = vec![jiaoban_test_planned_task(&workflow_id, 1, "t", vec![])]; // role codex-dev
        let warnings = reconcile_stale_running_dispatches(&path, &planned, now_ms);
        assert_eq!(
            fix5_dispatch_state(&path, "d-stale-1").as_deref(),
            Some("failed"),
            "超龄 running 墓碑应标结 failed"
        );
        let d = read_json_file(&path)["workflow_node_dispatches"]
            .as_array()
            .and_then(|a| {
                a.iter()
                    .find(|d| {
                        optional_string_from(d, "dispatch_id").as_deref() == Some("d-stale-1")
                    })
                    .cloned()
            })
            .expect("dispatch 在");
        assert!(
            string_array(&d, "warnings")
                .iter()
                .any(|w| w.contains("中断遗留")),
            "记录应带说明 warning"
        );
        assert!(
            warnings.iter().any(|w| w.contains("已标结")),
            "outcome 应告知已标结：{warnings:?}"
        );
        assert!(
            audit_has(&path, "stale_running_dispatch_superseded"),
            "标结应有审计事件"
        );
        let _ = fs::remove_dir_all(dir);
    }

    // fix5·防误杀：新鲜（1 分钟前）running 不被碰 + 出「可能仍在执行」人话 + 无标结/审计。
    #[test]
    fn jiaoban_fix5_fresh_running_dispatch_not_touched() {
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let dir = test_temp_dir("fix5-fresh");
        let path = dir.join("workflow-state.v0.json");
        bootstrap_project_workflow_at(&path, &fixture_project(test_root)).expect("workflow");
        let workflow_id = default_workflow_id(test_root);
        let node_id = format!("{workflow_id}:node:codex-dev");
        let now_ms = 2_000_000_000_000i64;
        fix5_inject_dispatch(
            &path,
            "d-fresh",
            &workflow_id,
            &node_id,
            "running",
            now_ms - 60_000,
        );
        let planned = vec![jiaoban_test_planned_task(&workflow_id, 1, "t", vec![])];
        let warnings = reconcile_stale_running_dispatches(&path, &planned, now_ms);
        assert_eq!(
            fix5_dispatch_state(&path, "d-fresh").as_deref(),
            Some("running"),
            "新鲜 running 绝不被碰（防误杀真活）"
        );
        assert!(
            warnings.iter().any(|w| w.contains("可能仍有一次执行")),
            "应出「可能仍在执行」人话：{warnings:?}"
        );
        assert!(!warnings.iter().any(|w| w.contains("已标结")), "不该标结");
        assert!(
            !audit_has(&path, "stale_running_dispatch_superseded"),
            "不该有标结审计"
        );
        let _ = fs::remove_dir_all(dir);
    }

    // fix5·范围纪律：别的 workflow / canvas 形状 node / prepared / 终态 —— 全不被碰（逐类断言原样）。
    #[test]
    fn jiaoban_fix5_scope_only_touches_planned_node_running() {
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let dir = test_temp_dir("fix5-scope");
        let path = dir.join("workflow-state.v0.json");
        bootstrap_project_workflow_at(&path, &fixture_project(test_root)).expect("workflow");
        let workflow_id = default_workflow_id(test_root);
        let node_id = format!("{workflow_id}:node:codex-dev");
        let now_ms = 2_000_000_000_000i64;
        let old = now_ms - 40 * 60_000;
        fix5_inject_dispatch(
            &path,
            "d-other-wf",
            "other-workflow",
            "other-workflow:node:codex-dev",
            "running",
            old,
        ); // 别的 workflow
        fix5_inject_dispatch(
            &path,
            "d-canvas",
            &workflow_id,
            &format!("{workflow_id}:node:canvas-legacy"),
            "running",
            old,
        ); // 本 wf·非本轮节点
        fix5_inject_dispatch(&path, "d-prepared", &workflow_id, &node_id, "prepared", old); // 本轮节点·非 running
        fix5_inject_dispatch(
            &path,
            "d-completed",
            &workflow_id,
            &node_id,
            "completed",
            old,
        ); // 终态
        let planned = vec![jiaoban_test_planned_task(&workflow_id, 1, "t", vec![])]; // 本轮节点=codex-dev
        let warnings = reconcile_stale_running_dispatches(&path, &planned, now_ms);
        assert_eq!(
            fix5_dispatch_state(&path, "d-other-wf").as_deref(),
            Some("running"),
            "别的 workflow 的 running 不扫"
        );
        assert_eq!(
            fix5_dispatch_state(&path, "d-canvas").as_deref(),
            Some("running"),
            "本 wf 但非本轮节点（canvas 形状）不碰"
        );
        assert_eq!(
            fix5_dispatch_state(&path, "d-prepared").as_deref(),
            Some("prepared"),
            "prepared 不碰（闸不数它）"
        );
        assert_eq!(
            fix5_dispatch_state(&path, "d-completed").as_deref(),
            Some("completed"),
            "终态不碰"
        );
        assert!(
            !warnings.iter().any(|w| w.contains("已标结")),
            "没有匹配的超龄 running → 不标结：{warnings:?}"
        );
        assert!(!audit_has(&path, "stale_running_dispatch_superseded"));
        let _ = fs::remove_dir_all(dir);
    }

    // fix5·端到端（re-plan 两分支同享挂载点）：注入线上同款墓碑（codex-dev·running·1 小时前）→ auto_advance →
    // 标结解除 duplicate_blocked → 链跑通 ran + 墓碑 failed + warnings 告知。
    #[test]
    fn jiaoban_fix5_replan_unblocks_after_superseding_stale_dispatch() {
        let (dir, index_path, index, workflow_id) = auto_advance_fixture("fix5-unblock", true);
        let path = dir.join("workflow-state.v0.json");
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let node_id = format!("{workflow_id}:node:codex-dev");
        let now_ms = unix_timestamp_ms();
        fix5_inject_dispatch(
            &path,
            "d-crash-tombstone",
            &workflow_id,
            &node_id,
            "running",
            now_ms - 60 * 60_000,
        ); // 1 小时前
        let runner = fix4_permissive_runner();
        let outcome = run_auto_advance_authorized_role_loop(
            &path,
            &index,
            &index_path,
            &runner,
            &StubDirector,
            test_root,
            &workflow_id,
            "tester",
            10,
            None,
        )
        .expect("标结墓碑后应跑通");
        assert_eq!(
            outcome.stage, "completed",
            "超龄墓碑标结后 re-plan 应 completed（不再 duplicate_blocked）：{outcome:?}"
        );
        assert!(
            outcome.warnings.iter().any(|w| w.contains("已标结")),
            "应告知标结：{:?}",
            outcome.warnings
        );
        assert_eq!(
            fix5_dispatch_state(&path, "d-crash-tombstone").as_deref(),
            Some("failed"),
            "墓碑记录应被标结 failed"
        );
        let _ = fs::remove_dir_all(dir);
    }

    // fix5·approved 路同享：挂载点在两分支合流后 → approved（所批即所跑）路径也标结超龄墓碑（§4 明列）。
    #[test]
    fn jiaoban_fix5_approved_path_also_supersedes_stale_dispatch() {
        let timestamp_ms = 1_765_300_000_000;
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let thread_id = "thread-fix5-approved";
        let dir = test_temp_dir("fix5-approved");
        let path = dir.join("workflow-state.v0.json");
        let index_path = dir.join("codex-index.json");
        let index = fixture_dispatch_index(test_root, thread_id);
        bootstrap_project_workflow_at(&path, &fixture_project(test_root)).expect("workflow");
        let (proposal, _auth, _rev) = create_active_project_director_authorization_fixture(
            &path,
            test_root,
            thread_id,
            timestamp_ms,
        );
        let workflow_id = default_workflow_id(test_root);
        bind_workflow_node_codex_session_for_index_at(
            &path,
            &index,
            &fixture_node_session_bind_request(
                test_root,
                &format!("{workflow_id}:node:codex-dev"),
                None,
                thread_id,
            ),
        )
        .expect("bind");
        let ctx = load_project_context(test_root).expect("ctx");
        let approved = StubDirector.plan(&ctx, &proposal).expect("approved graph");
        let now_ms = unix_timestamp_ms();
        fix5_inject_dispatch(
            &path,
            "d-approved-tombstone",
            &workflow_id,
            &format!("{workflow_id}:node:codex-dev"),
            "running",
            now_ms - 60 * 60_000,
        );
        let runner = fix4_permissive_runner();
        let outcome = run_auto_advance_authorized_role_loop(
            &path,
            &index,
            &index_path,
            &runner,
            &BombDirector,
            test_root,
            &workflow_id,
            "tester",
            10,
            Some(&approved),
        )
        .expect("approved 标结墓碑后应跑通");
        assert_eq!(
            outcome.stage, "completed",
            "approved 路也应标结墓碑后 completed：{outcome:?}"
        );
        assert_eq!(
            fix5_dispatch_state(&path, "d-approved-tombstone").as_deref(),
            Some("failed"),
            "approved 路墓碑也被标结（挂载点两分支同享）"
        );
        let _ = fs::remove_dir_all(dir);
    }

    // P3 项目面真跑（C 映射）·机器闸：用已存在的 work_item（带任务包）+ 绑定，stub runner 验证
    // execute_project_workflow_node_at 从任务包构造指令 + 走通派发到 completed。
    #[test]
    fn project_workflow_node_dispatch_uses_task_package_and_dispatches() {
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let dir =
            std::env::temp_dir().join(format!("project-node-dispatch-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        let index_path = dir.join("codex-index.json");
        let project = fixture_project(test_root);
        let index = fixture_dispatch_index(test_root, "thread-proj-1");
        fs::create_dir_all(&dir).expect("fixture dir should exist");
        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_task_draft_at(
            &path,
            &fixture_task_draft_request(test_root, "项目节点真跑任务"),
        )
        .expect("work item should exist");
        let work_item_id =
            optional_string_from(&read_json_file(&path)["work_items"][0], "work_item_id")
                .expect("work item id should exist");
        update_work_item_state_at(
            &path,
            &fixture_work_item_state_update_request(test_root, &work_item_id, "ready_to_dispatch"),
        )
        .expect("work item should be ready");
        let workflow_id = default_workflow_id(test_root);
        let node_id = format!("{workflow_id}:node:codex-dev");
        let session = fixture_session("thread-proj-1", test_root, true);
        bind_workflow_node_codex_session_at(
            &path,
            &fixture_node_session_bind_request(
                test_root,
                &node_id,
                Some(&work_item_id),
                "thread-proj-1",
            ),
            &session,
        )
        .expect("binding should write");
        let runner = PermissiveExperimentRunner {
            stats: CodexDispatchReadbackStats {
                transcript_event_count: 3,
                transcript_target_hits: 1,
            },
        };
        DISPATCH_READBACK_NATIVE_READ_COUNT.with(|count| count.set(0));
        let request = ProjectWorkflowNodeRunRequest {
            project_root: test_root.to_string(),
            node_id,
            work_item_id,
            workflow_id: None,
        };
        let result =
            execute_project_workflow_node_at(&path, &index, &index_path, &runner, &request)
                .expect("project node dispatch should complete");
        assert_eq!(result.dispatch.state, "completed");
        assert_eq!(result.dispatch.exit_code, Some(0));
    }

    // 只读单的案发式回归：任务包漏 allowed_write 时，H5 bridge 的 fail-open 不在这条主管派发链上；
    // 本执行入口必须保守地把它喂为 read-only + 空写根。
    #[test]
    fn project_workflow_node_dispatch_keeps_missing_task_package_write_scope_readonly() {
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let dir = std::env::temp_dir().join(format!(
            "project-node-readonly-missing-write-{}",
            unix_timestamp_string()
        ));
        let path = dir.join("workflow-state.v0.json");
        let index_path = dir.join("codex-index.json");
        let index = fixture_dispatch_index(test_root, "thread-proj-readonly");
        fs::create_dir_all(&dir).expect("fixture dir should exist");
        bootstrap_project_workflow_at(&path, &fixture_project(test_root))
            .expect("workflow should exist");
        create_task_draft_at(
            &path,
            &fixture_task_draft_request(test_root, "只读任务包缺写范围"),
        )
        .expect("work item should exist");
        let work_item_id =
            optional_string_from(&read_json_file(&path)["work_items"][0], "work_item_id")
                .expect("work item id should exist");
        update_work_item_state_at(
            &path,
            &fixture_work_item_state_update_request(test_root, &work_item_id, "ready_to_dispatch"),
        )
        .expect("work item should be ready");
        let state = read_json_file(&path);
        let artifact = state["artifacts"]
            .as_array()
            .and_then(|artifacts| artifacts.first())
            .expect("task package artifact should exist");
        assert!(
            artifact.get("allowed_write").is_none(),
            "案发夹具必须复刻缺 allowed_write：{artifact:?}"
        );
        let workflow_id = default_workflow_id(test_root);
        let node_id = format!("{workflow_id}:node:codex-dev");
        bind_workflow_node_codex_session_at(
            &path,
            &fixture_node_session_bind_request(
                test_root,
                &node_id,
                Some(&work_item_id),
                "thread-proj-readonly",
            ),
            &fixture_session("thread-proj-readonly", test_root, true),
        )
        .expect("binding should write");
        let runner = RecordingOptionsRunner::default();
        let result = execute_project_workflow_node_at(
            &path,
            &index,
            &index_path,
            &runner,
            &ProjectWorkflowNodeRunRequest {
                project_root: test_root.to_string(),
                node_id,
                work_item_id,
                workflow_id: None,
            },
        )
        .expect("missing allowed_write should still dispatch as readonly");
        assert_eq!(result.dispatch.state, "completed");
        let options = runner
            .options
            .borrow()
            .clone()
            .expect("runner should receive execution options");
        assert_eq!(options.sandbox_mode.as_deref(), Some("read-only"));
        assert!(
            options.allowed_write_roots.is_empty(),
            "只读派发不可带写目录：{:?}",
            options.allowed_write_roots
        );
        println!(
            "[READONLY_DISPATCH_OPTIONS] sandbox={:?} write_roots={:?}",
            options.sandbox_mode, options.allowed_write_roots
        );
        let _ = fs::remove_dir_all(dir);
    }

    // C 映射语义：work_item 不是 ready_to_dispatch 时，派发自身清楚拒绝（不自动推进状态）。
    #[test]
    fn project_workflow_node_dispatch_refuses_when_work_item_not_ready() {
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let dir =
            std::env::temp_dir().join(format!("project-node-notready-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        let index_path = dir.join("codex-index.json");
        let project = fixture_project(test_root);
        let index = fixture_dispatch_index(test_root, "thread-proj-2");
        fs::create_dir_all(&dir).expect("fixture dir should exist");
        bootstrap_project_workflow_at(&path, &project).expect("workflow should exist");
        create_task_draft_at(
            &path,
            &fixture_task_draft_request(test_root, "未就绪项目任务"),
        )
        .expect("work item should exist");
        let work_item_id =
            optional_string_from(&read_json_file(&path)["work_items"][0], "work_item_id")
                .expect("work item id should exist");
        // 故意不推进到 ready_to_dispatch（停在 draft）。
        let workflow_id = default_workflow_id(test_root);
        let node_id = format!("{workflow_id}:node:codex-dev");
        let session = fixture_session("thread-proj-2", test_root, true);
        bind_workflow_node_codex_session_at(
            &path,
            &fixture_node_session_bind_request(
                test_root,
                &node_id,
                Some(&work_item_id),
                "thread-proj-2",
            ),
            &session,
        )
        .expect("binding should write");
        let runner = PermissiveExperimentRunner {
            stats: CodexDispatchReadbackStats {
                transcript_event_count: 0,
                transcript_target_hits: 0,
            },
        };
        let request = ProjectWorkflowNodeRunRequest {
            project_root: test_root.to_string(),
            node_id,
            work_item_id,
            workflow_id: None,
        };
        let error = execute_project_workflow_node_at(&path, &index, &index_path, &runner, &request)
            .expect_err("draft work item 不可派发，应拒绝");
        assert!(error.contains("待派发"), "错误应点明工作项未就绪：{error}");
    }

    // P3 E · 多工作流底座（架构 §12）·机器闸。
    #[test]
    fn submit_project_workflow_draft_creates_new_workflow() {
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let dir = std::env::temp_dir().join(format!("submit-new-wf-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        fs::create_dir_all(&dir).expect("fixture dir should exist");
        bootstrap_project_workflow_at(&path, &fixture_project(test_root))
            .expect("workflow should exist");
        let before = read_json_file(&path)["workflows"]
            .as_array()
            .map(|a| a.len())
            .unwrap_or(0);
        let request = SubmitProjectWorkflowDraftRequest {
            project_root: test_root.to_string(),
            workflow_id: None,
            title: "我的新工作流".to_string(),
            nodes: vec![
                json!({"id":"d1","kind":"director","label":"主管","prompt":"统筹","position":{"x":1,"y":2}}),
                json!({"id":"a1","kind":"subagent","label":"开发","prompt":"写代码","position":{"x":3,"y":4}}),
            ],
            edges: vec![json!({"id":"e1","from":"d1","to":"a1"})],
        };
        let result =
            submit_project_workflow_draft_at(&path, &request).expect("submit new should write");
        assert!(
            result.message.contains("已新建"),
            "应报已新建：{}",
            result.message
        );
        let after = read_json_file(&path);
        assert_eq!(
            after["workflows"].as_array().unwrap().len(),
            before + 1,
            "应新增一个工作流（不覆盖默认）"
        );
        // 新工作流带 2 节点 1 边 + canvas_payload 往返。
        let new_wf = after["workflows"]
            .as_array()
            .unwrap()
            .iter()
            .find(|w| optional_string_from(w, "title").as_deref() == Some("我的新工作流"))
            .expect("new workflow present");
        let wid = optional_string_from(new_wf, "workflow_id").unwrap();
        assert_ne!(
            wid,
            default_workflow_id(test_root),
            "新工作流 id 不是 default"
        );
        let node_count = after["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|n| optional_string_from(n, "workflow_id").as_deref() == Some(wid.as_str()))
            .count();
        assert_eq!(node_count, 2, "新工作流应有 2 个节点");
        let director = after["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| {
                optional_string_from(n, "workflow_id").as_deref() == Some(wid.as_str())
                    && optional_string_from(n, "node_type").as_deref() == Some("director")
            })
            .expect("director node present");
        assert_eq!(
            optional_string_from(&director["canvas_payload"], "prompt").as_deref(),
            Some("统筹"),
            "canvas_payload 应原样存（往返）"
        );
    }

    #[test]
    fn submit_project_workflow_draft_rejects_without_director() {
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let dir = std::env::temp_dir().join(format!("submit-nodir-wf-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        fs::create_dir_all(&dir).expect("fixture dir should exist");
        bootstrap_project_workflow_at(&path, &fixture_project(test_root))
            .expect("workflow should exist");
        let request = SubmitProjectWorkflowDraftRequest {
            project_root: test_root.to_string(),
            workflow_id: None,
            title: "缺主管的工作流".to_string(),
            nodes: vec![
                json!({"id":"a1","kind":"subagent","label":"开发","position":{"x":1,"y":1}}),
            ],
            edges: vec![],
        };
        let error = submit_project_workflow_draft_at(&path, &request)
            .expect_err("无 director 应被运行性检查挡");
        assert!(error.contains("运行性"), "错误应点明运行性未通过：{error}");
    }

    #[test]
    fn submit_project_workflow_draft_updates_existing_workflow() {
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let dir = std::env::temp_dir().join(format!("submit-upd-wf-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        fs::create_dir_all(&dir).expect("fixture dir should exist");
        bootstrap_project_workflow_at(&path, &fixture_project(test_root))
            .expect("workflow should exist");
        // 先新建一个有 2 节点的工作流。
        submit_project_workflow_draft_at(
            &path,
            &SubmitProjectWorkflowDraftRequest {
                project_root: test_root.to_string(),
                workflow_id: None,
                title: "待更新工作流".to_string(),
                nodes: vec![
                    json!({"id":"d1","kind":"director","label":"主管","position":{"x":1,"y":1}}),
                    json!({"id":"a1","kind":"subagent","label":"开发","position":{"x":2,"y":2}}),
                ],
                edges: vec![],
            },
        )
        .expect("create should write");
        let wid = optional_string_from(
            read_json_file(&path)["workflows"]
                .as_array()
                .unwrap()
                .iter()
                .find(|w| optional_string_from(w, "title").as_deref() == Some("待更新工作流"))
                .unwrap(),
            "workflow_id",
        )
        .unwrap();
        // 用同 workflow_id 提交，只留 1 个 director 节点 → 应替换成 1 节点。
        let result = submit_project_workflow_draft_at(
            &path,
            &SubmitProjectWorkflowDraftRequest {
                project_root: test_root.to_string(),
                workflow_id: Some(wid.clone()),
                title: "待更新工作流（改）".to_string(),
                nodes: vec![json!({"id":"d1","kind":"director","label":"只剩主管","position":{"x":1,"y":1}})],
                edges: vec![],
            },
        )
        .expect("update should write");
        assert!(
            result.message.contains("已更新"),
            "应报已更新：{}",
            result.message
        );
        let after = read_json_file(&path);
        let node_count = after["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|n| optional_string_from(n, "workflow_id").as_deref() == Some(wid.as_str()))
            .count();
        assert_eq!(node_count, 1, "更新应替换该工作流的节点为 1 个");
    }

    // 后置B·机器闸：编辑删掉某节点后，它的会话绑定被 prune（不悬空、不静默重挂）。
    #[test]
    fn submit_project_workflow_draft_prunes_binding_of_removed_node() {
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let dir =
            std::env::temp_dir().join(format!("submit-prune-bind-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        fs::create_dir_all(&dir).expect("fixture dir should exist");
        bootstrap_project_workflow_at(&path, &fixture_project(test_root))
            .expect("workflow should exist");
        // 新建一个含 director + subagent 的工作流。
        submit_project_workflow_draft_at(
            &path,
            &SubmitProjectWorkflowDraftRequest {
                project_root: test_root.to_string(),
                workflow_id: None,
                title: "B-prune 工作流".to_string(),
                nodes: vec![
                    json!({"id":"d1","kind":"director","label":"主管","position":{"x":1,"y":1}}),
                    json!({"id":"s1","kind":"subagent","label":"开发","position":{"x":2,"y":2}}),
                ],
                edges: vec![],
            },
        )
        .expect("create should write");
        let wid = optional_string_from(
            read_json_file(&path)["workflows"]
                .as_array()
                .unwrap()
                .iter()
                .find(|w| optional_string_from(w, "title").as_deref() == Some("B-prune 工作流"))
                .unwrap(),
            "workflow_id",
        )
        .unwrap();
        // 读回真实 node_id（架构债修后用节点稳定 id、非位置式；测试别假设格式）。
        let subagent_node_id = optional_string_from(
            read_json_file(&path)["nodes"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| {
                    optional_string_from(n, "workflow_id").as_deref() == Some(wid.as_str())
                        && optional_string_from(n, "node_type").as_deref() == Some("subagent")
                })
                .expect("subagent 节点应在 workflow-state 里"),
            "node_id",
        )
        .unwrap();
        // 直接注入一条该节点的 active 绑定（绕开 bind 命令的 default_workflow_id 写死——那是 C#2 的另一处）。
        {
            let mut value = read_json_file(&path);
            value["workflow_node_session_bindings"]
                .as_array_mut()
                .expect("bindings array")
                .push(json!({
                  "binding_id": "binding:b-prune-test",
                  "project_id": project_id(test_root),
                  "workflow_id": wid,
                  "node_id": subagent_node_id,
                  "work_item_id": Value::Null,
                  "native_thread_id": "thread-prune-1",
                  "lifecycle": "active",
                  "created_at_ms": 1,
                  "updated_at_ms": 1,
                  "warnings": []
                }));
            write_validated_workflow_state(&path, &value).expect("inject binding should write");
        }
        let bound_before = read_json_file(&path)["workflow_node_session_bindings"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|b| {
                optional_string_from(b, "node_id").as_deref() == Some(subagent_node_id.as_str())
            })
            .count();
        assert_eq!(bound_before, 1, "前置：subagent 节点应有 1 条绑定");
        // 编辑：只留 director（删掉 subagent）→ 提交更新。
        submit_project_workflow_draft_at(
            &path,
            &SubmitProjectWorkflowDraftRequest {
                project_root: test_root.to_string(),
                workflow_id: Some(wid.clone()),
                title: "B-prune 工作流".to_string(),
                nodes: vec![
                    json!({"id":"d1","kind":"director","label":"主管","position":{"x":1,"y":1}}),
                ],
                edges: vec![],
            },
        )
        .expect("update should write");
        let bound_after = read_json_file(&path)["workflow_node_session_bindings"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|b| {
                optional_string_from(b, "node_id").as_deref() == Some(subagent_node_id.as_str())
            })
            .count();
        assert_eq!(
            bound_after, 0,
            "删掉 subagent 节点后，它的绑定应被 prune（不悬空/不重挂）"
        );
    }

    // 架构债·根治 B：node_id 用节点稳定 id（非位置式）→ 重排节点后，某节点 node_id 不变 → 会话绑定不漂。
    // 位置式时代 subagent 在第 2 位 vs 第 1 位会得到不同 node_id（:node:1-.. ↔ :node:0-..），本测试正是该回归断言。
    #[test]
    fn submit_project_workflow_draft_node_id_stable_across_reorder() {
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let dir =
            std::env::temp_dir().join(format!("submit-stable-id-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        fs::create_dir_all(&dir).expect("fixture dir should exist");
        bootstrap_project_workflow_at(&path, &fixture_project(test_root))
            .expect("workflow should exist");
        let find_subagent = |wid: &str| {
            optional_string_from(
                read_json_file(&path)["nodes"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|n| {
                        optional_string_from(n, "workflow_id").as_deref() == Some(wid)
                            && optional_string_from(n, "node_type").as_deref() == Some("subagent")
                    })
                    .expect("subagent 节点应在"),
                "node_id",
            )
            .unwrap()
        };
        let dir_node =
            json!({"id":"dir","kind":"director","label":"主管","position":{"x":1,"y":1}});
        let sub_node = json!({"id":"sa","kind":"subagent","label":"开发","position":{"x":2,"y":2}});
        // v1：director 在前、subagent 在后。
        submit_project_workflow_draft_at(
            &path,
            &SubmitProjectWorkflowDraftRequest {
                project_root: test_root.to_string(),
                workflow_id: None,
                title: "稳定id 工作流".to_string(),
                nodes: vec![dir_node.clone(), sub_node.clone()],
                edges: vec![],
            },
        )
        .expect("create");
        let wid = optional_string_from(
            read_json_file(&path)["workflows"]
                .as_array()
                .unwrap()
                .iter()
                .find(|w| optional_string_from(w, "title").as_deref() == Some("稳定id 工作流"))
                .unwrap(),
            "workflow_id",
        )
        .unwrap();
        let node_id_v1 = find_subagent(&wid);
        // v2：编辑——把 subagent 挪到第 1 位（位置变），但 canvas id "sa" 不变。
        submit_project_workflow_draft_at(
            &path,
            &SubmitProjectWorkflowDraftRequest {
                project_root: test_root.to_string(),
                workflow_id: Some(wid.clone()),
                title: "稳定id 工作流".to_string(),
                nodes: vec![sub_node, dir_node],
                edges: vec![],
            },
        )
        .expect("update");
        let node_id_v2 = find_subagent(&wid);
        assert_eq!(
            node_id_v1, node_id_v2,
            "subagent 的 node_id 应跨重排稳定（用稳定 id、非位置式）"
        );
    }

    // 后置C#2·机器闸：画布建的（非默认）工作流，节点载荷带 resume 会话 → 运行时自动建临时 work_item
    // + 现绑会话 + 走通派发到 completed（stub）。证明「画布建的工作流也能真跑」闭合。
    #[test]
    fn project_canvas_workflow_node_auto_runs_with_payload_session() {
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let dir = std::env::temp_dir().join(format!("c2-canvas-run-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        let index_path = dir.join("codex-index.json");
        fs::create_dir_all(&dir).expect("fixture dir should exist");
        bootstrap_project_workflow_at(&path, &fixture_project(test_root))
            .expect("workflow should exist");
        // 画布新建一个工作流：director + 一个带 resume 会话载荷的 subagent 节点。
        submit_project_workflow_draft_at(
            &path,
            &SubmitProjectWorkflowDraftRequest {
                project_root: test_root.to_string(),
                workflow_id: None,
                title: "C2画布工作流".to_string(),
                nodes: vec![
                    json!({"id":"d1","kind":"director","label":"主管","position":{"x":1,"y":1}}),
                    json!({"id":"c1","kind":"subagent","label":"开发","position":{"x":2,"y":2},
                           "data":{"session":{"mode":"resume","thread_id":"thread-c2"},"prompt":"建文件 c2-proof.txt","sandbox":"workspace-write"}}),
                ],
                edges: vec![],
            },
        )
        .expect("create canvas workflow");
        let wid = optional_string_from(
            read_json_file(&path)["workflows"]
                .as_array()
                .unwrap()
                .iter()
                .find(|w| optional_string_from(w, "title").as_deref() == Some("C2画布工作流"))
                .unwrap(),
            "workflow_id",
        )
        .unwrap();
        // 读回真实 node_id（架构债修后用节点稳定 id，不再是位置式；测试别假设格式）。
        let node_id = optional_string_from(
            read_json_file(&path)["nodes"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| {
                    optional_string_from(n, "workflow_id").as_deref() == Some(wid.as_str())
                        && optional_string_from(n, "node_type").as_deref() == Some("subagent")
                })
                .expect("提交的 subagent 节点应在 workflow-state 里"),
            "node_id",
        )
        .unwrap();
        let count_wi = |p: &Path, w: &str| {
            read_json_file(p)["work_items"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter(|it| optional_string_from(it, "workflow_id").as_deref() == Some(w))
                        .count()
                })
                .unwrap_or(0)
        };
        let wi_before = count_wi(&path, &wid);
        let runner = PermissiveExperimentRunner {
            stats: CodexDispatchReadbackStats {
                transcript_event_count: 2,
                transcript_target_hits: 1,
            },
        };
        DISPATCH_READBACK_NATIVE_READ_COUNT.with(|count| count.set(0));
        let index = fixture_dispatch_index(test_root, "thread-c2");
        let request = ProjectWorkflowNodeRunRequest {
            project_root: test_root.to_string(),
            node_id: node_id.clone(),
            work_item_id: String::new(), // 空 → 自动建临时 work_item
            workflow_id: Some(wid.clone()),
        };
        let result =
            execute_project_workflow_node_at(&path, &index, &index_path, &runner, &request)
                .expect("画布工作流节点应能自动建票+绑+派发");
        assert_eq!(result.dispatch.state, "completed");
        assert_eq!(result.dispatch.exit_code, Some(0));
        assert!(count_wi(&path, &wid) > wi_before, "应自动建临时 work_item");
        let bound = read_json_file(&path)["workflow_node_session_bindings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|b| {
                optional_string_from(b, "node_id").as_deref() == Some(node_id.as_str())
                    && optional_string_from(b, "native_thread_id").as_deref() == Some("thread-c2")
            });
        assert!(bound, "应给画布节点现绑载荷里的 resume 会话");
    }

    // ===== P1 工作流自动连环 controller（决策 2026-06-23 · 圈固定测试项目） =====

    fn chain_test_workflow_id_by_title(path: &Path, title: &str) -> String {
        optional_string_from(
            read_json_file(path)["workflows"]
                .as_array()
                .unwrap()
                .iter()
                .find(|w| optional_string_from(w, "title").as_deref() == Some(title))
                .expect("workflow by title should exist"),
            "workflow_id",
        )
        .unwrap()
    }

    // 三节点链 a→b→c，每节点带 resume 会话载荷（thread-1/2/3）。
    fn submit_chain_test_workflow(path: &Path, test_root: &str, title: &str) {
        submit_project_workflow_draft_at(
            path,
            &SubmitProjectWorkflowDraftRequest {
                project_root: test_root.to_string(),
                workflow_id: None,
                title: title.to_string(),
                nodes: vec![
                    // a = director（满足运行性检查「需 director 节点」）；带 session，链照样真跑它。
                    json!({"id":"a","kind":"director","label":"主管A","position":{"x":1,"y":1},
                           "data":{"session":{"mode":"resume","thread_id":"thread-1"},"prompt":"步骤A","sandbox":"workspace-write"}}),
                    json!({"id":"b","kind":"subagent","label":"B","position":{"x":2,"y":2},
                           "data":{"session":{"mode":"resume","thread_id":"thread-2"},"prompt":"步骤B","sandbox":"workspace-write"}}),
                    json!({"id":"c","kind":"subagent","label":"C","position":{"x":3,"y":3},
                           "data":{"session":{"mode":"resume","thread_id":"thread-3"},"prompt":"步骤C","sandbox":"workspace-write"}}),
                ],
                edges: vec![
                    json!({"id":"e1","from":"a","to":"b"}),
                    json!({"id":"e2","from":"b","to":"c"}),
                ],
            },
        )
        .expect("create chain workflow");
    }

    fn chain_test_runner() -> PermissiveExperimentRunner {
        PermissiveExperimentRunner {
            stats: CodexDispatchReadbackStats {
                transcript_event_count: 2,
                transcript_target_hits: 1,
            },
        }
    }

    fn audit_has(path: &Path, event_type: &str) -> bool {
        read_json_file(path)["audit_events"]
            .as_array()
            .map(|a| {
                a.iter()
                    .any(|e| optional_string_from(e, "event_type").as_deref() == Some(event_type))
            })
            .unwrap_or(false)
    }

    #[test]
    fn project_workflow_chain_runs_all_nodes_to_completion() {
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let dir = std::env::temp_dir().join(format!("chain-run-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        let index_path = dir.join("codex-index.json");
        fs::create_dir_all(&dir).expect("fixture dir");
        bootstrap_project_workflow_at(&path, &fixture_project(test_root)).expect("workflow");
        submit_chain_test_workflow(&path, test_root, "链工作流");
        let wid = chain_test_workflow_id_by_title(&path, "链工作流");
        let runner = chain_test_runner();
        DISPATCH_READBACK_NATIVE_READ_COUNT.with(|c| c.set(0));
        let index = fixture_multi_thread_index(test_root, &["thread-1", "thread-2", "thread-3"]);
        let request = ProjectWorkflowChainRunRequest {
            project_root: test_root.to_string(),
            workflow_id: wid.clone(),
            max_nodes: None,
        };
        let result = run_project_workflow_chain_at(&path, &index, &index_path, &runner, &request)
            .expect("chain should run to completion");
        assert_eq!(result.state, "completed");
        assert_eq!(result.dispatched_count, 3, "三节点都应真派发");
        for n in &result.nodes {
            assert_eq!(
                optional_string_from(n, "state").as_deref(),
                Some("completed"),
                "每节点应 completed"
            );
        }
        // 拓扑序：a 在 b 前、b 在 c 前（按 canvas id 还原顺序）
        let order: Vec<String> = result
            .nodes
            .iter()
            .filter_map(|n| optional_string_from(n, "node_id"))
            .collect();
        let pos = |canvas: &str| {
            let nid = format!("{wid}:node:{}", stable_id(canvas));
            order.iter().position(|n| *n == nid).unwrap()
        };
        assert!(pos("a") < pos("b") && pos("b") < pos("c"), "应按拓扑序");
        // 审计：链起 + 链完成
        assert!(audit_has(&path, "workflow_chain_run_started"));
        assert!(audit_has(&path, "workflow_chain_run_completed"));
        assert!(audit_has(&path, "workflow_chain_node_completed"));
        // 链运行记录落盘 + completed
        let runs = read_json_file(&path)["workflow_chain_runs"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        assert_eq!(runs.len(), 1, "只一条链运行记录");
        assert_eq!(
            optional_string_from(&runs[0], "state").as_deref(),
            Some("completed")
        );
    }

    #[test]
    fn project_workflow_chain_stops_on_first_node_failure() {
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let dir = std::env::temp_dir().join(format!("chain-fail-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        let index_path = dir.join("codex-index.json");
        fs::create_dir_all(&dir).expect("fixture dir");
        bootstrap_project_workflow_at(&path, &fixture_project(test_root)).expect("workflow");
        submit_chain_test_workflow(&path, test_root, "失败链");
        let wid = chain_test_workflow_id_by_title(&path, "失败链");
        // 失败 runner：第一个节点就 codex 失败 → 失败即停
        let runner = FailingCodexResumeRunner {
            exit_code: 1,
            timed_out: false,
        };
        DISPATCH_READBACK_NATIVE_READ_COUNT.with(|c| c.set(0));
        let index = fixture_multi_thread_index(test_root, &["thread-1", "thread-2", "thread-3"]);
        let request = ProjectWorkflowChainRunRequest {
            project_root: test_root.to_string(),
            workflow_id: wid.clone(),
            max_nodes: None,
        };
        let result = run_project_workflow_chain_at(&path, &index, &index_path, &runner, &request)
            .expect("chain call itself returns Ok with failed state");
        assert_eq!(result.state, "failed", "失败即停 → 链 failed");
        assert_eq!(result.dispatched_count, 0, "没有节点 completed");
        // 第一个节点 failed，后两个仍 pending（没被派发）
        let state_of = |canvas: &str| {
            let nid = format!("{wid}:node:{}", stable_id(canvas));
            result
                .nodes
                .iter()
                .find(|n| optional_string_from(n, "node_id").as_deref() == Some(nid.as_str()))
                .and_then(|n| optional_string_from(n, "state"))
        };
        assert_eq!(state_of("a").as_deref(), Some("failed"));
        assert_eq!(
            state_of("b").as_deref(),
            Some("pending"),
            "失败后不应继续派发 b"
        );
        assert_eq!(
            state_of("c").as_deref(),
            Some("pending"),
            "失败后不应继续派发 c"
        );
        assert!(audit_has(&path, "workflow_chain_node_failed"));
        assert!(audit_has(&path, "workflow_chain_run_failed"));
    }

    #[test]
    fn project_workflow_chain_gate_seals_non_test_project() {
        let dir = std::env::temp_dir().join(format!("chain-gate-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        let index_path = dir.join("codex-index.json");
        fs::create_dir_all(&dir).expect("fixture dir");
        // 非测试真实项目 root → path-lock 闸必须拒（连环更不能碰真实仓）
        let runner = chain_test_runner();
        let index = fixture_dispatch_index("/tmp/some-real-project", "thread-x");
        let request = ProjectWorkflowChainRunRequest {
            project_root: "/tmp/some-real-project".to_string(),
            workflow_id: "workflow:whatever:1".to_string(),
            max_nodes: None,
        };
        let err = run_project_workflow_chain_at(&path, &index, &index_path, &runner, &request)
            .expect_err("非测试项目必须被闸拒");
        assert_eq!(
            err,
            legacy_product_command_blocked_message("start_project_workflow_chain"),
            "应是 path-lock 闸的拒绝消息"
        );
    }

    #[test]
    fn project_workflow_chain_honors_runaway_cap() {
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let dir = std::env::temp_dir().join(format!("chain-cap-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        let index_path = dir.join("codex-index.json");
        fs::create_dir_all(&dir).expect("fixture dir");
        bootstrap_project_workflow_at(&path, &fixture_project(test_root)).expect("workflow");
        submit_chain_test_workflow(&path, test_root, "上限链");
        let wid = chain_test_workflow_id_by_title(&path, "上限链");
        let runner = chain_test_runner();
        DISPATCH_READBACK_NATIVE_READ_COUNT.with(|c| c.set(0));
        let index = fixture_multi_thread_index(test_root, &["thread-1", "thread-2", "thread-3"]);
        // runaway 上限 = 1 → 跑完第一个就到顶停
        let request = ProjectWorkflowChainRunRequest {
            project_root: test_root.to_string(),
            workflow_id: wid.clone(),
            max_nodes: Some(1),
        };
        let result = run_project_workflow_chain_at(&path, &index, &index_path, &runner, &request)
            .expect("chain should run then cap");
        assert_eq!(result.state, "stopped", "到 runaway 上限 → stopped");
        assert_eq!(result.dispatched_count, 1, "只派发 1 个");
        assert!(result.message.contains("runaway 上限"), "消息应点明上限");
    }

    #[test]
    fn project_workflow_chain_resumes_skipping_completed_nodes() {
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let dir = std::env::temp_dir().join(format!("chain-resume-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        let index_path = dir.join("codex-index.json");
        fs::create_dir_all(&dir).expect("fixture dir");
        bootstrap_project_workflow_at(&path, &fixture_project(test_root)).expect("workflow");
        submit_chain_test_workflow(&path, test_root, "续跑链");
        let wid = chain_test_workflow_id_by_title(&path, "续跑链");
        let runner = chain_test_runner();
        DISPATCH_READBACK_NATIVE_READ_COUNT.with(|c| c.set(0));
        let index = fixture_multi_thread_index(test_root, &["thread-1", "thread-2", "thread-3"]);
        // 第一跑：上限 1 → a 完成、链 stopped
        let run1 = run_project_workflow_chain_at(
            &path,
            &index,
            &index_path,
            &runner,
            &ProjectWorkflowChainRunRequest {
                project_root: test_root.to_string(),
                workflow_id: wid.clone(),
                max_nodes: Some(1),
            },
        )
        .expect("run1");
        assert_eq!(run1.state, "stopped");
        let a_node_id = optional_string_from(&run1.nodes[0], "node_id").unwrap();
        let a_dispatch_id_1 = optional_string_from(&run1.nodes[0], "dispatch_id");
        assert!(a_dispatch_id_1.is_some(), "a 应已派发并记 dispatch_id");
        // 第二跑：无上限 → 复用同一条链运行记录（断点续），跳过 a，跑完 b、c
        let run2 = run_project_workflow_chain_at(
            &path,
            &index,
            &index_path,
            &runner,
            &ProjectWorkflowChainRunRequest {
                project_root: test_root.to_string(),
                workflow_id: wid.clone(),
                max_nodes: None,
            },
        )
        .expect("run2");
        assert_eq!(run2.state, "completed");
        assert_eq!(run2.dispatched_count, 3, "三节点最终都完成");
        assert_eq!(
            run1.chain_run_id, run2.chain_run_id,
            "断点续：复用同一条链运行记录"
        );
        let runs = read_json_file(&path)["workflow_chain_runs"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        assert_eq!(runs.len(), 1, "续跑不该新建第二条记录");
        // a 没被重跑：dispatch_id 不变
        let a_dispatch_id_2 = run2
            .nodes
            .iter()
            .find(|n| optional_string_from(n, "node_id").as_deref() == Some(a_node_id.as_str()))
            .and_then(|n| optional_string_from(n, "dispatch_id"));
        assert_eq!(
            a_dispatch_id_1, a_dispatch_id_2,
            "已完成的 a 应被跳过、不重跑"
        );
    }

    #[test]
    fn project_workflow_chain_interrupts_at_node_boundary_on_stop_flag() {
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let dir = std::env::temp_dir().join(format!("chain-stop-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        let index_path = dir.join("codex-index.json");
        fs::create_dir_all(&dir).expect("fixture dir");
        bootstrap_project_workflow_at(&path, &fixture_project(test_root)).expect("workflow");
        submit_chain_test_workflow(&path, test_root, "停链测试");
        let wid = chain_test_workflow_id_by_title(&path, "停链测试");

        // 模拟「跑第一个节点时用户点了停」：runner 每次跑完顺手把 running 链的 stop_requested 置真，
        // 写回状态文件。下个节点边界 controller 重读 → 看到 stop → 停。
        struct StopDuringRunner {
            state_path: PathBuf,
            stats: CodexDispatchReadbackStats,
        }
        impl CodexResumeRunner for StopDuringRunner {
            fn resume_with_options(
                &self,
                _thread_id: &str,
                _prompt: &str,
                last_message_path: &Path,
                _options: &CodexResumeRequestOptions,
            ) -> Result<(CodexResumeRunResult, WorkflowNodeDispatchExecutionOptions), String>
            {
                if let Some(parent) = last_message_path.parent() {
                    fs::create_dir_all(parent).ok();
                }
                fs::write(last_message_path, "STOP_DURING_OK").ok();
                let mut v = read_workflow_state_value(&self.state_path)?;
                if let Some(runs) = v
                    .get_mut("workflow_chain_runs")
                    .and_then(Value::as_array_mut)
                {
                    if let Some(run) = runs
                        .iter_mut()
                        .find(|r| optional_string_from(r, "state").as_deref() == Some("running"))
                    {
                        run["stop_requested"] = json!(true);
                    }
                }
                write_validated_workflow_state(&self.state_path, &v)?;
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
        let runner = StopDuringRunner {
            state_path: path.clone(),
            stats: CodexDispatchReadbackStats {
                transcript_event_count: 2,
                transcript_target_hits: 1,
            },
        };
        DISPATCH_READBACK_NATIVE_READ_COUNT.with(|c| c.set(0));
        let index = fixture_multi_thread_index(test_root, &["thread-1", "thread-2", "thread-3"]);
        let result = run_project_workflow_chain_at(
            &path,
            &index,
            &index_path,
            &runner,
            &ProjectWorkflowChainRunRequest {
                project_root: test_root.to_string(),
                workflow_id: wid.clone(),
                max_nodes: None,
            },
        )
        .expect("chain should stop mid-way");
        assert_eq!(result.state, "stopped", "收到停 → 节点边界停");
        assert_eq!(result.dispatched_count, 1, "只完成第一个，停在边界");
        assert!(result.message.contains("停链请求"));
        assert!(audit_has(&path, "workflow_chain_run_stopped"));
    }

    #[test]
    fn stop_project_workflow_chain_sets_flag_and_errors_when_none_running() {
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let dir = std::env::temp_dir().join(format!("chain-stopcmd-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        fs::create_dir_all(&dir).expect("fixture dir");
        bootstrap_project_workflow_at(&path, &fixture_project(test_root)).expect("workflow");
        submit_chain_test_workflow(&path, test_root, "停命令链");
        let wid = chain_test_workflow_id_by_title(&path, "停命令链");
        // 没有 running 链 → 停链报清错
        let none = stop_project_workflow_chain_at(
            &path,
            &ProjectWorkflowChainStopRequest {
                project_root: test_root.to_string(),
                workflow_id: wid.clone(),
            },
        );
        assert!(none.is_err(), "无 running 链时停链应报错");
        // 注入一条 running 链记录 → 停链置 stop_requested + 记审计
        let mut v = read_workflow_state_value(&path).unwrap();
        ensure_array_mut(&mut v, "workflow_chain_runs")
            .unwrap()
            .push(json!({
              "chain_run_id": "cr-stopcmd-1",
              "project_id": project_id(test_root),
              "workflow_id": wid,
              "state": "running",
              "stop_requested": false,
              "max_nodes": 3,
              "started_at": "t0",
              "ended_at": Value::Null,
              "nodes": []
            }));
        write_validated_workflow_state(&path, &v).unwrap();
        let ok = stop_project_workflow_chain_at(
            &path,
            &ProjectWorkflowChainStopRequest {
                project_root: test_root.to_string(),
                workflow_id: wid.clone(),
            },
        )
        .expect("有 running 链 → 停链成功");
        assert_eq!(ok.chain_run_id, "cr-stopcmd-1");
        let runs = read_json_file(&path)["workflow_chain_runs"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        let run = runs
            .iter()
            .find(|r| optional_string_from(r, "chain_run_id").as_deref() == Some("cr-stopcmd-1"))
            .unwrap();
        assert_eq!(
            run.get("stop_requested").and_then(Value::as_bool),
            Some(true)
        );
        assert!(audit_has(&path, "workflow_chain_stop_requested"));
    }

    // 回归：跑过链/节点后会累积 canvas_run 临时 work_item（无任务包 artifact）。submit 的运行性检查
    // 曾遍历它们 → 全 blocked（缺模型/读写范围/验收/会话）→ 跑过一次后再也存不了草案（真机踩到）。
    // 修后：剔除 canvas_run 临时件再查，存草案不受其影响、编辑的 prompt 能写进去。
    #[test]
    fn submit_project_workflow_draft_not_blocked_by_canvas_run_work_items() {
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let dir =
            std::env::temp_dir().join(format!("submit-after-run-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        fs::create_dir_all(&dir).expect("fixture dir");
        bootstrap_project_workflow_at(&path, &fixture_project(test_root)).expect("workflow");
        let make_req = |wid: Option<String>, sub_prompt: &str| SubmitProjectWorkflowDraftRequest {
            project_root: test_root.to_string(),
            workflow_id: wid,
            title: "存盘回归".to_string(),
            nodes: vec![
                json!({"id":"a","kind":"director","label":"主管","position":{"x":1,"y":1},
                       "data":{"prompt":"P","session":{"mode":"resume","thread_id":"t1"}}}),
                json!({"id":"b","kind":"subagent","label":"B","position":{"x":2,"y":2},
                       "data":{"prompt":sub_prompt,"session":{"mode":"resume","thread_id":"t2"}}}),
            ],
            edges: vec![json!({"id":"e1","from":"a","to":"b"})],
        };
        submit_project_workflow_draft_at(&path, &make_req(None, "")).expect("初次创建应成功");
        let wid = chain_test_workflow_id_by_title(&path, "存盘回归");
        // 注入一个 canvas_run 临时 work_item（无 artifact）——模拟跑过链/节点后累积的临时件。
        let mut v = read_workflow_state_value(&path).unwrap();
        ensure_array_mut(&mut v, "work_items").unwrap().push(json!({
          "work_item_id": "work-item:canvas-run-temp-1",
          "project_id": project_id(test_root),
          "workflow_id": wid,
          "title": "临时跑件",
          "state": "ready_for_review",
          "source_kind": "canvas_run",
          "assigned_role_id": "codex-dev",
          "created_at": "t", "updated_at": "t", "warnings": []
        }));
        write_validated_workflow_state(&path, &v).unwrap();
        // 编辑再提交（update）：有了 canvas_run 临时件也应成功（不被运行性检查挡）。
        submit_project_workflow_draft_at(&path, &make_req(Some(wid.clone()), "新填的子agent提示"))
            .expect("有 canvas_run 临时件也应能存草案");
        // 编辑的 subagent prompt 真写进去了。
        let after = read_json_file(&path);
        let sub_prompt = after["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| {
                optional_string_from(n, "workflow_id").as_deref() == Some(wid.as_str())
                    && optional_string_from(n, "node_type").as_deref() == Some("subagent")
            })
            .and_then(|n| n.get("canvas_payload"))
            .and_then(|p| p.get("data"))
            .and_then(|d| optional_string_from(d, "prompt"));
        assert_eq!(
            sub_prompt.as_deref(),
            Some("新填的子agent提示"),
            "编辑的 prompt 应存住"
        );
    }

    // 积压清理回归：每跑一次节点自动建一个 canvas_run 临时 work_item + 一条绑定，跑多了会累积。
    // 修后：同 (workflow, node) 的旧 canvas_run 件连同绑定一起剔、封顶 1。dispatch 审计留痕不动。
    #[test]
    fn canvas_run_temp_work_items_and_bindings_capped_per_node() {
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let dir = std::env::temp_dir().join(format!("canvas-run-cap-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        let index_path = dir.join("codex-index.json");
        fs::create_dir_all(&dir).expect("fixture dir");
        bootstrap_project_workflow_at(&path, &fixture_project(test_root)).expect("workflow");
        submit_project_workflow_draft_at(
            &path,
            &SubmitProjectWorkflowDraftRequest {
                project_root: test_root.to_string(),
                workflow_id: None,
                title: "封顶测试".to_string(),
                nodes: vec![
                    json!({"id":"a","kind":"director","label":"主管","position":{"x":1,"y":1},
                           "data":{"prompt":"P","session":{"mode":"resume","thread_id":"thread-cap"}}}),
                    json!({"id":"b","kind":"subagent","label":"B","position":{"x":2,"y":2},
                           "data":{"session":{"mode":"resume","thread_id":"thread-cap"},"prompt":"做事","sandbox":"workspace-write"}}),
                ],
                edges: vec![],
            },
        )
        .expect("create");
        let wid = chain_test_workflow_id_by_title(&path, "封顶测试");
        let node_id = optional_string_from(
            read_json_file(&path)["nodes"]
                .as_array()
                .unwrap()
                .iter()
                .find(|n| {
                    optional_string_from(n, "workflow_id").as_deref() == Some(wid.as_str())
                        && optional_string_from(n, "node_type").as_deref() == Some("subagent")
                })
                .unwrap(),
            "node_id",
        )
        .unwrap();
        let runner = PermissiveExperimentRunner {
            stats: CodexDispatchReadbackStats {
                transcript_event_count: 2,
                transcript_target_hits: 1,
            },
        };
        DISPATCH_READBACK_NATIVE_READ_COUNT.with(|c| c.set(0));
        let index = fixture_dispatch_index(test_root, "thread-cap");
        let req = ProjectWorkflowNodeRunRequest {
            project_root: test_root.to_string(),
            node_id: node_id.clone(),
            work_item_id: String::new(),
            workflow_id: Some(wid.clone()),
        };
        execute_project_workflow_node_at(&path, &index, &index_path, &runner, &req).expect("run1");
        execute_project_workflow_node_at(&path, &index, &index_path, &runner, &req).expect("run2");
        let v = read_json_file(&path);
        let wi_count = v["work_items"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|wi| {
                optional_string_from(wi, "source_kind").as_deref() == Some("canvas_run")
                    && optional_string_from(wi, "workflow_id").as_deref() == Some(wid.as_str())
                    && optional_string_from(wi, "origin_node_id").as_deref()
                        == Some(node_id.as_str())
            })
            .count();
        assert_eq!(
            wi_count, 1,
            "同节点 canvas_run 临时件应封顶 1，实际 {wi_count}"
        );
        let bind_count = v["workflow_node_session_bindings"]
            .as_array()
            .map(|b| {
                b.iter()
                    .filter(|x| {
                        optional_string_from(x, "node_id").as_deref() == Some(node_id.as_str())
                    })
                    .count()
            })
            .unwrap_or(0);
        assert_eq!(bind_count, 1, "同节点会话绑定应封顶 1，实际 {bind_count}");
    }

    // ===== S1 执行层合一：B 画布派发过 A 强闸（option A：path-lock 作授权、guard 取执行安全子集） =====

    // 铁律（§3）：authorized_for_real_runner ⟹ authorization_complete ⟹ path-lock 命中。
    // B 把 authorization_complete 只赋 path-lock 命中 → 非测试项目必拦、测试项目+其余满足才放。
    #[test]
    fn s1_gate_iron_law_path_lock_required_for_authorized() {
        let input = |authz: bool| crate::real_execution_command::RealExecutionCommandGateInput {
            command_name: "execute_project_workflow_node",
            command_family: "workflow_real_execution",
            operation_id: "resume",
            h5_unified_product_command: true,
            authorization_complete: authz,
            user_rejected: false,
            duplicate_blocked: false,
            guard_blocked: false,
            diagnostics_blocked: false,
            stale_memory_blocked: false,
            readback_required: true,
        };
        // 非测试项目 → path-lock 不命中
        assert!(!workflow_engine_test_project_unsealed(
            "/tmp/not-test-project"
        ));
        assert!(!workflow_engine_test_project_unsealed(
            "/Users/yoyi/workspace/product-line"
        ));
        assert!(
            !crate::real_execution_command::decide_real_execution_command(input(false))
                .runner_call_allowed,
            "铁律：path-lock 不命中（authorization_complete=false）→ 不授权"
        );
        // 测试项目 → path-lock 命中；其余判据满足 → 授权
        assert!(workflow_engine_test_project_unsealed(
            WORKFLOW_ENGINE_TEST_PROJECT_ROOT
        ));
        assert!(
            crate::real_execution_command::decide_real_execution_command(input(true))
                .runner_call_allowed,
            "测试项目 + 各判据满足 → 授权"
        );
    }

    // duplicate_blocked：只数 "running"（真正执行中），不数 "prepared"（每次派发残留的 orphan）。
    #[test]
    fn s1_has_inflight_dispatch_counts_running_only() {
        let with_state = |state: &str| json!({"workflow_node_dispatches":[{"workflow_id":"wf","node_id":"wf:node:x","state":state}]});
        assert!(has_inflight_dispatch(
            &with_state("running"),
            "wf",
            "wf:node:x"
        ));
        assert!(
            !has_inflight_dispatch(&with_state("prepared"), "wf", "wf:node:x"),
            "prepared 是 orphan 残留、不算在飞（否则误拦合法重跑）"
        );
        assert!(!has_inflight_dispatch(
            &with_state("completed"),
            "wf",
            "wf:node:x"
        ));
        assert!(!has_inflight_dispatch(
            &with_state("failed"),
            "wf",
            "wf:node:x"
        ));
        assert!(
            !has_inflight_dispatch(&with_state("running"), "wf", "wf:node:other"),
            "不同 node 不算"
        );
    }

    // guard_blocked（option A）：A 的 3 道授权 reason 不计入 B；执行安全 reason 照计。
    #[test]
    fn s1_canvas_node_guard_blocked_excludes_authorization_reasons() {
        let mk = |reasons: Vec<String>| CodexLocalExecutionGuard {
            guard_version: 1,
            status: "x".to_string(),
            severity: "x".to_string(),
            blocks_execution: true,
            allows_dry_run: false,
            requires_user_confirmation: false,
            duplicate_running_attempt: false,
            command_plan: None,
            reasons,
            required_fixes: vec![],
            warnings: vec![],
        };
        // 纯授权 reason → 不计入 B
        assert!(!canvas_node_guard_blocked(&mk(vec![
            "user_confirmation_required".to_string(),
            "authorization_scope_missing".to_string(),
            "audit_ref_missing".to_string(),
        ])));
        // 夹一条执行安全 reason → 计入
        assert!(canvas_node_guard_blocked(&mk(vec![
            "user_confirmation_required".to_string(),
            "check_paths_failed".to_string(),
        ])));
    }

    // guard 安全子集真拦：prompt 含密钥词 → secret_deny_list 触发（执行安全 reason）→ guard_blocked。
    #[test]
    fn s1_guard_blocks_prompt_with_secret_but_allows_clean() {
        let roots = vec![WORKFLOW_ENGINE_TEST_PROJECT_ROOT.to_string()];
        let secret_req = build_canvas_node_codex_local_request(
            WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
            "project:test",
            "wf",
            "wf:node:x",
            "thread-1",
            "wi-1",
            "请读取 .env 里的 token 再说",
            "workspace-write",
            &roots,
        );
        let g1 = crate::codex_local_runner::inspect_codex_local_execution_guard(&secret_req);
        assert!(
            canvas_node_guard_blocked(&g1),
            "含密钥词的 prompt 应被执行安全子集拦：{:?}",
            g1.reasons
        );
        let clean_req = build_canvas_node_codex_local_request(
            WORKFLOW_ENGINE_TEST_PROJECT_ROOT,
            "project:test",
            "wf",
            "wf:node:x",
            "thread-1",
            "wi-1",
            "建一个文本文档",
            "workspace-write",
            &roots,
        );
        let g2 = crate::codex_local_runner::inspect_codex_local_execution_guard(&clean_req);
        assert!(
            !canvas_node_guard_blocked(&g2),
            "干净 prompt 不应被执行安全子集拦（剩的只是被排除的授权 reason）：{:?}",
            g2.reasons
        );
    }

    // 端到端：节点已有在飞 running 派发 → execute_project_workflow_node_at 被 duplicate 闸拦、不起 runner。
    #[test]
    fn s1_gate_blocks_dispatch_when_node_has_inflight_running() {
        let test_root = WORKFLOW_ENGINE_TEST_PROJECT_ROOT;
        let dir = std::env::temp_dir().join(format!("s1-dup-{}", unix_timestamp_string()));
        let path = dir.join("workflow-state.v0.json");
        let index_path = dir.join("codex-index.json");
        fs::create_dir_all(&dir).expect("fixture dir");
        bootstrap_project_workflow_at(&path, &fixture_project(test_root)).expect("workflow");
        submit_chain_test_workflow(&path, test_root, "S1去重");
        let wid = chain_test_workflow_id_by_title(&path, "S1去重");
        let node_id = format!("{wid}:node:{}", stable_id("a"));
        // 注入一条该节点的 "running" 派发（模拟在飞）。
        let mut v = read_workflow_state_value(&path).unwrap();
        ensure_array_mut(&mut v, "workflow_node_dispatches")
            .unwrap()
            .push(json!({
              "dispatch_id":"d-inflight","workflow_id":wid,"node_id":node_id,"state":"running"
            }));
        write_validated_workflow_state(&path, &v).unwrap();
        let runner = chain_test_runner();
        DISPATCH_READBACK_NATIVE_READ_COUNT.with(|c| c.set(0));
        let index = fixture_multi_thread_index(test_root, &["thread-1", "thread-2", "thread-3"]);
        let req = ProjectWorkflowNodeRunRequest {
            project_root: test_root.to_string(),
            node_id: node_id.clone(),
            work_item_id: String::new(),
            workflow_id: Some(wid.clone()),
        };
        let err = execute_project_workflow_node_at(&path, &index, &index_path, &runner, &req)
            .expect_err("在飞 running 应被 duplicate 闸拦");
        assert!(
            err.contains("duplicate_blocked"),
            "应被 duplicate 闸拦、不起 runner：{err}"
        );
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
            workbench_bound: false,
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
