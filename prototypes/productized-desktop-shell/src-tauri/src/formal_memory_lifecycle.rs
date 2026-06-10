use crate::{
    FormalMemoryLifecycleImpactSummary, FormalMemoryLifecycleInput,
    FormalMemoryLifecycleOperationKind, FormalMemoryLifecycleOutput, FormalMemoryLifecyclePreview,
    FormalMemoryLifecyclePreviewInput, FormalMemoryLifecycleStatusChange, FormalMemoryMergePlan,
    FormalMemoryRequiredApproval, FormalMemoryRevisePlan, FormalMemoryScopeChangePlan,
    FormalMemorySplitPlan, FormalMemorySplitRecordDraft, FormalMemoryStoreV1, MemoryAuditEvent,
    MemoryAuditRef, MemoryLifecycleStatus, MemoryRecord, MemoryScope, MemorySourceRef,
    MemoryVersion,
};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

pub(crate) fn preview_operation(
    workflow_state_path: &Path,
    input: &FormalMemoryLifecyclePreviewInput,
    timestamp: &str,
) -> Result<FormalMemoryLifecyclePreview, String> {
    let draft = LifecycleDraft::from_preview(input);
    validate_lifecycle_context_binding(workflow_state_path, &draft)?;
    let store = crate::formal_memory_store::load_store(workflow_state_path, timestamp)?;
    validate_expected_revisions(&draft, &store)?;
    let prepared = prepare_operation(&store, &draft, timestamp)?;
    Ok(build_preview(&store, &draft, &prepared, timestamp))
}

pub(crate) fn record_operation(
    workflow_state_path: &Path,
    input: &FormalMemoryLifecycleInput,
    timestamp: &str,
    write_id: &str,
) -> Result<FormalMemoryLifecycleOutput, String> {
    let draft = LifecycleDraft::from_input(input);
    validate_lifecycle_context_binding(workflow_state_path, &draft)?;
    let sidecar = crate::formal_memory_store::sidecar_path(workflow_state_path)?;
    let parent = sidecar
        .parent()
        .ok_or_else(|| format!("正式记忆 sidecar 没有父目录：{}", sidecar.display()))?;
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "创建正式记忆 sidecar 目录失败 {}：{error}",
            parent.display()
        )
    })?;
    let lock_path = crate::formal_memory_store::lock_path_for_sidecar(&sidecar)?;
    let lock = crate::formal_memory_store::StoreLock::acquire(&lock_path, write_id)?;
    let mut store = crate::formal_memory_store::load_store(workflow_state_path, timestamp)?;
    validate_expected_revisions(&draft, &store)?;
    let prepared = prepare_operation(&store, &draft, timestamp)?;
    validate_required_approval(
        &prepared.required_approval,
        input.confirmed_by.as_deref(),
        input.confirmation_summary.as_deref(),
    )?;

    let operation_id = format!(
        "formal-memory-lifecycle:v1:{timestamp}:{}",
        short_hash(&format!(
            "{write_id}:{}",
            operation_name(draft.operation_kind)
        ))
    );
    let audit_event_type = format!(
        "formal_memory_{}_recorded",
        operation_name(draft.operation_kind)
    );
    let audit_event_id = format!(
        "audit:{}:{timestamp}:{}",
        normalize(&audit_event_type),
        short_hash(&operation_id)
    );
    let reason = confirmation_reason(&draft, input.confirmation_summary.as_deref());
    let source_refs = combined_source_refs(
        prepared
            .changes
            .iter()
            .flat_map(|change| change.after.source_refs.iter().cloned()),
    );
    let audit_event = MemoryAuditEvent {
        audit_event_id: audit_event_id.clone(),
        event_type: audit_event_type.clone(),
        actor_id: draft.actor_id.clone(),
        actor_role: draft.actor_role.clone(),
        project_id: draft.project_id.clone(),
        workflow_id: draft.workflow_id.clone(),
        session_id: prepared
            .changes
            .iter()
            .find_map(|change| change.after.scope.session_id.clone()),
        target_kind: "memory_lifecycle_operation".to_string(),
        target_id: Some(operation_id.clone()),
        before_state: Some(prepared.before_state_text()),
        after_state: Some(prepared.after_state_text()),
        reason: reason.clone(),
        source_refs,
        status: "succeeded".to_string(),
        created_at: timestamp.to_string(),
    };

    let mut changed_records = Vec::new();
    let mut versions = Vec::new();
    for (index, change) in prepared.changes.iter().enumerate() {
        let before_status = change.before.as_ref().map(|record| record.status);
        let after_status = Some(change.after.status);
        let mut next_record = change.after.clone();
        next_record.audit_refs.push(MemoryAuditRef {
            audit_ref_id: format!(
                "audit-ref:{}:{timestamp}:{}",
                normalize(&audit_event_type),
                short_hash(&format!("{}:{index}", next_record.memory_id))
            ),
            audit_event_id: Some(audit_event_id.clone()),
            event_type: audit_event_type.clone(),
            actor_id: draft.actor_id.clone(),
            actor_role: draft.actor_role.clone(),
            target_kind: "memory_record".to_string(),
            target_id: next_record.memory_id.clone(),
            before_status,
            after_status,
            reason: reason.clone(),
            created_at: timestamp.to_string(),
        });
        let version = MemoryVersion {
            version_id: format!(
                "memver:v1:{timestamp}:{}",
                short_hash(&format!(
                    "{}:{}:{}:{write_id}",
                    next_record.memory_id, next_record.record_version, index
                ))
            ),
            memory_id: next_record.memory_id.clone(),
            version_number: next_record.record_version,
            change_type: change.change_type.clone(),
            change_summary: change.change_summary.clone(),
            record_snapshot: next_record.clone(),
            source_refs: next_record.source_refs.clone(),
            changed_by_role: draft.actor_role.clone(),
            reviewed_by: input.confirmed_by.clone(),
            created_at: timestamp.to_string(),
        };
        upsert_record(&mut store.records, next_record.clone())?;
        changed_records.push(next_record);
        versions.push(version);
    }

    store.project_id = draft.project_id.clone().or(store.project_id);
    store.workflow_id = draft.workflow_id.clone().or(store.workflow_id);
    store.revision += 1;
    store.updated_at = timestamp.to_string();
    store.versions.extend(versions.clone());
    store.audit_events.push(audit_event.clone());
    crate::formal_memory_store::write_store_atomic(&sidecar, &store, timestamp, write_id)?;
    drop(lock);

    let preview = build_preview(
        &store,
        &draft,
        &prepared.with_after_records(changed_records.clone()),
        timestamp,
    );

    Ok(FormalMemoryLifecycleOutput {
        operation_id,
        preview,
        records: changed_records,
        versions,
        audit_event,
        store_revision: store.revision,
        warnings: prepared.warnings,
    })
}

#[derive(Clone, Debug)]
struct LifecycleDraft {
    project_root: String,
    project_id: Option<String>,
    workflow_id: Option<String>,
    operation_kind: FormalMemoryLifecycleOperationKind,
    memory_id: Option<String>,
    memory_ids: Vec<String>,
    revise: Option<FormalMemoryRevisePlan>,
    merge: Option<FormalMemoryMergePlan>,
    split: Option<FormalMemorySplitPlan>,
    scope_change: Option<FormalMemoryScopeChangePlan>,
    actor_id: String,
    actor_role: String,
    reason: String,
    expected_store_revision: Option<i64>,
    expected_record_versions: BTreeMap<String, i64>,
}

impl LifecycleDraft {
    fn from_preview(input: &FormalMemoryLifecyclePreviewInput) -> Self {
        Self {
            project_root: input.project_root.clone(),
            project_id: input.project_id.clone(),
            workflow_id: input.workflow_id.clone(),
            operation_kind: input.operation_kind,
            memory_id: input.memory_id.clone(),
            memory_ids: input.memory_ids.clone(),
            revise: input.revise.clone(),
            merge: input.merge.clone(),
            split: input.split.clone(),
            scope_change: input.scope_change.clone(),
            actor_id: input.actor_id.clone(),
            actor_role: input.actor_role.clone(),
            reason: input.reason.clone(),
            expected_store_revision: input.expected_store_revision,
            expected_record_versions: input.expected_record_versions.clone(),
        }
    }

    fn from_input(input: &FormalMemoryLifecycleInput) -> Self {
        Self {
            project_root: input.project_root.clone(),
            project_id: input.project_id.clone(),
            workflow_id: input.workflow_id.clone(),
            operation_kind: input.operation_kind,
            memory_id: input.memory_id.clone(),
            memory_ids: input.memory_ids.clone(),
            revise: input.revise.clone(),
            merge: input.merge.clone(),
            split: input.split.clone(),
            scope_change: input.scope_change.clone(),
            actor_id: input.actor_id.clone(),
            actor_role: input.actor_role.clone(),
            reason: input.reason.clone(),
            expected_store_revision: input.expected_store_revision,
            expected_record_versions: input.expected_record_versions.clone(),
        }
    }

    fn target_ids(&self) -> Vec<String> {
        match self.operation_kind {
            FormalMemoryLifecycleOperationKind::Merge => self
                .merge
                .as_ref()
                .map(|plan| plan.source_memory_ids.clone())
                .unwrap_or_else(|| self.memory_ids.clone()),
            FormalMemoryLifecycleOperationKind::Split => self
                .split
                .as_ref()
                .map(|plan| vec![plan.source_memory_id.clone()])
                .unwrap_or_else(|| self.single_or_many_ids()),
            _ => self.single_or_many_ids(),
        }
    }

    fn single_or_many_ids(&self) -> Vec<String> {
        let mut ids = self.memory_ids.clone();
        if let Some(memory_id) = &self.memory_id {
            if !ids.contains(memory_id) {
                ids.push(memory_id.clone());
            }
        }
        ids
    }
}

#[derive(Clone, Debug)]
struct PreparedLifecycle {
    changes: Vec<PreparedRecordChange>,
    target_memory_ids: Vec<String>,
    required_approval: FormalMemoryRequiredApproval,
    warnings: Vec<String>,
}

impl PreparedLifecycle {
    fn before_records(&self) -> Vec<MemoryRecord> {
        self.changes
            .iter()
            .filter_map(|change| change.before.clone())
            .collect()
    }

    fn proposed_records(&self) -> Vec<MemoryRecord> {
        self.changes
            .iter()
            .map(|change| change.after.clone())
            .collect()
    }

    fn before_state_text(&self) -> String {
        self.changes
            .iter()
            .filter_map(|change| {
                change
                    .before
                    .as_ref()
                    .map(|record| format!("{}={}", record.memory_id, status_name(record.status)))
            })
            .collect::<Vec<_>>()
            .join(",")
    }

    fn after_state_text(&self) -> String {
        self.changes
            .iter()
            .map(|change| {
                format!(
                    "{}={}",
                    change.after.memory_id,
                    status_name(change.after.status)
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    }

    fn with_after_records(&self, records: Vec<MemoryRecord>) -> Self {
        let mut next = self.clone();
        for change in &mut next.changes {
            if let Some(record) = records
                .iter()
                .find(|record| record.memory_id == change.after.memory_id)
            {
                change.after = record.clone();
            }
        }
        next
    }
}

#[derive(Clone, Debug)]
struct PreparedRecordChange {
    before: Option<MemoryRecord>,
    after: MemoryRecord,
    change_type: String,
    change_summary: String,
}

fn prepare_operation(
    store: &FormalMemoryStoreV1,
    draft: &LifecycleDraft,
    timestamp: &str,
) -> Result<PreparedLifecycle, String> {
    validate_actor_and_reason(draft)?;
    let target_ids = draft.target_ids();
    if target_ids.is_empty() {
        return Err("正式记忆 lifecycle 操作缺少 memory_id".to_string());
    }
    ensure_unique_ids(&target_ids)?;
    let changes = match draft.operation_kind {
        FormalMemoryLifecycleOperationKind::Revise => prepare_revise(store, draft, timestamp)?,
        FormalMemoryLifecycleOperationKind::Deprecate => prepare_status_change(
            store,
            draft,
            timestamp,
            MemoryLifecycleStatus::MemoryDeprecated,
            "deprecated",
            "废弃正式记忆；不是删除，旧版本继续保留。",
        )?,
        FormalMemoryLifecycleOperationKind::Freeze => prepare_status_change(
            store,
            draft,
            timestamp,
            MemoryLifecycleStatus::MemoryFrozen,
            "frozen",
            "冻结正式记忆；冻结后不能普通编辑。",
        )?,
        FormalMemoryLifecycleOperationKind::Unfreeze => prepare_status_change(
            store,
            draft,
            timestamp,
            MemoryLifecycleStatus::MemoryActive,
            "unfrozen",
            "解冻正式记忆；恢复 active 后仍按任务包规则评估。",
        )?,
        FormalMemoryLifecycleOperationKind::Archive => prepare_status_change(
            store,
            draft,
            timestamp,
            MemoryLifecycleStatus::MemoryArchived,
            "archived",
            "归档正式记忆；非 active 记忆默认不进任务包。",
        )?,
        FormalMemoryLifecycleOperationKind::Merge => prepare_merge(store, draft, timestamp)?,
        FormalMemoryLifecycleOperationKind::Split => prepare_split(store, draft, timestamp)?,
        FormalMemoryLifecycleOperationKind::PromoteToGlobal => {
            prepare_scope_change(store, draft, timestamp, "promoted_to_global")?
        }
        FormalMemoryLifecycleOperationKind::DemoteToProject => {
            prepare_scope_change(store, draft, timestamp, "demoted_to_project")?
        }
    };
    validate_context_for_changes(draft, &changes)?;
    let required_approval = required_approval(draft.operation_kind, &changes);
    Ok(PreparedLifecycle {
        changes,
        target_memory_ids: target_ids,
        required_approval,
        warnings: lifecycle_warnings(draft.operation_kind),
    })
}

fn prepare_revise(
    store: &FormalMemoryStoreV1,
    draft: &LifecycleDraft,
    timestamp: &str,
) -> Result<Vec<PreparedRecordChange>, String> {
    let memory_id = single_target_id(draft)?;
    let before = find_record(store, memory_id)?;
    require_status(
        before,
        &[MemoryLifecycleStatus::MemoryActive],
        "revise 只能处理 active 正式记忆；冻结后需先解冻",
    )?;
    if before.superseded_by_memory_id.is_some() {
        return Err("revise 已拒绝：被替代的正式记忆不能普通编辑".to_string());
    }
    let plan = draft
        .revise
        .as_ref()
        .ok_or_else(|| "revise 操作缺少 revise plan".to_string())?;
    let mut after = before.clone();
    apply_revise_plan(&mut after, plan);
    after.record_version += 1;
    after.updated_at = timestamp.to_string();
    validate_memory_payload(&after)?;
    Ok(vec![PreparedRecordChange {
        before: Some(before.clone()),
        after,
        change_type: "manual_revision".to_string(),
        change_summary: "人工编辑提案已记录为新版本；旧版本未覆盖。".to_string(),
    }])
}

fn prepare_status_change(
    store: &FormalMemoryStoreV1,
    draft: &LifecycleDraft,
    timestamp: &str,
    next_status: MemoryLifecycleStatus,
    change_type: &str,
    change_summary: &str,
) -> Result<Vec<PreparedRecordChange>, String> {
    let memory_id = single_target_id(draft)?;
    let before = find_record(store, memory_id)?;
    match (draft.operation_kind, before.status) {
        (FormalMemoryLifecycleOperationKind::Freeze, MemoryLifecycleStatus::MemoryActive) => {}
        (FormalMemoryLifecycleOperationKind::Unfreeze, MemoryLifecycleStatus::MemoryFrozen) => {}
        (FormalMemoryLifecycleOperationKind::Deprecate, MemoryLifecycleStatus::MemoryArchived)
        | (FormalMemoryLifecycleOperationKind::Archive, MemoryLifecycleStatus::MemoryArchived) => {
            return Err("正式记忆已经归档，不能重复执行该 lifecycle 状态操作".to_string());
        }
        (FormalMemoryLifecycleOperationKind::Deprecate, _)
        | (FormalMemoryLifecycleOperationKind::Archive, _) => {}
        _ => {
            return Err(format!(
                "{} 状态不允许执行 {} 操作",
                status_name(before.status),
                operation_name(draft.operation_kind)
            ));
        }
    }
    let mut after = before.clone();
    after.status = next_status;
    after.record_version += 1;
    after.updated_at = timestamp.to_string();
    Ok(vec![PreparedRecordChange {
        before: Some(before.clone()),
        after,
        change_type: change_type.to_string(),
        change_summary: change_summary.to_string(),
    }])
}

fn prepare_merge(
    store: &FormalMemoryStoreV1,
    draft: &LifecycleDraft,
    timestamp: &str,
) -> Result<Vec<PreparedRecordChange>, String> {
    let plan = draft
        .merge
        .as_ref()
        .ok_or_else(|| "merge 操作缺少 merge plan".to_string())?;
    if plan.source_memory_ids.len() < 2 {
        return Err("merge 必须显式选择至少两条正式记忆".to_string());
    }
    ensure_unique_ids(&plan.source_memory_ids)?;
    if plan.merged_claim.trim().is_empty() || plan.merged_body.trim().is_empty() {
        return Err("merge 必须提供明确的合并后 claim/body".to_string());
    }
    let sources = plan
        .source_memory_ids
        .iter()
        .map(|memory_id| find_record(store, memory_id).cloned())
        .collect::<Result<Vec<_>, _>>()?;
    for source in &sources {
        require_merge_source(source)?;
    }

    let target_id = plan.target_memory_id.clone().unwrap_or_else(|| {
        format!(
            "mem:v1:{timestamp}:{}",
            short_hash(&format!(
                "merge:{}:{}",
                store.revision + 1,
                plan.source_memory_ids.join(",")
            ))
        )
    });
    if store
        .records
        .iter()
        .any(|record| record.memory_id == target_id)
        && plan.target_memory_id.is_none()
    {
        return Err("merge 生成的新 memory_id 已存在，已拒绝写入".to_string());
    }

    let source_id_text = plan.source_memory_ids.join(",");
    let mut changes = Vec::new();
    if let Some(target_memory_id) = &plan.target_memory_id {
        if !plan.source_memory_ids.contains(target_memory_id) {
            return Err("merge target_memory_id 必须来自显式选中的 source_memory_ids".to_string());
        }
        let target_before = sources
            .iter()
            .find(|record| record.memory_id == *target_memory_id)
            .ok_or_else(|| "merge target_memory_id 不存在".to_string())?;
        require_status(
            target_before,
            &[MemoryLifecycleStatus::MemoryActive],
            "merge 目标正式记忆必须是 active",
        )?;
        let mut target_after = target_before.clone();
        target_after.claim = plan.merged_claim.trim().to_string();
        target_after.body = plan.merged_body.trim().to_string();
        target_after.memory_type = plan
            .memory_type
            .clone()
            .unwrap_or_else(|| target_before.memory_type.clone());
        target_after.scope = plan
            .scope
            .clone()
            .unwrap_or_else(|| target_before.scope.clone());
        target_after.source_refs = if plan.source_refs.is_empty() {
            combined_source_refs(sources.iter().flat_map(|record| record.source_refs.clone()))
        } else {
            plan.source_refs.clone()
        };
        target_after.supersedes_memory_id = Some(
            plan.source_memory_ids
                .iter()
                .filter(|memory_id| *memory_id != target_memory_id)
                .cloned()
                .collect::<Vec<_>>()
                .join(","),
        );
        target_after.status = MemoryLifecycleStatus::MemoryActive;
        target_after.record_version += 1;
        target_after.updated_at = timestamp.to_string();
        validate_memory_payload(&target_after)?;
        changes.push(PreparedRecordChange {
            before: Some(target_before.clone()),
            after: target_after,
            change_type: "merged_target_revision".to_string(),
            change_summary: "显式选择的正式记忆已合并为目标记录新版本；未做语义推断。".to_string(),
        });
    } else {
        let first = sources
            .first()
            .ok_or_else(|| "merge 缺少来源正式记忆".to_string())?;
        let new_record = MemoryRecord {
            memory_id: target_id.clone(),
            schema_version: "memory_governance.v1".to_string(),
            record_version: 1,
            scope: plan.scope.clone().unwrap_or_else(|| first.scope.clone()),
            memory_type: plan
                .memory_type
                .clone()
                .unwrap_or_else(|| first.memory_type.clone()),
            claim: plan.merged_claim.trim().to_string(),
            body: plan.merged_body.trim().to_string(),
            source_refs: if plan.source_refs.is_empty() {
                combined_source_refs(sources.iter().flat_map(|record| record.source_refs.clone()))
            } else {
                plan.source_refs.clone()
            },
            status: MemoryLifecycleStatus::MemoryActive,
            supersedes_memory_id: Some(source_id_text.clone()),
            superseded_by_memory_id: None,
            conflict_refs: vec![],
            audit_refs: vec![],
            created_at: timestamp.to_string(),
            updated_at: timestamp.to_string(),
        };
        validate_memory_payload(&new_record)?;
        changes.push(PreparedRecordChange {
            before: None,
            after: new_record,
            change_type: "merged_record_created".to_string(),
            change_summary: "显式选择的正式记忆已合并为新正式记忆；未做语义推断。".to_string(),
        });
    }

    for source in sources {
        if plan.target_memory_id.as_deref() == Some(source.memory_id.as_str()) {
            continue;
        }
        let mut after = source.clone();
        after.status = MemoryLifecycleStatus::MemoryDeprecated;
        after.superseded_by_memory_id = Some(target_id.clone());
        after.record_version += 1;
        after.updated_at = timestamp.to_string();
        changes.push(PreparedRecordChange {
            before: Some(source),
            after,
            change_type: "merged_source_deprecated".to_string(),
            change_summary: "显式 merge 后来源正式记忆转为 deprecated；旧版本和审计保留。"
                .to_string(),
        });
    }
    Ok(changes)
}

fn prepare_split(
    store: &FormalMemoryStoreV1,
    draft: &LifecycleDraft,
    timestamp: &str,
) -> Result<Vec<PreparedRecordChange>, String> {
    let plan = draft
        .split
        .as_ref()
        .ok_or_else(|| "split 操作缺少 split plan".to_string())?;
    if plan.split_records.len() < 2 {
        return Err("split 必须显式提供至少两条拆分后正式记忆草稿".to_string());
    }
    let source = find_record(store, &plan.source_memory_id)?;
    require_merge_source(source)?;
    let mut new_ids = Vec::new();
    let mut changes = Vec::new();
    for (index, draft_record) in plan.split_records.iter().enumerate() {
        validate_split_draft(draft_record)?;
        let memory_id = format!(
            "mem:v1:{timestamp}:{}",
            short_hash(&format!(
                "split:{}:{}:{}",
                store.revision + 1,
                source.memory_id,
                index
            ))
        );
        if store
            .records
            .iter()
            .any(|record| record.memory_id == memory_id)
        {
            return Err("split 生成的新 memory_id 已存在，已拒绝写入".to_string());
        }
        let new_record = MemoryRecord {
            memory_id: memory_id.clone(),
            schema_version: "memory_governance.v1".to_string(),
            record_version: 1,
            scope: draft_record
                .scope
                .clone()
                .unwrap_or_else(|| source.scope.clone()),
            memory_type: draft_record
                .memory_type
                .clone()
                .unwrap_or_else(|| source.memory_type.clone()),
            claim: draft_record.claim.trim().to_string(),
            body: draft_record.body.trim().to_string(),
            source_refs: if draft_record.source_refs.is_empty() {
                source.source_refs.clone()
            } else {
                draft_record.source_refs.clone()
            },
            status: MemoryLifecycleStatus::MemoryActive,
            supersedes_memory_id: Some(source.memory_id.clone()),
            superseded_by_memory_id: None,
            conflict_refs: vec![],
            audit_refs: vec![],
            created_at: timestamp.to_string(),
            updated_at: timestamp.to_string(),
        };
        validate_memory_payload(&new_record)?;
        new_ids.push(memory_id);
        changes.push(PreparedRecordChange {
            before: None,
            after: new_record,
            change_type: "split_record_created".to_string(),
            change_summary: "显式 split 生成新的正式记忆；来源记录不被物理移除。".to_string(),
        });
    }
    let mut source_after = source.clone();
    source_after.status = MemoryLifecycleStatus::MemoryDeprecated;
    source_after.superseded_by_memory_id = Some(new_ids.join(","));
    source_after.record_version += 1;
    source_after.updated_at = timestamp.to_string();
    changes.push(PreparedRecordChange {
        before: Some(source.clone()),
        after: source_after,
        change_type: "split_source_deprecated".to_string(),
        change_summary: "显式 split 后来源正式记忆转为 deprecated；旧版本和审计保留。".to_string(),
    });
    Ok(changes)
}

fn prepare_scope_change(
    store: &FormalMemoryStoreV1,
    draft: &LifecycleDraft,
    timestamp: &str,
    change_type: &str,
) -> Result<Vec<PreparedRecordChange>, String> {
    let memory_id = single_target_id(draft)?;
    let before = find_record(store, memory_id)?;
    require_status(
        before,
        &[MemoryLifecycleStatus::MemoryActive],
        "scope lifecycle 只处理 active 正式记忆",
    )?;
    let plan = draft
        .scope_change
        .as_ref()
        .ok_or_else(|| "scope lifecycle 操作缺少 scope_change plan".to_string())?;
    validate_scope_change_plan(draft.operation_kind, plan)?;
    let mut after = before.clone();
    after.scope = plan.target_scope.clone();
    after.record_version += 1;
    after.updated_at = timestamp.to_string();
    validate_memory_payload(&after)?;
    Ok(vec![PreparedRecordChange {
        before: Some(before.clone()),
        after,
        change_type: change_type.to_string(),
        change_summary: format!(
            "{}；{}",
            if draft.operation_kind == FormalMemoryLifecycleOperationKind::PromoteToGlobal {
                "正式记忆 scope 已上升为 global"
            } else {
                "正式记忆 scope 已下沉为 project"
            },
            plan.applicability.trim()
        ),
    }])
}

fn build_preview(
    store: &FormalMemoryStoreV1,
    draft: &LifecycleDraft,
    prepared: &PreparedLifecycle,
    timestamp: &str,
) -> FormalMemoryLifecyclePreview {
    let impact = impact_summary(prepared);
    let display_text = format!(
        "{} 预览：affected {} / new versions {} / approval {}",
        operation_name(draft.operation_kind),
        impact.affected_memory_ids.len(),
        impact.new_version_count,
        prepared.required_approval.approval_kind
    );
    FormalMemoryLifecyclePreview {
        preview_id: format!(
            "formal-memory-lifecycle-preview:v1:{timestamp}:{}",
            short_hash(&format!(
                "{}:{}:{}",
                store.revision,
                operation_name(draft.operation_kind),
                prepared.target_memory_ids.join(",")
            ))
        ),
        operation_kind: draft.operation_kind,
        store_revision: store.revision,
        target_memory_ids: prepared.target_memory_ids.clone(),
        impact,
        required_approval: prepared.required_approval.clone(),
        before_records: prepared.before_records(),
        proposed_records: prepared.proposed_records(),
        display_text,
        warnings: prepared.warnings.clone(),
    }
}

fn impact_summary(prepared: &PreparedLifecycle) -> FormalMemoryLifecycleImpactSummary {
    let mut affected = BTreeSet::new();
    let mut created = Vec::new();
    let mut source_ref_count = 0_usize;
    let mut status_changes = Vec::new();
    for change in &prepared.changes {
        affected.insert(change.after.memory_id.clone());
        source_ref_count += change.after.source_refs.len();
        if let Some(before) = &change.before {
            status_changes.push(FormalMemoryLifecycleStatusChange {
                memory_id: change.after.memory_id.clone(),
                before_status: before.status,
                after_status: change.after.status,
            });
        } else {
            created.push(change.after.memory_id.clone());
        }
    }
    let non_active_count = prepared
        .changes
        .iter()
        .filter(|change| change.after.status != MemoryLifecycleStatus::MemoryActive)
        .count();
    let task_packet_eligibility_change = if non_active_count > 0 {
        "非 active 或被替代的正式记忆默认不进任务包 included list。".to_string()
    } else {
        "仍按 active 状态、scope、lint 和外发策略评估任务包入选。".to_string()
    };
    FormalMemoryLifecycleImpactSummary {
        affected_memory_ids: affected.into_iter().collect(),
        created_memory_ids: created.clone(),
        status_changes,
        created_memory_count: created.len(),
        new_version_count: prepared.changes.len(),
        task_packet_eligibility_change: task_packet_eligibility_change.clone(),
        source_ref_count,
        display_text: format!(
            "影响 {} 条正式记忆，创建 {} 条新正式记忆，新增 {} 个版本；{}",
            prepared.changes.len(),
            created.len(),
            prepared.changes.len(),
            task_packet_eligibility_change
        ),
        warnings: prepared.warnings.clone(),
    }
}

fn validate_lifecycle_context_binding(
    workflow_state_path: &Path,
    draft: &LifecycleDraft,
) -> Result<(), String> {
    let project_root = draft.project_root.trim();
    if project_root.is_empty() {
        return Err("正式记忆 lifecycle 缺少 project_root".to_string());
    }
    let expected_project_id = crate::project_id(project_root);
    let expected_workflow_id = crate::default_workflow_id(project_root);
    validate_context_field(
        "project_id",
        draft.project_id.as_deref(),
        &expected_project_id,
    )?;
    validate_context_field(
        "workflow_id",
        draft.workflow_id.as_deref(),
        &expected_workflow_id,
    )?;
    crate::validate_formal_memory_project_registered(workflow_state_path, project_root)?;
    Ok(())
}

fn validate_context_for_changes(
    draft: &LifecycleDraft,
    changes: &[PreparedRecordChange],
) -> Result<(), String> {
    let expected_project_id = crate::project_id(draft.project_root.trim());
    let expected_workflow_id = crate::default_workflow_id(draft.project_root.trim());
    for change in changes {
        validate_record_scope_context(
            "proposed.scope",
            &change.after.scope,
            &expected_project_id,
            &expected_workflow_id,
        )?;
        if let Some(before) = &change.before {
            validate_record_scope_context(
                "current.scope",
                &before.scope,
                &expected_project_id,
                &expected_workflow_id,
            )?;
        }
    }
    Ok(())
}

fn validate_record_scope_context(
    label: &str,
    scope: &MemoryScope,
    expected_project_id: &str,
    expected_workflow_id: &str,
) -> Result<(), String> {
    if matches!(
        scope.scope_type.as_str(),
        "project" | "workflow" | "session"
    ) {
        validate_context_field(
            &format!("{label}.project_id"),
            scope.project_id.as_deref(),
            expected_project_id,
        )?;
    }
    if matches!(scope.scope_type.as_str(), "workflow" | "session") {
        validate_context_field(
            &format!("{label}.workflow_id"),
            scope.workflow_id.as_deref(),
            expected_workflow_id,
        )?;
    }
    Ok(())
}

fn validate_context_field(
    field_name: &str,
    actual: Option<&str>,
    expected: &str,
) -> Result<(), String> {
    if let Some(actual) = actual {
        let actual = actual.trim();
        if actual != expected {
            return Err(format!(
                "正式记忆 lifecycle 上下文绑定失败：{field_name} 与 project_root 不匹配，expected {expected}，actual {actual}"
            ));
        }
    }
    Ok(())
}

fn validate_actor_and_reason(draft: &LifecycleDraft) -> Result<(), String> {
    if draft.actor_id.trim().is_empty() {
        return Err("正式记忆 lifecycle 缺少 actor_id".to_string());
    }
    if !matches!(
        draft.actor_role.as_str(),
        "user" | "project_director" | "global_director"
    ) {
        return Err(
            "正式记忆 lifecycle 只能由 user / project_director / global_director 触发；秘书和系统不能批准"
                .to_string(),
        );
    }
    if draft.reason.trim().is_empty() {
        return Err("正式记忆 lifecycle 必须记录 reason".to_string());
    }
    Ok(())
}

fn validate_expected_revisions(
    draft: &LifecycleDraft,
    store: &FormalMemoryStoreV1,
) -> Result<(), String> {
    if let Some(expected) = draft.expected_store_revision {
        if expected != store.revision {
            return Err(format!(
                "formal_memory_lifecycle_conflict: expected revision {expected}, actual {}",
                store.revision
            ));
        }
    }
    for (memory_id, expected_version) in &draft.expected_record_versions {
        let record = find_record(store, memory_id)?;
        if record.record_version != *expected_version {
            return Err(format!(
                "formal_memory_record_version_conflict: memory_id {memory_id} expected v{expected_version}, actual v{}",
                record.record_version
            ));
        }
    }
    Ok(())
}

fn validate_required_approval(
    required_approval: &FormalMemoryRequiredApproval,
    confirmed_by: Option<&str>,
    confirmation_summary: Option<&str>,
) -> Result<(), String> {
    if !required_approval.required {
        return Ok(());
    }
    if confirmed_by.is_none_or(|value| value.trim().is_empty()) {
        return Err(format!(
            "正式记忆 lifecycle 需要确认权：{}",
            required_approval.reason
        ));
    }
    if confirmation_summary.is_none_or(|value| value.trim().is_empty()) {
        return Err("正式记忆 lifecycle 需要 confirmation_summary 记录确认理由".to_string());
    }
    if required_approval.required_actor_role == "user"
        && confirmed_by.is_none_or(|value| value.trim() != "user")
    {
        return Err(format!(
            "正式记忆 lifecycle 需要用户确认：{}",
            required_approval.reason
        ));
    }
    Ok(())
}

fn required_approval(
    operation_kind: FormalMemoryLifecycleOperationKind,
    changes: &[PreparedRecordChange],
) -> FormalMemoryRequiredApproval {
    let high_impact = operation_kind == FormalMemoryLifecycleOperationKind::PromoteToGlobal
        || operation_kind == FormalMemoryLifecycleOperationKind::DemoteToProject
        || changes.iter().any(|change| {
            high_impact_record(&change.after)
                || change.before.as_ref().is_some_and(high_impact_record)
        });
    if high_impact {
        return FormalMemoryRequiredApproval {
            required: true,
            approval_kind: "user_confirmation".to_string(),
            required_actor_role: "user".to_string(),
            reason: "用户偏好、全局蓝图、成熟模式、global scope 或跨项目影响必须有用户确认。"
                .to_string(),
        };
    }
    FormalMemoryRequiredApproval {
        required: true,
        approval_kind: "project_director_or_user_confirmation".to_string(),
        required_actor_role: "project_director_or_user".to_string(),
        reason: "项目内正式记忆 lifecycle 需要项目主管或用户确认。".to_string(),
    }
}

fn high_impact_record(record: &MemoryRecord) -> bool {
    matches!(
        record.memory_type.as_str(),
        "user_preference" | "global_blueprint" | "mature_pattern"
    ) || matches!(
        record.scope.scope_type.as_str(),
        "user_preference" | "global"
    ) || record
        .source_refs
        .iter()
        .any(|source| source.sensitive_level == "secret")
}

fn validate_memory_payload(record: &MemoryRecord) -> Result<(), String> {
    if record.claim.trim().is_empty() {
        return Err("正式记忆 lifecycle 后 claim 不能为空".to_string());
    }
    if record.body.trim().is_empty() {
        return Err("正式记忆 lifecycle 后 body 不能为空".to_string());
    }
    if record.source_refs.is_empty() {
        return Err("正式记忆 lifecycle 后必须保留来源".to_string());
    }
    if !matches!(
        record.memory_type.as_str(),
        "user_preference"
            | "global_blueprint"
            | "project_memory"
            | "workflow_summary"
            | "session_summary"
            | "mature_pattern"
    ) {
        return Err(format!("未知正式记忆类型：{}", record.memory_type));
    }
    if !matches!(
        record.scope.scope_type.as_str(),
        "user_preference"
            | "global"
            | "project"
            | "workflow"
            | "session"
            | "role_limited"
            | "document_limited"
    ) {
        return Err(format!(
            "未知正式记忆 scope_type：{}",
            record.scope.scope_type
        ));
    }
    if !matches!(
        record.scope.model_export_policy.as_str(),
        "local_only" | "allowed_with_redaction" | "blocked"
    ) {
        return Err(format!(
            "未知正式记忆 model_export_policy：{}",
            record.scope.model_export_policy
        ));
    }
    if record
        .source_refs
        .iter()
        .any(|source| source.sensitive_level == "secret")
        && record.scope.model_export_policy != "blocked"
    {
        return Err("secret 来源的正式记忆 lifecycle 后必须阻止外发模型上下文".to_string());
    }
    let text = format!("{} {}", record.claim, record.body).to_lowercase();
    if (text.contains("[secret]")
        || text.contains("sensitive:secret")
        || text.contains("token:")
        || text.contains("password:"))
        && record.scope.model_export_policy != "blocked"
    {
        return Err("敏感内容正式记忆 lifecycle 后必须阻止外发模型上下文".to_string());
    }
    Ok(())
}

fn validate_split_draft(draft: &FormalMemorySplitRecordDraft) -> Result<(), String> {
    if draft.claim.trim().is_empty() || draft.body.trim().is_empty() {
        return Err("split record draft 必须提供 claim/body".to_string());
    }
    Ok(())
}

fn validate_scope_change_plan(
    operation_kind: FormalMemoryLifecycleOperationKind,
    plan: &FormalMemoryScopeChangePlan,
) -> Result<(), String> {
    if plan.applicability.trim().is_empty() {
        return Err("scope lifecycle 必须记录 applicability".to_string());
    }
    match operation_kind {
        FormalMemoryLifecycleOperationKind::PromoteToGlobal
            if plan.target_scope.scope_type != "global" =>
        {
            Err("promote_to_global 的 target_scope.scope_type 必须是 global".to_string())
        }
        FormalMemoryLifecycleOperationKind::DemoteToProject
            if plan.target_scope.scope_type != "project" =>
        {
            Err("demote_to_project 的 target_scope.scope_type 必须是 project".to_string())
        }
        _ => Ok(()),
    }
}

fn apply_revise_plan(record: &mut MemoryRecord, plan: &FormalMemoryRevisePlan) {
    if let Some(claim) = &plan.claim {
        record.claim = claim.trim().to_string();
    }
    if let Some(body) = &plan.body {
        record.body = body.trim().to_string();
    }
    if let Some(source_refs) = &plan.source_refs {
        record.source_refs = source_refs.clone();
    }
}

fn require_merge_source(record: &MemoryRecord) -> Result<(), String> {
    if matches!(
        record.status,
        MemoryLifecycleStatus::MemoryArchived | MemoryLifecycleStatus::MemoryConflicted
    ) {
        return Err(format!(
            "{} 状态不允许作为 merge/split 来源",
            status_name(record.status)
        ));
    }
    Ok(())
}

fn require_status(
    record: &MemoryRecord,
    allowed: &[MemoryLifecycleStatus],
    message: &str,
) -> Result<(), String> {
    if allowed.contains(&record.status) {
        Ok(())
    } else {
        Err(message.to_string())
    }
}

fn single_target_id(draft: &LifecycleDraft) -> Result<&str, String> {
    let ids = draft.single_or_many_ids();
    if ids.len() != 1 {
        return Err(format!(
            "{} 操作必须且只能指定一条正式记忆",
            operation_name(draft.operation_kind)
        ));
    }
    draft
        .memory_id
        .as_deref()
        .or_else(|| draft.memory_ids.first().map(String::as_str))
        .ok_or_else(|| "正式记忆 lifecycle 操作缺少 memory_id".to_string())
}

fn find_record<'a>(
    store: &'a FormalMemoryStoreV1,
    memory_id: &str,
) -> Result<&'a MemoryRecord, String> {
    store
        .records
        .iter()
        .find(|record| record.memory_id == memory_id)
        .ok_or_else(|| format!("正式记忆不存在：{memory_id}"))
}

fn upsert_record(records: &mut Vec<MemoryRecord>, record: MemoryRecord) -> Result<(), String> {
    if let Some(existing) = records
        .iter_mut()
        .find(|existing| existing.memory_id == record.memory_id)
    {
        *existing = record;
        return Ok(());
    }
    records.push(record);
    Ok(())
}

fn ensure_unique_ids(ids: &[String]) -> Result<(), String> {
    let mut seen = BTreeSet::new();
    for id in ids {
        if id.trim().is_empty() {
            return Err("正式记忆 lifecycle memory_id 不能为空".to_string());
        }
        if !seen.insert(id) {
            return Err(format!("正式记忆 lifecycle 重复选择 memory_id：{id}"));
        }
    }
    Ok(())
}

fn combined_source_refs<I>(refs: I) -> Vec<MemorySourceRef>
where
    I: IntoIterator<Item = MemorySourceRef>,
{
    let mut seen = BTreeSet::new();
    let mut result = Vec::new();
    for source in refs {
        let key = format!(
            "{}:{}:{}",
            source.source_ref_id,
            source.source_type,
            source.source_id.clone().unwrap_or_default()
        );
        if seen.insert(key) {
            result.push(source);
        }
    }
    result
}

fn confirmation_reason(draft: &LifecycleDraft, confirmation_summary: Option<&str>) -> String {
    match confirmation_summary {
        Some(summary) if !summary.trim().is_empty() => {
            format!("{}；确认：{}", draft.reason.trim(), summary.trim())
        }
        _ => draft.reason.trim().to_string(),
    }
}

fn lifecycle_warnings(operation_kind: FormalMemoryLifecycleOperationKind) -> Vec<String> {
    let mut warnings = vec![
        "formal_memory_lifecycle_versions_and_audit_recorded".to_string(),
        "formal_memory_lifecycle_no_physical_delete".to_string(),
    ];
    if matches!(
        operation_kind,
        FormalMemoryLifecycleOperationKind::Merge | FormalMemoryLifecycleOperationKind::Split
    ) {
        warnings
            .push("formal_memory_lifecycle_explicit_selection_only_no_semantic_dedupe".to_string());
    }
    warnings
}

fn operation_name(operation_kind: FormalMemoryLifecycleOperationKind) -> &'static str {
    match operation_kind {
        FormalMemoryLifecycleOperationKind::Revise => "revise",
        FormalMemoryLifecycleOperationKind::Deprecate => "deprecate",
        FormalMemoryLifecycleOperationKind::Freeze => "freeze",
        FormalMemoryLifecycleOperationKind::Unfreeze => "unfreeze",
        FormalMemoryLifecycleOperationKind::Archive => "archive",
        FormalMemoryLifecycleOperationKind::Merge => "merge",
        FormalMemoryLifecycleOperationKind::Split => "split",
        FormalMemoryLifecycleOperationKind::PromoteToGlobal => "promote_to_global",
        FormalMemoryLifecycleOperationKind::DemoteToProject => "demote_to_project",
    }
}

fn status_name(status: MemoryLifecycleStatus) -> &'static str {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CreateFormalMemoryRecordInput, ProjectRecord};
    use std::path::PathBuf;

    #[test]
    fn formal_memory_lifecycle_revise_creates_new_version_and_audit() {
        let (dir, path, project_root, record) = setup_lifecycle_fixture("revise");
        let input = lifecycle_input(
            project_root,
            FormalMemoryLifecycleOperationKind::Revise,
            Some(record.memory_id.clone()),
        )
        .with_revise("接口验收边界已更新", "编辑会创建新版本，不覆盖旧版本。")
        .with_expected_revision(1)
        .with_expected_record_version(&record.memory_id, 1);

        let output = record_operation(&path, &input, "2026-06-05T00:00:01Z", "write-revise")
            .expect("revise should write lifecycle operation");

        assert_eq!(output.store_revision, 2);
        assert_eq!(output.records[0].record_version, 2);
        assert_eq!(output.records[0].claim, "接口验收边界已更新");
        assert_eq!(output.versions[0].change_type, "manual_revision");
        assert_eq!(
            output.audit_event.event_type,
            "formal_memory_revise_recorded"
        );
        let store = crate::formal_memory_store::load_store(&path, "2026-06-05T00:00:02Z")
            .expect("store should load");
        assert_eq!(store.records.len(), 1);
        assert_eq!(store.versions.len(), 2);
        assert_eq!(store.audit_events.len(), 2);

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn formal_memory_lifecycle_freeze_blocks_revise_until_unfreeze() {
        let (dir, path, project_root, record) = setup_lifecycle_fixture("freeze");
        let freeze = lifecycle_input(
            project_root,
            FormalMemoryLifecycleOperationKind::Freeze,
            Some(record.memory_id.clone()),
        )
        .with_expected_revision(1);
        record_operation(&path, &freeze, "2026-06-05T00:01:01Z", "write-freeze")
            .expect("freeze should write");

        let revise = lifecycle_input(
            project_root,
            FormalMemoryLifecycleOperationKind::Revise,
            Some(record.memory_id.clone()),
        )
        .with_revise("冻结后编辑", "该编辑应被阻断。")
        .with_expected_revision(2);
        let err =
            record_operation(&path, &revise, "2026-06-05T00:01:02Z", "write-revise").unwrap_err();
        assert!(err.contains("冻结后需先解冻"));

        let unfreeze = lifecycle_input(
            project_root,
            FormalMemoryLifecycleOperationKind::Unfreeze,
            Some(record.memory_id.clone()),
        )
        .with_expected_revision(2);
        let output = record_operation(&path, &unfreeze, "2026-06-05T00:01:03Z", "write-unfreeze")
            .expect("unfreeze should write");
        assert_eq!(
            output.records[0].status,
            MemoryLifecycleStatus::MemoryActive
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn formal_memory_lifecycle_deprecate_and_archive_exclude_from_task_packet() {
        let (dir, path, project_root, record) = setup_lifecycle_fixture("deprecate");
        let deprecate = lifecycle_input(
            project_root,
            FormalMemoryLifecycleOperationKind::Deprecate,
            Some(record.memory_id.clone()),
        )
        .with_expected_revision(1);
        let output = record_operation(&path, &deprecate, "2026-06-05T00:02:01Z", "write-deprecate")
            .expect("deprecate should write");
        assert_eq!(
            output.records[0].status,
            MemoryLifecycleStatus::MemoryDeprecated
        );
        assert!(output
            .preview
            .impact
            .task_packet_eligibility_change
            .contains("默认不进任务包"));

        let archive = lifecycle_input(
            project_root,
            FormalMemoryLifecycleOperationKind::Archive,
            Some(record.memory_id.clone()),
        )
        .with_expected_revision(2);
        let output = record_operation(&path, &archive, "2026-06-05T00:02:02Z", "write-archive")
            .expect("archive should write");
        assert_eq!(
            output.records[0].status,
            MemoryLifecycleStatus::MemoryArchived
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn formal_memory_lifecycle_merge_is_explicit_and_versions_sources() {
        let (dir, path, project_root, first) = setup_lifecycle_fixture("merge");
        let second = create_record(
            &path,
            project_root,
            "接口验收必须保留来源",
            "第二条显式选择的正式记忆。",
            "2026-06-05T00:03:01Z",
            "write-merge-second",
        );
        let input = lifecycle_input(
            project_root,
            FormalMemoryLifecycleOperationKind::Merge,
            None,
        )
        .with_merge(vec![first.memory_id.clone(), second.memory_id.clone()])
        .with_expected_revision(2);

        let output = record_operation(&path, &input, "2026-06-05T00:03:02Z", "write-merge")
            .expect("merge should write");

        assert_eq!(output.preview.impact.created_memory_count, 1);
        assert_eq!(output.versions.len(), 3);
        assert!(output
            .records
            .iter()
            .any(|record| record.status == MemoryLifecycleStatus::MemoryActive));
        assert_eq!(
            output
                .records
                .iter()
                .filter(|record| record.status == MemoryLifecycleStatus::MemoryDeprecated)
                .count(),
            2
        );
        assert!(output.warnings.contains(
            &"formal_memory_lifecycle_explicit_selection_only_no_semantic_dedupe".to_string()
        ));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn formal_memory_lifecycle_split_keeps_source_and_creates_children() {
        let (dir, path, project_root, record) = setup_lifecycle_fixture("split");
        let input = lifecycle_input(
            project_root,
            FormalMemoryLifecycleOperationKind::Split,
            Some(record.memory_id.clone()),
        )
        .with_split(record.memory_id.clone())
        .with_expected_revision(1);

        let output = record_operation(&path, &input, "2026-06-05T00:04:01Z", "write-split")
            .expect("split should write");

        assert_eq!(output.preview.impact.created_memory_count, 2);
        assert_eq!(output.versions.len(), 3);
        assert!(output
            .records
            .iter()
            .any(|item| item.memory_id == record.memory_id
                && item.status == MemoryLifecycleStatus::MemoryDeprecated));

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn formal_memory_lifecycle_promote_and_demote_require_confirmation() {
        let (dir, path, project_root, record) = setup_lifecycle_fixture("scope");
        let promote_without_confirmation = lifecycle_input(
            project_root,
            FormalMemoryLifecycleOperationKind::PromoteToGlobal,
            Some(record.memory_id.clone()),
        )
        .with_scope("global")
        .without_confirmation()
        .with_expected_revision(1);

        let err = record_operation(
            &path,
            &promote_without_confirmation,
            "2026-06-05T00:05:01Z",
            "write-promote-missing-confirm",
        )
        .unwrap_err();
        assert!(err.contains("需要确认权"));

        let promote_without_user_confirmation = lifecycle_input(
            project_root,
            FormalMemoryLifecycleOperationKind::PromoteToGlobal,
            Some(record.memory_id.clone()),
        )
        .with_scope("global")
        .with_expected_revision(1);
        let err = record_operation(
            &path,
            &promote_without_user_confirmation,
            "2026-06-05T00:05:02Z",
            "write-promote-project-director-confirm",
        )
        .unwrap_err();
        assert!(err.contains("需要用户确认"));

        let promote = lifecycle_input(
            project_root,
            FormalMemoryLifecycleOperationKind::PromoteToGlobal,
            Some(record.memory_id.clone()),
        )
        .with_scope("global")
        .with_user_confirmation()
        .with_expected_revision(1);
        let output = record_operation(&path, &promote, "2026-06-05T00:05:03Z", "write-promote")
            .expect("promote should write");
        assert_eq!(output.records[0].scope.scope_type, "global");

        let demote = lifecycle_input(
            project_root,
            FormalMemoryLifecycleOperationKind::DemoteToProject,
            Some(record.memory_id.clone()),
        )
        .with_scope("project")
        .with_user_confirmation()
        .with_expected_revision(2);
        let output = record_operation(&path, &demote, "2026-06-05T00:05:04Z", "write-demote")
            .expect("demote should write");
        assert_eq!(output.records[0].scope.scope_type, "project");

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn formal_memory_lifecycle_rejects_revision_conflict_and_damaged_json() {
        let (dir, path, project_root, record) = setup_lifecycle_fixture("conflict");
        let stale = lifecycle_input(
            project_root,
            FormalMemoryLifecycleOperationKind::Freeze,
            Some(record.memory_id.clone()),
        )
        .with_expected_revision(0);
        let err =
            record_operation(&path, &stale, "2026-06-05T00:06:01Z", "write-stale").unwrap_err();
        assert!(err.contains("formal_memory_lifecycle_conflict"));

        let formal_path = crate::formal_memory_store::sidecar_path(&path).expect("sidecar path");
        fs::write(&formal_path, "{not valid json").expect("damaged sidecar should write");
        let err = preview_operation(
            &path,
            &FormalMemoryLifecyclePreviewInput {
                project_root: project_root.to_string(),
                project_id: Some(crate::project_id(project_root)),
                workflow_id: Some(crate::default_workflow_id(project_root)),
                operation_kind: FormalMemoryLifecycleOperationKind::Archive,
                memory_id: Some(record.memory_id),
                memory_ids: vec![],
                revise: None,
                merge: None,
                split: None,
                scope_change: None,
                actor_id: "project-director-offline".to_string(),
                actor_role: "project_director".to_string(),
                reason: "损坏 JSON 不可覆盖。".to_string(),
                expected_store_revision: None,
                expected_record_versions: BTreeMap::new(),
            },
            "2026-06-05T00:06:02Z",
        )
        .unwrap_err();
        assert!(err.contains("正式记忆 sidecar JSON 损坏"));
        assert_eq!(
            fs::read_to_string(&formal_path).expect("sidecar should remain"),
            "{not valid json"
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[derive(Clone)]
    struct TestLifecycleInput(FormalMemoryLifecycleInput);

    impl TestLifecycleInput {
        fn with_expected_revision(mut self, revision: i64) -> Self {
            self.0.expected_store_revision = Some(revision);
            self
        }

        fn with_expected_record_version(mut self, memory_id: &str, version: i64) -> Self {
            self.0
                .expected_record_versions
                .insert(memory_id.to_string(), version);
            self
        }

        fn with_revise(mut self, claim: &str, body: &str) -> Self {
            self.0.revise = Some(FormalMemoryRevisePlan {
                claim: Some(claim.to_string()),
                body: Some(body.to_string()),
                source_refs: None,
            });
            self
        }

        fn with_merge(mut self, ids: Vec<String>) -> Self {
            self.0.memory_ids = ids.clone();
            self.0.merge = Some(FormalMemoryMergePlan {
                source_memory_ids: ids,
                target_memory_id: None,
                merged_claim: "接口验收合并后的正式记忆".to_string(),
                merged_body: "该记录来自显式选择的正式记忆，不包含语义推断。".to_string(),
                memory_type: None,
                scope: None,
                source_refs: vec![],
            });
            self
        }

        fn with_split(mut self, source_id: String) -> Self {
            self.0.split = Some(FormalMemorySplitPlan {
                source_memory_id: source_id,
                split_records: vec![
                    FormalMemorySplitRecordDraft {
                        claim: "拆分后正式记忆 A".to_string(),
                        body: "A 的明确正文。".to_string(),
                        memory_type: None,
                        scope: None,
                        source_refs: vec![],
                    },
                    FormalMemorySplitRecordDraft {
                        claim: "拆分后正式记忆 B".to_string(),
                        body: "B 的明确正文。".to_string(),
                        memory_type: None,
                        scope: None,
                        source_refs: vec![],
                    },
                ],
            });
            self
        }

        fn with_scope(mut self, scope_type: &str) -> Self {
            let project_root = self.0.project_root.clone();
            self.0.scope_change = Some(FormalMemoryScopeChangePlan {
                target_scope: if scope_type == "global" {
                    MemoryScope {
                        scope_id: "scope:global:test".to_string(),
                        scope_type: "global".to_string(),
                        user_id: None,
                        project_id: None,
                        workflow_id: None,
                        session_id: None,
                        role_ids: vec![],
                        document_refs: vec![],
                        permission_policy_ref: None,
                        model_export_policy: "local_only".to_string(),
                        valid_from: "2026-06-05T00:00:00Z".to_string(),
                        valid_until: None,
                    }
                } else {
                    MemoryScope {
                        scope_id: "scope:project:test".to_string(),
                        scope_type: "project".to_string(),
                        user_id: None,
                        project_id: Some(crate::project_id(&project_root)),
                        workflow_id: None,
                        session_id: None,
                        role_ids: vec![],
                        document_refs: vec![],
                        permission_policy_ref: None,
                        model_export_policy: "local_only".to_string(),
                        valid_from: "2026-06-05T00:00:00Z".to_string(),
                        valid_until: None,
                    }
                },
                applicability: format!("适用于 {scope_type} lifecycle 测试。"),
            });
            self
        }

        fn without_confirmation(mut self) -> Self {
            self.0.confirmed_by = None;
            self.0.confirmation_summary = None;
            self
        }

        fn with_user_confirmation(mut self) -> Self {
            self.0.confirmed_by = Some("user".to_string());
            self.0.confirmation_summary = Some("用户已确认高影响 lifecycle 操作。".to_string());
            self
        }
    }

    impl std::ops::Deref for TestLifecycleInput {
        type Target = FormalMemoryLifecycleInput;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    fn lifecycle_input(
        project_root: &str,
        operation_kind: FormalMemoryLifecycleOperationKind,
        memory_id: Option<String>,
    ) -> TestLifecycleInput {
        TestLifecycleInput(FormalMemoryLifecycleInput {
            project_root: project_root.to_string(),
            project_id: Some(crate::project_id(project_root)),
            workflow_id: Some(crate::default_workflow_id(project_root)),
            operation_kind,
            memory_id,
            memory_ids: vec![],
            revise: None,
            merge: None,
            split: None,
            scope_change: None,
            actor_id: "project-director-offline".to_string(),
            actor_role: "project_director".to_string(),
            reason: "M9 lifecycle 测试确认。".to_string(),
            confirmed_by: Some("project-director-offline".to_string()),
            confirmation_summary: Some("离线测试确认 lifecycle 操作。".to_string()),
            expected_store_revision: None,
            expected_record_versions: BTreeMap::new(),
        })
    }

    fn setup_lifecycle_fixture(prefix: &str) -> (PathBuf, PathBuf, &'static str, MemoryRecord) {
        let dir = std::env::temp_dir().join(format!(
            "formal-memory-lifecycle-{prefix}-{}",
            crate::unix_timestamp_nanos()
        ));
        fs::create_dir_all(&dir).expect("fixture dir should exist");
        let path = dir.join("workflow-state.v0.json");
        let project_root = "/tmp/formal-memory-lifecycle-project";
        crate::bootstrap_project_workflow_at(&path, &fixture_project(project_root))
            .expect("workflow state should include project");
        let record = create_record(
            &path,
            project_root,
            "接口验收必须保留控制核心边界",
            "正式记忆 lifecycle fixture。",
            "2026-06-05T00:00:00Z",
            "write-lifecycle-fixture",
        );
        (dir, path, project_root, record)
    }

    fn create_record(
        path: &Path,
        project_root: &str,
        claim: &str,
        body: &str,
        timestamp: &str,
        write_id: &str,
    ) -> MemoryRecord {
        let input = CreateFormalMemoryRecordInput {
            project_root: project_root.to_string(),
            project_id: Some(crate::project_id(project_root)),
            workflow_id: Some(crate::default_workflow_id(project_root)),
            scope: MemoryScope {
                scope_id: "scope:project:lifecycle".to_string(),
                scope_type: "project".to_string(),
                user_id: None,
                project_id: Some(crate::project_id(project_root)),
                workflow_id: None,
                session_id: None,
                role_ids: vec![],
                document_refs: vec![],
                permission_policy_ref: None,
                model_export_policy: "local_only".to_string(),
                valid_from: "2026-06-05T00:00:00Z".to_string(),
                valid_until: None,
            },
            memory_type: "project_memory".to_string(),
            claim: claim.to_string(),
            body: body.to_string(),
            source_refs: vec![MemorySourceRef {
                source_ref_id: format!("source:{write_id}"),
                source_type: "evidence".to_string(),
                source_id: Some(write_id.to_string()),
                source_path: Some("evidence/lifecycle.md".to_string()),
                source_title: Some("M9 lifecycle fixture".to_string()),
                anchor: None,
                source_created_at: None,
                captured_at: timestamp.to_string(),
                authority_level: "evidence".to_string(),
                sensitive_level: "project".to_string(),
                content_hash: None,
            }],
            actor_id: "project-director-offline".to_string(),
            actor_role: "project_director".to_string(),
            reason: "创建 lifecycle 测试正式记忆。".to_string(),
            audit_event_type: None,
            expected_store_revision: None,
        };
        crate::create_formal_memory_record_at(path, &input, timestamp, write_id)
            .expect("formal memory should be created")
            .record
    }

    fn fixture_project(project_root: &str) -> ProjectRecord {
        ProjectRecord {
            project_root: project_root.to_string(),
            name: "Lifecycle Project".to_string(),
            active_hint: true,
            thread_count: 0,
            active_thread_count: 0,
            archived_thread_count: 0,
            latest_updated_at_ms: None,
            authority_files: vec![],
            handoff_files: vec![],
            evidence_files: vec![],
            harness_candidates: vec![],
            harness_resources: vec![],
            context_warnings: vec![],
            warnings: vec![],
        }
    }
}
