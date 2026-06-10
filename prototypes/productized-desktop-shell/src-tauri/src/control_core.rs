// Control core guard helpers for fact-changing workflow commands.
// These helpers centralize state, permission, dispatch, review, and blackboard
// boundary checks without changing the workflow-state JSON shape.

use std::path::{Component, Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum BlackboardCandidateDecisionOutcome {
    Pending,
    ConfirmedForFollowup,
    Rejected,
    Deferred,
    Discarded,
}

pub(crate) fn work_item_transition_allowed(before: &str, after: &str) -> bool {
    if before == after {
        return false;
    }
    if after == "paused" {
        return before != "accepted";
    }
    matches!(
        (before, after),
        ("draft", "ready_to_dispatch")
            | ("ready_to_dispatch", "running")
            | ("running", "waiting_for_permission")
            | ("running", "retry_pending")
            | ("running", "failed")
            | ("running", "timed_out")
            | ("running", "cancelled")
            | ("running", "ready_for_review")
            | ("waiting_for_permission", "running")
            | ("waiting_for_permission", "failed")
            | ("waiting_for_permission", "cancelled")
            | ("retry_pending", "running")
            | ("retry_pending", "failed")
            | ("failed", "retry_pending")
            | ("failed", "needs_changes")
            | ("timed_out", "retry_pending")
            | ("timed_out", "needs_changes")
            | ("cancelled", "needs_changes")
            | ("ready_for_review", "accepted")
            | ("ready_for_review", "needs_changes")
            | ("needs_changes", "ready_to_dispatch")
            | ("paused", "ready_to_dispatch")
    )
}

pub(crate) fn validate_work_item_state_transition(before: &str, after: &str) -> Result<(), String> {
    if work_item_transition_allowed(before, after) {
        Ok(())
    } else {
        Err(format!("非法工作项状态跳转：{before} -> {after}"))
    }
}

pub(crate) fn validate_dispatch_prepare(work_item_state: &str) -> Result<(), String> {
    if work_item_state == "ready_to_dispatch" {
        Ok(())
    } else {
        Err(format!(
            "工作项当前状态不是待派发，控制核心已拒绝准备派发：{work_item_state}"
        ))
    }
}

pub(crate) fn validate_dispatch_start(work_item_state: &str) -> Result<(), String> {
    if work_item_state == "ready_to_dispatch" {
        Ok(())
    } else {
        Err(format!(
            "工作项当前状态不是待派发，控制核心已拒绝启动派发：{work_item_state}"
        ))
    }
}

pub(crate) fn validate_dispatch_completion_transition(
    work_item_state: &str,
    next_state: &str,
) -> Result<(), String> {
    validate_work_item_state_transition(work_item_state, next_state).map_err(|_| {
        format!("派发结果不能把工作项从 {work_item_state} 推进到 {next_state}，控制核心已拒绝")
    })
}

pub(crate) fn validate_offline_role_handoff(
    dispatch_state: &str,
    next_work_item_state: &str,
) -> Result<(), String> {
    if dispatch_state != "prepared" {
        return Err(format!(
            "离线派发记录不是 prepared，控制核心已拒绝记录回传：{dispatch_state}"
        ));
    }
    if next_work_item_state != "ready_for_review" {
        return Err(format!(
            "离线回传只能进入 ready_for_review，控制核心已拒绝：{next_work_item_state}"
        ));
    }
    Ok(())
}

pub(crate) fn validate_director_review(
    work_item_state: &str,
    dispatch_state: &str,
    decision: &str,
) -> Result<(), String> {
    validate_director_review_work_item_state(work_item_state)?;
    if dispatch_state != "completed" {
        return Err(format!(
            "派发记录不是 completed，控制核心已拒绝总指导回收：{dispatch_state}"
        ));
    }
    if matches!(
        decision,
        "accepted" | "needs_changes" | "paused" | "discarded"
    ) {
        Ok(())
    } else {
        Err(format!("未知总指导回收结论：{decision}"))
    }
}

pub(crate) fn validate_director_review_work_item_state(
    work_item_state: &str,
) -> Result<(), String> {
    if work_item_state == "ready_for_review" {
        Ok(())
    } else {
        Err(format!(
            "工作项当前状态不是待回收，控制核心已拒绝总指导回收：{work_item_state}"
        ))
    }
}

pub(crate) fn validate_permission_decision(status: &str, decision: &str) -> Result<(), String> {
    if status != "pending" {
        return Err(format!(
            "权限请求当前状态不是 pending，控制核心已拒绝重复记录结论：{status}"
        ));
    }
    if matches!(decision, "approved" | "rejected") {
        Ok(())
    } else {
        Err(format!("未知权限结论：{decision}"))
    }
}

pub(crate) fn validate_workflow_machine_start_state(work_item_state: &str) -> Result<(), String> {
    if work_item_state == "ready_to_dispatch" || work_item_state == "needs_changes" {
        Ok(())
    } else {
        Err(format!(
            "工作项当前状态不是待派发或需修改，控制核心已拒绝运行工作流机器：{work_item_state}"
        ))
    }
}

pub(crate) fn validate_workflow_machine_final_state(final_state: &str) -> Result<(), String> {
    if matches!(final_state, "accepted" | "needs_changes" | "failed") {
        Ok(())
    } else {
        Err(format!(
            "未知工作流机器收口状态，控制核心已拒绝写入：{final_state}"
        ))
    }
}

pub(crate) fn validate_blackboard_candidate_decision(
    entry_kind: &str,
    target_kind: &str,
    decision: &str,
) -> Result<BlackboardCandidateDecisionOutcome, String> {
    match decision {
        "mark_pending" | "pending" | "candidate_pending_control_core" => {
            Ok(BlackboardCandidateDecisionOutcome::Pending)
        }
        "candidate_confirmed_for_followup" => {
            validate_blackboard_candidate_followup_target(entry_kind, target_kind)?;
            Ok(BlackboardCandidateDecisionOutcome::ConfirmedForFollowup)
        }
        "reject_candidate" | "rejected" | "candidate_rejected" => {
            Ok(BlackboardCandidateDecisionOutcome::Rejected)
        }
        "candidate_deferred" => Ok(BlackboardCandidateDecisionOutcome::Deferred),
        "candidate_discarded" => Ok(BlackboardCandidateDecisionOutcome::Discarded),
        "confirm_candidate" | "approved" => {
            Err(blackboard_confirmation_rejection(entry_kind, target_kind))
        }
        "candidate_confirmed_for_memory"
        | "candidate_confirmed_for_fact"
        | "permission_approved"
        | "workflow_state_change"
        | "memory_active" => Err(blackboard_confirmation_rejection(entry_kind, target_kind)),
        other => Err(format!("未知黑板候选处理动作：{other}")),
    }
}

fn validate_blackboard_candidate_followup_target(
    entry_kind: &str,
    target_kind: &str,
) -> Result<(), String> {
    if target_kind == "permission_approved" || target_kind == "workflow_state_change" {
        return Err(blackboard_confirmation_rejection(entry_kind, target_kind));
    }
    Ok(())
}

fn blackboard_confirmation_rejection(entry_kind: &str, target_kind: &str) -> String {
    match (entry_kind, target_kind) {
        ("permission_request", "permission_decision") => {
            "权限请求必须走权限确认命令；项目黑板不能直接批准或拒绝权限".to_string()
        }
        ("memory_candidate", "formal_memory") => {
            "记忆候选缺少正式记忆写入计划；项目黑板不能直接写正式记忆".to_string()
        }
        ("knowledge_ref", "formal_memory") => {
            "知识引用不是记忆；项目黑板不能把知识引用直接升级为正式记忆".to_string()
        }
        ("tool_summary", "workflow_state_change") | ("tool_summary", "workflow_fact") => {
            "工具摘要只能作为候选资料；项目黑板不能直接推进 workflow state".to_string()
        }
        ("subagent_report", "workflow_fact") | ("risk", "workflow_risk") => {
            "子汇报和风险缺少事实晋升计划；项目黑板不能直接写正式事实".to_string()
        }
        _ => format!(
            "黑板候选缺少目标事实类型或迁移计划，控制核心已拒绝：{entry_kind} -> {target_kind}"
        ),
    }
}

pub(crate) fn inspect_auto_dispatch_scope(
    store: &crate::PlanAuthorizationStoreV1,
    input: &crate::AutoDispatchGuardInput,
    checked_at_ms: i64,
) -> crate::AutoDispatchGuardResult {
    let matching = store
        .authorizations
        .iter()
        .filter(|authorization| {
            authorization.project_id == input.project_id
                && authorization.workflow_id == input.workflow_id
        })
        .collect::<Vec<_>>();
    if matching.is_empty() {
        return guard_result(
            "blocked",
            None,
            vec!["缺少有效方案授权".to_string()],
            true,
            true,
            checked_at_ms,
        );
    }

    let active = matching
        .iter()
        .rev()
        .find(|authorization| active_authorization_ready(authorization, checked_at_ms))
        .copied();
    let Some(authorization) = active else {
        let latest = matching
            .last()
            .expect("matching authorizations cannot be empty here");
        return inactive_authorization_result(latest, checked_at_ms);
    };

    let mut blocked_reasons = Vec::new();
    let mut needs_review_reasons = Vec::new();
    if !matches!(
        input.dispatch_kind.as_str(),
        "inspect_only" | "prepare_offline" | "prepare_real"
    ) {
        blocked_reasons.push(format!("未知派发类型：{}", input.dispatch_kind));
    }
    check_allowed_value(
        "目标角色不在授权范围内",
        &input.target_role_id,
        &authorization.scope.allowed_role_ids,
        &mut blocked_reasons,
    );
    match input.target_agent_id.as_deref() {
        Some(agent_id) if !agent_id.trim().is_empty() => check_allowed_value(
            "目标 agent 不在授权范围内",
            agent_id,
            &authorization.scope.allowed_agent_ids,
            &mut blocked_reasons,
        ),
        _ if !authorization.scope.allowed_agent_ids.is_empty() => {
            blocked_reasons.push("目标 agent 缺失，无法确认是否在授权范围内".to_string())
        }
        _ => {}
    }
    check_requested_paths(
        "读取范围超出方案授权",
        &input.requested_read_roots,
        &authorization.scope.allowed_read_roots,
        &mut blocked_reasons,
    );
    check_requested_paths(
        "写入范围超出方案授权",
        &input.requested_write_roots,
        &authorization.scope.allowed_write_roots,
        &mut blocked_reasons,
    );
    check_requested_values(
        "工具超出方案授权",
        &input.requested_tools,
        &authorization.scope.allowed_tools,
        &mut blocked_reasons,
    );
    check_requested_values(
        "检查超出方案授权",
        &input.requested_checks,
        &authorization.scope.allowed_checks,
        &mut blocked_reasons,
    );
    if !authorization.scope.allowed_task_package_kinds.is_empty() {
        match input.task_package_kind.as_deref() {
            Some(kind) if !kind.trim().is_empty() => check_allowed_value(
                "任务包类型超出方案授权",
                kind,
                &authorization.scope.allowed_task_package_kinds,
                &mut blocked_reasons,
            ),
            _ => blocked_reasons.push("任务包类型缺失，无法确认是否在授权范围内".to_string()),
        }
    }
    if stop_condition_requires_user(&authorization.scope.stop_conditions, input) {
        needs_review_reasons.push("触发必须请用户确认的停止条件".to_string());
    }

    if !blocked_reasons.is_empty() {
        return guard_result(
            "blocked",
            Some(authorization.authorization_id.clone()),
            blocked_reasons,
            false,
            false,
            checked_at_ms,
        );
    }
    if !needs_review_reasons.is_empty() {
        return guard_result(
            "needs_review",
            Some(authorization.authorization_id.clone()),
            needs_review_reasons,
            true,
            false,
            checked_at_ms,
        );
    }
    guard_result(
        "authorized",
        Some(authorization.authorization_id.clone()),
        Vec::new(),
        false,
        false,
        checked_at_ms,
    )
}

fn active_authorization_ready(
    authorization: &crate::PlanAuthorization,
    checked_at_ms: i64,
) -> bool {
    authorization.status == crate::PlanAuthorizationStatus::Active
        && authorization.user_confirmation.is_some()
        && authorization
            .global_boundary_review
            .as_ref()
            .is_some_and(|review| review.status == "approved")
        && authorization
            .expires_at_ms
            .is_none_or(|expires_at_ms| expires_at_ms > checked_at_ms)
}

fn inactive_authorization_result(
    authorization: &crate::PlanAuthorization,
    checked_at_ms: i64,
) -> crate::AutoDispatchGuardResult {
    if authorization
        .expires_at_ms
        .is_some_and(|expires_at_ms| expires_at_ms <= checked_at_ms)
    {
        return guard_result(
            "blocked",
            Some(authorization.authorization_id.clone()),
            vec!["方案授权已过期".to_string()],
            true,
            true,
            checked_at_ms,
        );
    }
    match authorization.status {
        crate::PlanAuthorizationStatus::Draft
        | crate::PlanAuthorizationStatus::PendingUserConfirmation => guard_result(
            "needs_review",
            Some(authorization.authorization_id.clone()),
            vec!["方案授权待用户确认".to_string()],
            true,
            false,
            checked_at_ms,
        ),
        crate::PlanAuthorizationStatus::UserConfirmed
        | crate::PlanAuthorizationStatus::PendingGlobalBoundaryReview => guard_result(
            "needs_review",
            Some(authorization.authorization_id.clone()),
            vec!["方案授权待全局边界复核".to_string()],
            false,
            true,
            checked_at_ms,
        ),
        crate::PlanAuthorizationStatus::Paused => guard_result(
            "blocked",
            Some(authorization.authorization_id.clone()),
            vec!["方案授权已暂停或边界复核未通过".to_string()],
            false,
            true,
            checked_at_ms,
        ),
        crate::PlanAuthorizationStatus::Revoked => guard_result(
            "blocked",
            Some(authorization.authorization_id.clone()),
            vec!["方案授权已撤销".to_string()],
            true,
            true,
            checked_at_ms,
        ),
        crate::PlanAuthorizationStatus::Expired => guard_result(
            "blocked",
            Some(authorization.authorization_id.clone()),
            vec!["方案授权已过期".to_string()],
            true,
            true,
            checked_at_ms,
        ),
        crate::PlanAuthorizationStatus::Completed => guard_result(
            "blocked",
            Some(authorization.authorization_id.clone()),
            vec!["方案授权已完成，不能继续自动推进".to_string()],
            false,
            false,
            checked_at_ms,
        ),
        crate::PlanAuthorizationStatus::Active => guard_result(
            "needs_review",
            Some(authorization.authorization_id.clone()),
            vec!["方案授权缺少用户确认或全局边界复核记录".to_string()],
            authorization.user_confirmation.is_none(),
            authorization.global_boundary_review.is_none(),
            checked_at_ms,
        ),
    }
}

fn guard_result(
    status: &str,
    authorization_id: Option<String>,
    reasons: Vec<String>,
    required_user_confirmation: bool,
    required_global_review: bool,
    checked_at_ms: i64,
) -> crate::AutoDispatchGuardResult {
    crate::AutoDispatchGuardResult {
        status: status.to_string(),
        authorization_id,
        reasons,
        required_user_confirmation,
        required_global_review,
        checked_at_ms,
    }
}

fn check_allowed_value(
    reason: &str,
    requested: &str,
    allowed: &[String],
    blocked_reasons: &mut Vec<String>,
) {
    if allowed.is_empty() {
        return;
    }
    let requested = normalize_symbol(requested);
    if !allowed
        .iter()
        .any(|allowed| normalize_symbol(allowed) == requested)
    {
        blocked_reasons.push(reason.to_string());
    }
}

fn check_requested_values(
    reason: &str,
    requested: &[String],
    allowed: &[String],
    blocked_reasons: &mut Vec<String>,
) {
    let requested = requested
        .iter()
        .map(|value| normalize_symbol(value))
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    if requested.is_empty() {
        return;
    }
    if allowed.is_empty() {
        blocked_reasons.push(reason.to_string());
        return;
    }
    let allowed = allowed
        .iter()
        .map(|value| normalize_symbol(value))
        .collect::<Vec<_>>();
    if requested
        .iter()
        .any(|requested| !allowed.iter().any(|allowed| allowed == requested))
    {
        blocked_reasons.push(reason.to_string());
    }
}

fn check_requested_paths(
    reason: &str,
    requested: &[String],
    allowed: &[String],
    blocked_reasons: &mut Vec<String>,
) {
    let requested_paths = requested
        .iter()
        .map(|path| normalized_absolute_path(path))
        .collect::<Result<Vec<_>, _>>();
    let requested_paths = match requested_paths {
        Ok(paths) => paths,
        Err(error) => {
            blocked_reasons.push(error);
            return;
        }
    };
    if requested_paths.is_empty() {
        return;
    }
    let allowed_paths = allowed
        .iter()
        .map(|path| normalized_absolute_path(path))
        .collect::<Result<Vec<_>, _>>();
    let allowed_paths = match allowed_paths {
        Ok(paths) => paths,
        Err(error) => {
            blocked_reasons.push(error);
            return;
        }
    };
    if allowed_paths.is_empty()
        || requested_paths.iter().any(|requested| {
            !allowed_paths
                .iter()
                .any(|allowed| path_contains(allowed, requested))
        })
    {
        blocked_reasons.push(reason.to_string());
    }
}

fn normalized_absolute_path(raw: &str) -> Result<PathBuf, String> {
    let cleaned = raw
        .trim()
        .trim_matches('`')
        .trim_matches('"')
        .trim_matches('\'')
        .replace('\\', "/");
    if cleaned.is_empty() {
        return Ok(PathBuf::new());
    }
    let path = Path::new(&cleaned);
    if !path.is_absolute() {
        return Err(format!("授权路径不是绝对路径，无法检查范围：{cleaned}"));
    }
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::RootDir => normalized.push("/"),
            Component::CurDir => {}
            Component::Normal(part) => normalized.push(part),
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Prefix(_) => {
                return Err(format!("授权路径包含不支持的前缀：{cleaned}"));
            }
        }
    }
    Ok(normalized)
}

fn path_contains(allowed: &Path, requested: &Path) -> bool {
    !allowed.as_os_str().is_empty()
        && !requested.as_os_str().is_empty()
        && (requested == allowed || requested.strip_prefix(allowed).is_ok())
}

fn stop_condition_requires_user(
    conditions: &[crate::PlanAuthorizationStopCondition],
    input: &crate::AutoDispatchGuardInput,
) -> bool {
    let triggered = input
        .triggered_stop_conditions
        .iter()
        .map(|condition| normalize_symbol(condition))
        .collect::<Vec<_>>();
    if triggered.is_empty() {
        return false;
    }
    conditions.iter().any(|condition| {
        condition.requires_user_confirmation
            && triggered.iter().any(|trigger| {
                trigger == &normalize_symbol(&condition.condition_id)
                    || trigger == &normalize_symbol(&condition.kind)
                    || trigger == &normalize_symbol(&condition.summary)
            })
    })
}

fn normalize_symbol(value: &str) -> String {
    value.trim().replace('\\', "/").to_lowercase()
}

pub(crate) fn validate_memory_candidate_create(
    source_ref_count: usize,
    scope_type: &str,
    model_export_policy: &str,
    sensitive_level: &str,
) -> Result<(), String> {
    validate_memory_candidate_source_refs(source_ref_count)?;
    validate_memory_candidate_scope(scope_type, model_export_policy)?;
    if sensitive_level == "secret" && model_export_policy != "blocked" {
        return Err("secret 记忆候选必须阻止外发模型上下文".to_string());
    }
    Ok(())
}

pub(crate) fn validate_memory_candidate_scope(
    scope_type: &str,
    model_export_policy: &str,
) -> Result<(), String> {
    if !matches!(
        scope_type,
        "user_preference"
            | "global"
            | "project"
            | "workflow"
            | "session"
            | "role_limited"
            | "document_limited"
    ) {
        return Err(format!("未知记忆作用域：{scope_type}"));
    }
    if !matches!(
        model_export_policy,
        "local_only" | "allowed_with_redaction" | "blocked"
    ) {
        return Err(format!("未知模型外发策略：{model_export_policy}"));
    }
    Ok(())
}

pub(crate) fn validate_memory_candidate_source_refs(source_ref_count: usize) -> Result<(), String> {
    if source_ref_count == 0 {
        Err("记忆候选缺少来源，控制核心已拒绝".to_string())
    } else {
        Ok(())
    }
}

pub(crate) fn validate_memory_candidate_status_transition(
    before: &str,
    after: &str,
) -> Result<(), String> {
    if after.starts_with("memory_") {
        return Err(format!(
            "记忆候选不能直接进入正式记忆状态：{before} -> {after}"
        ));
    }
    if before == after {
        return Err(format!("记忆候选状态没有变化：{before}"));
    }
    if matches!(
        (before, after),
        ("candidate_draft", "candidate_needs_review")
            | ("candidate_needs_review", "candidate_confirmed")
            | ("candidate_needs_review", "candidate_rejected")
            | ("candidate_needs_review", "candidate_quarantined")
            | ("candidate_needs_review", "candidate_discarded")
            | ("candidate_quarantined", "candidate_needs_review")
            | ("candidate_confirmed", "candidate_superseded")
            | ("candidate_confirmed", "candidate_discarded")
    ) {
        Ok(())
    } else {
        Err(format!("非法记忆候选状态跳转：{before} -> {after}"))
    }
}

pub(crate) fn validate_memory_candidate_adoption(
    candidate_status: &str,
    already_adopted: bool,
    source_ref_count: usize,
    source_sensitive_levels: &[String],
    memory_type: &str,
    scope_type: &str,
    model_export_policy: &str,
    risk_level: &str,
    sensitive_level: &str,
    requires_user_confirmation: bool,
    actor_role: &str,
) -> Result<(), String> {
    if candidate_status != "candidate_confirmed" {
        return Err(format!(
            "只能采纳 candidate_confirmed 记忆候选，当前状态：{candidate_status}"
        ));
    }
    if already_adopted {
        return Err("记忆候选已经采纳为正式记忆，控制核心已拒绝重复采纳".to_string());
    }
    validate_formal_memory_source_refs(source_ref_count)?;
    validate_formal_memory_type(memory_type)?;
    validate_memory_candidate_scope(scope_type, model_export_policy)?;
    if source_sensitive_levels
        .iter()
        .any(|level| level == "secret")
        && model_export_policy != "blocked"
    {
        return Err("secret 记忆候选必须阻止外发模型上下文".to_string());
    }
    if sensitive_level == "secret" && model_export_policy != "blocked" {
        return Err("secret 记忆候选必须阻止外发模型上下文".to_string());
    }
    if !matches!(actor_role, "user" | "project_director" | "global_director") {
        return Err(format!(
            "{actor_role} 不允许采纳正式记忆；只能由 user / project_director / global_director 走受控采纳"
        ));
    }
    if actor_role == "global_director" {
        return Err(
            "M2 暂不允许 global_director 采纳正式全局记忆；需要后续任务单独授权".to_string(),
        );
    }
    let requires_user = requires_user_confirmation
        || matches!(
            memory_type,
            "user_preference" | "global_blueprint" | "mature_pattern"
        )
        || matches!(risk_level, "medium" | "high")
        || matches!(sensitive_level, "private" | "secret")
        || matches!(scope_type, "user_preference" | "global");
    if requires_user && actor_role != "user" {
        return Err("该记忆候选必须由 user 采纳，项目主管不能绕过用户确认".to_string());
    }
    if actor_role == "project_director" {
        if !matches!(
            memory_type,
            "project_memory" | "workflow_summary" | "session_summary"
        ) {
            return Err(
                "project_director 只能采纳低风险本项目 project/workflow/session 记忆候选"
                    .to_string(),
            );
        }
        if !matches!(scope_type, "project" | "workflow" | "session") {
            return Err("project_director 只能采纳本项目 scope 记忆候选".to_string());
        }
        if risk_level != "low" {
            return Err("project_director 只能采纳低风险本项目记忆候选".to_string());
        }
        if !matches!(sensitive_level, "public" | "project") {
            return Err("project_director 不能采纳敏感或跨用户范围记忆候选".to_string());
        }
    }
    Ok(())
}

pub(crate) fn validate_observation_create(
    project_root: &str,
    summary: &str,
    source_ref_count: usize,
    source_kinds: &[String],
    source_sensitive_levels: &[String],
    observation_type: &str,
    generated_by_role: &str,
    risk_level: &str,
    sensitive_level: &str,
    scope_type: &str,
    model_export_policy: &str,
) -> Result<(), String> {
    if project_root.trim().is_empty() {
        return Err("observation 创建缺少 project_root".to_string());
    }
    if summary.trim().is_empty() {
        return Err("observation summary 不能为空".to_string());
    }
    if source_ref_count == 0 {
        return Err("observation 缺少 source_refs，控制核心已拒绝".to_string());
    }
    validate_observation_type(observation_type)?;
    validate_observation_actor_role(generated_by_role)?;
    validate_observation_risk_level(risk_level)?;
    validate_observation_sensitive_level(sensitive_level)?;
    validate_memory_candidate_scope(scope_type, model_export_policy)?;

    for source_kind in source_kinds {
        if matches!(
            source_kind.as_str(),
            "ordinary_chat" | "chat" | "chat_message" | "transcript" | "conversation"
        ) {
            return Err(
                "普通聊天不能自动记录为 observation；必须先被明确确认为工作流事实或用户确认来源"
                    .to_string(),
            );
        }
        validate_observation_source_kind(source_kind)?;
    }
    if source_sensitive_levels
        .iter()
        .any(|level| level == "secret")
        && sensitive_level != "secret"
    {
        return Err("secret observation 来源必须保留为 secret 敏感级别".to_string());
    }
    if sensitive_level == "secret" && model_export_policy != "blocked" {
        return Err("secret observation 必须阻止外发模型上下文".to_string());
    }
    Ok(())
}

pub(crate) fn validate_observation_candidate_creation(
    observation_status: &str,
    source_ref_count: usize,
    already_has_candidate: bool,
    actor_role: &str,
    memory_type: &str,
    scope_type: &str,
) -> Result<(), String> {
    if source_ref_count == 0 {
        return Err("observation 缺少 source_refs，控制核心已拒绝生成候选".to_string());
    }
    if already_has_candidate {
        return Err("observation 已经生成过 candidate，控制核心已拒绝重复生成".to_string());
    }
    if observation_status != "recorded" {
        return Err(format!(
            "只能从 recorded observation 生成记忆候选，当前状态：{observation_status}"
        ));
    }
    if actor_role != "project_director" {
        return Err("M3 只允许 project_director 从 observation 生成本项目记忆候选".to_string());
    }
    if !matches!(
        memory_type,
        "project_memory" | "workflow_summary" | "session_summary"
    ) {
        return Err(
            "project_director 只能从 observation 生成本项目 project/workflow/session 记忆候选"
                .to_string(),
        );
    }
    if !matches!(scope_type, "project" | "workflow" | "session") {
        return Err(
            "project_director 只能处理本项目 project/workflow/session observation".to_string(),
        );
    }
    Ok(())
}

pub(crate) fn validate_task_memory_packet_preview(
    project_root: &str,
    task_goal: &str,
    role_id: &str,
    retrieval_intent: &str,
    model_context_policy: &str,
    max_memory_items: usize,
    max_estimated_tokens: usize,
) -> Result<(), String> {
    if project_root.trim().is_empty() {
        return Err("任务记忆包预览缺少 project_root".to_string());
    }
    if task_goal.trim().is_empty() {
        return Err("任务记忆包预览缺少 task_goal".to_string());
    }
    if role_id.trim().is_empty() {
        return Err("任务记忆包预览缺少 role_id".to_string());
    }
    if !matches!(
        retrieval_intent,
        "worker_task" | "project_director_review" | "global_director_review" | "result_acceptance"
    ) {
        return Err(format!(
            "未知任务记忆包 retrieval_intent：{retrieval_intent}"
        ));
    }
    if !matches!(
        model_context_policy,
        "local_only" | "external_model_context"
    ) {
        return Err(format!(
            "未知任务记忆包 model_context_policy：{model_context_policy}"
        ));
    }
    if max_memory_items == 0 || max_memory_items > 20 {
        return Err("任务记忆包 max_memory_items 必须在 1..=20".to_string());
    }
    if max_estimated_tokens == 0 || max_estimated_tokens > 8000 {
        return Err("任务记忆包 max_estimated_tokens 必须在 1..=8000".to_string());
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn evaluate_task_memory_packet_item(
    memory_status: &str,
    conflict_ref_count: usize,
    valid_until: Option<&str>,
    now: &str,
    scope_type: &str,
    scope_project_id: Option<&str>,
    scope_workflow_id: Option<&str>,
    expected_project_id: &str,
    expected_workflow_id: &str,
    model_export_policy: &str,
    model_context_policy: &str,
) -> Option<crate::TaskMemoryPacketExclusionReason> {
    if memory_status == "memory_conflicted" || conflict_ref_count > 0 {
        return Some(crate::TaskMemoryPacketExclusionReason::Conflicted);
    }
    if matches!(
        memory_status,
        "memory_deprecated" | "memory_frozen" | "memory_archived"
    ) {
        return Some(crate::TaskMemoryPacketExclusionReason::Stale);
    }
    if memory_status != "memory_active" {
        return Some(crate::TaskMemoryPacketExclusionReason::StatusNotActive);
    }
    if let Some(valid_until) = valid_until {
        if !valid_until.trim().is_empty() && valid_until <= now {
            return Some(crate::TaskMemoryPacketExclusionReason::Stale);
        }
    }
    if scope_type == "global" {
        // M12: user-confirmed mature pattern / global memories may be recalled
        // across projects, while status, lint, export policy and token guards
        // still apply below.
    } else if matches!(scope_type, "project" | "workflow" | "session") {
        if scope_project_id != Some(expected_project_id) {
            return Some(crate::TaskMemoryPacketExclusionReason::PermissionBlocked);
        }
    } else {
        return Some(crate::TaskMemoryPacketExclusionReason::PermissionBlocked);
    }
    if matches!(scope_type, "workflow" | "session")
        && scope_workflow_id != Some(expected_workflow_id)
    {
        return Some(crate::TaskMemoryPacketExclusionReason::PermissionBlocked);
    }
    if model_context_policy == "external_model_context" && model_export_policy == "blocked" {
        return Some(crate::TaskMemoryPacketExclusionReason::ModelExportBlocked);
    }
    None
}

pub(crate) fn validate_memory_lint_run(
    project_root: &str,
    actor_id: &str,
    actor_role: &str,
    lint_intent: &str,
) -> Result<(), String> {
    if project_root.trim().is_empty() {
        return Err("memory lint 缺少 project_root".to_string());
    }
    if actor_id.trim().is_empty() {
        return Err("memory lint 缺少 actor_id".to_string());
    }
    if !matches!(
        actor_role,
        "project_director" | "global_director" | "system"
    ) {
        return Err(format!("memory lint actor_role 不允许：{actor_role}"));
    }
    if !matches!(
        lint_intent,
        "candidate_adoption_guard"
            | "task_packet_guard"
            | "maintenance_preview"
            | "maintenance_run"
    ) {
        return Err(format!("memory lint_intent 不允许：{lint_intent}"));
    }
    Ok(())
}

fn validate_observation_type(observation_type: &str) -> Result<(), String> {
    if matches!(
        observation_type,
        "worker_report"
            | "process_fact"
            | "project_director_confirmation"
            | "global_director_review"
            | "plan_adopted"
            | "result_acceptance"
    ) {
        Ok(())
    } else {
        Err(format!("未知 observation_type：{observation_type}"))
    }
}

fn validate_observation_source_kind(source_kind: &str) -> Result<(), String> {
    if matches!(
        source_kind,
        "workflow_event"
            | "worker_report"
            | "director_review"
            | "task_package"
            | "evidence"
            | "handoff"
            | "user_confirmation"
    ) {
        Ok(())
    } else {
        Err(format!("未知 observation source_kind：{source_kind}"))
    }
}

fn validate_observation_actor_role(actor_role: &str) -> Result<(), String> {
    if matches!(
        actor_role,
        "worker" | "project_director" | "global_director" | "user" | "system"
    ) {
        Ok(())
    } else {
        Err(format!("未知 observation generated_by_role：{actor_role}"))
    }
}

fn validate_observation_risk_level(risk_level: &str) -> Result<(), String> {
    if matches!(risk_level, "low" | "medium" | "high") {
        Ok(())
    } else {
        Err(format!("未知 observation risk_level：{risk_level}"))
    }
}

fn validate_observation_sensitive_level(sensitive_level: &str) -> Result<(), String> {
    if matches!(
        sensitive_level,
        "public" | "internal" | "sensitive" | "secret"
    ) {
        Ok(())
    } else {
        Err(format!(
            "未知 observation sensitive_level：{sensitive_level}"
        ))
    }
}

pub(crate) fn validate_formal_memory_create(
    claim: &str,
    body: &str,
    source_ref_count: usize,
    source_sensitive_levels: &[String],
    scope_type: &str,
    scope_project_id: Option<&str>,
    scope_workflow_id: Option<&str>,
    model_export_policy: &str,
    memory_type: &str,
    actor_role: &str,
    project_id: Option<&str>,
    workflow_id: Option<&str>,
) -> Result<(), String> {
    if claim.trim().is_empty() {
        return Err("正式记忆 claim 不能为空".to_string());
    }
    if body.trim().is_empty() {
        return Err("正式记忆 body 不能为空".to_string());
    }
    validate_formal_memory_source_refs(source_ref_count)?;
    validate_memory_candidate_scope(scope_type, model_export_policy)?;
    validate_formal_memory_type(memory_type)?;
    validate_formal_memory_actor_role(actor_role)?;
    validate_formal_memory_status("memory_active")?;
    if source_sensitive_levels
        .iter()
        .any(|level| level == "secret")
        && model_export_policy != "blocked"
    {
        return Err("secret 来源的正式记忆必须阻止外发模型上下文".to_string());
    }
    let claim_body_lower = format!("{} {}", claim, body).to_lowercase();
    if (claim_body_lower.contains("[secret]")
        || claim_body_lower.contains("sensitive:secret")
        || claim_body_lower.contains("token:")
        || claim_body_lower.contains("password:"))
        && model_export_policy != "blocked"
    {
        return Err("敏感内容正式记忆必须阻止外发模型上下文".to_string());
    }
    validate_formal_memory_actor_boundary(
        actor_role,
        memory_type,
        scope_type,
        scope_project_id,
        scope_workflow_id,
        project_id,
        workflow_id,
    )
}

pub(crate) fn validate_formal_memory_status(status: &str) -> Result<(), String> {
    if status == "memory_active" {
        Ok(())
    } else {
        Err(format!(
            "正式记忆初始状态只能是 memory_active，控制核心已拒绝：{status}"
        ))
    }
}

fn validate_formal_memory_source_refs(source_ref_count: usize) -> Result<(), String> {
    if source_ref_count == 0 {
        Err("正式记忆缺少来源，控制核心已拒绝".to_string())
    } else {
        Ok(())
    }
}

fn validate_formal_memory_type(memory_type: &str) -> Result<(), String> {
    if matches!(
        memory_type,
        "user_preference"
            | "global_blueprint"
            | "project_memory"
            | "workflow_summary"
            | "session_summary"
            | "mature_pattern"
    ) {
        Ok(())
    } else {
        Err(format!("未知正式记忆类型：{memory_type}"))
    }
}

fn validate_formal_memory_actor_role(actor_role: &str) -> Result<(), String> {
    if matches!(
        actor_role,
        "user" | "project_director" | "global_director" | "system"
    ) {
        Ok(())
    } else {
        Err(format!("未知正式记忆 actor_role：{actor_role}"))
    }
}

fn validate_formal_memory_actor_boundary(
    actor_role: &str,
    memory_type: &str,
    scope_type: &str,
    scope_project_id: Option<&str>,
    scope_workflow_id: Option<&str>,
    project_id: Option<&str>,
    workflow_id: Option<&str>,
) -> Result<(), String> {
    match actor_role {
        "user" => Ok(()),
        "project_director" => validate_project_director_formal_memory_boundary(
            memory_type,
            scope_type,
            scope_project_id,
            scope_workflow_id,
            project_id,
            workflow_id,
        ),
        "global_director" => {
            Err("M1 暂不允许 global_director 创建正式全局记忆；需要后续任务单独授权".to_string())
        }
        "system" => validate_system_formal_memory_boundary(memory_type, scope_type),
        _ => Err(format!("未知正式记忆 actor_role：{actor_role}")),
    }
}

fn validate_project_director_formal_memory_boundary(
    memory_type: &str,
    scope_type: &str,
    scope_project_id: Option<&str>,
    scope_workflow_id: Option<&str>,
    project_id: Option<&str>,
    workflow_id: Option<&str>,
) -> Result<(), String> {
    if !matches!(
        memory_type,
        "project_memory" | "workflow_summary" | "session_summary"
    ) {
        return Err(
            "project_director 只能创建 project_memory、workflow_summary 或 session_summary"
                .to_string(),
        );
    }
    if !matches!(scope_type, "project" | "workflow" | "session") {
        return Err(
            "project_director 只能创建本项目 / workflow / session 作用域正式记忆".to_string(),
        );
    }
    let project_id = project_id.ok_or_else(|| {
        "project_director 创建正式记忆必须带 project_id，控制核心已拒绝".to_string()
    })?;
    if let Some(scope_project_id) = scope_project_id {
        if scope_project_id != project_id {
            return Err("project_director 不能创建其他项目作用域正式记忆".to_string());
        }
    }
    if matches!(scope_type, "workflow" | "session") {
        let workflow_id = workflow_id.ok_or_else(|| {
            "project_director 创建 workflow/session 作用域正式记忆必须带 workflow_id".to_string()
        })?;
        if let Some(scope_workflow_id) = scope_workflow_id {
            if scope_workflow_id != workflow_id {
                return Err("project_director 不能创建其他 workflow 作用域正式记忆".to_string());
            }
        }
    }
    Ok(())
}

fn validate_system_formal_memory_boundary(
    memory_type: &str,
    scope_type: &str,
) -> Result<(), String> {
    if matches!(
        memory_type,
        "user_preference" | "global_blueprint" | "mature_pattern"
    ) || matches!(scope_type, "user_preference" | "global")
    {
        return Err("system 默认不能创建高风险正式记忆".to_string());
    }
    Ok(())
}
