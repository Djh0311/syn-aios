// Station 1 supervisor MCP role. It owns an isolated sidecar and never writes workflow chain state.

use super::{McpServerConfig, SupervisorQuotaLimits};
use crate::CodexResumeRunner;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const STORE_SCHEMA_VERSION: &str = "supervisor_orchestrator.v1";
const SIDECAR_NAME: &str = "supervisor-orchestrator.v1.json";
const LOCK_NAME: &str = ".supervisor-orchestrator.v1.lock";
const LOCK_RETRY_COUNT: usize = 5;
const LOCK_RETRY_DELAY: Duration = Duration::from_millis(100);
const ACTOR: &str = "supervisor_orchestrator";

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
struct SupervisorStore {
    #[serde(default)]
    schema_version: String,
    #[serde(default)]
    revision: i64,
    #[serde(default)]
    updated_at_ms: i64,
    #[serde(default)]
    sessions: Vec<SupervisorSession>,
    #[serde(default)]
    audit_events: Vec<SupervisorAuditEvent>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
struct SupervisorSession {
    #[serde(default)]
    run_id: String,
    #[serde(default)]
    workers: Vec<SupervisorWorker>,
    #[serde(default)]
    final_marks: Vec<SupervisorFinalMark>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
struct SupervisorWorker {
    #[serde(default)]
    worker_id: String,
    #[serde(default)]
    project_root: String,
    #[serde(default)]
    workflow_id: String,
    #[serde(default)]
    node_id: String,
    #[serde(default)]
    work_item_id: String,
    #[serde(default)]
    authorization_id: String,
    #[serde(default)]
    native_thread_id: String,
    #[serde(default)]
    dispatch_id: String,
    #[serde(default)]
    allowed_write: Vec<String>,
    #[serde(default)]
    state: String,
    #[serde(default)]
    started_at_ms: i64,
    #[serde(default)]
    follow_up_count: usize,
    #[serde(default)]
    last_report: Option<Value>,
    #[serde(default)]
    last_result_summary: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
struct SupervisorFinalMark {
    #[serde(default)]
    authorization_id: String,
    #[serde(default)]
    project_root: String,
    #[serde(default)]
    workflow_id: String,
    #[serde(default)]
    verdict: String,
    #[serde(default)]
    reason: String,
    #[serde(default)]
    created_at_ms: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
struct SupervisorAuditEvent {
    #[serde(default)]
    event_id: String,
    #[serde(default)]
    actor: String,
    #[serde(default)]
    run_id: String,
    #[serde(default)]
    tool: String,
    #[serde(default)]
    parameter_summary: String,
    #[serde(default)]
    result_summary: String,
    #[serde(default)]
    created_at_ms: i64,
}

#[derive(Debug, Clone)]
struct DispatchInput {
    project_root: String,
    workflow_id: String,
    authorization_id: String,
    node_id: String,
    work_item_id: String,
    allowed_write: Vec<String>,
}

#[derive(Debug, Clone, Default)]
struct WorkerLaunch {
    worker_id: String,
    native_thread_id: String,
    dispatch_id: String,
    state: String,
    initial_report: Option<Value>,
    result_summary: String,
}

trait WorkerInvoker {
    fn dispatch(
        &self,
        config: &McpServerConfig,
        input: &DispatchInput,
    ) -> Result<WorkerLaunch, String>;
    fn follow_up(
        &self,
        config: &McpServerConfig,
        worker: &SupervisorWorker,
        prompt: &str,
    ) -> Result<String, String>;
}

struct WorkbenchWorkerInvoker;

impl WorkerInvoker for WorkbenchWorkerInvoker {
    fn dispatch(
        &self,
        config: &McpServerConfig,
        input: &DispatchInput,
    ) -> Result<WorkerLaunch, String> {
        if !crate::workflow_engine_test_project_unsealed(&input.project_root) {
            return Err(crate::legacy_product_command_blocked_message(
                "execute_project_workflow_node",
            ));
        }
        let app_state = crate::AppState::new();
        let index = crate::read_index(&app_state)?;
        let runner = crate::codex_local_runner::RealWorkflowNodeCodexRunner;
        let result = crate::execute_project_workflow_node_at(
            workflow_state_path(config)?,
            &index,
            &crate::codex_db::default_state_db_path(),
            &runner,
            &crate::ProjectWorkflowNodeRunRequest {
                project_root: input.project_root.clone(),
                node_id: input.node_id.clone(),
                work_item_id: input.work_item_id.clone(),
                workflow_id: Some(input.workflow_id.clone()),
            },
        )?;
        if result.dispatch.plan_authorization_id.as_deref() != Some(input.authorization_id.as_str())
        {
            return Err("现成派发返回的授权段与主管请求不一致，已拒绝纳入主管账本".to_string());
        }
        Ok(WorkerLaunch {
            worker_id: result.dispatch.dispatch_id.clone(),
            native_thread_id: result.dispatch.native_thread_id,
            dispatch_id: result.dispatch.dispatch_id,
            state: result.dispatch.state,
            initial_report: None,
            result_summary: result.message,
        })
    }

    fn follow_up(
        &self,
        config: &McpServerConfig,
        worker: &SupervisorWorker,
        prompt: &str,
    ) -> Result<String, String> {
        if !crate::workflow_engine_test_project_unsealed(&worker.project_root) {
            return Err(crate::legacy_product_command_blocked_message(
                "execute_project_workflow_node",
            ));
        }
        let value = crate::read_workflow_state_value(workflow_state_path(config)?)?;
        let dispatch = crate::find_workflow_node_dispatch(&value, &worker.dispatch_id)
            .ok_or_else(|| "主管账本对应的节点派发记录不存在，拒绝追问".to_string())?;
        let prompt_kind = crate::optional_string_from(dispatch, "prompt_kind")
            .ok_or_else(|| "节点派发记录缺 prompt_kind，拒绝追问".to_string())?;
        let user_reviewed_instruction = if prompt_kind == "user_reviewed_instruction" {
            Some(crate::user_reviewed_instruction_input_from_value(
                dispatch
                    .get("user_reviewed_instruction")
                    .ok_or_else(|| "节点派发记录缺用户审核指令，拒绝追问".to_string())?,
            )?)
        } else {
            None
        };
        let app_state = crate::AppState::new();
        let index = crate::read_index(&app_state)?;
        let context = crate::workflow_node_dispatch_context(
            workflow_state_path(config)?,
            &index,
            &crate::WorkflowNodeDispatchPrepareRequest {
                project_root: worker.project_root.clone(),
                node_id: worker.node_id.clone(),
                work_item_id: worker.work_item_id.clone(),
                prompt_kind,
                user_reviewed_instruction,
            },
        )?;
        let authorization = crate::inspect_workflow_node_dispatch_authorization(
            workflow_state_path(config)?,
            &context,
        )?;
        crate::ensure_authorized_for_prepare(&authorization)?;
        if authorization.authorization_id.as_deref() != Some(worker.authorization_id.as_str()) {
            return Err("追问时授权段已变化，拒绝跨授权段续跑".to_string());
        }
        if context.native_thread_id != worker.native_thread_id {
            return Err("追问时绑定会话与初始 worker 不一致，拒绝跨 thread resume".to_string());
        }
        let sidecar = sidecar_path(config)?;
        let parent = sidecar
            .parent()
            .ok_or_else(|| "主管账本路径缺父目录，拒绝写追问回传".to_string())?;
        let last_message_path = parent.join(format!(
            "supervisor-follow-up-{}.txt",
            stable_fragment(&worker.worker_id)
        ));
        let runner = crate::codex_local_runner::RealWorkflowNodeCodexRunner;
        let (result, _) = runner.resume_with_options(
            &context.native_thread_id,
            prompt,
            &last_message_path,
            &crate::codex_resume_options_for_context(&context)?,
        )?;
        if result.exit_code != 0 {
            return Err(result
                .stderr_summary
                .unwrap_or_else(|| format!("追问 worker 退出码 {}", result.exit_code)));
        }
        Ok("已通过现成 runner 在原绑定会话续问；结果待口供工具读取。".to_string())
    }
}

pub fn list_tools() -> Value {
    json!({
        "tools": [
            tool_def("dispatch_worker", "在已授权、已绑定且 path-lock 通过后派发 worker", json!({
                "type": "object", "properties": {
                    "project_root": {"type": "string"}, "workflow_id": {"type": "string"},
                    "authorization_id": {"type": "string"}, "node_id": {"type": "string"},
                    "work_item_id": {"type": "string"}, "allowed_write": {"type": "array", "items": {"type": "string"}}
                }, "required": ["project_root", "workflow_id", "authorization_id", "node_id", "work_item_id", "allowed_write"], "additionalProperties": false
            })),
            tool_def("read_worker_report", "只读投影 worker 结构化口供", json!({
                "type": "object", "properties": {"worker_id": {"type": "string"}}, "required": ["worker_id"], "additionalProperties": false
            })),
            tool_def("follow_up_worker", "在同授权段、同 thread、现成 runner 下追问 worker", json!({
                "type": "object", "properties": {"worker_id": {"type": "string"}, "prompt": {"type": "string"}}, "required": ["worker_id", "prompt"], "additionalProperties": false
            })),
            tool_def("wait_for_worker", "读取 worker 当前状态，不管理或终止进程", json!({
                "type": "object", "properties": {"worker_id": {"type": "string"}}, "required": ["worker_id"], "additionalProperties": false
            })),
            tool_def("read_key_file", "在授权允许读取根内读取关键文本文件", json!({
                "type": "object", "properties": {"project_root": {"type": "string"}, "workflow_id": {"type": "string"}, "authorization_id": {"type": "string"}, "path": {"type": "string"}}, "required": ["project_root", "workflow_id", "authorization_id", "path"], "additionalProperties": false
            })),
            tool_def("final_mark", "写主管 advisory 终标意见，不改 workflow chain 状态", json!({
                "type": "object", "properties": {"project_root": {"type": "string"}, "workflow_id": {"type": "string"}, "authorization_id": {"type": "string"}, "verdict": {"type": "string", "enum": ["pass", "needs_rework"]}, "reason": {"type": "string"}}, "required": ["project_root", "workflow_id", "authorization_id", "verdict", "reason"], "additionalProperties": false
            })),
            tool_def("report_user", "返回用户报告文本并写主管账本审计，不作用户决定", json!({
                "type": "object", "properties": {"project_root": {"type": "string"}, "workflow_id": {"type": "string"}, "authorization_id": {"type": "string"}, "message": {"type": "string"}}, "required": ["project_root", "workflow_id", "authorization_id", "message"], "additionalProperties": false
            }))
        ]
    })
}

pub fn call_tool(config: &McpServerConfig, params: Value) -> Result<Value, String> {
    call_tool_with_invoker(config, params, &WorkbenchWorkerInvoker)
}

fn call_tool_with_invoker(
    config: &McpServerConfig,
    params: Value,
    invoker: &dyn WorkerInvoker,
) -> Result<Value, String> {
    let name = require_string(&params, "name")?;
    let arguments = params
        .get("arguments")
        .cloned()
        .unwrap_or(Value::Object(Default::default()));
    let result = match name.as_str() {
        "dispatch_worker" => dispatch_worker(config, &arguments, invoker),
        "read_worker_report" => read_worker_report(config, &arguments),
        "follow_up_worker" => follow_up_worker(config, &arguments, invoker),
        "wait_for_worker" => wait_for_worker(config, &arguments),
        "read_key_file" => read_key_file(config, &arguments),
        "final_mark" => final_mark(config, &arguments),
        "report_user" => report_user(config, &arguments),
        _ => Err(format!("主管编排角色不认识工具：{name}")),
    };
    let result_summary = match &result {
        Ok(value) => summary_of_value(value),
        Err(error) => format!(
            "denied: {}",
            crate::run_error_translation::humanize_error_for_display(error)
        ),
    };
    append_audit(
        config,
        &name,
        &parameter_summary(&arguments),
        &result_summary,
    )?;
    result
        .map(tool_result)
        .map_err(|error| crate::run_error_translation::humanize_error_for_display(&error))
}

fn dispatch_worker(
    config: &McpServerConfig,
    args: &Value,
    invoker: &dyn WorkerInvoker,
) -> Result<Value, String> {
    let input = DispatchInput {
        project_root: require_string(args, "project_root")?,
        workflow_id: require_string(args, "workflow_id")?,
        authorization_id: require_string(args, "authorization_id")?,
        node_id: require_string(args, "node_id")?,
        work_item_id: require_string(args, "work_item_id")?,
        allowed_write: require_string_array(args, "allowed_write")?,
    };
    if !crate::workflow_engine_test_project_unsealed(&input.project_root) {
        return Err(crate::legacy_product_command_blocked_message(
            "execute_project_workflow_node",
        ));
    }
    check_authorization(
        &input.project_root,
        &input.workflow_id,
        &input.authorization_id,
        &input.node_id,
        &input.allowed_write,
        config,
    )?;
    let reservation_id = reserve_dispatch(config, &input)?;
    match invoker.dispatch(config, &input) {
        Ok(launch) => {
            complete_dispatch(config, &reservation_id, &launch)?;
            Ok(
                json!({"worker_id": launch.worker_id, "state": launch.state, "dispatch_id": launch.dispatch_id}),
            )
        }
        Err(error) => {
            fail_dispatch(config, &reservation_id, &error)?;
            Err(error)
        }
    }
}

fn read_worker_report(config: &McpServerConfig, args: &Value) -> Result<Value, String> {
    let worker = find_worker(config, &require_string(args, "worker_id")?)?;
    check_authorization(
        &worker.project_root,
        &worker.workflow_id,
        &worker.authorization_id,
        &worker.node_id,
        &worker.allowed_write,
        config,
    )?;
    if let Some(report) = worker.last_report {
        return Ok(report);
    }
    let value = crate::read_workflow_state_value(workflow_state_path(config)?)?;
    let event = value
        .get("audit_events")
        .and_then(Value::as_array)
        .and_then(|events| {
            events.iter().rev().find(|event| {
                crate::optional_string_from(event, "event_type").as_deref()
                    == Some("worker_structured_report_recorded")
                    && crate::optional_string_from(event, "dispatch_id").as_deref()
                        == Some(worker.dispatch_id.as_str())
            })
        })
        .ok_or_else(|| "尚无该 worker 的结构化口供；不会猜测或读取完整 transcript".to_string())?;
    Ok(json!({
        "worker_id": worker.worker_id,
        "dispatch_id": worker.dispatch_id,
        "acceptance_status": event.get("acceptance_status").cloned().unwrap_or(Value::Null),
        "executed_what": event.get("executed_what").cloned().unwrap_or(Value::Null),
        "changed_what": event.get("changed_what").cloned().unwrap_or(Value::Null),
        "evidence": event.get("evidence").cloned().unwrap_or(Value::Null),
        "direction_risks": event.get("direction_risks").cloned().unwrap_or(Value::Null),
        "follow_up_suggestions": event.get("follow_up_suggestions").cloned().unwrap_or(Value::Null)
    }))
}

fn follow_up_worker(
    config: &McpServerConfig,
    args: &Value,
    invoker: &dyn WorkerInvoker,
) -> Result<Value, String> {
    let worker_id = require_string(args, "worker_id")?;
    let prompt = require_string(args, "prompt")?;
    if prompt.trim().is_empty() {
        return Err("追问内容不能为空".to_string());
    }
    let worker = find_worker(config, &worker_id)?;
    if !crate::workflow_engine_test_project_unsealed(&worker.project_root) {
        return Err(crate::legacy_product_command_blocked_message(
            "execute_project_workflow_node",
        ));
    }
    check_authorization(
        &worker.project_root,
        &worker.workflow_id,
        &worker.authorization_id,
        &worker.node_id,
        &worker.allowed_write,
        config,
    )?;
    reserve_follow_up(config, &worker_id)?;
    let summary = invoker.follow_up(config, &worker, &prompt)?;
    update_worker_result(config, &worker_id, "completed", &summary)?;
    Ok(json!({"worker_id": worker_id, "state": "completed", "summary": summary}))
}

fn wait_for_worker(config: &McpServerConfig, args: &Value) -> Result<Value, String> {
    let worker = find_worker(config, &require_string(args, "worker_id")?)?;
    check_authorization(
        &worker.project_root,
        &worker.workflow_id,
        &worker.authorization_id,
        &worker.node_id,
        &worker.allowed_write,
        config,
    )?;
    ensure_worker_within_runtime(config, &worker)?;
    Ok(json!({
        "worker_id": worker.worker_id,
        "state": worker.state,
        "follow_up_count": worker.follow_up_count,
        "last_result_summary": worker.last_result_summary
    }))
}

fn read_key_file(config: &McpServerConfig, args: &Value) -> Result<Value, String> {
    let project_root = require_string(args, "project_root")?;
    let workflow_id = require_string(args, "workflow_id")?;
    let authorization_id = require_string(args, "authorization_id")?;
    let path = PathBuf::from(require_string(args, "path")?);
    let authorization =
        active_authorization(config, &project_root, &workflow_id, &authorization_id)?;
    deny_sensitive_file(&path)?;
    let canonical_path = fs::canonicalize(&path)
        .map_err(|error| format!("关键文件不存在或不可读 {}：{error}", path.display()))?;
    let permitted = authorization.scope.allowed_read_roots.iter().any(|root| {
        fs::canonicalize(root)
            .map(|allowed| canonical_path.starts_with(allowed))
            .unwrap_or(false)
    });
    if !permitted {
        return Err("关键文件不在当前授权 allowed_read 根内，已拒绝读取".to_string());
    }
    let text = fs::read_to_string(&canonical_path)
        .map_err(|error| format!("读取关键文件失败 {}：{error}", canonical_path.display()))?;
    if text.len() > 64 * 1024 {
        return Err("关键文件超过 64KiB 投影上限，已拒绝返回完整内容".to_string());
    }
    Ok(json!({"path": canonical_path, "content": text}))
}

fn final_mark(config: &McpServerConfig, args: &Value) -> Result<Value, String> {
    let project_root = require_string(args, "project_root")?;
    let workflow_id = require_string(args, "workflow_id")?;
    let authorization_id = require_string(args, "authorization_id")?;
    let verdict = require_string(args, "verdict")?;
    let reason = require_string(args, "reason")?;
    if !matches!(verdict.as_str(), "pass" | "needs_rework") || reason.trim().is_empty() {
        return Err("终标只允许 pass / needs_rework，且 reason 不能为空".to_string());
    }
    active_authorization(config, &project_root, &workflow_id, &authorization_id)?;
    let created_at_ms = now_ms();
    update_store(config, "final-mark", |store| {
        session_mut(store, &config.run_id)
            .final_marks
            .push(SupervisorFinalMark {
                authorization_id: authorization_id.clone(),
                project_root: project_root.clone(),
                workflow_id: workflow_id.clone(),
                verdict: verdict.clone(),
                reason: reason.clone(),
                created_at_ms,
            });
        Ok(())
    })?;
    Ok(json!({
        "verdict": verdict,
        "reason": reason,
        "advisory_only": true,
        "workflow_chain_state_written": false
    }))
}

fn report_user(config: &McpServerConfig, args: &Value) -> Result<Value, String> {
    let project_root = require_string(args, "project_root")?;
    let workflow_id = require_string(args, "workflow_id")?;
    let authorization_id = require_string(args, "authorization_id")?;
    let message = require_string(args, "message")?;
    if message.trim().is_empty() {
        return Err("用户报告内容不能为空".to_string());
    }
    active_authorization(config, &project_root, &workflow_id, &authorization_id)?;
    Ok(json!({"message": message, "user_decision_written": false}))
}

fn tool_def(name: &str, description: &str, schema: Value) -> Value {
    json!({"name": name, "description": description, "inputSchema": schema})
}

fn tool_result(value: Value) -> Value {
    let text = serde_json::to_string_pretty(&value).unwrap_or_else(|_| value.to_string());
    json!({"content": [{"type": "text", "text": text}]})
}

fn require_string(value: &Value, key: &str) -> Result<String, String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .ok_or_else(|| format!("缺少非空字段 {key}"))
}

fn require_string_array(value: &Value, key: &str) -> Result<Vec<String>, String> {
    let items = value
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("缺少数组字段 {key}"))?;
    let mut result = Vec::with_capacity(items.len());
    for item in items {
        let path = item
            .as_str()
            .map(str::trim)
            .filter(|path| !path.is_empty())
            .ok_or_else(|| format!("{key} 只能包含非空字符串"))?;
        result.push(path.to_string());
    }
    Ok(result)
}

fn workflow_state_path(config: &McpServerConfig) -> Result<&Path, String> {
    config
        .supervisor_workflow_state_path
        .as_deref()
        .ok_or_else(|| "主管 MCP 缺 workflow state 路径".to_string())
}

fn quota_limits(config: &McpServerConfig) -> Result<SupervisorQuotaLimits, String> {
    config
        .supervisor_quota_limits
        .ok_or_else(|| "主管 MCP 缺显式配额配置".to_string())
}

fn sidecar_path(config: &McpServerConfig) -> Result<PathBuf, String> {
    crate::utils::store_paths::sidecar_path(workflow_state_path(config)?, SIDECAR_NAME, "主管编排")
}

fn empty_store(timestamp_ms: i64) -> SupervisorStore {
    SupervisorStore {
        schema_version: STORE_SCHEMA_VERSION.to_string(),
        revision: 0,
        updated_at_ms: timestamp_ms,
        sessions: vec![],
        audit_events: vec![],
    }
}

fn load_store(config: &McpServerConfig) -> Result<SupervisorStore, String> {
    let path = sidecar_path(config)?;
    if !path.exists() {
        return Ok(empty_store(now_ms()));
    }
    let text = fs::read_to_string(&path)
        .map_err(|error| format!("读取主管编排 sidecar 失败 {}：{error}", path.display()))?;
    let store: SupervisorStore = serde_json::from_str(&text).map_err(|error| {
        format!(
            "主管编排 sidecar JSON 损坏，已拒绝覆盖 {}：{error}",
            path.display()
        )
    })?;
    if store.schema_version != STORE_SCHEMA_VERSION || store.revision < 0 {
        return Err("主管编排 sidecar schema 或 revision 不合法，已拒绝覆盖".to_string());
    }
    Ok(store)
}

fn update_store<R>(
    config: &McpServerConfig,
    write_id: &str,
    update: impl FnOnce(&mut SupervisorStore) -> Result<R, String>,
) -> Result<R, String> {
    let sidecar = sidecar_path(config)?;
    let parent = sidecar
        .parent()
        .ok_or_else(|| "主管编排 sidecar 没有父目录".to_string())?;
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "创建主管编排 sidecar 目录失败 {}：{error}",
            parent.display()
        )
    })?;
    let _lock = StoreLock::acquire(&parent.join(LOCK_NAME), write_id)?;
    let mut store = load_store(config)?;
    let result = update(&mut store)?;
    store.revision += 1;
    store.updated_at_ms = now_ms();
    write_store_atomic(&sidecar, &store, write_id)?;
    Ok(result)
}

fn write_store_atomic(
    sidecar: &Path,
    store: &SupervisorStore,
    write_id: &str,
) -> Result<(), String> {
    let parent = sidecar
        .parent()
        .ok_or_else(|| "主管编排 sidecar 没有父目录".to_string())?;
    let temp = parent.join(format!(".{SIDECAR_NAME}.{}.tmp", stable_fragment(write_id)));
    let text = serde_json::to_string_pretty(store)
        .map_err(|error| format!("序列化主管编排 sidecar 失败：{error}"))?;
    let mut file = fs::File::create(&temp)
        .map_err(|error| format!("创建主管编排临时文件失败 {}：{error}", temp.display()))?;
    file.write_all(text.as_bytes())
        .map_err(|error| format!("写入主管编排临时文件失败 {}：{error}", temp.display()))?;
    file.sync_all()
        .map_err(|error| format!("同步主管编排临时文件失败 {}：{error}", temp.display()))?;
    fs::rename(&temp, sidecar).map_err(|error| {
        format!(
            "原子替换主管编排 sidecar 失败 {}：{error}",
            sidecar.display()
        )
    })?;
    if let Ok(dir) = fs::File::open(parent) {
        let _ = dir.sync_all();
    }
    Ok(())
}

struct StoreLock {
    path: PathBuf,
}

impl StoreLock {
    fn acquire(path: &Path, write_id: &str) -> Result<Self, String> {
        for retry in 0..=LOCK_RETRY_COUNT {
            match fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path)
            {
                Ok(mut file) => {
                    file.write_all(write_id.as_bytes()).map_err(|error| {
                        format!("写入主管编排 lock 失败 {}：{error}", path.display())
                    })?;
                    return Ok(Self {
                        path: path.to_path_buf(),
                    });
                }
                Err(error)
                    if error.kind() == std::io::ErrorKind::AlreadyExists
                        && retry < LOCK_RETRY_COUNT =>
                {
                    thread::sleep(LOCK_RETRY_DELAY);
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                    return Err(format!(
                        "supervisor_orchestrator_store_locked: {}；稍等几秒再点一次就好",
                        path.display()
                    ));
                }
                Err(error) => {
                    return Err(format!(
                        "创建主管编排 lock 失败 {}：{error}",
                        path.display()
                    ));
                }
            }
        }
        unreachable!("有限重试循环会在最后一次返回")
    }
}

impl Drop for StoreLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn session_mut<'a>(store: &'a mut SupervisorStore, run_id: &str) -> &'a mut SupervisorSession {
    if let Some(index) = store
        .sessions
        .iter()
        .position(|session| session.run_id == run_id)
    {
        return &mut store.sessions[index];
    }
    store.sessions.push(SupervisorSession {
        run_id: run_id.to_string(),
        ..SupervisorSession::default()
    });
    store.sessions.last_mut().expect("刚追加主管会话")
}

fn session<'a>(store: &'a SupervisorStore, run_id: &str) -> Option<&'a SupervisorSession> {
    store
        .sessions
        .iter()
        .find(|session| session.run_id == run_id)
}

fn find_worker(config: &McpServerConfig, worker_id: &str) -> Result<SupervisorWorker, String> {
    let store = load_store(config)?;
    session(&store, &config.run_id)
        .and_then(|session| {
            session
                .workers
                .iter()
                .find(|worker| worker.worker_id == worker_id)
        })
        .cloned()
        .ok_or_else(|| "主管当前会话没有该 worker，拒绝跨会话访问".to_string())
}

fn reserve_dispatch(config: &McpServerConfig, input: &DispatchInput) -> Result<String, String> {
    let limits = quota_limits(config)?;
    let reservation_id = format!(
        "reservation:{}:{}",
        stable_fragment(&config.run_id),
        crate::unix_timestamp_nanos()
    );
    let started_at_ms = now_ms();
    update_store(config, "reserve-dispatch", |store| {
        let session = session_mut(store, &config.run_id);
        let active = session
            .workers
            .iter()
            .filter(|worker| matches!(worker.state.as_str(), "reserved" | "running"))
            .count();
        if active >= limits.max_active_workers {
            return Err(format!(
                "主管 worker 并发配额已满（{}），拒绝再派发",
                limits.max_active_workers
            ));
        }
        session.workers.push(SupervisorWorker {
            worker_id: reservation_id.clone(),
            project_root: input.project_root.clone(),
            workflow_id: input.workflow_id.clone(),
            node_id: input.node_id.clone(),
            work_item_id: input.work_item_id.clone(),
            authorization_id: input.authorization_id.clone(),
            allowed_write: input.allowed_write.clone(),
            state: "reserved".to_string(),
            started_at_ms,
            ..SupervisorWorker::default()
        });
        Ok(())
    })?;
    Ok(reservation_id)
}

fn complete_dispatch(
    config: &McpServerConfig,
    reservation_id: &str,
    launch: &WorkerLaunch,
) -> Result<(), String> {
    update_store(config, "complete-dispatch", |store| {
        let worker = session_mut(store, &config.run_id)
            .workers
            .iter_mut()
            .find(|worker| worker.worker_id == reservation_id)
            .ok_or_else(|| "派发预约记录丢失，拒绝伪造 worker".to_string())?;
        worker.worker_id = launch.worker_id.clone();
        worker.native_thread_id = launch.native_thread_id.clone();
        worker.dispatch_id = launch.dispatch_id.clone();
        worker.state = launch.state.clone();
        worker.last_report = launch.initial_report.clone();
        worker.last_result_summary = launch.result_summary.clone();
        Ok(())
    })
}

fn fail_dispatch(
    config: &McpServerConfig,
    reservation_id: &str,
    error: &str,
) -> Result<(), String> {
    update_store(config, "fail-dispatch", |store| {
        if let Some(worker) = session_mut(store, &config.run_id)
            .workers
            .iter_mut()
            .find(|worker| worker.worker_id == reservation_id)
        {
            worker.state = "failed".to_string();
            worker.last_result_summary =
                crate::run_error_translation::humanize_error_for_display(error);
        }
        Ok(())
    })
}

fn reserve_follow_up(config: &McpServerConfig, worker_id: &str) -> Result<(), String> {
    let limits = quota_limits(config)?;
    update_store(config, "reserve-follow-up", |store| {
        let worker = session_mut(store, &config.run_id)
            .workers
            .iter_mut()
            .find(|worker| worker.worker_id == worker_id)
            .ok_or_else(|| "主管当前会话没有该 worker，拒绝追问".to_string())?;
        ensure_worker_runtime_for_limits(&limits, worker)?;
        if worker.follow_up_count >= limits.max_follow_ups_per_worker {
            return Err(format!(
                "worker 追问配额已满（{}），请转人工决定",
                limits.max_follow_ups_per_worker
            ));
        }
        worker.follow_up_count += 1;
        Ok(())
    })
}

fn update_worker_result(
    config: &McpServerConfig,
    worker_id: &str,
    state: &str,
    summary: &str,
) -> Result<(), String> {
    update_store(config, "update-worker-result", |store| {
        let worker = session_mut(store, &config.run_id)
            .workers
            .iter_mut()
            .find(|worker| worker.worker_id == worker_id)
            .ok_or_else(|| "主管当前会话没有该 worker，拒绝写结果".to_string())?;
        worker.state = state.to_string();
        worker.last_result_summary = summary.to_string();
        Ok(())
    })
}

fn ensure_worker_within_runtime(
    config: &McpServerConfig,
    worker: &SupervisorWorker,
) -> Result<(), String> {
    ensure_worker_runtime_for_limits(&quota_limits(config)?, worker)
}

fn ensure_worker_runtime_for_limits(
    limits: &SupervisorQuotaLimits,
    worker: &SupervisorWorker,
) -> Result<(), String> {
    let elapsed = now_ms().saturating_sub(worker.started_at_ms);
    if elapsed > limits.max_runtime_minutes.saturating_mul(60_000) {
        return Err("worker 已超过主管会话时长配额，等待人工决定；此工具不终止进程".to_string());
    }
    Ok(())
}

fn active_authorization(
    config: &McpServerConfig,
    project_root: &str,
    workflow_id: &str,
    authorization_id: &str,
) -> Result<crate::PlanAuthorization, String> {
    let store =
        crate::plan_authorization_store::load_store(workflow_state_path(config)?, now_ms())?;
    let authorization = store
        .authorizations
        .iter()
        .find(|authorization| authorization.authorization_id == authorization_id)
        .cloned()
        .ok_or_else(|| "找不到当前主管请求的授权段，已拒绝".to_string())?;
    if authorization.status != crate::PlanAuthorizationStatus::Active
        || authorization
            .expires_at_ms
            .is_some_and(|expires_at_ms| expires_at_ms <= now_ms())
    {
        return Err("授权段不是有效 active 状态，已拒绝".to_string());
    }
    if authorization.project_id != crate::project_id(project_root)
        || authorization.workflow_id != workflow_id
    {
        return Err("授权段与请求 project/workflow 不一致，已拒绝".to_string());
    }
    Ok(authorization)
}

fn check_authorization(
    project_root: &str,
    workflow_id: &str,
    authorization_id: &str,
    node_id: &str,
    allowed_write: &[String],
    config: &McpServerConfig,
) -> Result<(), String> {
    let authorization = active_authorization(config, project_root, workflow_id, authorization_id)?;
    let role_id = crate::role_id_from_node_id(node_id);
    if !authorization
        .scope
        .allowed_role_ids
        .iter()
        .any(|role| role == &role_id)
    {
        return Err("授权段不允许该 worker role，已拒绝".to_string());
    }
    if !allowed_write.iter().all(|path| {
        authorization
            .scope
            .allowed_write_roots
            .iter()
            .any(|root| root == path)
    }) {
        return Err("请求 allowed_write 超出授权段允许写根，已拒绝".to_string());
    }
    Ok(())
}

fn deny_sensitive_file(path: &Path) -> Result<(), String> {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    let lower = name.to_ascii_lowercase();
    if matches!(name, ".env" | "auth.json") || lower.contains("token") || lower.contains("secret") {
        return Err("关键文件读取不允许 auth/.env/token/secret 内容".to_string());
    }
    Ok(())
}

fn append_audit(
    config: &McpServerConfig,
    tool: &str,
    parameter_summary: &str,
    result_summary: &str,
) -> Result<(), String> {
    let created_at_ms = now_ms();
    update_store(config, "append-tool-audit", |store| {
        store.audit_events.push(SupervisorAuditEvent {
            event_id: format!(
                "supervisor-orchestrator:{}:{}:{}",
                stable_fragment(&config.run_id),
                stable_fragment(tool),
                crate::unix_timestamp_nanos()
            ),
            actor: ACTOR.to_string(),
            run_id: config.run_id.clone(),
            tool: tool.to_string(),
            parameter_summary: parameter_summary.to_string(),
            result_summary: result_summary.to_string(),
            created_at_ms,
        });
        Ok(())
    })
}

fn parameter_summary(arguments: &Value) -> String {
    let mut compact = serde_json::to_string(arguments)
        .unwrap_or_else(|_| "<unserializable>".to_string())
        .chars()
        .take(4_000)
        .collect::<String>();
    if let Some(prompt) = arguments.get("prompt").and_then(Value::as_str) {
        compact = format!("{}; prompt_full_summary={}", compact, prompt);
    }
    compact
}

fn summary_of_value(value: &Value) -> String {
    serde_json::to_string(value)
        .unwrap_or_else(|_| "<unserializable>".to_string())
        .chars()
        .take(2_000)
        .collect()
}

fn stable_fragment(value: &str) -> String {
    value
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect::<String>()
        .chars()
        .take(80)
        .collect()
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    const PROJECT: &str = "/Users/yoyi/codex-workflow-mario-test";
    const WORKFLOW: &str = "workflow:users-yoyi-codex-workflow-mario-test:default";
    const AUTH: &str = "plan-auth:station1";
    const NODE: &str = "workflow:users-yoyi-codex-workflow-mario-test:default:node:worker";
    static FIXTURE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    #[derive(Default)]
    struct FakeInvoker;

    impl WorkerInvoker for FakeInvoker {
        fn dispatch(
            &self,
            _config: &McpServerConfig,
            _input: &DispatchInput,
        ) -> Result<WorkerLaunch, String> {
            Ok(WorkerLaunch {
                worker_id: "worker-1".to_string(),
                native_thread_id: "thread-1".to_string(),
                dispatch_id: "dispatch-1".to_string(),
                state: "completed".to_string(),
                initial_report: Some(
                    json!({"worker_id": "worker-1", "acceptance_status": "reported_completed"}),
                ),
                result_summary: "mock dispatched".to_string(),
            })
        }

        fn follow_up(
            &self,
            _config: &McpServerConfig,
            _worker: &SupervisorWorker,
            prompt: &str,
        ) -> Result<String, String> {
            Ok(format!("mock follow-up: {prompt}"))
        }
    }

    struct Fixture {
        root: PathBuf,
        state_path: PathBuf,
        read_root: PathBuf,
        config: McpServerConfig,
    }

    impl Fixture {
        fn new() -> Self {
            let unique = format!(
                "supervisor-orchestrator-{}-{}-{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("system clock")
                    .as_nanos(),
                FIXTURE_SEQUENCE.fetch_add(1, Ordering::Relaxed)
            );
            let root = std::env::temp_dir().join(unique);
            let read_root = root.join("read-root");
            fs::create_dir_all(&read_root).expect("fixture dirs");
            let state_path = root.join("workflow-state.json");
            fs::write(
                &state_path,
                json!({"workflow_chain_runs": [{"status": "running"}], "audit_events": []})
                    .to_string(),
            )
            .expect("state");
            let fixture = Self {
                root,
                state_path: state_path.clone(),
                read_root,
                config: McpServerConfig {
                    role: super::super::McpRole::SupervisorOrchestrator,
                    run_id: "supervisor-test-run".to_string(),
                    node_id: None,
                    supervisor_workflow_state_path: Some(state_path),
                    supervisor_quota_limits: Some(SupervisorQuotaLimits {
                        max_active_workers: 1,
                        max_follow_ups_per_worker: 2,
                        max_runtime_minutes: 30,
                    }),
                },
            };
            fixture.write_active_authorization();
            fixture
        }

        fn write_active_authorization(&self) {
            let store = crate::PlanAuthorizationStoreV1 {
                schema_version: "plan_authorization_store.v1".to_string(),
                revision: 1,
                authorizations: vec![crate::PlanAuthorization {
                    authorization_id: AUTH.to_string(),
                    schema_version: "plan_authorization.v1".to_string(),
                    project_id: crate::project_id(PROJECT),
                    workflow_id: WORKFLOW.to_string(),
                    source_proposal_id: None,
                    title: "station1".to_string(),
                    goal_summary: "station1".to_string(),
                    status: crate::PlanAuthorizationStatus::Active,
                    scope: crate::AuthorizedExecutionScope {
                        project_id: crate::project_id(PROJECT),
                        workflow_id: WORKFLOW.to_string(),
                        allowed_role_ids: vec!["worker".to_string()],
                        allowed_agent_ids: vec!["thread-1".to_string()],
                        allowed_read_roots: vec![self.read_root.display().to_string()],
                        allowed_write_roots: vec![PROJECT.to_string()],
                        allowed_tools: vec!["codex_exec_resume".to_string()],
                        allowed_checks: vec![],
                        allowed_task_package_kinds: vec![],
                        max_worker_dispatches: None,
                        max_runtime_minutes: None,
                        stop_conditions: vec![],
                    },
                    user_confirmation: None,
                    global_boundary_review: None,
                    audit_refs: vec![],
                    created_at_ms: now_ms(),
                    updated_at_ms: now_ms(),
                    expires_at_ms: None,
                }],
                audit_events: vec![],
                updated_at_ms: now_ms(),
                warnings: vec![],
            };
            let auth_path =
                crate::plan_authorization_store::sidecar_path(&self.state_path).expect("auth path");
            fs::write(auth_path, serde_json::to_string(&store).expect("auth json"))
                .expect("auth store");
        }

        fn call(&self, name: &str, arguments: Value) -> Result<Value, String> {
            call_tool_with_invoker(
                &self.config,
                json!({"name": name, "arguments": arguments}),
                &FakeInvoker,
            )
        }

        fn audit_count(&self, tool: &str) -> usize {
            load_store(&self.config)
                .expect("store")
                .audit_events
                .iter()
                .filter(|event| event.tool == tool && event.actor == ACTOR)
                .count()
        }

        fn dispatch(&self) -> Result<Value, String> {
            self.call(
                "dispatch_worker",
                json!({
                    "project_root": PROJECT, "workflow_id": WORKFLOW, "authorization_id": AUTH,
                    "node_id": NODE, "work_item_id": "work-1", "allowed_write": [PROJECT]
                }),
            )
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn dispatch_worker_denies_bad_authorization_and_audits_success() {
        let fixture = Fixture::new();
        assert!(fixture.call("dispatch_worker", json!({"project_root": PROJECT, "workflow_id": WORKFLOW, "authorization_id": "bad", "node_id": NODE, "work_item_id": "work-1", "allowed_write": [PROJECT]})).is_err());
        fixture.dispatch().expect("dispatch");
        assert_eq!(fixture.audit_count("dispatch_worker"), 2);
    }

    #[test]
    fn read_worker_report_denies_unknown_worker_and_audits_success() {
        let fixture = Fixture::new();
        assert!(fixture
            .call("read_worker_report", json!({"worker_id": "missing"}))
            .is_err());
        fixture.dispatch().expect("dispatch");
        fixture
            .call("read_worker_report", json!({"worker_id": "worker-1"}))
            .expect("report");
        assert_eq!(fixture.audit_count("read_worker_report"), 2);
    }

    #[test]
    fn follow_up_denies_unknown_worker_and_audits_success() {
        let fixture = Fixture::new();
        assert!(fixture
            .call(
                "follow_up_worker",
                json!({"worker_id": "missing", "prompt": "what changed?"})
            )
            .is_err());
        fixture.dispatch().expect("dispatch");
        fixture
            .call(
                "follow_up_worker",
                json!({"worker_id": "worker-1", "prompt": "what changed?"}),
            )
            .expect("follow up");
        assert_eq!(fixture.audit_count("follow_up_worker"), 2);
    }

    #[test]
    fn wait_denies_unknown_worker_and_audits_success() {
        let fixture = Fixture::new();
        assert!(fixture
            .call("wait_for_worker", json!({"worker_id": "missing"}))
            .is_err());
        fixture.dispatch().expect("dispatch");
        fixture
            .call("wait_for_worker", json!({"worker_id": "worker-1"}))
            .expect("wait");
        assert_eq!(fixture.audit_count("wait_for_worker"), 2);
    }

    #[test]
    fn read_key_file_denies_outside_root_and_audits_success() {
        let fixture = Fixture::new();
        let allowed = fixture.read_root.join("evidence.txt");
        fs::write(&allowed, "evidence").expect("evidence");
        let outside = fixture.root.join("outside.txt");
        fs::write(&outside, "outside").expect("outside");
        active_authorization(&fixture.config, PROJECT, WORKFLOW, AUTH).expect("active auth");
        let denied = fixture
            .call(
                "read_key_file",
                json!({"project_root": PROJECT, "workflow_id": WORKFLOW, "authorization_id": AUTH, "path": outside}),
            )
            .expect_err("outside read must be denied");
        assert!(
            denied.contains("allowed_read"),
            "wrong deny reason: {denied}"
        );
        fixture.call("read_key_file", json!({"project_root": PROJECT, "workflow_id": WORKFLOW, "authorization_id": AUTH, "path": allowed})).expect("read");
        assert_eq!(fixture.audit_count("read_key_file"), 2);
    }

    #[test]
    fn final_mark_denies_invalid_verdict_and_audits_success_without_chain_mutation() {
        let fixture = Fixture::new();
        let before = fs::read_to_string(&fixture.state_path).expect("before");
        assert!(fixture.call("final_mark", json!({"project_root": PROJECT, "workflow_id": WORKFLOW, "authorization_id": AUTH, "verdict": "completed", "reason": "no"})).is_err());
        fixture.call("final_mark", json!({"project_root": PROJECT, "workflow_id": WORKFLOW, "authorization_id": AUTH, "verdict": "needs_rework", "reason": "mock yellow"})).expect("mark");
        assert_eq!(
            fs::read_to_string(&fixture.state_path).expect("after"),
            before,
            "advisory must not mutate chain state"
        );
        assert_eq!(fixture.audit_count("final_mark"), 2);
    }

    #[test]
    fn report_user_denies_empty_message_and_audits_success() {
        let fixture = Fixture::new();
        assert!(fixture.call("report_user", json!({"project_root": PROJECT, "workflow_id": WORKFLOW, "authorization_id": AUTH, "message": ""})).is_err());
        fixture.call("report_user", json!({"project_root": PROJECT, "workflow_id": WORKFLOW, "authorization_id": AUTH, "message": "mock report"})).expect("report user");
        assert_eq!(fixture.audit_count("report_user"), 2);
    }

    #[test]
    fn mock_end_to_end_dispatch_report_and_advisory_leave_chain_running() {
        let fixture = Fixture::new();
        fixture.dispatch().expect("dispatch");
        fixture
            .call("read_worker_report", json!({"worker_id": "worker-1"}))
            .expect("report");
        fixture.call("final_mark", json!({"project_root": PROJECT, "workflow_id": WORKFLOW, "authorization_id": AUTH, "verdict": "pass", "reason": "mock accepted"})).expect("final");
        let state: Value =
            serde_json::from_str(&fs::read_to_string(&fixture.state_path).expect("state"))
                .expect("json");
        assert_eq!(state["workflow_chain_runs"][0]["status"], "running");
    }

    #[test]
    fn supervisor_role_requires_explicit_quota_flags() {
        assert!(super::super::parse_args(&[
            "--role".to_string(),
            "supervisor_orchestrator".to_string(),
            "--run-id".to_string(),
            "r1".to_string(),
            "--workflow-state-path".to_string(),
            "/tmp/state.json".to_string(),
        ])
        .is_err());
    }
}
