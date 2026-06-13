use crate::utils::store_paths;
use crate::{
    MemoryEntityRegistry, MemoryEntityRelationStoreSummary, MemoryEntityRelationStoreV1,
    MemoryRelationStatus,
};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

const STORE_VERSION: &str = "memory_entity_relations.v1";
const SIDECAR_NAME: &str = "memory-entity-relations.v1.json";
const LOCK_NAME: &str = ".memory-entity-relations.v1.lock";

pub(crate) fn sidecar_path(workflow_state_path: &Path) -> Result<PathBuf, String> {
    store_paths::sidecar_path(workflow_state_path, SIDECAR_NAME, "实体关系")
}

pub(crate) fn load_store(
    workflow_state_path: &Path,
    timestamp: &str,
) -> Result<MemoryEntityRelationStoreV1, String> {
    let sidecar = sidecar_path(workflow_state_path)?;
    if !sidecar.exists() {
        return Ok(empty_store(timestamp));
    }
    let text = fs::read_to_string(&sidecar)
        .map_err(|error| format!("读取实体关系 sidecar 失败 {}：{error}", sidecar.display()))?;
    let store: MemoryEntityRelationStoreV1 = serde_json::from_str(&text).map_err(|error| {
        format!(
            "实体关系 sidecar JSON 损坏，已拒绝覆盖 {}：{error}",
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
    F: FnOnce(&mut MemoryEntityRelationStoreV1) -> Result<T, String>,
{
    let sidecar = sidecar_path(workflow_state_path)?;
    let parent = sidecar
        .parent()
        .ok_or_else(|| format!("实体关系 sidecar 没有父目录：{}", sidecar.display()))?;
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "创建实体关系 sidecar 目录失败 {}：{error}",
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

pub(crate) fn summarize_store(
    store: &MemoryEntityRelationStoreV1,
) -> MemoryEntityRelationStoreSummary {
    let confirmed_relation_count = store
        .relations
        .iter()
        .filter(|relation| relation.status == MemoryRelationStatus::Confirmed)
        .count();
    MemoryEntityRelationStoreSummary {
        sidecar_name: SIDECAR_NAME.to_string(),
        revision: store.revision,
        entity_count: store.registry.entities.len(),
        entity_candidate_count: store.entity_candidates.len(),
        merge_candidate_count: store.merge_candidates.len(),
        relation_candidate_count: store.relation_candidates.len(),
        confirmed_relation_count,
        display_text: format!(
            "实体 / 关系治理：entity {} / entity candidates {} / merge candidates {} / relation candidates {} / confirmed relations {}；候选不会自行成为正式事实。",
            store.registry.entities.len(),
            store.entity_candidates.len(),
            store.merge_candidates.len(),
            store.relation_candidates.len(),
            confirmed_relation_count
        ),
        warnings: store.warnings.clone(),
    }
}

fn empty_store(timestamp: &str) -> MemoryEntityRelationStoreV1 {
    MemoryEntityRelationStoreV1 {
        store_version: STORE_VERSION.to_string(),
        project_id: None,
        workflow_id: None,
        revision: 0,
        registry: MemoryEntityRegistry {
            entities: vec![],
            updated_at: timestamp.to_string(),
            warnings: vec!["memory_entity_registry_minimal".to_string()],
        },
        entity_candidates: vec![],
        merge_candidates: vec![],
        relation_candidates: vec![],
        relations: vec![],
        audit_events: vec![],
        updated_at: timestamp.to_string(),
        warnings: vec![
            "memory_entity_relation_store_m10_minimal_sidecar".to_string(),
            "relation_candidates_do_not_affect_task_packet".to_string(),
        ],
    }
}

fn validate_store(store: &MemoryEntityRelationStoreV1) -> Result<(), String> {
    if store.store_version != STORE_VERSION {
        return Err(format!(
            "实体关系 store_version 不匹配：{}",
            store.store_version
        ));
    }
    if store.revision < 0 {
        return Err("实体关系 revision 不能小于 0".to_string());
    }
    Ok(())
}

fn write_store_atomic(
    sidecar: &Path,
    store: &MemoryEntityRelationStoreV1,
    timestamp: &str,
    write_id: &str,
) -> Result<(), String> {
    let parent = sidecar
        .parent()
        .ok_or_else(|| format!("实体关系 sidecar 没有父目录：{}", sidecar.display()))?;
    if sidecar.exists() {
        let backup_dir = parent.join("backups");
        fs::create_dir_all(&backup_dir).map_err(|error| {
            format!("创建实体关系备份目录失败 {}：{error}", backup_dir.display())
        })?;
        let backup = backup_dir.join(format!(
            "memory-entity-relations.v1.{timestamp}.{}.json",
            store.revision.saturating_sub(1)
        ));
        fs::copy(sidecar, &backup)
            .map_err(|error| format!("备份实体关系 sidecar 失败 {}：{error}", backup.display()))?;
        prune_backups(&backup_dir, "memory-entity-relations.v1.")?;
    }
    let temp_path = parent.join(format!(
        ".memory-entity-relations.v1.{timestamp}.{write_id}.tmp"
    ));
    let text = serde_json::to_string_pretty(store)
        .map_err(|error| format!("实体关系 sidecar 序列化失败：{error}"))?;
    {
        let mut file = fs::File::create(&temp_path).map_err(|error| {
            format!("创建实体关系临时文件失败 {}：{error}", temp_path.display())
        })?;
        file.write_all(text.as_bytes()).map_err(|error| {
            format!("写入实体关系临时文件失败 {}：{error}", temp_path.display())
        })?;
        file.sync_all().map_err(|error| {
            format!("同步实体关系临时文件失败 {}：{error}", temp_path.display())
        })?;
    }
    fs::rename(&temp_path, sidecar).map_err(|error| {
        format!(
            "原子替换实体关系 sidecar 失败 {}：{error}",
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
        .map_err(|error| format!("读取实体关系备份目录失败 {}：{error}", backup_dir.display()))?
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
                    format!("写入实体关系 lock 失败 {}：{error}", path.display())
                })?;
                Ok(Self {
                    path: path.to_path_buf(),
                })
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Err(format!(
                "memory_entity_relation_store_locked: {}",
                path.display()
            )),
            Err(error) => Err(format!(
                "创建实体关系 lock 失败 {}：{error}",
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
