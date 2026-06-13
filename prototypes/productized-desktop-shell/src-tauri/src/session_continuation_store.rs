use crate::utils::hash::{sha256_hex, short_hash};
use crate::{
    codex_local_runner, real_execution_command, runtime_log_store,
    CleanupSessionContinuationStaleAttemptInput, CleanupSessionContinuationStaleAttemptOutput,
    CodexLocalActiveAttempt, CodexLocalAuditRef, CodexLocalExecutionRequest,
    CodexLocalReadbackPlan, CodexLocalRuntimeLogRef, ConfirmControlledSessionContinuationInput,
    ConfirmControlledSessionContinuationOutput, ControlledSessionContinuation,
    H2RealResumeAuthorizationMatrix, H3RealNewSessionAuthorizationMatrix,
    InspectControlledSessionContinuationRealResumeInput,
    InspectControlledSessionContinuationRealResumeOutput,
    RunControlledSessionContinuationRealNewSessionH3BInput,
    RunControlledSessionContinuationRealNewSessionH3BOutput,
    RunControlledSessionContinuationRealResumePhaseAInput,
    RunControlledSessionContinuationRealResumePhaseAOutput,
    RunControlledSessionContinuationRealResumePhaseBInput,
    RunControlledSessionContinuationRealResumePhaseBOutput,
    RunControlledSessionContinuationStubInput, RunControlledSessionContinuationStubOutput,
    SessionContinuationAttempt, SessionContinuationAuditEvent, SessionContinuationPreview,
    SessionContinuationReadbackSummary, SessionContinuationStoreScope, SessionContinuationStoreV1,
};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

const SCHEMA_VERSION: &str = "session_continuation_store.v1";
const STORAGE_KIND: &str = "sidecar_json_v0";
const SIDECAR_NAME: &str = "session-continuations.v1.json";
const LOCK_NAME: &str = ".session-continuations.v1.lock";

pub(crate) fn sidecar_path(workflow_state_path: &Path) -> Result<PathBuf, String> {
    Ok(workflow_state_path
        .parent()
        .ok_or_else(|| {
            format!(
                "workflow state 路径没有父目录，无法推导 continuation sidecar：{}",
                workflow_state_path.display()
            )
        })?
        .join(SIDECAR_NAME))
}

pub(crate) fn load_store(
    workflow_state_path: &Path,
    timestamp: &str,
) -> Result<SessionContinuationStoreV1, String> {
    let sidecar = sidecar_path(workflow_state_path)?;
    if !sidecar.exists() {
        return Ok(empty_store(
            workflow_state_path,
            &sidecar,
            timestamp,
            vec![],
        ));
    }
    let text = fs::read_to_string(&sidecar).map_err(|error| {
        format!(
            "读取 continuation sidecar 失败 {}：{error}",
            sidecar.display()
        )
    })?;
    let store: SessionContinuationStoreV1 = serde_json::from_str(&text).map_err(|error| {
        format!(
            "continuation sidecar JSON 损坏，已拒绝覆盖 {}：{error}",
            sidecar.display()
        )
    })?;
    validate_store(&store)?;
    Ok(store)
}

pub(crate) fn empty_store_with_warning(
    workflow_state_path: &Path,
    timestamp: &str,
    warning: String,
) -> SessionContinuationStoreV1 {
    let sidecar = sidecar_path(workflow_state_path)
        .unwrap_or_else(|_| workflow_state_path.with_file_name(SIDECAR_NAME));
    empty_store(workflow_state_path, &sidecar, timestamp, vec![warning])
}

pub(crate) fn confirm_continuation(
    workflow_state_path: &Path,
    input: &ConfirmControlledSessionContinuationInput,
    timestamp: &str,
    write_id: &str,
) -> Result<ConfirmControlledSessionContinuationOutput, String> {
    validate_confirm_input(input)?;
    let sidecar = sidecar_path(workflow_state_path)?;
    let parent = sidecar
        .parent()
        .ok_or_else(|| format!("continuation sidecar 没有父目录：{}", sidecar.display()))?;
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "创建 continuation sidecar 目录失败 {}：{error}",
            parent.display()
        )
    })?;
    let lock_path = parent.join(LOCK_NAME);
    let lock = StoreLock::acquire(&lock_path, write_id)?;
    let mut store = load_store(workflow_state_path, timestamp)?;
    if let Some(expected) = input.expected_store_revision {
        if expected != store.revision {
            drop(lock);
            return Err(format!(
                "session_continuation_store_conflict: expected revision {expected}, actual {}",
                store.revision
            ));
        }
    }

    let before_store_revision = store.revision;
    let continuation_id = stable_continuation_id(&input.preview);
    let before_index = store
        .continuations
        .iter()
        .position(|record| record.continuation_id == continuation_id);
    let before_status = before_index.map(|index| store.continuations[index].status.clone());
    let audit_ref = format!(
        "audit:session-continuation-confirmed:{}:{}",
        timestamp,
        short_hash(&continuation_id)
    );
    let warnings = level_a_warnings(vec![
        "controlled_session_continuation_only".to_string(),
        "codex_local_only".to_string(),
        "requires_project_workflow_node_session_binding".to_string(),
        "level_b_real_execution_requires_user_approval".to_string(),
        "no_planned_adapter_execution".to_string(),
    ]);
    let continuation = continuation_from_preview(
        &input.preview,
        &continuation_id,
        timestamp,
        input.confirmed_by.trim(),
        input.confirmation_reason.trim(),
        merge_warnings(
            before_index
                .and_then(|index| store.continuations.get(index))
                .map(|record| record.warnings.clone())
                .unwrap_or_default(),
            warnings.clone(),
        ),
    )?;
    match before_index {
        Some(index) => {
            store.continuations[index] = continuation.clone();
        }
        None => {
            store.continuations.push(continuation.clone());
        }
    }
    let next_revision = before_store_revision + 1;
    let audit_event = SessionContinuationAuditEvent {
        event_version: 1,
        event_id: audit_ref.clone(),
        event_type: "session_continuation_preview_confirmed".to_string(),
        continuation_id: continuation_id.clone(),
        attempt_id: None,
        preview_id: input.preview.preview_id.clone(),
        actor_role: input.confirmed_by.trim().to_string(),
        before_status,
        after_status: continuation.status.clone(),
        store_revision: next_revision,
        reason: input.confirmation_reason.trim().to_string(),
        created_at: timestamp.to_string(),
        warnings: warnings.clone(),
    };
    store.audit_events.push(audit_event.clone());
    store.revision = next_revision;
    store.last_write_id = Some(write_id.to_string());
    store.updated_at = timestamp.to_string();
    remember_project_root(&mut store, &continuation.project_root);
    write_store_atomic(&sidecar, &store, timestamp, write_id)?;
    drop(lock);

    Ok(ConfirmControlledSessionContinuationOutput {
        continuation,
        audit_event,
        store_revision: store.revision,
        warnings,
    })
}

pub(crate) fn run_stub(
    workflow_state_path: &Path,
    input: &RunControlledSessionContinuationStubInput,
    timestamp: &str,
    write_id: &str,
) -> Result<RunControlledSessionContinuationStubOutput, String> {
    validate_stub_input(input)?;
    let sidecar = sidecar_path(workflow_state_path)?;
    let parent = sidecar
        .parent()
        .ok_or_else(|| format!("continuation sidecar 没有父目录：{}", sidecar.display()))?;
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "创建 continuation sidecar 目录失败 {}：{error}",
            parent.display()
        )
    })?;
    let lock_path = parent.join(LOCK_NAME);
    let lock = StoreLock::acquire(&lock_path, write_id)?;
    let mut store = load_store(workflow_state_path, timestamp)?;
    if let Some(expected) = input.expected_store_revision {
        if expected != store.revision {
            drop(lock);
            return Err(format!(
                "session_continuation_store_conflict: expected revision {expected}, actual {}",
                store.revision
            ));
        }
    }
    let index = store
        .continuations
        .iter()
        .position(|record| record.continuation_id == input.continuation_id)
        .ok_or_else(|| "未找到 continuation，已拒绝创建 stub attempt".to_string())?;
    let before_status = store.continuations[index].status.clone();
    if before_status != "preview_confirmed" && before_status != "queued" {
        drop(lock);
        return Err(format!(
            "session_continuation_not_runnable_in_stub: {before_status}"
        ));
    }
    if store.continuations[index].adapter_id != "codex-local" {
        drop(lock);
        return Err("planned adapter 不允许进入 E5 stub attempt".to_string());
    }

    let mut continuation = store.continuations[index].clone();
    let attempt_id = format!(
        "session-continuation-attempt:{}:{}",
        timestamp,
        short_hash(&continuation.continuation_id)
    );
    let started_audit_ref = format!(
        "audit:session-continuation-stub-started:{}:{}",
        timestamp,
        short_hash(&attempt_id)
    );
    let completed_audit_ref = format!(
        "audit:session-continuation-stub-completed:{}:{}",
        timestamp,
        short_hash(&attempt_id)
    );
    let failed = input.force_stub_failure.unwrap_or(false);
    let final_status = if failed {
        "failed_stub"
    } else {
        "succeeded_stub"
    };
    let readback_status = if failed {
        "not_attempted_stub"
    } else {
        "readback_unavailable"
    };
    let warnings = level_a_warnings(vec![
        "stub_runner_only".to_string(),
        "prompt_not_sent".to_string(),
        "real_codex_execution_not_authorized".to_string(),
        "codex_home_not_touched".to_string(),
        "readback_unavailable_is_not_zero_results".to_string(),
    ]);
    let attempt = SessionContinuationAttempt {
        attempt_version: 1,
        attempt_id: attempt_id.clone(),
        continuation_id: continuation.continuation_id.clone(),
        runner_kind: "stub".to_string(),
        execution_level: "level_a_stub_only".to_string(),
        status: final_status.to_string(),
        started_at: timestamp.to_string(),
        finished_at: Some(timestamp.to_string()),
        timeout_ms: input.timeout_ms,
        command_preview: continuation.command_preview.clone(),
        prompt_sent: false,
        real_codex_executed: false,
        writes_codex_home: false,
        writes_workbench_state: true,
        readback_summary: SessionContinuationReadbackSummary {
            status: readback_status.to_string(),
            source_kind: "stub_no_transcript_read".to_string(),
            result_count: None,
            unavailable_reason: Some(
                "Level A stub 不读取真实 transcript；unavailable 不等于空读回结果。".to_string(),
            ),
            warnings: vec![
                "readback_unavailable_is_not_zero_results".to_string(),
                "no_real_transcript_read_in_level_a".to_string(),
            ],
        },
        failure_reason: if failed {
            Some("forced_stub_failure_for_test".to_string())
        } else {
            None
        },
        audit_refs: vec![started_audit_ref.clone(), completed_audit_ref.clone()],
        warnings: warnings.clone(),
    };
    let started_event = SessionContinuationAuditEvent {
        event_version: 1,
        event_id: started_audit_ref,
        event_type: "session_continuation_stub_started".to_string(),
        continuation_id: continuation.continuation_id.clone(),
        attempt_id: Some(attempt_id.clone()),
        preview_id: continuation.preview_id.clone(),
        actor_role: input.actor_role.trim().to_string(),
        before_status: Some(before_status),
        after_status: "running_stub".to_string(),
        store_revision: store.revision + 1,
        reason: "Level A stub attempt started; no real Codex execution.".to_string(),
        created_at: timestamp.to_string(),
        warnings: warnings.clone(),
    };
    let completed_event = SessionContinuationAuditEvent {
        event_version: 1,
        event_id: completed_audit_ref,
        event_type: if failed {
            "session_continuation_stub_failed".to_string()
        } else {
            "session_continuation_stub_completed".to_string()
        },
        continuation_id: continuation.continuation_id.clone(),
        attempt_id: Some(attempt_id),
        preview_id: continuation.preview_id.clone(),
        actor_role: input.actor_role.trim().to_string(),
        before_status: Some("running_stub".to_string()),
        after_status: final_status.to_string(),
        store_revision: store.revision + 1,
        reason: if failed {
            "Level A forced stub failure; no prompt sent.".to_string()
        } else {
            "Level A stub completed; readback remains unavailable.".to_string()
        },
        created_at: timestamp.to_string(),
        warnings: warnings.clone(),
    };
    continuation.status = final_status.to_string();
    continuation.updated_at = timestamp.to_string();
    continuation.audit_refs.extend(attempt.audit_refs.clone());
    continuation.audit_refs.sort();
    continuation.audit_refs.dedup();
    continuation.warnings = merge_warnings(continuation.warnings, warnings.clone());
    store.continuations[index] = continuation.clone();
    store.attempts.push(attempt.clone());
    store.audit_events.push(started_event.clone());
    store.audit_events.push(completed_event.clone());
    store.revision += 1;
    store.last_write_id = Some(write_id.to_string());
    store.updated_at = timestamp.to_string();
    remember_project_root(&mut store, &continuation.project_root);
    write_store_atomic(&sidecar, &store, timestamp, write_id)?;
    drop(lock);

    Ok(RunControlledSessionContinuationStubOutput {
        continuation,
        attempt,
        audit_events: vec![started_event, completed_event],
        store_revision: store.revision,
        warnings,
    })
}

pub(crate) fn inspect_real_resume_authorization(
    workflow_state_path: &Path,
    input: &InspectControlledSessionContinuationRealResumeInput,
    timestamp: &str,
    write_id: &str,
) -> Result<InspectControlledSessionContinuationRealResumeOutput, String> {
    validate_real_resume_input(input)?;
    let sidecar = sidecar_path(workflow_state_path)?;
    let parent = sidecar
        .parent()
        .ok_or_else(|| format!("continuation sidecar 没有父目录：{}", sidecar.display()))?;
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "创建 continuation sidecar 目录失败 {}：{error}",
            parent.display()
        )
    })?;
    let lock_path = parent.join(LOCK_NAME);
    let lock = StoreLock::acquire(&lock_path, write_id)?;
    let mut store = load_store(workflow_state_path, timestamp)?;
    if let Some(expected) = input.expected_store_revision {
        if expected != store.revision {
            drop(lock);
            return Err(format!(
                "session_continuation_store_conflict: expected revision {expected}, actual {}",
                store.revision
            ));
        }
    }

    let index = store
        .continuations
        .iter()
        .position(|record| record.continuation_id == input.continuation_id)
        .ok_or_else(|| "未找到 continuation，已拒绝 H2 real resume 预检".to_string())?;
    let before_status = store.continuations[index].status.clone();
    if store.continuations[index].adapter_id != "codex-local" {
        drop(lock);
        return Err("planned adapter 不允许进入 H2 real resume 预检".to_string());
    }
    if store.continuations[index].operation_id != "resume" {
        drop(lock);
        return Err("H2 只允许 resume 预检，不处理 send_message".to_string());
    }
    let mut continuation = store.continuations[index].clone();
    let mut missing = inspect_authorization_matrix(&continuation, &input.authorization);
    if has_active_attempt_in_h4_scope(&store, &continuation) {
        missing.push("duplicate_running_attempt".to_string());
    }
    missing.sort();
    missing.dedup();
    let matrix_authorized = missing.is_empty();
    let codex_local_request = if matrix_authorized {
        Some(build_codex_local_request_for_h2(
            &continuation,
            &input.authorization,
            &store,
        ))
    } else {
        None
    };
    let codex_local_guard = codex_local_request
        .as_ref()
        .map(codex_local_runner::inspect_codex_local_execution_guard);
    if let Some(guard) = &codex_local_guard {
        if guard.blocks_execution {
            missing.push("codex_local_guard_blocked".to_string());
            missing.extend(
                guard
                    .reasons
                    .iter()
                    .map(|reason| format!("codex_local_guard:{reason}")),
            );
            missing.sort();
            missing.dedup();
        }
    }
    let authorized = matrix_authorized
        && codex_local_guard
            .as_ref()
            .is_some_and(|guard| !guard.blocks_execution);
    let attempt_id = format!(
        "session-continuation-attempt:h2-preflight:{}:{}",
        timestamp,
        short_hash(&continuation.continuation_id)
    );
    let audit_ref = format!(
        "audit:session-continuation-h2-real-resume-preflight:{}:{}",
        timestamp,
        short_hash(&attempt_id)
    );
    let status = if authorized {
        "ready_for_real_resume_authorization"
    } else if missing
        .iter()
        .any(|item| item == "duplicate_running_attempt")
    {
        "duplicate_blocked"
    } else if matrix_authorized
        && codex_local_guard
            .as_ref()
            .is_some_and(|guard| guard.blocks_execution)
    {
        "blocked_by_codex_local_guard"
    } else {
        "blocked_waiting_authorization"
    };
    let warnings = h2_preflight_warnings(authorized);
    let attempt = SessionContinuationAttempt {
        attempt_version: 1,
        attempt_id: attempt_id.clone(),
        continuation_id: continuation.continuation_id.clone(),
        runner_kind: "codex_local_real_preflight".to_string(),
        execution_level: "h2_real_resume_preflight_no_execution".to_string(),
        status: status.to_string(),
        started_at: timestamp.to_string(),
        finished_at: Some(timestamp.to_string()),
        timeout_ms: input.authorization.timeout_ms,
        command_preview: redacted_h2_command_preview(&continuation),
        prompt_sent: false,
        real_codex_executed: false,
        writes_codex_home: false,
        writes_workbench_state: true,
        readback_summary: SessionContinuationReadbackSummary {
            status: "readback_unavailable".to_string(),
            source_kind: "h2_preflight_no_transcript_read".to_string(),
            result_count: None,
            unavailable_reason: Some(
                "H2 预检不读取真实 transcript；unavailable 不等于空读回结果。".to_string(),
            ),
            warnings: vec![
                "readback_unavailable_is_not_zero_results".to_string(),
                "no_real_transcript_read_in_h2_preflight".to_string(),
            ],
        },
        failure_reason: if authorized {
            None
        } else {
            Some(format!(
                "blocked_waiting_authorization:{}",
                missing.join(",")
            ))
        },
        audit_refs: vec![audit_ref.clone()],
        warnings: warnings.clone(),
    };
    let audit_event = SessionContinuationAuditEvent {
        event_version: 1,
        event_id: audit_ref,
        event_type: "session_continuation_h2_real_resume_preflight".to_string(),
        continuation_id: continuation.continuation_id.clone(),
        attempt_id: Some(attempt_id),
        preview_id: continuation.preview_id.clone(),
        actor_role: input.actor_role.trim().to_string(),
        before_status: Some(before_status),
        after_status: status.to_string(),
        store_revision: store.revision + 1,
        reason: if authorized {
            "H2 real resume authorization matrix complete; no real Codex execution performed in preflight."
                .to_string()
        } else if matrix_authorized {
            format!(
                "H2 real resume authorization matrix complete but CodexLocal guard blocked execution; missing_or_invalid={}",
                missing.join(",")
            )
        } else {
            format!(
                "H2 real resume blocked before execution; missing_or_invalid={}",
                missing.join(",")
            )
        },
        created_at: timestamp.to_string(),
        warnings: warnings.clone(),
    };

    continuation.status = status.to_string();
    continuation.execution_level = "h2_real_resume_preflight_no_execution".to_string();
    continuation.runner_kind = "codex_local_real_preflight".to_string();
    continuation.updated_at = timestamp.to_string();
    continuation.audit_refs.extend(attempt.audit_refs.clone());
    continuation.audit_refs.sort();
    continuation.audit_refs.dedup();
    continuation.warnings = merge_warnings(continuation.warnings, warnings.clone());
    store.continuations[index] = continuation.clone();
    store.attempts.push(attempt.clone());
    store.audit_events.push(audit_event.clone());
    store.revision += 1;
    store.last_write_id = Some(write_id.to_string());
    store.updated_at = timestamp.to_string();
    remember_project_root(&mut store, &continuation.project_root);
    write_store_atomic(&sidecar, &store, timestamp, write_id)?;
    drop(lock);

    Ok(InspectControlledSessionContinuationRealResumeOutput {
        continuation,
        attempt,
        audit_event,
        store_revision: store.revision,
        authorization_status: if authorized {
            "complete_but_not_executed".to_string()
        } else if matrix_authorized {
            "blocked_by_codex_local_guard".to_string()
        } else {
            "blocked_waiting_authorization".to_string()
        },
        missing_or_invalid_items: missing,
        codex_local_request,
        codex_local_guard,
        warnings,
    })
}

pub(crate) fn run_real_resume_phase_a(
    workflow_state_path: &Path,
    input: &RunControlledSessionContinuationRealResumePhaseAInput,
    timestamp: &str,
    write_id: &str,
) -> Result<RunControlledSessionContinuationRealResumePhaseAOutput, String> {
    let runner = codex_local_runner::NoopCodexLocalPhaseAProcessRunner;
    run_real_resume_phase_a_with_runner(workflow_state_path, input, timestamp, write_id, &runner)
}

pub(crate) fn run_real_resume_phase_b(
    workflow_state_path: &Path,
    input: &RunControlledSessionContinuationRealResumePhaseBInput,
    timestamp: &str,
    write_id: &str,
) -> Result<RunControlledSessionContinuationRealResumePhaseBOutput, String> {
    let runner = codex_local_runner::RealCodexLocalPhaseBProcessRunner;
    let last_message_path = h2_phase_b_last_message_path(workflow_state_path, input, timestamp)?;
    run_real_resume_phase_b_with_runner(
        workflow_state_path,
        input,
        timestamp,
        write_id,
        &last_message_path,
        &runner,
    )
}

pub(crate) fn run_real_new_session_h3_b(
    workflow_state_path: &Path,
    input: &RunControlledSessionContinuationRealNewSessionH3BInput,
    timestamp: &str,
    write_id: &str,
) -> Result<RunControlledSessionContinuationRealNewSessionH3BOutput, String> {
    let runner = codex_local_runner::RealCodexLocalPhaseBProcessRunner;
    let last_message_path = h3_b_last_message_path(workflow_state_path, input, timestamp)?;
    run_real_new_session_h3_b_with_runner(
        workflow_state_path,
        input,
        timestamp,
        write_id,
        &last_message_path,
        &runner,
    )
}

fn run_real_resume_phase_a_with_runner<R: codex_local_runner::CodexLocalPhaseAProcessRunner>(
    workflow_state_path: &Path,
    input: &RunControlledSessionContinuationRealResumePhaseAInput,
    timestamp: &str,
    write_id: &str,
    runner: &R,
) -> Result<RunControlledSessionContinuationRealResumePhaseAOutput, String> {
    validate_real_resume_phase_a_input(input)?;
    let sidecar = sidecar_path(workflow_state_path)?;
    let parent = sidecar
        .parent()
        .ok_or_else(|| format!("continuation sidecar 没有父目录：{}", sidecar.display()))?;
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "创建 continuation sidecar 目录失败 {}：{error}",
            parent.display()
        )
    })?;
    let lock_path = parent.join(LOCK_NAME);
    let lock = StoreLock::acquire(&lock_path, write_id)?;
    runtime_log_store::ensure_appendable(workflow_state_path)?;
    let mut store = load_store(workflow_state_path, timestamp)?;
    if let Some(expected) = input.expected_store_revision {
        if expected != store.revision {
            drop(lock);
            return Err(format!(
                "session_continuation_store_conflict: expected revision {expected}, actual {}",
                store.revision
            ));
        }
    }

    let index = store
        .continuations
        .iter()
        .position(|record| record.continuation_id == input.continuation_id)
        .ok_or_else(|| "未找到 continuation，已拒绝 H2.5 Phase A runner path".to_string())?;
    let before_status = store.continuations[index].status.clone();
    if store.continuations[index].adapter_id != "codex-local" {
        drop(lock);
        return Err("planned adapter 不允许进入 H2.5 Phase A runner path".to_string());
    }
    if store.continuations[index].operation_id != "resume" {
        drop(lock);
        return Err("H2.5 Phase A 只允许 resume，不处理 send_message".to_string());
    }

    let mut continuation = store.continuations[index].clone();
    let user_rejected = input
        .execution_decision
        .as_deref()
        .unwrap_or("approved_for_phase_a")
        == "rejected";
    let mut missing = inspect_authorization_matrix(&continuation, &input.authorization);
    if user_rejected {
        missing.push("user_rejected_real_resume".to_string());
    }
    if has_active_attempt_in_h4_scope(&store, &continuation) {
        missing.push("duplicate_running_attempt".to_string());
    }
    missing.sort();
    missing.dedup();

    let matrix_authorized = missing.is_empty();
    let codex_local_request = if matrix_authorized {
        Some(build_codex_local_request_for_h2(
            &continuation,
            &input.authorization,
            &store,
        ))
    } else {
        None
    };
    let codex_local_guard = codex_local_request
        .as_ref()
        .map(codex_local_runner::inspect_codex_local_execution_guard);
    if let Some(guard) = &codex_local_guard {
        if guard.blocks_execution {
            missing.push("codex_local_guard_blocked".to_string());
            missing.extend(
                guard
                    .reasons
                    .iter()
                    .map(|reason| format!("codex_local_guard:{reason}")),
            );
            missing.sort();
            missing.dedup();
        }
    }
    let authorized = matrix_authorized
        && codex_local_guard
            .as_ref()
            .is_some_and(|guard| !guard.blocks_execution);

    let started_audit_ref = format!(
        "audit:session-continuation-h2-phase-a-started:{}:{}",
        timestamp,
        short_hash(&continuation.continuation_id)
    );
    let completed_audit_ref = format!(
        "audit:session-continuation-h2-phase-a-completed:{}:{}",
        timestamp,
        short_hash(&continuation.continuation_id)
    );
    let blocked_audit_ref = format!(
        "audit:session-continuation-h2-phase-a-blocked:{}:{}",
        timestamp,
        short_hash(&continuation.continuation_id)
    );
    let base_warnings = h2_phase_a_warnings(authorized);

    let (attempt, audit_events, authorization_status, codex_local_attempt) = if !authorized {
        let status = if user_rejected {
            "user_rejected"
        } else if missing
            .iter()
            .any(|item| item == "duplicate_running_attempt")
        {
            "duplicate_blocked"
        } else if missing
            .iter()
            .any(|item| item == "codex_local_guard_blocked")
        {
            "blocked_by_guard"
        } else {
            "blocked_waiting_authorization"
        };
        let attempt_id = format!(
            "session-continuation-attempt:h2-phase-a-blocked:{}:{}",
            timestamp,
            short_hash(&continuation.continuation_id)
        );
        let attempt = phase_a_store_attempt(
            &continuation,
            &attempt_id,
            "codex_local_phase_a_runner_path",
            "h2_phase_a_runner_path_no_real_codex",
            status,
            timestamp,
            input.authorization.timeout_ms,
            status,
            None,
            Some(format!("H2.5 Phase A blocked: {}", missing.join(","))),
            vec![blocked_audit_ref.clone()],
            base_warnings.clone(),
        );
        let audit_event = phase_a_audit_event(
            &continuation,
            &blocked_audit_ref,
            "session_continuation_h2_phase_a_blocked",
            &attempt_id,
            input.actor_role.trim(),
            Some(before_status.clone()),
            status,
            store.revision + 1,
            format!(
                "H2.5 Phase A blocked before runner path: {}",
                missing.join(",")
            ),
            timestamp,
            base_warnings.clone(),
        );
        (attempt, vec![audit_event], status.to_string(), None)
    } else {
        let request = codex_local_request
            .clone()
            .ok_or_else(|| "H2.5 Phase A 缺少 codex-local request".to_string())?;
        let codex_attempt =
            codex_local_runner::run_h2_phase_a_with_runner(request, timestamp, runner);
        let attempt_id = format!(
            "session-continuation-attempt:h2-phase-a:{}:{}",
            timestamp,
            short_hash(&continuation.continuation_id)
        );
        let started_event = phase_a_audit_event(
            &continuation,
            &started_audit_ref,
            "session_continuation_h2_phase_a_started",
            &attempt_id,
            input.actor_role.trim(),
            Some(before_status.clone()),
            "running_h2_phase_a",
            store.revision + 1,
            "H2.5 Phase A runner path started; no real Codex process is spawned in Phase A."
                .to_string(),
            timestamp,
            base_warnings.clone(),
        );
        let final_status = codex_attempt.status.clone();
        let mut warnings = merge_warnings(base_warnings.clone(), codex_attempt.warnings.clone());
        warnings.push(format!(
            "codex_local_attempt_ref:{}",
            codex_attempt.attempt_id
        ));
        warnings.sort();
        warnings.dedup();
        let attempt = phase_a_store_attempt(
            &continuation,
            &attempt_id,
            &codex_attempt.runner_kind,
            "h2_phase_a_runner_path_no_real_codex",
            &final_status,
            timestamp,
            input.authorization.timeout_ms,
            &codex_attempt.readback_result.status,
            codex_attempt.readback_result.result_count,
            codex_attempt
                .failure_reason
                .as_ref()
                .map(|failure| failure.message.clone()),
            vec![started_audit_ref.clone(), completed_audit_ref.clone()],
            warnings.clone(),
        );
        let completed_event = phase_a_audit_event(
            &continuation,
            &completed_audit_ref,
            "session_continuation_h2_phase_a_completed",
            &attempt_id,
            input.actor_role.trim(),
            Some("running_h2_phase_a".to_string()),
            &final_status,
            store.revision + 1,
            format!(
                "H2.5 Phase A runner path completed as {final_status}; prompt_sent=false, real_codex_executed=false, writes_codex_home=false."
            ),
            timestamp,
            warnings,
        );
        (
            attempt,
            vec![started_event, completed_event],
            "phase_a_runner_path_recorded_no_real_execution".to_string(),
            Some(codex_attempt),
        )
    };

    continuation.status = attempt.status.clone();
    continuation.execution_level = attempt.execution_level.clone();
    continuation.runner_kind = attempt.runner_kind.clone();
    continuation.updated_at = timestamp.to_string();
    continuation.audit_refs.extend(attempt.audit_refs.clone());
    continuation.audit_refs.sort();
    continuation.audit_refs.dedup();
    continuation.warnings = merge_warnings(continuation.warnings, attempt.warnings.clone());
    store.continuations[index] = continuation.clone();
    store.attempts.push(attempt.clone());
    store.audit_events.extend(audit_events.clone());
    store.revision += 1;
    store.last_write_id = Some(write_id.to_string());
    store.updated_at = timestamp.to_string();
    remember_project_root(&mut store, &continuation.project_root);
    write_store_atomic(&sidecar, &store, timestamp, write_id)?;
    runtime_log_store::append_session_continuation_attempt(
        workflow_state_path,
        &store,
        &continuation,
        &attempt,
        timestamp,
        write_id,
    )?;
    drop(lock);

    Ok(RunControlledSessionContinuationRealResumePhaseAOutput {
        continuation,
        attempt,
        audit_events,
        store_revision: store.revision,
        authorization_status,
        missing_or_invalid_items: missing,
        codex_local_request,
        codex_local_guard,
        codex_local_attempt,
        warnings: h2_phase_a_warnings(authorized),
    })
}

pub(crate) fn run_real_resume_phase_b_with_runner<
    R: codex_local_runner::CodexLocalPhaseBProcessRunner,
>(
    workflow_state_path: &Path,
    input: &RunControlledSessionContinuationRealResumePhaseBInput,
    timestamp: &str,
    write_id: &str,
    last_message_path: &Path,
    runner: &R,
) -> Result<RunControlledSessionContinuationRealResumePhaseBOutput, String> {
    validate_real_resume_phase_b_input(input)?;
    let sidecar = sidecar_path(workflow_state_path)?;
    let parent = sidecar
        .parent()
        .ok_or_else(|| format!("continuation sidecar 没有父目录：{}", sidecar.display()))?;
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "创建 continuation sidecar 目录失败 {}：{error}",
            parent.display()
        )
    })?;
    let lock_path = parent.join(LOCK_NAME);
    let lock = StoreLock::acquire(&lock_path, write_id)?;
    runtime_log_store::ensure_appendable(workflow_state_path)?;
    let mut store = load_store(workflow_state_path, timestamp)?;
    if let Some(expected) = input.expected_store_revision {
        if expected != store.revision {
            drop(lock);
            return Err(format!(
                "session_continuation_store_conflict: expected revision {expected}, actual {}",
                store.revision
            ));
        }
    }

    let index = store
        .continuations
        .iter()
        .position(|record| record.continuation_id == input.continuation_id)
        .ok_or_else(|| "未找到 continuation，已拒绝 H2 Phase B real resume".to_string())?;
    let before_status = store.continuations[index].status.clone();
    if store.continuations[index].adapter_id != "codex-local" {
        drop(lock);
        return Err("planned adapter 不允许进入 H2 Phase B real resume".to_string());
    }
    if store.continuations[index].operation_id != "resume" {
        drop(lock);
        return Err("H2 Phase B 只允许 resume，不处理 send_message".to_string());
    }

    let mut continuation = store.continuations[index].clone();
    let user_rejected = input
        .execution_decision
        .as_deref()
        .unwrap_or("approved_for_phase_b")
        == "rejected";
    let mut missing = inspect_authorization_matrix(&continuation, &input.authorization);
    if user_rejected {
        missing.push("user_rejected_real_resume".to_string());
    }
    if has_active_attempt_in_h4_scope(&store, &continuation) {
        missing.push("duplicate_running_attempt".to_string());
    }
    if sha256_hex(&input.prompt_body) != input.authorization.prompt_sha256 {
        missing.push("prompt_body_hash_mismatch".to_string());
    }
    missing.sort();
    missing.dedup();

    let matrix_authorized = missing.is_empty();
    let codex_local_request = if matrix_authorized {
        Some(build_codex_local_request_for_h2(
            &continuation,
            &input.authorization,
            &store,
        ))
    } else {
        None
    };
    let codex_local_guard = codex_local_request
        .as_ref()
        .map(codex_local_runner::inspect_codex_local_execution_guard);
    if let Some(guard) = &codex_local_guard {
        if guard.blocks_execution {
            missing.push("codex_local_guard_blocked".to_string());
            missing.extend(
                guard
                    .reasons
                    .iter()
                    .map(|reason| format!("codex_local_guard:{reason}")),
            );
            missing.sort();
            missing.dedup();
        }
    }
    let duplicate_blocked = missing
        .iter()
        .any(|item| item == "duplicate_running_attempt");
    let guard_blocked = missing
        .iter()
        .any(|item| item == "codex_local_guard_blocked");
    let diagnostics_blocked = missing
        .iter()
        .any(|item| item.contains("diagnostics_blocking_degraded"));
    let stale_memory_blocked = missing
        .iter()
        .any(|item| item.contains("task_memory_packet_stale"));
    let product_gate = real_execution_command::decide_real_execution_command(
        real_execution_command::RealExecutionCommandGateInput {
            command_name: "run_controlled_session_continuation_real_resume_phase_b",
            command_family: "controlled_session_continuation",
            operation_id: "resume",
            h5_unified_product_command: true,
            authorization_complete: matrix_authorized,
            user_rejected,
            duplicate_blocked,
            guard_blocked,
            diagnostics_blocked,
            stale_memory_blocked,
            readback_required: continuation.readback_strategy == "required"
                && !input.authorization.readback_plan.trim().is_empty(),
        },
    );
    let authorized = product_gate.runner_call_allowed;
    if !authorized {
        missing.push(format!("product_gate:{}", product_gate.status));
        missing.sort();
        missing.dedup();
    }

    let started_audit_ref = format!(
        "audit:session-continuation-h2-phase-b-started:{}:{}",
        timestamp,
        short_hash(&continuation.continuation_id)
    );
    let completed_audit_ref = format!(
        "audit:session-continuation-h2-phase-b-completed:{}:{}",
        timestamp,
        short_hash(&continuation.continuation_id)
    );
    let blocked_audit_ref = format!(
        "audit:session-continuation-h2-phase-b-blocked:{}:{}",
        timestamp,
        short_hash(&continuation.continuation_id)
    );
    let base_warnings = merge_warnings(
        h2_phase_b_warnings(authorized),
        product_gate.warnings.clone(),
    );

    let (attempt, audit_events, authorization_status, codex_local_attempt) = if !authorized {
        let status = product_gate.status.as_str();
        let attempt_id = format!(
            "session-continuation-attempt:h2-phase-b-blocked:{}:{}",
            timestamp,
            short_hash(&continuation.continuation_id)
        );
        let attempt = h2_store_attempt(
            &continuation,
            &attempt_id,
            "codex_local_phase_b_runner_path",
            "h2_phase_b_real_codex_resume",
            status,
            timestamp,
            input.authorization.timeout_ms,
            status,
            None,
            false,
            false,
            false,
            Some(format!(
                "H2 Phase B blocked by unified product gate: {}; {}",
                product_gate.reason,
                missing.join(",")
            )),
            vec![blocked_audit_ref.clone()],
            base_warnings.clone(),
        );
        let audit_event = h2_audit_event(
            &continuation,
            &blocked_audit_ref,
            "session_continuation_h2_phase_b_blocked",
            &attempt_id,
            input.actor_role.trim(),
            Some(before_status.clone()),
            status,
            store.revision + 1,
            format!(
                "H2 Phase B blocked before real runner by unified product gate: {}; {}",
                product_gate.reason,
                missing.join(",")
            ),
            timestamp,
            base_warnings.clone(),
        );
        (attempt, vec![audit_event], status.to_string(), None)
    } else {
        let request = codex_local_request
            .clone()
            .ok_or_else(|| "H2 Phase B 缺少 codex-local request".to_string())?;
        let attempt_id = format!(
            "session-continuation-attempt:h2-phase-b:{}:{}",
            timestamp,
            short_hash(&continuation.continuation_id)
        );
        let started_event = h2_audit_event(
            &continuation,
            &started_audit_ref,
            "session_continuation_h2_phase_b_started",
            &attempt_id,
            input.actor_role.trim(),
            Some(before_status.clone()),
            "running_h2_phase_b",
            store.revision + 1,
            "H2 Phase B real codex resume started; prompt body is sent via stdin and is not persisted."
                .to_string(),
            timestamp,
            base_warnings.clone(),
        );
        let codex_attempt = codex_local_runner::run_h2_phase_b_with_runner(
            request,
            timestamp,
            &input.prompt_body,
            last_message_path,
            runner,
        );
        let final_status = codex_attempt.status.clone();
        let mut warnings = merge_warnings(base_warnings.clone(), codex_attempt.warnings.clone());
        warnings.push(format!(
            "codex_local_attempt_ref:{}",
            codex_attempt.attempt_id
        ));
        warnings.sort();
        warnings.dedup();
        let attempt = h2_store_attempt(
            &continuation,
            &attempt_id,
            &codex_attempt.runner_kind,
            "h2_phase_b_real_codex_resume",
            &final_status,
            timestamp,
            input.authorization.timeout_ms,
            &codex_attempt.readback_result.status,
            codex_attempt.readback_result.result_count,
            codex_attempt.prompt_sent,
            codex_attempt.real_codex_executed,
            codex_attempt.writes_codex_home,
            codex_attempt
                .failure_reason
                .as_ref()
                .map(|failure| failure.message.clone()),
            vec![started_audit_ref.clone(), completed_audit_ref.clone()],
            warnings.clone(),
        );
        let completed_event = h2_audit_event(
            &continuation,
            &completed_audit_ref,
            "session_continuation_h2_phase_b_completed",
            &attempt_id,
            input.actor_role.trim(),
            Some("running_h2_phase_b".to_string()),
            &final_status,
            store.revision + 1,
            format!(
                "H2 Phase B real runner completed as {final_status}; prompt_sent={}, real_codex_executed={}, writes_codex_home={}.",
                codex_attempt.prompt_sent, codex_attempt.real_codex_executed, codex_attempt.writes_codex_home
            ),
            timestamp,
            warnings,
        );
        (
            attempt,
            vec![started_event, completed_event],
            "phase_b_real_resume_executed".to_string(),
            Some(codex_attempt),
        )
    };

    continuation.status = attempt.status.clone();
    continuation.execution_level = attempt.execution_level.clone();
    continuation.runner_kind = attempt.runner_kind.clone();
    continuation.updated_at = timestamp.to_string();
    continuation.audit_refs.extend(attempt.audit_refs.clone());
    continuation.audit_refs.sort();
    continuation.audit_refs.dedup();
    continuation.warnings =
        merge_phase_b_current_warnings(continuation.warnings, attempt.warnings.clone());
    store.continuations[index] = continuation.clone();
    store.attempts.push(attempt.clone());
    store.audit_events.extend(audit_events.clone());
    store.revision += 1;
    store.last_write_id = Some(write_id.to_string());
    store.updated_at = timestamp.to_string();
    remember_project_root(&mut store, &continuation.project_root);
    write_store_atomic(&sidecar, &store, timestamp, write_id)?;
    runtime_log_store::append_session_continuation_attempt(
        workflow_state_path,
        &store,
        &continuation,
        &attempt,
        timestamp,
        write_id,
    )?;
    drop(lock);

    Ok(RunControlledSessionContinuationRealResumePhaseBOutput {
        continuation,
        attempt,
        audit_events,
        store_revision: store.revision,
        authorization_status,
        missing_or_invalid_items: missing,
        codex_local_request,
        codex_local_guard,
        codex_local_attempt,
        warnings: base_warnings,
    })
}

pub(crate) fn run_real_new_session_h3_b_with_runner<
    R: codex_local_runner::CodexLocalPhaseBProcessRunner,
>(
    workflow_state_path: &Path,
    input: &RunControlledSessionContinuationRealNewSessionH3BInput,
    timestamp: &str,
    write_id: &str,
    last_message_path: &Path,
    runner: &R,
) -> Result<RunControlledSessionContinuationRealNewSessionH3BOutput, String> {
    validate_real_new_session_h3_b_input(input)?;
    let sidecar = sidecar_path(workflow_state_path)?;
    let parent = sidecar
        .parent()
        .ok_or_else(|| format!("continuation sidecar 没有父目录：{}", sidecar.display()))?;
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "创建 continuation sidecar 目录失败 {}：{error}",
            parent.display()
        )
    })?;
    let lock_path = parent.join(LOCK_NAME);
    let lock = StoreLock::acquire(&lock_path, write_id)?;
    runtime_log_store::ensure_appendable(workflow_state_path)?;
    let mut store = load_store(workflow_state_path, timestamp)?;
    if let Some(expected) = input.expected_store_revision {
        if expected != store.revision {
            drop(lock);
            return Err(format!(
                "session_continuation_store_conflict: expected revision {expected}, actual {}",
                store.revision
            ));
        }
    }

    let index = store
        .continuations
        .iter()
        .position(|record| record.continuation_id == input.continuation_id)
        .ok_or_else(|| "未找到 continuation，已拒绝 H3-B real new session".to_string())?;
    let before_status = store.continuations[index].status.clone();
    if store.continuations[index].adapter_id != "codex-local" {
        drop(lock);
        return Err("planned adapter 不允许进入 H3-B real new session".to_string());
    }
    if store.continuations[index].operation_id != "new_session" {
        drop(lock);
        return Err("H3-B 只允许 new_session，不处理 resume / send_message".to_string());
    }

    let mut continuation = store.continuations[index].clone();
    let user_rejected = input
        .execution_decision
        .as_deref()
        .unwrap_or("approved_for_h3_b")
        == "rejected";
    let mut missing = inspect_h3_b_authorization_matrix(&continuation, &input.authorization);
    if user_rejected {
        missing.push("user_rejected_real_new_session".to_string());
    }
    if has_active_attempt_in_h4_scope(&store, &continuation) {
        missing.push("duplicate_running_attempt".to_string());
    }
    if sha256_hex(&input.prompt_body) != input.authorization.prompt_sha256 {
        missing.push("prompt_body_hash_mismatch".to_string());
    }
    missing.sort();
    missing.dedup();

    let matrix_authorized = missing.is_empty();
    let codex_local_request = if matrix_authorized {
        Some(build_codex_local_request_for_h3_b(
            &continuation,
            &input.authorization,
            &store,
        ))
    } else {
        None
    };
    let codex_local_guard = codex_local_request
        .as_ref()
        .map(codex_local_runner::inspect_codex_local_execution_guard);
    if let Some(guard) = &codex_local_guard {
        if guard.blocks_execution {
            missing.push("codex_local_guard_blocked".to_string());
            missing.extend(
                guard
                    .reasons
                    .iter()
                    .map(|reason| format!("codex_local_guard:{reason}")),
            );
            missing.sort();
            missing.dedup();
        }
    }
    let duplicate_blocked = missing
        .iter()
        .any(|item| item == "duplicate_running_attempt");
    let guard_blocked = missing
        .iter()
        .any(|item| item == "codex_local_guard_blocked");
    let diagnostics_blocked = missing
        .iter()
        .any(|item| item.contains("diagnostics_blocking_degraded"));
    let stale_memory_blocked = missing
        .iter()
        .any(|item| item.contains("task_memory_packet_stale"));
    let product_gate = real_execution_command::decide_real_execution_command(
        real_execution_command::RealExecutionCommandGateInput {
            command_name: "run_controlled_session_continuation_real_new_session_h3_b",
            command_family: "controlled_session_continuation",
            operation_id: "new_session",
            h5_unified_product_command: true,
            authorization_complete: matrix_authorized,
            user_rejected,
            duplicate_blocked,
            guard_blocked,
            diagnostics_blocked,
            stale_memory_blocked,
            readback_required: continuation.readback_strategy == "required"
                && !input.authorization.readback_plan.trim().is_empty(),
        },
    );
    let authorized = product_gate.runner_call_allowed;
    if !authorized {
        missing.push(format!("product_gate:{}", product_gate.status));
        missing.sort();
        missing.dedup();
    }

    let started_audit_ref = format!(
        "audit:session-continuation-h3-b-started:{}:{}",
        timestamp,
        short_hash(&continuation.continuation_id)
    );
    let completed_audit_ref = format!(
        "audit:session-continuation-h3-b-completed:{}:{}",
        timestamp,
        short_hash(&continuation.continuation_id)
    );
    let blocked_audit_ref = format!(
        "audit:session-continuation-h3-b-blocked:{}:{}",
        timestamp,
        short_hash(&continuation.continuation_id)
    );
    let base_warnings = merge_warnings(h3_b_warnings(authorized), product_gate.warnings.clone());

    let (attempt, audit_events, authorization_status, codex_local_attempt) = if !authorized {
        let status = product_gate.status.as_str();
        let attempt_id = format!(
            "session-continuation-attempt:h3-b-blocked:{}:{}",
            timestamp,
            short_hash(&continuation.continuation_id)
        );
        let attempt = h3_b_store_attempt(
            &continuation,
            &attempt_id,
            "codex_local_h3_b_runner_path",
            "h3_b_real_codex_new_session",
            status,
            timestamp,
            input.authorization.timeout_ms,
            status,
            None,
            false,
            false,
            false,
            Some(format!(
                "H3-B blocked by unified product gate: {}; {}",
                product_gate.reason,
                missing.join(",")
            )),
            vec![blocked_audit_ref.clone()],
            base_warnings.clone(),
        );
        let audit_event = h2_audit_event(
            &continuation,
            &blocked_audit_ref,
            "session_continuation_h3_b_blocked",
            &attempt_id,
            input.actor_role.trim(),
            Some(before_status.clone()),
            status,
            store.revision + 1,
            format!(
                "H3-B blocked before real runner by unified product gate: {}; {}",
                product_gate.reason,
                missing.join(",")
            ),
            timestamp,
            base_warnings.clone(),
        );
        (attempt, vec![audit_event], status.to_string(), None)
    } else {
        let request = codex_local_request
            .clone()
            .ok_or_else(|| "H3-B 缺少 codex-local request".to_string())?;
        let attempt_id = format!(
            "session-continuation-attempt:h3-b:{}:{}",
            timestamp,
            short_hash(&continuation.continuation_id)
        );
        let started_event = h2_audit_event(
            &continuation,
            &started_audit_ref,
            "session_continuation_h3_b_started",
            &attempt_id,
            input.actor_role.trim(),
            Some(before_status.clone()),
            "running_h3_b",
            store.revision + 1,
            "H3-B real codex new_session started; prompt body is sent via stdin and is not persisted."
                .to_string(),
            timestamp,
            base_warnings.clone(),
        );
        let codex_attempt = codex_local_runner::run_h2_phase_b_with_runner(
            request,
            timestamp,
            &input.prompt_body,
            last_message_path,
            runner,
        );
        let final_status = codex_attempt.status.clone();
        let mut warnings = merge_warnings(base_warnings.clone(), codex_attempt.warnings.clone());
        warnings.push(format!(
            "codex_local_attempt_ref:{}",
            codex_attempt.attempt_id
        ));
        warnings.sort();
        warnings.dedup();
        let attempt = h3_b_store_attempt(
            &continuation,
            &attempt_id,
            &codex_attempt.runner_kind,
            "h3_b_real_codex_new_session",
            &final_status,
            timestamp,
            input.authorization.timeout_ms,
            &codex_attempt.readback_result.status,
            codex_attempt.readback_result.result_count,
            codex_attempt.prompt_sent,
            codex_attempt.real_codex_executed,
            codex_attempt.writes_codex_home,
            codex_attempt
                .failure_reason
                .as_ref()
                .map(|failure| failure.message.clone()),
            vec![started_audit_ref.clone(), completed_audit_ref.clone()],
            warnings.clone(),
        );
        let completed_event = h2_audit_event(
            &continuation,
            &completed_audit_ref,
            "session_continuation_h3_b_completed",
            &attempt_id,
            input.actor_role.trim(),
            Some("running_h3_b".to_string()),
            &final_status,
            store.revision + 1,
            format!(
                "H3-B real runner completed as {final_status}; prompt_sent={}, real_codex_executed={}, writes_codex_home={}.",
                codex_attempt.prompt_sent, codex_attempt.real_codex_executed, codex_attempt.writes_codex_home
            ),
            timestamp,
            warnings,
        );
        (
            attempt,
            vec![started_event, completed_event],
            "h3_b_real_new_session_executed".to_string(),
            Some(codex_attempt),
        )
    };

    continuation.status = attempt.status.clone();
    continuation.execution_level = attempt.execution_level.clone();
    continuation.runner_kind = attempt.runner_kind.clone();
    continuation.updated_at = timestamp.to_string();
    continuation.audit_refs.extend(attempt.audit_refs.clone());
    continuation.audit_refs.sort();
    continuation.audit_refs.dedup();
    continuation.warnings =
        merge_phase_b_current_warnings(continuation.warnings, attempt.warnings.clone());
    store.continuations[index] = continuation.clone();
    store.attempts.push(attempt.clone());
    store.audit_events.extend(audit_events.clone());
    store.revision += 1;
    store.last_write_id = Some(write_id.to_string());
    store.updated_at = timestamp.to_string();
    remember_project_root(&mut store, &continuation.project_root);
    write_store_atomic(&sidecar, &store, timestamp, write_id)?;
    runtime_log_store::append_session_continuation_attempt(
        workflow_state_path,
        &store,
        &continuation,
        &attempt,
        timestamp,
        write_id,
    )?;
    drop(lock);

    Ok(RunControlledSessionContinuationRealNewSessionH3BOutput {
        continuation,
        attempt,
        audit_events,
        store_revision: store.revision,
        authorization_status,
        missing_or_invalid_items: missing,
        codex_local_request,
        codex_local_guard,
        codex_local_attempt,
        warnings: base_warnings,
    })
}

pub(crate) fn cleanup_stale_attempt(
    workflow_state_path: &Path,
    input: &CleanupSessionContinuationStaleAttemptInput,
    timestamp: &str,
    write_id: &str,
) -> Result<CleanupSessionContinuationStaleAttemptOutput, String> {
    if input.actor_role.trim().is_empty() {
        return Err("stale cleanup 缺少 actor_role".to_string());
    }
    if input.stale_reason.trim().is_empty() {
        return Err("stale cleanup 缺少 stale_reason".to_string());
    }
    let Some(expected_revision) = input.expected_store_revision else {
        return Err("stale cleanup 必须提供 expected_store_revision".to_string());
    };

    let sidecar = sidecar_path(workflow_state_path)?;
    let parent = sidecar
        .parent()
        .ok_or_else(|| format!("continuation sidecar 没有父目录：{}", sidecar.display()))?;
    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "创建 continuation sidecar 目录失败 {}：{error}",
            parent.display()
        )
    })?;
    let lock_path = parent.join(LOCK_NAME);
    let lock = StoreLock::acquire(&lock_path, write_id)?;
    runtime_log_store::ensure_appendable(workflow_state_path)?;
    let mut store = load_store(workflow_state_path, timestamp)?;
    if expected_revision != store.revision {
        drop(lock);
        return Err(format!(
            "session_continuation_store_conflict: expected revision {expected_revision}, actual {}",
            store.revision
        ));
    }

    let attempt_index = store
        .attempts
        .iter()
        .position(|attempt| attempt.attempt_id == input.attempt_id)
        .ok_or_else(|| "未找到 attempt，已拒绝 stale cleanup".to_string())?;
    let before_attempt_status = store.attempts[attempt_index].status.clone();
    if !crate::h4_execution_boundary::is_h4_active_attempt_status(&before_attempt_status) {
        drop(lock);
        return Err(format!(
            "stale cleanup 只处理工作台自有 active attempt，当前状态为 {before_attempt_status}"
        ));
    }
    let continuation_id = store.attempts[attempt_index].continuation_id.clone();
    let continuation_index = store
        .continuations
        .iter()
        .position(|record| record.continuation_id == continuation_id)
        .ok_or_else(|| "未找到 continuation，已拒绝 stale cleanup".to_string())?;
    let mut continuation = store.continuations[continuation_index].clone();
    let audit_ref = format!(
        "audit:session-continuation-stale-cleanup:{}:{}",
        timestamp,
        short_hash(&input.attempt_id)
    );
    let warnings = level_a_warnings(vec![
        "h4_stale_cleanup_workbench_state_only".to_string(),
        "no_codex_kill".to_string(),
        "codex_home_not_touched".to_string(),
        "no_auto_retry".to_string(),
        crate::h4_execution_boundary::h4_unknown_result_warning(),
    ]);
    let mut attempt = store.attempts[attempt_index].clone();
    attempt.status = "stale_cancelled".to_string();
    attempt.finished_at = Some(timestamp.to_string());
    attempt.readback_summary.status = "stale_cancelled".to_string();
    attempt.readback_summary.result_count = None;
    attempt.readback_summary.unavailable_reason = Some(
        "H4 stale cleanup 只取消工作台自有 stale attempt；未 kill Codex，未读写 .codex，结果数未知。"
            .to_string(),
    );
    attempt
        .readback_summary
        .warnings
        .push(crate::h4_execution_boundary::h4_unknown_result_warning());
    attempt.failure_reason = Some(input.stale_reason.trim().to_string());
    attempt.audit_refs.push(audit_ref.clone());
    attempt.audit_refs.sort();
    attempt.audit_refs.dedup();
    attempt.warnings = merge_warnings(attempt.warnings, warnings.clone());

    let audit_event = h2_audit_event(
        &continuation,
        &audit_ref,
        "session_continuation_stale_attempt_cancelled",
        &attempt.attempt_id,
        input.actor_role.trim(),
        Some(before_attempt_status),
        "stale_cancelled",
        store.revision + 1,
        format!(
            "H4 stale cleanup marked workbench-owned attempt stale_cancelled: {}",
            input.stale_reason.trim()
        ),
        timestamp,
        warnings.clone(),
    );

    continuation.status = "stale_cancelled".to_string();
    continuation.updated_at = timestamp.to_string();
    continuation.audit_refs.extend(attempt.audit_refs.clone());
    continuation.audit_refs.sort();
    continuation.audit_refs.dedup();
    continuation.warnings = merge_warnings(continuation.warnings, warnings.clone());
    store.attempts[attempt_index] = attempt.clone();
    store.continuations[continuation_index] = continuation.clone();
    store.audit_events.push(audit_event.clone());
    store.revision += 1;
    store.last_write_id = Some(write_id.to_string());
    store.updated_at = timestamp.to_string();
    write_store_atomic(&sidecar, &store, timestamp, write_id)?;
    runtime_log_store::append_session_continuation_attempt(
        workflow_state_path,
        &store,
        &continuation,
        &attempt,
        timestamp,
        write_id,
    )?;
    drop(lock);

    Ok(CleanupSessionContinuationStaleAttemptOutput {
        continuation,
        attempt,
        audit_event,
        store_revision: store.revision,
        warnings,
    })
}

fn empty_store(
    workflow_state_path: &Path,
    sidecar: &Path,
    timestamp: &str,
    warnings: Vec<String>,
) -> SessionContinuationStoreV1 {
    SessionContinuationStoreV1 {
        schema_version: SCHEMA_VERSION.to_string(),
        store_version: 1,
        storage_kind: STORAGE_KIND.to_string(),
        scope: SessionContinuationStoreScope {
            scope_kind: "workflow_state_sidecar".to_string(),
            workflow_state_path: Some(workflow_state_path.display().to_string()),
            sidecar_path: Some(sidecar.display().to_string()),
            project_roots: vec![],
        },
        revision: 0,
        last_write_id: None,
        generated_by: "control_core".to_string(),
        created_at: timestamp.to_string(),
        updated_at: timestamp.to_string(),
        continuations: vec![],
        attempts: vec![],
        audit_events: vec![],
        warnings,
    }
}

fn validate_store(store: &SessionContinuationStoreV1) -> Result<(), String> {
    if store.schema_version != SCHEMA_VERSION {
        return Err(format!(
            "continuation schema_version 不匹配：{}",
            store.schema_version
        ));
    }
    if store.store_version != 1 {
        return Err(format!(
            "continuation store_version 不匹配：{}",
            store.store_version
        ));
    }
    if store.storage_kind != STORAGE_KIND {
        return Err(format!(
            "continuation storage_kind 不匹配：{}",
            store.storage_kind
        ));
    }
    if store.revision < 0 {
        return Err("continuation revision 不能小于 0".to_string());
    }
    Ok(())
}

fn validate_confirm_input(input: &ConfirmControlledSessionContinuationInput) -> Result<(), String> {
    let preview = &input.preview;
    if input.confirmed_by.trim().is_empty() {
        return Err("controlled continuation 缺少 confirmed_by".to_string());
    }
    if input.confirmation_reason.trim().is_empty() {
        return Err("controlled continuation 缺少 confirmation_reason".to_string());
    }
    if preview.adapter_id != "codex-local" || preview.request.adapter_id != "codex-local" {
        return Err("E5 Level A 只允许 codex-local continuation".to_string());
    }
    if !matches!(
        preview.operation_id.as_str(),
        "new_session" | "send_message" | "resume"
    ) {
        return Err(
            "controlled continuation 只允许 new_session / send_message / resume".to_string(),
        );
    }
    if matches!(
        preview.guard_result.status.as_str(),
        "blocked" | "requires_future_task"
    ) {
        return Err(format!(
            "guard blocked preview 不能创建 runnable continuation：{}",
            preview.guard_result.status
        ));
    }
    if preview
        .guard_result
        .reasons
        .iter()
        .any(|reason| reason.contains("planned_adapter_blocked"))
    {
        return Err("planned adapter preview 不能进入 E5 continuation".to_string());
    }
    for (name, value) in [
        ("project_id", preview.project_id.as_deref()),
        ("project_root", preview.project_root.as_deref()),
        ("workflow_id", preview.workflow_id.as_deref()),
        ("node_id", preview.node_id.as_deref()),
        ("target_cwd", preview.target_cwd.as_deref()),
    ] {
        if value.unwrap_or("").trim().is_empty() {
            return Err(format!("controlled continuation 缺少 {name}"));
        }
    }
    if preview.operation_id == "new_session" {
        if preview
            .work_item_id
            .as_deref()
            .unwrap_or("")
            .trim()
            .is_empty()
        {
            return Err("controlled continuation new_session 缺少 work_item_id".to_string());
        }
    } else if preview
        .target_session_id
        .as_deref()
        .unwrap_or("")
        .trim()
        .is_empty()
    {
        return Err("controlled continuation 缺少 target_session_id".to_string());
    }
    if preview.allowed_write_roots_summary.is_empty() && preview.sandbox_summary != "read-only" {
        return Err("controlled continuation 缺少 allowed_write_roots".to_string());
    }
    if preview.prompt_summary.trim().is_empty() {
        return Err("controlled continuation 缺少 prompt_summary".to_string());
    }
    if preview.readback_expectation.strategy != "required"
        || preview.request.readback_strategy != "required"
    {
        return Err("controlled continuation 必须有 required readback strategy".to_string());
    }
    if contains_sensitive_fragment(preview.target_cwd.as_deref().unwrap_or_default())
        || preview
            .allowed_write_roots_summary
            .iter()
            .any(|root| contains_sensitive_fragment(root))
    {
        return Err("controlled continuation 命中敏感路径，已拒绝".to_string());
    }
    let target_cwd = preview.target_cwd.as_deref().unwrap_or_default();
    let project_root = preview.project_root.as_deref().unwrap_or_default();
    if !path_within_scope(target_cwd, project_root)
        && !preview
            .allowed_write_roots_summary
            .iter()
            .any(|root| path_within_scope(target_cwd, root))
    {
        return Err("controlled continuation target_cwd 越界，已拒绝".to_string());
    }
    Ok(())
}

fn validate_stub_input(input: &RunControlledSessionContinuationStubInput) -> Result<(), String> {
    if input.continuation_id.trim().is_empty() {
        return Err("stub attempt 缺少 continuation_id".to_string());
    }
    if input.actor_role.trim().is_empty() {
        return Err("stub attempt 缺少 actor_role".to_string());
    }
    Ok(())
}

fn validate_real_resume_input(
    input: &InspectControlledSessionContinuationRealResumeInput,
) -> Result<(), String> {
    if input.continuation_id.trim().is_empty() {
        return Err("H2 real resume 预检缺少 continuation_id".to_string());
    }
    if input.actor_role.trim().is_empty() {
        return Err("H2 real resume 预检缺少 actor_role".to_string());
    }
    Ok(())
}

fn validate_real_resume_phase_a_input(
    input: &RunControlledSessionContinuationRealResumePhaseAInput,
) -> Result<(), String> {
    if input.continuation_id.trim().is_empty() {
        return Err("H2.5 Phase A 缺少 continuation_id".to_string());
    }
    if input.actor_role.trim().is_empty() {
        return Err("H2.5 Phase A 缺少 actor_role".to_string());
    }
    if let Some(decision) = &input.execution_decision {
        if !matches!(decision.as_str(), "approved_for_phase_a" | "rejected") {
            return Err(
                "H2.5 Phase A execution_decision 只能是 approved_for_phase_a 或 rejected"
                    .to_string(),
            );
        }
    }
    Ok(())
}

fn validate_real_resume_phase_b_input(
    input: &RunControlledSessionContinuationRealResumePhaseBInput,
) -> Result<(), String> {
    if input.continuation_id.trim().is_empty() {
        return Err("H2 Phase B 缺少 continuation_id".to_string());
    }
    if input.actor_role.trim().is_empty() {
        return Err("H2 Phase B 缺少 actor_role".to_string());
    }
    if input.prompt_body.trim().is_empty() {
        return Err("H2 Phase B 缺少 prompt_body".to_string());
    }
    if let Some(decision) = &input.execution_decision {
        if !matches!(decision.as_str(), "approved_for_phase_b" | "rejected") {
            return Err(
                "H2 Phase B execution_decision 只能是 approved_for_phase_b 或 rejected".to_string(),
            );
        }
    }
    Ok(())
}

fn validate_real_new_session_h3_b_input(
    input: &RunControlledSessionContinuationRealNewSessionH3BInput,
) -> Result<(), String> {
    if input.continuation_id.trim().is_empty() {
        return Err("H3-B 缺少 continuation_id".to_string());
    }
    if input.actor_role.trim().is_empty() {
        return Err("H3-B 缺少 actor_role".to_string());
    }
    if input.prompt_body.trim().is_empty() {
        return Err("H3-B 缺少 prompt_body".to_string());
    }
    if let Some(decision) = &input.execution_decision {
        if !matches!(decision.as_str(), "approved_for_h3_b" | "rejected") {
            return Err("H3-B execution_decision 只能是 approved_for_h3_b 或 rejected".to_string());
        }
    }
    Ok(())
}

fn continuation_from_preview(
    preview: &SessionContinuationPreview,
    continuation_id: &str,
    timestamp: &str,
    confirmed_by: &str,
    confirmation_reason: &str,
    warnings: Vec<String>,
) -> Result<ControlledSessionContinuation, String> {
    Ok(ControlledSessionContinuation {
        record_version: 1,
        continuation_id: continuation_id.to_string(),
        preview_id: preview.preview_id.clone(),
        adapter_id: preview.adapter_id.clone(),
        operation_id: preview.operation_id.clone(),
        project_id: preview.project_id.clone().unwrap_or_default(),
        project_root: preview.project_root.clone().unwrap_or_default(),
        workflow_id: preview.workflow_id.clone().unwrap_or_default(),
        node_id: preview.node_id.clone().unwrap_or_default(),
        session_id: preview.target_session_id.clone().unwrap_or_default(),
        work_item_id: preview.work_item_id.clone(),
        target_cwd: preview.target_cwd.clone().unwrap_or_default(),
        allowed_write_roots: preview.allowed_write_roots_summary.clone(),
        sandbox: preview.sandbox_summary.clone(),
        prompt_source_kind: preview.prompt_source_kind.clone(),
        prompt_summary: preview.prompt_summary.clone(),
        command_preview: command_preview_for(preview),
        readback_strategy: preview.readback_expectation.strategy.clone(),
        status: "preview_confirmed".to_string(),
        execution_level: "level_a_stub_only".to_string(),
        runner_kind: "stub".to_string(),
        user_confirmation_state: "confirmed".to_string(),
        guard_status: preview.guard_result.status.clone(),
        requested_by: preview.request.requested_by.clone(),
        confirmed_by: confirmed_by.to_string(),
        confirmation_reason: confirmation_reason.to_string(),
        created_at: timestamp.to_string(),
        updated_at: timestamp.to_string(),
        audit_refs: vec![format!(
            "audit:session-continuation-confirmed:{}:{}",
            timestamp,
            short_hash(continuation_id)
        )],
        warnings,
    })
}

fn command_preview_for(preview: &SessionContinuationPreview) -> String {
    if preview.operation_id == "new_session" {
        format!(
            "H3.1 preview only: codex exec --skip-git-repo-check --json --output-last-message <workbench-managed> -C {} --sandbox {} <stdin:workbench-managed-prompt>",
            preview.target_cwd.as_deref().unwrap_or("<missing-cwd>"),
            preview.sandbox_summary
        )
    } else {
        format!(
            "Level A preview only: codex exec resume --skip-git-repo-check --json --output-last-message <workbench-managed> -C {} --sandbox {} {}",
            preview.target_cwd.as_deref().unwrap_or("<missing-cwd>"),
            preview.sandbox_summary,
            preview
                .target_session_id
                .as_deref()
                .unwrap_or("<missing-session>")
        )
    }
}

fn stable_continuation_id(preview: &SessionContinuationPreview) -> String {
    format!(
        "session-continuation:v1:{}",
        sha256_hex(
            &[
                normalize(&preview.preview_id),
                normalize(&preview.adapter_id),
                normalize(&preview.operation_id),
                normalize(preview.project_id.as_deref().unwrap_or_default()),
                normalize(preview.workflow_id.as_deref().unwrap_or_default()),
                normalize(preview.node_id.as_deref().unwrap_or_default()),
                normalize(preview.target_session_id.as_deref().unwrap_or_default()),
                normalize(preview.work_item_id.as_deref().unwrap_or_default()),
            ]
            .join("\0")
        )
    )
}

fn write_store_atomic(
    sidecar: &Path,
    store: &SessionContinuationStoreV1,
    timestamp: &str,
    write_id: &str,
) -> Result<(), String> {
    let parent = sidecar
        .parent()
        .ok_or_else(|| format!("continuation sidecar 没有父目录：{}", sidecar.display()))?;
    if sidecar.exists() {
        let backup_dir = parent.join("backups");
        fs::create_dir_all(&backup_dir).map_err(|error| {
            format!(
                "创建 continuation 备份目录失败 {}：{error}",
                backup_dir.display()
            )
        })?;
        let backup = backup_dir.join(format!(
            "session-continuations.v1.{timestamp}.{}.json",
            store.revision.saturating_sub(1)
        ));
        fs::copy(sidecar, &backup).map_err(|error| {
            format!(
                "备份 continuation sidecar 失败 {}：{error}",
                backup.display()
            )
        })?;
        prune_backups(&backup_dir, "session-continuations.v1.")?;
    }
    let temp_path = parent.join(format!(
        ".session-continuations.v1.{timestamp}.{write_id}.tmp"
    ));
    let text = serde_json::to_string_pretty(store)
        .map_err(|error| format!("continuation sidecar 序列化失败：{error}"))?;
    {
        let mut file = fs::File::create(&temp_path).map_err(|error| {
            format!(
                "创建 continuation 临时文件失败 {}：{error}",
                temp_path.display()
            )
        })?;
        file.write_all(text.as_bytes()).map_err(|error| {
            format!(
                "写入 continuation 临时文件失败 {}：{error}",
                temp_path.display()
            )
        })?;
        file.sync_all().map_err(|error| {
            format!(
                "同步 continuation 临时文件失败 {}：{error}",
                temp_path.display()
            )
        })?;
    }
    fs::rename(&temp_path, sidecar).map_err(|error| {
        format!(
            "原子替换 continuation sidecar 失败 {}：{error}",
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
                "读取 continuation 备份目录失败 {}：{error}",
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

fn remember_project_root(store: &mut SessionContinuationStoreV1, project_root: &str) {
    if !project_root.trim().is_empty()
        && !store
            .scope
            .project_roots
            .iter()
            .any(|root| root == project_root)
    {
        store.scope.project_roots.push(project_root.to_string());
        store.scope.project_roots.sort();
    }
}

fn level_a_warnings(mut warnings: Vec<String>) -> Vec<String> {
    warnings.extend([
        "level_a_stub_only".to_string(),
        "prompt_sent_false".to_string(),
        "real_codex_executed_false".to_string(),
        "writes_codex_home_false".to_string(),
    ]);
    warnings.sort();
    warnings.dedup();
    warnings
}

fn h2_preflight_warnings(authorized: bool) -> Vec<String> {
    let mut warnings = vec![
        "h2_real_resume_preflight_only".to_string(),
        "prompt_sent_false".to_string(),
        "real_codex_executed_false".to_string(),
        "writes_codex_home_false".to_string(),
        "readback_unavailable_is_not_zero_results".to_string(),
    ];
    if authorized {
        warnings.push("authorization_matrix_complete_but_runner_not_called".to_string());
    } else {
        warnings.push("blocked_before_real_codex_resume".to_string());
    }
    warnings.sort();
    warnings.dedup();
    warnings
}

fn h2_phase_a_warnings(authorized: bool) -> Vec<String> {
    let mut warnings = vec![
        "h2_phase_a_runner_path_no_real_codex".to_string(),
        "prompt_sent_false".to_string(),
        "real_codex_executed_false".to_string(),
        "writes_codex_home_false".to_string(),
        "writes_workbench_state_true".to_string(),
        "readback_unavailable_or_failed_is_not_zero_results".to_string(),
    ];
    if authorized {
        warnings.push("phase_a_uses_replaceable_process_runner".to_string());
    } else {
        warnings.push("phase_a_blocked_before_runner_path".to_string());
    }
    warnings.sort();
    warnings.dedup();
    warnings
}

fn h2_phase_b_warnings(authorized: bool) -> Vec<String> {
    let mut warnings = vec![
        "h2_phase_b_real_runner_path".to_string(),
        "prompt_body_sent_via_stdin_not_persisted".to_string(),
        "writes_workbench_state_true".to_string(),
        "readback_unavailable_or_failed_is_not_zero_results".to_string(),
    ];
    if authorized {
        warnings.push("phase_b_real_codex_resume_authorized".to_string());
    } else {
        warnings.push("phase_b_blocked_before_real_runner".to_string());
    }
    warnings.sort();
    warnings.dedup();
    warnings
}

fn h3_b_warnings(authorized: bool) -> Vec<String> {
    let mut warnings = vec![
        "h3_b_real_new_session_runner_path".to_string(),
        "prompt_body_sent_via_stdin_not_persisted".to_string(),
        "writes_workbench_state_true".to_string(),
        "readback_unavailable_or_failed_is_not_zero_results".to_string(),
    ];
    if authorized {
        warnings.push("h3_b_real_codex_new_session_authorized".to_string());
    } else {
        warnings.push("h3_b_blocked_before_real_runner".to_string());
    }
    warnings.sort();
    warnings.dedup();
    warnings
}

fn build_codex_local_request_for_h2(
    continuation: &ControlledSessionContinuation,
    auth: &H2RealResumeAuthorizationMatrix,
    store: &SessionContinuationStoreV1,
) -> CodexLocalExecutionRequest {
    CodexLocalExecutionRequest {
        request_version: 1,
        adapter_id: continuation.adapter_id.clone(),
        operation_id: continuation.operation_id.clone(),
        project_id: continuation.project_id.clone(),
        project_root: auth.project_root.clone(),
        workflow_id: continuation.workflow_id.clone(),
        node_id: continuation.node_id.clone(),
        session_id: Some(auth.target_session.clone()),
        work_item_id: continuation.work_item_id.clone(),
        continuation_id: Some(continuation.continuation_id.clone()),
        target_cwd: auth.target_cwd.clone(),
        allowed_write_roots: auth.allowed_write_roots.clone(),
        sandbox: auth.sandbox.clone(),
        prompt_source_kind: continuation.prompt_source_kind.clone(),
        prompt_summary: auth.prompt_summary.clone(),
        prompt_sha256: auth.prompt_sha256.clone(),
        prompt_ref: auth.prompt_ref.clone(),
        readback_plan: CodexLocalReadbackPlan {
            strategy: "required".to_string(),
            required: true,
            expected_sources: vec![
                "workbench_managed_last_message".to_string(),
                "session_continuation_attempt".to_string(),
                "runtime_log_ref".to_string(),
            ],
            unavailable_behavior: auth.readback_plan.clone(),
            trust_policy: "must_be_explicit_readback_result_not_raw_transcript".to_string(),
            warnings: vec!["readback_unavailable_is_not_zero_results".to_string()],
        },
        requested_by: "h2_real_resume_preflight".to_string(),
        user_confirmation_state: "confirmed".to_string(),
        authorization_scope_id: Some(format!(
            "h2-real-resume-authorization:{}",
            short_hash(&continuation.continuation_id)
        )),
        runtime_log_refs: vec![CodexLocalRuntimeLogRef {
            ref_id: format!(
                "runtime-log:codex-local:h2-real-resume:{}",
                short_hash(&continuation.continuation_id)
            ),
            category: "dispatch_attempt".to_string(),
            status: "authorization_matrix_complete".to_string(),
            redaction_status: "redacted_safe_summary".to_string(),
        }],
        audit_refs: continuation
            .audit_refs
            .iter()
            .map(|ref_id| CodexLocalAuditRef {
                ref_id: ref_id.clone(),
                event_type: "session_continuation_authorization_ref".to_string(),
                actor_role: "user_or_global_supervisor".to_string(),
                decision: "referenced_for_h2_preflight".to_string(),
            })
            .collect(),
        active_attempts: active_attempts_in_h4_scope(store, continuation),
        warnings: vec![
            "h2_request_builder_only".to_string(),
            "phase_b_runner_requires_explicit_call".to_string(),
            format!("timeout_ms:{}", auth.timeout_ms.unwrap_or(120_000)),
        ],
    }
}

fn build_codex_local_request_for_h3_b(
    continuation: &ControlledSessionContinuation,
    auth: &H3RealNewSessionAuthorizationMatrix,
    store: &SessionContinuationStoreV1,
) -> CodexLocalExecutionRequest {
    CodexLocalExecutionRequest {
        request_version: 1,
        adapter_id: continuation.adapter_id.clone(),
        operation_id: "new_session".to_string(),
        project_id: continuation.project_id.clone(),
        project_root: auth.project_root.clone(),
        workflow_id: continuation.workflow_id.clone(),
        node_id: continuation.node_id.clone(),
        session_id: None,
        work_item_id: continuation.work_item_id.clone(),
        continuation_id: Some(continuation.continuation_id.clone()),
        target_cwd: auth.target_cwd.clone(),
        allowed_write_roots: auth.allowed_write_roots.clone(),
        sandbox: auth.sandbox.clone(),
        prompt_source_kind: continuation.prompt_source_kind.clone(),
        prompt_summary: auth.prompt_summary.clone(),
        prompt_sha256: auth.prompt_sha256.clone(),
        prompt_ref: auth.prompt_ref.clone(),
        readback_plan: CodexLocalReadbackPlan {
            strategy: "required".to_string(),
            required: true,
            expected_sources: vec![
                "workbench_managed_last_message".to_string(),
                "session_continuation_attempt".to_string(),
                "runtime_log_ref".to_string(),
            ],
            unavailable_behavior: auth.readback_plan.clone(),
            trust_policy: "must_be_explicit_readback_result_not_raw_transcript".to_string(),
            warnings: vec!["readback_unavailable_is_not_zero_results".to_string()],
        },
        requested_by: "h3_b_real_new_session_fixture_run".to_string(),
        user_confirmation_state: "confirmed".to_string(),
        authorization_scope_id: Some(format!(
            "h3-b-real-new-session-authorization:{}",
            short_hash(&continuation.continuation_id)
        )),
        runtime_log_refs: vec![CodexLocalRuntimeLogRef {
            ref_id: format!(
                "runtime-log:codex-local:h3-b-real-new-session:{}",
                short_hash(&continuation.continuation_id)
            ),
            category: "dispatch_attempt".to_string(),
            status: "authorization_matrix_complete".to_string(),
            redaction_status: "redacted_safe_summary".to_string(),
        }],
        audit_refs: continuation
            .audit_refs
            .iter()
            .map(|ref_id| CodexLocalAuditRef {
                ref_id: ref_id.clone(),
                event_type: "session_continuation_authorization_ref".to_string(),
                actor_role: "user_or_global_supervisor".to_string(),
                decision: "referenced_for_h3_b_real_new_session".to_string(),
            })
            .collect(),
        active_attempts: active_attempts_in_h4_scope(store, continuation),
        warnings: vec![
            "h3_b_request_builder".to_string(),
            "real_new_session_runner_requires_explicit_call".to_string(),
            format!("timeout_ms:{}", auth.timeout_ms.unwrap_or(120_000)),
        ],
    }
}

fn phase_a_store_attempt(
    continuation: &ControlledSessionContinuation,
    attempt_id: &str,
    runner_kind: &str,
    execution_level: &str,
    status: &str,
    timestamp: &str,
    timeout_ms: Option<i64>,
    readback_status: &str,
    readback_result_count: Option<i64>,
    failure_reason: Option<String>,
    audit_refs: Vec<String>,
    warnings: Vec<String>,
) -> SessionContinuationAttempt {
    SessionContinuationAttempt {
        attempt_version: 1,
        attempt_id: attempt_id.to_string(),
        continuation_id: continuation.continuation_id.clone(),
        runner_kind: runner_kind.to_string(),
        execution_level: execution_level.to_string(),
        status: status.to_string(),
        started_at: timestamp.to_string(),
        finished_at: Some(timestamp.to_string()),
        timeout_ms,
        command_preview: continuation.command_preview.clone(),
        prompt_sent: false,
        real_codex_executed: false,
        writes_codex_home: false,
        writes_workbench_state: true,
        readback_summary: SessionContinuationReadbackSummary {
            status: readback_status.to_string(),
            source_kind: "h2_phase_a_no_raw_transcript_read".to_string(),
            result_count: readback_result_count_for(status, readback_status, readback_result_count),
            unavailable_reason: if readback_result_count_for(
                status,
                readback_status,
                readback_result_count,
            )
            .is_none()
            {
                Some(
                    "H2.5 Phase A 不读取真实 transcript；unavailable/failed/timed_out 不等于 0 条结果。"
                        .to_string(),
                )
            } else {
                None
            },
            warnings: vec![
                "readback_unavailable_or_failed_is_not_zero_results".to_string(),
                crate::h4_execution_boundary::h4_unknown_result_warning(),
                "no_real_transcript_read_in_h2_phase_a".to_string(),
            ],
        },
        failure_reason,
        audit_refs,
        warnings,
    }
}

fn h2_store_attempt(
    continuation: &ControlledSessionContinuation,
    attempt_id: &str,
    runner_kind: &str,
    execution_level: &str,
    status: &str,
    timestamp: &str,
    timeout_ms: Option<i64>,
    readback_status: &str,
    readback_result_count: Option<i64>,
    prompt_sent: bool,
    real_codex_executed: bool,
    writes_codex_home: bool,
    failure_reason: Option<String>,
    audit_refs: Vec<String>,
    warnings: Vec<String>,
) -> SessionContinuationAttempt {
    SessionContinuationAttempt {
        attempt_version: 1,
        attempt_id: attempt_id.to_string(),
        continuation_id: continuation.continuation_id.clone(),
        runner_kind: runner_kind.to_string(),
        execution_level: execution_level.to_string(),
        status: status.to_string(),
        started_at: timestamp.to_string(),
        finished_at: Some(timestamp.to_string()),
        timeout_ms,
        command_preview: redacted_real_resume_command_preview(continuation),
        prompt_sent,
        real_codex_executed,
        writes_codex_home,
        writes_workbench_state: true,
        readback_summary: SessionContinuationReadbackSummary {
            status: readback_status.to_string(),
            source_kind: "h2_phase_b_workbench_managed_last_message".to_string(),
            result_count: readback_result_count_for(status, readback_status, readback_result_count),
            unavailable_reason: if readback_result_count_for(
                status,
                readback_status,
                readback_result_count,
            )
            .is_none()
            {
                Some(
                    "H2 Phase B readback unavailable/failed/timed_out 不等于 0 条结果。"
                        .to_string(),
                )
            } else {
                None
            },
            warnings: vec![
                "readback_unavailable_or_failed_is_not_zero_results".to_string(),
                crate::h4_execution_boundary::h4_unknown_result_warning(),
                "raw_transcript_not_read_in_h2_phase_b".to_string(),
            ],
        },
        failure_reason,
        audit_refs,
        warnings,
    }
}

fn h3_b_store_attempt(
    continuation: &ControlledSessionContinuation,
    attempt_id: &str,
    runner_kind: &str,
    execution_level: &str,
    status: &str,
    timestamp: &str,
    timeout_ms: Option<i64>,
    readback_status: &str,
    readback_result_count: Option<i64>,
    prompt_sent: bool,
    real_codex_executed: bool,
    writes_codex_home: bool,
    failure_reason: Option<String>,
    audit_refs: Vec<String>,
    warnings: Vec<String>,
) -> SessionContinuationAttempt {
    SessionContinuationAttempt {
        attempt_version: 1,
        attempt_id: attempt_id.to_string(),
        continuation_id: continuation.continuation_id.clone(),
        runner_kind: runner_kind.to_string(),
        execution_level: execution_level.to_string(),
        status: status.to_string(),
        started_at: timestamp.to_string(),
        finished_at: Some(timestamp.to_string()),
        timeout_ms,
        command_preview: continuation.command_preview.clone(),
        prompt_sent,
        real_codex_executed,
        writes_codex_home,
        writes_workbench_state: true,
        readback_summary: SessionContinuationReadbackSummary {
            status: readback_status.to_string(),
            source_kind: "h3_b_workbench_managed_last_message".to_string(),
            result_count: readback_result_count_for(status, readback_status, readback_result_count),
            unavailable_reason: if readback_result_count_for(
                status,
                readback_status,
                readback_result_count,
            )
            .is_none()
            {
                Some("H3-B readback unavailable/failed/timed_out 不等于 0 条结果。".to_string())
            } else {
                None
            },
            warnings: vec![
                "readback_unavailable_or_failed_is_not_zero_results".to_string(),
                crate::h4_execution_boundary::h4_unknown_result_warning(),
                "raw_transcript_not_read_in_h3_b".to_string(),
            ],
        },
        failure_reason,
        audit_refs,
        warnings,
    }
}

fn phase_a_audit_event(
    continuation: &ControlledSessionContinuation,
    event_id: &str,
    event_type: &str,
    attempt_id: &str,
    actor_role: &str,
    before_status: Option<String>,
    after_status: &str,
    store_revision: i64,
    reason: String,
    timestamp: &str,
    warnings: Vec<String>,
) -> SessionContinuationAuditEvent {
    SessionContinuationAuditEvent {
        event_version: 1,
        event_id: event_id.to_string(),
        event_type: event_type.to_string(),
        continuation_id: continuation.continuation_id.clone(),
        attempt_id: Some(attempt_id.to_string()),
        preview_id: continuation.preview_id.clone(),
        actor_role: actor_role.to_string(),
        before_status,
        after_status: after_status.to_string(),
        store_revision,
        reason,
        created_at: timestamp.to_string(),
        warnings,
    }
}

fn h2_audit_event(
    continuation: &ControlledSessionContinuation,
    event_id: &str,
    event_type: &str,
    attempt_id: &str,
    actor_role: &str,
    before_status: Option<String>,
    after_status: &str,
    store_revision: i64,
    reason: String,
    timestamp: &str,
    warnings: Vec<String>,
) -> SessionContinuationAuditEvent {
    SessionContinuationAuditEvent {
        event_version: 1,
        event_id: event_id.to_string(),
        event_type: event_type.to_string(),
        continuation_id: continuation.continuation_id.clone(),
        attempt_id: Some(attempt_id.to_string()),
        preview_id: continuation.preview_id.clone(),
        actor_role: actor_role.to_string(),
        before_status,
        after_status: after_status.to_string(),
        store_revision,
        reason,
        created_at: timestamp.to_string(),
        warnings,
    }
}

fn h2_phase_b_last_message_path(
    workflow_state_path: &Path,
    input: &RunControlledSessionContinuationRealResumePhaseBInput,
    timestamp: &str,
) -> Result<PathBuf, String> {
    let parent = workflow_state_path.parent().ok_or_else(|| {
        format!(
            "workflow state 路径没有父目录，无法写 H2 Phase B last message：{}",
            workflow_state_path.display()
        )
    })?;
    Ok(parent.join("runtime").join("h2-phase-b").join(format!(
        "{}.{}.last-message.txt",
        timestamp,
        short_hash(&input.continuation_id)
    )))
}

fn h3_b_last_message_path(
    workflow_state_path: &Path,
    input: &RunControlledSessionContinuationRealNewSessionH3BInput,
    timestamp: &str,
) -> Result<PathBuf, String> {
    let parent = workflow_state_path.parent().ok_or_else(|| {
        format!(
            "workflow state 路径没有父目录，无法写 H3-B last message：{}",
            workflow_state_path.display()
        )
    })?;
    Ok(parent.join("runtime").join("h3-b").join(format!(
        "{}.{}.last-message.txt",
        timestamp,
        short_hash(&input.continuation_id)
    )))
}

fn has_active_attempt_in_h4_scope(
    store: &SessionContinuationStoreV1,
    continuation: &ControlledSessionContinuation,
) -> bool {
    store.attempts.iter().any(|attempt| {
        crate::h4_execution_boundary::is_h4_active_attempt_status(&attempt.status)
            && attempt_in_h4_duplicate_scope(store, attempt, continuation)
    })
}

fn active_attempts_in_h4_scope(
    store: &SessionContinuationStoreV1,
    continuation: &ControlledSessionContinuation,
) -> Vec<CodexLocalActiveAttempt> {
    store
        .attempts
        .iter()
        .filter(|attempt| attempt_in_h4_duplicate_scope(store, attempt, continuation))
        .map(|attempt| CodexLocalActiveAttempt {
            attempt_id: attempt.attempt_id.clone(),
            status: attempt.status.clone(),
            continuation_id: Some(attempt.continuation_id.clone()),
        })
        .collect()
}

fn attempt_in_h4_duplicate_scope(
    store: &SessionContinuationStoreV1,
    attempt: &SessionContinuationAttempt,
    continuation: &ControlledSessionContinuation,
) -> bool {
    if attempt.continuation_id == continuation.continuation_id {
        return true;
    }
    let Some(existing) = store
        .continuations
        .iter()
        .find(|item| item.continuation_id == attempt.continuation_id)
    else {
        return false;
    };
    if existing.adapter_id != continuation.adapter_id
        || existing.operation_id != continuation.operation_id
    {
        return false;
    }
    if continuation.operation_id == "resume"
        && !continuation.session_id.trim().is_empty()
        && existing.session_id == continuation.session_id
    {
        return true;
    }
    if continuation.operation_id == "new_session"
        && !continuation
            .work_item_id
            .as_deref()
            .unwrap_or("")
            .trim()
            .is_empty()
        && existing.work_item_id == continuation.work_item_id
    {
        return true;
    }
    existing.workflow_id == continuation.workflow_id
        && existing.node_id == continuation.node_id
        && !continuation
            .work_item_id
            .as_deref()
            .unwrap_or("")
            .trim()
            .is_empty()
        && existing.work_item_id == continuation.work_item_id
}

fn readback_result_count_for(status: &str, readback_status: &str, raw: Option<i64>) -> Option<i64> {
    crate::h4_execution_boundary::h4_result_count(status, readback_status, raw)
}

fn inspect_authorization_matrix(
    continuation: &ControlledSessionContinuation,
    auth: &H2RealResumeAuthorizationMatrix,
) -> Vec<String> {
    let mut missing = Vec::new();
    if auth.operation_type != "resume" {
        missing.push("operation_type_must_be_resume".to_string());
    }
    required_non_empty("test_project", &auth.test_project, &mut missing);
    required_non_empty("project_root", &auth.project_root, &mut missing);
    required_non_empty("target_cwd", &auth.target_cwd, &mut missing);
    required_non_empty("target_session", &auth.target_session, &mut missing);
    required_non_empty("prompt_summary", &auth.prompt_summary, &mut missing);
    required_non_empty("prompt_ref", &auth.prompt_ref, &mut missing);
    required_non_empty("codex_home_scope", &auth.codex_home_scope, &mut missing);
    required_non_empty("sandbox", &auth.sandbox, &mut missing);
    required_non_empty("readback_plan", &auth.readback_plan, &mut missing);
    required_non_empty("evidence_path", &auth.evidence_path, &mut missing);
    required_non_empty("rollback_plan", &auth.rollback_plan, &mut missing);
    if auth.allowed_write_roots.is_empty() && auth.sandbox != "read-only" {
        missing.push("allowed_write_roots_missing".to_string());
    }
    if auth.timeout_ms.unwrap_or_default() <= 0 {
        missing.push("timeout_ms_missing_or_invalid".to_string());
    }
    if auth.prompt_sha256.len() != 64
        || !auth
            .prompt_sha256
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    {
        missing.push("prompt_sha256_invalid".to_string());
    }
    if !auth.user_confirmed_real_resume {
        missing.push("user_confirmed_real_resume_missing".to_string());
    }
    if !auth.global_supervisor_confirmed {
        missing.push("global_supervisor_confirmed_missing".to_string());
    }
    if auth.sandbox == "dangerously-bypass-approvals-and-sandbox"
        || auth.sandbox.contains("dangerously")
    {
        missing.push("dangerous_sandbox_forbidden".to_string());
    }
    for (field, value) in [
        ("project_root", auth.project_root.as_str()),
        ("target_cwd", auth.target_cwd.as_str()),
        ("evidence_path", auth.evidence_path.as_str()),
    ] {
        if contains_sensitive_fragment(value) {
            missing.push(format!("{field}_sensitive_path_forbidden"));
        }
    }
    if auth
        .allowed_write_roots
        .iter()
        .any(|root| contains_sensitive_fragment(root))
    {
        missing.push("allowed_write_roots_sensitive_path_forbidden".to_string());
    }
    if auth.project_root != continuation.project_root {
        missing.push("project_root_mismatch_continuation".to_string());
    }
    if auth.target_cwd != continuation.target_cwd {
        missing.push("target_cwd_mismatch_continuation".to_string());
    }
    if auth.target_session != continuation.session_id {
        missing.push("target_session_mismatch_continuation".to_string());
    }
    if !path_within_scope(&auth.target_cwd, &auth.project_root)
        && !auth
            .allowed_write_roots
            .iter()
            .any(|root| path_within_scope(&auth.target_cwd, root))
    {
        missing.push("target_cwd_out_of_authorized_scope".to_string());
    }
    if auth
        .allowed_write_roots
        .iter()
        .any(|root| !path_within_scope(root, &auth.project_root))
    {
        missing.push("allowed_write_root_out_of_project_scope".to_string());
    }
    missing.sort();
    missing.dedup();
    missing
}

fn inspect_h3_b_authorization_matrix(
    continuation: &ControlledSessionContinuation,
    auth: &H3RealNewSessionAuthorizationMatrix,
) -> Vec<String> {
    let mut missing = Vec::new();
    if auth.operation_type != "new_session" {
        missing.push("operation_type_must_be_new_session".to_string());
    }
    required_non_empty("test_project", &auth.test_project, &mut missing);
    required_non_empty("project_root", &auth.project_root, &mut missing);
    required_non_empty("target_cwd", &auth.target_cwd, &mut missing);
    required_non_empty("work_item_id", &auth.work_item_id, &mut missing);
    required_non_empty("prompt_summary", &auth.prompt_summary, &mut missing);
    required_non_empty("prompt_ref", &auth.prompt_ref, &mut missing);
    required_non_empty("codex_home_scope", &auth.codex_home_scope, &mut missing);
    required_non_empty("sandbox", &auth.sandbox, &mut missing);
    required_non_empty("readback_plan", &auth.readback_plan, &mut missing);
    required_non_empty("evidence_path", &auth.evidence_path, &mut missing);
    required_non_empty("rollback_plan", &auth.rollback_plan, &mut missing);
    if auth.allowed_write_roots.is_empty() && auth.sandbox != "read-only" {
        missing.push("allowed_write_roots_missing".to_string());
    }
    if auth.timeout_ms.unwrap_or_default() <= 0 {
        missing.push("timeout_ms_missing_or_invalid".to_string());
    }
    if auth.prompt_sha256.len() != 64
        || !auth
            .prompt_sha256
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    {
        missing.push("prompt_sha256_invalid".to_string());
    }
    if !auth.user_confirmed_real_new_session {
        missing.push("user_confirmed_real_new_session_missing".to_string());
    }
    if !auth.global_supervisor_confirmed {
        missing.push("global_supervisor_confirmed_missing".to_string());
    }
    if auth.sandbox == "dangerously-bypass-approvals-and-sandbox"
        || auth.sandbox.contains("dangerously")
    {
        missing.push("dangerous_sandbox_forbidden".to_string());
    }
    for (field, value) in [
        ("project_root", auth.project_root.as_str()),
        ("target_cwd", auth.target_cwd.as_str()),
        ("evidence_path", auth.evidence_path.as_str()),
    ] {
        if contains_sensitive_fragment(value) {
            missing.push(format!("{field}_sensitive_path_forbidden"));
        }
    }
    if auth
        .allowed_write_roots
        .iter()
        .any(|root| contains_sensitive_fragment(root))
    {
        missing.push("allowed_write_roots_sensitive_path_forbidden".to_string());
    }
    if auth.project_root != continuation.project_root {
        missing.push("project_root_mismatch_continuation".to_string());
    }
    if auth.target_cwd != continuation.target_cwd {
        missing.push("target_cwd_mismatch_continuation".to_string());
    }
    if Some(auth.work_item_id.as_str()) != continuation.work_item_id.as_deref() {
        missing.push("work_item_id_mismatch_continuation".to_string());
    }
    if !continuation.session_id.trim().is_empty() {
        missing.push("new_session_must_not_bind_existing_session".to_string());
    }
    if !path_within_scope(&auth.target_cwd, &auth.project_root)
        && !auth
            .allowed_write_roots
            .iter()
            .any(|root| path_within_scope(&auth.target_cwd, root))
    {
        missing.push("target_cwd_out_of_authorized_scope".to_string());
    }
    if auth
        .allowed_write_roots
        .iter()
        .any(|root| !path_within_scope(root, &auth.project_root))
    {
        missing.push("allowed_write_root_out_of_project_scope".to_string());
    }
    missing.sort();
    missing.dedup();
    missing
}

fn required_non_empty(field: &str, value: &str, missing: &mut Vec<String>) {
    if value.trim().is_empty() {
        missing.push(format!("{field}_missing"));
    }
}

fn redacted_h2_command_preview(continuation: &ControlledSessionContinuation) -> String {
    format!(
        "H2 preflight only: codex exec resume --skip-git-repo-check --json --output-last-message <workbench-managed> -C {} --sandbox {} <session:{}>",
        continuation.target_cwd,
        continuation.sandbox,
        short_hash(&continuation.session_id)
    )
}

fn redacted_real_resume_command_preview(continuation: &ControlledSessionContinuation) -> String {
    format!(
        "Controlled real resume command: codex exec resume --skip-git-repo-check --json --output-last-message <workbench-managed> -C {} --sandbox {} <session:{}>",
        continuation.target_cwd,
        continuation.sandbox,
        short_hash(&continuation.session_id)
    )
}

fn merge_warnings(existing: Vec<String>, next: Vec<String>) -> Vec<String> {
    let mut seen = BTreeSet::new();
    existing
        .into_iter()
        .chain(next)
        .filter(|warning| seen.insert(warning.clone()))
        .collect()
}

fn merge_phase_b_current_warnings(existing: Vec<String>, next: Vec<String>) -> Vec<String> {
    merge_warnings(existing, next)
        .into_iter()
        .filter(|warning| !is_stale_pre_phase_b_warning(warning))
        .collect()
}

fn is_stale_pre_phase_b_warning(warning: &str) -> bool {
    matches!(
        warning,
        "level_a_stub_only"
            | "level_b_real_execution_requires_user_approval"
            | "prompt_sent_false"
            | "real_codex_executed_false"
            | "writes_codex_home_false"
    )
}

fn contains_sensitive_fragment(value: &str) -> bool {
    let normalized = value.to_ascii_lowercase();
    normalized.contains("/.codex")
        || normalized.contains("\\.codex")
        || normalized.ends_with(".codex")
        || normalized.contains(".env")
        || normalized.contains("keychain")
        || normalized.contains("oauth")
        || normalized.contains("provider credential")
        || normalized.contains("token")
        || normalized.contains("secret")
        || normalized.contains("/auth")
        || normalized.contains("\\auth")
}

fn path_within_scope(path: &str, root: &str) -> bool {
    if root.trim().is_empty() {
        return false;
    }
    let path = Path::new(path);
    let root = Path::new(root);
    path == root || path.starts_with(root)
}

fn normalize(value: &str) -> String {
    value.trim().replace('\\', "/").to_lowercase()
}

struct StoreLock {
    path: PathBuf,
}

impl StoreLock {
    fn acquire(path: &Path, write_id: &str) -> Result<Self, String> {
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
        {
            Ok(mut file) => {
                file.write_all(write_id.as_bytes()).map_err(|error| {
                    format!("写入 continuation lock 失败 {}：{error}", path.display())
                })?;
                Ok(Self {
                    path: path.to_path_buf(),
                })
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Err(format!(
                "session_continuation_store_locked: {}",
                path.display()
            )),
            Err(error) => Err(format!(
                "创建 continuation lock 失败 {}：{error}",
                path.display()
            )),
        }
    }
}

impl Drop for StoreLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        CodexLocalCommandPlan, ContinuationAuditImpact, ContinuationFailureBoundary,
        H5DiagnosticSummaryInput, H5ProjectWorkflowDispatchPreviewInput,
        ProviderAvailabilitySummary, ReadbackExpectation, SessionContinuationGuardResult,
        SessionContinuationRequest,
    };
    use std::env;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn session_continuation_store_atomic_write_and_stub_level_a_flags() {
        let dir = temp_test_dir("session-continuation-store-atomic");
        let workflow_state_path = dir.join("workflow-state.v0.json");
        fs::write(&workflow_state_path, "{}").unwrap();
        let preview = safe_preview("needs_user_confirmation", "codex-local");
        let confirm = confirm_continuation(
            &workflow_state_path,
            &ConfirmControlledSessionContinuationInput {
                preview,
                confirmed_by: "user".to_string(),
                confirmation_reason: "Level A stub 验收确认".to_string(),
                expected_store_revision: Some(0),
            },
            "2026-06-06T10:00:00Z",
            "write-confirm",
        )
        .unwrap();
        assert_eq!(confirm.store_revision, 1);
        assert_eq!(confirm.continuation.execution_level, "level_a_stub_only");
        assert_eq!(confirm.continuation.runner_kind, "stub");

        let run = run_stub(
            &workflow_state_path,
            &RunControlledSessionContinuationStubInput {
                continuation_id: confirm.continuation.continuation_id.clone(),
                actor_role: "user".to_string(),
                expected_store_revision: Some(1),
                timeout_ms: Some(30000),
                force_stub_failure: None,
            },
            "2026-06-06T10:01:00Z",
            "write-stub",
        )
        .unwrap();
        assert_eq!(run.store_revision, 2);
        assert_eq!(run.attempt.runner_kind, "stub");
        assert_eq!(run.attempt.execution_level, "level_a_stub_only");
        assert!(!run.attempt.prompt_sent);
        assert!(!run.attempt.real_codex_executed);
        assert!(!run.attempt.writes_codex_home);
        assert_eq!(run.attempt.readback_summary.status, "readback_unavailable");
        assert_eq!(run.attempt.readback_summary.result_count, None);
        assert!(run
            .attempt
            .warnings
            .contains(&"readback_unavailable_is_not_zero_results".to_string()));
        assert!(sidecar_path(&workflow_state_path).unwrap().exists());
        assert!(dir.join("backups").exists());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn session_continuation_store_refuses_corrupt_json_without_overwrite() {
        let dir = temp_test_dir("session-continuation-store-corrupt");
        let workflow_state_path = dir.join("workflow-state.v0.json");
        fs::write(&workflow_state_path, "{}").unwrap();
        let sidecar = sidecar_path(&workflow_state_path).unwrap();
        fs::write(&sidecar, "{not json").unwrap();
        let error = load_store(&workflow_state_path, "2026-06-06T10:00:00Z").unwrap_err();
        assert!(error.contains("JSON 损坏"));
        assert_eq!(fs::read_to_string(&sidecar).unwrap(), "{not json");
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn session_continuation_store_revision_conflict_blocks_write() {
        let dir = temp_test_dir("session-continuation-store-conflict");
        let workflow_state_path = dir.join("workflow-state.v0.json");
        fs::write(&workflow_state_path, "{}").unwrap();
        let error = confirm_continuation(
            &workflow_state_path,
            &ConfirmControlledSessionContinuationInput {
                preview: safe_preview("needs_user_confirmation", "codex-local"),
                confirmed_by: "user".to_string(),
                confirmation_reason: "确认".to_string(),
                expected_store_revision: Some(99),
            },
            "2026-06-06T10:00:00Z",
            "write-conflict",
        )
        .unwrap_err();
        assert!(error.contains("session_continuation_store_conflict"));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn controlled_session_continuation_blocks_planned_adapter_or_guarded_preview() {
        let dir = temp_test_dir("controlled-session-continuation-blocked");
        let workflow_state_path = dir.join("workflow-state.v0.json");
        fs::write(&workflow_state_path, "{}").unwrap();
        let planned = confirm_continuation(
            &workflow_state_path,
            &ConfirmControlledSessionContinuationInput {
                preview: safe_preview("blocked", "claude-code"),
                confirmed_by: "user".to_string(),
                confirmation_reason: "确认".to_string(),
                expected_store_revision: Some(0),
            },
            "2026-06-06T10:00:00Z",
            "write-planned",
        )
        .unwrap_err();
        assert!(planned.contains("codex-local") || planned.contains("guard blocked"));

        let blocked = confirm_continuation(
            &workflow_state_path,
            &ConfirmControlledSessionContinuationInput {
                preview: safe_preview("blocked", "codex-local"),
                confirmed_by: "user".to_string(),
                confirmation_reason: "确认".to_string(),
                expected_store_revision: Some(0),
            },
            "2026-06-06T10:00:00Z",
            "write-blocked",
        )
        .unwrap_err();
        assert!(blocked.contains("guard blocked preview"));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn h2_real_resume_preflight_blocks_missing_authorization_without_execution() {
        let dir = temp_test_dir("h2-real-resume-preflight-blocked");
        let workflow_state_path = dir.join("workflow-state.v0.json");
        fs::write(&workflow_state_path, "{}").unwrap();
        let confirm = confirm_continuation(
            &workflow_state_path,
            &ConfirmControlledSessionContinuationInput {
                preview: safe_preview("needs_user_confirmation", "codex-local"),
                confirmed_by: "user".to_string(),
                confirmation_reason: "准备 H2 预检".to_string(),
                expected_store_revision: Some(0),
            },
            "2026-06-07T10:00:00Z",
            "write-h2-confirm",
        )
        .unwrap();
        let blocked = inspect_real_resume_authorization(
            &workflow_state_path,
            &InspectControlledSessionContinuationRealResumeInput {
                continuation_id: confirm.continuation.continuation_id.clone(),
                actor_role: "global_supervisor".to_string(),
                expected_store_revision: Some(1),
                authorization: incomplete_authorization(),
            },
            "2026-06-07T10:01:00Z",
            "write-h2-blocked",
        )
        .unwrap();

        assert_eq!(
            blocked.authorization_status,
            "blocked_waiting_authorization"
        );
        assert_eq!(blocked.attempt.status, "blocked_waiting_authorization");
        assert_eq!(
            blocked.attempt.execution_level,
            "h2_real_resume_preflight_no_execution"
        );
        assert!(!blocked.attempt.prompt_sent);
        assert!(!blocked.attempt.real_codex_executed);
        assert!(!blocked.attempt.writes_codex_home);
        assert_eq!(blocked.attempt.readback_summary.result_count, None);
        assert!(blocked
            .missing_or_invalid_items
            .contains(&"global_supervisor_confirmed_missing".to_string()));
        assert!(blocked
            .missing_or_invalid_items
            .contains(&"user_confirmed_real_resume_missing".to_string()));
        assert!(blocked.codex_local_request.is_none());
        assert!(blocked.codex_local_guard.is_none());
        let store = load_store(&workflow_state_path, "2026-06-07T10:02:00Z").unwrap();
        assert_eq!(store.attempts.len(), 1);
        assert_eq!(store.audit_events.len(), 2);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn h2_real_resume_preflight_complete_matrix_still_does_not_execute() {
        let dir = temp_test_dir("h2-real-resume-preflight-complete");
        let workflow_state_path = dir.join("workflow-state.v0.json");
        fs::write(&workflow_state_path, "{}").unwrap();
        let confirm = confirm_continuation(
            &workflow_state_path,
            &ConfirmControlledSessionContinuationInput {
                preview: safe_preview("needs_user_confirmation", "codex-local"),
                confirmed_by: "user".to_string(),
                confirmation_reason: "准备 H2 完整授权预检".to_string(),
                expected_store_revision: Some(0),
            },
            "2026-06-07T10:00:00Z",
            "write-h2-confirm-complete",
        )
        .unwrap();
        let ready = inspect_real_resume_authorization(
            &workflow_state_path,
            &InspectControlledSessionContinuationRealResumeInput {
                continuation_id: confirm.continuation.continuation_id.clone(),
                actor_role: "global_supervisor".to_string(),
                expected_store_revision: Some(1),
                authorization: complete_authorization(),
            },
            "2026-06-07T10:01:00Z",
            "write-h2-ready",
        )
        .unwrap();

        assert_eq!(ready.authorization_status, "complete_but_not_executed");
        assert_eq!(ready.attempt.status, "ready_for_real_resume_authorization");
        assert!(ready.missing_or_invalid_items.is_empty());
        assert!(!ready.attempt.prompt_sent);
        assert!(!ready.attempt.real_codex_executed);
        assert!(!ready.attempt.writes_codex_home);
        assert_eq!(ready.attempt.readback_summary.result_count, None);
        assert!(ready
            .attempt
            .warnings
            .contains(&"authorization_matrix_complete_but_runner_not_called".to_string()));
        let request = ready
            .codex_local_request
            .as_ref()
            .expect("complete H2 preflight should build H1 request");
        assert_eq!(request.operation_id, "resume");
        assert_eq!(request.session_id.as_deref(), Some("thread:offline"));
        assert_eq!(request.target_cwd, "/tmp/offline-project");
        assert_eq!(
            request.prompt_sha256,
            complete_authorization().prompt_sha256
        );
        assert_eq!(request.user_confirmation_state, "confirmed");
        let guard = ready
            .codex_local_guard
            .as_ref()
            .expect("complete H2 preflight should inspect H1 guard");
        assert!(!guard.blocks_execution);
        assert!(guard.allows_dry_run);
        let command_plan = guard
            .command_plan
            .as_ref()
            .expect("complete H2 preflight should expose command plan");
        assert_eq!(command_plan.program, "codex");
        assert!(!command_plan.shell_invocation);
        assert!(!command_plan.prompt_in_command);
        assert!(command_plan.argv.iter().any(|arg| arg == "resume"));
        assert!(!command_plan
            .argv
            .iter()
            .any(|arg| arg.contains(&complete_authorization().prompt_sha256)));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn h2_phase_a_noop_runner_path_writes_attempt_and_audit_without_real_execution() {
        let dir = temp_test_dir("h2-phase-a-runner-path");
        let workflow_state_path = dir.join("workflow-state.v0.json");
        fs::write(&workflow_state_path, "{}").unwrap();
        let confirm = confirm_continuation(
            &workflow_state_path,
            &ConfirmControlledSessionContinuationInput {
                preview: safe_preview("needs_user_confirmation", "codex-local"),
                confirmed_by: "user".to_string(),
                confirmation_reason: "准备 H2.5 Phase A".to_string(),
                expected_store_revision: Some(0),
            },
            "2026-06-07T11:00:00Z",
            "write-h2-phase-a-confirm",
        )
        .unwrap();

        let run = run_real_resume_phase_a(
            &workflow_state_path,
            &RunControlledSessionContinuationRealResumePhaseAInput {
                continuation_id: confirm.continuation.continuation_id.clone(),
                actor_role: "global_supervisor".to_string(),
                expected_store_revision: Some(1),
                authorization: complete_authorization(),
                execution_decision: Some("approved_for_phase_a".to_string()),
            },
            "2026-06-07T11:01:00Z",
            "write-h2-phase-a-run",
        )
        .unwrap();

        assert_eq!(
            run.authorization_status,
            "phase_a_runner_path_recorded_no_real_execution"
        );
        assert_eq!(run.store_revision, 2);
        assert_eq!(run.audit_events.len(), 2);
        assert_eq!(
            run.attempt.execution_level,
            "h2_phase_a_runner_path_no_real_codex"
        );
        assert_eq!(
            run.attempt.runner_kind,
            "codex_local_phase_a_noop_process_runner"
        );
        assert!(!run.attempt.prompt_sent);
        assert!(!run.attempt.real_codex_executed);
        assert!(!run.attempt.writes_codex_home);
        assert!(run.attempt.writes_workbench_state);
        assert_eq!(run.attempt.readback_summary.status, "readback_unavailable");
        assert_eq!(run.attempt.readback_summary.result_count, None);
        assert!(run.codex_local_request.is_some());
        assert!(run.codex_local_guard.is_some());
        let codex_attempt = run
            .codex_local_attempt
            .expect("phase A should expose codex-local attempt summary");
        assert!(!codex_attempt.real_codex_executed);
        assert!(!codex_attempt.writes_codex_home);
        assert_eq!(codex_attempt.readback_result.result_count, None);
        let store = load_store(&workflow_state_path, "2026-06-07T11:02:00Z").unwrap();
        assert_eq!(store.attempts.len(), 1);
        assert_eq!(store.audit_events.len(), 3);
        let runtime_store = runtime_log_store::load_store(&workflow_state_path)
            .expect("H2.6 should explicitly write runtime log sidecar for Phase A attempt");
        assert_eq!(runtime_store.revision, 1);
        assert!(runtime_store
            .warnings
            .contains(&"runtime_log_sidecar_explicitly_written".to_string()));
        assert!(runtime_store.entries.iter().any(|entry| {
            entry.category == "dispatch_attempt" && entry.status == run.attempt.status
        }));
        assert!(runtime_store.entries.iter().any(|entry| {
            entry.category == "readback"
                && entry.status == "readback_unavailable"
                && entry.summary.contains("result_count=unavailable")
        }));
        assert!(runtime_store
            .entries
            .iter()
            .all(|entry| entry.redaction_status == "redacted_safe_summary"));
        let serialized_runtime =
            serde_json::to_string(&runtime_store).expect("serialize runtime store");
        assert!(!serialized_runtime.contains("E4 preview summary"));
        assert!(!serialized_runtime.contains("codex exec resume"));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn h2_phase_a_refuses_corrupt_runtime_log_without_continuation_write() {
        let dir = temp_test_dir("h2-phase-a-runtime-log-corrupt");
        let workflow_state_path = dir.join("workflow-state.v0.json");
        fs::write(&workflow_state_path, "{}").unwrap();
        let confirm = confirm_continuation(
            &workflow_state_path,
            &ConfirmControlledSessionContinuationInput {
                preview: safe_preview("needs_user_confirmation", "codex-local"),
                confirmed_by: "user".to_string(),
                confirmation_reason: "准备 H2.6 runtime log 损坏保护".to_string(),
                expected_store_revision: Some(0),
            },
            "2026-06-07T11:20:00Z",
            "write-h2-runtime-corrupt-confirm",
        )
        .unwrap();
        let runtime_sidecar = runtime_log_store::sidecar_path(&workflow_state_path).unwrap();
        fs::write(&runtime_sidecar, "{not json").unwrap();

        let error = run_real_resume_phase_a(
            &workflow_state_path,
            &RunControlledSessionContinuationRealResumePhaseAInput {
                continuation_id: confirm.continuation.continuation_id.clone(),
                actor_role: "global_supervisor".to_string(),
                expected_store_revision: Some(1),
                authorization: complete_authorization(),
                execution_decision: Some("approved_for_phase_a".to_string()),
            },
            "2026-06-07T11:21:00Z",
            "write-h2-runtime-corrupt-run",
        )
        .unwrap_err();

        assert!(error.contains("runtime_log_sidecar_unreadable_refuse_h2_attempt"));
        assert_eq!(fs::read_to_string(&runtime_sidecar).unwrap(), "{not json");
        let store = load_store(&workflow_state_path, "2026-06-07T11:22:00Z").unwrap();
        assert_eq!(store.revision, 1);
        assert!(store.attempts.is_empty());
        assert_eq!(store.audit_events.len(), 1);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn h2_phase_a_duplicate_running_attempt_is_blocked_without_runner_call() {
        let dir = temp_test_dir("h2-phase-a-duplicate");
        let workflow_state_path = dir.join("workflow-state.v0.json");
        fs::write(&workflow_state_path, "{}").unwrap();
        let confirm = confirm_continuation(
            &workflow_state_path,
            &ConfirmControlledSessionContinuationInput {
                preview: safe_preview("needs_user_confirmation", "codex-local"),
                confirmed_by: "user".to_string(),
                confirmation_reason: "准备 H2.5 Phase A duplicate".to_string(),
                expected_store_revision: Some(0),
            },
            "2026-06-07T11:10:00Z",
            "write-h2-phase-a-confirm-duplicate",
        )
        .unwrap();
        let first = run_real_resume_phase_a(
            &workflow_state_path,
            &RunControlledSessionContinuationRealResumePhaseAInput {
                continuation_id: confirm.continuation.continuation_id.clone(),
                actor_role: "global_supervisor".to_string(),
                expected_store_revision: Some(1),
                authorization: complete_authorization(),
                execution_decision: Some("approved_for_phase_a".to_string()),
            },
            "2026-06-07T11:11:00Z",
            "write-h2-phase-a-run-duplicate-first",
        )
        .unwrap();
        assert_eq!(
            first.authorization_status,
            "phase_a_runner_path_recorded_no_real_execution"
        );

        let mut store = load_store(&workflow_state_path, "2026-06-07T11:12:00Z").unwrap();
        store.attempts[0].status = "running_h2_phase_a".to_string();
        write_store_atomic(
            &sidecar_path(&workflow_state_path).unwrap(),
            &store,
            "2026-06-07T11:12:00Z",
            "manual-running-fixture",
        )
        .unwrap();

        let duplicate = run_real_resume_phase_a(
            &workflow_state_path,
            &RunControlledSessionContinuationRealResumePhaseAInput {
                continuation_id: confirm.continuation.continuation_id.clone(),
                actor_role: "global_supervisor".to_string(),
                expected_store_revision: Some(2),
                authorization: complete_authorization(),
                execution_decision: Some("approved_for_phase_a".to_string()),
            },
            "2026-06-07T11:13:00Z",
            "write-h2-phase-a-run-duplicate-second",
        )
        .unwrap();

        assert_eq!(duplicate.authorization_status, "duplicate_blocked");
        assert_eq!(duplicate.attempt.status, "duplicate_blocked");
        assert!(duplicate.codex_local_attempt.is_none());
        assert_eq!(duplicate.attempt.readback_summary.result_count, None);
        assert!(duplicate
            .missing_or_invalid_items
            .contains(&"duplicate_running_attempt".to_string()));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn h4_duplicate_guard_blocks_same_session_scope_without_runner_call() {
        let dir = temp_test_dir("h4-duplicate-session-scope");
        let workflow_state_path = dir.join("workflow-state.v0.json");
        fs::write(&workflow_state_path, "{}").unwrap();
        let first_confirm = confirm_continuation(
            &workflow_state_path,
            &ConfirmControlledSessionContinuationInput {
                preview: safe_preview("needs_user_confirmation", "codex-local"),
                confirmed_by: "user".to_string(),
                confirmation_reason: "准备第一条 H4 duplicate scope fixture".to_string(),
                expected_store_revision: Some(0),
            },
            "2026-06-08T01:00:00Z",
            "write-h4-duplicate-first-confirm",
        )
        .unwrap();
        let first = run_real_resume_phase_a(
            &workflow_state_path,
            &RunControlledSessionContinuationRealResumePhaseAInput {
                continuation_id: first_confirm.continuation.continuation_id.clone(),
                actor_role: "global_supervisor".to_string(),
                expected_store_revision: Some(1),
                authorization: complete_authorization(),
                execution_decision: Some("approved_for_phase_a".to_string()),
            },
            "2026-06-08T01:01:00Z",
            "write-h4-duplicate-first-run",
        )
        .unwrap();
        assert!(first.codex_local_attempt.is_some());

        let mut store = load_store(&workflow_state_path, "2026-06-08T01:02:00Z").unwrap();
        store.attempts[0].status = "running_real".to_string();
        write_store_atomic(
            &sidecar_path(&workflow_state_path).unwrap(),
            &store,
            "2026-06-08T01:02:00Z",
            "manual-h4-running-real",
        )
        .unwrap();

        let mut second_preview = safe_preview("needs_user_confirmation", "codex-local");
        second_preview.preview_id =
            "session-continuation-preview:codex-local:resume:second-binding".to_string();
        second_preview.node_id = Some("node:other".to_string());
        second_preview.binding_id = Some("binding:other".to_string());
        second_preview.work_item_id = Some("work-item:other".to_string());
        second_preview.request.node_id = Some("node:other".to_string());
        second_preview.request.work_item_id = Some("work-item:other".to_string());
        let second_confirm = confirm_continuation(
            &workflow_state_path,
            &ConfirmControlledSessionContinuationInput {
                preview: second_preview,
                confirmed_by: "user".to_string(),
                confirmation_reason: "准备第二条 H4 duplicate scope fixture".to_string(),
                expected_store_revision: Some(2),
            },
            "2026-06-08T01:03:00Z",
            "write-h4-duplicate-second-confirm",
        )
        .unwrap();

        let duplicate = run_real_resume_phase_a(
            &workflow_state_path,
            &RunControlledSessionContinuationRealResumePhaseAInput {
                continuation_id: second_confirm.continuation.continuation_id.clone(),
                actor_role: "global_supervisor".to_string(),
                expected_store_revision: Some(3),
                authorization: complete_authorization(),
                execution_decision: Some("approved_for_phase_a".to_string()),
            },
            "2026-06-08T01:04:00Z",
            "write-h4-duplicate-second-run",
        )
        .unwrap();

        assert_eq!(duplicate.authorization_status, "duplicate_blocked");
        assert_eq!(duplicate.attempt.status, "duplicate_blocked");
        assert!(duplicate.codex_local_attempt.is_none());
        assert_eq!(duplicate.attempt.readback_summary.result_count, None);
        let runtime_store = runtime_log_store::load_store(&workflow_state_path).unwrap();
        assert!(runtime_store.entries.iter().any(|entry| {
            entry.category == "dispatch_attempt" && entry.status == "duplicate_blocked"
        }));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn h4_stale_cleanup_requires_revision_and_writes_audit_runtime_log_without_kill() {
        let dir = temp_test_dir("h4-stale-cleanup");
        let workflow_state_path = dir.join("workflow-state.v0.json");
        fs::write(&workflow_state_path, "{}").unwrap();
        let confirm = confirm_continuation(
            &workflow_state_path,
            &ConfirmControlledSessionContinuationInput {
                preview: safe_preview("needs_user_confirmation", "codex-local"),
                confirmed_by: "user".to_string(),
                confirmation_reason: "准备 H4 stale cleanup fixture".to_string(),
                expected_store_revision: Some(0),
            },
            "2026-06-08T01:10:00Z",
            "write-h4-stale-confirm",
        )
        .unwrap();
        let run = run_real_resume_phase_a(
            &workflow_state_path,
            &RunControlledSessionContinuationRealResumePhaseAInput {
                continuation_id: confirm.continuation.continuation_id.clone(),
                actor_role: "global_supervisor".to_string(),
                expected_store_revision: Some(1),
                authorization: complete_authorization(),
                execution_decision: Some("approved_for_phase_a".to_string()),
            },
            "2026-06-08T01:11:00Z",
            "write-h4-stale-run",
        )
        .unwrap();
        let attempt_id = run.attempt.attempt_id.clone();
        let mut store = load_store(&workflow_state_path, "2026-06-08T01:12:00Z").unwrap();
        store.attempts[0].status = "running_h2_phase_a".to_string();
        write_store_atomic(
            &sidecar_path(&workflow_state_path).unwrap(),
            &store,
            "2026-06-08T01:12:00Z",
            "manual-h4-stale-running",
        )
        .unwrap();

        let missing_revision = cleanup_stale_attempt(
            &workflow_state_path,
            &CleanupSessionContinuationStaleAttemptInput {
                attempt_id: attempt_id.clone(),
                actor_role: "global_supervisor".to_string(),
                expected_store_revision: None,
                stale_reason: "attempt exceeded H4 stale window".to_string(),
            },
            "2026-06-08T01:13:00Z",
            "write-h4-stale-missing-revision",
        )
        .unwrap_err();
        assert!(missing_revision.contains("expected_store_revision"));

        let cleanup = cleanup_stale_attempt(
            &workflow_state_path,
            &CleanupSessionContinuationStaleAttemptInput {
                attempt_id,
                actor_role: "global_supervisor".to_string(),
                expected_store_revision: Some(2),
                stale_reason: "attempt exceeded H4 stale window".to_string(),
            },
            "2026-06-08T01:14:00Z",
            "write-h4-stale-cleanup",
        )
        .unwrap();

        assert_eq!(cleanup.attempt.status, "stale_cancelled");
        assert_eq!(cleanup.attempt.readback_summary.result_count, None);
        assert!(!cleanup.attempt.prompt_sent);
        assert!(!cleanup.attempt.real_codex_executed);
        assert!(!cleanup.attempt.writes_codex_home);
        assert_eq!(
            cleanup.audit_event.event_type,
            "session_continuation_stale_attempt_cancelled"
        );
        assert!(cleanup.warnings.contains(&"no_codex_kill".to_string()));
        let runtime_store = runtime_log_store::load_store(&workflow_state_path).unwrap();
        assert!(runtime_store.entries.iter().any(|entry| {
            entry.category == "dispatch_attempt" && entry.status == "stale_cancelled"
        }));
        assert!(runtime_store.entries.iter().any(|entry| {
            entry.category == "readback"
                && entry.status == "stale_cancelled"
                && entry.summary.contains("result_count=unavailable")
        }));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn h2_phase_b_fake_runner_writes_attempt_audit_and_runtime_log() {
        struct FakePhaseBRunner;

        impl codex_local_runner::CodexLocalPhaseBProcessRunner for FakePhaseBRunner {
            fn run_phase_b(
                &self,
                _request: &CodexLocalExecutionRequest,
                _command_plan: &CodexLocalCommandPlan,
                _prompt_body: &str,
                last_message_path: &Path,
                _timeout_ms: Option<i64>,
            ) -> codex_local_runner::CodexLocalPhaseBProcessResult {
                fs::create_dir_all(last_message_path.parent().unwrap()).unwrap();
                fs::write(last_message_path, "H2_PHASE_B_FAKE_OK").unwrap();
                codex_local_runner::CodexLocalPhaseBProcessResult {
                    runner_kind: "fake_phase_b_process".to_string(),
                    status: "succeeded".to_string(),
                    exit_code: Some(0),
                    timed_out: false,
                    prompt_sent: true,
                    real_codex_executed: true,
                    writes_codex_home: true,
                    writes_project_files: false,
                    readback_status: "succeeded".to_string(),
                    readback_attempted: true,
                    readback_result_count: Some(1),
                    last_message_path: Some(last_message_path.display().to_string()),
                    failure_code: None,
                    failure_message: None,
                    retryable: false,
                    user_action_required: false,
                    warnings: vec!["fake_phase_b_runner".to_string()],
                }
            }
        }

        let dir = temp_test_dir("h2-phase-b-runner-path");
        let workflow_state_path = dir.join("workflow-state.v0.json");
        fs::write(&workflow_state_path, "{}").unwrap();
        let confirm = confirm_continuation(
            &workflow_state_path,
            &ConfirmControlledSessionContinuationInput {
                preview: safe_preview("needs_user_confirmation", "codex-local"),
                confirmed_by: "user".to_string(),
                confirmation_reason: "准备 H2 Phase B".to_string(),
                expected_store_revision: Some(0),
            },
            "2026-06-07T12:00:00Z",
            "write-h2-phase-b-confirm",
        )
        .unwrap();
        let prompt_body = "H2 phase B fake safe prompt";
        let mut authorization = complete_authorization();
        authorization.prompt_sha256 = sha256_hex(prompt_body);
        let last_message_path = dir.join("runtime/h2-phase-b/fake.last-message.txt");
        let run = run_real_resume_phase_b_with_runner(
            &workflow_state_path,
            &RunControlledSessionContinuationRealResumePhaseBInput {
                continuation_id: confirm.continuation.continuation_id.clone(),
                actor_role: "global_supervisor".to_string(),
                expected_store_revision: Some(1),
                authorization,
                execution_decision: Some("approved_for_phase_b".to_string()),
                prompt_body: prompt_body.to_string(),
            },
            "2026-06-07T12:01:00Z",
            "write-h2-phase-b-run",
            &last_message_path,
            &FakePhaseBRunner,
        )
        .unwrap();

        assert_eq!(run.authorization_status, "phase_b_real_resume_executed");
        assert_eq!(run.store_revision, 2);
        assert_eq!(run.audit_events.len(), 2);
        assert_eq!(run.attempt.execution_level, "h2_phase_b_real_codex_resume");
        assert_eq!(run.attempt.runner_kind, "fake_phase_b_process");
        assert!(run.attempt.prompt_sent);
        assert!(run.attempt.real_codex_executed);
        assert!(run.attempt.writes_codex_home);
        assert!(run.attempt.writes_workbench_state);
        assert!(run
            .attempt
            .command_preview
            .starts_with("Controlled real resume command:"));
        assert!(!run.attempt.command_preview.contains("preview only"));
        assert_eq!(run.attempt.readback_summary.status, "succeeded");
        assert_eq!(run.attempt.readback_summary.result_count, Some(1));
        assert!(run
            .warnings
            .contains(&"runner_call_allowed_after_unified_product_gate".to_string()));
        assert!(run
            .attempt
            .warnings
            .contains(&"real_execution_command_gate_v1".to_string()));
        let codex_attempt = run
            .codex_local_attempt
            .expect("phase B should expose codex-local attempt summary");
        assert!(codex_attempt.prompt_sent);
        assert!(codex_attempt.real_codex_executed);
        assert!(codex_attempt.writes_codex_home);
        let store = load_store(&workflow_state_path, "2026-06-07T12:02:00Z").unwrap();
        assert_eq!(store.attempts.len(), 1);
        assert_eq!(store.audit_events.len(), 3);
        let stored_continuation = store.continuations.first().unwrap();
        for stale_warning in [
            "level_a_stub_only",
            "level_b_real_execution_requires_user_approval",
            "prompt_sent_false",
            "real_codex_executed_false",
            "writes_codex_home_false",
        ] {
            assert!(
                !stored_continuation
                    .warnings
                    .contains(&stale_warning.to_string()),
                "Phase B current continuation summary must not keep stale warning {stale_warning}"
            );
        }
        assert!(stored_continuation
            .warnings
            .contains(&"h2_phase_b_real_runner_path".to_string()));
        assert!(store.audit_events[0]
            .warnings
            .contains(&"level_a_stub_only".to_string()));
        let runtime_store = runtime_log_store::load_store(&workflow_state_path).unwrap();
        assert_eq!(runtime_store.revision, 1);
        assert!(runtime_store
            .entries
            .iter()
            .any(|entry| { entry.category == "dispatch_attempt" && entry.status == "succeeded" }));
        assert!(runtime_store.entries.iter().any(|entry| {
            entry.category == "readback"
                && entry.status == "succeeded"
                && entry.summary.contains("result_count=1")
        }));
        let serialized_store = serde_json::to_string(&store).unwrap();
        assert!(!serialized_store.contains(prompt_body));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn h2_phase_b_blocks_prompt_hash_mismatch_before_runner() {
        let dir = temp_test_dir("h2-phase-b-hash-mismatch");
        let workflow_state_path = dir.join("workflow-state.v0.json");
        fs::write(&workflow_state_path, "{}").unwrap();
        let confirm = confirm_continuation(
            &workflow_state_path,
            &ConfirmControlledSessionContinuationInput {
                preview: safe_preview("needs_user_confirmation", "codex-local"),
                confirmed_by: "user".to_string(),
                confirmation_reason: "准备 H2 Phase B hash mismatch".to_string(),
                expected_store_revision: Some(0),
            },
            "2026-06-07T12:10:00Z",
            "write-h2-phase-b-hash-confirm",
        )
        .unwrap();
        let run = run_real_resume_phase_b_with_runner(
            &workflow_state_path,
            &RunControlledSessionContinuationRealResumePhaseBInput {
                continuation_id: confirm.continuation.continuation_id.clone(),
                actor_role: "global_supervisor".to_string(),
                expected_store_revision: Some(1),
                authorization: complete_authorization(),
                execution_decision: Some("approved_for_phase_b".to_string()),
                prompt_body: "different prompt body".to_string(),
            },
            "2026-06-07T12:11:00Z",
            "write-h2-phase-b-hash-run",
            &dir.join("runtime/h2-phase-b/hash.last-message.txt"),
            &codex_local_runner::RealCodexLocalPhaseBProcessRunner,
        )
        .unwrap();

        assert_eq!(run.authorization_status, "blocked_waiting_authorization");
        assert_eq!(run.attempt.status, "blocked_waiting_authorization");
        assert!(!run.attempt.prompt_sent);
        assert!(!run.attempt.real_codex_executed);
        assert!(!run.attempt.writes_codex_home);
        assert!(run.codex_local_attempt.is_none());
        assert!(run
            .missing_or_invalid_items
            .contains(&"prompt_body_hash_mismatch".to_string()));
        assert!(run
            .missing_or_invalid_items
            .contains(&"product_gate:blocked_waiting_authorization".to_string()));
        assert!(run
            .warnings
            .contains(&"runner_call_blocked_by_unified_product_gate".to_string()));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn h2_phase_b_user_rejected_does_not_call_runner() {
        struct PanicPhaseBRunner;

        impl codex_local_runner::CodexLocalPhaseBProcessRunner for PanicPhaseBRunner {
            fn run_phase_b(
                &self,
                _request: &crate::CodexLocalExecutionRequest,
                _command_plan: &CodexLocalCommandPlan,
                _prompt_body: &str,
                _last_message_path: &std::path::Path,
                _timeout_ms: Option<i64>,
            ) -> codex_local_runner::CodexLocalPhaseBProcessResult {
                panic!("Phase B runner must not be called when user rejected execution")
            }
        }

        let dir = temp_test_dir("h2-phase-b-user-rejected");
        let workflow_state_path = dir.join("workflow-state.v0.json");
        fs::write(&workflow_state_path, "{}").unwrap();
        let confirm = confirm_continuation(
            &workflow_state_path,
            &ConfirmControlledSessionContinuationInput {
                preview: safe_preview("needs_user_confirmation", "codex-local"),
                confirmed_by: "user".to_string(),
                confirmation_reason: "准备 H2 Phase B user rejected".to_string(),
                expected_store_revision: Some(0),
            },
            "2026-06-07T12:20:00Z",
            "write-h2-phase-b-rejected-confirm",
        )
        .unwrap();
        let prompt_body = "safe rejected prompt body";
        let mut authorization = complete_authorization();
        authorization.prompt_sha256 = sha256_hex(prompt_body);

        let run = run_real_resume_phase_b_with_runner(
            &workflow_state_path,
            &RunControlledSessionContinuationRealResumePhaseBInput {
                continuation_id: confirm.continuation.continuation_id.clone(),
                actor_role: "global_supervisor".to_string(),
                expected_store_revision: Some(1),
                authorization,
                execution_decision: Some("rejected".to_string()),
                prompt_body: prompt_body.to_string(),
            },
            "2026-06-07T12:21:00Z",
            "write-h2-phase-b-rejected-run",
            &dir.join("runtime/h2-phase-b/rejected.last-message.txt"),
            &PanicPhaseBRunner,
        )
        .unwrap();

        assert_eq!(run.authorization_status, "user_rejected");
        assert_eq!(run.attempt.status, "user_rejected");
        assert!(!run.attempt.prompt_sent);
        assert!(!run.attempt.real_codex_executed);
        assert!(!run.attempt.writes_codex_home);
        assert!(run.codex_local_attempt.is_none());
        assert!(run
            .missing_or_invalid_items
            .contains(&"user_rejected_real_resume".to_string()));
        assert!(run
            .missing_or_invalid_items
            .contains(&"product_gate:user_rejected".to_string()));
        assert!(run
            .attempt
            .warnings
            .contains(&"runner_call_blocked_by_unified_product_gate".to_string()));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn h3_b_fake_runner_writes_new_session_attempt_audit_and_runtime_log() {
        struct FakeH3BRunner;

        impl codex_local_runner::CodexLocalPhaseBProcessRunner for FakeH3BRunner {
            fn run_phase_b(
                &self,
                _request: &CodexLocalExecutionRequest,
                _command_plan: &CodexLocalCommandPlan,
                _prompt_body: &str,
                last_message_path: &Path,
                _timeout_ms: Option<i64>,
            ) -> codex_local_runner::CodexLocalPhaseBProcessResult {
                fs::create_dir_all(last_message_path.parent().unwrap()).unwrap();
                fs::write(last_message_path, "H3_B_FAKE_LAST_MESSAGE_OK").unwrap();
                codex_local_runner::CodexLocalPhaseBProcessResult {
                    runner_kind: "fake_h3_b_process".to_string(),
                    status: "succeeded".to_string(),
                    exit_code: Some(0),
                    timed_out: false,
                    prompt_sent: true,
                    real_codex_executed: true,
                    writes_codex_home: true,
                    writes_project_files: false,
                    readback_status: "succeeded".to_string(),
                    readback_attempted: true,
                    readback_result_count: Some(1),
                    last_message_path: Some(last_message_path.display().to_string()),
                    failure_code: None,
                    failure_message: None,
                    retryable: false,
                    user_action_required: false,
                    warnings: vec!["fake_h3_b_runner".to_string()],
                }
            }
        }

        let dir = temp_test_dir("h3-b-new-session-runner-path");
        let workflow_state_path = dir.join("workflow-state.v0.json");
        fs::write(&workflow_state_path, "{}").unwrap();
        let confirm = confirm_continuation(
            &workflow_state_path,
            &ConfirmControlledSessionContinuationInput {
                preview: h3_new_session_preview("/tmp/offline-project"),
                confirmed_by: "user_and_global_supervisor".to_string(),
                confirmation_reason: "准备 H3-B fake new session".to_string(),
                expected_store_revision: Some(0),
            },
            "2026-06-07T13:00:00Z",
            "write-h3-b-confirm",
        )
        .unwrap();
        let prompt_body = "H3-B fake safe prompt";
        let mut authorization = complete_h3_b_authorization("/tmp/offline-project", prompt_body);
        authorization.prompt_sha256 = sha256_hex(prompt_body);
        let last_message_path = dir.join("runtime/h3-b/fake.last-message.txt");
        let run = run_real_new_session_h3_b_with_runner(
            &workflow_state_path,
            &RunControlledSessionContinuationRealNewSessionH3BInput {
                continuation_id: confirm.continuation.continuation_id.clone(),
                actor_role: "global_supervisor".to_string(),
                expected_store_revision: Some(1),
                authorization,
                execution_decision: Some("approved_for_h3_b".to_string()),
                prompt_body: prompt_body.to_string(),
            },
            "2026-06-07T13:01:00Z",
            "write-h3-b-run",
            &last_message_path,
            &FakeH3BRunner,
        )
        .unwrap();

        assert_eq!(run.authorization_status, "h3_b_real_new_session_executed");
        assert_eq!(run.store_revision, 2);
        assert_eq!(run.audit_events.len(), 2);
        assert_eq!(run.attempt.execution_level, "h3_b_real_codex_new_session");
        assert_eq!(run.attempt.runner_kind, "fake_h3_b_process");
        assert!(run.attempt.prompt_sent);
        assert!(run.attempt.real_codex_executed);
        assert!(run.attempt.writes_codex_home);
        assert!(run.attempt.writes_workbench_state);
        assert_eq!(run.attempt.readback_summary.status, "succeeded");
        assert_eq!(run.attempt.readback_summary.result_count, Some(1));
        assert!(run
            .warnings
            .contains(&"runner_call_allowed_after_unified_product_gate".to_string()));
        assert!(run
            .attempt
            .warnings
            .contains(&"real_execution_command_gate_v1".to_string()));
        let request = run
            .codex_local_request
            .as_ref()
            .expect("H3-B should expose codex-local request");
        assert_eq!(request.operation_id, "new_session");
        assert_eq!(request.session_id, None);
        assert_eq!(request.work_item_id.as_deref(), Some("work-item:h3-b"));
        let command_plan = run
            .codex_local_guard
            .as_ref()
            .and_then(|guard| guard.command_plan.as_ref())
            .expect("H3-B should expose command plan");
        assert_eq!(command_plan.program, "codex");
        assert!(!command_plan.shell_invocation);
        assert!(!command_plan.prompt_in_command);
        assert!(!command_plan.argv.iter().any(|arg| arg == "resume"));
        let store = load_store(&workflow_state_path, "2026-06-07T13:02:00Z").unwrap();
        let serialized_store = serde_json::to_string(&store).unwrap();
        assert!(!serialized_store.contains(prompt_body));
        assert!(store
            .audit_events
            .iter()
            .any(|event| { event.event_type == "session_continuation_h3_b_completed" }));
        let runtime_store = runtime_log_store::load_store(&workflow_state_path).unwrap();
        assert!(runtime_store
            .entries
            .iter()
            .any(|entry| { entry.category == "dispatch_attempt" && entry.status == "succeeded" }));
        assert!(runtime_store.entries.iter().any(|entry| {
            entry.category == "readback"
                && entry.status == "succeeded"
                && entry.summary.contains("result_count=1")
        }));
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn h3_b_blocks_prompt_hash_mismatch_before_runner() {
        let dir = temp_test_dir("h3-b-hash-mismatch");
        let workflow_state_path = dir.join("workflow-state.v0.json");
        fs::write(&workflow_state_path, "{}").unwrap();
        let confirm = confirm_continuation(
            &workflow_state_path,
            &ConfirmControlledSessionContinuationInput {
                preview: h3_new_session_preview("/tmp/offline-project"),
                confirmed_by: "user_and_global_supervisor".to_string(),
                confirmation_reason: "准备 H3-B hash mismatch".to_string(),
                expected_store_revision: Some(0),
            },
            "2026-06-07T13:10:00Z",
            "write-h3-b-hash-confirm",
        )
        .unwrap();
        let run = run_real_new_session_h3_b_with_runner(
            &workflow_state_path,
            &RunControlledSessionContinuationRealNewSessionH3BInput {
                continuation_id: confirm.continuation.continuation_id.clone(),
                actor_role: "global_supervisor".to_string(),
                expected_store_revision: Some(1),
                authorization: complete_h3_b_authorization(
                    "/tmp/offline-project",
                    "expected prompt body",
                ),
                execution_decision: Some("approved_for_h3_b".to_string()),
                prompt_body: "different prompt body".to_string(),
            },
            "2026-06-07T13:11:00Z",
            "write-h3-b-hash-run",
            &dir.join("runtime/h3-b/hash.last-message.txt"),
            &codex_local_runner::RealCodexLocalPhaseBProcessRunner,
        )
        .unwrap();

        assert_eq!(run.authorization_status, "blocked_waiting_authorization");
        assert_eq!(run.attempt.status, "blocked_waiting_authorization");
        assert!(!run.attempt.prompt_sent);
        assert!(!run.attempt.real_codex_executed);
        assert!(!run.attempt.writes_codex_home);
        assert!(run.codex_local_attempt.is_none());
        assert!(run
            .missing_or_invalid_items
            .contains(&"prompt_body_hash_mismatch".to_string()));
        assert!(run
            .missing_or_invalid_items
            .contains(&"product_gate:blocked_waiting_authorization".to_string()));
        assert!(run
            .warnings
            .contains(&"runner_call_blocked_by_unified_product_gate".to_string()));
        assert_eq!(run.attempt.readback_summary.result_count, None);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    #[ignore = "requires explicit H2 Phase B real codex resume authorization"]
    fn h2_phase_b_real_mario_test_probe_requires_env_authorization() {
        let project_root = env::var("H2_PHASE_B_PROJECT_ROOT")
            .expect("H2_PHASE_B_PROJECT_ROOT is required for real probe");
        let session_id = env::var("H2_PHASE_B_SESSION_ID")
            .expect("H2_PHASE_B_SESSION_ID is required for real probe");
        let prompt_body = env::var("H2_PHASE_B_PROMPT_BODY")
            .expect("H2_PHASE_B_PROMPT_BODY is required for real probe");
        let expected_marker = env::var("H2_PHASE_B_EXPECTED_MARKER")
            .expect("H2_PHASE_B_EXPECTED_MARKER is required for real probe");
        let parent = env::var("H2_PHASE_B_WORKFLOW_STATE_PARENT")
            .expect("H2_PHASE_B_WORKFLOW_STATE_PARENT is required for real probe");
        assert!(
            parent.starts_with("/Users/yoyi/workspace/product-line/tmp/h2-phase-b-real-resume"),
            "real probe workflow state parent must be inside product-line tmp"
        );
        assert_eq!(project_root, "/Users/yoyi/Documents/mario test");
        assert_eq!(session_id, "019e798a-6ce5-76c3-b8ee-33bd0fda841f");

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = PathBuf::from(parent).join(format!("run-{unique}"));
        fs::create_dir_all(&dir).unwrap();
        let workflow_state_path = dir.join("workflow-state.v0.json");
        fs::write(&workflow_state_path, "{}").unwrap();

        let mut preview = safe_preview("needs_user_confirmation", "codex-local");
        preview.project_id = Some("project:mario-test".to_string());
        preview.project_root = Some(project_root.clone());
        preview.workflow_id = Some("workflow:mario-test:h2-phase-b".to_string());
        preview.node_id = Some("node:global-director:h2-phase-b".to_string());
        preview.binding_id = Some("binding:mario-test:director".to_string());
        preview.work_item_id = Some("work-item:h2-phase-b:mario-test-safe-probe".to_string());
        preview.target_session_id = Some(session_id.clone());
        preview.target_session_title = Some("mario test 总指导".to_string());
        preview.target_cwd = Some(project_root.clone());
        preview.allowed_write_roots_summary = vec![project_root.clone()];
        preview.sandbox_summary = "workspace-write".to_string();
        preview.prompt_summary = "H2 Phase B mario test safe real resume probe".to_string();
        preview.request.project_id = Some("project:mario-test".to_string());
        preview.request.project_root = Some(project_root.clone());
        preview.request.workflow_id = Some("workflow:mario-test:h2-phase-b".to_string());
        preview.request.node_id = Some("node:global-director:h2-phase-b".to_string());
        preview.request.session_id = Some(session_id.clone());
        preview.request.work_item_id =
            Some("work-item:h2-phase-b:mario-test-safe-probe".to_string());
        preview.request.target_cwd = Some(project_root.clone());
        preview.request.allowed_write_roots = vec![project_root.clone()];
        preview.request.sandbox = "workspace-write".to_string();
        preview.request.prompt_summary = "H2 Phase B mario test safe real resume probe".to_string();

        let confirm = confirm_continuation(
            &workflow_state_path,
            &ConfirmControlledSessionContinuationInput {
                preview,
                confirmed_by: "user_and_global_supervisor".to_string(),
                confirmation_reason:
                    "用户授权测试项目和 mario test 权限，执行 H2 Phase B 安全 probe。".to_string(),
                expected_store_revision: Some(0),
            },
            "2026-06-08T00:00:00Z",
            "write-h2-phase-b-real-confirm",
        )
        .unwrap();
        let authorization = H2RealResumeAuthorizationMatrix {
            operation_type: "resume".to_string(),
            test_project: "mario test H2 Phase B safe probe".to_string(),
            project_root: project_root.clone(),
            target_cwd: project_root.clone(),
            target_session: session_id,
            prompt_summary: "H2 Phase B mario test safe real resume probe".to_string(),
            prompt_sha256: sha256_hex(&prompt_body),
            prompt_ref: "workbench-managed:h2-phase-b:mario-test-safe-probe:2026-06-08"
                .to_string(),
            allowed_write_roots: vec![project_root],
            codex_home_scope: "Codex CLI minimum session state for one authorized resume; no credential material requested."
                .to_string(),
            sandbox: "workspace-write".to_string(),
            timeout_ms: Some(120_000),
            readback_plan: "workbench-managed last message; unavailable is not zero".to_string(),
            evidence_path:
                "/Users/yoyi/workspace/product-line/evidence/2026-06-08-stage-h-h2-phase-b-mario-test-real-resume-productization-probe-v1.md"
                    .to_string(),
            rollback_plan: "hash mario test files before and after; no project file edits expected"
                .to_string(),
            user_confirmed_real_resume: true,
            global_supervisor_confirmed: true,
        };
        let last_message_path = dir.join("runtime/h2-phase-b/mario.last-message.txt");
        let run = run_real_resume_phase_b_with_runner(
            &workflow_state_path,
            &RunControlledSessionContinuationRealResumePhaseBInput {
                continuation_id: confirm.continuation.continuation_id.clone(),
                actor_role: "global_supervisor".to_string(),
                expected_store_revision: Some(1),
                authorization,
                execution_decision: Some("approved_for_phase_b".to_string()),
                prompt_body,
            },
            "2026-06-08T00:01:00Z",
            "write-h2-phase-b-real-run",
            &last_message_path,
            &codex_local_runner::RealCodexLocalPhaseBProcessRunner,
        )
        .unwrap();

        println!(
            "H2_PHASE_B_WORKFLOW_STATE_PATH={}",
            workflow_state_path.display()
        );
        println!(
            "H2_PHASE_B_LAST_MESSAGE_PATH={}",
            last_message_path.display()
        );
        println!(
            "H2_PHASE_B_AUTHORIZATION_STATUS={}",
            run.authorization_status
        );
        println!("H2_PHASE_B_ATTEMPT_STATUS={}", run.attempt.status);
        assert_eq!(run.authorization_status, "phase_b_real_resume_executed");
        assert_eq!(run.attempt.status, "succeeded");
        assert!(run.attempt.prompt_sent);
        assert!(run.attempt.real_codex_executed);
        assert!(run.attempt.writes_codex_home);
        assert_eq!(run.attempt.readback_summary.result_count, Some(1));
        let last_message = fs::read_to_string(&last_message_path).unwrap();
        assert!(last_message.contains(&expected_marker));
        let serialized_store = serde_json::to_string(
            &load_store(&workflow_state_path, "2026-06-08T00:02:00Z").unwrap(),
        )
        .unwrap();
        assert!(!serialized_store.contains(&expected_marker));
    }

    #[test]
    #[ignore = "requires explicit H3-B real codex new-session authorization"]
    fn h3_b_real_new_session_fixture_probe_requires_env_authorization() {
        let fixture_root =
            env::var("H3_B_FIXTURE_ROOT").expect("H3_B_FIXTURE_ROOT is required for real probe");
        let prompt_path =
            env::var("H3_B_PROMPT_PATH").expect("H3_B_PROMPT_PATH is required for real probe");
        let expected_marker = env::var("H3_B_EXPECTED_MARKER")
            .expect("H3_B_EXPECTED_MARKER is required for real probe");
        let parent = env::var("H3_B_WORKFLOW_STATE_PARENT")
            .expect("H3_B_WORKFLOW_STATE_PARENT is required for real probe");
        assert_eq!(
            fixture_root,
            "/Users/yoyi/workspace/product-line/tmp/h3-new-session-fixture"
        );
        assert!(
            prompt_path.starts_with(
                "/Users/yoyi/workspace/product-line/tmp/h3-new-session-fixture/.workbench/"
            ),
            "H3-B prompt ref must be inside fixture .workbench"
        );
        assert!(
            parent.starts_with(
                "/Users/yoyi/workspace/product-line/tmp/h3-new-session-fixture/.workbench/h3-b-runs"
            ),
            "H3-B workflow state parent must be inside fixture .workbench/h3-b-runs"
        );

        let prompt_body = fs::read_to_string(&prompt_path).expect("read H3-B prompt ref");
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = PathBuf::from(parent).join(format!("run-{unique}"));
        fs::create_dir_all(&dir).unwrap();
        let workflow_state_path = dir.join("workflow-state.v0.json");
        fs::write(&workflow_state_path, "{}").unwrap();

        let confirm = confirm_continuation(
            &workflow_state_path,
            &ConfirmControlledSessionContinuationInput {
                preview: h3_new_session_preview(&fixture_root),
                confirmed_by: "user_and_global_supervisor".to_string(),
                confirmation_reason: "用户授权 H3-B 隔离 fixture、allowed write roots、Codex home 最小新会话副作用和一次真实 codex exec new_session probe。".to_string(),
                expected_store_revision: Some(0),
            },
            "2026-06-08T00:10:00Z",
            "write-h3-b-real-confirm",
        )
        .unwrap();
        let authorization = H3RealNewSessionAuthorizationMatrix {
            operation_type: "new_session".to_string(),
            test_project: "H3-B isolated new-session fixture".to_string(),
            project_root: fixture_root.clone(),
            target_cwd: fixture_root.clone(),
            work_item_id: "work-item:h3-b".to_string(),
            prompt_summary: "H3 real new session safe probe".to_string(),
            prompt_sha256: sha256_hex(&prompt_body),
            prompt_ref: "workbench-managed:h3-real-new-session-safe-probe:v1".to_string(),
            allowed_write_roots: vec![fixture_root],
            codex_home_scope: "Codex CLI minimum session state for one authorized new session; no credential material requested."
                .to_string(),
            sandbox: "workspace-write".to_string(),
            timeout_ms: Some(120_000),
            readback_plan: "workbench-managed last message; unavailable is not zero".to_string(),
            evidence_path:
                "/Users/yoyi/workspace/product-line/evidence/2026-06-07-stage-h-h3-b-real-new-session-final-approval-and-fixture-run-v1.md"
                    .to_string(),
            rollback_plan:
                "hash fixture files before and after; only fixture changes are allowed"
                    .to_string(),
            user_confirmed_real_new_session: true,
            global_supervisor_confirmed: true,
        };
        let run = run_real_new_session_h3_b(
            &workflow_state_path,
            &RunControlledSessionContinuationRealNewSessionH3BInput {
                continuation_id: confirm.continuation.continuation_id.clone(),
                actor_role: "global_supervisor".to_string(),
                expected_store_revision: Some(1),
                authorization,
                execution_decision: Some("approved_for_h3_b".to_string()),
                prompt_body,
            },
            "2026-06-08T00:11:00Z",
            "write-h3-b-real-run",
        )
        .unwrap();
        let last_message_path = h3_b_last_message_path(
            &workflow_state_path,
            &RunControlledSessionContinuationRealNewSessionH3BInput {
                continuation_id: confirm.continuation.continuation_id.clone(),
                actor_role: "global_supervisor".to_string(),
                expected_store_revision: Some(1),
                authorization: complete_h3_b_authorization(
                    "/Users/yoyi/workspace/product-line/tmp/h3-new-session-fixture",
                    "placeholder",
                ),
                execution_decision: Some("approved_for_h3_b".to_string()),
                prompt_body: "placeholder".to_string(),
            },
            "2026-06-08T00:11:00Z",
        )
        .unwrap();

        println!("H3_B_WORKFLOW_STATE_PATH={}", workflow_state_path.display());
        println!("H3_B_LAST_MESSAGE_PATH={}", last_message_path.display());
        println!("H3_B_AUTHORIZATION_STATUS={}", run.authorization_status);
        println!("H3_B_ATTEMPT_STATUS={}", run.attempt.status);
        assert_eq!(run.authorization_status, "h3_b_real_new_session_executed");
        assert_eq!(run.attempt.status, "succeeded");
        assert!(run.attempt.prompt_sent);
        assert!(run.attempt.real_codex_executed);
        assert!(run.attempt.writes_codex_home);
        assert_eq!(run.attempt.readback_summary.result_count, Some(1));
        let last_message = fs::read_to_string(&last_message_path).unwrap();
        assert!(last_message.contains(&expected_marker));
        let serialized_store = serde_json::to_string(
            &load_store(&workflow_state_path, "2026-06-08T00:12:00Z").unwrap(),
        )
        .unwrap();
        assert!(!serialized_store.contains(&expected_marker));
    }

    #[test]
    #[ignore = "requires explicit H5-Level-B1 real project workflow dispatch authorization"]
    fn h5_level_b1_real_mario_test_project_workflow_dispatch_requires_env_authorization() {
        let project_root =
            env::var("H5_B1_PROJECT_ROOT").expect("H5_B1_PROJECT_ROOT is required for real probe");
        let session_id =
            env::var("H5_B1_SESSION_ID").expect("H5_B1_SESSION_ID is required for real probe");
        let prompt_path =
            env::var("H5_B1_PROMPT_PATH").expect("H5_B1_PROMPT_PATH is required for real probe");
        let expected_marker = env::var("H5_B1_EXPECTED_MARKER")
            .expect("H5_B1_EXPECTED_MARKER is required for real probe");
        let parent = env::var("H5_B1_WORKFLOW_STATE_PARENT")
            .expect("H5_B1_WORKFLOW_STATE_PARENT is required for real probe");
        assert_eq!(project_root, "/Users/yoyi/Documents/mario test");
        assert_eq!(session_id, "019e798a-ac37-7771-b982-e38084fcd22e");
        assert_eq!(
            expected_marker,
            "H5_LEVEL_B_MARIO_TEST_CODEX_DEV_REAL_DISPATCH_OK_2026_06_08"
        );
        assert!(
            prompt_path
                .starts_with("/Users/yoyi/workspace/product-line/tmp/h5-level-b1-real-dispatch/"),
            "H5-B1 prompt ref must be inside product-line tmp/h5-level-b1-real-dispatch"
        );
        assert!(
            parent.starts_with(
                "/Users/yoyi/workspace/product-line/tmp/h5-level-b1-real-dispatch/runs"
            ),
            "H5-B1 workflow state parent must be inside product-line tmp/h5-level-b1-real-dispatch/runs"
        );

        let prompt_body = fs::read_to_string(&prompt_path).expect("read H5-B1 prompt ref");
        let prompt_sha256 = sha256_hex(&prompt_body);
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = PathBuf::from(parent).join(format!("run-{unique}"));
        fs::create_dir_all(&dir).unwrap();
        let workflow_state_path = dir.join("workflow-state.v0.json");
        write_h5_level_b1_workflow_state(&workflow_state_path, &project_root, &session_id);

        let preview_request = H5ProjectWorkflowDispatchPreviewInput {
            project_root: project_root.clone(),
            project_id: "project:users-yoyi-documents-mario-test".to_string(),
            workflow_id: "workflow:users-yoyi-documents-mario-test:default".to_string(),
            dispatch_id: "dispatch:h5-level-b1:mario-test:codex-dev:read-only-probe:v1"
                .to_string(),
            actor_id: "project_director".to_string(),
            operation_id: Some("resume".to_string()),
            session_id: Some(session_id.clone()),
            target_cwd: Some(project_root.clone()),
            sandbox: Some("read-only".to_string()),
            prompt_summary:
                "H5 Level B1 project-workflow-bound read-only dispatch probe for mario test codex-dev worker."
                    .to_string(),
            prompt_ref:
                "workbench-managed:h5-level-b1:mario-test:codex-dev:read-only-probe:v1"
                    .to_string(),
            prompt_sha256: prompt_sha256.clone(),
            h3_b_level_b_authorized: false,
            expected_workflow_revision: Some(1),
            diagnostic_summary: Some(H5DiagnosticSummaryInput {
                overall_severity: "ok".to_string(),
                blocked_count: 0,
                degraded_states: vec![],
            }),
        };
        let preview = crate::h5_project_dispatch_bridge::preview_h5_project_workflow_dispatch_at(
            &workflow_state_path,
            &preview_request,
        )
        .unwrap();
        assert_eq!(
            preview.memory_packet.fingerprint.as_deref(),
            Some("h5-level-b1-memory-fingerprint-mario-test-codex-dev-2026-06-08")
        );
        assert!(!preview
            .blocked_reasons
            .contains(&"task_memory_packet_stale".to_string()));
        assert!(!preview
            .blocked_reasons
            .contains(&"diagnostics_blocking_degraded".to_string()));
        assert!(!preview
            .blocked_reasons
            .contains(&"duplicate_dispatch_blocked".to_string()));

        let confirm = confirm_continuation(
            &workflow_state_path,
            &ConfirmControlledSessionContinuationInput {
                preview: h5_level_b1_session_preview(&project_root, &session_id),
                confirmed_by: "user_and_global_supervisor".to_string(),
                confirmation_reason: "用户和全局主管授权 H5-Level-B1 mario test 开发线 worker session 的一次 read-only project workflow real dispatch resume probe。".to_string(),
                expected_store_revision: Some(0),
            },
            "2026-06-08T00:30:00Z",
            "write-h5-level-b1-real-confirm",
        )
        .unwrap();
        let authorization = H2RealResumeAuthorizationMatrix {
            operation_type: "resume".to_string(),
            test_project: "mario test H5-Level-B1 project workflow read-only dispatch probe"
                .to_string(),
            project_root: project_root.clone(),
            target_cwd: project_root.clone(),
            target_session: session_id.clone(),
            prompt_summary:
                "H5 Level B1 project-workflow-bound read-only dispatch probe for mario test codex-dev worker."
                    .to_string(),
            prompt_sha256,
            prompt_ref:
                "workbench-managed:h5-level-b1:mario-test:codex-dev:read-only-probe:v1"
                    .to_string(),
            allowed_write_roots: vec![project_root.clone()],
            codex_home_scope: "Codex CLI minimum session state for one authorized H5-Level-B1 resume; no credential material requested."
                .to_string(),
            sandbox: "read-only".to_string(),
            timeout_ms: Some(120_000),
            readback_plan:
                "workbench-managed last message plus continuation/runtime/audit refs; unavailable is not zero"
                    .to_string(),
            evidence_path:
                "/Users/yoyi/workspace/product-line/evidence/2026-06-08-stage-h-h5-level-b1-mario-test-project-workflow-real-dispatch-run-v1.md"
                    .to_string(),
            rollback_plan:
                "hash mario test project files before and after; B1 does not authorize project file edits"
                    .to_string(),
            user_confirmed_real_resume: true,
            global_supervisor_confirmed: true,
        };
        let run_input = RunControlledSessionContinuationRealResumePhaseBInput {
            continuation_id: confirm.continuation.continuation_id.clone(),
            actor_role: "project_director".to_string(),
            expected_store_revision: Some(1),
            authorization,
            execution_decision: Some("approved_for_phase_b".to_string()),
            prompt_body,
        };
        let run = run_real_resume_phase_b(
            &workflow_state_path,
            &run_input,
            "2026-06-08T00:31:00Z",
            "write-h5-level-b1-real-run",
        )
        .unwrap();
        let last_message_path =
            h2_phase_b_last_message_path(&workflow_state_path, &run_input, "2026-06-08T00:31:00Z")
                .unwrap();

        println!(
            "H5_B1_WORKFLOW_STATE_PATH={}",
            workflow_state_path.display()
        );
        println!(
            "H5_B1_SESSION_CONTINUATION_STORE_PATH={}",
            dir.join("session-continuations.v1.json").display()
        );
        println!(
            "H5_B1_RUNTIME_LOG_PATH={}",
            dir.join("runtime-logs.v1.json").display()
        );
        println!("H5_B1_LAST_MESSAGE_PATH={}", last_message_path.display());
        println!("H5_B1_AUTHORIZATION_STATUS={}", run.authorization_status);
        println!("H5_B1_ATTEMPT_STATUS={}", run.attempt.status);
        assert_eq!(run.authorization_status, "phase_b_real_resume_executed");
        assert_eq!(run.attempt.status, "succeeded");
        assert!(run.attempt.prompt_sent);
        assert!(run.attempt.real_codex_executed);
        assert!(run.attempt.writes_codex_home);
        assert_eq!(run.attempt.readback_summary.result_count, Some(1));
        assert_eq!(
            run.codex_local_request
                .as_ref()
                .and_then(|request| request.work_item_id.as_deref()),
            Some("work-item:h5-level-b1:mario-test:codex-dev:read-only-probe:v1")
        );
        let last_message = fs::read_to_string(&last_message_path).unwrap();
        assert!(last_message.contains(&expected_marker));
        let serialized_store = serde_json::to_string(
            &load_store(&workflow_state_path, "2026-06-08T00:32:00Z").unwrap(),
        )
        .unwrap();
        assert!(!serialized_store.contains(&expected_marker));
    }

    #[test]
    #[ignore = "requires explicit H5-Level-B2 real project workflow write-probe authorization"]
    fn h5_level_b2_real_mario_test_project_workflow_write_probe_requires_env_authorization() {
        let project_root =
            env::var("H5_B2_PROJECT_ROOT").expect("H5_B2_PROJECT_ROOT is required for real probe");
        let session_id =
            env::var("H5_B2_SESSION_ID").expect("H5_B2_SESSION_ID is required for real probe");
        let prompt_path =
            env::var("H5_B2_PROMPT_PATH").expect("H5_B2_PROMPT_PATH is required for real probe");
        let expected_marker = env::var("H5_B2_EXPECTED_MARKER")
            .expect("H5_B2_EXPECTED_MARKER is required for real probe");
        let parent = env::var("H5_B2_WORKFLOW_STATE_PARENT")
            .expect("H5_B2_WORKFLOW_STATE_PARENT is required for real probe");
        assert_eq!(project_root, "/Users/yoyi/Documents/mario test");
        assert_eq!(session_id, "019e798a-ac37-7771-b982-e38084fcd22e");
        assert_eq!(
            expected_marker,
            "H5_LEVEL_B2_MARIO_TEST_CODEX_DEV_WRITE_PROBE_OK_2026_06_08"
        );
        assert!(
            prompt_path
                .starts_with("/Users/yoyi/workspace/product-line/tmp/h5-level-b2-real-dispatch/"),
            "H5-B2 prompt ref must be inside product-line tmp/h5-level-b2-real-dispatch"
        );
        assert!(
            parent.starts_with(
                "/Users/yoyi/workspace/product-line/tmp/h5-level-b2-real-dispatch/runs"
            ),
            "H5-B2 workflow state parent must be inside product-line tmp/h5-level-b2-real-dispatch/runs"
        );

        let project_root_path = PathBuf::from(&project_root);
        let write_root_path = project_root_path.join(".workbench/h5-b2");
        fs::create_dir_all(&write_root_path).expect("create authorized H5-B2 write root");
        let write_root = write_root_path.display().to_string();
        let probe_path = write_root_path.join("real-dispatch-write-probe.md");
        let probe_pre_state = if probe_path.exists() {
            format!("exists:{}", sha256_file(&probe_path))
        } else {
            "missing".to_string()
        };
        let pre_core_hashes = mario_core_file_hashes(&project_root_path);
        let pre_project_files = project_file_set(&project_root_path);

        let prompt_body = fs::read_to_string(&prompt_path).expect("read H5-B2 prompt ref");
        let prompt_sha256 = sha256_hex(&prompt_body);
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = PathBuf::from(parent).join(format!("run-{unique}"));
        fs::create_dir_all(&dir).unwrap();
        let workflow_state_path = dir.join("workflow-state.v0.json");
        write_h5_level_b2_workflow_state(
            &workflow_state_path,
            &project_root,
            &write_root,
            &session_id,
        );

        let preview_request = H5ProjectWorkflowDispatchPreviewInput {
            project_root: project_root.clone(),
            project_id: "project:users-yoyi-documents-mario-test".to_string(),
            workflow_id: "workflow:users-yoyi-documents-mario-test:default".to_string(),
            dispatch_id: "dispatch:h5-level-b2:mario-test:codex-dev:write-probe:v1".to_string(),
            actor_id: "project_director".to_string(),
            operation_id: Some("resume".to_string()),
            session_id: Some(session_id.clone()),
            target_cwd: Some(write_root.clone()),
            sandbox: Some("workspace-write".to_string()),
            prompt_summary:
                "H5 Level B2 project-workflow-bound workspace-write dispatch probe for mario test codex-dev worker."
                    .to_string(),
            prompt_ref: "workbench-managed:h5-level-b2:mario-test:codex-dev:write-probe:v1"
                .to_string(),
            prompt_sha256: prompt_sha256.clone(),
            h3_b_level_b_authorized: false,
            expected_workflow_revision: Some(1),
            diagnostic_summary: Some(H5DiagnosticSummaryInput {
                overall_severity: "ok".to_string(),
                blocked_count: 0,
                degraded_states: vec![],
            }),
        };
        let preview = crate::h5_project_dispatch_bridge::preview_h5_project_workflow_dispatch_at(
            &workflow_state_path,
            &preview_request,
        )
        .unwrap();
        assert_eq!(
            preview.memory_packet.fingerprint.as_deref(),
            Some("h5-level-b2-memory-fingerprint-mario-test-codex-dev-2026-06-08")
        );
        assert!(!preview
            .blocked_reasons
            .contains(&"task_memory_packet_stale".to_string()));
        assert!(!preview
            .blocked_reasons
            .contains(&"diagnostics_blocking_degraded".to_string()));
        assert!(!preview
            .blocked_reasons
            .contains(&"duplicate_dispatch_blocked".to_string()));

        let confirm = confirm_continuation(
            &workflow_state_path,
            &ConfirmControlledSessionContinuationInput {
                preview: h5_level_b2_session_preview(&project_root, &write_root, &session_id),
                confirmed_by: "user_and_global_supervisor".to_string(),
                confirmation_reason: "用户和全局主管授权 H5-Level-B2 mario test 开发线 worker session 的一次 workspace-write project workflow write probe。".to_string(),
                expected_store_revision: Some(0),
            },
            "2026-06-08T01:30:00Z",
            "write-h5-level-b2-real-confirm",
        )
        .unwrap();
        let authorization = H2RealResumeAuthorizationMatrix {
            operation_type: "resume".to_string(),
            test_project: "mario test H5-Level-B2 project workflow write probe".to_string(),
            project_root: project_root.clone(),
            target_cwd: write_root.clone(),
            target_session: session_id.clone(),
            prompt_summary:
                "H5 Level B2 project-workflow-bound workspace-write dispatch probe for mario test codex-dev worker."
                    .to_string(),
            prompt_sha256,
            prompt_ref: "workbench-managed:h5-level-b2:mario-test:codex-dev:write-probe:v1"
                .to_string(),
            allowed_write_roots: vec![write_root.clone()],
            codex_home_scope: "Codex CLI minimum session state for one authorized H5-Level-B2 resume; no credential material requested."
                .to_string(),
            sandbox: "workspace-write".to_string(),
            timeout_ms: Some(120_000),
            readback_plan:
                "workbench-managed last message plus continuation/runtime/audit refs; unavailable is not zero"
                    .to_string(),
            evidence_path:
                "/Users/yoyi/workspace/product-line/evidence/2026-06-08-stage-h-h5-level-b2-mario-test-project-workflow-write-probe-v1.md"
                    .to_string(),
            rollback_plan:
                "hash mario test core files before and after; only .workbench/h5-b2 probe file is authorized"
                    .to_string(),
            user_confirmed_real_resume: true,
            global_supervisor_confirmed: true,
        };
        let run_input = RunControlledSessionContinuationRealResumePhaseBInput {
            continuation_id: confirm.continuation.continuation_id.clone(),
            actor_role: "project_director".to_string(),
            expected_store_revision: Some(1),
            authorization,
            execution_decision: Some("approved_for_phase_b".to_string()),
            prompt_body,
        };
        let run = run_real_resume_phase_b(
            &workflow_state_path,
            &run_input,
            "2026-06-08T01:31:00Z",
            "write-h5-level-b2-real-run",
        )
        .unwrap();
        let last_message_path =
            h2_phase_b_last_message_path(&workflow_state_path, &run_input, "2026-06-08T01:31:00Z")
                .unwrap();

        let post_core_hashes = mario_core_file_hashes(&project_root_path);
        let post_project_files = project_file_set(&project_root_path);
        let new_project_files = post_project_files
            .difference(&pre_project_files)
            .cloned()
            .collect::<Vec<_>>();
        let probe_body = fs::read_to_string(&probe_path).expect("read H5-B2 probe file");
        let probe_hash = sha256_file(&probe_path);

        println!(
            "H5_B2_WORKFLOW_STATE_PATH={}",
            workflow_state_path.display()
        );
        println!(
            "H5_B2_SESSION_CONTINUATION_STORE_PATH={}",
            dir.join("session-continuations.v1.json").display()
        );
        println!(
            "H5_B2_RUNTIME_LOG_PATH={}",
            dir.join("runtime-logs.v1.json").display()
        );
        println!("H5_B2_LAST_MESSAGE_PATH={}", last_message_path.display());
        println!("H5_B2_PROBE_PATH={}", probe_path.display());
        println!("H5_B2_PROBE_PRE_STATE={probe_pre_state}");
        println!("H5_B2_PROBE_SHA256={probe_hash}");
        println!("H5_B2_NEW_PROJECT_FILES={}", new_project_files.join(","));
        println!("H5_B2_AUTHORIZATION_STATUS={}", run.authorization_status);
        println!("H5_B2_ATTEMPT_STATUS={}", run.attempt.status);
        assert_eq!(run.authorization_status, "phase_b_real_resume_executed");
        assert_eq!(run.attempt.status, "succeeded");
        assert!(run.attempt.prompt_sent);
        assert!(run.attempt.real_codex_executed);
        assert!(run.attempt.writes_codex_home);
        assert_eq!(run.attempt.readback_summary.result_count, Some(1));
        assert_eq!(
            run.codex_local_request
                .as_ref()
                .map(|request| request.sandbox.as_str()),
            Some("workspace-write")
        );
        assert_eq!(
            run.codex_local_request
                .as_ref()
                .and_then(|request| request.work_item_id.as_deref()),
            Some("work-item:h5-level-b2:mario-test:codex-dev:write-probe:v1")
        );
        assert_eq!(pre_core_hashes, post_core_hashes);
        assert!(probe_body.contains(&expected_marker));
        assert!(probe_body.contains(".workbench/h5-b2/real-dispatch-write-probe.md"));
        assert!(new_project_files
            .iter()
            .all(|path| path.starts_with(".workbench/h5-b2/")));
        let last_message = fs::read_to_string(&last_message_path).unwrap();
        assert!(last_message.contains(&expected_marker));
        let serialized_store = serde_json::to_string(
            &load_store(&workflow_state_path, "2026-06-08T01:32:00Z").unwrap(),
        )
        .unwrap();
        assert!(!serialized_store.contains(&expected_marker));
    }

    fn sha256_file(path: &Path) -> String {
        let bytes = fs::read(path).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        format!("{:x}", hasher.finalize())
    }

    fn mario_core_file_hashes(project_root: &Path) -> Vec<(String, String)> {
        ["index.html", "styles.css", "game.js", "README.md"]
            .iter()
            .map(|file| {
                let path = project_root.join(file);
                ((*file).to_string(), sha256_file(&path))
            })
            .collect()
    }

    fn project_file_set(project_root: &Path) -> std::collections::BTreeSet<String> {
        fn visit(root: &Path, dir: &Path, files: &mut std::collections::BTreeSet<String>) {
            let entries = match fs::read_dir(dir) {
                Ok(entries) => entries,
                Err(_) => return,
            };
            for entry in entries.flatten() {
                let path = entry.path();
                let relative = path.strip_prefix(root).unwrap_or(&path);
                let relative_text = relative.to_string_lossy().replace('\\', "/");
                if relative_text.starts_with(".git/") || relative_text == ".git" {
                    continue;
                }
                if path.is_dir() {
                    visit(root, &path, files);
                } else if path.is_file() {
                    files.insert(relative_text);
                }
            }
        }

        let mut files = std::collections::BTreeSet::new();
        visit(project_root, project_root, &mut files);
        files
    }

    fn safe_preview(status: &str, adapter_id: &str) -> SessionContinuationPreview {
        let provider = ProviderAvailabilitySummary {
            adapter_id: adapter_id.to_string(),
            provider_id: "local-codex-cli".to_string(),
            provider_label: "Codex 本地 CLI".to_string(),
            provider_kind: "local_cli".to_string(),
            adapter_status: "available".to_string(),
            availability_status: "available_readonly".to_string(),
            credential_status: "not_required_by_workbench".to_string(),
            model_status: "local_cli_managed".to_string(),
            external_call_status: "not_needed_for_readonly".to_string(),
            cost_risk_status: "unknown".to_string(),
            user_visible_reason: "只读摘要".to_string(),
            safe_to_display: true,
            requires_user_configuration: false,
            requires_future_task: false,
            warnings: vec![],
        };
        let request = SessionContinuationRequest {
            adapter_id: adapter_id.to_string(),
            operation_id: "resume".to_string(),
            project_id: Some("project:offline".to_string()),
            project_root: Some("/tmp/offline-project".to_string()),
            workflow_id: Some("workflow:offline".to_string()),
            node_id: Some("node:dev".to_string()),
            session_id: Some("thread:offline".to_string()),
            work_item_id: Some("work-item:offline".to_string()),
            target_cwd: Some("/tmp/offline-project".to_string()),
            allowed_write_roots: vec!["/tmp/offline-project".to_string()],
            sandbox: "workspace-write-preview-only".to_string(),
            prompt_source_kind: "task_package_summary".to_string(),
            prompt_summary: "E4 preview summary".to_string(),
            readback_strategy: "required".to_string(),
            requested_by: "workbench_e4_preview".to_string(),
            user_confirmation_state: "missing".to_string(),
        };
        SessionContinuationPreview {
            preview_id: format!("session-continuation-preview:{adapter_id}:resume:binding"),
            adapter_id: adapter_id.to_string(),
            operation_id: "resume".to_string(),
            target_session_id: Some("thread:offline".to_string()),
            target_session_title: Some("Offline session".to_string()),
            project_id: Some("project:offline".to_string()),
            project_root: Some("/tmp/offline-project".to_string()),
            workflow_id: Some("workflow:offline".to_string()),
            node_id: Some("node:dev".to_string()),
            binding_id: Some("binding:offline".to_string()),
            work_item_id: Some("work:offline".to_string()),
            target_cwd: Some("/tmp/offline-project".to_string()),
            allowed_write_roots_summary: vec!["/tmp/offline-project".to_string()],
            sandbox_summary: "workspace-write-preview-only".to_string(),
            prompt_source_kind: "task_package_summary".to_string(),
            prompt_summary: "E4 preview summary".to_string(),
            readback_expectation: ReadbackExpectation {
                strategy: "required".to_string(),
                required: true,
                expected_sources: vec!["future_e5_attempt_audit".to_string()],
                unavailable_behavior: "unavailable 不等于空读回结果".to_string(),
                warnings: vec![],
            },
            failure_handling: ContinuationFailureBoundary {
                timeout_policy: "deferred".to_string(),
                retry_policy: "no_retry".to_string(),
                failure_record: "stub".to_string(),
                user_visible_behavior: "stub".to_string(),
                warnings: vec![],
            },
            audit_impact: ContinuationAuditImpact {
                impact_kind: "preview_only_no_execution".to_string(),
                writes_attempt_in_e4: false,
                writes_dispatch_in_e4: false,
                writes_readback_in_e4: false,
                future_audit_requirement: "E5 writes continuation record".to_string(),
                warnings: vec![],
            },
            provider_availability_summary: Some(provider),
            guard_result: SessionContinuationGuardResult {
                status: status.to_string(),
                severity: "medium".to_string(),
                blocks_execution: true,
                allows_preview: status != "blocked",
                requires_user_confirmation: status == "needs_user_confirmation",
                reasons: if adapter_id == "codex-local" {
                    vec!["user_confirmation_required_before_execution".to_string()]
                } else {
                    vec!["planned_adapter_blocked:claude-code".to_string()]
                },
                required_fixes: vec![],
                warnings: vec![],
            },
            request,
            user_visible_warnings: vec!["session_continuation_preview_only".to_string()],
        }
    }

    fn h3_new_session_preview(project_root: &str) -> SessionContinuationPreview {
        let mut preview = safe_preview("needs_user_confirmation", "codex-local");
        preview.preview_id =
            "session-continuation-preview:codex-local:new-session:h3-b".to_string();
        preview.operation_id = "new_session".to_string();
        preview.target_session_id = None;
        preview.target_session_title = Some("H3-B new session candidate".to_string());
        preview.project_id = Some("project:h3-b-fixture".to_string());
        preview.project_root = Some(project_root.to_string());
        preview.workflow_id = Some("workflow:h3-b-fixture".to_string());
        preview.node_id = Some("node:h3-b-new-session".to_string());
        preview.binding_id = Some("binding:h3-b-fixture".to_string());
        preview.work_item_id = Some("work-item:h3-b".to_string());
        preview.target_cwd = Some(project_root.to_string());
        preview.allowed_write_roots_summary = vec![project_root.to_string()];
        preview.sandbox_summary = "workspace-write".to_string();
        preview.prompt_source_kind = "h3_new_session_task_package".to_string();
        preview.prompt_summary = "H3 real new session safe probe".to_string();
        preview.request.operation_id = "new_session".to_string();
        preview.request.project_id = Some("project:h3-b-fixture".to_string());
        preview.request.project_root = Some(project_root.to_string());
        preview.request.workflow_id = Some("workflow:h3-b-fixture".to_string());
        preview.request.node_id = Some("node:h3-b-new-session".to_string());
        preview.request.session_id = None;
        preview.request.work_item_id = Some("work-item:h3-b".to_string());
        preview.request.target_cwd = Some(project_root.to_string());
        preview.request.allowed_write_roots = vec![project_root.to_string()];
        preview.request.sandbox = "workspace-write".to_string();
        preview.request.prompt_source_kind = "h3_new_session_task_package".to_string();
        preview.request.prompt_summary = "H3 real new session safe probe".to_string();
        preview.request.requested_by = "workbench_h3_b_fixture".to_string();
        preview
    }

    fn h5_level_b1_session_preview(
        project_root: &str,
        session_id: &str,
    ) -> SessionContinuationPreview {
        let mut preview = safe_preview("needs_user_confirmation", "codex-local");
        preview.preview_id =
            "session-continuation-preview:h5-level-b1:mario-test:codex-dev:resume".to_string();
        preview.operation_id = "resume".to_string();
        preview.target_session_id = Some(session_id.to_string());
        preview.target_session_title = Some("mario test Codex 开发线".to_string());
        preview.project_id = Some("project:users-yoyi-documents-mario-test".to_string());
        preview.project_root = Some(project_root.to_string());
        preview.workflow_id = Some("workflow:users-yoyi-documents-mario-test:default".to_string());
        preview.node_id =
            Some("workflow:users-yoyi-documents-mario-test:default:node:codex-dev".to_string());
        preview.binding_id = Some(
            "binding:workflow:users-yoyi-documents-mario-test:default:node:codex-dev".to_string(),
        );
        preview.work_item_id =
            Some("work-item:h5-level-b1:mario-test:codex-dev:read-only-probe:v1".to_string());
        preview.target_cwd = Some(project_root.to_string());
        preview.allowed_write_roots_summary = vec![project_root.to_string()];
        preview.sandbox_summary = "read-only".to_string();
        preview.prompt_source_kind =
            "workbench_managed_h5_level_b1_task_package_prompt_ref".to_string();
        preview.prompt_summary =
            "H5 Level B1 project-workflow-bound read-only dispatch probe for mario test codex-dev worker."
                .to_string();
        preview.readback_expectation.expected_sources = vec![
            "workbench_managed_last_message".to_string(),
            "session_continuation_attempt".to_string(),
            "runtime_log_ref".to_string(),
        ];
        preview.readback_expectation.unavailable_behavior =
            "readback unavailable/failed/timed_out keeps result_count=null".to_string();
        preview.request.operation_id = "resume".to_string();
        preview.request.project_id = Some("project:users-yoyi-documents-mario-test".to_string());
        preview.request.project_root = Some(project_root.to_string());
        preview.request.workflow_id =
            Some("workflow:users-yoyi-documents-mario-test:default".to_string());
        preview.request.node_id =
            Some("workflow:users-yoyi-documents-mario-test:default:node:codex-dev".to_string());
        preview.request.session_id = Some(session_id.to_string());
        preview.request.work_item_id =
            Some("work-item:h5-level-b1:mario-test:codex-dev:read-only-probe:v1".to_string());
        preview.request.target_cwd = Some(project_root.to_string());
        preview.request.allowed_write_roots = vec![project_root.to_string()];
        preview.request.sandbox = "read-only".to_string();
        preview.request.prompt_source_kind =
            "workbench_managed_h5_level_b1_task_package_prompt_ref".to_string();
        preview.request.prompt_summary =
            "H5 Level B1 project-workflow-bound read-only dispatch probe for mario test codex-dev worker."
                .to_string();
        preview.request.requested_by = "workbench_h5_level_b1_project_director".to_string();
        preview
    }

    fn h5_level_b2_session_preview(
        project_root: &str,
        write_root: &str,
        session_id: &str,
    ) -> SessionContinuationPreview {
        let mut preview = safe_preview("needs_user_confirmation", "codex-local");
        preview.preview_id =
            "session-continuation-preview:h5-level-b2:mario-test:codex-dev:resume".to_string();
        preview.operation_id = "resume".to_string();
        preview.target_session_id = Some(session_id.to_string());
        preview.target_session_title = Some("mario test Codex 开发线".to_string());
        preview.project_id = Some("project:users-yoyi-documents-mario-test".to_string());
        preview.project_root = Some(project_root.to_string());
        preview.workflow_id = Some("workflow:users-yoyi-documents-mario-test:default".to_string());
        preview.node_id =
            Some("workflow:users-yoyi-documents-mario-test:default:node:codex-dev".to_string());
        preview.binding_id = Some(
            "binding:workflow:users-yoyi-documents-mario-test:default:node:codex-dev".to_string(),
        );
        preview.work_item_id =
            Some("work-item:h5-level-b2:mario-test:codex-dev:write-probe:v1".to_string());
        preview.target_cwd = Some(write_root.to_string());
        preview.allowed_write_roots_summary = vec![write_root.to_string()];
        preview.sandbox_summary = "workspace-write".to_string();
        preview.prompt_source_kind =
            "workbench_managed_h5_level_b2_task_package_prompt_ref".to_string();
        preview.prompt_summary =
            "H5 Level B2 project-workflow-bound workspace-write dispatch probe for mario test codex-dev worker."
                .to_string();
        preview.readback_expectation.expected_sources = vec![
            "workbench_managed_last_message".to_string(),
            "session_continuation_attempt".to_string(),
            "runtime_log_ref".to_string(),
        ];
        preview.readback_expectation.unavailable_behavior =
            "readback unavailable/failed/timed_out keeps result_count=null".to_string();
        preview.request.operation_id = "resume".to_string();
        preview.request.project_id = Some("project:users-yoyi-documents-mario-test".to_string());
        preview.request.project_root = Some(project_root.to_string());
        preview.request.workflow_id =
            Some("workflow:users-yoyi-documents-mario-test:default".to_string());
        preview.request.node_id =
            Some("workflow:users-yoyi-documents-mario-test:default:node:codex-dev".to_string());
        preview.request.session_id = Some(session_id.to_string());
        preview.request.work_item_id =
            Some("work-item:h5-level-b2:mario-test:codex-dev:write-probe:v1".to_string());
        preview.request.target_cwd = Some(write_root.to_string());
        preview.request.allowed_write_roots = vec![write_root.to_string()];
        preview.request.sandbox = "workspace-write".to_string();
        preview.request.prompt_source_kind =
            "workbench_managed_h5_level_b2_task_package_prompt_ref".to_string();
        preview.request.prompt_summary =
            "H5 Level B2 project-workflow-bound workspace-write dispatch probe for mario test codex-dev worker."
                .to_string();
        preview.request.requested_by = "workbench_h5_level_b2_project_director".to_string();
        preview
    }

    fn write_h5_level_b1_workflow_state(
        workflow_state_path: &Path,
        project_root: &str,
        session_id: &str,
    ) {
        let snapshot = serde_json::json!({
            "snapshot_id": "task-memory-packet-snapshot:h5-level-b1:mario-test:codex-dev:2026-06-08",
            "schema_version": "task_package_memory_packet_snapshot.v1",
            "source_packet_id": "task-memory-packet:h5-level-b1:mario-test:codex-dev:2026-06-08",
            "project_id": "project:users-yoyi-documents-mario-test",
            "workflow_id": "workflow:users-yoyi-documents-mario-test:default",
            "work_item_id": "work-item:h5-level-b1:mario-test:codex-dev:read-only-probe:v1",
            "task_package_artifact_id": "artifact:h5-level-b1:mario-test:codex-dev:read-only-probe:v1",
            "role_id": "codex-dev",
            "retrieval_intent": "worker_task",
            "included_memories": [],
            "excluded_items": [],
            "review_materials": [],
            "store_revisions": {
                "formal_store_revision": 0,
                "candidate_store_revision": 0,
                "observation_store_revision": 0,
                "lint_store_revision": 0,
                "entity_relation_store_revision": 0
            },
            "estimated_tokens": 0,
            "max_estimated_tokens": 2000,
            "fingerprint": "h5-level-b1-memory-fingerprint-mario-test-codex-dev-2026-06-08",
            "generated_at": "2026-06-08T00:29:00Z",
            "stale": false,
            "stale_reasons": [],
            "warnings": []
        });
        let value = serde_json::json!({
            "revision": 1,
            "workflow_node_dispatches": [{
                "dispatch_id": "dispatch:h5-level-b1:mario-test:codex-dev:read-only-probe:v1",
                "project_id": "project:users-yoyi-documents-mario-test",
                "workflow_id": "workflow:users-yoyi-documents-mario-test:default",
                "node_id": "workflow:users-yoyi-documents-mario-test:default:node:codex-dev",
                "work_item_id": "work-item:h5-level-b1:mario-test:codex-dev:read-only-probe:v1",
                "native_thread_id": session_id,
                "prompt_preview": "redacted prompt preview; body is workbench-managed and sent via stdin only",
                "prompt_kind": "h5_level_b1_authorized_project_workflow_dispatch",
                "memory_packet_snapshot_id": "task-memory-packet-snapshot:h5-level-b1:mario-test:codex-dev:2026-06-08",
                "memory_packet_fingerprint": "h5-level-b1-memory-fingerprint-mario-test-codex-dev-2026-06-08",
                "plan_authorization_id": "authorization:h5-level-b1:mario-test:codex-dev:read-only-probe:v1",
                "task_package_id": "artifact:h5-level-b1:mario-test:codex-dev:read-only-probe:v1",
                "authorization_check": {"status": "authorized"},
                "state": "prepared"
            }],
            "artifacts": [{
                "artifact_id": "artifact:h5-level-b1:mario-test:codex-dev:read-only-probe:v1",
                "artifact_type": "task_package",
                "source_ref": "work-item:h5-level-b1:mario-test:codex-dev:read-only-probe:v1",
                "allowed_write": [project_root],
                "memory_packet_snapshot": snapshot
            }]
        });
        fs::write(
            workflow_state_path,
            serde_json::to_string_pretty(&value).unwrap(),
        )
        .unwrap();
    }

    fn write_h5_level_b2_workflow_state(
        workflow_state_path: &Path,
        project_root: &str,
        write_root: &str,
        session_id: &str,
    ) {
        let snapshot = serde_json::json!({
            "snapshot_id": "task-memory-packet-snapshot:h5-level-b2:mario-test:codex-dev:2026-06-08",
            "schema_version": "task_package_memory_packet_snapshot.v1",
            "source_packet_id": "task-memory-packet:h5-level-b2:mario-test:codex-dev:2026-06-08",
            "project_id": "project:users-yoyi-documents-mario-test",
            "workflow_id": "workflow:users-yoyi-documents-mario-test:default",
            "work_item_id": "work-item:h5-level-b2:mario-test:codex-dev:write-probe:v1",
            "task_package_artifact_id": "artifact:h5-level-b2:mario-test:codex-dev:write-probe:v1",
            "role_id": "codex-dev",
            "retrieval_intent": "worker_task",
            "included_memories": [],
            "excluded_items": [],
            "review_materials": [],
            "store_revisions": {
                "formal_store_revision": 0,
                "candidate_store_revision": 0,
                "observation_store_revision": 0,
                "lint_store_revision": 0,
                "entity_relation_store_revision": 0
            },
            "estimated_tokens": 0,
            "max_estimated_tokens": 2000,
            "fingerprint": "h5-level-b2-memory-fingerprint-mario-test-codex-dev-2026-06-08",
            "generated_at": "2026-06-08T01:29:00Z",
            "stale": false,
            "stale_reasons": [],
            "warnings": []
        });
        let value = serde_json::json!({
            "revision": 1,
            "workflow_node_dispatches": [{
                "dispatch_id": "dispatch:h5-level-b2:mario-test:codex-dev:write-probe:v1",
                "project_id": "project:users-yoyi-documents-mario-test",
                "workflow_id": "workflow:users-yoyi-documents-mario-test:default",
                "node_id": "workflow:users-yoyi-documents-mario-test:default:node:codex-dev",
                "work_item_id": "work-item:h5-level-b2:mario-test:codex-dev:write-probe:v1",
                "native_thread_id": session_id,
                "prompt_preview": "redacted prompt preview; body is workbench-managed and sent via stdin only",
                "prompt_kind": "h5_level_b2_authorized_project_workflow_write_probe",
                "memory_packet_snapshot_id": "task-memory-packet-snapshot:h5-level-b2:mario-test:codex-dev:2026-06-08",
                "memory_packet_fingerprint": "h5-level-b2-memory-fingerprint-mario-test-codex-dev-2026-06-08",
                "plan_authorization_id": "authorization:h5-level-b2:mario-test:codex-dev:write-probe:v1",
                "task_package_id": "artifact:h5-level-b2:mario-test:codex-dev:write-probe:v1",
                "authorization_check": {"status": "authorized"},
                "state": "prepared"
            }],
            "artifacts": [{
                "artifact_id": "artifact:h5-level-b2:mario-test:codex-dev:write-probe:v1",
                "artifact_type": "task_package",
                "source_ref": "work-item:h5-level-b2:mario-test:codex-dev:write-probe:v1",
                "allowed_write": [write_root],
                "memory_packet_snapshot": snapshot
            }],
            "projects": [{
                "project_id": "project:users-yoyi-documents-mario-test",
                "project_root": project_root
            }]
        });
        fs::write(
            workflow_state_path,
            serde_json::to_string_pretty(&value).unwrap(),
        )
        .unwrap();
    }

    fn incomplete_authorization() -> H2RealResumeAuthorizationMatrix {
        H2RealResumeAuthorizationMatrix {
            operation_type: "resume".to_string(),
            test_project: "".to_string(),
            project_root: "/tmp/offline-project".to_string(),
            target_cwd: "/tmp/offline-project".to_string(),
            target_session: "thread:offline".to_string(),
            prompt_summary: "H2 preflight summary".to_string(),
            prompt_sha256: "not-a-valid-hash".to_string(),
            prompt_ref: "prompt-ref:h2".to_string(),
            allowed_write_roots: vec!["/tmp/offline-project".to_string()],
            codex_home_scope: "".to_string(),
            sandbox: "workspace-write".to_string(),
            timeout_ms: None,
            readback_plan: "workbench-managed last message".to_string(),
            evidence_path: "/tmp/offline-project/evidence/h2.md".to_string(),
            rollback_plan: "hash before and after".to_string(),
            user_confirmed_real_resume: false,
            global_supervisor_confirmed: false,
        }
    }

    fn complete_authorization() -> H2RealResumeAuthorizationMatrix {
        H2RealResumeAuthorizationMatrix {
            operation_type: "resume".to_string(),
            test_project: "offline h2 fixture".to_string(),
            project_root: "/tmp/offline-project".to_string(),
            target_cwd: "/tmp/offline-project".to_string(),
            target_session: "thread:offline".to_string(),
            prompt_summary: "H2 preflight summary".to_string(),
            prompt_sha256: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                .to_string(),
            prompt_ref: "prompt-ref:h2".to_string(),
            allowed_write_roots: vec!["/tmp/offline-project".to_string()],
            codex_home_scope:
                "codex resume minimum session state only; no auth token or full transcript"
                    .to_string(),
            sandbox: "workspace-write".to_string(),
            timeout_ms: Some(30_000),
            readback_plan: "workbench-managed last message; unavailable is not zero".to_string(),
            evidence_path: "/tmp/offline-project/evidence/h2.md".to_string(),
            rollback_plan: "hash before and after".to_string(),
            user_confirmed_real_resume: true,
            global_supervisor_confirmed: true,
        }
    }

    fn complete_h3_b_authorization(
        project_root: &str,
        prompt_body: &str,
    ) -> H3RealNewSessionAuthorizationMatrix {
        H3RealNewSessionAuthorizationMatrix {
            operation_type: "new_session".to_string(),
            test_project: "offline h3-b fixture".to_string(),
            project_root: project_root.to_string(),
            target_cwd: project_root.to_string(),
            work_item_id: "work-item:h3-b".to_string(),
            prompt_summary: "H3 real new session safe probe".to_string(),
            prompt_sha256: sha256_hex(prompt_body),
            prompt_ref: "workbench-managed:h3-real-new-session-safe-probe:v1".to_string(),
            allowed_write_roots: vec![project_root.to_string()],
            codex_home_scope:
                "codex new session minimum state only; no auth token or full transcript".to_string(),
            sandbox: "workspace-write".to_string(),
            timeout_ms: Some(30_000),
            readback_plan: "workbench-managed last message; unavailable is not zero".to_string(),
            evidence_path: format!("{project_root}/evidence/h3-b.md"),
            rollback_plan: "hash fixture files before and after".to_string(),
            user_confirmed_real_new_session: true,
            global_supervisor_confirmed: true,
        }
    }

    fn temp_test_dir(prefix: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("{prefix}-{unique}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }
}
