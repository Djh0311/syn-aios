/// Project-scoped supervisor conversation state.  The thread and its private
/// CODEX_HOME outlive a single `codex exec` process; the process itself never does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SupervisorResidentSessionState {
    pub(crate) project_id: String,
    pub(crate) project_root: String,
    pub(crate) workflow_id: String,
    pub(crate) thread_id: String,
    pub(crate) host_pid: u32,
    pub(crate) generation: u64,
    pub(crate) launch_status: String,
    pub(crate) active_message_id: String,
    pub(crate) active_proposal_outcome: String,
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
        .map(resident_session_state))
}

/// Lifecycle reconciliation also needs to see a first `codex exec` that died
/// before emitting `thread.started`.  It deliberately does not authorize a
/// conversation continuation: callers must still use `load_resident_session`
/// when they require a durable thread binding.
pub(crate) fn load_resident_turn_for_reconciliation(
    config: &McpServerConfig,
) -> Result<Option<SupervisorResidentSessionState>, String> {
    let store = load_store(config)?;
    Ok(session(&store, &config.run_id).map(resident_session_state))
}

fn resident_session_state(session: &SupervisorSession) -> SupervisorResidentSessionState {
    SupervisorResidentSessionState {
        project_id: session.resident_project_id.clone(),
        project_root: session.project_root.clone(),
        workflow_id: session.workflow_id.clone(),
        thread_id: session.resident_thread_id.clone(),
        host_pid: session.resident_host_pid,
        generation: session.resident_generation,
        launch_status: session.launch_status.clone(),
        active_message_id: session.resident_active_message_id.clone(),
        active_proposal_outcome: session.resident_active_proposal_outcome.clone(),
    }
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
        "项目主管会话已创建；本轮一次一发进程正在建立同一 threadId。",
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
        "项目主管会话已复用；本轮通过一次一发 codex exec resume 续接既有 threadId。",
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
            "项目主管会话已换代；原因={replacement_reason}。新 threadId 已用黑板既有条目与正式记忆重新注入。"
        ),
    )
}

/// Record a finite `codex exec` turn immediately after its process group exists,
/// but before the initial turn has emitted `thread.started`.  During this short
/// state the private MCP server must not accept `submit_proposal`: it waits for
/// the durable thread binding instead of accidentally authorizing a prior
/// generation's thread.
pub(crate) fn record_resident_turn_prepared(
    config: &McpServerConfig,
    project_root: &str,
    workflow_id: &str,
    host_pid: u32,
    generation: u64,
    active_message_id: Option<&str>,
) -> Result<(), String> {
    if project_root.trim().is_empty() || workflow_id.trim().is_empty() || host_pid == 0 {
        return Err("supervisor_resident_turn_prepared_identity_incomplete".to_string());
    }
    let created_at_ms = now_ms();
    let project_id = crate::project_id(project_root);
    let active_message_id = active_message_id
        .map(str::trim)
        .filter(|message_id| !message_id.is_empty());
    update_store(
        config,
        "record-supervisor-resident-turn-prepared",
        |store| {
            let session = session_mut(store, &config.run_id);
            if !session.project_root.trim().is_empty() && session.project_root != project_root {
                return Err("supervisor_resident_session_project_binding_mismatch".to_string());
            }
            session.project_root = project_root.to_string();
            session.workflow_id = workflow_id.to_string();
            session.resident_project_id = project_id.clone();
            session.resident_host_pid = host_pid;
            session.resident_generation = generation;
            match active_message_id {
                Some(message_id) if session.resident_active_message_id != message_id => {
                    session.resident_active_message_id = message_id.to_string();
                    session.resident_active_proposal_outcome = "not_requested".to_string();
                }
                Some(_) => {}
                None => {
                    session.resident_active_message_id.clear();
                    session.resident_active_proposal_outcome.clear();
                }
            }
            session.launch_status = "resident_turn_starting".to_string();
            session.started_at_ms = created_at_ms;
            session.ended_at_ms = None;
            session.termination_reason.clear();
            store.audit_events.push(SupervisorAuditEvent {
            event_id: format!(
                "supervisor-orchestrator:{}:resident-turn-prepared:{}",
                stable_fragment(&config.run_id),
                crate::unix_timestamp_nanos()
            ),
            actor: ACTOR.to_string(),
            run_id: config.run_id.clone(),
            tool: "supervisor_resident_session".to_string(),
            event_type: "supervisor_resident_turn_prepared".to_string(),
            parameter_summary: format!(
                "project_id={project_id}; workflow_id={workflow_id}; host_pid={host_pid}; generation={generation}; active_message_id={}",
                active_message_id.unwrap_or("none")
            ),
            result_summary: "主管一次一发进程已启动，正在等待 thread.started 的持久化绑定。".to_string(),
            result_status: "accepted".to_string(),
            created_at_ms,
        });
            Ok(())
        },
    )
}

/// Record only a server-observed proposal result for the currently bound user
/// turn.  The raw MCP/CLI detail remains owner-only audit data; callers expose
/// only the stable outcome name through the existing answer command.
pub(crate) fn record_resident_active_proposal_outcome(
    config: &McpServerConfig,
    message_id: &str,
    outcome: &str,
    raw_detail: Option<&str>,
) -> Result<(), String> {
    if message_id.trim().is_empty() || !matches!(outcome, "materialized" | "tool_failed") {
        return Err("supervisor_resident_proposal_outcome_invalid".to_string());
    }
    let created_at_ms = now_ms();
    update_store(
        config,
        "record-supervisor-resident-proposal-outcome",
        |store| {
            let session = session_mut(store, &config.run_id);
            if session.resident_active_message_id != message_id {
                return Err("supervisor_resident_proposal_outcome_message_mismatch".to_string());
            }
            if session.resident_active_proposal_outcome == "materialized"
                || session.resident_active_proposal_outcome == outcome
            {
                return Ok(());
            }
            session.resident_active_proposal_outcome = outcome.to_string();
            store.audit_events.push(SupervisorAuditEvent {
                event_id: format!(
                    "supervisor-orchestrator:{}:resident-proposal-outcome:{}",
                    stable_fragment(&config.run_id),
                    crate::unix_timestamp_nanos()
                ),
                actor: ACTOR.to_string(),
                run_id: config.run_id.clone(),
                tool: "supervisor_resident_session".to_string(),
                event_type: "supervisor_resident_proposal_outcome".to_string(),
                parameter_summary: format!(
                    "message_id={message_id}; outcome={outcome}; raw_detail={}",
                    raw_detail.unwrap_or("none")
                ),
                result_summary: if outcome == "materialized" {
                    "主管本回合的结构化方案已落为待用户确认卡。".to_string()
                } else {
                    "主管本回合已完成，但结构化方案动作未完成；已保留私有诊断。".to_string()
                },
                result_status: if outcome == "materialized" {
                    "accepted".to_string()
                } else {
                    "warning".to_string()
                },
                created_at_ms,
            });
            Ok(())
        },
    )
}

/// Preserve a non-terminal runtime diagnostic for the currently bound user
/// turn without claiming that `submit_proposal` ran or failed.  The caller may
/// finish a normal conversation after a transport diagnostic; only the tool
/// handler itself is allowed to change the proposal outcome.
pub(crate) fn record_resident_turn_recoverable_diagnostic(
    config: &McpServerConfig,
    message_id: &str,
    raw_detail: &str,
) -> Result<(), String> {
    if message_id.trim().is_empty() || raw_detail.trim().is_empty() {
        return Err("supervisor_resident_recoverable_diagnostic_invalid".to_string());
    }
    let created_at_ms = now_ms();
    update_store(
        config,
        "record-supervisor-resident-recoverable-diagnostic",
        |store| {
            let session = session_mut(store, &config.run_id);
            if session.resident_active_message_id != message_id {
                return Err(
                    "supervisor_resident_recoverable_diagnostic_message_mismatch".to_string(),
                );
            }
            store.audit_events.push(SupervisorAuditEvent {
                event_id: format!(
                    "supervisor-orchestrator:{}:resident-recoverable-diagnostic:{}",
                    stable_fragment(&config.run_id),
                    crate::unix_timestamp_nanos()
                ),
                actor: ACTOR.to_string(),
                run_id: config.run_id.clone(),
                tool: "supervisor_resident_session".to_string(),
                event_type: "supervisor_resident_recoverable_diagnostic".to_string(),
                parameter_summary: format!("message_id={message_id}; raw_detail={raw_detail}"),
                result_summary: "主管本回合已完成；运行诊断已保留，不影响对话结果。".to_string(),
                result_status: "warning".to_string(),
                created_at_ms,
            });
            Ok(())
        },
    )
}

pub(crate) fn resident_active_proposal_outcome(
    config: &McpServerConfig,
    message_id: &str,
) -> Result<String, String> {
    let session = load_resident_turn_for_reconciliation(config)?
        .ok_or_else(|| "supervisor_resident_session_binding_missing".to_string())?;
    if session.active_message_id != message_id {
        return Err("supervisor_resident_active_message_mismatch".to_string());
    }
    Ok(if session.active_proposal_outcome.trim().is_empty() {
        "not_requested".to_string()
    } else {
        session.active_proposal_outcome
    })
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
                result_summary: "项目咨询/主管拆解已并入一次一发主管会话。".to_string(),
                result_status: "accepted".to_string(),
                created_at_ms,
            });
            Ok(())
        },
    )
}

/// The child is fully reaped before this is written.  The thread binding remains
/// durable so the next user message can resume it, while `host_pid=0` means there
/// is no resident guardian to attach to or leak.
pub(crate) fn record_resident_turn_exited(
    config: &McpServerConfig,
    termination_reason: &str,
) -> Result<(), String> {
    let created_at_ms = now_ms();
    update_store(config, "record-supervisor-resident-turn-exited", |store| {
        let session = session_mut(store, &config.run_id);
        let pid = session.resident_host_pid;
        session.launch_status = "resident_exited".to_string();
        session.ended_at_ms = Some(created_at_ms);
        session.termination_reason = termination_reason.to_string();
        session.resident_host_pid = 0;
        store.audit_events.push(SupervisorAuditEvent {
            event_id: format!(
                "supervisor-orchestrator:{}:resident-turn-exited:{}",
                stable_fragment(&config.run_id),
                crate::unix_timestamp_nanos()
            ),
            actor: ACTOR.to_string(),
            run_id: config.run_id.clone(),
            tool: "supervisor_resident_session".to_string(),
            event_type: "supervisor_resident_turn_exited".to_string(),
            parameter_summary: format!("pid={pid}; termination_reason={termination_reason}"),
            result_summary: "主管一次一发进程已结束；会话 threadId 保留供下一轮 resume。"
                .to_string(),
            result_status: "accepted".to_string(),
            created_at_ms,
        });
        Ok(())
    })
}

/// Keep the original CLI rejection in the owner-only audit parameters and the
/// private stderr artifact.  `result_summary` deliberately stays human-facing:
/// it is projected into the pilot read model.
pub(crate) fn record_resident_invalid_resume_detected(
    config: &McpServerConfig,
    classification: &str,
    exit_code: Option<i32>,
    thread_started_event_seen: bool,
    stderr_path: Option<&std::path::Path>,
    raw_detail: &str,
) -> Result<(), String> {
    if classification.trim().is_empty() {
        return Err("supervisor_resident_invalid_resume_classification_missing".to_string());
    }
    let created_at_ms = now_ms();
    let exit_code = exit_code
        .map(|value| value.to_string())
        .unwrap_or_else(|| "unknown".to_string());
    let stderr_artifact = stderr_path
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "none".to_string());
    update_store(
        config,
        "record-supervisor-resident-invalid-resume",
        |store| {
            store.audit_events.push(SupervisorAuditEvent {
                event_id: format!(
                    "supervisor-orchestrator:{}:resident-invalid-resume:{}",
                    stable_fragment(&config.run_id),
                    crate::unix_timestamp_nanos()
                ),
                actor: ACTOR.to_string(),
                run_id: config.run_id.clone(),
                tool: "supervisor_resident_session".to_string(),
                event_type: "supervisor_resident_invalid_resume_detected".to_string(),
                parameter_summary: format!(
                    "classification={classification}; exit_code={exit_code}; thread_started_event_seen={thread_started_event_seen}; stderr_artifact={stderr_artifact}; raw_detail={raw_detail}"
                ),
                result_summary: "主管已有对话已失效；系统将基于项目事实换代，并仅重试一次。"
                    .to_string(),
                result_status: "warning".to_string(),
                created_at_ms,
            });
            Ok(())
        },
    )
}

/// Never erase the PID when the TERM/KILL sweep itself failed.  The process
/// registry keeps its own durable entry too; this sidecar state makes the
/// unresolved cleanup visible and prevents a later turn from being described
/// as a clean exit.
pub(crate) fn record_resident_turn_cleanup_failed(
    config: &McpServerConfig,
    detail: &str,
) -> Result<(), String> {
    if detail.trim().is_empty() {
        return Err("supervisor_resident_cleanup_failure_detail_missing".to_string());
    }
    let created_at_ms = now_ms();
    update_store(
        config,
        "record-supervisor-resident-turn-cleanup-failed",
        |store| {
            let session = session_mut(store, &config.run_id);
            let pid = session.resident_host_pid;
            if pid == 0 {
                return Err("supervisor_resident_cleanup_failure_pid_missing".to_string());
            }
            session.launch_status = "resident_turn_cleanup_failed".to_string();
            session.ended_at_ms = None;
            session.termination_reason = format!("process_group_cleanup_failed:{detail}");
            store.audit_events.push(SupervisorAuditEvent {
                event_id: format!(
                    "supervisor-orchestrator:{}:resident-turn-cleanup-failed:{}",
                    stable_fragment(&config.run_id),
                    crate::unix_timestamp_nanos()
                ),
                actor: ACTOR.to_string(),
                run_id: config.run_id.clone(),
                tool: "supervisor_resident_session".to_string(),
                event_type: "supervisor_resident_turn_cleanup_failed".to_string(),
                parameter_summary: format!("pid={pid}"),
                result_summary: format!(
                    "主管一次一发进程组未能确认清理；保留 PID 供后续进程组对账：{detail}"
                ),
                result_status: "warning".to_string(),
                created_at_ms,
            });
            Ok(())
        },
    )
}

/// Startup reconciliation never kills a PID from a stale record.  The caller may
/// mark it exited only after it independently established that this exact PID no
/// longer exists.
pub(crate) fn record_resident_stale_turn_reaped(
    config: &McpServerConfig,
    stale_pid: u32,
) -> Result<bool, String> {
    if stale_pid == 0 {
        return Ok(false);
    }
    let created_at_ms = now_ms();
    update_store(
        config,
        "record-supervisor-resident-stale-turn-reaped",
        |store| {
            let session = session_mut(store, &config.run_id);
            if !matches!(
                session.launch_status.as_str(),
                "resident_turn_starting" | "resident_turn_running" | "resident_turn_cleanup_failed"
            ) || session.resident_host_pid != stale_pid
            {
                return Ok(false);
            }
            session.launch_status = "resident_exited".to_string();
            session.ended_at_ms = Some(created_at_ms);
            session.termination_reason = "startup_stale_pid_not_alive".to_string();
            session.resident_host_pid = 0;
            store.audit_events.push(SupervisorAuditEvent {
                event_id: format!(
                    "supervisor-orchestrator:{}:resident-stale-reaped:{}",
                    stable_fragment(&config.run_id),
                    crate::unix_timestamp_nanos()
                ),
                actor: ACTOR.to_string(),
                run_id: config.run_id.clone(),
                tool: "supervisor_resident_session".to_string(),
                event_type: "supervisor_resident_session_stale_reaped".to_string(),
                parameter_summary: format!("stale_pid={stale_pid}"),
                result_summary: "启动时发现主管一次一发进程已不存在，已只清理陈旧会话状态。"
                    .to_string(),
                result_status: "accepted".to_string(),
                created_at_ms,
            });
            Ok(true)
        },
    )
}

pub(crate) fn record_resident_liveness_unavailable(
    config: &McpServerConfig,
    pid: u32,
    detail: &str,
) -> Result<(), String> {
    append_audit(
        config,
        "supervisor_resident_session",
        &format!("action=stale_scan_unavailable; pid={pid}"),
        &format!("主管会话陈账 PID 无法核验，未擅自改写状态：{detail}"),
        "warning",
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
            if session.launch_status == "resident_turn_starting"
                && session.resident_host_pid != host_pid
            {
                return Err("supervisor_resident_turn_started_pid_mismatch".to_string());
            }
            session.project_root = project_root.to_string();
            if !workflow_id.trim().is_empty() {
                session.workflow_id = workflow_id.to_string();
            }
            session.resident_project_id = project_id.clone();
            session.resident_thread_id = thread_id.to_string();
            session.resident_host_pid = host_pid;
            session.resident_generation = generation;
            session.launch_status = "resident_turn_running".to_string();
            session.started_at_ms = created_at_ms;
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
