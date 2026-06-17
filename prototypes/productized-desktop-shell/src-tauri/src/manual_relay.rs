use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use crate::utils::hash::{sha256_hex, short_hash};

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
    let project_root = normalize_path_text(&input.target_project_root);
    let target_cwd = normalize_path_text(&input.target_cwd);
    let allowed_write_roots = input
        .allowed_write_roots
        .iter()
        .map(|root| normalize_path_text(root))
        .collect::<Vec<_>>();
    let prompt_sha256 = sha256_hex(&input.original_user_text);
    let target_hash = sha256_hex(&format!(
        "{}\n{}\n{}\n{}\n{}",
        project_root,
        target_cwd,
        input.target_session_id.as_deref().unwrap_or("new-session"),
        input.new_session,
        input.sandbox
    ));
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
            project_root_canonical: project_root.clone(),
            target_cwd_canonical: target_cwd.clone(),
            target_session_id: input.target_session_id.clone(),
            new_session: input.new_session,
            sandbox: input.sandbox.clone(),
            allowed_write_roots: allowed_write_roots.clone(),
            target_hash,
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
    validate_run_binding(&input)?;
    let scope = input.envelope.policy.duplicate_scope.clone();
    {
        let registry = active_attempts()
            .lock()
            .map_err(|_| "manual_relay_registry_poisoned".to_string())?;
        if registry
            .values()
            .any(|attempt| attempt.duplicate_scope == scope && attempt.status == "running")
        {
            return Err("manual_relay_duplicate_running_attempt".to_string());
        }
    }
    {
        let consumed = consumed_confirmations()
            .lock()
            .map_err(|_| "manual_relay_confirmation_registry_poisoned".to_string())?;
        if consumed.contains_key(&input.confirmation_id) {
            return Err("manual_relay_confirmation_already_consumed".to_string());
        }
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
        let mut registry = active_attempts()
            .lock()
            .map_err(|_| "manual_relay_registry_poisoned".to_string())?;
        registry.insert(
            attempt_id.clone(),
            ActiveManualRelayAttempt {
                duplicate_scope: scope,
                status: "running".to_string(),
                receipt: receipt.clone(),
            },
        );
    }
    let mut consumed = consumed_confirmations()
        .lock()
        .map_err(|_| "manual_relay_confirmation_registry_poisoned".to_string())?;
    consumed.insert(input.confirmation_id.clone(), attempt_id);

    Ok(receipt)
}

pub(crate) fn stop_manual_relay_attempt(
    input: ManualRelayStopInput,
    _timestamp: &str,
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
    receipt.status = "stopped_by_user".to_string();
    receipt.ended_at = Some("manual_relay_stop_requested".to_string());
    receipt.exit_code = None;
    receipt.killed_by_user = true;
    receipt.prompt_sent = false;
    receipt.real_codex_executed = false;
    receipt.syn_read_codex_home = false;
    receipt.syn_wrote_codex_home = false;
    receipt
        .warnings
        .push("manual_relay_stop_killed_only_requested_attempt".to_string());
    receipt.warnings.sort();
    receipt.warnings.dedup();
    Ok(receipt)
}

#[derive(Clone, Debug)]
struct ActiveManualRelayAttempt {
    duplicate_scope: String,
    status: String,
    receipt: ManualRelayReceipt,
}

fn active_attempts() -> &'static Mutex<BTreeMap<String, ActiveManualRelayAttempt>> {
    static REGISTRY: OnceLock<Mutex<BTreeMap<String, ActiveManualRelayAttempt>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(BTreeMap::new()))
}

fn consumed_confirmations() -> &'static Mutex<BTreeMap<String, String>> {
    static REGISTRY: OnceLock<Mutex<BTreeMap<String, String>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(BTreeMap::new()))
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

fn normalize_path_text(value: &str) -> String {
    let path = PathBuf::from(value.trim());
    let absolute = if path.is_absolute() {
        path
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("/"))
            .join(path)
    };
    std::fs::canonicalize(&absolute)
        .unwrap_or_else(|_| clean_path(&absolute))
        .display()
        .to_string()
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
    use std::sync::{Mutex, OnceLock};

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
        assert!(duplicate.contains("manual_relay_duplicate_running_attempt"));
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
