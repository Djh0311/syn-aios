use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};

const LOCK_NAME: &str = ".workflow-state.v0.lock";
const BACKUP_PREFIX: &str = "workflow-state.v0.";
const BACKUP_SUFFIX: &str = ".json";
const RETAIN_RECENT_BACKUPS: usize = 30;
const MILLIS_PER_DAY: u128 = 86_400_000;

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
    if path.exists() {
        read_value(path)?;
    }
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
    let lock_path = workflow_state_lock_path(path)?;
    let _lock = StoreLock::acquire(&lock_path, &format!("backup:{timestamp}"))?;
    let backups_dir = parent.join("backups");
    fs::create_dir_all(&backups_dir)
        .map_err(|error| format!("创建备份目录失败 {}：{error}", backups_dir.display()))?;
    let backup = backups_dir.join(format!("workflow-state.v0.{timestamp}.json"));
    fs::copy(path, &backup)
        .map_err(|error| format!("备份旧状态文件失败 {}：{error}", backup.display()))?;
    prune_workflow_state_backups(&backups_dir)?;
    Ok(backup)
}

pub(crate) fn atomic_write(path: &Path, value: &Value, timestamp: &str) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("状态文件路径没有父目录：{}", path.display()))?;
    let lock_path = workflow_state_lock_path(path)?;
    let _lock = StoreLock::acquire(&lock_path, &format!("write:{timestamp}"))?;
    if path.exists() {
        read_value(path)?;
    }
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

fn workflow_state_lock_path(path: &Path) -> Result<PathBuf, String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("状态文件路径没有父目录：{}", path.display()))?;
    Ok(parent.join(LOCK_NAME))
}

struct StoreLock {
    path: PathBuf,
}

impl StoreLock {
    fn acquire(path: &Path, write_id: &str) -> Result<Self, String> {
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
        {
            Ok(mut file) => {
                file.write_all(write_id.as_bytes()).map_err(|error| {
                    format!("写入 workflow state lock 失败 {}：{error}", path.display())
                })?;
                Ok(Self {
                    path: path.to_path_buf(),
                })
            }
            Err(error) if error.kind() == ErrorKind::AlreadyExists => {
                Err(format!("workflow_state_store_locked: {}", path.display()))
            }
            Err(error) => Err(format!(
                "创建 workflow state lock 失败 {}：{error}",
                path.display()
            )),
        }
    }
}

impl Drop for StoreLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

struct BackupEntry {
    path: PathBuf,
    timestamp: String,
    day_key: String,
}

fn prune_workflow_state_backups(backups_dir: &Path) -> Result<(), String> {
    if !backups_dir.exists() {
        return Ok(());
    }

    let mut entries = Vec::new();
    for entry in fs::read_dir(backups_dir).map_err(|error| {
        format!(
            "读取 workflow state 备份目录失败 {}：{error}",
            backups_dir.display()
        )
    })? {
        let entry = entry.map_err(|error| {
            format!(
                "读取 workflow state 备份目录项失败 {}：{error}",
                backups_dir.display()
            )
        })?;
        let file_type = entry.file_type().map_err(|error| {
            format!(
                "读取 workflow state 备份文件类型失败 {}：{error}",
                entry.path().display()
            )
        })?;
        if !file_type.is_file() {
            continue;
        }
        let file_name = entry.file_name().to_string_lossy().to_string();
        if let Some((timestamp, day_key)) = workflow_state_backup_timestamp_and_day(&file_name) {
            entries.push(BackupEntry {
                path: entry.path(),
                timestamp,
                day_key,
            });
        }
    }

    entries.sort_by(|left, right| {
        right
            .timestamp
            .cmp(&left.timestamp)
            .then_with(|| right.path.cmp(&left.path))
    });

    let mut keep_paths = BTreeSet::new();
    for entry in entries.iter().take(RETAIN_RECENT_BACKUPS) {
        keep_paths.insert(entry.path.clone());
    }

    let mut latest_daily = BTreeMap::new();
    for entry in &entries {
        latest_daily
            .entry(entry.day_key.clone())
            .or_insert_with(|| entry.path.clone());
    }
    keep_paths.extend(latest_daily.into_values());

    for entry in entries {
        if !keep_paths.contains(&entry.path) {
            fs::remove_file(&entry.path).map_err(|error| {
                format!(
                    "删除 workflow state 过期备份失败 {}：{error}",
                    entry.path.display()
                )
            })?;
        }
    }

    Ok(())
}

fn workflow_state_backup_timestamp_and_day(file_name: &str) -> Option<(String, String)> {
    let timestamp = file_name
        .strip_prefix(BACKUP_PREFIX)?
        .strip_suffix(BACKUP_SUFFIX)?;
    if timestamp.is_empty() {
        return None;
    }
    backup_day_key(timestamp).map(|day_key| (timestamp.to_string(), day_key))
}

fn backup_day_key(timestamp: &str) -> Option<String> {
    if timestamp.len() >= 10 {
        let bytes = timestamp.as_bytes();
        let date_like = bytes[0..4].iter().all(u8::is_ascii_digit)
            && bytes[4] == b'-'
            && bytes[5..7].iter().all(u8::is_ascii_digit)
            && bytes[7] == b'-'
            && bytes[8..10].iter().all(u8::is_ascii_digit);
        if date_like {
            return Some(timestamp[..10].to_string());
        }
    }
    if timestamp.bytes().all(|byte| byte.is_ascii_digit()) {
        let millis = timestamp.parse::<u128>().ok()?;
        return Some(format!("millis-day:{}", millis / MILLIS_PER_DAY));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_test_dir(prefix: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        let dir = std::env::temp_dir().join(format!("{prefix}-{unique}"));
        fs::create_dir_all(&dir).expect("test dir should be created");
        dir
    }

    fn valid_state(marker: &str) -> Value {
        json!({
            "schema_version": "workflow_state_v0",
            "workflow_version": 1,
            "updated_at": marker,
            "projects": [],
            "agent_adapters": [],
            "workflows": [],
            "nodes": [],
            "edges": [],
            "work_items": [],
            "artifacts": [],
            "reviews": [],
            "audit_events": [],
            "capabilities": [],
            "harness_resources": []
        })
    }

    fn validate_test_state(value: &Value) -> Vec<String> {
        let mut warnings = Vec::new();
        if value.get("schema_version").and_then(Value::as_str) != Some("workflow_state_v0") {
            warnings.push("schema_version 不是 workflow_state_v0".to_string());
        }
        if value.get("workflow_version").and_then(Value::as_i64) != Some(1) {
            warnings.push("workflow_version 不是 1".to_string());
        }
        warnings
    }

    fn validate_revision_conflict(_value: &Value) -> Vec<String> {
        vec!["workflow revision 不匹配：expected 99, actual 1".to_string()]
    }

    fn atomic_write_test(path: &Path, value: &Value) -> Result<(), String> {
        atomic_write(path, value, "2026-06-10T00-00-testZ")
    }

    #[test]
    fn workflow_state_atomic_write_refuses_lock_busy_without_overwrite() {
        let dir = temp_test_dir("workflow-state-lock-busy");
        let path = dir.join("workflow-state.v0.json");
        atomic_write(&path, &valid_state("old"), "2026-06-10T00-00-00Z")
            .expect("initial write should succeed");
        let original = fs::read_to_string(&path).expect("original state should exist");

        let lock_path = workflow_state_lock_path(&path).expect("lock path should be stable");
        let lock = StoreLock::acquire(&lock_path, "held-by-test").expect("lock should acquire");
        let error = atomic_write(&path, &valid_state("new"), "2026-06-10T00-00-01Z")
            .expect_err("busy lock should reject write");

        assert!(error.contains("workflow_state_store_locked"));
        assert_eq!(fs::read_to_string(&path).unwrap(), original);
        drop(lock);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn workflow_state_write_validated_refuses_corrupt_existing_without_overwrite() {
        let dir = temp_test_dir("workflow-state-corrupt-existing");
        let path = dir.join("workflow-state.v0.json");
        fs::write(&path, "{not-json").expect("corrupt state should be written");

        let error = write_validated(
            &path,
            &valid_state("new"),
            validate_test_state,
            atomic_write_test,
        )
        .expect_err("corrupt existing state should block overwrite");

        assert!(error.contains("工作流状态 JSON 解析失败"));
        assert_eq!(fs::read_to_string(&path).unwrap(), "{not-json");
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn workflow_state_revision_conflict_refuses_write_without_overwrite() {
        let dir = temp_test_dir("workflow-state-revision-conflict");
        let path = dir.join("workflow-state.v0.json");
        atomic_write(&path, &valid_state("old"), "2026-06-10T00-00-00Z")
            .expect("initial write should succeed");
        let original = fs::read_to_string(&path).expect("original state should exist");

        let error = write_validated(
            &path,
            &valid_state("new"),
            validate_revision_conflict,
            atomic_write_test,
        )
        .expect_err("revision conflict should reject write");

        assert!(error.contains("workflow revision 不匹配"));
        assert_eq!(fs::read_to_string(&path).unwrap(), original);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn workflow_state_backup_retention_keeps_recent_30_and_daily_one() {
        let dir = temp_test_dir("workflow-state-backup-retention");
        let backups_dir = dir.join("backups");
        fs::create_dir_all(&backups_dir).expect("backup dir should be created");

        for index in 0..60 {
            let name = format!("workflow-state.v0.2026-06-10T00-00-{index:02}.json");
            fs::write(backups_dir.join(name), "{}").expect("backup fixture should be written");
        }
        for day in 1..=3 {
            for index in 0..3 {
                let name = format!("workflow-state.v0.2026-06-0{day}T00-00-0{index}.json");
                fs::write(backups_dir.join(name), "{}").expect("daily backup should be written");
            }
        }
        fs::write(backups_dir.join("notes.txt"), "keep").expect("non-backup should remain");

        prune_workflow_state_backups(&backups_dir).expect("backup pruning should succeed");

        let remaining = fs::read_dir(&backups_dir)
            .expect("backup dir should be readable")
            .map(|entry| {
                entry
                    .expect("backup entry should be readable")
                    .file_name()
                    .to_string_lossy()
                    .to_string()
            })
            .collect::<Vec<_>>();
        let workflow_backups = remaining
            .iter()
            .filter(|name| name.starts_with("workflow-state.v0."))
            .collect::<Vec<_>>();

        assert_eq!(workflow_backups.len(), 33);
        assert!(remaining.contains(&"notes.txt".to_string()));
        assert!(remaining.contains(&"workflow-state.v0.2026-06-10T00-00-59.json".to_string()));
        assert!(remaining.contains(&"workflow-state.v0.2026-06-01T00-00-02.json".to_string()));
        assert!(!remaining.contains(&"workflow-state.v0.2026-06-10T00-00-00.json".to_string()));
        assert!(!remaining.contains(&"workflow-state.v0.2026-06-01T00-00-00.json".to_string()));
        let _ = fs::remove_dir_all(dir);
    }
}
