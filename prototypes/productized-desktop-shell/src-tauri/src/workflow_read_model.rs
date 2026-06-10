use crate::WorkflowLedgerEntry;
use serde_json::Value;

pub(crate) fn derive_project_blackboards<TWorkflow, TBlackboard>(
    workflows: &[TWorkflow],
    derive_one: fn(&TWorkflow) -> TBlackboard,
) -> Vec<TBlackboard> {
    workflows.iter().map(derive_one).collect()
}

pub(crate) struct WorkflowLedgerDerivationFns {
    pub(crate) optional_string_from: fn(&Value, &str) -> Option<String>,
    pub(crate) string_array: fn(&Value, &str) -> Vec<String>,
    pub(crate) i64_value: fn(&Value, &str) -> Option<i64>,
    pub(crate) ledger_entry_type_from_audit: fn(&str) -> String,
    pub(crate) compact_ledger_summary: fn(&str) -> String,
}

pub(crate) fn derive_workflow_ledger_entries(
    workflow_id: &str,
    audit_events: &[Value],
    node_dispatches: &[Value],
    director_reviews: &[Value],
    permission_requests: &[Value],
    fns: WorkflowLedgerDerivationFns,
) -> Vec<WorkflowLedgerEntry> {
    let mut entries = Vec::new();
    for event in audit_events.iter().filter(|event| {
        (fns.optional_string_from)(event, "target_ref")
            .is_some_and(|target| target.contains(workflow_id))
            || (fns.optional_string_from)(event, "event_type").is_some_and(|event_type| {
                event_type.starts_with("task_")
                    || event_type.starts_with("workflow_")
                    || event_type.starts_with("offline_")
            })
    }) {
        let event_type = (fns.optional_string_from)(event, "event_type")
            .unwrap_or_else(|| "audit_event".to_string());
        entries.push(WorkflowLedgerEntry {
            ledger_entry_id: (fns.optional_string_from)(event, "event_id")
                .unwrap_or_else(|| "audit:missing".to_string()),
            workflow_id: workflow_id.to_string(),
            workflow_node_id: (fns.optional_string_from)(event, "node_id"),
            entry_type: (fns.ledger_entry_type_from_audit)(&event_type),
            actor_role: (fns.optional_string_from)(event, "actor_ref"),
            actor_session_id: None,
            summary: (fns.optional_string_from)(event, "reason")
                .unwrap_or_else(|| "未登记摘要".to_string()),
            source_refs: (fns.optional_string_from)(event, "target_ref")
                .into_iter()
                .collect(),
            tool_call_refs: vec![],
            audit_refs: (fns.optional_string_from)(event, "event_id")
                .into_iter()
                .collect(),
            risk_flags: (fns.string_array)(event, "risk_flags"),
            created_at: (fns.optional_string_from)(event, "created_at"),
        });
    }
    for dispatch in node_dispatches.iter().filter(|dispatch| {
        (fns.optional_string_from)(dispatch, "workflow_id").as_deref() == Some(workflow_id)
    }) {
        entries.push(WorkflowLedgerEntry {
            ledger_entry_id: format!(
                "ledger:dispatch:{}",
                (fns.optional_string_from)(dispatch, "dispatch_id")
                    .unwrap_or_else(|| "missing".to_string())
            ),
            workflow_id: workflow_id.to_string(),
            workflow_node_id: (fns.optional_string_from)(dispatch, "node_id"),
            entry_type: if (fns.optional_string_from)(dispatch, "prompt_kind").as_deref()
                == Some("tool_call_summary")
            {
                "tool_call_summary".to_string()
            } else {
                "subagent_started".to_string()
            },
            actor_role: Some("project_director".to_string()),
            actor_session_id: (fns.optional_string_from)(dispatch, "native_thread_id"),
            summary: (fns.compact_ledger_summary)(
                &(fns.optional_string_from)(dispatch, "prompt_preview")
                    .unwrap_or_else(|| "派发记录缺摘要".to_string()),
            ),
            source_refs: (fns.optional_string_from)(dispatch, "work_item_id")
                .into_iter()
                .collect(),
            tool_call_refs: (fns.optional_string_from)(dispatch, "tool_call_ref")
                .into_iter()
                .collect(),
            audit_refs: vec![],
            risk_flags: (fns.string_array)(dispatch, "warnings"),
            created_at: (fns.i64_value)(dispatch, "created_at_ms").map(|value| value.to_string()),
        });
    }
    for review in director_reviews.iter().filter(|review| {
        (fns.optional_string_from)(review, "workflow_id").as_deref() == Some(workflow_id)
    }) {
        entries.push(WorkflowLedgerEntry {
            ledger_entry_id: format!(
                "ledger:review:{}",
                (fns.optional_string_from)(review, "review_id")
                    .unwrap_or_else(|| "missing".to_string())
            ),
            workflow_id: workflow_id.to_string(),
            workflow_node_id: (fns.optional_string_from)(review, "workflow_node_id"),
            entry_type: "review_result".to_string(),
            actor_role: (fns.optional_string_from)(review, "reviewer_role"),
            actor_session_id: None,
            summary: (fns.optional_string_from)(review, "summary").unwrap_or_default(),
            source_refs: (fns.optional_string_from)(review, "work_item_id")
                .into_iter()
                .collect(),
            tool_call_refs: vec![],
            audit_refs: vec![],
            risk_flags: (fns.string_array)(review, "warnings"),
            created_at: (fns.optional_string_from)(review, "created_at"),
        });
    }
    for request in permission_requests.iter().filter(|request| {
        (fns.optional_string_from)(request, "workflow_id").as_deref() == Some(workflow_id)
    }) {
        let status =
            (fns.optional_string_from)(request, "status").unwrap_or_else(|| "pending".to_string());
        entries.push(WorkflowLedgerEntry {
            ledger_entry_id: format!(
                "ledger:permission:{}",
                (fns.optional_string_from)(request, "request_id")
                    .unwrap_or_else(|| "missing".to_string())
            ),
            workflow_id: workflow_id.to_string(),
            workflow_node_id: None,
            entry_type: match status.as_str() {
                "approved" => "permission_granted".to_string(),
                "rejected" => "permission_denied".to_string(),
                _ => "permission_requested".to_string(),
            },
            actor_role: Some("subagent_or_project_director".to_string()),
            actor_session_id: None,
            summary: (fns.optional_string_from)(request, "reason")
                .unwrap_or_else(|| "权限请求缺摘要".to_string()),
            source_refs: (fns.optional_string_from)(request, "work_item_id")
                .into_iter()
                .collect(),
            tool_call_refs: vec![],
            audit_refs: (fns.optional_string_from)(request, "audit_event_id")
                .into_iter()
                .collect(),
            risk_flags: (fns.string_array)(request, "warnings"),
            created_at: (fns.optional_string_from)(request, "requested_at"),
        });
    }
    entries
}
