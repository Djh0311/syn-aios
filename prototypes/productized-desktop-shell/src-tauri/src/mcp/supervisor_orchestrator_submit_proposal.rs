use super::{load_resident_session, workflow_state_path, McpServerConfig};
use serde_json::{json, Value};
use std::thread;
use std::time::{Duration, Instant};

const RESIDENT_RUN_ID_PREFIX: &str = "supervisor-resident:";
const RESIDENT_THREAD_BINDING_WAIT: Duration = Duration::from_secs(3);
const RESIDENT_THREAD_BINDING_POLL: Duration = Duration::from_millis(10);

pub(super) fn is_resident_supervisor_run(config: &McpServerConfig) -> bool {
    config
        .run_id
        .strip_prefix(RESIDENT_RUN_ID_PREFIX)
        .is_some_and(|suffix| !suffix.trim().is_empty())
}

pub(super) fn error_summary_for_read_model(
    config: &McpServerConfig,
    name: &str,
    error: &str,
) -> String {
    if name == "submit_proposal" && is_resident_supervisor_run(config) {
        "denied: 主管本回合的方案卡没有生成；已保留私有诊断。".to_string()
    } else {
        format!(
            "denied: {}",
            crate::run_error_translation::humanize_error_for_display(error)
        )
    }
}

pub(super) fn tool_result_summary(
    config: &McpServerConfig,
    name: &str,
    result: &Result<Value, String>,
) -> String {
    match result {
        Ok(value) => super::summary_of_value(value),
        Err(error) => error_summary_for_read_model(config, name, error),
    }
}

pub(super) fn audit_write_failure_for_caller(
    config: &McpServerConfig,
    name: &str,
    result_status: &str,
    raw_error: String,
) -> String {
    if name == "submit_proposal" && is_resident_supervisor_run(config) {
        if result_status == "accepted" {
            "方案已落卡，但工具审计未完成。".to_string()
        } else {
            "方案卡没有生成；内部审计未完成。".to_string()
        }
    } else {
        raw_error
    }
}

pub(super) fn input_schema() -> Value {
    let string_array = json!({"type": "array", "items": {"type": "string"}});
    let risk = json!({
        "type": "object",
        "properties": {
            "severity": {"type": "string"},
            "summary": {"type": "string"},
            "mitigation": {"type": "string"}
        },
        "required": ["severity", "summary", "mitigation"],
        "additionalProperties": false
    });
    let execution_scope = json!({
        "type": "object",
        "properties": {
            "requires_write": {"type": "boolean"},
            "write_roots": string_array.clone(),
            "target_files": string_array.clone(),
            "tools": string_array.clone(),
            "checks": string_array.clone()
        },
        "required": ["requires_write", "write_roots", "target_files", "tools", "checks"],
        "additionalProperties": false
    });
    let task = json!({
        "type": "object",
        "properties": {
            "title": {"type": "string"},
            "task_goal": {"type": "string"},
            "target_role": {"type": "string"},
            "depends_on": string_array.clone(),
            "acceptance_criteria": string_array.clone(),
            "report_format": string_array.clone()
        },
        "required": ["title", "task_goal", "target_role", "depends_on", "acceptance_criteria", "report_format"],
        "additionalProperties": false
    });
    json!({
        "type": "object",
        "properties": {
            "user_goal": {"type": "string"},
            "goal_summary": {"type": "string"},
            "scope_note": {"type": "string"},
            "reasoning": string_array.clone(),
            "risks": {"type": "array", "items": risk},
            "must_stop_points": string_array.clone(),
            "next_steps": string_array.clone(),
            "worker_acceptance_criteria": string_array.clone(),
            "control_core_acceptance_criteria": string_array.clone(),
            "supervisor_acceptance_criteria": string_array.clone(),
            "execution_scope": {"anyOf": [{"type": "null"}, execution_scope]},
            "suggest_workflow": {"type": "boolean"},
            "tasks": {"type": "array", "items": task}
        },
        "required": [
            "user_goal",
            "goal_summary",
            "scope_note",
            "reasoning",
            "risks",
            "must_stop_points",
            "next_steps",
            "worker_acceptance_criteria",
            "control_core_acceptance_criteria",
            "supervisor_acceptance_criteria",
            "execution_scope",
            "suggest_workflow",
            "tasks"
        ],
        "additionalProperties": false
    })
}

pub(super) fn submit(config: &McpServerConfig, arguments: &Value) -> Result<Value, String> {
    if !is_resident_supervisor_run(config) {
        return Err(
            "submit_proposal 只允许项目常驻主管私有会话调用，当前会话未获授权。".to_string(),
        );
    }
    let session = wait_for_resident_turn_binding(config)?;
    if session.project_id.trim().is_empty()
        || session.project_root.trim().is_empty()
        || session.workflow_id.trim().is_empty()
        || session.thread_id.trim().is_empty()
        || session.host_pid == 0
        || session.launch_status != "resident_turn_running"
        || session.active_message_id.trim().is_empty()
    {
        return Err(
            "submit_proposal 已拒绝：常驻主管会话的 project/root/workflow/thread 绑定不完整。"
                .to_string(),
        );
    }
    if session.project_id != crate::project_id(&session.project_root) {
        return Err("submit_proposal 已拒绝：常驻主管项目身份与项目根不一致。".to_string());
    }
    let proposal = match submit_for_bound_resident_user_turn(config, arguments, &session) {
        Ok(proposal) => proposal,
        Err(error) => {
            if super::record_resident_active_proposal_outcome(
                config,
                &session.active_message_id,
                "tool_failed",
                Some(&error),
            )
            .is_err()
            {
                return Err("方案卡没有生成；内部审计未完成。".to_string());
            }
            return Err("方案卡没有生成；详细诊断已保留。".to_string());
        }
    };
    if super::record_resident_active_proposal_outcome(
        config,
        &session.active_message_id,
        "materialized",
        None,
    )
    .is_err()
    {
        return Err("方案已落卡，但内部审计未完成。".to_string());
    }
    let proposal_json = serde_json::to_value(&proposal)
        .map_err(|_| "方案已落卡，但工具回执未完成。".to_string())?;
    Ok(json!({
        "status": "proposal_created_pending_user_confirmation",
        "proposal_id": proposal_json.get("proposal_id").cloned().unwrap_or(Value::Null),
        "message": "方案已落为待用户确认卡；尚未批准，工作流未推进。"
    }))
}

fn submit_for_bound_resident_user_turn(
    config: &McpServerConfig,
    arguments: &Value,
    session: &super::SupervisorResidentSessionState,
) -> Result<crate::ProjectConsultationProposal, String> {
    let proposal = crate::parse_supervisor_submit_proposal_arguments(arguments)
        .map_err(|error| format!("方案参数不合法，未落卡：{error}"))?;
    let user_message = crate::supervisor_session_launcher::supervisor_resident_active_user_message(
        config,
        &session.workflow_id,
        &session.active_message_id,
    )
    .map_err(|error| format!("无法确认本回合真实用户消息，未落卡：{error}"))?;
    if user_message.message_text.trim().is_empty() {
        return Err("无法确认本回合真实用户消息，未落卡。".to_string());
    }
    let idempotency_key = crate::utils::hash::sha256_hex(&format!(
        "{}:{}:{}",
        session.project_id, session.workflow_id, user_message.message_id
    ));
    crate::write_consultation_proposal_for_resident_turn(
        workflow_state_path(config)?,
        &proposal,
        &session.project_root,
        &user_message.message_text,
        &format!("supervisor-resident:{idempotency_key}"),
        &idempotency_key,
    )
    .map_err(|error| format!("方案未落卡：{error}"))
}

/// A new `codex exec` process emits `thread.started` asynchronously while its
/// private MCP server may already receive a tool call.  A prepared turn is not
/// authorization: wait briefly for the parent process to durably bind that
/// exact turn, and otherwise fail closed rather than borrowing an older thread.
fn wait_for_resident_turn_binding(
    config: &McpServerConfig,
) -> Result<super::SupervisorResidentSessionState, String> {
    let deadline = Instant::now() + RESIDENT_THREAD_BINDING_WAIT;
    loop {
        match load_resident_session(config)
            .map_err(|_| "submit_proposal 已拒绝：无法读取可验证的常驻主管会话绑定。".to_string())?
        {
            Some(session)
                if session.launch_status == "resident_turn_running" && session.host_pid != 0 =>
            {
                return Ok(session);
            }
            Some(session)
                if session.launch_status == "resident_turn_starting"
                    && Instant::now() < deadline =>
            {
                thread::sleep(RESIDENT_THREAD_BINDING_POLL);
            }
            None if Instant::now() < deadline => thread::sleep(RESIDENT_THREAD_BINDING_POLL),
            Some(_) => {
                return Err(
                    "submit_proposal 已拒绝：本回合未在时限内完成可验证的 thread 绑定。"
                        .to_string(),
                );
            }
            None => {
                return Err(
                    "submit_proposal 已拒绝：未找到可验证的常驻主管会话绑定（本回合没有可用 durable binding）。"
                        .to_string(),
                );
            }
        }
    }
}
