//! M6D04 Secretary -> Global Supervisor consultation over the M3-owned
//! Handoff lifecycle.
//!
//! M6 stores only a replay projection containing request and result refs.  M3
//! remains the sole owner of state, revision, receipts, idempotency, and
//! transition legality.  Cross-project advice is produced by M6D03 only after
//! an exact M3 ACCEPTED receipt and no path in this module owns a project
//! command, project store, provider/model invocation, or writeback port.

use crate::m3_handoff::{Handoff, HandoffId, HandoffPermissionRequest, HandoffState};
use crate::m3_role_session::{
    ConversationContext, ConversationContextRef, CorrelationId, OpaqueRef,
    PermissionSnapshotDescriptor, ProviderHandle, ProviderHandleBindingStatus,
    ProviderHandleNaturalKey, ProviderHandleRef, RequestIdempotencyKey, RetrievalStatus,
    RoleSessionId, ServerResolvedBinding, Sha256Digest,
};
use crate::m3_role_session_repository::{
    AcceptHandoffCommand, BindProviderHandleCommand, ClaimProviderEffectCommand,
    CreateHandoffCommand, CreateRoleSessionCommand, HandoffReturnResult,
    HandoffSourceObjectValidationProof, M3CommandMetadata, M3EffectMutationMetadata,
    M3HandoffCommandOutcome, M3HandoffRepositoryPort, M3HandoffSessionAuthority,
    M3ProviderEffectState, M3ReadPermissionDisposition, M3RoleSessionSnapshotQuery,
    M3RoleSessionSqliteRepository, M3SessionBindingReadState, RecordHandoffReturnResultCommand,
    RecordProviderEffectReceiptCommand, RejectHandoffCommand, RequestHandoffReturnCommand,
    UpsertConversationContextCommand,
};
use crate::m4_secretary_service::{
    M4SecretaryHandoffOutcome, M4SecretaryHandoffPort, M4SecretaryHandoffPortRecord,
    M4SecretaryHandoffReceipt, M4SecretaryHandoffRequest, M4SecretaryHash, M4SecretaryOpaqueRef,
    M4SecretaryServiceError, M4SecretaryTypedRef,
};
use crate::m6_org_cross_project_advisory::{
    consult_handoff_ref_for, validate_project_queries_for_consult,
};
use crate::m6_org_dto::{
    M6OrgConsultDecision, M6OrgConsultHandoffBinding, M6OrgConsultHandoffProjection,
    M6OrgConsultHandoffRefInput, M6OrgConsultRejectionReason, M6OrgCrossProjectAdvisory,
    M6OrgCrossProjectAdvisoryRequest, M6OrgGlobalSupervisorConsultDecisionRequest,
    M6OrgSecretaryConsultReadRequest, M6OrgSecretaryConsultStartRequest,
};
use crate::m6_org_store::M6OrgStore;
use serde::Serialize;
use std::cell::RefCell;
use std::collections::BTreeSet;

const CONSULT_KIND: &str = "SECRETARY_TO_GLOBAL_SUPERVISOR";
const CREATE_CAPABILITY: &str = "create_m3_handoff";
const CONSULT_ENDPOINT_SESSION_MATERIAL: &str =
    "syn.m6.org.consult-recipient.role-session/personal-primary/v1";
const CONSULT_ENDPOINT_CHANNEL_MATERIAL: &str =
    "syn.m6.org.consult-recipient.channel/internal-handoff/v1";
const CONSULT_ENDPOINT_PERMISSION_MATERIAL: &str =
    "syn.m6.org.consult-recipient.permission/read-only-handoff/v1";
const CONSULT_ENDPOINT_PROVIDER_KIND_MATERIAL: &str =
    "syn.m6.org.consult-recipient.provider-kind/internal-local/v1";
const CONSULT_ENDPOINT_PROVIDER_NAMESPACE_MATERIAL: &str =
    "syn.m6.org.consult-recipient.provider-namespace/organization/v1";
const CONSULT_ENDPOINT_PROVIDER_CONVERSATION_MATERIAL: &str =
    "syn.m6.org.consult-recipient.provider-conversation/personal-primary/v1";

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct M6OrgSecretaryConsultOutcome {
    pub(crate) handoff: M6OrgConsultHandoffBinding,
    pub(crate) advisory: Option<M6OrgCrossProjectAdvisory>,
    pub(crate) rejection_reason: Option<M6OrgConsultRejectionReason>,
    pub(crate) blocked_reasons: Vec<String>,
    pub(crate) replayed: bool,
    pub(crate) project_command_attempts: u64,
    pub(crate) provider_invocations: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct M6OrgSecretaryConsultCommandResponse {
    pub(crate) consult: M6OrgSecretaryConsultOutcome,
    pub(crate) secretary_handoff: M4SecretaryHandoffOutcome,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct M6OrgMemberContactHandoff {
    pub(crate) handoff_id: String,
    pub(crate) handoff_receipt_ref: String,
    pub(crate) role_session_id: String,
    pub(crate) from_actor_id: String,
    pub(crate) source_command_receipt_ref: String,
    pub(crate) replayed: bool,
}

#[derive(Clone)]
struct ConsultAuthorities {
    repository: M3RoleSessionSqliteRepository,
    source: M3HandoffSessionAuthority,
    recipient: Option<M3HandoffSessionAuthority>,
    to_role_ref: OpaqueRef,
    to_recipient_ref: OpaqueRef,
}

pub(crate) struct M6OrgConsultHandoffAdapter<'a> {
    state: &'a crate::AppState,
    start_request: Option<M6OrgSecretaryConsultStartRequest>,
    read_handoff_id: Option<String>,
    now_ms: i64,
    outcome: RefCell<Option<M6OrgSecretaryConsultOutcome>>,
}

impl<'a> M6OrgConsultHandoffAdapter<'a> {
    pub(crate) fn for_start(
        state: &'a crate::AppState,
        request: M6OrgSecretaryConsultStartRequest,
        now_ms: i64,
    ) -> Self {
        Self {
            state,
            start_request: Some(request),
            read_handoff_id: None,
            now_ms,
            outcome: RefCell::new(None),
        }
    }

    pub(crate) fn for_read(state: &'a crate::AppState, handoff_id: String, now_ms: i64) -> Self {
        Self {
            state,
            start_request: None,
            read_handoff_id: Some(handoff_id),
            now_ms,
            outcome: RefCell::new(None),
        }
    }

    pub(crate) fn take_outcome(&self) -> Result<M6OrgSecretaryConsultOutcome, String> {
        self.outcome
            .borrow_mut()
            .take()
            .ok_or_else(|| "m6_org_consult_adapter_outcome_missing".to_string())
    }
}

impl M4SecretaryHandoffPort for M6OrgConsultHandoffAdapter<'_> {
    fn create_handoff(
        &self,
        request: &M4SecretaryHandoffRequest,
    ) -> Result<M4SecretaryHandoffPortRecord, M4SecretaryServiceError> {
        let start_request = self
            .start_request
            .as_ref()
            .ok_or_else(|| M4SecretaryServiceError::new("M6_CONSULT_ADAPTER_MODE_MISMATCH"))?;
        let expected = m4_request_for_start(self.state, start_request, self.now_ms)
            .map_err(scrubbed_service_error)?;
        if request != &expected {
            return Err(M4SecretaryServiceError::new(
                "M6_CONSULT_M4_REQUEST_MISMATCH",
            ));
        }
        let outcome = start_for_state(self.state, start_request, self.now_ms)
            .map_err(scrubbed_service_error)?;
        let record = m4_record_for_outcome(&outcome).map_err(scrubbed_service_error)?;
        *self.outcome.borrow_mut() = Some(outcome);
        Ok(record)
    }

    fn read_handoff_receipt(
        &self,
        handoff_ref: &M4SecretaryOpaqueRef,
    ) -> Result<M4SecretaryHandoffPortRecord, M4SecretaryServiceError> {
        let expected = self
            .read_handoff_id
            .as_deref()
            .ok_or_else(|| M4SecretaryServiceError::new("M6_CONSULT_ADAPTER_MODE_MISMATCH"))?;
        if handoff_ref.as_str() != expected {
            return Err(M4SecretaryServiceError::new(
                "M6_CONSULT_HANDOFF_REF_MISMATCH",
            ));
        }
        let outcome = read_for_state(
            self.state,
            &M6OrgSecretaryConsultReadRequest {
                handoff_id: expected.to_string(),
            },
            self.now_ms,
        )
        .map_err(scrubbed_service_error)?;
        let record = m4_record_for_outcome(&outcome).map_err(scrubbed_service_error)?;
        *self.outcome.borrow_mut() = Some(outcome);
        Ok(record)
    }
}

pub(crate) fn m4_request_for_start(
    state: &crate::AppState,
    request: &M6OrgSecretaryConsultStartRequest,
    now_ms: i64,
) -> Result<M4SecretaryHandoffRequest, String> {
    let request = validate_start_request(request, now_ms)?;
    let status = state.m3_role_session_read_runtime.secretary_status()?;
    let source_binding = secretary_binding()?;
    if status.actor_id != source_binding.actor_id.as_str()
        || status.role_ref != source_binding.role_ref.as_str()
        || status.scope_ref != source_binding.scope_ref.as_str()
        || status.current_object_ref != source_binding.current_object_ref.as_str()
        || status.execution_channel != source_binding.execution_channel.as_str()
        || status.permission_snapshot_ref != source_binding.permission_snapshot_ref.as_str()
        || status.owner_fingerprint != source_binding.owner_fingerprint.as_str()
    {
        return Err("m6_org_consult_secretary_binding_mismatch".to_string());
    }
    let global = state.m6_org_global_role_session.authority_seed()?;
    let handoff_id = handoff_id_for(&status.role_session_id, &request)?
        .as_str()
        .to_string();
    let mut object_refs = vec![
        crate::mcp::identity_kernel::M4_PRIMARY_SECRETARY_CURRENT_OBJECT_ID.to_string(),
        request.question_ref.clone(),
    ];
    object_refs.extend(request.source_refs.clone());
    object_refs.sort();
    object_refs.dedup();
    Ok(M4SecretaryHandoffRequest {
        request_ref: m4_opaque(&handoff_id)?,
        from_role_session_ref: m4_opaque(&status.role_session_id)?,
        scope_ref: M4SecretaryTypedRef::new(
            crate::mcp::identity_kernel::M4_PRIMARY_SECRETARY_SCOPE_ID,
        )
        .map_err(|error| error.code)?,
        to_role_ref: M4SecretaryTypedRef::new(global.binding.role_ref.as_str())
            .map_err(|error| error.code)?,
        to_recipient_ref: m4_opaque(global.binding.actor_id.as_str())?,
        requested_outcome_ref: m4_opaque(
            opaque("outcome", &format!("{handoff_id}/advisory"))?.as_str(),
        )?,
        object_refs: object_refs
            .into_iter()
            .map(M4SecretaryTypedRef::new)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.code)?,
        risk_class_code: "LOW".to_string(),
        reason_ref: m4_opaque(opaque("reason", &request.question_ref)?.as_str())?,
        permission_request_ref: m4_opaque(
            opaque("permission-request", &format!("{handoff_id}/create"))?.as_str(),
        )?,
        correlation_ref: m4_opaque(correlation_id(&handoff_id)?.as_str())?,
    })
}

pub(crate) fn start_for_state(
    state: &crate::AppState,
    request: &M6OrgSecretaryConsultStartRequest,
    now_ms: i64,
) -> Result<M6OrgSecretaryConsultOutcome, String> {
    let request = validate_start_request(request, now_ms)?;
    let source_status = state.m3_role_session_read_runtime.secretary_status()?;
    let source_revision = source_status.session_revision;
    let authorities = authorities_for_start(state, source_revision)?;
    let handoff_id = handoff_id_for(&source_status.role_session_id, &request)?;
    let request_hash = request_hash(&request)?;
    let source_validation = upsert_validation_context(
        &authorities.repository,
        &authorities.source,
        &handoff_id,
        &request,
        "source-create",
    )?;
    let command = create_handoff_command(
        &authorities,
        &handoff_id,
        &request,
        source_validation.receipt.receipt_id.clone(),
        now_ms,
    )?;
    let created = authorities
        .repository
        .create_handoff(&command)
        .map_err(repository_error)?;
    if !created.replayed && !matches!(created.handoff.status, HandoffState::Created) {
        return Err("m6_org_consult_handoff_create_not_pending".to_string());
    }
    let mut store = M6OrgStore::open(&state.m6_org_store_path()?)?;
    let existing = store.load_consult_handoff_by_idempotency(&request.idempotency_key)?;
    let projection = M6OrgConsultHandoffProjection {
        handoff_id: handoff_id.as_str().to_string(),
        idempotency_key: request.idempotency_key.clone(),
        request_hash,
        request: request.clone(),
        source_session_revision: source_revision,
        source_validation_receipt_ref: source_validation.receipt.receipt_id.as_str().to_string(),
        advisory: None,
        advisory_blocked_reasons: Vec::new(),
        accepted_receipt_ref: None,
        accepted_revision: None,
        returned_receipt_ref: None,
        rejection_reason: None,
        created_at_ms: now_ms,
        updated_at_ms: now_ms,
    };
    let projection = store.record_consult_handoff(&projection)?;
    outcome_from_handoff(
        &created.handoff,
        &projection,
        created.replayed || existing.is_some(),
    )
}

/// Starts an M3-owned Handoff from the ordinary Secretary RoleSession to one
/// explicit stable-member contact binding.  The directory receives only the
/// returned refs; this helper grants no capability and invokes no provider.
pub(crate) fn start_member_contact_handoff_for_state(
    state: &crate::AppState,
    member_id: &str,
    binding: &crate::m6_org_member_directory::M6OrgMemberContactBinding,
    reason_ref: &str,
    source_refs: &[String],
    accept_by_utc: &str,
    idempotency_key: &str,
    now_ms: i64,
) -> Result<M6OrgMemberContactHandoff, String> {
    #[cfg(not(test))]
    let _ = now_ms;
    let global = state.m6_org_global_role_session.authority_seed()?;
    let source_status = state.m3_role_session_read_runtime.secretary_status()?;
    let source = secretary_authority(&global.repository, source_status.session_revision)?;
    let material = format!("syn.m6.org.member-contact/v1/{member_id}/{idempotency_key}");
    let handoff_id = HandoffId::try_from_canonical(sealed_ref("handoff", &material))
        .map_err(|_| "m6_org_member_contact_handoff_id_invalid".to_string())?;
    let mut context_source_refs = source_refs
        .iter()
        .map(|reference| opaque("source", reference))
        .collect::<Result<Vec<_>, _>>()?;
    context_source_refs.push(opaque("stable-member", member_id)?);
    context_source_refs.push(opaque("contact-binding", &binding.binding_ref)?);
    context_source_refs.sort();
    context_source_refs.dedup();
    let context = ConversationContext {
        context_ref: ConversationContextRef::try_from_canonical(sealed_ref(
            "context",
            &format!("{material}/source-validation"),
        ))
        .map_err(|_| "m6_org_member_contact_context_ref_invalid".to_string())?,
        role_session_id: source.role_session_id.clone(),
        objective_ref: opaque("reason", reason_ref)?,
        scope_ref: source.binding.scope_ref.clone(),
        current_object_ref: source.binding.current_object_ref.clone(),
        source_refs: context_source_refs.clone(),
        included_material_refs: context_source_refs.clone(),
        included_skill_refs: Vec::new(),
        source_watermark: opaque("watermark", &request_fingerprint(&material))?,
        freshness_or_staleness_marker: opaque("freshness", "current")?,
        known_gaps: Vec::new(),
        known_conflicts_or_uncertainties: Vec::new(),
        excluded_material_refs_with_reason: Vec::new(),
        retrieval_status: RetrievalStatus::Complete,
        request_more_material_ref: None,
        scrubbed_summary_ref: Some(opaque("summary", reason_ref)?),
        source_link_labels: context_source_refs,
        projection_version: "projection:v1".to_string(),
    };
    let validation = global
        .repository
        .upsert_handoff_validation_context(&UpsertConversationContextCommand {
            context,
            binding: source.binding.clone(),
            previous_permission: Some(source.previous_permission.clone()),
            current_permission: Some(source.current_permission.clone()),
            expected_session_revision: source.expected_session_revision,
            metadata: metadata(
                &global.repository,
                &format!("{material}/source-validation"),
                "validate",
            )?,
        })
        .map_err(repository_error)?;
    let source_command_receipt_ref = validation.receipt.receipt_id.clone();
    let object_refs = [
        source.binding.current_object_ref.clone(),
        opaque("stable-member", member_id)?,
        opaque("contact-binding", &binding.binding_ref)?,
        opaque("reason", reason_ref)?,
    ]
    .into_iter()
    .chain(
        source_refs
            .iter()
            .map(|reference| opaque("source", reference))
            .collect::<Result<Vec<_>, _>>()?,
    )
    .collect::<BTreeSet<_>>();
    let risk_class = opaque("risk", "syn.m6.org.member-contact/low/v1")?;
    let command = CreateHandoffCommand {
        handoff_id: handoff_id.clone(),
        source: source.clone(),
        source_command_receipt_ref: source_command_receipt_ref.clone(),
        to_role_ref: opaque_from_text(&binding.to_role_ref)?,
        to_recipient_ref: opaque_from_text(&binding.to_recipient_ref)?,
        requested_outcome_ref: opaque(
            "outcome",
            &format!("{}/member-contact", handoff_id.as_str()),
        )?,
        object_refs: object_refs.clone(),
        risk_class: risk_class.clone(),
        permission_request: HandoffPermissionRequest {
            request_id: opaque(
                "permission-request",
                &format!("{}/create", handoff_id.as_str()),
            )?,
            requested_capability_refs: [opaque("capability", CREATE_CAPABILITY)?]
                .into_iter()
                .collect(),
            requested_scope_ref: source.binding.scope_ref.clone(),
            requested_object_refs: object_refs,
            risk_class,
            reason_ref: opaque("reason", reason_ref)?,
            source_permission_snapshot_ref: source.binding.permission_snapshot_ref.clone(),
        },
        accept_by: accept_by_utc.to_string(),
        metadata: handoff_metadata(&global.repository, handoff_id.as_str(), "contact-create")?,
        #[cfg(test)]
        test_clock_now: utc_from_millis(now_ms)?,
    };
    let outcome = global
        .repository
        .create_handoff(&command)
        .map_err(repository_error)?;
    if !outcome.replayed && outcome.handoff.status != HandoffState::Created {
        return Err("m6_org_member_contact_handoff_not_created".to_string());
    }
    Ok(M6OrgMemberContactHandoff {
        handoff_id: outcome.handoff.handoff_id.as_str().to_string(),
        handoff_receipt_ref: outcome.handoff.current_receipt_id.as_str().to_string(),
        role_session_id: source.role_session_id.as_str().to_string(),
        from_actor_id: source.binding.actor_id.as_str().to_string(),
        source_command_receipt_ref: source_command_receipt_ref.as_str().to_string(),
        replayed: outcome.replayed,
    })
}

pub(crate) fn decide_for_state(
    state: &crate::AppState,
    request: &M6OrgGlobalSupervisorConsultDecisionRequest,
    now_ms: i64,
) -> Result<M6OrgSecretaryConsultOutcome, String> {
    validate_decision_request(request)?;
    let mut projection = load_projection(state, &request.handoff_id)?;
    let authorities = authorities_for_projection(state, &projection, true, now_ms)?;
    let handoff_id = parse_handoff_id(&projection.handoff_id)?;
    let current = replay_create(&authorities, &handoff_id, &projection)?.handoff;
    match request.decision {
        M6OrgConsultDecision::Accept => {
            if current.status == HandoffState::Rejected {
                return Err("m6_org_consult_decision_conflict".to_string());
            }
            accept_and_return(state, authorities, handoff_id, projection, now_ms)
        }
        M6OrgConsultDecision::Reject => {
            if !matches!(
                current.status,
                HandoffState::Created | HandoffState::Rejected
            ) {
                return Err("m6_org_consult_decision_conflict".to_string());
            }
            let reason = request
                .rejection_reason
                .ok_or_else(|| "m6_org_consult_rejection_reason_required".to_string())?;
            if projection
                .rejection_reason
                .is_some_and(|existing| existing != reason)
            {
                return Err("m6_org_consult_rejection_reason_conflict".to_string());
            }
            let recipient = authorities
                .recipient
                .clone()
                .ok_or_else(|| "m6_org_consult_recipient_unavailable".to_string())?;
            let rejected = authorities
                .repository
                .reject_handoff(&RejectHandoffCommand {
                    handoff_id,
                    source: authorities.source.clone(),
                    recipient,
                    expected_handoff_revision: 1,
                    metadata: handoff_metadata(
                        &authorities.repository,
                        &projection.handoff_id,
                        "reject",
                    )?,
                    #[cfg(test)]
                    test_clock_now: utc_from_millis(now_ms)?,
                })
                .map_err(repository_error)?;
            if rejected.handoff.status != HandoffState::Rejected {
                return Err("m6_org_consult_reject_not_committed".to_string());
            }
            projection.rejection_reason = Some(reason);
            projection.updated_at_ms = now_ms;
            persist_projection(state, &projection)?;
            outcome_from_handoff(&rejected.handoff, &projection, rejected.replayed)
        }
    }
}

pub(crate) fn read_for_state(
    state: &crate::AppState,
    request: &M6OrgSecretaryConsultReadRequest,
    now_ms: i64,
) -> Result<M6OrgSecretaryConsultOutcome, String> {
    validate_handoff_id_text(&request.handoff_id)?;
    let projection = load_projection(state, &request.handoff_id)?;
    let authorities = authorities_for_projection(state, &projection, false, now_ms)?;
    let handoff_id = parse_handoff_id(&projection.handoff_id)?;
    let replayed = replay_create(&authorities, &handoff_id, &projection)?;
    outcome_from_handoff(&replayed.handoff, &projection, replayed.replayed)
}

fn accept_and_return(
    state: &crate::AppState,
    authorities: ConsultAuthorities,
    handoff_id: HandoffId,
    mut projection: M6OrgConsultHandoffProjection,
    now_ms: i64,
) -> Result<M6OrgSecretaryConsultOutcome, String> {
    let recipient = authorities
        .recipient
        .clone()
        .ok_or_else(|| "m6_org_consult_recipient_unavailable".to_string())?;
    let accepted = authorities
        .repository
        .accept_handoff(&AcceptHandoffCommand {
            handoff_id: handoff_id.clone(),
            source: authorities.source.clone(),
            recipient: recipient.clone(),
            expected_handoff_revision: 1,
            metadata: handoff_metadata(&authorities.repository, &projection.handoff_id, "accept")?,
            #[cfg(test)]
            test_clock_now: utc_from_millis(now_ms)?,
        })
        .map_err(repository_error)?;
    let accepted_receipt = accepted
        .transition_receipt
        .as_ref()
        .ok_or_else(|| "m6_org_consult_accepted_receipt_missing".to_string())?;
    if accepted_receipt.handoff_status != HandoffState::Accepted {
        return Err("m6_org_consult_accept_not_committed".to_string());
    }
    let accepted_join = M6OrgConsultHandoffRefInput {
        consult_handoff_ref: consult_handoff_ref_for(
            &projection.handoff_id,
            accepted_receipt.handoff_revision,
            HandoffState::Accepted.as_str(),
            accepted_receipt.receipt_id.as_str(),
        ),
        handoff_id: projection.handoff_id.clone(),
        handoff_revision: accepted_receipt.handoff_revision,
        status_ref: HandoffState::Accepted.as_str().to_string(),
        receipt_ref: accepted_receipt.receipt_id.as_str().to_string(),
    };
    let advisory_was_present = projection.advisory.is_some();
    let advisory_response = crate::m6_org_cross_project_advisory::run_for_state(
        state,
        &M6OrgCrossProjectAdvisoryRequest {
            project_queries: projection.request.project_queries.clone(),
            consult_handoff: accepted_join,
            idempotency_key: format!("{}:advisory", projection.idempotency_key),
        },
        now_ms,
    )?;
    projection.accepted_receipt_ref = Some(accepted_receipt.receipt_id.as_str().to_string());
    projection.accepted_revision = Some(accepted_receipt.handoff_revision);
    projection.advisory_blocked_reasons = advisory_response.blocked_reasons.clone();
    let Some(advisory) = advisory_response.advisory else {
        projection.updated_at_ms = now_ms;
        persist_projection(state, &projection)?;
        return outcome_from_handoff(
            &accepted.handoff,
            &projection,
            accepted.replayed || advisory_was_present,
        );
    };
    projection.advisory = Some(advisory.clone());
    let return_pending = authorities
        .repository
        .request_handoff_return(&RequestHandoffReturnCommand {
            handoff_id: handoff_id.clone(),
            source: authorities.source.clone(),
            expected_handoff_revision: accepted_receipt.handoff_revision,
            return_by: projection.request.return_by_utc.clone(),
            metadata: handoff_metadata(
                &authorities.repository,
                &projection.handoff_id,
                "request-return",
            )?,
            #[cfg(test)]
            test_clock_now: utc_from_millis(now_ms)?,
        })
        .map_err(repository_error)?;
    let return_pending_receipt = return_pending
        .transition_receipt
        .as_ref()
        .filter(|receipt| receipt.handoff_status == HandoffState::ReturnPending)
        .ok_or_else(|| "m6_org_consult_return_request_not_committed".to_string())?;
    let validation = upsert_validation_context(
        &authorities.repository,
        &authorities.source,
        &handoff_id,
        &projection.request,
        "source-return-validation",
    )?;
    let result_ref = advisory_result_ref(&advisory)?;
    let result_hash = advisory_result_hash(&advisory)?;
    let returned = authorities
        .repository
        .record_handoff_return_result(&RecordHandoffReturnResultCommand {
            handoff_id,
            source: authorities.source.clone(),
            recipient,
            expected_handoff_revision: return_pending_receipt.handoff_revision,
            result: HandoffReturnResult::Returned {
                result_ref,
                result_hash,
                source_object_validation: HandoffSourceObjectValidationProof {
                    role_session_id: authorities.source.role_session_id.clone(),
                    binding: authorities.source.binding.clone(),
                    object_ref: authorities.source.binding.current_object_ref.clone(),
                    validation_receipt_ref: validation.receipt.receipt_id,
                },
            },
            metadata: handoff_metadata(
                &authorities.repository,
                &projection.handoff_id,
                "record-return",
            )?,
            #[cfg(test)]
            test_clock_now: utc_from_millis(now_ms)?,
        })
        .map_err(repository_error)?;
    if returned.handoff.status != HandoffState::Returned {
        return Err("m6_org_consult_return_not_committed".to_string());
    }
    projection.returned_receipt_ref =
        Some(returned.handoff.current_receipt_id.as_str().to_string());
    projection.updated_at_ms = now_ms;
    persist_projection(state, &projection)?;
    outcome_from_handoff(
        &returned.handoff,
        &projection,
        accepted.replayed || return_pending.replayed || returned.replayed || advisory_was_present,
    )
}

fn authorities_for_start(
    state: &crate::AppState,
    source_revision: u64,
) -> Result<ConsultAuthorities, String> {
    let global = state.m6_org_global_role_session.authority_seed()?;
    let source = secretary_authority(&global.repository, source_revision)?;
    Ok(ConsultAuthorities {
        repository: global.repository,
        source,
        recipient: None,
        to_role_ref: global.binding.role_ref,
        to_recipient_ref: global.binding.actor_id,
    })
}

fn authorities_for_projection(
    state: &crate::AppState,
    projection: &M6OrgConsultHandoffProjection,
    with_recipient: bool,
    now_ms: i64,
) -> Result<ConsultAuthorities, String> {
    let global = state.m6_org_global_role_session.authority_seed()?;
    let source = secretary_authority(&global.repository, projection.source_session_revision)?;
    let recipient = if with_recipient {
        let recipient =
            ensure_consult_recipient(&global.repository, &global.binding, &source.binding, now_ms)?;
        let handoff_id = parse_handoff_id(&projection.handoff_id)?;
        upsert_validation_context(
            &global.repository,
            &recipient,
            &handoff_id,
            &projection.request,
            "recipient-decision",
        )?;
        Some(recipient)
    } else {
        None
    };
    Ok(ConsultAuthorities {
        repository: global.repository,
        source,
        recipient,
        to_role_ref: global.binding.role_ref,
        to_recipient_ref: global.binding.actor_id,
    })
}

fn secretary_authority(
    repository: &M3RoleSessionSqliteRepository,
    expected_session_revision: u64,
) -> Result<M3HandoffSessionAuthority, String> {
    let identity = crate::mcp::identity_kernel::resolve_m4_primary_secretary_identity()
        .map_err(|error| error.code().to_string())?;
    let binding = identity
        .m3_server_resolved_binding()
        .map_err(|error| error.code().to_string())?;
    let status = repository
        .load_authorized_role_session_snapshot(&M3RoleSessionSnapshotQuery {
            role_session_id: secretary_role_session_id()?,
            binding: binding.clone(),
        })
        .map_err(repository_error)?
        .ok_or_else(|| "m6_org_consult_secretary_session_missing".to_string())?;
    if !matches!(status.permission, M3ReadPermissionDisposition::Current)
        || !matches!(
            status.current_binding,
            M3SessionBindingReadState::Verified { .. }
        )
    {
        return Err("m6_org_consult_secretary_binding_unavailable".to_string());
    }
    let permission = crate::m4_secretary_domain::permission_descriptor(&identity, &binding)?;
    Ok(M3HandoffSessionAuthority {
        role_session_id: status.session.role_session_id,
        binding,
        previous_permission: permission.clone(),
        current_permission: permission,
        expected_session_revision,
    })
}

fn ensure_consult_recipient(
    repository: &M3RoleSessionSqliteRepository,
    global_binding: &ServerResolvedBinding,
    source_binding: &ServerResolvedBinding,
    now_ms: i64,
) -> Result<M3HandoffSessionAuthority, String> {
    let binding = ServerResolvedBinding::from_parts(
        global_binding.actor_id.clone(),
        global_binding.role_ref.clone(),
        source_binding.scope_ref.clone(),
        source_binding.current_object_ref.clone(),
        opaque("channel", CONSULT_ENDPOINT_CHANNEL_MATERIAL)?,
        opaque("permission", CONSULT_ENDPOINT_PERMISSION_MATERIAL)?,
    )
    .map_err(|_| "m6_org_consult_recipient_binding_invalid".to_string())?;
    let role_session_id =
        RoleSessionId::try_from_canonical(sealed_ref("session", CONSULT_ENDPOINT_SESSION_MATERIAL))
            .map_err(|_| "m6_org_consult_recipient_session_id_invalid".to_string())?;
    let create_material = format!("{CONSULT_ENDPOINT_SESSION_MATERIAL}/create");
    let created = repository
        .create_role_session(&CreateRoleSessionCommand {
            role_session_id: role_session_id.clone(),
            binding: binding.clone(),
            metadata: metadata(repository, &create_material, "create")?,
        })
        .map_err(repository_error)?;
    let mut effect = created
        .provider_effect
        .ok_or_else(|| "m6_org_consult_recipient_effect_missing".to_string())?;
    let provider_attempt_ref = opaque("attempt", &format!("{create_material}/local"))?;
    if effect.state == M3ProviderEffectState::Registered {
        let claim = repository
            .claim_registered_provider_effect(&ClaimProviderEffectCommand {
                effect_attempt_id: effect.effect_attempt_id.clone(),
                provider_attempt_ref: provider_attempt_ref.clone(),
                binding: binding.clone(),
                expected_session_revision: 1,
                metadata: effect_metadata(
                    repository,
                    &format!("{create_material}/claim"),
                    effect.correlation_id.clone(),
                )?,
            })
            .map_err(repository_error)?;
        if !claim.dispatch_granted {
            return Err("m6_org_consult_recipient_local_claim_not_granted".to_string());
        }
        effect = claim.effect;
    }
    if effect.state == M3ProviderEffectState::DispatchClaimed {
        effect = repository
            .record_provider_effect_receipt(&RecordProviderEffectReceiptCommand {
                effect_attempt_id: effect.effect_attempt_id.clone(),
                provider_attempt_ref: provider_attempt_ref.clone(),
                provider_receipt_ref: opaque(
                    "provider-receipt",
                    &format!("{create_material}/local-ready"),
                )?,
                metadata: effect_metadata(
                    repository,
                    &format!("{create_material}/receipt"),
                    effect.correlation_id.clone(),
                )?,
            })
            .map_err(repository_error)?;
    }
    if effect.state == M3ProviderEffectState::ProviderReceiptRecorded {
        let provider_handle = ProviderHandle {
            handle_ref: ProviderHandleRef::try_from_canonical(sealed_ref(
                "provider-handle",
                CONSULT_ENDPOINT_PROVIDER_CONVERSATION_MATERIAL,
            ))
            .map_err(|_| "m6_org_consult_recipient_handle_invalid".to_string())?,
            natural_key: ProviderHandleNaturalKey::from_server_resolved(
                sealed_ref("provider-kind", CONSULT_ENDPOINT_PROVIDER_KIND_MATERIAL),
                Some(sealed_ref(
                    "provider-namespace",
                    CONSULT_ENDPOINT_PROVIDER_NAMESPACE_MATERIAL,
                )),
                sealed_ref(
                    "provider-conversation",
                    CONSULT_ENDPOINT_PROVIDER_CONVERSATION_MATERIAL,
                ),
            )
            .map_err(|_| "m6_org_consult_recipient_natural_key_invalid".to_string())?,
            owner_fingerprint: binding.owner_fingerprint.clone(),
            binding_status: ProviderHandleBindingStatus::Verified,
            last_verified_at: created.receipt.created_at.clone(),
            provenance_ref: opaque(
                "provenance",
                "syn.m6.org.consult-recipient.local-adapter/v1",
            )?,
            source_hash: Sha256Digest::of_bytes(b"syn.m6.org.consult-recipient.local-adapter/v1"),
            quarantine_reason: None,
        };
        let mut bind_metadata = metadata(repository, &format!("{create_material}/bind"), "bind")?;
        bind_metadata.correlation_id = effect.correlation_id.clone();
        repository
            .bind_provider_handle(&BindProviderHandleCommand {
                role_session_id: role_session_id.clone(),
                create_effect_attempt_id: effect.effect_attempt_id,
                provider_attempt_ref,
                provider_handle,
                binding: binding.clone(),
                previous_permission: None,
                current_permission: None,
                expected_session_revision: 1,
                expected_binding_revision: 0,
                metadata: bind_metadata,
            })
            .map_err(repository_error)?;
    }
    let snapshot = repository
        .load_authorized_role_session_snapshot(&M3RoleSessionSnapshotQuery {
            role_session_id: role_session_id.clone(),
            binding: binding.clone(),
        })
        .map_err(repository_error)?
        .ok_or_else(|| "m6_org_consult_recipient_session_missing".to_string())?;
    if !matches!(
        snapshot.current_binding,
        M3SessionBindingReadState::Verified { .. }
    ) || !matches!(snapshot.permission, M3ReadPermissionDisposition::Current)
        || snapshot.session.revision != 1
    {
        return Err("m6_org_consult_recipient_binding_unavailable".to_string());
    }
    let permission = consult_recipient_permission(&binding)?;
    let _ = now_ms;
    Ok(M3HandoffSessionAuthority {
        role_session_id,
        binding,
        previous_permission: permission.clone(),
        current_permission: permission,
        expected_session_revision: snapshot.session.revision,
    })
}

fn create_handoff_command(
    authorities: &ConsultAuthorities,
    handoff_id: &HandoffId,
    request: &M6OrgSecretaryConsultStartRequest,
    source_command_receipt_ref: OpaqueRef,
    now_ms: i64,
) -> Result<CreateHandoffCommand, String> {
    #[cfg(not(test))]
    let _ = now_ms;
    let object_refs = handoff_object_refs(&authorities.source.binding, request)?;
    let risk_class = opaque("risk", "syn.m6.org.consult-handoff/low/v1")?;
    Ok(CreateHandoffCommand {
        handoff_id: handoff_id.clone(),
        source: authorities.source.clone(),
        source_command_receipt_ref,
        to_role_ref: authorities.to_role_ref.clone(),
        to_recipient_ref: authorities.to_recipient_ref.clone(),
        requested_outcome_ref: opaque(
            "outcome",
            &format!("{}/cross-project-advisory", handoff_id.as_str()),
        )?,
        object_refs: object_refs.clone(),
        risk_class: risk_class.clone(),
        permission_request: HandoffPermissionRequest {
            request_id: opaque(
                "permission-request",
                &format!("{}/create", handoff_id.as_str()),
            )?,
            requested_capability_refs: [opaque("capability", CREATE_CAPABILITY)?]
                .into_iter()
                .collect(),
            requested_scope_ref: authorities.source.binding.scope_ref.clone(),
            requested_object_refs: object_refs,
            risk_class,
            reason_ref: opaque("reason", &request.question_ref)?,
            source_permission_snapshot_ref: authorities
                .source
                .binding
                .permission_snapshot_ref
                .clone(),
        },
        accept_by: request.accept_by_utc.clone(),
        metadata: handoff_metadata(&authorities.repository, handoff_id.as_str(), "create")?,
        #[cfg(test)]
        test_clock_now: utc_from_millis(now_ms)?,
    })
}

fn replay_create(
    authorities: &ConsultAuthorities,
    handoff_id: &HandoffId,
    projection: &M6OrgConsultHandoffProjection,
) -> Result<M3HandoffCommandOutcome, String> {
    let command = create_handoff_command(
        authorities,
        handoff_id,
        &projection.request,
        opaque_from_text(&projection.source_validation_receipt_ref)?,
        projection.created_at_ms,
    )?;
    authorities
        .repository
        .create_handoff(&command)
        .map_err(repository_error)
}

fn upsert_validation_context(
    repository: &M3RoleSessionSqliteRepository,
    authority: &M3HandoffSessionAuthority,
    handoff_id: &HandoffId,
    request: &M6OrgSecretaryConsultStartRequest,
    actor_kind: &str,
) -> Result<crate::m3_role_session_repository::M3RepositoryCommandOutcome, String> {
    repository
        .upsert_handoff_validation_context(&UpsertConversationContextCommand {
            context: validation_context(authority, handoff_id, request, actor_kind)?,
            binding: authority.binding.clone(),
            previous_permission: Some(authority.previous_permission.clone()),
            current_permission: Some(authority.current_permission.clone()),
            expected_session_revision: authority.expected_session_revision,
            metadata: metadata(
                repository,
                &format!("{}/validation/{actor_kind}", handoff_id.as_str()),
                "validate",
            )?,
        })
        .map_err(repository_error)
}

fn validation_context(
    authority: &M3HandoffSessionAuthority,
    handoff_id: &HandoffId,
    request: &M6OrgSecretaryConsultStartRequest,
    actor_kind: &str,
) -> Result<ConversationContext, String> {
    let mut source_refs = request
        .source_refs
        .iter()
        .map(|value| opaque("source", value))
        .collect::<Result<Vec<_>, _>>()?;
    source_refs.extend(
        request
            .project_queries
            .iter()
            .map(|query| opaque("project-summary", &query.project_id))
            .collect::<Result<Vec<_>, _>>()?,
    );
    source_refs.sort();
    source_refs.dedup();
    let context_ref = ConversationContextRef::try_from_canonical(sealed_ref(
        "context",
        &format!("{}/validation/{actor_kind}", handoff_id.as_str()),
    ))
    .map_err(|_| "m6_org_consult_context_ref_invalid".to_string())?;
    Ok(ConversationContext {
        context_ref,
        role_session_id: authority.role_session_id.clone(),
        objective_ref: opaque("question", &request.question_ref)?,
        scope_ref: authority.binding.scope_ref.clone(),
        current_object_ref: authority.binding.current_object_ref.clone(),
        source_refs: source_refs.clone(),
        included_material_refs: source_refs.clone(),
        included_skill_refs: Vec::new(),
        source_watermark: opaque("watermark", &request_hash(request)?)?,
        freshness_or_staleness_marker: opaque("freshness", "current")?,
        known_gaps: Vec::new(),
        known_conflicts_or_uncertainties: Vec::new(),
        excluded_material_refs_with_reason: Vec::new(),
        retrieval_status: RetrievalStatus::Complete,
        request_more_material_ref: None,
        scrubbed_summary_ref: Some(opaque("summary", &request.question_ref)?),
        source_link_labels: source_refs,
        projection_version: "projection:v1".to_string(),
    })
}

fn outcome_from_handoff(
    handoff: &Handoff,
    projection: &M6OrgConsultHandoffProjection,
    replayed: bool,
) -> Result<M6OrgSecretaryConsultOutcome, String> {
    let mut object_refs = handoff
        .object_refs
        .iter()
        .map(|reference| reference.as_str().to_string())
        .collect::<Vec<_>>();
    object_refs.sort();
    let receipt_ref = handoff.current_receipt_id.as_str().to_string();
    Ok(M6OrgSecretaryConsultOutcome {
        handoff: M6OrgConsultHandoffBinding {
            consult_handoff_ref: consult_handoff_ref_for(
                handoff.handoff_id.as_str(),
                handoff.revision,
                handoff.status.as_str(),
                &receipt_ref,
            ),
            handoff_id: handoff.handoff_id.as_str().to_string(),
            handoff_revision: handoff.revision,
            status_ref: handoff.status.as_str().to_string(),
            consult_kind: CONSULT_KIND.to_string(),
            from_role_session_id: handoff.from_role_session_id.as_str().to_string(),
            to_role_ref: handoff.to_role_ref.as_str().to_string(),
            to_recipient_ref: handoff.to_recipient_ref.as_str().to_string(),
            scope_ref: handoff.scope_ref.as_str().to_string(),
            question_ref: projection.request.question_ref.clone(),
            object_refs,
            receipt_ref,
            project_write_capability: false,
        },
        advisory: projection.advisory.clone(),
        rejection_reason: projection.rejection_reason,
        blocked_reasons: projection.advisory_blocked_reasons.clone(),
        replayed,
        project_command_attempts: 0,
        provider_invocations: 0,
    })
}

pub(crate) fn m4_record_for_outcome(
    outcome: &M6OrgSecretaryConsultOutcome,
) -> Result<M4SecretaryHandoffPortRecord, String> {
    let handoff_ref = m4_opaque(&outcome.handoff.handoff_id)?;
    let receipt_ref = m4_opaque(&outcome.handoff.receipt_ref)?;
    match outcome.handoff.status_ref.as_str() {
        "CREATED" | "ACCEPTED" | "RETURN_PENDING" => Ok(M4SecretaryHandoffPortRecord::Pending {
            handoff_ref,
            request_receipt_ref: receipt_ref,
        }),
        "RETURNED" => {
            let advisory = outcome
                .advisory
                .as_ref()
                .ok_or_else(|| "m6_org_consult_returned_advisory_missing".to_string())?;
            Ok(M4SecretaryHandoffPortRecord::Returned {
                handoff_ref: handoff_ref.clone(),
                receipt: M4SecretaryHandoffReceipt {
                    receipt_ref,
                    handoff_ref,
                    receipt_kind_code: "RETURNED".to_string(),
                    status_code: "RETURNED".to_string(),
                    result_ref: Some(m4_opaque(advisory_result_ref(advisory)?.as_str())?),
                    result_hash: Some(
                        M4SecretaryHash::new(advisory_result_hash(advisory)?.as_str())
                            .map_err(|error| error.code)?,
                    ),
                },
            })
        }
        "REJECTED" => Ok(M4SecretaryHandoffPortRecord::Failed {
            handoff_ref: handoff_ref.clone(),
            receipt: Some(M4SecretaryHandoffReceipt {
                receipt_ref,
                handoff_ref,
                receipt_kind_code: "REJECTED".to_string(),
                status_code: "REJECTED".to_string(),
                result_ref: None,
                result_hash: None,
            }),
            recovery_code: format!(
                "M6_CONSULT_REJECTED_{}",
                outcome
                    .rejection_reason
                    .map(M6OrgConsultRejectionReason::as_str)
                    .unwrap_or("REASON_UNAVAILABLE")
            ),
        }),
        _ => Ok(M4SecretaryHandoffPortRecord::Failed {
            handoff_ref,
            receipt: None,
            recovery_code: "M6_CONSULT_TERMINAL_FAILURE".to_string(),
        }),
    }
}

fn validate_start_request(
    request: &M6OrgSecretaryConsultStartRequest,
    now_ms: i64,
) -> Result<M6OrgSecretaryConsultStartRequest, String> {
    validate_external_ref("question_ref", &request.question_ref)?;
    validate_external_ref("idempotency_key", &request.idempotency_key)?;
    if request.source_refs.is_empty() || request.source_refs.len() > 64 {
        return Err("m6_org_consult_source_refs_required".to_string());
    }
    let mut source_refs = request.source_refs.clone();
    for source_ref in &source_refs {
        validate_external_ref("source_ref", source_ref)?;
    }
    source_refs.sort();
    if source_refs.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err("m6_org_consult_duplicate_source_ref".to_string());
    }
    validate_project_queries_for_consult(&request.project_queries)?;
    let accept_by_ms = parse_utc_millis(&request.accept_by_utc)?;
    let return_by_ms = parse_utc_millis(&request.return_by_utc)?;
    if accept_by_ms <= now_ms || return_by_ms <= accept_by_ms {
        return Err("m6_org_consult_deadline_invalid".to_string());
    }
    let mut project_queries = request.project_queries.clone();
    project_queries.sort_by(|left, right| left.project_id.cmp(&right.project_id));
    Ok(M6OrgSecretaryConsultStartRequest {
        question_ref: request.question_ref.clone(),
        source_refs,
        project_queries,
        accept_by_utc: request.accept_by_utc.clone(),
        return_by_utc: request.return_by_utc.clone(),
        idempotency_key: request.idempotency_key.clone(),
    })
}

fn validate_decision_request(
    request: &M6OrgGlobalSupervisorConsultDecisionRequest,
) -> Result<(), String> {
    validate_handoff_id_text(&request.handoff_id)?;
    match (request.decision, request.rejection_reason) {
        (M6OrgConsultDecision::Accept, None) | (M6OrgConsultDecision::Reject, Some(_)) => Ok(()),
        (M6OrgConsultDecision::Accept, Some(_)) => {
            Err("m6_org_consult_accept_reason_forbidden".to_string())
        }
        (M6OrgConsultDecision::Reject, None) => {
            Err("m6_org_consult_rejection_reason_required".to_string())
        }
    }
}

fn load_projection(
    state: &crate::AppState,
    handoff_id: &str,
) -> Result<M6OrgConsultHandoffProjection, String> {
    M6OrgStore::open(&state.m6_org_store_path()?)?
        .load_consult_handoff(handoff_id)?
        .ok_or_else(|| "m6_org_consult_handoff_not_found".to_string())
}

fn persist_projection(
    state: &crate::AppState,
    projection: &M6OrgConsultHandoffProjection,
) -> Result<(), String> {
    M6OrgStore::open(&state.m6_org_store_path()?)?
        .update_consult_handoff_projection(projection)
        .map(|_| ())
}

fn handoff_object_refs(
    source_binding: &ServerResolvedBinding,
    request: &M6OrgSecretaryConsultStartRequest,
) -> Result<BTreeSet<OpaqueRef>, String> {
    let mut refs = BTreeSet::from([
        source_binding.current_object_ref.clone(),
        opaque("question", &request.question_ref)?,
    ]);
    for source_ref in &request.source_refs {
        refs.insert(opaque("source", source_ref)?);
    }
    for query in &request.project_queries {
        refs.insert(opaque("project-summary", &query.project_id)?);
    }
    Ok(refs)
}

fn handoff_id_for(
    source_role_session_id: &str,
    request: &M6OrgSecretaryConsultStartRequest,
) -> Result<HandoffId, String> {
    HandoffId::try_from_canonical(sealed_ref(
        "handoff",
        &format!(
            "syn.m6.org.consult-handoff/v1/{source_role_session_id}/{}",
            request.idempotency_key
        ),
    ))
    .map_err(|_| "m6_org_consult_handoff_id_invalid".to_string())
}

fn parse_handoff_id(value: &str) -> Result<HandoffId, String> {
    HandoffId::try_from_canonical(value.to_string())
        .map_err(|_| "m6_org_consult_handoff_id_invalid".to_string())
}

fn validate_handoff_id_text(value: &str) -> Result<(), String> {
    parse_handoff_id(value).map(|_| ())
}

fn secretary_binding() -> Result<ServerResolvedBinding, String> {
    crate::mcp::identity_kernel::resolve_m4_primary_secretary_identity()
        .map_err(|error| error.code().to_string())?
        .m3_server_resolved_binding()
        .map_err(|error| error.code().to_string())
}

fn secretary_role_session_id() -> Result<RoleSessionId, String> {
    RoleSessionId::try_from_canonical(sealed_ref(
        "session",
        "syn.m4.secretary-role-session/personal-primary/v1",
    ))
    .map_err(|_| "m6_org_consult_secretary_session_id_invalid".to_string())
}

fn consult_recipient_permission(
    binding: &ServerResolvedBinding,
) -> Result<PermissionSnapshotDescriptor, String> {
    Ok(PermissionSnapshotDescriptor {
        snapshot_ref: binding.permission_snapshot_ref.clone(),
        allowed_capability_refs: BTreeSet::new(),
        denied_capability_refs: [
            opaque("capability", "write_project_fact")?,
            opaque("capability", "write_project_task")?,
            opaque("capability", "write_workflow_state")?,
            opaque("capability", "send_external_message")?,
            opaque("capability", "use_external_connector")?,
        ]
        .into_iter()
        .collect(),
        constraint_refs: [
            opaque("constraint", "handoff_only")?,
            opaque("constraint", "read_only")?,
            opaque("constraint", "no_project_command")?,
        ]
        .into_iter()
        .collect(),
    })
}

fn advisory_result_ref(advisory: &M6OrgCrossProjectAdvisory) -> Result<OpaqueRef, String> {
    opaque("advisory-result", &advisory.advisory_id)
}

fn advisory_result_hash(advisory: &M6OrgCrossProjectAdvisory) -> Result<Sha256Digest, String> {
    let bytes = serde_json::to_vec(advisory)
        .map_err(|error| format!("m6_org_consult_advisory_serialize:{error}"))?;
    Ok(Sha256Digest::of_bytes(&bytes))
}

fn request_hash(request: &M6OrgSecretaryConsultStartRequest) -> Result<String, String> {
    let bytes = serde_json::to_vec(request)
        .map_err(|error| format!("m6_org_consult_request_serialize:{error}"))?;
    Ok(Sha256Digest::of_bytes(&bytes).as_str().to_string())
}

fn request_fingerprint(material: &str) -> String {
    Sha256Digest::of_bytes(material.as_bytes())
        .as_str()
        .to_string()
}

#[cfg(test)]
pub(crate) fn bind_fake_secretary_for_member_directory(
    state: &crate::AppState,
) -> Result<(), String> {
    let repository = state
        .m6_org_global_role_session
        .authority_seed()?
        .repository;
    let binding = secretary_binding()?;
    let role_session_id = secretary_role_session_id()?;
    let create_material = "syn.m4.secretary-role-session-create/personal-primary/v1";
    let created = repository
        .create_role_session(&CreateRoleSessionCommand {
            role_session_id: role_session_id.clone(),
            binding: binding.clone(),
            metadata: metadata(&repository, create_material, "create")?,
        })
        .map_err(repository_error)?;
    let mut effect = created
        .provider_effect
        .ok_or_else(|| "m6_org_member_test_secretary_effect_missing".to_string())?;
    let provider_attempt_ref = opaque("attempt", "m6d05/fake-secretary/create")?;
    if effect.state == M3ProviderEffectState::Registered {
        let claim = repository
            .claim_registered_provider_effect(&ClaimProviderEffectCommand {
                effect_attempt_id: effect.effect_attempt_id.clone(),
                provider_attempt_ref: provider_attempt_ref.clone(),
                binding: binding.clone(),
                expected_session_revision: 1,
                metadata: effect_metadata(
                    &repository,
                    "m6d05/fake-secretary/claim",
                    effect.correlation_id.clone(),
                )?,
            })
            .map_err(repository_error)?;
        if !claim.dispatch_granted {
            return Err("m6_org_member_test_secretary_claim_denied".to_string());
        }
        effect = claim.effect;
    }
    if effect.state == M3ProviderEffectState::DispatchClaimed {
        effect = repository
            .record_provider_effect_receipt(&RecordProviderEffectReceiptCommand {
                effect_attempt_id: effect.effect_attempt_id.clone(),
                provider_attempt_ref: provider_attempt_ref.clone(),
                provider_receipt_ref: opaque("provider-receipt", "m6d05/fake-secretary/create")?,
                metadata: effect_metadata(
                    &repository,
                    "m6d05/fake-secretary/receipt",
                    effect.correlation_id.clone(),
                )?,
            })
            .map_err(repository_error)?;
    }
    if effect.state == M3ProviderEffectState::ProviderReceiptRecorded {
        let provider_handle = ProviderHandle {
            handle_ref: ProviderHandleRef::try_from_canonical(sealed_ref(
                "provider-handle",
                "m6d05/fake-secretary",
            ))
            .map_err(|_| "m6_org_member_test_secretary_handle_invalid".to_string())?,
            natural_key: ProviderHandleNaturalKey::from_server_resolved(
                sealed_ref("provider-kind", "m6d05/fake"),
                Some(sealed_ref("provider-namespace", "m6d05/tests")),
                sealed_ref("provider-conversation", "m6d05/fake-secretary"),
            )
            .map_err(|_| "m6_org_member_test_secretary_natural_key_invalid".to_string())?,
            owner_fingerprint: binding.owner_fingerprint.clone(),
            binding_status: ProviderHandleBindingStatus::Verified,
            last_verified_at: created.receipt.created_at.clone(),
            provenance_ref: opaque("provenance", "m6d05/fake-secretary/readback")?,
            source_hash: Sha256Digest::of_bytes(b"m6d05/fake-secretary/readback"),
            quarantine_reason: None,
        };
        let mut bind_metadata = metadata(&repository, "m6d05/fake-secretary/bind", "bind")?;
        bind_metadata.correlation_id = effect.correlation_id.clone();
        repository
            .bind_provider_handle(&BindProviderHandleCommand {
                role_session_id,
                create_effect_attempt_id: effect.effect_attempt_id,
                provider_attempt_ref,
                provider_handle,
                binding,
                previous_permission: None,
                current_permission: None,
                expected_session_revision: 1,
                expected_binding_revision: 0,
                metadata: bind_metadata,
            })
            .map_err(repository_error)?;
    }
    state
        .m3_role_session_read_runtime
        .secretary_status()
        .map(|_| ())
}

fn handoff_metadata(
    repository: &M3RoleSessionSqliteRepository,
    handoff_id: &str,
    operation: &str,
) -> Result<M3CommandMetadata, String> {
    let mut metadata = metadata(
        repository,
        &format!("syn.m6.org.consult-handoff/v1/{handoff_id}/{operation}"),
        operation,
    )?;
    metadata.correlation_id = correlation_id(handoff_id)?;
    Ok(metadata)
}

fn metadata(
    repository: &M3RoleSessionSqliteRepository,
    material: &str,
    operation: &str,
) -> Result<M3CommandMetadata, String> {
    Ok(M3CommandMetadata {
        receipt_id: opaque("receipt", &format!("{material}/receipt"))?,
        event_id: opaque("event", &format!("{material}/event"))?,
        audit_id: opaque("audit", &format!("{material}/audit"))?,
        correlation_id: correlation_id(material)?,
        request_idempotency_key: RequestIdempotencyKey::try_from_canonical(sealed_ref(
            "request",
            &format!("{material}/idempotency/{operation}"),
        ))
        .map_err(|_| "m6_org_consult_idempotency_ref_invalid".to_string())?,
        occurred_at: repository
            .capture_server_utc_now()
            .map_err(repository_error)?,
    })
}

fn effect_metadata(
    repository: &M3RoleSessionSqliteRepository,
    material: &str,
    correlation_id: CorrelationId,
) -> Result<M3EffectMutationMetadata, String> {
    Ok(M3EffectMutationMetadata {
        event_id: opaque("event", &format!("{material}/event"))?,
        audit_id: opaque("audit", &format!("{material}/audit"))?,
        correlation_id,
        occurred_at: repository
            .capture_server_utc_now()
            .map_err(repository_error)?,
    })
}

fn correlation_id(material: &str) -> Result<CorrelationId, String> {
    CorrelationId::try_from_canonical(sealed_ref("correlation", material))
        .map_err(|_| "m6_org_consult_correlation_invalid".to_string())
}

fn opaque(namespace: &str, material: &str) -> Result<OpaqueRef, String> {
    OpaqueRef::try_from_canonical(sealed_ref(namespace, material))
        .map_err(|_| "m6_org_consult_opaque_ref_invalid".to_string())
}

fn opaque_from_text(value: &str) -> Result<OpaqueRef, String> {
    OpaqueRef::try_from_canonical(value.to_string())
        .map_err(|_| "m6_org_consult_opaque_ref_invalid".to_string())
}

fn m4_opaque(value: &str) -> Result<M4SecretaryOpaqueRef, String> {
    M4SecretaryOpaqueRef::new(value.to_string()).map_err(|error| error.code)
}

fn sealed_ref(namespace: &str, material: &str) -> String {
    format!(
        "{namespace}:sha256:{}",
        Sha256Digest::of_bytes(material.as_bytes()).as_str()
    )
}

fn repository_error(
    error: crate::m3_role_session_repository::M3RoleSessionRepositoryError,
) -> String {
    error.code
}

fn scrubbed_service_error(error: String) -> M4SecretaryServiceError {
    let code = if error.contains("unavailable") || error.contains("missing") {
        "M6_CONSULT_UNAVAILABLE"
    } else if error.contains("deadline") {
        "M6_CONSULT_DEADLINE_INVALID"
    } else if error.contains("idempotency") || error.contains("collision") {
        "M6_CONSULT_IDEMPOTENCY_CONFLICT"
    } else {
        "M6_CONSULT_FAILED"
    };
    M4SecretaryServiceError::new(code)
}

fn validate_external_ref(field: &str, value: &str) -> Result<(), String> {
    let lower = value.to_ascii_lowercase();
    if value.is_empty()
        || value.len() > 512
        || value.trim() != value
        || value.chars().any(char::is_control)
        || value.contains('\\')
        || value.starts_with('/')
        || value.starts_with("./")
        || value.starts_with("../")
        || value.contains("/./")
        || value.contains("/../")
        || lower.contains("://")
        || value.contains('@')
        || [
            "password",
            "credential",
            "api_key",
            "apikey",
            "access_token",
            "refresh_token",
            "private_key",
        ]
        .iter()
        .any(|needle| lower.contains(needle))
    {
        return Err(format!("m6_org_consult_invalid_ref:{field}"));
    }
    Ok(())
}

fn parse_utc_millis(value: &str) -> Result<i64, String> {
    let bytes = value.as_bytes();
    if bytes.len() != 24
        || bytes.get(4) != Some(&b'-')
        || bytes.get(7) != Some(&b'-')
        || bytes.get(10) != Some(&b'T')
        || bytes.get(13) != Some(&b':')
        || bytes.get(16) != Some(&b':')
        || bytes.get(19) != Some(&b'.')
        || bytes.get(23) != Some(&b'Z')
    {
        return Err("m6_org_consult_utc_millis_required".to_string());
    }
    let number = |start: usize, end: usize| -> Result<i64, String> {
        std::str::from_utf8(&bytes[start..end])
            .ok()
            .and_then(|part| part.parse::<i64>().ok())
            .ok_or_else(|| "m6_org_consult_utc_millis_required".to_string())
    };
    let year = number(0, 4)?;
    let month = number(5, 7)?;
    let day = number(8, 10)?;
    let hour = number(11, 13)?;
    let minute = number(14, 16)?;
    let second = number(17, 19)?;
    let millis = number(20, 23)?;
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let month_days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => 0,
    };
    if year < 1970
        || day < 1
        || day > month_days
        || hour > 23
        || minute > 59
        || second > 59
        || millis > 999
    {
        return Err("m6_org_consult_utc_millis_required".to_string());
    }
    let days = days_from_civil(year, month, day);
    days.checked_mul(86_400_000)
        .and_then(|base| base.checked_add(hour * 3_600_000))
        .and_then(|base| base.checked_add(minute * 60_000))
        .and_then(|base| base.checked_add(second * 1_000))
        .and_then(|base| base.checked_add(millis))
        .ok_or_else(|| "m6_org_consult_utc_millis_required".to_string())
}

fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = year - i64::from(month <= 2);
    let era = year.div_euclid(400);
    let year_of_era = year - era * 400;
    let shifted_month = month + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * shifted_month + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

fn utc_from_millis(epoch_millis: i64) -> Result<String, String> {
    if epoch_millis < 0 {
        return Err("m6_org_consult_clock_invalid".to_string());
    }
    let seconds = epoch_millis.div_euclid(1_000);
    let millis = epoch_millis.rem_euclid(1_000);
    let days = seconds.div_euclid(86_400);
    let seconds_in_day = seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    Ok(format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}.{millis:03}Z",
        seconds_in_day / 3_600,
        (seconds_in_day % 3_600) / 60,
        seconds_in_day % 60
    ))
}

fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let shifted = days + 719_468;
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    year += i64::from(month <= 2);
    (year, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::m3_role_session::{
        ProviderHandle, ProviderHandleBindingStatus, ProviderHandleNaturalKey,
    };
    use crate::m3_role_session_repository::{
        BindProviderHandleCommand, ClaimProviderEffectCommand, CreateRoleSessionCommand,
        M3ProviderEffectState, RecordProviderEffectReceiptCommand,
        M3_ORDINARY_ROLE_SESSION_RELATIVE_PATH,
    };
    use crate::m4_secretary_service::M4SecretaryHandoffStatus;
    use crate::m5_project_summary::{ensure_summary_schema, ProjectSummary, SourceRef};
    use crate::m6_org_cross_project_advisory::{
        policy_allow_ref_for, project_owner_ref_for, summary_id_for,
    };
    use crate::m6_org_dto::{
        M6OrgConsultDecision, M6OrgConsultRejectionReason, M6OrgProjectSummaryQueryInput,
    };
    use crate::AppState;
    use rusqlite::{params, Connection};
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    const NOW_MS: i64 = 1_787_097_600_000;
    const ACCEPT_BY: &str = "2026-08-19T00:10:00.000Z";
    const RETURN_BY: &str = "2026-08-19T00:20:00.000Z";
    static SCRATCH_SEQUENCE: AtomicU64 = AtomicU64::new(1);

    struct Fixture {
        root: PathBuf,
        app_data_root: PathBuf,
        state: AppState,
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }

    fn fixture(label: &str) -> Fixture {
        let sequence = SCRATCH_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "syn-m6d04-{label}-{}-{sequence}",
            std::process::id()
        ));
        let app_data_root = root.join(crate::m1_project_index::M1_ORDINARY_APP_DATA_DIR_NAME);
        std::fs::create_dir_all(&app_data_root).expect("create M6D04 app-data root");
        let app_data_root =
            std::fs::canonicalize(&app_data_root).expect("canonical M6D04 app-data root");
        let seed_dir = root.join("synthetic-ordinary-product-seeds");
        std::fs::create_dir_all(&seed_dir).expect("create M6D04 seeds");
        let index_seed = seed_dir.join("codex-index.json");
        let tasks_seed = seed_dir.join("README.md");
        std::fs::write(&index_seed, r#"{"projects":[]}"#).expect("write M6D04 index seed");
        std::fs::write(&tasks_seed, "# synthetic M6D04 tasks\n").expect("write M6D04 tasks seed");
        let state = AppState::try_new_with_tauri_ordinary_product_seeds(
            &app_data_root,
            &index_seed,
            &tasks_seed,
        )
        .expect("ordinary M6D04 AppState");
        Fixture {
            root,
            app_data_root,
            state,
        }
    }

    fn bind_fake_secretary(fixture: &Fixture) {
        let seed = fixture
            .state
            .m6_org_global_role_session
            .authority_seed()
            .expect("M6D04 repository seed");
        let repository = seed.repository;
        let binding = secretary_binding().expect("Secretary binding");
        let role_session_id = secretary_role_session_id().expect("Secretary session id");
        let create_material = "syn.m4.secretary-role-session-create/personal-primary/v1";
        let created = repository
            .create_role_session(&CreateRoleSessionCommand {
                role_session_id: role_session_id.clone(),
                binding: binding.clone(),
                metadata: metadata(&repository, create_material, "create")
                    .expect("Secretary create replay metadata"),
            })
            .expect("replay Secretary create");
        let mut effect = created.provider_effect.expect("Secretary create effect");
        let provider_attempt_ref =
            opaque("attempt", "m6d04/fake-secretary/create").expect("fake Secretary attempt ref");
        if effect.state == M3ProviderEffectState::Registered {
            let claim = repository
                .claim_registered_provider_effect(&ClaimProviderEffectCommand {
                    effect_attempt_id: effect.effect_attempt_id.clone(),
                    provider_attempt_ref: provider_attempt_ref.clone(),
                    binding: binding.clone(),
                    expected_session_revision: 1,
                    metadata: effect_metadata(
                        &repository,
                        "m6d04/fake-secretary/claim",
                        effect.correlation_id.clone(),
                    )
                    .expect("fake Secretary claim metadata"),
                })
                .expect("claim fake Secretary create");
            assert!(claim.dispatch_granted);
            effect = claim.effect;
        }
        if effect.state == M3ProviderEffectState::DispatchClaimed {
            effect = repository
                .record_provider_effect_receipt(&RecordProviderEffectReceiptCommand {
                    effect_attempt_id: effect.effect_attempt_id.clone(),
                    provider_attempt_ref: provider_attempt_ref.clone(),
                    provider_receipt_ref: opaque("provider-receipt", "m6d04/fake-secretary/create")
                        .expect("fake Secretary provider receipt"),
                    metadata: effect_metadata(
                        &repository,
                        "m6d04/fake-secretary/receipt",
                        effect.correlation_id.clone(),
                    )
                    .expect("fake Secretary receipt metadata"),
                })
                .expect("record fake Secretary receipt");
        }
        if effect.state == M3ProviderEffectState::ProviderReceiptRecorded {
            let provider_handle = ProviderHandle {
                handle_ref: ProviderHandleRef::try_from_canonical(sealed_ref(
                    "provider-handle",
                    "m6d04/fake-secretary",
                ))
                .expect("fake Secretary handle"),
                natural_key: ProviderHandleNaturalKey::from_server_resolved(
                    sealed_ref("provider-kind", "m6d04/fake"),
                    Some(sealed_ref("provider-namespace", "m6d04/tests")),
                    sealed_ref("provider-conversation", "m6d04/fake-secretary"),
                )
                .expect("fake Secretary natural key"),
                owner_fingerprint: binding.owner_fingerprint.clone(),
                binding_status: ProviderHandleBindingStatus::Verified,
                last_verified_at: created.receipt.created_at.clone(),
                provenance_ref: opaque("provenance", "m6d04/fake-secretary/readback")
                    .expect("fake Secretary provenance"),
                source_hash: Sha256Digest::of_bytes(b"m6d04/fake-secretary/readback"),
                quarantine_reason: None,
            };
            let mut bind_metadata = metadata(&repository, "m6d04/fake-secretary/bind", "bind")
                .expect("fake Secretary bind metadata");
            bind_metadata.correlation_id = effect.correlation_id.clone();
            repository
                .bind_provider_handle(&BindProviderHandleCommand {
                    role_session_id,
                    create_effect_attempt_id: effect.effect_attempt_id,
                    provider_attempt_ref,
                    provider_handle,
                    binding,
                    previous_permission: None,
                    current_permission: None,
                    expected_session_revision: 1,
                    expected_binding_revision: 0,
                    metadata: bind_metadata,
                })
                .expect("bind fake Secretary provider");
        }
        fixture
            .state
            .m3_role_session_read_runtime
            .secretary_status()
            .expect("bound Secretary read status");
    }

    fn summary(project_id: &str, orchestration_id: &str, watermark_ms: i64) -> ProjectSummary {
        ProjectSummary {
            project_id: project_id.to_string(),
            orchestration_id: orchestration_id.to_string(),
            schema_version: "m5.project-summary.v1".to_string(),
            version: 1,
            watermark_ms,
            summary_hash: Sha256Digest::of_bytes(
                format!("{project_id}:{orchestration_id}:{watermark_ms}").as_bytes(),
            )
            .as_str()
            .to_string(),
            source_refs: vec![SourceRef {
                source_type: "project_fact".to_string(),
                source_id: format!("fact-ref-{project_id}"),
                last_updated_ms: watermark_ms,
            }],
            fact_count: 3,
            unverified_claim_count: 1,
            open_run_count: 1,
            rebuilt_at_ms: watermark_ms,
        }
    }

    fn query(summary: &ProjectSummary) -> M6OrgProjectSummaryQueryInput {
        let owner_ref = project_owner_ref_for(&summary.project_id);
        M6OrgProjectSummaryQueryInput {
            summary_id: Some(summary_id_for(&summary.project_id)),
            project_id: summary.project_id.clone(),
            project_owner_ref: Some(owner_ref.clone()),
            policy_decision_ref: Some(policy_allow_ref_for(&summary.project_id, &owner_ref)),
            expected_schema_version: Some(summary.schema_version.clone()),
            expected_version: Some(summary.version),
            expected_source_watermark: Some(summary.watermark_ms),
            expected_summary_hash: Some(summary.summary_hash.clone()),
        }
    }

    fn seed_summaries(state: &AppState, summaries: &[ProjectSummary]) {
        let store = state.open_m5_store().expect("open synthetic M5 store");
        ensure_summary_schema(&store).expect("ensure synthetic summary schema");
        for summary in summaries {
            let source_refs_json =
                serde_json::to_string(&summary.source_refs).expect("serialize source refs");
            store
                .connection()
                .execute(
                    "INSERT OR REPLACE INTO m5_project_summaries (
                        project_id,orchestration_id,schema_version,version,watermark_ms,
                        summary_hash,source_refs_json,fact_count,unverified_claim_count,
                        open_run_count,rebuilt_at_ms
                     ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)",
                    params![
                        summary.project_id,
                        summary.orchestration_id,
                        summary.schema_version,
                        summary.version as i64,
                        summary.watermark_ms,
                        summary.summary_hash,
                        source_refs_json,
                        summary.fact_count,
                        summary.unverified_claim_count,
                        summary.open_run_count,
                        summary.rebuilt_at_ms
                    ],
                )
                .expect("seed synthetic ProjectSummary");
        }
    }

    fn request(label: &str, summaries: &[ProjectSummary]) -> M6OrgSecretaryConsultStartRequest {
        M6OrgSecretaryConsultStartRequest {
            question_ref: format!("question-ref:{label}"),
            source_refs: summaries
                .iter()
                .map(|summary| format!("source-ref:{}", summary.project_id))
                .collect(),
            project_queries: summaries.iter().map(query).collect(),
            accept_by_utc: ACCEPT_BY.to_string(),
            return_by_utc: RETURN_BY.to_string(),
            idempotency_key: format!("consult-idempotency:{label}"),
        }
    }

    fn m3_count(fixture: &Fixture, table: &str) -> i64 {
        let allowed = [
            "m3_handoffs",
            "m3_handoff_command_receipts",
            "m3_handoff_receipts",
        ];
        assert!(allowed.contains(&table));
        Connection::open(
            fixture
                .app_data_root
                .join(M3_ORDINARY_ROLE_SESSION_RELATIVE_PATH),
        )
        .expect("open M3 count connection")
        .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
            row.get::<_, i64>(0)
        })
        .expect("count M3 rows")
    }

    fn product_hashes(state: &AppState) -> Vec<(PathBuf, Option<String>)> {
        [
            &state.index_path,
            &state.tasks_path,
            &state.workflow_state_path,
        ]
        .into_iter()
        .map(|path| {
            let hash = std::fs::read(path)
                .ok()
                .map(|bytes| Sha256Digest::of_bytes(&bytes).as_str().to_string());
            (path.clone(), hash)
        })
        .collect()
    }

    #[test]
    fn m6d04_start_accept_return_read_and_exact_replay_are_m3_owned() {
        let fixture = fixture("accept-return");
        bind_fake_secretary(&fixture);
        let summaries = [
            summary("project-alpha", "orch-alpha", NOW_MS - 2_000),
            summary("project-beta", "orch-beta", NOW_MS - 1_000),
        ];
        seed_summaries(&fixture.state, &summaries);
        let product_before = product_hashes(&fixture.state);
        let request = request("accept-return", &summaries);

        let started = crate::secretary_agent::start_global_supervisor_consult_for_state(
            &fixture.state,
            &request,
            NOW_MS,
        )
        .expect("Secretary starts consult through M4");
        assert_eq!(started.consult.handoff.status_ref, "CREATED");
        assert_eq!(
            started.secretary_handoff.status,
            M4SecretaryHandoffStatus::Pending
        );
        assert!(!started.consult.handoff.project_write_capability);
        assert_eq!(started.consult.handoff.question_ref, request.question_ref);
        assert!(!started.consult.handoff.from_role_session_id.is_empty());
        assert!(!started.consult.handoff.to_role_ref.is_empty());
        assert!(!started.consult.handoff.to_recipient_ref.is_empty());
        assert!(!started.consult.handoff.scope_ref.is_empty());
        assert!(!started.consult.handoff.receipt_ref.is_empty());
        assert_eq!(
            started.consult.handoff.consult_kind,
            "SECRETARY_TO_GLOBAL_SUPERVISOR"
        );
        assert_eq!(started.consult.handoff.handoff_revision, 1);
        let binding_shape =
            serde_json::to_value(&started.consult.handoff).expect("serialize ConsultHandoff");
        for required in [
            "consult_handoff_ref",
            "handoff_id",
            "handoff_revision",
            "status_ref",
            "consult_kind",
            "from_role_session_id",
            "to_role_ref",
            "scope_ref",
            "question_ref",
            "object_refs",
            "receipt_ref",
            "project_write_capability",
        ] {
            assert!(binding_shape.get(required).is_some(), "missing {required}");
        }
        assert!(binding_shape.get("revision").is_none());
        assert!(binding_shape.get("status").is_none());

        let accepted = decide_for_state(
            &fixture.state,
            &M6OrgGlobalSupervisorConsultDecisionRequest {
                handoff_id: started.consult.handoff.handoff_id.clone(),
                decision: M6OrgConsultDecision::Accept,
                rejection_reason: None,
            },
            NOW_MS,
        )
        .expect("Global Supervisor accepts and returns advisory");
        assert_eq!(accepted.handoff.status_ref, "RETURNED");
        assert_eq!(accepted.project_command_attempts, 0);
        assert_eq!(accepted.provider_invocations, 0);
        let advisory = accepted.advisory.as_ref().expect("returned advisory");
        assert_eq!(advisory.source_links.len(), 2);
        assert!(advisory.source_links.iter().all(|link| {
            !link.object_ref.is_empty()
                && !link.scrubbed_summary_ref.is_empty()
                && !link.deep_link_metadata_ref.is_empty()
        }));
        let projection = load_projection(&fixture.state, &accepted.handoff.handoff_id)
            .expect("load M6 consult projection");
        let accepted_receipt_ref = projection
            .accepted_receipt_ref
            .as_deref()
            .expect("accepted receipt ref");
        let accepted_revision = projection.accepted_revision.expect("accepted revision");
        assert_eq!(
            advisory.consult_handoff_ref,
            consult_handoff_ref_for(
                &projection.handoff_id,
                accepted_revision,
                "ACCEPTED",
                accepted_receipt_ref,
            )
        );

        let read = crate::secretary_agent::read_global_supervisor_consult_for_state(
            &fixture.state,
            &M6OrgSecretaryConsultReadRequest {
                handoff_id: accepted.handoff.handoff_id.clone(),
            },
            NOW_MS,
        )
        .expect("Secretary reads returned M3 receipt through M4");
        assert_eq!(read.consult.handoff.status_ref, "RETURNED");
        assert_eq!(
            read.secretary_handoff.status,
            M4SecretaryHandoffStatus::Returned
        );
        assert_eq!(read.consult.advisory, accepted.advisory);
        assert_eq!(product_hashes(&fixture.state), product_before);

        let counts_before = [
            m3_count(&fixture, "m3_handoffs"),
            m3_count(&fixture, "m3_handoff_command_receipts"),
            m3_count(&fixture, "m3_handoff_receipts"),
        ];
        let replay_start = crate::secretary_agent::start_global_supervisor_consult_for_state(
            &fixture.state,
            &request,
            NOW_MS,
        )
        .expect("exact start replay");
        assert!(replay_start.consult.replayed);
        let replay_accept = decide_for_state(
            &fixture.state,
            &M6OrgGlobalSupervisorConsultDecisionRequest {
                handoff_id: accepted.handoff.handoff_id.clone(),
                decision: M6OrgConsultDecision::Accept,
                rejection_reason: None,
            },
            NOW_MS,
        )
        .expect("exact full-flow replay");
        assert!(replay_accept.replayed);
        assert_eq!(
            counts_before,
            [
                m3_count(&fixture, "m3_handoffs"),
                m3_count(&fixture, "m3_handoff_command_receipts"),
                m3_count(&fixture, "m3_handoff_receipts"),
            ]
        );
        let m6_store = M6OrgStore::open(&fixture.state.m6_org_store_path().expect("M6 path"))
            .expect("open M6 store");
        assert_eq!(
            m6_store
                .count_rows("m6_consult_handoff_bindings")
                .expect("count consult bindings"),
            1
        );
        assert_eq!(
            m6_store
                .count_rows("m6_cross_project_advisories")
                .expect("count advisories"),
            1
        );
    }

    #[test]
    fn m6d04_explicit_rejection_keeps_reason_and_never_generates_advisory() {
        let fixture = fixture("reject");
        bind_fake_secretary(&fixture);
        let summaries = [
            summary("project-reject-a", "orch-reject-a", NOW_MS - 2_000),
            summary("project-reject-b", "orch-reject-b", NOW_MS - 1_000),
        ];
        seed_summaries(&fixture.state, &summaries);
        let request = request("reject", &summaries);
        let started = crate::secretary_agent::start_global_supervisor_consult_for_state(
            &fixture.state,
            &request,
            NOW_MS,
        )
        .expect("start rejectable consult");
        let rejected = decide_for_state(
            &fixture.state,
            &M6OrgGlobalSupervisorConsultDecisionRequest {
                handoff_id: started.consult.handoff.handoff_id.clone(),
                decision: M6OrgConsultDecision::Reject,
                rejection_reason: Some(M6OrgConsultRejectionReason::InsufficientEvidence),
            },
            NOW_MS,
        )
        .expect("explicit reject");
        assert_eq!(rejected.handoff.status_ref, "REJECTED");
        assert_eq!(
            rejected.rejection_reason,
            Some(M6OrgConsultRejectionReason::InsufficientEvidence)
        );
        assert!(rejected.advisory.is_none());
        let read = crate::secretary_agent::read_global_supervisor_consult_for_state(
            &fixture.state,
            &M6OrgSecretaryConsultReadRequest {
                handoff_id: rejected.handoff.handoff_id.clone(),
            },
            NOW_MS,
        )
        .expect("read rejected receipt");
        assert_eq!(
            read.secretary_handoff.status,
            M4SecretaryHandoffStatus::Failed
        );
        assert_eq!(read.consult.rejection_reason, rejected.rejection_reason);
        let m6_store = M6OrgStore::open(&fixture.state.m6_org_store_path().expect("M6 path"))
            .expect("open M6 store");
        assert_eq!(
            m6_store
                .count_rows("m6_cross_project_advisories")
                .expect("count rejected advisories"),
            0
        );
    }

    #[test]
    fn m6d04_missing_fields_reject_before_m3_or_m6_write() {
        let fixture = fixture("missing-fields");
        let m3_path = fixture
            .app_data_root
            .join(M3_ORDINARY_ROLE_SESSION_RELATIVE_PATH);
        let m3_before = Sha256Digest::of_bytes(&std::fs::read(&m3_path).expect("read M3 before"))
            .as_str()
            .to_string();
        let m6_path = fixture.state.m6_org_store_path().expect("M6 path");
        assert!(!m6_path.exists());
        let summaries = [
            summary("project-missing-a", "orch-missing-a", NOW_MS - 2_000),
            summary("project-missing-b", "orch-missing-b", NOW_MS - 1_000),
        ];
        let mut invalid = request("missing-fields", &summaries);
        invalid.question_ref.clear();
        let error = crate::secretary_agent::start_global_supervisor_consult_for_state(
            &fixture.state,
            &invalid,
            NOW_MS,
        )
        .expect_err("missing question ref rejected");
        assert!(error.contains("question_ref"));
        let m3_after = Sha256Digest::of_bytes(&std::fs::read(&m3_path).expect("read M3 after"))
            .as_str()
            .to_string();
        assert_eq!(m3_after, m3_before);
        assert!(!m6_path.exists());
    }

    #[test]
    fn m6d04_real_command_registry_and_m4_entry_chain_are_present() {
        let commands = include_str!("commands.rs");
        let registry = include_str!("command_registry.rs");
        let secretary = include_str!("secretary_agent.rs");
        let lib = include_str!("lib.rs");
        for command in [
            "start_secretary_global_supervisor_consult",
            "decide_global_supervisor_consult_handoff",
            "read_secretary_global_supervisor_consult_receipt",
        ] {
            assert!(commands.contains(&format!("fn {command}")));
            assert!(registry.contains(command));
        }
        assert!(secretary.contains("M4SecretaryApplicationService::new("));
        assert!(secretary.contains(".request_handoff(&m4_request)"));
        assert!(secretary.contains(".read_handoff_receipt(&handoff_ref)"));
        assert!(lib.contains("mod m6_org_consult_handoff;"));
    }
}
