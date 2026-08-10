//! M4C05 ordinary-product Secretary command adapter.
//!
//! The application-service core owns deterministic context and brief
//! construction. This module only composes its narrow ports from the ordinary
//! product runtime; it does not inspect a cwd, workflow sidecar, frontend
//! cache, provider response, or source-owner write boundary.

use crate::m4_secretary_read_model::M4CoordinationSnapshot;
#[cfg(test)]
use crate::m4_secretary_read_model::M4PersonalActionRead;
use crate::m4_secretary_service::{
    M4SecretaryApplicationOutcome, M4SecretaryApplicationService,
    M4SecretaryControlledModelEnhancementPort, M4SecretaryCoordinationSnapshotReadPort,
    M4SecretaryHandoffPort, M4SecretaryHandoffPortRecord, M4SecretaryHandoffRequest,
    M4SecretaryHash, M4SecretaryInvocationClaimOutcome, M4SecretaryInvocationReceipt,
    M4SecretaryInvocationTerminal, M4SecretaryModelEnhancementRequest,
    M4SecretaryModelInvocationClaim, M4SecretaryModelInvocationLedgerPort,
    M4SecretaryModelPortOutcome, M4SecretaryOpaqueRef, M4SecretaryRoleSessionReadPort,
    M4SecretaryRoleSessionState, M4SecretaryServiceError, M4SecretaryTypedRef,
};
use serde::Serialize;

const M4C05_RUNTIME_UNAVAILABLE: &str = "M4C05_SECRETARY_RUNTIME_UNAVAILABLE";
const M4C05_REPOSITORY_UNAVAILABLE: &str = "M4C05_COORDINATION_REPOSITORY_UNAVAILABLE";
const M4C05_HANDOFF_UNAVAILABLE: &str = "M6_RECIPIENT_UNAVAILABLE";
const M4C05_MODEL_LEDGER_UNAVAILABLE: &str = "M4C05_MODEL_LEDGER_UNAVAILABLE";
const M4C05_MODEL_UNAVAILABLE: &str = "M4C05_MODEL_ADAPTER_UNAVAILABLE";
const M4C05_CONTEXT_UNAVAILABLE_REASON: &str = "秘书上下文暂不可用，请稍后重试。";

/// Existing command fields remain stable. The optional metadata only exposes
/// server-minted context/brief refs and mechanical counts.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) struct SecretaryExplainOutcome {
    /// ready | unavailable
    pub(crate) status: String,
    pub(crate) explanation: Option<String>,
    pub(crate) reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) context_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) brief_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) scope_source_watermark: Option<String>,
    pub(crate) attention_count: usize,
    pub(crate) personal_action_count: usize,
}

impl SecretaryExplainOutcome {
    fn ready(application_outcome: &M4SecretaryApplicationOutcome) -> Self {
        let brief = &application_outcome.deterministic_brief;
        Self {
            status: "ready".to_string(),
            explanation: Some(render_mechanical_explanation(application_outcome)),
            reason: None,
            context_ref: Some(application_outcome.context.context_ref.as_str().to_string()),
            brief_ref: Some(brief.brief_ref.as_str().to_string()),
            scope_source_watermark: Some(brief.scope_source_watermark.as_str().to_string()),
            attention_count: brief.attention_items.len(),
            personal_action_count: brief.personal_actions.len(),
        }
    }

    fn unavailable() -> Self {
        Self {
            status: "unavailable".to_string(),
            explanation: None,
            reason: Some(M4C05_CONTEXT_UNAVAILABLE_REASON.to_string()),
            context_ref: None,
            brief_ref: None,
            scope_source_watermark: None,
            attention_count: 0,
            personal_action_count: 0,
        }
    }
}

/// This wording is intentionally mechanical: it never receives raw source
/// titles, user messages, prompt text, or model output.
fn render_mechanical_explanation(application_outcome: &M4SecretaryApplicationOutcome) -> String {
    let brief = &application_outcome.deterministic_brief;
    match (
        brief.attention_items.len(),
        brief.personal_actions.len(),
    ) {
        (0, 0) => "当前没有来源关注事项，也没有独立个人待办。".to_string(),
        (attention_count, personal_action_count) => format!(
            "当前有 {attention_count} 项来源关注和 {personal_action_count} 项独立个人待办，已按服务端协调快照机械整理。"
        ),
    }
}

/// M3 supplies the only identity and PersonalScope authority. Any missing,
/// malformed, or non-exact session becomes the same stable soft failure at the
/// command boundary.
struct OrdinaryProductRoleSessionReadPort<'a> {
    runtime: &'a crate::m3_role_session_read_model::M3RoleSessionReadRuntimeSlot,
}

impl M4SecretaryRoleSessionReadPort for OrdinaryProductRoleSessionReadPort<'_> {
    fn read_personal_secretary_role_session(
        &self,
    ) -> Result<M4SecretaryRoleSessionState, M4SecretaryServiceError> {
        let status = self
            .runtime
            .secretary_status()
            .map_err(|_| M4SecretaryServiceError::new(M4C05_RUNTIME_UNAVAILABLE))?;
        let invalid = || M4SecretaryServiceError::new(M4C05_RUNTIME_UNAVAILABLE);

        Ok(M4SecretaryRoleSessionState {
            role_session_ref: M4SecretaryOpaqueRef::new(status.role_session_id)
                .map_err(|_| invalid())?,
            role_ref: M4SecretaryTypedRef::new(status.role_ref).map_err(|_| invalid())?,
            scope_ref: M4SecretaryTypedRef::new(status.scope_ref).map_err(|_| invalid())?,
            current_object_ref: M4SecretaryTypedRef::new(status.current_object_ref)
                .map_err(|_| invalid())?,
            execution_channel_code: status.execution_channel,
            session_state_code: status.session_state,
            permission_snapshot_ref: M4SecretaryOpaqueRef::new(status.permission_snapshot_ref)
                .map_err(|_| invalid())?,
            owner_fingerprint: M4SecretaryHash::new(status.owner_fingerprint)
                .map_err(|_| invalid())?,
        })
    }
}

#[cfg(not(test))]
struct OrdinaryProductCoordinationSnapshotReadPort<'a> {
    repository: &'a crate::m4_secretary_repository::M4SecretarySqliteRepository,
}

#[cfg(not(test))]
impl M4SecretaryCoordinationSnapshotReadPort for OrdinaryProductCoordinationSnapshotReadPort<'_> {
    fn read_coordination_snapshot(
        &self,
        scope_ref: &M4SecretaryTypedRef,
    ) -> Result<M4CoordinationSnapshot, M4SecretaryServiceError> {
        self.repository
            .read_coordination_snapshot(scope_ref.as_str())
            .map_err(|_| M4SecretaryServiceError::new(M4C05_REPOSITORY_UNAVAILABLE))
    }
}

/// M6 recipient composition is not installed in the ordinary product yet.
/// The port stays explicit and returns only a bounded unavailable state.
#[derive(Clone, Copy, Default)]
struct OrdinaryProductUnavailableHandoffPort;

impl M4SecretaryHandoffPort for OrdinaryProductUnavailableHandoffPort {
    fn create_handoff(
        &self,
        _request: &M4SecretaryHandoffRequest,
    ) -> Result<M4SecretaryHandoffPortRecord, M4SecretaryServiceError> {
        Ok(M4SecretaryHandoffPortRecord::Unavailable {
            error_code: M4C05_HANDOFF_UNAVAILABLE.to_string(),
        })
    }

    fn read_handoff_receipt(
        &self,
        _handoff_ref: &M4SecretaryOpaqueRef,
    ) -> Result<M4SecretaryHandoffPortRecord, M4SecretaryServiceError> {
        Ok(M4SecretaryHandoffPortRecord::Unavailable {
            error_code: M4C05_HANDOFF_UNAVAILABLE.to_string(),
        })
    }
}

/// No ordinary-product durable model ledger has been composed yet. A rejected
/// claim is deliberately not a dispatch grant.
#[derive(Clone, Copy, Default)]
struct OrdinaryProductUnavailableInvocationLedgerPort;

impl M4SecretaryModelInvocationLedgerPort for OrdinaryProductUnavailableInvocationLedgerPort {
    fn claim_invocation(
        &self,
        _claim: &M4SecretaryModelInvocationClaim,
    ) -> Result<M4SecretaryInvocationClaimOutcome, M4SecretaryServiceError> {
        Ok(M4SecretaryInvocationClaimOutcome::Rejected {
            error_code: M4C05_MODEL_LEDGER_UNAVAILABLE.to_string(),
        })
    }

    fn terminal_invocation(
        &self,
        _terminal: &M4SecretaryInvocationTerminal,
    ) -> Result<M4SecretaryInvocationReceipt, M4SecretaryServiceError> {
        Err(M4SecretaryServiceError::new(M4C05_MODEL_LEDGER_UNAVAILABLE))
    }
}

/// The ordinary product also has no controlled M6 model adapter. Read and
/// explain never reach this port because they use read_deterministic_brief.
#[derive(Clone, Copy, Default)]
struct OrdinaryProductUnavailableModelPort;

impl M4SecretaryControlledModelEnhancementPort for OrdinaryProductUnavailableModelPort {
    fn enhance(
        &self,
        _request: &M4SecretaryModelEnhancementRequest,
    ) -> Result<M4SecretaryModelPortOutcome, M4SecretaryServiceError> {
        Err(M4SecretaryServiceError::new(M4C05_MODEL_UNAVAILABLE))
    }
}

fn run_secretary_explain_with_ports<RS, CS, HP, LP, MP>(
    role_session_port: &RS,
    coordination_port: &CS,
    handoff_port: &HP,
    invocation_ledger_port: &LP,
    model_port: &MP,
) -> SecretaryExplainOutcome
where
    RS: M4SecretaryRoleSessionReadPort,
    CS: M4SecretaryCoordinationSnapshotReadPort,
    HP: M4SecretaryHandoffPort,
    LP: M4SecretaryModelInvocationLedgerPort,
    MP: M4SecretaryControlledModelEnhancementPort,
{
    let service = M4SecretaryApplicationService::new(
        role_session_port,
        coordination_port,
        handoff_port,
        invocation_ledger_port,
        model_port,
    );
    match service.read_deterministic_brief() {
        Ok(application_outcome) => SecretaryExplainOutcome::ready(&application_outcome),
        Err(_) => SecretaryExplainOutcome::unavailable(),
    }
}

#[cfg(not(test))]
fn run_secretary_explain_from_app_state(state: &crate::AppState) -> SecretaryExplainOutcome {
    let Some(repository) = state.m4_secretary_repository.as_ref() else {
        return SecretaryExplainOutcome::unavailable();
    };
    let role_session_port = OrdinaryProductRoleSessionReadPort {
        runtime: &state.m3_role_session_read_runtime,
    };
    let coordination_port = OrdinaryProductCoordinationSnapshotReadPort { repository };
    let handoff_port = OrdinaryProductUnavailableHandoffPort;
    let invocation_ledger_port = OrdinaryProductUnavailableInvocationLedgerPort;
    let model_port = OrdinaryProductUnavailableModelPort;
    run_secretary_explain_with_ports(
        &role_session_port,
        &coordination_port,
        &handoff_port,
        &invocation_ledger_port,
        &model_port,
    )
}

/// AppState intentionally has no M4 repository field in test builds. The
/// focused tests inject narrow ports below, while this command-facing branch
/// keeps a stable soft failure if it is exercised from a broader test.
#[cfg(test)]
fn run_secretary_explain_from_app_state(_state: &crate::AppState) -> SecretaryExplainOutcome {
    SecretaryExplainOutcome::unavailable()
}

#[tauri::command]
pub(crate) async fn run_secretary_explain(
    state: tauri::State<'_, crate::AppState>,
) -> Result<SecretaryExplainOutcome, String> {
    Ok(run_secretary_explain_from_app_state(state.inner()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    fn fixture_hash(character: char) -> String {
        std::iter::repeat(character).take(64).collect()
    }

    fn fixture_opaque_ref(namespace: &str, character: char) -> M4SecretaryOpaqueRef {
        M4SecretaryOpaqueRef::new(format!("{namespace}:sha256:{}", fixture_hash(character)))
            .expect("fixture opaque reference")
    }

    fn fixture_role_session() -> M4SecretaryRoleSessionState {
        M4SecretaryRoleSessionState {
            role_session_ref: fixture_opaque_ref("role-session", '1'),
            role_ref: M4SecretaryTypedRef::new("role:secretary:personal-primary")
                .expect("fixture role ref"),
            scope_ref: M4SecretaryTypedRef::new("scope:personal:primary")
                .expect("fixture scope ref"),
            current_object_ref: M4SecretaryTypedRef::new("personal-workbench:primary")
                .expect("fixture current object ref"),
            execution_channel_code: "DAILY".to_string(),
            session_state_code: "ACTIVE".to_string(),
            permission_snapshot_ref: fixture_opaque_ref("permission", '2'),
            owner_fingerprint: M4SecretaryHash::new(fixture_hash('3')).expect("fixture owner hash"),
        }
    }

    fn fixture_snapshot() -> M4CoordinationSnapshot {
        M4CoordinationSnapshot {
            scope_ref: "scope:personal:primary".to_string(),
            scope_source_watermark: fixture_hash('4'),
            inbox_items: Vec::new(),
            open_loops: Vec::new(),
            personal_actions: vec![M4PersonalActionRead {
                personal_action_id: format!("personal-action:{}", fixture_hash('5')),
                explicit_user_command_ref: fixture_opaque_ref("command", '6').as_str().to_string(),
                title: "private user text must not appear".to_string(),
                status: "OPEN".to_string(),
                due_at_utc: None,
                revision: "0".to_string(),
            }],
            notifications: Vec::new(),
            reminders: Vec::new(),
            owner_writeback_receipts: Vec::new(),
        }
    }

    struct StaticRoleSessionPort {
        result: Result<M4SecretaryRoleSessionState, M4SecretaryServiceError>,
    }

    impl M4SecretaryRoleSessionReadPort for StaticRoleSessionPort {
        fn read_personal_secretary_role_session(
            &self,
        ) -> Result<M4SecretaryRoleSessionState, M4SecretaryServiceError> {
            self.result.clone()
        }
    }

    struct StaticCoordinationPort {
        result: Result<M4CoordinationSnapshot, M4SecretaryServiceError>,
    }

    impl M4SecretaryCoordinationSnapshotReadPort for StaticCoordinationPort {
        fn read_coordination_snapshot(
            &self,
            _scope_ref: &M4SecretaryTypedRef,
        ) -> Result<M4CoordinationSnapshot, M4SecretaryServiceError> {
            self.result.clone()
        }
    }

    #[derive(Default)]
    struct CountingHandoffPort {
        calls: Cell<usize>,
    }

    impl M4SecretaryHandoffPort for CountingHandoffPort {
        fn create_handoff(
            &self,
            _request: &M4SecretaryHandoffRequest,
        ) -> Result<M4SecretaryHandoffPortRecord, M4SecretaryServiceError> {
            self.calls.set(self.calls.get() + 1);
            Err(M4SecretaryServiceError::new("test_handoff_called"))
        }

        fn read_handoff_receipt(
            &self,
            _handoff_ref: &M4SecretaryOpaqueRef,
        ) -> Result<M4SecretaryHandoffPortRecord, M4SecretaryServiceError> {
            self.calls.set(self.calls.get() + 1);
            Err(M4SecretaryServiceError::new("test_handoff_called"))
        }
    }

    #[derive(Default)]
    struct CountingInvocationLedgerPort {
        calls: Cell<usize>,
    }

    impl M4SecretaryModelInvocationLedgerPort for CountingInvocationLedgerPort {
        fn claim_invocation(
            &self,
            _claim: &M4SecretaryModelInvocationClaim,
        ) -> Result<M4SecretaryInvocationClaimOutcome, M4SecretaryServiceError> {
            self.calls.set(self.calls.get() + 1);
            Err(M4SecretaryServiceError::new("test_ledger_called"))
        }

        fn terminal_invocation(
            &self,
            _terminal: &M4SecretaryInvocationTerminal,
        ) -> Result<M4SecretaryInvocationReceipt, M4SecretaryServiceError> {
            self.calls.set(self.calls.get() + 1);
            Err(M4SecretaryServiceError::new("test_ledger_called"))
        }
    }

    #[derive(Default)]
    struct CountingModelPort {
        calls: Cell<usize>,
    }

    impl M4SecretaryControlledModelEnhancementPort for CountingModelPort {
        fn enhance(
            &self,
            _request: &M4SecretaryModelEnhancementRequest,
        ) -> Result<M4SecretaryModelPortOutcome, M4SecretaryServiceError> {
            self.calls.set(self.calls.get() + 1);
            Err(M4SecretaryServiceError::new("test_model_called"))
        }
    }

    fn run_fixture(
        role_session_result: Result<M4SecretaryRoleSessionState, M4SecretaryServiceError>,
        snapshot_result: Result<M4CoordinationSnapshot, M4SecretaryServiceError>,
    ) -> (
        SecretaryExplainOutcome,
        CountingHandoffPort,
        CountingInvocationLedgerPort,
        CountingModelPort,
    ) {
        let role_session_port = StaticRoleSessionPort {
            result: role_session_result,
        };
        let coordination_port = StaticCoordinationPort {
            result: snapshot_result,
        };
        let handoff_port = CountingHandoffPort::default();
        let invocation_ledger_port = CountingInvocationLedgerPort::default();
        let model_port = CountingModelPort::default();
        let outcome = run_secretary_explain_with_ports(
            &role_session_port,
            &coordination_port,
            &handoff_port,
            &invocation_ledger_port,
            &model_port,
        );
        (outcome, handoff_port, invocation_ledger_port, model_port)
    }

    #[test]
    fn m4c05_secretary_adapter_renders_mechanical_brief_without_model() {
        let (outcome, handoff, ledger, model) =
            run_fixture(Ok(fixture_role_session()), Ok(fixture_snapshot()));

        assert_eq!(outcome.status, "ready");
        assert_eq!(outcome.reason, None);
        assert_eq!(outcome.attention_count, 0);
        assert_eq!(outcome.personal_action_count, 1);
        assert!(outcome
            .context_ref
            .as_deref()
            .is_some_and(|value| value.starts_with("secretary-context:sha256:")));
        assert!(outcome
            .brief_ref
            .as_deref()
            .is_some_and(|value| value.starts_with("secretary-brief:sha256:")));
        assert!(
            !outcome
                .explanation
                .as_deref()
                .unwrap_or_default()
                .contains("private user text"),
            "mechanical renderer must not surface a personal-action title"
        );
        assert_eq!(handoff.calls.get(), 0, "read/explain has no Handoff write");
        assert_eq!(ledger.calls.get(), 0, "read/explain has no ledger claim");
        assert_eq!(model.calls.get(), 0, "read/explain has no model invocation");
    }

    #[test]
    fn m4c05_secretary_adapter_missing_runtime_is_stable_soft_failure() {
        let (outcome, handoff, ledger, model) = run_fixture(
            Err(M4SecretaryServiceError::new(M4C05_RUNTIME_UNAVAILABLE)),
            Ok(fixture_snapshot()),
        );

        assert_eq!(outcome.status, "unavailable");
        assert_eq!(outcome.explanation, None);
        assert_eq!(
            outcome.reason.as_deref(),
            Some(M4C05_CONTEXT_UNAVAILABLE_REASON)
        );
        assert_eq!(handoff.calls.get(), 0);
        assert_eq!(ledger.calls.get(), 0);
        assert_eq!(model.calls.get(), 0);
    }

    #[test]
    fn m4c05_secretary_adapter_missing_repository_is_stable_soft_failure() {
        let (outcome, handoff, ledger, model) = run_fixture(
            Ok(fixture_role_session()),
            Err(M4SecretaryServiceError::new(M4C05_REPOSITORY_UNAVAILABLE)),
        );

        assert_eq!(outcome.status, "unavailable");
        assert_eq!(outcome.explanation, None);
        assert_eq!(
            outcome.reason.as_deref(),
            Some(M4C05_CONTEXT_UNAVAILABLE_REASON)
        );
        assert_eq!(outcome.context_ref, None);
        assert_eq!(outcome.brief_ref, None);
        assert_eq!(handoff.calls.get(), 0);
        assert_eq!(ledger.calls.get(), 0);
        assert_eq!(model.calls.get(), 0);
    }
}
