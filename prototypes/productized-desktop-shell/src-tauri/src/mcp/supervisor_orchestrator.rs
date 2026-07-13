// Station 1 supervisor MCP role. It owns an isolated sidecar and never writes workflow chain state.

use super::{McpServerConfig, SupervisorQuotaLimits};
use crate::CodexResumeRunner;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeSet;
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
    project_root: String,
    #[serde(default)]
    workflow_id: String,
    #[serde(default)]
    authorization_id: String,
    #[serde(default)]
    model_id: String,
    #[serde(default)]
    reasoning_effort: String,
    #[serde(default)]
    max_active_workers: usize,
    #[serde(default)]
    max_follow_ups_per_worker: usize,
    #[serde(default)]
    max_runtime_minutes: i64,
    #[serde(default)]
    launch_status: String,
    #[serde(default)]
    started_at_ms: i64,
    #[serde(default)]
    ended_at_ms: Option<i64>,
    #[serde(default)]
    termination_reason: String,
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
    result_status: String,
    #[serde(default)]
    created_at_ms: i64,
}

#[derive(Debug, Clone)]
pub(crate) struct SupervisorPilotSessionLaunch {
    pub(crate) project_root: String,
    pub(crate) workflow_id: String,
    pub(crate) authorization_id: String,
    pub(crate) model_id: String,
    pub(crate) reasoning_effort: String,
    pub(crate) workbench_executable_path: String,
    pub(crate) workbench_build_id: String,
    pub(crate) supervisor_contract_version: String,
    pub(crate) supervisor_contract_sha256: String,
    pub(crate) worker_report_contract_sha256: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SupervisorPilotAuditEventReadModel {
    pub(crate) event_id: String,
    pub(crate) tool: String,
    pub(crate) result_summary: String,
    pub(crate) result_status: String,
    pub(crate) created_at_ms: i64,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SupervisorPilotMetricsReadModel {
    pub(crate) denied_tool_call_count: usize,
    pub(crate) max_follow_ups_per_worker: usize,
    pub(crate) follow_up_count: usize,
    pub(crate) follow_up_budget_respected: bool,
    pub(crate) max_runtime_minutes: i64,
    pub(crate) session_timed_out: bool,
    pub(crate) ledger_replay_event_count: usize,
    pub(crate) ledger_replay_ready: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SupervisorPilotReadModel {
    pub(crate) run_id: String,
    pub(crate) launch_status: String,
    pub(crate) project_root: String,
    pub(crate) workflow_id: String,
    pub(crate) authorization_id: String,
    pub(crate) started_at_ms: i64,
    pub(crate) ended_at_ms: Option<i64>,
    pub(crate) termination_reason: String,
    pub(crate) metrics: SupervisorPilotMetricsReadModel,
    pub(crate) audit_events: Vec<SupervisorPilotAuditEventReadModel>,
}

#[derive(Debug, Clone)]
pub(crate) struct DispatchInput {
    pub(crate) project_root: String,
    pub(crate) workflow_id: String,
    pub(crate) authorization_id: String,
    pub(crate) node_id: String,
    pub(crate) work_item_id: String,
    pub(crate) allowed_write: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct WorkerLaunch {
    pub(crate) worker_id: String,
    pub(crate) native_thread_id: String,
    pub(crate) dispatch_id: String,
    pub(crate) canonical_work_item_id: String,
    pub(crate) state: String,
    pub(crate) initial_report: Option<Value>,
    pub(crate) result_summary: String,
}

#[derive(Debug, Clone)]
struct WorkerFollowUp {
    summary: String,
    report: Value,
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
    ) -> Result<WorkerFollowUp, String>;
}

struct WorkbenchWorkerInvoker;

impl WorkerInvoker for WorkbenchWorkerInvoker {
    fn dispatch(
        &self,
        config: &McpServerConfig,
        input: &DispatchInput,
    ) -> Result<WorkerLaunch, String> {
        // 站 3b（2026-07-12）：3b 项目仅当派发写范围为空（只读 worker）才放行；S1 原闸不动。
        if !crate::workflow_engine_test_project_unsealed(&input.project_root)
            && !crate::station3b_readonly_project_unsealed(
                &input.project_root,
                &input.allowed_write,
            )
        {
            return Err(crate::legacy_product_command_blocked_message(
                "execute_project_workflow_node",
            ));
        }
        let app_state = crate::AppState::new();
        let index = crate::read_index(&app_state)?;
        let runner = crate::codex_local_runner::RealWorkflowNodeCodexRunner;
        let workflow_state_path = workflow_state_path(config)?;
        dispatch_workbench_worker_with(
            config,
            input,
            &index,
            workflow_state_path,
            &crate::codex_db::default_state_db_path(),
            &runner,
            &crate::ManualRelayJiaobanNewSessionCreator,
        )
    }

    fn follow_up(
        &self,
        config: &McpServerConfig,
        worker: &SupervisorWorker,
        prompt: &str,
    ) -> Result<WorkerFollowUp, String> {
        // 站 3b：追问同派发口径——3b 项目仅限只读 worker（写范围空）。
        if !crate::workflow_engine_test_project_unsealed(&worker.project_root)
            && !crate::station3b_readonly_project_unsealed(
                &worker.project_root,
                &worker.allowed_write,
            )
        {
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
        let workflow_state_path = workflow_state_path(config)?;
        let parent = crate::utils::store_paths::ensure_runtime_artifact_dir(
            workflow_state_path,
            "supervisor",
            &config.run_id,
        )?;
        let last_message_path = parent.join(format!(
            "follow-up-{}-{}.last-message.txt",
            stable_fragment(&worker.worker_id),
            crate::unix_timestamp_nanos()
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
        let raw = fs::read_to_string(&last_message_path).map_err(|error| {
            format!(
                "report_invalid: 无法读取 worker 追问最终消息 {}：{error}",
                last_message_path.display()
            )
        })?;
        let report = normalized_worker_report_from_raw(worker, &raw)?;
        Ok(WorkerFollowUp {
            summary: format!(
                "已在原绑定会话续问并读回新报告：{}",
                last_message_path.display()
            ),
            report,
        })
    }
}

pub(crate) fn dispatch_workbench_worker_with(
    config: &McpServerConfig,
    input: &DispatchInput,
    index: &Value,
    workflow_state_path: &Path,
    readback_db_path: &Path,
    runner: &dyn crate::CodexResumeRunner,
    creator: &dyn crate::JiaobanNewSessionCreator,
) -> Result<WorkerLaunch, String> {
    let canonical_work_item_id = canonical_prepared_work_item_id(workflow_state_path, input)?;
    if canonical_work_item_id != input.work_item_id {
        append_audit(
                config,
                "dispatch_worker_work_item_canonicalized",
                &format!(
                    "requested_work_item_id={}; canonical_work_item_id={}",
                    input.work_item_id, canonical_work_item_id
                ),
                "主管提供的 work item 文本有偏差；工作台按当前授权段唯一 prepared dispatch 恢复正本 ID。",
                "warning",
            )?;
    }
    let state_before_binding = crate::read_workflow_state_value(workflow_state_path)?;
    let historical_thread_ids = historical_native_thread_ids(&state_before_binding);
    let (task_title, task_package_id) =
        supervisor_task_session_metadata(&state_before_binding, &canonical_work_item_id)?;
    let fresh_thread_id = crate::create_and_bind_fresh_task_session(
        workflow_state_path,
        index,
        &crate::FreshTaskSessionBindingRequest {
            project_root: &input.project_root,
            workflow_id: &input.workflow_id,
            node_id: &input.node_id,
            work_item_id: &canonical_work_item_id,
            task_title: &task_title,
            task_package_id: Some(task_package_id.as_str()),
            requested_by: ACTOR,
            forbidden_thread_ids: &historical_thread_ids,
        },
        creator,
    )?;
    let state_after_binding = crate::read_workflow_state_value(workflow_state_path)?;
    let bound_thread_id = exact_work_item_native_thread_id(
        &state_after_binding,
        &input.workflow_id,
        &input.node_id,
        &canonical_work_item_id,
    )
    .ok_or_else(|| {
        let reason = "新建任务会话后缺少该 work item 的精确绑定，拒绝派发 worker";
        let _ = abandon_fresh_task_binding(
            config,
            workflow_state_path,
            input,
            &canonical_work_item_id,
            &fresh_thread_id,
            reason,
        );
        reason.to_string()
    })?;
    if bound_thread_id != fresh_thread_id {
        let error = format!(
                "新建任务会话绑定与预期 native_thread_id 不一致（expected {fresh_thread_id}, actual {bound_thread_id}），拒绝派发 worker"
            );
        abandon_fresh_task_binding(
            config,
            workflow_state_path,
            input,
            &canonical_work_item_id,
            &fresh_thread_id,
            &error,
        )?;
        return Err(error);
    }
    append_audit(
        config,
        "fresh_task_session_bound",
        &format!("work_item_id={canonical_work_item_id}; native_thread_id={fresh_thread_id}"),
        "已通过 C1 建会话并精确绑定本 work item；历史 native_thread_id 不会复用。",
        "accepted",
    )?;
    let result = match crate::execute_authorized_project_workflow_node_at(
        workflow_state_path,
        index,
        readback_db_path,
        runner,
        &crate::ProjectWorkflowNodeRunRequest {
            project_root: input.project_root.clone(),
            node_id: input.node_id.clone(),
            work_item_id: canonical_work_item_id.clone(),
            workflow_id: Some(input.workflow_id.clone()),
        },
        &input.authorization_id,
        &input.allowed_write,
    ) {
        Ok(result) => result,
        Err(error) => {
            let cleanup = abandon_fresh_task_binding(
                config,
                workflow_state_path,
                input,
                &canonical_work_item_id,
                &fresh_thread_id,
                &format!("worker 派发失败：{error}"),
            );
            return Err(match cleanup {
                Ok(()) => error,
                Err(cleanup_error) => {
                    format!("{error}；同时清理 active 绑定失败：{cleanup_error}")
                }
            });
        }
    };
    if result.dispatch.plan_authorization_id.as_deref() != Some(input.authorization_id.as_str()) {
        let error = "现成派发返回的授权段与主管请求不一致，已拒绝纳入主管账本";
        abandon_fresh_task_binding(
            config,
            workflow_state_path,
            input,
            &canonical_work_item_id,
            &fresh_thread_id,
            error,
        )?;
        return Err(error.to_string());
    }
    if result.dispatch.native_thread_id != fresh_thread_id {
        let error = format!(
                "现成派发返回的 native_thread_id 与本任务新建绑定不一致（expected {fresh_thread_id}, actual {}），拒绝纳入主管账本",
                result.dispatch.native_thread_id
            );
        abandon_fresh_task_binding(
            config,
            workflow_state_path,
            input,
            &canonical_work_item_id,
            &fresh_thread_id,
            &error,
        )?;
        return Err(error);
    }
    Ok(WorkerLaunch {
        worker_id: result.dispatch.dispatch_id.clone(),
        native_thread_id: result.dispatch.native_thread_id,
        dispatch_id: result.dispatch.dispatch_id,
        canonical_work_item_id,
        state: result.dispatch.state,
        initial_report: None,
        result_summary: result.message,
    })
}

fn canonical_prepared_work_item_id(
    workflow_state_path: &Path,
    input: &DispatchInput,
) -> Result<String, String> {
    let value = crate::read_workflow_state_value(workflow_state_path)?;
    canonical_prepared_work_item_id_from_value(&value, input)
}

fn canonical_prepared_work_item_id_from_value(
    value: &Value,
    input: &DispatchInput,
) -> Result<String, String> {
    let expected_project_id = crate::project_id(&input.project_root);
    let candidates = value
        .get("workflow_node_dispatches")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|dispatch| {
            crate::optional_string_from(dispatch, "state").as_deref() == Some("prepared")
                && crate::optional_string_from(dispatch, "prompt_kind").as_deref()
                    == Some("authorized_prepared_auto_dispatch")
                && crate::optional_string_from(dispatch, "project_id").as_deref()
                    == Some(expected_project_id.as_str())
                && crate::optional_string_from(dispatch, "workflow_id").as_deref()
                    == Some(input.workflow_id.as_str())
                && crate::optional_string_from(dispatch, "node_id").as_deref()
                    == Some(input.node_id.as_str())
                && crate::optional_string_from(dispatch, "plan_authorization_id").as_deref()
                    == Some(input.authorization_id.as_str())
        })
        .filter_map(|dispatch| crate::optional_string_from(dispatch, "work_item_id"))
        .collect::<BTreeSet<_>>();
    if candidates.contains(&input.work_item_id) {
        return Ok(input.work_item_id.clone());
    }
    if candidates.len() != 1 {
        return Err(format!(
            "当前授权段匹配到 {} 个 prepared work item，无法唯一恢复正本 ID，已拒绝启动 worker",
            candidates.len()
        ));
    }
    candidates
        .into_iter()
        .next()
        .ok_or_else(|| "prepared work item 正本 ID 丢失，已拒绝启动 worker".to_string())
}

fn historical_native_thread_ids(value: &Value) -> BTreeSet<String> {
    [
        "workflow_node_dispatches",
        "workflow_node_session_bindings",
        "audit_events",
    ]
    .into_iter()
    .flat_map(|key| {
        value
            .get(key)
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
    })
    .filter_map(|record| crate::optional_string_from(record, "native_thread_id"))
    .filter(|thread_id| !thread_id.trim().is_empty())
    .collect()
}

fn supervisor_task_session_metadata(
    value: &Value,
    work_item_id: &str,
) -> Result<(String, String), String> {
    let artifact = value
        .get("artifacts")
        .and_then(Value::as_array)
        .and_then(|artifacts| {
            artifacts.iter().find(|artifact| {
                crate::optional_string_from(artifact, "artifact_type").as_deref()
                    == Some("task_package")
                    && crate::optional_string_from(artifact, "source_ref").as_deref()
                        == Some(work_item_id)
            })
        })
        .ok_or_else(|| {
            "prepared work item 缺 task package artifact，拒绝新建并绑定任务会话".to_string()
        })?;
    let task_title = crate::optional_string_from(artifact, "task_name")
        .or_else(|| crate::optional_string_from(artifact, "title"))
        .filter(|title| !title.trim().is_empty())
        .ok_or_else(|| "task package 缺任务标题，拒绝新建并绑定任务会话".to_string())?;
    let task_package_id = crate::optional_string_from(artifact, "artifact_id")
        .ok_or_else(|| "task package 缺 artifact_id，拒绝新建并绑定任务会话".to_string())?;
    Ok((task_title, task_package_id))
}

pub(crate) fn exact_work_item_native_thread_id(
    value: &Value,
    workflow_id: &str,
    node_id: &str,
    work_item_id: &str,
) -> Option<String> {
    value
        .get("workflow_node_session_bindings")
        .and_then(Value::as_array)
        .and_then(|bindings| {
            bindings.iter().find_map(|binding| {
                (crate::optional_string_from(binding, "workflow_id").as_deref()
                    == Some(workflow_id)
                    && crate::optional_string_from(binding, "node_id").as_deref() == Some(node_id)
                    && crate::optional_string_from(binding, "work_item_id").as_deref()
                        == Some(work_item_id)
                    && crate::optional_string_from(binding, "lifecycle").as_deref()
                        == Some("active"))
                .then(|| crate::optional_string_from(binding, "native_thread_id"))
                .flatten()
            })
        })
}

pub(crate) fn abandon_fresh_task_binding(
    config: &McpServerConfig,
    workflow_state_path: &Path,
    input: &DispatchInput,
    canonical_work_item_id: &str,
    native_thread_id: &str,
    reason: &str,
) -> Result<(), String> {
    let mut value = crate::read_workflow_state_value(workflow_state_path)?;
    let now_ms = now_ms();
    let mut detached = false;
    if let Some(binding) = value
        .get_mut("workflow_node_session_bindings")
        .and_then(Value::as_array_mut)
        .and_then(|bindings| {
            bindings.iter_mut().find(|binding| {
                crate::optional_string_from(binding, "workflow_id").as_deref()
                    == Some(input.workflow_id.as_str())
                    && crate::optional_string_from(binding, "node_id").as_deref()
                        == Some(input.node_id.as_str())
                    && crate::optional_string_from(binding, "work_item_id").as_deref()
                        == Some(canonical_work_item_id)
                    && crate::optional_string_from(binding, "native_thread_id").as_deref()
                        == Some(native_thread_id)
                    && crate::optional_string_from(binding, "lifecycle").as_deref()
                        == Some("active")
            })
        })
    {
        binding["lifecycle"] = Value::String("detached".to_string());
        binding["updated_at_ms"] = Value::Number(now_ms.into());
        detached = true;
    }
    if let Some(artifacts) = value.get_mut("artifacts").and_then(Value::as_array_mut) {
        if let Some(artifact) = artifacts.iter_mut().find(|artifact| {
            crate::optional_string_from(artifact, "source_ref").as_deref()
                == Some(canonical_work_item_id)
                && crate::optional_string_from(artifact, "target_session_id").as_deref()
                    == Some(native_thread_id)
        }) {
            artifact["target_session_id"] = Value::Null;
        }
    }
    let audit_events = value
        .get_mut("audit_events")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| "workflow state 缺 audit_events，无法记录新会话失败收尾".to_string())?;
    audit_events.push(json!({
        "event_id": crate::workflow_audit::audit_event_identity(
            "supervisor-task-session-abandoned",
            native_thread_id,
            crate::unix_timestamp_nanos()
        ),
        "event_type": "supervisor_task_session_abandoned",
        "target_ref": canonical_work_item_id,
        "actor_ref": ACTOR,
        "source_kind": "workspace_state",
        "permission_level": "authorized_supervisor_execution",
        "native_thread_id": native_thread_id,
        "before_state": if detached { "active" } else { "created_or_unbound" },
        "after_state": "detached_or_orphaned",
        "created_at": crate::unix_timestamp_string(),
        "created_at_ms": now_ms,
        "reason": reason
    }));
    value["updated_at"] = Value::String(crate::unix_timestamp_string());
    crate::write_validated_workflow_state(workflow_state_path, &value)?;
    append_audit(
        config,
        "fresh_task_session_abandoned",
        &format!(
            "work_item_id={canonical_work_item_id}; native_thread_id={native_thread_id}; binding_detached={detached}"
        ),
        reason,
        "warning",
    )
}

pub fn list_tools() -> Value {
    json!({
        "tools": [
            tool_def("read_worker_report", "只读投影 worker 结构化口供", json!({
                "type": "object", "properties": {"worker_id": {"type": "string"}}, "required": ["worker_id"], "additionalProperties": false
            })),
            tool_def("wait_for_worker", "读取 worker 当前状态，不管理或终止进程", json!({
                "type": "object", "properties": {"worker_id": {"type": "string"}}, "required": ["worker_id"], "additionalProperties": false
            })),
            tool_def("read_key_file", "在授权允许读取根内读取关键文本文件", json!({
                "type": "object", "properties": {"project_root": {"type": "string"}, "workflow_id": {"type": "string"}, "authorization_id": {"type": "string"}, "path": {"type": "string"}}, "required": ["project_root", "workflow_id", "authorization_id", "path"], "additionalProperties": false
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
    _invoker: &dyn WorkerInvoker,
) -> Result<Value, String> {
    let name = require_string(&params, "name")?;
    let arguments = params
        .get("arguments")
        .cloned()
        .unwrap_or(Value::Object(Default::default()));
    let result = match name.as_str() {
        "read_worker_report" => read_worker_report(config, &arguments),
        "wait_for_worker" => wait_for_worker(config, &arguments),
        "read_key_file" => read_key_file(config, &arguments),
        _ => Err(format!("主管编排角色不认识工具：{name}")),
    };
    let result_summary = match &result {
        Ok(value) => summary_of_value(value),
        Err(error) => format!(
            "denied: {}",
            crate::run_error_translation::humanize_error_for_display(error)
        ),
    };
    let result_status = if result.is_ok() { "accepted" } else { "denied" };
    append_audit(
        config,
        &name,
        &parameter_summary(&arguments),
        &result_summary,
        result_status,
    )?;
    result
        .map(tool_result)
        .map_err(|error| crate::run_error_translation::humanize_error_for_display(&error))
}

// Side-effecting supervisor actions deliberately bypass the MCP tools/call surface.
// They remain behind this host-only bridge so the controller can bind authority before
// invoking the existing guarded worker entrypoints.
pub(crate) fn ensure_control_core_run_active(config: &McpServerConfig) -> Result<(), String> {
    let store = load_store(config)?;
    let session = session(&store, &config.run_id)
        .ok_or_else(|| "authorization_stale: 主管 run 尚未登记或已不存在。".to_string())?;
    if session.launch_status != "running" {
        return Err("authorization_stale: 主管 run 已结束，拒绝执行新动作。".to_string());
    }
    Ok(())
}

pub(crate) fn ensure_control_core_run_binding(
    config: &McpServerConfig,
    project_root: &str,
    workflow_id: &str,
    authorization_id: &str,
) -> Result<(), String> {
    ensure_control_core_run_active(config)?;
    let store = load_store(config)?;
    let session = session(&store, &config.run_id)
        .ok_or_else(|| "authorization_stale: 主管 run 尚未登记或已不存在。".to_string())?;
    if session.project_root != project_root
        || session.workflow_id != workflow_id
        || session.authorization_id != authorization_id
    {
        return Err(
            "authorization_stale: 主管 run 与当前 project/workflow/authorization 绑定不一致。"
                .to_string(),
        );
    }
    Ok(())
}

pub(crate) fn control_core_dispatch_worker(
    config: &McpServerConfig,
    project_root: &str,
    workflow_id: &str,
    authorization_id: &str,
    node_id: &str,
    work_item_id: &str,
) -> Result<Value, String> {
    let authorization = active_authorization(config, project_root, workflow_id, authorization_id)?;
    let arguments = json!({
        "project_root": project_root,
        "workflow_id": workflow_id,
        "authorization_id": authorization_id,
        "node_id": node_id,
        "work_item_id": work_item_id,
        "allowed_write": authorization.scope.allowed_write_roots,
    });
    control_core_call_with_invoker(
        config,
        "dispatch_worker",
        &arguments,
        &WorkbenchWorkerInvoker,
    )
}

pub(crate) fn control_core_read_worker_report(
    config: &McpServerConfig,
    worker_id: &str,
) -> Result<Value, String> {
    control_core_call_with_invoker(
        config,
        "inspect_worker",
        &json!({"worker_id": worker_id}),
        &WorkbenchWorkerInvoker,
    )
}

pub(crate) fn control_core_follow_up_worker(
    config: &McpServerConfig,
    worker_id: &str,
    prompt: &str,
) -> Result<Value, String> {
    control_core_call_with_invoker(
        config,
        "follow_up_worker",
        &json!({"worker_id": worker_id, "prompt": prompt}),
        &WorkbenchWorkerInvoker,
    )
}

pub(crate) fn control_core_wait_for_worker(
    config: &McpServerConfig,
    worker_id: &str,
) -> Result<Value, String> {
    control_core_call_with_invoker(
        config,
        "wait_worker",
        &json!({"worker_id": worker_id}),
        &WorkbenchWorkerInvoker,
    )
}

pub(crate) fn control_core_finalize(
    config: &McpServerConfig,
    project_root: &str,
    workflow_id: &str,
    authorization_id: &str,
    verdict: &str,
    reason: &str,
) -> Result<Value, String> {
    control_core_call_with_invoker(
        config,
        "finalize",
        &json!({
            "project_root": project_root,
            "workflow_id": workflow_id,
            "authorization_id": authorization_id,
            "verdict": verdict,
            "reason": reason,
        }),
        &WorkbenchWorkerInvoker,
    )
}

pub(crate) fn control_core_report_user(
    config: &McpServerConfig,
    project_root: &str,
    workflow_id: &str,
    authorization_id: &str,
    message: &str,
) -> Result<Value, String> {
    control_core_call_with_invoker(
        config,
        "report_user",
        &json!({
            "project_root": project_root,
            "workflow_id": workflow_id,
            "authorization_id": authorization_id,
            "message": message,
        }),
        &WorkbenchWorkerInvoker,
    )
}

fn control_core_call_with_invoker(
    config: &McpServerConfig,
    action: &str,
    arguments: &Value,
    invoker: &dyn WorkerInvoker,
) -> Result<Value, String> {
    let result = match action {
        "dispatch_worker" => dispatch_worker(config, arguments, invoker),
        "inspect_worker" => read_worker_report(config, arguments),
        "follow_up_worker" => follow_up_worker(config, arguments, invoker),
        "wait_worker" => wait_for_worker(config, arguments),
        "finalize" => final_mark(config, arguments),
        "report_user" => report_user(config, arguments),
        _ => return Err(format!("控制核心不认识主管动作：{action}")),
    };
    let result_summary = match &result {
        Ok(value) => summary_of_value(value),
        Err(error) => format!(
            "denied: {}",
            crate::run_error_translation::humanize_error_for_display(error)
        ),
    };
    let result_status = if result.is_ok() { "accepted" } else { "denied" };
    append_audit(
        config,
        &format!("control_core_{action}"),
        &parameter_summary(arguments),
        &result_summary,
        result_status,
    )?;
    result
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
    // 站 3b（2026-07-12）：3b 项目仅当派发写范围为空（只读 worker）才放行；S1 原闸不动。
    if !crate::workflow_engine_test_project_unsealed(&input.project_root)
        && !crate::station3b_readonly_project_unsealed(&input.project_root, &input.allowed_write)
    {
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
    if worker.last_report.is_none()
        && matches!(
            worker.state.as_str(),
            "waiting_follow_up" | "follow_up_failed"
        )
    {
        return Err(
            "report_invalid: worker 追问尚未产生可验证的新报告；旧报告已失效。".to_string(),
        );
    }
    if let Some(report) = worker.last_report {
        return Ok(report);
    }
    let value = crate::read_workflow_state_value(workflow_state_path(config)?)?;
    let report = if let Some(event) = value
        .get("audit_events")
        .and_then(Value::as_array)
        .and_then(|events| {
            events.iter().rev().find(|event| {
                crate::optional_string_from(event, "event_type").as_deref()
                    == Some("worker_structured_report_recorded")
                    && crate::optional_string_from(event, "dispatch_id").as_deref()
                        == Some(worker.dispatch_id.as_str())
            })
        }) {
        json!({
            "worker_id": worker.worker_id,
            "dispatch_id": worker.dispatch_id,
            "acceptance_status": event.get("acceptance_status").cloned().unwrap_or(Value::Null),
            "executed_what": event.get("executed_what").cloned().unwrap_or(Value::Null),
            "changed_what": event.get("changed_what").cloned().unwrap_or(Value::Null),
            "summary": event.get("reason").cloned().unwrap_or(Value::Null),
            "evidence_refs": event.get("evidence_refs").cloned().unwrap_or(Value::Null),
            "open_issues": event.get("open_issues").cloned().unwrap_or(Value::Null),
            "permission_requests": event.get("permission_requests").cloned().unwrap_or(Value::Null),
            "direction_risks": event.get("direction_risks").cloned().unwrap_or(Value::Null),
            "follow_up_suggestions": event.get("follow_up_suggestions").cloned().unwrap_or(Value::Null)
        })
    } else {
        normalized_raw_worker_report(&value, &worker)?
    };
    update_store(config, "persist-worker-report", |store| {
        let worker = session_mut(store, &config.run_id)
            .workers
            .iter_mut()
            .find(|candidate| candidate.worker_id == worker.worker_id)
            .ok_or_else(|| "主管 worker 在回程持久化前丢失，拒绝伪造报告".to_string())?;
        worker.last_report = Some(report.clone());
        Ok(())
    })?;
    Ok(report)
}

fn normalized_raw_worker_report(value: &Value, worker: &SupervisorWorker) -> Result<Value, String> {
    let last_message_path = value
        .get("workflow_node_dispatches")
        .and_then(Value::as_array)
        .and_then(|dispatches| {
            dispatches.iter().find(|dispatch| {
                crate::optional_string_from(dispatch, "dispatch_id").as_deref()
                    == Some(worker.dispatch_id.as_str())
            })
        })
        .and_then(|dispatch| crate::optional_string_from(dispatch, "last_message_path"))
        .ok_or_else(|| "report_invalid: 尚无该 worker 的结构化回程或最终消息路径。".to_string())?;
    let raw = fs::read_to_string(&last_message_path).map_err(|error| {
        format!(
            "report_invalid: 无法读取 worker 最终消息 {}：{error}",
            last_message_path
        )
    })?;
    normalized_worker_report_from_raw(worker, &raw)
}

fn normalized_worker_report_from_raw(
    worker: &SupervisorWorker,
    raw: &str,
) -> Result<Value, String> {
    let report = crate::worker_report::parse_worker_report(&raw).ok_or_else(|| {
        "report_invalid: worker 最终消息不是符合回程契约的 JSON 代码块。".to_string()
    })?;
    let status = report.status.trim().to_ascii_lowercase();
    if !matches!(status.as_str(), "done" | "partial" | "failed" | "blocked") {
        return Err(format!(
            "report_invalid: worker 回程 status 不在冻结枚举内：{}。",
            report.status
        ));
    }
    if report.did.trim().is_empty() {
        return Err("report_invalid: worker 回程 did 不能为空。".to_string());
    }
    let has_help_fields = !report.permission_requests.is_empty()
        || !report.open_issues.is_empty()
        || !report.direction_risks.is_empty()
        || !report.follow_up_suggestions.is_empty();
    let acceptance_status = if status == "blocked" || has_help_fields {
        "blocked"
    } else {
        match status.as_str() {
            "done" => "reported_completed",
            "partial" => "needs_rework",
            "failed" => "reported_not_completed",
            _ => unreachable!("status is already frozen"),
        }
    };
    Ok(json!({
        "worker_id": worker.worker_id,
        "dispatch_id": worker.dispatch_id,
        "acceptance_status": acceptance_status,
        "executed_what": report.did,
        "changed_what": if report.outputs.is_empty() {
            "worker 未列出产出文件".to_string()
        } else {
            report.outputs.join("；")
        },
        "summary": report.did,
        "evidence_refs": report.evidence,
        // 只读/分析类单的结论正文（逐条判定、问题清单、总评，带 file:line + 原文引用）。
        // 站 3b 首单实证：不带出它，主管只见摘要、误判证据不足并试图越权 follow_up。
        "findings": report.findings,
        "open_issues": report.open_issues,
        "permission_requests": report.permission_requests,
        "direction_risks": report.direction_risks,
        "follow_up_suggestions": report.follow_up_suggestions
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
    // 站 3b：追问同派发口径——3b 项目仅限只读 worker（写范围空）。
    if !crate::workflow_engine_test_project_unsealed(&worker.project_root)
        && !crate::station3b_readonly_project_unsealed(&worker.project_root, &worker.allowed_write)
    {
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
    let follow_up = match invoker.follow_up(config, &worker, &prompt) {
        Ok(follow_up) => follow_up,
        Err(error) => {
            fail_follow_up(config, &worker_id, &error)?;
            return Err(error);
        }
    };
    update_worker_follow_up_result(config, &worker_id, &follow_up)?;
    Ok(json!({
        "worker_id": worker_id,
        "state": "completed",
        "summary": follow_up.summary,
        "report": follow_up.report
    }))
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
    if !matches!(verdict.as_str(), "pass" | "needs_rework" | "blocked") || reason.trim().is_empty()
    {
        return Err("终标只允许 pass / needs_rework / blocked，且 reason 不能为空".to_string());
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

pub(crate) fn record_pilot_session_started(
    config: &McpServerConfig,
    launch: &SupervisorPilotSessionLaunch,
) -> Result<(), String> {
    let limits = quota_limits(config)?;
    let started_at_ms = now_ms();
    update_store(config, "record-pilot-session-started", |store| {
        {
            let session = session_mut(store, &config.run_id);
            session.project_root = launch.project_root.clone();
            session.workflow_id = launch.workflow_id.clone();
            session.authorization_id = launch.authorization_id.clone();
            session.model_id = launch.model_id.clone();
            session.reasoning_effort = launch.reasoning_effort.clone();
            session.max_active_workers = limits.max_active_workers;
            session.max_follow_ups_per_worker = limits.max_follow_ups_per_worker;
            session.max_runtime_minutes = limits.max_runtime_minutes;
            session.launch_status = "running".to_string();
            session.started_at_ms = started_at_ms;
            session.ended_at_ms = None;
            session.termination_reason.clear();
        }
        store.audit_events.push(SupervisorAuditEvent {
            event_id: format!(
                "supervisor-orchestrator:{}:session-started:{}",
                stable_fragment(&config.run_id),
                crate::unix_timestamp_nanos()
            ),
            actor: ACTOR.to_string(),
            run_id: config.run_id.clone(),
            tool: "supervisor_session_launcher".to_string(),
            parameter_summary: format!(
                "authorization_id={}; model_id={}; reasoning_effort={}; workbench_executable_path={}; workbench_build_id={}; supervisor_contract_version={}; supervisor_contract_sha256={}; worker_report_contract_sha256={}",
                launch.authorization_id,
                launch.model_id,
                launch.reasoning_effort,
                launch.workbench_executable_path,
                launch.workbench_build_id,
                launch.supervisor_contract_version,
                launch.supervisor_contract_sha256,
                launch.worker_report_contract_sha256
            ),
            result_summary: "主管会话已启动；后续工具调用会落入同一账本。".to_string(),
            result_status: "accepted".to_string(),
            created_at_ms: started_at_ms,
        });
        Ok(())
    })
}

pub(crate) fn record_pilot_session_finished(
    config: &McpServerConfig,
    exit_code: Option<i32>,
) -> Result<(), String> {
    let ended_at_ms = now_ms();
    let launch_status = if exit_code == Some(0) {
        "exited"
    } else {
        "failed"
    };
    let termination_reason = match exit_code {
        Some(code) => format!(
            "主管 codex exec 退出码 {code}（仅表示进程结束；业务状态以权威 worker 回程和验收账本为准）"
        ),
        None => "主管 codex exec 被系统信号结束".to_string(),
    };
    update_store(config, "record-pilot-session-finished", |store| {
        let (final_status, final_reason) = {
            let session = session_mut(store, &config.run_id);
            if session.launch_status == "waiting_user" {
                session.ended_at_ms.get_or_insert(ended_at_ms);
                (
                    session.launch_status.clone(),
                    session.termination_reason.clone(),
                )
            } else {
                session.launch_status = launch_status.to_string();
                session.ended_at_ms = Some(ended_at_ms);
                session.termination_reason = termination_reason.clone();
                (
                    session.launch_status.clone(),
                    session.termination_reason.clone(),
                )
            }
        };
        store.audit_events.push(SupervisorAuditEvent {
            event_id: format!(
                "supervisor-orchestrator:{}:session-finished:{}",
                stable_fragment(&config.run_id),
                crate::unix_timestamp_nanos()
            ),
            actor: ACTOR.to_string(),
            run_id: config.run_id.clone(),
            tool: "supervisor_session_launcher".to_string(),
            parameter_summary: "等待主管 codex exec 收尾并注销进程登记。".to_string(),
            result_summary: final_reason,
            result_status: final_status,
            created_at_ms: ended_at_ms,
        });
        Ok(())
    })
}

pub(crate) fn record_pilot_protocol_invalid(
    config: &McpServerConfig,
    attempt: usize,
    detail: &str,
) -> Result<(), String> {
    let created_at_ms = now_ms();
    let waiting_user = attempt >= 2;
    let result_summary = if waiting_user {
        format!(
            "主管连续两次输出格式错误，当前无效动作未执行。{}第二次错误：{detail}",
            prior_worker_truth_prefix_for_store(config, "")?
        )
    } else {
        format!(
            "主管输出格式错误（第 {attempt} 次）：{detail}；当前无效动作未执行，系统已要求其按正确 JSON 格式纠正一次。"
        )
    };
    update_store(config, "record-pilot-protocol-invalid", |store| {
        if waiting_user {
            let session = session_mut(store, &config.run_id);
            session.launch_status = "waiting_user".to_string();
            session.ended_at_ms = Some(created_at_ms);
            session.termination_reason = format!(
                "主管连续两次输出格式错误，当前无效动作未执行。{}",
                prior_worker_truth(session)
            );
        }
        store.audit_events.push(SupervisorAuditEvent {
            event_id: format!(
                "supervisor-orchestrator:{}:protocol-invalid:{}",
                stable_fragment(&config.run_id),
                crate::unix_timestamp_nanos()
            ),
            actor: ACTOR.to_string(),
            run_id: config.run_id.clone(),
            tool: "supervisor_action_protocol".to_string(),
            parameter_summary: format!("attempt={attempt}"),
            result_summary,
            result_status: if waiting_user {
                "waiting_user".to_string()
            } else {
                "protocol_invalid".to_string()
            },
            created_at_ms,
        });
        Ok(())
    })
}

pub(crate) fn record_pilot_waiting_user(
    config: &McpServerConfig,
    reason: &str,
) -> Result<(), String> {
    let created_at_ms = now_ms();
    update_store(config, "record-pilot-waiting-user", |store| {
        let session = session_mut(store, &config.run_id);
        session.launch_status = "waiting_user".to_string();
        session.ended_at_ms = Some(created_at_ms);
        session.termination_reason = reason.to_string();
        store.audit_events.push(SupervisorAuditEvent {
            event_id: format!(
                "supervisor-orchestrator:{}:worker-return-waiting:{}",
                stable_fragment(&config.run_id),
                crate::unix_timestamp_nanos()
            ),
            actor: ACTOR.to_string(),
            run_id: config.run_id.clone(),
            tool: "supervisor_worker_return".to_string(),
            parameter_summary: "worker 回程未进入可验收状态。".to_string(),
            result_summary: reason.to_string(),
            result_status: "waiting_user".to_string(),
            created_at_ms,
        });
        Ok(())
    })
}

fn prior_worker_truth_prefix_for_store(
    config: &McpServerConfig,
    fallback: &str,
) -> Result<String, String> {
    let store = load_store(config)?;
    Ok(store
        .sessions
        .iter()
        .find(|session| session.run_id == config.run_id)
        .map(prior_worker_truth)
        .unwrap_or_else(|| fallback.to_string()))
}

fn prior_worker_truth(session: &SupervisorSession) -> String {
    let Some(worker) = session.workers.last() else {
        return "本单未执行。".to_string();
    };
    let result = if worker.last_result_summary.trim().is_empty() {
        "尚未获得结构化结果".to_string()
    } else {
        worker.last_result_summary.clone()
    };
    format!(
        "本单此前已派发 {} 个 worker；最近 worker {} 当前状态 {}，结果：{}。",
        session.workers.len(),
        worker.worker_id,
        worker.state,
        result
    )
}

pub(crate) fn record_pilot_temporary_home_created(
    config: &McpServerConfig,
    temporary_home: &Path,
) -> Result<(), String> {
    append_audit(
        config,
        "supervisor_temporary_codex_home",
        &format!(
            "action=created; temporary_home={}",
            temporary_home.display()
        ),
        "主管临时 CODEX_HOME 已创建；auth.json 仅为到 ~/.codex/auth.json 的符号链接。",
        "accepted",
    )
}

pub(crate) fn record_pilot_temporary_home_cleaned(
    config: &McpServerConfig,
    temporary_home: &Path,
    cleanup_trigger: &str,
    token_was_refreshed: bool,
    cleanup_succeeded: bool,
) -> Result<(), String> {
    let result_summary = if token_was_refreshed {
        "主管会话期间 token 被刷新,如遇登录失效请重登 codex".to_string()
    } else if cleanup_succeeded {
        "主管临时 CODEX_HOME 已清理。".to_string()
    } else {
        "主管临时 CODEX_HOME 清理未完成；保留现场以避免静默遗留凭据。".to_string()
    };
    let result_status = if token_was_refreshed || !cleanup_succeeded {
        "warning"
    } else {
        "accepted"
    };
    append_audit(
        config,
        "supervisor_temporary_codex_home",
        &format!(
            "action=cleaned; trigger={cleanup_trigger}; temporary_home={}",
            temporary_home.display()
        ),
        &result_summary,
        result_status,
    )
}

pub(crate) fn load_pilot_read_model(
    config: &McpServerConfig,
) -> Result<SupervisorPilotReadModel, String> {
    let store = load_store(config)?;
    let session = session(&store, &config.run_id)
        .cloned()
        .ok_or_else(|| "主管试点账本中找不到该 run-id".to_string())?;
    let audit_events = store
        .audit_events
        .iter()
        .filter(|event| event.run_id == config.run_id)
        .map(|event| SupervisorPilotAuditEventReadModel {
            event_id: event.event_id.clone(),
            tool: event.tool.clone(),
            result_summary: event.result_summary.clone(),
            result_status: if event.result_status.trim().is_empty() {
                if event.result_summary.starts_with("denied:") {
                    "denied".to_string()
                } else {
                    "legacy_unknown".to_string()
                }
            } else {
                event.result_status.clone()
            },
            created_at_ms: event.created_at_ms,
        })
        .collect::<Vec<_>>();
    let follow_up_count = session
        .workers
        .iter()
        .map(|worker| worker.follow_up_count)
        .sum::<usize>();
    let follow_up_budget_respected = session
        .workers
        .iter()
        .all(|worker| worker.follow_up_count <= session.max_follow_ups_per_worker);
    let denied_tool_call_count = audit_events
        .iter()
        .filter(|event| event.result_status == "denied")
        .count();
    let session_timed_out = session.termination_reason.contains("timed out")
        || session.termination_reason.contains("超时");
    let metrics = SupervisorPilotMetricsReadModel {
        denied_tool_call_count,
        max_follow_ups_per_worker: session.max_follow_ups_per_worker,
        follow_up_count,
        follow_up_budget_respected,
        max_runtime_minutes: session.max_runtime_minutes,
        session_timed_out,
        ledger_replay_event_count: audit_events.len(),
        ledger_replay_ready: !audit_events.is_empty(),
    };
    Ok(SupervisorPilotReadModel {
        run_id: session.run_id,
        launch_status: session.launch_status,
        project_root: session.project_root,
        workflow_id: session.workflow_id,
        authorization_id: session.authorization_id,
        started_at_ms: session.started_at_ms,
        ended_at_ms: session.ended_at_ms,
        termination_reason: session.termination_reason,
        metrics,
        audit_events,
    })
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
        if !launch.canonical_work_item_id.is_empty() {
            worker.work_item_id = launch.canonical_work_item_id.clone();
        }
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
        // 追问已获准进入原会话后，旧报告立即失效。即使 resume 或新回程解析失败，inspect 也不能
        // 回退到追问前的 last_report，避免主管拿旧事实终标。
        worker.last_report = None;
        worker.state = "waiting_follow_up".to_string();
        worker.last_result_summary = "worker 追问已发起，等待新报告。".to_string();
        Ok(())
    })
}

fn fail_follow_up(config: &McpServerConfig, worker_id: &str, error: &str) -> Result<(), String> {
    update_store(config, "fail-follow-up", |store| {
        let worker = session_mut(store, &config.run_id)
            .workers
            .iter_mut()
            .find(|worker| worker.worker_id == worker_id)
            .ok_or_else(|| "主管当前会话没有该 worker，拒绝写追问失败结果".to_string())?;
        worker.state = "follow_up_failed".to_string();
        worker.last_report = None;
        worker.last_result_summary =
            crate::run_error_translation::humanize_error_for_display(error);
        Ok(())
    })
}

fn update_worker_follow_up_result(
    config: &McpServerConfig,
    worker_id: &str,
    follow_up: &WorkerFollowUp,
) -> Result<(), String> {
    update_store(config, "update-worker-follow-up-result", |store| {
        let worker = session_mut(store, &config.run_id)
            .workers
            .iter_mut()
            .find(|worker| worker.worker_id == worker_id)
            .ok_or_else(|| "主管当前会话没有该 worker，拒绝写追问结果".to_string())?;
        worker.state = "completed".to_string();
        worker.last_report = Some(follow_up.report.clone());
        worker.last_result_summary = follow_up.summary.clone();
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
    result_status: &str,
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
            result_status: result_status.to_string(),
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
                canonical_work_item_id: String::new(),
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
            worker: &SupervisorWorker,
            prompt: &str,
        ) -> Result<WorkerFollowUp, String> {
            Ok(WorkerFollowUp {
                summary: format!("mock follow-up: {prompt}"),
                report: json!({
                    "worker_id": worker.worker_id,
                    "dispatch_id": worker.dispatch_id,
                    "acceptance_status": "reported_completed",
                    "executed_what": "answered follow-up",
                    "changed_what": "worker 未列出产出文件",
                    "summary": "fresh follow-up report",
                    "evidence_refs": ["follow-up:evidence"],
                    "findings": ["fresh follow-up finding"],
                    "open_issues": [],
                    "permission_requests": [],
                    "direction_risks": [],
                    "follow_up_suggestions": []
                }),
            })
        }
    }

    struct FailingFollowUpInvoker;

    impl WorkerInvoker for FailingFollowUpInvoker {
        fn dispatch(
            &self,
            _config: &McpServerConfig,
            _input: &DispatchInput,
        ) -> Result<WorkerLaunch, String> {
            unreachable!("failure fixture only exercises follow-up")
        }

        fn follow_up(
            &self,
            _config: &McpServerConfig,
            _worker: &SupervisorWorker,
            _prompt: &str,
        ) -> Result<WorkerFollowUp, String> {
            Err("report_invalid: fresh follow-up report is malformed".to_string())
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

        fn control_core(&self, action: &str, arguments: Value) -> Result<Value, String> {
            control_core_call_with_invoker(&self.config, action, &arguments, &FakeInvoker)
        }

        fn dispatch(&self) -> Result<Value, String> {
            self.control_core(
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
        assert!(fixture.control_core("dispatch_worker", json!({"project_root": PROJECT, "workflow_id": WORKFLOW, "authorization_id": "bad", "node_id": NODE, "work_item_id": "work-1", "allowed_write": [PROJECT]})).is_err());
        fixture.dispatch().expect("dispatch");
        assert_eq!(fixture.audit_count("control_core_dispatch_worker"), 2);
    }

    #[test]
    fn pilot_session_start_audits_actual_workbench_binary_and_build_id() {
        let fixture = Fixture::new();
        record_pilot_session_started(
            &fixture.config,
            &SupervisorPilotSessionLaunch {
                project_root: PROJECT.to_string(),
                workflow_id: WORKFLOW.to_string(),
                authorization_id: AUTH.to_string(),
                model_id: "account-default".to_string(),
                reasoning_effort: "medium".to_string(),
                workbench_executable_path: "/Applications/CodexGovernanceWorkbench.app/Contents/MacOS/CodexGovernanceWorkbench".to_string(),
                workbench_build_id: "codex-governance-workbench@0.1.0:bytes=123:mtime=1784000000:sha256=binary-hash".to_string(),
                supervisor_contract_version: "supervisor_action_proposal.v1".to_string(),
                supervisor_contract_sha256: "supervisor-hash".to_string(),
                worker_report_contract_sha256: "worker-hash".to_string(),
            },
        )
        .expect("pilot launch audit");
        let ledger = load_store(&fixture.config).expect("pilot ledger");
        let start_event = ledger
            .audit_events
            .iter()
            .find(|event| event.tool == "supervisor_session_launcher")
            .expect("session start audit");
        assert!(start_event.parameter_summary.contains("workbench_executable_path=/Applications/CodexGovernanceWorkbench.app/Contents/MacOS/CodexGovernanceWorkbench"));
        assert!(start_event.parameter_summary.contains(
            "workbench_build_id=codex-governance-workbench@0.1.0:bytes=123:mtime=1784000000:sha256=binary-hash"
        ));
        assert!(start_event
            .parameter_summary
            .contains("supervisor_contract_version=supervisor_action_proposal.v1"));
        assert!(start_event
            .parameter_summary
            .contains("supervisor_contract_sha256=supervisor-hash"));
        assert!(start_event
            .parameter_summary
            .contains("worker_report_contract_sha256=worker-hash"));
    }

    #[test]
    fn canonicalizes_model_work_item_typo_only_from_unique_authorized_prepared_dispatch() {
        let input = DispatchInput {
            project_root: PROJECT.to_string(),
            workflow_id: WORKFLOW.to_string(),
            authorization_id: AUTH.to_string(),
            node_id: NODE.to_string(),
            work_item_id: "work-item:workflow-users-yoyi-codex-workflow-mario-test:default:wrong"
                .to_string(),
            allowed_write: vec![PROJECT.to_string()],
        };
        let canonical = "work-item:workflow:users-yoyi-codex-workflow-mario-test:default:prepared";
        let value = json!({
            "workflow_node_dispatches": [{
                "state": "prepared",
                "prompt_kind": "authorized_prepared_auto_dispatch",
                "project_id": crate::project_id(PROJECT),
                "workflow_id": WORKFLOW,
                "node_id": NODE,
                "plan_authorization_id": AUTH,
                "work_item_id": canonical
            }]
        });
        assert_eq!(
            canonical_prepared_work_item_id_from_value(&value, &input).expect("canonical id"),
            canonical
        );
    }

    #[test]
    fn refuses_to_guess_work_item_when_authorized_prepared_dispatch_is_ambiguous() {
        let input = DispatchInput {
            project_root: PROJECT.to_string(),
            workflow_id: WORKFLOW.to_string(),
            authorization_id: AUTH.to_string(),
            node_id: NODE.to_string(),
            work_item_id: "wrong".to_string(),
            allowed_write: vec![PROJECT.to_string()],
        };
        let prepared = |work_item_id: &str| {
            json!({
                "state": "prepared",
                "prompt_kind": "authorized_prepared_auto_dispatch",
                "project_id": crate::project_id(PROJECT),
                "workflow_id": WORKFLOW,
                "node_id": NODE,
                "plan_authorization_id": AUTH,
                "work_item_id": work_item_id
            })
        };
        let value = json!({"workflow_node_dispatches": [prepared("one"), prepared("two")]});
        assert!(canonical_prepared_work_item_id_from_value(&value, &input)
            .expect_err("ambiguous prepared dispatch must fail")
            .contains("无法唯一恢复"));
    }

    #[test]
    fn supervisor_dispatch_fresh_session_gate_uses_history_and_exact_work_item_binding() {
        let value = json!({
            "workflow_node_dispatches": [
                {"native_thread_id": "thread-v3"},
                {"native_thread_id": "thread-v4"},
                {"native_thread_id": null}
            ],
            "workflow_node_session_bindings": [
                {
                    "workflow_id": WORKFLOW,
                    "node_id": NODE,
                    "work_item_id": "work-v5",
                    "native_thread_id": "thread-v5-fresh",
                    "lifecycle": "active"
                },
                {
                    "workflow_id": WORKFLOW,
                    "node_id": NODE,
                    "work_item_id": null,
                    "native_thread_id": "thread-v3",
                    "lifecycle": "active"
                }
            ],
            "audit_events": [{
                "event_type": "supervisor_task_session_birth",
                "native_thread_id": "thread-created-but-rejected"
            }],
            "artifacts": [{
                "artifact_id": "artifact:work-v5",
                "artifact_type": "task_package",
                "source_ref": "work-v5",
                "task_name": "Worker：创建 station3a-control-core-proof-v5.txt"
            }]
        });
        let historical = historical_native_thread_ids(&value);
        assert_eq!(
            historical,
            BTreeSet::from([
                "thread-v3".to_string(),
                "thread-v4".to_string(),
                "thread-v5-fresh".to_string(),
                "thread-created-but-rejected".to_string()
            ])
        );
        assert_eq!(
            exact_work_item_native_thread_id(&value, WORKFLOW, NODE, "work-v5"),
            Some("thread-v5-fresh".to_string()),
            "work item binding must win over old node-level binding"
        );
        assert_eq!(
            supervisor_task_session_metadata(&value, "work-v5").expect("task package metadata"),
            (
                "Worker：创建 station3a-control-core-proof-v5.txt".to_string(),
                "artifact:work-v5".to_string()
            )
        );
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
    fn station3a_v3_raw_worker_return_is_strict_and_preserves_blocked_signal() {
        let fixture = Fixture::new();
        fixture.dispatch().expect("dispatch");
        let last_message_path = fixture.root.join("worker-last-message.txt");
        let mut state: Value =
            serde_json::from_slice(&fs::read(&fixture.state_path).expect("workflow state"))
                .expect("workflow state json");
        state["workflow_node_dispatches"] = json!([{
            "dispatch_id": "dispatch-1",
            "last_message_path": last_message_path
        }]);
        fs::write(
            &fixture.state_path,
            serde_json::to_vec(&state).expect("workflow state json"),
        )
        .expect("write workflow state");
        update_store(&fixture.config, "clear-fake-initial-report", |store| {
            session_mut(store, &fixture.config.run_id).workers[0].last_report = None;
            Ok(())
        })
        .expect("clear fake report");

        fs::write(&last_message_path, "not a JSON report").expect("bad worker return");
        let invalid = read_worker_report(&fixture.config, &json!({"worker_id": "worker-1"}))
            .expect_err("bad worker return must be report invalid");
        assert!(invalid.contains("report_invalid"));

        fs::write(
            &last_message_path,
            "```json\n{\"did\":\"无法继续\",\"outputs\":[],\"status\":\"blocked\",\"evidence\":[],\"open_issues\":[\"缺少确认\"]}\n```",
        )
        .expect("blocked worker return");
        let blocked = read_worker_report(&fixture.config, &json!({"worker_id": "worker-1"}))
            .expect("blocked report projection");
        assert_eq!(blocked["acceptance_status"], "blocked");
        assert_eq!(blocked["summary"], "无法继续");
        assert_eq!(
            load_store(&fixture.config)
                .expect("stored blocked report")
                .sessions[0]
                .workers[0]
                .last_report,
            Some(blocked.clone())
        );
        update_store(
            &fixture.config,
            "clear-blocked-report-for-audit-projection",
            |store| {
                session_mut(store, &fixture.config.run_id).workers[0].last_report = None;
                Ok(())
            },
        )
        .expect("clear blocked report for next projection source");

        let mut state: Value =
            serde_json::from_slice(&fs::read(&fixture.state_path).expect("workflow state"))
                .expect("workflow state json");
        state["audit_events"] = json!([{
            "event_type": "worker_structured_report_recorded",
            "dispatch_id": "dispatch-1",
            "acceptance_status": "reported_completed",
            "executed_what": "写入 proof",
            "changed_what": "station3a-control-core-proof-v3.txt",
            "reason": "文件已回读",
            "evidence_refs": ["readback:proof"],
            "open_issues": [],
            "permission_requests": [],
            "direction_risks": [],
            "follow_up_suggestions": []
        }]);
        fs::write(
            &fixture.state_path,
            serde_json::to_vec(&state).expect("workflow state json"),
        )
        .expect("write structured report audit");
        let projected = read_worker_report(&fixture.config, &json!({"worker_id": "worker-1"}))
            .expect("structured report projection");
        assert_eq!(projected["summary"], "文件已回读");
        assert_eq!(projected["evidence_refs"], json!(["readback:proof"]));
        assert_eq!(
            load_store(&fixture.config)
                .expect("stored completed report")
                .sessions[0]
                .workers[0]
                .last_report,
            Some(projected)
        );
    }

    // 站 3b 首单缺口 end-to-end 回归：worker 只读侦察把结论写进 findings（并误塞自造的
    // promise_verdicts/top_5_issues）→ 主管 read_worker_report 投影必须带出 findings，
    // 且不因结论存在而误判 blocked。修此前主管只见摘要、误判证据不足并试图越权 follow_up。
    #[test]
    fn station3b_readonly_findings_reach_supervisor_projection() {
        let fixture = Fixture::new();
        fixture.dispatch().expect("dispatch");
        let last_message_path = fixture.root.join("worker-last-message.txt");
        let mut state: Value =
            serde_json::from_slice(&fs::read(&fixture.state_path).expect("workflow state"))
                .expect("workflow state json");
        state["workflow_node_dispatches"] = json!([{
            "dispatch_id": "dispatch-1",
            "last_message_path": last_message_path
        }]);
        fs::write(
            &fixture.state_path,
            serde_json::to_vec(&state).expect("workflow state json"),
        )
        .expect("write workflow state");
        update_store(&fixture.config, "clear-fake-initial-report", |store| {
            session_mut(store, &fixture.config.run_id).workers[0].last_report = None;
            Ok(())
        })
        .expect("clear fake report");

        fs::write(
            &last_message_path,
            "```json\n{\"did\":\"只读盘点完成，未写入任何文件\",\"outputs\":[],\"status\":\"done\",\"evidence\":[\"node --check game.js 退出码 0\"],\"findings\":[\"P0 game.js:137 未按 delta 缩放，原文 \\\"player.x += player.vx;\\\"\",\"总评：核心玩法齐全\"],\"promise_verdicts\":[{\"verdict\":\"已实现\"}],\"top_5_issues\":[{\"rank\":1}]}\n```",
        )
        .expect("readonly worker return with findings");
        let projected = read_worker_report(&fixture.config, &json!({"worker_id": "worker-1"}))
            .expect("readonly report projection");
        // 结论正文到达主管：findings 两条都在投影里。
        assert_eq!(
            projected["findings"],
            json!([
                "P0 game.js:137 未按 delta 缩放，原文 \"player.x += player.vx;\"",
                "总评：核心玩法齐全"
            ])
        );
        // findings 是正常产出、不是求助：仍判 reported_completed（不误判 blocked）。
        assert_eq!(projected["acceptance_status"], "reported_completed");
        // 自造顶层字段不出现在投影（struct 无此字段，serde 丢弃）。
        assert!(projected.get("promise_verdicts").is_none());
        assert!(projected.get("top_5_issues").is_none());
    }

    #[test]
    fn station3a_v3_second_protocol_error_after_dispatch_keeps_prior_worker_truth() {
        let fixture = Fixture::new();
        fixture.dispatch().expect("dispatch");
        record_pilot_protocol_invalid(&fixture.config, 1, "未知字段 target")
            .expect("first protocol diagnostic");
        record_pilot_protocol_invalid(&fixture.config, 2, "target.worker_id 不允许")
            .expect("second protocol diagnostic");
        let read_model = load_pilot_read_model(&fixture.config).expect("read model");
        assert_eq!(read_model.launch_status, "waiting_user");
        assert!(read_model
            .termination_reason
            .contains("此前已派发 1 个 worker"));
        assert!(!read_model.termination_reason.contains("本单未执行"));
        assert!(read_model
            .audit_events
            .iter()
            .any(|event| event.result_summary.contains("当前无效动作未执行")));
    }

    #[test]
    fn follow_up_denies_unknown_worker_and_audits_success() {
        let fixture = Fixture::new();
        assert!(fixture
            .control_core(
                "follow_up_worker",
                json!({"worker_id": "missing", "prompt": "what changed?"})
            )
            .is_err());
        fixture.dispatch().expect("dispatch");
        let followed_up = fixture
            .control_core(
                "follow_up_worker",
                json!({"worker_id": "worker-1", "prompt": "what changed?"}),
            )
            .expect("follow up");
        assert_eq!(followed_up["report"]["summary"], "fresh follow-up report");
        let readback = fixture
            .control_core("inspect_worker", json!({"worker_id": "worker-1"}))
            .expect("read fresh follow-up report");
        assert_eq!(readback["summary"], "fresh follow-up report");
        assert_eq!(readback["findings"], json!(["fresh follow-up finding"]));
        assert_eq!(fixture.audit_count("control_core_follow_up_worker"), 2);
    }

    #[test]
    fn failed_follow_up_invalidates_cached_report_and_inspect_refuses_stale_fallback() {
        let fixture = Fixture::new();
        fixture.dispatch().expect("dispatch");
        assert!(read_worker_report(&fixture.config, &json!({"worker_id": "worker-1"})).is_ok());

        let error = follow_up_worker(
            &fixture.config,
            &json!({"worker_id": "worker-1", "prompt": "补充证据"}),
            &FailingFollowUpInvoker,
        )
        .expect_err("malformed follow-up must fail");
        assert!(error.contains("report_invalid"));

        let stale = read_worker_report(&fixture.config, &json!({"worker_id": "worker-1"}))
            .expect_err("old report must stay invalid after failed follow-up");
        assert!(stale.contains("旧报告已失效"));
        let worker = find_worker(&fixture.config, "worker-1").expect("worker");
        assert_eq!(worker.state, "follow_up_failed");
        assert!(worker.last_report.is_none());
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
        assert!(fixture.control_core("finalize", json!({"project_root": PROJECT, "workflow_id": WORKFLOW, "authorization_id": AUTH, "verdict": "completed", "reason": "no"})).is_err());
        fixture.control_core("finalize", json!({"project_root": PROJECT, "workflow_id": WORKFLOW, "authorization_id": AUTH, "verdict": "needs_rework", "reason": "mock yellow"})).expect("mark");
        assert_eq!(
            fs::read_to_string(&fixture.state_path).expect("after"),
            before,
            "advisory must not mutate chain state"
        );
        assert_eq!(fixture.audit_count("control_core_finalize"), 2);
    }

    #[test]
    fn report_user_denies_empty_message_and_audits_success() {
        let fixture = Fixture::new();
        assert!(fixture.control_core("report_user", json!({"project_root": PROJECT, "workflow_id": WORKFLOW, "authorization_id": AUTH, "message": ""})).is_err());
        fixture.control_core("report_user", json!({"project_root": PROJECT, "workflow_id": WORKFLOW, "authorization_id": AUTH, "message": "mock report"})).expect("report user");
        assert_eq!(fixture.audit_count("control_core_report_user"), 2);
    }

    #[test]
    fn mock_end_to_end_dispatch_report_and_advisory_leave_chain_running() {
        let fixture = Fixture::new();
        fixture.dispatch().expect("dispatch");
        fixture
            .call("read_worker_report", json!({"worker_id": "worker-1"}))
            .expect("report");
        fixture.control_core("finalize", json!({"project_root": PROJECT, "workflow_id": WORKFLOW, "authorization_id": AUTH, "verdict": "pass", "reason": "mock accepted"})).expect("final");
        let state: Value =
            serde_json::from_str(&fs::read_to_string(&fixture.state_path).expect("state"))
                .expect("json");
        assert_eq!(state["workflow_chain_runs"][0]["status"], "running");
    }

    #[test]
    fn station3a_mcp_toolface_is_read_only_and_rejects_side_effect_names() {
        let fixture = Fixture::new();
        let toolface = list_tools();
        let tools = toolface["tools"].as_array().expect("tools array");
        let names = tools
            .iter()
            .filter_map(|tool| tool["name"].as_str())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            names,
            BTreeSet::from(["read_key_file", "read_worker_report", "wait_for_worker"])
        );
        for rejected in [
            "dispatch_worker",
            "follow_up_worker",
            "final_mark",
            "report_user",
        ] {
            assert!(
                fixture.call(rejected, json!({})).is_err(),
                "{rejected} must not be an MCP action"
            );
        }
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
