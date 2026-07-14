use crate::utils::store_paths;
use crate::{
    AuthorizedExecutionScope, AutoDispatchGuardInput, AutoDispatchGuardResult,
    CreatePlanAuthorizationInput, CreatePlanAuthorizationOutput, GlobalBoundaryReviewChecklist,
    GlobalBoundaryReviewFinding, PlanAuthorization, PlanAuthorizationActorScope,
    PlanAuthorizationAuditEvent, PlanAuthorizationGlobalBoundaryReview, PlanAuthorizationReadModel,
    PlanAuthorizationResourceScope, PlanAuthorizationStatus, PlanAuthorizationStoreV1,
    PlanAuthorizationUserConfirmation, ProjectConsultationProposalStatus,
    RecordGlobalBoundaryReviewInput, RecordGlobalBoundaryReviewOutput,
    RecordPlanAuthorizationGlobalBoundaryReviewInput, RecordPlanAuthorizationOutput,
    RecordPlanAuthorizationUserConfirmationInput, RevokePlanAuthorizationInput,
};
use serde_json::Value;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

const STORE_SCHEMA_VERSION: &str = "plan_authorization_store.v1";
const AUTHORIZATION_SCHEMA_VERSION: &str = "plan_authorization.v1";
const SIDECAR_NAME: &str = "plan-authorizations.v1.json";
const LOCK_NAME: &str = ".plan-authorizations.v1.lock";
const LOCK_RETRY_COUNT: usize = 5;
const LOCK_RETRY_DELAY: Duration = Duration::from_millis(100);

pub(crate) fn sidecar_path(workflow_state_path: &Path) -> Result<PathBuf, String> {
    store_paths::sidecar_path(workflow_state_path, SIDECAR_NAME, "方案授权")
}

pub(crate) fn load_store(
    workflow_state_path: &Path,
    timestamp_ms: i64,
) -> Result<PlanAuthorizationStoreV1, String> {
    let sidecar = sidecar_path(workflow_state_path)?;
    if !sidecar.exists() {
        return Ok(empty_store(timestamp_ms));
    }
    let text = fs::read_to_string(&sidecar)
        .map_err(|error| format!("读取方案授权 sidecar 失败 {}：{error}", sidecar.display()))?;
    let store: PlanAuthorizationStoreV1 = serde_json::from_str(&text).map_err(|error| {
        format!(
            "方案授权 sidecar JSON 损坏，已拒绝覆盖 {}：{error}",
            sidecar.display()
        )
    })?;
    validate_store(&store)?;
    Ok(store)
}

pub(crate) fn create_authorization(
    workflow_state_path: &Path,
    input: &CreatePlanAuthorizationInput,
    timestamp_ms: i64,
    write_id: &str,
) -> Result<CreatePlanAuthorizationOutput, String> {
    validate_create_input(input)?;
    let project_id_value = input
        .project_id
        .clone()
        .unwrap_or_else(|| project_id(&input.project_root));
    let workflow_id_value = input
        .workflow_id
        .clone()
        .unwrap_or_else(|| default_workflow_id(&input.project_root));
    if input.scope.project_id != project_id_value || input.scope.workflow_id != workflow_id_value {
        return Err("方案授权 scope 的 project_id / workflow_id 必须和授权对象一致".to_string());
    }
    ensure_workflow_identity(workflow_state_path, &project_id_value, &workflow_id_value)?;

    let sidecar = sidecar_path(workflow_state_path)?;
    ensure_sidecar_parent(&sidecar)?;
    let lock = StoreLock::acquire(&lock_path_for(&sidecar)?, write_id)?;
    let mut store = load_store(workflow_state_path, timestamp_ms)?;
    validate_expected_revision(input.expected_store_revision, store.revision)?;

    let authorization_id = format!(
        "plan-auth:{}:{}",
        stable_id(&format!(
            "{}:{}:{}:{}",
            project_id_value, workflow_id_value, input.title, input.goal_summary
        )),
        timestamp_ms
    );
    let audit_event_id = format!(
        "audit:plan-authorization-created:{}:{}",
        stable_id(&authorization_id),
        timestamp_ms
    );
    let mut authorization = PlanAuthorization {
        authorization_id: authorization_id.clone(),
        schema_version: AUTHORIZATION_SCHEMA_VERSION.to_string(),
        project_id: project_id_value.clone(),
        workflow_id: workflow_id_value.clone(),
        source_proposal_id: input.source_proposal_id.clone(),
        title: input.title.trim().to_string(),
        goal_summary: input.goal_summary.trim().to_string(),
        status: PlanAuthorizationStatus::PendingUserConfirmation,
        scope: input.scope.clone(),
        user_confirmation: None,
        global_boundary_review: None,
        audit_refs: vec![audit_event_id.clone()],
        created_at_ms: timestamp_ms,
        updated_at_ms: timestamp_ms,
        expires_at_ms: input.expires_at_ms,
    };
    let audit_event = PlanAuthorizationAuditEvent {
        audit_event_id: audit_event_id.clone(),
        event_type: "plan_authorization_created".to_string(),
        actor_id: input.actor_id.trim().to_string(),
        actor_role: input.actor_role.trim().to_string(),
        project_id: project_id_value.clone(),
        workflow_id: workflow_id_value.clone(),
        authorization_id: Some(authorization_id),
        work_item_id: None,
        before_status: None,
        after_status: Some(authorization.status),
        reason: "创建方案授权对象；等待用户确认和全局边界复核。".to_string(),
        guard_result: None,
        created_at_ms: timestamp_ms,
    };

    store.revision += 1;
    store.updated_at_ms = timestamp_ms;
    store.authorizations.push(authorization.clone());
    store.audit_events.push(audit_event.clone());
    if let Some(repository) =
        crate::workbench_sqlite_storage_mode::primary_repository_for_write(workflow_state_path)?
    {
        // M4 keeps authorization CAS on the individual authorization record. This creation is
        // therefore 0 -> 1, independent from the sidecar-wide revision counter.
        let mut authorization_value = serde_json::to_value(&authorization)
            .map_err(|error| format!("序列化方案授权 DB 主写记录失败：{error}"))?;
        let authorization_object = authorization_value
            .as_object_mut()
            .ok_or_else(|| "方案授权 DB 主写记录必须是对象".to_string())?;
        authorization_object.insert(
            "proposal_id".to_string(),
            authorization
                .source_proposal_id
                .clone()
                .map(Value::String)
                .unwrap_or(Value::Null),
        );
        authorization_object.insert("revision".to_string(), Value::from(1_i64));
        let audit_value = serde_json::to_value(&audit_event)
            .map_err(|error| format!("序列化方案授权审计 DB 主写记录失败：{error}"))?;
        repository.save_authorization_with_audit(
            &authorization_value,
            0,
            &crate::workbench_sqlite_repository::RepositoryAuditEntry {
                event_id: audit_event.audit_event_id.clone(),
                target_kind: "plan_authorization".to_string(),
                target_id: authorization.authorization_id.clone(),
                payload: audit_value,
            },
            None,
        )?;
        crate::workbench_sqlite_storage_mode::complete_db_primary_json_projection(
            workflow_state_path,
            "plan_authorization",
            || write_store_atomic(&sidecar, &store, timestamp_ms, write_id),
        )?;
    } else {
        write_store_atomic(&sidecar, &store, timestamp_ms, write_id)?;
    }
    drop(lock);
    authorization.audit_refs = vec![audit_event_id];

    Ok(CreatePlanAuthorizationOutput {
        authorization,
        audit_event,
        read_model: summarize_store_for_workflow(
            &store,
            &project_id_value,
            &workflow_id_value,
            timestamp_ms,
        ),
        store_revision: store.revision,
        warnings: store.warnings.clone(),
    })
}

pub(crate) fn replay_db_primary_projection(
    workflow_state_path: &Path,
    authorizations: &[Value],
    audit_events: &[Value],
    replace_db_primary_leading: bool,
    timestamp_ms: i64,
    write_id: &str,
) -> Result<usize, String> {
    if authorizations.is_empty() && audit_events.is_empty() {
        return Ok(0);
    }
    let sidecar = sidecar_path(workflow_state_path)?;
    ensure_sidecar_parent(&sidecar)?;
    let _lock = StoreLock::acquire(&lock_path_for(&sidecar)?, write_id)?;
    let mut store = load_store(workflow_state_path, timestamp_ms)?;
    let mut authorization_writes = 0_i64;
    let mut total_writes = 0;

    for value in authorizations {
        let authorization: PlanAuthorization = serde_json::from_value(value.clone())
            .map_err(|error| format!("DB 方案授权投影记录无法解析：{error}"))?;
        if let Some(existing) = store
            .authorizations
            .iter()
            .find(|existing| existing.authorization_id == authorization.authorization_id)
        {
            if existing != &authorization {
                if !replace_db_primary_leading {
                    return Err(format!(
                        "db_json_projection_hash_mismatch:plan_authorizations:{}",
                        authorization.authorization_id
                    ));
                }
                let index = store
                    .authorizations
                    .iter()
                    .position(|existing| {
                        existing.authorization_id == authorization.authorization_id
                    })
                    .expect("existing authorization index");
                store.authorizations[index] = authorization;
                total_writes += 1;
            }
        } else {
            store.authorizations.push(authorization);
            authorization_writes += 1;
            total_writes += 1;
        }
    }

    for value in audit_events {
        let audit_event: PlanAuthorizationAuditEvent = serde_json::from_value(value.clone())
            .map_err(|error| format!("DB 方案授权审计投影记录无法解析：{error}"))?;
        if let Some(existing) = store
            .audit_events
            .iter()
            .find(|existing| existing.audit_event_id == audit_event.audit_event_id)
        {
            if existing != &audit_event {
                if !replace_db_primary_leading {
                    return Err(format!(
                        "db_json_projection_hash_mismatch:plan_authorization_audit:{}",
                        audit_event.audit_event_id
                    ));
                }
                let index = store
                    .audit_events
                    .iter()
                    .position(|existing| existing.audit_event_id == audit_event.audit_event_id)
                    .expect("existing authorization audit index");
                store.audit_events[index] = audit_event;
                total_writes += 1;
            }
        } else {
            store.audit_events.push(audit_event);
            total_writes += 1;
        }
    }

    if total_writes > 0 {
        store.revision = store
            .revision
            .checked_add(authorization_writes)
            .ok_or_else(|| "方案授权 sidecar revision 已到上限".to_string())?;
        store.updated_at_ms = timestamp_ms;
        validate_store(&store)?;
        write_store_atomic(&sidecar, &store, timestamp_ms, write_id)?;
    }
    Ok(total_writes)
}

pub(crate) fn record_user_confirmation(
    workflow_state_path: &Path,
    input: &RecordPlanAuthorizationUserConfirmationInput,
    timestamp_ms: i64,
    write_id: &str,
) -> Result<RecordPlanAuthorizationOutput, String> {
    let sidecar = sidecar_path(workflow_state_path)?;
    ensure_sidecar_parent(&sidecar)?;
    let lock = StoreLock::acquire(&lock_path_for(&sidecar)?, write_id)?;
    let mut store = load_store(workflow_state_path, timestamp_ms)?;
    validate_expected_revision(input.expected_store_revision, store.revision)?;
    let index = find_authorization_index(&store, &input.authorization_id)?;
    let before = store.authorizations[index].status;
    if !matches!(
        before,
        PlanAuthorizationStatus::Draft | PlanAuthorizationStatus::PendingUserConfirmation
    ) {
        drop(lock);
        return Err(format!(
            "当前方案授权状态不能记录用户确认：{}",
            status_name(before)
        ));
    }

    let audit_event_id = format!(
        "audit:plan-authorization-confirmed-by-user:{}:{}",
        stable_id(&input.authorization_id),
        timestamp_ms
    );
    {
        let authorization = &mut store.authorizations[index];
        authorization.status = PlanAuthorizationStatus::PendingGlobalBoundaryReview;
        authorization.user_confirmation = Some(PlanAuthorizationUserConfirmation {
            confirmed_by: "user".to_string(),
            confirmed_at_ms: timestamp_ms,
            confirmation_summary: input.confirmation_summary.trim().to_string(),
        });
        authorization.updated_at_ms = timestamp_ms;
        authorization.audit_refs.push(audit_event_id.clone());
    }
    let authorization = store.authorizations[index].clone();
    ensure_workflow_identity(
        workflow_state_path,
        &authorization.project_id,
        &authorization.workflow_id,
    )?;
    let audit_event = PlanAuthorizationAuditEvent {
        audit_event_id,
        event_type: "plan_authorization_confirmed_by_user".to_string(),
        actor_id: input.actor_id.trim().to_string(),
        actor_role: "user".to_string(),
        project_id: authorization.project_id.clone(),
        workflow_id: authorization.workflow_id.clone(),
        authorization_id: Some(authorization.authorization_id.clone()),
        work_item_id: None,
        before_status: Some(before),
        after_status: Some(authorization.status),
        reason: input.confirmation_summary.trim().to_string(),
        guard_result: None,
        created_at_ms: timestamp_ms,
    };
    store.revision += 1;
    store.updated_at_ms = timestamp_ms;
    store.audit_events.push(audit_event.clone());
    write_authorization_update_db_primary(
        workflow_state_path,
        &sidecar,
        &store,
        &authorization,
        &audit_event,
        timestamp_ms,
        write_id,
        "plan_authorization_user_confirmation",
    )?;
    drop(lock);
    Ok(RecordPlanAuthorizationOutput {
        authorization,
        audit_event,
        read_model: summarize_store_for_workflow(
            &store,
            &store.authorizations[index].project_id,
            &store.authorizations[index].workflow_id,
            timestamp_ms,
        ),
        store_revision: store.revision,
        warnings: store.warnings.clone(),
    })
}

pub(crate) fn record_global_boundary_review(
    workflow_state_path: &Path,
    input: &RecordPlanAuthorizationGlobalBoundaryReviewInput,
    timestamp_ms: i64,
    write_id: &str,
) -> Result<RecordPlanAuthorizationOutput, String> {
    if !matches!(
        input.review_status.trim(),
        "approved" | "blocked" | "needs_changes"
    ) {
        return Err(format!("未知全局边界复核结论：{}", input.review_status));
    }
    let sidecar = sidecar_path(workflow_state_path)?;
    ensure_sidecar_parent(&sidecar)?;
    let lock = StoreLock::acquire(&lock_path_for(&sidecar)?, write_id)?;
    let mut store = load_store(workflow_state_path, timestamp_ms)?;
    validate_expected_revision(input.expected_store_revision, store.revision)?;
    let index = find_authorization_index(&store, &input.authorization_id)?;
    let before = store.authorizations[index].status;
    if store.authorizations[index].user_confirmation.is_none() {
        drop(lock);
        return Err("方案授权缺少用户确认，不能记录为 active。".to_string());
    }
    let next_status = if input.review_status.trim() == "approved" {
        PlanAuthorizationStatus::Active
    } else {
        PlanAuthorizationStatus::Paused
    };
    let audit_event_id = format!(
        "audit:plan-authorization-boundary-reviewed:{}:{}",
        stable_id(&input.authorization_id),
        timestamp_ms
    );
    {
        let authorization = &mut store.authorizations[index];
        authorization.status = next_status;
        authorization.global_boundary_review = Some(PlanAuthorizationGlobalBoundaryReview {
            reviewed_by: "global_director".to_string(),
            reviewed_at_ms: timestamp_ms,
            status: input.review_status.trim().to_string(),
            summary: input.summary.trim().to_string(),
            source_proposal_id: input.source_proposal_id.clone(),
            checklist: input.checklist.clone(),
            findings: input.findings.clone(),
            reviewed_scope_fingerprint: input.reviewed_scope_fingerprint.clone(),
        });
        authorization.updated_at_ms = timestamp_ms;
        authorization.audit_refs.push(audit_event_id.clone());
    }
    let authorization = store.authorizations[index].clone();
    ensure_workflow_identity(
        workflow_state_path,
        &authorization.project_id,
        &authorization.workflow_id,
    )?;
    let audit_event = PlanAuthorizationAuditEvent {
        audit_event_id,
        event_type: "plan_authorization_boundary_reviewed".to_string(),
        actor_id: input.actor_id.trim().to_string(),
        actor_role: "global_director".to_string(),
        project_id: authorization.project_id.clone(),
        workflow_id: authorization.workflow_id.clone(),
        authorization_id: Some(authorization.authorization_id.clone()),
        work_item_id: None,
        before_status: Some(before),
        after_status: Some(authorization.status),
        reason: input.summary.trim().to_string(),
        guard_result: None,
        created_at_ms: timestamp_ms,
    };
    store.revision += 1;
    store.updated_at_ms = timestamp_ms;
    store.audit_events.push(audit_event.clone());
    write_authorization_update_db_primary(
        workflow_state_path,
        &sidecar,
        &store,
        &authorization,
        &audit_event,
        timestamp_ms,
        write_id,
        "plan_authorization_global_boundary_review",
    )?;
    drop(lock);
    Ok(RecordPlanAuthorizationOutput {
        authorization,
        audit_event,
        read_model: summarize_store_for_workflow(
            &store,
            &store.authorizations[index].project_id,
            &store.authorizations[index].workflow_id,
            timestamp_ms,
        ),
        store_revision: store.revision,
        warnings: store.warnings.clone(),
    })
}

pub(crate) fn record_global_boundary_review_with_proposal(
    workflow_state_path: &Path,
    input: &RecordGlobalBoundaryReviewInput,
    timestamp_ms: i64,
    write_id: &str,
) -> Result<RecordGlobalBoundaryReviewOutput, String> {
    validate_global_boundary_review_input(input)?;
    let proposal_store =
        crate::project_consultation_proposal_store::load_store(workflow_state_path, timestamp_ms)?;
    let proposal = proposal_store
        .proposals
        .iter()
        .find(|proposal| proposal.proposal_id == input.proposal_id)
        .ok_or_else(|| format!("找不到已确认项目咨询方案：{}", input.proposal_id))?;
    if proposal.status != ProjectConsultationProposalStatus::UserConfirmed {
        return Err("项目咨询方案尚未由用户确认，不能做全局边界复核。".to_string());
    }
    if proposal.project_id != input.project_id || proposal.workflow_id != input.workflow_id {
        return Err("项目咨询方案与 C3 输入的 project_id / workflow_id 不一致。".to_string());
    }
    if proposal.plan_authorization_id.as_deref() != Some(input.authorization_id.as_str()) {
        return Err("项目咨询方案缺少匹配的 C1 方案授权回链。".to_string());
    }

    let store = load_store(workflow_state_path, timestamp_ms)?;
    validate_expected_revision(input.expected_authorization_revision, store.revision)?;
    let authorization = store
        .authorizations
        .iter()
        .find(|authorization| authorization.authorization_id == input.authorization_id)
        .ok_or_else(|| format!("找不到方案授权对象：{}", input.authorization_id))?;
    if authorization.project_id != input.project_id
        || authorization.workflow_id != input.workflow_id
    {
        return Err("方案授权对象与 C3 输入的 project_id / workflow_id 不一致。".to_string());
    }
    if authorization.source_proposal_id.as_deref() != Some(input.proposal_id.as_str()) {
        return Err("方案授权对象 source_proposal_id 与项目咨询方案不匹配。".to_string());
    }
    if authorization.user_confirmation.is_none() {
        return Err("方案授权缺少用户确认，不能通过全局边界复核。".to_string());
    }
    if input.review_status.trim() == "approved"
        && (authorization.status == PlanAuthorizationStatus::Active
            || authorization
                .global_boundary_review
                .as_ref()
                .is_some_and(|review| review.status == "approved"))
    {
        return Err("方案授权已通过全局边界复核，拒绝重复 approved。".to_string());
    }
    if input.review_status.trim() == "approved" {
        ensure_checklist_complete(&input.checklist)?;
        if input
            .findings
            .iter()
            .any(|finding| finding.severity.trim() == "blocking")
        {
            return Err("存在 blocking finding，不能批准并生效。".to_string());
        }
    }

    let scope_fingerprint = scope_fingerprint(&authorization.scope);
    let output = record_global_boundary_review(
        workflow_state_path,
        &RecordPlanAuthorizationGlobalBoundaryReviewInput {
            project_root: input.project_root.clone(),
            authorization_id: input.authorization_id.clone(),
            actor_id: input.actor_id.clone(),
            review_status: input.review_status.clone(),
            summary: input.summary.clone(),
            source_proposal_id: Some(input.proposal_id.clone()),
            checklist: Some(input.checklist.clone()),
            findings: input.findings.clone(),
            reviewed_scope_fingerprint: Some(scope_fingerprint),
            expected_store_revision: Some(store.revision),
        },
        timestamp_ms,
        write_id,
    )?;
    let guard_result = crate::control_core::inspect_auto_dispatch_scope(
        &PlanAuthorizationStoreV1 {
            schema_version: STORE_SCHEMA_VERSION.to_string(),
            revision: output.store_revision,
            authorizations: vec![output.authorization.clone()],
            audit_events: vec![],
            updated_at_ms: timestamp_ms,
            warnings: vec![],
        },
        &guard_input_for_authorization(&output.authorization),
        timestamp_ms,
    );

    Ok(RecordGlobalBoundaryReviewOutput {
        authorization: output.authorization,
        audit_event: output.audit_event,
        read_model: output.read_model,
        guard_result,
        store_revision: output.store_revision,
        warnings: output.warnings,
    })
}

pub(crate) fn revoke_authorization(
    workflow_state_path: &Path,
    input: &RevokePlanAuthorizationInput,
    timestamp_ms: i64,
    write_id: &str,
) -> Result<RecordPlanAuthorizationOutput, String> {
    if input.reason.trim().is_empty() {
        return Err("撤销方案授权缺少 reason".to_string());
    }
    let sidecar = sidecar_path(workflow_state_path)?;
    ensure_sidecar_parent(&sidecar)?;
    let lock = StoreLock::acquire(&lock_path_for(&sidecar)?, write_id)?;
    let mut store = load_store(workflow_state_path, timestamp_ms)?;
    validate_expected_revision(input.expected_store_revision, store.revision)?;
    let index = find_authorization_index(&store, &input.authorization_id)?;
    let before = store.authorizations[index].status;
    let audit_event_id = format!(
        "audit:plan-authorization-revoked:{}:{}",
        stable_id(&input.authorization_id),
        timestamp_ms
    );
    {
        let authorization = &mut store.authorizations[index];
        authorization.status = PlanAuthorizationStatus::Revoked;
        authorization.updated_at_ms = timestamp_ms;
        authorization.audit_refs.push(audit_event_id.clone());
    }
    let authorization = store.authorizations[index].clone();
    ensure_workflow_identity(
        workflow_state_path,
        &authorization.project_id,
        &authorization.workflow_id,
    )?;
    let audit_event = PlanAuthorizationAuditEvent {
        audit_event_id,
        event_type: "plan_authorization_revoked".to_string(),
        actor_id: input.actor_id.trim().to_string(),
        actor_role: input.actor_role.trim().to_string(),
        project_id: authorization.project_id.clone(),
        workflow_id: authorization.workflow_id.clone(),
        authorization_id: Some(authorization.authorization_id.clone()),
        work_item_id: None,
        before_status: Some(before),
        after_status: Some(PlanAuthorizationStatus::Revoked),
        reason: input.reason.trim().to_string(),
        guard_result: None,
        created_at_ms: timestamp_ms,
    };
    store.revision += 1;
    store.updated_at_ms = timestamp_ms;
    store.audit_events.push(audit_event.clone());
    write_authorization_update_db_primary(
        workflow_state_path,
        &sidecar,
        &store,
        &authorization,
        &audit_event,
        timestamp_ms,
        write_id,
        "plan_authorization_revoke",
    )?;
    drop(lock);
    Ok(RecordPlanAuthorizationOutput {
        authorization,
        audit_event,
        read_model: summarize_store_for_workflow(
            &store,
            &store.authorizations[index].project_id,
            &store.authorizations[index].workflow_id,
            timestamp_ms,
        ),
        store_revision: store.revision,
        warnings: store.warnings.clone(),
    })
}

pub(crate) fn inspect_auto_dispatch_authorization(
    workflow_state_path: &Path,
    input: &AutoDispatchGuardInput,
    timestamp_ms: i64,
    write_id: &str,
) -> Result<AutoDispatchGuardResult, String> {
    let sidecar = sidecar_path(workflow_state_path)?;
    ensure_sidecar_parent(&sidecar)?;
    let lock = StoreLock::acquire(&lock_path_for(&sidecar)?, write_id)?;
    let mut store = load_store(workflow_state_path, timestamp_ms)?;
    let result = crate::control_core::inspect_auto_dispatch_scope(&store, input, timestamp_ms);
    let audit_event = PlanAuthorizationAuditEvent {
        audit_event_id: format!(
            "audit:auto-dispatch-scope-checked:{}:{}",
            stable_id(&format!(
                "{}:{}:{}:{}",
                input.project_id, input.workflow_id, input.work_item_id, input.dispatch_kind
            )),
            timestamp_ms
        ),
        event_type: "auto_dispatch_scope_checked".to_string(),
        actor_id: "control_core".to_string(),
        actor_role: "control_core".to_string(),
        project_id: input.project_id.clone(),
        workflow_id: input.workflow_id.clone(),
        authorization_id: result.authorization_id.clone(),
        work_item_id: Some(input.work_item_id.clone()),
        before_status: None,
        after_status: None,
        reason: result.reasons.join("；"),
        guard_result: Some(result.clone()),
        created_at_ms: timestamp_ms,
    };
    store.revision += 1;
    store.updated_at_ms = timestamp_ms;
    store.audit_events.push(audit_event.clone());
    if let Some(repository) =
        crate::workbench_sqlite_storage_mode::primary_repository_for_write(workflow_state_path)?
    {
        let audit_value = serde_json::to_value(&audit_event)
            .map_err(|error| format!("序列化自动派发授权检查审计失败：{error}"))?;
        repository.append_audit(
            &crate::workbench_sqlite_repository::RepositoryAuditEntry {
                event_id: audit_event.audit_event_id.clone(),
                target_kind: "plan_authorization".to_string(),
                target_id: result
                    .authorization_id
                    .clone()
                    .unwrap_or_else(|| input.work_item_id.clone()),
                payload: audit_value,
            },
            None,
        )?;
        crate::workbench_sqlite_storage_mode::complete_db_primary_json_projection(
            workflow_state_path,
            "plan_authorization_auto_dispatch_scope_checked",
            || write_store_atomic(&sidecar, &store, timestamp_ms, write_id),
        )?;
    } else {
        write_store_atomic(&sidecar, &store, timestamp_ms, write_id)?;
    }
    drop(lock);
    Ok(result)
}

pub(crate) fn summarize_store_for_workflow(
    store: &PlanAuthorizationStoreV1,
    project_id: &str,
    workflow_id: &str,
    timestamp_ms: i64,
) -> PlanAuthorizationReadModel {
    let matching = store
        .authorizations
        .iter()
        .filter(|authorization| {
            authorization.project_id == project_id && authorization.workflow_id == workflow_id
        })
        .collect::<Vec<_>>();
    let latest = matching.last().copied();
    let active = matching
        .iter()
        .rev()
        .find(|authorization| {
            authorization.status == PlanAuthorizationStatus::Active
                && authorization
                    .expires_at_ms
                    .is_none_or(|expires_at_ms| expires_at_ms > timestamp_ms)
        })
        .copied();
    let source = active.or(latest);
    let actor_scope = source.map(|authorization| PlanAuthorizationActorScope {
        allowed_role_ids: authorization.scope.allowed_role_ids.clone(),
        allowed_agent_ids: authorization.scope.allowed_agent_ids.clone(),
    });
    let resource_scope = source.map(|authorization| PlanAuthorizationResourceScope {
        allowed_read_roots: authorization.scope.allowed_read_roots.clone(),
        allowed_write_roots: authorization.scope.allowed_write_roots.clone(),
        allowed_tools: authorization.scope.allowed_tools.clone(),
        allowed_checks: authorization.scope.allowed_checks.clone(),
        allowed_task_package_kinds: authorization.scope.allowed_task_package_kinds.clone(),
    });
    let recent_audit_event = store
        .audit_events
        .iter()
        .rev()
        .find(|event| event.project_id == project_id && event.workflow_id == workflow_id);
    let status_label = latest
        .map(|authorization| {
            status_label(
                authorization.status,
                authorization.expires_at_ms,
                timestamp_ms,
            )
        })
        .unwrap_or_else(|| "未建立".to_string());
    let display_text = if let Some(authorization) = source {
        format!(
            "{}；角色 {} / agent {} / 读 {} / 写 {} / 工具 {} / 检查 {} / 停止条件 {}",
            status_label,
            authorization.scope.allowed_role_ids.len(),
            authorization.scope.allowed_agent_ids.len(),
            authorization.scope.allowed_read_roots.len(),
            authorization.scope.allowed_write_roots.len(),
            authorization.scope.allowed_tools.len(),
            authorization.scope.allowed_checks.len(),
            authorization.scope.stop_conditions.len(),
        )
    } else {
        "未建立方案授权；不能自动推进".to_string()
    };
    PlanAuthorizationReadModel {
        sidecar_name: SIDECAR_NAME.to_string(),
        revision: store.revision,
        project_id: project_id.to_string(),
        workflow_id: workflow_id.to_string(),
        authorization_count: matching.len(),
        active_authorization_id: active.map(|authorization| authorization.authorization_id.clone()),
        latest_authorization_id: latest.map(|authorization| authorization.authorization_id.clone()),
        latest_status: latest.map(|authorization| authorization.status),
        actor_scope,
        resource_scope,
        stop_condition_count: source
            .map(|authorization| authorization.scope.stop_conditions.len())
            .unwrap_or(0),
        recent_audit_event_id: recent_audit_event.map(|event| event.audit_event_id.clone()),
        recent_guard_result: recent_audit_event.and_then(|event| event.guard_result.clone()),
        display_text,
        warnings: store.warnings.clone(),
    }
}

fn empty_store(timestamp_ms: i64) -> PlanAuthorizationStoreV1 {
    PlanAuthorizationStoreV1 {
        schema_version: STORE_SCHEMA_VERSION.to_string(),
        revision: 0,
        authorizations: vec![],
        audit_events: vec![],
        updated_at_ms: timestamp_ms,
        warnings: vec!["plan_authorization_store_c1_empty_no_auto_dispatch".to_string()],
    }
}

fn validate_store(store: &PlanAuthorizationStoreV1) -> Result<(), String> {
    if store.schema_version != STORE_SCHEMA_VERSION {
        return Err(format!(
            "方案授权 schema_version 不匹配：{}",
            store.schema_version
        ));
    }
    if store.revision < 0 {
        return Err("方案授权 revision 不能小于 0".to_string());
    }
    for authorization in &store.authorizations {
        if authorization.schema_version != AUTHORIZATION_SCHEMA_VERSION {
            return Err(format!(
                "方案授权对象 schema_version 不匹配：{}",
                authorization.schema_version
            ));
        }
        validate_scope(&authorization.scope)?;
    }
    Ok(())
}

fn validate_create_input(input: &CreatePlanAuthorizationInput) -> Result<(), String> {
    if input.project_root.trim().is_empty() {
        return Err("方案授权缺少 project_root".to_string());
    }
    if input.title.trim().is_empty() {
        return Err("方案授权缺少 title".to_string());
    }
    if input.goal_summary.trim().is_empty() {
        return Err("方案授权缺少 goal_summary".to_string());
    }
    if input.actor_id.trim().is_empty() || input.actor_role.trim().is_empty() {
        return Err("方案授权缺少 actor_id 或 actor_role".to_string());
    }
    validate_scope(&input.scope)
}

fn validate_global_boundary_review_input(
    input: &RecordGlobalBoundaryReviewInput,
) -> Result<(), String> {
    if input.project_root.trim().is_empty()
        || input.project_id.trim().is_empty()
        || input.workflow_id.trim().is_empty()
        || input.proposal_id.trim().is_empty()
        || input.authorization_id.trim().is_empty()
    {
        return Err(
            "全局边界复核缺少 project/workflow/proposal/authorization 身份字段".to_string(),
        );
    }
    if input.actor_id.trim().is_empty() {
        return Err("全局边界复核缺少 actor_id".to_string());
    }
    if !matches!(
        input.review_status.trim(),
        "approved" | "blocked" | "needs_changes"
    ) {
        return Err(format!("未知全局边界复核结论：{}", input.review_status));
    }
    let summary = input.summary.trim();
    if summary.is_empty() {
        return Err("全局边界复核缺少 summary".to_string());
    }
    if summary.chars().count() > 1000 {
        return Err("全局边界复核 summary 不能超过 1000 字符".to_string());
    }
    if input.findings.len() > 12 {
        return Err("全局边界复核 findings 不能超过 12 条".to_string());
    }
    for finding in &input.findings {
        validate_global_boundary_review_finding(finding)?;
    }
    Ok(())
}

fn validate_global_boundary_review_finding(
    finding: &GlobalBoundaryReviewFinding,
) -> Result<(), String> {
    if finding.finding_id.trim().is_empty() {
        return Err("全局边界复核 finding 缺少 finding_id".to_string());
    }
    if !matches!(finding.severity.trim(), "info" | "warning" | "blocking") {
        return Err(format!(
            "未知全局边界复核 finding severity：{}",
            finding.severity
        ));
    }
    if finding.summary.trim().is_empty() {
        return Err("全局边界复核 finding 缺少 summary".to_string());
    }
    if finding.summary.chars().count() > 240 {
        return Err("全局边界复核 finding summary 不能超过 240 字符".to_string());
    }
    if finding
        .recommendation
        .as_ref()
        .is_some_and(|recommendation| recommendation.chars().count() > 240)
    {
        return Err("全局边界复核 finding recommendation 不能超过 240 字符".to_string());
    }
    Ok(())
}

fn ensure_checklist_complete(checklist: &GlobalBoundaryReviewChecklist) -> Result<(), String> {
    let missing = [
        (
            checklist.architecture_boundary_checked,
            "architecture_boundary_checked",
        ),
        (
            checklist.cross_project_impact_checked,
            "cross_project_impact_checked",
        ),
        (
            checklist.permission_scope_checked,
            "permission_scope_checked",
        ),
        (
            checklist.read_write_scope_checked,
            "read_write_scope_checked",
        ),
        (
            checklist.tool_and_check_scope_checked,
            "tool_and_check_scope_checked",
        ),
        (checklist.memory_boundary_checked, "memory_boundary_checked"),
        (checklist.stop_conditions_checked, "stop_conditions_checked"),
        (
            checklist.acceptance_criteria_checked,
            "acceptance_criteria_checked",
        ),
    ]
    .into_iter()
    .filter_map(|(checked, key)| (!checked).then_some(key))
    .collect::<Vec<_>>();
    if missing.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "批准并生效前必须完成全部全局边界 checklist：{}",
            missing.join(", ")
        ))
    }
}

fn validate_scope(scope: &AuthorizedExecutionScope) -> Result<(), String> {
    if scope.project_id.trim().is_empty() || scope.workflow_id.trim().is_empty() {
        return Err("方案授权 scope 缺少 project_id 或 workflow_id".to_string());
    }
    if scope
        .allowed_role_ids
        .iter()
        .any(|value| value.trim().is_empty())
    {
        return Err("方案授权 allowed_role_ids 包含空值".to_string());
    }
    if scope
        .allowed_read_roots
        .iter()
        .chain(scope.allowed_write_roots.iter())
        .any(|value| value.trim().is_empty())
    {
        return Err("方案授权读写范围包含空值".to_string());
    }
    Ok(())
}

fn guard_input_for_authorization(authorization: &PlanAuthorization) -> AutoDispatchGuardInput {
    AutoDispatchGuardInput {
        project_id: authorization.project_id.clone(),
        workflow_id: authorization.workflow_id.clone(),
        work_item_id: format!(
            "work-item:global-boundary-review:{}",
            stable_id(&authorization.authorization_id)
        ),
        task_package_id: Some(format!(
            "task-package:global-boundary-review:{}",
            stable_id(&authorization.authorization_id)
        )),
        task_package_kind: authorization
            .scope
            .allowed_task_package_kinds
            .first()
            .cloned()
            .or_else(|| Some("task_package".to_string())),
        target_role_id: authorization
            .scope
            .allowed_role_ids
            .first()
            .cloned()
            .unwrap_or_else(|| "project_director".to_string()),
        target_agent_id: authorization.scope.allowed_agent_ids.first().cloned(),
        requested_read_roots: authorization.scope.allowed_read_roots.clone(),
        requested_write_roots: authorization.scope.allowed_write_roots.clone(),
        requested_tools: authorization.scope.allowed_tools.clone(),
        requested_checks: authorization.scope.allowed_checks.clone(),
        triggered_stop_conditions: vec![],
        dispatch_kind: "inspect_only".to_string(),
    }
}

fn scope_fingerprint(scope: &AuthorizedExecutionScope) -> String {
    stable_id(&serde_json::to_string(scope).unwrap_or_else(|_| {
        format!(
            "{}:{}:{}:{}:{}:{}",
            scope.project_id,
            scope.workflow_id,
            scope.allowed_role_ids.join(","),
            scope.allowed_read_roots.join(","),
            scope.allowed_write_roots.join(","),
            scope.allowed_tools.join(",")
        )
    }))
}

fn validate_expected_revision(expected: Option<i64>, actual: i64) -> Result<(), String> {
    if let Some(expected) = expected {
        if expected != actual {
            return Err(format!(
                "plan_authorization_store_conflict: expected revision {expected}, actual {actual}"
            ));
        }
    }
    Ok(())
}

fn find_authorization_index(
    store: &PlanAuthorizationStoreV1,
    authorization_id: &str,
) -> Result<usize, String> {
    store
        .authorizations
        .iter()
        .position(|authorization| authorization.authorization_id == authorization_id)
        .ok_or_else(|| format!("找不到方案授权对象：{authorization_id}"))
}

fn ensure_workflow_identity(
    workflow_state_path: &Path,
    project_id_value: &str,
    workflow_id_value: &str,
) -> Result<(), String> {
    let text = fs::read_to_string(workflow_state_path).map_err(|error| {
        format!(
            "读取 workflow state 失败，无法校验方案授权上下文 {}：{error}",
            workflow_state_path.display()
        )
    })?;
    let value: Value = serde_json::from_str(&text).map_err(|error| {
        format!("workflow state JSON 解析失败，无法校验方案授权上下文：{error}")
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
            "workflow state 中找不到 project_id，已拒绝方案授权：{project_id_value}"
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
            "workflow state 中找不到 workflow_id，已拒绝方案授权：{workflow_id_value}"
        ));
    }
    Ok(())
}

fn write_authorization_update_db_primary(
    workflow_state_path: &Path,
    sidecar: &Path,
    store: &PlanAuthorizationStoreV1,
    authorization: &PlanAuthorization,
    audit_event: &PlanAuthorizationAuditEvent,
    timestamp_ms: i64,
    write_id: &str,
    phase: &str,
) -> Result<(), String> {
    let Some(repository) =
        crate::workbench_sqlite_storage_mode::primary_repository_for_write(workflow_state_path)?
    else {
        return write_store_atomic(sidecar, store, timestamp_ms, write_id);
    };
    let authorization_value = serde_json::to_value(authorization)
        .map_err(|error| format!("序列化方案授权 DB 主写记录失败：{error}"))?;
    let audit_value = serde_json::to_value(audit_event)
        .map_err(|error| format!("序列化方案授权审计 DB 主写记录失败：{error}"))?;
    repository.upsert_plan_authorization_with_audit(
        &authorization_value,
        &crate::workbench_sqlite_repository::RepositoryAuditEntry {
            event_id: audit_event.audit_event_id.clone(),
            target_kind: "plan_authorization".to_string(),
            target_id: authorization.authorization_id.clone(),
            payload: audit_value,
        },
        None,
    )?;
    crate::workbench_sqlite_storage_mode::complete_db_primary_json_projection(
        workflow_state_path,
        phase,
        || write_store_atomic(sidecar, store, timestamp_ms, write_id),
    )
}

fn write_store_atomic(
    sidecar: &Path,
    store: &PlanAuthorizationStoreV1,
    timestamp_ms: i64,
    write_id: &str,
) -> Result<(), String> {
    let parent = sidecar
        .parent()
        .ok_or_else(|| format!("方案授权 sidecar 没有父目录：{}", sidecar.display()))?;
    if sidecar.exists() {
        let backup_dir = parent.join("backups");
        fs::create_dir_all(&backup_dir).map_err(|error| {
            format!("创建方案授权备份目录失败 {}：{error}", backup_dir.display())
        })?;
        let backup = backup_dir.join(format!(
            "plan-authorizations.v1.{timestamp_ms}.{}.json",
            store.revision.saturating_sub(1)
        ));
        fs::copy(sidecar, &backup)
            .map_err(|error| format!("备份方案授权 sidecar 失败 {}：{error}", backup.display()))?;
        prune_backups(&backup_dir, "plan-authorizations.v1.")?;
    }
    let temp_path = parent.join(format!(
        ".plan-authorizations.v1.{timestamp_ms}.{write_id}.tmp"
    ));
    let text = serde_json::to_string_pretty(store)
        .map_err(|error| format!("方案授权 sidecar 序列化失败：{error}"))?;
    {
        let mut file = fs::File::create(&temp_path).map_err(|error| {
            format!("创建方案授权临时文件失败 {}：{error}", temp_path.display())
        })?;
        file.write_all(text.as_bytes()).map_err(|error| {
            format!("写入方案授权临时文件失败 {}：{error}", temp_path.display())
        })?;
        file.sync_all().map_err(|error| {
            format!("同步方案授权临时文件失败 {}：{error}", temp_path.display())
        })?;
    }
    fs::rename(&temp_path, sidecar).map_err(|error| {
        format!(
            "原子替换方案授权 sidecar 失败 {}：{error}",
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
        .map_err(|error| format!("读取方案授权备份目录失败 {}：{error}", backup_dir.display()))?
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
        .ok_or_else(|| format!("方案授权 sidecar 没有父目录：{}", sidecar.display()))?;
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "创建方案授权 sidecar 目录失败 {}：{error}",
            parent.display()
        )
    })
}

fn lock_path_for(sidecar: &Path) -> Result<PathBuf, String> {
    Ok(sidecar
        .parent()
        .ok_or_else(|| format!("方案授权 sidecar 没有父目录：{}", sidecar.display()))?
        .join(LOCK_NAME))
}

fn optional_string_from(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(|value| value.to_string())
}

fn status_label(
    status: PlanAuthorizationStatus,
    expires_at_ms: Option<i64>,
    timestamp_ms: i64,
) -> String {
    if expires_at_ms.is_some_and(|expires_at_ms| expires_at_ms <= timestamp_ms) {
        return "已过期".to_string();
    }
    match status {
        PlanAuthorizationStatus::Draft => "草稿".to_string(),
        PlanAuthorizationStatus::PendingUserConfirmation => "待用户确认".to_string(),
        PlanAuthorizationStatus::UserConfirmed => "用户已确认，待全局复核".to_string(),
        PlanAuthorizationStatus::PendingGlobalBoundaryReview => "待全局复核".to_string(),
        PlanAuthorizationStatus::Active => "授权有效".to_string(),
        PlanAuthorizationStatus::Paused => "已暂停".to_string(),
        PlanAuthorizationStatus::Revoked => "已撤销".to_string(),
        PlanAuthorizationStatus::Expired => "已过期".to_string(),
        PlanAuthorizationStatus::Completed => "已完成".to_string(),
    }
}

fn status_name(status: PlanAuthorizationStatus) -> &'static str {
    match status {
        PlanAuthorizationStatus::Draft => "draft",
        PlanAuthorizationStatus::PendingUserConfirmation => "pending_user_confirmation",
        PlanAuthorizationStatus::UserConfirmed => "user_confirmed",
        PlanAuthorizationStatus::PendingGlobalBoundaryReview => "pending_global_boundary_review",
        PlanAuthorizationStatus::Active => "active",
        PlanAuthorizationStatus::Paused => "paused",
        PlanAuthorizationStatus::Revoked => "revoked",
        PlanAuthorizationStatus::Expired => "expired",
        PlanAuthorizationStatus::Completed => "completed",
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
                        format!("写入方案授权 lock 失败 {}：{error}", path.display())
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
                        "plan_authorization_store_locked: {}；稍等几秒再点一次就好",
                        path.display()
                    ));
                }
                Err(error) => {
                    return Err(format!(
                        "创建方案授权 lock 失败 {}：{error}",
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
            "plan-authorization-lock-{label}-{}-{unique}",
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
        assert!(error.contains("plan_authorization_store_locked"));
        assert!(error.contains("稍等几秒再点一次就好"));

        drop(holder);
        let _ = fs::remove_dir_all(lock_path.parent().expect("test lock parent"));
    }
}
