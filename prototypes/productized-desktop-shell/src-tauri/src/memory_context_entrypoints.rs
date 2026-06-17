// Memory command bridge, observation bridge, task memory preview, and context guard helpers split out during Root Treatment R2-B7.
// This file is included at crate root so helper visibility and behavior stay unchanged.
#[cfg(test)]
#[path = "ru_dogfood.rs"]
mod ru_dogfood;

fn create_formal_memory_record_at(
    path: &Path,
    request: &CreateFormalMemoryRecordInput,
    timestamp: &str,
    write_id: &str,
) -> Result<CreateFormalMemoryRecordOutput, String> {
    validate_formal_memory_context_binding(path, request)?;
    formal_memory_store::create_record(path, request, timestamp, write_id)
}

pub(crate) fn adopt_memory_candidate_to_formal_memory_at(
    path: &Path,
    request: &AdoptMemoryCandidateInput,
    timestamp: &str,
    candidate_write_id: &str,
    formal_write_id: &str,
) -> Result<AdoptMemoryCandidateOutput, String> {
    let lint_input = MemoryLintRunInput {
        project_root: request.project_root.clone(),
        project_id: Some(project_id(&request.project_root)),
        workflow_id: Some(default_workflow_id(&request.project_root)),
        actor_id: request.actor_id.clone(),
        actor_role: request.actor_role.clone(),
        lint_intent: MemoryLintRunIntent::CandidateAdoptionGuard,
        candidate_key: Some(request.candidate_key.clone()),
        task_id: None,
        revoked_source_ids: vec![],
        expected_formal_store_revision: request.expected_formal_store_revision,
        expected_candidate_store_revision: request.expected_candidate_store_revision,
        expected_lint_store_revision: None,
        dry_run: Some(false),
    };
    let lint_output = run_memory_lint_at(
        path,
        &lint_input,
        timestamp,
        &format!("{candidate_write_id}-lint"),
    )?;
    if lint_output.blocking_count > 0 {
        return Err(format!(
            "memory_lint_blocking_findings: candidate adoption blocked by {} finding(s): {}",
            lint_output.blocking_count,
            lint_output
                .new_findings
                .iter()
                .map(|finding| finding.finding_id.clone())
                .collect::<Vec<_>>()
                .join(",")
        ));
    }
    memory_candidate_store::adopt_candidate_to_formal_memory(
        path,
        request,
        timestamp,
        candidate_write_id,
        |candidate| {
            let formal_input = CreateFormalMemoryRecordInput {
                project_root: request.project_root.clone(),
                project_id: Some(project_id(&request.project_root)),
                workflow_id: Some(default_workflow_id(&request.project_root)),
                scope: candidate.scope.clone(),
                memory_type: candidate.memory_type.clone(),
                claim: candidate.claim.clone(),
                body: candidate.body.clone(),
                source_refs: candidate.source_refs.clone(),
                actor_id: request.actor_id.clone(),
                actor_role: request.actor_role.clone(),
                reason: format!(
                    "{}；candidate_key={}",
                    request.adoption_reason.trim(),
                    candidate.candidate_key
                ),
                audit_event_type: Some("memory_candidate_adopted_to_formal_memory".to_string()),
                expected_store_revision: request.expected_formal_store_revision,
            };
            create_formal_memory_record_at(path, &formal_input, timestamp, formal_write_id)
        },
    )
}

fn run_memory_lint_at(
    path: &Path,
    request: &MemoryLintRunInput,
    timestamp: &str,
    write_id: &str,
) -> Result<MemoryLintRunOutput, String> {
    validate_memory_lint_context_binding(path, request)?;
    memory_lint_store::run_lint(path, request, timestamp, write_id)
}

fn validate_memory_lint_context_binding(
    path: &Path,
    request: &MemoryLintRunInput,
) -> Result<(), String> {
    let project_root = request.project_root.trim();
    if project_root.is_empty() {
        return Err("memory lint 缺少 project_root".to_string());
    }
    let expected_project_id = project_id(project_root);
    let expected_workflow_id = default_workflow_id(project_root);
    validate_task_memory_packet_context_field(
        "project_id",
        request.project_id.as_deref(),
        &expected_project_id,
    )?;
    validate_task_memory_packet_context_field(
        "workflow_id",
        request.workflow_id.as_deref(),
        &expected_workflow_id,
    )?;
    validate_task_memory_packet_project_registered(path, project_root)?;
    Ok(())
}

fn create_observation_at(
    path: &Path,
    request: &CreateObservationInput,
    timestamp: &str,
    write_id: &str,
) -> Result<CreateObservationOutput, String> {
    validate_observation_context_binding(path, request)?;
    observation_store::create_observation(path, request, timestamp, write_id)
}

fn create_memory_candidate_from_observation_at(
    path: &Path,
    request: &CreateMemoryCandidateFromObservationInput,
    timestamp: &str,
    observation_write_id: &str,
    candidate_write_id: &str,
) -> Result<CreateMemoryCandidateFromObservationOutput, String> {
    validate_observation_project_registered(path, request.project_root.trim())?;
    observation_store::create_memory_candidate_from_observation(
        path,
        request,
        timestamp,
        observation_write_id,
        |candidate_input| {
            memory_candidate_store::create_candidate(
                path,
                candidate_input,
                timestamp,
                candidate_write_id,
            )
        },
    )
}

fn preview_task_memory_packet_at(
    path: &Path,
    request: &TaskMemoryPacketBuildInput,
    timestamp: &str,
) -> Result<TaskMemoryPacketBuildOutput, String> {
    validate_task_memory_packet_context_binding(path, request)?;
    task_memory_packet_builder::build_preview(path, request, timestamp)
}

fn validate_task_memory_packet_context_binding(
    path: &Path,
    request: &TaskMemoryPacketBuildInput,
) -> Result<(), String> {
    let project_root = request.project_root.trim();
    if project_root.is_empty() {
        return Err("任务记忆包预览缺少 project_root".to_string());
    }
    let expected_project_id = project_id(project_root);
    let expected_workflow_id = default_workflow_id(project_root);
    validate_task_memory_packet_context_field(
        "project_id",
        request.project_id.as_deref(),
        &expected_project_id,
    )?;
    validate_task_memory_packet_context_field(
        "workflow_id",
        request.workflow_id.as_deref(),
        &expected_workflow_id,
    )?;
    validate_task_memory_packet_project_registered(path, project_root)?;
    Ok(())
}

fn validate_task_memory_packet_context_field(
    field_name: &str,
    actual: Option<&str>,
    expected: &str,
) -> Result<(), String> {
    if let Some(actual) = actual {
        let actual = actual.trim();
        if actual != expected {
            return Err(format!(
                "任务记忆包上下文绑定失败：{field_name} 与 project_root 不匹配，expected {expected}，actual {actual}"
            ));
        }
    }
    Ok(())
}

fn validate_task_memory_packet_project_registered(
    path: &Path,
    project_root: &str,
) -> Result<(), String> {
    if !path.exists() {
        return Err("任务记忆包上下文绑定失败：workflow state 不存在，已拒绝生成预览".to_string());
    }
    let value = read_workflow_state_value(path)?;
    let validation_warnings = validate_workflow_state(&value);
    if !validation_warnings.is_empty() {
        return Err(format!(
            "任务记忆包上下文绑定失败：workflow state 未通过 schema 校验：{}",
            validation_warnings.join(", ")
        ));
    }
    let projects = value
        .get("projects")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            "任务记忆包上下文绑定失败：workflow state 缺少 projects[]，已拒绝生成预览".to_string()
        })?;
    let registered = projects.iter().any(|project| {
        optional_string_from(project, "root_path").as_deref() == Some(project_root)
            || optional_string_from(project, "project_root").as_deref() == Some(project_root)
    });
    if !registered {
        return Err(format!(
            "任务记忆包上下文绑定失败：workflow state projects[] 不包含 project_root：{project_root}"
        ));
    }
    Ok(())
}

fn validate_observation_context_binding(
    path: &Path,
    request: &CreateObservationInput,
) -> Result<(), String> {
    let project_root = request.project_root.trim();
    if project_root.is_empty() {
        return Err("observation 创建缺少 project_root".to_string());
    }
    let expected_project_id = project_id(project_root);
    let expected_workflow_id = default_workflow_id(project_root);

    validate_observation_context_field(
        "project_id",
        request.project_id.as_deref(),
        &expected_project_id,
    )?;
    validate_observation_context_field(
        "workflow_id",
        request.workflow_id.as_deref(),
        &expected_workflow_id,
    )?;
    validate_observation_context_field(
        "scope.project_id",
        request.scope.project_id.as_deref(),
        &expected_project_id,
    )?;
    validate_observation_context_field(
        "scope.workflow_id",
        request.scope.workflow_id.as_deref(),
        &expected_workflow_id,
    )?;
    for source in &request.source_refs {
        validate_observation_context_field(
            "source_refs.project_id",
            source.project_id.as_deref(),
            &expected_project_id,
        )?;
        validate_observation_context_field(
            "source_refs.workflow_id",
            source.workflow_id.as_deref(),
            &expected_workflow_id,
        )?;
    }

    let scope_type = request.scope.scope_type.as_str();
    if matches!(scope_type, "project" | "workflow" | "session")
        && option_trimmed_is_empty(request.scope.project_id.as_deref())
    {
        return Err(
            "observation 上下文绑定失败：project/workflow/session scope 必须带 scope.project_id"
                .to_string(),
        );
    }
    if matches!(scope_type, "workflow" | "session")
        && option_trimmed_is_empty(request.scope.workflow_id.as_deref())
    {
        return Err(
            "observation 上下文绑定失败：workflow/session scope 必须带 scope.workflow_id"
                .to_string(),
        );
    }

    validate_observation_project_registered(path, project_root)?;
    Ok(())
}

fn validate_observation_context_field(
    field_name: &str,
    actual: Option<&str>,
    expected: &str,
) -> Result<(), String> {
    if let Some(actual) = actual {
        let actual = actual.trim();
        if actual != expected {
            return Err(format!(
                "observation 上下文绑定失败：{field_name} 与 project_root 不匹配，expected {expected}，actual {actual}"
            ));
        }
    }
    Ok(())
}

fn validate_observation_project_registered(path: &Path, project_root: &str) -> Result<(), String> {
    if project_root.trim().is_empty() {
        return Err("observation 缺少 project_root".to_string());
    }
    if !path.exists() {
        return Err(
            "observation 上下文绑定失败：workflow state 不存在，已拒绝写入 observation".to_string(),
        );
    }
    let value = read_workflow_state_value(path)?;
    let validation_warnings = validate_workflow_state(&value);
    if !validation_warnings.is_empty() {
        return Err(format!(
            "observation 上下文绑定失败：workflow state 未通过 schema 校验：{}",
            validation_warnings.join(", ")
        ));
    }
    let projects = value
        .get("projects")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            "observation 上下文绑定失败：workflow state 缺少 projects[]，已拒绝写入 observation"
                .to_string()
        })?;
    if projects.is_empty() {
        return Err(
            "observation 上下文绑定失败：workflow state projects[] 为空，已拒绝写入 observation"
                .to_string(),
        );
    }
    let registered = projects.iter().any(|project| {
        optional_string_from(project, "root_path").as_deref() == Some(project_root)
            || optional_string_from(project, "project_root").as_deref() == Some(project_root)
    });
    if !registered {
        return Err(format!(
            "observation 上下文绑定失败：workflow state projects[] 不包含 project_root：{project_root}"
        ));
    }
    Ok(())
}

fn validate_formal_memory_context_binding(
    path: &Path,
    request: &CreateFormalMemoryRecordInput,
) -> Result<(), String> {
    let project_root = request.project_root.trim();
    if project_root.is_empty() {
        return Err("正式记忆创建缺少 project_root".to_string());
    }
    let expected_project_id = project_id(project_root);
    let expected_workflow_id = default_workflow_id(project_root);

    validate_formal_memory_context_field(
        "project_id",
        request.project_id.as_deref(),
        &expected_project_id,
    )?;
    validate_formal_memory_context_field(
        "workflow_id",
        request.workflow_id.as_deref(),
        &expected_workflow_id,
    )?;
    validate_formal_memory_context_field(
        "scope.project_id",
        request.scope.project_id.as_deref(),
        &expected_project_id,
    )?;
    validate_formal_memory_context_field(
        "scope.workflow_id",
        request.scope.workflow_id.as_deref(),
        &expected_workflow_id,
    )?;

    let scope_type = request.scope.scope_type.as_str();
    if matches!(scope_type, "project" | "workflow" | "session")
        && option_trimmed_is_empty(request.scope.project_id.as_deref())
    {
        return Err(
            "正式记忆上下文绑定失败：project/workflow/session scope 必须带 scope.project_id"
                .to_string(),
        );
    }
    if matches!(scope_type, "workflow" | "session")
        && option_trimmed_is_empty(request.scope.workflow_id.as_deref())
    {
        return Err(
            "正式记忆上下文绑定失败：workflow/session scope 必须带 scope.workflow_id".to_string(),
        );
    }

    if request.actor_role == "project_director"
        && !matches!(scope_type, "project" | "workflow" | "session")
    {
        return Err(
            "project_director 只能创建本项目 / workflow / session 作用域正式记忆".to_string(),
        );
    }

    validate_formal_memory_project_registered(path, project_root)?;
    Ok(())
}

fn validate_formal_memory_context_field(
    field_name: &str,
    actual: Option<&str>,
    expected: &str,
) -> Result<(), String> {
    if let Some(actual) = actual {
        let actual = actual.trim();
        if actual != expected {
            return Err(format!(
                "正式记忆上下文绑定失败：{field_name} 与 project_root 不匹配，expected {expected}，actual {actual}"
            ));
        }
    }
    Ok(())
}

fn validate_formal_memory_project_registered(
    path: &Path,
    project_root: &str,
) -> Result<(), String> {
    if !path.exists() {
        return Err(
            "正式记忆上下文绑定失败：workflow state 不存在，已拒绝创建正式记忆".to_string(),
        );
    }
    let value = read_workflow_state_value(path)?;
    let validation_warnings = validate_workflow_state(&value);
    if !validation_warnings.is_empty() {
        return Err(format!(
            "正式记忆上下文绑定失败：workflow state 未通过 schema 校验：{}",
            validation_warnings.join(", ")
        ));
    }
    let projects = value
        .get("projects")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            "正式记忆上下文绑定失败：workflow state 缺少 projects[]，已拒绝创建正式记忆".to_string()
        })?;
    if projects.is_empty() {
        return Err(
            "正式记忆上下文绑定失败：workflow state projects[] 为空，已拒绝创建正式记忆"
                .to_string(),
        );
    }
    let registered = projects.iter().any(|project| {
        optional_string_from(project, "root_path").as_deref() == Some(project_root)
            || optional_string_from(project, "project_root").as_deref() == Some(project_root)
    });
    if !registered {
        return Err(format!(
            "正式记忆上下文绑定失败：workflow state projects[] 不包含 project_root：{project_root}"
        ));
    }
    Ok(())
}
