use crate::utils::store_paths;
use crate::{
    AuthorizedExecutionScope, CreatePlanAuthorizationInput, CreateProjectConsultationProposalInput,
    CreateProjectConsultationProposalOutput, PlanAuthorization, PlanAuthorizationAuditEvent,
    PlanAuthorizationStatus, PlanAuthorizationStopCondition, ProjectConsultationProposal,
    ProjectConsultationProposalAuditEvent, ProjectConsultationProposalDecision,
    ProjectConsultationProposalDecisionKind, ProjectConsultationProposalMarkdown,
    ProjectConsultationProposalReadModel, ProjectConsultationProposalScopeDraft,
    ProjectConsultationProposalStatus, ProjectConsultationProposalStoreV1,
    RecordPlanAuthorizationUserConfirmationInput, RecordProjectConsultationProposalDecisionInput,
    RecordProjectConsultationProposalDecisionOutput,
    RenderProjectConsultationProposalMarkdownInput,
};
use serde_json::Value;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

const STORE_SCHEMA_VERSION: &str = "project_consultation_proposal_store.v1";
const PROPOSAL_SCHEMA_VERSION: &str = "project_consultation_proposal.v1";
const SIDECAR_NAME: &str = "project-proposals.v1.json";
const LOCK_NAME: &str = ".project-proposals.v1.lock";
const LOCK_RETRY_COUNT: usize = 5;
const LOCK_RETRY_DELAY: Duration = Duration::from_millis(100);

pub(crate) fn sidecar_path(workflow_state_path: &Path) -> Result<PathBuf, String> {
    store_paths::sidecar_path(workflow_state_path, SIDECAR_NAME, "项目咨询方案")
}

pub(crate) fn load_store(
    workflow_state_path: &Path,
    timestamp_ms: i64,
) -> Result<ProjectConsultationProposalStoreV1, String> {
    let sidecar = sidecar_path(workflow_state_path)?;
    if !sidecar.exists() {
        return Ok(empty_store(timestamp_ms));
    }
    let text = fs::read_to_string(&sidecar).map_err(|error| {
        format!(
            "读取项目咨询方案 sidecar 失败 {}：{error}",
            sidecar.display()
        )
    })?;
    let store: ProjectConsultationProposalStoreV1 =
        serde_json::from_str(&text).map_err(|error| {
            format!(
                "项目咨询方案 sidecar JSON 损坏，已拒绝覆盖 {}：{error}",
                sidecar.display()
            )
        })?;
    validate_store(&store)?;
    Ok(store)
}

pub(crate) fn create_proposal(
    workflow_state_path: &Path,
    input: &CreateProjectConsultationProposalInput,
    timestamp_ms: i64,
    write_id: &str,
) -> Result<CreateProjectConsultationProposalOutput, String> {
    validate_create_input(input)?;
    let project_id_value = input
        .project_id
        .clone()
        .unwrap_or_else(|| project_id(&input.project_root));
    let workflow_id_value = input
        .workflow_id
        .clone()
        .unwrap_or_else(|| default_workflow_id(&input.project_root));
    ensure_workflow_identity(workflow_state_path, &project_id_value, &workflow_id_value)?;

    let sidecar = sidecar_path(workflow_state_path)?;
    ensure_sidecar_parent(&sidecar)?;
    let lock = StoreLock::acquire(&lock_path_for(&sidecar)?, write_id)?;
    let mut store = load_store(workflow_state_path, timestamp_ms)?;
    validate_expected_revision(input.expected_store_revision, store.revision)?;

    let proposal_id = format!(
        "proposal:{}:{}",
        stable_id(&format!(
            "{}:{}:{}:{}",
            project_id_value, workflow_id_value, input.title, input.goal_summary
        )),
        timestamp_ms
    );
    let audit_event_id = format!(
        "audit:project-consultation-proposal-created:{}:{}",
        stable_id(&proposal_id),
        timestamp_ms
    );
    let mut proposal = ProjectConsultationProposal {
        proposal_id: proposal_id.clone(),
        schema_version: PROPOSAL_SCHEMA_VERSION.to_string(),
        project_id: project_id_value.clone(),
        workflow_id: workflow_id_value.clone(),
        title: input.title.trim().to_string(),
        user_goal: input.user_goal.trim().to_string(),
        user_requirement_snapshot: if input.user_requirement_snapshot.is_empty() {
            input.user_goal.clone()
        } else {
            input.user_requirement_snapshot.clone()
        },
        goal_summary: input.goal_summary.trim().to_string(),
        proposed_steps: trim_non_empty(&input.proposed_steps),
        scope_draft: trim_scope_draft(&input.scope_draft),
        risks: input.risks.clone(),
        worker_acceptance_criteria: trim_non_empty(&input.worker_acceptance_criteria),
        control_core_acceptance_criteria: trim_non_empty(&input.control_core_acceptance_criteria),
        supervisor_acceptance_criteria: trim_non_empty(&input.supervisor_acceptance_criteria),
        acceptance_criteria: trim_non_empty(&input.acceptance_criteria),
        status: ProjectConsultationProposalStatus::PendingUserConfirmation,
        plan_authorization_id: None,
        created_by_role: input.created_by_role,
        // 交办·刀2 2.5：透传咨询的「建议按工作流」轻标记（map 从咨询 LM 判定填入 input）。
        suggest_workflow: input.suggest_workflow,
        created_at_ms: timestamp_ms,
        updated_at_ms: timestamp_ms,
    };
    let audit_event = ProjectConsultationProposalAuditEvent {
        audit_event_id: audit_event_id.clone(),
        event_type: "project_consultation_proposal_created".to_string(),
        actor_id: input.actor_id.trim().to_string(),
        actor_role: creator_role_name(input.created_by_role).to_string(),
        project_id: project_id_value.clone(),
        workflow_id: workflow_id_value.clone(),
        proposal_id: Some(proposal_id),
        plan_authorization_id: None,
        before_status: None,
        after_status: Some(proposal.status),
        reason: "创建项目咨询方案草案；等待用户确认。".to_string(),
        created_at_ms: timestamp_ms,
    };

    store.revision += 1;
    store.updated_at_ms = timestamp_ms;
    store.proposals.push(proposal.clone());
    store.audit_events.push(audit_event.clone());
    write_store_atomic(&sidecar, &store, timestamp_ms, write_id)?;
    drop(lock);
    proposal.status = ProjectConsultationProposalStatus::PendingUserConfirmation;

    Ok(CreateProjectConsultationProposalOutput {
        proposal,
        audit_event,
        read_model: summarize_store_for_workflow(&store, &project_id_value, &workflow_id_value),
        store_revision: store.revision,
        warnings: store.warnings.clone(),
    })
}

pub(crate) fn render_markdown(
    workflow_state_path: &Path,
    input: &RenderProjectConsultationProposalMarkdownInput,
    timestamp_ms: i64,
) -> Result<ProjectConsultationProposalMarkdown, String> {
    let store = load_store(workflow_state_path, timestamp_ms)?;
    let proposal = store
        .proposals
        .iter()
        .find(|proposal| proposal.proposal_id == input.proposal_id)
        .ok_or_else(|| format!("找不到项目咨询方案草案：{}", input.proposal_id))?;
    ensure_workflow_identity(
        workflow_state_path,
        &proposal.project_id,
        &proposal.workflow_id,
    )?;
    let markdown = render_proposal_markdown(proposal);
    Ok(ProjectConsultationProposalMarkdown {
        proposal_id: proposal.proposal_id.clone(),
        markdown,
        warnings: store.warnings.clone(),
    })
}

pub(crate) fn record_decision(
    workflow_state_path: &Path,
    input: &RecordProjectConsultationProposalDecisionInput,
    timestamp_ms: i64,
    proposal_write_id: &str,
    authorization_write_id: &str,
    authorization_confirm_write_id: &str,
) -> Result<RecordProjectConsultationProposalDecisionOutput, String> {
    if input.actor_id.trim().is_empty() {
        return Err("项目咨询方案决定缺少 actor_id".to_string());
    }
    if input.summary.trim().is_empty() {
        return Err("项目咨询方案决定缺少 summary".to_string());
    }

    let sidecar = sidecar_path(workflow_state_path)?;
    ensure_sidecar_parent(&sidecar)?;
    let lock = StoreLock::acquire(&lock_path_for(&sidecar)?, proposal_write_id)?;
    let mut store = load_store(workflow_state_path, timestamp_ms)?;
    validate_expected_revision(input.expected_proposal_store_revision, store.revision)?;
    let index = find_proposal_index(&store, &input.proposal_id)?;
    let before = store.proposals[index].status;
    if !matches!(
        before,
        ProjectConsultationProposalStatus::Draft
            | ProjectConsultationProposalStatus::PendingUserConfirmation
    ) {
        drop(lock);
        return Err(format!(
            "当前项目咨询方案状态不能重复记录用户决定：{}",
            status_name(before)
        ));
    }

    let proposal_before_update = store.proposals[index].clone();
    ensure_workflow_identity(
        workflow_state_path,
        &proposal_before_update.project_id,
        &proposal_before_update.workflow_id,
    )?;

    let mut linked_authorization: Option<PlanAuthorization> = None;
    let mut linked_authorization_audit: Option<PlanAuthorizationAuditEvent> = None;
    let mut linked_authorization_revision: Option<i64> = None;
    let mut plan_authorization_id = proposal_before_update.plan_authorization_id.clone();

    if input.decision == ProjectConsultationProposalDecisionKind::Confirm {
        let existing_pending_authorization =
            if before == ProjectConsultationProposalStatus::PendingUserConfirmation {
                pending_authorization_for_proposal(
                    workflow_state_path,
                    &proposal_before_update,
                    timestamp_ms,
                )?
            } else {
                None
            };
        let (authorization_id, authorization_revision) = if let Some((authorization, revision)) =
            existing_pending_authorization
        {
            if let Some(expected_revision) = input.expected_plan_authorization_store_revision {
                if expected_revision != revision {
                    return Err(format!(
                        "plan_authorization_store_conflict: expected revision {expected_revision}, actual {revision}"
                    ));
                }
            }
            (authorization.authorization_id, revision)
        } else {
            let created = crate::plan_authorization_store::create_authorization(
                workflow_state_path,
                &CreatePlanAuthorizationInput {
                    project_root: input.project_root.clone(),
                    project_id: Some(proposal_before_update.project_id.clone()),
                    workflow_id: Some(proposal_before_update.workflow_id.clone()),
                    source_proposal_id: Some(proposal_before_update.proposal_id.clone()),
                    title: proposal_before_update.title.clone(),
                    goal_summary: proposal_before_update.goal_summary.clone(),
                    scope: proposal_scope_to_authorized_scope(&proposal_before_update),
                    actor_id: "project_consultation".to_string(),
                    actor_role: "project_consultant".to_string(),
                    expires_at_ms: None,
                    expected_store_revision: input.expected_plan_authorization_store_revision,
                },
                timestamp_ms,
                authorization_write_id,
            )?;
            (
                created.authorization.authorization_id,
                created.store_revision,
            )
        };
        let confirmed = crate::plan_authorization_store::record_user_confirmation(
            workflow_state_path,
            &RecordPlanAuthorizationUserConfirmationInput {
                project_root: input.project_root.clone(),
                authorization_id,
                actor_id: input.actor_id.trim().to_string(),
                confirmation_summary: input.summary.trim().to_string(),
                expected_store_revision: Some(authorization_revision),
            },
            timestamp_ms + 1,
            authorization_confirm_write_id,
        )?;
        plan_authorization_id = Some(confirmed.authorization.authorization_id.clone());
        linked_authorization = Some(confirmed.authorization);
        linked_authorization_audit = Some(confirmed.audit_event);
        linked_authorization_revision = Some(confirmed.store_revision);
    }

    let next_status = match input.decision {
        ProjectConsultationProposalDecisionKind::Confirm => {
            ProjectConsultationProposalStatus::UserConfirmed
        }
        ProjectConsultationProposalDecisionKind::RequestChanges => {
            ProjectConsultationProposalStatus::ChangesRequested
        }
        ProjectConsultationProposalDecisionKind::Reject => {
            ProjectConsultationProposalStatus::Rejected
        }
    };
    let decision_id = format!(
        "decision:project-consultation-proposal:{}:{}",
        stable_id(&input.proposal_id),
        timestamp_ms
    );
    let audit_event_id = format!(
        "audit:project-consultation-proposal-{}:{}:{}",
        decision_event_suffix(input.decision),
        stable_id(&input.proposal_id),
        timestamp_ms
    );
    let decision = ProjectConsultationProposalDecision {
        decision_id,
        proposal_id: input.proposal_id.clone(),
        decided_by: "user".to_string(),
        decision: input.decision,
        summary: input.summary.trim().to_string(),
        created_at_ms: timestamp_ms,
    };
    {
        let proposal = &mut store.proposals[index];
        proposal.status = next_status;
        proposal.plan_authorization_id = plan_authorization_id.clone();
        proposal.updated_at_ms = timestamp_ms;
    }
    let proposal = store.proposals[index].clone();
    let audit_event = ProjectConsultationProposalAuditEvent {
        audit_event_id,
        event_type: decision_event_type(input.decision).to_string(),
        actor_id: input.actor_id.trim().to_string(),
        actor_role: "user".to_string(),
        project_id: proposal.project_id.clone(),
        workflow_id: proposal.workflow_id.clone(),
        proposal_id: Some(proposal.proposal_id.clone()),
        plan_authorization_id,
        before_status: Some(before),
        after_status: Some(next_status),
        reason: input.summary.trim().to_string(),
        created_at_ms: timestamp_ms,
    };
    store.revision += 1;
    store.updated_at_ms = timestamp_ms;
    store.decisions.push(decision.clone());
    store.audit_events.push(audit_event.clone());
    write_store_atomic(&sidecar, &store, timestamp_ms, proposal_write_id)?;
    drop(lock);

    Ok(RecordProjectConsultationProposalDecisionOutput {
        proposal,
        decision,
        audit_event,
        read_model: summarize_store_for_workflow(
            &store,
            &proposal_before_update.project_id,
            &proposal_before_update.workflow_id,
        ),
        plan_authorization: linked_authorization,
        plan_authorization_audit_event: linked_authorization_audit,
        plan_authorization_store_revision: linked_authorization_revision,
        store_revision: store.revision,
        warnings: store.warnings.clone(),
    })
}

fn pending_authorization_for_proposal(
    workflow_state_path: &Path,
    proposal: &ProjectConsultationProposal,
    timestamp_ms: i64,
) -> Result<Option<(PlanAuthorization, i64)>, String> {
    let store = crate::plan_authorization_store::load_store(workflow_state_path, timestamp_ms)?;
    let mut matching = store.authorizations.iter().filter(|authorization| {
        authorization.project_id == proposal.project_id
            && authorization.workflow_id == proposal.workflow_id
            && authorization.source_proposal_id.as_deref() == Some(proposal.proposal_id.as_str())
            && authorization.status == PlanAuthorizationStatus::PendingUserConfirmation
            && authorization.user_confirmation.is_none()
    });
    let Some(authorization) = matching.next() else {
        return Ok(None);
    };
    if matching.next().is_some() {
        return Err(format!(
            "同一待确认方案存在多个待用户确认授权，已拒绝新建授权：{}",
            proposal.proposal_id
        ));
    }
    Ok(Some((authorization.clone(), store.revision)))
}

pub(crate) fn summarize_store_for_workflow(
    store: &ProjectConsultationProposalStoreV1,
    project_id: &str,
    workflow_id: &str,
) -> ProjectConsultationProposalReadModel {
    let matching = store
        .proposals
        .iter()
        .filter(|proposal| proposal.project_id == project_id && proposal.workflow_id == workflow_id)
        .collect::<Vec<_>>();
    let latest = matching.last().copied();
    let decision_count = store
        .decisions
        .iter()
        .filter(|decision| {
            matching
                .iter()
                .any(|proposal| proposal.proposal_id == decision.proposal_id)
        })
        .count();
    let display_text = latest
        .map(|proposal| {
            format!(
                "{}；步骤 {} / 风险 {} / 停止条件 {}",
                status_label(proposal.status),
                proposal.proposed_steps.len(),
                proposal.risks.len(),
                proposal.scope_draft.stop_conditions.len()
            )
        })
        .unwrap_or_else(|| "还没有项目咨询方案草案".to_string());

    ProjectConsultationProposalReadModel {
        sidecar_name: SIDECAR_NAME.to_string(),
        revision: store.revision,
        project_id: project_id.to_string(),
        workflow_id: workflow_id.to_string(),
        proposal_count: matching.len(),
        latest_proposal_id: latest.map(|proposal| proposal.proposal_id.clone()),
        latest_status: latest.map(|proposal| proposal.status),
        linked_plan_authorization_id: latest
            .and_then(|proposal| proposal.plan_authorization_id.clone()),
        decision_count,
        risk_count: latest.map(|proposal| proposal.risks.len()).unwrap_or(0),
        stop_condition_count: latest
            .map(|proposal| proposal.scope_draft.stop_conditions.len())
            .unwrap_or(0),
        display_text,
        warnings: store.warnings.clone(),
    }
}

fn empty_store(timestamp_ms: i64) -> ProjectConsultationProposalStoreV1 {
    ProjectConsultationProposalStoreV1 {
        schema_version: STORE_SCHEMA_VERSION.to_string(),
        revision: 0,
        proposals: vec![],
        decisions: vec![],
        audit_events: vec![],
        updated_at_ms: timestamp_ms,
        warnings: vec![
            "project_consultation_proposal_store_c2_empty_no_user_confirmation".to_string(),
        ],
    }
}

fn validate_store(store: &ProjectConsultationProposalStoreV1) -> Result<(), String> {
    if store.schema_version != STORE_SCHEMA_VERSION {
        return Err(format!(
            "项目咨询方案 schema_version 不匹配：{}",
            store.schema_version
        ));
    }
    if store.revision < 0 {
        return Err("项目咨询方案 revision 不能小于 0".to_string());
    }
    for proposal in &store.proposals {
        if proposal.schema_version != PROPOSAL_SCHEMA_VERSION {
            return Err(format!(
                "项目咨询方案对象 schema_version 不匹配：{}",
                proposal.schema_version
            ));
        }
        validate_proposal_fields(
            &proposal.user_goal,
            &proposal.goal_summary,
            &proposal.proposed_steps,
            &proposal.scope_draft,
            &proposal.acceptance_criteria,
        )?;
    }
    Ok(())
}

fn validate_create_input(input: &CreateProjectConsultationProposalInput) -> Result<(), String> {
    if input.project_root.trim().is_empty() {
        return Err("项目咨询方案缺少 project_root".to_string());
    }
    if input.title.trim().is_empty() {
        return Err("项目咨询方案缺少 title".to_string());
    }
    if input.actor_id.trim().is_empty() {
        return Err("项目咨询方案缺少 actor_id".to_string());
    }
    validate_proposal_fields(
        &input.user_goal,
        &input.goal_summary,
        &input.proposed_steps,
        &input.scope_draft,
        &input.acceptance_criteria,
    )?;
    validate_role_acceptance_criteria(
        &input.worker_acceptance_criteria,
        &input.control_core_acceptance_criteria,
        &input.supervisor_acceptance_criteria,
    )
}

fn validate_proposal_fields(
    user_goal: &str,
    goal_summary: &str,
    proposed_steps: &[String],
    scope_draft: &ProjectConsultationProposalScopeDraft,
    acceptance_criteria: &[String],
) -> Result<(), String> {
    if user_goal.trim().is_empty() {
        return Err("项目咨询方案缺少 user_goal".to_string());
    }
    if goal_summary.trim().is_empty() {
        return Err("项目咨询方案缺少 goal_summary".to_string());
    }
    if trim_non_empty(proposed_steps).is_empty() {
        return Err("项目咨询方案至少需要一个 proposed step".to_string());
    }
    if trim_non_empty(acceptance_criteria).is_empty() {
        return Err("项目咨询方案至少需要一个 acceptance criterion".to_string());
    }
    if scope_draft
        .allowed_read_roots
        .iter()
        .chain(scope_draft.allowed_write_roots.iter())
        .any(|value| value.trim().is_empty())
    {
        return Err("项目咨询方案读写范围包含空值".to_string());
    }
    if scope_draft
        .allowed_role_ids
        .iter()
        .any(|value| value.trim().is_empty())
    {
        return Err("项目咨询方案 allowed_role_ids 包含空值".to_string());
    }
    Ok(())
}

fn validate_role_acceptance_criteria(
    worker_acceptance_criteria: &[String],
    control_core_acceptance_criteria: &[String],
    supervisor_acceptance_criteria: &[String],
) -> Result<(), String> {
    for (field, criteria) in [
        ("worker_acceptance_criteria", worker_acceptance_criteria),
        (
            "control_core_acceptance_criteria",
            control_core_acceptance_criteria,
        ),
        (
            "supervisor_acceptance_criteria",
            supervisor_acceptance_criteria,
        ),
    ] {
        if criteria.is_empty() || criteria.iter().any(|criterion| criterion.trim().is_empty()) {
            return Err(format!("项目咨询方案缺少有效 {field}"));
        }
    }
    Ok(())
}

fn validate_expected_revision(expected: Option<i64>, actual: i64) -> Result<(), String> {
    if let Some(expected) = expected {
        if expected != actual {
            return Err(format!(
                "project_consultation_proposal_store_conflict: expected revision {expected}, actual {actual}"
            ));
        }
    }
    Ok(())
}

fn find_proposal_index(
    store: &ProjectConsultationProposalStoreV1,
    proposal_id: &str,
) -> Result<usize, String> {
    store
        .proposals
        .iter()
        .position(|proposal| proposal.proposal_id == proposal_id)
        .ok_or_else(|| format!("找不到项目咨询方案草案：{proposal_id}"))
}

fn proposal_scope_to_authorized_scope(
    proposal: &ProjectConsultationProposal,
) -> AuthorizedExecutionScope {
    AuthorizedExecutionScope {
        project_id: proposal.project_id.clone(),
        workflow_id: proposal.workflow_id.clone(),
        allowed_role_ids: proposal.scope_draft.allowed_role_ids.clone(),
        allowed_agent_ids: proposal.scope_draft.allowed_agent_ids.clone(),
        allowed_read_roots: proposal.scope_draft.allowed_read_roots.clone(),
        allowed_write_roots: proposal.scope_draft.allowed_write_roots.clone(),
        allowed_tools: proposal.scope_draft.allowed_tools.clone(),
        allowed_checks: proposal.scope_draft.allowed_checks.clone(),
        allowed_task_package_kinds: proposal.scope_draft.allowed_task_package_kinds.clone(),
        max_worker_dispatches: proposal.scope_draft.max_worker_dispatches,
        max_runtime_minutes: proposal.scope_draft.max_runtime_minutes,
        stop_conditions: proposal
            .scope_draft
            .stop_conditions
            .iter()
            .map(|condition| PlanAuthorizationStopCondition {
                condition_id: format!("proposal-stop-{}", stable_id(condition)),
                kind: "project_consultation_stop_condition".to_string(),
                summary: condition.trim().to_string(),
                requires_user_confirmation: true,
            })
            .collect(),
    }
}

fn render_proposal_markdown(proposal: &ProjectConsultationProposal) -> String {
    let mut lines = vec![
        format!("# {}", proposal.title),
        String::new(),
        format!("状态：{}", status_label(proposal.status)),
        String::new(),
        "## 用户目标".to_string(),
        proposal.user_goal.clone(),
        String::new(),
        "## 用户原始需求快照（逐字保留）".to_string(),
        if proposal.user_requirement_snapshot.is_empty() {
            proposal.user_goal.clone()
        } else {
            proposal.user_requirement_snapshot.clone()
        },
        String::new(),
        "## 方案摘要".to_string(),
        proposal.goal_summary.clone(),
        String::new(),
        "## 主要步骤".to_string(),
    ];
    lines.extend(
        proposal
            .proposed_steps
            .iter()
            .map(|step| format!("- {}", step)),
    );
    lines.push(String::new());
    lines.push("## 授权范围草案".to_string());
    lines.push(format!(
        "- 允许角色：{}",
        proposal.scope_draft.allowed_role_ids.join("；")
    ));
    lines.push(format!(
        "- 允许 agent：{}",
        proposal.scope_draft.allowed_agent_ids.join("；")
    ));
    lines.push(format!(
        "- 允许读取：{}",
        proposal.scope_draft.allowed_read_roots.join("；")
    ));
    lines.push(format!(
        "- 允许写入：{}",
        proposal.scope_draft.allowed_write_roots.join("；")
    ));
    lines.push(format!(
        "- 工具 / 检查：{} / {}",
        proposal.scope_draft.allowed_tools.join("；"),
        proposal.scope_draft.allowed_checks.join("；")
    ));
    lines.push(String::new());
    lines.push("## 停止条件".to_string());
    lines.extend(
        proposal
            .scope_draft
            .stop_conditions
            .iter()
            .map(|condition| format!("- {}", condition)),
    );
    lines.push(String::new());
    lines.push("## Worker 验收".to_string());
    lines.extend(
        proposal
            .worker_acceptance_criteria
            .iter()
            .map(|criterion| format!("- {}", criterion)),
    );
    lines.push(String::new());
    lines.push("## 控制核心验收".to_string());
    lines.extend(
        proposal
            .control_core_acceptance_criteria
            .iter()
            .map(|criterion| format!("- {}", criterion)),
    );
    lines.push(String::new());
    lines.push("## 主管验收".to_string());
    lines.extend(
        proposal
            .supervisor_acceptance_criteria
            .iter()
            .map(|criterion| format!("- {}", criterion)),
    );
    lines.push(String::new());
    lines.push("## 旧版统一验收（仅兼容历史方案）".to_string());
    lines.extend(
        proposal
            .acceptance_criteria
            .iter()
            .map(|criterion| format!("- {}", criterion)),
    );
    lines.push(String::new());
    lines.push("> 用户确认后仍需全局主管复核；本方案不会启动真实 worker。".to_string());
    lines.join("\n")
}

fn ensure_workflow_identity(
    workflow_state_path: &Path,
    project_id_value: &str,
    workflow_id_value: &str,
) -> Result<(), String> {
    let text = fs::read_to_string(workflow_state_path).map_err(|error| {
        format!(
            "读取 workflow state 失败，无法校验项目咨询方案上下文 {}：{error}",
            workflow_state_path.display()
        )
    })?;
    let value: Value = serde_json::from_str(&text).map_err(|error| {
        format!("workflow state JSON 解析失败，无法校验项目咨询方案上下文：{error}")
    })?;
    let project_exists = value
        .get("projects")
        .and_then(Value::as_array)
        .is_some_and(|projects| {
            projects.iter().any(|project| {
                optional_string_from(project, "project_id").as_deref() == Some(project_id_value)
            })
        });
    if !project_exists {
        return Err(format!(
            "workflow state 中找不到 project_id，已拒绝项目咨询方案：{project_id_value}"
        ));
    }
    let workflow_exists = value
        .get("workflows")
        .and_then(Value::as_array)
        .is_some_and(|workflows| {
            workflows.iter().any(|workflow| {
                optional_string_from(workflow, "workflow_id").as_deref() == Some(workflow_id_value)
                    && optional_string_from(workflow, "project_id").as_deref()
                        == Some(project_id_value)
            })
        });
    if !workflow_exists {
        return Err(format!(
            "workflow state 中找不到 workflow_id，已拒绝项目咨询方案：{workflow_id_value}"
        ));
    }
    Ok(())
}

fn write_store_atomic(
    sidecar: &Path,
    store: &ProjectConsultationProposalStoreV1,
    timestamp_ms: i64,
    write_id: &str,
) -> Result<(), String> {
    let parent = sidecar
        .parent()
        .ok_or_else(|| format!("项目咨询方案 sidecar 没有父目录：{}", sidecar.display()))?;
    if sidecar.exists() {
        let backup_dir = parent.join("backups");
        fs::create_dir_all(&backup_dir).map_err(|error| {
            format!(
                "创建项目咨询方案备份目录失败 {}：{error}",
                backup_dir.display()
            )
        })?;
        let backup = backup_dir.join(format!(
            "project-proposals.v1.{timestamp_ms}.{}.json",
            store.revision.saturating_sub(1)
        ));
        fs::copy(sidecar, &backup).map_err(|error| {
            format!(
                "备份项目咨询方案 sidecar 失败 {}：{error}",
                backup.display()
            )
        })?;
        prune_backups(&backup_dir, "project-proposals.v1.")?;
    }
    let temp_path = parent.join(format!(
        ".project-proposals.v1.{timestamp_ms}.{write_id}.tmp"
    ));
    let text = serde_json::to_string_pretty(store)
        .map_err(|error| format!("项目咨询方案 sidecar 序列化失败：{error}"))?;
    {
        let mut file = fs::File::create(&temp_path).map_err(|error| {
            format!(
                "创建项目咨询方案临时文件失败 {}：{error}",
                temp_path.display()
            )
        })?;
        file.write_all(text.as_bytes()).map_err(|error| {
            format!(
                "写入项目咨询方案临时文件失败 {}：{error}",
                temp_path.display()
            )
        })?;
        file.sync_all().map_err(|error| {
            format!(
                "同步项目咨询方案临时文件失败 {}：{error}",
                temp_path.display()
            )
        })?;
    }
    fs::rename(&temp_path, sidecar).map_err(|error| {
        format!(
            "原子替换项目咨询方案 sidecar 失败 {}：{error}",
            sidecar.display()
        )
    })?;
    if let Ok(dir) = fs::File::open(parent) {
        let _ = dir.sync_all();
    }
    Ok(())
}

fn prune_backups(backup_dir: &Path, prefix: &str) -> Result<(), String> {
    let mut backups = fs::read_dir(backup_dir)
        .map_err(|error| {
            format!(
                "读取项目咨询方案备份目录失败 {}：{error}",
                backup_dir.display()
            )
        })?
        .filter_map(Result::ok)
        .filter(|entry| entry.file_name().to_string_lossy().starts_with(prefix))
        .collect::<Vec<_>>();
    backups.sort_by_key(|entry| entry.file_name());
    let remove_count = backups.len().saturating_sub(20);
    for entry in backups.into_iter().take(remove_count) {
        let _ = fs::remove_file(entry.path());
    }
    Ok(())
}

fn ensure_sidecar_parent(sidecar: &Path) -> Result<(), String> {
    let parent = sidecar
        .parent()
        .ok_or_else(|| format!("项目咨询方案 sidecar 没有父目录：{}", sidecar.display()))?;
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "创建项目咨询方案 sidecar 目录失败 {}：{error}",
            parent.display()
        )
    })
}

fn lock_path_for(sidecar: &Path) -> Result<PathBuf, String> {
    Ok(sidecar
        .parent()
        .ok_or_else(|| format!("项目咨询方案 sidecar 没有父目录：{}", sidecar.display()))?
        .join(LOCK_NAME))
}

fn optional_string_from(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(|value| value.to_string())
}

fn trim_scope_draft(
    scope: &ProjectConsultationProposalScopeDraft,
) -> ProjectConsultationProposalScopeDraft {
    ProjectConsultationProposalScopeDraft {
        allowed_role_ids: trim_non_empty(&scope.allowed_role_ids),
        allowed_agent_ids: trim_non_empty(&scope.allowed_agent_ids),
        allowed_read_roots: trim_non_empty(&scope.allowed_read_roots),
        allowed_write_roots: trim_non_empty(&scope.allowed_write_roots),
        allowed_tools: trim_non_empty(&scope.allowed_tools),
        allowed_checks: trim_non_empty(&scope.allowed_checks),
        allowed_task_package_kinds: trim_non_empty(&scope.allowed_task_package_kinds),
        stop_conditions: trim_non_empty(&scope.stop_conditions),
        max_worker_dispatches: scope.max_worker_dispatches,
        max_runtime_minutes: scope.max_runtime_minutes,
    }
}

fn trim_non_empty(values: &[String]) -> Vec<String> {
    values
        .iter()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn creator_role_name(role: crate::ProjectConsultationProposalCreatorRole) -> &'static str {
    match role {
        crate::ProjectConsultationProposalCreatorRole::ProjectConsultant => "project_consultant",
        crate::ProjectConsultationProposalCreatorRole::ProjectDirector => "project_director",
        crate::ProjectConsultationProposalCreatorRole::User => "user",
    }
}

fn decision_event_suffix(decision: ProjectConsultationProposalDecisionKind) -> &'static str {
    match decision {
        ProjectConsultationProposalDecisionKind::Confirm => "confirmed-by-user",
        ProjectConsultationProposalDecisionKind::RequestChanges => "changes-requested",
        ProjectConsultationProposalDecisionKind::Reject => "rejected",
    }
}

fn decision_event_type(decision: ProjectConsultationProposalDecisionKind) -> &'static str {
    match decision {
        ProjectConsultationProposalDecisionKind::Confirm => {
            "project_consultation_proposal_confirmed_by_user"
        }
        ProjectConsultationProposalDecisionKind::RequestChanges => {
            "project_consultation_proposal_changes_requested"
        }
        ProjectConsultationProposalDecisionKind::Reject => "project_consultation_proposal_rejected",
    }
}

fn status_label(status: ProjectConsultationProposalStatus) -> &'static str {
    match status {
        ProjectConsultationProposalStatus::Draft => "草案",
        ProjectConsultationProposalStatus::PendingUserConfirmation => "待用户确认",
        ProjectConsultationProposalStatus::UserConfirmed => "用户已确认，待全局复核",
        ProjectConsultationProposalStatus::ChangesRequested => "用户要求修改",
        ProjectConsultationProposalStatus::Rejected => "用户已拒绝",
        ProjectConsultationProposalStatus::Superseded => "已被新方案取代",
    }
}

fn status_name(status: ProjectConsultationProposalStatus) -> &'static str {
    match status {
        ProjectConsultationProposalStatus::Draft => "draft",
        ProjectConsultationProposalStatus::PendingUserConfirmation => "pending_user_confirmation",
        ProjectConsultationProposalStatus::UserConfirmed => "user_confirmed",
        ProjectConsultationProposalStatus::ChangesRequested => "changes_requested",
        ProjectConsultationProposalStatus::Rejected => "rejected",
        ProjectConsultationProposalStatus::Superseded => "superseded",
    }
}

fn project_id(project_root: &str) -> String {
    format!("project:{}", stable_id(project_root))
}

fn default_workflow_id(project_root: &str) -> String {
    format!("workflow:{}:default", stable_id(project_root))
}

fn stable_id(value: &str) -> String {
    value
        .trim()
        .to_lowercase()
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

struct StoreLock {
    path: PathBuf,
}

impl StoreLock {
    fn acquire(path: &Path, write_id: &str) -> Result<Self, String> {
        for retry in 0..=LOCK_RETRY_COUNT {
            match fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path)
            {
                Ok(mut file) => {
                    file.write_all(write_id.as_bytes()).map_err(|error| {
                        format!("写入项目咨询方案 lock 失败 {}：{error}", path.display())
                    })?;
                    return Ok(Self {
                        path: path.to_path_buf(),
                    });
                }
                Err(error)
                    if error.kind() == std::io::ErrorKind::AlreadyExists
                        && retry < LOCK_RETRY_COUNT =>
                {
                    thread::sleep(LOCK_RETRY_DELAY);
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                    return Err(format!(
                        "project_consultation_proposal_store_locked: {}；稍等几秒再点一次就好",
                        path.display()
                    ));
                }
                Err(error) => {
                    return Err(format!(
                        "创建项目咨询方案 lock 失败 {}：{error}",
                        path.display()
                    ));
                }
            }
        }
        unreachable!("有限重试循环会在最后一次返回")
    }
}

impl Drop for StoreLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

#[cfg(test)]
mod lock_retry_tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn test_lock_path(label: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "project-consultation-proposal-lock-{label}-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create temporary lock directory");
        root.join(LOCK_NAME)
    }

    #[test]
    fn store_lock_retries_until_concurrent_writer_releases() {
        let lock_path = test_lock_path("concurrent-writer");
        let holder = StoreLock::acquire(&lock_path, "first-writer").expect("first writer lock");
        let releaser = thread::spawn(move || {
            thread::sleep(Duration::from_millis(120));
            drop(holder);
        });

        let retried = StoreLock::acquire(&lock_path, "second-writer")
            .expect("second writer should acquire after the transient lock clears");
        drop(retried);
        releaser.join().expect("lock holder should finish");
        let _ = fs::remove_dir_all(lock_path.parent().expect("test lock parent"));
    }

    #[test]
    fn store_lock_retries_when_incident_lock_file_is_removed_shortly_afterwards() {
        let lock_path = test_lock_path("incident-replay");
        fs::write(&lock_path, "incident lock").expect("create transient incident lock");
        let release_path = lock_path.clone();
        let releaser = thread::spawn(move || {
            thread::sleep(Duration::from_millis(120));
            fs::remove_file(release_path).expect("release transient incident lock");
        });

        let retried = StoreLock::acquire(&lock_path, "incident-retry")
            .expect("retry should acquire once the incident lock disappears");
        drop(retried);
        releaser
            .join()
            .expect("incident lock releaser should finish");
        let _ = fs::remove_dir_all(lock_path.parent().expect("test lock parent"));
    }

    #[test]
    fn store_lock_exhaustion_tells_user_to_retry_later() {
        let lock_path = test_lock_path("retry-copy");
        let holder = StoreLock::acquire(&lock_path, "holder").expect("hold lock");

        let error = match StoreLock::acquire(&lock_path, "contender") {
            Ok(lock) => {
                drop(lock);
                panic!("lock should remain held");
            }
            Err(error) => error,
        };
        assert!(error.contains("project_consultation_proposal_store_locked"));
        assert!(error.contains("稍等几秒再点一次就好"));

        drop(holder);
        let _ = fs::remove_dir_all(lock_path.parent().expect("test lock parent"));
    }
}
