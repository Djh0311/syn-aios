use crate::{
    AdoptMemoryCandidateInput, AdoptMemoryCandidateOutput, CreateFormalMemoryRecordOutput,
    CreateMemoryCandidateInput, CreateMemoryCandidateOutput, MemoryAuditRef, MemoryCandidate,
    MemoryCandidateAdoptionRef, MemoryCandidateStoreV1, MemoryLifecycleStatus,
    RecordMemoryCandidateDecisionInput, RecordMemoryCandidateDecisionOutput,
};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

const STORE_VERSION: &str = "memory_candidate_store.v1";
const SIDECAR_NAME: &str = "memory-candidates.v1.json";
const LOCK_NAME: &str = ".memory-candidates.v1.lock";

pub(crate) fn sidecar_path(workflow_state_path: &Path) -> Result<PathBuf, String> {
    Ok(workflow_state_path
        .parent()
        .ok_or_else(|| {
            format!(
                "workflow state 路径没有父目录，无法推导记忆候选 sidecar：{}",
                workflow_state_path.display()
            )
        })?
        .join(SIDECAR_NAME))
}

pub(crate) fn load_store(
    workflow_state_path: &Path,
    timestamp: &str,
) -> Result<MemoryCandidateStoreV1, String> {
    let sidecar = sidecar_path(workflow_state_path)?;
    if !sidecar.exists() {
        return Ok(empty_store(timestamp));
    }
    let text = fs::read_to_string(&sidecar)
        .map_err(|error| format!("读取记忆候选 sidecar 失败 {}：{error}", sidecar.display()))?;
    let store: MemoryCandidateStoreV1 = serde_json::from_str(&text).map_err(|error| {
        format!(
            "记忆候选 sidecar JSON 损坏，已拒绝覆盖 {}：{error}",
            sidecar.display()
        )
    })?;
    validate_store(&store)?;
    Ok(store)
}

pub(crate) fn create_candidate(
    workflow_state_path: &Path,
    input: &CreateMemoryCandidateInput,
    timestamp: &str,
    write_id: &str,
) -> Result<CreateMemoryCandidateOutput, String> {
    validate_create_input(input)?;
    let sidecar = sidecar_path(workflow_state_path)?;
    let parent = sidecar
        .parent()
        .ok_or_else(|| format!("记忆候选 sidecar 没有父目录：{}", sidecar.display()))?;
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "创建记忆候选 sidecar 目录失败 {}：{error}",
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
                "memory_candidate_store_conflict: expected revision {expected}, actual {}",
                store.revision
            ));
        }
    }

    let candidate_key = stable_candidate_key(input)?;
    if store
        .candidates
        .iter()
        .any(|candidate| candidate.candidate_key == candidate_key)
    {
        drop(lock);
        return Err(
            "memory_candidate_duplicate: 相同 candidate_key 已存在；不会自动覆盖候选".to_string(),
        );
    }
    let candidate_id = format!("memcand:{}:{}", timestamp, short_hash(&candidate_key));
    let audit_event = MemoryAuditRef {
        audit_ref_id: format!(
            "audit:memory-candidate-created:{}:{}",
            timestamp,
            short_hash(&candidate_key)
        ),
        audit_event_id: None,
        event_type: "memory_candidate_created".to_string(),
        actor_id: input.generated_by_role.clone(),
        actor_role: input.generated_by_role.clone(),
        target_kind: "memory_candidate".to_string(),
        target_id: candidate_id.clone(),
        before_status: None,
        after_status: Some(MemoryLifecycleStatus::CandidateNeedsReview),
        reason: input.review_reason.trim().to_string(),
        created_at: timestamp.to_string(),
    };
    let candidate = MemoryCandidate {
        candidate_id: candidate_id.clone(),
        candidate_key,
        schema_version: "memory_governance.v1".to_string(),
        scope: input.scope.clone(),
        memory_type: input.memory_type.clone(),
        claim: input.claim.trim().to_string(),
        body: input.body.trim().to_string(),
        source_refs: input.source_refs.clone(),
        generated_by_role: input.generated_by_role.clone(),
        generated_from: input.generated_from.clone(),
        status: MemoryLifecycleStatus::CandidateNeedsReview,
        risk_level: input.risk_level.clone(),
        sensitive_level: input.sensitive_level.clone(),
        requires_user_confirmation: input.requires_user_confirmation,
        review_reason: input.review_reason.trim().to_string(),
        conflicts: vec![],
        audit_refs: vec![audit_event.clone()],
        adoption: None,
        created_at: timestamp.to_string(),
        updated_at: timestamp.to_string(),
    };
    store.project_id = input.project_id.clone().or(store.project_id);
    store.workflow_id = input.workflow_id.clone().or(store.workflow_id);
    store.revision += 1;
    store.updated_at = timestamp.to_string();
    store.events.push(audit_event.clone());
    store.candidates.push(candidate.clone());
    write_store_atomic(&sidecar, &store, timestamp, write_id)?;
    drop(lock);
    Ok(CreateMemoryCandidateOutput {
        candidate,
        audit_event,
        store_revision: store.revision,
        warnings: vec!["memory_candidate_only_not_formal_memory".to_string()],
    })
}

pub(crate) fn record_decision(
    workflow_state_path: &Path,
    input: &RecordMemoryCandidateDecisionInput,
    timestamp: &str,
    write_id: &str,
) -> Result<RecordMemoryCandidateDecisionOutput, String> {
    validate_decision_input(input)?;
    let sidecar = sidecar_path(workflow_state_path)?;
    let parent = sidecar
        .parent()
        .ok_or_else(|| format!("记忆候选 sidecar 没有父目录：{}", sidecar.display()))?;
    let lock_path = parent.join(LOCK_NAME);
    let lock = StoreLock::acquire(&lock_path, write_id)?;
    let mut store = load_store(workflow_state_path, timestamp)?;
    if let Some(expected) = input.expected_store_revision {
        if expected != store.revision {
            drop(lock);
            return Err(format!(
                "memory_candidate_store_conflict: expected revision {expected}, actual {}",
                store.revision
            ));
        }
    }
    let index = store
        .candidates
        .iter()
        .position(|candidate| candidate.candidate_key == input.candidate_key)
        .ok_or_else(|| "未找到记忆候选，已拒绝记录状态".to_string())?;
    let before_status = store.candidates[index].status;
    crate::control_core::validate_memory_candidate_status_transition(
        memory_status_name(before_status),
        memory_status_name(input.requested_status),
    )?;
    let mut candidate = store.candidates[index].clone();
    candidate.status = input.requested_status;
    candidate.updated_at = timestamp.to_string();
    let audit_event = MemoryAuditRef {
        audit_ref_id: format!(
            "audit:memory-candidate-status:{}:{}",
            timestamp,
            short_hash(&input.candidate_key)
        ),
        audit_event_id: None,
        event_type: "memory_candidate_status_changed".to_string(),
        actor_id: input.actor_id.clone(),
        actor_role: input.actor_role.clone(),
        target_kind: "memory_candidate".to_string(),
        target_id: candidate.candidate_id.clone(),
        before_status: Some(before_status),
        after_status: Some(input.requested_status),
        reason: input.reason.trim().to_string(),
        created_at: timestamp.to_string(),
    };
    candidate.audit_refs.push(audit_event.clone());
    store.candidates[index] = candidate.clone();
    store.events.push(audit_event.clone());
    store.revision += 1;
    store.updated_at = timestamp.to_string();
    write_store_atomic(&sidecar, &store, timestamp, write_id)?;
    drop(lock);
    Ok(RecordMemoryCandidateDecisionOutput {
        candidate,
        audit_event,
        store_revision: store.revision,
        warnings: vec!["candidate_confirmed_is_not_memory_record".to_string()],
    })
}

pub(crate) fn adopt_candidate_to_formal_memory<F>(
    workflow_state_path: &Path,
    input: &AdoptMemoryCandidateInput,
    timestamp: &str,
    write_id: &str,
    formal_writer: F,
) -> Result<AdoptMemoryCandidateOutput, String>
where
    F: FnOnce(&MemoryCandidate) -> Result<CreateFormalMemoryRecordOutput, String>,
{
    validate_adoption_input(input)?;
    let sidecar = sidecar_path(workflow_state_path)?;
    let parent = sidecar
        .parent()
        .ok_or_else(|| format!("记忆候选 sidecar 没有父目录：{}", sidecar.display()))?;
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "创建记忆候选 sidecar 目录失败 {}：{error}",
            parent.display()
        )
    })?;
    let lock_path = parent.join(LOCK_NAME);
    let lock = StoreLock::acquire(&lock_path, write_id)?;
    let mut store = load_store(workflow_state_path, timestamp)?;
    if let Some(expected) = input.expected_candidate_store_revision {
        if expected != store.revision {
            drop(lock);
            return Err(format!(
                "memory_candidate_store_conflict: expected revision {expected}, actual {}",
                store.revision
            ));
        }
    }
    let index = store
        .candidates
        .iter()
        .position(|candidate| candidate.candidate_key == input.candidate_key)
        .ok_or_else(|| "未找到记忆候选，已拒绝采纳为正式记忆".to_string())?;
    let candidate = store.candidates[index].clone();
    let source_sensitive_levels = candidate
        .source_refs
        .iter()
        .map(|source| source.sensitive_level.clone())
        .collect::<Vec<_>>();
    crate::control_core::validate_memory_candidate_adoption(
        memory_status_name(candidate.status),
        candidate.adoption.is_some(),
        candidate.source_refs.len(),
        &source_sensitive_levels,
        &candidate.memory_type,
        &candidate.scope.scope_type,
        &candidate.scope.model_export_policy,
        &candidate.risk_level,
        &candidate.sensitive_level,
        candidate.requires_user_confirmation,
        &input.actor_role,
    )?;

    let formal = formal_writer(&candidate)?;
    let adoption = MemoryCandidateAdoptionRef {
        adopted_memory_id: formal.record.memory_id.clone(),
        adopted_version_id: formal.version.version_id.clone(),
        adopted_audit_event_id: formal.audit_event.audit_event_id.clone(),
        adopted_at: timestamp.to_string(),
        adopted_by_role: input.actor_role.clone(),
        adoption_reason: input.adoption_reason.trim().to_string(),
    };
    let audit_event = MemoryAuditRef {
        audit_ref_id: format!(
            "audit:memory-candidate-adopted:{}:{}",
            timestamp,
            short_hash(&input.candidate_key)
        ),
        audit_event_id: Some(formal.audit_event.audit_event_id.clone()),
        event_type: "memory_candidate_adopted_to_formal_memory".to_string(),
        actor_id: input.actor_id.clone(),
        actor_role: input.actor_role.clone(),
        target_kind: "memory_candidate".to_string(),
        target_id: candidate.candidate_id.clone(),
        before_status: Some(candidate.status),
        after_status: Some(MemoryLifecycleStatus::CandidateConfirmed),
        reason: input.adoption_reason.trim().to_string(),
        created_at: timestamp.to_string(),
    };
    let mut updated_candidate = candidate.clone();
    updated_candidate.adoption = Some(adoption.clone());
    updated_candidate.audit_refs.push(audit_event.clone());
    updated_candidate.updated_at = timestamp.to_string();
    store.candidates[index] = updated_candidate;
    store.events.push(audit_event.clone());
    store.revision += 1;
    store.updated_at = timestamp.to_string();
    write_store_atomic(&sidecar, &store, timestamp, write_id).map_err(|error| {
        format!("formal_memory_written_candidate_adoption_link_failed: {error}")
    })?;
    drop(lock);

    let mut warnings = formal.warnings.clone();
    warnings.push("memory_candidate_adopted_to_formal_memory".to_string());
    warnings.push("candidate_history_retained_with_adoption_link".to_string());
    warnings.push("cross_sidecar_write_formal_then_candidate_link".to_string());
    Ok(AdoptMemoryCandidateOutput {
        candidate_key: candidate.candidate_key,
        candidate_status: MemoryLifecycleStatus::CandidateConfirmed,
        record: formal.record,
        version: formal.version,
        audit_event: formal.audit_event,
        adoption,
        candidate_store_revision: store.revision,
        formal_store_revision: formal.store_revision,
        warnings,
    })
}

fn empty_store(timestamp: &str) -> MemoryCandidateStoreV1 {
    MemoryCandidateStoreV1 {
        store_version: STORE_VERSION.to_string(),
        project_id: None,
        workflow_id: None,
        revision: 0,
        candidates: vec![],
        events: vec![],
        updated_at: timestamp.to_string(),
    }
}

fn validate_store(store: &MemoryCandidateStoreV1) -> Result<(), String> {
    if store.store_version != STORE_VERSION {
        return Err(format!(
            "记忆候选 store_version 不匹配：{}",
            store.store_version
        ));
    }
    if store.revision < 0 {
        return Err("记忆候选 revision 不能小于 0".to_string());
    }
    Ok(())
}

fn validate_create_input(input: &CreateMemoryCandidateInput) -> Result<(), String> {
    if input.project_root.trim().is_empty() {
        return Err("记忆候选缺少 project_root".to_string());
    }
    if input.claim.trim().is_empty() {
        return Err("记忆候选 claim 不能为空".to_string());
    }
    if input.body.trim().is_empty() {
        return Err("记忆候选 body 不能为空".to_string());
    }
    crate::control_core::validate_memory_candidate_create(
        input.source_refs.len(),
        &input.scope.scope_type,
        &input.scope.model_export_policy,
        &input.sensitive_level,
    )?;
    Ok(())
}

fn validate_decision_input(input: &RecordMemoryCandidateDecisionInput) -> Result<(), String> {
    if input.project_root.trim().is_empty() {
        return Err("记忆候选处理缺少 project_root".to_string());
    }
    if input.candidate_key.trim().is_empty() {
        return Err("记忆候选处理缺少 candidate_key".to_string());
    }
    if input.reason.trim().is_empty() {
        return Err("记忆候选处理缺少 reason".to_string());
    }
    if memory_status_name(input.requested_status).starts_with("memory_") {
        return Err("记忆候选处理不能请求正式记忆状态".to_string());
    }
    Ok(())
}

fn validate_adoption_input(input: &AdoptMemoryCandidateInput) -> Result<(), String> {
    if input.project_root.trim().is_empty() {
        return Err("记忆候选采纳缺少 project_root".to_string());
    }
    if input.candidate_key.trim().is_empty() {
        return Err("记忆候选采纳缺少 candidate_key".to_string());
    }
    if input.actor_id.trim().is_empty() {
        return Err("记忆候选采纳缺少 actor_id".to_string());
    }
    if input.actor_role.trim().is_empty() {
        return Err("记忆候选采纳缺少 actor_role".to_string());
    }
    if input.adoption_reason.trim().is_empty() {
        return Err("记忆候选采纳缺少 adoption_reason".to_string());
    }
    Ok(())
}

fn stable_candidate_key(input: &CreateMemoryCandidateInput) -> Result<String, String> {
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
        return Err("记忆候选来源规范化后为空".to_string());
    }
    Ok(format!(
        "memcand:v1:{}",
        sha256_hex(
            &[
                normalize(&input.scope.scope_type),
                normalize(input.scope.project_id.as_deref().unwrap_or_default()),
                normalize(input.scope.workflow_id.as_deref().unwrap_or_default()),
                normalize(input.scope.session_id.as_deref().unwrap_or_default()),
                normalize(&input.memory_type),
                normalize(&input.claim),
                refs.join("\n"),
            ]
            .join("\0")
        )
    ))
}

fn write_store_atomic(
    sidecar: &Path,
    store: &MemoryCandidateStoreV1,
    timestamp: &str,
    write_id: &str,
) -> Result<(), String> {
    let parent = sidecar
        .parent()
        .ok_or_else(|| format!("记忆候选 sidecar 没有父目录：{}", sidecar.display()))?;
    if sidecar.exists() {
        let backup_dir = parent.join("backups");
        fs::create_dir_all(&backup_dir).map_err(|error| {
            format!("创建记忆候选备份目录失败 {}：{error}", backup_dir.display())
        })?;
        let backup = backup_dir.join(format!(
            "memory-candidates.v1.{timestamp}.{}.json",
            store.revision.saturating_sub(1)
        ));
        fs::copy(sidecar, &backup)
            .map_err(|error| format!("备份记忆候选 sidecar 失败 {}：{error}", backup.display()))?;
        prune_backups(&backup_dir, "memory-candidates.v1.")?;
    }
    let temp_path = parent.join(format!(".memory-candidates.v1.{timestamp}.{write_id}.tmp"));
    let text = serde_json::to_string_pretty(store)
        .map_err(|error| format!("记忆候选 sidecar 序列化失败：{error}"))?;
    {
        let mut file = fs::File::create(&temp_path).map_err(|error| {
            format!("创建记忆候选临时文件失败 {}：{error}", temp_path.display())
        })?;
        file.write_all(text.as_bytes()).map_err(|error| {
            format!("写入记忆候选临时文件失败 {}：{error}", temp_path.display())
        })?;
        file.sync_all().map_err(|error| {
            format!("同步记忆候选临时文件失败 {}：{error}", temp_path.display())
        })?;
    }
    fs::rename(&temp_path, sidecar).map_err(|error| {
        format!(
            "原子替换记忆候选 sidecar 失败 {}：{error}",
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
        .map_err(|error| format!("读取记忆候选备份目录失败 {}：{error}", backup_dir.display()))?
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

pub(crate) fn memory_status_name(status: MemoryLifecycleStatus) -> &'static str {
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
                    format!("写入记忆候选 lock 失败 {}：{error}", path.display())
                })?;
                Ok(Self {
                    path: path.to_path_buf(),
                })
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                Err(format!("memory_candidate_store_locked: {}", path.display()))
            }
            Err(error) => Err(format!(
                "创建记忆候选 lock 失败 {}：{error}",
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
