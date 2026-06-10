use crate::{
    FormalMemoryStoreV1, MemoryCandidate, MemoryCandidateStoreV1, MemoryEntityRelationStoreV1,
    MemoryLifecycleStatus, MemoryLintFinding, MemoryLintFindingSeverity, MemoryLintFindingStatus,
    MemoryLintFindingType, MemoryLintRunInput, MemoryLintRunIntent, MemoryLintStoreV1,
    MemoryMaintenanceCheckKind, MemoryMaintenanceCheckSummary, MemoryMaintenanceIndexStatus,
    MemoryMaintenanceRecommendation, MemoryMaintenanceReport, MemoryRecord, MemoryRelationSource,
    MemoryRelationStatus, MemorySourceRef, ObservationStoreV1,
};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

const SCHEMA_VERSION: &str = "memory_governance.v1";
pub(crate) const CLAIM_SIMILARITY_THRESHOLD: f32 = 0.80;

pub(crate) fn build_findings(
    input: &MemoryLintRunInput,
    formal_store: &FormalMemoryStoreV1,
    candidate_store: &MemoryCandidateStoreV1,
    observation_store: &ObservationStoreV1,
    entity_relation_store: &MemoryEntityRelationStoreV1,
    timestamp: &str,
) -> Result<Vec<MemoryLintFinding>, String> {
    let mut findings = Vec::new();
    append_duplicate_claim_findings(formal_store, timestamp, &mut findings);
    append_authority_superseded_findings(formal_store, timestamp, &mut findings);
    append_revoked_source_findings(input, formal_store, timestamp, &mut findings);
    append_relation_source_revoked_findings(input, entity_relation_store, timestamp, &mut findings);
    append_candidate_conflict_findings(
        input,
        formal_store,
        candidate_store,
        timestamp,
        &mut findings,
    )?;
    if should_run_maintenance_checks(input.lint_intent) {
        append_stale_memory_findings(formal_store, timestamp, &mut findings);
        append_missing_source_findings(formal_store, timestamp, &mut findings);
        append_sensitive_source_findings(formal_store, timestamp, &mut findings);
        append_entity_drift_findings(entity_relation_store, timestamp, &mut findings);
        append_index_status_findings(
            formal_store,
            entity_relation_store,
            timestamp,
            &mut findings,
        );
        append_mature_pattern_signal_findings(
            candidate_store,
            observation_store,
            timestamp,
            &mut findings,
        );
    }
    Ok(findings)
}

pub(crate) fn build_maintenance_report(
    _input: &MemoryLintRunInput,
    formal_store: &FormalMemoryStoreV1,
    candidate_store: &MemoryCandidateStoreV1,
    observation_store: &ObservationStoreV1,
    entity_relation_store: &MemoryEntityRelationStoreV1,
    lint_store_revision: i64,
    existing_findings: &[MemoryLintFinding],
    new_findings: &[MemoryLintFinding],
    run_id: &str,
    timestamp: &str,
) -> Result<MemoryMaintenanceReport, String> {
    let open_findings = existing_findings
        .iter()
        .chain(new_findings.iter())
        .filter(|finding| finding.status == MemoryLintFindingStatus::Open)
        .collect::<Vec<_>>();
    let blocking_count = open_findings
        .iter()
        .filter(|finding| finding.severity == MemoryLintFindingSeverity::Blocking)
        .count();
    let needs_review_count = open_findings
        .iter()
        .filter(|finding| finding.severity == MemoryLintFindingSeverity::NeedsReview)
        .count();
    let info_count = open_findings
        .iter()
        .filter(|finding| finding.severity == MemoryLintFindingSeverity::Info)
        .count();
    let index_status = build_index_status(
        formal_store,
        entity_relation_store,
        lint_store_revision,
        timestamp,
    );
    let check_summaries = build_check_summaries(&open_findings, formal_store, candidate_store);
    let recommendations = open_findings
        .iter()
        .filter(|finding| finding.severity != MemoryLintFindingSeverity::Info)
        .take(8)
        .map(|finding| maintenance_recommendation(finding))
        .collect::<Vec<_>>();
    let report_id = format!(
        "memory-maintenance-report:v1:{}",
        short_hash(&format!(
            "{}:{}:{}:{}",
            run_id,
            formal_store.revision,
            candidate_store.revision,
            open_findings.len()
        ))
    );
    Ok(MemoryMaintenanceReport {
        report_id,
        run_id: run_id.to_string(),
        checked_memory_count: formal_store.records.len(),
        checked_candidate_count: candidate_store.candidates.len(),
        checked_observation_count: observation_store.observations.len(),
        checked_relation_count: entity_relation_store.relations.len()
            + entity_relation_store.relation_candidates.len(),
        open_count: open_findings.len(),
        blocking_count,
        needs_review_count,
        info_count,
        check_summaries,
        recommendations,
        index_status,
        display_text: format!(
            "维护任务摘要：检查正式记忆 {} / 候选 {} / observation {} / relation {}；open {} / blocking {} / needs_review {} / info {}。维护任务只生成 finding，不会自动修改正式记忆。",
            formal_store.records.len(),
            candidate_store.candidates.len(),
            observation_store.observations.len(),
            entity_relation_store.relations.len() + entity_relation_store.relation_candidates.len(),
            open_findings.len(),
            blocking_count,
            needs_review_count,
            info_count
        ),
        warnings: vec![
            "memory_maintenance_findings_only".to_string(),
            "memory_maintenance_does_not_call_lifecycle".to_string(),
            "blocking_finding_blocks_task_packet_recall".to_string(),
        ],
        created_at: timestamp.to_string(),
    })
}

pub(crate) fn open_blocking_findings_for_memory<'a>(
    store: &'a MemoryLintStoreV1,
    memory_id: &str,
) -> Vec<&'a MemoryLintFinding> {
    store
        .findings
        .iter()
        .filter(|finding| {
            is_open_blocking(finding)
                && (finding.target_memory_id.as_deref() == Some(memory_id)
                    || (finding.source_kind == "memory_record" && finding.source_id == memory_id))
        })
        .collect()
}

pub(crate) fn is_open_blocking(finding: &MemoryLintFinding) -> bool {
    finding.status == MemoryLintFindingStatus::Open
        && finding.severity == MemoryLintFindingSeverity::Blocking
}

fn append_duplicate_claim_findings(
    store: &FormalMemoryStoreV1,
    timestamp: &str,
    findings: &mut Vec<MemoryLintFinding>,
) {
    for left_index in 0..store.records.len() {
        for right in store.records.iter().skip(left_index + 1) {
            let left = &store.records[left_index];
            if !same_scope_type_and_memory_type(left, right) {
                continue;
            }
            let exact = normalize(&left.claim) == normalize(&right.claim);
            let similarity = claim_similarity(&left.claim, &right.claim);
            if exact || similarity >= CLAIM_SIMILARITY_THRESHOLD {
                findings.push(MemoryLintFinding {
                    finding_id: finding_id(
                        MemoryLintFindingType::DuplicateClaim,
                        &right.memory_id,
                        Some(&left.memory_id),
                    ),
                    schema_version: SCHEMA_VERSION.to_string(),
                    finding_type: MemoryLintFindingType::DuplicateClaim,
                    severity: MemoryLintFindingSeverity::NeedsReview,
                    status: MemoryLintFindingStatus::Open,
                    source_kind: "memory_record".to_string(),
                    source_id: right.memory_id.clone(),
                    target_memory_id: Some(left.memory_id.clone()),
                    target_candidate_key: None,
                    scope_type: Some(right.scope.scope_type.clone()),
                    memory_type: Some(right.memory_type.clone()),
                    claim: Some(right.claim.clone()),
                    summary: if exact {
                        "正式记忆 claim 完全重复，需要人工确认是否废弃或合并".to_string()
                    } else {
                        format!(
                            "正式记忆 claim 相似度 {:.2} 达到阈值 {:.2}，需要人工复核",
                            similarity, CLAIM_SIMILARITY_THRESHOLD
                        )
                    },
                    recommended_action: "review_and_deprecate".to_string(),
                    evidence_refs: combined_source_refs(left, right),
                    audit_event_id: None,
                    created_at: timestamp.to_string(),
                    updated_at: timestamp.to_string(),
                });
            }
        }
    }
}

fn append_authority_superseded_findings(
    store: &FormalMemoryStoreV1,
    timestamp: &str,
    findings: &mut Vec<MemoryLintFinding>,
) {
    for old in &store.records {
        if old.memory_type != "user_preference" && old.memory_type != "project_memory" {
            continue;
        }
        for newer in &store.records {
            if old.memory_id == newer.memory_id
                || !same_scope_type_and_memory_type(old, newer)
                || newer.created_at <= old.created_at
                || !has_stronger_authority(&newer.source_refs)
                || claim_similarity(&old.claim, &newer.claim) < CLAIM_SIMILARITY_THRESHOLD
            {
                continue;
            }
            findings.push(MemoryLintFinding {
                finding_id: finding_id(
                    MemoryLintFindingType::AuthoritySuperseded,
                    &old.memory_id,
                    Some(&newer.memory_id),
                ),
                schema_version: SCHEMA_VERSION.to_string(),
                finding_type: MemoryLintFindingType::AuthoritySuperseded,
                severity: MemoryLintFindingSeverity::NeedsReview,
                status: MemoryLintFindingStatus::Open,
                source_kind: "memory_record".to_string(),
                source_id: old.memory_id.clone(),
                target_memory_id: Some(old.memory_id.clone()),
                target_candidate_key: None,
                scope_type: Some(old.scope.scope_type.clone()),
                memory_type: Some(old.memory_type.clone()),
                claim: Some(old.claim.clone()),
                summary: format!(
                    "较新的高权威来源 {} 可能覆盖旧记忆，但 M5 不会自动修改正式记忆状态",
                    newer.memory_id
                ),
                recommended_action: "review_staleness".to_string(),
                evidence_refs: combined_source_refs(old, newer),
                audit_event_id: None,
                created_at: timestamp.to_string(),
                updated_at: timestamp.to_string(),
            });
        }
    }
}

fn append_revoked_source_findings(
    input: &MemoryLintRunInput,
    store: &FormalMemoryStoreV1,
    timestamp: &str,
    findings: &mut Vec<MemoryLintFinding>,
) {
    if input.revoked_source_ids.is_empty() {
        return;
    }
    let revoked = input
        .revoked_source_ids
        .iter()
        .map(|item| normalize(item))
        .collect::<BTreeSet<_>>();
    for record in &store.records {
        if !record.source_refs.iter().any(|source| {
            source
                .source_id
                .as_deref()
                .is_some_and(|source_id| revoked.contains(&normalize(source_id)))
        }) {
            continue;
        }
        findings.push(MemoryLintFinding {
            finding_id: finding_id(
                MemoryLintFindingType::SourcePermissionRevoked,
                &record.memory_id,
                None,
            ),
            schema_version: SCHEMA_VERSION.to_string(),
            finding_type: MemoryLintFindingType::SourcePermissionRevoked,
            severity: MemoryLintFindingSeverity::Blocking,
            status: MemoryLintFindingStatus::Open,
            source_kind: "memory_record".to_string(),
            source_id: record.memory_id.clone(),
            target_memory_id: Some(record.memory_id.clone()),
            target_candidate_key: None,
            scope_type: Some(record.scope.scope_type.clone()),
            memory_type: Some(record.memory_type.clone()),
            claim: Some(record.claim.clone()),
            summary: "正式记忆来源权限已撤回；任务记忆包预览必须阻断".to_string(),
            recommended_action: "review_source_permission".to_string(),
            evidence_refs: record.source_refs.clone(),
            audit_event_id: None,
            created_at: timestamp.to_string(),
            updated_at: timestamp.to_string(),
        });
    }
}

fn append_candidate_conflict_findings(
    input: &MemoryLintRunInput,
    formal_store: &FormalMemoryStoreV1,
    candidate_store: &MemoryCandidateStoreV1,
    timestamp: &str,
    findings: &mut Vec<MemoryLintFinding>,
) -> Result<(), String> {
    let Some(candidate_key) = input.candidate_key.as_deref() else {
        return Ok(());
    };
    let candidate = candidate_store
        .candidates
        .iter()
        .find(|candidate| candidate.candidate_key == candidate_key)
        .ok_or_else(|| "memory_lint_candidate_missing: 未找到待检查候选".to_string())?;
    for record in &formal_store.records {
        if record.status != MemoryLifecycleStatus::MemoryActive
            || !same_candidate_scope_and_type(candidate, record)
            || !has_exclusive_terms(&candidate.claim, &record.claim)
        {
            continue;
        }
        findings.push(MemoryLintFinding {
            finding_id: finding_id(
                MemoryLintFindingType::CandidateConflictsWithActiveMemory,
                &candidate.candidate_key,
                Some(&record.memory_id),
            ),
            schema_version: SCHEMA_VERSION.to_string(),
            finding_type: MemoryLintFindingType::CandidateConflictsWithActiveMemory,
            severity: MemoryLintFindingSeverity::Blocking,
            status: MemoryLintFindingStatus::Open,
            source_kind: "memory_candidate".to_string(),
            source_id: candidate.candidate_key.clone(),
            target_memory_id: Some(record.memory_id.clone()),
            target_candidate_key: Some(candidate.candidate_key.clone()),
            scope_type: Some(candidate.scope.scope_type.clone()),
            memory_type: Some(candidate.memory_type.clone()),
            claim: Some(candidate.claim.clone()),
            summary: format!(
                "候选与 active 正式记忆 {} 命中确定性互斥词，已阻断采纳",
                record.memory_id
            ),
            recommended_action: "block_adoption".to_string(),
            evidence_refs: candidate.source_refs.clone(),
            audit_event_id: None,
            created_at: timestamp.to_string(),
            updated_at: timestamp.to_string(),
        });
    }
    Ok(())
}

fn append_stale_memory_findings(
    store: &FormalMemoryStoreV1,
    timestamp: &str,
    findings: &mut Vec<MemoryLintFinding>,
) {
    for record in &store.records {
        if record.status != MemoryLifecycleStatus::MemoryActive {
            continue;
        }
        let expired = record
            .scope
            .valid_until
            .as_deref()
            .is_some_and(|valid_until| valid_until <= timestamp);
        if expired {
            findings.push(MemoryLintFinding {
                finding_id: finding_id(MemoryLintFindingType::StaleMemory, &record.memory_id, None),
                schema_version: SCHEMA_VERSION.to_string(),
                finding_type: MemoryLintFindingType::StaleMemory,
                severity: MemoryLintFindingSeverity::Blocking,
                status: MemoryLintFindingStatus::Open,
                source_kind: "memory_record".to_string(),
                source_id: record.memory_id.clone(),
                target_memory_id: Some(record.memory_id.clone()),
                target_candidate_key: None,
                scope_type: Some(record.scope.scope_type.clone()),
                memory_type: Some(record.memory_type.clone()),
                claim: Some(record.claim.clone()),
                summary: "正式记忆已超过 valid_until；blocking finding 会阻止召回，M11 不会自动废弃正式记忆".to_string(),
                recommended_action: "review_lifecycle_deprecate_or_freeze".to_string(),
                evidence_refs: record.source_refs.clone(),
                audit_event_id: None,
                created_at: timestamp.to_string(),
                updated_at: timestamp.to_string(),
            });
        } else if record.superseded_by_memory_id.is_some() {
            findings.push(MemoryLintFinding {
                finding_id: finding_id(
                    MemoryLintFindingType::StaleMemory,
                    &record.memory_id,
                    record.superseded_by_memory_id.as_deref(),
                ),
                schema_version: SCHEMA_VERSION.to_string(),
                finding_type: MemoryLintFindingType::StaleMemory,
                severity: MemoryLintFindingSeverity::NeedsReview,
                status: MemoryLintFindingStatus::Open,
                source_kind: "memory_record".to_string(),
                source_id: record.memory_id.clone(),
                target_memory_id: Some(record.memory_id.clone()),
                target_candidate_key: None,
                scope_type: Some(record.scope.scope_type.clone()),
                memory_type: Some(record.memory_type.clone()),
                claim: Some(record.claim.clone()),
                summary:
                    "正式记忆已有 superseded_by 回链，需要人工复核是否通过 lifecycle 废弃或归档"
                        .to_string(),
                recommended_action: "review_staleness".to_string(),
                evidence_refs: record.source_refs.clone(),
                audit_event_id: None,
                created_at: timestamp.to_string(),
                updated_at: timestamp.to_string(),
            });
        }
    }
}

fn append_missing_source_findings(
    store: &FormalMemoryStoreV1,
    timestamp: &str,
    findings: &mut Vec<MemoryLintFinding>,
) {
    for record in &store.records {
        if record.source_refs.is_empty() {
            findings.push(MemoryLintFinding {
                finding_id: finding_id(
                    MemoryLintFindingType::MissingSource,
                    &record.memory_id,
                    None,
                ),
                schema_version: SCHEMA_VERSION.to_string(),
                finding_type: MemoryLintFindingType::MissingSource,
                severity: MemoryLintFindingSeverity::Blocking,
                status: MemoryLintFindingStatus::Open,
                source_kind: "memory_record".to_string(),
                source_id: record.memory_id.clone(),
                target_memory_id: Some(record.memory_id.clone()),
                target_candidate_key: None,
                scope_type: Some(record.scope.scope_type.clone()),
                memory_type: Some(record.memory_type.clone()),
                claim: Some(record.claim.clone()),
                summary: "正式记忆缺少 source_refs；维护任务只生成 finding，任务包召回应阻断"
                    .to_string(),
                recommended_action: "review_source_integrity".to_string(),
                evidence_refs: vec![],
                audit_event_id: None,
                created_at: timestamp.to_string(),
                updated_at: timestamp.to_string(),
            });
            continue;
        }
        if record.source_refs.iter().all(weak_source_ref) {
            findings.push(MemoryLintFinding {
                finding_id: finding_id(
                    MemoryLintFindingType::MissingSource,
                    &record.memory_id,
                    Some("weak_source"),
                ),
                schema_version: SCHEMA_VERSION.to_string(),
                finding_type: MemoryLintFindingType::MissingSource,
                severity: MemoryLintFindingSeverity::NeedsReview,
                status: MemoryLintFindingStatus::Open,
                source_kind: "memory_record".to_string(),
                source_id: record.memory_id.clone(),
                target_memory_id: Some(record.memory_id.clone()),
                target_candidate_key: None,
                scope_type: Some(record.scope.scope_type.clone()),
                memory_type: Some(record.memory_type.clone()),
                claim: Some(record.claim.clone()),
                summary: "正式记忆来源较弱，缺少明确 source_id / source_path / source_title，需要人工补证据".to_string(),
                recommended_action: "review_source_integrity".to_string(),
                evidence_refs: record.source_refs.clone(),
                audit_event_id: None,
                created_at: timestamp.to_string(),
                updated_at: timestamp.to_string(),
            });
        }
    }
}

fn append_sensitive_source_findings(
    store: &FormalMemoryStoreV1,
    timestamp: &str,
    findings: &mut Vec<MemoryLintFinding>,
) {
    for record in &store.records {
        let has_secret_source = record
            .source_refs
            .iter()
            .any(|source| source.sensitive_level == "secret");
        let has_private_source = record
            .source_refs
            .iter()
            .any(|source| matches!(source.sensitive_level.as_str(), "private" | "secret"));
        if has_secret_source && record.scope.model_export_policy != "blocked" {
            findings.push(MemoryLintFinding {
                finding_id: finding_id(
                    MemoryLintFindingType::SensitiveExportRisk,
                    &record.memory_id,
                    Some("secret_source"),
                ),
                schema_version: SCHEMA_VERSION.to_string(),
                finding_type: MemoryLintFindingType::SensitiveExportRisk,
                severity: MemoryLintFindingSeverity::Blocking,
                status: MemoryLintFindingStatus::Open,
                source_kind: "memory_record".to_string(),
                source_id: record.memory_id.clone(),
                target_memory_id: Some(record.memory_id.clone()),
                target_candidate_key: None,
                scope_type: Some(record.scope.scope_type.clone()),
                memory_type: Some(record.memory_type.clone()),
                claim: Some(record.claim.clone()),
                summary: "secret 来源记忆未设置 blocked 外发策略；blocking finding 会阻止召回"
                    .to_string(),
                recommended_action: "review_source_permission".to_string(),
                evidence_refs: record.source_refs.clone(),
                audit_event_id: None,
                created_at: timestamp.to_string(),
                updated_at: timestamp.to_string(),
            });
        } else if has_private_source && record.scope.model_export_policy == "allowed_with_redaction"
        {
            findings.push(MemoryLintFinding {
                finding_id: finding_id(
                    MemoryLintFindingType::SensitiveExportRisk,
                    &record.memory_id,
                    Some("private_redaction"),
                ),
                schema_version: SCHEMA_VERSION.to_string(),
                finding_type: MemoryLintFindingType::SensitiveExportRisk,
                severity: MemoryLintFindingSeverity::NeedsReview,
                status: MemoryLintFindingStatus::Open,
                source_kind: "memory_record".to_string(),
                source_id: record.memory_id.clone(),
                target_memory_id: Some(record.memory_id.clone()),
                target_candidate_key: None,
                scope_type: Some(record.scope.scope_type.clone()),
                memory_type: Some(record.memory_type.clone()),
                claim: Some(record.claim.clone()),
                summary: "private/secret 来源记忆允许脱敏外发，需要人工确认 redaction 充分"
                    .to_string(),
                recommended_action: "review_source_permission".to_string(),
                evidence_refs: record.source_refs.clone(),
                audit_event_id: None,
                created_at: timestamp.to_string(),
                updated_at: timestamp.to_string(),
            });
        }
        if contains_secret_like_terms(record) {
            findings.push(MemoryLintFinding {
                finding_id: finding_id(
                    MemoryLintFindingType::PrivateSourceRisk,
                    &record.memory_id,
                    Some("secret_like_terms"),
                ),
                schema_version: SCHEMA_VERSION.to_string(),
                finding_type: MemoryLintFindingType::PrivateSourceRisk,
                severity: MemoryLintFindingSeverity::NeedsReview,
                status: MemoryLintFindingStatus::Open,
                source_kind: "memory_record".to_string(),
                source_id: record.memory_id.clone(),
                target_memory_id: Some(record.memory_id.clone()),
                target_candidate_key: None,
                scope_type: Some(record.scope.scope_type.clone()),
                memory_type: Some(record.memory_type.clone()),
                claim: Some(record.claim.clone()),
                summary: "记忆正文或来源路径命中 private / secret 关键词，需要人工复核；维护任务不读取完整 transcript".to_string(),
                recommended_action: "review_private_source_scan".to_string(),
                evidence_refs: record.source_refs.clone(),
                audit_event_id: None,
                created_at: timestamp.to_string(),
                updated_at: timestamp.to_string(),
            });
        }
    }
}

fn append_entity_drift_findings(
    store: &MemoryEntityRelationStoreV1,
    timestamp: &str,
    findings: &mut Vec<MemoryLintFinding>,
) {
    for candidate in &store.merge_candidates {
        if candidate.status == MemoryRelationStatus::Rejected {
            continue;
        }
        findings.push(MemoryLintFinding {
            finding_id: finding_id(
                MemoryLintFindingType::EntityDrift,
                &candidate.merge_candidate_id,
                Some(&candidate.normalized_key),
            ),
            schema_version: SCHEMA_VERSION.to_string(),
            finding_type: MemoryLintFindingType::EntityDrift,
            severity: MemoryLintFindingSeverity::NeedsReview,
            status: MemoryLintFindingStatus::Open,
            source_kind: "memory_entity_merge_candidate".to_string(),
            source_id: candidate.merge_candidate_id.clone(),
            target_memory_id: None,
            target_candidate_key: None,
            scope_type: None,
            memory_type: None,
            claim: Some(format!(
                "{} / {}",
                candidate.left_label, candidate.right_label
            )),
            summary: "实体 dedupe / alias drift 候选仍待复核；M11 不会自动合并实体或关系"
                .to_string(),
            recommended_action: "review_entity_drift".to_string(),
            evidence_refs: vec![],
            audit_event_id: None,
            created_at: timestamp.to_string(),
            updated_at: timestamp.to_string(),
        });
    }
}

fn append_relation_source_revoked_findings(
    input: &MemoryLintRunInput,
    store: &MemoryEntityRelationStoreV1,
    timestamp: &str,
    findings: &mut Vec<MemoryLintFinding>,
) {
    if input.revoked_source_ids.is_empty() {
        return;
    }
    let revoked = input
        .revoked_source_ids
        .iter()
        .map(|item| normalize(item))
        .collect::<BTreeSet<_>>();
    for relation in &store.relations {
        if !relation.source_refs.iter().any(|source| {
            source
                .source_id
                .as_ref()
                .is_some_and(|id| revoked.contains(&normalize(id)))
        }) {
            continue;
        }
        findings.push(MemoryLintFinding {
            finding_id: finding_id(
                MemoryLintFindingType::RelationSourceRevoked,
                &relation.relation_id,
                None,
            ),
            schema_version: SCHEMA_VERSION.to_string(),
            finding_type: MemoryLintFindingType::RelationSourceRevoked,
            severity: MemoryLintFindingSeverity::NeedsReview,
            status: MemoryLintFindingStatus::Open,
            source_kind: "memory_relation".to_string(),
            source_id: relation.relation_id.clone(),
            target_memory_id: None,
            target_candidate_key: None,
            scope_type: None,
            memory_type: None,
            claim: Some(format!(
                "{} -> {} / {}",
                relation.subject_label, relation.object_label, relation.predicate
            )),
            summary: "已确认关系的来源权限已撤回；关系解释需要人工复核，不会自动改正式记忆"
                .to_string(),
            recommended_action: "review_relation_source_permission".to_string(),
            evidence_refs: relation_sources_to_memory_sources(&relation.source_refs),
            audit_event_id: None,
            created_at: timestamp.to_string(),
            updated_at: timestamp.to_string(),
        });
    }
}

fn append_index_status_findings(
    formal_store: &FormalMemoryStoreV1,
    entity_relation_store: &MemoryEntityRelationStoreV1,
    timestamp: &str,
    findings: &mut Vec<MemoryLintFinding>,
) {
    if formal_store.records.is_empty() {
        return;
    }
    let empty_entity_index = entity_relation_store.revision == 0
        && entity_relation_store.registry.entities.is_empty()
        && entity_relation_store.relations.is_empty();
    let stale_relation_index = !entity_relation_store.updated_at.is_empty()
        && entity_relation_store.updated_at < formal_store.updated_at;
    if empty_entity_index || stale_relation_index {
        findings.push(MemoryLintFinding {
            finding_id: finding_id(
                MemoryLintFindingType::DerivedIndexStale,
                &format!("formal-rev:{}", formal_store.revision),
                Some(&format!("entity-rev:{}", entity_relation_store.revision)),
            ),
            schema_version: SCHEMA_VERSION.to_string(),
            finding_type: MemoryLintFindingType::DerivedIndexStale,
            severity: MemoryLintFindingSeverity::Info,
            status: MemoryLintFindingStatus::Open,
            source_kind: "derived_index".to_string(),
            source_id: format!("memory-entity-relations:rev:{}", entity_relation_store.revision),
            target_memory_id: None,
            target_candidate_key: None,
            scope_type: None,
            memory_type: None,
            claim: None,
            summary: "派生实体 / 关系索引可能落后于正式记忆；M11 只生成 index status finding，不会重建索引或改事实".to_string(),
            recommended_action: "review_index_status".to_string(),
            evidence_refs: vec![],
            audit_event_id: None,
            created_at: timestamp.to_string(),
            updated_at: timestamp.to_string(),
        });
    }
}

fn append_mature_pattern_signal_findings(
    candidate_store: &MemoryCandidateStoreV1,
    observation_store: &ObservationStoreV1,
    timestamp: &str,
    findings: &mut Vec<MemoryLintFinding>,
) {
    let mut confirmed_by_type = BTreeMap::<String, Vec<&MemoryCandidate>>::new();
    for candidate in &candidate_store.candidates {
        if candidate.status == MemoryLifecycleStatus::CandidateConfirmed
            && candidate.adoption.is_none()
        {
            let key = format!("{}:{}", candidate.scope.scope_type, candidate.memory_type);
            confirmed_by_type.entry(key).or_default().push(candidate);
        }
    }
    for (key, candidates) in confirmed_by_type {
        if candidates.len() < 3 {
            continue;
        }
        let source_refs = candidates
            .iter()
            .flat_map(|candidate| candidate.source_refs.clone())
            .collect::<Vec<_>>();
        findings.push(MemoryLintFinding {
            finding_id: finding_id(MemoryLintFindingType::MaturePatternSignal, &key, None),
            schema_version: SCHEMA_VERSION.to_string(),
            finding_type: MemoryLintFindingType::MaturePatternSignal,
            severity: MemoryLintFindingSeverity::NeedsReview,
            status: MemoryLintFindingStatus::Open,
            source_kind: "memory_candidate_group".to_string(),
            source_id: key.clone(),
            target_memory_id: None,
            target_candidate_key: candidates.first().map(|candidate| candidate.candidate_key.clone()),
            scope_type: candidates.first().map(|candidate| candidate.scope.scope_type.clone()),
            memory_type: candidates.first().map(|candidate| candidate.memory_type.clone()),
            claim: Some(format!("{} 个已确认候选可能形成成熟模式", candidates.len())),
            summary: "多个已确认候选形成 mature pattern signal；后续 M12 可人工研究，M11 不会自动成为规则或全局记忆".to_string(),
            recommended_action: "review_mature_pattern_candidate".to_string(),
            evidence_refs: source_refs,
            audit_event_id: None,
            created_at: timestamp.to_string(),
            updated_at: timestamp.to_string(),
        });
    }
    let process_fact_observations = observation_store
        .observations
        .iter()
        .filter(|observation| observation.observation_type == "process_fact")
        .count();
    if process_fact_observations >= 3 {
        findings.push(MemoryLintFinding {
            finding_id: finding_id(
                MemoryLintFindingType::MaturePatternSignal,
                "process_fact_observations",
                Some(&process_fact_observations.to_string()),
            ),
            schema_version: SCHEMA_VERSION.to_string(),
            finding_type: MemoryLintFindingType::MaturePatternSignal,
            severity: MemoryLintFindingSeverity::Info,
            status: MemoryLintFindingStatus::Open,
            source_kind: "observation_group".to_string(),
            source_id: "process_fact_observations".to_string(),
            target_memory_id: None,
            target_candidate_key: None,
            scope_type: None,
            memory_type: Some("process_fact".to_string()),
            claim: Some(format!(
                "{} 条 process_fact observation 可供后续成熟模式研究",
                process_fact_observations
            )),
            summary: "process_fact observation 出现重复信号；M11 只提示，不写候选或正式记忆"
                .to_string(),
            recommended_action: "review_mature_pattern_candidate".to_string(),
            evidence_refs: vec![],
            audit_event_id: None,
            created_at: timestamp.to_string(),
            updated_at: timestamp.to_string(),
        });
    }
}

fn build_check_summaries(
    open_findings: &[&MemoryLintFinding],
    formal_store: &FormalMemoryStoreV1,
    candidate_store: &MemoryCandidateStoreV1,
) -> Vec<MemoryMaintenanceCheckSummary> {
    let mut summaries = Vec::new();
    for check_kind in [
        MemoryMaintenanceCheckKind::ExpiredOrStale,
        MemoryMaintenanceCheckKind::SourceIntegrity,
        MemoryMaintenanceCheckKind::DuplicateAndConflict,
        MemoryMaintenanceCheckKind::EntityRelationDrift,
        MemoryMaintenanceCheckKind::PermissionRevocation,
        MemoryMaintenanceCheckKind::SensitiveExportRisk,
        MemoryMaintenanceCheckKind::IndexStatus,
        MemoryMaintenanceCheckKind::MaturePatternSignal,
    ] {
        let findings = open_findings
            .iter()
            .filter(|finding| finding_check_kind(finding.finding_type) == check_kind)
            .collect::<Vec<_>>();
        let blocking_count = findings
            .iter()
            .filter(|finding| finding.severity == MemoryLintFindingSeverity::Blocking)
            .count();
        let needs_review_count = findings
            .iter()
            .filter(|finding| finding.severity == MemoryLintFindingSeverity::NeedsReview)
            .count();
        let info_count = findings
            .iter()
            .filter(|finding| finding.severity == MemoryLintFindingSeverity::Info)
            .count();
        let checked_count = match check_kind {
            MemoryMaintenanceCheckKind::MaturePatternSignal => candidate_store.candidates.len(),
            _ => formal_store.records.len(),
        };
        summaries.push(MemoryMaintenanceCheckSummary {
            check_kind: check_kind.clone(),
            checked_count,
            finding_count: findings.len(),
            blocking_count,
            needs_review_count,
            info_count,
            display_text: format!(
                "{}：checked {} / finding {} / blocking {} / needs_review {} / info {}",
                maintenance_check_label(&check_kind),
                checked_count,
                findings.len(),
                blocking_count,
                needs_review_count,
                info_count
            ),
        });
    }
    summaries
}

fn build_index_status(
    formal_store: &FormalMemoryStoreV1,
    entity_relation_store: &MemoryEntityRelationStoreV1,
    lint_store_revision: i64,
    timestamp: &str,
) -> MemoryMaintenanceIndexStatus {
    let stale = !entity_relation_store.updated_at.is_empty()
        && entity_relation_store.updated_at < formal_store.updated_at;
    let empty_index = !formal_store.records.is_empty()
        && entity_relation_store.revision == 0
        && entity_relation_store.registry.entities.is_empty();
    let status = if stale {
        "stale"
    } else if empty_index {
        "not_built"
    } else {
        "ok"
    };
    MemoryMaintenanceIndexStatus {
        status: status.to_string(),
        formal_store_revision: formal_store.revision,
        lint_store_revision,
        entity_relation_store_revision: entity_relation_store.revision,
        checked_at: timestamp.to_string(),
        display_text: format!(
            "索引状态 {status}：formal rev {} / entity-relation rev {}；状态 finding 不会重建索引或改变事实",
            formal_store.revision, entity_relation_store.revision
        ),
        warnings: if status == "ok" {
            vec![]
        } else {
            vec!["derived_index_status_requires_review".to_string()]
        },
    }
}

fn maintenance_recommendation(finding: &MemoryLintFinding) -> MemoryMaintenanceRecommendation {
    MemoryMaintenanceRecommendation {
        recommendation_id: format!(
            "memory-maintenance-recommendation:v1:{}",
            short_hash(&finding.finding_id)
        ),
        severity: finding.severity,
        target_kind: finding.source_kind.clone(),
        target_id: finding
            .target_memory_id
            .clone()
            .or_else(|| finding.target_candidate_key.clone())
            .or_else(|| Some(finding.source_id.clone())),
        action_label: finding.recommended_action.clone(),
        display_text: format!(
            "{}：{}；建议人工复核，不会自动执行 lifecycle",
            finding_type_name(finding.finding_type),
            finding.summary
        ),
    }
}

fn finding_check_kind(finding_type: MemoryLintFindingType) -> MemoryMaintenanceCheckKind {
    match finding_type {
        MemoryLintFindingType::StaleMemory | MemoryLintFindingType::AuthoritySuperseded => {
            MemoryMaintenanceCheckKind::ExpiredOrStale
        }
        MemoryLintFindingType::MissingSource => MemoryMaintenanceCheckKind::SourceIntegrity,
        MemoryLintFindingType::DuplicateClaim
        | MemoryLintFindingType::ClaimConflict
        | MemoryLintFindingType::CandidateConflictsWithActiveMemory => {
            MemoryMaintenanceCheckKind::DuplicateAndConflict
        }
        MemoryLintFindingType::EntityDrift => MemoryMaintenanceCheckKind::EntityRelationDrift,
        MemoryLintFindingType::SourcePermissionRevoked
        | MemoryLintFindingType::RelationSourceRevoked => {
            MemoryMaintenanceCheckKind::PermissionRevocation
        }
        MemoryLintFindingType::SensitiveExportRisk | MemoryLintFindingType::PrivateSourceRisk => {
            MemoryMaintenanceCheckKind::SensitiveExportRisk
        }
        MemoryLintFindingType::DerivedIndexStale => MemoryMaintenanceCheckKind::IndexStatus,
        MemoryLintFindingType::MaturePatternSignal => {
            MemoryMaintenanceCheckKind::MaturePatternSignal
        }
    }
}

fn maintenance_check_label(check_kind: &MemoryMaintenanceCheckKind) -> &'static str {
    match check_kind {
        MemoryMaintenanceCheckKind::ExpiredOrStale => "过期 / stale",
        MemoryMaintenanceCheckKind::SourceIntegrity => "来源完整性",
        MemoryMaintenanceCheckKind::DuplicateAndConflict => "重复 / 冲突",
        MemoryMaintenanceCheckKind::EntityRelationDrift => "实体 / 关系漂移",
        MemoryMaintenanceCheckKind::PermissionRevocation => "权限撤回",
        MemoryMaintenanceCheckKind::SensitiveExportRisk => "私密 / 外发风险",
        MemoryMaintenanceCheckKind::IndexStatus => "索引状态",
        MemoryMaintenanceCheckKind::MaturePatternSignal => "成熟模式信号",
    }
}

fn should_run_maintenance_checks(intent: MemoryLintRunIntent) -> bool {
    matches!(
        intent,
        MemoryLintRunIntent::MaintenancePreview | MemoryLintRunIntent::MaintenanceRun
    )
}

fn weak_source_ref(source: &MemorySourceRef) -> bool {
    source.source_id.as_deref().unwrap_or("").trim().is_empty()
        && source
            .source_path
            .as_deref()
            .unwrap_or("")
            .trim()
            .is_empty()
        && source
            .source_title
            .as_deref()
            .unwrap_or("")
            .trim()
            .is_empty()
}

fn contains_secret_like_terms(record: &MemoryRecord) -> bool {
    let mut values = vec![record.claim.as_str(), record.body.as_str()];
    for source in &record.source_refs {
        if let Some(path) = source.source_path.as_deref() {
            values.push(path);
        }
        if let Some(title) = source.source_title.as_deref() {
            values.push(title);
        }
    }
    values.iter().any(|value| {
        let normalized = normalize(value);
        ["api_key", "apikey", "password", "secret", ".env", "token"]
            .iter()
            .any(|term| normalized.contains(term))
    })
}

fn relation_sources_to_memory_sources(sources: &[MemoryRelationSource]) -> Vec<MemorySourceRef> {
    sources
        .iter()
        .enumerate()
        .map(|(index, source)| MemorySourceRef {
            source_ref_id: format!(
                "relation-source:{}:{}",
                index,
                source.source_id.clone().unwrap_or_default()
            ),
            source_type: format!("{:?}", source.source_kind).to_lowercase(),
            source_id: source.source_id.clone(),
            source_path: source.source_path.clone(),
            source_title: source.source_title.clone(),
            anchor: None,
            source_created_at: None,
            captured_at: String::new(),
            authority_level: source.authority_level.clone(),
            sensitive_level: source.sensitive_level.clone(),
            content_hash: None,
        })
        .collect()
}

fn same_scope_type_and_memory_type(left: &MemoryRecord, right: &MemoryRecord) -> bool {
    left.scope.scope_type == right.scope.scope_type
        && left.scope.project_id == right.scope.project_id
        && left.scope.workflow_id == right.scope.workflow_id
        && left.memory_type == right.memory_type
}

fn same_candidate_scope_and_type(candidate: &MemoryCandidate, record: &MemoryRecord) -> bool {
    candidate.scope.scope_type == record.scope.scope_type
        && candidate.scope.project_id == record.scope.project_id
        && candidate.scope.workflow_id == record.scope.workflow_id
        && candidate.memory_type == record.memory_type
}

fn has_stronger_authority(source_refs: &[MemorySourceRef]) -> bool {
    source_refs.iter().any(|source| {
        matches!(
            normalize(&source.authority_level).as_str(),
            "user_confirmed" | "current_authority" | "policy"
        )
    })
}

fn has_exclusive_terms(left: &str, right: &str) -> bool {
    let left = normalize(left);
    let right = normalize(right);
    const PAIRS: [(&str, &str); 8] = [
        ("必须", "禁止"),
        ("允许", "禁止"),
        ("启用", "禁用"),
        ("需要", "不需要"),
        ("使用", "不使用"),
        ("always", "never"),
        ("enabled", "disabled"),
        ("allow", "forbid"),
    ];
    PAIRS.iter().any(|(a, b)| {
        (left.contains(a) && right.contains(b)) || (left.contains(b) && right.contains(a))
    })
}

fn claim_similarity(left: &str, right: &str) -> f32 {
    let left_tokens = token_set(left);
    let right_tokens = token_set(right);
    if left_tokens.is_empty() || right_tokens.is_empty() {
        return 0.0;
    }
    let intersection = left_tokens.intersection(&right_tokens).count();
    let union = left_tokens.union(&right_tokens).count();
    intersection as f32 / union as f32
}

fn token_set(value: &str) -> BTreeSet<String> {
    let normalized = normalize(value);
    let mut tokens = normalized
        .split(|ch: char| ch.is_whitespace() || ch.is_ascii_punctuation())
        .filter(|token| token.chars().count() >= 2)
        .map(ToString::to_string)
        .collect::<BTreeSet<_>>();
    if tokens.len() <= 1 {
        let chars = normalized.chars().collect::<Vec<_>>();
        for pair in chars.windows(2) {
            let token = pair.iter().collect::<String>().trim().to_string();
            if token.chars().count() >= 2 {
                tokens.insert(token);
            }
        }
    }
    tokens
}

fn combined_source_refs(left: &MemoryRecord, right: &MemoryRecord) -> Vec<MemorySourceRef> {
    left.source_refs
        .iter()
        .chain(right.source_refs.iter())
        .cloned()
        .collect()
}

fn finding_id(
    finding_type: MemoryLintFindingType,
    source_id: &str,
    target_id: Option<&str>,
) -> String {
    format!(
        "memlint:v1:{}:{}",
        finding_type_name(finding_type),
        short_hash(&format!(
            "{}:{}:{}",
            finding_type_name(finding_type),
            normalize(source_id),
            normalize(target_id.unwrap_or_default())
        ))
    )
}

fn finding_type_name(finding_type: MemoryLintFindingType) -> &'static str {
    match finding_type {
        MemoryLintFindingType::DuplicateClaim => "duplicate_claim",
        MemoryLintFindingType::ClaimConflict => "claim_conflict",
        MemoryLintFindingType::SourcePermissionRevoked => "source_permission_revoked",
        MemoryLintFindingType::AuthoritySuperseded => "authority_superseded",
        MemoryLintFindingType::StaleMemory => "stale_memory",
        MemoryLintFindingType::MissingSource => "missing_source",
        MemoryLintFindingType::CandidateConflictsWithActiveMemory => {
            "candidate_conflicts_with_active_memory"
        }
        MemoryLintFindingType::EntityDrift => "entity_drift",
        MemoryLintFindingType::RelationSourceRevoked => "relation_source_revoked",
        MemoryLintFindingType::SensitiveExportRisk => "sensitive_export_risk",
        MemoryLintFindingType::PrivateSourceRisk => "private_source_risk",
        MemoryLintFindingType::DerivedIndexStale => "derived_index_stale",
        MemoryLintFindingType::MaturePatternSignal => "mature_pattern_signal",
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
