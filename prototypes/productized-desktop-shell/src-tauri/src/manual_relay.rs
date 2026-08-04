use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{btree_map::Entry, BTreeMap};
use std::fs;
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::{Duration, Instant};

use crate::utils::hash::{sha256_hex, short_hash};
#[cfg(unix)]
use std::os::unix::process::CommandExt;

pub(crate) mod conversation_transport;

const MANUAL_RELAY_REAL_CODEX_CONFIRM_ENV: &str = "MANUAL_RELAY_REAL_CODEX_CONFIRM";
const MANUAL_RELAY_REAL_CODEX_CONFIRM_VALUE: &str = "CONFIRMED_USER_PRESENT_REAL_RELAY";
const SUPERVISOR_CAPTURE_MAX_TOTAL_BYTES: usize = 64 * 1024;
const SUPERVISOR_CAPTURE_MAX_FRAME_BYTES: usize = 8 * 1024;
const SUPERVISOR_CAPTURE_MAX_LIVE_EVENTS: usize = 128;
static SUPERVISOR_COLLISION_RECOVERY_NONCE: AtomicU64 = AtomicU64::new(1);

#[cfg(test)]
#[derive(Clone, Copy, PartialEq, Eq)]
enum SupervisorRelaySpawnTestFailurePoint {
    StdinWrite,
}

#[cfg(test)]
thread_local! {
    static SUPERVISOR_RELAY_SPAWN_TEST_FAILURE: std::cell::Cell<Option<SupervisorRelaySpawnTestFailurePoint>> = const { std::cell::Cell::new(None) };
}

#[cfg(test)]
struct SupervisorRelaySpawnTestFailureGuard;

#[cfg(test)]
impl Drop for SupervisorRelaySpawnTestFailureGuard {
    fn drop(&mut self) {
        SUPERVISOR_RELAY_SPAWN_TEST_FAILURE.with(|failure| failure.set(None));
    }
}

#[cfg(test)]
fn force_supervisor_relay_spawn_test_failure(
    point: SupervisorRelaySpawnTestFailurePoint,
) -> SupervisorRelaySpawnTestFailureGuard {
    SUPERVISOR_RELAY_SPAWN_TEST_FAILURE.with(|failure| failure.set(Some(point)));
    SupervisorRelaySpawnTestFailureGuard
}

#[cfg(test)]
fn supervisor_relay_spawn_test_failure_active(point: SupervisorRelaySpawnTestFailurePoint) -> bool {
    SUPERVISOR_RELAY_SPAWN_TEST_FAILURE.with(|failure| failure.get() == Some(point))
}

// Simulates a same-key active-attempt claimant after the ordinary duplicate
// check but before the supervisor reserves its pre-spawn slot. The fixture
// proves that the slot reservation itself rejects the collision, before any
// child, durable registration, or capture can exist.
#[cfg(test)]
thread_local! {
    static SUPERVISOR_RELAY_ACTIVE_SLOT_COLLISION: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

#[cfg(test)]
struct SupervisorRelayActiveSlotCollisionGuard;

#[cfg(test)]
impl Drop for SupervisorRelayActiveSlotCollisionGuard {
    fn drop(&mut self) {
        SUPERVISOR_RELAY_ACTIVE_SLOT_COLLISION.with(|enabled| enabled.set(false));
    }
}

#[cfg(test)]
fn force_supervisor_relay_active_slot_collision_for_test() -> SupervisorRelayActiveSlotCollisionGuard
{
    SUPERVISOR_RELAY_ACTIVE_SLOT_COLLISION.with(|enabled| enabled.set(true));
    SupervisorRelayActiveSlotCollisionGuard
}

#[cfg(test)]
fn install_supervisor_relay_active_slot_collision_for_test(
    attempt_id: &str,
    receipt: &ManualRelayReceipt,
) {
    let enabled = SUPERVISOR_RELAY_ACTIVE_SLOT_COLLISION.with(std::cell::Cell::get);
    if !enabled {
        return;
    }
    let mut registry = active_attempts()
        .lock()
        .expect("fixture active-attempt registry lock");
    registry.entry(attempt_id.to_string()).or_insert_with(|| {
        let mut existing_receipt = receipt.clone();
        existing_receipt.status = "running".to_string();
        ActiveManualRelayAttempt {
            duplicate_scope: format!("fixture-active-slot-collision:{attempt_id}"),
            status: "running".to_string(),
            receipt: existing_receipt,
            child: None,
            completed_status: "fixture-collision".to_string(),
            output_paths: None,
            supervisor_capture: None,
            process_registration: None,
        }
    });
}

// This second fixture bypasses the pre-spawn reservation deliberately, so
// the final-install fallback is exercised against a future caller that might
// forget the reservation protocol. Production code has no such insertion
// path; the test proves the fallback re-keys the new child without replacing
// the old owner.
#[cfg(test)]
thread_local! {
    static SUPERVISOR_RELAY_FINAL_ACTIVE_SLOT_COLLISION: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

#[cfg(test)]
struct SupervisorRelayFinalActiveSlotCollisionGuard;

#[cfg(test)]
impl Drop for SupervisorRelayFinalActiveSlotCollisionGuard {
    fn drop(&mut self) {
        SUPERVISOR_RELAY_FINAL_ACTIVE_SLOT_COLLISION.with(|enabled| enabled.set(false));
    }
}

#[cfg(test)]
fn force_supervisor_relay_final_active_slot_collision_for_test(
) -> SupervisorRelayFinalActiveSlotCollisionGuard {
    SUPERVISOR_RELAY_FINAL_ACTIVE_SLOT_COLLISION.with(|enabled| enabled.set(true));
    SupervisorRelayFinalActiveSlotCollisionGuard
}

#[cfg(test)]
fn install_supervisor_relay_final_active_slot_collision_for_test(
    attempt_id: &str,
    receipt: &ManualRelayReceipt,
) {
    let enabled = SUPERVISOR_RELAY_FINAL_ACTIVE_SLOT_COLLISION.with(std::cell::Cell::get);
    if !enabled {
        return;
    }
    let mut registry = active_attempts()
        .lock()
        .expect("fixture active-attempt registry lock");
    let mut existing_receipt = receipt.clone();
    existing_receipt.confirmation_id = format!("fixture-foreign-confirmation:{attempt_id}");
    existing_receipt.status = "running".to_string();
    registry.insert(
        attempt_id.to_string(),
        ActiveManualRelayAttempt {
            duplicate_scope: format!("fixture-final-active-slot-collision:{attempt_id}"),
            status: "running".to_string(),
            receipt: existing_receipt,
            child: None,
            completed_status: "fixture-foreign-collision".to_string(),
            output_paths: None,
            supervisor_capture: None,
            process_registration: None,
        },
    );
}

#[cfg(test)]
thread_local! {
    static SUPERVISOR_RELAY_USE_TEMPORARY_DURABLE_REGISTRATION: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

#[cfg(test)]
struct SupervisorRelayTemporaryDurableRegistrationGuard;

#[cfg(test)]
impl Drop for SupervisorRelayTemporaryDurableRegistrationGuard {
    fn drop(&mut self) {
        SUPERVISOR_RELAY_USE_TEMPORARY_DURABLE_REGISTRATION.with(|enabled| enabled.set(false));
    }
}

/// Tests exercise the real sidecar unregister lifecycle without probing a
/// live `ps` identity or ever touching the application's workflow state.
#[cfg(test)]
fn force_supervisor_relay_temporary_durable_registration_for_test(
) -> SupervisorRelayTemporaryDurableRegistrationGuard {
    SUPERVISOR_RELAY_USE_TEMPORARY_DURABLE_REGISTRATION.with(|enabled| enabled.set(true));
    SupervisorRelayTemporaryDurableRegistrationGuard
}

#[cfg(test)]
thread_local! {
    static MANUAL_RELAY_CHILD_STOP_TEST_FAILURES_REMAINING: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
pub(crate) struct ManualRelayChildStopTestFailureGuard;

#[cfg(test)]
impl Drop for ManualRelayChildStopTestFailureGuard {
    fn drop(&mut self) {
        MANUAL_RELAY_CHILD_STOP_TEST_FAILURES_REMAINING.with(|failure| failure.set(0));
    }
}

#[cfg(test)]
fn force_manual_relay_child_stop_test_failure() -> ManualRelayChildStopTestFailureGuard {
    force_manual_relay_child_stop_test_failures_for_test(1)
}

/// A deterministic bounded failure sequence proves that a supervisor attempt
/// keeps its protected resources when both the immediate stop and its trusted
/// reverse retry cannot confirm the child has exited.
#[cfg(test)]
pub(crate) fn force_manual_relay_child_stop_test_failures_for_test(
    count: usize,
) -> ManualRelayChildStopTestFailureGuard {
    MANUAL_RELAY_CHILD_STOP_TEST_FAILURES_REMAINING.with(|failure| failure.set(count));
    ManualRelayChildStopTestFailureGuard
}

#[cfg(test)]
fn take_manual_relay_child_stop_test_failure() -> bool {
    MANUAL_RELAY_CHILD_STOP_TEST_FAILURES_REMAINING.with(|failure| {
        let remaining = failure.get();
        if remaining == 0 {
            return false;
        }
        failure.set(remaining - 1);
        true
    })
}

// The public generic relay entry can still reach `register_running_attempt_once`
// with a fixture `stay_running` request.  Exercise that exact write path at
// the moment a protected supervisor attempt is being retained after failed
// cleanup, when its active-map slot is deliberately absent but its marker
// must remain authoritative.
#[cfg(test)]
thread_local! {
    static SUPERVISOR_RELAY_RETAIN_COMPETING_NON_SAFE_INSERT: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static SUPERVISOR_RELAY_RETAIN_COMPETING_NON_SAFE_INSERT_RESULT: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
}

#[cfg(test)]
struct SupervisorRelayRetainCompetingNonSafeInsertGuard;

#[cfg(test)]
impl Drop for SupervisorRelayRetainCompetingNonSafeInsertGuard {
    fn drop(&mut self) {
        SUPERVISOR_RELAY_RETAIN_COMPETING_NON_SAFE_INSERT.with(|enabled| enabled.set(false));
        SUPERVISOR_RELAY_RETAIN_COMPETING_NON_SAFE_INSERT_RESULT
            .with(|result| *result.borrow_mut() = None);
    }
}

#[cfg(test)]
fn force_supervisor_relay_retain_competing_non_safe_insert_for_test(
) -> SupervisorRelayRetainCompetingNonSafeInsertGuard {
    SUPERVISOR_RELAY_RETAIN_COMPETING_NON_SAFE_INSERT.with(|enabled| enabled.set(true));
    SUPERVISOR_RELAY_RETAIN_COMPETING_NON_SAFE_INSERT_RESULT
        .with(|result| *result.borrow_mut() = None);
    SupervisorRelayRetainCompetingNonSafeInsertGuard
}

#[cfg(test)]
fn supervisor_relay_retain_competing_non_safe_insert_result_for_test() -> Option<String> {
    SUPERVISOR_RELAY_RETAIN_COMPETING_NON_SAFE_INSERT_RESULT.with(|result| result.borrow().clone())
}

/// This policy is constructed only by the host-selected supervisor profile.
/// It is intentionally not serializable or debuggable because the markers can
/// contain the relay endpoint and grant used solely by the child process.
#[derive(Clone)]
pub(super) struct SupervisorRelayExecutionPolicy {
    redaction_markers: Vec<String>,
}

impl SupervisorRelayExecutionPolicy {
    pub(super) fn new(redaction_markers: Vec<String>) -> Self {
        Self { redaction_markers }
    }
}

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
pub(crate) struct ManualRelayGuiDirectNewSessionInput {
    pub(crate) original_user_text: String,
    pub(crate) target_project_root: String,
    pub(crate) target_cwd: String,
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
    pub(crate) assistant_message_text: Option<String>,
    pub(crate) thread_event_summary: ManualRelayThreadEventSummary,
    pub(crate) live_events: Vec<ManualRelayLiveEvent>,
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
pub(crate) struct ManualRelayThreadEventSummary {
    pub(crate) thread_id: Option<String>,
    pub(crate) assistant_item_id: Option<String>,
    pub(crate) assistant_message_text: Option<String>,
    pub(crate) turn_completed: bool,
    pub(crate) turn_failed: bool,
    pub(crate) usage: BTreeMap<String, i64>,
    pub(crate) event_types: Vec<String>,
    pub(crate) json_line_count: i64,
    pub(crate) malformed_json_line_count: i64,
    pub(crate) stderr_summary: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct ManualRelayLiveEvent {
    pub(crate) sequence: i64,
    pub(crate) event_type: String,
    pub(crate) thread_id: Option<String>,
    pub(crate) item_id: Option<String>,
    pub(crate) item_type: Option<String>,
    pub(crate) title: String,
    pub(crate) text: Option<String>,
    pub(crate) delta: Option<String>,
    pub(crate) tool_name: Option<String>,
    pub(crate) arguments_preview: Option<String>,
    pub(crate) output_preview: Option<String>,
    pub(crate) stdout: Option<String>,
    pub(crate) stderr: Option<String>,
    pub(crate) exit_code: Option<i32>,
    pub(crate) status: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ManualRelayThreadEventReport {
    summary: ManualRelayThreadEventSummary,
    live_events: Vec<ManualRelayLiveEvent>,
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct ManualRelayPollInput {
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

pub(crate) fn run_manual_relay_gui_direct_new_session_once(
    input: ManualRelayGuiDirectNewSessionInput,
    timestamp: &str,
) -> Result<ManualRelayReceipt, String> {
    run_manual_relay_gui_direct_new_session_once_with_process_mode(
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

#[cfg(test)]
fn run_manual_relay_gui_direct_new_session_once_for_test(
    input: ManualRelayGuiDirectNewSessionInput,
    timestamp: &str,
    mock_behavior: &str,
) -> Result<ManualRelayReceipt, String> {
    let Some(process_mode) = manual_relay_process_mode(mock_behavior) else {
        return Err("manual_relay_gui_direct_test_process_mode_required".to_string());
    };
    if matches!(process_mode, ManualRelayProcessMode::RealCodexEnvGated) {
        return Err("manual_relay_gui_direct_test_must_not_use_real_codex".to_string());
    }
    run_manual_relay_gui_direct_new_session_once_with_process_mode(input, timestamp, process_mode)
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

fn run_manual_relay_gui_direct_new_session_once_with_process_mode(
    input: ManualRelayGuiDirectNewSessionInput,
    timestamp: &str,
    process_mode: ManualRelayProcessMode,
) -> Result<ManualRelayReceipt, String> {
    validate_gui_direct_new_session_input(&input)?;
    let preview = preview_manual_relay(
        ManualRelayPreviewInput {
            original_user_text: input.original_user_text,
            target_project_root: input.target_project_root,
            target_cwd: input.target_cwd,
            target_session_id: None,
            new_session: true,
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
    validate_gui_direct_new_session_target_and_command_plan(&preview.envelope, command_plan)?;
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
    run_manual_relay_once_with_process_mode_and_command_profile(
        input,
        timestamp,
        process_mode_override,
        None,
    )
}

/// 共享 Conversation Transport 复用现有 relay 的进程生命周期、JSONL 解析、poll 和 Stop。
///
/// 旧入口传 `None`，保持既有 command plan 不变；新的 profile 只能在这里把宿主已冻结
/// 的额外 argv 加到 guard 产出的基础 plan 上。这样 supervisor profile 不需要改动
/// `codex_local_runner`，也不会把安全参数开放给前端。
fn run_manual_relay_once_with_process_mode_and_command_profile(
    input: ManualRelayRunInput,
    timestamp: &str,
    process_mode_override: Option<ManualRelayProcessMode>,
    command_profile: Option<&conversation_transport::ConversationTransportCommandProfile>,
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
    let Some(mut command_plan) = guard.command_plan.clone() else {
        return Err("manual_relay_command_plan_missing".to_string());
    };
    if let Some(command_profile) = command_profile {
        conversation_transport::apply_command_profile(
            &input.envelope,
            &mut command_plan,
            command_profile,
        )?;
    }

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
            let _visibility_gate = raw_manual_relay_visibility_gate()
                .lock()
                .map_err(|_| "manual_relay_managed_conversation_attempt_protected".to_string())?;
            let markers = safe_only_attempts()
                .lock()
                .map_err(|_| "manual_relay_managed_conversation_attempt_protected".to_string())?;
            if markers.contains_key(&attempt_id) {
                return Err("manual_relay_managed_conversation_attempt_protected".to_string());
            }
            drop(markers);
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
                    completed_status: "completed_process".to_string(),
                    output_paths: None,
                    supervisor_capture: None,
                    process_registration: None,
                },
            );
            consumed.insert(input.confirmation_id.clone(), attempt_id);
            return Ok(receipt);
        }

        let process_config = process_config_for_mode(process_mode, command_plan)?;
        if process_config.return_running {
            let supervisor_policy =
                command_profile.and_then(|profile| profile.supervisor_execution_policy());
            let supervisor_mode = supervisor_policy.is_some();
            let supervisor_start_gate = supervisor_mode
                .then(acquire_supervisor_relay_start_gate)
                .transpose()?;
            if supervisor_mode {
                reserve_safe_only_attempt(&attempt_id, &input.confirmation_id)?;
            }
            let result = spawn_running_codex_like_process(
                &scope,
                &input.confirmation_id,
                attempt_id.clone(),
                &input.envelope,
                process_config,
                timestamp,
                dirty_before,
                supervisor_policy,
                None,
            );
            drop(supervisor_start_gate);
            // A successful supervisor spawn remains safe-receipt-only until
            // its trusted transport reaches a terminal cleanup path.  The
            // inner/outer maps are registered later, so clearing this marker
            // here would reopen raw poll/stop in that handoff window.
            let retain_safe_only_ownership = supervisor_mode
                && (matches!(&result, Ok(_))
                    || matches!(
                        &result,
                        Err(error) if error == "supervisor_relay_cleanup_pending"
                    ));
            if !retain_safe_only_ownership {
                clear_safe_only_attempt(&attempt_id);
                clear_consumed_confirmation_attempt(&input.confirmation_id, &attempt_id);
            }
            return result;
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
                completed_status: "completed_fixture".to_string(),
                output_paths: None,
                supervisor_capture: None,
                process_registration: None,
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
    let attempt_id = input.relay_attempt_id.clone();
    with_raw_manual_relay_attempt_visibility(&attempt_id, || {
        stop_manual_relay_attempt_trusted(input, timestamp)
    })
}

pub(super) fn stop_safe_only_manual_relay_attempt(
    input: ManualRelayStopInput,
    timestamp: &str,
) -> Result<ManualRelayReceipt, String> {
    stop_manual_relay_attempt_trusted(input, timestamp)
}

pub(super) fn abort_safe_only_manual_relay_attempt(
    input: ManualRelayStopInput,
    timestamp: &str,
) -> Result<(), String> {
    let attempt_id = input.relay_attempt_id.clone();
    match stop_manual_relay_attempt_trusted(input, timestamp) {
        Ok(_) => {
            clear_safe_only_attempt(&attempt_id);
            Ok(())
        }
        Err(_) if !safe_only_attempt_is_reserved(&attempt_id) => {
            // A direct stop can report its first failed wait even though the
            // reverse retry fully settled every resource.  The marker is the
            // authoritative closure signal for the outer trusted transport.
            Ok(())
        }
        Err(error) => Err(error),
    }
}

fn stop_manual_relay_attempt_trusted(
    input: ManualRelayStopInput,
    timestamp: &str,
) -> Result<ManualRelayReceipt, String> {
    if input.requested_by.trim().is_empty() {
        return Err("manual_relay_stop_requested_by_missing".to_string());
    }
    let active = active_attempts()
        .lock()
        .map_err(|_| "manual_relay_registry_poisoned".to_string())?
        .remove(&input.relay_attempt_id);
    let Some(mut active) = active else {
        return Err("manual_relay_attempt_not_running".to_string());
    };
    let supervisor_mode = active.supervisor_capture.is_some();
    let had_child = active.child.is_some();
    let stop_result = match active.child.as_mut() {
        Some(child) => stop_manual_relay_child_process(child),
        None => Ok((None, false, Vec::new())),
    };
    let (exit_code, process_killed, mut stop_warnings) = match stop_result {
        Ok(result) => result,
        Err(error) => {
            let _ = cleanup_removed_manual_relay_attempt(active, &input.relay_attempt_id, false);
            return Err(if supervisor_mode {
                "supervisor_relay_cleanup_failed".to_string()
            } else {
                format!("manual_relay_stop_cleanup_failed:{error}")
            });
        }
    };
    let mut receipt = match finish_removed_manual_relay_attempt(
        active,
        &input.relay_attempt_id,
        timestamp,
        exit_code,
        false,
        process_killed,
    ) {
        Ok(receipt) => receipt,
        Err(active) => {
            retain_unsettled_manual_relay_attempt(&input.relay_attempt_id, active);
            return Err(if supervisor_mode {
                "supervisor_relay_cleanup_failed".to_string()
            } else {
                "manual_relay_stop_cleanup_failed:durable_registration_retained".to_string()
            });
        }
    };
    receipt.warnings.append(&mut stop_warnings);
    if !had_child || process_killed {
        receipt.status = "stopped_by_user".to_string();
        receipt.ended_at = Some(timestamp.to_string());
        receipt.killed_by_user = true;
        receipt.syn_read_codex_home = false;
        receipt.syn_wrote_codex_home = false;
        receipt
            .warnings
            .push("manual_relay_stop_killed_only_requested_attempt".to_string());
    }
    receipt.warnings.sort();
    receipt.warnings.dedup();
    Ok(receipt)
}

/// App 正常退出时只清理本进程内登记的 relay 尝试；不扫描、不匹配、更不终止登记外 PID。
/// 硬崩溃来不及走这里时，`process_registration` 留在 sidecar，交给下次启动的 orphan reaper。
pub(crate) fn stop_all_active_manual_relay_attempts() -> Result<usize, String> {
    {
        let mut shutdown = supervisor_relay_shutdown_gate()
            .lock()
            .map_err(|_| "supervisor_relay_shutdown_gate_unavailable".to_string())?;
        *shutdown = true;
    }
    let result = (|| {
        let attempts = {
            let mut registry = active_attempts()
                .lock()
                .map_err(|_| "manual_relay_registry_poisoned".to_string())?;
            std::mem::take(&mut *registry)
        };
        let mut stopped = 0;
        let mut errors = Vec::new();
        for (attempt_id, active) in attempts {
            let had_child = active.child.is_some();
            let cleanup = cleanup_removed_manual_relay_attempt(active, &attempt_id, false);
            if cleanup.errors.is_empty() && had_child {
                stopped += 1;
            }
            errors.extend(
                cleanup
                    .errors
                    .into_iter()
                    .map(|error| format!("{attempt_id}:{error}")),
            );
        }
        if errors.is_empty() {
            Ok(stopped)
        } else {
            Err(format!(
                "manual_relay_shutdown_cleanup_failed:{}",
                errors.join(";")
            ))
        }
    })();
    if let Ok(mut shutdown) = supervisor_relay_shutdown_gate().lock() {
        *shutdown = false;
    }
    result
}

pub(crate) fn poll_manual_relay_attempt(
    input: ManualRelayPollInput,
    timestamp: &str,
) -> Result<ManualRelayReceipt, String> {
    let attempt_id = input.relay_attempt_id.clone();
    with_raw_manual_relay_attempt_visibility(&attempt_id, || {
        poll_manual_relay_attempt_trusted(input, timestamp)
    })
}

pub(super) fn poll_safe_only_manual_relay_attempt(
    input: ManualRelayPollInput,
    timestamp: &str,
) -> Result<ManualRelayReceipt, String> {
    poll_manual_relay_attempt_trusted(input, timestamp)
}

fn poll_manual_relay_attempt_trusted(
    input: ManualRelayPollInput,
    timestamp: &str,
) -> Result<ManualRelayReceipt, String> {
    if input.requested_by.trim().is_empty() {
        return Err("manual_relay_poll_requested_by_missing".to_string());
    }
    let poll_result = {
        let mut registry = active_attempts()
            .lock()
            .map_err(|_| "manual_relay_registry_poisoned".to_string())?;
        let Some(active) = registry.get_mut(&input.relay_attempt_id) else {
            return Err("manual_relay_attempt_not_running".to_string());
        };
        let Some(child) = active.child.as_mut() else {
            return Ok(active.receipt.clone());
        };
        child.try_wait()
    };
    let exit_status = match poll_result {
        Ok(status) => status,
        Err(error) => {
            let active = active_attempts()
                .lock()
                .map_err(|_| "manual_relay_registry_poisoned".to_string())?
                .remove(&input.relay_attempt_id);
            if let Some(active) = active {
                let supervisor_mode = active.supervisor_capture.is_some();
                let _ =
                    cleanup_removed_manual_relay_attempt(active, &input.relay_attempt_id, false);
                return Err(if supervisor_mode {
                    "supervisor_relay_poll_cleanup_failed".to_string()
                } else {
                    format!("manual_relay_process_wait_failed:{error}")
                });
            }
            return Err("manual_relay_attempt_not_running".to_string());
        }
    };
    if let Some(exit_status) = exit_status {
        let mut active = active_attempts()
            .lock()
            .map_err(|_| "manual_relay_registry_poisoned".to_string())?
            .remove(&input.relay_attempt_id)
            .ok_or_else(|| "manual_relay_attempt_not_running".to_string())?;
        let supervisor_mode = active.supervisor_capture.is_some();
        // `try_wait` only establishes that the leader has exited.  Before a
        // terminal receipt joins supervisor capture or unregisters its PGID,
        // sweep the original group so a background descendant cannot survive
        // while retaining a pipe or a secret-bearing child-only configuration.
        let group_cleanup = active
            .child
            .as_mut()
            .map(stop_manual_relay_child_process)
            .transpose();
        let (terminal_exit_code, process_killed, mut group_warnings) = match group_cleanup {
            Ok(Some(result)) => result,
            Ok(None) => (exit_status.code(), false, Vec::new()),
            Err(error) => {
                let cleanup =
                    cleanup_removed_manual_relay_attempt(active, &input.relay_attempt_id, false);
                return Err(if supervisor_mode {
                    if cleanup.errors.is_empty() {
                        "supervisor_relay_poll_cleanup_failed".to_string()
                    } else {
                        "supervisor_relay_poll_cleanup_pending".to_string()
                    }
                } else {
                    format!("manual_relay_process_group_cleanup_failed:{error}")
                });
            }
        };
        return match finish_removed_manual_relay_attempt(
            active,
            &input.relay_attempt_id,
            timestamp,
            terminal_exit_code,
            false,
            process_killed,
        ) {
            Ok(mut receipt) => {
                receipt.warnings.append(&mut group_warnings);
                receipt.warnings.sort();
                receipt.warnings.dedup();
                Ok(receipt)
            }
            Err(active) => {
                retain_unsettled_manual_relay_attempt(&input.relay_attempt_id, active);
                Err(if supervisor_mode {
                    "supervisor_relay_poll_cleanup_failed".to_string()
                } else {
                    "manual_relay_process_unregister_failed".to_string()
                })
            }
        };
    }

    let (mut running_receipt, transport_failed) = {
        let mut registry = active_attempts()
            .lock()
            .map_err(|_| "manual_relay_registry_poisoned".to_string())?;
        let active = registry
            .get_mut(&input.relay_attempt_id)
            .ok_or_else(|| "manual_relay_attempt_not_running".to_string())?;
        refresh_running_receipt_from_output(
            &mut active.receipt,
            active.output_paths.as_ref(),
            active.supervisor_capture.as_ref(),
        );
        (
            active.receipt.clone(),
            active.receipt.thread_event_summary.turn_failed,
        )
    };
    if !transport_failed {
        return Ok(running_receipt);
    }
    let active = active_attempts()
        .lock()
        .map_err(|_| "manual_relay_registry_poisoned".to_string())?
        .remove(&input.relay_attempt_id)
        .ok_or_else(|| "manual_relay_attempt_not_running".to_string())?;
    let supervisor_mode = active.supervisor_capture.is_some();
    let cleanup = cleanup_removed_manual_relay_attempt(active, &input.relay_attempt_id, false);
    if supervisor_mode && !cleanup.errors.is_empty() {
        return Err("supervisor_relay_poll_cleanup_failed".to_string());
    }
    running_receipt.status = "failed_process".to_string();
    running_receipt.ended_at = Some(timestamp.to_string());
    running_receipt.exit_code = cleanup.exit_code;
    running_receipt.real_process_killed = cleanup.process_killed;
    running_receipt
        .warnings
        .push("manual_relay_terminal_thread_failure_reaped".to_string());
    if !cleanup.errors.is_empty() {
        running_receipt
            .warnings
            .push("manual_relay_terminal_cleanup_failed".to_string());
    }
    running_receipt.warnings.sort();
    running_receipt.warnings.dedup();
    Ok(running_receipt)
}

/// A relay attempt is removed from the active map before this function runs.
/// Keeping the reverse cleanup in one place prevents a post-spawn failure from
/// leaving a raw-readable attempt after its safe owner has returned an error.
fn finish_removed_manual_relay_attempt(
    mut active: ActiveManualRelayAttempt,
    attempt_id: &str,
    timestamp: &str,
    exit_code: Option<i32>,
    timed_out: bool,
    killed: bool,
) -> Result<ManualRelayReceipt, ActiveManualRelayAttempt> {
    let confirmation_id = active.receipt.confirmation_id.clone();
    let receipt = finalize_running_codex_like_attempt(
        active.receipt.clone(),
        timestamp,
        exit_code,
        timed_out,
        killed,
        &active.completed_status,
        active.output_paths.as_ref(),
        active.supervisor_capture.as_mut(),
    );
    active.child.take();
    if let Some(registration) = active.process_registration.as_mut() {
        if registration.unregister_preserving_on_error().is_err() {
            active.receipt = receipt;
            return Err(active);
        }
    }
    active.process_registration.take();
    clear_safe_only_attempt(attempt_id);
    clear_consumed_confirmation_attempt(&confirmation_id, attempt_id);
    Ok(receipt)
}

/// Best-effort reverse cleanup for an attempt that cannot produce a normal
/// terminal receipt.  It never returns early: child/process group, durable
/// registration, in-memory capture, safe marker, and confirmation reservation
/// are all visited even when an earlier operation fails.
struct ManualRelayCleanupOutcome {
    errors: Vec<String>,
    exit_code: Option<i32>,
    process_killed: bool,
}

fn cleanup_removed_manual_relay_attempt(
    mut active: ActiveManualRelayAttempt,
    attempt_id: &str,
    child_already_terminated: bool,
) -> ManualRelayCleanupOutcome {
    let confirmation_id = active.receipt.confirmation_id.clone();
    let mut errors = Vec::new();
    let mut child_terminated = child_already_terminated;
    let mut exit_code = None;
    let mut process_killed = false;
    if let Some(child) = active.child.as_mut() {
        match cleanup_manual_relay_child_process(child) {
            Ok((stopped_exit_code, stopped_process_killed, _)) => {
                child_terminated = true;
                exit_code = stopped_exit_code;
                process_killed = stopped_process_killed;
                active.child.take();
            }
            Err(error) => errors.push(format!("child_cleanup_failed:{error}")),
        }
    }
    if child_terminated {
        if let Some(registration) = active.process_registration.as_mut() {
            if let Err(error) = registration.unregister_preserving_on_error() {
                errors.push(format!("durable_unregister_failed:{error}"));
            } else {
                active.process_registration.take();
            }
        }
    } else if active.process_registration.is_some() {
        errors.push("durable_registration_retained_for_orphan_reaper".to_string());
    }
    if let Some(capture) = active.supervisor_capture.as_mut() {
        if child_terminated {
            let _ = capture.finish_snapshot_and_clear();
        } else {
            // Do not block forever joining a reader if a hostile/failed child
            // could still hold the pipe.  `clear` makes subsequent chunks
            // inert and drops all in-memory text immediately.
            capture.clear();
        }
    }
    if errors.is_empty() {
        clear_safe_only_attempt(attempt_id);
        clear_consumed_confirmation_attempt(&confirmation_id, attempt_id);
    } else {
        retain_unsettled_manual_relay_attempt(attempt_id, active);
    }
    ManualRelayCleanupOutcome {
        errors,
        exit_code,
        process_killed,
    }
}

/// Failure closure must remain fail-closed even when a child cannot be
/// confirmed dead.  Keeping the active record preserves its durable entry and
/// lets a trusted safe-only retry finish cleanup; the pre-spawn marker remains
/// until that happens, so raw endpoints never gain a readable half-state.
fn retain_unsettled_manual_relay_attempt(attempt_id: &str, active: ActiveManualRelayAttempt) {
    #[cfg(test)]
    attempt_competing_non_safe_insert_during_supervisor_retain_for_test(attempt_id, &active);

    // Failure closure must not discard the only trusted retry handle merely
    // because an earlier test or caller poisoned the mutex.  The raw surface
    // still fails closed on that mutex; here we recover its contents solely to
    // retain the already protected attempt for its safe-only cleanup path.
    let mut attempts = active_attempts()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    attempts.entry(attempt_id.to_string()).or_insert(active);
}

#[cfg(test)]
fn attempt_competing_non_safe_insert_during_supervisor_retain_for_test(
    attempt_id: &str,
    retained: &ActiveManualRelayAttempt,
) {
    let enabled = SUPERVISOR_RELAY_RETAIN_COMPETING_NON_SAFE_INSERT.with(std::cell::Cell::get);
    if !enabled {
        return;
    }
    let confirmation_id = format!("fixture-competing-non-safe:{attempt_id}");
    let mut receipt = retained.receipt.clone();
    receipt.confirmation_id = confirmation_id.clone();
    let result = register_running_attempt_once(
        &format!("fixture-competing-non-safe:{attempt_id}"),
        &confirmation_id,
        attempt_id.to_string(),
        ActiveManualRelayAttempt {
            duplicate_scope: format!("fixture-competing-non-safe:{attempt_id}"),
            status: "running".to_string(),
            receipt,
            child: None,
            completed_status: "fixture-competing-non-safe".to_string(),
            output_paths: None,
            supervisor_capture: None,
            process_registration: None,
        },
    );
    let observed = match result {
        Ok(()) => "unexpected_competing_non_safe_insert_success".to_string(),
        Err(error) => error,
    };
    SUPERVISOR_RELAY_RETAIN_COMPETING_NON_SAFE_INSERT_RESULT
        .with(|stored| *stored.borrow_mut() = Some(observed));
}

fn configure_manual_relay_process_group(command: &mut Command) {
    #[cfg(unix)]
    {
        command.process_group(0);
    }
    #[cfg(not(unix))]
    {
        let _ = command;
    }
}

fn stop_manual_relay_child_process(
    child: &mut Child,
) -> Result<(Option<i32>, bool, Vec<String>), String> {
    #[cfg(test)]
    if take_manual_relay_child_stop_test_failure() {
        // The production helper always retains this borrowed handle.  Inject
        // before any signal so reverse cleanup must retry with the same child
        // after an arbitrary stop failure.
        return Err("manual_relay_child_stop_test_injected_failure".to_string());
    }
    let mut warnings = Vec::new();
    match child.try_wait() {
        Ok(Some(status)) => {
            // A shell/Codex group leader may exit before a descendant that
            // inherited its stdout/stderr.  Do not finalize the receipt or
            // unregister the durable identity until the original process
            // group is confirmed empty as well.
            let swept = sweep_manual_relay_process_group(child.id(), &mut warnings)?;
            return Ok((status.code(), swept, warnings));
        }
        Ok(None) => {}
        Err(error) => warnings.push(format!("manual_relay_process_wait_probe_failed:{error}")),
    }

    #[cfg(unix)]
    {
        let process_id = child.id();
        let term_signaled = signal_manual_relay_process_group(process_id, "TERM", &mut warnings);
        match wait_manual_relay_child_for(&mut *child, Duration::from_millis(800)) {
            Ok(Some(status)) => {
                let swept = sweep_manual_relay_process_group(process_id, &mut warnings)?;
                return Ok((status.code(), term_signaled || swept, warnings));
            }
            Ok(None) => {}
            Err(error) => warnings.push(format!(
                "manual_relay_process_wait_after_term_failed:{error}"
            )),
        }

        let kill_signaled = sweep_manual_relay_process_group(process_id, &mut warnings)?;
        let child_killed = child.kill().is_ok();
        let status = child
            .wait()
            .map_err(|error| format!("manual_relay_process_wait_failed:{error}"))?;
        return Ok((
            status.code(),
            term_signaled || kill_signaled || child_killed,
            warnings,
        ));
    }

    #[cfg(not(unix))]
    {
        let child_killed = child.kill().is_ok();
        let status = child
            .wait()
            .map_err(|error| format!("manual_relay_process_wait_failed:{error}"))?;
        Ok((status.code(), child_killed, warnings))
    }
}

/// The final `wait` may itself fail after TERM/KILL was sent.  Retain the
/// child handle and make one more bounded stop attempt before reverse cleanup
/// releases its durable registration or safe-only marker.
fn cleanup_manual_relay_child_process(
    child: &mut Child,
) -> Result<(Option<i32>, bool, Vec<String>), String> {
    match stop_manual_relay_child_process(child) {
        Ok(result) => Ok(result),
        Err(first_error) => stop_manual_relay_child_process(child)
            .map_err(|second_error| format!("{first_error};retry:{second_error}")),
    }
}

fn wait_manual_relay_child_for(
    child: &mut Child,
    timeout: Duration,
) -> Result<Option<std::process::ExitStatus>, String> {
    let started = Instant::now();
    loop {
        if let Some(status) = child
            .try_wait()
            .map_err(|error| format!("manual_relay_process_wait_failed:{error}"))?
        {
            return Ok(Some(status));
        }
        if started.elapsed() >= timeout {
            return Ok(None);
        }
        thread::sleep(Duration::from_millis(25));
    }
}

#[cfg(unix)]
fn signal_manual_relay_process_group(
    process_id: u32,
    signal: &str,
    warnings: &mut Vec<String>,
) -> bool {
    let status = Command::new("/bin/kill")
        .arg(format!("-{signal}"))
        .arg(format!("-{process_id}"))
        .status();
    match status {
        Ok(status) if status.success() => true,
        Ok(status) => {
            warnings.push(format!(
                "manual_relay_process_group_signal_{signal}_failed:{status}"
            ));
            false
        }
        Err(error) => {
            warnings.push(format!(
                "manual_relay_process_group_signal_{signal}_failed:{error}"
            ));
            false
        }
    }
}

/// Confirm the group created with `process_group(0)` is gone before any
/// trusted lifecycle path drops its child handle, memory capture, durable
/// registration, or safe-only marker.  It uses only the child-owned PGID;
/// failure is retained for a later trusted retry rather than being treated as
/// a normal leader exit.
#[cfg(unix)]
fn sweep_manual_relay_process_group(
    process_id: u32,
    warnings: &mut Vec<String>,
) -> Result<bool, String> {
    if !manual_relay_process_group_exists(process_id)? {
        return Ok(false);
    }
    let kill_signaled = signal_manual_relay_process_group(process_id, "KILL", warnings);
    let started = Instant::now();
    loop {
        if !manual_relay_process_group_exists(process_id)? {
            return Ok(kill_signaled);
        }
        if started.elapsed() >= Duration::from_millis(800) {
            return Err(format!(
                "manual_relay_process_group_still_alive_after_sweep:{process_id}"
            ));
        }
        thread::sleep(Duration::from_millis(25));
    }
}

#[cfg(unix)]
fn manual_relay_process_group_exists(process_id: u32) -> Result<bool, String> {
    let status = Command::new("/bin/kill")
        .arg("-0")
        .arg(format!("-{process_id}"))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|error| format!("manual_relay_process_group_probe_failed:{error}"))?;
    Ok(status.success())
}

#[cfg(not(unix))]
fn sweep_manual_relay_process_group(
    process_id: u32,
    warnings: &mut Vec<String>,
) -> Result<bool, String> {
    let _ = (process_id, warnings);
    Ok(false)
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ManualRelayProcessMode {
    PlaceholderSleep,
    MockCodexComplete(PathBuf),
    MockCodexSleep(PathBuf),
    MockCodexSleepSh(String),
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
    if let Some(body) = mock_behavior.strip_prefix("mock_codex_process_sleep_sh:") {
        return Some(ManualRelayProcessMode::MockCodexSleepSh(body.to_string()));
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
        ManualRelayProcessMode::MockCodexSleepSh(body) => {
            // 进程夹具确定性边界：载荷经 argv 喂给常驻温热的 /bin/sh，不新建脚本文件。
            // 新建脚本首次 exec 在本沙箱实测 155ms~3.2s，全量并行会撞穿夹具就绪预算。
            command_plan.program = "/bin/sh".to_string();
            command_plan.argv = vec!["-c".to_string(), body]
                .into_iter()
                .chain(command_plan.argv)
                .collect();
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
    supervisor_policy: Option<SupervisorRelayExecutionPolicy>,
    registration_workflow_state_path: Option<&Path>,
) -> Result<ManualRelayReceipt, String> {
    let supervisor_mode = supervisor_policy.is_some();
    if supervisor_mode {
        let registry = active_attempts()
            .lock()
            .map_err(|_| "manual_relay_registry_poisoned".to_string())?;
        if registry.contains_key(&attempt_id) {
            return Err("manual_relay_attempt_id_reused".to_string());
        }
        if registry
            .values()
            .any(|attempt| attempt.duplicate_scope == scope && attempt.status == "running")
        {
            return Err("manual_relay_duplicate_running_attempt".to_string());
        }
    } else {
        with_non_safe_only_active_attempt_insertion(&attempt_id, |registry| {
            if registry.contains_key(&attempt_id) {
                return Err("manual_relay_attempt_id_reused".to_string());
            }
            if registry
                .values()
                .any(|attempt| attempt.duplicate_scope == scope && attempt.status == "running")
            {
                return Err("manual_relay_duplicate_running_attempt".to_string());
            }
            Ok(())
        })?;
    }
    {
        let mut consumed = consumed_confirmations()
            .lock()
            .map_err(|_| "manual_relay_confirmation_registry_poisoned".to_string())?;
        reserve_confirmation_in_map(&mut consumed, confirmation_id)?;
    }
    let mut receipt_template = fixture_receipt(
        &attempt_id,
        confirmation_id,
        "running",
        envelope,
        process_config.command_plan.clone(),
        timestamp,
        false,
        dirty_before,
        None,
    );
    receipt_template.process_kind = process_config.process_kind.clone();
    // `stdin.write_all` is the only delivery acknowledgement available to
    // this host.  Keep the receipt false until the spawn helper returns a
    // fully initialized process, because a post-spawn stdin failure can leave
    // only a protected cleanup route and must never be reported as delivered.
    receipt_template.prompt_sent = false;
    receipt_template.real_codex_executed = process_config.real_codex_executed;
    receipt_template.readback_status = "not_attempted_running_process".to_string();
    receipt_template
        .warnings
        .push("process_spawned_with_thread_event_output_capture".to_string());
    if !receipt_template.real_codex_executed {
        receipt_template
            .warnings
            .push("mock_codex_fixture_no_real_codex".to_string());
    }
    receipt_template.warnings.sort();
    receipt_template.warnings.dedup();
    #[cfg(test)]
    install_supervisor_relay_active_slot_collision_for_test(&attempt_id, &receipt_template);
    if supervisor_mode {
        reserve_supervisor_active_attempt_slot(
            scope,
            &attempt_id,
            confirmation_id,
            &receipt_template,
            &process_config.completed_status,
        )?;
    }
    let mut spawned = match spawn_codex_like_process_capture_to_files(
        &process_config.command_plan,
        envelope,
        Some(&envelope.payload.effective_prompt),
        process_config
            .real_codex_executed
            .then_some(attempt_id.as_str()),
        supervisor_policy,
        registration_workflow_state_path,
    ) {
        Ok(spawned) => spawned,
        Err(SpawnCodexLikeProcessError::Failed(error)) => {
            if supervisor_mode {
                clear_reserved_supervisor_active_attempt_slot(&attempt_id, confirmation_id);
            }
            clear_consumed_confirmation_attempt(confirmation_id, &attempt_id);
            return Err(error);
        }
        Err(SpawnCodexLikeProcessError::SupervisorCleanupPending(mut spawned)) => {
            let mut retained_receipt = receipt_template;
            retained_receipt.process_id = Some(spawned.child.id());
            let pending_receipt = supervisor_cleanup_pending_receipt(retained_receipt.clone());
            let active = ActiveManualRelayAttempt {
                duplicate_scope: scope.to_string(),
                status: "running".to_string(),
                receipt: retained_receipt,
                child: Some(spawned.child),
                completed_status: process_config.completed_status,
                output_paths: spawned.output_paths.take(),
                supervisor_capture: spawned.supervisor_capture.take(),
                process_registration: spawned.process_registration.take(),
            };
            return match install_supervisor_active_attempt_into_reserved_slot(
                &attempt_id,
                confirmation_id,
                active,
            ) {
                Ok(()) => Ok(pending_receipt),
                Err(active) => {
                    let (recovery_attempt_id, retained_receipt) =
                        move_supervisor_active_attempt_to_collision_recovery(
                            &attempt_id,
                            confirmation_id,
                            active,
                        );
                    match abort_safe_only_manual_relay_attempt(
                        ManualRelayStopInput {
                            relay_attempt_id: recovery_attempt_id,
                            requested_by: "supervisor_active_slot_collision_cleanup".to_string(),
                        },
                        timestamp,
                    ) {
                        Ok(()) => Err("manual_relay_attempt_id_reused".to_string()),
                        Err(_) => Ok(supervisor_cleanup_pending_receipt(retained_receipt)),
                    }
                }
            };
        }
    };
    // The successful helper return is after its single prompt write.  All
    // earlier post-spawn failure paths keep `prompt_sent=false`.
    receipt_template.prompt_sent = true;
    let process_id = Some(spawned.child.id());
    let mut receipt = receipt_template;
    receipt.process_id = process_id;
    let supervisor_pending_receipt = supervisor_cleanup_pending_receipt(receipt.clone());
    let mut active = Some(ActiveManualRelayAttempt {
        duplicate_scope: scope.to_string(),
        status: "running".to_string(),
        receipt: receipt.clone(),
        child: Some(spawned.child),
        completed_status: process_config.completed_status,
        output_paths: spawned.output_paths.take(),
        supervisor_capture: spawned.supervisor_capture.take(),
        process_registration: spawned.process_registration.take(),
    });
    #[cfg(test)]
    install_supervisor_relay_final_active_slot_collision_for_test(&attempt_id, &receipt);
    let insert_result = if supervisor_mode {
        let pending_active = active
            .take()
            .expect("active relay attempt is present until registry insertion");
        match install_supervisor_active_attempt_into_reserved_slot(
            &attempt_id,
            confirmation_id,
            pending_active,
        ) {
            Ok(()) => Ok(()),
            Err(pending_active) => {
                active = Some(pending_active);
                Err("manual_relay_attempt_id_reused".to_string())
            }
        }
    } else {
        with_non_safe_only_active_attempt_insertion(&attempt_id, |registry| {
            if registry
                .values()
                .any(|attempt| attempt.duplicate_scope == scope && attempt.status == "running")
            {
                return Err("manual_relay_duplicate_running_attempt".to_string());
            }
            if registry.contains_key(&attempt_id) {
                return Err("manual_relay_attempt_id_reused".to_string());
            }
            registry.insert(
                attempt_id.clone(),
                active
                    .take()
                    .expect("active relay attempt is present until registry insertion"),
            );
            Ok(())
        })
    };
    if let Err(error) = insert_result {
        let cleanup_settled = if let Some(active) = active.take() {
            if supervisor_mode {
                let (recovery_attempt_id, retained_receipt) =
                    move_supervisor_active_attempt_to_collision_recovery(
                        &attempt_id,
                        confirmation_id,
                        active,
                    );
                return match abort_safe_only_manual_relay_attempt(
                    ManualRelayStopInput {
                        relay_attempt_id: recovery_attempt_id,
                        requested_by: "supervisor_active_slot_collision_cleanup".to_string(),
                    },
                    timestamp,
                ) {
                    Ok(()) => {
                        clear_consumed_confirmation_attempt(confirmation_id, &attempt_id);
                        Err(error)
                    }
                    Err(_) => Ok(supervisor_cleanup_pending_receipt(retained_receipt)),
                };
            }
            cleanup_active_manual_relay_attempt(active, &attempt_id)
        } else {
            true
        };
        if cleanup_settled {
            clear_consumed_confirmation_attempt(confirmation_id, &attempt_id);
            return Err(error);
        }
        return if supervisor_mode {
            Ok(supervisor_pending_receipt)
        } else {
            Err(error)
        };
    }
    if let Err(error) = set_consumed_confirmation_attempt(confirmation_id, &attempt_id) {
        let active = if supervisor_mode {
            take_supervisor_active_attempt_from_reserved_slot(&attempt_id, confirmation_id)
        } else {
            active_attempts()
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .remove(&attempt_id)
        };
        let cleanup_settled = match active {
            Some(active) => cleanup_active_manual_relay_attempt(active, &attempt_id),
            // A supervisor marker is the authoritative proof that this
            // attempt still needs host-owned cleanup.  A missing active
            // record while that marker remains must therefore fail closed;
            // do not treat a poisoned/missing lookup as completed cleanup.
            None if supervisor_mode => !safe_only_attempt_is_reserved(&attempt_id),
            None => true,
        };
        if cleanup_settled {
            clear_consumed_confirmation_attempt(confirmation_id, &attempt_id);
            return Err(error);
        }
        return if supervisor_mode {
            Ok(supervisor_pending_receipt)
        } else {
            Err(error)
        };
    }
    Ok(receipt)
}

/// A supervisor start can fail after a child has been created but before the
/// normal transport/command records are installed.  Keep that failure
/// observable only through the existing safe receipt path so the host can
/// register a trusted retry; generic raw endpoints stay closed by the marker.
fn supervisor_cleanup_pending_receipt(mut receipt: ManualRelayReceipt) -> ManualRelayReceipt {
    receipt.status = "supervisor_relay_cleanup_pending".to_string();
    receipt
        .warnings
        .push("supervisor_relay_cleanup_pending".to_string());
    receipt.warnings.sort();
    receipt.warnings.dedup();
    receipt
}

fn run_codex_like_process_to_completion(
    attempt_id: &str,
    confirmation_id: &str,
    envelope: &ManualRelayEnvelope,
    process_config: ManualRelayProcessConfig,
    timestamp: &str,
    dirty_before: bool,
) -> Result<ManualRelayReceipt, String> {
    let process_output = run_codex_like_process_capture(
        &process_config.command_plan,
        envelope,
        Some(&envelope.payload.effective_prompt),
    )?;
    let process_id = Some(process_output.process_id);
    let exit_status = process_output.exit_status;
    let (readback_status, last_message_hash, last_message_size_bytes) =
        read_last_message_summary(&process_config.command_plan.last_message_path);
    let thread_event_report =
        parse_thread_event_report(&process_output.stdout, &process_output.stderr);
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
    apply_thread_event_report_to_receipt(&mut receipt, thread_event_report);
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

fn finalize_running_codex_like_attempt(
    mut receipt: ManualRelayReceipt,
    timestamp: &str,
    exit_code: Option<i32>,
    timed_out: bool,
    killed: bool,
    completed_status: &str,
    output_paths: Option<&ManualRelayProcessOutputPaths>,
    supervisor_capture: Option<&mut SupervisorRelayMemoryCapture>,
) -> ManualRelayReceipt {
    let process_success = exit_code == Some(0);
    let supervisor_mode = supervisor_capture.is_some();
    let (thread_event_report, mut output_warnings, capture_failed) =
        if let Some(capture) = supervisor_capture {
            let snapshot = capture.finish_snapshot_and_clear();
            let mut warnings = vec!["supervisor_relay_memory_capture_only".to_string()];
            if snapshot.stderr_seen {
                warnings.push("supervisor_relay_stderr_suppressed".to_string());
            }
            (snapshot.report, warnings, snapshot.failed_closed)
        } else {
            let (stdout, stderr, warnings) = read_process_output_paths(output_paths);
            (parse_thread_event_report(&stdout, &stderr), warnings, false)
        };
    let (readback_status, last_message_hash, last_message_size_bytes) = if supervisor_mode {
        (
            "supervisor_relay_readback_not_captured".to_string(),
            None,
            None,
        )
    } else {
        read_last_message_summary(&receipt.command_plan.last_message_path)
    };
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
    apply_thread_event_report_to_receipt(&mut receipt, thread_event_report);
    receipt.status = if timed_out {
        "timed_out".to_string()
    } else if supervisor_mode && (capture_failed || receipt.thread_event_summary.turn_failed) {
        "failed_process".to_string()
    } else if process_success {
        completed_status.to_string()
    } else {
        "failed_process".to_string()
    };
    if !supervisor_mode {
        receipt
            .warnings
            .push("readback_last_message_only_no_full_transcript".to_string());
    }
    if capture_failed {
        receipt
            .warnings
            .push("supervisor_relay_capture_failed_closed".to_string());
    }
    receipt.warnings.append(&mut output_warnings);
    receipt.warnings.sort();
    receipt.warnings.dedup();
    receipt
}

fn refresh_running_receipt_from_output(
    receipt: &mut ManualRelayReceipt,
    output_paths: Option<&ManualRelayProcessOutputPaths>,
    supervisor_capture: Option<&SupervisorRelayMemoryCapture>,
) {
    if let Some(capture) = supervisor_capture {
        let snapshot = capture.running_snapshot();
        let mut report = snapshot.report;
        if snapshot.failed_closed {
            report.summary.turn_failed = true;
        }
        if report.summary.json_line_count == 0 && !snapshot.stderr_seen && !snapshot.failed_closed {
            return;
        }
        apply_thread_event_report_to_receipt(receipt, report);
        receipt
            .warnings
            .push("supervisor_relay_memory_capture_only".to_string());
        if snapshot.stderr_seen {
            receipt
                .warnings
                .push("supervisor_relay_stderr_suppressed".to_string());
        }
        if snapshot.failed_closed {
            receipt
                .warnings
                .push("supervisor_relay_capture_failed_closed".to_string());
        }
        receipt.warnings.sort();
        receipt.warnings.dedup();
        return;
    }
    let (stdout, stderr, mut output_warnings) = read_process_output_paths(output_paths);
    if stdout.is_empty() && stderr.is_empty() {
        return;
    }
    let thread_event_report = parse_thread_event_report(&stdout, &stderr);
    apply_thread_event_report_to_receipt(receipt, thread_event_report);
    receipt.warnings.append(&mut output_warnings);
    receipt.warnings.sort();
    receipt.warnings.dedup();
}

fn apply_thread_event_report_to_receipt(
    receipt: &mut ManualRelayReceipt,
    report: ManualRelayThreadEventReport,
) {
    receipt.live_events = report.live_events;
    apply_thread_event_summary_to_receipt(receipt, report.summary);
}

fn apply_thread_event_summary_to_receipt(
    receipt: &mut ManualRelayReceipt,
    thread_event_summary: ManualRelayThreadEventSummary,
) {
    receipt.assistant_message_text = thread_event_summary.assistant_message_text.clone();
    receipt.thread_event_summary = thread_event_summary;
    if receipt.thread_event_summary.turn_completed && receipt.assistant_message_text.is_some() {
        receipt.readback_status = "thread_event_agent_message_available".to_string();
    } else if receipt.thread_event_summary.malformed_json_line_count > 0 {
        receipt
            .warnings
            .push("thread_event_jsonl_malformed_lines_present".to_string());
    } else if receipt
        .thread_event_summary
        .event_types
        .iter()
        .any(|event_type| event_type == "turn.completed")
    {
        receipt
            .warnings
            .push("thread_event_agent_message_unavailable".to_string());
    }
    if receipt.thread_event_summary.stderr_summary.is_some() {
        receipt
            .warnings
            .push("thread_event_stderr_summary_available".to_string());
    }
}

fn read_process_output_paths(
    output_paths: Option<&ManualRelayProcessOutputPaths>,
) -> (Vec<u8>, Vec<u8>, Vec<String>) {
    let Some(paths) = output_paths else {
        return (Vec::new(), Vec::new(), Vec::new());
    };
    let mut warnings = Vec::new();
    let stdout = match fs::read(&paths.stdout_path) {
        Ok(bytes) => bytes,
        Err(error) => {
            warnings.push(format!("thread_event_stdout_read_failed:{error}"));
            Vec::new()
        }
    };
    let stderr = match fs::read(&paths.stderr_path) {
        Ok(bytes) => bytes,
        Err(error) => {
            warnings.push(format!("thread_event_stderr_read_failed:{error}"));
            Vec::new()
        }
    };
    (stdout, stderr, warnings)
}

struct ManualRelayProcessOutput {
    process_id: u32,
    exit_status: std::process::ExitStatus,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

fn run_codex_like_process_capture(
    command_plan: &ManualRelayCommandPlan,
    envelope: &ManualRelayEnvelope,
    stdin_prompt: Option<&str>,
) -> Result<ManualRelayProcessOutput, String> {
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
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    configure_manual_relay_process_group(&mut command);
    let mut child = command
        .spawn()
        .map_err(|error| format!("manual_relay_process_spawn_failed:{error}"))?;
    let process_id = child.id();
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
    let output = child
        .wait_with_output()
        .map_err(|error| format!("manual_relay_process_wait_failed:{error}"))?;
    Ok(ManualRelayProcessOutput {
        process_id,
        exit_status: output.status,
        stdout: output.stdout,
        stderr: output.stderr,
    })
}

fn parse_thread_event_report(stdout: &[u8], stderr: &[u8]) -> ManualRelayThreadEventReport {
    let mut summary = empty_thread_event_summary();
    let mut live_events = Vec::new();
    let stdout_text = String::from_utf8_lossy(stdout);
    for line in stdout_text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        match serde_json::from_str::<Value>(trimmed) {
            Ok(value) => {
                summary.json_line_count += 1;
                apply_thread_event_value(&mut summary, &value);
                if let Some(event) =
                    live_event_from_thread_event_value(summary.json_line_count, &value)
                {
                    live_events.push(event);
                }
            }
            Err(_) => {
                summary.malformed_json_line_count += 1;
            }
        }
    }
    summary.stderr_summary = bounded_stderr_summary(stderr);
    ManualRelayThreadEventReport {
        summary,
        live_events,
    }
}

fn apply_thread_event_value(summary: &mut ManualRelayThreadEventSummary, value: &Value) {
    let Some(event_type) = value.get("type").and_then(Value::as_str) else {
        summary.malformed_json_line_count += 1;
        return;
    };
    summary.event_types.push(event_type.to_string());
    match event_type {
        "thread.started" => {
            summary.thread_id = value
                .get("thread_id")
                .and_then(Value::as_str)
                .map(str::to_string);
        }
        "item.completed" => {
            if let Some(item) = value.get("item").and_then(Value::as_object) {
                if item.get("type").and_then(Value::as_str) == Some("agent_message") {
                    summary.assistant_item_id =
                        item.get("id").and_then(Value::as_str).map(str::to_string);
                    summary.assistant_message_text =
                        item.get("text").and_then(Value::as_str).map(str::to_string);
                }
            }
        }
        "turn.completed" => {
            summary.turn_completed = true;
            if let Some(usage) = value.get("usage").and_then(Value::as_object) {
                for (key, item) in usage {
                    if let Some(count) = item.as_i64() {
                        summary.usage.insert(key.clone(), count);
                    }
                }
            }
        }
        "turn.failed" | "error" => {
            summary.turn_failed = true;
        }
        _ => {}
    }
}

fn empty_thread_event_summary() -> ManualRelayThreadEventSummary {
    ManualRelayThreadEventSummary {
        thread_id: None,
        assistant_item_id: None,
        assistant_message_text: None,
        turn_completed: false,
        turn_failed: false,
        usage: BTreeMap::new(),
        event_types: Vec::new(),
        json_line_count: 0,
        malformed_json_line_count: 0,
        stderr_summary: None,
    }
}

fn live_event_from_thread_event_value(
    sequence: i64,
    value: &Value,
) -> Option<ManualRelayLiveEvent> {
    let event_type = value.get("type").and_then(Value::as_str)?;
    let thread_id = value
        .get("thread_id")
        .and_then(Value::as_str)
        .map(str::to_string);
    match event_type {
        "thread.started" => Some(ManualRelayLiveEvent {
            sequence,
            event_type: event_type.to_string(),
            thread_id,
            item_id: None,
            item_type: None,
            title: "对话已创建".to_string(),
            text: value
                .get("thread_id")
                .and_then(Value::as_str)
                .map(str::to_string),
            delta: None,
            tool_name: None,
            arguments_preview: None,
            output_preview: None,
            stdout: None,
            stderr: None,
            exit_code: None,
            status: "started".to_string(),
        }),
        "turn.started" => Some(ManualRelayLiveEvent {
            sequence,
            event_type: event_type.to_string(),
            thread_id,
            item_id: None,
            item_type: None,
            title: "Codex 开始处理".to_string(),
            text: None,
            delta: None,
            tool_name: None,
            arguments_preview: None,
            output_preview: None,
            stdout: None,
            stderr: None,
            exit_code: None,
            status: "running".to_string(),
        }),
        "turn.completed" => Some(ManualRelayLiveEvent {
            sequence,
            event_type: event_type.to_string(),
            thread_id,
            item_id: None,
            item_type: None,
            title: "Codex 完成".to_string(),
            text: value.get("usage").map(bounded_value_preview),
            delta: None,
            tool_name: None,
            arguments_preview: None,
            output_preview: value.get("usage").map(bounded_value_preview),
            stdout: None,
            stderr: None,
            exit_code: None,
            status: "completed".to_string(),
        }),
        "turn.failed" | "error" => Some(ManualRelayLiveEvent {
            sequence,
            event_type: event_type.to_string(),
            thread_id,
            item_id: None,
            item_type: None,
            title: "Codex 失败".to_string(),
            text: thread_event_error_text(value),
            delta: None,
            tool_name: None,
            arguments_preview: None,
            output_preview: Some(bounded_value_preview(value)),
            stdout: None,
            stderr: None,
            exit_code: None,
            status: "failed".to_string(),
        }),
        "item.started" | "item.updated" | "item.completed" => {
            live_item_event_from_thread_event_value(sequence, event_type, thread_id, value)
        }
        _ => Some(ManualRelayLiveEvent {
            sequence,
            event_type: event_type.to_string(),
            thread_id,
            item_id: None,
            item_type: None,
            title: event_type.to_string(),
            text: None,
            delta: None,
            tool_name: None,
            arguments_preview: Some(bounded_value_preview(value)),
            output_preview: None,
            stdout: None,
            stderr: None,
            exit_code: None,
            status: "running".to_string(),
        }),
    }
}

fn live_item_event_from_thread_event_value(
    sequence: i64,
    event_type: &str,
    thread_id: Option<String>,
    value: &Value,
) -> Option<ManualRelayLiveEvent> {
    let item = value.get("item").and_then(Value::as_object)?;
    let item_id = item.get("id").and_then(Value::as_str).map(str::to_string);
    let item_type = item.get("type").and_then(Value::as_str).map(str::to_string);
    let item_type_str = item_type.as_deref().unwrap_or("item");
    let status = if event_type == "item.completed" {
        "completed"
    } else {
        "running"
    };
    let text = first_string_field(item, &["text", "message", "summary", "content"])
        .or_else(|| first_string_field(value.as_object()?, &["text", "message"]));
    let delta = thread_event_delta_text(value);
    let tool_name = first_string_field(item, &["name", "tool_name"]).or_else(|| {
        if item_type_str == "local_shell_call" || item_type_str == "function_call" {
            Some(item_type_str.to_string())
        } else {
            None
        }
    });
    let arguments_preview = item
        .get("arguments")
        .or_else(|| item.get("args"))
        .or_else(|| item.get("input"))
        .map(bounded_value_preview);
    let output_preview = item.get("output").map(bounded_value_preview);
    let stdout = item
        .get("stdout")
        .and_then(Value::as_str)
        .map(bounded_text_preview);
    let stderr = item
        .get("stderr")
        .and_then(Value::as_str)
        .map(bounded_text_preview);
    let exit_code = item
        .get("exit_code")
        .and_then(Value::as_i64)
        .and_then(|value| i32::try_from(value).ok());
    let title = live_item_title(event_type, item_type_str);
    Some(ManualRelayLiveEvent {
        sequence,
        event_type: event_type.to_string(),
        thread_id,
        item_id,
        item_type,
        title,
        text: text.as_deref().map(bounded_text_preview),
        delta: delta.as_deref().map(bounded_text_preview),
        tool_name,
        arguments_preview,
        output_preview,
        stdout,
        stderr,
        exit_code,
        status: status.to_string(),
    })
}

fn live_item_title(event_type: &str, item_type: &str) -> String {
    match item_type {
        "agent_message" => {
            if event_type == "item.completed" {
                "Codex 回复完成"
            } else {
                "Codex 正在回复"
            }
        }
        "reasoning" => {
            if event_type == "item.completed" {
                "思考完成"
            } else {
                "思考中"
            }
        }
        "local_shell_call" => {
            if event_type == "item.completed" {
                "命令完成"
            } else {
                "正在运行命令"
            }
        }
        "function_call" => {
            if event_type == "item.completed" {
                "工具完成"
            } else {
                "正在调用工具"
            }
        }
        "function_call_output" => "工具输出",
        _ => item_type,
    }
    .to_string()
}

fn first_string_field(
    object: &serde_json::Map<String, Value>,
    field_names: &[&str],
) -> Option<String> {
    for field_name in field_names {
        if let Some(value) = object.get(*field_name).and_then(Value::as_str) {
            if !value.trim().is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

fn thread_event_delta_text(value: &Value) -> Option<String> {
    let delta = value.get("delta")?;
    if let Some(text) = delta.as_str() {
        return Some(text.to_string());
    }
    if let Some(object) = delta.as_object() {
        return first_string_field(object, &["text", "message", "content"]);
    }
    None
}

fn thread_event_error_text(value: &Value) -> Option<String> {
    first_string_field(
        value.as_object()?,
        &["message", "error", "details", "reason"],
    )
}

fn bounded_value_preview(value: &Value) -> String {
    let text = match value {
        Value::String(text) => text.clone(),
        _ => serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string()),
    };
    bounded_text_preview(&text)
}

fn bounded_text_preview(text: &str) -> String {
    const LIMIT: usize = 4096;
    if text.chars().count() <= LIMIT {
        return text.to_string();
    }
    let mut bounded = text.chars().take(LIMIT).collect::<String>();
    bounded.push_str("\n...[truncated]");
    bounded
}

fn bounded_stderr_summary(stderr: &[u8]) -> Option<String> {
    let text = String::from_utf8_lossy(stderr).trim().to_string();
    if text.is_empty() {
        None
    } else {
        Some(text.chars().take(2048).collect())
    }
}

struct SpawnedManualRelayProcess {
    child: Child,
    output_paths: Option<ManualRelayProcessOutputPaths>,
    supervisor_capture: Option<SupervisorRelayMemoryCapture>,
    process_registration: Option<crate::exec_process_registry::DurableProcessRegistration>,
}

/// A supervisor-only post-spawn failure keeps ownership of every recovery
/// handle until the caller has installed an active safe-only attempt.  A
/// string error alone would drop the child before trusted cleanup can retry.
enum SpawnCodexLikeProcessError {
    Failed(String),
    SupervisorCleanupPending(SpawnedManualRelayProcess),
}

impl std::fmt::Debug for SpawnCodexLikeProcessError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Failed(error) => formatter.debug_tuple("Failed").field(error).finish(),
            Self::SupervisorCleanupPending(_) => {
                formatter.write_str("SupervisorCleanupPending(<retained>)")
            }
        }
    }
}

fn spawn_codex_like_process_capture_to_files(
    command_plan: &ManualRelayCommandPlan,
    envelope: &ManualRelayEnvelope,
    stdin_prompt: Option<&str>,
    registration_run_id: Option<&str>,
    supervisor_policy: Option<SupervisorRelayExecutionPolicy>,
    registration_workflow_state_path: Option<&Path>,
) -> Result<SpawnedManualRelayProcess, SpawnCodexLikeProcessError> {
    let supervisor_mode = supervisor_policy.is_some();
    let last_message_path = PathBuf::from(&command_plan.last_message_path);
    let output_paths = if supervisor_mode {
        None
    } else {
        let Some(output_dir) = last_message_path.parent() else {
            return Err(SpawnCodexLikeProcessError::Failed(
                "manual_relay_last_message_parent_missing".to_string(),
            ));
        };
        fs::create_dir_all(output_dir).map_err(|error| {
            SpawnCodexLikeProcessError::Failed(format!(
                "manual_relay_last_message_dir_create_failed:{error}"
            ))
        })?;
        let output_paths = ManualRelayProcessOutputPaths {
            stdout_path: output_dir.join("thread-events.stdout.jsonl"),
            stderr_path: output_dir.join("thread-events.stderr.txt"),
        };
        let stdout_file = fs::File::create(&output_paths.stdout_path).map_err(|error| {
            SpawnCodexLikeProcessError::Failed(format!(
                "manual_relay_stdout_capture_create_failed:{error}"
            ))
        })?;
        let stderr_file = fs::File::create(&output_paths.stderr_path).map_err(|error| {
            SpawnCodexLikeProcessError::Failed(format!(
                "manual_relay_stderr_capture_create_failed:{error}"
            ))
        })?;
        Some((output_paths, stdout_file, stderr_file))
    };
    let mut command = Command::new(&command_plan.program);
    for arg in &command_plan.argv {
        if arg == "<workbench-managed-last-message>" {
            command.arg(&last_message_path);
        } else {
            command.arg(arg);
        }
    }
    command.current_dir(&envelope.target_binding.target_cwd_canonical);
    command.stdin(if stdin_prompt.is_some() {
        Stdio::piped()
    } else {
        Stdio::null()
    });
    if let Some((_, stdout_file, stderr_file)) = output_paths.as_ref() {
        command
            .stdout(Stdio::from(stdout_file.try_clone().map_err(|error| {
                SpawnCodexLikeProcessError::Failed(format!(
                    "manual_relay_stdout_capture_clone_failed:{error}"
                ))
            })?))
            .stderr(Stdio::from(stderr_file.try_clone().map_err(|error| {
                SpawnCodexLikeProcessError::Failed(format!(
                    "manual_relay_stderr_capture_clone_failed:{error}"
                ))
            })?));
    } else {
        command.stdout(Stdio::piped()).stderr(Stdio::piped());
    }
    configure_manual_relay_process_group(&mut command);
    let mut child = command.spawn().map_err(|error| {
        SpawnCodexLikeProcessError::Failed(format!("manual_relay_process_spawn_failed:{error}"))
    })?;
    let mut supervisor_capture = match supervisor_policy {
        Some(policy) => match SupervisorRelayMemoryCapture::start(
            child.stdout.take(),
            child.stderr.take(),
            policy,
        ) {
            Ok(capture) => Some(capture),
            Err(_) => {
                let cleanup = cleanup_spawned_manual_relay_parts(child, None, None);
                return Err(spawn_cleanup_error(
                    supervisor_mode,
                    cleanup,
                    "supervisor_relay_capture_start_failed".to_string(),
                ));
            }
        },
        None => None,
    };
    let mut process_registration = if let Some(run_id) = registration_run_id {
        let workflow_state_path = registration_workflow_state_path
            .map(Path::to_path_buf)
            .unwrap_or_else(crate::default_workflow_state_path);
        #[cfg(test)]
        let registration = if supervisor_mode
            && SUPERVISOR_RELAY_USE_TEMPORARY_DURABLE_REGISTRATION.with(std::cell::Cell::get)
        {
            crate::exec_process_registry::register_temporary_durable_process_for_cleanup_test(
                &workflow_state_path,
                run_id,
                child.id(),
            )
        } else if supervisor_mode {
            crate::exec_process_registry::register_host_owned_supervisor_conversation_process_group(
                &workflow_state_path,
                run_id,
                child.id(),
            )
        } else {
            crate::exec_process_registry::register_manual_relay_process_group(
                &workflow_state_path,
                run_id,
                child.id(),
            )
        };
        #[cfg(not(test))]
        let registration = if supervisor_mode {
            crate::exec_process_registry::register_host_owned_supervisor_conversation_process_group(
                &workflow_state_path,
                run_id,
                child.id(),
            )
        } else {
            crate::exec_process_registry::register_manual_relay_process_group(
                &workflow_state_path,
                run_id,
                child.id(),
            )
        };
        match registration {
            Ok(registration) => Some(registration),
            Err(error) => {
                let cleanup =
                    cleanup_spawned_manual_relay_parts(child, None, supervisor_capture.take());
                let failure = if supervisor_mode {
                    "supervisor_relay_process_registration_failed".to_string()
                } else {
                    format!("manual_relay_process_registration_failed:{error}")
                };
                return Err(spawn_cleanup_error(supervisor_mode, cleanup, failure));
            }
        }
    } else {
        None
    };
    if let Some(prompt) = stdin_prompt {
        let Some(mut stdin) = child.stdin.take() else {
            let cleanup = cleanup_spawned_manual_relay_parts(
                child,
                process_registration.take(),
                supervisor_capture.take(),
            );
            let failure = if supervisor_mode {
                "supervisor_relay_process_stdin_unavailable".to_string()
            } else {
                "manual_relay_process_stdin_unavailable".to_string()
            };
            return Err(spawn_cleanup_error(supervisor_mode, cleanup, failure));
        };
        #[cfg(test)]
        if supervisor_mode
            && supervisor_relay_spawn_test_failure_active(
                SupervisorRelaySpawnTestFailurePoint::StdinWrite,
            )
        {
            drop(stdin);
            let cleanup = cleanup_spawned_manual_relay_parts(
                child,
                process_registration.take(),
                supervisor_capture.take(),
            );
            return Err(spawn_cleanup_error(
                supervisor_mode,
                cleanup,
                "supervisor_relay_process_stdin_write_failed".to_string(),
            ));
        }
        if let Err(error) = stdin.write_all(prompt.as_bytes()) {
            let cleanup = cleanup_spawned_manual_relay_parts(
                child,
                process_registration.take(),
                supervisor_capture.take(),
            );
            let failure = if supervisor_mode {
                "supervisor_relay_process_stdin_write_failed".to_string()
            } else {
                format!("manual_relay_process_stdin_write_failed:{error}")
            };
            return Err(spawn_cleanup_error(supervisor_mode, cleanup, failure));
        }
    }
    Ok(SpawnedManualRelayProcess {
        child,
        output_paths: output_paths.map(|(paths, _, _)| paths),
        supervisor_capture,
        process_registration,
    })
}

fn cleanup_spawned_manual_relay_parts(
    mut child: Child,
    mut process_registration: Option<crate::exec_process_registry::DurableProcessRegistration>,
    mut supervisor_capture: Option<SupervisorRelayMemoryCapture>,
) -> Result<(), SpawnedManualRelayProcess> {
    let child_terminated = cleanup_manual_relay_child_process(&mut child).is_ok();
    let mut cleanup_failed = !child_terminated;
    if child_terminated {
        if let Some(registration) = process_registration.as_mut() {
            if registration.unregister_preserving_on_error().is_err() {
                cleanup_failed = true;
            } else {
                process_registration.take();
            }
        }
    }
    if let Some(capture) = supervisor_capture.as_mut() {
        if child_terminated {
            let _ = capture.finish_snapshot_and_clear();
        } else {
            capture.clear();
        }
    }
    if cleanup_failed {
        Err(SpawnedManualRelayProcess {
            child,
            output_paths: None,
            supervisor_capture,
            process_registration,
        })
    } else {
        Ok(())
    }
}

fn spawn_cleanup_error(
    supervisor_mode: bool,
    cleanup: Result<(), SpawnedManualRelayProcess>,
    failure: String,
) -> SpawnCodexLikeProcessError {
    match cleanup {
        Ok(()) => SpawnCodexLikeProcessError::Failed(failure),
        Err(retained) if supervisor_mode => {
            SpawnCodexLikeProcessError::SupervisorCleanupPending(retained)
        }
        Err(_) => SpawnCodexLikeProcessError::Failed(failure),
    }
}

fn cleanup_active_manual_relay_attempt(active: ActiveManualRelayAttempt, attempt_id: &str) -> bool {
    cleanup_removed_manual_relay_attempt(active, attempt_id, false)
        .errors
        .is_empty()
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
    completed_status: String,
    output_paths: Option<ManualRelayProcessOutputPaths>,
    supervisor_capture: Option<SupervisorRelayMemoryCapture>,
    process_registration: Option<crate::exec_process_registry::DurableProcessRegistration>,
}

#[derive(Clone, Debug)]
struct ManualRelayProcessOutputPaths {
    stdout_path: PathBuf,
    stderr_path: PathBuf,
}

struct SupervisorRelayMemoryCapture {
    state: Arc<Mutex<SupervisorRelayMemoryCaptureState>>,
    workers: Vec<thread::JoinHandle<()>>,
}

struct SupervisorRelayMemoryCaptureState {
    total_bytes: usize,
    stdout_frame: Vec<u8>,
    discard_stdout_until_newline: bool,
    report: ManualRelayThreadEventReport,
    stderr_seen: bool,
    overflowed: bool,
    parse_failed: bool,
    io_failed: bool,
    cleared: bool,
    redaction_markers: Vec<String>,
}

struct SupervisorRelayMemoryCaptureSnapshot {
    report: ManualRelayThreadEventReport,
    stderr_seen: bool,
    failed_closed: bool,
}

#[derive(Clone, Copy)]
enum SupervisorCaptureStream {
    Stdout,
    Stderr,
}

impl SupervisorRelayMemoryCapture {
    fn start(
        stdout: Option<std::process::ChildStdout>,
        stderr: Option<std::process::ChildStderr>,
        policy: SupervisorRelayExecutionPolicy,
    ) -> Result<Self, String> {
        let stdout = stdout.ok_or_else(|| "supervisor_relay_stdout_unavailable".to_string())?;
        let stderr = stderr.ok_or_else(|| "supervisor_relay_stderr_unavailable".to_string())?;
        let state = Arc::new(Mutex::new(SupervisorRelayMemoryCaptureState {
            total_bytes: 0,
            stdout_frame: Vec::new(),
            discard_stdout_until_newline: false,
            report: ManualRelayThreadEventReport {
                summary: empty_thread_event_summary(),
                live_events: Vec::new(),
            },
            stderr_seen: false,
            overflowed: false,
            parse_failed: false,
            io_failed: false,
            cleared: false,
            redaction_markers: policy.redaction_markers,
        }));
        let stdout_state = Arc::clone(&state);
        let stderr_state = Arc::clone(&state);
        Ok(Self {
            state,
            workers: vec![
                thread::spawn(move || {
                    read_supervisor_capture_stream(
                        stdout,
                        SupervisorCaptureStream::Stdout,
                        stdout_state,
                    )
                }),
                thread::spawn(move || {
                    read_supervisor_capture_stream(
                        stderr,
                        SupervisorCaptureStream::Stderr,
                        stderr_state,
                    )
                }),
            ],
        })
    }

    fn running_snapshot(&self) -> SupervisorRelayMemoryCaptureSnapshot {
        supervisor_capture_snapshot(&self.state)
    }

    fn finish_snapshot_and_clear(&mut self) -> SupervisorRelayMemoryCaptureSnapshot {
        for worker in std::mem::take(&mut self.workers) {
            if worker.join().is_err() {
                if let Ok(mut state) = self.state.lock() {
                    state.io_failed = true;
                }
            }
        }
        let snapshot = supervisor_capture_snapshot(&self.state);
        self.clear();
        snapshot
    }

    fn clear(&mut self) {
        if let Ok(mut state) = self.state.lock() {
            state.cleared = true;
            state.total_bytes = 0;
            state.stdout_frame.clear();
            state.discard_stdout_until_newline = false;
            state.report = ManualRelayThreadEventReport {
                summary: empty_thread_event_summary(),
                live_events: Vec::new(),
            };
            state.stderr_seen = false;
            state.overflowed = false;
            state.parse_failed = false;
            state.io_failed = false;
            state.redaction_markers.clear();
        }
    }
}

impl Drop for SupervisorRelayMemoryCapture {
    fn drop(&mut self) {
        self.clear();
    }
}

fn read_supervisor_capture_stream<R: Read>(
    mut reader: R,
    stream: SupervisorCaptureStream,
    state: Arc<Mutex<SupervisorRelayMemoryCaptureState>>,
) {
    let mut buffer = [0_u8; 4096];
    loop {
        match reader.read(&mut buffer) {
            Ok(0) => return,
            Ok(count) => record_supervisor_capture_chunk(&state, stream, &buffer[..count]),
            Err(_) => {
                if let Ok(mut state) = state.lock() {
                    state.io_failed = true;
                }
                return;
            }
        }
    }
}

fn record_supervisor_capture_chunk(
    state: &Arc<Mutex<SupervisorRelayMemoryCaptureState>>,
    stream: SupervisorCaptureStream,
    bytes: &[u8],
) {
    let Ok(mut state) = state.lock() else {
        return;
    };
    if state.cleared {
        return;
    }
    state.total_bytes = state.total_bytes.saturating_add(bytes.len());
    if state.total_bytes > SUPERVISOR_CAPTURE_MAX_TOTAL_BYTES {
        state.overflowed = true;
        state.stdout_frame.clear();
        state.discard_stdout_until_newline = true;
        return;
    }
    if matches!(stream, SupervisorCaptureStream::Stderr) {
        state.stderr_seen |= !bytes.is_empty();
        return;
    }
    for byte in bytes {
        if state.discard_stdout_until_newline {
            if *byte == b'\n' {
                state.discard_stdout_until_newline = false;
            }
            continue;
        }
        if *byte == b'\n' {
            let frame = std::mem::take(&mut state.stdout_frame);
            record_supervisor_json_frame(&mut state, &frame);
            continue;
        }
        if state.stdout_frame.len() >= SUPERVISOR_CAPTURE_MAX_FRAME_BYTES {
            state.overflowed = true;
            state.stdout_frame.clear();
            state.discard_stdout_until_newline = true;
            continue;
        }
        state.stdout_frame.push(*byte);
    }
}

fn record_supervisor_json_frame(state: &mut SupervisorRelayMemoryCaptureState, frame: &[u8]) {
    if frame.iter().all(u8::is_ascii_whitespace) {
        return;
    }
    match serde_json::from_slice::<Value>(frame) {
        Ok(value) => {
            state.report.summary.json_line_count += 1;
            apply_supervisor_thread_event_value(state, &value);
        }
        Err(_) => {
            state.report.summary.malformed_json_line_count += 1;
            state.parse_failed = true;
        }
    }
}

fn apply_supervisor_thread_event_value(
    state: &mut SupervisorRelayMemoryCaptureState,
    value: &Value,
) {
    let Some(event_type) = value.get("type").and_then(Value::as_str) else {
        state.report.summary.malformed_json_line_count += 1;
        state.parse_failed = true;
        return;
    };
    match event_type {
        "thread.started" => {
            state
                .report
                .summary
                .event_types
                .push("thread.started".to_string());
            state.report.summary.thread_id = value
                .get("thread_id")
                .and_then(Value::as_str)
                .and_then(|value| supervisor_visible_text(state, value));
            push_supervisor_live_event(
                state,
                "thread.started",
                "对话已创建",
                state.report.summary.thread_id.clone(),
                None,
                None,
                "started",
            );
        }
        "turn.started" => {
            state
                .report
                .summary
                .event_types
                .push("turn.started".to_string());
            push_supervisor_live_event(
                state,
                "turn.started",
                "Codex 开始处理",
                state.report.summary.thread_id.clone(),
                None,
                None,
                "running",
            );
        }
        "item.completed" => {
            let Some(item) = value.get("item").and_then(Value::as_object) else {
                state.report.summary.malformed_json_line_count += 1;
                state.parse_failed = true;
                return;
            };
            if item.get("type").and_then(Value::as_str) != Some("agent_message") {
                return;
            }
            state
                .report
                .summary
                .event_types
                .push("item.completed".to_string());
            state.report.summary.assistant_item_id = item
                .get("id")
                .and_then(Value::as_str)
                .and_then(|value| supervisor_visible_text(state, value));
            state.report.summary.assistant_message_text = item
                .get("text")
                .and_then(Value::as_str)
                .and_then(|value| supervisor_visible_text(state, value));
            push_supervisor_live_event(
                state,
                "item.completed",
                "Codex 回复完成",
                state.report.summary.thread_id.clone(),
                state.report.summary.assistant_item_id.clone(),
                state.report.summary.assistant_message_text.clone(),
                "completed",
            );
        }
        "turn.completed" => {
            state
                .report
                .summary
                .event_types
                .push("turn.completed".to_string());
            state.report.summary.turn_completed = true;
            if let Some(usage) = value.get("usage").and_then(Value::as_object) {
                for (key, item) in usage {
                    if !supervisor_usage_key_allowed(key) {
                        // A supervisor child controls this JSON object. Never
                        // project arbitrary key text into a safe receipt: an
                        // unknown key could itself carry a relay secret.
                        state.parse_failed = true;
                        continue;
                    }
                    if let Some(count) = item.as_i64() {
                        state.report.summary.usage.insert(key.clone(), count);
                    } else {
                        state.parse_failed = true;
                    }
                }
            }
            push_supervisor_live_event(
                state,
                "turn.completed",
                "Codex 完成",
                state.report.summary.thread_id.clone(),
                None,
                None,
                "completed",
            );
        }
        "turn.failed" | "error" => {
            state
                .report
                .summary
                .event_types
                .push("supervisor_relay_error".to_string());
            state.report.summary.turn_failed = true;
            push_supervisor_live_event(
                state,
                "supervisor_relay_error",
                "Codex 失败",
                state.report.summary.thread_id.clone(),
                None,
                Some("supervisor_relay_child_failed".to_string()),
                "failed",
            );
        }
        _ => {}
    }
}

fn supervisor_usage_key_allowed(key: &str) -> bool {
    matches!(
        key,
        "input_tokens" | "cached_input_tokens" | "output_tokens" | "reasoning_output_tokens"
    )
}

fn supervisor_visible_text(
    state: &SupervisorRelayMemoryCaptureState,
    value: &str,
) -> Option<String> {
    if state
        .redaction_markers
        .iter()
        .any(|marker| !marker.is_empty() && value.contains(marker))
    {
        None
    } else {
        Some(bounded_text_preview(value))
    }
}

fn push_supervisor_live_event(
    state: &mut SupervisorRelayMemoryCaptureState,
    event_type: &str,
    title: &str,
    thread_id: Option<String>,
    item_id: Option<String>,
    text: Option<String>,
    status: &str,
) {
    if state.report.live_events.len() >= SUPERVISOR_CAPTURE_MAX_LIVE_EVENTS {
        state.overflowed = true;
        return;
    }
    let sequence = state.report.summary.json_line_count;
    state.report.live_events.push(ManualRelayLiveEvent {
        sequence,
        event_type: event_type.to_string(),
        thread_id,
        item_id,
        item_type: (event_type == "item.completed").then(|| "agent_message".to_string()),
        title: title.to_string(),
        text,
        delta: None,
        tool_name: None,
        arguments_preview: None,
        output_preview: None,
        stdout: None,
        stderr: None,
        exit_code: None,
        status: status.to_string(),
    });
}

fn supervisor_capture_snapshot(
    state: &Arc<Mutex<SupervisorRelayMemoryCaptureState>>,
) -> SupervisorRelayMemoryCaptureSnapshot {
    match state.lock() {
        Ok(state) => SupervisorRelayMemoryCaptureSnapshot {
            report: state.report.clone(),
            stderr_seen: state.stderr_seen,
            failed_closed: state.overflowed || state.parse_failed || state.io_failed,
        },
        Err(_) => SupervisorRelayMemoryCaptureSnapshot {
            report: ManualRelayThreadEventReport {
                summary: empty_thread_event_summary(),
                live_events: Vec::new(),
            },
            stderr_seen: false,
            failed_closed: true,
        },
    }
}

fn active_attempts() -> &'static Mutex<BTreeMap<String, ActiveManualRelayAttempt>> {
    static REGISTRY: OnceLock<Mutex<BTreeMap<String, ActiveManualRelayAttempt>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(BTreeMap::new()))
}

/// A supervisor attempt claims its active-map key before spawning.  The
/// safe-only marker already closes raw access at this point; the extra slot
/// makes a same-key collision fail before a child, durable registration, or
/// memory capture exists, rather than overwriting an unrelated active owner
/// after spawn.
fn reserve_supervisor_active_attempt_slot(
    scope: &str,
    attempt_id: &str,
    confirmation_id: &str,
    receipt: &ManualRelayReceipt,
    completed_status: &str,
) -> Result<(), String> {
    let _visibility_gate = raw_manual_relay_visibility_gate()
        .lock()
        .map_err(|_| "manual_relay_safe_only_registry_poisoned".to_string())?;
    let markers = safe_only_attempts()
        .lock()
        .map_err(|_| "manual_relay_safe_only_registry_poisoned".to_string())?;
    if !markers
        .get(attempt_id)
        .is_some_and(|owner| owner == confirmation_id)
    {
        return Err("manual_relay_safe_only_attempt_ownership_mismatch".to_string());
    }
    let mut registry = active_attempts()
        .lock()
        .map_err(|_| "manual_relay_registry_poisoned".to_string())?;
    if registry
        .values()
        .any(|attempt| attempt.duplicate_scope == scope && attempt.status == "running")
    {
        return Err("manual_relay_duplicate_running_attempt".to_string());
    }
    match registry.entry(attempt_id.to_string()) {
        Entry::Vacant(entry) => {
            entry.insert(ActiveManualRelayAttempt {
                duplicate_scope: scope.to_string(),
                status: "starting".to_string(),
                receipt: receipt.clone(),
                child: None,
                completed_status: completed_status.to_string(),
                output_paths: None,
                supervisor_capture: None,
                process_registration: None,
            });
            Ok(())
        }
        Entry::Occupied(_) => Err("manual_relay_attempt_id_reused".to_string()),
    }
}

fn clear_reserved_supervisor_active_attempt_slot(attempt_id: &str, confirmation_id: &str) {
    let Ok(mut registry) = active_attempts().lock() else {
        return;
    };
    let is_ours = registry.get(attempt_id).is_some_and(|attempt| {
        attempt.status == "starting"
            && attempt.receipt.confirmation_id == confirmation_id
            && attempt.child.is_none()
            && attempt.process_registration.is_none()
            && attempt.supervisor_capture.is_none()
    });
    if is_ours {
        registry.remove(attempt_id);
    }
}

fn install_supervisor_active_attempt_into_reserved_slot(
    attempt_id: &str,
    confirmation_id: &str,
    active: ActiveManualRelayAttempt,
) -> Result<(), ActiveManualRelayAttempt> {
    let Ok(mut registry) = active_attempts().lock() else {
        return Err(active);
    };
    let is_ours = registry.get(attempt_id).is_some_and(|reserved| {
        reserved.status == "starting"
            && reserved.receipt.confirmation_id == confirmation_id
            && reserved.child.is_none()
            && reserved.process_registration.is_none()
            && reserved.supervisor_capture.is_none()
    });
    if !is_ours {
        return Err(active);
    }
    registry.insert(attempt_id.to_string(), active);
    Ok(())
}

fn take_supervisor_active_attempt_from_reserved_slot(
    attempt_id: &str,
    confirmation_id: &str,
) -> Option<ActiveManualRelayAttempt> {
    let Ok(mut registry) = active_attempts().lock() else {
        return None;
    };
    let is_ours = registry
        .get(attempt_id)
        .is_some_and(|attempt| attempt.receipt.confirmation_id == confirmation_id);
    is_ours.then(|| registry.remove(attempt_id)).flatten()
}

fn consumed_confirmations() -> &'static Mutex<BTreeMap<String, String>> {
    static REGISTRY: OnceLock<Mutex<BTreeMap<String, String>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(BTreeMap::new()))
}

/// Kept separately from `active_attempts`: the marker must become visible
/// before `Command::spawn`, so a generic raw poll/stop can never observe a
/// host-owned supervisor attempt in an unprotected interval.
fn safe_only_attempts() -> &'static Mutex<BTreeMap<String, String>> {
    static REGISTRY: OnceLock<Mutex<BTreeMap<String, String>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(BTreeMap::new()))
}

/// Linearizes a raw poll/stop against the supervisor's pre-spawn marker.
/// It is deliberately separate from the marker map: the short map lock is
/// still used for every read/write, while this gate stays held only around a
/// raw operation's final active-attempt access.  Thus either the marker wins
/// before spawn, or the raw operation completes before a safe-only spawn may
/// reserve the same attempt id.
fn raw_manual_relay_visibility_gate() -> &'static Mutex<()> {
    static GATE: OnceLock<Mutex<()>> = OnceLock::new();
    GATE.get_or_init(|| Mutex::new(()))
}

/// Serializes host-owned supervisor spawn with application shutdown.  A
/// shutdown cannot observe a pre-spawn marker, miss its not-yet-active child,
/// and then allow that child to start after shutdown has completed.
fn supervisor_relay_shutdown_gate() -> &'static Mutex<bool> {
    static GATE: OnceLock<Mutex<bool>> = OnceLock::new();
    GATE.get_or_init(|| Mutex::new(false))
}

fn acquire_supervisor_relay_start_gate() -> Result<std::sync::MutexGuard<'static, bool>, String> {
    let gate = supervisor_relay_shutdown_gate()
        .lock()
        .map_err(|_| "supervisor_relay_shutdown_gate_unavailable".to_string())?;
    if *gate {
        Err("supervisor_relay_shutdown_in_progress".to_string())
    } else {
        Ok(gate)
    }
}

#[cfg(test)]
fn clear_supervisor_relay_shutdown_gate_for_test() {
    if let Ok(mut gate) = supervisor_relay_shutdown_gate().lock() {
        *gate = false;
    }
}

fn reserve_safe_only_attempt(attempt_id: &str, confirmation_id: &str) -> Result<(), String> {
    let _gate = raw_manual_relay_visibility_gate()
        .lock()
        .map_err(|_| "manual_relay_safe_only_registry_poisoned".to_string())?;
    let mut attempts = safe_only_attempts()
        .lock()
        .map_err(|_| "manual_relay_safe_only_registry_poisoned".to_string())?;
    if attempts.contains_key(attempt_id) {
        return Err("manual_relay_safe_only_attempt_id_reused".to_string());
    }
    let registry = active_attempts()
        .lock()
        .map_err(|_| "manual_relay_registry_poisoned".to_string())?;
    if registry.contains_key(attempt_id) {
        return Err("manual_relay_attempt_id_reused".to_string());
    }
    attempts.insert(attempt_id.to_string(), confirmation_id.to_string());
    Ok(())
}

fn clear_safe_only_attempt(attempt_id: &str) {
    if let Ok(mut attempts) = safe_only_attempts().lock() {
        attempts.remove(attempt_id);
    }
}

fn safe_only_attempt_is_reserved(attempt_id: &str) -> bool {
    safe_only_attempts()
        .lock()
        .map(|attempts| attempts.contains_key(attempt_id))
        .unwrap_or(true)
}

pub(crate) fn reject_raw_safe_only_manual_relay_attempt(attempt_id: &str) -> Result<(), String> {
    let attempts = safe_only_attempts()
        .lock()
        .map_err(|_| "manual_relay_managed_conversation_attempt_protected".to_string())?;
    if attempts.contains_key(attempt_id) {
        Err("manual_relay_managed_conversation_attempt_protected".to_string())
    } else {
        Ok(())
    }
}

fn with_raw_manual_relay_attempt_visibility<T>(
    attempt_id: &str,
    access: impl FnOnce() -> Result<T, String>,
) -> Result<T, String> {
    let _gate = raw_manual_relay_visibility_gate()
        .lock()
        .map_err(|_| "manual_relay_managed_conversation_attempt_protected".to_string())?;
    reject_raw_safe_only_manual_relay_attempt(attempt_id)?;
    access()
}

/// Non-supervisor writers share the active map with host-owned supervisor
/// relays.  Acquire the visibility gate before checking the safe-only marker
/// and inserting, so a generic attempt cannot take an id during a retained
/// supervisor cleanup window.
fn with_non_safe_only_active_attempt_insertion<T>(
    attempt_id: &str,
    insert: impl FnOnce(&mut BTreeMap<String, ActiveManualRelayAttempt>) -> Result<T, String>,
) -> Result<T, String> {
    let _visibility_gate = raw_manual_relay_visibility_gate()
        .lock()
        .map_err(|_| "manual_relay_managed_conversation_attempt_protected".to_string())?;
    let markers = safe_only_attempts()
        .lock()
        .map_err(|_| "manual_relay_managed_conversation_attempt_protected".to_string())?;
    if markers.contains_key(attempt_id) {
        return Err("manual_relay_managed_conversation_attempt_protected".to_string());
    }
    drop(markers);
    let mut registry = active_attempts()
        .lock()
        .map_err(|_| "manual_relay_registry_poisoned".to_string())?;
    insert(&mut registry)
}

/// A supervisor slot should make this fallback unreachable during ordinary
/// starts.  It still handles a future missed insertion site or an injected
/// collision without overwriting the old owner: the new child receives a
/// fresh, host-only safe key before any retry route is returned.
fn move_supervisor_active_attempt_to_collision_recovery(
    attempt_id: &str,
    confirmation_id: &str,
    mut active: ActiveManualRelayAttempt,
) -> (String, ManualRelayReceipt) {
    let _visibility_gate = raw_manual_relay_visibility_gate()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut markers = safe_only_attempts()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut registry = active_attempts()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    loop {
        let nonce = SUPERVISOR_COLLISION_RECOVERY_NONCE.fetch_add(1, Ordering::Relaxed);
        let recovery_attempt_id = format!("{attempt_id}:host-cleanup-recovery:{nonce}");
        if markers.contains_key(&recovery_attempt_id) || registry.contains_key(&recovery_attempt_id)
        {
            continue;
        }
        active.receipt.relay_attempt_id = recovery_attempt_id.clone();
        let receipt = active.receipt.clone();
        markers.insert(recovery_attempt_id.clone(), confirmation_id.to_string());
        registry.insert(recovery_attempt_id.clone(), active);
        if markers
            .get(attempt_id)
            .is_some_and(|owner| owner == confirmation_id)
        {
            markers.remove(attempt_id);
        }
        drop(registry);
        drop(markers);
        let mut consumed = consumed_confirmations()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if consumed
            .get(confirmation_id)
            .is_some_and(|owner| owner == "reserved" || owner == attempt_id)
        {
            consumed.insert(confirmation_id.to_string(), recovery_attempt_id.clone());
        }
        return (recovery_attempt_id, receipt);
    }
}

#[cfg(test)]
pub(crate) fn reserve_safe_only_manual_relay_attempt_for_test(
    attempt_id: &str,
) -> Result<(), String> {
    reserve_safe_only_attempt(attempt_id, "test-confirmation")
}

#[cfg(test)]
pub(crate) fn clear_safe_only_manual_relay_attempt_for_test(attempt_id: &str) {
    clear_safe_only_attempt(attempt_id);
}

#[cfg(test)]
pub(crate) fn install_safe_only_fixture_attempt_for_test(attempt_id: &str) -> Result<(), String> {
    let confirmation_id = format!("{attempt_id}:confirmation");
    reserve_safe_only_attempt(attempt_id, &confirmation_id)?;
    let envelope = ManualRelayEnvelope {
        relay_id: "manual-relay-safe-only-test".to_string(),
        target_binding: ManualRelayTargetBinding {
            project_root_canonical: "/tmp/manual-relay-safe-only-test".to_string(),
            target_cwd_canonical: "/tmp/manual-relay-safe-only-test".to_string(),
            target_session_id: None,
            new_session: true,
            sandbox: "read-only".to_string(),
            allowed_write_roots: Vec::new(),
            target_hash: "safe-only-test-target".to_string(),
            path_verified: true,
        },
        payload: ManualRelayPayload {
            original_user_text: "fixture".to_string(),
            effective_prompt: "fixture".to_string(),
            payload_layers: Vec::new(),
            prompt_sha256: sha256_hex("fixture"),
            prompt_length_bytes: 7,
            exact_original: true,
        },
        policy: ManualRelayPolicy {
            manual_once: true,
            auto_chain: false,
            duplicate_scope: format!("safe-only-test:{attempt_id}"),
            denied_material_policy: "fixture".to_string(),
        },
        future_hooks: ManualRelayFutureHooks {
            role_id: None,
            task_package_ref: None,
            memory_packet_ref: None,
            supervisor_review_ref: None,
            post_run_memory_capture_policy: None,
        },
        audit_refs: Vec::new(),
        receipt_refs: Vec::new(),
    };
    let receipt = fixture_receipt(
        attempt_id,
        &confirmation_id,
        "running",
        &envelope,
        ManualRelayCommandPlan {
            program: "fixture".to_string(),
            argv: Vec::new(),
            stdin_prompt_ref: "fixture".to_string(),
            stdin_prompt_sha256: sha256_hex("fixture"),
            prompt_in_command: false,
            shell_invocation: false,
            redacted_preview: "fixture".to_string(),
            last_message_path: "supervisor-memory-only".to_string(),
        },
        "2026-07-23T00:00:00Z",
        false,
        false,
        None,
    );
    let active = ActiveManualRelayAttempt {
        duplicate_scope: envelope.policy.duplicate_scope,
        status: "running".to_string(),
        receipt,
        child: None,
        completed_status: "completed_fixture".to_string(),
        output_paths: None,
        supervisor_capture: None,
        process_registration: None,
    };
    let mut attempts = active_attempts()
        .lock()
        .map_err(|_| "manual_relay_registry_poisoned".to_string())?;
    if attempts.contains_key(attempt_id) {
        drop(attempts);
        clear_safe_only_attempt(attempt_id);
        return Err("manual_relay_safe_only_attempt_id_reused".to_string());
    }
    attempts.insert(attempt_id.to_string(), active);
    drop(attempts);
    consumed_confirmations()
        .lock()
        .map_err(|_| "manual_relay_confirmation_registry_poisoned".to_string())?
        .insert(confirmation_id, attempt_id.to_string());
    Ok(())
}

#[cfg(test)]
pub(crate) fn safe_only_fixture_attempt_is_cleared_for_test(attempt_id: &str) -> bool {
    let active_clear = active_attempts()
        .lock()
        .map(|attempts| !attempts.contains_key(attempt_id))
        .unwrap_or(false);
    let protected_clear = safe_only_attempts()
        .lock()
        .map(|attempts| !attempts.contains_key(attempt_id))
        .unwrap_or(false);
    active_clear && protected_clear
}

/// The command-layer lifecycle tests need a host-observed thread id without
/// emitting any child output or widening the fixture into a real transport.
#[cfg(test)]
pub(crate) fn set_safe_only_fixture_thread_id_for_test(
    attempt_id: &str,
    thread_id: &str,
) -> Result<(), String> {
    let mut attempts = active_attempts()
        .lock()
        .map_err(|_| "manual_relay_registry_poisoned".to_string())?;
    let active = attempts
        .get_mut(attempt_id)
        .ok_or_else(|| "manual_relay_attempt_not_running".to_string())?;
    active.receipt.thread_event_summary.thread_id = Some(thread_id.to_string());
    Ok(())
}

/// Offline fixture for the command-level outer-registration failure paths.
/// It owns only a temporary mock shell process, a temporary durable registry
/// entry, and an in-memory supervisor capture; it never starts Codex or
/// touches the application's real workflow state.
#[cfg(test)]
pub(crate) struct SafeOnlySupervisorCleanupFixtureForTest {
    attempt_id: String,
    root: PathBuf,
    workflow_state_path: PathBuf,
    process_group_id: u32,
    ready_path: PathBuf,
    leaked_path: PathBuf,
    capture_state: Arc<Mutex<SupervisorRelayMemoryCaptureState>>,
}

#[cfg(test)]
static SAFE_ONLY_SUPERVISOR_CLEANUP_FIXTURE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[cfg(test)]
#[derive(Clone, Copy)]
enum SafeOnlySupervisorCleanupFixtureMode {
    RunningLeader,
    LeaderExitsBeforePoll,
}

#[cfg(test)]
impl SafeOnlySupervisorCleanupFixtureForTest {
    pub(crate) fn wait_until_child_ready(&self) -> Result<(), String> {
        let started = Instant::now();
        while !self.ready_path.exists() && started.elapsed() < Duration::from_secs(3) {
            thread::sleep(Duration::from_millis(25));
        }
        if self.ready_path.exists() {
            Ok(())
        } else {
            Err("manual_relay_cleanup_fixture_child_not_ready".to_string())
        }
    }

    fn capture_is_cleared(&self) -> Result<bool, String> {
        self.capture_state
            .lock()
            .map_err(|_| "manual_relay_supervisor_capture_registry_poisoned".to_string())
            .map(|state| {
                state.cleared
                    && state.total_bytes == 0
                    && state.stdout_frame.is_empty()
                    && !state.discard_stdout_until_newline
                    && state.report.live_events.is_empty()
                    && state.report.summary.event_types.is_empty()
                    && state.report.summary.thread_id.is_none()
                    && state.report.summary.assistant_item_id.is_none()
                    && state.report.summary.assistant_message_text.is_none()
                    && !state.stderr_seen
                    && !state.overflowed
                    && !state.parse_failed
                    && !state.io_failed
                    && state.redaction_markers.is_empty()
            })
    }

    /// Make the running safe-only poll take its fail-closed terminal path
    /// without placing any raw process output in the test fixture.
    pub(crate) fn force_terminal_capture_failure(&self) -> Result<(), String> {
        let mut state = self
            .capture_state
            .lock()
            .map_err(|_| "manual_relay_supervisor_capture_registry_poisoned".to_string())?;
        state.parse_failed = true;
        Ok(())
    }

    pub(crate) fn force_terminal_capture_overflow(&self) -> Result<(), String> {
        let mut state = self
            .capture_state
            .lock()
            .map_err(|_| "manual_relay_supervisor_capture_registry_poisoned".to_string())?;
        state.overflowed = true;
        Ok(())
    }

    /// A persistent stop failure must retain every authoritative recovery
    /// handle.  The later trusted retry is the only path allowed to clear it.
    pub(crate) fn is_retained_for_trusted_retry(&self) -> Result<bool, String> {
        let confirmation_id = format!("{}:confirmation", self.attempt_id);
        let active_retained = active_attempts()
            .lock()
            .map_err(|_| "manual_relay_registry_poisoned".to_string())
            .map(|attempts| attempts.contains_key(&self.attempt_id))?;
        let protected_retained = safe_only_attempts()
            .lock()
            .map_err(|_| "manual_relay_safe_only_registry_poisoned".to_string())
            .map(|attempts| attempts.contains_key(&self.attempt_id))?;
        let confirmation_retained = consumed_confirmations()
            .lock()
            .map_err(|_| "manual_relay_confirmation_registry_poisoned".to_string())
            .map(|confirmations| confirmations.contains_key(&confirmation_id))?;
        let durable_retained = !crate::exec_process_registry::
            temporary_durable_process_registry_is_empty_for_cleanup_test(&self.workflow_state_path)?;
        Ok(active_retained
            && protected_retained
            && confirmation_retained
            && durable_retained
            && self.capture_is_cleared()?)
    }

    pub(crate) fn is_fully_cleared(&self) -> Result<bool, String> {
        let capture_cleared = self.capture_is_cleared()?;
        let confirmation_id = format!("{}:confirmation", self.attempt_id);
        let confirmation_cleared = consumed_confirmations()
            .lock()
            .map_err(|_| "manual_relay_confirmation_registry_poisoned".to_string())
            .map(|confirmations| !confirmations.contains_key(&confirmation_id))?;
        let durable_cleared = crate::exec_process_registry::
            temporary_durable_process_registry_is_empty_for_cleanup_test(&self.workflow_state_path)?;
        #[cfg(unix)]
        let process_group_cleared = !manual_relay_process_group_exists(self.process_group_id)?;
        #[cfg(not(unix))]
        let process_group_cleared = true;
        Ok(
            safe_only_fixture_attempt_is_cleared_for_test(&self.attempt_id)
                && confirmation_cleared
                && durable_cleared
                && capture_cleared
                && process_group_cleared
                && !self.leaked_path.exists(),
        )
    }
}

#[cfg(test)]
impl Drop for SafeOnlySupervisorCleanupFixtureForTest {
    fn drop(&mut self) {
        let _ = abort_safe_only_manual_relay_attempt(
            ManualRelayStopInput {
                relay_attempt_id: self.attempt_id.clone(),
                requested_by: "safe_only_supervisor_cleanup_fixture_drop".to_string(),
            },
            "2026-07-23T00:00:00Z",
        );
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[cfg(test)]
pub(crate) fn install_safe_only_supervisor_cleanup_fixture_for_test(
    attempt_id: &str,
) -> Result<SafeOnlySupervisorCleanupFixtureForTest, String> {
    install_safe_only_supervisor_cleanup_fixture_with_mode_for_test(
        attempt_id,
        SafeOnlySupervisorCleanupFixtureMode::RunningLeader,
    )
}

#[cfg(test)]
fn install_safe_only_supervisor_cleanup_fixture_with_mode_for_test(
    attempt_id: &str,
    mode: SafeOnlySupervisorCleanupFixtureMode,
) -> Result<SafeOnlySupervisorCleanupFixtureForTest, String> {
    let root = std::env::temp_dir().join(format!(
        "manual-relay-safe-only-supervisor-cleanup-{}-{}-{}",
        std::process::id(),
        crate::unix_timestamp_nanos(),
        SAFE_ONLY_SUPERVISOR_CLEANUP_FIXTURE_SEQUENCE.fetch_add(1, Ordering::Relaxed),
    ));
    fs::create_dir_all(&root)
        .map_err(|error| format!("manual_relay_cleanup_fixture_root_create_failed:{error}"))?;
    let workflow_state_path = root.join("workflow-state.json");
    let ready_path = root.join("child-ready-before-cleanup.txt");
    let leaked_path = root.join("child-must-not-write-after-cleanup.txt");
    if let Err(error) = install_safe_only_fixture_attempt_for_test(attempt_id) {
        let _ = fs::remove_dir_all(&root);
        return Err(error);
    }

    let quoted_ready_path = format!(
        "'{}'",
        ready_path.to_string_lossy().replace('\'', "'\\\"'\\\"'")
    );
    let quoted_leaked_path = format!(
        "'{}'",
        leaked_path.to_string_lossy().replace('\'', "'\\\"'\\\"'")
    );
    let shell_body = match mode {
        SafeOnlySupervisorCleanupFixtureMode::RunningLeader => format!(
            "( printf 'ready\\n' > {quoted_ready_path}; sleep 1; printf 'leaked\\n' > {quoted_leaked_path} ) &\nsleep 30\n"
        ),
        SafeOnlySupervisorCleanupFixtureMode::LeaderExitsBeforePoll => format!(
            "( trap '' TERM; printf 'ready\\n' > {quoted_ready_path}; sleep 2; printf 'leaked\\n' > {quoted_leaked_path} ) &\nexit 0\n"
        ),
    };
    let mut command = Command::new("/bin/sh");
    command
        .arg("-c")
        .arg(shell_body)
        .current_dir(&root)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    configure_manual_relay_process_group(&mut command);
    let child = match command.spawn() {
        Ok(child) => child,
        Err(error) => {
            let _ = abort_safe_only_manual_relay_attempt(
                ManualRelayStopInput {
                    relay_attempt_id: attempt_id.to_string(),
                    requested_by: "safe_only_supervisor_cleanup_fixture_start".to_string(),
                },
                "2026-07-23T00:00:00Z",
            );
            let _ = fs::remove_dir_all(&root);
            return Err(format!("manual_relay_cleanup_fixture_spawn_failed:{error}"));
        }
    };
    let mut child = Some(child);
    let mut capture = match SupervisorRelayMemoryCapture::start(
        child.as_mut().and_then(|running| running.stdout.take()),
        child.as_mut().and_then(|running| running.stderr.take()),
        SupervisorRelayExecutionPolicy::new(Vec::new()),
    ) {
        Ok(capture) => Some(capture),
        Err(error) => {
            if let Some(child) = child.as_mut() {
                let _ = cleanup_manual_relay_child_process(child);
            }
            let _ = abort_safe_only_manual_relay_attempt(
                ManualRelayStopInput {
                    relay_attempt_id: attempt_id.to_string(),
                    requested_by: "safe_only_supervisor_cleanup_fixture_capture".to_string(),
                },
                "2026-07-23T00:00:00Z",
            );
            let _ = fs::remove_dir_all(&root);
            return Err(error);
        }
    };
    let capture_state = Arc::clone(
        &capture
            .as_ref()
            .expect("successful supervisor capture remains available")
            .state,
    );
    let process_id = child
        .as_ref()
        .expect("successful fixture child remains available")
        .id();
    let mut registration =
        match crate::exec_process_registry::register_temporary_durable_process_for_cleanup_test(
            &workflow_state_path,
            "safe-only-supervisor-cleanup-fixture",
            process_id,
        ) {
            Ok(registration) => Some(registration),
            Err(error) => {
                if let (Some(child), Some(capture)) = (child.take(), capture.take()) {
                    let _ = cleanup_spawned_manual_relay_parts(child, None, Some(capture));
                }
                let _ = abort_safe_only_manual_relay_attempt(
                    ManualRelayStopInput {
                        relay_attempt_id: attempt_id.to_string(),
                        requested_by: "safe_only_supervisor_cleanup_fixture_registration"
                            .to_string(),
                    },
                    "2026-07-23T00:00:00Z",
                );
                let _ = fs::remove_dir_all(&root);
                return Err(error);
            }
        };
    let installed = active_attempts()
        .lock()
        .map_err(|_| "manual_relay_registry_poisoned".to_string())
        .and_then(|mut attempts| {
            let active = attempts
                .get_mut(attempt_id)
                .ok_or_else(|| "manual_relay_cleanup_fixture_attempt_missing".to_string())?;
            active.receipt.process_id = Some(process_id);
            active.child = child.take();
            active.supervisor_capture = capture.take();
            active.process_registration = registration.take();
            Ok(())
        });
    if let Err(error) = installed {
        if let (Some(child), capture, registration) =
            (child.take(), capture.take(), registration.take())
        {
            let _ = cleanup_spawned_manual_relay_parts(child, registration, capture);
        }
        let _ = abort_safe_only_manual_relay_attempt(
            ManualRelayStopInput {
                relay_attempt_id: attempt_id.to_string(),
                requested_by: "safe_only_supervisor_cleanup_fixture_install".to_string(),
            },
            "2026-07-23T00:00:00Z",
        );
        let _ = fs::remove_dir_all(&root);
        return Err(error);
    }
    Ok(SafeOnlySupervisorCleanupFixtureForTest {
        attempt_id: attempt_id.to_string(),
        root,
        workflow_state_path,
        process_group_id: process_id,
        ready_path,
        leaked_path,
        capture_state,
    })
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
    let active_attempt_id = attempt_id.clone();
    with_non_safe_only_active_attempt_insertion(&attempt_id, move |registry| {
        if registry.contains_key(&active_attempt_id) {
            return Err("manual_relay_attempt_id_reused".to_string());
        }
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
        registry.insert(active_attempt_id, attempt);
        Ok(())
    })
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

fn clear_consumed_confirmation_attempt(confirmation_id: &str, attempt_id: &str) {
    if let Ok(mut consumed) = consumed_confirmations().lock() {
        if consumed
            .get(confirmation_id)
            .is_some_and(|value| value == "reserved" || value == attempt_id)
        {
            consumed.remove(confirmation_id);
        }
    }
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

fn validate_gui_direct_new_session_input(
    input: &ManualRelayGuiDirectNewSessionInput,
) -> Result<(), String> {
    if input.original_user_text.trim().is_empty() {
        return Err("manual_relay_gui_direct_prompt_required".to_string());
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

fn validate_gui_direct_new_session_target_and_command_plan(
    envelope: &ManualRelayEnvelope,
    command_plan: &ManualRelayCommandPlan,
) -> Result<(), String> {
    verify_strict_run_paths(envelope)?;
    if !envelope.target_binding.new_session || envelope.target_binding.target_session_id.is_some() {
        return Err("manual_relay_gui_direct_new_session_must_not_bind_session".to_string());
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
    if !command_plan.argv.windows(2).any(|pair| {
        pair.first().is_some_and(|arg| arg == "-C")
            && pair
                .get(1)
                .is_some_and(|arg| arg == &envelope.target_binding.target_cwd_canonical)
    }) {
        return Err("manual_relay_gui_direct_command_missing_target_cwd".to_string());
    }
    if command_plan.argv.iter().any(|arg| arg == "resume") {
        return Err("manual_relay_gui_direct_new_session_must_not_resume".to_string());
    }
    if command_plan
        .argv
        .iter()
        .any(|arg| codex_approval_bypass_arg(arg))
    {
        return Err("manual_relay_gui_direct_command_contains_approval_bypass".to_string());
    }
    if !command_plan.argv.iter().any(|arg| arg == "--json") {
        return Err("manual_relay_gui_direct_command_missing_json".to_string());
    }
    if command_plan.prompt_in_command || command_plan.shell_invocation {
        return Err("manual_relay_gui_direct_command_must_use_stdin_no_shell".to_string());
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
    configure_manual_relay_process_group(&mut command);
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
        assistant_message_text: None,
        thread_event_summary: empty_thread_event_summary(),
        live_events: Vec::new(),
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

/// Manual-relay fixture state is process-global.  Tests outside this module
/// also exercise the trusted conversation transport, so they must share this
/// guard rather than racing a fixture install against shutdown/cleanup tests.
#[cfg(test)]
pub(crate) fn manual_relay_test_guard_for_shared_state() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let guard = LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    clear_supervisor_relay_shutdown_gate_for_test();
    let stale_attempts = std::mem::take(
        &mut *active_attempts()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()),
    );
    for (attempt_id, attempt) in stale_attempts {
        let _ = cleanup_removed_manual_relay_attempt(attempt, &attempt_id, false);
    }
    safe_only_attempts()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clear();
    consumed_confirmations()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clear();
    guard
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::mpsc;
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::thread;
    use std::time::{Duration, Instant};

    fn supervisor_capture_state_for_test(
        markers: Vec<String>,
    ) -> Arc<Mutex<SupervisorRelayMemoryCaptureState>> {
        Arc::new(Mutex::new(SupervisorRelayMemoryCaptureState {
            total_bytes: 0,
            stdout_frame: Vec::new(),
            discard_stdout_until_newline: false,
            report: ManualRelayThreadEventReport {
                summary: empty_thread_event_summary(),
                live_events: Vec::new(),
            },
            stderr_seen: false,
            overflowed: false,
            parse_failed: false,
            io_failed: false,
            cleared: false,
            redaction_markers: markers,
        }))
    }

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
    fn raw_visibility_gate_serializes_pre_spawn_marker_with_raw_access() {
        let _guard = test_guard();
        let attempt_id = format!(
            "manual-relay-raw-visibility-gate:{}",
            test_temp_suffix("gate")
        );
        let (raw_checked_tx, raw_checked_rx) = mpsc::channel();
        let (continue_raw_tx, continue_raw_rx) = mpsc::channel();
        let (start_result_tx, start_result_rx) = mpsc::channel();
        let raw_attempt_id = attempt_id.clone();
        let raw = thread::spawn(move || {
            let poll_attempt_id = raw_attempt_id.clone();
            with_raw_manual_relay_attempt_visibility(&raw_attempt_id, || {
                raw_checked_tx
                    .send(())
                    .expect("raw test reports its visibility check");
                continue_raw_rx
                    .recv()
                    .expect("raw test waits until the competing start is blocked");
                poll_manual_relay_attempt_trusted(
                    ManualRelayPollInput {
                        relay_attempt_id: poll_attempt_id,
                        requested_by: "raw-fixture".to_string(),
                    },
                    "2026-07-23T00:00:00Z",
                )
            })
        });
        raw_checked_rx
            .recv_timeout(Duration::from_secs(1))
            .expect("raw access must hold the visibility gate before start reserves");

        let start_attempt_id = attempt_id.clone();
        let starter = thread::spawn(move || {
            start_result_tx
                .send(reserve_safe_only_attempt(
                    &start_attempt_id,
                    "safe-confirmation",
                ))
                .expect("start test reports its reservation result");
        });
        assert!(
            start_result_rx
                .recv_timeout(Duration::from_millis(100))
                .is_err(),
            "safe-only reservation must wait while a raw access has linearized first"
        );

        continue_raw_tx
            .send(())
            .expect("allow the raw access to complete without an active attempt");
        assert_eq!(
            raw.join()
                .expect("raw access thread must not panic")
                .expect_err("raw access that won first may only observe no active attempt"),
            "manual_relay_attempt_not_running"
        );
        start_result_rx
            .recv_timeout(Duration::from_secs(1))
            .expect("safe-only reservation must proceed after raw access exits")
            .expect("safe-only reservation must succeed before spawn");
        starter
            .join()
            .expect("start reservation thread must not panic");

        assert_eq!(
            poll_manual_relay_attempt(
                ManualRelayPollInput {
                    relay_attempt_id: attempt_id.clone(),
                    requested_by: "raw-fixture".to_string(),
                },
                "2026-07-23T00:00:01Z",
            )
            .expect_err("raw access after the pre-spawn reservation must be protected"),
            "manual_relay_managed_conversation_attempt_protected"
        );
        clear_safe_only_attempt(&attempt_id);
    }

    #[test]
    fn raw_visibility_gate_serializes_pre_spawn_marker_with_raw_stop_access() {
        let _guard = test_guard();
        let attempt_id = format!(
            "manual-relay-raw-stop-visibility-gate:{}",
            test_temp_suffix("stop-gate")
        );
        let (raw_checked_tx, raw_checked_rx) = mpsc::channel();
        let (continue_raw_tx, continue_raw_rx) = mpsc::channel();
        let (start_result_tx, start_result_rx) = mpsc::channel();
        let raw_attempt_id = attempt_id.clone();
        let raw = thread::spawn(move || {
            let stop_attempt_id = raw_attempt_id.clone();
            with_raw_manual_relay_attempt_visibility(&raw_attempt_id, || {
                raw_checked_tx
                    .send(())
                    .expect("raw stop test reports its visibility check");
                continue_raw_rx
                    .recv()
                    .expect("raw stop test waits until the competing start is blocked");
                stop_manual_relay_attempt_trusted(
                    ManualRelayStopInput {
                        relay_attempt_id: stop_attempt_id,
                        requested_by: "raw-stop-fixture".to_string(),
                    },
                    "2026-07-23T00:00:00Z",
                )
            })
        });
        raw_checked_rx
            .recv_timeout(Duration::from_secs(1))
            .expect("raw stop access must hold the visibility gate before start reserves");

        let start_attempt_id = attempt_id.clone();
        let starter = thread::spawn(move || {
            start_result_tx
                .send(reserve_safe_only_attempt(
                    &start_attempt_id,
                    "safe-stop-confirmation",
                ))
                .expect("start test reports its reservation result");
        });
        assert!(
            start_result_rx
                .recv_timeout(Duration::from_millis(100))
                .is_err(),
            "safe-only reservation must wait while a raw stop access has linearized first"
        );

        continue_raw_tx
            .send(())
            .expect("allow the raw stop access to complete without an active attempt");
        assert_eq!(
            raw.join()
                .expect("raw stop access thread must not panic")
                .expect_err("raw stop that won first may only observe no active attempt"),
            "manual_relay_attempt_not_running"
        );
        start_result_rx
            .recv_timeout(Duration::from_secs(1))
            .expect("safe-only reservation must proceed after raw stop access exits")
            .expect("safe-only reservation must succeed before spawn");
        starter
            .join()
            .expect("start reservation thread must not panic");

        assert_eq!(
            stop_manual_relay_attempt(
                ManualRelayStopInput {
                    relay_attempt_id: attempt_id.clone(),
                    requested_by: "raw-stop-fixture".to_string(),
                },
                "2026-07-23T00:00:01Z",
            )
            .expect_err("raw stop after the pre-spawn reservation must be protected"),
            "manual_relay_managed_conversation_attempt_protected"
        );
        clear_safe_only_attempt(&attempt_id);
    }

    #[cfg(unix)]
    #[test]
    fn supervisor_stop_failure_retains_child_for_reverse_cleanup() {
        let _guard = test_guard();
        let attempt_id = format!(
            "manual-relay-stop-cleanup:{}",
            test_temp_suffix("stop-cleanup")
        );
        let confirmation_id = format!("{attempt_id}:confirmation");
        let marker_dir = std::env::temp_dir().join(format!(
            "manual-relay-stop-cleanup-marker-{}",
            test_temp_suffix("stop-cleanup-marker")
        ));
        fs::create_dir_all(&marker_dir).expect("fixture marker directory");
        let leaked_path = marker_dir.join("leaked-after-stop-failure.txt");
        let script = mock_codex_script(
            "supervisor-stop-final-wait-failure",
            &format!(
                "#!/bin/sh\ntrap '' TERM\n( sleep 1; printf 'leaked\\n' > \"{}\" ) &\nwhile :; do :; done\n",
                leaked_path.display()
            ),
        );
        let preview = preview_manual_relay(
            existing_fixture_preview_input("supervisor stop cleanup fixture"),
            "2026-07-23T00:00:00Z",
        );
        let command_plan = preview
            .guard
            .command_plan
            .clone()
            .expect("fixture command plan");
        let mut command = Command::new(&script);
        command
            .current_dir(&preview.envelope.target_binding.target_cwd_canonical)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        configure_manual_relay_process_group(&mut command);
        let mut child = command.spawn().expect("fixture child starts");
        let supervisor_capture = SupervisorRelayMemoryCapture::start(
            child.stdout.take(),
            child.stderr.take(),
            SupervisorRelayExecutionPolicy::new(Vec::new()),
        )
        .expect("fixture starts bounded supervisor capture");
        let mut receipt = fixture_receipt(
            &attempt_id,
            &confirmation_id,
            "running",
            &preview.envelope,
            command_plan,
            "2026-07-23T00:00:00Z",
            false,
            false,
            None,
        );
        receipt.process_id = Some(child.id());
        reserve_safe_only_attempt(&attempt_id, &confirmation_id)
            .expect("fixture marks the safe-only attempt before child cleanup");
        active_attempts()
            .lock()
            .expect("active registry lock")
            .insert(
                attempt_id.clone(),
                ActiveManualRelayAttempt {
                    duplicate_scope: format!("safe-stop:{attempt_id}"),
                    status: "running".to_string(),
                    receipt,
                    child: Some(child),
                    completed_status: "completed_fixture".to_string(),
                    output_paths: None,
                    supervisor_capture: Some(supervisor_capture),
                    process_registration: None,
                },
            );
        set_consumed_confirmation_attempt(&confirmation_id, &attempt_id)
            .expect("fixture consumes its confirmation");

        let error = {
            let _failure = force_manual_relay_child_stop_test_failure();
            stop_safe_only_manual_relay_attempt(
                ManualRelayStopInput {
                    relay_attempt_id: attempt_id.clone(),
                    requested_by: "trusted-fixture".to_string(),
                },
                "2026-07-23T00:00:01Z",
            )
            .expect_err("injected stop failure must take the reverse cleanup path")
        };
        assert_eq!(error, "supervisor_relay_cleanup_failed");
        thread::sleep(Duration::from_millis(1200));
        assert!(
            !leaked_path.exists(),
            "retry cleanup must kill the process group before its child writes"
        );
        assert!(
            !active_attempts()
                .lock()
                .expect("active registry lock")
                .contains_key(&attempt_id),
            "reverse cleanup must remove the active attempt"
        );
        assert!(
            !safe_only_attempts()
                .lock()
                .expect("safe-only registry lock")
                .contains_key(&attempt_id),
            "reverse cleanup must clear the pre-spawn marker"
        );
        assert!(
            !consumed_confirmations()
                .lock()
                .expect("confirmation registry lock")
                .contains_key(&confirmation_id),
            "reverse cleanup must clear the consumed confirmation"
        );
        let _ = fs::remove_dir_all(&marker_dir);
    }

    #[test]
    fn supervisor_memory_capture_redacts_cross_chunk_stdout_stderr_and_json_errors() {
        let _guard = test_guard();
        let sentinel = "relay-secret-sentinel-cross-chunk";
        let state = supervisor_capture_state_for_test(vec![sentinel.to_string()]);

        record_supervisor_capture_chunk(
            &state,
            SupervisorCaptureStream::Stdout,
            format!(
                "{{\"type\":\"item.completed\",\"item\":{{\"id\":\"item\",\"type\":\"agent_message\",\"text\":\"before-{sentinel}"
            )
            .as_bytes(),
        );
        record_supervisor_capture_chunk(&state, SupervisorCaptureStream::Stdout, b"-after\"}}\n");
        record_supervisor_capture_chunk(
            &state,
            SupervisorCaptureStream::Stderr,
            format!("stderr-{sentinel}").as_bytes(),
        );
        record_supervisor_capture_chunk(
            &state,
            SupervisorCaptureStream::Stdout,
            format!("{{\"type\":\"error\",\"message\":\"json-error-{sentinel}\"}}\n").as_bytes(),
        );

        let snapshot = supervisor_capture_snapshot(&state);
        assert!(snapshot.stderr_seen);
        assert!(snapshot.report.summary.turn_failed);
        let rendered = format!("{:?}", snapshot.report);
        assert!(!rendered.contains(sentinel));
        assert!(rendered.contains("supervisor_relay_child_failed"));
        assert!(
            snapshot
                .report
                .live_events
                .iter()
                .all(|event| event.arguments_preview.is_none()
                    && event.output_preview.is_none()
                    && event.stderr.is_none()
                    && event.stdout.is_none()),
            "supervisor safe projection must not retain raw process material"
        );
    }

    #[test]
    fn supervisor_memory_capture_never_creates_capture_or_last_message_files_and_clears_on_terminal(
    ) {
        let _guard = test_guard();
        let sentinel = "relay-secret-sentinel-no-disk";
        let artifact_root = std::env::temp_dir().join(format!(
            "manual-relay-supervisor-no-disk-{}",
            test_temp_suffix("supervisor-no-disk")
        ));
        let script_body = format!(
            "#!/bin/sh\nprintf '%s' '{{\"type\":\"item.completed\",\"item\":{{\"id\":\"item\",\"type\":\"agent_message\",\"text\":\"{sentinel}\"'\nprintf '%s\\n' '\"}}}}'\nprintf '%s\\n' '{{\"type\":\"error\",\"message\":\"{sentinel}\"}}'\nprintf '%s\\n' 'stderr-{sentinel}' >&2\n"
        );
        let script = mock_codex_script("supervisor-memory-only", &script_body);
        let preview = preview_manual_relay(
            existing_fixture_preview_input("supervisor memory-only fixture"),
            "2026-07-23T00:00:00Z",
        );
        let mut command_plan = preview
            .guard
            .command_plan
            .clone()
            .expect("fixture command plan");
        command_plan.program = script.display().to_string();
        command_plan.argv.clear();
        command_plan.last_message_path =
            artifact_root.join("last-message.txt").display().to_string();

        let mut spawned = spawn_codex_like_process_capture_to_files(
            &command_plan,
            &preview.envelope,
            None,
            None,
            Some(SupervisorRelayExecutionPolicy::new(vec![
                sentinel.to_string()
            ])),
            None,
        )
        .expect("supervisor fixture child must spawn without a real Codex CLI");
        assert!(spawned.output_paths.is_none());
        assert!(spawned.process_registration.is_none());
        assert!(spawned
            .child
            .wait()
            .expect("fixture child must exit")
            .success());
        let mut capture = spawned
            .supervisor_capture
            .take()
            .expect("supervisor child uses memory-only capture");
        let snapshot = capture.finish_snapshot_and_clear();
        assert!(snapshot.report.summary.turn_failed);
        let rendered = format!("{:?}", snapshot.report);
        assert!(!rendered.contains(sentinel));
        assert!(
            !artifact_root.exists(),
            "supervisor mode must never create last-message or thread-events files"
        );
        let cleared = capture.running_snapshot();
        assert!(cleared.report.live_events.is_empty());
        assert!(cleared.report.summary.event_types.is_empty());

        let mut receipt = fixture_receipt(
            "supervisor-memory-only-attempt",
            "supervisor-memory-only-confirmation",
            "running",
            &preview.envelope,
            command_plan,
            "2026-07-23T00:00:01Z",
            false,
            false,
            None,
        );
        apply_thread_event_report_to_receipt(&mut receipt, snapshot.report);
        let receipt_bytes = serde_json::to_vec(&receipt).expect("safe receipt serializes");
        assert!(!String::from_utf8_lossy(&receipt_bytes).contains(sentinel));

        let _ = fs::remove_dir_all(&artifact_root);
    }

    #[test]
    fn supervisor_usage_keys_are_allowlisted_and_unknown_keys_fail_closed() {
        let _guard = test_guard();
        let sentinel = format!(
            "relay-secret-sentinel-usage-key-{}",
            test_temp_suffix("supervisor-usage-key")
        );
        let state = supervisor_capture_state_for_test(vec![sentinel.clone()]);
        let frame =
            format!(r#"{{"type":"turn.completed","usage":{{"input_tokens":7,"{sentinel}":1}}}}"#);
        let midpoint = frame.len() / 2;
        record_supervisor_capture_chunk(
            &state,
            SupervisorCaptureStream::Stdout,
            &frame.as_bytes()[..midpoint],
        );
        let mut suffix = frame.as_bytes()[midpoint..].to_vec();
        suffix.push(b'\n');
        record_supervisor_capture_chunk(&state, SupervisorCaptureStream::Stdout, &suffix);

        let snapshot = supervisor_capture_snapshot(&state);
        assert!(
            snapshot.failed_closed,
            "an unknown usage key must make the supervisor capture fail closed"
        );
        assert_eq!(snapshot.report.summary.usage.get("input_tokens"), Some(&7));
        assert!(!snapshot.report.summary.usage.contains_key(&sentinel));
        assert!(
            !format!("{:?}", snapshot.report).contains(&sentinel),
            "the raw usage key must not enter the supervisor report"
        );
    }

    #[test]
    fn supervisor_sentinel_temp_artifacts_are_byte_clean_before_cleanup() {
        fn assert_no_sentinel_in_tree(path: &Path, sentinel: &[u8]) {
            for entry in fs::read_dir(path).expect("scan root stays readable") {
                let entry = entry.expect("scan entry stays readable");
                let entry_path = entry.path();
                let file_type = entry.file_type().expect("scan entry type stays readable");
                if file_type.is_dir() {
                    assert_no_sentinel_in_tree(&entry_path, sentinel);
                } else if file_type.is_file() {
                    let bytes = fs::read(&entry_path).expect("scan artifact bytes stay readable");
                    assert!(
                        !bytes
                            .windows(sentinel.len())
                            .any(|window| window == sentinel),
                        "temporary artifact must not retain the supervisor sentinel: {}",
                        entry_path.display()
                    );
                }
            }
        }

        let _guard = test_guard();
        let sentinel = format!(
            "relay-secret-sentinel-byte-scan-{}",
            test_temp_suffix("supervisor-byte-scan")
        );
        let midpoint = sentinel.len() / 2;
        let (sentinel_prefix, sentinel_suffix) = sentinel.split_at(midpoint);
        let artifact_root = std::env::temp_dir().join(format!(
            "manual-relay-supervisor-byte-scan-{}",
            test_temp_suffix("supervisor-byte-scan-root")
        ));
        fs::create_dir_all(&artifact_root).expect("fixture artifact root");
        let script = artifact_root.join("supervisor-output.sh");
        let mut script_body =
            format!("#!/bin/sh\nfirst='{sentinel_prefix}'\nsecond='{sentinel_suffix}'\n");
        script_body.push_str(
            r#"printf '%s' '{"type":"item.completed","item":{"id":"item","type":"agent_message","text":"before-'
printf '%s' "$first"
printf '%s' "$second"
printf '%s\n' '-after"}}'
printf '%s' '{"type":"error","message":"json-error-'
printf '%s' "$first"
printf '%s' "$second"
printf '%s\n' '"}'
printf '%s' '{"type":"turn.completed","usage":{"'
printf '%s' "$first"
printf '%s' "$second"
printf '%s\n' '":1,"input_tokens":7}}'
printf '%s' 'stderr-' >&2
printf '%s' "$first" >&2
printf '%s\n' "$second" >&2
"#,
        );
        fs::write(&script, script_body).expect("fixture script");
        let mut permissions = fs::metadata(&script)
            .expect("fixture script metadata")
            .permissions();
        permissions.set_mode(0o700);
        fs::set_permissions(&script, permissions).expect("fixture script executable");

        let preview = preview_manual_relay(
            existing_fixture_preview_input("supervisor sentinel byte scan"),
            "2026-07-23T00:00:00Z",
        );
        let mut command_plan = preview
            .guard
            .command_plan
            .clone()
            .expect("fixture command plan");
        command_plan.program = script.display().to_string();
        command_plan.argv.clear();
        command_plan.last_message_path = artifact_root
            .join("capture")
            .join("last-message.txt")
            .display()
            .to_string();
        let workflow_state_path = artifact_root.join("workflow-state.json");
        let registration_run_id = "supervisor-byte-scan-registration";
        let mut spawned = {
            let _temporary_registration =
                force_supervisor_relay_temporary_durable_registration_for_test();
            spawn_codex_like_process_capture_to_files(
                &command_plan,
                &preview.envelope,
                None,
                Some(registration_run_id),
                Some(SupervisorRelayExecutionPolicy::new(vec![sentinel.clone()])),
                Some(&workflow_state_path),
            )
            .expect("supervisor fixture child must spawn with memory-only capture")
        };
        assert!(spawned.output_paths.is_none());
        assert!(
            !artifact_root.join("capture").exists(),
            "supervisor capture must never create a last-message or thread-events directory"
        );
        assert!(spawned
            .child
            .wait()
            .expect("fixture child must exit")
            .success());
        let mut capture = spawned
            .supervisor_capture
            .take()
            .expect("supervisor fixture keeps its capture in memory");
        let snapshot = capture.finish_snapshot_and_clear();
        let rendered_snapshot = format!("{:?}", snapshot.report);
        assert!(!rendered_snapshot.contains(&sentinel));
        assert!(snapshot.report.summary.turn_failed);

        let mut safe_receipt = fixture_receipt(
            "supervisor-byte-scan-attempt",
            "supervisor-byte-scan-confirmation",
            "running",
            &preview.envelope,
            command_plan,
            "2026-07-23T00:00:01Z",
            false,
            false,
            None,
        );
        apply_thread_event_report_to_receipt(&mut safe_receipt, snapshot.report);
        fs::write(
            artifact_root.join("safe-receipt.json"),
            serde_json::to_vec(&safe_receipt).expect("safe receipt serializes"),
        )
        .expect("write test-only safe receipt artifact");

        let protected_attempt_id = "supervisor-byte-scan-protected-attempt";
        reserve_safe_only_attempt(protected_attempt_id, "supervisor-byte-scan-confirmation")
            .expect("fixture reserves raw-protected attempt");
        let raw_poll = poll_manual_relay_attempt(
            ManualRelayPollInput {
                relay_attempt_id: protected_attempt_id.to_string(),
                requested_by: "raw-byte-scan-test".to_string(),
            },
            "2026-07-23T00:00:01Z",
        )
        .expect_err("raw poll remains protected");
        let raw_stop = stop_manual_relay_attempt(
            ManualRelayStopInput {
                relay_attempt_id: protected_attempt_id.to_string(),
                requested_by: "raw-byte-scan-test".to_string(),
            },
            "2026-07-23T00:00:01Z",
        )
        .expect_err("raw stop remains protected");
        assert_eq!(
            raw_poll,
            "manual_relay_managed_conversation_attempt_protected"
        );
        assert_eq!(
            raw_stop,
            "manual_relay_managed_conversation_attempt_protected"
        );
        fs::write(
            artifact_root.join("raw-command-results.json"),
            serde_json::to_vec(&vec![raw_poll, raw_stop])
                .expect("fixed protected errors serialize"),
        )
        .expect("write test-only raw result artifact");
        clear_safe_only_attempt(protected_attempt_id);

        assert!(
            !crate::exec_process_registry::
                temporary_durable_process_registry_is_empty_for_cleanup_test(&workflow_state_path)
                    .expect("temporary durable sidecar remains readable"),
            "the scan must include an occupied durable registry sidecar"
        );
        assert_no_sentinel_in_tree(&artifact_root, sentinel.as_bytes());

        spawned
            .process_registration
            .as_mut()
            .expect("supervisor fixture keeps a durable registration")
            .unregister_preserving_on_error()
            .expect("test-side durable registration unregisters");
        let _ = fs::remove_dir_all(&artifact_root);
    }

    #[test]
    fn supervisor_memory_capture_total_limit_fails_closed_without_retaining_a_frame() {
        let _guard = test_guard();
        let state = supervisor_capture_state_for_test(Vec::new());
        record_supervisor_capture_chunk(
            &state,
            SupervisorCaptureStream::Stdout,
            &vec![b'x'; SUPERVISOR_CAPTURE_MAX_TOTAL_BYTES + 1],
        );
        let snapshot = supervisor_capture_snapshot(&state);
        assert!(snapshot.failed_closed);
        assert!(snapshot.report.live_events.is_empty());
        assert!(snapshot.report.summary.event_types.is_empty());
    }

    #[test]
    fn supervisor_terminal_parse_and_overflow_failures_reap_full_safe_only_fixture() {
        let _guard = test_guard();
        for failure in ["parse", "overflow"] {
            let attempt_id = format!(
                "supervisor-terminal-{failure}-cleanup:{}:{}",
                std::process::id(),
                crate::unix_timestamp_nanos()
            );
            let fixture = install_safe_only_supervisor_cleanup_fixture_for_test(&attempt_id)
                .expect("fixture installs child, durable registration, and bounded capture");
            fixture
                .wait_until_child_ready()
                .expect("fixture background child must run before terminal cleanup");
            match failure {
                "parse" => fixture
                    .force_terminal_capture_failure()
                    .expect("fixture injects a closed parse failure"),
                "overflow" => fixture
                    .force_terminal_capture_overflow()
                    .expect("fixture injects a closed overflow failure"),
                _ => unreachable!(),
            }

            let receipt = poll_safe_only_manual_relay_attempt(
                ManualRelayPollInput {
                    relay_attempt_id: attempt_id.clone(),
                    requested_by: "trusted-supervisor-terminal-fixture".to_string(),
                },
                "2026-07-23T00:00:01Z",
            )
            .expect("trusted terminal poll must reap a failed supervisor capture");
            assert_eq!(receipt.status, "failed_process");
            assert!(receipt.thread_event_summary.turn_failed);
            thread::sleep(Duration::from_millis(1200));
            assert!(
                fixture
                    .is_fully_cleared()
                    .expect("fixture cleanup state remains readable"),
                "{failure} terminal cleanup must clear child group, durable registration, active/protected state, confirmation, and capture"
            );
        }
    }

    #[test]
    fn supervisor_terminal_parse_failure_with_persistent_cleanup_retains_full_safe_fixture() {
        let _guard = test_guard();
        let attempt_id = format!(
            "supervisor-terminal-parse-persistent-cleanup:{}:{}",
            std::process::id(),
            crate::unix_timestamp_nanos()
        );
        let fixture = install_safe_only_supervisor_cleanup_fixture_for_test(&attempt_id)
            .expect("fixture installs child, durable registration, and bounded capture");
        fixture
            .wait_until_child_ready()
            .expect("fixture background child must run before terminal cleanup");
        fixture
            .force_terminal_capture_failure()
            .expect("fixture injects a closed parse failure");

        let error = {
            let _persistent_stop_failures = force_manual_relay_child_stop_test_failures_for_test(3);
            poll_safe_only_manual_relay_attempt(
                ManualRelayPollInput {
                    relay_attempt_id: attempt_id.clone(),
                    requested_by: "trusted-supervisor-terminal-persistent-fixture".to_string(),
                },
                "2026-07-23T00:00:01Z",
            )
            .expect_err("persistent terminal cleanup failure must retain the safe-only attempt")
        };
        assert_eq!(error, "supervisor_relay_poll_cleanup_failed");
        assert!(
            fixture
                .is_retained_for_trusted_retry()
                .expect("fixture retained state remains readable"),
            "terminal parse cleanup must retain child, durable entry, active attempt, marker, confirmation, and cleared capture until trusted retry"
        );
        for result in [
            poll_manual_relay_attempt(
                ManualRelayPollInput {
                    relay_attempt_id: attempt_id.clone(),
                    requested_by: "raw-terminal-test".to_string(),
                },
                "2026-07-23T00:00:01Z",
            ),
            stop_manual_relay_attempt(
                ManualRelayStopInput {
                    relay_attempt_id: attempt_id.clone(),
                    requested_by: "raw-terminal-test".to_string(),
                },
                "2026-07-23T00:00:01Z",
            ),
        ] {
            assert_eq!(
                result.expect_err("raw endpoint remains closed while terminal cleanup is pending"),
                "manual_relay_managed_conversation_attempt_protected"
            );
        }

        abort_safe_only_manual_relay_attempt(
            ManualRelayStopInput {
                relay_attempt_id: attempt_id.clone(),
                requested_by: "trusted-terminal-retry".to_string(),
            },
            "2026-07-23T00:00:02Z",
        )
        .expect("trusted retry must settle the terminal-retained attempt");
        thread::sleep(Duration::from_millis(1200));
        assert!(
            fixture
                .is_fully_cleared()
                .expect("fixture cleanup state remains readable"),
            "trusted retry must clear every resource retained after terminal parse cleanup failure"
        );
    }

    #[test]
    fn retained_supervisor_attempt_rejects_competing_non_safe_active_write() {
        let _guard = test_guard();
        let attempt_id = format!(
            "supervisor-retained-competing-generic-write:{}:{}",
            std::process::id(),
            crate::unix_timestamp_nanos()
        );
        let fixture = install_safe_only_supervisor_cleanup_fixture_for_test(&attempt_id)
            .expect("fixture installs child, durable registration, and bounded capture");
        fixture
            .wait_until_child_ready()
            .expect("fixture child must run before cleanup failure");

        let competing_write = force_supervisor_relay_retain_competing_non_safe_insert_for_test();
        let error = {
            let _persistent_stop_failures = force_manual_relay_child_stop_test_failures_for_test(3);
            abort_safe_only_manual_relay_attempt(
                ManualRelayStopInput {
                    relay_attempt_id: attempt_id.clone(),
                    requested_by: "trusted-competing-generic-write-fixture".to_string(),
                },
                "2026-07-23T00:00:01Z",
            )
            .expect_err("failed cleanup must retain the host-owned supervisor attempt")
        };
        assert_eq!(error, "supervisor_relay_cleanup_failed");
        assert_eq!(
            supervisor_relay_retain_competing_non_safe_insert_result_for_test().as_deref(),
            Some("manual_relay_managed_conversation_attempt_protected"),
            "the ordinary running-attempt writer must reject the protected id before it can replace the cleanup owner"
        );
        assert!(
            fixture
                .is_retained_for_trusted_retry()
                .expect("retained fixture state remains readable"),
            "child, durable identity, capture, marker, and confirmation must remain paired with the original supervisor attempt"
        );
        drop(competing_write);

        abort_safe_only_manual_relay_attempt(
            ManualRelayStopInput {
                relay_attempt_id: attempt_id.clone(),
                requested_by: "trusted-competing-generic-write-retry".to_string(),
            },
            "2026-07-23T00:00:02Z",
        )
        .expect("trusted retry must still own and settle the original supervisor attempt");
        thread::sleep(Duration::from_millis(1200));
        assert!(
            fixture
                .is_fully_cleared()
                .expect("fixture cleanup state remains readable"),
            "trusted retry must clear the original child, durable identity, capture, marker, and confirmation"
        );
    }

    #[cfg(unix)]
    #[test]
    fn supervisor_terminal_poll_sweeps_descendants_before_capture_finalization() {
        let _guard = test_guard();
        let attempt_id = format!(
            "supervisor-terminal-descendant-sweep:{}:{}",
            std::process::id(),
            crate::unix_timestamp_nanos()
        );
        let fixture = install_safe_only_supervisor_cleanup_fixture_with_mode_for_test(
            &attempt_id,
            SafeOnlySupervisorCleanupFixtureMode::LeaderExitsBeforePoll,
        )
        .expect("fixture installs an exited leader with a TERM-ignoring descendant");
        fixture
            .wait_until_child_ready()
            .expect("fixture descendant must be running before terminal poll");
        thread::sleep(Duration::from_millis(150));

        let started = Instant::now();
        let receipt = poll_safe_only_manual_relay_attempt(
            ManualRelayPollInput {
                relay_attempt_id: attempt_id.clone(),
                requested_by: "trusted-supervisor-terminal-descendant-fixture".to_string(),
            },
            "2026-07-23T00:00:01Z",
        )
        .expect("terminal poll must sweep the remaining process group before joining capture");
        assert_eq!(receipt.status, "completed_fixture");
        assert!(
            started.elapsed() < Duration::from_secs(1),
            "capture finalization must not wait for a surviving descendant to close its pipes"
        );
        assert!(
            fixture
                .is_fully_cleared()
                .expect("terminal cleanup state remains readable"),
            "leader exit must not clear durable/protected state until its descendant group is gone"
        );
        thread::sleep(Duration::from_millis(2200));
        assert!(
            fixture
                .is_fully_cleared()
                .expect("descendant must remain absent after its delayed write deadline"),
            "the TERM-ignoring descendant must be killed before it can write or retain a pipe"
        );
    }

    #[test]
    fn supervisor_app_shutdown_reaps_full_safe_only_fixture() {
        let _guard = test_guard();
        let attempt_id = format!(
            "supervisor-app-shutdown-cleanup:{}:{}",
            std::process::id(),
            crate::unix_timestamp_nanos()
        );
        let fixture = install_safe_only_supervisor_cleanup_fixture_for_test(&attempt_id)
            .expect("fixture installs child, durable registration, and bounded capture");
        fixture
            .wait_until_child_ready()
            .expect("fixture background child must run before app shutdown");

        assert_eq!(stop_all_active_manual_relay_attempts().unwrap(), 1);
        thread::sleep(Duration::from_millis(1200));
        assert!(
            fixture
                .is_fully_cleared()
                .expect("fixture cleanup state remains readable"),
            "app shutdown must clear child group, durable registration, active/protected state, confirmation, and capture"
        );
    }

    #[test]
    fn supervisor_app_shutdown_persistent_cleanup_failure_retains_every_safe_handle_for_retry() {
        let _guard = test_guard();
        let attempt_id = format!(
            "supervisor-app-shutdown-persistent-cleanup:{}:{}",
            std::process::id(),
            crate::unix_timestamp_nanos()
        );
        let fixture = install_safe_only_supervisor_cleanup_fixture_for_test(&attempt_id)
            .expect("fixture installs child, durable registration, and bounded capture");
        fixture
            .wait_until_child_ready()
            .expect("fixture background child must run before app shutdown");

        let error = {
            let _persistent_stop_failures = force_manual_relay_child_stop_test_failures_for_test(3);
            stop_all_active_manual_relay_attempts()
                .expect_err("persistent shutdown cleanup failure must retain the protected attempt")
        };
        assert!(error.starts_with("manual_relay_shutdown_cleanup_failed:"));
        assert!(
            fixture
                .is_retained_for_trusted_retry()
                .expect("fixture retained state remains readable"),
            "shutdown must retain child, durable entry, active attempt, marker, confirmation, and cleared capture until trusted retry"
        );
        for result in [
            poll_manual_relay_attempt(
                ManualRelayPollInput {
                    relay_attempt_id: attempt_id.clone(),
                    requested_by: "raw-shutdown-test".to_string(),
                },
                "2026-07-23T00:00:01Z",
            ),
            stop_manual_relay_attempt(
                ManualRelayStopInput {
                    relay_attempt_id: attempt_id.clone(),
                    requested_by: "raw-shutdown-test".to_string(),
                },
                "2026-07-23T00:00:01Z",
            ),
        ] {
            assert_eq!(
                result.expect_err("raw endpoint remains closed after failed shutdown cleanup"),
                "manual_relay_managed_conversation_attempt_protected"
            );
        }

        abort_safe_only_manual_relay_attempt(
            ManualRelayStopInput {
                relay_attempt_id: attempt_id.clone(),
                requested_by: "trusted-shutdown-retry".to_string(),
            },
            "2026-07-23T00:00:02Z",
        )
        .expect("trusted retry must settle the shutdown-retained attempt");
        thread::sleep(Duration::from_millis(1200));
        assert!(
            fixture
                .is_fully_cleared()
                .expect("fixture cleanup state remains readable"),
            "trusted retry must clear every resource retained by failed shutdown cleanup"
        );
    }

    #[test]
    fn supervisor_start_gate_rejects_reservation_while_shutdown_is_pending() {
        let _guard = test_guard();
        {
            let mut shutdown = supervisor_relay_shutdown_gate()
                .lock()
                .expect("shutdown gate lock");
            *shutdown = true;
        }
        let result = acquire_supervisor_relay_start_gate();
        match result {
            Ok(_) => panic!("supervisor start must not cross an active shutdown gate"),
            Err(error) => assert_eq!(error, "supervisor_relay_shutdown_in_progress"),
        }
        clear_supervisor_relay_shutdown_gate_for_test();
    }

    #[test]
    fn supervisor_registration_and_stdin_failures_reap_child_and_leave_no_durable_sink() {
        let _guard = test_guard();
        for (label, failure_point, stdin_prompt, force_registration_error, expected_error) in [
            (
                "registration",
                None,
                None,
                true,
                "supervisor_relay_process_registration_failed",
            ),
            (
                "stdin",
                Some(SupervisorRelaySpawnTestFailurePoint::StdinWrite),
                Some("fixture prompt"),
                false,
                "supervisor_relay_process_stdin_write_failed",
            ),
        ] {
            let marker_dir = std::env::temp_dir().join(format!(
                "manual-relay-supervisor-cleanup-{label}-{}",
                test_temp_suffix(label)
            ));
            fs::create_dir_all(&marker_dir).expect("fixture marker directory");
            let workflow_state_path = if force_registration_error {
                let non_directory_parent = marker_dir.join("not-a-workflow-state-directory");
                fs::write(&non_directory_parent, b"fixture")
                    .expect("fixture creates a non-directory registration parent");
                non_directory_parent.join("workflow-state.json")
            } else {
                marker_dir.join("workflow-state.json")
            };
            let leaked_path = marker_dir.join("leaked-after-failure.txt");
            let script = mock_codex_script(
                &format!("supervisor-cleanup-{label}"),
                &format!(
                    "#!/bin/sh\n( sleep 1; printf 'leaked\\n' > \"{}\" ) &\nsleep 30\n",
                    leaked_path.display()
                ),
            );
            let preview = preview_manual_relay(
                existing_fixture_preview_input(&format!("supervisor cleanup {label}")),
                "2026-07-23T00:00:00Z",
            );
            let artifact_root = marker_dir.join("must-stay-empty");
            let mut command_plan = preview
                .guard
                .command_plan
                .clone()
                .expect("fixture command plan");
            command_plan.program = script.display().to_string();
            command_plan.argv.clear();
            command_plan.last_message_path =
                artifact_root.join("last-message.txt").display().to_string();

            let start = || {
                spawn_codex_like_process_capture_to_files(
                    &command_plan,
                    &preview.envelope,
                    stdin_prompt,
                    Some("supervisor-cleanup-failure-fixture"),
                    Some(SupervisorRelayExecutionPolicy::new(Vec::new())),
                    Some(&workflow_state_path),
                )
            };
            let result = {
                let _temporary_registration =
                    force_supervisor_relay_temporary_durable_registration_for_test();
                match failure_point {
                    Some(failure_point) => {
                        let _failure = force_supervisor_relay_spawn_test_failure(failure_point);
                        start()
                    }
                    None => start(),
                }
            };
            let error = match result {
                Ok(_) => panic!("injected supervisor post-spawn failure must return an error"),
                Err(SpawnCodexLikeProcessError::Failed(error)) => error,
                Err(SpawnCodexLikeProcessError::SupervisorCleanupPending(_)) => {
                    panic!("single cleanup failure injection must settle before returning")
                }
            };
            assert_eq!(error, expected_error);
            thread::sleep(Duration::from_millis(1200));
            assert!(
                !leaked_path.exists(),
                "{label} failure must kill the child process group before its background child writes"
            );
            assert!(
                !artifact_root.exists(),
                "{label} failure must not leave a supervisor last-message or capture sink"
            );
            if !force_registration_error {
                assert!(
                    crate::exec_process_registry::
                        temporary_durable_process_registry_is_empty_for_cleanup_test(
                            &workflow_state_path,
                        )
                        .expect("temporary durable registry remains readable"),
                    "{label} failure must unregister an already-durable supervisor child"
                );
            }
            let _ = fs::remove_dir_all(&marker_dir);
        }
    }

    #[test]
    fn supervisor_pre_active_persistent_cleanup_retains_durable_safe_attempt_for_retry() {
        let _guard = test_guard();
        let root = std::env::temp_dir().join(format!(
            "manual-relay-supervisor-pre-active-pending-{}",
            test_temp_suffix("pre-active-pending")
        ));
        fs::create_dir_all(&root).expect("temporary fixture root");
        let workflow_state_path = root.join("workflow-state.json");
        let ready_path = root.join("child-ready.txt");
        let leaked_path = root.join("child-leaked-after-retry.txt");
        let script = mock_codex_script(
            "supervisor-pre-active-persistent-cleanup",
            &format!(
                "#!/bin/sh\n( printf 'ready\\n' > \"{}\"; sleep 1; printf 'leaked\\n' > \"{}\" ) &\nsleep 30\n",
                ready_path.display(),
                leaked_path.display(),
            ),
        );
        let preview = preview_manual_relay(
            existing_fixture_preview_input("supervisor pre-active pending cleanup"),
            "2026-07-23T00:00:00Z",
        );
        let mut command_plan = preview
            .guard
            .command_plan
            .clone()
            .expect("fixture command plan");
        command_plan.program = script.display().to_string();
        command_plan.argv.clear();
        command_plan.last_message_path = root.join("must-stay-empty.txt").display().to_string();
        let attempt_id = format!(
            "supervisor-pre-active-pending:{}:{}",
            std::process::id(),
            crate::unix_timestamp_nanos()
        );
        let confirmation_id = format!("{attempt_id}:confirmation");
        let scope = format!("safe-only-pre-active:{attempt_id}");
        reserve_safe_only_attempt(&attempt_id, &confirmation_id)
            .expect("safe-only marker reserves the attempt before spawn");

        let result = {
            let _child_failures = force_manual_relay_child_stop_test_failures_for_test(2);
            let _temporary_registration =
                force_supervisor_relay_temporary_durable_registration_for_test();
            let _stdin_failure = force_supervisor_relay_spawn_test_failure(
                SupervisorRelaySpawnTestFailurePoint::StdinWrite,
            );
            spawn_running_codex_like_process(
                &scope,
                &confirmation_id,
                attempt_id.clone(),
                &preview.envelope,
                ManualRelayProcessConfig {
                    command_plan,
                    process_kind: "supervisor-test-process".to_string(),
                    real_codex_executed: true,
                    return_running: true,
                    completed_status: "completed_supervisor_test_process".to_string(),
                },
                "2026-07-23T00:00:00Z",
                false,
                Some(SupervisorRelayExecutionPolicy::new(Vec::new())),
                Some(&workflow_state_path),
            )
        };
        let pending_receipt = result.expect(
            "persistent cleanup must retain a pre-active supervisor child behind a safe receipt",
        );
        assert_eq!(pending_receipt.status, "supervisor_relay_cleanup_pending");
        assert!(
            !pending_receipt.prompt_sent,
            "an injected stdin-write failure must never claim prompt delivery while cleanup is pending"
        );
        assert!(
            pending_receipt
                .warnings
                .iter()
                .any(|warning| warning == "supervisor_relay_cleanup_pending"),
            "the returned receipt must explain only the fixed cleanup-pending family"
        );
        let started = Instant::now();
        while !ready_path.exists() && started.elapsed() < Duration::from_secs(3) {
            thread::sleep(Duration::from_millis(25));
        }
        assert!(
            ready_path.exists(),
            "fixture child must be running before retry"
        );
        let retained_parts = active_attempts()
            .lock()
            .expect("active registry lock")
            .get(&attempt_id)
            .map(|active| {
                (
                    active.child.is_some(),
                    active.supervisor_capture.is_some(),
                    active.process_registration.is_some(),
                )
            });
        assert_eq!(
            retained_parts,
            Some((true, true, true)),
            "pending pre-active cleanup must retain child, capture, and durable registration"
        );
        assert!(
            safe_only_attempts()
                .lock()
                .expect("safe-only registry lock")
                .contains_key(&attempt_id),
            "pre-spawn marker must remain until trusted retry"
        );
        assert!(
            consumed_confirmations()
                .lock()
                .expect("confirmation registry lock")
                .contains_key(&confirmation_id),
            "confirmation reservation must remain paired with pending cleanup"
        );
        assert!(
            !crate::exec_process_registry::
                temporary_durable_process_registry_is_empty_for_cleanup_test(&workflow_state_path)
                    .expect("temporary durable registry remains readable"),
            "registered supervisor child must remain reapable until trusted retry"
        );
        assert_eq!(
            poll_manual_relay_attempt(
                ManualRelayPollInput {
                    relay_attempt_id: attempt_id.clone(),
                    requested_by: "raw-pre-active-test".to_string(),
                },
                "2026-07-23T00:00:01Z",
            )
            .expect_err("raw poll must remain protected while pre-active cleanup is pending"),
            "manual_relay_managed_conversation_attempt_protected"
        );

        abort_safe_only_manual_relay_attempt(
            ManualRelayStopInput {
                relay_attempt_id: attempt_id.clone(),
                requested_by: "trusted-pre-active-retry".to_string(),
            },
            "2026-07-23T00:00:02Z",
        )
        .expect("trusted retry must settle the retained pre-active attempt");
        thread::sleep(Duration::from_millis(1200));
        assert!(
            !leaked_path.exists(),
            "trusted retry must kill the retained process group before its child writes"
        );
        assert!(
            !active_attempts()
                .lock()
                .expect("active registry lock")
                .contains_key(&attempt_id)
                && !safe_only_attempts()
                    .lock()
                    .expect("safe-only registry lock")
                    .contains_key(&attempt_id)
                && !consumed_confirmations()
                    .lock()
                    .expect("confirmation registry lock")
                    .contains_key(&confirmation_id),
            "trusted retry must remove the active, marker, and confirmation state together"
        );
        assert!(
            crate::exec_process_registry::
                temporary_durable_process_registry_is_empty_for_cleanup_test(&workflow_state_path)
                    .expect("temporary durable registry remains readable"),
            "trusted retry must unregister the durable supervisor identity"
        );
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn supervisor_active_slot_collision_fails_before_child_spawn_and_preserves_owner() {
        let _guard = test_guard();
        let root = std::env::temp_dir().join(format!(
            "manual-relay-supervisor-active-slot-preflight-{}",
            test_temp_suffix("active-slot-preflight")
        ));
        fs::create_dir_all(&root).expect("temporary fixture root");
        let spawned_path = root.join("child-spawned.txt");
        let script = mock_codex_script(
            "supervisor-active-slot-preflight",
            &format!(
                "#!/bin/sh\nprintf 'spawned\\n' > \"{}\"\nsleep 30\n",
                spawned_path.display()
            ),
        );
        let preview = preview_manual_relay(
            existing_fixture_preview_input("supervisor active slot preflight"),
            "2026-07-23T00:00:00Z",
        );
        let mut command_plan = preview
            .guard
            .command_plan
            .clone()
            .expect("fixture command plan");
        command_plan.program = script.display().to_string();
        command_plan.argv.clear();
        command_plan.last_message_path = root.join("must-stay-empty.txt").display().to_string();
        let attempt_id = format!(
            "supervisor-active-slot-preflight:{}:{}",
            std::process::id(),
            crate::unix_timestamp_nanos()
        );
        let confirmation_id = format!("{attempt_id}:confirmation");
        reserve_safe_only_attempt(&attempt_id, &confirmation_id)
            .expect("safe-only marker reserves the supervisor id before spawn");

        let result = {
            let _collision = force_supervisor_relay_active_slot_collision_for_test();
            spawn_running_codex_like_process(
                &format!("safe-only-preflight:{attempt_id}"),
                &confirmation_id,
                attempt_id.clone(),
                &preview.envelope,
                ManualRelayProcessConfig {
                    command_plan,
                    process_kind: "supervisor-test-process".to_string(),
                    real_codex_executed: true,
                    return_running: true,
                    completed_status: "completed_supervisor_test_process".to_string(),
                },
                "2026-07-23T00:00:00Z",
                false,
                Some(SupervisorRelayExecutionPolicy::new(Vec::new())),
                None,
            )
        };
        assert_eq!(
            result.expect_err("same active id must fail before child spawn"),
            "manual_relay_attempt_id_reused"
        );
        thread::sleep(Duration::from_millis(100));
        assert!(
            !spawned_path.exists(),
            "preflight collision must not create the child process"
        );
        let preserved = active_attempts()
            .lock()
            .expect("active registry lock")
            .get(&attempt_id)
            .map(|active| {
                (
                    active.status.clone(),
                    active.child.is_some(),
                    active.duplicate_scope.clone(),
                )
            });
        assert_eq!(
            preserved,
            Some((
                "running".to_string(),
                false,
                format!("fixture-active-slot-collision:{attempt_id}"),
            )),
            "the pre-existing owner must remain untouched"
        );
        active_attempts()
            .lock()
            .expect("active registry lock")
            .remove(&attempt_id);
        clear_safe_only_attempt(&attempt_id);
        clear_consumed_confirmation_attempt(&confirmation_id, &attempt_id);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn supervisor_final_active_slot_collision_rekeys_pending_child_and_preserves_old_owner() {
        let _guard = test_guard();
        let root = std::env::temp_dir().join(format!(
            "manual-relay-supervisor-active-slot-recovery-{}",
            test_temp_suffix("active-slot-recovery")
        ));
        fs::create_dir_all(&root).expect("temporary fixture root");
        let workflow_state_path = root.join("workflow-state.json");
        let ready_path = root.join("child-ready.txt");
        let leaked_path = root.join("child-leaked-after-retry.txt");
        let preview = preview_manual_relay(
            existing_fixture_preview_input("supervisor active slot recovery"),
            "2026-07-23T00:00:00Z",
        );
        let mut command_plan = preview
            .guard
            .command_plan
            .clone()
            .expect("fixture command plan");
        // 进程夹具确定性边界：载荷经 argv 喂给常驻温热的 /bin/sh，不新建脚本文件。
        // 新建脚本首次 exec 在本沙箱实测 155ms~3.2s，会撞穿 3s 就绪预算。
        command_plan.program = "/bin/sh".to_string();
        command_plan.argv = vec![
            "-c".to_string(),
            format!(
                "( printf 'ready\\n' > \"{}\"; sleep 1; printf 'leaked\\n' > \"{}\" ) &\nsleep 30\n",
                ready_path.display(),
                leaked_path.display(),
            ),
        ];
        command_plan.last_message_path = root.join("must-stay-empty.txt").display().to_string();
        let attempt_id = format!(
            "supervisor-active-slot-recovery:{}:{}",
            std::process::id(),
            crate::unix_timestamp_nanos()
        );
        let confirmation_id = format!("{attempt_id}:confirmation");
        reserve_safe_only_attempt(&attempt_id, &confirmation_id)
            .expect("safe-only marker reserves the supervisor id before spawn");

        let pending_receipt = {
            let _child_failures = force_manual_relay_child_stop_test_failures_for_test(3);
            let _temporary_registration =
                force_supervisor_relay_temporary_durable_registration_for_test();
            let _collision = force_supervisor_relay_final_active_slot_collision_for_test();
            spawn_running_codex_like_process(
                &format!("safe-only-recovery:{attempt_id}"),
                &confirmation_id,
                attempt_id.clone(),
                &preview.envelope,
                ManualRelayProcessConfig {
                    command_plan,
                    process_kind: "supervisor-test-process".to_string(),
                    real_codex_executed: true,
                    return_running: true,
                    completed_status: "completed_supervisor_test_process".to_string(),
                },
                "2026-07-23T00:00:00Z",
                false,
                Some(SupervisorRelayExecutionPolicy::new(Vec::new())),
                Some(&workflow_state_path),
            )
            .expect("persistent cleanup after a final collision must return a safe retry receipt")
        };
        assert_eq!(pending_receipt.status, "supervisor_relay_cleanup_pending");
        let recovery_attempt_id = pending_receipt.relay_attempt_id.clone();
        assert_ne!(recovery_attempt_id, attempt_id);
        assert!(
            recovery_attempt_id.starts_with(&format!("{attempt_id}:host-cleanup-recovery:")),
            "only a host-generated recovery key may expose the retained new child"
        );
        let owners = active_attempts().lock().expect("active registry lock");
        let old_owner = owners
            .get(&attempt_id)
            .expect("the old owner must survive the collision");
        assert_eq!(
            old_owner.receipt.confirmation_id,
            format!("fixture-foreign-confirmation:{attempt_id}")
        );
        assert!(old_owner.child.is_none());
        let recovery = owners
            .get(&recovery_attempt_id)
            .expect("the new child must be retained under its recovery key");
        assert!(
            recovery.child.is_some()
                && recovery.supervisor_capture.is_some()
                && recovery.process_registration.is_some(),
            "child, capture, and durable registration must stay paired under the new key"
        );
        drop(owners);
        assert!(
            poll_manual_relay_attempt(
                ManualRelayPollInput {
                    relay_attempt_id: recovery_attempt_id.clone(),
                    requested_by: "raw-active-slot-recovery".to_string(),
                },
                "2026-07-23T00:00:01Z",
            )
            .is_err(),
            "the newly retained child must remain raw-protected under its recovery key"
        );
        assert!(
            poll_manual_relay_attempt(
                ManualRelayPollInput {
                    relay_attempt_id: attempt_id.clone(),
                    requested_by: "raw-old-owner".to_string(),
                },
                "2026-07-23T00:00:01Z",
            )
            .is_ok(),
            "the new marker must not silently claim the old owner id"
        );
        let started = Instant::now();
        while !ready_path.exists() && started.elapsed() < Duration::from_secs(3) {
            thread::sleep(Duration::from_millis(25));
        }
        assert!(
            ready_path.exists(),
            "fixture child must run before trusted recovery"
        );

        abort_safe_only_manual_relay_attempt(
            ManualRelayStopInput {
                relay_attempt_id: recovery_attempt_id.clone(),
                requested_by: "trusted-active-slot-recovery".to_string(),
            },
            "2026-07-23T00:00:02Z",
        )
        .expect("trusted recovery must settle the re-keyed child");
        thread::sleep(Duration::from_millis(1200));
        assert!(
            !leaked_path.exists(),
            "trusted recovery must kill the child group before its descendant writes"
        );
        assert!(
            !active_attempts()
                .lock()
                .expect("active registry lock")
                .contains_key(&recovery_attempt_id)
                && !safe_only_attempts()
                    .lock()
                    .expect("safe-only registry lock")
                    .contains_key(&recovery_attempt_id)
                && !consumed_confirmations()
                    .lock()
                    .expect("confirmation registry lock")
                    .contains_key(&confirmation_id),
            "trusted recovery must clear the new active, marker, and confirmation together"
        );
        assert!(
            crate::exec_process_registry::temporary_durable_process_registry_is_empty_for_cleanup_test(
                &workflow_state_path,
            )
            .expect("temporary durable registry remains readable"),
            "trusted recovery must unregister the new durable identity"
        );
        assert!(
            active_attempts()
                .lock()
                .expect("active registry lock")
                .contains_key(&attempt_id),
            "the old owner must still remain after the new child settles"
        );
        active_attempts()
            .lock()
            .expect("active registry lock")
            .remove(&attempt_id);
        clear_safe_only_attempt(&attempt_id);
        let _ = fs::remove_dir_all(&root);
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
            test_temp_suffix("strict paths")
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
    fn manual_relay_gui_direct_new_session_uses_unbound_exec_json_command() {
        let _guard = test_guard();
        std::env::remove_var("MANUAL_RELAY_REAL_CODEX_CONFIRM");
        let script = mock_codex_script(
            "gui-direct-new-session-json-events",
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
printf 'new session json event mock last message: %s\n' "$prompt" > "$last"
printf '%s\n' '{"type":"thread.started","thread_id":"thread-new-session-json-fixture"}'
printf '%s\n' '{"type":"turn.started"}'
printf '%s\n' '{"type":"item.completed","item":{"id":"item-new-session-json-fixture","type":"agent_message","text":"NEW_SESSION_JSON_EVENT_REPLY_OK"}}'
printf '%s\n' '{"type":"turn.completed","usage":{"input_tokens":7,"output_tokens":3}}'
exit 0
"#,
        );
        let fixture = new_session_fixture_preview_input("GUI direct new session exact prompt");
        let input = ManualRelayGuiDirectNewSessionInput {
            original_user_text: fixture.original_user_text.clone(),
            target_project_root: fixture.target_project_root.clone(),
            target_cwd: fixture.target_cwd.clone(),
            sandbox: fixture.sandbox.clone(),
            allowed_write_roots: fixture.allowed_write_roots.clone(),
            requested_by: "user".to_string(),
        };

        let receipt = run_manual_relay_gui_direct_new_session_once_for_test(
            input,
            "2026-06-18T08:30:00Z",
            &format!("mock_codex_process:{}", script.display()),
        )
        .expect("GUI direct new session should run through mock codex process");

        assert_eq!(receipt.status, "completed_mock_codex");
        assert_eq!(receipt.process_kind, "mock_codex");
        assert!(receipt.prompt_sent);
        assert_eq!(receipt.target.target_session_id, None);
        assert!(receipt.target.new_session);
        assert!(!receipt.command_plan.argv.iter().any(|arg| arg == "resume"));
        assert!(receipt.command_plan.argv.iter().any(|arg| arg == "--json"));
        assert!(receipt
            .command_plan
            .argv
            .iter()
            .any(|arg| arg == "--output-last-message"));
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
        assert!(!receipt
            .command_plan
            .argv
            .iter()
            .any(|arg| arg == "--full-auto" || arg.contains("dangerously-bypass")));
        assert_eq!(
            receipt.thread_event_summary.thread_id.as_deref(),
            Some("thread-new-session-json-fixture")
        );
        assert_eq!(
            receipt.assistant_message_text.as_deref(),
            Some("NEW_SESSION_JSON_EVENT_REPLY_OK")
        );
    }

    #[test]
    fn manual_relay_gui_direct_parses_json_thread_events_for_reply_and_completion() {
        let _guard = test_guard();
        std::env::remove_var("MANUAL_RELAY_REAL_CODEX_CONFIRM");
        let script = mock_codex_script(
            "gui-direct-json-events",
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
printf 'json event mock last message: %s\n' "$prompt" > "$last"
printf '%s\n' '{"type":"thread.started","thread_id":"thread-json-fixture"}'
printf '%s\n' '{"type":"turn.started"}'
printf '%s\n' '{"type":"item.completed","item":{"id":"item-json-fixture","type":"agent_message","text":"JSON_EVENT_REPLY_OK"}}'
printf '%s\n' '{"type":"turn.completed","usage":{"input_tokens":10,"cached_input_tokens":2,"output_tokens":4,"reasoning_output_tokens":1}}'
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
            "2026-06-18T09:00:00Z",
            &format!("mock_codex_process:{}", script.display()),
        )
        .expect("GUI direct send should parse mock codex JSON events");

        assert_eq!(receipt.status, "completed_mock_codex");
        assert_eq!(
            receipt.assistant_message_text.as_deref(),
            Some("JSON_EVENT_REPLY_OK")
        );
        assert!(receipt
            .live_events
            .iter()
            .any(|event| event.event_type == "item.completed"
                && event.item_type.as_deref() == Some("agent_message")
                && event.text.as_deref() == Some("JSON_EVENT_REPLY_OK")));
        assert_eq!(
            receipt.thread_event_summary.thread_id.as_deref(),
            Some("thread-json-fixture")
        );
        assert_eq!(
            receipt.thread_event_summary.assistant_item_id.as_deref(),
            Some("item-json-fixture")
        );
        assert!(receipt.thread_event_summary.turn_completed);
        assert_eq!(
            receipt
                .thread_event_summary
                .usage
                .get("reasoning_output_tokens"),
            Some(&1)
        );
        assert_eq!(
            receipt.readback_status,
            "thread_event_agent_message_available"
        );
    }

    #[test]
    fn manual_relay_gui_direct_running_poll_parses_json_thread_events_for_reply_and_completion() {
        let _guard = test_guard();
        std::env::remove_var("MANUAL_RELAY_REAL_CODEX_CONFIRM");
        let script = mock_codex_script(
            "gui-direct-running-json-events",
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
printf 'running json event mock last message\n' > "$last"
sleep 0.2
printf '%s\n' '{"type":"thread.started","thread_id":"thread-running-json-fixture"}'
printf '%s\n' '{"type":"turn.started"}'
printf '%s\n' '{"type":"item.completed","item":{"id":"item-running-json-fixture","type":"agent_message","text":"RUNNING_JSON_EVENT_REPLY_OK"}}'
printf '%s\n' '{"type":"turn.completed","usage":{"input_tokens":12,"cached_input_tokens":3,"output_tokens":5,"reasoning_output_tokens":2}}'
exit 0
"#,
        );
        let fixture = existing_fixture_preview_input("GUI direct running exact prompt");
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

        let running = run_manual_relay_gui_direct_once_for_test(
            input,
            "2026-06-18T09:10:00Z",
            &format!("mock_codex_process_sleep:{}", script.display()),
        )
        .expect("GUI direct running send should spawn mock codex and return running");
        assert_eq!(running.status, "running");
        assert_eq!(running.process_kind, "mock_codex");
        assert!(running.process_id.is_some());
        assert!(running.prompt_sent);
        assert_eq!(running.assistant_message_text, None);

        let completed = poll_manual_relay_attempt_until_terminal_for_test(
            &running.relay_attempt_id,
            "2026-06-18T09:10:01Z",
            5_000,
        )
        .expect("poll should finalize mock codex JSON events");

        assert_eq!(completed.status, "completed_mock_codex");
        assert_eq!(
            completed.assistant_message_text.as_deref(),
            Some("RUNNING_JSON_EVENT_REPLY_OK")
        );
        assert_eq!(
            completed.thread_event_summary.thread_id.as_deref(),
            Some("thread-running-json-fixture")
        );
        assert_eq!(
            completed.thread_event_summary.assistant_item_id.as_deref(),
            Some("item-running-json-fixture")
        );
        assert!(completed.thread_event_summary.turn_completed);
        assert_eq!(
            completed
                .thread_event_summary
                .usage
                .get("reasoning_output_tokens"),
            Some(&2)
        );
        assert_eq!(
            completed.readback_status,
            "thread_event_agent_message_available"
        );
        assert!(active_attempts()
            .lock()
            .expect("registry should not poison")
            .is_empty());
    }

    #[test]
    fn manual_relay_gui_direct_running_poll_returns_live_thread_events_before_completion() {
        let _guard = test_guard();
        std::env::remove_var("MANUAL_RELAY_REAL_CODEX_CONFIRM");
        let script = mock_codex_script(
            "gui-direct-live-thread-events",
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
printf 'running live event mock last message\n' > "$last"
printf '%s\n' '{"type":"thread.started","thread_id":"thread-live-json-fixture"}'
printf '%s\n' '{"type":"turn.started"}'
printf '%s\n' '{"type":"item.started","item":{"id":"item-live-reply","type":"agent_message","text":"LIVE_PARTIAL"}}'
sleep 1
printf '%s\n' '{"type":"item.completed","item":{"id":"item-live-reply","type":"agent_message","text":"LIVE_FINAL_OK"}}'
printf '%s\n' '{"type":"turn.completed","usage":{"input_tokens":14,"output_tokens":6}}'
exit 0
"#,
        );
        let fixture = existing_fixture_preview_input("GUI direct live exact prompt");
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

        let running = run_manual_relay_gui_direct_once_for_test(
            input,
            "2026-06-18T09:20:00Z",
            &format!("mock_codex_process_sleep:{}", script.display()),
        )
        .expect("GUI direct running send should spawn mock codex and return running");
        assert_eq!(running.status, "running");

        let mut live = None;
        for _ in 0..100 {
            let receipt = poll_manual_relay_attempt(
                ManualRelayPollInput {
                    relay_attempt_id: running.relay_attempt_id.clone(),
                    requested_by: "user".to_string(),
                },
                "2026-06-18T09:20:00Z",
            )
            .expect("running poll should return a receipt");
            if receipt.live_events.iter().any(|event| {
                event.event_type == "item.started"
                    && event.item_type.as_deref() == Some("agent_message")
                    && event.text.as_deref() == Some("LIVE_PARTIAL")
            }) {
                live = Some(receipt);
                break;
            }
            thread::sleep(Duration::from_millis(50));
        }

        let live = live.expect("poll should expose live item.started before process completion");
        assert_eq!(live.status, "running");
        assert_eq!(
            live.thread_event_summary.thread_id.as_deref(),
            Some("thread-live-json-fixture")
        );
        assert_eq!(live.assistant_message_text, None);

        let completed = poll_manual_relay_attempt_until_terminal_for_test(
            &running.relay_attempt_id,
            "2026-06-18T09:20:01Z",
            5_000,
        )
        .expect("poll should eventually finalize live mock codex JSON events");
        assert_eq!(
            completed.assistant_message_text.as_deref(),
            Some("LIVE_FINAL_OK")
        );
        assert!(completed.thread_event_summary.turn_completed);
        assert!(completed
            .live_events
            .iter()
            .any(|event| event.event_type == "turn.completed"));
    }

    #[test]
    fn manual_relay_gui_direct_running_stop_kills_mock_process() {
        let _guard = test_guard();
        std::env::remove_var("MANUAL_RELAY_REAL_CODEX_CONFIRM");
        let script = mock_codex_script(
            "gui-direct-running-stop",
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
  printf 'gui direct running stop started\n' > "$last"
fi
sleep 30
"#,
        );
        let fixture = existing_fixture_preview_input("GUI direct running stop");
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

        let running = run_manual_relay_gui_direct_once_for_test(
            input,
            "2026-06-18T09:20:00Z",
            &format!("mock_codex_process_sleep:{}", script.display()),
        )
        .expect("GUI direct running send should spawn mock codex");
        assert_eq!(running.status, "running");
        assert_eq!(running.process_kind, "mock_codex");
        assert!(running.process_id.is_some());

        let stopped = stop_manual_relay_attempt(
            ManualRelayStopInput {
                relay_attempt_id: running.relay_attempt_id,
                requested_by: "user".to_string(),
            },
            "2026-06-18T09:20:01Z",
        )
        .expect("stop must kill GUI direct mock codex process");
        assert_eq!(stopped.status, "stopped_by_user");
        assert!(stopped.killed_by_user);
        assert!(stopped.real_process_killed);
        assert_eq!(stopped.process_kind, "mock_codex");
        assert!(!stopped.real_codex_executed);
    }

    #[test]
    fn manual_relay_terminal_thread_failure_reaps_process_without_waiting_for_exit() {
        let _guard = test_guard();
        let script = mock_codex_script(
            "gui-direct-terminal-failure",
            r#"#!/bin/sh
last=""
while [ "$#" -gt 0 ]; do
  if [ "$1" = "--output-last-message" ]; then
    shift
    last="$1"
  fi
  shift || true
done
printf '%s\n' '{"type":"thread.started","thread_id":"thread-terminal-failure"}'
printf '%s\n' '{"type":"turn.started"}'
printf '%s\n' '{"type":"turn.failed","error":{"message":"fixture transport failed"}}'
sleep 30
"#,
        );
        let fixture = existing_fixture_preview_input("GUI direct terminal failure");
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
        let running = run_manual_relay_gui_direct_once_for_test(
            input,
            "2026-06-18T09:20:00Z",
            &format!("mock_codex_process_sleep:{}", script.display()),
        )
        .expect("terminal failure fixture should start");

        let started = Instant::now();
        let failed = poll_manual_relay_attempt_until_terminal_for_test(
            &running.relay_attempt_id,
            "2026-06-18T09:20:01Z",
            5_000,
        )
        .expect("turn.failed should be reaped as a terminal result");

        assert!(started.elapsed() < Duration::from_secs(5));
        assert_eq!(failed.status, "failed_process");
        assert!(failed.thread_event_summary.turn_failed);
        assert!(failed.real_process_killed);
        assert!(failed
            .warnings
            .contains(&"manual_relay_terminal_thread_failure_reaped".to_string()));
        assert!(active_attempts().lock().unwrap().is_empty());
    }

    #[test]
    fn manual_relay_gui_direct_stop_kills_mock_process_group_children() {
        let _guard = test_guard();
        std::env::remove_var("MANUAL_RELAY_REAL_CODEX_CONFIRM");
        let marker_dir = std::env::temp_dir().join(format!(
            "manual-relay-process-group-stop-{}",
            test_temp_suffix("gui-direct-process-group-children")
        ));
        std::fs::create_dir_all(&marker_dir).expect("marker dir should be created");
        let ready_path = marker_dir.join("ready.txt");
        let leaked_path = marker_dir.join("leaked.txt");
        let _ = std::fs::remove_file(&ready_path);
        let _ = std::fs::remove_file(&leaked_path);
        let script = mock_codex_script(
            "gui-direct-process-group-stop",
            &format!(
                r#"#!/bin/sh
last=""
while [ "$#" -gt 0 ]; do
  if [ "$1" = "--output-last-message" ]; then
    shift
    last="$1"
  fi
  shift || true
done
( sleep 1; printf 'leaked child survived stop\n' > "{leaked}" ) &
printf 'child spawned\n' > "{ready}"
if [ -n "$last" ]; then
  mkdir -p "$(dirname "$last")"
  printf 'mock codex process group child started\n' > "$last"
fi
wait
"#,
                leaked = leaked_path.display(),
                ready = ready_path.display(),
            ),
        );
        let fixture = existing_fixture_preview_input("GUI direct process group stop");
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

        let running = run_manual_relay_gui_direct_once_for_test(
            input,
            "2026-06-18T09:21:00Z",
            &format!("mock_codex_process_sleep:{}", script.display()),
        )
        .expect("GUI direct running send should spawn mock codex");
        let started = Instant::now();
        while !ready_path.exists() && started.elapsed() < Duration::from_millis(1000) {
            thread::sleep(Duration::from_millis(25));
        }
        assert!(
            ready_path.exists(),
            "mock child must be spawned before Stop"
        );

        let stopped = stop_manual_relay_attempt(
            ManualRelayStopInput {
                relay_attempt_id: running.relay_attempt_id,
                requested_by: "user".to_string(),
            },
            "2026-06-18T09:21:01Z",
        )
        .expect("stop must kill GUI direct mock codex process group");
        assert_eq!(stopped.status, "stopped_by_user");
        assert!(stopped.killed_by_user);
        assert!(stopped.real_process_killed);

        thread::sleep(Duration::from_millis(1300));
        assert!(
            !leaked_path.exists(),
            "Stop must kill child processes in the relay process group"
        );
    }

    #[test]
    fn manual_relay_app_shutdown_kills_active_process_group_children() {
        let _guard = test_guard();
        let marker_dir = std::env::temp_dir().join(format!(
            "manual-relay-app-shutdown-{}",
            test_temp_suffix("app-shutdown-process-group")
        ));
        std::fs::create_dir_all(&marker_dir).expect("marker dir should be created");
        let ready_path = marker_dir.join("ready.txt");
        let leaked_path = marker_dir.join("leaked.txt");
        let _ = std::fs::remove_file(&ready_path);
        let _ = std::fs::remove_file(&leaked_path);
        let mock_body = format!(
            r#"( sleep 1; printf 'leaked child survived shutdown\n' > "{leaked}" ) &
printf 'child spawned\n' > "{ready}"
cat >/dev/null
wait
"#,
            leaked = leaked_path.display(),
            ready = ready_path.display(),
        );
        let fixture = existing_fixture_preview_input("GUI direct app shutdown");
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
        run_manual_relay_gui_direct_once_for_test(
            input,
            "2026-06-18T09:22:00Z",
            &format!("mock_codex_process_sleep_sh:{mock_body}"),
        )
        .expect("app shutdown fixture should start");
        let started = Instant::now();
        while !ready_path.exists() && started.elapsed() < Duration::from_secs(3) {
            thread::sleep(Duration::from_millis(25));
        }
        assert!(ready_path.exists());

        assert_eq!(stop_all_active_manual_relay_attempts().unwrap(), 1);
        assert!(active_attempts().lock().unwrap().is_empty());
        thread::sleep(Duration::from_millis(1300));
        assert!(
            !leaked_path.exists(),
            "app shutdown must kill children in the relay process group"
        );
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
        let suffix = test_temp_suffix(prompt);
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
        let suffix = test_temp_suffix(prompt);
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

    fn new_session_fixture_preview_input(prompt: &str) -> ManualRelayPreviewInput {
        let suffix = test_temp_suffix(prompt);
        let project_root = std::env::temp_dir().join(format!("manual-relay-new-session-{suffix}"));
        std::fs::create_dir_all(&project_root).expect("fixture project root should be created");
        ManualRelayPreviewInput {
            original_user_text: prompt.to_string(),
            target_project_root: project_root.display().to_string(),
            target_cwd: project_root.display().to_string(),
            target_session_id: None,
            new_session: true,
            sandbox: "workspace-write".to_string(),
            allowed_write_roots: vec![project_root.display().to_string()],
            requested_by: "user".to_string(),
        }
    }

    fn mock_codex_script(name: &str, body: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "manual-relay-mock-codex-{}",
            test_temp_suffix(&format!("{name}:{body}"))
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
            test_temp_suffix(label)
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

    fn poll_manual_relay_attempt_until_terminal_for_test(
        relay_attempt_id: &str,
        timestamp: &str,
        timeout_ms: u64,
    ) -> Result<ManualRelayReceipt, String> {
        let started = Instant::now();
        let timeout = Duration::from_millis(timeout_ms.max(1));
        loop {
            let receipt = poll_manual_relay_attempt(
                ManualRelayPollInput {
                    relay_attempt_id: relay_attempt_id.to_string(),
                    requested_by: "user".to_string(),
                },
                timestamp,
            )?;
            if receipt.status != "running" {
                return Ok(receipt);
            }
            if started.elapsed() >= timeout {
                let _ = stop_manual_relay_attempt(
                    ManualRelayStopInput {
                        relay_attempt_id: relay_attempt_id.to_string(),
                        requested_by: "user".to_string(),
                    },
                    timestamp,
                );
                return Err("manual_relay_poll_test_timed_out".to_string());
            }
            thread::sleep(Duration::from_millis(50));
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

    fn test_temp_suffix(label: &str) -> String {
        static SEQUENCE: AtomicU64 = AtomicU64::new(0);
        format!(
            "{}-{}-{}",
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed),
            short_hash(label)
        )
    }

    // 2026-07-08 根治「12-failed 级联」(四次现身·重跑即绿):任一测试**持锁期间 panic** → 三把
    // Mutex 中毒 → 之后 23 个测试全在这里的 expect 上炸=一个真抽风带崩一片。修法:**中毒恢复**
    // (into_inner)——串行锁不带数据,中毒仅意味着"前一个测试挂了",串行语义分毫不损。还要回收
    // 遗留 mock 子进程，不能只 clear 登记表后把它们留在下一个测试夹具之外。
    // 此共享 guard 也供 conversation transport 的安全夹具使用。
    fn test_guard() -> std::sync::MutexGuard<'static, ()> {
        manual_relay_test_guard_for_shared_state()
    }
}
