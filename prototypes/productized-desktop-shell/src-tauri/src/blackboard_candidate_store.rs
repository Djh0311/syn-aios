use crate::utils::hash::{sha256_hex, short_hash};
use crate::{
    BlackboardCandidateAuditEvent, BlackboardCandidateDecision, BlackboardCandidateRecord,
    BlackboardCandidateSourceRef, BlackboardCandidateState, BlackboardCandidateStoreScope,
    BlackboardCandidateStoreV1, RecordBlackboardCandidateDecisionInput,
    RecordBlackboardCandidateDecisionOutput,
};
use std::collections::BTreeSet;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

const SCHEMA_VERSION: &str = "blackboard_candidate_persistence.v1";
const STORAGE_KIND: &str = "sidecar_json_v0";
const SIDECAR_NAME: &str = "blackboard-candidates.v1.json";
const LOCK_NAME: &str = ".blackboard-candidates.v1.lock";

pub(crate) fn sidecar_path(workflow_state_path: &Path) -> Result<PathBuf, String> {
    Ok(workflow_state_path
        .parent()
        .ok_or_else(|| {
            format!(
                "workflow state 路径没有父目录，无法推导黑板候选 sidecar：{}",
                workflow_state_path.display()
            )
        })?
        .join(SIDECAR_NAME))
}

pub(crate) fn load_store(
    workflow_state_path: &Path,
    timestamp: &str,
) -> Result<BlackboardCandidateStoreV1, String> {
    let sidecar = sidecar_path(workflow_state_path)?;
    if !sidecar.exists() {
        return Ok(empty_store(workflow_state_path, &sidecar, timestamp));
    }
    let text = fs::read_to_string(&sidecar)
        .map_err(|error| format!("读取黑板候选 sidecar 失败 {}：{error}", sidecar.display()))?;
    let store: BlackboardCandidateStoreV1 = serde_json::from_str(&text).map_err(|error| {
        format!(
            "黑板候选 sidecar JSON 损坏，已拒绝覆盖 {}：{error}",
            sidecar.display()
        )
    })?;
    validate_store(&store)?;
    Ok(store)
}

pub(crate) fn record_decision(
    workflow_state_path: &Path,
    input: &RecordBlackboardCandidateDecisionInput,
    timestamp: &str,
    write_id: &str,
) -> Result<RecordBlackboardCandidateDecisionOutput, String> {
    validate_input(input)?;
    let sidecar = sidecar_path(workflow_state_path)?;
    let parent = sidecar
        .parent()
        .ok_or_else(|| format!("黑板候选 sidecar 没有父目录：{}", sidecar.display()))?;
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "创建黑板候选 sidecar 目录失败 {}：{error}",
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
                "blackboard_candidate_store_conflict: expected revision {expected}, actual {}",
                store.revision
            ));
        }
    }

    let before_store_revision = store.revision;
    let candidate_key = match input
        .candidate_key
        .as_deref()
        .map(str::trim)
        .filter(|key| !key.is_empty())
    {
        Some(key) => key.to_string(),
        None => stable_candidate_key(input)?,
    };
    let content_fingerprint = content_fingerprint(input);
    let before_index = store
        .records
        .iter()
        .position(|record| record.candidate_key == candidate_key);
    let before_state = before_index.map(|index| store.records[index].state);
    let state = input.requested_state;
    let candidate_id = before_index
        .map(|index| store.records[index].candidate_id.clone())
        .unwrap_or_else(|| format!("bbcand:{}:{}", timestamp, short_hash(&candidate_key)));
    let warnings = decision_warnings(
        input,
        before_state,
        &content_fingerprint,
        before_index.and_then(|index| store.records.get(index)),
    );
    let decision = BlackboardCandidateDecision {
        decision_version: 1,
        decision_id: format!("bbdecision:{}:{}", timestamp, short_hash(&candidate_key)),
        decided_by_role: input.actor_role.clone(),
        decided_by_session_id: input.actor_session_id.clone(),
        decision_reason: input.reason.trim().to_string(),
        decided_at: timestamp.to_string(),
        requested_state: input.requested_state,
        resulting_state: state,
        promotion_target_blocked: true,
        followup_required: state == BlackboardCandidateState::CandidateConfirmedForFollowup,
        followup_task_ref: None,
    };
    let audit_ref = format!(
        "audit:blackboard-candidate:{}:{}",
        timestamp,
        short_hash(&candidate_key)
    );
    let record = match before_index {
        Some(index) => {
            let mut record = store.records[index].clone();
            record.state = state;
            record.content_fingerprint = content_fingerprint.clone();
            record.title_snapshot = input
                .title_snapshot
                .clone()
                .unwrap_or_else(|| record.title_snapshot.clone());
            record.summary_snapshot = input
                .summary_snapshot
                .clone()
                .unwrap_or_else(|| record.summary_snapshot.clone());
            record.source_status = input.source_status.clone().or(record.source_status);
            record.source_refs = input.source_refs.clone();
            record.decision = decision.clone();
            record.updated_at = timestamp.to_string();
            record.last_seen_at = Some(timestamp.to_string());
            record.appearance_count += 1;
            if !record.audit_refs.contains(&audit_ref) {
                record.audit_refs.push(audit_ref.clone());
            }
            record.warnings = merge_warnings(record.warnings, warnings.clone());
            store.records[index] = record.clone();
            record
        }
        None => {
            let mut audit_refs = Vec::new();
            audit_refs.push(audit_ref.clone());
            let record = BlackboardCandidateRecord {
                record_version: 1,
                candidate_id: candidate_id.clone(),
                candidate_key: candidate_key.clone(),
                candidate_key_version: 1,
                content_fingerprint: content_fingerprint.clone(),
                source_entry_id: input.source_entry_id.clone(),
                project_id: input.project_id.clone(),
                project_root: input.project_root.clone(),
                workflow_id: input.workflow_id.clone(),
                work_item_id: input.work_item_id.clone(),
                workflow_node_id: input.workflow_node_id.clone(),
                entry_kind: input.entry_kind,
                target_kind: input.target_kind,
                state,
                title_snapshot: input.title_snapshot.clone().unwrap_or_else(|| {
                    input
                        .source_entry_id
                        .clone()
                        .unwrap_or_else(|| "黑板候选".to_string())
                }),
                summary_snapshot: input
                    .summary_snapshot
                    .clone()
                    .unwrap_or_else(|| input.reason.clone()),
                source_status: input.source_status.clone(),
                source_refs: input.source_refs.clone(),
                decision,
                created_at: timestamp.to_string(),
                updated_at: timestamp.to_string(),
                last_seen_at: Some(timestamp.to_string()),
                appearance_count: 1,
                superseded_by_candidate_id: None,
                audit_refs,
                warnings: warnings.clone(),
            };
            store.records.push(record.clone());
            record
        }
    };
    let next_revision = before_store_revision + 1;
    let audit_event = BlackboardCandidateAuditEvent {
        event_version: 1,
        event_id: audit_ref,
        event_type: event_type_for_state(state).to_string(),
        candidate_id,
        candidate_key,
        project_id: input.project_id.clone(),
        workflow_id: input.workflow_id.clone(),
        actor_role: input.actor_role.clone(),
        actor_session_id: input.actor_session_id.clone(),
        before_state,
        after_state: state,
        store_revision: next_revision,
        reason: input.reason.trim().to_string(),
        created_at: timestamp.to_string(),
        source_refs: input.source_refs.clone(),
        warnings: warnings.clone(),
    };
    store.audit_events.push(audit_event.clone());
    store.revision = next_revision;
    store.last_write_id = Some(write_id.to_string());
    store.updated_at = timestamp.to_string();
    if !store.scope.project_roots.contains(&input.project_root) {
        store.scope.project_roots.push(input.project_root.clone());
        store.scope.project_roots.sort();
    }
    store.scope.workflow_state_path = Some(workflow_state_path.display().to_string());
    store.scope.sidecar_path = Some(sidecar.display().to_string());

    write_store_atomic(&sidecar, &store, timestamp, write_id)?;
    drop(lock);
    Ok(RecordBlackboardCandidateDecisionOutput {
        record,
        audit_event,
        store_revision: store.revision,
        warnings,
    })
}

fn empty_store(
    workflow_state_path: &Path,
    sidecar: &Path,
    timestamp: &str,
) -> BlackboardCandidateStoreV1 {
    BlackboardCandidateStoreV1 {
        schema_version: SCHEMA_VERSION.to_string(),
        store_version: 1,
        storage_kind: STORAGE_KIND.to_string(),
        scope: BlackboardCandidateStoreScope {
            scope_kind: "workflow_state_sidecar".to_string(),
            workflow_state_path: Some(workflow_state_path.display().to_string()),
            sidecar_path: Some(sidecar.display().to_string()),
            project_roots: vec![],
        },
        revision: 0,
        last_write_id: None,
        generated_by: "control_core".to_string(),
        created_at: timestamp.to_string(),
        updated_at: timestamp.to_string(),
        records: vec![],
        audit_events: vec![],
        warnings: vec![],
    }
}

fn validate_store(store: &BlackboardCandidateStoreV1) -> Result<(), String> {
    if store.schema_version != SCHEMA_VERSION {
        return Err(format!(
            "黑板候选 schema_version 不匹配：{}",
            store.schema_version
        ));
    }
    if store.store_version != 1 {
        return Err(format!(
            "黑板候选 store_version 不匹配：{}",
            store.store_version
        ));
    }
    if store.storage_kind != STORAGE_KIND {
        return Err(format!(
            "黑板候选 storage_kind 不匹配：{}",
            store.storage_kind
        ));
    }
    if store.revision < 0 {
        return Err("黑板候选 revision 不能小于 0".to_string());
    }
    Ok(())
}

fn validate_input(input: &RecordBlackboardCandidateDecisionInput) -> Result<(), String> {
    if input.project_id.trim().is_empty() {
        return Err("黑板候选缺少 project_id".to_string());
    }
    if input.project_root.trim().is_empty() {
        return Err("黑板候选缺少 project_root".to_string());
    }
    if input.workflow_id.trim().is_empty() {
        return Err("黑板候选缺少 workflow_id".to_string());
    }
    if input.reason.trim().is_empty() {
        return Err("黑板候选处理缺少 reason".to_string());
    }
    if input.source_refs.is_empty() {
        return Err("黑板候选缺少 source_refs，控制核心已拒绝".to_string());
    }
    if input
        .source_refs
        .iter()
        .any(|source| source.source_kind.trim().is_empty() || source.source_id.trim().is_empty())
    {
        return Err("黑板候选 source_refs 不能有空 source_kind/source_id".to_string());
    }
    Ok(())
}

fn stable_candidate_key(input: &RecordBlackboardCandidateDecisionInput) -> Result<String, String> {
    let normalized_sources = normalize_source_refs(&input.source_refs);
    if normalized_sources.is_empty() {
        return Err("黑板候选 source_refs 规范化后为空".to_string());
    }
    Ok(format!(
        "bbcand:v1:{}",
        sha256_hex(
            &[
                normalize(&input.project_id),
                normalize(&input.workflow_id),
                format!("{:?}", input.entry_kind).to_lowercase(),
                format!("{:?}", input.target_kind).to_lowercase(),
                normalized_sources,
            ]
            .join("\0")
        )
    ))
}

fn content_fingerprint(input: &RecordBlackboardCandidateDecisionInput) -> String {
    format!(
        "bbcand-content:v1:{}",
        sha256_hex(
            &[
                normalize(input.title_snapshot.as_deref().unwrap_or_default()),
                normalize(input.summary_snapshot.as_deref().unwrap_or_default()),
                normalize(input.source_status.as_deref().unwrap_or_default()),
                normalize_source_refs(&input.source_refs),
            ]
            .join("\0")
        )
    )
}

fn decision_warnings(
    input: &RecordBlackboardCandidateDecisionInput,
    before_state: Option<BlackboardCandidateState>,
    content_fingerprint: &str,
    existing: Option<&BlackboardCandidateRecord>,
) -> Vec<String> {
    let mut warnings = vec!["blackboard_candidate_state_only_not_formal_promotion".to_string()];
    if matches!(
        input.target_kind,
        crate::BlackboardCandidateTargetKind::PermissionDecision
    ) {
        warnings.push("permission_decision_requires_permission_command".to_string());
    }
    if matches!(
        input.target_kind,
        crate::BlackboardCandidateTargetKind::FormalMemory
    ) {
        warnings.push("formal_memory_requires_memory_governance".to_string());
    }
    if let (Some(record), Some(state)) = (existing, before_state) {
        if matches!(
            state,
            BlackboardCandidateState::CandidateRejected
                | BlackboardCandidateState::CandidateDiscarded
        ) && record.content_fingerprint != content_fingerprint
        {
            warnings.push(match state {
                BlackboardCandidateState::CandidateRejected => {
                    "source_content_changed_after_rejection".to_string()
                }
                BlackboardCandidateState::CandidateDiscarded => {
                    "source_content_changed_after_discard".to_string()
                }
                _ => unreachable!(),
            });
        }
    }
    warnings
}

fn event_type_for_state(state: BlackboardCandidateState) -> &'static str {
    match state {
        BlackboardCandidateState::CandidatePendingControlCore => {
            "blackboard_candidate_pending_recorded"
        }
        BlackboardCandidateState::CandidateConfirmedForFollowup => "blackboard_candidate_confirmed",
        BlackboardCandidateState::CandidateRejected => "blackboard_candidate_rejected",
        BlackboardCandidateState::CandidateDeferred => "blackboard_candidate_deferred",
        BlackboardCandidateState::CandidateDiscarded => "blackboard_candidate_discarded",
    }
}

fn write_store_atomic(
    sidecar: &Path,
    store: &BlackboardCandidateStoreV1,
    timestamp: &str,
    write_id: &str,
) -> Result<(), String> {
    let parent = sidecar
        .parent()
        .ok_or_else(|| format!("黑板候选 sidecar 没有父目录：{}", sidecar.display()))?;
    if sidecar.exists() {
        let backup_dir = parent.join("backups");
        fs::create_dir_all(&backup_dir).map_err(|error| {
            format!("创建黑板候选备份目录失败 {}：{error}", backup_dir.display())
        })?;
        let backup = backup_dir.join(format!(
            "blackboard-candidates.v1.{timestamp}.{}.json",
            store.revision.saturating_sub(1)
        ));
        fs::copy(sidecar, &backup)
            .map_err(|error| format!("备份黑板候选 sidecar 失败 {}：{error}", backup.display()))?;
        prune_backups(&backup_dir, "blackboard-candidates.v1.")?;
    }
    let temp_path = parent.join(format!(
        ".blackboard-candidates.v1.{timestamp}.{write_id}.tmp"
    ));
    let text = serde_json::to_string_pretty(store)
        .map_err(|error| format!("黑板候选 sidecar 序列化失败：{error}"))?;
    {
        let mut file = fs::File::create(&temp_path).map_err(|error| {
            format!("创建黑板候选临时文件失败 {}：{error}", temp_path.display())
        })?;
        file.write_all(text.as_bytes()).map_err(|error| {
            format!("写入黑板候选临时文件失败 {}：{error}", temp_path.display())
        })?;
        file.sync_all().map_err(|error| {
            format!("同步黑板候选临时文件失败 {}：{error}", temp_path.display())
        })?;
    }
    fs::rename(&temp_path, sidecar).map_err(|error| {
        format!(
            "原子替换黑板候选 sidecar 失败 {}：{error}",
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
        .map_err(|error| format!("读取黑板候选备份目录失败 {}：{error}", backup_dir.display()))?
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

fn merge_warnings(existing: Vec<String>, next: Vec<String>) -> Vec<String> {
    let mut seen = BTreeSet::new();
    existing
        .into_iter()
        .chain(next)
        .filter(|warning| seen.insert(warning.clone()))
        .collect()
}

fn normalize(value: &str) -> String {
    value.trim().replace('\\', "/").to_lowercase()
}

fn normalize_source_refs(source_refs: &[BlackboardCandidateSourceRef]) -> String {
    let mut refs = source_refs
        .iter()
        .map(|source| {
            format!(
                "{}\0{}",
                normalize(&source.source_kind),
                normalize(&source.source_id)
            )
        })
        .filter(|source| !source.trim_matches('\0').is_empty())
        .collect::<Vec<_>>();
    refs.sort();
    refs.dedup();
    refs.join("\n")
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
                    format!("写入黑板候选 lock 失败 {}：{error}", path.display())
                })?;
                Ok(Self {
                    path: path.to_path_buf(),
                })
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Err(format!(
                "blackboard_candidate_store_locked: {}",
                path.display()
            )),
            Err(error) => Err(format!(
                "创建黑板候选 lock 失败 {}：{error}",
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
