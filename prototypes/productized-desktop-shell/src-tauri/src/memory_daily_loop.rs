use crate::{
    operation_control::OperationControlDecisionRequest, CaptureMemoryEventInput,
    CaptureMemoryEventOutput, MemoryCaptureCandidateDraft, MemoryCaptureSourceRef, MemoryScope,
};
use std::path::Path;

pub(crate) struct MemoryDailyLoopContext<'a> {
    pub(crate) project_root: &'a str,
    pub(crate) project_id: &'a str,
    pub(crate) workflow_id: &'a str,
    pub(crate) workflow_node_id: Option<&'a str>,
    pub(crate) run_unit_id: Option<&'a str>,
    pub(crate) actor_id: &'a str,
    pub(crate) created_at: &'a str,
}

pub(crate) struct MemoryDailyLoopWriteIds<'a> {
    pub(crate) capture_write_id: &'a str,
    pub(crate) observation_write_id: &'a str,
    pub(crate) candidate_write_id: &'a str,
}

pub(crate) fn operation_control_decision_capture_input(
    context: &MemoryDailyLoopContext<'_>,
    operation: &OperationControlDecisionRequest,
) -> Result<CaptureMemoryEventInput, String> {
    if operation.does_execute_in_l3 {
        return Err("memory_daily_loop_rejects_executed_operation_capture".to_string());
    }
    if operation.status_after_confirmation != "confirmed_recorded" {
        return Err("memory_daily_loop_requires_confirmed_recorded_operation".to_string());
    }
    if !operation.requires_separate_authorized_window || !operation.blocks_k3_b2 {
        return Err("memory_daily_loop_requires_l3_safety_boundaries".to_string());
    }

    let source_id = format!(
        "operation-control:{}:{}",
        operation.operation_id, context.created_at
    );
    Ok(CaptureMemoryEventInput {
        project_root: context.project_root.to_string(),
        project_id: Some(context.project_id.to_string()),
        workflow_id: Some(context.workflow_id.to_string()),
        workflow_node_id: context.workflow_node_id.map(str::to_string),
        run_unit_id: context.run_unit_id.map(str::to_string),
        product_command_id: None,
        product_attempt_id: None,
        runtime_log_ref: Some(format!(
            "runtime-log:operation-control:{}:{}",
            operation.operation_id, context.created_at
        )),
        audit_refs: vec![format!(
            "audit:operation-control:{}:{}",
            operation.operation_id, context.created_at
        )],
        readback_ref: Some(format!(
            "readback:operation-control:{}:not-attempted",
            operation.operation_id
        )),
        task_package_ref: None,
        memory_packet_ref: None,
        scope: MemoryScope {
            scope_id: format!("scope:l5-operation-control:{}", operation.operation_id),
            scope_type: "workflow".to_string(),
            user_id: None,
            project_id: Some(context.project_id.to_string()),
            workflow_id: Some(context.workflow_id.to_string()),
            session_id: None,
            role_ids: vec!["user".to_string(), "project_director".to_string()],
            document_refs: Vec::new(),
            permission_policy_ref: None,
            model_export_policy: "local_only".to_string(),
            valid_from: context.created_at.to_string(),
            valid_until: None,
        },
        source_type: "operation_control_decision".to_string(),
        source_refs: vec![MemoryCaptureSourceRef {
            source_ref_id: format!("source:l5-operation-control:{}", operation.operation_id),
            source_type: "operation_control_decision".to_string(),
            source_id,
            project_id: Some(context.project_id.to_string()),
            workflow_id: Some(context.workflow_id.to_string()),
            workflow_node_id: context.workflow_node_id.map(str::to_string),
            run_unit_id: context.run_unit_id.map(str::to_string),
            product_command_id: None,
            product_attempt_id: None,
            runtime_log_ref: Some(operation.runtime_status_after_confirmation.clone()),
            audit_ref_id: Some(operation.audit_event_type.clone()),
            readback_ref: Some(operation.readback_status.clone()),
            task_package_ref: None,
            memory_packet_ref: None,
            evidence_ref: None,
            summary: format!(
                "{} operation control decision recorded; real operation remains separately authorized.",
                operation.label
            ),
            sensitive_level: "internal".to_string(),
            created_at: context.created_at.to_string(),
        }],
        summary: format!(
            "用户确认记录 {} 操作控制决策；该决策待处理且没有触发真实运行。",
            operation.label
        ),
        evidence_summary: format!(
            "L3 operation control status={} gate={} readback={}；结果数保持未知/不可用。",
            operation.status_after_confirmation, operation.current_gate, operation.readback_status
        ),
        sensitivity: "internal".to_string(),
        candidate_policy: "candidate_allowed".to_string(),
        generated_by_role: "project_director".to_string(),
        actor_id: context.actor_id.to_string(),
        risk_level: "medium".to_string(),
        reason: "L5 daily loop captures operation control decisions as reviewable memory candidates; capture does not write FormalMemory."
            .to_string(),
        candidate: Some(MemoryCaptureCandidateDraft {
            memory_type: "workflow_summary".to_string(),
            claim: format!(
                "L3 {} 操作控制确认只登记决策，不执行真实操作。",
                operation.label
            ),
            body: format!(
                "用户在运行控制面确认 {}；状态进入 confirmed_recorded，真实操作仍需另窗授权，K3-B2 仍阻断。",
                operation.label
            ),
            review_reason: "从 L5 daily operation capture 生成待确认候选；候选不是正式记忆。".to_string(),
            requires_user_confirmation: true,
            actor_role: "project_director".to_string(),
        }),
        expected_capture_store_revision: None,
        expected_observation_store_revision: None,
        expected_candidate_store_revision: None,
    })
}

pub(crate) fn capture_daily_memory_event(
    workflow_state_path: &Path,
    input: &CaptureMemoryEventInput,
    timestamp: &str,
    write_ids: &MemoryDailyLoopWriteIds<'_>,
) -> Result<CaptureMemoryEventOutput, String> {
    let mut output = crate::memory_capture_bus::capture_event(
        workflow_state_path,
        input,
        timestamp,
        write_ids.capture_write_id,
        write_ids.observation_write_id,
        write_ids.candidate_write_id,
    )?;
    if !output
        .warnings
        .iter()
        .any(|warning| warning == "memory_daily_loop_capture_requires_user_confirmation")
    {
        output
            .warnings
            .push("memory_daily_loop_capture_requires_user_confirmation".to_string());
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{formal_memory_store, memory_candidate_store, memory_capture_bus};
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_workflow_path(label: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("{label}-{nanos}"));
        fs::create_dir_all(&dir).expect("temp dir");
        dir.join("workflow-state.v0.json")
    }

    fn fixture_context() -> MemoryDailyLoopContext<'static> {
        MemoryDailyLoopContext {
            project_root: "/tmp/l5-memory-daily-loop-project",
            project_id: "project:tmp-l5-memory-daily-loop-project",
            workflow_id: "workflow:tmp-l5-memory-daily-loop-project:default",
            workflow_node_id: Some("workflow:tmp-l5-memory-daily-loop-project:default:node:runner"),
            run_unit_id: Some("run-unit:l5-operation-control"),
            actor_id: "user:l5-daily-loop",
            created_at: "2026-06-16T12:00:00Z",
        }
    }

    fn fixture_operation() -> OperationControlDecisionRequest {
        OperationControlDecisionRequest {
            operation_id: "resume".to_string(),
            label: "恢复".to_string(),
            current_status: "available".to_string(),
            status_after_confirmation: "confirmed_recorded".to_string(),
            current_gate: "gated_real_resume_mario_test_only".to_string(),
            would_write_if_real: "codex_home_and_workbench_state".to_string(),
            risk_disclosure: "L3 只登记恢复决策，不执行真实 resume。".to_string(),
            readback_status: "not_attempted_l3_decision_only".to_string(),
            readback_result_count: None,
            audit_event_type: "operation_decision_recorded".to_string(),
            runtime_status_after_confirmation:
                "operation_decision_recorded_pending_real_authorization".to_string(),
            does_execute_in_l3: false,
            requires_separate_authorized_window: true,
            blocks_k3_b2: true,
        }
    }

    #[test]
    fn l5_operation_control_capture_input_is_candidate_allowed_with_source_refs() {
        let input =
            operation_control_decision_capture_input(&fixture_context(), &fixture_operation())
                .expect("operation control capture input");

        assert_eq!(input.source_type, "operation_control_decision");
        assert_eq!(input.candidate_policy, "candidate_allowed");
        assert_eq!(input.generated_by_role, "project_director");
        assert_eq!(input.source_refs.len(), 1);
        assert_eq!(
            input.source_refs[0].source_type,
            "operation_control_decision"
        );
        assert!(input.candidate.is_some());
        assert!(
            input
                .candidate
                .as_ref()
                .expect("candidate draft")
                .requires_user_confirmation
        );
        assert!(!input.summary.to_ascii_lowercase().contains("prompt body"));
        assert!(!input.summary.contains("/Users/yoyi/.codex"));
    }

    #[test]
    fn l5_daily_capture_creates_observation_and_candidate_without_formal_memory() {
        let path = temp_workflow_path("l5-memory-daily-loop-capture");
        let input =
            operation_control_decision_capture_input(&fixture_context(), &fixture_operation())
                .expect("operation control capture input");

        let output = capture_daily_memory_event(
            &path,
            &input,
            "2026-06-16T12:00:00Z",
            &MemoryDailyLoopWriteIds {
                capture_write_id: "write-l5-capture",
                observation_write_id: "write-l5-observation",
                candidate_write_id: "write-l5-candidate",
            },
        )
        .expect("daily capture should create observation and candidate");

        assert!(output.observation.is_some());
        assert!(output.candidate.is_some());
        assert!(output
            .warnings
            .contains(&"formal_memory_not_written_by_memory_capture".to_string()));
        assert!(memory_capture_bus::sidecar_path(&path)
            .expect("capture path")
            .exists());
        assert!(memory_candidate_store::sidecar_path(&path)
            .expect("candidate path")
            .exists());
        assert!(
            !formal_memory_store::sidecar_path(&path)
                .expect("formal path")
                .exists(),
            "daily capture must not create formal memory sidecar"
        );

        let _ = fs::remove_dir_all(path.parent().expect("workflow parent"));
    }
}
