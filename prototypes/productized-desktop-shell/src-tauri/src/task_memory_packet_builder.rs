use crate::utils::hash::short_hash;
use crate::utils::normalization::normalize_slash_lowercase as normalize;
use crate::{
    FormalMemoryStoreV1, MemoryCandidateStoreV1, MemoryEntityRelationStoreV1, MemoryLintStoreV1,
    MemoryRecord, MemoryRelation, MemoryRelationSourceKind, MemoryRelationStatus,
    MemoryRelationTaskExplanation, ObservationStoreV1, TaskMemoryPacketBuildInput,
    TaskMemoryPacketBuildOutput, TaskMemoryPacketExcludedItem, TaskMemoryPacketExclusionReason,
    TaskMemoryPacketItem, TaskMemoryPacketPreview, TaskMemoryPacketReviewMaterial,
};
use std::path::Path;

pub(crate) fn build_preview(
    workflow_state_path: &Path,
    input: &TaskMemoryPacketBuildInput,
    timestamp: &str,
) -> Result<TaskMemoryPacketBuildOutput, String> {
    validate_input(input)?;
    let formal_store = crate::formal_memory_store::load_store(workflow_state_path, timestamp)?;
    let candidate_store =
        crate::memory_candidate_store::load_store(workflow_state_path, timestamp)?;
    let observation_store = crate::observation_store::load_store(workflow_state_path, timestamp)?;
    let lint_store = crate::memory_lint_store::load_store(workflow_state_path, timestamp)?;
    let entity_relation_store =
        crate::memory_entity_relation_store::load_store(workflow_state_path, timestamp)?;
    validate_expected_revisions(input, &formal_store, &candidate_store, &observation_store)?;

    let expected_project_id = input
        .project_id
        .clone()
        .unwrap_or_else(|| crate::project_id(&input.project_root));
    let expected_workflow_id = input
        .workflow_id
        .clone()
        .unwrap_or_else(|| crate::default_workflow_id(&input.project_root));
    let mut included_memories = Vec::new();
    let mut excluded_items = Vec::new();
    let mut estimated_tokens = 0_usize;

    for record in &formal_store.records {
        let item_tokens = estimate_tokens(&record.claim, &record.body);
        if let Some(reason) = crate::control_core::evaluate_task_memory_packet_item(
            crate::memory_candidate_store::memory_status_name(record.status),
            record.conflict_refs.len(),
            record.scope.valid_until.as_deref(),
            timestamp,
            &record.scope.scope_type,
            record.scope.project_id.as_deref(),
            record.scope.workflow_id.as_deref(),
            &expected_project_id,
            &expected_workflow_id,
            &record.scope.model_export_policy,
            &input.model_context_policy,
        ) {
            excluded_items.push(excluded_memory(record, reason, item_tokens));
            continue;
        }
        let blocking_lint_findings = crate::memory_lint_engine::open_blocking_findings_for_memory(
            &lint_store,
            &record.memory_id,
        );
        if !blocking_lint_findings.is_empty() {
            excluded_items.push(excluded_memory_with_detail(
                record,
                TaskMemoryPacketExclusionReason::Conflicted,
                format!(
                    "memory lint open blocking finding 阻断任务记忆包预览：{}",
                    blocking_lint_findings
                        .iter()
                        .map(|finding| finding.finding_id.clone())
                        .collect::<Vec<_>>()
                        .join(",")
                ),
            ));
            continue;
        }
        if !is_relevant(record, &input.task_goal, &input.retrieval_intent) {
            excluded_items.push(excluded_memory(
                record,
                TaskMemoryPacketExclusionReason::NotRelevant,
                item_tokens,
            ));
            continue;
        }
        if included_memories.len() >= input.max_memory_items
            || estimated_tokens.saturating_add(item_tokens) > input.max_estimated_tokens
        {
            excluded_items.push(excluded_memory(
                record,
                TaskMemoryPacketExclusionReason::TokenLimit,
                item_tokens,
            ));
            continue;
        }
        estimated_tokens += item_tokens;
        let relation_explanations =
            relation_explanations_for(record, &entity_relation_store, &input.model_context_policy);
        let retrieval_reason = if relation_explanations.is_empty() {
            retrieval_reason(record, &input.task_goal)
        } else {
            format!(
                "{}；已确认关系用于解释召回原因：{}",
                retrieval_reason(record, &input.task_goal),
                relation_explanations
                    .iter()
                    .map(|explanation| explanation.linked_label.clone())
                    .collect::<Vec<_>>()
                    .join(" / ")
            )
        };
        included_memories.push(TaskMemoryPacketItem {
            memory_id: record.memory_id.clone(),
            memory_type: record.memory_type.clone(),
            scope_type: record.scope.scope_type.clone(),
            claim: record.claim.clone(),
            body: record.body.clone(),
            source_refs: record.source_refs.clone(),
            retrieval_reason,
            relation_explanations,
            estimated_tokens: item_tokens,
            model_export_policy: record.scope.model_export_policy.clone(),
        });
    }

    let mut review_materials = Vec::new();
    append_candidate_materials(&candidate_store, &mut excluded_items, &mut review_materials);
    append_observation_materials(
        &observation_store,
        &mut excluded_items,
        &mut review_materials,
    );

    let warnings = packet_warnings(
        input,
        &formal_store,
        &candidate_store,
        &observation_store,
        &lint_store,
    );
    let preview = TaskMemoryPacketPreview {
        packet_id: format!(
            "task-memory-packet-preview:v1:{timestamp}:{}",
            short_hash(&format!(
                "{}:{}:{}:{}",
                input.project_root,
                input.task_id.clone().unwrap_or_default(),
                input.role_id,
                input.task_goal
            ))
        ),
        schema_version: "task_memory_packet.v1".to_string(),
        project_id: Some(expected_project_id),
        workflow_id: Some(expected_workflow_id),
        task_id: input.task_id.clone(),
        role_id: input.role_id.clone(),
        retrieval_intent: input.retrieval_intent.clone(),
        included_memories,
        excluded_items,
        review_materials,
        estimated_tokens,
        max_estimated_tokens: input.max_estimated_tokens,
        generated_at: timestamp.to_string(),
        warnings: warnings.clone(),
    };

    Ok(TaskMemoryPacketBuildOutput {
        preview,
        formal_store_revision: formal_store.revision,
        candidate_store_revision: candidate_store.revision,
        observation_store_revision: observation_store.revision,
        lint_store_revision: lint_store.revision,
        entity_relation_store_revision: entity_relation_store.revision,
        warnings,
    })
}

fn validate_input(input: &TaskMemoryPacketBuildInput) -> Result<(), String> {
    crate::control_core::validate_task_memory_packet_preview(
        &input.project_root,
        &input.task_goal,
        &input.role_id,
        &input.retrieval_intent,
        &input.model_context_policy,
        input.max_memory_items,
        input.max_estimated_tokens,
    )?;
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
                "任务记忆包上下文绑定失败：{field_name} 与 project_root 不匹配，expected {expected}，actual {}",
                actual.trim()
            ));
        }
    }
    Ok(())
}

fn validate_expected_revisions(
    input: &TaskMemoryPacketBuildInput,
    formal_store: &FormalMemoryStoreV1,
    candidate_store: &MemoryCandidateStoreV1,
    observation_store: &ObservationStoreV1,
) -> Result<(), String> {
    if let Some(expected) = input.expected_formal_store_revision {
        if expected != formal_store.revision {
            return Err(format!(
                "task_memory_packet_formal_store_conflict: expected revision {expected}, actual {}",
                formal_store.revision
            ));
        }
    }
    if let Some(expected) = input.expected_candidate_store_revision {
        if expected != candidate_store.revision {
            return Err(format!(
                "task_memory_packet_candidate_store_conflict: expected revision {expected}, actual {}",
                candidate_store.revision
            ));
        }
    }
    if let Some(expected) = input.expected_observation_store_revision {
        if expected != observation_store.revision {
            return Err(format!(
                "task_memory_packet_observation_store_conflict: expected revision {expected}, actual {}",
                observation_store.revision
            ));
        }
    }
    Ok(())
}

fn excluded_memory(
    record: &MemoryRecord,
    reason: TaskMemoryPacketExclusionReason,
    estimated_tokens: usize,
) -> TaskMemoryPacketExcludedItem {
    TaskMemoryPacketExcludedItem {
        source_kind: "memory_record".to_string(),
        source_id: record.memory_id.clone(),
        claim: Some(record.claim.clone()),
        reason,
        detail: format!(
            "正式记忆未进入预览 included list；status={} / scope={} / estimated_tokens={estimated_tokens}",
            crate::memory_candidate_store::memory_status_name(record.status),
            record.scope.scope_type
        ),
    }
}

fn excluded_memory_with_detail(
    record: &MemoryRecord,
    reason: TaskMemoryPacketExclusionReason,
    detail: String,
) -> TaskMemoryPacketExcludedItem {
    TaskMemoryPacketExcludedItem {
        source_kind: "memory_record".to_string(),
        source_id: record.memory_id.clone(),
        claim: Some(record.claim.clone()),
        reason,
        detail,
    }
}

fn append_candidate_materials(
    candidate_store: &MemoryCandidateStoreV1,
    excluded_items: &mut Vec<TaskMemoryPacketExcludedItem>,
    review_materials: &mut Vec<TaskMemoryPacketReviewMaterial>,
) {
    for candidate in &candidate_store.candidates {
        excluded_items.push(TaskMemoryPacketExcludedItem {
            source_kind: "memory_candidate".to_string(),
            source_id: candidate.candidate_key.clone(),
            claim: Some(candidate.claim.clone()),
            reason: TaskMemoryPacketExclusionReason::CandidateUnconfirmed,
            detail: "记忆候选不是正式记忆；不会进入任务记忆包 included list".to_string(),
        });
        review_materials.push(TaskMemoryPacketReviewMaterial {
            source_kind: "memory_candidate".to_string(),
            source_id: candidate.candidate_key.clone(),
            title: candidate.claim.clone(),
            reason: TaskMemoryPacketExclusionReason::CandidateUnconfirmed,
        });
    }
}

fn append_observation_materials(
    observation_store: &ObservationStoreV1,
    excluded_items: &mut Vec<TaskMemoryPacketExcludedItem>,
    review_materials: &mut Vec<TaskMemoryPacketReviewMaterial>,
) {
    for observation in &observation_store.observations {
        excluded_items.push(TaskMemoryPacketExcludedItem {
            source_kind: "observation".to_string(),
            source_id: observation.observation_key.clone(),
            claim: Some(observation.summary.clone()),
            reason: TaskMemoryPacketExclusionReason::ObservationNotFormalMemory,
            detail: "observation 不是正式记忆；不会进入任务记忆包 included list".to_string(),
        });
        review_materials.push(TaskMemoryPacketReviewMaterial {
            source_kind: "observation".to_string(),
            source_id: observation.observation_key.clone(),
            title: observation.summary.clone(),
            reason: TaskMemoryPacketExclusionReason::ObservationNotFormalMemory,
        });
    }
}

fn packet_warnings(
    input: &TaskMemoryPacketBuildInput,
    formal_store: &FormalMemoryStoreV1,
    candidate_store: &MemoryCandidateStoreV1,
    observation_store: &ObservationStoreV1,
    lint_store: &MemoryLintStoreV1,
) -> Vec<String> {
    let mut warnings = vec![
        "preview_only_not_injected".to_string(),
        "worker_has_not_received_memory_packet".to_string(),
        "candidate_and_observation_review_materials_only".to_string(),
    ];
    if formal_store.records.is_empty() {
        warnings.push("formal_memory_store_empty".to_string());
    }
    if !candidate_store.candidates.is_empty() {
        warnings.push("memory_candidates_excluded_as_unconfirmed".to_string());
    }
    if !observation_store.observations.is_empty() {
        warnings.push("observations_excluded_as_not_formal_memory".to_string());
    }
    if input.model_context_policy == "external_model_context" {
        warnings.push("external_model_context_filters_blocked_memories".to_string());
    }
    let blocking_count = lint_store
        .findings
        .iter()
        .filter(|finding| crate::memory_lint_engine::is_open_blocking(finding))
        .count();
    if blocking_count > 0 {
        warnings.push("memory_lint_blocking_findings_excluded".to_string());
    }
    warnings
}

fn is_relevant(record: &MemoryRecord, task_goal: &str, retrieval_intent: &str) -> bool {
    let haystack = normalize(&format!(
        "{} {} {} {}",
        record.claim, record.body, record.memory_type, record.scope.scope_type
    ));
    for token in goal_tokens(task_goal) {
        if haystack.contains(&token) {
            return true;
        }
    }
    retrieval_intent != "worker_task"
        && matches!(
            record.memory_type.as_str(),
            "project_memory" | "workflow_summary" | "session_summary"
        )
}

fn retrieval_reason(record: &MemoryRecord, task_goal: &str) -> String {
    let matched = goal_tokens(task_goal)
        .into_iter()
        .find(|token| normalize(&format!("{} {}", record.claim, record.body)).contains(token))
        .unwrap_or_else(|| "scope_match".to_string());
    format!(
        "active formal memory matched task goal by {matched}; scope={}",
        record.scope.scope_type
    )
}

fn relation_explanations_for(
    record: &MemoryRecord,
    store: &MemoryEntityRelationStoreV1,
    model_context_policy: &str,
) -> Vec<MemoryRelationTaskExplanation> {
    let memory_entity_id = entity_id_for_memory_record(&record.memory_id);
    store
        .relations
        .iter()
        .filter(|relation| relation_can_explain(relation, &memory_entity_id, model_context_policy))
        .map(|relation| {
            let memory_is_subject = relation.subject_entity_id == memory_entity_id;
            MemoryRelationTaskExplanation {
                relation_id: relation.relation_id.clone(),
                relation_kind: relation.relation_kind,
                linked_entity_id: if memory_is_subject {
                    relation.object_entity_id.clone()
                } else {
                    relation.subject_entity_id.clone()
                },
                linked_label: if memory_is_subject {
                    relation.object_label.clone()
                } else {
                    relation.subject_label.clone()
                },
                explanation: format!(
                    "已确认关系用于解释召回原因：{} -> {} / {}",
                    relation.subject_label, relation.object_label, relation.predicate
                ),
                source_count: relation.source_refs.len(),
            }
        })
        .collect()
}

fn relation_can_explain(
    relation: &MemoryRelation,
    memory_entity_id: &str,
    model_context_policy: &str,
) -> bool {
    if relation.status != MemoryRelationStatus::Confirmed {
        return false;
    }
    if relation.source_kind == MemoryRelationSourceKind::LlmInferred
        || relation.source_kind == MemoryRelationSourceKind::SimilarityHit
    {
        return false;
    }
    if relation.subject_entity_id != memory_entity_id
        && relation.object_entity_id != memory_entity_id
    {
        return false;
    }
    if relation
        .source_refs
        .iter()
        .any(|source| source.sensitive_level == "secret")
    {
        return false;
    }
    if model_context_policy == "external_model_context"
        && relation
            .source_refs
            .iter()
            .any(|source| matches!(source.sensitive_level.as_str(), "private" | "secret"))
    {
        return false;
    }
    true
}

fn entity_id_for_memory_record(memory_id: &str) -> String {
    let canonical_key = format!("memory_record:{}", alias_key(memory_id));
    format!("entity:v1:memory_record:{}", short_hash(&canonical_key))
}

fn goal_tokens(task_goal: &str) -> Vec<String> {
    normalize(task_goal)
        .split(|ch: char| {
            ch.is_whitespace()
                || ch.is_ascii_punctuation()
                || matches!(
                    ch,
                    '，' | '。'
                        | '；'
                        | '：'
                        | '、'
                        | '（'
                        | '）'
                        | '【'
                        | '】'
                        | '《'
                        | '》'
                        | '！'
                        | '？'
                )
        })
        .filter(|token| token.chars().count() >= 2)
        .flat_map(expand_goal_token)
        .collect()
}

fn expand_goal_token(token: &str) -> Vec<String> {
    let mut tokens = vec![token.to_string()];
    let chars = token.chars().collect::<Vec<_>>();
    if chars.len() > 4 && chars.iter().any(|character| !character.is_ascii()) {
        for window in chars.windows(4) {
            tokens.push(window.iter().collect());
        }
    }
    tokens
}

fn estimate_tokens(claim: &str, body: &str) -> usize {
    let (non_ascii_chars, ascii_chars) = claim.chars().chain(body.chars()).fold(
        (0_usize, 0_usize),
        |(non_ascii_chars, ascii_chars), character| {
            if character.is_ascii() {
                (non_ascii_chars, ascii_chars + 1)
            } else {
                (non_ascii_chars + 1, ascii_chars)
            }
        },
    );
    (non_ascii_chars + ascii_chars / 4).max(1) + 16
}

#[cfg(test)]
mod tests {
    use super::estimate_tokens;

    #[test]
    fn task_memory_packet_estimates_mixed_cjk_and_ascii_in_separate_buckets() {
        let claim = "中文记忆候选转正";
        let body = "abcdEFGH";
        let previous_estimate = ((claim.chars().count() + body.chars().count()) / 4).max(1) + 16;

        assert_eq!(previous_estimate, 20);
        assert_eq!(estimate_tokens(claim, body), 26);
    }
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
