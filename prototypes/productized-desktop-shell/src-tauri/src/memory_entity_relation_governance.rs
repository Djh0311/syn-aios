use crate::utils::hash::short_hash;
use crate::{
    MemoryEntity, MemoryEntityAlias, MemoryEntityAliasDecisionKind, MemoryEntityCandidate,
    MemoryEntityKind, MemoryEntityMergeCandidate, MemoryEntityMergeDecisionKind,
    MemoryEntityRelationPreviewOutput, MemoryEntityRelationStoreV1, MemoryRelation,
    MemoryRelationAuditEvent, MemoryRelationCandidate, MemoryRelationCandidateDecisionKind,
    MemoryRelationKind, MemoryRelationSource, MemoryRelationSourceKind, MemoryRelationStatus,
    PreviewMemoryEntityRelationCandidatesInput, RecordMemoryEntityAliasDecisionInput,
    RecordMemoryEntityAliasDecisionOutput, RecordMemoryEntityMergeDecisionInput,
    RecordMemoryEntityMergeDecisionOutput, RecordMemoryRelationCandidateDecisionInput,
    RecordMemoryRelationCandidateDecisionOutput,
};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub(crate) fn preview_candidates(
    workflow_state_path: &Path,
    input: &PreviewMemoryEntityRelationCandidatesInput,
    timestamp: &str,
) -> Result<MemoryEntityRelationPreviewOutput, String> {
    validate_preview_input(input)?;
    let store = crate::memory_entity_relation_store::load_store(workflow_state_path, timestamp)?;
    let derived = derive_candidates(workflow_state_path, input, timestamp, &store)?;
    Ok(MemoryEntityRelationPreviewOutput {
        store_revision: store.revision,
        entity_candidates: derived.entity_candidates,
        merge_candidates: derived.merge_candidates,
        relation_candidates: derived.relation_candidates,
        summary: crate::memory_entity_relation_store::summarize_store(&store),
        warnings: preview_warnings(&store),
    })
}

pub(crate) fn record_alias_decision(
    workflow_state_path: &Path,
    input: &RecordMemoryEntityAliasDecisionInput,
    timestamp: &str,
    write_id: &str,
) -> Result<RecordMemoryEntityAliasDecisionOutput, String> {
    validate_actor_can_decide(&input.actor_role)?;
    validate_non_empty("entity_candidate_id", &input.entity_candidate_id)?;
    validate_non_empty("reason", &input.reason)?;
    let preview_input = preview_input_from_project_root(&input.project_root);
    crate::memory_entity_relation_store::with_locked_store(
        workflow_state_path,
        timestamp,
        write_id,
        |store| {
            validate_expected_revision(input.expected_store_revision, store)?;
            let derived = derive_candidates(workflow_state_path, &preview_input, timestamp, store)?;
            let candidate = derived
                .entity_candidates
                .into_iter()
                .find(|candidate| candidate.candidate_id == input.entity_candidate_id)
                .ok_or_else(|| {
                    format!(
                        "找不到实体候选，无法记录 alias 决定：{}",
                        input.entity_candidate_id
                    )
                })?;
            let after_status = match input.decision {
                MemoryEntityAliasDecisionKind::ConfirmAlias => MemoryRelationStatus::Confirmed,
                MemoryEntityAliasDecisionKind::RejectAlias => MemoryRelationStatus::Rejected,
            };
            let mut decided_candidate = candidate.clone();
            decided_candidate.status = after_status;
            let entity = if input.decision == MemoryEntityAliasDecisionKind::ConfirmAlias {
                Some(upsert_entity_from_candidate(
                    store, &candidate, timestamp, None,
                ))
            } else {
                None
            };
            push_entity_candidate_decision(store, decided_candidate.clone());
            let audit_event = audit_event(
                "memory_entity_alias_decision_recorded",
                &input.actor_id,
                &input.actor_role,
                "memory_entity_candidate",
                &candidate.candidate_id,
                Some(MemoryRelationStatus::Candidate),
                after_status,
                &input.reason,
                timestamp,
                vec!["entity_candidate_decision_only".to_string()],
            );
            store.audit_events.push(audit_event.clone());
            store.project_id = Some(crate::project_id(&input.project_root));
            store.workflow_id = Some(crate::default_workflow_id(&input.project_root));
            store.registry.updated_at = timestamp.to_string();
            store.revision += 1;
            Ok(RecordMemoryEntityAliasDecisionOutput {
                store_revision: store.revision,
                entity,
                candidate: decided_candidate,
                audit_event,
                warnings: vec!["entity_alias_decision_does_not_modify_formal_memory".to_string()],
            })
        },
    )
}

pub(crate) fn record_merge_decision(
    workflow_state_path: &Path,
    input: &RecordMemoryEntityMergeDecisionInput,
    timestamp: &str,
    write_id: &str,
) -> Result<RecordMemoryEntityMergeDecisionOutput, String> {
    validate_actor_can_decide(&input.actor_role)?;
    validate_non_empty("merge_candidate_id", &input.merge_candidate_id)?;
    validate_non_empty("reason", &input.reason)?;
    let preview_input = preview_input_from_project_root(&input.project_root);
    crate::memory_entity_relation_store::with_locked_store(
        workflow_state_path,
        timestamp,
        write_id,
        |store| {
            validate_expected_revision(input.expected_store_revision, store)?;
            let derived = derive_candidates(workflow_state_path, &preview_input, timestamp, store)?;
            let merge_candidate = derived
                .merge_candidates
                .into_iter()
                .find(|candidate| candidate.merge_candidate_id == input.merge_candidate_id)
                .ok_or_else(|| {
                    format!(
                        "找不到 entity merge 候选，无法记录决定：{}",
                        input.merge_candidate_id
                    )
                })?;
            if input.decision == MemoryEntityMergeDecisionKind::ConfirmMerge {
                validate_confirmation_actor(
                    input.confirmed_by.as_deref(),
                    &input.actor_role,
                    merge_candidate.requires_user_confirmation,
                )?;
            }
            let after_status = match input.decision {
                MemoryEntityMergeDecisionKind::ConfirmMerge => MemoryRelationStatus::Confirmed,
                MemoryEntityMergeDecisionKind::RejectMerge => MemoryRelationStatus::Rejected,
            };
            let mut decided_candidate = merge_candidate.clone();
            decided_candidate.status = after_status;
            let entity = if input.decision == MemoryEntityMergeDecisionKind::ConfirmMerge {
                let left = derived
                    .entity_candidates
                    .iter()
                    .find(|candidate| {
                        candidate.candidate_id == merge_candidate.left_entity_candidate_id
                    })
                    .ok_or_else(|| "merge 候选缺少 left entity candidate".to_string())?;
                let right = derived
                    .entity_candidates
                    .iter()
                    .find(|candidate| {
                        candidate.candidate_id == merge_candidate.right_entity_candidate_id
                    })
                    .ok_or_else(|| "merge 候选缺少 right entity candidate".to_string())?;
                let entity = upsert_entity_from_candidate(
                    store,
                    left,
                    timestamp,
                    Some(merge_candidate.normalized_key.clone()),
                );
                let merged = upsert_entity_from_candidate(
                    store,
                    right,
                    timestamp,
                    Some(merge_candidate.normalized_key.clone()),
                );
                Some(if merged.entity_id == entity.entity_id {
                    merged
                } else {
                    entity
                })
            } else {
                None
            };
            push_merge_candidate_decision(store, decided_candidate.clone());
            let audit_event = audit_event(
                "memory_entity_merge_decision_recorded",
                &input.actor_id,
                &input.actor_role,
                "memory_entity_merge_candidate",
                &merge_candidate.merge_candidate_id,
                Some(MemoryRelationStatus::Candidate),
                after_status,
                &input.reason,
                timestamp,
                vec![
                    "entity_merge_decision_does_not_modify_formal_memory".to_string(),
                    "similarity_hit_only_candidate_until_confirmed".to_string(),
                ],
            );
            store.audit_events.push(audit_event.clone());
            store.project_id = Some(crate::project_id(&input.project_root));
            store.workflow_id = Some(crate::default_workflow_id(&input.project_root));
            store.registry.updated_at = timestamp.to_string();
            store.revision += 1;
            Ok(RecordMemoryEntityMergeDecisionOutput {
                store_revision: store.revision,
                entity,
                merge_candidate: decided_candidate,
                audit_event,
                warnings: vec![
                    "entity_merge_decision_does_not_modify_formal_memory".to_string(),
                    "similarity_hit_only_candidate_until_confirmed".to_string(),
                ],
            })
        },
    )
}

pub(crate) fn record_relation_decision(
    workflow_state_path: &Path,
    input: &RecordMemoryRelationCandidateDecisionInput,
    timestamp: &str,
    write_id: &str,
) -> Result<RecordMemoryRelationCandidateDecisionOutput, String> {
    validate_actor_can_decide(&input.actor_role)?;
    validate_non_empty("relation_candidate_id", &input.relation_candidate_id)?;
    validate_non_empty("reason", &input.reason)?;
    let preview_input = preview_input_from_project_root(&input.project_root);
    crate::memory_entity_relation_store::with_locked_store(
        workflow_state_path,
        timestamp,
        write_id,
        |store| {
            validate_expected_revision(input.expected_store_revision, store)?;
            let derived = derive_candidates(workflow_state_path, &preview_input, timestamp, store)?;
            let relation_candidate = derived
                .relation_candidates
                .into_iter()
                .find(|candidate| candidate.candidate_id == input.relation_candidate_id)
                .ok_or_else(|| {
                    format!(
                        "找不到关系候选，无法记录决定：{}",
                        input.relation_candidate_id
                    )
                })?;
            let after_status = match input.decision {
                MemoryRelationCandidateDecisionKind::ConfirmRelation => {
                    validate_relation_confirmation(input, &relation_candidate)?;
                    MemoryRelationStatus::Confirmed
                }
                MemoryRelationCandidateDecisionKind::RejectRelation => {
                    MemoryRelationStatus::Rejected
                }
                MemoryRelationCandidateDecisionKind::QuarantineRelation => {
                    MemoryRelationStatus::Quarantined
                }
            };
            let mut decided_candidate = relation_candidate.clone();
            decided_candidate.status = after_status;
            let relation = if input.decision == MemoryRelationCandidateDecisionKind::ConfirmRelation
            {
                let relation = relation_from_candidate(
                    &relation_candidate,
                    input.confirmed_by.as_deref().unwrap_or(&input.actor_role),
                    &input.actor_role,
                    &input.reason,
                    timestamp,
                );
                if !store
                    .relations
                    .iter()
                    .any(|existing| existing.relation_id == relation.relation_id)
                {
                    store.relations.push(relation.clone());
                }
                Some(relation)
            } else {
                None
            };
            push_relation_candidate_decision(store, decided_candidate.clone());
            let audit_event = audit_event(
                "memory_relation_candidate_decision_recorded",
                &input.actor_id,
                &input.actor_role,
                "memory_relation_candidate",
                &relation_candidate.candidate_id,
                Some(MemoryRelationStatus::Candidate),
                after_status,
                &input.reason,
                timestamp,
                vec![
                    "relation_candidate_decision_does_not_modify_formal_memory".to_string(),
                    "confirmed_relations_only_explain_task_packet_retrieval".to_string(),
                ],
            );
            store.audit_events.push(audit_event.clone());
            store.project_id = Some(crate::project_id(&input.project_root));
            store.workflow_id = Some(crate::default_workflow_id(&input.project_root));
            store.revision += 1;
            Ok(RecordMemoryRelationCandidateDecisionOutput {
                store_revision: store.revision,
                relation,
                relation_candidate: decided_candidate,
                audit_event,
                warnings: vec![
                    "relation_candidate_decision_does_not_modify_formal_memory".to_string(),
                    "confirmed_relations_only_explain_task_packet_retrieval".to_string(),
                ],
            })
        },
    )
}

struct DerivedCandidates {
    entity_candidates: Vec<MemoryEntityCandidate>,
    merge_candidates: Vec<MemoryEntityMergeCandidate>,
    relation_candidates: Vec<MemoryRelationCandidate>,
}

fn derive_candidates(
    workflow_state_path: &Path,
    input: &PreviewMemoryEntityRelationCandidatesInput,
    timestamp: &str,
    store: &MemoryEntityRelationStoreV1,
) -> Result<DerivedCandidates, String> {
    let formal_store = crate::formal_memory_store::load_store(workflow_state_path, timestamp)?;
    let candidate_store =
        crate::memory_candidate_store::load_store(workflow_state_path, timestamp)?;
    let observation_store = crate::observation_store::load_store(workflow_state_path, timestamp)?;
    let project_id = input
        .project_id
        .clone()
        .unwrap_or_else(|| crate::project_id(&input.project_root));
    let workflow_id = input
        .workflow_id
        .clone()
        .unwrap_or_else(|| crate::default_workflow_id(&input.project_root));
    let mut entity_candidates = Vec::<MemoryEntityCandidate>::new();
    push_entity_candidate(
        &mut entity_candidates,
        entity_candidate(
            MemoryEntityKind::Project,
            &input.project_root,
            MemoryRelationSourceKind::Manual,
            Some(project_id.clone()),
            Some(input.project_root.clone()),
            Some(input.project_root.clone()),
            vec![manual_relation_source(
                Some(project_id.clone()),
                Some(input.project_root.clone()),
                Some(input.project_root.clone()),
            )],
            "project_root_explicit",
            "项目 root 派生项目实体候选。",
            timestamp,
        ),
    );
    push_entity_candidate(
        &mut entity_candidates,
        entity_candidate(
            MemoryEntityKind::Workflow,
            &workflow_id,
            MemoryRelationSourceKind::Manual,
            Some(workflow_id.clone()),
            None,
            Some(workflow_id.clone()),
            vec![manual_relation_source(
                Some(workflow_id.clone()),
                None,
                Some(workflow_id.clone()),
            )],
            "workflow_id_explicit",
            "workflow id 派生工作流实体候选。",
            timestamp,
        ),
    );

    for record in &formal_store.records {
        let memory_source = MemoryRelationSource {
            source_kind: MemoryRelationSourceKind::FormalMemory,
            source_id: Some(record.memory_id.clone()),
            source_path: None,
            source_title: Some(record.claim.clone()),
            authority_level: "formal_memory".to_string(),
            sensitive_level: source_sensitive_level(&record.source_refs),
        };
        push_entity_candidate(
            &mut entity_candidates,
            entity_candidate(
                MemoryEntityKind::MemoryRecord,
                &record.claim,
                MemoryRelationSourceKind::FormalMemory,
                Some(record.memory_id.clone()),
                None,
                Some(record.claim.clone()),
                vec![memory_source],
                "formal_memory_record",
                "正式记忆 record 派生实体候选。",
                timestamp,
            ),
        );
        for source in &record.source_refs {
            if let Some(candidate) = entity_candidate_from_memory_source(source, timestamp) {
                push_entity_candidate(&mut entity_candidates, candidate);
            }
        }
    }

    for candidate in &candidate_store.candidates {
        push_entity_candidate(
            &mut entity_candidates,
            entity_candidate(
                MemoryEntityKind::MemoryCandidate,
                &candidate.claim,
                MemoryRelationSourceKind::MemoryCandidate,
                Some(candidate.candidate_key.clone()),
                None,
                Some(candidate.claim.clone()),
                vec![MemoryRelationSource {
                    source_kind: MemoryRelationSourceKind::MemoryCandidate,
                    source_id: Some(candidate.candidate_key.clone()),
                    source_path: None,
                    source_title: Some(candidate.claim.clone()),
                    authority_level: "memory_candidate".to_string(),
                    sensitive_level: candidate.sensitive_level.clone(),
                }],
                "memory_candidate_record",
                "记忆候选派生实体候选；候选不是正式记忆。",
                timestamp,
            ),
        );
        for source in &candidate.source_refs {
            if let Some(entity_candidate) = entity_candidate_from_memory_source(source, timestamp) {
                push_entity_candidate(&mut entity_candidates, entity_candidate);
            }
        }
    }

    for observation in &observation_store.observations {
        for source in &observation.source_refs {
            if let Some(entity_candidate) =
                entity_candidate_from_observation_source(source, timestamp)
            {
                push_entity_candidate(&mut entity_candidates, entity_candidate);
            }
        }
    }

    let merge_candidates = derive_merge_candidates(&entity_candidates, timestamp);
    let relation_candidates = derive_relation_candidates(&formal_store.records, timestamp);
    let decided_entity_ids = decided_entity_candidate_ids(store);
    let decided_merge_ids = decided_merge_candidate_ids(store);
    let decided_relation_ids = decided_relation_candidate_ids(store);

    Ok(DerivedCandidates {
        entity_candidates: entity_candidates
            .into_iter()
            .filter(|candidate| !decided_entity_ids.contains(&candidate.candidate_id))
            .collect(),
        merge_candidates: merge_candidates
            .into_iter()
            .filter(|candidate| !decided_merge_ids.contains(&candidate.merge_candidate_id))
            .collect(),
        relation_candidates: relation_candidates
            .into_iter()
            .filter(|candidate| !decided_relation_ids.contains(&candidate.candidate_id))
            .collect(),
    })
}

fn derive_merge_candidates(
    entity_candidates: &[MemoryEntityCandidate],
    timestamp: &str,
) -> Vec<MemoryEntityMergeCandidate> {
    let mut groups = BTreeMap::<String, Vec<&MemoryEntityCandidate>>::new();
    for candidate in entity_candidates {
        let key = format!(
            "{}:{}",
            entity_kind_name(candidate.entity_kind),
            alias_key(&candidate.display_name)
        );
        if alias_key(&candidate.display_name).chars().count() >= 3 {
            groups.entry(key).or_default().push(candidate);
        }
    }
    let mut output = Vec::new();
    for candidates in groups.values() {
        if candidates.len() < 2 {
            continue;
        }
        for pair in candidates.windows(2) {
            let left = pair[0];
            let right = pair[1];
            if left.candidate_id == right.candidate_id {
                continue;
            }
            let source_kind = if left.source_kind == MemoryRelationSourceKind::SimilarityHit
                || right.source_kind == MemoryRelationSourceKind::SimilarityHit
            {
                MemoryRelationSourceKind::SimilarityHit
            } else {
                left.source_kind
            };
            output.push(MemoryEntityMergeCandidate {
                merge_candidate_id: format!(
                    "entity-merge-candidate:v1:{}",
                    short_hash(&format!("{}:{}", left.candidate_id, right.candidate_id))
                ),
                left_entity_candidate_id: left.candidate_id.clone(),
                right_entity_candidate_id: right.candidate_id.clone(),
                left_label: left.display_name.clone(),
                right_label: right.display_name.clone(),
                normalized_key: alias_key(&left.display_name),
                source_kind,
                status: MemoryRelationStatus::Candidate,
                requires_user_confirmation: false,
                reason: if source_kind == MemoryRelationSourceKind::SimilarityHit {
                    "相似度命中仅作候选，需人工确认后才会登记实体合并决定。".to_string()
                } else {
                    "同一规范化名称出现多个实体候选，需人工确认 alias / dedupe。".to_string()
                },
                created_at: timestamp.to_string(),
                warnings: vec!["merge_candidate_does_not_mutate_entities".to_string()],
            });
        }
    }
    output
}

fn derive_relation_candidates(
    records: &[crate::MemoryRecord],
    timestamp: &str,
) -> Vec<MemoryRelationCandidate> {
    let mut output = Vec::new();
    for record in records {
        let subject_entity_id = entity_id_for(
            MemoryEntityKind::MemoryRecord,
            &canonical_key_for(
                MemoryEntityKind::MemoryRecord,
                &record.memory_id,
                &record.claim,
            ),
        );
        for source in &record.source_refs {
            let Some(object_candidate) = entity_candidate_from_memory_source(source, timestamp)
            else {
                continue;
            };
            let source_kind = relation_source_kind_from_memory_source(source);
            let relation_kind = relation_kind_for_record_source(record, source_kind);
            let requires_user_confirmation = source_kind == MemoryRelationSourceKind::LlmInferred;
            let reason = relation_candidate_reason(relation_kind, source_kind);
            output.push(MemoryRelationCandidate {
                candidate_id: format!(
                    "relation-candidate:v1:{}",
                    short_hash(&format!(
                        "{}:{}:{}:{}",
                        record.memory_id,
                        object_candidate.normalized_key,
                        relation_kind_name(relation_kind),
                        source_kind_name(source_kind)
                    ))
                ),
                relation_kind,
                subject_entity_id: subject_entity_id.clone(),
                object_entity_id: entity_id_for(
                    object_candidate.entity_kind,
                    &object_candidate.normalized_key,
                ),
                subject_label: record.claim.clone(),
                object_label: object_candidate.display_name,
                predicate: predicate_for_relation(relation_kind, source_kind).to_string(),
                source_kind,
                source_refs: vec![relation_source_from_memory_source(source, source_kind)],
                confidence_kind: if source_kind == MemoryRelationSourceKind::LlmInferred {
                    "llm_inferred_candidate_only".to_string()
                } else {
                    "deterministic_source_ref".to_string()
                },
                status: MemoryRelationStatus::Candidate,
                requires_user_confirmation,
                reason,
                created_at: timestamp.to_string(),
                warnings: vec!["relation_candidate_not_confirmed_fact".to_string()],
            });
        }
    }
    dedupe_relation_candidates(output)
}

fn entity_candidate_from_memory_source(
    source: &crate::MemorySourceRef,
    timestamp: &str,
) -> Option<MemoryEntityCandidate> {
    let source_kind = relation_source_kind_from_memory_source(source);
    let entity_kind = entity_kind_from_memory_source(source)?;
    let display_name = source
        .source_title
        .as_deref()
        .or(source.source_id.as_deref())
        .or(source.source_path.as_deref())?
        .trim();
    if display_name.is_empty() {
        return None;
    }
    Some(entity_candidate(
        entity_kind,
        display_name,
        source_kind,
        source.source_id.clone(),
        source.source_path.clone(),
        source.source_title.clone(),
        vec![relation_source_from_memory_source(source, source_kind)],
        "source_ref",
        "来源引用派生实体候选。",
        timestamp,
    ))
}

fn entity_candidate_from_observation_source(
    source: &crate::ObservationSourceRef,
    timestamp: &str,
) -> Option<MemoryEntityCandidate> {
    let entity_kind = if source.source_kind == "task_package" {
        MemoryEntityKind::Proposal
    } else if source.source_kind == "workflow_event" {
        MemoryEntityKind::Workflow
    } else {
        return None;
    };
    let display_name = if source.summary.trim().is_empty() {
        source.source_id.as_str()
    } else {
        source.summary.as_str()
    };
    Some(entity_candidate(
        entity_kind,
        display_name,
        MemoryRelationSourceKind::Observation,
        Some(source.source_id.clone()),
        source.file_path.clone(),
        Some(source.summary.clone()),
        vec![MemoryRelationSource {
            source_kind: MemoryRelationSourceKind::Observation,
            source_id: Some(source.source_id.clone()),
            source_path: source.file_path.clone(),
            source_title: Some(source.summary.clone()),
            authority_level: "observation".to_string(),
            sensitive_level: source.sensitive_level.clone(),
        }],
        "observation_source_ref",
        "observation 来源派生实体候选。",
        timestamp,
    ))
}

fn entity_candidate(
    entity_kind: MemoryEntityKind,
    display_name: &str,
    source_kind: MemoryRelationSourceKind,
    source_id: Option<String>,
    source_path: Option<String>,
    source_title: Option<String>,
    source_refs: Vec<MemoryRelationSource>,
    confidence_kind: &str,
    reason: &str,
    timestamp: &str,
) -> MemoryEntityCandidate {
    let normalized_key = canonical_key_for(
        entity_kind,
        source_id
            .as_deref()
            .or(source_path.as_deref())
            .unwrap_or(display_name),
        display_name,
    );
    MemoryEntityCandidate {
        candidate_id: format!(
            "entity-candidate:v1:{}",
            short_hash(&format!(
                "{}:{}:{}:{}",
                entity_kind_name(entity_kind),
                normalized_key,
                source_kind_name(source_kind),
                source_id.clone().unwrap_or_default()
            ))
        ),
        entity_kind,
        display_name: display_name.trim().to_string(),
        normalized_key,
        source_kind,
        source_id,
        source_path,
        source_title,
        source_refs,
        confidence_kind: confidence_kind.to_string(),
        status: MemoryRelationStatus::Candidate,
        reason: reason.to_string(),
        created_at: timestamp.to_string(),
        warnings: vec!["entity_candidate_requires_manual_decision".to_string()],
    }
}

fn upsert_entity_from_candidate(
    store: &mut MemoryEntityRelationStoreV1,
    candidate: &MemoryEntityCandidate,
    timestamp: &str,
    override_key: Option<String>,
) -> MemoryEntity {
    let canonical_key = override_key.unwrap_or_else(|| candidate.normalized_key.clone());
    let entity_id = entity_id_for(candidate.entity_kind, &canonical_key);
    let alias = MemoryEntityAlias {
        alias_id: format!(
            "entity-alias:v1:{}",
            short_hash(&format!("{}:{}", entity_id, candidate.display_name))
        ),
        alias: candidate.display_name.clone(),
        source_kind: candidate.source_kind,
        source_id: candidate.source_id.clone(),
        created_at: timestamp.to_string(),
    };
    if let Some(index) = store
        .registry
        .entities
        .iter()
        .position(|entity| entity.entity_id == entity_id)
    {
        let entity = &mut store.registry.entities[index];
        if !entity
            .aliases
            .iter()
            .any(|existing| existing.alias == alias.alias)
        {
            entity.aliases.push(alias);
        }
        for source in &candidate.source_refs {
            if !entity
                .source_refs
                .iter()
                .any(|existing| existing.source_id == source.source_id)
            {
                entity.source_refs.push(source.clone());
            }
        }
        entity.updated_at = timestamp.to_string();
        return entity.clone();
    }
    let entity = MemoryEntity {
        entity_id,
        entity_kind: candidate.entity_kind,
        canonical_key,
        display_name: candidate.display_name.clone(),
        aliases: vec![alias],
        source_refs: candidate.source_refs.clone(),
        status: "registered".to_string(),
        created_at: timestamp.to_string(),
        updated_at: timestamp.to_string(),
        warnings: vec!["memory_entity_registry_minimal".to_string()],
    };
    store.registry.entities.push(entity.clone());
    entity
}

fn relation_from_candidate(
    candidate: &MemoryRelationCandidate,
    confirmed_by: &str,
    confirmation_role: &str,
    reason: &str,
    timestamp: &str,
) -> MemoryRelation {
    MemoryRelation {
        relation_id: format!(
            "relation:v1:{}",
            short_hash(&format!(
                "{}:{}:{}:{}",
                candidate.subject_entity_id,
                candidate.object_entity_id,
                relation_kind_name(candidate.relation_kind),
                candidate.predicate
            ))
        ),
        relation_kind: candidate.relation_kind,
        subject_entity_id: candidate.subject_entity_id.clone(),
        object_entity_id: candidate.object_entity_id.clone(),
        subject_label: candidate.subject_label.clone(),
        object_label: candidate.object_label.clone(),
        predicate: candidate.predicate.clone(),
        source_kind: candidate.source_kind,
        source_refs: candidate.source_refs.clone(),
        status: MemoryRelationStatus::Confirmed,
        confirmed_by: confirmed_by.to_string(),
        confirmation_role: confirmation_role.to_string(),
        confirmation_reason: reason.to_string(),
        created_at: timestamp.to_string(),
        updated_at: timestamp.to_string(),
        warnings: vec![
            "confirmed_relation_explains_retrieval_only".to_string(),
            "relation_store_does_not_override_formal_memory".to_string(),
        ],
    }
}

fn validate_relation_confirmation(
    input: &RecordMemoryRelationCandidateDecisionInput,
    candidate: &MemoryRelationCandidate,
) -> Result<(), String> {
    if candidate.source_kind == MemoryRelationSourceKind::LlmInferred {
        return Err(
            "llm_inferred relation 只能保留为关系候选，不能直接写 confirmed relation".to_string(),
        );
    }
    if candidate.source_kind == MemoryRelationSourceKind::SimilarityHit {
        return Err(
            "similarity_hit relation 只能保留为候选，不能直接写 confirmed relation".to_string(),
        );
    }
    validate_confirmation_actor(
        input.confirmed_by.as_deref(),
        &input.actor_role,
        candidate.requires_user_confirmation,
    )?;
    if candidate.relation_kind == MemoryRelationKind::Causal
        && !matches!(
            input.confirmed_by.as_deref().unwrap_or(&input.actor_role),
            "project_director" | "user"
        )
    {
        return Err("causal relation 需要项目主管或用户确认".to_string());
    }
    Ok(())
}

fn validate_confirmation_actor(
    confirmed_by: Option<&str>,
    actor_role: &str,
    requires_user: bool,
) -> Result<(), String> {
    let confirmer = confirmed_by.unwrap_or(actor_role);
    if requires_user && confirmer != "user" {
        return Err("该实体 / 关系决定需要用户确认".to_string());
    }
    if !matches!(confirmer, "project_director" | "user") {
        return Err("实体 / 关系决定只能由项目主管或用户确认".to_string());
    }
    Ok(())
}

fn validate_actor_can_decide(actor_role: &str) -> Result<(), String> {
    if actor_role == "secretary" {
        return Err("秘书不能确认实体合并、因果关系或正式关系".to_string());
    }
    if matches!(actor_role, "project_director" | "user" | "global_director") {
        return Ok(());
    }
    Err(format!("当前角色不能记录实体 / 关系治理决定：{actor_role}"))
}

fn validate_preview_input(
    input: &PreviewMemoryEntityRelationCandidatesInput,
) -> Result<(), String> {
    validate_non_empty("project_root", &input.project_root)?;
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

fn validate_context_field(
    field_name: &str,
    actual: Option<&str>,
    expected: &str,
) -> Result<(), String> {
    if let Some(actual) = actual {
        if actual.trim() != expected {
            return Err(format!(
                "实体 / 关系上下文绑定失败：{field_name} 与 project_root 不匹配，expected {expected}，actual {}",
                actual.trim()
            ));
        }
    }
    Ok(())
}

fn validate_expected_revision(
    expected: Option<i64>,
    store: &MemoryEntityRelationStoreV1,
) -> Result<(), String> {
    if let Some(expected) = expected {
        if expected != store.revision {
            return Err(format!(
                "memory_entity_relation_store_conflict: expected revision {expected}, actual {}",
                store.revision
            ));
        }
    }
    Ok(())
}

fn validate_non_empty(label: &str, value: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err(format!("实体 / 关系治理缺少 {label}"));
    }
    Ok(())
}

fn push_entity_candidate(
    candidates: &mut Vec<MemoryEntityCandidate>,
    candidate: MemoryEntityCandidate,
) {
    if !candidates
        .iter()
        .any(|existing| existing.candidate_id == candidate.candidate_id)
    {
        candidates.push(candidate);
    }
}

fn push_entity_candidate_decision(
    store: &mut MemoryEntityRelationStoreV1,
    candidate: MemoryEntityCandidate,
) {
    store
        .entity_candidates
        .retain(|existing| existing.candidate_id != candidate.candidate_id);
    store.entity_candidates.push(candidate);
}

fn push_merge_candidate_decision(
    store: &mut MemoryEntityRelationStoreV1,
    candidate: MemoryEntityMergeCandidate,
) {
    store
        .merge_candidates
        .retain(|existing| existing.merge_candidate_id != candidate.merge_candidate_id);
    store.merge_candidates.push(candidate);
}

fn push_relation_candidate_decision(
    store: &mut MemoryEntityRelationStoreV1,
    candidate: MemoryRelationCandidate,
) {
    store
        .relation_candidates
        .retain(|existing| existing.candidate_id != candidate.candidate_id);
    store.relation_candidates.push(candidate);
}

fn decided_entity_candidate_ids(store: &MemoryEntityRelationStoreV1) -> BTreeSet<String> {
    store
        .entity_candidates
        .iter()
        .filter(|candidate| candidate.status != MemoryRelationStatus::Candidate)
        .map(|candidate| candidate.candidate_id.clone())
        .collect()
}

fn decided_merge_candidate_ids(store: &MemoryEntityRelationStoreV1) -> BTreeSet<String> {
    store
        .merge_candidates
        .iter()
        .filter(|candidate| candidate.status != MemoryRelationStatus::Candidate)
        .map(|candidate| candidate.merge_candidate_id.clone())
        .collect()
}

fn decided_relation_candidate_ids(store: &MemoryEntityRelationStoreV1) -> BTreeSet<String> {
    store
        .relation_candidates
        .iter()
        .filter(|candidate| candidate.status != MemoryRelationStatus::Candidate)
        .map(|candidate| candidate.candidate_id.clone())
        .collect()
}

fn dedupe_relation_candidates(
    candidates: Vec<MemoryRelationCandidate>,
) -> Vec<MemoryRelationCandidate> {
    let mut seen = BTreeSet::new();
    let mut output = Vec::new();
    for candidate in candidates {
        if seen.insert(candidate.candidate_id.clone()) {
            output.push(candidate);
        }
    }
    output
}

fn preview_input_from_project_root(
    project_root: &str,
) -> PreviewMemoryEntityRelationCandidatesInput {
    PreviewMemoryEntityRelationCandidatesInput {
        project_root: project_root.to_string(),
        project_id: Some(crate::project_id(project_root)),
        workflow_id: Some(crate::default_workflow_id(project_root)),
    }
}

fn entity_kind_from_memory_source(source: &crate::MemorySourceRef) -> Option<MemoryEntityKind> {
    match source.source_type.as_str() {
        "knowledge_doc" => Some(MemoryEntityKind::KnowledgeDoc),
        "user_confirmed_proposal" => Some(MemoryEntityKind::Proposal),
        "session_summary" => Some(MemoryEntityKind::Session),
        "workflow_summary" => Some(MemoryEntityKind::Workflow),
        "tool" => Some(MemoryEntityKind::Tool),
        "model" => Some(MemoryEntityKind::Model),
        "harness" => Some(MemoryEntityKind::Harness),
        "similarity_hit" => Some(MemoryEntityKind::MemoryRecord),
        "llm_inferred" => Some(MemoryEntityKind::Proposal),
        "manual_note" | "evidence" | "handoff" | "stage_report" | "director_review" => {
            Some(MemoryEntityKind::KnowledgeDoc)
        }
        _ => None,
    }
}

fn relation_source_kind_from_memory_source(
    source: &crate::MemorySourceRef,
) -> MemoryRelationSourceKind {
    match source.source_type.as_str() {
        "knowledge_doc" => MemoryRelationSourceKind::KnowledgeDoc,
        "task_package" => MemoryRelationSourceKind::TaskPackage,
        "memory_candidate" => MemoryRelationSourceKind::MemoryCandidate,
        "observation_ref" => MemoryRelationSourceKind::Observation,
        "llm_inferred" => MemoryRelationSourceKind::LlmInferred,
        "similarity_hit" => MemoryRelationSourceKind::SimilarityHit,
        "manual_note" => MemoryRelationSourceKind::Manual,
        _ => MemoryRelationSourceKind::FormalMemory,
    }
}

fn relation_kind_for_record_source(
    record: &crate::MemoryRecord,
    source_kind: MemoryRelationSourceKind,
) -> MemoryRelationKind {
    if source_kind == MemoryRelationSourceKind::LlmInferred {
        return MemoryRelationKind::Causal;
    }
    if source_kind == MemoryRelationSourceKind::SimilarityHit {
        return MemoryRelationKind::Semantic;
    }
    let text = normalize(&format!("{} {}", record.claim, record.body));
    if text.contains("导致")
        || text.contains("因果")
        || text.contains("because")
        || text.contains("causal")
        || text.contains("causes")
    {
        return MemoryRelationKind::Causal;
    }
    MemoryRelationKind::Semantic
}

fn relation_candidate_reason(
    relation_kind: MemoryRelationKind,
    source_kind: MemoryRelationSourceKind,
) -> String {
    if source_kind == MemoryRelationSourceKind::LlmInferred {
        return "LLM 推断仅作候选，不能直接进入已确认关系。".to_string();
    }
    if source_kind == MemoryRelationSourceKind::SimilarityHit {
        return "相似度命中仅作候选，不会自行合并实体或确认关系。".to_string();
    }
    if relation_kind == MemoryRelationKind::Causal {
        return "待确认因果关系；确认后才可用于解释召回原因。".to_string();
    }
    "来源引用派生关系候选；确认后才可用于解释召回原因。".to_string()
}

fn predicate_for_relation(
    relation_kind: MemoryRelationKind,
    source_kind: MemoryRelationSourceKind,
) -> &'static str {
    if source_kind == MemoryRelationSourceKind::LlmInferred {
        return "llm_inferred_candidate";
    }
    match relation_kind {
        MemoryRelationKind::Causal => "causal_candidate",
        MemoryRelationKind::Temporal => "temporal_candidate",
        MemoryRelationKind::Entity => "entity_reference",
        MemoryRelationKind::Semantic => "semantic_reference",
    }
}

fn relation_source_from_memory_source(
    source: &crate::MemorySourceRef,
    source_kind: MemoryRelationSourceKind,
) -> MemoryRelationSource {
    MemoryRelationSource {
        source_kind,
        source_id: source.source_id.clone(),
        source_path: source.source_path.clone(),
        source_title: source.source_title.clone(),
        authority_level: source.authority_level.clone(),
        sensitive_level: source.sensitive_level.clone(),
    }
}

fn manual_relation_source(
    source_id: Option<String>,
    source_path: Option<String>,
    source_title: Option<String>,
) -> MemoryRelationSource {
    MemoryRelationSource {
        source_kind: MemoryRelationSourceKind::Manual,
        source_id,
        source_path,
        source_title,
        authority_level: "manual".to_string(),
        sensitive_level: "project".to_string(),
    }
}

fn source_sensitive_level(source_refs: &[crate::MemorySourceRef]) -> String {
    if source_refs
        .iter()
        .any(|source| source.sensitive_level == "secret")
    {
        "secret".to_string()
    } else if source_refs
        .iter()
        .any(|source| source.sensitive_level == "private")
    {
        "private".to_string()
    } else {
        "project".to_string()
    }
}

fn audit_event(
    event_type: &str,
    actor_id: &str,
    actor_role: &str,
    target_kind: &str,
    target_id: &str,
    before_status: Option<MemoryRelationStatus>,
    after_status: MemoryRelationStatus,
    reason: &str,
    timestamp: &str,
    warnings: Vec<String>,
) -> MemoryRelationAuditEvent {
    MemoryRelationAuditEvent {
        audit_event_id: format!(
            "audit:memory-relation:v1:{}:{}",
            timestamp,
            short_hash(&format!("{event_type}:{target_id}:{reason}"))
        ),
        event_type: event_type.to_string(),
        actor_id: actor_id.to_string(),
        actor_role: actor_role.to_string(),
        target_kind: target_kind.to_string(),
        target_id: target_id.to_string(),
        before_status,
        after_status: Some(after_status),
        reason: reason.trim().to_string(),
        created_at: timestamp.to_string(),
        warnings,
    }
}

fn preview_warnings(store: &MemoryEntityRelationStoreV1) -> Vec<String> {
    let mut warnings = store.warnings.clone();
    warnings.push("preview_only_no_entity_relation_sidecar_write".to_string());
    warnings.push("llm_inferred_relation_candidate_only".to_string());
    warnings.push("similarity_hit_candidate_only".to_string());
    dedupe(warnings)
}

fn canonical_key_for(
    entity_kind: MemoryEntityKind,
    source_key: &str,
    display_name: &str,
) -> String {
    let base = if source_key.trim().is_empty() {
        display_name
    } else {
        source_key
    };
    format!("{}:{}", entity_kind_name(entity_kind), alias_key(base))
}

fn entity_id_for(entity_kind: MemoryEntityKind, canonical_key: &str) -> String {
    format!(
        "entity:v1:{}:{}",
        entity_kind_name(entity_kind),
        short_hash(canonical_key)
    )
}

fn alias_key(value: &str) -> String {
    let mut normalized = normalize(value)
        .replace("codex cli", "codex")
        .replace(" cli", "")
        .replace(" tool", "")
        .replace("工具", "")
        .replace("模型", "model");
    normalized.retain(|character| character.is_alphanumeric() || !character.is_ascii());
    normalized
}

fn normalize(value: &str) -> String {
    value.trim().replace('\\', "/").to_lowercase()
}

fn dedupe(values: Vec<String>) -> Vec<String> {
    let mut output = Vec::new();
    for value in values {
        if !output.contains(&value) {
            output.push(value);
        }
    }
    output
}

fn entity_kind_name(kind: MemoryEntityKind) -> &'static str {
    match kind {
        MemoryEntityKind::Project => "project",
        MemoryEntityKind::Workflow => "workflow",
        MemoryEntityKind::Session => "session",
        MemoryEntityKind::Role => "role",
        MemoryEntityKind::KnowledgeDoc => "knowledge_doc",
        MemoryEntityKind::Tool => "tool",
        MemoryEntityKind::Model => "model",
        MemoryEntityKind::Harness => "harness",
        MemoryEntityKind::Proposal => "proposal",
        MemoryEntityKind::MemoryRecord => "memory_record",
        MemoryEntityKind::MemoryCandidate => "memory_candidate",
    }
}

fn relation_kind_name(kind: MemoryRelationKind) -> &'static str {
    match kind {
        MemoryRelationKind::Entity => "entity",
        MemoryRelationKind::Temporal => "temporal",
        MemoryRelationKind::Causal => "causal",
        MemoryRelationKind::Semantic => "semantic",
    }
}

fn source_kind_name(kind: MemoryRelationSourceKind) -> &'static str {
    match kind {
        MemoryRelationSourceKind::Manual => "manual",
        MemoryRelationSourceKind::FormalMemory => "formal_memory",
        MemoryRelationSourceKind::MemoryCandidate => "memory_candidate",
        MemoryRelationSourceKind::Observation => "observation",
        MemoryRelationSourceKind::KnowledgeDoc => "knowledge_doc",
        MemoryRelationSourceKind::TaskPackage => "task_package",
        MemoryRelationSourceKind::LlmInferred => "llm_inferred",
        MemoryRelationSourceKind::SimilarityHit => "similarity_hit",
    }
}
