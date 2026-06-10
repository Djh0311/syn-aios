use crate::{
    CreateFormalMemoryRecordInput, CreateFormalMemoryRecordOutput, FormalMemoryStoreV1,
    MaturePatternAuditEvent, MaturePatternCandidate, MaturePatternCandidateStatus,
    MaturePatternDecisionKind, MaturePatternPreviewOutput, MemoryCandidateStoreV1,
    MemoryClusterMemberRef, MemoryClusterReport, MemoryEntityRelationStoreV1,
    MemoryLifecycleStatus, MemoryLintFinding, MemoryLintFindingType, MemoryLintStoreV1,
    MemoryPatternStoreV1, MemoryScope, MemorySourceRef, MemorySystemAcceptanceGate,
    MemorySystemAcceptanceSummary, ObservationStoreV1, PreviewMaturePatternsInput,
    RecordMaturePatternDecisionInput, RecordMaturePatternDecisionOutput,
};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub(crate) fn preview_mature_patterns(
    workflow_state_path: &Path,
    input: &PreviewMaturePatternsInput,
    timestamp: &str,
) -> Result<MaturePatternPreviewOutput, String> {
    validate_preview_input(input)?;
    let store = crate::mature_pattern_store::load_store(workflow_state_path, timestamp)?;
    let formal_store = crate::formal_memory_store::load_store(workflow_state_path, timestamp)?;
    let candidate_store =
        crate::memory_candidate_store::load_store(workflow_state_path, timestamp)?;
    let observation_store = crate::observation_store::load_store(workflow_state_path, timestamp)?;
    let lint_store = crate::memory_lint_store::load_store(workflow_state_path, timestamp)?;
    let entity_relation_store =
        crate::memory_entity_relation_store::load_store(workflow_state_path, timestamp)?;
    let derived = derive_mature_pattern_materials(
        input,
        timestamp,
        &store,
        &formal_store,
        &candidate_store,
        &observation_store,
        &lint_store,
        &entity_relation_store,
    );
    Ok(MaturePatternPreviewOutput {
        store_revision: store.revision,
        mature_pattern_candidates: derived.candidates,
        cluster_reports: derived.cluster_reports,
        acceptance_summary: build_acceptance_summary(
            &formal_store,
            &candidate_store,
            &observation_store,
            &lint_store,
            &entity_relation_store,
            &store,
            timestamp,
        ),
        summary: crate::mature_pattern_store::summarize_store(&store),
        warnings: vec![
            "mature_pattern_preview_readonly".to_string(),
            "cluster_reports_are_not_formal_memory".to_string(),
        ],
    })
}

pub(crate) fn record_mature_pattern_decision(
    workflow_state_path: &Path,
    input: &RecordMaturePatternDecisionInput,
    timestamp: &str,
    pattern_write_id: &str,
    formal_write_id: &str,
) -> Result<RecordMaturePatternDecisionOutput, String> {
    validate_decision_input(input)?;
    let preview_input = preview_input_from_project_root(&input.project_root);
    crate::mature_pattern_store::with_locked_store(
        workflow_state_path,
        timestamp,
        pattern_write_id,
        |store| {
            validate_expected_revision(input.expected_pattern_store_revision, store)?;
            let formal_store =
                crate::formal_memory_store::load_store(workflow_state_path, timestamp)?;
            let candidate_store =
                crate::memory_candidate_store::load_store(workflow_state_path, timestamp)?;
            let observation_store =
                crate::observation_store::load_store(workflow_state_path, timestamp)?;
            let lint_store = crate::memory_lint_store::load_store(workflow_state_path, timestamp)?;
            let entity_relation_store =
                crate::memory_entity_relation_store::load_store(workflow_state_path, timestamp)?;
            let derived = derive_mature_pattern_materials(
                &preview_input,
                timestamp,
                store,
                &formal_store,
                &candidate_store,
                &observation_store,
                &lint_store,
                &entity_relation_store,
            );
            let candidate = derived
                .candidates
                .into_iter()
                .find(|candidate| candidate.candidate_id == input.candidate_id)
                .ok_or_else(|| {
                    format!("找不到成熟模式候选，无法记录决定：{}", input.candidate_id)
                })?;
            let (after_status, formal_memory_output) = match input.decision {
                MaturePatternDecisionKind::ConfirmAsFormalMemory => {
                    validate_user_confirmation(input)?;
                    (
                        MaturePatternCandidateStatus::Confirmed,
                        Some(create_formal_mature_pattern_memory(
                            workflow_state_path,
                            input,
                            &candidate,
                            formal_write_id,
                        )?),
                    )
                }
                MaturePatternDecisionKind::Reject => (MaturePatternCandidateStatus::Rejected, None),
                MaturePatternDecisionKind::Quarantine => {
                    (MaturePatternCandidateStatus::Quarantined, None)
                }
                MaturePatternDecisionKind::RequestChanges => {
                    (MaturePatternCandidateStatus::ChangesRequested, None)
                }
            };
            let mut decided_candidate = candidate.clone();
            decided_candidate.status = after_status;
            decided_candidate.updated_at = timestamp.to_string();
            push_candidate_decision(store, decided_candidate.clone());
            for report in derived.cluster_reports {
                if !store
                    .cluster_reports
                    .iter()
                    .any(|existing| existing.report_id == report.report_id)
                {
                    store.cluster_reports.push(report);
                }
            }
            let formal_memory_id = formal_memory_output
                .as_ref()
                .map(|output| output.record.memory_id.clone());
            let audit_event = MaturePatternAuditEvent {
                audit_event_id: format!(
                    "mature-pattern-audit:v1:{timestamp}:{}",
                    short_hash(&format!(
                        "{}:{}:{}",
                        input.candidate_id,
                        decision_name(input.decision),
                        input.actor_id
                    ))
                ),
                event_type: "mature_pattern_decision_recorded".to_string(),
                actor_id: input.actor_id.clone(),
                actor_role: input.actor_role.clone(),
                target_kind: "mature_pattern_candidate".to_string(),
                target_id: input.candidate_id.clone(),
                before_status: Some(MaturePatternCandidateStatus::Candidate),
                after_status: Some(after_status),
                formal_memory_id,
                reason: input.reason.trim().to_string(),
                created_at: timestamp.to_string(),
                warnings: vec![
                    "mature_pattern_candidate_decision_does_not_delete_sources".to_string(),
                    "user_confirmation_required_for_formal_mature_pattern".to_string(),
                ],
            };
            store.audit_events.push(audit_event.clone());
            store.project_id = Some(crate::project_id(&input.project_root));
            store.workflow_id = Some(crate::default_workflow_id(&input.project_root));
            store.revision += 1;
            let acceptance_formal_store = if formal_memory_output.is_some() {
                crate::formal_memory_store::load_store(workflow_state_path, timestamp)?
            } else {
                formal_store
            };
            Ok(RecordMaturePatternDecisionOutput {
                store_revision: store.revision,
                candidate: decided_candidate,
                formal_memory_output,
                audit_event,
                acceptance_summary: build_acceptance_summary(
                    &acceptance_formal_store,
                    &candidate_store,
                    &observation_store,
                    &lint_store,
                    &entity_relation_store,
                    store,
                    timestamp,
                ),
                warnings: vec![
                    "cluster_reports_are_not_formal_memory".to_string(),
                    "unconfirmed_mature_pattern_candidates_do_not_enter_task_packet".to_string(),
                ],
            })
        },
    )
}

struct DerivedMaturePatternMaterials {
    candidates: Vec<MaturePatternCandidate>,
    cluster_reports: Vec<MemoryClusterReport>,
}

fn derive_mature_pattern_materials(
    input: &PreviewMaturePatternsInput,
    timestamp: &str,
    store: &MemoryPatternStoreV1,
    formal_store: &FormalMemoryStoreV1,
    candidate_store: &MemoryCandidateStoreV1,
    observation_store: &ObservationStoreV1,
    lint_store: &MemoryLintStoreV1,
    entity_relation_store: &MemoryEntityRelationStoreV1,
) -> DerivedMaturePatternMaterials {
    let mut candidates = Vec::new();
    append_lint_signal_candidates(input, timestamp, lint_store, &mut candidates);
    append_repeated_candidate_patterns(input, timestamp, candidate_store, &mut candidates);
    append_repeated_observation_patterns(input, timestamp, observation_store, &mut candidates);
    append_relation_theme_patterns(
        input,
        timestamp,
        formal_store,
        entity_relation_store,
        &mut candidates,
    );
    let mut cluster_reports = build_cluster_reports(timestamp, &candidates);
    overlay_persisted_candidates(store, &mut candidates);
    overlay_persisted_reports(store, &mut cluster_reports);
    dedupe_candidates(&mut candidates);
    dedupe_reports(&mut cluster_reports);
    DerivedMaturePatternMaterials {
        candidates,
        cluster_reports,
    }
}

fn append_lint_signal_candidates(
    input: &PreviewMaturePatternsInput,
    timestamp: &str,
    lint_store: &MemoryLintStoreV1,
    output: &mut Vec<MaturePatternCandidate>,
) {
    for finding in &lint_store.findings {
        if finding.finding_type != MemoryLintFindingType::MaturePatternSignal {
            continue;
        }
        let claim = finding
            .claim
            .clone()
            .unwrap_or_else(|| finding.summary.clone());
        let source_refs = if finding.evidence_refs.is_empty() {
            vec![source_ref_from_finding(finding, timestamp)]
        } else {
            finding.evidence_refs.clone()
        };
        let member_refs = vec![member_ref(
            "maintenance_finding",
            &finding.finding_id,
            input
                .project_id
                .clone()
                .or_else(|| finding.scope_type.clone()),
            &finding.summary,
            source_refs.clone(),
        )];
        output.push(candidate_from_parts(
            input,
            timestamp,
            "maintenance_signal",
            &format!("成熟模式信号：{}", short_title(&claim)),
            &claim,
            &format!(
                "{}。该信号来自 M11 maintenance finding，必须用户确认后才可写正式记忆。",
                finding.summary
            ),
            source_refs,
            member_refs,
            vec![finding.finding_id.clone()],
        ));
    }
}

fn append_repeated_candidate_patterns(
    input: &PreviewMaturePatternsInput,
    timestamp: &str,
    candidate_store: &MemoryCandidateStoreV1,
    output: &mut Vec<MaturePatternCandidate>,
) {
    let mut groups: BTreeMap<String, Vec<_>> = BTreeMap::new();
    for candidate in &candidate_store.candidates {
        if candidate.status != MemoryLifecycleStatus::CandidateConfirmed {
            continue;
        }
        groups
            .entry(pattern_key(&candidate.claim))
            .or_default()
            .push(candidate);
    }
    for (_key, group) in groups {
        if group.len() < 2 {
            continue;
        }
        let source_refs = group
            .iter()
            .flat_map(|candidate| candidate.source_refs.clone())
            .collect::<Vec<_>>();
        let member_refs = group
            .iter()
            .map(|candidate| {
                member_ref(
                    "memory_candidate",
                    &candidate.candidate_key,
                    candidate.scope.project_id.clone(),
                    &candidate.claim,
                    candidate.source_refs.clone(),
                )
            })
            .collect::<Vec<_>>();
        let claim = format!("重复候选显示稳定模式：{}", short_title(&group[0].claim));
        output.push(candidate_from_parts(
            input,
            timestamp,
            "repeated_candidate",
            &claim,
            &claim,
            "多条已确认候选出现相似 claim，可作为成熟模式候选；候选未确认不会进入任务包。",
            source_refs,
            member_refs,
            group
                .iter()
                .map(|candidate| candidate.candidate_key.clone())
                .collect(),
        ));
    }
}

fn append_repeated_observation_patterns(
    input: &PreviewMaturePatternsInput,
    timestamp: &str,
    observation_store: &ObservationStoreV1,
    output: &mut Vec<MaturePatternCandidate>,
) {
    let mut groups: BTreeMap<String, Vec<_>> = BTreeMap::new();
    for observation in &observation_store.observations {
        groups
            .entry(pattern_key(&observation.summary))
            .or_default()
            .push(observation);
    }
    for (_key, group) in groups {
        if group.len() < 2 {
            continue;
        }
        let source_refs = group
            .iter()
            .flat_map(|observation| {
                observation
                    .source_refs
                    .iter()
                    .map(|source| source_ref_from_observation_source(source, timestamp))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let member_refs = group
            .iter()
            .map(|observation| {
                member_ref(
                    "observation",
                    &observation.observation_key,
                    observation.project_id.clone(),
                    &observation.summary,
                    observation
                        .source_refs
                        .iter()
                        .map(|source| source_ref_from_observation_source(source, timestamp))
                        .collect(),
                )
            })
            .collect::<Vec<_>>();
        let claim = format!("重复观察显示稳定模式：{}", short_title(&group[0].summary));
        output.push(candidate_from_parts(
            input,
            timestamp,
            "repeated_observation",
            &claim,
            &claim,
            "多条 recorded observation 出现相似摘要，可作为成熟模式候选；observation 仍不是正式记忆。",
            source_refs,
            member_refs,
            group
                .iter()
                .map(|observation| observation.observation_key.clone())
                .collect(),
        ));
    }
}

fn append_relation_theme_patterns(
    input: &PreviewMaturePatternsInput,
    timestamp: &str,
    formal_store: &FormalMemoryStoreV1,
    entity_relation_store: &MemoryEntityRelationStoreV1,
    output: &mut Vec<MaturePatternCandidate>,
) {
    if entity_relation_store.relations.is_empty() || formal_store.records.len() < 2 {
        return;
    }
    let records = formal_store
        .records
        .iter()
        .filter(|record| record.status == MemoryLifecycleStatus::MemoryActive)
        .take(4)
        .collect::<Vec<_>>();
    if records.len() < 2 {
        return;
    }
    let source_refs = records
        .iter()
        .flat_map(|record| record.source_refs.clone())
        .collect::<Vec<_>>();
    let member_refs = records
        .iter()
        .map(|record| {
            member_ref(
                "memory_record",
                &record.memory_id,
                record.scope.project_id.clone(),
                &record.claim,
                record.source_refs.clone(),
            )
        })
        .collect::<Vec<_>>();
    output.push(candidate_from_parts(
        input,
        timestamp,
        "cross_project_theme",
        "跨项目主题报告候选",
        "已确认关系和多条正式记忆显示跨项目主题",
        "M10 confirmed relation 可作为解释线索，但跨项目主题仍必须用户确认后才可写正式记忆。",
        source_refs,
        member_refs,
        entity_relation_store
            .relations
            .iter()
            .map(|relation| relation.relation_id.clone())
            .collect(),
    ));
}

fn candidate_from_parts(
    input: &PreviewMaturePatternsInput,
    timestamp: &str,
    pattern_kind: &str,
    title: &str,
    claim: &str,
    body: &str,
    source_refs: Vec<MemorySourceRef>,
    member_refs: Vec<MemoryClusterMemberRef>,
    signal_refs: Vec<String>,
) -> MaturePatternCandidate {
    let candidate_id = format!(
        "mature-pattern-candidate:v1:{}",
        short_hash(&format!(
            "{}:{}:{}:{}",
            pattern_kind,
            claim,
            member_refs
                .iter()
                .map(|member| member.member_id.clone())
                .collect::<Vec<_>>()
                .join("|"),
            input.project_root
        ))
    );
    MaturePatternCandidate {
        candidate_id,
        pattern_kind: pattern_kind.to_string(),
        scope: global_scope(input, timestamp),
        title: title.to_string(),
        claim: claim.to_string(),
        body: body.to_string(),
        source_refs: dedupe_source_refs(source_refs),
        member_refs,
        signal_refs,
        status: MaturePatternCandidateStatus::Candidate,
        requires_user_confirmation: true,
        review_summary: "成熟模式 / 跨项目记忆必须用户确认；候选未确认，不会进入任务包。"
            .to_string(),
        created_at: timestamp.to_string(),
        updated_at: timestamp.to_string(),
        warnings: vec![
            "mature_pattern_candidate_not_formal_memory".to_string(),
            "user_confirmation_required".to_string(),
        ],
    }
}

fn build_cluster_reports(
    timestamp: &str,
    candidates: &[MaturePatternCandidate],
) -> Vec<MemoryClusterReport> {
    candidates
        .iter()
        .filter(|candidate| candidate.member_refs.len() >= 2)
        .map(|candidate| {
            let project_ids = candidate
                .member_refs
                .iter()
                .filter_map(|member| member.project_id.clone())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();
            MemoryClusterReport {
                report_id: format!(
                    "memory-cluster-report:v1:{}",
                    short_hash(&candidate.candidate_id)
                ),
                report_kind: candidate.pattern_kind.clone(),
                scope_type: candidate.scope.scope_type.clone(),
                title: format!("跨项目主题报告：{}", candidate.title),
                project_ids,
                member_refs: candidate.member_refs.clone(),
                source_refs: candidate.source_refs.clone(),
                status: "derived_report".to_string(),
                staleness: "derived_from_current_sidecars".to_string(),
                display_text: format!(
                    "跨项目主题报告包含 {} 个 member refs / {} 个 source refs；报告可下钻来源，但不是正式事实。",
                    candidate.member_refs.len(),
                    candidate.source_refs.len()
                ),
                created_at: timestamp.to_string(),
                warnings: vec!["cluster_report_not_formal_memory".to_string()],
            }
        })
        .collect()
}

fn create_formal_mature_pattern_memory(
    workflow_state_path: &Path,
    input: &RecordMaturePatternDecisionInput,
    candidate: &MaturePatternCandidate,
    formal_write_id: &str,
) -> Result<CreateFormalMemoryRecordOutput, String> {
    let request = CreateFormalMemoryRecordInput {
        project_root: input.project_root.clone(),
        project_id: Some(crate::project_id(&input.project_root)),
        workflow_id: Some(crate::default_workflow_id(&input.project_root)),
        scope: candidate.scope.clone(),
        memory_type: "mature_pattern".to_string(),
        claim: candidate.claim.clone(),
        body: format!(
            "{}\n\n来源成员：{}。用户确认后才写入正式记忆。",
            candidate.body,
            candidate.member_refs.len()
        ),
        source_refs: candidate.source_refs.clone(),
        actor_id: input.actor_id.clone(),
        actor_role: "user".to_string(),
        reason: input.reason.clone(),
        audit_event_type: Some("mature_pattern_user_confirmed_to_formal_memory".to_string()),
        expected_store_revision: input.expected_formal_store_revision,
    };
    crate::formal_memory_store::create_record(
        workflow_state_path,
        &request,
        &crate::unix_timestamp_string(),
        formal_write_id,
    )
}

pub(crate) fn build_acceptance_summary(
    formal_store: &FormalMemoryStoreV1,
    candidate_store: &MemoryCandidateStoreV1,
    observation_store: &ObservationStoreV1,
    lint_store: &MemoryLintStoreV1,
    entity_relation_store: &MemoryEntityRelationStoreV1,
    pattern_store: &MemoryPatternStoreV1,
    timestamp: &str,
) -> MemorySystemAcceptanceSummary {
    let mut gates = Vec::new();
    gates.push(gate(
        "observation",
        "观察入口",
        !observation_store.observations.is_empty(),
        "deferred",
        format!("observation count {}", observation_store.observations.len()),
        None,
    ));
    gates.push(gate(
        "candidate",
        "候选记忆",
        !candidate_store.candidates.is_empty(),
        "deferred",
        format!("candidate count {}", candidate_store.candidates.len()),
        None,
    ));
    gates.push(gate(
        "formal_memory",
        "正式记忆 / version / audit",
        !formal_store.records.is_empty()
            && !formal_store.versions.is_empty()
            && !formal_store.audit_events.is_empty(),
        "blocked",
        format!(
            "record {} / version {} / audit {}",
            formal_store.records.len(),
            formal_store.versions.len(),
            formal_store.audit_events.len()
        ),
        Some("缺少正式记忆、版本或审计"),
    ));
    gates.push(gate(
        "lint",
        "权限 / 冲突 / 维护 finding",
        !lint_store.findings.is_empty() || !lint_store.maintenance_reports.is_empty(),
        "deferred",
        format!(
            "finding {} / maintenance report {}",
            lint_store.findings.len(),
            lint_store.maintenance_reports.len()
        ),
        None,
    ));
    gates.push(gate(
        "relation",
        "实体 / 关系解释",
        !entity_relation_store.relations.is_empty()
            || !entity_relation_store.relation_candidates.is_empty(),
        "deferred",
        format!(
            "relations {} / relation candidates {}",
            entity_relation_store.relations.len(),
            entity_relation_store.relation_candidates.len()
        ),
        None,
    ));
    gates.push(gate(
        "mature_pattern",
        "成熟模式 / 跨项目候选",
        !pattern_store.mature_pattern_candidates.is_empty()
            || !pattern_store.cluster_reports.is_empty(),
        "deferred",
        format!(
            "mature pattern candidates {} / cluster reports {}",
            pattern_store.mature_pattern_candidates.len(),
            pattern_store.cluster_reports.len()
        ),
        None,
    ));
    gates.push(gate(
        "task_packet",
        "任务包召回边界",
        formal_store
            .records
            .iter()
            .any(|record| record.status == MemoryLifecycleStatus::MemoryActive),
        "blocked",
        "active formal memory can be evaluated by task packet builder".to_string(),
        Some("缺少 active formal memory"),
    ));
    let passed_count = gates.iter().filter(|gate| gate.status == "passed").count();
    let blocked_count = gates.iter().filter(|gate| gate.status == "blocked").count();
    let deferred_count = gates
        .iter()
        .filter(|gate| gate.status == "deferred")
        .count();
    MemorySystemAcceptanceSummary {
        summary_id: format!(
            "memory-system-acceptance:m1-m12:{}",
            short_hash(&format!(
                "{}:{}:{}:{}:{}",
                formal_store.revision,
                candidate_store.revision,
                observation_store.revision,
                lint_store.revision,
                pattern_store.revision
            ))
        ),
        scope_label: "M1-M12 memory system acceptance summary".to_string(),
        gate_count: gates.len(),
        passed_count,
        blocked_count,
        deferred_count,
        gates,
        display_text: format!(
            "M1-M12 验收摘要：passed {} / blocked {} / deferred {}；M13 最终验收仍后置。",
            passed_count, blocked_count, deferred_count
        ),
        warnings: vec![
            "m12_is_not_m13_final_acceptance".to_string(),
            "cluster_reports_are_not_formal_memory".to_string(),
        ],
        created_at: timestamp.to_string(),
    }
}

fn gate(
    gate_id: &str,
    label: &str,
    passed: bool,
    fallback_status: &str,
    evidence: String,
    blocking_reason: Option<&str>,
) -> MemorySystemAcceptanceGate {
    MemorySystemAcceptanceGate {
        gate_id: gate_id.to_string(),
        label: label.to_string(),
        status: if passed {
            "passed".to_string()
        } else {
            fallback_status.to_string()
        },
        evidence,
        blocking_reason: if passed {
            None
        } else {
            blocking_reason.map(|reason| reason.to_string())
        },
    }
}

fn validate_preview_input(input: &PreviewMaturePatternsInput) -> Result<(), String> {
    if input.project_root.trim().is_empty() {
        return Err("成熟模式 preview 缺少 project_root".to_string());
    }
    let expected_project_id = crate::project_id(&input.project_root);
    let expected_workflow_id = crate::default_workflow_id(&input.project_root);
    validate_context_field(
        "project_id",
        input.project_id.as_deref(),
        &expected_project_id,
    )?;
    validate_context_field(
        "workflow_id",
        input.workflow_id.as_deref(),
        &expected_workflow_id,
    )?;
    Ok(())
}

fn validate_decision_input(input: &RecordMaturePatternDecisionInput) -> Result<(), String> {
    if input.project_root.trim().is_empty() {
        return Err("成熟模式决定缺少 project_root".to_string());
    }
    if input.candidate_id.trim().is_empty() {
        return Err("成熟模式决定缺少 candidate_id".to_string());
    }
    if input.actor_id.trim().is_empty() {
        return Err("成熟模式决定缺少 actor_id".to_string());
    }
    if input.reason.trim().is_empty() {
        return Err("成熟模式决定缺少 reason".to_string());
    }
    if !matches!(
        input.actor_role.as_str(),
        "user" | "project_director" | "global_director" | "secretary"
    ) {
        return Err(format!(
            "当前角色不能记录成熟模式决定：{}",
            input.actor_role
        ));
    }
    if input.actor_role == "secretary" {
        return Err("秘书只能解释成熟模式候选，不能记录决定".to_string());
    }
    Ok(())
}

fn validate_user_confirmation(input: &RecordMaturePatternDecisionInput) -> Result<(), String> {
    if input.actor_role != "user" || input.confirmed_by.as_deref() != Some("user") {
        return Err("成熟模式 / 跨项目 / 全局记忆正式化必须由用户确认".to_string());
    }
    Ok(())
}

fn validate_expected_revision(
    expected: Option<i64>,
    store: &MemoryPatternStoreV1,
) -> Result<(), String> {
    if let Some(expected) = expected {
        if expected != store.revision {
            return Err(format!(
                "memory_pattern_store_conflict: expected revision {expected}, actual {}",
                store.revision
            ));
        }
    }
    Ok(())
}

fn validate_context_field(name: &str, actual: Option<&str>, expected: &str) -> Result<(), String> {
    if let Some(actual) = actual {
        if actual != expected {
            return Err(format!(
                "成熟模式上下文不匹配：{name} expected {expected}, actual {actual}"
            ));
        }
    }
    Ok(())
}

fn preview_input_from_project_root(project_root: &str) -> PreviewMaturePatternsInput {
    PreviewMaturePatternsInput {
        project_root: project_root.to_string(),
        project_id: Some(crate::project_id(project_root)),
        workflow_id: Some(crate::default_workflow_id(project_root)),
    }
}

fn global_scope(input: &PreviewMaturePatternsInput, timestamp: &str) -> MemoryScope {
    MemoryScope {
        scope_id: format!("scope:global:{}", short_hash(&input.project_root)),
        scope_type: "global".to_string(),
        user_id: None,
        project_id: None,
        workflow_id: None,
        session_id: None,
        role_ids: vec![],
        document_refs: vec![],
        permission_policy_ref: Some("user_confirmed_mature_pattern".to_string()),
        model_export_policy: "allowed_with_redaction".to_string(),
        valid_from: timestamp.to_string(),
        valid_until: None,
    }
}

fn source_ref_from_finding(finding: &MemoryLintFinding, timestamp: &str) -> MemorySourceRef {
    MemorySourceRef {
        source_ref_id: format!(
            "src:mature-pattern-finding:{}",
            short_hash(&finding.finding_id)
        ),
        source_type: "maintenance_finding".to_string(),
        source_id: Some(finding.finding_id.clone()),
        source_path: None,
        source_title: Some(finding.summary.clone()),
        anchor: finding.claim.clone(),
        source_created_at: Some(finding.created_at.clone()),
        captured_at: timestamp.to_string(),
        authority_level: "project_director_review".to_string(),
        sensitive_level: "internal".to_string(),
        content_hash: Some(short_hash(&finding.summary)),
    }
}

fn source_ref_from_observation_source(
    source: &crate::ObservationSourceRef,
    timestamp: &str,
) -> MemorySourceRef {
    MemorySourceRef {
        source_ref_id: format!(
            "src:observation-source:{}",
            short_hash(&source.source_ref_id)
        ),
        source_type: source.source_kind.clone(),
        source_id: Some(source.source_id.clone()),
        source_path: source.file_path.clone(),
        source_title: Some(source.summary.clone()),
        anchor: source.evidence_ref.clone(),
        source_created_at: Some(source.created_at.clone()),
        captured_at: timestamp.to_string(),
        authority_level: "workflow_observation".to_string(),
        sensitive_level: source.sensitive_level.clone(),
        content_hash: Some(short_hash(&source.summary)),
    }
}

fn member_ref(
    member_kind: &str,
    member_id: &str,
    project_id: Option<String>,
    title: &str,
    source_refs: Vec<MemorySourceRef>,
) -> MemoryClusterMemberRef {
    MemoryClusterMemberRef {
        member_ref_id: format!(
            "cluster-member:v1:{}",
            short_hash(&format!("{}:{}:{}", member_kind, member_id, title))
        ),
        member_kind: member_kind.to_string(),
        member_id: member_id.to_string(),
        project_id,
        title: title.to_string(),
        source_refs,
    }
}

fn overlay_persisted_candidates(
    store: &MemoryPatternStoreV1,
    candidates: &mut Vec<MaturePatternCandidate>,
) {
    for persisted in &store.mature_pattern_candidates {
        if let Some(existing) = candidates
            .iter_mut()
            .find(|candidate| candidate.candidate_id == persisted.candidate_id)
        {
            *existing = persisted.clone();
        } else {
            candidates.push(persisted.clone());
        }
    }
}

fn overlay_persisted_reports(store: &MemoryPatternStoreV1, reports: &mut Vec<MemoryClusterReport>) {
    for persisted in &store.cluster_reports {
        if !reports
            .iter()
            .any(|report| report.report_id == persisted.report_id)
        {
            reports.push(persisted.clone());
        }
    }
}

fn push_candidate_decision(store: &mut MemoryPatternStoreV1, candidate: MaturePatternCandidate) {
    if let Some(index) = store
        .mature_pattern_candidates
        .iter()
        .position(|existing| existing.candidate_id == candidate.candidate_id)
    {
        store.mature_pattern_candidates[index] = candidate;
    } else {
        store.mature_pattern_candidates.push(candidate);
    }
}

fn dedupe_candidates(candidates: &mut Vec<MaturePatternCandidate>) {
    let mut seen = BTreeSet::new();
    candidates.retain(|candidate| seen.insert(candidate.candidate_id.clone()));
}

fn dedupe_reports(reports: &mut Vec<MemoryClusterReport>) {
    let mut seen = BTreeSet::new();
    reports.retain(|report| seen.insert(report.report_id.clone()));
}

fn dedupe_source_refs(source_refs: Vec<MemorySourceRef>) -> Vec<MemorySourceRef> {
    let mut seen = BTreeSet::new();
    let mut output = Vec::new();
    for source in source_refs {
        let key = format!(
            "{}:{}:{}",
            source.source_type,
            source.source_id.clone().unwrap_or_default(),
            source.source_title.clone().unwrap_or_default()
        );
        if seen.insert(key) {
            output.push(source);
        }
    }
    output
}

fn pattern_key(text: &str) -> String {
    normalize(text)
        .split_whitespace()
        .take(8)
        .collect::<Vec<_>>()
        .join(" ")
}

fn normalize(value: &str) -> String {
    value
        .trim()
        .to_lowercase()
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character.is_whitespace() {
                character
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn short_title(value: &str) -> String {
    value.chars().take(48).collect::<String>()
}

fn decision_name(decision: MaturePatternDecisionKind) -> &'static str {
    match decision {
        MaturePatternDecisionKind::ConfirmAsFormalMemory => "confirm_as_formal_memory",
        MaturePatternDecisionKind::Reject => "reject",
        MaturePatternDecisionKind::Quarantine => "quarantine",
        MaturePatternDecisionKind::RequestChanges => "request_changes",
    }
}

fn short_hash(value: &str) -> String {
    sha256_hex(value).chars().take(16).collect()
}

fn sha256_hex(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    format!("{:x}", hasher.finalize())
}
