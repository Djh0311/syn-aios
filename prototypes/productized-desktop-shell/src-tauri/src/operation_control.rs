use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::Path;

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct OperationControlReadModel {
    pub(crate) schema_version: String,
    pub(crate) generated_at: String,
    pub(crate) status_contract: Vec<String>,
    pub(crate) operations: Vec<OperationControlItem>,
    pub(crate) audit_boundary: OperationAuditBoundary,
    pub(crate) runtime_boundary: OperationRuntimeBoundary,
    pub(crate) readback_boundary: OperationReadbackBoundary,
    pub(crate) memory_capture_boundary: OperationMemoryCaptureBoundary,
    pub(crate) true_operation_available: bool,
    pub(crate) k3_b2_unlocked: bool,
    pub(crate) user_summary: Vec<String>,
    pub(crate) warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct OperationControlItem {
    pub(crate) operation_id: String,
    pub(crate) label: String,
    pub(crate) status: String,
    pub(crate) applies_to: String,
    pub(crate) would_write_if_real: String,
    pub(crate) current_gate: String,
    pub(crate) does_execute_in_l3: bool,
    pub(crate) status_after_confirmation: String,
    pub(crate) requires_separate_authorized_window: bool,
    pub(crate) risk_disclosure: String,
    pub(crate) confirmation_label: String,
    pub(crate) audit_event_type: String,
    pub(crate) runtime_status_after_confirmation: String,
    pub(crate) readback_status: String,
    pub(crate) readback_result_count: Option<i64>,
    pub(crate) blocks_k3_b2: bool,
    pub(crate) user_visible_summary: String,
    pub(crate) developer_details: Vec<OperationDeveloperDetail>,
    pub(crate) warnings: Vec<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct OperationDeveloperDetail {
    pub(crate) label: String,
    pub(crate) value: String,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct OperationAuditBoundary {
    pub(crate) event_type: String,
    pub(crate) records_actor: bool,
    pub(crate) records_operation: bool,
    pub(crate) records_risk_acknowledgement: bool,
    pub(crate) records_supervisor_review: bool,
    pub(crate) stores_sensitive_material: bool,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct OperationRuntimeBoundary {
    pub(crate) records_operation_kind: bool,
    pub(crate) records_operation_status: bool,
    pub(crate) records_pending_state: bool,
    pub(crate) real_process_control: bool,
    pub(crate) stores_prompt_body: bool,
    pub(crate) stores_codex_home_content: bool,
    pub(crate) allowed_summary: String,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct OperationReadbackBoundary {
    pub(crate) status: String,
    pub(crate) result_count: Option<i64>,
    pub(crate) unavailable_reason: String,
    pub(crate) real_readback_performed: bool,
    pub(crate) user_submitted_evidence_only: bool,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct OperationMemoryCaptureBoundary {
    pub(crate) capture_event_allowed: bool,
    pub(crate) observation_allowed: bool,
    pub(crate) candidate_allowed: bool,
    pub(crate) formal_memory_auto_write: bool,
    pub(crate) suggested_candidate_text: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct OperationDecisionInput<'a> {
    pub(crate) operation_id: &'a str,
    pub(crate) current_status: &'a str,
    pub(crate) actor_ref: &'a str,
    pub(crate) risk_acknowledged: bool,
    pub(crate) duplicate_scope: &'a str,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct OperationDecisionRecord {
    pub(crate) operation_id: String,
    pub(crate) status_after_decision: String,
    pub(crate) real_operation_executed: bool,
    pub(crate) real_codex_executed: bool,
    pub(crate) k3_b2_unlocked: bool,
    pub(crate) readback_status: String,
    pub(crate) readback_result_count: Option<i64>,
    pub(crate) audit_event_type: String,
    pub(crate) runtime_status: String,
    pub(crate) duplicate_scope: String,
    pub(crate) warnings: Vec<String>,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct OperationControlDecisionRequest {
    pub(crate) operation_id: String,
    pub(crate) label: String,
    pub(crate) current_status: String,
    pub(crate) status_after_confirmation: String,
    pub(crate) current_gate: String,
    pub(crate) would_write_if_real: String,
    pub(crate) risk_disclosure: String,
    pub(crate) readback_status: String,
    pub(crate) readback_result_count: Option<i64>,
    pub(crate) audit_event_type: String,
    pub(crate) runtime_status_after_confirmation: String,
    pub(crate) does_execute_in_l3: bool,
    pub(crate) requires_separate_authorized_window: bool,
    pub(crate) blocks_k3_b2: bool,
}

pub(crate) const OPERATION_CONTROL_STATES: [&str; 6] = [
    "not_applicable",
    "available",
    "pending_confirmation",
    "confirmed_recorded",
    "rejected",
    "blocked",
];

pub(crate) fn derive_operation_control_read_model(
    generated_at: &str,
    workflow_state: Option<&Value>,
) -> OperationControlReadModel {
    let mut operations = vec![
        operation_item(
            "retry",
            "重试",
            "available",
            "failed_run_unit",
            "workbench_state_only",
            "requires_user_confirmation_and_new_authorized_window",
            "确认后只记录重试请求和风险确认；不会自动重试，不调用 runner。",
        ),
        operation_item(
            "stop",
            "停止",
            "available",
            "running_session_only",
            "workbench_state_only",
            "blocked_no_runtime_handle",
            "确认后只记录停止请求；当前没有真实 runtime handle，不会 kill 进程或会话。",
        ),
        operation_item(
            "restart",
            "重启",
            "available",
            "existing_or_running_session",
            "codex_home_and_workbench_state",
            "blocked_restart_semantics_not_defined",
            "确认后只记录重启意图；不会新建会话、resume 旧会话或重跑任务。",
        ),
        operation_item(
            "resume",
            "恢复",
            "available",
            "bound_or_existing_session",
            "codex_home_and_workbench_state",
            "gated_real_resume_mario_test_only",
            "确认后只记录恢复决策；不会进入 real-resume phase B，也不扩大既有门。",
        ),
    ];
    let recorded_count = apply_decision_audits(&mut operations, workflow_state);
    let mut user_summary = vec![
        "retry / stop / restart / resume 现在是可确认、可审计、可回收的产品控制面。".to_string(),
        "确认只记录决策和待处理状态，不执行真实操作，不显示成功。".to_string(),
        "真实 retry / stop / restart / resume 必须另开独立授权窗口。".to_string(),
    ];
    if recorded_count > 0 {
        user_summary.push(format!(
            "已有 {recorded_count} 个操作决策从 workflow-state audit 投影为 confirmed_recorded。"
        ));
    }

    OperationControlReadModel {
        schema_version: "operation_control_read_model.v1".to_string(),
        generated_at: generated_at.to_string(),
        status_contract: OPERATION_CONTROL_STATES
            .iter()
            .map(|value| value.to_string())
            .collect(),
        operations,
        audit_boundary: OperationAuditBoundary {
            event_type: "operation_decision_recorded".to_string(),
            records_actor: true,
            records_operation: true,
            records_risk_acknowledgement: true,
            records_supervisor_review: true,
            stores_sensitive_material: false,
        },
        runtime_boundary: OperationRuntimeBoundary {
            records_operation_kind: true,
            records_operation_status: true,
            records_pending_state: true,
            real_process_control: false,
            stores_prompt_body: false,
            stores_codex_home_content: false,
            allowed_summary: "只记录操作决策、待处理状态和引用；不记录 prompt body、secret、完整 transcript 或 .codex 原文。".to_string(),
        },
        readback_boundary: OperationReadbackBoundary {
            status: "not_attempted_l3_decision_only".to_string(),
            result_count: None,
            unavailable_reason: "L3 不执行真实操作；readback 未发生，结果数未知/不可用，不能显示为 0 条。".to_string(),
            real_readback_performed: false,
            user_submitted_evidence_only: true,
        },
        memory_capture_boundary: OperationMemoryCaptureBoundary {
            capture_event_allowed: true,
            observation_allowed: true,
            candidate_allowed: true,
            formal_memory_auto_write: false,
            suggested_candidate_text: "用户对 retry / stop / restart / resume 发起了产品确认；该决策已记录待处理，未触发真实执行，真实操作仍需独立授权。".to_string(),
        },
        true_operation_available: false,
        k3_b2_unlocked: false,
        user_summary,
        warnings: vec![
            "operation_control_l3_decision_only".to_string(),
            "confirmed_recorded_is_not_executed".to_string(),
            "no_real_retry_stop_restart_resume".to_string(),
            "k3_b2_remains_blocked".to_string(),
        ],
    }
}

pub(crate) fn derive_read_model_from_state(
    generated_at: &str,
    workflow_state: Option<&Value>,
) -> OperationControlReadModel {
    derive_operation_control_read_model(generated_at, workflow_state)
}

pub(crate) fn read(generated_at: &str, workflow_state_path: &Path) -> OperationControlReadModel {
    let workflow_state = crate::read_workflow_state_value(workflow_state_path).ok();
    derive_read_model_from_state(generated_at, workflow_state.as_ref())
}

pub(crate) fn record_operation_decision(
    input: OperationDecisionInput<'_>,
) -> Result<OperationDecisionRecord, String> {
    if !["retry", "stop", "restart", "resume"].contains(&input.operation_id) {
        return Err(format!(
            "operation_control_unknown_operation: {}",
            input.operation_id
        ));
    }
    if input.current_status == "confirmed_recorded" {
        return Err(format!(
            "operation_control_duplicate: {} already confirmed in {}",
            input.operation_id, input.duplicate_scope
        ));
    }
    if input.current_status == "blocked" {
        return Err(format!(
            "operation_control_blocked: {} cannot bypass the current gate",
            input.operation_id
        ));
    }
    if !input.risk_acknowledged {
        return Err("operation_control_risk_acknowledgement_required".to_string());
    }

    Ok(OperationDecisionRecord {
        operation_id: input.operation_id.to_string(),
        status_after_decision: "confirmed_recorded".to_string(),
        real_operation_executed: false,
        real_codex_executed: false,
        k3_b2_unlocked: false,
        readback_status: "not_attempted_l3_decision_only".to_string(),
        readback_result_count: None,
        audit_event_type: "operation_decision_recorded".to_string(),
        runtime_status: "operation_decision_recorded_pending_real_authorization".to_string(),
        duplicate_scope: input.duplicate_scope.to_string(),
        warnings: vec![
            format!("actor_ref:{}", input.actor_ref),
            "confirmed_recorded_is_not_success".to_string(),
            "requires_separate_authorized_window_for_real_operation".to_string(),
        ],
    })
}

pub(crate) fn record_operation_control_decision_at(
    workflow_state_path: &Path,
    request: &OperationControlDecisionRequest,
    timestamp: &str,
) -> Result<crate::WorkflowStateMutationResult, String> {
    if request.does_execute_in_l3 {
        return Err("operation_control_decision_must_not_execute_in_l3".to_string());
    }
    if request.status_after_confirmation != "confirmed_recorded" {
        return Err(format!(
            "operation_control_invalid_after_status: {}",
            request.status_after_confirmation
        ));
    }
    if request.readback_result_count.is_some() {
        return Err("operation_control_readback_result_count_must_remain_null".to_string());
    }
    if !request.requires_separate_authorized_window {
        return Err(
            "operation_control_requires_separate_authorized_window_must_remain_true".to_string(),
        );
    }
    if !request.blocks_k3_b2 {
        return Err("operation_control_blocks_k3_b2_must_remain_true".to_string());
    }
    if request.audit_event_type != "operation_decision_recorded" {
        return Err(format!(
            "operation_control_invalid_audit_event_type: {}",
            request.audit_event_type
        ));
    }

    let decision = record_operation_decision(OperationDecisionInput {
        operation_id: &request.operation_id,
        current_status: &request.current_status,
        actor_ref: "user_confirmed_desktop_shell",
        risk_acknowledged: true,
        duplicate_scope: &format!("operation-control:{}", request.operation_id),
    })?;

    let mut value = crate::read_workflow_state_value(workflow_state_path)?;
    let validation_warnings = crate::validate_workflow_state(&value);
    if !validation_warnings.is_empty() {
        return Err(format!(
            "当前状态文件未通过 schema 校验：{}",
            validation_warnings.join(", ")
        ));
    }
    reject_duplicate_audit(&value, &request.operation_id)?;

    let backup = crate::backup_workflow_state_file(workflow_state_path, timestamp)?;
    let audit_event_id = crate::workflow_audit::audit_event_identity(
        "l3-operation",
        &request.operation_id,
        timestamp,
    );
    let mut audit_event = crate::workflow_audit::operation_decision_recorded(
        crate::workflow_audit::OperationDecisionRecordedAudit {
            event_id: audit_event_id.clone(),
            operation_id: &request.operation_id,
            before_state: &request.current_status,
            after_state: &decision.status_after_decision,
            created_at: timestamp,
            actor_ref: "user_confirmed_desktop_shell",
            current_gate: &request.current_gate,
            risk_acknowledged: true,
            supervisor_review_required: true,
        },
    );
    enrich_audit_event(&mut audit_event, request, &decision);

    crate::array_mut(&mut value, "audit_events")?.push(audit_event);
    value["updated_at"] = Value::String(timestamp.to_string());
    crate::write_m5b_batch2_workflow_state(
        workflow_state_path,
        "operation_decision_recorded",
        &value,
    )?;
    let snapshot = crate::read_workflow_state_snapshot(workflow_state_path)?;

    Ok(crate::WorkflowStateMutationResult {
        message: format!(
            "{} 决策已写入 workflow-state 审计；状态为 confirmed_recorded，未执行真实操作。",
            request.label
        ),
        path: workflow_state_path.display().to_string(),
        backup_path: Some(backup.display().to_string()),
        audit_event_id,
        first_initialize: false,
        snapshot,
    })
}

fn operation_item(
    operation_id: &str,
    label: &str,
    status: &str,
    applies_to: &str,
    would_write_if_real: &str,
    current_gate: &str,
    risk_disclosure: &str,
) -> OperationControlItem {
    OperationControlItem {
        operation_id: operation_id.to_string(),
        label: label.to_string(),
        status: status.to_string(),
        applies_to: applies_to.to_string(),
        would_write_if_real: would_write_if_real.to_string(),
        current_gate: current_gate.to_string(),
        does_execute_in_l3: false,
        status_after_confirmation: "confirmed_recorded".to_string(),
        requires_separate_authorized_window: true,
        risk_disclosure: risk_disclosure.to_string(),
        confirmation_label: format!("确认记录 {label} 决策"),
        audit_event_type: "operation_decision_recorded".to_string(),
        runtime_status_after_confirmation: "operation_decision_recorded_pending_real_authorization"
            .to_string(),
        readback_status: "not_attempted_l3_decision_only".to_string(),
        readback_result_count: None,
        blocks_k3_b2: true,
        user_visible_summary: format!("{label} 在 L3 只会记录决策，不会执行真实操作。"),
        developer_details: vec![
            detail("operation_id", operation_id),
            detail("current_gate", current_gate),
            detail("status_after_confirmation", "confirmed_recorded"),
            detail("does_execute_in_l3", "false"),
        ],
        warnings: vec![
            "decision_only_control_surface".to_string(),
            "confirmed_recorded_is_not_executed".to_string(),
        ],
    }
}

fn apply_decision_audits(
    operations: &mut [OperationControlItem],
    workflow_state: Option<&Value>,
) -> usize {
    let Some(audit_events) = workflow_state
        .and_then(|value| value.get("audit_events"))
        .and_then(Value::as_array)
    else {
        return 0;
    };
    let mut count = 0;
    for operation in operations {
        let Some(event) = audit_events.iter().rev().find(|event| {
            event.get("event_type").and_then(Value::as_str) == Some("operation_decision_recorded")
                && event.get("operation_id").and_then(Value::as_str)
                    == Some(operation.operation_id.as_str())
        }) else {
            continue;
        };
        if event.get("after_state").and_then(Value::as_str) == Some("confirmed_recorded") {
            operation.status = "confirmed_recorded".to_string();
            operation.current_gate = event
                .get("current_gate")
                .and_then(Value::as_str)
                .unwrap_or(operation.current_gate.as_str())
                .to_string();
            operation.confirmation_label = format!("已记录 {} 决策", operation.label);
            operation.user_visible_summary = format!(
                "{} 决策已从 workflow-state audit 投影为 confirmed_recorded；仍未执行真实操作。",
                operation.label
            );
            operation
                .warnings
                .push("decision_audit_recorded".to_string());
            count += 1;
        }
    }
    count
}

fn reject_duplicate_audit(value: &Value, operation_id: &str) -> Result<(), String> {
    let duplicate = value
        .get("audit_events")
        .and_then(Value::as_array)
        .unwrap_or(&vec![])
        .iter()
        .any(|event| {
            event.get("event_type").and_then(Value::as_str) == Some("operation_decision_recorded")
                && event.get("operation_id").and_then(Value::as_str) == Some(operation_id)
                && event.get("after_state").and_then(Value::as_str) == Some("confirmed_recorded")
        });
    if duplicate {
        return Err(format!(
            "operation_control_duplicate: {operation_id} already has confirmed_recorded audit"
        ));
    }
    Ok(())
}

fn enrich_audit_event(
    audit_event: &mut Value,
    request: &OperationControlDecisionRequest,
    decision: &OperationDecisionRecord,
) {
    if let Some(object) = audit_event.as_object_mut() {
        object.insert("label".to_string(), json!(request.label));
        object.insert(
            "would_write_if_real".to_string(),
            json!(request.would_write_if_real),
        );
        object.insert(
            "risk_disclosure".to_string(),
            json!(request.risk_disclosure),
        );
        object.insert(
            "readback_status".to_string(),
            json!(decision.readback_status),
        );
        object.insert("readback_result_count".to_string(), Value::Null);
        object.insert(
            "runtime_status_after_confirmation".to_string(),
            json!(decision.runtime_status),
        );
        object.insert(
            "does_execute_in_l3".to_string(),
            json!(request.does_execute_in_l3),
        );
        object.insert(
            "requires_separate_authorized_window".to_string(),
            json!(request.requires_separate_authorized_window),
        );
        object.insert("blocks_k3_b2".to_string(), json!(request.blocks_k3_b2));
        object.insert(
            "warnings".to_string(),
            json!([
                "operation_decision_is_not_real_execution",
                "confirmed_recorded_is_not_success",
                "readback_result_count_unknown_not_zero"
            ]),
        );
    }
}

fn detail(label: &str, value: &str) -> OperationDeveloperDetail {
    OperationDeveloperDetail {
        label: label.to_string(),
        value: value.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn l3_operation_contract_covers_four_controls_without_execution() {
        let model = derive_operation_control_read_model("2026-06-16T00:00:00Z", None);

        assert_eq!(model.schema_version, "operation_control_read_model.v1");
        assert_eq!(model.operations.len(), 4);
        for expected in ["retry", "stop", "restart", "resume"] {
            let operation = model
                .operations
                .iter()
                .find(|item| item.operation_id == expected)
                .expect("operation should exist");
            assert!(!operation.does_execute_in_l3);
            assert_ne!(operation.status_after_confirmation, "executed");
            assert_ne!(operation.status_after_confirmation, "succeeded");
            assert!(operation.requires_separate_authorized_window);
            assert_eq!(operation.readback_result_count, None);
        }

        assert!(!model.true_operation_available);
        assert!(!model.k3_b2_unlocked);
        assert_eq!(model.readback_boundary.result_count, None);
    }

    #[test]
    fn l3_confirmed_recorded_is_a_recoverable_decision_not_success() {
        let decision = record_operation_decision(OperationDecisionInput {
            operation_id: "resume",
            current_status: "pending_confirmation",
            actor_ref: "user_confirmed_desktop_shell",
            risk_acknowledged: true,
            duplicate_scope: "run-unit:l3-fixture:resume",
        })
        .expect("resume decision should be recorded");

        assert_eq!(decision.status_after_decision, "confirmed_recorded");
        assert!(!decision.real_operation_executed);
        assert!(!decision.real_codex_executed);
        assert!(!decision.k3_b2_unlocked);
        assert_eq!(decision.readback_status, "not_attempted_l3_decision_only");
        assert_eq!(decision.readback_result_count, None);
        assert_eq!(decision.audit_event_type, "operation_decision_recorded");
    }

    #[test]
    fn l3_duplicate_and_blocked_operations_do_not_auto_route() {
        let duplicate = record_operation_decision(OperationDecisionInput {
            operation_id: "retry",
            current_status: "confirmed_recorded",
            actor_ref: "user_confirmed_desktop_shell",
            risk_acknowledged: true,
            duplicate_scope: "run-unit:l3-fixture:retry",
        })
        .expect_err("duplicate confirmed operation should be rejected");
        assert!(duplicate.contains("operation_control_duplicate"));

        let blocked = record_operation_decision(OperationDecisionInput {
            operation_id: "stop",
            current_status: "blocked",
            actor_ref: "user_confirmed_desktop_shell",
            risk_acknowledged: true,
            duplicate_scope: "run-unit:l3-fixture:stop",
        })
        .expect_err("blocked operation must not bypass the gate");
        assert!(blocked.contains("operation_control_blocked"));
    }

    #[test]
    fn l3_operation_decision_writes_audit_without_real_execution_or_zero_readback() {
        let dir = std::env::temp_dir().join(format!(
            "operation-control-audit-{}",
            crate::unix_timestamp_nanos()
        ));
        fs::create_dir_all(&dir).expect("fixture dir should exist");
        let path = dir.join("workflow-state.v0.json");
        let initial = crate::initial_workflow_state_json(
            "2026-06-16T00:00:00Z",
            "audit:init:l3-operation-control",
            false,
            &path,
        );
        crate::atomic_write_json(&path, &initial).expect("initial workflow state should write");

        let request = fixture_decision_request("resume");
        let result = record_operation_control_decision_at(&path, &request, "2026-06-16T12:00:00Z")
            .expect("operation control decision should write audit");

        assert!(result.audit_event_id.contains("audit:l3-operation:resume"));
        assert_eq!(result.snapshot.counts.audit_events, 2);
        let updated = crate::read_workflow_state_value(&path).expect("state should read");
        let model = derive_operation_control_read_model("2026-06-16T12:00:01Z", Some(&updated));
        let resume = model
            .operations
            .iter()
            .find(|operation| operation.operation_id == "resume")
            .expect("resume operation should exist");
        assert_eq!(resume.status, "confirmed_recorded");
        assert!(resume
            .warnings
            .contains(&"decision_audit_recorded".to_string()));
        assert!(model
            .user_summary
            .iter()
            .any(|line| line.contains("workflow-state audit")));
        let event = updated["audit_events"]
            .as_array()
            .expect("audit events should be array")
            .iter()
            .find(|event| event["event_type"] == "operation_decision_recorded")
            .expect("operation decision audit should exist");
        assert_eq!(event["operation_id"], "resume");
        assert_eq!(event["after_state"], "confirmed_recorded");
        assert_eq!(event["real_operation_executed"], false);
        assert_eq!(event["real_codex_executed"], false);
        assert_eq!(event["k3_b2_unlocked"], false);
        assert_eq!(event["readback_result_count"], Value::Null);
        assert_eq!(event["does_execute_in_l3"], false);

        let duplicate =
            record_operation_control_decision_at(&path, &request, "2026-06-16T12:01:00Z")
                .expect_err("duplicate operation decision should be blocked");
        assert!(duplicate.contains("operation_control_duplicate"));

        let serialized = serde_json::to_string(event).expect("event should serialize");
        for forbidden in [
            "prompt body",
            "secret=",
            "full transcript",
            "/Users/yoyi/.codex/state",
            "result_count: 0",
        ] {
            assert!(
                !serialized.contains(forbidden),
                "operation audit leaked forbidden fragment {forbidden}"
            );
        }

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn l3_operation_decision_rejects_execution_claims_before_write() {
        let dir = std::env::temp_dir().join(format!(
            "operation-control-reject-{}",
            crate::unix_timestamp_nanos()
        ));
        fs::create_dir_all(&dir).expect("fixture dir should exist");
        let path = dir.join("workflow-state.v0.json");
        let initial = crate::initial_workflow_state_json(
            "2026-06-16T00:00:00Z",
            "audit:init:l3-operation-control-reject",
            false,
            &path,
        );
        crate::atomic_write_json(&path, &initial).expect("initial workflow state should write");

        let mut request = fixture_decision_request("retry");
        request.does_execute_in_l3 = true;
        let error = record_operation_control_decision_at(&path, &request, "2026-06-16T12:00:00Z")
            .expect_err("execution claim should be rejected");
        assert!(error.contains("operation_control_decision_must_not_execute_in_l3"));
        let updated = crate::read_workflow_state_value(&path).expect("state should read");
        assert_eq!(updated["audit_events"].as_array().unwrap().len(), 1);

        let mut weak_window = fixture_decision_request("retry");
        weak_window.requires_separate_authorized_window = false;
        let error =
            record_operation_control_decision_at(&path, &weak_window, "2026-06-16T12:01:00Z")
                .expect_err("weak authorization window claim should be rejected");
        assert!(error
            .contains("operation_control_requires_separate_authorized_window_must_remain_true"));

        let mut weak_gate = fixture_decision_request("retry");
        weak_gate.blocks_k3_b2 = false;
        let error = record_operation_control_decision_at(&path, &weak_gate, "2026-06-16T12:02:00Z")
            .expect_err("K3-B2 unblock claim should be rejected");
        assert!(error.contains("operation_control_blocks_k3_b2_must_remain_true"));

        let _ = fs::remove_dir_all(dir);
    }

    fn fixture_decision_request(operation_id: &str) -> OperationControlDecisionRequest {
        OperationControlDecisionRequest {
            operation_id: operation_id.to_string(),
            label: "恢复".to_string(),
            current_status: "available".to_string(),
            status_after_confirmation: "confirmed_recorded".to_string(),
            current_gate: "gated_real_resume_mario_test_only".to_string(),
            would_write_if_real: "codex_home_and_workbench_state".to_string(),
            risk_disclosure: "确认后只记录恢复决策；不会进入 real-resume phase B。".to_string(),
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
}
