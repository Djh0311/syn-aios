//! Profile-driven facade over the existing manual relay process transport.
//!
//! The facade deliberately has separate agent and supervisor entry points.  A
//! caller can supply a message/session binding, but never a sandbox, write
//! root, approval policy, MCP capability list, or MCP server definition.

use super::{
    canonical_path_text, codex_approval_bypass_arg, confirm_manual_relay_once,
    preview_manual_relay, run_manual_relay_once_with_process_mode_and_command_profile,
    validate_gui_direct_new_session_target_and_command_plan,
    validate_gui_direct_target_and_command_plan, ManualRelayCommandPlan, ManualRelayConfirmInput,
    ManualRelayPollInput, ManualRelayPreviewInput, ManualRelayProcessMode, ManualRelayReceipt,
    ManualRelayRunInput, ManualRelayStopInput,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Component, Path};
use std::sync::{Mutex, OnceLock};

pub(crate) const AGENT_CODEX_WORKSPACE_WRITE_PROFILE_ID: &str = "agent-codex-workspace-write";
pub(crate) const SUPERVISOR_READ_ONLY_PROFILE_ID: &str = "supervisor-read-only";
pub(crate) const SUPERVISOR_CONVERSATION_RUN_ID_PREFIX: &str = "supervisor-conversation:";

/// Stable host identity for one supervisor turn.  The source identifiers are
/// never copied into argv or an MCP config value; the prefix is shared with
/// the trusted binding validator, while the hash keeps the run id per-turn.
pub(crate) fn supervisor_run_id_for(
    conversation_id: &str,
    turn_id: &str,
) -> Result<String, String> {
    if conversation_id.trim().is_empty() || turn_id.trim().is_empty() {
        return Err("conversation_transport_run_identity_input_required".to_string());
    }
    Ok(format!(
        "{SUPERVISOR_CONVERSATION_RUN_ID_PREFIX}{}",
        crate::utils::hash::sha256_hex(&format!("{conversation_id}\n{turn_id}"))
    ))
}

/// The profile is deliberately selected by a host entry point rather than
/// deserialized from a frontend request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ConversationTransportProfileId {
    AgentCodexWorkspaceWrite,
    SupervisorReadOnly,
}

impl ConversationTransportProfileId {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::AgentCodexWorkspaceWrite => AGENT_CODEX_WORKSPACE_WRITE_PROFILE_ID,
            Self::SupervisorReadOnly => SUPERVISOR_READ_ONLY_PROFILE_ID,
        }
    }
}

/// Message/session data accepted by the shared transport.  It intentionally
/// has no sandbox, write-root, approval, capability, command, or MCP fields.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct ConversationTransportStartInput {
    pub(crate) conversation_id: String,
    pub(crate) turn_id: String,
    pub(crate) original_user_text: String,
    pub(crate) target_project_root: String,
    pub(crate) target_cwd: String,
    pub(crate) target_session_id: Option<String>,
    pub(crate) new_session: bool,
    pub(crate) requested_by: String,
}

/// Host-only context for the supervisor profile.  This type intentionally does
/// not implement `Deserialize`: it must be derived from trusted app state and
/// the turn binding, not supplied by a Tauri/JSON caller.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SupervisorConversationHostContext {
    pub(crate) project_id: String,
    pub(crate) project_root: String,
    pub(crate) workflow_id: String,
    pub(crate) run_id: String,
    pub(crate) workflow_state_path: String,
    pub(crate) max_active_workers: usize,
    pub(crate) max_follow_ups_per_worker: usize,
    pub(crate) max_runtime_minutes: i64,
    /// Host-issued, child-only configuration for the short-lived
    /// `knowledge_open` relay.  It cannot be deserialized from a UI request.
    pub(crate) knowledge_open_relay: crate::knowledge_open_relay::KnowledgeOpenRelayMcpConfig,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct ConversationTransportAttemptInput {
    pub(crate) conversation_id: String,
    pub(crate) turn_id: String,
    pub(crate) attempt_id: String,
    pub(crate) requested_by: String,
}

/// Safe, layered receipt for UI use.  It intentionally omits argv, paths,
/// raw stderr/stdout, tool arguments, environment and process identifiers
/// contained in `ManualRelayReceipt`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct ConversationTransportReceipt {
    pub(crate) profile_id: String,
    pub(crate) conversation_id: String,
    pub(crate) thread_id: Option<String>,
    pub(crate) turn_id: String,
    pub(crate) lifecycle: String,
    pub(crate) transport: ConversationTransportLayerReceipt,
    pub(crate) assistant_reply: ConversationAssistantReplyReceipt,
    pub(crate) tool_action: ConversationLayerReceipt,
    pub(crate) read_model_projection: ConversationLayerReceipt,
    pub(crate) canonical_mirror: ConversationLayerReceipt,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct ConversationTransportLayerReceipt {
    pub(crate) status: String,
    pub(crate) attempt_id: String,
    pub(crate) started_at: String,
    pub(crate) ended_at: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct ConversationAssistantReplyReceipt {
    pub(crate) status: String,
    pub(crate) text: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct ConversationLayerReceipt {
    pub(crate) status: String,
    pub(crate) summary: Option<String>,
}

#[derive(Clone, Debug)]
pub(super) enum ConversationTransportCommandProfile {
    AgentWorkspaceWrite,
    SupervisorReadOnly(SupervisorMcpEndpoint),
}

impl ConversationTransportCommandProfile {
    pub(super) fn supervisor_execution_policy(
        &self,
    ) -> Option<super::SupervisorRelayExecutionPolicy> {
        match self {
            Self::AgentWorkspaceWrite => None,
            Self::SupervisorReadOnly(endpoint) => Some(super::SupervisorRelayExecutionPolicy::new(
                endpoint.output_redaction_markers(),
            )),
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct SupervisorMcpEndpoint {
    executable: String,
    run_id: String,
    workflow_state_path: String,
    max_active_workers: usize,
    max_follow_ups_per_worker: usize,
    max_runtime_minutes: i64,
    knowledge_open_relay: crate::knowledge_open_relay::KnowledgeOpenRelayMcpConfig,
}

#[derive(Clone, Debug)]
struct ConversationTransportAttemptRecord {
    profile_id: ConversationTransportProfileId,
    conversation_id: String,
    turn_id: String,
    target_session_id: Option<String>,
}

pub(crate) fn start_agent_conversation_transport(
    input: ConversationTransportStartInput,
    timestamp: &str,
) -> Result<ConversationTransportReceipt, String> {
    start_conversation_transport(
        input,
        ConversationTransportProfileId::AgentCodexWorkspaceWrite,
        ConversationTransportCommandProfile::AgentWorkspaceWrite,
        timestamp,
    )
}

pub(crate) fn start_supervisor_conversation_transport(
    mut input: ConversationTransportStartInput,
    host: SupervisorConversationHostContext,
    timestamp: &str,
) -> Result<ConversationTransportReceipt, String> {
    validate_supervisor_host_context(&host)?;
    let host_project_root = canonical_path_text(&host.project_root)
        .map_err(|_| "conversation_transport_supervisor_project_root_unverified".to_string())?;
    if host.project_id != crate::project_id(&host_project_root) {
        return Err("conversation_transport_supervisor_project_id_mismatch".to_string());
    }
    let input_project_root = canonical_path_text(&input.target_project_root).map_err(|_| {
        "conversation_transport_supervisor_input_project_root_unverified".to_string()
    })?;
    let input_cwd = canonical_path_text(&input.target_cwd)
        .map_err(|_| "conversation_transport_supervisor_input_cwd_unverified".to_string())?;
    if input_project_root != host_project_root || input_cwd != host_project_root {
        return Err("conversation_transport_supervisor_project_root_mismatch".to_string());
    }

    // Bind the actual canonical host root into the envelope even if the caller
    // used an equivalent spelling.  The profile never accepts caller-selected
    // cwd or write scope.
    input.target_project_root = host_project_root.clone();
    input.target_cwd = host_project_root;
    let endpoint = SupervisorMcpEndpoint::from_host_context(host)?;
    start_conversation_transport(
        input,
        ConversationTransportProfileId::SupervisorReadOnly,
        ConversationTransportCommandProfile::SupervisorReadOnly(endpoint),
        timestamp,
    )
}

pub(crate) fn poll_conversation_transport_attempt(
    input: ConversationTransportAttemptInput,
    timestamp: &str,
) -> Result<ConversationTransportReceipt, String> {
    let record = conversation_attempt_record(&input)?;
    let poll_input = ManualRelayPollInput {
        relay_attempt_id: input.attempt_id.clone(),
        requested_by: input.requested_by,
    };
    let receipt = match record.profile_id {
        ConversationTransportProfileId::SupervisorReadOnly => {
            super::poll_safe_only_manual_relay_attempt(poll_input, timestamp)
        }
        ConversationTransportProfileId::AgentCodexWorkspaceWrite => {
            super::poll_manual_relay_attempt(poll_input, timestamp)
        }
    }
    .map_err(|error| {
        match remove_settled_supervisor_attempt_record_after_trusted_error(
            &record,
            &input.attempt_id,
        ) {
            Ok(()) => error,
            Err(removal_error) => removal_error,
        }
    })?;
    let output = receipt_for_record(&record, &receipt);
    if is_terminal_lifecycle(&output.lifecycle) {
        remove_attempt_record(&input.attempt_id)?;
    }
    Ok(output)
}

pub(crate) fn stop_conversation_transport_attempt(
    input: ConversationTransportAttemptInput,
    timestamp: &str,
) -> Result<ConversationTransportReceipt, String> {
    let record = conversation_attempt_record(&input)?;
    let stop_input = ManualRelayStopInput {
        relay_attempt_id: input.attempt_id.clone(),
        requested_by: input.requested_by,
    };
    let receipt = match record.profile_id {
        ConversationTransportProfileId::SupervisorReadOnly => {
            super::stop_safe_only_manual_relay_attempt(stop_input, timestamp)
        }
        ConversationTransportProfileId::AgentCodexWorkspaceWrite => {
            super::stop_manual_relay_attempt(stop_input, timestamp)
        }
    }
    .map_err(|error| {
        match remove_settled_supervisor_attempt_record_after_trusted_error(
            &record,
            &input.attempt_id,
        ) {
            Ok(()) => error,
            Err(removal_error) => removal_error,
        }
    })?;
    let output = receipt_for_record(&record, &receipt);
    remove_attempt_record(&input.attempt_id)?;
    Ok(output)
}

fn start_conversation_transport(
    input: ConversationTransportStartInput,
    profile_id: ConversationTransportProfileId,
    command_profile: ConversationTransportCommandProfile,
    timestamp: &str,
) -> Result<ConversationTransportReceipt, String> {
    validate_start_input(&input)?;
    let (sandbox, allowed_write_roots) = profile_scope(profile_id, &input.target_project_root);
    let preview = preview_manual_relay(
        ManualRelayPreviewInput {
            original_user_text: input.original_user_text.clone(),
            target_project_root: input.target_project_root.clone(),
            target_cwd: input.target_cwd.clone(),
            target_session_id: input.target_session_id.clone(),
            new_session: input.new_session,
            sandbox,
            allowed_write_roots,
            requested_by: input.requested_by.clone(),
        },
        timestamp,
    );
    if preview.guard.blocks_execution {
        return Err(format!(
            "conversation_transport_guard_blocked:{}",
            preview.guard.reasons.join(",")
        ));
    }
    let Some(mut command_plan) = preview.guard.command_plan.clone() else {
        return Err("conversation_transport_command_plan_missing".to_string());
    };
    apply_command_profile(&preview.envelope, &mut command_plan, &command_profile)?;

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
    let receipt = run_manual_relay_once_with_process_mode_and_command_profile(
        ManualRelayRunInput {
            envelope: preview.envelope,
            confirmation: confirmation.clone(),
            confirmation_id: confirmation.confirmation_id.clone(),
            expected_prompt_sha256: confirmation.prompt_sha256.clone(),
            expected_target_hash: confirmation.target_hash.clone(),
            expected_sandbox: confirmation.sandbox.clone(),
            expected_allowed_write_roots: confirmation.allowed_write_roots.clone(),
            mock_behavior: "conversation_transport_internal_process_mode".to_string(),
        },
        timestamp,
        Some(ManualRelayProcessMode::RealCodexProductGui),
        Some(&command_profile),
    )?;
    let record = ConversationTransportAttemptRecord {
        profile_id,
        conversation_id: input.conversation_id,
        turn_id: input.turn_id,
        target_session_id: input.target_session_id,
    };
    let output = receipt_for_record(&record, &receipt);
    if !is_terminal_lifecycle(&output.lifecycle) {
        if let Err(error) = register_attempt_record(&receipt.relay_attempt_id, record.clone()) {
            let cleanup = abort_started_conversation_transport_attempt(
                record.profile_id,
                &receipt.relay_attempt_id,
                timestamp,
            );
            return match cleanup {
                Ok(()) => Err(error),
                // The running manual attempt remains protected by its
                // pre-spawn marker.  Return the existing redacted receipt so
                // the host command layer can install (or retain) its sole
                // trusted cleanup route rather than dropping an unreachable
                // active child on an inner-record collision/poison.
                Err(_) => Ok(output),
            };
        }
    }
    Ok(output)
}

/// The command-level supervisor registration is intentionally later than the
/// transport registration.  If it fails, only this trusted helper may tear
/// down the still-running safe-only relay; generic raw Tauri endpoints remain
/// closed from the pre-spawn marker onward.
pub(crate) fn abort_supervisor_conversation_transport_attempt(
    attempt_id: &str,
    timestamp: &str,
) -> Result<(), String> {
    let stop_result = super::abort_safe_only_manual_relay_attempt(
        ManualRelayStopInput {
            relay_attempt_id: attempt_id.to_string(),
            requested_by: "conversation_transport_outer_cleanup".to_string(),
        },
        timestamp,
    );
    if stop_result.is_err() {
        // Keep the trusted transport record paired with the protected manual
        // attempt until child/durable cleanup has actually settled.  Removing
        // it here would create a raw-readable-looking half-state while the
        // safe-only marker is deliberately still fail-closed.
        return Err("conversation_transport_outer_cleanup_failed".to_string());
    }
    if super::reject_raw_safe_only_manual_relay_attempt(attempt_id).is_err() {
        return Err("conversation_transport_outer_cleanup_failed".to_string());
    }
    // The absent marker is the authoritative proof that child, durable
    // registration, capture, active attempt, and confirmation were already
    // closed by the trusted abort.  A poisoned inner bookkeeping map must not
    // keep the outer host recovery route alive forever: leave its stale
    // in-memory record fail-closed for this process lifetime and let the
    // command layer remove/revoke its own route.
    let _ = remove_attempt_record(attempt_id);
    Ok(())
}

fn abort_started_conversation_transport_attempt(
    profile_id: ConversationTransportProfileId,
    attempt_id: &str,
    timestamp: &str,
) -> Result<(), String> {
    match profile_id {
        ConversationTransportProfileId::SupervisorReadOnly => {
            super::abort_safe_only_manual_relay_attempt(
                ManualRelayStopInput {
                    relay_attempt_id: attempt_id.to_string(),
                    requested_by: "conversation_transport_inner_cleanup".to_string(),
                },
                timestamp,
            )
            .map(|_| ())
        }
        ConversationTransportProfileId::AgentCodexWorkspaceWrite => {
            super::stop_manual_relay_attempt(
                ManualRelayStopInput {
                    relay_attempt_id: attempt_id.to_string(),
                    requested_by: "conversation_transport_inner_cleanup".to_string(),
                },
                timestamp,
            )
            .map(|_| ())
        }
    }
}

pub(super) fn apply_command_profile(
    envelope: &super::ManualRelayEnvelope,
    command_plan: &mut ManualRelayCommandPlan,
    profile: &ConversationTransportCommandProfile,
) -> Result<(), String> {
    match profile {
        ConversationTransportCommandProfile::AgentWorkspaceWrite => {
            if envelope.target_binding.new_session {
                validate_gui_direct_new_session_target_and_command_plan(envelope, command_plan)
            } else {
                validate_gui_direct_target_and_command_plan(envelope, command_plan)
            }
        }
        ConversationTransportCommandProfile::SupervisorReadOnly(endpoint) => {
            append_supervisor_command_overrides(command_plan, endpoint)?;
            remove_supervisor_last_message_output(command_plan);
            validate_supervisor_command_plan(envelope, command_plan, endpoint)
        }
    }
}

fn profile_scope(
    profile_id: ConversationTransportProfileId,
    project_root: &str,
) -> (String, Vec<String>) {
    match profile_id {
        ConversationTransportProfileId::AgentCodexWorkspaceWrite => (
            "workspace-write".to_string(),
            vec![project_root.to_string()],
        ),
        ConversationTransportProfileId::SupervisorReadOnly => ("read-only".to_string(), Vec::new()),
    }
}

fn validate_start_input(input: &ConversationTransportStartInput) -> Result<(), String> {
    if input.conversation_id.trim().is_empty() {
        return Err("conversation_transport_conversation_id_required".to_string());
    }
    if input.turn_id.trim().is_empty() {
        return Err("conversation_transport_turn_id_required".to_string());
    }
    if input.original_user_text.trim().is_empty() {
        return Err("conversation_transport_prompt_required".to_string());
    }
    if input.target_project_root.trim().is_empty() || input.target_cwd.trim().is_empty() {
        return Err("conversation_transport_project_target_required".to_string());
    }
    if input.requested_by.trim().is_empty() {
        return Err("conversation_transport_requested_by_required".to_string());
    }
    if input.new_session {
        if input.target_session_id.is_some() {
            return Err("conversation_transport_new_session_must_not_bind_thread".to_string());
        }
    } else if input
        .target_session_id
        .as_deref()
        .is_none_or(|value| value.trim().is_empty())
    {
        return Err("conversation_transport_existing_session_requires_thread".to_string());
    }
    Ok(())
}

fn validate_supervisor_host_context(
    host: &SupervisorConversationHostContext,
) -> Result<(), String> {
    if host.project_id.trim().is_empty() || host.workflow_id.trim().is_empty() {
        return Err("conversation_transport_supervisor_binding_required".to_string());
    }
    if !host
        .run_id
        .starts_with(SUPERVISOR_CONVERSATION_RUN_ID_PREFIX)
        || host.run_id.len() == SUPERVISOR_CONVERSATION_RUN_ID_PREFIX.len()
    {
        return Err("conversation_transport_supervisor_run_id_invalid".to_string());
    }
    if !safe_absolute_path(&host.workflow_state_path) {
        return Err("conversation_transport_supervisor_workflow_state_path_invalid".to_string());
    }
    if host.max_active_workers == 0 || host.max_runtime_minutes <= 0 {
        return Err("conversation_transport_supervisor_quota_invalid".to_string());
    }
    Ok(())
}

impl SupervisorMcpEndpoint {
    fn from_host_context(host: SupervisorConversationHostContext) -> Result<Self, String> {
        let executable = std::env::current_exe()
            .map_err(|error| format!("conversation_transport_current_exe_unavailable:{error}"))?;
        if !executable.is_absolute() {
            return Err("conversation_transport_current_exe_not_absolute".to_string());
        }
        Ok(Self {
            executable: executable.display().to_string(),
            run_id: host.run_id,
            workflow_state_path: host.workflow_state_path,
            max_active_workers: host.max_active_workers,
            max_follow_ups_per_worker: host.max_follow_ups_per_worker,
            max_runtime_minutes: host.max_runtime_minutes,
            knowledge_open_relay: host.knowledge_open_relay,
        })
    }

    fn mcp_args(&self) -> Vec<String> {
        let mut args = vec![
            "__mcp_server".to_string(),
            "--role".to_string(),
            "supervisor_orchestrator".to_string(),
            "--run-id".to_string(),
            self.run_id.clone(),
            "--workflow-state-path".to_string(),
            self.workflow_state_path.clone(),
            "--max-active-workers".to_string(),
            self.max_active_workers.to_string(),
            "--max-follow-ups-per-worker".to_string(),
            self.max_follow_ups_per_worker.to_string(),
            "--max-runtime-minutes".to_string(),
            self.max_runtime_minutes.to_string(),
        ];
        self.knowledge_open_relay.append_mcp_args(&mut args);
        args
    }

    fn inline_config_overrides(&self) -> Result<Vec<String>, String> {
        let command = serde_json::to_string(&self.executable)
            .map_err(|error| format!("conversation_transport_mcp_command_encode_failed:{error}"))?;
        let args = serde_json::to_string(&self.mcp_args())
            .map_err(|error| format!("conversation_transport_mcp_args_encode_failed:{error}"))?;
        Ok(vec![
            "features.multi_agent=false".to_string(),
            format!("mcp_servers.supervisor_orchestrator.command={command}"),
            format!("mcp_servers.supervisor_orchestrator.args={args}"),
        ])
    }

    fn output_redaction_markers(&self) -> Vec<String> {
        let args = self.mcp_args();
        let mut markers = vec![
            self.executable.clone(),
            self.workflow_state_path.clone(),
            "mcp_servers.supervisor_orchestrator.args".to_string(),
            "--knowledge-open-relay-endpoint".to_string(),
            "--knowledge-open-relay-grant".to_string(),
        ];
        for pair in args.windows(2) {
            if matches!(
                pair[0].as_str(),
                "--knowledge-open-relay-endpoint" | "--knowledge-open-relay-grant"
            ) {
                markers.push(pair[1].clone());
            }
        }
        markers
    }
}

fn remove_supervisor_last_message_output(command_plan: &mut ManualRelayCommandPlan) {
    let mut index = 0;
    while index < command_plan.argv.len() {
        if command_plan.argv[index] == "--output-last-message" {
            command_plan.argv.remove(index);
            if index < command_plan.argv.len() {
                command_plan.argv.remove(index);
            }
        } else {
            index += 1;
        }
    }
    command_plan.last_message_path = "supervisor-memory-only".to_string();
}

fn append_supervisor_command_overrides(
    command_plan: &mut ManualRelayCommandPlan,
    endpoint: &SupervisorMcpEndpoint,
) -> Result<(), String> {
    let mut overrides = vec!["--ignore-user-config".to_string()];
    for override_value in endpoint.inline_config_overrides()? {
        overrides.push("-c".to_string());
        overrides.push(override_value);
    }
    let insert_at = command_plan
        .argv
        .iter()
        .position(|argument| argument == "resume")
        .unwrap_or(command_plan.argv.len());
    command_plan.argv.splice(insert_at..insert_at, overrides);
    command_plan.redacted_preview =
        "codex supervisor-read-only <stdin:workbench-managed> # host-owned-mcp".to_string();
    Ok(())
}

fn validate_supervisor_command_plan(
    envelope: &super::ManualRelayEnvelope,
    command_plan: &ManualRelayCommandPlan,
    endpoint: &SupervisorMcpEndpoint,
) -> Result<(), String> {
    super::verify_strict_run_paths(envelope)?;
    if envelope.target_binding.sandbox != "read-only" {
        return Err("conversation_transport_supervisor_sandbox_must_be_read_only".to_string());
    }
    if !envelope.target_binding.allowed_write_roots.is_empty() {
        return Err("conversation_transport_supervisor_write_roots_must_be_empty".to_string());
    }
    if envelope.target_binding.target_cwd_canonical
        != envelope.target_binding.project_root_canonical
    {
        return Err("conversation_transport_supervisor_cwd_must_equal_project_root".to_string());
    }
    if command_plan.program != "codex"
        || command_plan.prompt_in_command
        || command_plan.shell_invocation
    {
        return Err("conversation_transport_supervisor_command_shape_invalid".to_string());
    }
    if command_plan
        .argv
        .iter()
        .filter(|argument| argument.as_str() == "-C")
        .count()
        != 1
        || command_plan
            .argv
            .iter()
            .any(|argument| argument.starts_with("-C="))
        || !argv_contains_pair(
            &command_plan.argv,
            "-C",
            &envelope.target_binding.project_root_canonical,
        )
        || command_plan
            .argv
            .iter()
            .filter(|argument| argument.as_str() == "--sandbox")
            .count()
            != 1
        || command_plan
            .argv
            .iter()
            .any(|argument| argument.starts_with("--sandbox="))
        || !argv_contains_pair(&command_plan.argv, "--sandbox", "read-only")
    {
        return Err("conversation_transport_supervisor_command_binding_missing".to_string());
    }
    if command_plan.argv.iter().any(|argument| {
        argument == "--add-dir"
            || argument.starts_with("--add-dir=")
            || codex_approval_bypass_arg(argument)
            || argument.starts_with("--full-auto=")
            || argument == "--config"
            || argument.starts_with("--config=")
            || argument == "--mcp-config"
            || argument.starts_with("--mcp-config=")
            || argument == "--output-last-message"
            || argument.starts_with("--output-last-message=")
            || argument.contains('*')
            || argument.eq_ignore_ascii_case("all")
    }) {
        return Err("conversation_transport_supervisor_permission_expansion_forbidden".to_string());
    }
    if command_plan
        .argv
        .iter()
        .filter(|argument| argument.as_str() == "--ignore-user-config")
        .count()
        != 1
    {
        return Err("conversation_transport_supervisor_user_config_isolation_missing".to_string());
    }
    let expected_overrides = endpoint.inline_config_overrides()?;
    let actual_overrides = command_plan
        .argv
        .windows(2)
        .filter_map(|pair| (pair[0] == "-c").then(|| pair[1].clone()))
        .collect::<Vec<_>>();
    if actual_overrides != expected_overrides {
        return Err("conversation_transport_supervisor_mcp_config_not_host_owned".to_string());
    }
    if command_plan
        .argv
        .iter()
        .filter(|argument| argument.as_str() == "-c")
        .count()
        != expected_overrides.len()
    {
        return Err("conversation_transport_supervisor_mcp_config_not_exact".to_string());
    }
    Ok(())
}

fn argv_contains_pair(argv: &[String], flag: &str, value: &str) -> bool {
    argv.windows(2)
        .any(|pair| pair[0] == flag && pair[1] == value)
}

fn safe_absolute_path(value: &str) -> bool {
    let path = Path::new(value);
    path.is_absolute()
        && !path
            .components()
            .any(|component| matches!(component, Component::ParentDir))
}

fn conversation_attempts() -> &'static Mutex<BTreeMap<String, ConversationTransportAttemptRecord>> {
    static ATTEMPTS: OnceLock<Mutex<BTreeMap<String, ConversationTransportAttemptRecord>>> =
        OnceLock::new();
    ATTEMPTS.get_or_init(|| Mutex::new(BTreeMap::new()))
}

// A real `Mutex` poison would permanently contaminate the process-global
// attempt registry and make the test suite order-dependent.  This hook
// injects the exact record-removal failure after trusted manual cleanup, while
// leaving the stale record observable for explicit test-side cleanup.
#[cfg(test)]
thread_local! {
    static CONVERSATION_TRANSPORT_REMOVE_ATTEMPT_RECORD_FAILURE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

#[cfg(test)]
struct ConversationTransportRemoveAttemptRecordFailureGuard;

#[cfg(test)]
impl Drop for ConversationTransportRemoveAttemptRecordFailureGuard {
    fn drop(&mut self) {
        CONVERSATION_TRANSPORT_REMOVE_ATTEMPT_RECORD_FAILURE.with(|enabled| enabled.set(false));
    }
}

#[cfg(test)]
fn force_conversation_transport_remove_attempt_record_failure_for_test(
) -> ConversationTransportRemoveAttemptRecordFailureGuard {
    CONVERSATION_TRANSPORT_REMOVE_ATTEMPT_RECORD_FAILURE.with(|enabled| enabled.set(true));
    ConversationTransportRemoveAttemptRecordFailureGuard
}

fn register_attempt_record(
    attempt_id: &str,
    record: ConversationTransportAttemptRecord,
) -> Result<(), String> {
    let mut attempts = conversation_attempts()
        .lock()
        .map_err(|_| "conversation_transport_attempt_registry_poisoned".to_string())?;
    if attempts.contains_key(attempt_id) {
        return Err("conversation_transport_attempt_id_reused".to_string());
    }
    attempts.insert(attempt_id.to_string(), record);
    Ok(())
}

fn conversation_attempt_record(
    input: &ConversationTransportAttemptInput,
) -> Result<ConversationTransportAttemptRecord, String> {
    if input.requested_by.trim().is_empty() {
        return Err("conversation_transport_requested_by_required".to_string());
    }
    let attempts = conversation_attempts()
        .lock()
        .map_err(|_| "conversation_transport_attempt_registry_poisoned".to_string())?;
    let Some(record) = attempts.get(&input.attempt_id) else {
        return Err("conversation_transport_attempt_not_running".to_string());
    };
    if record.conversation_id != input.conversation_id || record.turn_id != input.turn_id {
        return Err("conversation_transport_attempt_binding_mismatch".to_string());
    }
    Ok(record.clone())
}

fn remove_attempt_record(attempt_id: &str) -> Result<(), String> {
    #[cfg(test)]
    if CONVERSATION_TRANSPORT_REMOVE_ATTEMPT_RECORD_FAILURE.with(std::cell::Cell::get) {
        return Err("conversation_transport_attempt_registry_poisoned".to_string());
    }
    conversation_attempts()
        .lock()
        .map_err(|_| "conversation_transport_attempt_registry_poisoned".to_string())?
        .remove(attempt_id);
    Ok(())
}

/// A direct trusted supervisor stop/poll can retain its initial error after a
/// reverse cleanup has already settled every manual relay resource.  The
/// safe-only marker is the authoritative fail-closed signal: only when it is
/// absent can the paired transport record be removed.  If the marker remains
/// (or cannot be inspected), retain the record for the next trusted retry.
fn remove_settled_supervisor_attempt_record_after_trusted_error(
    record: &ConversationTransportAttemptRecord,
    attempt_id: &str,
) -> Result<(), String> {
    if record.profile_id != ConversationTransportProfileId::SupervisorReadOnly
        || super::reject_raw_safe_only_manual_relay_attempt(attempt_id).is_err()
    {
        return Ok(());
    }
    remove_attempt_record(attempt_id)
        .map_err(|_| "conversation_transport_terminal_cleanup_record_removal_failed".to_string())
}

#[cfg(test)]
pub(crate) fn install_supervisor_attempt_record_for_outer_cleanup_test(
    attempt_id: &str,
) -> Result<(), String> {
    register_attempt_record(
        attempt_id,
        ConversationTransportAttemptRecord {
            profile_id: ConversationTransportProfileId::SupervisorReadOnly,
            conversation_id: "conversation:outer-cleanup-fixture".to_string(),
            turn_id: "turn:outer-cleanup-fixture".to_string(),
            target_session_id: None,
        },
    )
}

#[cfg(test)]
pub(crate) fn supervisor_attempt_record_is_cleared_for_outer_cleanup_test(
    attempt_id: &str,
) -> bool {
    conversation_attempts()
        .lock()
        .map(|attempts| !attempts.contains_key(attempt_id))
        .unwrap_or(false)
}

fn receipt_for_record(
    record: &ConversationTransportAttemptRecord,
    receipt: &ManualRelayReceipt,
) -> ConversationTransportReceipt {
    let lifecycle = lifecycle_for_manual_receipt(receipt);
    let thread_id = receipt.thread_event_summary.thread_id.clone().or_else(|| {
        (!receipt.target.new_session)
            .then(|| record.target_session_id.clone())
            .flatten()
    });
    let assistant_reply = if let Some(text) = receipt.assistant_message_text.clone() {
        ConversationAssistantReplyReceipt {
            status: "available".to_string(),
            text: Some(text),
        }
    } else if lifecycle == "running" || lifecycle == "starting" {
        ConversationAssistantReplyReceipt {
            status: "pending".to_string(),
            text: None,
        }
    } else if lifecycle == "cleanup_pending" {
        ConversationAssistantReplyReceipt {
            status: "unavailable".to_string(),
            text: None,
        }
    } else if lifecycle == "failed" {
        ConversationAssistantReplyReceipt {
            status: "failed".to_string(),
            text: None,
        }
    } else if lifecycle == "stopped" {
        ConversationAssistantReplyReceipt {
            status: "stopped".to_string(),
            text: None,
        }
    } else {
        ConversationAssistantReplyReceipt {
            status: "unavailable".to_string(),
            text: None,
        }
    };
    ConversationTransportReceipt {
        profile_id: record.profile_id.as_str().to_string(),
        conversation_id: record.conversation_id.clone(),
        thread_id,
        turn_id: record.turn_id.clone(),
        lifecycle: lifecycle.clone(),
        transport: ConversationTransportLayerReceipt {
            status: lifecycle,
            attempt_id: receipt.relay_attempt_id.clone(),
            started_at: receipt.started_at.clone(),
            ended_at: receipt.ended_at.clone(),
        },
        assistant_reply,
        tool_action: ConversationLayerReceipt {
            status: "not_requested".to_string(),
            summary: None,
        },
        read_model_projection: ConversationLayerReceipt {
            status: "not_started".to_string(),
            summary: None,
        },
        canonical_mirror: ConversationLayerReceipt {
            status: "not_started".to_string(),
            summary: None,
        },
    }
}

fn lifecycle_for_manual_receipt(receipt: &ManualRelayReceipt) -> String {
    if receipt.status == "supervisor_relay_cleanup_pending" {
        "cleanup_pending".to_string()
    } else if receipt.status == "running" {
        "running".to_string()
    } else if receipt.status == "stopped_by_user" || receipt.killed_by_user {
        "stopped".to_string()
    } else if receipt.timed_out
        || receipt.thread_event_summary.turn_failed
        || receipt.status.contains("failed")
    {
        "failed".to_string()
    } else if receipt.status.starts_with("completed") {
        "completed".to_string()
    } else {
        "failed".to_string()
    }
}

fn is_terminal_lifecycle(lifecycle: &str) -> bool {
    matches!(lifecycle, "completed" | "failed" | "stopped")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::hash::sha256_hex;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TRANSPORT_CLEANUP_TEST_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    fn cleanup_test_attempt_id(label: &str) -> String {
        format!(
            "conversation-transport-{label}:{}:{}",
            std::process::id(),
            TRANSPORT_CLEANUP_TEST_SEQUENCE.fetch_add(1, Ordering::Relaxed),
        )
    }

    fn fixture_envelope(
        sandbox: &str,
        allowed_write_roots: Vec<String>,
    ) -> super::super::ManualRelayEnvelope {
        let root = std::env::current_dir()
            .expect("current directory")
            .canonicalize()
            .expect("canonical directory")
            .display()
            .to_string();
        let input = ManualRelayPreviewInput {
            original_user_text: "fixture message".to_string(),
            target_project_root: root.clone(),
            target_cwd: root,
            target_session_id: None,
            new_session: true,
            sandbox: sandbox.to_string(),
            allowed_write_roots,
            requested_by: "fixture".to_string(),
        };
        preview_manual_relay(input, "2026-07-23T00:00:00Z").envelope
    }

    fn fixture_endpoint() -> SupervisorMcpEndpoint {
        SupervisorMcpEndpoint {
            executable: "/tmp/workbench".to_string(),
            run_id: format!("{SUPERVISOR_CONVERSATION_RUN_ID_PREFIX}fixture"),
            workflow_state_path: "/tmp/workflow-state.json".to_string(),
            max_active_workers: 1,
            max_follow_ups_per_worker: 0,
            max_runtime_minutes: 1,
            knowledge_open_relay:
                crate::knowledge_open_relay::KnowledgeOpenRelayMcpConfig::from_mcp_arguments(
                    "/tmp/syn-knowledge-open-fixture/relay.sock".to_string(),
                    "a".repeat(64),
                    "turn:fixture".to_string(),
                    "project:fixture".to_string(),
                )
                .expect("fixed host-only relay fixture"),
        }
    }

    #[test]
    fn supervisor_mcp_args_are_exact_host_owned_value_pairs() {
        let endpoint = fixture_endpoint();
        let mut expected = vec![
            "__mcp_server",
            "--role",
            "supervisor_orchestrator",
            "--run-id",
            "supervisor-conversation:fixture",
            "--workflow-state-path",
            "/tmp/workflow-state.json",
            "--max-active-workers",
            "1",
            "--max-follow-ups-per-worker",
            "0",
            "--max-runtime-minutes",
            "1",
            "--knowledge-open-relay-endpoint",
            "/tmp/syn-knowledge-open-fixture/relay.sock",
            "--knowledge-open-relay-grant",
        ]
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();
        expected.push("a".repeat(64));
        expected.extend([
            "--knowledge-open-relay-turn-id".to_string(),
            "turn:fixture".to_string(),
            "--knowledge-open-relay-project-id".to_string(),
            "project:fixture".to_string(),
        ]);
        assert_eq!(
            endpoint.mcp_args(),
            expected,
            "the supervisor endpoint must not accept omitted, repeated, or caller-selected MCP flags"
        );
    }

    #[test]
    fn supervisor_profile_plan_is_read_only_and_has_only_host_owned_mcp_overrides() {
        let envelope = fixture_envelope("read-only", Vec::new());
        let mut plan = ManualRelayCommandPlan {
            program: "codex".to_string(),
            argv: vec![
                "exec".to_string(),
                "-C".to_string(),
                envelope.target_binding.project_root_canonical.clone(),
                "--sandbox".to_string(),
                "read-only".to_string(),
                "--json".to_string(),
                "--output-last-message".to_string(),
                "/tmp/last-message.txt".to_string(),
            ],
            stdin_prompt_ref: "fixture".to_string(),
            stdin_prompt_sha256: sha256_hex("fixture"),
            prompt_in_command: false,
            shell_invocation: false,
            redacted_preview: "fixture".to_string(),
            last_message_path: "/tmp/last-message.txt".to_string(),
        };
        let endpoint = fixture_endpoint();
        apply_command_profile(
            &envelope,
            &mut plan,
            &ConversationTransportCommandProfile::SupervisorReadOnly(endpoint),
        )
        .expect("supervisor profile plan");
        assert!(argv_contains_pair(&plan.argv, "--sandbox", "read-only"));
        assert!(!plan.argv.iter().any(|value| value == "--add-dir"));
        assert!(plan
            .argv
            .iter()
            .any(|value| value == "--ignore-user-config"));
        assert!(plan
            .argv
            .iter()
            .any(|value| value == "features.multi_agent=false"));
        assert!(
            !plan
                .argv
                .iter()
                .any(|value| value == "--output-last-message"),
            "the supervisor profile has no child-writable last-message sink"
        );
        assert_eq!(plan.last_message_path, "supervisor-memory-only");
    }

    #[test]
    fn supervisor_profile_rejects_any_expanded_write_root_or_bypass() {
        let root = std::env::current_dir()
            .expect("current directory")
            .canonicalize()
            .expect("canonical directory")
            .display()
            .to_string();
        let envelope = fixture_envelope("workspace-write", vec![root.clone()]);
        let endpoint = fixture_endpoint();
        let mut plan = ManualRelayCommandPlan {
            program: "codex".to_string(),
            argv: vec![
                "exec".to_string(),
                "-C".to_string(),
                root,
                "--sandbox".to_string(),
                "workspace-write".to_string(),
                "--add-dir".to_string(),
                "/tmp".to_string(),
                "--full-auto".to_string(),
            ],
            stdin_prompt_ref: "fixture".to_string(),
            stdin_prompt_sha256: sha256_hex("fixture"),
            prompt_in_command: false,
            shell_invocation: false,
            redacted_preview: "fixture".to_string(),
            last_message_path: "/tmp/last-message.txt".to_string(),
        };
        assert!(apply_command_profile(
            &envelope,
            &mut plan,
            &ConversationTransportCommandProfile::SupervisorReadOnly(endpoint),
        )
        .is_err());
    }

    #[test]
    fn supervisor_profile_rejects_equals_form_permission_expansion() {
        let envelope = fixture_envelope("read-only", Vec::new());
        let root = envelope.target_binding.project_root_canonical.clone();
        let endpoint = fixture_endpoint();
        let mut plan = ManualRelayCommandPlan {
            program: "codex".to_string(),
            argv: vec![
                "exec".to_string(),
                "-C".to_string(),
                root,
                "--sandbox".to_string(),
                "read-only".to_string(),
                "--sandbox=workspace-write".to_string(),
                "--add-dir=/tmp".to_string(),
                "--full-auto=1".to_string(),
            ],
            stdin_prompt_ref: "fixture".to_string(),
            stdin_prompt_sha256: sha256_hex("fixture"),
            prompt_in_command: false,
            shell_invocation: false,
            redacted_preview: "fixture".to_string(),
            last_message_path: "/tmp/last-message.txt".to_string(),
        };
        assert!(apply_command_profile(
            &envelope,
            &mut plan,
            &ConversationTransportCommandProfile::SupervisorReadOnly(endpoint),
        )
        .is_err());
    }

    #[test]
    fn safe_receipt_omits_raw_command_and_process_material() {
        let envelope = fixture_envelope("read-only", Vec::new());
        let raw_receipt = super::super::fixture_receipt(
            "manual-relay-attempt:fixture",
            "confirmation:fixture",
            "completed_fixture",
            &envelope,
            ManualRelayCommandPlan {
                program: "codex".to_string(),
                argv: vec![
                    "--token=must-not-reach-ui".to_string(),
                    "/private/host/path".to_string(),
                    "--knowledge-open-relay-endpoint".to_string(),
                    "/private/syn-relay/relay.sock".to_string(),
                    "--knowledge-open-relay-grant".to_string(),
                    "grant-must-not-reach-ui".to_string(),
                ],
                stdin_prompt_ref: "fixture".to_string(),
                stdin_prompt_sha256: sha256_hex("fixture"),
                prompt_in_command: false,
                shell_invocation: false,
                redacted_preview: "fixture".to_string(),
                last_message_path: "/private/host/last-message.txt".to_string(),
            },
            "2026-07-23T00:00:00Z",
            false,
            false,
            None,
        );
        let receipt = receipt_for_record(
            &ConversationTransportAttemptRecord {
                profile_id: ConversationTransportProfileId::SupervisorReadOnly,
                conversation_id: "conversation-fixture".to_string(),
                turn_id: "turn-fixture".to_string(),
                target_session_id: None,
            },
            &raw_receipt,
        );
        let serialized = serde_json::to_string(&receipt).expect("serialize safe receipt");
        for forbidden in [
            "argv",
            "stderr",
            "must-not-reach-ui",
            "/private/host/path",
            "last-message.txt",
            "syn-relay",
            "grant-must-not-reach-ui",
        ] {
            assert!(
                !serialized.contains(forbidden),
                "safe receipt must omit {forbidden}: {serialized}"
            );
        }

        let mut cleanup_pending = raw_receipt;
        cleanup_pending.status = "supervisor_relay_cleanup_pending".to_string();
        cleanup_pending.prompt_sent = false;
        let cleanup_pending_receipt = receipt_for_record(
            &ConversationTransportAttemptRecord {
                profile_id: ConversationTransportProfileId::SupervisorReadOnly,
                conversation_id: "conversation-fixture".to_string(),
                turn_id: "turn-fixture".to_string(),
                target_session_id: None,
            },
            &cleanup_pending,
        );
        assert_eq!(cleanup_pending_receipt.lifecycle, "cleanup_pending");
        assert!(!is_terminal_lifecycle(&cleanup_pending_receipt.lifecycle));
        assert_eq!(
            cleanup_pending_receipt.assistant_reply.status,
            "unavailable"
        );
        assert!(cleanup_pending_receipt.assistant_reply.text.is_none());
    }

    #[test]
    fn outer_safe_attempt_registration_failure_abort_clears_inner_and_manual_safe_only_state() {
        let _manual_relay_guard = crate::manual_relay::manual_relay_test_guard_for_shared_state();
        let attempt_id = format!(
            "conversation-transport-outer-cleanup:{}",
            std::process::id()
        );
        crate::manual_relay::install_safe_only_fixture_attempt_for_test(&attempt_id)
            .expect("fixture installs an already-started safe-only attempt");
        register_attempt_record(
            &attempt_id,
            ConversationTransportAttemptRecord {
                profile_id: ConversationTransportProfileId::SupervisorReadOnly,
                conversation_id: "conversation:fixture".to_string(),
                turn_id: "turn:fixture".to_string(),
                target_session_id: None,
            },
        )
        .expect("fixture installs the inner transport record");

        abort_supervisor_conversation_transport_attempt(&attempt_id, "2026-07-23T00:00:00Z")
            .expect("outer registration cleanup must settle a safe-only fixture");

        assert!(crate::manual_relay::safe_only_fixture_attempt_is_cleared_for_test(&attempt_id));
        assert!(
            !conversation_attempts()
                .lock()
                .expect("inner registry lock")
                .contains_key(&attempt_id),
            "outer cleanup must remove the inner transport record"
        );
    }

    #[test]
    fn supervisor_outer_abort_accepts_settled_manual_cleanup_when_inner_record_removal_is_unavailable(
    ) {
        let _manual_relay_guard = crate::manual_relay::manual_relay_test_guard_for_shared_state();
        let attempt_id = cleanup_test_attempt_id("outer-abort-inner-removal-unavailable");
        let fixture =
            crate::manual_relay::install_safe_only_supervisor_cleanup_fixture_for_test(&attempt_id)
                .expect("fixture installs child, durable registration, and bounded capture");
        fixture
            .wait_until_child_ready()
            .expect("fixture child must run before trusted outer abort");
        register_attempt_record(
            &attempt_id,
            ConversationTransportAttemptRecord {
                profile_id: ConversationTransportProfileId::SupervisorReadOnly,
                conversation_id: "conversation:outer-abort-inner-removal-unavailable".to_string(),
                turn_id: "turn:outer-abort-inner-removal-unavailable".to_string(),
                target_session_id: None,
            },
        )
        .expect("fixture installs the transport record");

        let removal_failure = force_conversation_transport_remove_attempt_record_failure_for_test();
        abort_supervisor_conversation_transport_attempt(&attempt_id, "2026-07-23T00:00:01Z")
            .expect("a settled manual cleanup must let the outer host release its route even when only inner bookkeeping removal fails");
        std::thread::sleep(std::time::Duration::from_millis(1200));
        assert!(
            fixture
                .is_fully_cleared()
                .expect("fixture cleanup state remains readable"),
            "the trusted abort must close child, durable registration, capture, active state, marker, and confirmation before inner bookkeeping is tolerated as stale"
        );
        assert!(
            crate::manual_relay::reject_raw_safe_only_manual_relay_attempt(&attempt_id).is_ok(),
            "the marker must be absent before an inner-record removal failure is tolerated"
        );
        assert_eq!(
            crate::manual_relay::poll_manual_relay_attempt(
                crate::manual_relay::ManualRelayPollInput {
                    relay_attempt_id: attempt_id.clone(),
                    requested_by: "raw-after-settled-outer-abort".to_string(),
                },
                "2026-07-23T00:00:02Z",
            )
            .expect_err("a stale inner record must not resurrect or expose a manual relay attempt"),
            "manual_relay_attempt_not_running"
        );
        drop(removal_failure);
        remove_attempt_record(&attempt_id)
            .expect("test-side cleanup removes only the deliberately stale inner record");
    }

    #[test]
    fn supervisor_stop_error_after_settled_reverse_cleanup_removes_transport_record() {
        let _manual_relay_guard = crate::manual_relay::manual_relay_test_guard_for_shared_state();
        let attempt_id = cleanup_test_attempt_id("settled-stop-cleanup");
        let fixture =
            crate::manual_relay::install_safe_only_supervisor_cleanup_fixture_for_test(&attempt_id)
                .expect("fixture installs child, durable registration, and bounded capture");
        fixture
            .wait_until_child_ready()
            .expect("fixture background child must run before cleanup");
        register_attempt_record(
            &attempt_id,
            ConversationTransportAttemptRecord {
                profile_id: ConversationTransportProfileId::SupervisorReadOnly,
                conversation_id: "conversation:settled-stop-cleanup".to_string(),
                turn_id: "turn:settled-stop-cleanup".to_string(),
                target_session_id: None,
            },
        )
        .expect("fixture installs the transport record");

        let error = {
            let _first_stop_failure =
                crate::manual_relay::force_manual_relay_child_stop_test_failures_for_test(1);
            stop_conversation_transport_attempt(
                ConversationTransportAttemptInput {
                    conversation_id: "conversation:settled-stop-cleanup".to_string(),
                    turn_id: "turn:settled-stop-cleanup".to_string(),
                    attempt_id: attempt_id.clone(),
                    requested_by: "trusted-transport-stop".to_string(),
                },
                "2026-07-23T00:00:01Z",
            )
            .expect_err("the direct stop preserves its first failed wait result")
        };
        assert_eq!(error, "supervisor_relay_cleanup_failed");
        std::thread::sleep(std::time::Duration::from_millis(1200));
        assert!(
            fixture
                .is_fully_cleared()
                .expect("fixture cleanup state remains readable"),
            "the trusted reverse cleanup settled every manual relay resource"
        );
        assert!(
            supervisor_attempt_record_is_cleared_for_outer_cleanup_test(&attempt_id),
            "a settled trusted cleanup must not leave a stale transport record"
        );
        assert_eq!(
            stop_conversation_transport_attempt(
                ConversationTransportAttemptInput {
                    conversation_id: "conversation:settled-stop-cleanup".to_string(),
                    turn_id: "turn:settled-stop-cleanup".to_string(),
                    attempt_id,
                    requested_by: "trusted-transport-stop-retry".to_string(),
                },
                "2026-07-23T00:00:02Z",
            )
            .expect_err("the settled attempt no longer has a transport record"),
            "conversation_transport_attempt_not_running"
        );
    }

    #[test]
    fn supervisor_stop_error_keeps_transport_record_while_safe_only_cleanup_is_unsettled() {
        let _manual_relay_guard = crate::manual_relay::manual_relay_test_guard_for_shared_state();
        let attempt_id = cleanup_test_attempt_id("unsettled-stop-cleanup");
        let fixture =
            crate::manual_relay::install_safe_only_supervisor_cleanup_fixture_for_test(&attempt_id)
                .expect("fixture installs child, durable registration, and bounded capture");
        fixture
            .wait_until_child_ready()
            .expect("fixture background child must run before cleanup");
        register_attempt_record(
            &attempt_id,
            ConversationTransportAttemptRecord {
                profile_id: ConversationTransportProfileId::SupervisorReadOnly,
                conversation_id: "conversation:unsettled-stop-cleanup".to_string(),
                turn_id: "turn:unsettled-stop-cleanup".to_string(),
                target_session_id: None,
            },
        )
        .expect("fixture installs the transport record");

        let error = {
            let _persistent_stop_failures =
                crate::manual_relay::force_manual_relay_child_stop_test_failures_for_test(3);
            stop_conversation_transport_attempt(
                ConversationTransportAttemptInput {
                    conversation_id: "conversation:unsettled-stop-cleanup".to_string(),
                    turn_id: "turn:unsettled-stop-cleanup".to_string(),
                    attempt_id: attempt_id.clone(),
                    requested_by: "trusted-transport-stop".to_string(),
                },
                "2026-07-23T00:00:01Z",
            )
            .expect_err("a persistent cleanup failure must leave the trusted record available")
        };
        assert_eq!(error, "supervisor_relay_cleanup_failed");
        assert!(
            fixture
                .is_retained_for_trusted_retry()
                .expect("fixture retained state remains readable"),
            "the safe-only marker and every cleanup handle remain for a trusted retry"
        );
        assert!(
            !supervisor_attempt_record_is_cleared_for_outer_cleanup_test(&attempt_id),
            "the inner transport record must remain paired with unsettled protected state"
        );

        abort_supervisor_conversation_transport_attempt(&attempt_id, "2026-07-23T00:00:02Z")
            .expect("the authoritative trusted retry settles the retained attempt");
        std::thread::sleep(std::time::Duration::from_millis(1200));
        assert!(
            fixture
                .is_fully_cleared()
                .expect("fixture cleanup state remains readable"),
            "the trusted retry clears every retained manual relay resource"
        );
        assert!(
            supervisor_attempt_record_is_cleared_for_outer_cleanup_test(&attempt_id),
            "the authoritative trusted retry removes its paired transport record"
        );
    }

    #[test]
    fn supervisor_run_identity_is_deterministic_and_does_not_embed_source_ids() {
        let first =
            supervisor_run_id_for("conversation:private-id", "turn:private-id").expect("run id");
        let second =
            supervisor_run_id_for("conversation:private-id", "turn:private-id").expect("run id");
        assert_eq!(first, second);
        assert!(first.starts_with(SUPERVISOR_CONVERSATION_RUN_ID_PREFIX));
        assert!(!first.contains("conversation:private-id"));
        assert!(!first.contains("turn:private-id"));
    }
}
