//! Ordinary-product dispatcher from the durable owner outbox into M4.
//!
//! The owner database and M4 database are deliberately separate.  Delivery is
//! therefore at-least-once: a lease is claimed in the owner DB, M4 performs its
//! own event/hash dedupe, and only the resulting M4 receipt advances the owner
//! checkpoint.  A crash between those last two steps safely replays the same
//! sealed publication.

use crate::m4_secretary_domain::{
    M4AttentionSignals, M4DecisionOwnerStatus, M4RegisteredPublicationKind,
    M4RegisteredSourcePublication, M4SourceStatus,
};
use crate::m4_secretary_repository::{
    M4RegisteredSourcePublicationOutcome, M4SecretaryRepositoryError,
    M4SecretarySqliteRepository,
};
use crate::m4_source_owner_schema::{
    map_proposal_owner_status, M4ClaimedSourceOwnerPublicationV1, M4SourceOwnerClaimOutcomeV1,
    M4SourceOwnerRetryOutcomeV1, M4SourceOwnerTerminalStatusV1,
    RegisteredWorkItemSourceOwnerMapper, M4_PROPOSAL_DECISION_SOURCE_ADAPTER_ID,
    M4_WORK_ITEM_SOURCE_ADAPTER_ID,
};
use crate::workbench_sqlite_repository::WorkbenchSqliteRepository;

pub(crate) const M4_SOURCE_DISPATCH_DEFAULT_BATCH_LIMIT: usize = 64;

/// The concrete checkpoint marker required by the remediation contract.  The
/// durable row itself lives in the owner overlay and is keyed per adapter.
pub(crate) struct M4WorkItemSourceConsumerCheckpointV1;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct M4SourceDispatchBatchOutcome {
    pub(crate) claimed_count: usize,
    pub(crate) delivered_count: usize,
    pub(crate) quarantined_count: usize,
    pub(crate) retry_scheduled_count: usize,
    pub(crate) replayed_count: usize,
    pub(crate) drained: bool,
}

pub(crate) fn dispatch_pending_m4_source_owner_outbox(
    owner_repository: &WorkbenchSqliteRepository,
    m4_repository: &M4SecretarySqliteRepository,
    claimer_id: &str,
    max_publications: usize,
) -> Result<M4SourceDispatchBatchOutcome, String> {
    let limit = max_publications.clamp(1, 256);
    let mut batch = M4SourceDispatchBatchOutcome::default();
    for _ in 0..limit {
        let claim = owner_repository
            .claim_next_m4_source_owner_publication(claimer_id, crate::unix_timestamp_ms())?;
        let claim = match claim {
            M4SourceOwnerClaimOutcomeV1::Idle => {
                batch.drained = true;
                break;
            }
            M4SourceOwnerClaimOutcomeV1::Quarantined { .. } => {
                batch.quarantined_count += 1;
                continue;
            }
            M4SourceOwnerClaimOutcomeV1::Claimed(claim) => claim,
        };
        batch.claimed_count += 1;

        let publication = match map_claimed_publication(&claim) {
            Ok(publication) => publication,
            Err(error) => {
                owner_repository.quarantine_claimed_m4_source_owner_publication(
                    &claim,
                    &error,
                    crate::unix_timestamp_ms(),
                )?;
                batch.quarantined_count += 1;
                continue;
            }
        };

        let ingestion = match ingest_workflow_attention_source_from_registered_publication(
            m4_repository,
            &publication,
        ) {
            Ok(ingestion) => ingestion,
            Err(error) => {
                match owner_repository.record_m4_source_owner_publication_retry(
                    &claim,
                    &error.code,
                    crate::unix_timestamp_ms(),
                )? {
                    M4SourceOwnerRetryOutcomeV1::Scheduled { .. } => {
                        batch.retry_scheduled_count += 1;
                    }
                    M4SourceOwnerRetryOutcomeV1::Quarantined { .. } => {
                        batch.quarantined_count += 1;
                    }
                }
                continue;
            }
        };
        if ingestion.ingestion.replayed {
            batch.replayed_count += 1;
        }
        match ingestion.ingestion.disposition.as_str() {
            "ADMITTED" => {
                owner_repository.mark_m4_source_owner_publication_terminal(
                    &claim,
                    M4SourceOwnerTerminalStatusV1::Delivered,
                    &ingestion.ingestion.ingestion_receipt_id,
                    None,
                    crate::unix_timestamp_ms(),
                )?;
                batch.delivered_count += 1;
            }
            "QUARANTINED" => {
                owner_repository.mark_m4_source_owner_publication_terminal(
                    &claim,
                    M4SourceOwnerTerminalStatusV1::Quarantined,
                    &ingestion.ingestion.ingestion_receipt_id,
                    Some(&ingestion.ingestion.outcome_code),
                    crate::unix_timestamp_ms(),
                )?;
                batch.quarantined_count += 1;
            }
            disposition => {
                match owner_repository.record_m4_source_owner_publication_retry(
                    &claim,
                    &format!("m4_ingestion_disposition_unrecognized:{disposition}"),
                    crate::unix_timestamp_ms(),
                )? {
                    M4SourceOwnerRetryOutcomeV1::Scheduled { .. } => {
                        batch.retry_scheduled_count += 1;
                    }
                    M4SourceOwnerRetryOutcomeV1::Quarantined { .. } => {
                        batch.quarantined_count += 1;
                    }
                }
            }
        }
    }
    Ok(batch)
}

/// Keep the production adapter edge explicit: the dispatcher can only enter
/// M4 through the registered-publication transaction, never through a fixture
/// or a direct projection writer.
fn ingest_workflow_attention_source_from_registered_publication(
    m4_repository: &M4SecretarySqliteRepository,
    publication: &M4RegisteredSourcePublication,
) -> Result<M4RegisteredSourcePublicationOutcome, M4SecretaryRepositoryError> {
    m4_repository.ingest_registered_source_publication(publication)
}

fn map_claimed_publication(
    claim: &M4ClaimedSourceOwnerPublicationV1,
) -> Result<M4RegisteredSourcePublication, String> {
    validate_work_item_source_provenance(claim)?;
    let owner = &claim.publication;
    let (publication_kind, mapped, decision_owner_status) = if owner.adapter_id
        == M4_WORK_ITEM_SOURCE_ADAPTER_ID
        && owner.publication_kind == "WORK_ITEM_ATTENTION"
    {
        let mapped = RegisteredWorkItemSourceOwnerMapper::map(&owner.owner_status_code)?;
        (M4RegisteredPublicationKind::WorkItemAttention, mapped, None)
    } else if owner.adapter_id == M4_PROPOSAL_DECISION_SOURCE_ADAPTER_ID
        && owner.publication_kind == "PROPOSAL_DECISION"
    {
        let mapped = map_proposal_owner_status(&owner.owner_status_code)?;
        let decision_status = match owner.owner_status_code.as_str() {
            "draft" | "pending_user_confirmation" => M4DecisionOwnerStatus::Open,
            "user_confirmed" | "changes_requested" | "rejected" => M4DecisionOwnerStatus::Answered,
            "superseded" => M4DecisionOwnerStatus::Withdrawn,
            "expired" => M4DecisionOwnerStatus::Expired,
            _ => return Err("m4_proposal_owner_status_unregistered".to_string()),
        };
        (
            M4RegisteredPublicationKind::ProposalDecision,
            mapped,
            Some(decision_status),
        )
    } else {
        return Err("m4_source_owner_dispatch_adapter_binding_invalid".to_string());
    };
    let source_status = M4SourceStatus::parse(mapped.source_status_code)
        .ok_or_else(|| "m4_source_owner_dispatch_status_mapping_invalid".to_string())?;
    if mapped.attention != owner.attention {
        return Err("m4_source_owner_dispatch_attention_mapping_drift".to_string());
    }
    Ok(M4RegisteredSourcePublication {
        publication_sequence: claim.publication_sequence,
        publication_id: owner.publication_id.clone(),
        adapter_id: owner.adapter_id.clone(),
        publication_kind,
        native_scope_seal: owner.native_scope_seal.clone(),
        source_owner_ref: owner.source_owner_ref.clone(),
        source_object_type: owner.object_type.clone(),
        canonical_source_object_id: owner.canonical_object_id.clone(),
        source_revision: owner.source_revision,
        source_event_id: owner.source_event_id.clone(),
        source_owner_watermark: owner.source_owner_watermark.clone(),
        occurred_at_utc: owner.occurred_at_utc.clone(),
        source_status,
        decision_owner_status,
        attention_signals: M4AttentionSignals {
            external_commitment: owner.attention.external_commitment,
            time_sensitive: owner.attention.time_sensitive,
            requires_user_decision: owner.attention.requires_user_decision,
            source_blocked: owner.attention.source_blocked,
            attention_required: owner.attention.attention_required,
            material_change: owner.attention.material_change,
        },
        due_at_utc: owner.due_at_utc.clone(),
        opaque_route_ref: owner.opaque_route_ref.clone(),
        scrubbed_summary_ref: owner.scrubbed_summary_ref.clone(),
        payload_hash: owner.payload_hash.clone(),
    })
}

/// Revalidate the admitted WorkItem status/attention tuple at the consumer
/// boundary. Owner-side transaction validation proves the native tuple; this
/// second check prevents a drifted dispatcher mapping from entering M4.
fn validate_work_item_source_provenance(
    claim: &M4ClaimedSourceOwnerPublicationV1,
) -> Result<(), String> {
    let owner = &claim.publication;
    if owner.adapter_id != M4_WORK_ITEM_SOURCE_ADAPTER_ID {
        return Ok(());
    }
    if owner.publication_kind != "WORK_ITEM_ATTENTION" {
        return Err("m4_work_item_source_publication_kind_invalid".to_string());
    }
    let mapped = RegisteredWorkItemSourceOwnerMapper::map(&owner.owner_status_code)?;
    if mapped.attention != owner.attention {
        return Err("m4_work_item_source_provenance_attention_drift".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::m4_source_owner_schema::{
        M4SourceAttentionFlagsV1, M4SourceOwnerOutboxEnvelopeV1, M4_SOURCE_OWNER_ENVELOPE_SCHEMA,
        M4_WORK_ITEM_SOURCE_OWNER_REF,
    };

    fn claimed(
        status: &str,
        attention: M4SourceAttentionFlagsV1,
    ) -> M4ClaimedSourceOwnerPublicationV1 {
        M4ClaimedSourceOwnerPublicationV1 {
            publication_sequence: 7,
            expected_checkpoint_sequence: None,
            lease_token: "source-lease:sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            attempt_count: 1,
            publication: M4SourceOwnerOutboxEnvelopeV1 {
                schema_version: M4_SOURCE_OWNER_ENVELOPE_SCHEMA.to_string(),
                publication_id: "source-publication:sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string(),
                adapter_id: M4_WORK_ITEM_SOURCE_ADAPTER_ID.to_string(),
                publication_kind: "WORK_ITEM_ATTENTION".to_string(),
                owner_native_event_id: "019d6a63-847b-7000-8000-000000000001".to_string(),
                owner_native_watermark: "019d6a63-847b-7000-8000-000000000001".to_string(),
                owner_native_payload_hash: "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc".to_string(),
                source_event_id: "source-event:sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd".to_string(),
                source_owner_watermark: "source-watermark:sha256:eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".to_string(),
                native_scope_seal: "native-scope:sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".to_string(),
                source_owner_ref: M4_WORK_ITEM_SOURCE_OWNER_REF.to_string(),
                object_type: "workflow_attention".to_string(),
                canonical_object_id: "work-item:fixture".to_string(),
                source_revision: 1,
                owner_status_code: status.to_string(),
                attention,
                occurred_at_utc: "2026-08-11T00:00:00.000Z".to_string(),
                due_at_utc: None,
                opaque_route_ref: "source-route:sha256:1111111111111111111111111111111111111111111111111111111111111111".to_string(),
                scrubbed_summary_ref: "source-summary:sha256:2222222222222222222222222222222222222222222222222222222222222222".to_string(),
                payload_hash: "3333333333333333333333333333333333333333333333333333333333333333".to_string(),
            },
        }
    }

    #[test]
    fn dispatcher_uses_the_registered_work_item_mapper() {
        let mapped = RegisteredWorkItemSourceOwnerMapper::map("waiting_for_permission")
            .expect("registered map");
        let publication =
            map_claimed_publication(&claimed("waiting_for_permission", mapped.attention))
                .expect("map claimed publication");
        assert_eq!(publication.source_status, M4SourceStatus::WaitingUser);
        assert_eq!(publication.decision_owner_status, None);
    }
}
