use serde_json::Value;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

pub(crate) fn read_value(path: &Path) -> Result<Value, String> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("无法读取工作流状态文件 {}：{error}", path.display()))?;
    serde_json::from_str(&text)
        .map_err(|error| format!("工作流状态 JSON 解析失败 {}：{error}", path.display()))
}

pub(crate) fn validate_value(
    value: &Value,
    optional_string_from: fn(&Value, &str) -> Option<String>,
    i64_value: fn(&Value, &str) -> Option<i64>,
) -> Vec<String> {
    let mut warnings = Vec::new();
    if optional_string_from(value, "schema_version").as_deref() != Some("workflow_state_v0") {
        warnings.push("schema_version 不是 workflow_state_v0".to_string());
    }
    if i64_value(value, "workflow_version") != Some(1) {
        warnings.push("workflow_version 不是 1".to_string());
    }
    for key in [
        "projects",
        "agent_adapters",
        "workflows",
        "nodes",
        "edges",
        "work_items",
        "artifacts",
        "reviews",
        "audit_events",
        "capabilities",
        "harness_resources",
    ] {
        if !value.get(key).and_then(Value::as_array).is_some() {
            warnings.push(format!("{key} 不是数组或缺失"));
        }
    }
    warnings
}

pub(crate) fn write_validated(
    path: &Path,
    value: &Value,
    validate_workflow_state: fn(&Value) -> Vec<String>,
    atomic_write_json: fn(&Path, &Value) -> Result<(), String>,
) -> Result<(), String> {
    let validation_warnings = validate_workflow_state(value);
    if !validation_warnings.is_empty() {
        return Err(format!(
            "写入前 schema 校验失败：{}",
            validation_warnings.join(", ")
        ));
    }
    atomic_write_json(path, value)
}

pub(crate) fn backup_file(path: &Path, timestamp: &str) -> Result<PathBuf, String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("状态文件路径没有父目录：{}", path.display()))?;
    let backups_dir = parent.join("backups");
    fs::create_dir_all(&backups_dir)
        .map_err(|error| format!("创建备份目录失败 {}：{error}", backups_dir.display()))?;
    let backup = backups_dir.join(format!("workflow-state.v0.{timestamp}.json"));
    fs::copy(path, &backup)
        .map_err(|error| format!("备份旧状态文件失败 {}：{error}", backup.display()))?;
    Ok(backup)
}

pub(crate) fn atomic_write(path: &Path, value: &Value, timestamp: &str) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("状态文件路径没有父目录：{}", path.display()))?;
    let temp_path = parent.join(format!(".workflow-state.v0.{timestamp}.tmp"));
    let text = serde_json::to_string_pretty(value)
        .map_err(|error| format!("状态 JSON 序列化失败：{error}"))?;
    {
        let mut file = fs::File::create(&temp_path)
            .map_err(|error| format!("创建临时状态文件失败 {}：{error}", temp_path.display()))?;
        file.write_all(text.as_bytes())
            .map_err(|error| format!("写入临时状态文件失败 {}：{error}", temp_path.display()))?;
        file.sync_all()
            .map_err(|error| format!("同步临时状态文件失败 {}：{error}", temp_path.display()))?;
    }
    fs::rename(&temp_path, path)
        .map_err(|error| format!("原子替换状态文件失败 {}：{error}", path.display()))
}
