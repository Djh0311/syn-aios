use crate::{
    TaskMemoryPacketBuildOutput, TaskMemoryPacketExcludedItem, TaskMemoryPacketExclusionReason,
    TaskMemoryPacketItem, TaskPackageMemoryInjectionSummary, TaskPackageMemoryPacketSnapshot,
    TaskPackageMemoryPacketStoreRevisions,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::Path;

const SNAPSHOT_SCHEMA_VERSION: &str = "task_package_memory_packet_snapshot.v1";

pub(crate) fn snapshot_from_build_output(
    output: &TaskMemoryPacketBuildOutput,
    work_item_id: &str,
    artifact_id: Option<&str>,
    generated_at: &str,
) -> Result<TaskPackageMemoryPacketSnapshot, String> {
    let store_revisions = TaskPackageMemoryPacketStoreRevisions {
        formal_store_revision: output.formal_store_revision,
        candidate_store_revision: output.candidate_store_revision,
        observation_store_revision: output.observation_store_revision,
        lint_store_revision: Some(output.lint_store_revision),
        entity_relation_store_revision: Some(output.entity_relation_store_revision),
    };
    let warnings = snapshot_warnings(&output.preview.warnings);
    let fingerprint = snapshot_fingerprint(output, &store_revisions, &warnings)?;
    let snapshot_id = format!(
        "task-package-memory-packet-snapshot:v1:{}:{}",
        crate::stable_id(work_item_id),
        fingerprint.chars().take(16).collect::<String>()
    );

    Ok(TaskPackageMemoryPacketSnapshot {
        snapshot_id,
        schema_version: SNAPSHOT_SCHEMA_VERSION.to_string(),
        source_packet_id: output.preview.packet_id.clone(),
        project_id: output.preview.project_id.clone(),
        workflow_id: output.preview.workflow_id.clone(),
        work_item_id: work_item_id.to_string(),
        task_package_artifact_id: artifact_id.map(str::to_string),
        role_id: output.preview.role_id.clone(),
        retrieval_intent: output.preview.retrieval_intent.clone(),
        included_memories: output.preview.included_memories.clone(),
        excluded_items: output.preview.excluded_items.clone(),
        review_materials: output.preview.review_materials.clone(),
        store_revisions,
        estimated_tokens: output.preview.estimated_tokens,
        max_estimated_tokens: output.preview.max_estimated_tokens,
        fingerprint,
        generated_at: generated_at.to_string(),
        stale: false,
        stale_reasons: vec![],
        warnings,
    })
}

pub(crate) fn write_snapshot_to_artifact(
    artifact: &mut Value,
    snapshot: &TaskPackageMemoryPacketSnapshot,
) -> Result<(), String> {
    artifact["memory_packet_snapshot"] = serde_json::to_value(snapshot)
        .map_err(|error| format!("序列化任务包记忆快照失败：{error}"))?;
    artifact["memory_packet_fingerprint"] = Value::String(snapshot.fingerprint.clone());
    artifact["memory_packet_generated_at"] = Value::String(snapshot.generated_at.clone());
    artifact["memory_packet_store_revisions"] = serde_json::to_value(&snapshot.store_revisions)
        .map_err(|error| format!("序列化任务包记忆 store revision 失败：{error}"))?;
    artifact["memory_packet_stale"] = Value::Bool(snapshot.stale);
    artifact["memory_packet_warnings"] = json!(snapshot.warnings);
    artifact["available_memory_refs"] = json!(snapshot
        .included_memories
        .iter()
        .map(|item| item.memory_id.clone())
        .collect::<Vec<_>>());
    Ok(())
}

pub(crate) fn snapshot_from_artifact(artifact: &Value) -> Option<TaskPackageMemoryPacketSnapshot> {
    artifact
        .get("memory_packet_snapshot")
        .filter(|value| !value.is_null())
        .and_then(|value| serde_json::from_value(value.clone()).ok())
}

pub(crate) fn summary_from_snapshot(
    snapshot: &TaskPackageMemoryPacketSnapshot,
    stale_reasons: Vec<String>,
) -> TaskPackageMemoryInjectionSummary {
    let stale = snapshot.stale || !stale_reasons.is_empty();
    let mut warnings = snapshot.warnings.clone();
    if stale {
        warnings.push("task_memory_packet_snapshot_stale".to_string());
    }
    TaskPackageMemoryInjectionSummary {
        snapshot_id: Some(snapshot.snapshot_id.clone()),
        included_count: snapshot.included_memories.len(),
        excluded_count: snapshot.excluded_items.len(),
        review_material_count: snapshot.review_materials.len(),
        stale,
        stale_reasons,
        display_text: format!(
            "任务包记忆注入摘要：included {} / excluded {} / review materials {}；{}。仅 active 正式记忆可进入任务包；候选 / 观察仅作为待审查材料；任务包内容不会回灌成正式记忆。",
            snapshot.included_memories.len(),
            snapshot.excluded_items.len(),
            snapshot.review_materials.len(),
            if stale { "快照已 stale" } else { "快照 fresh" }
        ),
        warnings: dedupe(warnings),
    }
}

pub(crate) fn missing_summary() -> TaskPackageMemoryInjectionSummary {
    TaskPackageMemoryInjectionSummary {
        snapshot_id: None,
        included_count: 0,
        excluded_count: 0,
        review_material_count: 0,
        stale: true,
        stale_reasons: vec!["task_memory_packet_snapshot_missing".to_string()],
        display_text: "任务包记忆注入摘要：尚未生成任务包记忆快照。仅 active 正式记忆可进入任务包；候选 / 观察仅作为待审查材料；任务包内容不会回灌成正式记忆。"
            .to_string(),
        warnings: vec!["task_memory_packet_snapshot_missing".to_string()],
    }
}

pub(crate) fn summary_from_artifact_with_current_revisions(
    workflow_state_path: &Path,
    artifact: &Value,
    timestamp: &str,
) -> Result<TaskPackageMemoryInjectionSummary, String> {
    let Some(snapshot) = snapshot_from_artifact(artifact) else {
        return Ok(missing_summary());
    };
    let current = current_store_revisions(workflow_state_path, timestamp)?;
    let stale_reasons = stale_reasons(&snapshot, &current);
    Ok(summary_from_snapshot(&snapshot, stale_reasons))
}

pub(crate) fn current_store_revisions(
    workflow_state_path: &Path,
    timestamp: &str,
) -> Result<TaskPackageMemoryPacketStoreRevisions, String> {
    Ok(TaskPackageMemoryPacketStoreRevisions {
        formal_store_revision: crate::formal_memory_store::load_store(
            workflow_state_path,
            timestamp,
        )?
        .revision,
        candidate_store_revision: crate::memory_candidate_store::load_store(
            workflow_state_path,
            timestamp,
        )?
        .revision,
        observation_store_revision: crate::observation_store::load_store(
            workflow_state_path,
            timestamp,
        )?
        .revision,
        lint_store_revision: Some(
            crate::memory_lint_store::load_store(workflow_state_path, timestamp)?.revision,
        ),
        entity_relation_store_revision: Some(
            crate::memory_entity_relation_store::load_store(workflow_state_path, timestamp)?
                .revision,
        ),
    })
}

pub(crate) fn stale_reasons(
    snapshot: &TaskPackageMemoryPacketSnapshot,
    current: &TaskPackageMemoryPacketStoreRevisions,
) -> Vec<String> {
    let mut reasons = Vec::new();
    push_revision_stale_reason(
        &mut reasons,
        "formal_store_revision",
        snapshot.store_revisions.formal_store_revision,
        current.formal_store_revision,
    );
    push_revision_stale_reason(
        &mut reasons,
        "candidate_store_revision",
        snapshot.store_revisions.candidate_store_revision,
        current.candidate_store_revision,
    );
    push_revision_stale_reason(
        &mut reasons,
        "observation_store_revision",
        snapshot.store_revisions.observation_store_revision,
        current.observation_store_revision,
    );
    match (
        snapshot.store_revisions.lint_store_revision,
        current.lint_store_revision,
    ) {
        (Some(snapshot_revision), Some(current_revision)) => push_revision_stale_reason(
            &mut reasons,
            "lint_store_revision",
            snapshot_revision,
            current_revision,
        ),
        (None, Some(_)) => reasons.push("lint_store_revision missing in snapshot".to_string()),
        _ => {}
    }
    match (
        snapshot.store_revisions.entity_relation_store_revision,
        current.entity_relation_store_revision,
    ) {
        (Some(snapshot_revision), Some(current_revision)) => push_revision_stale_reason(
            &mut reasons,
            "entity_relation_store_revision",
            snapshot_revision,
            current_revision,
        ),
        (None, Some(_)) => {
            reasons.push("entity_relation_store_revision missing in snapshot".to_string())
        }
        _ => {}
    }
    if snapshot.stale {
        reasons.extend(snapshot.stale_reasons.clone());
    }
    dedupe(reasons)
}

pub(crate) fn render_markdown_block(snapshot: &TaskPackageMemoryPacketSnapshot) -> String {
    format!(
        r#"## 正式记忆上下文

- snapshot：`{snapshot_id}`
- fingerprint：`{fingerprint}`
- store revisions：formal `{formal_revision}` / candidate `{candidate_revision}` / observation `{observation_revision}` / lint `{lint_revision}` / entity-relation `{entity_relation_revision}`
- 边界：仅 active 正式记忆可进入任务包；候选 / 观察仅作为待审查材料；任务包内容不会回灌成正式记忆。

### 入选正式记忆

{included}

### 排除摘要

{excluded}

### 待审查材料摘要

{review}

### 记忆注入 warnings

{warnings}
"#,
        snapshot_id = snapshot.snapshot_id,
        fingerprint = snapshot.fingerprint,
        formal_revision = snapshot.store_revisions.formal_store_revision,
        candidate_revision = snapshot.store_revisions.candidate_store_revision,
        observation_revision = snapshot.store_revisions.observation_store_revision,
        lint_revision = snapshot
            .store_revisions
            .lint_store_revision
            .map(|revision| revision.to_string())
            .unwrap_or_else(|| "未登记".to_string()),
        entity_relation_revision = snapshot
            .store_revisions
            .entity_relation_store_revision
            .map(|revision| revision.to_string())
            .unwrap_or_else(|| "未登记".to_string()),
        included = included_lines(&snapshot.included_memories),
        excluded = excluded_summary_lines(&snapshot.excluded_items),
        review = review_summary_lines(snapshot),
        warnings = warning_lines(&snapshot.warnings),
    )
}

pub(crate) fn render_prompt_block(snapshot: &TaskPackageMemoryPacketSnapshot) -> String {
    render_markdown_block(snapshot)
}

pub(crate) fn audit_reason(snapshot: &TaskPackageMemoryPacketSnapshot) -> String {
    format!(
        "task_memory_packet_injected_into_task_package work_item_id={} snapshot_id={} included_count={} excluded_count={}",
        snapshot.work_item_id,
        snapshot.snapshot_id,
        snapshot.included_memories.len(),
        snapshot.excluded_items.len()
    )
}

fn snapshot_fingerprint(
    output: &TaskMemoryPacketBuildOutput,
    store_revisions: &TaskPackageMemoryPacketStoreRevisions,
    warnings: &[String],
) -> Result<String, String> {
    let stable = json!({
        "schema_version": SNAPSHOT_SCHEMA_VERSION,
        "project_id": output.preview.project_id,
        "workflow_id": output.preview.workflow_id,
        "task_id": output.preview.task_id,
        "role_id": output.preview.role_id,
        "retrieval_intent": output.preview.retrieval_intent,
        "included_memories": output.preview.included_memories,
        "excluded_items": output.preview.excluded_items,
        "review_materials": output.preview.review_materials,
        "store_revisions": store_revisions,
        "estimated_tokens": output.preview.estimated_tokens,
        "max_estimated_tokens": output.preview.max_estimated_tokens,
        "warnings": warnings,
    });
    serde_json::to_string(&stable)
        .map(|text| sha256_hex(&text))
        .map_err(|error| format!("序列化任务包记忆快照 fingerprint 失败：{error}"))
}

fn snapshot_warnings(preview_warnings: &[String]) -> Vec<String> {
    let mut warnings = preview_warnings
        .iter()
        .filter(|warning| {
            warning.as_str() != "preview_only_not_injected"
                && warning.as_str() != "worker_has_not_received_memory_packet"
        })
        .cloned()
        .collect::<Vec<_>>();
    warnings.push("candidate_and_observation_review_materials_only".to_string());
    warnings.push("task_package_content_not_formal_memory_source".to_string());
    dedupe(warnings)
}

fn push_revision_stale_reason(
    reasons: &mut Vec<String>,
    label: &str,
    snapshot_revision: i64,
    current_revision: i64,
) {
    if snapshot_revision != current_revision {
        reasons.push(format!(
            "{label} changed: snapshot {snapshot_revision}, current {current_revision}"
        ));
    }
}

fn included_lines(items: &[TaskMemoryPacketItem]) -> String {
    if items.is_empty() {
        return "- 本任务包没有入选 active 正式记忆。".to_string();
    }
    items
        .iter()
        .map(|item| {
            let relation_text = if item.relation_explanations.is_empty() {
                "无已确认关系解释。".to_string()
            } else {
                item.relation_explanations
                    .iter()
                    .map(|explanation| {
                        format!("{} / {}", explanation.linked_label, explanation.explanation)
                    })
                    .collect::<Vec<_>>()
                    .join("；")
            };
            format!(
                "- `{}`：{}\n  - 来源：{}\n  - 入选理由：{}\n  - 关系解释：{}",
                item.memory_id,
                item.claim,
                source_summary(item),
                item.retrieval_reason,
                relation_text
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn excluded_summary_lines(items: &[TaskMemoryPacketExcludedItem]) -> String {
    if items.is_empty() {
        return "- 无排除项。".to_string();
    }
    let mut counts = BTreeMap::<String, usize>::new();
    for item in items {
        *counts.entry(reason_name(item.reason)).or_insert(0) += 1;
    }
    counts
        .into_iter()
        .map(|(reason, count)| format!("- {reason}: {count}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn review_summary_lines(snapshot: &TaskPackageMemoryPacketSnapshot) -> String {
    if snapshot.review_materials.is_empty() {
        return "- 无候选或观察待审查材料。".to_string();
    }
    let mut counts = BTreeMap::<String, usize>::new();
    for material in &snapshot.review_materials {
        *counts.entry(material.source_kind.clone()).or_insert(0) += 1;
    }
    counts
        .into_iter()
        .map(|(kind, count)| format!("- {kind}: {count}（只作为待审查材料）"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn warning_lines(warnings: &[String]) -> String {
    if warnings.is_empty() {
        "- 无。".to_string()
    } else {
        warnings
            .iter()
            .map(|warning| format!("- {warning}"))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

fn source_summary(item: &TaskMemoryPacketItem) -> String {
    if item.source_refs.is_empty() {
        return "来源未登记".to_string();
    }
    item.source_refs
        .iter()
        .map(|source| {
            let source_id = source.source_id.as_deref().unwrap_or("no-source-id");
            let title = source.source_title.as_deref().unwrap_or("untitled");
            format!("{} / {} / {}", source.source_type, source_id, title)
        })
        .collect::<Vec<_>>()
        .join("; ")
}

fn reason_name(reason: TaskMemoryPacketExclusionReason) -> String {
    serde_json::to_string(&reason)
        .unwrap_or_else(|_| format!("{reason:?}"))
        .trim_matches('"')
        .to_string()
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

fn sha256_hex(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    format!("{:x}", hasher.finalize())
}
