//! M4C05/C06 ordinary-product Secretary command bridge.
//!
//! The application-service core owns deterministic context and brief
//! construction. This module composes only server-owned M3/M4 ports and
//! exposes a narrow coordination command bridge; it never accepts a cwd,
//! project locator, renderer role/scope, source-owner payload, or provider
//! body.

use crate::m4_secretary_read_model::M4CoordinationSnapshot;
#[cfg(test)]
use crate::m4_secretary_read_model::M4PersonalActionRead;
use crate::m4_secretary_repository::M4CoordinationCommandOutcome;
#[cfg(not(test))]
use crate::m4_secretary_repository::M4SecretarySqliteRepository;
use crate::m4_secretary_service::{
    M4SecretaryApplicationOutcome, M4SecretaryApplicationService,
    M4SecretaryControlledModelEnhancementPort, M4SecretaryCoordinationSnapshotReadPort,
    M4SecretaryHandoffOutcome, M4SecretaryHandoffPort, M4SecretaryHandoffPortRecord,
    M4SecretaryHandoffRequest, M4SecretaryHash, M4SecretaryInvocationClaimOutcome,
    M4SecretaryInvocationReceipt, M4SecretaryInvocationTerminal,
    M4SecretaryModelEnhancementRequest, M4SecretaryModelInvocationClaim,
    M4SecretaryModelInvocationLedgerPort, M4SecretaryModelPortOutcome, M4SecretaryOpaqueRef,
    M4SecretaryRoleSessionReadPort, M4SecretaryRoleSessionState, M4SecretaryServiceError,
    M4SecretaryTypedRef,
};
use serde::{Deserialize, Serialize};

const M4C05_RUNTIME_UNAVAILABLE: &str = "M4C05_SECRETARY_RUNTIME_UNAVAILABLE";
const M4C05_REPOSITORY_UNAVAILABLE: &str = "M4C05_COORDINATION_REPOSITORY_UNAVAILABLE";
const M4C05_HANDOFF_UNAVAILABLE: &str = "M6_RECIPIENT_UNAVAILABLE";
const M4C05_MODEL_LEDGER_UNAVAILABLE: &str = "M4C05_MODEL_LEDGER_UNAVAILABLE";
const M4C05_MODEL_UNAVAILABLE: &str = "M4C05_MODEL_ADAPTER_UNAVAILABLE";
const M4C05_CONTEXT_UNAVAILABLE_REASON: &str = "秘书上下文暂不可用，请稍后重试。";
const M4C05_HOME_CONTEXT_UNAVAILABLE_CODE: &str = "M4C05_CONTEXT_UNAVAILABLE";
const M4C06_COORDINATION_REQUEST_INVALID: &str = "M4C06_COORDINATION_REQUEST_INVALID";
const M4C06_COORDINATION_UNAVAILABLE: &str = "M4C06_COORDINATION_UNAVAILABLE";
const M4C06_COORDINATION_OPERATION_FAILED: &str = "M4C06_COORDINATION_OPERATION_FAILED";
const M4R02_PERSONAL_OBJECT_REQUEST_INVALID: &str = "M4R02_PERSONAL_OBJECT_REQUEST_INVALID";
const M4R02_PERSONAL_OBJECT_UNAVAILABLE: &str = "M4R02_PERSONAL_OBJECT_UNAVAILABLE";
const M4R02_PERSONAL_OBJECT_OPERATION_FAILED: &str = "M4R02_PERSONAL_OBJECT_OPERATION_FAILED";

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

/// A home read is a ready/unavailable envelope. A ready envelope contains the
/// complete typed application outcome, not a renderer-assembled substitute.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) struct SecretaryHomeContextEnvelope {
    /// ready | unavailable
    pub(crate) status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) application_outcome: Option<M4SecretaryApplicationOutcome>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) reason: Option<String>,
}

impl SecretaryHomeContextEnvelope {
    fn ready(application_outcome: M4SecretaryApplicationOutcome) -> Self {
        Self {
            status: "ready".to_string(),
            application_outcome: Some(application_outcome),
            reason: None,
        }
    }

    fn unavailable() -> Self {
        Self {
            status: "unavailable".to_string(),
            application_outcome: None,
            reason: Some(M4C05_HOME_CONTEXT_UNAVAILABLE_CODE.to_string()),
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
/// malformed, or non-exact session becomes a scrubbed failure at the command
/// boundary.
struct OrdinaryProductRoleSessionReadPort<'a> {
    runtime: &'a crate::m3_role_session_read_model::M3RoleSessionReadRuntimeSlot,
}

fn ordinary_product_role_session_state(
    status: crate::m3_role_session_read_model::M3SecretaryRoleSessionStatusDto,
) -> Result<M4SecretaryRoleSessionState, M4SecretaryServiceError> {
    let invalid = || M4SecretaryServiceError::new(M4C05_RUNTIME_UNAVAILABLE);
    let identity = crate::mcp::identity_kernel::resolve_m4_primary_secretary_identity()
        .map_err(|_| invalid())?;
    let binding = identity
        .m3_server_resolved_binding()
        .map_err(|_| invalid())?;
    if status.actor_id != binding.actor_id.as_str()
        || status.role_ref != binding.role_ref.as_str()
        || status.scope_ref != binding.scope_ref.as_str()
        || status.current_object_ref != binding.current_object_ref.as_str()
        || status.execution_channel != binding.execution_channel.as_str()
        || status.permission_snapshot_ref != binding.permission_snapshot_ref.as_str()
        || status.owner_fingerprint != binding.owner_fingerprint.as_str()
    {
        return Err(invalid());
    }

    Ok(M4SecretaryRoleSessionState {
        role_session_ref: M4SecretaryOpaqueRef::new(status.role_session_id)
            .map_err(|_| invalid())?,
        role_ref: M4SecretaryTypedRef::new(
            crate::mcp::identity_kernel::M4_PRIMARY_SECRETARY_ROLE_ID,
        )
        .map_err(|_| invalid())?,
        scope_ref: M4SecretaryTypedRef::new(
            crate::mcp::identity_kernel::M4_PRIMARY_SECRETARY_SCOPE_ID,
        )
        .map_err(|_| invalid())?,
        current_object_ref: M4SecretaryTypedRef::new(
            crate::mcp::identity_kernel::M4_PRIMARY_SECRETARY_CURRENT_OBJECT_ID,
        )
        .map_err(|_| invalid())?,
        execution_channel_code: "DAILY".to_string(),
        session_state_code: status.session_state,
        permission_snapshot_ref: M4SecretaryOpaqueRef::new(status.permission_snapshot_ref)
            .map_err(|_| invalid())?,
        owner_fingerprint: M4SecretaryHash::new(status.owner_fingerprint).map_err(|_| invalid())?,
    })
}

impl M4SecretaryRoleSessionReadPort for OrdinaryProductRoleSessionReadPort<'_> {
    fn read_personal_secretary_role_session(
        &self,
    ) -> Result<M4SecretaryRoleSessionState, M4SecretaryServiceError> {
        let status = self
            .runtime
            .secretary_status()
            .map_err(|_| M4SecretaryServiceError::new(M4C05_RUNTIME_UNAVAILABLE))?;
        ordinary_product_role_session_state(status)
    }
}

#[cfg(not(test))]
struct OrdinaryProductCoordinationSnapshotReadPort<'a> {
    repository: &'a M4SecretarySqliteRepository,
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
/// The port remains explicit and returns only a bounded unavailable state.
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

/// The consult entry only invokes M4's RoleSession and Handoff paths.  This
/// explicit unused port proves it cannot accidentally read or mutate the M4
/// coordination projection while still traversing the accepted application
/// service boundary.
#[derive(Clone, Copy, Default)]
struct M6ConsultUnusedCoordinationPort;

impl M4SecretaryCoordinationSnapshotReadPort for M6ConsultUnusedCoordinationPort {
    fn read_coordination_snapshot(
        &self,
        _scope_ref: &M4SecretaryTypedRef,
    ) -> Result<M4CoordinationSnapshot, M4SecretaryServiceError> {
        Err(M4SecretaryServiceError::new(
            "M6_CONSULT_COORDINATION_PORT_MUST_NOT_BE_CALLED",
        ))
    }
}

pub(crate) fn start_global_supervisor_consult_for_state(
    state: &crate::AppState,
    request: &crate::m6_org_dto::M6OrgSecretaryConsultStartRequest,
    now_ms: i64,
) -> Result<crate::m6_org_consult_handoff::M6OrgSecretaryConsultCommandResponse, String> {
    let role_session_port = OrdinaryProductRoleSessionReadPort {
        runtime: &state.m3_role_session_read_runtime,
    };
    let coordination_port = M6ConsultUnusedCoordinationPort;
    let handoff_port = crate::m6_org_consult_handoff::M6OrgConsultHandoffAdapter::for_start(
        state,
        request.clone(),
        now_ms,
    );
    let invocation_ledger_port = OrdinaryProductUnavailableInvocationLedgerPort;
    let model_port = OrdinaryProductUnavailableModelPort;
    let m4_request = crate::m6_org_consult_handoff::m4_request_for_start(state, request, now_ms)?;
    let secretary_handoff = M4SecretaryApplicationService::new(
        &role_session_port,
        &coordination_port,
        &handoff_port,
        &invocation_ledger_port,
        &model_port,
    )
    .request_handoff(&m4_request)
    .map_err(|error| error.code)?;
    let consult = handoff_port.take_outcome()?;
    Ok(
        crate::m6_org_consult_handoff::M6OrgSecretaryConsultCommandResponse {
            consult,
            secretary_handoff,
        },
    )
}

pub(crate) fn read_global_supervisor_consult_for_state(
    state: &crate::AppState,
    request: &crate::m6_org_dto::M6OrgSecretaryConsultReadRequest,
    now_ms: i64,
) -> Result<crate::m6_org_consult_handoff::M6OrgSecretaryConsultCommandResponse, String> {
    let role_session_port = OrdinaryProductRoleSessionReadPort {
        runtime: &state.m3_role_session_read_runtime,
    };
    let coordination_port = M6ConsultUnusedCoordinationPort;
    let handoff_port = crate::m6_org_consult_handoff::M6OrgConsultHandoffAdapter::for_read(
        state,
        request.handoff_id.clone(),
        now_ms,
    );
    let invocation_ledger_port = OrdinaryProductUnavailableInvocationLedgerPort;
    let model_port = OrdinaryProductUnavailableModelPort;
    let handoff_ref =
        M4SecretaryOpaqueRef::new(request.handoff_id.clone()).map_err(|error| error.code)?;
    let secretary_handoff: M4SecretaryHandoffOutcome = M4SecretaryApplicationService::new(
        &role_session_port,
        &coordination_port,
        &handoff_port,
        &invocation_ledger_port,
        &model_port,
    )
    .read_handoff_receipt(&handoff_ref)
    .map_err(|error| error.code)?;
    let consult = handoff_port.take_outcome()?;
    Ok(
        crate::m6_org_consult_handoff::M6OrgSecretaryConsultCommandResponse {
            consult,
            secretary_handoff,
        },
    )
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

/// The ordinary product also has no controlled M6 model adapter. Home loads
/// and the existing explanation command never reach this port.
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

fn read_secretary_application_outcome_with_ports<RS, CS, HP, LP, MP>(
    role_session_port: &RS,
    coordination_port: &CS,
    handoff_port: &HP,
    invocation_ledger_port: &LP,
    model_port: &MP,
) -> Result<M4SecretaryApplicationOutcome, M4SecretaryServiceError>
where
    RS: M4SecretaryRoleSessionReadPort,
    CS: M4SecretaryCoordinationSnapshotReadPort,
    HP: M4SecretaryHandoffPort,
    LP: M4SecretaryModelInvocationLedgerPort,
    MP: M4SecretaryControlledModelEnhancementPort,
{
    M4SecretaryApplicationService::new(
        role_session_port,
        coordination_port,
        handoff_port,
        invocation_ledger_port,
        model_port,
    )
    .read_deterministic_brief()
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
    match read_secretary_application_outcome_with_ports(
        role_session_port,
        coordination_port,
        handoff_port,
        invocation_ledger_port,
        model_port,
    ) {
        Ok(application_outcome) => SecretaryExplainOutcome::ready(&application_outcome),
        Err(_) => SecretaryExplainOutcome::unavailable(),
    }
}

fn load_secretary_home_context_with_ports<RS, CS, HP, LP, MP>(
    role_session_port: &RS,
    coordination_port: &CS,
    handoff_port: &HP,
    invocation_ledger_port: &LP,
    model_port: &MP,
) -> SecretaryHomeContextEnvelope
where
    RS: M4SecretaryRoleSessionReadPort,
    CS: M4SecretaryCoordinationSnapshotReadPort,
    HP: M4SecretaryHandoffPort,
    LP: M4SecretaryModelInvocationLedgerPort,
    MP: M4SecretaryControlledModelEnhancementPort,
{
    match read_secretary_application_outcome_with_ports(
        role_session_port,
        coordination_port,
        handoff_port,
        invocation_ledger_port,
        model_port,
    ) {
        Ok(application_outcome) => SecretaryHomeContextEnvelope::ready(application_outcome),
        Err(_) => SecretaryHomeContextEnvelope::unavailable(),
    }
}

#[cfg(not(test))]
fn load_secretary_home_context_from_server_state(
    runtime: crate::m3_role_session_read_model::M3RoleSessionReadRuntimeSlot,
    repository: M4SecretarySqliteRepository,
) -> SecretaryHomeContextEnvelope {
    let role_session_port = OrdinaryProductRoleSessionReadPort { runtime: &runtime };
    let coordination_port = OrdinaryProductCoordinationSnapshotReadPort {
        repository: &repository,
    };
    let handoff_port = OrdinaryProductUnavailableHandoffPort;
    let invocation_ledger_port = OrdinaryProductUnavailableInvocationLedgerPort;
    let model_port = OrdinaryProductUnavailableModelPort;
    load_secretary_home_context_with_ports(
        &role_session_port,
        &coordination_port,
        &handoff_port,
        &invocation_ledger_port,
        &model_port,
    )
}

#[cfg(not(test))]
fn run_secretary_explain_from_server_state(
    runtime: crate::m3_role_session_read_model::M3RoleSessionReadRuntimeSlot,
    repository: M4SecretarySqliteRepository,
) -> SecretaryExplainOutcome {
    let home_context = load_secretary_home_context_from_server_state(runtime, repository);
    match home_context.application_outcome {
        Some(application_outcome) => SecretaryExplainOutcome::ready(&application_outcome),
        None => SecretaryExplainOutcome::unavailable(),
    }
}

#[tauri::command]
pub(crate) async fn run_secretary_explain(
    state: tauri::State<'_, crate::AppState>,
) -> Result<SecretaryExplainOutcome, String> {
    #[cfg(not(test))]
    {
        let runtime = state.m3_role_session_read_runtime.clone();
        let Some(repository) = state.m4_secretary_repository.clone() else {
            return Ok(SecretaryExplainOutcome::unavailable());
        };
        return Ok(tauri::async_runtime::spawn_blocking(move || {
            run_secretary_explain_from_server_state(runtime, repository)
        })
        .await
        .unwrap_or_else(|_| SecretaryExplainOutcome::unavailable()));
    }
    #[cfg(test)]
    {
        let _ = state;
        Ok(SecretaryExplainOutcome::unavailable())
    }
}

/// Loads server-resolved Secretary context for the home surface. The renderer
/// supplies neither identity nor scope and this path cannot invoke a model.
#[tauri::command]
pub(crate) async fn load_secretary_home_context(
    state: tauri::State<'_, crate::AppState>,
) -> Result<SecretaryHomeContextEnvelope, String> {
    #[cfg(not(test))]
    {
        // The isolated R06 driver consumes one existing unavailable envelope
        // before the ordinary Home read. That makes the ordinary App take its
        // guarded legacy-report fallback path without assigning a production
        // failure cause to the synthetic trigger.
        if crate::m4r06_ordinary_legacy_read_driver::consume_synthetic_home_unavailable_trigger()? {
            return Ok(SecretaryHomeContextEnvelope::unavailable());
        }
        let runtime = state.m3_role_session_read_runtime.clone();
        let Some(repository) = state.m4_secretary_repository.clone() else {
            return Ok(SecretaryHomeContextEnvelope::unavailable());
        };
        return Ok(tauri::async_runtime::spawn_blocking(move || {
            load_secretary_home_context_from_server_state(runtime, repository)
        })
        .await
        .unwrap_or_else(|_| SecretaryHomeContextEnvelope::unavailable()));
    }
    #[cfg(test)]
    {
        let _ = state;
        Ok(SecretaryHomeContextEnvelope::unavailable())
    }
}

/// Only the local C04 coordination transitions exposed here may be requested
/// from the renderer. This enum intentionally excludes all source-owner,
/// PersonalAction, callback, path, and payload operations.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum SecretaryCoordinationAction {
    InboxMarkRead,
    InboxDismiss,
    OpenLoopAcknowledge,
    OpenLoopSnooze,
    OpenLoopClose,
    OpenLoopDismiss,
    OpenLoopReopen,
    OpenLoopCarryOver,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub(crate) struct SecretaryCoordinationRequest {
    pub(crate) action: SecretaryCoordinationAction,
    pub(crate) item_ref: String,
    /// Canonical decimal string. Never accept a renderer number here.
    pub(crate) expected_revision: String,
    pub(crate) idempotency_key: String,
    #[serde(default)]
    pub(crate) snoozed_until_utc: Option<String>,
}

#[derive(Clone, Debug)]
struct ValidatedSecretaryCoordinationRequest {
    action: SecretaryCoordinationAction,
    item_ref: M4SecretaryTypedRef,
    expected_revision: u64,
    idempotency_key: M4SecretaryOpaqueRef,
    snoozed_until_utc: Option<String>,
}

fn validate_secretary_coordination_request(
    request: SecretaryCoordinationRequest,
) -> Result<ValidatedSecretaryCoordinationRequest, String> {
    let item_ref = M4SecretaryTypedRef::new(request.item_ref)
        .map_err(|_| M4C06_COORDINATION_REQUEST_INVALID.to_string())?;
    let expected_revision = parse_canonical_coordination_revision(&request.expected_revision)?;
    let idempotency_key = M4SecretaryOpaqueRef::new(request.idempotency_key)
        .map_err(|_| M4C06_COORDINATION_REQUEST_INVALID.to_string())?;
    let is_inbox_action = matches!(
        request.action,
        SecretaryCoordinationAction::InboxMarkRead | SecretaryCoordinationAction::InboxDismiss
    );
    let is_open_loop_action = !is_inbox_action;
    if (is_inbox_action && !item_ref.as_str().starts_with("inbox:"))
        || (is_open_loop_action && !item_ref.as_str().starts_with("open-loop:"))
    {
        return Err(M4C06_COORDINATION_REQUEST_INVALID.to_string());
    }

    match (&request.action, request.snoozed_until_utc.as_deref()) {
        (SecretaryCoordinationAction::OpenLoopSnooze, Some(snoozed_until_utc))
            if crate::m4_secretary_domain::m4_parse_rfc3339_utc_key(snoozed_until_utc)
                .is_some() => {}
        (SecretaryCoordinationAction::OpenLoopSnooze, _) => {
            return Err(M4C06_COORDINATION_REQUEST_INVALID.to_string())
        }
        (_, None) => {}
        (_, Some(_)) => return Err(M4C06_COORDINATION_REQUEST_INVALID.to_string()),
    }

    Ok(ValidatedSecretaryCoordinationRequest {
        action: request.action,
        item_ref,
        expected_revision,
        idempotency_key,
        snoozed_until_utc: request.snoozed_until_utc,
    })
}

fn parse_canonical_coordination_revision(value: &str) -> Result<u64, String> {
    if value.is_empty()
        || (value.len() > 1 && value.starts_with('0'))
        || !value.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(M4C06_COORDINATION_REQUEST_INVALID.to_string());
    }
    value
        .parse::<u64>()
        .map_err(|_| M4C06_COORDINATION_REQUEST_INVALID.to_string())
}

/// The bridge port mirrors exactly the existing C04 repository capabilities
/// selected for the home surface. It has no source-owner or personal-action
/// method.
trait SecretaryCoordinationPort {
    fn mark_inbox_item_read(
        &self,
        inbox_item_id: &str,
        expected_revision: u64,
        idempotency_key: &str,
    ) -> Result<M4CoordinationCommandOutcome, String>;

    fn dismiss_inbox_item(
        &self,
        inbox_item_id: &str,
        expected_revision: u64,
        idempotency_key: &str,
    ) -> Result<M4CoordinationCommandOutcome, String>;

    fn acknowledge_open_loop(
        &self,
        open_loop_id: &str,
        expected_revision: u64,
        idempotency_key: &str,
    ) -> Result<M4CoordinationCommandOutcome, String>;

    fn snooze_open_loop(
        &self,
        open_loop_id: &str,
        expected_revision: u64,
        snoozed_until_utc: &str,
        idempotency_key: &str,
    ) -> Result<M4CoordinationCommandOutcome, String>;

    fn close_open_loop(
        &self,
        open_loop_id: &str,
        expected_revision: u64,
        idempotency_key: &str,
    ) -> Result<M4CoordinationCommandOutcome, String>;

    fn dismiss_open_loop(
        &self,
        open_loop_id: &str,
        expected_revision: u64,
        idempotency_key: &str,
    ) -> Result<M4CoordinationCommandOutcome, String>;

    fn reopen_open_loop(
        &self,
        open_loop_id: &str,
        expected_revision: u64,
        idempotency_key: &str,
    ) -> Result<M4CoordinationCommandOutcome, String>;

    fn carry_over_open_loop(
        &self,
        open_loop_id: &str,
        expected_revision: u64,
        idempotency_key: &str,
    ) -> Result<M4CoordinationCommandOutcome, String>;
}

#[cfg(not(test))]
impl SecretaryCoordinationPort for M4SecretarySqliteRepository {
    fn mark_inbox_item_read(
        &self,
        inbox_item_id: &str,
        expected_revision: u64,
        idempotency_key: &str,
    ) -> Result<M4CoordinationCommandOutcome, String> {
        M4SecretarySqliteRepository::mark_inbox_item_read(
            self,
            inbox_item_id,
            expected_revision,
            idempotency_key,
        )
        .map_err(|_| M4C06_COORDINATION_OPERATION_FAILED.to_string())
    }

    fn dismiss_inbox_item(
        &self,
        inbox_item_id: &str,
        expected_revision: u64,
        idempotency_key: &str,
    ) -> Result<M4CoordinationCommandOutcome, String> {
        M4SecretarySqliteRepository::dismiss_inbox_item(
            self,
            inbox_item_id,
            expected_revision,
            idempotency_key,
        )
        .map_err(|_| M4C06_COORDINATION_OPERATION_FAILED.to_string())
    }

    fn acknowledge_open_loop(
        &self,
        open_loop_id: &str,
        expected_revision: u64,
        idempotency_key: &str,
    ) -> Result<M4CoordinationCommandOutcome, String> {
        M4SecretarySqliteRepository::acknowledge_open_loop(
            self,
            open_loop_id,
            expected_revision,
            idempotency_key,
        )
        .map_err(|_| M4C06_COORDINATION_OPERATION_FAILED.to_string())
    }

    fn snooze_open_loop(
        &self,
        open_loop_id: &str,
        expected_revision: u64,
        snoozed_until_utc: &str,
        idempotency_key: &str,
    ) -> Result<M4CoordinationCommandOutcome, String> {
        M4SecretarySqliteRepository::snooze_open_loop(
            self,
            open_loop_id,
            expected_revision,
            snoozed_until_utc,
            idempotency_key,
        )
        .map_err(|_| M4C06_COORDINATION_OPERATION_FAILED.to_string())
    }

    fn close_open_loop(
        &self,
        open_loop_id: &str,
        expected_revision: u64,
        idempotency_key: &str,
    ) -> Result<M4CoordinationCommandOutcome, String> {
        M4SecretarySqliteRepository::close_open_loop(
            self,
            open_loop_id,
            expected_revision,
            idempotency_key,
        )
        .map_err(|_| M4C06_COORDINATION_OPERATION_FAILED.to_string())
    }

    fn dismiss_open_loop(
        &self,
        open_loop_id: &str,
        expected_revision: u64,
        idempotency_key: &str,
    ) -> Result<M4CoordinationCommandOutcome, String> {
        M4SecretarySqliteRepository::dismiss_open_loop(
            self,
            open_loop_id,
            expected_revision,
            idempotency_key,
        )
        .map_err(|_| M4C06_COORDINATION_OPERATION_FAILED.to_string())
    }

    fn reopen_open_loop(
        &self,
        open_loop_id: &str,
        expected_revision: u64,
        idempotency_key: &str,
    ) -> Result<M4CoordinationCommandOutcome, String> {
        M4SecretarySqliteRepository::reopen_open_loop(
            self,
            open_loop_id,
            expected_revision,
            idempotency_key,
        )
        .map_err(|_| M4C06_COORDINATION_OPERATION_FAILED.to_string())
    }

    fn carry_over_open_loop(
        &self,
        open_loop_id: &str,
        expected_revision: u64,
        idempotency_key: &str,
    ) -> Result<M4CoordinationCommandOutcome, String> {
        M4SecretarySqliteRepository::carry_over_open_loop(
            self,
            open_loop_id,
            expected_revision,
            idempotency_key,
        )
        .map_err(|_| M4C06_COORDINATION_OPERATION_FAILED.to_string())
    }
}

/// The receipt contains only repository-minted refs, codes, a canonical
/// revision string, and replay state. It contains no source-owner fact or
/// source payload.
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) struct SecretaryCoordinationReceipt {
    pub(crate) command_receipt_ref: M4SecretaryOpaqueRef,
    pub(crate) coordination_event_ref: M4SecretaryOpaqueRef,
    pub(crate) aggregate_kind_code: String,
    pub(crate) item_ref: M4SecretaryTypedRef,
    pub(crate) coordination_revision: String,
    pub(crate) outcome_code: String,
    pub(crate) replayed: bool,
}

impl SecretaryCoordinationReceipt {
    fn from_repository_outcome(outcome: M4CoordinationCommandOutcome) -> Result<Self, String> {
        if !is_coordination_code(&outcome.aggregate_kind)
            || !is_coordination_code(&outcome.outcome_code)
        {
            return Err(M4C06_COORDINATION_OPERATION_FAILED.to_string());
        }
        Ok(Self {
            command_receipt_ref: M4SecretaryOpaqueRef::new(outcome.command_receipt_id)
                .map_err(|_| M4C06_COORDINATION_OPERATION_FAILED.to_string())?,
            coordination_event_ref: M4SecretaryOpaqueRef::new(outcome.coordination_event_id)
                .map_err(|_| M4C06_COORDINATION_OPERATION_FAILED.to_string())?,
            aggregate_kind_code: outcome.aggregate_kind,
            item_ref: M4SecretaryTypedRef::new(outcome.aggregate_id)
                .map_err(|_| M4C06_COORDINATION_OPERATION_FAILED.to_string())?,
            coordination_revision: canonical_revision_string(&outcome.aggregate_revision)
                .map_err(|_| M4C06_COORDINATION_OPERATION_FAILED.to_string())?,
            outcome_code: outcome.outcome_code,
            replayed: outcome.replayed,
        })
    }
}

fn is_coordination_code(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 96
        && value
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
}

fn canonical_revision_string(value: &str) -> Result<String, ()> {
    parse_canonical_coordination_revision(value)
        .map(|value| value.to_string())
        .map_err(|_| ())
}

fn operate_secretary_coordination_with_port<P: SecretaryCoordinationPort>(
    port: &P,
    request: SecretaryCoordinationRequest,
) -> Result<SecretaryCoordinationReceipt, String> {
    let request = validate_secretary_coordination_request(request)?;
    let item_ref = request.item_ref.as_str();
    let idempotency_key = request.idempotency_key.as_str();
    let outcome = match request.action {
        SecretaryCoordinationAction::InboxMarkRead => {
            port.mark_inbox_item_read(item_ref, request.expected_revision, idempotency_key)
        }
        SecretaryCoordinationAction::InboxDismiss => {
            port.dismiss_inbox_item(item_ref, request.expected_revision, idempotency_key)
        }
        SecretaryCoordinationAction::OpenLoopAcknowledge => {
            port.acknowledge_open_loop(item_ref, request.expected_revision, idempotency_key)
        }
        SecretaryCoordinationAction::OpenLoopSnooze => port.snooze_open_loop(
            item_ref,
            request.expected_revision,
            request
                .snoozed_until_utc
                .as_deref()
                .expect("validated snooze request"),
            idempotency_key,
        ),
        SecretaryCoordinationAction::OpenLoopClose => {
            port.close_open_loop(item_ref, request.expected_revision, idempotency_key)
        }
        SecretaryCoordinationAction::OpenLoopDismiss => {
            port.dismiss_open_loop(item_ref, request.expected_revision, idempotency_key)
        }
        SecretaryCoordinationAction::OpenLoopReopen => {
            port.reopen_open_loop(item_ref, request.expected_revision, idempotency_key)
        }
        SecretaryCoordinationAction::OpenLoopCarryOver => {
            port.carry_over_open_loop(item_ref, request.expected_revision, idempotency_key)
        }
    }?;
    SecretaryCoordinationReceipt::from_repository_outcome(outcome)
}

#[cfg(not(test))]
fn operate_secretary_coordination_from_server_state(
    runtime: crate::m3_role_session_read_model::M3RoleSessionReadRuntimeSlot,
    repository: M4SecretarySqliteRepository,
    request: SecretaryCoordinationRequest,
) -> Result<SecretaryCoordinationReceipt, String> {
    let role_session_port = OrdinaryProductRoleSessionReadPort { runtime: &runtime };
    let coordination_port = OrdinaryProductCoordinationSnapshotReadPort {
        repository: &repository,
    };
    let handoff_port = OrdinaryProductUnavailableHandoffPort;
    let invocation_ledger_port = OrdinaryProductUnavailableInvocationLedgerPort;
    let model_port = OrdinaryProductUnavailableModelPort;
    read_secretary_application_outcome_with_ports(
        &role_session_port,
        &coordination_port,
        &handoff_port,
        &invocation_ledger_port,
        &model_port,
    )
    .map_err(|_| M4C06_COORDINATION_UNAVAILABLE.to_string())?;
    operate_secretary_coordination_with_port(&repository, request)
}

/// Applies one existing local C04 coordination transition. The M3 identity
/// and M4 snapshot are revalidated from server state before the repository
/// transition, and all blocking SQLite work runs off the async UI thread.
#[tauri::command]
pub(crate) async fn operate_secretary_coordination(
    state: tauri::State<'_, crate::AppState>,
    request: SecretaryCoordinationRequest,
) -> Result<SecretaryCoordinationReceipt, String> {
    #[cfg(not(test))]
    {
        let runtime = state.m3_role_session_read_runtime.clone();
        let repository = state
            .m4_secretary_repository
            .clone()
            .ok_or_else(|| M4C06_COORDINATION_UNAVAILABLE.to_string())?;
        return tauri::async_runtime::spawn_blocking(move || {
            operate_secretary_coordination_from_server_state(runtime, repository, request)
        })
        .await
        .map_err(|_| M4C06_COORDINATION_UNAVAILABLE.to_string())?;
    }
    #[cfg(test)]
    {
        let _ = (state, request);
        Err(M4C06_COORDINATION_UNAVAILABLE.to_string())
    }
}

/// Ordinary user operations for M4-owned local objects.  The tagged sum is
/// intentionally closed: there is no Reminder fire, Notification create or
/// deliver, Decision owner transition, identity, scope, source payload, path,
/// provider or connector field.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(
    tag = "action",
    rename_all = "SCREAMING_SNAKE_CASE",
    deny_unknown_fields
)]
pub(crate) enum SecretaryPersonalObjectRequest {
    PersonalActionCreate {
        title: String,
        #[serde(default)]
        due_at_utc: Option<String>,
        idempotency_key: String,
    },
    PersonalActionComplete {
        item_ref: String,
        expected_revision: String,
        idempotency_key: String,
    },
    PersonalActionCancel {
        item_ref: String,
        expected_revision: String,
        idempotency_key: String,
    },
    PersonalActionReopen {
        item_ref: String,
        expected_revision: String,
        idempotency_key: String,
    },
    ReminderCreate {
        owner_ref: String,
        scheduled_for_utc: String,
        iana_timezone: String,
        idempotency_key: String,
    },
    ReminderSnooze {
        item_ref: String,
        expected_revision: String,
        snoozed_until_utc: String,
        idempotency_key: String,
    },
    ReminderDismiss {
        item_ref: String,
        expected_revision: String,
        idempotency_key: String,
    },
    ReminderCancel {
        item_ref: String,
        expected_revision: String,
        idempotency_key: String,
    },
    NotificationRead {
        item_ref: String,
        expected_revision: String,
        idempotency_key: String,
    },
    NotificationDismiss {
        item_ref: String,
        expected_revision: String,
        idempotency_key: String,
    },
    DecisionRead {
        item_ref: String,
        expected_revision: String,
        idempotency_key: String,
    },
    DecisionDismiss {
        item_ref: String,
        expected_revision: String,
        idempotency_key: String,
    },
}

fn validate_secretary_personal_object_request(
    request: &SecretaryPersonalObjectRequest,
) -> Result<(), String> {
    let invalid = || M4R02_PERSONAL_OBJECT_REQUEST_INVALID.to_string();
    let validate_idempotency = |value: &str| {
        M4SecretaryOpaqueRef::new(value.to_string())
            .map(|_| ())
            .map_err(|_| invalid())
    };
    let validate_ref_revision = |item_ref: &str, prefix: &str, revision: &str| {
        let item_ref = M4SecretaryTypedRef::new(item_ref.to_string()).map_err(|_| invalid())?;
        if !item_ref.as_str().starts_with(prefix) {
            return Err(invalid());
        }
        parse_canonical_coordination_revision(revision).map(|_| ())
    };
    match request {
        SecretaryPersonalObjectRequest::PersonalActionCreate {
            title,
            due_at_utc,
            idempotency_key,
        } => {
            if title.is_empty()
                || title.trim() != title
                || title.chars().count() > 160
                || title.chars().any(char::is_control)
                || due_at_utc.as_deref().is_some_and(|value| {
                    crate::m4_secretary_domain::m4_parse_rfc3339_utc_key(value).is_none()
                })
            {
                return Err(invalid());
            }
            validate_idempotency(idempotency_key)
        }
        SecretaryPersonalObjectRequest::PersonalActionComplete {
            item_ref,
            expected_revision,
            idempotency_key,
        }
        | SecretaryPersonalObjectRequest::PersonalActionCancel {
            item_ref,
            expected_revision,
            idempotency_key,
        }
        | SecretaryPersonalObjectRequest::PersonalActionReopen {
            item_ref,
            expected_revision,
            idempotency_key,
        } => {
            validate_ref_revision(item_ref, "personal-action:", expected_revision)?;
            validate_idempotency(idempotency_key)
        }
        SecretaryPersonalObjectRequest::ReminderCreate {
            owner_ref,
            scheduled_for_utc,
            iana_timezone,
            idempotency_key,
        } => {
            M4SecretaryTypedRef::new(owner_ref.to_string()).map_err(|_| invalid())?;
            if crate::m4_secretary_domain::m4_parse_rfc3339_utc_key(scheduled_for_utc).is_none()
                || iana_timezone.is_empty()
                || iana_timezone.len() > 128
            {
                return Err(invalid());
            }
            validate_idempotency(idempotency_key)
        }
        SecretaryPersonalObjectRequest::ReminderSnooze {
            item_ref,
            expected_revision,
            snoozed_until_utc,
            idempotency_key,
        } => {
            validate_ref_revision(item_ref, "reminder:", expected_revision)?;
            if crate::m4_secretary_domain::m4_parse_rfc3339_utc_key(snoozed_until_utc).is_none() {
                return Err(invalid());
            }
            validate_idempotency(idempotency_key)
        }
        SecretaryPersonalObjectRequest::ReminderDismiss {
            item_ref,
            expected_revision,
            idempotency_key,
        }
        | SecretaryPersonalObjectRequest::ReminderCancel {
            item_ref,
            expected_revision,
            idempotency_key,
        } => {
            validate_ref_revision(item_ref, "reminder:", expected_revision)?;
            validate_idempotency(idempotency_key)
        }
        SecretaryPersonalObjectRequest::NotificationRead {
            item_ref,
            expected_revision,
            idempotency_key,
        }
        | SecretaryPersonalObjectRequest::NotificationDismiss {
            item_ref,
            expected_revision,
            idempotency_key,
        } => {
            validate_ref_revision(item_ref, "notification:", expected_revision)?;
            validate_idempotency(idempotency_key)
        }
        SecretaryPersonalObjectRequest::DecisionRead {
            item_ref,
            expected_revision,
            idempotency_key,
        }
        | SecretaryPersonalObjectRequest::DecisionDismiss {
            item_ref,
            expected_revision,
            idempotency_key,
        } => {
            validate_ref_revision(item_ref, "decision-projection:", expected_revision)?;
            validate_idempotency(idempotency_key)
        }
    }
}

#[cfg(not(test))]
fn operate_secretary_personal_object_from_server_state(
    runtime: crate::m3_role_session_read_model::M3RoleSessionReadRuntimeSlot,
    repository: M4SecretarySqliteRepository,
    request: SecretaryPersonalObjectRequest,
) -> Result<SecretaryCoordinationReceipt, String> {
    validate_secretary_personal_object_request(&request)?;
    let role_session_port = OrdinaryProductRoleSessionReadPort { runtime: &runtime };
    let coordination_port = OrdinaryProductCoordinationSnapshotReadPort {
        repository: &repository,
    };
    let handoff_port = OrdinaryProductUnavailableHandoffPort;
    let invocation_ledger_port = OrdinaryProductUnavailableInvocationLedgerPort;
    let model_port = OrdinaryProductUnavailableModelPort;
    let authorized = read_secretary_application_outcome_with_ports(
        &role_session_port,
        &coordination_port,
        &handoff_port,
        &invocation_ledger_port,
        &model_port,
    )
    .map_err(|_| M4R02_PERSONAL_OBJECT_UNAVAILABLE.to_string())?;

    let parse_revision = |value: &str| {
        parse_canonical_coordination_revision(value)
            .map_err(|_| M4R02_PERSONAL_OBJECT_REQUEST_INVALID.to_string())
    };
    let operation = match request {
        SecretaryPersonalObjectRequest::PersonalActionCreate {
            title,
            due_at_utc,
            idempotency_key,
        } => repository.create_personal_action(&title, due_at_utc.as_deref(), &idempotency_key),
        SecretaryPersonalObjectRequest::PersonalActionComplete {
            item_ref,
            expected_revision,
            idempotency_key,
        } => repository.complete_personal_action(
            &item_ref,
            parse_revision(&expected_revision)?,
            &idempotency_key,
        ),
        SecretaryPersonalObjectRequest::PersonalActionCancel {
            item_ref,
            expected_revision,
            idempotency_key,
        } => repository.cancel_personal_action(
            &item_ref,
            parse_revision(&expected_revision)?,
            &idempotency_key,
        ),
        SecretaryPersonalObjectRequest::PersonalActionReopen {
            item_ref,
            expected_revision,
            idempotency_key,
        } => repository.reopen_personal_action(
            &item_ref,
            parse_revision(&expected_revision)?,
            &idempotency_key,
        ),
        SecretaryPersonalObjectRequest::ReminderCreate {
            owner_ref,
            scheduled_for_utc,
            iana_timezone,
            idempotency_key,
        } => {
            let owner_is_authorized = authorized
                .local_objects
                .personal_actions
                .iter()
                .any(|item| item.personal_action_id == owner_ref)
                || authorized
                    .local_objects
                    .reminder_owner_refs
                    .iter()
                    .any(|candidate| candidate == &owner_ref);
            if !owner_is_authorized {
                return Err(M4R02_PERSONAL_OBJECT_REQUEST_INVALID.to_string());
            }
            repository.create_reminder(
                &owner_ref,
                &scheduled_for_utc,
                &iana_timezone,
                &idempotency_key,
            )
        }
        SecretaryPersonalObjectRequest::ReminderSnooze {
            item_ref,
            expected_revision,
            snoozed_until_utc,
            idempotency_key,
        } => repository.snooze_reminder(
            &item_ref,
            parse_revision(&expected_revision)?,
            &snoozed_until_utc,
            &idempotency_key,
        ),
        SecretaryPersonalObjectRequest::ReminderDismiss {
            item_ref,
            expected_revision,
            idempotency_key,
        } => repository.dismiss_reminder(
            &item_ref,
            parse_revision(&expected_revision)?,
            &idempotency_key,
        ),
        SecretaryPersonalObjectRequest::ReminderCancel {
            item_ref,
            expected_revision,
            idempotency_key,
        } => repository.cancel_reminder(
            &item_ref,
            parse_revision(&expected_revision)?,
            &idempotency_key,
        ),
        SecretaryPersonalObjectRequest::NotificationRead {
            item_ref,
            expected_revision,
            idempotency_key,
        } => repository.mark_notification_read(
            &item_ref,
            parse_revision(&expected_revision)?,
            &idempotency_key,
        ),
        SecretaryPersonalObjectRequest::NotificationDismiss {
            item_ref,
            expected_revision,
            idempotency_key,
        } => repository.dismiss_notification(
            &item_ref,
            parse_revision(&expected_revision)?,
            &idempotency_key,
        ),
        SecretaryPersonalObjectRequest::DecisionRead {
            item_ref,
            expected_revision,
            idempotency_key,
        } => repository.mark_decision_read(
            &item_ref,
            parse_revision(&expected_revision)?,
            &idempotency_key,
        ),
        SecretaryPersonalObjectRequest::DecisionDismiss {
            item_ref,
            expected_revision,
            idempotency_key,
        } => repository.dismiss_decision(
            &item_ref,
            parse_revision(&expected_revision)?,
            &idempotency_key,
        ),
    }
    .map_err(|_| M4R02_PERSONAL_OBJECT_OPERATION_FAILED.to_string())?;
    SecretaryCoordinationReceipt::from_repository_outcome(operation)
        .map_err(|_| M4R02_PERSONAL_OBJECT_OPERATION_FAILED.to_string())
}

#[tauri::command]
pub(crate) async fn operate_secretary_personal_object(
    state: tauri::State<'_, crate::AppState>,
    request: SecretaryPersonalObjectRequest,
) -> Result<SecretaryCoordinationReceipt, String> {
    #[cfg(not(test))]
    {
        let runtime = state.m3_role_session_read_runtime.clone();
        let repository = state
            .m4_secretary_repository
            .clone()
            .ok_or_else(|| M4R02_PERSONAL_OBJECT_UNAVAILABLE.to_string())?;
        return tauri::async_runtime::spawn_blocking(move || {
            operate_secretary_personal_object_from_server_state(runtime, repository, request)
        })
        .await
        .map_err(|_| M4R02_PERSONAL_OBJECT_UNAVAILABLE.to_string())?;
    }
    #[cfg(test)]
    {
        let _ = (state, request);
        Err(M4R02_PERSONAL_OBJECT_UNAVAILABLE.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::cell::{Cell, RefCell};

    fn fixture_hash(character: char) -> String {
        format!("{:064x}", u32::from(character))
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

    fn exact_ordinary_role_session_status(
    ) -> crate::m3_role_session_read_model::M3SecretaryRoleSessionStatusDto {
        let identity = crate::mcp::identity_kernel::resolve_m4_primary_secretary_identity()
            .expect("fixed Secretary identity");
        let binding = identity
            .m3_server_resolved_binding()
            .expect("fixed M3 binding");
        crate::m3_role_session_read_model::M3SecretaryRoleSessionStatusDto {
            host: "SECRETARY".to_string(),
            role_session_id: fixture_opaque_ref("session", '9').as_str().to_string(),
            session_revision: 1,
            session_state: "ACTIVE".to_string(),
            actor_id: binding.actor_id.as_str().to_string(),
            role_ref: binding.role_ref.as_str().to_string(),
            scope_ref: binding.scope_ref.as_str().to_string(),
            current_object_ref: binding.current_object_ref.as_str().to_string(),
            execution_channel: binding.execution_channel.as_str().to_string(),
            permission_snapshot_ref: binding.permission_snapshot_ref.as_str().to_string(),
            owner_fingerprint: binding.owner_fingerprint.as_str().to_string(),
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
                revision: "1".to_string(),
            }],
            notifications: Vec::new(),
            reminders: Vec::new(),
            decisions: Vec::new(),
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
    fn m4r02_ordinary_adapter_validates_sealed_m3_binding_before_typed_projection() {
        let projected = ordinary_product_role_session_state(exact_ordinary_role_session_status())
            .expect("exact ordinary M3 binding projects to the fixed M4 identity");
        assert_eq!(
            projected.role_ref.as_str(),
            crate::mcp::identity_kernel::M4_PRIMARY_SECRETARY_ROLE_ID
        );
        assert_eq!(
            projected.scope_ref.as_str(),
            crate::mcp::identity_kernel::M4_PRIMARY_SECRETARY_SCOPE_ID
        );
        assert_eq!(
            projected.current_object_ref.as_str(),
            crate::mcp::identity_kernel::M4_PRIMARY_SECRETARY_CURRENT_OBJECT_ID
        );
        assert_eq!(projected.execution_channel_code, "DAILY");
        assert_eq!(projected.session_state_code, "ACTIVE");

        let mut mismatched = exact_ordinary_role_session_status();
        mismatched.scope_ref = fixture_opaque_ref("scope", '8').as_str().to_string();
        assert_eq!(
            ordinary_product_role_session_state(mismatched)
                .expect_err("a different sealed M3 scope must fail closed")
                .code,
            M4C05_RUNTIME_UNAVAILABLE
        );
    }

    #[test]
    fn m4r02_home_unavailable_envelope_serializes_stable_machine_code() {
        assert_eq!(
            serde_json::to_value(SecretaryHomeContextEnvelope::unavailable())
                .expect("serialize unavailable Home envelope"),
            json!({
                "status": "unavailable",
                "reason": M4C05_HOME_CONTEXT_UNAVAILABLE_CODE,
            })
        );
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

    #[test]
    fn m4c06_load_home_context_is_ready_and_zero_model() {
        let role_session_port = StaticRoleSessionPort {
            result: Ok(fixture_role_session()),
        };
        let coordination_port = StaticCoordinationPort {
            result: Ok(fixture_snapshot()),
        };
        let handoff_port = CountingHandoffPort::default();
        let invocation_ledger_port = CountingInvocationLedgerPort::default();
        let model_port = CountingModelPort::default();

        let envelope = load_secretary_home_context_with_ports(
            &role_session_port,
            &coordination_port,
            &handoff_port,
            &invocation_ledger_port,
            &model_port,
        );
        assert_eq!(envelope.status, "ready");
        let application_outcome = envelope
            .application_outcome
            .expect("ready envelope carries application outcome");
        assert_eq!(
            application_outcome.local_objects.personal_actions[0].title,
            "private user text must not appear"
        );
        assert!(
            !serde_json::to_string(&application_outcome.deterministic_brief)
                .expect("serialize deterministic brief")
                .contains("private user text"),
            "PersonalAction title is renderer-local and absent from the model-facing brief"
        );
        assert_eq!(handoff_port.calls.get(), 0);
        assert_eq!(invocation_ledger_port.calls.get(), 0);
        assert_eq!(model_port.calls.get(), 0);
    }

    #[test]
    fn m4c06_request_matrix_rejects_unknown_fields_and_accepts_full_u64() {
        let base = json!({
            "action": "OPEN_LOOP_SNOOZE",
            "item_ref": format!("open-loop:{}", fixture_hash('7')),
            "expected_revision": u64::MAX.to_string(),
            "idempotency_key": fixture_opaque_ref("idempotency", '8').as_str(),
            "snoozed_until_utc": "2026-08-10T13:00:00Z"
        });
        let request: SecretaryCoordinationRequest =
            serde_json::from_value(base).expect("deserialize exact request");
        assert_eq!(
            validate_secretary_coordination_request(request)
                .expect("full u64 accepted")
                .expected_revision,
            u64::MAX
        );

        let unknown = json!({
            "action": "OPEN_LOOP_CLOSE",
            "item_ref": format!("open-loop:{}", fixture_hash('7')),
            "expected_revision": "1",
            "idempotency_key": fixture_opaque_ref("idempotency", '8').as_str(),
            "unexpected": "denied"
        });
        assert!(
            serde_json::from_value::<SecretaryCoordinationRequest>(unknown).is_err(),
            "request DTO must deny unknown renderer fields"
        );

        let non_snooze_with_time: SecretaryCoordinationRequest = serde_json::from_value(json!({
            "action": "OPEN_LOOP_CLOSE",
            "item_ref": format!("open-loop:{}", fixture_hash('7')),
            "expected_revision": "1",
            "idempotency_key": fixture_opaque_ref("idempotency", '8').as_str(),
            "snoozed_until_utc": "2026-08-10T13:00:00Z"
        }))
        .expect("deserialize matrix candidate");
        assert_eq!(
            validate_secretary_coordination_request(non_snooze_with_time)
                .expect_err("non-snooze must reject snooze field"),
            M4C06_COORDINATION_REQUEST_INVALID
        );

        let snooze_without_time: SecretaryCoordinationRequest = serde_json::from_value(json!({
            "action": "OPEN_LOOP_SNOOZE",
            "item_ref": format!("open-loop:{}", fixture_hash('7')),
            "expected_revision": "1",
            "idempotency_key": fixture_opaque_ref("idempotency", '8').as_str()
        }))
        .expect("deserialize matrix candidate");
        assert_eq!(
            validate_secretary_coordination_request(snooze_without_time)
                .expect_err("snooze must require timestamp"),
            M4C06_COORDINATION_REQUEST_INVALID
        );
    }

    #[test]
    fn m4r02_personal_object_request_is_closed_and_carries_no_renderer_authority() {
        let decision: SecretaryPersonalObjectRequest = serde_json::from_value(json!({
            "action": "DECISION_DISMISS",
            "item_ref": format!("decision-projection:{}", fixture_hash('b')),
            "expected_revision": u64::MAX.to_string(),
            "idempotency_key": fixture_opaque_ref("idempotency", 'c').as_str()
        }))
        .expect("deserialize exact Decision local command");
        validate_secretary_personal_object_request(&decision)
            .expect("full u64 Decision revision is accepted as canonical text");

        let personal_create: SecretaryPersonalObjectRequest = serde_json::from_value(json!({
            "action": "PERSONAL_ACTION_CREATE",
            "title": "准备季度复盘",
            "due_at_utc": "2026-08-11T09:00:00Z",
            "idempotency_key": fixture_opaque_ref("idempotency", 'd').as_str()
        }))
        .expect("deserialize exact PersonalAction create");
        validate_secretary_personal_object_request(&personal_create)
            .expect("ordinary PersonalAction create validates");

        for invalid in [
            json!({
                "action": "REMINDER_FIRE",
                "item_ref": format!("reminder:{}", fixture_hash('e')),
                "expected_revision": "1",
                "idempotency_key": fixture_opaque_ref("idempotency", 'f').as_str()
            }),
            json!({
                "action": "DECISION_READ",
                "item_ref": format!("decision-projection:{}", fixture_hash('g')),
                "expected_revision": "1",
                "idempotency_key": fixture_opaque_ref("idempotency", 'h').as_str(),
                "owner_status": "ANSWERED"
            }),
            json!({
                "action": "NOTIFICATION_READ",
                "item_ref": format!("notification:{}", fixture_hash('i')),
                "expected_revision": "1",
                "idempotency_key": fixture_opaque_ref("idempotency", 'j').as_str(),
                "scope_ref": "scope:personal:other"
            }),
        ] {
            assert!(
                serde_json::from_value::<SecretaryPersonalObjectRequest>(invalid).is_err(),
                "Reminder fire, owner mutation and renderer authority all fail at DTO decode"
            );
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct RecordedCoordinationCall {
        action: String,
        item_ref: String,
        expected_revision: u64,
        idempotency_key: String,
        snoozed_until_utc: Option<String>,
    }

    #[derive(Default)]
    struct FakeCoordinationPort {
        calls: RefCell<Vec<RecordedCoordinationCall>>,
        seen: RefCell<Vec<(String, String)>>,
    }

    impl FakeCoordinationPort {
        fn record(
            &self,
            action: &str,
            aggregate_kind: &str,
            item_ref: &str,
            expected_revision: u64,
            idempotency_key: &str,
            snoozed_until_utc: Option<&str>,
        ) -> Result<M4CoordinationCommandOutcome, String> {
            let key = (action.to_string(), idempotency_key.to_string());
            let replayed = self.seen.borrow().contains(&key);
            if !replayed {
                self.seen.borrow_mut().push(key);
            }
            self.calls.borrow_mut().push(RecordedCoordinationCall {
                action: action.to_string(),
                item_ref: item_ref.to_string(),
                expected_revision,
                idempotency_key: idempotency_key.to_string(),
                snoozed_until_utc: snoozed_until_utc.map(str::to_string),
            });
            Ok(M4CoordinationCommandOutcome {
                command_receipt_id: fixture_opaque_ref("coordination-receipt", '9')
                    .as_str()
                    .to_string(),
                coordination_event_id: fixture_opaque_ref("coordination-event", 'a')
                    .as_str()
                    .to_string(),
                aggregate_kind: aggregate_kind.to_string(),
                aggregate_id: item_ref.to_string(),
                aggregate_revision: expected_revision.to_string(),
                outcome_code: if action == "OPEN_LOOP_CARRY_OVER" {
                    "CARRIED_OVER".to_string()
                } else {
                    "APPLIED".to_string()
                },
                replayed,
                busy_retries: 0,
            })
        }
    }

    impl SecretaryCoordinationPort for FakeCoordinationPort {
        fn mark_inbox_item_read(
            &self,
            inbox_item_id: &str,
            expected_revision: u64,
            idempotency_key: &str,
        ) -> Result<M4CoordinationCommandOutcome, String> {
            self.record(
                "INBOX_MARK_READ",
                "INBOX_ITEM",
                inbox_item_id,
                expected_revision,
                idempotency_key,
                None,
            )
        }

        fn dismiss_inbox_item(
            &self,
            inbox_item_id: &str,
            expected_revision: u64,
            idempotency_key: &str,
        ) -> Result<M4CoordinationCommandOutcome, String> {
            self.record(
                "INBOX_DISMISS",
                "INBOX_ITEM",
                inbox_item_id,
                expected_revision,
                idempotency_key,
                None,
            )
        }

        fn acknowledge_open_loop(
            &self,
            open_loop_id: &str,
            expected_revision: u64,
            idempotency_key: &str,
        ) -> Result<M4CoordinationCommandOutcome, String> {
            self.record(
                "OPEN_LOOP_ACKNOWLEDGE",
                "OPEN_LOOP",
                open_loop_id,
                expected_revision,
                idempotency_key,
                None,
            )
        }

        fn snooze_open_loop(
            &self,
            open_loop_id: &str,
            expected_revision: u64,
            snoozed_until_utc: &str,
            idempotency_key: &str,
        ) -> Result<M4CoordinationCommandOutcome, String> {
            self.record(
                "OPEN_LOOP_SNOOZE",
                "OPEN_LOOP",
                open_loop_id,
                expected_revision,
                idempotency_key,
                Some(snoozed_until_utc),
            )
        }

        fn close_open_loop(
            &self,
            open_loop_id: &str,
            expected_revision: u64,
            idempotency_key: &str,
        ) -> Result<M4CoordinationCommandOutcome, String> {
            self.record(
                "OPEN_LOOP_CLOSE",
                "OPEN_LOOP",
                open_loop_id,
                expected_revision,
                idempotency_key,
                None,
            )
        }

        fn dismiss_open_loop(
            &self,
            open_loop_id: &str,
            expected_revision: u64,
            idempotency_key: &str,
        ) -> Result<M4CoordinationCommandOutcome, String> {
            self.record(
                "OPEN_LOOP_DISMISS",
                "OPEN_LOOP",
                open_loop_id,
                expected_revision,
                idempotency_key,
                None,
            )
        }

        fn reopen_open_loop(
            &self,
            open_loop_id: &str,
            expected_revision: u64,
            idempotency_key: &str,
        ) -> Result<M4CoordinationCommandOutcome, String> {
            self.record(
                "OPEN_LOOP_REOPEN",
                "OPEN_LOOP",
                open_loop_id,
                expected_revision,
                idempotency_key,
                None,
            )
        }

        fn carry_over_open_loop(
            &self,
            open_loop_id: &str,
            expected_revision: u64,
            idempotency_key: &str,
        ) -> Result<M4CoordinationCommandOutcome, String> {
            self.record(
                "OPEN_LOOP_CARRY_OVER",
                "OPEN_LOOP",
                open_loop_id,
                expected_revision,
                idempotency_key,
                None,
            )
        }
    }

    fn coordination_request(
        action: SecretaryCoordinationAction,
        item_ref: String,
        expected_revision: String,
        idempotency_character: char,
        snoozed_until_utc: Option<&str>,
    ) -> SecretaryCoordinationRequest {
        SecretaryCoordinationRequest {
            action,
            item_ref,
            expected_revision,
            idempotency_key: fixture_opaque_ref("idempotency", idempotency_character)
                .as_str()
                .to_string(),
            snoozed_until_utc: snoozed_until_utc.map(str::to_string),
        }
    }

    #[test]
    fn m4c06_coordination_actions_map_exactly_and_preserve_idempotent_receipts() {
        let port = FakeCoordinationPort::default();
        let inbox_ref = format!("inbox:{}", fixture_hash('b'));
        let open_loop_ref = format!("open-loop:{}", fixture_hash('c'));
        let cases = vec![
            (
                SecretaryCoordinationAction::InboxMarkRead,
                inbox_ref.clone(),
                u64::MAX.to_string(),
                'd',
                None,
                "INBOX_MARK_READ",
            ),
            (
                SecretaryCoordinationAction::InboxDismiss,
                inbox_ref.clone(),
                "2".to_string(),
                'e',
                None,
                "INBOX_DISMISS",
            ),
            (
                SecretaryCoordinationAction::OpenLoopAcknowledge,
                open_loop_ref.clone(),
                "3".to_string(),
                'f',
                None,
                "OPEN_LOOP_ACKNOWLEDGE",
            ),
            (
                SecretaryCoordinationAction::OpenLoopSnooze,
                open_loop_ref.clone(),
                "4".to_string(),
                'g',
                Some("2026-08-10T13:00:00Z"),
                "OPEN_LOOP_SNOOZE",
            ),
            (
                SecretaryCoordinationAction::OpenLoopClose,
                open_loop_ref.clone(),
                "5".to_string(),
                'h',
                None,
                "OPEN_LOOP_CLOSE",
            ),
            (
                SecretaryCoordinationAction::OpenLoopDismiss,
                open_loop_ref.clone(),
                "6".to_string(),
                'i',
                None,
                "OPEN_LOOP_DISMISS",
            ),
            (
                SecretaryCoordinationAction::OpenLoopReopen,
                open_loop_ref.clone(),
                "7".to_string(),
                'j',
                None,
                "OPEN_LOOP_REOPEN",
            ),
            (
                SecretaryCoordinationAction::OpenLoopCarryOver,
                open_loop_ref.clone(),
                "8".to_string(),
                'k',
                None,
                "OPEN_LOOP_CARRY_OVER",
            ),
        ];
        for (action, item_ref, expected_revision, idempotency_character, snoozed_until, _) in &cases
        {
            let receipt = operate_secretary_coordination_with_port(
                &port,
                coordination_request(
                    action.clone(),
                    item_ref.clone(),
                    expected_revision.clone(),
                    *idempotency_character,
                    *snoozed_until,
                ),
            )
            .expect("exact C04 method mapping");
            assert_eq!(&receipt.item_ref.as_str(), &item_ref.as_str());
            assert_eq!(&receipt.coordination_revision, expected_revision);
        }
        let mapped_actions: Vec<String> = port
            .calls
            .borrow()
            .iter()
            .map(|call| call.action.clone())
            .collect();
        assert_eq!(
            mapped_actions,
            cases
                .iter()
                .map(|(_, _, _, _, _, expected_action)| (*expected_action).to_string())
                .collect::<Vec<_>>()
        );
        assert_eq!(port.calls.borrow()[0].expected_revision, u64::MAX);
        assert_eq!(
            port.calls.borrow()[3].snoozed_until_utc.as_deref(),
            Some("2026-08-10T13:00:00Z")
        );

        let replay = operate_secretary_coordination_with_port(
            &port,
            coordination_request(
                SecretaryCoordinationAction::InboxMarkRead,
                inbox_ref,
                u64::MAX.to_string(),
                'd',
                None,
            ),
        )
        .expect("exact idempotent replay");
        assert!(replay.replayed);
    }
}
