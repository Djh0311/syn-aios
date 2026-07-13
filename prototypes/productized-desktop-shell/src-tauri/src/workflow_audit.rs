use serde_json::{json, Value};

pub(crate) fn audit_event_identity(
    kind_slug: &str,
    entity: &str,
    timestamp_ms: impl std::fmt::Display,
) -> String {
    let entity_slug = audit_identity_slug(entity);
    let entity_hash = crate::utils::hash::sha256_hex(entity);
    format!(
        "audit:{kind_slug}:{entity_slug}:{}:{timestamp_ms}",
        &entity_hash[..12]
    )
}

fn audit_identity_slug(entity: &str) -> String {
    let mut slug = String::new();
    for character in entity.chars() {
        if character.is_ascii_alphanumeric() {
            slug.push(character.to_ascii_lowercase());
        } else if !slug.ends_with('-') {
            slug.push('-');
        }
    }
    slug.trim_matches('-').to_string()
}

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

pub(crate) struct OperationDecisionRecordedAudit<'a> {
    pub(crate) event_id: String,
    pub(crate) operation_id: &'a str,
    pub(crate) before_state: &'a str,
    pub(crate) after_state: &'a str,
    pub(crate) created_at: &'a str,
    pub(crate) actor_ref: &'a str,
    pub(crate) current_gate: &'a str,
    pub(crate) risk_acknowledged: bool,
    pub(crate) supervisor_review_required: bool,
}

pub(crate) fn operation_decision_recorded(event: OperationDecisionRecordedAudit<'_>) -> Value {
    json!({
      "event_id": event.event_id,
      "event_type": "operation_decision_recorded",
      "target_ref": event.operation_id,
      "actor_ref": event.actor_ref,
      "source_kind": "l3_operation_control_product_path",
      "permission_level": "operation_decision_no_real_execution",
      "operation_id": event.operation_id,
      "before_state": event.before_state,
      "after_state": event.after_state,
      "current_gate": event.current_gate,
      "created_at": event.created_at,
      "risk_acknowledged": event.risk_acknowledged,
      "supervisor_review_required": event.supervisor_review_required,
      "real_operation_executed": false,
      "real_codex_executed": false,
      "k3_b2_unlocked": false,
      "stores_prompt_body": false,
      "stores_sensitive_material": false,
      "stores_codex_home_content": false,
      "reason": "记录 L3 operation control 决策；不调用 runner、不执行 Codex、不发送 prompt、不停止或重启真实进程。"
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn audit_event_identity_keeps_same_millisecond_batch_unique() {
        let timestamp_ms = 1_720_915_200_123_i64;
        let ids = [
            "dispatch:workflow:node:one",
            "dispatch:workflow:node:two",
            "dispatch:workflow:node:three",
            "dispatch:workflow:node:four",
        ]
        .into_iter()
        .map(|dispatch_id| {
            audit_event_identity("workflow-node-dispatch-prepared", dispatch_id, timestamp_ms)
        })
        .collect::<BTreeSet<_>>();

        assert_eq!(ids.len(), 4);
    }

    #[test]
    fn audit_event_identity_distinguishes_slug_fold_collisions() {
        let timestamp_ms = 1_720_915_200_123_i64;
        let hyphenated = audit_event_identity("worker-report", "a-b", timestamp_ms);
        let underscored = audit_event_identity("worker-report", "a_b", timestamp_ms);

        assert_eq!(
            hyphenated,
            "audit:worker-report:a-b:d44362d67d92:1720915200123"
        );
        assert_eq!(
            underscored,
            "audit:worker-report:a-b:648fa9b31bc7:1720915200123"
        );
        assert_ne!(hyphenated, underscored);
    }

    #[test]
    fn audit_event_identity_uses_full_slug_and_documented_format() {
        let entity = format!("worker-{}", "x".repeat(100));
        let timestamp_ms = 1_720_915_200_123_i64;
        let expected = format!(
            "audit:workflow-node-dispatch-prepared:{entity}:{}:{timestamp_ms}",
            &crate::utils::hash::sha256_hex(&entity)[..12]
        );

        assert_eq!(
            audit_event_identity("workflow-node-dispatch-prepared", &entity, timestamp_ms),
            expected
        );
    }

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

    #[test]
    fn operation_decision_audit_event_records_decision_without_execution_or_sensitive_payloads() {
        let event = operation_decision_recorded(OperationDecisionRecordedAudit {
            event_id: "audit:l3-operation:resume:v1".to_string(),
            operation_id: "resume",
            before_state: "pending_confirmation",
            after_state: "confirmed_recorded",
            created_at: "2026-06-16T00:00:00Z",
            actor_ref: "user_confirmed_desktop_shell",
            current_gate: "gated_real_resume_mario_test_only",
            risk_acknowledged: true,
            supervisor_review_required: true,
        });

        assert_eq!(event["event_type"], "operation_decision_recorded");
        assert_eq!(event["operation_id"], "resume");
        assert_eq!(event["after_state"], "confirmed_recorded");
        assert_eq!(event["real_operation_executed"], false);
        assert_eq!(event["real_codex_executed"], false);
        assert_eq!(event["k3_b2_unlocked"], false);
        assert_eq!(event["stores_prompt_body"], false);
        assert_eq!(event["stores_sensitive_material"], false);
        assert_eq!(event["stores_codex_home_content"], false);
        let serialized = serde_json::to_string(&event).expect("audit event should serialize");
        for forbidden in [
            "prompt body",
            "secret=",
            "full transcript",
            "/Users/yoyi/.codex/state",
        ] {
            assert!(
                !serialized.contains(forbidden),
                "operation audit leaked forbidden fragment {forbidden}"
            );
        }
    }
}
