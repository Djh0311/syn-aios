use serde_json::{json, Value};

pub(crate) struct WorkItemStateChangedAudit<'a> {
    pub(crate) event_id: String,
    pub(crate) work_item_id: &'a str,
    pub(crate) before_state: &'a str,
    pub(crate) after_state: &'a str,
    pub(crate) created_at: &'a str,
    pub(crate) reason: String,
}

pub(crate) fn work_item_state_changed(event: WorkItemStateChangedAudit<'_>) -> Value {
    json!({
      "event_id": event.event_id,
      "event_type": "work_item_state_changed",
      "target_ref": event.work_item_id,
      "actor_ref": "user_confirmed_desktop_shell",
      "source_kind": "workspace_state",
      "permission_level": "user_confirmed_write",
      "before_state": event.before_state,
      "after_state": event.after_state,
      "created_at": event.created_at,
      "reason": event.reason
    })
}

pub(crate) struct WorkflowPermissionDecisionRecordedAudit<'a> {
    pub(crate) event_id: String,
    pub(crate) request_id: &'a str,
    pub(crate) before_state: &'a str,
    pub(crate) after_state: &'a str,
    pub(crate) created_at: &'a str,
}

pub(crate) fn workflow_permission_decision_recorded(
    event: WorkflowPermissionDecisionRecordedAudit<'_>,
) -> Value {
    json!({
      "event_id": event.event_id,
      "event_type": "workflow_permission_decision_recorded",
      "target_ref": event.request_id,
      "actor_ref": "user_confirmed_desktop_shell",
      "source_kind": "workspace_state_permission_queue",
      "permission_level": "user_confirmed_write",
      "before_state": event.before_state,
      "after_state": event.after_state,
      "created_at": event.created_at,
      "reason": "用户确认记录权限请求结论；不启动 Codex、不 resume、不发送消息。"
    })
}

pub(crate) struct K3B1RecoveryDecisionRecordedAudit<'a> {
    pub(crate) event_id: String,
    pub(crate) execution_point_id: &'a str,
    pub(crate) recovery_choice: &'a str,
    pub(crate) before_state: &'a str,
    pub(crate) after_state: &'a str,
    pub(crate) created_at: &'a str,
    pub(crate) actor_ref: &'a str,
    pub(crate) risk_acknowledged: bool,
    pub(crate) supervisor_review_required: bool,
}

pub(crate) fn k3_b1_recovery_decision_recorded(
    event: K3B1RecoveryDecisionRecordedAudit<'_>,
) -> Value {
    json!({
      "event_id": event.event_id,
      "event_type": "k3_b1_recovery_decision_recorded",
      "target_ref": event.execution_point_id,
      "actor_ref": event.actor_ref,
      "source_kind": "k3_b1_recovery_product_path",
      "permission_level": "recovery_decision_no_real_execution",
      "recovery_choice": event.recovery_choice,
      "before_state": event.before_state,
      "after_state": event.after_state,
      "created_at": event.created_at,
      "risk_acknowledged": event.risk_acknowledged,
      "supervisor_review_required": event.supervisor_review_required,
      "stores_prompt_body": false,
      "stores_sensitive_material": false,
      "stores_codex_home_content": false,
      "reason": "记录 K3-B1 recovery 选择；不执行 Codex、不发送 prompt、不读取或写入 .codex。"
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn k3_b1_recovery_audit_event_records_choice_without_sensitive_payloads() {
        let event = k3_b1_recovery_decision_recorded(K3B1RecoveryDecisionRecordedAudit {
            event_id: "audit:k3-b1-recovery:manual:v1".to_string(),
            execution_point_id: "stage-k-k3-b1-mario-test-workflow-read-only",
            recovery_choice: "manual_exact_command_submission",
            before_state: "blocked_by_safety_review_again",
            after_state: "manual_recovery_needs_review",
            created_at: "2026-06-16T00:00:00Z",
            actor_ref: "user_confirmed_desktop_shell",
            risk_acknowledged: true,
            supervisor_review_required: true,
        });

        assert_eq!(event["event_type"], "k3_b1_recovery_decision_recorded");
        assert_eq!(
            event["target_ref"],
            "stage-k-k3-b1-mario-test-workflow-read-only"
        );
        assert_eq!(event["recovery_choice"], "manual_exact_command_submission");
        assert_eq!(event["after_state"], "manual_recovery_needs_review");
        assert_eq!(event["supervisor_review_required"], true);
        assert_eq!(event["stores_prompt_body"], false);
        assert_eq!(event["stores_sensitive_material"], false);
        assert_eq!(event["stores_codex_home_content"], false);
        let serialized = serde_json::to_string(&event).expect("audit event should serialize");
        assert!(!serialized.contains("prompt body"));
        assert!(!serialized.contains("secret="));
        assert!(!serialized.contains("/Users/yoyi/.codex/state"));
    }
}
