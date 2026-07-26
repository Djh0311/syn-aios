//! Durable, host-established binding for one supervisor conversation turn.
//!
//! This module intentionally has no file or process operations.  The existing
//! supervisor orchestrator persists this value in its DB-primary session record
//! and JSON compatibility projection; MCP callers may only validate and consume
//! that host-established record.

use super::capability_registry::{
    self, CapabilityAccess, CapabilityAuthorizationError, PROFILE_SUPERVISOR_READ_ONLY,
    ROLE_PROJECT_SUPERVISOR,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use std::path::{Component, Path, PathBuf};

pub(crate) const SUPERVISOR_CONVERSATION_RUN_ID_PREFIX: &str = "supervisor-conversation:";
/// The shared supervisor profile is intentionally short-lived.  This value is
/// persisted with the host-created binding so an MCP process argument cannot
/// extend an already-authorized conversation turn.
pub(crate) const SUPERVISOR_CONVERSATION_MAX_RUNTIME_MINUTES: i64 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ConversationTurnLifecycle {
    Inactive,
    Starting,
    Active,
    Completed,
    Failed,
    Stopped,
}

impl Default for ConversationTurnLifecycle {
    fn default() -> Self {
        Self::Inactive
    }
}

impl ConversationTurnLifecycle {
    pub(crate) const fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Stopped)
    }
}

/// The only UI-visible settlement fact kept with a trusted turn binding.  It
/// never carries tool arguments, output, proposal content, or diagnostics.
/// The transport can use it after a host-observed tool call without inferring
/// success from a natural-language reply.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ConversationCapabilityOutcome {
    NotRequested,
    Succeeded,
    Failed,
}

impl Default for ConversationCapabilityOutcome {
    fn default() -> Self {
        Self::NotRequested
    }
}

/// Fields supplied only by the host when it starts a `supervisor-read-only`
/// conversation turn.  No frontend or MCP payload has a way to set a sandbox,
/// write root, role, profile, or capability list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SupervisorConversationTurnInput {
    pub(crate) project_id: String,
    pub(crate) project_root: String,
    pub(crate) workflow_id: String,
    pub(crate) turn_id: String,
    pub(crate) transport_attempt: u32,
    pub(crate) run_id: String,
    /// Host-observed user text used by the existing proposal writer.  It is
    /// never sourced from MCP tool arguments or a client-side capability claim.
    pub(crate) user_message_snapshot: String,
    pub(crate) created_at_ms: i64,
    pub(crate) max_runtime_minutes: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ConversationTurnBindingExpectation<'a> {
    pub(crate) project_id: &'a str,
    pub(crate) project_root: &'a str,
    pub(crate) workflow_id: &'a str,
    pub(crate) run_id: &'a str,
}

/// This is intentionally serializable as one field on the pre-existing
/// `SupervisorSession`, rather than a new sidecar or storage schema source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ConversationTurnBinding {
    #[serde(default)]
    pub(crate) profile_id: String,
    #[serde(default)]
    pub(crate) role: String,
    #[serde(default)]
    pub(crate) project_id: String,
    #[serde(default)]
    pub(crate) project_root: String,
    #[serde(default)]
    pub(crate) workflow_id: String,
    #[serde(default)]
    pub(crate) turn_id: String,
    #[serde(default)]
    pub(crate) transport_attempt: u32,
    #[serde(default)]
    pub(crate) run_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) thread_id: Option<String>,
    #[serde(default)]
    pub(crate) allowed_capabilities: Vec<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub(crate) capability_outcomes: BTreeMap<String, ConversationCapabilityOutcome>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub(crate) capability_audit_outcomes: BTreeMap<String, ConversationCapabilityOutcome>,
    #[serde(default)]
    pub(crate) lifecycle: ConversationTurnLifecycle,
    #[serde(default)]
    pub(crate) created_at_ms: i64,
    #[serde(default)]
    pub(crate) updated_at_ms: i64,
    #[serde(default)]
    pub(crate) max_runtime_minutes: i64,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub(crate) user_message_snapshot: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ConversationTurnBindingError {
    MissingField(&'static str),
    InvalidProjectRoot,
    ProjectIdMismatch,
    InvalidRunIdentity,
    InvalidTimestamp,
    InvalidRuntimeLimit,
    RuntimeExpired,
    InvalidLifecycleTransition {
        from: ConversationTurnLifecycle,
        to: ConversationTurnLifecycle,
    },
    InactiveLifecycle(ConversationTurnLifecycle),
    ThreadUnbound,
    ContextMismatch(&'static str),
    Capability(CapabilityAuthorizationError),
}

impl fmt::Display for ConversationTurnBindingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingField(field) => {
                write!(formatter, "主管对话可信绑定缺少 {field}，已拒绝。")
            }
            Self::InvalidProjectRoot => {
                write!(formatter, "主管对话项目根不是规范化绝对路径，已拒绝。")
            }
            Self::ProjectIdMismatch => {
                write!(formatter, "主管对话 project_id 与项目根不一致，已拒绝。")
            }
            Self::InvalidRunIdentity => write!(
                formatter,
                "主管对话 run identity 不属于受控宿主回合，已拒绝。"
            ),
            Self::InvalidTimestamp => write!(formatter, "主管对话可信绑定时间戳不完整，已拒绝。"),
            Self::InvalidRuntimeLimit => {
                write!(formatter, "主管对话宿主运行时限无效，已拒绝。")
            }
            Self::RuntimeExpired => {
                write!(formatter, "主管对话可信绑定已超过宿主运行时限，已拒绝。")
            }
            Self::InvalidLifecycleTransition { .. } => {
                write!(formatter, "主管对话可信绑定生命周期转换不合法，已拒绝。")
            }
            Self::InactiveLifecycle(_) => {
                write!(
                    formatter,
                    "主管对话回合当前不处于 active 生命周期，已拒绝。"
                )
            }
            Self::ThreadUnbound => {
                write!(formatter, "主管对话回合尚未完成可信 thread 绑定，已拒绝。")
            }
            Self::ContextMismatch(_) => {
                write!(
                    formatter,
                    "主管对话可信绑定与当前 project/workflow/run 不一致，已拒绝。"
                )
            }
            Self::Capability(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for ConversationTurnBindingError {}

impl From<CapabilityAuthorizationError> for ConversationTurnBindingError {
    fn from(error: CapabilityAuthorizationError) -> Self {
        Self::Capability(error)
    }
}

impl ConversationTurnBinding {
    /// Construct the only new supervisor conversation binding.  Profile, role,
    /// and capability set are frozen in this function, rather than accepted
    /// from an MCP request or frontend payload.
    pub(crate) fn establish_supervisor_read_only(
        input: SupervisorConversationTurnInput,
    ) -> Result<Self, ConversationTurnBindingError> {
        require_exact_nonempty(&input.project_id, "project_id")?;
        require_exact_nonempty(&input.workflow_id, "workflow_id")?;
        require_exact_nonempty(&input.turn_id, "turn_id")?;
        require_exact_nonempty(&input.run_id, "run_id")?;
        require_nonempty_text(&input.user_message_snapshot, "user_message_snapshot")?;
        if input.transport_attempt == 0 {
            return Err(ConversationTurnBindingError::MissingField(
                "transport_attempt",
            ));
        }
        if input.created_at_ms <= 0 {
            return Err(ConversationTurnBindingError::InvalidTimestamp);
        }
        if input.max_runtime_minutes != SUPERVISOR_CONVERSATION_MAX_RUNTIME_MINUTES {
            return Err(ConversationTurnBindingError::InvalidRuntimeLimit);
        }
        if !input
            .run_id
            .starts_with(SUPERVISOR_CONVERSATION_RUN_ID_PREFIX)
            || input.run_id == SUPERVISOR_CONVERSATION_RUN_ID_PREFIX
        {
            return Err(ConversationTurnBindingError::InvalidRunIdentity);
        }

        let project_root = normalize_project_root(&input.project_root)?;
        if input.project_id != crate::project_id(&project_root) {
            return Err(ConversationTurnBindingError::ProjectIdMismatch);
        }
        let access = CapabilityAccess::new(PROFILE_SUPERVISOR_READ_ONLY, ROLE_PROJECT_SUPERVISOR);
        let allowed_capabilities = capability_registry::frozen_capability_names(access)?;
        // Guard the registry result itself so a future empty/default registry
        // cannot silently produce a valid supervisor binding.
        capability_registry::validate_exact_capability_set(access, &allowed_capabilities)?;

        Ok(Self {
            profile_id: PROFILE_SUPERVISOR_READ_ONLY.to_string(),
            role: ROLE_PROJECT_SUPERVISOR.to_string(),
            project_id: input.project_id,
            project_root,
            workflow_id: input.workflow_id,
            turn_id: input.turn_id,
            transport_attempt: input.transport_attempt,
            run_id: input.run_id,
            thread_id: None,
            allowed_capabilities,
            capability_outcomes: BTreeMap::new(),
            capability_audit_outcomes: BTreeMap::new(),
            lifecycle: ConversationTurnLifecycle::Starting,
            created_at_ms: input.created_at_ms,
            updated_at_ms: input.created_at_ms,
            max_runtime_minutes: input.max_runtime_minutes,
            user_message_snapshot: input.user_message_snapshot,
        })
    }

    /// The host may call this only after observing `thread.started` (or after
    /// confirming an existing thread).  MCP cannot manufacture this binding.
    pub(crate) fn activate_with_host_observed_thread(
        &mut self,
        thread_id: &str,
        updated_at_ms: i64,
    ) -> Result<(), ConversationTurnBindingError> {
        self.validate_host_fields()?;
        require_exact_nonempty(thread_id, "thread_id")?;
        if self.lifecycle != ConversationTurnLifecycle::Starting {
            return Err(ConversationTurnBindingError::InvalidLifecycleTransition {
                from: self.lifecycle,
                to: ConversationTurnLifecycle::Active,
            });
        }
        if updated_at_ms < self.updated_at_ms || updated_at_ms <= 0 {
            return Err(ConversationTurnBindingError::InvalidTimestamp);
        }
        self.thread_id = Some(thread_id.to_string());
        self.lifecycle = ConversationTurnLifecycle::Active;
        self.updated_at_ms = updated_at_ms;
        Ok(())
    }

    pub(crate) fn mark_terminal(
        &mut self,
        lifecycle: ConversationTurnLifecycle,
        updated_at_ms: i64,
    ) -> Result<(), ConversationTurnBindingError> {
        self.validate_host_fields()?;
        if !lifecycle.is_terminal() || self.lifecycle.is_terminal() {
            return Err(ConversationTurnBindingError::InvalidLifecycleTransition {
                from: self.lifecycle,
                to: lifecycle,
            });
        }
        if updated_at_ms < self.updated_at_ms || updated_at_ms <= 0 {
            return Err(ConversationTurnBindingError::InvalidTimestamp);
        }
        self.lifecycle = lifecycle;
        self.updated_at_ms = updated_at_ms;
        Ok(())
    }

    /// Validate the durable binding immediately before dispatching a capability.
    /// The expectation is host-derived from the already-selected supervisor run;
    /// no client parameter is used for project/workflow/run authorization.
    pub(crate) fn validate_for_capability(
        &self,
        expected: ConversationTurnBindingExpectation<'_>,
        capability_name: &str,
    ) -> Result<(), ConversationTurnBindingError> {
        self.validate_host_fields()?;
        require_exact_nonempty(expected.project_id, "expected_project_id")?;
        require_exact_nonempty(expected.workflow_id, "expected_workflow_id")?;
        require_exact_nonempty(expected.run_id, "expected_run_id")?;
        let expected_root = normalize_project_root(expected.project_root)?;
        if self.project_id != expected.project_id {
            return Err(ConversationTurnBindingError::ContextMismatch("project_id"));
        }
        if self.project_root != expected_root {
            return Err(ConversationTurnBindingError::ContextMismatch(
                "project_root",
            ));
        }
        if self.workflow_id != expected.workflow_id {
            return Err(ConversationTurnBindingError::ContextMismatch("workflow_id"));
        }
        if self.run_id != expected.run_id {
            return Err(ConversationTurnBindingError::ContextMismatch("run_id"));
        }
        if self.lifecycle != ConversationTurnLifecycle::Active {
            return Err(ConversationTurnBindingError::InactiveLifecycle(
                self.lifecycle,
            ));
        }
        match self.thread_id.as_deref() {
            Some(thread_id) if !thread_id.trim().is_empty() && thread_id == thread_id.trim() => {}
            _ => return Err(ConversationTurnBindingError::ThreadUnbound),
        }

        let access = CapabilityAccess::new(&self.profile_id, &self.role);
        capability_registry::validate_exact_capability_set(access, &self.allowed_capabilities)?;
        capability_registry::authorize(access, capability_name)?;
        Ok(())
    }

    /// The host-configured run window is part of the trusted capability
    /// boundary.  A stale `Active` record after a crashed or missing receipt
    /// cannot keep publishing a tool indefinitely.
    pub(crate) fn validate_for_capability_at(
        &self,
        expected: ConversationTurnBindingExpectation<'_>,
        capability_name: &str,
        observed_at_ms: i64,
    ) -> Result<(), ConversationTurnBindingError> {
        self.validate_for_capability(expected, capability_name)?;
        self.validate_runtime_window(observed_at_ms)
    }

    pub(crate) fn user_message_snapshot(&self) -> &str {
        &self.user_message_snapshot
    }

    pub(crate) fn proposal_idempotency_material(&self) -> String {
        format!("{}:{}:{}", self.project_id, self.workflow_id, self.turn_id)
    }

    /// Record the handler outcome only from the server-side MCP dispatcher.
    /// Audit settlement is deliberately stored separately so a later audit
    /// failure cannot erase a proposal that the existing writer persisted.
    pub(crate) fn record_capability_outcome(
        &mut self,
        capability_name: &str,
        outcome: ConversationCapabilityOutcome,
        updated_at_ms: i64,
    ) -> Result<(), ConversationTurnBindingError> {
        self.validate_host_fields()?;
        if self.lifecycle != ConversationTurnLifecycle::Active {
            return Err(ConversationTurnBindingError::InactiveLifecycle(
                self.lifecycle,
            ));
        }
        if outcome == ConversationCapabilityOutcome::NotRequested {
            return Err(ConversationTurnBindingError::ContextMismatch(
                "capability_outcome",
            ));
        }
        if updated_at_ms < self.updated_at_ms || updated_at_ms <= 0 {
            return Err(ConversationTurnBindingError::InvalidTimestamp);
        }
        self.validate_runtime_window(updated_at_ms)?;
        let access = CapabilityAccess::new(&self.profile_id, &self.role);
        capability_registry::validate_exact_capability_set(access, &self.allowed_capabilities)?;
        capability_registry::authorize(access, capability_name)?;
        self.capability_outcomes
            .insert(capability_name.to_string(), outcome);
        self.updated_at_ms = updated_at_ms;
        Ok(())
    }

    pub(crate) fn record_capability_audit_outcome(
        &mut self,
        capability_name: &str,
        outcome: ConversationCapabilityOutcome,
        updated_at_ms: i64,
    ) -> Result<(), ConversationTurnBindingError> {
        self.validate_host_fields()?;
        if self.lifecycle != ConversationTurnLifecycle::Active {
            return Err(ConversationTurnBindingError::InactiveLifecycle(
                self.lifecycle,
            ));
        }
        if outcome == ConversationCapabilityOutcome::NotRequested {
            return Err(ConversationTurnBindingError::ContextMismatch(
                "capability_audit_outcome",
            ));
        }
        if updated_at_ms < self.updated_at_ms || updated_at_ms <= 0 {
            return Err(ConversationTurnBindingError::InvalidTimestamp);
        }
        self.validate_runtime_window(updated_at_ms)?;
        let access = CapabilityAccess::new(&self.profile_id, &self.role);
        capability_registry::validate_exact_capability_set(access, &self.allowed_capabilities)?;
        capability_registry::authorize(access, capability_name)?;
        self.capability_audit_outcomes
            .insert(capability_name.to_string(), outcome);
        self.updated_at_ms = updated_at_ms;
        Ok(())
    }

    /// Read a previously settled exact capability after a terminal transport
    /// receipt.  Lifecycle is intentionally not required to be active here:
    /// the host must still be able to report a safely settled result while it
    /// closes the turn.
    pub(crate) fn capability_outcome(
        &self,
        capability_name: &str,
    ) -> Result<ConversationCapabilityOutcome, ConversationTurnBindingError> {
        self.validate_host_fields()?;
        let access = CapabilityAccess::new(&self.profile_id, &self.role);
        capability_registry::validate_exact_capability_set(access, &self.allowed_capabilities)?;
        capability_registry::authorize(access, capability_name)?;
        Ok(self
            .capability_outcomes
            .get(capability_name)
            .copied()
            .unwrap_or_default())
    }

    pub(crate) fn capability_audit_outcome(
        &self,
        capability_name: &str,
    ) -> Result<ConversationCapabilityOutcome, ConversationTurnBindingError> {
        self.validate_host_fields()?;
        let access = CapabilityAccess::new(&self.profile_id, &self.role);
        capability_registry::validate_exact_capability_set(access, &self.allowed_capabilities)?;
        capability_registry::authorize(access, capability_name)?;
        Ok(self
            .capability_audit_outcomes
            .get(capability_name)
            .copied()
            .unwrap_or_default())
    }

    fn validate_runtime_window(
        &self,
        observed_at_ms: i64,
    ) -> Result<(), ConversationTurnBindingError> {
        if observed_at_ms < self.created_at_ms || observed_at_ms <= 0 {
            return Err(ConversationTurnBindingError::InvalidTimestamp);
        }
        let runtime_ms = self
            .max_runtime_minutes
            .checked_mul(60_000)
            .filter(|runtime_ms| *runtime_ms > 0)
            .ok_or(ConversationTurnBindingError::InvalidRuntimeLimit)?;
        let expires_at_ms = self
            .created_at_ms
            .checked_add(runtime_ms)
            .ok_or(ConversationTurnBindingError::InvalidRuntimeLimit)?;
        if observed_at_ms > expires_at_ms {
            return Err(ConversationTurnBindingError::RuntimeExpired);
        }
        Ok(())
    }

    fn validate_host_fields(&self) -> Result<(), ConversationTurnBindingError> {
        if self.profile_id != PROFILE_SUPERVISOR_READ_ONLY || self.role != ROLE_PROJECT_SUPERVISOR {
            return Err(ConversationTurnBindingError::ContextMismatch(
                "profile_or_role",
            ));
        }
        require_exact_nonempty(&self.project_id, "project_id")?;
        require_exact_nonempty(&self.workflow_id, "workflow_id")?;
        require_exact_nonempty(&self.turn_id, "turn_id")?;
        require_exact_nonempty(&self.run_id, "run_id")?;
        require_nonempty_text(&self.user_message_snapshot, "user_message_snapshot")?;
        if self.transport_attempt == 0 {
            return Err(ConversationTurnBindingError::MissingField(
                "transport_attempt",
            ));
        }
        if self.created_at_ms <= 0 || self.updated_at_ms < self.created_at_ms {
            return Err(ConversationTurnBindingError::InvalidTimestamp);
        }
        if self.max_runtime_minutes != SUPERVISOR_CONVERSATION_MAX_RUNTIME_MINUTES {
            return Err(ConversationTurnBindingError::InvalidRuntimeLimit);
        }
        if !self
            .run_id
            .starts_with(SUPERVISOR_CONVERSATION_RUN_ID_PREFIX)
            || self.run_id == SUPERVISOR_CONVERSATION_RUN_ID_PREFIX
        {
            return Err(ConversationTurnBindingError::InvalidRunIdentity);
        }
        let normalized_root = normalize_project_root(&self.project_root)?;
        if self.project_root != normalized_root {
            return Err(ConversationTurnBindingError::InvalidProjectRoot);
        }
        if self.project_id != crate::project_id(&self.project_root) {
            return Err(ConversationTurnBindingError::ProjectIdMismatch);
        }
        Ok(())
    }
}

/// Lexically normalize a trusted project root without resolving symlinks or
/// touching the filesystem.  The host then persists this canonical spelling,
/// so later equality checks cannot accept `.` / `..` / duplicate-separator
/// variants as an equivalent-but-unverified project root.
pub(crate) fn normalize_project_root(
    project_root: &str,
) -> Result<String, ConversationTurnBindingError> {
    require_exact_nonempty(project_root, "project_root")?;
    let path = Path::new(project_root);
    if !path.is_absolute() {
        return Err(ConversationTurnBindingError::InvalidProjectRoot);
    }

    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(component.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() {
                    return Err(ConversationTurnBindingError::InvalidProjectRoot);
                }
            }
            Component::Normal(segment) => normalized.push(segment),
        }
    }
    let output = normalized.to_string_lossy().to_string();
    if output.trim().is_empty() || !Path::new(&output).is_absolute() {
        return Err(ConversationTurnBindingError::InvalidProjectRoot);
    }
    Ok(output)
}

fn require_exact_nonempty(
    value: &str,
    field: &'static str,
) -> Result<(), ConversationTurnBindingError> {
    if value.trim().is_empty() || value != value.trim() {
        return Err(ConversationTurnBindingError::MissingField(field));
    }
    Ok(())
}

fn require_nonempty_text(
    value: &str,
    field: &'static str,
) -> Result<(), ConversationTurnBindingError> {
    if value.trim().is_empty() {
        return Err(ConversationTurnBindingError::MissingField(field));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROOT: &str = "/tmp/supervisor-conversation-binding";
    const RUN_ID: &str = "supervisor-conversation:offline-test-run";

    fn input() -> SupervisorConversationTurnInput {
        SupervisorConversationTurnInput {
            project_id: crate::project_id(ROOT),
            project_root: ROOT.to_string(),
            workflow_id: "workflow:offline-test".to_string(),
            turn_id: "turn:offline-test".to_string(),
            transport_attempt: 1,
            run_id: RUN_ID.to_string(),
            user_message_snapshot: "请给出方案".to_string(),
            created_at_ms: 10,
            max_runtime_minutes: SUPERVISOR_CONVERSATION_MAX_RUNTIME_MINUTES,
        }
    }

    fn expectation<'a>(project_id: &'a str) -> ConversationTurnBindingExpectation<'a> {
        ConversationTurnBindingExpectation {
            project_id,
            project_root: ROOT,
            workflow_id: "workflow:offline-test",
            run_id: RUN_ID,
        }
    }

    #[test]
    fn host_binding_is_exactly_supervisor_read_only_and_active_after_thread_observation() {
        let mut binding = ConversationTurnBinding::establish_supervisor_read_only(input()).unwrap();
        let project_id = crate::project_id(ROOT);
        assert_eq!(binding.profile_id, PROFILE_SUPERVISOR_READ_ONLY);
        assert_eq!(binding.role, ROLE_PROJECT_SUPERVISOR);
        assert_eq!(
            binding.allowed_capabilities,
            vec![
                "submit_proposal",
                "knowledge_search",
                "knowledge_read",
                "knowledge_open",
                "knowledge_cite"
            ]
        );
        assert_eq!(binding.lifecycle, ConversationTurnLifecycle::Starting);

        binding
            .activate_with_host_observed_thread("thread:offline-test", 11)
            .unwrap();
        assert_eq!(binding.lifecycle, ConversationTurnLifecycle::Active);
        assert!(binding
            .validate_for_capability(expectation(&project_id), "submit_proposal")
            .is_ok());
    }

    #[test]
    fn binding_fails_closed_for_missing_thread_context_or_capability() {
        let mut binding = ConversationTurnBinding::establish_supervisor_read_only(input()).unwrap();
        let project_id = crate::project_id(ROOT);
        assert!(matches!(
            binding.validate_for_capability(expectation(&project_id), "submit_proposal"),
            Err(ConversationTurnBindingError::InactiveLifecycle(_))
        ));

        binding
            .activate_with_host_observed_thread("thread:offline-test", 11)
            .unwrap();
        binding.allowed_capabilities.clear();
        assert!(matches!(
            binding.validate_for_capability(expectation(&project_id), "submit_proposal"),
            Err(ConversationTurnBindingError::Capability(
                CapabilityAuthorizationError::EmptyCapabilitySet
            ))
        ));

        binding.allowed_capabilities = capability_registry::frozen_capability_names(
            CapabilityAccess::new(PROFILE_SUPERVISOR_READ_ONLY, ROLE_PROJECT_SUPERVISOR),
        )
        .unwrap();
        assert!(matches!(
            binding.validate_for_capability(
                ConversationTurnBindingExpectation {
                    workflow_id: "workflow:other",
                    ..expectation(&project_id)
                },
                "submit_proposal"
            ),
            Err(ConversationTurnBindingError::ContextMismatch("workflow_id"))
        ));
    }

    #[test]
    fn active_binding_from_before_knowledge_capability_expansion_is_stale_and_rejected() {
        let mut binding = ConversationTurnBinding::establish_supervisor_read_only(input()).unwrap();
        let project_id = crate::project_id(ROOT);
        binding
            .activate_with_host_observed_thread("thread:offline-test", 11)
            .unwrap();

        // This is the exact persisted pre-O5 set.  It is deliberately not
        // hot-extended: the existing active turn must close instead of gaining
        // new knowledge access after it was established.
        binding.allowed_capabilities = vec!["submit_proposal".to_string()];
        assert!(matches!(
            binding.validate_for_capability(expectation(&project_id), "knowledge_read"),
            Err(ConversationTurnBindingError::Capability(
                CapabilityAuthorizationError::CapabilitySetNotExact { .. }
            ))
        ));
    }

    #[test]
    fn active_binding_rejects_an_added_knowledge_write_without_changing_its_lifecycle() {
        let mut binding = ConversationTurnBinding::establish_supervisor_read_only(input()).unwrap();
        let project_id = crate::project_id(ROOT);
        binding
            .activate_with_host_observed_thread("thread:offline-test", 11)
            .unwrap();
        binding
            .allowed_capabilities
            .push("knowledge_write".to_string());

        assert!(matches!(
            binding.validate_for_capability(expectation(&project_id), "knowledge_read"),
            Err(ConversationTurnBindingError::Capability(
                CapabilityAuthorizationError::UnknownCapability(name)
            )) if name == "knowledge_write"
        ));
        assert_eq!(binding.lifecycle, ConversationTurnLifecycle::Active);
        assert_eq!(binding.thread_id.as_deref(), Some("thread:offline-test"));
    }

    #[test]
    fn active_binding_rejects_knowledge_capability_when_project_context_mismatches() {
        let mut binding = ConversationTurnBinding::establish_supervisor_read_only(input()).unwrap();
        let project_id = crate::project_id(ROOT);
        binding
            .activate_with_host_observed_thread("thread:offline-test", 11)
            .unwrap();

        assert!(matches!(
            binding.validate_for_capability(
                ConversationTurnBindingExpectation {
                    project_id: &project_id,
                    project_root: "/tmp/other-project",
                    workflow_id: "workflow:offline-test",
                    run_id: RUN_ID,
                },
                "knowledge_search"
            ),
            Err(ConversationTurnBindingError::ContextMismatch(
                "project_root"
            ))
        ));
    }

    #[test]
    fn root_is_normalized_before_identity_is_accepted() {
        assert_eq!(
            normalize_project_root("/tmp/supervisor/./conversation/../binding").unwrap(),
            "/tmp/supervisor/binding"
        );
        let mut malformed = input();
        malformed.project_root = "/tmp/./supervisor-conversation-binding".to_string();
        let normalized =
            ConversationTurnBinding::establish_supervisor_read_only(malformed).unwrap();
        assert_eq!(normalized.project_root, ROOT);

        let mut wrong_project_id = input();
        wrong_project_id.project_id = "project:other".to_string();
        assert!(matches!(
            ConversationTurnBinding::establish_supervisor_read_only(wrong_project_id),
            Err(ConversationTurnBindingError::ProjectIdMismatch)
        ));
    }

    #[test]
    fn only_host_observed_thread_may_activate_and_terminal_state_cannot_reopen() {
        let mut binding = ConversationTurnBinding::establish_supervisor_read_only(input()).unwrap();
        assert!(binding
            .activate_with_host_observed_thread(" thread:bad", 11)
            .is_err());
        binding
            .activate_with_host_observed_thread("thread:offline-test", 11)
            .unwrap();
        binding
            .mark_terminal(ConversationTurnLifecycle::Stopped, 12)
            .unwrap();
        assert!(binding
            .activate_with_host_observed_thread("thread:another", 13)
            .is_err());
        assert!(binding
            .mark_terminal(ConversationTurnLifecycle::Completed, 13)
            .is_err());
    }

    #[test]
    fn active_binding_expires_at_the_fixed_host_runtime_window() {
        let mut binding = ConversationTurnBinding::establish_supervisor_read_only(input()).unwrap();
        let project_id = crate::project_id(ROOT);
        binding
            .activate_with_host_observed_thread("thread:offline-test", 11)
            .unwrap();

        assert!(binding
            .validate_for_capability_at(expectation(&project_id), "submit_proposal", 60_010)
            .is_ok());
        assert!(matches!(
            binding.validate_for_capability_at(expectation(&project_id), "submit_proposal", 60_011),
            Err(ConversationTurnBindingError::RuntimeExpired)
        ));
    }
}
