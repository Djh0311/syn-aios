use crate::{
    CreateFormalMemoryRecordInput, CreateFormalMemoryRecordOutput, FormalMemoryStoreSummary,
    FormalMemoryStoreV1, MemoryAuditEvent, MemoryAuditRef, MemoryLifecycleStatus, MemoryRecord,
    MemoryVersion,
};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

const STORE_VERSION: &str = "formal_memory_store.v1";
const SIDECAR_NAME: &str = "formal-memories.v1.json";
const LOCK_NAME: &str = ".formal-memories.v1.lock";

pub(crate) fn sidecar_path(workflow_state_path: &Path) -> Result<PathBuf, String> {
    Ok(workflow_state_path
        .parent()
        .ok_or_else(|| {
            format!(
                "workflow state 路径没有父目录，无法推导正式记忆 sidecar：{}",
                workflow_state_path.display()
            )
        })?
        .join(SIDECAR_NAME))
}

pub(crate) fn lock_path_for_sidecar(sidecar: &Path) -> Result<PathBuf, String> {
    Ok(sidecar
        .parent()
        .ok_or_else(|| format!("正式记忆 sidecar 没有父目录：{}", sidecar.display()))?
        .join(LOCK_NAME))
}

pub(crate) fn load_store(
    workflow_state_path: &Path,
    timestamp: &str,
) -> Result<FormalMemoryStoreV1, String> {
    let sidecar = sidecar_path(workflow_state_path)?;
    if !sidecar.exists() {
        return Ok(empty_store(timestamp));
    }
    let text = fs::read_to_string(&sidecar)
        .map_err(|error| format!("读取正式记忆 sidecar 失败 {}：{error}", sidecar.display()))?;
    let store: FormalMemoryStoreV1 = serde_json::from_str(&text).map_err(|error| {
        format!(
            "正式记忆 sidecar JSON 损坏，已拒绝覆盖 {}：{error}",
            sidecar.display()
        )
    })?;
    validate_store(&store)?;
    Ok(store)
}

pub(crate) fn create_record(
    workflow_state_path: &Path,
    input: &CreateFormalMemoryRecordInput,
    timestamp: &str,
    write_id: &str,
) -> Result<CreateFormalMemoryRecordOutput, String> {
    validate_create_input(input)?;
    let sidecar = sidecar_path(workflow_state_path)?;
    let parent = sidecar
        .parent()
        .ok_or_else(|| format!("正式记忆 sidecar 没有父目录：{}", sidecar.display()))?;
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "创建正式记忆 sidecar 目录失败 {}：{error}",
            parent.display()
        )
    })?;
    let lock_path = parent.join(LOCK_NAME);
    let lock = StoreLock::acquire(&lock_path, write_id)?;
    let mut store = load_store(workflow_state_path, timestamp)?;
    if let Some(expected) = input.expected_store_revision {
        if expected != store.revision {
            drop(lock);
            return Err(format!(
                "formal_memory_store_conflict: expected revision {expected}, actual {}",
                store.revision
            ));
        }
    }

    let fingerprint = stable_memory_fingerprint(input)?;
    let memory_id = format!("mem:v1:{timestamp}:{}", short_hash(&fingerprint));
    let audit_event_type = input
        .audit_event_type
        .as_deref()
        .unwrap_or("memory_record_created");
    let audit_event_id = format!(
        "audit:{}:{timestamp}:{}",
        normalize(audit_event_type),
        short_hash(&memory_id)
    );
    let audit_ref = MemoryAuditRef {
        audit_ref_id: format!(
            "audit-ref:{}:{timestamp}:{}",
            normalize(audit_event_type),
            short_hash(&memory_id)
        ),
        audit_event_id: Some(audit_event_id.clone()),
        event_type: audit_event_type.to_string(),
        actor_id: input.actor_id.clone(),
        actor_role: input.actor_role.clone(),
        target_kind: "memory_record".to_string(),
        target_id: memory_id.clone(),
        before_status: None,
        after_status: Some(MemoryLifecycleStatus::MemoryActive),
        reason: input.reason.trim().to_string(),
        created_at: timestamp.to_string(),
    };
    let record = MemoryRecord {
        memory_id: memory_id.clone(),
        schema_version: "memory_governance.v1".to_string(),
        record_version: 1,
        scope: input.scope.clone(),
        memory_type: input.memory_type.clone(),
        claim: input.claim.trim().to_string(),
        body: input.body.trim().to_string(),
        source_refs: input.source_refs.clone(),
        status: MemoryLifecycleStatus::MemoryActive,
        supersedes_memory_id: None,
        superseded_by_memory_id: None,
        conflict_refs: vec![],
        audit_refs: vec![audit_ref],
        created_at: timestamp.to_string(),
        updated_at: timestamp.to_string(),
    };
    let version = MemoryVersion {
        version_id: format!("memver:v1:{timestamp}:{}", short_hash(&memory_id)),
        memory_id: memory_id.clone(),
        version_number: 1,
        change_type: "created".to_string(),
        change_summary: if audit_event_type == "memory_candidate_adopted_to_formal_memory" {
            "从记忆候选受控采纳为正式记忆第一版".to_string()
        } else {
            "创建正式记忆第一版".to_string()
        },
        record_snapshot: record.clone(),
        source_refs: input.source_refs.clone(),
        changed_by_role: input.actor_role.clone(),
        reviewed_by: None,
        created_at: timestamp.to_string(),
    };
    let audit_event = MemoryAuditEvent {
        audit_event_id,
        event_type: audit_event_type.to_string(),
        actor_id: input.actor_id.clone(),
        actor_role: input.actor_role.clone(),
        project_id: input.project_id.clone(),
        workflow_id: input.workflow_id.clone(),
        session_id: input.scope.session_id.clone(),
        target_kind: "memory_record".to_string(),
        target_id: Some(memory_id),
        before_state: None,
        after_state: Some("memory_active".to_string()),
        reason: input.reason.trim().to_string(),
        source_refs: input.source_refs.clone(),
        status: "succeeded".to_string(),
        created_at: timestamp.to_string(),
    };

    store.project_id = input.project_id.clone().or(store.project_id);
    store.workflow_id = input.workflow_id.clone().or(store.workflow_id);
    store.revision += 1;
    store.updated_at = timestamp.to_string();
    store.records.push(record.clone());
    store.versions.push(version.clone());
    store.audit_events.push(audit_event.clone());
    write_store_atomic(&sidecar, &store, timestamp, write_id)?;
    drop(lock);

    Ok(CreateFormalMemoryRecordOutput {
        record,
        version,
        audit_event,
        store_revision: store.revision,
        warnings: vec![
            "formal_memory_store_m1_no_candidate_adoption_or_task_injection".to_string(),
        ],
    })
}

pub(crate) fn summarize_store(store: &FormalMemoryStoreV1) -> FormalMemoryStoreSummary {
    let active_count = store
        .records
        .iter()
        .filter(|record| record.status == MemoryLifecycleStatus::MemoryActive)
        .count();
    let non_active_count = store.records.len().saturating_sub(active_count);
    let recent_audit_event = store.audit_events.last().cloned();
    FormalMemoryStoreSummary {
        sidecar_name: SIDECAR_NAME.to_string(),
        revision: store.revision,
        record_count: store.records.len(),
        active_count,
        non_active_count,
        version_count: store.versions.len(),
        audit_event_count: store.audit_events.len(),
        recent_audit_event,
        warnings: store.warnings.clone(),
        display_text: format!(
            "受控正式记忆：record {} / active {} / version {} / audit {}；创建时写入 version 和 audit；M1 不包含候选采纳和任务包注入",
            store.records.len(),
            active_count,
            store.versions.len(),
            store.audit_events.len()
        ),
    }
}

fn empty_store(timestamp: &str) -> FormalMemoryStoreV1 {
    FormalMemoryStoreV1 {
        store_version: STORE_VERSION.to_string(),
        project_id: None,
        workflow_id: None,
        revision: 0,
        records: vec![],
        versions: vec![],
        audit_events: vec![],
        updated_at: timestamp.to_string(),
        warnings: vec!["formal_memory_store_m1_read_model_only".to_string()],
    }
}

fn validate_store(store: &FormalMemoryStoreV1) -> Result<(), String> {
    if store.store_version != STORE_VERSION {
        return Err(format!(
            "正式记忆 store_version 不匹配：{}",
            store.store_version
        ));
    }
    if store.revision < 0 {
        return Err("正式记忆 revision 不能小于 0".to_string());
    }
    if store
        .records
        .iter()
        .any(|record| memory_status_name(record.status).starts_with("candidate_"))
    {
        return Err("正式记忆 store 含候选状态，已拒绝读取".to_string());
    }
    Ok(())
}

fn validate_create_input(input: &CreateFormalMemoryRecordInput) -> Result<(), String> {
    if input.project_root.trim().is_empty() {
        return Err("正式记忆创建缺少 project_root".to_string());
    }
    if input.reason.trim().is_empty() {
        return Err("正式记忆创建缺少 reason".to_string());
    }
    let source_sensitive_levels = input
        .source_refs
        .iter()
        .map(|source| source.sensitive_level.clone())
        .collect::<Vec<_>>();
    crate::control_core::validate_formal_memory_create(
        &input.claim,
        &input.body,
        input.source_refs.len(),
        &source_sensitive_levels,
        &input.scope.scope_type,
        input.scope.project_id.as_deref(),
        input.scope.workflow_id.as_deref(),
        &input.scope.model_export_policy,
        &input.memory_type,
        &input.actor_role,
        input.project_id.as_deref(),
        input.workflow_id.as_deref(),
    )
}

fn stable_memory_fingerprint(input: &CreateFormalMemoryRecordInput) -> Result<String, String> {
    let refs = input
        .source_refs
        .iter()
        .map(|source| {
            format!(
                "{}:{}:{}",
                normalize(&source.source_type),
                normalize(source.source_id.as_deref().unwrap_or_default()),
                normalize(source.source_path.as_deref().unwrap_or_default())
            )
        })
        .collect::<Vec<_>>();
    if refs.is_empty() {
        return Err("正式记忆来源规范化后为空".to_string());
    }
    Ok(sha256_hex(
        &[
            normalize(&input.scope.scope_type),
            normalize(input.scope.project_id.as_deref().unwrap_or_default()),
            normalize(input.scope.workflow_id.as_deref().unwrap_or_default()),
            normalize(input.scope.session_id.as_deref().unwrap_or_default()),
            normalize(&input.memory_type),
            normalize(&input.claim),
            refs.join("\n"),
        ]
        .join("\0"),
    ))
}

pub(crate) fn write_store_atomic(
    sidecar: &Path,
    store: &FormalMemoryStoreV1,
    timestamp: &str,
    write_id: &str,
) -> Result<(), String> {
    let parent = sidecar
        .parent()
        .ok_or_else(|| format!("正式记忆 sidecar 没有父目录：{}", sidecar.display()))?;
    if sidecar.exists() {
        let backup_dir = parent.join("backups");
        fs::create_dir_all(&backup_dir).map_err(|error| {
            format!("创建正式记忆备份目录失败 {}：{error}", backup_dir.display())
        })?;
        let backup = backup_dir.join(format!(
            "formal-memories.v1.{timestamp}.{}.json",
            store.revision.saturating_sub(1)
        ));
        fs::copy(sidecar, &backup)
            .map_err(|error| format!("备份正式记忆 sidecar 失败 {}：{error}", backup.display()))?;
        prune_backups(&backup_dir, "formal-memories.v1.")?;
    }
    let temp_path = parent.join(format!(".formal-memories.v1.{timestamp}.{write_id}.tmp"));
    let text = serde_json::to_string_pretty(store)
        .map_err(|error| format!("正式记忆 sidecar 序列化失败：{error}"))?;
    {
        let mut file = fs::File::create(&temp_path).map_err(|error| {
            format!("创建正式记忆临时文件失败 {}：{error}", temp_path.display())
        })?;
        file.write_all(text.as_bytes()).map_err(|error| {
            format!("写入正式记忆临时文件失败 {}：{error}", temp_path.display())
        })?;
        file.sync_all().map_err(|error| {
            format!("同步正式记忆临时文件失败 {}：{error}", temp_path.display())
        })?;
    }
    fs::rename(&temp_path, sidecar).map_err(|error| {
        format!(
            "原子替换正式记忆 sidecar 失败 {}：{error}",
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
        .map_err(|error| format!("读取正式记忆备份目录失败 {}：{error}", backup_dir.display()))?
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

fn memory_status_name(status: MemoryLifecycleStatus) -> &'static str {
    match status {
        MemoryLifecycleStatus::CandidateDraft => "candidate_draft",
        MemoryLifecycleStatus::CandidateNeedsReview => "candidate_needs_review",
        MemoryLifecycleStatus::CandidateConfirmed => "candidate_confirmed",
        MemoryLifecycleStatus::CandidateRejected => "candidate_rejected",
        MemoryLifecycleStatus::CandidateQuarantined => "candidate_quarantined",
        MemoryLifecycleStatus::CandidateSuperseded => "candidate_superseded",
        MemoryLifecycleStatus::CandidateDiscarded => "candidate_discarded",
        MemoryLifecycleStatus::MemoryActive => "memory_active",
        MemoryLifecycleStatus::MemoryConflicted => "memory_conflicted",
        MemoryLifecycleStatus::MemoryDeprecated => "memory_deprecated",
        MemoryLifecycleStatus::MemoryFrozen => "memory_frozen",
        MemoryLifecycleStatus::MemoryArchived => "memory_archived",
    }
}

fn normalize(value: &str) -> String {
    value.trim().replace('\\', "/").to_lowercase()
}

fn sha256_hex(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn short_hash(value: &str) -> String {
    sha256_hex(value).chars().take(16).collect()
}

pub(crate) struct StoreLock {
    path: PathBuf,
}

impl StoreLock {
    pub(crate) fn acquire(path: &Path, write_id: &str) -> Result<Self, String> {
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
        {
            Ok(mut file) => {
                file.write_all(write_id.as_bytes()).map_err(|error| {
                    format!("写入正式记忆 lock 失败 {}：{error}", path.display())
                })?;
                Ok(Self {
                    path: path.to_path_buf(),
                })
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                Err(format!("formal_memory_store_locked: {}", path.display()))
            }
            Err(error) => Err(format!(
                "创建正式记忆 lock 失败 {}：{error}",
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
