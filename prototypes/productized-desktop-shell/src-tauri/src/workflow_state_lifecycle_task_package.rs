// Workflow state lifecycle and task package helpers split out during Root Treatment R2-B3.
// This file is included at crate root so helper visibility and behavior stay unchanged.

fn read_workflow_state_snapshot(path: &Path) -> Result<WorkflowStateSnapshot, String> {
    if !path.exists() {
        return Ok(empty_workflow_state_snapshot(
            path,
            vec!["状态文件不存在；不会自动创建。".to_string()],
        ));
    }

    let text = fs::read_to_string(path)
        .map_err(|error| format!("无法读取工作流状态文件 {}：{error}", path.display()))?;
    let value: Value = serde_json::from_str(&text)
        .map_err(|error| format!("工作流状态 JSON 解析失败 {}：{error}", path.display()))?;
    let mut warnings = validate_workflow_state(&value);

    let project_workflows = project_workflow_summaries(&value);
    let project_blackboards = project_blackboards_from_workflows(&project_workflows);

    Ok(WorkflowStateSnapshot {
        exists: true,
        path: path.display().to_string(),
        schema_version: optional_string_from(&value, "schema_version"),
        workflow_version: i64_value(&value, "workflow_version"),
        workspace_id: optional_string_from(&value, "workspace_id"),
        updated_at: optional_string_from(&value, "updated_at"),
        initialized: warnings.is_empty(),
        counts: workflow_state_counts(&value),
        project_workflows,
        project_blackboards,
        warnings: {
            if warnings.is_empty() {
                warnings.push("状态文件已读取；只展示元数据，不展示正文。".to_string());
            }
            warnings
        },
    })
}

fn initialize_workflow_state_at(path: &Path) -> Result<WorkflowStateMutationResult, String> {
    let existed = path.exists();
    let timestamp = unix_timestamp_string();
    let audit_event_id = format!("audit:init:{timestamp}");
    let parent = path
        .parent()
        .ok_or_else(|| format!("状态文件路径没有父目录：{}", path.display()))?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("创建状态目录失败 {}：{error}", parent.display()))?;

    let backup_path = if existed {
        let backups_dir = parent.join("backups");
        fs::create_dir_all(&backups_dir)
            .map_err(|error| format!("创建备份目录失败 {}：{error}", backups_dir.display()))?;
        let backup = backups_dir.join(format!("workflow-state.v0.{timestamp}.json"));
        fs::copy(path, &backup)
            .map_err(|error| format!("备份旧状态文件失败 {}：{error}", backup.display()))?;
        Some(backup)
    } else {
        None
    };

    let value = initial_workflow_state_json(&timestamp, &audit_event_id, existed, path);
    let validation_warnings = validate_workflow_state(&value);
    if !validation_warnings.is_empty() {
        return Err(format!(
            "初始化状态未通过 schema 校验：{}",
            validation_warnings.join(", ")
        ));
    }

    atomic_write_json(path, &value)?;
    let snapshot = read_workflow_state_snapshot(path)?;
    if !snapshot.exists
        || snapshot.schema_version.as_deref() != Some("workflow_state_v0")
        || snapshot.workflow_version != Some(1)
    {
        return Err("初始化后重新读取校验失败".to_string());
    }

    Ok(WorkflowStateMutationResult {
        message: if existed {
            "已在用户确认后初始化工作流事实层；旧状态文件已先备份。".to_string()
        } else {
            "已在用户确认后首次初始化工作流事实层；此前无旧状态文件可备份。".to_string()
        },
        path: path.display().to_string(),
        backup_path: backup_path.map(|backup| backup.display().to_string()),
        audit_event_id,
        first_initialize: !existed,
        snapshot,
    })
}

fn bootstrap_project_workflow_at(
    path: &Path,
    project: &ProjectRecord,
) -> Result<WorkflowStateMutationResult, String> {
    let existed = path.exists();
    let timestamp = unix_timestamp_string();
    let audit_event_id = format!(
        "audit:bootstrap:{}:{timestamp}",
        stable_id(&project.project_root)
    );
    let parent = path
        .parent()
        .ok_or_else(|| format!("状态文件路径没有父目录：{}", path.display()))?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("创建状态目录失败 {}：{error}", parent.display()))?;

    let backup_path = if existed {
        let backups_dir = parent.join("backups");
        fs::create_dir_all(&backups_dir)
            .map_err(|error| format!("创建备份目录失败 {}：{error}", backups_dir.display()))?;
        let backup = backups_dir.join(format!("workflow-state.v0.{timestamp}.json"));
        fs::copy(path, &backup)
            .map_err(|error| format!("备份旧状态文件失败 {}：{error}", backup.display()))?;
        Some(backup)
    } else {
        None
    };

    let mut value = if existed {
        let text = fs::read_to_string(path)
            .map_err(|error| format!("无法读取工作流状态文件 {}：{error}", path.display()))?;
        serde_json::from_str(&text)
            .map_err(|error| format!("工作流状态 JSON 解析失败 {}：{error}", path.display()))?
    } else {
        initial_workflow_state_json(&timestamp, &format!("audit:init:{timestamp}"), false, path)
    };

    let validation_warnings = validate_workflow_state(&value);
    if !validation_warnings.is_empty() {
        return Err(format!(
            "当前状态文件未通过 schema 校验：{}",
            validation_warnings.join(", ")
        ));
    }

    let workflow_id = default_workflow_id(&project.project_root);
    if workflow_exists(&value, &workflow_id) {
        let snapshot = read_workflow_state_snapshot(path)?;
        return Ok(WorkflowStateMutationResult {
            message: format!("该项目已有默认工作流草稿，未重复创建：{}", project.name),
            path: path.display().to_string(),
            backup_path: None,
            audit_event_id: "no-op:existing-workflow".to_string(),
            first_initialize: !existed,
            snapshot,
        });
    }

    append_default_project_workflow(&mut value, project, &timestamp, &audit_event_id)?;
    let validation_warnings = validate_workflow_state(&value);
    if !validation_warnings.is_empty() {
        return Err(format!(
            "写入前 schema 校验失败：{}",
            validation_warnings.join(", ")
        ));
    }

    atomic_write_json(path, &value)?;
    let snapshot = read_workflow_state_snapshot(path)?;
    if !snapshot.exists
        || snapshot.counts.workflows == 0
        || snapshot.counts.nodes < 7
        || snapshot.counts.edges < 6
    {
        return Err("创建项目默认工作流后重新读取校验失败".to_string());
    }

    Ok(WorkflowStateMutationResult {
        message: format!("已创建项目默认工作流草稿：{}", project.name),
        path: path.display().to_string(),
        backup_path: backup_path.map(|backup| backup.display().to_string()),
        audit_event_id,
        first_initialize: !existed,
        snapshot,
    })
}

fn append_default_project_workflow(
    value: &mut Value,
    project: &ProjectRecord,
    timestamp: &str,
    audit_event_id: &str,
) -> Result<(), String> {
    let project_id = project_id(&project.project_root);
    let workflow_id = default_workflow_id(&project.project_root);
    let project_ref = project.project_root.clone();

    if !project_exists(value, &project_id) {
        array_mut(value, "projects")?.push(json!({
          "project_id": project_id,
          "display_name": project.name,
          "root_path": project.project_root,
          "source_kind": "codex_index",
          "permission_level": "read_only",
          "created_at": timestamp,
          "updated_at": timestamp,
          "warnings": project.warnings
        }));
    }

    array_mut(value, "workflows")?.push(json!({
      "workflow_id": workflow_id,
      "workflow_version": 1,
      "project_id": project_id,
      "title": format!("{} 默认工作流草稿", project.name),
      "state": "draft",
      "source_kind": "workspace_state",
      "permission_level": "user_confirmed_write",
      "model_policy": "none",
      "created_at": timestamp,
      "updated_at": timestamp
    }));

    let nodes = [
        (
            "director",
            "director",
            "总指导 / Director",
            "role:director",
            None,
        ),
        (
            "codex-dev",
            "actor",
            "Codex 开发线",
            "adapter:codex-local",
            Some("codex"),
        ),
        ("validation", "validation", "验证线", "role:tester", None),
        ("task", "task", "任务包", "artifact:task-package", None),
        ("handoff", "artifact", "Handoff", "artifact:handoff", None),
        (
            "evidence",
            "artifact",
            "Evidence",
            "artifact:evidence",
            None,
        ),
        ("review", "review", "Review", "artifact:review", None),
    ];
    for (index, (suffix, node_type, title, source_ref, agent_type)) in nodes.iter().enumerate() {
        array_mut(value, "nodes")?.push(json!({
      "node_id": format!("{workflow_id}:node:{suffix}"),
      "workflow_id": workflow_id,
      "node_type": node_type,
      "title": title,
      "state": "draft",
      "source_kind": if *suffix == "codex-dev" { "workspace_state" } else { "derived" },
      "source_ref": source_ref,
      "agent_type": agent_type,
      "adapter_id": if *suffix == "codex-dev" { Value::String("codex-local".to_string()) } else { Value::Null },
      "artifact_type": if ["task", "handoff", "evidence", "review"].contains(suffix) { Value::String((*suffix).to_string()) } else { Value::Null },
      "permission_level": "user_confirmed_write",
      "position": {
        "x": 120 + (index as i64 % 4) * 220,
        "y": 120 + (index as i64 / 4) * 180
      },
      "warnings": []
    }));
    }

    let edges = [
        ("assigns_task", "director", "task", "decomposes_to"),
        ("assigned_to_codex", "task", "codex-dev", "assigned_to"),
        ("produces_handoff", "codex-dev", "handoff", "produces"),
        ("produces_evidence", "codex-dev", "evidence", "produces"),
        ("validates_artifacts", "validation", "evidence", "validates"),
        ("reviews_handoff", "review", "handoff", "reviews"),
    ];
    for (suffix, from, to, edge_type) in edges {
        array_mut(value, "edges")?.push(json!({
          "edge_id": format!("{workflow_id}:edge:{suffix}"),
          "workflow_id": workflow_id,
          "from_node_id": format!("{workflow_id}:node:{from}"),
          "to_node_id": format!("{workflow_id}:node:{to}"),
          "edge_type": edge_type,
          "state": "draft",
          "source_kind": "derived",
          "permission_level": "user_confirmed_write",
          "created_at": timestamp,
          "updated_at": timestamp,
          "warnings": []
        }));
    }

    array_mut(value, "audit_events")?.push(json!({
      "event_id": audit_event_id,
      "event_type": "project_default_workflow_bootstrapped",
      "target_ref": workflow_id,
      "actor_ref": "user_confirmed_desktop_shell",
      "source_kind": "workspace_state",
      "permission_level": "user_confirmed_write",
      "before_state": "missing_project_workflow",
      "after_state": "draft",
      "created_at": timestamp,
      "reason": format!("用户确认给索引内项目创建默认工作流草稿：{project_ref}")
    }));

    value["updated_at"] = Value::String(timestamp.to_string());
    Ok(())
}

fn create_task_draft_at(
    path: &Path,
    request: &TaskDraftRequest,
) -> Result<WorkflowStateMutationResult, String> {
    if request.title.trim().is_empty() {
        return Err("任务包草稿标题不能为空".to_string());
    }
    if request.objective.trim().is_empty() {
        return Err("任务包草稿目标说明不能为空".to_string());
    }
    if !path.exists() {
        return Err("工作流状态文件不存在；请先初始化并创建项目默认工作流草稿".to_string());
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
        return Err("当前项目还没有本地 workflow；请先创建默认工作流草稿".to_string());
    }

    let normalized_title = request.title.trim();
    if task_draft_exists(&value, &workflow_id, normalized_title) {
        let snapshot = read_workflow_state_snapshot(path)?;
        return Ok(WorkflowStateMutationResult {
            message: format!("同名任务包草稿已存在，未重复创建：{normalized_title}"),
            path: path.display().to_string(),
            backup_path: None,
            audit_event_id: "no-op:existing-task-draft".to_string(),
            first_initialize: false,
            snapshot,
        });
    }

    let parent = path
        .parent()
        .ok_or_else(|| format!("状态文件路径没有父目录：{}", path.display()))?;
    let backups_dir = parent.join("backups");
    fs::create_dir_all(&backups_dir)
        .map_err(|error| format!("创建备份目录失败 {}：{error}", backups_dir.display()))?;
    let backup = backups_dir.join(format!("workflow-state.v0.{timestamp}.json"));
    fs::copy(path, &backup)
        .map_err(|error| format!("备份旧状态文件失败 {}：{error}", backup.display()))?;

    let assigned_role = request.assigned_role.as_deref().unwrap_or("codex-dev");
    let work_item_id = format!("work-item:{workflow_id}:{timestamp}");
    let artifact_id = format!("artifact:{workflow_id}:task-package:{timestamp}");
    let audit_event_id = format!(
        "audit:task-draft:{}:{timestamp}",
        stable_id(normalized_title)
    );

    array_mut(&mut value, "work_items")?.push(json!({
      "work_item_id": work_item_id,
      "project_id": project_id(&request.project_root),
      "workflow_id": workflow_id,
      "title": normalized_title,
      "state": "draft",
      "source_kind": "workspace_state",
      "source_ref": artifact_id,
      "assigned_role_id": assigned_role,
      "current_node_id": workflow_node_for_work_item_state(&workflow_id, "draft"),
      "agent_type": "codex",
      "adapter_id": "codex-local",
      "permission_level": "user_confirmed_write",
      "created_at": timestamp,
      "updated_at": timestamp
    }));

    array_mut(&mut value, "artifacts")?.push(json!({
      "artifact_id": artifact_id,
      "artifact_type": "task_package",
      "project_id": project_id(&request.project_root),
      "path": Value::Null,
      "title": normalized_title,
      "brief": request.objective.trim(),
      "source_kind": "workspace_state",
      "source_ref": work_item_id,
      "permission_level": "user_confirmed_write",
      "version": 1,
      "stale": true,
      "stale_reasons": ["任务包仍是草稿，必须补齐字段、检查并生成可派发版本。"],
      "created_at": timestamp,
      "updated_at": timestamp,
      "warnings": ["draft_only_no_markdown_file"]
    }));

    let task_node_before_state = update_task_node_state(&mut value, &workflow_id, &timestamp)?;

    array_mut(&mut value, "audit_events")?.push(json!({
      "event_id": audit_event_id,
      "event_type": "task_draft_created",
      "target_ref": work_item_id,
      "actor_ref": "user_confirmed_desktop_shell",
      "source_kind": "workspace_state",
      "permission_level": "user_confirmed_write",
      "before_state": "missing_task_draft",
      "after_state": "draft",
      "created_at": timestamp,
      "reason": format!("用户确认登记任务包草稿：{normalized_title}")
    }));

    if let Some(before_state) = task_node_before_state {
        array_mut(&mut value, "audit_events")?.push(json!({
          "event_id": format!("audit:task-node-draft:{}:{timestamp}", stable_id(normalized_title)),
          "event_type": "task_node_state_updated",
          "target_ref": format!("{workflow_id}:node:task"),
          "actor_ref": "user_confirmed_desktop_shell",
          "source_kind": "workspace_state",
          "permission_level": "user_confirmed_write",
          "before_state": before_state,
          "after_state": "draft",
          "created_at": timestamp,
          "reason": format!("登记任务包草稿时同步任务包节点状态：{normalized_title}")
        }));
    }

    value["updated_at"] = Value::String(timestamp.to_string());
    let validation_warnings = validate_workflow_state(&value);
    if !validation_warnings.is_empty() {
        return Err(format!(
            "写入前 schema 校验失败：{}",
            validation_warnings.join(", ")
        ));
    }

    atomic_write_json(path, &value)?;
    let snapshot = read_workflow_state_snapshot(path)?;
    if snapshot.counts.work_items == 0 || snapshot.counts.artifacts == 0 {
        return Err("登记任务包草稿后重新读取校验失败".to_string());
    }

    Ok(WorkflowStateMutationResult {
        message: format!("已登记任务包草稿：{normalized_title}"),
        path: path.display().to_string(),
        backup_path: Some(backup.display().to_string()),
        audit_event_id,
        first_initialize: false,
        snapshot,
    })
}

fn render_task_package_preview_at(
    path: &Path,
    project: &ProjectRecord,
    request: &TaskPackagePreviewRequest,
) -> Result<TaskPackagePreview, String> {
    if !path.exists() {
        return Err("工作流状态文件不存在；无法渲染任务包预览".to_string());
    }

    let value = read_workflow_state_value(path)?;
    let validation_warnings = validate_workflow_state(&value);
    if !validation_warnings.is_empty() {
        return Err(format!(
            "当前状态文件未通过 schema 校验：{}",
            validation_warnings.join(", ")
        ));
    }

    let workflow_id = default_workflow_id(&request.project_root);
    if !workflow_exists(&value, &workflow_id) {
        return Err("当前项目还没有本地 workflow；无法渲染任务包预览".to_string());
    }

    let work_item = find_work_item(&value, &workflow_id, &request.work_item_id)
        .ok_or_else(|| "当前 workflow 下找不到该 work item；无法渲染任务包预览".to_string())?;
    let artifact = find_task_package_artifact(&value, &request.work_item_id, &work_item)
        .ok_or_else(|| {
            "当前 work item 找不到 task_package artifact；无法渲染任务包预览".to_string()
        })?;

    let fields = task_package_fields_from(
        work_item,
        artifact,
        project,
        &workflow_id,
        &request.work_item_id,
    );
    let artifact_id = optional_string_from(artifact, "artifact_id");
    let warnings = preview_warnings(work_item, artifact);
    let memory_snapshot = task_memory_injection::snapshot_from_artifact(artifact);
    let memory_injection_summary = memory_snapshot
        .as_ref()
        .map(|snapshot| task_memory_injection::summary_from_snapshot(snapshot, vec![]))
        .unwrap_or_else(task_memory_injection::missing_summary);
    let markdown =
        render_task_package_markdown(&fields, artifact_id.as_deref(), memory_snapshot.as_ref());

    Ok(TaskPackagePreview {
        project_root: request.project_root.clone(),
        workflow_id,
        work_item_id: request.work_item_id.clone(),
        artifact_id,
        markdown,
        memory_injection_summary,
        warnings,
    })
}

fn update_task_package_draft_fields_at(
    path: &Path,
    request: &TaskPackageFieldsUpdateRequest,
) -> Result<WorkflowStateMutationResult, String> {
    update_task_package_fields_at(path, request, TaskPackageFieldWriteMode::DraftUpdate)
}

enum TaskPackageFieldWriteMode {
    DraftUpdate,
    DispatchCorrection,
}

fn update_task_package_fields_at(
    path: &Path,
    request: &TaskPackageFieldsUpdateRequest,
    mode: TaskPackageFieldWriteMode,
) -> Result<WorkflowStateMutationResult, String> {
    if !path.exists() {
        return Err("工作流状态文件不存在；无法更新任务包字段".to_string());
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
        return Err("当前项目还没有本地 workflow；无法更新任务包字段".to_string());
    }

    let work_item_index = find_work_item_index(&value, &workflow_id, &request.work_item_id)
        .ok_or_else(|| "当前 workflow 下找不到该 work item；无法更新任务包字段".to_string())?;
    let current_state = value
        .get("work_items")
        .and_then(Value::as_array)
        .and_then(|items| items.get(work_item_index))
        .and_then(|item| optional_string_from(item, "state"))
        .unwrap_or_else(|| "draft".to_string());
    let artifact_index =
        find_task_package_artifact_index(&value, &request.work_item_id, work_item_index)
            .ok_or_else(|| {
                "当前 work item 找不到 task_package artifact；无法更新任务包字段".to_string()
            })?;

    let parent = path
        .parent()
        .ok_or_else(|| format!("状态文件路径没有父目录：{}", path.display()))?;
    let backups_dir = parent.join("backups");
    fs::create_dir_all(&backups_dir)
        .map_err(|error| format!("创建备份目录失败 {}：{error}", backups_dir.display()))?;
    let backup = backups_dir.join(format!("workflow-state.v0.{timestamp}.json"));
    fs::copy(path, &backup)
        .map_err(|error| format!("备份旧状态文件失败 {}：{error}", backup.display()))?;

    let task_name = cleaned_scalar(&request.fields.task_name);
    let assigned_line = cleaned_scalar(&request.fields.assigned_line);
    let title_for_work_item = if task_name.is_empty() {
        "待补充".to_string()
    } else {
        task_name.clone()
    };
    let assigned_role_id = assigned_line_id(&assigned_line);

    let work_items = array_mut(&mut value, "work_items")?;
    let work_item = work_items
        .get_mut(work_item_index)
        .ok_or_else(|| "当前 workflow 下找不到该 work item；无法更新任务包字段".to_string())?;
    work_item["title"] = Value::String(title_for_work_item.clone());
    work_item["assigned_role_id"] = Value::String(assigned_role_id.to_string());
    work_item["current_node_id"] = Value::String(workflow_node_for_work_item_state(
        &workflow_id,
        &current_state,
    ));
    work_item["updated_at"] = Value::String(timestamp.clone());

    let artifacts = array_mut(&mut value, "artifacts")?;
    let artifact = artifacts.get_mut(artifact_index).ok_or_else(|| {
        "当前 work item 找不到 task_package artifact；无法更新任务包字段".to_string()
    })?;
    artifact["title"] = Value::String(title_for_work_item);
    artifact["brief"] = Value::String(join_lines(&request.fields.goals));
    artifact["task_name"] = Value::String(task_name);
    artifact["assigned_line"] = Value::String(assigned_line);
    artifact["background"] = string_vec_value(&request.fields.background);
    artifact["goals"] = string_vec_value(&request.fields.goals);
    artifact["allowed_read"] = string_vec_value(&request.fields.allowed_read);
    artifact["allowed_write"] = string_vec_value(&request.fields.allowed_write);
    artifact["forbidden_actions"] = string_vec_value(&request.fields.forbidden_actions);
    artifact["acceptance_criteria"] = string_vec_value(&request.fields.acceptance_criteria);
    artifact["required_return"] = string_vec_value(&request.fields.required_return);
    artifact["review_focus"] = string_vec_value(&request.fields.review_focus);
    artifact["template_version"] = Value::String("task_package_v1".to_string());
    artifact["version"] = Value::Number((i64_value(artifact, "version").unwrap_or(0) + 1).into());
    artifact["stale"] = Value::Bool(true);
    artifact["stale_reasons"] = string_vec_value(&vec![
        "任务包字段已编辑，必须重新运行检查并生成新版本。".to_string(),
    ]);
    if matches!(mode, TaskPackageFieldWriteMode::DraftUpdate) {
        artifact["path"] = Value::Null;
    }
    artifact["updated_at"] = Value::String(timestamp.clone());
    artifact["warnings"] = string_vec_value(&field_warning_strings(&request.fields));

    let (audit_prefix, event_type, reason, message) = match mode {
        TaskPackageFieldWriteMode::DraftUpdate => (
            "task-fields",
            "task_package_fields_updated",
            "用户确认更新任务包草稿结构化字段",
            "已更新任务包草稿结构化字段。",
        ),
        TaskPackageFieldWriteMode::DispatchCorrection => (
            "task-fields-correction",
            "task_package_fields_corrected_for_dispatch",
            "用户确认修正任务包派发字段；没有生成真实任务包文件",
            "已保存任务包派发字段修正；没有生成真实任务包文件。",
        ),
    };
    let audit_event_id = format!(
        "audit:{audit_prefix}:{}:{timestamp}",
        stable_id(&request.work_item_id)
    );
    array_mut(&mut value, "audit_events")?.push(json!({
      "event_id": audit_event_id,
      "event_type": event_type,
      "target_ref": request.work_item_id,
      "actor_ref": "user_confirmed_desktop_shell",
      "source_kind": "workspace_state",
      "permission_level": "user_confirmed_write",
      "before_state": "draft",
      "after_state": "draft",
      "created_at": timestamp,
      "reason": reason
    }));

    value["updated_at"] = Value::String(timestamp);
    let validation_warnings = validate_workflow_state(&value);
    if !validation_warnings.is_empty() {
        return Err(format!(
            "写入前 schema 校验失败：{}",
            validation_warnings.join(", ")
        ));
    }

    atomic_write_json(path, &value)?;
    let snapshot = read_workflow_state_snapshot(path)?;
    if !snapshot.exists {
        return Err("更新任务包字段后重新读取校验失败".to_string());
    }

    Ok(WorkflowStateMutationResult {
        message: message.to_string(),
        path: path.display().to_string(),
        backup_path: Some(backup.display().to_string()),
        audit_event_id,
        first_initialize: false,
        snapshot,
    })
}

fn generate_task_package_file_at(
    path: &Path,
    project: &ProjectRecord,
    request: &TaskPackageFileGenerationRequest,
    tasks_dir: &Path,
) -> Result<TaskPackageFileGenerationResult, String> {
    if !path.exists() {
        return Err("工作流状态文件不存在；无法生成真实任务包文件".to_string());
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
        return Err("当前项目还没有本地 workflow；无法生成真实任务包文件".to_string());
    }

    let work_item_index = find_work_item_index(&value, &workflow_id, &request.work_item_id)
        .ok_or_else(|| "当前 workflow 下找不到该 work item；无法生成真实任务包文件".to_string())?;
    let artifact_index =
        find_task_package_artifact_index(&value, &request.work_item_id, work_item_index)
            .ok_or_else(|| {
                "当前 work item 找不到 task_package artifact；无法生成真实任务包文件".to_string()
            })?;

    let work_item = value
        .get("work_items")
        .and_then(Value::as_array)
        .and_then(|items| items.get(work_item_index))
        .ok_or_else(|| "当前 workflow 下找不到该 work item；无法生成真实任务包文件".to_string())?;
    let artifact = value
        .get("artifacts")
        .and_then(Value::as_array)
        .and_then(|artifacts| artifacts.get(artifact_index))
        .ok_or_else(|| {
            "当前 work item 找不到 task_package artifact；无法生成真实任务包文件".to_string()
        })?;

    let fields = task_package_fields_from(
        work_item,
        artifact,
        project,
        &workflow_id,
        &request.work_item_id,
    );
    let artifact_id = optional_string_from(artifact, "artifact_id");
    let memory_input = task_memory_packet_input_from_task_package(
        &request.project_root,
        &workflow_id,
        &request.work_item_id,
        work_item,
        artifact,
        &fields,
    );
    let memory_output = preview_task_memory_packet_at(path, &memory_input, &timestamp)?;
    let memory_snapshot = task_memory_injection::snapshot_from_build_output(
        &memory_output,
        &request.work_item_id,
        artifact_id.as_deref(),
        &timestamp,
    )?;
    let memory_injection_summary =
        task_memory_injection::summary_from_snapshot(&memory_snapshot, vec![]);
    let markdown =
        render_task_package_markdown(&fields, artifact_id.as_deref(), Some(&memory_snapshot));
    validate_generated_task_package_markdown(&markdown)?;

    fs::create_dir_all(tasks_dir)
        .map_err(|error| format!("创建任务包目录失败 {}：{error}", tasks_dir.display()))?;
    let (file_path, file_already_matched) = next_task_package_path_or_existing_match(
        tasks_dir,
        &fields.task_name,
        &request.work_item_id,
        &markdown,
    )?;

    let parent = path
        .parent()
        .ok_or_else(|| format!("状态文件路径没有父目录：{}", path.display()))?;
    let backups_dir = parent.join("backups");
    fs::create_dir_all(&backups_dir)
        .map_err(|error| format!("创建备份目录失败 {}：{error}", backups_dir.display()))?;
    let backup = backups_dir.join(format!("workflow-state.v0.{timestamp}.json"));
    fs::copy(path, &backup)
        .map_err(|error| format!("备份旧状态文件失败 {}：{error}", backup.display()))?;

    if !file_already_matched {
        atomic_write_new_text_file(&file_path, &markdown)?;
    }

    let written = fs::read_to_string(&file_path)
        .map_err(|error| format!("重新读取生成任务包失败 {}：{error}", file_path.display()))?;
    if written != markdown {
        return Err("生成任务包后重新读取校验失败：文件内容不一致".to_string());
    }
    validate_generated_task_package_markdown(&written)?;

    let audit_event_id = format!(
        "audit:task-file:{}:{timestamp}",
        stable_id(&request.work_item_id)
    );
    {
        let artifacts = array_mut(&mut value, "artifacts")?;
        let artifact = artifacts.get_mut(artifact_index).ok_or_else(|| {
            "当前 work item 找不到 task_package artifact；无法生成真实任务包文件".to_string()
        })?;
        artifact["path"] = Value::String(file_path.display().to_string());
        artifact["last_generated_fingerprint"] = Value::String(task_package_fingerprint(&fields));
        task_memory_injection::write_snapshot_to_artifact(artifact, &memory_snapshot)?;
        artifact["stale"] = Value::Bool(false);
        artifact["stale_reasons"] = Value::Array(vec![]);
        artifact["updated_at"] = Value::String(timestamp.clone());
        let warnings = string_array(artifact, "warnings")
            .into_iter()
            .filter(|warning| warning != "draft_only_no_markdown_file")
            .collect::<Vec<_>>();
        artifact["warnings"] = string_vec_value(&warnings);
    }
    array_mut(&mut value, "audit_events")?.push(json!({
      "event_id": audit_event_id,
      "event_type": "task_package_file_generated",
      "target_ref": request.work_item_id,
      "actor_ref": "user_confirmed_desktop_shell",
      "source_kind": "workspace_state",
      "permission_level": "user_confirmed_write",
      "before_state": "draft",
      "after_state": "draft",
      "created_at": timestamp,
      "reason": format!("用户确认从选中任务草稿生成真实任务包文件：{}", file_path.display())
    }));
    let memory_audit_event_id = format!(
        "audit:task-memory-injection:{}:{timestamp}",
        stable_id(&request.work_item_id)
    );
    array_mut(&mut value, "audit_events")?.push(json!({
      "event_id": memory_audit_event_id,
      "event_type": "task_memory_packet_injected_into_task_package",
      "target_ref": request.work_item_id,
      "actor_ref": "user_confirmed_desktop_shell",
      "source_kind": "workspace_state",
      "permission_level": "user_confirmed_write",
      "before_state": "task_package_without_memory_packet_snapshot",
      "after_state": "task_package_with_memory_packet_snapshot",
      "created_at": timestamp,
      "reason": task_memory_injection::audit_reason(&memory_snapshot)
    }));
    value["updated_at"] = Value::String(timestamp);

    let validation_warnings = validate_workflow_state(&value);
    if !validation_warnings.is_empty() {
        return Err(format!(
            "写入前 schema 校验失败：{}",
            validation_warnings.join(", ")
        ));
    }

    atomic_write_json(path, &value)?;
    let snapshot = read_workflow_state_snapshot(path)?;
    let updated = read_workflow_state_value(path)?;
    let updated_artifact = updated
        .get("artifacts")
        .and_then(Value::as_array)
        .and_then(|artifacts| artifacts.get(artifact_index))
        .ok_or_else(|| "生成任务包后状态快照校验失败：artifact 缺失".to_string())?;
    if optional_string_from(updated_artifact, "path").as_deref()
        != Some(file_path.display().to_string().as_str())
    {
        return Err("生成任务包后状态快照校验失败：artifact path 未更新".to_string());
    }

    Ok(TaskPackageFileGenerationResult {
        message: format!("已生成真实任务包文件：{}", file_path.display()),
        file_path: file_path.display().to_string(),
        workflow_state_path: path.display().to_string(),
        backup_path: backup.display().to_string(),
        audit_event_id,
        memory_injection_summary,
        snapshot,
    })
}

fn inspect_task_package_dispatch_readiness_at(
    path: &Path,
    project: &ProjectRecord,
    request: &TaskPackageDispatchReadinessRequest,
) -> Result<TaskPackageDispatchReadiness, String> {
    if !path.exists() {
        return Err("工作流状态文件不存在；无法检查任务包派发准备状态".to_string());
    }

    let value = read_workflow_state_value(path)?;
    let validation_warnings = validate_workflow_state(&value);
    if !validation_warnings.is_empty() {
        return Err(format!(
            "当前状态文件未通过 schema 校验：{}",
            validation_warnings.join(", ")
        ));
    }

    let workflow_id = default_workflow_id(&request.project_root);
    if !workflow_exists(&value, &workflow_id) {
        return Err("当前项目还没有本地 workflow；无法检查任务包派发准备状态".to_string());
    }

    let work_item =
        find_work_item(&value, &workflow_id, &request.work_item_id).ok_or_else(|| {
            "当前 workflow 下找不到该 work item；无法检查任务包派发准备状态".to_string()
        })?;
    let artifact = find_task_package_artifact(&value, &request.work_item_id, work_item)
        .ok_or_else(|| {
            "当前 work item 找不到 task_package artifact；无法检查任务包派发准备状态".to_string()
        })?;

    let fields = task_package_fields_from(
        work_item,
        artifact,
        project,
        &workflow_id,
        &request.work_item_id,
    );
    let artifact_path = optional_string_from(artifact, "path");
    let artifact_id = optional_string_from(artifact, "artifact_id");
    let mut blocking_reasons = dispatch_blocking_reasons(&fields, artifact_path.as_deref());
    let memory_injection_summary =
        task_memory_injection::summary_from_artifact_with_current_revisions(
            path,
            artifact,
            &unix_timestamp_string(),
        )?;
    let authorization_check = inspect_task_package_authorization_at(
        path,
        project,
        &workflow_id,
        work_item,
        artifact,
        &fields,
        "inspect_only",
    )?;
    if authorization_check.status != "authorized" {
        blocking_reasons.extend(authorization_check.reasons.clone());
    }
    let mut warnings = dispatch_warning_reasons(artifact, artifact_path.as_deref());
    warnings.extend(
        memory_injection_summary
            .warnings
            .iter()
            .map(|warning| format!("memory packet: {warning}")),
    );
    warnings.extend(
        memory_injection_summary
            .stale_reasons
            .iter()
            .map(|reason| format!("memory packet stale: {reason}")),
    );
    if bool_value(artifact, "stale") {
        blocking_reasons.push("任务包已 stale；派发前必须重新检查并生成新版本。".to_string());
    }
    if memory_injection_summary.stale && memory_injection_summary.snapshot_id.is_some() {
        blocking_reasons.push("任务包记忆快照已 stale；派发前必须重新生成任务包。".to_string());
    }
    if optional_string_from(artifact, "model_id").is_none() {
        blocking_reasons.push("缺模型；系统不会自动选择模型。".to_string());
    }
    if string_array(artifact, "required_return").is_empty() {
        blocking_reasons.push("缺 report format；必须回传格式未登记。".to_string());
    }
    if bool_value(artifact, "requires_harness")
        && string_array(artifact, "harness_requirements").is_empty()
    {
        blocking_reasons.push("节点要求 harness 但没有配置。".to_string());
    }
    if bool_value(artifact, "requires_tools")
        && string_array(artifact, "callable_tool_capabilities").is_empty()
    {
        blocking_reasons.push("节点需要工具但没有工具白名单。".to_string());
    }
    if bool_value(artifact, "requires_knowledge_refs")
        && string_array(artifact, "available_knowledge_refs").is_empty()
    {
        blocking_reasons.push("任务包声明需要知识库引用，但没有显式引用。".to_string());
    }
    if bool_value(artifact, "requires_memory_refs")
        && string_array(artifact, "available_memory_refs").is_empty()
    {
        blocking_reasons.push("任务包声明需要记忆作为依据，但没有确认记忆引用。".to_string());
    }
    if bool_value(artifact, "requires_memory_refs")
        && memory_injection_summary.snapshot_id.is_none()
    {
        blocking_reasons.push("任务包声明需要记忆作为依据，但记忆快照缺失。".to_string());
    }
    if bool_value(artifact, "requires_memory_refs")
        && memory_injection_summary.snapshot_id.is_some()
        && memory_injection_summary.included_count == 0
    {
        blocking_reasons
            .push("任务包声明需要记忆作为依据，但快照 included 正式记忆为空。".to_string());
    }
    let status = if optional_string_from(artifact, "dispatch_readiness_status").as_deref()
        == Some("blocked")
    {
        "blocked".to_string()
    } else if blocking_reasons.is_empty() {
        "ready".to_string()
    } else {
        "not_ready".to_string()
    };
    if status == "blocked" && blocking_reasons.is_empty() {
        blocking_reasons.push("任务包已被显式标记为 blocked。".to_string());
    }

    Ok(TaskPackageDispatchReadiness {
        project_root: request.project_root.clone(),
        workflow_id,
        work_item_id: request.work_item_id.clone(),
        artifact_id,
        artifact_path,
        status: status.clone(),
        blocking_reasons,
        warnings,
        can_generate_next_version: status == "ready",
        memory_injection_summary,
        authorization_check: Some(authorization_check),
    })
}
