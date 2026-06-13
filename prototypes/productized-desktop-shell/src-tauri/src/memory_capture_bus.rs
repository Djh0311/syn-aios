use crate::utils::hash::{sha256_hex, short_hash12 as short_hash};
use crate::{
    CaptureMemoryEventInput, CaptureMemoryEventOutput, CreateMemoryCandidateFromObservationInput,
    CreateObservationInput, MemoryCaptureEventRecord, MemoryCaptureStoreV1, ObservationSourceRef,
};
use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};

const STORE_VERSION: &str = "memory_capture_store.v1";
const SIDECAR_NAME: &str = "memory-capture-events.v1.json";
const LOCK_NAME: &str = ".memory-capture-events.v1.lock";

pub(crate) fn sidecar_path(workflow_state_path: &Path) -> Result<PathBuf, String> {
    Ok(workflow_state_path
        .parent()
        .ok_or_else(|| {
            format!(
                "workflow state 路径没有父目录，无法推导 memory capture sidecar：{}",
                workflow_state_path.display()
            )
        })?
        .join(SIDECAR_NAME))
}

pub(crate) fn load_store(
    workflow_state_path: &Path,
    timestamp: &str,
) -> Result<MemoryCaptureStoreV1, String> {
    let sidecar = sidecar_path(workflow_state_path)?;
    if !sidecar.exists() {
        return Ok(empty_store(timestamp));
    }
    let text = fs::read_to_string(&sidecar).map_err(|error| {
        format!(
            "读取 memory capture sidecar 失败 {}：{error}",
            sidecar.display()
        )
    })?;
    let store: MemoryCaptureStoreV1 = serde_json::from_str(&text).map_err(|error| {
        format!(
            "memory capture sidecar JSON 损坏，已拒绝覆盖 {}：{error}",
            sidecar.display()
        )
    })?;
    validate_store(&store)?;
    Ok(store)
}

pub(crate) fn capture_event(
    workflow_state_path: &Path,
    input: &CaptureMemoryEventInput,
    timestamp: &str,
    capture_write_id: &str,
    observation_write_id: &str,
    candidate_write_id: &str,
) -> Result<CaptureMemoryEventOutput, String> {
    validate_input(input)?;
    let event_key = stable_event_key(input)?;
    let store = load_store(workflow_state_path, timestamp)?;
    if let Some(expected) = input.expected_capture_store_revision {
        if expected != store.revision {
            return Err(format!(
                "memory_capture_store_conflict: expected revision {expected}, actual {}",
                store.revision
            ));
        }
    }
    if store
        .events
        .iter()
        .any(|event| event.event_key == event_key)
    {
        return Err(
            "memory_capture_duplicate: 相同 capture event 已存在；不会自动覆盖".to_string(),
        );
    }

    let mut warnings = vec!["memory_capture_event_is_not_formal_memory".to_string()];
    let mut observation = None;
    let mut candidate = None;
    let mut observation_store_revision = None;
    let mut candidate_store_revision = None;
    let mut blocked_reason = None;

    match input.candidate_policy.as_str() {
        "audit_only" => {
            warnings.push("memory_capture_audit_only_no_observation".to_string());
        }
        "blocked_sensitive" => {
            blocked_reason = Some("memory_capture_blocked_sensitive".to_string());
            warnings.push("memory_capture_blocked_sensitive_no_observation".to_string());
        }
        "observation_only" | "candidate_allowed" => {
            let observation_input = observation_input_from_capture(input)?;
            let created_observation = crate::observation_store::create_observation(
                workflow_state_path,
                &observation_input,
                timestamp,
                observation_write_id,
            )?;
            observation_store_revision = Some(created_observation.store_revision);
            warnings.extend(created_observation.warnings.clone());

            if input.candidate_policy == "candidate_allowed" {
                let draft = input
                    .candidate
                    .as_ref()
                    .ok_or_else(|| "candidate_allowed 缺少 candidate draft".to_string())?;
                let candidate_input = CreateMemoryCandidateFromObservationInput {
                    project_root: input.project_root.clone(),
                    observation_key: created_observation.observation.observation_key.clone(),
                    actor_id: input.actor_id.clone(),
                    actor_role: draft.actor_role.clone(),
                    memory_type: draft.memory_type.clone(),
                    claim: draft.claim.clone(),
                    body: draft.body.clone(),
                    review_reason: draft.review_reason.clone(),
                    requires_user_confirmation: draft.requires_user_confirmation,
                    expected_observation_store_revision: observation_store_revision,
                    expected_candidate_store_revision: input.expected_candidate_store_revision,
                };
                let created_candidate =
                    crate::observation_store::create_memory_candidate_from_observation(
                        workflow_state_path,
                        &candidate_input,
                        timestamp,
                        observation_write_id,
                        |candidate_input| {
                            crate::memory_candidate_store::create_candidate(
                                workflow_state_path,
                                candidate_input,
                                timestamp,
                                candidate_write_id,
                            )
                        },
                    )?;
                observation_store_revision = Some(created_candidate.observation_store_revision);
                candidate_store_revision = Some(created_candidate.candidate_store_revision);
                warnings.extend(created_candidate.warnings.clone());
                candidate = Some(created_candidate.candidate);
                observation = Some(created_candidate.observation);
            } else {
                observation = Some(created_observation.observation);
            }
        }
        _ => unreachable!("candidate policy should be validated"),
    }

    let capture_event = MemoryCaptureEventRecord {
        capture_event_id: format!("memory-capture:{timestamp}:{}", short_hash(&event_key)),
        event_key,
        schema_version: "memory_capture_event.v1".to_string(),
        source_type: input.source_type.clone(),
        source_ref_id: input.source_refs[0].source_ref_id.clone(),
        project_id: input.project_id.clone(),
        workflow_id: input.workflow_id.clone(),
        workflow_node_id: input.workflow_node_id.clone(),
        run_unit_id: input.run_unit_id.clone(),
        product_command_id: input.product_command_id.clone(),
        product_attempt_id: input.product_attempt_id.clone(),
        runtime_log_ref: input.runtime_log_ref.clone(),
        audit_refs: input.audit_refs.clone(),
        readback_ref: input.readback_ref.clone(),
        task_package_ref: input.task_package_ref.clone(),
        memory_packet_ref: input.memory_packet_ref.clone(),
        summary: input.summary.trim().to_string(),
        evidence_summary: input.evidence_summary.trim().to_string(),
        sensitivity: input.sensitivity.clone(),
        candidate_policy: input.candidate_policy.clone(),
        blocked_reason,
        observation_id: observation
            .as_ref()
            .map(|record| record.observation_id.clone()),
        candidate_key: candidate
            .as_ref()
            .map(|record| record.candidate_key.clone()),
        created_by: input.actor_id.clone(),
        created_at: timestamp.to_string(),
        updated_at: timestamp.to_string(),
    };

    append_event(
        workflow_state_path,
        capture_event.clone(),
        timestamp,
        capture_write_id,
    )?;
    let final_store = load_store(workflow_state_path, timestamp)?;
    warnings.push("formal_memory_not_written_by_memory_capture".to_string());
    warnings = dedupe(warnings);

    Ok(CaptureMemoryEventOutput {
        capture_event,
        observation,
        candidate,
        observation_store_revision,
        candidate_store_revision,
        capture_store_revision: final_store.revision,
        warnings,
    })
}

fn observation_input_from_capture(
    input: &CaptureMemoryEventInput,
) -> Result<CreateObservationInput, String> {
    Ok(CreateObservationInput {
        project_root: input.project_root.clone(),
        project_id: input.project_id.clone(),
        workflow_id: input.workflow_id.clone(),
        scope: input.scope.clone(),
        observation_type: observation_type_from_source_type(&input.source_type).to_string(),
        summary: input.summary.trim().to_string(),
        source_refs: input
            .source_refs
            .iter()
            .map(|source| ObservationSourceRef {
                source_ref_id: source.source_ref_id.clone(),
                source_kind: observation_source_kind_from_capture_source(&source.source_type)
                    .to_string(),
                source_id: source.source_id.clone(),
                project_id: source.project_id.clone().or(input.project_id.clone()),
                workflow_id: source.workflow_id.clone().or(input.workflow_id.clone()),
                session_id: None,
                file_path: None,
                evidence_ref: source.evidence_ref.clone(),
                summary: source.summary.clone(),
                sensitive_level: observation_sensitive_level(&source.sensitive_level).to_string(),
                created_at: source.created_at.clone(),
            })
            .collect(),
        generated_by_role: input.generated_by_role.clone(),
        actor_id: input.actor_id.clone(),
        risk_level: input.risk_level.clone(),
        sensitive_level: observation_sensitive_level(&input.sensitivity).to_string(),
        reason: input.reason.clone(),
        expected_store_revision: input.expected_observation_store_revision,
    })
}

fn append_event(
    workflow_state_path: &Path,
    event: MemoryCaptureEventRecord,
    timestamp: &str,
    write_id: &str,
) -> Result<(), String> {
    let sidecar = sidecar_path(workflow_state_path)?;
    let parent = sidecar
        .parent()
        .ok_or_else(|| format!("memory capture sidecar 没有父目录：{}", sidecar.display()))?;
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "创建 memory capture sidecar 目录失败 {}：{error}",
            parent.display()
        )
    })?;
    let lock_path = parent.join(LOCK_NAME);
    let lock = StoreLock::acquire(&lock_path, write_id)?;
    let mut store = load_store(workflow_state_path, timestamp)?;
    if store
        .events
        .iter()
        .any(|existing| existing.event_key == event.event_key)
    {
        drop(lock);
        return Err(
            "memory_capture_duplicate: 相同 capture event 已存在；不会自动覆盖".to_string(),
        );
    }
    store.project_id = event.project_id.clone().or(store.project_id);
    store.workflow_id = event.workflow_id.clone().or(store.workflow_id);
    store.revision += 1;
    store.updated_at = timestamp.to_string();
    store.events.push(event);
    write_store_atomic(&sidecar, &store, timestamp, write_id)?;
    drop(lock);
    Ok(())
}

fn empty_store(timestamp: &str) -> MemoryCaptureStoreV1 {
    MemoryCaptureStoreV1 {
        store_version: STORE_VERSION.to_string(),
        project_id: None,
        workflow_id: None,
        revision: 0,
        events: vec![],
        updated_at: timestamp.to_string(),
        warnings: vec!["memory_capture_j3_bridge_only".to_string()],
    }
}

fn validate_store(store: &MemoryCaptureStoreV1) -> Result<(), String> {
    if store.store_version != STORE_VERSION {
        return Err(format!(
            "memory capture store_version 不匹配：{}",
            store.store_version
        ));
    }
    if store.revision < 0 {
        return Err("memory capture revision 不能小于 0".to_string());
    }
    for event in &store.events {
        if event.schema_version != "memory_capture_event.v1" {
            return Err(format!(
                "memory capture schema_version 不匹配：{}",
                event.schema_version
            ));
        }
    }
    Ok(())
}

fn validate_input(input: &CaptureMemoryEventInput) -> Result<(), String> {
    if input.project_root.trim().is_empty() {
        return Err("memory capture 缺少 project_root".to_string());
    }
    if input.summary.trim().is_empty() {
        return Err("memory capture summary 不能为空".to_string());
    }
    if input.evidence_summary.trim().is_empty() {
        return Err("memory capture evidence_summary 不能为空".to_string());
    }
    if input.source_refs.is_empty() {
        return Err("memory capture 缺少 source_refs".to_string());
    }
    validate_source_type(&input.source_type)?;
    validate_candidate_policy(&input.candidate_policy)?;
    validate_sensitivity(&input.sensitivity)?;
    validate_actor_role(&input.generated_by_role)?;
    validate_risk_level(&input.risk_level)?;
    validate_forbidden_text(&input.summary)?;
    validate_forbidden_text(&input.evidence_summary)?;
    for source in &input.source_refs {
        validate_source_type(&source.source_type)?;
        validate_sensitivity(&source.sensitive_level)?;
        validate_forbidden_text(&source.summary)?;
        if source.source_ref_id.trim().is_empty() || source.source_id.trim().is_empty() {
            return Err("memory capture source_ref_id/source_id 不能为空".to_string());
        }
    }
    if input.sensitivity == "secret" && input.candidate_policy != "blocked_sensitive" {
        return Err(
            "secret memory capture 必须使用 blocked_sensitive，不能生成 observation/candidate"
                .to_string(),
        );
    }
    if input
        .source_refs
        .iter()
        .any(|source| source.sensitive_level == "secret")
        && input.candidate_policy != "blocked_sensitive"
    {
        return Err(
            "secret source 必须使用 blocked_sensitive，不能生成 observation/candidate".to_string(),
        );
    }
    if input.candidate_policy == "candidate_allowed" {
        let draft = input
            .candidate
            .as_ref()
            .ok_or_else(|| "candidate_allowed 缺少 candidate draft".to_string())?;
        if draft.actor_role != "project_director" {
            return Err("J3 只允许 project_director 从 capture observation 生成候选".to_string());
        }
        if draft.claim.trim().is_empty() || draft.body.trim().is_empty() {
            return Err("candidate draft claim/body 不能为空".to_string());
        }
        validate_forbidden_text(&draft.claim)?;
        validate_forbidden_text(&draft.body)?;
        validate_forbidden_text(&draft.review_reason)?;
    }
    Ok(())
}

fn validate_source_type(value: &str) -> Result<(), String> {
    if matches!(
        value,
        "user_action"
            | "product_command"
            | "runtime_log"
            | "readback"
            | "worker_report"
            | "process_fact_decision"
            | "final_review"
    ) {
        Ok(())
    } else {
        Err(format!("未知 memory capture source_type：{value}"))
    }
}

fn validate_candidate_policy(value: &str) -> Result<(), String> {
    if matches!(
        value,
        "observation_only" | "candidate_allowed" | "audit_only" | "blocked_sensitive"
    ) {
        Ok(())
    } else {
        Err(format!("未知 memory capture candidate_policy：{value}"))
    }
}

fn validate_sensitivity(value: &str) -> Result<(), String> {
    if matches!(
        value,
        "public" | "internal" | "project_confidential" | "secret"
    ) {
        Ok(())
    } else {
        Err(format!("未知 memory capture sensitivity：{value}"))
    }
}

fn validate_actor_role(value: &str) -> Result<(), String> {
    if matches!(
        value,
        "worker" | "project_director" | "global_director" | "user" | "system"
    ) {
        Ok(())
    } else {
        Err(format!("未知 memory capture actor role：{value}"))
    }
}

fn validate_risk_level(value: &str) -> Result<(), String> {
    if matches!(value, "low" | "medium" | "high") {
        Ok(())
    } else {
        Err(format!("未知 memory capture risk_level：{value}"))
    }
}

fn validate_forbidden_text(value: &str) -> Result<(), String> {
    let lower = value.to_ascii_lowercase();
    let forbidden = [
        "full transcript",
        "raw stdout",
        "raw stderr",
        "prompt body",
        "auth token",
        "oauth",
        "keychain",
        ".env",
        "rollout",
        "provider credential",
    ];
    if let Some(hit) = forbidden.iter().find(|needle| lower.contains(**needle)) {
        return Err(format!(
            "memory_capture_forbidden_sensitive_text: {hit} 不能进入 capture / observation / candidate"
        ));
    }
    Ok(())
}

fn observation_type_from_source_type(source_type: &str) -> &'static str {
    match source_type {
        "worker_report" | "readback" => "worker_report",
        "process_fact_decision" => "process_fact",
        "final_review" => "global_director_review",
        "user_action" => "plan_adopted",
        "product_command" | "runtime_log" => "process_fact",
        _ => "process_fact",
    }
}

fn observation_source_kind_from_capture_source(source_type: &str) -> &'static str {
    match source_type {
        "worker_report" | "readback" => "worker_report",
        "final_review" | "process_fact_decision" => "director_review",
        "user_action" => "user_confirmation",
        "product_command" | "runtime_log" => "workflow_event",
        _ => "workflow_event",
    }
}

fn observation_sensitive_level(value: &str) -> &'static str {
    match value {
        "public" => "public",
        "internal" => "internal",
        "project_confidential" => "sensitive",
        "secret" => "secret",
        _ => "sensitive",
    }
}

fn stable_event_key(input: &CaptureMemoryEventInput) -> Result<String, String> {
    let refs = input
        .source_refs
        .iter()
        .map(|source| {
            format!(
                "{}:{}:{}",
                normalize(&source.source_type),
                normalize(&source.source_id),
                normalize(&source.source_ref_id)
            )
        })
        .collect::<Vec<_>>();
    if refs.is_empty() {
        return Err("memory capture 来源规范化后为空".to_string());
    }
    Ok(format!(
        "memory-capture:v1:{}",
        sha256_hex(
            &[
                normalize(&input.source_type),
                normalize(input.project_id.as_deref().unwrap_or_default()),
                normalize(input.workflow_id.as_deref().unwrap_or_default()),
                normalize(input.run_unit_id.as_deref().unwrap_or_default()),
                normalize(input.product_command_id.as_deref().unwrap_or_default()),
                normalize(&input.summary),
                refs.join("\n"),
            ]
            .join("\0")
        )
    ))
}

fn write_store_atomic(
    sidecar: &Path,
    store: &MemoryCaptureStoreV1,
    timestamp: &str,
    write_id: &str,
) -> Result<(), String> {
    let parent = sidecar
        .parent()
        .ok_or_else(|| format!("memory capture sidecar 没有父目录：{}", sidecar.display()))?;
    let temp_path = parent.join(format!(
        ".memory-capture-events.v1.{timestamp}.{write_id}.tmp"
    ));
    let text = serde_json::to_string_pretty(store)
        .map_err(|error| format!("memory capture sidecar 序列化失败：{error}"))?;
    fs::write(&temp_path, text).map_err(|error| {
        format!(
            "写入 memory capture 临时文件失败 {}：{error}",
            temp_path.display()
        )
    })?;
    fs::rename(&temp_path, sidecar).map_err(|error| {
        let _ = fs::remove_file(&temp_path);
        format!(
            "替换 memory capture sidecar 失败 {}：{error}",
            sidecar.display()
        )
    })
}

fn normalize(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

fn dedupe(values: Vec<String>) -> Vec<String> {
    let mut result = Vec::new();
    for value in values {
        if !result.contains(&value) {
            result.push(value);
        }
    }
    result
}

struct StoreLock {
    path: PathBuf,
}

impl StoreLock {
    fn acquire(path: &Path, write_id: &str) -> Result<Self, String> {
        match OpenOptions::new().write(true).create_new(true).open(path) {
            Ok(_) => Ok(StoreLock {
                path: path.to_path_buf(),
            }),
            Err(error) => Err(format!(
                "memory capture sidecar lock busy {} ({write_id})：{error}",
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{MemoryCaptureCandidateDraft, MemoryCaptureSourceRef, MemoryScope};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_workflow_path(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("{name}-{nanos}"));
        fs::create_dir_all(&dir).expect("temp dir should be created");
        dir.join("workflow-state.v0.json")
    }

    fn scope(project_id: &str, workflow_id: &str) -> MemoryScope {
        MemoryScope {
            scope_id: format!("scope:{workflow_id}"),
            scope_type: "workflow".to_string(),
            user_id: None,
            project_id: Some(project_id.to_string()),
            workflow_id: Some(workflow_id.to_string()),
            session_id: None,
            role_ids: vec!["project_director".to_string()],
            document_refs: vec![],
            permission_policy_ref: None,
            model_export_policy: "local_only".to_string(),
            valid_from: "2026-06-09T00:00:00Z".to_string(),
            valid_until: None,
        }
    }

    fn capture_input(project_root: &str) -> CaptureMemoryEventInput {
        let project_id = crate::project_id(project_root);
        let workflow_id = crate::default_workflow_id(project_root);
        CaptureMemoryEventInput {
            project_root: project_root.to_string(),
            project_id: Some(project_id.clone()),
            workflow_id: Some(workflow_id.clone()),
            workflow_node_id: Some(format!("{workflow_id}:node:codex-dev")),
            run_unit_id: Some("run-unit:j2-b:b2:developer".to_string()),
            product_command_id: Some("real-exec-command:codex-control:test".to_string()),
            product_attempt_id: Some(
                "real-exec-command-attempt:phase-b-new-session:test".to_string(),
            ),
            runtime_log_ref: Some("runtime-log:dispatch-attempt:test".to_string()),
            audit_refs: vec!["audit:workflow:test".to_string()],
            readback_ref: Some("readback:last-message:test".to_string()),
            task_package_ref: Some("task-package:j2-b:test".to_string()),
            memory_packet_ref: Some("memory-packet:j2-b:test".to_string()),
            scope: scope(&project_id, &workflow_id),
            source_type: "worker_report".to_string(),
            source_refs: vec![MemoryCaptureSourceRef {
                source_ref_id: "source:j2-b:b2-worker-report".to_string(),
                source_type: "worker_report".to_string(),
                source_id: "worker-report:j2-b:b2".to_string(),
                project_id: Some(project_id),
                workflow_id: Some(workflow_id),
                workflow_node_id: Some("node:codex-dev".to_string()),
                run_unit_id: Some("run-unit:j2-b:b2:developer".to_string()),
                product_command_id: Some("real-exec-command:codex-control:test".to_string()),
                product_attempt_id: Some(
                    "real-exec-command-attempt:phase-b-new-session:test".to_string(),
                ),
                runtime_log_ref: Some("runtime-log:dispatch-attempt:test".to_string()),
                audit_ref_id: Some("audit:workflow:test".to_string()),
                readback_ref: Some("readback:last-message:test".to_string()),
                task_package_ref: Some("task-package:j2-b:test".to_string()),
                memory_packet_ref: Some("memory-packet:j2-b:test".to_string()),
                evidence_ref: Some(
                    "evidence/2026-06-09-stage-j-j2-b-b2-real-isolated-project-workflow-new-session-write-probe-v1.md"
                        .to_string(),
                ),
                summary:
                    "J2-B B2 worker report marker returned and allowed write path was updated."
                        .to_string(),
                sensitive_level: "internal".to_string(),
                created_at: "2026-06-09T12:00:03Z".to_string(),
            }],
            summary:
                "J2-B B2 worker report confirmed allowed write path and readback marker."
                    .to_string(),
            evidence_summary: "Readback succeeded with result_count=1; input content was not persisted."
                .to_string(),
            sensitivity: "internal".to_string(),
            candidate_policy: "candidate_allowed".to_string(),
            generated_by_role: "project_director".to_string(),
            actor_id: "project-director:j3".to_string(),
            risk_level: "low".to_string(),
            reason: "J3 capture bus records B2 worker report as observation and candidate source."
                .to_string(),
            candidate: Some(MemoryCaptureCandidateDraft {
                memory_type: "workflow_summary".to_string(),
                claim: "J2-B B2 isolated run unit can write only the allowed path.".to_string(),
                body:
                    "J2-B B2 completed through Product Command Phase B and produced a worker report candidate for the allowed write path."
                        .to_string(),
                review_reason: "从 J2-B B2 worker report observation 生成待审候选；候选不是正式记忆。"
                    .to_string(),
                requires_user_confirmation: false,
                actor_role: "project_director".to_string(),
            }),
            expected_capture_store_revision: None,
            expected_observation_store_revision: None,
            expected_candidate_store_revision: None,
        }
    }

    #[test]
    fn memory_capture_candidate_allowed_creates_observation_and_candidate_only() {
        let path = temp_workflow_path("memory-capture-candidate");
        let project_root = "/tmp/memory-capture-candidate-project";
        let output = capture_event(
            &path,
            &capture_input(project_root),
            "2026-06-09T12:30:00Z",
            "write-capture",
            "write-observation",
            "write-candidate",
        )
        .expect("capture should create observation and candidate");

        assert!(output.observation.is_some());
        assert!(output.candidate.is_some());
        assert_eq!(
            output.capture_event.candidate_key,
            output
                .candidate
                .as_ref()
                .map(|candidate| candidate.candidate_key.clone())
        );
        assert!(path
            .parent()
            .expect("workflow parent")
            .join("memory-capture-events.v1.json")
            .exists());
        assert!(!path
            .parent()
            .expect("workflow parent")
            .join("formal-memories.v1.json")
            .exists());
        assert!(output
            .warnings
            .contains(&"formal_memory_not_written_by_memory_capture".to_string()));

        let _ = fs::remove_dir_all(path.parent().expect("workflow parent"));
    }

    #[test]
    fn memory_capture_audit_only_writes_no_observation_or_candidate() {
        let path = temp_workflow_path("memory-capture-audit-only");
        let project_root = "/tmp/memory-capture-audit-only-project";
        let mut input = capture_input(project_root);
        input.candidate_policy = "audit_only".to_string();
        input.candidate = None;

        let output = capture_event(
            &path,
            &input,
            "2026-06-09T12:31:00Z",
            "write-capture-audit",
            "write-observation-audit",
            "write-candidate-audit",
        )
        .expect("audit-only capture should be recorded");

        assert!(output.observation.is_none());
        assert!(output.candidate.is_none());
        assert!(!path
            .parent()
            .expect("workflow parent")
            .join("observations.v1.json")
            .exists());
        assert!(!path
            .parent()
            .expect("workflow parent")
            .join("memory-candidates.v1.json")
            .exists());

        let _ = fs::remove_dir_all(path.parent().expect("workflow parent"));
    }

    #[test]
    fn memory_capture_duplicate_event_is_rejected_without_append() {
        let path = temp_workflow_path("memory-capture-duplicate");
        let project_root = "/tmp/memory-capture-duplicate-project";
        let mut input = capture_input(project_root);
        input.candidate_policy = "audit_only".to_string();
        input.candidate = None;

        capture_event(
            &path,
            &input,
            "2026-06-09T12:31:10Z",
            "write-capture-duplicate-1",
            "write-observation-duplicate-1",
            "write-candidate-duplicate-1",
        )
        .expect("first audit-only capture should be recorded");

        let err = capture_event(
            &path,
            &input,
            "2026-06-09T12:31:11Z",
            "write-capture-duplicate-2",
            "write-observation-duplicate-2",
            "write-candidate-duplicate-2",
        )
        .unwrap_err();

        assert!(err.contains("memory_capture_duplicate"));
        let store = load_store(&path, "2026-06-09T12:31:12Z").expect("store should remain valid");
        assert_eq!(store.revision, 1);
        assert_eq!(store.events.len(), 1);
        assert!(!path
            .parent()
            .expect("workflow parent")
            .join("observations.v1.json")
            .exists());
        assert!(!path
            .parent()
            .expect("workflow parent")
            .join("memory-candidates.v1.json")
            .exists());

        let _ = fs::remove_dir_all(path.parent().expect("workflow parent"));
    }

    #[test]
    fn memory_capture_revision_conflict_does_not_overwrite_store() {
        let path = temp_workflow_path("memory-capture-revision-conflict");
        let project_root = "/tmp/memory-capture-revision-conflict-project";
        let mut input = capture_input(project_root);
        input.candidate_policy = "audit_only".to_string();
        input.candidate = None;

        capture_event(
            &path,
            &input,
            "2026-06-09T12:31:20Z",
            "write-capture-conflict-1",
            "write-observation-conflict-1",
            "write-candidate-conflict-1",
        )
        .expect("first audit-only capture should be recorded");

        let mut stale_input = capture_input(project_root);
        stale_input.summary =
            "J2-B B2 second worker report summary for stale revision check.".to_string();
        stale_input.candidate_policy = "audit_only".to_string();
        stale_input.candidate = None;
        stale_input.expected_capture_store_revision = Some(0);

        let err = capture_event(
            &path,
            &stale_input,
            "2026-06-09T12:31:21Z",
            "write-capture-conflict-2",
            "write-observation-conflict-2",
            "write-candidate-conflict-2",
        )
        .unwrap_err();

        assert!(err.contains("memory_capture_store_conflict"));
        let store = load_store(&path, "2026-06-09T12:31:22Z").expect("store should remain valid");
        assert_eq!(store.revision, 1);
        assert_eq!(store.events.len(), 1);

        let _ = fs::remove_dir_all(path.parent().expect("workflow parent"));
    }

    #[test]
    fn memory_capture_corrupt_json_is_rejected_without_overwrite() {
        let path = temp_workflow_path("memory-capture-corrupt-json");
        let project_root = "/tmp/memory-capture-corrupt-json-project";
        let sidecar = sidecar_path(&path).expect("sidecar path should be derived");
        fs::create_dir_all(sidecar.parent().expect("sidecar parent")).expect("sidecar parent");
        let corrupt = "{ this is not valid memory capture json";
        fs::write(&sidecar, corrupt).expect("corrupt sidecar should be written");

        let err = capture_event(
            &path,
            &capture_input(project_root),
            "2026-06-09T12:31:30Z",
            "write-capture-corrupt",
            "write-observation-corrupt",
            "write-candidate-corrupt",
        )
        .unwrap_err();

        assert!(err.contains("memory capture sidecar JSON 损坏"));
        assert_eq!(
            fs::read_to_string(&sidecar).expect("corrupt sidecar should remain"),
            corrupt
        );
        assert!(!path
            .parent()
            .expect("workflow parent")
            .join("observations.v1.json")
            .exists());
        assert!(!path
            .parent()
            .expect("workflow parent")
            .join("memory-candidates.v1.json")
            .exists());

        let _ = fs::remove_dir_all(path.parent().expect("workflow parent"));
    }

    #[test]
    fn memory_capture_rejects_secret_candidate_path() {
        let path = temp_workflow_path("memory-capture-secret");
        let project_root = "/tmp/memory-capture-secret-project";
        let mut input = capture_input(project_root);
        input.sensitivity = "secret".to_string();

        let err = capture_event(
            &path,
            &input,
            "2026-06-09T12:32:00Z",
            "write-capture-secret",
            "write-observation-secret",
            "write-candidate-secret",
        )
        .unwrap_err();

        assert!(err.contains("secret memory capture 必须使用 blocked_sensitive"));
        assert!(!path
            .parent()
            .expect("workflow parent")
            .join("memory-capture-events.v1.json")
            .exists());

        let _ = fs::remove_dir_all(path.parent().expect("workflow parent"));
    }

    #[test]
    fn memory_capture_rejects_prompt_body_text() {
        let path = temp_workflow_path("memory-capture-prompt-body");
        let project_root = "/tmp/memory-capture-prompt-body-project";
        let mut input = capture_input(project_root);
        input.summary = "This includes prompt body and must be rejected.".to_string();

        let err = capture_event(
            &path,
            &input,
            "2026-06-09T12:33:00Z",
            "write-capture-prompt",
            "write-observation-prompt",
            "write-candidate-prompt",
        )
        .unwrap_err();

        assert!(err.contains("memory_capture_forbidden_sensitive_text"));
        assert!(!path
            .parent()
            .expect("workflow parent")
            .join("memory-capture-events.v1.json")
            .exists());

        let _ = fs::remove_dir_all(path.parent().expect("workflow parent"));
    }
}
