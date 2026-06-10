use crate::{
    codex_local_runner, h4_execution_boundary, session_continuation_store, task_memory_injection,
    CodexLocalActiveAttempt, CodexLocalAuditRef, CodexLocalExecutionRequest,
    CodexLocalReadbackPlan, CodexLocalRuntimeLogRef, H5DiagnosticSummaryInput,
    H5PermissionEnvelopePreview, H5ProjectWorkflowDispatchPreview,
    H5ProjectWorkflowDispatchPreviewInput, H5ReadbackBoundaryPreview, H5RuntimeAuditPreview,
    H5TaskMemoryPacketDispatchSummary, ObservationSourceRef,
    ProjectDirectorProcessFactDecisionInput, WorkerStructuredReportInput,
};
use serde_json::Value;
use std::path::Path;

pub(crate) fn preview_h5_project_workflow_dispatch_at(
    workflow_state_path: &Path,
    request: &H5ProjectWorkflowDispatchPreviewInput,
) -> Result<H5ProjectWorkflowDispatchPreview, String> {
    let timestamp = crate::unix_timestamp_string();
    let value = crate::read_workflow_state_value(workflow_state_path)?;
    let dispatch = crate::find_workflow_node_dispatch(&value, &request.dispatch_id)
        .ok_or_else(|| format!("找不到 prepared dispatch：{}", request.dispatch_id))?;
    let dispatch_state =
        crate::optional_string_from(dispatch, "state").unwrap_or_else(|| "missing".to_string());

    let workflow_node_id = require_field(dispatch, "node_id")?;
    let work_item_id = require_field(dispatch, "work_item_id")?;
    let dispatch_project_id = require_field(dispatch, "project_id")?;
    let dispatch_workflow_id = require_field(dispatch, "workflow_id")?;
    let authorization_id = crate::optional_string_from(dispatch, "plan_authorization_id");
    let task_package_id = crate::optional_string_from(dispatch, "task_package_id");
    let target_session_id = request
        .session_id
        .clone()
        .or_else(|| crate::optional_string_from(dispatch, "native_thread_id"));
    let artifact = crate::find_task_package_artifact_by_id(&value, &work_item_id);
    let memory_snapshot = artifact.and_then(task_memory_injection::snapshot_from_artifact);
    let memory_summary = memory_summary(workflow_state_path, memory_snapshot.as_ref(), &timestamp);
    let operation_id = request.operation_id.clone().unwrap_or_else(|| {
        if target_session_id.is_some() {
            "resume".to_string()
        } else {
            "new_session".to_string()
        }
    });
    let allowed_write_roots = artifact
        .map(|artifact| crate::string_array(artifact, "allowed_write"))
        .filter(|items| !items.is_empty())
        .unwrap_or_else(|| vec![request.project_root.clone()]);
    let runtime_log_refs = vec![CodexLocalRuntimeLogRef {
        ref_id: format!("runtime-log-preview:h5:{}", request.dispatch_id),
        category: "dispatch_attempt".to_string(),
        status: "preview_only_not_written".to_string(),
        redaction_status: "redacted_safe_summary".to_string(),
    }];
    let audit_refs = vec![CodexLocalAuditRef {
        ref_id: format!("audit-preview:h5-permission:{}", request.dispatch_id),
        event_type: "h5_level_a_permission_preview".to_string(),
        actor_role: request.actor_id.clone(),
        decision: "preview_not_approved".to_string(),
    }];
    let active_attempts = active_attempts_for_scope(
        workflow_state_path,
        &workflow_node_id,
        &work_item_id,
        target_session_id.as_deref(),
        &timestamp,
    );
    let mut blocked_reasons = Vec::new();
    let mut warnings = vec![
        "h5_level_a_preview_only_no_real_runner".to_string(),
        "prompt_body_not_included".to_string(),
    ];

    if dispatch_state != "prepared" {
        blocked_reasons.push(format!("dispatch_state_not_prepared:{dispatch_state}"));
    }
    if dispatch_project_id != request.project_id {
        blocked_reasons.push("dispatch_project_mismatch".to_string());
    }
    if dispatch_workflow_id != request.workflow_id {
        blocked_reasons.push("dispatch_workflow_mismatch".to_string());
    }
    if request.prompt_summary.trim().is_empty()
        || request.prompt_ref.trim().is_empty()
        || request.prompt_sha256.trim().is_empty()
    {
        blocked_reasons.push("prompt_summary_ref_or_hash_missing".to_string());
    }
    if memory_summary.snapshot_id.is_none() {
        blocked_reasons.push("task_memory_packet_snapshot_missing".to_string());
    }
    if memory_summary.stale {
        blocked_reasons.push("task_memory_packet_stale".to_string());
    }
    if operation_id == "resume" && target_session_id.is_none() {
        blocked_reasons.push("resume_target_session_missing".to_string());
    }
    if operation_id == "new_session" && !request.h3_b_level_b_authorized {
        blocked_reasons.push("new_session_waiting_h3_b_retry_or_level_b_authorization".to_string());
    }
    if !active_attempts.is_empty() {
        blocked_reasons.push("duplicate_dispatch_blocked".to_string());
    }
    if let Some(expected) = request.expected_workflow_revision {
        if workflow_revision(&value) != Some(expected) {
            blocked_reasons.push("workflow_revision_mismatch".to_string());
        }
    }

    let diagnostic_blockers = diagnostic_blockers(request.diagnostic_summary.as_ref());
    if !diagnostic_blockers.is_empty() {
        blocked_reasons.push("diagnostics_blocking_degraded".to_string());
    }
    if request.diagnostic_summary.is_none() {
        warnings.push("g2_diagnostic_summary_not_supplied_level_b_must_check".to_string());
    }

    let codex_request = CodexLocalExecutionRequest {
        request_version: 1,
        adapter_id: "codex-local".to_string(),
        operation_id: operation_id.clone(),
        project_id: request.project_id.clone(),
        project_root: request.project_root.clone(),
        workflow_id: request.workflow_id.clone(),
        node_id: workflow_node_id.clone(),
        session_id: target_session_id.clone(),
        work_item_id: Some(work_item_id.clone()),
        continuation_id: None,
        target_cwd: request
            .target_cwd
            .clone()
            .unwrap_or_else(|| request.project_root.clone()),
        allowed_write_roots: allowed_write_roots.clone(),
        sandbox: request
            .sandbox
            .clone()
            .unwrap_or_else(|| "workspace-write".to_string()),
        prompt_source_kind: "task_package_prompt_ref".to_string(),
        prompt_summary: request.prompt_summary.clone(),
        prompt_sha256: request.prompt_sha256.clone(),
        prompt_ref: request.prompt_ref.clone(),
        readback_plan: CodexLocalReadbackPlan {
            strategy: "required".to_string(),
            required: true,
            expected_sources: vec![
                "runtime_log_ref".to_string(),
                "audit_ref".to_string(),
                "worker_report_candidate".to_string(),
            ],
            unavailable_behavior:
                "readback_unavailable_or_failed_keeps_result_count_null_and_blocks_final_acceptance"
                    .to_string(),
            trust_policy: "workbench_managed_refs_only_no_full_transcript_by_default".to_string(),
            warnings: vec![h4_execution_boundary::h4_unknown_result_warning()],
        },
        requested_by: request.actor_id.clone(),
        user_confirmation_state: "h5_level_a_preview_not_approved".to_string(),
        authorization_scope_id: authorization_id.clone(),
        runtime_log_refs: runtime_log_refs.clone(),
        audit_refs: audit_refs.clone(),
        active_attempts,
        warnings: vec![
            "h5_level_a_request_preview_only".to_string(),
            "permission_envelope_not_approved_for_real_execution".to_string(),
        ],
    };
    let guard = codex_local_runner::inspect_codex_local_execution_guard(&codex_request);
    if guard.blocks_execution {
        blocked_reasons.extend(
            guard
                .reasons
                .iter()
                .map(|reason| format!("h1_guard:{reason}")),
        );
    }

    let status = if blocked_reasons.is_empty() {
        "ready_for_level_b_permission_preview_not_executed"
    } else {
        "blocked_for_level_b_not_executed"
    }
    .to_string();
    let permission = H5PermissionEnvelopePreview {
        status: "awaiting_explicit_level_b_authorization".to_string(),
        explicit_approval_required: true,
        approved_for_real_execution: false,
        adapter_id: "codex-local".to_string(),
        operation_id: operation_id.clone(),
        target_session_id: target_session_id.clone(),
        cwd: codex_request.target_cwd.clone(),
        project_root: request.project_root.clone(),
        allowed_write_roots,
        denied_paths: vec![
            "/Users/yoyi/.codex unless Level B explicitly authorizes minimal scope".to_string(),
            "auth/token/secret/.env/keychain/OAuth/provider credential".to_string(),
            "full transcript/rollout by default".to_string(),
        ],
        prompt_summary: request.prompt_summary.clone(),
        prompt_ref: request.prompt_ref.clone(),
        prompt_sha256: request.prompt_sha256.clone(),
        memory_packet_fingerprint: memory_summary.fingerprint.clone(),
        readback_boundary:
            "readback_unavailable/readback_failed/timed_out => result_count=null".to_string(),
        codex_home_boundary:
            "Level A does not read or write /Users/yoyi/.codex; Level B must authorize minimal side effects."
                .to_string(),
        warnings: vec![
            "do_not_show_codex_received_task_before_real_execution".to_string(),
            "do_not_show_worker_running_before_real_execution".to_string(),
        ],
    };
    let readback_boundary = H5ReadbackBoundaryPreview {
        status: "not_attempted".to_string(),
        result_count: h4_execution_boundary::h4_result_count(
            "not_attempted",
            "not_attempted",
            Some(0),
        ),
        unavailable_behavior:
            "unknown result remains null; never display failed readback as 0 results".to_string(),
        worker_report_candidate_allowed: false,
        warnings: vec![h4_execution_boundary::h4_unknown_result_warning()],
    };
    let worker_report_candidate = Some(worker_report_candidate(
        request,
        &workflow_node_id,
        &work_item_id,
        &timestamp,
    ));
    let process_fact_handoff = Some(process_fact_handoff(
        request,
        &workflow_node_id,
        &work_item_id,
        &timestamp,
    ));

    Ok(H5ProjectWorkflowDispatchPreview {
        preview_version: 1,
        preview_id: format!("h5-level-a-preview:{}:{timestamp}", request.dispatch_id),
        status,
        level: "h5_level_a_non_real_product_path_preview".to_string(),
        project_id: request.project_id.clone(),
        workflow_id: request.workflow_id.clone(),
        workflow_node_id,
        work_item_id,
        dispatch_id: request.dispatch_id.clone(),
        task_package_id,
        operation_id,
        target_session_id,
        memory_packet: memory_summary,
        permission_envelope: permission,
        codex_local_request: Some(codex_request),
        codex_local_guard: Some(guard),
        runtime_audit_preview: H5RuntimeAuditPreview {
            runtime_log_refs,
            audit_refs,
            diagnostic_status: request
                .diagnostic_summary
                .as_ref()
                .map(|summary| summary.overall_severity.clone())
                .unwrap_or_else(|| "not_supplied".to_string()),
            diagnostic_blockers,
            warnings: vec!["runtime_log_and_audit_refs_are_preview_only_not_written".to_string()],
        },
        readback_boundary,
        worker_report_candidate,
        process_fact_handoff,
        final_review_handoff_status:
            "c6_final_review_requires_worker_report_and_project_director_process_fact_decision"
                .to_string(),
        prompt_sent: false,
        real_codex_executed: false,
        writes_codex_home: false,
        writes_project_files: false,
        writes_workbench_state: false,
        blocked_reasons: crate::dedupe_strings(blocked_reasons),
        warnings: crate::dedupe_strings(warnings),
    })
}

fn memory_summary(
    workflow_state_path: &Path,
    snapshot: Option<&crate::TaskPackageMemoryPacketSnapshot>,
    timestamp: &str,
) -> H5TaskMemoryPacketDispatchSummary {
    let Some(snapshot) = snapshot else {
        return H5TaskMemoryPacketDispatchSummary {
            snapshot_id: None,
            fingerprint: None,
            included_count: 0,
            excluded_count: 0,
            review_material_count: 0,
            stale: true,
            stale_reasons: vec!["task_memory_packet_snapshot_missing".to_string()],
            warnings: vec!["task_memory_packet_snapshot_missing".to_string()],
        };
    };
    let stale_reasons =
        task_memory_injection::current_store_revisions(workflow_state_path, timestamp)
            .map(|current| task_memory_injection::stale_reasons(snapshot, &current))
            .unwrap_or_else(|error| vec![format!("memory_packet_revision_check_failed:{error}")]);
    let stale = snapshot.stale || !stale_reasons.is_empty();
    let mut warnings = snapshot.warnings.clone();
    if stale {
        warnings.push("task_memory_packet_snapshot_stale".to_string());
    }
    H5TaskMemoryPacketDispatchSummary {
        snapshot_id: Some(snapshot.snapshot_id.clone()),
        fingerprint: Some(snapshot.fingerprint.clone()),
        included_count: snapshot.included_memories.len(),
        excluded_count: snapshot.excluded_items.len(),
        review_material_count: snapshot.review_materials.len(),
        stale,
        stale_reasons,
        warnings: crate::dedupe_strings(warnings),
    }
}

fn active_attempts_for_scope(
    workflow_state_path: &Path,
    workflow_node_id: &str,
    work_item_id: &str,
    target_session_id: Option<&str>,
    timestamp: &str,
) -> Vec<CodexLocalActiveAttempt> {
    let Ok(store) = session_continuation_store::load_store(workflow_state_path, timestamp) else {
        return Vec::new();
    };
    store
        .attempts
        .iter()
        .filter(|attempt| h4_execution_boundary::is_h4_active_attempt_status(&attempt.status))
        .filter_map(|attempt| {
            let continuation = store
                .continuations
                .iter()
                .find(|item| item.continuation_id == attempt.continuation_id)?;
            let same_work_item = continuation.work_item_id.as_deref() == Some(work_item_id);
            let same_node = continuation.node_id == workflow_node_id;
            let same_session =
                target_session_id.is_some_and(|session_id| continuation.session_id == session_id);
            if same_work_item || same_node || same_session {
                Some(CodexLocalActiveAttempt {
                    attempt_id: attempt.attempt_id.clone(),
                    status: attempt.status.clone(),
                    continuation_id: Some(attempt.continuation_id.clone()),
                })
            } else {
                None
            }
        })
        .collect()
}

fn diagnostic_blockers(summary: Option<&H5DiagnosticSummaryInput>) -> Vec<String> {
    let Some(summary) = summary else {
        return Vec::new();
    };
    let mut blockers = Vec::new();
    if summary.blocked_count > 0 || summary.overall_severity == "blocked" {
        blockers.push("diagnostic_summary_blocked".to_string());
    }
    for state in &summary.degraded_states {
        if state.blocks_real_execution {
            blockers.push(state.kind.clone());
        }
    }
    crate::dedupe_strings(blockers)
}

fn worker_report_candidate(
    request: &H5ProjectWorkflowDispatchPreviewInput,
    workflow_node_id: &str,
    work_item_id: &str,
    timestamp: &str,
) -> WorkerStructuredReportInput {
    WorkerStructuredReportInput {
        project_root: request.project_root.clone(),
        project_id: request.project_id.clone(),
        workflow_id: request.workflow_id.clone(),
        workflow_node_id: workflow_node_id.to_string(),
        work_item_id: work_item_id.to_string(),
        dispatch_id: Some(request.dispatch_id.clone()),
        actor_role: "codex_local_worker_candidate".to_string(),
        executed_what: "Level A preview only; worker did not execute.".to_string(),
        changed_what: "No project files changed; no prompt was sent.".to_string(),
        summary: "H5 Level A generated a worker report candidate shape only.".to_string(),
        evidence_refs: vec![format!("dispatch:{}", request.dispatch_id)],
        open_issues: vec!["worker_report_candidate_not_formal_fact".to_string()],
        permission_requests: vec!["h5_level_b_real_execution_authorization_required".to_string()],
        direction_risks: vec!["readback_unknown_must_not_be_counted_as_zero".to_string()],
        follow_up_suggestions: vec![
            "After real Level B readback succeeds, record C5 worker report through existing command."
                .to_string(),
        ],
        acceptance_status: "reported_not_completed".to_string(),
        source_refs: vec![source_ref(request, "worker_report_candidate", timestamp)],
        expected_workflow_revision: request.expected_workflow_revision,
    }
}

fn process_fact_handoff(
    request: &H5ProjectWorkflowDispatchPreviewInput,
    _workflow_node_id: &str,
    work_item_id: &str,
    _timestamp: &str,
) -> ProjectDirectorProcessFactDecisionInput {
    ProjectDirectorProcessFactDecisionInput {
        project_root: request.project_root.clone(),
        project_id: request.project_id.clone(),
        workflow_id: request.workflow_id.clone(),
        report_id: format!("h5-worker-report-candidate:{}", request.dispatch_id),
        actor_id: request.actor_id.clone(),
        actor_role: "project_director".to_string(),
        decision: "request_rework".to_string(),
        accepted_facts: Vec::new(),
        rejected_fact_ids: vec![format!("process-fact-candidate:{work_item_id}")],
        summary: "Level A preview cannot confirm process fact; real C5 decision remains required."
            .to_string(),
        expected_workflow_revision: request.expected_workflow_revision,
        expected_observation_store_revision: None,
    }
}

fn source_ref(
    request: &H5ProjectWorkflowDispatchPreviewInput,
    source_kind: &str,
    timestamp: &str,
) -> ObservationSourceRef {
    ObservationSourceRef {
        source_ref_id: format!("source-ref:h5:{}:{source_kind}", request.dispatch_id),
        source_kind: "workflow_event".to_string(),
        source_id: request.dispatch_id.clone(),
        project_id: Some(request.project_id.clone()),
        workflow_id: Some(request.workflow_id.clone()),
        session_id: request.session_id.clone(),
        file_path: None,
        evidence_ref: None,
        summary: "H5 Level A preview source ref; not a formal memory source.".to_string(),
        sensitive_level: "low".to_string(),
        created_at: timestamp.to_string(),
    }
}

fn require_field(value: &Value, key: &str) -> Result<String, String> {
    crate::optional_string_from(value, key)
        .ok_or_else(|| format!("prepared dispatch 缺少字段：{key}"))
}

fn workflow_revision(value: &Value) -> Option<i64> {
    value.get("revision").and_then(Value::as_i64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    static FIXTURE_COUNTER: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn h5_preview_builds_codex_request_without_real_execution() {
        let (dir, path) = fixture_state(false, false);
        let request = preview_request(dir.display().to_string(), "resume");
        let output = preview_h5_project_workflow_dispatch_at(&path, &request).unwrap();

        assert_eq!(output.level, "h5_level_a_non_real_product_path_preview");
        assert!(!output.prompt_sent);
        assert!(!output.real_codex_executed);
        assert!(!output.writes_codex_home);
        assert_eq!(output.readback_boundary.result_count, None);
        assert_eq!(
            output.memory_packet.fingerprint.as_deref(),
            Some("memory-fingerprint-1")
        );
        assert!(output.codex_local_request.is_some());
        assert!(output
            .codex_local_guard
            .as_ref()
            .unwrap()
            .reasons
            .contains(&"user_confirmation_required".to_string()));
        assert!(output
            .worker_report_candidate
            .as_ref()
            .unwrap()
            .open_issues
            .contains(&"worker_report_candidate_not_formal_fact".to_string()));
    }

    #[test]
    fn h5_preview_blocks_stale_memory_and_new_session_without_h3_b_authorization() {
        let (dir, path) = fixture_state(true, false);
        let request = preview_request(dir.display().to_string(), "new_session");
        let output = preview_h5_project_workflow_dispatch_at(&path, &request).unwrap();

        assert_eq!(output.status, "blocked_for_level_b_not_executed");
        assert!(output
            .blocked_reasons
            .contains(&"task_memory_packet_stale".to_string()));
        assert!(output
            .blocked_reasons
            .contains(&"new_session_waiting_h3_b_retry_or_level_b_authorization".to_string()));
    }

    #[test]
    fn h5_preview_blocks_duplicate_active_attempt() {
        let (dir, path) = fixture_state(false, true);
        let request = preview_request(dir.display().to_string(), "resume");
        let output = preview_h5_project_workflow_dispatch_at(&path, &request).unwrap();

        assert!(output
            .blocked_reasons
            .contains(&"duplicate_dispatch_blocked".to_string()));
        assert!(
            output
                .codex_local_guard
                .as_ref()
                .unwrap()
                .duplicate_running_attempt
        );
    }

    #[test]
    fn h5_preview_blocks_diagnostics_and_missing_prompt_without_real_execution() {
        let (dir, path) = fixture_state(false, false);
        let mut request = preview_request(dir.display().to_string(), "resume");
        request.prompt_ref = "".to_string();
        request.diagnostic_summary = Some(H5DiagnosticSummaryInput {
            overall_severity: "blocked".to_string(),
            blocked_count: 1,
            degraded_states: vec![crate::H5DiagnosticDegradedStateInput {
                kind: "diagnostics:blocking_fixture".to_string(),
                blocks_real_execution: true,
            }],
        });
        let output = preview_h5_project_workflow_dispatch_at(&path, &request).unwrap();

        assert_eq!(output.status, "blocked_for_level_b_not_executed");
        assert!(!output.prompt_sent);
        assert!(!output.real_codex_executed);
        assert!(!output.writes_codex_home);
        assert_eq!(output.readback_boundary.result_count, None);
        assert!(output
            .blocked_reasons
            .contains(&"prompt_summary_ref_or_hash_missing".to_string()));
        assert!(output
            .blocked_reasons
            .contains(&"diagnostics_blocking_degraded".to_string()));
        assert_eq!(
            output.runtime_audit_preview.diagnostic_blockers.as_slice(),
            &[
                "diagnostic_summary_blocked".to_string(),
                "diagnostics:blocking_fixture".to_string()
            ]
        );
    }

    fn fixture_state(
        stale_memory: bool,
        duplicate_attempt: bool,
    ) -> (std::path::PathBuf, std::path::PathBuf) {
        let dir = std::env::temp_dir().join(format!(
            "h5-project-dispatch-{}-{}",
            crate::unix_timestamp_nanos(),
            FIXTURE_COUNTER.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("workflow-state.v0.json");
        let snapshot = json!({
            "snapshot_id": "snapshot-1",
            "schema_version": "task_package_memory_packet_snapshot.v1",
            "source_packet_id": "packet-1",
            "project_id": "project:h5",
            "workflow_id": "workflow:h5",
            "work_item_id": "work-item:h5",
            "task_package_artifact_id": "artifact:h5",
            "role_id": "codex_worker",
            "retrieval_intent": "worker_task",
            "included_memories": [],
            "excluded_items": [],
            "review_materials": [],
            "store_revisions": {
                "formal_store_revision": 0,
                "candidate_store_revision": 0,
                "observation_store_revision": 0,
                "lint_store_revision": 0,
                "entity_relation_store_revision": 0
            },
            "estimated_tokens": 0,
            "max_estimated_tokens": 2000,
            "fingerprint": "memory-fingerprint-1",
            "generated_at": "2026-06-08T00:00:00Z",
            "stale": stale_memory,
                "stale_reasons": if stale_memory { vec!["fixture_stale"] } else { Vec::<&str>::new() },
            "warnings": []
        });
        let value = json!({
            "revision": 7,
            "workflow_node_dispatches": [{
                "dispatch_id": "dispatch:h5",
                "project_id": "project:h5",
                "workflow_id": "workflow:h5",
                "node_id": "node:h5",
                "work_item_id": "work-item:h5",
                "native_thread_id": "session:h5",
                "prompt_preview": "redacted prompt preview",
                "prompt_kind": "authorized_prepared_auto_dispatch",
                "memory_packet_snapshot_id": "snapshot-1",
                "memory_packet_fingerprint": "memory-fingerprint-1",
                "plan_authorization_id": "authorization:h5",
                "authorization_check": {"status": "authorized"},
                "state": "prepared"
            }],
            "artifacts": [{
                "artifact_id": "artifact:h5",
                "artifact_type": "task_package",
                "source_ref": "work-item:h5",
                "allowed_write": [dir.display().to_string()],
                "memory_packet_snapshot": snapshot
            }]
        });
        fs::write(&path, serde_json::to_string_pretty(&value).unwrap()).unwrap();

        if duplicate_attempt {
            let continuation = json!({
                "record_version": 1,
                "continuation_id": "continuation:h5",
                "preview_id": "preview:h5",
                "adapter_id": "codex-local",
                "operation_id": "resume",
                "project_id": "project:h5",
                    "project_root": dir.display().to_string(),
                "workflow_id": "workflow:h5",
                "node_id": "node:h5",
                "session_id": "session:h5",
                "work_item_id": "work-item:h5",
                "target_cwd": dir.display().to_string(),
                "allowed_write_roots": [dir.display().to_string()],
                "sandbox": "workspace-write",
                "prompt_source_kind": "task_package_prompt_ref",
                "prompt_summary": "summary",
                "command_preview": "redacted",
                "readback_strategy": "required",
                "status": "queued",
                "execution_level": "h5_fixture",
                "runner_kind": "fake",
                "user_confirmation_state": "confirmed",
                "guard_status": "allowed",
                "requested_by": "project_director",
                "confirmed_by": "user",
                "confirmation_reason": "fixture",
                "created_at": "2026-06-08T00:00:00Z",
                "updated_at": "2026-06-08T00:00:00Z",
                "audit_refs": [],
                "warnings": []
            });
            let attempt = json!({
                "attempt_version": 1,
                "attempt_id": "attempt:h5",
                "continuation_id": "continuation:h5",
                "runner_kind": "fake",
                "execution_level": "h5_fixture",
                "status": "running",
                "started_at": "2026-06-08T00:00:00Z",
                "finished_at": null,
                "timeout_ms": null,
                "command_preview": "redacted",
                "prompt_sent": false,
                "real_codex_executed": false,
                "writes_codex_home": false,
                "writes_workbench_state": true,
                "readback_summary": {
                    "status": "not_attempted",
                    "source_kind": "fixture",
                    "result_count": null,
                    "unavailable_reason": "running",
                    "warnings": []
                },
                "failure_reason": null,
                "audit_refs": [],
                "warnings": []
            });
            let store = json!({
                "schema_version": "session_continuation_store.v1",
                "store_version": 1,
                "storage_kind": "sidecar_json_v0",
                "scope": {
                    "scope_kind": "workflow_state_sidecar",
                    "workflow_state_path": path.display().to_string(),
                    "sidecar_path": dir.join("session-continuations.v1.json").display().to_string(),
                    "project_roots": [dir.display().to_string()]
                },
                "revision": 1,
                "last_write_id": null,
                "generated_by": "fixture",
                "created_at": "2026-06-08T00:00:00Z",
                "updated_at": "2026-06-08T00:00:00Z",
                "continuations": [continuation],
                "attempts": [attempt],
                "audit_events": [],
                "warnings": []
            });
            fs::write(
                dir.join("session-continuations.v1.json"),
                serde_json::to_string_pretty(&store).unwrap(),
            )
            .unwrap();
        }

        (dir, path)
    }

    fn preview_request(
        project_root: String,
        operation_id: &str,
    ) -> H5ProjectWorkflowDispatchPreviewInput {
        H5ProjectWorkflowDispatchPreviewInput {
            project_root: project_root.clone(),
            project_id: "project:h5".to_string(),
            workflow_id: "workflow:h5".to_string(),
            dispatch_id: "dispatch:h5".to_string(),
            actor_id: "project_director".to_string(),
            operation_id: Some(operation_id.to_string()),
            session_id: Some("session:h5".to_string()),
            target_cwd: Some(project_root),
            sandbox: Some("workspace-write".to_string()),
            prompt_summary: "H5 Level A safe preview".to_string(),
            prompt_ref: "workbench-managed:h5-preview:v1".to_string(),
            prompt_sha256: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                .to_string(),
            h3_b_level_b_authorized: false,
            expected_workflow_revision: Some(7),
            diagnostic_summary: None,
        }
    }
}
