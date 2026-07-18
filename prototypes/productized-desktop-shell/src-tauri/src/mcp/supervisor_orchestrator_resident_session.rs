/// P1-A read model for the project-scoped resident MCP host. The process itself is
/// intentionally in-memory; this record is the durable mapping needed to audit a
/// create/reuse/replacement and to seed the next generation with facts instead of a
/// transcript.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SupervisorResidentSessionState {
    pub(crate) project_id: String,
    pub(crate) project_root: String,
    pub(crate) workflow_id: String,
    pub(crate) thread_id: String,
    pub(crate) host_pid: u32,
    pub(crate) generation: u64,
}

fn is_zero_u32(value: &u32) -> bool {
    *value == 0
}

fn is_zero_u64(value: &u64) -> bool {
    *value == 0
}

pub(crate) fn load_resident_session(
    config: &McpServerConfig,
) -> Result<Option<SupervisorResidentSessionState>, String> {
    let store = load_store(config)?;
    Ok(session(&store, &config.run_id)
        .filter(|session| !session.resident_thread_id.trim().is_empty())
        .map(|session| SupervisorResidentSessionState {
            project_id: session.resident_project_id.clone(),
            project_root: session.project_root.clone(),
            workflow_id: session.workflow_id.clone(),
            thread_id: session.resident_thread_id.clone(),
            host_pid: session.resident_host_pid,
            generation: session.resident_generation,
        }))
}

pub(crate) fn record_resident_session_created(
    config: &McpServerConfig,
    project_root: &str,
    workflow_id: &str,
    thread_id: &str,
    host_pid: u32,
    generation: u64,
) -> Result<(), String> {
    record_resident_session_event(
        config,
        project_root,
        workflow_id,
        thread_id,
        host_pid,
        generation,
        "supervisor_resident_session_created",
        "项目主管常驻会话已创建；后续项目消息必须以同一 threadId 续接。",
    )
}

pub(crate) fn record_resident_session_reused(
    config: &McpServerConfig,
    project_root: &str,
    workflow_id: &str,
    thread_id: &str,
    host_pid: u32,
    generation: u64,
) -> Result<(), String> {
    record_resident_session_event(
        config,
        project_root,
        workflow_id,
        thread_id,
        host_pid,
        generation,
        "supervisor_resident_session_reused",
        "项目主管常驻会话已复用；本轮通过 codex-reply 续接既有 threadId。",
    )
}

pub(crate) fn record_resident_session_replaced(
    config: &McpServerConfig,
    project_root: &str,
    workflow_id: &str,
    thread_id: &str,
    host_pid: u32,
    generation: u64,
    replacement_reason: &str,
) -> Result<(), String> {
    record_resident_session_event(
        config,
        project_root,
        workflow_id,
        thread_id,
        host_pid,
        generation,
        "supervisor_resident_session_replaced",
        &format!(
            "项目主管常驻宿主已换代；原因={replacement_reason}。新 threadId 已用黑板既有条目与正式记忆重新注入。"
        ),
    )
}

pub(crate) fn record_resident_consult_merged(
    config: &McpServerConfig,
    project_root: &str,
    workflow_id: &str,
    prompt_kind: &str,
) -> Result<(), String> {
    let created_at_ms = now_ms();
    let project_id = crate::project_id(project_root);
    update_store(
        config,
        "record-supervisor-resident-consult-merged",
        |store| {
            store.audit_events.push(SupervisorAuditEvent {
                event_id: format!(
                    "supervisor-orchestrator:{}:resident-consult-merged:{}",
                    stable_fragment(&config.run_id),
                    crate::unix_timestamp_nanos()
                ),
                actor: ACTOR.to_string(),
                run_id: config.run_id.clone(),
                tool: "supervisor_resident_session".to_string(),
                event_type: "supervisor_resident_consult_merged".to_string(),
                parameter_summary: format!(
                    "project_id={project_id}; workflow_id={workflow_id}; prompt_kind={prompt_kind}"
                ),
                result_summary: "项目咨询/主管拆解已并入常驻主管会话。".to_string(),
                result_status: "accepted".to_string(),
                created_at_ms,
            });
            Ok(())
        },
    )
}

fn record_resident_session_event(
    config: &McpServerConfig,
    project_root: &str,
    workflow_id: &str,
    thread_id: &str,
    host_pid: u32,
    generation: u64,
    event_type: &str,
    result_summary: &str,
) -> Result<(), String> {
    if project_root.trim().is_empty() || thread_id.trim().is_empty() || host_pid == 0 {
        return Err("supervisor_resident_session_identity_incomplete".to_string());
    }
    let created_at_ms = now_ms();
    let project_id = crate::project_id(project_root);
    update_store(config, &format!("record-{event_type}"), |store| {
        {
            let session = session_mut(store, &config.run_id);
            if !session.project_root.trim().is_empty() && session.project_root != project_root {
                return Err("supervisor_resident_session_project_binding_mismatch".to_string());
            }
            session.project_root = project_root.to_string();
            if !workflow_id.trim().is_empty() {
                session.workflow_id = workflow_id.to_string();
            }
            session.resident_project_id = project_id.clone();
            session.resident_thread_id = thread_id.to_string();
            session.resident_host_pid = host_pid;
            session.resident_generation = generation;
            session.launch_status = "resident_running".to_string();
            session.started_at_ms = if session.started_at_ms == 0 {
                created_at_ms
            } else {
                session.started_at_ms
            };
            session.ended_at_ms = None;
            session.termination_reason.clear();
        }
        store.audit_events.push(SupervisorAuditEvent {
            event_id: format!(
                "supervisor-orchestrator:{}:{}:{}",
                stable_fragment(&config.run_id),
                stable_fragment(event_type),
                crate::unix_timestamp_nanos()
            ),
            actor: ACTOR.to_string(),
            run_id: config.run_id.clone(),
            tool: "supervisor_resident_session".to_string(),
            event_type: event_type.to_string(),
            parameter_summary: format!(
                "project_id={project_id}; workflow_id={workflow_id}; thread_id={thread_id}; host_pid={host_pid}; generation={generation}"
            ),
            result_summary: result_summary.to_string(),
            result_status: "accepted".to_string(),
            created_at_ms,
        });
        Ok(())
    })
}
