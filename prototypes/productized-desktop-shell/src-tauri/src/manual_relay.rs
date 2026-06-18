use serde::{Deserialize, Serialize};
use std::collections::{btree_map::Entry, BTreeMap};
use std::fs;
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Mutex, OnceLock};

use crate::utils::hash::{sha256_hex, short_hash};

const MANUAL_RELAY_REAL_CODEX_CONFIRM_ENV: &str = "MANUAL_RELAY_REAL_CODEX_CONFIRM";
const MANUAL_RELAY_REAL_CODEX_CONFIRM_VALUE: &str = "CONFIRMED_USER_PRESENT_REAL_RELAY";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct ManualRelayPreviewInput {
    pub(crate) original_user_text: String,
    pub(crate) target_project_root: String,
    pub(crate) target_cwd: String,
    pub(crate) target_session_id: Option<String>,
    pub(crate) new_session: bool,
    pub(crate) sandbox: String,
    pub(crate) allowed_write_roots: Vec<String>,
    pub(crate) requested_by: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct ManualRelayEnvelope {
    pub(crate) relay_id: String,
    pub(crate) target_binding: ManualRelayTargetBinding,
    pub(crate) payload: ManualRelayPayload,
    pub(crate) policy: ManualRelayPolicy,
    pub(crate) future_hooks: ManualRelayFutureHooks,
    pub(crate) audit_refs: Vec<String>,
    pub(crate) receipt_refs: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct ManualRelayTargetBinding {
    pub(crate) project_root_canonical: String,
    pub(crate) target_cwd_canonical: String,
    pub(crate) target_session_id: Option<String>,
    pub(crate) new_session: bool,
    pub(crate) sandbox: String,
    pub(crate) allowed_write_roots: Vec<String>,
    pub(crate) target_hash: String,
    pub(crate) path_verified: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct ManualRelayPayload {
    pub(crate) original_user_text: String,
    pub(crate) effective_prompt: String,
    pub(crate) payload_layers: Vec<String>,
    pub(crate) prompt_sha256: String,
    pub(crate) prompt_length_bytes: i64,
    pub(crate) exact_original: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct ManualRelayPolicy {
    pub(crate) manual_once: bool,
    pub(crate) auto_chain: bool,
    pub(crate) duplicate_scope: String,
    pub(crate) denied_material_policy: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct ManualRelayFutureHooks {
    pub(crate) role_id: Option<String>,
    pub(crate) task_package_ref: Option<String>,
    pub(crate) memory_packet_ref: Option<String>,
    pub(crate) supervisor_review_ref: Option<String>,
    pub(crate) post_run_memory_capture_policy: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct ManualRelayCommandPlan {
    pub(crate) program: String,
    pub(crate) argv: Vec<String>,
    pub(crate) stdin_prompt_ref: String,
    pub(crate) stdin_prompt_sha256: String,
    pub(crate) prompt_in_command: bool,
    pub(crate) shell_invocation: bool,
    pub(crate) redacted_preview: String,
    pub(crate) last_message_path: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct ManualRelayGuard {
    pub(crate) status: String,
    pub(crate) blocks_execution: bool,
    pub(crate) reasons: Vec<String>,
    pub(crate) warnings: Vec<String>,
    pub(crate) command_plan: Option<ManualRelayCommandPlan>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct ManualRelayPreview {
    pub(crate) envelope: ManualRelayEnvelope,
    pub(crate) guard: ManualRelayGuard,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct ManualRelayConfirmInput {
    pub(crate) envelope: ManualRelayEnvelope,
    pub(crate) actor_ref: String,
    pub(crate) target_hash: String,
    pub(crate) prompt_sha256: String,
    pub(crate) sandbox: String,
    pub(crate) allowed_write_roots: Vec<String>,
    pub(crate) risk_acknowledged: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct ManualRelayConfirmation {
    pub(crate) confirmation_id: String,
    pub(crate) relay_id: String,
    pub(crate) prompt_sha256: String,
    pub(crate) target_hash: String,
    pub(crate) sandbox: String,
    pub(crate) allowed_write_roots: Vec<String>,
    pub(crate) manual_once: bool,
    pub(crate) auto_chain: bool,
    pub(crate) confirmed_by: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct ManualRelayRunInput {
    pub(crate) envelope: ManualRelayEnvelope,
    pub(crate) confirmation: ManualRelayConfirmation,
    pub(crate) confirmation_id: String,
    pub(crate) expected_prompt_sha256: String,
    pub(crate) expected_target_hash: String,
    pub(crate) expected_sandbox: String,
    pub(crate) expected_allowed_write_roots: Vec<String>,
    pub(crate) mock_behavior: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct ManualRelayGuiDirectRunInput {
    pub(crate) original_user_text: String,
    pub(crate) target_project_root: String,
    pub(crate) target_cwd: String,
    pub(crate) target_session_id: String,
    pub(crate) sandbox: String,
    pub(crate) allowed_write_roots: Vec<String>,
    pub(crate) requested_by: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct ManualRelayReceipt {
    pub(crate) relay_attempt_id: String,
    pub(crate) confirmation_id: String,
    pub(crate) target: ManualRelayTargetBinding,
    pub(crate) effective_prompt_sha256: String,
    pub(crate) prompt_length_bytes: i64,
    pub(crate) prompt_exact_original: bool,
    pub(crate) command_plan: ManualRelayCommandPlan,
    pub(crate) started_at: String,
    pub(crate) ended_at: Option<String>,
    pub(crate) exit_code: Option<i32>,
    pub(crate) process_id: Option<u32>,
    pub(crate) process_kind: String,
    pub(crate) real_process_killed: bool,
    pub(crate) status: String,
    pub(crate) prompt_sent: bool,
    pub(crate) real_codex_executed: bool,
    pub(crate) syn_read_codex_home: bool,
    pub(crate) syn_wrote_codex_home: bool,
    pub(crate) killed_by_user: bool,
    pub(crate) timed_out: bool,
    pub(crate) readback_status: String,
    pub(crate) last_message_hash: Option<String>,
    pub(crate) last_message_size_bytes: Option<i64>,
    pub(crate) changed_files: Vec<String>,
    pub(crate) git_head_before: Option<String>,
    pub(crate) git_head_after: Option<String>,
    pub(crate) git_status_before: String,
    pub(crate) git_status_after: String,
    pub(crate) rollback: ManualRelayRollbackSummary,
    pub(crate) warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct ManualRelayRollbackSummary {
    pub(crate) git_available: bool,
    pub(crate) dirty_before: bool,
    pub(crate) auto_rollback_performed: bool,
    pub(crate) rollback_suggestion_available: bool,
    pub(crate) summary: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct ManualRelayStopInput {
    pub(crate) relay_attempt_id: String,
    pub(crate) requested_by: String,
}

pub(crate) fn preview_manual_relay(
    input: ManualRelayPreviewInput,
    timestamp: &str,
) -> ManualRelayPreview {
    let project_root = normalize_path_for_preview(&input.target_project_root);
    let target_cwd = normalize_path_for_preview(&input.target_cwd);
    let allowed_write_roots_with_verification = input
        .allowed_write_roots
        .iter()
        .map(|root| normalize_path_for_preview(root))
        .collect::<Vec<_>>();
    let allowed_write_roots = allowed_write_roots_with_verification
        .iter()
        .map(|path| path.normalized.clone())
        .collect::<Vec<_>>();
    let path_verified = project_root.verified
        && target_cwd.verified
        && allowed_write_roots_with_verification
            .iter()
            .all(|path| path.verified);
    let prompt_sha256 = sha256_hex(&input.original_user_text);
    let target_hash = target_hash_for_binding(
        &project_root.normalized,
        &target_cwd.normalized,
        input.target_session_id.as_deref(),
        input.new_session,
        &input.sandbox,
    );
    let relay_id = format!(
        "manual-relay:{}:{}",
        timestamp.replace([':', '.', 'Z'], ""),
        short_hash(&format!("{target_hash}:{prompt_sha256}"))
    );
    let duplicate_scope = format!(
        "manual-relay:{}:{}",
        short_hash(&target_hash),
        input
            .target_session_id
            .as_deref()
            .map(short_hash)
            .unwrap_or_else(|| "new-session".to_string())
    );
    let run_dir = std::env::temp_dir()
        .join("codex-governance-workbench")
        .join("manual-relay-runs")
        .join(short_hash(&relay_id));
    let last_message_path = run_dir.join("last-message.txt");
    let envelope = ManualRelayEnvelope {
        relay_id: relay_id.clone(),
        target_binding: ManualRelayTargetBinding {
            project_root_canonical: project_root.normalized.clone(),
            target_cwd_canonical: target_cwd.normalized.clone(),
            target_session_id: input.target_session_id.clone(),
            new_session: input.new_session,
            sandbox: input.sandbox.clone(),
            allowed_write_roots: allowed_write_roots.clone(),
            target_hash,
            path_verified,
        },
        payload: ManualRelayPayload {
            original_user_text: input.original_user_text.clone(),
            effective_prompt: input.original_user_text.clone(),
            payload_layers: Vec::new(),
            prompt_sha256: prompt_sha256.clone(),
            prompt_length_bytes: input.original_user_text.as_bytes().len() as i64,
            exact_original: true,
        },
        policy: ManualRelayPolicy {
            manual_once: true,
            auto_chain: false,
            duplicate_scope,
            denied_material_policy:
                "deny_secret_token_env_keychain_oauth_credential_full_transcript_rollout_codex_home"
                    .to_string(),
        },
        future_hooks: ManualRelayFutureHooks {
            role_id: None,
            task_package_ref: None,
            memory_packet_ref: None,
            supervisor_review_ref: None,
            post_run_memory_capture_policy: None,
        },
        audit_refs: vec![format!(
            "audit:manual-relay-preview:{}",
            short_hash(&relay_id)
        )],
        receipt_refs: Vec::new(),
    };
    let guard = inspect_manual_relay_guard(&envelope, &input.requested_by, &last_message_path);
    ManualRelayPreview { envelope, guard }
}

pub(crate) fn confirm_manual_relay_once(
    input: ManualRelayConfirmInput,
    timestamp: &str,
) -> Result<ManualRelayConfirmation, String> {
    if !input.risk_acknowledged {
        return Err("manual_relay_risk_acknowledgement_required".to_string());
    }
    if input.target_hash != input.envelope.target_binding.target_hash {
        return Err("manual_relay_target_hash_mismatch".to_string());
    }
    if input.prompt_sha256 != input.envelope.payload.prompt_sha256 {
        return Err("manual_relay_prompt_hash_mismatch".to_string());
    }
    if input.sandbox != input.envelope.target_binding.sandbox {
        return Err("manual_relay_sandbox_mismatch".to_string());
    }
    if input.allowed_write_roots != input.envelope.target_binding.allowed_write_roots {
        return Err("manual_relay_allowed_write_roots_mismatch".to_string());
    }
    if !input.envelope.payload.payload_layers.is_empty()
        || input.envelope.payload.effective_prompt != input.envelope.payload.original_user_text
        || !input.envelope.payload.exact_original
    {
        return Err("manual_relay_payload_must_be_exact_original".to_string());
    }
    if !input.envelope.policy.manual_once || input.envelope.policy.auto_chain {
        return Err("manual_relay_policy_must_be_manual_once_without_auto_chain".to_string());
    }
    Ok(ManualRelayConfirmation {
        confirmation_id: format!(
            "manual-relay-confirmation:{}:{}",
            timestamp.replace([':', '.', 'Z'], ""),
            short_hash(&format!(
                "{}:{}:{}",
                input.envelope.relay_id, input.prompt_sha256, input.target_hash
            ))
        ),
        relay_id: input.envelope.relay_id,
        prompt_sha256: input.prompt_sha256,
        target_hash: input.target_hash,
        sandbox: input.sandbox,
        allowed_write_roots: input.allowed_write_roots,
        manual_once: true,
        auto_chain: false,
        confirmed_by: input.actor_ref,
    })
}

pub(crate) fn run_manual_relay_once(
    input: ManualRelayRunInput,
    timestamp: &str,
) -> Result<ManualRelayReceipt, String> {
    run_manual_relay_once_with_process_mode(input, timestamp, None)
}

pub(crate) fn run_manual_relay_gui_direct_once(
    input: ManualRelayGuiDirectRunInput,
    timestamp: &str,
) -> Result<ManualRelayReceipt, String> {
    run_manual_relay_gui_direct_once_with_process_mode(
        input,
        timestamp,
        ManualRelayProcessMode::RealCodexProductGui,
    )
}

#[cfg(test)]
fn run_manual_relay_gui_direct_once_for_test(
    input: ManualRelayGuiDirectRunInput,
    timestamp: &str,
    mock_behavior: &str,
) -> Result<ManualRelayReceipt, String> {
    let Some(process_mode) = manual_relay_process_mode(mock_behavior) else {
        return Err("manual_relay_gui_direct_test_process_mode_required".to_string());
    };
    if matches!(process_mode, ManualRelayProcessMode::RealCodexEnvGated) {
        return Err("manual_relay_gui_direct_test_must_not_use_real_codex".to_string());
    }
    run_manual_relay_gui_direct_once_with_process_mode(input, timestamp, process_mode)
}

fn run_manual_relay_gui_direct_once_with_process_mode(
    input: ManualRelayGuiDirectRunInput,
    timestamp: &str,
    process_mode: ManualRelayProcessMode,
) -> Result<ManualRelayReceipt, String> {
    validate_gui_direct_input(&input)?;
    let preview = preview_manual_relay(
        ManualRelayPreviewInput {
            original_user_text: input.original_user_text,
            target_project_root: input.target_project_root,
            target_cwd: input.target_cwd,
            target_session_id: Some(input.target_session_id),
            new_session: false,
            sandbox: input.sandbox,
            allowed_write_roots: input.allowed_write_roots,
            requested_by: input.requested_by.clone(),
        },
        timestamp,
    );
    if preview.guard.blocks_execution {
        return Err(format!(
            "manual_relay_guard_blocked:{}",
            preview.guard.reasons.join(",")
        ));
    }
    let Some(command_plan) = preview.guard.command_plan.as_ref() else {
        return Err("manual_relay_command_plan_missing".to_string());
    };
    validate_gui_direct_target_and_command_plan(&preview.envelope, command_plan)?;
    let confirmation = confirm_manual_relay_once(
        ManualRelayConfirmInput {
            envelope: preview.envelope.clone(),
            actor_ref: input.requested_by,
            target_hash: preview.envelope.target_binding.target_hash.clone(),
            prompt_sha256: preview.envelope.payload.prompt_sha256.clone(),
            sandbox: preview.envelope.target_binding.sandbox.clone(),
            allowed_write_roots: preview.envelope.target_binding.allowed_write_roots.clone(),
            risk_acknowledged: true,
        },
        timestamp,
    )?;
    let run_input = ManualRelayRunInput {
        envelope: preview.envelope.clone(),
        confirmation: confirmation.clone(),
        confirmation_id: confirmation.confirmation_id.clone(),
        expected_prompt_sha256: preview.envelope.payload.prompt_sha256.clone(),
        expected_target_hash: preview.envelope.target_binding.target_hash.clone(),
        expected_sandbox: preview.envelope.target_binding.sandbox.clone(),
        expected_allowed_write_roots: preview.envelope.target_binding.allowed_write_roots.clone(),
        mock_behavior: "gui_direct_internal_process_mode".to_string(),
    };
    run_manual_relay_once_with_process_mode(run_input, timestamp, Some(process_mode))
}

fn run_manual_relay_once_with_process_mode(
    input: ManualRelayRunInput,
    timestamp: &str,
    process_mode_override: Option<ManualRelayProcessMode>,
) -> Result<ManualRelayReceipt, String> {
    validate_run_binding(&input)?;
    let scope = input.envelope.policy.duplicate_scope.clone();
    let process_mode =
        process_mode_override.or_else(|| manual_relay_process_mode(&input.mock_behavior));
    if matches!(process_mode, Some(ManualRelayProcessMode::PlaceholderSleep))
        && !placeholder_process_mode_allowed()
    {
        return Err("manual_relay_placeholder_process_mode_test_only".to_string());
    }
    if matches!(
        process_mode,
        Some(ManualRelayProcessMode::MockCodexComplete(_))
            | Some(ManualRelayProcessMode::MockCodexSleep(_))
    ) && !mock_codex_process_mode_allowed()
    {
        return Err("manual_relay_mock_codex_process_mode_test_only".to_string());
    }
    if process_mode.is_some() || is_process_mode(&input.mock_behavior) {
        verify_strict_run_paths(&input.envelope)?;
    }
    let last_message_path = std::env::temp_dir()
        .join("codex-governance-workbench")
        .join("manual-relay-runs")
        .join(short_hash(&input.envelope.relay_id))
        .join("last-message.txt");
    let guard = inspect_manual_relay_guard(
        &input.envelope,
        &input.confirmation.confirmed_by,
        &last_message_path,
    );
    if guard.blocks_execution {
        return Err(format!(
            "manual_relay_guard_blocked:{}",
            guard.reasons.join(",")
        ));
    }
    let Some(command_plan) = guard.command_plan.clone() else {
        return Err("manual_relay_command_plan_missing".to_string());
    };

    let attempt_id = format!(
        "manual-relay-attempt:{}:{}",
        timestamp.replace([':', '.', 'Z'], ""),
        short_hash(&format!(
            "{}:{}",
            input.envelope.relay_id, input.confirmation_id
        ))
    );
    let dirty_before = input.mock_behavior.contains("dirty_tree");
    if let Some(process_mode) = process_mode {
        if process_mode == ManualRelayProcessMode::PlaceholderSleep {
            let placeholder_plan = placeholder_command_plan(&last_message_path, &input.envelope);
            let mut registry = active_attempts()
                .lock()
                .map_err(|_| "manual_relay_registry_poisoned".to_string())?;
            if registry
                .values()
                .any(|attempt| attempt.duplicate_scope == scope && attempt.status == "running")
            {
                return Err("manual_relay_duplicate_running_attempt".to_string());
            }
            let mut consumed = consumed_confirmations()
                .lock()
                .map_err(|_| "manual_relay_confirmation_registry_poisoned".to_string())?;
            reserve_confirmation_in_map(&mut consumed, &input.confirmation_id)?;
            let child = spawn_placeholder_process(&placeholder_plan, &input.envelope)?;
            let process_id = Some(child.id());
            let mut receipt = fixture_receipt(
                &attempt_id,
                &input.confirmation_id,
                "running",
                &input.envelope,
                placeholder_plan,
                timestamp,
                false,
                dirty_before,
                None,
            );
            receipt.process_id = process_id;
            receipt.process_kind = "placeholder".to_string();
            receipt
                .warnings
                .push("placeholder_process_spawned_no_codex".to_string());
            receipt.warnings.sort();
            receipt.warnings.dedup();
            registry.insert(
                attempt_id.clone(),
                ActiveManualRelayAttempt {
                    duplicate_scope: scope,
                    status: "running".to_string(),
                    receipt: receipt.clone(),
                    child: Some(child),
                },
            );
            consumed.insert(input.confirmation_id.clone(), attempt_id);
            return Ok(receipt);
        }

        let process_config = process_config_for_mode(process_mode, command_plan)?;
        if process_config.return_running {
            return spawn_running_codex_like_process(
                &scope,
                &input.confirmation_id,
                attempt_id,
                &input.envelope,
                process_config,
                timestamp,
                dirty_before,
            );
        }

        reserve_non_running_attempt_once(&scope, &input.confirmation_id)?;
        let receipt = run_codex_like_process_to_completion(
            &attempt_id,
            &input.confirmation_id,
            &input.envelope,
            process_config,
            timestamp,
            dirty_before,
        )?;
        set_consumed_confirmation_attempt(&input.confirmation_id, &attempt_id)?;
        return Ok(receipt);
    }
    let mut receipt = fixture_receipt(
        &attempt_id,
        &input.confirmation_id,
        if input.mock_behavior.starts_with("stay_running") {
            "running"
        } else {
            "completed_fixture"
        },
        &input.envelope,
        command_plan,
        timestamp,
        false,
        dirty_before,
        Some(sha256_hex("manual relay fixture last message")),
    );
    receipt
        .warnings
        .push("fixture_runner_no_real_codex".to_string());
    receipt
        .warnings
        .push("manual_relay_no_model_call_in_this_package".to_string());
    receipt.warnings.sort();
    receipt.warnings.dedup();

    if receipt.status == "running" {
        register_running_attempt_once(
            &scope,
            &input.confirmation_id,
            attempt_id.clone(),
            ActiveManualRelayAttempt {
                duplicate_scope: scope.clone(),
                status: "running".to_string(),
                receipt: receipt.clone(),
                child: None,
            },
        )?;
    } else {
        reserve_non_running_attempt_once(&scope, &input.confirmation_id)?;
    }
    set_consumed_confirmation_attempt(&input.confirmation_id, &attempt_id)?;

    Ok(receipt)
}

pub(crate) fn stop_manual_relay_attempt(
    input: ManualRelayStopInput,
    timestamp: &str,
) -> Result<ManualRelayReceipt, String> {
    if input.requested_by.trim().is_empty() {
        return Err("manual_relay_stop_requested_by_missing".to_string());
    }
    let mut registry = active_attempts()
        .lock()
        .map_err(|_| "manual_relay_registry_poisoned".to_string())?;
    let Some(active) = registry.remove(&input.relay_attempt_id) else {
        return Err("manual_relay_attempt_not_running".to_string());
    };
    let mut receipt = active.receipt;
    if let Some(mut child) = active.child {
        let kill_result = child.kill();
        let wait_result = child.wait();
        receipt.real_process_killed = kill_result.is_ok() && wait_result.is_ok();
        if let Ok(status) = wait_result {
            receipt.exit_code = status.code();
        }
    }
    receipt.status = "stopped_by_user".to_string();
    receipt.ended_at = Some(timestamp.to_string());
    receipt.killed_by_user = true;
    receipt.syn_read_codex_home = false;
    receipt.syn_wrote_codex_home = false;
    receipt
        .warnings
        .push("manual_relay_stop_killed_only_requested_attempt".to_string());
    receipt.warnings.sort();
    receipt.warnings.dedup();
    Ok(receipt)
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ManualRelayProcessMode {
    PlaceholderSleep,
    MockCodexComplete(PathBuf),
    MockCodexSleep(PathBuf),
    RealCodexEnvGated,
    RealCodexProductGui,
}

struct ManualRelayProcessConfig {
    command_plan: ManualRelayCommandPlan,
    process_kind: String,
    real_codex_executed: bool,
    return_running: bool,
    completed_status: String,
}

fn manual_relay_process_mode(mock_behavior: &str) -> Option<ManualRelayProcessMode> {
    if is_placeholder_process_mode(mock_behavior) {
        return Some(ManualRelayProcessMode::PlaceholderSleep);
    }
    if mock_behavior == "real_codex_env_gated" {
        return Some(ManualRelayProcessMode::RealCodexEnvGated);
    }
    if let Some(path) = mock_behavior.strip_prefix("mock_codex_process_sleep:") {
        return Some(ManualRelayProcessMode::MockCodexSleep(PathBuf::from(path)));
    }
    mock_behavior
        .strip_prefix("mock_codex_process:")
        .map(|path| ManualRelayProcessMode::MockCodexComplete(PathBuf::from(path)))
}

fn is_placeholder_process_mode(mock_behavior: &str) -> bool {
    mock_behavior == "placeholder_process_sleep"
}

#[cfg(test)]
fn placeholder_process_mode_allowed() -> bool {
    true
}

#[cfg(not(test))]
fn placeholder_process_mode_allowed() -> bool {
    false
}

#[cfg(test)]
fn mock_codex_process_mode_allowed() -> bool {
    true
}

#[cfg(not(test))]
fn mock_codex_process_mode_allowed() -> bool {
    false
}

fn process_config_for_mode(
    process_mode: ManualRelayProcessMode,
    mut command_plan: ManualRelayCommandPlan,
) -> Result<ManualRelayProcessConfig, String> {
    match process_mode {
        ManualRelayProcessMode::PlaceholderSleep => {
            Err("manual_relay_placeholder_process_mode_unexpected".to_string())
        }
        ManualRelayProcessMode::MockCodexComplete(program) => {
            command_plan.program = program.display().to_string();
            command_plan.redacted_preview =
                "mock codex fixture <stdin prompt> # workbench-managed last-message".to_string();
            Ok(ManualRelayProcessConfig {
                command_plan,
                process_kind: "mock_codex".to_string(),
                real_codex_executed: false,
                return_running: false,
                completed_status: "completed_mock_codex".to_string(),
            })
        }
        ManualRelayProcessMode::MockCodexSleep(program) => {
            command_plan.program = program.display().to_string();
            command_plan.redacted_preview =
                "mock codex sleep fixture <stdin prompt> # workbench-managed last-message"
                    .to_string();
            Ok(ManualRelayProcessConfig {
                command_plan,
                process_kind: "mock_codex".to_string(),
                real_codex_executed: false,
                return_running: true,
                completed_status: "completed_mock_codex".to_string(),
            })
        }
        ManualRelayProcessMode::RealCodexEnvGated => {
            ensure_real_codex_env_authorized()?;
            Ok(ManualRelayProcessConfig {
                command_plan,
                process_kind: "real_codex".to_string(),
                real_codex_executed: true,
                return_running: true,
                completed_status: "completed_real_codex".to_string(),
            })
        }
        ManualRelayProcessMode::RealCodexProductGui => Ok(ManualRelayProcessConfig {
            command_plan,
            process_kind: "real_codex".to_string(),
            real_codex_executed: true,
            return_running: true,
            completed_status: "completed_real_codex".to_string(),
        }),
    }
}

fn ensure_real_codex_env_authorized() -> Result<(), String> {
    match std::env::var(MANUAL_RELAY_REAL_CODEX_CONFIRM_ENV) {
        Ok(value) if value == MANUAL_RELAY_REAL_CODEX_CONFIRM_VALUE => Ok(()),
        _ => Err("manual_relay_real_codex_env_authorization_required".to_string()),
    }
}

fn spawn_running_codex_like_process(
    scope: &str,
    confirmation_id: &str,
    attempt_id: String,
    envelope: &ManualRelayEnvelope,
    process_config: ManualRelayProcessConfig,
    timestamp: &str,
    dirty_before: bool,
) -> Result<ManualRelayReceipt, String> {
    let mut registry = active_attempts()
        .lock()
        .map_err(|_| "manual_relay_registry_poisoned".to_string())?;
    if registry
        .values()
        .any(|attempt| attempt.duplicate_scope == scope && attempt.status == "running")
    {
        return Err("manual_relay_duplicate_running_attempt".to_string());
    }
    let mut consumed = consumed_confirmations()
        .lock()
        .map_err(|_| "manual_relay_confirmation_registry_poisoned".to_string())?;
    reserve_confirmation_in_map(&mut consumed, confirmation_id)?;
    let child = spawn_codex_like_process(
        &process_config.command_plan,
        envelope,
        Some(&envelope.payload.effective_prompt),
    )?;
    let process_id = Some(child.id());
    let mut receipt = fixture_receipt(
        &attempt_id,
        confirmation_id,
        "running",
        envelope,
        process_config.command_plan,
        timestamp,
        false,
        dirty_before,
        None,
    );
    receipt.process_id = process_id;
    receipt.process_kind = process_config.process_kind;
    receipt.prompt_sent = true;
    receipt.real_codex_executed = process_config.real_codex_executed;
    receipt.readback_status = "not_attempted_running_process".to_string();
    receipt
        .warnings
        .push("process_spawned_with_workbench_managed_last_message_only".to_string());
    if !receipt.real_codex_executed {
        receipt
            .warnings
            .push("mock_codex_fixture_no_real_codex".to_string());
    }
    receipt.warnings.sort();
    receipt.warnings.dedup();
    registry.insert(
        attempt_id.clone(),
        ActiveManualRelayAttempt {
            duplicate_scope: scope.to_string(),
            status: "running".to_string(),
            receipt: receipt.clone(),
            child: Some(child),
        },
    );
    consumed.insert(confirmation_id.to_string(), attempt_id);
    Ok(receipt)
}

fn run_codex_like_process_to_completion(
    attempt_id: &str,
    confirmation_id: &str,
    envelope: &ManualRelayEnvelope,
    process_config: ManualRelayProcessConfig,
    timestamp: &str,
    dirty_before: bool,
) -> Result<ManualRelayReceipt, String> {
    let mut child = spawn_codex_like_process(
        &process_config.command_plan,
        envelope,
        Some(&envelope.payload.effective_prompt),
    )?;
    let process_id = Some(child.id());
    let exit_status = child
        .wait()
        .map_err(|error| format!("manual_relay_process_wait_failed:{error}"))?;
    let (readback_status, last_message_hash, last_message_size_bytes) =
        read_last_message_summary(&process_config.command_plan.last_message_path);
    let status = if exit_status.success() {
        process_config.completed_status.as_str()
    } else {
        "failed_process"
    };
    let mut receipt = fixture_receipt(
        attempt_id,
        confirmation_id,
        status,
        envelope,
        process_config.command_plan,
        timestamp,
        false,
        dirty_before,
        last_message_hash,
    );
    receipt.process_id = process_id;
    receipt.process_kind = process_config.process_kind;
    receipt.prompt_sent = true;
    receipt.real_codex_executed = process_config.real_codex_executed;
    receipt.exit_code = exit_status.code();
    receipt.readback_status = readback_status;
    receipt.last_message_size_bytes = last_message_size_bytes;
    receipt
        .warnings
        .push("readback_last_message_only_no_full_transcript".to_string());
    if !receipt.real_codex_executed {
        receipt
            .warnings
            .push("mock_codex_fixture_no_real_codex".to_string());
    }
    receipt.warnings.sort();
    receipt.warnings.dedup();
    Ok(receipt)
}

fn spawn_codex_like_process(
    command_plan: &ManualRelayCommandPlan,
    envelope: &ManualRelayEnvelope,
    stdin_prompt: Option<&str>,
) -> Result<Child, String> {
    let last_message_path = PathBuf::from(&command_plan.last_message_path);
    let Some(output_dir) = last_message_path.parent() else {
        return Err("manual_relay_last_message_parent_missing".to_string());
    };
    fs::create_dir_all(output_dir)
        .map_err(|error| format!("manual_relay_last_message_dir_create_failed:{error}"))?;
    let mut command = Command::new(&command_plan.program);
    for arg in &command_plan.argv {
        if arg == "<workbench-managed-last-message>" {
            command.arg(&last_message_path);
        } else {
            command.arg(arg);
        }
    }
    command.current_dir(&envelope.target_binding.target_cwd_canonical);
    command
        .stdin(if stdin_prompt.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let mut child = command
        .spawn()
        .map_err(|error| format!("manual_relay_process_spawn_failed:{error}"))?;
    if let Some(prompt) = stdin_prompt {
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| "manual_relay_process_stdin_unavailable".to_string())?;
        if let Err(error) = stdin.write_all(prompt.as_bytes()) {
            let _ = child.kill();
            let _ = child.wait();
            return Err(format!("manual_relay_process_stdin_write_failed:{error}"));
        }
    }
    Ok(child)
}

fn read_last_message_summary(path: &str) -> (String, Option<String>, Option<i64>) {
    match fs::read_to_string(path) {
        Ok(message) if !message.is_empty() => (
            "workbench_managed_last_message_available".to_string(),
            Some(sha256_hex(&message)),
            Some(message.as_bytes().len() as i64),
        ),
        Ok(_) => (
            "workbench_managed_last_message_empty".to_string(),
            None,
            Some(0),
        ),
        Err(error) => (
            format!("workbench_managed_last_message_unavailable:{error}"),
            None,
            None,
        ),
    }
}

struct ActiveManualRelayAttempt {
    duplicate_scope: String,
    status: String,
    receipt: ManualRelayReceipt,
    child: Option<Child>,
}

fn active_attempts() -> &'static Mutex<BTreeMap<String, ActiveManualRelayAttempt>> {
    static REGISTRY: OnceLock<Mutex<BTreeMap<String, ActiveManualRelayAttempt>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(BTreeMap::new()))
}

fn consumed_confirmations() -> &'static Mutex<BTreeMap<String, String>> {
    static REGISTRY: OnceLock<Mutex<BTreeMap<String, String>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(BTreeMap::new()))
}

fn reserve_confirmation_once(confirmation_id: &str) -> Result<String, String> {
    let mut consumed = consumed_confirmations()
        .lock()
        .map_err(|_| "manual_relay_confirmation_registry_poisoned".to_string())?;
    reserve_confirmation_in_map(&mut consumed, confirmation_id)
}

fn reserve_confirmation_in_map(
    consumed: &mut BTreeMap<String, String>,
    confirmation_id: &str,
) -> Result<String, String> {
    match consumed.entry(confirmation_id.to_string()) {
        Entry::Vacant(entry) => {
            entry.insert("reserved".to_string());
            Ok("reserved".to_string())
        }
        Entry::Occupied(_) => Err("manual_relay_confirmation_already_consumed".to_string()),
    }
}

fn register_running_attempt_once(
    scope: &str,
    confirmation_id: &str,
    attempt_id: String,
    attempt: ActiveManualRelayAttempt,
) -> Result<(), String> {
    let mut registry = active_attempts()
        .lock()
        .map_err(|_| "manual_relay_registry_poisoned".to_string())?;
    if registry
        .values()
        .any(|attempt| attempt.duplicate_scope == scope && attempt.status == "running")
    {
        return Err("manual_relay_duplicate_running_attempt".to_string());
    }
    let mut consumed = consumed_confirmations()
        .lock()
        .map_err(|_| "manual_relay_confirmation_registry_poisoned".to_string())?;
    reserve_confirmation_in_map(&mut consumed, confirmation_id)?;
    registry.insert(attempt_id, attempt);
    Ok(())
}

fn reserve_non_running_attempt_once(scope: &str, confirmation_id: &str) -> Result<(), String> {
    let registry = active_attempts()
        .lock()
        .map_err(|_| "manual_relay_registry_poisoned".to_string())?;
    if registry
        .values()
        .any(|attempt| attempt.duplicate_scope == scope && attempt.status == "running")
    {
        return Err("manual_relay_duplicate_running_attempt".to_string());
    }
    let mut consumed = consumed_confirmations()
        .lock()
        .map_err(|_| "manual_relay_confirmation_registry_poisoned".to_string())?;
    reserve_confirmation_in_map(&mut consumed, confirmation_id)?;
    Ok(())
}

fn set_consumed_confirmation_attempt(
    confirmation_id: &str,
    attempt_id: &str,
) -> Result<(), String> {
    let mut consumed = consumed_confirmations()
        .lock()
        .map_err(|_| "manual_relay_confirmation_registry_poisoned".to_string())?;
    consumed.insert(confirmation_id.to_string(), attempt_id.to_string());
    Ok(())
}

fn inspect_manual_relay_guard(
    envelope: &ManualRelayEnvelope,
    requested_by: &str,
    last_message_path: &Path,
) -> ManualRelayGuard {
    let mut reasons = Vec::new();
    let mut warnings = vec![
        "manual_relay_fixture_only_no_real_codex".to_string(),
        "payload_layers_empty_in_v1".to_string(),
    ];
    if denied_material_requested(&envelope.payload.original_user_text) {
        reasons.push("manual_relay_denied_material_requested".to_string());
    }
    if envelope.payload.effective_prompt != envelope.payload.original_user_text
        || !envelope.payload.payload_layers.is_empty()
        || !envelope.payload.exact_original
    {
        reasons.push("manual_relay_payload_must_be_exact_original".to_string());
    }
    if !envelope.policy.manual_once || envelope.policy.auto_chain {
        reasons.push("manual_relay_policy_must_be_manual_once_without_auto_chain".to_string());
    }
    if envelope.target_binding.new_session && envelope.target_binding.target_session_id.is_some() {
        reasons.push("manual_relay_new_session_must_not_bind_target_session".to_string());
    }
    if !envelope.target_binding.new_session && envelope.target_binding.target_session_id.is_none() {
        reasons.push("manual_relay_existing_session_requires_target_session".to_string());
    }
    let codex_request = build_codex_local_request(envelope, requested_by);
    let codex_guard =
        crate::codex_local_runner::inspect_codex_local_execution_guard(&codex_request);
    reasons.extend(
        codex_guard
            .reasons
            .iter()
            .map(|reason| format!("codex_local_guard:{reason}")),
    );
    warnings.extend(
        codex_guard
            .warnings
            .iter()
            .map(|warning| format!("codex_local_guard:{warning}")),
    );
    reasons.sort();
    reasons.dedup();
    warnings.sort();
    warnings.dedup();
    let command_plan = codex_guard.command_plan.map(|plan| ManualRelayCommandPlan {
        program: plan.program,
        argv: plan
            .argv
            .into_iter()
            .map(|arg| {
                if arg == "<workbench-managed-last-message>" {
                    last_message_path.display().to_string()
                } else {
                    arg
                }
            })
            .collect(),
        stdin_prompt_ref: plan.stdin_prompt_ref,
        stdin_prompt_sha256: plan.stdin_prompt_sha256,
        prompt_in_command: plan.prompt_in_command,
        shell_invocation: plan.shell_invocation,
        redacted_preview: plan.redacted_preview,
        last_message_path: last_message_path.display().to_string(),
    });

    ManualRelayGuard {
        status: if reasons.is_empty() {
            "ready_fixture_only".to_string()
        } else {
            "blocked".to_string()
        },
        blocks_execution: !reasons.is_empty(),
        reasons,
        warnings,
        command_plan,
    }
}

fn build_codex_local_request(
    envelope: &ManualRelayEnvelope,
    requested_by: &str,
) -> crate::CodexLocalExecutionRequest {
    crate::CodexLocalExecutionRequest {
        request_version: 1,
        adapter_id: "codex-local".to_string(),
        operation_id: if envelope.target_binding.new_session {
            "new_session".to_string()
        } else {
            "resume".to_string()
        },
        project_id: format!(
            "project:manual-relay:{}",
            short_hash(&envelope.target_binding.project_root_canonical)
        ),
        project_root: envelope.target_binding.project_root_canonical.clone(),
        workflow_id: "manual-relay".to_string(),
        node_id: "manual-relay-once".to_string(),
        session_id: envelope.target_binding.target_session_id.clone(),
        work_item_id: if envelope.target_binding.new_session {
            Some(format!(
                "work-item:manual-relay:{}",
                short_hash(&envelope.relay_id)
            ))
        } else {
            None
        },
        continuation_id: None,
        target_cwd: envelope.target_binding.target_cwd_canonical.clone(),
        allowed_write_roots: envelope.target_binding.allowed_write_roots.clone(),
        sandbox: envelope.target_binding.sandbox.clone(),
        prompt_source_kind: "manual_relay_original_user_text".to_string(),
        prompt_summary: format!(
            "manual relay exact user text, sha256:{}",
            short_hash(&envelope.payload.prompt_sha256)
        ),
        prompt_sha256: envelope.payload.prompt_sha256.clone(),
        prompt_ref: format!("manual-relay-prompt:{}", envelope.payload.prompt_sha256),
        readback_plan: crate::CodexLocalReadbackPlan {
            strategy: "required".to_string(),
            required: true,
            expected_sources: vec!["workbench_managed_last_message".to_string()],
            unavailable_behavior: "readback_unavailable_is_not_zero_results".to_string(),
            trust_policy: "last_message_only_no_full_transcript_read".to_string(),
            warnings: vec!["manual_relay_does_not_read_rollout_body".to_string()],
        },
        requested_by: requested_by.to_string(),
        user_confirmation_state: "confirmed".to_string(),
        authorization_scope_id: Some(format!(
            "manual-relay-once:{}",
            short_hash(&envelope.relay_id)
        )),
        runtime_log_refs: vec![crate::CodexLocalRuntimeLogRef {
            ref_id: format!("runtime:manual-relay:{}", short_hash(&envelope.relay_id)),
            category: "manual_relay".to_string(),
            status: "preview_only_fixture_runner".to_string(),
            redaction_status: "prompt_body_not_persisted".to_string(),
        }],
        audit_refs: envelope
            .audit_refs
            .iter()
            .map(|ref_id| crate::CodexLocalAuditRef {
                ref_id: ref_id.clone(),
                event_type: "manual_relay_confirmation".to_string(),
                actor_role: requested_by.to_string(),
                decision: "confirmed_once".to_string(),
            })
            .collect(),
        active_attempts: active_attempts_for_scope(&envelope.policy.duplicate_scope),
        warnings: vec!["manual_relay_fixture_only".to_string()],
    }
}

fn active_attempts_for_scope(scope: &str) -> Vec<crate::CodexLocalActiveAttempt> {
    active_attempts()
        .lock()
        .ok()
        .map(|registry| {
            registry
                .iter()
                .filter(|(_, attempt)| {
                    attempt.duplicate_scope == scope && attempt.status == "running"
                })
                .map(|(attempt_id, _)| crate::CodexLocalActiveAttempt {
                    attempt_id: attempt_id.clone(),
                    status: "running".to_string(),
                    continuation_id: None,
                })
                .collect()
        })
        .unwrap_or_default()
}

fn validate_run_binding(input: &ManualRelayRunInput) -> Result<(), String> {
    if input.confirmation_id != input.confirmation.confirmation_id {
        return Err("manual_relay_confirmation_id_mismatch".to_string());
    }
    if input.confirmation.relay_id != input.envelope.relay_id {
        return Err("manual_relay_confirmation_relay_id_mismatch".to_string());
    }
    if input.expected_prompt_sha256 != input.envelope.payload.prompt_sha256
        || input.confirmation.prompt_sha256 != input.envelope.payload.prompt_sha256
    {
        return Err("manual_relay_prompt_hash_mismatch".to_string());
    }
    if input.expected_target_hash != input.envelope.target_binding.target_hash
        || input.confirmation.target_hash != input.envelope.target_binding.target_hash
    {
        return Err("manual_relay_target_hash_mismatch".to_string());
    }
    if input.expected_sandbox != input.envelope.target_binding.sandbox
        || input.confirmation.sandbox != input.envelope.target_binding.sandbox
    {
        return Err("manual_relay_sandbox_mismatch".to_string());
    }
    if input.expected_allowed_write_roots != input.envelope.target_binding.allowed_write_roots
        || input.confirmation.allowed_write_roots
            != input.envelope.target_binding.allowed_write_roots
    {
        return Err("manual_relay_allowed_write_roots_mismatch".to_string());
    }
    if !input.confirmation.manual_once || input.confirmation.auto_chain {
        return Err("manual_relay_confirmation_must_be_one_shot".to_string());
    }
    Ok(())
}

fn validate_gui_direct_input(input: &ManualRelayGuiDirectRunInput) -> Result<(), String> {
    if input.original_user_text.trim().is_empty() {
        return Err("manual_relay_gui_direct_prompt_required".to_string());
    }
    if input.target_session_id.trim().is_empty() {
        return Err("manual_relay_gui_direct_requires_bound_session".to_string());
    }
    if input.target_project_root.trim().is_empty() || input.target_cwd.trim().is_empty() {
        return Err("manual_relay_gui_direct_target_required".to_string());
    }
    if input.requested_by.trim().is_empty() {
        return Err("manual_relay_gui_direct_requested_by_required".to_string());
    }
    Ok(())
}

fn validate_gui_direct_target_and_command_plan(
    envelope: &ManualRelayEnvelope,
    command_plan: &ManualRelayCommandPlan,
) -> Result<(), String> {
    verify_strict_run_paths(envelope)?;
    if envelope.target_binding.new_session || envelope.target_binding.target_session_id.is_none() {
        return Err("manual_relay_gui_direct_requires_bound_session".to_string());
    }
    if envelope.target_binding.sandbox != "workspace-write" {
        return Err("manual_relay_gui_direct_sandbox_must_be_workspace_write".to_string());
    }
    if envelope.target_binding.target_cwd_canonical
        != envelope.target_binding.project_root_canonical
    {
        return Err("manual_relay_gui_direct_cwd_must_equal_project_root".to_string());
    }
    if envelope.target_binding.allowed_write_roots
        != vec![envelope.target_binding.project_root_canonical.clone()]
    {
        return Err("manual_relay_gui_direct_write_roots_must_equal_project_root".to_string());
    }
    if command_plan.program != "codex" {
        return Err("manual_relay_gui_direct_program_must_be_codex".to_string());
    }
    if command_plan.prompt_in_command || command_plan.shell_invocation {
        return Err("manual_relay_gui_direct_prompt_must_use_stdin_no_shell".to_string());
    }
    if !argv_contains_pair(
        &command_plan.argv,
        "--sandbox",
        &envelope.target_binding.sandbox,
    ) {
        return Err("manual_relay_gui_direct_sandbox_arg_missing".to_string());
    }
    if !argv_contains_pair(
        &command_plan.argv,
        "--add-dir",
        &envelope.target_binding.project_root_canonical,
    ) {
        return Err("manual_relay_gui_direct_add_dir_arg_missing".to_string());
    }
    if command_plan
        .argv
        .iter()
        .any(|arg| codex_approval_bypass_arg(arg))
    {
        return Err("manual_relay_gui_direct_approval_bypass_arg_forbidden".to_string());
    }
    Ok(())
}

fn argv_contains_pair(argv: &[String], flag: &str, value: &str) -> bool {
    argv.windows(2).any(|pair| {
        pair.first().is_some_and(|arg| arg == flag) && pair.get(1).is_some_and(|arg| arg == value)
    })
}

fn codex_approval_bypass_arg(arg: &str) -> bool {
    arg == "--full-auto"
        || arg.contains("dangerously-bypass")
        || arg.starts_with("--approval")
        || arg == "full-auto"
}

fn is_process_mode(mock_behavior: &str) -> bool {
    mock_behavior.starts_with("placeholder_process_")
        || manual_relay_process_mode(mock_behavior).is_some()
}

fn verify_strict_run_paths(envelope: &ManualRelayEnvelope) -> Result<(), String> {
    let project_root = canonical_path_text(&envelope.target_binding.project_root_canonical)
        .map_err(|_| "manual_relay_paths_not_verified".to_string())?;
    let target_cwd = canonical_path_text(&envelope.target_binding.target_cwd_canonical)
        .map_err(|_| "manual_relay_paths_not_verified".to_string())?;
    let allowed_write_roots = envelope
        .target_binding
        .allowed_write_roots
        .iter()
        .map(|root| canonical_path_text(root))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "manual_relay_paths_not_verified".to_string())?;
    let target_hash = target_hash_for_binding(
        &project_root,
        &target_cwd,
        envelope.target_binding.target_session_id.as_deref(),
        envelope.target_binding.new_session,
        &envelope.target_binding.sandbox,
    );
    if !envelope.target_binding.path_verified
        || project_root != envelope.target_binding.project_root_canonical
        || target_cwd != envelope.target_binding.target_cwd_canonical
        || allowed_write_roots != envelope.target_binding.allowed_write_roots
        || target_hash != envelope.target_binding.target_hash
    {
        return Err("manual_relay_paths_not_verified".to_string());
    }
    Ok(())
}

fn placeholder_command_plan(
    last_message_path: &Path,
    envelope: &ManualRelayEnvelope,
) -> ManualRelayCommandPlan {
    ManualRelayCommandPlan {
        program: "/bin/sleep".to_string(),
        argv: vec!["30".to_string()],
        stdin_prompt_ref: format!("manual-relay-prompt:{}", envelope.payload.prompt_sha256),
        stdin_prompt_sha256: envelope.payload.prompt_sha256.clone(),
        prompt_in_command: false,
        shell_invocation: false,
        redacted_preview: "placeholder process: /bin/sleep 30 <stdin prompt ignored>".to_string(),
        last_message_path: last_message_path.display().to_string(),
    }
}

fn spawn_placeholder_process(
    command_plan: &ManualRelayCommandPlan,
    envelope: &ManualRelayEnvelope,
) -> Result<Child, String> {
    let mut command = Command::new(&command_plan.program);
    command.args(&command_plan.argv);
    command.current_dir(&envelope.target_binding.target_cwd_canonical);
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    command
        .spawn()
        .map_err(|error| format!("manual_relay_placeholder_spawn_failed:{error}"))
}

fn fixture_receipt(
    attempt_id: &str,
    confirmation_id: &str,
    status: &str,
    envelope: &ManualRelayEnvelope,
    command_plan: ManualRelayCommandPlan,
    timestamp: &str,
    killed_by_user: bool,
    dirty_before: bool,
    last_message_hash: Option<String>,
) -> ManualRelayReceipt {
    ManualRelayReceipt {
        relay_attempt_id: attempt_id.to_string(),
        confirmation_id: confirmation_id.to_string(),
        target: envelope.target_binding.clone(),
        effective_prompt_sha256: envelope.payload.prompt_sha256.clone(),
        prompt_length_bytes: envelope.payload.prompt_length_bytes,
        prompt_exact_original: envelope.payload.exact_original
            && envelope.payload.effective_prompt == envelope.payload.original_user_text,
        command_plan,
        started_at: timestamp.to_string(),
        ended_at: if status == "running" {
            None
        } else {
            Some(timestamp.to_string())
        },
        exit_code: if status == "running" { None } else { Some(0) },
        process_id: None,
        process_kind: "fixture".to_string(),
        real_process_killed: false,
        status: status.to_string(),
        prompt_sent: false,
        real_codex_executed: false,
        syn_read_codex_home: false,
        syn_wrote_codex_home: false,
        killed_by_user,
        timed_out: false,
        readback_status: if status == "running" {
            "not_attempted_running_fixture".to_string()
        } else {
            "fixture_last_message_available".to_string()
        },
        last_message_hash,
        last_message_size_bytes: if status == "running" { None } else { Some(33) },
        changed_files: Vec::new(),
        git_head_before: Some("fixture-head-before".to_string()),
        git_head_after: Some("fixture-head-after".to_string()),
        git_status_before: if dirty_before {
            "dirty_fixture".to_string()
        } else {
            "clean_fixture".to_string()
        },
        git_status_after: if dirty_before {
            "dirty_fixture_no_auto_rollback".to_string()
        } else {
            "clean_fixture".to_string()
        },
        rollback: ManualRelayRollbackSummary {
            git_available: true,
            dirty_before,
            auto_rollback_performed: false,
            rollback_suggestion_available: !dirty_before,
            summary: if dirty_before {
                "target tree was dirty before relay; manual recovery suggestion only, no git reset/checkout performed".to_string()
            } else {
                "target tree was clean; rollback suggestion may be generated later, real restore requires separate confirmation".to_string()
            },
        },
        warnings: vec![
            "manual_relay_fixture_runner_only".to_string(),
            "no_real_codex_exec_in_this_package".to_string(),
            "no_codex_home_read_or_write_by_syn".to_string(),
        ],
    }
}

fn denied_material_requested(text: &str) -> bool {
    let lowered = text.to_ascii_lowercase();
    [
        "/users/yoyi/.codex",
        ".codex",
        "auth.json",
        "secret",
        "token",
        ".env",
        "keychain",
        "oauth",
        "credential",
        "full transcript",
        "完整 transcript",
        "rollout",
        "prompt body",
    ]
    .iter()
    .any(|needle| lowered.contains(needle))
}

#[derive(Clone, Debug)]
struct PreviewPath {
    normalized: String,
    verified: bool,
}

fn normalize_path_for_preview(value: &str) -> PreviewPath {
    let path = PathBuf::from(value.trim());
    let absolute = if path.is_absolute() {
        path
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("/"))
            .join(path)
    };
    match std::fs::canonicalize(&absolute) {
        Ok(path) => PreviewPath {
            normalized: path.display().to_string(),
            verified: true,
        },
        Err(_) => PreviewPath {
            normalized: clean_path(&absolute).display().to_string(),
            verified: false,
        },
    }
}

fn canonical_path_text(value: &str) -> Result<String, String> {
    let path = PathBuf::from(value.trim());
    std::fs::canonicalize(&path)
        .map(|path| path.display().to_string())
        .map_err(|error| format!("manual_relay_path_canonicalize_failed:{error}"))
}

fn target_hash_for_binding(
    project_root: &str,
    target_cwd: &str,
    target_session_id: Option<&str>,
    new_session: bool,
    sandbox: &str,
) -> String {
    sha256_hex(&format!(
        "{}\n{}\n{}\n{}\n{}",
        project_root,
        target_cwd,
        target_session_id.unwrap_or("new-session"),
        new_session,
        sandbox
    ))
}

fn clean_path(path: &Path) -> PathBuf {
    let mut cleaned = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                cleaned.pop();
            }
            other => cleaned.push(other.as_os_str()),
        }
    }
    cleaned
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    use std::sync::{Mutex, OnceLock};
    use std::thread;
    use std::time::{Duration, Instant};

    #[test]
    fn manual_relay_preview_keeps_payload_exact_and_structured_command_safe() {
        let _guard = test_guard();
        let preview =
            preview_manual_relay(fixture_preview_input("hello relay"), "2026-06-17T12:00:00Z");

        assert_eq!(preview.envelope.payload.original_user_text, "hello relay");
        assert_eq!(preview.envelope.payload.effective_prompt, "hello relay");
        assert!(preview.envelope.payload.payload_layers.is_empty());
        assert!(preview.envelope.payload.exact_original);
        assert!(preview.envelope.policy.manual_once);
        assert!(!preview.envelope.policy.auto_chain);
        let plan = preview
            .guard
            .command_plan
            .expect("safe preview should include command plan");
        assert_eq!(plan.program, "codex");
        assert!(!plan.shell_invocation);
        assert!(!plan.prompt_in_command);
        assert_eq!(
            plan.stdin_prompt_sha256,
            preview.envelope.payload.prompt_sha256
        );
        assert!(plan.last_message_path.contains("manual-relay-runs"));
    }

    #[test]
    fn manual_relay_blocks_hash_mismatch_and_duplicate() {
        let _guard = test_guard();
        let preview =
            preview_manual_relay(fixture_preview_input("hash gate"), "2026-06-17T12:00:00Z");
        let confirmation = confirm_manual_relay_once(
            fixture_confirm_input(&preview.envelope),
            "2026-06-17T12:00:01Z",
        )
        .expect("confirmation should pass");

        let mut mismatch = fixture_run_input(&preview.envelope, &confirmation);
        mismatch.expected_prompt_sha256 = "0".repeat(64);
        let error = run_manual_relay_once(mismatch, "2026-06-17T12:00:02Z")
            .expect_err("prompt mismatch must block");
        assert!(error.contains("manual_relay_prompt_hash_mismatch"));

        let mut target_mismatch = fixture_run_input(&preview.envelope, &confirmation);
        target_mismatch.expected_target_hash = "1".repeat(64);
        let error = run_manual_relay_once(target_mismatch, "2026-06-17T12:00:02Z")
            .expect_err("target mismatch must block");
        assert!(error.contains("manual_relay_target_hash_mismatch"));

        let mut confirmation_mismatch = fixture_run_input(&preview.envelope, &confirmation);
        confirmation_mismatch.confirmation_id = "manual-relay-confirmation:wrong".to_string();
        let error = run_manual_relay_once(confirmation_mismatch, "2026-06-17T12:00:02Z")
            .expect_err("confirmation id mismatch must block");
        assert!(error.contains("manual_relay_confirmation_id_mismatch"));

        let mut running = fixture_run_input(&preview.envelope, &confirmation);
        running.mock_behavior = "stay_running".to_string();
        let running_receipt = run_manual_relay_once(running, "2026-06-17T12:00:03Z")
            .expect("first running attempt should start");
        assert_eq!(running_receipt.status, "running");

        let duplicate = run_manual_relay_once(
            fixture_run_input(&preview.envelope, &confirmation),
            "2026-06-17T12:00:04Z",
        )
        .expect_err("duplicate running attempt must block");
        assert!(
            duplicate.contains("duplicate_running_attempt"),
            "unexpected duplicate error: {duplicate}"
        );
    }

    #[test]
    fn manual_relay_consumes_confirmation_once_and_receipt_records_contract_fields() {
        let _guard = test_guard();
        let preview = preview_manual_relay(
            fixture_preview_input("receipt contract"),
            "2026-06-17T12:00:00Z",
        );
        let confirmation = confirm_manual_relay_once(
            fixture_confirm_input(&preview.envelope),
            "2026-06-17T12:00:01Z",
        )
        .expect("confirmation should pass");

        let receipt = run_manual_relay_once(
            fixture_run_input(&preview.envelope, &confirmation),
            "2026-06-17T12:00:02Z",
        )
        .expect("fixture run should complete");
        assert_eq!(receipt.status, "completed_fixture");
        assert_eq!(receipt.target, preview.envelope.target_binding);
        assert_eq!(
            receipt.effective_prompt_sha256,
            preview.envelope.payload.prompt_sha256
        );
        assert_eq!(
            receipt.prompt_length_bytes,
            preview.envelope.payload.prompt_length_bytes
        );
        assert!(receipt.prompt_exact_original);
        assert_eq!(
            receipt.command_plan.stdin_prompt_sha256,
            preview.envelope.payload.prompt_sha256
        );
        assert_eq!(receipt.started_at, "2026-06-17T12:00:02Z");
        assert_eq!(receipt.ended_at.as_deref(), Some("2026-06-17T12:00:02Z"));
        assert_eq!(receipt.exit_code, Some(0));
        assert_eq!(receipt.last_message_size_bytes, Some(33));
        assert!(receipt.changed_files.is_empty());
        assert_eq!(receipt.git_status_before, "clean_fixture");
        assert_eq!(receipt.git_status_after, "clean_fixture");
        assert!(!receipt.real_codex_executed);

        let replay = run_manual_relay_once(
            fixture_run_input(&preview.envelope, &confirmation),
            "2026-06-17T12:00:03Z",
        )
        .expect_err("same confirmation must not be reusable after terminal receipt");
        assert!(replay.contains("manual_relay_confirmation_already_consumed"));
    }

    #[test]
    fn manual_relay_blocks_sensitive_or_codex_home_requests() {
        let _guard = test_guard();
        for prompt in [
            "read /Users/yoyi/.codex/auth.json",
            "show me the full transcript and rollout body",
            "cat .env token secret keychain OAuth credential",
        ] {
            let preview =
                preview_manual_relay(fixture_preview_input(prompt), "2026-06-17T12:00:00Z");
            assert!(
                preview.guard.blocks_execution,
                "prompt should be blocked: {prompt}"
            );
            assert!(preview
                .guard
                .reasons
                .contains(&"manual_relay_denied_material_requested".to_string()));
        }
    }

    #[test]
    fn manual_relay_stop_only_kills_current_attempt_and_dirty_tree_never_auto_rolls_back() {
        let _guard = test_guard();
        let first = preview_manual_relay(fixture_preview_input("first"), "2026-06-17T12:00:00Z");
        let first_confirmation = confirm_manual_relay_once(
            fixture_confirm_input(&first.envelope),
            "2026-06-17T12:00:01Z",
        )
        .expect("first confirmation should pass");
        let mut first_run = fixture_run_input(&first.envelope, &first_confirmation);
        first_run.mock_behavior = "stay_running_dirty_tree".to_string();
        let first_receipt = run_manual_relay_once(first_run, "2026-06-17T12:00:02Z")
            .expect("first attempt should run");

        let second = preview_manual_relay(fixture_preview_input("second"), "2026-06-17T12:00:03Z");
        let second_confirmation = confirm_manual_relay_once(
            fixture_confirm_input(&second.envelope),
            "2026-06-17T12:00:04Z",
        )
        .expect("second confirmation should pass");
        let mut second_run = fixture_run_input(&second.envelope, &second_confirmation);
        second_run.mock_behavior = "stay_running".to_string();
        let second_receipt = run_manual_relay_once(second_run, "2026-06-17T12:00:05Z")
            .expect("second attempt should run");

        let stopped = stop_manual_relay_attempt(
            ManualRelayStopInput {
                relay_attempt_id: first_receipt.relay_attempt_id.clone(),
                requested_by: "user".to_string(),
            },
            "2026-06-17T12:00:06Z",
        )
        .expect("stop should kill the requested attempt");
        assert!(stopped.killed_by_user);
        assert_eq!(stopped.status, "stopped_by_user");
        assert!(stopped.rollback.dirty_before);
        assert!(!stopped.rollback.auto_rollback_performed);

        let still_running = stop_manual_relay_attempt(
            ManualRelayStopInput {
                relay_attempt_id: second_receipt.relay_attempt_id,
                requested_by: "user".to_string(),
            },
            "2026-06-17T12:00:07Z",
        )
        .expect("second attempt should still be independently stoppable");
        assert!(still_running.killed_by_user);
    }

    #[test]
    fn manual_relay_strict_run_requires_verified_paths() {
        let _guard = test_guard();
        let mut preview_input = fixture_preview_input("strict paths");
        let missing_root = std::env::temp_dir().join(format!(
            "manual-relay-missing-{}",
            short_hash("strict paths")
        ));
        preview_input.target_project_root = missing_root.display().to_string();
        preview_input.target_cwd = missing_root.display().to_string();
        preview_input.allowed_write_roots = vec![missing_root.display().to_string()];
        let preview = preview_manual_relay(preview_input, "2026-06-18T00:00:00Z");
        assert!(!preview.envelope.target_binding.path_verified);

        let confirmation = confirm_manual_relay_once(
            fixture_confirm_input(&preview.envelope),
            "2026-06-18T00:00:01Z",
        )
        .expect("confirmation should still be preview-level only");
        let mut run_input = fixture_run_input(&preview.envelope, &confirmation);
        run_input.mock_behavior = "placeholder_process_complete".to_string();
        let error = run_manual_relay_once(run_input, "2026-06-18T00:00:02Z")
            .expect_err("strict run must reject unverified paths");
        assert!(error.contains("manual_relay_paths_not_verified"));
    }

    #[test]
    fn manual_relay_confirmation_reservation_is_atomic_for_reentrant_submit() {
        let _guard = test_guard();
        let preview = preview_manual_relay(
            fixture_preview_input("atomic confirmation"),
            "2026-06-18T00:00:00Z",
        );
        let confirmation = confirm_manual_relay_once(
            fixture_confirm_input(&preview.envelope),
            "2026-06-18T00:00:01Z",
        )
        .expect("confirmation should pass");

        let first = reserve_confirmation_once(&confirmation.confirmation_id)
            .expect("first reservation should win");
        let second = reserve_confirmation_once(&confirmation.confirmation_id)
            .expect_err("second reservation must be rejected atomically");
        assert_eq!(first, "reserved");
        assert!(second.contains("manual_relay_confirmation_already_consumed"));
    }

    #[test]
    fn manual_relay_run_consumes_confirmation_once_for_concurrent_submit() {
        let _guard = test_guard();
        let preview = preview_manual_relay(
            fixture_preview_input("atomic run confirmation"),
            "2026-06-18T00:00:00Z",
        );
        let confirmation = confirm_manual_relay_once(
            fixture_confirm_input(&preview.envelope),
            "2026-06-18T00:00:01Z",
        )
        .expect("confirmation should pass");
        let run_input = fixture_run_input(&preview.envelope, &confirmation);

        let first_input = run_input.clone();
        let second_input = run_input.clone();
        let first =
            std::thread::spawn(move || run_manual_relay_once(first_input, "2026-06-18T00:00:02Z"));
        let second =
            std::thread::spawn(move || run_manual_relay_once(second_input, "2026-06-18T00:00:03Z"));
        let results = vec![
            first.join().expect("first runner should not panic"),
            second.join().expect("second runner should not panic"),
        ];

        assert_eq!(
            results.iter().filter(|result| result.is_ok()).count(),
            1,
            "exactly one concurrent run may consume a confirmation"
        );
        assert_eq!(
            results
                .iter()
                .filter(|result| {
                    result.as_ref().err().is_some_and(|error| {
                        error.contains("manual_relay_confirmation_already_consumed")
                    })
                })
                .count(),
            1,
            "the losing concurrent run must be rejected by the one-shot guard"
        );
    }

    #[test]
    fn manual_relay_duplicate_scope_reservation_is_atomic_for_distinct_confirmations() {
        let _guard = test_guard();
        let preview = preview_manual_relay(
            fixture_preview_input("atomic duplicate scope"),
            "2026-06-18T00:00:00Z",
        );
        let first_confirmation = confirm_manual_relay_once(
            fixture_confirm_input(&preview.envelope),
            "2026-06-18T00:00:01Z",
        )
        .expect("first confirmation should pass");
        let second_confirmation = confirm_manual_relay_once(
            fixture_confirm_input(&preview.envelope),
            "2026-06-18T00:00:02Z",
        )
        .expect("second confirmation should pass");
        assert_ne!(
            first_confirmation.confirmation_id,
            second_confirmation.confirmation_id
        );

        let mut first_input = fixture_run_input(&preview.envelope, &first_confirmation);
        first_input.mock_behavior = "stay_running".to_string();
        let mut second_input = fixture_run_input(&preview.envelope, &second_confirmation);
        second_input.mock_behavior = "stay_running".to_string();
        let first =
            std::thread::spawn(move || run_manual_relay_once(first_input, "2026-06-18T00:00:03Z"));
        let second =
            std::thread::spawn(move || run_manual_relay_once(second_input, "2026-06-18T00:00:04Z"));
        let results = vec![
            first.join().expect("first runner should not panic"),
            second.join().expect("second runner should not panic"),
        ];

        assert_eq!(
            results.iter().filter(|result| result.is_ok()).count(),
            1,
            "only one running attempt may occupy a duplicate scope"
        );
        assert_eq!(
            results
                .iter()
                .filter(|result| {
                    result
                        .as_ref()
                        .err()
                        .is_some_and(|error| error.contains("duplicate_running_attempt"))
                })
                .count(),
            1,
            "the losing concurrent run must be rejected by the duplicate-scope guard"
        );
    }

    #[test]
    fn manual_relay_placeholder_process_can_be_stopped_and_reaped() {
        let _guard = test_guard();
        let preview = preview_manual_relay(
            existing_fixture_preview_input("placeholder process"),
            "2026-06-18T00:00:00Z",
        );
        assert!(preview.envelope.target_binding.path_verified);
        let confirmation = confirm_manual_relay_once(
            fixture_confirm_input(&preview.envelope),
            "2026-06-18T00:00:01Z",
        )
        .expect("confirmation should pass");
        let mut run_input = fixture_run_input(&preview.envelope, &confirmation);
        run_input.mock_behavior = "placeholder_process_sleep".to_string();
        let running = run_manual_relay_once(run_input, "2026-06-18T00:00:02Z")
            .expect("placeholder process should start");
        assert_eq!(running.status, "running");
        assert_eq!(running.process_kind, "placeholder");
        assert!(running.process_id.is_some());
        assert!(!running.real_codex_executed);

        let stopped = stop_manual_relay_attempt(
            ManualRelayStopInput {
                relay_attempt_id: running.relay_attempt_id.clone(),
                requested_by: "user".to_string(),
            },
            "2026-06-18T00:00:03Z",
        )
        .expect("stop must kill placeholder process");
        assert_eq!(stopped.status, "stopped_by_user");
        assert!(stopped.killed_by_user);
        assert!(stopped.real_process_killed);
        assert_eq!(stopped.process_kind, "placeholder");
        assert!(!stopped.real_codex_executed);
    }

    #[test]
    fn manual_relay_real_codex_env_gated_without_env_does_not_spawn() {
        let _guard = test_guard();
        std::env::remove_var("MANUAL_RELAY_REAL_CODEX_CONFIRM");
        let preview = preview_manual_relay(
            existing_fixture_preview_input("real codex no env"),
            "2026-06-18T00:00:00Z",
        );
        let confirmation = confirm_manual_relay_once(
            fixture_confirm_input(&preview.envelope),
            "2026-06-18T00:00:01Z",
        )
        .expect("confirmation should pass");
        let mut run_input = fixture_run_input(&preview.envelope, &confirmation);
        run_input.mock_behavior = "real_codex_env_gated".to_string();
        let error = run_manual_relay_once(run_input, "2026-06-18T00:00:02Z")
            .expect_err("real codex path must require explicit env authorization");
        assert!(error.contains("manual_relay_real_codex_env_authorization_required"));
        assert!(active_attempts()
            .lock()
            .expect("registry should not poison")
            .is_empty());
    }

    #[test]
    fn manual_relay_gui_direct_send_uses_bound_target_without_approval_bypass() {
        let _guard = test_guard();
        std::env::remove_var("MANUAL_RELAY_REAL_CODEX_CONFIRM");
        let script = mock_codex_script(
            "gui-direct-complete",
            r#"#!/bin/sh
last=""
while [ "$#" -gt 0 ]; do
  if [ "$1" = "--output-last-message" ]; then
    shift
    last="$1"
  fi
  shift || true
done
if [ -z "$last" ]; then
  exit 42
fi
mkdir -p "$(dirname "$last")"
prompt="$(cat)"
printf 'gui direct mock last message: %s\n' "$prompt" > "$last"
exit 0
"#,
        );
        let fixture = existing_fixture_preview_input("GUI direct exact prompt");
        let input = ManualRelayGuiDirectRunInput {
            original_user_text: fixture.original_user_text.clone(),
            target_project_root: fixture.target_project_root.clone(),
            target_cwd: fixture.target_cwd.clone(),
            target_session_id: fixture
                .target_session_id
                .clone()
                .expect("GUI direct send must bind an existing session"),
            sandbox: fixture.sandbox.clone(),
            allowed_write_roots: fixture.allowed_write_roots.clone(),
            requested_by: "user".to_string(),
        };

        let receipt = run_manual_relay_gui_direct_once_for_test(
            input,
            "2026-06-18T08:00:00Z",
            &format!("mock_codex_process:{}", script.display()),
        )
        .expect("GUI direct send should run through the mock codex process");

        assert_eq!(receipt.status, "completed_mock_codex");
        assert_eq!(receipt.process_kind, "mock_codex");
        assert!(receipt.prompt_sent);
        assert!(!receipt.real_codex_executed);
        assert!(!receipt.command_plan.prompt_in_command);
        assert!(!receipt.command_plan.shell_invocation);
        assert!(receipt
            .command_plan
            .argv
            .iter()
            .any(|arg| arg == "--sandbox"));
        assert!(receipt
            .command_plan
            .argv
            .iter()
            .any(|arg| arg == "workspace-write"));
        assert!(receipt
            .command_plan
            .argv
            .iter()
            .any(|arg| arg == "--add-dir"));
        assert!(receipt
            .command_plan
            .argv
            .iter()
            .any(|arg| arg == &receipt.target.project_root_canonical));
        assert!(!receipt
            .command_plan
            .argv
            .iter()
            .any(|arg| arg == "--full-auto" || arg.contains("dangerously-bypass")));
        assert!(receipt.target.path_verified);
        assert_eq!(receipt.target.target_session_id, fixture.target_session_id);
        assert!(!receipt.target.new_session);
    }

    #[test]
    fn manual_relay_mock_codex_process_writes_last_message_readback() {
        let _guard = test_guard();
        let script = mock_codex_script(
            "complete",
            r#"#!/bin/sh
last=""
while [ "$#" -gt 0 ]; do
  if [ "$1" = "--output-last-message" ]; then
    shift
    last="$1"
  fi
  shift || true
done
if [ -z "$last" ]; then
  exit 42
fi
mkdir -p "$(dirname "$last")"
prompt="$(cat)"
printf 'mock codex last message: %s\n' "$prompt" > "$last"
exit 0
"#,
        );
        let preview = preview_manual_relay(
            existing_fixture_preview_input("mock codex readback"),
            "2026-06-18T00:00:00Z",
        );
        let confirmation = confirm_manual_relay_once(
            fixture_confirm_input(&preview.envelope),
            "2026-06-18T00:00:01Z",
        )
        .expect("confirmation should pass");
        let mut run_input = fixture_run_input(&preview.envelope, &confirmation);
        run_input.mock_behavior = format!("mock_codex_process:{}", script.display());
        let receipt = run_manual_relay_once(run_input, "2026-06-18T00:00:02Z")
            .expect("mock codex process should complete");
        let expected_last_message = "mock codex last message: mock codex readback\n".to_string();
        assert_eq!(receipt.status, "completed_mock_codex");
        assert_eq!(receipt.process_kind, "mock_codex");
        assert_eq!(receipt.exit_code, Some(0));
        assert!(receipt.prompt_sent);
        assert!(!receipt.real_codex_executed);
        assert_eq!(
            receipt.readback_status,
            "workbench_managed_last_message_available"
        );
        assert_eq!(
            receipt.last_message_hash.as_deref(),
            Some(sha256_hex(&expected_last_message).as_str())
        );
        assert_eq!(
            receipt.last_message_size_bytes,
            Some(expected_last_message.as_bytes().len() as i64)
        );
        assert!(!receipt.syn_read_codex_home);
        assert!(!receipt.syn_wrote_codex_home);
        assert!(receipt
            .warnings
            .contains(&"readback_last_message_only_no_full_transcript".to_string()));
    }

    #[test]
    fn manual_relay_mock_codex_process_can_be_stopped_and_reaped() {
        let _guard = test_guard();
        let script = mock_codex_script(
            "sleep",
            r#"#!/bin/sh
last=""
while [ "$#" -gt 0 ]; do
  if [ "$1" = "--output-last-message" ]; then
    shift
    last="$1"
  fi
  shift || true
done
if [ -n "$last" ]; then
  mkdir -p "$(dirname "$last")"
  printf 'mock codex started\n' > "$last"
fi
sleep 30
"#,
        );
        let preview = preview_manual_relay(
            existing_fixture_preview_input("mock codex stop"),
            "2026-06-18T00:00:00Z",
        );
        let confirmation = confirm_manual_relay_once(
            fixture_confirm_input(&preview.envelope),
            "2026-06-18T00:00:01Z",
        )
        .expect("confirmation should pass");
        let mut run_input = fixture_run_input(&preview.envelope, &confirmation);
        run_input.mock_behavior = format!("mock_codex_process_sleep:{}", script.display());
        let running = run_manual_relay_once(run_input, "2026-06-18T00:00:02Z")
            .expect("mock codex process should start");
        assert_eq!(running.status, "running");
        assert_eq!(running.process_kind, "mock_codex");
        assert!(running.process_id.is_some());
        assert!(running.prompt_sent);
        assert!(!running.real_codex_executed);

        let stopped = stop_manual_relay_attempt(
            ManualRelayStopInput {
                relay_attempt_id: running.relay_attempt_id,
                requested_by: "user".to_string(),
            },
            "2026-06-18T00:00:03Z",
        )
        .expect("stop must kill mock codex process");
        assert_eq!(stopped.status, "stopped_by_user");
        assert!(stopped.killed_by_user);
        assert!(stopped.real_process_killed);
        assert_eq!(stopped.process_kind, "mock_codex");
        assert!(!stopped.real_codex_executed);
    }

    #[test]
    fn manual_relay_b1_runner_entry_uses_temp_fixture_defaults_with_mock_process() {
        let _guard = test_guard();
        let script = mock_codex_script(
            "b1-complete",
            r#"#!/bin/sh
last=""
while [ "$#" -gt 0 ]; do
  if [ "$1" = "--output-last-message" ]; then
    shift
    last="$1"
  fi
  shift || true
done
if [ -z "$last" ]; then
  exit 42
fi
mkdir -p "$(dirname "$last")"
prompt="$(cat)"
printf 'manual relay b1 mock last message: %s\n' "$prompt" > "$last"
exit 0
"#,
        );
        let preview_input = b1_real_relay_fixture_preview_input("mock-defaults")
            .expect("B1 fixture input should be built");
        assert!(preview_input.original_user_text.contains("hello.txt"));
        assert!(preview_input.original_user_text.contains("hi"));
        assert!(preview_input.new_session);
        assert!(preview_input.target_session_id.is_none());
        assert_eq!(preview_input.sandbox, "workspace-write");
        assert_eq!(
            preview_input.allowed_write_roots,
            vec![preview_input.target_project_root.clone()]
        );
        assert!(PathBuf::from(&preview_input.target_project_root).exists());

        let preview = preview_manual_relay(preview_input, "2026-06-18T00:00:00Z");
        assert!(preview.envelope.target_binding.path_verified);
        let plan = preview
            .guard
            .command_plan
            .as_ref()
            .expect("B1 fixture should expose a codex command plan");
        assert_eq!(plan.program, "codex");
        assert!(plan.argv.iter().any(|arg| arg == "--output-last-message"));
        assert!(plan.argv.iter().any(|arg| arg == "--skip-git-repo-check"));
        assert!(!plan.prompt_in_command);
        assert!(!plan.shell_invocation);

        let confirmation = confirm_manual_relay_once(
            fixture_confirm_input(&preview.envelope),
            "2026-06-18T00:00:01Z",
        )
        .expect("confirmation should pass");
        let mut run_input = fixture_run_input(&preview.envelope, &confirmation);
        run_input.mock_behavior = format!("mock_codex_process:{}", script.display());
        let receipt = run_manual_relay_once(run_input, "2026-06-18T00:00:02Z")
            .expect("B1 mock runner should complete");
        assert_eq!(receipt.status, "completed_mock_codex");
        assert_eq!(receipt.process_kind, "mock_codex");
        assert!(receipt.prompt_sent);
        assert!(!receipt.real_codex_executed);
        assert_eq!(
            receipt.readback_status,
            "workbench_managed_last_message_available"
        );
        assert!(receipt.last_message_hash.is_some());
        assert!(receipt.last_message_size_bytes.unwrap_or_default() > 0);
    }

    #[test]
    fn manual_relay_b1_runner_entry_stop_kills_mock_process() {
        let _guard = test_guard();
        let script = mock_codex_script(
            "b1-sleep",
            r#"#!/bin/sh
last=""
while [ "$#" -gt 0 ]; do
  if [ "$1" = "--output-last-message" ]; then
    shift
    last="$1"
  fi
  shift || true
done
if [ -n "$last" ]; then
  mkdir -p "$(dirname "$last")"
  printf 'manual relay b1 mock started\n' > "$last"
fi
sleep 30
"#,
        );
        let preview_input =
            b1_real_relay_fixture_preview_input("mock-stop").expect("B1 fixture input");
        let preview = preview_manual_relay(preview_input, "2026-06-18T00:00:00Z");
        let confirmation = confirm_manual_relay_once(
            fixture_confirm_input(&preview.envelope),
            "2026-06-18T00:00:01Z",
        )
        .expect("confirmation should pass");
        let mut run_input = fixture_run_input(&preview.envelope, &confirmation);
        run_input.mock_behavior = format!("mock_codex_process_sleep:{}", script.display());
        let running = run_manual_relay_once(run_input, "2026-06-18T00:00:02Z")
            .expect("B1 mock process should start");
        assert_eq!(running.status, "running");
        assert!(running.prompt_sent);
        assert!(!running.real_codex_executed);

        let stopped = stop_manual_relay_attempt(
            ManualRelayStopInput {
                relay_attempt_id: running.relay_attempt_id,
                requested_by: "user".to_string(),
            },
            "2026-06-18T00:00:03Z",
        )
        .expect("stop should kill only the B1 mock attempt");
        assert_eq!(stopped.status, "stopped_by_user");
        assert!(stopped.real_process_killed);
        assert_eq!(stopped.process_kind, "mock_codex");
        assert!(!stopped.real_codex_executed);
    }

    #[test]
    #[ignore = "real Codex relay is a separate user-present window; this gate proves env authorization is required"]
    fn manual_relay_real_codex_requires_env_authorization() {
        let _guard = test_guard();
        let confirmation = std::env::var("MANUAL_RELAY_REAL_CODEX_CONFIRM")
            .expect("MANUAL_RELAY_REAL_CODEX_CONFIRM is required");
        assert_eq!(confirmation, "CONFIRMED_USER_PRESENT_REAL_RELAY");

        let preview = preview_manual_relay(
            existing_fixture_preview_input("real codex env-gated placeholder"),
            "2026-06-18T00:00:00Z",
        );
        assert!(preview.envelope.target_binding.path_verified);
        let plan = preview
            .guard
            .command_plan
            .as_ref()
            .expect("env-gated runner should expose a codex command plan");
        assert_eq!(plan.program, "codex");
        assert!(!plan.prompt_in_command);
        assert!(!plan.shell_invocation);
        assert_eq!(
            plan.stdin_prompt_sha256,
            preview.envelope.payload.prompt_sha256
        );
        let process_config =
            process_config_for_mode(ManualRelayProcessMode::RealCodexEnvGated, plan.clone())
                .expect("env authorization should unlock the real Codex process config");
        assert_eq!(process_config.command_plan.program, "codex");
        assert_eq!(process_config.process_kind, "real_codex");
        assert!(process_config.real_codex_executed);
        assert!(process_config.return_running);
    }

    #[test]
    #[ignore = "B1 first true Codex relay requires user-present env authorization; do not run in implementation package"]
    fn manual_relay_b1_real_codex_runner_entry_requires_user_present_env() {
        let _guard = test_guard();
        let confirmation = std::env::var("MANUAL_RELAY_REAL_CODEX_CONFIRM")
            .expect("MANUAL_RELAY_REAL_CODEX_CONFIRM is required");
        assert_eq!(confirmation, "CONFIRMED_USER_PRESENT_REAL_RELAY");

        let preview_input = b1_real_relay_fixture_preview_input("real-codex")
            .expect("B1 real relay fixture input should be built");
        let project_root = PathBuf::from(&preview_input.target_project_root);
        let hello_path = project_root.join("hello.txt");
        if hello_path.exists() {
            std::fs::remove_file(&hello_path).expect("stale hello.txt should be removable");
        }
        let preview = preview_manual_relay(preview_input, "2026-06-18T00:00:00Z");
        assert!(preview.envelope.target_binding.path_verified);
        let confirmation = confirm_manual_relay_once(
            fixture_confirm_input(&preview.envelope),
            "2026-06-18T00:00:01Z",
        )
        .expect("confirmation should pass");
        let mut run_input = fixture_run_input(&preview.envelope, &confirmation);
        run_input.mock_behavior = "real_codex_env_gated".to_string();
        let running = run_manual_relay_once(run_input, "2026-06-18T00:00:02Z")
            .expect("B1 user-present env should spawn real Codex");
        assert_eq!(running.status, "running");
        assert_eq!(running.process_kind, "real_codex");
        assert!(running.process_id.is_some());
        assert!(running.prompt_sent);
        assert!(running.real_codex_executed);

        let completed = wait_manual_relay_attempt_for_test(
            &running.relay_attempt_id,
            "2026-06-18T00:01:02Z",
            60_000,
        )
        .expect("B1 real Codex attempt should finish or classify cleanly");
        assert_eq!(completed.status, "completed_real_codex");
        assert!(completed.real_codex_executed);
        assert_eq!(
            completed.readback_status,
            "workbench_managed_last_message_available"
        );
        assert!(completed.last_message_hash.is_some());
        assert!(hello_path.exists());
        assert_eq!(
            std::fs::read_to_string(&hello_path).expect("hello.txt should be readable"),
            "hi\n"
        );
    }

    fn fixture_preview_input(prompt: &str) -> ManualRelayPreviewInput {
        let suffix = short_hash(prompt);
        let project_root = std::env::temp_dir().join(format!("manual-relay-project-{suffix}"));
        let session_id = format!("session:manual-relay-fixture:{suffix}");
        ManualRelayPreviewInput {
            original_user_text: prompt.to_string(),
            target_project_root: project_root.display().to_string(),
            target_cwd: project_root.display().to_string(),
            target_session_id: Some(session_id),
            new_session: false,
            sandbox: "workspace-write".to_string(),
            allowed_write_roots: vec![project_root.display().to_string()],
            requested_by: "user".to_string(),
        }
    }

    fn existing_fixture_preview_input(prompt: &str) -> ManualRelayPreviewInput {
        let suffix = short_hash(prompt);
        let project_root = std::env::temp_dir().join(format!("manual-relay-existing-{suffix}"));
        std::fs::create_dir_all(&project_root).expect("fixture project root should be created");
        let session_id = format!("session:manual-relay-fixture:{suffix}");
        ManualRelayPreviewInput {
            original_user_text: prompt.to_string(),
            target_project_root: project_root.display().to_string(),
            target_cwd: project_root.display().to_string(),
            target_session_id: Some(session_id),
            new_session: false,
            sandbox: "workspace-write".to_string(),
            allowed_write_roots: vec![project_root.display().to_string()],
            requested_by: "user".to_string(),
        }
    }

    fn mock_codex_script(name: &str, body: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "manual-relay-mock-codex-{}",
            short_hash(&format!("{name}:{body}"))
        ));
        std::fs::create_dir_all(&dir).expect("mock codex dir should be created");
        let script = dir.join("mock-codex.sh");
        std::fs::write(&script, body).expect("mock codex script should be written");
        let mut permissions = std::fs::metadata(&script)
            .expect("mock codex script metadata should exist")
            .permissions();
        permissions.set_mode(0o700);
        std::fs::set_permissions(&script, permissions)
            .expect("mock codex script should be executable");
        script
    }

    fn b1_real_relay_fixture_preview_input(label: &str) -> Result<ManualRelayPreviewInput, String> {
        let project_root = std::env::temp_dir().join(format!(
            "manual-relay-b1-real-fixture-{}",
            short_hash(label)
        ));
        std::fs::create_dir_all(&project_root)
            .map_err(|error| format!("b1_fixture_dir_create_failed:{error}"))?;
        let init_status = std::process::Command::new("git")
            .arg("init")
            .arg("--quiet")
            .current_dir(&project_root)
            .status()
            .map_err(|error| format!("b1_fixture_git_init_spawn_failed:{error}"))?;
        if !init_status.success() {
            return Err(format!(
                "b1_fixture_git_init_failed:{}",
                init_status.code().unwrap_or(-1)
            ));
        }
        let hello_path = project_root.join("hello.txt");
        if hello_path.exists() {
            std::fs::remove_file(&hello_path)
                .map_err(|error| format!("b1_fixture_hello_cleanup_failed:{error}"))?;
        }
        Ok(ManualRelayPreviewInput {
            original_user_text: "Create a file named hello.txt in the current directory containing exactly one line: hi\nThen reply with MANUAL_RELAY_B1_REAL_CODEX_OK.".to_string(),
            target_project_root: project_root.display().to_string(),
            target_cwd: project_root.display().to_string(),
            target_session_id: None,
            new_session: true,
            sandbox: "workspace-write".to_string(),
            allowed_write_roots: vec![project_root.display().to_string()],
            requested_by: "user".to_string(),
        })
    }

    fn wait_manual_relay_attempt_for_test(
        relay_attempt_id: &str,
        timestamp: &str,
        timeout_ms: u64,
    ) -> Result<ManualRelayReceipt, String> {
        let started = Instant::now();
        let timeout = Duration::from_millis(timeout_ms.max(1));
        loop {
            let status = {
                let mut registry = active_attempts()
                    .lock()
                    .map_err(|_| "manual_relay_registry_poisoned".to_string())?;
                let active = registry
                    .get_mut(relay_attempt_id)
                    .ok_or_else(|| "manual_relay_attempt_not_running".to_string())?;
                let child = active
                    .child
                    .as_mut()
                    .ok_or_else(|| "manual_relay_attempt_has_no_process".to_string())?;
                child
                    .try_wait()
                    .map_err(|error| format!("manual_relay_process_wait_failed:{error}"))?
            };
            if let Some(status) = status {
                let active = active_attempts()
                    .lock()
                    .map_err(|_| "manual_relay_registry_poisoned".to_string())?
                    .remove(relay_attempt_id)
                    .ok_or_else(|| "manual_relay_attempt_not_running".to_string())?;
                return Ok(finalize_manual_relay_attempt_for_test(
                    active.receipt,
                    timestamp,
                    status.code(),
                    false,
                    false,
                ));
            }
            if started.elapsed() >= timeout {
                let mut active = active_attempts()
                    .lock()
                    .map_err(|_| "manual_relay_registry_poisoned".to_string())?
                    .remove(relay_attempt_id)
                    .ok_or_else(|| "manual_relay_attempt_not_running".to_string())?;
                let mut exit_code = None;
                let mut killed = false;
                if let Some(mut child) = active.child.take() {
                    killed = child.kill().is_ok();
                    exit_code = child.wait().ok().and_then(|status| status.code());
                }
                return Ok(finalize_manual_relay_attempt_for_test(
                    active.receipt,
                    timestamp,
                    exit_code,
                    true,
                    killed,
                ));
            }
            thread::sleep(Duration::from_millis(100));
        }
    }

    fn finalize_manual_relay_attempt_for_test(
        mut receipt: ManualRelayReceipt,
        timestamp: &str,
        exit_code: Option<i32>,
        timed_out: bool,
        killed: bool,
    ) -> ManualRelayReceipt {
        let (readback_status, last_message_hash, last_message_size_bytes) =
            read_last_message_summary(&receipt.command_plan.last_message_path);
        receipt.ended_at = Some(timestamp.to_string());
        receipt.exit_code = exit_code;
        receipt.timed_out = timed_out;
        receipt.real_process_killed = killed;
        receipt.readback_status = if timed_out {
            "readback_timed_out".to_string()
        } else {
            readback_status
        };
        receipt.last_message_hash = last_message_hash;
        receipt.last_message_size_bytes = last_message_size_bytes;
        receipt.status = if timed_out {
            "timed_out".to_string()
        } else if exit_code == Some(0) && receipt.last_message_hash.is_some() {
            match receipt.process_kind.as_str() {
                "real_codex" => "completed_real_codex".to_string(),
                "mock_codex" => "completed_mock_codex".to_string(),
                _ => "completed_process".to_string(),
            }
        } else if exit_code == Some(0) {
            "readback_unavailable".to_string()
        } else {
            "failed_process".to_string()
        };
        receipt
            .warnings
            .push("readback_last_message_only_no_full_transcript".to_string());
        receipt.warnings.sort();
        receipt.warnings.dedup();
        receipt
    }

    fn fixture_confirm_input(envelope: &ManualRelayEnvelope) -> ManualRelayConfirmInput {
        ManualRelayConfirmInput {
            envelope: envelope.clone(),
            actor_ref: "user".to_string(),
            target_hash: envelope.target_binding.target_hash.clone(),
            prompt_sha256: envelope.payload.prompt_sha256.clone(),
            sandbox: envelope.target_binding.sandbox.clone(),
            allowed_write_roots: envelope.target_binding.allowed_write_roots.clone(),
            risk_acknowledged: true,
        }
    }

    fn fixture_run_input(
        envelope: &ManualRelayEnvelope,
        confirmation: &ManualRelayConfirmation,
    ) -> ManualRelayRunInput {
        ManualRelayRunInput {
            envelope: envelope.clone(),
            confirmation: confirmation.clone(),
            confirmation_id: confirmation.confirmation_id.clone(),
            expected_prompt_sha256: envelope.payload.prompt_sha256.clone(),
            expected_target_hash: envelope.target_binding.target_hash.clone(),
            expected_sandbox: envelope.target_binding.sandbox.clone(),
            expected_allowed_write_roots: envelope.target_binding.allowed_write_roots.clone(),
            mock_behavior: "complete_clean_tree".to_string(),
        }
    }

    fn test_guard() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        let guard = LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .expect("manual relay test lock should not poison");
        active_attempts()
            .lock()
            .expect("manual relay registry should not poison")
            .clear();
        consumed_confirmations()
            .lock()
            .expect("manual relay confirmation registry should not poison")
            .clear();
        guard
    }
}
