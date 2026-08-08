// Workflow dispatch execution control, offline role dispatch, and workflow machine helpers split out during Root Treatment R2-B6.
// This file is included at crate root so helper visibility and behavior stay unchanged.

fn inspect_workflow_node_dispatch_authorization(
    path: &Path,
    context: &WorkflowNodeDispatchContext,
) -> Result<AutoDispatchGuardResult, String> {
    // The authorization inspection itself records an audit receipt.  In
    // DB-primary mode that must not become an early JSON-side write after a
    // post-commit projection failure: the caller has not yet reserved a
    // prepared dispatch, but a new authorization audit would still create a
    // second writer.  Keep the check at the shared production entrypoint so
    // prepare and supervisor follow-up cannot bypass it.
    let _ = crate::workbench_sqlite_storage_mode::primary_repository_for_m2_t2_fail_closed_write(
        path,
        "workflow_node_dispatch_authorization_inspect",
    )?;
    let mut requested_read_roots = Vec::new();
    let mut requested_write_roots = Vec::new();
    let mut requested_tools = Vec::new();
    if let Some(instruction) = &context.user_reviewed_instruction {
        requested_read_roots.extend(instruction.allowed_reads.clone());
        requested_write_roots.extend(instruction.allowed_write_roots.clone());
        requested_write_roots.extend(instruction.allowed_writes.clone());
        requested_tools.push("codex_exec_resume".to_string());
    }
    let input = AutoDispatchGuardInput {
        project_id: context.project_id.clone(),
        workflow_id: context.workflow_id.clone(),
        work_item_id: context.work_item_id.clone(),
        task_package_id: context.memory_packet_snapshot_id.clone(),
        task_package_kind: Some(context.prompt_kind.clone()),
        target_role_id: role_id_from_node_id(&context.node_id),
        target_agent_id: Some(context.native_thread_id.clone()),
        requested_read_roots,
        requested_write_roots,
        requested_tools,
        requested_checks: vec![],
        triggered_stop_conditions: vec![],
        dispatch_kind: "prepare_real".to_string(),
    };
    plan_authorization_store::inspect_auto_dispatch_authorization(
        path,
        &input,
        unix_timestamp_ms(),
        &format!("write-node-dispatch-auth-check-{}", unix_timestamp_nanos()),
    )
}

fn ensure_authorized_for_prepare(result: &AutoDispatchGuardResult) -> Result<(), String> {
    if result.status == "authorized" {
        return Ok(());
    }
    let reason = if result.reasons.is_empty() {
        "方案授权检查未通过".to_string()
    } else {
        result.reasons.join("；")
    };
    Err(format!("方案授权检查未通过，已拒绝准备派发：{reason}"))
}

fn role_id_from_node_id(node_id: &str) -> String {
    node_id
        .rsplit(":node:")
        .next()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "project_director".to_string())
}

fn ensure_valid_dispatch_state(value: &Value) -> Result<(), String> {
    let mut cloned = value.clone();
    ensure_workflow_node_session_bindings_array(&mut cloned)?;
    ensure_workflow_node_dispatches_array(&mut cloned)?;
    let validation_warnings = validate_workflow_state(&cloned);
    if !validation_warnings.is_empty() {
        return Err(format!(
            "当前状态文件未通过 schema 校验：{}",
            validation_warnings.join(", ")
        ));
    }
    Ok(())
}

fn validate_user_reviewed_instruction(
    instruction: &UserReviewedInstructionInput,
) -> Result<(), String> {
    for (label, value) in [
        ("instruction_id", instruction.instruction_id.as_str()),
        ("summary", instruction.summary.as_str()),
        ("objective", instruction.objective.as_str()),
        ("execution_cwd", instruction.execution_cwd.as_str()),
        ("sandbox_mode", instruction.sandbox_mode.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(format!("用户审核业务指令缺少 {label}"));
        }
    }
    if !matches!(
        instruction.sandbox_mode.trim(),
        "read-only" | "workspace-write"
    ) {
        return Err(
            "用户审核业务指令 sandbox_mode 只允许 read-only 或 workspace-write".to_string(),
        );
    }
    if instruction.timeout_seconds <= 0 {
        return Err("用户审核业务指令 timeout_seconds 必须大于 0".to_string());
    }
    if instruction.max_retries < 0 {
        return Err("用户审核业务指令 max_retries 不能为负数".to_string());
    }
    if instruction.max_retries > 0 {
        return Err("用户审核业务指令 max_retries 当前只允许 0；自动重试还未产品化".to_string());
    }
    if instruction.allowed_reads.is_empty() {
        return Err("用户审核业务指令缺少 allowed_reads".to_string());
    }
    if instruction.forbidden_actions.is_empty() {
        return Err("用户审核业务指令缺少 forbidden_actions".to_string());
    }
    if instruction.required_return.is_empty() {
        return Err("用户审核业务指令缺少 required_return".to_string());
    }
    if instruction.sandbox_mode.trim() == "workspace-write"
        && instruction.allowed_write_roots.is_empty()
    {
        return Err("workspace-write 业务派发必须提供 allowed_write_roots".to_string());
    }
    Ok(())
}

fn user_reviewed_instruction_value(instruction: &UserReviewedInstructionInput) -> Value {
    json!({
      "instruction_id": instruction.instruction_id,
      "summary": instruction.summary,
      "objective": instruction.objective,
      "execution_cwd": instruction.execution_cwd,
      "sandbox_mode": instruction.sandbox_mode,
      "allowed_write_roots": instruction.allowed_write_roots,
      "allowed_reads": instruction.allowed_reads,
      "allowed_writes": instruction.allowed_writes,
      "forbidden_actions": instruction.forbidden_actions,
      "timeout_seconds": instruction.timeout_seconds,
      "max_retries": instruction.max_retries,
      "required_return": instruction.required_return,
      "prompt_preview": instruction.prompt_preview
    })
}

fn user_reviewed_instruction_input_from_value(
    value: &Value,
) -> Result<UserReviewedInstructionInput, String> {
    Ok(UserReviewedInstructionInput {
        instruction_id: optional_string_from(value, "instruction_id")
            .ok_or_else(|| "user_reviewed_instruction 缺 instruction_id".to_string())?,
        summary: optional_string_from(value, "summary").unwrap_or_default(),
        objective: optional_string_from(value, "objective").unwrap_or_default(),
        execution_cwd: optional_string_from(value, "execution_cwd").unwrap_or_default(),
        sandbox_mode: optional_string_from(value, "sandbox_mode").unwrap_or_default(),
        allowed_write_roots: string_array(value, "allowed_write_roots"),
        allowed_reads: string_array(value, "allowed_reads"),
        allowed_writes: string_array(value, "allowed_writes"),
        forbidden_actions: string_array(value, "forbidden_actions"),
        timeout_seconds: i64_value(value, "timeout_seconds").unwrap_or_default(),
        max_retries: i64_value(value, "max_retries").unwrap_or_default(),
        required_return: string_array(value, "required_return"),
        prompt_preview: optional_string_from(value, "prompt_preview"),
    })
}

fn codex_resume_options_for_context(
    context: &WorkflowNodeDispatchContext,
) -> Result<CodexResumeRequestOptions, String> {
    if context.prompt_kind == "safe_probe" {
        return Ok(CodexResumeRequestOptions {
            prompt_kind: context.prompt_kind.clone(),
            execution_cwd: None,
            sandbox_mode: None,
            allowed_write_roots: vec![],
            timeout_seconds: None,
        });
    }
    let instruction = context
        .user_reviewed_instruction
        .as_ref()
        .ok_or_else(|| "用户审核模式缺少完整派发字段，已阻止真实业务派发".to_string())?;
    Ok(CodexResumeRequestOptions {
        prompt_kind: context.prompt_kind.clone(),
        execution_cwd: Some(PathBuf::from(instruction.execution_cwd.trim())),
        sandbox_mode: Some(instruction.sandbox_mode.trim().to_string()),
        allowed_write_roots: instruction
            .allowed_write_roots
            .iter()
            .map(|root| PathBuf::from(root.trim()))
            .collect(),
        timeout_seconds: Some(instruction.timeout_seconds),
    })
}

fn render_user_reviewed_business_prompt(instruction: &UserReviewedInstructionInput) -> String {
    if let Some(preview) = instruction
        .prompt_preview
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    {
        return preview.to_string();
    }
    format!(
        "你将执行一条用户审核过的真实业务指令。\n\n指令 ID：{}\n摘要：{}\n\n目标：\n{}\n\n执行目录：\n{}\n\n沙箱模式：\n{}\n\n允许读取：\n{}\n\n允许写入：\n{}\n\n允许写入根目录：\n{}\n\n禁止事项：\n{}\n\n超时秒数：{}\n最大重试：{}\n\n必须回传：\n{}",
        instruction.instruction_id,
        instruction.summary,
        instruction.objective,
        instruction.execution_cwd,
        instruction.sandbox_mode,
        markdown_list_or_empty(&instruction.allowed_reads),
        markdown_list_or_empty(&instruction.allowed_writes),
        markdown_list_or_empty(&instruction.allowed_write_roots),
        markdown_list_or_empty(&instruction.forbidden_actions),
        instruction.timeout_seconds,
        instruction.max_retries,
        markdown_list_or_empty(&instruction.required_return)
    )
}

fn classify_codex_resume_failure(
    exit_code: i32,
    timed_out: bool,
    instruction: &Option<UserReviewedInstructionInput>,
    error: &str,
) -> Vec<String> {
    let mut warnings = Vec::new();
    // fix8：codex resume 的 stderr（error）命中供给类特征→加 provider 标签，供 is_tier1_early_exit
    // 排除重试 + UI 上脸。只加报告标签，不改判决体（仍按 exit_code 走 write_failed_dispatch）。
    if let Some(human) = codex_local_runner::classify_codex_provider_failure(error) {
        warnings.push(format!("codex_provider_unavailable:{human}"));
    }
    if timed_out {
        warnings.push("timeout".to_string());
    }
    if exit_code != 0 && !timed_out {
        warnings.push("codex_resume_exit_nonzero".to_string());
        warnings.push(format!("codex_exec_resume_exit_code_{exit_code}"));
    }
    if exit_code == -1 && !error.trim().is_empty() {
        warnings.push("codex_resume_spawn_failed".to_string());
    }
    if let Some(instruction) = instruction {
        if instruction.sandbox_mode.trim() == "read-only" && !instruction.allowed_writes.is_empty()
        {
            warnings.push("sandbox_read_only".to_string());
        }
        if instruction.sandbox_mode.trim() == "workspace-write"
            && instruction.allowed_write_roots.is_empty()
        {
            warnings.push("allowed_write_roots_missing".to_string());
        }
        if instruction.sandbox_mode.trim() == "workspace-write"
            && !instruction.allowed_writes.is_empty()
            && !instruction.allowed_write_roots.is_empty()
            && instruction.allowed_writes.iter().any(|allowed_write| {
                let trimmed = allowed_write.trim();
                trimmed.starts_with('/')
                    && !instruction
                        .allowed_write_roots
                        .iter()
                        .any(|root| trimmed.starts_with(root.trim()))
            })
        {
            warnings.push("target_path_not_writable".to_string());
        }
    }
    if !error.trim().is_empty() {
        warnings.push(compact_failure_warning(error));
    }
    dedupe_strings(warnings)
}

fn compact_failure_warning(error: &str) -> String {
    let compact = error.split_whitespace().collect::<Vec<_>>().join("_");
    let trimmed = compact.trim_matches('_');
    if trimmed.is_empty() {
        "codex_resume_failed".to_string()
    } else {
        trimmed.chars().take(120).collect()
    }
}

fn dedupe_strings(values: Vec<String>) -> Vec<String> {
    let mut seen = BTreeSet::new();
    values
        .into_iter()
        .filter(|value| seen.insert(value.clone()))
        .collect()
}

// P1-D 人闸收敛·A2：派发两笔审计的 actor/reason 一律改真话——「project_director 自动…」，不再冒称
// 「用户确认」。实测锁死(§A4 mock E2E)：合流命令→C1 每任务派发这条**最常走的路**在这一层已经拿不到
// plan_authorization_id 了(execute_project_workflow_node_at 的未授权分支——prepare 阶段已经把授权
// scope 核过，execute 阶段复用现成 gated _at、不重传授权对象，见 commands.rs 调用链)，条件式「有 id 才
// 改真话/没 id 走老文案」在这条主路径上恒假、等于没改。改成一律真话 + 有 id 就顺手引用（主管试点经
// execute_authorized_project_workflow_node_at 传了 id，能拿到）；旧画布入口 execute_workflow_node_dispatch
// （唯一前端调用者 WorkflowCanvasEngine.tsx）复用同一份代码，文案连带改真——recon 已点名知情，不代表它
// 变成了自动，只是同一处 audit 写点措辞统一，不产生新的越权语义（S1/path-lock/guard 三支闸零改零碰）。
fn dispatch_prepared_audit_actor_and_reason(
    plan_authorization_id: Option<&str>,
    prompt_kind: &str,
) -> (&'static str, &'static str, String) {
    let base = if prompt_kind == "safe_probe" {
        "项目主管自动准备工作流节点 Codex safe probe 派发；只写工作台状态，不发送消息。".to_string()
    } else {
        "项目主管自动准备工作流节点用户审核业务派发；只写工作台状态，不发送消息。".to_string()
    };
    let reason = match plan_authorization_id {
        Some(authorization_id) => format!("{base}（active 授权 {authorization_id}）"),
        None => base,
    };
    ("project_director", "plan_authorized_prepared", reason)
}

fn dispatch_started_audit_actor_and_reason(
    plan_authorization_id: Option<&str>,
    prompt_kind: &str,
) -> (&'static str, &'static str, String) {
    let base = if prompt_kind == "safe_probe" {
        "项目主管自动向绑定 Codex 会话发送 safe probe；会写 /Users/yoyi/.codex 和工作台 workflow state。".to_string()
    } else {
        "项目主管自动向绑定 Codex 会话发送用户审核业务指令；会写 /Users/yoyi/.codex、工作台 workflow state 和用户允许的业务路径。".to_string()
    };
    let reason = match plan_authorization_id {
        Some(authorization_id) => format!("{base}（active 授权 {authorization_id}）"),
        None => base,
    };
    ("project_director", "plan_authorized_dispatched", reason)
}

fn write_prepared_dispatch(
    path: &Path,
    context: WorkflowNodeDispatchContext,
) -> Result<WorkflowNodeDispatchResult, String> {
    if let Some(repository) =
        crate::workbench_sqlite_storage_mode::primary_repository_for_m2_t2_fail_closed_write(
            path,
            "workflow_node_dispatch_prepared",
        )?
    {
        return write_prepared_dispatch_db_primary(path, context, &repository);
    }
    let timestamp = unix_timestamp_string();
    let timestamp_ms = unix_timestamp_ms();
    let mut value = read_workflow_state_value(path)?;
    ensure_workflow_node_dispatches_array(&mut value)?;
    let backup = backup_workflow_state_file(path, &timestamp)?;
    let dispatch_id = next_workflow_node_dispatch_id(&context, &timestamp);
    let dispatch = json!({
      "dispatch_id": dispatch_id,
      "project_id": context.project_id,
      "workflow_id": context.workflow_id,
      "node_id": context.node_id,
      "work_item_id": context.work_item_id,
      "binding_id": context.binding_id,
      "native_thread_id": context.native_thread_id,
      "prompt_preview": context.prompt_preview,
      "worker_prompt": context.prompt_preview,
      "prompt_kind": context.prompt_kind,
      "memory_packet_snapshot_id": context.memory_packet_snapshot_id,
      "memory_packet_fingerprint": context.memory_packet_fingerprint,
      "plan_authorization_id": context.plan_authorization_id,
      "authorization_check": context.authorization_check.as_ref().map(|check| serde_json::to_value(check).unwrap_or(Value::Null)).unwrap_or(Value::Null),
      "user_reviewed_instruction": context.user_reviewed_instruction.as_ref().map(user_reviewed_instruction_value).unwrap_or(Value::Null),
      "state": "prepared",
      "started_at_ms": Value::Null,
      "ended_at_ms": Value::Null,
      "exit_code": Value::Null,
      "last_message_path": Value::Null,
      "last_message_summary": Value::Null,
      "transcript_event_count": Value::Null,
      "transcript_target_hits": Value::Null,
      "warnings": context.warnings,
      "created_at_ms": timestamp_ms,
      "updated_at_ms": timestamp_ms
    });
    array_mut(&mut value, "workflow_node_dispatches")?.push(dispatch);
    let audit_event_id = crate::workflow_audit::audit_event_identity(
        "workflow-node-dispatch-prepared",
        &dispatch_id,
        &timestamp,
    );
    let (actor_ref, permission_level, reason) = dispatch_prepared_audit_actor_and_reason(
        context.plan_authorization_id.as_deref(),
        &context.prompt_kind,
    );
    array_mut(&mut value, "audit_events")?.push(json!({
      "event_id": audit_event_id,
      "event_type": "workflow_node_dispatch_prepared",
      "target_ref": context.work_item_id,
      "actor_ref": actor_ref,
      "source_kind": "workspace_state",
      "permission_level": permission_level,
      "before_state": context.work_item_state,
      "after_state": "prepared",
      "created_at": timestamp,
      "reason": reason
    }));
    value["updated_at"] = Value::String(timestamp);
    write_validated_workflow_state(path, &value)?;
    dispatch_result_from_state(
        path,
        Some(backup),
        &audit_event_id,
        &dispatch_id,
        "已准备节点派发。",
    )
}

fn write_prepared_dispatch_db_primary(
    path: &Path,
    context: WorkflowNodeDispatchContext,
    repository: &crate::workbench_sqlite_repository::WorkbenchSqliteRepository,
) -> Result<WorkflowNodeDispatchResult, String> {
    let timestamp = unix_timestamp_string();
    let timestamp_ms = unix_timestamp_ms();
    let mut value = read_workflow_state_value(path)?;
    ensure_workflow_node_dispatches_array(&mut value)?;
    let dispatch_id = next_workflow_node_dispatch_id(&context, &timestamp);
    let dispatch = json!({
      "dispatch_id": dispatch_id,
      "project_id": context.project_id,
      "workflow_id": context.workflow_id,
      "node_id": context.node_id,
      "work_item_id": context.work_item_id,
      "binding_id": context.binding_id,
      "native_thread_id": context.native_thread_id,
      "prompt_preview": context.prompt_preview,
      "worker_prompt": context.prompt_preview,
      "prompt_kind": context.prompt_kind,
      "memory_packet_snapshot_id": context.memory_packet_snapshot_id,
      "memory_packet_fingerprint": context.memory_packet_fingerprint,
      "plan_authorization_id": context.plan_authorization_id,
      "authorization_check": context.authorization_check.as_ref().map(|check| serde_json::to_value(check).unwrap_or(Value::Null)).unwrap_or(Value::Null),
      "user_reviewed_instruction": context.user_reviewed_instruction.as_ref().map(user_reviewed_instruction_value).unwrap_or(Value::Null),
      "state": "prepared",
      "started_at_ms": Value::Null,
      "ended_at_ms": Value::Null,
      "exit_code": Value::Null,
      "last_message_path": Value::Null,
      "last_message_summary": Value::Null,
      "transcript_event_count": Value::Null,
      "transcript_target_hits": Value::Null,
      "warnings": context.warnings,
      "created_at_ms": timestamp_ms,
      "updated_at_ms": timestamp_ms
    });
    array_mut(&mut value, "workflow_node_dispatches")?.push(dispatch.clone());
    let audit_event_id = crate::workflow_audit::audit_event_identity(
        "workflow-node-dispatch-prepared",
        &dispatch_id,
        &timestamp,
    );
    let (actor_ref, permission_level, reason) = dispatch_prepared_audit_actor_and_reason(
        context.plan_authorization_id.as_deref(),
        &context.prompt_kind,
    );
    let audit_event = json!({
      "event_id": audit_event_id.clone(),
      "event_type": "workflow_node_dispatch_prepared",
      "target_ref": dispatch["work_item_id"].clone(),
      "actor_ref": actor_ref,
      "source_kind": "workspace_state",
      "permission_level": permission_level,
      "before_state": context.work_item_state,
      "after_state": "prepared",
      "created_at": timestamp.clone(),
      "reason": reason
    });
    array_mut(&mut value, "audit_events")?.push(audit_event.clone());
    value["updated_at"] = Value::String(timestamp.clone());
    repository.record_dispatch_with_audit(
        &dispatch,
        &crate::workbench_sqlite_repository::RepositoryAuditEntry {
            event_id: audit_event_id.clone(),
            target_kind: "workflow_state".to_string(),
            target_id: dispatch_id.clone(),
            payload: audit_event,
        },
        None,
    )?;
    crate::workbench_sqlite_storage_mode::complete_db_primary_json_projection(
        path,
        "workflow_node_dispatch_prepared",
        || {
            let backup = backup_workflow_state_file(path, &timestamp)?;
            write_validated_workflow_state(path, &value)?;
            dispatch_result_from_state(
                path,
                Some(backup),
                &audit_event_id,
                &dispatch_id,
                "已准备节点派发。",
            )
        },
    )
}

/// Mint only when this invocation inherited an exact, already-prepared M2
/// execution-grant envelope.  The frozen M1 authorization shape has neither
/// quota and remains a no-grant dispatch; it cannot be silently upgraded into
/// a default grant.  A partial envelope is an invalid M2 authorization.
fn mint_execution_grant_for_started_dispatch(
    path: &Path,
    context: &WorkflowNodeDispatchContext,
    dispatch_id: &str,
    timestamp_ms: i64,
) -> Result<
    Option<(
        crate::mcp::execution_grant::ExecutionGrant,
        crate::mcp::execution_grant::ExecutionGrantAuthorizationSource,
    )>,
    String,
> {
    let Some(authorization_id) = context.plan_authorization_id.as_deref() else {
        return Ok(None);
    };
    let source = crate::plan_authorization_store::load_active_execution_grant_source(
        path,
        authorization_id,
        &context.project_id,
        &context.workflow_id,
        timestamp_ms,
    )?;
    // M1 authorization records already had optional quota metadata.  It is
    // not an opt-in to M2: the fresh canonical source must carry the explicit
    // server capability as well, so a caller cannot upgrade a legacy record
    // by changing only a prepared JSON projection.
    if !source
        .allowed_tools
        .iter()
        .any(|capability| {
            capability == crate::mcp::execution_grant::EXECUTION_GRANT_LEDGER_V2_CAPABILITY
        })
    {
        return Ok(None);
    }
    match (source.max_worker_dispatches, source.max_runtime_minutes) {
        (None, None) => return Ok(None),
        (Some(_), Some(_)) => {}
        _ => return Err("execution_grant_quota_envelope_incomplete".to_string()),
    }
    let prepared_dispatch_id = context
        .prepared_dispatch_id
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "execution_grant_prepared_dispatch_reference_missing".to_string())?;
    let authorization_check = context
        .authorization_check
        .as_ref()
        .ok_or_else(|| "execution_grant_authorization_check_missing".to_string())?;
    if authorization_check.status != "authorized"
        || authorization_check.authorization_id.as_deref() != Some(authorization_id)
        || authorization_check.required_user_confirmation
        || authorization_check.required_global_review
    {
        return Err("execution_grant_authorization_check_rejected".to_string());
    }
    let actor_role = role_id_from_node_id(&context.node_id);
    if (!source.allowed_agent_ids.is_empty()
        && !source
            .allowed_agent_ids
            .iter()
            .any(|agent| agent == &context.native_thread_id))
        || !source
            .allowed_role_ids
            .iter()
            .any(|role| role == &actor_role)
    {
        return Err("execution_grant_current_binding_subject_or_role_rejected".to_string());
    }
    let ttl_seconds = u64::try_from(
        source
            .max_runtime_minutes
            .ok_or_else(|| "execution_grant_runtime_quota_missing_or_invalid".to_string())?,
    )
    .map_err(|_| "execution_grant_runtime_quota_invalid".to_string())?
    .checked_mul(60)
    .ok_or_else(|| "execution_grant_runtime_quota_overflow".to_string())?;
    let grant = crate::mcp::execution_grant::mint_dispatch_grant(
        &source,
        &crate::mcp::execution_grant::ExecutionGrantBinding {
            dispatch_id: dispatch_id.to_string(),
            project_id: context.project_id.clone(),
            workflow_id: context.workflow_id.clone(),
            workflow_node_id: context.node_id.clone(),
            work_item_id: context.work_item_id.clone(),
            binding_id: context.binding_id.clone(),
            principal: context.native_thread_id.clone(),
            prepared_dispatch_id: prepared_dispatch_id.to_string(),
        },
        ttl_seconds,
    )?;
    Ok(Some((grant, source)))
}

fn execution_grant_dispatch_value(
    grant: Option<&crate::mcp::execution_grant::ExecutionGrant>,
) -> Result<Value, String> {
    match grant {
        Some(grant) => serde_json::to_value(grant)
            .map_err(|error| format!("execution_grant_serialize_failed:{error}")),
        None => Ok(Value::Null),
    }
}

fn validate_current_execution_grant_binding(
    value: &Value,
    context: &WorkflowNodeDispatchContext,
) -> Result<(), String> {
    if context.plan_authorization_id.is_none() {
        return Ok(());
    }
    let binding_index = workflow_node_session_binding_index(
        value,
        &context.workflow_id,
        &context.node_id,
        Some(&context.work_item_id),
    )
    .ok_or_else(|| "execution_grant_exact_work_item_binding_required".to_string())?;
    let binding = value
        .get("workflow_node_session_bindings")
        .and_then(Value::as_array)
        .and_then(|bindings| bindings.get(binding_index))
        .ok_or_else(|| "execution_grant_exact_work_item_binding_missing".to_string())?;
    if optional_string_from(binding, "binding_id").as_deref() != Some(context.binding_id.as_str())
        || optional_string_from(binding, "native_thread_id").as_deref()
            != Some(context.native_thread_id.as_str())
        || optional_string_from(binding, "lifecycle").as_deref() != Some("active")
    {
        return Err("execution_grant_exact_work_item_binding_stale".to_string());
    }
    Ok(())
}

fn reserve_prepared_dispatch_in_value(
    value: &mut Value,
    context: &WorkflowNodeDispatchContext,
    dispatch_id: &str,
    grant: &crate::mcp::execution_grant::ExecutionGrant,
    source: &crate::mcp::execution_grant::ExecutionGrantAuthorizationSource,
    timestamp_ms: i64,
) -> Result<(Value, String), String> {
    let authorization_id = context
        .plan_authorization_id
        .as_deref()
        .ok_or_else(|| "execution_grant_authorization_reference_missing".to_string())?;
    let prepared_dispatch_id = context
        .prepared_dispatch_id
        .as_deref()
        .ok_or_else(|| "execution_grant_prepared_dispatch_reference_missing".to_string())?;
    let max_dispatches = source
        .max_worker_dispatches
        .filter(|value| *value > 0)
        .ok_or_else(|| "execution_grant_worker_quota_missing_or_invalid".to_string())?;
    let reserved_count = value
        .get("workflow_node_dispatches")
        .and_then(Value::as_array)
        .map(|dispatches| {
            dispatches
                .iter()
                .filter(|candidate| {
                    optional_string_from(candidate, "plan_authorization_id").as_deref()
                        == Some(authorization_id)
                        && candidate
                            .get("execution_grant")
                            .is_some_and(|execution_grant| !execution_grant.is_null())
                })
                .count()
        })
        .unwrap_or_default();
    if i64::try_from(reserved_count).unwrap_or(i64::MAX) >= max_dispatches {
        return Err("execution_grant_worker_quota_exhausted".to_string());
    }
    let dispatches = array_mut(value, "workflow_node_dispatches")?;
    let prepared = dispatches
        .iter_mut()
        .find(|candidate| {
            optional_string_from(candidate, "dispatch_id").as_deref() == Some(prepared_dispatch_id)
        })
        .ok_or_else(|| "execution_grant_prepared_dispatch_not_found".to_string())?;
    if optional_string_from(prepared, "state").as_deref() != Some("prepared")
        || optional_string_from(prepared, "plan_authorization_id").as_deref()
            != Some(authorization_id)
        || optional_string_from(prepared, "workflow_id").as_deref()
            != Some(context.workflow_id.as_str())
        || optional_string_from(prepared, "node_id").as_deref() != Some(context.node_id.as_str())
        || optional_string_from(prepared, "work_item_id").as_deref()
            != Some(context.work_item_id.as_str())
    {
        return Err("execution_grant_prepared_dispatch_not_reservable".to_string());
    }
    let before_hash =
        crate::utils::hash::sha256_hex(&serde_json::to_string(prepared).map_err(|error| {
            format!("execution_grant_prepared_dispatch_serialize_failed:{error}")
        })?);
    prepared["state"] = Value::String("consumed".to_string());
    prepared["consumed_by_dispatch_id"] = Value::String(dispatch_id.to_string());
    prepared["consumed_execution_grant_id"] = Value::String(grant.grant_id.0.clone());
    prepared["consumed_execution_attempt_id"] = Value::String(
        grant
            .attempt_id
            .clone()
            .ok_or_else(|| "execution_grant_attempt_id_missing".to_string())?,
    );
    prepared["consumed_authorization_source_hash"] =
        Value::String(source.authorization_source_hash.clone());
    prepared["consumed_at_ms"] = Value::Number(timestamp_ms.into());
    Ok((prepared.clone(), before_hash))
}

fn execution_grant_attempt_value(
    context: &WorkflowNodeDispatchContext,
    dispatch_id: &str,
    attempt_id: &str,
    timestamp: &str,
) -> Value {
    json!({
      "attempt_id": attempt_id,
      "project_id": context.project_id,
      "workflow_id": context.workflow_id,
      "work_item_id": context.work_item_id,
      "dispatch_id": dispatch_id,
      "attempt_no": 1,
      "state": "running",
      "started_at": timestamp,
      "ended_at": Value::Null,
      "failure_reason": Value::Null,
      "retry_scheduled_at": Value::Null,
      "timed_out_at": Value::Null,
      "cancel_requested_at": Value::Null,
      "warnings": ["server_owned_execution_grant_attempt"]
    })
}

fn write_started_dispatch(
    path: &Path,
    context: &WorkflowNodeDispatchContext,
) -> Result<WorkflowNodeDispatchRecord, String> {
    if let Some(repository) =
        crate::workbench_sqlite_storage_mode::primary_repository_for_m2_t2_fail_closed_write(
            path,
            "workflow_node_dispatch_started",
        )?
    {
        return write_started_dispatch_db_primary(path, context, &repository);
    }
    let timestamp = unix_timestamp_string();
    let timestamp_ms = unix_timestamp_ms();
    let mut value = read_workflow_state_value(path)?;
    ensure_workflow_node_dispatches_array(&mut value)?;
    let work_item_index = find_work_item_index(&value, &context.workflow_id, &context.work_item_id)
        .ok_or_else(|| "当前 workflow 下找不到该 work item；无法执行节点派发".to_string())?;
    let before_state = optional_string_from(&value["work_items"][work_item_index], "state")
        .unwrap_or_else(|| "unknown".to_string());
    control_core::validate_dispatch_start(&before_state)?;
    // Re-read the exact work-item binding from the state we are about to
    // mutate.  The C4 authorization check is only a prepare-time fact; it
    // must not authorize a later rebind through this execution path.
    let dispatch_id = next_workflow_node_dispatch_id(context, &timestamp);
    let execution_grant_material =
        mint_execution_grant_for_started_dispatch(path, context, &dispatch_id, timestamp_ms)?;
    if execution_grant_material.is_some() {
        validate_current_execution_grant_binding(&value, context)?;
    }
    let execution_grant = execution_grant_material.as_ref().map(|(grant, _)| grant);
    let execution_grant_id = execution_grant.map(|grant| grant.grant_id.0.clone());
    let execution_attempt_id = execution_grant.and_then(|grant| grant.attempt_id.clone());
    let execution_grant_value = execution_grant_dispatch_value(execution_grant)?;
    let output_dir = default_workflow_node_dispatch_output_dir();
    let output_nonce = unix_timestamp_nanos();
    let last_message_path = output_dir.join(format!(
        "{}-{}-{}-{}-last-message.txt",
        stable_id(&dispatch_id),
        timestamp_ms,
        std::process::id(),
        output_nonce
    ));
    let dispatch = json!({
      "dispatch_id": dispatch_id,
      "project_id": context.project_id,
      "workflow_id": context.workflow_id,
      "node_id": context.node_id,
      "work_item_id": context.work_item_id,
      "binding_id": context.binding_id,
      "native_thread_id": context.native_thread_id,
      "prompt_preview": context.prompt_preview,
      "worker_prompt": context.prompt_preview,
      "prompt_kind": context.prompt_kind,
      "memory_packet_snapshot_id": context.memory_packet_snapshot_id,
      "memory_packet_fingerprint": context.memory_packet_fingerprint,
      "plan_authorization_id": context.plan_authorization_id,
      "authorization_check": context.authorization_check.as_ref().map(|check| serde_json::to_value(check).unwrap_or(Value::Null)).unwrap_or(Value::Null),
      "execution_grant_id": execution_grant_id,
      "execution_attempt_id": execution_attempt_id,
      "execution_grant": execution_grant_value,
      "user_reviewed_instruction": context.user_reviewed_instruction.as_ref().map(user_reviewed_instruction_value).unwrap_or(Value::Null),
      "state": "running",
      "started_at_ms": timestamp_ms,
      "ended_at_ms": Value::Null,
      "exit_code": Value::Null,
      "last_message_path": last_message_path.display().to_string(),
      "last_message_summary": Value::Null,
      "transcript_event_count": Value::Null,
      "transcript_target_hits": Value::Null,
      "warnings": context.warnings,
      "created_at_ms": timestamp_ms,
      "updated_at_ms": timestamp_ms
    });
    if let Some((grant, source)) = execution_grant_material.as_ref() {
        let (prepared_after, _prepared_before_hash) = reserve_prepared_dispatch_in_value(
            &mut value,
            context,
            &dispatch_id,
            grant,
            source,
            timestamp_ms,
        )?;
        let attempt_id = grant
            .attempt_id
            .as_deref()
            .ok_or_else(|| "execution_grant_attempt_id_missing".to_string())?;
        ensure_array_mut(&mut value, "execution_attempts")?.push(execution_grant_attempt_value(
            context,
            &dispatch_id,
            attempt_id,
            &timestamp,
        ));
        // Keep this value live through the state mutation so JSON-only
        // callers get the same fail-closed prepared transition.  SQLite has
        // an additional transaction-level recheck below.
        let _ = prepared_after;
    }
    array_mut(&mut value, "workflow_node_dispatches")?.push(dispatch);
    {
        let work_items = array_mut(&mut value, "work_items")?;
        let work_item = work_items
            .get_mut(work_item_index)
            .ok_or_else(|| "当前 workflow 下找不到该 work item；无法执行节点派发".to_string())?;
        work_item["state"] = Value::String("running".to_string());
        work_item["current_node_id"] = Value::String(context.node_id.clone());
        work_item["updated_at"] = Value::String(timestamp.clone());
    }
    update_node_state_for_id(&mut value, &context.node_id, "running", &timestamp)?;
    let (actor_ref, permission_level, reason) = dispatch_started_audit_actor_and_reason(
        context.plan_authorization_id.as_deref(),
        &context.prompt_kind,
    );
    array_mut(&mut value, "audit_events")?.push(json!({
      "event_id": crate::workflow_audit::audit_event_identity("workflow-node-dispatch-started", &dispatch_id, &timestamp),
      "event_type": "workflow_node_dispatch_started",
      "target_ref": context.work_item_id,
      "actor_ref": actor_ref,
      "source_kind": "workspace_state_and_codex_resume",
      "permission_level": permission_level,
      "before_state": before_state,
      "after_state": "running",
      "created_at": timestamp,
      "reason": reason
    }));
    value["updated_at"] = Value::String(timestamp);
    write_validated_workflow_state(path, &value)?;
    let updated = read_workflow_state_value(path)?;
    parse_workflow_node_dispatch_record(
        find_workflow_node_dispatch(&updated, &dispatch_id)
            .ok_or_else(|| "写入 running 派发记录后校验失败".to_string())?,
    )
}

fn write_started_dispatch_db_primary(
    path: &Path,
    context: &WorkflowNodeDispatchContext,
    repository: &crate::workbench_sqlite_repository::WorkbenchSqliteRepository,
) -> Result<WorkflowNodeDispatchRecord, String> {
    let timestamp = unix_timestamp_string();
    let timestamp_ms = unix_timestamp_ms();
    let mut value = read_workflow_state_value(path)?;
    ensure_workflow_node_dispatches_array(&mut value)?;
    let work_item_index = find_work_item_index(&value, &context.workflow_id, &context.work_item_id)
        .ok_or_else(|| "当前 workflow 下找不到该 work item；无法执行节点派发".to_string())?;
    let before_state = optional_string_from(&value["work_items"][work_item_index], "state")
        .unwrap_or_else(|| "unknown".to_string());
    control_core::validate_dispatch_start(&before_state)?;
    let dispatch_id = next_workflow_node_dispatch_id(context, &timestamp);
    let execution_grant_material =
        mint_execution_grant_for_started_dispatch(path, context, &dispatch_id, timestamp_ms)?;
    if execution_grant_material.is_some() {
        validate_current_execution_grant_binding(&value, context)?;
    }
    let execution_grant = execution_grant_material.as_ref().map(|(grant, _)| grant);
    let execution_grant_id = execution_grant.map(|grant| grant.grant_id.0.clone());
    let execution_attempt_id = execution_grant.and_then(|grant| grant.attempt_id.clone());
    let execution_grant_value = execution_grant_dispatch_value(execution_grant)?;
    let output_dir = default_workflow_node_dispatch_output_dir();
    let output_nonce = unix_timestamp_nanos();
    let last_message_path = output_dir.join(format!(
        "{}-{}-{}-{}-last-message.txt",
        stable_id(&dispatch_id),
        timestamp_ms,
        std::process::id(),
        output_nonce
    ));
    let dispatch = json!({
      "dispatch_id": dispatch_id,
      "project_id": context.project_id,
      "workflow_id": context.workflow_id,
      "node_id": context.node_id,
      "work_item_id": context.work_item_id,
      "binding_id": context.binding_id,
      "native_thread_id": context.native_thread_id,
      "prompt_preview": context.prompt_preview,
      "worker_prompt": context.prompt_preview,
      "prompt_kind": context.prompt_kind,
      "memory_packet_snapshot_id": context.memory_packet_snapshot_id,
      "memory_packet_fingerprint": context.memory_packet_fingerprint,
      "plan_authorization_id": context.plan_authorization_id,
      "authorization_check": context.authorization_check.as_ref().map(|check| serde_json::to_value(check).unwrap_or(Value::Null)).unwrap_or(Value::Null),
      "execution_grant_id": execution_grant_id,
      "execution_attempt_id": execution_attempt_id,
      "execution_grant": execution_grant_value,
      "user_reviewed_instruction": context.user_reviewed_instruction.as_ref().map(user_reviewed_instruction_value).unwrap_or(Value::Null),
      "state": "running",
      "started_at_ms": timestamp_ms,
      "ended_at_ms": Value::Null,
      "exit_code": Value::Null,
      "last_message_path": last_message_path.display().to_string(),
      "last_message_summary": Value::Null,
      "transcript_event_count": Value::Null,
      "transcript_target_hits": Value::Null,
      "warnings": context.warnings,
      "created_at_ms": timestamp_ms,
      "updated_at_ms": timestamp_ms
    });
    let prepared_grant_reservation = if let Some((grant, source)) =
        execution_grant_material.as_ref()
    {
        let (prepared_after, expected_prepared_hash) = reserve_prepared_dispatch_in_value(
            &mut value,
            context,
            &dispatch_id,
            grant,
            source,
            timestamp_ms,
        )?;
        let attempt_id = grant
            .attempt_id
            .as_deref()
            .ok_or_else(|| "execution_grant_attempt_id_missing".to_string())?;
        let attempt = execution_grant_attempt_value(context, &dispatch_id, attempt_id, &timestamp);
        ensure_array_mut(&mut value, "execution_attempts")?.push(attempt.clone());
        Some((prepared_after, expected_prepared_hash, attempt))
    } else {
        None
    };
    array_mut(&mut value, "workflow_node_dispatches")?.push(dispatch.clone());
    {
        let work_items = array_mut(&mut value, "work_items")?;
        let work_item = work_items
            .get_mut(work_item_index)
            .ok_or_else(|| "当前 workflow 下找不到该 work item；无法执行节点派发".to_string())?;
        work_item["state"] = Value::String("running".to_string());
        work_item["current_node_id"] = Value::String(context.node_id.clone());
        work_item["updated_at"] = Value::String(timestamp.clone());
    }
    update_node_state_for_id(&mut value, &context.node_id, "running", &timestamp)?;
    let audit_event_id = crate::workflow_audit::audit_event_identity(
        "workflow-node-dispatch-started",
        &dispatch_id,
        &timestamp,
    );
    let (actor_ref, permission_level, reason) = dispatch_started_audit_actor_and_reason(
        context.plan_authorization_id.as_deref(),
        &context.prompt_kind,
    );
    let audit_event = json!({
      "event_id": audit_event_id.clone(),
      "event_type": "workflow_node_dispatch_started",
      "target_ref": context.work_item_id,
      "actor_ref": actor_ref,
      "source_kind": "workspace_state_and_codex_resume",
      "permission_level": permission_level,
      "before_state": before_state.clone(),
      "after_state": "running",
      "created_at": timestamp.clone(),
      "reason": reason
    });
    array_mut(&mut value, "audit_events")?.push(audit_event.clone());
    value["updated_at"] = Value::String(timestamp.clone());
    let work_item_after = value
        .get("work_items")
        .and_then(Value::as_array)
        .and_then(|items| items.get(work_item_index))
        .cloned()
        .ok_or_else(|| "DB 主写找不到更新后的 work item".to_string())?;
    let node_after = value
        .get("nodes")
        .and_then(Value::as_array)
        .and_then(|nodes| {
            nodes.iter().find(|node| {
                optional_string_from(node, "node_id").as_deref() == Some(context.node_id.as_str())
            })
        })
        .cloned()
        .ok_or_else(|| "DB 主写找不到更新后的 workflow node".to_string())?;
    let repository_audit = crate::workbench_sqlite_repository::RepositoryAuditEntry {
        event_id: audit_event_id,
        target_kind: "workflow_state".to_string(),
        target_id: dispatch_id.clone(),
        payload: audit_event,
    };
    if let (Some((grant, source)), Some((prepared_after, expected_prepared_hash, attempt))) = (
        execution_grant_material.as_ref(),
        prepared_grant_reservation.as_ref(),
    ) {
        let prepared_dispatch_id = context
            .prepared_dispatch_id
            .as_deref()
            .ok_or_else(|| "execution_grant_prepared_dispatch_reference_missing".to_string())?;
        let authorization_id = context
            .plan_authorization_id
            .as_deref()
            .ok_or_else(|| "execution_grant_authorization_reference_missing".to_string())?;
        repository.reserve_prepared_execution_grant_with_audit(
            &dispatch,
            &work_item_after,
            &node_after,
            &before_state,
            &repository_audit,
            &crate::workbench_sqlite_repository::PreparedExecutionGrantReservation {
                prepared_dispatch_id,
                expected_prepared_hash,
                prepared_after,
                authorization_id,
                authorization_source_hash: &source.authorization_source_hash,
                max_worker_dispatches: source
                    .max_worker_dispatches
                    .ok_or_else(|| "execution_grant_worker_quota_missing_or_invalid".to_string())?,
                binding_id: &context.binding_id,
                native_thread_id: &context.native_thread_id,
                workflow_id: &context.workflow_id,
                node_id: &context.node_id,
                work_item_id: &context.work_item_id,
                execution_attempt: attempt,
            },
            None,
        )?;
        let _ = grant;
    } else {
        repository.reserve_dispatch_with_audit(
            &dispatch,
            &work_item_after,
            &node_after,
            &before_state,
            &repository_audit,
            None,
        )?;
    }
    crate::workbench_sqlite_storage_mode::complete_db_primary_json_projection(
        path,
        "workflow_node_dispatch_started",
        || {
            let _backup = backup_workflow_state_file(path, &timestamp)?;
            write_validated_workflow_state(path, &value)?;
            let updated = read_workflow_state_value(path)?;
            parse_workflow_node_dispatch_record(
                find_workflow_node_dispatch(&updated, &dispatch_id)
                    .ok_or_else(|| "写入 running 派发记录后校验失败".to_string())?,
            )
        },
    )
}

fn write_completed_dispatch(
    path: &Path,
    dispatch_id: &str,
    exit_code: i32,
    stats: CodexDispatchReadbackStats,
) -> Result<WorkflowNodeDispatchResult, String> {
    crate::workbench_sqlite_storage_mode::primary_repository_for_m2_t2_fail_closed_write(
        path,
        "workflow_node_dispatch_completed",
    )?;
    let timestamp = unix_timestamp_string();
    let timestamp_ms = unix_timestamp_ms();
    let mut value = read_workflow_state_value(path)?;
    let backup = backup_workflow_state_file(path, &timestamp)?;
    let dispatch_index = find_workflow_node_dispatch_index(&value, dispatch_id)
        .ok_or_else(|| "找不到 running 节点派发记录；无法完成派发".to_string())?;
    let work_item_id = optional_string_from(
        &value["workflow_node_dispatches"][dispatch_index],
        "work_item_id",
    )
    .ok_or_else(|| "节点派发记录缺 work_item_id".to_string())?;
    let workflow_id = optional_string_from(
        &value["workflow_node_dispatches"][dispatch_index],
        "workflow_id",
    )
    .ok_or_else(|| "节点派发记录缺 workflow_id".to_string())?;
    let dispatch_node_id = optional_string_from(
        &value["workflow_node_dispatches"][dispatch_index],
        "node_id",
    )
    .ok_or_else(|| "节点派发记录缺 node_id".to_string())?;
    let dispatch_state =
        optional_string_from(&value["workflow_node_dispatches"][dispatch_index], "state")
            .unwrap_or_else(|| "unknown".to_string());
    if dispatch_state != "running" {
        return Err(format!(
            "节点派发记录不是 running，控制核心已拒绝完成派发：{dispatch_state}"
        ));
    }
    // Process completion/readback belongs to the execution owner.  A
    // server-minted grant still makes the later worker report an unverified
    // claim, but that claim is recorded by its own command and never performs
    // this execution transition.
    let has_execution_grant = value["workflow_node_dispatches"][dispatch_index]
        .get("execution_grant_id")
        .and_then(Value::as_str)
        .is_some_and(|grant_id| !grant_id.trim().is_empty());
    let next_dispatch_state = "completed";
    let next_work_item_state = "ready_for_review";
    let work_item_index = find_work_item_index(&value, &workflow_id, &work_item_id)
        .ok_or_else(|| "当前 workflow 下找不到该 work item；无法完成节点派发".to_string())?;
    let work_item_state = optional_string_from(&value["work_items"][work_item_index], "state")
        .unwrap_or_else(|| "unknown".to_string());
    control_core::validate_dispatch_completion_transition(&work_item_state, next_work_item_state)?;
    let prompt_kind = optional_string_from(
        &value["workflow_node_dispatches"][dispatch_index],
        "prompt_kind",
    )
    .unwrap_or_else(|| "safe_probe".to_string());
    let last_message_path = optional_string_from(
        &value["workflow_node_dispatches"][dispatch_index],
        "last_message_path",
    );
    let last_message_summary = last_message_path
        .as_deref()
        .and_then(|path| fs::read_to_string(path).ok())
        .map(|text| compact_last_message_summary(&text));

    {
        let dispatches = array_mut(&mut value, "workflow_node_dispatches")?;
        let dispatch = dispatches
            .get_mut(dispatch_index)
            .ok_or_else(|| "找不到节点派发记录；无法完成派发".to_string())?;
        dispatch["state"] = Value::String(next_dispatch_state.to_string());
        dispatch["ended_at_ms"] = Value::Number(timestamp_ms.into());
        dispatch["exit_code"] = Value::Number(exit_code.into());
        dispatch["last_message_summary"] = last_message_summary
            .map(Value::String)
            .unwrap_or(Value::Null);
        dispatch["transcript_event_count"] = Value::Number(stats.transcript_event_count.into());
        dispatch["transcript_target_hits"] = Value::Number(stats.transcript_target_hits.into());
        dispatch["updated_at_ms"] = Value::Number(timestamp_ms.into());
    }
    {
        let work_items = array_mut(&mut value, "work_items")?;
        let work_item = work_items
            .get_mut(work_item_index)
            .ok_or_else(|| "当前 workflow 下找不到该 work item；无法完成节点派发".to_string())?;
        work_item["state"] = Value::String(next_work_item_state.to_string());
        work_item["current_node_id"] = Value::String(workflow_node_for_work_item_state(
            &workflow_id,
            next_work_item_state,
        ));
        work_item["updated_at"] = Value::String(timestamp.clone());
    }
    let review_node_id = workflow_node_for_work_item_state(&workflow_id, next_work_item_state);
    update_node_state_for_id(
        &mut value,
        &review_node_id,
        next_work_item_state,
        &timestamp,
    )?;
    update_node_state_for_id(
        &mut value,
        &dispatch_node_id,
        next_work_item_state,
        &timestamp,
    )?;
    let audit_event_id = crate::workflow_audit::audit_event_identity(
        "workflow-node-dispatch-completed",
        dispatch_id,
        &timestamp,
    );
    array_mut(&mut value, "audit_events")?.push(json!({
      "event_id": audit_event_id,
      "event_type": "workflow_node_dispatch_completed",
      "target_ref": work_item_id,
      "actor_ref": "desktop_shell_dispatcher",
      "source_kind": "workspace_state_and_codex_resume",
      "permission_level": "user_confirmed_write",
      "before_state": "running",
      "after_state": next_work_item_state,
      "created_at": timestamp,
      "reason": "Codex resume 完成；已写执行 owner 的最终回复摘要和 transcript 统计。若有 worker report，它仍须经独立 claim/review command，不能由该事件或 grant 自动成为事实。"
    }));
    array_mut(&mut value, "audit_events")?.push(json!({
      "event_id": crate::workflow_audit::audit_event_identity("workflow-node-dispatch-readback", dispatch_id, &timestamp),
      "event_type": "workflow_node_dispatch_readback_completed",
      "target_ref": work_item_id,
      "actor_ref": "desktop_shell_native_transcript_parser",
      "source_kind": "native_transcript_readback_stats",
      "permission_level": "metadata_read",
      "before_state": "running",
      "after_state": "readback_completed",
      "created_at": timestamp,
      "reason": format!("native transcript parser 只回填统计：events={} hits={}", stats.transcript_event_count, stats.transcript_target_hits)
    }));
    let canonical_attempt_id = value["workflow_node_dispatches"][dispatch_index]
        .get("execution_attempt_id")
        .and_then(Value::as_str)
        .filter(|attempt_id| !attempt_id.trim().is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| format!("attempt:{}:{}", stable_id(dispatch_id), timestamp));
    if prompt_kind == "user_reviewed_instruction" {
        let dispatch_project_id = optional_string_from(
            &value["workflow_node_dispatches"][dispatch_index],
            "project_id",
        )
        .unwrap_or_default();
        let dispatch_instruction = value["workflow_node_dispatches"][dispatch_index]
            .get("user_reviewed_instruction")
            .cloned()
            .unwrap_or(Value::Null);
        let controls = ensure_array_mut(&mut value, "workflow_execution_controls")?;
        controls.push(json!({
          "control_id": format!("control:{}:{}", stable_id(dispatch_id), timestamp),
          "project_id": dispatch_project_id,
          "workflow_id": workflow_id.clone(),
          "work_item_id": work_item_id.clone(),
          "control_state": next_work_item_state,
          "long_task_state": next_dispatch_state,
          "retry_count": 0,
          "max_retries": 0,
          "timeout_seconds": Value::Null,
          "cancel_requested_at": Value::Null,
          "failure_reason": Value::Null,
          "user_reviewed_instruction": dispatch_instruction,
          "audit_event_types": ["workflow_node_dispatch_started", "workflow_node_dispatch_completed"],
          "warnings": []
        }));
        let dispatch_project_id = optional_string_from(
            &value["workflow_node_dispatches"][dispatch_index],
            "project_id",
        )
        .unwrap_or_default();
        let attempts = ensure_array_mut(&mut value, "execution_attempts")?;
        if has_execution_grant {
            let attempt = attempts
                .iter_mut()
                .find(|candidate| {
                    optional_string_from(candidate, "attempt_id").as_deref()
                        == Some(canonical_attempt_id.as_str())
                })
                .ok_or_else(|| "execution_grant_attempt_ledger_missing".to_string())?;
            if optional_string_from(attempt, "state").as_deref() != Some("running") {
                return Err("execution_grant_attempt_ledger_state_mismatch".to_string());
            }
            attempt["state"] = Value::String(next_dispatch_state.to_string());
            attempt["ended_at"] = Value::String(timestamp.clone());
        } else {
            attempts.push(json!({
              "attempt_id": canonical_attempt_id,
              "project_id": dispatch_project_id,
              "workflow_id": workflow_id.clone(),
              "work_item_id": work_item_id.clone(),
              "dispatch_id": dispatch_id,
              "attempt_no": 1,
              "state": next_dispatch_state,
              "started_at": Value::Null,
              "ended_at": timestamp,
              "failure_reason": Value::Null,
              "retry_scheduled_at": Value::Null,
              "timed_out_at": Value::Null,
              "cancel_requested_at": Value::Null,
              "warnings": []
            }));
        }
    }
    value["updated_at"] = Value::String(timestamp);
    write_m5b_batch1_workflow_state(path, "workflow_node_dispatch_completed", &value)?;
    dispatch_result_from_state(
        path,
        Some(backup),
        &audit_event_id,
        dispatch_id,
        "节点派发完成，工作项已进入待回收；worker report 如有仅能由独立 claim command 入账。",
    )
}

fn write_failed_dispatch(
    path: &Path,
    dispatch_id: &str,
    exit_code: i32,
    warnings: Vec<String>,
) -> Result<WorkflowNodeDispatchResult, String> {
    crate::workbench_sqlite_storage_mode::primary_repository_for_m2_t2_fail_closed_write(
        path,
        "workflow_node_dispatch_failed",
    )?;
    let timestamp = unix_timestamp_string();
    let timestamp_ms = unix_timestamp_ms();
    let mut value = read_workflow_state_value(path)?;
    let backup = backup_workflow_state_file(path, &timestamp)?;
    let dispatch_index = find_workflow_node_dispatch_index(&value, dispatch_id)
        .ok_or_else(|| "找不到 running 节点派发记录；无法标记失败".to_string())?;
    let work_item_id = optional_string_from(
        &value["workflow_node_dispatches"][dispatch_index],
        "work_item_id",
    )
    .ok_or_else(|| "节点派发记录缺 work_item_id".to_string())?;
    let workflow_id = optional_string_from(
        &value["workflow_node_dispatches"][dispatch_index],
        "workflow_id",
    )
    .ok_or_else(|| "节点派发记录缺 workflow_id".to_string())?;
    let dispatch_node_id = optional_string_from(
        &value["workflow_node_dispatches"][dispatch_index],
        "node_id",
    )
    .ok_or_else(|| "节点派发记录缺 node_id".to_string())?;
    let dispatch_state =
        optional_string_from(&value["workflow_node_dispatches"][dispatch_index], "state")
            .unwrap_or_else(|| "unknown".to_string());
    if dispatch_state != "running" {
        return Err(format!(
            "节点派发记录不是 running，控制核心已拒绝标记失败：{dispatch_state}"
        ));
    }
    let work_item_index = find_work_item_index(&value, &workflow_id, &work_item_id)
        .ok_or_else(|| "当前 workflow 下找不到该 work item；无法标记失败".to_string())?;
    let prompt_kind = optional_string_from(
        &value["workflow_node_dispatches"][dispatch_index],
        "prompt_kind",
    )
    .unwrap_or_else(|| "safe_probe".to_string());
    let dispatch_project_id = optional_string_from(
        &value["workflow_node_dispatches"][dispatch_index],
        "project_id",
    )
    .unwrap_or_default();
    let dispatch_instruction = value["workflow_node_dispatches"][dispatch_index]
        .get("user_reviewed_instruction")
        .cloned()
        .unwrap_or(Value::Null);
    let attempt_state = if warnings.iter().any(|warning| warning == "timeout") {
        "timed_out"
    } else {
        "failed"
    };
    let work_item_state = optional_string_from(&value["work_items"][work_item_index], "state")
        .unwrap_or_else(|| "unknown".to_string());
    control_core::validate_dispatch_completion_transition(&work_item_state, attempt_state)?;
    let failure_reason = if warnings.is_empty() {
        "codex_resume_failed".to_string()
    } else {
        warnings.join(", ")
    };
    // A grant-owned dispatch already created one canonical running attempt at
    // reservation time.  A failed worker result must close that exact attempt;
    // creating a second `attempt:<dispatch>:timestamp` would leave the
    // grant-owned attempt falsely running and break receipt/attempt identity.
    let canonical_attempt_id = value["workflow_node_dispatches"][dispatch_index]
        .get("execution_attempt_id")
        .and_then(Value::as_str)
        .filter(|attempt_id| !attempt_id.trim().is_empty())
        .map(str::to_string);
    {
        let dispatches = array_mut(&mut value, "workflow_node_dispatches")?;
        let dispatch = dispatches
            .get_mut(dispatch_index)
            .ok_or_else(|| "找不到节点派发记录；无法标记失败".to_string())?;
        dispatch["state"] = Value::String("failed".to_string());
        dispatch["ended_at_ms"] = Value::Number(timestamp_ms.into());
        dispatch["exit_code"] = Value::Number(exit_code.into());
        let mut merged_warnings = string_array(dispatch, "warnings");
        merged_warnings.extend(warnings.clone());
        dispatch["warnings"] = string_vec_value(&merged_warnings);
        dispatch["updated_at_ms"] = Value::Number(timestamp_ms.into());
    }
    {
        let work_items = array_mut(&mut value, "work_items")?;
        let work_item = work_items
            .get_mut(work_item_index)
            .ok_or_else(|| "当前 workflow 下找不到该 work item；无法标记失败".to_string())?;
        work_item["state"] = Value::String(attempt_state.to_string());
        work_item["current_node_id"] = Value::String(dispatch_node_id.clone());
        work_item["updated_at"] = Value::String(timestamp.clone());
    }
    update_node_state_for_id(&mut value, &dispatch_node_id, attempt_state, &timestamp)?;
    if let Some(canonical_attempt_id) = canonical_attempt_id.as_deref() {
        let attempts = ensure_array_mut(&mut value, "execution_attempts")?;
        let attempt = attempts
            .iter_mut()
            .find(|candidate| {
                optional_string_from(candidate, "attempt_id").as_deref()
                    == Some(canonical_attempt_id)
            })
            .ok_or_else(|| "execution_grant_attempt_ledger_missing".to_string())?;
        if optional_string_from(attempt, "dispatch_id").as_deref() != Some(dispatch_id)
            || optional_string_from(attempt, "state").as_deref() != Some("running")
        {
            return Err("execution_grant_attempt_ledger_state_mismatch".to_string());
        }
        attempt["state"] = Value::String(attempt_state.to_string());
        attempt["ended_at"] = Value::String(timestamp.clone());
        attempt["failure_reason"] = Value::String(failure_reason.clone());
        attempt["timed_out_at"] = if attempt_state == "timed_out" {
            json!(timestamp)
        } else {
            Value::Null
        };
        attempt["warnings"] = string_vec_value(&warnings);
    } else if prompt_kind == "user_reviewed_instruction" {
        let instruction_timeout = dispatch_instruction
            .get("timeout_seconds")
            .and_then(Value::as_i64)
            .map(Value::from)
            .unwrap_or(Value::Null);
        let instruction_max_retries = dispatch_instruction
            .get("max_retries")
            .and_then(Value::as_i64)
            .map(Value::from)
            .unwrap_or_else(|| Value::from(0));
        ensure_array_mut(&mut value, "workflow_execution_controls")?.push(json!({
          "control_id": format!("control:{}:{}", stable_id(dispatch_id), timestamp),
          "project_id": dispatch_project_id.clone(),
          "workflow_id": workflow_id.clone(),
          "work_item_id": work_item_id.clone(),
          "control_state": attempt_state,
          "long_task_state": attempt_state,
          "retry_count": 0,
          "max_retries": instruction_max_retries,
          "timeout_seconds": instruction_timeout,
          "cancel_requested_at": Value::Null,
          "failure_reason": failure_reason,
          "user_reviewed_instruction": dispatch_instruction,
          "audit_event_types": ["workflow_node_dispatch_started", "workflow_node_dispatch_failed"],
          "warnings": warnings.clone()
        }));
        ensure_array_mut(&mut value, "execution_attempts")?.push(json!({
          "attempt_id": format!("attempt:{}:{}", stable_id(dispatch_id), timestamp),
          "project_id": dispatch_project_id,
          "workflow_id": workflow_id.clone(),
          "work_item_id": work_item_id.clone(),
          "dispatch_id": dispatch_id,
          "attempt_no": 1,
          "state": attempt_state,
          "started_at": Value::Null,
          "ended_at": timestamp,
          "failure_reason": failure_reason,
          "retry_scheduled_at": Value::Null,
          "timed_out_at": if attempt_state == "timed_out" { json!(timestamp) } else { Value::Null },
          "cancel_requested_at": Value::Null,
          "warnings": warnings.clone()
        }));
    }
    let audit_event_id = crate::workflow_audit::audit_event_identity(
        "workflow-node-dispatch-failed",
        dispatch_id,
        &timestamp,
    );
    array_mut(&mut value, "audit_events")?.push(json!({
      "event_id": audit_event_id,
      "event_type": "workflow_node_dispatch_failed",
      "target_ref": work_item_id,
      "actor_ref": "desktop_shell_dispatcher",
      "source_kind": "workspace_state_and_codex_resume",
      "permission_level": "user_confirmed_write",
      "before_state": "running",
      "after_state": attempt_state,
      "created_at": timestamp,
      "reason": format!("Codex resume 未成功完成，exit_code={exit_code}。")
    }));
    value["updated_at"] = Value::String(timestamp);
    write_m5b_batch1_workflow_state(path, "workflow_node_dispatch_failed", &value)?;
    dispatch_result_from_state(
        path,
        Some(backup),
        &audit_event_id,
        dispatch_id,
        "节点派发失败，已保留审计记录。",
    )
}

fn write_readback_dispatch(
    path: &Path,
    dispatch_id: &str,
    stats: CodexDispatchReadbackStats,
) -> Result<WorkflowNodeDispatchResult, String> {
    crate::workbench_sqlite_storage_mode::primary_repository_for_m2_t2_fail_closed_write(
        path,
        "workflow_node_dispatch_readback",
    )?;
    let timestamp = unix_timestamp_string();
    let timestamp_ms = unix_timestamp_ms();
    let mut value = read_workflow_state_value(path)?;
    let backup = backup_workflow_state_file(path, &timestamp)?;
    let dispatch_index = find_workflow_node_dispatch_index(&value, dispatch_id)
        .ok_or_else(|| "找不到节点派发记录；无法回读结果".to_string())?;
    let work_item_id = optional_string_from(
        &value["workflow_node_dispatches"][dispatch_index],
        "work_item_id",
    )
    .ok_or_else(|| "节点派发记录缺 work_item_id".to_string())?;
    {
        let dispatches = array_mut(&mut value, "workflow_node_dispatches")?;
        let dispatch = dispatches
            .get_mut(dispatch_index)
            .ok_or_else(|| "找不到节点派发记录；无法回读结果".to_string())?;
        dispatch["transcript_event_count"] = Value::Number(stats.transcript_event_count.into());
        dispatch["transcript_target_hits"] = Value::Number(stats.transcript_target_hits.into());
        dispatch["updated_at_ms"] = Value::Number(timestamp_ms.into());
    }
    let audit_event_id = crate::workflow_audit::audit_event_identity(
        "workflow-node-dispatch-readback",
        dispatch_id,
        &timestamp,
    );
    array_mut(&mut value, "audit_events")?.push(json!({
      "event_id": audit_event_id,
      "event_type": "workflow_node_dispatch_readback_completed",
      "target_ref": work_item_id,
      "actor_ref": "desktop_shell_native_transcript_parser",
      "source_kind": "native_transcript_readback_stats",
      "permission_level": "metadata_read",
      "before_state": "dispatch_record_exists",
      "after_state": "readback_completed",
      "created_at": timestamp,
      "reason": format!("native transcript parser 只回填统计：events={} hits={}", stats.transcript_event_count, stats.transcript_target_hits)
    }));
    value["updated_at"] = Value::String(timestamp);
    write_m5b_batch1_workflow_state(path, "workflow_node_dispatch_readback", &value)?;
    dispatch_result_from_state(
        path,
        Some(backup),
        &audit_event_id,
        dispatch_id,
        "已回读节点派发统计。",
    )
}

fn record_workflow_dispatch_director_review_at(
    path: &Path,
    request: &WorkflowDispatchDirectorReviewRequest,
) -> Result<WorkflowStateMutationResult, String> {
    if !path.exists() {
        return Err("工作流状态文件不存在；无法记录总指导回收意见".to_string());
    }
    crate::workbench_sqlite_storage_mode::primary_repository_for_m2_t2_fail_closed_write(
        path,
        "workflow_dispatch_director_review",
    )?;

    let timestamp = unix_timestamp_string();
    let mut value = read_workflow_state_value(path)?;
    ensure_workflow_node_dispatches_array(&mut value)?;
    let validation_warnings = validate_workflow_state(&value);
    if !validation_warnings.is_empty() {
        return Err(format!(
            "当前状态文件未通过 schema 校验：{}",
            validation_warnings.join(", ")
        ));
    }

    let workflow_id = default_workflow_id(&request.project_root);
    if !workflow_exists(&value, &workflow_id) {
        return Err("当前项目还没有本地 workflow；无法记录总指导回收意见".to_string());
    }
    let work_item = find_work_item(&value, &workflow_id, &request.work_item_id)
        .ok_or_else(|| "当前 workflow 下找不到该 work item；无法记录总指导回收意见".to_string())?;
    let work_item_state =
        optional_string_from(work_item, "state").unwrap_or_else(|| "unknown".to_string());
    control_core::validate_director_review_work_item_state(&work_item_state)?;
    let dispatch = find_workflow_node_dispatch(&value, &request.dispatch_id)
        .ok_or_else(|| "找不到该节点派发记录；无法记录总指导回收意见".to_string())?;
    if optional_string_from(dispatch, "workflow_id").as_deref() != Some(workflow_id.as_str())
        || optional_string_from(dispatch, "work_item_id").as_deref()
            != Some(request.work_item_id.as_str())
    {
        return Err("派发记录不属于目标 work item，已拒绝记录总指导回收意见".to_string());
    }
    let decision = normalize_director_review_decision(&request.decision)?;
    let dispatch_state =
        optional_string_from(dispatch, "state").unwrap_or_else(|| "unknown".to_string());
    control_core::validate_director_review(&work_item_state, &dispatch_state, &decision)?;
    let summary = request.summary.trim();
    if summary.is_empty() {
        return Err("总指导回收意见 summary 不能为空".to_string());
    }

    let backup = backup_workflow_state_file(path, &timestamp)?;
    let review_id = format!(
        "review:{}:{}:{}",
        stable_id(&workflow_id),
        stable_id(&request.work_item_id),
        timestamp
    );
    let review = json!({
      "review_id": review_id,
      "project_id": project_id(&request.project_root),
      "workflow_id": workflow_id,
      "work_item_id": request.work_item_id,
      "dispatch_id": request.dispatch_id,
      "reviewer_role": "director",
      "decision": decision,
      "summary": summary,
      "evidence_refs": [
        "/Users/yoyi/workspace/product-line/evidence/2026-05-30-workflow-node-safe-probe-real-confirmed-dispatch-v1.md"
      ],
      "handoff_refs": [
        "/Users/yoyi/workspace/product-line/handoffs/2026-05-30-workflow-node-safe-probe-real-confirmed-dispatch-v1-result.md"
      ],
      "created_at": timestamp,
      "updated_at": timestamp,
      "warnings": string_array(dispatch, "warnings")
    });
    array_mut(&mut value, "reviews")?.push(review);
    let audit_event_id = crate::workflow_audit::audit_event_identity(
        "workflow-dispatch-director-review",
        &request.work_item_id,
        &timestamp,
    );
    array_mut(&mut value, "audit_events")?.push(json!({
      "event_id": audit_event_id,
      "event_type": "workflow_dispatch_director_review_recorded",
      "target_ref": request.work_item_id,
      "actor_ref": "director_confirmed_desktop_shell",
      "source_kind": "workspace_state",
      "permission_level": "user_confirmed_write",
      "before_state": "ready_for_review",
      "after_state": decision,
      "created_at": timestamp,
      "reason": "用户确认记录总指导对已完成派发结果的回收意见；没有发送 Codex 消息。"
    }));
    value["updated_at"] = Value::String(timestamp);
    write_m5b_batch1_workflow_state(path, "workflow_dispatch_director_review", &value)?;
    let snapshot = read_workflow_state_snapshot(path)?;
    Ok(WorkflowStateMutationResult {
        message: "已记录总指导回收意见；没有发送 Codex 消息。".to_string(),
        path: path.display().to_string(),
        backup_path: Some(backup.display().to_string()),
        audit_event_id,
        receipt_id: None,
        first_initialize: false,
        snapshot,
    })
}

fn normalize_director_review_decision(decision: &str) -> Result<String, String> {
    match decision.trim() {
        "accepted" | "needs_changes" | "paused" | "discarded" => Ok(decision.trim().to_string()),
        other => Err(format!("未知总指导回收结论：{other}")),
    }
}

fn record_workflow_permission_decision_at(
    path: &Path,
    request: &WorkflowPermissionDecisionRequest,
) -> Result<WorkflowStateMutationResult, String> {
    if !path.exists() {
        return Err("工作流状态文件不存在；无法记录权限结论".to_string());
    }

    let timestamp = unix_timestamp_string();
    let mut value = read_workflow_state_value(path)?;
    let validation_warnings = validate_workflow_state(&value);
    if !validation_warnings.is_empty() {
        return Err(format!(
            "当前状态文件未通过 schema 校验：{}",
            validation_warnings.join(", ")
        ));
    }

    let workflow_id = default_workflow_id(&request.project_root);
    if !workflow_exists(&value, &workflow_id) {
        return Err("当前项目还没有本地 workflow；无法记录权限结论".to_string());
    }
    if find_work_item(&value, &workflow_id, &request.work_item_id).is_none() {
        return Err("当前 workflow 下找不到该 work item；无法记录权限结论".to_string());
    }
    let permission_index = find_permission_request_index(
        &value,
        &workflow_id,
        &request.work_item_id,
        &request.request_id,
    )
    .ok_or_else(|| "当前 workflow 下找不到该权限请求；无法记录权限结论".to_string())?;
    let before_status =
        optional_string_from(&value["permission_requests"][permission_index], "status")
            .unwrap_or_else(|| "pending".to_string());
    let decision = request.decision.trim();
    control_core::validate_permission_decision(&before_status, decision)?;

    let backup = backup_workflow_state_file(path, &timestamp)?;
    {
        let permission_requests = array_mut(&mut value, "permission_requests")?;
        let permission_request = permission_requests
            .get_mut(permission_index)
            .ok_or_else(|| "当前 workflow 下找不到该权限请求；无法记录权限结论".to_string())?;
        permission_request["status"] = Value::String(decision.to_string());
        permission_request["decision"] = Value::String(decision.to_string());
        permission_request["decided_at"] = Value::String(timestamp.clone());
        let mut warnings = string_array(permission_request, "warnings");
        warnings.push("permission_decision_recorded_by_control_core".to_string());
        permission_request["warnings"] = string_vec_value(&dedupe_strings(warnings));
    }
    let audit_event_id = crate::workflow_audit::audit_event_identity(
        "workflow-permission-decision",
        &request.request_id,
        &timestamp,
    );
    array_mut(&mut value, "audit_events")?.push(
        workflow_audit::workflow_permission_decision_recorded(
            workflow_audit::WorkflowPermissionDecisionRecordedAudit {
                event_id: audit_event_id.clone(),
                request_id: &request.request_id,
                before_state: &before_status,
                after_state: decision,
                created_at: &timestamp,
            },
        ),
    );
    value["updated_at"] = Value::String(timestamp);
    write_m5b_batch2_workflow_state(path, "workflow_permission_decision_recorded", &value)?;
    let snapshot = read_workflow_state_snapshot(path)?;
    Ok(WorkflowStateMutationResult {
        message: format!("已记录权限结论：{}", permission_decision_label(decision)),
        path: path.display().to_string(),
        backup_path: Some(backup.display().to_string()),
        audit_event_id,
        receipt_id: None,
        first_initialize: false,
        snapshot,
    })
}

fn permission_decision_label(decision: &str) -> &str {
    match decision {
        "approved" => "批准",
        "rejected" => "拒绝",
        _ => decision,
    }
}

fn prepare_offline_role_dispatch_at(
    path: &Path,
    request: &OfflineRoleDispatchRequest,
) -> Result<WorkflowNodeDispatchResult, String> {
    if !path.exists() {
        return Err("工作流状态文件不存在；无法记录离线角色派发".to_string());
    }
    validate_offline_role_dispatch_request(request)?;
    let timestamp = unix_timestamp_string();
    let timestamp_ms = unix_timestamp_ms();
    let mut value = read_workflow_state_value(path)?;
    ensure_workflow_node_dispatches_array(&mut value)?;
    let validation_warnings = validate_workflow_state(&value);
    if !validation_warnings.is_empty() {
        return Err(format!(
            "当前状态文件未通过 schema 校验：{}",
            validation_warnings.join(", ")
        ));
    }
    let workflow_id = default_workflow_id(&request.project_root);
    if !workflow_exists(&value, &workflow_id) {
        return Err("当前项目还没有本地 workflow；无法记录离线角色派发".to_string());
    }
    let node_id = offline_role_node_id(&workflow_id, &request.target_role_id)?;
    if !node_exists(&value, &workflow_id, &node_id) {
        return Err("当前 workflow 下找不到目标角色 node；无法记录离线角色派发".to_string());
    }
    let work_item = find_work_item(&value, &workflow_id, &request.work_item_id)
        .ok_or_else(|| "当前 workflow 下找不到该 work item；无法记录离线角色派发".to_string())?;
    let work_item_state =
        optional_string_from(work_item, "state").unwrap_or_else(|| "unknown".to_string());
    control_core::validate_dispatch_prepare(&work_item_state)?;
    if has_pending_offline_role_dispatch(&value, &workflow_id, &request.work_item_id) {
        return Err("同一工作项已有待回传的离线派发，已拒绝重复记录".to_string());
    }
    let authorization_check = plan_authorization_store::inspect_auto_dispatch_authorization(
        path,
        &AutoDispatchGuardInput {
            project_id: project_id(&request.project_root),
            workflow_id: workflow_id.clone(),
            work_item_id: request.work_item_id.clone(),
            task_package_id: Some(format!("offline-role:{}", request.target_role_id)),
            task_package_kind: Some("offline_role_dispatch".to_string()),
            target_role_id: request.target_role_id.clone(),
            target_agent_id: None,
            requested_read_roots: request.allowed_reads.clone(),
            requested_write_roots: request.allowed_writes.clone(),
            requested_tools: vec![],
            requested_checks: vec![],
            triggered_stop_conditions: vec![],
            dispatch_kind: "prepare_offline".to_string(),
        },
        unix_timestamp_ms(),
        &format!(
            "write-offline-dispatch-auth-check-{}",
            unix_timestamp_nanos()
        ),
    )?;
    ensure_authorized_for_prepare(&authorization_check)?;

    let backup = backup_workflow_state_file(path, &timestamp)?;
    let context_id = format!(
        "{}:{}:{}",
        workflow_id, request.work_item_id, request.target_role_id
    );
    let dispatch_id = format!("offline-dispatch:{}:{}", stable_id(&context_id), timestamp);
    let memory_snapshot = find_task_package_artifact(&value, &request.work_item_id, work_item)
        .and_then(task_memory_injection::snapshot_from_artifact);
    let prompt_preview = memory_snapshot
        .as_ref()
        .map(|snapshot| {
            format!(
                "{}\n\n{}",
                request.raw_block,
                task_memory_injection::render_prompt_block(snapshot)
            )
        })
        .unwrap_or_else(|| request.raw_block.clone());
    let mut offline_dispatch_payload = offline_role_dispatch_value(request);
    offline_dispatch_payload["raw_block"] = Value::String(prompt_preview.clone());
    let mut dispatch_warnings = vec!["offline_only_no_codex_resume".to_string()];
    if let Some(snapshot) = &memory_snapshot {
        dispatch_warnings.push("task_memory_packet_snapshot_attached".to_string());
        let current =
            task_memory_injection::current_store_revisions(path, &unix_timestamp_string())?;
        if !task_memory_injection::stale_reasons(snapshot, &current).is_empty() {
            dispatch_warnings.push("task_memory_packet_snapshot_stale".to_string());
        }
    }
    array_mut(&mut value, "workflow_node_dispatches")?.push(json!({
      "dispatch_id": dispatch_id,
      "project_id": project_id(&request.project_root),
      "workflow_id": workflow_id,
      "node_id": node_id,
      "work_item_id": request.work_item_id,
      "binding_id": format!("offline-role-binding:{}", stable_id(&request.target_role_id)),
      "native_thread_id": format!("offline-role:{}", request.target_role_id),
      "prompt_preview": prompt_preview,
      "prompt_kind": "offline_role_dispatch",
      "memory_packet_snapshot_id": memory_snapshot.as_ref().map(|snapshot| snapshot.snapshot_id.clone()),
      "memory_packet_fingerprint": memory_snapshot.as_ref().map(|snapshot| snapshot.fingerprint.clone()),
      "plan_authorization_id": authorization_check.authorization_id.clone(),
      "authorization_check": serde_json::to_value(&authorization_check).unwrap_or(Value::Null),
      "offline_role_dispatch": offline_dispatch_payload,
      "state": "prepared",
      "started_at_ms": Value::Null,
      "ended_at_ms": Value::Null,
      "exit_code": Value::Null,
      "last_message_path": Value::Null,
      "last_message_summary": Value::Null,
      "transcript_event_count": Value::Null,
      "transcript_target_hits": Value::Null,
      "warnings": dispatch_warnings,
      "created_at_ms": timestamp_ms,
      "updated_at_ms": timestamp_ms
    }));
    let audit_event_id = crate::workflow_audit::audit_event_identity(
        "offline-role-dispatch-prepared",
        &dispatch_id,
        &timestamp,
    );
    array_mut(&mut value, "audit_events")?.push(json!({
      "event_id": audit_event_id,
      "event_type": "offline_role_dispatch_prepared",
      "target_ref": request.work_item_id,
      "actor_ref": "director_confirmed_desktop_shell",
      "source_kind": "workspace_state_offline_orchestration",
      "permission_level": "user_confirmed_write",
      "before_state": "ready_to_dispatch",
      "after_state": "prepared",
      "created_at": timestamp,
      "reason": "记录工作台内离线角色派发块；不启动 Codex、不执行 codex exec resume、不写 /Users/yoyi/.codex。"
    }));
    update_node_state_for_id(&mut value, &node_id, "ready_to_dispatch", &timestamp)?;
    value["updated_at"] = Value::String(timestamp);
    write_m5b_batch2_workflow_state(path, "offline_role_dispatch_prepared", &value)?;
    dispatch_result_from_state(
        path,
        Some(backup),
        &audit_event_id,
        &dispatch_id,
        "已记录离线角色派发；未启动 Codex。",
    )
}

fn record_offline_role_result_handoff_at(
    path: &Path,
    request: &OfflineRoleResultHandoffRequest,
) -> Result<WorkflowNodeDispatchResult, String> {
    if !path.exists() {
        return Err("工作流状态文件不存在；无法记录离线角色回传".to_string());
    }
    validate_non_empty("target_role_id", &request.target_role_id)?;
    validate_non_empty("summary", &request.summary)?;
    validate_non_empty("markdown", &request.markdown)?;
    let timestamp = unix_timestamp_string();
    let timestamp_ms = unix_timestamp_ms();
    let mut value = read_workflow_state_value(path)?;
    ensure_workflow_node_dispatches_array(&mut value)?;
    let validation_warnings = validate_workflow_state(&value);
    if !validation_warnings.is_empty() {
        return Err(format!(
            "当前状态文件未通过 schema 校验：{}",
            validation_warnings.join(", ")
        ));
    }
    let workflow_id = default_workflow_id(&request.project_root);
    let dispatch_index = find_workflow_node_dispatch_index(&value, &request.dispatch_id)
        .ok_or_else(|| "找不到离线角色派发记录；无法记录回传".to_string())?;
    let dispatch = &value["workflow_node_dispatches"][dispatch_index];
    if optional_string_from(dispatch, "workflow_id").as_deref() != Some(workflow_id.as_str())
        || optional_string_from(dispatch, "work_item_id").as_deref()
            != Some(request.work_item_id.as_str())
    {
        return Err("离线派发记录不属于目标 work item，已拒绝记录回传".to_string());
    }
    let dispatch_state =
        optional_string_from(dispatch, "state").unwrap_or_else(|| "unknown".to_string());
    control_core::validate_offline_role_handoff(&dispatch_state, "ready_for_review")?;
    let work_item_index = find_work_item_index(&value, &workflow_id, &request.work_item_id)
        .ok_or_else(|| "当前 workflow 下找不到该 work item；无法记录离线回传".to_string())?;
    let node_id = optional_string_from(dispatch, "node_id")
        .ok_or_else(|| "离线派发记录缺 node_id；无法记录回传".to_string())?;
    let backup = backup_workflow_state_file(path, &timestamp)?;
    let artifact_id = format!(
        "artifact:{}:offline-handoff:{}",
        stable_id(&workflow_id),
        timestamp
    );
    {
        let dispatches = array_mut(&mut value, "workflow_node_dispatches")?;
        let dispatch = dispatches
            .get_mut(dispatch_index)
            .ok_or_else(|| "找不到离线派发记录；无法记录回传".to_string())?;
        dispatch["state"] = Value::String("completed".to_string());
        dispatch["ended_at_ms"] = Value::Number(timestamp_ms.into());
        dispatch["exit_code"] = Value::Number(0.into());
        dispatch["last_message_summary"] = Value::String(request.summary.trim().to_string());
        dispatch["transcript_event_count"] = Value::Number(0.into());
        dispatch["transcript_target_hits"] = Value::Number(0.into());
        dispatch["updated_at_ms"] = Value::Number(timestamp_ms.into());
    }
    {
        let work_items = array_mut(&mut value, "work_items")?;
        let work_item = work_items
            .get_mut(work_item_index)
            .ok_or_else(|| "当前 workflow 下找不到该 work item；无法记录离线回传".to_string())?;
        work_item["state"] = Value::String("ready_for_review".to_string());
        work_item["current_node_id"] = Value::String(workflow_node_for_work_item_state(
            &workflow_id,
            "ready_for_review",
        ));
        work_item["updated_at"] = Value::String(timestamp.clone());
    }
    array_mut(&mut value, "artifacts")?.push(json!({
      "artifact_id": artifact_id,
      "artifact_type": "handoff",
      "project_id": project_id(&request.project_root),
      "path": Value::Null,
      "title": format!("离线角色回传：{}", request.target_role_id.trim()),
      "brief": request.summary.trim(),
      "markdown": request.markdown.trim(),
      "source_kind": "workspace_state_offline_orchestration",
      "source_ref": request.work_item_id,
      "dispatch_id": request.dispatch_id,
      "role_id": request.target_role_id.trim(),
      "permission_level": "user_confirmed_write",
      "created_at": timestamp,
      "updated_at": timestamp,
      "warnings": ["offline_handoff_no_codex_resume"]
    }));
    update_node_state_for_id(&mut value, &node_id, "ready_for_review", &timestamp)?;
    update_node_state_for_id(
        &mut value,
        &workflow_node_for_work_item_state(&workflow_id, "ready_for_review"),
        "ready_for_review",
        &timestamp,
    )?;
    let audit_event_id = crate::workflow_audit::audit_event_identity(
        "offline-role-result-handoff",
        &request.dispatch_id,
        &timestamp,
    );
    array_mut(&mut value, "audit_events")?.push(json!({
      "event_id": audit_event_id,
      "event_type": "offline_role_result_handoff_recorded",
      "target_ref": request.work_item_id,
      "actor_ref": "role_stub_desktop_shell",
      "source_kind": "workspace_state_offline_orchestration",
      "permission_level": "user_confirmed_write",
      "before_state": "prepared",
      "after_state": "ready_for_review",
      "created_at": timestamp,
      "reason": "记录离线角色桩结果并回传总指导；没有发送 Codex 消息。"
    }));
    value["updated_at"] = Value::String(timestamp);
    write_m5b_batch2_workflow_state(path, "offline_role_result_handoff_recorded", &value)?;
    dispatch_result_from_state(
        path,
        Some(backup),
        &audit_event_id,
        &request.dispatch_id,
        "已记录离线角色回传，工作项进入待回收。",
    )
}

fn record_offline_director_review_at(
    path: &Path,
    request: &OfflineDirectorReviewRequest,
) -> Result<WorkflowStateMutationResult, String> {
    if !path.exists() {
        return Err("工作流状态文件不存在；无法记录离线总指导回收".to_string());
    }
    let decision = normalize_director_review_decision(&request.decision)?;
    validate_non_empty("summary", &request.summary)?;
    let timestamp = unix_timestamp_string();
    let mut value = read_workflow_state_value(path)?;
    ensure_workflow_node_dispatches_array(&mut value)?;
    let validation_warnings = validate_workflow_state(&value);
    if !validation_warnings.is_empty() {
        return Err(format!(
            "当前状态文件未通过 schema 校验：{}",
            validation_warnings.join(", ")
        ));
    }
    let workflow_id = default_workflow_id(&request.project_root);
    let work_item_index = find_work_item_index(&value, &workflow_id, &request.work_item_id)
        .ok_or_else(|| "当前 workflow 下找不到该 work item；无法记录离线总指导回收".to_string())?;
    let work_item_state = optional_string_from(&value["work_items"][work_item_index], "state")
        .unwrap_or_else(|| "unknown".to_string());
    control_core::validate_director_review_work_item_state(&work_item_state)?;
    let dispatch = find_workflow_node_dispatch(&value, &request.dispatch_id)
        .ok_or_else(|| "找不到离线派发记录；无法记录离线总指导回收".to_string())?;
    if optional_string_from(dispatch, "workflow_id").as_deref() != Some(workflow_id.as_str())
        || optional_string_from(dispatch, "work_item_id").as_deref()
            != Some(request.work_item_id.as_str())
    {
        return Err("离线派发记录不属于目标 work item，已拒绝记录总指导回收".to_string());
    }
    let dispatch_state =
        optional_string_from(dispatch, "state").unwrap_or_else(|| "unknown".to_string());
    control_core::validate_director_review(&work_item_state, &dispatch_state, &decision)?;
    let backup = backup_workflow_state_file(path, &timestamp)?;
    let review_id = format!(
        "review:{}:{}:offline:{}",
        stable_id(&workflow_id),
        stable_id(&request.work_item_id),
        timestamp
    );
    let handoff_refs = offline_handoff_refs_for_dispatch(&value, &request.dispatch_id);
    array_mut(&mut value, "reviews")?.push(json!({
      "review_id": review_id,
      "project_id": project_id(&request.project_root),
      "workflow_id": workflow_id,
      "work_item_id": request.work_item_id,
      "dispatch_id": request.dispatch_id,
      "reviewer_role": "director",
      "decision": decision,
      "summary": request.summary.trim(),
      "evidence_refs": [],
      "handoff_refs": handoff_refs,
      "created_at": timestamp,
      "updated_at": timestamp,
      "warnings": ["offline_review_no_codex_resume"]
    }));
    {
        let work_items = array_mut(&mut value, "work_items")?;
        let work_item = work_items.get_mut(work_item_index).ok_or_else(|| {
            "当前 workflow 下找不到该 work item；无法记录离线总指导回收".to_string()
        })?;
        work_item["state"] = Value::String(decision.clone());
        work_item["current_node_id"] = Value::String(workflow_node_for_work_item_state(
            &default_workflow_id(&request.project_root),
            &decision,
        ));
        work_item["updated_at"] = Value::String(timestamp.clone());
    }
    let audit_event_id = crate::workflow_audit::audit_event_identity(
        "offline-director-review",
        &request.work_item_id,
        &timestamp,
    );
    array_mut(&mut value, "audit_events")?.push(json!({
      "event_id": audit_event_id,
      "event_type": "offline_director_review_recorded",
      "target_ref": request.work_item_id,
      "actor_ref": "director_confirmed_desktop_shell",
      "source_kind": "workspace_state_offline_orchestration",
      "permission_level": "user_confirmed_write",
      "before_state": "ready_for_review",
      "after_state": decision,
      "created_at": timestamp,
      "reason": "记录离线总指导回收并推进工作项状态；没有发送 Codex 消息。"
    }));
    value["updated_at"] = Value::String(timestamp);
    write_m5b_batch2_workflow_state(path, "offline_director_review_recorded", &value)?;
    let snapshot = read_workflow_state_snapshot(path)?;
    Ok(WorkflowStateMutationResult {
        message: "已记录离线总指导回收，工作项状态已推进。".to_string(),
        path: path.display().to_string(),
        backup_path: Some(backup.display().to_string()),
        audit_event_id,
        receipt_id: None,
        first_initialize: false,
        snapshot,
    })
}

fn validate_offline_role_dispatch_request(
    request: &OfflineRoleDispatchRequest,
) -> Result<(), String> {
    for (label, value) in [
        ("work_item_id", request.work_item_id.as_str()),
        ("target_role_id", request.target_role_id.as_str()),
        ("target_role_label", request.target_role_label.as_str()),
        ("task_title", request.task_title.as_str()),
        ("objective", request.objective.as_str()),
        ("execution_cwd", request.execution_cwd.as_str()),
        ("raw_block", request.raw_block.as_str()),
    ] {
        validate_non_empty(label, value)?;
    }
    if request.allowed_reads.is_empty() {
        return Err("离线派发块缺少 allowed_reads".to_string());
    }
    if request.forbidden_actions.is_empty() {
        return Err("离线派发块缺少 forbidden_actions".to_string());
    }
    if request.acceptance_criteria.is_empty() {
        return Err("离线派发块缺少 acceptance_criteria".to_string());
    }
    if request.required_return.is_empty() {
        return Err("离线派发块缺少 required_return".to_string());
    }
    if request.timeout_seconds <= 0 {
        return Err("离线派发块 timeout_seconds 必须大于 0".to_string());
    }
    offline_role_node_suffix(&request.target_role_id)?;
    Ok(())
}

fn validate_non_empty(label: &str, value: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        Err(format!("{label} 不能为空"))
    } else {
        Ok(())
    }
}

fn offline_role_dispatch_value(request: &OfflineRoleDispatchRequest) -> Value {
    json!({
      "project_root": request.project_root,
      "work_item_id": request.work_item_id,
      "target_role_id": request.target_role_id,
      "target_role_label": request.target_role_label,
      "task_title": request.task_title,
      "objective": request.objective,
      "execution_cwd": request.execution_cwd,
      "allowed_reads": request.allowed_reads,
      "allowed_writes": request.allowed_writes,
      "forbidden_actions": request.forbidden_actions,
      "acceptance_criteria": request.acceptance_criteria,
      "timeout_seconds": request.timeout_seconds,
      "required_return": request.required_return,
      "raw_block": request.raw_block
    })
}

fn offline_role_node_id(workflow_id: &str, role_id: &str) -> Result<String, String> {
    Ok(format!(
        "{workflow_id}:node:{}",
        offline_role_node_suffix(role_id)?
    ))
}

fn offline_role_node_suffix(role_id: &str) -> Result<&'static str, String> {
    match role_id.trim() {
        "director" => Ok("director"),
        "codex-dev" | "developer" => Ok("codex-dev"),
        "validation" | "verifier" => Ok("validation"),
        "review" | "reviewer" => Ok("review"),
        other => Err(format!("未知离线角色：{other}")),
    }
}

fn offline_handoff_refs_for_dispatch(value: &Value, dispatch_id: &str) -> Vec<String> {
    value
        .get("artifacts")
        .and_then(Value::as_array)
        .map(|artifacts| {
            artifacts
                .iter()
                .filter(|artifact| {
                    optional_string_from(artifact, "artifact_type").as_deref() == Some("handoff")
                        && optional_string_from(artifact, "dispatch_id").as_deref()
                            == Some(dispatch_id)
                })
                .filter_map(|artifact| optional_string_from(artifact, "artifact_id"))
                .collect()
        })
        .unwrap_or_default()
}

fn has_pending_offline_role_dispatch(value: &Value, workflow_id: &str, work_item_id: &str) -> bool {
    value
        .get("workflow_node_dispatches")
        .and_then(Value::as_array)
        .is_some_and(|dispatches| {
            dispatches.iter().any(|dispatch| {
                optional_string_from(dispatch, "workflow_id").as_deref() == Some(workflow_id)
                    && optional_string_from(dispatch, "work_item_id").as_deref()
                        == Some(work_item_id)
                    && optional_string_from(dispatch, "prompt_kind").as_deref()
                        == Some("offline_role_dispatch")
                    && optional_string_from(dispatch, "state").as_deref() == Some("prepared")
            })
        })
}

fn run_workflow_machine_at(
    path: &Path,
    index: &Value,
    readback_db_path: &Path,
    runner: &dyn CodexResumeRunner,
    request: &WorkflowMachineRunRequest,
) -> Result<WorkflowMachineRunResult, String> {
    // 四角色机器真实现已被 H5 受控会话续（controlled_session_continuation）取代并封
    // （boundary spec deprecated:true·CURRENT ④a）。真实现分支及其独占 helper 已删；此处只留 blocked
    // 响应——与命令面 run_workflow_machine / CLI __run_workflow_machine_real 同款封禁消息（逐字节一致）。
    let _ = (path, index, readback_db_path, runner, request);
    Err(legacy_product_command_blocked_message(
        "run_workflow_machine",
    ))
}

fn workflow_machine_final_acceptance(summary: &str) -> bool {
    summary.contains("WORKFLOW_MACHINE_FINAL_ACCEPTED")
        || summary.contains("最终目标完成")
        || summary.contains("最终结论：通过")
        || summary.contains("我的判断：通过")
        || summary.contains("判断：通过")
        || summary.contains("结论：通过")
}
