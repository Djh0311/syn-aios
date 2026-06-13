use crate::utils::hash::{sha256_hex, short_hash};
use crate::{
    CreateMemoryCandidateFromObservationInput, CreateMemoryCandidateFromObservationOutput,
    CreateMemoryCandidateInput, CreateMemoryCandidateOutput, CreateObservationInput,
    CreateObservationOutput, MemorySourceRef, ObservationAuditRef, ObservationRecord,
    ObservationStatus, ObservationStoreSummary, ObservationStoreV1,
};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

const STORE_VERSION: &str = "observation_store.v1";
const SIDECAR_NAME: &str = "observations.v1.json";
const LOCK_NAME: &str = ".observations.v1.lock";

pub(crate) fn sidecar_path(workflow_state_path: &Path) -> Result<PathBuf, String> {
    Ok(workflow_state_path
        .parent()
        .ok_or_else(|| {
            format!(
                "workflow state 路径没有父目录，无法推导 observation sidecar：{}",
                workflow_state_path.display()
            )
        })?
        .join(SIDECAR_NAME))
}

pub(crate) fn load_store(
    workflow_state_path: &Path,
    timestamp: &str,
) -> Result<ObservationStoreV1, String> {
    let sidecar = sidecar_path(workflow_state_path)?;
    if !sidecar.exists() {
        return Ok(empty_store(timestamp));
    }
    let text = fs::read_to_string(&sidecar).map_err(|error| {
        format!(
            "读取 observation sidecar 失败 {}：{error}",
            sidecar.display()
        )
    })?;
    let store: ObservationStoreV1 = serde_json::from_str(&text).map_err(|error| {
        format!(
            "observation sidecar JSON 损坏，已拒绝覆盖 {}：{error}",
            sidecar.display()
        )
    })?;
    validate_store(&store)?;
    Ok(store)
}

pub(crate) fn create_observation(
    workflow_state_path: &Path,
    input: &CreateObservationInput,
    timestamp: &str,
    write_id: &str,
) -> Result<CreateObservationOutput, String> {
    validate_create_input(input)?;
    let sidecar = sidecar_path(workflow_state_path)?;
    let parent = sidecar
        .parent()
        .ok_or_else(|| format!("observation sidecar 没有父目录：{}", sidecar.display()))?;
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "创建 observation sidecar 目录失败 {}：{error}",
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
                "observation_store_conflict: expected revision {expected}, actual {}",
                store.revision
            ));
        }
    }

    let observation_key = stable_observation_key(input)?;
    if store
        .observations
        .iter()
        .any(|observation| observation.observation_key == observation_key)
    {
        drop(lock);
        return Err(
            "observation_duplicate: 相同 observation_key 已存在；不会自动覆盖观察记录".to_string(),
        );
    }
    let observation_id = format!("obs:v1:{timestamp}:{}", short_hash(&observation_key));
    let audit_event = ObservationAuditRef {
        audit_ref_id: format!(
            "audit:observation-recorded:{timestamp}:{}",
            short_hash(&observation_key)
        ),
        event_type: "observation_recorded".to_string(),
        actor_id: input.actor_id.clone(),
        actor_role: input.generated_by_role.clone(),
        target_kind: "observation".to_string(),
        target_id: observation_id.clone(),
        before_status: None,
        after_status: Some(ObservationStatus::Recorded),
        reason: input.reason.trim().to_string(),
        created_at: timestamp.to_string(),
    };
    let observation = ObservationRecord {
        observation_id: observation_id.clone(),
        observation_key,
        schema_version: "memory_observation.v1".to_string(),
        project_id: input.project_id.clone(),
        workflow_id: input.workflow_id.clone(),
        scope: input.scope.clone(),
        observation_type: input.observation_type.clone(),
        summary: input.summary.trim().to_string(),
        source_refs: input.source_refs.clone(),
        status: ObservationStatus::Recorded,
        generated_by_role: input.generated_by_role.clone(),
        actor_id: input.actor_id.clone(),
        risk_level: input.risk_level.clone(),
        sensitive_level: input.sensitive_level.clone(),
        candidate_key: None,
        audit_refs: vec![audit_event.clone()],
        created_at: timestamp.to_string(),
        updated_at: timestamp.to_string(),
    };
    store.project_id = input.project_id.clone().or(store.project_id);
    store.workflow_id = input.workflow_id.clone().or(store.workflow_id);
    store.revision += 1;
    store.updated_at = timestamp.to_string();
    store.events.push(audit_event.clone());
    store.observations.push(observation.clone());
    write_store_atomic(&sidecar, &store, timestamp, write_id)?;
    drop(lock);

    Ok(CreateObservationOutput {
        observation,
        audit_event,
        store_revision: store.revision,
        warnings: vec!["observation_is_not_formal_memory".to_string()],
    })
}

pub(crate) fn create_memory_candidate_from_observation<F>(
    workflow_state_path: &Path,
    input: &CreateMemoryCandidateFromObservationInput,
    timestamp: &str,
    observation_write_id: &str,
    candidate_creator: F,
) -> Result<CreateMemoryCandidateFromObservationOutput, String>
where
    F: FnOnce(&CreateMemoryCandidateInput) -> Result<CreateMemoryCandidateOutput, String>,
{
    validate_candidate_input(input)?;
    let sidecar = sidecar_path(workflow_state_path)?;
    let parent = sidecar
        .parent()
        .ok_or_else(|| format!("observation sidecar 没有父目录：{}", sidecar.display()))?;
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "创建 observation sidecar 目录失败 {}：{error}",
            parent.display()
        )
    })?;
    let lock_path = parent.join(LOCK_NAME);
    let lock = StoreLock::acquire(&lock_path, observation_write_id)?;
    let mut store = load_store(workflow_state_path, timestamp)?;
    if let Some(expected) = input.expected_observation_store_revision {
        if expected != store.revision {
            drop(lock);
            return Err(format!(
                "observation_store_conflict: expected revision {expected}, actual {}",
                store.revision
            ));
        }
    }
    let index = store
        .observations
        .iter()
        .position(|observation| observation.observation_key == input.observation_key)
        .ok_or_else(|| "未找到 observation，已拒绝生成记忆候选".to_string())?;
    let observation = store.observations[index].clone();
    validate_observation_matches_project_root(&observation, &input.project_root)?;
    crate::control_core::validate_observation_candidate_creation(
        observation_status_name(observation.status),
        observation.source_refs.len(),
        observation.candidate_key.is_some(),
        &input.actor_role,
        &input.memory_type,
        &observation.scope.scope_type,
    )?;

    let candidate_input = candidate_input_from_observation(&observation, input)?;
    let candidate_output = candidate_creator(&candidate_input)
        .map_err(|error| format!("observation_candidate_create_failed: {error}"))?;
    let before_status = observation.status;
    let audit_event = ObservationAuditRef {
        audit_ref_id: format!(
            "audit:observation-candidate-created:{timestamp}:{}",
            short_hash(&input.observation_key)
        ),
        event_type: "observation_candidate_created".to_string(),
        actor_id: input.actor_id.clone(),
        actor_role: input.actor_role.clone(),
        target_kind: "observation".to_string(),
        target_id: observation.observation_id.clone(),
        before_status: Some(before_status),
        after_status: Some(ObservationStatus::CandidateCreated),
        reason: input.review_reason.trim().to_string(),
        created_at: timestamp.to_string(),
    };
    let mut updated_observation = observation.clone();
    updated_observation.status = ObservationStatus::CandidateCreated;
    updated_observation.candidate_key = Some(candidate_output.candidate.candidate_key.clone());
    updated_observation.audit_refs.push(audit_event.clone());
    updated_observation.updated_at = timestamp.to_string();
    store.observations[index] = updated_observation.clone();
    store.events.push(audit_event.clone());
    store.revision += 1;
    store.updated_at = timestamp.to_string();
    write_store_atomic(&sidecar, &store, timestamp, observation_write_id)
        .map_err(|error| format!("memory_candidate_written_observation_link_failed: {error}"))?;
    drop(lock);

    let mut warnings = candidate_output.warnings.clone();
    warnings.push("observation_candidate_created".to_string());
    warnings.push("observation_is_not_formal_memory".to_string());
    warnings.push("candidate_still_needs_review_and_adoption".to_string());
    Ok(CreateMemoryCandidateFromObservationOutput {
        observation: updated_observation,
        candidate: candidate_output.candidate,
        observation_audit_event: audit_event,
        candidate_audit_event: candidate_output.audit_event,
        observation_store_revision: store.revision,
        candidate_store_revision: candidate_output.store_revision,
        warnings,
    })
}

pub(crate) fn summarize_store(store: &ObservationStoreV1) -> ObservationStoreSummary {
    let statuses = store
        .observations
        .iter()
        .map(|observation| observation.status)
        .collect::<Vec<_>>();
    let recorded_count = count_status(&statuses, ObservationStatus::Recorded);
    let candidate_created_count = count_status(&statuses, ObservationStatus::CandidateCreated);
    let ignored_count = count_status(&statuses, ObservationStatus::Ignored);
    let quarantined_count = count_status(&statuses, ObservationStatus::Quarantined);
    let recent_audit_event = store.events.last().cloned();
    let recent_candidate_key = store
        .observations
        .iter()
        .rev()
        .find_map(|observation| observation.candidate_key.clone());
    ObservationStoreSummary {
        sidecar_name: SIDECAR_NAME.to_string(),
        revision: store.revision,
        observation_count: store.observations.len(),
        recorded_count,
        candidate_created_count,
        ignored_count,
        quarantined_count,
        recent_audit_event,
        recent_candidate_key,
        warnings: store.warnings.clone(),
        display_text: format!(
            "工作流观察 {} / recorded {} / candidate_created {} / ignored {} / quarantined {}；observation 不是正式记忆",
            store.observations.len(),
            recorded_count,
            candidate_created_count,
            ignored_count,
            quarantined_count
        ),
    }
}

fn empty_store(timestamp: &str) -> ObservationStoreV1 {
    ObservationStoreV1 {
        store_version: STORE_VERSION.to_string(),
        project_id: None,
        workflow_id: None,
        revision: 0,
        observations: vec![],
        events: vec![],
        updated_at: timestamp.to_string(),
        warnings: vec!["observation_store_m3_candidate_entry_only".to_string()],
    }
}

fn validate_store(store: &ObservationStoreV1) -> Result<(), String> {
    if store.store_version != STORE_VERSION {
        return Err(format!(
            "observation store_version 不匹配：{}",
            store.store_version
        ));
    }
    if store.revision < 0 {
        return Err("observation revision 不能小于 0".to_string());
    }
    for observation in &store.observations {
        if observation.schema_version != "memory_observation.v1" {
            return Err(format!(
                "observation schema_version 不匹配：{}",
                observation.schema_version
            ));
        }
    }
    Ok(())
}

fn validate_create_input(input: &CreateObservationInput) -> Result<(), String> {
    if input.reason.trim().is_empty() {
        return Err("observation 创建缺少 reason".to_string());
    }
    let source_kinds = input
        .source_refs
        .iter()
        .map(|source| source.source_kind.clone())
        .collect::<Vec<_>>();
    let source_sensitive_levels = input
        .source_refs
        .iter()
        .map(|source| source.sensitive_level.clone())
        .collect::<Vec<_>>();
    crate::control_core::validate_observation_create(
        &input.project_root,
        &input.summary,
        input.source_refs.len(),
        &source_kinds,
        &source_sensitive_levels,
        &input.observation_type,
        &input.generated_by_role,
        &input.risk_level,
        &input.sensitive_level,
        &input.scope.scope_type,
        &input.scope.model_export_policy,
    )?;
    for source in &input.source_refs {
        if source.source_ref_id.trim().is_empty() {
            return Err("observation source_ref_id 不能为空".to_string());
        }
        if source.source_id.trim().is_empty() {
            return Err("observation source_id 不能为空".to_string());
        }
        if source.summary.trim().is_empty() {
            return Err("observation source summary 不能为空".to_string());
        }
        if source.created_at.trim().is_empty() {
            return Err("observation source created_at 不能为空".to_string());
        }
    }
    Ok(())
}

fn validate_candidate_input(
    input: &CreateMemoryCandidateFromObservationInput,
) -> Result<(), String> {
    if input.project_root.trim().is_empty() {
        return Err("observation 生成候选缺少 project_root".to_string());
    }
    if input.observation_key.trim().is_empty() {
        return Err("observation 生成候选缺少 observation_key".to_string());
    }
    if input.actor_id.trim().is_empty() {
        return Err("observation 生成候选缺少 actor_id".to_string());
    }
    if input.actor_role.trim().is_empty() {
        return Err("observation 生成候选缺少 actor_role".to_string());
    }
    if input.claim.trim().is_empty() {
        return Err("observation 生成候选缺少 claim".to_string());
    }
    if input.body.trim().is_empty() {
        return Err("observation 生成候选缺少 body".to_string());
    }
    if input.review_reason.trim().is_empty() {
        return Err("observation 生成候选缺少 review_reason".to_string());
    }
    Ok(())
}

fn validate_observation_matches_project_root(
    observation: &ObservationRecord,
    project_root: &str,
) -> Result<(), String> {
    let expected_project_id = crate::project_id(project_root);
    let expected_workflow_id = crate::default_workflow_id(project_root);
    validate_context_field(
        "observation.project_id",
        observation.project_id.as_deref(),
        &expected_project_id,
    )?;
    validate_context_field(
        "observation.workflow_id",
        observation.workflow_id.as_deref(),
        &expected_workflow_id,
    )?;
    validate_context_field(
        "observation.scope.project_id",
        observation.scope.project_id.as_deref(),
        &expected_project_id,
    )?;
    validate_context_field(
        "observation.scope.workflow_id",
        observation.scope.workflow_id.as_deref(),
        &expected_workflow_id,
    )?;
    Ok(())
}

fn validate_context_field(
    field_name: &str,
    actual: Option<&str>,
    expected: &str,
) -> Result<(), String> {
    if let Some(actual) = actual {
        if actual.trim() != expected {
            return Err(format!(
                "observation 上下文绑定失败：{field_name} 与 project_root 不匹配，expected {expected}，actual {}",
                actual.trim()
            ));
        }
    }
    Ok(())
}

fn candidate_input_from_observation(
    observation: &ObservationRecord,
    input: &CreateMemoryCandidateFromObservationInput,
) -> Result<CreateMemoryCandidateInput, String> {
    let mut source_refs = observation
        .source_refs
        .iter()
        .map(observation_source_to_memory_source)
        .collect::<Vec<_>>();
    source_refs.push(MemorySourceRef {
        source_ref_id: format!("source:observation:{}", observation.observation_id),
        source_type: "observation_ref".to_string(),
        source_id: Some(observation.observation_id.clone()),
        source_path: None,
        source_title: Some(observation.summary.clone()),
        anchor: None,
        source_created_at: Some(observation.created_at.clone()),
        captured_at: observation.updated_at.clone(),
        authority_level: "audit".to_string(),
        sensitive_level: candidate_sensitive_level(&observation.sensitive_level),
        content_hash: Some(short_hash(&observation.observation_key)),
    });
    Ok(CreateMemoryCandidateInput {
        project_root: input.project_root.clone(),
        project_id: observation.project_id.clone(),
        workflow_id: observation.workflow_id.clone(),
        scope: observation.scope.clone(),
        memory_type: input.memory_type.clone(),
        claim: input.claim.trim().to_string(),
        body: input.body.trim().to_string(),
        source_refs,
        generated_by_role: input.actor_role.clone(),
        generated_from: format!("observation:{}", observation.observation_id),
        risk_level: observation.risk_level.clone(),
        sensitive_level: candidate_sensitive_level(&observation.sensitive_level),
        requires_user_confirmation: input.requires_user_confirmation
            || observation.risk_level == "high"
            || observation.sensitive_level == "secret",
        review_reason: input.review_reason.trim().to_string(),
        expected_store_revision: input.expected_candidate_store_revision,
    })
}

fn observation_source_to_memory_source(source: &crate::ObservationSourceRef) -> MemorySourceRef {
    MemorySourceRef {
        source_ref_id: format!("source:observation-source:{}", source.source_ref_id),
        source_type: observation_source_kind_to_memory_source_type(&source.source_kind).to_string(),
        source_id: Some(source.source_id.clone()),
        source_path: source.file_path.clone().or(source.evidence_ref.clone()),
        source_title: Some(source.summary.clone()),
        anchor: None,
        source_created_at: Some(source.created_at.clone()),
        captured_at: source.created_at.clone(),
        authority_level: observation_source_kind_to_authority_level(&source.source_kind)
            .to_string(),
        sensitive_level: candidate_sensitive_level(&source.sensitive_level),
        content_hash: Some(short_hash(&format!(
            "{}:{}:{}",
            source.source_kind, source.source_id, source.summary
        ))),
    }
}

fn stable_observation_key(input: &CreateObservationInput) -> Result<String, String> {
    let refs = input
        .source_refs
        .iter()
        .map(|source| {
            format!(
                "{}:{}:{}",
                normalize(&source.source_kind),
                normalize(&source.source_id),
                normalize(source.file_path.as_deref().unwrap_or_default())
            )
        })
        .collect::<Vec<_>>();
    if refs.is_empty() {
        return Err("observation 来源规范化后为空".to_string());
    }
    Ok(format!(
        "obs:v1:{}",
        sha256_hex(
            &[
                normalize(&input.scope.scope_type),
                normalize(input.scope.project_id.as_deref().unwrap_or_default()),
                normalize(input.scope.workflow_id.as_deref().unwrap_or_default()),
                normalize(input.scope.session_id.as_deref().unwrap_or_default()),
                normalize(&input.observation_type),
                normalize(&input.summary),
                refs.join("\n"),
            ]
            .join("\0")
        )
    ))
}

fn write_store_atomic(
    sidecar: &Path,
    store: &ObservationStoreV1,
    timestamp: &str,
    write_id: &str,
) -> Result<(), String> {
    let parent = sidecar
        .parent()
        .ok_or_else(|| format!("observation sidecar 没有父目录：{}", sidecar.display()))?;
    if sidecar.exists() {
        let backup_dir = parent.join("backups");
        fs::create_dir_all(&backup_dir).map_err(|error| {
            format!(
                "创建 observation 备份目录失败 {}：{error}",
                backup_dir.display()
            )
        })?;
        let backup = backup_dir.join(format!(
            "observations.v1.{timestamp}.{}.json",
            store.revision.saturating_sub(1)
        ));
        fs::copy(sidecar, &backup).map_err(|error| {
            format!(
                "备份 observation sidecar 失败 {}：{error}",
                backup.display()
            )
        })?;
        prune_backups(&backup_dir, "observations.v1.")?;
    }
    let temp_path = parent.join(format!(".observations.v1.{timestamp}.{write_id}.tmp"));
    let text = serde_json::to_string_pretty(store)
        .map_err(|error| format!("observation sidecar 序列化失败：{error}"))?;
    {
        let mut file = fs::File::create(&temp_path).map_err(|error| {
            format!(
                "创建 observation 临时文件失败 {}：{error}",
                temp_path.display()
            )
        })?;
        file.write_all(text.as_bytes()).map_err(|error| {
            format!(
                "写入 observation 临时文件失败 {}：{error}",
                temp_path.display()
            )
        })?;
        file.sync_all().map_err(|error| {
            format!(
                "同步 observation 临时文件失败 {}：{error}",
                temp_path.display()
            )
        })?;
    }
    fs::rename(&temp_path, sidecar).map_err(|error| {
        format!(
            "原子替换 observation sidecar 失败 {}：{error}",
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
        .map_err(|error| {
            format!(
                "读取 observation 备份目录失败 {}：{error}",
                backup_dir.display()
            )
        })?
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

pub(crate) fn observation_status_name(status: ObservationStatus) -> &'static str {
    match status {
        ObservationStatus::Recorded => "recorded",
        ObservationStatus::CandidateCreated => "candidate_created",
        ObservationStatus::Ignored => "ignored",
        ObservationStatus::Quarantined => "quarantined",
    }
}

fn observation_source_kind_to_memory_source_type(source_kind: &str) -> &'static str {
    match source_kind {
        "worker_report" => "stage_report",
        "director_review" => "director_review",
        "task_package" => "workflow_summary",
        "evidence" => "evidence",
        "handoff" => "handoff",
        "user_confirmation" => "user_confirmed_proposal",
        "workflow_event" => "audit_event",
        _ => "manual_note",
    }
}

fn observation_source_kind_to_authority_level(source_kind: &str) -> &'static str {
    match source_kind {
        "director_review" | "workflow_event" => "audit",
        "evidence" => "evidence",
        "handoff" => "handoff",
        "user_confirmation" => "user_confirmed",
        "worker_report" | "task_package" => "derived_summary",
        _ => "unverified_note",
    }
}

fn candidate_sensitive_level(observation_sensitive_level: &str) -> String {
    match observation_sensitive_level {
        "public" => "public",
        "internal" => "project",
        "sensitive" => "private",
        "secret" => "secret",
        _ => "private",
    }
    .to_string()
}

fn count_status(statuses: &[ObservationStatus], status: ObservationStatus) -> usize {
    statuses
        .iter()
        .filter(|candidate| **candidate == status)
        .count()
}

fn normalize(value: &str) -> String {
    value.trim().replace('\\', "/").to_lowercase()
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
                    format!("写入 observation lock 失败 {}：{error}", path.display())
                })?;
                Ok(Self {
                    path: path.to_path_buf(),
                })
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                Err(format!("observation_store_locked: {}", path.display()))
            }
            Err(error) => Err(format!(
                "创建 observation lock 失败 {}：{error}",
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
