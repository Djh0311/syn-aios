use crate::{
    ControlledSessionContinuation, ReadbackBoundaryStatus, RuntimeAttentionSourceRef,
    RuntimeSessionAttention, SessionContinuationAttempt, SessionContinuationPreview,
    SessionContinuationStoreV1, SessionRunStatusSummary,
};
use std::collections::BTreeMap;

pub(crate) fn derive_runtime_session_attention(
    previews: &[SessionContinuationPreview],
    store: &SessionContinuationStoreV1,
    generated_at: &str,
) -> (Vec<RuntimeSessionAttention>, Vec<SessionRunStatusSummary>) {
    let mut attention = Vec::new();

    for preview in previews {
        if let Some(item) = attention_from_preview(preview, generated_at) {
            attention.push(item);
        }
    }

    let latest_attempts = latest_attempt_by_continuation(&store.attempts);
    for continuation in &store.continuations {
        if let Some(attempt) = latest_attempts.get(&continuation.continuation_id) {
            attention.push(attention_from_attempt(continuation, attempt, generated_at));
        } else {
            attention.push(attention_from_continuation_waiting(
                continuation,
                generated_at,
            ));
        }
    }

    attention.sort_by(|a, b| {
        severity_rank(&b.severity)
            .cmp(&severity_rank(&a.severity))
            .then_with(|| b.updated_at.cmp(&a.updated_at))
            .then_with(|| a.attention_id.cmp(&b.attention_id))
    });

    let summaries = summarize_attention(&attention);
    (attention, summaries)
}

fn attention_from_preview(
    preview: &SessionContinuationPreview,
    generated_at: &str,
) -> Option<RuntimeSessionAttention> {
    let guard = &preview.guard_result;
    if guard.status == "allowed_preview" && !guard.requires_user_confirmation {
        return None;
    }

    let (kind, severity, status, title, message, next_step, requires_user_action, blocks) =
        if guard.status == "blocked" || guard.blocks_execution {
            (
                "blocked_by_guard",
                "blocking",
                "blocked_by_guard",
                "会话继续被 guard 阻断",
                "当前预览没有进入执行；guard 已阻断，不能发送或 resume。",
                "先查看 required fixes；不要把阻断预览当成可执行任务。",
                false,
                true,
            )
        } else if guard.status == "needs_user_confirmation" || guard.requires_user_confirmation {
            (
                "waiting_permission",
                "needs_user",
                "waiting_permission",
                "会话继续等待用户确认",
                "该预览只说明目标会话、cwd、prompt 摘要和 readback 预期；确认前不会发送。",
                "查看预览边界；如要真实执行仍需后续 Level B 授权。",
                true,
                false,
            )
        } else {
            (
                "waiting_level_b_authorization",
                "warning",
                "waiting_level_b_authorization",
                "会话继续等待后续任务",
                "当前只到预览边界，尚未获得真实执行授权。",
                "保留为后续任务；不要显示成已发送或已读回。",
                false,
                true,
            )
        };

    let source_refs = vec![RuntimeAttentionSourceRef {
        source_kind: "session_continuation_preview".to_string(),
        source_id: preview.preview_id.clone(),
        label: format!(
            "{} {} {}",
            preview.adapter_id, preview.operation_id, guard.status
        ),
    }];
    let readback_boundary = readback_boundary(
        "readback_unavailable",
        if blocks {
            "guard_blocked"
        } else {
            "level_b_not_authorized"
        },
        false,
        false,
        None,
        "readback unavailable 是边界状态，不是 0 条结果。",
        "E4 preview 没有真实 readback 来源；E6 只派生关注状态。",
        source_refs.clone(),
        preview.readback_expectation.warnings.clone(),
    );

    Some(RuntimeSessionAttention {
        attention_id: format!("runtime-attention:preview:{}", preview.preview_id),
        project_id: preview.project_id.clone(),
        workflow_id: preview.workflow_id.clone(),
        node_id: preview.node_id.clone(),
        session_id: preview.target_session_id.clone(),
        adapter_id: preview.adapter_id.clone(),
        source_refs,
        kind: kind.to_string(),
        severity: severity.to_string(),
        status: status.to_string(),
        title: title.to_string(),
        user_message: message.to_string(),
        technical_summary: format!(
            "guard_status={} operation={} readback_strategy={}",
            guard.status, preview.operation_id, preview.readback_expectation.strategy
        ),
        recommended_next_step: next_step.to_string(),
        requires_user_action,
        blocks_continuation: blocks,
        readback_boundary,
        created_at: generated_at.to_string(),
        updated_at: generated_at.to_string(),
        warnings: preview
            .user_visible_warnings
            .iter()
            .chain(guard.warnings.iter())
            .cloned()
            .collect(),
    })
}

fn attention_from_continuation_waiting(
    continuation: &ControlledSessionContinuation,
    _generated_at: &str,
) -> RuntimeSessionAttention {
    let source_refs = vec![RuntimeAttentionSourceRef {
        source_kind: "controlled_session_continuation".to_string(),
        source_id: continuation.continuation_id.clone(),
        label: format!(
            "{} {} {}",
            continuation.adapter_id, continuation.operation_id, continuation.status
        ),
    }];
    RuntimeSessionAttention {
        attention_id: format!(
            "runtime-attention:continuation:{}",
            continuation.continuation_id
        ),
        project_id: Some(continuation.project_id.clone()),
        workflow_id: Some(continuation.workflow_id.clone()),
        node_id: Some(continuation.node_id.clone()),
        session_id: Some(continuation.session_id.clone()),
        adapter_id: continuation.adapter_id.clone(),
        source_refs: source_refs.clone(),
        kind: "waiting_level_b_authorization".to_string(),
        severity: "needs_user".to_string(),
        status: "waiting_level_b_authorization".to_string(),
        title: "受控 continuation 等待 Level B 授权".to_string(),
        user_message:
            "已存在工作台自有 confirmation record；真实 prompt 没有发送，仍不能执行真实 resume。"
                .to_string(),
        technical_summary: format!(
            "continuation_status={} execution_level={} runner_kind={}",
            continuation.status, continuation.execution_level, continuation.runner_kind
        ),
        recommended_next_step:
            "查看 E5 记录；是否进入真实执行需另行授权具体 session、cwd、prompt 和读写范围。"
                .to_string(),
        requires_user_action: true,
        blocks_continuation: false,
        readback_boundary: readback_boundary(
            "readback_unavailable",
            "level_b_not_authorized",
            false,
            false,
            None,
            "还没有真实 readback 来源；unavailable 不是 0 条结果。",
            "E5 Level A confirmation record 尚未进入真实 runner。",
            source_refs,
            continuation.warnings.clone(),
        ),
        created_at: continuation.created_at.clone(),
        updated_at: continuation.updated_at.clone(),
        warnings: continuation.warnings.clone(),
    }
}

fn attention_from_attempt(
    continuation: &ControlledSessionContinuation,
    attempt: &SessionContinuationAttempt,
    generated_at: &str,
) -> RuntimeSessionAttention {
    let source_refs = vec![
        RuntimeAttentionSourceRef {
            source_kind: "controlled_session_continuation".to_string(),
            source_id: continuation.continuation_id.clone(),
            label: format!("{} {}", continuation.adapter_id, continuation.operation_id),
        },
        RuntimeAttentionSourceRef {
            source_kind: "session_continuation_attempt".to_string(),
            source_id: attempt.attempt_id.clone(),
            label: format!("{} {}", attempt.runner_kind, attempt.status),
        },
    ];
    let status = attempt_status_kind(attempt);
    let severity = attempt_severity(status, attempt);
    let (title, user_message, next_step) = attempt_messages(status, attempt);
    let readback_boundary = boundary_from_attempt(attempt, source_refs.clone());

    RuntimeSessionAttention {
        attention_id: format!("runtime-attention:attempt:{}", attempt.attempt_id),
        project_id: Some(continuation.project_id.clone()),
        workflow_id: Some(continuation.workflow_id.clone()),
        node_id: Some(continuation.node_id.clone()),
        session_id: Some(continuation.session_id.clone()),
        adapter_id: continuation.adapter_id.clone(),
        source_refs,
        kind: status.to_string(),
        severity: severity.to_string(),
        status: status.to_string(),
        title: title.to_string(),
        user_message: user_message.to_string(),
        technical_summary: format!(
            "attempt_status={} execution_level={} prompt_sent={} real_codex_executed={} readback_status={}",
            attempt.status,
            attempt.execution_level,
            attempt.prompt_sent,
            attempt.real_codex_executed,
            attempt.readback_summary.status
        ),
        recommended_next_step: next_step.to_string(),
        requires_user_action: matches!(
            status,
            "failed_stub" | "timed_out" | "readback_failed" | "readback_unavailable"
        ),
        blocks_continuation: matches!(status, "failed_stub" | "timed_out" | "readback_failed"),
        readback_boundary,
        created_at: attempt.started_at.clone(),
        updated_at: attempt
            .finished_at
            .clone()
            .unwrap_or_else(|| generated_at.to_string()),
        warnings: attempt
            .warnings
            .iter()
            .chain(attempt.readback_summary.warnings.iter())
            .cloned()
            .collect(),
    }
}

fn boundary_from_attempt(
    attempt: &SessionContinuationAttempt,
    source_refs: Vec<RuntimeAttentionSourceRef>,
) -> ReadbackBoundaryStatus {
    let summary = &attempt.readback_summary;
    let status = if crate::h4_execution_boundary::h4_unknown_result_status(&attempt.status)
        && summary.status != "succeeded"
    {
        attempt.status.as_str()
    } else if summary.status == "readback_unavailable"
        || summary.status == "not_attempted_stub"
        || !attempt.real_codex_executed
    {
        "readback_unavailable"
    } else if summary.status.contains("failed") || attempt.status.contains("failed") {
        "readback_failed"
    } else if summary.status == "readback_timed_out" || attempt.status == "timed_out" {
        "readback_timed_out"
    } else {
        summary.status.as_str()
    };
    let reason = if !attempt.real_codex_executed {
        "not_attempted_stub"
    } else {
        summary
            .unavailable_reason
            .as_deref()
            .unwrap_or("unknown_failure")
    };
    let result_count = crate::h4_execution_boundary::h4_result_count(
        attempt.status.as_str(),
        status,
        summary.result_count,
    );
    readback_boundary(
        status,
        reason,
        attempt.real_codex_executed,
        attempt.real_codex_executed
            && !crate::h4_execution_boundary::h4_unknown_result_status(status),
        result_count,
        if status == "readback_failed" {
            "readback failed 代表读回失败或不可信；不能显示成 0 条结果。"
        } else if status == "readback_timed_out" || status == "timed_out" {
            "readback timed out 代表读回超时；不能显示成 0 条结果，也不会自动重试。"
        } else {
            "readback unavailable 代表没有真实读取来源；不是 0 条结果。"
        },
        &format!(
            "runner_kind={} execution_level={} source_kind={}",
            attempt.runner_kind, attempt.execution_level, summary.source_kind
        ),
        source_refs,
        summary.warnings.clone(),
    )
}

fn readback_boundary(
    status: &str,
    reason: &str,
    attempted: bool,
    real_readback_performed: bool,
    result_count: Option<i64>,
    user_message: &str,
    technical_summary: &str,
    source_refs: Vec<RuntimeAttentionSourceRef>,
    warnings: Vec<String>,
) -> ReadbackBoundaryStatus {
    ReadbackBoundaryStatus {
        status: status.to_string(),
        reason: reason.to_string(),
        attempted,
        real_readback_performed,
        result_count,
        user_message: user_message.to_string(),
        technical_summary: technical_summary.to_string(),
        source_refs,
        warnings,
    }
}

fn attempt_status_kind(attempt: &SessionContinuationAttempt) -> &str {
    if attempt.status == "running_stub" {
        "running_stub"
    } else if attempt.status == "succeeded_stub" {
        if attempt.readback_summary.status == "readback_unavailable"
            || attempt.readback_summary.status == "not_attempted_stub"
        {
            "readback_unavailable"
        } else {
            "succeeded_stub"
        }
    } else if attempt.status == "failed_stub" {
        "failed_stub"
    } else if attempt.status == "timed_out" {
        "timed_out"
    } else if attempt.readback_summary.status.contains("failed") {
        "readback_failed"
    } else {
        attempt.status.as_str()
    }
}

fn attempt_severity(status: &str, attempt: &SessionContinuationAttempt) -> &'static str {
    if attempt.prompt_sent || attempt.real_codex_executed || attempt.writes_codex_home {
        "blocking"
    } else if matches!(status, "failed_stub" | "timed_out" | "readback_failed") {
        "needs_user"
    } else if status == "readback_unavailable" {
        "warning"
    } else {
        "info"
    }
}

fn attempt_messages(
    status: &str,
    attempt: &SessionContinuationAttempt,
) -> (&'static str, &'static str, &'static str) {
    match status {
        "running_stub" => (
            "stub attempt 运行中",
            "这里只表示 Level A stub 路径在运行；没有真实 prompt 发送。",
            "等待 stub 记录完成；不要启动真实 resume。",
        ),
        "failed_stub" => (
            "stub attempt 失败",
            "Level A stub 路径失败；这不是真实 Codex 执行失败。",
            "查看 attempt failure_reason；如需真实执行仍要另行授权。",
        ),
        "timed_out" => (
            "stub attempt 超时",
            "超时只说明工作台自有 attempt 没有按时完成；不代表 agent 已被停止。",
            "查看 attempt 边界；不要自动重试。",
        ),
        "readback_failed" => (
            "readback failed",
            "readback 尝试失败或结果不可信；不能显示成 0 条读回。",
            "查看失败原因；不要写正式事实或记忆。",
        ),
        "readback_unavailable" => (
            "readback unavailable",
            if attempt.real_codex_executed {
                "本次没有可用 readback 来源；不能显示成 0 条结果。"
            } else {
                "Level A stub 没有真实 readback 来源；不能显示成 0 条结果。"
            },
            "保留边界状态；真实 readback 需要后续授权或 G 阶段验收。",
        ),
        _ => (
            "continuation attempt 状态",
            "工作台只展示可证明的 attempt 状态。",
            "查看详情，不自动执行后续动作。",
        ),
    }
}

fn latest_attempt_by_continuation(
    attempts: &[SessionContinuationAttempt],
) -> BTreeMap<String, SessionContinuationAttempt> {
    let mut map = BTreeMap::new();
    for attempt in attempts {
        map.insert(attempt.continuation_id.clone(), attempt.clone());
    }
    map
}

fn summarize_attention(attention: &[RuntimeSessionAttention]) -> Vec<SessionRunStatusSummary> {
    let mut groups: BTreeMap<String, Vec<&RuntimeSessionAttention>> = BTreeMap::new();
    for item in attention {
        let session_id = item
            .session_id
            .clone()
            .unwrap_or_else(|| format!("unbound:{}", item.adapter_id));
        groups.entry(session_id).or_default().push(item);
    }

    let mut summaries = Vec::new();
    for (session_id, mut items) in groups {
        items.sort_by(|a, b| {
            severity_rank(&b.severity)
                .cmp(&severity_rank(&a.severity))
                .then_with(|| b.updated_at.cmp(&a.updated_at))
        });
        let lead = items[0];
        let blocking_count = items
            .iter()
            .filter(|item| item.blocks_continuation || item.severity == "blocking")
            .count();
        let needs_user_count = items
            .iter()
            .filter(|item| item.requires_user_action || item.severity == "needs_user")
            .count();
        let mut source_refs = Vec::new();
        for item in &items {
            for source in &item.source_refs {
                if source_refs
                    .iter()
                    .any(|existing: &RuntimeAttentionSourceRef| {
                        existing.source_kind == source.source_kind
                            && existing.source_id == source.source_id
                    })
                {
                    continue;
                }
                source_refs.push(source.clone());
            }
        }
        summaries.push(SessionRunStatusSummary {
            session_id,
            adapter_id: lead.adapter_id.clone(),
            project_id: lead.project_id.clone(),
            workflow_id: lead.workflow_id.clone(),
            node_id: lead.node_id.clone(),
            current_status: lead.status.clone(),
            current_status_label: status_label(&lead.status).to_string(),
            attention_count: items.len(),
            blocking_count,
            needs_user_count,
            readback_status: lead.readback_boundary.status.clone(),
            latest_attention_ids: items
                .iter()
                .take(4)
                .map(|item| item.attention_id.clone())
                .collect(),
            source_refs: source_refs.into_iter().take(6).collect(),
            warnings: items
                .iter()
                .flat_map(|item| item.warnings.iter().cloned())
                .take(8)
                .collect(),
        });
    }

    summaries.sort_by(|a, b| {
        b.blocking_count
            .cmp(&a.blocking_count)
            .then_with(|| b.needs_user_count.cmp(&a.needs_user_count))
            .then_with(|| b.attention_count.cmp(&a.attention_count))
            .then_with(|| a.session_id.cmp(&b.session_id))
    });
    summaries
}

fn severity_rank(severity: &str) -> i32 {
    match severity {
        "blocking" => 4,
        "needs_user" => 3,
        "warning" => 2,
        "info" => 1,
        _ => 0,
    }
}

fn status_label(status: &str) -> &str {
    match status {
        "waiting_permission" => "等待用户确认",
        "waiting_level_b_authorization" => "等待 Level B 授权",
        "running_stub" => "stub 运行中",
        "succeeded_stub" => "stub 完成",
        "failed_stub" => "stub 失败",
        "timed_out" => "超时",
        "readback_failed" => "readback failed",
        "readback_timed_out" => "readback timed out",
        "readback_unavailable" => "readback unavailable",
        "duplicate_blocked" => "duplicate guard 阻断",
        "user_rejected" => "用户拒绝",
        "stale_cancelled" => "stale attempt 已取消",
        "blocked_by_guard" => "guard 阻断",
        "needs_user" => "需要用户介入",
        _ => status,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ContinuationAuditImpact, ContinuationFailureBoundary, ProviderAvailabilitySummary,
        ReadbackExpectation, SessionContinuationGuardResult, SessionContinuationRequest,
        SessionContinuationStoreScope,
    };

    #[test]
    fn runtime_session_attention_distinguishes_stub_unavailable_from_zero_results() {
        let preview = preview("needs_user_confirmation");
        let mut store = empty_store();
        store.continuations.push(continuation());
        store.attempts.push(SessionContinuationAttempt {
            attempt_version: 1,
            attempt_id: "attempt-1".to_string(),
            continuation_id: "continuation-1".to_string(),
            runner_kind: "stub".to_string(),
            execution_level: "level_a_stub_only".to_string(),
            status: "succeeded_stub".to_string(),
            started_at: "2026-06-06T00:00:00Z".to_string(),
            finished_at: Some("2026-06-06T00:00:01Z".to_string()),
            timeout_ms: Some(1000),
            command_preview: "preview only".to_string(),
            prompt_sent: false,
            real_codex_executed: false,
            writes_codex_home: false,
            writes_workbench_state: true,
            readback_summary: crate::SessionContinuationReadbackSummary {
                status: "readback_unavailable".to_string(),
                source_kind: "none_level_a_stub".to_string(),
                result_count: None,
                unavailable_reason: Some("not_attempted_stub".to_string()),
                warnings: vec!["readback unavailable is not zero results".to_string()],
            },
            failure_reason: None,
            audit_refs: vec!["audit-1".to_string()],
            warnings: vec![],
        });

        let (attention, summaries) =
            derive_runtime_session_attention(&[preview], &store, "2026-06-06T00:00:02Z");
        let readback_items: Vec<_> = attention
            .iter()
            .filter(|item| item.readback_boundary.status == "readback_unavailable")
            .collect();
        assert!(!readback_items.is_empty());
        assert!(readback_items
            .iter()
            .all(|item| item.readback_boundary.result_count.is_none()));
        assert!(readback_items
            .iter()
            .any(|item| item.readback_boundary.reason == "not_attempted_stub"));
        assert!(summaries
            .iter()
            .any(|summary| summary.readback_status == "readback_unavailable"));
    }

    #[test]
    fn runtime_session_attention_marks_guard_blocking() {
        let preview = preview("blocked");
        let store = empty_store();
        let (attention, _) =
            derive_runtime_session_attention(&[preview], &store, "2026-06-06T00:00:00Z");
        let blocked = attention
            .iter()
            .find(|item| item.status == "blocked_by_guard")
            .expect("blocked guard attention");
        assert!(blocked.blocks_continuation);
        assert_eq!(blocked.severity, "blocking");
        assert_eq!(blocked.readback_boundary.reason, "guard_blocked");
        assert_eq!(blocked.readback_boundary.result_count, None);
    }

    fn empty_store() -> SessionContinuationStoreV1 {
        SessionContinuationStoreV1 {
            schema_version: "session_continuation_store.v1".to_string(),
            store_version: 1,
            storage_kind: "sidecar_json_v0".to_string(),
            scope: SessionContinuationStoreScope {
                scope_kind: "workflow_state_sidecar".to_string(),
                workflow_state_path: None,
                sidecar_path: None,
                project_roots: vec![],
            },
            revision: 0,
            last_write_id: None,
            generated_by: "test".to_string(),
            created_at: "2026-06-06T00:00:00Z".to_string(),
            updated_at: "2026-06-06T00:00:00Z".to_string(),
            continuations: vec![],
            attempts: vec![],
            audit_events: vec![],
            warnings: vec![],
        }
    }

    fn continuation() -> ControlledSessionContinuation {
        ControlledSessionContinuation {
            record_version: 1,
            continuation_id: "continuation-1".to_string(),
            preview_id: "preview-1".to_string(),
            adapter_id: "codex-local".to_string(),
            operation_id: "resume".to_string(),
            project_id: "project-1".to_string(),
            project_root: "/tmp/project".to_string(),
            workflow_id: "workflow-1".to_string(),
            node_id: "node-1".to_string(),
            session_id: "session-1".to_string(),
            work_item_id: Some("work-item-1".to_string()),
            target_cwd: "/tmp/project".to_string(),
            allowed_write_roots: vec!["/tmp/project".to_string()],
            sandbox: "workspace-write".to_string(),
            prompt_source_kind: "workflow_followup".to_string(),
            prompt_summary: "stub prompt summary".to_string(),
            command_preview: "preview only".to_string(),
            readback_strategy: "required".to_string(),
            status: "succeeded_stub".to_string(),
            execution_level: "level_a_stub_only".to_string(),
            runner_kind: "stub".to_string(),
            user_confirmation_state: "confirmed".to_string(),
            guard_status: "needs_user_confirmation".to_string(),
            requested_by: "user".to_string(),
            confirmed_by: "user".to_string(),
            confirmation_reason: "test".to_string(),
            created_at: "2026-06-06T00:00:00Z".to_string(),
            updated_at: "2026-06-06T00:00:01Z".to_string(),
            audit_refs: vec!["audit-1".to_string()],
            warnings: vec![],
        }
    }

    fn preview(status: &str) -> SessionContinuationPreview {
        let provider = ProviderAvailabilitySummary {
            adapter_id: "codex-local".to_string(),
            provider_id: "codex-cli".to_string(),
            provider_label: "Codex CLI".to_string(),
            provider_kind: "local_cli".to_string(),
            adapter_status: "available".to_string(),
            availability_status: "available_readonly".to_string(),
            credential_status: "not_required_by_workbench".to_string(),
            model_status: "local_cli_managed".to_string(),
            external_call_status: "not_needed_for_readonly".to_string(),
            cost_risk_status: "not_estimated_readonly".to_string(),
            user_visible_reason: "readonly".to_string(),
            safe_to_display: true,
            requires_user_configuration: false,
            requires_future_task: false,
            warnings: vec![],
        };
        let request = SessionContinuationRequest {
            adapter_id: "codex-local".to_string(),
            operation_id: "resume".to_string(),
            project_id: Some("project-1".to_string()),
            project_root: Some("/tmp/project".to_string()),
            workflow_id: Some("workflow-1".to_string()),
            node_id: Some("node-1".to_string()),
            session_id: Some("session-1".to_string()),
            work_item_id: Some("work-item-1".to_string()),
            target_cwd: Some("/tmp/project".to_string()),
            allowed_write_roots: vec!["/tmp/project".to_string()],
            sandbox: "workspace-write".to_string(),
            prompt_source_kind: "workflow_followup".to_string(),
            prompt_summary: "preview prompt".to_string(),
            readback_strategy: "required".to_string(),
            requested_by: "user".to_string(),
            user_confirmation_state: "missing".to_string(),
        };
        SessionContinuationPreview {
            preview_id: format!("preview-{status}"),
            adapter_id: "codex-local".to_string(),
            operation_id: "resume".to_string(),
            target_session_id: Some("session-1".to_string()),
            target_session_title: Some("Session 1".to_string()),
            project_id: Some("project-1".to_string()),
            project_root: Some("/tmp/project".to_string()),
            workflow_id: Some("workflow-1".to_string()),
            node_id: Some("node-1".to_string()),
            binding_id: Some("binding-1".to_string()),
            work_item_id: Some("work-item-1".to_string()),
            target_cwd: Some("/tmp/project".to_string()),
            allowed_write_roots_summary: vec!["/tmp/project".to_string()],
            sandbox_summary: "workspace-write".to_string(),
            prompt_source_kind: "workflow_followup".to_string(),
            prompt_summary: "preview prompt".to_string(),
            readback_expectation: ReadbackExpectation {
                strategy: "required".to_string(),
                required: true,
                expected_sources: vec!["codex-last-message".to_string()],
                unavailable_behavior: "show_unavailable".to_string(),
                warnings: vec![],
            },
            failure_handling: ContinuationFailureBoundary {
                timeout_policy: "record_timeout".to_string(),
                retry_policy: "no_auto_retry".to_string(),
                failure_record: "workbench_owned".to_string(),
                user_visible_behavior: "show_boundary".to_string(),
                warnings: vec![],
            },
            audit_impact: ContinuationAuditImpact {
                impact_kind: "preview_only_no_execution".to_string(),
                writes_attempt_in_e4: false,
                writes_dispatch_in_e4: false,
                writes_readback_in_e4: false,
                future_audit_requirement: "required".to_string(),
                warnings: vec![],
            },
            provider_availability_summary: Some(provider),
            guard_result: SessionContinuationGuardResult {
                status: status.to_string(),
                severity: if status == "blocked" {
                    "high".to_string()
                } else {
                    "medium".to_string()
                },
                blocks_execution: status == "blocked",
                allows_preview: true,
                requires_user_confirmation: status == "needs_user_confirmation",
                reasons: vec![format!("guard {status}")],
                required_fixes: if status == "blocked" {
                    vec!["fix guard".to_string()]
                } else {
                    vec![]
                },
                warnings: vec![],
            },
            request,
            user_visible_warnings: vec![],
        }
    }
}
