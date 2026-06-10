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
