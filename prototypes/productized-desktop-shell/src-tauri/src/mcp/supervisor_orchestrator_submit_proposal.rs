use super::{load_resident_session, workflow_state_path, McpServerConfig};
use serde_json::{json, Value};

const RESIDENT_RUN_ID_PREFIX: &str = "supervisor-resident:";

pub(super) fn is_resident_supervisor_run(config: &McpServerConfig) -> bool {
    config
        .run_id
        .strip_prefix(RESIDENT_RUN_ID_PREFIX)
        .is_some_and(|suffix| !suffix.trim().is_empty())
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
    let session = load_resident_session(config)?
        .ok_or_else(|| "submit_proposal 已拒绝：未找到可验证的常驻主管会话绑定。".to_string())?;
    if session.project_id.trim().is_empty()
        || session.project_root.trim().is_empty()
        || session.workflow_id.trim().is_empty()
        || session.thread_id.trim().is_empty()
    {
        return Err(
            "submit_proposal 已拒绝：常驻主管会话的 project/root/workflow/thread 绑定不完整。"
                .to_string(),
        );
    }
    if session.project_id != crate::project_id(&session.project_root) {
        return Err("submit_proposal 已拒绝：常驻主管项目身份与项目根不一致。".to_string());
    }
    let proposal = crate::parse_supervisor_submit_proposal_arguments(arguments)
        .map_err(|error| format!("方案参数不合法，未落卡：{error}"))?;
    let snapshot =
        crate::supervisor_session_launcher::supervisor_resident_latest_user_message_snapshot(
            config,
            &session.workflow_id,
        )
        .map_err(|error| format!("无法确认真实用户消息，未落卡：{error}"))?;
    if snapshot.trim().is_empty() {
        return Err("无法确认真实用户消息，未落卡。".to_string());
    }
    let proposal = crate::write_consultation_proposal(
        workflow_state_path(config)?,
        &proposal,
        &session.project_root,
        &snapshot,
        &format!("supervisor-resident:{}", session.thread_id),
    )
    .map_err(|error| format!("方案未落卡：{error}"))?;
    let proposal_json = serde_json::to_value(&proposal)
        .map_err(|error| format!("方案已落库但无法生成工具回执：{error}"))?;
    Ok(json!({
        "status": "proposal_created_pending_user_confirmation",
        "proposal_id": proposal_json.get("proposal_id").cloned().unwrap_or(Value::Null),
        "message": "方案已落为待用户确认卡；尚未批准，工作流未推进。"
    }))
}
