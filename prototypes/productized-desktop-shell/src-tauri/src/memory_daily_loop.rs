// S3 每日记忆采集（L5 daily loop）：把真治理活动结构化映射成「待确认记忆候选」攒进 candidate_store。
// 死线：**只写候选不写正式**（candidate_allowed，正式仍走 M2 用户采纳）；**不漏敏感**（summary/claim/body
// 不含 prompt body/.codex/凭据/全文 transcript；bus 自身亦校验）；**best-effort**（capture 失败只 warning，
// 绝不拖垮治理命令主返回）；不碰 codex/执行/闸。本文件 = S0 删掉的 memory_daily_loop.rs 恢复 + 扩 3 映射器。

use crate::{
    operation_control::OperationControlDecisionRequest, CaptureMemoryEventInput,
    CaptureMemoryEventOutput, GlobalFinalResultReviewInput, MemoryCaptureCandidateDraft,
    MemoryCaptureSourceRef, MemoryScope, RecordPlanAuthorizationUserConfirmationInput,
    WorkerStructuredReportInput,
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

// ===== S3 扩充：3 个治理事件→候选映射器（照 operation_control 模板；脱敏=只用结构化字段、不回显自由文本）=====
// 脱敏原则：summary/claim/body 用「角色+id+决定」等结构化字段拼，**不直接回显** worker/review 的自由文本
// summary（那可能夹带敏感内容），bus 亦会拦 prompt body/.codex/transcript 等字串。

// 1) worker 结构化汇报 → 候选
pub(crate) fn worker_report_capture_input(
    context: &MemoryDailyLoopContext<'_>,
    report: &WorkerStructuredReportInput,
) -> Result<CaptureMemoryEventInput, String> {
    let source_id = format!(
        "worker-report:{}:{}",
        report.work_item_id, context.created_at
    );
    Ok(CaptureMemoryEventInput {
        project_root: context.project_root.to_string(),
        project_id: Some(context.project_id.to_string()),
        workflow_id: Some(context.workflow_id.to_string()),
        workflow_node_id: Some(report.workflow_node_id.clone()),
        run_unit_id: context.run_unit_id.map(str::to_string),
        product_command_id: None,
        product_attempt_id: None,
        runtime_log_ref: None,
        audit_refs: vec![format!("audit:worker-report:{}", report.work_item_id)],
        readback_ref: None,
        task_package_ref: None,
        memory_packet_ref: None,
        scope: MemoryScope {
            scope_id: format!("scope:l5-worker-report:{}", report.work_item_id),
            scope_type: "workflow".to_string(),
            user_id: None,
            project_id: Some(context.project_id.to_string()),
            workflow_id: Some(context.workflow_id.to_string()),
            session_id: None,
            role_ids: vec!["worker".to_string(), "project_director".to_string()],
            document_refs: Vec::new(),
            permission_policy_ref: None,
            model_export_policy: "local_only".to_string(),
            valid_from: context.created_at.to_string(),
            valid_until: None,
        },
        source_type: "worker_report".to_string(),
        source_refs: vec![MemoryCaptureSourceRef {
            source_ref_id: format!("source:l5-worker-report:{}", report.work_item_id),
            source_type: "worker_report".to_string(),
            source_id,
            project_id: Some(context.project_id.to_string()),
            workflow_id: Some(context.workflow_id.to_string()),
            workflow_node_id: Some(report.workflow_node_id.clone()),
            run_unit_id: context.run_unit_id.map(str::to_string),
            product_command_id: None,
            product_attempt_id: None,
            runtime_log_ref: None,
            audit_ref_id: report.dispatch_id.clone(),
            readback_ref: None,
            task_package_ref: None,
            memory_packet_ref: None,
            evidence_ref: None,
            summary: format!(
                "worker 在 {} 提交结构化汇报，acceptance={}。",
                report.work_item_id, report.acceptance_status
            ),
            sensitive_level: "internal".to_string(),
            created_at: context.created_at.to_string(),
        }],
        summary: format!(
            "worker 结构化汇报已记录（work_item={}，acceptance={}）；待项目主管确认，非正式事实。",
            report.work_item_id, report.acceptance_status
        ),
        evidence_summary: format!(
            "来源 worker_report；证据引用 {} 条；open_issues {} 条。",
            report.evidence_refs.len(),
            report.open_issues.len()
        ),
        sensitivity: "internal".to_string(),
        candidate_policy: "candidate_allowed".to_string(),
        generated_by_role: "worker".to_string(),
        actor_id: context.actor_id.to_string(),
        risk_level: "low".to_string(),
        reason: "L5 daily loop 把 worker 结构化汇报采成待确认候选；不写正式记忆。".to_string(),
        candidate: Some(MemoryCaptureCandidateDraft {
            memory_type: "workflow_summary".to_string(),
            claim: format!(
                "worker 在 {} 完成一轮工作并提交结构化汇报（acceptance={}）。",
                report.work_item_id, report.acceptance_status
            ),
            body: format!(
                "角色 {} 提交 worker 汇报：work_item={}，acceptance={}，证据 {} 条；为待确认候选、需项目主管确认后方成观察/记忆。",
                report.actor_role,
                report.work_item_id,
                report.acceptance_status,
                report.evidence_refs.len()
            ),
            review_reason: "从 L5 daily worker-report capture 生成待确认候选；候选不是正式记忆。"
                .to_string(),
            requires_user_confirmation: true,
            actor_role: "project_director".to_string(),
        }),
        expected_capture_store_revision: None,
        expected_observation_store_revision: None,
        expected_candidate_store_revision: None,
    })
}

// 2) 方案授权·用户确认（方案采纳）→ 候选（source_type=user_action：用户确认是用户动作）
pub(crate) fn plan_authorization_capture_input(
    context: &MemoryDailyLoopContext<'_>,
    confirmation: &RecordPlanAuthorizationUserConfirmationInput,
) -> Result<CaptureMemoryEventInput, String> {
    let source_id = format!(
        "plan-auth-confirm:{}:{}",
        confirmation.authorization_id, context.created_at
    );
    Ok(CaptureMemoryEventInput {
        project_root: context.project_root.to_string(),
        project_id: Some(context.project_id.to_string()),
        workflow_id: Some(context.workflow_id.to_string()),
        workflow_node_id: context.workflow_node_id.map(str::to_string),
        run_unit_id: context.run_unit_id.map(str::to_string),
        product_command_id: None,
        product_attempt_id: None,
        runtime_log_ref: None,
        audit_refs: vec![format!(
            "audit:plan-auth-confirm:{}",
            confirmation.authorization_id
        )],
        readback_ref: None,
        task_package_ref: None,
        memory_packet_ref: None,
        scope: MemoryScope {
            scope_id: format!("scope:l5-plan-auth:{}", confirmation.authorization_id),
            scope_type: "workflow".to_string(),
            user_id: None,
            project_id: Some(context.project_id.to_string()),
            workflow_id: Some(context.workflow_id.to_string()),
            session_id: None,
            role_ids: vec!["user".to_string()],
            document_refs: Vec::new(),
            permission_policy_ref: None,
            model_export_policy: "local_only".to_string(),
            valid_from: context.created_at.to_string(),
            valid_until: None,
        },
        source_type: "user_action".to_string(),
        source_refs: vec![MemoryCaptureSourceRef {
            source_ref_id: format!("source:l5-plan-auth:{}", confirmation.authorization_id),
            source_type: "user_action".to_string(),
            source_id,
            project_id: Some(context.project_id.to_string()),
            workflow_id: Some(context.workflow_id.to_string()),
            workflow_node_id: context.workflow_node_id.map(str::to_string),
            run_unit_id: context.run_unit_id.map(str::to_string),
            product_command_id: None,
            product_attempt_id: None,
            runtime_log_ref: None,
            audit_ref_id: None,
            readback_ref: None,
            task_package_ref: None,
            memory_packet_ref: None,
            evidence_ref: None,
            summary: format!(
                "用户确认方案授权 {}（采纳范围）。",
                confirmation.authorization_id
            ),
            sensitive_level: "internal".to_string(),
            created_at: context.created_at.to_string(),
        }],
        summary: format!(
            "用户确认了方案授权 {}（方案采纳）；待边界复核激活，非正式记忆。",
            confirmation.authorization_id
        ),
        evidence_summary: "来源 user_action：方案授权用户确认。".to_string(),
        sensitivity: "internal".to_string(),
        candidate_policy: "candidate_allowed".to_string(),
        generated_by_role: "user".to_string(),
        actor_id: context.actor_id.to_string(),
        risk_level: "low".to_string(),
        reason: "L5 daily loop 把方案授权用户确认采成待确认候选；不写正式记忆。".to_string(),
        candidate: Some(MemoryCaptureCandidateDraft {
            memory_type: "workflow_summary".to_string(),
            claim: format!(
                "用户采纳了方案授权 {}，授权进入待全局边界复核。",
                confirmation.authorization_id
            ),
            body: format!(
                "用户确认方案授权 {}；这表示用户认可该方案范围，下一步走全局边界复核。为待确认候选、非正式记忆。",
                confirmation.authorization_id
            ),
            review_reason: "从 L5 daily plan-authorization capture 生成待确认候选；候选不是正式记忆。"
                .to_string(),
            requires_user_confirmation: true,
            actor_role: "project_director".to_string(),
        }),
        expected_capture_store_revision: None,
        expected_observation_store_revision: None,
        expected_candidate_store_revision: None,
    })
}

// 3) 全局最终复核 → 候选（source_type=final_review）
pub(crate) fn final_review_capture_input(
    context: &MemoryDailyLoopContext<'_>,
    review: &GlobalFinalResultReviewInput,
) -> Result<CaptureMemoryEventInput, String> {
    let source_id = format!(
        "final-review:{}:{}",
        review.authorization_id, context.created_at
    );
    Ok(CaptureMemoryEventInput {
        project_root: context.project_root.to_string(),
        project_id: Some(context.project_id.to_string()),
        workflow_id: Some(context.workflow_id.to_string()),
        workflow_node_id: context.workflow_node_id.map(str::to_string),
        run_unit_id: context.run_unit_id.map(str::to_string),
        product_command_id: None,
        product_attempt_id: None,
        runtime_log_ref: None,
        audit_refs: vec![format!("audit:final-review:{}", review.authorization_id)],
        readback_ref: None,
        task_package_ref: None,
        memory_packet_ref: None,
        scope: MemoryScope {
            scope_id: format!("scope:l5-final-review:{}", review.authorization_id),
            scope_type: "workflow".to_string(),
            user_id: None,
            project_id: Some(context.project_id.to_string()),
            workflow_id: Some(context.workflow_id.to_string()),
            session_id: None,
            role_ids: vec!["global_director".to_string()],
            document_refs: Vec::new(),
            permission_policy_ref: None,
            model_export_policy: "local_only".to_string(),
            valid_from: context.created_at.to_string(),
            valid_until: None,
        },
        source_type: "final_review".to_string(),
        source_refs: vec![MemoryCaptureSourceRef {
            source_ref_id: format!("source:l5-final-review:{}", review.authorization_id),
            source_type: "final_review".to_string(),
            source_id,
            project_id: Some(context.project_id.to_string()),
            workflow_id: Some(context.workflow_id.to_string()),
            workflow_node_id: context.workflow_node_id.map(str::to_string),
            run_unit_id: context.run_unit_id.map(str::to_string),
            product_command_id: None,
            product_attempt_id: None,
            runtime_log_ref: None,
            audit_ref_id: None,
            readback_ref: None,
            task_package_ref: None,
            memory_packet_ref: None,
            evidence_ref: None,
            summary: format!(
                "全局主管最终复核决定={}（authorization={}）。",
                review.decision, review.authorization_id
            ),
            sensitive_level: "internal".to_string(),
            created_at: context.created_at.to_string(),
        }],
        summary: format!(
            "全局最终复核已记录（decision={}，authorization={}）；待用户最终验收，非正式记忆。",
            review.decision, review.authorization_id
        ),
        evidence_summary: format!(
            "来源 final_review；已确认过程事实 {} 条、证据 {} 条。",
            review.accepted_process_fact_ids.len(),
            review.evidence_refs.len()
        ),
        sensitivity: "internal".to_string(),
        candidate_policy: "candidate_allowed".to_string(),
        generated_by_role: "global_director".to_string(),
        actor_id: context.actor_id.to_string(),
        risk_level: "low".to_string(),
        reason: "L5 daily loop 把全局最终复核采成待确认候选；不写正式记忆。".to_string(),
        candidate: Some(MemoryCaptureCandidateDraft {
            memory_type: "workflow_summary".to_string(),
            claim: format!(
                "全局主管对 authorization {} 的最终复核决定为 {}。",
                review.authorization_id, review.decision
            ),
            body: format!(
                "全局最终复核：decision={}，authorization={}，已确认过程事实 {} 条；待用户最终验收。为待确认候选、非正式记忆。",
                review.decision,
                review.authorization_id,
                review.accepted_process_fact_ids.len()
            ),
            review_reason: "从 L5 daily final-review capture 生成待确认候选；候选不是正式记忆。"
                .to_string(),
            requires_user_confirmation: true,
            actor_role: "project_director".to_string(),
        }),
        expected_capture_store_revision: None,
        expected_observation_store_revision: None,
        expected_candidate_store_revision: None,
    })
}

// best-effort 挂钩：治理命令记录后调它采集候选；**capture 失败只回 warning，绝不让治理命令失败/改主返回**。
pub(crate) fn capture_governance_event_best_effort(
    workflow_state_path: &Path,
    input: &CaptureMemoryEventInput,
    timestamp: &str,
    write_ids: &MemoryDailyLoopWriteIds<'_>,
) -> Vec<String> {
    match capture_daily_memory_event(workflow_state_path, input, timestamp, write_ids) {
        Ok(output) => output.warnings,
        Err(error) => vec![format!("memory_daily_capture_best_effort_skipped:{error}")],
    }
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

    fn fixture_worker_report() -> WorkerStructuredReportInput {
        WorkerStructuredReportInput {
            project_root: "/tmp/l5-memory-daily-loop-project".to_string(),
            project_id: "project:tmp-l5-memory-daily-loop-project".to_string(),
            workflow_id: "workflow:tmp-l5-memory-daily-loop-project:default".to_string(),
            workflow_node_id: "workflow:tmp-l5-memory-daily-loop-project:default:node:codex-dev"
                .to_string(),
            work_item_id: "work-item:l5:001".to_string(),
            dispatch_id: Some("dispatch:l5:001".to_string()),
            // SYN-FND-004B 测试夹具: dispatch_id 即 attempt_id
            attempt_id: Some("dispatch:l5:001".to_string()),
            authenticated_actor: "project:tmp-l5-memory-daily-loop-project".to_string(),
            report_hash: "hash:fixture".to_string(),
            actor_role: "codex-dev".to_string(),
            executed_what: "执行了一轮工作".to_string(),
            changed_what: "改了若干文件".to_string(),
            summary: "worker 汇报摘要".to_string(),
            evidence_refs: vec!["evidence:l5:001".to_string()],
            open_issues: vec![],
            permission_requests: vec![],
            direction_risks: vec![],
            follow_up_suggestions: vec![],
            acceptance_status: "reported_completed".to_string(),
            source_refs: vec![],
            expected_workflow_revision: None,
        }
    }

    fn fixture_plan_auth_confirmation() -> RecordPlanAuthorizationUserConfirmationInput {
        RecordPlanAuthorizationUserConfirmationInput {
            project_root: "/tmp/l5-memory-daily-loop-project".to_string(),
            authorization_id: "plan-auth:l5:001".to_string(),
            actor_id: "user:l5".to_string(),
            confirmation_summary: "用户采纳方案".to_string(),
            expected_store_revision: None,
        }
    }

    fn fixture_final_review() -> GlobalFinalResultReviewInput {
        GlobalFinalResultReviewInput {
            project_root: "/tmp/l5-memory-daily-loop-project".to_string(),
            project_id: "project:tmp-l5-memory-daily-loop-project".to_string(),
            workflow_id: "workflow:tmp-l5-memory-daily-loop-project:default".to_string(),
            authorization_id: "plan-auth:l5:001".to_string(),
            proposal_id: "proposal:l5:001".to_string(),
            actor_id: "global:l5".to_string(),
            actor_role: "global_director".to_string(),
            decision: "accepted".to_string(),
            summary: "最终复核通过".to_string(),
            evidence_refs: vec!["evidence:l5:final".to_string()],
            accepted_process_fact_ids: vec!["process-fact:l5:001".to_string()],
            open_issues: vec![],
            deferred_items: vec![],
            expected_workflow_revision: None,
        }
    }

    fn assert_candidate_allowed_and_desensitized(input: &CaptureMemoryEventInput) {
        assert_eq!(input.candidate_policy, "candidate_allowed");
        assert!(
            input
                .candidate
                .as_ref()
                .expect("candidate draft")
                .requires_user_confirmation
        );
        for text in [&input.summary, &input.evidence_summary] {
            let low = text.to_ascii_lowercase();
            assert!(!low.contains("prompt body"), "脱敏:不含 prompt body");
            assert!(!low.contains("full transcript"), "脱敏:不含全文 transcript");
            assert!(
                !text.contains("/Users/yoyi/.codex"),
                "脱敏:不含 .codex 路径"
            );
        }
    }

    #[test]
    fn l5_worker_report_capture_input_is_candidate_allowed_desensitized() {
        let input = worker_report_capture_input(&fixture_context(), &fixture_worker_report())
            .expect("worker report capture input");
        assert_eq!(input.source_type, "worker_report");
        assert_eq!(input.generated_by_role, "worker");
        assert_eq!(input.source_refs[0].source_type, "worker_report");
        assert_candidate_allowed_and_desensitized(&input);
    }

    #[test]
    fn l5_plan_authorization_capture_input_is_candidate_allowed_desensitized() {
        let input =
            plan_authorization_capture_input(&fixture_context(), &fixture_plan_auth_confirmation())
                .expect("plan auth capture input");
        assert_eq!(input.source_type, "user_action");
        assert_eq!(input.generated_by_role, "user");
        assert_candidate_allowed_and_desensitized(&input);
    }

    #[test]
    fn l5_final_review_capture_input_is_candidate_allowed_desensitized() {
        let input = final_review_capture_input(&fixture_context(), &fixture_final_review())
            .expect("final review capture input");
        assert_eq!(input.source_type, "final_review");
        assert_eq!(input.generated_by_role, "global_director");
        assert_candidate_allowed_and_desensitized(&input);
    }

    #[test]
    fn l5_worker_report_capture_lands_candidate_without_formal_memory() {
        let path = temp_workflow_path("l5-worker-report-capture");
        let input = worker_report_capture_input(&fixture_context(), &fixture_worker_report())
            .expect("worker report capture input");
        let output = capture_daily_memory_event(
            &path,
            &input,
            "2026-06-16T12:00:00Z",
            &MemoryDailyLoopWriteIds {
                capture_write_id: "write-wr-capture",
                observation_write_id: "write-wr-observation",
                candidate_write_id: "write-wr-candidate",
            },
        )
        .expect("worker report daily capture");
        assert!(output.candidate.is_some());
        let store = memory_candidate_store::load_store(&path, "2026-06-16T12:00:00Z")
            .expect("candidate store should load");
        assert_eq!(store.candidates.len(), 1, "候选应真攒进 store");
        assert!(
            !formal_memory_store::sidecar_path(&path)
                .expect("formal path")
                .exists(),
            "采集不得建正式记忆 sidecar"
        );
        let _ = fs::remove_dir_all(path.parent().expect("workflow parent"));
    }

    #[test]
    fn l5_best_effort_swallows_capture_failure() {
        // 注入会被 bus 拒的输入（sensitivity=secret 但 candidate_policy 非 blocked_sensitive）→ capture_event Err；
        // best-effort 应只回 warning、不 panic、不 Err（治理命令主返回不受影响）。
        let path = temp_workflow_path("l5-best-effort");
        let mut input = worker_report_capture_input(&fixture_context(), &fixture_worker_report())
            .expect("worker report capture input");
        input.sensitivity = "secret".to_string(); // 与 candidate_allowed 冲突 → bus 拒
        let warnings = capture_governance_event_best_effort(
            &path,
            &input,
            "2026-06-16T12:00:00Z",
            &MemoryDailyLoopWriteIds {
                capture_write_id: "write-be-capture",
                observation_write_id: "write-be-observation",
                candidate_write_id: "write-be-candidate",
            },
        );
        assert!(
            warnings
                .iter()
                .any(|w| w.starts_with("memory_daily_capture_best_effort_skipped")),
            "best-effort 应把失败收成 warning，实际：{warnings:?}"
        );
        let _ = fs::remove_dir_all(path.parent().expect("workflow parent"));
    }
}
