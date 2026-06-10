// Workflow state JSON helpers split out during Root Treatment R2-B2.
// This file is included at crate root so helper visibility and behavior stay unchanged.

fn initial_workflow_state_json(
    timestamp: &str,
    audit_event_id: &str,
    existed: bool,
    path: &Path,
) -> Value {
    json!({
      "schema_version": "workflow_state_v0",
      "workflow_version": 1,
      "workspace_id": workspace_id(),
      "created_at": timestamp,
      "updated_at": timestamp,
      "source_kind": "workspace_state",
      "permission_level": "user_confirmed_write",
      "projects": [],
      "agent_adapters": [
        {
          "adapter_id": "codex-local",
          "agent_type": "codex",
          "agent_id": "codex-local",
          "display_name": "Codex",
          "provider": "local-codex-index",
          "capabilities": ["codex_index_read", "workflow_state_v0"],
          "status": "available",
          "permission_level": "read_only",
          "source_kind": "workspace_state"
        }
      ],
      "workflows": [],
      "nodes": [],
      "edges": [],
      "work_items": [],
      "artifacts": [],
      "reviews": [],
      "workflow_node_session_bindings": [],
      "workflow_node_dispatches": [],
      "audit_events": [
        {
          "event_id": audit_event_id,
          "event_type": "workflow_state_initialized",
          "target_ref": path.display().to_string(),
          "actor_ref": "user_confirmed_desktop_shell",
          "source_kind": "workspace_state",
          "permission_level": "user_confirmed_write",
          "before_state": if existed { "existing_state_backed_up" } else { "missing_state_no_backup" },
          "after_state": "initialized",
          "created_at": timestamp,
          "reason": if existed { "用户确认初始化工作流事实层 v0，写入前已备份旧文件。" } else { "用户确认首次初始化工作流事实层 v0；此前无旧文件可备份。" }
        }
      ],
      "capabilities": [],
      "harness_resources": []
    })
}

fn read_workflow_state_value(path: &Path) -> Result<Value, String> {
    workflow_state_store::read_value(path)
}

fn validate_workflow_state(value: &Value) -> Vec<String> {
    workflow_state_store::validate_value(value, optional_string_from, i64_value)
}

fn write_validated_workflow_state(path: &Path, value: &Value) -> Result<(), String> {
    workflow_state_store::write_validated(path, value, validate_workflow_state, atomic_write_json)
}

fn backup_workflow_state_file(path: &Path, timestamp: &str) -> Result<PathBuf, String> {
    workflow_state_store::backup_file(path, timestamp)
}

fn ensure_workflow_node_session_bindings_array(value: &mut Value) -> Result<(), String> {
    if value.get("workflow_node_session_bindings").is_none() {
        value["workflow_node_session_bindings"] = Value::Array(vec![]);
    }
    if !value
        .get("workflow_node_session_bindings")
        .and_then(Value::as_array)
        .is_some()
    {
        return Err("workflow_node_session_bindings 不是数组".to_string());
    }
    Ok(())
}

fn ensure_workflow_node_dispatches_array(value: &mut Value) -> Result<(), String> {
    if value.get("workflow_node_dispatches").is_none() {
        value["workflow_node_dispatches"] = Value::Array(vec![]);
    }
    if !value
        .get("workflow_node_dispatches")
        .and_then(Value::as_array)
        .is_some()
    {
        return Err("workflow_node_dispatches 不是数组".to_string());
    }
    Ok(())
}

fn array_mut<'a>(value: &'a mut Value, key: &str) -> Result<&'a mut Vec<Value>, String> {
    value
        .get_mut(key)
        .and_then(Value::as_array_mut)
        .ok_or_else(|| format!("{key} 不是数组或缺失"))
}

fn ensure_array_mut<'a>(value: &'a mut Value, key: &str) -> Result<&'a mut Vec<Value>, String> {
    if value.get(key).is_none() {
        value[key] = Value::Array(vec![]);
    }
    array_mut(value, key)
}

fn find_workflow_node_dispatch<'a>(value: &'a Value, dispatch_id: &str) -> Option<&'a Value> {
    value
        .get("workflow_node_dispatches")
        .and_then(Value::as_array)
        .and_then(|dispatches| {
            dispatches.iter().find(|dispatch| {
                optional_string_from(dispatch, "dispatch_id").as_deref() == Some(dispatch_id)
            })
        })
}

fn find_workflow_node_dispatch_index(value: &Value, dispatch_id: &str) -> Option<usize> {
    value
        .get("workflow_node_dispatches")
        .and_then(Value::as_array)
        .and_then(|dispatches| {
            dispatches.iter().position(|dispatch| {
                optional_string_from(dispatch, "dispatch_id").as_deref() == Some(dispatch_id)
            })
        })
}

fn node_exists(value: &Value, workflow_id: &str, node_id: &str) -> bool {
    value
        .get("nodes")
        .and_then(Value::as_array)
        .is_some_and(|nodes| {
            nodes.iter().any(|node| {
                optional_string_from(node, "workflow_id").as_deref() == Some(workflow_id)
                    && optional_string_from(node, "node_id").as_deref() == Some(node_id)
            })
        })
}

fn workflow_node_session_binding_index(
    value: &Value,
    workflow_id: &str,
    node_id: &str,
    work_item_id: Option<&str>,
) -> Option<usize> {
    value
        .get("workflow_node_session_bindings")
        .and_then(Value::as_array)
        .and_then(|bindings| {
            bindings.iter().position(|binding| {
                optional_string_from(binding, "workflow_id").as_deref() == Some(workflow_id)
                    && optional_string_from(binding, "node_id").as_deref() == Some(node_id)
                    && optional_string_from(binding, "lifecycle").as_deref() == Some("active")
                    && optional_string_from(binding, "work_item_id").as_deref() == work_item_id
            })
        })
}

fn project_exists(value: &Value, project_id: &str) -> bool {
    value
        .get("projects")
        .and_then(Value::as_array)
        .is_some_and(|projects| {
            projects.iter().any(|project| {
                optional_string_from(project, "project_id").as_deref() == Some(project_id)
            })
        })
}

fn workflow_exists(value: &Value, workflow_id: &str) -> bool {
    value
        .get("workflows")
        .and_then(Value::as_array)
        .is_some_and(|workflows| {
            workflows.iter().any(|workflow| {
                optional_string_from(workflow, "workflow_id").as_deref() == Some(workflow_id)
            })
        })
}
