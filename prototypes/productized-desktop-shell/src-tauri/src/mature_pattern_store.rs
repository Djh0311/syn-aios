use crate::utils::store_paths;
use crate::{MaturePatternCandidateStatus, MemoryPatternStoreSummary, MemoryPatternStoreV1};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

const STORE_VERSION: &str = "memory_patterns.v1";
const SIDECAR_NAME: &str = "memory-patterns.v1.json";
const LOCK_NAME: &str = ".memory-patterns.v1.lock";

pub(crate) fn sidecar_path(workflow_state_path: &Path) -> Result<PathBuf, String> {
    store_paths::sidecar_path(workflow_state_path, SIDECAR_NAME, "成熟模式")
}

pub(crate) fn load_store(
    workflow_state_path: &Path,
    timestamp: &str,
) -> Result<MemoryPatternStoreV1, String> {
    let sidecar = sidecar_path(workflow_state_path)?;
    if !sidecar.exists() {
        return Ok(empty_store(timestamp));
    }
    let text = fs::read_to_string(&sidecar)
        .map_err(|error| format!("读取成熟模式 sidecar 失败 {}：{error}", sidecar.display()))?;
    let store: MemoryPatternStoreV1 = serde_json::from_str(&text).map_err(|error| {
        format!(
            "成熟模式 sidecar JSON 损坏，已拒绝覆盖 {}：{error}",
            sidecar.display()
        )
    })?;
    validate_store(&store)?;
    Ok(store)
}

pub(crate) fn with_locked_store<F, T>(
    workflow_state_path: &Path,
    timestamp: &str,
    write_id: &str,
    mutate: F,
) -> Result<T, String>
where
    F: FnOnce(&mut MemoryPatternStoreV1) -> Result<T, String>,
{
    let sidecar = sidecar_path(workflow_state_path)?;
    let parent = sidecar
        .parent()
        .ok_or_else(|| format!("成熟模式 sidecar 没有父目录：{}", sidecar.display()))?;
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "创建成熟模式 sidecar 目录失败 {}：{error}",
            parent.display()
        )
    })?;
    let lock_path = parent.join(LOCK_NAME);
    let lock = StoreLock::acquire(&lock_path, write_id)?;
    let mut store = load_store(workflow_state_path, timestamp)?;
    let output = mutate(&mut store)?;
    store.updated_at = timestamp.to_string();
    write_store_atomic(&sidecar, &store, timestamp, write_id)?;
    drop(lock);
    Ok(output)
}

pub(crate) fn summarize_store(store: &MemoryPatternStoreV1) -> MemoryPatternStoreSummary {
    let confirmed_pattern_count = store
        .mature_pattern_candidates
        .iter()
        .filter(|candidate| candidate.status == MaturePatternCandidateStatus::Confirmed)
        .count();
    MemoryPatternStoreSummary {
        sidecar_name: SIDECAR_NAME.to_string(),
        revision: store.revision,
        mature_pattern_candidate_count: store.mature_pattern_candidates.len(),
        cluster_report_count: store.cluster_reports.len(),
        confirmed_pattern_count,
        display_text: format!(
            "成熟模式治理：candidate {} / cluster report {} / confirmed {}；候选和报告未确认不会进入任务包。",
            store.mature_pattern_candidates.len(),
            store.cluster_reports.len(),
            confirmed_pattern_count
        ),
        warnings: store.warnings.clone(),
    }
}

fn empty_store(timestamp: &str) -> MemoryPatternStoreV1 {
    MemoryPatternStoreV1 {
        store_version: STORE_VERSION.to_string(),
        project_id: None,
        workflow_id: None,
        revision: 0,
        mature_pattern_candidates: vec![],
        cluster_reports: vec![],
        audit_events: vec![],
        updated_at: timestamp.to_string(),
        warnings: vec![
            "memory_patterns_m12_minimal_sidecar".to_string(),
            "cluster_reports_are_not_formal_memory".to_string(),
        ],
    }
}

fn validate_store(store: &MemoryPatternStoreV1) -> Result<(), String> {
    if store.store_version != STORE_VERSION {
        return Err(format!(
            "成熟模式 store_version 不匹配：{}",
            store.store_version
        ));
    }
    if store.revision < 0 {
        return Err("成熟模式 revision 不能小于 0".to_string());
    }
    Ok(())
}

fn write_store_atomic(
    sidecar: &Path,
    store: &MemoryPatternStoreV1,
    timestamp: &str,
    write_id: &str,
) -> Result<(), String> {
    let parent = sidecar
        .parent()
        .ok_or_else(|| format!("成熟模式 sidecar 没有父目录：{}", sidecar.display()))?;
    if sidecar.exists() {
        let backup_dir = parent.join("backups");
        fs::create_dir_all(&backup_dir).map_err(|error| {
            format!("创建成熟模式备份目录失败 {}：{error}", backup_dir.display())
        })?;
        let backup = backup_dir.join(format!(
            "memory-patterns.v1.{timestamp}.{}.json",
            store.revision.saturating_sub(1)
        ));
        fs::copy(sidecar, &backup)
            .map_err(|error| format!("备份成熟模式 sidecar 失败 {}：{error}", backup.display()))?;
        prune_backups(&backup_dir, "memory-patterns.v1.")?;
    }
    let temp_path = parent.join(format!(".memory-patterns.v1.{timestamp}.{write_id}.tmp"));
    let text = serde_json::to_string_pretty(store)
        .map_err(|error| format!("成熟模式 sidecar 序列化失败：{error}"))?;
    {
        let mut file = fs::File::create(&temp_path).map_err(|error| {
            format!("创建成熟模式临时文件失败 {}：{error}", temp_path.display())
        })?;
        file.write_all(text.as_bytes()).map_err(|error| {
            format!("写入成熟模式临时文件失败 {}：{error}", temp_path.display())
        })?;
        file.sync_all().map_err(|error| {
            format!("同步成熟模式临时文件失败 {}：{error}", temp_path.display())
        })?;
    }
    fs::rename(&temp_path, sidecar).map_err(|error| {
        format!(
            "原子替换成熟模式 sidecar 失败 {}：{error}",
            sidecar.display()
        )
    })?;
    if let Ok(dir) = fs::File::open(parent) {
        let _ = dir.sync_all();
    }
    Ok(())
}

fn prune_backups(backup_dir: &Path, prefix: &str) -> Result<(), String> {
    let mut backups = fs::read_dir(backup_dir)
        .map_err(|error| format!("读取成熟模式备份目录失败 {}：{error}", backup_dir.display()))?
        .filter_map(Result::ok)
        .filter(|entry| entry.file_name().to_string_lossy().starts_with(prefix))
        .collect::<Vec<_>>();
    backups.sort_by_key(|entry| entry.file_name());
    let remove_count = backups.len().saturating_sub(20);
    for entry in backups.into_iter().take(remove_count) {
        let _ = fs::remove_file(entry.path());
    }
    Ok(())
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
                    format!("写入成熟模式 lock 失败 {}：{error}", path.display())
                })?;
                Ok(Self {
                    path: path.to_path_buf(),
                })
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                Err(format!("memory_pattern_store_locked: {}", path.display()))
            }
            Err(error) => Err(format!(
                "创建成熟模式 lock 失败 {}：{error}",
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
